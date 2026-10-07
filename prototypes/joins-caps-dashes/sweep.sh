#!/bin/bash
# PROTOTYPE #750: paint a project at 1 to 10 painters over several paint-chunk sizes (ADR-0144),
# with the blur/shadow bounds hint of #652 on and off, hashing every frame's raw RGB before the
# encoder. Every run is compared to the one-painter, hint-on baseline.
# Usage: sweep.sh <project> <out-dir>
set -u
cd "$(dirname "$0")"
B=${MONTAGENT_BIN:-montagent}
project=$1
dir=$2
mkdir -p "$dir"
rm -f "$dir"/*.raw "$dir"/*.err "$dir"/*.mp4
for bound in bounded unbounded; do
  for kc in 1,1000 2,5 3,2 4,1 5,4 6,9 7,3 8,3 9,2 10,7 default; do
    f=$dir/$bound-${kc/,/x}
    env_args=(MONTAGENT_PROTO_HASHES="$PWD/$f.raw" MONTAGENT_PROTO_COUNT=1)
    [ "$bound" = unbounded ] && env_args+=(MONTAGENT_PROTO_UNBOUND=1)
    [ "$kc" != default ] && env_args+=(MONTAGENT_PAINTING="$kc")
    env "${env_args[@]}" "$B" render "$project" --output "$f.mp4" >/dev/null 2>"$f.err" || echo "FAIL $f"
    rm -f "$f.mp4"
    same=$(cmp -s "$f.raw" "$dir/bounded-1x1000.raw" && echo SAME || echo DIFFERS)
    printf "%-34s frames=%-4s hinted-layers=%-6s %s\n" "$f" "$(wc -l < "$f.raw" | tr -d ' ')" \
      "$(grep -c '^proto-hint' "$f.err")" "$same"
    grep -v '^proto-hint' "$f.err" > "$f.err.tmp"; mv "$f.err.tmp" "$f.err"
  done
done
echo "distinct frame-hash sequences: $(shasum "$dir"/*.raw | awk '{print $1}' | sort -u | wc -l | tr -d ' ')"
