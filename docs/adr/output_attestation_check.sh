#!/usr/bin/env bash
# ADR-0104's behavioural claims, asserted end to end against a built binary.
#
# Run from anywhere:  bash docs/adr/output_attestation_check.sh
# Exits non-zero, naming every defect, the moment one of them stops holding.
#
# Why this exists
# ---------------
# ADR-0104's claim is about the filesystem: rendering a project must not rename another
# project's finished deliverable out of existence. A test at the verb seam can assert the
# report; only a run of the real binary can assert that the bytes of the earlier cut are
# still there afterwards, and the bytes are what the incident would have lost.
#
# It is also what distinguishes the fix from a description of one. Run it against the
# commit before this ADR and assertion 3 fails: the sibling render exits 0, says
# `0 errors`, and the first project's deliverable is gone.
#
# The two directions matter equally and both are asserted. A `render` that simply refused
# to overwrite anything would satisfy "the finished cut survives" while breaking the
# ordinary loop — the source incident's own cut took four attempts — so assertion 2 pins
# the silence of a re-render, and assertion 5 pins that an unattested file is replaced.
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

# Two sibling projects in one directory, contending for one `output`. This is the incident:
# a generator hardcoded the path and the second language inherited it unchanged. They are
# byte-identical apart from their filenames, so they agree on duration, frame count and
# frame size — which is exactly why a duration comparison would have waved the clobber
# through.
episode() {
  cat > "$WORK/$1.montagent.json" <<JSON
{
  "frame": {"width": 160, "height": 120},
  "fps": 25,
  "background": "$2",
  "duration": 1000,
  "output": "out/episode.mp4",
  "tracks": []
}
JSON
}
# Same duration, same frame count, same frame size; **different pixels**. Both halves of
# that matter. The differing pixels are what makes a clobber detectable at all — with two
# identical documents the render is deterministic, the replacement file is byte-identical to
# the one it destroyed, and assertion 3 would pass on a commit with no check in it. The
# identical duration and frame count are what makes this the right fixture for the decision:
# the heuristic the source doc proposed compares exactly those, and would wave this through.
episode da "#000000"
episode de "#112233"

# A third project whose own `output` is elsewhere. Assertion 7 needs this: a project
# previewing onto its *own* declared `output` is already refused by ADR-0021's rule, so
# using `de` there would pass on any commit and prove nothing about this ADR.
cat > "$WORK/fr.montagent.json" <<JSON
{
  "frame": {"width": 160, "height": 120},
  "fps": 25,
  "background": "#445566",
  "duration": 1000,
  "output": "out/fr.mp4",
  "tracks": []
}
JSON

DELIVERABLE="$WORK/out/episode.mp4"

# ---------------------------------------------------------------------------
# 1. The first render publishes, and stamps what it published.
# ---------------------------------------------------------------------------
echo
echo "1. the first render publishes a stamped deliverable"
"$MONTAGENT" render "$WORK/da.montagent.json" > "$WORK/1.out" 2> "$WORK/1.err"
[ $? -eq 0 ] || fail "the first render should succeed"
[ -f "$DELIVERABLE" ] || fail "the deliverable should exist"
STAMP="$(ffprobe -v error -print_format json -show_format "$DELIVERABLE" \
  | tr ',' '\n' | grep '"comment"')"
case "$STAMP" in
  *"montagent/1 project="*da.montagent.json*) pass "stamped with its own project identity" ;;
  *) fail "the deliverable carries no attestation naming da.montagent.json: $STAMP" ;;
esac

# ---------------------------------------------------------------------------
# 2. Re-rendering the same project over its own last answer is silent.
#    The case mere existence gets wrong, and the reason it was rejected.
# ---------------------------------------------------------------------------
echo
echo "2. a re-render of the same project says nothing"
"$MONTAGENT" render "$WORK/da.montagent.json" > "$WORK/2.out" 2> "$WORK/2.err"
[ $? -eq 0 ] || fail "attempt two over attempt one should succeed"
if grep -qE 'OUTPUT-FOREIGN|OUTPUT-UNATTESTED' "$WORK/2.out"; then
  fail "a project's own attestation should not fire: $(cat "$WORK/2.out")"
else
  pass "the normal working loop is unremarked"
fi

# ---------------------------------------------------------------------------
# 3. THE INCIDENT. The sibling project is refused and the finished cut survives.
# ---------------------------------------------------------------------------
echo
echo "3. a sibling project is refused and the earlier cut survives"
BEFORE="$(shasum -a 256 < "$DELIVERABLE")"
"$MONTAGENT" render "$WORK/de.montagent.json" > "$WORK/3.out" 2> "$WORK/3.err"
[ $? -ne 0 ] && pass "exit non-zero" || fail "rendering the sibling should be refused"
grep -q 'E-OUTPUT-FOREIGN' "$WORK/3.out" \
  && pass "at its own code" \
  || fail "no E-OUTPUT-FOREIGN in: $(cat "$WORK/3.out")"
AFTER="$(shasum -a 256 < "$DELIVERABLE")"
[ "$BEFORE" = "$AFTER" ] \
  && pass "the earlier deliverable is untouched, byte for byte" \
  || fail "the earlier deliverable was destroyed"

# ---------------------------------------------------------------------------
# 4. ADR-0093 condition 1: the refusal costs no wall clock.
#    `Deliverable` writes to a dotted sibling, so a temp file beside the output
#    would mean frames were painted before anyone looked at the path.
# ---------------------------------------------------------------------------
echo
echo "4. the refusal is pre-flight, so no encoder ran"
SIBLINGS="$(find "$WORK/out" -type f ! -name 'episode.mp4' | wc -l | tr -d ' ')"
[ "$SIBLINGS" = "0" ] \
  && pass "no temp sibling: the encoder was never spawned" \
  || fail "$SIBLINGS temp file(s) beside the output — the encode ran before the check"

# ---------------------------------------------------------------------------
# 5. An unattested file is a review, not a refusal, and is replaced.
#    The migration case: every deliverable that predates attestation.
# ---------------------------------------------------------------------------
echo
echo "5. an unattested file is reviewed and replaced"
rm -rf "$WORK/out"
mkdir -p "$WORK/out"
printf 'not a montagent render' > "$DELIVERABLE"
"$MONTAGENT" render "$WORK/da.montagent.json" > "$WORK/5.out" 2> "$WORK/5.err"
[ $? -eq 0 ] && pass "no evidence is not a refusal" || fail "an unattested file should not refuse"
grep -q 'R-OUTPUT-UNATTESTED' "$WORK/5.out" \
  && pass "but it is said out loud" \
  || fail "no R-OUTPUT-UNATTESTED in: $(cat "$WORK/5.out")"
SQUATTER="$(printf 'not a montagent render' | shasum -a 256)"
[ "$(shasum -a 256 < "$DELIVERABLE")" = "$SQUATTER" ] \
  && fail "the render did not proceed" \
  || pass "and the render proceeded"

# ---------------------------------------------------------------------------
# 6. `--no-clobber` only ever tightens.
# ---------------------------------------------------------------------------
echo
echo "6. --no-clobber escalates the unattested case to a refusal"
rm -rf "$WORK/out"
mkdir -p "$WORK/out"
printf 'not a montagent render' > "$DELIVERABLE"
"$MONTAGENT" render --no-clobber "$WORK/da.montagent.json" > "$WORK/6.out" 2> "$WORK/6.err"
# Exit 1 exactly — `error` findings. A commit without the flag exits 3 (clap rejects the
# argument), which is "non-zero" for entirely the wrong reason and would pass a looser test.
CODE=$?
[ "$CODE" -eq 1 ] \
  && pass "exit 1: refused on a finding, not on the argument" \
  || fail "--no-clobber should refuse an unattested file with exit 1, got $CODE"
grep -q 'R-OUTPUT-UNATTESTED' "$WORK/6.out" \
  && pass "at the same code as the review, raised to error" \
  || fail "no R-OUTPUT-UNATTESTED in: $(cat "$WORK/6.out")"
SQUATTER="$(printf 'not a montagent render' | shasum -a 256)"
[ "$(shasum -a 256 < "$DELIVERABLE")" = "$SQUATTER" ] \
  && pass "refused means the bytes are untouched" \
  || fail "the file was replaced despite --no-clobber"

# ---------------------------------------------------------------------------
# 7. `preview` may not destroy another project's deliverable either.
#    ADR-0093 exempts preview from the no-promotion rule; that exemption must
#    not become the route by which a deliverable is clobbered.
# ---------------------------------------------------------------------------
echo
echo "7. a preview refuses to land on another project's deliverable"
rm -rf "$WORK/out"
"$MONTAGENT" render "$WORK/da.montagent.json" > /dev/null 2>&1
BEFORE="$(shasum -a 256 < "$DELIVERABLE")"
"$MONTAGENT" preview "$WORK/fr.montagent.json" --output "$DELIVERABLE" \
  > "$WORK/7.out" 2> "$WORK/7.err"
[ $? -ne 0 ] && pass "exit non-zero" || fail "the preview should be refused"
[ "$BEFORE" = "$(shasum -a 256 < "$DELIVERABLE")" ] \
  && pass "the deliverable is untouched" \
  || fail "a preview destroyed a deliverable"

# ---------------------------------------------------------------------------
echo
if [ "$FAILURES" -eq 0 ]; then
  echo "ADR-0104 holds"
  exit 0
fi
echo "ADR-0104 does NOT hold: $FAILURES failure(s)"
exit 1
