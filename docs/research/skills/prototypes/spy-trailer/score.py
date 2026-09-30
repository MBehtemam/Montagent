"""Prototype: the trailer's score and sound design, synthesised from scratch.
Run: uv run --with numpy --with scipy python score.py -> score.wav (48 kHz stereo)"""
import numpy as np
from scipy.io import wavfile
from scipy.signal import butter, lfilter

from plan import BIG_HITS, COLD, DOTS, DROP, DURATION, HITS, HUD, IRIS, MONTAGE, TITLE

SR = 48_000
rng = np.random.default_rng(11)
mix = np.zeros((int(SR * DURATION / 1000) + SR, 2))


def at(ms):
    return int(SR * ms / 1000)


def lp(x, hz, order=2):
    b, a = butter(order, hz / (SR / 2), "low")
    return lfilter(b, a, x)


def hp(x, hz, order=2):
    b, a = butter(order, hz / (SR / 2), "high")
    return lfilter(b, a, x)


def bp(x, lo, hi):
    b, a = butter(2, [lo / (SR / 2), hi / (SR / 2)], "band")
    return lfilter(b, a, x)


def add(ms, x, gain=1.0, pan=0.0):
    i = at(ms)
    x = x[: len(mix) - i]
    mix[i:i + len(x), 0] += x * gain * (1 - pan) ** 0.5 / 2 ** -0.5 / 2
    mix[i:i + len(x), 1] += x * gain * (1 + pan) ** 0.5 / 2 ** -0.5 / 2


def t_(dur):
    return np.arange(int(SR * dur)) / SR


def saw(f, t):
    ph = np.cumsum(np.broadcast_to(f, t.shape)) / SR
    return 2 * (ph % 1) - 1


def env(t, a, d):
    return np.minimum(t / a, 1) * np.exp(-t / d)


def braam(dur=3.0, big=False):
    t = t_(dur)
    x = sum(saw(f * (1 + dt), t) for f in (55, 82.41, 110, 164.8) for dt in (-0.004, 0, 0.005))
    x = lp(x, 900 if big else 600, 2) * env(t, 0.015, dur / 3)
    sub = np.sin(2 * np.pi * np.cumsum(np.linspace(70, 32, len(t))) / SR) * env(t, 0.005, 0.9)
    noise = hp(rng.normal(0, 1, len(t)), 200) * env(t, 0.002, 0.08)
    return 0.16 * x + 0.9 * sub + 0.25 * noise


def boom():
    t = t_(1.6)
    sub = np.sin(2 * np.pi * np.cumsum(np.linspace(80, 30, len(t))) / SR) * env(t, 0.003, 0.5)
    crack = hp(rng.normal(0, 1, len(t)), 800) * env(t, 0.001, 0.04)
    return 0.9 * sub + 0.35 * crack


def tick(accent=False):
    t = t_(0.06)
    click = bp(rng.normal(0, 1, len(t)), 2500, 7000) * env(t, 0.0005, 0.006)
    tone = np.sin(2 * np.pi * (1900 if accent else 1500) * t) * env(t, 0.0005, 0.01)
    return 0.6 * click + 0.25 * tone


def whoosh(dur=0.5, rise=True):
    t = t_(dur)
    n = rng.normal(0, 1, len(t))
    shape = (t / dur) ** 2 if rise else (1 - t / dur) ** 2
    # a sweeping band: filter the noise in chunks with a moving centre frequency
    out = np.zeros_like(n)
    step = 1200
    for i in range(0, len(n), step):
        f = 300 + 3000 * ((i / len(n)) if rise else 1 - i / len(n))
        out[i:i + step] = bp(n[max(0, i - 2000):i + step], f * 0.7, f * 1.4)[-len(n[i:i + step]):]
    return out * shape * 0.5


def tom(f0=110):
    t = t_(0.5)
    body = np.sin(2 * np.pi * np.cumsum(np.linspace(f0, f0 * 0.6, len(t))) / SR) * env(t, 0.002, 0.18)
    skin = bp(rng.normal(0, 1, len(t)), 200, 1200) * env(t, 0.001, 0.03)
    return 0.8 * body + 0.3 * skin


def pad(dur, notes=(110, 130.8, 164.8), cutoff=500):
    t = t_(dur)
    lfo = 1 + 0.004 * np.sin(2 * np.pi * 0.2 * t)
    x = sum(saw(f * lfo * (1 + dt), t) for f in notes for dt in (-0.003, 0.003))
    x = lp(x, cutoff, 2)
    fade = np.minimum(np.minimum(t / 1.5, 1), np.minimum((dur - t) / 1.5, 1))
    return x * fade * 0.05


def riser(dur):
    t = t_(dur)
    u = t / dur
    tone = sum(np.sin(2 * np.pi * np.cumsum(f * 2 ** (2 * u)) / SR) for f in (220, 330, 440))
    noise = hp(rng.normal(0, 1, len(t)), 2000) * u ** 3
    return (0.12 * tone * u ** 2 + 0.25 * noise) * 0.8


def blip():
    t = t_(0.07)
    return np.sign(np.sin(2 * np.pi * rng.choice([880, 1320, 1760, 2640]) * t)) * env(t, 0.001, 0.02) * 0.15


# --- cold open: ticking clock, a whoosh per dot, the iris hum rising into the boom
for ms in range(COLD[0], COLD[1] - 200, 500):
    add(ms, tick(ms % 1000 == 0), 0.8)
for n, ms in enumerate(DOTS):
    add(ms - 150, whoosh(0.35), 0.5, pan=-0.6 + 0.4 * n)
hum = t_((COLD[1] - IRIS) / 1000)
add(IRIS, np.sin(2 * np.pi * 55 * hum) * (hum / hum[-1]) ** 2 * 0.35 + lp(rng.normal(0, 1, len(hum)), 300) * (hum / hum[-1]) ** 3 * 0.3)

# --- the bed under the dialogue, in A minor, with a darker section under the villain
add(COLD[1], pad((MONTAGE[0] - COLD[1]) / 1000 + 0.8), 1.0)
add(12000, pad(4.4, notes=(87.3, 110, 130.8), cutoff=380), 1.2)

# --- hits, each with a whoosh arriving into it
for ms in HITS:
    add(ms - 450, whoosh(0.45), 0.7)
    if ms in BIG_HITS:
        add(ms, braam(4.0 if ms == TITLE[0] else 3.0, big=True), 1.0)
    else:
        add(ms, boom(), 0.9)
        add(ms, braam(1.6), 0.5)

# --- HUD: digital blips
for ms in range(HUD[0], HUD[1], 90):
    add(ms, blip(), 0.8, pan=rng.uniform(-0.7, 0.7))

# --- montage: toms on every cut, doubling up, and a riser; then cut to silence
for i, ms in enumerate(range(MONTAGE[0], MONTAGE[1], 250)):
    add(ms, tom(110 if i % 2 == 0 else 82), 0.9 if i % 2 == 0 else 0.5)
add(MONTAGE[0], riser((MONTAGE[1] - MONTAGE[0]) / 1000), 1.0)
add(MONTAGE[1] - 400, whoosh(0.4), 0.6)

# --- the drop: a slow heartbeat of ticks
for ms in range(DROP[0] + 200, DROP[1] - 300, 400):
    add(ms, tick(True), 0.5)

# --- the title: a high sustained note over the braam tail
tt = t_(5.0)
add(TITLE[0] + 300, np.sin(2 * np.pi * 880 * tt) * np.minimum(tt / 1.0, 1) * np.exp(-tt / 3) * 0.05)
add(TITLE[0] + 300, np.sin(2 * np.pi * 1318.5 * tt) * np.minimum(tt / 1.2, 1) * np.exp(-tt / 3) * 0.03)

mix = mix[: at(DURATION)]
mix /= np.abs(mix).max() / 0.89
wavfile.write("score.wav", SR, (mix * 32767).astype(np.int16))
print("score.wav", len(mix) / SR, "s")
