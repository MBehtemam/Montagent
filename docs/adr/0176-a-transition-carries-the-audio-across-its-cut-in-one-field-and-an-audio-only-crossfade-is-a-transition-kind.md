---
status: proposed (becomes accepted once the three-leg table for the check in §7 is committed; until then the 0.1 dB tolerance is provisional. The owner's blind listen is committed and recorded in §7)
amends: 0059 (a transition may now carry sound as well as picture, and `audio_crossfade` is a kind that paints nothing; its "a transition inherently reads two elements' pixels" rationale is widened, not retired), 0150 (a fifth kind, and the `from`/`to` reference rule is kind-dependent for it), 0169 (the per-element graph gains one stage, after `pan` and before `adelay`)
---

# A transition carries the audio across its cut in one field, and an audio-only crossfade is a transition kind

[Does a transition carry the audio across its cut, or do audio crossfades get their own mechanism?](https://github.com/MBehtemam/Montagent/issues/803)
on the audio map ([#795](https://github.com/MBehtemam/Montagent/issues/795)). Today the mix ignores
transitions: a picture dissolve between two clips has a hard audio cut under it, and an audio-only
cut between two music beds has no crossfade at all. This ADR gives a transition one optional
field, `audio`, that carries the sound across the same window, and adds one kind, `audio_crossfade`,
for the case with no picture. It conforms to [ADR-0173](0173-every-audio-capability-is-accepted-by-a-measured-repo-test-and-verify-checks-only-what-master-declares.md).

## Decisions

### 1. One field on the transition carries the sound: `audio`

```json
{ "type": "transition", "id": "t1", "kind": "crossfade", "from": "a", "to": "b",
  "start": 12000, "end": 12800, "audio": "constant_power" }
```

- `audio` is optional on `crossfade`, `wipe`, `slide` and `push`. Its closed vocabulary is
  `"cut" | "constant_power" | "constant_gain"`.
- The sound uses the transition's own window. `from` fades out and `to` fades in over
  `start`..`end`. There is no second window, so there is no second string to keep in step, and a
  trim of either bridged element moves the one window that already exists.
- **One key covers on, off and curve.** `"cut"` is a value, so switching the crossfade on or off
  is a one-string replace and `query` can always print a value.
- **The two curves answer a real question, not taste.** Under `amix normalize=0` the two sides are
  summed, so:
  - `constant_power` holds the level for two different signals (two different clips, the normal
    case) and bumps +3 dB at the midpoint for the *same* signal on both sides;
  - `constant_gain` holds the level for the same signal on both sides (a cutaway back into one
    take, a split clip) and dips 3 dB at the midpoint for two different signals.

  Guidance says: write `constant_power`, unless both sides are the same source. §7's script
  measures all four numbers.
- **`ease` is picture-only.** The audio's shape is `audio`'s alone; a visual `ease` does not bend
  it. This falls out of one field carrying the curve, and the court was not asked it.

Rejected:
- **A separate audio element that names the two bridged elements and carries its own window.**
  The author writes the same two integers twice, and `validate` would need a code whose only job
  is catching the drift the design created. Court: 3/3.
- **`audio_curve` plus a separate `none`.** A second concept for on/off. Court: Juror 2 only.
- **A third curve, `exponential`** (Premiere has it). It is a stylistic fade with no rule an agent
  can pick by, and a fourth value later breaks no existing document. The map keeps the door open.
  Court: Jurors 1 and 3.

### 2. `audio_crossfade` is a fifth transition kind, for sound alone

```json
{ "type": "transition", "id": "t2", "kind": "audio_crossfade", "from": "bed-a", "to": "bed-b",
  "start": 28000, "end": 30000, "audio": "constant_power" }
```

- It reuses everything a transition already has: `from`, `to`, `start`, `end`, the window equal
  to the two elements' overlap, the two separate tracks, and `E-TRANSITION-RANGE`,
  `E-TRANSITION-NO-OVERLAP` and `E-TRANSITION-REF-SELF`. It paints nothing.
- **Its references may name an `audio` element or a `video`** (whose embedded audio is crossfaded;
  the picture is untouched). The visual kinds keep refusing an `audio` element exactly as today.
  `E-TRANSITION-REF-MISSING` keeps its repair form (a value) and its text names what the kind
  accepts.
- **The schema holds the rest, per kind.** For `audio_crossfade`, `audio` is required and is
  `constant_power` or `constant_gain` (`cut` is refused: a kind with nothing to do is a
  mistake); `direction` and `ease` are unknown keys. For the visual kinds `audio` stays optional.
  The per-kind presence rule is a small irregularity an agent meets once and reads from the
  schema error.
- **A J-cut or L-cut, whose window is not the overlap, stays hand-written** as `volume`
  keyframes. That is a different thing from a crossfade and this ADR adds nothing for it.

Rejected: a new element type with its own `from`/`to`/window (it would duplicate every rule and
code above: 3/3), and nothing new, leaving audio-only cuts as paired volume keyframes (they
cannot be read back as one crossfade, and the agent writes four or more literals that must stay
in step).

### 3. The gain applies at the end of the element's own chain

The per-element graph of [ADR-0169](0169-audio-effects-are-an-ordered-list-before-volume-and-gain-and-routing-stay-flat.md)
becomes:

`atrim → atempo → aloop → [audio_effects] → volume → [pan/balance] → transition gain → adelay → amix`

- **Reason:** ADR-0169 puts the effects list before `volume` so a compressor cannot push a
  ducking dip back up. A crossfade is a gain curve exactly like a dip, so it takes the same
  position, after everything the element's own sound design can do. A reverb tail inside
  `audio_effects` is faded with the dry signal, which is the conventional result. Court: 3/3.
- **Stacking is a plain product.** An element's `volume` keyframes (the author's own fade or
  duck) multiply with the transition gain. The renderer adds no rule beyond that.
- **Render:** `afade` out on the `from` side and `afade` in on the `to` side, with `curve=qsin`
  for `constant_power` and `curve=tri` for `constant_gain`, over the window in the element's
  own time (`start − element.start`). The two chains meet in the existing `amix normalize=0`.
  `acrossfade` is never used: it accepts no runtime commands (§4.2) and takes whole streams, so
  its output is `len1 + len2 − d` long (§5)
  ([`FFMPEG-FILTERS.md`](../research/audio-effects/FFMPEG-FILTERS.md)). `afade` is
  sample-exact, so a crossfade is exact even where a keyframed `volume` once slipped a frame
  (ADR-0175).
- **An absent or `cut` value adds no stage.** The graph is today's graph.

### 4. Absence is today's hard cut, and one review tells an agent the key exists

- **No `audio`, or `"audio": "cut"`, renders exactly as today.** Every existing document is
  byte-identical. Making the crossfade the default would change the sound of every project that
  uses a transition; a required field would break every file. Court: 3/3.
- **`R-TRANSITION-AUDIO-UNSET`** (review). A visual-kind transition with no `audio`, where *both*
  bridged elements carry an audio stream and neither is a constant `volume: 0`, gets one finding.
  Its text gives both literals: `"audio": "constant_power"` to crossfade the sound, or
  `"audio": "cut"` to keep the hard cut on purpose. Either value silences it for good. Without
  this finding a new agent writing a dissolve would never learn the key exists.
- **Not an amendment of ADR-0030.** [ADR-0030](0030-defaultable-field-presence-is-content.md)
  says `validate` never flags either spelling of the seven fields it names, and `fmt` leaves a
  defaultable field's presence alone. `fmt` still does so here. This ADR names `audio` as a field
  whose absence is itself read, because the default is the one that sounds wrong and the finding
  is the only way the key is discovered. It reads only the transitions that bridge two sounding
  elements.
- **A new transition** is written with `audio` every time. The schema example and the agent
  guidance show `constant_power`.

### 5. What `validate` says

| Code | Class | Fires when | Repair form (ADR-0043) |
| --- | --- | --- | --- |
| `E-TRANSITION-REF-MISSING` (existing) | error | on `audio_crossfade`, `from`/`to` names no `audio` or `video` element | a value, as today |
| `E-TRANSITION-AUDIO-NO-STREAM` (new) | error | an `audio_crossfade` side whose source has no audio stream | `"none"`, with a census naming the side, target and source. The fix forks (point it elsewhere, or drop the transition), so it is refuse-class (ADR-0120) |
| `R-TRANSITION-AUDIO-UNSET` (new) | review | §4 | none (reviews carry no `repair`) |
| `R-TRANSITION-VOLUME-STACK` (new) | review | a bridged element's `volume` changes level inside the window: a keyframe with `t` strictly inside `(start, end)`, or a segment that straddles it with differing `v`. A flat volume across the window is not stacking | none |
| `R-TRANSITION-AUDIO-SILENT` (new) | review | a bridged side is a constant `volume: 0` or a speed-ramped `video` (`E-REMAP-AUDIBLE`'s required `volume: 0`), on any kind | none |

- Stacking is a review, not an error, because the same file can mean an accidental double fade
  or a deliberate duck under narration that spans the cut, and `validate` cannot tell which. The
  text names the transition, the element and the keyframe times.
- A silent side means the crossfade is a one-sided fade. `volume: 0` is an authored literal and
  `E-REMAP-AUDIBLE` already explains why a ramp carries it, so it is a review and never an error.
- On a **visual** kind, a bridged element with no audio stream is fine: the other side still
  fades, and `N-NO-AUDIO-STREAM` already reports it. Only `audio_crossfade` refuses it, because
  that kind exists to carry sound and a side with none makes it a fade in disguise, which
  `volume` keyframes already express. Court: Juror 3's rule; Juror 1 would error only if *neither*
  side had sound.

### 6. What the reading tools say

The smallest set that stops an agent being misled:
- **`query --at`** at an instant inside the window lists each sounding element's resolved gain
  with its factors and the transition that produced it: `volume 1.0 × transition t1
  constant_power 0.63 → 0.63`. With `audio` absent it prints `transition t1 audio: cut
  (absent)`, which is what teaches an agent that absence means cut. It never prints a gain
  without the id the agent has to edit.
- **`timeline`** shows the value as one token on the transition row (`audio=constant_power`).
- **`compare`** diffs `audio` like any field of the transition. **`shift`** needs nothing new:
  `audio` carries no time and the window is the one it already moves. Its tests add the new kind.

Not added, by design: derived volume keyframes in the file, in `query` or in `compare`; a sampled
gain table or curve plot; a separate listing of audio transitions; a second place to edit the
window or the curve.

### 7. The measured check (conforms to ADR-0173)

[`check_audio_crossfade_curves.py`](../research/audio-effects/audio-crossfade/check_audio_crossfade_curves.py)
(stdlib and ffmpeg only; exits non-zero when a number stops holding), with its numbers in
[`measurements.json`](../research/audio-effects/audio-crossfade/measurements.json):

- [ ] **Conforms to ADR-0173.**
- [ ] **Midpoint level.** Metric: RMS over the 40 ms around the window's midpoint, against one
  side alone. Side of the encoder: PCM, before any encoder. Fixture: lavfi sines at 500 Hz and
  750 Hz (whole cycles in the window, so they are orthogonal), then 500 Hz on both sides. Expected
  value, from the `afade` definitions: unrelated signals, `constant_power` 0.00 dB and
  `constant_gain` −3.01 dB; the same signal, `constant_power` +3.01 dB and `constant_gain`
  0.00 dB. Two-sided, ±0.10 dB. The measured values are −0.00, −3.01, +3.01 and 0.00 on ffmpeg
  6.1.1 and on 7.1.5, both Linux x86_64.
- [ ] **Window edges.** One side at a time on a constant signal: the outgoing side is untouched
  the sample before the window, full on its first sample, and exactly 0 at the window's end; the
  incoming side is 0 up to the window's start and full at its end. Exact (0 samples).
- [ ] **Bypass identity.** A transition with `"audio": "cut"` produces PCM byte-identical to the
  same document with the key absent, and a document with no new feature produces the committed
  graph string, unchanged. These two are for the build's test; the script cannot run them.
- [ ] **The tolerance's origin.** ±0.10 dB is the `astats` print precision, set before any
  three-leg run. ADR-0173 §4 says it becomes max(2 × the spread across the three CI legs, the
  meter's resolution), with the per-leg table committed. That table does not exist yet; this is
  why the status is `proposed`.
- [ ] **The meter.** The script reads PCM in Python. The repo test ports the same metric to
  ffmpeg's `astats` and pins the parse (ADR-0173 §2).
- [x] **A/B.** The owner's blind listen is committed in
  [`VERDICT.md`](../research/audio-effects/audio-crossfade/VERDICT.md), with the clips and `KEY`, as ADR-0173 §8
  asks (2026-10-08, ffmpeg 6.1.1). Music into music: the crossfade was heard as smooth and today's mix as a
  sudden rise. The same speech on both sides, `constant_gain` against `constant_power`: "almost the same".
  The 3 dB bump is real (§7's script) and small to this ear in one 2 s window. The guidance on which curve
  to reach for stands, and is a second-order choice next to switching the crossfade on.

## Consequences

- The schema gains `audio` on `transition` and the kind `audio_crossfade`, with the per-kind rules
  in §2. `CONTEXT.md` gains **Audio crossfade** and the **Transition** entry names the `audio`
  field.
- The mix graph gains one `afade` stage per bridged side, only where `audio` is set.
- `validate` gains one error and three reviews. `E-TRANSITION-REF-MISSING`'s text becomes
  kind-dependent. Existing projects that use transitions between sounding elements gain one
  `R-TRANSITION-AUDIO-UNSET` each: noise once, one string to clear.
- ADR-0059, ADR-0150 and ADR-0169 carry an *Amended by* banner and the README's column says so.
- **Not decided here** (the map's fog): `exponential` as a third value; per-side different
  curves; an offset (J/L) crossfade; whether `audio` may be animated (a window-wide choice, so
  probably not); a write tool that emits a transition and its audio together; a visual-only
  dissolve that deliberately keeps the cut, which is `"audio": "cut"` and is already expressible.

## Evidence

Two three-juror courts (Opus, Sonnet, Fable), run blind with no recommendation in the packet. The
owner ruled with the Judge on both. The first (six frontier tickets, of which this was Q-F) is in
[`docs/research/juries/audio-first-wave/`](../research/juries/audio-first-wave/BALLOTS.md); the
second, which asked this ticket's seven questions from the point of view of an agent using the
format, is in [`docs/research/juries/audio-transitions/`](../research/juries/audio-transitions/BALLOTS.md).

- First court: three jurors, three shapes. The Judge leaned to a field on the transition with
  default `cut` plus an audio-only kind.
- Second court:
  - the transition owns the window, the gain sits at the end of the element's chain, absent means
    the hard cut with a review for the unset case, an audio-only kind on `transition`, stacking
    as a review, and the smallest reading set: all 3/3;
  - two curves under one `audio` key: 2/3 (Juror 2 voted for three under `audio_curve`);
  - a silent bridged side: three different rules, and the Judge took Juror 3's;
  - `ease` being picture-only, and ADR-0030's reading: not put to the court.

Precedent: Premiere's audio crossfades (Constant Power, the default, Constant Gain, and Exponential
Fade) are objects separate from its video transitions, so a mechanism exists by ADR-0145's rule.
Our departure is to hang the sound on the transition that already owns the window, because the
format forbids writing a derived value twice. CapCut documents no audio crossfade mechanism
([`PRECEDENT.md`](../research/audio-effects/PRECEDENT.md)).
