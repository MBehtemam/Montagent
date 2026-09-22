---
status: accepted
amends: 0007 (strikes "text background box" from #22's candidate list, already struck by ADR-0014; documented here as inherited, not reopened), 0010 (names its "obvious members" — blur, drop shadow — as the accepted v1 set), 0012 (confirms skew stays out of the transform model and does not migrate here), 0014 (confirms stroke stays a paint field, not an effect member)
---

# The effect model: attachment, order, v1 vocabulary, and what's deliberately absent

> **Amended by six later ADRs.** Read them before relying on anything below.
>
> - [ADR-0068](0068-the-bare-mask-key-retires-masks-are-effects-members.md) — **retires this
>   ADR's "no migration needed" Consequences bullet**, which contradicts its own schema
>   clause: masks live in `effects`, so the fixture's bare `mask` key is an unknown key and
>   does migrate. Also states the param-less `mask` member's geometry, which this ADR wrote
>   as `...shape params` and left undecided
> - [ADR-0048](0048-per-word-highlighting-is-a-timed-window-on-the-run.md) — settles the
>   deferral that ADR excluded from its own scope
> - [ADR-0049](0049-v1-colour-filter-vocabulary-four-scalar-members.md) — settles the
>   colour-filter deferral that ADR named and excluded from its own scope
> - [ADR-0055](0055-audio-mixing-model-volume-fades-ducking-deferred.md) — clarifies
>   `effects` is scoped to visual treatment
> - [ADR-0059](0059-transitions-element-type-crossfade-only-exact-window.md) — confirms the
>   "own shape — likely id-targeting" prediction; the effect vocabulary's element-locality
>   is not stretched to cover transitions
> - [ADR-0084](0084-the-mask-rect-is-one-shape-independent-parameter-set.md) — **writes the
>   `...shape params` ellipsis this ADR left unwritten**: the `mask` member's parameters are
>   one shape-independent rect, and per-shape field sets are refused as ADR-0049's two-level
>   lookup. Also states the coordinate space and transform behaviour this ADR's effect model
>   implied but never said

[#22](https://github.com/MBehtemam/Montaget/issues/22) asked five questions. #19 had
already fixed the vocabulary as closed, named and published in the schema — never a
stack in the After Effects sense, never a plugin architecture. What #22 owed: how an
effect attaches, whether order is meaningful, the concrete v1 list, whether text effects
are a distinct mechanism, and what is deliberately absent so the list reads as a decision.

Item 5 (schema expressibility of a discriminated union) was resolved before this ticket
started, by ADR-0009: the Rust host's `jsonschema` crate enforces full JSON Schema
2020-12 against a hand-written, embedded document, so a discriminated union keyed on
effect name is expressible regardless of what any SDK would have generated.

## Decisions

### Attachment: `"effects": [...]`, a list field on the element, order significant

Not an id-targeting element (transitions' natural shape), not a per-type baked-in
property. The deciding test is the same one that closed #19's vocabulary question:
agent discoverability by reading the schema. A list field keeps the whole vocabulary
reachable from the element it applies to — read one element, see every effect on it,
in the order they apply. Baking effects into element types scatters the vocabulary
across every type's schema and forces N×M duplication; id-targeting buys nothing for
element-local effects and adds referential-integrity policing (dangling ids, cross-track
targeting, ordering ambiguity when two effect elements target one id) for a concept
(transitions) this ticket explicitly excludes.

Order is semantically real — blur-then-shadow is a different frame from
shadow-then-blur — so the container is a list, not a map, and two effects of the same
name (two shadows) are ordinary rather than forbidden. `compare`/`validate` are free to
flag a duplicate as a review-level finding later; the schema does not forbid it.

**This does not foreclose transitions.** A transition is a timeline concept between two
elements, not an element-local one, and will need its own shape — likely id-targeting —
in its own future ticket. Choosing a list here does not tax that decision.

### v1 vocabulary: `blur`, `shadow`, `mask` (shape-only). Colour filter and skew deferred.

Three effects, each passing every test applied to it: a closed, small, purely-numeric
parameter surface; a thing the CapCut/Premiere reference class treats as basic; evidence
from the fixture or the reference class rather than assumption.

- **`blur`** (gaussian, one parameter: radius). Canonical closed effect, native to
  skia-safe.
- **`shadow`** (drop shadow: offset x/y, radius, colour, opacity). Same shape as blur,
  and the fixture's own subtitle format has a slot for it — all 35 ASS styles set
  `Shadow: 0`, meaning the concept is addressable in the source material even though this
  project didn't use it.
- **`mask`, shape-only** (a closed shape vocabulary: `circle`, `rect`, `ellipse`, with
  numeric parameters only — no image-source or alpha/soft mask). This is the strongest
  evidenced candidate of any considered: the committed fixture carries an undeclared,
  currently-inert `mask:"circle"` on `handle-logo`. Per ADR-0003's asymmetry that is
  evidence the capability is **needed** — a real authoring gesture the schema has no
  opinion about today. Shipping it converts that stray field into either a valid
  declaration or a validation error. Shape-only is the line: a shape mask is fully
  numeric and enumerable from the schema alone, consistent with the closed-vocabulary
  rule. An image-source soft/alpha mask is a different kind of thing — it drags in a
  second asset reference inside an effect, and immediately opens unanswered questions
  (fitted how? luminance or alpha? what if the source is a video?) that are each their
  own small ADR. Deferred, not rejected — see Not Yet Specified below.

**Colour filter is deferred, not included**, on a closed-vocabulary argument rather than
a difficulty one: "tint / grayscale / duotone / etc." is the shape of a family that does
not stop closing itself — grayscale invites sepia invites duotone invites
brightness/contrast/saturation invites curves invites LUTs. A vocabulary that admits one
open-ended family has quietly stopped being closed, even if every member proposed today
is small. If a colour operation is needed, it gets a named parameter set and its own ADR
stating explicitly where the family stops, not a line item borrowed from this one.

**Skew is deferred, not adopted.** It was already excluded from the transform model
(ADR-0012) on reference-class grounds — CapCut has no skew, Premiere's Motion panel has
none. "Rejected from its natural home, therefore adopt it here" is not a reason on its
own: skew is a geometric matrix, and admitting it as an effect would put it on the
opposite side of the transform/effect boundary from `rotation` and `scale`, its closest
relatives, purely because it was turned away once. It also uniquely among effect
candidates *changes an element's bounds*, which none of blur/shadow/mask do and which
complicates every other effect's bounds reasoning if admitted alongside them. No evidence
names a forcing case.

### Text effects: the same mechanism. Per-word/karaoke highlighting is out of this ticket.

A drop shadow on a text element is a whole-element pixel post-process — it does not
address runs or glyphs — so it is exactly what `effects` is for. No separate
text-effects concept exists; inventing one would duplicate the vocabulary for elements
that are, for this purpose, ordinary.

Per-word (karaoke-style) highlighting is excluded from this ticket's scope entirely, on
the same boundary [ADR-0014](./0014-stroke-is-paint-the-text-box-is-required.md) already
drew for `stroke`: *effects attach to elements, and a run lives inside one.* Reversing
that here to let highlighting in would make every effect's addressing ambiguous, not
just this one's. It fails a second, independent test too: its interesting form is
animated against word timings, and ADR-0012 restricts keyframes to transform properties
on the element's own absolute clock — a per-run, audio-timed paint change is neither a
transform nor addressed at the right granularity. Resolving it inside an effects-
vocabulary ticket would settle a run-addressing model and a non-transform timing model
as side effects of a question about blur radii — precisely how a format acquires an
accidental exception.

A **static** per-word highlight (a fixed paint on a fixed run, no timing) is already
expressible today with no schema change — the same run-level style-delta mechanism
`stroke` lives on. Only the **animated**, audio-synced form is the actual gap, and it is
the whole of what the future ticket (graduated below) has to design.

### Deliberately absent from v1

Recorded by reason, so a reader can tell a closed decision from an oversight — and so
the two are never confused with the four items below that are *not* absent, only
relocated.

**Deferred, with a named successor:**
- **Per-word / karaoke highlighting** — needs a run-addressing model and a non-transform,
  audio-synced timing model. Graduated to a new ticket, below.
- **Colour filter** — needs its own closed, bounded vocabulary and its own ADR stating
  where the family stops. Graduated to a new ticket, below.
- **Soft / alpha / image-source masks** — needs an asset-reference model for effect
  inputs (fitting, luminance-vs-alpha semantics, video sources) that does not exist yet.
  Left in Not Yet Specified: not sharp enough to ticket until that prerequisite exists.

**Rejected on model grounds, not deferred:**
- **Skew** — a geometric transform, consistently excluded alongside its rejection from
  the transform model (ADR-0012).
- **3D / perspective distortion** — the format's spatial model is explicitly 2D (`x`,
  `y`, `scale`, `rotation`, a non-rotating rectangular `clip`); this isn't a missing
  effect, it's outside the coordinate model.

**Out of scope for a read-not-executed, agent-first format:**
- **Chroma key / green-screen** — its result depends on source pixels and
  tolerance/spill parameters no document can predict; not readable from the schema
  alone.
- **LUTs** — semantics live entirely in an external `.cube` file, which is a plugin in
  all but name and directly against the closed-vocabulary rule.

**Composable today, not shipped as a named effect — record what to use instead, not a
bare absence:**
- **Vignette** — express as a `rect`/`ellipse` element with a gradient or feathered
  shape mask over the frame.
- **Glow** — a composite of blur, additive blend and colour; waits on blend modes
  existing as their own concept, not on this ticket.

**Boundary markers, stated even though nobody proposed them, because they are the
likeliest silent wrong assumption:**
- **Animated effect parameters** — v1 effects are static. Keyframes are transform-only
  (ADR-0012), so an agent cannot keyframe a blur radius or a shadow offset in v1. This is
  the single most likely thing a future reader assumes works and is quietly wrong about.
- **Particle / generator effects** — an effect filters an existing element; it never
  authors new visual content. This is the line that keeps the model from becoming a
  rendering language.
- **Audio effects** — the effect model is visual. Audio operations, if any arrive, are
  their own vocabulary on audio elements, not a branch of this one.

**Explicitly kept off this list, because they were relocated rather than rejected, and
listing them here would misread as the opposite:** transitions (out of scope by being a
separate, future, timeline-level ticket — not considered and declined as an effect),
`stroke` (a paint field, ADR-0014), the text background box (already a `rect` element,
ADR-0014), Ken Burns and `clip` (transform-model fields, ADR-0012). An "absent from v1"
list means "you cannot do this in v1." All four of these you can.

## Consequences

- `mask:"circle"` on `handle-logo` in the committed fixture becomes a valid declaration
  under this ADR rather than an inert stray field — no migration needed, since the value
  was already legal shape-vocabulary syntax; `validate` should stop treating it as an
  unknown key once the schema lands.
- The schema gains a discriminated union `effects: [{name, ...params}]` with exactly
  three members in v1: `blur{radius}`, `shadow{dx, dy, radius, color, opacity}`,
  `mask{shape: "circle"|"rect"|"ellipse", ...shape params}`.
- Two new tickets graduate from this ADR's deferrals (below), and one fog entry is added
  for soft/alpha masks, not yet sharp enough to ticket.
- `#22`'s item 5 (schema expressibility) needed no new work here, having been closed by
  ADR-0009 before this ticket started.

## Evidence

**A three-juror court** (Opus, Haiku, Fable — via `/court`), each given the same
question packet cold, blocked from each other's ballots and from the author's
recommendation. Full ballots and the Judge's read are committed at
`docs/research/juries/effect-model/BALLOTS.md`. Unanimous 3/3 on attachment (list field,
order significant), on
including blur/shadow/shape-only-mask while deferring skew and soft masks, and on text
effects sharing the mechanism while deferring per-word highlighting. Split 2–1 on colour
filter (Haiku and Fable would include a small closed list; Opus would defer it on
closed-vocabulary-erosion grounds) — resolved here in Opus's favour, on the project's own
track record of "closed" categories drifting once a first small member is admitted.
Split on whether transitions belong on the absent-list at all (Opus: no, it misreads as
rejection; Fable: yes, as a cross-reference pointer; Haiku: silent) — resolved here in
Opus's favour, for the same misreading risk stated above, generalised to the other three
relocated concepts Opus named and Fable's ballot did not.
