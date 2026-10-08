#!/usr/bin/env python3
"""PROTOTYPE, throwaway: the noise gate as an `audio_effects` member (ADR-0181 draft).
Usage: python3 -I prototype_gate.py [--ffmpeg PATH] [--seed N]   (stdlib + ffmpeg only)

  {"name":"noise_gate","threshold_db":-38,"ratio":10,"attack_ms":5,"release_ms":150,"range_db":30}

The shared TTS narration has digital silence between phrases, so a gate has nothing to do on it.
This adds a seeded pink-noise floor (about -48 dBFS RMS; pink noise reads 14.3 dB under its amplitude) to the narration (what a recorded voice sounds
like) and renders two blind A/B clips through AAC 160k: the floor ungated against the floor gated.
Numbers: the floor's RMS in the pauses and the speech's RMS, bypass against gated, on PCM.
Which of X/Y is the gated one is in ab/KEY: do not open it until you have judged.
"""
import argparse, array, json, math, random, subprocess, sys
from pathlib import Path
HERE = Path(__file__).resolve().parent
NARR = HERE.parent / "fixtures" / "narration-over-bed" / "narration.flac"
lin = lambda db: 10 ** (db / 20)
M = {"name": "noise_gate", "threshold_db": -34, "ratio": 10, "attack_ms": 5, "release_ms": 150, "range_db": 30}
GATE = (f"agate=threshold={lin(M['threshold_db']):.9f}:ratio={(M['ratio'] + 1) / 2}:attack={M['attack_ms']}:"
        f"release={M['release_ms']}:range={lin(-M['range_db']):.9f}:knee=1:detection=rms:link=average")
# pauses of the narration (silencedetect -40 dB, d=0.2) and a stretch of speech
PAUSES = [(0.1, 0.75), (5.97, 6.12), (8.08, 8.23), (13.7, 13.93), (15.65, 15.84), (19.65, 19.99)]  # each starts 0.2 s after speech ends: the release has run
SPEECH = [(1.0, 5.5), (8.5, 13.3), (16.0, 19.2)]


def ff(F, args, out=False):
    p = subprocess.run([F, "-hide_banner", "-nostdin", *args], capture_output=True, text=True)
    if p.returncode:
        sys.exit(p.stderr[-1500:])
    return p.stderr


def noisy(F, dst, gate, seed):
    fc = (f"[0:a]aformat=sample_fmts=dbl:sample_rates=48000:channel_layouts=mono[v];"
          f"anoisesrc=color=pink:amplitude={lin(-48 + 14.3):.6f}:seed={seed}:d=20:r=48000,aformat=sample_fmts=dbl:channel_layouts=mono[n];"
          f"[v][n]amix=inputs=2:normalize=0:duration=first,{GATE + ',' if gate else ''}aformat=sample_rates=48000:channel_layouts=stereo")
    ff(F, ["-y", "-i", str(NARR), "-filter_complex", fc, *(["-c:a", "pcm_f32le"] if str(dst).endswith(".wav") else ["-c:a", "aac", "-b:a", "160k"]), str(dst)])


def rms(F, wav, spans):
    """RMS dBFS over the union of spans, read from the PCM (left channel)."""
    raw = subprocess.run([F, "-hide_banner", "-loglevel", "error", "-i", str(wav), "-af", "pan=mono|c0=c0,aformat=sample_fmts=dbl", "-f", "f64le", "-"],
                         capture_output=True, check=True).stdout
    x = array.array("d")
    x.frombytes(raw)
    seg = [v for lo, hi in spans for v in x[int(lo * 48000):int(hi * 48000)]]
    return 10 * math.log10(sum(v * v for v in seg) / len(seg) + 1e-300)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--ffmpeg", default="ffmpeg")
    ap.add_argument("--seed", type=int, default=795)
    a = ap.parse_args()
    F, out = a.ffmpeg, HERE / "ab"
    out.mkdir(exist_ok=True)
    res = {"ffmpeg": ff(F, ["-version"]).splitlines()[0] if False else subprocess.run([F, "-version"], capture_output=True, text=True).stdout.splitlines()[0], "member": M}
    for tag, gate in (("bypass", False), ("gated", True)):
        w = HERE / f"_{tag}.wav"
        noisy(F, w, gate, 7)
        res[tag] = {"pause_rms_db": round(rms(F, w, PAUSES), 2), "speech_rms_db": round(rms(F, w, SPEECH), 2)}
        w.unlink()
    res["pause_reduction_db"] = round(res["gated"]["pause_rms_db"] - res["bypass"]["pause_rms_db"], 2)
    res["speech_change_db"] = round(res["gated"]["speech_rms_db"] - res["bypass"]["speech_rms_db"], 2)
    random.seed(a.seed)
    gated_is_x = random.random() < 0.5
    noisy(F, out / ("X.m4a" if gated_is_x else "Y.m4a"), True, 7)
    noisy(F, out / ("Y.m4a" if gated_is_x else "X.m4a"), False, 7)
    (out / "KEY").write_text(f"X = {'gated' if gated_is_x else 'bypass (noise floor ungated)'}\nY = {'bypass (noise floor ungated)' if gated_is_x else 'gated'}\n")
    (HERE / "measurements-ab.json").write_text(json.dumps(res, indent=1) + "\n")
    print(json.dumps(res, indent=1))


main()
