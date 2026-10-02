#!/usr/bin/env python3
"""Generate hoot.montagent.json: Hoot explains editing a word (brief H6).

Montagent transforms are flat, so the rig's hierarchy is solved here: every
frame, forward kinematics places each part's pivot and writes it as a keyframe.
"""
import json, math, os

HERE = os.path.dirname(os.path.abspath(__file__))
PROJ = os.path.join(HERE, "hoot.montagent.json")
rig = json.load(open(os.path.join(HERE, "character/rig.json")))
line = json.load(open(os.path.join(HERE, "character/voice/line-2.json")))

FPS, DUR = 30, 10000
FRAMES = [i * 1000 // FPS for i in range(DUR * FPS // 1000)]
S = 0.65                      # owl scale
HOME = (560, 1000)            # feet (torso pivot) on the study floor
V0 = 1000                     # voice starts here
P = {k: v["pivot"] for k, v in rig["parts"].items()}

# ---- props layout --------------------------------------------------------
LAP_CX, LAP_BOTTOM = 1250, 1000
LAP_W, LAP_H = 708, 566
LAP_L, LAP_T = LAP_CX - LAP_W // 2, LAP_BOTTOM - LAP_H
SCR_CX = LAP_L + (120 + 587) / 2
SCR_CY = LAP_T + (41 + 312) / 2
SCR_W, SCR_H = 467, 263       # session-01 at 16:9 inside the 467x271 screen
CARD_CX, CARD_CY = 1250, 212  # rendered video card, final place
CARD_W, CARD_H = 480, 270

def sstep(u):
    u = max(0.0, min(1.0, u))
    return u * u * (3 - 2 * u)

def rot(v, deg):
    a = math.radians(deg)
    c, s = math.cos(a), math.sin(a)
    return (v[0] * c - v[1] * s, v[0] * s + v[1] * c)

def shoulder_world(side):
    p = P["upper_arm_" + side]
    return (HOME[0] + S * (p[0] - P["torso"][0]), HOME[1] + S * (p[1] - P["torso"][1]))

def aim(side, target, bend):
    """Upper-arm and forearm angles that point the right arm at target."""
    sx, sy = shoulder_world(side)
    phi = math.degrees(math.atan2(target[1] - sy, target[0] - sx))
    return phi - 25.8 - bend * 0.5, bend

# ---- poses (relative joint angles, degrees clockwise) ---------------------
# left arm: + raises, forearm + bends the hand up; right arm: - raises,
# forearm - bends the hand up. Those are the only directions the elbows bend.
REST = dict(torso=0, head=0, uaL=0, faL=0, uaR=0, faR=0)
def pose(**kw):
    d = dict(REST); d.update(kw); return d

ua_scr, fa_scr = aim("right", (SCR_CX - 40, SCR_CY), -12)
ua_card, fa_card = aim("right", (CARD_CX - 60, CARD_CY + 40), -10)

KEYS = [
    (0, pose(torso=6)),
    (850, pose(torso=6)),
    (1000, pose(torso=-1, uaL=-4, uaR=4)),
    # "Want to change one word in a video?" — head cocked, palms up
    (1350, pose(torso=-2, head=-11, uaL=16, faL=52, uaR=-16, faR=-52)),
    (2100, pose(torso=-2, head=-9, uaL=14, faL=48, uaR=-14, faR=-48)),
    (2450, pose(torso=-3, head=-15, uaL=20, faL=60, uaR=-20, faR=-60)),
    (2800, pose(torso=-3, head=-14, uaL=19, faL=58, uaR=-19, faR=-58)),
    # pause: settle
    (3080, pose(torso=0, head=-2, uaL=-6, faL=10, uaR=-4, faR=-6)),
    # "Just edit the file" — point at the laptop, glance at it
    (3400, pose(torso=2, head=7, uaL=-22, faL=24, uaR=ua_scr, faR=fa_scr)),
    (4300, pose(torso=2, head=8, uaL=-22, faL=24, uaR=ua_scr, faR=fa_scr)),
    # "and Montagent renders it again" — point up at the rendered video
    (4800, pose(torso=1, head=12, uaL=-18, faL=22, uaR=ua_card, faR=fa_card)),
    (6300, pose(torso=1, head=11, uaL=-18, faL=22, uaR=ua_card, faR=fa_card)),
    # brand: turn back to the viewer, present the brand, wave
    (6900, pose(torso=0, head=-4, uaL=58, faL=10, uaR=-14, faR=-38)),
    (8700, pose(torso=0, head=-4, uaL=58, faL=10, uaR=-14, faR=-38)),
    (9300, pose(torso=0, head=2, uaL=-4, faL=8, uaR=-12, faR=-30)),
    (10000, pose(torso=0, head=2, uaL=-4, faL=8, uaR=-12, faR=-30)),
]

def keyed(t):
    for (t0, a), (t1, b) in zip(KEYS, KEYS[1:]):
        if t0 <= t <= t1:
            u = sstep((t - t0) / (t1 - t0))
            return {k: a[k] + (b[k] - a[k]) * u for k in a}
    return dict(KEYS[-1][1])

NODS = [w["start"] + V0 for w in line["words"] if w["word"] in
        ("change", "word", "video", "edit", "file", "Montagent", "again")]

def state(t):
    q = keyed(t)
    # entrance: hop in from the left, three hops, wings flapping
    if t < 900:
        u = t / 900
        x = -60 + (HOME[0] + 60) * (1 - (1 - u) ** 2)
        hop = abs(math.sin(math.pi * 3 * u))
        y = HOME[1] - 70 * hop
        q["uaL"] += 22 * hop; q["uaR"] -= 22 * hop
    else:
        x, y = HOME
    # idle: sway about the feet, breathing head, drifting arms
    q["torso"] += 1.3 * math.sin(2 * math.pi * t / 2600)
    q["head"] += 1.8 * math.sin(2 * math.pi * t / 1900 + 1.0)
    q["uaL"] += 2.0 * math.sin(2 * math.pi * t / 2300 + 0.4)
    q["uaR"] -= 2.0 * math.sin(2 * math.pi * t / 2100 + 1.9)
    # head nods on stressed words
    for n in NODS:
        q["head"] += 3.5 * math.exp(-(((t - n - 60) / 110.0) ** 2))
    # wave (left forearm swings, always bending the hand up)
    w = sstep((t - 6900) / 250) * (1 - sstep((t - 8500) / 250))
    q["faL"] += w * (14 + 14 * math.sin(2 * math.pi * (t - 6900) / 520))
    return (x, y), q

def fk(t):
    root, q = state(t)
    tp = P["torso"]
    def world(d):  # a drawing point carried by the torso
        v = rot((d[0] - tp[0], d[1] - tp[1]), q["torso"])
        return (root[0] + S * v[0], root[1] + S * v[1])
    out = {"torso": (root, q["torso"]), "head": (world(P["head"]), q["torso"] + q["head"])}
    for side, ua, fa in (("left", "uaL", "faL"), ("right", "uaR", "faR")):
        up = "upper_arm_" + side; fo = "forearm_" + side
        sp = world(P[up]); ang = q["torso"] + q[ua]
        v = rot((P[fo][0] - P[up][0], P[fo][1] - P[up][1]), ang)
        out[up] = (sp, ang)
        out[fo] = ((sp[0] + S * v[0], sp[1] + S * v[1]), ang + q[fa])
    return out

POSES = {t: fk(t) for t in FRAMES}

# ---- element writers -------------------------------------------------------
def kf(series, ease="linear", rnd=None):
    """Collapse a per-frame series into keyframes, dropping redundant ones."""
    recs = []
    for i, (t, v) in enumerate(series):
        if recs and i + 1 < len(series) and v == recs[-1]["v"] and series[i + 1][1] == v:
            continue
        r = {"t": t, "v": v}
        if recs: r["ease"] = "step" if v == recs[-1]["v"] else ease
        recs.append(r)
    if all(r["v"] == recs[0]["v"] for r in recs):
        return recs[0]["v"]
    return recs

def step_kf(series):
    recs = []
    for t, v in series:
        if recs and recs[-1]["v"] == v:
            continue
        r = {"t": t, "v": v}
        if recs: r["ease"] = "step"
        recs.append(r)
    return recs[0]["v"] if len(recs) == 1 else recs

def part_el(eid, name, joint, opacity=None):
    part = rig["parts"][name]
    xs = [(t, round(POSES[t][joint][0][0])) for t in FRAMES]
    ys = [(t, round(POSES[t][joint][0][1])) for t in FRAMES]
    rs = [(t, round(POSES[t][joint][1], 2)) for t in FRAMES]
    el = {"id": eid, "type": "image", "start": 0, "end": DUR,
          "source": "character/" + part["file"], "x": kf(xs), "y": kf(ys),
          "origin": "center", "width": part["width"], "height": part["height"],
          "fit": "literal", "scale": [S, S], "rotation": kf(rs)}
    if opacity is not None:
        el["opacity"] = opacity
    return el

# mouths from visemes, sampled on the frame grid
def mouth_at(t):
    tau = t - V0
    cur = None
    for ms, vid in line["visemes"]:
        if ms <= tau:
            cur = rig["visemes"][str(vid)]
    return cur if 0 <= tau <= line["duration_ms"] else None

BLINKS = [420, 2760, 5640, 8950]
def blink_at(t):
    return any(b <= t < b + 100 for b in BLINKS)

def tracks():
    T = []
    def track(name, layer, els):
        T.append({"name": name, "layer": layer, "elements": els})

    track("study", 0, [{"id": "study", "type": "image", "start": 0, "end": DUR,
                        "source": "character/study.png", "x": 0, "y": 0, "origin": "top-left",
                        "width": 1920, "height": 1080, "fit": "literal"}])

    # laptop group: fades and rises in on "Just", out before the brand
    def fade(t_in, t_out, d_in=300, d_out=350):
        return [{"t": t_in, "v": 0.0}, {"t": t_in + d_in, "v": 1.0, "ease": "ease-out"},
                {"t": t_out - d_out, "v": 1.0, "ease": "step"}, {"t": t_out - 20, "v": 0.0, "ease": "ease-in"}]
    def rise(y, t_in, d=300, dy=40):
        return [{"t": t_in, "v": y + dy}, {"t": t_in + d, "v": y, "ease": "ease-out"}]
    L_IN, L_OUT = 2950, 6950
    track("laptop", 10, [{"id": "laptop", "type": "image", "start": L_IN, "end": L_OUT,
        "source": "character/laptop.png", "x": LAP_CX, "y": rise(LAP_BOTTOM, L_IN),
        "origin": "bottom-center", "width": LAP_W, "height": LAP_H, "fit": "literal",
        "opacity": fade(L_IN, L_OUT)}])
    track("screen", 11, [{"id": "screen-file", "type": "image", "start": L_IN, "end": L_OUT,
        "source": "stills/session-01.png", "x": round(SCR_CX), "y": rise(round(SCR_CY), L_IN),
        "origin": "center", "width": SCR_W, "height": SCR_H, "fit": "literal",
        "opacity": fade(L_IN, L_OUT)}])
    # the edit: a Signal outline around the diff of the project file
    k = SCR_W / 1920
    dx0, dy0, dx1, dy1 = 108, 378, 1782, 642
    hx = round(SCR_CX + ((dx0 + dx1) / 2 - 960) * k)
    hy = round(SCR_CY + ((dy0 + dy1) / 2 - 540) * k)
    track("edit", 12, [{"id": "edit-mark", "type": "rect", "start": 3350, "end": L_OUT,
        "x": hx, "y": hy, "origin": "center",
        "width": round((dx1 - dx0) * k) + 10, "height": round((dy1 - dy0) * k) + 10,
        "stroke": "#FF5A36", "stroke_width": 4, "radius": 6,
        "scale": [{"t": 3350, "v": [1.25, 1.6]}, {"t": 3600, "v": [1.0, 1.0], "ease": "ease-out"}],
        "opacity": [{"t": 3350, "v": 0.0}, {"t": 3550, "v": 1.0, "ease": "ease-out"},
                    {"t": L_OUT - 350, "v": 1.0, "ease": "step"}, {"t": L_OUT - 20, "v": 0.0, "ease": "ease-in"}]}])
    # the render: the video card lifts out of the screen
    C_IN = 4600
    cy = [{"t": C_IN, "v": round(SCR_CY)}, {"t": C_IN + 550, "v": CARD_CY, "ease": [0.2, 0.0, 0.2, 1.08]}]
    cs = [{"t": C_IN, "v": [0.9, 0.9]}, {"t": C_IN + 550, "v": [1.0, 1.0], "ease": "ease-out"}]
    cop = [{"t": C_IN, "v": 0.0}, {"t": C_IN + 150, "v": 1.0, "ease": "linear"},
           {"t": L_OUT - 350, "v": 1.0, "ease": "step"}, {"t": L_OUT - 20, "v": 0.0, "ease": "ease-in"}]
    track("card-frame", 13, [{"id": "card-frame", "type": "rect", "start": C_IN, "end": L_OUT,
        "x": CARD_CX, "y": cy, "origin": "center", "width": CARD_W + 28, "height": CARD_H + 28,
        "fill": "#101418", "radius": 18, "scale": cs, "opacity": cop,
        "effects": [{"name": "shadow", "dx": 0, "dy": 10, "radius": 18, "color": "#101418", "opacity": 0.3}]}])
    track("card-video", 14, [{"id": "card-video", "type": "image", "start": C_IN, "end": L_OUT,
        "source": "stills/session-02.png", "x": CARD_CX, "y": cy, "origin": "center",
        "width": CARD_W, "height": CARD_H, "fit": "literal", "scale": cs, "opacity": cop}])
    # playback bar along the bottom of the rendered video
    bar_x = CARD_CX - CARD_W // 2 + 18
    bar_y = CARD_CY + CARD_H // 2 - 18
    track("card-bar-bg", 15, [{"id": "card-bar-bg", "type": "rect", "start": 5150, "end": L_OUT,
        "x": bar_x, "y": bar_y, "origin": "center-left", "width": CARD_W - 36, "height": 8,
        "fill": "#F5F0E655", "radius": 4,
        "opacity": [{"t": 5150, "v": 0.0}, {"t": 5300, "v": 1.0, "ease": "linear"},
                    {"t": L_OUT - 350, "v": 1.0, "ease": "step"}, {"t": L_OUT - 20, "v": 0.0, "ease": "ease-in"}]}])
    track("card-bar", 16, [{"id": "card-bar", "type": "rect", "start": 5150, "end": L_OUT,
        "x": bar_x, "y": bar_y, "origin": "center-left", "width": CARD_W - 36, "height": 8,
        "fill": "#FF5A36", "radius": 4,
        "scale": [{"t": 5150, "v": [0.02, 1.0]}, {"t": 6600, "v": [1.0, 1.0], "ease": "linear"}],
        "opacity": [{"t": 5150, "v": 0.0}, {"t": 5300, "v": 1.0, "ease": "linear"},
                    {"t": L_OUT - 350, "v": 1.0, "ease": "step"}, {"t": L_OUT - 20, "v": 0.0, "ease": "ease-in"}]}])
    # arrow from the laptop up to the video: shaft and chevron
    A_IN = 5000
    ay_top = CARD_CY + CARD_H // 2 + 14 + 12
    ay_bot = LAP_T - 10
    aop = [{"t": A_IN, "v": 0.0}, {"t": A_IN + 200, "v": 1.0, "ease": "ease-out"},
           {"t": L_OUT - 350, "v": 1.0, "ease": "step"}, {"t": L_OUT - 20, "v": 0.0, "ease": "ease-in"}]
    track("arrow-shaft", 17, [{"id": "arrow-shaft", "type": "rect", "start": A_IN, "end": L_OUT,
        "x": CARD_CX, "y": ay_bot, "origin": "bottom-center", "width": 12, "height": ay_bot - ay_top,
        "fill": "#FF5A36", "radius": 6,
        "scale": [{"t": A_IN, "v": [1.0, 0.1]}, {"t": A_IN + 250, "v": [1.0, 1.0], "ease": "ease-out"}],
        "opacity": aop}])
    for i, (side, ang) in enumerate((("l", -45), ("r", 45))):
        ox = -1 if side == "l" else 1
        track("arrow-head-" + side, 18 + i, [{"id": "arrow-head-" + side, "type": "rect",
            "start": A_IN + 150, "end": L_OUT,
            "x": CARD_CX + ox * 11, "y": ay_top + 11, "origin": "center",
            "width": 40, "height": 12, "fill": "#FF5A36", "radius": 6, "rotation": ang,
            "opacity": [{"t": A_IN + 150, "v": 0.0}, {"t": A_IN + 300, "v": 1.0, "ease": "ease-out"},
                        {"t": L_OUT - 350, "v": 1.0, "ease": "step"}, {"t": L_OUT - 20, "v": 0.0, "ease": "ease-in"}]}])

    # the question: a thought bubble with a question mark
    Q_IN, Q_OUT = 1150, 2950
    pop = [{"t": Q_IN, "v": [0.0, 0.0]}, {"t": Q_IN + 300, "v": [1.0, 1.0], "ease": [0.3, 1.5, 0.6, 1.0]}]
    qop = [{"t": Q_IN, "v": 1.0}, {"t": Q_OUT - 250, "v": 1.0, "ease": "step"}, {"t": Q_OUT - 20, "v": 0.0, "ease": "ease-in"}]
    BX, BY = 900, 240
    track("bubble-dots", 30, [
        {"id": "bubble-dot-1", "type": "ellipse", "start": Q_IN - 100, "end": Q_OUT,
         "x": 792, "y": 360, "origin": "center", "width": 26, "height": 26,
         "fill": "#F5F0E6", "stroke": "#3A2E2C", "stroke_width": 5,
         "opacity": [{"t": Q_IN - 100, "v": 0.0}, {"t": Q_IN, "v": 1.0, "ease": "linear"},
                     {"t": Q_OUT - 250, "v": 1.0, "ease": "step"}, {"t": Q_OUT - 20, "v": 0.0, "ease": "ease-in"}]}])
    track("bubble-dots-2", 31, [
        {"id": "bubble-dot-2", "type": "ellipse", "start": Q_IN - 50, "end": Q_OUT,
         "x": 826, "y": 322, "origin": "center", "width": 40, "height": 40,
         "fill": "#F5F0E6", "stroke": "#3A2E2C", "stroke_width": 5,
         "opacity": [{"t": Q_IN - 50, "v": 0.0}, {"t": Q_IN + 50, "v": 1.0, "ease": "linear"},
                     {"t": Q_OUT - 250, "v": 1.0, "ease": "step"}, {"t": Q_OUT - 20, "v": 0.0, "ease": "ease-in"}]}])
    track("bubble", 32, [{"id": "bubble", "type": "ellipse", "start": Q_IN, "end": Q_OUT,
        "x": BX, "y": BY, "origin": "center", "width": 170, "height": 160,
        "fill": "#F5F0E6", "stroke": "#3A2E2C", "stroke_width": 6, "scale": pop, "opacity": qop}])
    track("bubble-q", 33, [{"id": "bubble-q", "type": "text", "start": Q_IN, "end": Q_OUT,
        "x": BX, "y": BY + 4, "origin": "center", "width": 120, "height": 130,
        "font": "bold", "size": 108, "line_height": 1.0, "color": "#FF5A36", "align": "center",
        "runs": [{"text": "?"}],
        "scale": pop, "opacity": qop,
        "rotation": [{"t": Q_IN, "v": -14.0}, {"t": Q_IN + 500, "v": 8.0, "ease": "ease-in-out"},
                     {"t": Q_IN + 1100, "v": -6.0, "ease": "ease-in-out"}, {"t": Q_OUT - 50, "v": 4.0, "ease": "ease-in-out"}],
        "caption": False}])

    # the brand
    B_IN = 6950
    LW, LH = 2694, 623
    track("brand", 40, [{"id": "brand-lockup", "type": "image", "start": B_IN, "end": DUR,
        "source": "brand/lockup.png", "x": 1215, "y": 470, "origin": "center",
        "width": round(LW * 0.25), "height": round(LH * 0.25), "fit": "literal",
        "scale": [{"t": B_IN, "v": [0.85, 0.85]}, {"t": B_IN + 450, "v": [1.0, 1.0], "ease": [0.3, 1.4, 0.6, 1.0]}],
        "opacity": [{"t": B_IN, "v": 0.0}, {"t": B_IN + 300, "v": 1.0, "ease": "ease-out"}]}])

    # the owl, back to front in the rig's draw order
    joint_of = {"torso": "torso", "forearm_left": "forearm_left", "forearm_right": "forearm_right",
                "upper_arm_left": "upper_arm_left", "upper_arm_right": "upper_arm_right", "head": "head"}
    layer = 50
    for name in rig["draw_order"]:
        if name in joint_of:
            el = part_el("owl-" + name.replace("_", "-"), name, joint_of[name])
        elif name.startswith("mouth_"):
            shape = name[len("mouth_"):]
            op = step_kf([(t, 1.0 if mouth_at(t) == shape else 0.0) for t in FRAMES])
            el = part_el("owl-" + name.replace("_", "-"), name, "head", op)
        else:  # eyes_closed
            op = step_kf([(t, 1.0 if blink_at(t) else 0.0) for t in FRAMES])
            el = part_el("owl-eyes-closed", name, "head", op)
        track("owl-" + name.replace("_", "-"), layer, [el])
        layer += 1

    # sound
    vlen = 5520
    track("voice", 100, [{"id": "voice", "type": "audio", "start": V0, "end": V0 + vlen,
        "source": "character/voice/line-2.wav", "source_start": 0, "source_end": vlen, "volume": 1.4}])
    track("music", 101, [{"id": "music", "type": "audio", "start": 0, "end": 9800,
        "source": "music/bed-120bpm.wav", "source_start": 0, "source_end": 9800,
        "volume": [{"t": 0, "v": 0.0}, {"t": 400, "v": 0.3, "ease": "ease-out"},
                   {"t": 850, "v": 0.3, "ease": "step"}, {"t": 1000, "v": 0.1, "ease": "ease-in-out"},
                   {"t": 6550, "v": 0.1, "ease": "step"}, {"t": 7000, "v": 0.3, "ease": "ease-in-out"},
                   {"t": 8800, "v": 0.3, "ease": "step"}, {"t": 9750, "v": 0.0, "ease": "ease-in"}]}])
    return T

def main():
    doc = json.load(open(PROJ))
    doc["fonts"] = {"bold": [{"file": "vendor/Inter-Bold.ttf"}]}
    doc["tracks"] = tracks()
    head = {k: doc[k] for k in ("frame", "fps", "background", "duration", "output", "fonts", "fontVendor")}
    head["tracks"] = doc["tracks"]
    with open(PROJ, "w") as f:
        json.dump(head, f, ensure_ascii=False)

if __name__ == "__main__":
    main()
