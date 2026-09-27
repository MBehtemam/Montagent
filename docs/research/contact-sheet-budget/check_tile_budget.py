#!/usr/bin/env python3
"""Re-executable check for the NUMERIC claims in ADR-0095 (issue #399).

ADR-0095 decides that the contact sheet's budget is denominated in **served tile
width**, with a 180 px target and a 140 px floor, and that no wall-clock budget
needs to exist because the width floor already bounds the time. Every number that
argument spends is re-derived here.

    python3 check_tile_budget.py                     # geometry + cost claims

    MONTAGENT=target/release/montagent \
    PROJECT=fixtures/en-halloween-decorating/en-halloween-decorating.montagent.json \
        python3 check_tile_budget.py                 # also re-measures wall clock

The geometry half shares its cost model with #396's
`../contact-sheet-legibility/check_contact_sheet_geometry.py` deliberately — that
script grounds the legibility *thresholds*, this one grounds the *policy* derived
from them, and if the shared model ever drifts both must fail rather than one.

Exits non-zero the moment any claim stops reproducing.
"""
import math
import os
import subprocess
import sys
import time

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


# --- the cost model, as established by #397 and re-used by #396 --------------
STD = (1568, 1568)  # standard tier: (max long edge px, max visual tokens)
HI = (2576, 4784)  # Claude 4.7 and later


def tok(w, h):
    return math.ceil(w / 28) * math.ceil(h / 28)


def resized(width, height, max_edge=1568, max_tokens=1568):
    def fits(w, h):
        return (
            math.ceil(w / 28) * 28 <= max_edge
            and math.ceil(h / 28) * 28 <= max_edge
            and tok(w, h) <= max_tokens
        )

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


# A tile is the 1080x1920 frame plus an 11% label strip, per #396's prototype:
# that composite is what gets gridded, so it is what the geometry runs on.
TILE_W, TILE_H = 1080, 1920
CELL_H = TILE_H + int(TILE_H * 0.11)


def layout(n, cols, rows, tile_w=TILE_W, cell_h=CELL_H, tier=STD):
    sw, sh, t = served(cols * tile_w, rows * cell_h, tier)
    return dict(
        n=n,
        cols=cols,
        rows=rows,
        served=(sw, sh),
        tokens=t,
        tile_w=sw / cols,
        tile_h=(sh / rows) * (TILE_H / cell_h),
    )


def best_grid(n, tile_w=TILE_W, cell_h=CELL_H, tier=STD):
    """The near-square grid ADR-0094/#396 prescribe: maximise served tile area."""
    best = None
    for c in range(1, n + 1):
        r = math.ceil(n / c)
        L = layout(n, c, r, tile_w, cell_h, tier)
        px = L["tile_w"] * L["tile_h"]
        if best is None or px > best[0]:
            best = (px, L)
    return best[1]


def max_tiles_at(min_width, tier=STD, tile_w=TILE_W, cell_h=CELL_H, cap=400):
    """The largest n whose best grid still serves tiles at >= min_width px.

    This is the direction ADR-0095 actually uses: the constant is the width, and
    the tile count is derived from it. Monotonicity is not assumed — every n up to
    the cap is tested and the largest passing one wins.
    """
    best = 0
    for n in range(1, cap + 1):
        if best_grid(n, tile_w, cell_h, tier)["tile_w"] >= min_width:
            best = n
    return best


TARGET_TILE_WIDTH = 180  # ADR-0095's target
FLOOR_TILE_WIDTH = 140  # ADR-0095's floor (#396's measured cliff)


print("A token budget never fires: the sheet saturates the tier at every count")
sweep = {n: best_grid(n) for n in range(4, 49)}
toks = [L["tokens"] for L in sweep.values()]
check("minimum tokens over n=4..48", min(toks), 1518)
check("maximum tokens over n=4..48", max(toks), 1568)
claim(
    "every count spends within 4% of the 1568-token cap",
    all(t >= 1568 * 0.96 for t in toks),
    f"min {min(toks)} of 1568 = {min(toks) / 1568:.1%}",
)
claim(
    "tokens do not fall monotonically with tile count -- they are flat, not a budget",
    not all(
        sweep[n]["tokens"] >= sweep[n + 1]["tokens"] for n in range(4, 48)
    ),
    "a denominator has to vary with the thing it bounds",
)

print()
print("Served tile width, by contrast, falls monotonically and is the real knob")
widths = [sweep[n]["tile_w"] for n in range(4, 49)]
claim(
    "tile width is non-increasing across n=4..48",
    all(widths[i] >= widths[i + 1] for i in range(len(widths) - 1)),
    f"{widths[0]:.1f} px at n=4 down to {widths[-1]:.1f} px at n=48",
)

print()
print("The two constants derive two exact tile counts (9:16 frame)")
check("max tiles at the 180 px target", max_tiles_at(TARGET_TILE_WIDTH), 18)
check("max tiles at the 140 px floor", max_tiles_at(FLOOR_TILE_WIDTH), 30)
check("n=18 grid", (sweep[18]["cols"], sweep[18]["rows"]), (6, 3))
check("n=18 served sheet", sweep[18]["served"], (1107, 1092))
check("n=18 tokens", sweep[18]["tokens"], 1560)
check("n=18 served tile width (rounded)", round(sweep[18]["tile_w"], 1), 184.5)
claim(
    "19 tiles falls below the target, which is why 18 is a cliff and not a rounding",
    sweep[19]["tile_w"] < TARGET_TILE_WIDTH,
    f"{sweep[19]['tile_w']:.1f} px",
)
claim(
    "31 tiles falls below the floor, which is why 30 is the floor's count",
    sweep[31]["tile_w"] < FLOOR_TILE_WIDTH,
    f"{sweep[31]['tile_w']:.1f} px",
)

print()
print("One sheet of 18 states costs exactly one `frame --full`")
_, _, full_frame = served(1080, 1920)
_, _, half_frame = served(540, 960)
check("frame --full, served on the standard tier", full_frame, 1560)
check("frame default (half scale), served", half_frame, 700)
check("the 18-tile sheet", sweep[18]["tokens"], full_frame)
claim(
    "the sheet is 18 states for the price of one full-resolution frame",
    sweep[18]["tokens"] == full_frame,
    f"{sweep[18]['tokens']} tokens either way",
)
claim(
    "and 2.2x the half-scale default",
    abs(sweep[18]["tokens"] / half_frame - 2.23) < 0.01,
    f"{sweep[18]['tokens'] / half_frame:.2f}x",
)
_, _, full_frame_hi = served(1080, 1920, HI)
check("ADR-0011's 2691 figure holds only on the high-res tier", full_frame_hi, 2691)

print()
print("The width floor is aspect-dependent, which is why width and not count is the constant")
# #396's cropped caption band: a 3:1 tile, same budget, no label strip modelled
# (the band is its own crop, so the 11% strip is not part of this composite).
BAND_W, BAND_H = 1080, 360
band_18 = best_grid(18, tile_w=BAND_W, cell_h=BAND_H)
claim(
    "the same 18 tiles cropped to a 3:1 band serve far wider than 9:16 tiles do",
    band_18["tile_w"] > sweep[18]["tile_w"] * 2,
    f"{band_18['tile_w']:.0f} px band vs {sweep[18]['tile_w']:.0f} px whole-frame",
)
claim(
    "so one tile-count constant would mean two different legibility outcomes",
    max_tiles_at(TARGET_TILE_WIDTH, tile_w=BAND_W, cell_h=BAND_H) > 18,
    f"{max_tiles_at(TARGET_TILE_WIDTH, tile_w=BAND_W, cell_h=BAND_H)} band tiles "
    f"clear 180 px against 18 whole frames",
)

print()
print("A strip is still strictly dominated, so overflow may never reshape into one")
strip = layout(18, 18, 1)
grid = sweep[18]
claim(
    "the 18x1 strip leaves most of the budget unspent",
    strip["tokens"] < grid["tokens"] * 0.5,
    f"{strip['tokens']} tokens vs the grid's {grid['tokens']}",
)
claim(
    "and serves narrower tiles while doing it",
    strip["tile_w"] < grid["tile_w"],
    f"{strip['tile_w']:.0f} px vs {grid['tile_w']:.0f} px",
)

print()
print("The refusal is the common case for anything much longer than the fixture")
# The fixture is 65216 ms and collapses to 18 visual states (ADR-0094, #396), so this
# density is ~0.276 states/second. ADR-0095 claims the fixture fits the target *exactly*
# by luck, and that roughly 110 s of the same density overflows even the floor.
FIXTURE_MS, FIXTURE_STATES = 65216, 18
density = FIXTURE_STATES / (FIXTURE_MS / 1000)
check(
    "the fixture lands exactly on the target's tile count, with nothing spare",
    FIXTURE_STATES,
    max_tiles_at(TARGET_TILE_WIDTH),
)
fits_floor_s = max_tiles_at(FLOOR_TILE_WIDTH) / density
claim(
    "at the fixture's density the floor is exhausted by ~110 s of video",
    105 <= fits_floor_s <= 115,
    f"{fits_floor_s:.0f}s of the same density reaches "
    f"{max_tiles_at(FLOOR_TILE_WIDTH)} states",
)
claim(
    "so a range only ~1.7x the fixture's length already refuses",
    fits_floor_s < (FIXTURE_MS / 1000) * 2,
    f"{fits_floor_s / (FIXTURE_MS / 1000):.2f}x the fixture",
)

print()
print("The high-res tier buys resolution, not tiles -- policy must hold on standard")
hi_18 = best_grid(18, tier=HI)
claim(
    "the same 18-tile sheet costs ~3x more on the high-res tier",
    hi_18["tokens"] > grid["tokens"] * 2.9,
    f"{hi_18['tokens']} vs {grid['tokens']} = {hi_18['tokens'] / grid['tokens']:.2f}x",
)
claim(
    "for well under 2x the tile width",
    hi_18["tile_w"] < grid["tile_w"] * 2,
    f"{hi_18['tile_w']:.0f} px vs {grid['tile_w']:.0f} px",
)

# --- the wall-clock claim, measured rather than derived -----------------------
# ADR-0095's claim is a *bound*, not a target: the width floor caps the tile count
# at 30, and 30 rasterizations land inside `preview`'s 5 s without anything
# enforcing it. Only runnable where a built binary and a project are supplied.
BIN = os.environ.get("MONTAGENT")
PROJECT = os.environ.get("PROJECT")

print()
if not (BIN and PROJECT and os.path.exists(BIN) and os.path.exists(PROJECT)):
    print("Wall clock: SKIPPED (set MONTAGENT and PROJECT to measure)")
    print("  ADR-0095's timing claims are bounds on this fixture's reference hardware;")
    print("  re-measure before citing them on different hardware.")
else:
    INSTANTS = [
        0, 3040, 6080, 9120, 12160, 15200, 18240, 21280, 24320,
        27360, 30400, 33440, 36480, 39520, 42560, 45600, 48640, 51680,
    ]

    def sweep_frames(extra):
        subprocess.run(
            [BIN, "frame", PROJECT, "--at", "1000", *extra, "--out", os.devnull],
            capture_output=True,
        )  # warm the caches first; a cold first call is a different measurement
        t0 = time.time()
        for at in INSTANTS:
            r = subprocess.run(
                [BIN, "frame", PROJECT, "--at", str(at), *extra, "--out", os.devnull],
                capture_output=True,
            )
            if r.returncode != 0:
                return None
        return time.time() - t0

    print("Wall clock: rasterization is linear in tile count, so the width floor bounds it")
    full = sweep_frames(["--full"])
    half = sweep_frames([])
    if full is None or half is None:
        claim("18 frames rasterized", False, "a frame call failed")
    else:
        per_full = full / len(INSTANTS)
        print(f"  ..  18 frames at --full: {full:.2f}s ({per_full * 1000:.0f} ms each)")
        print(f"  ..  18 frames at half scale: {half:.2f}s ({half / len(INSTANTS) * 1000:.0f} ms each)")
        claim(
            "18 full-resolution frames stay well inside preview's 5 s budget",
            full < 5.0,
            f"{full:.2f}s",
        )
        claim(
            "and 30 -- the floor's tile count -- would too, extrapolating linearly",
            per_full * 30 < 5.0,
            f"{per_full * 30:.2f}s projected",
        )
        claim(
            "but a single sheet cannot meet `frame`'s 500 ms budget (ADR-0021)",
            full > 0.5,
            f"{full:.2f}s is {full / 0.5:.1f}x it",
        )
        claim(
            "rasterizing at half scale buys far less than the 4x pixel-area ratio",
            half > full * 0.5,
            f"{(1 - half / full):.0%} saved, not 75% -- decode dominates, as ADR-0065 found",
        )
        # The densest sheet #396 tested. ADR-0095 notes it would miss preview's budget,
        # and that the legibility floor forbids it first -- the two constraints agreeing
        # is the reason no separate deadline is needed.
        claim(
            "72 tiles -- #396's densest -- would miss preview's 5 s, but the floor "
            "forbids it first",
            per_full * 72 > 5.0 and 72 > max_tiles_at(FLOOR_TILE_WIDTH),
            f"{per_full * 72:.1f}s projected, and 72 > the floor's "
            f"{max_tiles_at(FLOOR_TILE_WIDTH)} tiles",
        )

print()
if FAILURES:
    print(f"FAILED {len(FAILURES)}: " + ", ".join(FAILURES))
    sys.exit(1)
print("all claims reproduce")
