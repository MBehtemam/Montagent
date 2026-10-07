#!/bin/bash
# PROTOTYPE #760: the `validate` texts (out/validate.txt, out/schema.txt) and the `query --at`
# captures (out/query.txt: the text line, then the JSON `trim` block).
set -u
cd "$(dirname "$0")"
B=${MONTAGENT_BIN:-montagent}
"$B" validate errors.json --verbose 2>&1 \
  | awk '/^error/{p=1; print; next} /^[a-z]/{p=0} p && /^  `/{print}' > out/validate.txt
: > out/schema.txt
for f in schema/*.json; do
  echo "== $f" >> out/schema.txt
  "$B" validate "$f" 2>&1 | grep -A2 "E-SCHEMA" | grep -vE "^--" >> out/schema.txt
done
{
  for t in 1500 1000 2900; do
    echo "== query query.json --at $t"
    "$B" query query.json --at "$t" 2>/dev/null | grep -E '^  L'
    echo
  done
  echo "== query query.json --at 1500 --json: each element's trim block"
  "$B" query query.json --at 1500 --json 2>/dev/null | python3 -c "
import json, sys
d = json.load(sys.stdin)
for p in d['query']['stack']:
    print(p['id'], json.dumps(p.get('trim'), ensure_ascii=False))
"
  echo
  echo "== query ../joins-caps-dashes/query.json --at 1500 --json: untrimmed elements carry no trim block"
  "$B" query ../joins-caps-dashes/query.json --at 1500 --json 2>/dev/null | python3 -c "
import json, sys
d = json.load(sys.stdin)
print('elements with a trim key:', sum('trim' in p for p in d['query']['stack']), 'of', len(d['query']['stack']))
"
} > out/query.txt
cat out/validate.txt out/schema.txt out/query.txt
