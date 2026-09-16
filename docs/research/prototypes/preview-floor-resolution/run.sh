#!/usr/bin/env bash
# PROTOTYPE — throwaway. Resolves #117: pick preview's hard-refuse floor resolution.
#
# No production Montaget renderer exists yet, so this prototype uses the fixture's own
# published reference render (fixtures/en-halloween-decorating/reference/en-halloween-decorating.mp4)
# as ground truth for the exact project — same 1080x1920 frame, same timeline, same text —
# and simulates a proxy-resolution preview by downscaling native frames to each candidate
# floor with bilinear filtering (the resampling ADR-0009/#7/#34 identified as the dominant
# per-frame cost, and the realistic behaviour of a real-time scaler), then scaling back up
# to native size with the same filter so every tier is judged at equal viewing size — the
# way a preview player actually displays a proxy frame in a fixed-size viewport.
#
# Resolution tiers (long-edge-capped, ADR-0046's rule, applied to the fixture's 1080x1920):
#   native  1080x1920  (reference)
#   720p    720x1280   (ADR-0046's adopted proxy-preview target — not a floor candidate)
#   540p    540x960
#   360p    360x640
#   240p    240x426
set -euo pipefail
cd "$(dirname "$0")"

SRC=../../../../fixtures/en-halloween-decorating/reference/en-halloween-decorating.mp4
FRAMES=frames
CROPS=crops
mkdir -p "$FRAMES" "$CROPS"

# timestamp (seconds) -> label. Chosen to stress the fixture's smallest text and densest
# compositions (sizes from en-halloween-decorating.montaget.json):
#   t=0.5   intro-title (58px) + chip-text (52px) + handle-text (34px, on screen throughout)
#   t=11.5  sentence-05 (55px) — matches the fixture's existing frame-05-at-11s.png reference
#   t=45.0  word-08-target (49px) + word-08-bridge (35px, the smallest caption text in the file)
#   t=62.0  sentence-quiz (55px)
declare -A TIMES=( [intro]=0.5 [sentence-05]=11.5 [word-08]=45.0 [quiz]=62.0 )

# tier -> WxH (long-edge cap, even dims)
declare -A TIERS=( [native]=1080x1920 [720p]=720x1280 [540p]=540x960 [360p]=360x640 [240p]=240x426 )
TIER_ORDER=(native 720p 540p 360p 240p)

# label -> crop "x,y,w,h" in native-frame coordinates, padded around the element's declared box
declare -A CROPS_BY_LABEL=(
  [intro]="500,90,560,180"
  [sentence-05]="500,1510,1024,220"
  [word-08]="480,1330,1064,150"
  [quiz]="500,1510,1024,220"
)

for label in "${!TIMES[@]}"; do
  t="${TIMES[$label]}"
  native_png="$FRAMES/${label}-native.png"
  ffmpeg -y -v error -ss "$t" -i "$SRC" -frames:v 1 "$native_png"

  for tier in "${TIER_ORDER[@]}"; do
    [ "$tier" = "native" ] && continue
    wh="${TIERS[$tier]}"
    down="$FRAMES/${label}-${tier}-down.png"
    upback="$FRAMES/${label}-${tier}.png"
    # downscale to the tier's real pixel budget, bilinear
    ffmpeg -y -v error -i "$native_png" -vf "scale=${wh//x/:}:flags=bilinear" "$down"
    # scale back to native size for equal-size side-by-side viewing, same filter
    ffmpeg -y -v error -i "$down" -vf "scale=1080:1920:flags=bilinear" "$upback"
  done

  # crop each tier (including native) at the label's region, 3x nearest-neighbor zoom for legibility
  IFS=',' read -r cx cy cw ch <<< "${CROPS_BY_LABEL[$label]}"
  for tier in "${TIER_ORDER[@]}"; do
    src_png="$FRAMES/${label}-${tier}.png"
    [ "$tier" = "native" ] && src_png="$native_png"
    ffmpeg -y -v error -i "$src_png" -vf "crop=${cw}:${ch}:${cx}:${cy},scale=$((cw*3)):$((ch*3)):flags=neighbor" "$CROPS/${label}-${tier}-crop.png"
  done

  # a side-by-side strip of the five equal-size crops, in tier order
  inputs=()
  for tier in "${TIER_ORDER[@]}"; do
    inputs+=(-i "$CROPS/${label}-${tier}-crop.png")
  done
  n=${#TIER_ORDER[@]}
  ffmpeg -y -v error "${inputs[@]}" -filter_complex "hstack=inputs=${n}" "$CROPS/${label}-strip.png"
done

echo "Frames in $FRAMES/, crops and comparison strips in $CROPS/"
