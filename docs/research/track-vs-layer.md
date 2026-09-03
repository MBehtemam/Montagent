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

_(pending)_
