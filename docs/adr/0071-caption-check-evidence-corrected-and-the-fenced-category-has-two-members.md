---
status: accepted
amends: 0034 (both of its "on the fixture" paragraphs are corrected against its own
  mechanics — the pace floor fires once and not twice, and the repeat check finds three
  disagreements and not one; neither check's rule changes), 0061 (its census of the fenced
  category — "`R-CAPTION-PACE` is the only one" — is superseded: `R-CAPTION-MIN-DURATION`
  is a second member, admitted by the same test, and the registry is made the live answer)
---

# What the caption checks actually fire on: ADR-0034's evidence corrected, and ADR-0061's fenced category has two members

[#199](https://github.com/MBehtemam/Montaget/issues/199) implemented the four
`R-CAPTION-*` checks against the mechanics
[ADR-0034](./0034-caption-pace-and-repeat-duration-checks.md) and
[ADR-0054](./0054-caption-audio-backing-and-minimum-duration-checks.md) state, and three
of the ADR series' own claims about them turned out not to reproduce. None of the three is
a rule: two are evidence paragraphs describing what the checks do to the committed
fixture, and the third is a count of how many checks borrow a threshold. All three are the
kind of claim a reader takes on trust precisely because it looks like reporting rather
than deciding, which is why they are corrected here rather than left to be rediscovered by
the next implementer. Raised as [#259](https://github.com/MBehtemam/Montaget/issues/259)
and [#260](https://github.com/MBehtemam/Montaget/issues/260).

**Every number below is re-derived by `caption_check_scan.py`**, beside this file, which
asserts both directions of each claim and exits non-zero the moment any of them stops
holding.

## 1. ADR-0034's evidence table counts the `\n` its metric excludes

ADR-0034's **Metric** paragraph is unambiguous: characters-per-second over grapheme
clusters, *"spaces included, `\n` excluded"*. Its evidence table, two paragraphs above,
counts `hook-05` at 31 characters and `quiz-question` at 46 — the counts **including** the
break. The table's counts are what its *"On the fixture"* paragraph then reports:

| caption | the table | as the **Metric** paragraph states it |
| --- | --- | --- |
| `hook-loop` | 31 chars → 25.8 cps, fires | 30 chars → **25.0 cps**, fires |
| `quiz-question` | 46 chars → 20.4 cps, fires | 45 chars → **19.91 cps**, silent |

So *"two of five captions fire"* is **one**, and the second — the ADR's own *"marginal but
over the line"* case — is under the line by 0.09 cps.

**The metric governs and the table is corrected**, for the reason
[ADR-0061](./0061-validate-judgment-boundary-threshold-provenance-and-a-fenced-exception.md)
gives about this very check: the finding's substance is *"the raw measured fact"*, and a
measurement is defined by the rule that produces it, not by a worked example of it. The
opposite resolution — count the `\n`, keep the table — would make a caption's pace depend
on how the *agent* chose to break its lines (ADR-0008 gives line breaks to the agent), so
rewrapping a caption at the same size and duration would change its measured reading
speed. That is not a fact about readers, which is the only thing the 20 cps threshold is
calibrated against.

**`quiz-question` at 19.91 cps is not a gap this ADR closes.** It sits 0.4% under a
threshold ADR-0034 already describes as borrowed and approximate, and ADR-0034's own
posture — *"the error this ships with is under-flagging, never over-flagging, which is the
acceptable direction for a `review`-level check"* — covers it exactly. Moving the
threshold to catch it would be tuning a published external constant against one element of
one fixture, which is the practice ADR-0061's citation rule exists to make impossible.

## 2. ADR-0034's repeat check finds three disagreements, not one

ADR-0034's trigger groups byte-identical text *"project-wide — no dependency on `track` or
`group`"*, and ADR-0054 ratifies that scope explicitly: the checks *"carry no track-name
restriction anywhere in their mechanics"*. Its *"On the fixture"* paragraph reports **one**
finding. The fixture has three:

| text | occurrences | durations |
| --- | --- | --- |
| `"What is this called\nin English?"` | `hook-05`, `hook-loop` | 2298 ms, 1200 ms |
| `"I hang cobwebs over the door."` | `sentence-05`, `sentence-quiz` | 7004 ms, 2900 ms |
| `"cobweb  -  cobweb"` | `word-05`, `word-quiz` | 12156 ms, 2900 ms |

**No scope reproduces "one".** The obvious explanation — that the ADR was reading the
`caption` track, where its worked examples come from — does not survive the scan either:
`word-05` and `word-quiz` are both in `caption`, so even that reading gives two. The count
was an undercount, not a narrower question.

The two the ADR does not name are the same defect class as the one it does: a line held
for twelve seconds as a teaching card and for under three as a quiz callback. Whether
either is deliberate is exactly what the check refuses to say (ADR-0034's symmetry rule),
and both are now reported, symmetrically, naming every value.

**The must-not-fire half stands unchanged and is asserted by the same scan:**
`word-08-target` and `word-08-bridge` carry `"string of lights"` across two different
tracks at identical durations, and are silent. A check that grouped on anything but text —
or that had no tolerance — would report them.

## 3. `R-CAPTION-MIN-DURATION` is the fenced category's second member

ADR-0061 states: *"Applied to every check `validate` currently runs, `R-CAPTION-PACE` is
the only one that fails [the fact-only test]."* That was true when it was written.
ADR-0054's 834 ms floor, implemented in #199, fails the same test: strip 834 and the check
has no predicate left, and 834 is nowhere in any document and not entailed by any
rendering rule — it is 5/6 of a second from Netflix's *General Requirements* page, rounded
up to the stricter integer millisecond. ADR-0054 says so itself, placing the floor *"in
the same register as `R-CAPTION-PACE`'s 20 cps: an externally documented constant about
human reading capacity, not a property of the render"*.

So it is admitted under the fence rather than refused, and it satisfies all three
conditions as written: `review`, the measured duration as the finding's substance, and the
source cited inline in the finding. **This is ADR-0061 working, not being bent** — it
called citation *"binding policy for future checks of this shape, not best-effort"*, and
this is the first check of that shape to arrive since.

**What is corrected is the count, and how it is kept.** A census of the fenced category in
ADR-0061's prose was accurate for exactly one check and goes stale silently — nothing
fails when it does, which is how it survived #199's implementation unremarked. The live
answer is `crate::registry`, where every check declares its `ThresholdProvenance`, the
declaration is what the code reads, and `tests/registry.rs` already asserts the fence's
three conditions over *every* `External` member rather than over a named one:

```rust
for spec in registry::all() {
    if let ThresholdProvenance::External { source, adr } = spec.threshold { ... }
}
```

ADR-0061's test stands unchanged; **"which checks carry a borrowed judgment?" is answered
by the registry, not by counting in prose.** A future check of this shape needs no
amendment to ADR-0061 — it needs an `External` declaration, which the existing test then
holds to the fence.

## An enforcement gap this found

`check_amendment_banners.py` read only the **first line** of a YAML `amends:` header. Both
of ADR-0061's amendment edges are declared on continuation lines, so its amendment of
ADR-0034 was invisible to the tool that exists to make amendments discoverable: ADR-0034's
banner did not name ADR-0061, `README.md`'s row did not list it, and the check reported
`OK`. Fixed here, in the script, and both now name it.

This is the same failure ADR-0067 diagnosed and this script was written to prevent —
*"the rule was already written; what was missing was anything that enforced it"* — one
layer down: the enforcement was written, and a regex silently exempted the longest
headers, which are the ones on the ADRs that amend the most.

## Consequences

- **No check changes.** ADR-0034's and ADR-0054's mechanics are what #199 implemented and
  what this ADR confirms; the two *"On the fixture"* paragraphs are evidence, and evidence
  that disagrees with the mechanic it illustrates is corrected to the mechanic.
- **The committed fixture carries ten `review` findings**, and this is not a defect in the
  checks. Spec [#168](https://github.com/MBehtemam/Montaget/issues/168) makes a check that
  fires on the fixture wrong *"unless an ADR says otherwise"*; ADR-0034 and ADR-0054 were
  both written **from** this file's defects, so they say otherwise about all ten.
  `crates/montaget-core/tests/fixture.rs` names each one.
- **ADR-0061's fenced category has two members and is no longer counted in prose.** The
  registry is the live answer and the existing registry test is the enforcement.
- **`caption_check_scan.py` joins the two existing hand-run checks** under `docs/adr/`.
  All three are registered in CI, and CI is disabled for this repository
  ([#249](https://github.com/MBehtemam/Montaget/issues/249)), so all three are checks a
  reader runs.
- **`check_amendment_banners.py` reads multi-line `amends:` headers.** Running it before
  this ADR reported `OK` on a series with a missing edge.
