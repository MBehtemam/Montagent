#!/usr/bin/env python3
"""Run one arm of the discovery evaluation on one task, end to end, and record it.

    uv run docs/research/discovery-eval/harness/run_arm.py --task <task_id> --arm <arm> [--phase pilot-smoke]

Arms:
- `cli-bare`: Clean-room workspace, montagent in PATH, zero skills, no MCP.
- `cli-skills`: Clean-room workspace, montagent in PATH, Montagent skill suite installed, no MCP.
- `mcp-bare`: Clean-room workspace, stdio MCP connection to Montagent server, file tools, zero skills, binary omitted from PATH.
- `mcp-skills`: Clean-room workspace, stdio MCP connection to Montagent server, file tools, complete Montagent skill suite installed, binary omitted from PATH.

Caps & Stopping Rules:
- Max wall-clock time: 300s (SIGTERM, SIGKILL after 15s)
- Max agent turns: 25 turns
- Max context tokens: 100,000 tokens
- Max budget: $2.00 / run
- Active abort on 5 consecutive identical errors (ERGONOMIC_FRICTION / BUDGET_EXHAUSTION)
- Active abort on 3 ungrounded hallucinations (MODEL_HALLUCINATION)
"""

# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///

from __future__ import annotations

import argparse
import contextlib
import datetime as dt
import fcntl
import json
import os
import re
import shutil
import signal
import subprocess
import sys
import tempfile
import time
from pathlib import Path

HARNESS_DIR = Path(__file__).resolve().parent
EVAL_DIR = HARNESS_DIR.parent
REPO_ROOT = EVAL_DIR.parent.parent
TASKS_DIR = EVAL_DIR / "tasks"
RUNS_DIR = EVAL_DIR / "runs"
PINS_FILE = HARNESS_DIR / "pins.json"

ARMS = ("cli-bare", "cli-skills", "mcp-bare", "mcp-skills")
MAX_WALL_CLOCK_S = 300
MAX_AGENT_TURNS = 25
MAX_CONTEXT_TOKENS = 100000
MAX_BUDGET_USD = 2.00
DEFAULT_VERSION = "v0.2.1"

# Shared isolation lock
CACHE_DIR = Path.home() / ".cache" / "montagent-discovery-eval"
LOCK_FILE = CACHE_DIR / "run.lock"
SANDBOX_TMP = Path(f"/private/tmp/claude-{os.getuid()}")

# Known valid Montagent commands and subcommands for hallucination detection
KNOWN_COMMANDS = {
    "validate", "render", "verify", "create-project", "create_project",
    "shift", "preview", "measure", "query", "vendor", "acquire",
    "compare", "fonts", "frame"
}


def load_pins(path: Path | None = None) -> dict:
    target = path or PINS_FILE
    if target.exists():
        with open(target, "r") as f:
            return json.load(f)
    return {
        "model": "claude-3-7-sonnet",
        "effort": "high",
        "claude_code_version": "2.1.296",
        "caps": {
            "wall_clock_s": MAX_WALL_CLOCK_S,
            "max_turns": MAX_AGENT_TURNS,
            "max_tokens": MAX_CONTEXT_TOKENS,
            "budget_usd": MAX_BUDGET_USD,
        },
        "network": {
            "allowed_domains": [],
        },
        "target_version": DEFAULT_VERSION,
    }


def find_montagent_bin() -> Path:
    env_bin = os.environ.get("MONTAGENT")
    if env_bin and Path(env_bin).exists():
        return Path(env_bin).resolve()
    which_bin = shutil.which("montagent")
    if which_bin:
        return Path(which_bin).resolve()
    debug_bin = REPO_ROOT / "target" / "debug" / "montagent"
    if debug_bin.exists():
        return debug_bin.resolve()
    release_bin = REPO_ROOT / "target" / "release" / "montagent"
    if release_bin.exists():
        return release_bin.resolve()
    # Default to cargo built binary path
    return debug_bin.resolve()


def path_without_montagent() -> list[str]:
    return [
        d for d in os.environ.get("PATH", "").split(os.pathsep)
        if d and not (Path(d) / "montagent").exists()
    ]


def build_env(scratch: Path, work: Path, arm: str, bin_path: Path) -> dict:
    raw_path = path_without_montagent()
    if arm in ("cli-bare", "cli-skills"):
        raw_path.insert(0, str(bin_path.parent))

    cache = work / ".cache"
    env = {
        "HOME": os.environ.get("HOME", str(Path.home())),
        "USER": os.environ.get("USER", ""),
        "LANG": os.environ.get("LANG", "en_US.UTF-8"),
        "TERM": "dumb",
        "PATH": os.pathsep.join(raw_path),
        "TMPDIR": str(scratch / "tmp"),
        "DISABLE_AUTOUPDATER": "1",
        "MONTAGENT_CACHE_DIR": str(cache / "montagent"),
        "npm_config_cache": str(cache / "npm"),
        "UV_CACHE_DIR": str(cache / "uv"),
        "PIP_CACHE_DIR": str(cache / "pip"),
        "XDG_CACHE_HOME": str(cache / "xdg"),
    }
    if "ANTHROPIC_API_KEY" in os.environ:
        env["ANTHROPIC_API_KEY"] = os.environ["ANTHROPIC_API_KEY"]
    return env


def settings(pins: dict) -> dict:
    return {
        "permissions": {"deny": ["WebFetch", "WebSearch"]},
        "sandbox": {
            "enabled": True,
            "autoAllowBashIfSandboxed": True,
            "allowUnsandboxedCommands": False,
            "network": {"allowedDomains": pins.get("network", {}).get("allowed_domains", [])},
        },
    }


def claude_argv(pins: dict, arm: str, prompt: str, bin_path: Path) -> list[str]:
    argv = [
        "claude", "-p", prompt,
        "--model", pins.get("model", "claude-3-7-sonnet"),
        "--effort", pins.get("effort", "high"),
        "--output-format", "stream-json", "--verbose",
        "--max-budget-usd", str(pins.get("caps", {}).get("budget_usd", MAX_BUDGET_USD)),
        "--permission-mode", "dontAsk",
        "--permission-prompts", "none",
        "--allowedTools", "Bash", "Read", "Edit", "Write", "Glob", "Grep", "mcp__montagent",
        "--setting-sources", "project",
        "--settings", json.dumps(settings(pins)),
        "--disallowedTools", "WebFetch", "WebSearch",
        "--no-session-persistence",
        "--strict-mcp-config",
    ]
    if arm in ("mcp-bare", "mcp-skills"):
        mcp_cfg = {
            "mcpServers": {
                "montagent": {
                    "command": str(bin_path),
                    "args": ["mcp"],
                }
            }
        }
        argv += ["--mcp-config", json.dumps(mcp_cfg)]
    return argv


class LoopAbortMonitor:
    """Monitors live event stream and trips early abort triggers."""

    def __init__(self, max_turns: int = MAX_AGENT_TURNS, max_tokens: int = MAX_CONTEXT_TOKENS):
        self.max_turns = max_turns
        self.max_tokens = max_tokens
        self.turns = 0
        self.total_tokens = 0
        self.consecutive_identical_errors = 0
        self.last_error_signature: str | None = None
        self.consecutive_hallucinations = 0
        self.inspected_help = False

    def process_event(self, event: dict) -> tuple[bool, str | None]:
        etype = event.get("type")

        # Turn accounting
        if etype in ("user", "assistant"):
            if etype == "user":
                self.turns += 1
                if self.turns > self.max_turns:
                    return True, "BUDGET_EXHAUSTION"

        # Token accounting from result / usage
        usage = event.get("usage") or event.get("message", {}).get("usage")
        if usage:
            inp = usage.get("input_tokens", 0)
            out = usage.get("output_tokens", 0)
            self.total_tokens = max(self.total_tokens, inp + out)
            if self.total_tokens > self.max_tokens:
                return True, "BUDGET_EXHAUSTION"

        # Tool calls & help inspection
        if etype == "assistant":
            content = event.get("message", {}).get("content", [])
            for block in content:
                if isinstance(block, dict) and block.get("type") == "tool_use":
                    name = block.get("name", "")
                    inp = block.get("input", {}) or {}
                    cmd = inp.get("command", "") if name == "Bash" else ""

                    # Did agent inspect help?
                    if "--help" in cmd or "-h" in cmd or "tools/list" in cmd:
                        self.inspected_help = True
                        self.consecutive_hallucinations = 0

                    # Check for hallucinated command
                    if name == "Bash" and "montagent" in cmd:
                        # Extract subcommand
                        parts = re.findall(r"\bmontagent\s+([a-zA-Z0-9_-]+)", cmd)
                        if parts:
                            subcmd = parts[0]
                            if subcmd not in KNOWN_COMMANDS and not subcmd.startswith("-"):
                                if not self.inspected_help:
                                    self.consecutive_hallucinations += 1
                                    if self.consecutive_hallucinations >= 3:
                                        return True, "MODEL_HALLUCINATION"
                            else:
                                self.consecutive_hallucinations = 0

        # Tool result error tracking
        if etype == "tool_result" or (etype == "user" and "tool_result" in str(event)):
            is_error = event.get("is_error", False)
            content = str(event.get("content", ""))
            if is_error or "Error:" in content:
                # Normalize signature
                sig = re.sub(r"\s+", " ", content.strip()[:100])
                if sig == self.last_error_signature:
                    self.consecutive_identical_errors += 1
                else:
                    self.consecutive_identical_errors = 1
                    self.last_error_signature = sig

                if self.consecutive_identical_errors >= 5:
                    return True, "ERGONOMIC_FRICTION"
            else:
                self.consecutive_identical_errors = 0
                self.last_error_signature = None

        return False, None


@contextlib.contextmanager
def one_run_at_a_time():
    CACHE_DIR.mkdir(parents=True, exist_ok=True)
    with open(LOCK_FILE, "w") as f:
        try:
            fcntl.flock(f, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            print("Another run holds the lock; waiting for it to finish...", flush=True)
            fcntl.flock(f, fcntl.LOCK_EX)
        try:
            yield
        finally:
            fcntl.flock(f, fcntl.LOCK_UN)


def sandbox_tmp_entries() -> set[str]:
    return {p.name for p in SANDBOX_TMP.iterdir()} if SANDBOX_TMP.is_dir() else set()


def sweep_sandbox_tmp(before: set[str], dest: Path) -> list[str]:
    new = sorted(sandbox_tmp_entries() - before)
    dest.mkdir(parents=True, exist_ok=True)
    for name in new:
        src = SANDBOX_TMP / name
        dst = dest / name
        try:
            shutil.move(str(src), str(dst))
        except Exception:
            pass
    return new


def write_manifest(
    out_dir: Path,
    task_id: str,
    arm: str,
    trial: int,
    version: str,
    model: str,
    duration_s: float,
    turns: int,
    tokens: int,
    cost_usd: float,
    exit_code: int,
    abort_reason: str | None = None,
) -> dict:
    out_dir.mkdir(parents=True, exist_ok=True)
    manifest = {
        "task_id": task_id,
        "arm": arm,
        "trial": trial,
        "version": version,
        "model": model,
        "timestamp_start": dt.datetime.now(dt.timezone.utc).isoformat(),
        "duration_s": duration_s,
        "turn_count": turns,
        "token_count": tokens,
        "cost_usd": cost_usd,
        "exit_code": exit_code,
        "abort_reason": abort_reason,
    }
    manifest_path = out_dir / "manifest.json"
    with open(manifest_path, "w") as f:
        json.dump(manifest, f, indent=2)
    return manifest


def provision_workspace(
    work_dir: Path,
    arm: str,
    task_id: str,
    task_dir: Path,
    repo_skills_dir: Path,
) -> None:
    work_dir.mkdir(parents=True, exist_ok=True)
    (work_dir / ".cache").mkdir(parents=True, exist_ok=True)

    # Copy fixtures
    fixtures_dir = task_dir / "fixtures"
    if fixtures_dir.is_dir():
        for item in fixtures_dir.iterdir():
            dest = work_dir / item.name
            if item.is_dir():
                shutil.copytree(item, dest, dirs_exist_ok=True)
            else:
                shutil.copy2(item, dest)

    # In skills arms, provision skills into .claude/skills/
    if arm in ("cli-skills", "mcp-skills"):
        claude_skills = work_dir / ".claude" / "skills"
        claude_skills.mkdir(parents=True, exist_ok=True)
        if repo_skills_dir.is_dir():
            for skill_dir in repo_skills_dir.iterdir():
                if skill_dir.is_dir() and (skill_dir / "SKILL.md").exists():
                    dest_skill = claude_skills / skill_dir.name
                    if not dest_skill.exists():
                        shutil.copytree(skill_dir, dest_skill)


def execute_run(
    task_id: str,
    arm: str,
    trial: int = 1,
    phase: str = "pilot-smoke",
    version: str = DEFAULT_VERSION,
    pins_path: Path | None = None,
) -> int:
    pins = load_pins(pins_path)
    task_dir = TASKS_DIR / task_id
    if not task_dir.is_dir():
        print(f"Error: task directory {task_dir} not found", file=sys.stderr)
        return 1

    prompt_path = task_dir / "prompt.txt"
    if not prompt_path.is_file():
        print(f"Error: prompt {prompt_path} not found", file=sys.stderr)
        return 1

    prompt_text = prompt_path.read_text().strip()
    bin_path = find_montagent_bin()

    run_dir = RUNS_DIR / version / phase / task_id / f"{arm}-{trial}"
    run_dir.mkdir(parents=True, exist_ok=True)
    local_dir = RUNS_DIR / ".local" / version / phase / task_id / f"{arm}-{trial}"
    local_dir.mkdir(parents=True, exist_ok=True)

    transcript_path = local_dir / "transcript.jsonl"
    stderr_path = local_dir / "stderr.log"

    with one_run_at_a_time():
        with tempfile.TemporaryDirectory(prefix=f"discovery-eval-{task_id}-{arm}-") as tmp_root:
            tmp_path = Path(tmp_root)
            work_dir = tmp_path / "work"
            scratch_dir = tmp_path / "scratch"
            scratch_tmp = scratch_dir / "tmp"
            scratch_tmp.mkdir(parents=True, exist_ok=True)

            provision_workspace(
                work_dir=work_dir,
                arm=arm,
                task_id=task_id,
                task_dir=task_dir,
                repo_skills_dir=REPO_ROOT / "skills",
            )

            env = build_env(scratch_dir, work_dir, arm, bin_path)
            argv = claude_argv(pins, arm, prompt_text, bin_path)

            before_tmp = sandbox_tmp_entries()
            started = time.monotonic()
            abort_reason = None
            exit_code = 0

            monitor = LoopAbortMonitor(
                max_turns=pins.get("caps", {}).get("max_turns", MAX_AGENT_TURNS),
                max_tokens=pins.get("caps", {}).get("max_tokens", MAX_CONTEXT_TOKENS),
            )

            with open(transcript_path, "w") as t_out, open(stderr_path, "w") as e_out:
                proc = subprocess.Popen(
                    argv,
                    cwd=work_dir,
                    env=env,
                    stdout=subprocess.PIPE,
                    stderr=e_out,
                    text=True,
                    bufsize=1,
                    start_new_session=True,
                )

                while True:
                    # Check timeout
                    elapsed = time.monotonic() - started
                    if elapsed > MAX_WALL_CLOCK_S:
                        abort_reason = "BUDGET_EXHAUSTION"
                        try:
                            os.killpg(proc.pid, signal.SIGTERM)
                            proc.wait(timeout=15)
                        except (subprocess.TimeoutExpired, ProcessLookupError):
                            try:
                                os.killpg(proc.pid, signal.SIGKILL)
                            except ProcessLookupError:
                                pass
                        break

                    line = proc.stdout.readline() if proc.stdout else ""
                    if not line and proc.poll() is not None:
                        break

                    if line:
                        t_out.write(line)
                        t_out.flush()
                        try:
                            event = json.loads(line)
                            aborted, reason = monitor.process_event(event)
                            if aborted:
                                abort_reason = reason
                                try:
                                    os.killpg(proc.pid, signal.SIGTERM)
                                    proc.wait(timeout=15)
                                except (subprocess.TimeoutExpired, ProcessLookupError):
                                    try:
                                        os.killpg(proc.pid, signal.SIGKILL)
                                    except ProcessLookupError:
                                        pass
                                break
                        except json.JSONDecodeError:
                            pass

                exit_code = proc.returncode or 0

            sweep_sandbox_tmp(before_tmp, scratch_dir / "swept")
            duration_s = round(time.monotonic() - started, 2)

            # Copy agent workspace deliverables to local workspace cache
            local_ws = local_dir / "workspace"
            shutil.copytree(work_dir, local_ws, dirs_exist_ok=True)

            write_manifest(
                out_dir=run_dir,
                task_id=task_id,
                arm=arm,
                trial=trial,
                version=version,
                model=pins.get("model", "claude-3-7-sonnet"),
                duration_s=duration_s,
                turns=monitor.turns,
                tokens=monitor.total_tokens,
                cost_usd=0.0,
                exit_code=exit_code,
                abort_reason=abort_reason,
            )

    return exit_code


def main() -> int:
    parser = argparse.ArgumentParser(description="Montagent Discovery Evaluation Runner")
    parser.add_argument("--task", help="Task ID (e.g. discovery-01-assembly-inspect)")
    parser.add_argument("--arm", choices=ARMS, help="Evaluation arm")
    parser.add_argument("--trial", type=int, default=1, help="Trial index")
    parser.add_argument("--phase", default="pilot-smoke", help="Phase (pilot-smoke, baseline)")
    parser.add_argument("--version", default=DEFAULT_VERSION, help="Target Montagent version")
    parser.add_argument("--pins", type=Path, help="Path to custom pins.json")
    parser.add_argument("--clean-local", action="store_true", help="Clean local run cache")
    parser.add_argument("--days", type=int, default=14, help="Days threshold for --clean-local")
    parser.add_argument("--all", action="store_true", help="Purge all local run cache")
    parser.add_argument("--report", action="store_true", help="Print register status report")
    parser.add_argument("--filter", help="Filter for --report (e.g. stale,untested)")
    parser.add_argument("--stale", action="store_true", help="Run only tasks for stale capabilities")

    args = parser.parse_args()

    if args.clean_local:
        local_root = RUNS_DIR / ".local"
        if not local_root.exists():
            print("No local cache directory found.")
            return 0
        if args.all:
            shutil.rmtree(local_root)
            print("Purged all local run caches.")
            return 0
        cutoff = time.time() - (args.days * 86400)
        for root, dirs, files in os.walk(local_root, topdown=False):
            for f in files:
                p = Path(root) / f
                if p.stat().st_mtime < cutoff:
                    p.unlink()
        print(f"Cleaned local run caches older than {args.days} days.")
        return 0

    if args.report or args.stale:
        print("Note: --report and --stale will be connected to register synchronization in Ticket #969.")
        return 0

    if not args.task or not args.arm:
        parser.print_help()
        return 1

    return execute_run(
        task_id=args.task,
        arm=args.arm,
        trial=args.trial,
        phase=args.phase,
        version=args.version,
        pins_path=args.pins,
    )


if __name__ == "__main__":
    sys.exit(main())
