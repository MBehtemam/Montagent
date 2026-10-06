"""PROTOTYPE #722: cost per member per frame at 1080p.

Each cost/<case>.json is one full-frame 1920x1080 gradient rect carrying the member, 30 frames,
painted by one painter. `MONTAGENT_PROTO_COST` times the element's whole effect chain (the
layers, the draw and every filter) per frame. Three runs per case, interleaved; the median of all
frames is reported, with the 1-minute load average read before each run.
"""

import os
import pathlib
import statistics
import subprocess

HERE = pathlib.Path(__file__).parent
BIN = os.environ.get("MONTAGENT_BIN", "montagent")
RUNS = 3


def load():
    return float(os.getloadavg()[0])


cases = sorted(p.stem for p in (HERE / "cost").glob("*.json"))
samples = {c: [] for c in cases}
loads = {c: [] for c in cases}
for run in range(RUNS):
    for case in cases:
        env = dict(os.environ, MONTAGENT_PAINTING="1,1000", MONTAGENT_PROTO_COST="1")
        loads[case].append(load())
        r = subprocess.run([BIN, "render", str(HERE / "cost" / f"{case}.json"), "--output",
                            str(HERE / "out" / "cost.mp4")], env=env, capture_output=True, text=True)
        (HERE / "out" / "cost.mp4").unlink(missing_ok=True)
        for line in r.stderr.splitlines():
            if line.startswith("proto-cost "):
                samples[case].append(int(line.split()[-1]) / 1e6)

base = statistics.median(samples["none"])
lines = [f"1080p, one painter, 30 frames x {RUNS} runs per case; ms per frame for the element's "
         f"effect chain (median, p10-p90). TAKEN UNDER LOAD: 1-min load average per run shown.",
         f"{'case':22s} {'median':>8s} {'p10':>7s} {'p90':>7s} {'vs none':>8s}  load"]
for case in cases:
    s = sorted(samples[case])
    med = statistics.median(s)
    p10, p90 = s[len(s) // 10], s[(len(s) * 9) // 10]
    lines.append(f"{case:22s} {med:8.2f} {p10:7.2f} {p90:7.2f} {med - base:+8.2f}  "
                 + ", ".join(f"{x:.1f}" for x in loads[case]))
text = "\n".join(lines) + "\n"
(HERE / "out" / "cost.txt").write_text(text)
print(text)
