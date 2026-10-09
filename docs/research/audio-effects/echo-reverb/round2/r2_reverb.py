#!/usr/bin/env python3
"""PROTOTYPE, throwaway: reverb round 2, three blind candidates, objective figures.

Imports (does not edit) ../prototype_echo_reverb.py for the fixture, the runner, the meter and the
afir option switch. Usage (needs ffmpeg and numpy):

  python3 -I r2_reverb.py calibrate [--ffmpeg PATH]      # prints the wet-level literals (CALIB) and IR facts
  python3 -I r2_reverb.py ab        [--ffmpeg PATH]      # writes ab/r2-*.m4a, KEY.md, measurements-round2.json (after `check`)
  python3 -I r2_reverb.py check     [--ffmpeg PATH] [--others P1,P2,..]

Conventions (all literal, closed vocabulary, deterministic; no IR file):
  out = dry + mix * K * wet      ("send" style; dry stays at unity, mix 0 is the bypass)
  K is a literal calibrated so that, at mix 1, the wet-only RMS equals the dry RMS on the narration.
Two constructions of `wet`:
  ir   : generated impulse response through `afir`: pre-delay, early reflection taps (aevalsrc),
         two-band noise tail (low band decays over decay_ms, high band over decay_ms * hf_ratio),
         high band low-passed, exponential decay, decorrelated L/R seeds. No irnorm/gtype traps are
         left to the spectrum: K is a literal measured on the narration.
  taps : no afir. Per channel one `aecho` holding early taps + six feedback-comb banks written out as
         exponentially decaying taps (aecho cannot recurse), minus the dry copy aecho passes through,
         then two short `aecho` diffusers and a low-pass. All positive gains (aecho rejects negative
         ones), so a true Schroeder all-pass is not expressible; `acomb` does not exist in ffmpeg.
"""
import argparse, hashlib, importlib.util, json, math, random, re, subprocess, sys, tempfile
from pathlib import Path

sys.dont_write_bytecode = True
import numpy as np

HERE = Path(__file__).resolve().parent
_spec = importlib.util.spec_from_file_location("proto", HERE.parent / "prototype_echo_reverb.py")
P = importlib.util.module_from_spec(_spec); _spec.loader.exec_module(P)
run, meter, pcm, db, num = P.run, P.meter, P.pcm, P.db, P.num
RATE, FIX, END_MS, TOTAL_S, MATCH = P.RATE, P.FIX, P.SPEECH_END_MS, P.TOTAL_S, P.MATCH

# ------------------------------------------------------------------ candidates (the sealed part)
COMMON_IR = {"construction": "ir", "predelay_ms": 20.0, "split_hz": 1500, "hf_lowpass_hz": 7500, "hf_ratio": 0.4,
             "hf_level": 1.8, "lf_highpass_hz": 120, "er_lowpass_hz": 4500,
             "seeds_lo": (21, 22), "seeds_hi": (31, 32),
             "er_ms_l": (7, 14, 23, 34, 47, 62), "er_ms_r": (9, 17, 26, 38, 52, 68),
             "er_gain": (2.2, 1.9, 1.6, 1.3, 1.1, 0.9)}
CANDS = {
    "ir-2200": {**COMMON_IR, "decay_ms": 2200.0, "mix": 0.55},
    "ir-3000": {**COMMON_IR, "decay_ms": 3000.0, "mix": 0.70, "predelay_ms": 30.0},
    "taps-2200": {"construction": "taps", "decay_ms": 2200.0, "mix": 0.55, "predelay_ms": 20.0, "lowpass_hz": 5000,
                  "comb_ms_l": (29.7, 37.1, 41.1, 43.7, 53.9, 61.3), "comb_ms_r": (30.9, 36.3, 42.3, 46.1, 52.7, 59.1),
                  "comb_gain": 0.35,
                  "er_ms_l": (7, 14, 23, 34, 47, 62), "er_ms_r": (9, 17, 26, 38, 52, 68),
                  "er_gain": (0.55, 0.5, 0.42, 0.36, 0.3, 0.25),
                  "diff1_l": ((2.3, 5.1, 8.9, 13.7), (0.45, 0.35, 0.28, 0.2)),
                  "diff2_l": ((4.1, 9.7, 16.3, 23.9), (0.4, 0.3, 0.22, 0.15)),
                  "diff1_r": ((2.9, 5.9, 9.7, 14.9), (0.45, 0.35, 0.28, 0.2)),
                  "diff2_r": ((4.7, 10.9, 17.9, 25.3), (0.4, 0.3, 0.22, 0.15))},
}
# literal wet gains K from `calibrate` (wet-only RMS = dry RMS on the narration, ffmpeg 7.0.2)
CALIB = {"ir-2200": 0.0498, "ir-3000": 0.0428, "taps-2200": 0.4447}
ROUND1 = {"decay_ms": 1200.0, "mix": 0.3}   # the round-1 reverb, kept only as a yardstick in the figures


def join(xs): return "|".join(num(x) for x in xs)


# ------------------------------------------------------------------ the IR (kind ir)
def ir_chain(c, k=1.0):
    T = c["decay_ms"] / 1000; pd = c["predelay_ms"] / 1000
    n_er = lambda ms: [int(round((c["predelay_ms"] + m) * RATE / 1000)) for m in ms]
    env = (f"if(gte(t\\,{num(pd)})\\,min(1\\,(t-{num(pd)})/0.004)\\,0)")
    outs = []
    for i in (0, 1):
        er = n_er(c["er_ms_l"] if i == 0 else c["er_ms_r"])
        er_expr = "+".join(f"{num(g)}*eq(n\\,{n})" for g, n in zip(c["er_gain"], er))
        outs.append(
            f"anoisesrc=d={num(T)}:c=pink:r={RATE}:a=0.5:seed={c['seeds_lo'][i]},"
            f"highpass=f={c['lf_highpass_hz']},lowpass=f={c['split_hz']},"
            f"aeval=val(0)*{env}*pow(10\\,-3*(t-{num(pd)})/{num(T)}),aformat=channel_layouts=mono[lo{i}];"
            f"anoisesrc=d={num(T)}:c=pink:r={RATE}:a=0.5:seed={c['seeds_hi'][i]},"
            f"highpass=f={c['split_hz']},lowpass=f={c['hf_lowpass_hz']},"
            f"aeval=val(0)*{num(c['hf_level'])}*{env}*pow(10\\,-3*(t-{num(pd)})/{num(T * c['hf_ratio'])}),aformat=channel_layouts=mono[hi{i}];"
            f"aevalsrc={er_expr}:s={RATE}:d={num(T)},lowpass=f={c['er_lowpass_hz']},aformat=channel_layouts=mono[er{i}];"
            f"[lo{i}][hi{i}][er{i}]amix=inputs=3:normalize=0:duration=longest[ch{i}]")
    return ";".join(outs) + f";[ch0][ch1]amerge=inputs=2,volume={num(k)}[ir]"


def wet_ir(c, k, inp, out):
    return f"{ir_chain(c, k)};[{inp}][ir]afir=dry=1:wet=1:{P.AFIR_OPTS}[{out}]"


# ------------------------------------------------------------------ the tap bank (kind taps)
def comb_taps(c, side):
    T = c["decay_ms"]; pd = c["predelay_ms"]
    ms = c["er_ms_" + side]; taps = [(pd + m, g) for m, g in zip(ms, c["er_gain"])]
    for d in c["comb_ms_" + side]:
        g = 10 ** (-3 * d / T)
        k = 1
        while k * d <= T:
            taps.append((pd + k * d, c["comb_gain"] * g ** k)); k += 1
    return sorted(taps)


def wet_taps(c, k, inp, out):
    parts = [f"[{inp}]channelsplit=channel_layout=stereo[sl][sr]"]
    for side, s in (("l", "sl"), ("r", "sr")):
        t = comb_taps(c, side)
        d1, d2 = c["diff1_" + side], c["diff2_" + side]
        parts.append(
            f"[{s}]asplit=2[{s}a][{s}b];[{s}a]aecho=in_gain=1:out_gain=1:delays={join([d for d, _ in t])}:decays={join([g for _, g in t])}[{s}e];"
            f"[{s}b]volume=-1[{s}n];[{s}e][{s}n]amix=inputs=2:normalize=0:duration=longest,"
            f"aecho=in_gain=1:out_gain=1:delays={join(d1[0])}:decays={join(d1[1])},"
            f"aecho=in_gain=1:out_gain=1:delays={join(d2[0])}:decays={join(d2[1])},"
            f"lowpass=f={c['lowpass_hz']},volume={num(k)}[w{side}]")
    parts.append(f"[wl][wr]amerge=inputs=2[{out}]")
    return ";".join(parts)


def wet_graph(c, k, inp, out):
    return (wet_ir if c["construction"] == "ir" else wet_taps)(c, k, inp, out)


def element(c, k, src="0:a", out="el", wet_only=False):
    E = num(END_MS / 1000)
    pre = f"[{src}]atrim=end={E},asetpts=PTS-STARTPTS,aformat=sample_rates={RATE}:channel_layouts=stereo"
    cut = f"atrim=end={E},asetpts=PTS-STARTPTS"
    if c is None:
        return f"{pre},{cut}[{out}]"
    if wet_only:
        return f"{pre}[pre];{wet_graph(c, k, 'pre', 'w')};[w]{cut}[{out}]"
    return (f"{pre},asplit=2[d][w0];{wet_graph(c, k, 'w0', 'w')};[w]volume={num(c['mix'])}[wv];"
            f"[d][wv]amix=inputs=2:normalize=0:duration=longest,{cut}[{out}]")


def mix_graph(c, k, total=TOTAL_S):
    return (element(c, k) + f";[1:a]aformat=sample_rates={RATE}:channel_layouts=stereo[bed];"
            f"[el][bed]amix=inputs=2:normalize=0:duration=longest,apad=whole_dur={total},atrim=end={total}[m]")


def render_mix(ff, c, k, dst, extra=()):
    P.use(ff)
    run(ff, [*extra, "-i", str(FIX / "narration.flac"), "-i", str(FIX / "bed.flac"), "-y", "-filter_complex",
             mix_graph(c, k), "-map", "[m]", "-c:a", "pcm_f32le", "-ar", str(RATE), str(dst)])


def render_wet_narration(ff, c, k):
    P.use(ff)
    return pcm(ff, ["-i", str(FIX / "narration.flac"), "-filter_complex", element(c, k, wet_only=True), "-map", "[el]"])


def dry_narration(ff):
    return pcm(ff, ["-i", str(FIX / "narration.flac"), "-filter_complex", element(None, 1), "-map", "[el]"])


def rms(x): return float(np.sqrt((x.astype(np.float64) ** 2).mean()))


# ------------------------------------------------------------------ calibrate
def cmd_calibrate(a):
    ff = a.ffmpeg; P.use(ff)
    dry = dry_narration(ff)
    out = {}
    for name, c in CANDS.items():
        w = render_wet_narration(ff, c, 1.0)
        n = min(len(w), len(dry))
        r = rms(dry[:n]) / rms(w[:n])
        out[name] = round(r, 4)
        print(f"{name}: wet-only RMS at K=1 is {db(rms(w[:n]))-db(rms(dry[:n])):+.2f} dB re dry -> K = {r:.4f}")
    print("CALIB =", out)
    # IR facts: early/total energy for the ir candidates
    for name, c in CANDS.items():
        if c["construction"] != "ir": continue
        ir = pcm(ff, ["-filter_complex", ir_chain(c), "-map", "[ir]"])
        e = (ir.astype(np.float64) ** 2).sum(axis=1)
        k_ = int((c["predelay_ms"] + 90) * RATE / 1000)
        print(f"{name}: IR length {len(ir)} samples, energy in first {c['predelay_ms']+90:.0f} ms = {e[:k_].sum()/e.sum():.1%}, peak {np.abs(ir).max():.3f}")


# ------------------------------------------------------------------ objective figures
def impulse_wet(ff, c, k, secs=9, at=24000, end_ms=8500):
    """wet-only response (full chain minus bypass) to one impulse at sample `at`; also the full response"""
    x = np.zeros((secs * RATE, 2), dtype="<f4"); x[at] = 1.0
    f = tempfile.NamedTemporaryFile(suffix=".f32", delete=False); f.write(x.tobytes()); f.close()
    src = ["-f", "f32le", "-ar", str(RATE), "-ac", "2", "-i", f.name]
    P.use(ff)
    E = num(end_ms / 1000)
    g = (f"[0:a]atrim=end={E},asetpts=PTS-STARTPTS,aformat=sample_rates={RATE}:channel_layouts=stereo[pre];"
         f"{wet_graph(c, k, 'pre', 'w')};[w]atrim=end={E},asetpts=PTS-STARTPTS[el]")
    w = pcm(ff, [*src, "-filter_complex", g, "-map", "[el]"])
    Path(f.name).unlink()
    return w


def band(x, lo, hi):
    X = np.fft.rfft(x, axis=0); fr = np.fft.rfftfreq(len(x), 1 / RATE)
    X[(fr < lo) | (fr >= hi)] = 0
    return np.fft.irfft(X, n=len(x), axis=0)


def decay_figures(w, t0):
    """Schroeder backward integration of the wet response (both channels summed) from sample t0"""
    e = (w[t0:].astype(np.float64) ** 2).sum(axis=1)
    edc = np.cumsum(e[::-1])[::-1]; edc_db = 10 * np.log10(np.maximum(edc / edc[0], 1e-30))
    t = np.arange(len(edc_db)) / RATE
    def rt(lo, hi):
        m = (edc_db <= lo) & (edc_db >= hi)
        if m.sum() < 50: return None
        s = np.polyfit(t[m], edc_db[m], 1)[0]
        return round(-60 / s, 3) if s < 0 else None
    t60 = np.flatnonzero(edc_db <= -60)
    return {"EDT_s": (lambda m: round(-60 / np.polyfit(t[m], edc_db[m], 1)[0], 3))((edc_db <= 0) & (edc_db >= -10)),
            "T20_s": rt(-5, -25), "T30_s": rt(-5, -35),
            "time_to_minus60dB_s": round(float(t[t60[0]]), 3) if len(t60) else None}


def objective(ff, c, k):
    w = impulse_wet(ff, c, k)
    at = 24000
    first = int(np.flatnonzero(np.abs(w).max(axis=1) > 1e-5)[0])
    full = decay_figures(w, first)
    bands = {}
    for lo, hi in ((100, 500), (500, 2000), (2000, 8000)):
        bands[f"{lo}-{hi}Hz"] = decay_figures(band(w, lo, hi), first)["T30_s"]
    # wet energy in the first 80 ms after the first wet sample (early part) vs total
    e = (w.astype(np.float64) ** 2).sum(axis=1)
    early = e[first:first + int(0.08 * RATE)].sum() / e[first:].sum()
    return {"wet_first_sample_after_impulse": first - at, "wet_first_sample_after_impulse_ms": round((first - at) * 1000 / RATE, 2),
            "decay_full_band": full, "T30_by_band_s": bands, "early_80ms_energy_share": round(float(early), 4)}


def band_ratio(ff, c, k):
    """wet (at the candidate's mix) vs dry energy, by band, on the narration, dB"""
    dry = dry_narration(ff).astype(np.float64); w = render_wet_narration(ff, c, k).astype(np.float64)[:len(dry)] * c["mix"]
    dry = dry[:len(w)]; res = {}
    for lo, hi in ((100, 500), (500, 2000), (2000, 8000), (8000, 16000)):
        res[f"{lo}-{hi}Hz"] = round(db(rms(band(w, lo, hi))) - db(rms(band(dry, lo, hi))), 2)
    res["all"] = round(db(rms(w)) - db(rms(dry)), 2)
    return res


def r1_band_ratio(ff):
    P.use(ff)
    dry = dry_narration(ff).astype(np.float64)
    g = P.element_graph("reverb", ROUND1, END_MS, 0, "0:a", "el")
    x = pcm(ff, ["-i", str(FIX / "narration.flac"), "-filter_complex", g, "-map", "[el]"]).astype(np.float64)
    n = min(len(x), len(dry)); w = x[:n] - dry[:n]; dry = dry[:n]; res = {}
    for lo, hi in ((100, 500), (500, 2000), (2000, 8000), (8000, 16000)):
        res[f"{lo}-{hi}Hz"] = round(db(rms(band(w, lo, hi))) - db(rms(band(dry, lo, hi))), 2)
    res["all"] = round(db(rms(w)) - db(rms(dry)), 2)
    return res


def r1_impulse(ff):
    P.use(ff)
    x = np.zeros((9 * RATE, 2), dtype="<f4"); x[24000] = 1.0
    f = tempfile.NamedTemporaryFile(suffix=".f32", delete=False); f.write(x.tobytes()); f.close()
    src = ["-f", "f32le", "-ar", str(RATE), "-ac", "2", "-i", f.name]
    g = P.element_graph("reverb", {**ROUND1, "mix": 1.0}, 8500, 0, "0:a", "el")
    full = pcm(ff, [*src, "-filter_complex", g, "-map", "[el]"])
    byp = pcm(ff, [*src, "-filter_complex", P.element_graph("bypass", {}, 8500, 0, "0:a", "el"), "-map", "[el]"])
    Path(f.name).unlink()
    w = full[:len(byp)].astype(np.float64) - byp[:len(full)]
    first = 24000 + 1
    out = decay_figures(w, 24001)
    out["T30_by_band_s"] = {f"{lo}-{hi}Hz": decay_figures(band(w, lo, hi), 24001)["T30_s"] for lo, hi in ((100, 500), (500, 2000), (2000, 8000))}
    return out


# ------------------------------------------------------------------ check
CLAIMS = []
def claim(ok, text):
    CLAIMS.append({"ok": bool(ok), "claim": text}); print(("PASS " if ok else "FAIL ") + text)


def nm(ff): return run(ff, ["-version"]).stdout.decode().splitlines()[0]


def cmd_check(a):
    ff = a.ffmpeg; others = [o for o in (a.others or "").split(",") if o]
    P.use(ff)
    out = {"ffmpeg": nm(ff), "others": [nm(o) for o in others], "candidates": {}}
    letters = assign()
    for name, L in letters.items():
        c = CANDS[name]; k = CALIB[name]; r = {"name": name, "params": jsonable(c), "wet_gain_K": k}
        # ---- onset latency: impulse at sample 24000 through the element (dry+wet), must be 0 samples late, exact length
        x = np.zeros((4 * RATE, 2), dtype="<f4"); x[24000] = 1.0
        f = tempfile.NamedTemporaryFile(suffix=".f32", delete=False); f.write(x.tobytes()); f.close()
        src = ["-f", "f32le", "-ar", str(RATE), "-ac", "2", "-i", f.name]
        P.use(ff)
        y = pcm(ff, [*src, "-filter_complex", element(c, k, "0:a", "el").replace(num(END_MS / 1000), "4"), "-map", "[el]"])
        Path(f.name).unlink()
        nz = np.flatnonzero(np.abs(y).max(axis=1) > 1e-5)
        r["onset"] = {"first_sample_above_1e-5": int(nz[0]), "error_samples": int(nz[0]) - 24000, "length_error": len(y) - 4 * RATE,
                      "max_abs_before_onset": float(np.abs(y[:24000]).max())}
        claim(r["onset"]["error_samples"] == 0 and r["onset"]["length_error"] == 0,
              f"onset: {L}: dry onset +{r['onset']['error_samples']} samples, length {r['onset']['length_error']:+d}")
        r["objective"] = objective(ff, c, k)
        claim(r["objective"]["wet_first_sample_after_impulse_ms"] >= c["predelay_ms"] - 0.1,
              f"onset: {L}: no wet energy before the literal pre-delay (first wet sample > 1e-5 at +{r['objective']['wet_first_sample_after_impulse_ms']} ms, pre-delay {c['predelay_ms']} ms)")
        r["narration_band_ratio_db"] = band_ratio(ff, c, k)
        # ---- smoke on the narration mix
        with tempfile.TemporaryDirectory() as w_:
            w_ = Path(w_)
            P.render_mix(ff, "bypass", {}, w_ / "b.wav"); b = pcm(ff, ["-i", str(w_ / "b.wav")]); lb, pb = meter(ff, w_ / "b.wav")
            render_mix(ff, c, k, w_ / "x.wav"); xx = pcm(ff, ["-i", str(w_ / "x.wav")]); lx, px = meter(ff, w_ / "x.wav")
            n = min(len(b), len(xx)); diff = db(rms(xx[:n] - b[:n])) - db(rms(b[:n]))
            r["smoke"] = {"length_samples": len(xx), "finite": bool(np.isfinite(xx).all()), "peak_abs": float(np.abs(xx).max()),
                          "lufs": lx, "lufs_bypass": lb, "lufs_delta": round(lx - lb, 2), "diff_vs_bypass_rms_db": round(diff, 2),
                          "last_10ms_rms_dbfs": db(rms(xx[-480:]))}
            s = r["smoke"]
            claim(s["length_samples"] == TOTAL_S * RATE, f"smoke: {L}: length exactly {TOTAL_S} s")
            claim(s["finite"], f"smoke: {L}: no NaN/Inf")
            claim(s["peak_abs"] < 1.0, f"smoke: {L}: no clipping before the match gain (peak {s['peak_abs']:.3f})")
            claim(s["lufs"] > -70, f"smoke: {L}: above the -70 LUFS gate ({s['lufs']:.1f})")
            claim(s["diff_vs_bypass_rms_db"] > -30, f"smoke: {L}: not a no-op ({s['diff_vs_bypass_rms_db']:.1f} dB re bypass)")
            claim(abs(s["lufs_delta"]) <= 3.0, f"smoke: {L}: loudness within 3 LU of bypass ({s['lufs_delta']:+.2f})")
            # mix 0 => bypass
            z = {**c, "mix": 0.0}; render_mix(ff, z, k, w_ / "z.wav"); zz = pcm(ff, ["-i", str(w_ / "z.wav")])
            r["mix0_max_abs_diff_vs_bypass"] = float(np.abs(zz[:n] - b[:n]).max())
            claim(r["mix0_max_abs_diff_vs_bypass"] == 0.0, f"bypass: {L}: mix 0 is byte-identical to the bypass (max diff {r['mix0_max_abs_diff_vs_bypass']:.1e})")
            # ---- determinism
            def whole(binary, extra=()):
                render_mix(binary, c, k, w_ / "o.wav", extra); P.use(ff)
                return pcm(ff, [*extra, "-i", str(w_ / "o.wav")])
            a1, a2 = whole(ff), whole(ff)
            d = {"rerun_identical": bool(np.array_equal(a1, a2))}
            claim(d["rerun_identical"], f"determinism: {L}: byte-identical rerun")
            a0 = whole(ff, ("-cpuflags", "0"))
            d["scalar_vs_simd_identical"] = bool(np.array_equal(a1, a0)); d["scalar_vs_simd_max_abs_diff"] = float(np.abs(a1 - a0).max())
            d["builds"] = {}
            for o in others:
                c3 = whole(o); P.use(ff)
                n3 = min(len(a1), len(c3))
                dd = np.abs(a1[:n3].astype(np.float64) - c3[:n3])
                nzd = np.flatnonzero(dd.max(axis=1) > 0)
                d["builds"][nm(o)] = {"length_samples": len(c3), "identical": bool(len(a1) == len(c3) and not dd.any()),
                                      "max_abs_diff": float(dd.max()),
                                      "diff_rms_db_re_signal": round(db(rms(a1[:n3] - c3[:n3])) - db(rms(a1[:n3])), 1),
                                      "first_differing_sample": int(nzd[0]) if len(nzd) else None}
            r["drift"] = d
        out["candidates"][L] = r
    out["yardstick_round1_reverb_1200ms_mix0.3"] = {"narration_band_ratio_db": r1_band_ratio(ff), "decay": r1_impulse(ff)}
    out["letters"] = letters
    out["claims"] = CLAIMS
    (HERE / "_check.json").write_text(json.dumps(out, indent=1, default=float) + "\n")
    bad = [c for c in CLAIMS if not c["ok"]]
    print(f"\n{len(CLAIMS)-len(bad)} / {len(CLAIMS)} claims hold")


def jsonable(c): return json.loads(json.dumps(c))


def assign(seed=20262):
    names = list(CANDS); rng = random.Random(seed); rng.shuffle(names)
    return {n: f"r2-{l}" for n, l in zip(names, "ABC")}


# ------------------------------------------------------------------ ab
def cmd_ab(a):
    ff = a.ffmpeg; P.use(ff)
    letters = assign(); rng = random.Random(a.seed)
    (HERE / "ab").mkdir(exist_ok=True)
    key = {}
    with tempfile.TemporaryDirectory() as w:
        w = Path(w)
        for name, L in sorted(letters.items(), key=lambda kv: kv[1]):
            c, k = CANDS[name], CALIB[name]
            order = [("bypass", None), ("candidate", c)]; rng.shuffle(order)
            for xy, (kind, cc) in zip("XY", order):
                mix = w / f"{L}-{xy}.wav"
                render_mix(ff, cc, k, mix) if cc else P.render_mix(ff, "bypass", {}, mix)
                lufs, peak = meter(ff, mix); gain = MATCH - lufs
                dst = HERE / "ab" / f"{L}-{xy}.m4a"
                run(ff, ["-i", str(mix), "-y", "-af", f"volume={gain:.5f}dB", "-c:a", "aac", "-b:a", "160k", "-ar", str(RATE), "-ac", "2", str(dst)])
                dec = w / f"dec-{L}-{xy}.wav"; run(ff, ["-i", str(dst), "-y", str(dec)])
                after, apeak = meter(ff, dec)
                if abs(after - MATCH) > 0.3: sys.exit(f"{L}-{xy}: not loudness-matched ({after})")
                key[f"{L}-{xy}"] = {"what": "bypass" if cc is None else name, "lufs_before_match": lufs, "peak_dbfs_before_match": peak,
                                    "match_gain_db": round(gain, 3), "decoded_lufs": after, "decoded_peak_dbfs": apeak}
    (HERE / "_ab.json").write_text(json.dumps({"letters": letters, "clips": key}, indent=1) + "\n")
    print(json.dumps(key, indent=1))


# ------------------------------------------------------------------ seal: KEY.md + measurements-round2.json
def cmd_seal(a):
    chk = json.loads((HERE / "_check.json").read_text()); ab = json.loads((HERE / "_ab.json").read_text())
    letters = ab["letters"]; inv = {v: k for k, v in letters.items()}
    out = {"note": "sealed: opening this reveals which letter is which candidate. Open after judging.", "ffmpeg": chk["ffmpeg"],
           "other_builds": chk["others"], "letters": letters, "ab_clips": ab["clips"], "candidates": chk["candidates"],
           "yardstick_round1_reverb_1200ms_mix0.3": chk["yardstick_round1_reverb_1200ms_mix0.3"], "claims": chk["claims"]}
    (HERE / "measurements-round2.json").write_text(json.dumps(out, indent=1, default=float) + "\n")
    (HERE / "_check.json").unlink(); (HERE / "_ab.json").unlink()
    L = lambda l: chk["candidates"][l]
    def fmt(x): return "n/a" if x is None else f"{x:.2f}"
    rows = []
    for l in ("r2-A", "r2-B", "r2-C"):
        r = L(l); c = r["params"]; o = r["objective"]; d = o["decay_full_band"]; b = o["T30_by_band_s"]
        rows.append((l, r["name"], c, r, o, d, b))
    md = ["# KEY (round 2 reverb). Open only after judging.", "",
          f"Built with {chk['ffmpeg']}. Other builds compared: " + "; ".join(chk["others"]) + ".", "",
          "## Which clip is which", "", "| Clip | Is |", "|---|---|"]
    for k, v in ab["clips"].items():
        md.append(f"| `{k}.m4a` | {'bypass (the unprocessed source)' if v['what']=='bypass' else 'candidate ' + k[:4] + ' (' + v['what'] + ')'} |")
    md += ["", "The bypass clip is the same audio in all three pairs.", "",
           "## The candidates (all literal, seeded, no IR file; `out = dry + mix * K * wet`, dry at unity, mix 0 is the bypass)", "",
           "| Letter | Construction | decay_ms | mix | pre-delay ms | wet gain K | other |", "|---|---|---|---|---|---|---|"]
    for l, nme, c, r, o, d, b in rows:
        if c["construction"] == "ir":
            oth = (f"generated IR through `afir`; low band pink {c['lf_highpass_hz']}..{c['split_hz']} Hz decays over decay_ms; high band pink "
                   f"{c['split_hz']}..{c['hf_lowpass_hz']} Hz at x{c['hf_level']} decays over decay_ms x {c['hf_ratio']}; 4 ms attack ramp; 6 early-reflection taps per channel "
                   f"(L {list(c['er_ms_l'])} ms, R {list(c['er_ms_r'])} ms after the pre-delay, gains {list(c['er_gain'])}) low-passed at {c['er_lowpass_hz']} Hz; "
                   f"seeds low {list(c['seeds_lo'])}, high {list(c['seeds_hi'])}")
            cons = "afir IR"
        else:
            oth = (f"no afir. Per channel one `aecho` of 6 early taps (gains {list(c['er_gain'])}) plus 6 feedback combs written as decaying taps "
                   f"(L {list(c['comb_ms_l'])} ms, R {list(c['comb_ms_r'])} ms, tap k at pre-delay + k x d, gain {c['comb_gain']} x 10^(-3 k d / decay_ms)), "
                   f"minus the dry copy aecho passes, two 4-tap diffusion `aecho`s, low-pass {c['lowpass_hz']} Hz")
            cons = "aecho comb/diffusion bank"
        md.append(f"| {l} | {cons} | {c['decay_ms']:g} | {c['mix']} | {c['predelay_ms']:g} | {r['wet_gain_K']} | {oth} |")
    md += ["", "Round-1 reverb for scale (the one the owner could not tell from bypass): afir pink-noise IR, decay_ms 1200, mix 0.3 as a dry/wet crossfade, "
           "no pre-delay, no early taps, one band.", "",
           "## Objective energy decay (wet-only impulse response of the real graph, Schroeder backward integration, both channels)", "",
           "RT60-like = slope of the decay curve extrapolated to -60 dB. T30 fits -5..-35 dB, T20 fits -5..-25 dB, EDT fits 0..-10 dB. "
           "Compare with the literal decay_ms. Band T30s come from FFT-masked bands; the 100-500 Hz one is unreliable (few modes, "
           "the fit is noisy and runs long), read it as a rough figure.", "",
           "| Letter | decay_ms | EDT s | T20 s | T30 s | first -60 dB s | T30 100-500 Hz | T30 500-2k Hz | T30 2k-8k Hz | wet energy in first 80 ms |", "|---|---|---|---|---|---|---|---|---|---|"]
    for l, nme, c, r, o, d, b in rows:
        md.append(f"| {l} | {c['decay_ms']:g} | {fmt(d['EDT_s'])} | {fmt(d['T20_s'])} | {fmt(d['T30_s'])} | {fmt(d['time_to_minus60dB_s'])} | "
                  f"{fmt(b['100-500Hz'])} | {fmt(b['500-2000Hz'])} | {fmt(b['2000-8000Hz'])} | {o['early_80ms_energy_share']:.0%} |")
    y = chk["yardstick_round1_reverb_1200ms_mix0.3"]; yd = y["decay"]
    md.append(f"| round 1 | 1200 | {fmt(yd['EDT_s'])} | {fmt(yd['T20_s'])} | {fmt(yd['T30_s'])} | {fmt(yd['time_to_minus60dB_s'])} | "
              f"{fmt(yd['T30_by_band_s']['100-500Hz'])} | {fmt(yd['T30_by_band_s']['500-2000Hz'])} | {fmt(yd['T30_by_band_s']['2000-8000Hz'])} | n/a |")
    md += ["", "## Wet level against dry, by band, on the narration (wet at its mix, dB re the dry in that band)", "",
           "The round-1 complaint was 'higher noise'. The high bands show how much wet energy there is up there.", "",
           "| Letter | 100-500 | 500-2k | 2k-8k | 8k-16k | all |", "|---|---|---|---|---|---|"]
    for l, nme, c, r, o, d, b in rows:
        q = r["narration_band_ratio_db"]; md.append(f"| {l} | {q['100-500Hz']} | {q['500-2000Hz']} | {q['2000-8000Hz']} | {q['8000-16000Hz']} | {q['all']} |")
    q = y["narration_band_ratio_db"]; md.append(f"| round 1 | {q['100-500Hz']} | {q['500-2000Hz']} | {q['2000-8000Hz']} | {q['8000-16000Hz']} | {q['all']} |")
    md += ["", "## Onset latency, smoke, determinism (ffmpeg 7.0.2 unless stated)", "",
           "| Letter | dry onset error (samples) | length error | wet first sample (ms after impulse; literal pre-delay) | peak before match gain | LUFS delta vs bypass | mix 0 vs bypass | rerun identical | scalar = SIMD |", "|---|---|---|---|---|---|---|---|---|"]
    for l, nme, c, r, o, d, b in rows:
        md.append(f"| {l} | {r['onset']['error_samples']} | {r['onset']['length_error']} | {o['wet_first_sample_after_impulse_ms']} ({c['predelay_ms']:g}) | "
                  f"{r['smoke']['peak_abs']:.3f} | {r['smoke']['lufs_delta']:+.2f} | max diff {r['mix0_max_abs_diff_vs_bypass']:.0e} | {r['drift']['rerun_identical']} | "
                  f"{r['drift']['scalar_vs_simd_identical']} (max {r['drift']['scalar_vs_simd_max_abs_diff']:.1e}) |")
    md += ["", f"{len([c for c in chk['claims'] if c['ok']])} of {len(chk['claims'])} claims hold (list in `measurements-round2.json`).", "",
           "## Drift between ffmpeg builds (whole 20 s mix, float PCM, against the 7.0.2 render)", "",
           "| Letter | build | identical | max abs diff | diff RMS re signal | first differing sample |", "|---|---|---|---|---|---|"]
    for l, nme, c, r, o, d, b in rows:
        for bn, v in r["drift"]["builds"].items():
            md.append(f"| {l} | {bn.split(' Copyright')[0]} | {v['identical']} | {v['max_abs_diff']:.3g} | {v['diff_rms_db_re_signal']} dB | {v['first_differing_sample']} |")
    md += ["", "The narration element ends at sample 932400 (19.425 s). A first difference near 927744 is the `afir` end-of-source difference round 1 found on 6.1.1, "
           "now in the wet path only. The playwright build (ffmpeg 7.0.1) lacks filters this graph needs and was left out.", ""]
    (HERE / "KEY.md").write_text("\n".join(md))
    print("KEY.md sha256", hashlib.sha256((HERE / "KEY.md").read_bytes()).hexdigest())


if __name__ == "__main__":
    ap = argparse.ArgumentParser(); ap.add_argument("mode", choices=["calibrate", "ab", "check", "seal"])
    ap.add_argument("--ffmpeg", default="ffmpeg"); ap.add_argument("--others", default=""); ap.add_argument("--seed", type=int, default=7951)
    a = ap.parse_args()
    {"calibrate": cmd_calibrate, "ab": cmd_ab, "check": cmd_check, "seal": cmd_seal}[a.mode](a)
