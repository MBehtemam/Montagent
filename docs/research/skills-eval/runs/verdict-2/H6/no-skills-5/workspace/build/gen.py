"""Generate hoot.montagent.json: Hoot the owl explains editing a word (brief H6).

The rig is hierarchical but Montagent transforms are flat, so every rig part gets a
per-frame keyframe track computed here from the parent chain.
"""
import json, math, os

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PROJECT = os.path.join(ROOT, "hoot.montagent.json")
rig = json.load(open(os.path.join(ROOT, "character/rig.json")))
voice = json.load(open(os.path.join(ROOT, "character/voice/line-2.json")))

FPS, DUR = 30, 10000
FRAMES = [i * 1000 // FPS for i in range(DUR * FPS // 1000)]
S = 0.64                 # drawing px -> frame px
FEET_Y = 984             # torso pivot on screen (feet on the study floor)
OWL_X = 400
V0 = 900                 # voice starts here
INK, PAPER, SIGNAL = "#101418", "#F5F0E6", "#FF5A36"
OUTLINE = "#3B2E2A"

def W(t):  # word time on the timeline
    return t + V0

# ---------------------------------------------------------------- pose curves
CH = ["tx", "ty", "trot", "tsx", "tsy", "hrot", "hdx", "hdy", "ulL", "flL", "ulR", "flR"]
BASE = dict(tx=OWL_X, ty=0, trot=0, tsx=1, tsy=1, hrot=0, hdx=0, hdy=0,
            ulL=-25, flL=10, ulR=25, flR=-10)

def pose(**kw):
    p = dict(BASE); p.update(kw); return p

NEUTRAL = pose()
AIR = dict(ulL=40, flL=12, ulR=-40, flR=-12, tsx=0.96, tsy=1.05)
LAND = dict(ulL=-15, flL=8, ulR=15, flR=-8, tsx=1.06, tsy=0.92)
ASK = pose(ulL=8, flL=42, ulR=-8, flR=-42, hrot=-6, hdx=-2, trot=-2)
ASK2 = pose(ulL=14, flL=52, ulR=-14, flR=-52, hrot=-8, hdx=-3, hdy=-2, trot=-3, tsx=1.02, tsy=1.02)
PT_LAP = pose(ulL=-30, flL=12, ulR=-33, flR=-6, hrot=6, hdx=3, trot=2.5)
PT_CARD = pose(ulL=-30, flL=12, ulR=-62, flR=-10, hrot=7, hdx=3, hdy=-3, trot=3)
FRONT = pose(hrot=-3)
WAVE_A = pose(ulR=-45, flR=-22, hrot=-4, hdx=-2, trot=-1)
WAVE_B = pose(ulR=-48, flR=-52, hrot=-4, hdx=-2, trot=-1)
PRESENT = pose(ulR=-12, flR=-28, ulL=-25, flL=12, hrot=5, hdx=2, trot=1.5)

# hop in from the left: two hops landing at 420 and 840
X0, X1, X2 = -360, 50, OWL_X
KEYS = [
    (0, pose(tx=X0, **AIR)),
    (420, pose(tx=X1, **LAND)),
    (520, pose(tx=X1 + 20)),
    (840, pose(tx=X2, **LAND)),
    (980, NEUTRAL),
    # the question: "Want to change one word in a video?"  (W(50) .. W(1662))
    (W(150), NEUTRAL),
    (W(450), ASK),
    (W(1000), pose(**{**ASK, "hrot": -5, "hdx": -2})),
    (W(1300), ASK2),
    (W(1800), ASK2),
    # the pause, then "Just edit the file"  (W(2025) .. W(3212))
    (W(2050), NEUTRAL),
    (W(2350), PT_LAP),
    (W(3250), pose(**{**PT_LAP, "ulR": -30, "hrot": 5})),
    # "and Montagent renders it again"  (W(3362) .. W(5162))
    (W(3500), PT_LAP),
    (W(3850), PT_CARD),
    (W(5000), pose(**{**PT_CARD, "ulR": -58, "hrot": 6})),
    (W(5400), PT_CARD),
    # turn to camera, wave at the brand
    (6750, FRONT),
    (7100, WAVE_A),
    (7400, WAVE_B),
    (7700, WAVE_A),
    (8000, WAVE_B),
    (8300, WAVE_A),
    (8750, PRESENT),
    (10000, pose(**{**PRESENT, "ulR": -15, "hrot": 6})),
]
# The hop's vertical arc is applied analytically on top of the keyed poses.
HOPS = [(0, 420, 80), (420, 840, 70)]

def smooth(u):
    return u * u * (3 - 2 * u)

def pose_at(t):
    if t <= KEYS[0][0]:
        p = dict(KEYS[0][1])
    elif t >= KEYS[-1][0]:
        p = dict(KEYS[-1][1])
    else:
        for (ta, pa), (tb, pb) in zip(KEYS, KEYS[1:]):
            if ta <= t <= tb:
                u = smooth((t - ta) / (tb - ta))
                p = {c: pa[c] + (pb[c] - pa[c]) * u for c in CH}
                # horizontal travel during hops is linear, not eased
                if tb <= 840:
                    lu = (t - ta) / (tb - ta)
                    p["tx"] = pa["tx"] + (pb["tx"] - pa["tx"]) * lu
                break
    for a, b, h in HOPS:
        if a <= t < b:
            p["ty"] -= h * math.sin(math.pi * (t - a) / (b - a))
    # idle life: breathing, sway, a little arm drift, always on
    s = t / 1000
    p["tsy"] += 0.012 * math.sin(2 * math.pi * s / 1.9)
    p["tsx"] -= 0.006 * math.sin(2 * math.pi * s / 1.9)
    p["trot"] += 1.2 * math.sin(2 * math.pi * s / 2.7 + 0.6)
    p["hrot"] += 1.3 * math.sin(2 * math.pi * s / 2.3 + 1.1)
    p["ulL"] += 2.0 * math.sin(2 * math.pi * s / 2.1)
    p["ulR"] -= 2.0 * math.sin(2 * math.pi * s / 2.4 + 0.8)
    # keep elbows bending the natural way only
    p["flL"] = max(p["flL"], 4)
    p["flR"] = min(p["flR"], -4)
    return p

# ------------------------------------------------------------ rig solving
P = rig["parts"]

def rot(v, deg):
    a = math.radians(deg); c, s_ = math.cos(a), math.sin(a)
    return (v[0] * c - v[1] * s_, v[0] * s_ + v[1] * c)

def solve(p):
    """World (x, y, rotation, sx, sy) of every part's pivot."""
    out = {}
    sx, sy = p["tsx"], p["tsy"]
    tor = (p["tx"], FEET_Y + p["ty"], p["trot"])
    out["torso"] = (tor[0], tor[1], tor[2], sx, sy)
    def child(name, parent, own_rot, extra=(0, 0)):
        px, py, pr, psx, psy = out[parent]
        dx = (P[name]["pivot"][0] - P[parent]["pivot"][0] + extra[0]) * S * psx
        dy = (P[name]["pivot"][1] - P[parent]["pivot"][1] + extra[1]) * S * psy
        ox, oy = rot((dx, dy), pr)
        out[name] = (px + ox, py + oy, pr + own_rot, psx, psy)
    child("head", "torso", p["hrot"], (p["hdx"], p["hdy"]))
    child("upper_arm_left", "torso", p["ulL"])
    child("upper_arm_right", "torso", p["ulR"])
    child("forearm_left", "upper_arm_left", p["flL"])
    child("forearm_right", "upper_arm_right", p["flR"])
    return out

SOLVED = [solve(pose_at(t)) for t in FRAMES]

# ------------------------------------------------------------ emit helpers
def kf(values, ease="linear", dedupe=True):
    """values: list of (t, v). Drops records that repeat (keeps clamping semantics)."""
    recs = []
    for i, (t, v) in enumerate(values):
        if dedupe and recs and recs[-1]["v"] == v and i + 1 < len(values) and values[i + 1][1] == v:
            continue
        r = {"t": t, "v": v}
        if recs:
            r["ease"] = ease
        recs.append(r)
    if len(recs) == 1:
        return recs[0]["v"]
    return recs

def anim(pairs, ease="ease-in-out"):
    """Sparse keyframes: [(t, v), ...] with one ease for every arrival."""
    if len(pairs) == 1:
        return pairs[0][1]
    recs = [{"t": pairs[0][0], "v": pairs[0][1]}]
    for t, v, *e in pairs[1:]:
        recs.append({"t": t, "v": v, "ease": e[0] if e else ease})
    return recs

def img(id_, source, w, h, start=0, end=DUR, **kw):
    e = {"id": id_, "type": "image", "start": start, "end": end, "source": source}
    for k in ("x", "y", "origin"):
        if k in kw: e[k] = kw.pop(k)
    e.update({"width": w, "height": h, "fit": kw.pop("fit", "literal")})
    for k in ("clip", "scale", "rotation", "opacity", "effects"):
        if k in kw: e[k] = kw.pop(k)
    assert not kw, kw
    return e

def shape(kind, id_, w, h, start=0, end=DUR, **kw):
    e = {"id": id_, "type": kind, "start": start, "end": end}
    for k in ("x", "y", "origin"):
        if k in kw: e[k] = kw.pop(k)
    e.update({"width": w, "height": h})
    for k in ("fill", "stroke", "stroke_width", "radius", "scale", "rotation", "opacity", "effects"):
        if k in kw: e[k] = kw.pop(k)
    assert not kw, kw
    return e

def text(id_, runs, size, font, w, h, start=0, end=DUR, **kw):
    e = {"id": id_, "type": "text", "start": start, "end": end}
    for k in ("x", "y", "origin"):
        if k in kw: e[k] = kw.pop(k)
    e.update({"width": w, "height": h, "font": font, "size": size})
    for k in ("line_height", "color", "align"):
        if k in kw: e[k] = kw.pop(k)
    e["runs"] = runs
    for k in ("stroke", "stroke_width", "scale", "rotation", "opacity", "effects"):
        if k in kw: e[k] = kw.pop(k)
    e["caption"] = False
    assert not kw, kw
    return e

tracks = []
def track(name, layer, *elements):
    # every track its own layer, in the order written: later draws in front
    tracks.append({"name": name, "layer": len(tracks), "elements": list(elements)})

# ------------------------------------------------------------ the set
track("study", 0, img("study", "character/study.png", 1920, 1080, x=0, y=0, origin="top-left"))

# owl's contact shadow follows the feet and shrinks while airborne
sh_x = [(t, round(SOLVED[i]["torso"][0])) for i, t in enumerate(FRAMES)]
sh_s = []
for i, t in enumerate(FRAMES):
    lift = FEET_Y - SOLVED[i]["torso"][1]
    k = round(1 - lift / 200, 3)
    sh_s.append((t, [k, k]))
track("owl-shadow", 1, shape("ellipse", "owl-shadow", 300, 44, x=kf(sh_x), y=FEET_Y + 6, origin="center",
                             fill="#6B3F1E55", scale=kf(sh_s)))

# a small desk for the laptop, drawn in the set's flat style
DESK_L, DESK_R, DESK_TOP = 770, 1350, 834
PROPS_OUT = (6500, 6900)   # desk, laptop and video card leave for the brand
def fade_out():
    return anim([(PROPS_OUT[0], 1.0), (PROPS_OUT[1] - 50, 0.0, "ease-in")])
track("desk-shadow", 1, shape("ellipse", "desk-shadow", 640, 40, x=(DESK_L + DESK_R) // 2, y=1004, origin="center",
                              fill="#6B3F1E44", opacity=fade_out()))
track("desk-leg-l", 2, shape("rect", "desk-leg-l", 34, 170, x=DESK_L + 40, y=DESK_TOP + 20, origin="top-left",
                             fill="#A86B3D", stroke=OUTLINE, stroke_width=5, radius=6, opacity=fade_out()))
track("desk-leg-r", 2, shape("rect", "desk-leg-r", 34, 170, x=DESK_R - 74, y=DESK_TOP + 20, origin="top-left",
                             fill="#A86B3D", stroke=OUTLINE, stroke_width=5, radius=6, opacity=fade_out()))
track("desk-top", 3, shape("rect", "desk-top", DESK_R - DESK_L, 34, x=DESK_L, y=DESK_TOP - 4, origin="top-left",
                           fill="#C98A52", stroke=OUTLINE, stroke_width=5, radius=8, opacity=fade_out()))

# the laptop, and the project file open on it
LS = 0.8
LW, LH = round(708 * LS), round(566 * LS)
LX = (DESK_L + DESK_R) // 2 - LW // 2
LY = DESK_TOP - LH + 4
SX0, SY0 = LX + round(120 * LS), LY + round(41 * LS)
SX1, SY1 = LX + round(587 * LS), LY + round(312 * LS)
track("laptop", 4, img("laptop", "character/laptop.png", LW, LH, x=LX, y=LY, origin="top-left", opacity=fade_out()))
track("editor-bg", 5, shape("rect", "editor-bg", SX1 - SX0, SY1 - SY0, x=SX0, y=SY0, origin="top-left",
                            fill="#1A2027", opacity=fade_out()))
track("editor-tab", 6, shape("rect", "editor-tab", 210, 28, x=SX0 + 8, y=SY0 + 8, origin="top-left",
                             fill="#2A323C", radius=5, opacity=fade_out()))
PAD = 16
LINE0 = SY0 + 50
LS_Y = 36
def code(id_, s, row, start=0, end=DUR, **kw):
    return text(id_, s if isinstance(s, list) else [{"text": s}], 26, "reg", SX1 - SX0 - 2 * PAD, 32, start, end,
                x=SX0 + PAD, y=LINE0 + row * LS_Y, origin="top-left", color="#D8DEE6", **kw)
track("editor-name", 7, text("editor-name", [{"text": "hello.montagent.json"}], 18, "reg", 196, 22,
                             x=SX0 + 18, y=SY0 + 11, origin="top-left", color="#9AA3AD", opacity=fade_out()))
track("code-1", 7, code("code-1", '{"id": "hello",', 0, opacity=fade_out()))
track("code-2", 7, code("code-2", ' "type": "text",', 1, opacity=fade_out()))
track("code-4", 7, code("code-4", ' "size": 64}', 3, opacity=fade_out()))

# the edit: "world" is selected while Hoot says "edit", then replaced by "Montagent"
T_SELECT, T_SWAP = W(2387), W(2725)
PREFIX_W = 174.637 + 7   # measured advance of ' "text": "Hello, ' at 26px (plus the leading space)
track("code-3", 7,
      code("code-3a", [{"text": ' "text": "Hello, '}, {"text": "world"}, {"text": '",'}], 2, 0, T_SWAP),
      code("code-3b", [{"text": ' "text": "Hello, '}, {"text": "Montagent", "font": "bold", "color": SIGNAL},
                       {"text": '",'}], 2, T_SWAP, DUR, opacity=fade_out()))
track("selection", 6, shape("rect", "selection", 74, 34, T_SELECT, T_SWAP, x=round(SX0 + PAD + PREFIX_W) - 3,
                            y=LINE0 + 2 * LS_Y - 1, origin="top-left", fill="#FF5A3666", radius=4))
# the changed line is marked like a fresh edit in a diff
track("edit-mark", 5, shape("rect", "edit-mark", SX1 - SX0, 34, T_SWAP, DUR, x=SX0, y=LINE0 + 2 * LS_Y - 1,
                            origin="top-left", fill="#2E7D3240",
                            opacity=anim([(T_SWAP, 0.0), (T_SWAP + 150, 1.0, "ease-out"),
                                          (PROPS_OUT[0], 1.0, "linear"), (PROPS_OUT[1] - 50, 0.0, "ease-in")])))
CARET_X = round(SX0 + PAD + PREFIX_W + 138.7 + 9)
track("caret", 8, shape("rect", "caret", 3, 30, T_SWAP, W(3500), x=CARET_X, y=LINE0 + 2 * LS_Y,
                        origin="top-left", fill=PAPER,
                        opacity=anim([(T_SWAP, 1.0), (T_SWAP + 250, 1.0, "step"), (T_SWAP + 251, 0.0, "step"),
                                      (T_SWAP + 500, 1.0, "step")])))

# ------------------------------------------------------------ the rendered video
T_RENDER = W(3587)          # "Montagent"
T_LAND = T_RENDER + 550
CW, CH_ = 448, 252
CARD_END = (1660, 300)
CARD_START = ((SX0 + SX1) / 2, (SY0 + SY1) / 2)
def card_state(t):
    if t <= T_RENDER: u = 0.0
    elif t >= T_LAND: u = 1.0
    else:
        x = (t - T_RENDER) / (T_LAND - T_RENDER)
        u = 1 - (1 - x) ** 3
    # small overshoot settle in scale
    cx = CARD_START[0] + (CARD_END[0] - CARD_START[0]) * u
    cy = CARD_START[1] + (CARD_END[1] - CARD_START[1]) * u - 60 * math.sin(math.pi * u)
    sc = 0.25 + 0.75 * u
    if T_LAND <= t < T_LAND + 250:
        sc *= 1 + 0.04 * math.sin(math.pi * (t - T_LAND) / 250)
    return cx, cy, sc
CARD_FR = [t for t in FRAMES if T_RENDER <= t <= T_LAND + 280]
def card_el(kind, id_, w, h, off=(0, 0), start=T_RENDER, end=PROPS_OUT[1], **kw):
    xs, ys, ss = [], [], []
    for t in CARD_FR:
        cx, cy, sc = card_state(t)
        xs.append((t, round(cx + off[0] * sc))); ys.append((t, round(cy + off[1] * sc)))
        ss.append((t, [round(sc, 4), round(sc, 4)]))
    common = dict(x=kf(xs), y=kf(ys), origin="center", scale=kf(ss), opacity=fade_out(), **kw)
    if kind == "image":
        return img(id_, kw.pop("source") if "source" in kw else None, w, h, start, end, **common)
    return shape(kind, id_, w, h, start, end, **common)
track("card-frame", 10, card_el("rect", "card-frame", CW + 20, CH_ + 46, (0, 13), fill=INK, radius=14,
                                effects=[{"name": "shadow", "dx": 0, "dy": 10, "radius": 18, "color": "#3B2E2A",
                                          "opacity": 0.35}]))
cw = card_el("rect", "tmp", 1, 1)
track("card-image", 11, img("card-image", "stills/session-02.png", CW, CH_, T_RENDER, PROPS_OUT[1],
                            x=cw["x"], y=cw["y"], origin="center", scale=cw["scale"], opacity=fade_out()))
track("card-bar", 11, card_el("rect", "card-bar", CW - 24, 6, (0, CH_ / 2 + 18), fill="#3A434D", radius=3))
# the progress fill plays across once the video has landed
bar = card_el("rect", "card-play", CW - 24, 6, (0, CH_ / 2 + 18), fill=SIGNAL, radius=3)
bar_scale = [{"t": T_RENDER, "v": [0.0, 1.0]}, {"t": T_LAND, "v": [0.0, 1.0], "ease": "linear"},
             {"t": PROPS_OUT[0], "v": [1.0, 1.0], "ease": "linear"}]
# fill grows from its left end: origin center-left, x pinned to the bar's left edge once landed
lx = round(CARD_END[0] - (CW - 24) / 2)
ly = round(CARD_END[1] + CH_ / 2 + 18)
track("card-play", 12, shape("rect", "card-play", CW - 24, 6, T_LAND, PROPS_OUT[1], x=lx, y=ly, origin="center-left",
                             fill=SIGNAL, radius=3, scale=anim([(T_LAND, [0.0, 1.0]), (PROPS_OUT[0], [1.0, 1.0], "linear")]),
                             opacity=fade_out()))

# an arrow from the laptop screen to the video it rendered
A = (SX1 + 14, SY0 + 120)
B = (CARD_END[0] - CW / 2 - 22, CARD_END[1] + 60)
ang = math.degrees(math.atan2(B[1] - A[1], B[0] - A[0]))
length = math.hypot(B[0] - A[0], B[1] - A[1])
T_ARROW = T_RENDER + 200
arrow_op = anim([(T_ARROW, 0.0), (T_ARROW + 200, 1.0, "ease-out"), (PROPS_OUT[0], 1.0, "linear"),
                 (PROPS_OUT[1] - 50, 0.0, "ease-in")])
mid = ((A[0] + B[0]) / 2, (A[1] + B[1]) / 2)
track("arrow-shaft", 9, shape("rect", "arrow-shaft", round(length) - 6, 12, T_ARROW, PROPS_OUT[1], x=round(mid[0]),
                              y=round(mid[1]), origin="center", fill=SIGNAL, radius=6, rotation=round(ang, 2),
                              opacity=arrow_op))
for side, sgn in (("a", 1), ("b", -1)):
    ha = ang + 180 + sgn * 35
    hx = B[0] + 18 * math.cos(math.radians(ha)); hy = B[1] + 18 * math.sin(math.radians(ha))
    track(f"arrow-head-{side}", 9, shape("rect", f"arrow-head-{side}", 46, 12, T_ARROW, PROPS_OUT[1], x=round(hx),
                                         y=round(hy), origin="center", fill=SIGNAL, radius=6,
                                         rotation=round(ha, 2), opacity=arrow_op))

# ------------------------------------------------------------ Hoot
ORDER = rig["draw_order"]
BASE_LAYER = 20
def part_track(name, layer, opacity=None, src_name=None):
    part = P[src_name or name]
    body = src_name or name
    if name.startswith("mouth_") or name == "eyes_closed":
        body = "head"
    xs = [(t, round(SOLVED[i][body][0])) for i, t in enumerate(FRAMES)]
    ys = [(t, round(SOLVED[i][body][1])) for i, t in enumerate(FRAMES)]
    rs = [(t, round(SOLVED[i][body][2], 2)) for i, t in enumerate(FRAMES)]
    ss = [(t, [round(SOLVED[i][body][3] * S, 4), round(SOLVED[i][body][4] * S, 4)]) for i, t in enumerate(FRAMES)]
    kw = dict(x=kf(xs), y=kf(ys), origin="center", scale=kf(ss), rotation=kf(rs))
    if opacity is not None:
        kw["opacity"] = opacity
    track(f"owl-{name}", layer, img(f"owl-{name}", "character/" + part["file"], part["width"], part["height"], **kw))

# mouths from the visemes; a viseme holds until the next one
vis = {}
for ms, vid in voice["visemes"]:
    vis[W(ms)] = rig["visemes"][str(vid)]
changes = sorted(vis.items())
def mouth_opacity(m):
    recs = [{"t": 0, "v": 0.0}]
    cur = 0.0
    for t, mouth in changes:
        v = 1.0 if mouth == m else 0.0
        if v != cur:
            recs.append({"t": t, "v": v, "ease": "step"}); cur = v
    return recs

# blinks: a few frames each
BLINKS = [1500, W(1800), 5300, 7550, 9150]
def blink_opacity():
    recs = [{"t": 0, "v": 0.0}]
    for b in BLINKS:
        recs.append({"t": b, "v": 1.0, "ease": "step"})
        recs.append({"t": b + 100, "v": 0.0, "ease": "step"})
    return recs

for k, name in enumerate(ORDER):
    layer = BASE_LAYER + k
    if name.startswith("mouth_"):
        part_track(name, layer, mouth_opacity(name[len("mouth_"):]))
    elif name == "eyes_closed":
        part_track(name, layer, blink_opacity())
    else:
        part_track(name, layer)

# the question, drawn: a "?" pops beside Hoot's head
QX, QY = 690, 300
T_Q = W(500)
track("question-mark", 35, text("question-mark", [{"text": "?"}], 180, "bold", 110, 216, T_Q, W(2150),
                                x=QX, y=QY, origin="center", color=SIGNAL, stroke=OUTLINE, stroke_width=6,
                                scale=anim([(T_Q, [0.2, 0.2]), (T_Q + 220, [1.12, 1.12], "ease-out"),
                                            (T_Q + 380, [1.0, 1.0], "ease-in-out"),
                                            (W(1300), [1.0, 1.0], "linear"), (W(1450), [1.1, 1.1], "ease-out"),
                                            (W(1650), [1.0, 1.0], "ease-in-out")]),
                                rotation=anim([(T_Q, -20.0), (T_Q + 380, 10.0, "ease-out"),
                                               (W(1100), 4.0, "ease-in-out"), (W(1500), 12.0, "ease-in-out"),
                                               (W(1900), 8.0, "ease-in-out")]),
                                opacity=anim([(T_Q, 0.0), (T_Q + 120, 1.0, "ease-out"), (W(1950), 1.0, "linear"),
                                              (W(2100), 0.0, "ease-in")])))

# ------------------------------------------------------------ brand
T_BRAND = 6900
LOCK_W = 760
LOCK_H = round(LOCK_W * 623 / 2694)
BX, BY = 1180, 450
track("lockup", 40, img("lockup", "brand/lockup.png", LOCK_W, LOCK_H, T_BRAND, DUR, x=BX, y=BY, origin="center",
                        scale=anim([(T_BRAND, [0.85, 0.85]), (T_BRAND + 450, [1.0, 1.0], [0.2, 1.4, 0.4, 1.0])]),
                        opacity=anim([(T_BRAND, 0.0), (T_BRAND + 300, 1.0, "ease-out")])))
T_TAG = T_BRAND + 500
track("tagline", 41, text("tagline", [{"text": "Edit the file. Render "}, {"text": "again.", "color": SIGNAL}],
                          56, "bold", 740, 68, T_TAG, DUR, x=BX, y=BY + LOCK_H // 2 + 60, origin="top-center",
                          color=INK, opacity=anim([(T_TAG, 0.0), (T_TAG + 350, 1.0, "ease-out")]),
                          scale=anim([(T_TAG, [0.96, 0.96]), (T_TAG + 350, [1.0, 1.0], "ease-out")])))

# ------------------------------------------------------------ sound
VEND = W(voice["duration_ms"])
track("voice", 50, {"id": "voice", "type": "audio", "start": V0, "end": VEND, "source": "character/voice/line-2.wav",
                    "source_start": 0, "source_end": voice["duration_ms"], "volume": 1.5})
MUS_END = DUR
MUS_SILENT = 9700   # the bed has faded out before the end; the element runs on silent to 10 s
track("music", 51, {"id": "music", "type": "audio", "start": 0, "end": MUS_END, "source": "music/bed-120bpm.wav",
                    "source_start": 0, "source_end": MUS_END,
                    "volume": anim([(0, 0.3), (V0 - 200, 0.3, "linear"), (V0, 0.1, "ease-in-out"),
                                    (VEND, 0.1, "linear"), (VEND + 400, 0.3, "ease-in-out"),
                                    (8700, 0.3, "linear"), (MUS_SILENT, 0.0, "ease-in")])})

# ------------------------------------------------------------ write
head = json.load(open(PROJECT))
head["tracks"] = tracks
json.dump(head, open(PROJECT, "w"), ensure_ascii=False)
print("tracks", len(tracks), "lines", sum(len(t["elements"]) for t in tracks))
