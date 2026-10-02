"""Render the Montagent "one file" ad (Brief H1) frame by frame with Pillow.

Usage: python3.14 render.py OUT_DIR_FOR_RAW | ffmpeg ...   (writes rgb24 frames to stdout)
       python3.14 render.py --stills t1,t2,...  (writes PNG stills to $TMPDIR/h1/stills)
"""
import math
import os
import sys

from PIL import Image, ImageDraw, ImageFilter, ImageFont

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
TMP = os.path.join(os.environ.get("TMPDIR", "/tmp"), "h1")
REC_DIR = os.path.join(TMP, "rec")

W, H, FPS, DUR = 1920, 1080, 30, 14.0
INK = (16, 20, 24)
PAPER = (245, 240, 230)
SIGNAL = (255, 90, 54)

BOLD = os.path.join(ROOT, "fonts/Inter-Bold.ttf")
REG = os.path.join(ROOT, "fonts/Inter-Regular.ttf")


# ---------------------------------------------------------------- easing
def clamp(x, a=0.0, b=1.0):
    return max(a, min(b, x))


def prog(t, t0, t1):
    return clamp((t - t0) / (t1 - t0))


def ease_out_cubic(u):
    return 1 - (1 - u) ** 3


def ease_in_cubic(u):
    return u ** 3


def ease_in_out_cubic(u):
    return 4 * u ** 3 if u < 0.5 else 1 - (-2 * u + 2) ** 3 / 2


def ease_out_back(u, c1=2.2):
    c3 = c1 + 1
    return 1 + c3 * (u - 1) ** 3 + c1 * (u - 1) ** 2


def lerp(a, b, u):
    return a + (b - a) * u


# ---------------------------------------------------------------- sprites
def text_sprite(text, path, size, color, ss=1):
    """RGBA image cropped to the ink of `text` (with a little padding)."""
    font = ImageFont.truetype(path, size * ss)
    l, t, r, b = font.getbbox(text)
    pad = 4 * ss
    im = Image.new("RGBA", (r - l + 2 * pad, b - t + 2 * pad), color + (0,))
    ImageDraw.Draw(im).text((pad - l, pad - t), text, font=font, fill=color + (255,))
    return im


def rounded_mask(w, h, r, ss=4):
    m = Image.new("L", (w * ss, h * ss), 0)
    ImageDraw.Draw(m).rounded_rectangle((0, 0, w * ss - 1, h * ss - 1), r * ss, fill=255)
    return m.resize((w, h), Image.LANCZOS)


def ellipse_mask(w, h, ss=4):
    m = Image.new("L", (w * ss, h * ss), 0)
    ImageDraw.Draw(m).ellipse((0, 0, w * ss - 1, h * ss - 1), fill=255)
    return m.resize((w, h), Image.LANCZOS)


class Card:
    """A picture in a rounded frame with a hairline border and a soft drop shadow."""

    def __init__(self, w, h, radius=26, blur=34, dy=26, shadow_alpha=235):
        self.w, self.h, self.r = w, h, radius
        self.mask = rounded_mask(w, h, radius)
        border = Image.new("RGBA", (w, h), PAPER + (0,))
        bm = Image.new("L", (w * 4, h * 4), 0)
        d = ImageDraw.Draw(bm)
        d.rounded_rectangle((0, 0, w * 4 - 1, h * 4 - 1), radius * 4, fill=255)
        d.rounded_rectangle((8, 8, w * 4 - 9, h * 4 - 9), radius * 4 - 8, fill=0)
        bm = bm.resize((w, h), Image.LANCZOS).point(lambda v: v * 40 // 255)
        border.putalpha(bm)
        self.border = border
        m = blur * 3
        self.spad = m
        sh = Image.new("L", (w + 2 * m, h + 2 * m), 0)
        ImageDraw.Draw(sh).rounded_rectangle((m, m, m + w, m + h), radius, fill=shadow_alpha)
        sh = sh.filter(ImageFilter.GaussianBlur(blur))
        self.shadow = Image.new("RGBA", sh.size, (0, 0, 0, 0))
        self.shadow.putalpha(sh)
        self.dy = dy

    def draw(self, frame, content, x, y):
        x, y = round(x), round(y)
        frame.alpha_composite_safe(self.shadow, x - self.spad, y - self.spad + self.dy)
        pic = content.convert("RGBA").copy()
        pic.putalpha(self.mask)
        frame.alpha_composite_safe(pic, x, y)
        frame.alpha_composite_safe(self.border, x, y)


class Frame:
    def __init__(self, color):
        self.im = Image.new("RGBA", (W, H), color + (255,))

    def alpha_composite_safe(self, src, x, y, opacity=1.0):
        """alpha_composite that tolerates negative / off-frame positions."""
        x, y = round(x), round(y)
        if opacity <= 0:
            return
        if opacity < 1:
            src = src.copy()
            src.putalpha(src.getchannel("A").point(lambda v: round(v * opacity)))
        sx0, sy0 = max(0, -x), max(0, -y)
        dx0, dy0 = max(0, x), max(0, y)
        w = min(src.width - sx0, W - dx0)
        h = min(src.height - sy0, H - dy0)
        if w <= 0 or h <= 0:
            return
        if (sx0, sy0, w, h) != (0, 0, src.width, src.height):
            src = src.crop((sx0, sy0, sx0 + w, sy0 + h))
        self.im.alpha_composite(src, (dx0, dy0))

    def rect(self, box, color):
        ImageDraw.Draw(self.im).rectangle([round(v) for v in box], fill=color)


def scaled(sprite_big, scale_from_big):
    w = max(1, round(sprite_big.width * scale_from_big))
    h = max(1, round(sprite_big.height * scale_from_big))
    return sprite_big.resize((w, h), Image.LANCZOS)


# ---------------------------------------------------------------- assets
HEAD_SIZE = 230
timelines = text_sprite("Timelines.", BOLD, HEAD_SIZE, PAPER)
text_big = text_sprite("Text.", BOLD, HEAD_SIZE, SIGNAL, ss=2)  # 2x for clean scaling
subline = text_sprite("Your agent edits video the way it edits code.", REG, 56, PAPER)

head_font = ImageFont.truetype(BOLD, HEAD_SIZE)
_, cap_top, _, base = head_font.getbbox("T")
_, x_top, _, _ = head_font.getbbox("x")
HEAD_CY = 470  # centre of the headline ink
# strike-through sits in the middle of the x-height
strike_y = HEAD_CY + ((x_top + base) / 2 - (cap_top + base) / 2)
STRIKE_H = 24

LIST_SIZE = 44
items = ["Reads the format", "Edits one file", "Checks the frames", "Renders"]
item_num = [text_sprite(str(i + 1), BOLD, LIST_SIZE, PAPER) for i in range(4)]
item_lbl = [text_sprite(s, BOLD, LIST_SIZE, PAPER) for s in items]
dot = Image.new("RGBA", (18, 18), SIGNAL + (0,))
dot.putalpha(ellipse_mask(18, 18))
list_font = ImageFont.truetype(BOLD, LIST_SIZE)
LIST_CAP = list_font.getbbox("R")  # for vertical alignment on cap height

REC_W, REC_H = 1184, 666
REC_X, REC_Y = W - 56 - REC_W, (H - REC_H) // 2
rec_card = Card(REC_W, REC_H)
_rec_cache = {}


def rec_frame(i):
    i = int(clamp(i, 0, 119))
    if i not in _rec_cache:
        _rec_cache.clear()
        _rec_cache[i] = Image.open(os.path.join(REC_DIR, f"{i + 1:04d}.png")).convert("RGBA")
    return _rec_cache[i]


SHOT_W, SHOT_H = 820, 461
SHOT_Y = 250
SHOT_LX, SHOT_RX = 100, W - 100 - SHOT_W
shot_card = Card(SHOT_W, SHOT_H, radius=22)
shot1 = Image.open(os.path.join(ROOT, "stills/session-01.png")).convert("RGBA").resize((SHOT_W, SHOT_H), Image.LANCZOS)
shot2 = Image.open(os.path.join(ROOT, "stills/session-02.png")).convert("RGBA").resize((SHOT_W, SHOT_H), Image.LANCZOS)
cap1 = text_sprite("One line changed.", BOLD, 50, PAPER)
cap2 = text_sprite("One video out.", BOLD, 50, PAPER)
CAP_Y = SHOT_Y + SHOT_H + 64

# --- the mark, built from the pack's construction (1024 box, 460 tiles, 40 gap, r 96)
TILE_BIG = 460  # tiles rendered at mark-box units, then scaled
tile_sq = Image.new("RGBA", (TILE_BIG, TILE_BIG), INK + (0,))
tile_sq.putalpha(rounded_mask(TILE_BIG, TILE_BIG, 96))
tile_ci = Image.new("RGBA", (TILE_BIG, TILE_BIG), SIGNAL + (0,))
tile_ci.putalpha(ellipse_mask(TILE_BIG, TILE_BIG))
# order of arrival: top-left, top-right (circle), bottom-left, bottom-right
TILES = [((0, 0), tile_sq), ((500, 0), tile_ci), ((0, 500), tile_sq), ((500, 500), tile_sq)]
_tile_cache = {}


def tile_at(img, px):
    key = (id(img), px)
    if key not in _tile_cache:
        _tile_cache[key] = img.resize((px, px), Image.LANCZOS)
    return _tile_cache[key]


# lockup geometry taken from brand/lockup.png: tiles' ink spans 515 px there, the
# wordmark ink starts 190 px right of the mark ink, at wordmark.png's native scale.
wordmark = Image.open(os.path.join(ROOT, "brand/wordmark.png")).convert("RGBA")
wm_ink = wordmark.getchannel("A").getbbox()
wordmark = wordmark.crop(wm_ink)
_lock = Image.open(os.path.join(ROOT, "brand/lockup.png")).getchannel("A")
LOCK_MARK = _lock.crop((0, 0, 700, _lock.height)).getbbox()
LOCK_WORD = _lock.crop((700, 0, _lock.width, _lock.height)).getbbox()
LOCK_SPAN = LOCK_MARK[2] - LOCK_MARK[0]
WORD_DX = (LOCK_WORD[0] + 700 - LOCK_MARK[2]) / LOCK_SPAN  # gap, in mark spans
WORD_DY = (LOCK_WORD[1] - LOCK_MARK[1]) / LOCK_SPAN  # wordmark top vs mark top
WORD_SCALE = 1.0 / LOCK_SPAN  # wordmark px per mark-span px
WORD_W = wordmark.width * WORD_SCALE  # in mark spans

MARK_SPAN_A = 380  # centred, while assembling
MARK_SPAN_B = 250  # in the lockup
LOCK_W = MARK_SPAN_B * (1 + WORD_DX + WORD_W)
LOCK_X = (W - LOCK_W) / 2
LOCK_Y = 400 - MARK_SPAN_B / 2
wm_final = wordmark.resize((round(wordmark.width * WORD_SCALE * MARK_SPAN_B),
                            round(wordmark.height * WORD_SCALE * MARK_SPAN_B)), Image.LANCZOS)
tagline = text_sprite("The video editor your agent drives.", REG, 54, INK)
TAG_Y = LOCK_Y + MARK_SPAN_B + 120


# ---------------------------------------------------------------- scenes
def scene_intro(f, t):
    # "Timelines." + strike bar, then the drop
    drop = ease_in_cubic(prog(t, 1.5, 1.93)) * (H - HEAD_CY + timelines.height)
    if t < 1.95:
        tx = (W - timelines.width) / 2
        ty = HEAD_CY - timelines.height / 2 + drop
        f.alpha_composite_safe(timelines, tx, ty)
        g = ease_in_out_cubic(prog(t, 0.5, 1.0))
        if g > 0:
            x0 = tx - 18
            x1 = x0 + g * (timelines.width + 36)
            y0 = strike_y - STRIKE_H / 2 + drop
            f.rect((x0, y0, x1, y0 + STRIKE_H - 1), SIGNAL)

    # out-move for both lines, 3.5–3.9
    out = ease_in_cubic(prog(t, 3.5, 3.9))
    if t >= 2.0 and out < 1:
        s = 0.3 + 0.7 * ease_out_back(prog(t, 2.0, 2.4))
        sp = scaled(text_big, s / 2)
        f.alpha_composite_safe(sp, (W - sp.width) / 2, HEAD_CY - sp.height / 2 - 70 * out, 1 - out)
    if t >= 2.5 and out < 1:
        u = ease_out_cubic(prog(t, 2.5, 2.9))
        y = HEAD_CY + 190 + 60 * (1 - u) - 70 * out
        f.alpha_composite_safe(subline, (W - subline.width) / 2, y, u * (1 - out))


def scene_product(f, t):
    u = ease_out_cubic(prog(t, 4.0, 4.45))
    x = lerp(W + 80, REC_X, u)
    rec_card.draw(f, rec_frame(round((t - 4.0) * FPS)), x, REC_Y)

    pitch = 100
    for i in range(4):
        t0 = 4.5 + i
        if t < t0:
            continue
        a = ease_out_cubic(prog(t, t0, t0 + 0.35))
        cy = H / 2 + (i - 1.5) * pitch
        dx = -40 * (1 - a)
        ds = ease_out_back(prog(t, t0, t0 + 0.3), 3.0)
        if ds > 0.01:
            d = dot.resize((max(1, round(18 * ds)),) * 2, Image.LANCZOS)
            f.alpha_composite_safe(d, 96 - d.width / 2, cy - d.height / 2)
        top = cy - (LIST_CAP[3] - LIST_CAP[1]) / 2 - 4  # sprites carry 4 px padding
        f.alpha_composite_safe(item_num[i], 136 + dx, top, a * 0.5)
        f.alpha_composite_safe(item_lbl[i], 182 + dx, top, a)


def scene_split(f, t):
    u = ease_out_cubic(prog(t, 8.0, 8.5))
    x = lerp(-SHOT_W - 80, SHOT_LX, u)
    shot_card.draw(f, shot1, x, SHOT_Y)
    f.alpha_composite_safe(cap1, x + (SHOT_W - cap1.width) / 2, CAP_Y)
    if t >= 8.5:
        u = ease_out_cubic(prog(t, 8.5, 9.0))
        x = lerp(W + 80, SHOT_RX, u)
        shot_card.draw(f, shot2, x, SHOT_Y)
        f.alpha_composite_safe(cap2, x + (SHOT_W - cap2.width) / 2, CAP_Y)


def tile_drop_offset(t, t0, fall):
    """Vertical offset (negative = above) for a tile dropping in at t0 with a small bounce."""
    u = prog(t, t0, t0 + 0.5)
    k = 0.5
    if u < k:
        return -fall * (1 - (u / k) ** 2)  # gravity: accelerates into place
    v = (u - k) / (1 - k)
    return -0.07 * MARK_SPAN_A * math.sin(math.pi * v) * (1 - v)  # one small hop, settling


def scene_brand(f, t):
    m = ease_in_out_cubic(prog(t, 12.0, 12.5))
    span = lerp(MARK_SPAN_A, MARK_SPAN_B, m)
    mx = lerp((W - MARK_SPAN_A) / 2, LOCK_X, m)
    my = lerp(H / 2 - MARK_SPAN_A / 2 - 30, LOCK_Y, m)
    k = span / 960
    tpx = round(460 * k)
    for i, ((ox, oy), img) in enumerate(TILES):
        t0 = 10.0 + 0.5 * i
        if t < t0:
            continue
        tx, ty = mx + ox * k, my + oy * k
        off = tile_drop_offset(t, t0, ty + tpx + 40)
        f.alpha_composite_safe(tile_at(img, tpx), tx, ty + off)

    if t >= 12.0:
        a = ease_out_cubic(prog(t, 12.0, 12.5))
        wx = LOCK_X + MARK_SPAN_B * (1 + WORD_DX) - 60 * (1 - a)
        wy = LOCK_Y + MARK_SPAN_B * WORD_DY
        # keep the word clear of the mark while it slides over
        mark_right = mx + span + 20
        wm = wm_final
        if mark_right > wx:
            cut = round(mark_right - wx)
            if cut >= wm.width:
                wm = None
            else:
                wm = wm.crop((cut, 0, wm.width, wm.height))
                wx += cut
        if wm is not None:
            f.alpha_composite_safe(wm, wx, wy, a)
    if t >= 12.5:
        a = ease_out_cubic(prog(t, 12.5, 12.95))
        f.alpha_composite_safe(tagline, (W - tagline.width) / 2, TAG_Y + 24 * (1 - a), a)


def render(t):
    if t < 10.0:
        f = Frame(INK)
        if t < 4.0:
            scene_intro(f, t)
        elif t < 8.0:
            scene_product(f, t)
        else:
            scene_split(f, t)
    else:
        f = Frame(PAPER)
        scene_brand(f, t)
    return f.im.convert("RGB")


def layout_checks():
    lbl_right = max(182 + s.width for s in item_lbl)
    assert lbl_right < REC_X - 40, (lbl_right, REC_X)
    assert LOCK_X > 80 and LOCK_X + LOCK_W < W - 80
    assert TAG_Y + tagline.height < H - 80
    print(f"list right edge {lbl_right}, rec x {REC_X}; lockup x {LOCK_X:.0f}-{LOCK_X + LOCK_W:.0f}",
          file=sys.stderr)


if __name__ == "__main__":
    layout_checks()
    if len(sys.argv) > 2 and sys.argv[1] == "--stills":
        out = os.path.join(TMP, "stills")
        os.makedirs(out, exist_ok=True)
        for s in sys.argv[2].split(","):
            render(float(s)).save(os.path.join(out, f"t{float(s):05.2f}.png"))
        sys.exit(0)
    n = round(DUR * FPS)
    for i in range(n):
        sys.stdout.buffer.write(render(i / FPS).tobytes())
        if i % 30 == 0:
            print(f"frame {i}/{n}", file=sys.stderr)
