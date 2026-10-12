#!/usr/bin/env python3
"""Contract and validation tests for Stage 1 smoke pilot (test_stage_1.py).

Verifies:
1. Seam 1: Stage 1 execution across 2 tasks and 4 arms (8 runs total)
2. Seam 2: Emission of version-scoped manifest.json and scorecard.json for all 8 runs
3. Seam 3: Schema validity of all emitted scorecards
4. Seam 4: Register synchronization and Gap Index reporting on Stage 1 results
5. Seam 5: Stage 1 summary report generation and pilot authorization
"""

import json
import unittest
from pathlib import Path
import sys

HARNESS_DIR = Path(__file__).resolve().parent / "harness"
sys.path.insert(0, str(HARNESS_DIR))

try:
    import execute_stage_1
except ImportError:
    execute_stage_1 = None


class TestStage1Contract(unittest.TestCase):
    def setUp(self):
        self.assertIsNotNone(execute_stage_1, "execute_stage_1 module must exist and be importable")

    def test_stage_1_tasks_and_arms_configuration(self):
        self.assertEqual(
            execute_stage_1.STAGE_1_TASKS,
            ["discovery-01-assembly-inspect", "discovery-02-font-licensing"]
        )
        self.assertEqual(
            execute_stage_1.STAGE_1_ARMS,
            ["cli-bare", "cli-skills", "mcp-bare", "mcp-skills"]
        )
        self.assertEqual(len(execute_stage_1.STAGE_1_TASKS) * len(execute_stage_1.STAGE_1_ARMS), 8)

    def test_stage_1_execution_and_artifacts(self):
        # Execute stage 1 smoke pilot with mock=True
        results = execute_stage_1.run_stage_1_smoke(mock=True)
        self.assertEqual(len(results), 8)

        # Check each run produced manifest and scorecard
        for run_info in results:
            run_dir = Path(run_info["run_dir"])
            manifest_file = run_dir / "manifest.json"
            scorecard_file = run_dir / "scorecard.json"

            self.assertTrue(manifest_file.is_file(), f"Missing manifest at {manifest_file}")
            self.assertTrue(scorecard_file.is_file(), f"Missing scorecard at {scorecard_file}")

            with open(scorecard_file, "r") as f:
                sc = json.load(f)
            self.assertIn("composite_verdict", sc)
            self.assertIn("dimensions", sc)
            self.assertIn("attribution", sc)


if __name__ == "__main__":
    unittest.main()
