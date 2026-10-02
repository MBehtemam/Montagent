#!/usr/bin/env python3
"""Generate hoot.json: bakes the owl cut-out rig's hierarchy into flat per-frame keyframes."""
import json, math, os

WORK = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
RIG = json.load(open(os.path.join(WORK, 'character/rig.json')))
LINE = json.load(open(os.path.join(WORK, 'character/voice/line-3.json')))
PARTS = RIG['parts']

FPS = 30
DUR = 8000
NF = DUR * FPS // 1000
VO = 1000                       # voice starts at 1 s
S = 0.6                         # owl scale
FLOOR_Y = 995                   # frame y of the torso pivot (feet) when standing
ROOT_X = 700

W = {w['word'].strip('.!…,').lower(): (w['start'] + VO, w['end'] + VO) for w in LINE['words']}
W_FIXED = W['fixed']; W_BEAT = W['beat']

# ---------------------------------------------------------------- easing / channels
def ease_fn(name):
    if name == 'linear': return lambda u: u
    if name == 'in': return lambda u: u * u * u
    if name == 'out': return lambda u: 1 - (1 - u) ** 3
    if name == 'quadout': return lambda u: 1 - (1 - u) ** 2
    if name == 'quadin': return lambda u: u * u
    if name == 'back':  # overshoot
        c1 = 1.9; c3 = c1 + 1
        return lambda u: 1 + c3 * (u - 1) ** 3 + c1 * (u - 1) ** 2
    return lambda u: u * u * (3 - 2 * u)  # smooth

def chan(keys):
    """keys: [(t, v, ease)] ; ease applies to the segment arriving at that key."""
    def f(t):
        if t <= keys[0][0]: return keys[0][1]
        for (t0, v0, _), (t1, v1, e) in zip(keys, keys[1:]):
            if t <= t1:
                u = (t - t0) / (t1 - t0) if t1 > t0 else 1
                return v0 + (v1 - v0) * ease_fn(e)(u)
        return keys[-1][1]
    return f

def wave(t, t0, t1, amp, period):
    if t < t0 or t > t1: return 0.0
    env = min(1, (t - t0) / 200, (t1 - t) / 200)
    return amp * env * math.sin(2 * math.pi * (t - t0) / period)

# ---------------------------------------------------------------- geometry
def rot(v, deg):
    a = math.radians(deg); c, s = math.cos(a), math.sin(a)
    return (v[0] * c - v[1] * s, v[0] * s + v[1] * c)
def sub(a, b): return (a[0] - b[0], a[1] - b[1])
def add(a, b): return (a[0] + b[0], a[1] + b[1])
def mul(a, k): return (a[0] * k, a[1] * k)
def ang(v): return math.degrees(math.atan2(v[1], v[0]))
def piv(n): return tuple(PARTS[n]['pivot'])

# hand-centre reference points (drawing coords), read off the forearm parts
HAND = {'left': (115, 852), 'right': (909, 852)}
SH = {'left': piv('upper_arm_left'), 'right': piv('upper_arm_right')}
EL = {'left': piv('forearm_left'), 'right': piv('forearm_right')}
REST_UA = {k: ang(sub(EL[k], SH[k])) for k in SH}       # rest direction shoulder->elbow
REST_FA = {k: ang(sub(HAND[k], EL[k])) for k in SH}      # rest direction elbow->hand
L1 = {k: math.dist(EL[k], SH[k]) for k in SH}
L2 = {k: math.dist(HAND[k], EL[k]) for k in SH}

def arm_local(side, ua_dir, fa_dir):
    """absolute directions (torso frame) -> local rotations (upper rel torso, forearm rel upper)."""
    ru = ua_dir - REST_UA[side]
    rf = (fa_dir - REST_FA[side]) - ru
    return ru, rf

def ik(side, target, elbow_down=True):
    """directions (ua_dir, fa_dir) placing the hand centre at target (torso frame, drawing px)."""
    s = SH[side]; d = sub(target, s); dist = math.hypot(*d)
    a, b = L1[side], L2[side]
    dist = min(dist, a + b - 1e-6)
    base = ang(d)
    cosA = (a * a + dist * dist - b * b) / (2 * a * dist)
    A = math.degrees(math.acos(max(-1, min(1, cosA))))
    ua = base + (A if elbow_down else -A)
    e = add(s, mul((math.cos(math.radians(ua)), math.sin(math.radians(ua))), a))
    fa = ang(sub(target, e))
    return ua, fa

# ---------------------------------------------------------------- performance
T_HMM0, T_HMM1 = W['hmm']
T_LATE1 = W['late'][1]
T_LET0 = W['let'][0]
T_CHECK0, T_CHECK1 = W['check']
T_RIGHT0 = W['right'][0]

# arm directions (absolute, torso frame, degrees, y down; 90 = straight down)
SIDE_L = (112, 100)            # hanging by the left side
SIDE_R = (180 - 112, 180 - 100)
RAISE_L = (210, 234)           # up and out, a V over the head
RAISE_R = (180 - 210 + 360, 180 - 234 + 360)

think_w = chan([(0, 0, 's'), (T_HMM0 - 120, 0, 's'), (T_HMM0 + 280, 1, 'out'), (T_LATE1 - 20, 1, 's'), (T_LATE1 + 330, 0, 's')])
type_w = chan([(0, 0, 's'), (T_LATE1 + 150, 0, 's'), (T_LET0 + 150, 1, 'out'), (T_CHECK1 + 80, 1, 's'), (W_FIXED[0] - 150, 0, 's')])
raise_w = chan([(0, 0, 's'), (W_FIXED[0] - 170, 0, 's'), (W_FIXED[0] + 60, 1, 'out'), (T_RIGHT0 + 60, 1, 's'), (W['beat'][1] - 100, 0, 's')])

HOP_T0, HOP_T1 = W_FIXED[0] + 20, W_FIXED[0] + 420
HOP_H = 110                    # frame px
def hop(t):
    if t <= HOP_T0 or t >= HOP_T1: return 0.0
    u = (t - HOP_T0) / (HOP_T1 - HOP_T0)
    return HOP_H * 4 * u * (1 - u)

lean = chan([(0, 0, 's'), (T_LATE1 + 150, 0, 's'), (T_LET0 + 150, 8, 'out'), (T_CHECK1 + 80, 8, 's'), (W_FIXED[0] - 150, 0, 's')])

head_base = chan([
    (0, 0, 's'), (T_HMM0 - 120, 0, 's'), (T_HMM0 + 300, -11, 'out'),
    (T_LATE1 - 20, -11, 's'), (T_LET0 + 100, 9, 's'), (T_CHECK1 + 80, 9, 's'),
    (W_FIXED[0] - 150, 0, 's'), (HOP_T0 + 150, -4, 'out'), (HOP_T1, 3, 's'), (HOP_T1 + 180, 0, 's'),
    (W['beat'][0], 0, 's'), (W['beat'][0] + 160, 4, 'out'), (W['beat'][1] + 300, 0, 's')])

def head_rot(t):
    r = head_base(t)
    r += wave(t, 0, T_HMM0 - 100, 1.2, 1700)                 # idle sway
    r += wave(t, W['that'][0], T_LATE1, 1.6, 620)            # little nods while thinking aloud
    r += wave(t, T_CHECK1 + 80, W_FIXED[0] - 150, 0, 1000)
    r += wave(t, W['beat'][1] + 300, DUR, 1.0, 1600)         # settle sway at the end
    return r

TAPS = [T_LET0 + 260, T_LET0 + 470, T_CHECK0 + 230, T_CHECK0 + 430]   # tap-down instants
def tap(t):
    v = 0.0
    for tt in TAPS:
        d = (t - tt) / 95.0
        v = max(v, math.exp(-d * d))
    return v

# typing pose target (torso frame)
TYPE_UA, TYPE_FA = 0, 36        # right upper arm nearly level, forearm angled down to the keys
TAP_DEG = 20                    # forearm rotates down this much at each tap

def lerp(a, b, u): return a + (b - a) * u
def lerp2(a, b, u): return (lerp(a[0], b[0], u), lerp(a[1], b[1], u))

BEAK_TARGET = (470, 676)        # left hand centre under the beak, head-local drawing px

def pose(t):
    """returns dict of world transforms: name -> (x, y, rotation_deg)."""
    lt = lean(t)
    hr = head_rot(t)
    breathe = 2.0 * math.sin(2 * math.pi * t / 2400)  # tiny arm breathing
    root = (ROOT_X, FLOOR_Y - hop(t))

    def torso_w(p):
        return add(root, mul(rot(sub(p, piv('torso')), lt), S))

    # left arm: side <-> think (IK to beak) <-> raise
    ua_l, fa_l = SIDE_L[0] + breathe * 0.5, SIDE_L[1] + breathe
    tw = think_w(t)
    if tw > 0:
        hp = piv('head')
        tgt = add(hp, rot(sub(BEAK_TARGET, hp), hr))
        ti = ik('left', tgt, elbow_down=True)
        ua_l, fa_l = lerp(ua_l, ti[0], tw), lerp(fa_l, ti[1], tw)
    rw = raise_w(t)
    ua_l, fa_l = lerp(ua_l, RAISE_L[0], rw), lerp(fa_l, RAISE_L[1], rw)

    # right arm: side <-> type <-> raise
    ua_r, fa_r = SIDE_R[0] - breathe * 0.5, SIDE_R[1] - breathe
    yw = type_w(t)
    ua_r = lerp(ua_r, TYPE_UA, yw); fa_r = lerp(fa_r, TYPE_FA + TAP_DEG * tap(t), yw)
    ua_r, fa_r = lerp(ua_r, RAISE_R[0] - 360, rw), lerp(fa_r, RAISE_R[1] - 360, rw)

    out = {}
    out['torso'] = (*root, lt)
    hp = torso_w(piv('head')); out['head'] = (*hp, lt + hr)
    for side, (ua, fa) in (('left', (ua_l, fa_l)), ('right', (ua_r, fa_r))):
        ru, rf = arm_local(side, ua, fa)
        sp = torso_w(SH[side])
        out['upper_arm_' + side] = (*sp, lt + ru)
        ep = add(sp, mul(rot(sub(EL[side], SH[side]), lt + ru), S))
        out['forearm_' + side] = (*ep, lt + ru + rf)
        out['_hand_' + side] = add(ep, mul(rot(sub(HAND[side], EL[side]), lt + ru + rf), S))
    out['_hop'] = hop(t)
    return out

def ftime(k): return round(k * 1000 / FPS)
POSES = [pose(k * 1000 / FPS) for k in range(NF + 1)]

# ---------------------------------------------------------------- emit helpers
def kf_list(vals, rnd):
    """vals: [(t, v)] -> keyframe records, constant runs collapsed; or a scalar if all equal."""
    vals = [(t, rnd(v)) for t, v in vals]
    if all(v == vals[0][1] for _, v in vals): return vals[0][1]
    keep = [vals[0]]
    for i in range(1, len(vals) - 1):
        if not (vals[i][1] == vals[i - 1][1] == vals[i + 1][1]):
            keep.append(vals[i])
    keep.append(vals[-1])
    out = [{'t': keep[0][0], 'v': keep[0][1]}]
    for t, v in keep[1:]: out.append({'t': t, 'v': v, 'ease': 'linear'})
    return out

def r2(v):
    v = round(v, 2)
    return int(v) if v == int(v) else v

def part_elem(eid, part, start, end, drive=None):
    """an image element for a rig part, driven by `drive`'s world transform (default: itself)."""
    drive = drive or part
    p = PARTS[part]
    k0 = max(0, math.floor(start * FPS / 1000) - 1); k1 = min(NF, math.ceil(end * FPS / 1000) + 1)
    ks = range(k0, k1 + 1)
    xs = kf_list([(ftime(k), POSES[k][drive][0]) for k in ks], lambda v: int(round(v)))
    ys = kf_list([(ftime(k), POSES[k][drive][1]) for k in ks], lambda v: int(round(v)))
    rs = kf_list([(ftime(k), POSES[k][drive][2]) for k in ks], r2)
    e = {'id': eid, 'type': 'image', 'start': start, 'end': end, 'source': 'character/' + p['file'],
         'x': xs, 'y': ys, 'origin': 'center', 'width': p['width'], 'height': p['height'], 'fit': 'literal',
         'scale': [S, S]}
    if rs != 0: e['rotation'] = rs
    return e

tracks = []
def track(name, layer, elems): tracks.append({'name': name, 'layer': layer, 'elements': elems})

# ---------------------------------------------------------------- set
track('study', 0, [{'id': 'study', 'type': 'image', 'start': 0, 'end': DUR, 'source': 'character/study.png',
                    'x': 0, 'y': 0, 'origin': 'top-left', 'width': 1920, 'height': 1080, 'fit': 'cover'}])

# laptop placement: keyboard under the typing hand
LS = 0.62
LAP_W, LAP_H = round(708 * LS), round(566 * LS)
hand_type = POSES[round((TAPS[0] - 120) * FPS / 1000)]['_hand_right']
KEY_PT = (265, 400)                      # a key on the left half of the keyboard (laptop px)
LX = int(round(hand_type[0] - KEY_PT[0] * LS)); LY = int(round(hand_type[1] - KEY_PT[1] * LS))
LAP_BASE = LY + round(560 * LS)          # where the laptop's base sits
CRATE_X0, CRATE_X1 = LX - 18, LX + LAP_W + 18
CRATE_Y1 = 978                           # crate bottom on the floor
INK = '#2E2420'
crate = [
    {'id': 'crate-shadow', 'type': 'ellipse', 'start': 0, 'end': DUR, 'x': (CRATE_X0 + CRATE_X1) // 2, 'y': CRATE_Y1 - 2, 'origin': 'center',
     'width': CRATE_X1 - CRATE_X0 + 60, 'height': 34, 'fill': '#7A4E2A', 'opacity': 0.35},
]
track('crate-shadow', 2, crate)
crate_h = CRATE_Y1 - LAP_BASE
track('crate-body', 3, [{'id': 'crate-body', 'type': 'rect', 'start': 0, 'end': DUR, 'x': CRATE_X0, 'y': LAP_BASE, 'origin': 'top-left',
                         'width': CRATE_X1 - CRATE_X0, 'height': crate_h, 'fill': '#C98A4F', 'stroke': INK, 'stroke_width': 6, 'radius': 6}])
track('crate-top', 4, [{'id': 'crate-top', 'type': 'rect', 'start': 0, 'end': DUR, 'x': CRATE_X0, 'y': LAP_BASE, 'origin': 'top-left',
                        'width': CRATE_X1 - CRATE_X0, 'height': 22, 'fill': '#DDA466', 'stroke': INK, 'stroke_width': 6, 'radius': 6}])
plank_y = [LAP_BASE + 22 + (crate_h - 22) * i // 3 for i in (1, 2)]
for i, y in enumerate(plank_y):
    track('crate-plank-%d' % i, 5, [{'id': 'crate-plank-%d' % i, 'type': 'rect', 'start': 0, 'end': DUR, 'x': CRATE_X0 + 3, 'y': y - 2, 'origin': 'top-left',
                                     'width': CRATE_X1 - CRATE_X0 - 6, 'height': 5, 'fill': '#9A6538'}])
track('crate-post-l', 6, [{'id': 'crate-post-l', 'type': 'rect', 'start': 0, 'end': DUR, 'x': CRATE_X0 + 18, 'y': LAP_BASE + 22, 'origin': 'top-left', 'width': 6, 'height': crate_h - 25, 'fill': '#9A6538'}])
track('crate-post-r', 6, [{'id': 'crate-post-r', 'type': 'rect', 'start': 0, 'end': DUR, 'x': CRATE_X1 - 24, 'y': LAP_BASE + 22, 'origin': 'top-left', 'width': 6, 'height': crate_h - 25, 'fill': '#9A6538'}])

track('laptop', 8, [{'id': 'laptop', 'type': 'image', 'start': 0, 'end': DUR, 'source': 'character/laptop.png',
                     'x': LX, 'y': LY, 'origin': 'top-left', 'width': LAP_W, 'height': LAP_H, 'fit': 'contain'}])
# screen: image px x 120-587, y 41-312 -> frame, rounded inward
SX0 = math.floor(LX + 120 * LS); SX1 = round(LX + 587 * LS)
SY0 = math.floor(LY + 41 * LS); SY1 = round(LY + 312 * LS)
SW, SH_ = SX1 - SX0, SY1 - SY0
SCREEN_SWAP = W_BEAT[1] + 40
# session-02 covers the screen
cov = max(SW / 960, SH_ / 540)
iw, ih = int(960 * cov), int(540 * cov)
track('screen', 9, [
    {'id': 'screen-session', 'type': 'image', 'start': 0, 'end': SCREEN_SWAP, 'source': 'stills/session-02.png',
     'x': SX0 + SW // 2, 'y': SY0 + SH_ // 2, 'origin': 'center', 'width': iw, 'height': ih, 'fit': 'cover',
     'clip': [SX0, SY0, SW, SH_]},
    {'id': 'screen-ground', 'type': 'rect', 'start': SCREEN_SWAP, 'end': DUR, 'x': SX0, 'y': SY0, 'origin': 'top-left', 'width': SW, 'height': SH_, 'fill': '#F5F0E6'}])
MARK = round(SH_ * 0.56)
WM_W = round(SW * 0.56); WM_H = round(WM_W * 419 / 1998)
gap = 8
top = SY0 + (SH_ - (MARK + gap + WM_H)) // 2
track('screen-brand', 10, [
    {'id': 'screen-mark', 'type': 'image', 'start': SCREEN_SWAP, 'end': DUR, 'source': 'brand/mark.png',
     'x': SX0 + SW // 2, 'y': top, 'origin': 'top-center', 'width': MARK, 'height': MARK, 'fit': 'contain',
     'scale': [{'t': SCREEN_SWAP, 'v': [0.6, 0.6]}, {'t': SCREEN_SWAP + 250, 'v': [1.0, 1.0], 'ease': 'ease-out'}]}])
track('screen-wordmark', 11, [
    {'id': 'screen-wordmark', 'type': 'image', 'start': SCREEN_SWAP, 'end': DUR, 'source': 'brand/wordmark.png',
     'x': SX0 + SW // 2, 'y': top + MARK + gap, 'origin': 'top-center', 'width': WM_W, 'height': WM_H, 'fit': 'contain',
     'opacity': [{'t': SCREEN_SWAP + 100, 'v': 0.0}, {'t': SCREEN_SWAP + 300, 'v': 1.0, 'ease': 'ease-out'}]}])

# ---------------------------------------------------------------- tag
TAG_X = LX + LAP_W // 2 + 40; TAG_Y = LY - 72
T_TWO = W['two'][0]
def pop(t0, frm=0.0):
    return [{'t': t0, 'v': [frm, frm]}, {'t': t0 + 140, 'v': [1.16, 1.16], 'ease': 'ease-out'},
            {'t': t0 + 240, 'v': [0.95, 0.95], 'ease': 'ease-in-out'}, {'t': t0 + 330, 'v': [1.0, 1.0], 'ease': 'ease-in-out'}]
tag_pill = lambda eid, s, e, w, sc: {'id': eid, 'type': 'rect', 'start': s, 'end': e, 'x': TAG_X, 'y': TAG_Y, 'origin': 'center',
                                     'width': w, 'height': 84, 'fill': '#FF5A36', 'stroke': '#101418', 'stroke_width': 5, 'radius': 42, 'scale': sc}
tag_text = lambda eid, s, e, txt, w, sc: {'id': eid, 'type': 'text', 'start': s, 'end': e, 'x': TAG_X, 'y': TAG_Y, 'origin': 'center',
                                          'width': w, 'height': 58, 'font': 'bold', 'size': 48, 'color': '#F5F0E6',
                                          'runs': [{'text': txt}], 'scale': sc, 'caption': False}
track('tag-pill', 40, [tag_pill('tag-pill-late', T_TWO, W_FIXED[0], 300, pop(T_TWO)),
                       tag_pill('tag-pill-fixed', W_FIXED[0], DUR, 300, pop(W_FIXED[0], 0.85))])
track('tag-text', 41, [tag_text('tag-text-late', T_TWO, W_FIXED[0], '+2 frames', 240, pop(T_TWO)),
                       tag_text('tag-text-fixed', W_FIXED[0], DUR, '0 frames', 210, pop(W_FIXED[0], 0.85))])
# pointer under the tag
track('tag-pointer', 39, [
    {'id': 'tag-pointer-late', 'type': 'rect', 'start': T_TWO, 'end': W_FIXED[0], 'x': TAG_X, 'y': TAG_Y + 42, 'origin': 'center',
     'width': 26, 'height': 26, 'fill': '#FF5A36', 'stroke': '#101418', 'stroke_width': 5, 'rotation': 45.0, 'scale': pop(T_TWO)},
    {'id': 'tag-pointer-fixed', 'type': 'rect', 'start': W_FIXED[0], 'end': DUR, 'x': TAG_X, 'y': TAG_Y + 42, 'origin': 'center',
     'width': 26, 'height': 26, 'fill': '#FF5A36', 'stroke': '#101418', 'stroke_width': 5, 'rotation': 45.0, 'scale': pop(W_FIXED[0], 0.85)}])

# ---------------------------------------------------------------- owl
# floor shadow follows the hop
shadow_scale = kf_list([(ftime(k), 1 - 0.35 * POSES[k]['_hop'] / HOP_H) for k in range(NF + 1)], lambda v: round(v, 3))
if isinstance(shadow_scale, list):
    shadow_scale = [dict(d, v=[d['v'], d['v']]) for d in shadow_scale]
track('owl-shadow', 15, [{'id': 'owl-shadow', 'type': 'ellipse', 'start': 0, 'end': DUR, 'x': ROOT_X, 'y': FLOOR_Y + 4, 'origin': 'center',
                          'width': 300, 'height': 36, 'fill': '#6B4325', 'opacity': 0.4, 'scale': shadow_scale}])

# the thinking arm (viewer's left) is drawn in front of the head while the hand is at the beak
FRONT0 = T_HMM0 - 100
FRONT1 = T_LATE1 + 330
track('owl-torso', 20, [part_elem('torso', 'torso', 0, DUR)])
track('owl-forearm-left', 21, [part_elem('forearm-left-a', 'forearm_left', 0, FRONT0), part_elem('forearm-left-b', 'forearm_left', FRONT1, DUR)])
track('owl-forearm-right', 22, [part_elem('forearm-right', 'forearm_right', 0, DUR)])
track('owl-upper-arm-left', 23, [part_elem('upper-arm-left-a', 'upper_arm_left', 0, FRONT0), part_elem('upper-arm-left-b', 'upper_arm_left', FRONT1, DUR)])
track('owl-upper-arm-right', 24, [part_elem('upper-arm-right', 'upper_arm_right', 0, DUR)])
track('owl-head', 25, [part_elem('head', 'head', 0, DUR)])
track('owl-forearm-left-front', 26, [part_elem('forearm-left-front', 'forearm_left', FRONT0, FRONT1)])
track('owl-upper-arm-left-front', 27, [part_elem('upper-arm-left-front', 'upper_arm_left', FRONT0, FRONT1)])

# beak: viseme -> mouth overlay segments
segs = []
vis = LINE['visemes']
for i, (ms, vid) in enumerate(vis):
    m = RIG['visemes'][str(vid)]
    t0 = ms + VO
    t1 = (vis[i + 1][0] + VO) if i + 1 < len(vis) else LINE['duration_ms'] + VO
    if segs and segs[-1][2] == m and segs[-1][1] == t0:
        segs[-1][1] = t1
    else:
        segs.append([t0, t1, m])
mouths = [part_elem('mouth-%02d-%s' % (i, m), 'mouth_' + m, t0, t1, drive='head')
          for i, (t0, t1, m) in enumerate([s for s in segs if s[2]])]
track('owl-mouth', 28, mouths)

BLINKS = [430, 2560, 4330, 6420, 7600]
track('owl-eyes', 29, [part_elem('blink-%d' % i, 'eyes_closed', b, b + 100, drive='head') for i, b in enumerate(BLINKS)])

# ---------------------------------------------------------------- sound
track('voice', 50, [{'id': 'voice', 'type': 'audio', 'start': VO, 'end': VO + LINE['duration_ms'], 'source': 'character/voice/line-3.wav',
                     'source_start': 0, 'source_end': LINE['duration_ms'], 'volume': 1.4}])
track('music', 51, [{'id': 'music', 'type': 'audio', 'start': 0, 'end': DUR, 'source': 'music/bed-120bpm.wav', 'source_start': 0, 'source_end': DUR,
                     'volume': [{'t': 0, 'v': 0.14}, {'t': VO, 'v': 0.08, 'ease': 'ease-in-out'}, {'t': VO + LINE['duration_ms'], 'v': 0.08, 'ease': 'linear'},
                                {'t': 7700, 'v': 0.12, 'ease': 'ease-in-out'}, {'t': DUR, 'v': 0.0, 'ease': 'ease-in'}]}])

# ---------------------------------------------------------------- write
proj_path = os.path.join(WORK, 'hoot.json')
proj = json.load(open(proj_path))
proj['tracks'] = tracks
json.dump(proj, open(proj_path, 'w'))
print('laptop', LX, LY, LAP_W, LAP_H, 'screen', SX0, SY0, SW, SH_, 'crate', CRATE_X0, LAP_BASE, CRATE_X1, CRATE_Y1, 'tag', TAG_X, TAG_Y)
print('hand at type', hand_type)
print('segments', len(mouths))
