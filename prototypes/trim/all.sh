#!/bin/bash
# PROTOTYPE #760: every measurement, in order, with the binaries named by MONTAGENT_BIN (this
# branch) and BASELINE_BIN (prototype-joins-caps-dashes-750 at fd573eed, built as is).
set -eu
cd "$(dirname "$0")"
export PROTO_SCRATCH=${PROTO_SCRATCH:-/tmp}
python3 make.py
mkdir -p out
"$MONTAGENT_BIN" render trim.montagent.json >/dev/null 2>&1
"$MONTAGENT_BIN" render plain.json --output "$PROTO_SCRATCH/plain.mp4" >/dev/null 2>&1
ffmpeg -loglevel error -y -i out/trim.mp4 -i "$PROTO_SCRATCH/plain.mp4" -filter_complex \
  "[0:v]scale=960:540[a];[1:v]scale=960:540[b];[a][b]hstack=inputs=2" -c:v libx264 \
  -pix_fmt yuv420p -crf 18 out/side-by-side.mp4
./stills.sh
./texts.sh > /dev/null
./fullwin.sh | tee out/fullwin.txt
python3 check.py seam anchor > /dev/null
python3 check.py containment > /dev/null
./regress.sh | tee out/regress.txt
./sweep.sh trim.montagent.json "$PROTO_SCRATCH/sweep760" > out/painters-sweep.txt
./sweep.sh full-trimmed.json "$PROTO_SCRATCH/sweep760-full" > out/painters-sweep-full.txt
./cost.sh | tee out/cost.txt
