# Jury: the `mask` effect's parameter set

Evidence for [ADR-0084](../../../adr/0084-the-mask-rect-is-one-shape-independent-parameter-set.md).

Three independent jurors — **Opus 5**, **Sonnet 5**, **Fable 5.1** — each given the same brief
([`BRIEF.md`](BRIEF.md)) cold, blind to each other and to the author's recommendations,
instructed to answer from the packet alone and permitted to reject a question's framing. Full
ballots: [`opus-5.md`](opus-5.md), [`sonnet-5.md`](sonnet-5.md), [`fable-5-1.md`](fable-5-1.md).

The question: [ADR-0040](../../../adr/0040-effect-model-attachment-and-v1-vocabulary.md) wrote
the mask union member as `mask{shape: "circle"|"rect"|"ellipse", ...shape params}` and never
wrote the ellipsis.
[ADR-0068](../../../adr/0068-the-bare-mask-key-retires-masks-are-effects-members.md) stated the
minimum its migration forced — the param-less form is the inscribed shape — and graduated the
rest to [#185](https://github.com/MBehtemam/Montagent/issues/185).

## Verdicts

| | Opus 5 | Sonnet 5 | Fable 5.1 |
| --- | --- | --- | --- |
| **Q1** explicit geometry in v1 | **(a)** add | **(b)** param-less only | **(a)** add |
| **Q2** coordinate space | element-local, `(0,0)` = rect top-left, ignores `origin` | same | same |
| **Q3** transform | rides `scale`/`rotation`; indirect animation OK | same | same |
| **Q4** non-square `circle` | **(b)** `review`, `circle`-only | **(b)** `review`, `circle`-only | **(b)** `review`, `circle`-only |

**Unanimous on Q2, Q3 and Q4**, and — the part that carries weight — reached by routes other
than "the shipped renderer does it". All three argued Q2/Q3 from the `clip`-duplication
ground: a frame-space or transform-immune mask is a second spelling of a concept ADR-0025
already named and made permanently static. All three independently flagged that agreement with
unratified code is weak corroboration given
[ADR-0075](../../../adr/0075-the-badges-mask-changes-its-own-rim-and-nothing-else.md), and two
asked for a golden on a rotated, scaled, non-`top-left` masked element instead of an assertion.
That request is carried into ADR-0084's Evidence as a commissioned test.

**A terminology hazard the ballots expose:** Opus 5 and Fable 5.1 call the answer to Q3
*"pre-transform"*; Sonnet 5 calls the identical behaviour *"post-transform"*. They describe the
same pixels. ADR-0084 therefore states Q3 in drawing-order terms and uses neither word.

**Split 2–1 on Q1**, the only real disagreement. Sonnet 5 would ratify param-less-only and defer
explicit geometry pending a forcing case, on ADR-0049's don't-admit-until-forced discipline and
ADR-0068's explicit warning against guessing the explicit form's shape now. It turns the
project's own principle around with some force: if the fixture is never evidence a capability is
unneeded, neither is it evidence for shaping a member's parameters by what one project happens
to *lack*.

ADR-0084 resolves against that ballot on a precedent **no juror was given**:
[ADR-0014](../../../adr/0014-stroke-is-paint-the-text-box-is-required.md) admitted `rect`'s
`radius` with zero fixture evidence, on exactly this argument — *"a rounded rectangle is
unremarkable in the CapCut/Premiere reference class, and admitting it now costs one clause where
admitting it later is a schema change."* The dissent is preserved in the design rather than
overruled by it: the all-or-none rule and the shape-independent rect exist to keep a guessed
surface as small as a guess is allowed to be.

## What the jury did not supply

**No juror proposed the shape-independent rect that ADR-0084 adopts.** The panel's only concrete
sketch (Fable 5.1) was per-shape — `circle{cx,cy,r}`, `ellipse{cx,cy,rx,ry}`,
`rect{x,y,width,height,radius}` — and the objection that rules it out was found after the court,
in review of the ballots:
[ADR-0049](../../../adr/0049-v1-colour-filter-vocabulary-four-scalar-members.md) refused a single
`color{mode, ...}` effect because *"it would be the only two-level lookup in v1's vocabulary"*,
and per-shape field sets make `shape` exactly that. Recorded here because it is the substantive
respect in which the ADR is not what the jury described, and because a jury record that only
lists agreements is not a record.
