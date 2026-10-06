#!/bin/bash
# PROTOTYPE #722: what Skia makes of each member's filter: transparent black through every
# colour filter, each filter's fast bounds, and the bounds hint each layer gets.
set -u
cd "$(dirname "$0")"
B=${MONTAGENT_BIN:-montagent}
tmp=$(mktemp -d)
{
  for at in 10500 13500 16500; do
    MONTAGENT_PROTO_PROBE_TB=1 MONTAGENT_PROTO_COUNT=1 "$B" frame effects.montagent.json --at "$at" \
      --out "$tmp/f.jpg" 2>&1 >/dev/null | grep -E '^proto-(tb|hint)'
  done
  MONTAGENT_PROTO_PROBE_TB=1 MONTAGENT_PROTO_COUNT=1 MONTAGENT_PROTO_COLOUR_BOUND=1 "$B" frame \
    colour-bound.json --at 1000 --out "$tmp/f.jpg" 2>&1 >/dev/null | grep -E '^proto-(tb|hint)'
} | sort | uniq -c
MONTAGENT_PROTO_PROBE_INT=1 "$B" frame shift/shift-0.json --at 0 --out "$tmp/f.jpg" 2>&1 >/dev/null \
  | grep proto-probe-int | sort -u
for case in reach/dblur-45-40-on.json reach/glow-r40-on.json; do
  echo "compiles, $case at K=3, C=1:"
  MONTAGENT_PROTO_COMPILE=1 MONTAGENT_PAINTING=3,1 "$B" render "$case" --output "$tmp/c-${case////-}.mp4" \
    2>&1 >/dev/null | grep proto-compile
done
rm -rf "$tmp"
