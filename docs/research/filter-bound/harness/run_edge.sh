#!/bin/bash
# Renders every instant of a synthetic project as a full-size PNG, once per bound mode.
#
#   MONTAGENT=<path to the spike's montagent binary> \
#     ./run_edge.sh <project.json> <instants.tsv> <png dir> <mode>...
#
# Make the project first: `python3 gen_edge.py` (writes edge.montagent.json + instants.tsv) or
# `python3 gen_big.py` (big.montagent.json + instants_big.tsv), in a directory that holds a
# copy of the spy-trailer fixture's `fonts/` and `img/`. Always include the mode `none`: it is
# the reference every other mode is compared against (compare.py).
set -u
project=$1 instants=$2 pngdir=$3
shift 3
for mode in "$@"; do
  tag=$(echo "$mode" | tr ':+' '_p')
  mkdir -p "$pngdir/$tag"
  while IFS=$'\t' read -r name t; do
    MONTAGENT_FILTER_BOUND=$mode "$MONTAGENT" frame "$project" --at "$t" --full --png \
      --out "$pngdir/$tag/$t.png" > /dev/null 2>&1 || echo "FAILED $mode $name $t"
  done < "$instants"
done
