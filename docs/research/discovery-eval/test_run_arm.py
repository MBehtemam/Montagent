#!/usr/bin/env python3
"""Contract and unit tests for discovery eval runner (run_arm.py).

Verifies:
1. Seam 1: 4-Arm Environment Provisioning & Isolation (PATH, MCP, skills)
2. Seam 2: Active Loop Abort Rules (5 identical errors, 3 hallucinations, turn/token caps)
3. Seam 3: Claude CLI Arguments & Security Sandbox Settings
4. Seam 4: Version-Scoped Run Output & Manifest Generation
"""

import json
import os
import tempfile
import unittest
from pathlib import Path

# Target module to import
import sys
HARNESS_DIR = Path(__file__).resolve().parent / "harness"
sys.path.insert(0, str(HARNESS_DIR))

try:
    import run_arm
except ImportError:
    run_arm = None


class TestRunArmContract(unittest.TestCase):
    def setUp(self):
        self.assertIsNotNone(run_arm, "run_arm module must exist and be importable")

    def test_arms_defined(self):
        self.assertEqual(
            set(run_arm.ARMS),
            {"cli-bare", "cli-skills", "mcp-bare", "mcp-skills"}
        )

    def test_caps_defined(self):
        self.assertEqual(run_arm.MAX_WALL_CLOCK_S, 300)
        self.assertEqual(run_arm.MAX_AGENT_TURNS, 25)
        self.assertEqual(run_arm.MAX_CONTEXT_TOKENS, 100000)
        self.assertEqual(run_arm.MAX_BUDGET_USD, 2.00)

    def test_environment_provisioning_cli_bare(self):
        with tempfile.TemporaryDirectory() as tmp:
            scratch = Path(tmp) / "scratch"
            work = Path(tmp) / "work"
            scratch.mkdir()
            work.mkdir()
            fake_bin = scratch / "bin" / "montagent"
            fake_bin.parent.mkdir(parents=True)
            fake_bin.touch()

            env = run_arm.build_env(scratch, work, "cli-bare", fake_bin)
            path_parts = env["PATH"].split(os.pathsep)
            self.assertIn(str(fake_bin.parent), path_parts)
            self.assertEqual(path_parts[0], str(fake_bin.parent))

    def test_environment_provisioning_mcp_bare(self):
        with tempfile.TemporaryDirectory() as tmp:
            scratch = Path(tmp) / "scratch"
            work = Path(tmp) / "work"
            scratch.mkdir()
            work.mkdir()
            fake_bin = scratch / "bin" / "montagent"
            fake_bin.parent.mkdir(parents=True)
            fake_bin.touch()

            env = run_arm.build_env(scratch, work, "mcp-bare", fake_bin)
            path_parts = env["PATH"].split(os.pathsep)
            # Binary parent directory must NOT be on PATH
            self.assertNotIn(str(fake_bin.parent), path_parts)

    def test_settings_air_gapped(self):
        pins = run_arm.load_pins()
        s = run_arm.settings(pins)
        self.assertIn("WebFetch", s["permissions"]["deny"])
        self.assertIn("WebSearch", s["permissions"]["deny"])
        self.assertEqual(s["sandbox"]["network"]["allowedDomains"], [])

    def test_claude_argv_configuration(self):
        pins = run_arm.load_pins()
        fake_bin = Path("/tmp/montagent")
        
        # Test cli-bare argv
        argv_cli = run_arm.claude_argv(pins, "cli-bare", "prompt text", fake_bin)
        self.assertIn("--model", argv_cli)
        self.assertIn(pins["model"], argv_cli)
        self.assertIn("--effort", argv_cli)
        self.assertIn("high", argv_cli)
        self.assertIn("--permission-mode", argv_cli)
        self.assertIn("dontAsk", argv_cli)
        self.assertNotIn("--mcp-config", argv_cli)

        # Test mcp-bare argv
        argv_mcp = run_arm.claude_argv(pins, "mcp-bare", "prompt text", fake_bin)
        self.assertIn("--mcp-config", argv_mcp)
        mcp_idx = argv_mcp.index("--mcp-config") + 1
        mcp_cfg = json.loads(argv_mcp[mcp_idx])
        self.assertIn("montagent", mcp_cfg["mcpServers"])
        self.assertEqual(mcp_cfg["mcpServers"]["montagent"]["command"], str(fake_bin))

    def test_loop_abort_consecutive_errors(self):
        monitor = run_arm.LoopAbortMonitor()
        # Feed 5 identical errors
        error_event = {
            "type": "tool_result",
            "is_error": True,
            "content": "Error: unknown flag --foo"
        }
        for i in range(4):
            abort, reason = monitor.process_event(error_event)
            self.assertFalse(abort, f"Should not abort at error {i+1}")
        
        abort, reason = monitor.process_event(error_event)
        self.assertTrue(abort, "Should abort at 5 consecutive identical errors")
        self.assertIn(reason, ["ERGONOMIC_FRICTION", "BUDGET_EXHAUSTION"])

    def test_loop_abort_hallucinations(self):
        monitor = run_arm.LoopAbortMonitor()
        # Feed 3 ungrounded command hallucinations without help inspection
        h_event = {
            "type": "assistant",
            "message": {
                "content": [{
                    "type": "tool_use",
                    "name": "Bash",
                    "input": {"command": "montagent fake-command-xyz"}
                }]
            }
        }
        for i in range(2):
            abort, reason = monitor.process_event(h_event)
            self.assertFalse(abort, f"Should not abort at hallucination {i+1}")
        
        abort, reason = monitor.process_event(h_event)
        self.assertTrue(abort, "Should abort at 3 consecutive hallucinations")
        self.assertEqual(reason, "MODEL_HALLUCINATION")

    def test_loop_abort_turn_cap(self):
        monitor = run_arm.LoopAbortMonitor(max_turns=25)
        user_turn_event = {"type": "user", "message": {"content": "continue"}}
        for _ in range(25):
            abort, _ = monitor.process_event(user_turn_event)
            self.assertFalse(abort)
        abort, reason = monitor.process_event(user_turn_event)
        self.assertTrue(abort)
        self.assertEqual(reason, "BUDGET_EXHAUSTION")

    def test_manifest_generation(self):
        with tempfile.TemporaryDirectory() as tmp:
            out_dir = Path(tmp)
            manifest = run_arm.write_manifest(
                out_dir=out_dir,
                task_id="discovery-01-assembly-inspect",
                arm="cli-bare",
                trial=1,
                version="v0.2.1",
                model="claude-3-7-sonnet",
                duration_s=12.5,
                turns=5,
                tokens=12000,
                cost_usd=0.25,
                exit_code=0,
                abort_reason=None,
            )
            self.assertTrue((out_dir / "manifest.json").exists())
            self.assertEqual(manifest["task_id"], "discovery-01-assembly-inspect")
            self.assertEqual(manifest["arm"], "cli-bare")
            self.assertEqual(manifest["exit_code"], 0)


if __name__ == "__main__":
    unittest.main()
