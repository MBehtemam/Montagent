#!/usr/bin/env python3
"""Re-executable check for issue #400: served type size of honest label strings.

#396 measured exactly ONE label string -- `9 42763ms item-08 word` -- and its
`item-08 word` half is a hand-written human gist from `LABEL` in
`../contact-sheet-legibility/make_sheets.py`, not anything the document says.
This script measures the strings the sheet would actually have to print if
ADR-0094 is honoured: the sampled instant AND the run boundary, and the real
presence set as real element ids.

It reuses #396's own machinery unmodified -- `served()` / `best_grid()` from
`make_sheets.py` and a verbatim copy of `fitted()` from `label_test.py` -- and
needs no rendered frames, because label fitting is a pure text measurement
against the same font at the same authored geometry.

Every number this prints is asserted. Exits non-zero the moment one stops
reproducing.

    python3 docs/research/label-legibility/check_label_legibility.py
"""
import json
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
P396 = os.path.join(HERE, "..", "contact-sheet-legibility")
sys.path.insert(0, P396)

from PIL import Image, ImageDraw, ImageFont  # noqa: E402
from make_sheets import FONT, INSTANTS, LABEL, best_grid, served  # noqa: E402

PROJECT = os.path.join(
    HERE, "..", "..", "..", "fixtures", "en-halloween-decorating",
    "en-halloween-decorating.montagent.json")

# Sheet geometry, exactly as #396 composed it (make_sheets.build / build_labelled).
SRC_W, SRC_H = 1080, 1920
LABEL_FRAC = 0.11
LAB_H = int(SRC_H * LABEL_FRAC)        # 211 authored px of label strip
MAX_H = LAB_H * 0.78                   # the cap fitted() starts its search at
VISUAL = {"image", "rect", "text"}     # ADR-0094's filter: visual element types

FAILS = []


def check(label, got, want, tol=0.0):
    ok = abs(got - want) <= tol if isinstance(want, (int, float)) else got == want
    if not ok:
        FAILS.append(f"{label}: got {got!r}, expected {want!r} (tol {tol})")
    return ok


def fitted_size(draw, text, box_w=SRC_W, max_h=MAX_H):
    """Verbatim fitted() from #396's label_test.py, returning the size only."""
    for s in range(int(max_h), 5, -1):
        f = ImageFont.truetype(FONT, s)
        if draw.textlength(text, font=f) <= box_w * 0.94:
            return s
    return None


# ---------------------------------------------------------------------------
# 1. The real visual states, derived from the fixture (ADR-0094's rule)
# ---------------------------------------------------------------------------
def visual_runs():
    """Cut list filtered to visual element types, adjacent equal-set runs merged."""
    doc = json.load(open(PROJECT))
    fps, dur = doc["fps"], doc["duration"]
    els = [e for t in doc["tracks"] for e in t.get("elements", [])
           if e["type"] in VISUAL]
    pts = sorted({0, dur} | {e["start"] for e in els} | {e["end"] for e in els})
    runs = []
    for a, b in zip(pts, pts[1:]):
        s = frozenset(e["id"] for e in els if e["start"] <= a < b <= e["end"])
        if runs and runs[-1][2] == s:
            runs[-1][1] = b
        else:
            runs.append([a, b, s])
    return fps, dur, len(pts), runs


FPS, DUR, N_BOUNDARIES, RUNS = visual_runs()

# ADR-0094 decision 2: the instant is the least frame index n whose painted
# millisecond floor(n * 1000 / fps) falls inside the run. At 25 fps that is the
# least multiple of 40 >= the run start.
FRAME_MS = 1000.0 / FPS


def sampled_instant(start, end):
    n = math.ceil(start / FRAME_MS)
    ms = int(n * 1000 // FPS)
    return ms if ms < end else None


for r in RUNS:
    r.append(sampled_instant(r[0], r[1]))

# --- assertions on the derivation itself -----------------------------------
check("visual boundary points", N_BOUNDARIES, 19)
check("merged visual runs", len(RUNS), 18)
check("run starts == #396/#397 instant set", [r[0] for r in RUNS], INSTANTS)
check("every run has a painted frame", sum(r[3] is None for r in RUNS), 0)
check("fps", FPS, 25)
check("duration ms", DUR, 65216)

set_sizes = [len(r[2]) for r in RUNS]
check("min presence-set size", min(set_sizes), 10)
check("max presence-set size", max(set_sizes), 13)
check("modal presence-set size is 10", set_sizes.count(10), 12)

BUSIEST = max(RUNS, key=lambda r: len(r[2]))
check("busiest run start", BUSIEST[0], 47343)
check("busiest run size", len(BUSIEST[2]), 13)

# 8 elements are the always-on header chrome, present in all 18 states.
ALWAYS = frozenset.intersection(*[r[2] for r in RUNS])
check("always-present chrome count", len(ALWAYS), 8)

# The run #396's calibration label names: index 9, the word-08 run.
T9 = RUNS[8]
check("run 9 start", T9[0], 42763)
check("run 9 sampled instant", T9[3], 42800)
check("run 9 boundary != instant", T9[0] != T9[3], True)

TYPICAL = RUNS[2]   # 5316-10468, a 10-element state: the modal shape
check("typical run size", len(TYPICAL[2]), 10)


def ids(run):
    return " ".join(sorted(run[2]))


def head(run, i):
    """index + sampled instant + run boundary, per ADR-0094 decision 2."""
    return f"{i} {run[3]}ms b{run[0]}"


# ---------------------------------------------------------------------------
# 2. Working points: served tile width
# ---------------------------------------------------------------------------
# ADR-0095's target (18 tiles, 6x3) and floor (~30 tiles), from #396's geometry,
# plus the two nominal widths ADR-0095 states as policy.
def grid_tile_width(n, cols, rows):
    cell_h = SRC_H + LAB_H
    sw, sh, tokens = served(cols * SRC_W, rows * cell_h)
    return sw / cols, sw, sh, tokens


G18 = grid_tile_width(18, 6, 3)
G30 = grid_tile_width(30, 10, 3)
G72 = grid_tile_width(72, *best_grid(72, SRC_W, SRC_H + LAB_H))

check("6x3 n=18 served tile width", round(G18[0], 1), 184.5, 0.05)
check("6x3 n=18 tokens", G18[3], 1560)
check("10x3 n=30 served tile width", round(G30[0], 1), 141.9, 0.05)
check("best-grid n=72 served tile width", round(G72[0], 1), 92.2, 0.05)

WORKING = [
    ("180 px (ADR-0095 target)", 180.0),
    ("140 px (ADR-0095 floor)", 140.0),
    ("184.5 px (real 6x3, n=18)", G18[0]),
    ("141.9 px (real 10x3, n=30)", G30[0]),
    ("92.2 px (n=72, #396 calibration)", G72[0]),
]

REF_PX = 6.3   # #396's readable reference (see the calibration assertion below)

# ---------------------------------------------------------------------------
# 3. Candidates
# ---------------------------------------------------------------------------
CANDS = [
    ("1 index + instant",                 f"9 {T9[3]}ms"),
    ("2 + run boundary (ADR-0094)",       head(T9, 9)),
    ("3 #396 baseline (hand gist)",       "9 42763ms item-08 word"),
    ("4 + one element id",                head(T9, 9) + " photo-08"),
    ("5 + group id",                      head(T9, 9) + " item-08"),
    ("6 + typical presence set (10)",     head(TYPICAL, 3) + " " + ids(TYPICAL)),
    ("7 + busiest presence set (13)",     head(BUSIEST, 10) + " " + ids(BUSIEST)),
]

scratch = ImageDraw.Draw(Image.new("RGB", (8, 8)))


def served_px(text, tile_w):
    s = fitted_size(scratch, text)
    if s is None:
        return None, None
    return s, round(s * tile_w / SRC_W, 2)


# ---------------------------------------------------------------------------
# 4. Report
# ---------------------------------------------------------------------------
print(f"font: {os.path.basename(FONT)}   authored tile {SRC_W}x{SRC_H}  "
      f"label strip {LAB_H}px (11%)  fitted() cap {MAX_H:.1f}px  fit target 94% of tile")
print(f"fixture: 18 visual states, {N_BOUNDARIES} visual boundary points, "
      f"presence-set sizes {min(set_sizes)}-{max(set_sizes)} "
      f"({len(ALWAYS)} always-on header elements)\n")

for name, w in WORKING:
    print(f"--- served tile width {name} ---")
    print(f"{'candidate':<32} {'chars':>5} {'authored':>8} {'served px':>9} "
          f"{'vs 6.3':>7} {'px*chars':>8}")
    for cname, text in CANDS:
        s, px = served_px(text, w)
        rel = "above" if px >= REF_PX else "BELOW"
        print(f"{cname:<32} {len(text):>5} {s:>8} {px:>9} {rel:>7} "
              f"{px * len(text):>8.1f}")
    print()

print("candidate strings, verbatim:")
for cname, text in CANDS:
    print(f"  {cname:<32} ({len(text):>3}) {text}")
print()

# --- spread across all 18 real states, candidate 6/7's shape ---------------
print("--- per-tile spread, full presence set on every one of the 18 real states ---")
for name, w in WORKING[:4]:
    rows = []
    for i, r in enumerate(RUNS, 1):
        txt = head(r, i) + " " + ids(r)
        s, px = served_px(txt, w)
        rows.append((px, len(txt), s, i))
    lo, hi = min(rows), max(rows)
    print(f"{name:<32} served {lo[0]}-{hi[0]} px   "
          f"smallest type = tile {lo[3]} ({lo[1]} chars, authored {lo[2]}px); "
          f"largest = tile {hi[3]} ({hi[1]} chars, authored {hi[2]}px)")
print()

# ---------------------------------------------------------------------------
# 5. Assertions on every number quoted in the report
# ---------------------------------------------------------------------------
#                                chars, authored px, served@180, served@140
EXPECT = {
    "1 index + instant":            (9,   164, 27.33, 21.26),
    "2 + run boundary (ADR-0094)":  (16,  103, 17.17, 13.35),
    "3 #396 baseline (hand gist)":  (22,   81, 13.50, 10.50),
    "4 + one element id":           (25,   69, 11.50,  8.94),
    "5 + group id":                 (24,   72, 12.00,  9.33),
    "6 + typical presence set (10)": (122,  16,  2.67,  2.07),
    "7 + busiest presence set (13)": (167,  11,  1.83,  1.43),
}
for cname, text in CANDS:
    e = EXPECT[cname]
    check(f"chars[{cname}]", len(text), e[0])
    check(f"authored[{cname}]", fitted_size(scratch, text), e[1])
    check(f"served@180[{cname}]", served_px(text, 180.0)[1], e[2], 0.01)
    check(f"served@140[{cname}]", served_px(text, 140.0)[1], e[3], 0.01)

# --- calibration against #396's 6.3 px reference ---------------------------
# FINDINGS.md quotes `9 42763ms item-08 word` alongside "6.3 px served type at
# 92 px tiles". 6.3 is in fact the *sheet minimum* that `label_test.py` prints
# for E-long-72 -- the LONGEST of that sheet's 72 labels (authored 74 px), not
# the 22-char exemplar, which measures 6.9 px there. Both are re-derived here so
# the reference is unambiguous.
LONG396 = [f"{i+1}  {ms}ms  {LABEL.get(ms, '')}" for i, ms in enumerate((INSTANTS * 4)[:72])]
a396 = [fitted_size(scratch, t) for t in LONG396]
s396 = [round(s * G72[0] / SRC_W, 1) for s in a396]
check("E-long-72 authored range", (min(a396), max(a396)), (74, 164))
check("E-long-72 served minimum is #396's 6.3 px", min(s396), 6.3, 0.001)
check("E-long-72 served maximum", max(s396), 14.0, 0.001)
check("the 22-char exemplar itself at 92.2 px",
      served_px("9 42763ms item-08 word", G72[0])[1], 6.92, 0.01)
LONGEST396 = max(LONG396, key=len)
check("longest E-long-72 label length", len(LONGEST396), 25)

# --- inverse proportionality ----------------------------------------------
# For a WIDTH-limited string, served px x char count is near-constant, because
# fitted() sets the size from the string's rendered advance width. It holds to
# within +/-9% of the mean across a >10x range of lengths (16 -> 167 chars). The
# residual is glyph mix (digits are wider than lowercase in this face) plus the
# integer rounding of the authored size.
wprods = [served_px(t, 180.0)[1] * len(t) for n, t in CANDS if n != "1 index + instant"]
spread = (max(wprods) - min(wprods)) / (sum(wprods) / len(wprods))
check("inverse-proportionality band at 180px", round(spread, 3), 0.172, 0.001)
check("inverse-proportionality band at 180px is under +/-9%", spread / 2 < 0.09, True)
check("px*chars product range at 180px",
      (round(min(wprods)), round(max(wprods))), (275, 326))
# Candidate 1 is the exception and refutes it there: at 9 chars the fit is capped
# by the label strip's HEIGHT (fitted() starts at int(164.58) = 164), not width.
check("candidate 1 is height-capped, not width-capped",
      fitted_size(scratch, CANDS[0][1]), int(MAX_H))

# --- the variable-length presence set gives per-tile sizes that differ ------
sp = [served_px(head(r, i) + " " + ids(r), 180.0)[1] for i, r in enumerate(RUNS, 1)]
check("per-tile served px differs across one sheet", len(set(sp)) > 1, True)
check("full-set spread at 180px, min", min(sp), 1.83, 0.01)
check("full-set spread at 180px, max", max(sp), 2.67, 0.01)
sp140 = [served_px(head(r, i) + " " + ids(r), 140.0)[1] for i, r in enumerate(RUNS, 1)]
check("full-set spread at 140px, min", min(sp140), 1.43, 0.01)
check("full-set spread at 140px, max", max(sp140), 2.07, 0.01)

if FAILS:
    print(f"FAIL ({len(FAILS)}):")
    for f in FAILS:
        print("  -", f)
    sys.exit(1)
print("all checks pass")
