#!/usr/bin/env python3
"""The SVG-still route of `skills/montagent-prerender/scripts/prerender.py` (#810, ADR-0171 §3-§4).

    python3 -m unittest ci/test_prerender_still.py

What it holds, and what runs where:

- The machinery tests (recipe, rebuild, size, fonts, place) use a stand-in rasteriser written
  into the scratch folder, so they need only Python and ffmpeg. `place` also needs a
  `montagent` binary: `MONTAGENT`, `target/debug/montagent`, or `PATH`; without one those
  tests skip.
- The `resvg` tests rasterise the skill's own sample SVG with the real `resvg` command line
  and skip when it is not on `PATH`. They are the acceptance for the sample: shapes, a
  gradient and vendored-font text, and a decoded PNG that equals the rasteriser's output.
"""

import hashlib
import json
import os
import shutil
import struct
import subprocess
import sys
import tempfile
import unittest
import zlib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SKILL = ROOT / "skills/montagent-prerender"
SCRIPT = SKILL / "scripts/prerender.py"
FIXTURES = ROOT / "fixtures/skills"
FONT = FIXTURES / "fonts/Inter-Bold.ttf"

SVG = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 9">
  <rect width="16" height="9" fill="#204060"/>
  <text x="2" y="6" font-family="Inter" font-weight="700" font-size="3">Hi</text>
</svg>
"""

# Draws a flat picture whose colour is a hash of the SVG's text, so editing one number in the
# SVG changes the pixels. Writes one RGBA PNG at the size it is told. Prints the same warning
# `usvg` prints when `data-warn` is in the SVG.
STAND_IN = '''
import hashlib, struct, sys, zlib
svg, out, width, height = sys.argv[1], sys.argv[2], int(sys.argv[3]), int(sys.argv[4])
text = open(svg).read()
if "data-warn" in text:
    print("Warning (in usvg::text:143): No match for '\\"Nope\\"' font-family.", file=sys.stderr)
r, g, b = hashlib.sha256(text.encode()).digest()[:3]
if "wrong-size" in text:
    width += 1
row = b"\\x00" + bytes([r, g, b, 200]) * width
def chunk(kind, data):
    body = kind + data
    return struct.pack(">I", len(data)) + body + struct.pack(">I", zlib.crc32(body))
png = b"\\x89PNG\\r\\n\\x1a\\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0))
png += chunk(b"IDAT", zlib.compress(row * height)) + chunk(b"IEND", b"")
open(out, "wb").write(png)
'''


def montagent():
    for candidate in (os.environ.get("MONTAGENT"), str(ROOT / "target/debug/montagent"),
                      str(ROOT / "target/release/montagent"), shutil.which("montagent")):
        if candidate and Path(candidate).exists():
            return candidate
    return None


def decode_png(path):
    """(width, height, RGBA bytes) of an 8-bit RGBA, non-interlaced PNG, with no help from ffmpeg."""
    data = Path(path).read_bytes()
    assert data[:8] == b"\x89PNG\r\n\x1a\n"
    at, idat, header = 8, b"", None
    while at < len(data):
        (length,) = struct.unpack(">I", data[at : at + 4])
        kind, body = data[at + 4 : at + 8], data[at + 8 : at + 8 + length]
        if kind == b"IHDR":
            header = struct.unpack(">IIBBBBB", body)
        elif kind == b"IDAT":
            idat += body
        at += 12 + length
    width, height, depth, colour, _, _, interlace = header
    assert (depth, colour, interlace) == (8, 6, 0), "the test decoder reads 8-bit RGBA only"
    raw, stride, rows, prev = zlib.decompress(idat), width * 4, [], bytearray(width * 4)
    for y in range(height):
        line = raw[y * (stride + 1) : (y + 1) * (stride + 1)]
        mode, cur = line[0], bytearray(line[1:])
        for i in range(stride):
            a = cur[i - 4] if i >= 4 else 0
            b = prev[i]
            c = prev[i - 4] if i >= 4 else 0
            if mode == 1:
                cur[i] = (cur[i] + a) & 255
            elif mode == 2:
                cur[i] = (cur[i] + b) & 255
            elif mode == 3:
                cur[i] = (cur[i] + (a + b) // 2) & 255
            elif mode == 4:
                p = a + b - c
                pa, pb, pc = abs(p - a), abs(p - b), abs(p - c)
                cur[i] = (cur[i] + (a if pa <= pb and pa <= pc else b if pb <= pc else c)) & 255
        rows.append(bytes(cur))
        prev = cur
    return width, height, b"".join(rows)


class Still(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp())
        self.addCleanup(shutil.rmtree, self.tmp, ignore_errors=True)
        (self.tmp / "art.svg").write_text(SVG)
        (self.tmp / "raster.py").write_text(STAND_IN)
        shutil.copy(FONT, self.tmp / "Inter-Bold.ttf")

    def spec(self, **over):
        spec = {
            "name": "mark", "out": "media", "input": "svg", "svg": "art.svg", "width": 48, "height": 27,
            "render": [sys.executable, "raster.py", "art.svg", "{out}", "{width}", "{height}"],
            "files": ["art.svg", "raster.py"], "fonts": ["Inter-Bold.ttf"],
            "versions": {"raster": [sys.executable, "-c", "print('stand-in 1')"]},
        }
        spec.update(over)
        (self.tmp / "spec.json").write_text(json.dumps(spec))
        return self.tmp / "spec.json"

    def run_script(self, *args):
        return subprocess.run([sys.executable, str(SCRIPT), *map(str, args)],
                              capture_output=True, text=True)

    def build(self, **over):
        return self.run_script("build", self.spec(**over))

    def recipe(self):
        return json.loads((self.tmp / "media/mark.recipe/recipe.json").read_text())

    # -- build ----------------------------------------------------------------------

    def test_build_places_the_rasterisers_png_and_it_decodes_to_the_same_pixels(self):
        done = self.build()
        self.assertEqual(done.returncode, 0, done.stderr)
        width, height, pixels = decode_png(self.tmp / "media/mark.png")
        self.assertEqual((width, height), (48, 27))
        self.assertEqual(pixels[:4], bytes(hashlib.sha256(SVG.encode()).digest()[:3]) + bytes([200]))
        self.assertIn("decoded = drawn", done.stderr)
        self.assertFalse((self.tmp / "media/mark.mov").exists())

    def test_the_recipe_is_one_frame_with_the_size_the_tool_and_a_decoded_pixel_hash(self):
        self.assertEqual(self.build().returncode, 0)
        recipe = self.recipe()
        _, _, pixels = decode_png(self.tmp / "media/mark.png")
        self.assertEqual(recipe["decoded_frame_sha256"], [hashlib.sha256(pixels).hexdigest()])
        self.assertEqual((recipe["width"], recipe["height"], recipe["frames"]), (48, 27, 1))
        self.assertNotIn("fps", recipe)
        self.assertEqual(recipe["input"], "svg")
        self.assertEqual(recipe["versions"]["raster"], "stand-in 1")
        # the command carries the literal size, with no placeholder left for it
        self.assertEqual(recipe["render"][-2:], ["48", "27"])
        self.assertTrue((self.tmp / "media/mark.recipe/source/art.svg").exists())
        self.assertTrue((self.tmp / "media/mark.recipe/source/Inter-Bold.ttf").exists())

    def test_build_needs_a_literal_size_and_a_tool_version(self):
        for key in ("width", "height", "versions"):
            spec = json.loads(self.spec().read_text())
            del spec[key]
            (self.tmp / "spec.json").write_text(json.dumps(spec))
            done = self.run_script("build", self.tmp / "spec.json")
            self.assertNotEqual(done.returncode, 0, key)
            self.assertIn(key, done.stderr)

    def test_a_png_of_another_size_than_the_recipe_names_fails_the_build(self):
        (self.tmp / "art.svg").write_text(SVG + "<!-- wrong-size -->")
        done = self.build()
        self.assertNotEqual(done.returncode, 0)
        self.assertIn("48x27", done.stderr)

    # -- text -----------------------------------------------------------------------

    def test_an_svg_naming_a_font_that_is_not_vendored_fails_the_build(self):
        (self.tmp / "art.svg").write_text(SVG.replace("Inter", "Garamond"))
        done = self.build()
        self.assertNotEqual(done.returncode, 0)
        self.assertIn("Garamond", done.stderr)
        self.assertFalse((self.tmp / "media/mark.png").exists())

    def test_text_with_no_font_family_fails_the_build(self):
        (self.tmp / "art.svg").write_text(SVG.replace(' font-family="Inter"', ""))
        done = self.build()
        self.assertNotEqual(done.returncode, 0)
        self.assertIn("font-family", done.stderr)

    def test_a_generic_family_is_a_system_font_and_fails(self):
        (self.tmp / "art.svg").write_text(SVG.replace('"Inter"', '"sans-serif"'))
        self.assertNotEqual(self.build().returncode, 0)

    def test_a_family_inherited_from_a_group_and_a_css_property_both_count(self):
        grouped = SVG.replace(' font-family="Inter"', "").replace("<text", '<g font-family="Inter"><text').replace(
            "</text>", "</text></g>")
        (self.tmp / "art.svg").write_text(grouped)
        self.assertEqual(self.build().returncode, 0)
        (self.tmp / "art.svg").write_text(SVG.replace('font-family="Inter"', 'style="font-family: \'Inter\', serif"'))
        self.assertEqual(self.build().returncode, 0)

    def test_text_the_rasteriser_warns_it_could_not_set_fails_the_build(self):
        (self.tmp / "art.svg").write_text(SVG + "<!-- data-warn -->")
        done = self.build()
        self.assertNotEqual(done.returncode, 0)
        self.assertIn("font", done.stderr)

    def test_an_svg_with_no_text_needs_no_font(self):
        (self.tmp / "art.svg").write_text('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1 1"><rect width="1" height="1"/></svg>')
        done = self.build(fonts=[])
        self.assertEqual(done.returncode, 0, done.stderr)

    # -- rebuild --------------------------------------------------------------------

    def test_rebuild_reports_a_match(self):
        self.assertEqual(self.build().returncode, 0)
        done = self.run_script("rebuild", self.tmp / "media/mark.recipe")
        self.assertEqual(done.returncode, 0, done.stdout + done.stderr)
        self.assertIn("rebuild: match, 1 frames", done.stdout)
        self.assertIn("footage: match, 1 frames", done.stdout)

    def test_editing_one_number_in_the_recipes_svg_reports_the_difference(self):
        self.assertEqual(self.build().returncode, 0)
        copy = self.tmp / "media/mark.recipe/source/art.svg"
        copy.write_text(copy.read_text().replace('x="2"', 'x="3"'))
        done = self.run_script("rebuild", self.tmp / "media/mark.recipe")
        self.assertEqual(done.returncode, 1, done.stdout + done.stderr)
        self.assertIn("rebuild: differs, first at frame 0", done.stdout)
        self.assertIn("first pixel that differs: (0, 0)", done.stdout)
        self.assertIn("footage: match", done.stdout)

    def test_rebuild_reports_a_placed_png_that_no_longer_matches(self):
        self.assertEqual(self.build().returncode, 0)
        png = self.tmp / "media/mark.png"
        (self.tmp / "art.svg").write_text(SVG + "<!-- other -->")
        self.assertEqual(self.build(name="other").returncode, 0)
        shutil.copy(self.tmp / "media/other.png", png)
        done = self.run_script("rebuild", self.tmp / "media/mark.recipe")
        self.assertEqual(done.returncode, 1)
        self.assertIn("footage: differs", done.stdout)

    def test_rebuild_compares_pixels_and_never_file_bytes(self):
        self.assertEqual(self.build().returncode, 0)
        # Same pixels, other bytes: a different zlib level and an extra ancillary chunk.
        width, height, pixels = decode_png(self.tmp / "media/mark.png")
        chunk = lambda kind, data: (struct.pack(">I", len(data)) + kind + data
                                    + struct.pack(">I", zlib.crc32(kind + data)))
        rows = b"".join(b"\x00" + pixels[y * width * 4 : (y + 1) * width * 4] for y in range(height))
        png = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0))
        png += chunk(b"tEXt", b"Comment\x00other bytes") + chunk(b"IDAT", zlib.compress(rows, 1)) + chunk(b"IEND", b"")
        self.assertNotEqual(png, (self.tmp / "media/mark.png").read_bytes())
        (self.tmp / "media/mark.png").write_bytes(png)
        done = self.run_script("rebuild", self.tmp / "media/mark.recipe")
        self.assertEqual(done.returncode, 0, done.stdout + done.stderr)

    def test_rebuild_fails_when_the_font_has_gone_from_the_recipe(self):
        self.assertEqual(self.build().returncode, 0)
        (self.tmp / "media/mark.recipe/source/Inter-Bold.ttf").unlink()
        done = self.run_script("rebuild", self.tmp / "media/mark.recipe")
        self.assertNotEqual(done.returncode, 0)

    # -- place ----------------------------------------------------------------------

    @unittest.skipUnless(montagent(), "no montagent binary")
    def test_place_adds_a_plain_image_element_that_validates(self):
        self.assertEqual(self.build(width=48, height=27).returncode, 0)
        project = json.loads((FIXTURES / "type_on.montagent.json").read_text())
        project["frame"] = {"width": 160, "height": 90}
        project["duration"] = 2000
        project["fonts"], project["fontVendor"] = {}, {}
        (self.tmp / "project.montagent.json").write_text(json.dumps(project))
        (self.tmp / "placing.json").write_text(json.dumps(
            {"source": "media/mark.png", "id": "mark", "x": 80, "y": 45, "duration": 2000}))
        env = dict(os.environ, PATH=str(Path(montagent()).parent) + os.pathsep + os.environ["PATH"])
        done = subprocess.run([sys.executable, str(SCRIPT), "place", self.tmp / "project.montagent.json",
                               self.tmp / "placing.json"], capture_output=True, text=True, env=env)
        self.assertEqual(done.returncode, 0, done.stderr)
        placed = json.loads(done.stdout)
        element = placed["tracks"][-1]["elements"][0]
        self.assertEqual(element["type"], "image")
        self.assertEqual((element["width"], element["height"], element["fit"]), (48, 27, "contain"))
        self.assertEqual((element["start"], element["end"]), (0, 2000))
        self.assertNotIn("source_end", element)
        (self.tmp / "placed.montagent.json").write_text(done.stdout)
        checked = subprocess.run([montagent(), "validate", self.tmp / "placed.montagent.json", "--json"],
                                 capture_output=True, text=True)
        self.assertEqual(json.loads(checked.stdout)["summary"]["error"], 0, checked.stdout)


@unittest.skipUnless(shutil.which("resvg"), "no resvg on PATH")
class Sample(unittest.TestCase):
    """The skill's sample SVG, through the real `resvg` command line."""

    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp())
        self.addCleanup(shutil.rmtree, self.tmp, ignore_errors=True)
        shutil.copy(SKILL / "references/sample_art.svg", self.tmp)
        shutil.copy(FONT, self.tmp / "Inter-Bold.ttf")
        version = subprocess.run(["resvg", "--version"], capture_output=True, text=True).stdout.strip()
        self.spec = {
            "name": "mark", "out": "media", "input": "svg", "svg": "sample_art.svg", "width": 480, "height": 270,
            "render": ["resvg", "--skip-system-fonts", "--use-font-file", "Inter-Bold.ttf",
                       "-w", "{width}", "-h", "{height}", "sample_art.svg", "{out}"],
            "files": ["sample_art.svg"], "fonts": ["Inter-Bold.ttf"],
            "versions": {"resvg": ["resvg", "--version"]},
        }
        (self.tmp / "spec.json").write_text(json.dumps(self.spec))
        self.version = version

    def run_script(self, *args):
        return subprocess.run([sys.executable, str(SCRIPT), *map(str, args)], capture_output=True, text=True)

    def test_the_sample_rasterises_and_the_placed_png_equals_resvgs_output_pixel_for_pixel(self):
        done = self.run_script("build", self.tmp / "spec.json")
        self.assertEqual(done.returncode, 0, done.stderr)
        # resvg's own output, run by hand, with nothing of the script between
        direct = self.tmp / "direct.png"
        subprocess.run(["resvg", "--skip-system-fonts", "--use-font-file", str(self.tmp / "Inter-Bold.ttf"),
                        "-w", "480", "-h", "270", str(self.tmp / "sample_art.svg"), str(direct)], check=True)
        want, got = decode_png(direct), decode_png(self.tmp / "media/mark.png")
        self.assertEqual(got[:2], (480, 270))
        self.assertEqual(want, got)
        pixels = got[2]
        alphas = {pixels[i] for i in range(3, len(pixels), 4)}
        self.assertIn(0, alphas, "the sample has transparent corners")
        self.assertIn(255, alphas)
        self.assertGreater(len({pixels[i : i + 4] for i in range(0, len(pixels), 4)}), 200,
                           "a gradient, not flat fills")
        recipe = json.loads((self.tmp / "media/mark.recipe/recipe.json").read_text())
        self.assertEqual(recipe["versions"]["resvg"], self.version)

    def test_rebuild_matches_then_one_number_in_the_svg_reports_the_difference(self):
        self.assertEqual(self.run_script("build", self.tmp / "spec.json").returncode, 0)
        recipe = self.tmp / "media/mark.recipe"
        self.assertEqual(self.run_script("rebuild", recipe).returncode, 0)
        copy = recipe / "source/sample_art.svg"
        text = copy.read_text()
        self.assertIn('cx="80"', text)
        copy.write_text(text.replace('cx="80"', 'cx="81"', 1))
        done = self.run_script("rebuild", recipe)
        self.assertEqual(done.returncode, 1)
        self.assertIn("rebuild: differs", done.stdout)

    def test_the_sample_pointed_at_a_font_that_is_not_vendored_fails_the_build(self):
        svg = self.tmp / "sample_art.svg"
        svg.write_text(svg.read_text().replace("Inter", "Garamond"))
        done = self.run_script("build", self.tmp / "spec.json")
        self.assertNotEqual(done.returncode, 0)
        self.assertIn("Garamond", done.stderr)


if __name__ == "__main__":
    unittest.main()
