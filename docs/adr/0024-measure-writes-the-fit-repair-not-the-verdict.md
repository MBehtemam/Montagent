---
status: accepted
amends: 0011 (`measure` gains a fitted-extent output), 0015 (discharges its "no tool writes
  the repair" deferral)
---

# `measure` writes the fit repair, not the verdict

[ADR-0015](./0015-fit-is-a-derivation-claim-and-gravity-retires.md) promoted fit deviation to
an `error` at strict equality but shipped no tool that produces the integer the error names.
The author must recompute `(s_slack * b_driving) // s_driving` by hand, in exact integer
arithmetic, against source dimensions they must probe themselves — [#48](https://github.com/MBehtemam/Montaget/issues/48)'s
consumer exercise found a re-exported source forcing this was the single most common edit.
[#54](https://github.com/MBehtemam/Montaget/issues/54) asked what closes that gap, and raised
a larger question underneath it: should `width`/`height` even stay author-written, or should
they become tool-derived and optional under `cover`/`contain`, dissolving the strict-equality
rule ADR-0015 stated is conditional on authors typing the integer?

## Decision

**The larger question is not reopened here.** [ADR-0014](./0014-stroke-is-paint-the-text-box-is-required.md)
already fought this fight for text `height`, derivable on 15 of 22 real elements, and kept it
required and author-written: the derivation holds only at authoring time and is armed against
the next edit; an optional field is indistinguishable, under [ADR-0006](./0006-validate-reports-facts-and-render-enforces.md),
from a decision not to check; and "declared, never defaulted" is satisfied because the
derivation's inputs are already in the document. All three arguments transfer to fit extents
without modification — a re-encoded, re-oriented, or swapped source re-arms a tool-derived
`width`/`height` exactly as a lengthened line re-arms `height`. A missing repair tool is
evidence the tool surface is thin, not evidence the field design is wrong; treating a tooling
gap as license to relitigate a schema-semantics ADR would let every future tooling ticket do
the same. `width` and `height` stay required and author-written on every element.

**`measure` gains the fitted-extent output — no new verb.** [ADR-0011](./0011-tool-surface-reads-checks-renders.md)'s
tool-surface jury was built hostile to a fat API and already gave `measure` the role of
handing the author derived quantities before they write — ink box, advance width, break
opportunities for text. A fitted extent for a raster element is the same shape of answer for a
different element kind; a new verb would duplicate a role `measure` already owns, the exact
fattening ADR-0011 rejected.

**`measure` returns the bare derived extent and which axis drives — no verdict, no diff
against the declared value.** ADR-0006 gives `validate` sole authority to compare the
document against itself and pronounce error; a second tool that also compares would create
two paths to the same judgment that can silently disagree, most obviously across an edit made
between the two calls. `measure` does not require a declared value to exist at all — it must
work identically for an element being authored for the first time. The driving axis is part
of the derivation, not a verdict: it is what makes the number diagnosable rather than
mysterious.

**No special-casing of `contain`'s zero-extent case.** ADR-0015 already settled that a
300x7 source into a 10x10 box derives 10x0 with no clamp, because a clamp would fabricate an
integer the published rule did not produce. `measure` must report exactly that arithmetic,
including a literal `0`, or it becomes a second rule that silently disagrees with ADR-0015 —
an author who transcribes a clamped value from `measure` would then fail `validate`, which
is the exact failure this ticket exists to close.

## Evidence

Three-model court (Claude Opus, Claude Haiku 4.5, Claude Fable), each blind to the others'
ballots, voting on all four sub-questions above. **Unanimous 3/3 on every question**, with
convergent reasoning across independently-sampled models: draw the `validate`/`measure`
boundary at judgment versus derivation, and don't let a tooling ticket reopen a
schema-semantics decision by the back door. No juror flagged the framing as wrong or
identified a consideration the grilling round missed.

## Consequences

- `measure`'s return shape becomes element-kind-dependent: text elements return per-line ink
  box, advance width, block extent and break opportunities (ADR-0011); raster-source elements
  additionally return the derived `width`/`height` and the driving axis under `fit`.
- `width` and `height` remain required, author-written, and checked at strict equality by
  `validate` (ADR-0015), unconditionally — the strict-equality rule's stated premise ("while
  authors type the integer, there must be exactly one legal integer") holds and is not
  revisited.
- `validate`'s fit-deviation error message is unaffected by this decision; it does not gain a
  reference to `measure`'s output. If the two-call repair loop (read the error, call
  `measure`, write, re-validate) proves too slow in practice, the fix is enriching `validate`'s
  error message with the recomputed value, not adding a verdict to `measure`.
- `contain` can still legally derive a zero-extent element; `measure` surfaces it rather than
  masking it, leaving any decision about forbidding or flagging zero-extent elements to a
  future ADR amending ADR-0015, not to this one.

## Not settled here

- Whether a zero-extent (or near-zero) derived value should additionally trip a `validate`
  finding of its own, distinct from the fit-deviation error. Left to ADR-0015's owners.
- The larger question this ticket declined to reopen — whether `width`/`height` should ever
  become tool-derived and optional under `cover`/`contain` — remains open should new evidence
  (as opposed to renewed argument) surface.
