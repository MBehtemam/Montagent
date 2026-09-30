#!/usr/bin/env python3
"""Generate hoot.json: the Hoot mascot short (Brief C).

Montagent transforms are flat, so the rig's hierarchy is solved here by forward
kinematics and baked into per-frame keyframes for every part.
"""
import json
import math

FPS = 30
DUR = 8000
S = 0.62                      # drawing px -> frame px
VOICE_AT = 1000               # the line starts at 1 s

rig = json.load(open("character/rig.json"))
line = json.load(open("character/voice/line-1.json"))
P = {k: v for k, v in rig["parts"].items()}

FRAMES = [round(n * 1000 / FPS) for n in range(DUR * FPS // 1000 + 1)]


# ---------------------------------------------------------------- curves
def ease(kind, u):
    if kind == "lin":
        return u
    if kind == "io":
        return 3 * u * u - 2 * u * u * u
    if kind == "o":
        return 1 - (1 - u) ** 3
    if kind == "i":
        return u ** 3
    if kind == "back":  # ease-out with overshoot
        c = 1.9
        return 1 + (c + 1) * (u - 1) ** 3 + c * (u - 1) ** 2
    raise ValueError(kind)


def curve(keys, t):
    """keys: [(t, v), (t, v, ease), ...]; ease describes arrival at a key."""
    if t <= keys[0][0]:
        return keys[0][1]
    for a, b in zip(keys, keys[1:]):
        if t <= b[0]:
            u = (t - a[0]) / (b[0] - a[0])
            k = b[2] if len(b) > 2 else "io"
            return a[1] + (b[1] - a[1]) * ease(k, u)
    return keys[-1][1]


def env(t, a, b, c, d):
    """0 before a, ramps to 1 by b, holds, back to 0 from c to d."""
    if t <= a or t >= d:
        return 0.0
    if t < b:
        return ease("io", (t - a) / (b - a))
    if t > c:
        return ease("io", (d - t) / (d - c))
    return 1.0


# ---------------------------------------------------------------- motion
HOPS = [(0, 300, -330, 40, 80), (300, 600, 40, 360, 64), (600, 900, 360, 620, 46)]
ROOT_X, ROOT_Y = 620, 1000


def root(t):
    x, off = ROOT_X, 0.0
    for a, b, x0, x1, h in HOPS:
        if a <= t <= b:
            u = (t - a) / (b - a)
            x = x0 + (x1 - x0) * u
            off = h * 4 * u * (1 - u)
            break
    if t < 0:
        x = HOPS[0][2]
    # a small jump for the cheer
    if 7000 <= t <= 7320:
        u = (t - 7000) / 320
        off = 34 * 4 * u * (1 - u)
    return x, ROOT_Y - off, off


def torso_rot(t):
    lean = curve([(0, 7), (880, 7), (1000, -3.5, "o"), (1180, 0, "io")], t)
    sway = 1.1 * math.sin(2 * math.pi * (t - 1180) / 2600) * env(t, 1180, 1600, 7700, 8000)
    return lean + sway


def head_rot(t):
    v = curve([(0, -4), (880, -4), (1000, 5, "o"), (1200, 0, "io"),
               (1300, 0), (1450, 4), (2000, 4), (2250, 0),
               (3600, 0), (3900, 6), (6250, 6), (6550, 0),
               (6900, 0), (7150, -3), (7500, 2), (7800, 0)], t)
    idle = 1.8 * math.sin(2 * math.pi * t / 1900 + 0.7) * env(t, 1100, 1400, 7600, 8000)
    return v + idle


# arm locals: left positive = raise, right negative = raise. The torso's shoulder
# cut shows if an upper arm leaves roughly [-22, +2] degrees of rest, so the upper
# arms stay in that band and the forearms do the big moves.
SIDE_UP, SIDE_FORE = 17.0, 16.0     # arms hanging at the sides (mirrored per side)


def hop_flap(t):
    for a, b, *_ in HOPS:
        if a <= t <= b:
            return math.sin(math.pi * (t - a) / (b - a))
    return 0.0


def pump(t):
    return math.sin(2 * math.pi * (t - 7150) / 330) * env(t, 7120, 7200, 7650, 7850)


def arm_l(t):
    f = hop_flap(t)
    up = curve([(0, -SIDE_UP), (1000, -SIDE_UP), (1180, 2, "o"), (1980, 2),
                (2300, -SIDE_UP, "io"), (6850, -SIDE_UP), (7100, 2, "o")], t)
    fore = curve([(0, -SIDE_FORE), (1000, -SIDE_FORE), (1200, 106, "back"), (1950, 106),
                  (2300, -SIDE_FORE, "io"), (6850, -SIDE_FORE), (7120, 112, "back")], t)
    wave = 21 * math.sin(2 * math.pi * (t - 1190) / 330) * env(t, 1170, 1260, 1860, 1960)
    return up + 12 * f, fore + 30 * f + wave + 12 * pump(t)


def arm_r(t):
    f = hop_flap(t)
    up = curve([(0, SIDE_UP), (3550, SIDE_UP), (3850, -3, "o"), (6850, -3), (7100, -2, "o")], t)
    fore = curve([(0, SIDE_FORE), (3550, SIDE_FORE), (3860, -30, "back"),
                  (6150, -30), (6260, -38, "o"), (6400, -30, "io"),
                  (6850, -30), (7120, -112, "back")], t)
    hold = 2 * math.sin(2 * math.pi * (t - 3900) / 1700) * env(t, 3900, 4200, 5900, 6150)
    return up - 12 * f, fore - 30 * f + hold - 12 * pump(t)


# ---------------------------------------------------------------- FK
def rotv(dx, dy, deg):
    r = math.radians(deg)  # clockwise on screen (y down)
    return dx * math.cos(r) - dy * math.sin(r), dx * math.sin(r) + dy * math.cos(r)


def pose(t):
    rx, ry, off = root(t)
    out = {}
    tr = torso_rot(t)
    out["torso"] = (rx, ry, tr)

    def child(name, parent_name, local):
        px, py, prot = out[parent_name]
        cp, pp = P[name]["pivot"], P[parent_name]["pivot"]
        dx, dy = rotv((cp[0] - pp[0]) * S, (cp[1] - pp[1]) * S, prot)
        out[name] = (px + dx, py + dy, prot + local)

    child("head", "torso", head_rot(t))
    ul, fl = arm_l(t)
    ur, fr = arm_r(t)
    child("upper_arm_left", "torso", ul)
    child("forearm_left", "upper_arm_left", fl)
    child("upper_arm_right", "torso", ur)
    child("forearm_right", "upper_arm_right", fr)
    out["_off"] = off
    return out


POSES = {t: pose(t) for t in FRAMES}


def compress(seq):
    """Drop keyframes inside runs of identical values."""
    out = []
    for i, (t, v) in enumerate(seq):
        if 0 < i < len(seq) - 1 and seq[i - 1][1] == v == seq[i + 1][1]:
            continue
        out.append((t, v))
    return out


def kf(seq):
    seq = compress(seq)
    if len(seq) == 1:
        return seq[0][1]
    recs = []
    for i, (t, v) in enumerate(seq):
        r = {"t": t, "v": v}
        if i:
            r["ease"] = "linear"
        recs.append(r)
    return recs


def part_tracks(name, lo=None, hi=None):
    ts = FRAMES
    if lo is not None:
        i0 = max(i for i, t in enumerate(FRAMES) if t <= lo)
        i1 = min(i for i, t in enumerate(FRAMES) if t >= hi) if hi < FRAMES[-1] else len(FRAMES) - 1
        ts = FRAMES[i0:i1 + 1]
    xs = [(t, round(POSES[t][name][0])) for t in ts]
    ys = [(t, round(POSES[t][name][1])) for t in ts]
    rs = [(t, round(POSES[t][name][2], 2)) for t in ts]
    return kf(xs), kf(ys), kf(rs)


def part_el(eid, name, src_part, start, end, lo=None, hi=None):
    x, y, r = part_tracks(name, lo, hi)
    p = P[src_part]
    return {"id": eid, "type": "image", "start": start, "end": end,
            "source": "character/" + p["file"], "x": x, "y": y, "origin": "center",
            "width": p["width"], "height": p["height"], "fit": "literal",
            "scale": [S, S], "rotation": r}


tracks = []


def track(name, layer, elements):
    tracks.append({"name": name, "layer": layer, "elements": elements})


track("set", 0, [{"id": "study", "type": "image", "start": 0, "end": DUR,
                  "source": "character/study.png", "x": 0, "y": 0, "origin": "top-left", "width": 1920,
                  "height": 1080, "fit": "literal"}])

# floor shadow under the owl, shrinking while it is in the air
sx = kf([(t, round(POSES[t]["torso"][0])) for t in FRAMES])
ssc = kf([(t, [round(1 - POSES[t]["_off"] / 220, 3)] * 2) for t in FRAMES])
track("owl-shadow", 5, [{"id": "owl-shadow", "type": "ellipse", "start": 0, "end": DUR,
                         "x": sx, "y": ROOT_Y + 14, "origin": "center", "width": 300,
                         "height": 44, "fill": "#5A3A1E", "scale": ssc, "opacity": 0.28,
                         "effects": [{"name": "blur", "radius": 6}]}])

# ---------------------------------------------------------------- card
CARD_X, CARD_Y = 1300, 560
CARD_IN, SWAP = 3700, VOICE_AT + next(w["start"] for w in line["words"] if w["word"] == "movie")
card_scale = []
for t in [CARD_IN] + [t for t in FRAMES if CARD_IN < t <= CARD_IN + 400] + \
         [t for t in FRAMES if SWAP - 10 <= t <= SWAP + 260]:
    if t <= CARD_IN + 400:
        u = min(1.0, (t - CARD_IN) / 400)
        v = 0.3 + 0.7 * ease("back", u)
    else:
        u = (t - SWAP) / 250
        v = 1 + 0.04 * math.sin(math.pi * u) if 0 <= u <= 1 else 1.0
    card_scale.append((t, [round(v, 4)] * 2))
card_scale = kf(card_scale)
card_op = [{"t": CARD_IN, "v": 0.0}, {"t": CARD_IN + 100, "v": 1.0, "ease": "ease-out"}]

track("card-frame", 20, [{"id": "card-frame", "type": "rect", "start": CARD_IN, "end": DUR,
                          "x": CARD_X, "y": CARD_Y, "origin": "center", "width": 700,
                          "height": 413, "fill": "#101418", "radius": 22,
                          "scale": card_scale, "opacity": card_op,
                          "effects": [{"name": "shadow", "dx": 0, "dy": 14, "radius": 22,
                                       "color": "#3A2410", "opacity": 0.35}]}])


def card_img(eid, src, start, end, op):
    return {"id": eid, "type": "image", "start": start, "end": end, "source": src,
            "x": CARD_X, "y": CARD_Y, "origin": "center", "width": 672, "height": 378,
            "fit": "literal", "scale": card_scale, "opacity": op,
            "effects": [{"name": "mask", "shape": "rect", "radius": 10}]}


track("card-picture", 21, [card_img("card-session-01", "stills/session-01.png", CARD_IN, SWAP, card_op),
                           card_img("card-session-02", "stills/session-02.png", SWAP, DUR, 1.0)])

# ---------------------------------------------------------------- owl
LAYER = {n: 30 + i for i, n in enumerate(rig["draw_order"])}
for n in ["torso", "forearm_left", "forearm_right", "upper_arm_left", "upper_arm_right", "head"]:
    track("owl-" + n.replace("_", "-"), LAYER[n], [part_el(n.replace("_", "-"), n, n, 0, DUR)])

# mouths from the visemes
segs = []
vis = line["visemes"]
for i, (ms, vid) in enumerate(vis):
    nxt = vis[i + 1][0] if i + 1 < len(vis) else line["duration_ms"]
    if nxt <= ms:
        continue
    mouth = rig["visemes"][str(vid)]
    a, b = VOICE_AT + ms, VOICE_AT + nxt
    if segs and segs[-1][2] == mouth and segs[-1][1] == a:
        segs[-1][1] = b
    else:
        segs.append([a, b, mouth])
mouth_els = []
for a, b, mouth in segs:
    if mouth is None:
        continue
    el = part_el(f"mouth-{a}", "head", "mouth_" + mouth, a, b, a, b)
    mouth_els.append(el)
track("owl-mouth", 40, mouth_els)

BLINKS = [2150, 4500, 6700, 7850]
track("owl-eyes", 41, [part_el(f"blink-{a}", "head", "eyes_closed", a, a + 100, a, a + 100)
                       for a in BLINKS if a + 100 <= DUR])

# ---------------------------------------------------------------- brand
LOCK_IN = 6950
lock_y = []
for t in [LOCK_IN] + [t for t in FRAMES if LOCK_IN < t <= LOCK_IN + 450]:
    u = min(1.0, (t - LOCK_IN) / 450)
    lock_y.append((t, round(-120 + (150 + 120) * ease("back", u))))
track("lockup", 90, [{"id": "lockup", "type": "image", "start": LOCK_IN, "end": DUR,
                      "source": "brand/lockup.png", "x": ROOT_X, "y": kf(lock_y),
                      "origin": "center", "width": 640, "height": 148, "fit": "literal",
                      "opacity": [{"t": LOCK_IN, "v": 0.0},
                                  {"t": LOCK_IN + 150, "v": 1.0, "ease": "ease-out"}]}])

# ---------------------------------------------------------------- sound
track("voice", 0, [{"id": "voice-line-1", "type": "audio", "start": VOICE_AT, "end": VOICE_AT + 6048,
                    "source": "character/voice/line-1.wav", "source_start": 0, "source_end": 6048}])
track("music", 0, [{"id": "music-bed", "type": "audio", "start": 0, "end": DUR,
                    "source": "music/bed-120bpm.wav", "source_start": 0, "source_end": DUR,
                    "volume": [{"t": 0, "v": 0.0}, {"t": 400, "v": 0.1, "ease": "linear"},
                               {"t": 7200, "v": 0.1, "ease": "linear"},
                               {"t": DUR, "v": 0.0, "ease": "linear"}]}])

project = {"frame": {"width": 1920, "height": 1080}, "fps": FPS, "background": "#F5F0E6",
           "duration": DUR, "output": "deliverable.mp4", "tracks": tracks}
json.dump(project, open("hoot.json", "w"), ensure_ascii=False)
print("mouth segments:", [(a, b, m) for a, b, m in segs])
