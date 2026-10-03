#!/bin/sh
# usage: time.sh <project.json> <tag> [check-framemd5]
S=${PAINT_PROFILE_OUT:?set PAINT_PROFILE_OUT to a scratch dir holding target/, bench/ and p/}
BIN=$S/target/release/montagent
cd "$(dirname "$1")"
P=$(basename "$1")
OUT=$(python3 -c "import json;print(json.load(open('$P'))['output'])")
LOAD=$(sysctl -n vm.loadavg | awk '{print $2}')
/usr/bin/time -l "$BIN" render "$P" > $S/p/$2.log 2> $S/p/$2.time
REAL=$(grep -Eo '[0-9.]+ real' $S/p/$2.time | cut -d' ' -f1)
USER=$(grep -Eo '[0-9.]+ user' $S/p/$2.time | cut -d' ' -f1)
SYS=$(grep -Eo '[0-9.]+ sys' $S/p/$2.time | cut -d' ' -f1)
RSS=$(grep 'maximum resident' $S/p/$2.time | awk '{printf "%.0f", $1/1048576}')
CPU=$(python3 -c "print(round(100*($USER+$SYS)/$REAL))")
H=""
if [ -n "$3" ]; then
  ffmpeg -loglevel error -i "$OUT" -map 0:v -f framemd5 $S/p/$2.framemd5
  if diff -q <(grep -v '^#' $S/p/$2.framemd5) <(grep -v '^#' frames.framemd5) >/dev/null; then H="hashes=same"; else H="hashes=DIFFER"; fi
fi
echo "$2 load=$LOAD wall=${REAL}s cpu=${CPU}% rss=${RSS}MB $H" | tee -a $S/p/results.txt
