---
status: proposed (becomes accepted once the render-test table in §6 is committed with its three-leg spread; until then the ±0.1 LU tolerance is provisional)
---

# `normalize_loudness` is a singular audio effect with one authored `target_lufs` and one fixed gain

[Capability ADR and hand-off spec: per-element loudness normalisation](https://github.com/MBehtemam/Montagent/issues/844)
on the audio map ([#795](https://github.com/MBehtemam/Montagent/issues/795)). The shape was ruled in
[#816](https://github.com/MBehtemam/Montagent/issues/816): a singular `audio_effects` member with a
required `target_lufs`, measured over the element's own placed window, one fixed gain, never `loudnorm`,
and undefined loudness gets no gain and a finding. This ADR fixes what #816 left to it: the name, the
range, how a bed's target is written, the finding list and the measured check. It conforms to
[ADR-0169](0169-audio-effects-are-an-ordered-list-before-volume-and-gain-and-routing-stay-flat.md),
[ADR-0170](0170-audio-levels-are-written-in-db-and-their-keys-say-so-volume-stays-the-one-linear-level.md)
and
[ADR-0173](0173-every-audio-capability-is-accepted-by-a-measured-repo-test-and-verify-checks-only-what-master-declares.md).

## Decisions

### 1. The member

```json
{ "name": "normalize_loudness", "target_lufs": -23 }
```

- It is **singular** (ADR-0169): a second enabled copy on one element is that ADR's error, not a new one.
- **`target_lufs` is required, with no default and no preset.** The range is **−40.0 to −6.0
  inclusive**, a finite number. Below −40 the gain pushes a quiet bed toward the floor where the integrated
  meter stops being trustworthy; above −6 the element is louder than any delivery target, so the master
  limiter would be doing the loudness work this member exists to do first. The range holds the prototype's
  values (−23, −38) and every delivery target in [ADR-0172](0172-the-master-stage-is-a-top-level-loudness-target-and-true-peak-ceiling-reached-by-one-measured-gain.md)'s
  review band (−31 to −9), and sits inside the master's own legal span (−40 to −5).
- It takes no `window`, `gate` or `mode`. A keyframe list on `target_lufs` is a schema error: no
  first-wave parameter is animatable ([#842](https://github.com/MBehtemam/Montagent/issues/842)), and
  `target_lufs` is the worked example of a whole-window measurement a keyframe could not mean.

### 2. A bed's target is a number the author writes

The convention is **in the docs, not the schema**: dialogue at −23 LUFS, a music bed 15 LU under it at −38,
the owner-preferred mix of the prototype. The file holds only the literal, so exact-string replace edits it
and `validate` can range-check it.

Rejected: a relative form (`relative_to`, an offset from another element) because it adds a reference graph
to a format whose audio elements are otherwise self-contained; a `role` or `"bed"` token because it is a
second vocabulary and a hidden default that hides the 15 LU an agent should reason about; per-type presets,
which #816 already refused. The cost is one subtraction by the agent, and a wrong number is caught only by
the lift review in §4. A third convention figure for sound effects (−30) was offered by a juror and is not
written down: nothing measured supports it.

### 3. What is measured, and where

- The loudness is measured **on the element's placed window, at the member's position in the list**: after
  `atrim`, `atempo`, `aloop` and any members before it, so an EQ ahead of it is part of what it measures
  (ADR-0169's chain). One gain, `target_lufs − measured`, is applied there and held for the whole element.
- **Partial renders and `preview` use the whole placed window's gain**, as ADR-0172 does for the master: a
  `--from/--to` render measures the element's full window, so a clip sounds the same whichever range was
  rendered. The cost is that a preview decodes each normalised element in full.
- **Undefined loudness gets no gain.** Silence, or a window shorter than one 400 ms gating block, makes the
  meter print its −70 LUFS gate floor, which is read as "undefined", never as a loudness
  ([`check_normalize_loudness.py`](../research/audio-effects/loudness-normalisation/check_normalize_loudness.py)
  measures it: 399 ms is undefined, 400 ms is defined). The element renders unchanged and §4 reports it.
- The master's gain is applied afterwards, on the sum (ADR-0172). `volume` is post-fader: the final level of
  a normalised element is `target_lufs + 20·log10(volume)`, and a keyframed `volume` (a duck) moves it
  deliberately.
- It is never `loudnorm`: no dynamic gain, no true-peak limiting, no second pass.

### 4. Findings

Following ADR-0043, only errors carry a `repair`.

| Code | Class | Fires when | Repair |
| --- | --- | --- | --- |
| *(schema)* | error | `target_lufs` missing, a stray key, a keyframe list, or a second enabled copy | the existing schema errors (ADR-0169's singular error among them) |
| `E-NORMALIZE-TARGET-RANGE` | error | `target_lufs` is outside −40..−6 or not finite | a value: the nearest bound |
| `E-NORMALIZE-NO-AUDIO` | error | the element's source has no audio stream | `"none"` with a census (the fix forks: drop the member or change the source; refuse-class, ADR-0120) |
| `R-NORMALIZE-ABOVE-MASTER` | review | `master.target_lufs` is set and the element's `target_lufs` is above it | none |
| `R-NORMALIZE-LIFT` | review | at render time, the applied gain is above +20 dB (wrong asset or wrong target; it lifts the noise floor) | none |
| `N-NORMALIZE-UNDEFINED` | note | at render time, loudness was undefined, so no gain was applied | none |

- The last two are measurement-time findings, reported by `render` and `preview` as ADR-0172's are, because
  they depend on pass 1. The +20 dB figure is ADR-0172's own, taken from the same court split (2/3).
- A note, not a review, for undefined loudness: the render is well defined and the element is unchanged. A
  review, not an error, for the lift: it can be intentional, and an error must carry a repair.
- Not taken: a note for `volume` ≠ 1 beside the member (a ducked bed always has keyframes, so it would fire on
  the lead workflow every time); a review that a later compressor or limiter is a no-op (it needs a peak
  estimate `validate` does not have).

### 5. Bypass

`"enabled": false` renders no gain and no measurement pass: the PCM equals the member's absence, and a
document with no `normalize_loudness` produces the graph it produced before (ADR-0173 §5).

### 6. The measured check (conforms to ADR-0173)

- [ ] **Conforms to ADR-0173.**
- [ ] **Level.** Metric: integrated loudness (`ebur128`, ADR-0173's pinned parse) of the rendered PCM, before
  the encoder, over the element's placed window. Fixture: lavfi pink noise, 5 s. Expected: the `target_lufs`
  written (−23, −38, −14, −6). Two-sided, **±0.1 LU, provisional**: the evidence script reads the target to
  0.05 LU on one leg; ADR-0173 §4 makes the tolerance max(2 × the three-leg spread, 0.1 LU) once the table
  exists.
- [ ] **Window.** A fixture whose first two seconds are loud and the rest quiet, trimmed to the quiet part
  and also run under `speed: 2` and with `loop`. The gain must come from the placed window: the whole-file
  and window loudness differ by 27 LU on the evidence fixture, so a whole-file measurement fails the test.
- [ ] **Undefined.** Silence, and pink noise at 399 ms: no gain, the graph carries no `volume=` for the
  member, and `render` reports `N-NORMALIZE-UNDEFINED`.
- [ ] **The 15 LU gap.** The shared narration-over-bed fixture with the voice at −23 and the bed at −38:
  the rendered gap between the two elements' own integrated loudness is 15 LU, same tolerance.
- [ ] **Bypass identity.** `enabled: false` PCM equals member-absent PCM; the no-member graph string is
  unchanged.
- [ ] **Evidence script.** [`check_normalize_loudness.py`](../research/audio-effects/loudness-normalisation/check_normalize_loudness.py)
  (stdlib; exits non-zero when a number stops holding; results in
  [`measurements-check.json`](../research/audio-effects/loudness-normalisation/measurements-check.json)).
- [x] **A/B.** [`VERDICT.md`](../research/audio-effects/loudness-normalisation/VERDICT.md) with the clips and
  `KEY` (2026-10-08, ffmpeg 6.1.1): blind, the owner preferred the normalised mix; the only difference heard
  was the quieter second voice line falling away without it. The bed was not remarked on.

## Consequences

- The schema gains one `audio_effects` member and a `validate` pass for it (two errors, two reviews, one
  note). `render` and `preview` gain a measurement pass per enabled member, and the tools that read a
  document describe the member and its measured gain.
- `CONTEXT.md`'s **Audio effect** entry names the first-wave members.
- Not decided here (the map's fog): a measured-gain readback tool; normalising a mixed group of elements; a
  loudness-range target.

## Evidence

One three-juror court (Opus, Sonnet, Fable), run blind with no recommendation in the packet, together with
the EQ and dynamics questions. The owner ruled with the Judge. The packet and ballots are in
[`docs/research/juries/audio-eq-dynamics-loudness/`](../research/juries/audio-eq-dynamics-loudness/BALLOTS.md).

- An authored absolute bed target, never relative or a preset: 3/3. Out-of-range as an error: 3/3. The +20 dB
  lift as a review: 3/3. Undefined loudness as a note: 3/3. A lavfi-noise level test: 3/3.
- The range was a three-way split (−50..−10, −40..−5, −40..−6); the Judge took −40..−6. "No audio stream" as
  an error and "above the master" as a review are Juror 1's alone, taken because they match ADR-0176's
  refuse-class pattern and ADR-0172's own master review. Juror 1's note for `volume` ≠ 1 and Juror 3's
  review for a no-op dynamics member are not taken (§4). Juror 3's ±0.5 LU tolerance is not taken: the
  meter reads to 0.05 LU.

Precedent: Premiere's *Essential Sound > Loudness > Auto-Match* sets a clip's gain to a per-type target
(dialogue −23 LUFS confirmed) and CapCut's *Normalize loudness* applies one fixed −23 LUFS gain
([`PRECEDENT.md`](../research/audio-effects/PRECEDENT.md) §1). Premiere's other per-type targets are
unconfirmed there, and neither product writes a bed 15 LU under the voice; that figure is the prototype's,
heard and preferred by the owner.
