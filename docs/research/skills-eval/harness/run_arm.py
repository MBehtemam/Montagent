"""Run one arm of the skills eval on one brief, end to end, and record it.

    uv run docs/research/skills-eval/harness/run_arm.py --brief <brief.md> --arm <arm> [--phase baseline]

Arms: `reference` (no Montagent), `no-skills` (Montagent), `with-skills` (Montagent plus the
commit's `skills/`). The run happens in a fresh directory outside the repo that holds only
the pinned commit's asset pack (plus, in `with-skills`, the skills in `.claude/skills/`).
Claude Code loads only that directory's settings (`--setting-sources project`) and only the
Montagent MCP server (`--strict-mcp-config`): no user skills, plugins, settings or MCP servers,
and auto-memory starts empty because it is keyed to the new directory. The run's own init
event records what loaded, and anything beyond the arm's setup is an isolation problem.
The record lands in `runs/<phase>/<brief>/<arm>-<n>/`:

- `manifest.json`: every pin, the exact prompt, caps, timings, and the run-time signals
- `transcript.jsonl`: Claude Code's stream-json output, verbatim
- `stderr.log`: Claude Code's stderr
- `workspace/`: every file the agent made or changed (the project file, or the reference's
  source), minus caches and anything over 2 MB, which the manifest lists by hash instead
- `render.mp4`: the deliverable, re-encoded to 720p (short side), if one was delivered

It uses the machine's normal Claude Code login. It refuses to run while a global
`~/.claude/CLAUDE.md` exists, since nothing in the transcript would show it was loaded.

`check` proves the sandbox with one cheap Haiku session (web blocked, registries reachable,
home not writable); run it before verdict runs. `probe` prints what an isolated session
loads, so `pins.json`'s `builtin_skills` can be refreshed when Claude Code's pin moves.
"""

# /// script
# requires-python = ">=3.11"
# ///

from __future__ import annotations

import argparse
import contextlib
import datetime as dt
import fcntl
import json
import os
import shutil
import signal
import subprocess
import sys
import tempfile
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from common import (  # noqa: E402
    ARMS, CACHE, EVAL, MONTAGENT_ARMS, PHASES, REPO, RUNS, brief_id, encode_720p, ffprobe,
    isolation_problems, load_pins, tool_uses, montagent_signals, pinned_build, read_jsonl, rel, sh,
    sha256_file, sha256_text, skill_names, transcript_signals, write_json,
)

MAX_KEPT_BYTES = 2 * 1024 * 1024
# Where tool caches go inside the sandbox-writable work dir; never copied into the record.
CACHE_DIR = ".cache"
SKIP_DIRS = {".claude", CACHE_DIR, "out", "node_modules", ".venv", "venv", "__pycache__", ".git"}


def now() -> str:
    return dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds")


GLOBAL_CLAUDE_MD = Path.home() / ".claude" / "CLAUDE.md"


def check_no_global_instructions() -> None:
    if GLOBAL_CLAUDE_MD.exists():
        sys.exit(f"{GLOBAL_CLAUDE_MD} exists; move it aside for the eval, since a run cannot "
                 "show whether it was read")


def path_without_montagent() -> list[str]:
    return [d for d in os.environ.get("PATH", "").split(os.pathsep)
            if d and not (Path(d) / "montagent").exists()]


def build_env(scratch: Path, work: Path, arm: str, build: dict | None) -> dict:
    path = path_without_montagent()
    if arm in MONTAGENT_ARMS:
        path.insert(0, str(build["bin"].parent))
    cache = work / CACHE_DIR
    env = {
        "HOME": os.environ["HOME"],
        "USER": os.environ.get("USER", ""),
        "LANG": os.environ.get("LANG", "en_US.UTF-8"),
        "TERM": "dumb",
        # The login lives in the keychain, found through HOME and USER.
        "PATH": os.pathsep.join(path),
        "TMPDIR": str(scratch / "tmp"),
        "DISABLE_AUTOUPDATER": "1",
        # Tool caches are written inside the work dir, because the sandbox lets Bash write
        # nowhere else; and a run must not inherit another run's cache.
        "MONTAGENT_CACHE_DIR": str(cache / "montagent"),
        "npm_config_cache": str(cache / "npm"),
        "UV_CACHE_DIR": str(cache / "uv"),
        "PIP_CACHE_DIR": str(cache / "pip"),
        "XDG_CACHE_HOME": str(cache / "xdg"),
    }
    # Browsers already downloaded on this machine stay readable, so a reference build does
    # not have to fetch Chromium through the registry allowlist.
    pw = Path(os.environ["HOME"]) / "Library" / "Caches" / "ms-playwright"
    if pw.is_dir():
        env["PLAYWRIGHT_BROWSERS_PATH"] = str(pw)
    return env


def settings(pins: dict) -> dict:
    return {
        "permissions": {"deny": ["WebFetch", "WebSearch"]},
        "sandbox": {
            "enabled": True,
            "autoAllowBashIfSandboxed": True,
            "allowUnsandboxedCommands": False,
            "network": {"allowedDomains": pins["network"]["allowed_domains"]},
        },
    }


def claude_argv(pins: dict, arm: str, prompt: str, build: dict | None) -> list[str]:
    argv = [
        "claude", "-p", prompt,
        "--model", pins["model"],
        "--effort", pins["effort"],
        "--output-format", "stream-json", "--verbose",
        "--max-budget-usd", str(pins["caps"]["budget_usd"]),
        # Not bypassPermissions: that also approves the sandbox's "reach a new domain?"
        # prompt, so the network allowlist leaks. Here every prompt is denied instead.
        "--permission-mode", "dontAsk",
        "--permission-prompts", "none",
        "--allowedTools", "Bash", "Read", "Edit", "Write", "Glob", "Grep", "mcp__montagent",
        "--setting-sources", "project",
        "--settings", json.dumps(settings(pins)),
        "--disallowedTools", "WebFetch", "WebSearch",
        "--no-session-persistence",
        "--strict-mcp-config",
    ]
    if arm in MONTAGENT_ARMS:
        mcp = {"mcpServers": {"montagent": {"command": str(build["bin"]), "args": ["mcp"]}}}
        argv += ["--mcp-config", json.dumps(mcp)]
    return argv


def make_prompt(pins: dict, arm: str, brief_text: str) -> str:
    tools = pins["prompt"]["tools"]["reference" if arm == "reference" else "montagent"]
    return pins["prompt"]["common"].format(tools=tools, brief=brief_text.strip())


# Claude Code's sandbox points every session's TMPDIR at this one per-user directory, and no
# setting moves it. Runs that overlap would read each other's temp files (it happened: one
# baseline run executed another's `gen.py`), so runs hold a lock and never overlap, and
# whatever a run leaves at the top of this directory is swept into its own scratch.
SANDBOX_TMP = Path(f"/private/tmp/claude-{os.getuid()}")
LOCK = CACHE / "run.lock"


@contextlib.contextmanager
def one_run_at_a_time():
    LOCK.parent.mkdir(parents=True, exist_ok=True)
    with open(LOCK, "w") as f:
        try:
            fcntl.flock(f, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            print("another run holds the lock; waiting for it to finish…", flush=True)
            fcntl.flock(f, fcntl.LOCK_EX)
        yield


def sandbox_tmp_entries() -> set[str]:
    return {p.name for p in SANDBOX_TMP.iterdir()} if SANDBOX_TMP.is_dir() else set()


def sweep_sandbox_tmp(before: set[str], dest: Path) -> list[str]:
    """Move what appeared at the top of the shared sandbox temp dir into this run's scratch."""
    new = sorted(sandbox_tmp_entries() - before)
    dest.mkdir(parents=True, exist_ok=True)
    for name in new:
        shutil.move(str(SANDBOX_TMP / name), str(dest / name))
    return new


def run_claude(argv, env, cwd, transcript: Path, stderr: Path, wall_clock_s: int) -> dict:
    started = time.monotonic()
    with open(transcript, "w") as out, open(stderr, "w") as err:
        proc = subprocess.Popen(argv, cwd=cwd, env=env, stdout=out, stderr=err,
                                stdin=subprocess.DEVNULL, start_new_session=True)
        timed_out = False
        try:
            proc.wait(timeout=wall_clock_s)
        except subprocess.TimeoutExpired:
            timed_out = True
            os.killpg(proc.pid, signal.SIGTERM)
            try:
                proc.wait(timeout=30)
            except subprocess.TimeoutExpired:
                os.killpg(proc.pid, signal.SIGKILL)
                proc.wait()
    return {"exit_code": proc.returncode, "timed_out": timed_out,
            "wall_clock_s": round(time.monotonic() - started, 1)}


def keep_workspace(work: Path, pack: Path, dest: Path, deliverable: str) -> list[dict]:
    """Copy what the agent made or changed; return what was left out, by hash."""
    omitted = []
    for p in sorted(work.rglob("*")):
        if not p.is_file() or p.is_symlink():
            continue
        r = p.relative_to(work)
        if r.parts[0] in SKIP_DIRS or any(part in SKIP_DIRS for part in r.parts[:-1]):
            continue
        if str(r) == deliverable:
            continue
        original = pack / r
        if original.is_file() and original.stat().st_size == p.stat().st_size \
                and sha256_file(original) == sha256_file(p):
            continue
        if p.stat().st_size > MAX_KEPT_BYTES:
            omitted.append({"path": str(r), "bytes": p.stat().st_size, "sha256": sha256_file(p)})
            continue
        (dest / r).parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(p, dest / r)
    return omitted


def final_project(work: Path, deliverable: str) -> Path | None:
    """The project that renders the deliverable, else the one written last.

    A project is any JSON object with `frame` and `fps`, whatever its name: agents do not
    all use the `.montagent.json` suffix."""
    projects = []
    for p in work.rglob("*.json"):
        if any(part in SKIP_DIRS for part in p.relative_to(work).parts):
            continue
        try:
            doc = json.loads(p.read_text())
        except (json.JSONDecodeError, UnicodeDecodeError):
            continue
        if isinstance(doc, dict) and "frame" in doc and "fps" in doc:
            projects.append((p, doc))
    for p, doc in projects:
        out = doc.get("output")
        if isinstance(out, str) and (p.parent / out).resolve() == (work / deliverable).resolve():
            return p
    return max((p for p, _ in projects), key=lambda p: p.stat().st_mtime, default=None)


def next_index(parent: Path, arm: str) -> int:
    taken = [int(p.name.rsplit("-", 1)[1]) for p in parent.glob(f"{arm}-*") if p.name.rsplit("-", 1)[1].isdigit()]
    return max(taken, default=0) + 1


def check_verdict_preconditions(brief_sha: str) -> None:
    """A verdict run needs its rubric committed and its brief sealed by hash beforehand."""
    rubric = EVAL / "RUBRIC.md"
    tracked = sh("git", "ls-files", "--error-unmatch", rubric, cwd=REPO, check=False).returncode == 0
    clean = sh("git", "diff", "--quiet", "HEAD", "--", rubric, cwd=REPO, check=False).returncode == 0
    if not (tracked and clean):
        sys.exit("verdict runs need RUBRIC.md committed, unchanged, before any run")
    sealed = EVAL / "briefs" / "held-out.sha256"
    hashes = sealed.read_text().split() if sealed.exists() else []
    if brief_sha not in hashes:
        sys.exit(f"verdict runs need a held-out brief whose SHA-256 is in {rel(sealed)}")


def probe(pins: dict) -> None:
    with tempfile.TemporaryDirectory(prefix="montagent-eval-probe-") as tmp:
        scratch = Path(tmp)
        (scratch / "work").mkdir()
        (scratch / "tmp").mkdir()
        env = build_env(scratch, scratch / "work", "reference", None)
        argv = claude_argv({**pins, "model": "haiku"}, "reference", "Reply with: ok", None)
        r = subprocess.run(argv, cwd=scratch / "work", env=env, capture_output=True, text=True, timeout=300)
        init = next((e for e in map(json.loads, filter(None, r.stdout.splitlines()))
                     if e.get("subtype") == "init"), {})
        print(json.dumps({k: init.get(k) for k in ("claude_code_version", "skills", "plugins",
                                                    "mcp_servers", "tools", "memory_paths")}, indent=2))


CHECKS = [  # (what, command, passes if its output...)
    ("web is blocked", "curl -sS -m 20 -o /dev/null -w '%{http_code}' https://example.com",
     lambda out: "200" not in out),
    ("registries are reachable", "curl -sS -m 20 -o /dev/null -w '%{http_code}' https://registry.npmjs.org/playwright",
     lambda out: "200" in out),
    ("the pinned montagent is on PATH", "montagent --version", lambda out: "montagent" in out),
    ("the run directory is writable", "echo hi > made.txt && cat made.txt", lambda out: "hi" in out),
    ("home is not writable", "echo x > ~/montagent-eval-escape.txt",
     lambda out: not (Path.home() / "montagent-eval-escape.txt").exists()),
]


def check(pins: dict) -> None:
    """Prove the sandbox with one cheap Haiku session: each command's real output decides."""
    build = pinned_build("HEAD")
    with tempfile.TemporaryDirectory(prefix="montagent-eval-check-") as tmp:
        scratch = Path(tmp)
        work = scratch / "work"
        work.mkdir()
        (scratch / "tmp").mkdir()
        env = build_env(scratch, work, "no-skills", build)
        prompt = ("Run each of these commands with the Bash tool, one call per command, exactly as "
                  "written, then reply: done.\n" + "\n".join(c for _, c, _ in CHECKS))
        argv = claude_argv({**pins, "model": "haiku", "effort": "low"}, "no-skills", prompt, build)
        with one_run_at_a_time():
            r = subprocess.run(argv, cwd=work, env=env, capture_output=True, text=True, timeout=300)
    events = [json.loads(l) for l in r.stdout.splitlines() if l.strip().startswith("{")]
    commands = {b["id"]: b["input"].get("command", "") for b in tool_uses(events) if b.get("name") == "Bash"}
    outputs = {}
    for e in events:
        for c in (e.get("message", {}).get("content") or []) if e.get("type") == "user" else []:
            if isinstance(c, dict) and c.get("type") == "tool_result" and c.get("tool_use_id") in commands:
                outputs[commands[c["tool_use_id"]].strip()] = str(c.get("content"))
    ok = True
    for what, cmd, passes in CHECKS:
        out = outputs.get(cmd)
        good = out is not None and passes(out)
        ok &= good
        print(f"{'PASS' if good else 'FAIL'}  {what}: {'(not run)' if out is None else out.strip()[:120]}")
    (Path.home() / "montagent-eval-escape.txt").unlink(missing_ok=True)
    sys.exit(0 if ok else 1)


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--brief", type=Path, help="the brief's Markdown file")
    ap.add_argument("--arm", choices=ARMS)
    ap.add_argument("--phase", choices=PHASES, default="baseline")
    ap.add_argument("--commit", default="HEAD", help="the Montagent commit that pins binary, skills and pack")
    ap.add_argument("--keep", action="store_true", help="keep the scratch directory for inspection")
    ap.add_argument("--dry-run", action="store_true", help="set everything up and print the command; call no model")
    ap.add_argument("command", nargs="?", choices=["probe", "check"],
                    help="`probe`: print what an isolated Haiku session loads; `check`: prove the sandbox; then exit")
    args = ap.parse_args()

    pins = load_pins()
    if args.command == "probe":
        return probe(pins)
    if args.command == "check":
        return check(pins)
    if not (args.brief and args.arm):
        ap.error("--brief and --arm are required")

    version = sh("claude", "--version").stdout.split()[0]
    if version != pins["claude_code_version"]:
        sys.exit(f"Claude Code is {version}; pins.json pins {pins['claude_code_version']}. "
                 "Re-pin deliberately (and re-run `probe`) rather than mix versions.")
    check_no_global_instructions()

    brief_text = args.brief.read_text()
    brief_sha = sha256_text(brief_text)
    if args.phase == "verdict":
        check_verdict_preconditions(brief_sha)

    build = pinned_build(args.commit)
    ours = skill_names(build["skills"])
    if args.arm == "with-skills" and not ours:
        sys.exit(f"commit {build['commit'][:12]} has no skills/*/SKILL.md to install")

    bid = brief_id(args.brief)
    parent = RUNS / args.phase / bid
    record = parent / f"{args.arm}-{next_index(parent, args.arm)}"

    scratch = Path(tempfile.mkdtemp(prefix="montagent-eval-"))
    work = scratch / "work"
    shutil.copytree(build["pack"], work)
    (scratch / "tmp").mkdir()
    if args.arm == "with-skills":
        shutil.copytree(build["skills"], work / ".claude" / "skills")

    prompt = make_prompt(pins, args.arm, brief_text)
    argv = claude_argv(pins, args.arm, prompt, build)
    env = build_env(scratch, work, args.arm, build)
    if args.dry_run:
        print(json.dumps({"cwd": str(work), "argv": argv,
                          "env": env}, indent=2))
        print(f"scratch kept at {scratch}")
        return

    # Claim the run number atomically: parallel runs of one arm must not share a directory.
    while True:
        try:
            record.mkdir(parents=True)
            break
        except FileExistsError:
            record = parent / f"{args.arm}-{next_index(parent, args.arm)}"
    print(f"{args.arm} on {bid} → {rel(record)}  (scratch {scratch})", flush=True)
    with one_run_at_a_time():
        tmp_before = sandbox_tmp_entries()
        started = now()
        outcome = run_claude(argv, env, work, record / "transcript.jsonl", record / "stderr.log",
                             pins["caps"]["wall_clock_s"])
        ended = now()
        swept = sweep_sandbox_tmp(tmp_before, scratch / "sandbox-tmp")

    deliverable = work / pins["deliverable"]
    probe_info = ffprobe(deliverable) if deliverable.exists() else None
    if probe_info:
        encode_720p(deliverable, record / "render.mp4")
    omitted = keep_workspace(work, build["pack"], record / "workspace", pins["deliverable"])

    events = read_jsonl(record / "transcript.jsonl")
    sig = transcript_signals(events, ours)
    project = final_project(work, pins["deliverable"]) if args.arm in MONTAGENT_ARMS else None

    manifest = {
        "brief": {"id": bid, "sha256": brief_sha, "path": rel(args.brief)},
        "arm": args.arm,
        "phase": args.phase,
        "run": record.name,
        "pins": {
            "model": pins["model"],
            "effort": pins["effort"],
            "claude_code_version": version,
            "montagent_commit": build["commit"],
            "montagent_version": build["version"],
            "skills": ours if args.arm == "with-skills" else [],
            "caps": pins["caps"],
            "allowed_domains": pins["network"]["allowed_domains"],
            "web_tools": "removed (WebFetch, WebSearch)",
            "setting_sources": "project",
        },
        "prompt": prompt,
        "prompt_sha256": sha256_text(prompt),
        "started": started,
        "ended": ended,
        **outcome,
        "deliverable": probe_info,
        "project": str(project.relative_to(work)) if project else None,
        "omitted_files": omitted,
        "sandbox_tmp_swept": swept,
        "isolation_problems": isolation_problems(sig, args.arm, ours, pins["builtin_skills"]),
        "signals": {
            **sig,
            "montagent": montagent_signals(build["bin"], project) if project else None,
        },
    }
    write_json(record / "manifest.json", manifest)

    if args.keep:
        print(f"scratch kept at {scratch}")
    else:
        shutil.rmtree(scratch, ignore_errors=True)

    r = manifest["signals"]["result"]
    print(json.dumps({
        "record": rel(record),
        "delivered": bool(probe_info),
        "timed_out": outcome["timed_out"],
        "result": r["subtype"],
        "turns": r["num_turns"],
        "cost_usd": r["total_cost_usd"],
        "wall_clock_s": outcome["wall_clock_s"],
        "skills_triggered": sig["skills_triggered"],
        "isolation_problems": manifest["isolation_problems"],
    }, indent=2))


if __name__ == "__main__":
    main()
