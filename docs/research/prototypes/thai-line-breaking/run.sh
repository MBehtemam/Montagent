#!/usr/bin/env bash
# Regenerate everything in results/ and frames/ from scratch.
#
# Two stacks, one shared samples.rs, one shared rasterizer. parley is built
# twice -- the only difference between parley-off-* and parley-on-* is the
# `complex-scripts` Cargo feature.
set -euo pipefail
cd "$(dirname "$0")"

BOX=420          # the fixed box width every sample is laid out in
NARROW=1         # a 1px box forces a break at every opportunity, so the line
                 # starts enumerate the opportunity set itself

rm -rf results frames
mkdir -p results frames

echo "== oracle (macOS CFStringTokenizer) =="
swiftc -O -o oracle oracle.swift
./oracle > results/oracle-cfstringtokenizer.txt

echo "== cosmic-text =="
cargo build --release -p cosmic-probe
./target/release/cosmic-probe WordOrGlyph $BOX    frames > results/cosmic-WordOrGlyph.txt
./target/release/cosmic-probe Word        $BOX    frames > results/cosmic-Word.txt
./target/release/cosmic-probe Word        $NARROW        > results/cosmic-opportunities.txt

echo "== parley, complex-scripts OFF =="
cargo build --release -p parley-probe
cp target/release/parley-probe results/parley-probe-off.bin
./target/release/parley-probe Normal    $BOX    frames > results/parley-off-Normal.txt
./target/release/parley-probe BreakWord $BOX    frames > results/parley-off-BreakWord.txt
./target/release/parley-probe Normal    $NARROW        > results/parley-off-opportunities.txt

echo "== parley, complex-scripts ON =="
cargo build --release -p parley-probe --features complex-scripts
cp target/release/parley-probe results/parley-probe-on.bin
./target/release/parley-probe Normal    $BOX    frames > results/parley-on-Normal.txt
./target/release/parley-probe BreakWord $BOX    frames > results/parley-on-BreakWord.txt
./target/release/parley-probe Normal    $NARROW        > results/parley-on-opportunities.txt

echo "== segmentation vs oracle =="
./compare.py > results/segmentation-vs-oracle.txt

echo "== binary size =="
{
  echo "# release binaries, same crate, same code, one Cargo feature apart"
  ls -l results/parley-probe-off.bin results/parley-probe-on.bin \
        target/release/cosmic-probe | awk '{printf "%-40s %10d bytes\n", $NF, $5}'
} > results/binary-size.txt

echo "== startup + layout =="
./bench_startup.py > results/startup.txt

echo "== icu4x#7218 width sweep =="
{
  echo "# Does the Khmer break-before-space ever reach the frame? A line that"
  echo "# STARTS with a space is the bug surfacing. Swept 60..900px."
  cargo build --release -p parley-probe --features complex-scripts >/dev/null 2>&1
  hits=0; total=0
  for w in $(seq 60 5 900); do
    total=$((total+1))
    if ./target/release/parley-probe Normal "$w" 2>/dev/null | grep -qE '^line .*\] " '; then
      hits=$((hits+1)); echo "width=$w: a line starts on a space"
    fi
  done
  echo "widths where the bug surfaced: $hits of $total"
} > results/icu4x-7218-sweep.txt

rm -f results/parley-probe-off.bin results/parley-probe-on.bin

echo "done -- see results/ and frames/"
