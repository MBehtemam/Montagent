"""Writes social.json: Brief B, the 10 s vertical talking-head cut.

Run it, then `montagent fmt social.json`. Caption boxes come from `montagent measure`.
"""
import json
import math
import subprocess

PROJECT = "social.json"
INK, PAPER, SIGNAL = "#101418", "#F5F0E6", "#FF5A36"
FPS = 30


def frame_ms(k):
    # Montagent samples frame k at floor(k * 1000 / fps) ms.
    return (k * 1000) // FPS


p = json.load(open(PROJECT))
p["fonts"] = {
    "bold": [{"file": "fonts/Inter-Bold.ttf"}],
    "regular": [{"file": "fonts/Inter-Regular.ttf"}],
}
tracks = []

# --- Presenter, keyed over the Signal background -------------------------------------
# One key cannot serve the whole take: a key strong enough to clear the 1-2 px band where
# the red sweater meets the screen also eats the black hair, the white collar and skin.
# So the full frame carries the strong key (and the voice), and silent copies with a
# gentle key are clipped over the head, collar, hands and trousers.
def presenter(ident, tolerance, softness, volume, clip=None):
    e = {
        "id": ident, "type": "video", "start": 0, "end": 10000,
        "source": "presenter/take-1.mp4", "source_start": 0, "source_end": 9760,
        "x": 540, "y": 1920, "origin": "bottom-center", "width": 2880, "height": 1620,
        "fit": "literal",
    }
    if clip:
        e["clip"] = clip
    e.update({"overrun": "hold", "volume": volume, "effects": [
        {"name": "chroma", "color": "#00FF00", "tolerance": tolerance, "softness": softness, "spill": 1.0},
    ]})
    return e


tracks.append({"name": "presenter", "layer": 10, "elements": [presenter("presenter", 0.3, 0.1, 1.0)]})
DETAIL = {
    "head": [270, 280, 540, 515],
    "collar": [398, 795, 244, 115],
    "hands": [360, 1290, 320, 280],
    "trousers": [210, 1785, 660, 135],
}
for name, clip in DETAIL.items():
    tracks.append({"name": f"presenter-{name}", "layer": 11,
                   "elements": [presenter(f"presenter-{name}", 0.28, 0.04, 0.0, clip)]})

# --- Channel bug: lockup on an Ink pill, top left ------------------------------------
tracks.append({"name": "bug-pill", "layer": 20, "elements": [{
    "id": "bug-pill", "type": "rect", "start": 0, "end": 10000,
    "x": 60, "y": 90, "origin": "top-left", "width": 272, "height": 88,
    "fill": INK, "radius": 44,
}]})
tracks.append({"name": "bug-lockup", "layer": 21, "elements": [{
    "id": "bug-lockup", "type": "image", "start": 0, "end": 10000,
    "source": "brand/lockup-on-dark.png", "x": 196, "y": 134, "origin": "center",
    "width": 208, "height": 48, "fit": "literal",
}]})

# --- Captions: one element per phrase, one run per word, highlight = spoken window ----
words = json.load(open("presenter/take-1.words.json"))
phrases = [(0, 4), (4, 7), (7, 13), (13, 16), (16, 19), (19, 22), (22, 25)]
CAP_Y, CAP_SIZE, PAD_X, PAD_Y = 1265, 58, 36, 24

specs = []
for a, b in phrases:
    runs = []
    for i in range(a, b):
        w = words[i]
        text = w["word"] + (" " if i < b - 1 else "")
        runs.append({"text": text, "highlight": {"start": w["start"], "end": w["end"], "color": SIGNAL}})
    specs.append({"runs": runs, "font": "bold", "size": CAP_SIZE, "line_height": 1.0})

m = json.loads(subprocess.run(
    ["montagent", "measure", PROJECT, "--json", "--elements", json.dumps(specs)],
    capture_output=True, text=True, check=True).stdout)
extents = []
for r in m["measure"]["results"]:
    r = r["ok"]
    extents.append((math.ceil(r["extent"]["width"]), r["block_height"]))

panels, captions = [], []
for n, ((a, b), spec, (tw, th)) in enumerate(zip(phrases, specs, extents)):
    start = words[a]["start"]
    end = words[b]["start"] if b < len(words) else 10000
    pop = [{"t": start, "v": [0.9, 0.9]}, {"t": start + 120, "v": [1.0, 1.0], "ease": "ease-out"}]
    panels.append({
        "id": f"cap-panel-{n + 1}", "type": "rect", "start": start, "end": end,
        "x": 540, "y": CAP_Y, "origin": "center", "width": tw + 2 * PAD_X, "height": th + 2 * PAD_Y,
        "fill": INK, "radius": 28, "scale": pop,
    })
    captions.append({
        "id": f"cap-{n + 1}", "type": "text", "start": start, "end": end,
        "x": 540, "y": CAP_Y, "origin": "center", "width": tw + 4, "height": th,
        "font": "bold", "size": CAP_SIZE, "line_height": 1.0, "color": PAPER, "align": "center",
        "runs": spec["runs"], "scale": pop,
    })
tracks.append({"name": "caption-panels", "layer": 30, "elements": panels})
tracks.append({"name": "captions", "layer": 31, "elements": captions})

# --- Name bar: a true wipe, one frame-long slice per frame through a growing clip -----
BAR_X, BAR_Y, BAR_W, BAR_H = 60, 1450, 620, 180


def bar_slice(ident, start, end, width):
    return {
        "id": ident, "type": "image", "start": start, "end": end,
        "source": "gfx/namebar.png", "x": BAR_X, "y": BAR_Y, "origin": "top-left",
        "width": BAR_W, "height": BAR_H, "fit": "literal",
        "clip": [BAR_X, BAR_Y, width, BAR_H],
        "effects": [{"name": "shadow", "dx": 0, "dy": 10, "radius": 24, "color": INK, "opacity": 0.25}],
    }


bar = []
ON_FRAME, ON_LEN = 30, 9       # 1000 ms, nine frames, ease-out cubic
for i in range(ON_LEN):
    prog = 1 - (1 - (i + 1) / ON_LEN) ** 3
    k = ON_FRAME + i
    bar.append(bar_slice(f"namebar-on-{i + 1}", frame_ms(k), frame_ms(k + 1), max(1, round(BAR_W * prog))))
OFF_FRAME, OFF_LEN = 180, 8    # 6000 ms, eight frames, ease-in quadratic, right edge retracts
bar.append(bar_slice("namebar", frame_ms(ON_FRAME + ON_LEN), frame_ms(OFF_FRAME), BAR_W))
bar[-1].pop("clip")
for i in range(OFF_LEN - 1):
    prog = 1 - ((i + 1) / OFF_LEN) ** 2
    k = OFF_FRAME + i
    bar.append(bar_slice(f"namebar-off-{i + 1}", frame_ms(k), frame_ms(k + 1), max(1, round(BAR_W * prog))))
tracks.append({"name": "namebar", "layer": 35, "elements": bar})

# --- Circular picture-in-picture of the product ---------------------------------------
PIP_IN, PIP_SETTLE, PIP_HOLD, PIP_OUT = 5000, 5400, 9150, 9450
PIP_GONE = 9433  # the last sampled frame inside the element, so the pop-out lands on 0
BACK_OUT = [0.34, 1.56, 0.64, 1.0]   # overshoots to ~1.1 then settles
BACK_IN = [0.36, 0.0, 0.66, -0.56]
CX, CY, D = 810, 960, 400


def pop_track(v0, v1):
    return [
        {"t": PIP_IN, "v": v0},
        {"t": PIP_SETTLE, "v": v1, "ease": BACK_OUT},
        {"t": PIP_HOLD, "v": v1, "ease": "step"},
        {"t": PIP_GONE, "v": v0, "ease": BACK_IN},
    ]


# The source is drawn at 0.7x (1344x756); the circle sits over the project-file diff,
# whose centre is at (350, 326) in the element, (-322, -52) from the box centre. The box
# pivots about its centre, so its position rides the same curve as its scale and the
# circle's own centre stays fixed at (CX, CY) through the overshoot.
OFF_X, OFF_Y = -322, -52
tracks.append({"name": "pip-ring", "layer": 40, "elements": [{
    "id": "pip-ring", "type": "ellipse", "start": PIP_IN, "end": PIP_OUT,
    "x": CX, "y": CY, "origin": "center", "width": D + 24, "height": D + 24, "fill": PAPER,
    "scale": pop_track([0.0, 0.0], [1.0, 1.0]),
    "effects": [{"name": "shadow", "dx": 0, "dy": 14, "radius": 30, "color": INK, "opacity": 0.35}],
}]})
tracks.append({"name": "pip", "layer": 41, "elements": [{
    "id": "pip", "type": "video", "start": PIP_IN, "end": PIP_OUT,
    "source": "screen/session.mp4", "source_start": 34000, "source_end": 34000 + PIP_OUT - PIP_IN,
    "x": pop_track(CX, CX - OFF_X), "y": pop_track(CY, CY - OFF_Y), "origin": "center",
    "width": 1344, "height": 756, "fit": "literal",
    "scale": pop_track([0.0, 0.0], [1.0, 1.0]),
    "effects": [{"name": "mask", "shape": "circle", "x": 672 + OFF_X - D // 2, "y": 378 + OFF_Y - D // 2,
                 "width": D, "height": D}],
}]})

# --- Music: ducked under speech, lifted in the pauses, faded out by 9.9 s -------------
LOW, LIFT = 0.2, 0.5
vol = [
    {"t": 0, "v": 0.0},
    {"t": 150, "v": LOW, "ease": "ease-out"},
    {"t": 1760, "v": LOW, "ease": "step"},
    {"t": 1960, "v": 0.36, "ease": "ease-in-out"},
    {"t": 2220, "v": 0.36, "ease": "step"},
    {"t": 2400, "v": LOW, "ease": "ease-in-out"},
    {"t": 4380, "v": LOW, "ease": "step"},
    {"t": 4650, "v": LIFT, "ease": "ease-in-out"},
    {"t": 5380, "v": LIFT, "ease": "step"},
    {"t": 5600, "v": LOW, "ease": "ease-in-out"},
    {"t": 7840, "v": LOW, "ease": "step"},
    {"t": 8030, "v": 0.36, "ease": "ease-in-out"},
    {"t": 8200, "v": 0.36, "ease": "step"},
    {"t": 8380, "v": LOW, "ease": "ease-in-out"},
    {"t": 9480, "v": LOW, "ease": "step"},
    {"t": 9900, "v": 0.0, "ease": "ease-in"},
]
tracks.append({"name": "music", "layer": 0, "elements": [{
    "id": "music", "type": "audio", "start": 0, "end": 10000,
    "source": "music/bed-120bpm.wav", "source_start": 0, "source_end": 10000, "volume": vol,
}]})

p["tracks"] = tracks
json.dump(p, open(PROJECT, "w"), indent=2, ensure_ascii=False)
print("wrote", PROJECT)
