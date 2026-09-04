#!/usr/bin/env bash
# Time-to-first-frame for an input seek. #6's one real pathology was cost
# proportional to where the window starts, and it came from decoding.
MP4="$1"
echo "seek  time-to-first-frame (rgba 1080x1300, 3 runs)"
for t in 0 10 20 30 40 50 60; do
  best=99
  for i in 1 2 3; do
    s=$( { /usr/bin/time -p ffmpeg -hide_banner -loglevel error -ss $t -i "$MP4" \
      -vf scale=1080:1300 -frames:v 1 -f rawvideo -pix_fmt rgba - > /dev/null; } 2>&1 | awk '/^real/{print $2}' )
    best=$(python3 -c "print(min($best,$s))")
  done
  echo "  t=${t}s  ${best}s"
done
