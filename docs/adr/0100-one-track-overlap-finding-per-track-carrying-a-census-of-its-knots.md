---
status: accepted
amends: 0076 (its ratified `E-TRACK-OVERLAP` is now emitted once per track carrying a census, not once per overlapping pair; its registered template and field set are replaced, and the code, class and refuse-class repair it ratified are all unchanged), 0004 (the finding its "must distinguish overlap from gap" consequence requires reports the offending *set*, not a pair: one per track, located at the file and the track like `N-TRACK-GAP` and naming no element), 0006 (the sibling census becomes the mechanism by which one authorial mistake produces one finding, which is this ADR's reading of the noise budget as a safety property rather than a preference), 0043 (`E-TRACK-OVERLAP` acquires the census that ADR-0043 says a refuse-class finding carries instead of a repair; it had a `repair: "none"` and no census, which is half the rule)
---

# One `E-TRACK-OVERLAP` per track, carrying a census of its knots

**Ticket:** [#389](https://github.com/MBehtemam/Montagent/issues/389) (MONTAGENT-5), applying
[#384](https://github.com/MBehtemam/Montagent/issues/384)'s ruling 4.

## The gap

Per-pair emission is quadratic in a single authorial mistake. Fifteen elements written onto
one track — *"these fifteen are on one track and must not be"*, one sentence of a fix —
produced **105** `E-TRACK-OVERLAP` findings across a **38,739-character** report, measured on
this repo's own test harness. The first real end-to-end build through this tool hit it and
logged it as MONTAGENT-5.

**The source doc and #389 say 106, and 105 is the number.** `15 × 14 / 2` is 105 exactly, so
the extra one came from a second overlap elsewhere in that project rather than from the fifteen
— which is the defect restating itself, since a report of 106 pairwise sentences gave its reader
no way to see that 105 of them were one mistake and one of them was another.

The defect is not volume for its own sake. It is that **the count lied.** `checks/track.rs`
said so in its own words:

> Each pair is reported once, which is what makes the count the number of things to fix.

For fifteen elements the number of things to fix is one, and the report said 105. ADR-0006
makes the noise budget a safety property precisely because *"47 printed alignment lines are
how a reader learns to skip the output"* — and 105 sentences, each naming two elements the
reader must fix neither of on its own, is that failure with a worse ratio.

There is a second, quieter gap. ADR-0043 says a refuse-class finding **carries a sibling
census** — that is what it has *instead of* a repair, since it declines to state one.
`E-TRACK-OVERLAP` is refuse-class (ADR-0076 ratified it so) and carried `repair: "none"` and
**no census at all**, which is half of ADR-0043 implemented. The two gaps have one fix.

## Decision

### 1. One finding per track, carrying a census of the overlapping elements

Replacing the per-pair emission, per #384 ruling 4 verbatim: *"One `E-TRACK-OVERLAP` per track
carrying a census of the overlapping elements."*

The finding is located at the **file and the track**, and names **no element** — the shape
`N-TRACK-GAP` already has, and for the same reason: it is a fact about a track, and any
element it singled out would be one the document gives no grounds for singling out.

### 2. The census groups by the stretch of track a knot contends for

A census groups members by *one observable, document-derived value they share* (ADR-0006,
ADR-0043). What co-offending elements share is the **stretch of track they contend for**, so
the census field is `contended_stretch` and each group's value is a knot's half-open extent,
`"{from}..{to}"`, from its earliest `start` to its latest `end`.

A **knot** is a connected component of the overlap relation, not a clique: `a` overlapping `b`
and `b` overlapping `c` is **one** knot even where `a` and `c` are disjoint. Splitting it would
put the same `b` in two groups — and `b`, the element in the middle, is the likeliest single
element to be the misplaced one, which is exactly the inference a census may not invite.

**A group is therefore a knot an author untangles as a unit, and the group count is the number
of things to fix that the pair count only claimed to be.** That is what makes this a census
rather than a count with a preamble: the distribution is the actionable content.

The alternative considered and rejected was grouping all of a track's offenders into **one**
group. It is the reading #384's words also bear, and it is degenerate: what every offender on a
track shares is *the track*, which is already a field on the finding, so the census would
restate the location and distribute nothing. `checks/runs.rs` states the same objection where it
declines to emit a finding at all because its *"census would have one group"* — the test being
whether the grouping **can** distribute, not whether a particular project makes it. MONTAGENT-5's
own fifteen elements are one knot and so do produce a single group; a second knot elsewhere on
that track would produce a second, and the one-group-by-construction reading could not.

### 3. Groups are ordered by the clock, never by size

First member's `start` ascending. ADR-0043 is explicit that a census *"must not be worded in a
way that implies the larger group is the correct one"*, and **sorting by size is that wording
written into the ordering** — it is the same claim the prose would have made, moved somewhere
the prose rule does not reach.

### 4. Nothing names a first offending pair

The source doc proposes naming *"the first offending pair"*. #384 refused it and this ADR
implements the refusal: a pointer at one pair out of a knot of fifteen is a ranking of the
members, which ADR-0043 forbids for the reason the gravity experiment established — agents that
*found* the fork still guessed, and what stopped them guessing was being handed the
distribution. **The reader gets the set, not a pointer.**

### 5. `overlap` survives as contended track time, by depth sweep

The per-pair finding's `overlap` field — the one number it measured — survives as the **total
milliseconds of the track covered by more than one element**, summed over the track instead of
over a pair. On two elements it is the number it always reported, so the field did not change
meaning for the case that motivated it.

It is computed by a **depth sweep and not a sum of pairwise intersections**: three elements over
one 1000 ms stretch contend for 1000 ms of track, not 3000. Summing intersections would have
reproduced the same quadratic double-count inside a single number, which is the original defect
with the sentences removed.

### 6. Knots are found by `N-TRACK-GAP`'s own traversal, and no pair is enumerated

The per-pair check had to enumerate pairs, because a pair *was* the finding, and it needed a
sweep rather than a scan of neighbours: `[0,10000]`, `[1000,2000]`, `[3000,4000]` sorted by
`start` has one overlapping adjacent pair and two overlaps, and the one a neighbour scan misses
is the first element against the third.

**A knot needs no pairs at all.** Overlap over half-open ranges is an interval-graph relation,
so a knot is a **run** of the sorted spans — each starting before the furthest instant the run
has reached — which is the identical `covered_to` bookkeeping `Sequence::gaps` already does one
predicate away. ADR-0004's two halves, *"distinguish overlap from gap"*, become visibly the same
walk with `<` where the other has `>`. The traversal is **linear** in a track's elements where
the pair enumeration was quadratic, so the collapse bounds the work and not only the report.

The correctness obligation moves rather than disappearing: reading the previous element's `end`
instead of `covered_to` would split `[3000,4000]` out of the knot `[0,10000]` holds it in, which
before this ADR cost a sentence and now **loses a member of the census and of the count**. The
committed test that held the sweep is restated to assert membership.

### 7. What this ADR does not do

- **It does not import [#419](https://github.com/MBehtemam/Montagent/issues/419)'s prose
  collapse, and is not an instance of it.** #388 ruling 3 settled that `census` and the
  repetition collapse are **two mechanisms**, on the test *does one document fact produce many
  findings, or do many document facts share a code?* One mistake on one track is the census
  case. The two are independent: nothing here reads a threshold and nothing in ADR-0099 reads a
  census.
- **It does not touch `review`-class findings, `N-TRACK-GAP`, or any other code.** #384 §5's
  scope, and the gap collapses to one counted line by ADR-0006's existing rule already.
- **It does not make the shared census renderer name its members.** `crate::text` prints
  `census <field>: <n> at <value>` for *every* census-bearing finding, counts only, and the
  member set lives in the canonical JSON — which is the form ADR-0006 makes canonical and
  generates the text from. Naming members in the text form is a change to every census in the
  tool, it reintroduces an `O(elements)` line, and it is not this ticket's. **Stated as a
  known limit:** a text-only reader gets the track, the count, the knot count and each knot's
  extent, and must open the canonical JSON or the document to read the fifteen names. The
  extents are the narrowing ADR-0043 asks a census to provide.

## Consequences

- **`E-TRACK-OVERLAP`'s field set is replaced. This is a breaking change to machine-readable
  output**, and this is its version note — the form ADR-0093 and ADR-0080 use, there being no
  changelog or output-version constant in this repo to bump. `element`, `other`, `start`, `end`,
  `other_start` and `other_end` are **gone**; `count`, `sets` and a `census` are new; `overlap`
  and `track` remain, `overlap` re-based from a pair to the track. A consumer reading
  `fields.element` off this code now reads nothing, and one counting findings to count defects
  now gets a smaller and truer number.
- **The finding carries no element location**, so a consumer grouping findings by element loses
  this code from those groups. It gains it under the track, beside `N-TRACK-GAP`.
- **The code's grain is now a track, not a pair.** ADR-0006 makes a finding code the stable
  handle, so anything that ever keys on this one — a suppression, a findings diff, an agent's
  own bookkeeping — addresses a track at a time. That is the coarser handle and the correct
  one, since a pair was never separately fixable. Neither a suppression surface nor a
  findings-level diff exists in the tool today (`compare` computes its own drift predicates and
  does not read `validate`'s findings), so this is a statement about the contract and not a
  behaviour change anywhere else.
- **The report proper goes 38,739 → 790 characters on the fifteen-element case, and is flat in
  the number of overlapping elements thereafter** — flat in ADR-0099's sense and not in
  byte-identity's: the surviving counts print their own digits, so characters may move by
  `O(log elements)` and **the line count may not move at all**. That is the asserted measure.
- **The check's own work drops from quadratic to linear** in a track's elements, because a
  knot is a run of the sorted spans and no pair is enumerated. The collapse therefore bounds
  the work and not only the report — which is more than ADR-0099's collapse could claim, and
  is a property of this condition rather than of the mechanism.
- **No probe sidecar change and no re-probe.** This is a report-shape change only; nothing here
  reads the disk.

## Evidence

Re-executable: `cargo test -p montagent-core --test time`.

- `fifteen_elements_on_one_track_are_one_finding_and_not_a_hundred_and_five` — MONTAGENT-5's own
  case: one finding, `count = 15`, a census of fifteen members, and a report under a thousand
  characters where it was 38,739.
- `the_collapsed_finding_is_flat_in_the_number_of_overlapping_elements` — 15 against 45
  overlapping elements, asserting **identical line count**. This is the acceptance test stated
  as ADR-0099 had to restate its own: byte-identity would fail correctly, because the counts
  print their own digits.
- `a_chain_of_overlaps_is_one_knot_and_not_two` — ruling 2's connected-component rule, on the
  case that distinguishes it from a clique, with `overlap = 200` asserting ruling 5's depth
  sweep against the pairwise sum that would say 300.
- `two_knots_on_one_track_are_two_census_groups_in_time_order` — ruling 3, on a project whose
  **larger knot is second on the clock**, so a size sort would visibly reorder them and the
  test would catch it.
- `two_elements_of_one_track_sharing_an_instant_is_an_error` — the pre-existing test, restated:
  no element location, both members in one census group, and `overlap` still **1000**, which is
  ruling 5's continuity claim.
- `an_overlap_between_two_elements_that_are_not_adjacent_in_time_order_is_found` — the
  pre-existing sweep test, restated as ruling 6 requires: three members in one group, where a
  traversal reading the previous element's `end` rather than `covered_to` reports two.
