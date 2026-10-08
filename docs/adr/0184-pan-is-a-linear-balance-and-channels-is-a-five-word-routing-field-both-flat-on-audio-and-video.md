---
status: proposed (DRAFT; becomes accepted once the render-test table in section 7 is committed with its three-leg spread, the owner's listening test has confirmed the direction named in section 3, and the owner has confirmed the calls in "Owner calls to confirm"; until then the tolerances in section 7 are provisional)
---

# Pan is a linear balance and channels is a five-word routing field, both flat on audio and video

[Capability ADR and hand-off spec: pan/balance and channel operations](https://github.com/MBehtemam/Montagent/issues/795)
on the audio map (#795), modelled on
[ADR-0183](0183-the-noise-gate-is-one-stackable-audio-effect-with-a-range-no-hold-and-an-rms-threshold.md)
and [ADR-0180](0180-the-compressor-and-the-limiter-are-two-audio-effects-with-no-defaults-and-an-rms-threshold.md).
The map named these two under "level and space" and left both for the later wave; ADR-0169 reserved their
shape ("flat fields") and ADR-0170 fixed the pan range. This ADR fixes the law, the vocabulary, the slots,
the finding list and the measured check. The owner has accepted the research's recommended defaults as the
draft position and is to confirm them in review (the last section). It conforms to
[ADR-0169](0169-audio-effects-are-an-ordered-list-before-volume-and-gain-and-routing-stay-flat.md) ("Where
routing sits"),
[ADR-0170](0170-audio-levels-are-written-in-db-and-their-keys-say-so-volume-stays-the-one-linear-level.md)
("Pan is -1..+1"),
[ADR-0172](0172-the-master-stage-is-a-top-level-loudness-target-and-true-peak-ceiling-reached-by-one-measured-gain.md),
[ADR-0176](0176-a-transition-carries-the-audio-across-its-cut-in-one-field-and-an-audio-only-crossfade-is-a-transition-kind.md)
(the transition gain follows pan) and
[ADR-0173](0173-every-audio-capability-is-accepted-by-a-measured-repo-test-and-verify-checks-only-what-master-declares.md).
Evidence: [`docs/research/audio-effects/pan-channels/`](../research/audio-effects/pan-channels/README.md)
(`measure_pan_channels.py`, `measure_slot_loudness.py`, `measurements.json`; ffmpeg `n7.1.5-12-g1fdbca85aa`
and `6.1.1-3ubuntu5`, both passing every claim). The 48 kHz stereo bus is ADR-0077's.

## Decisions

### 1. Two flat fields on `audio` and `video`, not `audio_effects` members

```json
{ "type": "audio", "src": "narration.wav", "channels": "left", "pan": -0.3, "volume": 1 }
```

| Field | Value | Default (absent) |
| --- | --- | --- |
| `pan` | a finite number, `-1..1`; `0` is centre, negative is left | `0`: no filter |
| `channels` | one of `"stereo"`, `"swap"`, `"mono"`, `"left"`, `"right"` | `"stereo"`: no filter |

- **Flat, by ADR-0169's test:** a routing control has one correct position, and a list would invite
  meaningless orders and duplicates. A `video` carries both for its embedded track (ADR-0055, ADR-0169).
- **Absent means today's render.** A document with neither field renders the graph it rendered before
  (committed string, ADR-0173 section 5). So does `pan: 0`, and so does `channels: "stereo"`: the renderer emits
  no filter for either. Flat fields have no `enabled`; omitting the field is the bypass, and `fmt` does not
  drop an explicit default (it is what the review in section 4 reads).
- **`pan` is a literal static number.** A keyframe list on `pan` or on `channels` is a schema error (section 5,
  #842). `channels` is a closed word, so a list on it is a type error anyway.
- **Not taken, by name:** an `Invert` (polarity) word, a per-channel Channel Volume, a mixing pan, a
  constant-power pan, track-level pan. See section 8.

### 2. `pan` is a balance with a linear taper

`pan` is a **balance on the stereo bus**: the near channel stays at unity and the far channel is scaled by
`1 - |pan|`. It never moves content between channels.

| `pan` | left gain | right gain | far channel |
| --- | --- | --- | --- |
| -1 | 1 | 0 | silent (exact zero) |
| -0.5 | 1 | 0.5 | -6.02 dB |
| 0 | 1 | 1 | no filter |
| 0.5 | 0.5 | 1 | -6.02 dB |
| 1 | 0 | 1 | silent (exact zero) |

- **Why a balance.** On this bus every source is already L/R. Premiere's Panner on a stereo clip is described
  as a balance ("how much of each input channel is sent to the output channels", Adobe text); Adobe does not
  say whether it moves content, so the repo's measurement decides: with L = 440 Hz and R = 880 Hz, `pan: 1`
  leaves R bit-identical and L exactly zero, with no 440 Hz in R (below -100 dB). A mixing pan matrix does put
  440 Hz in R.
- **Why a 0 dB centre.** Only a law that is unity at centre keeps `pan: 0` identical to no pan. The -3 dB
  constant-power law and the -6 dB linear law change a clip's level the moment a key is written, even at 0,
  and shift `normalize_loudness` outcomes by 3 to 6 dB. The cost of a unity cap is that the pair dips as it
  moves off centre: the total power on a centred source is **-2.04 dB at |pan| 0.5 and -3.01 dB at 1**
  (measured `total_dB`). That dip is named, not corrected.
- **Why linear and not quarter-cosine.** Linear is explainable ("half the pan, half the far channel",
  -6.02 dB at 0.5) and the simplest for exact-string editing. The quarter-cosine is gentler (-3.01 dB far
  channel and a -1.25 dB total at 0.5) but not literal. The pan law is not documented on any Adobe page read
  (2026-10-08), so this is a choice, not a copy (section 8).
- **The bus fact.** `aformat=channel_layouts=stereo` already puts a **mono** source on L and R at -3.0103 dB
  each (swresample's centre). That is existing behaviour and this capability does not change it.
- **Rendered literally**, after the stereo bus guarantee:

```
pan=stereo|c0=<gl>*c0|c1=<gr>*c1        (a gain of 1 is written without its factor: c0=c0)
```

  with the gains as shortest decimals of `1 - |pan|`, to at most 6 decimal places, trailing zeros trimmed
  (so `pan: -0.3` renders `pan=stereo|c0=c0|c1=0.7*c1`).
- **Only `=` is used in a `pan` expression, never `<`.** Measured: `=` is literal, `<` *normalises* the gains
  (divides by their sum when it exceeds 1). `c0=c0+c1` on L = R reads +6.02 dB, `c0<c0+c1` reads 0 dB. A
  `<` would silently re-level every matrix here, so the renderer never writes it and a test asserts the
  rendered string contains none.
- **Latency 0, length exact, no drift.** An impulse at sample 100 lands on sample 100 through balance, swap and
  mono; the length is the input's (48000 of 48000). Nothing for `apad...atrim` to cancel (#843). The SHA-256 of
  the f64 PCM of six renders is identical between 6.1.1 and 7.1.5 and with `-cpuflags 0`; repeat runs are
  identical. `pan` is already in `FFMPEG-FILTERS.md` as zero-latency and command-dropping.

### 3. `channels` is five words, with `left` and `right` as fills

Each word is one exact matrix on the stereo bus (`c0` is left, `c1` is right):

| Word | Matrix | Meaning |
| --- | --- | --- |
| `stereo` | none | no filter (the default) |
| `swap` | `c0=c1`, `c1=c0` | left and right exchange places |
| `mono` | `c0=0.5*c0+0.5*c1`, `c1=0.5*c0+0.5*c1` | both outputs carry the average of the two |
| `left` | `c0=c0`, `c1=c0` | **keep the left channel; it is copied onto the right and the original right is discarded** |
| `right` | `c0=c1`, `c1=c1` | **keep the right channel; it is copied onto the left and the original left is discarded** |

- **The direction, exactly as this repo measured it** (`measure_pan_channels.py`, check 4, bit-exact in float
  on both builds): with `channels: "left"` both output channels are bit-identical to the **source left**; with
  `"right"` both are bit-identical to the **source right**. A left-only recording (a lav mic with the right
  empty) put through `left` returns a centred signal at the live channel's level (0.00 dB re the live channel,
  tolerance 0.01). The word names the channel that is kept.
- **This is not Premiere's name.** Premiere's effects are *Fill Left with Right* and *Fill Right with Left*.
  Adobe's own text for the first (read 2026-10-08) says it "duplicates the left channel information of the
  audio clip and places it in the right channel, discarding the original clip's right channel information":
  that is what this ADR's `left` does, and it is the opposite of what the effect's name reads as. Adobe's text
  settles what Adobe says, not what the effect does. **The owner's listening test has still to confirm which
  channel each word keeps (and so whether `left` is the right name) before this ADR is accepted.** Until then
  the measured behaviour above is the contract and the docs do not claim a mapping to Premiere's two names.
- **The fill forms only.** "Keep one channel and silence the other" is `pan: -1` or `pan: 1`
  (exact zero, section 2), so the vocabulary needs only the fills, which are also what a lav-mic fix needs.
  `channels: "left"` with `pan: -1` gives left audio on the left only; that composition is well defined and
  needs no rule.
- **`mono` is an explicit `0.5 (L + R)`, never `aformat`.** ffmpeg's own stereo-to-mono
  (`aformat=channel_layouts=mono`) is `0.7071 (L + R)`: +3.01 dB on a centred source, which can clip. Measured:

| Fold | correlated L = R | uncorrelated noise | left-only | anti-phase L = -R |
| --- | --- | --- | --- | --- |
| `0.5 (L + R)` | 0.00 dB | -2.98 dB | -6.02 dB | exact silence |
| `0.7071 (L + R)` | +3.01 | +0.03 | -3.01 | silent |

  `0.5 (L + R)` is the only fold that cannot raise a clip: unity on a dual-mono file (the TTS case), about
  -3 dB on uncorrelated material. Its cost is -6.02 dB on a one-channel recording, which is what `left` and
  `right` are for. A mono fold of a polarity-inverted pair is silence (a candidate review, not taken here).
- All five are the same `pan=` expression with `=` only; swap, `left` and `right` are bit-exact in float, so the
  tolerance for them is **exact**.

### 4. Where they sit in the graph

`atrim → atempo → aloop → [channels] → [audio_effects] → volume → [pan] → [transition gain, ADR-0176] → adelay → amix`

- **`channels` runs after `aloop` and before the `audio_effects` list.** ADR-0169 named this the leading
  candidate. Measured (`measure_slot_loudness.py`, ebur128, 5 s at 1 kHz): on a left-only source, `left` is
  **+3.00 LU** and `mono` is **-3.00 LU**; on a centred source both are 0. A routing op can move loudness by
  several LU, and `normalize_loudness`, a compressor threshold and a limiter all read level, so they must see
  the signal after the op or the author's `target_lufs` is met on a signal that is then re-levelled.
- **`pan` runs after `volume`**, as Premiere's Panner follows its Volume (ADR-0169). It is a placement control, so
  a duck and a normalise are never re-levelled by it. The consequence is stated, not hidden: `pan` is not seen
  by `normalize_loudness`, so a hard pan lowers the element's loudness after the member has met its target
  (-2.04 LU at 0.5, -3.01 LU at 1 on a centred source). The master stage measures the whole programme
  (ADR-0172) and is unaffected.
- **Ahead of the transition gain** (ADR-0176 already places it after `pan`). `channels` and `pan` do not
  change a document's length and add no latency, so no placement rule moves.
- **Only on the stereo bus.** The `pan=stereo|...` matrices are always 2-in, 2-out because a source is on the
  stereo bus before they run. The build slice confirms where the engine's existing `aformat` sits; if it
  comes after `aloop`, `channels` takes its own `aformat=channel_layouts=stereo` ahead of the matrix (a no-op
  for stereo input, and added only when `channels` is not `stereo`).

### 5. Not animatable

`pan` ignores `asendcmd` (measured: the output equals a render with no command). The one working path for a
keyed pan is `channelsplit` + per-channel `volume` + `asendcmd` + `join`, measured to silence L at 0.5 s while R
stays up. That is a five-filter graph per element, which fails the graph-level cost cap in the animatable-
parameters ruling (#842), and a filter that drops commands is never admitted. So **`pan` is a static literal
and a keyframe list on `pan` is a schema error.** This is a named departure from Premiere, whose Balance and Pan
are keyframable (Adobe text: Panner > Balance or Pan, then the Add/Remove Keyframe icon). Autopan and a move
across the picture are deferred under #842; an author who needs a moving pan today writes a second element.

### 6. Findings

| Code | Class | Fires when | Repair |
| --- | --- | --- | --- |
| *(schema)* | error | `channels` is not one of the five strings; `pan` is not a finite number; a keyframe list on `pan` or `channels` | the existing schema errors |
| `E-PAN-RANGE` | error | `pan` is finite but outside `-1..1` (so Premiere's `50` fails, ADR-0170) | a value: the nearest bound |
| `R-PAN-INERT` | review | `pan` is written as `0` | delete the field |
| `R-CHANNELS-INERT` | review | `channels` is written as `"stereo"` | delete the field |
| `R-CHANNELS-ON-MONO-SOURCE` | review | `channels` is `swap`, `mono`, `left` or `right` on a source the probe reports as mono (the bus already doubled it, so the op is inert) | none |

- **The explicit-default spellings are reviews** of the family of `R-EASE-INERT`: the field is spelled like the
  default and renders nothing, so a finished document carrying it is a leftover. The alternative (accept as
  harmless) is an owner call.
- **`R-CHANNELS-ON-MONO-SOURCE` needs the probe**, as other source-dependent findings do. It is a review
  because the bus really does double a mono source and nothing is lost.
- `pan` with `channels: "mono"` is allowed (fold, then balance); no finding.
- **Not taken:** `channels: "left"` or `"right"` where the kept channel is silent (needs a per-channel level
  probe, probably out of scope), and a review for `mono` on a polarity-inverted pair (silence; needs analysis).
  Neither belongs to `validate`.

### 7. The measured check (conforms to ADR-0173)

- [ ] **Conforms to ADR-0173.**
- [ ] **Method.** A repo integration test, the port of `measure_pan_channels.py`, through the real mix graph with
  the encoder swapped for PCM. Fixture: lavfi tones and `anoisesrc` with a pinned `seed`, all built from literal
  parameters. Metric: per-channel RMS in dB (over the last 3/4 of 1 s, f64) and sample equality. Expected
  values come from the matrix definitions in sections 2 and 3, never a stored render. Tolerances: **0.01 dB for
  gains, exact for swap, fill and zero**, provisional until the three-leg table is committed; ADR-0173
  section 4 then replaces them with max(2 x the three-leg spread, the print precision). The evidence run found
  zero spread between its two builds, so the tolerance lands on the print precision.

| Id | Assertion | Side | Tolerance |
| --- | --- | --- | --- |
| T1 | `pan` in {-1, -0.5, 0, 0.5, 1}: near channel 0 dB, far channel `20 log10(1 - abs(pan))` dB re the source | two-sided | 0.01 dB |
| T2 | `pan` of +1 and -1: the far channel's max abs sample is 0 | equality | exact |
| T3 | balance never mixes: L = 440 Hz, R = 880 Hz, `pan: 1` leaves the right channel bit-identical to the source right | equality | exact |
| T4 | `swap`, `left`, `right`: output channels equal the specified source channels (section 3) | equality | exact |
| T5 | `mono` on L = R: 0.00 dB; on seeded uncorrelated noise: -3.0 dB; on L = -R: exact zero | two-sided | 0.01 dB / 0.15 dB / exact |
| T6 | `mono` on a left-only source: -6.02 dB re the live channel | two-sided | 0.01 dB |
| T7 | `left` on a left-only source returns both channels at the live channel's level | two-sided | 0.01 dB |
| T8 | an impulse at sample 100 lands on 100 and the length is unchanged, for `pan` and each `channels` word (#843: 0 samples) | equality | exact |
| T9 | bypass identity: `pan: 0`, `channels: "stereo"`, and both absent produce the same PCM bytes and the same filtergraph string | equality | exact |
| T10 | a document with neither field renders the committed expected graph string, unchanged | equality | exact |
| T11 | no rendered `pan=` expression contains `<` (the trap in section 2) | equality | exact |
| T12 | slot: a left-only source with `channels: "left"` then `normalize_loudness` lands within 0.1 LU of its target (shows the op is seen first). Needs `normalize_loudness` built (ADR-0178) | two-sided | 0.1 LU |

  T1-T8 are the research's P1-P3, C1-C3 and L0; T9-T10 its B1-B2; T12 its S1 (proposed there, not yet run).
- [ ] **Command ignored.** `asendcmd` to `pan` moves nothing (L unchanged after 0.5 s), so section 5 can be
  reopened if `pan` ever honours it. Recorded in the evidence run; the test keeps the control.
- [ ] **Same-build determinism** (identical bytes across runs and `-cpuflags 0`).
- [ ] **Build-to-build drift.** Measured on two builds here (SHA-256 of six renders equal). The macOS
  Homebrew and Windows Chocolatey legs were **not available** to the evidence run; the per-leg table must be
  committed with them before the status flips.
- [ ] **A/B: recorded, not yet heard.** Flat fields have no `enabled`, so the comparison is the document with
  the field removed. The owner's short blind A/B on the shared narration fixture for `pan: -0.5` is the one
  claim numbers do not fully settle; it is not made yet. **A second listening test is the direction of `left`
  and `right`** (section 3): the numbers fix which channel is kept, the ear confirms the name. Neither is
  claimed here (one measurement run, no listening).
- [ ] **Evidence scripts.** [`measure_pan_channels.py`](../research/audio-effects/pan-channels/measure_pan_channels.py)
  (needs numpy; exits non-zero naming any claim that stops holding),
  [`measure_slot_loudness.py`](../research/audio-effects/pan-channels/measure_slot_loudness.py) (stdlib) and
  [`measurements.json`](../research/audio-effects/pan-channels/measurements.json) (the floor build's output).

### 8. Departures from the Premiere precedent

[`PRECEDENT.md`](../research/audio-effects/PRECEDENT.md) sections 2 and 3: precedent is confirmed in
**Premiere only**; CapCut has nothing first-party for either row. Premiere: Panner (Balance on a stereo clip,
Pan on a mono one), range -100..+100 (CONFIRMED), keyframable (CONFIRMED); Fill Left with Right, Fill Right with
Left, Swap Channels (stereo only), Invert and Channel Volume (CONFIRMED); the Audio Channels mapping matrix is
RECALLED.

- **-1..+1 instead of -100..+100** (ADR-0170, already ruled).
- **A pan law of the repo's choosing.** Adobe documents none, so the linear balance is a choice, not a copy.
- **A flat `channels` word** instead of separate effects, and a flat `pan` instead of a clip effect (ADR-0169).
- **`left` and `right` named for the channel kept**, not Premiere's names (section 3), pending the owner's ear.
- **No Invert** (polarity): not requested; a later `audio_effects` member if wanted (it is signal shaping, not
  routing, and has no parameters).
- **No Channel Volume**: it is per-channel `_db` (ADR-0170), a later separate member.
- **No keyframes in this slice** (section 5), against Premiere's keyframable Balance and Pan.
- **No track-level pan** (ADR-0004; the map rules track effects out).
- **No Audio Channels mapping matrix**; the five words cover the cases this format has a use for.

## Consequences

- The schema gains two optional flat fields on `audio` and `video`; `validate` gains one error, three reviews
  and the schema errors. The reading tools describe both fields.
- The per-element graph gains two stages, `[channels]` and `[pan]`, each present only when it does something.
  Today's graph is unchanged until a document uses a field (T10).
- ADR-0169's "Where routing sits" is settled: `channels` before the list, `pan` after `volume`.
- `CONTEXT.md`'s **Audio effect** entry notes that routing (`channels`, `pan`) is flat and not a member.
- The pan and channels slices do not depend on the `audio_effects` foundation (F0): both fields are flat. If F0
  is not built, `channels` sits after `aloop` and F0 later inserts the list after it.
- Not decided here (the map's fog): a keyframed pan (#842), a true stereo (mixing) pan, a constant-power pan,
  Invert, Channel Volume, track-level pan, a probe-based review for a silent kept channel.

## Owner calls to confirm

Every item below was assumed (the research's recommendation, accepted as the draft position) and is the owner's
to change in review.

1. **Taper.** Linear far channel `1 - |pan|`, near channel unity (against quarter-cosine, -3.01 dB at 0.5,
   gentler but not literal; a -3 dB constant-power centre is not taken because it re-levels `pan: 0`).
2. **Balance only.** `pan` never moves content between channels; a mixing or stereo-image pan is a later,
   separate member if wanted.
3. **Mono fold** is `0.5 (L + R)` (never clips, -6.02 dB on a one-channel file), not ffmpeg's `0.7071 (L + R)`
   (+3 dB on a dual-mono file, can clip).
4. **Vocabulary** is `stereo | swap | mono | left | right`, with `left` and `right` as fills (keep one channel and
   copy it over the other), not "silence the other side" (that is `pan: -1` or `1`).
5. **The direction of `left` and `right`** is what this repo measured (section 3), and **the owner's listening
   test has still to confirm the name** before acceptance. If the ear disagrees, the words swap meaning
   (or are renamed); the matrices and tests change in one place.
6. **Flat fields** on `audio` and `video`, not `audio_effects` members; no `enabled` on them.
7. **`pan` is static, range `-1..1`**, a keyframe list is a schema error. Keyframed pan is deferred under #842;
   say yes to it only with a tolerance on the cost cap.
8. **Slots.** `channels` after `aloop` and before `audio_effects`; `pan` after `volume` (and so unseen by
   `normalize_loudness`, with the -2.04 / -3.01 LU consequence stated).
9. **Explicit-default spellings** (`pan: 0`, `channels: "stereo"`) and `channels` on a probed mono source each
   get a **review**, rather than being accepted as harmless; `fmt` keeps them.
10. **Out of scope:** Invert and per-channel Channel Volume stay out.
11. **Wave.** Pan and channels are one slice each; they ship together or `channels` first (it is the lav-mic
    enabler and the one with a loudness slot decision). Nothing here forces the order.
12. **Cross-leg equality.** Whether the ADR may rely on the macOS and Windows legs matching the Linux floor
    (not measurable here). It flips to `accepted` only with the three-leg table committed.
13. **Render details this draft chose:** a gain of 1 is written without its factor (`c0=c0`), gains are written
    to 6 decimals, and `channels` takes its own `aformat` if the engine's existing one sits after `aloop`.
14. **Finding ids and names** (`E-PAN-RANGE`, `R-PAN-INERT`, `R-CHANNELS-INERT`, `R-CHANNELS-ON-MONO-SOURCE`)
    are this draft's.

## Evidence

No court has been convened for this draft. The numbers are `measure_pan_channels.py` and
`measure_slot_loudness.py` on ffmpeg `n7.1.5-12-g1fdbca85aa` and `6.1.1-3ubuntu5`. Premiere's pages were
re-checked against Adobe on 2026-10-08 ([#849](https://github.com/MBehtemam/Montagent/issues/849),
PR [#851](https://github.com/MBehtemam/Montagent/pull/851)): the range, keyframability and the Fill, Swap, Invert
and Channel Volume effects are CONFIRMED, the pan law is not documented, and the Audio Channels matrix is
RECALLED. No listening has been done for pan or for the direction of `left` and `right`.
