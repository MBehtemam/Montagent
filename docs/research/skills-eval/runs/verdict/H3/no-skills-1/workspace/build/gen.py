"""Generate hoot.json: Hoot the owl fixes a late caption (brief H3).

The rig is posed with forward kinematics here, because Montagent transforms are flat:
every part's world x/y/rotation is computed per frame and written as keyframes.
"""
import json, math, os

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
RIG = json.load(open(os.path.join(ROOT, 'character/rig.json')))
LINE = json.load(open(os.path.join(ROOT, 'character/voice/line-3.json')))
P = RIG['parts']

FPS = 30
DUR = 8000
NF = DUR * FPS // 1000
TIMES = [round(n * 1000 / FPS) for n in range(NF)]
VO = 1000  # the line starts at 1 s

def W(word_index, edge='start'):
    return LINE['words'][word_index][edge] + VO

# ---------------------------------------------------------------- layout
S = 0.6                     # owl scale
FEET = (760, 995)           # torso pivot (feet) in frame space
LAP_S = 0.625               # laptop scale
LAP = (860, 530)            # laptop top-left
SCREEN = [935, 556, 292, 169]   # laptop screen x 120-588, y 41-313, scaled, rounded inward
CRATE = (842, 872, 476, 114)    # x, y, w, h
TAG = (1200, 440)

# ---------------------------------------------------------------- easing
def smooth(u): return u * u * (3 - 2 * u)
def ease_out(u): return 1 - (1 - u) ** 2
def ease_in(u): return u * u
def lin(u): return u
def ease_out_back(u, k=1.9):
    u -= 1
    return 1 + (k + 1) * u ** 3 + k * u ** 2

def track(keys):
    """keys: [(t, value, ease_fn)] -> function of t. value may be a tuple."""
    def f(t):
        if t <= keys[0][0]: return keys[0][1]
        for (t0, v0, _), (t1, v1, e) in zip(keys, keys[1:]):
            if t <= t1:
                u = e((t - t0) / (t1 - t0))
                if isinstance(v0, tuple):
                    return tuple(a + (b - a) * u for a, b in zip(v0, v1))
                return v0 + (v1 - v0) * u
        return keys[-1][1]
    return f

# ---------------------------------------------------------------- rig maths
def rot(v, deg):
    a = math.radians(deg); c, s = math.cos(a), math.sin(a)
    return (v[0] * c - v[1] * s, v[0] * s + v[1] * c)

def ang(v): return math.degrees(math.atan2(v[1], v[0]))
def sub(a, b): return (a[0] - b[0], a[1] - b[1])
def add(a, b): return (a[0] + b[0], a[1] + b[1])
def norm(d):
    while d > 180: d -= 360
    while d <= -180: d += 360
    return d

PIV = {n: tuple(p['pivot']) for n, p in P.items()}
HAND = {'left': (105, 860), 'right': (912, 860)}   # glove centres in the drawing
BEAK = (507, 560)

def ik(side, target, elbow_down=True):
    """Local (upper, fore) rotations putting the hand centre on target (torso-local drawing)."""
    sh, el, ha = PIV['upper_arm_' + side], PIV['forearm_' + side], HAND[side]
    l1 = math.dist(sh, el); l2 = math.dist(el, ha)
    d = math.dist(sh, target)
    d = max(abs(l1 - l2) + 1, min(l1 + l2 - 1, d))
    base = math.atan2(target[1] - sh[1], target[0] - sh[0])
    a = math.acos((l1 * l1 + d * d - l2 * l2) / (2 * l1 * d))
    best = None
    for sgn in (1, -1):
        b1 = base + sgn * a
        e = (sh[0] + l1 * math.cos(b1), sh[1] + l1 * math.sin(b1))
        b2 = math.atan2(target[1] - e[1], target[0] - e[0])
        ua = norm(math.degrees(b1) - ang(sub(el, sh)))
        fa = norm(math.degrees(b2) - ang(sub(ha, el)) - ua)
        key = e[1] if elbow_down else -e[1]
        if best is None or key > best[0]:
            best = (key, (ua, fa))
    return best[1]

def to_torso_local(world, torso_rot, torso_off):
    feet = add(FEET, torso_off)
    v = rot(sub(world, feet), -torso_rot)
    return add(PIV['torso'], (v[0] / S, v[1] / S))

def pose_world(pose):
    """pose: dict of local rotations + torso offset -> {part: (x, y, rot)}."""
    out = {}
    feet = add(FEET, pose['torso_off'])
    tr = pose['torso']
    def place(p_draw, parent_origin_draw, parent_world, parent_rot):
        v = rot(sub(p_draw, parent_origin_draw), parent_rot)
        return add(parent_world, (v[0] * S, v[1] * S))
    out['torso'] = (feet[0], feet[1], tr)
    for child in ('head', 'upper_arm_left', 'upper_arm_right'):
        w = place(PIV[child], PIV['torso'], feet, tr)
        out[child] = (w[0], w[1], tr + pose[child])
    for side in ('left', 'right'):
        ua = out['upper_arm_' + side]
        w = place(PIV['forearm_' + side], PIV['upper_arm_' + side], ua[:2], ua[2])
        out['forearm_' + side] = (w[0], w[1], ua[2] + pose['forearm_' + side])
    return out

# ---------------------------------------------------------------- poses
SIDES_L = (-57.0, -7.0)
SIDES_R = (57.0, 7.0)
UP_L = (52.0, 22.0)
UP_R = (-52.0, -22.0)

THINK_TILT = -9.0
chin = add(PIV['head'], rot(sub((450, 636), PIV['head']), THINK_TILT))
THINK_L = ik('left', chin)

LEAN = 9.0
REACH_TILT = 6.0
key_world = (LAP[0] + 220 * LAP_S, LAP[1] + 345 * LAP_S)  # hover just above the keys' left half
REACH_R = ik('right', to_torso_local(key_world, LEAN, (0, 0)))
TAP = 22.0

T_THINK_IN = (W(0) - 150, W(0) + 230)           # "Hmm."
T_THINK_OUT = (W(6, 'end') - 120, W(7) - 40)    # after "late", into "Let"
T_LET, T_CHECK_END = W(7), W(9, 'end')
T_FIXED = W(10)
T_RIGHT, T_BEAT_END = W(11), W(14, 'end')
T_TWO = W(4)

left_arm = track([
    (0, SIDES_L, lin),
    (T_THINK_IN[0], SIDES_L, lin),
    (T_THINK_IN[1], THINK_L, smooth),
    (T_THINK_OUT[0], THINK_L, lin),
    (T_THINK_OUT[1], SIDES_L, smooth),
    (T_FIXED - 140, (-40.0, -4.0), smooth),
    (T_FIXED + 90, UP_L, ease_out),
    (T_RIGHT - 20, UP_L, lin),
    (T_BEAT_END - 120, SIDES_L, smooth),
])
right_arm = track([
    (0, SIDES_R, lin),
    (T_THINK_OUT[0] + 60, SIDES_R, lin),
    (T_LET - 20, REACH_R, smooth),
    (T_CHECK_END - 40, REACH_R, lin),
    (T_FIXED - 140, (40.0, 4.0), smooth),
    (T_FIXED + 90, UP_R, ease_out),
    (T_RIGHT - 20, UP_R, lin),
    (T_BEAT_END - 120, SIDES_R, smooth),
])

TAPS = [T_LET + 120, T_LET + 390, T_LET + 640]
def tap(t):
    v = 0.0
    for t0 in TAPS:
        if t0 <= t < t0 + 90: v += TAP * ease_in((t - t0) / 90)
        elif t0 + 90 <= t < t0 + 230: v += TAP * (1 - smooth((t - t0 - 90) / 140))
    return v

head_tilt = track([
    (0, 0.0, lin),
    (T_THINK_IN[0] + 40, 0.0, lin),
    (T_THINK_IN[1], THINK_TILT, smooth),
    (W(3), THINK_TILT - 2.5, smooth),      # tilts a little further while it thinks
    (T_THINK_OUT[0], THINK_TILT - 2.5, lin),
    (T_THINK_OUT[1] + 60, REACH_TILT, smooth),
    (T_CHECK_END, REACH_TILT, lin),
    (T_FIXED - 60, 0.0, smooth),
    (T_FIXED + 520, 0.0, lin),
    (T_FIXED + 620, -3.0, ease_out),        # landing nod
    (T_FIXED + 780, 0.0, smooth),
    (T_BEAT_END, 0.0, lin),
    (T_BEAT_END + 300, 5.0, smooth),         # glances at the screen
])
torso_lean = track([
    (0, 0.0, lin),
    (T_THINK_IN[0], 0.0, lin),
    (T_THINK_IN[1], -2.0, smooth),
    (T_THINK_OUT[0], -2.0, lin),
    (T_THINK_OUT[1] + 40, LEAN, smooth),
    (T_CHECK_END - 40, LEAN, lin),
    (T_FIXED - 40, 0.0, smooth),
])
HOP_UP, HOP_PEAK, HOP_DOWN = T_FIXED - 10, T_FIXED + 230, T_FIXED + 460
hop = track([
    (0, 0.0, lin),
    (HOP_UP, 0.0, lin),
    (HOP_PEAK, -135.0, ease_out),
    (HOP_DOWN, 0.0, ease_in),
])

def pose_at(t):
    ph = 2 * math.pi * t / 2600
    sway = 0.7 * math.sin(ph)
    nod = 1.2 * math.sin(ph * 1.3 + 0.8)
    lean = torso_lean(t)
    tr = lean + sway
    la, ra = left_arm(t), right_arm(t)
    # the reaching arm is solved against the leaning torso, so cancel the sway there
    pose = {
        'torso': tr, 'torso_off': (0, hop(t)),
        'head': head_tilt(t) + nod,
        'upper_arm_left': la[0] - sway * 0.5, 'forearm_left': la[1],
        'upper_arm_right': ra[0] - sway, 'forearm_right': ra[1] + tap(t),
    }
    return pose

FR = [pose_world(pose_at(t)) for t in TIMES]

# ---------------------------------------------------------------- keyframe writing
def thin(samples, tol):
    """samples [(t, v)] -> keep a subset that linear interpolation reproduces within tol."""
    keep = [samples[0]]
    i = 0
    while i < len(samples) - 1:
        j = i + 1
        while j + 1 < len(samples):
            t0, v0 = samples[i]; t1, v1 = samples[j + 1]
            ok = True
            for k in range(i + 1, j + 1):
                tk, vk = samples[k]
                vi = v0 + (v1 - v0) * (tk - t0) / (t1 - t0)
                if abs(vi - vk) > tol: ok = False; break
            if not ok: break
            j += 1
        keep.append(samples[j]); i = j
    return keep

def kf(samples, tol, is_int):
    rnd = (lambda v: int(round(v))) if is_int else (lambda v: round(v, 2))
    s = [(t, rnd(v)) for t, v in samples]
    if all(v == s[0][1] for _, v in s): return s[0][1]
    k = thin(s, tol)
    out = []
    for n, (t, v) in enumerate(k):
        r = {'t': t, 'v': v}
        if n: r['ease'] = 'step' if v == k[n - 1][1] else 'linear'
        out.append(r)
    return out

def frames_in(a, b):
    """indices of frames whose time lies in [a-1 frame, b+1 frame]."""
    return [i for i, t in enumerate(TIMES) if a - 34 <= t <= b + 34]

def part_el(eid, part, start, end, layer=None, src_part=None):
    idx = frames_in(start, end)
    xs = [(TIMES[i], FR[i][part][0]) for i in idx]
    ys = [(TIMES[i], FR[i][part][1]) for i in idx]
    rs = [(TIMES[i], FR[i][part][2]) for i in idx]
    p = P[src_part or part]
    el = {'id': eid, 'type': 'image', 'group': 'owl', 'start': start, 'end': end}
    if layer is not None: el['layer'] = layer
    el.update({'source': 'character/' + p['file'], 'x': kf(xs, 0.5, True), 'y': kf(ys, 0.5, True),
               'origin': 'center', 'width': p['width'], 'height': p['height'], 'fit': 'contain',
               'scale': [S, S], 'rotation': kf(rs, 0.04, False)})
    return el

tracks = []
def T(name, layer, els): tracks.append({'name': name, 'layer': layer, 'elements': els})

# set
T('set', 0, [{'id': 'study', 'type': 'image', 'start': 0, 'end': DUR, 'source': 'character/study.png',
              'x': 0, 'y': 0, 'origin': 'top-left', 'width': 1920, 'height': 1080, 'fit': 'contain'}])

# soft floor shadows
cx, cy, cw, ch = CRATE
shadow_scale = [(TIMES[i], 1 + hop(TIMES[i]) / 260) for i in range(len(TIMES))]
sh = kf(shadow_scale, 0.01, False)
T('shadows', 1, [
    {'id': 'crate-shadow', 'type': 'ellipse', 'start': 0, 'end': DUR, 'x': cx + cw // 2, 'y': cy + ch - 2,
     'origin': 'center', 'width': cw + 60, 'height': 34, 'fill': '#5A3A2033'},
])
T('owl-shadow', 2, [
    {'id': 'owl-shadow', 'type': 'ellipse', 'start': 0, 'end': DUR, 'x': FEET[0], 'y': FEET[1] - 4,
     'origin': 'center', 'width': 300, 'height': 38, 'fill': '#5A3A2040',
     'scale': sh if isinstance(sh, list) else [1.0, 1.0]},
])
if isinstance(sh, list):
    for r in tracks[-1]['elements'][0]['scale']:
        r['v'] = [r['v'], r['v']]

# crate, drawn flat with dark outlines like the set
OUT = '#4A3426'
T('crate', 3, [
    {'id': 'crate-body', 'type': 'rect', 'start': 0, 'end': DUR, 'x': cx, 'y': cy, 'origin': 'top-left',
     'width': cw, 'height': ch, 'fill': '#C98A55', 'stroke': OUT, 'stroke_width': 6, 'radius': 8},
])
T('crate-detail', 6, [
    {'id': 'crate-lid', 'type': 'rect', 'start': 0, 'end': DUR, 'x': cx, 'y': cy, 'origin': 'top-left',
     'width': cw, 'height': 30, 'fill': '#B07344', 'stroke': OUT, 'stroke_width': 6, 'radius': 8},
])
T('crate-slat', 4, [
    {'id': 'crate-slat', 'type': 'rect', 'start': 0, 'end': DUR, 'x': cx + 3, 'y': cy + 68, 'origin': 'top-left',
     'width': cw - 6, 'height': 5, 'fill': OUT},
])
T('crate-post-l', 5, [
    {'id': 'crate-post-l', 'type': 'rect', 'start': 0, 'end': DUR, 'x': cx + 40, 'y': cy + 27, 'origin': 'top-left',
     'width': 5, 'height': ch - 30, 'fill': OUT},
])
T('crate-post-r', 5, [
    {'id': 'crate-post-r', 'type': 'rect', 'start': 0, 'end': DUR, 'x': cx + cw - 45, 'y': cy + 27, 'origin': 'top-left',
     'width': 5, 'height': ch - 30, 'fill': OUT},
])

# laptop and its screen
T('laptop', 7, [
    {'id': 'laptop', 'type': 'image', 'start': 0, 'end': DUR, 'source': 'character/laptop.png',
     'x': LAP[0], 'y': LAP[1], 'origin': 'top-left', 'width': 708, 'height': 566, 'fit': 'contain',
     'scale': [LAP_S, LAP_S]},
])
SWAP = 7367   # first frame after "beat" ends (7324)
sx, sy, sw, shh = SCREEN
pic_h = shh; pic_w = round(960 * pic_h / 540)
T('screen', 8, [
    {'id': 'screen-render', 'type': 'image', 'start': 0, 'end': SWAP, 'source': 'stills/session-02.png',
     'x': sx + sw // 2 + 1, 'y': sy + shh // 2, 'origin': 'center', 'width': pic_w, 'height': pic_h,
     'fit': 'cover', 'clip': SCREEN},
    {'id': 'screen-ground', 'type': 'rect', 'start': SWAP, 'end': DUR, 'x': sx, 'y': sy, 'origin': 'top-left',
     'width': sw, 'height': shh, 'fill': '#F5F0E6'},
])
T('screen-mark', 9, [
    {'id': 'screen-mark', 'type': 'image', 'start': SWAP, 'end': DUR, 'source': 'brand/mark.png',
     'x': sx + sw // 2, 'y': sy + shh // 2, 'origin': 'center', 'width': 136, 'height': 136, 'fit': 'contain',
     'scale': [{'t': SWAP, 'v': [0.6, 0.6]}, {'t': SWAP + 170, 'v': [1.08, 1.08], 'ease': 'ease-out'},
               {'t': SWAP + 280, 'v': [1.0, 1.0], 'ease': 'ease-in-out'}]},
])

# the owl, back to front; the left arm comes in front of the head while its hand is at the beak
ARM_FRONT = (T_THINK_IN[0] + 60, T_THINK_OUT[1] - 60)
order = RIG['draw_order']
base = 10
L = {n: base + i for i, n in enumerate(order)}
FRONT_FORE, FRONT_UPPER = base + len(order), base + len(order) + 1
T('owl-torso', L['torso'], [part_el('owl-torso', 'torso', 0, DUR)])
T('owl-forearm-left', L['forearm_left'], [
    part_el('owl-forearm-left-a', 'forearm_left', 0, ARM_FRONT[0]),
    part_el('owl-forearm-left-b', 'forearm_left', ARM_FRONT[0], ARM_FRONT[1], layer=FRONT_FORE),
    part_el('owl-forearm-left-c', 'forearm_left', ARM_FRONT[1], DUR),
])
T('owl-forearm-right', L['forearm_right'], [part_el('owl-forearm-right', 'forearm_right', 0, DUR)])
T('owl-upper-arm-left', L['upper_arm_left'], [
    part_el('owl-upper-arm-left-a', 'upper_arm_left', 0, ARM_FRONT[0]),
    part_el('owl-upper-arm-left-b', 'upper_arm_left', ARM_FRONT[0], ARM_FRONT[1], layer=FRONT_UPPER),
    part_el('owl-upper-arm-left-c', 'upper_arm_left', ARM_FRONT[1], DUR),
])
T('owl-upper-arm-right', L['upper_arm_right'], [part_el('owl-upper-arm-right', 'upper_arm_right', 0, DUR)])
T('owl-head', L['head'], [part_el('owl-head', 'head', 0, DUR)])

# beak: one overlay per viseme run, riding the head
segs = []
vis = LINE['visemes']
for (t0, vid), nxt in zip(vis, vis[1:] + [[LINE['duration_ms'], 0]]):
    m = RIG['visemes'][str(vid)]
    a, b = t0 + VO, nxt[0] + VO
    if b <= a: continue
    if segs and segs[-1][2] == m and segs[-1][1] == a:
        segs[-1][1] = b
    else:
        segs.append([a, b, m])
mouths = []
for n, (a, b, m) in enumerate(s for s in segs if s[2]):
    mouths.append(part_el(f'beak-{n:02d}-{m}', 'head', a, b, src_part='mouth_' + m))
T('owl-beak', L['mouth_open'], mouths)

BLINKS = [566, 2466, 4333, 7000]
T('owl-blink', L['eyes_closed'], [part_el(f'blink-{n}', 'head', t, t + 100, src_part='eyes_closed')
                                   for n, t in enumerate(BLINKS)])

# the tag
def tag_pair(gid, text, start, end, scale_keys, box_w):
    sc = [dict(r) for r in scale_keys]
    box = {'id': gid + '-box', 'type': 'rect', 'group': gid, 'start': start, 'end': end,
           'x': TAG[0], 'y': TAG[1], 'origin': 'center', 'width': box_w, 'height': 96,
           'fill': '#FF5A36', 'stroke': '#101418', 'stroke_width': 5, 'radius': 22, 'scale': sc}
    txt = {'id': gid + '-text', 'type': 'text', 'group': gid, 'start': start, 'end': end,
           'x': TAG[0], 'y': TAG[1] + 2, 'origin': 'center', 'width': TEXT_W[text], 'height': TEXT_H[text],
           'font': 'inter-bold', 'size': 54, 'line_height': 1.0, 'color': '#F5F0E6', 'align': 'center',
           'runs': [{'text': text}], 'scale': [dict(r) for r in scale_keys], 'caption': False}
    return box, txt

TEXT_W = json.load(open(os.path.join(ROOT, 'build/text_sizes.json')))['w']
TEXT_H = json.load(open(os.path.join(ROOT, 'build/text_sizes.json')))['h']
pop = [{'t': T_TWO, 'v': [0.0, 0.0]}, {'t': T_TWO + 140, 'v': [1.16, 1.16], 'ease': 'ease-out'},
       {'t': T_TWO + 230, 'v': [0.95, 0.95], 'ease': 'ease-in-out'},
       {'t': T_TWO + 300, 'v': [1.0, 1.0], 'ease': 'ease-in-out'}]
bump = [{'t': T_FIXED, 'v': [0.82, 0.82]}, {'t': T_FIXED + 110, 'v': [1.12, 1.12], 'ease': 'ease-out'},
        {'t': T_FIXED + 220, 'v': [1.0, 1.0], 'ease': 'ease-in-out'}]
b1, t1 = tag_pair('tag-late', '+2 frames', T_TWO, T_FIXED, pop, TEXT_W['+2 frames'] + 64)
b2, t2 = tag_pair('tag-fixed', '0 frames', T_FIXED, DUR, bump, TEXT_W['+2 frames'] + 64)
T('tag-box', 40, [b1, b2])
T('tag-text', 41, [t1, t2])

# sound
T('voice', 0, [{'id': 'voice', 'type': 'audio', 'start': VO, 'end': VO + LINE['duration_ms'],
                'source': 'character/voice/line-3.wav', 'source_start': 0, 'source_end': LINE['duration_ms'], 'volume': 1.5}])
T('music', 0, [{'id': 'music', 'type': 'audio', 'start': 0, 'end': DUR, 'source': 'music/bed-120bpm.wav',
                'source_start': 0, 'source_end': DUR,
                'volume': [{'t': 0, 'v': 0.0}, {'t': 400, 'v': 0.12, 'ease': 'ease-out'},
                           {'t': 7300, 'v': 0.12, 'ease': 'step'}, {'t': 7966, 'v': 0.0, 'ease': 'ease-in'}]}])

proj = json.load(open(os.path.join(ROOT, 'hoot.json')))
proj['fonts'] = {'inter-bold': [{'file': 'vendor/Inter-Bold.ttf'}]}
proj['tracks'] = tracks
json.dump(proj, open(os.path.join(ROOT, 'hoot.json'), 'w'), indent=2, ensure_ascii=False)
print('think', THINK_L, 'reach', REACH_R, 'mouth segs', len(mouths))
