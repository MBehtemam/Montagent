#!/bin/bash
# PROTOTYPE #760: the full window under an offset against the untrimmed element. full-trimmed.json
# holds an ellipse (offset keyed 0 to 2.5), a rect (0.37), a dashed rounded rect (0.37), a
# square-capped closed path (0.37) and a dashed closed path (offset keyed -1.25 to 3.5), each
# with trim_start 0 and trim_end 1 and a shadow; full-plain.json is the same with every trim
# field dropped. Raw RGB at the encoder's input, frame by frame, one painter and ten.
set -u
cd "$(dirname "$0")"
B=${MONTAGENT_BIN:-montagent}
T=${PROTO_SCRATCH:-/tmp}/trim760-full
mkdir -p "$T"
for kc in 1,1000 10,7; do
  for p in full-trimmed full-plain; do
    rm -f "$T/$p-$kc.raw"
    MONTAGENT_PAINTING=$kc MONTAGENT_PROTO_HASHES="$T/$p-$kc.raw" "$B" render "$p.json" \
      --output "$T/$p.mp4" >/dev/null 2>&1 || echo "render exited non-zero: $p"
    rm -f "$T/$p.mp4"
  done
  printf "painting %-7s frames=%s  full window vs untrimmed: %s\n" "$kc" \
    "$(wc -l < "$T/full-plain-$kc.raw" | tr -d ' ')" \
    "$(cmp -s "$T/full-trimmed-$kc.raw" "$T/full-plain-$kc.raw" && echo SAME || echo DIFFERS)"
done
