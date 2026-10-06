#!/usr/bin/env python3
"""PROTOTYPE #718 — cost per moving element per frame at 1080p, samples 8/16/32 vs none.

    python3 prototypes/motion-blur/cost.py <montagent binary> [runs]

Each cost project paints four moving elements for 60 frames on one painter
(MONTAGENT_PAINTING=1,1000). The painter's own time comes from MONTAGENT_STAGES's `paint`.
Taken under load: the 1-minute load average is recorded before every run.
"""
import json
import os
import pathlib
import statistics
import subprocess
import sys

HERE = pathlib.Path(__file__).resolve().parent
BIN = sys.argv[1]
RUNS = int(sys.argv[2]) if len(sys.argv) > 2 else 3
ELEMENTS, FRAMES = 4, 60


def load1():
    return float(os.getloadavg()[0])


def stages(project):
    env = dict(os.environ, MONTAGENT_PAINTING="1,1000", MONTAGENT_STAGES="1")
    out = subprocess.run([BIN, "render", str(HERE / project)], env=env, capture_output=True,
                         text=True)
    for line in out.stderr.splitlines():
        if line.startswith("MONTAGENT_STAGES "):
            return json.loads(line.split(" ", 1)[1])
    raise SystemExit(f"no stages for {project}: {out.stderr[-2000:]}\n{out.stdout[-2000:]}")


rows = []
for kind in (sys.argv[3].split(",") if len(sys.argv) > 3 else ("cost", "cost-fx", "cost-text")):
    base = None
    for n in (0, 8, 16, 32):
        project = f"{kind}-{n}.montagent.json"
        paints, loads = [], []
        for _ in range(RUNS):
            loads.append(load1())
            s = stages(project)
            paints.append(s["paint_ms"] if "paint_ms" in s else s["paint"])
        med = statistics.median(paints)
        if n == 0:
            base = med
        per = med / (ELEMENTS * FRAMES)
        row = {"kind": kind, "samples": n, "paint_ms_median": med, "paint_ms_runs": paints,
               "ms_per_element_frame": round(per, 2),
               "extra_ms_per_element_frame": round((med - base) / (ELEMENTS * FRAMES), 2),
               "load1_before_runs": loads}
        rows.append(row)
        print(json.dumps(row), flush=True)
