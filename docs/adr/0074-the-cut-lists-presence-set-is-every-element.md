---
status: accepted
amends: 0011 (the cut list's presence set is **every** element, audio included — not only the on-screen ones; and `query --from --to` names the elements the document does not place on the clock rather than dropping them)
---

# The cut list's presence set is every element — "on-screen" was ADR-0011 writing too fast

> **Amended by [ADR-0094](./0094-the-sheets-instants-are-visual-states-sampled-at-the-first-painted-frame.md)**,
> which names `frame`'s contact-sheet range mode as the caller the `type` filter below
> anticipated, specifies the **re-merge** of intervals that become identical once audio is
> filtered out (the filter alone changes nothing — the merge is what turns 46 intervals into
> ~18), and turns the 28-boundary loss measured below into a **mandatory disclosure**: a sheet
> that narrows to the visual view must name the audio-only boundaries it dropped. Nothing in
> this ADR's own decision changes.

**Ticket:** [#250](https://github.com/MBehtemam/Montagent/issues/250). Evidence:
`docs/adr/cut_presence_scan.py`, which re-derives every measured number below — the element
counts, the boundary table, and the worked case's asymmetry — from the fixture, and exits
non-zero the moment one stops reproducing. The one number below that the scan does *not*
cover is labelled where it appears: the fixture has no unplaced elements, so `unplaced`'s
illustration is hypothetical by construction.

## The conflict

[ADR-0011](0011-tool-surface-reads-checks-renders.md) specifies the second `query` mode as

> **`query --from <a> --to <b>`** — the **cut list**: the intervals over which the set of
> **on-screen** elements is constant.

[#196](https://github.com/MBehtemam/Montagent/issues/196) built it over **every** element,
audio included, stating each member's `type` so that a caller wanting only the visual cut
list filters one field. #196's own rule was *"where this ticket and an ADR disagree, the
ADR wins and this ticket is wrong"*, so the implementation could not settle this by having
shipped; `crates/montagent-core/src/verbs/query/cuts.rs` carried the departure as a module
comment pointing at #250, and this ADR is that pointer's destination.

The word doing the damage is *on-screen*, and its cost is not stylistic: `presence set` is
a defined term in `CONTEXT.md`, used by both `query` modes, and under ADR-0011's wording it
would mean one thing in `--at` and a narrower thing in `--from --to`.

## Decision

**The cut list's presence set is every element the document places on the clock, audio
included.** ADR-0011's *"on-screen"* is amended to *"the presence set"*, the term
`CONTEXT.md` already defines, and the departure #196 shipped is ratified rather than
reverted.

Each member states its `type`, so a caller that wants the visual cut list filters one
field of an answer it already has. No flag, no second mode, and no `--visual`: the
narrowing is a filter over the returned members, not a different question to ask.

**This is the term, not one mode's reading of it.** `query --at`'s resolved stack is the
presence set at one instant and carries audio for the identical reason; ADR-0011's `--at`
line never said *on-screen*, so nothing there is being amended, but the two modes are now
explicitly answering about the same population. A presence set that meant one thing in
`--at` and another in `--from --to` is the outcome this ADR exists to prevent.

**`unplaced` is specified here too**, on both modes. An element the document does not place
on the clock — no `start`, no `end`, or one of them written as something other than whole
milliseconds — is **named in the answer**, not silently dropped. It cannot be in any
interval or any stack, because it has none to be in; naming it is what stops the omission
from being invisible.

## Why

### ADR-0001 already decided the population

[ADR-0001](0001-flat-element-list.md) is explicit that *"audio is an element like any
other; nothing owns it"*. The flat element list is the population every verb draws from,
and which lane an element sits in is a constraint on it
([ADR-0004](0004-tracks-as-constrained-lanes.md)), never a filter on whether it exists. A
verb that dropped a third of the fixture's elements from *"the presence set"* would be the
one place in Montagent where that phrase means something narrower than everywhere else — and
the reader who would be misled is the one who did the right thing and learned the term
first.

A third is measured, not rhetorical: **20 of the fixture's 60 elements are audio**,
exactly.

### The information only travels one way, and the fixture says how far

A caller given every element can compute the visual cut list. A caller given the visual one
cannot recover where the narration started, because those instants are not in the answer to
be recovered from.

Over `en-halloween-decorating`:

| presence set | boundaries |
| --- | --- |
| every element | **47** |
| visual only | **19** |
| reachable only through audio | **28** |

So the visual-only answer is not a coarser view of the same clock — **the majority of the
fixture's boundaries are not in it at all**. The scan asserts that direction explicitly, and
also asserts the converse never appears: dropping audio may only ever remove a boundary, so
a visual-only boundary absent from the full set would mean the two answers disagree about
the document rather than one containing the other.

And the defect ADR-0011's own consumer task hunts — *"an 800 ms drift on disk"*, one of the
four real tasks all ten sessions performed before being allowed to opine — is a
relationship between a **narration** boundary and a **photo** boundary. ADR-0011's own
worked version of that drift is unambiguous about where it lands:

> 19 of 20 elements at/after 30603 moved +800 ms; `vo-quiz-answer` did not.

`vo-quiz-answer` is `type: audio`, at 61116–63300 in the fixture. Under an on-screen-only
presence set the element the scenario is *about* is not in the answer at all — and the
shape of what is lost is worth being exact about, because it is not symmetric. Its **start**
survives: 61116 is also where `card-quiz`, `sentence-quiz` and `word-quiz` begin, so a
visual cut list still cuts there. Its **end** does not: nothing visual turns over at 63300,
so the visual answer shows one interval running 61116→64016 and cannot say that the
narration stopped 716 ms before the card did.

That is the whole failure mode in one instant. The visual cut list does not report a
narration boundary as missing; it reports an interval it believes is constant, and is
wrong about, with nothing in the output to suggest otherwise.

### Why not narrow the implementation and record the argument as refused

That was the other arm #250 offered, and it fails on cost asymmetry. The narrow answer is
recoverable from the wide one by reading one field; the wide one is not recoverable from the
narrow one at all. Where one of two candidate outputs is a pure function of the other, the
verb should return the one that is not — the caller can always spend a filter, and can never
spend its way back to information the verb declined to compute.

The tie-break is not that "every element" is more useful in the average case. It is that
picking it makes the wrong choice cheap to correct and the right choice impossible to
regret.

### `unplaced`, and why a view names it rather than judging it

A cut list silently computed over 58 of a 60-element document is a wrong answer that looks
like a right one, and nothing in the output would say so. Naming the two is the minimum that
keeps the answer honest about its own coverage. (Hypothetical by construction: the fixture
places all 60 of its elements on the clock, so `unplaced` is empty there. The case this
guards is a document mid-edit, which is exactly the document a view must still answer
about — ADR-0042.)

It stops exactly there. **Which** way such an element is malformed is `validate`'s finding,
not a view's — [ADR-0006](0006-validate-reports-facts-and-render-enforces.md)'s division of
labour, applied to a verb that reads a permissive tree
([ADR-0042](0042-montagent-json-is-a-convention-fmt-gets-a-shape-check.md)) precisely so it
can still answer about a document `validate` would reject. The same restraint is why the
mode does not judge an empty interval: that a stretch has nothing in it is a fact it
reports, and whether that is a defect is someone else's question.

## Consequences

- **ADR-0011's `query --from --to` line keeps its original words.** An ADR is amended, never
  rewritten (`docs/agents/domain.md`), so the replacement lives in ADR-0011's banner, which
  names this ADR and quotes the word it retires — a reader landing on that line first has
  already passed the banner saying *on-screen* no longer holds. Everywhere the term is
  restated — `CONTEXT.md`'s **Presence set** and **Cut list** entries — states the settled
  reading directly, and the sentence recording the question as open is gone.
- `cuts.rs`'s module comment, and `at.rs`'s on `At::stack`, stop describing a departure and
  cite this ADR instead. The code does not change: this ADR ratifies what #196 shipped, and
  the two tests that pin it
  (`an_audio_element_is_in_the_presence_set_and_carries_its_type`,
  `an_element_the_document_does_not_place_on_the_clock_is_named_rather_than_dropped`) become
  the enforcement of a decision rather than of a documented deviation.
- `unplaced` is now specified surface on both modes. It may not be dropped, and it may not
  grow a verdict.
- `docs/adr/cut_presence_scan.py` is registered in `.github/workflows/ci.yml` beside the
  other ADR scans, which means *by hand* for now — GitHub Actions is disabled for this
  repository. Run it when the fixture is edited. A failure means this ADR's prose cites
  numbers the fixture no longer produces, and the ADR needs an amendment rather than the
  scan a fix.

## Not settled here

**Whether `--where` can express the narrowing.** The decision above says a caller filters
the returned `type` field, which is a client-side operation on an answer it already holds.
Whether [ADR-0070](0070-the-where-predicate-is-a-conjunction-of-whole-value-terms.md)'s
predicate grammar should also be reachable *from* `--from --to` — one invocation asking for
the cut list of a matched subset — is a surface question #250 did not raise and this ADR
does not answer.

**The other verbs' populations are untouched.** This settles the term for the cut list,
where the contradiction was. `frame`, `preview` and `render` narrow to what paints because
painting is what they do, and no one has ever read those as claims about the presence set.
