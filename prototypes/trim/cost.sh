#!/bin/bash
# PROTOTYPE #760: wall time of the whole clip (690 frames, 1080p) on one painter, with the trim
# fields and with every trim field dropped (plain.json), three runs each, alternating. Prints
# the 1-min load average before each run, because other sessions share the machine.
set -u
cd "$(dirname "$0")"
B=${MONTAGENT_BIN:-montagent}
T=${PROTO_SCRATCH:-/tmp}
for i in 1 2 3; do
  for p in trim plain; do
    f=$p.json; [ $p = trim ] && f=trim.montagent.json
    rm -f "$T/cost-$p.mp4"
    load=$(sysctl -n vm.loadavg | awk '{print $2}')
    s=$(python3 -c 'import time; print(time.time())')
    MONTAGENT_PAINTING=1,1000 "$B" render "$f" --output "$T/cost-$p.mp4" >"$T/cost-$p.out" 2>"$T/cost-$p.err" || tail -3 "$T/cost-$p.err"
    e=$(python3 -c 'import time; print(time.time())')
    python3 -c "print(f'run $i  $p  load1 $load  {($e - $s):.2f} s  {($e - $s) / 690 * 1000:.2f} ms/frame')"
  done
done
rm -f "$T/cost-trim.mp4" "$T/cost-plain.mp4"
