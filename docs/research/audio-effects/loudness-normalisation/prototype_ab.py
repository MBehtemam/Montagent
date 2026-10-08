#!/usr/bin/env python3
"""PROTOTYPE, throwaway (#816): per-element loudness normalisation as an `audio_effects` member.

Usage:  python3 -I prototype_ab.py [--ffmpeg PATH] [--seed N] [--out DIR]

Stdlib + ffmpeg only. It hand-builds the mix graph of crates/montagent-core/src/verbs/render.rs
(aformat, atrim, atempo, aloop, atrim, [member], volume, atrim, adelay, amix normalize=0,
apad/atrim) with the proposed member spliced in where ADR-0169 puts the list, and renders:

  document A: the member `enabled: false` (bypass)  -- three elements as they come
  document B: the member on each element            -- voice lines to VOICE_LUFS, bed to BED_LUFS

The member, as prototyped:  {"name": "normalize_loudness", "target_lufs": -23}
  1. render the element's chain up to the member's slot (its own window: trimmed, sped, looped),
  2. measure its integrated loudness with ebur128 (BS.1770, gated),
  3. apply ONE fixed gain  volume=<target - measured>dB  (never loudnorm).
  Undefined loudness (silent or shorter than one 400 ms block) -> no gain, a finding.

The two mixes are then brought to the same integrated loudness by one gain (ADR-0173 §8), run
through AAC 160k (the shipped encoder settings), and written as ab/X.m4a and ab/Y.m4a with the
assignment shuffled by --seed into ab/KEY. Numbers (per-element before/after, the mix, the edge
cases) go to measurements.json. Exit 1 if a number stops holding.
"""
import argparse, json, random, re, subprocess, sys, tempfile, hashlib
from pathlib import Path

HERE = Path(__file__).resolve().parent
FIX = HERE.parent / "fixtures" / "narration-over-bed"
RATE = 48000
VOICE_LUFS = -23.0   # Premiere Auto-Match "Dialogue" (PRECEDENT.md §1)
BED_LUFS = -38.0     # 15 LU under the voice, as the fixture's own bed sits
MATCH_LUFS = -20.0   # both mixes are brought here for the listen
TOL_LU = 0.1         # ebur128 resolution; the element lands on target to this
SPLIT = 13.7         # a silence in the narration (13.50-13.94 s)
LINE2_DB = -9.0      # line 2 arrives quieter, as a second TTS call does


def run(ff, args, capture=True):
    p = subprocess.run([ff, "-hide_banner", "-nostdin", *args], capture_output=True, text=True)
    if p.returncode:
        sys.exit(f"ffmpeg failed: {' '.join(args)}\n{p.stderr[-1500:]}")
    return p.stderr


def lufs(ff, path, extra=""):
    """Integrated loudness and true peak via ebur128 (ADR-0173 §2). None when undefined."""
    err = run(ff, ["-i", str(path), "-af", "ebur128=peak=true", "-f", "null", "-"])
    tail = err[err.rindex("Summary:"):]
    i = re.search(r"I:\s+(-?[\d.]+|-inf)\s+LUFS", tail)
    p = re.search(r"Peak:\s+(-?[\d.]+|-inf)\s+dBFS", tail)
    if not i or not p:
        sys.exit("ebur128 parse broke:\n" + tail)
    f = lambda s: None if s == "-inf" else float(s)
    v = f(i.group(1))
    return (None if v is None or v <= -70.0 else v), f(p.group(1))


def chain(src, idx, s_start, s_end, speed=1.0, loop_ms=None, dur=None):
    """The render.rs chain up to (not including) `volume`: the member's slot."""
    f = (f"[{idx}:a]aformat=sample_rates={RATE}:channel_layouts=stereo,"
         f"atrim=start={s_start:.3f}:end={s_end:.3f},asetpts=PTS-STARTPTS")
    if speed != 1.0:
        f += f",atempo={speed}"
    if loop_ms:
        f += f",aloop=loop=-1:size={int(loop_ms * RATE / 1000)}"
    if dur is not None:
        f += f",atrim=end={dur:.3f},asetpts=PTS-STARTPTS"
    return f


def member_gain(ff, inputs, f_chain, target, work, name):
    """Render the chain to float PCM, measure it, return (measured, gain_db or None)."""
    wav = work / f"m-{name}.wav"
    args = []
    for p in inputs:
        args += ["-i", str(p)]
    run(ff, [*args, "-y", "-filter_complex", f_chain + "[o]", "-map", "[o]", "-c:a", "pcm_f32le", str(wav)])
    measured, _ = lufs(ff, wav)
    return measured, (None if measured is None else target - measured), wav


def render_mix(ff, inputs, elements, gains, total, out_wav):
    """elements: [(f_chain, delay_ms)]. gains: dB or None per element. Sum, pad, cut (mix_graph)."""
    parts, labels = [], ""
    for k, ((f_chain, delay), g) in enumerate(zip(elements, gains)):
        vol = f",volume={g:.6f}dB" if g is not None else ""
        parts.append(f"{f_chain}{vol},adelay=delays={delay}:all=1[a{k}]")
        labels += f"[a{k}]"
    graph = ";".join(parts) + f";{labels}amix=inputs={len(elements)}:normalize=0,apad=whole_dur={total:.3f},atrim=end={total:.3f}[mix]"
    args = []
    for p in inputs:
        args += ["-i", str(p)]
    run(ff, [*args, "-y", "-filter_complex", graph, "-map", "[mix]", "-c:a", "pcm_f32le", str(out_wav)])


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--ffmpeg", default="ffmpeg")
    ap.add_argument("--seed", type=int, default=816)
    ap.add_argument("--out", default=str(HERE))
    a = ap.parse_args()
    ff, out = a.ffmpeg, Path(a.out)
    (out / "ab").mkdir(parents=True, exist_ok=True)
    fails, res = [], {"ffmpeg": subprocess.run([ff, "-version"], capture_output=True, text=True).stdout.splitlines()[0]}
    with tempfile.TemporaryDirectory() as w:
        work = Path(w)
        nar, bed = FIX / "narration.flac", FIX / "bed.flac"
        v1, v2 = work / "voice1.wav", work / "voice2.wav"
        run(ff, ["-y", "-i", str(nar), "-af", f"atrim=0:{SPLIT}", str(v1)])
        run(ff, ["-y", "-i", str(nar), "-af", f"atrim={SPLIT}:20,asetpts=PTS-STARTPTS,volume={LINE2_DB}dB", str(v2)])
        inputs = [v1, v2, bed]
        spec = [  # name, input index, src span (s), delay ms, target
            ("voice1", 0, (0.0, SPLIT), 0, VOICE_LUFS),
            ("voice2", 1, (0.0, 20 - SPLIT), int(SPLIT * 1000), VOICE_LUFS),
            ("bed", 2, (0.0, 20.0), 0, BED_LUFS),
        ]
        elements, gains_on, rows = [], [], []
        for name, idx, (s0, s1), delay, target in spec:
            fc = chain(None, idx, s0, s1, dur=s1 - s0)
            elements.append((fc, delay))
            measured, g, _ = member_gain(ff, inputs, fc, target, work, name)
            gains_on.append(g)
            rows.append({"element": name, "measured_lufs": measured, "target_lufs": target,
                         "applied_gain_db": None if g is None else round(g, 3)})
        # the element lands on its target once the gain is applied (the capability's own number)
        for (name, idx, (s0, s1), delay, target), g, (fc, _) in zip(spec, gains_on, elements):
            wav = work / f"{name}-after.wav"
            run(ff, ["-i", str(inputs[0]), "-i", str(inputs[1]), "-i", str(inputs[2]), "-y", "-filter_complex", f"{fc},volume={g:.6f}dB[o]", "-map", "[o]", "-c:a", "pcm_f32le", str(wav)])
            after, _ = lufs(ff, wav)
            row = next(r for r in rows if r["element"] == name)
            row["after_lufs"] = after
            if after is None or abs(after - target) > TOL_LU:
                fails.append(f"{name}: {after} vs target {target}")
        res["elements"] = rows

        mixes = {}
        for label, gains in (("A_bypass", [None] * 3), ("B_normalised", gains_on)):
            wav = work / f"mix-{label}.wav"
            render_mix(ff, inputs, elements, gains, 20.0, wav)
            mixes[label] = wav
        before = {k: lufs(ff, p) for k, p in mixes.items()}
        res["mix_before_match"] = {k: {"lufs": v[0], "peak_dbfs": v[1]} for k, v in before.items()}
        matched = {}
        for k, p in mixes.items():
            g = MATCH_LUFS - before[k][0]
            m = work / f"matched-{k}.wav"
            run(ff, ["-i", str(p), "-y", "-af", f"volume={g:.6f}dB", "-c:a", "pcm_f32le", str(m)])
            matched[k] = (m, g)
        res["match_gain_db"] = {k: round(g, 3) for k, (m, g) in matched.items()}
        order = ["A_bypass", "B_normalised"]
        random.Random(a.seed).shuffle(order)
        key = {"X": order[0], "Y": order[1]}
        res["after_match"] = {}
        for letter, k in key.items():
            m4a = out / "ab" / f"{letter}.m4a"
            run(ff, ["-i", str(matched[k][0]), "-y", "-c:a", "aac", "-b:a", "160k", "-ar", str(RATE), "-ac", "2", str(m4a)])
            dec = work / f"dec-{letter}.wav"
            run(ff, ["-i", str(m4a), "-y", str(dec)])
            i, p = lufs(ff, dec)
            res["after_match"][letter] = {"decoded_lufs": i, "decoded_peak_dbfs": p}
            if p is not None and p > -0.5:
                fails.append(f"{letter}: decoded peak {p} dBFS is clipping-close; lower MATCH_LUFS")
        if abs(res["after_match"]["X"]["decoded_lufs"] - res["after_match"]["Y"]["decoded_lufs"]) > 0.3:
            fails.append("the A/B pair is not loudness-matched within 0.3 LU")
        (out / "ab" / "KEY").write_text(json.dumps(key, indent=1) + "\n")
        res["key_sha256"] = hashlib.sha256((out / "ab" / "KEY").read_bytes()).hexdigest()

        # Edge cases: what the member does where there is nothing honest to measure.
        edge = {}
        silent = work / "silent.wav"
        run(ff, ["-y", "-f", "lavfi", "-i", "anullsrc=r=48000:cl=stereo", "-t", "2", str(silent)])
        edge["silent_2s"] = lufs(ff, silent)[0]                    # expect None: no gain, a finding
        short = work / "short.wav"
        run(ff, ["-y", "-f", "lavfi", "-i", "sine=f=1000:r=48000:d=0.3", "-af", "aformat=channel_layouts=stereo", str(short)])
        edge["tone_0.3s"] = lufs(ff, short)[0]                     # under one 400 ms block
        # speed: the window is measured AFTER atempo; a sped-up clip gets the gain of what is heard
        fc_s = chain(None, 0, 0.0, 6.0, speed=1.5, dur=4.0)
        edge["voice_at_1.5x_measured"] = member_gain(ff, [v1], fc_s, VOICE_LUFS, work, "spd")[0]
        edge["voice_at_1.0x_measured"] = member_gain(ff, [v1], chain(None, 0, 0.0, 6.0, dur=6.0), VOICE_LUFS, work, "nospd")[0]
        # loop: the played window (every iteration) is what is measured, not one source pass
        fc_l = chain(None, 0, 0.8, 3.0, loop_ms=2200, dur=9.0)
        edge["looped_window_measured"] = member_gain(ff, [v1], fc_l, VOICE_LUFS, work, "loop")[0]
        edge["one_pass_measured"] = member_gain(ff, [v1], chain(None, 0, 0.8, 3.0, dur=2.2), VOICE_LUFS, work, "once")[0]
        res["edge_cases"] = edge
        for k in ("silent_2s", "tone_0.3s"):
            if edge[k] is not None:
                fails.append(f"edge {k}: expected undefined loudness, got {edge[k]}")
    (out / "measurements.json").write_text(json.dumps(res, indent=1) + "\n")
    print(json.dumps(res, indent=1))
    if fails:
        print("\nFAIL:\n  " + "\n  ".join(fails), file=sys.stderr)
        sys.exit(1)


main()
