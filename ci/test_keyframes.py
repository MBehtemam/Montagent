#!/usr/bin/env python3
"""Tests for skills/montagent-motion/scripts/keyframes.py (ADR-0217, #1089)."""

import importlib.util
import json
import math
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SCRIPT = ROOT / "skills/montagent-motion/scripts/keyframes.py"
FIXTURES = ROOT / "fixtures/skills"


def load_module():
    spec = importlib.util.spec_from_file_location("keyframes", SCRIPT)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def montagent_env() -> dict:
    env = os.environ.copy()
    for candidate in (ROOT / "target/debug", ROOT / "target/release"):
        if (candidate / "montagent").exists():
            env["PATH"] = f"{candidate}{os.pathsep}{env.get('PATH', '')}"
            break
    return env


def base_project() -> dict:
    return {
        "frame": {"width": 1080, "height": 1920},
        "fps": 30,
        "background": "#111318",
        "duration": 4000,
        "output": "out/test.mp4",
        "tracks": [
            {
                "name": "main",
                "layer": 1,
                "elements": [
                    {
                        "id": "card",
                        "type": "rect",
                        "start": 0,
                        "end": 4000,
                        "x": 540,
                        "y": 960,
                        "origin": "center",
                        "width": 400,
                        "height": 240,
                        "fill": "#FF5A36",
                        "scale": [1.0, 1.0],
                        "rotation": 0.0,
                        "opacity": 1.0,
                        "effects": [{"name": "blur", "radius": 4.0}],
                    }
                ],
            },
            {
                "name": "sound",
                "layer": 0,
                "elements": [
                    {
                        "id": "bed",
                        "type": "audio",
                        "start": 0,
                        "end": 4000,
                        "source": "media/beat.wav",
                        "source_start": 0,
                        "source_end": 4000,
                        "volume": 0.8,
                    }
                ],
            },
        ],
    }


def get_element(project: dict, el_id: str) -> dict:
    for track in project.get("tracks", []):
        for el in track.get("elements", []):
            if el.get("id") == el_id:
                return el
    raise KeyError(el_id)


class TestKeyframesScript(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.env = montagent_env()
        if not shutil.which("montagent", path=cls.env.get("PATH")):
            subprocess.run(
                ["cargo", "build", "--bin", "montagent"],
                cwd=ROOT,
                check=True,
            )
            cls.env = montagent_env()

    def run_cli(self, project_path: Path, spec_path: Path, *extra_args: str) -> subprocess.CompletedProcess:
        return subprocess.run(
            [sys.executable, str(SCRIPT), str(project_path), str(spec_path), *extra_args],
            capture_output=True,
            text=True,
            env=self.env,
        )

    def test_wiggle_determinism_by_seed_and_drawn_frame_quantization(self):
        mod = load_module()
        proj1 = base_project()
        proj2 = base_project()
        proj3 = base_project()

        spec_seed_7 = {
            "operations": [
                {
                    "op": "wiggle",
                    "id": "card",
                    "property": "x",
                    "start": 0,
                    "end": 1000,
                    "freq": 6.0,
                    "amp": 24.0,
                    "octaves": 2,
                    "mult": 0.5,
                    "seed": 7,
                    "tol": 0.5,
                }
            ]
        }
        spec_seed_99 = {
            "operations": [
                {
                    **spec_seed_7["operations"][0],
                    "seed": 99,
                }
            ]
        }

        out1 = mod.apply_spec(proj1, spec_seed_7)
        out2 = mod.apply_spec(proj2, spec_seed_7)
        out3 = mod.apply_spec(proj3, spec_seed_99)

        kfs1 = get_element(out1, "card")["x"]
        kfs2 = get_element(out2, "card")["x"]
        kfs3 = get_element(out3, "card")["x"]

        self.assertEqual(kfs1, kfs2, "same seed must produce identical keyframes")
        self.assertNotEqual(kfs1, kfs3, "different seeds must produce different keyframes")
        self.assertGreater(len(kfs1), 2)
        self.assertNotIn("ease", kfs1[0])
        for kf in kfs1[1:]:
            self.assertIn("ease", kf)
        for kf in kfs1:
            self.assertIsInstance(kf["v"], int, "integer property x must have int keyframe values")
            # Drawn frame at 30 fps: floor(round(t * 30 / 1000) * 1000 / 30) == t
            expected_t = math.floor(round(kf["t"] * 30 / 1000.0) * 1000.0 / 30.0)
            self.assertEqual(kf["t"], expected_t)

        # Idempotent re-run on already-wiggled project
        out1_rerun = mod.apply_spec(out1, spec_seed_7)
        self.assertEqual(get_element(out1_rerun, "card")["x"], kfs1)

    def test_bounce_decay_and_settling(self):
        mod = load_module()
        proj = base_project()
        card = get_element(proj, "card")
        card["y"] = [
            {"t": 0, "v": 200},
            {"t": 400, "v": 960, "ease": "linear"},
        ]
        spec = {
            "operations": [
                {
                    "op": "bounce",
                    "id": "card",
                    "property": "y",
                    "start": 400,
                    "end": 1600,
                    "freq": 3.0,
                    "decay": 5.0,
                    "tol": 0.5,
                }
            ]
        }
        out = mod.apply_spec(proj, spec)
        kfs = get_element(out, "card")["y"]
        self.assertEqual(kfs[0], {"t": 0, "v": 200})
        self.assertEqual(kfs[1]["t"], 400)
        self.assertEqual(kfs[1]["v"], 960)
        self.assertEqual(kfs[-1]["v"], 960, "bounce must settle back to v1")

        # Measure peak excursions across consecutive half-cycles to verify decay
        tail = [kf for kf in kfs if kf["t"] >= 400]
        max_overshoot = max(kf["v"] - 960 for kf in tail)
        min_undershoot = min(kf["v"] - 960 for kf in tail)
        self.assertGreater(max_overshoot, 10, "positive incoming velocity must overshoot above 960")
        self.assertLess(min_undershoot, 0, "second half-cycle must undershoot below 960")
        self.assertGreater(
            max_overshoot,
            abs(min_undershoot),
            "exponential decay must shrink successive half-cycle peaks",
        )

        # Idempotent re-run
        out_rerun = mod.apply_spec(out, spec)
        self.assertEqual(get_element(out_rerun, "card")["y"], kfs)

    def test_loop_cycle_offset_and_pingpong_bezier_mirroring(self):
        mod = load_module()
        proj = base_project()
        card = get_element(proj, "card")
        # Multi-segment slice with ease-in, ease-out, and raw cubic-bezier [x1, y1, x2, y2]
        card["x"] = [
            {"t": 0, "v": 100},
            {"t": 200, "v": 300, "ease": "ease-in"},
            {"t": 500, "v": 700, "ease": [0.2, 0.1, 0.8, 0.9]},
        ]
        spec_pingpong = {
            "operations": [
                {
                    "op": "loop",
                    "id": "card",
                    "property": "x",
                    "mode": "pingpong",
                    "start": 0,
                    "end": 1500,
                }
            ]
        }
        out_pp = mod.apply_spec(proj, spec_pingpong)
        kfs_pp = get_element(out_pp, "card")["x"]
        # Forward (0..500): (0, 100), (200, 300, ease-in), (500, 700, [0.2, 0.1, 0.8, 0.9])
        # Reverse (500..1000):
        #   First reversed segment has length 500-200=300 -> t=800, v=300, mirrored bezier [1-0.8, 1-0.9, 1-0.2, 1-0.1] = [0.2, 0.1, 0.8, 0.9]
        #   Let's test an asymmetric bezier [0.1, 0.2, 0.7, 0.95] -> [0.3, 0.05, 0.9, 0.8]!
        card["x"] = [
            {"t": 0, "v": 100},
            {"t": 200, "v": 300, "ease": "ease-in"},
            {"t": 500, "v": 700, "ease": [0.1, 0.2, 0.7, 0.95]},
        ]
        out_pp = mod.apply_spec(proj, spec_pingpong)
        kfs_pp = get_element(out_pp, "card")["x"]
        by_t = {kf["t"]: kf for kf in kfs_pp}
        self.assertIn(800, by_t)
        self.assertEqual(by_t[800]["v"], 300)
        self.assertEqual(by_t[800]["ease"], [0.3, 0.05, 0.9, 0.8])
        self.assertIn(1000, by_t)
        self.assertEqual(by_t[1000]["v"], 100)
        self.assertEqual(by_t[1000]["ease"], "ease-out")
        # Third pass (1000..1500) is forward again
        self.assertEqual(by_t[1200]["v"], 300)
        self.assertEqual(by_t[1200]["ease"], "ease-in")
        self.assertEqual(by_t[1500]["v"], 700)
        self.assertEqual(by_t[1500]["ease"], [0.1, 0.2, 0.7, 0.95])

        # Idempotent re-run on pingpong
        out_pp_rerun = mod.apply_spec(out_pp, spec_pingpong)
        self.assertEqual(get_element(out_pp_rerun, "card")["x"], kfs_pp)

        # Offset mode
        proj_off = base_project()
        get_element(proj_off, "card")["x"] = [
            {"t": 0, "v": 100},
            {"t": 500, "v": 250, "ease": "ease-in-out"},
        ]
        spec_offset = {
            "operations": [
                {
                    "op": "loop",
                    "id": "card",
                    "property": "x",
                    "mode": "offset",
                    "start": 0,
                    "end": 1500,
                }
            ]
        }
        out_off = mod.apply_spec(proj_off, spec_offset)
        kfs_off = get_element(out_off, "card")["x"]
        self.assertEqual(
            kfs_off,
            [
                {"t": 0, "v": 100},
                {"t": 500, "v": 250, "ease": "ease-in-out"},
                {"t": 1000, "v": 400, "ease": "ease-in-out"},
                {"t": 1500, "v": 550, "ease": "ease-in-out"},
            ],
        )

    def test_smooth_reduces_dense_keyframes(self):
        mod = load_module()
        proj = base_project()
        card = get_element(proj, "card")
        # Dense noisy linear ramp across 31 frames (0..1000ms at 30fps)
        dense = []
        for n in range(31):
            t = math.floor(n * 1000.0 / 30.0)
            jitter = (3 if n % 2 == 0 else -3) if 0 < n < 30 else 0
            kf = {"t": t, "v": round(100 + 400 * (t / 1000.0) + jitter)}
            if n > 0:
                kf["ease"] = "linear"
            dense.append(kf)
        card["x"] = dense

        spec = {
            "operations": [
                {
                    "op": "smooth",
                    "id": "card",
                    "property": "x",
                    "start": 0,
                    "end": 1000,
                    "window_ms": 100,
                    "tol": 4.0,
                }
            ]
        }
        out = mod.apply_spec(proj, spec)
        thinned = get_element(out, "card")["x"]
        self.assertLess(len(thinned), 6, f"expected heavily thinned curve, got {len(thinned)} keyframes")
        self.assertEqual(thinned[0]["t"], 0)
        self.assertEqual(thinned[0]["v"], 100)
        self.assertEqual(thinned[-1]["t"], 1000)
        self.assertEqual(thinned[-1]["v"], 500)

    def test_exp_scale_endpoints_and_geometric_midpoint(self):
        mod = load_module()
        proj = base_project()
        spec = {
            "operations": [
                {
                    "op": "exp_scale",
                    "id": "card",
                    "property": "scale",
                    "start": 0,
                    "end": 1000,
                    "from": [1.0, 1.0],
                    "to": [4.0, 9.0],
                    "tol": 0.0005,
                }
            ]
        }
        out = mod.apply_spec(proj, spec)
        kfs = get_element(out, "card")["scale"]
        self.assertEqual(kfs[0], {"t": 0, "v": [1.0, 1.0]})
        self.assertEqual(kfs[-1]["t"], 1000)
        self.assertEqual(kfs[-1]["v"], [4.0, 9.0])

        # Geometric midpoint at t = 500ms: sqrt(1 * 4) = 2.0, sqrt(1 * 9) = 3.0
        mid_val = mod.eval_keyframes(kfs, 500.0)
        self.assertAlmostEqual(mid_val[0], 2.0, places=2)
        self.assertAlmostEqual(mid_val[1], 3.0, places=2)

    def test_audio_rms_mapping_check_and_validate_cli(self):
        with tempfile.TemporaryDirectory() as tmp:
            tmp_path = Path(tmp)
            shutil.copytree(FIXTURES / "media", tmp_path / "media")
            proj = base_project()
            proj_path = tmp_path / "project.montagent.json"
            spec_path = tmp_path / "keyframes.spec.json"
            proj_path.write_text(json.dumps(proj, indent=2) + "\n")

            spec = {
                "operations": [
                    {
                        "op": "audio",
                        "id": "card",
                        "property": "scale",
                        "audio_id": "bed",
                        "start": 0,
                        "end": 2000,
                        "floor_db": -40.0,
                        "peak_db": -6.0,
                        "min_v": [1.0, 1.0],
                        "max_v": [1.4, 1.4],
                        "tol": 0.01,
                    },
                    {
                        "op": "wiggle",
                        "id": "card",
                        "property": "effects[0].radius",
                        "start": 0,
                        "end": 2000,
                        "freq": 4.0,
                        "amp": 2.0,
                        "seed": 11,
                        "tol": 0.1,
                    },
                ]
            }
            spec_path.write_text(json.dumps(spec, indent=2) + "\n")

            # Before running, --check must detect drift (exit 1)
            chk_before = self.run_cli(proj_path, spec_path, "--check")
            self.assertEqual(chk_before.returncode, 1)
            self.assertEqual(chk_before.stdout, "")

            # Run CLI to generate keyframes and validate via montagent validate
            res = self.run_cli(proj_path, spec_path)
            self.assertEqual(res.returncode, 0, msg=res.stderr)
            updated = json.loads(res.stdout)
            card = get_element(updated, "card")
            scale_kfs = card["scale"]
            blur_kfs = card["effects"][0]["radius"]
            self.assertIsInstance(scale_kfs, list)
            self.assertGreater(len(scale_kfs), 3)
            self.assertIsInstance(blur_kfs, list)
            self.assertGreater(len(blur_kfs), 2)

            # Values must lie within [min_v, max_v] and show variation over beat.wav
            scales_x = [kf["v"][0] for kf in scale_kfs]
            self.assertGreaterEqual(min(scales_x), 1.0)
            self.assertLessEqual(max(scales_x), 1.4)
            self.assertGreater(max(scales_x) - min(scales_x), 0.05)

            # Write updated project and verify --check exits 0
            proj_path.write_text(res.stdout)
            chk_after = self.run_cli(proj_path, spec_path, "--check")
            self.assertEqual(chk_after.returncode, 0, msg=chk_after.stderr)
            self.assertEqual(chk_after.stdout, "")

            # Idempotent re-run produces identical output
            res2 = self.run_cli(proj_path, spec_path)
            self.assertEqual(res2.returncode, 0, msg=res2.stderr)
            self.assertEqual(json.loads(res2.stdout), updated)

    def test_auto_bezier_c1_velocity_continuity_and_handles(self):
        mod = load_module()
        proj = base_project()
        card = get_element(proj, "card")
        # 4 keyframes (3 monotonic segments) with unequal durations and secants
        card["x"] = [
            {"t": 0, "v": 100},
            {"t": 400, "v": 260, "ease": "ease-in-out"},
            {"t": 1000, "v": 740, "ease": "ease-in-out"},
            {"t": 1800, "v": 1060, "ease": "ease-in-out"},
        ]
        spec = {
            "operations": [
                {
                    "op": "auto_bezier",
                    "id": "card",
                    "property": "x",
                    "start": 0,
                    "end": 1800,
                }
            ]
        }
        out = mod.apply_spec(proj, spec)
        kfs = get_element(out, "card")["x"]
        self.assertEqual(len(kfs), 4)
        self.assertNotIn("ease", kfs[0])

        for j in range(1, len(kfs)):
            ease = kfs[j]["ease"]
            self.assertIsInstance(ease, list)
            self.assertEqual(len(ease), 4)
            self.assertAlmostEqual(ease[0], round(1.0 / 3.0, 6), places=6)
            self.assertAlmostEqual(ease[2], round(2.0 / 3.0, 6), places=6)
            for coord in ease:
                self.assertEqual(coord, round(coord, 6))

        # Default boundary slopes m_0 = 0 -> y1 = 0 on first segment; m_n = 0 -> y2 = 1 on last segment
        self.assertAlmostEqual(kfs[1]["ease"][1], 0.0, places=6)
        self.assertAlmostEqual(kfs[-1]["ease"][3], 1.0, places=6)

        # Verify C^1 physical velocity continuity across every interior keyframe k_i
        for i in range(1, len(kfs) - 1):
            dt_in = kfs[i]["t"] - kfs[i - 1]["t"]
            dv_in = kfs[i]["v"] - kfs[i - 1]["v"]
            x1_in, y1_in, x2_in, y2_in = kfs[i]["ease"]
            v_in = ((1.0 - y2_in) / (1.0 - x2_in)) * (dv_in / dt_in)

            dt_out = kfs[i + 1]["t"] - kfs[i]["t"]
            dv_out = kfs[i + 1]["v"] - kfs[i]["v"]
            x1_out, y1_out, x2_out, y2_out = kfs[i + 1]["ease"]
            v_out = (y1_out / x1_out) * (dv_out / dt_out)

            self.assertGreater(v_in, 0.0, "interior monotonic keyframe must not pause at 0 velocity")
            self.assertAlmostEqual(v_in, v_out, places=4)

            # Verify Fritsch-Carlson weighted harmonic mean value
            sec_in = dv_in / dt_in
            sec_out = dv_out / dt_out
            w1 = 2.0 * dt_out + dt_in
            w2 = dt_out + 2.0 * dt_in
            expected_m = (w1 + w2) / (w1 / sec_in + w2 / sec_out)
            self.assertAlmostEqual(v_in, expected_m, places=4)

    def test_auto_bezier_monotonicity_at_extrema_and_steep_secants(self):
        mod = load_module()
        proj = base_project()
        card = get_element(proj, "card")
        # Includes a local maximum at t=400 (v=500), local minimum at t=800 (v=200),
        # and a shallow-to-steep monotonic pass-through at t=1200 (v=220 -> 800)
        card["x"] = [
            {"t": 0, "v": 100},
            {"t": 400, "v": 500, "ease": "linear"},
            {"t": 800, "v": 200, "ease": "linear"},
            {"t": 1200, "v": 220, "ease": "linear"},
            {"t": 1600, "v": 800, "ease": "linear"},
        ]
        spec = {
            "operations": [
                {
                    "op": "auto_bezier",
                    "id": "card",
                    "property": "x",
                    "start": 0,
                    "end": 1600,
                }
            ]
        }
        out = mod.apply_spec(proj, spec)
        kfs = get_element(out, "card")["x"]

        # Local maximum at k_1 (t=400) and local minimum at k_2 (t=800) must clamp m_i = 0:
        # incoming y2 == 1.0, outgoing y1 == 0.0
        self.assertAlmostEqual(kfs[1]["ease"][3], 1.0, places=6)
        self.assertAlmostEqual(kfs[2]["ease"][1], 0.0, places=6)
        self.assertAlmostEqual(kfs[2]["ease"][3], 1.0, places=6)
        self.assertAlmostEqual(kfs[3]["ease"][1], 0.0, places=6)

        # Sample every segment at 1ms resolution and verify strict monotonicity & no overshoot
        for j in range(1, len(kfs)):
            t_start, v_start = kfs[j - 1]["t"], float(kfs[j - 1]["v"])
            t_end, v_end = kfs[j]["t"], float(kfs[j]["v"])
            lo_v, hi_v = min(v_start, v_end), max(v_start, v_end)
            prev_val = v_start
            for t in range(t_start, t_end + 1):
                val = mod.eval_keyframes(kfs, float(t))
                self.assertGreaterEqual(val, lo_v - 1e-6, f"undershoot at t={t} on seg {j}")
                self.assertLessEqual(val, hi_v + 1e-6, f"overshoot at t={t} on seg {j}")
                if v_end >= v_start:
                    self.assertGreaterEqual(val, prev_val - 1e-6, f"non-monotonic rise at t={t}")
                else:
                    self.assertLessEqual(val, prev_val + 1e-6, f"non-monotonic fall at t={t}")
                prev_val = val

    def test_auto_bezier_cli_validate_and_idempotence(self):
        with tempfile.TemporaryDirectory() as tmp:
            tmp_path = Path(tmp)
            shutil.copytree(FIXTURES / "media", tmp_path / "media")
            proj = base_project()
            card = get_element(proj, "card")
            card["x"] = [
                {"t": 0, "v": 100},
                {"t": 500, "v": 300, "ease": "ease-in-out"},
                {"t": 1000, "v": 900, "ease": "ease-in-out"},
            ]
            proj_path = tmp_path / "project.montagent.json"
            spec_path = tmp_path / "keyframes.spec.json"
            proj_path.write_text(json.dumps(proj, indent=2) + "\n")
            spec = {
                "operations": [
                    {
                        "op": "auto_bezier",
                        "id": "card",
                        "property": "x",
                        "start": 0,
                        "end": 1000,
                    }
                ]
            }
            spec_path.write_text(json.dumps(spec, indent=2) + "\n")

            chk_before = self.run_cli(proj_path, spec_path, "--check")
            self.assertEqual(chk_before.returncode, 1)

            res = self.run_cli(proj_path, spec_path)
            self.assertEqual(res.returncode, 0, msg=res.stderr)
            updated = json.loads(res.stdout)
            kfs = get_element(updated, "card")["x"]
            self.assertEqual(
                kfs,
                [
                    {"t": 0, "v": 100},
                    {"t": 500, "v": 300, "ease": [0.333333, 0.0, 0.666667, 0.5]},
                    {"t": 1000, "v": 900, "ease": [0.333333, 0.166667, 0.666667, 1.0]},
                ],
            )

            proj_path.write_text(res.stdout)
            chk_after = self.run_cli(proj_path, spec_path, "--check")
            self.assertEqual(chk_after.returncode, 0, msg=chk_after.stderr)

            res2 = self.run_cli(proj_path, spec_path)
            self.assertEqual(res2.returncode, 0, msg=res2.stderr)
            self.assertEqual(json.loads(res2.stdout), updated)


if __name__ == "__main__":
    unittest.main()
