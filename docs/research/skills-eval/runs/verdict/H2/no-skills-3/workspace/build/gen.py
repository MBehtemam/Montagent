# Generates square.montagent.json (tracks) from the word timings.
import json, math, os
HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
P = os.path.join(ROOT, 'square.montagent.json')
doc = json.load(open(P))

INK, PAPER, SIGNAL = '#101418', '#F5F0E6', '#FF5A36'
PUNCH_IN, PUNCH_OUT, END = 2720, 6222, 9000
words = json.load(open(os.path.join(ROOT, 'presenter/take-3.words.json')))

def step(a, b):
    """Value a, jumping to b at the punch-in and back to a at the punch-out."""
    return [{"t": 0, "v": a}, {"t": PUNCH_IN, "v": b, "ease": "step"}, {"t": PUNCH_OUT, "v": a, "ease": "step"}]

# camera: wide is identity; close maps (536,334) -> (CX,CY) at scale S
S, HX, HY, CX, CY = 1.4, 536, 334, 400, 330
def cam(x, y): return round(S * (x - HX) + CX), round(S * (y - HY) + CY)

# presenter: 1600x900 box, top-left at (-260,180) in the wide framing
vx, vy = cam(-260, 180)
presenter = {"id": "presenter", "type": "video", "start": 0, "end": END,
    "source": "presenter/take-3.mp4", "source_start": 0, "source_end": 8920,
    "x": step(-260, vx), "y": step(180, vy), "origin": "top-left", "width": 1600, "height": 900, "fit": "contain",
    "scale": step([1.0, 1.0], [S, S]), "overrun": "hold", "volume": 1.6,
    "effects": [{"name": "chroma", "color": "#00FF22", "tolerance": 0.2, "softness": 0.1, "spill": 1.0}]}

dx, dy = cam(536, 420)
disc = {"id": "disc", "type": "ellipse", "start": 0, "end": END,
    "x": step(536, dx), "y": step(420, dy), "origin": "center", "width": 600, "height": 600,
    "fill": SIGNAL, "scale": step([1.0, 1.0], [S, S])}

bug = {"id": "bug", "type": "image", "start": 0, "end": END, "source": "brand/mark.png",
    "x": 48, "y": 48, "origin": "top-left", "width": 72, "height": 72, "fit": "contain"}

# product card: session at 1/3 scale (640x360), masked to its top-left 400x225
CARD_IN, CARD_IN_END, CARD_OUT, CARD_OUT_END = 3080, 3480, 6180, 6466
CARD_GONE = 6500  # one frame past the slide, so its last keyframe is drawn
CW, CH, CXL, CYT, BORDER = 400, 225, 640, 80, 8
OFF = 1080 + 40
def slide(home):
    return [{"t": CARD_IN, "v": OFF + (home - CXL)}, {"t": CARD_IN_END, "v": home, "ease": "ease-out"},
            {"t": CARD_OUT, "v": home, "ease": "linear"}, {"t": CARD_OUT_END, "v": OFF + (home - CXL), "ease": "ease-in"}]
card_frame = {"id": "card-frame", "type": "rect", "start": CARD_IN, "end": CARD_GONE,
    "x": slide(CXL - BORDER), "y": CYT - BORDER, "origin": "top-left", "width": CW + 2 * BORDER, "height": CH + 2 * BORDER,
    "fill": INK, "radius": 28,
    "effects": [{"name": "shadow", "dx": 0, "dy": 12, "radius": 24, "color": INK, "opacity": 0.3}]}
span = CARD_GONE - CARD_IN
card = {"id": "card", "type": "video", "start": CARD_IN, "end": CARD_GONE,
    "source": "screen/session.mp4", "source_start": 10000, "source_end": 10000 + 2 * span,
    "x": slide(CXL), "y": CYT, "origin": "top-left", "width": 640, "height": 360, "fit": "contain", "speed": 2.0,
    "effects": [{"name": "mask", "shape": "rect", "x": 0, "y": 0, "width": CW, "height": CH, "radius": 20}]}

# captions
phrases = [(0, 5, 1260), (5, 8, 2400), (8, 11, 3580), (11, 14, 4513), (14, 17, 5600), (17, 22, 7580), (22, 25, 8900)]
widths = {}
ACCENT = {"mouse.", "truth"}
SIZE, CAP_Y, PAD_X, PILL_H = 60, 905, 36, 100
pills, caps = [], []
CLEAR = "#10141800"
for i, (a, b, end) in enumerate(phrases):
    ws = words[a:b]
    start = ws[0]["start"]
    # one element per build state: words spoken so far are inked, the rest are laid out but clear,
    # so the line never reflows as it fills
    for k in range(len(ws)):
        s0 = start if k == 0 else ws[k]["start"]
        s1 = ws[k + 1]["start"] if k + 1 < len(ws) else end
        runs = []
        for j, w in enumerate(ws):
            text = w["word"] + (" " if j < len(ws) - 1 else "")
            color = (SIGNAL if w["word"] in ACCENT else INK) if j <= k else CLEAR
            runs.append({"text": text, "color": color})
        caps.append({"id": f"cap-{i+1}-{k+1}", "type": "text", "start": s0, "end": s1,
            "x": 540, "y": CAP_Y, "origin": "center", "width": 0, "height": 72, "font": "bold", "size": SIZE,
            "align": "center", "runs": runs, "_pill": i})
    pills.append({"id": f"pill-{i+1}", "type": "rect", "start": start, "end": end,
        "x": 540, "y": CAP_Y, "origin": "center", "width": 0, "height": PILL_H, "fill": PAPER, "radius": PILL_H // 2,
        "effects": [{"name": "shadow", "dx": 0, "dy": 6, "radius": 18, "color": INK, "opacity": 0.18}]})
pill_of = {c["id"]: c.pop("_pill") for c in caps}

# end card
LW = 760; LH = round(LW * 623 / 2694)
endcard = [
    {"id": "end-ground", "type": "rect", "start": END, "end": 10000, "x": 0, "y": 0, "origin": "top-left", "width": 1080, "height": 1080, "fill": PAPER},
    {"id": "end-lockup", "type": "image", "start": END, "end": 10000, "source": "brand/lockup.png",
     "x": 540, "y": 540, "origin": "center", "width": LW, "height": LH, "fit": "contain",
     "scale": [{"t": END, "v": [1.0, 1.0]}, {"t": 9966, "v": [1.04, 1.04], "ease": "linear"}]},
]

music = {"id": "music", "type": "audio", "start": 0, "end": 10000, "source": "music/bed-120bpm.wav",
    "source_start": 0, "source_end": 10000,
    "volume": [{"t": 0, "v": 0.25}, {"t": 8420, "v": 0.25, "ease": "linear"}, {"t": 9000, "v": 1.0, "ease": "ease-out"},
               {"t": 9250, "v": 1.0, "ease": "linear"}, {"t": 9900, "v": 0.0, "ease": "ease-in-out"}]}

doc["tracks"] = [
    {"name": "disc", "layer": 10, "elements": [disc]},
    {"name": "presenter", "layer": 20, "elements": [presenter]},
    {"name": "bug", "layer": 25, "elements": [bug]},
    {"name": "card-frame", "layer": 30, "elements": [card_frame]},
    {"name": "card", "layer": 31, "elements": [card]},
    {"name": "pills", "layer": 40, "elements": pills},
    {"name": "captions", "layer": 41, "elements": caps},
    {"name": "endcard", "layer": 50, "elements": endcard[:1]},
    {"name": "endcard-lockup", "layer": 51, "elements": endcard[1:]},
    {"name": "music", "layer": 0, "elements": [music]},
]
json.dump(doc, open(P, 'w'), indent=2, ensure_ascii=False)

# fill caption and pill widths from what the text actually occupies
import subprocess
m = json.loads(subprocess.run(["montagent", "measure", P, "--all", "--json"], capture_output=True, text=True).stdout)
adv = {r["id"]: r["ok"]["extent"]["width"] for r in m["measure"]["results"]}
for c in caps:
    c["width"] = math.ceil(adv[c["id"]]) + 4
    pills[pill_of[c["id"]]]["width"] = math.ceil(adv[c["id"]]) + 2 * PAD_X
json.dump(doc, open(P, 'w'), indent=2, ensure_ascii=False)
subprocess.run(["montagent", "fmt", P], capture_output=True)
