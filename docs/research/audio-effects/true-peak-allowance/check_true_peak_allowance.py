#!/usr/bin/env python3
"""Measures how far the decoded AAC delivery overshoots `master.ceiling_dbtp` (ADR-0173 §7, #815).

Usage:  python3 -I check_true_peak_allowance.py [--ffmpeg PATH] [--allowance DB] [--json OUT]

Exits 1 when the fixture's decoded-AAC true peak lands more than the allowance above the ceiling,
or a synthetic signal's more than ADVERSARIAL_BOUND_DB (or when a meter parse breaks), 0 otherwise. Stdlib only; ffmpeg is the only dependency.

Every case runs ADR-0172's closing bus stage on a signal driven OVER dB above the ceiling:

    volume=<ceiling + over - source true peak>dB, alimiter=limit=<ceiling as linear>:latency=1
      -> native `aac` encoder at 160k -> decode -> ebur128=peak=true

and records two overshoots, both against the ceiling:
  pcm  true peak of the limited float signal, before the encoder (the limiter's own miss:
       alimiter sees sample peaks, not inter-sample ones);
  aac  true peak of the decoded AAC (what `verify` measures, and what the allowance covers).

The signals: an fs/4 sine at a 45 degree phase offset (every sample sits at 0.707 of the true
peak, the worst case for a sample-peak limiter), clipped-sine bursts, seeded pink noise, and
the shared narration-over-bed fixture.

The allowance in ADR-0173 §6 is the worst `aac` overshoot across the three CI legs plus a
margin, rounded up to 0.1 dB. This script prints this leg's table; the legs are compared in the
resolution of #815.
"""
import argparse
import hashlib
import json
import math
import re
import subprocess
import sys
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
FIXTURE = HERE.parent / "fixtures" / "narration-over-bed" / "narration-over-bed.flac"
CEILING_DBTP = -1.0
OVERS_DB = (3.0, 6.0, 10.0)  # how far over the ceiling the limiter's input peaks
DEFAULT_ALLOWANCE_DB = 1.0  # ADR-0173 §6: the review threshold, held by the shared fixture
ADVERSARIAL_BOUND_DB = 1.9  # worst measured (+1.8, clipped bursts and +10 dB pink noise) + 0.1
SAMPLE_RATE = 48000


# The limiter stage under test. `plain` is ADR-0172 as written (a sample-peak limiter at the
# ceiling); `oversampled4x` runs the same limiter at 4x the rate so it sees inter-sample peaks.
STAGES = {
    "plain": "alimiter=limit={limit:.8f}:latency=1:level=0",
    "oversampled2x": (
        "aresample=96000,alimiter=limit={limit:.8f}:latency=1:level=0,aresample=48000"
    ),
    "oversampled8x": (
        "aresample=384000,alimiter=limit={limit:.8f}:latency=1:level=0,aresample=48000"
    ),
    "oversampled4x": (
        "aresample=192000,alimiter=limit={limit:.8f}:latency=1:level=0,aresample=48000"
    ),
}


def run(ffmpeg, args):
    p = subprocess.run([ffmpeg, "-hide_banner", "-nostdin", *args], capture_output=True, text=True)
    if p.returncode != 0:
        sys.exit(f"ffmpeg failed: {' '.join(args)}\n{p.stderr[-2000:]}")
    return p.stderr


def true_peak(ffmpeg, src_args):
    """Integrated true peak in dBTP via ebur128=peak=true. Fails loudly if the parse breaks."""
    log = run(ffmpeg, [*src_args, "-af", "ebur128=peak=true", "-f", "null", "-"])
    m = re.search(r"True peak:\s*\n\s*Peak:\s*(-?\d+(?:\.\d+)?|-inf)\s*dBFS", log)
    if not m:
        sys.exit(f"ebur128 true-peak parse broke; the summary now reads:\n{log[-1500:]}")
    return float(m.group(1))


def sources():
    tone = (
        f"aevalsrc=exprs='sin(2*PI*12000*t+PI/4)|sin(2*PI*12000*t+PI/4)':s={SAMPLE_RATE}:d=3"
    )
    clipped = (
        f"aevalsrc=exprs='clip(4*sin(2*PI*997*t),-1,1)*lt(mod(t,0.5),0.25)"
        f"|clip(4*sin(2*PI*997*t),-1,1)*lt(mod(t,0.5),0.25)':s={SAMPLE_RATE}:d=4"
    )
    pink = f"anoisesrc=color=pink:seed=7:amplitude=0.5:sample_rate={SAMPLE_RATE}:duration=5"
    return {
        "fs4_sine_45deg": ["-f", "lavfi", "-i", tone],
        "clipped_sine_bursts": ["-f", "lavfi", "-i", clipped],
        "pink_noise_seed7": ["-f", "lavfi", "-i", pink],
        "fixture_narration_over_bed": ["-i", str(FIXTURE)],
    }


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--ffmpeg", default="ffmpeg")
    ap.add_argument("--allowance", type=float, default=DEFAULT_ALLOWANCE_DB)
    ap.add_argument("--stage", choices=sorted(STAGES), default="oversampled4x")
    ap.add_argument("--json")
    args = ap.parse_args()
    ff = args.ffmpeg

    version = subprocess.run([ff, "-version"], capture_output=True, text=True).stdout.splitlines()[0]
    enc = subprocess.run([ff, "-hide_banner", "-encoders"], capture_output=True, text=True).stdout
    if not re.search(r"^\s*A\S*\s+aac\s", enc, re.M):
        sys.exit("ffmpeg has no native `aac` encoder")
    if not FIXTURE.exists():
        sys.exit(f"missing fixture {FIXTURE}")

    limit = 10 ** (CEILING_DBTP / 20)
    rows, failures = [], []
    with tempfile.TemporaryDirectory() as tmp:
        tmp = Path(tmp)
        for name, src in sources().items():
            src_tp = true_peak(ff, src)
            for over in OVERS_DB:
                gain = CEILING_DBTP + over - src_tp
                chain = (
                    f"aformat=sample_fmts=fltp:sample_rates={SAMPLE_RATE}:channel_layouts=stereo,"
                    f"volume={gain:.6f}dB," + STAGES[args.stage].format(limit=limit)
                )
                run(ff, ["-y", *src, "-af", chain, "-c:a", "pcm_f32le", str(tmp / "limited.wav")])
                run(ff, ["-y", "-i", str(tmp / "limited.wav"), "-c:a", "aac", "-b:a", "160k", str(tmp / "out.m4a")])
                run(ff, ["-y", "-i", str(tmp / "out.m4a"), "-c:a", "pcm_f32le", str(tmp / "decoded.wav")])
                pcm = true_peak(ff, ["-i", str(tmp / "limited.wav")]) - CEILING_DBTP
                aac = true_peak(ff, ["-i", str(tmp / "decoded.wav")]) - CEILING_DBTP
                if not math.isfinite(pcm) or not math.isfinite(aac):
                    sys.exit(f"{name} +{over}: silent output (pcm {pcm}, aac {aac})")
                rows.append(
                    {
                        "signal": name,
                        "over_db": over,
                        "source_tp_dbtp": round(src_tp, 2),
                        "pcm_overshoot_db": round(pcm, 2),
                        "aac_overshoot_db": round(aac, 2),
                        "limited_pcm_sha256": hashlib.sha256((tmp / "limited.wav").read_bytes()).hexdigest()[:16],
                    }
                )
                bound = args.allowance if name.startswith("fixture") else ADVERSARIAL_BOUND_DB
                if aac > bound:
                    failures.append(rows[-1])

    worst = max(r["aac_overshoot_db"] for r in rows)
    print(f"{version}\nceiling {CEILING_DBTP} dBTP, allowance +{args.allowance} dB, encoder aac 160k, stage {args.stage}\n")
    print(f"{'signal':30} {'over':>5} {'src TP':>8} {'pcm':>7} {'aac':>7}")
    for r in rows:
        print(
            f"{r['signal']:30} {r['over_db']:>5.0f} {r['source_tp_dbtp']:>8.2f} "
            f"{r['pcm_overshoot_db']:>+7.2f} {r['aac_overshoot_db']:>+7.2f}"
        )
    print(f"\nworst aac overshoot: {worst:+.2f} dB")
    if args.json:
        Path(args.json).write_text(
            json.dumps({"ffmpeg": version, "ceiling_dbtp": CEILING_DBTP, "stage": args.stage, "worst_aac_overshoot_db": worst, "rows": rows}, indent=2) + "\n"
        )
    if failures:
        print(f"\nFAIL: {len(failures)} case(s) over their bound (fixture {args.allowance}, synthetic {ADVERSARIAL_BOUND_DB} dB)", file=sys.stderr)
        sys.exit(1)
    print("ok")


if __name__ == "__main__":
    main()
