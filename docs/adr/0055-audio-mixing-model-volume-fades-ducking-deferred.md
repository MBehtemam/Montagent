---
status: accepted
amends: 0012 (names the replacement for the `opacity`-on-audio schema error), 0020 (extends the `speed`/`overrun` peer-field convention), 0040 (clarifies `effects` is scoped to visual treatment)
---

# `volume` is a keyframable 0..1..>1 multiplier, flat on audio and video elements; ducking is deferred, hand-authored as keyframes

> **Amended by [ADR-0172](0172-the-master-stage-is-a-top-level-loudness-target-and-true-peak-ceiling-reached-by-one-measured-gain.md)**: clipping past the summed mix is no longer only "the
> renderer's documented behaviour" when the project sets a ceiling: a master stage can hold the
> mix to a true-peak ceiling by one measured gain.

> **Amended by [ADR-0170](./0170-audio-levels-are-written-in-db-and-their-keys-say-so-volume-stays-the-one-linear-level.md)**: the rejection of dB below is
> scoped to `volume`. Every other audio level is written in dB (LUFS for a loudness target, dBTP
> for a true-peak ceiling), with the unit in its key (`gain_db`); `volume` stays the one linear
> level, with no dB sibling.
>
> **Amended by [ADR-0169](./0169-audio-effects-are-an-ordered-list-before-volume-and-gain-and-routing-stay-flat.md)**: `volume` is post-fader. An element's `audio_effects`
> list runs after `aloop` and before `volume`, so a fade or a duck is never undone by a
> compressor; pan/balance follows `volume`.
>
> **Amended by [ADR-0077](./0077-the-nine-render-readings-are-ratified.md)**, which states how the mix
> carries this ADR's decisions: the bus is **48 kHz stereo** so every `aloop` sample count
> is exact from the document alone, `amix` runs with **`normalize=0`** so two lines at
> `1.0` are each still at `1.0`, and a **keyframed `volume` is applied as the value
> `resolve` computes on every sampled frame**, as timed commands, rather than re-expressed
> in `ffmpeg`'s expression language.
>
> **Amended by [ADR-0157](./0157-a-speed-ramp-is-a-time-remap-curve-of-source-times-on-a-video-element.md)**: a
> `video` carrying `source_time` must carry `volume` as the literal `0`; a keyframe list or
> an absent `volume` is `E-REMAP-AUDIBLE`.

[ADR-0012](./0012-flat-transform-keyframes-carried-by-their-element.md) already
named the trap — an agent will write `opacity` on an audio element meaning
volume, and it fades nothing, forever — but deferred naming the real
replacement. No volume field exists anywhere today: not in any ADR, not in
`CONTEXT.md`, not on any of the fixture's 20 audio elements (all narration,
no music bed). Graduated from [#12](https://github.com/MBehtemam/Montagent/issues/12)
via the map's "Audio mixing model" fog entry, as [#126](https://github.com/MBehtemam/Montagent/issues/126).

## Decision

### `volume` is a keyframable property, using the existing `{t,v,ease}` mechanism

Not a flat-only scalar with a separate `fade_in`/`fade_out` pair. A fade *is*
two keyframes (`{"t":17472,"v":0}`, `{"t":17972,"v":1,"ease":"linear"}`);
inventing dedicated fade fields would be both the second animation mechanism
[ADR-0025](./0025-clip-stays-static.md) already pre-committed against ("any
time-varying effect parameter must reuse the existing keyframe/SPLIT
machinery rather than invent a second one") and a second spelling of what
keyframes already say — precisely the "two spellings of one freedom" hazard
this project's decision history keeps rejecting. `volume` stays a plain
scalar when constant, the same scalar-or-keyframe-array polymorphism every
other animatable property already has.

Decided **unanimous 3/3** (Opus, Haiku, Fable).

### Value domain: a linear multiplier, `0` = silent, `1` = source level (default), `>1` = amplify; negative is a schema error

Mirrors two conventions already in the format: `speed`
([ADR-0020](./0020-speed-overrun-hold-loop.md), "a rate you multiply by")
and `saturation{amount}` ([ADR-0049](./0049-v1-colour-filter-vocabulary-four-scalar-members.md),
`0`/extreme, `1`/identity, `>1`/intensified). An agent that has learned one
Montagent scalar has learned this one. Decibels were rejected: the scale is
logarithmic and the identity value would be `0`, which means "off" in every
other field in this format — a reader cannot tell the level at a keyframe
without doing log arithmetic, and the vocabulary is a UI for the agent, not
an audio engineer. A 0–100 percentage was rejected as the same freedom with
a different decimal point, clashing with every other scalar in the format.
Unlike `speed`, `0` must be legal here — silence is a common, legitimate
intent, and it is what makes the mute decision below work with no second
field. `>1` is permitted rather than capped; clipping past that point is the
renderer's documented behaviour, not a schema-enforced ceiling — the same
posture ADR-0015 takes toward `contain`'s zero-extent case (report the
arithmetic, don't clamp it).

Noted but not adopted as a schema concern: linear amplitude is not
perceptually linear (`0.5` does not sound "half as loud"), and this ADR
leaves that to the renderer and to `ease`, the same way this project has
kept fps-dependent and perceptual arithmetic out of the schema layer
elsewhere ([ADR-0035](./0035-keyframe-grid-alignment-is-a-review-check-not-a-schema-rule.md)).

Decided **unanimous 3/3** (Opus, Haiku, Fable).

### `video`'s embedded audio reuses the same `volume` field; no separate `mute`

A `video` element is one element with intrinsic audio, not a visual paired
with a separate audio object — there is nothing else to mute. A `mute: true`
boolean sitting next to a `volume` value (or a keyframed fade) is the
textbook two-spellings-of-one-freedom hazard: a reader would need a
precedence rule to know the effective level, exactly the drift this
project's inertness principle exists to prevent. `volume: 0` already says
"silent," including at a specific instant via a keyframe, with no second
field and no ambiguity to resolve.

Decided **unanimous 3/3** (Opus, Haiku, Fable).

### Automatic ducking is deferred; the author hand-keyframes the dip

Ducking — a music bed automatically dropping in level while narration plays
— makes one element's effective volume a function of another element's
presence and timing. That is a live, computed relationship read off the
*document's structure*, not off a value in the document, the same forbidden
shape this project has rejected every time it has appeared (live time
anchors, coupled keyframe timing, expressions). It is not solved by
declaring intent either: a threshold, attack, release and target level all
belong to a detector, not to a fact about the project. The author — who
already placed every narration clip's `start`/`end`, having written them —
authors the identical outcome today with an ordinary keyframe pair per
narration boundary on the music element (`1 → 0.2` before speech, `0.2 → 1`
after). Nothing new is needed for the field the format already has.

Two jurors (Opus, Fable), independently, named the same future direction
if the hand-authoring cost proves real: an MCP write **tool** that computes
and writes the keyframes into the document — never a live relational field.
Such a tool would take a complete element write (per the standing write-tool
invariant) with the narration elements' times as ordinary input, and its
*output* is the same inert keyframe array an author could have typed by
hand. This stays unbuilt for v1: the one fixture measured has no music bed
at all, so there is no evidence yet of the tedium a tool would need to
justify, and this project's own standing rule treats a fixture's *absence*
of a case as no evidence of unneed — but equally no evidence of urgency.
Recorded here so the door is visibly open in the one direction that doesn't
reintroduce evaluation into the file.

Decided **unanimous 3/3** (Opus, Haiku, Fable) on deferring; the tool-shaped
escape hatch is the author's synthesis of two jurors' independent additions,
not itself put to a vote.

### `volume` is a flat top-level field on the element, a peer to `speed`/`overrun` — not inside `effects`

[ADR-0040](./0040-effect-model-attachment-and-v1-vocabulary.md)'s `effects`
list is explicitly the visual/rasterization treatment chain (`blur`,
`shadow`, `mask`, colour filters), with meaningful order. Volume has no
order relative to those — it is not a treatment layered onto a rendered
result, it is an intrinsic property of the element's own playback, the
audio counterpart of `opacity` (which ADR-0012 already places top-level) and
the direct sibling of `speed`. An `audio` element has no `effects` list
today; putting `volume` there would force one into existence solely to hold
a single entry, or split the field's location by element type — the worst
outcome for an agent scanning the schema for the field ADR-0012's error
message told it to look for.

Decided **unanimous 3/3** (Opus, Haiku, Fable).

## Consequences

- Schema: `audio` and `video` elements gain `volume`, either a scalar number
  (`>= 0`) or an array of `{"t","v","ease"}` keyframe records with `v >= 0`,
  defaulting to `1` when omitted. Negative values are a schema error, same
  class as `speed`'s. `ease` is required on every non-first record per
  [ADR-0038](./0038-ease-is-required-on-every-non-first-keyframe-record.md)
  — no new rule needed, the existing keyframe mechanism already covers it.
- ADR-0012's `opacity`-on-audio schema error message now names `volume` as
  the real replacement, discharging the deferral it stated.
- `CONTEXT.md` gains a **Volume** glossary entry alongside **Speed** and
  **Overrun**.
- No `mute` field is added, on any element type.
- Automatic ducking is out of v1. A future ducking-generator write tool is
  named as the intended escape hatch if hand-authoring proves too costly,
  but is not designed or scheduled here — not sharp enough to ticket without
  evidence of the cost it would need to justify building.
- `overrun: "hold"` stays a schema error on audio (CONTEXT.md's existing
  rule) — `volume` does not interact with it; a silenced tail is still
  correctly expressed as a shorter element plus a gap.
- No change to `effects` (ADR-0040) or to the write-tool invariant.

## Evidence

Three-juror independent court (Opus, Haiku, Fable), blind to each other,
one ballot per sub-question, five sub-questions: **unanimous 15/15**. Full
ballots recorded in the resolution comment on
[#126](https://github.com/MBehtemam/Montagent/issues/126).
