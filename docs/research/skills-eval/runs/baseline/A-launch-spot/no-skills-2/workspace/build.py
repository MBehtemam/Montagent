"""Generate spot.json's tracks for Brief A. Run, then `montagent fmt spot.json`."""
import json, math

INK, PAPER, SIGNAL = "#101418", "#F5F0E6", "#FF5A36"
P = "spot.json"
proj = json.load(open(P))

def kf(*recs):
    out = []
    for i, (t, v, *e) in enumerate(recs):
        r = {"t": t, "v": v}
        if i:
            r["ease"] = e[0] if e else "linear"
        out.append(r)
    return out

tracks = []
def track(name, layer, *els):
    tracks.append({"name": name, "layer": layer, "elements": list(els)})

OUT = [0.16, 1, 0.3, 1]     # fast settle
INO = "ease-in-out"

# ---------- 0-4 s: hook words, one per beat ----------
words = ["Video", "your", "agent", "can", "read."]
adv = [342.83203125, 267.24609375, 335.625, 216.15234375, 303.984375]
space = 28.41796875
total = sum(adv) + space * 4
left = 960 - total / 2
cx, cur = [], left
for w in adv:
    cx.append(round(cur + w / 2))
    cur += w + space
Y = 540

def word(id_, text, color, b, x, w, opacity):
    return {"id": id_, "type": "text", "start": b, "end": 4000,
            "x": x,
            "y": kf((b, Y + 36), (b + 170, Y - 8, OUT), (b + 340, Y, INO), (3500, Y, "step"), (3900, Y - 40, "ease-in")),
            "origin": "center", "width": math.ceil(w) + 8, "height": 120,
            "font": "bold", "size": 120, "line_height": 1.0, "color": color,
            "align": "center", "runs": [{"text": text}],
            "scale": kf((b, [0.4, 0.4]), (b + 170, [1.1, 1.1], OUT), (b + 340, [1.0, 1.0], INO)),
            "opacity": opacity}

for i, (t, x, w) in enumerate(zip(words, cx, adv)):
    b = i * 500
    if t == "read.":
        op = kf((b, 0.0), (b + 100, 1.0, "ease-out"), (2250, 1.0, "step"), (2300, 0.0, "linear"))
    else:
        op = kf((b, 0.0), (b + 100, 1.0, "ease-out"), (3500, 1.0, "step"), (3900, 0.0, "ease-in"))
    track(f"hook-{i+1}", 40 + i, word(f"hook-{i+1}", t, PAPER, b, x, w, op))

# "read." turns Signal on the downbeat at 2.0 s
track("hook-accent", 46, word("hook-read-accent", "read.", SIGNAL, 2000, cx[4], adv[4],
      kf((2000, 0.0), (2200, 1.0, "ease-out"), (3500, 1.0, "step"), (3900, 0.0, "ease-in"))))

# ---------- 4-10 s: paper ground ----------
track("ground", 0, {"id": "paper-ground", "type": "rect", "start": 4000, "end": 10000,
                    "x": 0, "y": 0, "origin": "top-left", "width": 1920, "height": 1080, "fill": PAPER})

# ---------- 4-9 s: screenshots in a rounded, shadowed frame, pushing in ----------
CW, CH, R = 1440, 810, 28
card_y = kf((4000, 660), (4500, 540, OUT))
push = kf((4000, [1.0, 1.0]), (8950, [1.08, 1.08]))
enter = kf((4000, 0.0), (4250, 1.0, "ease-out"))
track("card", 10, {"id": "card", "type": "rect", "start": 4000, "end": 9000, "x": 960, "y": card_y,
                   "origin": "center", "width": CW, "height": CH, "fill": INK, "radius": R,
                   "scale": push, "opacity": enter,
                   "effects": [{"name": "shadow", "dx": 0, "dy": 28, "radius": 56, "color": INK, "opacity": 0.35}]})
mask = [{"name": "mask", "shape": "rect", "x": 0, "y": 0, "width": CW, "height": CH, "radius": R}]
track("shot-1", 20, {"id": "shot-1", "type": "image", "start": 4000, "end": 7500, "source": "stills/session-01.png",
                     "x": 960, "y": card_y, "origin": "center", "width": CW, "height": CH, "fit": "literal",
                     "scale": push, "opacity": enter, "effects": mask})
track("shot-2", 21, {"id": "shot-2", "type": "image", "start": 7000, "end": 9000, "source": "stills/session-02.png",
                     "x": 960, "y": card_y, "origin": "center", "width": CW, "height": CH, "fit": "literal",
                     "scale": push, "effects": mask})
track("xfade", 22, {"id": "xfade-shots", "type": "transition", "start": 7000, "end": 7500,
                    "kind": "crossfade", "from": "shot-1", "to": "shot-2"})

# ---------- 9-10 s: bar chart growing from its baseline ----------
BASE = 700
bars = [("Edit", 180, INK), ("Check", 310, INK), ("Render", 460, SIGNAL)]
track("chart-axis", 30, {"id": "chart-baseline", "type": "rect", "start": 9000, "end": 10000,
                         "x": 960, "y": BASE, "origin": "top-center", "width": 860, "height": 4, "fill": INK})
for i, (label, h, col) in enumerate(bars):
    x = 960 + (i - 1) * 270
    track(f"bar-{i+1}", 31 + i, {"id": f"bar-{label.lower()}", "type": "rect", "start": 9000, "end": 10000,
                                 "x": x, "y": BASE, "origin": "bottom-center", "width": 180, "height": h,
                                 "fill": col, "radius": 10,
                                 "scale": kf((9000, [1.0, 0.0]), (9450, [1.0, 1.0], OUT))})
    track(f"label-{i+1}", 35 + i, {"id": f"label-{label.lower()}", "type": "text", "start": 9000, "end": 10000,
                                   "x": x, "y": BASE + 28, "origin": "top-center", "width": 240, "height": 53,
                                   "font": "bold", "size": 44, "color": INK, "align": "center",
                                   "runs": [{"text": label}],
                                   "opacity": kf((9000, 0.0), (9250, 1.0, "ease-out"))})

# ---------- 10-12 s: wordmark revealed by a wiping bar ----------
WW, WH, WY = 1200, 252, 540
track("wordmark", 50, {"id": "wordmark", "type": "image", "start": 10000, "end": 12000,
                       "source": "brand/wordmark-on-dark.png", "x": 960, "y": WY, "origin": "center",
                       "width": WW, "height": WH, "fit": "literal"})
L, Rt = 960 - WW // 2 - 40, 960 + WW // 2 + 40
track("wipe-cover", 51, {"id": "wipe-cover", "type": "rect", "start": 10000, "end": 10500,
                         "x": Rt, "y": WY, "origin": "center-right", "width": Rt - L, "height": 340, "fill": INK,
                         "scale": kf((10000, [1.0, 1.0]), (10450, [0.0, 1.0], INO))})
track("wipe-bar", 52, {"id": "wipe-bar", "type": "rect", "start": 10000, "end": 10500,
                       "x": kf((10000, L), (10450, Rt, INO)), "y": WY, "origin": "center",
                       "width": 20, "height": 340, "fill": SIGNAL})

# ---------- music ----------
track("music", 0, {"id": "music", "type": "audio", "start": 0, "end": 12000, "source": "music/bed-120bpm.wav",
                   "source_start": 0, "source_end": 12000,
                   "volume": kf((11000, 1.0), (11950, 0.0))})

proj["tracks"] = tracks
json.dump(proj, open(P, "w"), ensure_ascii=False)
