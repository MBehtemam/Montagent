"""Generate hoot.montagent.json: Hoot explains editing a word (brief H6).

The format has one flat transform per element, so the rig's hierarchy is solved here:
every owl part gets per-frame x/y/rotation/scale keyframes from forward kinematics.
"""
import json, math

FPS = 30
DUR = 10000
NF = DUR * FPS // 1000
def tf(i): return round(i * 1000 / FPS)
FRAMES = list(range(NF + 1))

rig = json.load(open('character/rig.json'))
voice = json.load(open('character/voice/line-2.json'))
P = {k: v for k, v in rig['parts'].items()}

VO = 1000            # voice starts at 1.0 s on the timeline
S = 0.6              # owl scale
HOME = (560, 968)    # torso pivot (feet) once Hoot has landed
FLOOR_Y = HOME[1]

# ---------------------------------------------------------------- easing helpers
def smooth(u): u = min(1, max(0, u)); return u * u * (3 - 2 * u)

def channel(keys, t):
    """keys: [(sec, value)], eased in-out between keys, clamped at ends."""
    if t <= keys[0][0]: return keys[0][1]
    for (t0, v0), (t1, v1) in zip(keys, keys[1:]):
        if t <= t1: return v0 + (v1 - v0) * smooth((t - t0) / (t1 - t0))
    return keys[-1][1]

def bump(t, c, w):
    """A smooth 0..1..0 bump centred on c, half-width w."""
    u = abs(t - c) / w
    return 0.0 if u >= 1 else 0.5 * (1 + math.cos(math.pi * u))

# ---------------------------------------------------------------- the performance
# Relative joint angles, degrees clockwise. Left/right are the viewer's.
# upper_arm_left: + raises; forearm_left: + lifts the hand (elbow flexion).
# upper_arm_right: - raises; forearm_right: - lifts the hand.
RELAX = dict(uL=-28, fL=10, uR=28, fR=-10, head=0, lean=0)
ASK = dict(uL=14, fL=42, uR=-14, fR=-42, head=8, lean=-2.5)
POINT_FILE = dict(uL=-30, fL=8, uR=-20, fR=-4, head=5, lean=2.5)
POINT_VIDEO = dict(uL=-30, fL=10, uR=-46, fR=-10, head=8, lean=3)
PRESENT_BRAND = dict(uL=-22, fL=14, uR=-30, fR=-14, head=6, lean=1.5)
FLAP = dict(uL=45, fL=10, uR=-45, fR=-10, head=-3, lean=0)

POSES = [
    (0.00, FLAP), (0.20, FLAP), (0.40, RELAX), (0.55, FLAP), (0.62, FLAP), (0.82, RELAX),
    (1.10, RELAX),
    (1.42, ASK), (2.62, dict(ASK, head=10)),          # "Want to change one word in a video?"
    (3.00, POINT_FILE), (4.30, POINT_FILE),           # "Just edit the file,"
    (4.68, POINT_VIDEO), (6.20, POINT_VIDEO),         # "and Montagent renders it again."
    (6.50, RELAX), (6.62, FLAP), (6.80, FLAP), (7.00, RELAX),
    (7.35, PRESENT_BRAND), (8.70, PRESENT_BRAND), (9.20, RELAX), (10.0, RELAX),
]

def pose_at(t):
    out = {}
    for k in RELAX:
        out[k] = channel([(pt, p[k]) for pt, p in POSES], t)
    return out

# beats on stressed words: a little extra lift of the active hand and a nod
BEATS = [(1.05 + 0.10, 'L', 1.0), (1.825, 'L', 0.8), (2.25, 'L', 1.0),
         (3.10, 'R', 0.8), (3.45, 'R', 1.0), (3.80, 'R', 0.9),
         (4.62, 'R', 1.0), (5.25, 'R', 0.8), (5.75, 'R', 1.0)]

# hops: (take-off, landing, from_x, to_x, height)
HOPS = [(0.00, 0.40, -330, 140, 120), (0.40, 0.82, 140, HOME[0], 105), (6.55, 6.92, HOME[0], HOME[0], 55)]

def owl_state(t):
    p = pose_at(t)
    x, y = HOME
    air = 0.0
    sx = sy = 1.0
    for t0, t1, x0, x1, h in HOPS:
        if t0 <= t < t1:
            u = (t - t0) / (t1 - t0)
            x = x0 + (x1 - x0) * u
            air = 4 * u * (1 - u)
            y = FLOOR_Y - h * air
            sy *= 1 + 0.05 * air; sx *= 1 - 0.03 * air
        # squash on landing, anticipation before take-off
        sq = bump(t, t1 + 0.05, 0.09)
        sy *= 1 - 0.08 * sq; sx *= 1 + 0.05 * sq
        if t0 > 0.1:
            pre = bump(t, t0 - 0.05, 0.08)
            sy *= 1 - 0.06 * pre; sx *= 1 + 0.04 * pre
    # breathing and sway: never frozen
    sy *= 1 + 0.012 * math.sin(2 * math.pi * t / 1.7)
    sx *= 1 - 0.006 * math.sin(2 * math.pi * t / 1.7)
    lean = p['lean'] + 1.3 * math.sin(2 * math.pi * t / 2.6 + 0.4)
    head = p['head'] + 1.8 * math.sin(2 * math.pi * t / 2.1 + 1.1)
    uL = p['uL'] + 2.0 * math.sin(2 * math.pi * t / 1.9 + 0.3)
    uR = p['uR'] - 2.0 * math.sin(2 * math.pi * t / 2.3 + 2.0)
    fL = p['fL'] + 2.5 * math.sin(2 * math.pi * t / 1.6 + 0.8)
    fR = p['fR'] - 2.5 * math.sin(2 * math.pi * t / 1.75 + 2.4)
    for bt, side, amt in BEATS:
        b = bump(t, bt, 0.16) * amt
        head += 2.5 * b
        if side == 'L': uL += 5 * b; fL += 7 * b
        else: uR -= 5 * b; fR -= 7 * b
    # keep elbows bending the natural way only
    fL = max(fL, 0.0); fR = min(fR, 0.0)
    return dict(x=x, y=y, sx=sx, sy=sy, air=air, lean=lean, head=head, uL=uL, fL=fL, uR=uR, fR=fR)

def rot(vx, vy, deg):
    a = math.radians(deg); c, s = math.cos(a), math.sin(a)
    return vx * c - vy * s, vx * s + vy * c

def solve(st):
    """World pivot, rotation and scale of every part."""
    sc = (S * st['sx'], S * st['sy'])
    W = {}
    W['torso'] = ((st['x'], st['y']), st['lean'])
    rel = {'head': st['head'], 'upper_arm_left': st['uL'], 'upper_arm_right': st['uR'],
           'forearm_left': st['fL'], 'forearm_right': st['fR']}
    for name in ['head', 'upper_arm_left', 'upper_arm_right', 'forearm_left', 'forearm_right']:
        par = P[name]['parent']
        (px, py), pr = W[par]
        dx = (P[name]['pivot'][0] - P[par]['pivot'][0]) * sc[0]
        dy = (P[name]['pivot'][1] - P[par]['pivot'][1]) * sc[1]
        rx, ry = rot(dx, dy, pr)
        W[name] = ((px + rx, py + ry), pr + rel[name])
    return W, sc

STATES = [owl_state(tf(i) / 1000) for i in FRAMES]
SOLVED = [solve(s) for s in STATES]

def kf(values, frames, ease='linear'):
    out = []
    last = None
    for n, i in enumerate(frames):
        v = values[i]
        rec = {'t': tf(i), 'v': v}
        if n: rec['ease'] = ease
        out.append(rec)
    # drop interior records that repeat both neighbours (constant spans)
    pr = []
    for n, r in enumerate(out):
        if 0 < n < len(out) - 1 and out[n - 1]['v'] == r['v'] == out[n + 1]['v']:
            continue
        pr.append(r)
    pr[0].pop('ease', None)
    for r in pr[1:]: r['ease'] = ease
    return pr

def r2(v): return round(v, 2)
def r4(v): return round(v, 4)

def part_anim(name, frames):
    xs = {i: round(SOLVED[i][0][name][0][0]) for i in frames}
    ys = {i: round(SOLVED[i][0][name][0][1]) for i in frames}
    rs = {i: r2(SOLVED[i][0][name][1]) for i in frames}
    ss = {i: [r4(SOLVED[i][1][0]), r4(SOLVED[i][1][1])] for i in frames}
    return dict(x=kf(xs, frames), y=kf(ys, frames), scale=kf(ss, frames), rotation=kf(rs, frames))

def part_el(eid, name, start, end, frames, src=None):
    p = P[name]
    a = part_anim(name if name in SOLVED[0][0] else 'head', frames)  # face overlays ride the head
    return {'id': eid, 'type': 'image', 'start': start, 'end': end,
            'source': 'character/' + (src or p['file']), 'x': a['x'], 'y': a['y'], 'origin': 'center',
            'width': p['width'], 'height': p['height'], 'fit': 'literal', 'scale': a['scale'],
            'rotation': a['rotation']}

tracks = []
def track(name, layer, els): tracks.append({'name': name, 'layer': layer, 'elements': els})

# ---------------------------------------------------------------- set
track('set', 0, [{'id': 'study', 'type': 'image', 'start': 0, 'end': DUR, 'source': 'character/study.png',
                  'x': 0, 'y': 0, 'origin': 'top-left', 'width': 1920, 'height': 1080, 'fit': 'literal'}])

# owl's contact shadow follows the feet and shrinks in the air
shadow_w = {i: [r4(1 - 0.45 * STATES[i]['air']), r4(1 - 0.45 * STATES[i]['air'])] for i in FRAMES}
shadow_x = {i: round(STATES[i]['x']) for i in FRAMES}
track('owl-shadow', 1, [{'id': 'owl-shadow', 'type': 'ellipse', 'start': 0, 'end': DUR,
    'x': kf(shadow_x, FRAMES), 'y': FLOOR_Y + 4, 'origin': 'center', 'width': 330, 'height': 44,
    'fill': '#7A4E2A', 'scale': kf(shadow_w, FRAMES), 'opacity': 0.28}])

# ---------------------------------------------------------------- owl
LAYER0 = 20
for n, name in enumerate(['torso', 'forearm_left', 'forearm_right', 'upper_arm_left', 'upper_arm_right', 'head']):
    track('owl-' + name.replace('_', '-'), LAYER0 + n, [part_el('owl-' + name.replace('_', '-'), name, 0, DUR, FRAMES)])

# lip-sync: one mouth element per run of frames sharing a mouth shape
VIS = voice['visemes']
LEAD = 20  # ms of anticipation: the mouth shape lands a hair before the sound
def mouth_at(ms_rel):
    cur = 0
    for t, vid in VIS:
        if t <= ms_rel: cur = vid
        else: break
    if ms_rel < VIS[0][0]: cur = 0
    return rig['visemes'][str(cur)]
mouth_frames = []
for i in FRAMES[:-1]:
    rel = tf(i) - VO + LEAD
    mouth_frames.append(mouth_at(rel) if 0 <= rel <= voice['duration_ms'] else None)
mouth_els = []
i = 0
while i < NF:
    m = mouth_frames[i]
    j = i
    while j < NF and mouth_frames[j] == m: j += 1
    if m:
        fr = list(range(i, j + 1))
        mouth_els.append(part_el(f'mouth-{len(mouth_els)+1:03d}-{m}', 'mouth_' + m, tf(i), tf(j), fr))
    i = j
track('owl-mouth', LAYER0 + 6, mouth_els)

BLINKS = [0.95, 2.80, 4.98, 7.40, 9.05]
blink_els = []
for n, b in enumerate(BLINKS):
    i0 = round(b * FPS); i1 = i0 + 3
    blink_els.append(part_el(f'blink-{n+1}', 'eyes_closed', tf(i0), tf(i1), list(range(i0, i1 + 1))))
track('owl-blink', LAYER0 + 7, blink_els)

# ---------------------------------------------------------------- the question mark
QX, QY = 905, 290
track('question', 40, [{'id': 'question-mark', 'type': 'text', 'start': 1500, 'end': 3000,
    'x': QX, 'y': QY, 'origin': 'center', 'width': 140, 'height': 204, 'font': 'bold', 'size': 170,
    'color': '#FF5A36', 'runs': [{'text': '?'}],
    'stroke': '#101418', 'stroke_width': 6,
    'scale': [{'t': 1500, 'v': [0.0, 0.0]}, {'t': 1700, 'v': [1.18, 1.18], 'ease': 'ease-out'},
              {'t': 1820, 'v': [1.0, 1.0], 'ease': 'ease-in-out'}, {'t': 2200, 'v': [1.0, 1.0], 'ease': 'linear'},
              {'t': 2330, 'v': [1.12, 1.12], 'ease': 'ease-out'}, {'t': 2480, 'v': [1.0, 1.0], 'ease': 'ease-in-out'}],
    'rotation': [{'t': 1500, 'v': -25.0}, {'t': 1750, 'v': 10.0, 'ease': 'ease-out'}, {'t': 2000, 'v': -4.0, 'ease': 'ease-in-out'},
                 {'t': 2300, 'v': 8.0, 'ease': 'ease-in-out'}, {'t': 2650, 'v': 2.0, 'ease': 'ease-in-out'}],
    'opacity': [{'t': 1500, 'v': 0.0}, {'t': 1580, 'v': 1.0, 'ease': 'linear'}, {'t': 2800, 'v': 1.0, 'ease': 'linear'},
                {'t': 2980, 'v': 0.0, 'ease': 'ease-in'}],
    'caption': False}])

# ---------------------------------------------------------------- laptop and the file
LAP_S = 0.85
LAP = (1250, 985)                      # bottom-centre, on the floor
SCR_W, SCR_H = 397, 230                # screen 467x271 at 0.85
SCR_C = (1250, 985 - 331)              # screen centre
SCR_X0, SCR_Y0 = SCR_C[0] - SCR_W // 2, SCR_C[1] - SCR_H // 2
LAP_IN, LAP_OUT = 2700, 6650
fade_out = lambda t0, t1: [{'t': t0, 'v': 1.0}, {'t': t1, 'v': 0.0, 'ease': 'ease-in'}]
track('laptop-shadow', 2, [{'id': 'laptop-shadow', 'type': 'ellipse', 'start': LAP_IN, 'end': LAP_OUT,
    'x': LAP[0], 'y': LAP[1] - 2, 'origin': 'center', 'width': 640, 'height': 40, 'fill': '#7A4E2A',
    'scale': [{'t': 2700, 'v': [0.0, 0.0]}, {'t': 2900, 'v': [1.05, 1.05], 'ease': 'ease-out'}, {'t': 3020, 'v': [1.0, 1.0], 'ease': 'ease-in-out'}],
    'opacity': [{'t': 6400, 'v': 0.28}, {'t': 6650, 'v': 0.0, 'ease': 'ease-in'}]}])
track('laptop', 5, [{'id': 'laptop', 'type': 'image', 'start': LAP_IN, 'end': LAP_OUT, 'source': 'character/laptop.png',
    'x': LAP[0], 'y': LAP[1], 'origin': 'bottom-center', 'width': 708, 'height': 566, 'fit': 'literal',
    'scale': [{'t': 2700, 'v': [0.0, 0.0]}, {'t': 2900, 'v': [0.93, 0.93], 'ease': 'ease-out'},
              {'t': 3020, 'v': [LAP_S, LAP_S], 'ease': 'ease-in-out'}],
    'opacity': fade_out(6400, 6650)}])

CODE_IN = 3020
track('screen', 6, [{'id': 'screen-bg', 'type': 'rect', 'start': CODE_IN, 'end': LAP_OUT,
    'x': SCR_X0, 'y': SCR_Y0, 'origin': 'top-left', 'width': SCR_W, 'height': SCR_H, 'fill': '#101418',
    'opacity': [{'t': 3020, 'v': 0.0}, {'t': 3100, 'v': 1.0, 'ease': 'linear'}, {'t': 6400, 'v': 1.0, 'ease': 'linear'}, {'t': 6650, 'v': 0.0, 'ease': 'ease-in'}]}])

TX = SCR_X0 + 20
def code_text(eid, start, end, y, size, font, color, runs, height):
    return {'id': eid, 'type': 'text', 'start': start, 'end': end, 'x': TX, 'y': y, 'origin': 'top-left', 'width': SCR_W - 40,
            'height': height, 'font': font, 'size': size, 'color': color, 'runs': runs,
            'opacity': ([{'t': 3020, 'v': 0.0}, {'t': 3100, 'v': 1.0, 'ease': 'linear'}] if start <= 3020 else []) +
                       [{'t': 6400, 'v': 1.0, **({'ease': 'linear'} if start <= 3020 else {})}, {'t': 6650, 'v': 0.0, 'ease': 'ease-in'}],
            'caption': False}
L1Y, L2Y, L3Y, L4Y = SCR_Y0 + 18, SCR_Y0 + 70, SCR_Y0 + 112, SCR_Y0 + 158
for el in [
    code_text('code-filename', CODE_IN, LAP_OUT, L1Y, 20, 'regular', '#9AA3AD', [{'text': 'hello.montagent.json'}], 24),
    code_text('code-line-1', CODE_IN, LAP_OUT, L2Y, 24, 'regular', '#9AA3AD', [{'text': '{"id":"hello","type":"text",'}], 29),
    code_text('code-line-2', CODE_IN, LAP_OUT, L3Y, 28, 'regular', '#F5F0E6', [{'text': '"runs":[{"text":'}], 34),
]:
    track(el['id'], 7, [el])
# a rule under the file name, like an editor tab
track('code-tab', 7, [{'id': 'code-tab-rule', 'type': 'rect', 'start': CODE_IN, 'end': LAP_OUT,
    'x': SCR_X0, 'y': SCR_Y0 + 54, 'origin': 'top-left', 'width': SCR_W, 'height': 2, 'fill': '#2A3138',
    'opacity': [{'t': 3020, 'v': 0.0}, {'t': 3100, 'v': 1.0, 'ease': 'linear'}, {'t': 6400, 'v': 1.0, 'ease': 'linear'}, {'t': 6650, 'v': 0.0, 'ease': 'ease-in'}]}])

# the edited line: "World" is selected on "edit", then "Montagent" types in over it
PREFIX_W = 114.875
WORLD_W = 93.609
SEL_ON, TYPE_ON = tf(100), tf(107)
word = 'Montagent'
edit_els = [code_text('edit-0-world', CODE_IN, SEL_ON, L4Y, 32, 'bold', '#F5F0E6',
                      [{'text': '"Hello, World"}]'}], 39)]
edit_els.append(code_text('edit-1-selected', SEL_ON, TYPE_ON, L4Y, 32, 'bold', '#F5F0E6',
                          [{'text': '"Hello, '}, {'text': 'World', 'color': '#101418'}, {'text': '"}]'}], 39))
STEP = 67
for k in range(1, len(word) + 1):
    s0 = tf(107 + 2 * (k - 1))
    s1 = tf(107 + 2 * k) if k < len(word) else LAP_OUT
    edit_els.append(code_text(f'edit-{k+1}-typed', s0, s1, L4Y, 32, 'bold', '#F5F0E6',
                              [{'text': '"Hello, '}, {'text': word[:k], 'color': '#FF5A36'}, {'text': '"}]'}], 39))
track('code-edit', 8, edit_els)
track('code-selection', 7, [{'id': 'code-selection', 'type': 'rect', 'start': SEL_ON, 'end': TYPE_ON,
    'x': TX + round(PREFIX_W) - 3, 'y': L4Y + 1, 'origin': 'top-left', 'width': round(WORLD_W) + 6, 'height': 38, 'fill': '#FF5A36', 'radius': 4}])

# ---------------------------------------------------------------- the rendered video, out of the laptop
CARD_C = (1480, 250)
CARD_S = 0.6
FLY0, FLY1 = 4560, 5000
SCR_FIT = SCR_W / 960
card_xy = lambda c: [{'t': FLY0, 'v': c[0]}, {'t': FLY1, 'v': c[1], 'ease': 'ease-out'}]
def card_scale(base):
    return [{'t': FLY0, 'v': [r4(SCR_FIT * base), r4(SCR_FIT * base)]},
            {'t': FLY1 - 60, 'v': [r4(CARD_S * base * 1.05), r4(CARD_S * base * 1.05)], 'ease': 'ease-out'},
            {'t': FLY1 + 100, 'v': [r4(CARD_S * base), r4(CARD_S * base)], 'ease': 'ease-in-out'}]
card_op = [{'t': FLY0, 'v': 0.0}, {'t': FLY0 + 60, 'v': 1.0, 'ease': 'linear'}, {'t': 6400, 'v': 1.0, 'ease': 'linear'},
           {'t': 6650, 'v': 0.0, 'ease': 'ease-in'}]
track('card-frame', 9, [{'id': 'card-frame', 'type': 'rect', 'start': FLY0, 'end': LAP_OUT,
    'x': card_xy((SCR_C[0], CARD_C[0])), 'y': card_xy((SCR_C[1], CARD_C[1])), 'origin': 'center',
    'width': 1000, 'height': 580, 'fill': '#101418', 'radius': 26, 'scale': card_scale(1.0), 'opacity': card_op,
    'effects': [{'name': 'shadow', 'dx': 0, 'dy': 14, 'radius': 24, 'color': '#5A3A1E', 'opacity': 0.35}]}])
track('card-video', 10, [{'id': 'card-video', 'type': 'image', 'start': FLY0, 'end': LAP_OUT,
    'source': 'stills/session-02.png',
    'x': card_xy((SCR_C[0], CARD_C[0])), 'y': card_xy((SCR_C[1], CARD_C[1])), 'origin': 'center',
    'width': 960, 'height': 540, 'fit': 'literal', 'scale': card_scale(1.0), 'opacity': card_op}])
# playback bar once it has landed: the video plays
BAR_W = round(940 * CARD_S)
BAR_X = CARD_C[0] - BAR_W // 2
BAR_Y = CARD_C[1] + round(250 * CARD_S)
track('card-bar-track', 11, [{'id': 'card-bar-track', 'type': 'rect', 'start': FLY1 + 100, 'end': LAP_OUT,
    'x': BAR_X, 'y': BAR_Y, 'origin': 'top-left', 'width': BAR_W, 'height': 6, 'fill': '#F5F0E64D', 'radius': 3,
    'opacity': fade_out(6400, 6650)}])
track('card-bar-fill', 12, [{'id': 'card-bar-fill', 'type': 'rect', 'start': FLY1 + 100, 'end': LAP_OUT,
    'x': BAR_X, 'y': BAR_Y, 'origin': 'top-left', 'width': BAR_W, 'height': 6, 'fill': '#FF5A36', 'radius': 3,
    'scale': [{'t': FLY1 + 100, 'v': [0.02, 1.0]}, {'t': 6650, 'v': [0.75, 1.0], 'ease': 'linear'}],
    'opacity': fade_out(6400, 6650)}])
# an arrow from the laptop up to the video it rendered
track('render-arrow', 13, [{'id': 'render-arrow', 'type': 'text', 'start': 4700, 'end': LAP_OUT,
    'x': 1400, 'y': 462, 'origin': 'center', 'width': 140, 'height': 156, 'font': 'bold', 'size': 130,
    'color': '#FF5A36', 'runs': [{'text': '→'}],
    'scale': [{'t': 4700, 'v': [0.0, 0.0]}, {'t': 4900, 'v': [1.15, 1.15], 'ease': 'ease-out'}, {'t': 5050, 'v': [1.0, 1.0], 'ease': 'ease-in-out'}],
    'rotation': -62.0, 'opacity': fade_out(6400, 6650), 'caption': False}])

# ---------------------------------------------------------------- brand
LOCK_W = 2694
LOCK_S = 0.27
LOCK_C = (1240, 400)
track('brand', 40, [{'id': 'lockup', 'type': 'image', 'start': 6700, 'end': DUR, 'source': 'brand/lockup.png',
    'x': LOCK_C[0], 'y': LOCK_C[1], 'origin': 'center', 'width': 2694, 'height': 623, 'fit': 'literal',
    'scale': [{'t': 6700, 'v': [r4(LOCK_S * 0.6), r4(LOCK_S * 0.6)]}, {'t': 6950, 'v': [r4(LOCK_S * 1.06), r4(LOCK_S * 1.06)], 'ease': 'ease-out'},
              {'t': 7100, 'v': [LOCK_S, LOCK_S], 'ease': 'ease-in-out'}],
    'opacity': [{'t': 6700, 'v': 0.0}, {'t': 6820, 'v': 1.0, 'ease': 'linear'}]}])
track('tagline', 41, [{'id': 'tagline', 'type': 'text', 'start': 7350, 'end': DUR,
    'x': LOCK_C[0], 'y': [{'t': 7350, 'v': 540}, {'t': 7700, 'v': 520, 'ease': 'ease-out'}], 'origin': 'top-center',
    'width': 700, 'height': 60, 'font': 'regular', 'size': 50, 'color': '#101418',
    'runs': [{'text': 'Edit the file. Get the '}, {'text': 'video', 'font': 'bold', 'color': '#FF5A36'}, {'text': '.'}],
    'opacity': [{'t': 7350, 'v': 0.0}, {'t': 7700, 'v': 1.0, 'ease': 'ease-out'}], 'caption': False}])

# ---------------------------------------------------------------- sound
track('voice', 0, [{'id': 'hoot-line-2', 'type': 'audio', 'start': VO, 'end': VO + voice['duration_ms'],
    'source': 'character/voice/line-2.wav', 'source_start': 0, 'source_end': voice['duration_ms'], 'volume': 1.25}])
track('music', 0, [{'id': 'music-bed', 'type': 'audio', 'start': 0, 'end': 9900, 'source': 'music/bed-120bpm.wav',
    'source_start': 0, 'source_end': 9900,
    'volume': [{'t': 0, 'v': 0.0}, {'t': 400, 'v': 0.22, 'ease': 'ease-out'}, {'t': 900, 'v': 0.1, 'ease': 'ease-in-out'},
               {'t': 6400, 'v': 0.1, 'ease': 'linear'}, {'t': 6900, 'v': 0.32, 'ease': 'ease-in-out'},
               {'t': 8900, 'v': 0.32, 'ease': 'linear'}, {'t': 9900, 'v': 0.0, 'ease': 'ease-in'}]}])

# ---------------------------------------------------------------- write
proj = json.load(open('hoot.montagent.json'))
proj['tracks'] = tracks
json.dump(proj, open('hoot.montagent.json', 'w'), ensure_ascii=False)
print('elements:', sum(len(t['elements']) for t in tracks), 'mouth runs:', len(mouth_els))
