#!/usr/bin/env bash
# Every number in FINDINGS.md. Throwaway. ~10 min on an M1 Pro.
set -e
cd "$(dirname "$0")"
# The harness is a workspace member now (#189), so the binary lands in the
# workspace target directory rather than beside this script.
ROOT=../../../..
B=$ROOT/target/release/rast-bench
mkdir -p frames out

echo "=== 0. correctness: verify before timing (the #6 ground rule) ==="
# Written to out/ (gitignored) rather than to frames/, which holds #34's
# tracked evidence: since #189 the harness shapes in the vendored Open Runde
# rather than the system SF Pro Rounded, so rendering over those files would
# replace the evidence with differently-typeset images and dirty the tree.
#
# The two `ref-skiacanvas-t0.png` comparisons #34 ran are dropped here for the
# same reason: that reference is typeset in SF Pro Rounded, so the diff would
# now be measuring a font change rather than a rasterizer difference, and the
# number in FINDINGS.md is not reproducible against it. FINDINGS.md records what
# it measured at the time; the live arm-against-arm check is `tests/oracle.rs`.
for b in skia tiny; do
  $B --backend=$b --still --from=0  --out=out/$b-t0.png  >/dev/null
  $B --backend=$b --still --from=40 --out=out/$b-t40.png >/dev/null
  $B --backend=$b --scene=scene-video.json --still --from=40 --out=out/$b-video-t40.png >/dev/null
done
python3 compare.py \
  out/skia-t0.png  out/tiny-t0.png  "skia-safe vs tiny-skia @ t=0" \
  out/skia-t40.png out/tiny-t40.png "skia-safe vs tiny-skia @ t=40" \
  out/skia-video-t40.png out/tiny-video-t40.png "with video @ t=40"

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
./seek.sh $ROOT/fixtures/en-halloween-decorating/reference/en-halloween-decorating.mp4

echo
echo "=== 5. the 4K pass — 2160x3840, never re-derived since ADR-0003 ==="
for b in skia tiny; do
  $B --backend=$b --scale=2 --bench=60 --from=0
  /usr/bin/time -p $B --backend=$b --scale=2 --scene=scene-video.json \
    --from=40 --to=50 --out=out/$b-4k-prev.mp4 2>&1 | grep -E "^(skia|tiny|real)"
done
