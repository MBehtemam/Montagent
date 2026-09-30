#!/bin/sh
# Build deliverable.mp4: repair the rig, render frames, encode, mix the voice at 1 s.
# PY must be a Python with Pillow and numpy.
set -e
cd "$(dirname "$0")/.."
PY=${PY:-python3}
TMP=${TMPDIR:-/tmp}

$PY build/repair.py
$PY build/render.py | ffmpeg -y -loglevel error -f rawvideo -pix_fmt rgb24 -s 1920x1080 -r 30 -i - \
  -c:v libx264 -preset slow -crf 16 -tune animation -pix_fmt yuv420p "$TMP/hoot-video.mp4"

ffmpeg -y -loglevel error -i "$TMP/hoot-video.mp4" -i character/voice/line-1.wav -i music/bed-120bpm.wav \
  -filter_complex "[1:a]aresample=48000,pan=stereo|c0=c0|c1=c0,volume=4dB,adelay=1000|1000,apad[v];\
[2:a]atrim=0:8,asetpts=PTS-STARTPTS,volume=-14dB,afade=t=in:st=0:d=0.3,afade=t=out:st=7.1:d=0.9[m];\
[v][m]amix=inputs=2:duration=longest:normalize=0,atrim=0:8,alimiter=limit=0.89:level=false[a]" \
  -map 0:v -map "[a]" -c:v copy -c:a aac -b:a 192k -ar 48000 -t 8 -movflags +faststart deliverable.mp4
echo "wrote deliverable.mp4"
