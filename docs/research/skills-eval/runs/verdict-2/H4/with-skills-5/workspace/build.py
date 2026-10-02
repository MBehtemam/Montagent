"""Generate ad.montagent.json from the beat sheet. 120 BPM, downbeat at 0: a beat is 500 ms."""
import json

INK, PAPER, SIGNAL = "#101418", "#F5F0E6", "#FF5A36"
POP = [0.34, 1.56, 0.64, 1]
SETTLE = [0.16, 1, 0.3, 1]
B = 500  # one beat, ms

# Beat sheet (ms)
HOOK = (0, 6 * B)            # 0.0 - 3.0
PROOF = (6 * B, 13 * B)      # 3.0 - 6.5
RESULT = (13 * B, 18 * B)    # 6.5 - 9.0
CLAIM = (18 * B, 25 * B)     # 9.0 - 12.5
BRAND = (25 * B, 30 * B)     # 12.5 - 15.0
END = 15000
LAST_FRAME = 14966

# The diff lands in the session at about 33.45 s; start the clip so it lands on beat 8 (4.0 s).
SESSION_IN = 32550

proj = json.load(open("ad.montagent.json"))
proj["fonts"] = {"bold": [{"file": "fonts/Inter-Bold.ttf"}], "regular": [{"file": "fonts/Inter-Regular.ttf"}]}


def pop_scale(t, s0=0.6, length=400):
    return [{"t": t, "v": [s0, s0]}, {"t": t + length, "v": [1.0, 1.0], "ease": POP}]


def fade_in(t, length=200):
    return [{"t": t, "v": 0.0}, {"t": t + length, "v": 1.0, "ease": "ease-out"}]


def rise(t, y, d=30, length=400, ease=POP):
    return [{"t": t, "v": y + d}, {"t": t + length, "v": y, "ease": ease}]


def text(id, start, end, x, y, w, h, font, size, color, runs, origin="center", align="center", **kw):
    e = {"id": id, "type": "text", "start": start, "end": end, "x": x, "y": y, "origin": origin,
         "width": w, "height": h, "font": font, "size": size, "line_height": 1.1 if size > 80 else 1.2,
         "color": color, "align": align, "runs": runs}
    e.update(kw)
    e["caption"] = False
    return e


tracks = []

# Grounds: Ink is the project background; Paper for the product section and the brand.
tracks.append({"name": "ground", "layer": 1, "elements": [
    {"id": "paper-product", "type": "rect", "start": PROOF[0], "end": RESULT[1], "x": 0, "y": 0, "origin": "top-left", "width": 1080, "height": 1080, "fill": PAPER},
    {"id": "paper-brand", "type": "rect", "start": BRAND[0], "end": END, "x": 0, "y": 0, "origin": "top-left", "width": 1080, "height": 1080, "fill": PAPER},
]})

# Hook: one line per beat, the last word turns Signal on beat 3.
hook = []
for i, (line, y) in enumerate([("Your AI agent", 390), ("edits video", 540), ("like code.", 690)]):
    t = HOOK[0] + i * B
    runs = [{"text": line}] if i < 2 else [{"text": "like "}, {"text": "code.", "highlight": {"start": 3 * B, "end": HOOK[1], "color": SIGNAL}}]
    hook.append(text(f"hook-{i+1}", t, HOOK[1], 540, rise(t, y), 1000, 137, "bold", 124, PAPER, runs,
                     scale=pop_scale(t)))
for i, e in enumerate(hook):
    tracks.append({"name": f"hook-{i+1}", "layer": 20, "elements": [e]})

# Step badge: one Signal circle through proof and result, with the step number on it.
tracks.append({"name": "step-badge", "layer": 20, "elements": [
    {"id": "badge", "type": "ellipse", "start": PROOF[0], "end": RESULT[1], "x": 104, "y": 130, "origin": "center", "width": 64, "height": 64, "fill": SIGNAL,
     "scale": pop_scale(PROOF[0], 0.0, 350) + [{"t": RESULT[0], "v": [1.0, 1.0], "ease": "linear"}, {"t": RESULT[0] + 150, "v": [1.15, 1.15], "ease": "ease-out"}, {"t": RESULT[0] + 350, "v": [1.0, 1.0], "ease": "ease-in"}]},
]})
tracks.append({"name": "step-num", "layer": 21, "elements": [
    text("num-1", PROOF[0], RESULT[0], 104, 131, 64, 53, "bold", 44, INK, [{"text": "1"}], scale=pop_scale(PROOF[0], 0.0, 350)),
    text("num-2", RESULT[0], RESULT[1], 104, 131, 64, 53, "bold", 44, INK, [{"text": "2"}]),
]})
tracks.append({"name": "step-label", "layer": 20, "elements": [
    text("label-1", PROOF[0], RESULT[0], 152, rise(PROOF[0] + 90, 130, 20, 300, SETTLE), 600, 58, "bold", 48, INK,
         [{"text": "Your agent edits the file"}], origin="center-left", align="start", opacity=fade_in(PROOF[0] + 90)),
    text("label-2", RESULT[0], RESULT[1], 152, rise(RESULT[0] + 90, 130, 20, 300, SETTLE), 720, 58, "bold", 48, INK,
         [{"text": "Montagent renders the video"}], origin="center-left", align="start", opacity=fade_in(RESULT[0] + 90)),
]})

# Proof: the real session, a 1:1.1 crop of its left side, with the diff landing on beat 8.
# Element is the 1920x1080 source at 1.1x; the mask picks out a 936x800 card of it.
CARD = {"x": 72, "y": 196, "w": 936, "h": 800}
MASK_T = {"x": 28, "y": 83}
tracks.append({"name": "proof", "layer": 10, "elements": [
    {"id": "session", "type": "video", "start": PROOF[0], "end": PROOF[1], "source": "screen/session.mp4",
     "source_start": SESSION_IN, "source_end": SESSION_IN + (PROOF[1] - PROOF[0]),
     "x": CARD["x"] - MASK_T["x"], "y": rise(PROOF[0], CARD["y"] - MASK_T["y"], 40, 500, SETTLE), "origin": "top-left",
     "width": 2112, "height": 1188, "fit": "contain", "opacity": fade_in(PROOF[0], 250),
     "effects": [{"name": "mask", "shape": "rect", "x": MASK_T["x"], "y": MASK_T["y"], "width": CARD["w"], "height": CARD["h"], "radius": 24},
                 {"name": "shadow", "dx": 0, "dy": 24, "radius": 48, "color": INK, "opacity": 0.3}]},
]})

# Result: the frame the session rendered, cropped to its centre and enlarged 1.3x so the subtitle reads.
# Card 936x527 (16:9) centred under the label.
tracks.append({"name": "result", "layer": 10, "elements": [
    {"id": "render", "type": "image", "start": RESULT[0], "end": RESULT[1], "source": "stills/session-02.png",
     "x": 540, "y": 552, "origin": "center", "width": 1248, "height": 702, "fit": "contain",
     "scale": pop_scale(RESULT[0], 0.9, 400), "opacity": fade_in(RESULT[0], 150),
     "effects": [{"name": "mask", "shape": "rect", "x": 156, "y": 134, "width": 936, "height": 527, "radius": 24},
                 {"name": "shadow", "dx": 0, "dy": 24, "radius": 48, "color": INK, "opacity": 0.3}]},
]})

# Claim: one line in two, "check" turns Signal a beat after it lands.
tracks.append({"name": "claim", "layer": 20, "elements": [
    text("claim", CLAIM[0], CLAIM[1], 540, rise(CLAIM[0], 540, 30, 400), 960, 203, "bold", 92, PAPER,
         [{"text": "Your agent can\n"}, {"text": "check", "highlight": {"start": CLAIM[0] + B, "end": CLAIM[1], "color": SIGNAL}}, {"text": " every frame."}],
         scale=pop_scale(CLAIM[0], 0.85, 400)),
]})

# Brand: lockup pops on beat 25, the descriptor follows a beat later, then everything holds.
tracks.append({"name": "brand", "layer": 20, "elements": [
    {"id": "lockup", "type": "image", "start": BRAND[0], "end": END, "source": "brand/lockup.png",
     "x": 540, "y": 500, "origin": "center", "width": 820, "height": 190, "fit": "contain",
     "scale": pop_scale(BRAND[0], 0.8, 400), "opacity": fade_in(BRAND[0], 150)},
]})
tracks.append({"name": "tagline", "layer": 20, "elements": [
    text("tagline", BRAND[0] + B, END, 540, rise(BRAND[0] + B, 668, 20, 300, SETTLE), 900, 53, "regular", 44, INK,
         [{"text": "The video editor AI agents drive."}], opacity=fade_in(BRAND[0] + B, 300)),
]})

# Music: under everything, fading to silence on the last drawn frame.
tracks.append({"name": "music", "layer": 0, "elements": [
    {"id": "bed", "type": "audio", "start": 0, "end": END, "source": "music/bed-120bpm.wav", "source_start": 0, "source_end": END,
     "volume": [{"t": 14000, "v": 0.8}, {"t": LAST_FRAME, "v": 0.0, "ease": "linear"}]},
]})

proj["tracks"] = tracks
json.dump(proj, open("ad.montagent.json", "w"), indent=2, ensure_ascii=False)
