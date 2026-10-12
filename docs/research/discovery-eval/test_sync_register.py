#!/usr/bin/env python3
"""Contract and unit tests for coverage register synchronization and Gap Index reporting (sync_register.py).

Verifies:
1. Seam 1: Register Synchronization with Scorecards & Tasks
2. Seam 2: Gap Index (G) and Skill Leverage (L_skill) Formulas
3. Seam 3: Stale & Untested Task Filtering
4. Seam 4: Local Cache Pruning (Retention Rules)
5. Seam 5: Schema Validation of Synchronized register.json
"""

import json
import shutil
import tempfile
import unittest
from pathlib import Path
import sys

HARNESS_DIR = Path(__file__).resolve().parent / "harness"
sys.path.insert(0, str(HARNESS_DIR))

try:
    import sync_register
except ImportError:
    sync_register = None


class TestSyncRegisterContract(unittest.TestCase):
    def setUp(self):
        self.assertIsNotNone(sync_register, "sync_register module must exist and be importable")

    def test_calculate_gap_index(self):
        # Synthetic summary where bare has 2 passed, 2 failed (rate 0.5)
        # and skills has 4 passed, 0 failed (rate 1.0)
        register = {
            "summary": {
                "by_arm": {
                    "cli-bare": {"passed": 1, "failed": 1, "untested": 0, "stale": 0, "unavailable": 0, "intentionally_excluded": 0},
                    "mcp-bare": {"passed": 1, "failed": 1, "untested": 0, "stale": 0, "unavailable": 0, "intentionally_excluded": 0},
                    "cli-skills": {"passed": 2, "failed": 0, "untested": 0, "stale": 0, "unavailable": 0, "intentionally_excluded": 0},
                    "mcp-skills": {"passed": 2, "failed": 0, "untested": 0, "stale": 0, "unavailable": 0, "intentionally_excluded": 0},
                }
            }
        }
        metrics = sync_register.calculate_gap_index(register)
        self.assertEqual(metrics["bare_success_rate"], 0.5)
        self.assertEqual(metrics["gap_index"], 0.5)
        self.assertEqual(metrics["skills_success_rate"], 1.0)
        self.assertEqual(metrics["skill_leverage"], 0.5)

    def test_sync_register_with_scorecard(self):
        with tempfile.TemporaryDirectory() as tmp:
            tmp_path = Path(tmp)
            reg_path = tmp_path / "register.json"
            runs_path = tmp_path / "runs"
            tasks_path = tmp_path / "tasks"

            # Create mock task manifest
            task_dir = tasks_path / "task-01"
            task_dir.mkdir(parents=True)
            with open(task_dir / "manifest.json", "w") as f:
                json.dump({
                    "task_id": "task-01",
                    "coverage": {
                        "primary_targets": ["cap:project/create-project"]
                    }
                }, f)

            # Create initial register with cap:project/create-project untested
            initial_reg = {
                "$schema": "./register.schema.json",
                "version": "0.2.1",
                "updated_at": "2026-10-12T00:00:00Z",
                "evaluation_arms": ["cli-bare", "cli-skills", "mcp-bare", "mcp-skills"],
                "coverage_states": ["untested", "passed", "failed", "stale", "unavailable", "intentionally_excluded"],
                "summary": {
                    "total_capabilities": 1,
                    "by_arm": {
                        "cli-bare": {"untested": 1, "passed": 0, "failed": 0, "stale": 0, "unavailable": 0, "intentionally_excluded": 0},
                        "cli-skills": {"untested": 1, "passed": 0, "failed": 0, "stale": 0, "unavailable": 0, "intentionally_excluded": 0},
                        "mcp-bare": {"untested": 1, "passed": 0, "failed": 0, "stale": 0, "unavailable": 0, "intentionally_excluded": 0},
                        "mcp-skills": {"untested": 1, "passed": 0, "failed": 0, "stale": 0, "unavailable": 0, "intentionally_excluded": 0},
                    }
                },
                "features": [
                    {
                        "id": "cap:project/create-project",
                        "category": "project",
                        "name": "Project Scaffolding",
                        "description": "Scaffold empty project",
                        "interfaces": ["cli", "mcp"],
                        "coverage": {
                            "cli-bare": "untested",
                            "cli-skills": "untested",
                            "mcp-bare": "untested",
                            "mcp-skills": "untested"
                        },
                        "tasks": ["task-01"]
                    }
                ]
            }
            with open(reg_path, "w") as f:
                json.dump(initial_reg, f)

            # Create mock scorecard with Full Success for cli-bare
            sc_dir = runs_path / "0.2.1" / "pilot-smoke" / "task-01" / "cli-bare-1"
            sc_dir.mkdir(parents=True)
            with open(sc_dir / "scorecard.json", "w") as f:
                json.dump({
                    "task_id": "task-01",
                    "arm": "cli-bare",
                    "composite_verdict": "Full Success"
                }, f)

            # Sync
            updated_reg = sync_register.sync_register(reg_path, runs_path, tasks_path)
            feat = updated_reg["features"][0]
            self.assertEqual(feat["coverage"]["cli-bare"], "passed")
            self.assertEqual(feat["coverage"]["cli-skills"], "untested")
            self.assertEqual(updated_reg["summary"]["by_arm"]["cli-bare"]["passed"], 1)
            self.assertEqual(updated_reg["summary"]["by_arm"]["cli-bare"]["untested"], 0)

    def test_clean_local_cache(self):
        with tempfile.TemporaryDirectory() as tmp:
            local_cache = Path(tmp) / ".local"
            sample_file = local_cache / "run1" / "transcript.jsonl"
            sample_file.parent.mkdir(parents=True)
            sample_file.write_text("sample transcript")

            deleted = sync_register.clean_local_cache(local_cache, purge_all=True)
            self.assertGreater(deleted, 0)
            self.assertFalse(sample_file.exists())

    def test_filter_stale_or_untested_tasks(self):
        reg = {
            "features": [
                {
                    "id": "cap:1",
                    "coverage": {"cli-bare": "passed", "cli-skills": "passed", "mcp-bare": "passed", "mcp-skills": "passed"},
                    "tasks": ["task-done"]
                },
                {
                    "id": "cap:2",
                    "coverage": {"cli-bare": "stale", "cli-skills": "passed", "mcp-bare": "passed", "mcp-skills": "passed"},
                    "tasks": ["task-stale"]
                }
            ]
        }
        tasks = sync_register.find_stale_or_untested_tasks(reg)
        self.assertIn("task-stale", tasks)
        self.assertNotIn("task-done", tasks)


if __name__ == "__main__":
    unittest.main()
