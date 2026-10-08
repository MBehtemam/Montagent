---
status: proposed (becomes accepted once the render-test table in §6 is committed with its three-leg spread; until then the tolerances there are provisional)
---

# EQ is four stackable audio effects built from Butterworth biquad sections

[Capability ADR and hand-off spec: EQ](https://github.com/MBehtemam/Montagent/issues/845) on the audio map
([#795](https://github.com/MBehtemam/Montagent/issues/795)). The shape was ruled in
[#817](https://github.com/MBehtemam/Montagent/issues/817): several small stackable, non-singular
`audio_effects` members instead of a band list, rendered by ffmpeg biquads only. This ADR fixes the names,
the slope set, the ranges, the finding list and the measured check, and settles one thing the ticket only
asserted: how a 24 or 48 dB/oct slope still reads −3.01 dB at its cutoff. It conforms to
[ADR-0169](0169-audio-effects-are-an-ordered-list-before-volume-and-gain-and-routing-stay-flat.md),
[ADR-0170](0170-audio-levels-are-written-in-db-and-their-keys-say-so-volume-stays-the-one-linear-level.md)
and
[ADR-0173](0173-every-audio-capability-is-accepted-by-a-measured-repo-test-and-verify-checks-only-what-master-declares.md).

## Decisions

### 1. Four members, every key required, no defaults

```json
{ "name": "highpass", "frequency_hz": 100,  "slope_db_per_oct": 24 }
{ "name": "lowpass",  "frequency_hz": 8000, "slope_db_per_oct": 12 }
{ "name": "shelf",    "side": "high", "frequency_hz": 4000, "gain_db": 3 }
{ "name": "bell",     "frequency_hz": 1500, "gain_db": -6, "q": 1.4 }
```

| Key | Values | Notes |
| --- | --- | --- |
| `frequency_hz` | 20..20000 | the cutoff of a pass filter, the corner of a shelf (half its gain), the centre of a bell |
| `slope_db_per_oct` | `12`, `24` or `48` | pass filters only |
| `side` | `"low"` or `"high"` | shelf only |
| `gain_db` | −24..+24 | shelf and bell |
| `q` | 0.1..10 | bell only |

- All four are non-singular and stackable (ADR-0169): a high-pass and three bells is ordinary.
- There is **no default**, so `{"name": "bell"}` is an error naming the missing keys. This is the rule of
  ADR-0145 (literal values, checkable by `validate`) and of #816's `target_lufs`; for an EQ the defining
  numbers are exactly the ones an agent must choose.
- The mix rate is fixed at 48 kHz ([`MIX_RATE`](../../crates/montagent-core/src/verbs/render.rs)), so
  20000 Hz is always below the 0.45 × rate at which a biquad's warping begins to matter. There is no
  separate Nyquist check; if the mix rate is ever made a variable, one is owed.
- The unit is in the slope's key name, as ADR-0170 requires. The set stops at 12|24|48: a 6 dB/oct slope
  is a first-order section whose behaviour differs and buys little, and 36 would need an odd order.
- Premiere's Audition-derived ranges are wider (±30 dB, Q to 100, slope from 6; recalled, [`PRECEDENT.md`](../research/audio-effects/PRECEDENT.md) §5); this narrows them to what
  ffmpeg's biquads hold exactly (§2) and to values that read as a typo when exceeded. Loosening a range later
  is compatible; tightening is not, which is the reason to start here.

Rejected: a single `eq` member with a `bands` array (ADR-0169's text-addressing cannot single out one band,
and a band is a nested object `validate` would walk twice); `shelf` split into `low_shelf` and `high_shelf`
(a juror's proposal: #817 ruled `shelf` with a `side`, and `side` is a one-word enum an agent sets the same
way it sets any other); a `type` key on one catch-all member; an authorable shelf `q` (fixed, below); a
unitless `slope`.

### 2. How each member is rendered

All four are ffmpeg biquads: zero latency, no state beyond the filter, identical across builds. The
latency rule ([#843](https://github.com/MBehtemam/Montagent/issues/843)) therefore adds nothing: there is
nothing to compensate.

- **`bell`** is `equalizer=f=<frequency_hz>:width_type=q:width=<q>:g=<gain_db>`.
- **`shelf`** is `lowshelf` or `highshelf` with `f`, `g` and `width_type=q:width=0.707107`. The shelf's Q is
  fixed, not authorable, so the corner is always the point where the shelf has half its gain.
- **`highpass` and `lowpass`** are a **cascade of Butterworth sections**, one 2-pole biquad per pair of
  poles: 12 dB/oct is one section (Q 0.7071), 24 dB/oct is two (Q 1.3066 and 0.5412), 48 dB/oct is four
  (Q 2.5629, 0.9000, 0.6013 and 0.5098). A section is `highpass=f=<f>:poles=2:width_type=q:width=<Q>`.
  This is what keeps the contract honest. Cascading *identical* Q 0.7071 sections reads −6.02 dB at the
  cutoff for 24 dB/oct and −12.04 dB for 48 dB/oct, not −3.01
  ([`check_eq_biquads.py`](../research/audio-effects/eq/check_eq_biquads.py) keeps that as a negative
  control, so a regression to the naive cascade fails).
- **The list runs in float.** The renderer sets `aformat=sample_fmts=fltp` ahead of the first enabled
  `audio_effects` member and nowhere else, so a document without members keeps the graph it has today
  (ADR-0173 §5). The reason is measured: in lavfi's default `s16` a 48 dB/oct low-pass read an exact zero
  two octaves into its stopband, i.e. the filter was quantising to 2⁻¹⁵ instead of filtering.
- Members run in list order; frequencies are not scaled by `speed` (the list runs after `atempo`, ADR-0169).

### 3. Findings

| Code | Class | Fires when | Repair |
| --- | --- | --- | --- |
| *(schema)* | error | a key missing or unknown, a keyframe list on any parameter ([#842](https://github.com/MBehtemam/Montagent/issues/842)) | the existing schema errors |
| `E-EQ-RANGE` | error | `frequency_hz`, `gain_db` or `q` outside its range, or a `slope_db_per_oct` not in {12, 24, 48} | a value: the nearest bound or the nearest slope |
| `E-EQ-STACK-CAP` | error | more than 8 enabled EQ members on one element | `"none"` with a census naming the members (the fix forks: merge bands or delete one, so it is refuse-class, ADR-0120) |
| `R-EQ-GAIN-EXTREME` | review | `abs(gain_db)` is above 12 | none |
| `R-EQ-BAND-CROSSED` | review | one element has an enabled `highpass` whose `frequency_hz` is at or above an enabled `lowpass`'s | none |
| `N-EQ-NO-OP` | note | a `shelf` or `bell` has `gain_db` 0 | none |

- Only definite breakage is an error. A large gain and a crossed band pair can each be what the
  author meant, so they are reviews. The cap of 8 is Premiere's band count; past it the graph grows without a
  sound reason, and the fix forks, so it is an error with no repair.
- The 12 dB review sits below the owner's exaggerated-EQ round (a −14 dB bell, §6's A/B), so that round is
  flagged: deliberate exaggeration is what a review is for.
- Not taken: a note for identical duplicate members (ADR-0169 allows duplicates, and two identical bells
  are a deliberate doubling); a review on a very high high-pass or very low low-pass (a creative choice with
  no stated threshold to borrow).

### 4. Bypass

`"enabled": false` emits no section: the PCM equals the member's absence, and a document with no EQ member
produces the graph it produced before (ADR-0173 §5).

### 5. The ear check is on tones, not speech

The owner could not hear a gentle EQ on clean TTS, so every number below is measured on lavfi sines, not on
the fixture. The speech A/B in §6 is recorded as what was heard, not as the acceptance.

### 6. The measured check (conforms to ADR-0173)

- [ ] **Conforms to ADR-0173.**
- [ ] **Pass filters.** Metric: RMS of a rendered 1 s sine (PCM before the encoder, after the first 0.5 s)
  against the same tone unprocessed. Fixture: lavfi sines at `fc`, `fc/4` and `4·fc`, for `highpass` and
  `lowpass` at each slope with `fc` 1000 Hz. Expected: **−3.01 dB at `fc`**, two-sided **±0.15 dB**;
  passband (4·fc for a high-pass, fc/4 for a low-pass) 0 dB ± 0.3; two octaves into the stopband **at least**
  the ideal Butterworth figure + 1.5 dB (−22.6, −46.7 dB) and never asked for more than −75 dB (the
  48 dB/oct high-pass floors near −78 dB). The evidence run measured −3.012..−3.014 dB at all six
  pass-filter cases.
- [ ] **Bell.** Same metric. `gain_db` at the centre, two-sided **±0.15 dB** for ±6 and ±12 (measured
  within 0.006), and 0 ± 0.5 dB three octaves away.
- [ ] **Shelf.** Same metric. `gain_db` on the plateau and half of it at the corner, two-sided **±0.3 dB**
  (measured within 0.02 and 0.003), and 0 ± 0.3 dB on the far side, for both `side`s at ±6.
- [ ] **Range edges.** Every bound of every range (frequency 20 and 20000, gain ±24, Q 0.1 and 10, all slopes)
  renders without an ffmpeg error, so a document that passes `validate` never fails at render. The evidence
  script checks all of them.
- [ ] **Negative control.** The naive cascade must read more than 0.5 dB away from −3.01 at 24 and 48
  dB/oct. This keeps the check able to fail.
- [ ] **Float.** The same pass-filter run in the repo's Rust render test, so the `fltp` ahead of the list is
  exercised.
- [ ] **Bypass identity.** `enabled: false` PCM equals member-absent PCM; the no-member graph is unchanged.
- [ ] **Tolerances are provisional** (the figures above are the evidence script's single leg). ADR-0173 §4
  makes each max(2 × the three-leg spread, the meter's resolution) once that table is committed.
- [ ] **Evidence script.** [`check_eq_biquads.py`](../research/audio-effects/eq/check_eq_biquads.py) (stdlib;
  exits non-zero when a number stops holding; results in
  [`measurements-check.json`](../research/audio-effects/eq/measurements-check.json), ffmpeg 7.1.5).
- [x] **A/B.** [`VERDICT.md`](../research/audio-effects/eq/VERDICT.md), the clips and both `KEY`s (2026-10-08,
  ffmpeg 6.1.1). A gentle EQ on the speech fixture: "almost the same". An exaggerated one (400 Hz high-pass,
  +6 dB shelf at 4 kHz on the voice; a −14 dB bell and a 5 kHz low-pass on the bed): the owner picked the EQ'd
  clip as the different one, in the direction the members move it (thinner, more treble).

## Consequences

- The schema gains four `audio_effects` members and a `validate` pass for them (two errors, two reviews,
  one note). `render` gains the float format ahead of a non-empty list, and the tools that read a document
  describe each member.
- `CONTEXT.md`'s **Audio effect** entry names the first-wave members.
- Not decided here (the map's fog): a notch or band-pass member; a graphic or preset EQ; an authorable shelf
  Q; a 6 dB/oct slope; and animating any parameter (named in #842 as a departure from Premiere's keyframable
  EQ).

## Evidence

One three-juror court (Opus, Sonnet, Fable), run blind with no recommendation in the packet, together with
the loudness and dynamics questions. The owner ruled with the Judge. The packet and ballots are in
[`docs/research/juries/audio-eq-dynamics-loudness/`](../research/juries/audio-eq-dynamics-loudness/BALLOTS.md).

- `highpass`, `lowpass`, `bell`, `frequency_hz` 20..20000, a stack cap of 8, a lavfi-tone check, and a flag on a
  zero-gain stage: 3/3 (the flag is a note for Jurors 1 and 2, a review for Juror 3). A unit-named slope key:
  3/3 on the principle, with `slope_db_per_oct` from Jurors 1 and 2 and `slope_db_oct` from Juror 3.
  `shelf { side }`: 2/3 (Juror 3 would split it into `low_shelf` and `high_shelf`). `gain_db` ±24: 2/3
  (Juror 2 said ±18). `q` 0.1..10: 2/3 (Juror 1 said 0.1..18). The cap as an error: 2/3 (Juror 1 said review).
  The 12 dB review threshold: 2/3 (Juror 1 said 15).
- The Butterworth section Qs are Juror 1's alone and were taken because they are the only ballot that makes
  "−3.01 dB at the cutoff" true for every slope; Jurors 2 and 3 asserted the same figure without saying how,
  and the evidence run shows a plain cascade reads −6.02 and −12.04. The crossed-band review is also Juror 1's
  alone. Juror 2's reviews for a high-pass above 1 kHz or a low-pass below 2 kHz are not taken: they are
  taste with no source to borrow a threshold from.

Precedent: Premiere's Parametric Equalizer (per-band frequency, gain and Q, with high-pass, low-pass and
shelves) and its Highpass and Lowpass effects ([`PRECEDENT.md`](../research/audio-effects/PRECEDENT.md) §5).
CapCut's EQ is unconfirmed there, so the precedent is Premiere's.
