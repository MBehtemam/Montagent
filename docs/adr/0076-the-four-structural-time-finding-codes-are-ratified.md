---
status: accepted
amends: 0004 (names `E-TRACK-OVERLAP` and `N-TRACK-GAP` as the two findings its overlap/gap split requires, ratifies `N-TRACK-GAP` as `note`-class, and specifies that a gap is bounded by its track's own elements, never by the project's ends), 0006 (ratifies the code, class and repair field of all four), 0020 (names `E-SPEED-MISMATCH` and `E-OVERRUN-UNNEEDED` as its invariant's two arms)
---

# The four structural-time finding codes are ratified: `E-TRACK-OVERLAP`, `N-TRACK-GAP`, `E-SPEED-MISMATCH`, `E-OVERRUN-UNNEEDED`

> **Amended by [ADR-0100](./0100-one-track-overlap-finding-per-track-carrying-a-census-of-its-knots.md)**,
> which changes how the `E-TRACK-OVERLAP` ratified below is **emitted**: once per track
> carrying a census of the overlapping elements, never once per overlapping pair. Fifteen
> elements on one track printed 105 findings for one authorial mistake, so the count that
> was meant to be *"the number of things to fix"* was not. The code, the `error` class and
> the refuse-class repair this ADR ratified are all unchanged; the registered template and
> field set are replaced, and the finding now names a track and no element — the shape
> `N-TRACK-GAP` already had.

**Ticket:** [#255](https://github.com/MBehtemam/Montagent/issues/255).

## The gap

[#197](https://github.com/MBehtemam/Montagent/issues/197) made ADR-0004's, ADR-0006's and
ADR-0020's structural time checks live and had to invent four stable codes to do it. Each
ADR states the *condition* the check tests; none states a code, a class, or a repair
class. `crates/montagent-core/src/registry.rs` argues all three from this repo's own
rules — ADR-0043 fixes the repair class per code, ADR-0006 makes a code the handle an
author suppresses and `compare` diffs on — but that argument, however correct, is the
implementation choosing surface. ADR-0031: the ADR series is the specification, and a
code is a published contract, so four of them existing only in a source file is a gap in
the spec rather than only in the docs.

This ADR closes that gap by ratifying what `registry.rs` already carries, and settles the
two points that were not forced by prior text.

## Decision

**The four codes, their class, and their repair class are ratified as shipped:**

| code | class | repair | ADR |
| --- | --- | --- | --- |
| `E-TRACK-OVERLAP` | `error` | refuse-class | ADR-0004 |
| `N-TRACK-GAP` | `note` | — (not `error`; ADR-0043 does not apply) | ADR-0004 |
| `E-SPEED-MISMATCH` | `error` | advise-class | ADR-0020 |
| `E-OVERRUN-UNNEEDED` | `error` | refuse-class | ADR-0020 |

`E-TRACK-OVERLAP` is refuse-class on the same reasoning ADR-0043 states for any check
where two documents produce the identical file but call for different repairs: which of
the two overlapping elements is misplaced is not readable off the document, and moving
either — or moving one to a second track — are three different edits with no way to
choose among them from the file alone.

`E-SPEED-MISMATCH` is advise-class because ADR-0020 already names the one free variable:
*"`start`/`end` and `source_start`/`source_end` are authoritative and must never move
silently to satisfy this check ... `speed` is the free variable."* One free variable is a
fully determined fix, exactly ADR-0043's advise-class test.

`E-OVERRUN-UNNEEDED` is refuse-class, not a second instance of `E-SPEED-MISMATCH`: the
element is too short for its own `overrun`, and the document does not say whether the
author meant the element to run longer or meant no `overrun` at all — deleting the key
and extending `end` are different videos, and nothing in the file picks between them.

### 1. `N-TRACK-GAP` is a `note`, not a `review`

Ratified as shipped. ADR-0006 computes a gap's severity *"from the consequence at an
instant,"* and the consequence at an instant is a fact about the whole frame — whether
anything on another track fills it. One track's own traversal cannot see that, so a
`review` from this check would be stating a measurement it did not take. The `review` for
a gap that nothing else covers belongs to the check that can see the whole frame:
`R-VISUAL-GAP` (ADR-0018, [#200](https://github.com/MBehtemam/Montagent/issues/200)).

One gap, two checks: `N-TRACK-GAP` reports the fact of the silence, uniformly, at `note`;
`R-VISUAL-GAP` reports whether the silence is also a visual dropout, at whatever severity
the coverage question earns. Neither check states the other's conclusion.

The committed fixture emits **27** of these notes across three tracks, which ADR-0006's
noise budget collapses to one counted line. Because that count is non-zero, the fixture
no longer validates against an empty finding list, and `tests/fixture.rs` was relaxed
accordingly, at `#197` — a consequence of shipping this behaviour, not a defect in it.

### 2. A gap is bounded by two elements of the same track, never by the project's own ends

Ratified as shipped. `Sequence::gaps` (`crates/montagent-core/src/track.rs`) opens the
first gap only after the track's first element and closes accounting at its last, tracking
the furthest instant the track has reached rather than reading it off the previous
element positionally — so an element nested wholly inside a longer one cannot manufacture
a gap that is not there. A track holding one element inside a longer project reports no
gap: the distance from its end to the project's own end is slack, not silence, because
nothing bounds it on that side as a *track* fact. `N-TRACK-GAP` is a claim about what is
missing *between two things this track placed*, and a track with only one element has
made no claim about what comes before or after it.

This is consistent with ADR-0004's own framing — *"a track has no clock. It has no start,
no duration, and no origin"* — extended to gaps: a track that never started has nothing to
be late relative to, and a track that never continues has nothing left uncovered.

## Why this ADR exists rather than a `registry.rs` comment

`registry.rs`'s comment already made this argument, correctly, and that is exactly the
problem ADR-0031 names: an argument that lives only in a source file is not consultable by
someone reading the ADR series, is not what `docs/agents/domain.md` means by treating a
contradiction as something to surface rather than silently decide, and is at the mercy of
whoever next edits that file to keep it accurate. Ratifying it here makes the four codes,
their classes, and the two settled questions above part of the spec `registry.rs`
implements, rather than a fact only `registry.rs` currently knows.

## Consequences

- **ADR-0004** gains an "Amended by" banner pointing here, naming `E-TRACK-OVERLAP` and
  `N-TRACK-GAP` as the two findings its *"must distinguish overlap from gap"* consequence
  requires.
- **ADR-0006** gains an "Amended by" banner pointing here, ratifying the code, class and
  repair field of all four.
- **ADR-0020** gains an "Amended by" banner pointing here, naming `E-SPEED-MISMATCH` and
  `E-OVERRUN-UNNEEDED` as its invariant's two arms.
- `registry.rs`'s `#255` comment above the four `CheckSpec` entries is replaced with a
  citation to this ADR; the code does not change — this ADR ratifies what `#197` shipped.
- No test changes: `tests/fixture.rs`'s 27-gap assertion and the one-counted-line assertion
  already reflect the decisions ratified here.
