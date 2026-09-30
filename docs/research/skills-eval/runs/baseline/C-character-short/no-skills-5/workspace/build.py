#!/usr/bin/env python3
"""Generate hoot.json: the Hoot mascot short (Brief C).

Montagent transforms are flat, so the rig's hierarchy (torso > head / upper arms >
forearms) is solved here by forward kinematics and written as per-frame keyframes.
"""
import json, math

FPS = 30
DUR = 8000
NF = DUR * FPS // 1000  # 240 frames
VOICE_AT = 1000

rig = json.load(open("character/rig.json"))
vo = json.load(open("character/voice/line-1.json"))
P = rig["parts"]

S = 0.58             # owl scale (drawing px -> frame px)
FLOOR_Y = 962        # where the feet (torso pivot) stand
HOME_X = 660


def ft(n):
    """Integer ms at-or-before frame n's instant."""
    return n * 1000 // FPS


# ---------- curves -------------------------------------------------------------
def smooth(u):
    return u * u * (3 - 2 * u)


def back_out(u, k=1.9):
    u -= 1
    return 1 + (k + 1) * u ** 3 + k * u ** 2


EASES = {"lin": lambda u: u, "io": smooth,
         "out": lambda u: 1 - (1 - u) ** 3, "in": lambda u: u ** 3, "back": back_out}


def curve(keys):
    """keys: [(t_ms, value, ease_into)] -> f(t)."""
    def f(t):
        if t <= keys[0][0]:
            return keys[0][1]
        for (t0, v0, _), (t1, v1, e) in zip(keys, keys[1:]):
            if t <= t1:
                u = (t - t0) / (t1 - t0)
                return v0 + (v1 - v0) * EASES[e](u)
        return keys[-1][1]
    return f


# ---------- performance ----------------------------------------------------------
# Hop in: two hops from off the left edge, landing at 800 ms.
HOPS = [(0, 400, -340, 170, 120), (400, 800, 170, HOME_X, 95)]


def root(t):
    """(x, y, rotation) of the torso pivot (the feet)."""
    for t0, t1, x0, x1, h in HOPS:
        if t0 <= t < t1:
            u = (t - t0) / (t1 - t0)
            x = x0 + (x1 - x0) * u
            y = FLOOR_Y - h * 4 * u * (1 - u)
            rot = 9 * math.sin(math.pi * u) - 3 * math.sin(2 * math.pi * u)
            return x, y, rot
    # landed: recoil, then a gentle idle sway about the feet
    recoil = curve([(800, 0, "lin"), (900, -4.5, "out"), (1100, 1.5, "io"), (1300, 0, "io")])(t)
    sway = 1.2 * math.sin(2 * math.pi * (t - 1300) / 2600) * min(1, max(0, (t - 1300) / 400))
    # little hop of joy with the cheer
    bounce = curve([(6800, 0, "lin"), (6950, -2, "io"), (7150, 0, "io")])(t)
    lean = curve([(3500, 0, "lin"), (3750, 3, "io"), (5100, 3, "lin"), (5400, 0, "io"),
                  (6000, 0, "lin"), (6200, 2.5, "io"), (6750, 2.5, "lin"), (6950, 0, "io")])(t)
    return HOME_X, FLOOR_Y, recoil + sway + bounce + lean


# head tilt relative to torso
head_rot = lambda t: (
    curve([(0, 0, "lin"), (800, 0, "lin"), (900, 6, "out"), (1150, -2, "io"), (1350, 0, "io"),
           # "Hi, I'm Hoot!" friendly tilt
           (1400, 0, "lin"), (1600, -6, "io"), (2000, -6, "lin"), (2300, 0, "io"),
           # "writes the video" small nod
           (2900, 0, "lin"), (3050, 3, "io"), (3250, 0, "io"),
           # "as a file" look toward the card
           (3550, 0, "lin"), (3800, 7, "io"), (5000, 7, "lin"), (5400, 0, "io"),
           (6000, 0, "lin"), (6200, 6, "io"), (6700, 6, "lin"), (6950, -4, "io"), (7300, 0, "io")])(t)
    + 1.5 * math.sin(2 * math.pi * (t - 1300) / 1900) * min(1, max(0, (t - 1300) / 400)))

# arms: local rotations; 0 = rest pose (out and down 25 deg). Viewer's left arm waves.
SIDE = 57
ual = curve([  # upper arm left: positive = raise
    (0, -10, "lin"), (200, 25, "io"), (400, -15, "io"), (600, 20, "io"), (800, -SIDE, "out"),
    (950, -SIDE, "lin"), (1150, 30, "back"), (2050, 34, "io"), (2350, -SIDE, "io"),
    (6780, -SIDE, "lin"), (7050, 40, "back"), (7450, 34, "io"), (8000, 38, "io")])
fal = curve([  # forearm left, relative to upper arm: the wave
    (0, 0, "lin"), (800, -6, "io"),
    (950, -6, "lin"), (1150, 45, "back"),
    (1300, 10, "io"), (1450, 58, "io"), (1600, 12, "io"), (1750, 58, "io"),
    (1900, 14, "io"), (2050, 40, "io"), (2350, -6, "io"),
    (6780, -6, "lin"), (7050, 40, "back"), (7300, 28, "io"), (7550, 42, "io"), (8000, 34, "io")])
uar = curve([  # upper arm right: negative = raise
    (0, 10, "lin"), (200, -25, "io"), (400, 15, "io"), (600, -20, "io"), (800, SIDE, "out"),
    (3500, SIDE, "lin"), (3780, -44, "back"), (4400, -40, "io"), (5100, -42, "io"), (5450, SIDE, "io"),
    (5950, SIDE, "lin"), (6200, -42, "back"), (6700, -40, "io"),
    (7050, -40, "back"), (7450, -34, "io"), (8000, -38, "io")])
far = curve([  # forearm right: open presenting hand
    (0, 0, "lin"), (800, 6, "io"),
    (3500, 6, "lin"), (3780, -14, "back"), (4400, -6, "io"), (5100, -10, "io"), (5450, 6, "io"),
    (5950, 6, "lin"), (6200, -12, "back"), (6700, -6, "io"),
    (7050, -40, "back"), (7300, -28, "io"), (7550, -42, "io"), (8000, -34, "io")])

LOCAL = {"torso": None, "head": head_rot, "upper_arm_left": ual, "forearm_left": fal,
         "upper_arm_right": uar, "forearm_right": far}


def rot2(vx, vy, deg):
    a = math.radians(deg)
    c, s = math.cos(a), math.sin(a)
    return vx * c - vy * s, vx * s + vy * c


def pose(t):
    """World (x, y, rotation) of every part's pivot at time t."""
    out = {}
    rx, ry, rr = root(t)
    out["torso"] = (rx, ry, rr)
    for name in ["head", "upper_arm_left", "upper_arm_right", "forearm_left", "forearm_right"]:
        par = P[name]["parent"]
        px, py, pr = out[par]
        dx = (P[name]["pivot"][0] - P[par]["pivot"][0]) * S
        dy = (P[name]["pivot"][1] - P[par]["pivot"][1]) * S
        ox, oy = rot2(dx, dy, pr)
        out[name] = (px + ox, py + oy, pr + LOCAL[name](t))
    return out


POSES = [pose(ft(n)) for n in range(NF + 1)]


def kf(vals, frames, integer):
    recs = []
    for i, n in enumerate(frames):
        v = round(vals[n]) if integer else round(vals[n], 2)
        r = {"t": ft(n), "v": v}
        if i:
            r["ease"] = "linear"
        recs.append(r)
    return recs


def part_el(eid, name, part_for_pose, start, end, layer=None):
    f0 = max(0, start * FPS // 1000 - 1)
    f1 = min(NF, -(-end * FPS // 1000) + 1)
    frames = list(range(f0, f1 + 1))
    xs = {n: POSES[n][part_for_pose][0] for n in frames}
    ys = {n: POSES[n][part_for_pose][1] for n in frames}
    rs = {n: POSES[n][part_for_pose][2] for n in frames}
    p = P[name]
    el = {"id": eid, "type": "image", "start": start, "end": end}
    if layer is not None:
        el["layer"] = layer
    el.update({"source": "character/" + p["file"],
               "x": kf(xs, frames, True), "y": kf(ys, frames, True), "origin": "center",
               "width": p["width"], "height": p["height"], "fit": "literal",
               "scale": [S, S], "rotation": kf(rs, frames, False)})
    return el


tracks = []


def track(name, layer, elements):
    tracks.append({"name": name, "layer": layer, "elements": elements})


# ---------- set --------------------------------------------------------------------
track("study", 0, [{"id": "study", "type": "image", "start": 0, "end": DUR,
                    "source": "character/study.png", "x": 0, "y": 0, "origin": "top-left",
                    "width": 1920, "height": 1080, "fit": "cover"}])

# contact shadow under the feet, shrinking as the owl leaves the floor
sh_x, sh_s = [], []
for n in range(NF + 1):
    x, y, _ = POSES[n]["torso"]
    sh_x.append(x)
    k = 1 - 0.45 * (FLOOR_Y - y) / 120
    sh_s.append(k)
frames = list(range(NF + 1))
track("shadow", 1, [{"id": "owl-shadow", "type": "ellipse", "start": 0, "end": DUR,
                     "x": kf(sh_x, frames, True), "y": FLOOR_Y + 6, "origin": "center",
                     "width": 250, "height": 34, "fill": "#6B42203D",
                     "scale": [{"t": ft(n), "v": [round(sh_s[n], 3), round(sh_s[n], 3)],
                                **({"ease": "linear"} if n else {})} for n in frames]}])

# ---------- card ----------------------------------------------------------------------
CARD_X, CARD_Y = 1290, 470
CW, CH = 576, 324
POP = 3700          # "as"
MOVIE = VOICE_AT + [w for w in vo["words"] if w["word"] == "movie"][0]["start"]  # 6200
card_scale = [(POP, 0.0), (POP + 170, 1.12), (POP + 290, 0.96), (POP + 400, 1.0),
              (MOVIE, 1.0), (MOVIE + 90, 1.05), (MOVIE + 220, 1.0)]


def scale_kf(keys):
    out = []
    for i, (t, v) in enumerate(keys):
        r = {"t": t, "v": [v, v]}
        if i:
            r["ease"] = "ease-out" if v > keys[i - 1][1] else "ease-in-out"
        out.append(r)
    return out


shadow = [{"name": "shadow", "dx": 0, "dy": 10, "radius": 18, "color": "#3B2412", "opacity": 0.3}]
track("card-back", 4, [{"id": "card-back", "type": "rect", "group": "card", "start": POP, "end": DUR,
                        "x": CARD_X, "y": CARD_Y, "origin": "center", "width": CW + 36, "height": CH + 36,
                        "fill": "#F5F0E6", "stroke": "#101418", "stroke_width": 6, "radius": 22,
                        "scale": scale_kf(card_scale), "effects": shadow}])
track("card-pic", 5, [
    {"id": "card-pic-1", "type": "image", "group": "card", "start": POP, "end": MOVIE,
     "source": "stills/session-01.png", "x": CARD_X, "y": CARD_Y, "origin": "center",
     "width": CW, "height": CH, "fit": "cover", "scale": scale_kf(card_scale[:4]),
     "effects": [{"name": "mask", "shape": "rect", "radius": 8}]},
    {"id": "card-pic-2", "type": "image", "group": "card", "start": MOVIE, "end": DUR,
     "source": "stills/session-02.png", "x": CARD_X, "y": CARD_Y, "origin": "center",
     "width": CW, "height": CH, "fit": "cover", "scale": scale_kf(card_scale[4:]),
     "effects": [{"name": "mask", "shape": "rect", "radius": 8}]},
])

# ---------- owl ----------------------------------------------------------------------
BASE = 10
for i, name in enumerate(["torso", "forearm_left", "forearm_right", "upper_arm_left",
                          "upper_arm_right", "head"]):
    track("owl-" + name.replace("_", "-"), BASE + i,
          [part_el("owl-" + name.replace("_", "-"), name, name, 0, DUR)])

# mouths from visemes, quantised to frames
vis = {}
for ms, vid in vo["visemes"]:
    vis[ms] = vid  # a later duplicate time wins


def mouth_at(t):
    lt = t - VOICE_AT
    cur = None
    for ms in sorted(vis):
        if ms <= lt:
            cur = rig["visemes"][str(vis[ms])]
        else:
            break
    return cur


frame_mouth = [mouth_at(ft(n) + 15) for n in range(NF)]  # half a frame of lead
runs = []
for n, m in enumerate(frame_mouth):
    if runs and runs[-1][2] == m:
        runs[-1][1] = n + 1
    else:
        runs.append([n, n + 1, m])
mouth_els = []
for k, (a, b, m) in enumerate(runs):
    if m is None:
        continue
    mouth_els.append(part_el(f"mouth-{k:03d}-{m}", "mouth_" + m, "head", ft(a), ft(b)))
track("owl-mouth", BASE + 6, mouth_els)

BLINKS = [900, 2550, 4950, 7550]
track("owl-blink", BASE + 7, [part_el(f"blink-{i+1}", "eyes_closed", "head", t, t + 100)
                              for i, t in enumerate(BLINKS)])

# ---------- lockup ----------------------------------------------------------------------
LW = 640
LH = round(LW * 623 / 2694)
LAND = 6700
track("lockup", 30, [{"id": "lockup", "type": "image", "start": LAND, "end": DUR,
                      "source": "brand/lockup.png",
                      "x": HOME_X,
                      "y": [{"t": LAND, "v": -LH}, {"t": LAND + 260, "v": 160, "ease": "ease-in"},
                            {"t": LAND + 360, "v": 140, "ease": "ease-out"},
                            {"t": LAND + 470, "v": 150, "ease": "ease-in-out"}],
                      "origin": "center", "width": LW, "height": LH, "fit": "contain",
                      "rotation": [{"t": LAND, "v": -6}, {"t": LAND + 260, "v": 2, "ease": "ease-in"},
                                   {"t": LAND + 470, "v": 0, "ease": "ease-in-out"}]}])

# ---------- sound -------------------------------------------------------------------------
track("voice", 0, [{"id": "voice", "type": "audio", "start": VOICE_AT, "end": VOICE_AT + 6000,
                    "source": "character/voice/line-1.wav", "source_start": 0, "source_end": 6000}])
track("music", 0, [{"id": "music", "type": "audio", "start": 0, "end": DUR,
                    "source": "music/bed-120bpm.wav", "source_start": 0, "source_end": DUR,
                    "volume": [{"t": 0, "v": 0.22}, {"t": 900, "v": 0.09, "ease": "ease-in-out"},
                               {"t": 6800, "v": 0.09, "ease": "linear"},
                               {"t": 7100, "v": 0.2, "ease": "ease-in-out"},
                               {"t": 7500, "v": 0.2, "ease": "linear"},
                               {"t": 8000, "v": 0.0, "ease": "ease-in"}]}])

proj = {"frame": {"width": 1920, "height": 1080}, "fps": FPS, "background": "#F5F0E6",
        "duration": DUR, "output": "deliverable.mp4", "tracks": tracks}
json.dump(proj, open("hoot.json", "w"), indent=2)
print("mouth runs:", [(ft(a), ft(b), m) for a, b, m in runs])
