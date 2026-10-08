#!/usr/bin/env python3
"""Evidence for ADR-0180: the compressor's static curve and the limiter's ceiling on the real filters.

Renders lavfi sine tones through the exact filter strings the renderer will emit and
measures them in double precision (the tone is converted to dbl BEFORE the filter; see
eq/check_eq_biquads.py for why). Exits non-zero when a number stops holding. Stdlib only.
ffmpeg is taken from $FFMPEG or PATH. Writes measurements-check.json beside itself.

Level convention under test: threshold_db and every compressor level are RMS dBFS
(detection=rms), so a full-scale sine is -3.01 dB.
"""
import array, json, math, os, subprocess, sys

FFMPEG = os.environ.get("FFMPEG", "ffmpeg")
SR = 48000
FAILS, ROWS = [], []


def lin(db):
    return 10 ** (db / 20)


def render(src, chain):
    af = "aformat=sample_fmts=dbl," + (chain + "," if chain else "") + "aformat=sample_fmts=dbl"
    raw = subprocess.run([FFMPEG, "-hide_banner", "-loglevel", "error", "-f", "lavfi", "-i", src,
                          "-af", af, "-f", "f64le", "-"], check=True, capture_output=True).stdout
    a = array.array("d")
    a.frombytes(raw)
    return a


def tone(peak_db, dur):
    """A 1 kHz sine of exactly this peak. lavfi's own `sine` source is fixed at 1/8 amplitude."""
    return f"aevalsrc='{lin(peak_db):.9f}*sin(2*PI*1000*t)':s={SR}:d={dur}"


def rms_db(x):
    return 10 * math.log10(sum(v * v for v in x) / len(x) + 1e-300)


def compressor(thr, ratio, att, rel, makeup):
    return (f"acompressor=threshold={lin(thr):.9f}:ratio={ratio}:attack={att}:release={rel}"
            f":makeup={lin(makeup):.9f}:knee=1:link=average:detection=rms")


def check(name, got, want, tol, two_sided=True):
    ok = abs(got - want) <= tol if two_sided else got <= want + tol
    ROWS.append({"check": name, "got": round(got, 7), "want": want, "tol": tol, "ok": ok})
    print(("ok  " if ok else "FAIL"), f"{name}: {got:.4f} (want {'' if two_sided else '<= '}{want} +/- {tol})")
    if not ok:
        FAILS.append(name)


# 1. Static transfer on steady sines. in_rms is the tone's own RMS dBFS.
# The formula is exact only at the reference timing attack_ms = release_ms = 100: ffmpeg's
# envelope follower ripples on a sine, and faster timings follow the ripple and reduce deeper.
# measured on 7.1.5: 100/100 reads the formula to 0.001 dB; 5/50 is 1.6 dB deeper; 5/1000 is 2.2 dB.
REF_ATT, REF_REL = 100, 100
STEP = 0.3  # dB
for thr, ratio, makeup in ((-20, 4, 0), (-20, 4, 6), (-30, 8, 0), (-20, 1, 0), (-12, 2, 3)):
    for in_peak_db in (-50, -40, -30, -20, -12, -6, 0):
        in_rms = in_peak_db - 3.0103
        x = render(tone(in_peak_db, 3), compressor(thr, ratio, REF_ATT, REF_REL, makeup))
        out_rms = rms_db(x[len(x) * 2 // 3:])  # the last second, after the envelope settles
        want = in_rms + makeup if in_rms <= thr else thr + (in_rms - thr) / ratio + makeup
        check(f"transfer thr={thr} ratio={ratio} makeup={makeup} in_rms={in_rms:.1f}", out_rms, round(want, 3), STEP)

# 1b. The timing sweep: the realised curve is never shallower than the formula (+0.3 dB), and at
# most 2.5 dB deeper over the whole authorable-in-practice span. This is the property the ADR states.
for att, rel in ((1, 10), (5, 50), (5, 120), (20, 250), (50, 120), (100, 100), (200, 1000), (5, 1000)):
    x = render(tone(-6, 4), compressor(-20, 4, att, rel, 0))
    in_rms = -6 - 3.0103
    diff = rms_db(x[len(x) * 3 // 4:]) - (-20 + (in_rms + 20) / 4)
    ok = -2.5 <= diff <= 0.3
    ROWS.append({"check": f"timing sweep attack={att} release={rel}: realised minus formula (dB)", "got": round(diff, 3), "want": "[-2.5, +0.3]", "ok": ok})
    print(("ok  " if ok else "FAIL"), f"timing sweep attack={att} release={rel}: realised minus formula {diff:+.3f} dB (want within [-2.5, +0.3])")
    if not ok:
        FAILS.append(f"timing sweep {att}/{rel}")

# 2. Time constants: a step -30 -> -6 dB peak. Gain reduction is read from 2 ms RMS windows.
def step_reduction(att, rel):
    src = (f"aevalsrc='if(lt(t,1),{lin(-30)},{lin(-6)})*sin(2*PI*1000*t)':s={SR}:d=2")
    y = render(src, compressor(-24, 8, att, rel, 0))
    win = int(SR * 0.002)
    out = []
    for i in range(0, len(y) - win, win):
        out.append((i / SR, rms_db(y[i:i + win])))
    return out


for att in (5, 50):
    env = step_reduction(att, 250)
    # level just after the step vs the settled level at the end of the burst
    hot = [(t, l) for t, l in env if 1.0 <= t < 2.0]
    peak_level, final_level = hot[0][1], hot[-1][1]
    target = peak_level - 0.63 * (peak_level - final_level)
    t63 = next(t for t, l in hot if l <= target) - 1.0
    ROWS.append({"check": f"attack_ms={att}: time to 63% of the reduction (s)", "got": round(t63, 4)})
    print(f"info attack_ms={att}: 63% of the gain reduction reached after {t63 * 1000:.1f} ms")

t5 = [r for r in ROWS if r["check"].startswith("attack_ms=5:")][0]["got"]
t50 = [r for r in ROWS if r["check"].startswith("attack_ms=50:")][0]["got"]
ok = t50 > 2 * t5
ROWS.append({"check": "attack_ms 50 is more than twice as slow as 5", "got": [t5, t50], "ok": ok})
print(("ok  " if ok else "FAIL"), f"attack 50 ms ({t50 * 1000:.1f} ms) is more than 2x slower than attack 5 ms ({t5 * 1000:.1f} ms)")
if not ok:
    FAILS.append("attack ordering")

# 3. Limiter: hold the ceiling at sample peak, move no onset.
for ceiling in (-3.0, -12.0, -20.0, -24.0):
    chain = f"alimiter=level_in=1:level_out=1:limit={lin(ceiling):.9f}:attack=5:release=50:asc=0:level=0:latency=1"
    x = render(tone(6, 2), chain)
    peak = 20 * math.log10(max(abs(v) for v in x[SR // 2:]))
    check(f"limiter ceiling {ceiling:+.1f} dB sample peak (+6 dB sine)", peak, ceiling, 0.0002, two_sided=True)
    quiet = render(tone(ceiling - 6, 2), chain)
    ref = render(tone(ceiling - 6, 2), "")
    check(f"limiter passes a tone 6 dB under ceiling {ceiling:+.1f} (gain, dB)", rms_db(quiet[SR // 2:]) - rms_db(ref[SR // 2:]), 0.0, 0.01)

    # onset: 100 ms of silence, then a 0 dBFS burst. First sample over 1e-4 must not move.
    burst = f"aevalsrc='if(lt(t,0.1),0,{lin(6)}*sin(2*PI*1000*t))':s={SR}:d=0.5"
    a, b = render(burst, ""), render(burst, chain)
    ia = next(i for i, v in enumerate(a) if abs(v) > 1e-4)
    ib = next(i for i, v in enumerate(b) if abs(v) > 1e-4)
    check(f"limiter onset shift at ceiling {ceiling:+.1f} (samples)", ib - ia, 0, 0)

# Range edges: every bound of every range must be accepted by ffmpeg (a document that passes
# `validate` must never fail at render). The limiter floor is alimiter's own: limit >= 0.0625.
def accepts(chain):
    r = subprocess.run([FFMPEG, "-hide_banner", "-loglevel", "error", "-f", "lavfi", "-i", "anoisesrc=d=0.2",
                        "-af", chain, "-f", "null", "-"], capture_output=True, text=True)
    return r.returncode == 0

edges = [compressor(t, r, a, rl, m) for t, r, a, rl, m in ((-60, 20, 0.1, 1, 24), (0, 1, 2000, 9000, 0), (-60, 1, 0.1, 9000, 24))]
edges += [f"alimiter=limit={lin(c):.9f}:attack=5:release={rl}:asc=0:level=0:latency=1" for c in (-24, 0) for rl in (1, 1000)]
for c in edges:
    ok = accepts(c)
    ROWS.append({"check": "range edge accepted: " + c[:70], "ok": ok})
    print(("ok  " if ok else "FAIL"), "range edge accepted:", c[:70])
    if not ok:
        FAILS.append("range edge " + c)
bad = accepts(f"alimiter=limit={lin(-30):.9f}:attack=5:release=50:asc=0:level=0:latency=1")
ROWS.append({"check": "NEGATIVE CONTROL: a -30 dB limiter ceiling is rejected by alimiter", "ok": not bad})
print(("ok  " if not bad else "FAIL"), "-30 dB ceiling is rejected by alimiter (why the floor is -24)")
if bad:
    FAILS.append("alimiter now accepts -30: the floor can be revisited")

json.dump({"ffmpeg": subprocess.run([FFMPEG, "-version"], capture_output=True, text=True).stdout.splitlines()[0],
           "rows": ROWS, "pass": not FAILS}, open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "measurements-check.json"), "w"), indent=1)
if FAILS:
    print("\nFAILED:", FAILS)
    sys.exit(1)
print("\nall dynamics numbers hold")
