#!/usr/bin/env bash
# Every number in FINDINGS.md. Throwaway. ~10 min on an M1 Pro.
set -e
cd "$(dirname "$0")"
B=./target/release/rast-bench
mkdir -p frames out

echo "=== 0. correctness: verify before timing (the #6 ground rule) ==="
for b in skia tiny; do
  $B --backend=$b --still --from=0  --out=frames/$b-t0.png  >/dev/null
  $B --backend=$b --still --from=40 --out=frames/$b-t40.png >/dev/null
  $B --backend=$b --scene=scene-video.json --still --from=40 --out=frames/$b-video-t40.png >/dev/null
done
python3 compare.py \
  frames/skia-t0.png  frames/tiny-t0.png  "skia-safe vs tiny-skia @ t=0" \
  frames/skia-t40.png frames/tiny-t40.png "skia-safe vs tiny-skia @ t=40" \
  frames/skia-video-t40.png frames/tiny-video-t40.png "with video @ t=40" \
  frames/skia-t0.png  frames/ref-skiacanvas-t0.png "skia-safe vs #6 skia-canvas @ t=0" \
  frames/tiny-t0.png  frames/ref-skiacanvas-t0.png "tiny-skia vs #6 skia-canvas @ t=0"

echo
echo "=== 1. full render, 65.216 s at 1080x1920/30 (budget < 2 min) ==="
for b in skia tiny; do
  /usr/bin/time -p $B --backend=$b --out=out/$b-full.mp4 2>&1 | grep -E "^(skia|tiny|real)"
done

echo
echo "=== 1b. full render, same, with a 30 s video clip on the timeline ==="
for b in skia tiny; do
  /usr/bin/time -p $B --backend=$b --scene=scene-video.json --out=out/$b-full-video.mp4 2>&1 | grep -E "^(skia|tiny|real)"
done

echo
echo "=== 2. 10 s preview (budget < 5 s) — the gate that killed the browsers ==="
for sc in scene.json scene-video.json; do
  echo "--- $sc"
  for b in skia tiny; do
    for t in 0 40 55; do
      /usr/bin/time -p $B --backend=$b --scene=$sc --from=$t --to=$((t+10)) \
        --out=out/$b-prev-$t.mp4 2>&1 | grep -E "^(skia|tiny|real)"
    done
  done
done

echo
echo "=== 3. single frame, cold process (the agent self-verification loop) ==="
for sc in scene.json scene-video.json; do
  echo "--- $sc"
  for b in skia tiny; do
    for t in 0 40 55; do
      /usr/bin/time -p $B --backend=$b --scene=$sc --still --from=$t \
        --out=frames/cold-$b-$t.png 2>&1 | grep -E "^(skia|tiny|real)"
    done
  done
done

echo
echo "=== 4. resident per-frame cost (no process, no encoder) ==="
for sc in scene.json scene-video.json; do
  echo "--- $sc"
  for b in skia tiny; do
    for t in 0 40; do $B --backend=$b --scene=$sc --bench=120 --from=$t; done
  done
done

echo
echo "=== 4b. where the frame goes (resident, t=40, stills scene) ==="
for b in skia tiny; do
  echo "--- $b"
  $B --backend=$b --bench=120 --from=40
  $B --backend=$b --bench=120 --from=40 --notext
  $B --backend=$b --bench=120 --from=40 --noimage
  $B --backend=$b --bench=120 --from=40 --notext --noimage
done

echo
echo "=== 4c. decoder seek: is #6's position pathology real with a decoder? ==="
./seek.sh /Users/mohammedehtemam/projects/github/Montaget/fixtures/en-halloween-decorating/reference/en-halloween-decorating.mp4

echo
echo "=== 5. the 4K pass — 2160x3840, never re-derived since ADR-0003 ==="
for b in skia tiny; do
  $B --backend=$b --scale=2 --bench=60 --from=0
  /usr/bin/time -p $B --backend=$b --scale=2 --scene=scene-video.json \
    --from=40 --to=50 --out=out/$b-4k-prev.mp4 2>&1 | grep -E "^(skia|tiny|real)"
done
