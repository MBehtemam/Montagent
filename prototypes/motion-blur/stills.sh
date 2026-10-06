#!/bin/sh
# PROTOTYPE #718 — paint the stills the owner judges, blurred and sharp, at full scale PNG.
# Usage: prototypes/motion-blur/stills.sh <montagent binary>
set -e
BIN="$1"
cd "$(dirname "$0")"
mkdir -p out/stills
for t in 300 400 1700 3600 3700 6100 6200 9500 12400 12500 7500; do
  "$BIN" frame motion-blur.montagent.json --at "$t" --full --png --out "out/stills/blur-$t.png" >/dev/null
  "$BIN" frame no-blur.montagent.json --at "$t" --full --png --out "out/stills/sharp-$t.png" >/dev/null
done
ls out/stills
