#!/bin/bash
# PROTOTYPE #764: full-scale PNGs at the telling instants of each scene.
set -eu
cd "$(dirname "$0")"
B=${MONTAGENT_BIN:-montagent}
mkdir -p out/stills
for t in ${STILLS:-500 1000 1900 3000 3600 6000 8000 8300 9000 12000 12500 13000 14000 15500 18000 20200 20600 22000 23900}; do
  "$B" frame textpath.montagent.json --at "$t" --full --png --out "out/stills/at-$t.png" >/dev/null
done
"$B" frame reverse/reverse.json --at 0 --full --png --out out/stills/reverse.png >/dev/null
