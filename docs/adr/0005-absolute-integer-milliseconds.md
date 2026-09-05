---
status: accepted
---

# Time is absolute integer milliseconds, and structural edits belong to a tool

> **Amended by [ADR-0012](./0012-flat-transform-keyframes-carried-by-their-element.md).**
> The time model below stands. **`shift`'s defining sentence does not**: *"moves every time
> at or after `at`"* is wrong for keyframe times, and a reader should not implement it. A
> keyframe's `t` is not a timeline time — `shift` moves elements and their keyframes are
> carried, so an element entirely before `at` keeps keyframes numerically after it, and an
> element entirely after `at` moves keyframes numerically before it. Applying the sentence
> literally changes a shot that finished 397 ms **before** the edit point by 8.90 px of
> framing. ADR-0012 carries the complete case table, the SPLIT rule for straddlers, and the
> rounding rule this ADR left unpublished for `speed`.


Every element carries `start` and `end` as **integer milliseconds** on the
project's single absolute timeline. The interval is **half-open** — `[start, end)`
— so an element whose `end` is 7500 is not on screen at 7500 and its neighbour
starting at 7500 is.

There is no relative or sequential authoring mode, and no materialised second
representation of the same fact.

Because absolute times make structural edits O(n), **Montaget supplies a `shift`
tool** and the two decisions stand or fall together.

## Why absolute, and why the pairing is not optional

[ADR-0004](./0004-tracks-as-constrained-lanes.md) already chose absolute times as
part of adopting tracks; this ADR settles the rest of the time model and re-tests
that choice rather than inheriting it.

Three agents authored a real project and performed four edits on it. The result
splits cleanly and reproduces the finding from
[#20](https://github.com/MBehtemam/Montaget/issues/20):

**Reading is free.** All three answered "what is on screen at 6.2s" correctly and
immediately, by comparison, with no arithmetic. This is the operation the whole
format exists to make cheap, and absolute time is the only model that delivers it.
Under sequential storage the same question is a fold over the track — the
in-your-head evaluation the inert-data principle forbids. As one put it, relative
storage does not remove the expression language, it *moves it into the reader*.

**Writing is lethal.** All three broke the file on the *simplest* edit — "narration
#3 is now 0.8s longer" — by doing exactly the edit as stated, `end` and
`source_end`, and shipping an overlap. Repairing it correctly cost **eight elements
and fifteen hand-changed numbers**. The mechanism, named independently: *the edit
that expresses the intent is local; the edit that keeps the file legal is global.*

All three then attached the same condition to their vote, unprompted: absolute-only
is defensible **only** if a tool performs structural edits. "Absolute time without a
shift tool is a bad design" — stated after demonstrating it, not before.

**A materialised dual representation is rejected outright.** Every edit all three
made was an exact-string replace on an absolute number. A file holding both
representations acquires a stale half the first time that happens, and no reader
can tell which half the renderer used.

## Why integer milliseconds

Float seconds is the readable option and it failed in practice: two agents produced
`4.400000000000002` and `14.832999999999998` in their own runs, and both validators
needed epsilon comparisons — which is where subtle validator bugs live. Exact
adjacency (`end == next.start`) is the predicate the overlap and gap rules are built
on, and it should be decided by the type, not by every tool remembering to round.

The dissenting vote preferred 3-decimal seconds for readability and then named this
same fallback: *"I'd rather read `6200` than debug `0.30000000000000004`."*

Frames were rejected because they couple every number in the document to the
project's framerate — a 30→24 rebase rewrites the file. Premiere-style ticks
(254016000000/sec) are exact and illegible, which forfeits the read-the-file
property the design is built on.

**Accepted cost, to be stated in the schema rather than left as folklore:**
milliseconds do not divide evenly into a frame at 30fps (33.333ms), so a time can
name an instant between frames. The renderer must publish its rounding rule, and
`validate` should note boundaries that are not frame-aligned.

## Why `start` + `end`, not `start` + `duration`

`duration` is genuinely better for writing — one agent measured its insert at 8
numbers instead of 17, and a shift could never invert an element. It was rejected
anyway, unanimously, on the same grounds as the storage model: every read
(what is at T, does this overlap, is there a gap) becomes an addition *per element*.
Reads dominate and reads must stay arithmetic-free.

Storing all three is rejected for the reason the dual representation is: a
redundant field that hand-edits desynchronise.

## `shift`

`shift(path, at, delta, scope)` moves every time at or after `at` by `delta`. It
takes a timestamp and an offset — never a field name, never an element id — so it
satisfies the standing write-tool invariant.

**`scope` defaults to the whole project.** All three agents independently named a
single-track shift as their worst silent bug: two produced a project where the
narration outran the last still, one by 0.4s and one by 0.5s.

**Straddling elements are handled by type**, because a single rule is wrong for at
least one kind:

- **Time-invariant elements** (image, text, shape) have no internal clock. `shift`
  **stretches** the straddler's `end` by `delta`. This was the hand-fix two agents
  had to apply to close a three-second hole they had punched in the stills track.
- **Time-based elements** (audio, video) are **refused**. Stretching speech violates
  the source-range invariant — one agent's "obvious" uniform-stretch policy produced
  an element with an 8.1s timeline range against a 5.2s source, an invalid file.
  Moving it whole is also wrong: it relocates speech that had already begun.

The refusal must name the straddling elements and the nearest legal boundaries.
This is the decision's most valuable output, not its friction: **all three agents
found that "insert 3s at 0:12" was ill-posed**, because 12.0 fell mid-sentence, and
none of them could tell from the file. Two shipped the pause 3.3 seconds late. The
message they needed and never got is *"12.0s is 1.5s into line-03.mp3; nearest
boundaries are 10.1 and 13.5."*

## Gaps, and what the time model cannot catch

**Gaps are legal and are never errors.** Deliberate silence between narration lines
is a gap; so is a forgotten shift. Making them errors trains the reader to ignore
the validator, and five or six intentional gaps per short project is the normal
case.

**They must be reported as a distinct informational category, with size, sorted by
size.** For two agents a single gap line was the *only* signal that an edit had
punched black frames into the video — errors reported clean. Severity should follow
what the track renders: a gap in a visual track is black frames on screen and is
near-error; silence in an audio track is information.

**The time model cannot catch the worst bug the exercise produced.** All three
agents independently shipped a project whose narration outran its last visual, and
in all three cases it survived every check they ran. It is neither a gap nor an
overlap — both are within-track predicates — but a divergence *between* tracks, to
which a per-track validator is structurally blind. That is
[#23](https://github.com/MBehtemam/Montaget/issues/23), not this decision.

## Timeline range against the source

Two checks, deliberately separated, because they need different things:

1. **Internal consistency**, decidable by reading the element alone with no I/O:
   for a time-based element, `end - start` must equal `source_end - source_start`.
2. **The probe check**, requiring I/O and reported by `validate` and `render` as its
   own class: `source_end` must not exceed the file's actual duration, and the error
   states the duration it found.

A mismatch is an error **unless the element declares its intent** with an explicit
`fill` (`hold` / `loop`) or `speed`. Implicit filling and implicit speed-change are
rejected on the inert-data principle, in its strongest form yet: to know what is on
screen you would need the source file's duration, *which is not in the document at
all*. As one agent put it, that is worse than an expression language, which at least
ships its own inputs.

**Requiring `probe` before every write is rejected.** All three agents pushed back
on it; one called it backwards. `validate` reads the media and reports the real
duration, which delivers the fact at the moment it is needed instead of costing
eleven speculative probe calls before a line is written. Note this is precisely the
scenario the exercise's first edit simulated — in a real session nobody announces
that a file got 0.8s longer, so **comparing declared ranges against media on disk is
the highest-value check in the tool surface**, and the one no amount of careful
reading substitutes for.

## Consequences

- `validate` is **part of the edit loop, not a final check**. Three of four edits in
  the exercise produced defects only `validate` could see.
- **Elements are written sorted by `start` within a track, and formatted one element
  per line with stable key order.** The semantics stay order-free per ADR-0004; this
  is a writing convention. It is load-bearing because agents edit by exact-string
  replace — unstable formatting means a replace can silently hit two elements.
- **Time-invariant elements have no source range**, which is why `shift` may stretch
  them safely.
- The **half-open convention must be published in the schema.** Adjacent cuts are the
  normal case and every validator written during the exercise had to guess.
- **`--from`/`--to` partial render** inherits the half-open convention.
- Explicit `speed`, `hold` and `loop` are named here as fields but their vocabulary
  is not settled by this ADR.

## The exercise

Three agents, one brief, independently authored a project of 4 images and 6
narration elements over ~30s on 2–3 tracks, then performed four edits. Scores on
first attempt: **0/3 on the re-record, 0/3 clean on the insert, 1/3 on the
cross-track restack, 3/3 on the read.** Two of the three had their working
directory overwritten by a concurrent respondent and redid the exercise in
isolation; one saw a fragment of another's file before stopping. The third worked in
the contested directory throughout. Their answers diverge where independent work
diverges, but the collision is recorded rather than glossed.
