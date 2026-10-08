# Noise gate as an audio effect: evidence for ADR-0181 (draft)

Ticket: the noise-gate capability on the audio map ([#795](https://github.com/MBehtemam/Montagent/issues/795)),
modelled on the compressor and limiter ([ADR-0180](../../../adr/0180-the-compressor-and-the-limiter-are-two-audio-effects-with-no-defaults-and-an-rms-threshold.md)).

- `check_gate.py` is the evidence script (stdlib + ffmpeg; exits non-zero when a number stops holding).
  Run `python3 -I check_gate.py`. Results: `measurements.json`. All numbers below are **ffmpeg 6.1.1
  (Ubuntu), Linux only**: the three-leg run belongs to the repo test (ADR-0173 section 4).
- `prototype_gate.py` renders the blind A/B (`ab/X.m4a`, `ab/Y.m4a`, `ab/KEY`: do not open KEY before
  judging) and writes `measurements-ab.json`. **There is no `VERDICT.md`: the owner has not listened yet.**

Every number is measured on PCM in double precision before any AAC encode, from lavfi signals built from
literal parameters (ADR-0173 section 3).

## Shape under test: one member, stackable

```json
{ "name": "noise_gate", "threshold_db": -34, "ratio": 10, "attack_ms": 5, "release_ms": 150, "range_db": 30 }
```

Rendered as `agate=threshold=10^(T/20):ratio=(R+1)/2:attack=A:release=L:range=10^(-G/20):knee=1:detection=rms:link=average:makeup=1:mode=downward`.

## What `agate` does (measured)

1. **The threshold is RMS dBFS, and ffmpeg's `ratio` is not an n:1 ratio.** `agate` in RMS mode squares the
   threshold and works on a power-domain slope, so its `ratio` r gives **2 (r-1) dB of reduction per dB under
   the threshold**, not (r-1). The renderer therefore maps the authored ratio R to r = (R+1)/2, which makes R a
   plain downward-expander ratio: **reduction = min(range_db, (R-1) x (threshold - level))**, 0 above the
   threshold. Measured on 1 kHz sines at attack = release = 100 ms over 6 settings x 12 levels: every point
   within 0.01 dB (asserted at 0.3 dB; the 1.5 dB around the threshold is the transition and is recorded, not
   asserted). Range caps the reduction exactly (12, 40, 80 dB tested). R = 1 is the identity.
2. **Open means identity.** Far above the threshold the gate is bit-identical to bypass (max |out - in| = 0) and
   the length is exact.
3. **Latency and onset: 0 samples.** An isolated impulse peaks on the same sample (shift 0) and the length is
   unchanged, so the gate needs no `apad...atrim` compensation (FFMPEG-FILTERS section 5 already lists `agate`
   at 0). A loud burst after a closed gate has its first sample over 0.01 on the **same sample at attack 0.1 ms**;
   at attack 5 ms it comes 2 samples later and at 50 ms 5 samples later. That is the attack ramp (the gate
   opening), not latency, and it is why a slow attack softens the head of a word.
4. **The timings are not time constants, and the threshold moves with them.** The detector smooths the
   *linear power* with the attack coefficient when it rises and the release coefficient when it falls. On a
   sine, whose power ripples, that biases the effective threshold, in dB, by delta (measured with R = 2):

   | attack / release (ms) | delta (dB, + = reduces less than written) |
   | --- | --- |
   | 5/5, 50/50, 100/100, 1000/1000 | 0.03, 0.00, 0.00, 0.00 (exact when attack = release) |
   | 0.1/1, 1/10, 5/50, 20/250 | +2.32, +2.14, +2.13, +2.23 |
   | 5/1000, 50/9000 | +2.88, +2.87 (limit +3.01: the sine's peak-to-mean power) |
   | 20/10, 100/10, 500/100, 2000/1 | -1.06, -4.31, -2.77, -19.43 (looser than written) |

   The bias is **multiplied by (R - 1)** in the output (R = 10 at 5/50 reads 19 dB shallower on the
   same tone). It is the same at 200 Hz, 1 kHz and 5 kHz. The compressor's equivalent was up to 2.2 dB
   *deeper*; the gate's is *shallower* when attack < release and *deeper* when attack > release, and
   unbounded below. The contract is "exact at attack = release; threshold reads 0..+3.1 dB high when
   attack <= release" and a review when attack > release.
5. **Noise leaks earlier than a sine.** Pink noise, threshold -34..-38, R = 10, range 30: a floor 12 dB under the
   threshold is fully closed (29.8 dB), a floor 6 dB under is mostly open (3.4 dB at attack 5 / release 150;
   27.8 dB at 50/50). A gate's threshold must sit well over the floor; the docs say so.
6. **Attack and release as times (burst 6 dB over the threshold, from a closed gate, 1 ms windows):**
   within 1 dB of open after 0, 1, 4, 14 ms for attack 2, 10, 50, 200 ms; 20 dB of reduction after 6, 63, 322 ms
   for release 10, 100, 500 ms. Ordering is asserted; the numbers are not time constants and depend on how far
   over the threshold the burst is.
7. **`agate` does not honour commands (ADR-0842/#842: static parameters only).** `asendcmd` to `agate@g` for
   `threshold`, `range`, `ratio`, `attack` and `release` is accepted and changes nothing (gain after the command
   0.00 dB in all five, against -60 dB had the threshold been static), while the control, `volume@v`, moves by
   -6.0206 dB in the same harness. Matches FFMPEG-FILTERS section 4.2 (`agate_config_input` is never re-run).
   **No parameter of the gate can be keyframed**, and none is a candidate for admission.
8. **Build-to-build drift.** No stochastic or SIMD code: the output is byte-identical across runs and with
   `-cpuflags 0`. Across builds, the processing code of `af_agate.c` is **unchanged between n6.1.1, n7.1.5 and
   master** (a `diff` shows only `.unit =` option-table spelling and the `FFFilter`/format-negotiation API
   changes); the difference is in registration, not in `filter_frame`, `output_gain` or the detector.
   Expect no cross-build drift; ADR-0173 still wants the three-leg table, so the repo test commits it.
9. **Range edges.** `threshold` 1e-4..1, ffmpeg ratio 1..9000 (authored R up to 17999 would fit; the ADR caps
   at 100), attack 0.1..2000, release 1..9000 and range 1e-4..1 are all accepted; attack 0.001, release 9001 and
   ffmpeg ratio above 9000 are rejected (negative controls).

## The A/B (not yet heard)

`ab/X.m4a` and `ab/Y.m4a`: the shared TTS narration plus a seeded pink-noise floor (about -48 dBFS RMS; the
fixture has digital silence between phrases, so a gate would have nothing to do on it), AAC 160k, one of them
gated with the member above. On PCM: pause RMS **-51.1 -> -81.1 dBFS (-30.0 dB)**, speech RMS **-21.26 ->
-21.26 dBFS (0.00 dB)**. Loudness matching is not applied: integrated loudness is set by the speech, which the
gate does not touch. Listen for: the floor gone in the pauses, any chopped word onsets or tails, and whether the
return of the floor at each phrase is noticed. `measurements-ab.json` has the numbers.

## Precedent and departures (PRECEDENT.md section 6)

Premiere has a gate (Dynamics AutoGate: threshold, attack, hold, release) and an Expander (threshold, ratio);
CapCut has none confirmed. This member is one gate that carries the Expander's ratio and a range, with **no hold**
(`agate` has none), fixed RMS detection, hard knee, stereo link by average, and no side-chain. Parameter values
in Premiere are recalled, not quoted.
