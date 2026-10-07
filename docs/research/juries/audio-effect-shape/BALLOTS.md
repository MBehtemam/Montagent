# Ballots: where an audio effect sits on an element

Two courts on [Where does an audio effect sit on an element: an ordered list, flat fields, or both?](https://github.com/MBehtemam/Montagent/issues/799).
Each had three jurors (Opus, Sonnet, Fable), dispatched in parallel. Each juror got the same
packet blind, with none of the Judge's recommendations in it. The packets are
[`QUESTION-ROUND-1.md`](QUESTION-ROUND-1.md) and [`QUESTION-ROUND-2.md`](QUESTION-ROUND-2.md).
Each juror's reasoning below is condensed from their full ballot, which was reported verbatim to the owner in session. Votes are exact. Both times the owner ruled with the Judge's read. The decision is
[ADR-0169](../../../adr/0169-audio-effects-are-an-ordered-list-before-volume-and-gain-and-routing-stay-flat.md).

## Round 1 (Q1 to Q3)

| | Q1 container | Q2.1 chain vs `volume` | Q2.2 chain vs `aloop` | Q3 `video` |
|---|---|---|---|---|
| Juror 1 (Opus) | (c) list + flat | (b) post-fader | (b) after | (a) same field |
| Juror 2 (Sonnet) | (c) list + flat | (a) pre-fader | (b) after | (a) same field |
| Juror 3 (Fable) | (c) list + flat | (b) post-fader | (b) after | (a) same field |

### Juror 1 (Opus)

**Q1: (c).** This copies Premiere's layout directly: a clip has a few fixed controls (Volume, Channel Volume, Panner) plus an ordered effect rack. It also matches the visual side, where `effects` comes first and `opacity` and the blend mode are flat fields after it. A flat `"pan": -0.3` or `"channels": "mono"` is one unique string to replace. For EQ and dynamics, order really changes the sound, and two EQs must be allowed. Routing has no useful order for an author.
*Trade-offs:* (a) brings order and duplicate problems to controls that need neither. (b) fixes one order, which goes against ffmpeg and Premiere, and rules out two EQ bands. (c) costs two places to look, and the line between flat and listed has to be written down and enforced by `validate`.

**Q2: 2.1 (b) post-fader; 2.2 (b) after `aloop`.** The ducking tool writes `volume` keyframes. With a compressor after `volume`, the compressor would push level back and undo the duck, and a gate's threshold would start depending on the fade. A looped bed should sound continuous. Running the chain before `aloop` would cut tails and reset state at every seam.
*Trade-offs:* With `volume` > 1, the signal can exceed a per-element limiter's ceiling, so either cap it or document it. Whether a tail may run past the element's end has to be decided. Pitch shift acts on the sped-up audio, which needs writing down.

**Q3: (a).** A `video` already carries `volume`. Option (b)'s twin `audio` element means keeping trim, `speed`, `overrun` and placement in sync by hand, and exact-string editing is bad at catching that drift.
*Trade-offs:* `video` elements get bigger. The audio field needs a distinct name, and `validate` must reject members placed in the wrong list.

### Juror 2 (Sonnet)

**Q1: (c), as an ordered list `audioEffects`.** Order matters for signal shaping, which rules out (b). Pan, balance and channel operations are routing controls with one sensible position, as in Premiere.
*Trade-offs:* (a) lets routing be misordered. (b) fixes a wrong order and can't express two EQs. (c) costs two places to look and needs one clear rule for the boundary.

**Q2: 2.1 (a) pre-fader; 2.2 (b) after `aloop`.** Pre-fader means the limiter sits last and holds its ceiling. Post-fader would compress a bed at full level and only then turn it down. Processing after `aloop` avoids seam resets and cut tails, and the chain behaves the same with or without looping.
*Trade-offs:* With pre-fader, fades and ducks change what the compressor and gate react to; for example, a long duck can close a gate. The chain after `aloop` costs more compute.

**Q3: (a).** One vocabulary and one validation path, and a chain can be copied between `audio` and `video` unchanged. A twin `audio` element creates sync drift.
*Trade-offs:* An extra field on `video`, and the field name must keep `effects` visual only.

### Juror 3 (Fable)

**Q1: (c).** Volume, pan and channel operations are set once, often by a write tool, and have a fixed correct position. EQ, dynamics, restoration and creative effects need order and allow duplicates. This is both the Premiere shape and ADR-0040's visual shape.
*Trade-offs:* (a) turns a flat control into a list member that has to be located, and allows meaningless duplicates. (b) can't express order or repeats. (c) needs one documented rule and a stated position for the routing controls.

**Q2: 2.1 (b) post-fader; 2.2 (b) after `aloop`.** If `volume` came before a compressor or gate, the compressor would undo the duck and the gate would close mid-fade. Post-fader is the mixer convention and matches Premiere. EQ frequencies and timings are meant as the listener hears them, so they belong after `atempo`. Tails should carry across loop seams.
*Trade-offs:* A per-element limiter doesn't hold under `volume` > 1. A ceiling is really the master stage's job. The element's duration must still clip a tail.

**Q3: (a).** One element, one source, one place to edit. Two elements kept in lock-step would be implicit coupling.
*Trade-offs:* A second list on `video`. Option (b) also makes the `volume`-on-video precedent inconsistent.

## Round 2 (Q4 to Q9)

| | Q4 name | Q5 shape | Q6 bypass | Q7 duplicates | Q8 singling out | Q9 routing |
|---|---|---|---|---|---|---|
| Juror 1 (Opus) | (a) `audio_effects` | (a) tagged | (b) `enabled` | (c) | (a) text | split: channel ops before list, pan after `volume` |
| Juror 2 (Sonnet) | (a) `audio_effects` | (a) tagged | (b) `enabled` | (c) | (a) text | (a) after `volume` |
| Juror 3 (Fable) | (a) `audio_effects` | (a) tagged | (b) `enabled` | (c) | (a) text | (a) after `volume` |

### Juror 1 (Opus)

**Q4: (a).** It needs no lookup, is the clear counterpart of `effects`, and is greppable. `filters` clashes with **Colour filter** and with ffmpeg's word for every node. `sound` or `audio` reads as an element type. `fx` is an abbreviation.
**Q5: (a).** One mental model and one edit pattern across both lists. The cost is that `name` is reserved.
**Q6: (b).** Auditioning is how audio work gets done, and the evidence bar is an A/B. Deleting a member to bypass it loses its tuned parameters. `fmt` should drop `true` so there is one spelling. A disabled member is dead configuration that `review` should flag. Visual `effects` should arguably gain the same field.
**Q7: (c).** Stacking is real, and a second loudness normalisation is a silent mistake `validate` should catch. The rule for each member lives in its ADR, and the error must name the member.
**Q8: (a).** This is how visual effects are edited, and it needs no extra state. Byte-identical duplicates need a wider match. Ids are bookkeeping, and indices break when members are reordered.
**Q9: reject the framing and split it.** Channel operations decide what signal the effects see, so they belong at the source, as Premiere maps channels there. Pan is final placement, after Volume. The graph would be `aloop → [channel ops] → [list] → volume → [pan] → adelay`. The cost is two routing points.

### Juror 2 (Sonnet)

**Q4: (a).** Unambiguous, and it can't be confused with `effects` on a `video`. The other names collide or are vague.
**Q5: (a).** One pattern, and a parameter edit is a short replace. Options (b) and (c) add nesting.
**Q6: (b).** An A/B is far easier with a toggle, and it matches Premiere. `fmt` can omit `true`. The duplicate check and `review` must handle disabled members.
**Q7: (c).** Repeated EQs and compressors are normal. A duplicate loudness member gets an early, checkable error. Each ADR must state whether its member is singular.
**Q8: (a).** Matches visual effects and the standing rule. Identical duplicates need neighbouring context in the match.
**Q9: (a).** Follows Premiere's Panner-after-Volume. The cost is that effects can't see a post-routing signal, such as a mono downmix before a compressor.

### Juror 3 (Fable)

**Q4: (a).** It is the name an agent would guess before reading the schema. `sound`/`audio` would make an author expect `volume` inside it. The cost is a slight asymmetry with `effects`.
**Q5: (a).** One habit, one `fmt` order, one exact-string pattern. Option (b) makes the closed vocabulary a set of key names. Option (c)'s `params` wrapper buys nothing.
**Q6: (b).** Producing the A/B means flipping one literal and re-rendering, not deleting a member and rebuilding it. Deleting a multi-line member is also a more fragile replace. The cost is one field and a `review` rule.
**Q7: (c).** Two high-passes give a steeper slope. Two loudness normalisations only mean "the second wins", which `validate` should say at write time.
**Q8: (a).** Members are usually distinguishable by kind and parameters. An id is a second way to refer to the same thing, and indices go stale.
**Q9: (a).** Shape, set level, then place. Stereo effects see the original layout. The cost is that "one channel only, then EQ it" needs the channel operation done upstream.
