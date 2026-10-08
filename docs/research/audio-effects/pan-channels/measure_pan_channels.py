#!/usr/bin/env python3
"""Measurements for the pan/balance and channel-operations ADR (map #795).

Usage:  python3 -I measure_pan_channels.py [path/to/ffmpeg] [--json out.json] [--dump-pcm dir]

Every number is read from f64le PCM taken straight off the filtergraph (no encoder),
from lavfi signals built here. Exits 1 and names each claim that does not hold.
Needs numpy. --dump-pcm writes the PCM of a fixed set of renders so two builds can be
compared byte for byte (see "build drift" in README.md).
"""
import hashlib, json, math, os, subprocess, sys
import numpy as np

args = [a for a in sys.argv[1:]]
OUTJSON = DUMP = None
if "--json" in args:
    i = args.index("--json"); OUTJSON = args[i + 1]; del args[i:i + 2]
if "--dump-pcm" in args:
    i = args.index("--dump-pcm"); DUMP = args[i + 1]; del args[i:i + 2]
FF = args[0] if args else "ffmpeg"
R = 48000
FAIL, ROWS = [], {}
VERSION = subprocess.run([FF, "-version"], capture_output=True, text=True).stdout.splitlines()[0]


def render(src, af, ch=2):
    """src is a lavfi graph string; af is the graph under test. Returns (n, ch) float64."""
    cmd = [FF, "-hide_banner", "-loglevel", "error", "-f", "lavfi", "-i", src + ",aformat=sample_fmts=dbl",
           "-af", "aformat=sample_fmts=dbl" + ("," + af if af else ""), "-f", "f64le", "-"]
    raw = subprocess.run(cmd, check=True, capture_output=True).stdout
    a = np.frombuffer(raw, dtype="<f8")
    # channel count of the output is whatever the graph produced; ask via ch
    return a.reshape(-1, ch)


def sine(f, d=1.0, amp=0.5):
    return f"sine=f={f}:r={R}:d={d}:a={amp}" if False else f"sine=f={f}:r={R}:d={d}"


def stereo(fl, fr, d=1.0):
    return f"{sine(fl,d)}[l];{sine(fr,d)}[r];[l][r]amerge=inputs=2"


def mono(f=1000, d=1.0):
    return sine(f, d)


def db(x):
    return 20 * math.log10(max(x, 1e-12))


def rms(x):
    x = x[len(x) // 4:]  # skip nothing-special settle; tones are steady
    return float(np.sqrt(np.mean(x * x)))


def lvl(a):  # per-channel RMS in dB
    return [db(rms(a[:, c])) for c in range(a.shape[1])]


def check(name, got, want, tol, kind="two"):
    ok = abs(got - want) <= tol if kind == "two" else (got <= want + tol if kind == "le" else got >= want - tol)
    print(("ok    " if ok else "FAIL  ") + f"{name}: {got:+.4f} (want {want:+.4f} +/- {tol})")
    ROWS[name] = {"got": round(got, 5), "want": want, "tol": tol}
    if not ok:
        FAIL.append(name)


def exact(name, ok):
    print(("ok    " if ok else "FAIL  ") + name)
    ROWS[name] = bool(ok)
    if not ok:
        FAIL.append(name)


print(VERSION)
ST = stereo(1000, 1000)  # L = R = 1 kHz, the shape of a mono source after the bus's aformat
REF = lvl(render(ST, ""))[0]

# ---- 0. what the bus does to a mono source today ------------------------------------------
m = render(mono(), "aformat=channel_layouts=stereo")
check("0 mono source -> stereo bus (aformat upmix), per-channel dB re mono", lvl(m)[0] - db(rms(render(mono(), "", 1))), -3.0103, 0.01)

# ---- 1. pan laws on the stereo bus (L = R source) ------------------------------------------
# gains as (gL, gR) for pan p in -1..1
def law_balance_linear(p):   return (min(1, 1 - p), min(1, 1 + p))                      # 0 dB centre, opposite channel falls linearly
def law_balance_cos(p):      return (math.cos(max(p, 0) * math.pi / 2), math.cos(max(-p, 0) * math.pi / 2))  # 0 dB centre, opposite falls on a quarter cosine
def law_cp3(p):              t = (p + 1) * math.pi / 4; return (math.cos(t), math.sin(t))   # -3 dB centre, constant power
def law_lin6(p):             return ((1 - p) / 2, (1 + p) / 2)                          # -6 dB centre, constant sum

LAWS = {"balance_linear(0dB)": law_balance_linear, "balance_cos(0dB)": law_balance_cos,
        "constant_power(-3dB)": law_cp3, "linear(-6dB)": law_lin6}
PS = [-1, -0.5, 0, 0.5, 1]
print("\n# total power (L+R, dB re an unpanned stereo L=R source) and per-channel gain, by law and p")
for name, law in LAWS.items():
    row = []
    for p in PS:
        gl, gr = law(p)
        a = render(ST, f"pan=stereo|c0={gl:.9f}*c0|c1={gr:.9f}*c1")
        l = lvl(a)
        tot = 10 * math.log10(10 ** (l[0] / 10) + 10 ** (l[1] / 10)) - 10 * math.log10(2 * 10 ** (REF / 10))
        row.append({"p": p, "L_dB": round(l[0] - REF, 3), "R_dB": round(l[1] - REF, 3), "total_dB": round(tot, 3)})
        check(f"1 {name} p={p} left gain", l[0] - REF, db(gl), 0.01) if gl > 1e-9 else None
        check(f"1 {name} p={p} right gain", l[1] - REF, db(gr), 0.01) if gr > 1e-9 else None
    ROWS[f"law {name}"] = row
    print(f"{name:22s}", "  ".join(f"p={r['p']:+.1f}:L{r['L_dB']:+.1f}/R{r['R_dB']:+.1f}/T{r['total_dB']:+.1f}" for r in row))

# ---- 2. balance vs pan on a stereo source with different content in each channel -----------
# L = 440 Hz, R = 880 Hz. Balance only scales; a pan matrix would mix L into R.
D = stereo(440, 880)
base = render(D, "")
bal = render(D, "pan=stereo|c0=0*c0|c1=1*c1")                      # balance +1 (hard right)
check("2 balance +1: left channel silent (peak dB)", db(float(np.max(np.abs(bal[:, 0])))), -240.0, 0.0, "le") if False else None
exact("2 balance +1: left channel exactly zero", float(np.max(np.abs(bal[:, 0]))) == 0.0)
exact("2 balance +1: right channel bit-identical to source right", np.array_equal(bal[:, 1], base[:, 1]))
mix = render(D, "pan=stereo|c0=0*c0|c1=0.707106781*c0+0.707106781*c1")  # constant-power fold of both into R
# 440 Hz appears in R only when the matrix mixes; balance never creates it.
def tone_db(x, f):
    s = np.fft.rfft(x[:R] * np.hanning(R)); return db(abs(s[f]) / (R / 4))
check("2 balance +1: 440 Hz present in right channel (dB, must be tiny)", tone_db(bal[:, 1], 440), -120.0, 0.0, "le") if False else None
exact("2 balance never moves L content into R (440 Hz in R < -100 dB)", tone_db(bal[:, 1], 440) < -100)
exact("2 a mixing pan matrix does put 440 Hz in R (> -40 dB)", tone_db(mix[:, 1], 440) > -40)

# ---- 3. mono downmix level ------------------------------------------------------------------
NOISE = "anoisesrc=r=48000:d=2:c=white:seed=1:a=0.3[l];anoisesrc=r=48000:d=2:c=white:seed=2:a=0.3[r];[l][r]amerge=inputs=2"
ANTI = f"{sine(1000)}[l];{sine(1000)},volume=-1[r];[l][r]amerge=inputs=2"
LEFT_ONLY = f"{sine(1000)}[l];{sine(1000)},volume=0[r];[l][r]amerge=inputs=2"
MONO = {
    "half_sum 0.5(L+R)": "pan=stereo|c0=0.5*c0+0.5*c1|c1=0.5*c0+0.5*c1",
    "root_half 0.707(L+R)": "pan=stereo|c0=0.7071067812*c0+0.7071067812*c1|c1=0.7071067812*c0+0.7071067812*c1",
    "sum (L+R)": "pan=stereo|c0=c0+c1|c1=c0+c1",
}
print("\n# mono fold, dB per output channel relative to the stereo source's per-channel level")
ref = {}
for sname, src in (("correlated L=R", ST), ("uncorrelated noise", NOISE), ("anti-phase L=-R", ANTI), ("left only", LEFT_ONLY)):
    ref[sname] = lvl(render(src, ""))[0]  # the live channel (left)
for mname, af in MONO.items():
    out = {}
    for sname, src in (("correlated L=R", ST), ("uncorrelated noise", NOISE), ("anti-phase L=-R", ANTI), ("left only", LEFT_ONLY)):
        a = render(src, af)
        out[sname] = round(lvl(a)[0] - ref[sname], 3) if rms(a[:, 0]) > 1e-9 else "silent"
    ROWS[f"mono {mname}"] = out
    print(f"{mname:22s}", out)
# '<' normalises the gains (divides by the sum when it exceeds 1); '=' is literal. Measured, not documented behaviour assumed.
n1 = render(ST, "pan=stereo|c0<c0+c1|c1<c0+c1"); n2 = render(ST, "pan=stereo|c0=c0+c1|c1=c0+c1")
check("3 pan '<' with c0+c1 on L=R (normalised: unity)", lvl(n1)[0] - REF, 0.0, 0.01)
check("3 pan '=' with c0+c1 on L=R (literal: +6.02 dB)", lvl(n2)[0] - REF, 6.0206, 0.01)
check("3 half_sum on correlated L=R is 0 dB", ROWS["mono half_sum 0.5(L+R)"]["correlated L=R"], 0.0, 0.01)
check("3 half_sum on uncorrelated noise is -3 dB", ROWS["mono half_sum 0.5(L+R)"]["uncorrelated noise"], -3.0, 0.15)
check("3 half_sum on left-only is -6.02 dB", ROWS["mono half_sum 0.5(L+R)"]["left only"], -6.0206, 0.01)
check("3 root_half on correlated is +3.01 dB", ROWS["mono root_half 0.707(L+R)"]["correlated L=R"], 3.0103, 0.01)
exact("3 half_sum on anti-phase is exact silence", ROWS["mono half_sum 0.5(L+R)"]["anti-phase L=-R"] == "silent")
# ffmpeg's own stereo->mono (what aformat=channel_layouts=mono does)
a = render(ST, "aformat=channel_layouts=mono", 1)
check("3 aformat=channel_layouts=mono on L=R (re L): ffmpeg folds at 0.707(L+R)", db(rms(a[:, 0])) - REF, 3.0103, 0.01)
a = render(LEFT_ONLY, "aformat=channel_layouts=mono", 1)
check("3 aformat=channel_layouts=mono on left-only (re L): ditto", db(rms(a[:, 0])) - lvl(render(LEFT_ONLY, ""))[0], -3.0103, 0.02)

# ---- 4. swap, left-only, right-only -----------------------------------------------------------
sw = render(D, "pan=stereo|c0=c1|c1=c0")
exact("4 swap: L' is bit-identical to R, R' to L", np.array_equal(sw[:, 0], base[:, 1]) and np.array_equal(sw[:, 1], base[:, 0]))
lf = render(D, "pan=stereo|c0=c0|c1=c0")
exact("4 left (fill): both channels bit-identical to source L", np.array_equal(lf[:, 0], base[:, 0]) and np.array_equal(lf[:, 1], base[:, 0]))
rt = render(D, "pan=stereo|c0=c1|c1=c1")
exact("4 right (fill): both channels bit-identical to source R", np.array_equal(rt[:, 0], base[:, 1]) and np.array_equal(rt[:, 1], base[:, 1]))
mu = render(D, "pan=stereo|c0=c0|c1=0*c0")
exact("4 left (mute other): R exactly zero, L bit-identical", float(np.max(np.abs(mu[:, 1]))) == 0.0 and np.array_equal(mu[:, 0], base[:, 0]))
idn = render(D, "pan=stereo|c0=c0|c1=c1")
exact("4 identity matrix pan=stereo|c0=c0|c1=c1 is bit-identical in float", np.array_equal(idn, base))
# one-channel source (lav mic): 'left' recovers a centred signal at the same level as the live channel
check("4 lav-mic case: 'left' fill of left-only source, per-channel dB re the live channel",
      lvl(render(LEFT_ONLY, "pan=stereo|c0=c0|c1=c0"))[1] - lvl(render(LEFT_ONLY, ""))[0], 0.0, 0.01)

# ---- 5. latency and length -----------------------------------------------------------------
IMP = "aevalsrc=exprs='if(eq(n\\,100)\\,1\\,0)|if(eq(n\\,100)\\,1\\,0)':s=48000:d=1"
for label, af in (("pan balance", "pan=stereo|c0=0.5*c0|c1=1*c1"), ("pan swap", "pan=stereo|c0=c1|c1=c0"), ("pan mono", MONO["half_sum 0.5(L+R)"])):
    a = render(IMP, af)
    onset = int(np.argmax(np.abs(a[:, 1]) > 1e-9)) if label != "pan balance" else int(np.argmax(np.abs(a[:, 1]) > 1e-9))
    check(f"5 {label}: impulse onset sample (input at 100)", onset, 100, 0)
    check(f"5 {label}: output length in samples", len(a), R, 0)

# ---- 6. keyframes: pan drops runtime commands ---------------------------------------------
cmd = f"asendcmd=c='0.5 pan args {chr(34)}stereo|c0=0*c0|c1=1*c1{chr(34)}',pan=stereo|c0=c0|c1=c1"
r = subprocess.run([FF, "-hide_banner", "-loglevel", "error", "-f", "lavfi", "-i", ST + ",aformat=sample_fmts=dbl", "-af", cmd, "-f", "f64le", "-"], capture_output=True)
a = np.frombuffer(r.stdout, dtype="<f8").reshape(-1, 2) if r.stdout else np.zeros((1, 2))
exact("6 asendcmd to pan has no effect (L unchanged after 0.5 s)", len(r.stdout) > 0 and np.array_equal(a, render(ST, "")))
# the alternative that does take commands: per-channel volume via asplit/channelsplit/join is 3+ filters per element
CH = "channelsplit=channel_layout=stereo[L][R];[L]asendcmd=c='0.5 volume volume 0',volume=1[Lo];[R]anull[Ro];[Lo][Ro]join=inputs=2:channel_layout=stereo"
r = subprocess.run([FF, "-hide_banner", "-loglevel", "error", "-f", "lavfi", "-i", ST + ",aformat=sample_fmts=dbl", "-filter_complex", CH, "-f", "f64le", "-"], capture_output=True)
a = np.frombuffer(r.stdout, dtype="<f8").reshape(-1, 2) if r.stdout else np.zeros((1, 2))
exact("6 split + volume(asendcmd) + join does animate one channel (L silent in last 0.4 s)",
      len(r.stdout) > 0 and float(np.max(np.abs(a[int(0.6 * R):, 0]))) == 0.0 and float(np.max(np.abs(a[int(0.6 * R):, 1]))) > 0.1)

# ---- 7. build-drift digests -----------------------------------------------------------------
digests = {}
CASES = {
    "balance_cos p=0.5": (D, "pan=stereo|c0=0.707106781*c0|c1=1*c1"),
    "cp3 p=0.3": (ST, "pan=stereo|c0=%.9f*c0|c1=%.9f*c1" % law_cp3(0.3)),
    "mono half_sum": (D, MONO["half_sum 0.5(L+R)"]),
    "swap": (D, "pan=stereo|c0=c1|c1=c0"),
    "left fill": (D, "pan=stereo|c0=c0|c1=c0"),
    "noise half_sum": (NOISE, MONO["half_sum 0.5(L+R)"]),
}
for k, (src, af) in CASES.items():
    a = render(src, af)
    digests[k] = hashlib.sha256(a.tobytes()).hexdigest()[:16]
    a2 = render(src, af)
    exact(f"7 repeat-run identical: {k}", np.array_equal(a, a2))
    if DUMP:
        os.makedirs(DUMP, exist_ok=True); open(os.path.join(DUMP, k.replace(" ", "_").replace("=", "") + ".f64"), "wb").write(a.tobytes())
ROWS["digests"] = digests
print("\ndigests:", json.dumps(digests, indent=1))

ROWS["ffmpeg"] = VERSION
if OUTJSON:
    json.dump(ROWS, open(OUTJSON, "w"), indent=1)
print("\nFAILED: " + "; ".join(FAIL) if FAIL else "\nall claims hold")
sys.exit(1 if FAIL else 0)
