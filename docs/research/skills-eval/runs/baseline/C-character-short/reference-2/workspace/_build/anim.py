"""Brief C: Hoot introduces Montagent. Renders raw frames to ffmpeg."""
import json
import math
import os
import subprocess
import sys

from PIL import Image, ImageDraw, ImageFilter

from rig import Rig, T, S, R, mul, paste_affine

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
W, H, FPS, DUR = 1920, 1080, 30, 8.0
NF = int(DUR * FPS)
INK, PAPER = (16, 20, 24), (245, 240, 230)

VOICE_START = 1.0
with open(os.path.join(ROOT, "character/voice/line-1.json")) as f:
    LINE = json.load(f)
WORD = {w["word"]: VOICE_START + w["start"] / 1000 for w in LINE["words"]}

OWL_X, FLOOR_Y, OWL_SCALE = 600, 985, 0.6
CARD_C = (1330, 470)
CARD_POP = VOICE_START + 2.700          # "as" of "...as a file"
SWAP_FRAME = math.ceil(WORD["movie"] * FPS - 1e-6)  # first frame of "movie"
LOCKUP_T = 6.72
LOCKUP_C = (OWL_X, 150)
BLINKS = [2.45, 4.40, 7.55]
BLINK_FRAMES = 3


# ---- easing / interpolation ------------------------------------------------
def clamp(x, a=0.0, b=1.0):
    return max(a, min(b, x))


def ease_io(u):
    u = clamp(u)
    return 4 * u ** 3 if u < 0.5 else 1 - (-2 * u + 2) ** 3 / 2


def ease_out_back(u, k=2.2):
    u = clamp(u)
    return 1 + (k + 1) * (u - 1) ** 3 + k * (u - 1) ** 2


def keys(t, ks):
    """Piecewise ease-in-out between (time, value) keys; holds at the ends."""
    if t <= ks[0][0]:
        return ks[0][1]
    for (t0, v0), (t1, v1) in zip(ks, ks[1:]):
        if t <= t1:
            return v0 + (v1 - v0) * ease_io((t - t0) / (t1 - t0))
    return ks[-1][1]


def pulse(t, t0, dur):
    """0 -> 1 -> 0 smooth bump starting at t0."""
    u = (t - t0) / dur
    return math.sin(math.pi * u) ** 2 if 0 <= u <= 1 else 0.0


# ---- the owl's performance ---------------------------------------------------
JUMPS = [(0.02, 0.46, -330, 250, 150), (0.54, 0.92, 250, OWL_X, 115)]
LAND = 0.92
SIDES = (-55, 8)
RELAX = (-42, 14)
WAVE_UP, WAVE_T0, WAVE_T1, WAVE_PERIOD = 55, 1.15, 2.05, 0.36
POINT = (35, 4)
CHEER = (55, 30)


def body(t):
    x, y, lean, sy, air = OWL_X, FLOOR_Y, 0.0, 1.0, 0.0
    for t0, t1, x0, x1, h in JUMPS:
        if t0 <= t <= t1:
            u = (t - t0) / (t1 - t0)
            x = x0 + (x1 - x0) * u
            y = FLOOR_Y - h * 4 * u * (1 - u)
            lean = 7 * math.sin(math.pi * u)
            sy = 1 + 0.07 * abs(2 * u - 1)
            air = math.sin(math.pi * u)
            break
    else:
        if t < JUMPS[0][0]:
            x = JUMPS[0][2]
        elif t < JUMPS[1][0]:
            x = JUMPS[0][3]
            sy = 1 - 0.10 * math.sin(math.pi * (t - JUMPS[0][1]) / (JUMPS[1][0] - JUMPS[0][1]))
        elif t > LAND:
            tau = t - LAND
            sy = 1 - 0.14 * math.sin(tau * math.pi / 0.11) * math.exp(-tau * 6)
            lean = -3 * math.sin(tau * math.pi / 0.2) * math.exp(-tau * 8)
    if t > LAND + 0.3:
        sy += 0.010 * math.sin(2 * math.pi * (t - 1.2) / 1.7)      # breathing
    # lean into the card while presenting it, a small cheer hop at the end
    lean += keys(t, [(3.6, 0), (3.95, 3.0), (6.6, 3.0), (6.95, 0)])
    hop = pulse(t, 7.0, 0.34)
    y -= 26 * math.sin(math.pi * clamp((t - 7.0) / 0.34)) if 7.0 <= t <= 7.34 else 0
    if 6.9 <= t < 7.0:
        sy -= 0.06 * pulse(t, 6.9, 0.1)
    if 7.30 <= t <= 7.48:
        sy -= 0.07 * pulse(t, 7.30, 0.18)
    sy += 0.03 * hop
    return x, y, lean, sy, air


def arms(t, air):
    # left (viewer's) arm: sides -> wave -> relaxed -> cheer
    if t < 0.95:
        ll, bl = SIDES[0] + 22 * air, SIDES[1] + 10 * air
    elif t < WAVE_T0:
        u = (t - 0.95) / (WAVE_T0 - 0.95)
        ll = SIDES[0] + (WAVE_UP - SIDES[0]) * ease_out_back(u, 1.2)
        bl = SIDES[1] + (15 - SIDES[1]) * ease_io(u)
    elif t < WAVE_T1:
        ph = (t - WAVE_T0) / WAVE_PERIOD
        ll = WAVE_UP + 2 * math.sin(2 * math.pi * ph * 0.5)
        bl = 15 + 27 * math.sin(2 * math.pi * ph)
    else:
        ll = keys(t, [(WAVE_T1, WAVE_UP), (2.4, RELAX[0]), (2.8, RELAX[0]), (2.98, -22),
                      (3.35, -22), (3.65, RELAX[0]), (6.72, RELAX[0]), (7.0, CHEER[0])])
        bl = keys(t, [(WAVE_T1, 15), (2.4, RELAX[1]), (2.8, RELAX[1]), (2.98, 26),
                      (3.35, 26), (3.65, RELAX[1]), (6.72, RELAX[1]), (7.0, CHEER[1])])
    # right (viewer's) arm: sides -> relaxed -> talk beat -> point at card -> cheer
    if t < 0.95:
        lr, br = SIDES[0] + 22 * air, SIDES[1] + 10 * air
    else:
        lr = keys(t, [(0.95, SIDES[0]), (1.4, RELAX[0]), (2.8, RELAX[0]), (2.98, -22),
                      (3.35, -22), (3.62, -10), (3.9, POINT[0]), (6.72, POINT[0]), (7.0, CHEER[0])])
        br = keys(t, [(0.95, SIDES[1]), (1.4, RELAX[1]), (2.8, RELAX[1]), (2.98, 26),
                      (3.35, 26), (3.62, 12), (3.9, POINT[1]), (6.72, POINT[1]), (7.0, CHEER[1])])
        lr += 7 * pulse(t, WORD["Montagent"] - 0.05, 0.35) + 8 * pulse(t, WORD["movie"] - 0.05, 0.35)
    if t >= 7.0:   # little fist-pump of the cheer
        pump = 7 * math.sin(2 * math.pi * (t - 7.0) / 0.5) * math.exp(-(t - 7.0) * 1.2)
        ll += pump
        lr += pump
    return ll, bl, lr, br


def head(t):
    h = 0.0
    if t > LAND:
        h += 2.0 * math.sin(2 * math.pi * (t - 1.0) / 2.3)
    h += keys(t, [(0.95, 0), (1.15, -4), (2.05, -4), (2.35, 0), (3.6, 0), (3.95, 5),
                  (6.65, 5), (7.0, 0)])
    for w in ("Hoot", "file", "Montagent", "movie"):
        h += 2.5 * pulse(t, WORD[w] - 0.03, 0.3)
    return h


def mouth(t, rig):
    ms = (t + 0.5 / FPS - VOICE_START) * 1000
    vid = 0
    for at, v in LINE["visemes"]:
        if at <= ms:
            vid = v
        else:
            break
    return rig.visemes[str(vid)]


def blinking(frame):
    return any(0 <= frame - round(b * FPS) < BLINK_FRAMES for b in BLINKS)


# ---- props ---------------------------------------------------------------------
def make_card(path):
    pic_w, pic_h, border, radius, line = 720, 405, 16, 30, 6
    pic = Image.open(os.path.join(ROOT, path)).convert("RGB").resize((pic_w, pic_h), Image.LANCZOS)
    cw, ch = pic_w + 2 * border, pic_h + 2 * border
    pad = 60
    img = Image.new("RGBA", (cw + 2 * pad, ch + 2 * pad), (0, 0, 0, 0))
    sh = Image.new("L", img.size, 0)
    ImageDraw.Draw(sh).rounded_rectangle((pad + 8, pad + 18, pad + cw + 8, pad + ch + 18), radius, fill=90)
    sh = sh.filter(ImageFilter.GaussianBlur(14))
    shadow = Image.new("RGBA", img.size, (70, 40, 20, 0))
    shadow.putalpha(sh)
    img.alpha_composite(shadow)
    d = ImageDraw.Draw(img)
    d.rounded_rectangle((pad, pad, pad + cw, pad + ch), radius, fill=PAPER + (255,),
                        outline=INK + (255,), width=line)
    mask = Image.new("L", (pic_w, pic_h), 0)
    ImageDraw.Draw(mask).rounded_rectangle((0, 0, pic_w - 1, pic_h - 1), radius - border + 4, fill=255)
    img.paste(pic, (pad + border, pad + border), mask)
    d.rounded_rectangle((pad + border - 2, pad + border - 2, pad + border + pic_w + 1,
                         pad + border + pic_h + 1), radius - border + 5, outline=INK + (255,), width=3)
    return img.convert("RGBa")


def make_shadow():
    img = Image.new("L", (420, 90), 0)
    ImageDraw.Draw(img).ellipse((30, 20, 390, 70), fill=255)
    img = img.filter(ImageFilter.GaussianBlur(8))
    rgba = Image.new("RGBA", img.size, (90, 50, 20, 0))
    rgba.putalpha(img.point(lambda v: v * 70 // 255))
    return rgba.convert("RGBa")


def centred(img, cx, cy, sx, sy=None, rot=0.0):
    sy = sx if sy is None else sy
    return mul(T(cx, cy), mul(R(rot), mul(S(sx, sy), T(-img.width / 2, -img.height / 2))))


def main():
    rig = Rig(OWL_SCALE)
    study = Image.open(os.path.join(ROOT, "character/study.png")).convert("RGBA")
    cards = [make_card("stills/session-01.png"), make_card("stills/session-02.png")]
    shadow = make_shadow()
    lock = Image.open(os.path.join(ROOT, "brand/lockup.png")).convert("RGBA")
    lw = 660
    lock = lock.resize((lw, round(lock.height * lw / lock.width)), Image.LANCZOS).convert("RGBa")

    only = [int(a) for a in sys.argv[2:]] if len(sys.argv) > 2 else None
    out = sys.argv[1]
    ff = None
    if only is None:
        ff = subprocess.Popen(
            ["ffmpeg", "-y", "-loglevel", "error", "-f", "rawvideo", "-pix_fmt", "rgb24",
             "-s", f"{W}x{H}", "-r", str(FPS), "-i", "-", "-c:v", "libx264", "-preset", "slow",
             "-crf", "16", "-pix_fmt", "yuv420p", out], stdin=subprocess.PIPE)
    log = []
    for f in (only if only is not None else range(NF)):
        t = f / FPS
        fr = study.copy()
        x, y, lean, sy, air = body(t)
        # floor shadow
        k = 1 - 0.5 * clamp((FLOOR_Y - y) / 150)
        paste_affine(fr, shadow, centred(shadow, x, FLOOR_Y + 2, 0.78 * k, 0.78 * k))
        # the card (behind nothing: it sits beside the owl)
        if t >= CARD_POP:
            u = (t - CARD_POP) / 0.42
            s = ease_out_back(u, 2.4) if u < 1 else 1.0
            rot = -6 * (1 - ease_io(u))
            card = cards[1 if f >= SWAP_FRAME else 0]
            if f >= SWAP_FRAME:
                s *= 1 + 0.05 * pulse(t, SWAP_FRAME / FPS, 0.3)
            if s > 0.01:
                paste_affine(fr, card, centred(card, CARD_C[0], CARD_C[1], s, s, rot))
        ll, bl, lr, br = arms(t, air)
        m = mouth(t, rig)
        b = blinking(f)
        rig.draw(fr, dict(x=x, y=y, lean=lean, sx=1 / math.sqrt(sy) if sy > 0 else 1, sy=sy,
                          head=head(t), lift_l=ll, bend_l=bl, lift_r=lr, bend_r=br,
                          mouth=m, blink=b))
        # brand lockup drops in above the owl
        if t >= LOCKUP_T:
            u = (t - LOCKUP_T) / 0.3
            ly = -120 + (LOCKUP_C[1] + 120) * min(1.0, u * u) if u < 1 else LOCKUP_C[1]
            lsy = 1.0
            if u >= 1:
                lsy = 1 - 0.12 * math.sin((t - LOCKUP_T - 0.3) * math.pi / 0.1) * math.exp(-(t - LOCKUP_T - 0.3) * 9)
            elif u > 0.5:
                lsy = 1.06
            lx_s = 1 / math.sqrt(lsy)
            bottom = ly + lock.height / 2
            m_l = mul(T(LOCKUP_C[0], bottom), mul(S(lx_s, lsy), T(-lock.width / 2, -lock.height)))
            paste_affine(fr, lock, m_l)
        log.append((f, round(t, 3), m, b))
        rgb = fr.convert("RGB")
        if ff:
            ff.stdin.write(rgb.tobytes())
        else:
            rgb.save(f"{out}_{f:03d}.png")
    if ff:
        ff.stdin.close()
        ff.wait()
    with open(os.path.join(os.path.dirname(out) or ".", "frames_log.json"), "w") as fh:
        json.dump(log, fh)


if __name__ == "__main__":
    main()
