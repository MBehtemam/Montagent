---
status: accepted
amends: 0038 (designs the review-level lint ADR-0038 named but did not design), 0006 (adds a `validate` check)
---

# `validate` gains `R-EASE-INERT`: a review check for an `ease` that describes no motion

[Ticket #122](https://github.com/MBehtemam/Montaget/issues/122), graduated from
[#70](https://github.com/MBehtemam/Montaget/issues/70)/[ADR-0038](./0038-ease-is-required-on-every-non-first-keyframe-record.md).
ADR-0038 made `ease` required on every keyframe record but the first, closing a
schema silence that had produced three different agent behaviors on the identical
text. It also named, without designing, the acknowledged cost of that decision: a
**hold segment** — two consecutive keyframe records whose `v` is unchanged, so
nothing is actually easing — still must carry an explicit `ease`, typically
`"linear"`, describing motion that doesn't happen. This ADR designs the
`review`-level check that tells an author when that has happened, resolved by a
three-juror court (Opus, Haiku, Fable), unanimous or 2–1 on each sub-question below.

## Trigger: literal equality, not resolved equality

The comparison is between two **author-written** `v` fields on adjacent keyframe
records of the same element and property — not a resolved, rendered, or
interpolated value. Nothing sits between the two records but the file itself: no
interpolation math, no rasterization, no round trip through a renderer. That is a
different situation from [ADR-0033](./0033-same-source-cut-continuity-is-a-review-check.md)'s
`R-SOURCE-CUT-POP`, whose per-property tolerance table exists specifically to
absorb float noise introduced by *resolving* a value across a cut. There is no
noise source here for a tolerance to absorb.

**Decided: exact equality, no epsilon** (3/3). If an agent writes `1.0` on one
record and `0.9999999999` on the next, that is a real — if minuscule — authored
difference, and a `review` finding asserting they're "the same" would be stating
an intent the file doesn't carry, the exact overreach ADR-0006 forbids a finding
from making. This mirrors [ADR-0034](./0034-caption-pace-and-repeat-duration-checks.md)'s
`R-CAPTION-REPEAT-DURATION`, which also compares literal authored content
(byte-identical text) with no tolerance, rather than ADR-0033's resolved-value
case. It also avoids inventing a second tolerance table with a threshold per
property (position vs. rotation vs. opacity are different units and different
"visually equal" thresholds) to define and maintain for a benefit — catching
near-miss float drift on a hand-typed hold — that is unevidenced in this
project's fixtures. If that failure mode turns up in practice, it is a
distinguishable defect (junk precision in an authored value) and earns its own
check rather than being folded into this one by weakening its trigger.

## Scope: whole-value equality, not per-component

`scale` and `origin` are vectors (`[sx, sy]`). A vector-valued `ease` can be
inert on one axis while doing real work on the other — e.g. `scale: [1.0, 1.0]
→ [1.5, 1.0]` genuinely shapes `sx` and describes nothing on `sy`.

**Decided: the check fires only when every component of the vector is
unchanged** (2/1). The dissenting juror's argument — that per-component firing
would be more complete, and could be phrased to name the held axis rather than
accuse the whole ease — is real and is recorded here rather than dropped: a
per-component variant remains available if evidence later shows single-axis
holds are common enough that authors want to know about them. But whole-value
equality is adopted for v1 because it matches ADR-0038's own framing exactly —
*"the value doesn't change between two keyframes, so nothing is actually
easing"* is a claim about the whole segment, and a scale that moves on one axis
is, in the ordinary sense, a segment where something eases. It also degenerates
identically to the scalar case for `x`, `y`, `rotation`, `opacity`, so the rule
needs no per-property branching. Firing on a partial hold risks nagging an
author for a legitimate, common construction (a horizontal-only stretch, an
origin pinned on one axis) with a finding whose fix isn't obvious under the
schema's current shape — there is no way to write "ease only applies to sx."

## Finding code and severity

**`R-EASE-INERT`**, `review` (3/3 on the name). Matches this project's
established grammar for `R-`-prefixed codes — subject then fault
(`R-SOURCE-CUT-POP`, `R-CAPTION-PACE`) — rather than naming the segment shape
that produces the defect (the rejected `R-HOLD-EASE`). A hold segment is not
itself a defect — the schema requires and permits them — so a code built on
"hold" would name a legitimate construct rather than the actual complaint,
which is that the *ease* attached to it does nothing. Keeping the subject on
`ease` also clusters correctly with any future `ease`-scoped checks
(`R-EASE-*`) rather than colliding with a hypothetical future check about
holds themselves (e.g. an over-long hold).

Severity is `review`, not `error`: the segment renders correctly and legally —
an inert `ease` changes nothing about the video — so this is squarely
ADR-0006's *"legal, renders, and you must look at [the file] to know if it was
meant"* case, not a defect that blocks `render`. Not `note`: `note` is for a
fact the reader will not act on today, but an inert ease is exactly the kind of
in-file information debt this ADR gives an author a live opportunity to correct
(delete the field's meaning by acknowledging it costs nothing, or replace it
with an ease that would matter if `v` is later edited to actually move).

## Grouping and message

**One finding per run of consecutive hold segments on one element/property**,
not one per pair — the same "scope the output, never the analysis" discipline
ADR-0006 and ADR-0034 already apply: three keyframe records all sharing one `v`
produce two inert-ease segments back to back, and reporting them as two lines
duplicates the same fact. The finding states the facts and stops, per
ADR-0006's "every number inline" rule:

> *Example: `R-EASE-INERT` on `photo-06.scale` — 2 consecutive keyframes hold
> `v=[1.0000,1.0000]` from `t=12000` to `t=18000`; `ease="linear"` describes no
> motion.*

No repair is proposed (delete the `ease`? change `v`? leave it?) — that is an
authorial-intent judgment ADR-0006 forbids a finding from making, symmetric
with `R-SOURCE-CUT-POP`'s refusal to say whether a pop was deliberate.

## Consequences

- **`validate` gains `R-EASE-INERT`** (`review`): fires when two or more
  consecutive keyframe records on the same element/property carry an
  identical, literal `v` (whole-value equality for vectors, no tolerance), and
  reports the run as one finding naming the property, the value, and the time
  span.
- **ADR-0038's acknowledged cost is discharged**: the "future review-level
  lint" it named as the honest remedy for required-but-inert `ease` now has a
  code, a trigger, and a message shape.
- **No schema change.** `ease` remains required on every non-first keyframe
  record exactly as ADR-0038 states; this check is advisory only and `render`
  does not refuse on it.
- **Recorded, not designed:** a per-component variant of this check (firing
  when *any* single axis of a vector is held, rather than requiring the whole
  vector to hold) was considered and set aside for v1. If wide-vector
  properties (`scale`, `origin`) turn out to carry inert ceremony on one axis
  often enough to matter, that is a narrower follow-on check, not a
  reopening of this one's exact-equality or whole-value decisions.
