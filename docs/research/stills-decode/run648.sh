#!/bin/bash
# usage: run648.sh <mode: main|raster|cache> <tag>
S=${STILLS_SPIKE_OUT:?set STILLS_SPIKE_OUT to a scratch dir holding wt648/, target648/ and p/}
cd $S/wt648/fixtures/benchmark/spy-trailer
[ "$1" = main ] && unset MONTAGENT_SPIKE_STILLS || export MONTAGENT_SPIKE_STILLS=$1
LOAD=$(sysctl -n vm.loadavg | awk '{print $2}')
/usr/bin/time -l $S/target648/release/montagent render trailer.montagent.json >/dev/null 2>$S/p/$2.time
REAL=$(grep -Eo '[0-9.]+ real' $S/p/$2.time | cut -d' ' -f1); U=$(grep -Eo '[0-9.]+ user' $S/p/$2.time | cut -d' ' -f1); SY=$(grep -Eo '[0-9.]+ sys' $S/p/$2.time | cut -d' ' -f1)
RSS=$(grep 'maximum resident' $S/p/$2.time | awk '{printf "%.0f", $1/1048576}')
ffmpeg -loglevel error -i out/silent-protocol.mp4 -map 0:v -f framemd5 - | grep -v '^#' > $S/p/$2.md5
if diff -q $S/p/$2.md5 <(grep -v '^#' frames.framemd5) >/dev/null; then H=same; else H=DIFFER; fi
echo "$2 mode=$1 load=$LOAD wall=${REAL}s cpu=$(python3 -c "print(round(100*($U+$SY)/$REAL))")% rss=${RSS}MB hashes=$H" | tee -a $S/p/results648.txt
