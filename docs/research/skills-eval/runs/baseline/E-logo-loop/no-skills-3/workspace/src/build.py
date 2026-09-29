"""Generates logo-loop.json's tracks. Layout and timing live here; run, then `montagent fmt`."""
import json, subprocess, math, os

HERE = os.path.dirname(os.path.abspath(__file__))
PROJ = os.path.join(HERE, '..', 'logo-loop.json')

INK, PAPER, SIGNAL = '#101418', '#F5F0E6', '#FF5A36'
WORD = 'Montagent'
SIZE = 125

def f(n):  # frame n -> ms; floored, because frame n samples at exactly n/30 s and ranges are half-open
    return n * 1000 // 30

# --- measure every prefix in the project's own font -----------------------------------
specs = [{'font': 'bold', 'size': SIZE, 'y': 0, 'origin': 'top-left', 'runs': [{'text': WORD[:i]}]}
         for i in range(1, len(WORD) + 1)]
res = json.loads(subprocess.run(['montagent', 'measure', PROJ, '--elements', json.dumps(specs), '--json'],
                                capture_output=True, text=True, check=True).stdout)['measure']['results']
adv = [0.0] + [r['ok']['advance_width'] for r in res]
baseline_y = res[0]['ok']['lines'][0]['baseline_y']
block_h = res[0]['ok']['block_height']

# --- layout (mark scaled from the 1024 construction: tile 460 -> 86, gap 40 -> 7, r 96 -> 18)
TILE, GAP, RAD = 86, 7, 18
MARK = 2 * TILE + GAP                       # 179
MARK_TEXT_GAP = 68                          # lockup proportion, ~0.38 x mark
CUR_W, CUR_GAP = 11, 8
total = MARK + MARK_TEXT_GAP + math.ceil(adv[-1]) + CUR_GAP + CUR_W
X0 = (1080 - total) // 2
Y0 = 540 - MARK // 2
cx = [X0 + TILE // 2, X0 + TILE + GAP + TILE // 2]
cy = [Y0 + TILE // 2, Y0 + TILE + GAP + TILE // 2]
TEXT_X = X0 + MARK + MARK_TEXT_GAP
BASELINE = 572                              # caps sit slightly above mark centre, as in the lockup
TEXT_Y = round(BASELINE - baseline_y)
CAP_TOP = BASELINE - 91
CUR_TOP, CUR_H = CAP_TOP - 12, 91 + 12 + 20

def cur_x(i):
    return TEXT_X + round(adv[i]) + CUR_GAP

FLY = [0.16, 1, 0.3, 1]                     # fast in, long settle
SPIN = [0.4, 0.5, 0.6, 1]                   # gentle start: peak ~36 deg/frame, well under a square's 45 deg alias
OUT = [0.55, 0, 0.9, 0.45]                  # ease-in shrink

def kf(pairs):
    out = []
    for n, (t, v, e) in enumerate(pairs):
        r = {'t': t, 'v': v}
        if n:
            r['ease'] = e
        out.append(r)
    return out

END_ALL = f(175)
tracks = []

def tile(tid, kind, layer, pos, arrive, fly_from, spin=False):
    (x, y) = pos
    (a0, a1) = arrive
    (s0, s1) = shrink[tid]
    e = {'id': tid, 'type': kind, 'group': 'mark', 'start': 0, 'end': END_ALL}
    fx, fy = fly_from
    ease = SPIN if spin else FLY
    e['x'] = kf([(f(a0), fx, None), (f(a1), x, ease)]) if fx != x else x
    e['y'] = kf([(f(a0), fy, None), (f(a1), y, ease)]) if fy != y else y
    e['origin'] = 'center'
    e['width'] = TILE
    e['height'] = TILE
    e['fill'] = SIGNAL if kind == 'ellipse' else INK
    if kind == 'rect':
        e['radius'] = RAD
    e['scale'] = kf([(f(s0), [1.0, 1.0], None), (f(s1), [0.0, 0.0], OUT)])
    if spin:
        e['rotation'] = kf([(f(a0), -1080.0, None), (f(a1), 0.0, SPIN)])
    tracks.append({'name': tid, 'layer': layer, 'elements': [e]})

shrink = {'tile-tl': (156, 165), 'tile-tr': (159, 168), 'tile-bl': (162, 171), 'tile-br': (165, 174)}
tile('tile-tl', 'rect', 10, (cx[0], cy[0]), (2, 22), (-120, cy[0]))
tile('tile-tr', 'ellipse', 11, (cx[1], cy[0]), (7, 27), (cx[1], -120))
tile('tile-bl', 'rect', 12, (cx[0], cy[1]), (12, 32), (cx[0], 1200))
tile('tile-br', 'rect', 13, (cx[1], cy[1]), (16, 60), (1240, cy[1]), spin=True)

# --- text: one element per visible prefix; type on at 5 frames/letter, backspace at 2 ---
TYPE0, TYPE_STEP = 90, 5          # letters land at 3000, 3167 ... 4333
DEL0, DEL_STEP = 150, 2           # hold until 5000, then backspace to empty by 5533
n = len(WORD)
typed = [TYPE0 + TYPE_STEP * i for i in range(n)]            # frame prefix i+1 appears
deleted = [DEL0 + DEL_STEP * k for k in range(n)]             # frame prefix n-k is replaced by n-k-1
# spans: (frame_from, frame_to, prefix length)
spans = [(typed[i], typed[i + 1], i + 1) for i in range(n - 1)]
spans.append((typed[-1], deleted[0], n))
for k in range(n - 1):
    spans.append((deleted[k], deleted[k + 1], n - k - 1))
text_els = []
for a, b, L in spans:
    text_els.append({'id': f'word-{a:03d}', 'type': 'text', 'group': 'word', 'start': f(a), 'end': f(b),
                     'x': TEXT_X, 'y': TEXT_Y, 'origin': 'top-left',
                     'width': math.ceil(adv[L]) + 2, 'height': block_h,
                     'font': 'bold', 'size': SIZE, 'color': INK, 'runs': [{'text': WORD[:L]}]})
tracks.append({'name': 'word', 'layer': 20, 'elements': text_els})

# --- cursor: hard on/off elements; solid while it moves -------------------------------------
cur_spans = [(69, 76, 0), (83, 90, 0)]                         # appears ~2.3 s, blinks, waits
cur_spans += [(a, b, L) for a, b, L in spans if L > 0 and a < deleted[0]]   # rides ahead of each letter
cur_spans += [(typed[-1], 137, n)]                             # (replaces the last typing span)
cur_spans = [s for s in cur_spans if not (s[0] == typed[-1] and s[1] == deleted[0])]
cur_spans += [(144, 150, n)]                                   # one blink in the hold
cur_spans += [(a, b, L) for a, b, L in spans if a >= deleted[0]]   # rides back while deleting
cur_spans += [(deleted[-1], 170, 0)]                           # empty line, then gone
cur_els = [{'id': f'cursor-{a:03d}', 'type': 'rect', 'group': 'cursor', 'start': f(a), 'end': f(b),
            'x': cur_x(L), 'y': CUR_TOP, 'origin': 'top-left', 'width': CUR_W, 'height': CUR_H, 'fill': INK}
           for a, b, L in sorted(cur_spans)]
tracks.append({'name': 'cursor', 'layer': 21, 'elements': cur_els})

doc = json.load(open(PROJ))
doc['loop'] = True
doc['tracks'] = tracks
json.dump(doc, open(PROJ, 'w'), indent=2)
print('X0', X0, 'TEXT_X', TEXT_X, 'total', total, 'cursor spans', sorted(cur_spans))
