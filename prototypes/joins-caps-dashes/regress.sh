#!/bin/bash
# PROTOTYPE #750: every committed fixture with a `rect`, `ellipse` or `path` (none of them uses
# a new field), rendered by main (plus only the hashing probe: $BASELINE_BIN) and by this
# prototype ($MONTAGENT_BIN), comparing the raw RGB at the encoder's input frame by frame.
set -u
cd "$(dirname "$0")"
T=${PROTO_SCRATCH:-/tmp}/jc750-regress
mkdir -p "$T"
for p in \
  ../../fixtures/path/paths.montagent.json \
  ../../fixtures/animatable-shape/morph.montagent.json \
  ../../fixtures/keyframe-coincidence/keyframe-coincidence.montagent.json \
  ../../fixtures/keyframe-cross-boundary/keyframe-cross-boundary.montagent.json \
  ../../fixtures/unpainted-visual-state/unpainted-visual-state.montagent.json \
  ../../fixtures/long-state/long-state.montagent.json \
  ../../fixtures/en-halloween-decorating/en-halloween-decorating.montagent.json \
  ../../fixtures/video-decode/video-decode.montagent.json; do
  n=$(basename "$p" .montagent.json)
  for side in main proto; do
    bin=$BASELINE_BIN; [ $side = proto ] && bin=$MONTAGENT_BIN
    rm -f "$T/$n-$side.raw"
    MONTAGENT_PROTO_HASHES="$T/$n-$side.raw" "$bin" render "$p" --output "$T/$n-$side.mp4" \
      >/dev/null 2>"$T/$n-$side.err" || echo "render exited non-zero: $n ($side)"
    rm -f "$T/$n-$side.mp4"
  done
  frames=$(wc -l < "$T/$n-main.raw" 2>/dev/null | tr -d ' ')
  same=$(cmp -s "$T/$n-main.raw" "$T/$n-proto.raw" && echo SAME || echo DIFFERS)
  printf "%-28s frames=%-5s %s\n" "$n" "${frames:-0}" "$same"
done
