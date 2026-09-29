"""Run one arm of the skills eval on one brief, end to end, and record it.

    uv run docs/research/skills-eval/harness/run_arm.py --brief <brief.md> --arm <arm> [--phase baseline]

Arms: `reference` (no Montagent), `no-skills` (Montagent), `with-skills` (Montagent plus the
commit's `skills/`). The run happens in a fresh directory outside the repo that holds only
the pinned commit's asset pack (plus, in `with-skills`, the skills in `.claude/skills/`),
under a fresh Claude Code config directory: no user skills, plugins, memory, settings or
global CLAUDE.md. The record lands in `runs/<phase>/<brief>/<arm>-<n>/`:

- `manifest.json`: every pin, the exact prompt, caps, timings, and the run-time signals
- `transcript.jsonl`: Claude Code's stream-json output, verbatim
- `stderr.log`: Claude Code's stderr
- `workspace/`: every file the agent made or changed (the project file, or the reference's
  source), minus caches and anything over 2 MB, which the manifest lists by hash instead
- `render.mp4`: the deliverable, re-encoded to 720p (short side), if one was delivered

Needs `CLAUDE_CODE_OAUTH_TOKEN` (from `claude setup-token`) or `ANTHROPIC_API_KEY`: the
fresh config directory has no login, which is the point.

`probe` prints what a fresh, isolated session loads, without calling the model, so
`pins.json`'s `builtin_skills` can be refreshed when Claude Code's pin moves.
"""

# /// script
# requires-python = ">=3.11"
# ///

from __future__ import annotations

import argparse
import datetime as dt
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
    ARMS, EVAL, MONTAGENT_ARMS, PHASES, REPO, RUNS, brief_id, encode_720p, ffprobe,
    isolation_problems, load_pins, montagent_signals, pinned_build, read_jsonl, rel, sh,
    sha256_file, sha256_text, skill_names, transcript_signals, write_json,
)

MAX_KEPT_BYTES = 2 * 1024 * 1024
# Where tool caches go inside the sandbox-writable work dir; never copied into the record.
CACHE_DIR = ".cache"
SKIP_DIRS = {".claude", CACHE_DIR, "out", "node_modules", ".venv", "venv", "__pycache__", ".git"}


def now() -> str:
    return dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds")


TOKEN_FILE = Path.home() / ".config" / "montagent-eval" / "oauth-token"


def auth_env() -> tuple[str, dict]:
    if os.environ.get("CLAUDE_CODE_OAUTH_TOKEN"):
        return "oauth-token", {"CLAUDE_CODE_OAUTH_TOKEN": os.environ["CLAUDE_CODE_OAUTH_TOKEN"]}
    if os.environ.get("ANTHROPIC_API_KEY"):
        return "api-key", {"ANTHROPIC_API_KEY": os.environ["ANTHROPIC_API_KEY"]}
    if TOKEN_FILE.is_file():
        return "oauth-token", {"CLAUDE_CODE_OAUTH_TOKEN": TOKEN_FILE.read_text().strip()}
    sys.exit(f"no credentials: export CLAUDE_CODE_OAUTH_TOKEN or ANTHROPIC_API_KEY, or save the "
             f"token `claude setup-token` prints to {TOKEN_FILE}. The run's config directory is "
             "fresh, so it has no login.")


def path_without_montagent() -> list[str]:
    return [d for d in os.environ.get("PATH", "").split(os.pathsep)
            if d and not (Path(d) / "montagent").exists()]


def build_env(scratch: Path, work: Path, arm: str, build: dict | None, auth: dict) -> dict:
    path = path_without_montagent()
    if arm in MONTAGENT_ARMS:
        path.insert(0, str(build["bin"].parent))
    cache = work / CACHE_DIR
    env = {
        "HOME": os.environ["HOME"],
        "USER": os.environ.get("USER", ""),
        "LANG": os.environ.get("LANG", "en_US.UTF-8"),
        "TERM": "dumb",
        "PATH": os.pathsep.join(path),
        "TMPDIR": str(scratch / "tmp"),
        "CLAUDE_CONFIG_DIR": str(scratch / "config"),
        "DISABLE_AUTOUPDATER": "1",
        # Tool caches are written inside the work dir, because the sandbox lets Bash write
        # nowhere else; and a run must not inherit another run's cache.
        "MONTAGENT_CACHE_DIR": str(cache / "montagent"),
        "npm_config_cache": str(cache / "npm"),
        "UV_CACHE_DIR": str(cache / "uv"),
        "PIP_CACHE_DIR": str(cache / "pip"),
        "XDG_CACHE_HOME": str(cache / "xdg"),
        **auth,
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
        "--permission-mode", "bypassPermissions",
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
    """The project that renders the deliverable, else the one written last."""
    projects = [p for p in work.rglob("*.montagent.json")
                if not any(part in SKIP_DIRS for part in p.relative_to(work).parts)]
    for p in projects:
        try:
            out = json.loads(p.read_text()).get("output")
        except (json.JSONDecodeError, AttributeError):
            continue
        if out and (p.parent / out).resolve() == (work / deliverable).resolve():
            return p
    return max(projects, key=lambda p: p.stat().st_mtime, default=None)


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
        env = build_env(scratch, scratch / "work", "reference", None, {})
        r = subprocess.run(claude_argv(pins, "reference", "probe", None), cwd=scratch / "work",
                           env=env, capture_output=True, text=True, timeout=120)
        init = next((e for e in map(json.loads, filter(None, r.stdout.splitlines()))
                     if e.get("subtype") == "init"), {})
        print(json.dumps({k: init.get(k) for k in ("claude_code_version", "skills", "plugins",
                                                    "mcp_servers", "tools", "apiKeySource")}, indent=2))


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--brief", type=Path, help="the brief's Markdown file")
    ap.add_argument("--arm", choices=ARMS)
    ap.add_argument("--phase", choices=PHASES, default="baseline")
    ap.add_argument("--commit", default="HEAD", help="the Montagent commit that pins binary, skills and pack")
    ap.add_argument("--keep", action="store_true", help="keep the scratch directory for inspection")
    ap.add_argument("--dry-run", action="store_true", help="set everything up and print the command; call no model")
    ap.add_argument("probe", nargs="?", help="`probe`: print what an isolated session loads, then exit")
    args = ap.parse_args()

    pins = load_pins()
    if args.probe == "probe":
        return probe(pins)
    if not (args.brief and args.arm):
        ap.error("--brief and --arm are required")

    version = sh("claude", "--version").stdout.split()[0]
    if version != pins["claude_code_version"]:
        sys.exit(f"Claude Code is {version}; pins.json pins {pins['claude_code_version']}. "
                 "Re-pin deliberately (and re-run `probe`) rather than mix versions.")
    auth_kind, auth = ("none", {}) if args.dry_run else auth_env()

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
    (scratch / "config").mkdir()
    (scratch / "tmp").mkdir()
    if args.arm == "with-skills":
        shutil.copytree(build["skills"], work / ".claude" / "skills")

    prompt = make_prompt(pins, args.arm, brief_text)
    argv = claude_argv(pins, args.arm, prompt, build)
    env = build_env(scratch, work, args.arm, build, auth)
    if args.dry_run:
        print(json.dumps({"cwd": str(work), "argv": argv,
                          "env": {k: v for k, v in env.items() if "TOKEN" not in k and "KEY" not in k}}, indent=2))
        print(f"scratch kept at {scratch}")
        return

    record.mkdir(parents=True)
    print(f"{args.arm} on {bid} → {rel(record)}  (scratch {scratch})", flush=True)
    started = now()
    outcome = run_claude(argv, env, work, record / "transcript.jsonl", record / "stderr.log",
                         pins["caps"]["wall_clock_s"])
    ended = now()

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
            "auth": auth_kind,
        },
        "prompt": prompt,
        "prompt_sha256": sha256_text(prompt),
        "started": started,
        "ended": ended,
        **outcome,
        "deliverable": probe_info,
        "project": str(project.relative_to(work)) if project else None,
        "omitted_files": omitted,
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
