#!/bin/bash
# PROTOTYPE #750: ADR-0158 §5 says a zero gap "joins two dashes". A [10, 0] pattern on the
# dashed rect, rounded rect and ellipse, a closed path and an open miter path, against the same
# elements with no pattern: same bytes, or not, and how many pixels differ.
set -eu
cd "$(dirname "$0")"
B=${MONTAGENT_BIN:-montagent}
T=${PROTO_SCRATCH:-/tmp}
for n in gap0-dashed gap0-plain; do
  rm -rf "$T/$n"; mkdir -p "$T/$n"
  MONTAGENT_PROTO_DUMP="$T/$n" MONTAGENT_PAINTING=1,1000 "$B" render $n.json --output "$T/$n.mp4" >/dev/null 2>&1
done
python3 - "$T" <<'EOF' | tee out/gap0.txt
import json, sys, pathlib
T = pathlib.Path(sys.argv[1])
a = (T / "gap0-dashed" / "0.rgb").read_bytes()
b = (T / "gap0-plain" / "0.rgb").read_bytes()
W = 1920
els = json.load(open("gap0-plain.json"))["tracks"]
print(f"whole frame: {'SAME' if a == b else 'DIFFERS'}")
for t in els:
    e = t["elements"][0]
    n = levels = 0
    for y in range(e["y"], e["y"] + e["height"]):
        for x in range(e["x"], e["x"] + e["width"]):
            i = (y * W + x) * 3
            if a[i:i + 3] != b[i:i + 3]:
                n += 1
                levels = max(levels, max(abs(a[i + c] - b[i + c]) for c in range(3)))
    print(f"{e['id']:14s} pixels differing in its box: {n}, largest difference {levels} levels")
EOF
