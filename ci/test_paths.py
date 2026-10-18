import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
SCRIPTS_DIR = REPO_ROOT / "skills" / "montagent-motion" / "scripts"
sys.path.insert(0, str(SCRIPTS_DIR))

import paths  # noqa: E402


def _assert_valid_vertices(test_case: unittest.TestCase, pts: list, closed: bool = True, bounds=None):
    test_case.assertIsInstance(pts, list)
    test_case.assertGreaterEqual(len(pts), 3 if closed else 2)
    for idx, pt in enumerate(pts):
        test_case.assertIn("at", pt)
        at = pt["at"]
        test_case.assertEqual(len(at), 2)
        test_case.assertIsInstance(at[0], int)
        test_case.assertIsInstance(at[1], int)
        if bounds is not None:
            min_x, min_y, max_x, max_y = bounds
            test_case.assertGreaterEqual(at[0], min_x)
            test_case.assertLessEqual(at[0], max_x)
            test_case.assertGreaterEqual(at[1], min_y)
            test_case.assertLessEqual(at[1], max_y)
        for handle_key in ("in", "out"):
            if handle_key in pt:
                h = pt[handle_key]
                test_case.assertEqual(len(h), 2)
                test_case.assertIsInstance(h[0], int)
                test_case.assertIsInstance(h[1], int)
                if bounds is not None:
                    hx, hy = at[0] + h[0], at[1] + h[1]
                    test_case.assertGreaterEqual(hx, min_x)
                    test_case.assertLessEqual(hx, max_x)
                    test_case.assertGreaterEqual(hy, min_y)
                    test_case.assertLessEqual(hy, max_y)
        if not closed:
            if idx == 0:
                test_case.assertNotIn("in", pt, "Open path first vertex must not have 'in' handle")
            if idx == len(pts) - 1:
                test_case.assertNotIn("out", pt, "Open path last vertex must not have 'out' handle")


class TestPathsFunctions(unittest.TestCase):
    def test_taper_open_line_and_wave(self):
        line = [{"at": [40, 100]}, {"at": [360, 100]}]
        outline = paths.taper(
            line,
            start_width=4,
            end_width=28,
            start_length=0.3,
            end_length=0.3,
            start_ease=0.5,
            end_ease=0.5,
            width=24,
            cap="round",
            bounds=(0, 0, 400, 200),
        )
        _assert_valid_vertices(self, outline, closed=True, bounds=(0, 0, 400, 200))
        ys = [pt["at"][1] for pt in outline]
        self.assertLess(min(ys), 100)
        self.assertGreater(max(ys), 100)

        waved = paths.taper(
            line,
            start_width=6,
            end_width=6,
            width=20,
            wave_amount=0.5,
            wave_length=80,
            wave_phase=0.0,
            bounds=(0, 0, 400, 200),
        )
        _assert_valid_vertices(self, waved, closed=True, bounds=(0, 0, 400, 200))
        self.assertNotEqual(outline, waved)

    def test_boolean_operations(self):
        rect_a = [
            {"at": [50, 50]},
            {"at": [200, 50]},
            {"at": [200, 200]},
            {"at": [50, 200]},
        ]
        rect_b = [
            {"at": [120, 120]},
            {"at": [270, 120]},
            {"at": [270, 270]},
            {"at": [120, 270]},
        ]

        union_pts = paths.boolean([rect_a, rect_b], "union")
        _assert_valid_vertices(self, union_pts, closed=True)
        union_xs = [p["at"][0] for p in union_pts]
        union_ys = [p["at"][1] for p in union_pts]
        self.assertEqual(min(union_xs), 50)
        self.assertEqual(max(union_xs), 270)
        self.assertEqual(min(union_ys), 50)
        self.assertEqual(max(union_ys), 270)

        inter_pts = paths.boolean([rect_a, rect_b], "intersect")
        _assert_valid_vertices(self, inter_pts, closed=True)
        inter_xs = [p["at"][0] for p in inter_pts]
        inter_ys = [p["at"][1] for p in inter_pts]
        self.assertEqual(min(inter_xs), 120)
        self.assertEqual(max(inter_xs), 200)
        self.assertEqual(min(inter_ys), 120)
        self.assertEqual(max(inter_ys), 200)

        sub_pts = paths.boolean([rect_a, rect_b], "subtract")
        _assert_valid_vertices(self, sub_pts, closed=True)
        self.assertGreaterEqual(len(sub_pts), 6)

        excl_pts = paths.boolean([rect_a, rect_b], "exclude")
        _assert_valid_vertices(self, excl_pts, closed=True)

    def test_offset_joins(self):
        square = [
            {"at": [100, 100]},
            {"at": [200, 100]},
            {"at": [200, 200]},
            {"at": [100, 200]},
        ]
        for join in ("miter", "bevel", "round"):
            outward = paths.offset(square, 20, line_join=join, miter_limit=4)
            _assert_valid_vertices(self, outward, closed=True)
            xs = [p["at"][0] for p in outward]
            self.assertLessEqual(min(xs), 80)
            self.assertGreaterEqual(max(xs), 220)

        inward = paths.offset(square, -20, line_join="miter")
        _assert_valid_vertices(self, inward, closed=True)
        in_xs = [p["at"][0] for p in inward]
        self.assertEqual(min(in_xs), 120)
        self.assertEqual(max(in_xs), 180)

    def test_pucker_bloat(self):
        square = [
            {"at": [100, 100]},
            {"at": [300, 100]},
            {"at": [300, 300]},
            {"at": [100, 300]},
        ]
        bloated = paths.pucker_bloat(square, 40, bounds=(0, 0, 400, 400))
        _assert_valid_vertices(self, bloated, closed=True, bounds=(0, 0, 400, 400))
        self.assertTrue(any("in" in pt or "out" in pt for pt in bloated))

        puckered = paths.pucker_bloat(square, -40, bounds=(0, 0, 400, 400))
        _assert_valid_vertices(self, puckered, closed=True, bounds=(0, 0, 400, 400))
        self.assertNotEqual(bloated, puckered)

    def test_twist(self):
        poly = [
            {"at": [100, 100]},
            {"at": [200, 100]},
            {"at": [300, 100]},
            {"at": [300, 200]},
            {"at": [300, 300]},
            {"at": [200, 300]},
            {"at": [100, 300]},
            {"at": [100, 200]},
        ]
        twisted = paths.twist(poly, 90, center=(200, 200), radius=180, bounds=(0, 0, 400, 400))
        _assert_valid_vertices(self, twisted, closed=True, bounds=(0, 0, 400, 400))
        self.assertNotEqual(poly, twisted)

    def test_zig_zag_corner_and_smooth(self):
        line = [{"at": [50, 100]}, {"at": [350, 100]}]
        corner_zz = paths.zig_zag(line, ridges=4, size=20, smooth=False, closed=False, bounds=(0, 0, 400, 200))
        _assert_valid_vertices(self, corner_zz, closed=False, bounds=(0, 0, 400, 200))
        self.assertGreater(len(corner_zz), len(line))
        for pt in corner_zz:
            self.assertNotIn("in", pt)
            self.assertNotIn("out", pt)

        smooth_zz = paths.zig_zag(line, ridges=4, size=20, smooth=True, closed=False, bounds=(0, 0, 400, 200))
        _assert_valid_vertices(self, smooth_zz, closed=False, bounds=(0, 0, 400, 200))
        self.assertTrue(any("in" in pt or "out" in pt for pt in smooth_zz[1:-1]))

    def test_wiggle_deterministic_and_keyframed(self):
        base = [
            {"at": [80, 80]},
            {"at": [220, 80]},
            {"at": [220, 220]},
            {"at": [80, 220]},
        ]
        w1 = paths.wiggle(base, seed=42, octaves=2, amplitude=15, frequency=3, closed=True, bounds=(0, 0, 300, 300))
        w2 = paths.wiggle(base, seed=42, octaves=2, amplitude=15, frequency=3, closed=True, bounds=(0, 0, 300, 300))
        w3 = paths.wiggle(base, seed=99, octaves=2, amplitude=15, frequency=3, closed=True, bounds=(0, 0, 300, 300))
        self.assertEqual(w1, w2)
        self.assertNotEqual(w1, w3)
        _assert_valid_vertices(self, w1, closed=True, bounds=(0, 0, 300, 300))

        kfs = paths.wiggle(
            base,
            seed=7,
            octaves=2,
            amplitude=12,
            frequency=2,
            fps=10,
            start=0,
            end=500,
            closed=True,
            bounds=(0, 0, 300, 300),
        )
        self.assertGreaterEqual(len(kfs), 2)
        first_shape = [(len(pt["at"]), "in" in pt, "out" in pt) for pt in kfs[0]["v"]]
        for kf in kfs:
            self.assertIn("t", kf)
            self.assertIn("v", kf)
            _assert_valid_vertices(self, kf["v"], closed=True, bounds=(0, 0, 300, 300))
            shape = [(len(pt["at"]), "in" in pt, "out" in pt) for pt in kf["v"]]
            self.assertEqual(shape, first_shape)

    def test_trim_sequential_proportional_to_arc_length(self):
        # First path is 100px long, second path is 300px long -> 1:3 duration ratio
        p1 = [{"at": [0, 0]}, {"at": [100, 0]}]
        p2 = [{"at": [0, 0]}, {"at": [300, 0]}]
        trims = paths.trim_sequential([p1, p2], start=0, end=1000, ease="linear")
        self.assertEqual(len(trims), 2)
        kf1 = trims[0]["trim_end"]
        kf2 = trims[1]["trim_end"]
        self.assertEqual(kf1[0]["t"], 0)
        self.assertEqual(kf1[1]["t"], 250)
        self.assertEqual(kf2[0]["t"], 250)
        self.assertEqual(kf2[1]["t"], 1000)
        self.assertEqual(kf1[0]["v"], 0)
        self.assertEqual(kf1[1]["v"], 1)
        self.assertEqual(kf2[0]["v"], 0)
        self.assertEqual(kf2[1]["v"], 1)

    def test_cli_fixture_runs_and_validates(self):
        script = SCRIPTS_DIR / "paths.py"
        fixtures = REPO_ROOT / "fixtures" / "skills"
        project_path = fixtures / "paths.montagent.json"
        spec_path = fixtures / "paths.spec.json"
        self.assertTrue(project_path.exists(), f"Missing fixture {project_path}")
        self.assertTrue(spec_path.exists(), f"Missing spec {spec_path}")

        proc = subprocess.run(
            [sys.executable, str(script), str(project_path), str(spec_path)],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(proc.returncode, 0, msg=f"stderr: {proc.stderr}")
        project = json.loads(proc.stdout)
        self.assertIn("tracks", project)
        self.assertGreaterEqual(len(project["tracks"]), 5)

        unbaked_check = subprocess.run(
            [sys.executable, str(script), "--check", str(project_path), str(spec_path)],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(unbaked_check.returncode, 1)

        with tempfile.NamedTemporaryFile(mode="w", suffix=".montagent.json", delete=False) as tmp:
            tmp_baked = Path(tmp.name)
            tmp.write(proc.stdout)
        try:
            baked_check = subprocess.run(
                [sys.executable, str(script), "--check", str(tmp_baked), str(spec_path)],
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(baked_check.returncode, 0, msg=f"stderr: {baked_check.stderr}")
        finally:
            tmp_baked.unlink(missing_ok=True)

    def test_cli_all_operations_validate(self):
        script = SCRIPTS_DIR / "paths.py"
        fixtures = REPO_ROOT / "fixtures" / "skills"
        project_path = fixtures / "paths.montagent.json"
        spec = {
            "ops": [
                {
                    "op": "offset",
                    "track": "offset_track",
                    "layer": 10,
                    "id": "offset_box",
                    "start": 0,
                    "end": 3000,
                    "x": 50,
                    "y": 50,
                    "width": 300,
                    "height": 300,
                    "fill": "#22C55E",
                    "distance": 20,
                    "line_join": "round",
                    "points": [
                        {"at": [80, 80]},
                        {"at": [220, 80]},
                        {"at": [220, 220]},
                        {"at": [80, 220]},
                    ],
                },
                {
                    "op": "twist",
                    "track": "twist_track",
                    "layer": 20,
                    "id": "twisted_star",
                    "start": 0,
                    "end": 3000,
                    "x": 400,
                    "y": 50,
                    "width": 300,
                    "height": 300,
                    "fill": "#A855F7",
                    "angle": 120,
                    "points": [
                        {"at": [60, 60]},
                        {"at": [150, 60]},
                        {"at": [240, 60]},
                        {"at": [240, 150]},
                        {"at": [240, 240]},
                        {"at": [150, 240]},
                        {"at": [60, 240]},
                        {"at": [60, 150]},
                    ],
                },
                {
                    "op": "wiggle",
                    "track": "wiggle_track",
                    "layer": 30,
                    "id": "wiggle_blob",
                    "start": 0,
                    "end": 2000,
                    "x": 50,
                    "y": 400,
                    "width": 300,
                    "height": 300,
                    "fill": "#EC4899",
                    "seed": 11,
                    "octaves": 2,
                    "amplitude": 14,
                    "frequency": 3,
                    "animate": True,
                    "points": [
                        {"at": [70, 70], "in": [-20, 20], "out": [20, -20]},
                        {"at": [230, 70], "in": [-20, -20], "out": [20, 20]},
                        {"at": [230, 230], "in": [20, -20], "out": [-20, 20]},
                        {"at": [70, 230], "in": [20, 20], "out": [-20, -20]},
                    ],
                },
                {
                    "op": "boolean",
                    "mode": "subtract",
                    "holes_as_mask": True,
                    "track": "donut_track",
                    "layer": 40,
                    "id": "donut_cutout",
                    "start": 0,
                    "end": 3000,
                    "x": 400,
                    "y": 400,
                    "width": 320,
                    "height": 320,
                    "fill": "#F97316",
                    "paths": [
                        [
                            {"at": [40, 40]},
                            {"at": [280, 40]},
                            {"at": [280, 280]},
                            {"at": [40, 280]},
                        ],
                        [
                            {"at": [110, 110]},
                            {"at": [210, 110]},
                            {"at": [210, 210]},
                            {"at": [110, 210]},
                        ],
                    ],
                },
            ]
        }
        with tempfile.NamedTemporaryFile(mode="w", suffix=".spec.json", delete=False) as tmp:
            tmp_spec = Path(tmp.name)
            json.dump(spec, tmp)
        try:
            proc = subprocess.run(
                [sys.executable, str(script), str(project_path), str(tmp_spec)],
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(proc.returncode, 0, msg=f"stderr: {proc.stderr}")
            out_proj = json.loads(proc.stdout)
            donut = [
                el
                for t in out_proj["tracks"]
                for el in t["elements"]
                if el["id"] == "donut_cutout"
            ][0]
            self.assertIn("effects", donut)
            self.assertTrue(donut["effects"][0]["invert"])
        finally:
            tmp_spec.unlink(missing_ok=True)


if __name__ == "__main__":
    unittest.main()
