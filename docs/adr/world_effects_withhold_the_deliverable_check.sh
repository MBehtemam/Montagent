#!/usr/bin/env bash
# ADR-0093's behavioural claims, asserted end to end against a built binary.
#
# Run from anywhere:  bash docs/adr/world_effects_withhold_the_deliverable_check.sh
# Exits non-zero, naming every defect, the moment one of them stops holding.
#
# Why this exists
# ---------------
# ADR-0093's central claim is about the filesystem, not about prose:
#
#   A file at the output path is a render with zero errors.
#
# What shipped an eleven-minute silent cut was not an unread report — it was a plausible
# file existing, which is what a human uploads and what an MCP agent `stat`s. A test at the
# verb seam can assert the report; only a run of the real binary can assert what is on disk
# afterwards, which is the thing that actually misled someone.
#
# It is also the one artifact that distinguishes a fix from a description of one. Run it
# against the commit before this ADR and assertions 1–4 fail: the render exits 0, reports
# `0 errors`, and leaves a playable MP4 with no audio in it.
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

WORK="$(mktemp -d)"
trap 'chmod -R u+rwX "$WORK" 2>/dev/null; rm -rf "$WORK"' EXIT

echo "building the binary under test"
cargo build -q -p montagent || { echo "FAIL: the binary did not build"; exit 1; }
MONTAGENT="$REPO/target/debug/montagent"

FAILURES=0
fail() { echo "  FAIL: $*"; FAILURES=$((FAILURES + 1)); }
pass() { echo "  ok: $*"; }

NARRATION="$REPO/fixtures/en-halloween-decorating/audio/05-cobweb.mp3"
SILENT_VIDEO="$REPO/fixtures/en-halloween-decorating/reference/kenburns/05.mp4"

# One project per case, so a refusal in one cannot mask another.
project() {
  local dir="$1" element="$2"
  mkdir -p "$dir"
  cat > "$dir/p.montagent.json" <<JSON
{
  "frame": {"width": 200, "height": 200},
  "fps": 25,
  "background": "#000000",
  "duration": 1000,
  "output": "out/p.mp4",
  "tracks": [{"name": "only", "layer": 0, "elements": [$element]}]
}
JSON
}

# ---------------------------------------------------------------------------
# 1 + 2. An audible element that cannot be mixed: exit 1, a counted error, and
#        nothing at the output path.
# ---------------------------------------------------------------------------
echo
echo "1/2. an unmixable audible element withholds the deliverable"
CASE="$WORK/unmixable"
cp "$NARRATION" "$WORK/vo.mp3"
mkdir -p "$CASE"
cp "$WORK/vo.mp3" "$CASE/vo.mp3"
chmod 000 "$CASE/vo.mp3"
project "$CASE" '{"id":"vo","type":"audio","start":0,"end":1000,"source":"vo.mp3","source_start":0,"source_end":1000}'

OUT="$("$MONTAGENT" render "$CASE/p.montagent.json" --json 2> /dev/null)"
STATUS=$?

if [ "$STATUS" -eq 1 ]; then
  pass "exit 1"
else
  fail "exit $STATUS, expected 1"
fi

if echo "$OUT" | grep -q 'E-NOT-MIXED-'; then
  pass "the drop names its own code"
else
  fail "the drop carries no E-NOT-MIXED-* code: $(echo "$OUT" | head -c 400)"
fi

if [ -e "$CASE/out/p.mp4" ]; then
  fail "a render with an error left a file at the output path — this is the invariant"
else
  pass "no file at the output path"
fi

# Condition 1: the mix is pre-flighted, so there was never a temp file either.
LEFTOVER="$(ls -A "$CASE/out" 2> /dev/null | wc -l | tr -d ' ')"
if [ "$LEFTOVER" = "0" ]; then
  pass "nothing beside the output path either — the mix was pre-flighted"
else
  fail "the refusal happened after the encoder started: $(ls -A "$CASE/out")"
fi
chmod 644 "$CASE/vo.mp3"

# ---------------------------------------------------------------------------
# 3. An element whose range is empty is reported rather than silently absent.
#    No `validate` check states this, so before ADR-0093 it rendered at zero errors.
# ---------------------------------------------------------------------------
echo
echo "3. an element occupying no instant is an error"
CASE="$WORK/empty-range"
mkdir -p "$CASE"
cp "$NARRATION" "$CASE/vo.mp3"
project "$CASE" '{"id":"vo","type":"audio","start":500,"end":500,"source":"vo.mp3","source_start":0,"source_end":1000}'

OUT="$("$MONTAGENT" render "$CASE/p.montagent.json" --json 2> /dev/null)"
if echo "$OUT" | grep -q 'E-EMPTY-RANGE'; then
  pass "E-EMPTY-RANGE fired"
else
  fail "an element occupying no instant was passed over in silence"
fi
if [ -e "$CASE/out/p.mp4" ]; then
  fail "it published anyway"
else
  pass "nothing published"
fi

# ---------------------------------------------------------------------------
# 4. The reclassification, and the other direction: an ordinary silent video is a
#    `note`, exits 0, and DOES publish. Without this the three above are satisfied
#    by a `render` that has simply stopped working.
# ---------------------------------------------------------------------------
echo
echo "4. a silent video is a note and still publishes"
CASE="$WORK/silent-video"
project "$CASE" "$(printf '{"id":"clip","type":"video","start":0,"end":200,"source":"%s","source_start":0,"source_end":200,"x":100,"y":100,"width":100,"height":100,"fit":"cover"}' "$SILENT_VIDEO")"

OUT="$("$MONTAGENT" render "$CASE/p.montagent.json" --json 2> /dev/null)"
STATUS=$?

if [ "$STATUS" -eq 0 ]; then
  pass "exit 0"
else
  fail "exit $STATUS, expected 0 — a note must not withhold the deliverable"
fi
if echo "$OUT" | grep -q 'N-NO-AUDIO-STREAM'; then
  pass "N-NO-AUDIO-STREAM fired"
else
  fail "an ordinary silent video said nothing at all"
fi
if [ -e "$CASE/out/p.mp4" ]; then
  pass "the deliverable was published"
else
  fail "a note withheld the deliverable"
fi

# The file is real, read back through a decoder that is not the encoder that wrote it.
STREAMS="$(ffprobe -v error -show_entries stream=codec_type -of csv=p=0 "$CASE/out/p.mp4" 2> /dev/null | tr '\n' ',')"
if echo "$STREAMS" | grep -q 'video'; then
  pass "the published file carries a video stream ($STREAMS)"
else
  fail "the published file carries no video stream ($STREAMS)"
fi

echo
if [ "$FAILURES" -eq 0 ]; then
  echo "ADR-0093 holds"
  exit 0
fi
echo "ADR-0093 does not hold: $FAILURES failure(s)"
exit 1
