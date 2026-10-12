#!/usr/bin/env python3
"""Deterministic trace parser, 5-dimension scorecard evaluator, and failure attribution.

    uv run docs/research/discovery-eval/harness/score_trace.py --run-dir <path>

Evaluates:
1. 5 Scorecard Dimensions:
   - Discovery: Inspected, Hint-Prompted, Uninspected
   - Selection: Native Optimal, Acceptable Alternative, Unsupported Workaround, Mismatched Tool
   - Execution: Clean Pass, Recovered Pass, Failed Execution
   - Guessing: Clean, Incidental, Persistent
   - Missed Built-In Capability: None, Flagged
2. 3-Tier Composite Verdict: Full Success, Partial Success, Failure
3. 5-Category Failure Attribution Taxonomy:
   - DISCOVERY_INVISIBILITY
   - SELECTION_AMBIGUITY
   - ERGONOMIC_FRICTION
   - BUDGET_EXHAUSTION
   - MODEL_HALLUCINATION
4. Deliverable Verification via montagent verify / measure / probe
5. Emits schema-compliant scorecard.json
"""

# /// script
# requires-python = ">=3.11"
# dependencies = [
#   "jsonschema",
# ]
# ///

from __future__ import annotations

import argparse
import gzip
import json
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path

HARNESS_DIR = Path(__file__).resolve().parent
EVAL_DIR = HARNESS_DIR.parent
REPO_ROOT = EVAL_DIR.parent.parent
TASKS_DIR = EVAL_DIR / "tasks"
SCHEMA_FILE = EVAL_DIR / "scorecard.schema.json"

WORKAROUND_BINARIES = {"ffmpeg", "ffprobe", "sox", "convert", "magick"}


def read_jsonl(path: Path) -> list[dict]:
    events = []
    if not path.is_file():
        return events
    with open(path, "r", encoding="utf-8", errors="replace") as f:
        for line in f:
            line = line.strip()
            if line:
                try:
                    events.append(json.loads(line))
                except json.JSONDecodeError:
                    pass
    return events


def extract_trace_signals(events: list[dict]) -> dict:
    """Extract mechanical facts from stream-json transcript."""
    inspected_help = False
    inspected_after_error = False
    has_error_occurred = False
    first_action_turn: int | None = None
    turn_count = 0
    total_tokens = 0

    has_unsupported_workaround = False
    has_native_optimal = False
    has_acceptable_alternative = False
    mismatched_tool = False

    guessing_count = 0
    error_count = 0
    clean_execution = True
    recovered_execution = False

    for event in events:
        etype = event.get("type")
        if etype == "user":
            turn_count += 1

        usage = event.get("usage") or event.get("message", {}).get("usage")
        if usage:
            inp = usage.get("input_tokens", 0)
            out = usage.get("output_tokens", 0)
            total_tokens = max(total_tokens, inp + out)

        if etype == "assistant":
            content = event.get("message", {}).get("content", [])
            for block in content:
                if not isinstance(block, dict) or block.get("type") != "tool_use":
                    continue
                name = block.get("name", "")
                inp = block.get("input", {}) or {}

                # Check if this is an action turn
                if first_action_turn is None and name in ("Bash", "Write", "Edit"):
                    first_action_turn = max(1, turn_count)

                cmd = inp.get("command", "") if name == "Bash" else ""
                file_path = inp.get("file_path", "") if name in ("Read", "Write", "Edit") else ""

                # Discovery check
                if "--help" in cmd or "-h" in cmd or "tools/list" in cmd:
                    inspected_help = True
                    if has_error_occurred:
                        inspected_after_error = True

                if ".claude/skills" in file_path or "skills/montagent" in file_path:
                    inspected_help = True

                # Selection check: Workaround vs Native
                if name == "Bash":
                    tokens = set(re.findall(r"\b[a-zA-Z0-9_-]+\b", cmd))
                    if tokens & WORKAROUND_BINARIES:
                        has_unsupported_workaround = True

                    if "montagent" in tokens:
                        has_native_optimal = True

                if name.startswith("mcp__montagent"):
                    has_native_optimal = True

                if name in ("Write", "Edit") and ("project.json" in file_path or ".json" in file_path):
                    has_acceptable_alternative = True

        if etype == "tool_result" or (etype == "user" and "tool_result" in str(event)):
            is_error = event.get("is_error", False)
            content = str(event.get("content", ""))
            if is_error or "Error:" in content:
                has_error_occurred = True
                error_count += 1
                clean_execution = False
            else:
                if has_error_occurred:
                    recovered_execution = True

    discovery_turns = first_action_turn if first_action_turn is not None else max(1, turn_count)

    return {
        "inspected_help": inspected_help,
        "inspected_after_error": inspected_after_error,
        "discovery_turns": discovery_turns,
        "total_turns": turn_count,
        "total_tokens": total_tokens,
        "has_unsupported_workaround": has_unsupported_workaround,
        "has_native_optimal": has_native_optimal,
        "has_acceptable_alternative": has_acceptable_alternative,
        "mismatched_tool": mismatched_tool,
        "guessing_count": guessing_count,
        "error_count": error_count,
        "clean_execution": clean_execution,
        "recovered_execution": recovered_execution,
    }


def score_dimensions(info: dict, deliverable_valid: bool, missed_flagged: bool = False) -> dict:
    # 1. Discovery
    if info["inspected_help"]:
        discovery = "Hint-Prompted" if info["inspected_after_error"] else "Inspected"
    else:
        discovery = "Uninspected"

    # 2. Selection
    if info["has_unsupported_workaround"]:
        selection = "Unsupported Workaround"
    elif info["has_native_optimal"]:
        selection = "Native Optimal"
    elif info["has_acceptable_alternative"]:
        selection = "Acceptable Alternative"
    elif info["mismatched_tool"]:
        selection = "Mismatched Tool"
    else:
        selection = "Native Optimal"

    # 3. Execution
    if not deliverable_valid:
        execution = "Failed Execution"
    elif info["clean_execution"]:
        execution = "Clean Pass"
    elif info["recovered_execution"]:
        execution = "Recovered Pass"
    else:
        execution = "Failed Execution"

    # 4. Guessing
    gc = info.get("guessing_count", 0)
    if gc == 0:
        guessing = "Clean"
    elif gc <= 2:
        guessing = "Incidental"
    else:
        guessing = "Persistent"

    # 5. Missed Built-In Capability
    missed_capability = "Flagged" if missed_flagged else "None"

    return {
        "discovery": discovery,
        "selection": selection,
        "execution": execution,
        "guessing": guessing,
        "missed_capability": missed_capability,
    }


def evaluate_composite_verdict(dimensions: dict) -> str:
    disc = dimensions["discovery"]
    sel = dimensions["selection"]
    exe = dimensions["execution"]
    guess = dimensions["guessing"]
    missed = dimensions["missed_capability"]

    # Hard failures
    if exe == "Failed Execution":
        return "Failure"
    if sel in ("Unsupported Workaround", "Mismatched Tool"):
        return "Failure"

    # Full success check
    if (
        disc in ("Inspected", "Hint-Prompted")
        and sel in ("Native Optimal", "Acceptable Alternative")
        and exe in ("Clean Pass", "Recovered Pass")
        and guess in ("Clean", "Incidental")
        and missed == "None"
    ):
        return "Full Success"

    # Partial success
    if exe in ("Clean Pass", "Recovered Pass"):
        if missed == "Flagged" or guess == "Persistent" or sel == "Acceptable Alternative":
            return "Partial Success"

    return "Failure"


def attribute_outcome(
    composite_verdict: str,
    abort_reason: str | None,
    dimensions: dict,
    manifest: dict,
) -> dict:
    if composite_verdict == "Full Success":
        return {
            "primary_code": "NONE",
            "rationale": "Clean pass with verified native tool execution and no missed capabilities."
        }

    # Failure / Partial attribution
    if abort_reason == "BUDGET_EXHAUSTION" or manifest.get("turn_count", 0) >= 25 or manifest.get("duration_s", 0) >= 300:
        return {
            "primary_code": "BUDGET_EXHAUSTION",
            "rationale": "Agent exhausted allowable turn (25) or time (300s) ceiling before producing valid deliverable."
        }

    if abort_reason == "MODEL_HALLUCINATION" or dimensions.get("guessing") == "Persistent":
        return {
            "primary_code": "MODEL_HALLUCINATION",
            "rationale": "Agent issued multiple ungrounded commands/arguments contrary to documentation."
        }

    if abort_reason == "ERGONOMIC_FRICTION":
        return {
            "primary_code": "ERGONOMIC_FRICTION",
            "rationale": "Repeated syntax or schema validation failures prevented forward progress."
        }

    if dimensions.get("selection") == "Unsupported Workaround":
        return {
            "primary_code": "SELECTION_AMBIGUITY",
            "rationale": "Agent bypassed Montagent and chose unsupported external utility (e.g. ffmpeg/sox)."
        }

    if dimensions.get("discovery") == "Uninspected":
        return {
            "primary_code": "DISCOVERY_INVISIBILITY",
            "rationale": "Target capability was not inspected before invocation attempt."
        }

    return {
        "primary_code": "SELECTION_AMBIGUITY",
        "rationale": "Suboptimal tool selection or unassisted manual workaround."
    }


def compute_effort_tier(discovery_turns: int, total_turns: int) -> str:
    if discovery_turns <= 3:
        return "Direct"
    if discovery_turns <= 8:
        return "Moderate"
    return "Excessive"


def create_scorecard(
    run_id: str,
    task_id: str,
    arm: str,
    trial: int,
    montagent_version: str,
    dimensions: dict,
    metrics: dict,
    attribution: dict,
) -> dict:
    verdict = evaluate_composite_verdict(dimensions)
    return {
        "run_id": run_id,
        "task_id": task_id,
        "arm": arm,
        "trial": trial,
        "montagent_version": montagent_version,
        "composite_verdict": verdict,
        "dimensions": dimensions,
        "metrics": metrics,
        "attribution": attribution,
    }


def verify_workspace_deliverable(
    workspace_dir: Path,
    task_id: str,
    montagent_bin: Path,
) -> tuple[bool, str]:
    """Verify task output in workspace using Montagent or manifest rules."""
    if not workspace_dir.is_dir():
        return False, f"Workspace {workspace_dir} not found"

    task_dir = TASKS_DIR / task_id
    manifest_file = task_dir / "manifest.json"
    if not manifest_file.exists():
        return False, f"Task manifest {manifest_file} not found"

    with open(manifest_file, "r") as f:
        task_manifest = json.load(f)

    # Check for expected deliverables
    expected = task_manifest.get("deliverables", {}).get("expected_files", [])
    for exp_file in expected:
        fpath = workspace_dir / exp_file
        if not fpath.exists():
            return False, f"Missing required deliverable file {exp_file}"

    # If project.json exists, run montagent validate
    project_file = workspace_dir / "project.json"
    if project_file.is_file() and montagent_bin.is_file():
        r = subprocess.run(
            [str(montagent_bin), "validate", str(project_file)],
            capture_output=True,
            text=True,
        )
        if r.returncode != 0:
            return False, f"montagent validate failed: {r.stderr}"

    return True, "Deliverables verified successfully"


def score_run_directory(run_dir: Path) -> dict:
    manifest_file = run_dir / "manifest.json"
    if not manifest_file.exists():
        raise FileNotFoundError(f"Missing manifest.json in {run_dir}")

    with open(manifest_file, "r") as f:
        manifest = json.load(f)

    task_id = manifest["task_id"]
    arm = manifest["arm"]
    trial = manifest.get("trial", 1)
    version = manifest.get("version", "v0.2.1")
    abort_reason = manifest.get("abort_reason")
    run_id = f"{task_id}-{arm}-{trial}"

    # Transcript may live in run_dir or in .local cache
    transcript_path = run_dir / "transcript.jsonl"
    if not transcript_path.exists():
        local_transcript = EVAL_DIR / "runs" / ".local" / version / "pilot-smoke" / task_id / f"{arm}-{trial}" / "transcript.jsonl"
        if local_transcript.exists():
            transcript_path = local_transcript

    events = read_jsonl(transcript_path) if transcript_path.exists() else []
    signals = extract_trace_signals(events)

    # Check workspace deliverables
    ws_dir = run_dir / "workspace"
    if not ws_dir.exists():
        local_ws = EVAL_DIR / "runs" / ".local" / version / "pilot-smoke" / task_id / f"{arm}-{trial}" / "workspace"
        if local_ws.exists():
            ws_dir = local_ws

    bin_path = REPO_ROOT / "target" / "debug" / "montagent"
    if not bin_path.exists():
        which = shutil.which("montagent")
        bin_path = Path(which) if which else bin_path

    deliverable_valid, v_msg = verify_workspace_deliverable(ws_dir, task_id, bin_path)

    # If exit code was non-zero or aborted, deliverable is invalid
    if manifest.get("exit_code", 0) != 0 or abort_reason:
        deliverable_valid = False

    dimensions = score_dimensions(signals, deliverable_valid)
    metrics = {
        "discovery_turns": signals["discovery_turns"],
        "total_turns": manifest.get("turn_count", signals["total_turns"]),
        "total_tokens": manifest.get("token_count", signals["total_tokens"]),
        "discovery_effort_tier": compute_effort_tier(signals["discovery_turns"], manifest.get("turn_count", signals["total_turns"])),
    }
    attribution = attribute_outcome(
        composite_verdict=evaluate_composite_verdict(dimensions),
        abort_reason=abort_reason,
        dimensions=dimensions,
        manifest=manifest,
    )

    scorecard = create_scorecard(
        run_id=run_id,
        task_id=task_id,
        arm=arm,
        trial=trial,
        montagent_version=version,
        dimensions=dimensions,
        metrics=metrics,
        attribution=attribution,
    )

    # Save scorecard.json
    scorecard_path = run_dir / "scorecard.json"
    with open(scorecard_path, "w") as f:
        json.dump(scorecard, f, indent=2)

    # Per RETENTION.md: gzip transcript for failed / partial runs to transcript.jsonl.gz in run_dir
    if scorecard["composite_verdict"] in ("Failure", "Partial Success") and transcript_path.exists():
        gz_path = run_dir / "transcript.jsonl.gz"
        with open(transcript_path, "rb") as f_in, gzip.open(gz_path, "wb") as f_out:
            shutil.copyfileobj(f_in, f_out)

    return scorecard


def main() -> int:
    parser = argparse.ArgumentParser(description="Deterministic trace scorer and scorecard evaluator")
    parser.add_argument("--run-dir", type=Path, help="Run directory containing manifest.json")
    parser.add_argument("--sync-register", action="store_true", help="Synchronize coverage register from scorecards")

    args = parser.parse_args()

    if args.sync_register:
        from sync_register import sync_register, REGISTER_FILE
        sync_register(REGISTER_FILE, EVAL_DIR / "runs", TASKS_DIR)
        print(f"Synchronized {REGISTER_FILE.name} with scorecards.")
        return 0

    if not args.run_dir:
        parser.print_help()
        return 1

    try:
        sc = score_run_directory(args.run_dir)
        print(f"Scored {args.run_dir.name}: {sc['composite_verdict']} ({sc['attribution']['primary_code']})")
        return 0
    except Exception as e:
        print(f"Error scoring run: {e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
