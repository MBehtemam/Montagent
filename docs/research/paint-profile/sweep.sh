#!/bin/sh
S=${PAINT_PROFILE_OUT:?set PAINT_PROFILE_OUT to a scratch dir holding target/, bench/ and p/}
cd $S/bench
for rep in 1 2 3; do
for P in paint-b0-g0-off-t0 paint-b0-g0-still-t0 paint-b0-g0-moving-t0 paint-b1-g0-off-t0 paint-b4-g0-off-t0 paint-b0-g1-off-t0 paint-b0-g4-off-t0 paint-b0-g0-off-t108; do
  LOAD=$(sysctl -n vm.loadavg | awk '{print $2}')
  /usr/bin/time -l $S/target/release/montagent render $P.montagent.json --from 0 --to 6000 >/dev/null 2>$S/p/sw.time
  REAL=$(grep -Eo '[0-9.]+ real' $S/p/sw.time | cut -d' ' -f1)
  U=$(grep -Eo '[0-9.]+ user' $S/p/sw.time | cut -d' ' -f1); SY=$(grep -Eo '[0-9.]+ sys' $S/p/sw.time | cut -d' ' -f1)
  RSS=$(grep 'maximum resident' $S/p/sw.time | awk '{printf "%.0f", $1/1048576}')
  echo "$rep $P load=$LOAD wall=$REAL cpu=$(python3 -c "print(round(100*($U+$SY)/$REAL))") rss=$RSS" | tee -a $S/p/sweep.txt
done; done
