#!/usr/bin/env python3
"""Re-derive the facts ADR-0105 rests on about the sheet's `no-grid-frame` skipped run.

Run from the repo root:  python3 docs/research/contact-sheet-finding-codes/check_unpainted_runs.py
Needs a built `montagent` (`cargo build -p montagent`). Exits non-zero, naming every defect,
when it stops reproducing.

Why this exists
---------------
ADR-0094 section 4 justifies `skipped` / `no-grid-frame` with an example: *"The fixture's
4 ms run at 56112-56116 contains no painted frame at 25 fps."* That interval is real in
`query`'s cut list -- but the boundary at 56112 is `vo-quiz`, an **audio** element, ending.
ADR-0094 section 1 mandates dropping audio from the presence set and re-merging equal
neighbours, and after that re-merge the 4 ms interval is absorbed into a visual run that
paints dozens of frames. The example was computed before the re-merge the same ADR requires.

So on the repo's only real project the condition has **zero instances**, the shape #407
found for ADR-0094 section 3's keyframe population. The condition is still reachable: two
visual boundaries on different elements less than one frame period apart. The constructed
document below is one, and `validate` said nothing about it -- no `N-QUANTIZATION`, at any
class -- which is the gap ADR-0105 filed rather than closed, and #437 has since closed.

The run rule (ADR-0094 section 1) and the painted-frame rule (section 2, ADR-0077's
`floor(n * 1000 / fps)`) are reimplemented here in a dozen lines, because the sheet itself
is not built yet; the cut list they consume is `query`'s own, not a reimplementation.
"""
import json
import pathlib
import subprocess
import sys
import tempfile

ROOT = pathlib.Path(__file__).resolve().parents[3]
FIXTURE = ROOT / "fixtures/en-halloween-decorating/en-halloween-decorating.montagent.json"

defects = []


def claim(ok, text):
    print(("PASS  " if ok else "FAIL  ") + text)
    if not ok:
        defects.append(text)


def binary():
    for profile in ("debug", "release"):
        path = ROOT / "target" / profile / "montagent"
        if path.exists():
            return str(path)
    sys.exit("no montagent binary: run `cargo build -p montagent` first")


def run(*args):
    out = subprocess.run([binary(), *args], capture_output=True, text=True)
    return out.returncode, json.loads(out.stdout)


def intervals(project, frm, to):
    _, answer = run("query", str(project), "--from", str(frm), "--to", str(to), "--json")
    return answer["query"]["intervals"]


def visual_runs(cut_list):
    """ADR-0094 section 1: drop audio from each presence set, merge equal neighbours."""
    runs = []
    for iv in cut_list:
        present = frozenset(p["id"] for p in iv["present"] if p["type"] != "audio")
        if runs and runs[-1]["present"] == present and runs[-1]["end"] == iv["start"]:
            runs[-1]["end"] = iv["end"]
        else:
            runs.append({"start": iv["start"], "end": iv["end"], "present": present})
    return runs


def first_painted(start, end, fps):
    """ADR-0094 section 2: the least n whose floor(n * 1000 / fps) lies in [start, end)."""
    n = start * fps // 1000
    while n * 1000 // fps < start:
        n += 1
    ms = n * 1000 // fps
    return ms if ms < end else None


# ---- 1. The fixture: 46 intervals, 18 visual runs, every one painted ------------------

doc = json.loads(FIXTURE.read_text())
fps, duration = doc["fps"], doc["duration"]
cuts = intervals(FIXTURE, 0, duration)
runs = visual_runs(cuts)
unpainted = [r for r in runs if first_painted(r["start"], r["end"], fps) is None]

claim(len(cuts) == 46, f"fixture cut list over [0, {duration}) has 46 intervals (got {len(cuts)})")
claim(len(runs) == 18, f"audio filter + re-merge leaves 18 visual runs (got {len(runs)})")
claim(unpainted == [], f"no visual run lacks a painted frame (got {len(unpainted)})")

# ---- 2. ADR-0094 section 4's example is a pre-merge interval -----------------------------

example = [iv for iv in cuts if (iv["start"], iv["end"]) == (56112, 56116)]
claim(len(example) == 1, "the 4 ms interval 56112..56116 exists in query's cut list")
claim(first_painted(56112, 56116, fps) is None, "that interval, taken alone, paints no frame")

before = next(iv for iv in cuts if iv["end"] == 56112)
changed = {p["id"]: p["type"] for p in before["present"]}.items() ^ {
    p["id"]: p["type"] for p in example[0]["present"]}.items()
claim(changed == {("vo-quiz", "audio")},
      f"the only change at 56112 is audio element vo-quiz (got {sorted(changed)})")

host = next(r for r in runs if r["start"] <= 56112 and 56116 <= r["end"])
painted = first_painted(host["start"], host["end"], fps)
claim(painted is not None and host["end"] - host["start"] > 1000 // fps,
      f"after the re-merge it sits inside visual run {host['start']}..{host['end']}, "
      f"which paints from {painted} ms")

# ---- 3. The condition is reachable, and validate is silent on it -------------------------

constructed = {
    "frame": {"width": 1080, "height": 1920},
    "fps": 25,
    "background": "#FFFFFF",
    "duration": 2000,
    "tracks": [
        {"name": "bg", "layer": 0, "elements": [
            {"id": "bg", "type": "rect", "start": 0, "end": 2000, "x": 0, "y": 0,
             "origin": "top-left", "width": 1080, "height": 1920, "fill": "#222222"}]},
        {"name": "a", "layer": 1, "elements": [
            {"id": "a", "type": "rect", "start": 0, "end": 1010, "x": 100, "y": 100,
             "origin": "top-left", "width": 400, "height": 400, "fill": "#FF0000"}]},
        {"name": "b", "layer": 2, "elements": [
            {"id": "b", "type": "rect", "start": 1030, "end": 2000, "x": 100, "y": 100,
             "origin": "top-left", "width": 400, "height": 400, "fill": "#0000FF"}]},
    ],
}
with tempfile.TemporaryDirectory() as tmp:
    project = pathlib.Path(tmp) / "unpainted.montagent.json"
    project.write_text(json.dumps(constructed))
    c_runs = visual_runs(intervals(project, 0, 2000))
    c_unpainted = [(r["start"], r["end"], sorted(r["present"]))
                   for r in c_runs if first_painted(r["start"], r["end"], 25) is None]
    claim(c_unpainted == [(1010, 1030, ["bg"])],
          f"a constructed document has one unpainted visual state, 1010..1030 {{bg}} "
          f"(got {c_unpainted})")

    # When ADR-0105 was accepted this claim read "validate is silent on it", and passed.
    # #437 closed that gap, so the claim now checks the closing: one N-QUANTIZATION at
    # review, naming the state.
    code, report = run("validate", str(project), "--json")
    states = [(f["class"], f["fields"].get("from"), f["fields"].get("to"))
              for f in report["findings"] if f["code"] == "N-QUANTIZATION"]
    claim(code == 0 and states == [("review", 1010, 1030)],
          f"validate reports it (#437): exit 0, one N-QUANTIZATION at review for 1010..1030 "
          f"(got exit {code}, {states})")

print()
if defects:
    print(f"{len(defects)} claim(s) failed")
    sys.exit(1)
print("all claims pass")
