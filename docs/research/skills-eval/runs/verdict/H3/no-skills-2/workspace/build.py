#!/usr/bin/env python3
"""Generate hoot.json: Hoot fixes a late caption (brief H3).

Montagent transforms are flat, so the rig's hierarchy is solved here (forward
kinematics) and baked as one keyframe per frame for every part.
"""
import json, math

HERE = '.'
FPS = 30
DUR = 8000
NF = DUR * FPS // 1000            # 240 frames
VO = 1000                         # the line starts at 1 s

rig = json.load(open('character/rig.json'))
line = json.load(open('character/voice/line-3.json'))
P = rig['parts']

# ---------------------------------------------------------------- placement
S = 0.58                          # owl scale (drawing px -> frame px)
FEET_Y = 1002                     # frame y of the bottom of the feet
OWL_X = 740                       # frame x of the drawing's centre line (512)
FEET_DRAW_Y = 1331                # drawing y of the feet's lowest pixel

def to_frame(px, py, ox, oy):
    return (OWL_X + ox + (px - 512) * S, FEET_Y + oy + (py - FEET_DRAW_Y) * S)

# laptop on a crate, right of the owl
LAP_W, LAP_H = 425, 340
LAP_X, LAP_Y = 880, 452          # top-left of the laptop
kx, ky = LAP_W / 708, LAP_H / 566
SCREEN = (LAP_X + 120 * kx, LAP_Y + 41 * ky, LAP_X + 588 * kx, LAP_Y + 313 * ky)
CLIP = [math.ceil(SCREEN[0]), math.ceil(SCREEN[1])]
CLIP += [math.floor(SCREEN[2]) - CLIP[0], math.floor(SCREEN[3]) - CLIP[1]]
LAP_BOTTOM = LAP_Y + int(553 * ky)   # the base's underside (shadow stripe excluded)
CRATE = dict(x=LAP_X - 15, y=LAP_BOTTOM, w=LAP_W + 30, h=FEET_Y - 8 - LAP_BOTTOM)

# ---------------------------------------------------------------- helpers
def ms(f):
    return f * 1000 // FPS          # floor: frame f is sampled at f*1000/30

def smooth(u):
    u = min(1.0, max(0.0, u))
    return u * u * (3 - 2 * u)

class Track:
    """Piecewise pose channel: list of (t_ms, value); smoothstep between keys."""
    def __init__(self, keys):
        self.keys = keys
    def __call__(self, t):
        k = self.keys
        if t <= k[0][0]:
            return k[0][1]
        for (t0, v0), (t1, v1) in zip(k, k[1:]):
            if t <= t1:
                return v0 + (v1 - v0) * smooth((t - t0) / (t1 - t0))
        return k[-1][1]

def rot(v, deg):
    a = math.radians(deg)
    c, s = math.cos(a), math.sin(a)
    return (v[0] * c - v[1] * s, v[0] * s + v[1] * c)

def add(a, b):
    return (a[0] + b[0], a[1] + b[1])

def sub(a, b):
    return (a[0] - b[0], a[1] - b[1])

# joints, in drawing coordinates
FEET = tuple(P['torso']['pivot'])
NECK = tuple(P['head']['pivot'])
SH = {'left': tuple(P['upper_arm_left']['pivot']), 'right': tuple(P['upper_arm_right']['pivot'])}
EL = {'left': tuple(P['forearm_left']['pivot']), 'right': tuple(P['forearm_right']['pivot'])}
HAND = {'left': (98, 858), 'right': (926, 858)}       # palm centres at rest

def ang(v):
    return math.degrees(math.atan2(v[1], v[0]))

REST_UA = {s: ang(sub(EL[s], SH[s])) for s in SH}         # ~156.5 / ~25.8
REST_FA = {s: ang(sub(HAND[s], EL[s])) for s in SH}

def pose_world(p):
    """p: dict of local rotations + root offset -> world (pivot xy, rotation) per part."""
    root = (p['ox'], p['oy'])
    out = {}
    tr = p['torso']
    feet_w = to_frame(*FEET, *root)
    feet_w = (feet_w[0], feet_w[1] + (FEET[1] - FEET_DRAW_Y) * 0)   # pivot sits at drawing y 1322
    feet_w = to_frame(FEET[0], FEET[1], *root)
    out['torso'] = (feet_w, tr)
    def child(parent_xy, parent_draw, parent_rot, joint_draw):
        d = sub(joint_draw, parent_draw)
        d = rot((d[0] * S, d[1] * S), parent_rot)
        return add(parent_xy, d)
    neck = child(feet_w, FEET, tr, NECK)
    out['head'] = (neck, tr + p['head'])
    for s in ('left', 'right'):
        sh = child(feet_w, FEET, tr, SH[s])
        ua = tr + p['ua_' + s]
        out['upper_arm_' + s] = (sh, ua)
        el = child(sh, SH[s], ua, EL[s])
        fa = ua + p['fa_' + s]
        out['forearm_' + s] = (el, fa)
    return out

def hand_world(p, side):
    w = pose_world(p)
    el, fa = w['forearm_' + side]
    d = sub(HAND[side], EL[side])
    return add(el, rot((d[0] * S, d[1] * S), fa))

def ik(p, side, target, prefer):
    """Brute-force two-bone IK: local (ua, fa) putting the palm on target."""
    best = None
    for a10 in range(-1800, 1800, 10):
        a = a10 / 10
        for b10 in range(-1500, 1500, 20):
            b = b10 / 10
            q = dict(p, **{'ua_' + side: a, 'fa_' + side: b})
            h = hand_world(q, side)
            err = math.hypot(h[0] - target[0], h[1] - target[1])
            score = err + prefer(a, b)
            if best is None or score < best[0]:
                best = (score, a, b, err)
    return best

# ---------------------------------------------------------------- poses
# local rotations (deg, clockwise) relative to the drawn rest pose
SIDES = dict(ua_left=-56.0, fa_left=8.0, ua_right=54.0, fa_right=-8.0)     # arms down
UP = dict(ua_left=35.0, fa_left=50.0, ua_right=-35.0, fa_right=-50.0)       # arms raised

T = lambda s: s + VO   # voice-relative ms -> timeline ms
W = {w['word'].strip('.!…,').lower(): (T(w['start']), T(w['end'])) for w in line['words']}
# 'the' & 'that' etc. are unique enough; keep the ones we need
t_hmm = W['hmm']; t_two = W['two']; t_late = W['late']; t_let = W['let']
t_check = W['check']; t_fixed = W['fixed']; t_right = W['right']; t_beat = W['beat']

base = dict(ox=0.0, oy=0.0, torso=0.0, head=0.0, **SIDES)

# thinking pose: head tilts, left palm under the beak
THINK_HEAD = 8.0
think = dict(base, head=THINK_HEAD, torso=4.0)
# beak's lower left edge in drawing space, carried by the head
def head_point(p, pt):
    w = pose_world(p)
    neck, hr = w['head']
    d = sub(pt, NECK)
    return add(neck, rot((d[0] * S, d[1] * S), hr))
tgt = head_point(think, (462, 662))
r = ik(think, 'left', tgt, lambda a, b: 0.02 * abs(b - 120))
think['ua_left'], think['fa_left'] = r[1], r[2]
print('think IK', r)

# reaching: lean to the laptop, palm over the keyboard
reach = dict(base, torso=6.0, head=1.0, ua_left=-50.0, fa_left=6.0)
KEY_TGT = (LAP_X + 230 * kx, LAP_Y + 395 * ky)    # over the left half of the keys
r = ik(reach, 'right', KEY_TGT, lambda a, b: 0.02 * abs(b + 10))
reach['ua_right'], reach['fa_right'] = r[1], r[2]
print('reach IK', r, KEY_TGT)
TAP = 16.0      # forearm dips (clockwise = down for the right arm)

def keys(*pairs):
    return list(pairs)

def poseline(name, default):
    return default

# key poses on the timeline: (t, pose)
POSES = [
    (0, base),
    (T(t_hmm[0] - VO) - 80, base),
    (T(t_hmm[0] - VO) + 260, think),
    (t_late[1], think),
    (t_late[1] + 330, reach),
]
# taps during "Let me check…"
tap0 = t_let[0] + 120
taps = []
for i in range(2):
    s = tap0 + i * 260
    down = dict(reach, fa_right=reach['fa_right'] + TAP)
    taps += [(s, reach), (s + 110, down), (s + 230, reach)]
POSES += taps
# hold, then wind up for the hop
POSES += [
    (t_check[1] + 60, dict(reach)),
    (t_fixed[0] - 60, dict(base, **UP, head=-1.0, oy=0.0)),
]
# the hop: up, apex, land, settle
hop0 = t_fixed[0] + 20
POSES += [
    (hop0, dict(base, **UP, head=-1.0, oy=0.0)),
    (hop0 + 190, dict(base, **UP, head=1.0, oy=-125.0)),
    (hop0 + 380, dict(base, **UP, head=0.0, oy=0.0)),
    (t_right[0] + 40, dict(base, **UP, head=-0.5)),
    (t_right[0] + 520, dict(base, head=0.5)),
    (t_beat[1] + 300, base),
    (DUR, base),
]
POSES.sort(key=lambda kv: kv[0])
CHANNELS = ['ox', 'oy', 'torso', 'head', 'ua_left', 'fa_left', 'ua_right', 'fa_right']

def pose_at(t):
    p = {}
    for c in CHANNELS:
        if c == 'oy':
            continue
        p[c] = Track([(tt, v[c]) for tt, v in POSES])(t)
    # vertical: ballistic between take-off and landing, smooth elsewhere
    if hop0 <= t <= hop0 + 380:
        u = (t - hop0) / 380
        p['oy'] = -125.0 * 4 * u * (1 - u)
    else:
        p['oy'] = 0.0
    # life: a gentle breath / talk bob on the head
    p['head'] += 0.8 * math.sin(t / 1000 * 2 * math.pi * 0.55)
    return p

# ---------------------------------------------------------------- elements
els = {}
tracks = []

def kf(vals, ease='linear'):
    out = []
    for i, (t, v) in enumerate(vals):
        r = {'t': t, 'v': v}
        if i:
            r['ease'] = ease
        out.append(r)
    return out

def dedupe(vals):
    """Drop interior keys equal to both neighbours."""
    out = []
    for i, kv in enumerate(vals):
        if 0 < i < len(vals) - 1 and vals[i - 1][1] == kv[1] == vals[i + 1][1]:
            continue
        out.append(kv)
    return out

frames = [pose_world(pose_at(ms(f))) for f in range(NF + 1)]

def part_anim(part, f0=0, f1=NF):
    """x, y, rotation keyframes for a part (pivot at its canvas centre)."""
    src = part if part in frames[0] else 'head'
    xs, ys, rs = [], [], []
    for f in range(f0, max(f0 + 1, f1)):
        (x, y), r = frames[f][src]
        xs.append((ms(f), round(x)))
        ys.append((ms(f), round(y)))
        rs.append((ms(f), round(r, 2)))
    def pack(v, scalar):
        v = dedupe(v)
        return v[0][1] if len(v) == 1 else kf(v)
    return pack(xs, 1), pack(ys, 1), pack(rs, 1)

def image(id_, src, start, end, w, h, **kw):
    e = {'id': id_, 'type': 'image', 'start': start, 'end': end}
    if 'layer' in kw:
        e['layer'] = kw.pop('layer')
    e['source'] = src
    for k in ('x', 'y', 'origin'):
        if k in kw:
            e[k] = kw.pop(k)
    e['width'] = w; e['height'] = h; e['fit'] = kw.pop('fit', 'literal')
    for k in ('clip', 'scale', 'rotation', 'opacity', 'effects'):
        if k in kw:
            e[k] = kw.pop(k)
    assert not kw, kw
    return e

def owl_part(part, start=0, end=DUR, id_=None, source=None):
    p = P[part]
    f0 = max(0, start * FPS // 1000)
    f1 = min(NF, -(-end * FPS // 1000))
    x, y, r = part_anim(part if part in ('torso', 'head', 'upper_arm_left', 'upper_arm_right', 'forearm_left', 'forearm_right') else 'head', f0, f1)
    return image(id_ or part, source or 'character/' + p['file'], start, end, p['width'], p['height'],
                 x=x, y=y, origin='center', scale=[S, S], rotation=r)

def track(name, layer, elements):
    tracks.append({'name': name, 'layer': layer, 'elements': elements})

# set
track('set', 0, [image('study', 'character/study.png', 0, DUR, 1920, 1080, x=0, y=0, origin='top-left')])

# soft contact shadows
shadow_s = []
for f in range(NF + 1):
    oy = pose_at(ms(f))['oy']
    k = 1 - 0.45 * min(1, -oy / 125)
    shadow_s.append((ms(f), [round(k, 3), round(k, 3)]))
shadow_s = dedupe(shadow_s)
track('owl-shadow', 5, [
    {'id': 'owl-shadow', 'type': 'ellipse', 'start': 0, 'end': DUR, 'x': OWL_X, 'y': FEET_Y - 6,
     'origin': 'center', 'width': 300, 'height': 34, 'fill': '#5A3A1E40',
     'scale': kf(shadow_s) if len(shadow_s) > 1 else shadow_s[0][1]}])
track('crate-shadow', 1, [
    {'id': 'crate-shadow', 'type': 'ellipse', 'start': 0, 'end': DUR, 'x': CRATE['x'] + CRATE['w'] // 2,
     'y': CRATE['y'] + CRATE['h'] - 2, 'origin': 'center', 'width': CRATE['w'] + 70, 'height': 30,
     'fill': '#5A3A1E40'},
])

# the crate: plain, flat, outlined like the owl
INK = '#3B2F2C'
cx, cy, cw, ch = CRATE['x'], CRATE['y'], CRATE['w'], CRATE['h']
crate = [
    {'id': 'crate', 'type': 'rect', 'start': 0, 'end': DUR, 'x': cx, 'y': cy, 'origin': 'top-left', 'width': cw, 'height': ch,
     'fill': '#C8935A', 'stroke': INK, 'stroke_width': 6, 'radius': 10},
]
track('crate', 2, crate)
slats = []
for i, fy in enumerate((0.36, 0.68)):
    slats.append({'id': f'crate-slat-{i+1}', 'type': 'rect', 'start': 0, 'end': DUR, 'x': cx + 3,
                  'y': cy + round(ch * fy) - 2, 'origin': 'top-left', 'width': cw - 6, 'height': 5, 'fill': INK})
for i, sl in enumerate(slats):
    track(sl['id'], 3, [sl])
track('crate-lip', 4, [
    {'id': 'crate-lip', 'type': 'rect', 'start': 0, 'end': DUR, 'x': cx + 3, 'y': cy + 3, 'origin': 'top-left', 'width': cw - 6,
     'height': 16, 'fill': '#DDA86C'}])

# laptop and its screen
track('laptop', 6, [image('laptop', 'character/laptop.png', 0, DUR, LAP_W, LAP_H, x=LAP_X, y=LAP_Y, origin='top-left')])
SWITCH = t_beat[1] + 30
SW_F = -(-SWITCH * FPS // 1000)
SWITCH = ms(SW_F)
# session still: 960x540 covering the clip
cw_, ch_ = CLIP[2], CLIP[3]
sc = max(cw_ / 960, ch_ / 540)
iw, ih = int(960 * sc + 1e-9), int(540 * sc + 1e-9)
assert iw >= cw_ and ih >= ch_
track('screen', 7, [
    image('screen-session', 'stills/session-02.png', 0, SWITCH, iw, ih, x=CLIP[0] + cw_ // 2, y=CLIP[1] + ch_ // 2,
          origin='center', fit='cover', clip=CLIP),
    {'id': 'screen-paper', 'type': 'rect', 'start': SWITCH, 'end': DUR, 'x': CLIP[0], 'y': CLIP[1], 'origin': 'top-left',
     'width': cw_, 'height': ch_, 'fill': '#F5F0E6'},
])
MS = round(ch_ * 0.78)
track('screen-mark', 8, [
    image('screen-mark', 'brand/mark.png', SWITCH, DUR, MS, MS, x=CLIP[0] + cw_ // 2, y=CLIP[1] + ch_ // 2,
          origin='center', scale=kf([(SWITCH, [0.6, 0.6]), (SWITCH + 200, [1.0, 1.0])], 'ease-out'),
          opacity=kf([(SWITCH, 0.0), (SWITCH + 100, 1.0)], 'linear')),
])

# ---------------------------------------------------------------- owl
OWL_L = 20
order = ['torso', 'forearm_right', 'upper_arm_right', 'head']
for i, part in enumerate(order):
    track('owl-' + part.replace('_', '-'), OWL_L + i, [owl_part(part)])

# mouths, frame by frame from the visemes
vis = [(T(t), rig['visemes'][str(v)]) for t, v in line['visemes']]
def mouth_at(t):
    m = None
    for tt, mm in vis:
        if tt <= t:
            m = mm
        else:
            break
    if t >= T(line['duration_ms']):
        m = None
    return m
runs = []
for f in range(NF):
    m = mouth_at(ms(f) + 0.34)      # the frame's own instant
    if runs and runs[-1][2] == m:
        runs[-1][1] = f + 1
    else:
        runs.append([f, f + 1, m])
mouths = []
for f0, f1, m in runs:
    if m is None:
        continue
    mouths.append(owl_part('mouth_' + m, ms(f0), ms(f1), id_=f'mouth-{len(mouths)+1:02d}-{m}'))
track('owl-mouth', OWL_L + 4, mouths)
print('mouth runs', [(ms(a), ms(b), m) for a, b, m in runs])

# blinks: 3 frames each
BLINKS = [700, 2480, 4160, 6920]
blinks = []
for i, b in enumerate(BLINKS):
    f0 = b * FPS // 1000
    blinks.append(owl_part('eyes_closed', ms(f0), ms(f0 + 3), id_=f'blink-{i+1}'))
track('owl-blink', OWL_L + 5, blinks)

# left arm in front of the face, so the hand can reach the beak
track('owl-forearm-left', OWL_L + 6, [owl_part('forearm_left')])
track('owl-upper-arm-left', OWL_L + 7, [owl_part('upper_arm_left')])

# ---------------------------------------------------------------- the tag
TAG_X = LAP_X + LAP_W // 2
TAG_Y = LAP_Y - 70
def snap(t):
    return ms(-(-t * FPS // 1000))
POP = snap(t_two[0])
FIX = snap(t_fixed[0])
def pop(t0, big=1.18):
    return kf([(t0, [0.0, 0.0]), (t0 + 160, [big, big]), (t0 + 280, [1.0, 1.0])], 'ease-out')
def repop(t0):
    return kf([(t0, [0.85, 0.85]), (t0 + 120, [1.12, 1.12]), (t0 + 240, [1.0, 1.0])], 'ease-out')
TAG_W, TAG_H = 300, 96
track('tag', 40, [
    {'id': 'tag-late', 'type': 'rect', 'start': POP, 'end': FIX, 'x': TAG_X, 'y': TAG_Y, 'origin': 'center',
     'width': TAG_W, 'height': TAG_H, 'fill': '#FF5A36', 'stroke': '#101418', 'stroke_width': 5,
     'radius': 48, 'scale': pop(POP)},
    {'id': 'tag-fixed', 'type': 'rect', 'start': FIX, 'end': DUR, 'x': TAG_X, 'y': TAG_Y, 'origin': 'center',
     'width': TAG_W, 'height': TAG_H, 'fill': '#FF5A36', 'stroke': '#101418', 'stroke_width': 5,
     'radius': 48, 'scale': repop(FIX)},
])
def label(id_, text, start, end, sc):
    return {'id': id_, 'type': 'text', 'start': start, 'end': end, 'x': TAG_X, 'y': TAG_Y, 'origin': 'center',
            'width': TAG_W - 40, 'height': 68, 'font': 'inter-bold', 'size': 56, 'color': '#F5F0E6',
            'runs': [{'text': text}], 'scale': sc, 'caption': False}
track('tag-text', 41, [
    label('tag-late-text', '+2 frames', POP, FIX, pop(POP)),
    label('tag-fixed-text', '0 frames', FIX, DUR, repop(FIX)),
])

# ---------------------------------------------------------------- sound
track('voice', 0, [
    {'id': 'line-3', 'type': 'audio', 'start': VO, 'end': VO + line['duration_ms'] + 0,
     'source': 'character/voice/line-3.wav', 'source_start': 0, 'source_end': line['duration_ms']},
])
track('music', 0, [
    {'id': 'bed', 'type': 'audio', 'start': 0, 'end': DUR, 'source': 'music/bed-120bpm.wav',
     'source_start': 0, 'source_end': DUR,
     'volume': kf([(0, 0.0), (400, 0.10), (DUR - 700, 0.10), (DUR, 0.0)], 'linear')},
])

proj = {
    'frame': {'width': 1920, 'height': 1080},
    'fps': FPS,
    'background': '#F5F0E6',
    'duration': DUR,
    'output': 'deliverable.mp4',
    'fonts': {'inter-bold': [{'file': 'fonts/Inter-Bold.ttf'}]},
    'fontVendor': json.load(open('hoot.json'))['fontVendor'],
    'tracks': tracks,
}
json.dump(proj, open('hoot.json', 'w'))
print('clip', CLIP, 'screen', SCREEN, 'crate', CRATE, 'switch', SWITCH)
