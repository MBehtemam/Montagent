#!/bin/bash
# PROTOTYPE #764: the `path_reverse` question (ADR-0161 §4). The arc and the circle reversed by
# hand, rendered beside the originals (out/stills/reverse.png), with `validate` and `query --at`
# on each, and the open arc reversed without swapping `in` and `out` (out/reverse.txt).
set -u
cd "$(dirname "$0")"
B=${MONTAGENT_BIN:-montagent}
mkdir -p out/stills
"$B" frame reverse/reverse.json --at 0 --full --png --out out/stills/reverse.png >/dev/null
{
  echo "=== validate reverse/reverse.json"
  "$B" validate reverse/reverse.json 2>&1 | grep -E '^(error|review|[0-9]+ error)'
  echo "=== query --at 0 reverse/reverse.json"
  "$B" query reverse/reverse.json --at 0 2>&1 | grep -E 'hidden:' | sed -E 's/^ +//'
  echo "=== validate reverse/arc-noswap.json (reversed, in and out not swapped)"
  "$B" validate reverse/arc-noswap.json 2>&1 | grep -A1 -E '^error'
  echo "=== the vertex lists"
  python3 -c "
import json
for t in json.load(open('reverse/reverse.json'))['tracks']:
    e = t['elements'][0]
    if e['type'] == 'text':
        print(e['id'], json.dumps(e['path']['points']))
"
} > out/reverse.txt
