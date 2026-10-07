#!/bin/bash
# PROTOTYPE #764: how far rigid bodies lift off the curve (ADR-0161 §3), from inside the
# painter (MONTAGENT_PROTO_LIFT): for each drawn body, the larger distance from either end of its
# baseline (its midpoint +/- half its advance along the tangent) to the curve, the curve
# sampled every 0.25 px of its length. Every frame of the clip; the worst per element and body.
set -u
cd "$(dirname "$0")"
B=${MONTAGENT_BIN:-montagent}
T=${PROTO_SCRATCH:-/tmp}/tp764-lift
mkdir -p "$T"
rm -f "$T/lift.log"
MONTAGENT_PAINTING=1,1000 MONTAGENT_PROTO_LIFT="$T/lift.log" "$B" render textpath.montagent.json \
  --output "$T/lift.mp4" >/dev/null 2>&1
rm -f "$T/lift.mp4"
python3 - "$T/lift.log" <<'EOF'
import re, sys
worst = {}
for line in open(sys.argv[1]):
    m = re.match(r"(\d+) (\S+) letters=\[([0-9, ]*)\] advance=([0-9.]+) lift=([0-9.]+)", line)
    t, id, letters, adv, lift = m.groups()
    key = (id, letters)
    if key not in worst or float(lift) > worst[key][1]:
        worst[key] = (float(adv), float(lift), int(t))
per = {}
for (id, letters), (adv, lift, t) in worst.items():
    if id not in per or lift > per[id][2]:
        per[id] = (letters, adv, lift, t)
print("worst lift-off per element over the whole clip (px):")
print(f"{'element':22s} {'body letters':14s} {'advance':>8s} {'lift':>7s}  at")
for id in sorted(per):
    letters, adv, lift, t = per[id]
    print(f"{id:22s} [{letters:12s}] {adv:8.2f} {lift:7.2f}  {t} ms")
print()
print("case 8, every body:")
for (id, letters), (adv, lift, t) in sorted(worst.items()):
    if id.startswith("c8"):
        print(f"{id:22s} [{letters:12s}] advance {adv:7.2f}  lift {lift:6.2f}")
EOF
