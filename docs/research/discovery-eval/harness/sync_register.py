#!/usr/bin/env python3
"""Coverage register synchronization, Gap Index reporting, and local cache retention.

    uv run docs/research/discovery-eval/harness/sync_register.py [--report] [--clean-local] [--stale]

Commands:
- Synchronize register.json coverage states against version-scoped scorecards.
- Compute Gap Index (G) and Skill Leverage Factor (L_skill).
- Output formatted discovery status report across all 4 evaluation arms.
- Prune ephemeral runs/.local/ workspaces and traces per retention policies.
"""

# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///

from __future__ import annotations

import argparse
import datetime as dt
import json
import os
import shutil
import sys
import time
from pathlib import Path

HARNESS_DIR = Path(__file__).resolve().parent
EVAL_DIR = HARNESS_DIR.parent
TASKS_DIR = EVAL_DIR / "tasks"
RUNS_DIR = EVAL_DIR / "runs"
REGISTER_FILE = EVAL_DIR / "register.json"
SCHEMA_FILE = EVAL_DIR / "register.schema.json"


def load_register(path: Path | None = None) -> dict:
    target = path or REGISTER_FILE
    with open(target, "r") as f:
        return json.load(f)


def _calculate_arm_rate(
    by_arm: dict[str, dict[str, int]], arms: tuple[str, ...]
) -> tuple[int, int, float]:
    """Calculate (tested_count, passed_count, pass_rate) for a set of arms."""
    tested = sum(
        by_arm.get(arm, {}).get("passed", 0) + by_arm.get(arm, {}).get("failed", 0)
        for arm in arms
    )
    passed = sum(
        by_arm.get(arm, {}).get("passed", 0)
        for arm in arms
    )
    rate = (passed / tested) if tested > 0 else 0.0
    return tested, passed, rate


def calculate_gap_index(register: dict[str, Any]) -> dict[str, float]:
    """Compute Gap Index G = 1.0 - R_bare and Skill Leverage L_skill per PILOT.md."""
    by_arm = register.get("summary", {}).get("by_arm", {})

    bare_arms = ("cli-bare", "mcp-bare")
    skills_arms = ("cli-skills", "mcp-skills")

    _, _, bare_rate = _calculate_arm_rate(by_arm, bare_arms)
    _, _, skills_rate = _calculate_arm_rate(by_arm, skills_arms)

    gap_index = 1.0 - bare_rate
    skill_leverage = skills_rate - bare_rate

    return {
        "gap_index": round(gap_index, 3),
        "bare_success_rate": round(bare_rate, 3),
        "skills_success_rate": round(skills_rate, 3),
        "skill_leverage": round(skill_leverage, 3),
    }


def sync_register(
    register_path: Path,
    runs_path: Path,
    tasks_path: Path,
) -> dict[str, Any]:
    with open(register_path, "r") as f:
        register = json.load(f)

    task_manifest_cache: dict[str, list[str]] = {}

    def get_task_targets(task_id: str) -> list[str]:
        if task_id in task_manifest_cache:
            return task_manifest_cache[task_id]
        mfile = tasks_path / task_id / "manifest.json"
        if mfile.is_file():
            try:
                with open(mfile, "r") as mf:
                    doc = json.load(mf)
                    targets = (
                        doc.get("primary_target_capabilities")
                        or doc.get("coverage", {}).get("primary_targets", [])
                        or []
                    )
                    task_manifest_cache[task_id] = targets
                    return targets
            except (OSError, json.JSONDecodeError):
                pass
        return []

    # Map feature by id
    feature_map = {feat["id"]: feat for feat in register.get("features", [])}

    # Traverse scorecards
    if runs_path.is_dir():
        for sc_file in runs_path.rglob("scorecard.json"):
            # Skip .local directories
            if ".local" in sc_file.parts:
                continue
            try:
                with open(sc_file, "r") as f:
                    sc = json.load(f)
                task_id = sc.get("task_id")
                arm = sc.get("arm")
                verdict = sc.get("composite_verdict")
                if not task_id or not arm or not verdict:
                    continue

                targets = get_task_targets(task_id)
                for tid in targets:
                    feat = feature_map.get(tid)
                    if feat:
                        cur_state = feat.get("coverage", {}).get(arm, "untested")
                        if cur_state not in ("unavailable", "intentionally_excluded"):
                            new_state = "passed" if verdict == "Full Success" else "failed"
                            feat["coverage"][arm] = new_state
            except (OSError, json.JSONDecodeError, KeyError):
                pass

    # Recalculate summary counts
    arms = register.get("evaluation_arms", ["cli-bare", "cli-skills", "mcp-bare", "mcp-skills"])
    summary_by_arm: dict[str, dict[str, int]] = {}

    for arm in arms:
        counts = {
            "untested": 0,
            "passed": 0,
            "failed": 0,
            "stale": 0,
            "unavailable": 0,
            "intentionally_excluded": 0,
        }
        for feat in register.get("features", []):
            st = feat.get("coverage", {}).get(arm, "untested")
            counts[st] = counts.get(st, 0) + 1
        summary_by_arm[arm] = counts

    register["summary"]["by_arm"] = summary_by_arm
    register["summary"]["total_capabilities"] = len(register.get("features", []))
    register["updated_at"] = dt.datetime.now(dt.timezone.utc).isoformat()

    with open(register_path, "w") as f:
        json.dump(register, f, indent=2)

    return register


def find_stale_or_untested_tasks(register: dict[str, Any], tasks_dir: Path | None = None) -> list[str]:
    tasks = set()
    for feat in register.get("features", []):
        coverage = feat.get("coverage", {})
        if any(state in ("stale", "untested") for state in coverage.values()):
            for tid in feat.get("tasks", []):
                tasks.add(tid)
    return sorted(tasks)


def clean_local_cache(local_dir: Path, days: int = 14, purge_all: bool = False) -> int:
    if not local_dir.exists():
        return 0

    deleted_count = 0
    cutoff = time.time() - (days * 86400)

    for root, dirs, files in os.walk(local_dir, topdown=False):
        for f in files:
            p = Path(root) / f
            try:
                if purge_all or p.stat().st_mtime < cutoff:
                    p.unlink()
                    deleted_count += 1
            except OSError:
                pass
        for d in dirs:
            dp = Path(root) / d
            try:
                if purge_all or (dp.is_dir() and not any(dp.iterdir())):
                    dp.rmdir()
                    deleted_count += 1
            except OSError:
                pass

    if purge_all and local_dir.exists():
        try:
            shutil.rmtree(local_dir)
            deleted_count += 1
        except OSError:
            pass

    return deleted_count


def format_register_report(register: dict, filter_state: str | None = None) -> str:
    metrics = calculate_gap_index(register)
    summary = register.get("summary", {})
    by_arm = summary.get("by_arm", {})

    lines = [
        "=" * 78,
        f" Montagent Discovery Evaluation Coverage Register ({register.get('version', 'unknown')})",
        f" Last Updated: {register.get('updated_at', 'unknown')}",
        "=" * 78,
        f" Gap Index (G):          {metrics['gap_index']:.3f} (1.0 = total discovery barrier, 0.0 = zero friction)",
        f" Bare Success Rate:      {metrics['bare_success_rate']:.3f}",
        f" Skills Success Rate:    {metrics['skills_success_rate']:.3f}",
        f" Skill Leverage (L_skill): {metrics['skill_leverage']:.3f}",
        "-" * 78,
        " Arm Coverage Summary:",
        f" {'Arm':<12} {'Untested':<10} {'Passed':<8} {'Failed':<8} {'Stale':<8} {'Unavailable':<12} {'Excluded':<10}",
    ]
    for arm, counts in by_arm.items():
        lines.append(
            f" {arm:<12} {counts.get('untested', 0):<10} {counts.get('passed', 0):<8} "
            f"{counts.get('failed', 0):<8} {counts.get('stale', 0):<8} "
            f"{counts.get('unavailable', 0):<12} {counts.get('intentionally_excluded', 0):<10}"
        )

    lines.append("-" * 78)
    lines.append(f" Features List (filter: {filter_state or 'all'}):")
    lines.append(f" {'ID':<32} {'cli-bare':<10} {'cli-skills':<12} {'mcp-bare':<10} {'mcp-skills':<10}")

    for feat in register.get("features", []):
        cov = feat.get("coverage", {})
        if filter_state and not any(val == filter_state for val in cov.values()):
            continue
        lines.append(
            f" {feat.get('id', ''):<32} {cov.get('cli-bare', ''):<10} "
            f"{cov.get('cli-skills', ''):<12} {cov.get('mcp-bare', ''):<10} {cov.get('mcp-skills', ''):<10}"
        )

    lines.append("=" * 78)
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser(description="Montagent Coverage Register Synchronizer")
    parser.add_argument("--sync", action="store_true", help="Sync register with all run scorecards")
    parser.add_argument("--report", action="store_true", help="Print register status report")
    parser.add_argument("--filter", help="Filter features by status in report (e.g. untested, stale, failed)")
    parser.add_argument("--clean-local", action="store_true", help="Clean local ephemeral run cache")
    parser.add_argument("--days", type=int, default=14, help="Days threshold for local cache cleaning")
    parser.add_argument("--all", action="store_true", help="Purge all local run cache")
    parser.add_argument("--stale", action="store_true", help="List task IDs for stale or untested capabilities")

    args = parser.parse_args()

    if args.clean_local:
        local_dir = RUNS_DIR / ".local"
        del_count = clean_local_cache(local_dir, days=args.days, purge_all=args.all)
        print(f"Purged {del_count} ephemeral cache entries from {local_dir}")
        return 0

    if args.sync or (not args.report and not args.stale):
        sync_register(REGISTER_FILE, RUNS_DIR, TASKS_DIR)
        print(f"Synchronized {REGISTER_FILE.name} with scorecards in {RUNS_DIR.name}/")

    reg = load_register(REGISTER_FILE)

    if args.stale:
        stale_tasks = find_stale_or_untested_tasks(reg, TASKS_DIR)
        print("Tasks for stale or untested capabilities:")
        for t in stale_tasks:
            print(f"  - {t}")
        return 0

    if args.report or not args.sync:
        print(format_register_report(reg, filter_state=args.filter))

    return 0


if __name__ == "__main__":
    sys.exit(main())
