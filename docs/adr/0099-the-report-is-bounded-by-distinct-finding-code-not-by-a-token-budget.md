---
status: accepted
amends: 0006 (its collapse rule gains a second, independent bound — repetition, on top of inertness — so a
  non-inert class no longer prints every instance of every code; and its noise budget is given a growth order
  rather than a line count), 0011 (the report a verb answers with is bounded in the same way on every surface,
  and `render`'s verb block stops enumerating its elements; the `json: true` MCP parameter is recorded as a
  known open defect this ADR does not fix)
---

# The report is bounded by distinct finding code, not by a token budget

[#419](https://github.com/MBehtemam/Montagent/issues/419), on the map
[#383](https://github.com/MBehtemam/Montagent/issues/383). It ships
[#388](https://github.com/MBehtemam/Montagent/issues/388)'s rulings, whose reasoning and
measurements are in
[its resolution](https://github.com/MBehtemam/Montagent/issues/388#issuecomment-5868811887).

The reported failure was an MCP tool call returning
`Error: result (237,326 characters...) exceeds maximum allowed tokens`, on a project of 217
elements. Every ruling below is observable behaviour, so ADR-0031 makes it a spec gap until
ratified here.

## Decision

**Report size is O(distinct finding codes), not O(elements). The bound is a complexity
order, not a token budget, and it is stated with its hole.**

1. **A finding code prints in full up to N = 3 instances and collapses to one counted line
   beyond it** — `<class>  <code>  <count> — expand with --verbose`, the mechanism ADR-0006
   already specified for the inert classes. **N is unmeasured**; see §4.
2. **This is a second, independent bound, on top of the class rule.** `Class::prints_in_full`
   is unchanged: `note`, `unchecked` and `layout` still collapse at one instance because they
   are **inert**, and the new rule collapses any code past N because it is **repetitive**.
   Notes do not start expanding in small projects.
3. **A collapsed code prints no instance at all**, not the first three and an "and N more".
4. **`render`'s verb block is bounded on the same rule.** Its `audio` and `painted` lines
   stop naming every element past N, and its `not mixed` / `in part` / `not painted` rows
   collapse per code. **Every count and every total stays, at every verbosity.** `preview`
   shares the block and moves with it.
5. **`--verbose` restores every collapsed instance**, and the one-line summary keeps the
   exact counts unconditionally.
6. **Nothing else.** No `--min-severity`, no summary mode, no change to the canonical JSON,
   and no `census` added to any per-element check.

### The hole, stated rather than implied

**`CACHE` is O(sources) and does not collapse, so the MCP token error can still recur.**
ADR-0006 and ADR-0011 make that block the sole mechanism announcing a source that changed on
disk, and it is neither collapsible nor flag-gated. On the reference project it is **4,876 of
the 5,448 characters that remain** — 90% of the bounded report. The `sources` and `fonts`
lines of `render`'s block are O(sources) and O(fonts) for the same reason and are likewise
untouched. `frame`'s block is still O(elements at its instant); #388 did not rule on it and
neither does this ADR.

A bound that oversold itself would be the failure this map exists to fix, so the claim is
exactly: **the report does not grow with the project's element count.** Not *"the report
fits"* — see §1.

## Why

### 1. A token budget is not a statement this tool can make

The cap in the error message is **the MCP client's own**. Montagent cannot observe it, cannot
be told it, and it varies per client. So *"make the report fit"* has no denominator, while
*"report size does not grow with element count"* has one and is testable.

This is [ADR-0095](0095-the-sheets-budget-is-served-tile-width-and-overflow-refuses.md) §1
applied rather than re-derived — **a denominator has to vary with the thing it bounds** — and
a per-client constant does not vary with the project.

The one-line summary stays the exact-count channel, which is what keeps ADR-0006's *"output
may be filtered; analysis may not"* intact: the collapse hides no finding's **existence** and
no finding's **count**, only its per-instance prose.

### 2. The mechanism #388 was filed against was not the one at work

The ticket said the caption checks *"enumerat[e] every element [they] cover"*. They do not:
`checks/caption.rs` is strictly per-element and carries no census. The real split, measured on
the 22,885-character report:

| section | lines | chars |
| --- | --- | --- |
| summary | 2 | 80 |
| `CACHE` | 55 | 3,233 |
| **`review` findings (98)** | **197** | **19,027 (84%)** |
| `note` findings (301) | 4 | 142 |
| `NOT CHECKED` | 5 | 202 |

**301 notes cost 142 characters and 98 reviews cost 19,027**, because `review` sits on the
wrong side of one predicate. The lever was right; the reading of why was wrong.

*(That report is also **not reproducible**: the named project fires zero `R-CAPTION-PACE` and
zero `R-CAPTION-MIN-DURATION` today. ADR-0006's own amendment block records the same hole for
[#154](https://github.com/MBehtemam/Montagent/issues/154).)*

### 3. The severity floor is refused

A `--min-severity` floor at `error` changes **neither** the growth order, **nor** the `CACHE`
block, **nor** `render`'s ~140 verb-block lines — it does not solve the reported symptom. And
it would print `0 errors` over **72 genuine `R-CAPTION-NO-AUDIO` defects**, which is
ADR-0004/ADR-0006's *"false confidence"* exactly, and the very failure the ticket reports
suffering: *"the line was in the report, and the report was not readable."* A flag that makes
the tool say less while the exit code stays 0 is the `sequence` label wearing a new name.

### 4. N = 3, and no measurement backs it

Stated plainly because it is true: **every N from 1 to 25 produces a byte-identical report**
on the only real project available, because its codes fire at 1, 26, 72, 86 and 214
instances. On a synthetic project carrying 290 caption findings, N = 3 and N = 10 both give
3,910 characters. **3 and 10 are empirically indistinguishable on all available evidence**, so
N was chosen on principle:

1. The candidate reason for a larger N — *"roughly one screen, the unit a reader takes in
   without scrolling"* — **does not apply to this reader.** The consumer is an agent over
   MCP. It has no screen. The unit is wrong, so the number derived from it carries no weight.
2. N bounds the worst case multiplicatively: at ~40 registered codes, N = 10 admits 400 full
   findings where N = 3 admits 120. Where nothing distinguishes them, the tighter is free.
3. At 10 the claim is that nine near-identical sentences earn their place. At 3 the claim is
   that a reader needs to see the prose **vary** and no more. Only the second is defensible.

A first-K sample is rejected on separate ground: the printed set would be whichever instances
the timeline reached first, which is **not a ranking and must not read as one**. *(Not on
ADR-0043's first-offender prohibition, which an earlier draft of #388's analysis reached for:
timeline order is not a ranking, so ADR-0043 does not bite there. The weaker ground alone.)*

### 5. `census` and the prose collapse are two mechanisms

The test: **does one document fact produce many findings, or do many document facts share a
code?**

- **One fact, many findings → `census`.** It changes the finding set and the canonical JSON.
  Fifteen elements on one track is *one* authorial mistake; #384 ruling 4 settled this.
- **Many facts, one code → prose collapse.** It changes only the generated text. Seventy-two
  captions with no audio under them are **72 independent facts**, each with its own window.

A census over those 72 would have to assert a grouping value they do not share — fabricating
a document fact in the canonical JSON, which ADR-0043 forbids — and would erase the
per-caption window that is the finding's repair-relevant content.

**One edge, named so it is not relitigated:** 72 no-audio captions may in practice stem from
*one* omission — no audio anywhere — which would look like the census case. If so the answer
is a **new check detecting that document fact**, never a census bolted onto a per-element
check.

### 6. Why `render`'s block is not "the entire point of the call"

`Class::prints_in_full` keeps `Drift` printing in full on the reasoning that a `compare` fact
**is** the answer: collapse it and the caller has nothing left from the verb it ran. The same
sentence does not cover `render`'s block, and the difference is what the verb answers with.

`compare` answers with its findings and nothing else. `render` answers with **a file**, and
this block is a caption to it (ADR-0011). Every number in the caption survives — the path,
duration, frame count, bytes, and the counts under `audio`, `painted` and each reason group —
so an agent can still tell *"not there"* from *"not drawn"* and *"silent"* from *"not
mixed"*, which is the discrimination the block exists to provide. What goes is the list of
ids under a count that is still printed.

It is in scope for the second reason too, and it is the load-bearing one: **leave the block
O(elements) and §1's bound is false the day it ships**, which ADR-0031 will not let anything
ratify. At 217 elements with 134 not mixed it was ~140 lines.

### 7. The canonical JSON stays whole, and `json: true` is a known open defect

`--json` on the reference project is **171,706 characters** — already past any realistic
client cap. But *canonical* means lossless: filtering it would create a second, lossy
canonical form and break every consumer that assumes completeness, `compare` among them.
ADR-0006 assigns wholeness to the JSON and filtering to its consumer, and that holds.

What is wrong is the **MCP `json: true` parameter**, a tool call returning a form its own
transport cannot carry. **It is not this ADR's, on evidence:** the failing calls reported
2,086 lines at **114 characters per line**, and pretty-printed JSON runs **22** (171,706 over
7,760). The reporter was on the **default text path**, which this ADR fixes. The `json: true`
defect pre-dates every caption finding and is
[#420](https://github.com/MBehtemam/Montagent/issues/420) — **recorded here as open, so it is
not read as closed by this one.**

## Measured effect

`montagent validate episodes/job-interview/lang/da/project.json` in the
`youtube_language_learning` repo — 217 elements, 98 reviews, 301 notes — in characters:

| | before | after |
| --- | --- | --- |
| **the report** (warm probe cache) | 19,481 chars / 207 lines | **572 chars / 13 lines** |
| with a cold `CACHE` block | 24,357 chars / 262 lines | 5,448 chars / 68 lines |

**A 34× reduction on the report proper, and flat in element count thereafter.** The residue
is the hole: `CACHE` is 4,876 characters either way, untouched and unbounded in principle.

#388 projected 3,858 characters. That figure decomposes as ~625 of report plus a 3,233-character
`CACHE` block measured with shorter paths; the report half lands at **572**, so the model was
right to within about 50 characters and the difference in the totals is path lengths on the
machine each was measured on.

**This measurement cannot be a committed check** — the project lives in another repository —
so it will rot exactly as #154's did, and §2 above says as much about #388's own numbers. The
claim that does not rot is in
[`crates/montagent-core/tests/report_bound.rs`](../../crates/montagent-core/tests/report_bound.rs):
a report over 100 elements and one over 10,000 have **the same line count**, and differ only
by the digits of the counts that survive. That is the falsifiable form of §1, and the
`CACHE` hole is asserted there too, so no later reader mistakes the bound for a promise the
tool does not make.
