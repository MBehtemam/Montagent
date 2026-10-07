#!/bin/bash
# PROTOTYPE #764: text with no `path`, rendered by main (plus only the hashing probe:
# $BASELINE_BIN) and by this prototype ($MONTAGENT_BIN), comparing the raw RGB at the encoder's
# input frame by frame. The committed fixtures with a text element, the #750 set of shape
# fixtures (the derived animatable list gained `path.points`), and plain.json: this clip with
# every `path` and `path_offset` dropped, which puts Arabic, a mixed-direction line, mixed run
# sizes, a text stroke, letter staggers and shadowed text through both binaries.
# Left out: examples/hello-text (its `fonts/` is not committed, so neither binary renders it)
# and the spy-trailer benchmark (its score.wav is generated, not committed).
set -u
cd "$(dirname "$0")"
T=${PROTO_SCRATCH:-/tmp}/tp764-regress
mkdir -p "$T"
for p in \
  ../../fixtures/en-halloween-decorating/en-halloween-decorating.montagent.json \
  plain.json \
  ../../fixtures/path/paths.montagent.json \
  ../../fixtures/animatable-shape/morph.montagent.json \
  ../../fixtures/keyframe-coincidence/keyframe-coincidence.montagent.json \
  ../../fixtures/keyframe-cross-boundary/keyframe-cross-boundary.montagent.json \
  ../../fixtures/unpainted-visual-state/unpainted-visual-state.montagent.json \
  ../../fixtures/long-state/long-state.montagent.json \
  ../../fixtures/video-decode/video-decode.montagent.json; do
  n=$(basename "$p" .montagent.json)
  n=${n%.json}
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
