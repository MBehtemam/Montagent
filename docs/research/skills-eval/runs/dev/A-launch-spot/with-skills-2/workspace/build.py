"""Generate spot.json from the beat sheet. Run: python3 build.py"""
import json, subprocess

INK, PAPER, SIGNAL = "#101418", "#F5F0E6", "#FF5A36"
POP = [0.34, 1.56, 0.64, 1]
SWEEP = [0.65, 0, 0.35, 1]
GROW = [0.16, 1, 0.3, 1]

# Beat sheet (ms, 120 BPM: beat 500, bar 2000)
HOOK = [0, 500, 1000, 1500, 2000]   # words land
ACCENT = 2000                       # "read." in Signal
HOOK_EXIT = 3500                    # line clears over 300 ms
SHOW = 4000                         # light ground, screenshot 1 enters
XFADE = (7000, 7500)                # screenshot 1 -> 2
CHART = 9000                        # cut to chart
BRAND = 10000                       # dark ground, wordmark wipe
WIPE_END = 10500
FADE = (11000, 11933)               # music out
END = 12000

p = json.load(open("spot.json"))
p["tracks"] = []
T = p["tracks"]

def track(name, layer, *els):
    T.append({"name": name, "layer": layer, "elements": list(els)})

# Grounds
track("ground", 1, {"id": "paper", "type": "rect", "start": SHOW, "end": BRAND, "x": 0, "y": 0,
                    "origin": "top-left", "width": 1920, "height": 1080, "fill": PAPER})

# Screenshots: one linear push 1.00 -> 1.06 from SHOW to CHART
W, H = 1440, 810
def push(t):
    return round(1.0 + 0.06 * (t - SHOW) / (CHART - SHOW), 4)
PUSH = [{"t": SHOW, "v": [1.0, 1.0]}, {"t": CHART + 500, "v": [1.066, 1.066], "ease": "linear"}]
FRAME_FX = [{"name": "mask", "shape": "rect", "radius": 28},
            {"name": "shadow", "dx": 0, "dy": 28, "radius": 56, "color": INK, "opacity": 0.35}]
track("shot-1", 10, {"id": "shot-1", "type": "image", "start": SHOW, "end": XFADE[1],
    "source": "stills/session-01.png", "x": 960,
    "y": [{"t": SHOW, "v": 600}, {"t": SHOW + 500, "v": 540, "ease": "ease-out"}],
    "origin": "center", "width": W, "height": H, "fit": "contain",
    "scale": PUSH,
    "opacity": [{"t": SHOW, "v": 0.0}, {"t": SHOW + 400, "v": 1.0, "ease": "ease-out"}],
    "effects": FRAME_FX})
track("shot-2", 11, {"id": "shot-2", "type": "image", "start": XFADE[0], "end": CHART,
    "source": "stills/session-02.png", "x": 960, "y": 540, "origin": "center", "width": W, "height": H, "fit": "contain",
    "scale": PUSH,
    "effects": FRAME_FX})
track("joins", 12, {"id": "join", "type": "transition", "start": XFADE[0], "end": XFADE[1],
                    "kind": "crossfade", "from": "shot-1", "to": "shot-2"})

# Chart: baseline, three bars growing from it, labels as each settles
BASE_Y, BAR_W, GAP = 790, 220, 110
bars = [("Edit", 220, INK), ("Check", 380, INK), ("Render", 560, SIGNAL)]
track("chart-base", 20, {"id": "chart-base", "type": "rect", "start": CHART, "end": BRAND, "x": 960, "y": BASE_Y,
    "origin": "top-center", "width": 3 * BAR_W + 2 * GAP + 160, "height": 6, "fill": INK,
    "scale": [{"t": CHART, "v": [0.0, 1.0]}, {"t": CHART + 267, "v": [1.0, 1.0], "ease": GROW}]})
for i, (label, h, fill) in enumerate(bars):
    x = 960 + (i - 1) * (BAR_W + GAP)
    t0 = CHART + 100 * i
    t1 = t0 + 467
    track(f"bar-{i+1}", 21, {"id": f"bar-{label.lower()}", "type": "rect", "start": t0, "end": BRAND, "x": x, "y": BASE_Y,
        "origin": "bottom-center", "width": BAR_W, "height": h, "fill": fill, "radius": 12,
        "scale": [{"t": t0, "v": [1.0, 0.0]}, {"t": t1, "v": [1.0, 1.0], "ease": GROW}]})
    track(f"label-{i+1}", 22, {"id": f"label-{label.lower()}", "type": "text", "start": t0 + 300, "end": BRAND,
        "x": x, "y": BASE_Y + 28, "origin": "top-center", "width": 300, "height": 53, "font": "bold", "size": 44,
        "color": INK, "align": "center", "runs": [{"text": label}],
        "opacity": [{"t": t0 + 300, "v": 0.0}, {"t": t0 + 500, "v": 1.0, "ease": "ease-out"}]})

# Brand: wordmark wiped on by an occluder with a Signal edge bar
MW, MH = 1200, 252
track("wordmark", 30, {"id": "wordmark", "type": "image", "start": BRAND, "end": END,
    "source": "brand/wordmark-on-dark.png", "x": 960, "y": 500, "origin": "center", "width": MW, "height": MH, "fit": "contain"})
OW, OH = MW + 60, MH + 60
left, right = 960 - OW // 2, 960 + OW // 2
track("occluder", 31, {"id": "occluder", "type": "rect", "start": BRAND, "end": WIPE_END + 100, "x": right, "y": 500,
    "origin": "center-right", "width": OW, "height": OH, "fill": INK,
    "scale": [{"t": BRAND, "v": [1.0, 1.0]}, {"t": WIPE_END, "v": [0.0, 1.0], "ease": SWEEP}]})
track("edge", 32, {"id": "edge", "type": "rect", "start": BRAND, "end": WIPE_END + 300,
    "x": [{"t": BRAND, "v": left}, {"t": WIPE_END, "v": right, "ease": SWEEP}], "y": 500, "origin": "center",
    "width": 16, "height": OH + 30, "fill": SIGNAL,
    "opacity": [{"t": WIPE_END, "v": 1.0}, {"t": WIPE_END + 200, "v": 0.0, "ease": "ease-in"}]})
track("tagline", 33, {"id": "tagline", "type": "text", "start": WIPE_END, "end": END, "x": 960, "y": 700,
    "origin": "top-center", "width": 900, "height": 53, "font": "regular", "size": 44, "color": PAPER, "align": "center",
    "runs": [{"text": "The video editor agents drive."}],
    "opacity": [{"t": WIPE_END, "v": 0.0}, {"t": WIPE_END + 300, "v": 0.8, "ease": "ease-out"}]})

# Music
track("music", 0, {"id": "bed", "type": "audio", "start": 0, "end": END, "source": "music/bed-120bpm.wav",
    "source_start": 0, "source_end": END,
    "volume": [{"t": FADE[0], "v": 0.8}, {"t": FADE[1], "v": 0.0, "ease": "linear"}]})

json.dump(p, open("spot.json", "w"))

# Hook: words landing on beats, via the bundled script
spec = {"track": "hook", "layer": 40, "text": "Video your agent can read.", "by": "word", "font": "bold", "size": 120,
        "color": PAPER, "x": 960, "y": 540, "align": "center", "times": HOOK, "end": SHOW,
        "enter": "pop", "pop": {"from": 0.5, "ms": 400, "rise": 40},
        "highlights": [{"unit": 5, "start": ACCENT, "color": SIGNAL}],
        "exit": {"at": HOOK_EXIT, "ms": 300}}
json.dump(spec, open("hook.spec.json", "w"))
out = subprocess.run(["python3", ".claude/skills/montagent-motion/scripts/type_on.py", "spot.json", "hook.spec.json"],
                     check=True, capture_output=True, text=True).stdout
open("spot.json", "w").write(out)
subprocess.run(["montagent", "fmt", "spot.json"], check=True)
