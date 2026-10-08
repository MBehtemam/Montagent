#!/usr/bin/env python3
"""The Lottie route of `skills/montagent-prerender/scripts/prerender.py` (#811, ADR-0171 §2-§4).

    python3 -m unittest ci/test_prerender_lottie.py

- The machinery tests use a stand-in player written into the scratch folder, so they need only
  Python and ffmpeg: one process per frame, the refusals, the recipe and the rebuild.
- The ThorVG tests run the skill's own sample Lottie through its sample player and skip when
  `thorvg_python` is not installed (`pip install thorvg-python==1.1.3`). `place` and `validate`
  also need a `montagent` binary (`MONTAGENT`, `target/debug/montagent`, or `PATH`).
"""

import copy
import hashlib
import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SKILL = ROOT / "skills/montagent-prerender"
SCRIPT = SKILL / "scripts/prerender.py"
FIXTURES = ROOT / "fixtures/skills"
FONT = FIXTURES / "fonts/Inter-Bold.ttf"
W, H = 8, 4

try:
    import thorvg_python  # noqa: F401
    HAVE_THORVG = True
except ImportError:
    HAVE_THORVG = False


def prop(v):
    return {"a": 0, "k": v}


LOTTIE = {
    "v": "5.7.0", "fr": 25, "ip": 0, "op": 6, "w": 160, "h": 90, "assets": [],
    "fonts": {"list": [{"fName": "Inter-Bold", "fFamily": "Inter", "fStyle": "Bold"}]},
    "chars": [{"ch": "H", "fFamily": "Inter", "style": "Bold", "size": 100, "w": 70, "data": {"shapes": []}}],
    "layers": [
        {"ind": 1, "ty": 4, "nm": "dot", "ks": {"o": prop(100), "p": {"a": 1, "k": [
            {"t": 0, "s": [10, 10, 0]}, {"t": 5, "s": [50, 10, 0]}]}}, "shapes": [], "ip": 0, "op": 6},
    ],
}

# Draws a flat picture whose colour hashes the Lottie text and the frame, so editing one keyframe
# changes the pixels. Logs one line per run (the process id), so a test can count players.
STAND_IN = '''
import hashlib, os, sys
path, width, height, frame, out = sys.argv[1], int(sys.argv[2]), int(sys.argv[3]), sys.argv[4], sys.argv[5]
text = open(path).read()
with open("players.log", "a") as log:
    log.write(f"{os.getpid()} {frame}\\n")
if "data-skip" in text:
    print("Warning: effect Blur is not supported, skipped", file=sys.stderr)
r, g, b = hashlib.sha256((text + frame).encode()).digest()[:3]
open(out, "wb").write(bytes([r, g, b, 128]) * (width * height))
'''


def montagent():
    for candidate in (os.environ.get("MONTAGENT"), str(ROOT / "target/debug/montagent"),
                      str(ROOT / "target/release/montagent"), shutil.which("montagent")):
        if candidate and Path(candidate).exists():
            return candidate
    return None


def sha_frames(rgba, size):
    return [hashlib.sha256(rgba[i : i + size]).hexdigest() for i in range(0, len(rgba), size)]


def decode_mov(path, width, height):
    out = subprocess.run(["ffmpeg", "-v", "error", "-i", str(path), "-f", "rawvideo", "-pix_fmt", "rgba", "-"],
                         capture_output=True, check=True).stdout
    return out


class Machinery(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp())
        self.addCleanup(shutil.rmtree, self.tmp, ignore_errors=True)
        self.write(LOTTIE)
        (self.tmp / "player.py").write_text(STAND_IN)
        shutil.copy(FONT, self.tmp / "Inter-Bold.ttf")

    def write(self, lottie):
        (self.tmp / "anim.json").write_text(json.dumps(lottie))

    def spec(self, **over):
        spec = {
            "name": "anim", "out": "media", "input": "lottie", "lottie": "anim.json",
            "width": W, "height": H, "fps": 25, "frames": 6,
            "render": [sys.executable, "player.py", "anim.json", "{width}", "{height}", "{frame}", "{out}"],
            "files": ["anim.json", "player.py"],
            "versions": {"player": [sys.executable, "-c", "print('stand-in 1')"]},
        }
        spec.update(over)
        (self.tmp / "spec.json").write_text(json.dumps(spec))
        return self.tmp / "spec.json"

    def run_script(self, *args):
        return subprocess.run([sys.executable, str(SCRIPT), *map(str, args)], capture_output=True, text=True)

    def build(self, **over):
        return self.run_script("build", self.spec(**over))

    def recipe(self):
        return json.loads((self.tmp / "media/anim.recipe/recipe.json").read_text())

    def test_build_encodes_png_in_mov_that_decodes_to_the_frames_the_player_wrote(self):
        done = self.build()
        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertIn("6 frames, decoded = drawn", done.stderr)
        decoded = decode_mov(self.tmp / "media/anim.mov", W, H)
        text = (self.tmp / "anim.json").read_text()
        for i, got in enumerate(sha_frames(decoded, W * H * 4)):
            r, g, b = hashlib.sha256((text + repr(float(i))).encode()).digest()[:3]
            self.assertEqual(got, hashlib.sha256(bytes([r, g, b, 128]) * (W * H)).hexdigest(), i)

    def test_every_frame_comes_from_a_fresh_player_process(self):
        self.assertEqual(self.build().returncode, 0)
        lines = (self.tmp / "players.log").read_text().split("\n")[:-1]
        self.assertEqual(len(lines), 6)
        self.assertEqual(len({line.split()[0] for line in lines}), 6, "one process id per frame")

    def test_the_recipe_holds_size_fps_frames_player_version_hashes_and_a_copy_of_the_lottie(self):
        self.assertEqual(self.build().returncode, 0)
        recipe = self.recipe()
        self.assertEqual((recipe["width"], recipe["height"], recipe["fps"], recipe["frames"]), (W, H, 25, 6))
        self.assertEqual(recipe["input"], "lottie")
        self.assertEqual(recipe["lottie_frames"], [0.0, 1.0, 2.0, 3.0, 4.0, 5.0])
        self.assertEqual(recipe["versions"]["player"], "stand-in 1")
        self.assertEqual(len(recipe["decoded_frame_sha256"]), 6)
        self.assertEqual(recipe["render"][3:5], [str(W), str(H)])
        self.assertTrue((self.tmp / "media/anim.recipe/source/anim.json").exists())

    def test_the_frame_range_and_step_are_the_recipes_not_the_lottiess(self):
        self.assertEqual(self.build(fps=50, frames=4, start=1).returncode, 0)
        self.assertEqual(self.recipe()["lottie_frames"], [1.0, 1.5, 2.0, 2.5])
        done = self.build(frames=20)
        self.assertNotEqual(done.returncode, 0)
        self.assertIn("out point", done.stderr)

    def test_build_needs_a_literal_size_and_a_player_version(self):
        for key in ("width", "height", "versions", "lottie", "fps", "frames"):
            spec = json.loads(self.spec().read_text())
            del spec[key]
            (self.tmp / "spec.json").write_text(json.dumps(spec))
            done = self.run_script("build", self.tmp / "spec.json")
            self.assertNotEqual(done.returncode, 0, key)
            self.assertIn(key, done.stderr)

    # -- refusals -------------------------------------------------------------------

    def test_an_expression_fails_the_build_and_names_the_layer(self):
        lottie = copy.deepcopy(LOTTIE)
        lottie["layers"][0]["ks"]["o"] = {"a": 0, "k": 100, "x": "wiggle(2, 30)"}
        self.write(lottie)
        done = self.build()
        self.assertNotEqual(done.returncode, 0)
        self.assertIn("expression", done.stderr)
        self.assertIn("dot", done.stderr)
        self.assertIn("wiggle", done.stderr)
        self.assertFalse((self.tmp / "media/anim.mov").exists())

    def test_easing_handles_are_not_mistaken_for_expressions(self):
        lottie = copy.deepcopy(LOTTIE)
        lottie["layers"][0]["ks"]["o"] = {"a": 1, "k": [
            {"t": 0, "s": [0], "i": {"x": [0.5], "y": [1]}, "o": {"x": [0.5], "y": [0]}}, {"t": 5, "s": [100]}]}
        self.write(lottie)
        self.assertEqual(self.build().returncode, 0)

    def test_a_layer_effect_fails_the_build_unless_the_recipe_names_it_as_drawn(self):
        lottie = copy.deepcopy(LOTTIE)
        lottie["layers"][0]["ef"] = [{"ty": 29, "nm": "Gaussian Blur", "mn": "ADBE Gaussian Blur 2"}]
        self.write(lottie)
        done = self.build()
        self.assertNotEqual(done.returncode, 0)
        self.assertIn("Gaussian Blur", done.stderr)
        self.assertEqual(self.build(allow_effects=["ADBE Gaussian Blur 2"]).returncode, 0)

    def _text_lottie(self, text, font="Inter-Bold"):
        lottie = copy.deepcopy(LOTTIE)
        lottie["layers"].append({"ind": 2, "ty": 5, "nm": "title", "ks": {}, "ip": 0, "op": 6,
                                 "t": {"d": {"k": [{"s": {"s": 48, "f": font, "t": text}, "t": 0}]}}})
        self.write(lottie)

    def test_text_with_embedded_glyphs_needs_no_font(self):
        self._text_lottie("HH")
        self.assertEqual(self.build().returncode, 0)

    def test_text_without_glyphs_or_a_vendored_font_fails_and_a_vendored_font_passes(self):
        self._text_lottie("Hello")
        done = self.build()
        self.assertNotEqual(done.returncode, 0)
        self.assertIn("Inter", done.stderr)
        self.assertEqual(self.build(fonts=["Inter-Bold.ttf"]).returncode, 0)

    def test_text_in_a_font_the_lottie_does_not_list_fails(self):
        self._text_lottie("H", font="Garamond")
        done = self.build()
        self.assertNotEqual(done.returncode, 0)
        self.assertIn("Garamond", done.stderr)

    def test_a_skip_the_player_reports_fails_the_build(self):
        (self.tmp / "anim.json").write_text(json.dumps(LOTTIE) + " ")
        text = json.dumps(LOTTIE).replace('"nm": "dot"', '"nm": "data-skip"')
        (self.tmp / "anim.json").write_text(text)
        done = self.build()
        self.assertNotEqual(done.returncode, 0)
        self.assertIn("skipped", done.stderr)

    # -- rebuild --------------------------------------------------------------------

    def test_rebuild_reports_a_match(self):
        self.assertEqual(self.build().returncode, 0)
        done = self.run_script("rebuild", self.tmp / "media/anim.recipe")
        self.assertEqual(done.returncode, 0, done.stdout + done.stderr)
        self.assertIn("rebuild: match, 6 frames", done.stdout)
        self.assertIn("footage: match, 6 frames", done.stdout)

    def test_editing_one_keyframe_reports_the_first_differing_frame(self):
        self.assertEqual(self.build().returncode, 0)
        copy_ = self.tmp / "media/anim.recipe/source/anim.json"
        copy_.write_text(copy_.read_text().replace("[50, 10, 0]", "[51, 10, 0]"))
        done = self.run_script("rebuild", self.tmp / "media/anim.recipe")
        self.assertEqual(done.returncode, 1, done.stdout + done.stderr)
        self.assertIn("rebuild: differs, first at frame 0", done.stdout)
        self.assertIn("footage: match", done.stdout)

    def test_rebuild_refuses_a_recipe_whose_lottie_gained_an_expression(self):
        self.assertEqual(self.build().returncode, 0)
        copy_ = self.tmp / "media/anim.recipe/source/anim.json"
        lottie = json.loads(copy_.read_text())
        lottie["layers"][0]["ks"]["o"] = {"a": 0, "k": 100, "x": "time*10"}
        copy_.write_text(json.dumps(lottie))
        done = self.run_script("rebuild", self.tmp / "media/anim.recipe")
        self.assertNotEqual(done.returncode, 0)
        self.assertIn("expression", done.stderr)


@unittest.skipUnless(HAVE_THORVG, "thorvg_python is not installed")
class Sample(unittest.TestCase):
    """The skill's sample Lottie through its sample ThorVG player."""

    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp())
        self.addCleanup(shutil.rmtree, self.tmp, ignore_errors=True)
        for name in ("sample_lottie.json", "lottie_frame.py"):
            shutil.copy(SKILL / "references" / name, self.tmp)
        self.spec = {
            "name": "sample", "out": "media", "input": "lottie", "lottie": "sample_lottie.json",
            "width": 320, "height": 180, "fps": 25, "frames": 50,
            "render": [sys.executable, "lottie_frame.py", "sample_lottie.json", "{width}", "{height}", "{frame}", "{out}"],
            "files": ["sample_lottie.json", "lottie_frame.py"],
            "versions": {"thorvg-python": [sys.executable, "-c", "import importlib.metadata as m; print(m.version('thorvg-python'))"]},
        }
        (self.tmp / "spec.json").write_text(json.dumps(self.spec))

    def run_script(self, *args):
        return subprocess.run([sys.executable, str(SCRIPT), *map(str, args)], capture_output=True, text=True)

    def player(self, frame, out):
        subprocess.run([sys.executable, str(self.tmp / "lottie_frame.py"), str(self.tmp / "sample_lottie.json"),
                        "320", "180", repr(float(frame)), str(out)], check=True, cwd=self.tmp)
        return Path(out).read_bytes()

    def test_the_sample_builds_and_decoded_frames_equal_the_players_frames_alpha_included(self):
        done = self.run_script("build", self.tmp / "spec.json")
        self.assertEqual(done.returncode, 0, done.stderr)
        decoded = decode_mov(self.tmp / "media/sample.mov", 320, 180)
        size = 320 * 180 * 4
        self.assertEqual(len(decoded), 50 * size)
        for i in (0, 7, 25, 49):
            self.assertEqual(decoded[i * size : (i + 1) * size], self.player(i, self.tmp / "one.rgba"), i)
        alphas = {decoded[size * 25 + k] for k in range(3, size, 4)}
        self.assertIn(0, alphas, "transparent areas")
        self.assertIn(255, alphas)
        self.assertEqual(decoded[3], 0, "frame 0 draws nothing at the corner")
        recipe = json.loads((self.tmp / "media/sample.recipe/recipe.json").read_text())
        self.assertRegex(recipe["versions"]["thorvg-python"], r"^\d+\.\d+\.\d+")
        self.assertTrue((self.tmp / "media/sample.recipe/source/sample_lottie.json").exists())

    def test_the_same_frame_twice_is_the_same_bytes(self):
        for frame in (0, 3, 17.5, 49):
            self.assertEqual(self.player(frame, self.tmp / "a.rgba"), self.player(frame, self.tmp / "b.rgba"), frame)

    def test_rebuild_matches_then_one_keyframe_reports_the_first_differing_frame(self):
        self.assertEqual(self.run_script("build", self.tmp / "spec.json").returncode, 0)
        recipe = self.tmp / "media/sample.recipe"
        done = self.run_script("rebuild", recipe)
        self.assertEqual(done.returncode, 0, done.stdout + done.stderr)
        copy_ = recipe / "source/sample_lottie.json"
        text = copy_.read_text()
        self.assertIn("[270,50,0]", text)
        copy_.write_text(text.replace("[270,50,0]", "[271,50,0]"))
        done = self.run_script("rebuild", recipe)
        self.assertEqual(done.returncode, 1)
        self.assertRegex(done.stdout, r"rebuild: differs, first at frame (\d+)")

    @unittest.skipUnless(montagent(), "no montagent binary")
    def test_place_validates_probes_alpha_and_renders(self):
        self.assertEqual(self.run_script("build", self.tmp / "spec.json").returncode, 0)
        project = json.loads((FIXTURES / "type_on.montagent.json").read_text())
        project["frame"] = {"width": 320, "height": 180}
        project["fps"], project["duration"] = 25, 2000
        project["fonts"], project["fontVendor"] = {}, {}
        (self.tmp / "project.montagent.json").write_text(json.dumps(project))
        (self.tmp / "placing.json").write_text(json.dumps({"source": "media/sample.mov", "id": "sample", "x": 160, "y": 90}))
        env = dict(os.environ, PATH=str(Path(montagent()).parent) + os.pathsep + os.environ["PATH"])
        done = subprocess.run([sys.executable, str(SCRIPT), "place", self.tmp / "project.montagent.json",
                               self.tmp / "placing.json"], capture_output=True, text=True, env=env)
        self.assertEqual(done.returncode, 0, done.stderr)
        element = json.loads(done.stdout)["tracks"][-1]["elements"][0]
        self.assertEqual(element["type"], "video")
        (self.tmp / "placed.montagent.json").write_text(done.stdout)
        checked = subprocess.run([montagent(), "validate", self.tmp / "placed.montagent.json", "--json"],
                                 capture_output=True, text=True)
        self.assertEqual(json.loads(checked.stdout)["summary"]["error"], 0, checked.stdout)
        probed = subprocess.run([montagent(), "probe", "--json", self.tmp / "media/sample.mov"],
                                capture_output=True, text=True)
        self.assertTrue(json.loads(probed.stdout)["media"][0]["alpha"]["carries"])
        rendered = subprocess.run([montagent(), "render", self.tmp / "placed.montagent.json"],
                                  capture_output=True, text=True, cwd=self.tmp)
        self.assertEqual(rendered.returncode, 0, rendered.stdout + rendered.stderr)


if __name__ == "__main__":
    unittest.main()
