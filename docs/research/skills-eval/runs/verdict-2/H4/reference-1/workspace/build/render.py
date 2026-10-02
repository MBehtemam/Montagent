"""Render the Montagent 15 s square ad as raw RGB frames on stdout."""
import os, sys, glob
from PIL import Image, ImageDraw, ImageFont, ImageFilter

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
TMP = os.environ["TMPDIR"]
W = H = 1080
FPS = 30
N = 15 * FPS

INK = (0x10, 0x14, 0x18)
PAPER = (0xF5, 0xF0, 0xE6)
SIGNAL = (0xFF, 0x5A, 0x36)

BOLD = os.path.join(ROOT, "fonts/Inter-Bold.ttf")
REG = os.path.join(ROOT, "fonts/Inter-Regular.ttf")
_fonts = {}
def font(path, size):
    k = (path, size)
    if k not in _fonts:
        _fonts[k] = ImageFont.truetype(path, size)
    return _fonts[k]

def clamp(x, a=0.0, b=1.0):
    return max(a, min(b, x))

def ease_out(x):
    x = clamp(x)
    return 1 - (1 - x) ** 3

def ease_io(x):
    x = clamp(x)
    return 3 * x * x - 2 * x * x * x

def rounded_mask(size, r):
    m = Image.new("L", size, 0)
    ImageDraw.Draw(m).rounded_rectangle((0, 0, size[0] - 1, size[1] - 1), r, fill=255)
    return m

def card(frame, img, x, y, alpha=1.0, r=22):
    """Paste img with rounded corners at (x, y)."""
    m = rounded_mask(img.size, r)
    if alpha < 1:
        m = m.point(lambda v: int(v * alpha))
    frame.paste(img, (int(round(x)), int(round(y))), m)

def text_runs(frame, runs, x, y, size, path=BOLD, alpha=1.0, anchor_center=False):
    """Draw [(text, colour), ...] on one line. Returns width."""
    f = font(path, size)
    total = sum(f.getlength(t) for t, _ in runs)
    if anchor_center:
        x = x - total / 2
    layer = Image.new("RGBA", frame.size, (0, 0, 0, 0))
    d = ImageDraw.Draw(layer)
    cx = x
    for t, c in runs:
        d.text((cx, y), t, font=f, fill=c + (int(255 * alpha),))
        cx += f.getlength(t)
    frame.alpha_composite(layer) if frame.mode == "RGBA" else frame.paste(layer, (0, 0), layer)
    return total

def entry(t, t0, dur=0.35):
    """Fade + rise for an element entering at t0. Returns (alpha, dy)."""
    p = ease_out((t - t0) / dur)
    return p, (1 - p) * 36

# ---------------------------------------------------------------- assets
session01 = Image.open(os.path.join(ROOT, "stills/session-01.png")).convert("RGB")
session02 = Image.open(os.path.join(ROOT, "stills/session-02.png")).convert("RGB")
lockup = Image.open(os.path.join(ROOT, "brand/lockup.png")).convert("RGBA")
ask_frames = sorted(glob.glob(os.path.join(TMP, "ask", "f*.png")))  # 6.0 s .. 10.6 s @ 25 fps
ASK_T0 = 6.0

# cursor x per ask frame (rightmost bright pixel on the prompt line)
_cursor = {}
def cursor_x(path):
    if path not in _cursor:
        im = Image.open(path).convert("L").crop((40, 272, 1920, 294))
        bb = im.point(lambda v: 255 if v > 150 else 0).getbbox()
        _cursor[path] = 40 + (bb[2] if bb else 0)
    return _cursor[path]

# Margins: everything stays inside 72 px (> 5 % of 1080 = 54 px).
M = 72
CARD_W = W - 2 * M  # 936

# ---------------------------------------------------------------- scenes
# Beats every 0.5 s. Scene boundaries: 0 | 2.5 | 4.0 | 7.0 | 9.5 | 12.5 | 15
def scene_hook(t):
    fr = Image.new("RGB", (W, H), INK)
    size = 104
    lines = [
        (0.0, [("Your AI agent", PAPER)]),
        (0.5, [("edits ", PAPER), ("video", SIGNAL)]),
        (1.0, [("like it edits code.", PAPER)]),
    ]
    lh = 124
    y0 = H / 2 - lh * len(lines) / 2 - 10
    for i, (t0, runs) in enumerate(lines):
        a, dy = entry(t, t0)
        if a <= 0:
            continue
        text_runs(fr, runs, M + 20, y0 + i * lh + dy, size, alpha=a)
    return fr

def kicker(fr, t, t0, words, colour=INK):
    a, dy = entry(t, t0, 0.25)
    text_runs(fr, [(words, colour)], M + 20, 150 + dy, 66, alpha=a)

def settle(fr, img, t, t0, y):
    """Hard cut on the beat: the card is there at once and settles from 104 % to 100 %."""
    sc = 1.0 + 0.04 * (1 - ease_out((t - t0) / 0.5))
    if sc > 1.0005:
        w, h = int(img.width * sc), int(img.height * sc)
        big = img.resize((w, h), Image.BILINEAR)
        img = big.crop(((w - img.width) // 2, (h - img.height) // 2,
                        (w - img.width) // 2 + img.width, (h - img.height) // 2 + img.height))
    card(fr, img, M, y)

def scene_ask(t):
    fr = Image.new("RGB", (W, H), PAPER)
    lt = t - 2.5
    kicker(fr, t, 2.5, "You ask.")
    # footage: typing from 6.4 s to 10.0 s, played over 1.35 s
    src_t = 6.4 + clamp(lt / 1.35) * 3.6
    idx = int(round((src_t - ASK_T0) * 25))
    idx = max(0, min(len(ask_frames) - 1, idx))
    src = Image.open(ask_frames[idx]).convert("RGB")
    s = 1.25
    ch = int(256 * s)
    win_w, win_h = CARD_W / s, ch / s
    # camera glides right with the typing; the cursor stays in view throughout
    end_left = cursor_x(ask_frames[-1]) + 60 - win_w
    left = clamp(ease_io((lt - 0.25) / 1.15) * end_left, 0, 1920 - win_w)
    top = 164
    crop = src.crop((int(left), top, int(left + win_w), int(top + win_h))).resize((CARD_W, ch), Image.LANCZOS)
    settle(fr, crop, t, 2.5, 400)
    return fr

def scene_edit(t):
    fr = Image.new("RGB", (W, H), PAPER)
    lt = t - 4.0
    kicker(fr, t, 4.0, "It edits the project file.")
    ch = 640
    # slow push-in from the whole edit to the added lines
    p = ease_io(lt / 3.0)
    s = 0.84 + p * 0.2
    win_w, win_h = CARD_W / s, ch / s
    cx = 597 + p * (40 + win_w / 2 - 597)
    cy = 551 + p * (520 - 551)
    left, top = cx - win_w / 2, cy - win_h / 2
    crop = session01.crop((int(left), int(top), int(left + win_w), int(top + win_h))).resize((CARD_W, ch), Image.LANCZOS)
    settle(fr, crop, t, 4.0, 300)
    return fr

def scene_result(t):
    fr = Image.new("RGB", (W, H), PAPER)
    kicker(fr, t, 7.0, "It renders the video.")
    # crop the rendered frame to its centre (16:9 kept) so the subtitle reads
    cw, chh = 768, 432
    cx, cy = 480, 300
    crop = session02.crop((cx - cw // 2, cy - chh // 2, cx + cw // 2, cy + chh // 2))
    img = crop.resize((CARD_W, int(CARD_W * chh / cw)), Image.LANCZOS)
    settle(fr, img, t, 7.0, 320)
    # file name beneath, like the session reported it
    a2, dy2 = entry(t, 7.5, 0.3)
    text_runs(fr, [("out/hello-text.mp4", INK)], M + 20, 320 + 527 + 36 + dy2, 34, path=REG, alpha=a2)
    return fr

def scene_claim(t):
    fr = Image.new("RGB", (W, H), INK)
    size = 92
    lines = [
        (9.5, [("Now your video is", PAPER)]),
        (9.75, [("a ", PAPER), ("file", SIGNAL), (" your agent", PAPER)]),
        (10.0, [("can read, check", PAPER)]),
        (10.0, [("and change.", PAPER)]),
    ]
    lh = 112
    y0 = H / 2 - lh * len(lines) / 2 - 8
    for i, (t0, runs) in enumerate(lines):
        a, dy = entry(t, t0)
        if a <= 0:
            continue
        text_runs(fr, runs, M + 20, y0 + i * lh + dy, size, alpha=a)
    return fr

# lockup geometry: mark occupies x 54..569, y 54..569; wordmark from x 759
LK_W = 800
LK_S = LK_W / (2650 - 54)
lk = lockup.crop((54, 54, 2650, 569))
lk = lk.resize((LK_W, int(round(lk.height * LK_S))), Image.LANCZOS)
MARK_PX = int(round((569 - 54) * LK_S))
mark_part = lk.crop((0, 0, MARK_PX + 2, lk.height))
word_part = lk.crop((MARK_PX + 2, 0, lk.width, lk.height))

def scene_brand(t):
    fr = Image.new("RGBA", (W, H), PAPER + (255,))
    lx = (W - LK_W) // 2
    ly = int(H / 2 - lk.height / 2 - 50)
    # four tiles, one per eighth-beat, popping in from scale 0.6
    half = mark_part.width // 2
    for i, (qx, qy) in enumerate([(0, 0), (1, 0), (0, 1), (1, 1)]):
        t0 = 12.5 + i * 0.0625
        p = ease_out((t - t0) / 0.3)
        if p <= 0:
            continue
        q = mark_part.crop((qx * half, qy * half, (qx + 1) * half, (qy + 1) * half))
        sc = 0.6 + 0.4 * p
        qs = q.resize((max(1, int(half * sc)), max(1, int(half * sc))), Image.LANCZOS)
        if p < 1:
            qs.putalpha(qs.getchannel("A").point(lambda v: int(v * p)))
        ox = lx + qx * half + (half - qs.width) / 2
        oy = ly + qy * half + (half - qs.height) / 2
        fr.alpha_composite(qs, (int(round(ox)), int(round(oy))))
    # wordmark wipes in from behind the mark on the next beat
    p = ease_out((t - 13.0) / 0.45)
    if p > 0:
        reveal = int(word_part.width * p)
        wp = word_part.crop((word_part.width - reveal, 0, word_part.width, word_part.height)) if False else word_part
        shift = (1 - p) * -60
        tmp = Image.new("RGBA", fr.size, (0, 0, 0, 0))
        tmp.alpha_composite(wp, (int(lx + MARK_PX + 2 + shift), ly))
        if p < 1:
            tmp.putalpha(tmp.getchannel("A").point(lambda v: int(v * p)))
        fr.alpha_composite(tmp)
    # tagline
    a, dy = entry(t, 13.5, 0.35)
    if a > 0:
        text_runs(fr, [("The video editor your agent drives.", INK)], W / 2, ly + lk.height + 70 + dy, 42,
                  path=REG, alpha=a, anchor_center=True)
    return fr.convert("RGB")

def frame_at(t):
    if t < 2.5:
        return scene_hook(t)
    if t < 4.0:
        return scene_ask(t)
    if t < 7.0:
        return scene_edit(t)
    if t < 9.5:
        return scene_result(t)
    if t < 12.5:
        return scene_claim(t)
    return scene_brand(t)

if __name__ == "__main__":
    only = sys.argv[1:]
    if only:  # preview stills: render.py t1 t2 ...
        for s in only:
            frame_at(float(s)).save(os.path.join(TMP, f"prev_{s}.png"))
        sys.exit()
    out = sys.stdout.buffer
    for i in range(N):
        out.write(frame_at(i / FPS).tobytes())
