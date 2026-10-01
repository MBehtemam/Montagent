#!/usr/bin/env python3
"""Build hoot.json: the set, crate, laptop, screen, tag and voice, then the owl spec.

Run:  python3 build.py && bake && python3 post.py   (see make.sh)
"""
import json, math, sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from ik import ik, fk, rot, NECK, ROOT

OUT = "hoot.base.json"
FONT_SHA = "288316099b1e0a47a4716d159098005eef7c0066921f34e3200393dbdb01947f"

# --- the owl's placement --------------------------------------------------
X, Y, S = 690, 1000, 0.58            # torso pivot (feet) on the frame; frame px per drawing px

def to_drawing(fx, fy):
    return (ROOT[0] + (fx - X) / S, ROOT[1] + (fy - Y) / S)

def to_frame(p):
    return (X + (p[0] - ROOT[0]) * S, Y + (p[1] - ROOT[1]) * S)

# --- crate and laptop -----------------------------------------------------
LW, LH = 425, 340                    # laptop at ~0.6 of its 708x566 image
sx, sy = LW / 708, LH / 566
L = 840
CRATE_BOTTOM = 968
CRATE_H = 100
CRATE_TOP = CRATE_BOTTOM - CRATE_H
T = CRATE_TOP - round(563 * sy) + 2  # the laptop's base sits on the crate's lid
CX0, CX1 = L - 10, L + LW + 10       # crate a little wider than the laptop

# The screen: x 120-587, y 41-312 of the image; clip rounded inward.
scr_x0 = math.ceil(L + 120 * sx); scr_x1 = math.floor(L + 587 * sx)
scr_y0 = math.ceil(T + 41 * sy);  scr_y1 = math.floor(T + 312 * sy)
SW, SH = scr_x1 - scr_x0, scr_y1 - scr_y0
SCX, SCY = (scr_x0 + scr_x1) // 2, (scr_y0 + scr_y1) // 2

# --- times (timeline ms) --------------------------------------------------
VOICE = 1000
TAG_IN = 3133        # first drawn frame at or after "two" (3112)
FIXED = 5600         # first drawn frame at or after "Fixed" (5587)
MARK = 7333          # first drawn frame after "beat" ends (7324)
END = 8000

POP = [0.34, 1.56, 0.64, 1]
INK = "#101418"; PAPER = "#F5F0E6"; SIGNAL = "#FF5A36"
LINE = "#3D2B22"; WOOD = "#C98A55"; WOOD_D = "#A86D3E"


def el(**kw):
    return kw


def crate():
    els = []
    w = CX1 - CX0
    els.append(el(id="crate-shadow", type="ellipse", start=0, end=END, x=(CX0 + CX1) // 2, y=CRATE_BOTTOM,
                  origin="center", width=w + 60, height=28, fill="#5A3A2240"))
    els.append(el(id="crate-body", type="rect", start=0, end=END, x=CX0, y=CRATE_TOP, origin="top-left",
                  width=w, height=CRATE_H, fill=WOOD, stroke=LINE, stroke_width=5, radius=6))
    # planks: two seams, and a darker post at each end
    for i, yy in enumerate((CRATE_TOP + 34, CRATE_TOP + 67)):
        els.append(el(id=f"crate-seam-{i}", type="rect", start=0, end=END, x=CX0 + 4, y=yy - 2, origin="top-left",
                      width=w - 8, height=4, fill=LINE))
    for i, xx in enumerate((CX0 + 4, CX1 - 4 - 34)):
        els.append(el(id=f"crate-post-{i}", type="rect", start=0, end=END, x=xx, y=CRATE_TOP + 4, origin="top-left",
                      width=34, height=CRATE_H - 8, fill=WOOD_D, stroke=LINE, stroke_width=4))
    return els


def screen():
    k = max(SW / 960, SH / 540)
    iw, ih = round(960 * k), round(540 * k)
    clip = [scr_x0, scr_y0, SW, SH]
    shot = el(id="screen-session", type="image", start=0, end=MARK, source="stills/session-02.png",
              x=SCX, y=SCY, origin="center", width=iw, height=ih, fit="cover", clip=clip)
    ground = el(id="screen-paper", type="rect", start=MARK, end=END, x=scr_x0, y=scr_y0, origin="top-left",
                width=SW, height=SH, fill=PAPER)
    m = round(SH * 0.72)
    mark = el(id="screen-mark", type="image", start=MARK, end=END, source="brand/mark.png",
              x=SCX, y=SCY, origin="center", width=m, height=m, fit="contain",
              scale=[{"t": MARK, "v": [0.8, 0.8]}, {"t": MARK + 233, "v": [1.0, 1.0], "ease": POP}])
    return shot, ground, mark


TAG_X, TAG_Y, TAG_W, TAG_H = 1150, 432, 270, 84


def tag(ident, text, start, end, frm):
    sc = [{"t": start, "v": [frm, frm]}, {"t": start + 300, "v": [1.0, 1.0], "ease": POP}]
    pill = el(id=f"{ident}-pill", type="rect", start=start, end=end, x=TAG_X, y=TAG_Y, origin="center",
              width=TAG_W, height=TAG_H, fill=SIGNAL, radius=TAG_H // 2, scale=sc,
              effects=[{"name": "shadow", "dx": 0, "dy": 6, "radius": 8, "color": "#3D2B22", "opacity": 0.35}])
    txt = el(id=f"{ident}-text", type="text", start=start, end=end, x=TAG_X, y=TAG_Y, origin="center",
             width=TAG_W - 20, height=60, font="tag", size=48, color=PAPER, align="center",
             caption=False, runs=[{"text": text}], scale=sc)
    return pill, txt


def main():
    shot, ground, mark = screen()
    p1, t1 = tag("tag-late", "+2 frames", TAG_IN, FIXED, 0.0)
    p2, t2 = tag("tag-fixed", "0 frames", FIXED, END, 0.8)
    proj = {
        "frame": {"width": 1920, "height": 1080}, "fps": 30, "background": "#101418", "duration": END,
        "output": "deliverable.mp4",
        "fonts": {"tag": [{"file": "fonts/Inter-Bold.ttf"}]},
        "fontVendor": {"fonts/Inter-Bold.ttf": {"licence": "OFL-1.1", "source": "https://github.com/rsms/inter", "sha256": FONT_SHA}},
        "tracks": [
            {"name": "set", "layer": 0, "elements": [
                el(id="study", type="image", start=0, end=END, source="character/study.png", x=0, y=0,
                   origin="top-left", width=1920, height=1080, fit="literal")]},
            {"name": "shadows", "layer": 1, "elements": [owl_shadow()]},
            {"name": "crate", "layer": 2, "elements": crate()[:1]},
            {"name": "crate-body", "layer": 3, "elements": crate()[1:2]},
        ] + [{"name": e["id"], "layer": 5 if "post" in e["id"] else 4, "elements": [e]} for e in crate()[2:]] + [
            {"name": "laptop", "layer": 6, "elements": [
                el(id="laptop", type="image", start=0, end=END, source="character/laptop.png", x=L, y=T,
                   origin="top-left", width=LW, height=LH, fit="literal")]},
            {"name": "screen-ground", "layer": 7, "elements": [shot, ground]},
            {"name": "screen-mark", "layer": 8, "elements": [mark]},
            {"name": "tag-pill", "layer": 40, "elements": [p1, p2]},
            {"name": "tag-text", "layer": 41, "elements": [t1, t2]},
            {"name": "voice", "layer": 0, "elements": [
                el(id="voice", type="audio", start=VOICE, end=VOICE + 6672, source="character/voice/line-3.wav",
                   source_start=0, source_end=6672)]},
        ],
    }
    json.dump(proj, open(OUT, "w"), indent=1)
    json.dump(spec(), open("hoot.spec.json", "w"), indent=1)


HOP0, HOP1, HOP_H = 5533, 6000, 95


def owl_shadow():
    mid = (HOP0 + HOP1) // 2
    sc = [{"t": 0, "v": [1.0, 1.0]}, {"t": HOP0, "v": [1.0, 1.0], "ease": "linear"},
          {"t": mid, "v": [0.7, 0.7], "ease": "ease-out"}, {"t": HOP1, "v": [1.0, 1.0], "ease": "ease-in"}]
    return el(id="owl-shadow", type="ellipse", start=0, end=END, x=X, y=Y + 4, origin="center",
              width=330, height=44, fill="#5A3A2250", scale=sc)


def spec():
    # thinking: the left hand just under the beak, head tilted to the owl's left (our left)
    tilt = -9
    think_ual, think_fal = ik("l", 0, (478, 690), 1)
    # reaching to the laptop: hand hovering over the keyboard's left half
    lean = 3
    kb_y = T + 370 * sy         # top of the keys
    hover = to_drawing(L + 118, kb_y - 4)
    r_ual, r_fal = ik('r', lean, hover, -1)
    print(f"think ual {think_ual:.1f} fal {think_fal:.1f}; reach {r_ual:.1f} {r_fal:.1f}", file=sys.stderr)
    poses = {
        "down": {"upper_arm_left": -50, "upper_arm_right": 50, "forearm_left": -10, "forearm_right": 10,
                 "head": 0, "torso": 0},
        "think": {"upper_arm_left": round(think_ual, 1), "forearm_left": round(think_fal, 1), "head": tilt},
        "think2": {"head": tilt + 3},
        "think3": {"head": tilt - 1},
        "left_down": {"upper_arm_left": -50, "forearm_left": -10, "head": 0},
        "reach": {"torso": lean, "head": 6, "upper_arm_right": round(r_ual, 1), "forearm_right": round(r_fal, 1)},
        "tap": {"upper_arm_right": round(r_ual + 2, 1), "forearm_right": round(r_fal + 20, 1)},
        "lift": {"upper_arm_right": round(r_ual - 2, 1), "forearm_right": round(r_fal - 6, 1)},
        "cheer": {"torso": 0, "head": 0, "upper_arm_left": 60, "upper_arm_right": -60,
                  "forearm_left": 30, "forearm_right": -30},
        "arms_down": {"upper_arm_left": -50, "upper_arm_right": 50, "forearm_left": -10, "forearm_right": 10},
        "nod": {"head": 4}, "level": {"head": 0},
    }
    moves = [
        {"t": 0, "pose": "down"},
        {"t": 1300, "pose": "think", "in": 450},
        {"t": 2000, "pose": "think2", "in": 300},     # a beat of thought on "caption"
        {"t": 2800, "pose": "think3", "in": 350},     # and on "two frames"
        {"t": 4100, "pose": "left_down", "in": 350},  # lets go on "late"
        {"t": 4450, "pose": "reach", "in": 400, "ease": [0.34, 1.3, 0.64, 1]},
        {"t": 4600, "pose": "tap", "in": 133},
        {"t": 4767, "pose": "lift", "in": 133},
        {"t": 4933, "pose": "tap", "in": 133},
        {"t": 5100, "pose": "lift", "in": 133},
        {"t": 5667, "pose": "cheer", "in": 467},
        {"t": 6900, "pose": "arms_down", "in": 400},
        {"t": 7067, "pose": "nod", "in": 167},
        {"t": 7400, "pose": "level", "in": 300},
    ]
    return {
        "prefix": "owl", "layer": 10, "start": 0, "end": END, "scale": S,
        "place": [{"t": 0, "x": X, "y": Y}],
        "hops": [{"from": HOP0, "to": HOP1, "height": HOP_H}],
        "poses": poses, "moves": moves,
        "voice": {"id": "voice", "visemes": "character/voice/line-3.json", "lead": 0},
        "min_mouth": 2,
        "blinks": {"part": "eyes_closed", "at": [633, 2433, 4267, 6267, 7633], "frames": 3},
    }


if __name__ == "__main__":
    main()
