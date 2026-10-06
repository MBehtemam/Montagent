#!/bin/bash
# PROTOTYPE #722: one full-scale PNG per scene of the clip, through `frame`.
set -u
cd "$(dirname "$0")"
B=${MONTAGENT_BIN:-montagent}
mkdir -p out/stills
for at in 1500 4500 7500 10500 13500 16500; do
  "$B" frame effects.montagent.json --at "$at" --full --png --out "out/stills/at-$at.png" \
    >/dev/null 2>"out/stills/at-$at.err" || { echo "FAIL $at"; tail -5 "out/stills/at-$at.err"; }
done
ls out/stills
