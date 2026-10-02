#!/usr/bin/env python3
"""Generate presenter-cut.json (Brief H5) from take-2's word timings.

usage: build.py [tolerance softness spill]   (chroma key parameters)
"""
import json, subprocess, sys

INK, PAPER, SIGNAL = "#101418", "#F5F0E6", "#FF5A36"
T0 = 200                      # timeline instant the take starts
TAKE_MS = 9800                # video stream length of take-2
FADE_OUT = (9760, 9960)       # presenter scene leaves; last word ends at 9700
SCENE_END = 10000
CLOSE_IN = (10000, 10400)
END = 12000

words = json.load(open("presenter/take-2.words.json"))
W = [(w["word"], w["start"] + T0, w["end"] + T0) for w in words]

# The card the presenter stands in; captions sit in the band beneath it.
CARD = dict(x=48, y=140, w=984, h=860)
VID_X, VID_Y = 540 - 942, 155   # source presenter centre x≈942 → frame centre, 1:1 scale
BAND_MID = 1170

args = [float(a) for a in sys.argv[1:4]] or [0.25, 0.08, 0.9]
KEY = {"name": "chroma", "color": "#00FF22", "tolerance": args[0], "softness": args[1], "spill": args[2]}


def fade_in_out(fade_in):
    """Opacity: up over fade_in, held, down over FADE_OUT."""
    return [{"t": fade_in[0], "v": 0.0}, {"t": fade_in[1], "v": 1.0, "ease": "ease-out"},
            {"t": FADE_OUT[0], "v": 1.0, "ease": "linear"}, {"t": FADE_OUT[1], "v": 0.0, "ease": "ease-in"}]


def fade_out():
    return [{"t": FADE_OUT[0], "v": 1.0}, {"t": FADE_OUT[1], "v": 0.0, "ease": "ease-in"}]


def measure(el):
    out = subprocess.run(["montagent", "measure", "presenter-cut.json", "--json", "--element", json.dumps(el)],
                         capture_output=True, text=True)
    if out.returncode:
        sys.exit(out.stdout + out.stderr)
    return json.loads(out.stdout)["measure"]


# ---- captions -------------------------------------------------------------
# Each chunk is laid out once, then written as one element per spoken word: in the
# element for word k, words 0..k are drawn and the rest are transparent, so every
# word appears on the frame its voice starts and the layout never moves.
CHUNKS = [            # lines of word indices
    [[0, 1, 2]],
    [[3, 4], [5, 6, 7]],
    [[8, 9, 10], [11, 12]],
    [[13, 14, 15], [16, 17]],
]
PAYOFF = [[18, 19, 20], [21, 22], [23, 24, 25]]


def runs_upto(lines, k, color):
    runs = []
    for li, line in enumerate(lines):
        for wi, idx in enumerate(line):
            runs.append({"text": W[idx][0], "color": color if idx <= k else color + "00"})
            if wi < len(line) - 1:
                runs.append({"text": " "})
        if li < len(lines) - 1:
            runs.append({"text": "\n"})
    return runs


def chunk_states(name, lines, end, size, line_height, color):
    order = [i for line in lines for i in line]
    base = {"x": 540, "y": BAND_MID, "origin": "center", "width": 984, "height": 0,
            "font": "bold", "size": size, "line_height": line_height, "color": color, "align": "center"}
    m = measure({**base, "runs": runs_upto(lines, order[-1], color)})
    els = []
    for n, k in enumerate(order):
        s = W[k][1]
        e = W[order[n + 1]][1] if n + 1 < len(order) else end
        el = {"id": f"{name}-w{n + 1}", "type": "text", "group": name, "start": s, "end": e, **base,
              "runs": runs_upto(lines, k, color), "caption": False}
        el["height"] = m["block_height"]
        els.append(el)
    return els, m


caption_els = []
for c, lines in enumerate(CHUNKS):
    nxt = CHUNKS[c + 1][0][0] if c + 1 < len(CHUNKS) else PAYOFF[0][0]
    els, _ = chunk_states(f"cap{c + 1}", lines, W[nxt][1], 72, 1.2, PAPER)
    caption_els += els

payoff_els, pm = chunk_states("payoff", PAYOFF, SCENE_END, 74, 1.1, INK)
emph_start = payoff_els[0]["start"]
payoff_els[0]["scale"] = [{"t": emph_start, "v": [0.92, 0.92]},
                          {"t": emph_start + 200, "v": [1.0, 1.0], "ease": "ease-out"}]
payoff_els[-1]["opacity"] = fade_out()
caption_els += payoff_els

# The one Signal shape: a box behind the payoff, sized from the measured block.
box_w, box_h = round(pm["advance_width"]) + 2 * 48, pm["block_height"] + 2 * 22
payoff_box = {"id": "payoff-box", "type": "rect", "start": emph_start, "end": SCENE_END,
              "x": 540, "y": BAND_MID, "origin": "center", "width": box_w, "height": box_h,
              "fill": SIGNAL, "radius": 28,
              "scale": [{"t": emph_start, "v": [0.6, 0.6]}, {"t": emph_start + 200, "v": [1.0, 1.0], "ease": "ease-out"}],
              "opacity": fade_in_out((emph_start, emph_start + 120))}

# ---- scene ----------------------------------------------------------------
INTRO = (0, 280)
card = {"id": "card", "type": "rect", "start": 0, "end": SCENE_END,
        "x": CARD["x"], "y": CARD["y"], "origin": "top-left", "width": CARD["w"], "height": CARD["h"],
        "fill": PAPER, "radius": 56, "opacity": fade_in_out(INTRO)}

presenter = {"id": "presenter", "type": "video", "start": T0, "end": T0 + TAKE_MS,
             "source": "presenter/take-2.mp4", "source_start": 0, "source_end": TAKE_MS,
             "x": VID_X, "y": VID_Y, "origin": "top-left", "width": 1920, "height": 1080, "fit": "literal",
             "clip": [CARD["x"], CARD["y"], CARD["w"], CARD["h"]],
             "opacity": fade_in_out((T0, T0 + 160)), "volume": 1.8,
             "effects": [KEY]}

LOCK_W = 236
header = {"id": "header-lockup", "type": "image", "start": 0, "end": SCENE_END,
          "source": "brand/lockup-on-dark.png", "x": 540, "y": 70, "origin": "center",
          "width": LOCK_W, "height": round(LOCK_W * 623 / 2694), "fit": "literal",
          "opacity": fade_in_out(INTRO)}

# ---- close: the mark over the wordmark --------------------------------------
MARK = 300
WORD_W = 640
close_scale = [{"t": CLOSE_IN[0], "v": [0.94, 0.94]}, {"t": CLOSE_IN[1] + 200, "v": [1.0, 1.0], "ease": "ease-out"}]
close_mark = {"id": "close-mark", "type": "image", "start": CLOSE_IN[0], "end": END,
              "source": "brand/mark-on-dark.png", "x": 540, "y": 560, "origin": "center",
              "width": MARK, "height": MARK, "fit": "literal", "scale": close_scale,
              "opacity": [{"t": CLOSE_IN[0], "v": 0.0}, {"t": CLOSE_IN[1], "v": 1.0, "ease": "ease-out"}]}
close_word = {"id": "close-wordmark", "type": "image", "start": CLOSE_IN[0], "end": END,
              "source": "brand/wordmark-on-dark.png", "x": 540, "y": 830, "origin": "center",
              "width": WORD_W, "height": round(WORD_W * 419 / 1998), "fit": "literal",
              "opacity": [{"t": CLOSE_IN[0] + 120, "v": 0.0}, {"t": CLOSE_IN[1] + 160, "v": 1.0, "ease": "ease-out"}]}

# ---- sound -----------------------------------------------------------------
music = {"id": "music-bed", "type": "audio", "start": 0, "end": END,
         "source": "music/bed-120bpm.wav", "source_start": 0, "source_end": END,
         "volume": [{"t": 0, "v": 0.0}, {"t": 200, "v": 0.15, "ease": "ease-out"},
                    {"t": 9700, "v": 0.15, "ease": "linear"}, {"t": 10200, "v": 0.44, "ease": "ease-in-out"},
                    {"t": 11000, "v": 0.44, "ease": "linear"}, {"t": 11880, "v": 0.0, "ease": "ease-in"}]}

project = {
    "frame": {"width": 1080, "height": 1350}, "fps": 25, "background": INK, "duration": END,
    "output": "deliverable.mp4",
    "fonts": {"bold": [{"file": "fonts/Inter-Bold.ttf"}], "regular": [{"file": "fonts/Inter-Regular.ttf"}]},
    "fontVendor": json.load(open("presenter-cut.json"))["fontVendor"],
    "tracks": [
        {"name": "card", "layer": 10, "elements": [card]},
        {"name": "presenter", "layer": 20, "elements": [presenter]},
        {"name": "brand", "layer": 30, "elements": [header, close_mark]},
        {"name": "brand-word", "layer": 31, "elements": [close_word]},
        {"name": "payoff-box", "layer": 40, "elements": [payoff_box]},
        {"name": "captions", "layer": 50, "elements": caption_els},
        {"name": "music", "layer": 0, "elements": [music]},
    ],
}
json.dump(project, open("presenter-cut.json", "w"), indent=2, ensure_ascii=False)
subprocess.run(["montagent", "fmt", "presenter-cut.json"], check=True, capture_output=True)
print("payoff box", box_w, box_h, "captions", len(caption_els))
