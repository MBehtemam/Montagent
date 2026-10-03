"""Per-frame count of full-frame save_layers the trailer opens, from canvas.rs's rules:
one layer per effect (any effect, mask included, all with no bounds) plus one for opacity < 1.
Opacity is linearly interpolated; easing can't move a value across 1.0 between endpoints <= 1."""
import json, sys, statistics, collections

p = json.load(open(sys.argv[1]))
fps, dur = p["fps"], p["duration"]


def value(v, t):
    if not isinstance(v, list):
        return v
    if t <= v[0]["t"]:
        return v[0]["v"]
    for a, b in zip(v, v[1:]):
        if a["t"] <= t <= b["t"]:
            f = (t - a["t"]) / (b["t"] - a["t"]) if b["t"] > a["t"] else 1
            return a["v"] + (b["v"] - a["v"]) * f
    return v[-1]["v"]


els = [e for tr in p["tracks"] for e in tr["elements"] if e["type"] not in ("audio",)]
rows = []
for n in range(dur * fps // 1000):
    t = n * 1000 / fps
    layers = filt = blurish = elems = 0
    for e in els:
        if not (e["start"] <= t < e["end"]):
            continue
        op = value(e.get("opacity", 1.0), t)
        if op <= 0:
            continue
        elems += 1
        fx = [x["name"] for x in e.get("effects", [])]
        layers += len(fx) + (1 if op < 1 else 0)
        if fx:
            filt += 1
        blurish += sum(1 for x in fx if x in ("blur", "shadow"))
    rows.append((elems, filt, blurish, layers))

for i, name in enumerate(["painted elements", "elements with an effect", "blur/shadow layers", "full-frame layers"]):
    col = [r[i] for r in rows]
    q = statistics.quantiles(col, n=10)
    print(f"{name:26s} median {statistics.median(col):5.1f}  mean {statistics.mean(col):5.1f}  p10 {q[0]:4.0f}  p90 {q[-1]:4.0f}  max {max(col)}")
