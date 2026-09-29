"""Render the Montagent launch spot frames and pipe them to ffmpeg.

Timeline (120 BPM, beat = 0.5 s, 30 fps -> 15 frames per beat):
  0.0-2.0  hook words land one per beat (0, .5, 1, 1.5, 2) with spring overshoot
  2.0      "read." lands in Signal; line holds, clears 3.5-3.85
  4.0      screenshot 1 enters in a rounded, shadowed card; slow push-in
  7.0-7.5  crossfade to screenshot 2 inside the same card; push-in continues
  9.0      cut to a three-bar chart growing from its baseline
  10.0     cut to the lockup, revealed by a Signal bar wiping across (done 10.45)
  11-12    music fades out
"""
import math
import subprocess
import sys
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw, ImageFilter, ImageFont

ROOT = Path(__file__).resolve().parent.parent
W, H, FPS, DUR = 1920, 1080, 30, 12.0
N = int(DUR * FPS)

INK = (0x10, 0x14, 0x18)
PAPER = (0xF5, 0xF0, 0xE6)
SIGNAL = (0xFF, 0x5A, 0x36)

BOLD = str(ROOT / "fonts/Inter-Bold.ttf")
REG = str(ROOT / "fonts/Inter-Regular.ttf")


def clamp(x, a=0.0, b=1.0):
    return max(a, min(b, x))


def ease_out_cubic(x):
    x = clamp(x)
    return 1 - (1 - x) ** 3


def ease_in_cubic(x):
    x = clamp(x)
    return x ** 3


def ease_in_out(x):
    x = clamp(x)
    return 3 * x * x - 2 * x * x * x


def spring(t, zeta=0.40, omega=22.0):
    """Unit step response of an underdamped spring: 0 at t=0, peaks ~1.25, settles to 1."""
    if t <= 0:
        return 0.0
    wd = omega * math.sqrt(1 - zeta * zeta)
    return 1 - math.exp(-zeta * omega * t) * (math.cos(wd * t) + zeta * omega / wd * math.sin(wd * t))


def rounded_mask(size, radius, ss=4):
    w, h = size
    m = Image.new("L", (w * ss, h * ss), 0)
    ImageDraw.Draw(m).rounded_rectangle((0, 0, w * ss - 1, h * ss - 1), radius * ss, fill=255)
    return m.resize((w, h), Image.LANCZOS)


def paste_center(canvas, im, cx, cy, alpha=1.0):
    if alpha <= 0:
        return
    if alpha < 1:
        im = im.copy()
        a = im.getchannel("A").point(lambda v: int(v * alpha))
        im.putalpha(a)
    x = int(round(cx - im.width / 2))
    y = int(round(cy - im.height / 2))
    canvas.alpha_composite(im, (x, y)) if x >= 0 and y >= 0 and x + im.width <= W and y + im.height <= H else _paste_clipped(canvas, im, x, y)


def _paste_clipped(canvas, im, x, y):
    layer = Image.new("RGBA", canvas.size, (0, 0, 0, 0))
    layer.paste(im, (x, y), im)
    canvas.alpha_composite(layer)


# ---------------------------------------------------------------- hook words
HOOK = ["Video", "your", "agent", "can", "read."]
BEATS = [0.0, 0.5, 1.0, 1.5, 2.0]
HOOK_SIZE = 124
SS = 2  # render glyphs at 2x, scale down per frame
hook_font = ImageFont.truetype(BOLD, HOOK_SIZE * SS)
space = hook_font.getlength(" ") / SS * 1.3


def word_image(text, color):
    l, t, r, b = hook_font.getbbox(text)
    asc, desc = hook_font.getmetrics()
    pad = 20 * SS
    im = Image.new("RGBA", (int(r - l) + 2 * pad, asc + desc + 2 * pad), (0, 0, 0, 0))
    ImageDraw.Draw(im).text((pad - l, pad), text, font=hook_font, fill=color + (255,))
    return im, (r - l) / SS


words = []
for i, w in enumerate(HOOK):
    color = SIGNAL if i == len(HOOK) - 1 else PAPER
    im, width = word_image(w, color)
    words.append((im, width))
line_w = sum(wd for _, wd in words) + space * (len(words) - 1)
assert line_w < W - 160, line_w
x = (W - line_w) / 2
word_cx = []
for _, wd in words:
    word_cx.append(x + wd / 2)
    x += wd + space
HOOK_CY = H / 2


def draw_hook(canvas, t):
    clear = ease_in_cubic((t - 3.5) / 0.35)
    if clear >= 1:
        return
    for i, ((im, _), beat) in enumerate(zip(words, BEATS)):
        lt = t - beat
        if lt < 0:
            continue
        p = spring(lt + 1 / FPS)  # the beat frame already shows the word
        scale = 0.35 + 0.65 * p
        dy = 70 * (1 - p)
        alpha = clamp(0.55 + lt / 0.07)
        # clear: lift and fade, slight stagger left to right
        c = ease_in_cubic((t - 3.5 - i * 0.02) / 0.3)
        dy -= 60 * c
        alpha *= 1 - c
        if alpha <= 0:
            continue
        sw = max(1, int(im.width * scale / SS))
        sh = max(1, int(im.height * scale / SS))
        paste_center(canvas, im.resize((sw, sh), Image.LANCZOS), word_cx[i], HOOK_CY + dy, alpha)


# ---------------------------------------------------------------- screenshot card
CARD_W, CARD_H, CARD_R = 1600, 900, 36
shot1 = Image.open(ROOT / "stills/session-01.png").convert("RGB")
shot2 = Image.open(ROOT / "stills/session-02.png").convert("RGB")
ZMAX = 1.14
# pre-scale both shots to the largest size they are drawn at
base1 = shot1.resize((int(CARD_W * ZMAX), int(CARD_H * ZMAX)), Image.LANCZOS)
base2 = shot2.resize((int(CARD_W * ZMAX), int(CARD_H * ZMAX)), Image.LANCZOS)

SH_PAD = 160
_sh = Image.new("L", (CARD_W + 2 * SH_PAD, CARD_H + 2 * SH_PAD), 0)
ImageDraw.Draw(_sh).rounded_rectangle((SH_PAD, SH_PAD, SH_PAD + CARD_W, SH_PAD + CARD_H), CARD_R, fill=255)
_sh = _sh.filter(ImageFilter.GaussianBlur(48))
shadow_base = Image.new("RGBA", _sh.size, INK + (0,))
shadow_base.putalpha(_sh.point(lambda v: int(v * 0.55)))


def shot_content(base, zoom, size):
    """Crop the centre of `base` for content zoom `zoom` (1..ZMAX) and fit to `size`."""
    # visible window in base coordinates
    vw = CARD_W * ZMAX / zoom
    vh = CARD_H * ZMAX / zoom
    cx, cy = base.width / 2, base.height / 2
    box = (max(0, cx - vw / 2), max(0, cy - vh / 2),
           min(base.width, cx + vw / 2), min(base.height, cy + vh / 2))
    return base.resize(size, Image.BICUBIC, box=box)


def draw_card(canvas, t):
    enter = ease_out_cubic((t - 4.0 + 1 / FPS) / 0.45)  # beat frame already shows the card
    push = (t - 4.0) / 5.0  # 0..1 over 4-9 s, linear: a slow constant push
    scale = (0.84 + 0.08 * push) * (0.94 + 0.06 * enter)
    zoom = 1.0 + (ZMAX - 1.0) * push
    cw, ch = int(round(CARD_W * scale)), int(round(CARD_H * scale))
    r = CARD_R * scale
    cx, cy = W / 2, H / 2 + 90 * (1 - enter)

    mix = ease_in_out((t - 7.0) / 0.5)
    content = shot_content(base1, zoom, (cw, ch)) if mix < 1 else None
    if mix > 0:
        c2 = shot_content(base2, zoom, (cw, ch))
        content = c2 if content is None else Image.blend(content, c2, mix)
    card = content.convert("RGBA")
    card.putalpha(rounded_mask((cw, ch), r, ss=2))

    sh = shadow_base.resize((int(shadow_base.width * scale), int(shadow_base.height * scale)), Image.BILINEAR)
    paste_center(canvas, sh, cx, cy + 34 * scale, enter)
    paste_center(canvas, card, cx, cy, enter)


# ---------------------------------------------------------------- chart
BARS = [("Edit", 0.40, INK), ("Check", 0.68, INK), ("Render", 1.00, SIGNAL)]
BAR_W, BAR_GAP, BAR_MAXH = 230, 110, 520
BASE_Y = 790
label_font = ImageFont.truetype(BOLD, 52)
chart_x0 = (W - (3 * BAR_W + 2 * BAR_GAP)) / 2


def draw_chart(canvas, t):
    lt = t - 9.0 + 1 / FPS  # beat frame already shows the bars
    d = ImageDraw.Draw(canvas)
    for i, (label, frac, color) in enumerate(BARS):
        g = ease_out_cubic(lt / (0.50 + 0.10 * i))
        h = BAR_MAXH * frac * g
        x0 = chart_x0 + i * (BAR_W + BAR_GAP)
        if h >= 1:
            # square foot on the baseline, small rounded data-end on top
            top = BASE_Y - h
            r = min(10, h / 2)
            d.rounded_rectangle((x0, top, x0 + BAR_W, BASE_Y), r, fill=color)
            d.rectangle((x0, max(top + r, BASE_Y - r), x0 + BAR_W, BASE_Y), fill=color)
        a = int(255 * clamp(lt / 0.15 + 0.4))
        lw = label_font.getlength(label)
        lab = Image.new("RGBA", (int(lw) + 20, 80), (0, 0, 0, 0))
        ImageDraw.Draw(lab).text((10, 4), label, font=label_font, fill=INK + (a,))
        canvas.alpha_composite(lab, (int(x0 + BAR_W / 2 - lab.width / 2), BASE_Y + 34))
    # baseline
    d.rectangle((chart_x0 - 40, BASE_Y, chart_x0 + 3 * BAR_W + 2 * BAR_GAP + 40, BASE_Y + 4), fill=INK)


# ---------------------------------------------------------------- end card
lockup = Image.open(ROOT / "brand/lockup-on-dark.png").convert("RGBA")
LOCK_W = 1240
lockup = lockup.resize((LOCK_W, int(lockup.height * LOCK_W / lockup.width)), Image.LANCZOS)
LX, LY = (W - lockup.width) // 2, (H - lockup.height) // 2
WIPE_W = 26
WIPE_PAD = 40


def draw_end(canvas, t):
    lt = t - 10.0
    x_start, x_end = LX - WIPE_PAD, LX + lockup.width + WIPE_PAD
    p = ease_in_out(lt / 0.45) if lt < 0.45 else 1.0
    bar_x = x_start + (x_end - x_start) * p  # left edge of the bar
    reveal = int(clamp(bar_x - LX, 0, lockup.width))
    if reveal > 0:
        canvas.alpha_composite(lockup.crop((0, 0, reveal, lockup.height)), (LX, LY))
    # the bar: travels across, then collapses to nothing by 10.6 s
    collapse = ease_in_cubic((lt - 0.45) / 0.15)
    if collapse < 1:
        bh = (lockup.height + 2 * WIPE_PAD) * (1 - collapse)
        cy = LY + lockup.height / 2
        ImageDraw.Draw(canvas).rectangle(
            (bar_x, cy - bh / 2, bar_x + WIPE_W, cy + bh / 2), fill=SIGNAL)


# ---------------------------------------------------------------- frames
def frame(n):
    t = n / FPS
    if t < 4.0:
        c = Image.new("RGBA", (W, H), INK + (255,))
        draw_hook(c, t)
    elif t < 9.0:
        c = Image.new("RGBA", (W, H), PAPER + (255,))
        draw_card(c, t)
    elif t < 10.0:
        c = Image.new("RGBA", (W, H), PAPER + (255,))
        draw_chart(c, t)
    else:
        c = Image.new("RGBA", (W, H), INK + (255,))
        draw_end(c, t)
    return c.convert("RGB")


def main():
    out = sys.argv[1] if len(sys.argv) > 1 else str(ROOT / "build/video.mp4")
    if len(sys.argv) > 2:  # stills mode: render listed times to PNG
        for ts in sys.argv[2:]:
            frame(round(float(ts) * FPS)).save(f"{out}_{ts}.png")
        return
    ff = subprocess.Popen([
        "ffmpeg", "-y", "-loglevel", "error",
        "-f", "rawvideo", "-pix_fmt", "rgb24", "-s", f"{W}x{H}", "-r", str(FPS), "-i", "-",
        "-vf", "scale=out_color_matrix=bt709:out_range=tv",
        "-c:v", "libx264", "-preset", "slow", "-crf", "16", "-pix_fmt", "yuv420p",
        "-color_primaries", "bt709", "-color_trc", "bt709", "-colorspace", "bt709",
        out,
    ], stdin=subprocess.PIPE)
    for n in range(N):
        ff.stdin.write(frame(n).tobytes())
    ff.stdin.close()
    ff.wait()


if __name__ == "__main__":
    main()
