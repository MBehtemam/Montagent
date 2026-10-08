#!/usr/bin/env python3
"""PROTOTYPE, throwaway (#817): EQ as `audio_effects` members. Usage: python3 -I prototype_eq.py [--ffmpeg PATH] [--seed N]

Shape under test (several singly-purposed members; ADR-0169 lets any kind repeat, so stacking is the band list):
  {"name":"highpass","frequency_hz":100,"slope_db_per_oct":24}      12|24|48 (Butterworth, cascaded biquads)
  {"name":"lowpass","frequency_hz":8000,"slope_db_per_oct":12}
  {"name":"shelf","side":"low"|"high","frequency_hz":200,"gain_db":3}  (ffmpeg bass/treble, 2-pole, slope S=1)
  {"name":"bell","frequency_hz":1500,"gain_db":-6,"q":0.8}          (ffmpeg equalizer, t=q)
All render to the cleanest ffmpeg biquads (zero latency, identical on 6.1 and 7.1, FFMPEG-FILTERS.md 3.2).
"""
import argparse, json, math, random, re, subprocess, sys, tempfile
from pathlib import Path
HERE = Path(__file__).resolve().parent
FIX = HERE.parent / "fixtures" / "narration-over-bed"
RATE, MATCH = 48000, -20.0

def bw_qs(n_poles):  # Butterworth section Qs for an even order
    return [1/(2*math.cos(math.pi*(2*k-1)/(2*n_poles))) for k in range(1, n_poles//2+1)][::1]

def member_filter(m):
    n = m["name"]
    if n in ("highpass", "lowpass"):
        order = {12: 2, 24: 4, 48: 8}[m["slope_db_per_oct"]]
        f = m["frequency_hz"]
        return ",".join(f"{n}=f={f}:poles=2:width_type=q:w={q:.4f}" for q in bw_qs(order))
    if n == "shelf":
        flt = "bass" if m["side"] == "low" else "treble"
        return f"{flt}=g={m['gain_db']}:f={m['frequency_hz']}:width_type=s:w=1"
    if n == "bell":
        return f"equalizer=f={m['frequency_hz']}:width_type=q:w={m['q']}:g={m['gain_db']}"
    sys.exit("unknown member " + n)

def chain(members):
    return ",".join(member_filter(m) for m in members if m.get("enabled", True)) or "anull"

def ff(a, args):
    p = subprocess.run([a, "-hide_banner", "-nostdin", *args], capture_output=True, text=True)
    if p.returncode: sys.exit(p.stderr[-1500:])
    return p.stderr

def lufs(a, path):
    t = ff(a, ["-i", str(path), "-af", "ebur128=peak=true", "-f", "null", "-"])
    t = t[t.rindex("Summary:"):]
    return float(re.search(r"I:\s+(-?[\d.]+)\s+LUFS", t).group(1))

def rms_db(a, path, af):
    t = ff(a, ["-i", str(path), "-af", af + ",astats=measure_overall=RMS_level:measure_perchannel=none", "-f", "null", "-"])
    return float(re.findall(r"RMS level dB:\s+(-?[\d.]+)", t)[-1])

def main():
    ap = argparse.ArgumentParser(); ap.add_argument("--ffmpeg", default="ffmpeg"); ap.add_argument("--seed", type=int, default=817); ap.add_argument("--strong", action="store_true"); ap.add_argument("--ab", default="ab")
    a = ap.parse_args(); F = a.ffmpeg
    voice = [{"name": "highpass", "frequency_hz": 100, "slope_db_per_oct": 24}]
    bed = [{"name": "bell", "frequency_hz": 1500, "gain_db": -6, "q": 0.8},
           {"name": "shelf", "side": "high", "frequency_hz": 9000, "gain_db": -4}]
    if a.strong:
        voice = [{"name": "highpass", "frequency_hz": 400, "slope_db_per_oct": 24},
                 {"name": "shelf", "side": "high", "frequency_hz": 4000, "gain_db": 6}]
        bed = [{"name": "bell", "frequency_hz": 1500, "gain_db": -14, "q": 0.7},
               {"name": "lowpass", "frequency_hz": 5000, "slope_db_per_oct": 24}]
    fails, res = [], {"ffmpeg": subprocess.run([F, "-version"], capture_output=True, text=True).stdout.splitlines()[0],
                      "document": {"voice": voice, "bed": bed}}
    with tempfile.TemporaryDirectory() as w:
        w = Path(w)
        # --- measured check: gain at/around the cutoff on pure tones, vs bypass (lavfi)
        rows = []
        def tone(f): 
            p = w / f"t{f}.wav"; ff(F, ["-y", "-f", "lavfi", "-i", f"sine=f={f}:d=2:r={RATE}", "-ac", "2", "-c:a", "pcm_f32le", str(p)]); return p
        for slope in (12, 24, 48):
            hp = chain([{"name": "highpass", "frequency_hz": 1000, "slope_db_per_oct": slope}])
            at = {}
            for f in (125, 250, 500, 1000, 2000, 8000):
                t = tone(f); at[f] = round(rms_db(F, t, hp) - rms_db(F, t, "anull"), 2)
            rows.append({"member": f"highpass 1000 Hz {slope} dB/oct", "gain_db_at_hz": at})
            if abs(at[1000] + 3.01) > 0.3: fails.append(f"hp{slope} cutoff {at[1000]}")
            if abs(at[500] - (-slope)) > 1.5: fails.append(f"hp{slope} one octave down {at[500]}")
            if abs(at[8000]) > 0.2: fails.append(f"hp{slope} passband {at[8000]}")
        lp = chain([{"name": "lowpass", "frequency_hz": 1000, "slope_db_per_oct": 24}])
        at = {f: round(rms_db(F, tone(f), lp) - rms_db(F, tone(f), "anull"), 2) for f in (125, 1000, 2000, 4000)}
        rows.append({"member": "lowpass 1000 Hz 24 dB/oct", "gain_db_at_hz": at})
        if abs(at[1000] + 3.01) > 0.3 or abs(at[2000] + 24) > 1.5: fails.append(f"lp {at}")
        bl = chain([{"name": "bell", "frequency_hz": 1500, "gain_db": -6, "q": 0.8}])
        t = tone(1500); g = round(rms_db(F, t, bl) - rms_db(F, t, "anull"), 2)
        rows.append({"member": "bell 1500 Hz -6 dB q0.8", "gain_db_at_1500": g})
        if abs(g + 6) > 0.2: fails.append(f"bell {g}")
        sh = chain([{"name": "shelf", "side": "low", "frequency_hz": 200, "gain_db": 6}])
        t = tone(40); g = round(rms_db(F, t, sh) - rms_db(F, t, "anull"), 2)
        rows.append({"member": "low shelf 200 Hz +6", "gain_db_at_40": g})
        if abs(g - 6) > 0.5: fails.append(f"shelf {g}")
        # bypass: disabled member is byte-identical
        n = FIX / "narration.flac"
        r0, r1 = w / "r0.wav", w / "r1.wav"
        ff(F, ["-y", "-i", str(n), "-af", "anull", "-c:a", "pcm_f32le", str(r0)])
        ff(F, ["-y", "-i", str(n), "-af", chain([{**voice[0], "enabled": False}]), "-c:a", "pcm_f32le", str(r1)])
        res["disabled_member_byte_identical"] = r0.read_bytes() == r1.read_bytes()
        if not res["disabled_member_byte_identical"]: fails.append("bypass not identical")
        res["tone_checks"] = rows
        # --- A/B on the fixture stems (voice and bed separate, as the mix graph sees them)
        def mixwav(on, p):
            fv = chain(voice) if on else "anull"; fb = chain(bed) if on else "anull"
            ff(F, ["-y", "-i", str(FIX/"narration.flac"), "-i", str(FIX/"bed.flac"), "-filter_complex",
                   f"[0:a]aformat=sample_rates={RATE}:channel_layouts=stereo,{fv}[v];[1:a]aformat=sample_rates={RATE}:channel_layouts=stereo,{fb}[b];[v][b]amix=inputs=2:normalize=0[m]",
                   "-map", "[m]", "-c:a", "pcm_f32le", str(p)])
        A, B = w/"A.wav", w/"B.wav"; mixwav(False, A); mixwav(True, B)
        res["mix_lufs_before_match"] = {"bypass": lufs(F, A), "eq": lufs(F, B)}
        (HERE/a.ab).mkdir(exist_ok=True)
        order = ["bypass", "eq"]; random.Random(a.seed).shuffle(order)
        for lab, key, src in zip("XY", order, [A if o == "bypass" else B for o in order]):
            g = MATCH - lufs(F, src)
            ff(F, ["-y", "-i", str(src), "-af", f"volume={g:.5f}dB", "-c:a", "aac", "-b:a", "160k", str(HERE/a.ab/f"{lab}.m4a")])
        (HERE/a.ab/"KEY").write_text(f"X = {order[0]}\nY = {order[1]}\n")
        res["ab_decoded_lufs"] = {l: lufs(F, HERE/a.ab/f"{l}.m4a") for l in "XY"}
    (HERE/("measurements-strong.json" if a.strong else "measurements.json")).write_text(json.dumps(res, indent=2) + "\n")
    print(json.dumps(res, indent=2)); 
    if fails: sys.exit("FAILED: " + "; ".join(fails))
main()
