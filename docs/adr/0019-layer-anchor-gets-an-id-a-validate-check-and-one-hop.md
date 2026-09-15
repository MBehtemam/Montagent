---
status: accepted
amends: 0004 (the anchor's target namespace, chaining and hardening), 0006 (two new validate checks), 0011 (clarifies the write-tool invariant's scope)
---

# The layer anchor names an element's `id`, resolves in one hop, and `validate` checks both

[ADR-0004](./0004-tracks-as-constrained-lanes.md) gave an element the option of
stating its stacking relative to another: `{"below": "title"}`. It never said
what `"title"` names, whether the reference can chain, or what happens when it
is wrong. All three agents in [#8](https://github.com/MBehtemam/Montaget/issues/8)'s
editing exercise wrote a defective anchor and nothing complained. This ADR closes
those gaps.

## The anchor stays

[#20](https://github.com/MBehtemam/Montaget/issues/20)'s eight agents asked for
anchoring unanimously, despite ranking it worst to write — hand-maintaining
`panel.layer = title.layer - 1` is an error they said they would make repeatedly
and silently whenever the referenced element moves. One agent dissented after
hitting the inert-anchor defect firsthand, arguing that resolving a reference is
exactly the "evaluate it in your head" cost the inert-data principle exists to
prevent. A three-juror court (GitHub Copilot's auto-selected model, Qwen2 7B and
Llama 3.2, independent, blind to each other) was put the same question framed
from the acting agent's side and split 2–1 to keep it, on the same grounds as
#20: the alternative reintroduces exactly the hand-maintained arithmetic
anchoring was adopted to remove. **Kept, hardened below.**

## An anchor names an element's `id`, and `id` is now a formal field

Elements have always carried an `id` string informally — every worked example
since [ADR-0012](./0012-flat-transform-keyframes-carried-by-their-element.md)
and all 60 elements in the committed fixture use one — but no accepted document
defines it as a canonical field. That let a real ambiguity through: one of
#8's three agents could not tell whether `"title"` in an anchor named an
element or the track named `titles`, because tracks also carry a string name
([ADR-0004](./0004-tracks-as-constrained-lanes.md)).

**`id` is now a required, unique field on every element**, in the same shape
regardless of type. An anchor's `below`/`above` string resolves against this
namespace only — never a track name. A court on this question (same three
jurors) went 2–1 for element-only resolution with a formal `id`; the dissent
proposed anchors name tracks instead, which is incompatible with the evidence
that motivated this ADR — every failing case in #8 was one *element* anchored
to another *specific element*, not to a whole track.

This changes zero bytes of the committed fixture: every element already
carries an `id`, and none currently uses the anchor form.

## One hop, no chain

An anchor's target must itself resolve to a plain integer — its own override or
its track's layer — never to another anchor. Chaining was raised as a live risk
when anchoring was first proposed
([`track-vs-layer.md`](../research/track-vs-layer.md) §b5: unbounded chains need
cycle detection and turn "what's on top at 6.2s" into a graph walk instead of a
lookup) and never settled. The court went 2–1 for one hop only, on exactly that
argument: a direct relationship keeps the file readable by inspection; walking a
chain (the dissent's position) buys flexibility no evidence in this project has
asked for, at the cost of the flat-read requirement every prior decision on this
map has protected.

`validate` enforces this structurally and cheaply: an anchor's target's own
`layer` must not itself be an object. No walk, no possibility of a cycle by
construction.

## `validate` gains two checks

Both classified under [ADR-0006](./0006-validate-reports-facts-and-render-enforces.md)'s
existing three-level scheme — `error` ("the render is refused or is guaranteed
wrong"), `review` ("legal, renders, and you must look at a frame to know if it
was meant"), `note` ("a fact you may want and will not act on today"):

- **Anchor target does not exist, or is itself an anchor** → `error`. The
  z-order relationship cannot be resolved at all; this is no different from any
  other dangling reference the schema can't catch statically.
- **Anchor target exists but its time range never overlaps the anchored
  element's** → `review`. Legal — z-order only has consequences where two
  elements' time ranges overlap — but the anchor is then a permanent no-op for
  that element's entire lifetime, which the ticket that opened this ADR called
  "the hardest class of error to catch by reading." A silent no-op that looks
  like a completed edit is exactly what `review` exists for.

The court was unanimous in substance on both (one ballot mislabeled its own
vote letter but described this exact split in its reasoning — recorded as a
reliability caveat on small local models used as jurors, not as a dissent).

## The write-tool invariant does not forbid this

[ADR-0011](./0011-tool-surface-reads-checks-renders.md) states the invariant
precisely: *"A tool that writes may only take a complete element, as a
schema-shaped object. No tool takes a field name or an element id."* That
sentence is about **tool call arguments**. It says nothing about what an
element's own fields may contain, and an anchor's `{"below": "title"}` is data
on an element, never an argument to a write tool — `add_element` still takes
the complete element, anchor field included. The ticket that opened this ADR
read a tension here that does not exist; this ADR records the reading so it is
not rediscovered.

## Consequences

- `CONTEXT.md`'s **Element** entry gains `id` as a required field; a new **Id**
  entry defines its namespace and scope. The **Anchor** entry is updated for the
  one-hop rule and the `id`-only namespace.
- [ADR-0004](./0004-tracks-as-constrained-lanes.md) gets a pointer at its title
  noting this ADR settles the anchor's target namespace, chaining and
  validation, which it left open.
- [ADR-0011](./0011-tool-surface-reads-checks-renders.md) gets a pointer
  confirming the write-tool invariant's scope, since this is the second time
  (after the ticket itself) that sentence has been read as broader than
  written.
- Nothing renders yet, so none of this has been exercised against a frame —
  recorded rather than glossed, in the same spirit as prior ADRs on this map.

Evidence: [`docs/research/juries/layer-anchor/`](../research/juries/layer-anchor/README.md),
full ballots from both court rounds.
