#!/usr/bin/env python3
"""PROTOTYPE, throwaway (#818): compressor and per-element limiter as `audio_effects` members.
Usage: python3 -I prototype_dynamics.py [--ffmpeg PATH] [--seed N]   (stdlib + ffmpeg only)

  {"name":"compressor","threshold_db":-28,"ratio":6,"attack_ms":5,"release_ms":120,"makeup_db":8}
  {"name":"limiter","ceiling_db":-12,"release_ms":50}
Render: acompressor (detection=rms, knee=1 hard; threshold/makeup converted dB->linear) and
alimiter (level=0, latency=1, attack fixed 5 ms; limit converted). Neither is singular.
Checks: static transfer on a steady sine, limiter peak and sample alignment on PCM, bypass bytes.
A/B pairs (blind, loudness-matched, AAC 160k): compressor on the voice; limiter on a voice at volume 2.
"""
import argparse, array, json, random, re, subprocess, sys, tempfile
from pathlib import Path
HERE = Path(__file__).resolve().parent
FIX = HERE.parent / "fixtures" / "narration-over-bed"
RATE, MATCH = 48000, -20.0
lin = lambda db: 10 ** (db / 20)

def mf(m):
    if m["name"] == "compressor":
        return (f"acompressor=threshold={lin(m['threshold_db']):.6f}:ratio={m['ratio']}:attack={m['attack_ms']}:"
                f"release={m['release_ms']}:makeup={lin(m['makeup_db']):.6f}:knee=1:detection=rms")
    if m["name"] == "limiter":
        return f"alimiter=limit={lin(m['ceiling_db']):.6f}:attack=5:release={m['release_ms']}:level=0:latency=1"
    sys.exit("unknown member")
chain = lambda ms: ",".join(mf(m) for m in ms if m.get("enabled", True)) or "anull"

def ff(F, args):
    p = subprocess.run([F, "-hide_banner", "-nostdin", *args], capture_output=True, text=True)
    if p.returncode: sys.exit(p.stderr[-1500:])
    return p.stderr
def lufs(F, p):
    t = ff(F, ["-i", str(p), "-af", "ebur128", "-f", "null", "-"]); t = t[t.rindex("Summary:"):]
    return float(re.search(r"I:\s+(-?[\d.]+)\s+LUFS", t).group(1))
def rms_peak(F, p, af):
    t = ff(F, ["-i", str(p), "-af", af + ",astats=measure_overall=RMS_level+Peak_level:measure_perchannel=none", "-f", "null", "-"])
    return float(re.findall(r"RMS level dB:\s+(-?[\d.]+)", t)[-1]), float(re.findall(r"Peak level dB:\s+(-?[\d.]+)", t)[-1])
def pcm(F, src, af):
    r = subprocess.run([F, "-hide_banner", "-nostdin", "-i", str(src), "-af", af, "-ac", "1", "-f", "f32le", "-"], capture_output=True)
    a = array.array("f"); a.frombytes(r.stdout); return a

def main():
    ap = argparse.ArgumentParser(); ap.add_argument("--ffmpeg", default="ffmpeg"); ap.add_argument("--seed", type=int, default=818)
    a = ap.parse_args(); F = a.ffmpeg
    comp = {"name": "compressor", "threshold_db": -28, "ratio": 6, "attack_ms": 5, "release_ms": 120, "makeup_db": 8}
    lim = {"name": "limiter", "ceiling_db": -12, "release_ms": 50}
    fails, res = [], {"ffmpeg": subprocess.run([F, "-version"], capture_output=True, text=True).stdout.splitlines()[0],
                      "members": {"compressor": comp, "limiter": lim}}
    with tempfile.TemporaryDirectory() as w:
        w = Path(w)
        def sine(f, db, d=3):
            p = w / f"s{f}_{db}.wav"
            ff(F, ["-y", "-f", "lavfi", "-i", f"sine=f={f}:d={d}:r={RATE}", "-af", f"volume={db}dB", "-ac", "2", "-c:a", "pcm_f32le", str(p)]); return p
        # sine=... default amplitude is 1/8 (-18.06 dBFS peak); volume lifts it to the wanted PEAK level
        def peak_sine(db): return sine(1000, db + 18.0618)
        rows = []
        for thr, ratio, mk in ((-20, 4, 0), (-20, 4, 6), (-30, 8, 0), (-20, 1, 0)):
            m = {"name": "compressor", "threshold_db": thr, "ratio": ratio, "attack_ms": 1, "release_ms": 50, "makeup_db": mk}
            row = {"member": f"threshold {thr} ratio {ratio} makeup {mk}", "gain_db_at_peak_in": {}}
            for pk in (-40, -10, -3):
                s = peak_sine(pk)
                # steady state: last 1 s
                o, _ = rms_peak(F, s, f"{chain([m])},atrim=start=2")
                i, _ = rms_peak(F, s, "atrim=start=2")
                got = round(o - i, 2)
                lvl = pk - 3.0103  # RMS of a sine, what the detector reads
                exp = mk - (lvl - thr) * (1 - 1 / ratio) if lvl > thr else mk
                row["gain_db_at_peak_in"][pk] = {"got": got, "expected": round(exp, 2)}
                if abs(got - exp) > 0.5: fails.append(f"comp {row['member']} @ {pk}: {got} vs {exp}")
            rows.append(row)
        res["compressor_static_transfer"] = rows
        # limiter: peak at/below ceiling on PCM, and sample alignment of an onset
        lrows = []
        for ceil in (-3, -12):
            m = {"name": "limiter", "ceiling_db": ceil, "release_ms": 50}
            s = peak_sine(6)  # +6 dBFS peak in float
            _, pk = rms_peak(F, s, f"{chain([m])},atrim=start=0.5")
            lrows.append({"ceiling_db": ceil, "output_peak_db_after_0.5s": pk})
            if pk > ceil + 0.1: fails.append(f"limiter {ceil}: peak {pk}")
        res["limiter_peak"] = lrows
        burst = w / "burst.wav"   # silence, then a 20 ms 1 kHz burst at +0 dBFS starting exactly at 1.000 s
        ff(F, ["-y", "-f", "lavfi", "-i", f"sine=f=1000:d=0.02:r={RATE}", "-af", f"volume=18.0618dB,adelay=1000:all=1,apad=whole_dur=2", "-ac", "1", "-c:a", "pcm_f32le", str(burst)])
        for name, m in (("limiter", lim), ("compressor", comp)):
            x = pcm(F, burst, "anull"); y = pcm(F, burst, chain([m]))
            on = lambda v: next(i for i, s in enumerate(v) if abs(s) > 0.01)
            res[f"{name}_onset_shift_samples"] = on(y) - on(x)
            if on(y) != on(x): fails.append(f"{name} onset moved {on(y) - on(x)}")
        # bypass identity
        n = FIX / "narration.flac"; r0, r1 = w / "r0.wav", w / "r1.wav"
        ff(F, ["-y", "-i", str(n), "-af", "anull", "-c:a", "pcm_f32le", str(r0)])
        ff(F, ["-y", "-i", str(n), "-af", chain([{**comp, "enabled": False}, {**lim, "enabled": False}]), "-c:a", "pcm_f32le", str(r1)])
        res["disabled_members_byte_identical"] = r0.read_bytes() == r1.read_bytes()
        if not res["disabled_members_byte_identical"]: fails.append("bypass bytes differ")
        # A/B
        out = HERE / "ab"; out.mkdir(exist_ok=True); key = []
        def mix(vfx, vol_db, p):
            ff(F, ["-y", "-i", str(FIX / "narration.flac"), "-i", str(FIX / "bed.flac"), "-filter_complex",
                   f"[0:a]aformat=sample_rates={RATE}:channel_layouts=stereo,{vfx},volume={vol_db}dB[v];[1:a]aformat=sample_rates={RATE}:channel_layouts=stereo[b];[v][b]amix=inputs=2:normalize=0[m]",
                   "-map", "[m]", "-c:a", "pcm_f32le", str(p)])
        res["ab"] = {}
        for pair, on_fx, vol, seed in (("compressor", chain([comp]), 0, a.seed), ("limiter", chain([lim]), 6, a.seed + 1)):
            A, B = w / f"{pair}-bypass.wav", w / f"{pair}-on.wav"
            mix("anull", vol, A); mix(on_fx, vol, B)
            raw = {"bypass": A, "on": B}
            order = ["bypass", "on"]; random.Random(seed).shuffle(order)
            info = {"lufs_before_match": {k: lufs(F, v) for k, v in raw.items()}}
            for lab, k in zip("XY", order):
                g = MATCH - lufs(F, raw[k])
                dst = out / f"{pair}-{lab}.m4a"
                ff(F, ["-y", "-i", str(raw[k]), "-af", f"volume={g:.5f}dB", "-c:a", "aac", "-b:a", "160k", str(dst)])
                key.append(f"{pair}-{lab} = {k}")
            info["peak_db_before_match"] = {k: rms_peak(F, v, "anull")[1] for k, v in raw.items()}
            res["ab"][pair] = info
        (out / "KEY").write_text("\n".join(key) + "\n")
    (HERE / "measurements.json").write_text(json.dumps(res, indent=2) + "\n")
    print(json.dumps(res, indent=2))
    if fails: sys.exit("FAILED: " + "; ".join(fails))
main()
