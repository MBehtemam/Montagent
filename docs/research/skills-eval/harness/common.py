"""What every harness script shares: where things live, the pinned build, the pack, and
the signals read out of a run's transcript.

One commit pins everything a run uses. `pinned_build` exports that commit with
`git archive`, builds its `montagent` binary, and hands back the binary, the commit's
`skills/` directory and the commit's asset pack. A run never reads the working tree.
"""

from __future__ import annotations

import hashlib
import json
import os
import re
import shutil
import subprocess
import tarfile
import tempfile
from pathlib import Path

EVAL = Path(__file__).resolve().parents[1]
REPO = EVAL.parents[2]
RUNS = EVAL / "runs"
JUDGING = EVAL / "judging"
PACK_IN_REPO = "docs/research/skills-eval/assets"

ARMS = ("reference", "no-skills", "with-skills")
MONTAGENT_ARMS = ("no-skills", "with-skills")
PHASES = ("baseline", "dev", "verdict", "verdict-2")
# Phases that count: each has its rubric committed and its briefs sealed before its first run.
VERDICT_PHASES = ("verdict", "verdict-2")

# Built outside the repo: a pinned build is a cache, never evidence.
CACHE = Path(os.environ.get("MONTAGENT_EVAL_CACHE", Path.home() / ".cache" / "montagent-skills-eval"))

MCP_PREFIX = "mcp__montagent__"


def sh(*args, cwd=None, check=True, capture=True, env=None) -> subprocess.CompletedProcess:
    return subprocess.run(
        [str(a) for a in args], cwd=cwd, check=check, env=env,
        stdout=subprocess.PIPE if capture else None,
        stderr=subprocess.PIPE if capture else None, text=True,
    )


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def sha256_text(text: str) -> str:
    return hashlib.sha256(text.encode()).hexdigest()


def write_json(path: Path, data) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n")


def read_json(path: Path):
    return json.loads(Path(path).read_text())


def rel(path: Path) -> str:
    """A path as the repo would spell it, or absolute if it lives outside the repo."""
    try:
        return str(Path(path).resolve().relative_to(REPO))
    except ValueError:
        return str(path)


# --- the pinned build -------------------------------------------------------------


def resolve_commit(rev: str) -> str:
    return sh("git", "rev-parse", "--verify", f"{rev}^{{commit}}", cwd=REPO).stdout.strip()


def pinned_build(rev: str = "HEAD") -> dict:
    """Export `rev`, build its binary once, and return what a run needs from it.

    Returns {"commit", "bin", "skills" (a Path, or None if the commit has no `skills/`),
    "pack", "version"}. Cached per commit under CACHE.
    """
    commit = resolve_commit(rev)
    root = CACHE / commit
    binary = root / "bin" / "montagent"
    src = root / "src"
    if not binary.exists():
        if src.exists():
            shutil.rmtree(src)
        src.mkdir(parents=True)
        with tempfile.TemporaryDirectory() as tmp:
            archive = Path(tmp) / "src.tar"
            sh("git", "archive", "--format=tar", "-o", archive, commit, cwd=REPO)
            with tarfile.open(archive) as tar:
                tar.extractall(src, filter="data")
        print(f"building montagent at {commit[:12]} (once per commit)…", flush=True)
        sh("cargo", "build", "--release", "--locked", "-p", "montagent", cwd=src, capture=False)
        binary.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(src / "target" / "release" / "montagent", binary)
        # The build tree is gigabytes; the binary, the skills and the pack are all a run reads.
        shutil.rmtree(src / "target")
    skills = src / "skills"
    return {
        "commit": commit,
        "bin": binary,
        "skills": skills if skills.is_dir() else None,
        "pack": src / PACK_IN_REPO,
        "version": sh(binary, "--version").stdout.strip(),
    }


def skill_names(skills_dir: Path | None) -> list[str]:
    if skills_dir is None:
        return []
    return sorted(p.parent.name for p in skills_dir.glob("*/SKILL.md"))


# --- briefs -------------------------------------------------------------------------


def brief_id(path: Path) -> str:
    return Path(path).stem


def brief_duration_ms(text: str) -> int:
    """The declared length on a brief's first bold line: `**6 seconds · 1080×1080 …`."""
    m = re.search(r"\*\*\s*(\d+(?:\.\d+)?)\s*seconds?\b", text)
    if not m:
        raise ValueError("brief declares no length (expected `**<n> seconds · …`)")
    return round(float(m.group(1)) * 1000)


# --- media --------------------------------------------------------------------------


def ffprobe(path: Path) -> dict | None:
    """Duration and picture size, or None if the file does not decode as video."""
    r = sh("ffprobe", "-v", "error", "-print_format", "json", "-show_format", "-show_streams",
           path, check=False)
    if r.returncode != 0:
        return None
    data = json.loads(r.stdout)
    video = [s for s in data.get("streams", []) if s.get("codec_type") == "video"]
    if not video:
        return None
    v = video[0]
    return {
        "duration_s": round(float(data["format"].get("duration", 0)), 3),
        "width": v.get("width"),
        "height": v.get("height"),
        "audio": any(s.get("codec_type") == "audio" for s in data["streams"]),
    }


def encode_720p(src: Path, dst: Path) -> None:
    """The judged copy: short side 720, H.264, AAC if there is sound. Same for every arm."""
    dst.parent.mkdir(parents=True, exist_ok=True)
    sh("ffmpeg", "-v", "error", "-y", "-i", src,
       "-vf", "scale='if(lt(iw,ih),720,-2)':'if(lt(iw,ih),-2,720)':flags=lanczos,format=yuv420p",
       "-c:v", "libx264", "-preset", "slow", "-crf", "20",
       "-c:a", "aac", "-b:a", "160k", "-movflags", "+faststart", dst)


# --- transcript signals -------------------------------------------------------------

LOOK_VERBS = ("frame", "preview")
IMAGE_RE = re.compile(r"\.(png|jpe?g|webp|gif)$", re.I)


def read_jsonl(path: Path) -> list[dict]:
    out = []
    for line in Path(path).read_text().splitlines():
        line = line.strip()
        if line:
            try:
                out.append(json.loads(line))
            except json.JSONDecodeError:
                pass
    return out


def tool_uses(events: list[dict]):
    for e in events:
        if e.get("type") != "assistant":
            continue
        for block in e.get("message", {}).get("content", []) or []:
            if isinstance(block, dict) and block.get("type") == "tool_use":
                yield block


def montagent_verbs(block: dict) -> list[str]:
    """Every Montagent verb a tool call ran: one over MCP, or each CLI call in a Bash
    command, since agents chain several (`montagent validate p && montagent render p`)."""
    name = block.get("name", "")
    if name.startswith(MCP_PREFIX):
        return [name[len(MCP_PREFIX):]]
    if name == "Bash":
        return re.findall(r"(?:^|[\s;&|(/])montagent\s+([a-z][a-z-]*)", block.get("input", {}).get("command", ""))
    return []


def transcript_signals(events: list[dict], our_skills: list[str]) -> dict:
    """What the transcript says about process: isolation, skills, verbs, looks, cost."""
    init = next((e for e in events if e.get("type") == "system" and e.get("subtype") == "init"), {})
    result = next((e for e in reversed(events) if e.get("type") == "result"), {})

    verbs: dict[str, int] = {}
    skill_calls: list[str] = []
    skill_reads: list[str] = []
    image_reads = 0
    tool_counts: dict[str, int] = {}
    for b in tool_uses(events):
        tool_counts[b.get("name", "?")] = tool_counts.get(b.get("name", "?"), 0) + 1
        for verb in montagent_verbs(b):
            verbs[verb] = verbs.get(verb, 0) + 1
        inp = b.get("input", {}) or {}
        if b.get("name") == "Skill":
            skill_calls.append(str(inp.get("skill", inp.get("name", ""))))
        if b.get("name") == "Read":
            p = str(inp.get("file_path", ""))
            m = re.search(r"\.claude/skills/([^/]+)/", p)
            if m:
                skill_reads.append(m.group(1))
            if IMAGE_RE.search(p):
                image_reads += 1

    # A background task still running when a headless session ends is killed: a render the
    # agent left in the background never reaches the deliverable.
    killed = sum(1 for e in events if e.get("type") == "system" and e.get("subtype") == "task_updated"
                 and (e.get("patch") or {}).get("status") == "killed")

    triggered = sorted({s.split(":")[-1] for s in skill_calls + skill_reads} & set(our_skills))
    usage = result.get("usage", {}) or {}
    looks = sum(verbs.get(v, 0) for v in LOOK_VERBS)
    return {
        "init": {
            "model": init.get("model"),
            "claude_code_version": init.get("claude_code_version"),
            "mcp_servers": sorted(s.get("name") for s in init.get("mcp_servers", [])),
            "plugins": sorted(p.get("source", p.get("name")) for p in init.get("plugins", [])),
            "skills": sorted(init.get("skills", [])),
            "tools": sorted(init.get("tools", [])),
            "api_key_source": init.get("apiKeySource"),
        },
        "skills_installed": sorted(set(init.get("skills", [])) & set(our_skills)),
        "skills_triggered": triggered,
        "skill_tool_calls": skill_calls,
        "montagent_verbs": dict(sorted(verbs.items())),
        "look_loops": looks,
        "image_reads": image_reads,
        "looked_at": looks > 0 or image_reads > 0,
        "tool_calls": dict(sorted(tool_counts.items())),
        "background_tasks_killed": killed,
        "result": {
            "subtype": result.get("subtype"),
            "is_error": result.get("is_error"),
            "num_turns": result.get("num_turns"),
            "duration_ms": result.get("duration_ms"),
            "total_cost_usd": result.get("total_cost_usd"),
            "input_tokens": usage.get("input_tokens"),
            "output_tokens": usage.get("output_tokens"),
            "cache_read_input_tokens": usage.get("cache_read_input_tokens"),
            "cache_creation_input_tokens": usage.get("cache_creation_input_tokens"),
        },
    }


def load_pins(phase: str | None = None) -> dict:
    """`pins-v2.json` for the second verdict; `pins.json`, frozen, for every earlier phase.

    The first verdict's file predates `rubric`, `sealed_briefs` and the required commit, so
    they are filled in here; `pins-v2.json` states its own."""
    if phase == "verdict-2":
        return read_json(EVAL / "harness" / "pins-v2.json")
    return {"rubric": "RUBRIC.md", "sealed_briefs": "briefs/held-out.sha256",
            "montagent_commit_contains": None, **read_json(EVAL / "harness" / "pins.json")}


def sealed_hashes(pins: dict) -> dict[str, str]:
    """{sha256: brief id} from the phase's sealed-briefs file."""
    path = EVAL / pins["sealed_briefs"]
    if not path.exists():
        return {}
    return {sha: brief_id(Path(name)) for sha, name in
            (line.split() for line in path.read_text().splitlines() if line.strip())}


def commit_contains(commit: str, ancestor: str) -> bool:
    return sh("git", "merge-base", "--is-ancestor", ancestor, commit, cwd=REPO, check=False).returncode == 0


def montagent_signals(binary: Path, project: Path) -> dict:
    """`validate` counts and reach (`query --census type` over every element) for one
    project file, run where it sits so its relative paths resolve."""
    def run(*args):
        r = sh(binary, *args, "--json", project.name, cwd=project.parent, check=False)
        try:
            return json.loads(r.stdout)
        except json.JSONDecodeError:
            return {"exit_code": r.returncode, "unparsed": (r.stdout + r.stderr)[-2000:]}

    v = run("validate")
    q = run("query", "--where", "id exists", "--census", "type")
    groups = (q.get("query") or {}).get("census", {}).get("groups", [])
    codes: dict[str, int] = {}
    for f in v.get("findings", []):
        codes[f["code"]] = codes.get(f["code"], 0) + 1
    return {
        "validate": v.get("summary"),
        "validate_exit": v.get("exit_code"),
        "finding_codes": dict(sorted(codes.items())),
        "census_type": {g["value"]: len(g["members"]) for g in groups},
    }


def isolation_problems(sig: dict, arm: str, our_skills: list[str], builtin_skills: list[str]) -> list[str]:
    """Anything in the session's init that a stranger with this arm's setup would not have."""
    init = sig["init"]
    problems = []
    want_mcp = ["montagent"] if arm in MONTAGENT_ARMS else []
    if init["mcp_servers"] != want_mcp:
        problems.append(f"MCP servers {init['mcp_servers']}, expected {want_mcp}")
    extra_plugins = [p for p in init["plugins"] if not str(p).endswith("@builtin")]
    if extra_plugins:
        problems.append(f"non-builtin plugins loaded: {extra_plugins}")
    allowed = set(builtin_skills) | (set(our_skills) if arm == "with-skills" else set())
    extra = sorted(set(init["skills"]) - allowed)
    if extra:
        problems.append(f"skills beyond the built-ins and this arm's: {extra}")
    if arm == "with-skills" and sig["skills_installed"] != sorted(our_skills):
        problems.append(f"installed skills {sig['skills_installed']}, expected {sorted(our_skills)}")
    web = sorted({"WebFetch", "WebSearch"} & set(init["tools"]))
    if web:
        problems.append(f"web tools available: {web}")
    return problems
