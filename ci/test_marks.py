#!/usr/bin/env python3
"""Tests for compact mark authoring helper `skills/montagent-footage/scripts/marks.py` (#1152, #951).

    python3 -m unittest ci/test_marks.py

Covers:
1. Expanding a 1,060-mark compact spec deterministically into <= 40 tracks with zero
   `R-KEYFRAME-UNREACHED` findings from `montagent validate`.
2. Stable `mark_<id>` identifiers and overrides across mark additions, removals, and reordering.
3. Orphaned override refusal (non-zero exit naming the orphaned ID) and `--prune-overrides`.
4. Guarded regeneration via `<project>.marks-manifest.json`: refusing on direct `.montagent.json`
   edits with a diff, folding edits back into `overrides` with `--adopt-edits`, and overwriting
   with `--force`, while preserving non-generated tracks/elements.
5. `--inspect` provenance table output (element ID, track, `[start, end)`, source mark entry,
   and override keys).
"""

import copy
import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SCRIPT = ROOT / "skills/montagent-footage/scripts/marks.py"
FIXTURES = ROOT / "fixtures/skills"


def montagent():
    for candidate in (
        os.environ.get("MONTAGENT"),
        str(ROOT / "target/debug/montagent"),
        str(ROOT / "target/release/montagent"),
        str(Path.home() / ".cargo/shared-target/debug/montagent"),
        str(Path.home() / ".cargo/shared-target/release/montagent"),
        shutil.which("montagent"),
    ):
        for path in (candidate, candidate and candidate + ".exe"):
            if path and Path(path).is_file():
                return str(Path(path).resolve())
    return None


def setUpModule():
    missing = [
        n
        for n, found in (("montagent", montagent()), ("ffmpeg", shutil.which("ffmpeg")))
        if not found
    ]
    if missing:
        message = "needs " + " and ".join(missing)
        if os.environ.get("CI"):
            raise AssertionError(message + ": CI must run the marks script's tests")
        raise unittest.SkipTest(message)


class MarksTest(unittest.TestCase):
    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self._tmp.cleanup)
        self.dir = Path(self._tmp.name) / "fx"
        shutil.copytree(FIXTURES, self.dir, ignore=shutil.ignore_patterns("rig"))
        self.project = self.dir / "captions.montagent.json"
        self.spec = self.dir / "marks.spec.json"
        self.env = dict(
            os.environ,
            PATH=str(Path(montagent()).parent) + os.pathsep + os.environ["PATH"],
        )

    def run_marks(self, *args, spec=None, project=None, env=None):
        cmd = [
            sys.executable,
            str(SCRIPT),
            str(spec or self.spec),
            str(project or self.project),
            *map(str, args),
        ]
        return subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            cwd=self.dir,
            env=env or self.env,
        )

    def validate_json(self, project_path=None):
        target = str(project_path or self.project)
        res = subprocess.run(
            [montagent(), "validate", target, "--json"],
            capture_output=True,
            text=True,
            cwd=self.dir,
            env=self.env,
        )
        self.assertEqual(res.returncode, 0, res.stdout + res.stderr)
        return json.loads(res.stdout)

    def test_1060_marks_pack_under_40_tracks_deterministically_with_zero_unreached_keyframes(self):
        # 60,000 ms timeline; 1,060 marks of duration 800 ms spaced every 50 ms
        # Max concurrency at any instant is ceil(800 / 50) = 16 <= 40 tracks.
        doc = json.loads(self.project.read_text())
        doc["duration"] = 60000
        doc["tracks"][0]["elements"][0]["end"] = 60000
        doc["tracks"][0]["elements"][0]["overrun"] = "hold"
        doc["tracks"] = [doc["tracks"][0]]
        # Change voice to a rect background so we don't overrun a 6s audio file over 60s
        doc["tracks"][0] = {
            "name": "bg",
            "layer": 1,
            "elements": [
                {
                    "id": "bg_card",
                    "type": "rect",
                    "start": 0,
                    "end": 60000,
                    "x": 540,
                    "y": 960,
                    "origin": "center",
                    "width": 1080,
                    "height": 1920,
                    "fill": "#101418",
                }
            ],
        }
        self.project.write_text(json.dumps(doc))

        marks = []
        for i in range(1060):
            start = i * 50
            end = start + 800
            marks.append(
                {
                    "type": "rect",
                    "start": start,
                    "end": end,
                    "label": f"beat_{i}",
                    "x": 100 + (i % 10) * 80,
                    "y": 200 + ((i // 10) % 15) * 100,
                }
            )
        spec_data = {
            "defaults": {
                "type": "rect",
                "origin": "center",
                "width": 40,
                "height": 40,
                "fill": "#FF5A36",
                "fade_in_ms": 100,
                "fade_out_ms": 150,
                "ease": "ease-in-out",
                "track_prefix": "marks",
                "max_tracks": 40,
                "layer": 10,
            },
            "marks": marks,
            "overrides": {
                "rect_0_beat_0": {"fill": "#00FFAA"},
            },
        }
        self.spec.write_text(json.dumps(spec_data, indent=2))

        first_run = self.run_marks()
        self.assertEqual(first_run.returncode, 0, first_run.stderr)
        first_doc = json.loads(self.project.read_text())

        # Run a second time to verify deterministic expansion
        second_run = self.run_marks()
        self.assertEqual(second_run.returncode, 0, second_run.stderr)
        second_doc = json.loads(self.project.read_text())
        self.assertEqual(first_doc, second_doc)
        self.assertEqual(first_run.stdout, second_run.stdout)

        mark_tracks = [t for t in first_doc["tracks"] if t["name"].startswith("marks_")]
        self.assertLessEqual(len(mark_tracks), 40)
        self.assertEqual(len(mark_tracks), 16)

        # Non-generated track `bg` is preserved
        bg_tracks = [t for t in first_doc["tracks"] if t["name"] == "bg"]
        self.assertEqual(len(bg_tracks), 1)
        self.assertEqual(bg_tracks[0]["elements"][0]["id"], "bg_card")

        # All 1,060 marks are present with `mark_` prefix
        generated_elements = [e for t in mark_tracks for e in t["elements"]]
        self.assertEqual(len(generated_elements), 1060)
        self.assertTrue(all(e["id"].startswith("mark_") for e in generated_elements))

        # Validate through `montagent validate` and verify 0 errors and 0 R-KEYFRAME-UNREACHED
        report = self.validate_json()
        self.assertEqual(report["summary"]["error"], 0)
        unreached = [
            f for f in report.get("findings", []) if f.get("code") == "R-KEYFRAME-UNREACHED"
        ]
        self.assertEqual(unreached, [])

    def test_stable_mark_ids_and_overrides_survive_insert_remove_and_reorder(self):
        spec_data = {
            "defaults": {
                "type": "rect",
                "origin": "center",
                "width": 120,
                "height": 60,
                "fill": "#FF5A36",
                "fade_out_ms": 120,
                "track_prefix": "cue",
            },
            "marks": [
                {"start": 200, "end": 1000, "label": "Alpha", "x": 200, "y": 300},
                {"id": "custom_beta", "start": 500, "end": 1500, "x": 500, "y": 600},
                {"start": 1200, "end": 2200, "label": "Gamma", "x": 800, "y": 900},
            ],
            "overrides": {
                "rect_200_alpha": {"fill": "#112233", "width": 180},
                "custom_beta": {"height": 90},
            },
        }
        self.spec.write_text(json.dumps(spec_data, indent=2))
        res1 = self.run_marks()
        self.assertEqual(res1.returncode, 0, res1.stderr)
        doc1 = json.loads(self.project.read_text())
        by_id1 = {e["id"]: e for t in doc1["tracks"] for e in t["elements"]}

        self.assertIn("mark_rect_200_alpha", by_id1)
        self.assertIn("mark_custom_beta", by_id1)
        self.assertIn("mark_rect_1200_gamma", by_id1)
        self.assertEqual(by_id1["mark_rect_200_alpha"]["fill"], "#112233")
        self.assertEqual(by_id1["mark_rect_200_alpha"]["width"], 180)
        self.assertEqual(by_id1["mark_custom_beta"]["height"], 90)

        # Reorder marks, remove Gamma, insert a new mark at the front
        spec_data["marks"] = [
            {"id": "custom_beta", "start": 500, "end": 1500, "x": 500, "y": 600},
            {"start": 50, "end": 400, "label": "Intro", "x": 100, "y": 100},
            {"start": 200, "end": 1000, "label": "Alpha", "x": 200, "y": 300},
        ]
        self.spec.write_text(json.dumps(spec_data, indent=2))
        res2 = self.run_marks()
        self.assertEqual(res2.returncode, 0, res2.stderr)
        doc2 = json.loads(self.project.read_text())
        by_id2 = {e["id"]: e for t in doc2["tracks"] for e in t["elements"]}

        self.assertNotIn("mark_rect_1200_gamma", by_id2)
        self.assertIn("mark_rect_50_intro", by_id2)
        # Sibling IDs and their overrides are completely unchanged
        self.assertEqual(by_id2["mark_rect_200_alpha"], by_id1["mark_rect_200_alpha"])
        self.assertEqual(by_id2["mark_custom_beta"], by_id1["mark_custom_beta"])

    def test_orphaned_override_refuses_unless_prune_overrides_is_passed(self):
        spec_data = {
            "defaults": {"type": "rect", "width": 100, "height": 100, "fill": "#FF5A36"},
            "marks": [
                {"id": "kept", "start": 0, "end": 1000, "x": 540, "y": 960},
            ],
            "overrides": {
                "kept": {"width": 140},
                "ghost_mark": {"width": 200},
            },
        }
        self.spec.write_text(json.dumps(spec_data, indent=2))

        bad = self.run_marks()
        self.assertNotEqual(bad.returncode, 0)
        self.assertIn("ghost_mark", bad.stderr)

        pruned = self.run_marks("--prune-overrides")
        self.assertEqual(pruned.returncode, 0, pruned.stderr)
        updated_spec = json.loads(self.spec.read_text())
        self.assertNotIn("ghost_mark", updated_spec.get("overrides", {}))
        self.assertIn("kept", updated_spec.get("overrides", {}))

    def test_guarded_regeneration_detects_direct_json_edit_and_supports_adopt_and_force(self):
        spec_data = {
            "defaults": {"type": "rect", "width": 100, "height": 100, "fill": "#FF5A36"},
            "marks": [
                {"id": "callout", "start": 200, "end": 1200, "x": 300, "y": 400},
            ],
            "overrides": {},
        }
        self.spec.write_text(json.dumps(spec_data, indent=2))
        first = self.run_marks()
        self.assertEqual(first.returncode, 0, first.stderr)

        manifest_path = Path(f"{self.project}.marks-manifest.json")
        self.assertTrue(manifest_path.is_file())

        # Directly edit `mark_callout` in `.montagent.json`
        doc = json.loads(self.project.read_text())
        for t in doc["tracks"]:
            for e in t["elements"]:
                if e["id"] == "mark_callout":
                    e["x"] = 777
                    e["fill"] = "#00FFAA"
        self.project.write_text(json.dumps(doc, indent=2))

        # Regenerating without flags must refuse and print the diff
        refused = self.run_marks()
        self.assertNotEqual(refused.returncode, 0)
        self.assertIn("mark_callout", refused.stderr)
        self.assertIn("777", refused.stderr)
        self.assertIn("#00FFAA", refused.stderr)

        # `--adopt-edits` folds the direct edits into `overrides` in the spec file
        adopted = self.run_marks("--adopt-edits")
        self.assertEqual(adopted.returncode, 0, adopted.stderr)
        updated_spec = json.loads(self.spec.read_text())
        self.assertEqual(updated_spec["overrides"]["callout"]["x"], 777)
        self.assertEqual(updated_spec["overrides"]["callout"]["fill"], "#00FFAA")

        # Now edit `.montagent.json` directly again and use `--force` to discard the hand edit
        doc = json.loads(self.project.read_text())
        for t in doc["tracks"]:
            for e in t["elements"]:
                if e["id"] == "mark_callout":
                    e["x"] = 999
        self.project.write_text(json.dumps(doc, indent=2))

        forced = self.run_marks("--force")
        self.assertEqual(forced.returncode, 0, forced.stderr)
        doc_after_force = json.loads(self.project.read_text())
        callout = next(
            e
            for t in doc_after_force["tracks"]
            for e in t["elements"]
            if e["id"] == "mark_callout"
        )
        self.assertEqual(callout["x"], 777)

    def test_inspect_prints_provenance_table(self):
        spec_data = {
            "defaults": {
                "type": "rect",
                "width": 100,
                "height": 100,
                "fill": "#FF5A36",
                "track_prefix": "marks",
            },
            "marks": [
                {"id": "first_cue", "start": 100, "end": 900, "x": 200, "y": 300},
                {"start": 500, "end": 1400, "label": "Second", "x": 400, "y": 500},
            ],
            "overrides": {
                "first_cue": {"fill": "#FFFFFF"},
            },
        }
        self.spec.write_text(json.dumps(spec_data, indent=2))
        res = self.run_marks("--inspect")
        self.assertEqual(res.returncode, 0, res.stderr)
        self.assertIn("mark_first_cue", res.stdout)
        self.assertIn("mark_rect_500_second", res.stdout)
        self.assertIn("marks_1", res.stdout)
        self.assertIn("marks_2", res.stdout)
        self.assertIn("[100, 900)", res.stdout)
        self.assertIn("[500, 1400)", res.stdout)
        self.assertIn("fill", res.stdout)


if __name__ == "__main__":
    unittest.main()
