#!/bin/bash
# PROTOTYPE #764: the captured texts. `validate` on one minimal bad file per code
# (out/validate.txt), `query --at` at telling instants of the clip (out/query.txt), and `shift`
# on a text whose `path.points` is keyed 1000 -> 3000 (out/shift.txt): a cut inside the window,
# and cuts before and after it.
set -u
cd "$(dirname "$0")"
B=${MONTAGENT_BIN:-montagent}
T=${PROTO_SCRATCH:-/tmp}/tp764-texts
mkdir -p "$T" out

{
  for f in errors/*.json; do
    echo "=== $f"
    "$B" validate "$f" --verbose 2>&1 | grep -v '^$' | sed -n '1,/^NOT CHECKED/p' | grep -v '^NOT CHECKED'
  done
} > out/validate.txt

{
  for t in 0 1000 2400 3000 3900 6000 8100 8300 12000 13000 13500 14000 15000 18000 20000 20300 22000 23966; do
    echo "=== query --at $t"
    "$B" query textpath.montagent.json --at "$t" 2>&1 | grep -E 'hidden:|path unresolved' | sed -E 's/^ +//'
  done
} > out/query.txt

{
  for case in keyed:2000 keyed:500 keyed:3500 keyed-even:2000; do
    file=${case%%:*}; at=${case##*:}
    cp shift/$file.json shift/work.json
    echo "=== shift/$file.json: shift --at $at --delta 250 (path.points keyed at 1000 and 3000)"
    "$B" shift shift/work.json --at "$at" --delta 250 2>&1 | grep -v '^$'
    echo "--- the text's path.points and path_offset after:"
    python3 -c "
import json, sys
e = json.load(open('shift/work.json'))['tracks'][0]['elements'][0]
print('start', e['start'], 'end', e['end'])
print('path.points', json.dumps(e['path']['points']))
print('path_offset', json.dumps(e['path_offset']))
"
    echo
  done
} > out/shift.txt
rm -f shift/work.json
