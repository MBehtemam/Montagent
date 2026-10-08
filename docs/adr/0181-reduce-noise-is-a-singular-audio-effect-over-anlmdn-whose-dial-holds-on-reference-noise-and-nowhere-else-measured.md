---
status: proposed (becomes accepted once the three-leg table in §7 is committed and the owner's blind listen in §7 is recorded; until then the tolerances there are provisional, and the dial's transfer to real hiss is open, §4)
---

# Noise reduction is a singular `reduce_noise` audio effect over `anlmdn`, and its dial holds on reference noise and is measured nowhere else

[Capability ADR and hand-off spec: noise reduction](https://github.com/MBehtemam/Montagent/issues/853) on the audio map
([#795](https://github.com/MBehtemam/Montagent/issues/795)). The filter, shape, singularity and placement were ruled in
[#852](https://github.com/MBehtemam/Montagent/issues/852): `anlmdn`, one singular member `reduce_noise {reduction_db}`, a measured
check next to the owner's blind A/B, and a review when it follows a `compressor`. This ADR fixes the range, the mapping from
`reduction_db` to the filter's strength, the findings and the measured check. It also records what the calibration found, which
#852 did not expect: the strength is a level threshold, so one `reduction_db` is only a dB value for hiss at one level. It conforms
to [ADR-0169](0169-audio-effects-are-an-ordered-list-before-volume-and-gain-and-routing-stay-flat.md),
[ADR-0170](0170-audio-levels-are-written-in-db-and-their-keys-say-so-volume-stays-the-one-linear-level.md),
[ADR-0173](0173-every-audio-capability-is-accepted-by-a-measured-repo-test-and-verify-checks-only-what-master-declares.md)
and [ADR-0145](0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md).

## Decisions

### 1. One member, one required dial, no default

```json
{ "name": "reduce_noise", "reduction_db": 10 }
```

- `reduction_db` is a fractional dB value in **5..20**, required, with **no default** (ADR-0145, as in ADR-0178 and ADR-0180).
  The lower bound is where the mapping in §2 was checked; a value below 5 is refused rather than extrapolated. Bypass is
  `"enabled": false`, never `reduction_db: 0`.
- The member is **singular** (ADR-0169): a second enabled `reduce_noise` on one element is the singular error. Two filters
  over one hiss compound unpredictably, and #852 Q3 ruled it.
- A keyframe list on `reduction_db` is a schema error (ADR-0146 and #842): no parameter is animatable in the first wave.

Rejected: a `strength` key or a raw `anlmdn` parameter set. Both put the filter's internal settings in the author's hands. The
renderer owns the mapping (§2), and the author sees only a dB value.

### 2. How it renders: `reduction_db` maps to `anlmdn` strength

The renderer writes one filter:

```
anlmdn=s=<s>:p=0.002:r=0.006:o=o
```

with `s = 10^(−2 + (reduction_db − 5) / 32)`, written to six significant digits. `p`, `r` and `o` are the filter's defaults,
written out so a default change in ffmpeg cannot move the graph. At the ends of the range `s` runs from 0.01 (5 dB) to 0.0294
(20 dB).

The mapping's meaning is fixed by a reference: **the drop, in dB, the filter makes on seeded stationary white noise at
−50.0 dBFS RMS, measured over 1.0–2.5 s.** [`measurements.json`](../research/audio-effects/noise-reduction/measurements.json) (ffmpeg 7.1.5):

| `reduction_db` | 5 | 8 | 10 | 12 | 15 | 18 | 20 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| measured drop | 5.31 | 8.16 | 10.26 | 12.41 | 15.54 | 18.34 | 19.94 |

The largest gap is 0.54 dB. The gate in §7 is ±1.0 dB, provisional. The curve is a shallow sigmoid, and the mapping is linear in
`log10(s)` between about 5 and 20 dB; the coefficients (−2 and 1/32) were fitted to the reference table and are read back by the
check, not chosen to round.

### 3. The level law: `s` is a threshold on the hiss, so the dial moves with the hiss

On the same white noise the whole curve shifts with the noise's level: each 5 dB quieter moves it one decade of `s`. The table
below holds `reduction_db` fixed and changes only the level of the noise (all with the §2 strength):

| noise (RMS) | −40 dBFS | −45 dBFS | −50 dBFS (reference) | −55 dBFS | −60 dBFS |
| --- | --- | --- | --- | --- | --- |
| drop at `reduction_db` 10 | 0.00 | 0.00 | 10.26 | 26.44 | 27.07 |
| drop at `reduction_db` 20 | 0.00 | 0.02 | 19.94 | 26.84 | 27.10 |

Read it plainly. A 10 dB quieter hiss gets the filter's full ~27 dB cap at any `reduction_db` in range. Hiss 5 dB louder than the
reference gets almost nothing even at 20 dB. So for white noise `reduction_db` is not a reduction the author can hear on the
hiss they have; it is a dB value that holds at −50 dBFS and says, at the other levels, how far the hiss must sit below it to be
reduced at all. The mechanism is that `s` is compared with the noise in signal units, so the same `s` is a different strength at
each level. The source is not read here; the curve is measured.

The law is measured on white noise at four levels, and the table is what the evidence script records. It is not gated: a
per-level gate would assert the law, not the dial, and the owner's ear is the test of what it is worth.

### 4. What the fixture shows: the dial's transfer to real hiss is unmeasured

#848's recording ([`recorded-voice`](../research/audio-effects/fixtures/recorded-voice/PROVENANCE.md)) was meant to calibrate the
mapping (#852, last paragraph). It cannot, and this is the main finding of the calibration:

- **Speech is untouched.** The change over 2–18 s is at most 0.005 dB for every `reduction_db` from 5 to 20 (§7 gate).
- **The hiss-only stretch is too short.** The first ~0.1 s is the only stretch below −50 dBFS. It rises through the window, and
  the ~0.1 s used is flat: the drop is 13.2–13.5 dB for every `reduction_db` from 5 to 20. That is the filter's start-of-file
  behaviour, not the dial, so it is recorded and not gated.
- **The pauses are not noise.** The two stretches at 11.7–12.5 s and 15.7–16.5 s sit near −38 dBFS, with speech tails in them.
  They drop 0.16–0.37 dB across the range, which is what speech bleed would look like.
- **The white-noise law does not transfer.** At the lead-in's level the law predicts 26–27 dB at `reduction_db` 10; the fixture
  gives 13 dB at every value, flat. Real hiss is coloured and non-stationary, and anlmdn's patch matching sees that structure. The
  recording has no clean hiss floor to calibrate against.

So the mapping in §2 is the reference's, and it is **not** calibrated on #848 as #852 asked. Calibrating it on this recording would
fit a number to a start-of-file transient, which is worse than the white-noise reference. The owner may reopen this, but only with a
recording that has a long hiss-only stretch (the brighter recording #848 names for de-ess would not supply one). Until then the
honest reading of `reduce_noise` on archival hiss is: **it may do nothing, and the ear, not the dial, decides.**

### 5. Findings

| Code | Class | Fires when | Repair |
| --- | --- | --- | --- |
| *(schema)* | error | `reduction_db` missing, a stray key, a keyframe list ([#842](https://github.com/MBehtemam/Montagent/issues/842)), or a second enabled copy (ADR-0169's singular error) | the existing schema errors |
| `E-NOISE-RANGE` | error | `reduction_db` outside 5..20 | a value: the nearest bound |
| `R-NOISE-AFTER-COMPRESSOR` | review | an enabled `reduce_noise` comes after an enabled `compressor` in the same list | none |

- **`E-NOISE-RANGE`** is an error because the mapping is only checked on 5..20 (§2); a value outside it has no measured meaning.
  The repair is the nearest bound, as for `E-DYNAMICS-RANGE` (ADR-0180).
- **`R-NOISE-AFTER-COMPRESSOR`** is a review. A compressor's makeup gain raises the hiss, and §3 says louder hiss is reduced
  less. A compressor before the reducer therefore works against it. A deliberate order is possible, so it is not an error (the same
  reasoning as ADR-0180's `R-DYNAMICS-ORDER`). #852 Q5 ruled placement anywhere in the list, the author's order, with this review.
- No finding checks the level of the hiss. `validate` cannot know it without analysing the signal, which the format does not do
  (the no-whole-window-analysis rule ADR-0178 applies to `normalize_loudness`).

### 6. Where it sits, and bypass

The member sits in the `audio_effects` list in the author's order, before `volume` (ADR-0169), so a duck written as `volume`
keyframes is not undone by it. `"enabled": false` emits no filter: the PCM equals the member's absence, and a document with no
noise member produces the graph it produced before (ADR-0173 §5).

The filter's fixed latency is **384 samples** at these `p` and `r`, and the renderer cancels it with `apad=pad_len=384,atrim=start_sample=384`
on the member's output, as #843 requires. Measured in [`measurements.json`](../research/audio-effects/noise-reduction/measurements.json):
a 1 kHz burst starting at 0.5 s has onset 24001 raw, 24385 through the filter, and 24001 once compensated.

### 7. The measured check (conforms to ADR-0173)

- [ ] **Conforms to ADR-0173.** The gated numbers come from synthetic signals (`anoisesrc` with a pinned seed and a `sine`), and the
  one content check is §4's fixture speech gate, which ADR-0173 §3 allows where synthetic signals cannot model the behaviour
  (speech preservation on real speech).
- [x] **Reference drop** (gated, [`check_noise_reduction.py`](../research/audio-effects/noise-reduction/check_noise_reduction.py)):
  seeded white noise at −50.0 dBFS RMS, 1.0–2.5 s; the drop equals `reduction_db` to **±1.0 dB, provisional** at every value in
  {5, 8, 10, 12, 15, 18, 20}. Measured largest gap 0.54 dB, from §2.
- [x] **Tone level** (gated): a 1 kHz sine at −23 dBFS RMS over the same noise; the window's level changes by at most **±0.5 dB,
  provisional**. Measured: at most 0.005 dB.
- [x] **Fixture speech** (gated, §4): the speech window 2–18 s changes by at most **±0.5 dB, provisional**. Measured: at most 0.005 dB.
- [x] **Latency** (gated, §6): compensated onset equals raw onset, 0 samples.
- [x] **Determinism within one build** (gated): three runs, one byte digest.
- [ ] **Bypass identity** (ADR-0173 §5): `enabled: false` PCM equals member-absent PCM. Graph-level, asserted in the build slice.
- [ ] **Three-leg table** (ADR-0173 §4): the tolerances above become `max(2 × the spread across the three legs, the meter's resolution)`.
  Not run. The measurement is on **Homebrew ffmpeg 7.1.5** on one Mac, not the BtbN floor pin in `ci/install_ffmpeg_floor.sh`, and
  no leg has been measured.
- [ ] **Owner's blind A/B** (#852 Q4): [`ab/`](../research/audio-effects/noise-reduction/ab/) holds X and Y, the fixture bypassed and
  with `reduction_db` 20, both AAC 160k and loudness-matched (−19.2 LUFS integrated, −3.1 dBFS true peak). The key is in `ab/KEY`
  and is not to be opened before the owner's verdict. The verdict is open; this ADR does not record one.

**Build-platform risk, found while measuring.** On Homebrew ffmpeg **9.0.2** `anlmdn` failed intermittently: one
`Error submitting audio frame to the encoder` in six CLI runs, and one `SIGSEGV` from a Python harness. Twenty later runs of the
same command were byte-identical, and nothing like it occurred on 7.1.5. The floor is 7.1 (ADR-0115), but CI's macOS leg runs
`brew install ffmpeg` ([`ci.yml`](../../.github/workflows/ci.yml) line 107), which is 9.0.2 today. The build slice must either
pin `ffmpeg@7` on that leg or prove the filter stable on 9.0.2 before any macOS number is recorded. This is not yet resolved.

### Calibration note, and where the evidence lives

- `anlmdn` fixed-latency and onset behaviour: 384 samples at `p=0.002`, `r=0.006`; the fixed latency is cancelled exactly.
- The reference table, the level-law table and the fixture rows are in `measurements.json`, produced by
  `FFMPEG=<ffmpeg 7.1> python3 check_noise_reduction.py`. The script exits non-zero when a gated number stops holding.
- The script reads the build from `FFMPEG` (default `ffmpeg` on the PATH, which is 9.0.2 on the Mac that measured this), and
  writes the build's version line into `measurements.json`. Unset, it measures whatever the PATH holds.

## Consequences

- The schema gains one `audio_effects` member and a `validate` pass for it (`E-NOISE-RANGE`, `R-NOISE-AFTER-COMPRESSOR`, the schema
  errors). The reading tools describe the member and say that the dial is a reference-level value (§3).
- `CONTEXT.md`'s **Audio effect** entry is unchanged: noise reduction is a restoration member, and its name is listed when that
  entry's wave list is next revised.
- The dial is documented as a reference-level value. No `validate` finding reports the hiss's level, and none can.

## Not decided here (the map's fog)

- **De-ess** (#852 left it untouched), which needs a brighter recording (#848's own note).
- **A hiss-calibrated mapping**, which needs a recording with a long hiss-only stretch (§4). Until then the mapping is the reference's.
- **A level-independent noise reducer** (a spectral or gated one). That is a different capability and needs its own precedent.
- **An expander or gate**, and a **frequency-dependent** reduction. Both are the noise gate's ticket, not this one's.

## Evidence

[`#852`](https://github.com/MBehtemam/Montagent/issues/852) is the court's record: a three-juror court (Opus, Sonnet, Fable) on each
question, the owner ruling with the Judge. This ADR adds what the calibration found after that court: the strength's level dependence
(§3) and the fixture's failure to calibrate the mapping (§4). Neither was in the court's packet, and the jurors could not have known
them. The filter's own properties are in
[`FFMPEG-FILTERS.md`](../research/audio-effects/FFMPEG-FILTERS.md) (#797): `anlmdn` is built into the floor, deterministic on one build,
and has the 384-sample latency measured here.
