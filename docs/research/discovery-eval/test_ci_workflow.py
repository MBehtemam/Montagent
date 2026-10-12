#!/usr/bin/env python3
"""Contract and lint tests for discovery smoke CI workflow.

Verifies:
1. Seam 1: Workflow file existence and valid YAML structure
2. Seam 2: Path filters covering discovery-eval, CLI, core, and skills
3. Seam 3: Triggers (pull_request, push, workflow_dispatch)
4. Seam 4: Execution of task tests and harness contract tests
"""

import unittest
from pathlib import Path

WORKFLOW_FILE = Path(__file__).resolve().parent.parent.parent.parent / ".github" / "workflows" / "discovery-smoke.yml"


class TestDiscoverySmokeWorkflow(unittest.TestCase):
    def test_workflow_exists(self):
        self.assertTrue(WORKFLOW_FILE.is_file(), f"Workflow file {WORKFLOW_FILE} must exist")

    def test_workflow_yaml_structure_and_triggers(self):
        self.assertTrue(WORKFLOW_FILE.is_file())
        content = WORKFLOW_FILE.read_text()

        # Basic YAML parsing check
        try:
            import yaml
            data = yaml.safe_load(content)
        except ImportError:
            # Simple fallback check if pyyaml not installed
            data = None

        if data:
            self.assertIn("name", data)
            # Check triggers
            on = data.get("on", data.get(True, {}))
            self.assertTrue("pull_request" in on or "push" in on or "workflow_dispatch" in on)
            paths = []
            if "pull_request" in on and "paths" in on["pull_request"]:
                paths.extend(on["pull_request"]["paths"])
            if "push" in on and "paths" in on["push"]:
                paths.extend(on["push"]["paths"])

            self.assertTrue(any("docs/research/discovery-eval/**" in p for p in paths))
            self.assertTrue(any("crates/montagent-cli/**" in p for p in paths))
            self.assertTrue(any("crates/montagent-core/**" in p for p in paths))
            self.assertTrue(any("skills/**" in p for p in paths))
        else:
            self.assertIn("docs/research/discovery-eval/**", content)
            self.assertIn("crates/montagent-cli/**", content)
            self.assertIn("crates/montagent-core/**", content)
            self.assertIn("skills/**", content)
            self.assertIn("workflow_dispatch", content)

    def test_workflow_runs_test_suites(self):
        content = WORKFLOW_FILE.read_text()
        self.assertIn("test_tasks.py", content)
        self.assertIn("test_run_arm.py", content)
        self.assertIn("test_score_trace.py", content)
        self.assertIn("test_sync_register.py", content)


if __name__ == "__main__":
    unittest.main()
