#!/usr/bin/env python3
"""PROTOTYPE, throwaway (#833): an audio crossfade, blind A/B. ADR-0176.

Usage:  python3 -I prototype_ab.py [--ffmpeg PATH] [--seed N]      (stdlib + ffmpeg only)

It hand-builds the per-element graph of crates/montagent-core/src/verbs/render.rs
(aformat, atrim, asetpts, [transition gain], adelay, amix normalize=0) with ADR-0176's stage in the
slot it decides (after volume and pan, before adelay), on the shared narration-over-bed fixture.

Two transitions, each bridging two elements over the same 2 s window (9.000-11.000 s):

  pair 1  DIFFERENT SIGNALS: music into music. A = the bed's first 11 s, B = a later part of the bed
          (source 3 s on), placed at 9 s. X/Y = no `audio` key (today's mix) against
          `"audio": "constant_power"`.
  pair 2  THE SAME SIGNAL: a split clip. A = the narration's first 11 s, B = the same narration
          from 9 s on, placed at 9 s, so the two overlap on identical samples.
          X/Y = `"audio": "constant_power"` against `"audio": "constant_gain"`.

Each mix is cut to 5-15 s (the join sits in the middle), brought to -20 LUFS by one gain, and
encoded AAC 160k. Which of X and Y is which is shuffled by --seed into ab/KEY: do not open it
until you have listened. Numbers go to measurements-ab.json.
"""
import argparse, json, random, re, subprocess, sys, tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
FIX = HERE.parent / "fixtures" / "narration-over-bed"
RATE, MATCH = 48000, -20.0
WIN0, WIN1 = 9.0, 11.0                    # the transition's window, in timeline seconds
CUT0, CUT1 = 5.0, 15.0                    # what is written out
CURVE = {"constant_power": "qsin", "constant_gain": "tri"}


def run(ff, args):
    p = subprocess.run([ff, "-hide_banner", "-nostdin", *args], capture_output=True, text=True)
    if p.returncode:
        sys.exit(f"ffmpeg failed: {' '.join(args)}\n{p.stderr[-1500:]}")
    return p.stderr


def meter(ff, path):
    err = run(ff, ["-i", str(path), "-af", "ebur128=peak=true", "-f", "null", "-"])
    tail = err[err.rindex("Summary:"):]
    return (float(re.search(r"I:\s+(-?[\d.]+)\s+LUFS", tail).group(1)),
            float(re.search(r"Peak:\s+(-?[\d.]+)\s+dBFS", tail).group(1)))


def element(idx, src_from, src_to, start, side, audio):
    """One element's chain as render.rs builds it; the transition gain sits last, before adelay."""
    f = (f"[{idx}:a]aformat=sample_rates={RATE}:channel_layouts=stereo,"
         f"atrim=start={src_from}:end={src_to},asetpts=PTS-STARTPTS")
    if audio in CURVE:                     # absent or "cut" adds no stage: today's graph
        st = WIN0 - start if side == "out" else 0.0   # the window, in the element's own time
        f += f",afade=t={side}:st={st}:d={WIN1 - WIN0}:curve={CURVE[audio]}"
    return f + f",adelay=delays={int(start * 1000)}:all=1"


def render(ff, srcs, a, b, audio, out):
    """Two elements, each (src_from, src_to, start): A fades out and B fades in over the window."""
    total = b[2] + (b[1] - b[0])
    graph = (f"{element(0, *a, 'out', audio)}[a];{element(1, *b, 'in', audio)}[b];"
             f"[a][b]amix=inputs=2:normalize=0,apad=whole_dur={total},"
             f"atrim=start={CUT0}:end={CUT1},asetpts=PTS-STARTPTS[m]")
    run(ff, ["-i", str(srcs[0]), "-i", str(srcs[1]), "-y", "-filter_complex", graph, "-map", "[m]",
             "-c:a", "pcm_f32le", str(out)])


def main():
    ap = argparse.ArgumentParser(); ap.add_argument("--ffmpeg", default="ffmpeg"); ap.add_argument("--seed", type=int, default=833)
    args = ap.parse_args(); ff = args.ffmpeg
    bed, nar = FIX / "bed.flac", FIX / "narration.flac"
    pairs = {  # name: (inputs, element A (src_from, src_to, start), element B, [variant X-or-Y candidates])
        "pair1": ((bed, bed), (0.0, 11.0, 0.0), (3.0, 14.0, 9.0), [("today", None), ("constant_power", "constant_power")]),
        "pair2": ((nar, nar), (0.0, 11.0, 0.0), (9.0, 20.0, 9.0), [("constant_power", "constant_power"), ("constant_gain", "constant_gain")]),
    }
    (HERE / "ab").mkdir(exist_ok=True)
    res = {"ffmpeg": subprocess.run([ff, "-version"], capture_output=True, text=True).stdout.splitlines()[0], "window_s": [WIN0, WIN1], "cut_s": [CUT0, CUT1]}
    key, rng = {}, random.Random(args.seed)
    with tempfile.TemporaryDirectory() as w:
        w = Path(w)
        for name, (srcs, a, b, variants) in pairs.items():
            order = list(variants); rng.shuffle(order); res[name] = {}
            for letter, (label, audio) in zip("XY", order):
                raw = w / f"{name}-{label}.wav"
                render(ff, srcs, a, b, audio, raw)
                lufs, peak = meter(ff, raw)
                gain = MATCH - lufs
                dst = HERE / "ab" / f"{name}-{letter}.m4a"
                run(ff, ["-i", str(raw), "-y", "-af", f"volume={gain:.5f}dB", "-c:a", "aac", "-b:a", "160k", "-ar", str(RATE), "-ac", "2", str(dst)])
                dec = w / f"dec-{name}-{letter}.wav"; run(ff, ["-i", str(dst), "-y", str(dec)])
                after, apeak = meter(ff, dec)
                key[f"{name}-{letter}"] = label
                res[name][letter] = {"lufs_before_match": lufs, "peak_dbfs_before_match": peak, "match_gain_db": round(gain, 3), "decoded_lufs": after, "decoded_peak_dbfs": apeak}
                if abs(after - MATCH) > 0.3: sys.exit(f"{name}-{letter}: not loudness-matched ({after})")
    (HERE / "ab" / "KEY").write_text(json.dumps(key, indent=1) + "\n")
    (HERE / "measurements-ab.json").write_text(json.dumps(res, indent=1) + "\n")
    print(json.dumps(res, indent=1))


main()
