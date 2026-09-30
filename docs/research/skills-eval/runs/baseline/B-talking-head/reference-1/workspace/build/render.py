"""Brief B: 10 s vertical talking-head social cut for Montagent.

Renders 300 frames at 1080x1920 / 30 fps with numpy + Pillow, pipes them into
ffmpeg, mixes voice and ducked music in numpy, and muxes deliverable.mp4.
"""
import json
import math
import os
import subprocess
import sys
import wave

import numpy as np
from PIL import Image, ImageDraw, ImageFilter, ImageFont

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
P = lambda *a: os.path.join(ROOT, *a)
TMP = os.environ.get("TMPDIR", "/tmp")

W, H, FPS, DUR = 1080, 1920, 30, 10.0
NFRAMES = int(DUR * FPS)

INK = (16, 20, 24)
PAPER = (245, 240, 230)
SIGNAL = (255, 90, 54)

BOLD = P("fonts", "Inter-Bold.ttf")
REG = P("fonts", "Inter-Regular.ttf")

WORDS = json.load(open(P("presenter", "take-1.words.json")))
for w in WORDS:
    w["s"], w["e"] = w["start"] / 1000.0, w["end"] / 1000.0


# ---------------------------------------------------------------- easing
def clamp01(x):
    return max(0.0, min(1.0, x))


def ease_out_cubic(x):
    x = clamp01(x)
    return 1 - (1 - x) ** 3


def ease_in_cubic(x):
    x = clamp01(x)
    return x ** 3


def ease_out_back(x, s=1.3):
    # peaks at ~1.07: a small overshoot, then settles to 1
    x = clamp01(x)
    x -= 1
    return 1 + x * x * ((s + 1) * x + s)


def ease_in_back(x, s=1.7):
    x = clamp01(x)
    return x * x * ((s + 1) * x - s)


# ---------------------------------------------------------------- video readers
def reader(path, ss=None, t=None, fps=None, size=(1920, 1080)):
    cmd = ["ffmpeg", "-v", "error"]
    if ss is not None:
        cmd += ["-ss", str(ss)]
    if t is not None:
        cmd += ["-t", str(t)]
    cmd += ["-i", path]
    if fps:
        cmd += ["-vf", f"fps={fps}"]
    cmd += ["-f", "rawvideo", "-pix_fmt", "rgb24", "-"]
    proc = subprocess.Popen(cmd, stdout=subprocess.PIPE)
    n = size[0] * size[1] * 3
    while True:
        buf = proc.stdout.read(n)
        if len(buf) < n:
            break
        yield np.frombuffer(buf, np.uint8).reshape(size[1], size[0], 3)
    proc.wait()


# ---------------------------------------------------------------- presenter key
# Source 1920x1080, subject centred around x~937. Crop a 864-wide column and
# scale 1.25x to 1080x1350, bottom-aligned, subject sitting a little left of
# centre to leave room for the picture-in-picture on the right.
PRES_SCALE = 1.25
CROP_X0 = 561
CROP_W = int(round(W / PRES_SCALE))  # 864
PRES_H = int(round(1080 * PRES_SCALE))  # 1350
PRES_Y = H - PRES_H  # 570

KEY_LO, KEY_HI = 0.06, 0.70  # normalised green dominance -> alpha 1 .. 0
PRES_BBOX = [10 ** 9, 10 ** 9, -1, -1]


def key_frame(rgb):
    src = rgb[:, CROP_X0:CROP_X0 + CROP_W].astype(np.float32)
    r, g, b = src[..., 0], src[..., 1], src[..., 2]
    # the screen is not one flat green (two tones meet behind the presenter),
    # so take the key colour per column from the rows above the head
    bg = np.median(src[:30], axis=0)  # (w, 3)
    dg = np.maximum(bg[:, 1] - np.maximum(bg[:, 0], bg[:, 2]), 60.0)
    d = (g - np.maximum(r, b)) / dg[None, :]
    alpha = 1.0 - np.clip((d - KEY_LO) / (KEY_HI - KEY_LO), 0.0, 1.0)
    alpha = np.clip((alpha - 0.12) / 0.88, 0.0, 1.0)  # slight choke
    alpha[alpha > 0.97] = 1.0
    # unmix: remove the screen's contribution, recover straight foreground
    a = alpha[..., None]
    fg = (src - (1.0 - a) * bg[None, :, :]) / np.maximum(a, 1e-3)
    fg = np.clip(fg, 0, 255)
    # despill: green never above the mean of red and blue; spill that is
    # removed becomes a warm tint so the edges sit in the Signal ground
    fr, fgc, fb = fg[..., 0], fg[..., 1], fg[..., 2]
    lim = (fr + fb) * 0.5
    spill = np.clip(fgc - lim, 0, None)
    fgc = fgc - spill
    fr = fr + spill * 0.5
    out = np.stack([fr, fgc, fb, alpha * 255.0], -1)
    ys, xs = np.nonzero(alpha > 0.05)
    if len(xs):
        PRES_BBOX[0] = min(PRES_BBOX[0], xs.min()); PRES_BBOX[1] = min(PRES_BBOX[1], ys.min())
        PRES_BBOX[2] = max(PRES_BBOX[2], xs.max()); PRES_BBOX[3] = max(PRES_BBOX[3], ys.max())
    img = Image.fromarray(np.clip(out + 0.5, 0, 255).astype(np.uint8), "RGBA")
    return img.resize((W, PRES_H), Image.LANCZOS)


# ---------------------------------------------------------------- captions
CAP_FONT = ImageFont.truetype(BOLD, 72)
CAP_MAXW = 900
CAP_CY = 1235
CAP_PADX, CAP_PADY, CAP_LINE = 40, 26, 92

# phrase groups (indices into WORDS), broken at sentence ends and pauses
CHUNKS = [(0, 4), (4, 7), (7, 11), (11, 16), (16, 19), (19, 22), (22, 25)]


def chunk_windows():
    wins = []
    for i, (a, b) in enumerate(CHUNKS):
        s = WORDS[a]["s"]
        e = WORDS[b - 1]["e"] + 0.6
        if i + 1 < len(CHUNKS):
            e = min(e, WORDS[CHUNKS[i + 1][0]]["s"])
        else:
            e = DUR + 1
        wins.append((s, e))
    return wins


CHUNK_WIN = chunk_windows()


def layout_chunk(ci):
    a, b = CHUNKS[ci]
    space = CAP_FONT.getlength(" ")
    lines, cur, curw = [], [], 0.0
    for wi in range(a, b):
        ww = CAP_FONT.getlength(WORDS[wi]["word"])
        add = ww if not cur else curw + space + ww
        if cur and add > CAP_MAXW:
            lines.append((cur, curw))
            cur, curw = [wi], ww
        else:
            cur.append(wi)
            curw = add
    lines.append((cur, curw))
    return lines, space


_cap_cache = {}


def caption_image(ci, active):
    key = (ci, active)
    if key in _cap_cache:
        return _cap_cache[key]
    lines, space = layout_chunk(ci)
    boxw = int(max(lw for _, lw in lines) + 2 * CAP_PADX)
    boxh = int(len(lines) * CAP_LINE + 2 * CAP_PADY - (CAP_LINE - 76))
    pad = 30
    img = Image.new("RGBA", (boxw + 2 * pad, boxh + 2 * pad), (0, 0, 0, 0))
    # soft shadow
    sh = Image.new("RGBA", img.size, (0, 0, 0, 0))
    ImageDraw.Draw(sh).rounded_rectangle(
        (pad, pad + 8, pad + boxw, pad + boxh + 8), 30, fill=(0, 0, 0, 90))
    img = Image.alpha_composite(img, sh.filter(ImageFilter.GaussianBlur(12)))
    dr = ImageDraw.Draw(img)
    dr.rounded_rectangle((pad, pad, pad + boxw, pad + boxh), 30, fill=INK + (240,))
    y = pad + CAP_PADY
    for words, lw in lines:
        x = pad + (boxw - lw) / 2
        for wi in words:
            txt = WORDS[wi]["word"]
            col = SIGNAL if wi == active else PAPER
            dr.text((x, y), txt, font=CAP_FONT, fill=col + (255,), anchor="la")
            x += CAP_FONT.getlength(txt) + space
        y += CAP_LINE
    _cap_cache[key] = img
    return img


def caption_at(t):
    for ci, (s, e) in enumerate(CHUNK_WIN):
        if s <= t < e:
            a, b = CHUNKS[ci]
            active = None
            for wi in range(a, b):
                if WORDS[wi]["s"] <= t < WORDS[wi]["e"]:
                    active = wi
            return caption_image(ci, active)
    return None


# ---------------------------------------------------------------- name bar
NB_X, NB_Y = 60, 1395
NB_ON, NB_OFF = 1.0, 6.0
NB_WIPE_ON, NB_WIPE_OFF = 0.45, 0.40


def build_namebar():
    mark = Image.open(P("brand", "mark-on-dark.png")).convert("RGBA").resize((104, 104), Image.LANCZOS)
    f1 = ImageFont.truetype(BOLD, 80)
    f2 = ImageFont.truetype(REG, 40)
    t1, t2 = "Montagent", "Video your agent can read"
    w = int(max(f1.getlength(t1), f2.getlength(t2))) + 104 + 36 + 48 + 56
    h = 176
    img = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    dr = ImageDraw.Draw(img)
    dr.rounded_rectangle((0, 0, w, h), 22, fill=INK + (255,))
    dr.rectangle((0, 0, 14, h), fill=SIGNAL + (255,))
    dr.rounded_rectangle((0, 0, 28, h), 22, fill=SIGNAL + (255,))
    dr.rectangle((14, 0, 28, h), fill=INK + (255,))
    img.alpha_composite(mark, (48, (h - 104) // 2))
    tx = 48 + 104 + 36
    dr.text((tx, 26), t1, font=f1, fill=PAPER + (255,), anchor="la")
    dr.text((tx, 118), t2, font=f2, fill=PAPER + (215,), anchor="la")
    return img


NAMEBAR = build_namebar()


def namebar_at(t):
    """Returns (image, x) or None. Wipes on left->right, wipes off left->right."""
    w, h = NAMEBAR.size
    if t < NB_ON or t >= NB_OFF + NB_WIPE_OFF:
        return None
    if t < NB_OFF:
        p = ease_out_cubic((t - NB_ON) / NB_WIPE_ON)
        lo, hi = 0, int(round(p * w))
        edge = hi if p < 1 else None
    else:
        q = ease_in_cubic((t - NB_OFF) / NB_WIPE_OFF)
        lo, hi = int(round(q * w)), w
        edge = lo if q > 0 else None
    if hi - lo <= 0:
        return None
    out = Image.new("RGBA", (w + 16, h), (0, 0, 0, 0))
    out.alpha_composite(NAMEBAR.crop((lo, 0, hi, h)), (lo, 0))
    if edge is not None:
        # Signal leading edge that rides the wipe
        ImageDraw.Draw(out).rectangle((max(0, edge - 4), 0, edge + 10, h), fill=SIGNAL + (255,))
    return out


# ---------------------------------------------------------------- brand bug
def build_bug():
    lock = Image.open(P("brand", "lockup-on-dark.png")).convert("RGBA")
    hh = 50
    lock = lock.resize((int(lock.width * hh / lock.height), hh), Image.LANCZOS)
    w, h = lock.width + 56, hh + 36
    img = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    ImageDraw.Draw(img).rounded_rectangle((0, 0, w, h), h // 2, fill=INK + (235,))
    img.alpha_composite(lock, (28, 18))
    return img


BUG = build_bug()
BUG_POS = (60, 120)


# ---------------------------------------------------------------- picture in picture
PIP_IN, PIP_OUT = 3.4, 8.95
PIP_POP, PIP_DROP = 0.5, 0.3
PIP_D = 440
PIP_C = (800, 430)
PIP_RING = 10
SESSION_SS = 33.6  # the diff lands on screen at ~34 s
SESSION_RATE = 0.7  # slowed so the diff stays in view until the PiP leaves
PIP_SRC_C = (430, 470)
PIP_SRC_S0, PIP_SRC_S1 = 840, 700  # slow push-in over the PiP's lifetime


def pip_disc(frame_rgb, t):
    u = clamp01((t - PIP_IN) / (PIP_OUT - PIP_IN))
    s = PIP_SRC_S0 + (PIP_SRC_S1 - PIP_SRC_S0) * (u * u * (3 - 2 * u))
    cx, cy = PIP_SRC_C
    box = (cx - s / 2, cy - s / 2, cx + s / 2, cy + s / 2)
    D = PIP_D + 2 * PIP_RING
    ss = 3  # supersampled mask for a clean circle edge
    content = Image.fromarray(frame_rgb).resize((PIP_D, PIP_D), Image.LANCZOS, box=box)
    disc = Image.new("RGBA", (D, D), (0, 0, 0, 0))
    ring = Image.new("L", (D * ss, D * ss), 0)
    ImageDraw.Draw(ring).ellipse((0, 0, D * ss - 1, D * ss - 1), fill=255)
    ring = ring.resize((D, D), Image.LANCZOS)
    disc.paste(Image.new("RGBA", (D, D), PAPER + (255,)), (0, 0), ring)
    inner = Image.new("L", (PIP_D * ss, PIP_D * ss), 0)
    ImageDraw.Draw(inner).ellipse((0, 0, PIP_D * ss - 1, PIP_D * ss - 1), fill=255)
    inner = inner.resize((PIP_D, PIP_D), Image.LANCZOS)
    disc.paste(content, (PIP_RING, PIP_RING), inner)
    return disc


def pip_scale(t):
    if t < PIP_IN or t >= PIP_OUT + PIP_DROP:
        return 0.0
    if t < PIP_OUT:
        return ease_out_back((t - PIP_IN) / PIP_POP)
    return 1.0 - ease_in_back((t - PIP_OUT) / PIP_DROP) if t < PIP_OUT + PIP_DROP else 0.0


_shadow = None


def pip_shadow():
    global _shadow
    if _shadow is None:
        D = PIP_D + 2 * PIP_RING + 80
        m = Image.new("RGBA", (D, D), (0, 0, 0, 0))
        ImageDraw.Draw(m).ellipse((40, 52, D - 40, D - 28), fill=(60, 10, 0, 110))
        _shadow = m.filter(ImageFilter.GaussianBlur(16))
    return _shadow


# ---------------------------------------------------------------- audio
def read_audio(path, sr=48000, ch=1):
    raw = subprocess.run(
        ["ffmpeg", "-v", "error", "-i", path, "-f", "f32le", "-ac", str(ch), "-ar", str(sr), "-"],
        stdout=subprocess.PIPE, check=True).stdout
    return np.frombuffer(raw, np.float32).reshape(-1, ch).copy()


def rms_db(x):
    return 20 * math.log10(float(np.sqrt(np.mean(x.astype(np.float64) ** 2))) + 1e-12)


def build_audio(out_wav):
    sr = 48000
    n = int(DUR * sr)
    voice = read_audio(P("presenter", "take-1.mp4"), sr, 1)[:, 0]
    music = read_audio(P("music", "bed-120bpm.wav"), sr, 2)
    voice = np.pad(voice, (0, max(0, n - len(voice))))[:n]
    music = music[:n]
    t = np.arange(n) / sr

    # voice: speech RMS to about -19 dBFS
    speech = np.zeros(n, bool)
    for w in WORDS:
        speech[int(w["s"] * sr):int(w["e"] * sr)] = True
    vgain = 10 ** ((-19.0 - rms_db(voice[speech])) / 20)
    voice = voice * vgain

    # music: open level ~7 dB under the voice, ducked a further 11 dB under speech
    mbase = 10 ** ((-19.0 - 7.0 - rms_db(music)) / 20)
    duck_db = -11.0
    phrases = []
    for w in WORDS:
        if phrases and w["s"] - phrases[-1][1] < 0.3:
            phrases[-1][1] = w["e"]
        else:
            phrases.append([w["s"], w["e"]])
    duck = np.zeros(n)  # 0 = open, 1 = fully ducked
    attack, hold, release = 0.12, 0.08, 0.30
    for s, e in phrases:
        a0, a1 = s - attack, s
        r0, r1 = e + hold, e + hold + release
        seg = np.zeros(n)
        seg[(t >= a1) & (t < r0)] = 1
        m = (t >= a0) & (t < a1)
        seg[m] = 0.5 - 0.5 * np.cos(np.pi * (t[m] - a0) / attack)
        m = (t >= r0) & (t < r1)
        seg[m] = 0.5 + 0.5 * np.cos(np.pi * (t[m] - r0) / release)
        duck = np.maximum(duck, seg)
    env = 10 ** (duck * duck_db / 20)
    # fade out: equal-power-ish curve from 8.9 s, silent from 9.85 s
    f0, f1 = 8.9, 9.85
    fade = np.ones(n)
    m = (t >= f0) & (t < f1)
    fade[m] = np.cos(0.5 * np.pi * (t[m] - f0) / (f1 - f0)) ** 2
    fade[t >= f1] = 0.0
    # short fade-in so the bed does not click on at frame 0
    fin = np.clip(t / 0.02, 0, 1)
    mus = music * (mbase * env * fade * fin)[:, None]

    mix = mus + voice[:, None]
    peak = float(np.max(np.abs(mix)))
    if peak > 0.89:
        mix *= 0.89 / peak
    pcm = (np.clip(mix, -1, 1) * 32767).astype("<i2")
    with wave.open(out_wav, "wb") as wf:
        wf.setnchannels(2)
        wf.setsampwidth(2)
        wf.setframerate(sr)
        wf.writeframes(pcm.tobytes())
    print(f"audio: voice gain {20*math.log10(vgain):+.1f} dB, music base {20*math.log10(mbase):+.1f} dB, peak {peak:.2f}")


# ---------------------------------------------------------------- main render
def main():
    out = P("deliverable.mp4")
    wav = os.path.join(TMP, "briefb_mix.wav")
    build_audio(wav)

    vid_tmp = os.path.join(TMP, "briefb_video.mp4")
    enc = subprocess.Popen(
        ["ffmpeg", "-v", "error", "-y", "-f", "rawvideo", "-pix_fmt", "rgb24",
         "-s", f"{W}x{H}", "-r", str(FPS), "-i", "-",
         "-vf", "scale=out_color_matrix=bt709:out_range=tv,format=yuv420p",
         "-c:v", "libx264", "-preset", "slow", "-crf", "16",
         "-colorspace", "bt709", "-color_primaries", "bt709", "-color_trc", "bt709",
         "-color_range", "tv", "-movflags", "+faststart", vid_tmp],
        stdin=subprocess.PIPE)

    pres = reader(P("presenter", "take-1.mp4"))
    sess = reader(P("screen", "session.mp4"), ss=SESSION_SS, t=PIP_OUT - PIP_IN + PIP_DROP + 0.5)
    pres_idx, pres_img, pres_last = -1, None, None
    sess_idx, sess_frame = -1, None
    ground = Image.new("RGBA", (W, H), SIGNAL + (255,))

    for fi in range(NFRAMES):
        t = fi / FPS
        # presenter (25 fps source -> hold the frame covering t; hold last at the tail)
        want = int(math.floor(t * 25 + 1e-6))
        while pres_idx < want:
            try:
                pres_last = next(pres)
                pres_idx += 1
                pres_img = None
            except StopIteration:
                break
        if pres_img is None:
            pres_img = key_frame(pres_last)
        frame = ground.copy()
        frame.alpha_composite(pres_img, (0, PRES_Y))

        # picture in picture
        sc = pip_scale(t)
        if sc > 0.01:
            want_s = int(round((t - PIP_IN) * SESSION_RATE * 30))
            while sess_idx < want_s:
                try:
                    sess_frame = next(sess)
                    sess_idx += 1
                except StopIteration:
                    break
            disc = pip_disc(sess_frame, t)
            sh = pip_shadow()
            D = disc.width
            sz = max(1, int(round(D * sc)))
            shz = max(1, int(round(sh.width * sc)))
            frame.alpha_composite(sh.resize((shz, shz), Image.BILINEAR),
                                  (PIP_C[0] - shz // 2, PIP_C[1] - shz // 2))
            frame.alpha_composite(disc.resize((sz, sz), Image.LANCZOS),
                                  (PIP_C[0] - sz // 2, PIP_C[1] - sz // 2))

        frame.alpha_composite(BUG, BUG_POS)

        cap = caption_at(t)
        if cap is not None:
            frame.alpha_composite(cap, (W // 2 - cap.width // 2, CAP_CY - cap.height // 2))

        nb = namebar_at(t)
        if nb is not None:
            frame.alpha_composite(nb, (NB_X, NB_Y))

        enc.stdin.write(frame.convert("RGB").tobytes())
        if fi % 30 == 0:
            print(f"frame {fi}/{NFRAMES}", flush=True)
    enc.stdin.close()
    enc.wait()

    subprocess.run(
        ["ffmpeg", "-v", "error", "-y", "-i", vid_tmp, "-i", wav,
         "-map", "0:v", "-map", "1:a", "-c:v", "copy", "-c:a", "aac", "-b:a", "192k",
         "-t", str(DUR), "-movflags", "+faststart", out], check=True)
    print("presenter alpha bbox (crop coords)", PRES_BBOX)
    print("wrote", out)


if __name__ == "__main__":
    main()
