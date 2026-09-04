#!/usr/bin/env python3
"""Whole-process cost: spawn, font discovery, lay out every sample at 420px.

The three binaries are interleaved rather than run in three blocks, so machine
drift lands on all of them equally instead of on whichever ran last.
"""
import subprocess, time, statistics

bins = {"parley complex-scripts=off": ("results/parley-probe-off.bin", ["Normal", "420"]),
        "parley complex-scripts=on":  ("results/parley-probe-on.bin",  ["Normal", "420"]),
        "cosmic-text":                ("target/release/cosmic-probe",  ["Word", "420"])}
samples = {k: [] for k in bins}
for _ in range(40):
    for k, (b, a) in bins.items():
        t = time.perf_counter()
        subprocess.run([b] + a, stdout=subprocess.DEVNULL)
        samples[k].append((time.perf_counter() - t) * 1000)

print("# spawn + font discovery + layout of all samples, n=40, interleaved")
for k, v in samples.items():
    print(f"{k:<28} min {min(v):6.1f} ms   median {statistics.median(v):6.1f} ms")
