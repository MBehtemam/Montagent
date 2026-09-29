"""Synthesise the pack's music bed: 16 s at 120 BPM, 48 kHz stereo WAV.

Deterministic: the same script writes the same bytes. Run with
`uv run --with numpy python ../pack-src/make_music_bed.py music/bed-120bpm.wav` from `assets/`.

Beat grid: 120 BPM, one beat every 0.5 s, the first downbeat at 0.000 s, four beats
to a bar, so bars start at 0, 2, 4, ... 14 s. The last bar (14-16 s) rings out on a
held chord with no drums, so a fade over the final second has something to fade.
"""

import sys
import wave

import numpy as np

SR = 48_000
BPM = 120
BEAT = 60 / BPM
LENGTH = 16.0
N = int(SR * LENGTH)
rng = np.random.default_rng(465)
t_all = np.arange(N) / SR


def env(n, attack, decay):
    a = int(SR * attack)
    e = np.exp(-np.arange(n) / (SR * decay))
    e[:a] *= np.linspace(0, 1, a, endpoint=False) if a else 1
    return e


def place(buf, sig, at):
    i = int(round(at * SR))
    j = min(N, i + len(sig))
    buf[i:j] += sig[: j - i]


def kick():
    n = int(SR * 0.35)
    t = np.arange(n) / SR
    freq = 45 + 90 * np.exp(-t / 0.03)
    return np.sin(2 * np.pi * np.cumsum(freq) / SR) * env(n, 0.002, 0.12)


def hat():
    n = int(SR * 0.06)
    noise = rng.standard_normal(n)
    noise = np.diff(noise, prepend=0)  # tilt towards the highs
    return 0.25 * noise * env(n, 0.001, 0.015)


def clap():
    n = int(SR * 0.18)
    return 0.35 * rng.standard_normal(n) * env(n, 0.003, 0.05)


def midi(m):
    return 440 * 2 ** ((m - 69) / 12)


def pad(notes, dur):
    n = int(SR * dur)
    t = np.arange(n) / SR
    sig = np.zeros(n)
    for m in notes:
        f = midi(m)
        for detune in (-0.12, 0.0, 0.12):
            sig += np.sin(2 * np.pi * f * (1 + detune / 100) * t + rng.uniform(0, 2 * np.pi))
    shape = np.minimum(1, t / 0.08) * np.minimum(1, (dur - t) / 0.25)
    return sig / (3 * len(notes)) * np.clip(shape, 0, 1)


def pluck(m, dur=0.22):
    n = int(SR * dur)
    t = np.arange(n) / SR
    f = midi(m)
    return (np.sin(2 * np.pi * f * t) + 0.3 * np.sin(4 * np.pi * f * t)) * env(n, 0.002, 0.07)


# I-V-vi-IV in C, one chord per bar, repeated; the last bar holds the tonic.
CHORDS = [[48, 55, 60, 64], [43, 55, 59, 62], [45, 57, 60, 64], [41, 53, 57, 60]]
ARP = [0, 2, 3, 2]

drums = np.zeros(N)
music = np.zeros(N)
for bar in range(8):
    start = bar * 4 * BEAT
    chord = CHORDS[bar % 4] if bar < 7 else CHORDS[0]
    last = bar == 7
    place(music, 0.55 * pad(chord, 4 * BEAT if not last else 2.0), start)
    if last:
        continue
    for beat in range(4):
        at = start + beat * BEAT
        place(drums, 0.9 * kick(), at)
        place(drums, hat(), at + BEAT / 2)
        if beat in (1, 3):
            place(drums, clap(), at)
        for eighth in range(2):
            note = chord[ARP[(beat * 2 + eighth) % 4]] + 12
            place(music, 0.18 * pluck(note), at + eighth * BEAT / 2)

left = 0.8 * drums + music
right = 0.8 * drums + music
# a slow stereo drift on the pads so the bed is not mono
left += 0.05 * np.sin(2 * np.pi * 0.25 * t_all) * music
right -= 0.05 * np.sin(2 * np.pi * 0.25 * t_all) * music
stereo = np.stack([left, right], axis=1)
stereo *= 0.5 / np.max(np.abs(stereo))  # peak at about -6 dBFS
pcm = (stereo * 32767).astype("<i2")

with wave.open(sys.argv[1], "wb") as w:
    w.setnchannels(2)
    w.setsampwidth(2)
    w.setframerate(SR)
    w.writeframes(pcm.tobytes())
