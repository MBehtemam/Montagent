#!/usr/bin/env bash
# Every number in FINDINGS.md, for #87. Throwaway. ~15-20 min on an M1 Pro.
# Reuses the existing rust-rasterizer binary (#6/#34's harness) by relative
# path -- no fork of the Rust crate, no code changes.
set -e
cd "$(dirname "$0")"
./generate-media.sh
B=../rust-rasterizer/target/release/rast-bench
if [ ! -x "$B" ]; then
  echo "building rast-bench..."
  (cd ../rust-rasterizer && cargo build --release)
fi
mkdir -p out frames

# scale fractions: target-height / native-height, per scene
# 4K  (native height 3840): 1080p=0.5      720p=0.333333 540p=0.25     360p=0.166667
# 8K  (native height 7680): 1080p=0.25     720p=0.166667 540p=0.125    360p=0.083333
# Precision matters here: `(width as f64 * k) as u32` in main.rs TRUNCATES, and
# the encoder needs even width/height for yuv420p. A fraction whose f64 parse
# floors below the exact target (e.g. "0.333333333" parses to a hair under
# 1/3, so 2160*k floors to 719, an ODD width) kills the encoder with a broken
# pipe. These are each hand-verified: `float(s)*w` and `float(s)*h` floor to
# the exact even target on both dimensions.
declare -A SCALES_4K=( [native]=1.0 [1080p]=0.500000 [720p]=0.333334 [540p]=0.250000 [360p]=0.166667 )
declare -A SCALES_8K=( [native]=1.0 [1080p]=0.250000 [720p]=0.166667 [540p]=0.125000 [360p]=0.083334 )
ORDER="native 1080p 720p 540p 360p"

echo "=== 0. correctness: verify before timing (the #6/#34 ground rule) ==="
for res in 4k 8k; do
  $B --backend=skia --scene=scene-$res.json --still --from=10 --out=frames/skia-$res-t10.png >/dev/null
  $B --backend=tiny --scene=scene-$res.json --still --from=10 --out=frames/tiny-$res-t10.png >/dev/null
done
python3 ../rust-rasterizer/compare.py \
  frames/skia-4k-t10.png frames/tiny-4k-t10.png "skia-safe vs tiny-skia @ 4K t=10" \
  frames/skia-8k-t10.png frames/tiny-8k-t10.png "skia-safe vs tiny-skia @ 8K t=10"

echo
echo "=== 1. 10s scrub preview (budget < 5s), skia-safe, across the resolution matrix ==="
for res in 4k 8k; do
  declare -n SCALES="SCALES_${res^^}"
  echo "--- scene-$res.json (skia-safe) ---"
  for tier in $ORDER; do
    k=${SCALES[$tier]}
    echo "-- $tier (scale=$k)"
    /usr/bin/time -p $B --backend=skia --scene=scene-$res.json --scale=$k --noaudio \
      --from=10 --to=20 --out=out/skia-$res-$tier.mp4 2>&1 | grep -E "^(skia|tiny|real)"
  done
done

echo
echo "=== 2. sanity check: tiny-skia at the same matrix (does the axis change the conclusion?) ==="
for res in 4k 8k; do
  declare -n SCALES="SCALES_${res^^}"
  echo "--- scene-$res.json (tiny-skia) ---"
  for tier in $ORDER; do
    k=${SCALES[$tier]}
    echo "-- $tier (scale=$k)"
    /usr/bin/time -p $B --backend=tiny --scene=scene-$res.json --scale=$k --noaudio \
      --from=10 --to=20 --out=out/tiny-$res-$tier.mp4 2>&1 | grep -E "^(skia|tiny|real)"
  done
done

echo
echo "=== 3. resident per-frame ablations: does #34's ~93% resample finding hold as output shrinks? ==="
for res in 4k 8k; do
  declare -n SCALES="SCALES_${res^^}"
  echo "--- scene-$res.json (skia-safe, resident, from=10) ---"
  for tier in native 1080p 360p; do
    k=${SCALES[$tier]}
    echo "-- $tier (scale=$k)"
    $B --backend=skia --scene=scene-$res.json --scale=$k --bench=60 --from=10
    $B --backend=skia --scene=scene-$res.json --scale=$k --bench=60 --from=10 --notext
    $B --backend=skia --scene=scene-$res.json --scale=$k --bench=60 --from=10 --noimage
    $B --backend=skia --scene=scene-$res.json --scale=$k --bench=60 --from=10 --notext --noimage
  done
done
