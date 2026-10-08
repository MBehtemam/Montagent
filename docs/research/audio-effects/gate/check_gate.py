#!/usr/bin/env python3
"""Evidence for the noise-gate ADR (0181): agate's static curve, timing, onset, commands and drift.

Everything is measured on PCM (double precision, BEFORE any AAC encode; ADR-0173 section 3) from
lavfi signals built from literal parameters. Exits non-zero when a number stops holding.
Stdlib only. ffmpeg is taken from $FFMPEG or PATH. Writes measurements.json beside itself.

Level convention under test (same as the compressor, ADR-0180 section 2): threshold_db and every
gate level are RMS dBFS (detection=rms), so a full-scale sine reads -3.01.

The filter string the renderer will emit for the member
  {"name":"noise_gate","threshold_db":T,"ratio":R,"attack_ms":A,"release_ms":L,"range_db":G}
is gate() below: threshold -> 10^(T/20), range -> 10^(-G/20), and ratio -> (R+1)/2, because agate in
rms mode applies its `ratio` to a power-domain slope, so ffmpeg's ratio r gives a reduction of
2*(r-1) dB per dB under the threshold. With r = (R+1)/2 the authored R is a plain downward
expander ratio: reduction = (R-1) dB per dB under the threshold.
"""
import array, hashlib, json, math, os, subprocess, sys

FFMPEG = os.environ.get("FFMPEG", "ffmpeg")
SR = 48000
FAILS, ROWS = [], []


def lin(db):
    return 10 ** (db / 20)


def gate(thr, ratio, att, rel, rng, name=""):
    n = f"@{name}" if name else ""
    return (f"agate{n}=threshold={lin(thr):.9f}:ratio={(ratio + 1) / 2:.9f}:attack={att}:release={rel}"
            f":range={lin(-rng):.9f}:knee=1:detection=rms:link=average:makeup=1:mode=downward")


def render(src, chain, extra=()):
    af = "aformat=sample_fmts=dbl," + (chain + "," if chain else "") + "aformat=sample_fmts=dbl"
    raw = subprocess.run([FFMPEG, "-hide_banner", "-loglevel", "error", *extra, "-f", "lavfi", "-i", src,
                          "-af", af, "-f", "f64le", "-"], check=True, capture_output=True).stdout
    a = array.array("d")
    a.frombytes(raw)
    return a


def tone(peak_db, dur):
    return f"aevalsrc='{lin(peak_db):.9f}*sin(2*PI*1000*t)':s={SR}:d={dur}"


def rms_db(x):
    return 10 * math.log10(sum(v * v for v in x) / len(x) + 1e-300)


def row(name, ok, **kw):
    ROWS.append({"check": name, "ok": ok, **kw})
    if not ok:
        FAILS.append(name)
    return ok


def check(name, got, want, tol):
    ok = abs(got - want) <= tol
    row(name, ok, got=round(got, 5), want=want, tol=tol)
    print(("ok  " if ok else "FAIL"), f"{name}: {got:.4f} (want {want} +/- {tol})")


def info(name, **kw):
    ROWS.append({"check": name, "info": True, **kw})
    print("info", name, kw)


RMS_OFF = 3.0103
# 1. Static transfer at the reference timing (100/100 ms). Under the threshold the reduction is
#    min(range_db, (R-1) * (thr - in_rms)); at or above it, 0 dB. Two-sided.
STEP = 0.3
for thr, R, G in ((-30, 2, 80), (-30, 4, 80), (-30, 10, 80), (-40, 3, 12), (-20, 100, 40), (-30, 1, 80)):
    for in_rms in (-70, -60, -50, -45, -40, -36, -33, -31, -29, -25, -15, -3):
        x = render(tone(in_rms + RMS_OFF, 3), gate(thr, R, 100, 100, G))
        got = rms_db(x[len(x) * 2 // 3:]) - in_rms
        want = 0.0 if in_rms >= thr else -min(G, (R - 1) * (thr - in_rms))
        # The knee is hard, but the detector settles over ~100 ms and a level within 1 dB of the
        # threshold is on the transition: it is recorded, not asserted.
        if abs(in_rms - thr) < 1.5:
            info(f"transition thr={thr} R={R} G={G} in_rms={in_rms}", reduction_db=round(got, 3), formula_db=round(want, 3))
            continue
        check(f"transfer thr={thr} R={R} range={G} in_rms={in_rms}", got, round(want, 3), STEP)

# 2. Steady-state pass: far above the threshold the gate is the identity. Max sample difference.
src = tone(-6, 2)
a, b = render(src, ""), render(src, gate(-40, 4, 5, 50, 80))
d = max(abs(u - v) for u, v in zip(a[SR // 2:], b[SR // 2:]))
row("open gate is the identity (max |out-in| after 0.5 s)", d <= 1e-9, got=d, want=0, tol=1e-9)
print(("ok  " if d <= 1e-9 else "FAIL"), "open gate is the identity, max diff", d)
row("length is exact", len(a) == len(b), got=len(b), want=len(a))

# 3. Timing: the realised threshold. agate smooths the LINEAR power of the signal with the attack
#    coefficient when it rises and the release coefficient when it falls, so on a sine (whose power
#    ripples at 2 kHz) the detector reads high when attack < release and low when attack > release.
#    The bias is a shift of the effective threshold: reduction = (R-1) * (thr + delta - in).
#    delta is measured with R = 2 (1 dB of reduction per dB), deep under the threshold, settled.
def delta(att, rel, hz=1000, d=12):
    src = f"aevalsrc='{lin(-40 + RMS_OFF):.9f}*sin(2*PI*{hz}*t)':s={SR}:d={d}"
    x = render(src, gate(-30, 2, att, rel, 80))
    return rms_db(x[len(x) * 3 // 4:]) + 40 - (-10)  # realised minus formula, dB (+ = shallower = threshold read high)

DELTA_BAND = (-0.3, 3.1)  # +3.01 is the sine's own peak-to-mean power ratio: the limit as attack/release -> 0
swept = {}
for att, rel in ((0.1, 1), (1, 10), (5, 50), (20, 250), (5, 5), (50, 50), (100, 100), (1000, 1000), (5, 1000), (50, 9000)):
    dl = delta(att, rel)
    swept[f"{att}/{rel}"] = round(dl, 3)
    ok = DELTA_BAND[0] <= dl <= DELTA_BAND[1]
    row(f"threshold bias at attack={att} release={rel} (dB, attack <= release)", ok, got=round(dl, 3), want=list(DELTA_BAND))
    print(("ok  " if ok else "FAIL"), f"threshold bias attack={att} release={rel}: {dl:+.3f} dB (want within {DELTA_BAND})")
for att, rel in ((100, 100), (5, 5), (1000, 1000)):
    check(f"exact at attack = release = {att}", swept.get(f"{att}/{rel}", delta(att, rel)), 0.0, 0.3)
for att, rel in ((20, 10), (100, 10), (500, 100), (2000, 1)):  # attack slower than release: recorded, not asserted
    dl = delta(att, rel)
    info(f"threshold bias at attack={att} > release={rel} (dB; negative = gate looser than written)", bias_db=round(dl, 2))
    swept[f"{att}/{rel}"] = round(dl, 3)
for hz in (200, 5000):
    info(f"threshold bias at 5/50 on a {hz} Hz sine (dB)", bias_db=round(delta(5, 50, hz), 2))
check("1 kHz at 5/50 vs 100/100 differ: the bias is real, not noise", delta(5, 50) - delta(100, 100), 2.12, 0.3)

# 3b. Noise is not a sine: a floor of pink noise below the threshold leaks through a gate with a fast
#     attack and slow release, because the detector follows the noise's own power peaks. Recorded
#     (not asserted): the reduction of a seeded pink-noise floor, threshold -38, R = 10, range 30.
for att, rel in ((5, 150), (50, 50), (1, 20)):
    for lev in (-50, -44, -40):
        src = f"anoisesrc=color=pink:amplitude={lin(lev + 14.3):.6f}:seed=7:d=6:r={SR}"
        a, b = render(src, ""), render(src, gate(-38, 10, att, rel, 30))
        info(f"pink-noise floor {lev} dB rms, attack={att} release={rel}: reduction (dB; 30 = fully closed)", reduction_db=round(rms_db(a[SR * 2:]) - rms_db(b[SR * 2:]), 2))

# 4. Attack and release as times. A -50 rms tone; a burst at -24 rms (6 dB over a -30 threshold) from
#    1.0 s to 2.0 s. The detector smooths in the linear power domain, so how long the gate takes to
#    open depends on how far over the threshold the burst is: these are times for THIS signal only.
def env(att, rel, thr=-30, R=10, G=60):
    src = (f"aevalsrc='if(between(t,1,2),{lin(-24 + RMS_OFF)},{lin(-50 + RMS_OFF)})*sin(2*PI*1000*t)':s={SR}:d=4")
    y = render(src, gate(thr, R, att, rel, G))
    w = SR // 1000  # 1 ms windows = exactly one period of the 1 kHz tone
    return [rms_db(y[i:i + w]) for i in range(0, len(y) - w, w)]


def t_to(levels, t0, pred):
    for i in range(int(t0 * 1000), len(levels)):
        if pred(levels[i]):
            return (i - int(t0 * 1000)) / 1000
    return None


att_t, rel_t = {}, {}
for att in (2, 10, 50, 200):
    e = env(att, 100)
    att_t[att] = t_to(e, 1.0, lambda l: l >= -24 - 1.0)  # within 1 dB of the open level
    info(f"attack_ms={att}: ms from burst start to within 1 dB of open", ms=None if att_t[att] is None else att_t[att] * 1000)
for rel in (10, 100, 500):
    e = env(2, rel)
    rel_t[rel] = t_to(e, 2.0, lambda l: l <= -50 - 20)  # 20 dB of reduction reached
    info(f"release_ms={rel}: ms from burst end to 20 dB of reduction", ms=None if rel_t[rel] is None else rel_t[rel] * 1000)
ok = None not in att_t.values() and att_t[2] < att_t[10] < att_t[50] < att_t[200]
row("attack ordering: a longer attack opens slower", ok, got=att_t)
print(("ok  " if ok else "FAIL"), "attack ordering", att_t)
ok = None not in rel_t.values() and rel_t[10] < rel_t[100] < rel_t[500]
row("release ordering: a longer release closes slower", ok, got=rel_t)
print(("ok  " if ok else "FAIL"), "release ordering", rel_t)

# 5. Onset (ADR-0143/#843 rule: 0 samples or stay out). A quiet bed (closed gate) then a loud burst at 0.5 s;
#    the first sample whose |value| exceeds 0.01 must not move, for the fastest and a slow attack.
#    Also an isolated impulse: the sample index of its peak must not move.
for att in (0.1, 5, 50):
    burst = f"aevalsrc='if(lt(t,0.5),{lin(-70)},{lin(-6)})*sin(2*PI*1000*t)':s={SR}:d=1"
    a, b = render(burst, ""), render(burst, gate(-30, 10, att, 100, 60))
    ia = next(i for i, v in enumerate(a) if abs(v) > 0.01)
    ib = next((i for i, v in enumerate(b) if abs(v) > 0.01), None)
    # with a slow attack the burst is not shaped to threshold until the gain opens; the first sample
    # over 0.01 may come LATER (attack ramp), never earlier, and with attack 0.1 it must be the same.
    info(f"onset: first sample > 0.01, attack={att}", bypass=ia, gated=ib, shift=None if ib is None else ib - ia)
    if att == 0.1:
        row("onset shift at attack 0.1 ms (samples)", ib == ia, got=None if ib is None else ib - ia, want=0, tol=0)
        print(("ok  " if ib == ia else "FAIL"), "onset shift at attack 0.1 ms:", None if ib is None else ib - ia)
imp = f"aevalsrc='if(eq(n,{SR // 2}),0.9,0)':s={SR}:d=1"
a, b = render(imp, ""), render(imp, gate(-30, 10, 5, 100, 60))
pa, pb = max(range(len(a)), key=lambda i: abs(a[i])), max(range(len(b)), key=lambda i: abs(b[i]))
row("no latency: impulse peak does not move (samples)", pa == pb, got=pb - pa, want=0, tol=0)
row("length exact on impulse", len(a) == len(b), got=len(b), want=len(a))
print(("ok  " if pa == pb else "FAIL"), "impulse peak shift:", pb - pa, "| length", len(a), len(b))

# 6. Does agate honour commands? (ADR-0842: static parameters only unless the filter honours them.)
#    A steady -36 rms tone; the threshold starts at -50 (open) and is commanded to -20 (closed) at 1.0 s.
cmd_src = tone(-36 + RMS_OFF, 3)
base = f"agate@g=threshold={lin(-50):.9f}:ratio=5.5:attack=5:release=20:range={lin(-60):.9f}:knee=1:detection=rms"
def cmd_run(key, val):
    return render(cmd_src, f"asendcmd=c='1.0 agate@g {key} {val}',{base}")
def second_half_gain(x):
    return rms_db(x[int(SR * 2.0):]) - (-36)
static_closed = render(cmd_src, f"agate=threshold={lin(-20):.9f}:ratio=5.5:attack=5:release=20:range={lin(-60):.9f}:knee=1:detection=rms")
want_closed = second_half_gain(static_closed)
cmd_res = {}
for key, val in (("threshold", f"{lin(-20):.9f}"), ("range", f"{lin(-12):.9f}"), ("ratio", "1"), ("attack", "2000"), ("release", "2000")):
    cmd_res[key] = round(second_half_gain(cmd_run(key, val)), 3)
info("command effect on gain after t=1 s (dB)", **cmd_res, static_threshold_reference=round(want_closed, 3))
# Control: the harness does move a filter that honours commands (volume), so a null result is real.
vol = render(cmd_src, "asendcmd=c='1.0 volume@v volume 0.5',volume@v=1.0")
ctl = second_half_gain(vol)
check("control: asendcmd moves volume@v by -6.02 dB", ctl, -6.0206, 0.01)
honoured = abs(cmd_res["threshold"] - want_closed) < STEP
row("agate ignores a threshold command (ADR-0842: static parameters only)", not honoured, got=cmd_res["threshold"], static_reference=round(want_closed, 3),
    note="a command is accepted and does nothing; if this ever fails agate started honouring commands and the ADR can be revisited")
print(("ok  " if not honoured else "FAIL"), f"agate threshold command: gain {cmd_res['threshold']} dB vs {round(want_closed, 3)} dB had the value been static")
ok = all(abs(v) < 0.01 for v in cmd_res.values())
row("agate ignores range, ratio, attack and release commands too", ok, got=cmd_res)
print(("ok  " if ok else "FAIL"), "all five commands are no-ops:", cmd_res)

# 7. Range edges and the negative controls (a document that passes validate never fails at render).
def accepts(chain):
    return subprocess.run([FFMPEG, "-hide_banner", "-loglevel", "error", "-f", "lavfi", "-i", "anoisesrc=d=0.2:seed=1",
                           "-af", chain, "-f", "null", "-"], capture_output=True).returncode == 0
edges = [gate(t, r, a, rl, g) for t, r, a, rl, g in ((-80, 100, 0.1, 1, 80), (0, 1, 2000, 9000, 0), (-80, 1, 0.1, 9000, 80), (0, 100, 2000, 1, 80))]
for c in edges:
    ok = accepts(c)
    row("range edge accepted: " + c[:80], ok)
    print(("ok  " if ok else "FAIL"), "range edge accepted:", c[:80])
for name, c in (("ratio 18001 -> ffmpeg 9000.5", gate(-30, 17999, 5, 50, 60).replace(f"{(17999 + 1) / 2:.9f}", "9000.5")),
                ("attack 0.001 ms", gate(-30, 4, 0.001, 50, 60)), ("release 9001 ms", gate(-30, 4, 5, 9001, 60))):
    ok = not accepts(c)
    row("NEGATIVE CONTROL rejected: " + name, ok)
    print(("ok  " if ok else "FAIL"), "negative control rejected:", name)

# 8. Same-build determinism and SIMD independence: identical bytes across runs and -cpuflags 0.
c = gate(-30, 4, 5, 50, 60)
src = f"anoisesrc=seed=7:amplitude=0.05:d=3"
h = [hashlib.sha256(render(src, c, extra).tobytes()).hexdigest() for extra in ((), (), ("-cpuflags", "0"))]
row("byte-identical across runs and -cpuflags 0", h[0] == h[1] == h[2], got=[x[:12] for x in h])
print(("ok  " if h[0] == h[1] == h[2] else "FAIL"), "determinism / cpuflags:", [x[:12] for x in h])

ver = subprocess.run([FFMPEG, "-version"], capture_output=True, text=True).stdout.splitlines()[0]
json.dump({"ffmpeg": ver, "rows": ROWS, "pass": not FAILS},
          open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "measurements.json"), "w"), indent=1)
if FAILS:
    print("\nFAILED:", FAILS)
    sys.exit(1)
print("\nall gate numbers hold")
