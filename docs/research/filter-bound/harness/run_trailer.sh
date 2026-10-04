#!/bin/bash
# Renders the spy-trailer fixture under one bound mode and scores it.
#
#   MONTAGENT=<spike binary> OUT=<scratch dir> ./run_trailer.sh <mode>
#
# Run from the repository root, after `score.wav` exists (see the fixture's README). Writes
# <OUT>/<tag>/{render.mp4,frames.framemd5,stderr.txt,load.txt,verdict.txt}. stderr carries one
# P649 line per filtered element (ms inside the element's filter layers), one P649RAW line per
# frame (a hash of the RGB bytes handed to the encoder) and the MONTAGENT_STAGES line.
set -u
mode=$1
tag=${TAG:-$(echo "$mode" | tr ':+' '_p')}
root=$(pwd)
out=$OUT/$tag
mkdir -p "$out"
cd fixtures/benchmark/spy-trailer
uptime > "$out/load.txt"
s=$(python3 -c 'import time;print(time.time())')
MONTAGENT_FILTER_TIME=1 MONTAGENT_RAW_HASH=1 MONTAGENT_STAGES=1 MONTAGENT_FILTER_BOUND=$mode \
  "$MONTAGENT" render trailer.montagent.json --output "$out/render.mp4" > "$out/stdout.txt" 2> "$out/stderr.txt"
rc=$?
e=$(python3 -c 'import time;print(time.time())')
uptime >> "$out/load.txt"
python3 -c "print('wall_s', $e - $s)" >> "$out/load.txt"
echo "rc $rc" >> "$out/load.txt"
ffmpeg -v error -i "$out/render.mp4" -map 0:v -f framemd5 "$out/frames.framemd5"
python3 - "$root/fixtures/benchmark/spy-trailer/frames.framemd5" "$out/frames.framemd5" <<'EOF' | tee "$out/verdict.txt"
import sys
def rows(p):
    return [tuple(x.strip() for x in l.split(','))[2::3] for l in open(p) if not l.startswith('#')]
a, b = rows(sys.argv[1]), rows(sys.argv[2])
diff = [i for i, (x, y) in enumerate(zip(a, b)) if x != y]
print(f"frames ref={len(a)} got={len(b)} differ={len(diff)}" + (f" first={diff[:12]}" if diff else ""))
print("VERDICT", "PASS" if len(a) == len(b) == 1080 and not diff else "FAIL")
EOF
