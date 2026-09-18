#!/usr/bin/env bash
# Synthetic 4K/8K test assets for #87 -- NOT real footage. testsrc2 stills and
# video, generated on demand rather than committed (the 8K video alone is
# ~80 MB; see FINDINGS.md "What's committed vs regenerated").
set -e
cd "$(dirname "$0")"
mkdir -p media

if [ ! -f media/4k-still.png ]; then
  echo "generating media/4k-still.png (2160x2600, synthetic testsrc2)"
  ffmpeg -y -hide_banner -loglevel error -f lavfi -i "testsrc2=size=2160x2600" -frames:v 1 media/4k-still.png
fi

if [ ! -f media/8k-still.png ]; then
  echo "generating media/8k-still.png (4320x5200, synthetic testsrc2)"
  ffmpeg -y -hide_banner -loglevel error -f lavfi -i "testsrc2=size=4320x5200" -frames:v 1 media/8k-still.png
fi

if [ ! -f media/4k-video.mp4 ]; then
  echo "generating media/4k-video.mp4 (2160x3840/30, 25s, synthetic testsrc2)"
  ffmpeg -y -hide_banner -loglevel error -f lavfi -i "testsrc2=size=2160x3840:rate=30:duration=25" \
    -c:v libx264 -preset veryfast -crf 30 -pix_fmt yuv420p media/4k-video.mp4
fi

if [ ! -f media/8k-video.mp4 ]; then
  echo "generating media/8k-video.mp4 (4320x7680/30, 25s, synthetic testsrc2)"
  ffmpeg -y -hide_banner -loglevel error -f lavfi -i "testsrc2=size=4320x7680:rate=30:duration=25" \
    -c:v libx264 -preset veryfast -crf 30 -pix_fmt yuv420p media/8k-video.mp4
fi

echo "media ready:"
ls -la media
