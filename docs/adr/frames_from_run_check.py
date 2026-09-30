#!/usr/bin/env python3
"""ADR-0127's measurement, reproduced from scratch against this machine's `ffmpeg`.

Run from anywhere:  python3 docs/adr/frames_from_run_check.py
Exits non-zero, naming every defect, the moment one of its claims stops holding.

Why this exists
---------------
ADR-0127 rests on a comparison, not an argument: `frames_from`'s old command and its new one,
each run over the same matrix of sources, starts, frame rates and speeds, and each frame
checked against the frame the renderer paints at that timeline frame. The two claims are:

  1. The old command (`-ss from -vf fps=fps*speed`) disagrees with the render on almost every
     run — by the seek, by `fps=`'s nearest-tick rounding and by its inverted `speed`.
  2. The new chain (`-ss from-200 -copyts`, `settb`, the integer `setpts`, `fps=:round=up`)
     agrees with it on every run.

The renderer's rule is `render`'s arithmetic — instant `n*1000//fps`, offset
`from + round_half_up(instant * speed)` — then ADR-0096's at-or-before with one microsecond of
slack, read here off each source's own frame timestamps. Frames are identified by the index
the generator writes into their red channel, and on the committed reference MP4 by matching
against its own full decode.

`crates/montagent-core/tests/seek_clamp.rs` holds the behaviour of the fix itself; this holds
the claims about `ffmpeg` that made it necessary. Needs `ffmpeg`/`ffprobe` on PATH (ADR-0009)
and skips with a message rather than failing when they are absent.
"""
import hashlib
import os
import shutil
import subprocess
import sys
import tempfile
from fractions import Fraction as F

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
REF = os.path.join(REPO, "fixtures/en-halloween-decorating/reference/en-halloween-decorating.mp4")
SIZE, FRAMES, SAMPLES = 64, 82, 30
PICTURE = SIZE * SIZE * 4
GEN = "color=c=black:s=64x64:r={r},format=rgba,geq=r='N*3':g='0':b='0':a='255'"


def run(args):
    return subprocess.run(args, capture_output=True)


def encode(work, name, rate, extra=()):
    path = os.path.join(work, name)
    run(["ffmpeg", "-hide_banner", "-loglevel", "error", "-y", "-f", "lavfi", "-i",
         GEN.format(r=rate), "-frames:v", str(FRAMES), "-c:v", "ffv1", "-pix_fmt", "gbrp",
         *extra, path])
    return path


def timestamps(path):
    out = run(["ffprobe", "-v", "error", "-select_streams", "v:0", "-show_entries",
               "frame=pts_time", "-of", "csv=p=0", path]).stdout.decode().split()
    return [F(x.strip(",")) for x in out if x.strip(",")]


def pictures(stdout):
    return [stdout[i:i + PICTURE] for i in range(0, len(stdout) - PICTURE + 1, PICTURE)]


def old_run(path, from_ms, fps, num, den):
    rate = repr(fps * num / den)
    rate = rate[:-2] if rate.endswith(".0") else rate  # Rust's `Display` for an f64
    return pictures(run(["ffmpeg", "-hide_banner", "-loglevel", "error",
                         "-ss", f"{from_ms // 1000}.{from_ms % 1000:03d}", "-i", path,
                         "-vf", f"fps={rate},scale={SIZE}:{SIZE}",
                         "-f", "rawvideo", "-pix_fmt", "rgba", "-"]).stdout)


def new_run(path, from_ms, fps, num, den):
    window = max(from_ms - 200, 0)
    chain = (f"settb=1/1000000,"
             f"setpts='ceil((2*(ceil((PTS-1)/1000)-{from_ms})-1)*{den}/(2*{num}))*1000',"
             f"fps={fps}:round=up:start_time=0,scale={SIZE}:{SIZE}")
    return pictures(run(["ffmpeg", "-hide_banner", "-loglevel", "error",
                         "-ss", f"{window // 1000}.{window % 1000:03d}", "-copyts", "-i", path,
                         "-vf", chain, "-fps_mode", "passthrough",
                         "-f", "rawvideo", "-pix_fmt", "rgba", "-"]).stdout)


def painted(ts, from_ms, fps, num, den, n):
    elapsed = F((n * 1000) // fps)
    x = elapsed * F(num, den)
    offset = from_ms + (2 * x.numerator + x.denominator) // (2 * x.denominator)
    showing = [i for i, t in enumerate(ts) if t <= F(offset, 1000) + F(1, 10 ** 6)]
    return showing[-1] if showing else 0


def main():
    if not (shutil.which("ffmpeg") and shutil.which("ffprobe")):
        print("skipped: ffmpeg/ffprobe not on PATH (ADR-0009)")
        return 0
    work = tempfile.mkdtemp()
    try:
        red = lambda p: p[0] // 3 if p[0] % 3 == 0 else None
        sources = {
            "25 fps": (encode(work, "g25.mkv", "25"), red),
            "30000/1001": (encode(work, "frac.mov", "30000/1001"), red),
            "30 fps": (encode(work, "g30.mkv", "30"), red),
            "42 ms origin, a gap": (encode_vfr(work), red),
        }
        table = {}
        decoded = run(["ffmpeg", "-hide_banner", "-loglevel", "error", "-i", REF, "-vf",
                       f"scale={SIZE}:{SIZE}", "-fps_mode", "passthrough", "-f", "rawvideo",
                       "-pix_fmt", "rgba", "-"]).stdout
        for i, p in enumerate(pictures(decoded)):
            table.setdefault(hashlib.sha1(p).digest(), set()).add(i)
        sources["reference MP4"] = (REF, lambda p: table.get(hashlib.sha1(p).digest()))

        speeds = [(1, 1), (1, 2), (3, 2), (2, 1), (645, 1000)]
        tally = {"old": [0, 0], "new": [0, 0]}
        for label, (path, ident) in sources.items():
            ts = timestamps(path)
            starts = [0, 20, 1234, 5010] if path == REF else [0, 20, 1200, 1234, 1239, 1001]
            for from_ms in starts:
                for fps in (25, 30, 24, 60):
                    for num, den in speeds:
                        want = [painted(ts, from_ms, fps, num, den, n) for n in range(SAMPLES)]
                        last = from_ms + (F(((SAMPLES - 1) * 1000) // fps) * F(num, den))
                        if last > ts[-1] * 1000:
                            continue
                        for name, sample in (("old", old_run), ("new", new_run)):
                            got = [ident(p) for p in sample(path, from_ms, fps, num, den)][:SAMPLES]
                            ok = len(got) == SAMPLES and all(
                                g == w or (isinstance(g, set) and w in g) or g == {w}
                                for g, w in zip(got, want))
                            tally[name][0 if ok else 1] += 1
                            if name == "new" and not ok:
                                print(f"  FAIL: new chain, {label} from {from_ms} ms at {fps} fps, "
                                      f"speed {num}/{den}: got {got[:8]}, render paints {want[:8]}")
        old_ok, old_bad = tally["old"]
        new_ok, new_bad = tally["new"]
        total = old_ok + old_bad
        failures = 0
        if old_bad * 10 >= total * 9:
            print(f"  ok: the old command disagrees with the render on {old_bad} of {total} runs")
        else:
            print(f"  FAIL: the old command agrees with the render on {old_ok} of {total} runs; "
                  f"ADR-0127's premise no longer reproduces")
            failures += 1
        if new_bad == 0:
            print(f"  ok: the new chain agrees with the render on all {total} runs")
        else:
            failures += 1
        print()
        print("ADR-0127 holds." if failures == 0 else f"ADR-0127 does NOT hold: {failures} defect(s).")
        return failures
    finally:
        shutil.rmtree(work, ignore_errors=True)


def encode_vfr(work):
    # This repository's reference MP4's shape: frames from 42 ms, stepping 40 ms, one gap.
    path = os.path.join(work, "vfr.mkv")
    run(["ffmpeg", "-hide_banner", "-loglevel", "error", "-y", "-f", "lavfi", "-i",
         GEN.format(r="25") + ",setpts='(N*0.04+gte(N\\,30)*0.016)/TB'",
         "-frames:v", str(FRAMES), "-fps_mode", "passthrough", "-c:v", "ffv1",
         "-pix_fmt", "gbrp", "-output_ts_offset", "0.042", path])
    return path


if __name__ == "__main__":
    sys.exit(main())
