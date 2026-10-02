#!/usr/bin/env python3
"""Generate hoot.montagent.json: Hoot explains editing a word (brief H6)."""
import json, math, subprocess

ROOT = "/private/var/folders/bk/5qm8ch691nz6vjb_4h98_4vw0000gn/T/montagent-eval-hfgyv3gl/work"
PROJ = f"{ROOT}/hoot.montagent.json"
FPS, DUR = 30, 10000
NF = DUR * FPS // 1000
rig = json.load(open(f"{ROOT}/character/rig.json"))
voice = json.load(open(f"{ROOT}/character/voice/line-2.json"))

INK, PAPER, SIGNAL, GREY = "#101418", "#F5F0E6", "#FF5A36", "#9AA3AD"
VO = 1100          # voice starts here on the project clock
VOICE_LEN = 5520   # file length in ms
S = 0.6            # owl scale
HX, HY = 400, 958  # owl feet pivot at home

def tf(f):
    return f * 1000 // FPS

def smooth(u):
    u = min(1.0, max(0.0, u))
    return u * u * (3 - 2 * u)

def ease_out(u):
    u = min(1.0, max(0.0, u))
    return 1 - (1 - u) ** 3

def lerp(a, b, u):
    return a + (b - a) * u

# ---------------------------------------------------------------- owl poses
CH = ["tr", "hr", "ual", "fal", "uar", "far"]
def P(tr=0, hr=0, ual=0, fal=0, uar=0, far=0):
    return dict(tr=tr, hr=hr, ual=ual, fal=fal, uar=uar, far=far)

HOP = P(5, -3, 28, 18, -28, -18)
SETTLE = P(0, 0, 6, 6, -6, -6)
ASK1 = P(-3, -10, 34, 52, -34, -52)       # shrug, open palms up, head cocked
ASK2 = P(-4, -14, 38, 58, -38, -58)       # "...video?" deeper tilt
MID = P(1, 3, 0, 8, -10, -8)
PT_LAPTOP = P(3, 10, -16, 8, -20, -2)       # arm out towards the laptop
PT_LAPTOP2 = P(3, 8, -14, 10, -23, -4)
PT_VIDEO = P(2, 13, -16, 8, -46, -6)       # arm up towards the rendered video
PT_VIDEO2 = P(2, 11, -14, 10, -49, -8)
REST2 = P(0, 3, -4, 8, -14, -8)
TADA = P(-2, -5, 24, 26, -30, -22)        # presenting the brand
TADA2 = P(-1, -2, 20, 22, -27, -18)

KEYS = [
    (0, HOP), (900, HOP), (1150, SETTLE),
    (1500, ASK1), (2200, ASK1), (2450, ASK2), (2850, ASK2),
    (3150, MID), (3480, PT_LAPTOP), (4150, PT_LAPTOP2),
    (4650, PT_VIDEO), (5600, PT_VIDEO2), (6300, PT_VIDEO),
    (6750, REST2), (7300, TADA), (8600, TADA2), (10000, TADA),
]

def pose_at(t):
    for (t0, p0), (t1, p1) in zip(KEYS, KEYS[1:]):
        if t0 <= t <= t1:
            u = smooth((t - t0) / (t1 - t0))
            return {c: lerp(p0[c], p1[c], u) for c in CH}
    return dict(KEYS[-1][1])

STRESS = [1450, 1925, 2275, 3125, 3825, 4687, 5300, 5787]  # stressed word onsets

def owl_at(t):
    p = pose_at(t)
    # idle life: sway, breathing, head drift
    p["tr"] += 1.3 * math.sin(2 * math.pi * t / 2600)
    p["hr"] += 1.6 * math.sin(2 * math.pi * t / 1900 + 1.0)
    br = math.sin(2 * math.pi * t / 2100)
    p["ual"] += 2.5 * br
    p["uar"] -= 2.5 * br
    p["fal"] += 2.0 * max(0, math.sin(2 * math.pi * t / 1700))
    p["far"] -= 2.0 * max(0, math.sin(2 * math.pi * t / 1700 + 0.7))
    # head nods on stressed words
    for s in STRESS:
        d = (t - s - 60) / 110.0
        p["hr"] += 3.0 * math.exp(-d * d) * (1 if s > 3000 else -1)
    # hop in from the left
    tx, ty = 0.0, 0.0
    if t < 1000:
        u = t / 1000
        tx = -760 * (1 - smooth(u) * 0.35 - 0.65 * u)
        v = (t % 333.333) / 333.333
        ty = -70 * 4 * v * (1 - v)
        p["ual"] += 14 * math.sin(2 * math.pi * v)
        p["uar"] -= 14 * math.sin(2 * math.pi * v)
    # elbows only ever flex the right way
    p["fal"] = max(0.0, p["fal"])
    p["far"] = min(0.0, p["far"])
    return p, tx, ty

PARTS = rig["parts"]
def rot(dx, dy, deg):
    a = math.radians(deg)
    return dx * math.cos(a) - dy * math.sin(a), dx * math.sin(a) + dy * math.cos(a)

def fk(t):
    p, tx, ty = owl_at(t)
    local = {"torso": p["tr"], "head": p["hr"], "upper_arm_left": p["ual"],
             "forearm_left": p["fal"], "upper_arm_right": p["uar"], "forearm_right": p["far"]}
    world = {"torso": ((HX + tx, HY + ty), p["tr"])}
    for name in ["head", "upper_arm_left", "upper_arm_right", "forearm_left", "forearm_right"]:
        par = PARTS[name]["parent"]
        (px, py), pa = world[par]
        dx = PARTS[name]["pivot"][0] - PARTS[par]["pivot"][0]
        dy = PARTS[name]["pivot"][1] - PARTS[par]["pivot"][1]
        rx, ry = rot(dx * S, dy * S, pa)
        world[name] = ((px + rx, py + ry), pa + local[name])
    return world

FRAMES = [fk(tf(f)) for f in range(NF)]

def kf(values, ease="linear", fmt=lambda v: v):
    """Keyframe list from per-frame values, dropping records a linear ramp already implies."""
    out = []
    for f, v in enumerate(values):
        out.append([tf(f), v])
    keep = [out[0]]
    for i in range(1, len(out) - 1):
        a, b, c = keep[-1], out[i], out[i + 1]
        u = (b[0] - a[0]) / (c[0] - a[0])
        if isinstance(b[1], list):
            pred = [lerp(a[1][k], c[1][k], u) for k in range(2)]
            if all(abs(pred[k] - b[1][k]) < 1e-3 for k in range(2)):
                continue
        else:
            pred = lerp(a[1], c[1], u)
            if abs(pred - b[1]) < (0.51 if isinstance(b[1], int) else 0.02):
                continue
        keep.append(b)
    keep.append(out[-1])
    recs = []
    for i, (t, v) in enumerate(keep):
        r = {"t": t, "v": fmt(v)}
        if i:
            r["ease"] = ease
        recs.append(r)
    return recs

def step_kf(values):
    recs = [{"t": 0, "v": values[0]}]
    for f in range(1, len(values)):
        if values[f] != values[f - 1]:
            recs.append({"t": tf(f), "v": values[f], "ease": "step"})
    return recs

def part_el(name, source_part, layer_id, opacity=None):
    pd = PARTS[source_part]
    xs = [round(fr[name][0][0]) for fr in FRAMES]
    ys = [round(fr[name][0][1]) for fr in FRAMES]
    rs = [round(fr[name][1], 2) for fr in FRAMES]
    el = {"id": f"owl-{source_part}", "type": "image", "group": "owl", "start": 0, "end": DUR,
          "source": f"character/{pd['file']}", "x": kf(xs), "y": kf(ys), "origin": "center",
          "width": pd["width"], "height": pd["height"], "fit": "literal", "scale": [S, S],
          "rotation": kf(rs)}
    if opacity is not None:
        el["opacity"] = opacity
    return el

# ---------------------------------------------------------------- mouths & blinks
vis = voice["visemes"]
LEAD = 30
def mouth_at(t):
    a = t - VO + LEAD
    cur = None
    for ms, vid in vis:
        if ms <= a:
            cur = rig["visemes"][str(vid)]
        else:
            break
    if a < vis[0][0] or a > voice["duration_ms"]:
        return None
    return cur

MOUTHS = [mouth_at(tf(f)) for f in range(NF)]
BLINKS = [1350, 2930, 5000, 7650, 9250]
def blinking(t):
    return any(b <= t < b + 100 for b in BLINKS)
EYES = [blinking(tf(f)) for f in range(NF)]

# ---------------------------------------------------------------- tracks
tracks = []
def track(name, layer, *els):
    tracks.append({"name": name, "layer": layer, "elements": list(els)})

def img(id_, src, x, y, w, h, start=0, end=DUR, origin="center", **kw):
    el = {"id": id_, "type": "image", "start": start, "end": end, "source": src, "x": x, "y": y,
          "origin": origin, "width": w, "height": h, "fit": "literal"}
    el.update(kw)
    return el

def rect(id_, x, y, w, h, fill=None, start=0, end=DUR, origin="top-left", **kw):
    el = {"id": id_, "type": "rect", "start": start, "end": end, "x": x, "y": y, "origin": origin,
          "width": w, "height": h}
    if fill:
        el["fill"] = fill
    for k in ["stroke", "stroke_width", "radius"]:
        if k in kw:
            el[k] = kw.pop(k)
    el.update(kw)
    return el

def text(id_, x, y, runs, size, font="regular", color=PAPER, start=0, end=DUR, origin="center-left",
         width=None, **kw):
    el = {"id": id_, "type": "text", "start": start, "end": end, "x": x, "y": y, "origin": origin,
          "width": width or 600, "height": math.ceil(size * 1.2) + 2, "font": font, "size": size,
          "color": color, "runs": runs}
    el.update(kw)
    el["caption"] = False
    return el

def fade(t0, t1, v0=0.0, v1=1.0, ease="ease-out"):
    return [{"t": t0, "v": v0}, {"t": t1, "v": v1, "ease": ease}]

def fade_io(i0, i1, o0, o1):
    return [{"t": i0, "v": 0.0}, {"t": i1, "v": 1.0, "ease": "ease-out"},
            {"t": o0, "v": 1.0, "ease": "linear"}, {"t": o1, "v": 0.0, "ease": "ease-in"}]

OUT0, OUT1 = 6700, 7100  # the laptop scene clears for the brand

# set
track("set", 0, img("study", "character/study.png", 0, 0, 1920, 1080, origin="top-left"))

# desk, drawn flat in the bookcases' wood
DX0, DX1, DTOP = 745, 1415, 788
desk_op = [{"t": OUT0, "v": 1.0}, {"t": OUT1, "v": 0.0, "ease": "ease-in"}]
track("desk-shadow", 5, {"id": "desk-shadow", "type": "ellipse", "start": 0, "end": OUT1 + 1,
      "x": (DX0 + DX1) // 2, "y": 958, "origin": "center", "width": 760, "height": 36,
      "fill": "#7A4E2E55", "opacity": desk_op})
track("desk-leg-l", 6, rect("desk-leg-l", DX0 + 28, DTOP + 20, 34, 150, "#9C6A42", end=OUT1 + 1,
      stroke="#5A3A24", stroke_width=4, opacity=desk_op))
track("desk-leg-r", 7, rect("desk-leg-r", DX1 - 62, DTOP + 20, 34, 150, "#9C6A42", end=OUT1 + 1,
      stroke="#5A3A24", stroke_width=4, opacity=desk_op))
track("desk-apron", 8, rect("desk-apron", DX0 + 14, DTOP + 18, DX1 - DX0 - 28, 36, "#B47A4C", end=OUT1 + 1,
      stroke="#5A3A24", stroke_width=4, opacity=desk_op))
track("desk-top", 9, rect("desk-top", DX0, DTOP, DX1 - DX0, 26, "#C88E5C", end=OUT1 + 1,
      stroke="#5A3A24", stroke_width=4, radius=6, opacity=desk_op))

# laptop: 708x566 at 0.8, bottom-centre on the desk top
LS = 0.8
LX, LB = 1080, DTOP + 4
lap_left = LX - 708 * LS / 2
lap_top = LB - 566 * LS
SX0, SY0 = round(lap_left + 120 * LS), round(lap_top + 41 * LS)
SX1, SY1 = round(lap_left + 587 * LS), round(lap_top + 312 * LS)
SW, SH = SX1 - SX0, SY1 - SY0
track("laptop", 12, img("laptop", "character/laptop.png", LX, LB, 708, 566, end=OUT1 + 1,
      origin="bottom-center", scale=[LS, LS], opacity=desk_op))

# editor window on the laptop screen: the project file
ED0 = 3080
ed_op = fade_io(ED0, ED0 + 220, OUT0, OUT1)
EX = SX0 + 16
track("ed-bg", 13, rect("ed-bg", SX0 + 2, SY0 + 2, SW - 4, SH - 4, INK, start=ED0, end=OUT1 + 1,
      opacity=ed_op))
track("ed-tab", 14, rect("ed-tab", SX0 + 2, SY0 + 2, SW - 4, 30, "#252D35", start=ED0, end=OUT1 + 1,
      opacity=ed_op))
track("ed-name", 15, text("ed-name", EX, SY0 + 17, [{"text": "hello.montagent.json"}], 15, "bold", GREY,
      start=ED0, end=OUT1 + 1, width=330, opacity=ed_op))
CODE_SIZE = 28
track("ed-l1", 16, text("ed-l1", EX, SY0 + 54, [{"text": "{\"type\": \"text\","}], 18, color=GREY,
      start=ED0, end=OUT1 + 1, width=330, opacity=ed_op))
track("ed-l2", 17, text("ed-l2", EX, SY0 + 80, [{"text": "\"text\":"}], 18, color=GREY,
      start=ED0, end=OUT1 + 1, width=330, opacity=ed_op))
track("ed-l4", 18, text("ed-l4", EX, SY0 + 150, [{"text": "}"}], 18, color=GREY,
      start=ED0, end=OUT1 + 1, width=330, opacity=ed_op))

L3Y = SY0 + 115
k = CODE_SIZE / 24
PREFIX_W = 136.5 * k
DRAFT_W = (200.80078125 - 136.5) * k
# selection behind "draft" while Hoot says "edit"
SEL0, SEL1 = 3470, 3800
track("ed-sel", 19, rect("ed-sel", round(EX + PREFIX_W - 2), L3Y - 18, round(DRAFT_W - 9 * k), 36,
      "#FF5A3666", start=SEL0, end=SEL1, radius=4))
# the line, word by word
word = "document"
TY0, TSTEP = 3850, 40
line_els = [
    text("ed-l3-draft", EX, L3Y, [{"text": "\"Video as a draft\""}], CODE_SIZE, start=ED0, end=SEL1,
         width=330, opacity=fade(ED0, ED0 + 220)),
    text("ed-l3-empty", EX, L3Y, [{"text": "\"Video as a \""}], CODE_SIZE, start=SEL1, end=TY0, width=330),
]
for i in range(1, len(word)):
    line_els.append(text(f"ed-l3-type-{i}", EX, L3Y,
                         [{"text": "\"Video as a "}, {"text": word[:i], "color": SIGNAL}, {"text": "\""}],
                         CODE_SIZE, start=TY0 + (i - 1) * TSTEP, end=TY0 + i * TSTEP, width=330))
TY1 = TY0 + (len(word) - 1) * TSTEP
line_els.append(text("ed-l3-done", EX, L3Y,
                     [{"text": "\"Video as a "}, {"text": word, "color": SIGNAL}, {"text": "\""}],
                     CODE_SIZE, start=TY1, end=OUT1 + 1, width=330,
                     opacity=[{"t": OUT0, "v": 1.0}, {"t": OUT1, "v": 0.0, "ease": "ease-in"}]))
track("ed-l3", 20, *line_els)

# render progress inside the editor
R0, R1 = 4500, 5300
BAR_Y = SY1 - 26
track("ed-render-label", 21, text("ed-render-label", EX, BAR_Y - 14, [{"text": "rendering…"}], 14, "bold",
      GREY, start=R0, end=R1, width=200),
      text("ed-render-done", EX, BAR_Y - 14, [{"text": "rendered  hello.mp4"}], 14, "bold", SIGNAL,
      start=R1, end=OUT1 + 1, width=250,
      opacity=[{"t": OUT0, "v": 1.0}, {"t": OUT1, "v": 0.0, "ease": "ease-in"}]))
track("ed-bar-bg", 22, rect("ed-bar-bg", EX, BAR_Y, SW - 32, 6, "#2E3740", start=R0, end=OUT1 + 1,
      radius=3, opacity=fade_io(R0, R0 + 120, OUT0, OUT1)))
track("ed-bar", 23, rect("ed-bar", EX, BAR_Y, SW - 32, 6, SIGNAL, start=R0, end=OUT1 + 1, radius=3,
      scale=[{"t": R0 + 50, "v": [0.0, 1.0]}, {"t": R1 - 50, "v": [1.0, 1.0], "ease": "ease-in-out"}],
      opacity=[{"t": OUT0, "v": 1.0}, {"t": OUT1, "v": 0.0, "ease": "ease-in"}]))

# the rendered video flies out of the laptop screen
CX, CY = 1662, 268
F0, F1 = 5300, 5760
SCX, SCY = (SX0 + SX1) / 2, (SY0 + SY1) / 2
S0 = 0.22
def card_xy(ox, oy, origin_scale=1.0):
    """Keyframes for a card part whose origin sits (ox, oy) from the card centre."""
    xs, ys, ss = [], [], []
    n = 14
    for i in range(n + 1):
        t = round(F0 + (F1 - F0) * i / n)
        u = ease_out(i / n)
        cx, cy = lerp(SCX, CX, u), lerp(SCY, CY, u) - 60 * math.sin(math.pi * u)
        s = lerp(S0, 1.0, u)
        xs.append((t, round(cx + ox * s)))
        ys.append((t, round(cy + oy * s)))
        ss.append((t, round(s, 4)))
    mk = lambda seq: [dict({"t": t, "v": v}, **({"ease": "linear"} if j else {})) for j, (t, v) in enumerate(seq)]
    return mk(xs), mk(ys), [dict({"t": t, "v": [v * origin_scale if False else v, v]}, **({"ease": "linear"} if j else {}))
                            for j, (t, v) in enumerate(ss)]

card_op = fade_io(F0, F0 + 120, OUT0, OUT1)
CARD_END = OUT1 + 1
def card_part(id_, ox, oy, layer, make):
    x, y, sc = card_xy(ox, oy)
    el = make(x, y, sc)
    track(id_, layer, el)

card_part("card-frame", 0, 0, 30, lambda x, y, sc: rect("card-frame", x, y, 476, 304, PAPER, start=F0,
          end=CARD_END, origin="center", stroke=INK, stroke_width=5, radius=16, scale=sc, opacity=card_op,
          effects=[{"name": "shadow", "dx": 0, "dy": 10, "radius": 18, "color": "#3A2A1C", "opacity": 0.35}]))
card_part("card-video", 0, -12, 31, lambda x, y, sc: img("card-video", "stills/session-02.png", x, y, 448, 252,
          start=F0, end=CARD_END, scale=sc, opacity=card_op))
card_part("card-bar-bg", -210, 133, 32, lambda x, y, sc: rect("card-bar-bg", x, y, 420, 8, "#D9D2C4",
          start=F0, end=CARD_END, origin="center-left", radius=4, scale=sc, opacity=card_op))
card_part("card-tab", -238, -152, 29, lambda x, y, sc: rect("card-tab", x, y, 150, 40, INK, start=F0,
          end=CARD_END, origin="bottom-left", radius=10, scale=sc, opacity=card_op))
card_part("card-tab-label", -222, -170, 33, lambda x, y, sc: text("card-tab-label", x, y,
          [{"text": "hello.mp4"}], 18, "bold", PAPER, start=F0, end=CARD_END, width=140, scale=sc,
          opacity=card_op))
# playhead runs once the card has landed: the video plays
P0, P1 = F1 + 60, OUT0
track("card-bar", 34, rect("card-bar", CX - 210, CY + 133, 420, 8, SIGNAL, start=F1, end=CARD_END,
      origin="center-left", radius=4,
      scale=[{"t": P0, "v": [0.02, 1.0]}, {"t": P1, "v": [1.0, 1.0], "ease": "linear"}],
      opacity=[{"t": F1, "v": 0.0}, {"t": F1 + 80, "v": 1.0, "ease": "linear"},
               {"t": OUT0, "v": 1.0, "ease": "linear"}, {"t": OUT1, "v": 0.0, "ease": "ease-in"}]))

# arrow from the laptop to the video: file -> video
A0x, A0y = 1312, 372
A1x, A1y = 1412, 312
ang = math.degrees(math.atan2(A1y - A0y, A1x - A0x))
alen = round(math.hypot(A1x - A0x, A1y - A0y))
AR0, AR1 = 5420, 5700
arrow_op = [{"t": OUT0, "v": 1.0}, {"t": OUT1, "v": 0.0, "ease": "ease-in"}]
track("arrow-shaft", 26, rect("arrow-shaft", A0x, A0y, alen, 10, SIGNAL, start=AR0, end=CARD_END,
      origin="center-left", radius=5, rotation=round(ang, 2),
      scale=[{"t": AR0, "v": [0.05, 1.0]}, {"t": AR1, "v": [1.0, 1.0], "ease": "ease-out"}],
      opacity=arrow_op))
for side, d in (("a", 35), ("b", -35)):
    track(f"arrow-head-{side}", 27 if side == "a" else 28,
          rect(f"arrow-head-{side}", A1x, A1y, 34, 10, SIGNAL, start=AR1 - 60, end=CARD_END,
               origin="center-left", radius=5, rotation=round(ang + 180 + d, 2),
               scale=[{"t": AR1 - 60, "v": [0.1, 1.0]}, {"t": AR1 + 60, "v": [1.0, 1.0], "ease": "ease-out"}],
               opacity=arrow_op))

# brand
B0, B1 = 7000, 7450
LW = 840
lh = round(623 * LW / 2694)
BX, BY = 1160, 440
track("brand", 36, img("brand-lockup", "brand/lockup.png", BX, BY, 2694, 623, start=B0, end=DUR,
      scale=[{"t": B0, "v": [LW / 2694 * 0.9, LW / 2694 * 0.9]},
             {"t": B1, "v": [LW / 2694, LW / 2694], "ease": "ease-out"}],
      opacity=fade(B0, B1 - 100)))
track("tagline", 37, text("tagline", BX, BY + lh // 2 + 70,
      [{"text": "Edit the file. Render it "}, {"text": "again.", "color": SIGNAL}], 42, "regular", INK,
      start=B0 + 250, end=DUR, origin="center", width=900, opacity=fade(B0 + 250, B0 + 700)))

# owl, in draw order
for i, name in enumerate(rig["draw_order"]):
    layer = 50 + i
    if name in PARTS and not name.startswith(("mouth_", "eyes_")):
        el = part_el(name, name, layer)
    elif name.startswith("mouth_"):
        shape = name[len("mouth_"):]
        el = part_el("head", name, layer, opacity=step_kf([1.0 if m == shape else 0.0 for m in MOUTHS]))
    else:
        el = part_el("head", name, layer, opacity=step_kf([1.0 if e else 0.0 for e in EYES]))
    track(f"owl-{name}", layer, el)

# audio
track("voice", 100, {"id": "voice", "type": "audio", "start": VO, "end": VO + VOICE_LEN,
      "source": "character/voice/line-2.wav", "source_start": 0, "source_end": VOICE_LEN})
track("music", 101, {"id": "music", "type": "audio", "start": 0, "end": 9600,
      "source": "music/bed-120bpm.wav", "source_start": 0, "source_end": 9600,
      "volume": [{"t": 0, "v": 0.0}, {"t": 300, "v": 0.32, "ease": "ease-out"},
                 {"t": 900, "v": 0.32, "ease": "linear"}, {"t": 1100, "v": 0.13, "ease": "ease-in-out"},
                 {"t": 6650, "v": 0.13, "ease": "linear"}, {"t": 7100, "v": 0.3, "ease": "ease-in-out"},
                 {"t": 8600, "v": 0.3, "ease": "linear"}, {"t": 9550, "v": 0.0, "ease": "ease-in"}]})

proj = json.load(open(PROJ))
proj["tracks"] = tracks
json.dump(proj, open(PROJ, "w"), indent=2, ensure_ascii=False)
subprocess.run(["montagent", "fmt", PROJ], check=True, capture_output=True)
print("screen", SX0, SY0, SX1, SY1, "laptop top", lap_top)
