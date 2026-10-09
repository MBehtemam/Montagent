#!/usr/bin/env python3
"""PROTOTYPE, throwaway: echo and reverb on the narration, blind A/B and measured checks.
ADR-0143 (latency and tail rule, #843), ADR-0170 (units), ADR-0173 (smoke check).

Usage:
  python3 -I prototype_echo_reverb.py ab    [--ffmpeg PATH] [--seed N]   # writes ab/*.m4a + ab/KEY + measurements-ab.json
  python3 -I prototype_echo_reverb.py check [--ffmpeg PATH] [--ffmpeg2 PATH] [--tag -x]   # writes measurements<tag>.json

Needs ffmpeg and numpy. This is not product code. It builds, by string, the graph a renderer would
build for two candidate members and renders them through the pipeline's settings (48 kHz stereo,
AAC 160k) as a loudness-matched blind A/B against the bypassed source.

The two candidates (parameters in ADR-0170's units):

  echo   {delay_ms, feedback 0..1, mix 0..1}
         taps at k*delay_ms, k = 1..n, tap gain mix * feedback^(k-1); n stops when the gain falls
         under -60 dB or at 8. Rendered by ONE `aecho in_gain=1 out_gain=1`; the dry path stays at
         unity, so `x + mix * (sum of taps)` is exactly a dry/wet crossfade of `x` against `x + taps`.
         mix 0 is the bypass.
  reverb {decay_ms, mix 0..1}
         a generated impulse response, no file: two seeded pink-noise channels (seeds 11 and 12),
         shaped by an exponential envelope that reaches -60 dB at decay_ms, convolved by `afir`
         and mixed against the dry path: (1-mix) * dry + mix * wet. The IR is scaled to a fixed energy
         (K = C / (sigma * sqrt(rate * T / (6 ln 10))), T = decay_ms), afir's own normalisation is off,
         and C is calibrated on this narration; see README.
         NOTE `afir`'s own `dry`/`wet` are an input and an output gain, not a mix: dry=0 is silence.

Every element is cut at `end` after the effect ("tail truncated at end", ADR-0143); the kept-tail
clips pad the source with `tail_ms` of silence before the effect and cut `tail_ms` later, which is
the named departure (a literal `tail_ms`), here only so the owner can hear what the rule costs.
"""
import argparse, json, math, random, re, subprocess, sys, tempfile
from pathlib import Path
import numpy as np

HERE = Path(__file__).resolve().parent
FIX = HERE.parent / "fixtures" / "narration-over-bed"
RATE, MATCH = 48000, -20.0
SPEECH_END_MS = 19425          # silencedetect: the narration's last word ends here; the source ends at `end`
REVERB_WET_C = 0.447            # -7.0 dB: wet-only RMS equals dry RMS on this narration (README), within 0.15 dB over 0.6..2.4 s
PINK_SIGMA = 0.0970             # measured RMS of anoisesrc c=pink a=0.5 (seeds 3, 11, 12 agree to 0.7 %), both builds
AFIR_OPTS = "gtype=none"        # set per build by use(): 7.x also needs irnorm=-1 (its default normalises the IR)


def use(ff):
    """pick the afir options this build needs; call before building a reverb graph for `ff`"""
    global AFIR_OPTS
    h = subprocess.run([ff, "-hide_banner", "-h", "filter=afir"], capture_output=True).stdout.decode()
    AFIR_OPTS = "irnorm=-1:gtype=none" if "irnorm" in h else "gtype=none"
IR_SEEDS = (11, 12)
TOTAL_S = 20

ECHO = {"delay_ms": 300.0, "feedback": 0.5, "mix": 0.5}
REVERB = {"decay_ms": 1200.0, "mix": 0.3}
TAIL_MS_ECHO, TAIL_MS_REVERB = 1500, 1500


def run(ff, args, **kw):
    p = subprocess.run([ff, "-hide_banner", "-nostdin", *args], capture_output=True, **kw)
    if p.returncode:
        sys.exit(f"ffmpeg failed: {' '.join(args)}\n{p.stderr.decode(errors='replace')[-1500:]}")
    return p


def num(x):
    return f"{x:.6g}"


# ----------------------------------------------------------------- the two candidate graphs
def echo_taps(p):
    gains, k = [], 1
    while k <= 8:
        g = p["mix"] * p["feedback"] ** (k - 1)
        if g < 0.001: break
        gains.append((k * p["delay_ms"], g)); k += 1
    return gains


def echo_filter(p):
    """a filter-chain fragment, input `[x]` implied by chaining; '' when it is a bypass"""
    taps = echo_taps(p)
    if not taps: return ""
    return (f"aecho=in_gain=1:out_gain=1:delays={'|'.join(num(d) for d, _ in taps)}"
            f":decays={'|'.join(num(g) for _, g in taps)}")


def ir_gain(p):
    rt = p["decay_ms"] / 1000
    return REVERB_WET_C / (PINK_SIGMA * math.sqrt(RATE * rt / (6 * math.log(10))))


def ir_chain(p):
    rt = p["decay_ms"] / 1000
    k = ir_gain(p)
    outs = []
    for i, seed in enumerate(IR_SEEDS):
        outs.append(f"anoisesrc=d={num(rt)}:c=pink:r={RATE}:a=0.5:seed={seed},"
                    f"aeval=val(0)*{num(k)}*pow(10\\,-3*t/{num(rt)}),aformat=channel_layouts=mono[ir{i}]")
    return ";".join(outs) + ";[ir0][ir1]amerge=inputs=2[ir]"


def reverb_graph(p, inp, out):
    """filter_complex fragment: [inp] -> [out]. Includes the IR sources."""
    m = p["mix"]
    return (f"{ir_chain(p)};[{inp}]asplit=2[rd][rw];[rd]volume={num(1 - m)}[rdv];"
            f"[rw][ir]afir=dry=1:wet=1:{AFIR_OPTS},volume={num(m)}[rwv];"
            f"[rdv][rwv]amix=inputs=2:normalize=0:duration=longest[{out}]")


def element_graph(kind, p, end_ms, tail_ms=0, src_label="0:a", out="el"):
    """the narration element: trim to end, effect, cut at end (+tail_ms if the departure is used)"""
    end_s = end_ms / 1000
    pre = f"[{src_label}]atrim=end={num(end_s)},asetpts=PTS-STARTPTS,aformat=sample_rates={RATE}:channel_layouts=stereo"
    if tail_ms: pre += f",apad=pad_dur={num(tail_ms / 1000)}"
    cut = f"atrim=end={num(end_s + tail_ms / 1000)},asetpts=PTS-STARTPTS"
    if kind == "bypass":
        return f"{pre},{cut}[{out}]"
    if kind == "echo":
        f = echo_filter(p)
        return f"{pre}{',' + f if f else ''},{cut}[{out}]"
    if kind == "reverb":
        return f"{pre}[pre];{reverb_graph(p, 'pre', 'fx')};[fx]{cut}[{out}]"
    raise ValueError(kind)


def mix_graph(kind, p, tail_ms=0, total=TOTAL_S):
    """narration element + bed, amix normalize=0, 20 s (the renderer's chain shape, ADR-0077)"""
    return (element_graph(kind, p, SPEECH_END_MS, tail_ms, "0:a", "el") +
            f";[1:a]aformat=sample_rates={RATE}:channel_layouts=stereo[bed];"
            f"[el][bed]amix=inputs=2:normalize=0:duration=longest,apad=whole_dur={total},atrim=end={total}[m]")


def render_mix(ff, kind, p, dst_wav, tail_ms=0, total=TOTAL_S):
    run(ff, ["-i", str(FIX / "narration.flac"), "-i", str(FIX / "bed.flac"), "-y", "-filter_complex",
             mix_graph(kind, p, tail_ms, total), "-map", "[m]", "-c:a", "pcm_f32le", "-ar", str(RATE), str(dst_wav)])


def meter(ff, path):
    err = run(ff, ["-i", str(path), "-af", "ebur128=peak=true", "-f", "null", "-"]).stderr.decode()
    tail = err[err.rindex("Summary:"):]
    return (float(re.search(r"I:\s+(-?[\d.]+)\s+LUFS", tail).group(1)),
            float(re.search(r"Peak:\s+(-?[\d.]+)\s+dBFS", tail).group(1)))


def pcm(ff, args):
    """float32 stereo array from an ffmpeg invocation that writes the audio to stdout"""
    raw = run(ff, [*args, "-f", "f32le", "-ar", str(RATE), "-ac", "2", "-"]).stdout
    return np.frombuffer(raw, dtype="<f4").reshape(-1, 2)


def db(x): return 20 * math.log10(max(x, 1e-12))


# ----------------------------------------------------------------- ab
def cmd_ab(a):
    ff = a.ffmpeg
    use(ff)
    pairs = [   # (pair name, A label, A (kind, params, tail), B label, B)
        ("echo", ("bypass", {}, 0), ("echo", ECHO, 0)),
        ("reverb", ("bypass", {}, 0), ("reverb", REVERB, 0)),
        ("echo-tail", ("echo", ECHO, 0), ("echo", ECHO, TAIL_MS_ECHO)),
        ("reverb-tail", ("reverb", REVERB, 0), ("reverb", REVERB, TAIL_MS_REVERB)),
    ]
    rng = random.Random(a.seed)
    (HERE / "ab").mkdir(exist_ok=True)
    key, res = {}, {"ffmpeg": run(ff, ["-version"]).stdout.decode().splitlines()[0], "echo": ECHO, "reverb": REVERB,
                    "echo_taps": echo_taps(ECHO), "tail_ms": {"echo": TAIL_MS_ECHO, "reverb": TAIL_MS_REVERB},
                    "source_end_ms": SPEECH_END_MS, "tail_pairs_total_s": 21, "clips": {}}
    with tempfile.TemporaryDirectory() as w:
        w = Path(w)
        for name, A, B in pairs:
            order = [A, B]; rng.shuffle(order)
            for letter, (kind, p, tail) in zip("XY", order):
                label = kind + ("+tail_ms" if tail else "")
                if name.endswith("-tail"): label = "tail-cut" if not tail else f"tail-kept-{tail}ms"
                mix = w / f"{name}-{letter}.wav"
                render_mix(ff, kind, p, mix, tail, 21 if name.endswith("-tail") else TOTAL_S)
                lufs, peak = meter(ff, mix); gain = MATCH - lufs
                dst = HERE / "ab" / f"{name}-{letter}.m4a"
                run(ff, ["-i", str(mix), "-y", "-af", f"volume={gain:.5f}dB", "-c:a", "aac", "-b:a", "160k",
                         "-ar", str(RATE), "-ac", "2", str(dst)])
                dec = w / f"dec-{name}-{letter}.wav"; run(ff, ["-i", str(dst), "-y", str(dec)])
                after, apeak = meter(ff, dec)
                if abs(after - MATCH) > 0.3: sys.exit(f"{name}-{letter}: not loudness-matched ({after})")
                key[f"{name}-{letter}"] = label
                res["clips"][f"{name}-{letter}"] = {"what": label, "lufs_before_match": lufs,
                                                    "peak_dbfs_before_match": peak, "match_gain_db": round(gain, 3),
                                                    "decoded_lufs": after, "decoded_peak_dbfs": apeak}
    (HERE / "ab" / "KEY").write_text(json.dumps(key, indent=1) + "\n")
    (HERE / "measurements-ab.json").write_text(json.dumps(res, indent=1) + "\n")
    print(json.dumps(res["clips"], indent=1))


# ----------------------------------------------------------------- check
CLAIMS = []
def claim(ok, text):
    CLAIMS.append({"ok": bool(ok), "claim": text}); print(("PASS " if ok else "FAIL ") + text)


def impulse_file(at=24000, amp=1.0, seconds=4, burst=0):
    """a raw f32le stereo file: one impulse of height `amp` at sample `at`, or, with burst > 0, `burst`
    samples of a 220 Hz tone (speech-band, since the pink IR's gain depends on the input spectrum) at that amplitude"""
    x = np.zeros((seconds * RATE, 2), dtype="<f4")
    if burst: x[at:at + burst] = (amp * np.sin(2 * np.pi * 220 * np.arange(burst) / RATE))[:, None]
    else: x[at] = amp
    f = tempfile.NamedTemporaryFile(suffix=".f32", delete=False); f.write(x.tobytes()); f.close()
    return ["-f", "f32le", "-ar", str(RATE), "-ac", "2", "-i", f.name]


def impulse_response_through(ff, kind, p, tail_ms=0):
    """the element graph on a lavfi impulse (source = 'end' at 4 s); returns the stereo array"""
    end_ms = 4000
    g = element_graph(kind, p, end_ms, tail_ms, "0:a", "el")
    return pcm(ff, [*impulse_file(), "-filter_complex", g, "-map", "[el]"])


def first_nonzero(x, thr=1e-5):          # -100 dB re the impulse: FFT round-off in afir sits below this
    nz = np.flatnonzero(np.abs(x).max(axis=1) > thr)
    return int(nz[0]) if len(nz) else None


def cmd_check(a):
    ff, ff2 = a.ffmpeg, a.ffmpeg2
    use(ff)
    out = {"ffmpeg": run(ff, ["-version"]).stdout.decode().splitlines()[0]}
    if ff2: out["ffmpeg2"] = run(ff2, ["-version"]).stdout.decode().splitlines()[0]
    filters = run(ff, ["-filters"]).stdout.decode()
    for f in ("aecho", "afir", "anoisesrc", "aeval", "amix", "asplit"):
        claim(re.search(rf"\s{f}\s", filters), f"`{f}` is compiled in ({out['ffmpeg']})")

    # ---- onset latency: an impulse at sample 24000 (ADR-0143: must measure 0)
    lat = {}
    for label, kind, p in (("echo", "echo", ECHO), ("echo mix=1", "echo", {**ECHO, "mix": 1.0}),
                           ("reverb", "reverb", REVERB), ("reverb mix=1", "reverb", {**REVERB, "mix": 1.0}),
                           ("bypass", "bypass", {})):
        x = impulse_response_through(ff, kind, p)
        f0 = first_nonzero(x)
        peak_at = int(np.abs(x).max(axis=1).argmax())
        pre = float(np.abs(x[:24000]).max())
        lat[label] = {"pre_onset_max_abs": pre, "first_nonzero": f0, "onset_error_samples": f0 - 24000, "peak_at": peak_at,
                      "length_samples": len(x), "length_error": len(x) - 4 * RATE}
        claim(f0 == 24000 and len(x) == 4 * RATE, f"onset: {label}: first sample above 1e-5 at +{f0 - 24000}, length {len(x) - 4 * RATE:+d} (cut at end); max before onset {pre:.1e}")
    out["onset_latency"] = lat

    # ---- tail: with the tail kept, energy past the source end; cut: none, length exact
    tail = {}
    for label, kind, p, t in (("echo", "echo", ECHO, TAIL_MS_ECHO), ("reverb", "reverb", REVERB, TAIL_MS_REVERB)):
        # the impulse sits in the last 100 ms of the source, so what rings past `end` is measured
        end_ms = 4000; at = int((end_ms - 100) * RATE / 1000); nb = 100 * RATE // 1000
        g_cut = element_graph(kind, p, end_ms, 0, "0:a", "el"); g_kept = element_graph(kind, p, end_ms, t, "0:a", "el")
        src = impulse_file(at, 0.3, burst=nb)
        cut = pcm(ff, [*src, "-filter_complex", g_cut, "-map", "[el]"])
        kept = pcm(ff, [*src, "-filter_complex", g_kept, "-map", "[el]"])
        past = kept[4 * RATE:]
        e = float(np.sqrt((past ** 2).mean())) if len(past) else 0.0
        inside = kept[at:4 * RATE]
        lost = float((past ** 2).sum()) / max(float((kept ** 2).sum()), 1e-30)
        tail[label] = {"cut_length_error": len(cut) - 4 * RATE, "kept_length_error": len(kept) - (4 * RATE + int(t * RATE / 1000)),
                       "rms_dbfs_past_end": db(e), "peak_past_end": float(np.abs(past).max()) if len(past) else 0,
                       "energy_share_lost_by_cut": lost, "last_sample_abs_cut": float(np.abs(cut[-1]).max()),
                       "cut_vs_kept_prefix_max_abs_diff": float(np.abs(cut - kept[:len(cut)]).max())}
        t_ = tail[label]
        claim(t_["cut_length_error"] == 0, f"tail: {label}: cut render is exactly `end` long")
        claim(t_["kept_length_error"] == 0, f"tail: {label}: kept render is exactly end + tail_ms long")
        claim(t_["rms_dbfs_past_end"] > -60, f"tail: {label}: energy past end with the tail kept (RMS {t_['rms_dbfs_past_end']:.1f} dBFS > -60)")
        claim(t_["cut_vs_kept_prefix_max_abs_diff"] < 1e-6, f"tail: {label}: the cut render is the kept render's prefix (max diff {t_['cut_vs_kept_prefix_max_abs_diff']:.1e} < 1e-6)")
    out["tail"] = tail

    # ---- smoke check on the narration mix (PCM float, before encode)
    smoke = {}
    with tempfile.TemporaryDirectory() as w:
        w = Path(w)
        render_mix(ff, "bypass", {}, w / "b.wav")
        b = pcm(ff, ["-i", str(w / "b.wav")]); lb, pb = meter(ff, w / "b.wav")
        for label, kind, p in (("echo", "echo", ECHO), ("reverb", "reverb", REVERB)):
            render_mix(ff, kind, p, w / f"{label}.wav")
            x = pcm(ff, ["-i", str(w / f"{label}.wav")]); lx, px = meter(ff, w / f"{label}.wav")
            n = min(len(b), len(x))
            diff = db(float(np.sqrt(((x[:n] - b[:n]) ** 2).mean())))
            rms_b = db(float(np.sqrt((b[:n] ** 2).mean())))
            s = {"length_samples": len(x), "finite": bool(np.isfinite(x).all()), "peak_abs": float(np.abs(x).max()),
                 "lufs": lx, "lufs_bypass": lb, "lufs_delta": lx - lb, "true_peak_ish_dbfs": px, "peak_dbfs_bypass": pb,
                 "diff_rms_dbfs": diff, "diff_vs_bypass_rms_db": diff - rms_b,
                 "last_10ms_rms_dbfs": db(float(np.sqrt((x[-480:] ** 2).mean())))}
            smoke[label] = s
            claim(len(x) == TOTAL_S * RATE, f"smoke: {label}: length is exactly {TOTAL_S} s")
            claim(s["finite"], f"smoke: {label}: no NaN/Inf")
            claim(s["peak_abs"] < 1.0, f"smoke: {label}: no clipping (peak {s['peak_abs']:.3f} < 1.0)")
            claim(lx > -70, f"smoke: {label}: above the -70 LUFS gate ({lx:.1f})")
            claim(s["diff_vs_bypass_rms_db"] > -30, f"smoke: {label}: not a no-op (difference {s['diff_vs_bypass_rms_db']:.1f} dB re bypass RMS > -30)")
            claim(abs(s["lufs_delta"]) <= 3.0, f"smoke: {label}: loudness within 3 LU of bypass ({s['lufs_delta']:+.2f})")
        # an enabled:false / mix 0 member is the bypass: byte-identical PCM (ADR-0173 §5)
        for label, kind, p in (("echo mix=0", "echo", {**ECHO, "mix": 0.0}),):
            render_mix(ff, kind, p, w / "z.wav")
            claim(pcm(ff, ["-i", str(w / "z.wav")]).tobytes() == b.tobytes(), f"bypass: {label} is byte-identical to the bypass render")
        # reverb mix=0: the filter still runs; it is a numerical bypass, not necessarily bytes
        render_mix(ff, "reverb", {**REVERB, "mix": 0.0}, w / "z.wav")
        zr = pcm(ff, ["-i", str(w / "z.wav")])
        smoke["reverb mix=0 max abs diff vs bypass"] = float(np.abs(zr - b).max())
        claim(np.array_equal(zr, b), "bypass: reverb mix=0 is byte-identical to the bypass render "
                                     f"(max abs diff {smoke['reverb mix=0 max abs diff vs bypass']:.2e})")
    out["smoke"] = smoke

    # ---- determinism: reruns, scalar-vs-SIMD, and a second build
    drift = {}
    def whole(binary, kind, p, extra=()):
        use(binary)
        with tempfile.TemporaryDirectory() as w:
            render_mix_flags(binary, kind, p, Path(w) / "o.wav", extra)
            return pcm(binary, [*extra, "-i", str(Path(w) / "o.wav")])
    for label, kind, p in (("echo", "echo", ECHO), ("reverb", "reverb", REVERB)):
        a1 = whole(ff, kind, p); a2 = whole(ff, kind, p)
        d = {"rerun_identical": bool(np.array_equal(a1, a2))}
        claim(d["rerun_identical"], f"determinism: {label}: byte-identical rerun on one build")
        a0 = whole(ff, kind, p, ("-cpuflags", "0"))
        d["scalar_vs_simd_identical"] = bool(np.array_equal(a1, a0))
        d["scalar_vs_simd_max_abs_diff"] = float(np.abs(a1 - a0).max())
        print(f"info  {label}: -cpuflags 0 vs default identical={d['scalar_vs_simd_identical']} maxdiff={d['scalar_vs_simd_max_abs_diff']:.3e}")
        if ff2:
            c = whole(ff2, kind, p)
            n = min(len(a1), len(c))
            d["second_build_identical"] = bool(len(a1) == len(c) and np.array_equal(a1[:n], c[:n]))
            d["second_build_length"] = len(c)
            d["second_build_max_abs_diff"] = float(np.abs(a1[:n] - c[:n]).max())
            nzd = np.flatnonzero(np.abs(a1[:n] - c[:n]).max(axis=1) > 0)
            d["second_build_first_differing_sample"] = int(nzd[0]) if len(nzd) else None
            d["element_end_sample"] = SPEECH_END_MS * RATE // 1000
            m_ = SPEECH_END_MS * RATE // 1000 - 8192
            d["second_build_max_abs_diff_before_element_end_minus_8192"] = float(np.abs(a1[:m_] - c[:m_]).max())
            d["second_build_diff_rms_db_re_signal"] = db(float(np.sqrt(((a1[:n] - c[:n]) ** 2).mean()))) - db(float(np.sqrt((a1[:n] ** 2).mean())))
            print(f"info  {label}: second build: first differing sample {d['second_build_first_differing_sample']} (element ends {d['element_end_sample']}); max diff before that point minus 8192: {d['second_build_max_abs_diff_before_element_end_minus_8192']:.3e}")
            print(f"info  {label}: second build identical={d['second_build_identical']} maxdiff={d['second_build_max_abs_diff']:.3e} "
                  f"diff {d['second_build_diff_rms_db_re_signal']:.1f} dB re signal")
        drift[label] = d
    # the IR alone, both builds (what `anoisesrc seed` + `aeval pow` produce)
    ir_args = ["-filter_complex", ir_chain(REVERB), "-map", "[ir]"]
    ir1 = pcm(ff, ir_args); ir2 = pcm(ff, ir_args)
    drift["ir"] = {"rerun_identical": bool(np.array_equal(ir1, ir2)), "scalar_vs_simd_identical": bool(np.array_equal(ir1, pcm(ff, ["-cpuflags", "0", *ir_args])))}
    if ff2:
        ir3 = pcm(ff2, ir_args)
        drift["ir"]["second_build_identical"] = bool(ir1.shape == ir3.shape and np.array_equal(ir1, ir3))
        drift["ir"]["second_build_max_abs_diff"] = float(np.abs(ir1 - ir3).max()) if ir1.shape == ir3.shape else None
    claim(drift["ir"]["rerun_identical"], "determinism: the seeded generated IR is byte-identical on rerun")
    out["drift"] = drift

    out["claims"] = CLAIMS
    (HERE / f"measurements{a.tag}.json").write_text(json.dumps(out, indent=1, default=float) + "\n")
    bad = [c for c in CLAIMS if not c["ok"]]
    print(f"\n{len(CLAIMS) - len(bad)} / {len(CLAIMS)} claims hold")
    sys.exit(1 if bad else 0)


def render_mix_flags(ff, kind, p, dst, extra=()):
    run(ff, [*extra, "-i", str(FIX / "narration.flac"), "-i", str(FIX / "bed.flac"), "-y", "-filter_complex",
             mix_graph(kind, p), "-map", "[m]", "-c:a", "pcm_f32le", "-ar", str(RATE), str(dst)])


if __name__ == "__main__":
    ap = argparse.ArgumentParser(); ap.add_argument("mode", choices=["ab", "check"])
    ap.add_argument("--ffmpeg", default="ffmpeg"); ap.add_argument("--ffmpeg2", default=None)
    ap.add_argument("--seed", type=int, default=7950); ap.add_argument("--tag", default="")
    a = ap.parse_args()
    {"ab": cmd_ab, "check": cmd_check}[a.mode](a)
