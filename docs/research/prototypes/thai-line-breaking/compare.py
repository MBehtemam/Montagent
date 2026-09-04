#!/usr/bin/env python3
"""Compare each stack's break-opportunity set against the CFStringTokenizer oracle.

The oracle is a *word* segmenter and the stacks are *line-break* segmenters, so
the two are not required to agree exactly: a line breaker may legitimately offer
finer boundaries inside a compound (nak|rian inside nakrian). What it must never
do is offer a boundary the oracle rejects *and* that splits a word -- so the two
directions are reported separately and neither is scored as a single number.
"""
import re, sys, pathlib

RES = pathlib.Path("results")

def parse_probe(path):
    out, cur = {}, None
    for line in path.read_text().splitlines():
        m = re.match(r"^## (\S+) ", line)
        if m:
            cur = m.group(1)
        elif line.startswith("breaks:") and cur:
            out[cur] = eval(line.split(":", 1)[1].strip())
    return out

def parse_oracle(path):
    out, cur, seg = {}, None, {}
    for line in path.read_text().splitlines():
        m = re.match(r"^## (\S+) ", line)
        if m:
            cur = m.group(1)
        elif line.startswith("segmentation:") and cur:
            seg[cur] = line.split(":", 1)[1].strip()
        elif line.startswith("starts:") and cur:
            out[cur] = eval(line.split(":", 1)[1].strip())
    return out, seg

oracle, oracle_seg = parse_oracle(RES / "oracle-cfstringtokenizer.txt")
stacks = {
    "cosmic-text": parse_probe(RES / "cosmic-opportunities.txt"),
    "parley (flag off)": parse_probe(RES / "parley-off-opportunities.txt"),
    "parley (flag on)": parse_probe(RES / "parley-on-opportunities.txt"),
}

for sample in oracle:
    print(f"## {sample}")
    print(f"oracle (CFStringTokenizer): {oracle_seg[sample]}")
    print(f"oracle boundaries: {oracle[sample]}")
    for name, data in stacks.items():
        got = data.get(sample)
        if got is None:
            continue
        o, g = set(oracle[sample]), set(got)
        print(
            f"  {name:<20} offered {len(g):>3}  "
            f"missing {sorted(o - g)}  extra {sorted(g - o)}"
        )
    print()
