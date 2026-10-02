"""Render Brief H3: Hoot fixes a late caption. Writes raw RGB frames to stdout or PNG stills."""
import json, math, sys, os
from PIL import Image, ImageDraw, ImageFilter, ImageFont

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
P = lambda *a: os.path.join(ROOT, *a)

W, H, FPS, DUR = 1920, 1080, 30, 8.0
NFRAMES = int(DUR * FPS)
VOICE_AT = 1.0

INK = (16, 20, 24)
PAPER = (245, 240, 230)
SIGNAL = (255, 90, 54)
OUTLINE = (48, 36, 38)

# ---------------------------------------------------------------- layout
S = 0.66                      # owl drawing px -> screen px
FEET_DRAW = (512, 1332)       # bottom of the feet in the drawing
FEET = (735, 978)             # where the feet stand on the floor
LS = 0.68                     # laptop scale
LX, LY = 905, 392             # laptop top-left on screen
CRATE_TOP = LY + round(566 * LS) - 10   # top surface line the laptop sits on
CRATE_BOTTOM = 990

rig = json.load(open(P('character', 'rig.json')))
line = json.load(open(P('character', 'voice', 'line-3.json')))

# ---------------------------------------------------------------- affine helpers (3x3 as 6-tuples a,b,c,d,e,f: x'=ax+by+c, y'=dx+ey+f)
def mul(m, n):
    a, b, c, d, e, f = m
    A, B, C, D, E, F = n
    return (a*A + b*D, a*B + b*E, a*C + b*F + c, d*A + e*D, d*B + e*E, d*C + e*F + f)

def T(x, y): return (1, 0, x, 0, 1, y)
def Sc(sx, sy=None): return (sx, 0, 0, 0, sx if sy is None else sy, 0)
def R(deg):
    r = math.radians(deg); c, s = math.cos(r), math.sin(r)
    return (c, -s, 0, s, c, 0)     # y-down: positive = clockwise on screen
def inv(m):
    a, b, c, d, e, f = m
    det = a*e - b*d
    ia, ib, id_, ie = e/det, -b/det, -d/det, a/det
    return (ia, ib, -(ia*c + ib*f), id_, ie, -(id_*c + ie*f))
def ap(m, p):
    a, b, c, d, e, f = m
    return (a*p[0] + b*p[1] + c, d*p[0] + e*p[1] + f)
def about(px, py, m): return mul(T(px, py), mul(m, T(-px, -py)))

# ---------------------------------------------------------------- easing / keyframes
def smooth(x): x = min(1, max(0, x)); return x*x*(3 - 2*x)
def ease_out_back(x, k=2.2):
    x = min(1, max(0, x)); x -= 1
    return 1 + x*x*((k + 1)*x + k)

def keys(t, kf):
    """kf: list of (time, value); smoothstep between neighbours."""
    if t <= kf[0][0]: return kf[0][1]
    for (t0, v0), (t1, v1) in zip(kf, kf[1:]):
        if t <= t1:
            u = smooth((t - t0) / (t1 - t0)) if t1 > t0 else 1
            return v0 + (v1 - v0)*u
    return kf[-1][1]

# ---------------------------------------------------------------- parts
parts = {}
for name, spec in rig['parts'].items():
    im = Image.open(P('character', spec['file'])).convert('RGBA')
    w, h = im.size
    sw, sh = round(w*S), round(h*S)
    small = im.convert('RGBa').resize((sw, sh), Image.LANCZOS)
    # maps scaled-image pixel -> drawing coords (canvas centre on pivot)
    px, py = spec['pivot']
    to_draw = mul(T(px - w/2, py - h/2), Sc(w/sw, h/sh))
    bb = im.getbbox()
    corners = [(bb[0]*sw/w, bb[1]*sh/h), (bb[2]*sw/w, bb[1]*sh/h), (bb[0]*sw/w, bb[3]*sh/h), (bb[2]*sw/w, bb[3]*sh/h)]
    parts[name] = dict(img=small, to_draw=to_draw, corners=corners, pivot=(px, py), parent=spec['parent'])

CAM = mul(T(*FEET), mul(Sc(S), T(-FEET_DRAW[0], -FEET_DRAW[1])))

def draw_part(frame, name, M):
    p = parts[name]
    full = mul(M, p['to_draw'])
    pts = [ap(full, c) for c in p['corners']]
    x0 = max(0, int(math.floor(min(q[0] for q in pts))) - 2)
    y0 = max(0, int(math.floor(min(q[1] for q in pts))) - 2)
    x1 = min(W, int(math.ceil(max(q[0] for q in pts))) + 2)
    y1 = min(H, int(math.ceil(max(q[1] for q in pts))) + 2)
    if x1 <= x0 or y1 <= y0: return
    m = inv(mul(T(-x0, -y0), full))
    out = p['img'].transform((x1 - x0, y1 - y0), Image.AFFINE, m, resample=Image.BICUBIC).convert('RGBA')
    frame.alpha_composite(out, (x0, y0))

# ---------------------------------------------------------------- arms: angles and IK (in torso/drawing frame)
def ang(v): return math.degrees(math.atan2(v[1], v[0]))
def sub(a, b): return (a[0] - b[0], a[1] - b[1])

ARM = {}
for side, tip in (('left', (40, 894)), ('right', (983, 893))):
    Sh = parts['upper_arm_' + side]['pivot']
    El = parts['forearm_' + side]['pivot']
    Hd = (El[0] + 0.68*(tip[0] - El[0]), El[1] + 0.68*(tip[1] - El[1]))   # palm
    ARM[side] = dict(S=Sh, E=El, H=Hd, L1=math.dist(Sh, El), L2=math.dist(El, Hd),
                     phi1=ang(sub(El, Sh)), phi2=ang(sub(Hd, El)))

def ik(side, target, elbow_sign):
    a = ARM[side]
    d = math.dist(a['S'], target)
    d = min(max(d, abs(a['L1'] - a['L2']) + 1), a['L1'] + a['L2'] - 1)
    alpha = ang(sub(target, a['S']))
    beta = math.degrees(math.acos((a['L1']**2 + d*d - a['L2']**2) / (2*a['L1']*d)))
    th1 = alpha + elbow_sign*beta
    E = (a['S'][0] + a['L1']*math.cos(math.radians(th1)), a['S'][1] + a['L1']*math.sin(math.radians(th1)))
    th2 = ang(sub(target, E))
    return th1, th2

def unwrap(ref, a):
    while a - ref > 180: a -= 360
    while a - ref < -180: a += 360
    return a

def blend(p, q, u):
    return tuple(x + (unwrap(x, y) - x)*u for x, y in zip(p, q))

# world angles (torso frame) of upper arm and forearm
SIDES = {'left': (112, 98), 'right': (68, 82)}
RAISED = {'left': (-140, -118), 'right': (-40, -62)}

# ---------------------------------------------------------------- timing from the voice
words = [dict(w, s=w['start']/1000 + VOICE_AT, e=w['end']/1000 + VOICE_AT) for w in line['words']]
WT = {w['word'].strip('.!…,').lower(): w for w in words}
VIS = [(ms/1000 + VOICE_AT, vid) for ms, vid in line['visemes']]
VOICE_END = line['duration_ms']/1000 + VOICE_AT

T_HMM = WT['hmm']['s']; T_LATE_END = WT['late']['e']
T_TWO = WT['two']['s']
T_LET = WT['let']['s']; T_CHECK_END = WT['check']['e']
T_FIXED = WT['fixed']['s']; T_FIXED_END = WT['fixed']['e']
T_RIGHT = WT['right']['s']; T_BEAT = WT['beat']['s']; T_BEAT_END = WT['beat']['e']

def mouth_at(t):
    t += 0.02   # draw the shape a hair early, as animators do
    cur = None
    for vt, vid in VIS:
        if vt <= t: cur = vid
        else: break
    if cur is None or t >= VOICE_END: return None
    m = rig['visemes'][str(cur)]
    return None if m is None else 'mouth_' + m

BLINKS = [0.55, 2.62, 4.30, 6.95, 7.72]
def blinking(t):
    return any(b <= t < b + 3/FPS for b in BLINKS)

# hop
HOP_UP, HOP_DOWN, HOP_H = T_FIXED + 0.03, T_FIXED + 0.50, 125
TAPS = [T_LET + 0.27, T_LET + 0.57]

def hop_y(t):
    if HOP_UP <= t <= HOP_DOWN:
        u = (t - HOP_UP) / (HOP_DOWN - HOP_UP)
        return -HOP_H*4*u*(1 - u)
    return 0.0

def squash(t):
    """(sx, sy) of the whole body about the feet."""
    sy = 1.0
    # anticipation crouch, launch stretch, landing squash, settle
    sy = keys(t, [(HOP_UP - 0.20, 1.0), (HOP_UP - 0.04, 0.90), (HOP_UP + 0.06, 1.07), (HOP_UP + 0.22, 1.0),
                  (HOP_DOWN - 0.08, 1.03), (HOP_DOWN, 1.0), (HOP_DOWN + 0.06, 0.91), (HOP_DOWN + 0.20, 1.02), (HOP_DOWN + 0.32, 1.0)])
    sy *= 1 + 0.006*math.sin(2*math.pi*t/2.4)            # breathing
    sx = 1 + (1 - sy)*0.5
    return sx, sy

def lean(t):
    return keys(t, [(0, 0), (T_LET - 0.30, 0), (T_LET + 0.05, 6.0), (T_CHECK_END + 0.05, 6.0), (HOP_UP - 0.18, -1.0), (HOP_UP + 0.1, 0)]) \
        + 0.6*math.sin(2*math.pi*t/3.1)

def head_tilt(t):
    base = keys(t, [(0, 0), (T_HMM - 0.05, 0), (T_HMM + 0.30, 13), (T_LATE_END - 0.10, 11), (T_LET - 0.05, 4),
                    (T_CHECK_END + 0.05, 4), (HOP_UP - 0.1, 0), (HOP_UP + 0.08, -5), (HOP_DOWN - 0.05, 3), (HOP_DOWN + 0.15, -2),
                    (T_RIGHT + 0.10, 0), (T_BEAT - 0.05, -4), (T_BEAT + 0.20, 3), (T_BEAT_END + 0.25, 0)])
    # small accents on stressed words
    acc = 0
    for wkey, amt in (('two', -2.0), ('late', 2.0), ('check', -1.5), ('right', 2.0)):
        w = WT[wkey]
        acc += amt*math.sin(math.pi*min(1, max(0, (t - w['s'])/0.35)))
    return base + acc

def head_dy(t):
    # small bob (drawing px) on "two" and "late"
    d = 0
    for wkey, amt in (('two', 6), ('late', 5), ('right', 5)):
        w = WT[wkey]
        d += amt*math.sin(math.pi*min(1, max(0, (t - w['s'])/0.25)))
    return d

def left_arm(t, beak_target):
    think = ik('left', beak_target, -1)
    # a thoughtful little forearm drift while holding
    think = (think[0], think[1] + 3*math.sin(2*math.pi*(t - T_HMM)/1.3))
    pose = SIDES['left']
    pose = blend(pose, think, keys(t, [(T_HMM - 0.04, 0), (T_HMM + 0.30, 1), (T_LATE_END - 0.05, 1), (T_LET - 0.02, 0)]))
    pose = blend(pose, RAISED['left'], keys(t, [(HOP_UP - 0.14, 0), (HOP_UP + 0.04, 1), (T_RIGHT - 0.02, 1), (T_RIGHT + 0.42, 0)]))
    return pose

def right_arm(t, key_target):
    reach = ik('right', key_target, -1)
    tap = sum(keys(t, [(tt - 0.01, 0), (tt + 0.06, 1), (tt + 0.17, 0)]) for tt in TAPS)
    reach = (reach[0], reach[1] + 16*tap)
    pose = SIDES['right']
    pose = blend(pose, reach, keys(t, [(T_LET - 0.30, 0), (T_LET + 0.10, 1), (T_CHECK_END + 0.05, 1), (HOP_UP - 0.14, 0)]))
    pose = blend(pose, RAISED['right'], keys(t, [(HOP_UP - 0.14, 0), (HOP_UP + 0.04, 1), (T_RIGHT - 0.02, 1), (T_RIGHT + 0.42, 0)]))
    return pose

def idle_arm(pose, t, side):
    k = 1 if side == 'left' else -1
    return (pose[0] + k*1.2*math.sin(2*math.pi*t/2.4 + 0.5), pose[1] + k*1.8*math.sin(2*math.pi*t/2.4 + 1.1))

# keyboard target for the right palm, in screen px (laptop image px -> screen)
def laptop_to_screen(x, y): return (LX + x*LS, LY + y*LS)
KEY_TARGET = laptop_to_screen(192, 340)

def owl_matrices(t):
    sx, sy = squash(t)
    tor = mul(T(0, hop_y(t)/S), about(*FEET_DRAW, mul(R(lean(t)), Sc(sx, sy))))
    G = {'torso': mul(CAM, tor)}
    G['head'] = mul(G['torso'], mul(T(0, head_dy(t)), about(*parts['head']['pivot'], R(head_tilt(t)))))
    # beak target in torso frame (palm just below-left of the beak, fingers up to it)
    beak_screen = ap(G['head'], (462, 684))
    beak_t = ap(inv(G['torso']), beak_screen)
    key_t = ap(inv(G['torso']), KEY_TARGET)
    for side, pose in (('left', left_arm(t, beak_t)), ('right', right_arm(t, key_t))):
        th1, th2 = idle_arm(pose, t, side)
        a = ARM[side]
        a1 = th1 - a['phi1']
        a2 = th2 - a['phi2'] - a1
        up = mul(G['torso'], about(*a['S'], R(a1)))
        G['upper_arm_' + side] = up
        G['forearm_' + side] = mul(up, about(*a['E'], R(a2)))
    return G

def think_on_top(t):
    return T_HMM - 0.1 <= t <= T_LET + 0.15

def draw_owl(frame, t):
    G = owl_matrices(t)
    order = ['torso', 'forearm_left', 'forearm_right', 'upper_arm_left', 'upper_arm_right', 'head']
    late = []
    if think_on_top(t):
        order.remove('forearm_left'); order.remove('upper_arm_left')
        late = ['forearm_left', 'upper_arm_left']
    for n in order:
        draw_part(frame, n, G[n])
    m = mouth_at(t)
    if m: draw_part(frame, m, G['head'])
    if blinking(t): draw_part(frame, 'eyes_closed', G['head'])
    for n in late:
        draw_part(frame, n, G[n])
    return G

# ---------------------------------------------------------------- set dressing
def supersampled(size, fn, k=3):
    big = Image.new('RGBA', (size[0]*k, size[1]*k), (0, 0, 0, 0))
    fn(ImageDraw.Draw(big), k)
    return big.resize(size, Image.LANCZOS)

def make_crate():
    x0, x1 = LX - 18, LX + round(708*LS) + 18
    top_h = 34
    y_top = CRATE_TOP - top_h + 12
    w, h = x1 - x0 + 8, CRATE_BOTTOM - y_top + 8
    lw = 5
    def fn(d, k):
        o = 4*k; L = lw*k
        fw, fh = (x1 - x0)*k, (CRATE_BOTTOM - y_top)*k
        th = top_h*k
        inset = 26*k
        # top face (seen from slightly above)
        d.polygon([(o + inset, o), (o + fw - inset, o), (o + fw, o + th), (o, o + th)], fill=(222, 168, 112), outline=OUTLINE, width=L)
        d.line([(o + inset + 20*k, o + th*0.5), (o + fw - inset - 20*k, o + th*0.5)], fill=(196, 140, 88), width=2*k)
        # front face
        d.rounded_rectangle([o, o + th, o + fw, o + fh], radius=8*k, fill=(204, 140, 86), outline=OUTLINE, width=L)
        # planks
        ph = (fh - th)/3
        for i in (1, 2):
            y = o + th + ph*i
            d.line([(o + L, y), (o + fw - L, y)], fill=OUTLINE, width=round(L*0.7))
        # corner posts
        post = 34*k
        for xa in (o, o + fw - post):
            d.rounded_rectangle([xa, o + th, xa + post, o + fh], radius=6*k, fill=(178, 116, 68), outline=OUTLINE, width=L)
        # nails
        for xa in (o + post/2, o + fw - post/2):
            for i in range(3):
                cy = o + th + ph*i + ph*0.5
                r = 4*k
                d.ellipse([xa - r, cy - r, xa + r, cy + r], fill=OUTLINE)
        # a highlight strip on each plank
        for i in range(3):
            y = o + th + ph*i + 10*k
            d.line([(o + post + 14*k, y), (o + fw - post - 60*k, y)], fill=(222, 162, 106), width=3*k)
    img = supersampled((w, h), fn)
    return img, (x0 - 4, y_top - 4)

def shadow(cx, cy, rx, ry, alpha):
    pad = 40
    im = Image.new('RGBA', (int(2*rx + 2*pad), int(2*ry + 2*pad)), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    d.ellipse([pad, pad, pad + 2*rx, pad + 2*ry], fill=(90, 50, 25, int(255*alpha)))
    im = im.filter(ImageFilter.GaussianBlur(10))
    return im, (int(cx - rx - pad), int(cy - ry - pad))

SCREEN = (120, 41, 588, 313)   # laptop.png px, exclusive right/bottom (x 120-587, y 41-312)
SW, SH = SCREEN[2] - SCREEN[0], SCREEN[3] - SCREEN[1]

def cover(im, w, h):
    iw, ih = im.size
    s = max(w/iw, h/ih)
    nw, nh = math.ceil(iw*s), math.ceil(ih*s)
    im = im.resize((nw, nh), Image.LANCZOS)
    l, t = (nw - w)//2, (nh - h)//2
    return im.crop((l, t, l + w, t + h))

def mark_screen():
    k = 4
    im = Image.new('RGB', (SW*k, SH*k), PAPER)
    mark = Image.open(P('brand', 'mark.png')).convert('RGBA')
    word = Image.open(P('brand', 'wordmark.png')).convert('RGBA')
    mh = int(SH*k*0.56)
    mark = mark.resize((mh, mh), Image.LANCZOS)
    ww = int(SW*k*0.50); wh = round(word.size[1]*ww/word.size[0])
    word = word.resize((ww, wh), Image.LANCZOS)
    gap = int(SH*k*0.05)
    top = (SH*k - (mh + gap + wh))//2
    im.paste(mark, ((SW*k - mh)//2, top), mark)
    im.paste(word, ((SW*k - ww)//2, top + mh + gap), word)
    return im.resize((SW, SH), Image.LANCZOS)

laptop_raw = Image.open(P('character', 'laptop.png')).convert('RGBA')
def laptop_with(screen_img):
    l = laptop_raw.copy()
    l.paste(screen_img.convert('RGB'), SCREEN[:2])
    return l.resize((round(708*LS), round(566*LS)), Image.LANCZOS)

LAPTOP_A = laptop_with(cover(Image.open(P('stills', 'session-02.png')).convert('RGB'), SW, SH))
LAPTOP_B = laptop_with(mark_screen())
T_SCREEN = T_BEAT_END + 0.02
def laptop_at(t):
    u = min(1, max(0, (t - T_SCREEN)/(3/FPS)))
    if u <= 0: return LAPTOP_A
    if u >= 1: return LAPTOP_B
    return Image.blend(LAPTOP_A, LAPTOP_B, smooth(u))

# ---------------------------------------------------------------- tag
font = ImageFont.truetype(P('fonts', 'Inter-Bold.ttf'), 46*3)
def make_tag(text):
    k = 3
    tw = font.getbbox(text)
    bw, bh = (tw[2] - tw[0]) + 2*30*k, 84*k
    ptr = 18*k
    im = Image.new('RGBA', (bw + 12*k, bh + ptr + 12*k), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    o = 6*k
    cx = o + bw/2
    d.rounded_rectangle([o, o, o + bw, o + bh], radius=bh/2, fill=SIGNAL, outline=OUTLINE, width=5*k)
    d.polygon([(cx - ptr, o + bh - 5*k), (cx + ptr, o + bh - 5*k), (cx, o + bh + ptr)], fill=SIGNAL)
    d.line([(cx - ptr, o + bh - 2*k), (cx, o + bh + ptr)], fill=OUTLINE, width=5*k)
    d.line([(cx + ptr, o + bh - 2*k), (cx, o + bh + ptr)], fill=OUTLINE, width=5*k)
    d.text((cx, o + bh/2 + 2*k), text, font=font, fill=PAPER, anchor='mm')
    im = im.resize((im.size[0]//k, im.size[1]//k), Image.LANCZOS)
    return im, (im.size[0]/2, o/k + (bh + ptr)/k)   # anchor = pointer tip

TAG_A = make_tag('+2 frames')
TAG_B = make_tag('0 frames')
TAG_TIP = (LX + 354*LS, LY - 14)

def draw_tag(frame, t):
    if t < T_TWO: return
    if t < T_FIXED:
        img, anc = TAG_A; s = ease_out_back((t - T_TWO)/0.30, 2.6)
    else:
        img, anc = TAG_B; s = 0.8 + 0.2*ease_out_back((t - T_FIXED)/0.28, 3.0)
    s *= 1 + 0.012*math.sin(2*math.pi*(t - T_TWO)/1.6)
    if s <= 0.02: return
    sz = (max(1, round(img.size[0]*s)), max(1, round(img.size[1]*s)))
    im = img.resize(sz, Image.LANCZOS)
    frame.alpha_composite(im, (round(TAG_TIP[0] - anc[0]*s), round(TAG_TIP[1] - anc[1]*s)))

# ---------------------------------------------------------------- frame
study = Image.open(P('character', 'study.png')).convert('RGBA')
CRATE, CRATE_POS = make_crate()
CSH, CSH_POS = shadow(CRATE_POS[0] + CRATE.size[0]/2, CRATE_BOTTOM - 2, CRATE.size[0]/2 + 14, 16, 0.30)

def render(t):
    f = study.copy()
    f.alpha_composite(CSH, CSH_POS)
    f.alpha_composite(CRATE, CRATE_POS)
    f.alpha_composite(laptop_at(t), (LX, LY))
    h = -hop_y(t)
    k = 1 - 0.45*h/HOP_H
    sh, pos = shadow(FEET[0], FEET[1] - 2, 175*k, 20*k, 0.32*k)
    f.alpha_composite(sh, pos)
    draw_owl(f, t)
    draw_tag(f, t)
    return f.convert('RGB')

if __name__ == '__main__':
    if len(sys.argv) > 1 and sys.argv[1] == 'stills':
        out = sys.argv[2]
        for a in sys.argv[3:]:
            fr = int(a)
            render(fr/FPS).save(os.path.join(out, f'f{fr:03d}.png'))
    else:
        o = sys.stdout.buffer
        for fr in range(NFRAMES):
            o.write(render(fr/FPS).tobytes())
