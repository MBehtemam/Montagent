#!/usr/bin/env python3
"""Generate hoot.json: Hoot introduces Montagent (Brief C).

The rig is hierarchical but Montagent transforms are flat, so every part's world
transform is computed here (parent chain composed) and baked as one keyframe per
video frame. Each part is drawn at its native canvas size, origin `center` (its
pivot), scaled by S, so it turns about its own joint.
"""
import json
import math

FPS = 30
DUR = 8000
VO_AT = 1000
S = 0.64                      # drawing px -> frame px
HOME_X, FLOOR_Y = 700, 1010   # where the feet (torso pivot) land
CARD_X, CARD_Y = 1270, 470

rig = json.load(open("character/rig.json"))
parts = rig["parts"]
voice = json.load(open("character/voice/line-1.json"))

FRAMES = [round(n * 1000 / FPS) for n in range(DUR * FPS // 1000 + 1)]


def frame_floor(n):
    return math.floor(n * 1000 / FPS)


# ---------------------------------------------------------------- curves
def smooth(u):
    u = min(1.0, max(0.0, u))
    return u * u * (3 - 2 * u)


def ease_out_back(u, k=1.8):
    u = min(1.0, max(0.0, u))
    u -= 1
    return 1 + (k + 1) * u ** 3 + k * u ** 2


def track(keys, t):
    """keys: [(t, v)], smoothstep between neighbours, held outside."""
    if t <= keys[0][0]:
        return keys[0][1]
    for (t0, v0), (t1, v1) in zip(keys, keys[1:]):
        if t <= t1:
            return v0 + (v1 - v0) * smooth((t - t0) / (t1 - t0))
    return keys[-1][1]


# ---------------------------------------------------------------- poses
# local rotations (degrees clockwise) relative to the drawn rest pose
SIDES = dict(ual=-45, fal=-12, uar=45, far=12)
WAVE = dict(ual=30, fal=40)
# point the right arm from the shoulder at the card centre
sh = (HOME_X + (666 - 512) * S, FLOOR_Y - (1322 - 736) * S)
aim = math.degrees(math.atan2(CARD_Y - sh[1], CARD_X - sh[0]))
POINT = dict(uar=aim - 25.8, far=-4)
CHEER = dict(ual=55, fal=15, uar=-55, far=-15)

HOPS = [(0, 300, -380, -20, 110), (300, 600, -20, 360, 90), (600, 900, 360, HOME_X, 60)]


def pose(t):
    # --- root: hop in, then stand; a small hop in the cheer
    x, lift, lean = HOME_X, 0.0, 0.0
    for t0, t1, x0, x1, h in HOPS:
        if t0 <= t < t1:
            u = (t - t0) / (t1 - t0)
            x = x0 + (x1 - x0) * smooth(u)
            lift = h * 4 * u * (1 - u)
            lean = 7 * math.sin(math.pi * u)
    if t < 900:
        flap = 18 * math.sin(2 * math.pi * t / 300)
    else:
        flap = 0.0
    # settle after the landing: rock back and recover
    if 900 <= t < 1150:
        lean = -4 * math.sin(math.pi * (t - 900) / 250)
    # cheer hop
    if 7000 <= t < 7300:
        u = (t - 7000) / 300
        lift = 40 * 4 * u * (1 - u)
    sway = 1.2 * math.sin(2 * math.pi * (t - 1000) / 2600) if t >= 1150 else 0.0
    torso = lean + sway

    # --- arms: blend between poses
    ual = track([(0, SIDES["ual"]), (850, SIDES["ual"]), (1080, WAVE["ual"]),
                 (1980, WAVE["ual"]), (2300, SIDES["ual"]), (6800, SIDES["ual"]),
                 (7080, CHEER["ual"])], t)
    fal = track([(0, SIDES["fal"]), (850, SIDES["fal"]), (1080, WAVE["fal"]),
                 (1980, WAVE["fal"]), (2300, SIDES["fal"]), (6800, SIDES["fal"]),
                 (7080, CHEER["fal"])], t)
    # the wave: forearm swings from the elbow, 2.5 cycles over "Hi, I'm Hoot"
    if 1080 <= t < 1980:
        fal += 20 * math.sin(2 * math.pi * (t - 1080) / 360)
    uar = track([(0, SIDES["uar"]), (3550, SIDES["uar"]), (3850, POINT["uar"]),
                 (6600, POINT["uar"]), (7080, CHEER["uar"])], t)
    far = track([(0, SIDES["far"]), (3550, SIDES["far"]), (3850, POINT["far"]),
                 (6600, POINT["far"]), (7080, CHEER["far"])], t)
    # presenting beat on the swap to the movie
    if 6150 <= t < 6450:
        far += -8 * math.sin(math.pi * (t - 6150) / 300)
    if t < 900:
        ual += flap
        uar -= flap
    # cheer pumps
    if t >= 7080:
        pump = 8 * math.sin(2 * math.pi * (t - 7080) / 500)
        ual += pump
        uar -= pump

    # --- head: follows the talk, turns to the card
    head = track([(0, 0), (1000, 0), (1250, -5), (1950, -5), (2300, 0), (3600, 0),
                  (3900, 6), (6500, 6), (6900, 0)], t)
    if t >= 1000:
        head += 2.2 * math.sin(2 * math.pi * (t - 1000) / 1500)
    if t < 900:
        head += -3 * math.sin(2 * math.pi * t / 300)

    return dict(root=(x, FLOOR_Y - lift), torso=torso, head=head,
                upper_arm_left=ual, forearm_left=fal,
                upper_arm_right=uar, forearm_right=far)


def world(t):
    """Per part: frame-space centre (the pivot) and world rotation."""
    p = pose(t)
    out = {}

    def xf(name, local):
        par = parts[name]["parent"]
        piv = parts[name]["pivot"]
        if par is None:
            out[name] = (p["root"], local)
            return
        (px, py), prot = out[par]
        ppiv = parts[par]["pivot"]
        dx, dy = (piv[0] - ppiv[0]) * S, (piv[1] - ppiv[1]) * S
        a = math.radians(prot)
        cx = px + dx * math.cos(a) - dy * math.sin(a)
        cy = py + dx * math.sin(a) + dy * math.cos(a)
        out[name] = ((cx, cy), prot + local)

    xf("torso", p["torso"])
    xf("head", p["head"])
    for n in ("upper_arm_left", "upper_arm_right", "forearm_left", "forearm_right"):
        xf(n, p[n])
    # the torso's own centre is its canvas centre, not its pivot, only when
    # rotation is 0 — but every part's canvas centre IS its pivot, so no fix-up.
    return out


BAKED = {t: world(t) for t in FRAMES}


def keys(part, t0=0, t1=DUR):
    xs, ys, rs = [], [], []
    ts = [t for t in FRAMES if t0 - 40 <= t <= t1 + 40]
    for i, t in enumerate(ts):
        (cx, cy), r = BAKED[t][part]
        e = {} if i == 0 else {"ease": "linear"}
        xs.append({"t": t, "v": round(cx), **e})
        ys.append({"t": t, "v": round(cy), **e})
        rs.append({"t": t, "v": round(r, 3), **e})
    return xs, ys, rs


def part_el(eid, part, file, start, end, rig_part=None):
    rp = parts[rig_part or part]
    xs, ys, rs = keys(rig_part or part, start, end)
    return {"id": eid, "type": "image", "group": "hoot", "start": start, "end": end,
            "source": "character/" + file, "x": xs, "y": ys, "origin": "center",
            "width": rp["width"], "height": rp["height"], "fit": "contain",
            "scale": [S, S], "rotation": rs}


tracks = []


def add(name, layer, elements):
    tracks.append({"name": name, "layer": layer, "elements": elements})


add("set", 0, [{"id": "study", "type": "image", "start": 0, "end": DUR,
                "source": "character/study.png", "x": 0, "y": 0, "origin": "top-left", "width": 1920,
                "height": 1080, "fit": "cover"}])

# ---------------------------------------------------------------- card
CARD_IN = 3700
SWAP = VO_AT + next(w["start"] for w in voice["words"] if w["word"] == "movie")
pop = [(CARD_IN, 0.0), (CARD_IN + 170, 1.12), (CARD_IN + 300, 0.96), (CARD_IN + 420, 1.0),
       (SWAP, 1.0), (SWAP + 110, 1.05), (SWAP + 260, 1.0)]
pop_keys = []
for i, (t, v) in enumerate(pop):
    k = {"t": t, "v": [v, v]}
    if i:
        k["ease"] = "ease-out" if v > pop[i - 1][1] else "ease-in-out"
    pop_keys.append(k)
card_shadow = [{"name": "shadow", "dx": 0, "dy": 10, "radius": 18, "color": "#101418", "opacity": 0.28}]
add("card", 10, [{"id": "card-frame", "type": "rect", "group": "card", "start": CARD_IN, "end": DUR,
                  "x": CARD_X, "y": CARD_Y, "origin": "center", "width": 600, "height": 356,
                  "fill": "#101418", "radius": 22, "scale": pop_keys, "effects": card_shadow}])
mask = [{"name": "mask", "shape": "rect", "radius": 12}]
add("card-picture", 11, [
    {"id": "card-session-01", "type": "image", "group": "card", "start": CARD_IN, "end": SWAP,
     "source": "stills/session-01.png", "x": CARD_X, "y": CARD_Y, "origin": "center",
     "width": 1920, "height": 1080, "fit": "contain",
     "scale": [{"t": k["t"], "v": [k["v"][0] * 0.29, k["v"][1] * 0.29], **({"ease": k["ease"]} if "ease" in k else {})} for k in pop_keys],
     "effects": [{"name": "mask", "shape": "rect", "radius": 40}]},
    {"id": "card-session-02", "type": "image", "group": "card", "start": SWAP, "end": DUR,
     "source": "stills/session-02.png", "x": CARD_X, "y": CARD_Y, "origin": "center",
     "width": 960, "height": 540, "fit": "contain",
     "scale": [{"t": k["t"], "v": [k["v"][0] * 0.58, k["v"][1] * 0.58], **({"ease": k["ease"]} if "ease" in k else {})} for k in pop_keys],
     "effects": [{"name": "mask", "shape": "rect", "radius": 20}]},
])

# ---------------------------------------------------------------- owl
LAYER = {n: 20 + i for i, n in enumerate(rig["draw_order"])}
for name in ("torso", "forearm_left", "forearm_right", "upper_arm_left", "upper_arm_right", "head"):
    add("owl-" + name, LAYER[name],
        [part_el("owl-" + name, name, parts[name]["file"], 0, DUR)])

# mouths: the viseme active at each frame, merged into runs on frame boundaries
vis = sorted(voice["visemes"], key=lambda v: v[0])


def mouth_at(t):
    lt = t - VO_AT
    cur = None
    for ms, vid in vis:
        if ms <= lt:
            cur = rig["visemes"][str(vid)]
    return cur


runs = []
nframes = DUR * FPS // 1000
for n in range(nframes):
    m = mouth_at(n * 1000 / FPS)
    if runs and runs[-1][0] == m and runs[-1][2] == n:
        runs[-1][2] = n + 1
    else:
        runs.append([m, n, n + 1])
mouth_els = []
for i, (m, a, b) in enumerate(runs):
    if m is None:
        continue
    start, end = frame_floor(a), frame_floor(b)
    mouth_els.append(part_el(f"mouth-{i:02d}-{m}", "mouth_" + m, f"parts/mouth_{m}.png",
                             start, end, rig_part="head"))
add("owl-mouth", LAYER["mouth_open"], mouth_els)

# blinks: 3 frames each
BLINK_FRAMES = [72, 128, 176, 229]
add("owl-blink", LAYER["eyes_closed"],
    [part_el(f"blink-{i+1}", "eyes_closed", "parts/eyes_closed.png",
             frame_floor(n), frame_floor(n + 3), rig_part="head")
     for i, n in enumerate(BLINK_FRAMES)])

# ---------------------------------------------------------------- lockup
L_IN = 6950
add("lockup", 40, [{"id": "lockup", "type": "image", "group": "brand", "start": L_IN, "end": DUR,
                    "source": "brand/lockup.png",
                    "x": HOME_X, "y": [{"t": L_IN, "v": -120}, {"t": L_IN + 300, "v": 162, "ease": "ease-in"},
                                       {"t": L_IN + 420, "v": 140, "ease": "ease-out"},
                                       {"t": L_IN + 560, "v": 150, "ease": "ease-in-out"}],
                    "origin": "center", "width": 2694, "height": 623, "fit": "contain",
                    "scale": [{"t": L_IN + 280, "v": [0.26, 0.26]}, {"t": L_IN + 340, "v": [0.275, 0.24], "ease": "ease-out"},
                              {"t": L_IN + 480, "v": [0.26, 0.26], "ease": "ease-in-out"}],
                    "opacity": [{"t": L_IN, "v": 0.0}, {"t": L_IN + 120, "v": 1.0, "ease": "linear"}]}])

# ---------------------------------------------------------------- sound
add("voice", 0, [{"id": "vo-line-1", "type": "audio", "start": VO_AT, "end": VO_AT + 6048,
                  "source": "character/voice/line-1.wav", "source_start": 0, "source_end": 6048}])
add("music", 0, [{"id": "bed", "type": "audio", "start": 0, "end": DUR, "source": "music/bed-120bpm.wav",
                  "source_start": 0, "source_end": DUR,
                  "volume": [{"t": 0, "v": 0.14}, {"t": 7000, "v": 0.14, "ease": "linear"},
                             {"t": DUR, "v": 0.0, "ease": "ease-in"}]}])

doc = {"frame": {"width": 1920, "height": 1080}, "fps": FPS, "background": "#F5F0E6",
       "duration": DUR, "output": "deliverable.mp4", "tracks": tracks}
json.dump(doc, open("hoot.json", "w"))
print("mouth runs", len(mouth_els), "swap", SWAP, "aim", round(aim, 1))
