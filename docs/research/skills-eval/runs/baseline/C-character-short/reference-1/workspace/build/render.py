"""Render Brief C: Hoot the owl introduces Montagent (8 s, 1920x1080, 30 fps).

Writes raw RGB frames to stdout (or PNGs with --png DIR --frames a,b,c) for ffmpeg.
"""
import json, math, os, sys
from PIL import Image, ImageDraw, ImageFilter
import numpy as np

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CH = os.path.join(ROOT, "character")
W, H, FPS, DUR = 1920, 1080, 30, 8.0
NFRAMES = int(DUR * FPS)
VOICE_AT = 1.0  # seconds

rig = json.load(open(os.path.join(CH, "rig.json")))
line = json.load(open(os.path.join(CH, "voice", "line-1.json")))

# ---------------------------------------------------------------- layout
S0 = 0.62                    # owl scale: drawing px -> frame px
FEET = (512, 1322)           # torso pivot in the drawing
HOME_X, FLOOR_Y = 790, 1002  # where the feet stand
CARD_C = (1445, 520)         # card centre
CARD_IMG = (600, 338)
LOCKUP_W = 780
LOCKUP_CY = 114

INK = (16, 20, 24)
PAPER = (245, 240, 230)

# ---------------------------------------------------------------- helpers
def clamp(x, a=0.0, b=1.0):
    return max(a, min(b, x))

def smooth(x):
    x = clamp(x)
    return x * x * (3 - 2 * x)

def ease_out_back(x, s=1.7):
    x = clamp(x) - 1
    return 1 + (s + 1) * x ** 3 + s * x ** 2

def ramp(t, t0, t1, fn=smooth):
    return fn((t - t0) / (t1 - t0)) if t1 > t0 else float(t >= t0)

def spring(tau, damp=9.0, freq=12.0):
    """0 at tau<=0 -> 1 with a small overshoot (~10%)."""
    if tau <= 0:
        return 0.0
    return 1 - math.exp(-damp * tau) * math.cos(freq * tau)

def mat_t(x, y):
    return np.array([[1, 0, x], [0, 1, y], [0, 0, 1]], float)

def mat_s(sx, sy):
    return np.array([[sx, 0, 0], [0, sy, 0], [0, 0, 1]], float)

def mat_r(deg):
    """Rotate counter-clockwise *on screen* (y points down)."""
    a = math.radians(deg)
    c, s = math.cos(a), math.sin(a)
    return np.array([[c, s, 0], [-s, c, 0], [0, 0, 1]], float)

def about(px, py, m):
    return mat_t(px, py) @ m @ mat_t(-px, -py)

# ---------------------------------------------------------------- assets
REPAIRED = os.path.join(ROOT, "build", "parts")   # written by repair.py
REPAIRED_SIZES = json.load(open(os.path.join(REPAIRED, "sizes.json")))


def load_part(name):
    p = dict(rig["parts"][name])
    path = os.path.join(REPAIRED, name + ".png")
    if os.path.exists(path):
        p.update(REPAIRED_SIZES.get(name, {}))
        im = Image.open(path).convert("RGBA")
    else:
        im = Image.open(os.path.join(CH, p["file"])).convert("RGBA")
    assert im.size == (p["width"], p["height"]), name
    # Pre-scale once with a good filter so per-frame transforms stay near 1:1.
    # Pad a few px so bicubic sampling never clips the antialiased edge.
    sw, sh = round(im.width * S0), round(im.height * S0)
    im = im.resize((sw, sh), Image.LANCZOS)
    pad = 4
    canvas = Image.new("RGBA", (sw + 2 * pad, sh + 2 * pad), (0, 0, 0, 0))
    canvas.paste(im, (pad, pad))
    # canvas px -> drawing px
    ox = p["pivot"][0] - p["width"] / 2
    oy = p["pivot"][1] - p["height"] / 2
    to_drawing = mat_t(ox, oy) @ mat_s(p["width"] / sw, p["height"] / sh) @ mat_t(-pad, -pad)
    return {"img": canvas.convert("RGBa"), "to_drawing": to_drawing, "pivot": p["pivot"],
            "parent": p["parent"]}

PARTS = {n: load_part(n) for n in rig["parts"]}

study = Image.open(os.path.join(CH, "study.png")).convert("RGBA")
still1 = Image.open(os.path.join(ROOT, "stills", "session-01.png")).convert("RGBA")
still2 = Image.open(os.path.join(ROOT, "stills", "session-02.png")).convert("RGBA")
lockup_src = Image.open(os.path.join(ROOT, "brand", "lockup.png")).convert("RGBA")
lockup_src = lockup_src.crop(lockup_src.getchannel("A").getbbox())
LOCKUP = lockup_src.resize((LOCKUP_W, round(lockup_src.height * LOCKUP_W / lockup_src.width)),
                           Image.LANCZOS)


def rounded_mask(size, r, scale=4):
    w, h = size
    m = Image.new("L", (w * scale, h * scale), 0)
    ImageDraw.Draw(m).rounded_rectangle([0, 0, w * scale - 1, h * scale - 1], r * scale, fill=255)
    return m.resize(size, Image.LANCZOS)


def make_card(still):
    iw, ih = CARD_IMG
    m, border = 16, 6
    cw, ch = iw + 2 * m, ih + 2 * m
    pad = 60
    card = Image.new("RGBA", (cw + 2 * pad, ch + 2 * pad), (0, 0, 0, 0))
    # soft shadow
    sh = Image.new("RGBA", card.size, (0, 0, 0, 0))
    sm = Image.new("L", card.size, 0)
    sm.paste(rounded_mask((cw, ch), 30), (pad, pad + 16))
    sm = sm.filter(ImageFilter.GaussianBlur(16)).point(lambda v: v * 0.30)
    sh.paste((*INK, 255), (0, 0), sm)
    card.alpha_composite(sh)
    # ink outline + paper face, matching the owl's flat outlined style
    outer = Image.new("RGBA", (cw, ch), (*INK, 255))
    outer.putalpha(rounded_mask((cw, ch), 30))
    card.alpha_composite(outer, (pad, pad))
    face = Image.new("RGBA", (cw - 2 * border, ch - 2 * border), (*PAPER, 255))
    face.putalpha(rounded_mask(face.size, 30 - border))
    card.alpha_composite(face, (pad + border, pad + border))
    pic = still.resize((iw, ih), Image.LANCZOS)
    pic.putalpha(rounded_mask((iw, ih), 12))
    card.alpha_composite(pic, (pad + m, pad + m))
    return card.convert("RGBa")

CARD1, CARD2 = make_card(still1), make_card(still2)

# ---------------------------------------------------------------- timing
WORDS = {w["word"].lower(): w for w in line["words"]}
def word_t(word, which="start", idx=0):
    ws = [w for w in line["words"] if w["word"].lower() == word]
    return VOICE_AT + ws[idx][which] / 1000.0

T_WAVE0 = VOICE_AT - 0.12             # arm goes up just before "Hi"
T_WAVE1 = word_t("hoot", "end") + 0.05
T_CARD = word_t("as")                 # card pops in on "...as a file"
T_POINT = T_CARD + 0.05
T_MOVIE = word_t("movie")             # picture swaps on "movie"
T_LOCK = 6.72                         # lockup starts dropping, lands ~7.0
T_CHEER = 6.86
LINE_END = VOICE_AT + max(v[0] for v in line["visemes"]) / 1000.0

BLINKS = [2.52, 4.62, 7.50]           # each lasts 3 frames
BLINK_FRAMES = 3

VIS = sorted(line["visemes"], key=lambda v: v[0])
# A silence viseme shorter than ~a frame and a half inside speech is a flicker, not a pause:
# hold the previous mouth through it. Real pauses (and the end) stay at rest.
VIS = [v for k, v in enumerate(VIS)
       if not (v[1] == 0 and 0 < k < len(VIS) - 1 and VIS[k + 1][0] - v[0] < 50)]
def mouth_at(t):
    ms = (t - VOICE_AT) * 1000.0
    cur = None
    for at, vid in VIS:
        if at <= ms + 1e-6:
            cur = vid
        else:
            break
    if cur is None:
        return None
    m = rig["visemes"][str(cur)]
    return None if m is None else "mouth_" + m

# ---------------------------------------------------------------- pose
REST_ARM = -56  # "lift" for arms hanging at the sides (rest pose is arms out at 25 deg down)

def hop_state(t):
    """Root x, height above floor, squash, lean for the entrance."""
    x0, x1, x2 = -330, 300, HOME_X
    hops = [(0.00, 0.40, x0, x1, 170), (0.50, 0.82, x1, x2, 110)]
    x, h, lean = HOME_X, 0.0, 0.0
    sq = 0.0
    if t < hops[0][0]:
        return x0, 170, 0, 0
    for (a, b, xa, xb, hh) in hops:
        if a <= t <= b:
            u = (t - a) / (b - a)
            x = xa + (xb - xa) * u
            h = hh * 4 * u * (1 - u)
            sq = -0.07 * math.sin(math.pi * u)  # stretch in the air
            lean = -6 * math.sin(math.pi * u)
            return x, h, sq, lean
    if t < hops[1][0]:
        u = (t - hops[0][1]) / (hops[1][0] - hops[0][1])
        return x1, 0, 0.14 * math.sin(math.pi * u), 0
    # landing squash, settling
    u = t - hops[1][1]
    sq = 0.16 * math.exp(-7 * u) * math.cos(14 * u) if u < 1.2 else 0.0
    return HOME_X, 0.0, sq, 0.0


def arm_rest_bob(t):
    return 2.0 * math.sin(2 * math.pi * 0.45 * t)


def pose(t):
    P = {}
    x, h, sq, lean = hop_state(t)
    in_air = h > 0.5
    # small cheer bounce
    if t > T_CHEER:
        u = t - T_CHEER
        if u < 0.36:
            h += 18 * 4 * (u / 0.36) * (1 - u / 0.36)
            sq += -0.05 * math.sin(math.pi * u / 0.36)
        else:
            v = u - 0.36
            sq += 0.10 * math.exp(-8 * v) * math.cos(15 * v)
    breathe = 0.008 * math.sin(2 * math.pi * 0.5 * t)
    lean += 1.3 * math.sin(2 * math.pi * 0.35 * (t - 1.0)) * ramp(t, 1.0, 1.6)
    P["root"] = dict(x=x, y=FLOOR_Y - h, sx=1 + 0.55 * sq, sy=1 - sq + breathe, lean=lean, h=h)

    # head: follows speech a little, leans toward the card while gesturing
    talk = 1.0 if VOICE_AT <= t <= LINE_END else 0.0
    head = 2.2 * math.sin(2 * math.pi * 0.9 * t) * talk + 1.2 * math.sin(2 * math.pi * 0.23 * t)
    head += -4.0 * ramp(t, T_POINT, T_POINT + 0.4) * (1 - ramp(t, 6.6, 6.95))
    head += 3.0 * ramp(t, T_WAVE0, T_WAVE0 + 0.25) * (1 - ramp(t, T_WAVE1, T_WAVE1 + 0.3))
    if in_air or t < 1.0:
        head += 25 * sq  # drag on the hop
    P["head"] = head

    # ---- left arm (viewer's left): wave
    lift_l = REST_ARM + arm_rest_bob(t)
    fore_l = 0.0
    if in_air:
        lift_l += 14 * min(1, h / 80)  # arms float up a touch in the air
    up = ramp(t, T_WAVE0, T_WAVE0 + 0.22, ease_out_back)
    down = ramp(t, T_WAVE1, T_WAVE1 + 0.32)
    wave_amt = up * (1 - down)
    swing_t = t - (T_WAVE0 + 0.12)
    swing = 22 * math.sin(2 * math.pi * swing_t / 0.40) if swing_t > 0 else 0.0
    swing *= ramp(t, T_WAVE0 + 0.1, T_WAVE0 + 0.25) * (1 - ramp(t, T_WAVE1 - 0.08, T_WAVE1 + 0.1))
    lift_l = lift_l * (1 - wave_amt) + 44 * wave_amt
    fore_l = (40 + swing) * wave_amt
    # a small open-hand beat on "...writes the video"
    b_amt = ramp(t, word_t("writes") - 0.15, word_t("writes") + 0.12, ease_out_back) \
        * (1 - ramp(t, word_t("video", "end") - 0.05, word_t("as") + 0.05))
    lift_l += 30 * b_amt
    fore_l += (22 + 6 * math.sin(2 * math.pi * 1.6 * (t - word_t("writes")))) * b_amt

    # ---- right arm (viewer's right, the card's side): gesture to the card
    lift_r = REST_ARM - arm_rest_bob(t + 0.3)
    fore_r = 0.0
    if in_air:
        lift_r += 14 * min(1, h / 80)
    g_up = ramp(t, T_POINT, T_POINT + 0.30, ease_out_back)
    g_down = ramp(t, 6.62, 6.86)
    g = g_up * (1 - g_down)
    beat = 5 * math.sin(2 * math.pi * 0.8 * (t - T_POINT)) * ramp(t, T_POINT + 0.4, T_POINT + 0.8)
    movie = 9 * math.exp(-6 * (t - T_MOVIE)) * math.sin(math.pi * clamp((t - T_MOVIE) / 0.25)) \
        if t > T_MOVIE else 0.0
    lift_r = lift_r * (1 - g) + (36 + beat + movie) * g
    fore_r = (8 + 0.5 * beat) * g

    # ---- cheer: both arms up
    c = ramp(t, T_CHEER, T_CHEER + 0.24, ease_out_back)
    pump = 7 * math.sin(2 * math.pi * 1.8 * (t - T_CHEER - 0.25)) * ramp(t, T_CHEER + 0.25, T_CHEER + 0.45)
    lift_l = lift_l * (1 - c) + (46 + pump) * c
    lift_r = lift_r * (1 - c) + (46 + pump) * c
    fore_l = fore_l * (1 - c) + (42 + 0.8 * pump) * c
    fore_r = fore_r * (1 - c) + (42 + 0.8 * pump) * c

    # screen rotation: left arm raising = clockwise (negative), right = counter-clockwise
    P["upper_arm_left"] = -lift_l
    P["forearm_left"] = -fore_l
    P["upper_arm_right"] = lift_r
    P["forearm_right"] = fore_r

    blink = any(b <= t < b + BLINK_FRAMES / FPS for b in BLINKS)
    P["blink"] = blink
    P["mouth"] = mouth_at(t)
    return P


def world_mats(P):
    r = P["root"]
    # drawing -> frame: put the feet at (x, y), scale, squash about the feet
    root = mat_t(r["x"], r["y"]) @ mat_r(r["lean"]) @ mat_s(S0 * r["sx"], S0 * r["sy"]) \
        @ mat_t(-FEET[0], -FEET[1])
    M = {"torso": root}
    local = {"head": P["head"], "upper_arm_left": P["upper_arm_left"],
             "upper_arm_right": P["upper_arm_right"], "forearm_left": P["forearm_left"],
             "forearm_right": P["forearm_right"]}
    for n in ["head", "upper_arm_left", "upper_arm_right", "forearm_left", "forearm_right"]:
        part = PARTS[n]
        M[n] = M[part["parent"]] @ about(*part["pivot"], mat_r(local[n]))
    return M


def draw_part(frame, name, M):
    part = PARTS[name]
    A = M @ part["to_drawing"]   # canvas px -> frame px
    w, h = part["img"].size
    corners = np.array([[0, 0, 1], [w, 0, 1], [0, h, 1], [w, h, 1]], float).T
    fc = A @ corners
    x0, y0 = int(math.floor(fc[0].min())) - 2, int(math.floor(fc[1].min())) - 2
    x1, y1 = int(math.ceil(fc[0].max())) + 2, int(math.ceil(fc[1].max())) + 2
    x0c, y0c, x1c, y1c = max(x0, 0), max(y0, 0), min(x1, W), min(y1, H)
    if x1c <= x0c or y1c <= y0c:
        return
    Ainv = np.linalg.inv(mat_t(-x0c, -y0c) @ A)
    coeffs = (Ainv[0, 0], Ainv[0, 1], Ainv[0, 2], Ainv[1, 0], Ainv[1, 1], Ainv[1, 2])
    out = part["img"].transform((x1c - x0c, y1c - y0c), Image.AFFINE, coeffs, Image.BICUBIC)
    frame.alpha_composite(out.convert("RGBA"), (x0c, y0c))


def draw_owl(frame, P):
    M = world_mats(P)
    for name in rig["draw_order"]:
        if name.startswith("mouth_"):
            if P["mouth"] == name:
                draw_part(frame, name, M["head"])
        elif name == "eyes_closed":
            if P["blink"]:
                draw_part(frame, name, M["head"])
        else:
            draw_part(frame, name, M[name])


def draw_shadow(frame, P):
    r = P["root"]
    k = 1.0 / (1 + r["h"] / 120)
    w = 330 * k * r["sx"]
    hgt = 46 * k
    a = int(70 * k)
    lay = Image.new("L", (int(w) + 80, int(hgt) + 80), 0)
    ImageDraw.Draw(lay).ellipse([40, 40, 40 + w, 40 + hgt], fill=a)
    lay = lay.filter(ImageFilter.GaussianBlur(10))
    col = Image.new("RGBA", lay.size, (90, 50, 20, 255))
    col.putalpha(lay)
    frame.alpha_composite(col, (int(r["x"] - w / 2 - 40), int(FLOOR_Y - hgt / 2 - 40 - 6)))


def draw_scaled(frame, img_rgba_pm, cx, cy, scale, rot=0.0, alpha=1.0):
    if scale <= 0.01 or alpha <= 0.0:
        return
    w, h = img_rgba_pm.size
    A = mat_t(cx, cy) @ mat_r(rot) @ mat_s(scale, scale) @ mat_t(-w / 2, -h / 2)
    corners = np.array([[0, 0, 1], [w, 0, 1], [0, h, 1], [w, h, 1]], float).T
    fc = A @ corners
    x0, y0 = max(int(fc[0].min()) - 2, 0), max(int(fc[1].min()) - 2, 0)
    x1, y1 = min(int(fc[0].max()) + 3, W), min(int(fc[1].max()) + 3, H)
    if x1 <= x0 or y1 <= y0:
        return
    Ainv = np.linalg.inv(mat_t(-x0, -y0) @ A)
    coeffs = tuple(Ainv[:2].ravel())
    out = img_rgba_pm.transform((x1 - x0, y1 - y0), Image.AFFINE, coeffs, Image.BICUBIC).convert("RGBA")
    if alpha < 1:
        out.putalpha(out.getchannel("A").point(lambda v: int(v * alpha)))
    frame.alpha_composite(out, (x0, y0))


LOCKUP_PM = LOCKUP.convert("RGBa")


def render(i):
    t = i / FPS
    frame = study.copy()
    P = pose(t)

    # card: pops in with a small overshoot, then swaps its picture on "movie"
    if t >= T_CARD:
        tau = t - T_CARD
        s = spring(tau, 9.0, 12.0)
        rot = -7 * math.exp(-7 * tau) * math.cos(10 * tau)
        if t >= T_MOVIE:
            v = t - T_MOVIE
            s *= 1 + 0.05 * math.exp(-9 * v) * math.sin(math.pi * clamp(v / 0.12) + 0.0) \
                if v < 0.12 else 1 + 0.05 * math.exp(-9 * v) * math.cos(14 * (v - 0.12))
        img = CARD2 if t >= T_MOVIE else CARD1
        draw_scaled(frame, img, CARD_C[0], CARD_C[1], s, rot)

    # lockup lands above the owl
    if t >= T_LOCK:
        tau = t - T_LOCK
        drop = 1 - math.exp(-10 * tau) * math.cos(11 * tau)
        cy = LOCKUP_CY - 300 * (1 - drop)
        a = clamp(tau / 0.12)
        draw_scaled(frame, LOCKUP_PM, HOME_X, cy, 1.0, 0, a)

    draw_shadow(frame, P)
    draw_owl(frame, P)
    return frame.convert("RGB")


if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] == "--png":
        out = sys.argv[2]
        os.makedirs(out, exist_ok=True)
        frames = [int(x) for x in sys.argv[3].split(",")]
        for i in frames:
            render(i).save(os.path.join(out, f"f{i:03d}.png"))
    else:
        o = sys.stdout.buffer
        for i in range(NFRAMES):
            o.write(render(i).tobytes())
