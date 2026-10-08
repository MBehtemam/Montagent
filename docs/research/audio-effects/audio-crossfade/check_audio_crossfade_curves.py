#!/usr/bin/env python3
"""Re-runnable check for ADR-0176 (audio crossfade curves). Exits non-zero when a number stops holding.

Usage:  python3 -I check_audio_crossfade_curves.py [--ffmpeg PATH] [--out measurements.json]
        (stdlib + ffmpeg only)

The renderer's crossfade is two `afade` chains summed by the existing `amix normalize=0`:
the `from` side fades out, the `to` side fades in, over the transition's window. This script
measures the three claims ADR-0176 rests on, on lavfi sines (the expected value follows from the
signal's definition, ADR-0173 §3). Every figure is read from PCM before any encoder.

  1. Two UNRELATED signals (500 Hz and 750 Hz, orthogonal over the 40 ms window): at the
     window's midpoint `constant_power` (afade qsin) holds the level, 0 dB; `constant_gain`
     (afade tri) dips by 3.01 dB.
  2. The SAME signal on both sides: `constant_gain` holds the level, 0 dB; `constant_power`
     bumps +3.01 dB.
  3. The window is sample-exact: the last sample before the window is untouched, the window's
     first sample is still full level, the outgoing side is silent at the window's end, and
     the incoming side is silent at the window's start and full at its end.
"""
import argparse, array, json, math, subprocess, sys

RATE = 48000
ST, D = 2.0, 2.0            # the window: starts at 2.000 s, lasts 2.000 s
MID = ST + D / 2
HALF = 0.02                 # measure 40 ms around the midpoint: 20 and 30 whole cycles
TOL_DB = 0.10               # provisional: max(2 x the spread across the three CI legs, meter resolution)
CURVES = {"constant_power": "qsin", "constant_gain": "tri"}


def pcm(ff, graph):
    r = subprocess.run([ff, "-hide_banner", "-nostdin", "-filter_complex", graph, "-map", "[m]",
                        "-ac", "1", "-ar", str(RATE), "-f", "f32le", "-"], capture_output=True)
    if r.returncode:
        sys.exit(r.stderr.decode()[-1500:])
    a = array.array("f"); a.frombytes(r.stdout); return a


def sine(f, label):
    return f"sine=f={f}:r={RATE}:d=6,volume=0.5,aformat=channel_layouts=mono[{label}]"


def rms_db(x, t0, t1):
    s = x[int(t0 * RATE):int(t1 * RATE)]
    return 10 * math.log10(sum(v * v for v in s) / len(s))


def mix(ff, curve, f_from, f_to):
    g = (f"{sine(f_from, 'a')};{sine(f_to, 'b')};"
         f"[a]afade=t=out:st={ST}:d={D}:curve={curve}[fa];[b]afade=t=in:st={ST}:d={D}:curve={curve}[fb];"
         f"[fa][fb]amix=inputs=2:normalize=0[m]")
    return pcm(ff, g)


def main():
    ap = argparse.ArgumentParser(); ap.add_argument("--ffmpeg", default="ffmpeg"); ap.add_argument("--out")
    a = ap.parse_args(); ff = a.ffmpeg
    ver = subprocess.run([ff, "-version"], capture_output=True, text=True).stdout.splitlines()[0]
    res, fails = {"ffmpeg": ver, "tolerance_db": TOL_DB}, []
    ref = pcm(ff, f"{sine(500, 'r')};[r]anull[m]")
    ref_db = rms_db(ref, MID - HALF, MID + HALF)          # 500 Hz at 0.5 amplitude: -9.03 dB RMS
    for case, (f2, want) in {"unrelated": (750, {"constant_power": 0.0, "constant_gain": -3.01}),
                             "same_signal": (500, {"constant_power": 3.01, "constant_gain": 0.0})}.items():
        res[case] = {}
        for name, curve in CURVES.items():
            got = rms_db(mix(ff, curve, 500, f2), MID - HALF, MID + HALF) - ref_db
            res[case][name] = {"afade_curve": curve, "midpoint_db_vs_one_side": round(got, 3), "expected_db": want[name]}
            if abs(got - want[name]) > TOL_DB:
                fails.append(f"{case} {name}: {got:.3f} dB, expected {want[name]} +/- {TOL_DB}")
    # the window edges, one side at a time, on a constant signal so every sample is its own gain
    edge = {}
    s0, s1 = int(ST * RATE), int((ST + D) * RATE)
    for name, curve in CURVES.items():
        out = pcm(ff, f"aevalsrc=0.5:s={RATE}:d=6:c=mono,afade=t=out:st={ST}:d={D}:curve={curve}[m]")
        inn = pcm(ff, f"aevalsrc=0.5:s={RATE}:d=6:c=mono,afade=t=in:st={ST}:d={D}:curve={curve}[m]")
        edge[name] = {"out_before_window": out[s0 - 1], "out_first_window_sample": out[s0],
                      "out_last_window_sample": out[s1 - 1], "out_at_window_end": out[s1],
                      "in_before_window": inn[s0 - 1], "in_first_window_sample": inn[s0],
                      "in_at_window_end": inn[s1]}
        e = edge[name]
        if e["out_before_window"] != 0.5 or abs(e["out_first_window_sample"] - 0.5) > 1e-4: fails.append(f"{name}: outgoing side starts off the window: {e}")
        if not 0 < e["out_last_window_sample"] < 0.01 or e["out_at_window_end"] != 0.0: fails.append(f"{name}: outgoing side does not end on the window: {e}")
        if e["in_before_window"] != 0.0 or e["in_first_window_sample"] != 0.0: fails.append(f"{name}: incoming side starts off the window: {e}")
        if abs(e["in_at_window_end"] - 0.5) > 1e-6: fails.append(f"{name}: incoming side is not full at the window end: {e}")
    res["window_edges"] = edge
    if a.out:
        open(a.out, "w").write(json.dumps(res, indent=1) + "\n")
    print(json.dumps(res, indent=1))
    if fails:
        sys.exit("FAILED:\n  " + "\n  ".join(fails))


main()
