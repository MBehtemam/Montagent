#!/usr/bin/env python3
"""Generates hoot.montagent.json: Hoot explains editing a word (brief H6)."""
import json, math

FPS = 30
DUR = 10000
NF = DUR * FPS // 1000
def ft(n):  # integer instant of frame n (floor), safe for half-open ranges
    return (n * 1000) // FPS

rig = json.load(open('character/rig.json'))
line = json.load(open('character/voice/line-2.json'))
VO = 1000  # voice starts here
VO_END = VO + line['duration_ms']

# ---------- owl placement ----------
S = 0.58             # owl scale: drawing 156..1331 tall -> ~682 px
FLOOR_Y = 1000       # feet (torso pivot) on the floor
HOME_X = 470

def ease(u):  # smooth in-out
    u = max(0.0, min(1.0, u))
    return u * u * (3 - 2 * u)

def track(keys, t):
    """keys: list of (t, value); eased between, clamped at the ends."""
    if t <= keys[0][0]:
        return keys[0][1]
    for (t0, v0), (t1, v1) in zip(keys, keys[1:]):
        if t < t1:
            return v0 + (v1 - v0) * ease((t - t0) / (t1 - t0))
    return keys[-1][1]

# Local joint angles, degrees clockwise. Left/right are the viewer's.
# Left upper arm: + raises. Right upper arm: - raises.
# Left forearm: + bends up (flexion). Right forearm: - bends up.
IDLE = dict(T=0, H=0, UL=-40, FL=10, UR=40, FR=-10)
POSE = {
    'T':  [(0, 6), (800, 6), (900, -3), (1050, 0), (1250, -3), (2550, -3), (2900, 0),
           (3200, 4), (4800, 4), (5200, 3), (6200, 3), (6600, 0), (7000, 0), (7300, -2), (8400, -2), (8900, 0)],
    'H':  [(0, 0), (900, 0), (1350, 8), (2050, 8), (2250, 10), (2550, 8), (2950, 0),
           (3250, 8), (4800, 8), (5250, 4), (6200, 4), (6600, 0), (7000, 0), (7350, -7), (8400, -7), (8900, 4)],
    'UL': [(0, 20), (800, 20), (950, -40), (1100, -40), (1400, -6), (2550, -6), (2950, -40),
           (6900, -40), (7300, -8), (8400, -8), (8900, -30)],
    'FL': [(0, 30), (800, 30), (950, 10), (1100, 10), (1400, 62), (2550, 62), (2950, 10),
           (6900, 10), (7300, 40), (8400, 40), (8900, 20)],
    'UR': [(0, -20), (800, -20), (950, 40), (1100, 40), (1400, 6), (2550, 6), (2950, 40),
           (3150, 40), (3500, -29), (3700, -25), (3850, -29), (4900, -29), (5300, -62), (6200, -62),
           (6650, 40), (6900, 40), (7300, -35), (8400, -35), (8900, 30)],
    'FR': [(0, -30), (800, -30), (950, -10), (1100, -10), (1400, -62), (2550, -62), (2950, -10),
           (3150, -10), (3500, -4), (4900, -4), (5300, -8), (6200, -8),
           (6650, -10), (6900, -10), (7300, -30), (8400, -30), (8900, -20)],
}
# beats inside the question: a small nod-bounce on "word" and on "video?"
def beat(t, centre, width, amp):
    d = (t - centre) / width
    return amp * math.exp(-d * d)

def pose_at(t):
    p = {k: track(v, t) for k, v in POSE.items()}
    s = t / 1000.0
    # alive: gentle breathing / sway, never zero
    p['T'] += 1.2 * math.sin(2 * math.pi * s / 2.2)
    p['H'] += 1.5 * math.sin(2 * math.pi * s / 1.7 + 1.0)
    p['UL'] += 2.5 * math.sin(2 * math.pi * s / 2.0)
    p['UR'] -= 2.5 * math.sin(2 * math.pi * s / 2.3 + 0.5)
    p['FL'] += 3.0 * math.sin(2 * math.pi * s / 1.9 + 0.3)
    p['FR'] -= 3.0 * math.sin(2 * math.pi * s / 2.1 + 0.9)
    # question emphasis: palms lift on "word" and "video?"
    for c in (VO + 900, VO + 1300):
        b = beat(t, c, 110, 1.0)
        p['FL'] += 10 * b; p['FR'] -= 10 * b; p['T'] -= 1.5 * b
    # "edit": a little jab of the pointing arm
    p['UR'] -= 5 * beat(t, VO + 2450, 90, 1.0)
    # forearms never hyperextend
    p['FL'] = max(p['FL'], 0.0)
    p['FR'] = min(p['FR'], 0.0)
    # position: two hops in from the left, then planted
    if t < 800:
        u = t / 800.0
        x = -380 + (HOME_X + 380) * (1 - (1 - u) ** 1.6)
        ph = (t % 400) / 400.0
        y = FLOOR_Y - 45 * math.sin(math.pi * ph)
    else:
        x, y = HOME_X, FLOOR_Y
    return p, x, y

def rot(v, deg):
    a = math.radians(deg)
    c, s_ = math.cos(a), math.sin(a)
    return (v[0] * c - v[1] * s_, v[0] * s_ + v[1] * c)

PARTS = rig['parts']
ANGLE = {'torso': 'T', 'head': 'H', 'upper_arm_left': 'UL', 'forearm_left': 'FL',
         'upper_arm_right': 'UR', 'forearm_right': 'FR'}

def fk(t):
    p, gx, gy = pose_at(t)
    world = {}
    order = ['torso', 'head', 'upper_arm_left', 'upper_arm_right', 'forearm_left', 'forearm_right']
    for name in order:
        part = PARTS[name]
        par = part['parent']
        if par is None:
            pos, ang = (gx, gy), p[ANGLE[name]]
        else:
            ppos, pang = world[par]
            d = (part['pivot'][0] - PARTS[par]['pivot'][0], part['pivot'][1] - PARTS[par]['pivot'][1])
            r = rot((d[0] * S, d[1] * S), pang)
            pos, ang = (ppos[0] + r[0], ppos[1] + r[1]), pang + p[ANGLE[name]]
        world[name] = (pos, ang)
    return world

WORLD = [fk(ft(n)) for n in range(NF + 1)]
print('head range', min(w['head'][1] for w in WORLD), max(w['head'][1] for w in WORLD))

def kf_list(vals_by_frame, n0, n1, fmt):
    """Per-frame keyframes for frames n0..n1 inclusive."""
    out = []
    for n in range(n0, n1 + 1):
        rec = {'t': ft(n), 'v': fmt(vals_by_frame(n))}
        if out:
            rec['ease'] = 'linear'
        out.append(rec)
    return out

def rig_element(eid, part_name, follow, n0, n1, layer=None, opacity=None):
    part = PARTS[part_name]
    el = {'id': eid, 'type': 'image', 'start': ft(n0), 'end': ft(n1)}
    if layer is not None:
        el['layer'] = layer
    el['source'] = 'character/' + part['file']
    k0, k1 = max(n0 - 1, 0), min(n1, NF)
    el['x'] = kf_list(lambda n: WORLD[n][follow][0][0], k0, k1, lambda v: int(round(v)))
    el['y'] = kf_list(lambda n: WORLD[n][follow][0][1], k0, k1, lambda v: int(round(v)))
    el['origin'] = 'center'
    el['width'] = part['width']
    el['height'] = part['height']
    el['fit'] = 'literal'
    el['scale'] = [S, S]
    el['rotation'] = kf_list(lambda n: WORLD[n][follow][1], k0, k1, lambda v: round(v, 2))
    if opacity is not None:
        el['opacity'] = opacity
    return el

tracks = []
def add_track(name, layer, elements):
    tracks.append({'name': name, 'layer': layer, 'elements': elements})

# ---------- set ----------
add_track('study', 0, [{'id': 'study', 'type': 'image', 'start': 0, 'end': DUR, 'source': 'character/study.png',
                        'x': 0, 'y': 0, 'origin': 'top-left', 'width': 1920, 'height': 1080, 'fit': 'literal'}])

# ---------- the illustration: laptop (the file) -> video card ----------
L = 0.9
LW, LH = round(708 * L), round(566 * L)
LX, LY = 1118, FLOOR_Y  # bottom-center on the floor
LX0, LY0 = LX - LW // 2, LY - LH
IN_T, OUT_T = 2600, 6550   # laptop pops in during the pause, leaves after the line
FADE = 250
def pop_scale(t0, dur=300, over=1.08):
    return [{'t': t0, 'v': [0.0, 0.0]},
            {'t': t0 + int(dur * 0.65), 'v': [over, over], 'ease': 'ease-out'},
            {'t': t0 + dur, 'v': [1.0, 1.0], 'ease': 'ease-in-out'}]
def fade_out(t_end, dur=FADE):
    return [{'t': t_end - dur, 'v': 1.0}, {'t': t_end, 'v': 0.0, 'ease': 'ease-in'}]

add_track('laptop', 10, [{'id': 'laptop', 'type': 'image', 'start': IN_T, 'end': OUT_T,
                          'source': 'character/laptop.png', 'x': LX, 'y': LY, 'origin': 'bottom-center',
                          'width': LW, 'height': LH, 'fit': 'literal', 'scale': pop_scale(IN_T),
                          'opacity': fade_out(OUT_T)}])

# screen rectangle in frame space (inset 3 px from the bezel)
SX0 = LX0 + round(120 * L) + 3
SY0 = LY0 + round(41 * L) + 3
SX1 = LX0 + round(587 * L) - 3
SY1 = LY0 + round(312 * L) - 3
SW, SH = SX1 - SX0, SY1 - SY0
# show the diff region of the terminal still: source x 110.., y 230.., width chosen to fill the screen
CROP_X, CROP_Y, CROP_W = 110, 232, 700
k = SW / CROP_W
SCREEN_ON = IN_T + 350
add_track('screen', 11, [{'id': 'screen-diff', 'type': 'image', 'start': SCREEN_ON, 'end': OUT_T,
                          'source': 'stills/session-01.png', 'x': round(SX0 - CROP_X * k), 'y': round(SY0 - CROP_Y * k), 'origin': 'top-left',
                          'width': round(1920 * k), 'height': round(1080 * k), 'fit': 'literal',
                          'clip': [SX0, SY0, SW, SH],
                          'opacity': [{'t': SCREEN_ON, 'v': 0.0}, {'t': SCREEN_ON + 150, 'v': 1.0, 'ease': 'ease-out'},
                                      {'t': OUT_T - FADE, 'v': 1.0, 'ease': 'linear'}, {'t': OUT_T, 'v': 0.0, 'ease': 'ease-in'}]}])

# the edit: an outline round the changed line ("text":"Video as a document"), and a caret
def src_to_frame(x, y):
    return round(SX0 + (x - CROP_X) * k), round(SY0 + (y - CROP_Y) * k)
EDIT_T = VO + 2387   # "edit"
hx0, hy0 = src_to_frame(279, 578)
hx1, hy1 = src_to_frame(632, 612)
cx_caret, _ = src_to_frame(609, 0)
add_track('edit-mark', 12, [{'id': 'edit-box', 'type': 'rect', 'start': EDIT_T, 'end': OUT_T,
                             'x': hx0 - 4, 'y': hy0, 'origin': 'top-left', 'width': min(hx1 - hx0 + 8, SX1 - hx0), 'height': hy1 - hy0,
                             'stroke': '#FF5A36', 'stroke_width': 3, 'radius': 4,
                             'opacity': [{'t': EDIT_T, 'v': 0.0}, {'t': EDIT_T + 120, 'v': 1.0, 'ease': 'ease-out'},
                                         {'t': OUT_T - FADE, 'v': 1.0, 'ease': 'linear'}, {'t': OUT_T, 'v': 0.0, 'ease': 'ease-in'}]}])
caret = []
c_on = EDIT_T + 120
i = 0
while c_on < OUT_T - FADE:
    c_off = min(c_on + 270, OUT_T - FADE)
    caret.append({'id': f'caret-{i:02d}', 'type': 'rect', 'start': c_on, 'end': c_off,
                  'x': cx_caret, 'y': hy0 + 4, 'origin': 'top-left', 'width': 3, 'height': hy1 - hy0 - 8, 'fill': '#FF5A36'})
    c_on = c_off + 230
    i += 1
add_track('caret', 13, caret)

# video card: flies out of the laptop screen to the upper right
CARD_CX, CARD_CY = LX, 240
CW, CH = 592, 333
FRAME_PAD = 14
FLY0, FLY1 = VO + 3900, VO + 4400   # starts during "Montagent", lands on "renders"
SCX, SCY = (SX0 + SX1) / 2, (SY0 + SY1) / 2
s0 = 0.25
def card_scale_keys():
    return [{'t': FLY0, 'v': [s0, s0]}, {'t': FLY1, 'v': [1.0, 1.0], 'ease': 'ease-out'}]
def card_xy_keys(dx, dy):
    """dx,dy: offset of the child's centre from the card centre at full scale."""
    xs = [{'t': FLY0, 'v': round(SCX + dx * s0)}, {'t': FLY1, 'v': round(CARD_CX + dx), 'ease': 'ease-out'}]
    ys = [{'t': FLY0, 'v': round(SCY + dy * s0)}, {'t': FLY1, 'v': round(CARD_CY + dy), 'ease': 'ease-out'}]
    return xs, ys
# NB the centre path and the scale share one ease, so children stay locked to the card.
BAR_H = 8
bar_y_off = CH / 2 - 22
card_els = []
fx, fy = card_xy_keys(0, 0)
card_els.append({'id': 'card-frame', 'type': 'rect', 'start': FLY0, 'end': OUT_T, 'x': fx, 'y': fy, 'origin': 'center',
                 'width': CW + 2 * FRAME_PAD, 'height': CH + 2 * FRAME_PAD, 'fill': '#101418', 'stroke': '#F5F0E6',
                 'stroke_width': 4, 'radius': 18, 'scale': card_scale_keys(), 'opacity': fade_out(OUT_T),
                 'effects': [{'name': 'shadow', 'dx': 0, 'dy': 10, 'radius': 18, 'color': '#3A2A1A', 'opacity': 0.35}]})
add_track('card-frame', 20, card_els)
add_track('card-video', 21, [{'id': 'card-video', 'type': 'image', 'start': FLY0, 'end': OUT_T,
                              'source': 'stills/session-02.png', 'x': fx, 'y': fy, 'origin': 'center',
                              'width': CW, 'height': CH, 'fit': 'literal', 'scale': card_scale_keys(),
                              'opacity': fade_out(OUT_T)}])
# a playback bar: track and Signal progress, playing once the card lands
bx, by = card_xy_keys(0, bar_y_off)
add_track('card-bar', 22, [{'id': 'card-bar-track', 'type': 'rect', 'start': FLY0, 'end': OUT_T, 'x': bx, 'y': by,
                            'origin': 'center', 'width': CW - 60, 'height': BAR_H, 'fill': '#F5F0E655', 'radius': 4,
                            'scale': card_scale_keys(), 'opacity': fade_out(OUT_T)}])
PLAY0 = FLY1
bar_left = CARD_CX - (CW - 60) / 2
add_track('card-progress', 23, [{'id': 'card-progress', 'type': 'rect', 'start': PLAY0, 'end': OUT_T,
                                 'x': round(bar_left), 'y': round(CARD_CY + bar_y_off), 'origin': 'center-left',
                                 'width': CW - 60, 'height': BAR_H, 'fill': '#FF5A36', 'radius': 4,
                                 'scale': [{'t': PLAY0, 'v': [0.02, 1.0]}, {'t': OUT_T - FADE, 'v': [1.0, 1.0], 'ease': 'linear'}],
                                 'opacity': fade_out(OUT_T)}])
# a dotted trail from the screen to the card, drawn as the card flies
trail = []
TRAIL_N = 3
ax, ay = LX, LY0 - 2
bx2, by2 = LX, CARD_CY + CH / 2 + FRAME_PAD + 2
for j in range(TRAIL_N):
    u = (j + 1) / (TRAIL_N + 1)
    px, py = ax + (bx2 - ax) * u, ay + (by2 - ay) * u
    t0 = FLY0 + 80 + j * 70
    trail.append({'id': f'trail-{j}', 'type': 'ellipse', 'start': t0, 'end': OUT_T, 'x': round(px), 'y': round(py),
                  'origin': 'center', 'width': 16, 'height': 16, 'fill': '#FF5A36',
                  'scale': [{'t': t0, 'v': [0.0, 0.0]}, {'t': t0 + 150, 'v': [1.0, 1.0], 'ease': 'ease-out'}],
                  'opacity': fade_out(OUT_T)})
for j, el in enumerate(trail):
    add_track(f'trail-{j}', 16 + j, [el])

# ---------- brand ----------
BRAND_T = 6900
BW = 740
BH = round(623 * BW / 2694)
add_track('brand', 15, [{'id': 'lockup', 'type': 'image', 'start': BRAND_T, 'end': DUR, 'source': 'brand/lockup.png',
                         'x': 1180, 'y': 430, 'origin': 'center', 'width': BW, 'height': BH, 'fit': 'literal',
                         'scale': pop_scale(BRAND_T, 380, 1.06),
                         'opacity': [{'t': BRAND_T, 'v': 0.0}, {'t': BRAND_T + 200, 'v': 1.0, 'ease': 'ease-out'}]}])

# ---------- the owl ----------
LAYER0 = 30
for i, name in enumerate(['torso', 'forearm_left', 'forearm_right', 'upper_arm_left', 'upper_arm_right', 'head']):
    add_track('owl-' + name.replace('_', '-'), LAYER0 + i, [rig_element('owl-' + name.replace('_', '-'), name, name, 0, NF)])

# mouths: sample the viseme active at each frame, merge runs of the same mouth
vis = line['visemes']
def mouth_at(t):
    r = t - VO
    if r < vis[0][0] or r >= line['duration_ms']:
        return None
    cur = None
    for ms, vid in vis:
        if ms <= r:
            cur = vid
        else:
            break
    m = rig['visemes'][str(cur)]
    return m
runs = []
for n in range(NF):
    m = mouth_at(ft(n))
    if runs and runs[-1][0] == m and runs[-1][2] == n:
        runs[-1][2] = n + 1
    else:
        runs.append([m, n, n + 1])
mouth_els = []
for m, n0, n1 in runs:
    if m is None:
        continue
    mouth_els.append(rig_element(f'mouth-{ft(n0):05d}-{m}', 'mouth_' + m, 'head', n0, n1))
add_track('owl-mouth', LAYER0 + 6, mouth_els)

# blinks: 4 frames each
BLINKS = [1900, 2760, 4500, 6450, 7900, 9300]
blink_els = []
for b in BLINKS:
    n0 = math.ceil(b * FPS / 1000)
    blink_els.append(rig_element(f'blink-{b:05d}', 'eyes_closed', 'head', n0, n0 + 4))
add_track('owl-blink', LAYER0 + 7, blink_els)

# ---------- sound ----------
add_track('voice', 0, [{'id': 'hoot-line-2', 'type': 'audio', 'start': VO, 'end': VO_END,
                        'source': 'character/voice/line-2.wav', 'source_start': 0, 'source_end': line['duration_ms'], 'volume': 1.4}])
MUSIC_END = 9500
add_track('music', 0, [{'id': 'music-bed', 'type': 'audio', 'start': 0, 'end': MUSIC_END,
                        'source': 'music/bed-120bpm.wav', 'source_start': 0, 'source_end': MUSIC_END,
                        'volume': [{'t': 0, 'v': 0.0}, {'t': 300, 'v': 0.22, 'ease': 'ease-out'},
                                   {'t': VO - 150, 'v': 0.22, 'ease': 'linear'}, {'t': VO, 'v': 0.09, 'ease': 'ease-in-out'},
                                   {'t': VO_END, 'v': 0.09, 'ease': 'linear'}, {'t': VO_END + 300, 'v': 0.22, 'ease': 'ease-in-out'},
                                   {'t': 8500, 'v': 0.22, 'ease': 'linear'}, {'t': MUSIC_END, 'v': 0.0, 'ease': 'ease-in'}]}])

project = {'frame': {'width': 1920, 'height': 1080}, 'fps': FPS, 'background': '#F5F0E6', 'duration': DUR,
           'output': 'deliverable.mp4', 'tracks': tracks}
json.dump(project, open('hoot.montagent.json', 'w'), ensure_ascii=False)
print('screen', SX0, SY0, SW, SH, 'k', round(k, 3), 'mouth runs', len(mouth_els))
