#!/bin/sh
# PROTOTYPE #718 — the still and identity checks. Usage: run_checks.sh <test binary>
# Writes out/checks/*.jsonl.
T="$1"
cd "$(dirname "$0")"
HERE="$(pwd)"
mkdir -p out/checks
hashes() { MB718_PROJECT="$HERE/$1" "$T" --ignored --nocapture --exact motion_blur_hashes 2>/dev/null | grep '^{'; }
hashes still-on.montagent.json > out/checks/still-on.jsonl
hashes still-off.montagent.json > out/checks/still-off.jsonl
# Every still element forced through the accumulation: N identical samples.
MB718_FORCE_SAMPLING=1 MB718_IDENTITY=1 MB718_PROJECT="$HERE/still-on.montagent.json" \
  "$T" --ignored --nocapture --exact motion_blur_hashes > out/checks/forced.out 2>&1
grep '^{' out/checks/forced.out > out/checks/still-forced.jsonl
grep 'MB718_IDENTITY' out/checks/forced.out > out/checks/identity.txt
rm out/checks/forced.out
python3 - <<'EOF'
import json
def load(n): return json.loads(open(f"out/checks/{n}.jsonl").read())
on, off, forced = load("still-on"), load("still-off"), load("still-forced")
print("still-on == still-off:", on["hashes"] == off["hashes"], on["frames"], "frames")
d = [i for i,(a,b) in enumerate(zip(forced["hashes"], off["hashes"])) if a != b]
print("forced-through-accumulation == still-off:", not d, "differing frames:", len(d))
lines = open("out/checks/identity.txt").read().splitlines()
bad = [l for l in lines if not l.endswith("bytes_differing=0")]
print("identity checks:", len(lines), "element-frames; with any byte differing:", len(bad))
EOF
