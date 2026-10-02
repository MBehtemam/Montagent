#!/usr/bin/env bash
# Re-measures the encode and decode numbers in FINDINGS.md on this machine.
#
#   bench.sh encode   libx264 presets and VideoToolbox at 1080p30, fed rgb24 over a
#                     pipe the way encode.rs feeds it; size, speed, VMAF, PSNR
#   bench.sh decode   one streaming decoder at 1080p to rgba over a pipe, the way
#                     decode::frames_from runs it; -threads and -hwaccel videotoolbox
#   bench.sh exact    per-frame pts and pixel hashes, software vs -hwaccel videotoolbox,
#                     through frames_from's exact filter chain (ADR-0096)
#
# Each run is short and in the foreground. Needs ffmpeg 7.1+ built with libx264,
# libvmaf and VideoToolbox (Homebrew's formula has all three). Scratch space defaults
# to a temp dir; set WORK to keep the intermediates between runs.
set -euo pipefail
# FFMPEG_DIR picks another build, e.g. Homebrew's ffmpeg@7 for the 7.1 floor (ADR-0115)
if [ -n "${FFMPEG_DIR:-}" ]; then PATH="$FFMPEG_DIR:$PATH"; fi
REPO=$(cd "$(dirname "$0")/../../.." && pwd)
CAM="$REPO/docs/research/skills-eval/assets/presenter/take-1.mp4"   # camera, H.264 Main
SCREEN="$REPO/docs/research/skills-eval/assets/screen/session.mp4"  # screen, H.264 High
WORK=${WORK:-$(mktemp -d)}
mkdir -p "$WORK"
cd "$WORK"
FR=${FR:-150}                # 5 s at 30 fps per encode clip (240 are prepared)
Q=(-hide_banner -nostats -loglevel info)

rtime () { grep -o 'rtime=[0-9.]*' | tail -1 | cut -d= -f2; }

prepare () {
  # decode sources, 20 s at 1080p30
  [ -f dec-cam-crf20.mp4 ] || ffmpeg "${Q[@]}" -y -stream_loop 2 -i "$CAM" -t 20 -r 30 -an -c:v libx264 -preset medium -crf 20 -pix_fmt yuv420p dec-cam-crf20.mp4 2>/dev/null
  [ -f dec-cam-20M.mp4 ] || ffmpeg "${Q[@]}" -y -stream_loop 2 -i "$CAM" -t 20 -r 30 -an -c:v libx264 -preset veryfast -b:v 20M -maxrate 20M -bufsize 40M -pix_fmt yuv420p dec-cam-20M.mp4 2>/dev/null
  [ -f dec-testsrc2.mp4 ] || ffmpeg "${Q[@]}" -y -f lavfi -i testsrc2=size=1920x1080:rate=30 -t 20 -c:v libx264 -preset medium -crf 20 -pix_fmt yuv420p dec-testsrc2.mp4 2>/dev/null
  [ -f dec-cam-hevc.mp4 ] || ffmpeg "${Q[@]}" -y -stream_loop 2 -i "$CAM" -t 20 -r 30 -an -c:v libx265 -preset fast -crf 22 -pix_fmt yuv420p -tag:v hvc1 dec-cam-hevc.mp4 2>/dev/null
  return 0
}

# encode.rs's input side, verbatim: raw rgb24 frames on a pipe
# The frames reach the encoder from a second ffmpeg decoding the clip to rgb24 on a pipe,
# not from a raw file: 1080p rgb24 is 6.2 MB a frame, and on a 16 GB machine under memory
# pressure a raw file falls out of the page cache and the disk becomes the bottleneck.
feed () {
  case $1 in
    cam)      ffmpeg -hide_banner -loglevel error -i "$CAM" -vf fps=30 -frames:v $FR -f rawvideo -pix_fmt rgb24 - ;;
    screen)   ffmpeg -hide_banner -loglevel error -i "$SCREEN" -vf fps=30 -frames:v $FR -f rawvideo -pix_fmt rgb24 - ;;
    testsrc2) ffmpeg -hide_banner -loglevel error -f lavfi -i testsrc2=size=1920x1080:rate=30 -frames:v $FR -f rawvideo -pix_fmt rgb24 - ;;
  esac
}

encode_one () { # clip label encoder-args...
  local clip=$1 label=$2; shift 2
  # best of three, the input already in the page cache, so the number is the encoder's
  local best=999 t
  for _ in 1 2 3; do
    t=$(feed "$clip" | ffmpeg "${Q[@]}" -benchmark -y -f rawvideo -pix_fmt rgb24 -s 1920x1080 -r 30 -i pipe:0 \
          "$@" -r 30 -an -movflags +faststart -f mp4 "$clip-$label.mp4" 2>&1 | rtime)
    best=$(echo "if ($t < $best) $t else $best" | bc -l)
  done
  local bytes; bytes=$(stat -f %z "$clip-$label.mp4" 2>/dev/null || stat -c %s "$clip-$label.mp4")
  local kbps; kbps=$(echo "$bytes*8/($FR/30)/1000" | bc)
  local vmaf; vmaf=$(feed "$clip" | ffmpeg -hide_banner -nostats -loglevel info -i "$clip-$label.mp4" \
      -f rawvideo -pix_fmt rgb24 -s 1920x1080 -r 30 -i pipe:0 \
      -lavfi "[0:v]setpts=PTS-STARTPTS[d];[1:v]format=yuv420p,setpts=PTS-STARTPTS[r];[d][r]libvmaf=n_threads=4:shortest=1" \
      -f null - 2>&1 | grep -E 'VMAF score' | grep -o '[0-9.]*$')
  printf '%-9s %-18s fps=%6.1f  kbps=%6s  vmaf=%5.2f\n' "$clip" "$label" "$(echo "$FR/$best" | bc -l)" "$kbps" "$vmaf"
}

# the ceiling the input side sets: rgb24 on the pipe, converted to yuv420p, not encoded
feed_ceiling () {
  local t; t=$(feed "$1" | ffmpeg "${Q[@]}" -benchmark -f rawvideo -pix_fmt rgb24 -s 1920x1080 -r 30 -i pipe:0 \
        -pix_fmt yuv420p -f null - 2>&1 | rtime)
  printf '%-9s %-18s fps=%6.1f\n' "$1" "feed only, no encode" "$(echo "$FR/$t" | bc -l)"
}

encode () {
  prepare
  for clip in ${CLIPS:-cam screen testsrc2}; do
    feed "$clip" | cat >/dev/null  # warm the source into the page cache
    feed_ceiling "$clip"
    for p in medium fast veryfast ultrafast; do
      encode_one $clip "x264-$p" -c:v libx264 -preset $p -crf 20 -pix_fmt yuv420p
    done
    # VideoToolbox at the bitrate libx264 medium/CRF 20 chose for the same clip
    local b; b=$(ffprobe -v error -show_entries format=bit_rate -of csv=p=0 "$clip-x264-medium.mp4")
    encode_one $clip "vt-h264-matched" -c:v h264_videotoolbox -b:v "$b" -profile:v high -pix_fmt yuv420p
    encode_one $clip "vt-hevc-matched" -c:v hevc_videotoolbox -b:v "$b" -tag:v hvc1 -pix_fmt yuv420p
    encode_one $clip "vt-h264-q65" -c:v h264_videotoolbox -q:v 65 -profile:v high -pix_fmt yuv420p
    encode_one $clip "vt-h264-default" -c:v h264_videotoolbox -pix_fmt yuv420p
  done
}

# decode::frames_from's filter chain (crates/montagent-render/src/decode.rs), from 0 at speed 1
FROMS="settb=1/1000000,setpts='ceil((2*(ceil((PTS-1)/1000)-0)-1)*1/(2*1))*1000',fps=30:round=up:start_time=0,scale=1920:1080"

decode_one () { # src label input-args...
  local src=$1 label=$2; shift 2
  local n; n=$(ffprobe -v error -count_packets -select_streams v:0 -show_entries stream=nb_read_packets -of csv=p=0 "$src")
  local out t
  out=$(ffmpeg "${Q[@]}" -benchmark "$@" -ss 0.000 -copyts -i "$src" -vf "$FROMS" -fps_mode passthrough \
        -f rawvideo -pix_fmt rgba - 2>bench.err | cat >/dev/null; cat bench.err)
  t=$(echo "$out" | rtime)
  local u; u=$(echo "$out" | grep -o 'utime=[0-9.]*' | cut -d= -f2)
  printf '%-18s %-22s fps=%7.1f  cpu-s/wall-s=%4.1f\n' "$src" "$label" "$(echo "$n/$t" | bc -l)" "$(echo "$u/$t" | bc -l)"
}

decode () {
  prepare
  for src in dec-cam-crf20.mp4 dec-cam-20M.mp4 dec-testsrc2.mp4 dec-cam-hevc.mp4; do
    for th in 1 2 4 0; do decode_one $src "sw threads=$th" -threads $th; done
    decode_one $src "hwaccel videotoolbox" -hwaccel videotoolbox
  done
  # Raw decoder ceiling, no rgba conversion, no pipe
  for src in dec-cam-crf20.mp4 dec-cam-20M.mp4; do
    local n; n=$(ffprobe -v error -count_packets -select_streams v:0 -show_entries stream=nb_read_packets -of csv=p=0 "$src")
    local t; t=$(ffmpeg "${Q[@]}" -benchmark -i "$src" -f null - 2>&1 | rtime)
    printf '%-18s %-22s fps=%7.1f\n' "$src" "decode only, -f null" "$(echo "$n/$t" | bc -l)"
  done
  # Four streaming decoders at once: aggregate frames per second
  local start end
  start=$(python3 -c 'import time;print(time.time())')
  for i in 1 2 3 4; do
    ffmpeg -hide_banner -loglevel error -ss 0.000 -copyts -i dec-cam-20M.mp4 -vf "$FROMS" -fps_mode passthrough -f rawvideo -pix_fmt rgba - | cat >/dev/null &
  done
  wait
  end=$(python3 -c 'import time;print(time.time())')
  printf '%-18s %-22s fps=%7.1f (aggregate)\n' dec-cam-20M.mp4 "4 sw decoders at once" "$(echo "4*600/($end-$start)" | bc -l)"
}

exact () {
  prepare
  for src in dec-cam-crf20.mp4 dec-cam-hevc.mp4 dec-testsrc2.mp4; do
    for start in 0 7.3; do
      local win; win=$(python3 -c "print(f'{max(0.0,$start-0.2):.3f}')")
      local ms; ms=$(python3 -c "print(int($start*1000))")
      local f="settb=1/1000000,setpts='ceil((2*(ceil((PTS-1)/1000)-$ms)-1)*1/(2*1))*1000',fps=30:round=up:start_time=0,scale=1920:1080"
      local run=(ffmpeg -hide_banner -loglevel error -y)
      local out=(-fps_mode passthrough -frames:v 90 -pix_fmt rgba -f framemd5)
      # the reference: software decode, frames_from's chain verbatim
      "${run[@]}" -ss "$win" -copyts -i "$src" -vf "$f" "${out[@]}" sw.md5
      # 1. -hwaccel videotoolbox added, nothing else (frames arrive as nv12)
      "${run[@]}" -hwaccel videotoolbox -ss "$win" -copyts -i "$src" -vf "$f" "${out[@]}" hw.md5
      # 2. the same, with the downloaded nv12 repacked to yuv420p before the chain
      "${run[@]}" -hwaccel videotoolbox -ss "$win" -copyts -i "$src" -vf "format=yuv420p,$f" "${out[@]}" hw-yuv420p.md5
      # 3. control: software decode, but converted from nv12 like the hwaccel path
      "${run[@]}" -ss "$win" -copyts -i "$src" -vf "format=nv12,$f" "${out[@]}" sw-nv12.md5
      local frames; frames=$(grep -vc '^#' sw.md5)
      for v in hw hw-yuv420p sw-nv12; do
        local dpts dpx
        dpts=$(paste -d'|' <(grep -v '^#' sw.md5 | cut -d, -f2,3) <(grep -v '^#' $v.md5 | cut -d, -f2,3) | awk -F'|' '$1!=$2' | wc -l | tr -d ' ')
        dpx=$(paste -d'|' <(grep -v '^#' sw.md5 | cut -d, -f6) <(grep -v '^#' $v.md5 | cut -d, -f6) | awk -F'|' '$1!=$2' | wc -l | tr -d ' ')
        printf '%-18s start=%-4s %-11s frames=%s  pts-differ=%s  pixels-differ=%s\n' "$src" "$start" "$v" "$frames" "$dpts" "$dpx"
      done
    done
  done
}

ffmpeg -hide_banner -version | head -1
case "${1:-}" in
  encode) encode ;;
  decode) decode ;;
  exact)  exact ;;
  *) echo "usage: $0 encode|decode|exact" >&2; exit 2 ;;
esac
