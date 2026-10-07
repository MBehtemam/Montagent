#!/bin/bash
# PROTOTYPE #760: full-scale PNGs at the telling instants of each scene.
#   0      draw-on, first frame (must show no dot)     1250  draw-on half way
#   4050   loaders, the window straddling the start point (offset 0.875 turns)
#   4800   loaders, mid-turn
#   8000   square cap, octagon and diamond              9800  diamond drawing off
#   11500  crossing, before    12500 empty between the crossings    13500  after
#   15300  overshoot, trim_end raw past 1 (clamped)
#   18250  dashes half drawn on, over their untrimmed ghosts
#   21500  full window under an offset
set -eu
cd "$(dirname "$0")"
B=${MONTAGENT_BIN:-montagent}
mkdir -p out/stills
for t in 0 1250 4050 4800 8000 9800 11500 12500 13500 15300 18250 21500; do
  "$B" frame trim.montagent.json --at "$t" --full --png --out "out/stills/at-$t.png" >/dev/null
done
