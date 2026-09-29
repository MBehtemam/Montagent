#!/usr/bin/env bash
# ADR-0117's behavioural claims, asserted end to end against a built binary.
#
# Run from anywhere:  bash docs/adr/verify_check.sh
# Exits non-zero, naming every defect, the moment one of them stops holding.
#
# Why this exists
# ---------------
# ADR-0117's claim is about a file on disk and a document beside it: `verify` measures the
# deliverable with the decoder, tells a stale deliverable from a wrong one, and never
# attributes an author's silent recording to the engine. The verb-seam tests in
# `crates/montagent-core/tests/verify.rs` assert the report; only a run of the real binary
# can assert the CLI's exit codes, the stamp actually in the container, and `render`'s own
# boundary naming the verb.
#
# Every exit code is pinned exactly, never "non-zero". ADR-0104's script learned that the hard
# way: a commit lacking the verb satisfies "non-zero" through `clap`'s own argument error, and
# the assertion proves nothing. Against the commit before this ADR (`bc523042`) ten of twelve
# assertions fail: every `verify` call on exactly that — exit 3, `E-INVOCATION`, `unrecognized
# subcommand 'verify'` — and the stamp on `montagent/1`. The two that pass there are the two
# that should: a reformat is not staleness (vacuously, since nothing reports staleness), and
# `render` reads a `montagent/1` deliverable as its own.
#
# The two directions matter equally, as they did for ADR-0104. A `verify` that called every
# file stale would pass assertions 3 and 4, so assertion 1 pins a fresh render verifying clean
# and assertion 8 pins a reformatted document not being stale.
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
trap 'rm -rf "$WORK"' EXIT
export MONTAGENT_CACHE_DIR=""

echo "building the binary under test"
cargo build -q -p montagent || { echo "FAIL: the binary did not build"; exit 1; }
MONTAGENT="$REPO/target/debug/montagent"

FAILURES=0
fail() { echo "  FAIL: $*"; FAILURES=$((FAILURES + 1)); }
pass() { echo "  ok: $*"; }
ff() { ffmpeg -hide_banner -loglevel error -y "$@" || { echo "FAIL: ffmpeg $*"; exit 1; }; }

# Two sources of one length and different content: a tone and digital silence.
ff -f lavfi -i "sine=f=440:d=3" -ar 48000 "$WORK/line-01.wav"
ff -f lavfi -i "anullsrc=r=48000:cl=mono" -t 3 "$WORK/line-02.wav"

project() { # $1 = duration, $2 = second element's source
  cat > "$WORK/p.montagent.json" <<JSON
{
  "frame": {"width": 160, "height": 120},
  "fps": 25,
  "duration": $1,
  "output": "out/p.mp4",
  "tracks": [
    {
      "name": "a",
      "layer": 1,
      "elements": [
        {"id": "line-01", "type": "audio", "start": 0, "end": 1500, "source": "line-01.wav", "source_start": 0, "source_end": 1500},
        {"id": "line-02", "type": "audio", "start": 1500, "end": 3000, "source": "$2", "source_start": 0, "source_end": 1500}
      ]
    }
  ]
}
JSON
}
P="$WORK/p.montagent.json"
OUT="$WORK/out/p.mp4"

verify() { # sets CODE and TEXT
  TEXT="$("$MONTAGENT" verify "$P" 2>&1)"
  CODE=$?
}
render() {
  "$MONTAGENT" render "$P" > "$WORK/render.txt" 2> /dev/null
  RCODE=$?
}
remux_audio() { # keep the stamp, filter the audio
  ff -i "$OUT" -map 0:v -map 0:a -c:v copy -af "$1" -c:a aac -map_metadata 0 "$WORK/out/t.mp4"
  mv "$WORK/out/t.mp4" "$OUT"
}

echo "1. a fresh render of this document verifies clean"
project 3000 line-01.wav
render
[ "$RCODE" -eq 0 ] || fail "the render itself failed (exit $RCODE)"
verify
if [ "$CODE" -eq 0 ] && printf '%s' "$TEXT" | grep -q "^0 errors, 0 reviews"; then
  pass "exit 0, 0 errors, 0 reviews"
else
  fail "a fresh render did not verify clean (exit $CODE): $(printf '%s' "$TEXT" | head -3)"
fi

echo "2. the stamp records a digest and the engine, and render names verify"
STAMP="$(ffprobe -v error -show_entries format_tags=comment -of csv=p=0 "$OUT")"
case "$STAMP" in
  "montagent/2 engine="*" digest="[0-9a-f]*" project="*p.montagent.json) pass "stamped \`montagent/2\` with a digest" ;;
  *) fail "the deliverable's stamp carries no digest: $STAMP" ;;
esac
if grep -q "montagent verify" "$WORK/render.txt"; then
  pass "render's NOT CHECKED names \`montagent verify\`"
else
  fail "render's report does not name verify"
fi

echo "3. a truncated audio track is an error"
remux_audio "atrim=end=1"
verify
if [ "$CODE" -eq 1 ] && printf '%s' "$TEXT" | grep -q "E-VERIFY-AUDIO-EXTENT"; then
  pass "exit 1, E-VERIFY-AUDIO-EXTENT"
else
  fail "a mix cut to 1 s of 3 s was not an error (exit $CODE): $(printf '%s' "$TEXT" | head -3)"
fi

echo "4. a document edited after the render is one stale error, and nothing is measured"
render
project 1500 line-01.wav
JSON_OUT="$("$MONTAGENT" verify "$P" --json 2>&1)"
CODE=$?
CODES="$(printf '%s' "$JSON_OUT" | python3 -c 'import json,sys; d=json.load(sys.stdin); print(" ".join(f["code"] for f in d["findings"]), d["verify"] is None)')"
if [ "$CODE" -eq 1 ] && [ "$CODES" = "E-VERIFY-STALE True" ]; then
  pass "exit 1, exactly E-VERIFY-STALE, \`verify\` block null"
else
  fail "an edit did not collapse to one stale finding (exit $CODE): $CODES"
fi

echo "5. a renumbering shuffle, mtime preserved, is stale"
project 3000 line-02.wav
render
BEFORE="$(stat -f '%z %m' "$WORK/line-01.wav" 2>/dev/null || stat -c '%s %Y' "$WORK/line-01.wav")"
mv "$WORK/line-01.wav" "$WORK/swap.wav"
mv "$WORK/line-02.wav" "$WORK/line-01.wav"
mv "$WORK/swap.wav" "$WORK/line-02.wav"
AFTER="$(stat -f '%z %m' "$WORK/line-02.wav" 2>/dev/null || stat -c '%s %Y' "$WORK/line-02.wav")"
[ "$BEFORE" = "$AFTER" ] || fail "the fixture is not a shuffle: $BEFORE vs $AFTER"
verify
if [ "$CODE" -eq 1 ] && printf '%s' "$TEXT" | grep -q "E-VERIFY-STALE"; then
  pass "exit 1, E-VERIFY-STALE"
else
  fail "swapped takes were not stale (exit $CODE): $(printf '%s' "$TEXT" | head -3)"
fi
mv "$WORK/line-01.wav" "$WORK/swap.wav"
mv "$WORK/line-02.wav" "$WORK/line-01.wav"
mv "$WORK/swap.wav" "$WORK/line-02.wav"

echo "6. a silent source is censused as the author's material"
verify
if [ "$CODE" -eq 0 ] && printf '%s' "$TEXT" | grep -q "R-VERIFY-SILENT-SPAN" \
  && printf '%s' "$TEXT" | grep -q "silent in its own source (line-02)"; then
  pass "exit 0, R-VERIFY-SILENT-SPAN censuses line-02 as silent in its own source"
else
  fail "the silent recording was not attributed to its source (exit $CODE): $(printf '%s' "$TEXT" | head -8)"
fi

echo "7. an audible source under a silent mix is censused as the engine's"
remux_audio "volume=0"
verify
if printf '%s' "$TEXT" | grep -q "audible in its own source (line-01)"; then
  pass "line-01 is audible in its own source and the mix is silent"
else
  fail "the engine half of the census is missing: $(printf '%s' "$TEXT" | head -8)"
fi

echo "8. reformatting the document does not make the deliverable stale"
render
python3 -c "import json,sys; p=sys.argv[1]; d=json.load(open(p)); open(p,'w').write(json.dumps(d))" "$P"
verify
if printf '%s' "$TEXT" | grep -q "E-VERIFY-STALE"; then
  fail "a whitespace-only edit made the deliverable stale"
else
  pass "canonical form is what is hashed"
fi

echo "9. a \`montagent/1\` deliverable is this project's, staleness unknown, and render reads it as its own"
ff -i "$OUT" -map 0 -c copy -metadata "comment=montagent/1 project=$(cd "$WORK" && pwd -P)/p.montagent.json" "$WORK/out/old.mp4"
mv "$WORK/out/old.mp4" "$OUT"
verify
if printf '%s' "$TEXT" | grep -q "R-VERIFY-STALENESS-UNKNOWN"; then
  pass "R-VERIFY-STALENESS-UNKNOWN"
else
  fail "an old stamp was not read as staleness unknown: $(printf '%s' "$TEXT" | head -3)"
fi
render
if [ "$RCODE" -eq 0 ] && ! grep -q "OUTPUT-FOREIGN\|OUTPUT-UNATTESTED" "$WORK/render.txt"; then
  pass "render's pre-flight reads \`montagent/1\` as Mine"
else
  fail "render did not recognise its own \`montagent/1\` deliverable (exit $RCODE)"
fi

echo "10. nothing at the output path is an error"
rm -f "$OUT"
verify
if [ "$CODE" -eq 1 ] && printf '%s' "$TEXT" | grep -q "E-VERIFY-NO-OUTPUT"; then
  pass "exit 1, E-VERIFY-NO-OUTPUT"
else
  fail "a missing deliverable was not an error (exit $CODE)"
fi

if [ "$FAILURES" -eq 0 ]; then
  echo "ADR-0117 holds"
  exit 0
fi
echo "ADR-0117: $FAILURES failure(s)"
exit 1
