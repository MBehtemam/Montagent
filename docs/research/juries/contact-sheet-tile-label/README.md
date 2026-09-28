# Jury: what each tile on the contact sheet is labelled with

Ballots for [#400](https://github.com/MBehtemam/Montagent/issues/400), on the map
[#395](https://github.com/MBehtemam/Montagent/issues/395). Ratified as
[ADR-0098](../../adr/0098-the-tile-label-is-a-floored-fitted-line-naming-the-change-at-its-own-boundary.md).

Three jurors on three different models — **Opus 5, Sonnet 5, Fable 5.1** — each
sent the identical brief, dispatched in parallel, blind to each other. The brief
stated the options neutrally and **withheld the convening session's own
recommendations**, so the panel was not anchored to them.

## Method, and its declared weakness

The panel was put **eight questions in one ballot**. This is the same
methodological choice ADR-0097's README flagged against ADR-0094's
single-question panel, and it carries the same cost: it permits **cross-question
trading** and buys less depth per question. Juror 2's Q3 and Q4 answers visibly
trade against each other (it accepts a delta on cost grounds at Q3 and variable
arity on legibility grounds at Q4, and the two rest on the same character budget).
Recorded here so a later reader can discount accordingly.

## How the ballots landed

| | Juror 1 (Opus 5) | Juror 2 (Sonnet 5) | Juror 3 (Fable 5.1) |
|---|---|---|---|
| Q1 where | (a) gutter | (a) gutter | (a) gutter |
| Q2 register | (c), **text** load-bearing | (c), **visual** load-bearing | (c), **visual** on sheet + text on provenance line |
| Q2 marked? | unmarked default | unmarked default | unmarked default + assert zero counts |
| Q3 provenance | **full** | **delta** | **full** |
| Q4 arity | fixed, sheet-wide drop | may vary, optional field only | fixed + placeholder, never drop |
| Q5 floor | yes, **8 px** | yes, **9–10 px** | yes, **8 px** |
| Q5 on non-fit | drop sheet-wide, then refuse | truncate/omit to provenance | truncate id with ellipsis |
| Q6 drop boundary? | no — signed delta encoding | no — drop the id instead | no — print compactly |
| Q7 font | (c) vendor, tabular figures, hard build fail | (c) vendor | (c) vendor, ASCII subset + replacement glyph |
| Q8 selector | boundary change, signed, **highest** layer | boundary change, signed, **lowest** layer | boundary change, signed, **highest** layer |

**Unanimous on four of eight** (Q1, Q5's existence, Q6, Q7), and unanimous on the
*shape* of Q8 — all three independently reframed the field from *discriminating*
to **causal** and all three independently added the **sign**, neither of which was
in the brief.

**Where the ADR departs from the panel.** Q8's layer direction went 2–1 for
topmost, but neither side argued it substantively, and the ADR resolves it on
fixture evidence the panel did not have — which also **corrects a premise all
three jurors shared**: they took layer order to be total because "a layer tie is
already an error", and ADR-0060 makes it an error only when boxes overlap in time
*and* space. See ADR-0098 §8 and `docs/research/tile-label/label_field_scan.py`.

## The ballots

Verbatim, exactly as returned. Each juror named the model backing it.
