---
status: accepted
---

# Off-canvas is a standing `review` check, not a frame-change census

**Ticket:** [#84](https://github.com/MBehtemam/Montagent/issues/84)
**Amends:** [ADR-0006](./0006-validate-reports-facts-and-render-enforces.md) (adds a check),
[ADR-0012](./0012-flat-transform-keyframes-carried-by-their-element.md) (the evidence
this check is built on)
**Resolves the second of two findings [ADR-0018](./0018-cross-track-coverage-is-group-scoped-not-a-union.md)
graduated and declined to answer** (the first, group-shared keyframe-time
disagreement, is [#71](https://github.com/MBehtemam/Montagent/issues/71)/[ADR-0039](./0039-group-keyframe-time-check-retracted-from-validate-reassigned-to-compare.md))

## Decision

`validate` gains **`R-OFF-CANVAS`**: for every visual element, if the element's
declared rect, resolved across its *entire* active timeline range including every
keyframe, **never intersects the project's current `frame` at any instant**, that is
a finding.

Four things settle its shape, against the ticket's own four open questions:

1. **It is a standing invariant, not a history-triggered check.** `validate` always
   compares current element geometry against the current `frame`, exactly like the
   existing gap/overlap checks — never conditioned on "a `frame` edit just
   happened." The name **frame-change census**, in the ticket's own title, is
   retired along with the premise it implied; the check is named for what it
   checks (`R-OFF-CANVAS`), not for the edit class that happens to be its most
   common cause.
2. **The trigger condition is whole-range, not per-instant.** An element that is
   off-canvas at *some* instants of its active range — a slide-in starting at
   `x:-500` and animating to `x:0`, a slide-out doing the reverse — is ordinary,
   legal animation vocabulary and fires nothing. The finding fires only when the
   rect **never** intersects the frame across the element's whole active range: a
   fully-determined, document-only fact with no animation-intent judgment in it.
3. **Severity is `review`.** Off-canvas geometry is legal — it is exactly where
   every slide-in starts and every slide-out ends — and an element parked entirely
   off-canvas (scratch data, a work-in-progress placement mid-retarget) is not
   itself a defect the render refuses over. It is "legal, renders [nothing, for
   this element], and you must look to know if it was meant" — ADR-0006's own
   definition of `review`.
4. **No coupling to `measure` exists, and none is designed here.** The ticket's
   premise that a `frame` change leaves some text elements' `measure` results
   "unverified" does not survive inspection: `measure` consumes only an element's
   own `box`, font, size and content (ADR-0007), none of which a `frame`-dimensions
   edit touches. This is dropped from scope entirely, not deferred.

**No `role`/`kind` field is introduced.** The finding applies uniformly to every
visual element type — this check is a plain rect-against-frame intersection test
with no opacity, occlusion or area ambiguity, so it needs no per-element
classification to decide anything, unlike the coverage question ADR-0018 answered
by staying away from exactly such a field.

## Why

### The evidence

[ADR-0012](./0012-flat-transform-keyframes-carried-by-their-element.md) measured the
defect this check exists for: retargeting the fixture from 1080x1920 to 1920x1080
leaves **32 of 60 elements entirely outside the new frame**, and the 28 survivors are
exactly the 20 audio elements (which have no spatial extent) plus the 8 header-chrome
elements whose fixed pixel margins happen to still fit. That measurement is also why
this format uses absolute integer pixels rather than fractional/relative units —
fractions were rejected specifically because they hide exactly this class of defect
behind numbers that stay in range while the content they describe becomes
unrecognisable. Absolute pixels make the defect visible in principle; this ADR is
what makes `validate` actually say so.

### Why a standing invariant, not an edit-triggered one

Montagent has rejected every mechanism that requires knowing "what just happened" to
a file, for the same reason each time: [ADR-0005](./0005-absolute-integer-milliseconds.md)
rejected persisted time-anchors because they degrade to stale literals the moment
anything shifts; the derived-time-signature question ([#68](https://github.com/MBehtemam/Montagent/issues/68))
stays parked in this map's fog precisely because nothing records provenance and the
format is read-not-evaluated. An off-canvas element is a fact about the **current
file** — it is exactly as invisible whether a `frame` edit, a hand-edited `x`, or a
pasted-in element from another project put it there. Scoping the check to "only after
a `frame` edit" would require inventing the one thing this format has refused six
times over, to catch a strict subset of the cases a standing check catches for free.

### Why whole-range, not per-instant

The evidenced defect (32/60) is entirely the whole-range case: a **static** rect,
never repositioned after a retarget, sitting nowhere near the new canvas for its
entire life. A per-instant check would also fire on every legitimate slide-in and
slide-out — normal animation vocabulary this format has never restricted — and
firing there would force `validate` into judging whether a particular animation
choice "makes sense," which is precisely the intent-judgment ADR-0006 bars a finding
from making. Whole-range off-canvas has no such reading: an element that contributes
zero pixels across its **entire** active life is a fact, not a style choice.

### Why `review`, not `error`

The deciding question is what ADR-0006's `error` — "the render is refused or is
guaranteed wrong" — actually refers to: the *render*, not any one element's
individual purpose. An off-canvas element does not make the render wrong; it makes
the render correctly reflect a file in which that element currently contributes
nothing. Nothing in the format forbids that geometry — off-canvas is where every
slide-in starts and every slide-out ends, so the position itself carries no defect on
its own. Blocking `render` on this would also land badly on the exact workflow the
evidence is drawn from: an agent mid-retarget, having moved some but not all of 32
elements back on-canvas, would be refused a render of its own work-in-progress file
before it has finished — a harsher outcome than the file's own severity warrants,
since the file itself is not thereby wrong.
Escalating `review` to `error` later, if evidence shows agents routinely ignore this
finding, is a one-line amendment; walking back an `error` that blocked a legitimate
workflow is the harder direction to correct from.

This was the one split question put to a three-model court (Opus, Haiku, Fable),
2–1 for `review`. The dissent's `error` argument — that an element guaranteed never
to render is "guaranteed wrong" in the same sense as a keyframe-time disagreement —
does not survive the render/element distinction above: severity in this format has
consistently attached to the consequence for the render as a whole (ADR-0006's
"severity is computed from the consequence at an instant," ADR-0018's
group-scoped-not-project-wide coverage), never to whether one element individually
achieved its own apparent purpose.

### Why no `role`/`kind` field

[ADR-0018](./0018-cross-track-coverage-is-group-scoped-not-a-union.md) already
settled this question for a harder version of it — audio/visual coverage, which
needs opacity, colour, z-order and a settled text model to answer honestly, and
still rejected a declared field because it drifts from truth in both directions (a
repurposed chrome element leaves a stale label; a duration-based heuristic
misclassifies a legitimate full-length asset). This check is strictly simpler: a
rect either intersects the frame across its whole active range or it does not, with
no occlusion or opacity term to get wrong. There is nothing here a `role` field
would even help decide, so ADR-0018's reasoning transfers without needing to be
re-litigated.

### Why the `measure` coupling is dropped, not deferred

The ticket's fourth question assumed a `frame` edit could leave a text element's
`measure` result "unverified." Tracing the actual data flow: `measure` reads a text
element's `box`, its declared font and size, and its content (ADR-0007) — every one
of these is a field on the element itself, flat, and none is expressed relative to
or derived from the project's `frame`. A `frame`-dimensions edit changes exactly one
field, `frame`, and touches nothing on any element. There is no invalidation to
detect, so there is no provenance question to design an answer for — unlike the
derived-time-signature question this map keeps genuinely parked, this one dissolves
under inspection rather than needing further evidence.

## Consequences

- **`validate` gains `R-OFF-CANVAS`**, `review` severity: for each visual element
  whose declared rect, resolved across every keyframe over its full active range,
  never intersects the current `frame`, report the element id and (per ADR-0006's
  "every number inline" rule) its resolved bounding rect across that range against
  the frame's dimensions.
- **No new field.** `frame`, element geometry and keyframes are unchanged; the check
  reads only what the schema already requires.
- **The ticket's `measure`-invalidation question is retired, not answered
  elsewhere** — there was nothing to answer.
- **The check is general**, firing regardless of what produced the current state —
  a retarget is its most likely cause in practice, not a condition in its trigger.

## Reopening condition

Exhibit a legitimate authoring pattern in which an element is meant to be off-canvas
for its **entire** active range (not merely at some instants) — for instance, a
"disabled" or "staged" element an author wants to keep in the file without deleting
it. If such a pattern is real and common, the resolution is a scoped mechanism for
it (an explicit `enabled`-shaped field, most likely — not a `role` field, which
would misclassify rather than declare), not a weakening of this check's severity or
trigger.

## Not settled here

- **Escalating `R-OFF-CANVAS` from `review` to `error`**, if usage shows agents
  routinely ignore it the way `error`-class findings cannot be. Deferred until
  there is evidence, per this ADR's own argument for why the cheaper direction is
  the one to start from.
- **Partial-overlap severity** — whether an element clipped mostly (not wholly)
  outside the frame deserves any finding at all. Nothing in the evidence motivates
  one; not designed here, and not fog, since nothing points at it being a real gap
  yet.
