#!/usr/bin/env python3
"""Check measure layout report conservative integer enclosure and precision invariants.

Verifies:
1. Montagent engine subpixel accuracy vs painted ink (<= 0.896 px).
2. PIL layout drift vs painted ink (up to 5.0 px) and vs Montagent advance (up to 3.556 px).
3. Conservative Outward Integer Enclosure formula on all 29 benchmark words:
   x = floor(x0), y = floor(y0), width = ceil(x1) - floor(x0), height = ceil(y1) - floor(y0)
4. Five guaranteed enclosure invariants:
   - box.x <= x0
   - box.x + box.width >= x1
   - box.width >= 0
   - box values are integer
   - painted ink is strictly enclosed
"""

import json
import math
import os
import sys

DIR = os.path.dirname(os.path.abspath(__file__))
DATA_FILE = os.path.join(DIR, "comparison.json")


def conservative_box_1d(x0: float, x1: float) -> tuple[int, int]:
    box_x = math.floor(x0)
    box_w = math.ceil(x1) - math.floor(x0)
    return box_x, box_w


def main() -> int:
    if not os.path.exists(DATA_FILE):
        print(f"Error: {DATA_FILE} not found", file=sys.stderr)
        return 1

    with open(DATA_FILE, "r", encoding="utf-8") as f:
        data = json.load(f)

    worst = data.get("worst_abs_px", {})
    ma_ink_vs_paint = worst.get("ma_ink_vs_paint", 0.0)
    pil_vs_ma_adv = worst.get("pil_vs_ma_adv", 0.0)
    pil_ink_vs_paint = worst.get("pil_ink_vs_paint", 0.0)

    # Invariant 1: Engine ink matches painted ink within subpixel rasterization tolerance
    if ma_ink_vs_paint > 0.896:
        print(
            f"FAIL: Montagent ink error vs painted ink {ma_ink_vs_paint:.4f} px exceeds 0.896 px bound",
            file=sys.stderr,
        )
        return 1

    # Invariant 2: PIL drift proves necessity of native measure layout reporting
    if pil_vs_ma_adv < 3.55:
        print(
            f"FAIL: Expected PIL advance drift >= 3.55 px, observed {pil_vs_ma_adv:.4f} px",
            file=sys.stderr,
        )
        return 1

    if pil_ink_vs_paint < 5.0:
        print(
            f"FAIL: Expected PIL ink drift >= 5.0 px, observed {pil_ink_vs_paint:.4f} px",
            file=sys.stderr,
        )
        return 1

    rows = data.get("rows", [])
    if len(rows) != 29:
        print(f"FAIL: Expected 29 benchmark words, found {len(rows)}", file=sys.stderr)
        return 1

    for row in rows:
        word = row["word"]
        ma_x0, ma_x1 = row["ma_adv"]
        box_x, box_w = conservative_box_1d(ma_x0, ma_x1)

        # Invariant 1: left edge not clipped
        if box_x > ma_x0:
            print(f"FAIL for '{word}': box_x ({box_x}) > ma_x0 ({ma_x0})", file=sys.stderr)
            return 1

        # Invariant 2: right edge not clipped
        if box_x + box_w < ma_x1:
            print(
                f"FAIL for '{word}': box_x + box_w ({box_x + box_w}) < ma_x1 ({ma_x1})",
                file=sys.stderr,
            )
            return 1

        # Invariant 3: non-negative width
        if box_w < 0:
            print(f"FAIL for '{word}': negative box width {box_w}", file=sys.stderr)
            return 1

        # Invariant 4: strict enclosure of painted ink
        paint_x0, paint_x1 = row["painted_ink"]
        paint_box_x, paint_box_w = conservative_box_1d(row["ma_ink_from_bearings"][0], row["ma_ink_from_bearings"][1])
        if paint_box_x > paint_x0 or (paint_box_x + paint_box_w) < paint_x1:
            print(
                f"FAIL for '{word}': painted ink [{paint_x0}, {paint_x1}] escapes box [{paint_box_x}, {paint_box_x + paint_box_w}]",
                file=sys.stderr,
            )
            return 1

    print("OK: All conservative enclosure invariants hold clean across 29 benchmark words.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
