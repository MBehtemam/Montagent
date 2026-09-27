#!/usr/bin/env python3
"""Re-executable check for the NUMERIC claims in FINDINGS.md (issue #396).

The visibility thresholds themselves are perceptual and are grounded by the
committed sheets, not by this script (docs/agents/domain.md tiers them that
way). What IS re-derivable is the geometry the thresholds are *expressed in*:
the served tile size and token cost of every layout, and the three structural
claims that follow from them.

    python3 check_contact_sheet_geometry.py          # geometry only
    MONTAGENT=target/release/montagent \
    PROJECT=<doctored>.montagent.json \
        python3 check_contact_sheet_geometry.py      # also re-checks #402

Exits non-zero the moment any claim stops reproducing.
"""
import math, os, subprocess, sys, tempfile

FAILURES = []


def check(label, got, want):
    ok = got == want
    print(f"  {'ok  ' if ok else 'FAIL'} {label}: {got}" + ("" if ok else f"  want {want}"))
    if not ok:
        FAILURES.append(label)


def claim(label, cond, detail=""):
    print(f"  {'ok  ' if cond else 'FAIL'} {label}" + (f": {detail}" if detail else ""))
    if not cond:
        FAILURES.append(label)


# --- the cost model, as established and checked by #397 ----------------------
STD = (1568, 1568)      # standard tier: (max long edge px, max visual tokens)
HI = (2576, 4784)       # Claude 4.7 and later


def tok(w, h):
    return math.ceil(w / 28) * math.ceil(h / 28)


def resized(width, height, max_edge=1568, max_tokens=1568):
    def fits(w, h):
        return (math.ceil(w / 28) * 28 <= max_edge
                and math.ceil(h / 28) * 28 <= max_edge
                and tok(w, h) <= max_tokens)
    if fits(width, height):
        return (width, height)
    if height > width:
        rh, rw = resized(height, width, max_edge, max_tokens)
        return (rw, rh)
    ar = width / height
    lo, hi = 1, width
    while lo + 1 < hi:
        mid = (lo + hi) // 2
        if fits(mid, max(round(mid / ar), 1)):
            lo = mid
        else:
            hi = mid
    return (lo, max(round(lo / ar), 1))


def served(w, h, tier=STD):
    a, b = resized(w, h, tier[0], tier[1])
    return a, b, tok(a, b)


# A tile is the 1080x1920 frame plus an 11% label strip; that composite is what
# gets gridded, so it is what the geometry must be computed on.
TILE_W, TILE_H = 1080, 1920
CELL_H = TILE_H + int(TILE_H * 0.11)


def layout(n, cols, rows, tile_w=TILE_W, cell_h=CELL_H, tier=STD):
    sw, sh, t = served(cols * tile_w, rows * cell_h, tier)
    return dict(n=n, cols=cols, rows=rows, served=(sw, sh), tokens=t,
                tile_w=sw / cols, tile_h=(sh / rows) * (TILE_H / cell_h))


def best_grid(n, tile_w=TILE_W, cell_h=CELL_H, tier=STD):
    best = None
    for c in range(1, n + 1):
        r = math.ceil(n / c)
        L = layout(n, c, r, tile_w, cell_h, tier)
        px = L["tile_w"] * L["tile_h"]
        if best is None or px > best[0]:
            best = (px, L)
    return best[1]


print("A. The tile-count sweep FINDINGS.md quotes (standard tier, 9:16 frames)")
# (n, cols, rows, served tile width rounded, tokens)
SWEEP = [
    (4,  2, 2, 392, 1568),
    (18, 6, 3, 184, 1560),
    (28, 7, 4, 148, 1554),
    (32, 8, 4, 138, 1560),
    (48, 12, 4, 114, 1568),
    (72, 12, 6, 92, 1560),
]
for n, c, r, want_w, want_t in SWEEP:
    L = layout(n, c, r)
    check(f"n={n} {c}x{r} served tile width", round(L["tile_w"]), want_w)
    check(f"n={n} {c}x{r} tokens", L["tokens"], want_t)

print("\nB. Every sheet in the sweep fits one standard-tier image budget")
for n, c, r, _, _ in SWEEP:
    L = layout(n, c, r)
    claim(f"n={n} within 1568 visual tokens", L["tokens"] <= 1568, f"{L['tokens']} tok")

print("\nC. A strip is dominated: smaller tiles AND less of the budget spent")
for n in (18, 30):
    grid = best_grid(n)
    for c, r, tag in ((n, 1, "horizontal strip"), (1, n, "vertical strip")):
        s = layout(n, c, r)
        claim(f"n={n} {tag} tile is smaller than the best grid's",
              s["tile_w"] < grid["tile_w"],
              f"{s['tile_w']:.0f}px vs {grid['tile_w']:.0f}px")
        claim(f"n={n} {tag} leaves budget unspent",
              s["tokens"] < grid["tokens"],
              f"{s['tokens']} tok vs {grid['tokens']} tok")

print("\nD. The best grid for a 9:16 tile is near-square in SHEET aspect")
for n in (12, 18, 24, 32, 48, 72):
    g = best_grid(n)
    sw, sh = g["served"]
    ar = max(sw, sh) / min(sw, sh)
    claim(f"n={n} best grid {g['cols']}x{g['rows']} sheet aspect within 1:1.7",
          ar <= 1.7, f"aspect 1:{ar:.2f}")

print("\nE. Cropping to a band buys more than 2x the linear tile scale at n=18")
BAND_W, BAND_H = 1080, 360
BAND_CELL = BAND_H + int(BAND_H * 0.11)
whole = layout(18, 6, 3)
band = layout(18, 3, 6, BAND_W, BAND_CELL)
ratio = (band["served"][0] / 3) / whole["tile_w"]
claim("band tile is >2x the whole-frame tile width", ratio > 2.0,
      f"{band['served'][0]/3:.0f}px vs {whole['tile_w']:.0f}px = {ratio:.2f}x")
claim("both sheets cost about the same", abs(band["tokens"] - whole["tokens"]) < 60,
      f"{band['tokens']} tok vs {whole['tokens']} tok")

print("\nF. The high-resolution tier does NOT buy more tiles at the same cost")
hi = layout(18, 6, 3, tier=HI)
claim("4.7+ tier serves a bigger sheet but charges ~3x for it",
      hi["tokens"] > 3 * 1568 / 2 and hi["tile_w"] > whole["tile_w"],
      f"{hi['tokens']} tok, tile {hi['tile_w']:.0f}px "
      f"(standard: {whole['tokens']} tok, {whole['tile_w']:.0f}px)")

print("\nG. #402 reproduces: --crop without --full returns half the asked region")
BIN, PROJ = os.environ.get("MONTAGENT"), os.environ.get("PROJECT")
if not (BIN and PROJ and os.path.exists(BIN) and os.path.exists(PROJ)):
    print("  skip  set MONTAGENT= and PROJECT= to re-check (needs a built binary)")
else:
    try:
        from PIL import Image
    except ImportError:
        print("  skip  Pillow not installed")
    else:
        with tempfile.TemporaryDirectory() as d:
            got = {}
            for tag, extra in (("half", []), ("full", ["--full"])):
                p = os.path.join(d, f"{tag}.png")
                subprocess.run([BIN, "frame", PROJ, "--at", "47343", "--crop",
                                "0,1300,1080,360", "--png", "--out", p] + extra,
                               check=True, capture_output=True)
                got[tag] = Image.open(p).size
            check("--crop --full returns the region asked for", got["full"], (1080, 360))
            check("--crop alone returns it halved (the #402 defect)", got["half"], (540, 180))

print()
if FAILURES:
    print(f"FAILED ({len(FAILURES)}): " + "; ".join(FAILURES))
    sys.exit(1)
print("All geometry claims in FINDINGS.md reproduce.")
