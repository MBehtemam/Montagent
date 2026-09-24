#!/usr/bin/env bash
# Re-derives every numeric claim in ADR-0088 against the committed fixture.
# Exits non-zero, naming the defect, the moment one stops reproducing.
#
# Asserts BOTH directions throughout: the values that must key, and the values
# that must not. A change that makes the keyer more permissive fails here just
# as loudly as one that breaks it.
#
#   1. The fixture's screen colour is (0,205,0) at 5 points x 5 timestamps.
#   2. `chromakey` on the literal colour keys the subject from tolerance 0.05.
#   3. Tolerance 0.01 keys nothing (the identity-adjacent failure).
#   4. The plateau 0.05-0.30 is flat: every value keys within 1pp.
#   5. One static tolerance holds across all 140 frames: zero border-band leak.
#   6. `hsvkey` on a bare hue angle INVERTS — it keys the subject, not the screen.
#   7. `hsvkey` given the full HSV coordinates needs similarity >= 0.70.
#
# Claims 6 and 7 are why ADR-0088 spells the key as a literal `#RRGGBB` rather
# than the hue angle ADR-0049 suggests as the scalar redefinition route.
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
SRC="$HERE/green-screen-trex.mp4"
[ -f "$SRC" ] || { echo "FAIL: fixture missing at $SRC"; exit 1; }

command -v ffmpeg >/dev/null || { echo "FAIL: ffmpeg not on PATH"; exit 1; }
# Captured once: piping into `grep -q` would SIGPIPE ffmpeg, and `pipefail`
# turns that into a spurious failure.
FILTERS="$(ffmpeg -hide_banner -filters 2>/dev/null || true)"
for f in chromakey hsvkey; do
  case "$FILTERS" in
    *" $f "*) ;;
    *) echo "FAIL: this ffmpeg has no '$f' filter"; exit 1 ;;
  esac
done

W=1920; H=1080
fail () { echo "  FAIL: $*"; exit 1; }

# --- 1. the screen colour is invariant ---------------------------------------
echo "1. screen colour invariance"
for t in 0 1.5 3 4.5 5.7; do
  ffmpeg -hide_banner -loglevel error -ss "$t" -i "$SRC" -frames:v 1 \
      -f rawvideo -pix_fmt rgb24 - 2>/dev/null |
  python3 -c "
import sys
d=sys.stdin.buffer.read(); W,H=$W,$H
def px(x,y):
    i=(y*W+x)*3; return tuple(d[i:i+3])
pts=[(20,20),(W-20,20),(20,H-20),(W-20,H-20),(W//2,15)]
bad=[p for p in pts if px(*p)!=(0,205,0)]
if bad:
    print('    t=$t NOT (0,205,0) at', [(p,px(*p)) for p in bad]); sys.exit(1)
print('    t=$t  (0,205,0) at all 5 points')
" || fail "screen colour drifted at t=$t"
done

# --- helper: transparent fraction of one keyed frame -------------------------
keyed_pct () {  # $1 = filter expression
  ffmpeg -hide_banner -loglevel error -ss 2 -i "$SRC" -frames:v 1 \
      -vf "$1,format=rgba" -f rawvideo -pix_fmt rgba - 2>/dev/null |
  python3 -c "
import sys
a=sys.stdin.buffer.read()[3::4]
print(f'{sum(1 for v in a if v<55)*100/len(a):.2f}')
"
}

# --- 2/3/4. the chromakey plateau, and its lower cliff ------------------------
echo "2-4. chromakey plateau on the literal colour"
lo=$(keyed_pct "chromakey=0x00CD00:0.01:0.0")
awk -v v="$lo" 'BEGIN{exit !(v < 1.0)}' || fail "tolerance 0.01 keyed $lo% — expected ~0"
echo "    tolerance 0.01 -> ${lo}% keyed (must be ~0)"

prev=""
for tol in 0.05 0.10 0.20 0.30; do
  pct=$(keyed_pct "chromakey=0x00CD00:$tol:0.0")
  awk -v v="$pct" 'BEGIN{exit !(v > 85.0 && v < 95.0)}' \
    || fail "tolerance $tol keyed $pct% — expected 85-95%"
  if [ -n "$prev" ]; then
    awk -v a="$prev" -v b="$pct" 'BEGIN{d=a-b; if(d<0)d=-d; exit !(d < 1.0)}' \
      || fail "plateau is not flat: $prev% then $pct%"
  fi
  echo "    tolerance $tol -> ${pct}% keyed"
  prev="$pct"
done

# --- 5. one static tolerance holds for every frame ---------------------------
echo "5. static tolerance across all frames"
ffmpeg -hide_banner -loglevel error -i "$SRC" \
    -vf "chromakey=0x00CD00:0.10:0.08,format=rgba,scale=480:270" \
    -f rawvideo -pix_fmt rgba - 2>/dev/null |
python3 -c "
import sys
W,Hh=480,270; FS=W*Hh*4
data=sys.stdin.buffer.read(); n=len(data)//FS
if n < 100: print(f'    only {n} frames decoded'); sys.exit(1)
band=[y*W+x for y in range(Hh) for x in range(W) if x<8 or x>=W-8 or y<8 or y>=Hh-8]
worst=0
for f in range(n):
    a=data[f*FS:(f+1)*FS][3::4]
    worst=max(worst, sum(1 for i in band if a[i]>64))
print(f'    {n} frames, worst border-band leak = {worst} px')
sys.exit(0 if worst==0 else 1)
" || fail "a frame leaked background at the one static tolerance"

# --- 6. a bare hue angle inverts the key -------------------------------------
echo "6. hsvkey on a bare hue angle"
ffmpeg -hide_banner -loglevel error -ss 2 -i "$SRC" -frames:v 1 \
    -vf "hsvkey=hue=120:similarity=0.50,format=rgba" \
    -f rawvideo -pix_fmt rgba - 2>/dev/null |
python3 -c "
import sys
d=sys.stdin.buffer.read(); W=$W
def a(x,y): return d[(y*W+x)*4+3]
bg, subj = a(20,20), a(960,540)
print(f'    background alpha={bg}  subject alpha={subj}')
if bg > 200 and subj < 55:
    print('    inverted as ADR-0088 states: keys the subject, keeps the screen'); sys.exit(0)
print('    NOT inverted — ADR-0088 section \"Why color and not a hue angle\" no longer holds')
sys.exit(1)
" || fail "the bare-hue inversion stopped reproducing"

# --- 7. full HSV coordinates need a far higher similarity --------------------
echo "7. hsvkey given the full HSV coordinates"
for s in 0.05 0.30 0.50; do
  pct=$(keyed_pct "hsvkey=hue=120:sat=1:val=0.804:similarity=$s")
  awk -v v="$pct" 'BEGIN{exit !(v < 1.0)}' \
    || fail "hsvkey similarity $s keyed $pct% — ADR-0088 says it keys nothing below 0.70"
  echo "    similarity $s -> ${pct}% keyed (must be ~0)"
done
pct=$(keyed_pct "hsvkey=hue=120:sat=1:val=0.804:similarity=0.70")
awk -v v="$pct" 'BEGIN{exit !(v > 85.0)}' \
  || fail "hsvkey similarity 0.70 keyed $pct% — expected >85%"
echo "    similarity 0.70 -> ${pct}% keyed"

echo
echo "ALL ADR-0088 CLAIMS REPRODUCE"
