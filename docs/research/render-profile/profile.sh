#!/bin/zsh
# Reproduces #623's profile. Run from the repo root on branch research/623-render-profile:
#   docs/research/render-profile/profile.sh
# Builds the instrumented binary in a private target dir, makes the 1 s GOP copies,
# renders both projects with MONTAGENT_PROFILE=1 under /usr/bin/time -l, then times
# per-spawn costs, the encoder alone and the audio mix. Run on an idle machine.
set -e
zmodload zsh/datetime
ROOT=$PWD
D=$ROOT/docs/research/render-profile
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-$ROOT/../target-623}
cargo build --release --bin montagent
BIN=$CARGO_TARGET_DIR/release/montagent
A=$ROOT/docs/research/skills-eval/assets
cd $D
mkdir -p short out
for f in screen/session presenter/take-1; do
  ffmpeg -hide_banner -loglevel error -y -i $A/$f.mp4 -c:v libx264 -preset fast -crf 18 \
    -g 25 -keyint_min 25 -sc_threshold 0 -c:a copy short/${f:t}.mp4
done
for p in long-gop short-gop; do
  echo "=== $p"
  MONTAGENT_PROFILE=1 /usr/bin/time -l $BIN render $p.montagent.json 2>&1 >/dev/null \
    | grep -E "PROFILE|real"
done
python3 gop-cost.py $A/screen/session.mp4 $A/presenter/take-1.mp4 short/session.mp4 short/take-1.mp4

bench() { local name=$1; shift; local t0=$EPOCHREALTIME
  for i in {1..30}; do "$@" >/dev/null 2>&1; done
  printf "%-44s %7.1f ms\n" $name $(( (EPOCHREALTIME - t0) * 1000 / 30 )); }
S=$A/screen/session.mp4
bench "bare spawn" ffmpeg -hide_banner -loglevel error -f lavfi -i color=s=16x16 -frames:v 1 -f null -
bench "1 frame at keyframe 10.000s, rgba pipe" ffmpeg -hide_banner -loglevel error -ss 10.000 -copyts -t 0.001 -i $S -frames:v 1 -vf scale=1920:1080 -f rawvideo -pix_fmt rgba -
bench "frame_at 10.200s (0.2 s past keyframe)" ffmpeg -hide_banner -loglevel error -ss 10.000 -copyts -t 0.201 -i $S -vf "select='lte(t\,10.200001)',scale=1920:1080" -fps_mode passthrough -f rawvideo -pix_fmt rgba -
bench "frame_at 19.900s (9.9 s past keyframe)" ffmpeg -hide_banner -loglevel error -ss 19.700 -copyts -t 0.201 -i $S -vf "select='lte(t\,19.900001)',scale=1920:1080" -fps_mode passthrough -f rawvideo -pix_fmt rgba -

t0=$EPOCHREALTIME
ffmpeg -hide_banner -loglevel error -i out/long-gop.mp4 -f rawvideo -pix_fmt rgb24 - \
  | ffmpeg -hide_banner -loglevel error -y -f rawvideo -pix_fmt rgb24 -s 1920x1080 -r 30 -i pipe:0 \
      -c:v libx264 -preset medium -crf 20 -pix_fmt yuv420p -r 30 -an -f mp4 out/enc-only.mp4
echo "encoder alone, 1920 frames: $(( (EPOCHREALTIME-t0)*1000 )) ms"
t0=$EPOCHREALTIME
ffmpeg -hide_banner -loglevel error -y -f lavfi -i "sine=f=440:d=360:r=44100" -i $A/presenter/take-1.mp4 \
  -filter_complex "[0:a]aresample=48000,aformat=channel_layouts=stereo,volume=1[a0];[1:a]aresample=48000,aformat=channel_layouts=stereo,atrim=start=0:end=9,asetpts=PTS-STARTPTS,adelay=55000:all=1[a1];[a0][a1]amix=inputs=2:normalize=0,apad=whole_dur=360,atrim=end=360[mix]" \
  -map "[mix]" -c:a aac -b:a 160k -f mp4 out/mix.m4a
echo "audio mix, 6-min bus, 2 inputs, AAC 160k: $(( (EPOCHREALTIME-t0)*1000 )) ms"
