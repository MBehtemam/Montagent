#!/bin/bash
# PROTOTYPE #750: wall time of the whole clip (660 frames, 1080p) on one painter, with the new
# fields and with every new field dropped, three runs each, alternating. Prints the 1-min load
# average before each run, because other sessions share the machine.
set -u
cd "$(dirname "$0")"
B=${MONTAGENT_BIN:-montagent}
T=${PROTO_SCRATCH:-/tmp}
for i in 1 2 3; do
  for p in strokes plain; do
    f=$p.json; [ $p = strokes ] && f=strokes.montagent.json
    rm -f "$T/cost-$p.mp4"
    load=$(sysctl -n vm.loadavg | awk '{print $2}')
    s=$(python3 -c 'import time; print(time.time())')
    MONTAGENT_PAINTING=1,1000 "$B" render "$f" --output "$T/cost-$p.mp4" >"$T/cost-$p.out" 2>"$T/cost-$p.err" || tail -3 "$T/cost-$p.err"
    e=$(python3 -c 'import time; print(time.time())')
    python3 -c "print(f'run $i  $p  load1 $load  {($e - $s):.2f} s  {($e - $s) / 660 * 1000:.2f} ms/frame')"
  done
done
rm -f "$T/cost-$p.mp4"
