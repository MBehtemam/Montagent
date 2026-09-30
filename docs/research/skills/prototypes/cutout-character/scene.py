"""The scene: a tiny scene graph (camera > fox > parts) animated in Python and flattened.

Montagent has one flat transform per element and no parenting or camera, so every
element's screen transform is computed here, sampled per frame, simplified, and written
as keyframes. `elements()` is shared with ref_render.py (the no-Montagent reference).

Run: python3 scene.py   -> fox.montagent.json
"""
import json
import math
from pathlib import Path

HERE = Path(__file__).parent
RIG = json.loads((HERE / "parts/rig.json").read_text())
PROPS = json.loads((HERE / "parts/props.json").read_text())
FPS, W, H, DURATION = 30, 1920, 1080, 10500
# Frame instants are fractional ms at 30 fps (frame 263 is 8766.67), so they stay floats.
FRAMES = [i * 1000 / FPS for i in range(math.ceil(DURATION * FPS / 1000) + 1)]

# ---- animation curves -------------------------------------------------------------

def smooth(u):
    u = min(max(u, 0.0), 1.0)
    return u * u * (3 - 2 * u)

def back_out(u, s=1.6):
    u = min(max(u, 0.0), 1.0) - 1
    return u * u * ((s + 1) * u + s) + 1

def curve(keys, t):
    """keys: [(t, value, easefn)]; eased between neighbours, clamped at both ends."""
    if t <= keys[0][0]:
        return keys[0][1]
    for (t0, v0, _), (t1, v1, ease) in zip(keys, keys[1:]):
        if t <= t1:
            return v0 + (v1 - v0) * ease((t - t0) / (t1 - t0))
    return keys[-1][1]

S, B = smooth, back_out
CUT = 7900  # wide shot -> close-up

# Upper-arm and forearm angles, degrees clockwise; 0 is the drawn rest pose.
UPPER = [(0, 0, S), (1300, 0, S), (1700, -58, B), (2850, -58, S), (3300, 0, S),
         (4300, 0, S), (4700, -34, B), (5450, -34, S), (5800, -62, B), (6900, -62, S),
         (7300, -62, S), (7650, 0, S)]
FORE = [(0, 0, S), (1300, 0, S), (1700, -92, B), (2850, -92, S), (3300, 0, S),
        (4300, 0, S), (4700, -30, B), (5450, -30, S), (5800, -48, B), (6900, -48, S),
        (7300, -48, S), (7650, 0, S)]
HEAD = [(0, 0, S), (330, -5, S), (650, 0, B), (4300, 0, S), (4700, 4, S), (6900, 4, S),
        (7300, 4, S), (7420, -4, S), (7800, -2, S), (CUT, -2, S), (8250, 5, S)]
CAM = [  # (t, focus_x, focus_y, zoom)
    (0, 960, 540, 1.0), (3900, 960, 540, 1.0), (7200, 930, 590, 1.12),
]
CLOSE = (575, 385, 2.3)  # after the cut
TOAST_Y = [(0, 0, S), (7300, 0, S), (7520, -1.0, B), (10500, -1.0, S)]  # 0 hidden, -1 popped

def wave(t):
    """Forearm wave on top of FORE while the hand is up."""
    if 1750 <= t <= 2750:
        env = smooth((t - 1750) / 150) * smooth((2750 - t) / 150)
        return 17 * env * math.sin((t - 1750) / 300 * 2 * math.pi)
    return 0.0

def nod(t):
    """Small head accents on stressed words."""
    total = 0.0
    for at in (1600, 3050, 4550, 5720, 8360):
        u = (t - at) / 260
        if 0 <= u <= 1:
            total += 2.2 * math.sin(u * math.pi)
    return total

def camera(t):
    if t >= CUT:
        fx, fy, z = CLOSE
        return fx, fy, z * (1 + 0.04 * smooth((t - CUT) / (DURATION - CUT)))
    for (t0, *a), (t1, *b) in zip(CAM, CAM[1:]):
        if t <= t1:
            u = smooth((t - t0) / (t1 - t0)) if t >= t0 else 0
            return tuple(x + (y - x) * u for x, y in zip(a, b))
    return tuple(CAM[-1][1:])

# ---- scene graph ------------------------------------------------------------------

FOX_FEET, FOX_AT, FOX_K = (512, 1500), (560, 1012), 0.58
MACHINE_AT, MACHINE_H = (1430, 1000), 430  # bottom-centre on the floor
M_K = MACHINE_H / PROPS["machine"]["h"]
T_K = 150 / PROPS["toast"]["w"]
TOAST_X = MACHINE_AT[0] - 12

def rot(v, deg):
    r = math.radians(deg)
    c, s = math.cos(r), math.sin(r)
    return (v[0] * c - v[1] * s, v[0] * s + v[1] * c)

def fox_world(m):
    """Master-pixel point -> world point."""
    return (FOX_AT[0] + FOX_K * (m[0] - FOX_FEET[0]), FOX_AT[1] + FOX_K * (m[1] - FOX_FEET[1]))

def to_screen(p, t):
    fx, fy, z = camera(t)
    return (W / 2 + z * (p[0] - fx), H / 2 + z * (p[1] - fy)), z

def pose(t):
    """World transform of every node at t: {name: (x, y, rotation, scale)} in screen space."""
    out = {}
    def put(name, world_pt, rotation, k):
        (x, y), z = to_screen(world_pt, t)
        out[name] = (x, y, rotation, k * z)

    put("bg", (W / 2, H / 2), 0.0, 1.0)
    mw = PROPS["machine"]
    put("machine", (MACHINE_AT[0], MACHINE_AT[1] - mw["h"] * M_K / 2), 0.0, M_K)
    top = MACHINE_AT[1] - MACHINE_H
    th = PROPS["toast"]["h"] * T_K
    hidden_y, popped_y = top + th / 2 + 30, top - th / 2 + 40
    ty = hidden_y + (popped_y - hidden_y) * -curve(TOAST_Y, t)
    put("toast", (TOAST_X, ty), 0.0, T_K)

    put("torso", fox_world(RIG["torso"]["pivot"]), 0.0, FOX_K)
    put("head", fox_world(RIG["head"]["pivot"]), curve(HEAD, t) + nod(t), FOX_K)
    tu = curve(UPPER, t)
    sh = RIG["upper_arm"]["pivot"]
    put("upper_arm", fox_world(sh), tu, FOX_K)
    el = RIG["forearm"]["pivot"]
    d = rot((el[0] - sh[0], el[1] - sh[1]), tu)
    put("forearm", fox_world((sh[0] + d[0], sh[1] + d[1])), tu + curve(FORE, t) + wave(t), FOX_K)
    return out

# ---- the lines and the mouth ------------------------------------------------------

LINES = [  # (file, timeline start, source_end): trailing silence trimmed
    ("voice/line-1.wav", 300, 3400, "voice/line-1.json"),
    ("voice/line-2.wav", 3900, 3250, "voice/line-2.json"),
    ("voice/line-3.wav", 8300, 800, "voice/line-3.json"),
]
SHAPE = {}
for ids, shape in (((1, 2, 9, 11), "A"), ((3, 7, 8, 10, 13, 16), "O"),
                   ((4, 5, 6, 12, 14, 17, 19, 20), "S"), ((15, 18), "E"), ((21,), "M")):
    for i in ids:
        SHAPE[i] = shape

def mouth_spans():
    """[(start, end, shape)] on the timeline; silence = no overlay (the drawn smile)."""
    spans = []
    for _, at, src_end, meta in LINES:
        vis = json.loads((HERE / meta).read_text())["visemes"]
        for (t0, v), nxt in zip(vis, vis[1:] + [[src_end, 0]]):
            s, e = at + t0, at + min(nxt[0], src_end)
            if e <= s:
                continue
            shape = SHAPE.get(v)
            if v == 0 and e - s < 160 and spans and spans[-1][1] == s:
                shape = "M"  # a short gap mid-phrase closes the lips instead of smiling
            if shape is None:
                continue
            if spans and spans[-1][2] == shape and spans[-1][1] == s:
                spans[-1] = (spans[-1][0], e, shape)
            else:
                spans.append((s, e, shape))
    # Snap to frames and enforce a 2-frame minimum by merging into the previous span.
    # floor, not round: a range [start, end) must start at or before the frame's instant,
    # or that frame shows the previous mouth (round() made ~1/3 of the changes a frame late).
    frame = lambda ms: math.floor(round(ms * FPS / 1000) * 1000 / FPS)
    out = []
    for s, e, shape in spans:
        s, e = frame(s), frame(e)
        if out and s < out[-1][1]:
            s = out[-1][1]
        if e - s < 66:
            if out and out[-1][1] == s:
                out[-1] = (out[-1][0], e, out[-1][2])
            continue
        out.append((s, e, shape))
    return out

BLINKS = [(1000, 1133), (3500, 3633), (6300, 6433), (9300, 9533)]

def elements():
    """[(id, file, (w, h), layer, start, end)] — what is drawn, in back-to-front layers."""
    els = [("bg", "parts/bg.png", (1920, 1080), 0, 0, DURATION),
           ("toast", "parts/toast.png", (PROPS["toast"]["w"], PROPS["toast"]["h"]), 1, 0, DURATION),
           ("machine", "parts/machine.png", (PROPS["machine"]["w"], PROPS["machine"]["h"]), 2, 0, DURATION)]
    for i, (name, layer) in enumerate((("torso", 3), ("head", 4), ("forearm", 7), ("upper_arm", 8))):
        r = RIG[name]
        els.append((name, r["file"], (r["w"], r["h"]), layer, 0, DURATION))
    hr = RIG["head"]
    for n, (s, e, shape) in enumerate(mouth_spans()):
        els.append((f"mouth-{n}", RIG[f"mouth_{shape}"]["file"], (hr["w"], hr["h"]), 5, s, e))
    for n, (s, e) in enumerate(BLINKS):
        els.append((f"blink-{n}", RIG["eyes_closed"]["file"], (hr["w"], hr["h"]), 6, s, e))
    return els

def node_of(el_id):
    return "head" if el_id.startswith(("mouth-", "blink-")) else el_id

# ---- flatten to Montagent -----------------------------------------------------------

def simplify(samples, tol):
    """Greedy: keep a sample only where linear interpolation from the last kept one misses by > tol."""
    keep = [samples[0]]
    i = 0
    while i < len(samples) - 1:
        j = i + 1
        while j + 1 < len(samples):
            (t0, v0), (t1, v1) = samples[i], samples[j + 1]
            if all(abs(v0 + (v1 - v0) * (tm - t0) / (t1 - t0) - vm) <= tol for tm, vm in samples[i + 1:j + 1]):
                j += 1
            else:
                break
        keep.append(samples[j])
        i = j
    return keep

def animatable(samples, tol, cast):
    kept = simplify(samples, tol)
    if all(abs(v - kept[0][1]) <= tol for _, v in kept):
        return cast(kept[0][1])
    recs = []
    for n, (t, v) in enumerate(kept):
        rec = {"t": round(t), "v": cast(v)}
        if n:
            rec["ease"] = "linear"
        recs.append(rec)
    return recs

def project():
    poses = {t: pose(t) for t in FRAMES}
    tracks = {}
    for el_id, file, (w, h), layer, start, end in elements():
        node = node_of(el_id)
        ts = [t for t in FRAMES if start - 34 <= t <= end + 34]
        # Every frame is sampled, so simplification never smears the cut across a frame.
        samples = [(t, poses[t][node]) for t in ts]
        el = {"id": el_id, "type": "image", "start": start, "end": end, "source": file,
              "x": animatable([(t, p[0]) for t, p in samples], 0.5, round),
              "y": animatable([(t, p[1]) for t, p in samples], 0.5, round),
              "origin": "center", "width": w, "height": h, "fit": "literal"}
        sc = animatable([(t, p[3]) for t, p in samples], 0.0005, lambda v: round(v, 4))
        el["scale"] = [sc, sc] if not isinstance(sc, list) else [
            {**r, "v": [r["v"], r["v"]]} for r in sc]
        r = animatable([(t, p[2]) for t, p in samples], 0.15, lambda v: round(v, 2))
        if r != 0:
            el["rotation"] = r
        track = el_id.split("-")[0]
        tracks.setdefault(track, {"name": track, "layer": layer, "elements": []})["elements"].append(el)
    tr = list(tracks.values())
    tr.append({"name": "voice", "layer": 20, "elements": [
        {"id": f"line-{n + 1}", "type": "audio", "start": at, "end": at + src_end, "source": f,
         "source_start": 0, "source_end": src_end} for n, (f, at, src_end, _) in enumerate(LINES)]})
    return {"frame": {"width": W, "height": H}, "fps": FPS, "background": "#000000",
            "duration": DURATION, "output": "out/fox.mp4", "tracks": tr}

if __name__ == "__main__":
    doc = project()
    (HERE / "fox.montagent.json").write_text(json.dumps(doc, indent=1))
    n = sum(len(v) if isinstance(v, list) else 0 for tr in doc["tracks"] for el in tr["elements"]
            for k, v in el.items() if k in ("x", "y", "rotation"))
    print("elements", sum(len(t["elements"]) for t in doc["tracks"]), "keyframes(x,y,rot)", n,
          "mouth spans", len(mouth_spans()))
