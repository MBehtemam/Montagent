---
status: proposed (DRAFT; becomes accepted once the render-test table in section 6 is committed with its three-leg spread and the owner's blind listen to the gate A/B is recorded; until then the tolerances there are provisional)
---

# The noise gate is one stackable audio effect with a range, no hold and an RMS threshold

[Capability ADR and hand-off spec: noise gate](https://github.com/MBehtemam/Montagent/issues/795) on the audio
map (#795), modelled on [ADR-0180](0180-the-compressor-and-the-limiter-are-two-audio-effects-with-no-defaults-and-an-rms-threshold.md).
The map named the noise gate under dynamics but no ticket ruled its shape; this draft proposes one and lists
the calls the owner must make (section 8). It conforms to
[ADR-0169](0169-audio-effects-are-an-ordered-list-before-volume-and-gain-and-routing-stay-flat.md),
[ADR-0170](0170-audio-levels-are-written-in-db-and-their-keys-say-so-volume-stays-the-one-linear-level.md),
[ADR-0172](0172-the-master-stage-is-a-top-level-loudness-target-and-true-peak-ceiling-reached-by-one-measured-gain.md),
[ADR-0179](0179-eq-is-four-stackable-audio-effects-built-from-butterworth-biquad-sections.md) (the list runs in float)
and [ADR-0173](0173-every-audio-capability-is-accepted-by-a-measured-repo-test-and-verify-checks-only-what-master-declares.md).
Evidence: [`docs/research/audio-effects/gate/`](../research/audio-effects/gate/README.md)
(`check_gate.py`, ffmpeg 6.1.1).

## Decisions

### 1. One member; every key is required and nothing defaults

```json
{ "name": "noise_gate", "threshold_db": -34, "ratio": 10, "attack_ms": 5, "release_ms": 150, "range_db": 30 }
```

| Key | Range | Meaning |
| --- | --- | --- |
| `threshold_db` | -80..0 | RMS dBFS (a full-scale sine reads -3.01). Under it the gate reduces. |
| `ratio` | 1..100 | n:1 downward expansion: each dB under the threshold becomes `ratio` dB. |
| `attack_ms` | 0.1..2000 | detector smoothing when the level rises (the gate opening). |
| `release_ms` | 1..9000 | detector smoothing when the level falls (the gate closing). |
| `range_db` | 0..80 | the most the gate may reduce, a positive number of dB. 80 is effectively off-to-silent. |

- **Stackable (non-singular), ADR-0169.** Two gates in series are legitimate practice (a gentle one and a hard
  one); a second enabled copy is a note, not an error (`N-GATE-STACKED`).
- **No key defaults**, the ADR-0180 stance (ADR-0145: a missing key is a clean error naming the keys; the
  documentation carries the worked example above for an agent to copy and edit by exact-string replace). It is
  a minority call in ADR-0180 and stays the owner's to reopen for both (section 8).
- The ranges sit inside `agate`'s own (threshold 0..1 linear, ratio 1..9000, attack 0.01..9000, release
  0.01..9000, range 0..1), so no accepted document can fail at render time: every bound renders, and the
  just-outside values (attack 0.001, release 9001, ffmpeg ratio above 9000) are rejected by ffmpeg (the negative
  controls the check keeps).
- `ratio` stops at 100: with `range_db` at most 80 and a ratio of 100, any level more than 0.8 dB under the
  threshold is fully reduced, so a higher ratio only moves the knee-less edge by a fraction of a dB.
- **There is no hold.** Premiere's AutoGate has one; `agate` does not, and a hold built from other filters has
  not been measured. The release is what keeps a gate from chattering. See departures, section 7.
- `range_db` is a positive depth, not a negative gain, so that `0` reads "no reduction" and `80` "as much as
  possible". `makeup` is not authorable: a gate removes sound and gain is `volume`'s job.

Rejected: `mode: upward` (a different effect), `detection: peak` (the prototype reads a sine's peak about 3 dB
off RMS, as for the compressor), `knee` (no closed form to check), `link: maximum` (stereo link is fixed at
`average`), a side-chain input (a duck stays a script, ADR-0177), keyframing any key (section 6).

### 2. How it is rendered, and what the numbers mean

```
agate=threshold=<10^(threshold_db/20)>:ratio=<(ratio+1)/2>:attack=<attack_ms>:release=<release_ms>
     :range=<10^(-range_db/20)>:knee=1:detection=rms:link=average:makeup=1:mode=downward
```

- **`threshold_db` and every gate level are RMS dBFS**, the compressor's convention (ADR-0180 section 2), and the
  documentation says "RMS".
- **The authored `ratio` is not ffmpeg's.** In RMS mode `agate` squares its threshold and works on a
  power-domain slope, so ffmpeg's `ratio` r reduces by 2 (r - 1) dB per dB under the threshold. The renderer
  writes r = (ratio + 1) / 2 so the author's `ratio` is the plain n:1 it reads as. This is a render semantic the
  author never sees, like `atempo` chains.
- **The curve.** Below the threshold the reduction is `min(range_db, (ratio - 1) x (threshold_db - level))` dB,
  at or above it 0 dB, every level an RMS dBFS. Measured on 1 kHz sines at `attack_ms` = `release_ms` = 100 over
  six settings and twelve levels, every point within 0.01 dB. Far above the threshold the gate is bit-identical
  to bypass.
- **The timings move the threshold.** The detector smooths the linear power with the attack coefficient when it
  rises and the release coefficient when it falls, which biases the effective threshold by delta dB:
  **exact when `attack_ms` = `release_ms`; 0 to +3.01 dB high (the gate reduces less than written, by
  delta x (ratio - 1) dB) when `attack_ms` < `release_ms`**, +2.1 dB at 5/50 and +2.9 dB at 5/1000;
  **low (looser than written) when `attack_ms` > `release_ms`**, -4.3 dB at 100/10 and -19.4 dB at 2000/1. The
  contract is therefore the formula at the reference timing and a stated band
  of **-0.3..+3.1 dB on the effective threshold for `attack_ms` <= `release_ms`**; for `attack_ms` > `release_ms`
  the number is not promised and a review says so (section 3). The compressor's equivalent was deeper than
  written; the gate's is shallower when attack is faster than release, and looser in the other direction.
- **Noise leaks before a sine does.** A pink-noise floor 12 dB under the threshold is fully closed (29.8 of 30 dB)
  at 5/150, one 6 dB under is mostly open (3.4 dB). The documentation says to put the threshold at least 12 dB
  over the floor and shows the example above (floor about -48, threshold -34).
- **`attack_ms` and `release_ms` are not time constants.** How long the gate takes to open depends on how far
  over the threshold the signal rises: measured for a burst 6 dB over, within 1 dB of open after 0, 1, 4, 14 ms at
  `attack_ms` 2, 10, 50, 200, and 20 dB of reduction after 6, 63, 322 ms at `release_ms` 10, 100, 500. The check
  asserts the ordering and records the numbers.
- **Onset and latency: 0 samples, nothing to compensate (#843).** An isolated impulse peaks on the same sample
  and the length is unchanged, so the gate adds no `apad...atrim`. The attack still shapes the head of a word:
  the first sample of a burst over 0.01 comes 0 / 2 / 5 samples after bypass at attack 0.1 / 5 / 50 ms. That is
  the gate opening, not latency, and the documentation says a slow attack softens an onset.
- **Position.** In the author's order in `audio_effects`, before `volume` (ADR-0169), so a duck is not undone;
  a gate therefore reads the level before the fader, and is normally first in the list, ahead of a compressor
  that would lift the floor toward the speech (a review, section 3). The list runs in float (ADR-0179 section 2).

### 3. Findings

| Code | Class | Fires when | Repair |
| --- | --- | --- | --- |
| *(schema)* | error | a key missing or unknown, a keyframe list on any parameter (#842) | the existing schema errors |
| `E-GATE-RANGE` | error | a `noise_gate` key outside its range | a value: the nearest bound |
| `R-GATE-ATTACK-SLOWER` | review | `attack_ms` > `release_ms` | none |
| `R-GATE-AFTER-COMPRESSOR` | review | an enabled `noise_gate` comes after an enabled `compressor` in the same list | none |
| `N-GATE-RATIO-1` | note | `ratio` is 1 (the member does nothing) | none |
| `N-GATE-RANGE-0` | note | `range_db` is 0 (the member does nothing) | none |
| `N-GATE-STACKED` | note | more than one enabled `noise_gate` on one element | none |

- **`R-GATE-ATTACK-SLOWER`** is the finding that makes section 2's band honest: with a slow attack and a fast
  release the effective threshold reads low and the gate stays open under the written level, by an amount that
  grows with the gap. It is a review, not an error, because a slow-attack gate can be deliberate.
- **`R-GATE-AFTER-COMPRESSOR`**: a compressor with ratio over 1 or any make-up lifts the floor toward the
  speech, narrowing the gap the gate needs. It can be deliberate (a compressor feeding a gate for a gated
  effect), so it is a review; swapping changes the sound, so there is no repair.
- Not taken: a review for a threshold above -3 (a full-scale sine reads -3.01, so such a gate silences
  everything; this is the same kind of definite contradiction `E-LIMITER-ABOVE-MASTER` is an error for, but
  there is no master to compare, and `E-DYNAMICS-RANGE` stays the range's only error). Whether it should be a
  note is open (section 8).

### 4. Bypass

`"enabled": false` emits no filter: the PCM equals the member's absence and a document with no gate member
produces the graph it produced before (ADR-0173 section 5).

### 5. Not animatable

`agate` accepts `asendcmd` commands and ignores them: threshold, range, ratio, attack and release each left the
gain unchanged (0.00 dB after the command, against -60 dB had the threshold been static), while the control,
`volume`, moved by -6.0206 dB in the same harness (FFMPEG-FILTERS section 4.2, `agate_config_input` is never
re-run). Under #842 a filter that drops or ignores commands is never admitted, so **every key is a static
literal and a keyframe list on one is a schema error**. This is a named departure from Premiere, whose gate
parameters are keyframable.

### 6. The measured check (conforms to ADR-0173)

- [ ] **Conforms to ADR-0173.**
- [ ] **Static curve.** Metric: RMS of a rendered 1 kHz sine (PCM before the encoder, last second of a 3 s
  tone) in dBFS, minus the input's RMS. Fixture: lavfi sines at RMS -70, -60, -50, -45, -40, -36, -33, -25, -15
  and -3 dBFS for `(threshold, ratio, range)` = (-30, 2, 80), (-30, 4, 80), (-30, 10, 80), (-40, 3, 12),
  (-20, 100, 40) and (-30, 1, 80), at `attack_ms` = `release_ms` = 100. Expected: section 2's formula. Two-sided
  **+/-0.3 dB, provisional**: the evidence run reads it to 0.01 dB, so the figure is ADR-0180's, not the
  filter's noise; ADR-0173 section 4 replaces it with max(2 x the three-leg spread, the meter's resolution).
  Levels within 1.5 dB of the threshold are the transition and are not asserted.
- [ ] **Timing bias.** The same tone at R = 2, deep under the threshold, settled (12 s), through ten
  `(attack, release)` pairs with `attack_ms` <= `release_ms`: realised minus formula within **-0.3..+3.1 dB**;
  the pairs with attack = release read 0 within 0.3 dB. One-sided in intent; a value outside means the filter
  changed under us. The `attack_ms` > `release_ms` values are recorded, not asserted.
- [ ] **Timing order.** A burst 6 dB over the threshold from a closed gate: a longer `attack_ms` opens
  strictly slower (2 < 10 < 50 < 200) and a longer `release_ms` closes strictly slower (10 < 100 < 500).
- [ ] **Open is identity.** A -6 dB sine through a -40 dB gate: max |out - in| after 0.5 s is **0**, exact,
  and the length is exact.
- [ ] **Latency and onset.** A lavfi impulse at 0.5 s: its peak sample is the same with and without the member,
  **0 samples, exact**, and the length is unchanged (the #843 rule: 0 samples or stay out). A burst after a
  closed gate at `attack_ms` 0.1 has its first sample over 0.01 on the same sample.
- [ ] **Commands ignored.** The five commands move nothing and the control moves by -6.0206 dB; if `agate` ever
  honours a command this check fails and section 5 can be reopened.
- [ ] **Range edges.** Every bound of every range renders, the three negative controls are rejected.
- [ ] **Bypass identity** (`enabled: false` PCM equals member-absent PCM; the no-member graph unchanged),
  **and same-build determinism** (identical bytes across runs and `-cpuflags 0`).
- [ ] **Stereo link.** A stereo source with one channel loud and one under the threshold gets the same gain
  on both (`link=average`). Not in the evidence run; the build adds it.
- [ ] **Not on TTS.** The number is read off tones; the shared narration has digital silence between
  phrases, so a gate has nothing to do on it.
- [ ] **Evidence script.** [`check_gate.py`](../research/audio-effects/gate/check_gate.py) (stdlib; exits non-zero
  when a number stops holding; results in [`measurements.json`](../research/audio-effects/gate/measurements.json),
  ffmpeg 6.1.1).
- [ ] **Build-to-build drift.** Measured here on one build. `af_agate.c`'s processing code is unchanged between
  n6.1.1, n7.1.5 and master (README, point 8), so none is expected; the three-leg table commits the answer.
- [ ] **A/B: recorded, not yet heard.** [`ab/X.m4a`, `ab/Y.m4a`, `ab/KEY`](../research/audio-effects/gate/README.md):
  narration plus a seeded pink floor, one gated. PCM: pause RMS -51.1 -> -81.1 dBFS (-30.0 dB), speech RMS
  unchanged (0.00 dB). The owner's blind verdict goes in `VERDICT.md`; this box stays open until then, which is
  why the status is `proposed`.

### 7. Departures from the Premiere precedent

[`PRECEDENT.md`](../research/audio-effects/PRECEDENT.md) section 6: Premiere's Dynamics has AutoGate
(threshold, attack, hold, release) and a separate Expander (threshold, ratio); CapCut has none confirmed.

- **No hold.** `agate` has none (Premiere's AutoGate does). The release is the only anti-chatter control.
- **A ratio and a range on the gate.** Premiere splits gate and expander; this member is the expander's ratio
  with the gate's range, one member to learn. `ratio` 100 is a gate and 2 an expander.
- **RMS detection, hard knee, linked stereo, no side-chain, no look-ahead**, all fixed (whether Premiere exposes
  any of the five is recalled, not quoted).
- **Static parameters** (section 5), against Premiere's keyframable controls.
- **The ratio is remapped** at render (section 2); no Premiere parameter has that indirection.

## Consequences

- The schema gains one `audio_effects` member and a `validate` pass for it (one error, two reviews, three
  notes). The tools that read a document describe it and the RMS convention.
- `CONTEXT.md`'s **Audio effect** entry names the member.
- The compressor ADR's `R-DYNAMICS-ORDER` is unchanged; the gate adds its own order review.
- Not decided here (the map's fog): an expander as its own member, a hold, a side-chain gate, a de-esser or
  noise reduction (restoration needs a recorded-voice fixture), animating any parameter (#842).

## Owner calls

1. **Hold.** Stay without it (this ADR), or require one built from other filters before the gate enters.
2. **One member or two.** `noise_gate` carrying `ratio` and `range_db` (this ADR), or a hard `noise_gate`
   (threshold, attack, release, range; ratio fixed high) plus a later `expander`.
3. **No defaults**, as for the compressor (the minority call of ADR-0180). Five keys for "gate the hiss".
4. **`range_db` as a positive depth** (this ADR) against a negative `floor_db`.
5. **The ratio remap.** The author's `ratio` is n:1, the render halves ffmpeg's (section 2). The alternative is
   to expose ffmpeg's number and document "2 dB per dB", which is less honest.
6. **`attack_ms` > `release_ms`**: a review (this ADR) or an error. The gate's threshold moves down to 19 dB.
7. **Ranges**: `threshold_db` -80..0, `ratio` 1..100, `attack_ms` 0.1..2000, `release_ms` 1..9000,
   `range_db` 0..80, mirroring ADR-0180 where the member has an analogue.
8. **A note for a threshold above -3 dBFS** (it silences a full-scale sine).
9. **The ear.** Accept the numbers alone, or listen to the A/B first (ADR-0173). The compressor's A/B was
   "the same to me"; the gate's pause attenuation is 30 dB on PCM and a recorded-voice fixture does not yet exist.
10. **The latency rule's citation.** The brief for this draft pointed at ADR-0143 for "0-sample onset or stay
    out"; that ADR is the render encoder. The rule is #843 (as ADR-0179 and ADR-0180 cite it), and this ADR does too.

## Evidence

No court has been convened for this draft. The numbers are `check_gate.py` and `prototype_gate.py` on
ffmpeg 6.1.1; the departures from Premiere are listed in section 7; Premiere's parameter ranges are recalled,
not quoted ([`PRECEDENT.md`](../research/audio-effects/PRECEDENT.md) section 6).
