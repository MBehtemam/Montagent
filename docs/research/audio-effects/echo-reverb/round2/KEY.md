# KEY (round 2 reverb). Open only after judging.

Built with ffmpeg version 7.0.2-static https://johnvansickle.com/ffmpeg/  Copyright (c) 2000-2024 the FFmpeg developers. Other builds compared: ffmpeg version 6.1.1-3ubuntu5 Copyright (c) 2000-2023 the FFmpeg developers; ffmpeg version n7.1.5-12-g1fdbca85aa-20260731 Copyright (c) 2000-2026 the FFmpeg developers.

## Which clip is which

| Clip | Is |
|---|---|
| `r2-A-X.m4a` | bypass (the unprocessed source) |
| `r2-A-Y.m4a` | candidate r2-A (taps-2200) |
| `r2-B-X.m4a` | candidate r2-B (ir-2200) |
| `r2-B-Y.m4a` | bypass (the unprocessed source) |
| `r2-C-X.m4a` | candidate r2-C (ir-3000) |
| `r2-C-Y.m4a` | bypass (the unprocessed source) |

The bypass clip is the same audio in all three pairs.

## The candidates (all literal, seeded, no IR file; `out = dry + mix * K * wet`, dry at unity, mix 0 is the bypass)

| Letter | Construction | decay_ms | mix | pre-delay ms | wet gain K | other |
|---|---|---|---|---|---|---|
| r2-A | aecho comb/diffusion bank | 2200 | 0.55 | 20 | 0.4447 | no afir. Per channel one `aecho` of 6 early taps (gains [0.55, 0.5, 0.42, 0.36, 0.3, 0.25]) plus 6 feedback combs written as decaying taps (L [29.7, 37.1, 41.1, 43.7, 53.9, 61.3] ms, R [30.9, 36.3, 42.3, 46.1, 52.7, 59.1] ms, tap k at pre-delay + k x d, gain 0.35 x 10^(-3 k d / decay_ms)), minus the dry copy aecho passes, two 4-tap diffusion `aecho`s, low-pass 5000 Hz |
| r2-B | afir IR | 2200 | 0.55 | 20 | 0.0498 | generated IR through `afir`; low band pink 120..1500 Hz decays over decay_ms; high band pink 1500..7500 Hz at x1.8 decays over decay_ms x 0.4; 4 ms attack ramp; 6 early-reflection taps per channel (L [7, 14, 23, 34, 47, 62] ms, R [9, 17, 26, 38, 52, 68] ms after the pre-delay, gains [2.2, 1.9, 1.6, 1.3, 1.1, 0.9]) low-passed at 4500 Hz; seeds low [21, 22], high [31, 32] |
| r2-C | afir IR | 3000 | 0.7 | 30 | 0.0428 | generated IR through `afir`; low band pink 120..1500 Hz decays over decay_ms; high band pink 1500..7500 Hz at x1.8 decays over decay_ms x 0.4; 4 ms attack ramp; 6 early-reflection taps per channel (L [7, 14, 23, 34, 47, 62] ms, R [9, 17, 26, 38, 52, 68] ms after the pre-delay, gains [2.2, 1.9, 1.6, 1.3, 1.1, 0.9]) low-passed at 4500 Hz; seeds low [21, 22], high [31, 32] |

Round-1 reverb for scale (the one the owner could not tell from bypass): afir pink-noise IR, decay_ms 1200, mix 0.3 as a dry/wet crossfade, no pre-delay, no early taps, one band.

## Objective energy decay (wet-only impulse response of the real graph, Schroeder backward integration, both channels)

RT60-like = slope of the decay curve extrapolated to -60 dB. T30 fits -5..-35 dB, T20 fits -5..-25 dB, EDT fits 0..-10 dB. Compare with the literal decay_ms. Band T30s come from FFT-masked bands; the 100-500 Hz one is unreliable (few modes, the fit is noisy and runs long), read it as a rough figure.

| Letter | decay_ms | EDT s | T20 s | T30 s | first -60 dB s | T30 100-500 Hz | T30 500-2k Hz | T30 2k-8k Hz | wet energy in first 80 ms |
|---|---|---|---|---|---|---|---|---|---|
| r2-A | 2200 | 2.06 | 2.19 | 2.19 | 2.07 | 2.32 | 2.23 | 2.19 | 52% |
| r2-B | 2200 | 1.62 | 2.12 | 2.13 | 2.01 | 3.19 | 2.19 | 1.47 | 54% |
| r2-C | 3000 | 2.29 | 2.85 | 2.93 | 2.75 | 3.71 | 3.02 | 2.00 | 44% |
| round 1 | 1200 | 1.17 | 1.20 | 1.20 | 1.13 | 44.10 | 1.43 | 1.24 | n/a |

## Wet level against dry, by band, on the narration (wet at its mix, dB re the dry in that band)

The round-1 complaint was 'higher noise'. The high bands show how much wet energy there is up there.

| Letter | 100-500 | 500-2k | 2k-8k | 8k-16k | all |
|---|---|---|---|---|---|
| r2-A | -5.05 | -4.57 | -8.13 | -15.07 | -5.19 |
| r2-B | -3.7 | -6.2 | -14.99 | -21.62 | -5.2 |
| r2-C | -1.51 | -4.28 | -12.96 | -19.59 | -3.09 |
| round 1 | -6.55 | -7.2 | -10.0 | -10.11 | -7.14 |

## Onset latency, smoke, determinism (ffmpeg 7.0.2 unless stated)

| Letter | dry onset error (samples) | length error | wet first sample (ms after impulse; literal pre-delay) | peak before match gain | LUFS delta vs bypass | mix 0 vs bypass | rerun identical | scalar = SIMD |
|---|---|---|---|---|---|---|---|---|
| r2-A | 0 | 0 | 27.0 (20) | 0.863 | +1.00 | max diff 0e+00 | True | True (max 0.0e+00) |
| r2-B | 0 | 0 | 20.02 (20) | 0.796 | +0.90 | max diff 0e+00 | True | False (max 1.2e-07) |
| r2-C | 0 | 0 | 30.02 (30) | 0.806 | +1.30 | max diff 0e+00 | True | False (max 1.2e-07) |

30 of 30 claims hold (list in `measurements-round2.json`).

## Drift between ffmpeg builds (whole 20 s mix, float PCM, against the 7.0.2 render)

| Letter | build | identical | max abs diff | diff RMS re signal | first differing sample |
|---|---|---|---|---|---|
| r2-A | ffmpeg version 6.1.1-3ubuntu5 | True | 0 | -219.5 dB | None |
| r2-A | ffmpeg version n7.1.5-12-g1fdbca85aa-20260731 | True | 0 | -219.5 dB | None |
| r2-B | ffmpeg version 6.1.1-3ubuntu5 | False | 0.108 | -39.4 dB | 927744 |
| r2-B | ffmpeg version n7.1.5-12-g1fdbca85aa-20260731 | True | 0 | -219.5 dB | None |
| r2-C | ffmpeg version 6.1.1-3ubuntu5 | False | 0.108 | -40.0 dB | 927744 |
| r2-C | ffmpeg version n7.1.5-12-g1fdbca85aa-20260731 | True | 0 | -220.1 dB | None |

The narration element ends at sample 932400 (19.425 s). A first difference near 927744 is the `afir` end-of-source difference round 1 found on 6.1.1, now in the wet path only. The playwright build (ffmpeg 7.0.1) lacks filters this graph needs and was left out.

## Recommended listening order, and why

`r2-C`, then `r2-B`, then `r2-A`.
- `r2-C` is the longest and strongest (decay 3000 ms, mix 0.7, afir IR): if a generated room is audible at all, it is here, so it teaches the ear what to listen for.
- `r2-B` is the same construction milder (2200 ms, 0.55); `r2-A` is the no-afir tap bank at the same decay and mix as `r2-B`, so `r2-B` then `r2-A` is the like-for-like comparison of the two constructions.
- Within a pair, X, Y, X again; headphones. The pauses in the narration (silencedetect, -40 dB, 0.25 s) are at about 5.8, 7.9, 13.5 and 15.4 s, and the last word ends at 19.425 s: the reverb can only be heard ringing into those gaps, and it is cut at 19.425 s (the tail rule).

## What this round changed, and limits

- Fixes aimed at the round-1 'noise' impression: pre-delay (20 or 30 ms), exponential decay kept, high band lowpassed and decaying faster than the low band (afir candidates) or lowpassed (tap bank), early reflections, decorrelated L/R, a 120 Hz high-pass on the low band (no rumble). The IR candidates have much less 2k-16k wet energy than round 1 (table above); the tap bank has more.
- `acomb` does not exist in ffmpeg and `aecho` accepts only positive gains and does not recurse, so a true Schroeder comb/all-pass is not expressible; the tap bank writes each feedback comb out as decaying taps (about 300 per channel) and has no all-pass, only positive-gain diffusers. `aiir` could do a real comb but needs a coefficient list as long as the delay (thousands of values). A listener hearing this one as 'metallic' or 'comb-like' is hearing that limit, not a verdict on the idea.
- Level: wet gains K are literals calibrated on this narration so wet-only RMS equals dry RMS (ffmpeg 7.0.2); they are spectrum dependent for the IR candidates (afir) and not for the tap bank. All clips are then brought to -20 LUFS by one gain, so the audible reverb-to-voice ratio is what the mix values set, not a loudness cue.
- One listener, one voice, one fixture; the narration has only four short pauses.
