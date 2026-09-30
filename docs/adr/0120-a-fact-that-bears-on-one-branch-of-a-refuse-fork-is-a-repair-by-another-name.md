---
status: accepted
amends: 0043 (states what a refuse-class finding may not carry besides a `repair` value — a fact that bears on only one branch of its fork, or names something the author could adopt — and that a census partitioning the siblings without ranking them is not such a fact), 0111 (its §6 left `E-FONT-NO-GLYPH`'s census-shaped field open; the field is dropped, so the registry's one exception to *"every code that carries a census declares a mode"* is gone)
---

# A fact that bears on one branch of a refuse fork is a repair by another name

**Ticket:** [#498](https://github.com/MBehtemam/Montagent/issues/498), building
[#495](https://github.com/MBehtemam/Montagent/issues/495)'s
[rulings](https://github.com/MBehtemam/Montagent/issues/495#issuecomment-5906293886), on the
map [#383](https://github.com/MBehtemam/Montagent/issues/383).

## What was there

`E-FONT-NO-GLYPH` is refuse-class. A text element's font chain maps none of some characters
it uses, so they render as `.notdef`, and the fix forks: **the chain is short a file**, or
**the text carries a character it was never meant to** (a stray paste, a wrong-language string).
Nothing in the document says which. ADR-0043 is exactly this case.

The finding also attached a `census` field. It had one group per *other declared font key*
whose chain maps every missing character, and each group's members were the same list of
missing codepoints. It was omitted when no chain qualified, citing ADR-0058. In the text form it
printed `census font: brand+fa (4)`, in the place where every other census counts elements,
but here the 4 counted codepoints. [#427](https://github.com/MBehtemam/Montagent/issues/427)
ruling 6 found it partitions nothing and so is not a census (ADR-0111 §6), and left open
whether it should become an honestly named field or be dropped.

## Decision

### 1. The list is dropped, not renamed

A field such as `covering_fonts: ["brand+fa"]` would give it an honest name without changing
what it does. It speaks to one branch only, *the chain is short a file*, and makes that branch
a menu: *move the run to `brand+fa`*, *add that file to this chain*. The other branch,
*the text is wrong*, gets nothing. That asymmetry is the point. A reader choosing between two
fixes is given help with one of them.

ADR-0043's evidence is why "true and inert" does not save it. In the glossary experiment,
agents given an authoritative, correct account of the old semantics solved the fork **0 of
3**, while agents given the bare error solved it **2 of 3**. And 3 of the 4 agents who noticed
the gravity fork shipped the guessed repair anyway. The harm was never falsehood. It was the
tool volunteering the material for one answer.

Whether the field appeared was itself a signal (the court's first juror): present meant *the
easy branch is available*. Dropping the field removes that too.

### 2. The rule, in its narrow form

> **A fact that bears on only one branch of a refuse fork, or names something the author could
> adopt, is a repair by another name, and a refuse-class finding does not carry it. A census
> that partitions the siblings without ranking them is not one.**

The ticket first worded it as *"information that informs one branch"*. That is **too broad**:
every census informs a fork. ADR-0043's own gravity census (*6 elements share one geometry, 2
share another*) makes one branch more plausible, and it is the pattern ADR-0043 prescribes.
What separates the two is that a census partitions the affected siblings by an observable fact
and says nothing about the fix. Its groups bear on every branch at once. A list of candidate
fonts partitions no siblings and bears on one branch.

**How "names something the author could adopt" is read here** (§5 depends on it): a concrete
candidate the document would accept — a value, a key, a file, a font — and not a statement of
the *kinds* of edit that close every branch. `E-LAYER-TIE`'s *"give either element its own
integer `layer`, or an anchor naming the other"* names every branch and no candidate, so it
passes.

### 3. No replacement census, and the one this code could carry

The finding now carries no census, which is `E-RUN-SPLIT-CLUSTER`'s position: the census
attaches where a real sibling group exists. The candidate the ticket named, *other elements
short of the same characters*, gives one group and partitions nothing. It would also duplicate
ADR-0099's collapse, which already counts a code's instances.

**One admissible census is recorded and not built** (the court's third juror): the elements
set in the finding's font key, split into *fully mapped* and *carrying unmapped characters*,
unranked. It bears on both branches. *"Five of six `brand` elements map cleanly"* leans towards
a stray character, and *"all six are short the same letters"* leans towards a missing file.
Nothing yet shows it is needed.

### 4. ADR-0058 does not bear on it

`elsewhere()` cited ADR-0058 for omitting the field when empty, and the ticket asked whether
ADR-0058's rule settles the question. It does not. ADR-0058 governs how **absence** is written:
omit it, never *"no match found"*, because the negative implies a hypothesis was tested. It
was ruled on a `note` whose fact distinguishes two populations. It says nothing about whether
the positive may be stated on a refuse-class finding.

### 5. The audit: five other refuse-class codes conflict with the rule

The ticket asked for every refuse-class code to be checked against §2, with anything found named
and left for its own ticket. Read against their message templates:

| Code | What it carries | Against §2 |
| --- | --- | --- |
| `E-FONT-BLOCKLISTED` | *"Known open substitutes, none of them metric-compatible: {substitutes}"* | **Conflicts.** Candidate fonts the author could adopt: the same shape as the list this ADR drops. |
| `E-RETIRED-KEY` (refuse variant) | *"the format now says this with {replacement}"* | **Conflicts** with the second disjunct. ADR-0016 and ADR-0068 made naming the replacement a message-text property on purpose, and ADR-0043's own gravity experiment was run on such a message. |
| `E-SCHEMA-UNKNOWN-KEY` | *"Here the format publishes {expected}"* | **Conflicts** with both disjuncts. It names keys to adopt, which bears on the *typo* branch, while the same message raises the *newer format* branch. |
| `E-SHIFT-STRADDLE` | *"The nearest legal boundaries are {start} and {end}"* | **Conflicts** with the second disjunct. Concrete values, bearing on *move the shift point* and not on *re-cut the element*. |
| `E-SHIFT-SLACK` | *"Release it explicitly with `release: [[{from}, {to}]]` if that is intended"* | **Conflicts.** It names the adoptable value for the *intended* branch only. It is also `shift`'s consent mechanism, so it may be the case the rule has to exempt rather than the message to change. |
| `E-LAYER-TIE`, `E-OUTPUT-FOREIGN`, `R-OUTPUT-UNATTESTED`, `E-RUN-SPLIT-CLUSTER` | an instruction | **Pass.** Each names every branch and no candidate. |
| every other refuse-class code | the fault only | **Pass.** |

**None is changed here.** Whether each message changes or the rule narrows is a separate
question for each code, and `E-RETIRED-KEY`'s answer reaches back into ADR-0016. The five are
left for their own ticket, [#504](https://github.com/MBehtemam/Montagent/issues/504). Until it is decided, this ADR's rule governs **new** refuse-class
findings and `E-FONT-NO-GLYPH`, and does not yet claim the five.

## Consequences

- **Breaking: `E-FONT-NO-GLYPH`'s JSON loses `census`.** A consumer reading
  `findings[].census.groups[].value` for this code gets nothing. There is no changelog to bump.
  This note is the version record, as in ADR-0100. Its other fields (`element`, `font`,
  `characters`, `chain`), its template and `repair: "none"` are unchanged.
- **The text-form miscount goes with it.** `(4)` no longer sits in a slot that counts elements.
- **The registry has no census exception.** `tests/registry.rs` asserts that every code attaching
  a census declares a mode, with no special case, and that `E-FONT-NO-GLYPH` attaches none.
- **Stated cost:** a reader on the *short a file* branch no longer learns from the finding which
  other chain covers the characters. They learn it by adding a file and re-running `validate`,
  or with a font tool. That cost falls on the branch that should bear it.
- **`CONTEXT.md`**'s **Repair** entry carries §2's rule.

## Evidence

A three-juror court (Claude Opus 5.5, Sonnet 5.5, Fable 5.1), blind to each other, voted the
same way on all three questions: drop, no replacement census, a new ADR. Two refinements came
from it and were ratified by the human: the narrow wording of §2 (juror 1) and the admissible
census of §3 (juror 3). Ballots are in [#495](https://github.com/MBehtemam/Montagent/issues/495).
`tests/fonts.rs`'s
`a_missing_glyph_names_no_other_chain_even_where_one_would_draw_it` declares a second chain that
does map the characters, and asserts that the finding carries no census in the JSON or in its
text block. It fails against the previous `checks/fonts.rs`.
