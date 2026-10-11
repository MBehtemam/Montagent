"""CLI acceptance tests for the non-normative ink-on-paper stabilizer (#934)."""
import hashlib
import json
import math
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "skills/montagent-prerender/references/stabilize.py"
W, H, N = 96, 80, 8


def run(*args, **kwargs):
    return subprocess.run(list(map(str, args)), capture_output=True, **kwargs)


class Stabilize(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        frames = bytearray()
        for i in range(N):
            for y in range(H):
                for x in range(W):
                    radius = 9 + i * 0.35
                    value = round(240 - 190 * math.exp(-((x - 43 - i) ** 2 + (y - 38) ** 2) / (2 * radius ** 2)))
                    frames.extend((value, value, value, 255))
        self.encode(frames)
        self.spec = dict(source="source.mov", name="locked", out="media", width=W,
                         height=H, fps=25, reference_frame=3, fill="paper", paper=[240, 240, 240])

    def encode(self, frames):
        encoded = run("ffmpeg", "-y", "-v", "error", "-f", "rawvideo", "-pix_fmt", "rgba",
                      "-s", f"{W}x{H}", "-r", "25", "-i", "-", "-c:v", "png",
                      "-pix_fmt", "rgba", self.root / "source.mov", input=bytes(frames))
        self.assertEqual(encoded.returncode, 0, encoded.stderr)

    def build(self, **overrides):
        (self.root / "spec.json").write_text(json.dumps(self.spec | overrides))
        return run(sys.executable, SCRIPT, "build", self.root / "spec.json")

    def test_bakes_lossless_clip_with_recipe_and_rebuild(self):
        result = self.build()
        self.assertEqual(result.returncode, 0, result.stderr.decode())
        recipe_dir = self.root / "media/locked.recipe"
        recipe = json.loads((recipe_dir / "recipe.json").read_text())
        self.assertEqual(recipe["frames"], N)
        self.assertEqual(recipe["stabilize"]["reference_frame"], 3)
        self.assertEqual(recipe["stabilize"]["fit_model"], "cubic")
        self.assertEqual(recipe["stabilize"]["source_sha256"], hashlib.sha256((self.root / "source.mov").read_bytes()).hexdigest())
        decoded = run("ffmpeg", "-v", "error", "-i", self.root / "media/locked.mov",
                      "-f", "rawvideo", "-pix_fmt", "rgba", "-").stdout
        size = W * H * 4
        self.assertEqual(recipe["decoded_frame_sha256"],
                         [hashlib.sha256(decoded[i:i + size]).hexdigest() for i in range(0, len(decoded), size)])
        # Independent oracle: the Gaussian's half-height contour is locked to
        # the known reference (46,38), with diameter 2*sigma*sqrt(2*ln(2)).
        expected = 2 * (9 + 3 * 0.35) * math.sqrt(2 * math.log(2))
        diameters = []
        for start in range(0, len(decoded), size):
            frame = decoded[start:start + size]
            row = [frame[(38 * W + x) * 4] for x in range(W)]
            crossings = [x + (145 - a) / (b - a) for x, (a, b) in
                         enumerate(zip(row, row[1:])) if (a < 145 <= b) or (b < 145 <= a)]
            self.assertEqual(len(crossings), 2)
            self.assertAlmostEqual(sum(crossings) / 2, 46, delta=0.15)
            diameters.append(crossings[1] - crossings[0])
            self.assertTrue(all(frame[(y * W + x) * 4:(y * W + x) * 4 + 4] == bytes((240, 240, 240, 255))
                                for x, y in [(0, 0), (W - 1, 0), (0, H - 1), (W - 1, H - 1)]))
        self.assertTrue(all(abs(d / expected - 1) < 0.01 for d in diameters), diameters)
        self.assertIn("residual size drift", result.stderr.decode())
        rebuilt = run(sys.executable, SCRIPT, "rebuild", recipe_dir)
        self.assertEqual(rebuilt.returncode, 0, rebuilt.stderr.decode())
        self.assertIn(b"rebuild: match", rebuilt.stdout)

    def test_edge_fill_ping_pong_and_slowdown_are_baked(self):
        result = self.build(fill="edge", ping_pong=True, slowdown=2)
        self.assertEqual(result.returncode, 0, result.stderr.decode())
        recipe = json.loads((self.root / "media/locked.recipe/recipe.json").read_text())
        hashes = recipe["decoded_frame_sha256"]
        self.assertEqual(recipe["frames"], 28)
        self.assertEqual(hashes[::2], hashes[:16:2] + hashes[12:0:-2])
        self.assertEqual(hashes[::2], hashes[1::2])

    def test_rebuild_refuses_changed_source_and_version_pins(self):
        self.assertEqual(self.build().returncode, 0)
        recipe = self.root / "media/locked.recipe"
        source = recipe / "source/source.mov"
        original = source.read_bytes()
        source.write_bytes(original + b"changed")
        result = run(sys.executable, SCRIPT, "rebuild", recipe)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(b"source hash differs", result.stderr)
        source.write_bytes(original)
        settings_path = recipe / "source/stabilize.json"
        settings = json.loads(settings_path.read_text())
        settings["versions"]["python"] = "0.0.0"
        settings_path.write_text(json.dumps(settings))
        result = run(sys.executable, SCRIPT, "rebuild", recipe)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(b"tool versions differ", result.stderr)

    def test_rejects_invalid_reference_fill_and_timing(self):
        for override, message in [({"reference_frame": 8}, b"reference_frame"),
                                  ({"fill": "black"}, b"fill must"),
                                  ({"slowdown": 0}, b"slowdown"),
                                  ({"fps": 30}, b"fps must match"),
                                  ({"width": 48}, b"size must match")]:
            with self.subTest(override=override):
                result = self.build(**override)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn(message, result.stderr)

    def test_blank_ink_and_unexplained_size_jumps_fail_acceptance(self):
        self.encode(bytes((240, 240, 240, 255)) * (W * H * N))
        result = self.build()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(b"no measurable ink", result.stderr)
        frames = bytearray()
        for i in range(N):
            for y in range(H):
                for x in range(W):
                    radius = 7 if i % 2 else 13
                    value = round(240 - 190 * math.exp(-((x - 48) ** 2 + (y - 40) ** 2) / (2 * radius ** 2)))
                    frames.extend((value, value, value, 255))
        self.encode(frames)
        result = self.build()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(b"residual size drift", result.stderr)
        self.assertFalse((self.root / "media/locked.mov").exists())


if __name__ == "__main__":
    unittest.main()
