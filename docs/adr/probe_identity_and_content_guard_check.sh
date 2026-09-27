#!/usr/bin/env bash
#
# ADR-0092's two behavioural claims, re-derived end to end against a built binary.
#
# The claims are observable behaviour, so this script asserts them rather than describing
# them, and exits non-zero the moment either stops holding:
#
#   1. A render does not depend on the working directory it was started from. #385's silent
#      cut came from `Mix::of` canonicalising `Probe::source` — a label holding whichever
#      spelling first cached the probe — so a relative spelling cached by a run inside the
#      project directory resolved nowhere in a run started elsewhere. Every audible element
#      was declined, `chains` came out empty, and the encoder took its `-an` branch: a
#      complete, silent video at `0 errors`, exit 0.
#
#   2. A renumbering shuffle is announced. Renaming batch-generated files onto each other's
#      names leaves every path present and, because `mv` preserves mtime to the nanosecond,
#      leaves `size` as the only thing discriminating two takes. The content guard catches
#      it as a `rewritten` miss.
#
# Both need a real `ffmpeg`/`ffprobe`; the script skips rather than fails without them, on
# the same convention as the CLI tests that check for exit 70.
#
# Usage:  docs/adr/probe_identity_and_content_guard_check.sh [path/to/montagent]
# Default binary: target/debug/montagent (build with `cargo build -p montagent`).
#
# To watch it FAIL — which is the point of committing it — run it against a binary built
# from before ADR-0092 (`f344da08` or earlier). Claim 1 reports "none mixed — 1 audible
# element not mixed" and delivers an MP4 with no audio stream at exit 0; claim 2 reports
# `CHANGED`/nothing rather than `REWRITTEN`.

set -uo pipefail

repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
binary="${1:-$repo/target/debug/montagent}"

if [[ ! -x "$binary" ]]; then
  echo "no binary at $binary — run: cargo build -p montagent" >&2
  exit 2
fi
if ! command -v ffprobe >/dev/null || ! command -v ffmpeg >/dev/null; then
  echo "skipping: this check needs ffmpeg and ffprobe on PATH" >&2
  exit 0
fi

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
failures=0

fail() { echo "FAIL: $*" >&2; failures=$((failures + 1)); }
pass() { echo "ok: $*"; }

# A source with an audio stream, from the committed fixtures.
voice="$repo/fixtures/en-halloween-decorating/audio/05-cobweb.mp3"
second="$repo/fixtures/en-halloween-decorating/audio/07-skeleton.mp3"
for f in "$voice" "$second"; do
  [[ -f "$f" ]] || { echo "missing fixture $f" >&2; exit 2; }
done

# ---------------------------------------------------------------------------
# Claim 1: the working directory is not an input to a render.
# ---------------------------------------------------------------------------

project="$work/wd"
mkdir -p "$project/assets" "$project/cache"
cp "$voice" "$project/assets/vo.mp3"
cat > "$project/project.montagent.json" <<'JSON'
{
  "frame": {"width": 320, "height": 180},
  "fps": 12,
  "background": "#101418",
  "duration": 1000,
  "output": "out/repro.mp4",
  "tracks": [
    {
      "name": "sound",
      "layer": 0,
      "elements": [
        {"id":"vo","type":"audio","start":0,"end":1000,"source":"assets/vo.mp3","source_start":0,"source_end":1000}
      ]
    }
  ]
}
JSON

# Run 1 warms the cache from *inside* the project, with a relative project path, which is
# what makes the stored spelling relative.
( cd "$project" && MONTAGENT_CACHE_DIR="$project/cache" "$binary" render project.montagent.json ) >"$work/run1.txt" 2>&1
if grep -q "exit 70\|not found" "$work/run1.txt" && ! [[ -f "$project/out/repro.mp4" ]]; then
  echo "skipping: the first render could not run:" >&2
  cat "$work/run1.txt" >&2
  exit 0
fi

# The trap only exists if the cache really did store a relative spelling under an absolute
# key. Assert that, so a future change that stops storing it turns this into a visible skip
# rather than a silently vacuous pass.
sidecar="$project/cache/probe-cache.json"
if [[ ! -f "$sidecar" ]]; then
  fail "claim 1 setup: no sidecar was written to $sidecar"
else
  spelling="$(python3 -c "
import json,sys
d=json.load(open(sys.argv[1]))
print(next(iter(d['entries'].values()))['probe']['source'])" "$sidecar" 2>/dev/null)"
  if [[ "$spelling" == /* ]]; then
    fail "claim 1 setup: the cached spelling is absolute ('$spelling'), so this run does not exercise the trap"
  else
    pass "claim 1 setup: the cache stores the relative spelling '$spelling' under an absolute key"
  fi
fi

# Run 2 is the same project and the same files, from a different working directory.
rm -f "$project/out/repro.mp4"
( cd "$work" && MONTAGENT_CACHE_DIR="$project/cache" "$binary" render "$project/project.montagent.json" ) >"$work/run2.txt" 2>&1

streams="$(ffprobe -v error -select_streams a -show_entries stream=codec_name -of csv=p=0 "$project/out/repro.mp4" 2>/dev/null)"
if [[ -z "$streams" ]]; then
  fail "claim 1: a render from a different working directory delivered a video with NO audio stream"
  echo "       this is #385 exactly. What the run said:" >&2
  grep -iE "audio|not mixed" "$work/run2.txt" >&2 || true
else
  pass "claim 1: the render mixed audio ($streams) from a different working directory"
fi

if grep -q "established nothing about its source" "$work/run2.txt"; then
  fail "claim 1: the unactionable 'established nothing' wording is still reachable here"
else
  pass "claim 1: no element was declined with 'established nothing'"
fi

# ---------------------------------------------------------------------------
# Claim 2: a renumbering shuffle is announced.
# ---------------------------------------------------------------------------

shuffle="$work/shuffle"
mkdir -p "$shuffle/cache"
cp "$voice" "$shuffle/line-01.mp3"
cp "$second" "$shuffle/line-02.mp3"

# Truncate both to one length, so `size` cannot tell the two takes apart — the condition
# that leaves the guard as the only thing that can.
python3 -c "
import sys
d=sys.argv[1]
a=open(f'{d}/line-01.mp3','rb').read(); b=open(f'{d}/line-02.mp3','rb').read()
n=min(len(a),len(b))
open(f'{d}/line-01.mp3','wb').write(a[:n]); open(f'{d}/line-02.mp3','wb').write(b[:n])
" "$shuffle"

# A batch export writes its files in one operation, so they share an mtime exactly. This is
# what makes the shuffle invisible to the key rather than merely unlikely to be caught.
touch -r "$shuffle/line-01.mp3" "$shuffle/line-02.mp3"

MONTAGENT_CACHE_DIR="$shuffle/cache" "$binary" probe "$shuffle/line-01.mp3" "$shuffle/line-02.mp3" >/dev/null 2>&1

before="$(python3 -c "
import os,sys
d=sys.argv[1]
print(' '.join(f'{os.stat(f\"{d}/{n}\").st_size}:{os.stat(f\"{d}/{n}\").st_mtime_ns}' for n in ('line-01.mp3','line-02.mp3')))" "$shuffle")"

# The renumbering itself: each take's content lands on its neighbour's name, via `mv`, which
# carries the mtime with it untouched.
( cd "$shuffle" && mv line-01.mp3 .t && mv line-02.mp3 line-01.mp3 && mv .t line-02.mp3 )

after="$(python3 -c "
import os,sys
d=sys.argv[1]
print(' '.join(f'{os.stat(f\"{d}/{n}\").st_size}:{os.stat(f\"{d}/{n}\").st_mtime_ns}' for n in ('line-01.mp3','line-02.mp3')))" "$shuffle")"

if [[ "$before" != "$after" ]]; then
  fail "claim 2 setup: the shuffle moved a size or mtime ($before -> $after), so the key could catch it unaided"
else
  pass "claim 2 setup: every size and mtime survived the shuffle intact ($after)"
fi

MONTAGENT_CACHE_DIR="$shuffle/cache" "$binary" probe "$shuffle/line-01.mp3" "$shuffle/line-02.mp3" >"$work/shuffle.txt" 2>&1
if grep -qi "REWRITTEN IN PLACE" "$work/shuffle.txt"; then
  pass "claim 2: the shuffle is announced as rewritten in place"
else
  fail "claim 2: the shuffle was NOT announced — a stale probe can be served for another take"
  grep -iE "probed|changed|rewritten" "$work/shuffle.txt" >&2 || true
fi

# ---------------------------------------------------------------------------

echo
if (( failures )); then
  echo "$failures claim(s) no longer hold." >&2
  exit 1
fi
echo "ADR-0092 holds: both claims re-derived against $binary"
