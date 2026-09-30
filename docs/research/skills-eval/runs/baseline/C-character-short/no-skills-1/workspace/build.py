#!/usr/bin/env python3
"""Generate hoot.json: bakes the owl rig's forward kinematics into per-frame keyframes."""
import json, math

FPS = 30
DUR = 8000
NF = DUR * FPS // 1000
S = 0.62                      # owl scale
FLOOR_Y = 985                 # feet (torso pivot) on the study floor
HOME_X = 620
VO = 1000                     # voice starts here

rig = json.load(open("character/rig.json"))
voice = json.load(open("character/voice/line-1.json"))
P = {k: v["pivot"] for k, v in rig["parts"].items()}
DIM = {k: (v["width"], v["height"]) for k, v in rig["parts"].items()}

def ft(n):                    # frame n -> integer ms, never after the true instant
    return (n * 1000) // FPS

def frame_of(ms):
    return round(ms * FPS / 1000)

# ---------- animation curves ----------
def smooth(x):
    x = min(1.0, max(0.0, x))
    return x * x * (3 - 2 * x)

def track(keys, t):
    """keys: [(t, v)], smoothstep between neighbours, held at the ends."""
    if t <= keys[0][0]:
        return keys[0][1]
    for (t0, v0), (t1, v1) in zip(keys, keys[1:]):
        if t < t1:
            return v0 + (v1 - v0) * smooth((t - t0) / (t1 - t0))
    return keys[-1][1]

def back_out(x, k=1.9):
    x = min(1.0, max(0.0, x))
    x -= 1
    return 1 + (k + 1) * x ** 3 + k * x ** 2

SIDE_L, SIDE_R = -36.0, 36.0          # arms hanging at the sides
HANG = 14.0                           # the forearm hangs a little further when the arm is down
WAVE_UA = 45.0                        # left upper arm raised for the wave
POINT_UA, POINT_FA = -32.0, -2.0      # right arm pointing at the card
CHEER_UA_L, CHEER_FA_L = 54.0, 26.0
CHEER_UA_R, CHEER_FA_R = -54.0, -26.0

T_WAVE_UP, T_WAVE_DOWN0, T_WAVE_DOWN1 = VO + 0, VO + 1150, VO + 1420
T_CARD = VO + 2700 - 60               # "as"
T_POINT0, T_POINT1 = T_CARD - 80, T_CARD + 200
T_MOVIE = VO + 5200
T_CHEER = 6850
T_LOCK = 6850

def pose(t):
    # --- hop in: two hops from off-screen left, land at 900 ms ---
    hops = [(0, 420, -380, 150, 170), (420, 880, 150, HOME_X, 120)]
    x, y, lean = HOME_X, FLOOR_Y, 0.0
    for a, b, x0, x1, h in hops:
        if a <= t < b:
            u = (t - a) / (b - a)
            x = x0 + (x1 - x0) * u
            y = FLOOR_Y - h * 4 * u * (1 - u)
            lean = 9 * math.sin(math.pi * u)
    if t < 0:
        x = -380
    # landing settle: little rock back and forth after the landing
    if 880 <= t < 1300:
        u = (t - 880) / 420
        lean = -4 * math.sin(math.pi * u * 2) * (1 - u)
    # gentle idle sway once standing, breathing
    if t >= 1300:
        lean = 1.2 * math.sin((t - 1300) / 1000 * 2 * math.pi * 0.45)

    # --- head: bob while talking ---
    talk = 1.0 if VO <= t < VO + 5700 else 0.0
    head = -5 * math.sin(math.pi * min(1, max(0, (t - 0) / 880))) if t < 880 else 0.0
    head += talk * 3.0 * math.sin((t - VO) / 1000 * 2 * math.pi * 0.9)
    head += track([(T_CARD - 100, 0), (T_CARD + 150, 5), (T_MOVIE - 400, 5), (T_MOVIE - 100, 0)], t)
    head += track([(T_CHEER - 50, 0), (T_CHEER + 200, -4), (7700, -4), (8000, -2)], t)

    # --- left arm: sides -> wave -> sides -> cheer ---
    ual = track([(0, SIDE_L), (T_WAVE_UP - 120, SIDE_L), (T_WAVE_UP + 150, WAVE_UA),
                 (T_WAVE_DOWN0, WAVE_UA), (T_WAVE_DOWN1, SIDE_L),
                 (T_CHEER, SIDE_L), (T_CHEER + 220, CHEER_UA_L)], t)
    fal = 0.0
    if T_WAVE_UP - 120 <= t < T_WAVE_DOWN1:
        env = track([(T_WAVE_UP - 120, 0), (T_WAVE_UP + 150, 1), (T_WAVE_DOWN0, 1), (T_WAVE_DOWN1, 0)], t)
        swing = 34 + 24 * math.sin((t - (T_WAVE_UP + 150)) / 1000 * 2 * math.pi * 3.0)
        fal = env * swing
    # --- right arm: sides -> point at card -> sides -> cheer ---
    uar = track([(0, SIDE_R), (T_POINT0, SIDE_R), (T_POINT1, POINT_UA), (T_MOVIE + 350, POINT_UA),
                 (T_MOVIE + 650, SIDE_R), (T_CHEER, SIDE_R), (T_CHEER + 220, CHEER_UA_R)], t)
    far = track([(0, 0), (T_POINT0, 0), (T_POINT1, POINT_FA), (T_MOVIE + 350, POINT_FA),
                 (T_MOVIE + 650, 0), (T_CHEER, 0), (T_CHEER + 220, CHEER_FA_R)], t)
    # re-emphasise the point on "movie"
    uar += -6 * math.sin(math.pi * min(1, max(0, (t - T_MOVIE) / 300))) if T_MOVIE <= t < T_MOVIE + 300 else 0
    fal += track([(T_CHEER, 0), (T_CHEER + 220, CHEER_FA_L)], t)
    # cheer: forearms pump twice
    if t >= T_CHEER + 220:
        pump = 10 * math.sin((t - T_CHEER - 220) / 1000 * 2 * math.pi * 2.2) * max(0, 1 - (t - T_CHEER - 220) / 900)
        fal += pump
        far -= pump
        ual += pump * 0.3
        uar -= pump * 0.3
    fal -= HANG * min(1.0, max(0.0, ual / SIDE_L))
    far += HANG * min(1.0, max(0.0, uar / SIDE_R))
    return dict(x=x, y=y, torso=lean, head=head, ual=ual, fal=fal, uar=uar, far=far)

def rot(v, deg):
    a = math.radians(deg)
    c, s = math.cos(a), math.sin(a)
    return (v[0] * c - v[1] * s, v[0] * s + v[1] * c)

def rel(child, parent):
    return (P[child][0] - P[parent][0], P[child][1] - P[parent][1])

def solve(t):
    p = pose(t)
    W = {}
    W["torso"] = ((p["x"], p["y"]), p["torso"])
    def child(name, parent, local):
        (px, py), pr = W[parent]
        dx, dy = rot(rel(name, parent), pr)
        W[name] = ((px + S * dx, py + S * dy), pr + local)
    child("head", "torso", p["head"])
    child("upper_arm_left", "torso", p["ual"])
    child("upper_arm_right", "torso", p["uar"])
    child("forearm_left", "upper_arm_left", p["fal"])
    child("forearm_right", "upper_arm_right", p["far"])
    return W

BAKED = [solve(n * 1000 / FPS) for n in range(NF + 1)]
POSES = [pose(n * 1000 / FPS) for n in range(NF + 1)]

def thin(keys):
    """Drop a keyframe that sits between two holding the same value: it adds no motion."""
    out = []
    for i, k in enumerate(keys):
        if 0 < i < len(keys) - 1 and keys[i - 1]["v"] == k["v"] == keys[i + 1]["v"]:
            continue
        out.append(k)
    out = [dict(k) for k in out]
    out[0].pop("ease", None)
    for k in out[1:]:
        k["ease"] = "linear"
    return out

def last_frame(end):
    n = frame_of(end)
    while ft(n) >= end:
        n -= 1
    return n

def keys_for(part, n0, n1):
    xs, ys, rs = [], [], []
    for n in range(max(0, n0), min(NF - 1, n1) + 1):
        (x, y), r = BAKED[n][part]
        t = ft(n)
        xs.append({"t": t, "v": round(x)})
        ys.append({"t": t, "v": round(y)})
        rs.append({"t": t, "v": round(r, 2)})
    return thin(xs), thin(ys), thin(rs)

def rig_el(eid, part, src, start, end, pad=0, effects=None):
    xs, ys, rs = keys_for(part, frame_of(start) - pad, last_frame(end) + pad)
    w, h = DIM[src]
    el = {"id": eid, "type": "image", "group": "owl", "start": start, "end": end,
          "source": "character/" + rig["parts"][src]["file"],
          "x": xs, "y": ys, "origin": "center", "width": w, "height": h, "fit": "literal",
          "scale": [S, S], "rotation": rs}
    if effects:
        el["effects"] = effects
    return el

def intervals(pred):
    """[start, end) ms spans of the frames where pred(pose) holds."""
    spans, a = [], None
    for n in range(NF):
        ok = pred(POSES[n])
        if ok and a is None:
            a = n
        if not ok and a is not None:
            spans.append((ft(a), ft(n))); a = None
    if a is not None:
        spans.append((ft(a), DUR))
    return spans

tracks = []
def add_track(name, layer, els):
    tracks.append({"name": name, "layer": layer, "elements": els})

add_track("set", 0, [{"id": "study", "type": "image", "start": 0, "end": DUR,
                      "source": "character/study.png", "x": 0, "y": 0, "origin": "top-left", "width": 1920, "height": 1080,
                      "fit": "literal"}])

# ---------- card ----------
CX, CY = 1340, 500
CW, CH, BORDER = 720, 405, 16
pop = [{"t": T_CARD, "v": [0.3, 0.3]}, {"t": T_CARD + 380, "v": [1.0, 1.0], "ease": [0.3, 1.6, 0.55, 1.0]}]
pulse = [{"t": T_MOVIE, "v": [1.0, 1.0]}, {"t": T_MOVIE + 120, "v": [1.04, 1.04], "ease": "ease-out"},
         {"t": T_MOVIE + 320, "v": [1.0, 1.0], "ease": "ease-in-out"}]
card_scale = pop + [{**k, "ease": k.get("ease", "linear")} for k in pulse]
card_op = [{"t": T_CARD, "v": 0.0}, {"t": T_CARD + 100, "v": 1.0, "ease": "linear"}]
add_track("card-frame", 10, [{"id": "card-frame", "type": "rect", "group": "card", "start": T_CARD, "end": DUR,
    "x": CX, "y": CY, "origin": "center", "width": CW + 2 * BORDER, "height": CH + 2 * BORDER,
    "fill": "#F5F0E6", "stroke": "#101418", "stroke_width": 6, "radius": 22,
    "scale": card_scale, "opacity": card_op,
    "effects": [{"name": "shadow", "dx": 0, "dy": 14, "radius": 22, "color": "#101418", "opacity": 0.3}]}])
add_track("card-image", 11, [
    {"id": "card-still-1", "type": "image", "group": "card", "start": T_CARD, "end": T_MOVIE,
     "source": "stills/session-01.png", "x": CX, "y": CY, "origin": "center", "width": CW, "height": CH,
     "fit": "literal", "scale": pop, "opacity": card_op},
    {"id": "card-still-2", "type": "image", "group": "card", "start": T_MOVIE, "end": DUR,
     "source": "stills/session-02.png", "x": CX, "y": CY, "origin": "center", "width": CW, "height": CH,
     "fit": "literal", "scale": pulse}])

# ---------- owl ----------
L = 20
def mask(shape, x, y, w, h):
    return [{"name": "mask", "shape": shape, "x": x, "y": y, "width": w, "height": h}]

# The torso drawing keeps a stub of each sleeve beside the shoulder, which the upper arm
# hides at rest but which hangs loose once the arm is raised. So the torso is drawn as
# masked copies of itself, and each stub only while its arm is down over it.
STUB = 8.0                              # degrees past rest at which a raised arm uncovers its stub
add_track("owl-torso-lower", L, [rig_el("owl-torso-lower", "torso", "torso", 0, DUR,
                                        effects=mask("rect", 0, 214, 480, 1194))])
add_track("owl-torso-body", L + 1, [rig_el("owl-torso-body", "torso", "torso", 0, DUR,
                                           effects=mask("ellipse", 4, 24, 472, 752))])
add_track("owl-torso-core", L + 2, [rig_el("owl-torso-core", "torso", "torso", 0, DUR,
                                           effects=mask("rect", 56, 0, 368, 226))])
# The shoulders down to the armpit, outline and all, stay on: only the sleeve's lower
# edge below them is the loose part.
add_track("owl-torso-shoulders", L + 3, [rig_el("owl-torso-shoulders", "torso", "torso", 0, DUR,
                                               effects=mask("rect", 0, 30, 480, 130))])
add_track("owl-torso-stub-left", L + 6, [
    rig_el(f"owl-stub-left-{i+1}", "torso", "torso", a, b, effects=mask("rect", 0, 150, 70, 62))
    for i, (a, b) in enumerate(intervals(lambda p: p["ual"] <= STUB))])
add_track("owl-torso-stub-right", L + 7, [
    rig_el(f"owl-stub-right-{i+1}", "torso", "torso", a, b, effects=mask("rect", 410, 150, 70, 62))
    for i, (a, b) in enumerate(intervals(lambda p: p["uar"] >= -STUB))])
for i, part in enumerate(["forearm_left", "forearm_right", "upper_arm_left", "upper_arm_right", "head"]):
    name = "owl-" + part.replace("_", "-")
    add_track(name, L + 8 + i, [rig_el(name, part, part, 0, DUR)])

# mouths from visemes
vis = {}
for ms, vid in voice["visemes"]:
    vis[ms] = vid                       # a repeated time: the later one wins
segs = []
items = sorted(vis.items())
for i, (ms, vid) in enumerate(items):
    end = items[i + 1][0] if i + 1 < len(items) else voice["duration_ms"]
    mouth = rig["visemes"][str(vid)]
    a, b = frame_of(VO + ms), frame_of(VO + end)
    if b <= a:
        continue
    if segs and segs[-1][2] == mouth and segs[-1][1] == a:
        segs[-1][1] = b
    else:
        segs.append([a, b, mouth])
mouth_els = []
for k, (a, b, m) in enumerate(segs):
    if m is None:
        continue
    mouth_els.append(rig_el(f"mouth-{k:02d}-{m}", "head", "mouth_" + m, ft(a), ft(b), pad=1))
add_track("owl-mouth", L + 13, mouth_els)

blinks = [1350, 2950, 4900, 6300, 7550]
add_track("owl-eyes", L + 14, [rig_el(f"blink-{i+1}", "head", "eyes_closed", ft(frame_of(b)), ft(frame_of(b) + 4), pad=1)
                               for i, b in enumerate(blinks)])

# ---------- lockup ----------
LW = 740
LH = round(LW * 623 / 2694)
add_track("lockup", 40, [{"id": "lockup", "type": "image", "start": T_LOCK, "end": DUR,
    "source": "brand/lockup.png", "x": [{"t": T_LOCK, "v": HOME_X + 40}],
    "y": [{"t": T_LOCK, "v": -120}, {"t": T_LOCK + 420, "v": 150, "ease": [0.3, 1.45, 0.6, 1.0]}],
    "origin": "center", "width": LW, "height": LH, "fit": "literal",
    "rotation": [{"t": T_LOCK, "v": -6.0}, {"t": T_LOCK + 420, "v": 0.0, "ease": "ease-out"}]}])

# ---------- sound ----------
tracks.append({"name": "voice", "layer": 0, "elements": [{"id": "line-1", "type": "audio", "start": VO,
    "end": VO + 6048, "source": "character/voice/line-1.wav", "source_start": 0, "source_end": 6048}]})
tracks.append({"name": "music", "layer": 0, "elements": [{"id": "bed", "type": "audio", "start": 0, "end": DUR,
    "source": "music/bed-120bpm.wav", "source_start": 0, "source_end": DUR,
    "volume": [{"t": 0, "v": 0.0}, {"t": 400, "v": 0.22, "ease": "linear"}, {"t": 900, "v": 0.1, "ease": "ease-in-out"},
               {"t": 7100, "v": 0.1, "ease": "linear"}, {"t": 7300, "v": 0.2, "ease": "ease-out"},
               {"t": 7950, "v": 0.0, "ease": "ease-in"}]}]})

proj = {"frame": {"width": 1920, "height": 1080}, "fps": FPS, "background": "#F5F0E6", "duration": DUR,
        "output": "deliverable.mp4", "tracks": tracks}
json.dump(proj, open("hoot.json", "w"))
print("mouth segments:", [(ft(a), ft(b), m) for a, b, m in segs])
