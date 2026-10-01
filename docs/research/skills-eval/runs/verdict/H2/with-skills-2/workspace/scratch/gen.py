import json, math, subprocess
FPS = 30
def fr(t):  # first drawn instant at or after t
    k = math.ceil(t * FPS / 1000 - 1e-9)
    return math.floor(k * 1000 / FPS)

INK, PAPER, SIGNAL = "#101418", "#F5F0E6", "#FF5A36"
words = json.load(open("presenter/take-3.words.json"))
# pages: (word indices, end ms)
pages = [(range(0, 5), None), (range(5, 8), 2440), (range(8, 11), None), (range(11, 14), None),
         (range(14, 17), 5540), (range(17, 22), None), (range(22, 25), 9000)]
ACCENT = {"mouse.", "truth"}

p = json.load(open("project.json"))
p["fonts"] = {"cap": [{"file": "fonts/Inter-Bold.ttf"}]}

# measure page widths
specs = [{"runs": [{"text": " ".join(words[i]["word"] for i in idx)}], "font": "cap", "size": 64} for idx, _ in pages]
out = subprocess.run(["montagent", "measure", "project.json", "--elements", json.dumps(specs), "--json"], capture_output=True, text=True).stdout
mj = json.loads(out)
def find_extents(o):
    res = []
    if isinstance(o, dict):
        if "extent" in o:
            e = o["extent"]; res.append(e[0] if isinstance(e, list) else e.get("width", e.get("w")))
        else:
            for v in o.values(): res += find_extents(v)
    elif isinstance(o, list):
        for v in o: res += find_extents(v)
    return res
widths = find_extents(mj)
pspecs = []
for idx, _ in pages:
    ws = [words[i]["word"] for i in idx]
    for k in range(1, len(ws)):
        pspecs.append({"runs": [{"text": " ".join(ws[:k]) + " "}], "font": "cap", "size": 64})
pout = subprocess.run(["montagent", "measure", "project.json", "--elements", json.dumps(pspecs), "--json"], capture_output=True, text=True).stdout
pw_list = find_extents(json.loads(pout))
assert len(pw_list) == len(pspecs)
assert len(widths) == len(pages), (out[:2000])

caps, pills, occs = [], [], []
pi = 0
CY = 800
for n, (idx, end) in enumerate(pages):
    idx = list(idx)
    start = fr(words[idx[0]]["start"])
    nxt = pages[n + 1][0][0] if n + 1 < len(pages) else None
    if end is None:
        end = fr(words[nxt]["start"])
    else:
        end = fr(end)
    runs = []
    for j, i in enumerate(idx):
        w = words[i]["word"]
        txt = w + (" " if j < len(idx) - 1 else "")
        r = {"text": txt}
        if w in ACCENT: r["color"] = SIGNAL
        runs.append(r)
    caps.append({"id": f"cap-{n+1}", "type": "text", "start": start, "end": end, "x": 540, "y": CY, "origin": "center",
                 "width": 960, "height": 80, "font": "cap", "size": 64, "color": INK, "align": "center", "runs": runs})
    W = widths[n]
    left = 540 - W / 2
    right = 540 + W / 2 + 6
    ow = right - (left - 4)
    kfs = []
    for k in range(1, len(idx)):
        lk = left + pw_list[pi] - 7; pi += 1
        t = start if k == 1 else fr(words[idx[k - 1]]["start"])
        kf = {"t": t, "v": [round((right - lk) / ow, 4), 1.0]}
        if k > 1: kf["ease"] = "step"
        kfs.append(kf)
    occs.append({"id": f"reveal-{n+1}", "type": "rect", "start": start, "end": fr(words[idx[-1]]["start"]), "x": round(right), "y": CY,
                 "origin": "center-right", "width": round(ow), "height": 96, "fill": "#FFFFFF", "scale": kfs})
    pw = math.ceil(widths[n]) + 64
    pills.append({"id": f"pill-{n+1}", "type": "rect", "start": start, "end": end, "x": 540, "y": CY, "origin": "center",
                  "width": pw, "height": 112, "fill": "#FFFFFF", "radius": 28,
                  "effects": [{"name": "shadow", "dx": 0, "dy": 10, "radius": 28, "color": INK, "opacity": 0.18}]})

PUNCH_IN, PUNCH_OUT = 2700, 6200
def stepkf(a, b):
    return [{"t": 0, "v": a}, {"t": PUNCH_IN, "v": b, "ease": "step"}, {"t": PUNCH_OUT, "v": a, "ease": "step"}]

presenter = {"id": "presenter", "type": "video", "start": 0, "end": 9000, "source": "presenter/take-3.mp4",
             "source_start": 0, "source_end": 8920, "overrun": "hold", "volume": 1.0,
             "x": stepkf(546, 440), "y": stepkf(614, 756), "origin": "center", "width": 1920, "height": 1080, "fit": "cover",
             "scale": stepkf([0.9, 0.9], [1.35, 1.35]),
             "effects": [{"name": "chroma", "color": "#00FF00", "tolerance": 0.2, "softness": 0.08, "spill": 0.9}]}
disc = {"id": "disc", "type": "ellipse", "start": 0, "end": 9000, "x": stepkf(540, 440), "y": stepkf(400, 430),
        "origin": "center", "width": 720, "height": 720, "fill": SIGNAL, "scale": stepkf([1.0, 1.0], [1.2, 1.2])}

C_IN, C_ARRIVE, C_OUT, C_GONE = 3100, 3500, 6000, 6233
FX_OFF, FX_REST = 1110, 622
def cardx(off):
    return [{"t": C_IN, "v": FX_OFF + off}, {"t": C_ARRIVE, "v": FX_REST + off, "ease": [0.25, 1, 0.5, 1]},
            {"t": C_OUT, "v": FX_REST + off, "ease": "linear"}, {"t": C_GONE, "v": FX_OFF + off, "ease": "ease-in"}]
card_frame = {"id": "card-frame", "type": "rect", "start": C_IN, "end": C_GONE + 1, "x": cardx(0), "y": 80, "origin": "top-left",
              "width": 416, "height": 241, "fill": "#FFFFFF", "radius": 28,
              "effects": [{"name": "shadow", "dx": 0, "dy": 14, "radius": 32, "color": INK, "opacity": 0.25}]}
card = {"id": "card", "type": "video", "start": C_IN, "end": C_GONE + 1, "source": "screen/session.mp4",
        "source_start": 30000, "source_end": 30000 + (C_GONE + 1 - C_IN), "volume": 0.0,
        "x": cardx(8), "y": 88, "origin": "top-left", "width": 640, "height": 360, "fit": "cover",
        "effects": [{"name": "mask", "shape": "rect", "x": 0, "y": 0, "width": 400, "height": 225, "radius": 20}]}

endcard = {"id": "lockup", "type": "image", "start": 9000, "end": 10000, "source": "brand/lockup.png",
           "x": 540, "y": 540, "origin": "center", "width": 800, "height": 185, "fit": "contain",
           "scale": [{"t": 9000, "v": [1.0, 1.0]}, {"t": 9966, "v": [1.04, 1.04], "ease": "linear"}]}

bed = {"id": "bed", "type": "audio", "start": 0, "end": 10000, "source": "music/bed-120bpm.wav", "source_start": 0, "source_end": 10000,
       "volume": [{"t": 0, "v": 0.18}, {"t": 5040, "v": 0.18, "ease": "linear"}, {"t": 5240, "v": 0.5, "ease": "ease-in-out"},
                  {"t": 5920, "v": 0.5, "ease": "linear"}, {"t": 6120, "v": 0.18, "ease": "ease-in-out"},
                  {"t": 8440, "v": 0.18, "ease": "linear"}, {"t": 8700, "v": 0.7, "ease": "ease-in-out"},
                  {"t": 9000, "v": 0.7, "ease": "linear"}, {"t": 9966, "v": 0.0, "ease": "linear"}]}

p["tracks"] = [
    {"name": "music", "layer": 0, "elements": [bed]},
    {"name": "disc", "layer": 5, "elements": [disc]},
    {"name": "presenter", "layer": 10, "elements": [presenter]},
    {"name": "card-frame", "layer": 40, "elements": [card_frame]},
    {"name": "card", "layer": 41, "elements": [card]},
    {"name": "captions-bg", "layer": 50, "elements": pills},
    {"name": "captions", "layer": 51, "elements": caps},
    {"name": "captions-reveal", "layer": 52, "elements": occs},
    {"name": "endcard", "layer": 60, "elements": [endcard]},
]
json.dump(p, open("project.json", "w"), indent=2, ensure_ascii=False)
