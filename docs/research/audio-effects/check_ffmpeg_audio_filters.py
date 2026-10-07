#!/usr/bin/env python3
"""Re-derives every measured claim in FFMPEG-FILTERS.md (#797) against an ffmpeg.

Usage:  python3 -I check_ffmpeg_audio_filters.py [path/to/ffmpeg]     (default: ffmpeg on PATH)

Exits 1, naming each claim that no longer holds, or 0 when all of them do. Measured on
BtbN's `ffmpeg-n7.1.5-12-g1fdbca85aa` linux64 GPL build (the pin in
ci/install_ffmpeg_floor.sh, i.e. Montagent's floor); install it with
`ci/install_ffmpeg_floor.sh <dir>` and pass `<dir>/bin/ffmpeg`.

What it asserts, in the note's section order:
  A. every candidate filter is compiled in (rubberband is reported, never required);
  B. every candidate is byte-identical across two runs, across -filter_threads 1 vs 8,
     and inside a Montagent-shaped mix chain encoded to AAC 160k (-bitexact);
  C. the delay each filter adds (impulse-peak shift, in 48 kHz samples) and the samples it
     adds to or removes from a 4 s input;
  D. which runtime commands take effect, which are silently ignored, and that agate's
     `threshold` command is accepted and does nothing;
     It also asserts deesser's default (i=0) is a pass-through, and an unseeded anoisesrc is
     not deterministic;
  E. asendcmd applies a command at the first *frame* boundary at or after its time, and
     asetnsamples sets that granularity;
  F. the pitch/tempo filters are not sample-exact: the onset of a tone burst lands more
     than 100 samples away from where an exact time-stretch would put it.
  G. (report only) which outputs change when SIMD is disabled (-cpuflags 0): those are the
     filters whose bytes can differ between x86_64 and aarch64 builds.

Needs numpy.
"""
import hashlib
import os
import shutil
import subprocess
import sys
import tempfile

import numpy as np

FF = sys.argv[1] if len(sys.argv) > 1 else (shutil.which("ffmpeg") or "ffmpeg")
R = 48000
FAIL = []


def claim(ok, text):
    print(("ok    " if ok else "FAIL  ") + text)
    if not ok:
        FAIL.append(text)


def ff(args, check=True):
    p = subprocess.run([FF, "-hide_banner", "-nostdin", "-v", "error", "-y", *args], capture_output=True)
    if check and p.returncode:
        raise RuntimeError(p.stderr.decode(errors="replace")[-600:])
    return p


TMP = tempfile.mkdtemp(prefix="ffaudio-")
SRC = os.path.join(TMP, "src.wav")      # 0.5 s silence, then a tone mix with a noise band
SRC2 = os.path.join(TMP, "src2.wav")    # a second element for the mix
EXPR = ("if(lt(t,0.5),0,0.3*sin(2*PI*220*t)+0.15*sin(2*PI*1000*t)"
        "+0.1*sin(2*PI*6500*t)*(0.5+0.5*sin(2*PI*2*t))+0.05*(2*random(0)-1))")
EXPR2 = "0.3*sin(2*PI*330*t)+0.1*sin(2*PI*2500*t)"
for path, e in ((SRC, EXPR), (SRC2, EXPR2)):
    ff(["-f", "lavfi", "-i", f"aevalsrc=exprs='{e}|{e}':s={R}:d=4", "-c:a", "pcm_f32le", path])

TAIL = ",aformat=sample_fmts=flt:sample_rates=48000:channel_layouts=stereo[o]"


def pcm(graph, extra=(), inputs=(SRC,), lavfi=None):
    args = list(extra)
    if lavfi:
        args += ["-f", "lavfi", "-i", lavfi]
    for i in inputs:
        args += ["-i", i]
    p = ff(args + ["-filter_complex", graph + TAIL, "-map", "[o]", "-f", "f32le", "-"])
    return p.stdout


def mix_aac(effect):
    """`effect` inside the chain render.rs writes, two elements amixed, AAC 160k."""
    g = ("[0:a]aformat=sample_rates=48000:channel_layouts=stereo,atrim=start=0.000:end=4.000,"
         f"asetpts=PTS-STARTPTS,{effect},atrim=end=4.000,asetpts=PTS-STARTPTS,"
         "atrim=start=0.000:end=4.000,asetpts=PTS-STARTPTS,adelay=delays=250:all=1[a0];"
         "[1:a]aformat=sample_rates=48000:channel_layouts=stereo,atrim=start=0.000:end=2.000,"
         "asetpts=PTS-STARTPTS,volume=0.5,adelay=delays=1000:all=1[a1];"
         "[a0][a1]amix=inputs=2:normalize=0,apad=whole_dur=4.500,atrim=end=4.500[mix]")
    p = ff(["-i", SRC, "-i", SRC2, "-filter_complex", g, "-map", "[mix]", "-c:a", "aac", "-b:a", "160k",
            "-fflags", "+bitexact", "-flags:a", "+bitexact", "-f", "adts", "-"])
    return hashlib.sha256(p.stdout).hexdigest()


def frames(raw):
    return np.frombuffer(raw, np.float32).reshape(-1, 2)


# ---------------------------------------------------------------- A. availability
listing = ff(["-filters"]).stdout.decode()
names = {line.split()[1] for line in listing.splitlines() if len(line.split()) > 2 and line.startswith(" ")}
STOCK = ["loudnorm", "alimiter", "acompressor", "agate", "equalizer", "highpass", "lowpass", "bass",
         "treble", "anequalizer", "superequalizer", "firequalizer", "pan", "acrossfade", "afade",
         "afftdn", "anlmdn", "deesser", "aecho", "afir", "anoisesrc", "asetrate", "aresample", "atempo",
         "asendcmd", "asetnsamples", "volume", "dynaudnorm", "speechnorm", "compand"]
for f in STOCK:
    claim(f in names, f"A  `{f}` is compiled in")
HAVE_RB = "rubberband" in names
print(f"info  rubberband is {'present' if HAVE_RB else 'ABSENT'} in this build (needs --enable-librubberband)")

# ---------------------------------------------------------------- B. determinism
P = "I=-16:TP=-1.5:LRA=20"
meas = ff(["-v", "info", "-i", SRC, "-af", f"loudnorm={P}:print_format=json", "-f", "null", "-"]).stderr.decode()
import json  # noqa: E402
m = json.loads(meas[meas.rindex("{"):meas.rindex("}") + 1])
MEAS = (f"measured_I={m['input_i']}:measured_TP={m['input_tp']}:measured_LRA={m['input_lra']}:"
        f"measured_thresh={m['input_thresh']}")
CANDIDATES = {
    "loudnorm one-pass": f"loudnorm={P},aresample=48000",
    "loudnorm two-pass linear": f"loudnorm={P}:{MEAS}:linear=true,aresample=48000",
    "loudnorm two-pass dynamic": f"loudnorm={P}:{MEAS}:linear=false,aresample=48000",
    "alimiter": "alimiter=limit=0.3",
    "alimiter latency=1": "alimiter=limit=0.3:latency=1",
    "acompressor": "acompressor=threshold=0.05:ratio=4",
    "agate": "agate=threshold=0.05",
    "equalizer": "equalizer=f=1000:t=q:w=1:g=6",
    "highpass": "highpass=f=300",
    "lowpass": "lowpass=f=3000",
    "bass": "bass=g=6",
    "treble": "treble=g=-6",
    "anequalizer": "anequalizer=c0 f=1000 w=200 g=6 t=0|c1 f=1000 w=200 g=6 t=0",
    "superequalizer": "superequalizer=6b=2:7b=2:8b=2",
    "firequalizer": "firequalizer=gain_entry='entry(0,0);entry(1000,6);entry(4000,0)'",
    "pan": "pan=stereo|c0=0.7*c0+0.3*c1|c1=0.3*c0+0.7*c1",
    "afftdn": "afftdn=nr=12",
    "anlmdn": "anlmdn=s=0.001",
    "deesser": "deesser=i=0.5",
    "aecho": "aecho=0.8:0.9:60:0.4",
    "aecho multi-tap": "aecho=0.8:0.88:23|37|53|71:0.35|0.3|0.25|0.2",
    "asetrate pitch": "asetrate=60476.21,aresample=48000,atempo=0.793701",
    "atempo": "atempo=1.25",
    "dynaudnorm": "dynaudnorm",
}
if HAVE_RB:
    CANDIDATES["rubberband pitch"] = "rubberband=pitch=1.259921"
    CANDIDATES["rubberband tempo"] = "rubberband=tempo=1.25"
SIMD_SENSITIVE = []
for name, g in CANDIDATES.items():
    a = pcm(f"[0:a]{g}")
    b = pcm(f"[0:a]{g}")
    t1 = pcm(f"[0:a]{g}", extra=["-filter_threads", "1"])
    t8 = pcm(f"[0:a]{g}", extra=["-filter_threads", "8"])
    claim(a == b and a == t1 == t8, f"B  {name}: byte-identical across reruns and -filter_threads 1/8")
    claim(mix_aac(g) == mix_aac(g), f"B  {name}: byte-identical AAC through the mix chain")
    if pcm(f"[0:a]{g}", extra=["-cpuflags", "0"]) != a:
        SIMD_SENSITIVE.append(name)
# afir with an impulse response generated in the graph (seeded noise, no file).
IRG = ("anoisesrc=d=0.6:c=pink:r=48000:a=0.5:seed=1,afade=t=out:st=0:d=0.6:curve=exp,"
       "aformat=channel_layouts=stereo[ir];[0:a][ir]afir=dry=8:wet=2")
claim(pcm(IRG) == pcm(IRG), "B  afir with a seeded in-graph IR: byte-identical across reruns")
UNSEEDED = "anoisesrc=d=0.1:r=48000"
claim(pcm("[0:a]anull", inputs=(), lavfi=UNSEEDED) != pcm("[0:a]anull", inputs=(), lavfi=UNSEEDED),
      "B  anoisesrc without `seed` differs run to run (so the IR's seed must be pinned)")
CURVES = ("nofade tri qsin esin hsin log ipar qua cub squ cbr par exp iqsin ihsin dese desi losi "
          "sinc isinc quat quatr qsin2 hsin2").split()
xf_ok, xf_len = True, set()
for c in CURVES:
    g = f"[0:a][1:a]acrossfade=d=0.5:c1={c}:c2={c}"
    a = pcm(g, inputs=(SRC, SRC2))
    xf_ok &= a == pcm(g, inputs=(SRC, SRC2))
    xf_len.add(len(frames(a)))
claim(xf_ok, f"B  acrossfade: all {len(CURVES)} curves byte-identical across reruns")
claim(xf_len == {360000}, f"C  acrossfade d=0.5 of two 4 s inputs is exactly 7.5 s (got {sorted(xf_len)})")

# ---------------------------------------------------------------- C. delay and length
IMP = "aevalsrc=exprs='if(eq(n,24000),0.5,0)|if(eq(n,24000),0.5,0)':s=48000:d=4"


def delay_and_length(g):
    x = frames(pcm(f"[0:a]{g}", inputs=(), lavfi=IMP))[:, 0]
    return int(np.argmax(np.abs(x))) - 24000, len(x) - 4 * R


EXPECT = {  # filter: (impulse-peak delay in samples, samples added to a 4 s input)
    "loudnorm=I=-16,aresample=48000": (0, 0),
    "alimiter=limit=0.3": (239, 0),
    "alimiter=limit=0.3:attack=20": (959, 0),
    "alimiter=limit=0.3:latency=1": (0, 0),
    "acompressor=threshold=0.05": (0, 0),
    "agate": (0, 0),
    "equalizer=f=1000:t=q:w=1:g=6": (0, 0),
    "highpass=f=300": (0, 0),
    "bass=g=6": (0, 0),
    "anequalizer=c0 f=1000 w=200 g=6 t=0|c1 f=1000 w=200 g=6 t=0": (0, 0),
    "superequalizer=6b=2": (4095, 0),
    "firequalizer=gain_entry='entry(1000,6)'": (480, 960),
    "firequalizer=gain_entry='entry(1000,6)':zero_phase=1,atrim=start=0,asetpts=PTS-STARTPTS": (0, 480),
    "pan=stereo|c0=c0|c1=c1": (0, 0),
    "afftdn=nr=12": (1200, 0),
    "anlmdn=s=0.001": (384, 0),
    "deesser=i=0.5": (0, 0),
    "aecho=0.8:0.9:60:0.4": (0, 2880),
    "aecho=0.8:0.88:23|37|53|71:0.35|0.3|0.25|0.2": (0, 3408),
    # The generic compensation: pad the input by the filter's delay, cut that many from the front.
    "apad=pad_len=1200,afftdn=nr=12,atrim=start_sample=1200,asetpts=PTS-STARTPTS": (0, 0),
    "apad=pad_len=4095,superequalizer=6b=2,atrim=start_sample=4095,asetpts=PTS-STARTPTS": (0, 0),
    "apad=pad_len=384,anlmdn=s=0.001,atrim=start_sample=384,asetpts=PTS-STARTPTS": (0, 0),
}
for g, want in EXPECT.items():
    got = delay_and_length(g)
    claim(got == want, f"C  {g}: delay {got[0]} samples, length {got[1]:+d} (claimed {want[0]}, {want[1]:+d})")
xi = frames(pcm(IRG, inputs=(), lavfi=IMP))[:, 0]
first = int(np.flatnonzero(np.abs(xi) > 1e-6)[0]) - 24000
claim(first == 0 and len(xi) == 4 * R,
      f"C  afir (in-graph IR): no added delay ({first:+d}) and no tail past the input ({len(xi) - 4 * R:+d})")
if HAVE_RB:
    n = len(frames(pcm("[0:a]rubberband=tempo=1.25")))
    claim(n == 153600, f"C  rubberband tempo=1.25 on 192000 samples gives exactly 153600 (got {n})")
n = len(frames(pcm("[0:a]atempo=1.25")))
claim(n != 153600, f"C  atempo=1.25 on 192000 samples does NOT give exactly 153600 (got {n})")

# ---------------------------------------------------------------- D. runtime commands


def with_cmd(cmd, flt):
    path = os.path.join(TMP, "c.cmd")
    with open(path, "w") as fh:
        fh.write(cmd + ";\n")
    return pcm(f"[0:a]asendcmd=f={path},{flt}")


TAKES = {  # (command, filter) whose output must change
    "volume": ("1.0 volume@x volume 0.25", "volume@x=volume=1:eval=frame"),
    "equalizer g": ("1.0 equalizer@x g 12", "equalizer@x=f=1000:t=q:w=1:g=0"),
    "highpass f": ("1.0 highpass@x f 2000", "highpass@x=f=100"),
    "bass g": ("1.0 bass@x g 12", "bass@x=g=0"),
    "treble g": ("1.0 treble@x g -12", "treble@x=g=0"),
    "anequalizer change": ("1.0 anequalizer@x change 0|f=1000|w=200|g=12",
                           "anequalizer@x=c0 f=1000 w=200 g=0 t=0|c1 f=1000 w=200 g=0 t=0"),
    "firequalizer gain": ("1.0 firequalizer@x gain 'if(gt(f,1000),6,0)'", "firequalizer@x=gain=0"),
    "acompressor threshold": ("1.0 acompressor@x threshold 0.03", "acompressor@x=threshold=1"),
    "alimiter limit": ("1.0 alimiter@x limit 0.2", "alimiter@x=limit=1"),
    "afftdn nr": ("1.0 afftdn@x nr 40", "afftdn@x=nr=1"),
    "anlmdn s": ("1.0 anlmdn@x s 100", "anlmdn@x=s=0.00001"),
    "atempo tempo": ("1.0 atempo@x tempo 1.5", "atempo@x=tempo=1"),
    "dynaudnorm p": ("1.0 dynaudnorm@x p 0.3", "dynaudnorm@x"),
}
if HAVE_RB:
    TAKES["rubberband pitch"] = ("1.0 rubberband@x pitch 1.5", "rubberband@x=pitch=1")
for name, (cmd, flt) in TAKES.items():
    a = with_cmd(cmd, flt)
    claim(a != pcm(f"[0:a]{flt}") and a == with_cmd(cmd, flt),
          f"D  `{name}` command changes the output, deterministically")
IGNORED = {  # accepted by asendcmd, exit 0, output unchanged
    "agate threshold (accepted, no effect)": ("1.0 agate@x threshold 0.5", "agate@x=threshold=0.001"),
    "pan (no commands)": ("1.0 pan@x args mono", "pan@x=stereo|c0=c0|c1=c1"),
    "aecho (no commands)": ("1.0 aecho@x decays 0.1", "aecho@x=0.8:0.9:60:0.4"),
    "deesser (no commands)": ("1.0 deesser@x i 1", "deesser@x=i=0.5"),
    "superequalizer (no commands)": ("1.0 superequalizer@x 6b 10", "superequalizer@x"),
    "loudnorm (no commands)": ("1.0 loudnorm@x I -10", "loudnorm@x=I=-16,aresample=48000"),
}
for name, (cmd, flt) in IGNORED.items():
    claim(with_cmd(cmd, flt) == pcm(f"[0:a]{flt}"), f"D  `{name}`: the command is silently a no-op")
agate_static = pcm("[0:a]agate=threshold=0.5") != pcm("[0:a]agate=threshold=0.001")
claim(agate_static, "D  agate threshold 0.5 vs 0.001 does differ when set at init (so D's no-op is the command)")
claim(pcm("[0:a]deesser") == pcm("[0:a]anull"), "D  deesser at its defaults (i=0) passes the input through unchanged")

# ---------------------------------------------------------------- E. command granularity
CONST = "aevalsrc=0.5:s=48000:d=2"
for pre, want in (("", 48128), ("asetnsamples=n=480,", 48000), ("asetnsamples=n=1,", 48000)):
    raw = pcm(f"[0:a]{pre}asendcmd=c='1.0 volume@v volume 0.25',volume@v=volume=1:eval=frame",
              inputs=(), lavfi=CONST)
    x = frames(raw)[:, 0]
    first = int(np.flatnonzero(x < 0.5 * x[0])[0])  # the gain drops to a quarter
    claim(first == want, f"E  a volume command at 1.000 s with `{pre or '(1024-sample frames)'}` "
                         f"first lands at sample {first} (claimed {want})")

x = frames(pcm("[0:a]afade=t=in:st=1:d=0.5,afade=t=out:st=2:d=0.5", inputs=(),
               lavfi="aevalsrc=0.5:s=48000:d=3"))[:, 0]
nz, full = np.flatnonzero(x > 0), np.flatnonzero(np.abs(x - x.max()) < 1e-7)
got = (int(nz[0]), int(full[0]), int(full[-1]), int(nz[-1]))
claim(got == (48001, 72000, 96000, 119999),
      f"E  afade st=1:d=0.5 in / st=2:d=0.5 out is sample-exact on the 48 kHz grid (got {got})")

# ---------------------------------------------------------------- F. pitch/tempo onsets
BURST = "aevalsrc=exprs='if(between(t,0.5,2.5-1/48000),0.4*sin(2*PI*440*(t-0.5)),0)':s=48000:d=4"


def rise(g):
    x = np.abs(frames(pcm(f"[0:a]{g}", inputs=(), lavfi=BURST))[:, 0])
    e = np.convolve(x, np.ones(480) / 480, mode="same")
    half = 0.5 * np.median(e[e > 0.5 * e.max()])
    return int(np.flatnonzero(e > half)[0])


r0 = rise("anull")
STRETCH = {"atempo=1.25": 1 / 1.25, "atempo=0.8": 1 / 0.8,
           "asetrate=60476.21,aresample=48000,atempo=0.793701": 1.0}
if HAVE_RB:
    STRETCH.update({"rubberband=tempo=1.25": 1 / 1.25, "rubberband=pitch=1.259921": 1.0})
for g, k in STRETCH.items():
    off = rise(g) - round(r0 * k)
    claim(abs(off) > 100, f"F  {g}: burst onset lands {off:+d} samples from an exact stretch (claimed |off| > 100)")

# ---------------------------------------------------------------- G. SIMD report
print("info  outputs that change with -cpuflags 0 (SIMD-dependent bytes): "
      + (", ".join(SIMD_SENSITIVE) or "none"))

shutil.rmtree(TMP, ignore_errors=True)
print(f"\n{FF}: {ff(['-version']).stdout.decode().splitlines()[0]}")
if FAIL:
    print(f"{len(FAIL)} claim(s) no longer hold")
    sys.exit(1)
print("all claims hold")
