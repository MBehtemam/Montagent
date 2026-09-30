"""Generate hoot.json: Hoot the owl, a cut-out rig baked to flat per-frame keyframes."""
import json, math, os

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
rig = json.load(open(os.path.join(ROOT, 'character/rig.json')))
voice = json.load(open(os.path.join(ROOT, 'character/voice/line-1.json')))

FPS = 30
DUR = 8000
NFRAMES = DUR * FPS // 1000          # frames 0..239
VOICE_AT = 1000                       # the line starts at 1 s
S = 0.6                               # rig scale
HOME_X, FLOOR_Y = 620, 968            # where the feet land
CARD_X, CARD_Y = 1370, 440            # card centre
CARD_ROT = -2.0


def ft(n):
    """The integer ms boundary just at-or-before frame n's sample instant."""
    return n * 1000 // FPS


# ---------------------------------------------------------------- easing
def smooth(u):
    u = min(max(u, 0.0), 1.0)
    return u * u * (3 - 2 * u)


def curve(keys, t):
    """keys: [(t, v)] eased with smoothstep between neighbours; clamps."""
    if t <= keys[0][0]:
        return keys[0][1]
    for (t0, v0), (t1, v1) in zip(keys, keys[1:]):
        if t <= t1:
            return v0 + (v1 - v0) * smooth((t - t0) / (t1 - t0))
    return keys[-1][1]


def window(t, a, b, ramp):
    """1 inside [a, b], smoothly ramping over `ramp` s at each side."""
    return smooth((t - a) / ramp) * (1 - smooth((t - (b - ramp)) / ramp))


# ---------------------------------------------------------------- motion
HOPS = [(0.00, 0.27, 110), (0.27, 0.54, 85), (0.54, 0.80, 55)]
START_X = -340
LAND = 0.80

ARM_DOWN = 58.0     # rotation from rest that lowers an arm to the side
ELBOW_REST = 10.0   # a relaxed bend when hanging

# gesture: point the right arm at the card
sh = rig['parts']['upper_arm_right']['pivot']
ft_p = rig['parts']['torso']['pivot']
shoulder_w = (HOME_X + S * (sh[0] - ft_p[0]), FLOOR_Y + S * (sh[1] - ft_p[1]))
aim = math.degrees(math.atan2(CARD_Y - shoulder_w[1], CARD_X - 60 - shoulder_w[0]))
POINT_U = aim - 25.0 + 6.0   # rest direction is 25 deg below horizontal
POINT_F = -8.0

WAVE_UP, WAVE_SWING0, WAVE_SWING1, WAVE_DOWN = 1.00, 1.22, 2.30, 2.62
GEST_UP, GEST_DOWN = 3.52, 3.84
CHEER = 6.82
MOVIE = (VOICE_AT + 5200) / 1000.0


def root(t):
    """(x, y, torso rotation) of the feet pivot."""
    x, y, rot = HOME_X, FLOOR_Y, 0.0
    if t < LAND:
        for i, (a, b, h) in enumerate(HOPS):
            if a <= t < b:
                u = (t - a) / (b - a)
                x0 = START_X + (HOME_X - START_X) * i / 3
                x1 = START_X + (HOME_X - START_X) * (i + 1) / 3
                x = x0 + (x1 - x0) * u
                y = FLOOR_Y - h * 4 * u * (1 - u)
                rot = 7.0 * math.sin(math.pi * u)
        return x, y, rot
    # landing settle: a small rock back and forth
    if t < 1.2:
        rot += -3.0 * math.sin(math.pi * (t - LAND) / 0.4) * (1 - (t - LAND) / 0.4)
    # idle sway while talking
    rot += 1.0 * math.sin(2 * math.pi * (t - LAND) / 2.3) * smooth((t - LAND) / 0.5)
    # lean toward the card while gesturing
    rot += 2.5 * window(t, GEST_UP, CHEER + 0.1, 0.3)
    # cheer hop
    if 7.00 <= t < 7.30:
        u = (t - 7.00) / 0.30
        y -= 26 * 4 * u * (1 - u)
    return x, y, rot


# head nods on stressed words (timeline seconds)
NODS = [1.05, 1.62, 2.32, 3.98, 4.78, 6.20]


def head(t):
    r = 2.2 * math.sin(2 * math.pi * t / 1.7 + 0.6) * smooth((t - LAND) / 0.4)
    for n in NODS:
        d = t - n
        if -0.1 < d < 0.35:
            r += 3.5 * math.sin(math.pi * (d + 0.1) / 0.45)
    r += 4.0 * window(t, GEST_UP + 0.1, CHEER, 0.3)       # tilt toward the card
    if t < LAND:
        r -= 4.0 * math.sin(math.pi * ((t % 0.27) / 0.27))  # head lags the hop
    return r


def arms(t):
    """(UL, FL, UR, FR): local rotations from rest, degrees clockwise."""
    # during the hop the arms flap
    if t < LAND:
        for a, b, h in HOPS:
            if a <= t < b:
                f = math.sin(math.pi * (t - a) / (b - a))
        ul, fl = -ARM_DOWN + 40 * f, -ELBOW_REST + 20 * f
        ur, fr = ARM_DOWN - 40 * f, ELBOW_REST - 20 * f
        return ul, fl, ur, fr

    down_l = (-ARM_DOWN, -ELBOW_REST)
    down_r = (ARM_DOWN, ELBOW_REST)

    # left arm: wave, then the cheer
    wave_u, wave_f = 26.0, 46.0
    ul = curve([(WAVE_UP, down_l[0]), (WAVE_SWING0, wave_u), (WAVE_SWING1, wave_u),
                (WAVE_DOWN, down_l[0]), (CHEER, down_l[0]), (CHEER + 0.25, 32.0)], t)
    fl = curve([(WAVE_UP, down_l[1]), (WAVE_SWING0, wave_f), (WAVE_SWING1, wave_f),
                (WAVE_DOWN, down_l[1]), (CHEER, down_l[1]), (CHEER + 0.25, 48.0)], t)
    if WAVE_SWING0 - 0.08 < t < WAVE_SWING1:
        k = window(t, WAVE_SWING0 - 0.08, WAVE_SWING1, 0.08)
        ph = 2 * math.pi * (t - WAVE_SWING0) / 0.36
        fl += 22 * math.sin(ph) * k
        ul += 3 * math.sin(ph - 0.8) * k

    # right arm: point at the card, then the cheer
    ur = curve([(GEST_UP, down_r[0]), (GEST_DOWN, POINT_U), (CHEER, POINT_U),
                (CHEER + 0.25, -32.0)], t)
    fr = curve([(GEST_UP, down_r[1]), (GEST_DOWN, POINT_F), (CHEER, POINT_F),
                (CHEER + 0.25, -48.0)], t)
    # a little overshoot as the arm arrives, and an emphasis on "movie"
    ur += -5 * math.sin(math.pi * min(max((t - GEST_DOWN + 0.05) / 0.3, 0), 1))
    ur += -6 * math.sin(math.pi * min(max((t - MOVIE + 0.05) / 0.35, 0), 1))

    # cheer waggle
    if t > CHEER + 0.2:
        k = smooth((t - CHEER - 0.2) / 0.15)
        w = math.sin(2 * math.pi * (t - CHEER - 0.2) / 0.4)
        fl += 10 * w * k
        fr -= 10 * w * k
    return ul, fl, ur, fr


# ---------------------------------------------------------------- kinematics
# The torso part's shoulder is a cut block that a raised arm exposes as a notch and a
# loose flap. So the torso is drawn twice from its own part: an upper copy masked to trim
# the shoulder blocks at x 46..432 (canvas px) above y 212, and a lower copy for the body
# below. An ink plate behind each trimmed side, riding the torso, is its new outline.
# Plates: torso-canvas centre, size, tilt (the canvas's top-left is at drawing (272, 618)).
TORSO_TL = (272, 618)
CUT_L, CUT_R, SPLIT = 46, 432, 212
PATCH = {'patch_left': ((47.5, 133.0), 25, 138, 0.0),
         'patch_right': ((430.5, 133.0), 25, 138, 0.0)}
INK = '#302426'


def pivot(name):
    if name in PATCH:
        c = PATCH[name][0]
        return (TORSO_TL[0] + c[0], TORSO_TL[1] + c[1])
    return rig['parts'][name]['pivot']


def parent(name):
    return 'torso' if name in PATCH else rig['parts'][name]['parent']


def rotv(dx, dy, deg):
    a = math.radians(deg)
    return dx * math.cos(a) - dy * math.sin(a), dx * math.sin(a) + dy * math.cos(a)


LOCAL = {}


def pose(t):
    x, y, trot = root(t)
    ul, fl, ur, fr = arms(t)
    local = {'torso': trot, 'head': head(t), 'upper_arm_left': ul, 'forearm_left': fl,
             'upper_arm_right': ur, 'forearm_right': fr,
             'patch_left': PATCH['patch_left'][3], 'patch_right': PATCH['patch_right'][3]}
    world = {'torso': (x, y, trot)}
    for name in ['head', 'upper_arm_left', 'upper_arm_right', 'forearm_left', 'forearm_right',
                 'patch_left', 'patch_right']:
        par = parent(name)
        px, py, prot = world[par]
        dx = S * (pivot(name)[0] - pivot(par)[0])
        dy = S * (pivot(name)[1] - pivot(par)[1])
        ox, oy = rotv(dx, dy, prot)
        world[name] = (px + ox, py + oy, prot + local[name])
    return world


FRAMES = [pose(n / FPS) for n in range(NFRAMES + 1)]


def kf(values, lo=0, hi=NFRAMES - 1):
    """Per-frame keyframes over frames [lo, hi], collinear-constant runs dropped."""
    lo, hi = max(lo, 0), min(hi, NFRAMES - 1)
    pts = [(ft(n), values[n]) for n in range(lo, hi + 1)]
    out = []
    for i, (t, v) in enumerate(pts):
        if 0 < i < len(pts) - 1 and pts[i - 1][1] == v == pts[i + 1][1]:
            continue
        out.append((t, v))
    while len(out) > 1 and out[-1][1] == out[-2][1]:
        out.pop()                       # clamping already holds the last value
    if len(out) == 1:
        return out[0][1]
    res = [{'t': out[0][0], 'v': out[0][1]}]
    for (_, pv), (t, v) in zip(out, out[1:]):
        res.append({'t': t, 'v': v, 'ease': 'step' if v == pv else 'linear'})
    return res


def channels(name):
    xs = [int(round(f[name][0])) for f in FRAMES]
    ys = [int(round(f[name][1])) for f in FRAMES]
    rs = [round(f[name][2], 2) for f in FRAMES]
    return xs, ys, rs


CH = {n: channels(n) for n in ['torso', 'head', 'upper_arm_left', 'upper_arm_right',
                               'forearm_left', 'forearm_right', 'patch_left', 'patch_right']}


def part_el(eid, part, start, end, follow=None, mask=None):
    follow = follow or part
    p = rig['parts'][part]
    xs, ys, rs = CH[follow]
    lo = -(-start * FPS // 1000)            # the first frame sampled inside [start, end)
    hi = -(-end * FPS // 1000) - 1          # the last
    el = {'id': eid, 'type': 'image', 'group': 'hoot', 'start': start, 'end': end,
            'source': 'character/' + p['file'], 'x': kf(xs, lo, hi), 'y': kf(ys, lo, hi),
            'origin': 'center', 'width': p['width'], 'height': p['height'], 'fit': 'literal',
            'scale': [S, S], 'rotation': kf(rs, lo, hi)}
    if mask:
        el['effects'] = [dict(name='mask', shape='rect', x=mask[0], y=mask[1], width=mask[2],
                              height=mask[3])]
    return el


# ---------------------------------------------------------------- mouths
def mouth_segments():
    vis = {}
    for ms, vid in voice['visemes']:
        vis[ms] = vid                       # a repeated instant: the later id wins
    seq = sorted(vis.items())
    segs = []                               # (start_frame, end_frame, mouth)
    for i, (ms, vid) in enumerate(seq):
        end_ms = seq[i + 1][0] if i + 1 < len(seq) else voice['duration_ms']
        mouth = rig['visemes'][str(vid)]
        a = round((VOICE_AT + ms) * FPS / 1000)
        b = round((VOICE_AT + end_ms) * FPS / 1000)
        if b <= a:
            continue
        # a shape shorter than a frame is absorbed by the one before it
        segs.append([a, b, mouth])
    merged = []
    for a, b, m in segs:
        if merged and merged[-1][1] >= a and merged[-1][2] == m:
            merged[-1][1] = b
        elif merged and merged[-1][1] > a:
            merged.append([merged[-1][1], b, m])
        else:
            merged.append([a, b, m])
    return [(a, b, m) for a, b, m in merged if m is not None and b > a]


MOUTHS = mouth_segments()
BLINKS = [0.93, 2.57, 4.62, 5.95, 7.55]
BLINK_FRAMES = 4

# ---------------------------------------------------------------- assemble
tracks = []


def track(name, layer, elements):
    tracks.append({'name': name, 'layer': layer, 'elements': elements})


track('set', 0, [{'id': 'study', 'type': 'image', 'start': 0, 'end': DUR,
                  'source': 'character/study.png', 'x': 0, 'y': 0, 'origin': 'top-left', 'width': 1920,
                  'height': 1080, 'fit': 'cover'}])

# the owl's contact shadow follows the feet and shrinks as it leaves the floor
sx = [int(round(f['torso'][0])) for f in FRAMES]
lift = [FLOOR_Y - f['torso'][1] for f in FRAMES]
ssc = [round(max(0.5, 1 - l / 220), 3) for l in lift]
track('shadow', 1, [{'id': 'owl-shadow', 'type': 'ellipse', 'group': 'hoot', 'start': 0,
                     'end': DUR, 'x': kf(sx), 'y': FLOOR_Y + 4, 'origin': 'center',
                     'width': 380, 'height': 46, 'fill': '#5A3418',
                     'scale': kf([[v, v] for v in ssc]),
                     'opacity': kf([round(0.32 * v, 3) for v in ssc]),
                     'effects': [{'name': 'blur', 'radius': 10}]}])

# the card, behind the owl
POP = [(3620, 0.02), (3860, 1.1), (3990, 0.96), (4100, 1.0), (6200, 1.0), (6310, 1.05),
       (6470, 1.0)]
pop_scale = [{'t': POP[0][0], 'v': [POP[0][1]] * 2}]
for i, (t, v) in enumerate(POP[1:], 1):
    pop_scale.append({'t': t, 'v': [v, v], 'ease': 'ease-out' if i == 1 else
                      ('step' if v == POP[i - 1][1] else 'ease-in-out')})
pop_opacity = [{'t': 3620, 'v': 0.0}, {'t': 3700, 'v': 1.0, 'ease': 'linear'}]
CARD_START = 3620
card_common = {'x': CARD_X, 'y': CARD_Y, 'origin': 'center'}
track('card', 5, [dict(id='card-frame', type='rect', group='card', start=CARD_START, end=DUR,
                       **card_common, width=756, height=441, fill='#F5F0E6', stroke='#101418',
                       stroke_width=4, radius=22, scale=pop_scale, rotation=CARD_ROT,
                       opacity=pop_opacity,
                       effects=[{'name': 'shadow', 'dx': 0, 'dy': 14, 'radius': 22,
                                 'color': '#101418', 'opacity': 0.28}])])
MOVIE_MS = VOICE_AT + 5200
pic_mask = [{'name': 'mask', 'shape': 'rect', 'radius': 10}]
track('card-picture', 6, [
    dict(id='card-session-01', type='image', group='card', start=CARD_START, end=MOVIE_MS,
         source='stills/session-01.png', **card_common, width=720, height=405, fit='cover',
         scale=pop_scale, rotation=CARD_ROT, opacity=pop_opacity, effects=pic_mask),
    dict(id='card-session-02', type='image', group='card', start=MOVIE_MS, end=DUR,
         source='stills/session-02.png', **card_common, width=720, height=405, fit='cover',
         scale=pop_scale, rotation=CARD_ROT, effects=pic_mask),
])

patches = []
for name in ['patch_left', 'patch_right']:
    xs, ys, rs = CH[name]
    _, pw, ph, _ = PATCH[name]
    patches.append({'id': 'owl-shoulder-' + name[6:], 'type': 'rect', 'group': 'hoot', 'start': 0,
                    'end': DUR, 'x': kf(xs), 'y': kf(ys), 'origin': 'center', 'width': pw,
                    'height': ph, 'fill': INK, 'radius': 8, 'scale': [S, S], 'rotation': kf(rs)})
track('shoulder-plate-left', 7, [patches[0]])
track('shoulder-plate-right', 8, [patches[1]])
track('torso-lower', 9, [part_el('owl-torso-lower', 'torso', 0, DUR,
                                 mask=(0, SPLIT - 2, 480, 1408 - SPLIT + 2))])
track('torso-upper', 10, [part_el('owl-torso-upper', 'torso', 0, DUR,
                                  mask=(CUT_L, 0, CUT_R - CUT_L, SPLIT + 2))])

# the rest of the rig, in draw order
layer = 11
for name in ['forearm_left', 'forearm_right', 'upper_arm_left', 'upper_arm_right',
             'head']:
    track(name.replace('_', '-'), layer, [part_el('owl-' + name.replace('_', '-'), name, 0, DUR)])
    layer += 1

track('mouth', layer, [part_el('mouth-%02d-%s' % (i, m), 'mouth_' + m, ft(a), ft(b), 'head')
                       for i, (a, b, m) in enumerate(MOUTHS)])
layer += 1
blink_els = []
for i, b in enumerate(BLINKS):
    a = round(b * FPS)
    blink_els.append(part_el('blink-%d' % i, 'eyes_closed', ft(a), ft(a + BLINK_FRAMES), 'head'))
track('eyes', layer, blink_els)

# the lockup lands above the owl at about 7 s
LOCK_Y = 128
track('lockup', 30, [{'id': 'lockup', 'type': 'image', 'group': 'brand', 'start': 6900,
                      'end': DUR, 'source': 'brand/lockup.png', 'x': HOME_X,
                      'y': [{'t': 6900, 'v': -140}, {'t': 7130, 'v': LOCK_Y + 22, 'ease': 'ease-in'},
                            {'t': 7260, 'v': LOCK_Y - 8, 'ease': 'ease-out'},
                            {'t': 7380, 'v': LOCK_Y, 'ease': 'ease-in-out'}],
                      'origin': 'center', 'width': 780, 'height': 180, 'fit': 'contain',
                      'scale': [{'t': 7100, 'v': [1.0, 1.0]},
                                {'t': 7160, 'v': [1.06, 0.92], 'ease': 'ease-out'},
                                {'t': 7300, 'v': [1.0, 1.0], 'ease': 'ease-in-out'}]}])

track('voice', 40, [{'id': 'line-1', 'type': 'audio', 'start': VOICE_AT, 'end': VOICE_AT + 6048,
                     'source': 'character/voice/line-1.wav', 'source_start': 0,
                     'source_end': 6048}])
track('music', 41, [{'id': 'bed', 'type': 'audio', 'start': 0, 'end': DUR,
                     'source': 'music/bed-120bpm.wav', 'source_start': 0, 'source_end': DUR,
                     'volume': [{'t': 0, 'v': 0.1}, {'t': 7300, 'v': 0.1, 'ease': 'linear'},
                                {'t': 7966, 'v': 0.0, 'ease': 'ease-in'}]}])

proj = json.load(open(os.path.join(ROOT, 'hoot.json')))
proj['tracks'] = tracks
json.dump(proj, open(os.path.join(ROOT, 'hoot.json'), 'w'), indent=2, ensure_ascii=False)
print('mouth segments', len(MOUTHS), 'point angle', round(POINT_U, 1))
