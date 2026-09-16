#!/usr/bin/env python3
"""PROTOTYPE — generates the two view conditions for the #11 re-run.

Y1 (spatial) and Y2 (flat) are built from the SAME computed facts and differ
only in whether those facts are laid out on an axis. That is the ablation.
"""
import json, sys
from collections import defaultdict

P = "fixtures/en-halloween-decorating/en-halloween-decorating.montaget.json"
d = json.load(open(P)); DUR = d["duration"]; W = 120
tracks = d["tracks"]
els = [(t, e) for t in tracks for e in t["elements"]]
hdr = [e for _, e in els if e.get("group") == "header"]

# ---- shared facts -------------------------------------------------------
groups = defaultdict(list)
for t, e in els:
    groups[e.get("group", "-")].append(e)
gspan = {g: (min(x["start"] for x in v), max(x["end"] for x in v), len(v))
         for g, v in groups.items() if g != "header"}
gorder = sorted(gspan, key=lambda g: gspan[g][0])

tocc = []
for t in tracks:
    if all(e.get("group") == "header" for e in t["elements"]):
        continue
    es = sorted(t["elements"], key=lambda e: e["start"])
    tocc.append((t["name"], t["layer"], [(e["start"], e["end"], e["id"]) for e in es]))

aud = sorted((e["start"], e["end"]) for _, e in els if e["type"] == "audio")
sil, cur = [], 0
for s, e in aud:
    if s > cur: sil.append((cur, s))
    cur = max(cur, e)
if cur < DUR: sil.append((cur, DUR))

bound = sorted(e["start"] for _, e in els if e.get("group") != "header")
buckets = defaultdict(int)
for s in bound: buckets[s // 5000 * 5000] += 1


def ts(ms): return f"{ms//60000}:{ms%60000/1000:06.3f}"


# ---- Y1: spatial --------------------------------------------------------
def y1():
    o = []
    o.append(f'{d["output"]}   {DUR} ms ({ts(DUR)})   {d["frame"]["width"]}x{d["frame"]["height"]} @{d["fps"]}fps')
    o.append(f'HEADER: {len(hdr)} elements, layers 30-34, all 0..{DUR} unchanging — folded to this line')
    o.append("")
    lw = max(len(n) for n, _, _ in tocc) + 6
    chart = W - lw
    sc = chart / DUR
    ruler = [" "] * chart
    for s in range(0, DUR // 1000 + 1, 5):
        c = int(s * 1000 * sc)
        for i, ch in enumerate(f"{s}s"):
            if c + i < chart: ruler[c + i] = ch
    o.append(" " * lw + "".join(ruler))
    for name, layer, spans in tocc:
        row = ["·"] * chart
        for a, b, _ in spans:
            aa, bb = int(a * sc), max(int(b * sc), int(a * sc) + 1)
            for c in range(aa, min(bb, chart)): row[c] = "█"
            if aa < chart: row[aa] = "▌"
        o.append(f'{("L%d " % layer) + name:<{lw}}' + "".join(row))
    o.append(" " * lw + "".join(ruler))
    o.append("")
    o.append("SEGMENT DURATIONS (bar length ∝ duration)")
    mx = max(v[1] - v[0] for v in gspan.values())
    for g in gorder:
        a, b, n = gspan[g]
        ln = b - a
        o.append(f'  {g:<12}{"#" * int(ln / mx * 60):<62}{ln:>6} ms  {ts(a)}..{ts(b)}  {n} el')
    o.append("")
    o.append("SILENCE (no audio element active)")
    strip = ["─"] * chart
    for a, b in sil:
        for c in range(int(a * sc), min(int(b * sc), chart)): strip[c] = " "
    o.append(" " * lw + "".join(strip))
    o.append(f'{"  audible":<{lw}}' + f'{len(sil)} silent spans, {sum(b-a for a,b in sil)} ms total')
    o.append("")
    o.append("ELEMENT STARTS per aligned 5s bucket (header excluded)")
    for k in sorted(buckets):
        o.append(f'  {k:>6}..{min(k+5000,DUR):<6}{"*" * buckets[k]:<12}{buckets[k]}')
    return "\n".join(o)


# ---- Y2: flat, same facts, no layout ------------------------------------
def y2():
    o = []
    o.append(f'output={d["output"]} duration_ms={DUR} frame={d["frame"]["width"]}x{d["frame"]["height"]} fps={d["fps"]}')
    o.append(f'header_elements={len(hdr)} layers=30-34 span=0..{DUR} unchanging')
    o.append("")
    o.append("TRACK OCCUPANCY (track, layer, then each element start..end)")
    for name, layer, spans in tocc:
        o.append(f'  {name} layer={layer} n={len(spans)}')
        for a, b, i in spans:
            o.append(f'    {i} {a}..{b} ({b-a} ms)')
    o.append("")
    o.append("SEGMENT DURATIONS")
    for g in gorder:
        a, b, n = gspan[g]
        o.append(f'  {g} start={a} end={b} duration_ms={b-a} elements={n}')
    o.append("")
    o.append("SILENCE (no audio element active)")
    o.append(f'  count={len(sil)} total_ms={sum(b-a for a,b in sil)}')
    for a, b in sil:
        o.append(f'  {a}..{b} ({b-a} ms)')
    o.append("")
    o.append("ELEMENT STARTS per aligned 5s bucket (header excluded)")
    for k in sorted(buckets):
        o.append(f'  bucket_start={k} bucket_end={min(k+5000,DUR)} starts={buckets[k]}')
    return "\n".join(o)


open(sys.argv[1], "w").write(y1() + "\n")
open(sys.argv[2], "w").write(y2() + "\n")
