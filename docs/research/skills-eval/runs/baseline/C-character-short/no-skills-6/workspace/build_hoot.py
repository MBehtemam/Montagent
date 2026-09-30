#!/usr/bin/env python3
"""Generate hoot.json: the Hoot mascot short (Brief C).

Montagent has no parenting, so the rig's hierarchy is solved here: every part's
pivot position and cumulative rotation are computed per frame and written as
per-frame keyframes. Each part's pivot is its canvas centre, so origin "center"
at the pivot's frame position reproduces the rig exactly.
"""
import json, math

FPS = 30
DUR = 8000
NF = DUR * FPS // 1000  # 240 frames
VOICE_AT = 1000
S = 0.6                 # drawing px -> frame px
HOME_X, FLOOR_Y = 640, 960

rig = json.load(open("character/rig.json"))
line = json.load(open("character/voice/line-1.json"))
P = rig["parts"]


def ms(i):   # element boundary for frame i: floor keeps frame i inside [ms(i), ms(i+1))
    return (i * 1000) // FPS


def ft(i):   # frame instant
    return i * 1000.0 / FPS


def clamp01(u):
    return max(0.0, min(1.0, u))


def smooth(u):
    u = clamp01(u)
    return u * u * (3 - 2 * u)


def lerp(a, b, u):
    return a + (b - a) * u


def track(t, keys):
    """Piecewise smoothstep through [(t, value), ...]."""
    if t <= keys[0][0]:
        return keys[0][1]
    for (t0, v0), (t1, v1) in zip(keys, keys[1:]):
        if t < t1:
            return lerp(v0, v1, smooth((t - t0) / (t1 - t0)))
    return keys[-1][1]


# ---- poses: local rotations in degrees from rest (positive = clockwise on screen)
# rest directions: upper_arm_left 156, forearm_left 149, upper_arm_right 25, forearm_right 29.5
DOWN_LU, DOWN_LF = -58, 8          # left arm hanging at the side
DOWN_RU, DOWN_RF = 58, -8
WAVE_LU = 38                       # upper arm raised out to the left, a little above horizontal
WAVE_FW = 230                      # forearm world direction centre: up and out, clear of the head
POINT_RU = -30                     # right upper arm near horizontal
POINT_FW = -8                      # right forearm world direction: at the card
CHEER_LU, CHEER_LFW = 49, 232      # left arm up and out, forearm world
CHEER_RU, CHEER_RFW = -49, 308


def left_arm(t):
    """(LU, LF) local rotations."""
    # wave: raise 950-1150, swing 1150-1950, lower 1950-2250
    up = track(t, [(950, 0), (1150, 1), (1950, 1), (2250, 0), (6620, 0), (6900, 1)])
    cheer = t >= 4000
    if not cheer:
        lu_up = WAVE_LU + 2 * math.sin(2 * math.pi * (t - 1150) / 300)
        swing = 22 * math.sin(2 * math.pi * (t - 1150) / 300) * smooth((t - 1100) / 120) * (1 - smooth((t - 1900) / 120))
        lf_up = (WAVE_FW + swing) - 149 - lu_up
    else:
        pump = 5 * math.sin(2 * math.pi * (t - 7000) / 500) * smooth((t - 6950) / 100)
        lu_up = CHEER_LU + pump
        lf_up = CHEER_LFW - 149 - lu_up + 1.5 * pump
    # a little outward lift while airborne during the hop in
    air = hop(t)[2]
    lu_dn = DOWN_LU + 22 * air
    lf_dn = DOWN_LF + 10 * air
    return lerp(lu_dn, lu_up, up), lerp(lf_dn, lf_up, up)


def right_arm(t):
    """(RU, RF) local rotations."""
    air = hop(t)[2]
    ru_dn = DOWN_RU - 22 * air
    rf_dn = DOWN_RF - 10 * air
    point = track(t, [(3520, 0), (3780, 1)])
    # emphasis nudge on "movie" (6200)
    nudge = 6 * math.sin(math.pi * clamp01((t - 6150) / 250))
    ru_pt = POINT_RU - nudge + 1.5 * math.sin(2 * math.pi * (t - 3780) / 1400)
    rf_pt = POINT_FW - 29.5 - ru_pt - nudge * 0.5
    pump = 5 * math.sin(2 * math.pi * (t - 7000) / 500) * smooth((t - 6950) / 100)
    ru_ch = CHEER_RU - pump
    rf_ch = CHEER_RFW - 360 - 29.5 - ru_ch - 1.5 * pump
    ru = lerp(ru_dn, ru_pt, point)
    rf = lerp(rf_dn, rf_pt, point)
    ch = track(t, [(6620, 0), (6900, 1)])
    return lerp(ru, ru_ch, ch), lerp(rf, rf_ch, ch)


HOPS = [(0, 440, -330, 190, 150), (440, 880, 190, HOME_X, 120)]  # (t0, t1, x0, x1, height)


def hop(t):
    """(x, lift, airborne 0..1, lean deg)."""
    for t0, t1, x0, x1, h in HOPS:
        if t0 <= t < t1:
            u = (t - t0) / (t1 - t0)
            return lerp(x0, x1, u), h * math.sin(math.pi * u), math.sin(math.pi * u), 7 * math.sin(math.pi * u)
    if t < 0:
        return HOPS[0][2], 0, 0, 0
    return HOME_X, 0, 0, 0


def squash(t):
    """(sx, sy) of the torso: contacts at 440 and 880, a small cheer bounce at 6950."""
    sx = sy = 1.0
    for tc, amt, rec in [(440, 0.10, 160), (880, 0.14, 260), (6950, 0.06, 220)]:
        d = t - tc
        if -60 <= d < rec:
            if d < 0:
                k = amt * 0.4 * (1 + d / 60)
            else:
                u = d / rec
                k = amt * math.exp(-4 * u) * math.cos(2.5 * math.pi * u) * (1 - u)
            sy -= k
            sx += k * 0.6
    breathe = 0.008 * math.sin(2 * math.pi * (t - 900) / 1800) * smooth((t - 900) / 300)
    return sx - breathe * 0.3, sy + breathe


def cheer_lift(t):
    d = t - 6980
    if 0 <= d < 360:
        return 26 * math.sin(math.pi * d / 360)
    return 0


def head_tilt(t):
    talk = smooth((t - 1000) / 300)
    tilt = talk * (2.5 * math.sin(2 * math.pi * (t - 1000) / 1900) + 1.2 * math.sin(2 * math.pi * (t - 1000) / 730))
    tilt += track(t, [(3600, 0), (3900, 5), (6600, 5), (6900, -2)])  # lean towards the card
    tilt += -3 * hop(t)[2]
    return tilt


def rot(v, deg):
    a = math.radians(deg)
    c, s = math.cos(a), math.sin(a)
    return (v[0] * c - v[1] * s, v[0] * s + v[1] * c)


def solve(t):
    """World transform {part: (x, y, rot_deg, sx, sy)} at time t."""
    hx, lift, air, lean = hop(t)
    sx, sy = squash(t)
    tor = (hx, FLOOR_Y - lift - cheer_lift(t), lean + 0.8 * math.sin(2 * math.pi * (t - 900) / 2600) * smooth((t - 900) / 300))
    out = {"torso": (tor[0], tor[1], tor[2], S * sx, S * sy)}

    def child(name, parent_world, local):
        px, py, pr, psx, psy = parent_world
        par = P[name]["parent"]
        off = (P[name]["pivot"][0] - P[par]["pivot"][0], P[name]["pivot"][1] - P[par]["pivot"][1])
        off = (off[0] * psx, off[1] * psy)
        dx, dy = rot(off, pr)
        return (px + dx, py + dy, pr + local, S, S)

    lu, lf = left_arm(t)
    ru, rf = right_arm(t)
    out["upper_arm_left"] = child("upper_arm_left", out["torso"], lu)
    out["upper_arm_right"] = child("upper_arm_right", out["torso"], ru)
    out["forearm_left"] = child("forearm_left", out["upper_arm_left"], lf)
    out["forearm_right"] = child("forearm_right", out["upper_arm_right"], rf)
    out["head"] = child("head", out["torso"], head_tilt(t))
    return out


frames = [solve(ft(i)) for i in range(NF + 1)]


def compact(ks):
    """Drop records equal to both neighbours (exact under linear interpolation)."""
    if all(k["v"] == ks[0]["v"] for k in ks):
        return ks[0]["v"]
    out = [ks[0]]
    for a, b, c in zip(ks, ks[1:], ks[2:]):
        if not (a["v"] == b["v"] == c["v"]):
            out.append(b)
    out.append(ks[-1])
    return out


def kf_int(vals, i0, i1):
    ks = []
    for i in range(i0, i1 + 1):
        k = {"t": ms(i), "v": int(round(vals[i]))}
        if ks:
            k["ease"] = "linear"
        ks.append(k)
    return compact(ks)


def kf_num(vals, i0, i1, nd=3):
    ks = []
    for i in range(i0, i1 + 1):
        k = {"t": ms(i), "v": round(vals[i], nd)}
        if ks:
            k["ease"] = "linear"
        ks.append(k)
    return compact(ks)


def kf_scale(vals, i0, i1):
    ks = []
    for i in range(i0, i1 + 1):
        k = {"t": ms(i), "v": [round(vals[i][0], 4), round(vals[i][1], 4)]}
        if ks:
            k["ease"] = "linear"
        ks.append(k)
    return compact(ks)


def part_el(eid, part, i0, i1, src=None, transform_of=None, group="hoot"):
    """Image element for a rig part over frames [i0, i1) (keyframes run one past)."""
    tp = transform_of or part
    w = frames
    last = min(i1, NF) - 1
    xs = [f[tp][0] for f in w]
    ys = [f[tp][1] for f in w]
    rs = [f[tp][2] for f in w]
    ss = [(f[tp][3], f[tp][4]) for f in w]
    scale_const = all(abs(s[0] - S) < 1e-9 and abs(s[1] - S) < 1e-9 for s in ss[i0:last + 1])
    el = {
        "id": eid, "type": "image", "group": group, "start": ms(i0), "end": ms(i1) if i1 < NF else DUR,
        "source": "character/" + (src or P[part]["file"]),
        "x": kf_int(xs, i0, last), "y": kf_int(ys, i0, last), "origin": "center",
        "width": P[part]["width"], "height": P[part]["height"], "fit": "contain",
        "scale": [S, S] if scale_const else kf_scale(ss, i0, last),
        "rotation": kf_num(rs, i0, last, 2),
    }
    return el


# ---- mouths: per frame, the viseme covering most of the frame interval (voice clock)
vis = line["visemes"]


def mouth_at(i):
    a = ft(i) - VOICE_AT
    b = a + 1000.0 / FPS
    cover = {}
    for j, (vt, vid) in enumerate(vis):
        vend = vis[j + 1][0] if j + 1 < len(vis) else 1e9
        lo, hi = max(a, vt), min(b, vend)
        if hi > lo:
            m = rig["visemes"][str(vid)]
            cover[m] = cover.get(m, 0) + (hi - lo)
    if a + 1000.0 / FPS <= vis[0][0]:
        return None
    if not cover:
        return None
    return max(cover.items(), key=lambda kv: kv[1])[0]


mouths = [mouth_at(i) for i in range(NF)]
mouth_els = []
i = 0
n = 0
while i < NF:
    m = mouths[i]
    j = i
    while j < NF and mouths[j] == m:
        j += 1
    if m is not None:
        n += 1
        mouth_els.append(part_el(f"mouth-{n:02d}-{m}", "head", i, j, src=f"parts/mouth_{m}.png", transform_of="head"))
    i = j

# ---- blinks: eyes_closed for 3 frames each
blink_els = []
for k, t0 in enumerate([1560, 3330, 5050, 7400]):
    i0 = round(t0 * FPS / 1000)
    blink_els.append(part_el(f"blink-{k + 1}", "head", i0, i0 + 3, src="parts/eyes_closed.png", transform_of="head"))

# ---- floor shadow
shadow_w = []
shadow_o = []
for i in range(NF + 1):
    hx, lift, air, lean = hop(ft(i))
    lift += cheer_lift(ft(i))
    shadow_w.append(1 - 0.35 * min(1, lift / 150))
    shadow_o.append(0.22 * (1 - 0.5 * min(1, lift / 150)))
shadow = {
    "id": "owl-shadow", "type": "ellipse", "group": "hoot", "start": 0, "end": DUR,
    "x": kf_int([hop(ft(i))[0] for i in range(NF + 1)], 0, NF), "y": FLOOR_Y + 4, "origin": "center",
    "width": 300, "height": 44, "fill": "#5A3A22",
    "scale": kf_scale([(w, w) for w in shadow_w], 0, NF),
    "opacity": kf_num(shadow_o, 0, NF),
}

# ---- card
CARD_X, CARD_Y = 1290, 520
CW, CH = 688, 408
IW, IH = 640, 360
POP = 3700
card_scale = [
    {"t": POP, "v": [0.0, 0.0]},
    {"t": POP + 200, "v": [1.1, 1.1], "ease": "ease-out"},
    {"t": POP + 330, "v": [0.96, 0.96], "ease": "ease-in-out"},
    {"t": POP + 440, "v": [1.0, 1.0], "ease": "ease-in-out"},
    {"t": 6200, "v": [1.0, 1.0], "ease": "linear"},
    {"t": 6300, "v": [1.05, 1.05], "ease": "ease-out"},
    {"t": 6450, "v": [1.0, 1.0], "ease": "ease-in-out"},
]
card_rot = [
    {"t": POP, "v": -6.0},
    {"t": POP + 260, "v": 1.5, "ease": "ease-out"},
    {"t": POP + 440, "v": 0.0, "ease": "ease-in-out"},
]
card = {
    "id": "card", "type": "rect", "group": "card", "start": POP, "end": DUR,
    "x": CARD_X, "y": CARD_Y, "origin": "center", "width": CW, "height": CH,
    "fill": "#F5F0E6", "stroke": "#101418", "stroke_width": 6, "radius": 28,
    "scale": card_scale, "rotation": card_rot,
    "effects": [{"name": "shadow", "dx": 0, "dy": 14, "radius": 22, "color": "#3A2412", "opacity": 0.28}],
}


def still(eid, src, start, end):
    return {
        "id": eid, "type": "image", "group": "card", "start": start, "end": end, "source": src,
        "x": CARD_X, "y": CARD_Y, "origin": "center", "width": IW, "height": IH, "fit": "cover",
        "scale": card_scale, "rotation": card_rot,
        "effects": [{"name": "mask", "shape": "rect", "x": 0, "y": 0, "width": IW, "height": IH, "radius": 14}],
    }


MOVIE = VOICE_AT + next(w["start"] for w in line["words"] if w["word"] == "movie")
still1 = still("card-still-01", "stills/session-01.png", POP, MOVIE)
still2 = still("card-still-02", "stills/session-02.png", MOVIE, DUR)

# ---- lockup
LW, LH = 700, 162
LAND = 6850
lockup = {
    "id": "lockup", "type": "image", "group": "brand", "start": LAND, "end": DUR, "source": "brand/lockup.png",
    "x": HOME_X, "y": [
        {"t": LAND, "v": -110},
        {"t": LAND + 220, "v": 140, "ease": "ease-in"},
        {"t": LAND + 340, "v": 118, "ease": "ease-out"},
        {"t": LAND + 460, "v": 140, "ease": "ease-in"},
    ], "origin": "center", "width": LW, "height": LH, "fit": "contain",
    "scale": [
        {"t": LAND, "v": [1.0, 1.0]},
        {"t": LAND + 220, "v": [1.0, 1.0], "ease": "linear"},
        {"t": LAND + 280, "v": [1.06, 0.9], "ease": "ease-out"},
        {"t": LAND + 400, "v": [1.0, 1.0], "ease": "ease-in-out"},
    ],
}

# ---- audio
voice = {"id": "voice-line-1", "type": "audio", "start": VOICE_AT, "end": VOICE_AT + 6048,
         "source": "character/voice/line-1.wav", "source_start": 0, "source_end": 6048}
music = {"id": "music-bed", "type": "audio", "start": 0, "end": DUR, "source": "music/bed-120bpm.wav",
         "source_start": 0, "source_end": DUR, "volume": [
             {"t": 0, "v": 0.0}, {"t": 400, "v": 0.14, "ease": "ease-out"},
             {"t": 900, "v": 0.14, "ease": "linear"}, {"t": 1200, "v": 0.08, "ease": "ease-in-out"},
             {"t": 7100, "v": 0.08, "ease": "linear"}, {"t": 7300, "v": 0.14, "ease": "ease-out"},
             {"t": 7500, "v": 0.14, "ease": "linear"}, {"t": 7966, "v": 0.0, "ease": "ease-in"}]}

bg = {"id": "study", "type": "image", "start": 0, "end": DUR, "source": "character/study.png",
      "x": 0, "y": 0, "origin": "top-left", "width": 1920, "height": 1080, "fit": "cover"}

tracks = [
    ("set", 0, [bg]),
    ("floor-shadow", 1, [shadow]),
    ("card", 5, [card]),
    ("card-still", 6, [still1, still2]),
    ("torso", 10, [part_el("torso", "torso", 0, NF)]),
    ("forearm-left", 11, [part_el("forearm-left", "forearm_left", 0, NF)]),
    ("forearm-right", 12, [part_el("forearm-right", "forearm_right", 0, NF)]),
    ("upper-arm-left", 13, [part_el("upper-arm-left", "upper_arm_left", 0, NF)]),
    ("upper-arm-right", 14, [part_el("upper-arm-right", "upper_arm_right", 0, NF)]),
    ("head", 15, [part_el("head", "head", 0, NF)]),
    ("mouth", 16, mouth_els),
    ("eyes", 17, blink_els),
    ("lockup", 30, [lockup]),
    ("voice", 40, [voice]),
    ("music", 41, [music]),
]

proj = {
    "frame": {"width": 1920, "height": 1080}, "fps": FPS, "background": "#F5F0E6", "duration": DUR,
    "output": "deliverable.mp4",
    "tracks": [{"name": n, "layer": l, "elements": e} for n, l, e in tracks],
}
json.dump(proj, open("hoot.json", "w"), ensure_ascii=False)
print("mouth segments:", len(mouth_els), " movie at", MOVIE)
