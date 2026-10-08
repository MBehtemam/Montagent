---
status: proposed (becomes accepted once the render-test table in §6 is committed with its three-leg spread; until then the tolerances there are provisional. The compressor is not ear-confirmed on this fixture, §6)
---

# The compressor and the limiter are two audio effects with no defaults and an RMS threshold

[Capability ADR and hand-off spec: compressor and limiter](https://github.com/MBehtemam/Montagent/issues/846)
on the audio map ([#795](https://github.com/MBehtemam/Montagent/issues/795)). The shape was ruled in
[#818](https://github.com/MBehtemam/Montagent/issues/818): two non-singular members, `compressor` on
`acompressor` with RMS detection and `limiter` on `alimiter level=0 latency=1`. This ADR fixes the names,
the ranges, whether anything defaults, the finding list and the measured check, and it records what the
evidence run found about the compressor's timings, which changes how its contract is stated. It conforms to
[ADR-0169](0169-audio-effects-are-an-ordered-list-before-volume-and-gain-and-routing-stay-flat.md),
[ADR-0170](0170-audio-levels-are-written-in-db-and-their-keys-say-so-volume-stays-the-one-linear-level.md),
[ADR-0172](0172-the-master-stage-is-a-top-level-loudness-target-and-true-peak-ceiling-reached-by-one-measured-gain.md),
[ADR-0174](0174-the-master-limiter-runs-at-four-times-the-rate-and-ceiling-dbtp-promises-the-delivered-file-within-one-db.md)
and
[ADR-0173](0173-every-audio-capability-is-accepted-by-a-measured-repo-test-and-verify-checks-only-what-master-declares.md).

## Decisions

### 1. Two members; every key is required and nothing defaults

```json
{ "name": "compressor", "threshold_db": -24, "ratio": 3, "attack_ms": 20, "release_ms": 250, "makeup_db": 4 }
{ "name": "limiter",    "ceiling_db": -3, "release_ms": 50 }
```

| Member | Key | Range |
| --- | --- | --- |
| `compressor` | `threshold_db` | −60..0 |
| | `ratio` | 1..20 |
| | `attack_ms` | 0.1..2000 |
| | `release_ms` | 1..9000 |
| | `makeup_db` | 0..24 |
| `limiter` | `ceiling_db` | −24..0 |
| | `release_ms` | 1..1000 |

- Both are non-singular (ADR-0169): two compressors in series, or a compressor and a limiter, are ordinary.
- **No key defaults.** This is ADR-0145's rule and the rule #816 set for `target_lufs`; an agent that writes a
  compressor chose its timings, and a missing key is a clean error naming the keys. The cost is five keys for
  "squash it a bit"; the documentation carries a worked example (the one above), which an agent copies and
  edits by exact-string replace.
- The ranges sit inside `acompressor`'s own limits (threshold 0.000977..1 linear, ratio 1..20, attack
  0.01..2000 ms, release 0.01..9000 ms, make-up gain 1..64 linear), so no accepted document can fail at
  render time. The limiter's lower bound is `alimiter`'s own: its `limit` floor is 0.0625 linear, −24.08 dB
  (a −30 ceiling, which one juror proposed, fails at render with "out of range"). `makeup_db` is non-negative:
  a negative one is `volume`'s job.
- Premiere's Compressor adds a knee and its Limiter a look-ahead; neither is authorable here, for the
  reasons in §2. Ratios above 20 (Audition goes to 30:1) are the limiter's job.

Rejected: attack, release and makeup defaults (two jurors, with figures that disagreed with each other: 10 ms
and 100 ms against 20 ms and 250 ms; a default is a number the file does not show, and which one is "the
industry idle value" is exactly the judgement the format should not hide). A `mode: peak|rms` key
(peak detection read about 3 dB off in the prototype, so it is fixed). A `knee_db` key (a soft knee has no
closed form to check against).

### 2. How each is rendered, and what the numbers mean

- **`compressor`** is `acompressor` with `detection=rms`, `link=average`, `knee=1` (hard) and the dB values
  converted to ffmpeg's linear. **`threshold_db` and every compressor level are RMS dBFS**, so a full-scale
  sine reads −3.01. A reader who thinks in peak is about 3 dB off, which is why the key's documentation says
  "RMS".
- **`limiter`** is `alimiter=limit=<ceiling>:attack=5:release=<release_ms>:level=0:latency=1` with no
  automatic level and no ASC. The 5 ms look-ahead is fixed, and `latency=1` makes the filter's fixed delay
  something the renderer cancels (#843), so the onset does not move. **`ceiling_db` is a sample peak**; the
  true-peak ceiling of the delivered file is the master's (ADR-0172, ADR-0174), and the two are not the same
  number.
- **What the compressor's curve is.** Above the threshold the output follows
  `threshold + (in − threshold) / ratio + makeup`, below it `in + makeup`, with every level an RMS dBFS. That
  holds to **0.001 dB at the reference timing, `attack_ms` = `release_ms` = 100**. At faster timings the
  envelope follower follows the tone's own ripple and the realised reduction is **deeper than the formula**:
  1.6 dB deeper at 5 ms / 50 ms, 2.2 dB at 5 ms / 1000 ms, 0.8 dB at 50 ms / 120 ms, measured on ffmpeg 7.1.5
  across 1 to 200 ms attack and 10 to 1000 ms release. It is never shallower than the formula by more than
  0.3 dB. The contract is therefore stated as **the formula at the reference timing, and a band of
  −2.5 dB..+0.3 dB around it at any timing**, and the documentation says that a faster compressor is deeper
  than the number suggests. It is the property of the filter, not a defect to repair, and the check pins it
  so an ffmpeg upgrade that moves it is seen.
- **`attack_ms` is not a 63% time.** On a −30 to −6 dB step the gain reduction reached 63% of its final
  value after 2.0 ms for `attack_ms` 5 and 8.0 ms for `attack_ms` 50. The check asserts the ordering (a longer
  attack is slower) and records the numbers; it does not assert that `attack_ms` equals a time constant.

### 3. Findings

| Code | Class | Fires when | Repair |
| --- | --- | --- | --- |
| *(schema)* | error | a key missing or unknown, a keyframe list on any parameter ([#842](https://github.com/MBehtemam/Montagent/issues/842)) | the existing schema errors |
| `E-DYNAMICS-RANGE` | error | a `compressor` or `limiter` key outside its range | a value: the nearest bound |
| `E-LIMITER-ABOVE-MASTER` | error | `master.ceiling_dbtp` is set and a `limiter`'s `ceiling_db` is above `ceiling_dbtp − 1` | a value: `ceiling_dbtp − 1` |
| `R-DYNAMICS-ORDER` | review | an enabled `compressor` comes after an enabled `limiter` in the same list | none |
| `R-COMPRESSOR-MAKEUP-CLIP` | review | `makeup_db` exceeds `max(0, (−3 − threshold_db) · (1 − 1/ratio))`, no enabled `limiter` follows it in the list, and `master.ceiling_dbtp` is unset | none |
| `N-COMPRESSOR-RATIO-1` | note | `ratio` is 1 (the member is a plain `makeup_db` gain) | none |
| `N-LIMITER-STACKED` | note | more than one enabled `limiter` on one element | none |

- **`E-LIMITER-ABOVE-MASTER`.** A sample-peak ceiling above the master's true-peak ceiling less the 1 dB gap
  ADR-0174 allows cannot bind: the master's ceiling already governs the delivered file. It is an error
  because the contradiction is definite and the repair is a value. It fires only when `master.ceiling_dbtp`
  is present; with no master it cannot be wrong.
- **`R-COMPRESSOR-MAKEUP-CLIP`** is a static estimate for a full-scale sine and can be wrong either way, so
  it is a review. ADR-0169 anticipated the sibling: a per-element limiter under `volume` > 1 is not held, and
  the master (ADR-0172) is what holds the file.
- **`R-DYNAMICS-ORDER`.** Compressor after limiter re-exposes the peaks the limiter just removed. It can be
  deliberate (a "glue" chain), so it is a review. It is not an error because no repair is a guess: swapping
  changes the sound.
- Not taken: Juror 3's review for a fast attack with a high ratio (the sweep shows the realised curve is
  already deeper there, and "brick wall" is a taste); Juror 1's note for `attack_ms` above `release_ms`.

### 4. Where it sits in the chain

Both run in the `audio_effects` list in the author's order, before `volume` (ADR-0169), so a duck written as
`volume` keyframes (ADR-0177) is not undone by a compressor. A compressor therefore reads the level *before*
the fader, and `volume` > 1 pushes a limited element back over its ceiling; the master is what holds the file.
The list runs in float (ADR-0179 §2).

### 5. Bypass

`"enabled": false` emits no filter: the PCM equals the member's absence and a document with no dynamics
member produces the graph it produced before (ADR-0173 §5).

### 6. The measured check (conforms to ADR-0173)

- [ ] **Conforms to ADR-0173.**
- [ ] **Compressor, static curve.** Metric: RMS of a rendered 1 kHz sine (PCM before the encoder, last
  second of a 3 s tone) in dBFS. Fixture: lavfi sines at −50, −40, −30, −20, −12, −6 and 0 dBFS peak, for
  `(threshold, ratio, makeup)` = (−20, 4, 0), (−20, 4, 6), (−30, 8, 0), (−20, 1, 0) and (−12, 2, 3), at
  `attack_ms` = `release_ms` = 100. Expected: the formula of §2. Two-sided **±0.3 dB, provisional**: the
  evidence run reads the formula to 0.002 dB, so the tolerance is the old prototype's figure, not the filter's
  noise; ADR-0173 §4 replaces it with max(2 × the three-leg spread, the meter's resolution).
- [ ] **Compressor, timing sweep.** The same tone through eight `(attack, release)` pairs from 1/10 to
  200/1000 ms: the realised curve minus the formula lies within **−2.5..+0.3 dB**. One-sided in intent; a
  value outside it means the filter changed under us.
- [ ] **Compressor, step.** A −30 to −6 dB step: the reduction is slower for `attack_ms` 50 than 5 by more
  than a factor of two. The build adds the same ordering for `release_ms` (not exercised by the evidence
  script).
- [ ] **Limiter.** Metric: sample peak of a +6 dB sine through each of ceilings −3, −12, −20 and −24, after the
  first 0.5 s. Expected: the ceiling, two-sided **±0.0002 dB, provisional** (the evidence run reads the
  ceiling to 0.0000002 dB; the prototype's 0.00001 dB hold is what the figure bounds). A tone 6 dB under the
  ceiling passes within **0.01 dB**. A burst after 100 ms of silence has its first sample over 1e-4 on
  **the same sample** as the unprocessed burst (onset shift **0**, exact).
- [ ] **Range edges.** Every bound of every range renders without an ffmpeg error, and a `limiter` ceiling of
  −30 is rejected by `alimiter` (the negative control for the −24 floor). The evidence script checks both.
- [ ] **Bypass identity.** `enabled: false` PCM equals member-absent PCM; the no-member graph is unchanged.
- [ ] **Not on TTS.** Neither member's number is read off the speech fixture: the compressor was not told
  apart on it and its transfer is exact on tones.
- [ ] **Evidence script.** [`check_dynamics_curves.py`](../research/audio-effects/dynamics/check_dynamics_curves.py)
  (stdlib; exits non-zero when a number stops holding; results in
  [`measurements-check.json`](../research/audio-effects/dynamics/measurements-check.json), ffmpeg 7.1.5).
- [x] **A/B (limiter).** [`VERDICT.md`](../research/audio-effects/dynamics/VERDICT.md), clips and `KEY`
  (2026-10-08, ffmpeg 6.1.1): the owner picked the limited narration as the steadier; the unlimited one, over
  full scale, "sometimes has raising and falling".
- [ ] **A/B (compressor): recorded, not ear-confirmed.** The same verdict: "the same to me", even at
  threshold −28 dB, ratio 6, makeup +8. The numbers say it works (mix peak −2.8 to −6.1 dBFS, loudness −18.0 to
  −21.4 LUFS before matching), and the likely reason is the material, an already even narration. ADR-0173 asks
  for the owner's ear next to the number; this box stays open until the owner either accepts the numbers alone
  or hears a harsher fixture. That is why the status is `proposed`.

## Consequences

- The schema gains two `audio_effects` members and a `validate` pass for them (two errors, two reviews, two
  notes). The tools that read a document describe each member and the RMS convention.
- `CONTEXT.md`'s **Audio effect** entry names the first-wave members.
- Not decided here (the map's fog): an expander or gate; a knee or a look-ahead key; a sidechain compressor
  (a duck stays a script, ADR-0177); animating any parameter (#842); a multiband compressor.

## Evidence

One three-juror court (Opus, Sonnet, Fable), run blind with no recommendation in the packet, together with
the loudness and EQ questions. The owner ruled with the Judge. The packet and ballots are in
[`docs/research/juries/audio-eq-dynamics-loudness/`](../research/juries/audio-eq-dynamics-loudness/BALLOTS.md).

- RMS detection with a threshold in RMS dBFS, a fixed 5 ms look-ahead, `ceiling_db` as sample peak, makeup
  0..24 and a static-curve-plus-burst check with no TTS: 3/3.
- **No defaults: 1/3** (Juror 1). Jurors 2 and 3 gave attack, release and makeup defaults, and the Judge
  took Juror 1's reading because it follows ADR-0145 and #816; this is a minority taken on a principle, and
  is the item the owner may reopen. `attack_ms` 0.1..2000 and `release_ms` 1..9000 are Juror 1's, kept
  inside `acompressor`'s limits. `ceiling_db` was −30..0 (Juror 1) or −20..0 (Jurors 2 and 3); it is −24..0,
  the filter's own floor, which no ballot had.
- The limiter's ceiling against the master as an error: 2/3 (Jurors 2 and 3; Juror 1 said review); the `− 1`
  form is Juror 3's. Compressor after limiter as a review: 2/3 (Juror 3 said error). The makeup-clip review:
  3/3 (the formulae differ; the one here is the plain static estimate).
- The court's expected static curve is true only at the reference timing; none of the three ballots could
  have known (§2). The evidence run found it.

Precedent: Premiere's Compressor (threshold, ratio, attack, release, make-up) and Limiter (threshold,
release), with threshold in dBFS and ratio n:1
([`PRECEDENT.md`](../research/audio-effects/PRECEDENT.md) §6). CapCut has no confirmed first-party equivalent.
