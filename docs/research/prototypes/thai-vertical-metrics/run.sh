#!/usr/bin/env bash
# Regenerate results.txt and frames/ from scratch.
#
# Fetches the two OFL-1.1 Thai faces the probe measures, pinned by sha256, and
# refuses to run if either hash changes -- so a later reader re-derives the
# numbers against the same bytes, per docs/agents/domain.md's evidence rule.
set -euo pipefail
cd "$(dirname "$0")"

mkdir -p fonts frames

fetch() {  # fetch <url> <dest> <sha256>
  local url="$1" dest="fonts/$2" want="$3"
  if [ ! -f "$dest" ]; then
    echo "== fetching $2"
    curl -fsSL -o "$dest" "$url"
  fi
  local got
  got="$(shasum -a 256 "$dest" | awk '{print $1}')"
  if [ "$got" != "$want" ]; then
    echo "FAIL: $dest sha256 $got != pinned $want" >&2
    exit 1
  fi
}

# Noto Sans Thai 2.002, hinted static TTF, from the Noto project's own release
# repo. OFL-1.1 (name ID 13 carries the SIL OFL string; OFL.txt fetched below).
fetch "https://raw.githubusercontent.com/notofonts/notofonts.github.io/main/fonts/NotoSansThai/hinted/ttf/NotoSansThai-Regular.ttf" \
      "NotoSansThai-Regular.ttf" \
      "61cf814eec46b294d6ea4401ac295d0cecd5207bd2331dcc5a15e7301d30ee44"
fetch "https://raw.githubusercontent.com/notofonts/thai/main/OFL.txt" \
      "NotoSansThai-OFL.txt" \
      "dad6e6abc2bf3fc37cc698af7607c3f4d4235039695713b222e5a034fb5b9b1c"

# Sarabun 1.000, static TTF, from google/fonts. OFL-1.1.
fetch "https://raw.githubusercontent.com/google/fonts/main/ofl/sarabun/Sarabun-Regular.ttf" \
      "Sarabun-Regular.ttf" \
      "226d4f368fbc0457990ddef2692679badfd2c1a4e89e5ac4d43c10ba7743b2f1"
fetch "https://raw.githubusercontent.com/google/fonts/main/ofl/sarabun/OFL.txt" \
      "Sarabun-OFL.txt" \
      "b26cae1321380296ba8311b632a397d5eac11b47197f9d0aa0b9310f1531ad60"

echo "== probe (parley 0.11.1 complex-scripts + skrifa 0.46.2) =="
rm -f frames/*.png
( cd probe && cargo run --release -- ../frames ) > results.txt

echo "== re-deriving the write-up's headline numbers =="
# docs/agents/domain.md's evidence rule: a numeric claim needs a check that
# exits non-zero the moment it stops reproducing. Each line below is a claim
# made in ../../thai-vertical-metrics.md.
fail=0
must() {  # must <description> <grep-pattern>
  if grep -qF -e "$2" results.txt; then
    echo "  ok   $1"
  else
    echo "  FAIL $1  (expected to find: $2)" >&2
    fail=1
  fi
}

# Candidate 1 is a no-op: on both faces hhea == OS/2 sTypo field-for-field,
# and both already set USE_TYPO_METRICS, so skrifa is already on the typo path.
must "Noto hhea == sTypo, bit 7 set" \
     "OS/2 fsSelection = 0b111000000   USE_TYPO_METRICS (bit 7) = true"
must "Noto hhea/sTypo identical" \
     "hhea == sTypo ?  ascender true  descender true  lineGap true"
must "Noto hhea 1061/-450/0" \
     "hhea:  ascender=1061 descender=-450 lineGap=0"
must "Noto sTypo 1061/-450/0" \
     "OS/2 sTypo: ascender=1061 descender=-450 lineGap=0"
must "Sarabun hhea 1068/-232/0" \
     "hhea:  ascender=1068 descender=-232 lineGap=0"
must "Sarabun sTypo 1068/-232/0" \
     "OS/2 sTypo: ascender=1068 descender=-232 lineGap=0"
must "Sarabun usWin diverges from both" \
     "OS/2 usWin: ascent=1286 descent=567"
must "Sarabun sets bit 7 too" \
     "OS/2 fsSelection = 0b11000000   USE_TYPO_METRICS (bit 7) = true"

# The floors, and that 1.1 is below every Thai one and above every Latin one.
must "Noto Thai 2-line floor is 1.3" \
     "-> first tenth with no ink-to-ink collision: line_height=1.3  (1.1 is BELOW it)"
must "Sarabun Thai floor is 1.6" \
     "-> first tenth with no ink-to-ink collision: line_height=1.6  (1.1 is BELOW it)"
must "Latin control clears at 1.0" \
     "-> first tenth with no ink-to-ink collision: line_height=1.0  (1.1 is at or above it)"

# Candidate 1 falls short on Sarabun even in its most generous reading: a slot
# derived from sTypo is 1.30 em, and 1.3 still collides by +12.77px there.
must "Sarabun still collides at the sTypo-implied 1.3" \
     "line_height=1.3  slot=71.50  worst ink-to-ink seam=+12.77 COLLIDES"
# ...while the same reading clears on Noto, whose sTypo sum is 1.511 em.
must "Noto clears at 1.3" \
     "line_height=1.3  slot=71.50  worst ink-to-ink seam=-1.42"

if [ "$fail" -ne 0 ]; then
  echo "one or more headline numbers no longer reproduce" >&2
  exit 1
fi
echo "done -- see results.txt and frames/"
