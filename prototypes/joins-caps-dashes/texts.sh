#!/bin/bash
# PROTOTYPE #750: the `validate` texts (out/validate.txt, out/schema.txt) and `query --at`'s
# stroke block (out/query.txt).
set -u
cd "$(dirname "$0")"
B=${MONTAGENT_BIN:-montagent}
strip='^  (This finding|is determined|semantics|Montagent will|no flag|is advise)'
"$B" validate errors.json --verbose 2>&1 | grep -vE "$strip" | grep -v L-KEY-ORDER -A0 \
  | grep -vE "key order does not match" > out/validate.txt
: > out/schema.txt
for f in schema/*.json; do
  echo "== $f" >> out/schema.txt
  "$B" validate "$f" 2>&1 | grep -A1 "E-SCHEMA" | grep -vE "^error|^--" >> out/schema.txt
done
"$B" query query.json --at 1500 --json 2>/dev/null | python3 -c "
import json, sys
d = json.load(sys.stdin)
for p in d['query']['stack']:
    print(p['id'], json.dumps(p.get('stroke'), ensure_ascii=False))
" > out/query.txt
cat out/validate.txt out/schema.txt out/query.txt
