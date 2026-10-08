#!/usr/bin/env python3
"""Measured check for the `reduce_noise` audio effect (ADR-0181, #853 / #852).

Stdlib only. Renders synthetic signals through the `anlmdn` graph the renderer will
write, reads the PCM back, and asserts the numbers the ADR states. Exits non-zero when a
gated number stops holding. Results go to measurements.json beside this file.

Usage: FFMPEG=/path/to/ffmpeg-7.1 python3 check_noise_reduction.py [--fixture PATH] [--out PATH]
The default FFMPEG is the floor build (7.1.x); the numbers were measured on 7.1.5.
"""
import array
import hashlib
import json
import math
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
FFMPEG = os.environ.get("FFMPEG", "ffmpeg")
SR = 48000

# ADR-0181 §2: the renderer's mapping, reduction_db -> anlmdn strength s.
# Calibrated on white noise at -50 dBFS RMS (the reference). Range 5..20 dB.
REF_DB = -50.0
REDUCTION_RANGE = (5.0, 20.0)
PATCH = "0.002"    # anlmdn p, fixed by the renderer (the defaults; ADR-0181 §3)
RESEARCH = "0.006" # anlmdn r
# The fixture's hiss-only stretch: the first ~0.1 s before the cylinder's level rises (ADR-0181 §5).
QUIET = (0.02, 0.10)


def strength(reduction_db):
    return 10 ** (-2.0 + (reduction_db - 5.0) / 32.0)


def af_for(reduction_db):
    return f"anlmdn=s={strength(reduction_db):.6g}:p={PATCH}:r={RESEARCH}:o=o"


def decode(args):
    cmd = [FFMPEG, "-v", "error"] + args + ["-ac", "1", "-f", "f32le", "-ar", str(SR), "-"]
    raw = subprocess.run(cmd, check=True, capture_output=True).stdout
    a = array.array("f")
    a.frombytes(raw)
    return a


def lavfi(expr, af=None):
    args = ["-f", "lavfi", "-i", expr]
    if af:
        args += ["-af", af]
    return decode(args)


def file_pcm(path, af=None):
    args = ["-i", path]
    if af:
        args += ["-af", af]
    return decode(args + ["-map", "0:a:0"])


def rms_db(x):
    s = sum(v * v for v in x) / len(x)
    return 10 * math.log10(s) if s > 0 else -math.inf


def win(x, t0, t1):
    return x[int(t0 * SR):int(t1 * SR)]


def noise(level_db, seconds=3):
    amp = 10 ** (level_db / 20) * math.sqrt(3)  # uniform[-a, a] has RMS a/sqrt(3)
    return f"anoisesrc=d={seconds}:c=white:r={SR}:a={amp:.6g}:seed=7"


def gate(results, name, ok, detail):
    results["gates"].append({"name": name, "ok": bool(ok), "detail": detail})


def main(argv):
    fixture = os.path.join(HERE, "..", "fixtures", "recorded-voice", "recorded-voice.flac")
    out = os.path.join(HERE, "measurements.json")
    if "--fixture" in argv:
        fixture = argv[argv.index("--fixture") + 1]
    if "--out" in argv:
        out = argv[argv.index("--out") + 1]
    res = {"ffmpeg": subprocess.run([FFMPEG, "-version"], capture_output=True, text=True).stdout.splitlines()[0],
           "reference_db": REF_DB, "reduction_range_db": REDUCTION_RANGE,
           "mapping": "s = 10^(-2 + (reduction_db - 5)/32), written with 6 significant digits",
           "gates": []}
    grid = [5.0, 8.0, 10.0, 12.0, 15.0, 18.0, 20.0]

    # 1. Reference: the realised drop on white noise at -50 dBFS, over noise-only 1.0..2.5 s.
    x = lavfi(noise(REF_DB))
    in_db = rms_db(win(x, 1.0, 2.5))
    res["reference_in_dbfs"] = round(in_db, 2)
    ref = []
    for r in grid:
        y = lavfi(noise(REF_DB), af_for(r))
        drop = in_db - rms_db(win(y, 1.0, 2.5))
        ref.append({"reduction_db": r, "s": float(f"{strength(r):.6g}"), "drop_db": round(drop, 2)})
        gate(res, f"reference drop r={r}", abs(drop - r) <= 1.0, f"drop {drop:.2f} dB, target {r}, tolerance 1.0")
    res["reference"] = ref

    # 2. Level law: the same s on noise louder or quieter than the reference (recorded, not gated).
    law = []
    for lvl in [-40.0, -45.0, -55.0, -60.0]:
        xl = lavfi(noise(lvl))
        il = rms_db(win(xl, 1.0, 2.5))
        row = {"noise_dbfs": lvl, "drops": {}}
        for r in [10.0, 20.0]:
            yl = lavfi(noise(lvl), af_for(r))
            row["drops"][str(r)] = round(il - rms_db(win(yl, 1.0, 2.5)), 2)
        law.append(row)
    res["level_law"] = law

    # 3. Speech-level preservation: a 1 kHz tone at -23 dBFS RMS over white noise at -50 dBFS.
    tone = "aevalsrc=0.1*sin(2*PI*1000*t):s=48000:d=3"
    mixed = f"[0:a][1:a]amix=inputs=2:normalize=0"
    def tone_mix(af=None):
        chain = mixed if af is None else f"[0:a]{af}[n];[n][1:a]amix=inputs=2:normalize=0"
        cmd = [FFMPEG, "-v", "error", "-f", "lavfi", "-i", noise(REF_DB), "-f", "lavfi", "-i", tone,
               "-filter_complex", chain, "-ac", "1", "-f", "f32le", "-ar", str(SR), "-"]
        a = array.array("f")
        a.frombytes(subprocess.run(cmd, check=True, capture_output=True).stdout)
        return a
    t_in = rms_db(win(tone_mix(), 1.0, 2.5))
    tone_rows = []
    for r in grid:
        d = rms_db(win(tone_mix(af_for(r)), 1.0, 2.5)) - t_in
        tone_rows.append({"reduction_db": r, "change_db": round(d, 3)})
        gate(res, f"tone level r={r}", abs(d) <= 0.5, f"change {d:.3f} dB, tolerance 0.5 (provisional)")
    res["tone_preservation"] = tone_rows

    # 4. The fixture (ADR-0173 §3): speech change must hold; the noise drop is recorded, not gated.
    if os.path.exists(fixture):
        fx_in = file_pcm(fixture)
        q_in = rms_db(win(fx_in, *QUIET))
        s_in = rms_db(win(fx_in, 2.0, 18.0))
        fx_rows = []
        for r in grid:
            fy = file_pcm(fixture, af_for(r))
            dq = q_in - rms_db(win(fy, *QUIET))
            ds = rms_db(win(fy, 2.0, 18.0)) - s_in
            fx_rows.append({"reduction_db": r, "quiet_drop_db": round(dq, 2), "speech_change_db": round(ds, 3)})
            gate(res, f"fixture speech r={r}", abs(ds) <= 0.5, f"speech change {ds:.3f} dB, tolerance 0.5 (provisional)")
        pauses = [(11.7, 12.5), (15.7, 16.5)]
        pause_in = [rms_db(win(fx_in, a, b)) for a, b in pauses]
        for row in fx_rows:
            fy = file_pcm(fixture, af_for(row["reduction_db"]))
            row["pause_drop_db"] = [round(pi - rms_db(win(fy, a, b)), 2) for pi, (a, b) in zip(pause_in, pauses)]
        res["fixture"] = {"path": os.path.relpath(fixture, HERE), "quiet_window_s": list(QUIET),
                          "pause_windows_s": [list(p) for p in pauses], "rows": fx_rows}
    else:
        res["fixture"] = {"skipped": "fixture not found"}

    # 5. Latency: a 1 kHz burst starting abruptly at 0.5 s. Raw onset, then onset after
    #    apad..atrim cancels the filter's fixed latency (ADR-0181 §4, #843). Both must be 0 samples.
    burst = "aevalsrc=0.1*sin(2*PI*1000*t)*gt(t\\,0.5):s=48000:d=1.5"
    def onset(a):
        for i, v in enumerate(a):
            if abs(v) > 1e-3:
                return i
        return None
    raw = lavfi(burst)
    proc = lavfi(burst, af_for(10.0))
    comp = lavfi(burst, af_for(10.0) + ",apad=pad_len=384,atrim=start_sample=384")
    o_raw, o_proc, o_comp = onset(raw), onset(proc), onset(comp)
    res["latency"] = {"onset_raw": o_raw, "onset_processed": o_proc, "onset_compensated": o_comp,
                      "latency_samples_at_default_p_r": o_proc - o_raw if o_proc is not None and o_raw is not None else None}
    gate(res, "latency compensated onset", o_comp == o_raw, f"raw {o_raw}, compensated {o_comp}, expected equal")

    # 6. Determinism within one build: three runs, same bytes.
    digests = []
    for _ in range(3):
        y = subprocess.run([FFMPEG, "-v", "error", "-f", "lavfi", "-i", noise(REF_DB), "-af", af_for(10.0),
                            "-ac", "1", "-f", "f32le", "-ar", str(SR), "-"], check=True, capture_output=True).stdout
        digests.append(hashlib.sha256(y).hexdigest()[:16])
    res["determinism_sha256_16"] = digests
    gate(res, "same bytes across three runs", len(set(digests)) == 1, f"digests {digests}")

    failed = [g for g in res["gates"] if not g["ok"]]
    with open(out, "w") as fh:
        json.dump(res, fh, indent=2)
        fh.write("\n")
    for g in res["gates"]:
        print(("ok  " if g["ok"] else "FAIL"), g["name"], "-", g["detail"])
    print(f"{len(res['gates']) - len(failed)}/{len(res['gates'])} gates hold; wrote {out}")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
