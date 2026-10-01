#!/usr/bin/env python3
"""Measure which ffmpeg facts about speech timing catch brief B's late words file (#561).

Run from the repo root, with any ffmpeg on PATH or named by --ffmpeg:

    python3 docs/research/speech-timing-ffmpeg-facts/measure.py [--ffmpeg PATH] [--json OUT]

Standard library only. It reads the presenter take and the eval runs' project files, runs
ffmpeg's own filters on the take's audio, and prints:

  1. the decoded audio's MD5 and a SHA-256 of every filter's raw output, so two ffmpeg
     builds can be compared byte for byte;
  2. each signal's pauses (or onsets) around brief B's "Now";
  3. for every project with highlight windows on the presenter's words, how often each
     candidate fact fires, split into the late projects (no-skills runs, which copied
     take-1.words.json) and the repaired ones (with-skills runs).

No aligner or speech model is run or read: the only inputs are the media, the words file
and the projects.
"""

import argparse
import hashlib
import json
import re
import subprocess
import sys
from fractions import Fraction
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
EVAL = ROOT / "docs/research/skills-eval"
VOICE_REL = "presenter/take-1.mp4"
VOICE = EVAL / "assets" / VOICE_REL
WORDS = EVAL / "assets/presenter/take-1.words.json"

# Every eval run whose project has highlight windows on the presenter's words.
LATE = [
    "baseline/B-talking-head/no-skills-1/workspace/social.montagent.json",
    "baseline/B-talking-head/no-skills-3/workspace/social.json",
    "baseline/B-talking-head/no-skills-4/workspace/social.json",
    "baseline-superseded/B-talking-head/no-skills-1/workspace/social.json",
    "baseline-superseded/B-talking-head/no-skills-2/workspace/social.montagent.json",
    "baseline-superseded/B-talking-head/no-skills-2-shared-tmp/workspace/social.montagent.json",
    "baseline-superseded/B-talking-head/no-skills-3/workspace/social.montagent.json",
]
CORRECT = [
    "dev/B-talking-head/with-skills-1/workspace/social.montagent.json",
    "dev/B-talking-head/with-skills-2/workspace/project.json",
]

# Detector parameters swept. The dB levels and durations are borrowed numbers (see the note).
SILENCEDETECT = [(n, d) for n in (-30, -35, -40, -45, -50) for d in (0.05, 0.1, 0.15)]
SILENCEREMOVE = [(-35, 0.1), (-40, 0.1), (-45, 0.1)]
ASTATS = [(-35, 0.1), (-40, 0.1), (-45, 0.1)]          # 10 ms RMS blocks: (level dB, min pause s)
EBUR128 = [(-40, 0.1), (-50, 0.1), (-60, 0.1)]         # momentary loudness (LUFS), min pause s


def ff(ffmpeg, args):
    out = subprocess.run([ffmpeg, "-hide_banner", "-nostats", *args],
                         capture_output=True, text=True)
    if out.returncode != 0:
        sys.exit(f"ffmpeg failed ({out.returncode}): {' '.join(args)}\n{out.stderr[-2000:]}")
    return out


def norm_log(text):
    """Keep only what follows the log prefix: ffmpeg 9 names the filter instance
    differently ("[silencedetect @ 0x...]" against 7.1's "[Parsed_silencedetect_0 @ 0x...]")."""
    return re.sub(r"^\[[^]]*\] ", "", text, flags=re.M)


def sha(text):
    return hashlib.sha256(text.encode()).hexdigest()[:16]


# ---- signals: each returns (pauses as [(a_ms, b_ms)], raw output text) ----------------

def sig_silencedetect(ffmpeg, n, d, media=VOICE):
    log = ff(ffmpeg, ["-i", str(media), "-vn", "-af", f"silencedetect=n={n}dB:d={d}",
                      "-f", "null", "-"]).stderr
    lines = [l for l in norm_log(log).splitlines() if "silence_" in l]
    starts = [float(x) * 1000 for x in re.findall(r"silence_start: (-?[\d.]+)", log)]
    ends = [float(x) * 1000 for x in re.findall(r"silence_end: ([\d.]+)", log)]
    return list(zip(starts, ends)), "\n".join(lines)


def metadata_print(ffmpeg, chain, tmp):
    ff(ffmpeg, ["-i", str(VOICE), "-vn", "-af", f"{chain},ametadata=mode=print:file={tmp}",
                "-f", "null", "-"])
    text = Path(tmp).read_text()
    frames = []
    for line in text.splitlines():
        m = re.match(r"frame:\d+\s+pts:(\d+)\s+pts_time:([\d.]+)", line)
        if m:
            frames.append({"t": float(m.group(2)) * 1000})
        elif "=" in line and frames:
            k, v = line.split("=", 1)
            frames[-1][k] = float(v)
    return frames, text


def runs_below(frames, key, level, block_ms, min_ms, lag_ms=0.0):
    """Runs of consecutive blocks whose `key` is below `level`, at least `min_ms` long."""
    out, a = [], None
    for f in frames:
        quiet = f[key] < level
        t0 = f["t"] - lag_ms
        if quiet and a is None:
            a = t0
        if not quiet and a is not None:
            if t0 - a >= min_ms:
                out.append((a, t0))
            a = None
    if a is not None:
        end = frames[-1]["t"] + block_ms - lag_ms
        if end - a >= min_ms:
            out.append((a, end))
    return out


def sig_astats(ffmpeg, level, min_s, tmp):
    frames, raw = metadata_print(
        ffmpeg, "asetnsamples=n=160:p=0,astats=metadata=1:reset=1", tmp)
    key = "lavfi.astats.Overall.RMS_level"
    return runs_below(frames, key, level, 10, min_s * 1000), raw


def sig_ebur128(ffmpeg, level, min_s, tmp):
    # metadata=1 cuts the input into 100 ms frames; M on the frame at t covers [t-300, t+100).
    frames, raw = metadata_print(ffmpeg, "ebur128=metadata=1:framelog=quiet", tmp)
    for f in frames:
        f["t"] += 100  # report the instant M was computed (end of its block)
    pauses = runs_below(frames, "lavfi.r128.M", level, 100, min_s * 1000)
    return pauses, raw


def sig_silenceremove(ffmpeg, level, d):
    # 1 ms frames, timestamps copied, so a kept frame's pts says where audio was kept.
    out = ff(ffmpeg, ["-i", str(VOICE), "-vn", "-af",
                      f"asetnsamples=n=16:p=0,silenceremove=stop_periods=-1:stop_threshold={level}dB"
                      f":stop_duration={d}:detection=rms:window=0.02:timestamp=copy",
                      "-c:a", "pcm_f32le", "-f", "framecrc", "-"]).stdout
    kept = []
    for line in out.splitlines():
        if line.startswith("#"):
            continue
        p = [x.strip() for x in line.split(",")]
        pts, dur = int(p[1]), int(p[3])
        kept.append((pts / 16.0, (pts + dur) / 16.0))
    removed = []
    for (a0, a1), (b0, _) in zip(kept, kept[1:]):
        if b0 > a1 + 0.5:
            removed.append((a1, b0))
    return removed, "\n".join(l for l in out.splitlines() if not l.startswith("#"))


def flux_frames(ffmpeg, tmp):
    frames, raw = metadata_print(
        ffmpeg, "asetnsamples=n=256:p=0,aspectralstats=win_size=512:overlap=0.5:measure=flux", tmp)
    return frames, raw


# ---- projects -------------------------------------------------------------------------

def windows_of(rel, need_voice=True):
    d = json.loads((EVAL / "runs" / rel).read_text())
    fps = Fraction(d["fps"]).limit_denominator(1001)
    voice = None
    for t in d["tracks"]:
        for e in t["elements"]:
            if e.get("type") == "video" and e.get("source") == VOICE_REL and e.get("volume", 1) != 0:
                voice = voice or e
    # The voice must map document time to source time one to one.
    if need_voice:
        assert voice and voice["start"] == voice.get("source_start", 0) == 0 and not voice.get("speed"), rel
    wins = []
    for t in d["tracks"]:
        for e in t["elements"]:
            for r in e.get("runs") or []:
                if isinstance(r, dict) and "highlight" in r:
                    wins.append((r["text"].strip(), r["highlight"]["start"], r["highlight"]["end"]))
    return sorted(wins, key=lambda w: w[1]), fps


BEDS = [  # projects with highlight windows over a heard non-voice source (brief A: the bed only)
    "baseline/A-launch-spot/no-skills-3/workspace/launch.json",
    "baseline-superseded/A-launch-spot/no-skills-1/workspace/launch.montagent.json",
    "baseline-superseded/A-launch-spot/no-skills-2/workspace/spot.json",
    "baseline-superseded/A-launch-spot/no-skills-3/workspace/launch.montagent.json",
    "dev/A-launch-spot/with-skills-1/workspace/launch.json",
    "dev/A-launch-spot/with-skills-2/workspace/spot.json",
    "dev/A-launch-spot/with-skills-3/workspace/spot.montagent.json",
]


def has_audio(rel_source, _cache={}):
    if rel_source not in _cache:
        out = subprocess.run(["ffprobe", "-v", "error", "-select_streams", "a", "-show_entries",
                              "stream=index", "-of", "csv=p=0", str(EVAL / "assets" / rel_source)],
                             capture_output=True, text=True)
        _cache[rel_source] = bool(out.stdout.strip())
    return _cache[rel_source]


def heard_sources(rel):
    """Every element whose source has audio and whose volume is not 0 throughout.
    Volume envelopes are not applied: each source is read as it is on disk."""
    d = json.loads((EVAL / "runs" / rel).read_text())
    out = []
    for t in d["tracks"]:
        for e in t["elements"]:
            if e.get("type") not in ("audio", "video") or not has_audio(e["source"]):
                continue
            v = e.get("volume", 1)
            if (isinstance(v, list) and not any(k["v"] > 0 for k in v)) or v == 0:
                continue
            out.append(e)
    return out


def source_pauses_on_timeline(ffmpeg, e, n, d):
    """The element's source pauses, moved onto the document timeline and clipped to it."""
    pauses, _ = sig_silencedetect(ffmpeg, n, d, EVAL / "assets" / e["source"])
    s0, speed = e.get("source_start", 0), e.get("speed") or 1
    out = []
    for a, b in pauses:
        a2, b2 = e["start"] + (a - s0) / speed, e["start"] + (b - s0) / speed
        a2, b2 = max(a2, e["start"]), min(b2, e["end"])
        if a2 < b2:
            out.append((a2, b2))
    return out


def words_file_windows():
    return [(w["word"], w["start"], w["end"]) for w in json.loads(WORDS.read_text())], Fraction(30)


# ---- the candidate facts --------------------------------------------------------------

def facts(wins, fps, pauses):
    """Fire counts of each candidate fact for one project against one pause list."""
    step = Fraction(1000) / fps
    first, last = min(w[1] for w in wins), max(w[2] for w in wins)
    inner = [(a, b) for a, b in pauses if a > first and b < last]
    instants = [k * step for k in range(int(last / step) + 2)]

    def lit(t):
        return any(s <= t < e for _, s, e in wins)

    def in_pause(t):
        return any(a <= t < b for a, b in pauses)

    opens_in_pause = [w for w in wins if any(a < w[1] < b for a, b in pauses)]
    inside_pause = [w for w in wins if any(a <= w[1] and w[2] <= b for a, b in pauses)]
    head, tail, lit_quiet = [], [], []
    for a, b in inner:
        # head: painted instants from where speech resumes until a window is lit
        k = 0
        for t in instants:
            if t < b:
                continue
            if lit(t) or in_pause(t):
                break
            k += 1
        if k:
            head.append((round(b, 1), k))
        # tail: painted instants of speech before the pause, after the last lit instant
        k = 0
        for t in reversed(instants):
            if t >= a:
                continue
            if lit(t) or in_pause(t):
                break
            k += 1
        if k:
            tail.append((round(a, 1), k))
        n = sum(1 for t in instants if a <= t < b and lit(t))
        if n:
            lit_quiet.append(((round(a, 1), round(b, 1)), n))
    return {
        "windows": len(wins), "inner_pauses": len(inner),
        "opens_in_pause": len(opens_in_pause), "inside_pause": len(inside_pause),
        "head_unlit": head, "tail_unlit": tail, "lit_in_pause": lit_quiet,
    }


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--ffmpeg", default="ffmpeg")
    ap.add_argument("--json")
    ap.add_argument("--tmp", default="/tmp/speech-timing-ffmpeg-facts.txt")
    args = ap.parse_args()
    F = args.ffmpeg
    report = {"ffmpeg": ff(F, ["-version"]).stdout.splitlines()[0]}
    pcm = ff(F, ["-i", str(VOICE), "-map", "0:a", "-c:a", "pcm_f32le", "-f", "md5", "-"]).stdout.strip()
    report["decoded_pcm_f32le"] = pcm
    print(report["ffmpeg"])
    print("decoded audio", pcm)

    signals, raw_hash = {}, {}
    for n, d in SILENCEDETECT:
        p, raw = sig_silencedetect(F, n, d)
        signals[f"silencedetect n={n}dB d={d}"] = p
        raw_hash[f"silencedetect n={n}dB d={d}"] = sha(raw)
    for lv, d in SILENCEREMOVE:
        p, raw = sig_silenceremove(F, lv, d)
        signals[f"silenceremove rms thr={lv}dB d={d}"] = p
        raw_hash[f"silenceremove rms thr={lv}dB d={d}"] = sha(raw)
    for lv, d in ASTATS:
        p, raw = sig_astats(F, lv, d, args.tmp)
        signals[f"astats RMS/10ms <{lv}dB >={d}s"] = p
        raw_hash["astats RMS/10ms"] = sha(raw)
    for lv, d in EBUR128:
        p, raw = sig_ebur128(F, lv, d, args.tmp)
        signals[f"ebur128 M <{lv}LUFS >={d}s"] = p
        raw_hash["ebur128 M"] = sha(raw)
    flux, raw = flux_frames(F, args.tmp)
    raw_hash["aspectralstats flux"] = sha(raw)
    report["raw_output_sha256_16"] = raw_hash

    print("\n== raw output hashes (compare across ffmpeg builds)")
    for k, v in raw_hash.items():
        print(f"  {v}  {k}")

    print("\n== pauses (ms) per signal; the one around 'Now' is marked *")
    report["pauses"] = {}
    for k, p in signals.items():
        report["pauses"][k] = [(round(a, 1), round(b, 1)) for a, b in p]
        cells = " ".join(("*" if a < 2420 and b > 1800 and a < 2300 else "") +
                         f"{a:.1f}-{b:.1f}" for a, b in p)
        print(f"  {k:34} {cells}")

    # aspectralstats: where is spectral flux largest between the first sentence's end and "Now"?
    near = [f for f in flux if 1700 <= f["t"] <= 2600]
    top = sorted(near, key=lambda f: -f["lavfi.aspectralstats.1.flux"])[:5]
    report["flux_top5_1700_2600"] = [(round(f["t"]), f["lavfi.aspectralstats.1.flux"]) for f in top]
    print("\n== aspectralstats flux, 5 largest hops in 1700-2600 ms:",
          ", ".join(f"{t:.0f} ms ({v:.3g})" for t, v in report["flux_top5_1700_2600"]))

    projects = {"take-1.words.json": ("late", *words_file_windows())}
    for rel in LATE:
        projects[rel] = ("late", *windows_of(rel))
    for rel in CORRECT:
        projects[rel] = ("correct", *windows_of(rel))
    print("\n== 'Now' window per project")
    for name, (kind, wins, fps) in projects.items():
        now = next(w for w in wins if w[0].lower().startswith("now"))
        print(f"  {kind:7} {now[1]:>5}-{now[2]:<5} {name}")

    # "late" = the words file and the seven no-skills projects. One of them
    # (baseline-superseded no-skills-1) moved "Now" to 2300 itself; see the 'Now' table.
    print("\n== fires per signal: words file + no-skills projects (8) | with-skills projects (2)")
    print("   fact columns: opens-in-pause, inside-pause, head-unlit, tail-unlit, lit-in-pause")
    report["fires"] = {}
    for k, p in signals.items():
        row = {}
        for name, (kind, wins, fps) in projects.items():
            row[name] = (kind, facts(wins, fps, p))
        report["fires"][k] = row

        def agg(kind):
            rs = [f for kd, f in row.values() if kd == kind]
            return (sum(r["opens_in_pause"] for r in rs), sum(r["inside_pause"] for r in rs),
                    sum(len(r["head_unlit"]) for r in rs), sum(len(r["tail_unlit"]) for r in rs),
                    sum(len(r["lit_in_pause"]) for r in rs),
                    sum(1 for r in rs if r["head_unlit"]), len(rs),
                    sum(r["windows"] for r in rs), sum(r["inner_pauses"] for r in rs))
        L, C = agg("late"), agg("correct")
        print(f"  {k:34} late {L[:5]} (head fires in {L[5]}/{L[6]} projects; {L[8]} pauses, {L[7]} windows)"
              f" | correct {C[:5]} ({C[8]} pauses, {C[7]} windows)")

    print("\n== detail at silencedetect n=-35dB d=0.15 and n=-40dB d=0.1")
    for k in ("silencedetect n=-35dB d=0.15", "silencedetect n=-40dB d=0.1"):
        for name, (kind, f) in report["fires"][k].items():
            print(f"  {k} | {kind:7} {name}\n     head {f['head_unlit']} tail {f['tail_unlit']}"
                  f" lit-in-pause {f['lit_in_pause']} opens-in-pause {f['opens_in_pause']}")

    # Per heard source: the same facts, stated against every other heard source under the
    # windows (here, the music bed), reported apart from the voice. No source is classified.
    print("\n== per heard source other than the presenter take (pause facts against the same windows)")
    report["other_sources"] = {}
    for rel in LATE + CORRECT + BEDS:
        wins, fps = windows_of(rel, need_voice=rel not in BEDS)
        for e in heard_sources(rel):
            if e["source"] == VOICE_REL:
                continue
            for n, d in ((-30, 0.05), (-30, 0.1), (-35, 0.05), (-35, 0.15), (-40, 0.05), (-40, 0.1)):
                p = source_pauses_on_timeline(F, e, n, d)
                f = facts(wins, fps, p)
                key = f"{rel} | {e['source']} | n={n} d={d}"
                report["other_sources"][key] = f
                print(f"  {key}\n     windows {f['windows']} inner pauses {f['inner_pauses']}"
                      f" opens-in-pause {f['opens_in_pause']} inside {f['inside_pause']}"
                      f" head {f['head_unlit']} tail {f['tail_unlit']} lit-in-pause {f['lit_in_pause']}")

    # The pack's other words files: no project uses them, and nothing says they are correct.
    # For each pause, where the next word starts against where the sound resumes (ms).
    print("\n== other words files in the pack: next word start minus pause end (ms)")
    report["other_words_files"] = {}
    for media, words in [("presenter/take-1.mp4", "presenter/take-1.words.json"),
                         ("presenter/take-2.mp4", "presenter/take-2.words.json"),
                         ("presenter/take-3.mp4", "presenter/take-3.words.json"),
                         ("voiceover/voiceover.wav", "voiceover/voiceover.words.json")]:
        ws = json.loads((EVAL / "assets" / words).read_text())
        for n, d in ((-35, 0.15), (-40, 0.1)):
            pauses, _ = sig_silencedetect(F, n, d, EVAL / "assets" / media)
            gaps = []
            for a, b in pauses:
                nxt = [w for w in ws if w["start"] >= a]
                if nxt and a > ws[0]["start"]:
                    gaps.append((nxt[0]["word"], round(nxt[0]["start"] - b, 1)))
            report["other_words_files"][f"{words} n={n} d={d}"] = gaps
            print(f"  {words:32} n={n}dB d={d}: " + ", ".join(f"{w} {g:+.0f}" for w, g in gaps))

    if args.json:
        Path(args.json).write_text(json.dumps(report, indent=1, default=str))


if __name__ == "__main__":
    main()
