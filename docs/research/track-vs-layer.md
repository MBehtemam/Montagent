# Track vs `layer`: re-testing the no-track decision against the CapCut/Premiere reference class

Research for [#20](https://github.com/MBehtemam/Montaget/issues/20). Companion to
[the declarative video API survey](./declarative-video-api-models.md), which covered the
*authoring API* side (Shotstack, Creatomate, JSON2Video, Editly). This file covers the gap:
the **desktop NLE / interchange** side — OpenTimelineIO, Premiere Pro, CapCut, DaVinci
Resolve, FCPXML — and the agent-authoring argument.

All claims cite a primary source inline. Anything not established from a primary source is
marked **NOT CONFIRMED**. Sources that could not be fetched are marked **NOT REACHED**.

Status: **in progress** — this file is committed incrementally as each source is read.

## 1. Summary of what was established

_(pending)_

## 2. Per-tool findings

### 2.1 OpenTimelineIO — `Track` vs `Stack`

OTIO is the strongest source available: an interchange schema, published as source, that
needed *both* a sequential container and an overlapping one and named them separately. The
reason it needed both is visible in ~10 lines of C++.

**A `Track` child has no start time. Its position is derived by summing what came before
it.** From `Track::range_of_child_at_index`
([src/opentimelineio/track.cpp](https://github.com/AcademySoftwareFoundation/OpenTimelineIO/blob/main/src/opentimelineio/track.cpp)):

```cpp
RationalTime start_time(0, child_duration.rate());
for (int i = 0; i < index; i++) {
    Composable* child2 = children()[i];
    if (!child2->overlapping()) {
        start_time += children()[i]->duration(error_status);
    }
}
```

This is the whole argument in one loop. A track is not a lane that things are *placed on*;
it is a list whose *order is the timing*. Answering "where does child 7 sit?" is an O(n)
accumulation over children 0..6 — arithmetic, not a read.

**A `Stack` child's start time is 0 — all children begin together.** From
`Stack::range_of_child_at_index`
([src/opentimelineio/stack.cpp](https://github.com/AcademySoftwareFoundation/OpenTimelineIO/blob/main/src/opentimelineio/stack.cpp)):

```cpp
return TimeRange(RationalTime(0, duration.rate()), duration);
```

and `available_range` takes the **max** of children's durations, not the sum. So `Stack` is
the parallel/overlapping composition and `Track` is the sequential one. **The two schema
names exist precisely to split "plays after" from "draws over".**

**Z-order comes from position in the `Stack`, bottom-first.** *"The layers in a stack are
iterated from the bottom (the first entry in the stack) towards the top (the final entry in
the stack). Images in a stack overlay lower images using an alpha composite operation."*
([Timeline Structure](https://opentimelineio.readthedocs.io/en/stable/tutorials/otio-timeline-structure.html))
Note this is the *opposite* direction to Shotstack's first-is-top (see the
[API survey](./declarative-video-api-models.md)) — three of the formats surveyed so far
disagree on which end of a list is in front, which is exactly the kind of thing a generator
gets backwards. Montaget's explicit integer `layer` sidesteps that class of error entirely.

**Overlap *within* a track exists, but only as a special case.** *"Within a track, clips may
overlap via a `transition`. In that case, the contribution of track is the linear blend of
the elements joined by the transition."* (Timeline Structure, above.) The mechanism is the
`overlapping()` flag skipped in the accumulation loop and a `transition->in_offset()`
subtraction. So OTIO does not permit free overlap in a track; it permits a bounded,
typed exception for cross-dissolves, and pays for it with a special case in the
position calculation.

**Because a track has no absolute times, empty space needs an object.** `Gap` exists to
occupy time so following items land where intended — *"a `Gap` is meant to be
transparent"*, and in the worked example *"the `Gap` in 'Track-001' is 4 frames long, and
the track below, 'Track-002', has frames 102-105 of 'Clip-003' aligned with the `Gap`
above."* This is a direct cost of the sequential model that an absolute-time model does not
pay: **to leave a hole you must write an object into the file, and to read what is on
screen you must know that an object is deliberately nothing.**

**Nesting is uniform:** *"By nesting a `Composition` (either `Track` or `Stack`) we can
refer to a `Composition` as though it was just another `Clip` in the outer
`Composition`."* A `Timeline` *"always has a top-level `Stack` object to hold its `Track`
children"* — i.e. the standard NLE picture is literally *stack-of-tracks*: overlap between
lanes, sequence within a lane.

**What this establishes for Montaget.** OTIO is evidence *for* ADR-0001's premise, not
against it: the format that thought hardest about this refused to make one container mean
both things. Montaget's `elements[] + layer` is the *Stack* half — parallel composition
with explicit stacking — with each element carrying its own absolute time range instead of
being aligned at 0. What Montaget does not have is the *Track* half, and OTIO shows exactly
what that half is: **ordinal timing, plus `Gap` objects, plus a transition special case.**

### 2.2 Adobe Premiere Pro

_(pending)_

### 2.3 CapCut / JianYing `draft_content.json`

_(pending)_

### 2.4 DaVinci Resolve scripting API

_(pending)_

### 2.5 FCPXML `spine` / `lane`

_(pending)_

## 3. The five questions from #20

_(pending)_

## 4. Options

Presented as options, not a verdict. The choice is the human's.

Every option below is scored on the same three things: **what it costs**, **what it buys an
LLM agent authoring this JSON with ordinary file tools** (Read/Edit/Write over a file in
git — *not* a human dragging with a mouse, and not necessarily an MCP call), and **exactly
which text in [`docs/adr/0001-flat-element-list.md`](../adr/0001-flat-element-list.md) would
have to be amended**.

The pivot for all of them is the OTIO mechanism in §2.1: a sequential container buys a
cheap *write* (insert one thing, everything after it re-times for free, because position
*is* order) by selling a cheap *read* (position becomes an O(n) prefix sum, plus `Gap`
objects, plus a typed exception for overlap). ADR-0001's driving requirement is a cheap
read. So the real question in every option is: **how much read-time arithmetic is it
willing to buy back, and what does it get for it?**

### Option (a) — keep `layer` + free overlap, unchanged

No format change. Absolute `start`/`end` on every element, integer `layer`, free overlap,
`group` as an inert label.

**What it costs.**

- No format help for the three operations in question 4 (§3). "Insert 2s at 0:12" rewrites
  every downstream `start` and `end`: an O(n) *edit*, and — the concrete cost — an O(n)
  *diff*. A change that is conceptually one insertion shows up in git as a change to every
  element after it, so review can't see at a glance that only one thing was intended.
- **No expression for sequencing intent, therefore no validation surface.** Two elements
  meant to be back-to-back on `layer: 0` that accidentally overlap by 0.3s after a bad
  edit are indistinguishable, to the format and to the validator, from two elements
  deliberately cross-fading. The failure is silent and visual. This is the same class of
  failure the prior survey found in Shotstack (*"the overlap rule fails as silent flicker
  rather than an error"*, [API survey](./declarative-video-api-models.md)) — except
  Shotstack at least has a rule to fail.
- `layer` is an unfamiliar word to a reader arriving from CapCut or Premiere, who will look
  for lanes. (This is a docs cost, not a format cost.)

**What it buys the agent.**

- "What is on screen at 6.2s" stays a single filter over a single array — no prefix sums,
  no `Gap` objects to recognise as deliberate nothing, no transition special case. The
  agent answers it inside its own turn, from the text, with no tool call.
- Every element's record is self-contained: its absolute time is *in it*. A targeted edit
  is a targeted edit; a partial read of the file still yields true facts.
- Z-order is an explicit integer, so the first-is-top/last-is-top coin-flip that the two
  surveys together found four different answers to (OTIO bottom-first, Shotstack
  first-is-top, Creatomate `z_index` with its non-z-indexed sibling trap) is not a class of
  error that can occur here.

**ADR text to amend: none.** The decision stands as written. The only edits are supporting:
ADR-0001's Considered-options paragraph asserts *"A survey of four shipping declarative
video APIs found that no product uses a track as a pure stacking lane"* and CONTEXT.md's
rejected-term entry generalises that to *"Premiere, Resolve, OpenTimelineIO, Shotstack,
Creatomate"*. §2.1 confirms the OTIO half of that claim outright and gives it a better
citation than it currently has; the Premiere/Resolve/CapCut half should be re-worded to
match whatever §2.2–§2.4 establish rather than asserted.

### Option (b) — `layer` plus an opt-in sequencing affordance

The interesting middle, and the one with the most internal variation. "Opt-in sequencing in
an absolute-time format" is not one design; it is at least four, and they differ sharply on
whether the read cost comes back.

#### (b1) Sequencing as an *operation*, not a field — materialised into absolute times

Montaget grows an editing operation (an MCP tool, or a documented script) — `insert_at`,
`ripple`, `swap` — that rewrites absolute times and hands back a normal project file. The
document's shape never changes. There is no new field, no new noun, nothing new to read.

- **Cost:** two authoring paths that must agree. The stated target agent edits JSON with
  ordinary file tools; it gets no help unless it chooses to call the tool, and an agent
  that hand-edits after a ripple can produce something the tool would not have produced.
  Also, the tool has to guess scope — see the sync problem in §3 question 3: rippling
  "everything after 0:12" and rippling "everything after 0:12 on layer 0" are different
  operations and the format cannot say which was meant. Whichever it picks is wrong half
  the time, so the scope has to be an argument, which means the agent has to have thought
  about it — which is most of the work.
- **Buys:** the write cost drops to one call; the diff is still O(n) but is now
  machine-generated and uniform, which is easier to review than hand-arithmetic. The read
  cost stays exactly zero. Nothing about the inert-data principle is touched.
- **ADR text:** the Consequences bullet *"Ordinary cuts — 'insert a shot at 0:12 and shift
  everything after' — get no help from the format. Whether that is solved by relative
  authoring materialised into absolute times, or another way, is deferred to the time-model
  decision."* This option is that deferral resolving to **"solved outside the format, by an
  operation"**. The bullet would be rewritten to say so, and to name the scope-argument
  problem. Nothing else in the ADR changes; no container is introduced, so *"There is no
  track, no scene, and no container of any kind between the project and an element"*
  survives verbatim.

#### (b2) Relative timing in the document — `start: null`, or `{"after": "elem-7"}`

The Creatomate design: an element may omit its start and be sequenced after the previous
element sharing its track (see [API survey](./declarative-video-api-models.md)).

**This is the variant that reintroduces the O(n) read cost, and it reintroduces something
worse than arithmetic.** Under OTIO's `Track` the position of child 7 is at least a
*mechanical* sum of six durations sitting right there in order. Under a reference-based
relative start, the position of an element is the head of a dependency chain that must be
resolved — and if the referent is itself relative, resolved transitively. The document
stops being readable and starts being *evaluable*, which is precisely what CONTEXT.md's
opening sentence rules out: a project *"states, by being read, what is on screen at any
given moment"*.

- **Cost:** the driving requirement of ADR-0001 is gone. There are now two ways to say when
  something starts, so every reader must handle both, and every generator must choose. The
  prior survey already has the vendor's own verdict on this exact design: Creatomate's docs
  call the duration cascade *"the number-one source of broken renders"* and warn that
  smaller models *"produce broken RenderScript without warning"* — from the vendor with the
  most explicitly LLM-facing documentation of the four. That is close to a controlled
  experiment on the question this ticket asks, and it came out badly.
- **Buys:** genuinely short authoring for the common case — "these five clips, back to back"
  needs no arithmetic at write time at all, and stays correct when one of them changes
  duration. This is a real benefit and it is why Creatomate did it.
- A **bounded variant** — relative starts permitted only one level deep, only against an
  element that carries an absolute start — caps the read at O(1) per element and kills the
  transitivity problem. It still costs: two ways to express one thing, and a reader still
  doing arithmetic to answer the 6.2s question.
- **ADR text:** the driving-requirement paragraph — *"an agent must be able to answer 'what
  is on screen at 6.2s' by reading, with no arithmetic and no evaluation. Flat and
  absolute, that question is a single filter over a single array — a check the agent can run
  inside its own turn, without a tool call."* — must be weakened or deleted. That paragraph
  is the ADR's entire justification, so amending it is not an amendment; it is a new
  decision. CONTEXT.md's **Project** and **Element** entries would also need the second
  timing form added.

#### (b3) Declared sequencing intent that the renderer ignores — a lint, not a constraint

Elements may carry a free-text sequencing name, exactly parallel to `group`: the renderer
ignores it completely, times stay absolute, but Montaget's validator warns when two
elements sharing that name overlap in time or leave an unintended hole.

This is the only variant that adds *sequencing* without adding *sequencing semantics*.
Nothing is computed from it; it is a machine-checkable assertion about times that are
already fully written down.

- **Cost:** a fourth vocabulary word next to `group`, and the two are confusable — `group`
  means "these belong together, probably simultaneous", the new one means "these belong
  together, strictly one after another". Overloading `group` itself is cheaper in
  vocabulary but wrong in meaning: ADR-0001's own example of a group is *"every element of
  one vocabulary item"*, which is simultaneous by construction. There is also a real risk
  the field decays into decoration nobody maintains, since nothing breaks if it is absent
  or stale.
- **Buys:** the agent gets to *state* that a run of elements is meant to be contiguous, and
  then gets told when an edit broke it — converting §3's silent-overlap failure into a
  caught one, at zero read cost. It also gives the ripple operation of (b1) its missing
  scope argument: "ripple this sequence" is well-defined in a way "ripple layer 0" is not.
  And it costs nothing to ignore: a project that never uses it reads exactly as it does
  today.
- **ADR text:** the opening paragraph, *"Every element … carries the same `type`, time
  range, `layer` and optional `group`"* — the new optional field joins that list. A new
  Consequences bullet is needed stating that the field is inert and advisory. Everything
  else stands: no container is added, times stay absolute, the driving requirement is
  untouched. This is the smallest amendment of any option that changes the format at all.

#### (b4) Convention and recipes only — no format change, no tool

Document the idiom ("contiguous runs on one layer, kept adjacent in the array") and leave it
there. Costs nothing, buys nothing that (a) does not already have, and is listed only
because it is the honest floor of what "opt-in affordance" can mean.

### Option (c) — something genuinely track-shaped

#### (c1) Full OTIO-style tracks: order *is* timing

`tracks[]`, each an ordered list of children; a child has no start time; holes are `Gap`
objects; in-lane overlap is a typed transition or is impossible.

- **Cost:** everything §2.1 documents, paid in full. Position becomes an O(n) prefix sum, so
  ADR-0001's driving requirement is abandoned outright. Empty space stops being empty and
  becomes an object that a reader must recognise as deliberate nothing. Overlap becomes a
  special case with its own type. Two or three new nouns (`track`, `gap`, and probably
  `transition`) enter a vocabulary that currently has six. And **an element's absolute time
  is no longer written on the element**, so a partial read of the file no longer yields true
  facts — which is a specific, sharp loss for an agent that reads files in windows.
  ADR-0001's own fixture is the stress case: *"one still is on screen for 14.5s with six
  unrelated audio events under it, three belonging to no visual at all"* — under (c1) each
  of those events needs a lane, and the silences between them need `Gap` objects.
- **Buys:** the one-line insertion diff, structural impossibility of accidental overlap,
  familiarity to a reader who pictures CapCut, and free ripple *within a lane* (with the
  cross-lane sync caveat from §3 question 3 — which is not a caveat the format solves, it
  is one every track-based NLE solves in its UI, and Montaget has no UI to solve it in).
- **ADR text:** effectively a rewrite, and the new ADR would supersede rather than amend.
  The **title** (*"A project is a flat list of uniform elements, not tracks or scenes"*),
  the **opening paragraph** in full, the **driving-requirement paragraph**, and the
  **Considered options → Tracks** paragraph all invert. In CONTEXT.md, the **Layer** entry
  (whose *Avoid* list literally contains "track") and the whole **Track** rejected-term
  entry are deleted. Note one live tension: ADR-0001's argument against *scenes* is *"nesting
  implies a local clock whether or not one exists. You cannot put elements inside a
  container and then ask readers not to infer that their times are relative to it."* A
  `Track` is a container **whose children's times genuinely are relative to it**, so that
  argument does not merely fail to block (c1) — it becomes an argument that (c1) is at least
  *honest* where scenes were a trap. Whoever writes the superseding ADR has to deal with
  that sentence either way.

#### (c2) Tracks as pure z-lanes: containers whose children keep absolute times

`tracks[]` exists, children still carry absolute `start`/`end`, and the track is a named
bucket plus a stacking position — no sequencing constraint at all.

- **Cost:** this is the shape the prior survey found **no shipping product uses** — every
  track-like concept in Shotstack, Creatomate, Editly and JSON2Video carries a non-overlap
  constraint, and §2.1 adds OTIO to that list (OTIO calls the unconstrained one a `Stack`,
  not a `Track`). So the word would promise sequencing to every reader arriving from any of
  them and not deliver it, which is ADR-0001's stated objection, unchanged and now better
  evidenced. Calling it `lane` or keeping `layer` avoids the false promise but then it is
  barely (c) at all.
- **Buys:** structural locality. ADR-0001 currently recovers this by convention —
  *"Elements sharing a `group` are kept contiguous in the array by convention. This is a
  formatting rule only; nothing depends on it"* — and a container would make it structural.
  It also gives per-lane properties (mute, hide, lane-wide opacity) somewhere to live, which
  the flat model has no home for. Read cost is unchanged: absolute times stay on elements.
- **ADR text:** the sentence *"There is no track, no scene, and no container of any kind
  between the project and an element"* is directly contradicted and must go. The
  **Considered options → Tracks** paragraph must be rewritten, since its argument is
  specifically about the word, not the structure. The **group-contiguity Consequences
  bullet** becomes moot. The driving-requirement paragraph survives — the 6.2s question
  becomes a filter over two nested arrays instead of one, which is still a read.

### The axis these sit on

| | read cost for "what's on screen at 6.2s" | write cost of "insert 2s at 0:12" | new nouns |
|---|---|---|---|
| (a) | filter one array | rewrite N elements, O(n) diff | 0 |
| (b1) | filter one array | one operation call, O(n) diff | 0 |
| (b2) | resolve a dependency chain | rewrite 1 element | 0–1 |
| (b3) | filter one array | rewrite N elements, O(n) diff, **validated** | 1 (inert) |
| (b4) | filter one array | rewrite N elements, O(n) diff | 0 |
| (c1) | prefix-sum per lane, skip gaps | insert 1 child | 2–3 |
| (c2) | filter two nested arrays | rewrite N elements, O(n) diff | 1 |

The table is the argument in miniature: **only (b2) and (c1) buy the cheap write, and both
pay for it in the read.** Everything else is a choice about vocabulary, validation and where
the editing logic lives — not about the time model at all.
