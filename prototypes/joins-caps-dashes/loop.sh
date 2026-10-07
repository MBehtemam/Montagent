#!/bin/bash
# PROTOTYPE #750: the ants at their first key (0 ms), 10 ms past half way (1510, offset −120.8, so not a whole pattern) and their last key (3000),
# offset 0 -> -240 / +240 with a 40 px pattern. 0 and 3000 should be the same bytes.
set -eu
cd "$(dirname "$0")"
B=${MONTAGENT_BIN:-montagent}
T=${PROTO_SCRATCH:-/tmp}
for t in 0 1510 3000; do
  "$B" frame loop.json --at "$t" --full --png --out "$T/loop-$t.png" >/dev/null
done
{
  for t in 0 1510 3000; do
    echo "frame --at $t: $(shasum "$T/loop-$t.png" | cut -c1-16)"
  done
  cmp -s "$T/loop-0.png" "$T/loop-3000.png" && echo "0 ms and 3000 ms: SAME" || echo "0 ms and 3000 ms: DIFFER"
} | tee out/loop.txt
