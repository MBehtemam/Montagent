"""Generate launch.json: the Montagent 12 s launch spot (Brief A)."""
import json
import math
import subprocess

INK, PAPER, SIGNAL = "#101418", "#F5F0E6", "#FF5A36"
POP = [0.34, 1.56, 0.64, 1]          # overshoot then settle
OUT_EXPO = [0.16, 1, 0.3, 1]
IN_OUT = [0.65, 0, 0.35, 1]

proj = json.load(open("launch.json"))


def measure(specs):
    out = subprocess.run(["montagent", "measure", "launch.json", "--elements", json.dumps(specs), "--json"],
                         capture_output=True, text=True, check=True).stdout
    return [r["ok"] for r in json.loads(out)["measure"]["results"]]


tracks = []


def track(name, layer, *els):
    tracks.append({"name": name, "layer": layer, "elements": list(els)})


# ---------- 0–4 s: the hook ----------
WORDS = ["Video", "your", "agent", "can", "read."]
SIZE = 116
m = measure([{"runs": [{"text": w}], "font": "bold", "size": SIZE} for w in WORDS + [" ".join(WORDS)]])
widths = [r["advance_width"] for r in m[:5]]
full = m[5]["advance_width"]
space = (full - sum(widths)) / 4
block_h = m[0]["block_height"]
left = 960 - full / 2
for i, (w, adv) in enumerate(zip(WORDS, widths)):
    beat = i * 500
    cx = round(left + sum(widths[:i]) + space * i + adv / 2)
    run = {"text": w}
    if w == "read.":
        run["highlight"] = {"start": 2000, "end": 4000, "color": SIGNAL}
    el = {
        "id": f"hook-{i+1}", "type": "text", "start": beat, "end": 4000,
        "x": cx, "y": [{"t": beat, "v": 580}, {"t": beat + 400, "v": 540, "ease": POP},
                       {"t": 3500, "v": 540, "ease": "linear"}, {"t": 3900, "v": 500, "ease": "ease-in"}],
        "origin": "center", "width": math.ceil(adv) + 24, "height": block_h,
        "font": "bold", "size": SIZE, "color": PAPER, "align": "center", "runs": [run],
        "scale": [{"t": beat, "v": [0.6, 0.6]}, {"t": beat + 400, "v": [1.0, 1.0], "ease": POP}],
        "opacity": [{"t": beat, "v": 0.0}, {"t": beat + 100, "v": 1.0, "ease": "linear"},
                    {"t": 3500, "v": 1.0, "ease": "linear"}, {"t": 3900, "v": 0.0, "ease": "ease-in"}],
    }
    track(f"hook-{i+1}", 20 + i, el)

# ---------- 4–10 s: paper ground ----------
track("ground", 1, {"id": "paper", "type": "rect", "start": 4000, "end": 10000,
                    "x": 0, "y": 0, "origin": "top-left", "width": 1920, "height": 1080, "fill": PAPER})

# ---------- 4–9 s: screenshots in a rounded, shadowed card, pushing in ----------
CARD_W, CARD_H = 1440, 810
push = [{"t": 4000, "v": [0.94, 0.94]}, {"t": 9000, "v": [1.06, 1.06], "ease": "linear"}]
card_fx = [{"name": "mask", "shape": "rect", "radius": 28},
           {"name": "shadow", "dx": 0, "dy": 28, "radius": 48, "color": INK, "opacity": 0.35}]
shot1 = {"id": "shot-1", "type": "image", "start": 4000, "end": 7500, "source": "stills/session-01.png",
         "x": 960, "y": [{"t": 4000, "v": 640}, {"t": 4600, "v": 540, "ease": OUT_EXPO}],
         "origin": "center", "width": CARD_W, "height": CARD_H, "fit": "contain", "scale": push,
         "opacity": [{"t": 4000, "v": 0.0}, {"t": 4300, "v": 1.0, "ease": "ease-out"}], "effects": card_fx}
shot2 = {"id": "shot-2", "type": "image", "start": 7000, "end": 9000, "source": "stills/session-02.png",
         "x": 960, "y": 540, "origin": "center", "width": CARD_W, "height": CARD_H, "fit": "contain",
         "scale": push, "effects": card_fx}
track("shot-1", 10, shot1)
track("shot-2", 11, shot2)
track("transitions", 12, {"id": "xfade", "type": "transition", "start": 7000, "end": 7500,
                          "kind": "crossfade", "from": "shot-1", "to": "shot-2"})

# ---------- 9–10 s: three-bar chart ----------
BASE = 780
BAR_W, GAP = 220, 90
bars = [("Edit", 200, INK, 400), ("Check", 330, INK, 500), ("Render", 480, SIGNAL, 600)]
lm = measure([{"runs": [{"text": b[0]}], "font": "bold", "size": 44} for b in bars])
for i, ((label, h, fill, dur), lmi) in enumerate(zip(bars, lm)):
    cx = 960 + (i - 1) * (BAR_W + GAP)
    track(f"bar-{i+1}", 30 + i, {
        "id": f"bar-{label.lower()}", "type": "rect", "start": 9000, "end": 10000,
        "x": cx, "y": BASE, "origin": "bottom-center", "width": BAR_W, "height": h, "fill": fill,
        "scale": [{"t": 9000, "v": [1.0, 0.0]}, {"t": 9000 + dur, "v": [1.0, 1.0], "ease": "ease-out"}]})
    track(f"label-{i+1}", 40 + i, {
        "id": f"label-{label.lower()}", "type": "text", "start": 9000, "end": 10000,
        "x": cx, "y": BASE + 28, "origin": "top-center", "width": BAR_W + 60, "height": lmi["block_height"],
        "font": "bold", "size": 44, "color": INK, "align": "center", "runs": [{"text": label}],
        "opacity": [{"t": 9000, "v": 0.0}, {"t": 9300, "v": 1.0, "ease": "ease-out"}]})
track("baseline", 36, {"id": "baseline", "type": "rect", "start": 9000, "end": 10000,
                       "x": 960, "y": BASE, "origin": "center", "width": 3 * BAR_W + 2 * GAP + 80, "height": 6,
                       "fill": INK, "scale": [{"t": 9000, "v": [0.0, 1.0]}, {"t": 9300, "v": [1.0, 1.0], "ease": OUT_EXPO}]})

# ---------- 10–12 s: wordmark revealed by a wiping bar ----------
WM_W, WM_H = 1200, 252
wm_left, wm_right = 960 - WM_W // 2, 960 + WM_W // 2
wipe_x = [{"t": 10000, "v": wm_left - 40}, {"t": 10400, "v": wm_right + 30, "ease": IN_OUT}]
track("wordmark", 50, {"id": "wordmark", "type": "image", "start": 10000, "end": 12000,
                       "source": "brand/wordmark-on-dark.png", "x": 960, "y": 540, "origin": "center",
                       "width": WM_W, "height": WM_H, "fit": "contain"})
track("wipe-cover", 51, {"id": "wipe-cover", "type": "rect", "start": 10000, "end": 10500,
                         "x": wipe_x, "y": 540, "origin": "center-left", "width": 1400, "height": 360, "fill": INK})
track("wipe-bar", 52, {"id": "wipe-bar", "type": "rect", "start": 10000, "end": 12000,
                       "x": wipe_x, "y": 540, "origin": "center-left", "width": 18, "height": 300, "fill": SIGNAL,
                       "scale": [{"t": 10400, "v": [1.0, 1.0]}, {"t": 10700, "v": [1.0, 0.72], "ease": OUT_EXPO}]})

# ---------- music ----------
track("music", 0, {"id": "bed", "type": "audio", "start": 0, "end": 12000, "source": "music/bed-120bpm.wav",
                   "source_start": 0, "source_end": 12000,
                   "volume": [{"t": 11000, "v": 0.9}, {"t": 11900, "v": 0.0, "ease": "linear"}]})

proj["tracks"] = tracks
json.dump(proj, open("launch.json", "w"), ensure_ascii=False)
print("space", space, "full", full, "block_h", block_h)
