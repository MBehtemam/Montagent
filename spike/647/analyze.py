import sys, collections
P = sys.argv[1]
rows = []
for line in open(P):
    if not line.startswith("P647\t"):
        continue
    f = line.rstrip("\n").split("\t")
    _, inst, name, kind, effects, mbits, content, opacity, px, tr = f
    rows.append(dict(inst=int(inst), name=name, kind=kind, effects=effects,
                     m=mbits.split(","), content=content, opacity=float(opacity),
                     px=int(px), tr=tr))
insts = sorted({r["inst"] for r in rows})
# frame index from instant: render paints frame n at floor/round(n*1000/30)
def frame_of(i):
    return round(i * 30 / 1000)
for r in rows:
    r["f"] = frame_of(r["inst"])
    r["key"] = (r["name"], r["effects"], tuple(r["m"]), r["content"])
    r["key_noname"] = (r["effects"], tuple(r["m"]), r["content"])
frames = collections.defaultdict(list)
for r in rows:
    frames[r["f"]].append(r)
F = sorted(frames)
print("paints", len(rows), "frames with any filtered layer", len(F), "range", F[0], F[-1])
print("layers: blur", sum(r["kind"].count("blur") for r in rows), "shadow", sum(r["kind"].count("shadow") for r in rows))

# unbounded + prev-frame + last-N policies
seen = {}       # key -> last frame used
seen_nn = set()
prev_key = {}   # name -> (frame, key)
hits = collections.Counter()
for f in F:
    for r in frames[f]:
        k = r["key"]
        r["hit_unb"] = k in seen
        r["hit_nn"] = r["key_noname"] in seen_nn
        last = seen.get(k)
        for N in (1, 2, 30):
            r[f"hit_{N}"] = last is not None and f - last <= N
        pk = prev_key.get(r["name"])
        r["hit_prev"] = pk is not None and pk[0] == f - 1 and pk[1] == k
        r["prev"] = pk
    for r in frames[f]:
        seen[r["key"]] = f
        seen_nn.add(r["key_noname"])
        prev_key[r["name"]] = (f, r["key"])
        r["prev_row"] = None
n = len(rows)
for h in ("hit_unb", "hit_nn", "hit_prev", "hit_1", "hit_2", "hit_30"):
    c = sum(r[h] for r in rows)
    print(f"{h:10s} {c:5d} / {n} = {100*c/n:.1f}%")

# per-element previous row for cause analysis
last_row = {}
for f in F:
    for r in frames[f]:
        r["prev_row"] = last_row.get(r["name"])
    for r in frames[f]:
        last_row[r["name"]] = r

def cause(r):
    p = r["prev_row"]
    if p is None:
        return "cold (first paint of element)"
    c = []
    if p["content"] != r["content"]:
        c.append("content")
    if p["effects"] != r["effects"]:
        c.append("effect params")
    m, q = r["m"], p["m"]
    if (m[0], m[1], m[3], m[4]) != (q[0], q[1], q[3], q[4]):
        c.append("scale/rotation")
    if (m[2], m[5]) != (q[2], q[5]):
        whole = all(float.fromhex if False else True for _ in [0])
        c.append("translation")
    if not c:
        return "same as element's last paint, but that was >1 frame ago / key evicted"
    return "+".join(c)

miss_unb = [r for r in rows if not r["hit_unb"]]
print("\nMISS causes (unbounded cache), vs the element's previous paint:")
cc = collections.Counter(cause(r) for r in miss_unb)
for k, v in cc.most_common():
    print(f"  {v:5d}  {k}")

# by element
print("\nBy element: paints, unb hits, prev hits, frames span, miss cause breakdown")
byel = collections.defaultdict(list)
for r in rows:
    byel[r["name"]].append(r)
elrows = []
for name, rs in byel.items():
    hu = sum(r["hit_unb"] for r in rs)
    hp = sum(r["hit_prev"] for r in rs)
    causes = collections.Counter(cause(r) for r in rs if not r["hit_unb"])
    nk = len({r["key"] for r in rs})
    elrows.append((len(rs), name, rs[0]["kind"], hu, hp, nk, min(r["f"] for r in rs), max(r["f"] for r in rs), dict(causes)))
elrows.sort(key=lambda x: -x[0])
for e in elrows:
    print(f"  {e[1]:28s} {e[2]:12s} paints={e[0]:4d} unbHits={e[3]:4d} prevHits={e[4]:4d} keys={e[5]:4d} frames {e[6]}-{e[7]} {e[8]}")

# per-frame density
print("\nFrames by number of filtered element paints:")
dens = collections.Counter(len(frames[f]) for f in F)
bands = [(1, 4), (5, 9), (10, 13), (14, 28), (29, 999)]
for lo, hi in bands:
    fs = [f for f in F if lo <= len(frames[f]) <= hi]
    rs = [r for f in fs for r in frames[f]]
    if not rs:
        continue
    hu = sum(r["hit_unb"] for r in rs); hp = sum(r["hit_prev"] for r in rs)
    print(f"  {lo}-{hi} layers: {len(fs)} frames, {len(rs)} paints, unb hits {hu} ({100*hu/len(rs):.1f}%), prev hits {hp} ({100*hp/len(rs):.1f}%)")
# contiguous runs of frames
print("\nSections (contiguous frame runs, density, hits):")
runs = []
start = F[0]; prevf = F[0]
for f in F[1:] + [None]:
    if f is None or f != prevf + 1:
        runs.append((start, prevf));
        if f is not None: start = f
    if f is not None: prevf = f
for a, b in runs:
    rs = [r for f in range(a, b + 1) for r in frames[f]]
    dmax = max(len(frames[f]) for f in range(a, b + 1))
    hu = sum(r["hit_unb"] for r in rs); hp = sum(r["hit_prev"] for r in rs)
    names = sorted({r["name"] for r in rs})
    print(f"  frames {a}-{b} ({(b-a+1)} fr) max {dmax}/fr, {len(rs)} paints, unb hits {hu}, prev hits {hp}; elems {len(names)}: {', '.join(names[:8])}{'...' if len(names)>8 else ''}")

# memory
distinct = {}
for r in rows:
    distinct.setdefault(r["key"], r["px"])
tot = sum(distinct.values()) * 4
print(f"\nDistinct keys {len(distinct)}; cropped bytes all kept {tot/1e6:.1f} MB; full-frame {len(distinct)*1920*1080*4/1e6:.0f} MB")
print(f"median px per key {sorted(distinct.values())[len(distinct)//2]}, max {max(distinct.values())}")
for N in (1, 2, 30):
    peak_k = peak_b = 0
    for i, f in enumerate(F):
        live = {}
        for g in range(f - N + 1, f + 1):
            for r in frames.get(g, []):
                live[r["key"]] = r["px"]
        peak_k = max(peak_k, len(live)); peak_b = max(peak_b, sum(live.values()) * 4)
    print(f"  keep-last-{N}: peak live keys {peak_k}, peak cropped {peak_b/1e6:.1f} MB, full-frame {peak_k*8.3:.0f} MB")

import struct
def f32(h): return struct.unpack('>f', bytes.fromhex(h))[0]
print("\nHypothetical (NOT byte-safe per #644), for scale only:")
for label, keyf in [
    ("key without translation", lambda r: (r["name"], r["effects"], tuple(r["m"][i] for i in (0,1,3,4,6,7,8)), r["content"])),
    ("key with translation mod 1 px (subpixel phase)", lambda r: (r["name"], r["effects"], tuple(r["m"][i] for i in (0,1,3,4,6,7,8)), round(f32(r["m"][2])%1,6), round(f32(r["m"][5])%1,6), r["content"])),
]:
    s=set(); h=0
    for f in F:
        for r in frames[f]:
            if keyf(r) in s: h+=1
        for r in frames[f]: s.add(keyf(r))
    print(f"  {label}: {h} hits ({100*h/n:.1f}%)")
fr = [r for r in rows if r["prev_row"] is not None and r["m"][2]!=r["prev_row"]["m"][2]]
frac = sum(1 for r in fr if f32(r["m"][2])%1 != 0)
print(f"  translation-changing paints with fractional x: {frac}/{len(fr)}")
tit = [r for r in rows if r["name"].startswith("title-") and not r["name"].startswith("title-soft")]
print("  title scale range", min(f32(r["m"][0]) for r in tit), max(f32(r["m"][0]) for r in tit))
