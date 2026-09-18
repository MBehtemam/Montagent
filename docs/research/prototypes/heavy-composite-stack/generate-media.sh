#!/usr/bin/env bash
# Synthetic 4K/8K test assets for #159's heavy-composite-stack harness --
# NOT real footage. testsrc2 stills and video, generated on demand rather
# than committed (see FINDINGS.md "What's committed vs regenerated").
# Reuses #87's naming convention; a-* are the "main" assets (same content
# as #87's, regenerated independently so this prototype has no file
# dependency on ../proxy-preview-savings), b-* are the second simultaneous
# still/clip that makes the stack "heavy".
set -e
cd "$(dirname "$0")"
mkdir -p media

gen_still() {
  local out=$1 size=$2 seed=$3
  if [ ! -f "$out" ]; then
    echo "generating $out ($size, synthetic testsrc2, seed=$seed)"
    ffmpeg -y -hide_banner -loglevel error -f lavfi -i "testsrc2=size=${size}" \
      -vf "hue=h=${seed}" -frames:v 1 "$out"
  fi
}

gen_video() {
  local out=$1 size=$2 dur=$3 seed=$4
  if [ ! -f "$out" ]; then
    echo "generating $out (${size}/30, ${dur}s, synthetic testsrc2, seed=$seed)"
    ffmpeg -y -hide_banner -loglevel error -f lavfi -i "testsrc2=size=${size}:rate=30:duration=${dur}" \
      -vf "hue=h=${seed}" \
      -c:v libx264 -preset veryfast -crf 30 -pix_fmt yuv420p "$out"
  fi
}

# main (a-*): full-card scale, same dims #87 used
gen_still media/a-4k-still.png 2160x2600 0
gen_still media/a-8k-still.png 4320x5200 0
gen_video media/a-4k-video.mp4 2160x3840 25 0
gen_video media/a-8k-video.mp4 4320x7680 25 0

# second simultaneous still/clip (b-*): square PiP-inset scale, different
# hue so correctness frames visibly show two distinct sources compositing
gen_still media/b-4k-still.png 800x800 120
gen_still media/b-8k-still.png 1600x1600 120
gen_video media/b-4k-video.mp4 800x800 25 120
gen_video media/b-8k-video.mp4 1600x1600 25 120

echo "media ready:"
ls -la media
