#!/usr/bin/env python3
"""Re-derive ADR-0103's correction of ADR-0095 section 2's cropped-band figures.

Run from anywhere:  python3 docs/research/contact-sheet-crop/check_band_geometry.py
Exits non-zero, naming every defect, when it stops reproducing.

Why this exists
---------------
ADR-0095 section 2 argues that tile count is aspect-dependent -- the reason count is
*derived* from a width constant and never set by the caller -- and cites two numbers for
the cropped caption band: a 448 px tile at 18 tiles, and 112 tiles admitted at the 180 px
target. Neither is in that ADR's own check
(`docs/research/contact-sheet-budget/check_tile_budget.py`), and #396's FINDINGS.md states
**429 px** for the same sheet.

Both of ADR-0095's numbers reproduce exactly -- on a band composed with **no label strip**.
#396's sheets carry an 11% label strip under every tile (`make_sheets.py`'s `label_frac`
default, the sheets committed on `prototype/contact-sheet-legibility`), and ADR-0098 has
since made a label mandatory and floored its type at 8 px served. So the labelless figures
are not admissible for a sheet this project would ship, and the honest pair is **429 px and
98 tiles**.

The argument ADR-0095 built on them is unaffected: 98 against 18 is still 5.4x, so count is
still aspect-dependent and still derived. Only the constants move.

`tok`, `resized`, `served` and `best_grid` below are copied **verbatim** from
`docs/research/contact-sheet-legibility/make_sheets.py` on branch
`prototype/contact-sheet-legibility` (91f32677), so this file re-derives #396's numbers with
#396's own geometry rather than a reimplementation of it.
"""
import math
import sys

STD = (1568, 1568)            # standard tier: max long edge px, max visual tokens

# ---- verbatim from #396's make_sheets.py ------------------------------------------------

def tok(w, h): return math.ceil(w / 28) * math.ceil(h / 28)

def resized(width, height, max_edge=1568, max_tokens=1568):
    def fits(w, h):
        return (math.ceil(w / 28) * 28 <= max_edge
                and math.ceil(h / 28) * 28 <= max_edge
                and tok(w, h) <= max_tokens)
    if fits(width, height): return (width, height)
    if height > width:
        rh, rw = resized(height, width, max_edge, max_tokens); return (rw, rh)
    ar = width / height; lo, hi = 1, width
    while lo + 1 < hi:
        mid = (lo + hi) // 2
        if fits(mid, max(round(mid / ar), 1)): lo = mid
        else: hi = mid
    return (lo, max(round(lo / ar), 1))

def served(w, h, tier=STD):
    a, b = resized(w, h, tier[0], tier[1]); return a, b, tok(a, b)

def best_grid(n, tile_w, tile_h, tier=STD):
    """The (cols, rows) giving the largest served tile for n tiles of this shape."""
    best = None
    for c in range(1, n + 1):
        r = math.ceil(n / c)
        sw, sh, t = served(c * tile_w, r * tile_h, tier)
        px = (sw / c) * (sh / r)
        if best is None or px > best[0]:
            best = (px, c, r, sw, sh, t)
    return best[1], best[2]

# ---- this file's own composition, following make_sheets.py's `build` --------------------

LABEL_FRAC = 0.11             # make_sheets.py's default and the committed sheets' value

WHOLE = (1080, 1920)          # a whole 9:16 frame
BAND = (1080, 360)            # #396's caption band, the rect (0,1300,1080,360)


def sheet(n, src, label_frac=LABEL_FRAC):
    """Served tile width for n tiles of a src picture, on the grid closest to square."""
    src_w, src_h = src
    lab_h = int(src_h * label_frac)
    cell_w, cell_h = src_w, src_h + lab_h
    cols, rows = best_grid(n, cell_w, cell_h)
    sw, sh, t = served(cols * cell_w, rows * cell_h)
    return dict(n=n, cols=cols, rows=rows, tile_w=sw / cols, tokens=t)


def max_admissible(src, floor, label_frac=LABEL_FRAC):
    """The largest n whose sheet still serves tiles at >= floor px wide."""
    best = None
    for n in range(1, 400):
        s = sheet(n, src, label_frac)
        if s["tile_w"] >= floor:
            best = s
        elif best is not None:
            return best, s
    return best, None


defects = []


def claim(label, got, want, tol=0.5):
    ok = abs(got - want) <= tol if isinstance(want, float) else got == want
    print(f"  {'PASS' if ok else 'FAIL'}  {label}: {got} (expected {want})")
    if not ok:
        defects.append(f"{label}: got {got}, expected {want}")


print("1. The two 18-tile sheets ADR-0095 section 2 compares, with #396's label strip")
whole18 = sheet(18, WHOLE)
band18 = sheet(18, BAND)
claim("whole 9:16, 18 tiles, served tile width", round(whole18["tile_w"], 1), 184.5)
claim("whole 9:16, 18 tiles, grid", (whole18["cols"], whole18["rows"]), (6, 3))
claim("band 3:1, 18 tiles, served tile width", round(band18["tile_w"], 1), 429.3)
claim("band 3:1, 18 tiles, grid", (band18["cols"], band18["rows"]), (3, 6))
claim("band 3:1 is this much of the linear scale of a whole frame",
      round(band18["tile_w"] / whole18["tile_w"], 2), 2.33)

print("\n2. ADR-0103's corrected counts at ADR-0095's 180 px target")
w180, w180_bad = max_admissible(WHOLE, 180)
b180, b180_bad = max_admissible(BAND, 180)
claim("whole 9:16, max tiles at 180 px", w180["n"], 18)
claim("whole 9:16, first count below the target", w180_bad["n"], 19)
claim("band 3:1, max tiles at 180 px (ADR-0095 says 112)", b180["n"], 98)
claim("band 3:1, first count below the target", b180_bad["n"], 99)
claim("count is aspect-dependent by this factor", round(b180["n"] / w180["n"], 1), 5.4)

print("\n3. ADR-0095's own numbers reproduce with the label strip removed")
band18_bare = sheet(18, BAND, label_frac=0.0)
b180_bare, _ = max_admissible(BAND, 180, label_frac=0.0)
claim("band 3:1, 18 tiles, NO label: ADR-0095's 448 px",
      round(band18_bare["tile_w"], 1), 448.0)
claim("band 3:1, max tiles at 180 px, NO label: ADR-0095's 112", b180_bare["n"], 112)

print("\n4. ADR-0095's 140 px floor, for completeness")
w140, w140_bad = max_admissible(WHOLE, 140)
b140, _ = max_admissible(BAND, 140)
claim("whole 9:16, max tiles at 140 px", w140["n"], 30)
claim("whole 9:16, first count below the floor", w140_bad["n"], 31)
claim("band 3:1, max tiles at 140 px", b140["n"], 168)

print("\n5. A whole-frame sheet saturates the token cap, so tokens never gate this")
for n in (4, 18, 30, 48):
    t = sheet(n, WHOLE)["tokens"]
    if not 1518 <= t <= 1568:
        defects.append(f"whole-frame sheet at n={n} costs {t} tokens, outside 1518-1568")
    print(f"  {'PASS' if 1518 <= t <= 1568 else 'FAIL'}  "
          f"whole 9:16, {n} tiles: {t} tokens (expected 1518-1568)")

print()
if defects:
    print(f"{len(defects)} claim(s) no longer hold:")
    for d in defects:
        print(f"  - {d}")
    sys.exit(1)
print("All claims hold.")
