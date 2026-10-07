#!/bin/bash
# PROTOTYPE #764: paint the clip at 1 to 10 painters over several paint-chunk sizes (ADR-0144),
# with the blur/shadow bounds hint of #652 on and off, hashing every frame's raw RGB before the
# encoder. Every run is compared to the one-painter, hint-on baseline. Each run also logs, from
# inside the painter, the letters of every text-on-a-path whose glyphs it did not hand to the
# canvas (MONTAGENT_PROTO_HIDDEN), for hidden.py to compare against `query --at`.
# Usage: sweep.sh <project> <out-dir>
set -u
cd "$(dirname "$0")"
B=${MONTAGENT_BIN:-montagent}
project=$1
dir=$2
mkdir -p "$dir"; dir=$(cd "$dir" && pwd)
rm -f "$dir"/*.raw "$dir"/*.err "$dir"/*.mp4 "$dir"/*.hidden
for bound in bounded unbounded; do
  for kc in 1,1000 2,5 3,2 4,1 5,4 6,9 7,3 8,3 9,2 10,7 default; do
    f=$dir/$bound-${kc/,/x}
    env_args=(MONTAGENT_PROTO_HASHES="$f.raw" MONTAGENT_PROTO_COUNT=1 MONTAGENT_PROTO_HIDDEN="$f.hidden")
    [ "$bound" = unbounded ] && env_args+=(MONTAGENT_PROTO_UNBOUND=1)
    [ "$kc" != default ] && env_args+=(MONTAGENT_PAINTING="$kc")
    env "${env_args[@]}" "$B" render "$project" --output "$f.mp4" >/dev/null 2>"$f.err" || echo "FAIL $f"
    rm -f "$f.mp4"
    sort -k1,1n -k2,2 "$f.hidden" -o "$f.hidden"
    same=$(cmp -s "$f.raw" "$dir/bounded-1x1000.raw" && echo SAME || echo DIFFERS)
    hsame=$(cmp -s "$f.hidden" "$dir/bounded-1x1000.hidden" && echo SAME || echo DIFFERS)
    printf "%-34s frames=%-4s hinted-layers=%-6s frames:%-8s hidden-log:%s\n" "$(basename "$f")" \
      "$(wc -l < "$f.raw" | tr -d ' ')" "$(grep -c '^proto-hint' "$f.err")" "$same" "$hsame"
    grep -v '^proto-hint' "$f.err" > "$f.err.tmp"; mv "$f.err.tmp" "$f.err"
  done
done
echo "distinct frame-hash sequences: $(shasum "$dir"/*.raw | awk '{print $1}' | sort -u | wc -l | tr -d ' ')"
echo "distinct hidden logs: $(shasum "$dir"/*.hidden | awk '{print $1}' | sort -u | wc -l | tr -d ' ')"
