"""Summarize run_trailer.sh runs: raw-frame hashes against the `none` run, and timings.

    python3 summarize.py <OUT dir> <tag>...

raw:    P649RAW lines (the RGB bytes handed to the encoder, hashed per frame) compared with
        <OUT>/none. Stricter than framemd5, which hashes the encoder's output.
timing: P649 lines, one per element with a blur or shadow: wall ms from opening its first
        filter layer to its last restore (the content draw, the filter and the composite).
"""
import re
import statistics
import sys

out, tags = sys.argv[1], sys.argv[2:]


def read(tag):
    raw, ms, stages = {}, [], ""
    for line in open(f"{out}/{tag}/stderr.txt", errors="replace"):
        if line.startswith("P649RAW\t"):
            _, n, h = line.strip().split("\t")
            raw[int(n)] = h
        elif line.startswith("P649\t"):
            ms.append(float(line.split("\t")[1]))
        elif line.startswith("MONTAGENT_STAGES"):
            stages = line.strip()
    return raw, ms, stages


ref, _, _ = read("none")
for tag in tags:
    raw, ms, stages = read(tag)
    diff = sorted(n for n in ref if raw.get(n) != ref[n])
    load = open(f"{out}/{tag}/load.txt").read()
    wall = re.search(r"wall_s ([\d.]+)", load)
    la = re.findall(r"load averages: ([\d. ]+)", load)
    print(f"{tag}: raw frames {len(raw)}, differing from none {len(diff)} {diff[:8]}")
    if ms:
        print(
            f"  layers {len(ms)}  sum {sum(ms) / 1000:.1f} s  mean {statistics.mean(ms):.2f} ms"
            f"  median {statistics.median(ms):.2f} ms  wall {float(wall.group(1)):.0f} s  load {la}"
        )
    print(f"  {stages}")
