#!/usr/bin/env python3
"""Generate ad.montagent.json from the beat sheet. 120 BPM, downbeat at 0, beat = 500 ms."""
import json, subprocess

PROJECT = "ad.montagent.json"
INK, PAPER, SIGNAL = "#101418", "#F5F0E6", "#FF5A36"
W = H = 1080
END = 15000
LAST = 14966          # just before the last drawn frame (14966.67 ms), so that frame is at 0

POP = [0.34, 1.56, 0.64, 1]
SETTLE = [0.16, 1, 0.3, 1]

# Beat sheet (ms)
HOOK1, HOOK2, HOOK_ACCENT = 0, 500, 1000
PROOF = 2500          # proof cut; the diff lands at 4000 (source 33.5 s)
RESULT = 6500         # result cut
UNDERLINE = 7500      # accent shape under the new subtitle
CLAIM = 9000          # claim cut (ground back to Ink)
CLAIM_ACCENT = 9500
BRAND = 12500         # brand cut (ground to Paper)
WORDMARK = 12750
TAGLINE = 13000
FADE = 14000          # drums stop; music fades to 0 on the last frame

DIFF_AT_SOURCE = 33540   # source frame 838 (25 fps) is the first with the diff
SRC_START = DIFF_AT_SOURCE - (4000 - PROOF)


def measure(spec):
    out = subprocess.run(["montagent", "measure", PROJECT, "--json", "--elements", json.dumps([spec])],
                         capture_output=True, text=True, check=True).stdout
    return json.loads(out)["measure"]["results"][0]["ok"]


def text(id_, start, end, x, y, font, size, color, runs, lh=1.1, extra=None):
    spec = {"font": font, "size": size, "line_height": lh, "runs": runs}
    m = measure(spec)
    el = {"id": id_, "type": "text", "start": start, "end": end, "x": x, "y": y, "origin": "center",
          "width": int(m["advance_width"]) + 8, "height": m["block_height"], "font": font, "size": size,
          "line_height": lh, "color": color, "align": "center", "runs": runs}
    if extra:
        el.update(extra)
    el["caption"] = False
    return el


def rise(t, y, dy, dur, ease=SETTLE):
    return [{"t": t, "v": y + dy}, {"t": t + dur, "v": y, "ease": ease}]


def fade_in(t, dur):
    return [{"t": t, "v": 0.0}, {"t": t + dur, "v": 1.0, "ease": "ease-out"}]


def pop_scale(t, s0, dur=400):
    return [{"t": t, "v": [s0, s0]}, {"t": t + dur, "v": [1.0, 1.0], "ease": POP}]


SHADOW = {"name": "shadow", "dx": 0, "dy": 24, "radius": 48, "color": INK, "opacity": 0.3}

tracks = []

# Grounds: Ink is the project background; Paper for proof/result and for the brand.
tracks.append({"name": "ground", "layer": 1, "elements": [
    {"id": "paper-proof", "type": "rect", "start": PROOF, "end": CLAIM, "x": 0, "y": 0, "origin": "top-left",
     "width": W, "height": H, "fill": PAPER},
    {"id": "paper-brand", "type": "rect", "start": BRAND, "end": END, "x": 0, "y": 0, "origin": "top-left",
     "width": W, "height": H, "fill": PAPER},
]})

# Hook: two lines, one per beat; "video." turns Signal on the next beat.
HOOK_SIZE = 128
hook_y1, hook_y2 = 540 - 70, 540 + 70
tracks.append({"name": "hook-1", "layer": 20, "elements": [
    text("hook-1", HOOK1, PROOF, 540, hook_y1, "bold", HOOK_SIZE, PAPER, [{"text": "Your AI agent"}],
         extra={"y": rise(HOOK1, hook_y1, 30, 400, POP), "scale": pop_scale(HOOK1, 0.7),
                "opacity": fade_in(HOOK1, 250)}),
]})
tracks.append({"name": "hook-2", "layer": 21, "elements": [
    text("hook-2", HOOK2, PROOF, 540, hook_y2, "bold", HOOK_SIZE, PAPER,
         [{"text": "edits "}, {"text": "video.", "highlight": {"start": HOOK_ACCENT, "end": PROOF, "color": SIGNAL}}],
         extra={"y": rise(HOOK2, hook_y2, 30, 400, POP), "scale": pop_scale(HOOK2, 0.7),
                "opacity": fade_in(HOOK2, 250)}),
]})

# Proof: the real session, cropped to the left of the diff, 1.25x, as a rounded card.
S = 1.25
CARD_X, CARD_Y, CARD_W, CARD_H = 60, 236, 960, 752
SRC_X, SRC_Y = 24, 92
vx = CARD_X - round(SRC_X * S)
vy = CARD_Y - round(SRC_Y * S)
PROOF_LABEL_Y = 150
RESULT_CARD_Y, RESULT_LABEL_Y = 316, 230
LABEL_SIZE = 56
tracks.append({"name": "label", "layer": 30, "elements": [
    text("label-proof", PROOF, RESULT, 540, PROOF_LABEL_Y, "bold", LABEL_SIZE, INK,
         [{"text": "It edits the project file,"}],
         extra={"y": rise(PROOF, PROOF_LABEL_Y, 24, 350), "opacity": fade_in(PROOF, 250)}),
    text("label-result", RESULT, CLAIM, 540, RESULT_LABEL_Y, "bold", LABEL_SIZE, INK,
         [{"text": "then renders the result."}],
         extra={"y": rise(RESULT, RESULT_LABEL_Y, 24, 350), "opacity": fade_in(RESULT, 250)}),
]})
tracks.append({"name": "proof", "layer": 10, "elements": [
    {"id": "session", "type": "video", "start": PROOF, "end": RESULT, "source": "screen/session.mp4",
     "source_start": SRC_START, "source_end": SRC_START + (RESULT - PROOF),
     "x": vx, "y": rise(PROOF, vy, 40, 500), "origin": "top-left", "width": round(1920 * S), "height": round(1080 * S),
     "fit": "contain", "opacity": fade_in(PROOF, 250), "volume": 0.0,
     "effects": [{"name": "mask", "shape": "rect", "x": round(SRC_X * S), "y": round(SRC_Y * S),
                  "width": CARD_W, "height": CARD_H, "radius": 28}, SHADOW]},
]})

# Result: the rendered frame, centre 800x450 of it at 1.2x, so the 16:9 frame stays 16:9.
RS = 1.2
rw, rh = round(960 * RS), round(540 * RS)
mx, my = round(80 * RS), round(45 * RS)
r_cx, r_cy = 540, RESULT_CARD_Y + 270
tracks.append({"name": "result", "layer": 11, "elements": [
    {"id": "rendered", "type": "image", "start": RESULT, "end": CLAIM, "source": "stills/session-02.png",
     "x": r_cx, "y": rise(RESULT, r_cy, 40, 500), "origin": "center", "width": rw, "height": rh, "fit": "contain",
     "scale": [{"t": RESULT, "v": [0.94, 0.94]}, {"t": RESULT + 500, "v": [1.0, 1.0], "ease": SETTLE}],
     "opacity": fade_in(RESULT, 250),
     "effects": [{"name": "mask", "shape": "rect", "x": mx, "y": my, "width": 960, "height": 540, "radius": 24}, SHADOW]},
]})

# Accent shape: a Signal underline under the new subtitle, grown from its left edge.
SUB_X0, SUB_X1, SUB_UNDER_Y = 337, 623, 380      # in the still's own pixels
el_left, el_top = r_cx - rw // 2, r_cy - rh // 2
ux0 = el_left + round(SUB_X0 * RS)
ux1 = el_left + round(SUB_X1 * RS)
uy = el_top + round(SUB_UNDER_Y * RS)
tracks.append({"name": "underline", "layer": 12, "elements": [
    {"id": "underline", "type": "rect", "start": UNDERLINE, "end": CLAIM, "x": ux0, "y": uy, "origin": "center-left",
     "width": ux1 - ux0, "height": 6, "fill": SIGNAL,
     "scale": [{"t": UNDERLINE, "v": [0.0, 1.0]}, {"t": UNDERLINE + 450, "v": [1.0, 1.0], "ease": SETTLE}]},
]})

# Claim: one line of copy, "file" turns Signal on the next beat.
CLAIM_SIZE = 92
tracks.append({"name": "claim", "layer": 20, "elements": [
    text("claim", CLAIM, BRAND, 540, 540, "bold", CLAIM_SIZE, PAPER,
         [{"text": "Video becomes a "}, {"text": "file", "highlight": {"start": CLAIM_ACCENT, "end": BRAND, "color": SIGNAL}},
          {"text": "\nyour agent can edit."}], lh=1.2,
         extra={"y": rise(CLAIM, 540, 30, 450), "opacity": fade_in(CLAIM, 300)}),
]})

# Brand: mark pops, wordmark settles in, tagline follows; all still from 13300.
MARK = 300
mark_y, word_y, tag_y = 410, 650, 790
word_w = 620
word_h = round(word_w * 419 / 1998)
tracks.append({"name": "brand-mark", "layer": 40, "elements": [
    {"id": "mark", "type": "image", "start": BRAND, "end": END, "source": "brand/mark.png", "x": 540, "y": mark_y,
     "origin": "center", "width": MARK, "height": MARK, "fit": "contain", "scale": pop_scale(BRAND, 0.0, 450)},
]})
tracks.append({"name": "brand-wordmark", "layer": 41, "elements": [
    {"id": "wordmark", "type": "image", "start": WORDMARK, "end": END, "source": "brand/wordmark.png", "x": 540,
     "y": rise(WORDMARK, word_y, 30, 400), "origin": "center", "width": word_w, "height": word_h, "fit": "contain",
     "opacity": fade_in(WORDMARK, 250)},
]})
tracks.append({"name": "brand-tagline", "layer": 42, "elements": [
    text("tagline", TAGLINE, END, 540, tag_y, "regular", 40, INK, [{"text": "The video editor your agent drives."}],
         lh=1.2, extra={"y": rise(TAGLINE, tag_y, 20, 300), "opacity": fade_in(TAGLINE, 250)}),
]})

# Music: under the whole piece, fading to 0 on the last drawn frame.
tracks.append({"name": "music", "layer": 0, "elements": [
    {"id": "bed", "type": "audio", "start": 0, "end": END, "source": "music/bed-120bpm.wav", "source_start": 0,
     "source_end": END, "volume": [{"t": FADE, "v": 0.8}, {"t": LAST, "v": 0.0, "ease": "linear"}]},
]})

p = json.load(open(PROJECT))
p["tracks"] = tracks
json.dump(p, open(PROJECT, "w"), indent=2)
print("wrote", PROJECT)
