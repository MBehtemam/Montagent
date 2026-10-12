#!/usr/bin/env python3
"""Contract and unit tests for trace parser and scorecard evaluator (score_trace.py).

Verifies:
1. Seam 1: Deterministic Trace Parsing (inspections, tool calls, errors, external fallbacks)
2. Seam 2: 5-Dimension Scorecard Determination (Discovery, Selection, Execution, Guessing, Missed)
3. Seam 3: 3-Tier Composite Verdict Rules (Full Success, Partial Success, Failure)
4. Seam 4: 5-Category Failure Attribution Taxonomy
5. Seam 5: Schema Validation of Emitted scorecard.json
"""

import json
import tempfile
import unittest
from pathlib import Path
import sys

HARNESS_DIR = Path(__file__).resolve().parent / "harness"
sys.path.insert(0, str(HARNESS_DIR))

try:
    import score_trace
except ImportError:
    score_trace = None


class TestScoreTraceContract(unittest.TestCase):
    def setUp(self):
        self.assertIsNotNone(score_trace, "score_trace module must exist and be importable")

    def test_composite_verdict_full_success(self):
        dimensions = {
            "discovery": "Inspected",
            "selection": "Native Optimal",
            "execution": "Clean Pass",
            "guessing": "Clean",
            "missed_capability": "None"
        }
        verdict = score_trace.evaluate_composite_verdict(dimensions)
        self.assertEqual(verdict, "Full Success")

    def test_composite_verdict_partial_success_on_missed_capability(self):
        dimensions = {
            "discovery": "Inspected",
            "selection": "Acceptable Alternative",
            "execution": "Clean Pass",
            "guessing": "Clean",
            "missed_capability": "Flagged"
        }
        verdict = score_trace.evaluate_composite_verdict(dimensions)
        self.assertEqual(verdict, "Partial Success")

    def test_composite_verdict_failure_on_unsupported_workaround(self):
        # External ffmpeg/sox fallback counts as Unsupported Workaround and automatic Failure
        dimensions = {
            "discovery": "Uninspected",
            "selection": "Unsupported Workaround",
            "execution": "Clean Pass",
            "guessing": "Clean",
            "missed_capability": "None"
        }
        verdict = score_trace.evaluate_composite_verdict(dimensions)
        self.assertEqual(verdict, "Failure")

    def test_composite_verdict_failure_on_failed_execution(self):
        dimensions = {
            "discovery": "Inspected",
            "selection": "Native Optimal",
            "execution": "Failed Execution",
            "guessing": "Incidental",
            "missed_capability": "None"
        }
        verdict = score_trace.evaluate_composite_verdict(dimensions)
        self.assertEqual(verdict, "Failure")

    def test_failure_attribution_taxonomy(self):
        # Budget exhaustion
        attr = score_trace.attribute_outcome(
            composite_verdict="Failure",
            abort_reason="BUDGET_EXHAUSTION",
            dimensions={"discovery": "Inspected", "selection": "Native Optimal", "execution": "Failed Execution", "guessing": "Clean", "missed_capability": "None"},
            manifest={"turn_count": 26, "duration_s": 305}
        )
        self.assertEqual(attr["primary_code"], "BUDGET_EXHAUSTION")

        # Unsupported workaround -> Selection ambiguity
        attr2 = score_trace.attribute_outcome(
            composite_verdict="Failure",
            abort_reason=None,
            dimensions={"discovery": "Inspected", "selection": "Unsupported Workaround", "execution": "Clean Pass", "guessing": "Clean", "missed_capability": "None"},
            manifest={"turn_count": 5, "duration_s": 20}
        )
        self.assertEqual(attr2["primary_code"], "SELECTION_AMBIGUITY")

        # Model hallucination
        attr3 = score_trace.attribute_outcome(
            composite_verdict="Failure",
            abort_reason="MODEL_HALLUCINATION",
            dimensions={"discovery": "Uninspected", "selection": "Mismatched Tool", "execution": "Failed Execution", "guessing": "Persistent", "missed_capability": "None"},
            manifest={"turn_count": 8, "duration_s": 45}
        )
        self.assertEqual(attr3["primary_code"], "MODEL_HALLUCINATION")

    def test_parse_events_discovery_and_workaround(self):
        # Synthetic trace with help inspection and ffmpeg fallback
        events = [
            {
                "type": "assistant",
                "message": {"content": [{"type": "tool_use", "name": "Bash", "input": {"command": "montagent --help"}}]}
            },
            {
                "type": "tool_result",
                "content": "Usage: montagent <command>"
            },
            {
                "type": "assistant",
                "message": {"content": [{"type": "tool_use", "name": "Bash", "input": {"command": "ffmpeg -i in.mp4 -vf crop=100:100 out.mp4"}}]}
            }
        ]
        info = score_trace.extract_trace_signals(events)
        self.assertTrue(info["inspected_help"])
        self.assertTrue(info["has_unsupported_workaround"])
        self.assertEqual(info["discovery_turns"], 1)

    def test_schema_valid_scorecard_generation(self):
        schema_path = Path(__file__).resolve().parent / "scorecard.schema.json"
        self.assertTrue(schema_path.is_file(), f"Missing schema at {schema_path}")

        scorecard = score_trace.create_scorecard(
            run_id="run-01",
            task_id="discovery-01-assembly-inspect",
            arm="cli-bare",
            trial=1,
            montagent_version="v0.2.1",
            dimensions={
                "discovery": "Inspected",
                "selection": "Native Optimal",
                "execution": "Clean Pass",
                "guessing": "Clean",
                "missed_capability": "None"
            },
            metrics={
                "discovery_turns": 2,
                "total_turns": 6,
                "total_tokens": 15000,
                "discovery_effort_tier": "Direct"
            },
            attribution={
                "primary_code": "NONE",
                "rationale": "Clean pass with optimal native tool selection."
            }
        )

        with open(schema_path, "r") as f:
            schema = json.load(f)

        # Validate required fields and enums
        for req in schema["required"]:
            self.assertIn(req, scorecard)
        self.assertIn(scorecard["arm"], schema["properties"]["arm"]["enum"])
        self.assertIn(scorecard["composite_verdict"], schema["properties"]["composite_verdict"]["enum"])
        for dim, val in scorecard["dimensions"].items():
            self.assertIn(val, schema["properties"]["dimensions"]["properties"][dim]["enum"])
        self.assertIn(scorecard["metrics"]["discovery_effort_tier"], schema["properties"]["metrics"]["properties"]["discovery_effort_tier"]["enum"])
        self.assertIn(scorecard["attribution"]["primary_code"], schema["properties"]["attribution"]["properties"]["primary_code"]["enum"])

        # Also validate with jsonschema if available
        try:
            import jsonschema
            jsonschema.validate(instance=scorecard, schema=schema)
        except ImportError:
            pass

        self.assertEqual(scorecard["composite_verdict"], "Full Success")


if __name__ == "__main__":
    unittest.main()
