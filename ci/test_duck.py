#!/usr/bin/env python3
"""The ducking script's maths and behaviour (ADR-0177 §6, #838 D2 and D4).

    python3 -m unittest ci/test_duck.py

`skills/montagent-footage/scripts/duck.py` runs on the skill fixtures; the bed's `volume` is read
back through `montagent query --at` at the instants the words file names, and compared with the
linear value of the dB argument rounded to four decimals (tolerance 0.0001). The instants are
computed here from the words and the frame grid, not copied from the script's output.

Needs `montagent` (`MONTAGENT`, `target/debug/montagent`, `target/release/montagent` or `PATH`) and
`ffmpeg` (the closing `validate` probes the media). When either is missing the tests skip locally
and fail when `CI` is set.
"""

import ast
import copy
import json
import math
import os
import re
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SCRIPT = ROOT / "skills/montagent-footage/scripts/duck.py"
FIXTURES = ROOT / "fixtures/skills"
LEVELS = ROOT / "docs/research/audio-effects/ducking/check_duck_levels.py"
FPS = 30
UNDER, OVER, END = -15, -6, -1.5


def montagent():
    for candidate in (os.environ.get("MONTAGENT"), str(ROOT / "target/debug/montagent"),
                      str(ROOT / "target/release/montagent"), shutil.which("montagent")):
        for path in (candidate, candidate and candidate + ".exe"):
            if path and Path(path).is_file():
                return str(Path(path).resolve())
    return None


def lin(db):
    return round(10 ** (db / 20), 4)


def on_frame(t):
    return math.floor(round(t * FPS / 1000) * 1000 / FPS)


def setUpModule():
    missing = [n for n, found in (("montagent", montagent()), ("ffmpeg", shutil.which("ffmpeg"))) if not found]
    if missing:
        message = "needs " + " and ".join(missing)
        if os.environ.get("CI"):
            raise AssertionError(message + ": CI must run the duck script's tests")
        raise unittest.SkipTest(message)


class Duck(unittest.TestCase):
    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self._tmp.cleanup)
        self.dir = Path(self._tmp.name) / "fx"
        shutil.copytree(FIXTURES, self.dir, ignore=shutil.ignore_patterns("rig"))
        self.words = json.loads((self.dir / "media/duck.words.json").read_text())
        self.project = self.dir / "captions.montagent.json"
        self.env = dict(os.environ, PATH=str(Path(montagent()).parent) + os.pathsep + os.environ["PATH"])

    # -- helpers --------------------------------------------------------------------

    def run_duck(self, *args, project=None, env=None, words=True):
        extra = ["--words", "media/duck.words.json"] if words else []
        return subprocess.run(
            [sys.executable, str(SCRIPT), str(project or self.project), "--bed", "bed", "--voice", "voice",
             *extra, *map(str, args)],
            capture_output=True, text=True, cwd=self.dir, env=env or self.env)

    def duck_to(self, name, *args, **kw):
        """Run the script and keep what it prints as <name>."""
        done = self.run_duck(*args, **kw)
        self.assertEqual(done.returncode, 0, done.stderr)
        out = self.dir / name
        out.write_text(done.stdout)
        return out

    def volume_at(self, project, t):
        done = subprocess.run([montagent(), "query", str(project), "--at", str(t), "--json"],
                              capture_output=True, text=True, env=self.env, cwd=self.dir)
        self.assertEqual(done.returncode, 0, done.stdout + done.stderr)
        stack = json.loads(done.stdout)["query"]["stack"]
        bed = next(e for e in stack if e["id"] == "bed")
        return next(v["value"] for v in bed["values"] if v["property"] == "volume")

    def keys(self, project):
        doc = json.loads(Path(project).read_text())
        bed = next(e for t in doc["tracks"] for e in t["elements"] if e["id"] == "bed")
        return bed["volume"]

    def edit(self, project, change):
        doc = json.loads(Path(project).read_text())
        change(doc, next(e for t in doc["tracks"] for e in t["elements"] if e["id"] == "bed"))
        Path(project).write_text(json.dumps(doc))

    # -- the maths ------------------------------------------------------------------

    def spans(self, join=600):
        merged = []
        for w in self.words:
            if merged and w["start"] - merged[-1][1] < join:
                merged[-1][1] = w["end"]
            else:
                merged.append([w["start"], w["end"]])
        return merged

    def test_the_bed_reads_back_at_the_named_instants(self):
        out = self.duck_to("ducked.montagent.json")
        spans = self.spans()
        (s1, e1), (s2, e2) = spans
        lead, ramp = 100, 200
        gap_short = (self.words[0]["end"] + self.words[1]["start"]) // 2     # inside the first phrase
        gap_long = (e1 + s2) // 2                                            # between the phrases
        self.assertGreaterEqual(s2 - e1, 600)
        self.assertLess(self.words[1]["start"] - self.words[0]["end"], 600)
        cases = {
            "inside speech": (self.words[0]["start"] + 200, UNDER),
            "inside a pause of at least join_ms": (gap_long, OVER),
            "inside a pause shorter than join_ms": (gap_short, UNDER),
            "after the last word": (e2 + lead + ramp + 500, END),
            "the ramp's start, before the voice": (on_frame(s1 - lead - ramp), OVER),
            "the ramp's end, lead_ms before the voice": (on_frame(s1 - lead), UNDER),
            "the return ramp's start": (on_frame(e2 + lead), UNDER),
            "the return ramp's end": (on_frame(e2 + lead + ramp), END),
            "the down ramp into the second phrase ends": (on_frame(s2 - lead), UNDER),
        }
        for label, (t, db) in cases.items():
            with self.subTest(label, t=t):
                self.assertAlmostEqual(self.volume_at(out, t), lin(db), delta=0.0001)

    def test_the_timeline_mapping_follows_the_voices_trim_and_speed(self):
        self.edit(self.project, lambda doc, bed: [
            v.update(start=1000, end=2750, source_start=500, source_end=4000, speed=2)
            for t in doc["tracks"] for v in t["elements"] if v["id"] == "voice"])
        out = self.duck_to("moved.montagent.json")
        # the voice speaks "Made" at source 500 -> timeline 1000, at twice the speed
        self.assertAlmostEqual(self.volume_at(out, 1100), lin(UNDER), delta=0.0001)
        self.assertAlmostEqual(self.volume_at(out, 500), lin(OVER), delta=0.0001)

    def test_keys_are_written_linear_to_four_decimals_with_holds_and_ramps_eased(self):
        keys = self.keys(self.duck_to("k.montagent.json"))
        self.assertNotIn("ease", keys[0])
        for a, b in zip(keys, keys[1:]):
            self.assertEqual(b["ease"], "linear" if a["v"] == b["v"] else "ease-in-out")
        for k in keys:
            self.assertEqual(k["v"], round(k["v"], 4))
        self.assertEqual(sorted({k["v"] for k in keys}), sorted({lin(UNDER), lin(OVER), lin(END)}))

    def test_it_prints_the_db_to_linear_mapping(self):
        done = self.run_duck()
        self.assertIn(f"-15 dB = {lin(UNDER)}", done.stderr)
        self.assertIn(f"-1.5 dB = {lin(END)}", done.stderr)

    def test_fade_ms_adds_a_fade_to_zero_on_the_last_drawn_frame(self):
        keys = self.keys(self.duck_to("f.montagent.json", "--fade-ms", 1000))
        self.assertEqual(keys[-1], {"t": 5966, "v": 0.0, "ease": "ease-in-out"})
        self.assertNotIn(0.0, [k["v"] for k in self.keys(self.duck_to("nf.montagent.json"))])

    def test_the_defaults_equal_the_ones_the_level_check_holds(self):
        text = LEVELS.read_text()
        held = ast.literal_eval(re.search(r"^DEFAULTS_DB = (\{.*\})", text, re.M).group(1))
        spec = importlib_load()
        self.assertEqual(spec.DEFAULTS_DB, held)
        self.assertEqual(spec.DEFAULTS_MS, {"ramp_ms": 200, "lead_ms": 100, "join_ms": 600})

    # -- the inputs -----------------------------------------------------------------

    def test_neither_input_stops_naming_both(self):
        done = self.run_duck(words=False)
        self.assertNotEqual(done.returncode, 0)
        self.assertIn("--words", done.stderr)
        self.assertIn("--spans", done.stderr)
        self.assertIn("start", done.stderr)

    def test_two_inputs_stop(self):
        done = self.run_duck("--spans", "0-100")
        self.assertNotEqual(done.returncode, 0)
        self.assertIn("exactly one", done.stderr)

    def test_spans_give_the_same_curve_as_the_words_they_come_from(self):
        spans = ",".join(f"{w['start']}-{w['end']}" for w in self.words)
        a = self.run_duck()
        b = self.run_duck("--spans", spans, words=False)
        self.assertEqual(b.returncode, 0, b.stderr)
        self.assertEqual(a.stdout, b.stdout)
        path = self.dir / "spans.json"
        path.write_text(json.dumps([{"start": w["start"], "end": w["end"]} for w in self.words]))
        c = self.run_duck("--spans-file", path, words=False)
        self.assertEqual(a.stdout, c.stdout)

    def test_an_unknown_id_stops(self):
        done = subprocess.run([sys.executable, str(SCRIPT), str(self.project), "--bed", "nope", "--voice", "voice",
                               "--words", "media/duck.words.json"], capture_output=True, text=True,
                              cwd=self.dir, env=self.env)
        self.assertNotEqual(done.returncode, 0)
        self.assertIn("'nope'", done.stderr)

    # -- --replace, --check, validate -----------------------------------------------

    def test_a_static_volume_is_overwritten_without_replace(self):
        self.assertEqual(json.loads(self.project.read_text())["tracks"][1]["elements"][0]["volume"], 0.5)
        self.assertEqual(self.run_duck().returncode, 0)

    def test_it_refuses_over_keyframes_and_names_replace_and_replace_overwrites(self):
        ducked = self.duck_to("ducked.montagent.json")
        again = self.run_duck(project=ducked)
        self.assertNotEqual(again.returncode, 0)
        self.assertIn("--replace", again.stderr)
        self.assertEqual(again.stdout, "")
        done = self.run_duck("--replace", "--under-db", -20, project=ducked)
        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertEqual(self.keys_of(done.stdout)[2]["v"], lin(-20))

    def keys_of(self, text):
        doc = json.loads(text)
        return next(e for t in doc["tracks"] for e in t["elements"] if e["id"] == "bed")["volume"]

    def test_check_is_clean_on_its_own_output_and_writes_nothing(self):
        ducked = self.duck_to("ducked.montagent.json")
        before = ducked.read_bytes()
        done = self.run_duck("--check", project=ducked)
        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertEqual(done.stdout, "")
        self.assertEqual(ducked.read_bytes(), before)

    def test_check_names_the_first_instant_a_hand_edit_changes(self):
        ducked = self.duck_to("ducked.montagent.json")
        self.edit(ducked, lambda doc, bed: bed["volume"][3].update(v=0.2))
        was = self.keys(ducked)
        done = self.run_duck("--check", project=ducked)
        self.assertNotEqual(done.returncode, 0)
        self.assertIn(f"t={was[3]['t']}", done.stderr)
        self.assertIn("0.2", done.stderr)
        self.assertIn(str(lin(UNDER)), done.stderr)

    def test_check_compares_the_strings_as_written(self):
        ducked = self.duck_to("ducked.montagent.json")
        self.edit(ducked, lambda doc, bed: bed["volume"][3].update(v=lin(UNDER) + 0.0001))
        self.assertNotEqual(self.run_duck("--check", project=ducked).returncode, 0)

    def test_check_fails_when_the_arguments_changed(self):
        ducked = self.duck_to("ducked.montagent.json")
        self.assertNotEqual(self.run_duck("--check", "--under-db", -20, project=ducked).returncode, 0)

    def test_check_on_a_static_volume_is_not_equal(self):
        self.assertNotEqual(self.run_duck("--check").returncode, 0)

    def test_validate_prints_its_findings_and_a_review_does_not_fail_the_run(self):
        done = self.run_duck()
        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertIn("R-EASE-INERT", done.stderr)

    def test_an_error_from_validate_fails_the_run(self):
        self.edit(self.project, lambda doc, bed: bed.update(source="media/missing.wav"))
        done = self.run_duck()
        self.assertNotEqual(done.returncode, 0)

    def test_montagent_missing_from_path_is_a_failure_not_a_skip(self):
        done = self.run_duck(env=dict(os.environ, PATH=str(Path(sys.executable).parent)))
        self.assertNotEqual(done.returncode, 0)
        self.assertIn("montagent", done.stderr)

    # -- transition windows ---------------------------------------------------------

    def test_a_transition_across_a_ramp_is_named_and_no_keyframe_moves(self):
        plain = self.keys_of(self.run_duck().stdout)

        def add_dissolve(doc, bed):
            still = {"type": "image", "source": "media/still.png", "x": 540, "y": 960, "origin": "center",
                     "width": 400, "height": 400, "fit": "contain"}
            doc["tracks"] += [
                {"name": "a", "layer": 10, "elements": [dict(still, id="a", start=0, end=2500)]},
                {"name": "b", "layer": 11, "elements": [dict(still, id="b", start=2000, end=5000)]},
                {"name": "joins", "layer": 12, "elements": [
                    {"id": "dissolve", "type": "transition", "start": 2000, "end": 2500,
                     "kind": "crossfade", "from": "a", "to": "b"}]}]
        self.edit(self.project, add_dissolve)
        done = self.run_duck()
        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertRegex(done.stderr, r"duck: keyframes 0–4200 overlap transition dissolve \(2000–2500\)")
        self.assertEqual(self.keys_of(done.stdout), plain)

    def test_no_note_without_a_transition(self):
        self.assertNotIn("overlap transition", self.run_duck().stderr)


def importlib_load():
    import importlib.util
    spec = importlib.util.spec_from_file_location("duck", SCRIPT)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


if __name__ == "__main__":
    unittest.main()
