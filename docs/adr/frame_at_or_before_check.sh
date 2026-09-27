#!/usr/bin/env bash
# ADR-0096's measurements, reproduced from scratch against this machine's `ffmpeg`.
#
# Run from anywhere:  bash docs/adr/frame_at_or_before_check.sh
# Exits non-zero, naming every defect, the moment one of them stops holding.
#
# Why this exists
# ---------------
# ADR-0096 rests on facts about `ffmpeg` and about one committed file, not on an argument:
#
#   1. `ffmpeg`'s input seek returns the first frame at or AFTER the instant, so an off-grid
#      sample paints the next frame and the last one paints nothing.
#   2. The committed reference MP4's real frame grid is 25 fps from a 42.031 ms origin, and
#      NEITHER frame rate `ffprobe` reports for it is that grid. This is what kills the
#      arithmetic clamp #387 proposed and ADR-0093 ruling 6 assumed was cheap.
#   3. Bounding the read with `-t` is what makes the correct answer cost ~20% instead of 2x.
#   4. The `select` threshold needs microsecond slack, and 100x more breaks fractional rates.
#
# (1) and (2) are the load-bearing pair: without (1) the defect is not what it looks like,
# and without (2) the obvious fix would have been the right one. They are checked here
# because they are claims about the world, which no unit test of Montagent can falsify —
# `crates/montagent-core/tests/seek_clamp.rs` holds the behaviour of the fix itself.
#
# Frames are identified by an index written into their red channel, never by a picture: the
# question throughout is *which* frame came back, and a hash or an eyeball cannot say.
#
# Needs `ffmpeg`/`ffprobe` on PATH (ADR-0009 ships Montagent as "a binary, plus an ffmpeg
# the user supplies"), and skips with a message rather than failing when they are absent.

set -uo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO" || exit 1

if ! command -v ffprobe > /dev/null 2>&1 || ! command -v ffmpeg > /dev/null 2>&1; then
  echo "skipped: ffmpeg/ffprobe not on PATH (ADR-0009)"
  exit 0
fi
if ! command -v python3 > /dev/null 2>&1; then
  echo "skipped: python3 not on PATH (this script's arithmetic and byte reads)"
  exit 0
fi

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

FAILURES=0
fail() { echo "  FAIL: $*"; FAILURES=$((FAILURES + 1)); }
pass() { echo "  ok: $*"; }

REF="fixtures/en-halloween-decorating/reference/en-halloween-decorating.mp4"
[ -f "$REF" ] || { echo "FAIL: $REF is missing"; exit 1; }

# 82 frames at 25 fps is 3280 ms, which is the source MONTAGENT-2 was reported on.
FRAMES=82
GEN="color=c=black:s=64x64:r=RATE,format=rgba,geq=r='N*3':g='0':b='0':a='255'"

encode() { # encode <path> <rate>
  ffmpeg -hide_banner -loglevel error -y -f lavfi \
    -i "${GEN/RATE/$2}" -frames:v "$FRAMES" -c:v ffv1 -pix_fmt gbrp "$1" 2> /dev/null
}

# The frame index carried in a rawvideo stream on stdin, reading either the first or the
# last whole frame in it. The generator wrote 3 x the frame number into the red channel, so
# this says WHICH frame came back — the question the whole script is about, and one a hash or
# an eyeball cannot answer.
pick() { # pick <first|last>   (rawvideo rgba 64x64 on stdin)
  python3 -c '
import sys
b = sys.stdin.buffer.read()
n = 64 * 64 * 4
if len(b) < n:
    print("none")
else:
    red = b[0] if sys.argv[1] == "first" else b[len(b) - n]
    print(red // 3 if red % 3 == 0 else "unnumbered")
' "$1"
}

# A plain input seek asking for exactly one frame — Montagent's call before ADR-0096.
plain_seek() { # plain_seek <file> <seconds>
  ffmpeg -hide_banner -loglevel error -ss "$2" -i "$1" -frames:v 1 \
    -f rawvideo -pix_fmt rgba - 2> /dev/null | pick first
}

# ADR-0096's call: the last frame starting at or before `at_ms`, within one window.
# Note the argument order — `-ss`/`-copyts`/`-t` are input options and precede `-i`, while
# `-vf` is about the output and follows it. Putting `-vf` first silently decodes nothing.
at_or_before() { # at_or_before <file> <at_ms> <eps_seconds>
  local from thr read
  from=$(( $2 - 200 )); [ "$from" -lt 0 ] && from=0
  thr="$(python3 -c "print(f'{$2/1000 + $3:.9f}')")"
  read="$(python3 -c "print(f'{($2 - $from + 1)/1000:.3f}')")"
  ffmpeg -hide_banner -loglevel error \
    -ss "$(python3 -c "print(f'{$from/1000:.3f}')")" -copyts -t "$read" -i "$1" \
    -vf "select='lte(t\,$thr)',scale=64:64" -vsync 0 \
    -f rawvideo -pix_fmt rgba - 2> /dev/null | pick last
}

echo "== 1. ffmpeg's input seek returns the first frame at or AFTER the instant =="
echo "   (a plain -ss, which is what Montagent used before ADR-0096)"
encode "$WORK/g25.mkv" 25 || { echo "FAIL: could not build the fixture"; exit 1; }
# frame 30 covers [1200, 1240) at 25 fps; frame 81 covers [3240, 3280) and is the last.
for probe in "1.200 30 on its own start" \
             "1.234 31 inside frame 30, so one frame EARLY" \
             "1.239 31 inside frame 30, so one frame EARLY" \
             "3.240 81 on the last frame's start" \
             "3.276 none inside the last frame, so NOTHING — this is MONTAGENT-2"; do
  set -- $probe
  at="$1"; want="$2"; shift 2
  got="$(plain_seek "$WORK/g25.mkv" "$at")"
  if [ "$got" = "$want" ]; then pass "-ss $at -> $got ($*)"
  else fail "-ss $at -> $got, expected $want ($*)"; fi
done

echo "== 1b. keyframe density does not change it =="
ffmpeg -hide_banner -loglevel error -y -f lavfi -i "${GEN/RATE/25}" \
  -frames:v "$FRAMES" -c:v libx264 -g 25 -pix_fmt yuv420p -crf 0 "$WORK/gop.mp4" 2> /dev/null
got="$(plain_seek "$WORK/gop.mp4" 3.276)"
if [ "$got" = "none" ]; then pass "a long-GOP H.264 encode answers identically (none at 3.276)"
else fail "long-GOP at 3.276 -> $got, expected none"; fi

echo "== 2. the reference MP4's real grid is neither frame rate ffprobe reports =="
ffprobe -v error -select_streams v:0 -show_entries frame=pts_time -of csv=p=0 "$REF" \
  > "$WORK/pts.txt" 2> /dev/null
R_RATE="$(ffprobe -v error -select_streams v:0 -show_entries stream=r_frame_rate -of csv=p=0 "$REF")"
AVG_RATE="$(ffprobe -v error -select_streams v:0 -show_entries stream=avg_frame_rate -of csv=p=0 "$REF")"
python3 - "$WORK/pts.txt" "$R_RATE" "$AVG_RATE" <<'PY'
import sys
from fractions import Fraction
pts = [Fraction(l.strip().rstrip(',')) for l in open(sys.argv[1]) if l.strip()]
r, avg = (Fraction(*map(int, s.split('/'))) for s in sys.argv[2:4])
steps = {}
for a, b in zip(pts, pts[1:]):
    steps[round(float(b - a) * 1000, 1)] = steps.get(round(float(b - a) * 1000, 1), 0) + 1
common = max(steps, key=steps.get)
real = Fraction(1000).limit_denominator() / Fraction(common).limit_denominator()
bad = 0
def check(ok, msg):
    global bad
    print(("  ok: " if ok else "  FAIL: ") + msg)
    if not ok: bad += 1
check(abs(float(pts[0]) * 1000 - 42.031) < 0.01,
      f"its frames start at {float(pts[0]) * 1000:.3f} ms, not at zero")
check(abs(common - 40.0) < 0.01,
      f"its most common step is {common} ms, i.e. {float(real):.4f} fps")
check(abs(float(r) - float(real)) > 1.0,
      f"r_frame_rate says {float(r):.4f} fps, which is NOT the grid")
check(abs(float(avg) - float(real)) > 0.005,
      f"avg_frame_rate says {float(avg):.4f} fps, which is NOT the grid")
irregular = sorted(g for g in steps if abs(g - 40.0) > 1.0)
check(len(irregular) >= 1,
      f"and {sum(steps[g] for g in irregular)} gap(s) of {irregular} ms mean no single "
      f"rate describes it at all")
sys.exit(1 if bad else 0)
PY
[ $? -eq 0 ] || FAILURES=$((FAILURES + 1))

echo "== 3. -t is what makes the correct answer cheap =="
echo "   (wall clock over 8 off-grid instants at 1080p; ratios, not absolute times)"
ffmpeg -hide_banner -loglevel error -y -f lavfi \
  -i "testsrc2=size=1920x1080:rate=25" -frames:v 250 -c:v prores_ks -profile:v 0 \
  "$WORK/hd.mov" 2> /dev/null
time_form() { # time_form <bound|unbound>
  local total=0 start end
  for i in 0 1 2 3 4 5 6 7; do
    local at=$((1014 + i * 400)) from
    from=$((at - 200))
    start=$(python3 -c 'import time; print(time.perf_counter())')
    if [ "$1" = bound ]; then
      ffmpeg -hide_banner -loglevel error -ss "$(python3 -c "print(f'{$from/1000:.3f}')")" \
        -copyts -t 0.201 -i "$WORK/hd.mov" \
        -vf "select='lte(t\,$(python3 -c "print(f'{$at/1000:.6f}')"))',scale=1920:1080" \
        -vsync 0 -f rawvideo -pix_fmt rgba - > /dev/null 2>&1
    else
      ffmpeg -hide_banner -loglevel error -ss "$(python3 -c "print(f'{$from/1000:.3f}')")" \
        -copyts -i "$WORK/hd.mov" \
        -vf "select='lte(t\,$(python3 -c "print(f'{$at/1000:.6f}')"))',scale=1920:1080" \
        -vsync 0 -f rawvideo -pix_fmt rgba - > /dev/null 2>&1
    fi
    end=$(python3 -c 'import time; print(time.perf_counter())')
    total=$(python3 -c "print($total + ($end - $start))")
  done
  echo "$total"
}
BOUND="$(time_form bound)"
UNBOUND="$(time_form unbound)"
if python3 -c "import sys; sys.exit(0 if $UNBOUND > $BOUND * 1.2 else 1)"; then
  pass "$(python3 -c "print(f'bounded {$BOUND:.2f}s vs unbounded {$UNBOUND:.2f}s — -t saves {100*(1-$BOUND/$UNBOUND):.0f}%')")"
else
  fail "$(python3 -c "print(f'bounded {$BOUND:.2f}s vs unbounded {$UNBOUND:.2f}s — -t no longer pays')")"
fi

echo "== 4. the select threshold needs microsecond slack, and 100 us is too coarse =="
encode "$WORK/frac.mov" 30000/1001 || { echo "FAIL: could not build the fractional fixture"; exit 1; }
# 1400 ms IS frame 35's start at 25 fps; 1001 ms IS frame 30's start at 30000/1001.
# 133 ms is inside frame 3 at 30000/1001, because frame 4 starts at 133.467 ms.
for row in "g25.mkv 1400 35 0 no-slack-drops-the-frame-on-the-boundary" \
           "frac.mov 1001 30 0 no-slack-drops-the-frame-on-the-boundary"; do
  set -- $row
  got="$(at_or_before "$WORK/$1" "$2" "$4")"
  if [ "$got" != "$3" ]; then pass "without slack, $1 at $2 ms -> $got (wanted $3) — slack is needed"
  else fail "without slack, $1 at $2 ms already returns $3; §4's measurement no longer reproduces"; fi
done
for row in "g25.mkv 1400 35" "frac.mov 1001 30" "frac.mov 133 3" "frac.mov 134 4"; do
  set -- $row
  got="$(at_or_before "$WORK/$1" "$2" 0.000001)"
  if [ "$got" = "$3" ]; then pass "with 1 us of slack, $1 at $2 ms -> $got"
  else fail "with 1 us of slack, $1 at $2 ms -> $got, expected $3"; fi
done
# The over-inclusion needs a frame start that falls within the slack ABOVE a whole
# millisecond, which is the hazard a fractional rate creates and a 25 fps source never can.
# Frame 11 starts at 11*1001/30000 s = 367.033 ms, so 367 ms is inside frame 10 — and a
# 100 us threshold reaches past 367.033 and wrongly takes frame 11.
got="$(at_or_before "$WORK/frac.mov" 367 0.000001)"
if [ "$got" = "10" ]; then pass "with 1 us of slack, frac.mov at 367 ms -> 10"
else fail "with 1 us of slack, frac.mov at 367 ms -> $got, expected 10"; fi
got="$(at_or_before "$WORK/frac.mov" 367 0.0001)"
if [ "$got" = "11" ]; then pass "with 100 us of slack, frac.mov at 367 ms -> 11, over-including the next frame"
else fail "with 100 us of slack, frac.mov at 367 ms -> $got; §4's upper bound no longer reproduces"; fi

echo "== 5. the clamp is symmetric: a source whose frames start late is not refused =="
# The reference MP4's own origin, reproduced: -output_ts_offset puts the first frame at
# 42.031 ms, so instants below that are covered by no frame at all. Before the symmetric
# fallback these refused, which would have moved MONTAGENT-2's failure to the other end.
ffmpeg -hide_banner -loglevel error -y -f lavfi -i "${GEN/RATE/25}" \
  -frames:v "$FRAMES" -c:v ffv1 -pix_fmt gbrp -output_ts_offset 0.042 \
  "$WORK/offset.mov" 2> /dev/null
FIRST_PTS="$(ffprobe -v error -select_streams v:0 -show_entries frame=pts_time \
  -of csv=p=0 "$WORK/offset.mov" | head -1)"
if python3 -c "import sys; sys.exit(0 if abs(float('$FIRST_PTS') - 0.042031) < 1e-5 else 1)"; then
  pass "the fixture's first frame is at ${FIRST_PTS}s, the reference MP4's own origin"
else
  fail "the fixture's first frame is at ${FIRST_PTS}s, expected 0.042031"
fi
# `at_or_before` here is the SELECT pass only — it is what refuses, and the product falls
# back to the window's earliest frame when it comes back empty.
got="$(at_or_before "$WORK/offset.mov" 0 0.000001)"
if [ "$got" = "none" ]; then
  pass "asking only for a frame at-or-before 0 ms returns nothing — the fallback is load-bearing"
else
  fail "at-or-before 0 ms returned $got; §5's premise no longer reproduces"
fi
# And the fallback's own answer: the earliest frame within the window.
got="$(ffmpeg -hide_banner -loglevel error -ss 0.000 -copyts -t 0.001 -i "$WORK/offset.mov" \
  -frames:v 1 -vf scale=64:64 -f rawvideo -pix_fmt rgba - 2> /dev/null | pick first)"
if [ "$got" = "0" ]; then pass "the window's earliest frame is frame 0, which is what gets painted"
else fail "the window's earliest frame came back as $got, expected 0"; fi

echo
if [ "$FAILURES" -eq 0 ]; then
  echo "ADR-0096 holds: $(basename "$0") found no defects."
else
  echo "ADR-0096 does NOT hold: $FAILURES defect(s) above."
fi
exit "$FAILURES"
