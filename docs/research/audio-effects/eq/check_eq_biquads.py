#!/usr/bin/env python3
"""Evidence for ADR-0179: the EQ members' numbers hold on the real filters.

Renders lavfi sine tones through the exact filter strings the renderer will
emit (highpass/lowpass as Butterworth biquad cascades, shelf, bell), measures
the RMS of the processed tone against the unprocessed one, and exits non-zero
when a number stops holding. Stdlib only. ffmpeg is taken from $FFMPEG or PATH.

The tone is converted to double BEFORE the filter: lavfi's sine is s16, and a biquad
run in s16 quantises the stopband to 2^-15 (measured: a 48 dB/oct low-pass read -2978 dB,
i.e. exact zero). The renderer's graph is float, and the ADR's check must be too.

Why Butterworth sections: cascading identical Q=0.7071 biquads reads -3.01 dB
only at 12 dB/oct; the cascade of k equal sections is -3.01*k dB at the cutoff.
The negative control below proves it, so the section Qs are not decoration.
"""
import array, json, math, os, subprocess, sys

FFMPEG = os.environ.get("FFMPEG", "ffmpeg")
SR = 48000
DUR = 1.0


def butterworth_qs(order):
    """Q of each 2-pole section of an even-order Butterworth filter."""
    return [1 / (2 * math.sin((2 * k - 1) * math.pi / (2 * order))) for k in range(1, order // 2 + 1)]


def cascade(kind, fc, slope):
    order = {12: 2, 24: 4, 48: 8}[slope]
    return ",".join(
        f"{kind}=f={fc}:poles=2:width_type=q:width={q:.6f}" for q in butterworth_qs(order)
    )


def naive(kind, fc, slope):
    n = {12: 1, 24: 2, 48: 4}[slope]
    return ",".join(f"{kind}=f={fc}:poles=2:width_type=q:width=0.707107" for _ in range(n))


def rms_db(freq, chain):
    af = "aformat=sample_fmts=dbl," + (chain + "," if chain else "") + "aformat=sample_fmts=dbl"
    cmd = [FFMPEG, "-hide_banner", "-loglevel", "error", "-f", "lavfi", "-i",
           f"sine=f={freq}:r={SR}:d={DUR}", "-af", af, "-f", "f64le", "-"]
    raw = subprocess.run(cmd, check=True, capture_output=True).stdout
    a = array.array("d")
    a.frombytes(raw)
    tail = a[len(a) // 2:]  # settle first
    return 10 * math.log10(sum(x * x for x in tail) / len(tail) + 1e-300)


def rel(freq, chain):
    return rms_db(freq, chain) - rms_db(freq, "")


FAILS, OUT = [], {"ffmpeg": subprocess.run([FFMPEG, "-version"], capture_output=True, text=True).stdout.splitlines()[0], "rows": []}


def check(name, got, want, tol, two_sided=True):
    ok = abs(got - want) <= tol if two_sided else got <= want + tol
    OUT["rows"].append({"check": name, "got": round(got, 3), "want": want, "tol": tol, "ok": ok})
    print(("ok  " if ok else "FAIL"), f"{name}: {got:.3f} dB (want {'' if two_sided else '<= '}{want} +/- {tol})")
    if not ok:
        FAILS.append(name)


TOL = 0.15  # the meter here is a double-precision RMS; 0.15 dB is >10x its noise
for kind, near, far in (("highpass", lambda f: f / 4, 4), ("lowpass", lambda f: f * 4, 4)):
    for slope in (12, 24, 48):
        fc = 1000
        check(f"{kind} {slope} at fc", rel(fc, cascade(kind, fc, slope)), -3.01, TOL)
        stop = near(fc)
        theory = -10 * math.log10(1 + 16 ** (slope // 6))  # Butterworth order n = slope/6, ratio 4 -> (4^2)^n
        # One-sided: "at least this much attenuation". Measured reads differ from the ideal by
        # up to 1.5 dB (bilinear warping) and the 8th-order high-pass floors near -78 dB, so
        # the bound is the ideal plus 1.5 dB, never asked for more than -75 dB.
        check(f"{kind} {slope} two octaves into stopband (at least)", rel(stop, cascade(kind, fc, slope)), max(theory + 1.5, -75.0), 0.0, two_sided=False)
        check(f"{kind} {slope} passband", rel(fc * 4 if kind == "highpass" else fc / 4, cascade(kind, fc, slope)), 0.0, 0.3)

# negative control: identical sections do not hold -3.01 at 24/48
for slope in (24, 48):
    got = rel(1000, naive("highpass", 1000, slope))
    OUT["rows"].append({"check": f"NEGATIVE CONTROL naive cascade {slope}", "got": round(got, 3), "must_differ_from": -3.01})
    print(f"neg  naive {slope} dB/oct cascade reads {got:.3f} dB at fc (must not be -3.01)")
    if abs(got + 3.01) < 0.5:
        FAILS.append("negative control no longer discriminates")

for g in (6, -6, 12, -12):
    chain = f"equalizer=f=1000:width_type=q:width=1:g={g}"
    check(f"bell {g:+d} dB at centre", rel(1000, chain), g, TOL)
    check(f"bell {g:+d} dB three octaves away", rel(8000, chain), 0.0, 0.5)
for side, f, plateau, away in (("lowshelf", 200, 40, 5000), ("highshelf", 5000, 15000, 200)):
    for g in (6, -6):
        chain = f"{side}=f={f}:g={g}:width_type=q:width=0.707107"
        check(f"{side} {g:+d} plateau", rel(plateau, chain), g, 0.3)
        check(f"{side} {g:+d} far side", rel(away, chain), 0.0, 0.3)
        check(f"{side} {g:+d} at corner (half the gain)", rel(f, chain), g / 2, 0.3)

# Range edges: every bound of every range must be accepted by ffmpeg (a document that passes
# `validate` must never fail at render).
def accepts(chain):
    r = subprocess.run([FFMPEG, "-hide_banner", "-loglevel", "error", "-f", "lavfi", "-i", "anoisesrc=d=0.2:r=48000",
                        "-af", "aformat=sample_fmts=fltp," + chain, "-f", "null", "-"], capture_output=True, text=True)
    return r.returncode == 0

for f in (20, 20000):
    chains = [cascade("highpass", f, 48), cascade("lowpass", f, 48)]
    for g in (-24, 24):
        chains += [f"lowshelf=f={f}:g={g}:width_type=q:width=0.707107", f"highshelf=f={f}:g={g}:width_type=q:width=0.707107"]
        chains += [f"equalizer=f={f}:width_type=q:width={q}:g={g}" for q in (0.1, 10)]
    for c in chains:
        ok = accepts(c)
        OUT["rows"].append({"check": "range edge accepted: " + c[:70], "ok": ok})
        if not ok:
            FAILS.append("range edge " + c)
print("range edges: all accepted" if not [x for x in FAILS if x.startswith("range edge")] else "range edges FAILED")

OUT["pass"] = not FAILS
json.dump(OUT, open(os.path.join(os.path.dirname(__file__), "measurements-check.json"), "w"), indent=1)
if FAILS:
    print("\nFAILED:", FAILS)
    sys.exit(1)
print("\nall EQ numbers hold")
