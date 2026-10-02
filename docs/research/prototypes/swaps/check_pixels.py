"""Pixel-equivalence between two spellings of one video (#614's done-check).

    python3 check_pixels.py MONTAGENT A.montagent.json B.montagent.json FROM TO [OUT_DIR]

Draws every painted frame in `[FROM, TO)` — instant ⌊n × 1000 / fps⌋ — from both files
with `montagent frame --full --png`, and compares the decoded pixels exactly. Prints one
line per frame that differs (its instant and how many pixels), then a summary. Exit 0
only when every frame is identical.
"""

import json
import pathlib
import subprocess
import sys
import tempfile
from concurrent.futures import ThreadPoolExecutor

from PIL import Image


def painted(fps, start, end):
    n = -(-start * fps // 1000)  # the first frame at or after `start`
    while (instant := n * 1000 // fps) < end:
        yield instant
        n += 1


def draw(montagent, project, instant, out):
    subprocess.run(
        [montagent, "frame", str(project), "--at", str(instant), "--full", "--png",
         "--out", str(out)],
        check=False, capture_output=True,
    )
    return Image.open(out).convert("RGBA")


def main():
    montagent, a, b, start, end = sys.argv[1:6]
    a, b = pathlib.Path(a), pathlib.Path(b)
    out = pathlib.Path(sys.argv[6]) if len(sys.argv) > 6 else pathlib.Path(tempfile.mkdtemp())
    out.mkdir(parents=True, exist_ok=True)
    fps = json.loads(a.read_text())["fps"]
    instants = list(painted(fps, int(start), int(end)))

    def compare(instant):
        left = draw(montagent, a, instant, out / f"a-{instant}.png")
        right = draw(montagent, b, instant, out / f"b-{instant}.png")
        if left.size != right.size:
            return instant, -1
        # Per pixel, over all four channels. Not `difference(...).getbbox()`: on an RGBA
        # image `getbbox` reads the alpha band alone, so two opaque frames of different
        # colours would compare equal.
        differing = sum(1 for p, q in zip(left.getdata(), right.getdata()) if p != q)
        return instant, differing

    with ThreadPoolExecutor(max_workers=8) as pool:
        results = sorted(pool.map(compare, instants))

    differing = [(t, n) for t, n in results if n]
    for t, n in differing:
        print(f"{t} ms: {'size differs' if n < 0 else f'{n} pixels differ'}")
    print(f"{len(results) - len(differing)} of {len(results)} painted frames identical "
          f"in [{start}, {end}) at {fps} fps")
    sys.exit(1 if differing else 0)


if __name__ == "__main__":
    main()
