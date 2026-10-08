#!/usr/bin/env python3
"""PROTOTYPE, throwaway (#837): a duck under the narration, blind A/B. ADR-0177.

Usage:  python3 -I prototype_ab.py [--ffmpeg PATH] [--seed N]      (stdlib + ffmpeg only)

This is not duck.py. It reproduces what ADR-0177's script would write, on the shared
narration-over-bed fixture, so the owner can judge the sound of the duck itself:

  1. Speech spans come from `silencedetect` on the narration (-35 dB, 150 ms, as captions.py's
     `listen` does). The fixture has no words file, so they stand in for explicit spans.
  2. The keyframes are captions.py's `ducked()` shape with ADR-0177's defaults: under_db -15,
     over_db -6, end_db -1.5, ramp_ms 200, lead_ms 100, join_ms 600, no fade-out. Levels are
     written as linear `volume`, rounded to four decimals. Moving segments use the renderer's
     `ease-in-out` (crates/montagent-core/src/model/keyframe.rs: cubic-bezier(0.42, 0, 0.58, 1));
     held pairs are flat.
  3. The bed is raised 12 dB in BOTH clips. The fixture's own bed sits 15 LU under the voice,
     where a duck is nearly inaudible; raised, it sits 3 LU under, which is about where a library
     track arrives against a voice and is the case a duck exists for.
  4. X and Y are the bed unducked and the bed ducked, each brought to -20 LUFS by one gain and
     encoded AAC 160k. Which is which is shuffled by --seed into ab/KEY: do not open it until
     you have listened.
"""
import argparse, array, json, math, random, re, subprocess, sys, tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
FIX = HERE.parent / "fixtures" / "narration-over-bed"
RATE, MATCH, BED_BOOST_DB = 48000, -20.0, 12.0
SPEC = {"under_db": -15.0, "over_db": -6.0, "end_db": -1.5, "ramp_ms": 200, "lead_ms": 100, "join_ms": 600}
LISTEN = {"noise_db": -35, "min_ms": 150}
lin = lambda d: 10 ** (d / 20)


def run(ff, args, **kw):
    p = subprocess.run([ff, "-hide_banner", "-nostdin", *args], capture_output=True, **kw)
    if p.returncode:
        sys.exit(f"ffmpeg failed: {' '.join(args)}\n{p.stderr.decode(errors='replace')[-1500:]}")
    return p


def meter(ff, path):
    err = run(ff, ["-i", str(path), "-af", "ebur128=peak=true", "-f", "null", "-"]).stderr.decode()
    tail = err[err.rindex("Summary:"):]
    return (float(re.search(r"I:\s+(-?[\d.]+)\s+LUFS", tail).group(1)), float(re.search(r"Peak:\s+(-?[\d.]+)\s+dBFS", tail).group(1)))


def speech_spans(ff, src, total_ms):
    err = run(ff, ["-i", str(src), "-af", f"silencedetect=n={LISTEN['noise_db']}dB:d={LISTEN['min_ms'] / 1000}", "-f", "null", "-"]).stderr.decode()
    sil = list(zip([float(x) * 1000 for x in re.findall(r"silence_start: (-?[\d.]+)", err)],
                   [float(x) * 1000 for x in re.findall(r"silence_end: ([\d.]+)", err)]))
    spans, cur = [], 0.0
    for s, e in sil:
        if s - cur > 1: spans.append([cur, s])
        cur = e
    if total_ms - cur > 1: spans.append([cur, float(total_ms)])
    return [[round(a), round(b)] for a, b in spans if b - a >= 1]


def keyframes(spans, bed_start, spec):
    under, over, end = (round(lin(spec[k]), 4) for k in ("under_db", "over_db", "end_db"))
    ramp, lead, join = spec["ramp_ms"], spec["lead_ms"], spec["join_ms"]
    merged = []
    for s, e in spans:
        if merged and s - merged[-1][1] < join: merged[-1][1] = e
        else: merged.append([s, e])
    pts = [(bed_start, over)]
    for n, (s, e) in enumerate(merged):
        after = end if n + 1 == len(merged) else over
        pts += [(s - lead - ramp, over), (s - lead, under), (e + lead, under), (e + lead + ramp, after)]
    pts = [(max(bed_start, t), v) for t, v in pts]
    if len(pts) > 1 and pts[1][0] <= bed_start: pts[0] = (bed_start, under)
    keys = []
    for t, v in pts:
        if keys and t <= keys[-1]["t"]: keys[-1]["v"] = v; continue
        k = {"t": int(t), "v": v}
        if keys: k["ease"] = "linear" if v == keys[-1]["v"] else "ease-in-out"
        keys.append(k)
    return keys, merged


def bezier_y(p):                       # cubic-bezier(0.42, 0, 0.58, 1): y at x = p
    x1, y1, x2, y2 = 0.42, 0.0, 0.58, 1.0
    lo, hi = 0.0, 1.0
    for _ in range(40):
        t = (lo + hi) / 2
        x = 3 * (1 - t) ** 2 * t * x1 + 3 * (1 - t) * t * t * x2 + t ** 3
        lo, hi = (t, hi) if x < p else (lo, t)
    t = (lo + hi) / 2
    return 3 * (1 - t) ** 2 * t * y1 + 3 * (1 - t) * t * t * y2 + t ** 3


def envelope(keys, n_frames):
    env = array.array("f", [keys[0]["v"]]) * n_frames
    for a, b in zip(keys, keys[1:]):
        i0, i1 = max(0, int(a["t"] * RATE / 1000)), min(n_frames, int(b["t"] * RATE / 1000))
        for i in range(i0, i1):
            p = (i - a["t"] * RATE / 1000) / max(1, (b["t"] - a["t"]) * RATE / 1000)
            env[i] = a["v"] if b["ease"] == "linear" else a["v"] + (b["v"] - a["v"]) * bezier_y(p)
    last = int(keys[-1]["t"] * RATE / 1000)
    for i in range(max(0, last), n_frames): env[i] = keys[-1]["v"]
    return env


def main():
    ap = argparse.ArgumentParser(); ap.add_argument("--ffmpeg", default="ffmpeg"); ap.add_argument("--seed", type=int, default=837)
    a = ap.parse_args(); ff = a.ffmpeg
    nar, bed = FIX / "narration.flac", FIX / "bed.flac"
    total_ms = 20000
    res = {"ffmpeg": run(ff, ["-version"]).stdout.decode().splitlines()[0], "spec": SPEC, "listen": LISTEN, "bed_boost_db": BED_BOOST_DB}
    with tempfile.TemporaryDirectory() as w:
        w = Path(w)
        spans = speech_spans(ff, nar, total_ms)
        keys, merged = keyframes(spans, 0, SPEC)
        res["speech_spans_ms"], res["spans_after_join"], res["keyframes_written"] = spans, merged, keys
        raw = run(ff, ["-i", str(bed), "-af", f"aformat=sample_rates={RATE}:channel_layouts=stereo,volume={BED_BOOST_DB}dB", "-f", "f32le", "-"]).stdout
        pcm = array.array("f"); pcm.frombytes(raw)
        n_frames = len(pcm) // 2
        env = envelope(keys, n_frames)
        ducked = array.array("f", (pcm[i] * env[i // 2] for i in range(len(pcm))))
        files = {"unducked": pcm, "ducked": ducked}
        order = ["unducked", "ducked"]; random.Random(a.seed).shuffle(order)
        key, res["clips"] = {}, {}
        for letter, label in zip("XY", order):
            bed_raw = w / f"bed-{label}.f32"; bed_raw.write_bytes(files[label].tobytes())
            mix = w / f"mix-{label}.wav"
            run(ff, ["-i", str(nar), "-f", "f32le", "-ar", str(RATE), "-ac", "2", "-i", str(bed_raw), "-y", "-filter_complex",
                     f"[0:a]aformat=sample_rates={RATE}:channel_layouts=stereo[v];[v][1:a]amix=inputs=2:normalize=0,atrim=end=20[m]",
                     "-map", "[m]", "-c:a", "pcm_f32le", str(mix)])
            lufs, peak = meter(ff, mix); gain = MATCH - lufs
            dst = HERE / "ab" / f"{letter}.m4a"
            run(ff, ["-i", str(mix), "-y", "-af", f"volume={gain:.5f}dB", "-c:a", "aac", "-b:a", "160k", "-ar", str(RATE), "-ac", "2", str(dst)])
            dec = w / f"dec-{letter}.wav"; run(ff, ["-i", str(dst), "-y", str(dec)])
            after, apeak = meter(ff, dec)
            key[letter] = label
            res["clips"][letter] = {"lufs_before_match": lufs, "peak_dbfs_before_match": peak, "match_gain_db": round(gain, 3), "decoded_lufs": after, "decoded_peak_dbfs": apeak}
            if abs(after - MATCH) > 0.3: sys.exit(f"{letter}: not loudness-matched ({after})")
    (HERE / "ab" / "KEY").write_text(json.dumps(key, indent=1) + "\n")
    (HERE / "measurements-ab.json").write_text(json.dumps(res, indent=1) + "\n")
    print(json.dumps({k: res[k] for k in ("speech_spans_ms", "spans_after_join", "keyframes_written")}, indent=0)[:1800])


main()
