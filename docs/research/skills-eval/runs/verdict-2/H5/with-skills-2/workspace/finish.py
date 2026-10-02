import json, sys, copy
p = json.load(open(sys.argv[1]))
SIG = "#FF5A36"
END_FADE = [{"t": 9520, "v": 1.0}, {"t": 9760, "v": 0.0, "ease": "ease-in"}]
POP = [{"t": 6720, "v": [0.86, 0.86]}, {"t": 7040, "v": [1.0, 1.0], "ease": [0.34, 1.56, 0.64, 1]}]
STICKY = {"exactly", "one", "thing", "nothing", "else", "moved."}
base = None
for t in p["tracks"]:
    if t["name"] in ("captions", "captions-bg"):
        t["elements"] = [e for e in t["elements"] if not e["id"].endswith("-04")]
    if t["name"] in ("payoff", "payoff-bg"):
        for e in t["elements"]:
            e["end"] = 9800
            e["scale"] = POP
            e["opacity"] = END_FADE
            if e["type"] == "text":
                base = e
# Key words stay lit once spoken: an overlay with the same runs, transparent except the
# key words already said, cut each time another key word finishes.
words = [r for r in base["runs"] if r.get("highlight")]
ends = sorted({r["highlight"]["end"] for r in words if r["text"] in STICKY})
overlay = []
for i, t0 in enumerate(ends):
    t1 = ends[i + 1] if i + 1 < len(ends) else 9800
    e = copy.deepcopy(base)
    e["id"] = f"payoff-lit-{i+1:02d}"
    e["start"], e["end"] = t0, t1
    e["color"] = "#F5F0E600"
    for r in e["runs"]:
        h = r.pop("highlight", None)
        if h and r["text"] in STICKY and h["end"] <= t0:
            r["color"] = SIG
    e["caption"] = False
    overlay.append(e)
p["tracks"].append({"name": "payoff-lit", "layer": 33, "elements": overlay})

# --- brand backdrop collapses into the lockup's mark for the close ---
E = [0.65, 0, 0.35, 1]
K = 800 / 2694                      # lockup drawn 800 wide, centred at (540, 675)
LX, LY = 540 - 400, 675 - 623 * K / 2
def mark_centre(a, b):              # tile spans inside the lockup PNG
    return LX + (a + b) / 2 * K
MX = [mark_centre(57, 300), mark_centre(322, 566)]
MY = [LY + (57 + 300) / 2 * K, LY + (322 + 565) / 2 * K]
S = 244 * K / 470
MOVE0, MOVE1, LOCK0, LOCK1 = 9760, 10360, 10200, 10440
for t in p["tracks"]:
    for e in t["elements"]:
        if e["id"] in ("bg-tile-0", "bg-circle", "bg-tile-2", "bg-tile-3"):
            c, r = (e["x"] - 50) // 510, (e["y"] - 150) // 510
            cx, cy = e["x"] + 235, e["y"] + 235
            e["origin"] = "center"
            e["x"] = [{"t": 0, "v": cx}, {"t": MOVE0, "v": cx, "ease": "linear"}, {"t": MOVE1, "v": round(MX[c]), "ease": E}]
            e["y"] = [{"t": 0, "v": cy}, {"t": MOVE0, "v": cy, "ease": "linear"}, {"t": MOVE1, "v": round(MY[r]), "ease": E}]
            e["scale"] = [{"t": 0, "v": [1.0, 1.0]}, {"t": MOVE0, "v": [1.0, 1.0], "ease": "linear"}, {"t": MOVE1, "v": [round(S, 4), round(S, 4)], "ease": E}]
            e["end"] = LOCK1
            if e["type"] == "ellipse":
                e["fill"] = SIG
        if e["id"] == "lockup":
            e["start"] = LOCK0
            e.pop("scale", None)
            e["opacity"] = [{"t": LOCK0, "v": 0.0}, {"t": LOCK1, "v": 1.0, "ease": "ease-out"}]
        # name bar: wider, larger second line
        if e["id"] == "bar":
            e["width"] = 650
        if e["id"] == "bar-line":
            e["size"] = 34; e["height"] = 41; e["width"] = 480; e["y"] = 834
        if e["id"] == "bar-name":
            e["size"] = 58; e["y"] = 772
        if e["id"].startswith("bar-cover"):
            e["width"] = 610
            if e["id"] == "bar-cover-on":
                e["x"] = 64 + 650 - 20
json.dump(p, open(sys.argv[2], "w"), indent=2)
for t in p["tracks"]:
    for e in t["elements"]:
        for fx in e.get("effects", []):
            if fx["name"] == "chroma":
                fx["tolerance"], fx["softness"] = 0.26, 0.11
json.dump(p, open(sys.argv[2], "w"), indent=2)
