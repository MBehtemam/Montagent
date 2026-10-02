"""Render Brief H6: Hoot explains editing a word. Writes raw frames to stdout or PNGs."""
import json, math, sys, os
import numpy as np
from PIL import Image, ImageDraw, ImageFont, ImageFilter

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
A = lambda *p: os.path.join(ROOT, *p)
W, H, FPS, DUR = 1920, 1080, 30, 10.0
NF = int(DUR * FPS)
VOICE_AT = 1.0  # seconds into the piece the voice line starts

INK = (16, 20, 24)
PAPER = (245, 240, 230)
SIGNAL = (255, 90, 54)
OUTLINE = (59, 45, 40)

rig = json.load(open(A('character', 'rig.json')))
line = json.load(open(A('character', 'voice', 'line-2.json')))
parts = {n: Image.open(A('character', p['file'])).convert('RGBA') for n, p in rig['parts'].items()}
parts_pm = {n: im.convert('RGBa') for n, im in parts.items()}

F_BOLD = A('fonts', 'Inter-Bold.ttf')
F_REG = A('fonts', 'Inter-Regular.ttf')
F_MONO = '/System/Library/Fonts/Menlo.ttc'


# ---------------------------------------------------------------- helpers
def ease(p):
    p = min(max(p, 0.0), 1.0)
    return p * p * (3 - 2 * p)


def ease_out_back(p, s=1.7):
    p = min(max(p, 0.0), 1.0) - 1
    return 1 + p * p * ((s + 1) * p + s)


def track(t, keys):
    """keys: [(time, value)], eased between neighbours, held outside."""
    if t <= keys[0][0]:
        return keys[0][1]
    for (t0, v0), (t1, v1) in zip(keys, keys[1:]):
        if t <= t1:
            return v0 + (v1 - v0) * ease((t - t0) / (t1 - t0))
    return keys[-1][1]


def T(x, y):
    return np.array([[1, 0, x], [0, 1, y], [0, 0, 1]], float)


def R(deg):
    a = math.radians(deg)
    c, s = math.cos(a), math.sin(a)
    return np.array([[c, -s, 0], [s, c, 0], [0, 0, 1]], float)


def S(sx, sy):
    return np.array([[sx, 0, 0], [0, sy, 0], [0, 0, 1]], float)


def paste_affine(canvas, img_pm, M):
    """Composite premultiplied image img_pm onto canvas (RGBA) through affine M (image px -> canvas px)."""
    w, h = img_pm.size
    corners = M @ np.array([[0, w, 0, w], [0, 0, h, h], [1, 1, 1, 1]], float)
    x0, y0 = int(math.floor(corners[0].min())) - 1, int(math.floor(corners[1].min())) - 1
    x1, y1 = int(math.ceil(corners[0].max())) + 1, int(math.ceil(corners[1].max())) + 1
    x0c, y0c, x1c, y1c = max(x0, 0), max(y0, 0), min(x1, W), min(y1, H)
    if x1c <= x0c or y1c <= y0c:
        return
    inv = np.linalg.inv(T(x0c, y0c) @ np.eye(3)) if False else None
    Minv = np.linalg.inv(M) @ T(x0c, y0c)
    out = img_pm.transform((x1c - x0c, y1c - y0c), Image.AFFINE, data=tuple(Minv[:2].flatten()),
                           resample=Image.BICUBIC)
    canvas.alpha_composite(out.convert('RGBA'), (x0c, y0c))


def place(canvas, img, cx, cy, scale=1.0, rot=0.0, anchor=(0.5, 0.5), alpha=1.0):
    """Place an RGBA image with its anchor point at (cx, cy), uniform scale, rotation."""
    if scale <= 0.001 or alpha <= 0.003:
        return
    if alpha < 1:
        img = img.copy()
        img.putalpha(img.getchannel('A').point(lambda v: int(v * alpha)))
    w, h = img.size
    M = T(cx, cy) @ R(rot) @ S(scale, scale) @ T(-anchor[0] * w, -anchor[1] * h)
    paste_affine(canvas, img.convert('RGBa'), M)


def supersample(size, draw_fn, k=3):
    big = Image.new('RGBA', (size[0] * k, size[1] * k), (0, 0, 0, 0))
    draw_fn(ImageDraw.Draw(big), k)
    return big.resize(size, Image.LANCZOS)


# ---------------------------------------------------------------- static layers
study = Image.open(A('character', 'study.png')).convert('RGBA')

# a low side table in the set's flat style, for the laptop to sit on
TABLE_X0, TABLE_X1, TABLE_TOP = 742, 1318, 842


def make_table():
    w, h = TABLE_X1 - TABLE_X0 + 40, 170
    def d(dr, k):
        ow = 5 * k
        wood, apron, leg = (196, 132, 80), (170, 108, 62), (150, 94, 54)
        # legs
        for lx in (40, w - 70):
            dr.rounded_rectangle([lx * k, 40 * k, (lx + 30) * k, 158 * k], radius=8 * k, fill=leg, outline=OUTLINE, width=ow)
        # apron
        dr.rounded_rectangle([26 * k, 26 * k, (w - 26) * k, 58 * k], radius=6 * k, fill=apron, outline=OUTLINE, width=ow)
        # top
        dr.rounded_rectangle([8 * k, 4 * k, (w - 8) * k, 34 * k], radius=12 * k, fill=wood, outline=OUTLINE, width=ow)
        dr.line([20 * k, 13 * k, (w - 20) * k, 13 * k], fill=(222, 166, 112), width=4 * k)
    return supersample((w, h), d)


table_img = make_table()


def soft_ellipse(w, h, alpha):
    im = Image.new('RGBA', (w + 40, h + 40), (0, 0, 0, 0))
    ImageDraw.Draw(im).ellipse([20, 20, 20 + w, 20 + h], fill=(70, 40, 20, alpha))
    return im.filter(ImageFilter.GaussianBlur(8))


shadow_img = soft_ellipse(380, 46, 90)
table_shadow = soft_ellipse(560, 40, 70)

# the "before" render: the session's frame with the title set to "Hello, World"
after_src = Image.open(A('stills', 'session-02.png')).convert('RGBA')


def make_before():
    im = after_src.copy()
    dr = ImageDraw.Draw(im)
    dr.rectangle([180, 225, 780, 320], fill=(16, 20, 24, 255))
    # match the size of "Hello, Montagent" (x 216-744, descender bottom 306)
    size = 64
    for s in range(40, 120):
        f = ImageFont.truetype(F_BOLD, s)
        b = f.getbbox('Hello, Montagent', anchor='ls')
        if b[2] - b[0] >= 529:
            size = s
            break
    f = ImageFont.truetype(F_BOLD, size)
    b = f.getbbox('Hello, Montagent', anchor='ls')
    baseline = 306 - b[3]
    dr.text((480, baseline), 'Hello, World', font=f, fill=(245, 240, 230, 255), anchor='ms')
    return im


before_src = make_before()
VID_W, VID_H = 540, 304
vid_before = before_src.resize((VID_W, VID_H), Image.LANCZOS)
vid_after = after_src.resize((VID_W, VID_H), Image.LANCZOS)

# laptop and the file shown on its screen
laptop_src = Image.open(A('character', 'laptop.png')).convert('RGBA')
SCR = (120, 41, 587, 312)
mono = ImageFont.truetype(F_MONO, 25)
ui = ImageFont.truetype(F_REG, 17)
C_KEY, C_STR, C_PUN = (125, 196, 255), (236, 214, 120), (230, 226, 216)


def make_screen(typed, state, cursor_on):
    """state: 'plain' | 'select' | 'typing'. typed = text replacing World."""
    sw, sh = SCR[2] - SCR[0], SCR[3] - SCR[1]
    k = 2
    big = Image.new('RGBA', (sw * k, sh * k), (27, 29, 38, 255))
    dr = ImageDraw.Draw(big)
    m2 = ImageFont.truetype(F_MONO, 25 * k)
    u2 = ImageFont.truetype(F_REG, 17 * k)
    dr.rectangle([0, 0, sw * k, 32 * k], fill=(44, 47, 60))
    for i, c in enumerate([(255, 95, 87), (254, 188, 46), (40, 200, 64)]):
        dr.ellipse([(12 + 18 * i) * k, 11 * k, (22 + 18 * i) * k, 21 * k], fill=c)
    dr.text((sw * k / 2, 16 * k), 'hello.montagent.json', font=u2, fill=(200, 200, 210), anchor='mm')
    cw = m2.getlength('M')
    lh = 40 * k
    x0, y0 = 14 * k, 52 * k

    def seg(x, y, text, col):
        dr.text((x, y), text, font=m2, fill=col)
        return x + cw * len(text)

    y = y0
    x = seg(x0, y, '{', C_PUN); x = seg(x, y, '"id"', C_KEY); x = seg(x, y, ': ', C_PUN); x = seg(x, y, '"hello"', C_STR); seg(x, y, ',', C_PUN)
    y += lh
    x = seg(x0, y, ' "type"', C_KEY); x = seg(x, y, ': ', C_PUN); x = seg(x, y, '"text"', C_STR); seg(x, y, ',', C_PUN)
    y += lh
    x = seg(x0, y, ' "size"', C_KEY); x = seg(x, y, ': ', C_PUN); x = seg(x, y, '64', (190, 150, 255)); seg(x, y, ',', C_PUN)
    y += lh
    # the edited line gets a soft highlight
    if state != 'plain' or typed != 'World':
        dr.rectangle([0, y - 6 * k, sw * k, y + lh - 8 * k], fill=(52, 46, 44))
    x = seg(x0, y, ' "text"', C_KEY); x = seg(x, y, ': ', C_PUN); x = seg(x, y, '"Hello, ', C_STR)
    wx = x
    if state == 'select':
        dr.rectangle([wx, y - 4 * k, wx + cw * len(typed), y + 32 * k], fill=SIGNAL)
        x = seg(x, y, typed, (255, 255, 255))
    else:
        x = seg(x, y, typed, C_STR)
    cx = x
    x = seg(x, y, '"', C_STR); seg(x, y, '}', C_PUN)
    if state == 'typing' and cursor_on:
        dr.rectangle([cx, y - 4 * k, cx + 3 * k, y + 32 * k], fill=SIGNAL)
    return big.resize((sw, sh), Image.LANCZOS)


def make_laptop(screen):
    im = laptop_src.copy()
    sc = screen.copy()
    # round the screen corners slightly to sit inside the bezel
    mask = Image.new('L', sc.size, 0)
    ImageDraw.Draw(mask).rounded_rectangle([0, 0, sc.size[0] - 1, sc.size[1] - 1], radius=6, fill=255)
    im.paste(sc, (SCR[0], SCR[1]), mask)
    return im


LAPTOP_SCALE = 0.75
LAPTOP_X, LAPTOP_Y = 1022, TABLE_TOP + 2  # bottom-centre anchor

# the video frame hung on the wall
FR_X, FR_Y = 1330, 104  # top-left of the picture
BAR_H = 34


frame_border = None
render_font = ImageFont.truetype(F_BOLD, 24)


def make_frame(t_render_p, play_p, show_after, flash=0.0):
    pad = 12
    fw, fh = VID_W + pad * 2, VID_H + pad + BAR_H + 6
    im = Image.new('RGBA', (fw + 8, fh + 8), (0, 0, 0, 0))
    k = 3
    def d(dr, k):
        dr.rounded_rectangle([4 * k, 4 * k, (fw + 4) * k, (fh + 4) * k], radius=18 * k, fill=INK, outline=OUTLINE, width=4 * k)
    global frame_border
    if frame_border is None:
        frame_border = supersample(im.size, d)
    im = frame_border.copy()
    # picture
    if t_render_p is None:
        pic = vid_after if show_after else vid_before
        im.alpha_composite(pic, (pad + 4, pad + 4))
        if show_after and flash > 0:
            im.alpha_composite(Image.new('RGBA', (VID_W, VID_H), (255, 255, 255, int(200 * flash))), (pad + 4, pad + 4))
    else:
        # rendering: the old picture dims while the bar fills, with a spinner of dots
        pic = Image.blend(vid_before, Image.new('RGBA', (VID_W, VID_H), INK + (255,)), 0.92 * ease(t_render_p * 5))
        im.alpha_composite(pic, (pad + 4, pad + 4))
        dr = ImageDraw.Draw(im)
        cx, cy = pad + 4 + VID_W / 2, pad + 4 + VID_H / 2 - 14
        spin = t_render_p * 9
        for i in range(8):
            a = 2 * math.pi * i / 8
            lvl = ((i - spin * 8) % 8) / 8
            col = tuple(int(SIGNAL[j] * (1 - lvl) + 60 * lvl) for j in range(3))
            r = 7 - 3 * lvl
            px, py = cx + 30 * math.cos(a), cy + 30 * math.sin(a)
            dr.ellipse([px - r, py - r, px + r, py + r], fill=col)
        dr.text((cx, cy + 62), 'Rendering…', font=render_font, fill=PAPER, anchor='mm')
    # control bar: play triangle and progress
    dr = ImageDraw.Draw(im)
    by = pad + 4 + VID_H + BAR_H // 2 + 2
    bx0 = pad + 4
    dr.polygon([(bx0 + 4, by - 9), (bx0 + 4, by + 9), (bx0 + 19, by)], fill=PAPER)
    tx0, tx1 = bx0 + 36, bx0 + VID_W - 4
    dr.rounded_rectangle([tx0, by - 3, tx1, by + 3], radius=3, fill=(70, 74, 82))
    p = t_render_p if t_render_p is not None else play_p
    dr.rounded_rectangle([tx0, by - 3, tx0 + max(6, (tx1 - tx0) * p), by + 3], radius=3, fill=SIGNAL)
    return im


# arrow from laptop to frame
P0, P1, P2, P3 = (1238, 560), (1420, 640), (1610, 600), (1600, 486)


def bez(u):
    a = (1 - u) ** 3; b = 3 * (1 - u) ** 2 * u; c = 3 * (1 - u) * u * u; d = u ** 3
    return (a * P0[0] + b * P1[0] + c * P2[0] + d * P3[0], a * P0[1] + b * P1[1] + c * P2[1] + d * P3[1])


def dot_sprite(r, col):
    return supersample((2 * r + 2, 2 * r + 2), lambda dr, k: dr.ellipse([k, k, (2 * r + 1) * k, (2 * r + 1) * k], fill=col))


dot_small = dot_sprite(5, SIGNAL + (255,))
dot_big = dot_sprite(9, SIGNAL + (255,))
arrow_head = supersample((44, 40), lambda dr, k: dr.polygon([(2 * k, 2 * k), (42 * k, 20 * k), (2 * k, 38 * k), (10 * k, 20 * k)], fill=SIGNAL))


def draw_arrow(canvas, grow, flow, alpha):
    """grow: how much of the path is drawn (0..1); flow: phase of travelling dots."""
    if grow <= 0 or alpha <= 0:
        return
    n = 22
    for i in range(n + 1):
        u = i / n
        if u > grow:
            break
        x, y = bez(u)
        place(canvas, dot_small, x, y, alpha=alpha * 0.55)
    # travelling bright dots
    for j in range(3):
        u = (flow + j / 3) % 1.0
        if u <= grow:
            x, y = bez(u)
            place(canvas, dot_big, x, y, scale=0.8 + 0.4 * math.sin(math.pi * u), alpha=alpha)
    u = min(grow, 1.0)
    x, y = bez(u)
    x2, y2 = bez(max(u - 0.02, 0))
    ang = math.degrees(math.atan2(y - y2, x - x2))
    place(canvas, arrow_head, x, y, rot=ang, anchor=(0.6, 0.5), alpha=alpha)


# question mark
def make_qmark():
    f = ImageFont.truetype(F_BOLD, 170)
    im = Image.new('RGBA', (200, 230), (0, 0, 0, 0))
    ImageDraw.Draw(im).text((100, 120), '?', font=f, fill=SIGNAL, anchor='mm', stroke_width=7, stroke_fill=OUTLINE)
    return im


qmark = make_qmark()

# brand
lockup = Image.open(A('brand', 'lockup.png')).convert('RGBA')
LOCK_W = 690
lockup_s = lockup.resize((LOCK_W, round(LOCK_W * lockup.height / lockup.width)), Image.LANCZOS)


def make_tagline():
    f = ImageFont.truetype(F_REG, 40)
    a, b = 'Edit the file. Render it ', 'again.'
    wa, wb = f.getlength(a), f.getlength(b)
    im = Image.new('RGBA', (int(wa + wb) + 20, 70), (0, 0, 0, 0))
    dr = ImageDraw.Draw(im)
    dr.text((10, 50), a, font=f, fill=INK, anchor='ls')
    dr.text((10 + wa, 50), b, font=f, fill=SIGNAL, anchor='ls')
    return im


tagline = make_tagline()
BRAND_CX, BRAND_CY = 1196, 400

# ---------------------------------------------------------------- performance (owl)
OWL_SCALE = 0.68
OWL_X, OWL_FEET_Y = 430, 976  # where the torso pivot (feet) stands
TP = rig['parts']['torso']['pivot']

viseme_map = rig['visemes']
vis = line['visemes']


def mouth_at(t):
    lt = (t - VOICE_AT) * 1000 + 20  # show a hair early so the beak leads the sound
    cur = None
    for ms, vid in vis:
        if ms <= lt:
            cur = vid
        else:
            break
    if cur is None or lt < 0:
        return None
    return viseme_map[str(cur)]


BLINKS = [2.92, 4.95, 6.62, 8.75]


def eyes_closed(t):
    return any(b <= t < b + 4 / FPS for b in BLINKS)


def word_time(i):
    return VOICE_AT + line['words'][i]['start'] / 1000


# hops in from the left
HOPS = [(0.00, 0.40, -340, 110, 120), (0.44, 0.86, 110, OWL_X, 85)]


def pose(t):
    p = {}
    # base position and hop
    x, lift, squash = OWL_X, 0.0, 0.0
    flap = 0.0
    for (t0, t1, xa, xb, hgt) in HOPS:
        if t < t0:
            x = xa
            break
        if t0 <= t <= t1:
            u = (t - t0) / (t1 - t0)
            x = xa + (xb - xa) * u
            lift = hgt * 4 * u * (1 - u)
            flap = math.sin(math.pi * u)
            break
        x = xb
    # landings: squash after each touchdown
    for td in (0.40, 0.86):
        if td <= t < td + 0.22:
            u = (t - td) / 0.22
            squash += 0.09 * math.sin(math.pi * u) * (1 - u * 0.3)
    # take-off anticipation of the second hop
    # emphasis bounces on "Just" and "again"
    for te, amt in ((word_time(8) - 0.05, 0.035), (word_time(16), 0.035), (6.55, 0.05)):
        if te <= t < te + 0.3:
            u = (t - te) / 0.3
            squash += amt * math.sin(math.pi * u)
    breath = 0.008 * math.sin(2 * math.pi * t / 1.7)
    p['x'] = x
    p['y'] = OWL_FEET_Y - lift
    p['sx'] = 1 + squash * 0.6 - breath * 0.4
    p['sy'] = 1 - squash + breath
    p['lift'] = lift

    qa, qb = word_time(0) - 0.12, word_time(7) + 0.45  # question span (~0.93 .. 2.62)
    a0 = word_time(8) - 0.02    # "Just"  ~3.0
    a_file = word_time(11)      # "file"  ~3.7
    a_and = word_time(12) - 0.1  # ~4.26
    end = 6.3
    br = 7.25

    p['torso'] = track(t, [(0, 0), (qa, 0), (qa + 0.3, -2.0), (qb, -2.0), (qb + 0.3, 0), (a0, 0), (a0 + 0.3, 2.0),
                           (a_and, 2.0), (a_and + 0.35, 3.0), (end, 3.0), (end + 0.4, 0), (br, 0), (br + 0.4, 1.5)])
    p['torso'] += 0.8 * math.sin(2 * math.pi * t / 2.3)
    p['head'] = track(t, [(0, 0), (qa, 0), (qa + 0.3, -11), (qb - 0.1, -11), (qb + 0.3, 0), (a0, 0), (a0 + 0.3, 7),
                          (a_and, 7), (a_and + 0.35, 11), (end, 11), (end + 0.4, -3), (br, -3), (br + 0.4, 6)])
    p['head'] += 1.6 * math.sin(2 * math.pi * t / 1.9 + 0.7)
    # question beat: a nod-tilt on "video?"
    tv = word_time(7)
    if tv <= t < tv + 0.45:
        p['head'] += -4 * math.sin(math.pi * (t - tv) / 0.45)

    # arms: + raises the viewer's-left arm; - raises the viewer's-right arm
    hop_ual = 38 * flap
    p['ual'] = hop_ual + track(t, [(0, 0), (qa, 0), (qa + 0.3, 24), (qb, 24), (qb + 0.35, 2), (a0, 2), (a0 + 0.3, -14),
                                   (end, -14), (end + 0.25, 30), (br - 0.1, 30), (br + 0.35, -16)])
    p['fal'] = track(t, [(0, 0), (qa, 0), (qa + 0.3, 42), (qb, 42), (qb + 0.35, 6), (a0, 6), (a0 + 0.3, 12),
                         (end, 12), (end + 0.25, 25), (br - 0.1, 25), (br + 0.35, 18)])
    p['uar'] = -38 * flap + track(t, [(0, 0), (qa, 0), (qa + 0.3, -24), (qb, -24), (qb + 0.35, -2), (a0 - 0.05, -2),
                                      (a0 + 0.25, -24), (a_and, -24), (a_and + 0.35, -52), (end, -52),
                                      (end + 0.25, -30), (br - 0.1, -30), (br + 0.35, -38)])
    p['far'] = track(t, [(0, 0), (qa, 0), (qa + 0.3, -42), (qb, -42), (qb + 0.35, -6), (a0 - 0.05, -6),
                         (a0 + 0.25, -2), (a_and, -2), (a_and + 0.35, -6), (end, -6), (end + 0.25, -25),
                         (br - 0.1, -25), (br + 0.35, -12)])
    # wrist-flick while asking, small tap on the laptop while it types
    if qa + 0.3 < t < qb:
        w = 4 * math.sin(2 * math.pi * (t - qa) / 0.8)
        p['fal'] += w
        p['far'] -= w
    if a0 + 0.3 < t < a_and:
        p['far'] += -3 * math.sin(2 * math.pi * (t - a0) / 0.25)
    # idle sway so the arms are never frozen
    p['ual'] += 1.8 * math.sin(2 * math.pi * t / 2.1)
    p['uar'] -= 1.8 * math.sin(2 * math.pi * t / 2.4 + 1.1)
    p['fal'] += 1.5 * math.sin(2 * math.pi * t / 1.6 + 0.4)
    p['far'] -= 1.5 * math.sin(2 * math.pi * t / 1.8 + 2.0)
    return p


def part_matrix(name, M):
    pv = rig['parts'][name]['pivot']
    w, h = parts[name].size
    return M @ T(pv[0] - w / 2, pv[1] - h / 2)


def joint(Mparent, name, deg):
    pv = rig['parts'][name]['pivot']
    return Mparent @ T(*pv) @ R(deg) @ T(-pv[0], -pv[1])


def draw_owl(canvas, p, t):
    Mt = T(p['x'], p['y']) @ R(p['torso']) @ S(OWL_SCALE * p['sx'], OWL_SCALE * p['sy']) @ T(-TP[0], -TP[1])
    Mh = joint(Mt, 'head', p['head'])
    Mual = joint(Mt, 'upper_arm_left', p['ual'])
    Muar = joint(Mt, 'upper_arm_right', p['uar'])
    Mfal = joint(Mual, 'forearm_left', p['fal'])
    Mfar = joint(Muar, 'forearm_right', p['far'])
    mats = {'torso': Mt, 'head': Mh, 'upper_arm_left': Mual, 'upper_arm_right': Muar,
            'forearm_left': Mfal, 'forearm_right': Mfar}
    mouth = mouth_at(t)
    for name in rig['draw_order']:
        if name.startswith('mouth_'):
            if mouth is None or name != 'mouth_' + mouth:
                continue
            M = Mh
        elif name == 'eyes_closed':
            if not eyes_closed(t):
                continue
            M = Mh
        else:
            M = mats[name]
        paste_affine(canvas, parts_pm[name], part_matrix(name, M))


# ---------------------------------------------------------------- scene timing
T_FRAME_IN = word_time(7) - 0.25           # the video pops in as Hoot says "video" (~1.93)
T_LAPTOP_IN = word_time(8) - 0.1           # laptop on "Just" (~2.93)
T_SELECT = word_time(9)                    # "edit": select the word (~3.39)
T_DELETE = T_SELECT + 0.22
T_TYPE0 = T_DELETE + 0.06
T_TYPE_STEP = 0.05
T_RENDER0 = word_time(13) - 0.05           # arrow grows on "Montagent" (~4.54)
T_WIPE0, T_WIPE1 = word_time(14) - 0.1, word_time(16) + 0.1   # wipe across "renders it again"
T_OUT = 6.95
T_BRAND = 7.35


def pop(t, t0, dur=0.35):
    if t < t0:
        return 0.0
    return ease_out_back((t - t0) / dur)


def pop_out(t, t0, dur=0.28):
    if t < t0:
        return 1.0
    u = min((t - t0) / dur, 1.0)
    return max(0.0, 1 - u * u * (2.7 * u - 1.7) if u < 1 else 0.0)


laptop_cache = {}


def laptop_at(t):
    word = 'Montagent'
    if t < T_SELECT:
        key = ('World', 'plain', False)
    elif t < T_DELETE:
        key = ('World', 'select', False)
    else:
        n = int((t - T_TYPE0) / T_TYPE_STEP) + 1 if t >= T_TYPE0 else 0
        n = max(0, min(n, len(word)))
        cursor = (int(t * 4) % 2 == 0) or n < len(word)
        if t > T_WIPE1 + 0.3:
            cursor = False
        key = (word[:n], 'typing', cursor)
    if key not in laptop_cache:
        laptop_cache[key] = make_laptop(make_screen(*key)).resize(
            (round(laptop_src.width * LAPTOP_SCALE), round(laptop_src.height * LAPTOP_SCALE)), Image.LANCZOS)
    return laptop_cache[key]


def render(fi):
    t = fi / FPS
    c = study.copy()
    # table
    c.alpha_composite(table_shadow, (TABLE_X0 - 30, 968))
    c.alpha_composite(table_img, (TABLE_X0 - 20, TABLE_TOP - 4))
    # laptop
    s = pop(t, T_LAPTOP_IN) * pop_out(t, T_OUT)
    if s > 0:
        place(c, laptop_at(t), LAPTOP_X, LAPTOP_Y, scale=s, anchor=(0.5, 1.0))
    # video frame
    s = pop(t, T_FRAME_IN) * pop_out(t, T_OUT + 0.05)
    if s > 0:
        if t < T_WIPE0:
            fr = make_frame(None, ((t - T_FRAME_IN) / 6.0) % 1.0, False)
        elif t < T_WIPE1:
            fr = make_frame((t - T_WIPE0) / (T_WIPE1 - T_WIPE0), 0, False)
        else:
            u = t - T_WIPE1
            fr = make_frame(None, min(1.0, 0.02 + u / 4.0), True, flash=max(0.0, 1 - u / 0.25))
            if u < 0.3:
                s *= 1 + 0.05 * math.sin(math.pi * u / 0.3)
        place(c, fr, FR_X - 16 + fr.width / 2, FR_Y - 16 + fr.height / 2, scale=s)
    # arrow: file -> video
    if t >= T_RENDER0:
        grow = ease((t - T_RENDER0) / 0.45)
        alpha = 1.0 - ease((t - (T_OUT - 0.25)) / 0.3)
        draw_arrow(c, grow, (t - T_RENDER0) / 0.9, alpha)
    # question mark
    qa = word_time(0) + 0.15
    qb = word_time(7) + 0.55
    if qa <= t < qb + 0.25:
        s = pop(t, qa, 0.4) * pop_out(t, qb)
        place(c, qmark, 735 + 6 * math.sin(t * 5), 205 + 4 * math.sin(t * 3.3), scale=0.75 * s, rot=12 + 6 * math.sin(t * 4))
    # owl and shadow
    p = pose(t)
    sh = max(0.35, 1 - p['lift'] / 160)
    place(c, shadow_img, p['x'], OWL_FEET_Y + 4, scale=sh, alpha=sh)
    draw_owl(c, p, t)
    # brand
    if t >= T_BRAND:
        s = pop(t, T_BRAND, 0.45)
        place(c, lockup_s, BRAND_CX, BRAND_CY, scale=s)
        a = ease((t - (T_BRAND + 0.4)) / 0.4)
        place(c, tagline, BRAND_CX, BRAND_CY + 135 + 20 * (1 - a), alpha=a)
    return c.convert('RGB')


if __name__ == '__main__':
    if len(sys.argv) > 1 and sys.argv[1] == 'stills':
        out = sys.argv[2]
        os.makedirs(out, exist_ok=True)
        for tt in sys.argv[3:]:
            fi = int(round(float(tt) * FPS))
            render(fi).save(os.path.join(out, f'f{fi:03d}.png'))
    else:
        for fi in range(NF):
            sys.stdout.buffer.write(render(fi).tobytes())
