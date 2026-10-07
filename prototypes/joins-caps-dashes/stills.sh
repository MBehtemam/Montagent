#!/bin/bash
# PROTOTYPE #750: one full-scale PNG per scene, and four of the keyed miter path: its key,
# the frames either side of the keyed corner crossing the limit.
set -eu
cd "$(dirname "$0")"
B=${MONTAGENT_BIN:-montagent}
mkdir -p out/stills
for t in 1500 4500 6000 7433 7467 8000 11500 14500 17500 19500 20500; do
  "$B" frame strokes.montagent.json --at "$t" --full --png --out "out/stills/at-$t.png" >/dev/null
done
