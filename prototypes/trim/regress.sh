#!/bin/bash
# PROTOTYPE #760: projects with `rect`, `ellipse` or `path` and no trim field, rendered by the
# base branch's tip (prototype-joins-caps-dashes-750 at fd573eed: $BASELINE_BIN) and by this
# prototype ($MONTAGENT_BIN), comparing the raw RGB at the encoder's input frame by frame.
# The committed fixtures #750 used, plus #750's own clip (joins, caps and dashes, untrimmed).
set -u
cd "$(dirname "$0")"
T=${PROTO_SCRATCH:-/tmp}/trim760-regress
mkdir -p "$T"
for p in \
  ../../fixtures/path/paths.montagent.json \
  ../../fixtures/animatable-shape/morph.montagent.json \
  ../../fixtures/keyframe-coincidence/keyframe-coincidence.montagent.json \
  ../../fixtures/keyframe-cross-boundary/keyframe-cross-boundary.montagent.json \
  ../../fixtures/unpainted-visual-state/unpainted-visual-state.montagent.json \
  ../../fixtures/long-state/long-state.montagent.json \
  ../../fixtures/en-halloween-decorating/en-halloween-decorating.montagent.json \
  ../../fixtures/video-decode/video-decode.montagent.json \
  ../joins-caps-dashes/strokes.montagent.json \
  ../joins-caps-dashes/gap0-dashed.json; do
  n=$(basename "$p" .json)
  n=${n%.montagent}
  for side in base proto; do
    bin=$BASELINE_BIN; [ $side = proto ] && bin=$MONTAGENT_BIN
    rm -f "$T/$n-$side.raw"
    MONTAGENT_PROTO_HASHES="$T/$n-$side.raw" "$bin" render "$p" --output "$T/$n-$side.mp4" \
      >/dev/null 2>"$T/$n-$side.err" || echo "render exited non-zero: $n ($side)"
    rm -f "$T/$n-$side.mp4"
  done
  frames=$(wc -l < "$T/$n-base.raw" 2>/dev/null | tr -d ' ')
  same=$(cmp -s "$T/$n-base.raw" "$T/$n-proto.raw" && echo SAME || echo DIFFERS)
  printf "%-28s frames=%-5s %s\n" "$n" "${frames:-0}" "$same"
done
# `query --at` on untrimmed elements: text and JSON, base against prototype.
for p in ../joins-caps-dashes/query.json ../../fixtures/path/paths.montagent.json; do
  for fmt in text json; do
    args=(query "$p" --at 1500); [ $fmt = json ] && args+=(--json)
    "$BASELINE_BIN" "${args[@]}" > "$T/q-base" 2>&1
    "$MONTAGENT_BIN" "${args[@]}" > "$T/q-proto" 2>&1
    printf "query --at 1500 %-5s %-40s %s\n" "$fmt" "$(basename "$p")" \
      "$(cmp -s "$T/q-base" "$T/q-proto" && echo SAME || echo DIFFERS)"
  done
done
