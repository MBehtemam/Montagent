#!/usr/bin/env python3
"""Evidence for ADR-0178: what `normalize_loudness` measures, and where loudness is undefined.

Uses ffmpeg's ebur128 (the same meter ADR-0172's master gain uses) on lavfi noise.
Exits non-zero when a number stops holding. Stdlib only. ffmpeg from $FFMPEG or PATH.
Writes measurements-check.json beside itself.
"""
import json, os, re, subprocess, sys

FFMPEG = os.environ.get("FFMPEG", "ffmpeg")
FAILS, ROWS = [], []


def integrated(src, af=""):
    """Integrated loudness of `src` after `af`, parsed from ebur128's final summary. None if -inf."""
    chain = (af + "," if af else "") + "ebur128=framelog=quiet"
    err = subprocess.run([FFMPEG, "-hide_banner", "-nostats", "-f", "lavfi", "-i", src, "-af", chain, "-f", "null", "-"],
                         capture_output=True, text=True).stderr
    m = re.search(r"Integrated loudness:\s*\n\s*I:\s*(-?[\d.]+|-inf) LUFS", err)
    if not m or m.group(1) == "-inf":
        return None
    v = float(m.group(1))
    # ebur128 prints the absolute gate floor, -70.0, when nothing passed the gate (silence, or
    # fewer than the 400 ms one gating block needs). That is "undefined", not a loudness.
    return None if v <= -70.0 else v


def check(name, got, want, tol):
    ok = got is not None and abs(got - want) <= tol
    ROWS.append({"check": name, "got": got, "want": want, "tol": tol, "ok": ok})
    print(("ok  " if ok else "FAIL"), f"{name}: {got} (want {want} +/- {tol})")
    if not ok:
        FAILS.append(name)


def pink(seconds, amp):
    return f"anoisesrc=c=pink:r=48000:a={amp}:d={seconds}:seed=7"


# 1. One fixed gain lands the element on its target: measure, apply (target - measured), re-measure.
for target in (-23.0, -38.0, -14.0, -6.0):
    m = integrated(pink(5, 0.1))
    gain = target - m
    check(f"pink noise normalised to {target} LUFS (gain {gain:+.2f} dB)", integrated(pink(5, 0.1), f"volume={gain:.4f}dB"), target, 0.05)

# 2. The placed window is what is measured. A loud head trimmed off must not set the gain.
loud_then_quiet = "aevalsrc='if(lt(t,2),0.5,0.02)*(random(0)*2-1)':s=48000:d=6"
whole = integrated(loud_then_quiet)
window = integrated(loud_then_quiet, "atrim=start=2,asetpts=PTS-STARTPTS")
ROWS.append({"check": "whole file vs placed window differ", "whole": whole, "window": window, "ok": window is not None and whole is not None and whole - window > 10})
print("info whole file", whole, "LUFS; placed window (head trimmed)", window, "LUFS")
if not (window is not None and whole is not None and whole - window > 10):
    FAILS.append("window does not differ from whole file")
gain_win = -23.0 - window
check("gain from the window lands the trimmed element on -23", integrated(loud_then_quiet, f"atrim=start=2,asetpts=PTS-STARTPTS,volume={gain_win:.4f}dB"), -23.0, 0.05)

# 3. Undefined loudness: silence, and windows too short for one gating block.
sil = integrated("anullsrc=r=48000:cl=mono:d=3")
ROWS.append({"check": "silence is undefined", "got": sil, "ok": sil is None})
print(("ok  " if sil is None else "FAIL"), "silence: integrated loudness undefined", sil)
if sil is not None:
    FAILS.append("silence defined")
for ms in (200, 300, 399, 400, 500, 1000):
    v = integrated(pink(ms / 1000, 0.1))
    ROWS.append({"check": f"pink noise of {ms} ms", "got": v})
    print(f"info {ms} ms of pink noise: integrated =", v)
short_undefined = integrated(pink(0.3, 0.1)) is None
ROWS.append({"check": "a window under 400 ms is undefined", "ok": short_undefined})
print(("ok  " if short_undefined else "FAIL"), "a window under 400 ms is undefined")
if not short_undefined:
    FAILS.append("short window defined")
json.dump({"ffmpeg": subprocess.run([FFMPEG, "-version"], capture_output=True, text=True).stdout.splitlines()[0],
           "rows": ROWS, "pass": not FAILS}, open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "measurements-check.json"), "w"), indent=1)
if FAILS:
    print("\nFAILED:", FAILS)
    sys.exit(1)
print("\nall loudness numbers hold")
