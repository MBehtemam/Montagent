#!/usr/bin/env python3
"""Writes the tracks of social.montagent.json from the word timings, then `montagent fmt` tidies it.

Everything here is arithmetic that would be error-prone by hand: per-word highlight
windows, per-frame wipe masks for the lower third, and the music ducking envelope.
"""
import json
import math
import subprocess

PROJECT = "social.montagent.json"
FPS = 30
END = 10000

INK, PAPER, SIGNAL = "#101418", "#F5F0E6", "#FF5A36"


def frame_ms(k):
    """Floor of frame k's instant: [frame_ms(k), frame_ms(k+1)) holds exactly frame k."""
    return (k * 1000) // FPS


def ease_out_cubic(p):
    return 1 - (1 - p) ** 3


def ease_in_out_cubic(p):
    return 4 * p ** 3 if p < 0.5 else 1 - (-2 * p + 2) ** 3 / 2


words = json.load(open("presenter/take-1.words.json"))

# ---------------------------------------------------------------- presenter
presenter = {
    "id": "presenter", "type": "video", "start": 0, "end": END,
    "source": "presenter/take-1.mp4", "source_start": 0, "source_end": 9760,
    "x": 540, "y": 1920, "origin": "bottom-center", "width": 2880, "height": 1620, "fit": "literal",
    "overrun": "hold", "volume": 1.45,
    "effects": [{"name": "chroma", "color": "#00FE22", "tolerance": 0.2, "softness": 0.05, "spill": 1.0}],
}

# ---------------------------------------------------------------- brand bug
bug_pill = {"id": "bug-pill", "type": "rect", "start": 0, "end": END,
            "x": 48, "y": 64, "origin": "top-left", "width": 273, "height": 88, "fill": INK, "radius": 44}
bug_logo = {"id": "bug-logo", "type": "image", "start": 0, "end": END, "source": "brand/lockup-on-dark.png",
            "x": 72, "y": 82, "origin": "top-left", "width": 225, "height": 52, "fit": "literal"}

# ---------------------------------------------------------------- lower third
LT_X, LT_Y, LT_W, LT_H = 60, 1236, 624, 184
LT_ON, LT_OFF, WIPE_FRAMES = 1000, 6000, 12

lt_parts = {
    "lt-bar": {"type": "rect", "x": LT_X, "y": LT_Y, "origin": "top-left", "width": LT_W, "height": LT_H,
               "fill": INK, "radius": 20},
    "lt-accent": {"type": "rect", "x": LT_X + 24, "y": LT_Y + 30, "origin": "top-left", "width": 10,
                  "height": LT_H - 60, "fill": SIGNAL, "radius": 5},
    "lt-title": {"type": "text", "x": LT_X + 60, "y": LT_Y + 24, "origin": "top-left", "width": 440, "height": 88,
                 "font": "bold", "size": 80, "line_height": 1.1, "color": PAPER, "align": "start",
                 "runs": [{"text": "Montagent"}]},
    "lt-sub": {"type": "text", "x": LT_X + 60, "y": LT_Y + 116, "origin": "top-left", "width": 520, "height": 48,
               "font": "regular", "size": 40, "line_height": 1.2, "color": PAPER, "align": "start",
               "runs": [{"text": "Video your agent can read"}]},
}
BAR_SHADOW = {"name": "shadow", "dx": 0, "dy": 10, "radius": 28, "color": INK, "opacity": 0.35}


def lt_element(key, suffix, start, end, reveal=None):
    """reveal = (left, right) in frame x: the part of the bar that shows; None = all of it."""
    part = dict(lt_parts[key])
    el = {"id": f"{key}-{suffix}", "type": part.pop("type"), "group": "lower-third", "start": start, "end": end}
    effects = []
    if reveal is not None:
        left, right = reveal
        lx = max(0, min(part["width"], left - part["x"]))
        rx = max(0, min(part["width"], right - part["x"]))
        if rx - lx <= 0:
            return None
        if not (lx == 0 and rx == part["width"]):
            effects.append({"name": "mask", "shape": "rect", "x": lx, "y": 0, "width": rx - lx,
                            "height": part["height"]})
    if key == "lt-bar":
        effects.append(BAR_SHADOW)
    el.update(part)
    if effects:
        el["effects"] = effects
    return el


lt_tracks = {k: [] for k in lt_parts}
on_k, off_k = LT_ON * FPS // 1000, LT_OFF * FPS // 1000
for key in lt_parts:
    els = lt_tracks[key]
    for j in range(WIPE_FRAMES):  # wipe on: the edge sweeps left to right
        edge = LT_X + round(LT_W * ease_out_cubic((j + 1) / WIPE_FRAMES))
        e = lt_element(key, f"on{j:02d}", frame_ms(on_k + j), frame_ms(on_k + j + 1), (LT_X, edge))
        if e:
            els.append(e)
    els.append(lt_element(key, "hold", frame_ms(on_k + WIPE_FRAMES), frame_ms(off_k)))
    for j in range(WIPE_FRAMES):  # wipe off: the edge carries on through to the right
        edge = LT_X + round(LT_W * ease_in_out_cubic((j + 1) / WIPE_FRAMES))
        e = lt_element(key, f"off{j:02d}", frame_ms(off_k + j), frame_ms(off_k + j + 1), (edge, LT_X + LT_W))
        if e:
            els.append(e)

# ---------------------------------------------------------------- captions
PHRASES = [(0, 4, 7), (7, 11, 16), (16, 19, 19), (19, 22, 22), (22, 25, 25)]  # (first, line break, end)
CAP_Y, CAP_SIZE = 1660, 66
def line_texts(a, br, b):
    return [" ".join(w["word"] for w in words[x:y]) for x, y in ((a, br), (br, b)) if y > x]


spec = [{"runs": [{"text": "\n".join(line_texts(a, br, b))}], "font": "bold", "size": CAP_SIZE,
         "line_height": 1.2} for a, br, b in PHRASES]
m = json.loads(subprocess.run(["montagent", "measure", PROJECT, "--elements", json.dumps(spec), "--json"],
                              capture_output=True, text=True, check=True).stdout)
cap_text, cap_pill = [], []
for i, ((a, br, b), r) in enumerate(zip(PHRASES, m["measure"]["results"])):
    start = words[a]["start"]
    end = words[b]["start"] if b < len(words) else END
    runs = []
    for n, w in enumerate(words[a:b]):
        if n:
            runs.append({"text": "\n" if a + n == br else " "})
        runs.append({"text": w["word"], "highlight": {"start": w["start"], "end": w["end"], "color": SIGNAL}})
    width = math.ceil(r["ok"]["advance_width"])
    height = r["ok"]["block_height"]
    cap_pill.append({"id": f"cap-pill-{i + 1}", "type": "rect", "group": f"caption-{i + 1}", "start": start,
                     "end": end, "x": 540, "y": CAP_Y, "origin": "center", "width": width + 84, "height": height + 40,
                     "fill": INK, "radius": 32})
    cap_text.append({"id": f"cap-{i + 1}", "type": "text", "group": f"caption-{i + 1}", "start": start, "end": end,
                     "x": 540, "y": CAP_Y, "origin": "center", "width": width + 8, "height": height, "font": "bold",
                     "size": CAP_SIZE, "line_height": 1.2, "color": PAPER, "align": "center", "runs": runs})

# ---------------------------------------------------------------- picture-in-picture
# The circle is cut around the diff of the project file (source px ~(430, 420); still from 33.6 s to 37.5 s), which sits
# left of the source's centre. Scale pivots on the element's centre, so x/y ride the same keyframes and eases
# as scale, keeping the circle's centre fixed on screen while it pops: centre = P + s * d.
PIP_IN, PIP_OUT_AT, PIP_GONE, PIP_END = 4400, 9100, 9366, 9400
PIP_D, PIP_CX, PIP_CY = 440, 820, 1000
SRC_W, SRC_H = 1280, 720  # the session drawn at 2/3 size
FOCUS_X, FOCUS_Y = 287, 280  # the circle's centre, element-local
DX, DY = FOCUS_X - SRC_W // 2, FOCUS_Y - SRC_H // 2
BACK_OUT = [0.34, 1.56, 0.64, 1.0]
BACK_IN = [0.36, 0.0, 0.66, -0.56]
POP = [(PIP_IN, 0.0, None), (PIP_IN + 450, 1.0, BACK_OUT), (PIP_OUT_AT, 1.0, "linear"), (PIP_GONE, 0.0, BACK_IN)]


def keys(value):
    out = []
    for t, s, ease in POP:
        k = {"t": t, "v": value(s)}
        if ease:
            k["ease"] = ease
        out.append(k)
    return out


scale = keys(lambda s: [s, s])
pip_video = {"id": "pip-video", "type": "video", "start": PIP_IN, "end": PIP_END, "source": "screen/session.mp4",
             "source_start": 33600, "source_end": 37400, "x": keys(lambda s: round(PIP_CX - s * DX)),
             "y": keys(lambda s: round(PIP_CY - s * DY)), "origin": "center", "width": SRC_W, "height": SRC_H,
             "fit": "literal", "scale": scale, "speed": 0.76,
             "effects": [{"name": "mask", "shape": "circle", "x": FOCUS_X - PIP_D // 2, "y": FOCUS_Y - PIP_D // 2,
                          "width": PIP_D, "height": PIP_D}]}
pip_ring = {"id": "pip-ring", "type": "ellipse", "start": PIP_IN, "end": PIP_END, "x": PIP_CX, "y": PIP_CY,
            "origin": "center", "width": PIP_D + 24, "height": PIP_D + 24, "stroke": PAPER, "stroke_width": 14,
            "scale": scale, "effects": [{"name": "shadow", "dx": 0, "dy": 10, "radius": 30, "color": INK,
                                         "opacity": 0.4}]}

# ---------------------------------------------------------------- music
DUCK, OPEN = 0.2, 0.55
env = [(0, DUCK, None),
       (1740, DUCK, "linear"), (1920, OPEN, "ease-out"), (2260, OPEN, "linear"), (2420, DUCK, "ease-in"),
       (4360, DUCK, "linear"), (4560, OPEN, "ease-out"), (5440, OPEN, "linear"), (5621, DUCK, "ease-in"),
       (7820, DUCK, "linear"), (7980, OPEN * 0.8, "ease-out"), (8240, OPEN * 0.8, "linear"),
       (8400, DUCK, "ease-in"),
       (9480, DUCK, "linear"), (9600, 0.35, "ease-out"), (9930, 0.0, "ease-in")]
volume = []
for t, v, ease in env:
    k = {"t": t, "v": round(v, 3)}
    if ease:
        k["ease"] = ease
    volume.append(k)
music = {"id": "music", "type": "audio", "start": 0, "end": END, "source": "music/bed-120bpm.wav",
         "source_start": 0, "source_end": END, "volume": volume}

# ---------------------------------------------------------------- assemble
tracks = [
    {"name": "presenter", "layer": 10, "elements": [presenter]},
    {"name": "pip-ring", "layer": 20, "elements": [pip_ring]},
    {"name": "pip", "layer": 21, "elements": [pip_video]},
    {"name": "lt-bar", "layer": 30, "elements": lt_tracks["lt-bar"]},
    {"name": "lt-accent", "layer": 31, "elements": lt_tracks["lt-accent"]},
    {"name": "lt-title", "layer": 32, "elements": lt_tracks["lt-title"]},
    {"name": "lt-sub", "layer": 33, "elements": lt_tracks["lt-sub"]},
    {"name": "caption-pill", "layer": 40, "elements": cap_pill},
    {"name": "caption", "layer": 41, "elements": cap_text},
    {"name": "bug-pill", "layer": 50, "elements": [bug_pill]},
    {"name": "bug-logo", "layer": 51, "elements": [bug_logo]},
    {"name": "music", "layer": 0, "elements": [music]},
]

doc = json.load(open(PROJECT))
doc["tracks"] = tracks
json.dump(doc, open(PROJECT, "w"), ensure_ascii=False)
subprocess.run(["montagent", "fmt", PROJECT], check=True, capture_output=True)
print("wrote", sum(len(t["elements"]) for t in tracks), "elements")
