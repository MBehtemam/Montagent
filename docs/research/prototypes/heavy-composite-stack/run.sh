#!/usr/bin/env bash
# Every number in FINDINGS.md, for #159. Throwaway.
# Follows #87's run.sh shape exactly (same scale-fraction table, same
# correctness-then-timing structure) so the two are read against the same
# methodology and the same [10,20) 10s window.
set -e
cd "$(dirname "$0")"
./generate-media.sh
python3 gen_scenes.py
B=../rust-rasterizer/target/release/rast-bench
if [ ! -x "$B" ]; then
  echo "building rast-bench..."
  (cd ../rust-rasterizer && cargo build --release)
fi
mkdir -p out frames

# scale fractions: target-height / native-height, per scene (same table as #87)
declare -A SCALES_4K=( [native]=1.0 [1080p]=0.500000 [720p]=0.333334 [540p]=0.250000 [360p]=0.166667 )
declare -A SCALES_8K=( [native]=1.0 [1080p]=0.250000 [720p]=0.166667 [540p]=0.125000 [360p]=0.083334 )
ORDER="native 1080p 720p 540p"

echo "=== 0. correctness: verify before timing (the #6/#34/#87 ground rule) ==="
for res in 4k 8k; do
  $B --backend=skia --scene=scene-$res.json --still --from=10 --out=frames/skia-$res-t10.png >/dev/null
  $B --backend=tiny --scene=scene-$res.json --still --from=10 --out=frames/tiny-$res-t10.png >/dev/null
done
python3 ../rust-rasterizer/compare.py \
  frames/skia-4k-t10.png frames/tiny-4k-t10.png "skia-safe vs tiny-skia @ heavy 4K t=10" \
  frames/skia-8k-t10.png frames/tiny-8k-t10.png "skia-safe vs tiny-skia @ heavy 8K t=10"

echo
echo "=== 1. 10s scrub preview (budget < 5s), skia-safe, heavy stack, across the resolution matrix ==="
for res in 4k 8k; do
  if [ "$res" = "4k" ]; then declare -A SCALES=(); for k in "${!SCALES_4K[@]}"; do SCALES[$k]=${SCALES_4K[$k]}; done
  else declare -A SCALES=(); for k in "${!SCALES_8K[@]}"; do SCALES[$k]=${SCALES_8K[$k]}; done; fi
  echo "--- scene-$res.json (skia-safe) ---"
  for tier in $ORDER; do
    k=${SCALES[$tier]}
    echo "-- $tier (scale=$k)"
    /usr/bin/time -p $B --backend=skia --scene=scene-$res.json --scale=$k --noaudio \
      --from=10 --to=20 --out=out/skia-$res-$tier.mp4 2>&1 | grep -E "^(skia|tiny|real)"
  done
done

echo
echo "=== 2. resident per-frame ablations: where does the raster cost go under a heavy stack? ==="
for res in 4k 8k; do
  if [ "$res" = "4k" ]; then declare -A SCALES=(); for k in "${!SCALES_4K[@]}"; do SCALES[$k]=${SCALES_4K[$k]}; done
  else declare -A SCALES=(); for k in "${!SCALES_8K[@]}"; do SCALES[$k]=${SCALES_8K[$k]}; done; fi
  echo "--- scene-$res.json (skia-safe, resident, from=10) ---"
  for tier in native 720p 540p; do
    k=${SCALES[$tier]}
    echo "-- $tier (scale=$k)"
    $B --backend=skia --scene=scene-$res.json --scale=$k --bench=60 --from=10
    $B --backend=skia --scene=scene-$res.json --scale=$k --bench=60 --from=10 --notext
    $B --backend=skia --scene=scene-$res.json --scale=$k --bench=60 --from=10 --noimage
    $B --backend=skia --scene=scene-$res.json --scale=$k --bench=60 --from=10 --notext --noimage
  done
done
