#!/bin/bash
# PROTOTYPE #764: full-scale PNGs at the telling instants of each scene.
set -eu
cd "$(dirname "$0")"
B=${MONTAGENT_BIN:-montagent}
mkdir -p out/stills
for t in ${STILLS:-500 1000 2400 3900 6000 8300 9000 12000 13000 14000 18000 20300 21000 23900}; do
  "$B" frame textpath.montagent.json --at "$t" --full --png --out "out/stills/at-$t.png" >/dev/null
done
"$B" frame reverse/reverse.json --at 0 --full --png --out out/stills/reverse.png >/dev/null
