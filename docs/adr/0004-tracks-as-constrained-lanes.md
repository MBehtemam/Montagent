---
status: accepted
supersedes: partially supersedes 0001
---

# Elements live in tracks: constrained lanes with absolute times

> **Amended by [ADR-0019](./0019-layer-anchor-gets-an-id-a-validate-check-and-one-hop.md)**,
> which this ADR left open: an anchor's target namespace is elements' `id` only, never a
> track name; anchors may not chain; and `validate` checks both a missing target and a
> target that never overlaps the anchored element in time. Everything else below stands.

A project holds a `tracks` array. Each track is a named container with an integer
`layer` giving its stacking position — higher draws in front. Its elements keep
their own absolute `start` and `end`; **array order carries no timing meaning**.

**Children of one track may not overlap in time.** That is a validation error, not
a storage rule — the file can express it, `montaget validate` refuses it.

An element may **override its stacking** with its own integer `layer`, or with an
**anchor** naming another element: `"layer": {"below": "title"}` resolves to that
element's layer minus one, wherever either of them lives.

This replaces the flat `elements` array of [ADR-0001](./0001-flat-element-list.md).
Everything else in that ADR stands: no scene, no local clock, audio as a peer
element, no per-kind collections, inline `source`.

## Why

**The reference class stores exactly this.** [ADR-0003](./0003-general-video-editor-not-channel-tooling.md)
settled that Montaget is a general video editor in the CapCut/Premiere class.
Premiere's `getStartTime`, Resolve's `GetStart()` and CapCut's `target_timerange`
each pair a track with an **absolute time range per item** and a separate source
range ([research](../research/track-vs-layer.md) §2.2–§2.4). Field for field, that
is this decision. The shipping editors never made tracks and absolute times
alternatives; only the interchange formats — OTIO's `Track`, FCPXML's `spine` —
derive position from order, and Montaget is not an interchange format.

**ADR-0001 rejected the wrong thing.** Its objection was that "track" promises
sequencing it would not deliver. The research confirmed that premise five for five
from primary sources — and the fix is to *deliver* the sequencing constraint rather
than to avoid the word. A track here does constrain its children not to overlap,
which is what every reader arriving from any of those tools expects.

**Eight agents authored against both models and preferred this one, unanimously, on
the axis that matters.** Given four candidate formats and four realistic editing
tasks, eight independent agents ranked this shape **first for fewest mistakes, 8 of
8** — while only 6 of 8 ranked it easiest to *write*. Two ranked the flat list
easiest to type and simultaneously worst for errors. One put it plainly: *"ease of
typing and ease of staying correct are opposite forces here."* Since the format
exists to be authored by agents, that evidence is direct rather than analogical.

The mechanism they named repeatedly was **structural locality**: an insert means
scanning one small array rather than filtering a long flat one, so "I missed a
sibling" stops being a likely error. ADR-0001 reached for the same property by
convention — *"elements sharing a `group` are kept contiguous in the array by
convention"* — and a container makes it structural instead of hoped for.

**A constraint you cannot forget beats a check you must remember.** A rejected
option (a `sequence` label checked only by `validate`) scored *below doing nothing*
for two of the eight, who independently reached the same argument: an opt-in check
they might not tag, on a command they might not run, would replace real vigilance
with false confidence. Non-overlap is enforced here because it is what a track
means, not because someone remembered to ask for it.

**Anchoring was requested by all eight**, despite being the least pleasant thing in
the set to write — the polymorphic `layer` field ranked last on ease. They wanted it
anyway, because hand-maintaining `panel.layer = title.layer - 1` is an error they
said they would make repeatedly and silently. Four also found that a track-only
layer cannot stack two elements *within* one group without splitting the track,
"which then loses the free-overlap convenience the model is supposed to give you".
Hence the per-element override.

## Nesting without a local clock

ADR-0001's strongest argument was that **nesting implies a local clock whether or
not one exists** — JSON2Video's documented silent failure, where an element's
`start` is relative to its scene and a model computes it against the movie instead.

That argument is answered here, but only because it is answered *explicitly*:

**A track has no clock.** It has no start, no duration, and no origin. Its children
carry absolute times on the project's single timeline, identical to what they
carried in the flat list. There is nothing for a time to be relative *to*.

This is the difference between a track and a scene, and it is why one is adopted
and the other stays rejected. A scene owns a clock; a track owns a stacking
position. **The `scene` rejection in ADR-0001 stands unchanged.**

The risk is real and must be defended in the schema, not just in prose: a reader
who assumes array order means sequence will be wrong. One agent named exactly this
— a track array *"looks sequential"*, tempting an "eyeball confirmation" that is
invalid. The published schema and `montaget timeline` output must both make
absolute times unmissable.

## What this costs

**A second level to read.** "What is on screen at 6.2s" becomes a filter over two
nested arrays instead of one. Still a read, still no arithmetic, still answerable
inside the agent's turn — ADR-0001's driving requirement survives, which is the
condition on which this decision rests.

**Free overlap within a lane is gone.** Two elements that genuinely should overlap
now need two tracks. This is the constraint being bought deliberately, and it is
what the reference class does.

**Cross-track coupling has no home.** An image and its narration must move
together, and nothing says so. Raised independently during evaluation; it is what
`group` is for, and `group` survives ADR-0001 unchanged, but the interaction
between `group` and `track` is now an open question.

## Consequences

- **ADR-0001's "there is no track, no scene, and no container of any kind between
  the project and an element" is superseded.** Its **Considered options → Tracks**
  paragraph is superseded. Its **group-contiguity consequence** is moot: locality is
  structural now. Everything else in ADR-0001 — scenes, per-kind collections,
  clip-owned audio, the driving requirement — stands.
- **`track` leaves the rejected-terms list in `CONTEXT.md`** and becomes vocabulary.
  `scene`, `clip` and `asset` stay rejected.
- **`layer` moves to the track**, with an optional per-element override.
- **The overlap rule needs a validator**, and it must distinguish *overlap* from
  *gap*: a forgotten shift leaving a silent gap passes an overlap-only check. Raised
  during evaluation and not otherwise on the map.
- **A rejected option is recorded, not discarded:** a `sequence` label the renderer
  ignores, checked only by `validate`. It is rejected as *actively risky*, not
  merely redundant — see above.

## The finding this decision does not address

All eight agents, unprompted, named the same thing as the worst problem in every
candidate format: **inserting time and cascading the shift**. Every option leaves
the author to find each downstream element and hand-add the delta.

> *"the single most error-prone, most frequent edit in this whole exercise — and
> not one format offers relative/duration-chained timing to make that automatic.
> That's the real gap, bigger than the layer-anchoring or overlap-validation
> differences the four options otherwise compete on."*

This contradicts the research's own conclusion that ripple *"largely dissolves"*
because an agent can recompute trivially. It can. It does not do so reliably.
Capability was never the question.

Tracks narrow the search but do not perform the shift. ADR-0001 already deferred
this — *"whether that is solved by relative authoring materialised into absolute
times, or another way, is deferred to the time-model decision"* — and it now
arrives there as the **highest-priority** item rather than a loose end.
