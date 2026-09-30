"""Montagent logo reveal loop: 6 s, 1080x1080, 30 fps. Writes raw RGB frames to stdout."""
import math
import sys
from PIL import Image, ImageDraw, ImageFont

FPS, N, OUT = 30, 180, 1080
SS = 2                      # supersampling factor
W = OUT * SS
INK, PAPER, SIGNAL = (0x10, 0x14, 0x18), (0xF5, 0xF0, 0xE6), (0xFF, 0x5A, 0x36)
GROUND = PAPER

# ---- layout (ratios measured from brand/lockup.png, tile span S) ----
S = 176 * SS                # mark tile span (outer edge to outer edge)
k = S / 960                 # mark box units -> px (tiles span 960 of the 1024 box)
TILE, GAP, RAD = 460 * k, 40 * k, 96 * k
TEXT_GAP = 0.371 * S
CAP_TOP = 0.166 * S
CAP_H = 0.521 * S

font_probe = ImageFont.truetype("fonts/Inter-Bold.ttf", 1000)
cap_1000 = font_probe.getbbox("H")
FONT_PX = round(CAP_H / (cap_1000[3] - cap_1000[1]) * 1000)
font = ImageFont.truetype("fonts/Inter-Bold.ttf", FONT_PX)
WORD = "Montagent"
word_bbox = font.getbbox(WORD)
word_w = word_bbox[2] - word_bbox[0]
cap_bb = font.getbbox("H")

total_w = S + TEXT_GAP + word_w
mark_x = (W - total_w) / 2
mark_y = (W - S) / 2
text_ink_x = mark_x + S + TEXT_GAP
text_origin = (text_ink_x - word_bbox[0], mark_y + CAP_TOP - cap_bb[1])
baseline_y = text_origin[1] + font.getmetrics()[0]

# tile centres in final position: TL, TR (circle), BL, BR
def centre(col, row):
    return (mark_x + col * (TILE + GAP) + TILE / 2, mark_y + row * (TILE + GAP) + TILE / 2)

# ---- sprites ----
def _sprite(color, pad_frac, draw):
    # drawn 4x larger with integer geometry, then reduced: clean anti-aliased edges
    q = 4
    t = round(TILE * q)
    pad = round(TILE * pad_frac * q)
    size = t + 2 * pad
    size += size % (2 * q) and (2 * q - size % (2 * q))
    pad = (size - t) // 2
    im = Image.new("RGBA", (size, size), color + (0,))
    draw(ImageDraw.Draw(im), [pad, pad, pad + t - 1, pad + t - 1], color + (255,))
    return im.resize((size // q, size // q), Image.LANCZOS)

def rounded_sprite(color):
    return _sprite(color, 0.25, lambda d, box, c: d.rounded_rectangle(box, radius=round(RAD * 4), fill=c))

def circle_sprite(color):
    return _sprite(color, 0.02, lambda d, box, c: d.ellipse(box, fill=c))

SQUARE = rounded_sprite(INK)
CIRCLE = circle_sprite(SIGNAL)

# ---- easing ----
def clamp(x, a=0.0, b=1.0):
    return max(a, min(b, x))

def ease_out_cubic(x):
    return 1 - (1 - x) ** 3

def ease_in_back(x, s=0.8):
    return x * x * ((s + 1) * x - s)

def spin_profile(x, cruise=0.45):
    """Distance fraction for constant speed then linear deceleration to rest.
    Keeps peak angular speed low so each of the three turns reads clearly."""
    total = cruise + (1 - cruise) / 2
    if x <= cruise:
        return x / total
    u = x - cruise
    d = 1 - cruise
    return (cruise + u - u * u / (2 * d)) / total

# ---- timeline (seconds) ----
# arrival: (start, end, from-direction)
FLY = [
    (0.03, 0.73, "left"),    # TL square
    (0.28, 0.98, "top"),     # TR circle
    (0.53, 1.23, "bottom"),  # BL square
]
SPIN = (0.60, 2.00)          # BR square: flies in from the right, 3 full turns
TURNS = 3
CURSOR_ON = 2.4
TYPE_START, TYPE_STEP = 3.0, 1 / 6   # 9 letters: 3.0 .. 4.33
DELETE_START, DELETE_STEP = 4.95, 1 / 30
CLEAR = [(5.05, 5.35), (5.10, 5.40), (5.15, 5.45), (5.20, 5.50)]  # TL, TR, BL, BR
BLINK = 0.2                  # hard on/off half-period

def shape_states(t):
    """List of (sprite, cx, cy, angle_deg, scale) for this instant."""
    out = []
    finals = [centre(0, 0), centre(1, 0), centre(0, 1), centre(1, 1)]
    sprites = [SQUARE, CIRCLE, SQUARE, SQUARE]
    for i in range(4):
        fx, fy = finals[i]
        angle, scale = 0.0, 1.0
        if i < 3:
            a, b, d = FLY[i]
            if t < a:
                continue
            x = clamp((t - a) / (b - a))
            dx = {"left": -1, "right": 1, "top": 0, "bottom": 0}[d]
            dy = {"top": -1, "bottom": 1, "left": 0, "right": 0}[d]
            travel = abs((fx if dx else fy) - (0 if (dx < 0 or dy < 0) else W)) + TILE * 0.8
            # glide in, then a small fixed-size overshoot so it settles without hitting neighbours
            bump = 0.06 * TILE / travel * math.sin(math.pi * clamp((x - 0.6) / 0.4))
            dist = (1 - ease_out_cubic(x)) - bump
            cx, cy = fx + dx * travel * dist, fy + dy * travel * dist
        else:
            a, b = SPIN
            if t < a:
                continue
            p = spin_profile(clamp((t - a) / (b - a)))
            angle = 360 * TURNS * p
            travel = (W - fx) + TILE * 0.8
            cx, cy = fx + travel * (1 - p), fy
        ca, cb = CLEAR[i]
        if t >= ca:
            q = clamp((t - ca) / (cb - ca))
            if q >= 1:
                continue
            scale = 1 - ease_in_back(q)
            if scale <= 0.005:
                continue
        out.append((sprites[i], cx, cy, angle, scale))
    return out

def draw_shapes(canvas, t):
    for sprite, cx, cy, angle, scale in shape_states(t):
        im = sprite
        if scale != 1.0:
            s = max(2, int(round(im.width * scale)))
            im = im.resize((s, s), Image.BICUBIC)
        if angle % 360:
            im = im.rotate(angle, resample=Image.BICUBIC)
        canvas.alpha_composite(im, (int(round(cx - im.width / 2)), int(round(cy - im.height / 2))))

def letters_shown(t):
    if t < TYPE_START:
        return 0
    n = min(len(WORD), int((t - TYPE_START) / TYPE_STEP + 1e-6) + 1)
    if t >= DELETE_START:
        n = max(0, len(WORD) - int((t - DELETE_START) / DELETE_STEP + 1e-6) - 1)
    return n

CUR_W = round(0.055 * FONT_PX)
CUR_GAP = round(0.05 * FONT_PX)
CUR_TOP = text_origin[1] + cap_bb[1] - 0.10 * CAP_H
CUR_BOT = baseline_y + 0.10 * CAP_H
DELETE_END = DELETE_START + len(WORD) * DELETE_STEP

def cursor_visible(t):
    if t < CURSOR_ON or t >= DELETE_END + 1 / 60:
        return False
    typing = TYPE_START - 1e-6 <= t < TYPE_START + len(WORD) * TYPE_STEP
    deleting = t >= DELETE_START
    if typing or deleting:
        return True          # solid while typing or deleting, like a real caret
    ref = CURSOR_ON if t < TYPE_START else TYPE_START + len(WORD) * TYPE_STEP
    return int((t - ref) / BLINK + 1e-6) % 2 == 0

def draw_text(canvas, t):
    n = letters_shown(t)
    d = ImageDraw.Draw(canvas)
    if n:
        d.text(text_origin, WORD[:n], font=font, fill=INK)
    if cursor_visible(t):
        adv = font.getlength(WORD[:n]) if n else 0
        x = text_ink_x if n == 0 else text_origin[0] + adv + CUR_GAP
        d.rectangle([x, CUR_TOP, x + CUR_W, CUR_BOT], fill=INK)

def moving(t):
    return (FLY[0][0] <= t <= SPIN[1] + 0.05) or (CLEAR[0][0] <= t <= CLEAR[-1][1])

def render_instant(t):
    c = Image.new("RGBA", (W, W), GROUND + (255,))
    draw_shapes(c, t)
    return c

def frame(i):
    t = i / FPS
    if moving(t):
        # motion blur: average sub-frames across a 180-degree shutter
        subs = 6
        acc = None
        for j in range(subs):
            ts = t + (j / subs - 0.5) * 0.5 / FPS
            im = render_instant(max(0.0, ts)).convert("RGB")
            acc = im if acc is None else Image.blend(acc, im, 1 / (j + 1))
        c = acc.convert("RGBA")
    else:
        c = render_instant(t)
    draw_text(c, t)
    return c.convert("RGB").resize((OUT, OUT), Image.LANCZOS)

if __name__ == "__main__":
    only = [int(a) for a in sys.argv[1:]]
    if only:
        for i in only:
            frame(i).save(f"{__import__('os').environ['TMPDIR']}/f{i:03d}.png")
    else:
        for i in range(N):
            sys.stdout.buffer.write(frame(i).tobytes())
