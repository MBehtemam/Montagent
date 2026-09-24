---
status: accepted
amends: 0004 (array order carries no meaning is extended from timing to stacking; the track non-overlap rule's sibling gets its own check), 0006 (the geometry-aware same-layer check finally gets a home and a severity), 0011 (records the fallback-order question this ADR closes as no longer open)
---

# A geometry-overlapping layer tie is an error, not a fallback order; array order stays meaningless

Two elements can end up at the same resolved integer `layer` — most commonly two
different tracks both declaring `layer: 30`. Nothing in the format says which one
draws in front. The fixture has stayed benign so far only by coincidence (its two
tied clusters happen not to occupy the same pixels), and that coincidence has
already produced two independent silent failures: a Python `query --at` script and
a `jq` one-liner each invented a different draw order at the same tie, and a
candidate renderer built for a different ticket printed the same three-element stack
on all 47 sampled rows, never once naming the photo or caption beneath it. ADR-0006
sent "the geometry-aware same-layer check" to #24; #24 turned out to be the anchor
repair, not this question, and the check has had no owner since — resolving
[#134](https://github.com/MBehtemam/Montagent/issues/134), graduated from
[#10](https://github.com/MBehtemam/Montagent/issues/10).

Settled by a 3-juror court (Opus, Haiku, Fable), independent, blind to each other,
put the same three questions below. Unanimous on the first and third; 2–1 on the
second, and the dissent's own reasoning collapses once the first is settled — see
"Considered and rejected."

## Decisions

**A tie is `error`, never `review`, when the two elements' boxes overlap in both
time and space.** [ADR-0006](./0006-validate-reports-facts-and-render-enforces.md)'s
scheme reserves `review` for "legal, renders, and a human must look at a frame to
know if it was meant" — but there is no single frame to look at here, because which
frame renders is implementation-defined. Two tools have already disagreed. That is
`error`'s own definition: "the render is refused or is guaranteed wrong," because
whichever order ships is exactly as defensible as its opposite. The fix costs the
author one field — an integer `layer` override, or an anchor naming the other
element — using tooling the format already has
([ADR-0019](./0019-layer-anchor-gets-an-id-a-validate-check-and-one-hop.md)). `render`
already refuses on any `error`
([ADR-0006](./0006-validate-reports-facts-and-render-enforces.md)), so choosing
`error` here means **no fallback drawing order is ever needed for the conflicting
case** — the render simply does not happen until the author states one.

**Array order stays meaningless, full stop — never promoted to mean stacking, not
even narrowly for non-conflicting ties.** [ADR-0004](./0004-tracks-as-constrained-lanes.md)
already fought to keep array order meaningless for *timing*, specifically because
readers kept mistakenly treating a sequential-looking array as expressing sequence.
With the conflicting case settled as `error` above, the only ties left are ones
whose boxes never overlap — and by construction nothing on screen depends on their
order, so there is nothing left to define. Undefined-and-unobservable beats
defined-and-tempting: a "meaningful only for non-conflicting ties" rule is not one
authors or agents will carry in their heads, and a tie that starts non-conflicting
and becomes conflicting after an edit (an animation added, a box resized) would
silently inherit a rule nobody meant to lean on. Implementations may sort
non-conflicting ties by any stable internal method they like; the file makes no
promise and none is checked.

**The check samples across the elements' shared time range, not just their rest
extents.** Positions and scale can be keyframe-animated
([ADR-0012](./0012-flat-transform-keyframes-carried-by-their-element.md)); two
same-layer elements can be spatially disjoint at rest and swept into overlap
mid-animation by a pan or zoom, or the reverse. A static check both misses real
conflicts (a title panned into another title mid-shot renders differently in two
tools, `validate` stays clean) and — now that the verdict is `error` — wrongly
refuses valid projects whose elements are animated apart and never actually
co-occupy space. Evaluate both boxes at each keyframe boundary plus an interval
between them, across the elements' temporal intersection only. Scope stays narrow:
only element pairs that already share a resolved layer need this, and only across
their shared time range — not a pairwise scan of every element in the project.

## Considered and rejected

**`review` with a documented fallback order (2 of 3 jurors' first instinct, one
juror's final vote).** Requires the format to define what breaks the tie when it
renders anyway. The only candidate on offer with no schema cost is array order —
which reopens exactly the ambiguity ADR-0004 paid to close, in a new dimension.
The dissenting juror argued declaration order is "natural" and safely narrow because
non-overlapping elements "can't be confused with sequencing" — but that safety only
holds while the elements stay non-overlapping, and nothing prevents a later edit
from making a previously-inert tie start mattering, at which point the fallback
silently promotes from "never observed" to "the actual picture."

**Static-extent geometry check (all three jurors, unprompted).** Cheaper to build,
but paired with an `error` verdict it either lets a real mid-animation collision
through undetected or refuses a legal project whose elements never actually
co-occupy space — both failure modes unacceptable once the check gates the render
rather than merely reporting.

## Consequences

- `validate` gains a new `error`-level check: for every pair of elements sharing a
  resolved integer layer, sample both boxes across their temporal intersection
  (keyframe boundaries plus an interval); flag if any sample overlaps. `render`
  inherits the same check and refuses under it, per ADR-0006's existing pattern.
- The check needs no paint/effect awareness (opacity, fill colour, occlusion) to
  be correct at `error` level — it is purely geometric. A same-layer pair whose
  boxes overlap but one is fully transparent is still `error`: the format has no
  way to state "this collision is intentional and harmless" other than the same
  `layer`/anchor fields that resolve the tie outright.
- `CONTEXT.md`'s **Layer** and **Track** entries are updated to state the tie rule
  and to extend "array order carries no meaning" explicitly to stacking, not only
  timing.
- The fixture's current same-layer clusters (`chip-panel`/`handle-panel` at 30,
  `flag-field`/`handle-logo`/`handle-text` at 31) stay legal, since their boxes are
  spatially disjoint for their entire shared time range — this ADR changes zero
  bytes of the committed file, and adds a live regression case for the check once
  implemented (an edit that moved either cluster into the other's box would need
  to start failing `validate`).
- `query`'s presence-half finding (ADR-0011) — that it silently invented an order
  at a layer tie — is now closed structurally: the only ties `query` can encounter
  post-`validate` are non-conflicting ones, where any consistent internal order is
  correct by definition.
