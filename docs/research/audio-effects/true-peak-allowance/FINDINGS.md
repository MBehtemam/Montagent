# True-peak allowance: what was measured (#815)

[`check_true_peak_allowance.py`](check_true_peak_allowance.py) drives four signals 3, 6 and 10 dB
over a −1 dBTP ceiling through the master stage's closing limiter, the native `aac` encoder at
160k and a decode, and reads true peak with `ebur128=peak=true`. Run on 2026-10-08 on the CI
legs (workflow run on a throwaway branch; raw JSON in [`measurements/`](measurements/)). The
meter reads a full-scale 12 kHz sine at +0.6 dBTP, so the ceiling it enforces is the meter's.

## What it found

1. **ADR-0172's stage as written does not hold a dBTP ceiling.** Its `alimiter` is a
   *sample-peak* limiter, so a limit at −1 dBFS lets inter-sample peaks reach +2 dBTP (+3 dB
   over the ceiling) on an fs/4 sine, before any encoder. The worst decoded-AAC miss on the
   legs is **+3.3 to +3.8 dB**, not 0.5.
2. **`alimiter`'s `level` option is on by default** (auto-level back toward 0 dBFS). The
   stage must say `level=0`; with it left on, the same signals overshoot more still
   (fs/4 sine +4.6 dB). `FFMPEG-FILTERS.md` §3 lists `latency` but not `level`.
3. **Limiting at 4× the rate (`aresample=192000 … aresample=48000` around the limiter) closes
   the PCM gap** to ≤ +0.2 dB. It adds no delay and no length change (impulse at sample 2400
   stays at 2400, 48000 samples out) and is byte-identical across two runs on one build.
4. **The encoder then dominates.** AAC adds up to **+1.8 dB** on clipped-sine bursts and +1.7
   on seeded pink noise, identically on all three legs; on the narration-over-bed fixture it
   adds ≤ +0.5 dB (worst leg: Windows, 10 dB over).
5. **One leg differs:** on the plain stage macOS Homebrew gives +2.4 on the fixture at 10 dB
   over where Linux and Windows give +0.9. That is encoder drift across builds, and is why the
   allowance is measured on every leg.


### Stage `plain`: decoded-AAC overshoot over the −1 dBTP ceiling, dB (pre-encoder PCM in brackets)

| signal | over | Linux (floor) | macOS (Homebrew) | Windows (Chocolatey) |
| --- | ---: | ---: | ---: | ---: |
| fs4_sine_45deg | +3 | +3.0 (+3.0) | +2.7 (+3.0) | +2.7 (+3.0) |
| fs4_sine_45deg | +6 | +3.8 (+3.6) | +3.3 (+3.6) | +3.3 (+3.6) |
| fs4_sine_45deg | +10 | +3.8 (+3.6) | +3.3 (+3.6) | +3.3 (+3.6) |
| clipped_sine_bursts | +3 | +3.2 (+1.1) | +2.8 (+1.1) | +2.8 (+1.1) |
| clipped_sine_bursts | +6 | +3.2 (+1.1) | +2.8 (+1.1) | +2.8 (+1.1) |
| clipped_sine_bursts | +10 | +2.9 (+1.1) | +2.8 (+1.1) | +2.8 (+1.1) |
| pink_noise_seed7 | +3 | +0.1 (+0.0) | +0.1 (+0.0) | +0.1 (+0.0) |
| pink_noise_seed7 | +6 | +1.0 (+0.6) | +1.4 (+0.6) | +1.4 (+0.6) |
| pink_noise_seed7 | +10 | +1.8 (+1.0) | +1.7 (+1.0) | +1.7 (+1.0) |
| fixture_narration_over_bed | +3 | +0.1 (+0.1) | +0.2 (+0.1) | +0.1 (+0.1) |
| fixture_narration_over_bed | +6 | +0.7 (+0.5) | +0.5 (+0.5) | +0.5 (+0.5) |
| fixture_narration_over_bed | +10 | +0.9 (+0.9) | +2.4 (+0.9) | +0.9 (+0.9) |
| **worst** | | **+3.8** | **+3.3** | **+3.3** |

### Stage `oversampled4x`: decoded-AAC overshoot over the −1 dBTP ceiling, dB (pre-encoder PCM in brackets)

| signal | over | Linux (floor) | macOS (Homebrew) | Windows (Chocolatey) |
| --- | ---: | ---: | ---: | ---: |
| fs4_sine_45deg | +3 | +0.3 (+0.0) | +0.1 (+0.0) | +0.2 (+0.0) |
| fs4_sine_45deg | +6 | +0.2 (+0.0) | +0.2 (+0.0) | +0.2 (+0.0) |
| fs4_sine_45deg | +10 | +0.3 (+0.0) | +0.1 (+0.0) | +0.2 (+0.0) |
| clipped_sine_bursts | +3 | +1.2 (+0.0) | +1.8 (+0.0) | +1.8 (+0.0) |
| clipped_sine_bursts | +6 | +1.8 (+0.0) | +0.9 (+0.0) | +0.9 (+0.0) |
| clipped_sine_bursts | +10 | +0.6 (+0.0) | +1.8 (+0.0) | +1.8 (+0.0) |
| pink_noise_seed7 | +3 | -0.1 (+0.0) | -0.1 (+0.0) | -0.1 (+0.0) |
| pink_noise_seed7 | +6 | +1.0 (+0.1) | +1.3 (+0.1) | +1.3 (+0.1) |
| pink_noise_seed7 | +10 | +1.7 (+0.2) | +1.3 (+0.2) | +1.3 (+0.2) |
| fixture_narration_over_bed | +3 | +0.0 (+0.0) | +0.1 (+0.0) | +0.0 (+0.0) |
| fixture_narration_over_bed | +6 | +0.3 (+0.0) | +0.3 (+0.0) | +0.2 (+0.0) |
| fixture_narration_over_bed | +10 | +0.3 (+0.0) | +0.3 (+0.0) | +0.5 (+0.0) |
| **worst** | | **+1.8** | **+1.8** | **+1.8** |

## What this does not settle

The ticket's rule makes the allowance "the worst case across legs plus a margin, rounded up to
0.1 dB". Taken literally that is **+3.9 dB** for the stage as written and **+1.9 dB** for the
oversampled one, against 0.5 dB in ADR-0173 §6. Which stage the master takes, whether the
adversarial signals set the allowance or only the shared fixture does, and whether a limiter
headroom margin replaces a looser `verify` tolerance are decisions for ADR-0172 and ADR-0173,
not for this measurement. ADR-0173 therefore stays `proposed`.
