#!/usr/bin/env python3
"""Re-derives every number the #52 resolution rests on.

Exits non-zero if any of it stops reproducing. Run from the repo root.

The resolution of #52 is *no schema change*, so there is no ADR for this script
to defend. What it defends is the evidence chain that got there -- including the
three claims that were asserted during the investigation and then falsified.
"""
import json
import math
import sys
from collections import Counter
from decimal import Decimal, ROUND_CEILING
from pathlib import Path

FIXTURE = Path("fixtures/en-halloween-decorating/en-halloween-decorating.montaget.json")
IDIOM_EXAMPLE = Path("docs/research/juries/contain-slack/badge-idiom-example.montaget.json")

failures = []


def check(label, got, want):
    ok = got == want
    print(f"  {'ok  ' if ok else 'FAIL'}  {label}: {got!r}" + ("" if ok else f"  (expected {want!r})"))
    if not ok:
        failures.append(label)


def elements(node):
    """Every element carrying an `origin`, anywhere in the tree."""
    if isinstance(node, dict):
        if "id" in node and "origin" in node:
            yield node
        for value in node.values():
            yield from elements(value)
    elif isinstance(node, list):
        for value in node:
            yield from elements(value)


def fitted_extent(sw, sh, bw, bh, mode):
    """ADR-0015's contain/cover rule, in exact integer arithmetic."""
    width_drives = (bw * sh <= bh * sw) if mode == "contain" else (bw * sh >= bh * sw)
    if width_drives:
        return bw, (sh * bw) // sw
    return (sw * bh) // sh, bh


def main():
    els = list(elements(json.loads(FIXTURE.read_text())))
    by_id = {e["id"]: e for e in els}

    print("\n== the badge case: contain leaves slack, and `origin` spends it ==")
    # #52's worked example. Aperture 200x160, source 1200x400.
    w, h = fitted_extent(1200, 400, 200, 160, "contain")
    check("fitted extent of 1200x400 into 200x160", (w, h), (200, 66))
    check("vertical slack", 160 - h, 94)
    # The hand-computed spelling consumes the derived extent; the origin spelling does not.
    check("top-left spelling y = clip.y + (clip.h - h)//2", 1700 + (160 - h) // 2, 1747)
    check("center  spelling y = clip.y + clip.h//2", 1700 + 160 // 2, 1780)
    check("both spellings denote one rect (top edge)", 1780 - h // 2, 1747)

    print("\n== the idiom, committed: badge-idiom-example.montaget.json ==")
    # #74 item 4: the real fixture is all `cover`, so this idiom was unexercised by any
    # committed file. This is a synthetic single-element project, not production evidence --
    # it exists only so a regression in the arithmetic fails a check instead of passing one.
    example = json.loads(IDIOM_EXAMPLE.read_text())
    badge = next(elements(example))
    check("committed element uses contain", badge["fit"], "contain")
    check("committed element's clip", tuple(badge["clip"]), (840, 1700, 200, 160))
    w, h = fitted_extent(1200, 400, 200, 160, "contain")
    check("committed width/height match the derived extent", (badge["width"], badge["height"]), (w, h))
    check("committed origin is center", badge["origin"], "center")
    check("committed x follows the idiom (clip.x + clip.w//2)", badge["x"], 840 + 200 // 2)
    check("committed y follows the idiom (clip.y + clip.h//2)", badge["y"], 1700 + 160 // 2)
    check("committed y is not the hand-computed spelling", badge["y"] != 1700 + (160 - h) // 2, True)

    print("\n== the >=0.5px residue: parity, not a decision ==")
    # Corner-anchoring is exact iff the slack is even; centre-anchoring iff the box is.
    for (bw, bh), exact_corner, exact_centre in [((200, 160), True, True),
                                                 ((201, 161), True, False),
                                                 ((200, 161), False, False)]:
        fw, fh = fitted_extent(1200, 400, bw, bh, "contain")
        check(f"clip {bw}x{bh}: corner spelling exact", (bh - fh) % 2 == 0, exact_corner)
        check(f"clip {bw}x{bh}: centre spelling exact", bh % 2 == 0, exact_centre)

    print("\n== FALSIFIED: 'the fixture proves a floor rule for origin resolution' ==")
    odd_centre = [e for e in els if e["origin"] == "center" and e.get("height", 0) % 2 == 1]
    check("odd-height center-origin elements", len(odd_centre), 10)
    check("...all of type text", {e["type"] for e in odd_centre}, {"text"})
    check("...none carries fit or clip", any("fit" in e or "clip" in e for e in odd_centre), False)
    # 10 elements, but only 4 distinct geometries -- ADR-0013's own deflation.
    geoms = Counter((e["y"], e["height"]) for e in odd_centre)
    check("distinct (y,height) geometries among them", len(geoms), 4)
    # And only one of the four has a top-left twin to check against.
    tops = {(e["y"], e.get("height")) for e in els if e["origin"] == "top-left"}
    corroborated = [g for g in geoms if (g[0] - g[1] // 2, g[1]) in tops]
    check("...geometries with a corroborating top-left twin", len(corroborated), 1)
    # No element anywhere needs an odd dimension halved.
    check("non-text elements using a center origin",
          [e["id"] for e in els if e["origin"].startswith("center") and e["type"] != "text"], [])
    check("odd widths anywhere in the fixture",
          [e["id"] for e in els if e.get("width", 0) % 2 == 1], [])

    print("\n== FALSIFIED: 'where the text block sits inside its box is undefined' ==")
    # ADR-0007 places the *block*, by origin. The declared box is never an input.
    s05 = by_id["sentence-05"]
    block = s05["size"] * s05["line_height"]
    check("sentence-05 block height (55 x 1.1)", round(block, 10), 60.5)
    check("block top, centred on y (ADR-0007's worked example)", s05["y"] - block / 2, 1506.75)
    check("block bottom", s05["y"] + block / 2, 1567.25)
    # The rule proposed during the investigation describes the undrawn box, not the block.
    check("proposed `y - height//2` (the BOX, not drawn)", s05["y"] - s05["height"] // 2, 1453)
    check("...which is card-05's y -- a container-claim coincidence", by_id["card-05"]["y"], 1453)
    check("...distance from the drawn block top", round(s05["y"] - block / 2 - 1453, 2), 53.75)

    print("\n== FALSIFIED: ADR-0006's 'sentence-quiz overhangs its card by ~6 px' ==")
    sq, cq = by_id["sentence-quiz"], by_id["card-quiz"]
    # ADR-0008: no automatic wrapping; `\n` is the only break. This element has none.
    lines = 1 + sum(r.get("text", "").count("\n") for r in sq.get("runs", []))
    check("sentence-quiz line count", lines, 1)
    sq_block = lines * sq["size"] * sq["line_height"]
    overhang = cq["y"] - (sq["y"] - sq_block / 2)
    check("card top minus block top (negative == inside)", round(overhang, 2), -53.75)
    check("...so there is no overhang", overhang > 0, False)

    print("\n== OPEN: text block arithmetic has no stated numeric domain ==")
    # line_height is the format's only non-integer field. ADR-0013 banned floats from
    # the fit arithmetic over a 4.466% divergence; no equivalent clause exists here.
    def exact_ceil(size, n, lh="1.1"):
        return int((Decimal(size) * Decimal(lh) * n).to_integral_value(rounding=ROUND_CEILING))

    diverge = [(size, n) for size in range(1, 400) for n in (1, 2, 3)
               if math.ceil(size * 1.1 * n) != exact_ceil(size, n)]
    check("float/exact ceil divergence over sizes 1-399 x 1-3 lines", len(diverge), 76)
    check("...as a percentage", f"{len(diverge) / 1197 * 100:.2f}%", "6.35%")
    # The fixture escapes entirely -- exactly how ADR-0013 describes cover escaping its ULP bug.
    derived = [(e["size"], 1) for e in els
               if e["type"] == "text" and e.get("height") == math.ceil(e["size"] * e["line_height"])]
    check("fixture (size, lines) pairs that diverge",
          [p for p in derived if p in diverge], [])
    # But it is one edit away: add a \n to sentence-05 and the derived height forks.
    check("sentence-05 + one \\n: float ceil", math.ceil(55 * 1.1 * 2), 122)
    check("sentence-05 + one \\n: exact ceil", exact_ceil(55, 2), 121)

    print()
    if failures:
        print(f"FAILED: {len(failures)} check(s) no longer reproduce:")
        for f in failures:
            print(f"  - {f}")
        return 1
    print("All checks reproduce.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
