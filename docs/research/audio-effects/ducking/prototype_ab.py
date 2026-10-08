#!/usr/bin/env python3
"""PROTOTYPE, throwaway (#837): a duck under the narration, blind A/B. ADR-0177.

Usage:  python3 -I prototype_ab.py [--ffmpeg PATH] [--seed N]      (stdlib + ffmpeg only)

The shared narration-over-bed fixture, as two elements (the voice, and the bed that is ducked), mixed
the way crates/montagent-core/src/verbs/render.rs builds a mix: aformat, atrim, asetpts, [volume],
adelay, amix normalize=0, 48 kHz stereo, then AAC 160k.

  Three pairs, each shuffled into ab/KEY (do not open it until you have listened). Every clip is brought
  to -20 LUFS by one gain and encoded AAC 160k, so loudness is not a clue.

  pair 1  AS THE TICKET SAYS. The fixture as it is. X/Y = the bed unducked against the bed ducked by
          ADR-0177's defaults (under_db -15, over_db -6, end_db -1.5, ramp_ms 200, lead_ms 100,
          join_ms 600).
  pair 2  THE SAME, WITH A BED THAT COMPETES. The fixture's bed is already 15 LU under the voice, so a
          duck on it is nearly inaudible. Here the bed is raised until it sits 6 LU under the voice,
          which is where a duck earns its place. Unducked against the defaults.
  pair 3  THE PUMP. On the raised bed: the defaults (join_ms 600) against join_ms 300. The fixture's
          pauses are 340-440 ms, shorter than 2 x (lead_ms + ramp_ms) = 600, so at join_ms 300 the bed
          starts to rise in each pause and is pushed down again before it gets there.

The spans come from `silencedetect` on the narration and are written out below as explicit spans:
the fixture has no words file, and this prototype is not the script. The keyframes are
captions.py's `ducked()` maths with the levels spelled in dB.

HONEST APPROXIMATIONS
  * `volume` is applied here as a per-sample gain envelope multiplied in with `amultiply`, not through
    the engine's timed commands. ADR-0175 says the engine is sample-exact on a keyframe's instant;
    this envelope is too, so the *shape* is the same, but the engine itself is not exercised.
  * "ease-in-out" is taken to be CSS's cubic-bezier(0.42, 0, 0.58, 1). The engine's curve was not read.
  * The ffmpeg used is whatever is on PATH, which is below the 7.1 floor ADR-0115 names. Its version
    is recorded in measurements-ab.json, and the verdict records it too.
"""
import argparse, array, hashlib, json, math, random, re, subprocess, sys, tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
FIX = HERE.parent / "fixtures" / "narration-over-bed"
RATE, MATCH = 48000, -20.0
DEFAULTS = dict(under_db=-15.0, over_db=-6.0, end_db=-1.5, ramp_ms=200, lead_ms=100, join_ms=600)
SILENCE_DB, SILENCE_MIN_S = -40, 0.15           # silencedetect, for the prototype only


def run(ff, args):
    p = subprocess.run([ff, "-hide_banner", "-nostdin", *args], capture_output=True, text=True)
    if p.returncode:
        sys.exit(f"ffmpeg failed: {' '.join(args)}\n{p.stderr[-1500:]}")
    return p.stderr


def lin(db):
    return round(10 ** (db / 20), 4)


def spans_from_silence(ff, voice, total_ms):
    """Speech spans as the complement of silencedetect's silences, in ms, in the voice's own time."""
    err = run(ff, ["-i", str(voice), "-af",
                   f"silencedetect=noise={SILENCE_DB}dB:d={SILENCE_MIN_S}", "-f", "null", "-"])
    starts = [float(x) for x in re.findall(r"silence_start:\s*(-?[\d.]+)", err)]
    ends = [float(x) for x in re.findall(r"silence_end:\s*(-?[\d.]+)", err)]
    sil = []
    for i, s in enumerate(starts):
        sil.append((max(0.0, s) * 1000, ends[i] * 1000 if i < len(ends) else float(total_ms)))
    spans, cursor = [], 0.0
    for s, e in sil:
        if s - cursor > 1:
            spans.append([round(cursor), round(s)])
        cursor = max(cursor, e)
    if total_ms - cursor > 1:
        spans.append([round(cursor), round(total_ms)])
    return spans


def ducked(spans, total_ms, d):
    """captions.py's ducked(), levels in dB, no frame snapping and no fade-out (fade_ms is off)."""
    under, over, end = lin(d["under_db"]), lin(d["over_db"]), lin(d["end_db"])
    ramp, lead, join = d["ramp_ms"], d["lead_ms"], d["join_ms"]
    merged = []
    for s, e in spans:
        if merged and s - merged[-1][1] < join:
            merged[-1][1] = e
        else:
            merged.append([s, e])
    points = [(0, over)]
    for n, (s, e) in enumerate(merged):
        after = end if n + 1 == len(merged) else over
        points += [(s - lead - ramp, over), (s - lead, under), (e + lead, under), (e + lead + ramp, after)]
    points = [(max(0, min(t, total_ms)), v) for t, v in points]
    if points[1][0] <= 0:
        points[0] = (0, under)
    keys = []
    for t, v in points:
        v = round(v, 4)
        if keys and t <= keys[-1]["t"]:
            keys[-1]["v"] = v
            continue
        key = {"t": t, "v": v}
        if keys:
            key["ease"] = "linear" if v == keys[-1]["v"] else "ease-in-out"
        keys.append(key)
    return merged, keys


def bezier_ease(x, x1=0.42, y1=0.0, x2=0.58, y2=1.0):
    """CSS cubic-bezier easing: y at the parameter whose x is `x`."""
    lo, hi = 0.0, 1.0
    for _ in range(40):
        mid = (lo + hi) / 2
        bx = 3 * (1 - mid) ** 2 * mid * x1 + 3 * (1 - mid) * mid ** 2 * x2 + mid ** 3
        lo, hi = (mid, hi) if bx < x else (lo, mid)
    s = (lo + hi) / 2
    return 3 * (1 - s) ** 2 * s * y1 + 3 * (1 - s) * s ** 2 * y2 + s ** 3


def envelope(keys, total_ms, path):
    """A stereo f32le gain envelope, one value per sample, from the keyframes."""
    n = int(total_ms * RATE / 1000)
    mono = array.array("f", [keys[-1]["v"]]) * n
    first = int(keys[0]["t"] * RATE / 1000)
    for i in range(min(first, n)):
        mono[i] = keys[0]["v"]
    for a, b in zip(keys, keys[1:]):
        i0, i1 = int(a["t"] * RATE / 1000), min(n, int(b["t"] * RATE / 1000))
        span = max(1, i1 - i0)
        for i in range(i0, i1):
            u = (i - i0) / span
            e = u if b["ease"] == "linear" else bezier_ease(u)
            mono[i] = a["v"] + (b["v"] - a["v"]) * e
    st = array.array("f", [0.0]) * (2 * n)
    st[0::2] = mono
    st[1::2] = mono
    Path(path).write_bytes(st.tobytes())
    return n


def mix(ff, voice, bed, total_s, env, out_wav):
    """The voice and the bed as two elements, with the bed's volume curve (or none), amix normalize=0."""
    ins = ["-i", str(voice), "-i", str(bed)]
    f = [f"[0:a]aformat=sample_rates={RATE}:channel_layouts=stereo,atrim=start=0:end={total_s},"
         f"asetpts=PTS-STARTPTS,adelay=delays=0:all=1[v]",
         f"[1:a]aformat=sample_rates={RATE}:channel_layouts=stereo,atrim=start=0:end={total_s},"
         f"asetpts=PTS-STARTPTS[b0]"]
    if env:
        ins += ["-f", "f32le", "-ar", str(RATE), "-ac", "2", "-i", str(env)]
        f.append("[b0][2:a]amultiply,adelay=delays=0:all=1[b]")
    else:
        f.append("[b0]adelay=delays=0:all=1[b]")
    f.append("[v][b]amix=inputs=2:normalize=0:duration=longest[m]")
    run(ff, [*ins, "-filter_complex", ";".join(f), "-map", "[m]", "-c:a", "pcm_f32le", "-y", str(out_wav)])


def meter(ff, path):
    err = run(ff, ["-i", str(path), "-af", "ebur128=peak=true", "-f", "null", "-"])
    tail = err[err.rindex("Summary:"):]
    return (float(re.search(r"I:\s+(-?[\d.]+)\s+LUFS", tail).group(1)),
            float(re.search(r"Peak:\s+(-?[\d.]+)\s+dBFS", tail).group(1)))


def rms_db(ff, path, a_ms, b_ms):
    err = run(ff, ["-i", str(path), "-af",
                   f"atrim=start={a_ms / 1000}:end={b_ms / 1000},astats=metadata=0:reset=0", "-f", "null", "-"])
    return float(re.findall(r"RMS level dB:\s+(-?[\d.]+|-inf)", err)[-1])


def longest_hold(keys, level):
    best = None
    for a, b in zip(keys, keys[1:]):
        if a["v"] == b["v"] == level and (best is None or b["t"] - a["t"] > best[1] - best[0]):
            best = (a["t"], b["t"])
    return best


def probe_bed(ff, bed, voice, total_s, keys, tmp, bed_gain_db):
    """What a duck does to the bed alone, in a speech hold and in a pause hold (render, before the encoder)."""
    env = tmp / "probe-env.raw"
    envelope(keys, total_s * 1000, env)
    plain, duck = tmp / "bp.wav", tmp / "bd.wav"
    for path, e in ((plain, None), (duck, env)):
        run(ff, ["-i", str(bed), *( ["-f", "f32le", "-ar", str(RATE), "-ac", "2", "-i", str(e)] if e else [] ),
                 "-filter_complex",
                 f"[0:a]aformat=sample_rates={RATE}:channel_layouts=stereo,volume={bed_gain_db}dB,"
                 f"atrim=start=0:end={total_s},asetpts=PTS-STARTPTS" + ("[b0];[b0][1:a]amultiply[m]" if e else "[m]"),
                 "-map", "[m]", "-c:a", "pcm_f32le", "-y", str(path)])
    probes = {}
    for label, lvl, want in (("speech", lin(DEFAULTS["under_db"]), DEFAULTS["under_db"]),
                             ("pause", lin(DEFAULTS["over_db"]), DEFAULTS["over_db"])):
        h = longest_hold(keys, lvl)
        if h:
            a, b = h[0] + 50, h[1] - 50
            probes[label] = {"window_ms": [a, b], "expected_db": want,
                             "measured_db": round(rms_db(ff, duck, a, b) - rms_db(ff, plain, a, b), 3)}
    return probes


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--ffmpeg", default="ffmpeg")
    ap.add_argument("--seed", type=int, default=837)
    args = ap.parse_args()
    ff = args.ffmpeg
    voice, bed = FIX / "narration.flac", FIX / "bed.flac"
    total_s = 20
    total_ms = total_s * 1000
    ver = subprocess.run([ff, "-version"], capture_output=True, text=True).stdout.splitlines()[0]

    spans = spans_from_silence(ff, voice, total_ms)
    print("explicit spans (ms):", spans)
    out = {"ffmpeg": ver, "seed": args.seed, "defaults": DEFAULTS, "silencedetect": f"{SILENCE_DB}dB d={SILENCE_MIN_S}",
           "spans_ms": spans, "match_lufs": MATCH, "pairs": {}}
    cfg300 = dict(DEFAULTS, join_ms=300)
    raised = None                                    # set after the fixture bed is measured
    (HERE / "ab").mkdir(exist_ok=True)
    rng = random.Random(args.seed)
    key_lines = []
    with tempfile.TemporaryDirectory() as tmp:
        tmp = Path(tmp)
        v_lufs, _ = meter(ff, voice)
        b_lufs, _ = meter(ff, bed)
        raised = round((v_lufs - 6.0) - b_lufs, 2)    # bed 6 LU under the voice
        out["voice_lufs"], out["bed_lufs"], out["raised_bed_gain_db"] = v_lufs, b_lufs, raised

        plan = [("pair1", 0.0, [("unducked", None), ("ducked", DEFAULTS)]),
                ("pair2", raised, [("unducked", None), ("ducked", DEFAULTS)]),
                ("pair3", raised, [("join600", DEFAULTS), ("join300", cfg300)])]
        for pair, bed_gain, variants in plan:
            info = {"bed_gain_db": bed_gain, "clips": {}}
            wavs = {}
            for name, cfg in variants:
                env = None
                if cfg:
                    merged, keys = ducked(spans, total_ms, cfg)
                    env = tmp / f"{pair}-{name}.raw"
                    envelope(keys, total_ms, env)
                    info["clips"][name] = {"config": cfg, "merged_spans_ms": merged, "keyframes": keys}
                else:
                    info["clips"][name] = {"config": None}
                bedsrc = tmp / f"{pair}-bed.wav"
                if not bedsrc.exists():
                    run(ff, ["-i", str(bed), "-af", f"volume={bed_gain}dB", "-c:a", "pcm_f32le", "-y", str(bedsrc)])
                wav = tmp / f"{pair}-{name}.wav"
                mix(ff, voice, bedsrc, total_s, env, wav)
                lufs, peak = meter(ff, wav)
                gain = MATCH - lufs
                fin = tmp / f"{pair}-{name}-m.wav"
                run(ff, ["-i", str(wav), "-af", f"volume={gain}dB", "-c:a", "pcm_f32le", "-y", str(fin)])
                lufs2, peak2 = meter(ff, fin)
                info["clips"][name].update({"mix_lufs": lufs, "mix_peak_dbfs": peak, "gain_db": round(gain, 3),
                                            "matched_lufs": lufs2, "matched_peak_dbfs": peak2})
                wavs[name] = fin
            _, dkeys = ducked(spans, total_ms, DEFAULTS)
            info["bed_only_probes_defaults"] = probe_bed(ff, bed, voice, total_s, dkeys, tmp, bed_gain)
            order = [n for n, _ in variants]
            rng.shuffle(order)
            for letter, name in zip("XY", order):
                run(ff, ["-i", str(wavs[name]), "-c:a", "aac", "-b:a", "160k", "-ar", str(RATE), "-ac", "2",
                         "-y", str(HERE / "ab" / f"{pair}-{letter}.m4a")])
                key_lines.append(f"{pair}-{letter}: {name}\n")
            out["pairs"][pair] = info
    text = "".join(key_lines)
    (HERE / "ab" / "KEY").write_text(text)
    out["key_sha256"] = hashlib.sha256(text.encode()).hexdigest()
    (HERE / "measurements-ab.json").write_text(json.dumps(out, indent=2) + "\n")
    print("wrote ab/pair{1,2,3}-{X,Y}.m4a, ab/KEY (do not open it), measurements-ab.json")
    print("raised bed gain:", raised, "dB   KEY sha256:", out["key_sha256"])


if __name__ == "__main__":
    main()
