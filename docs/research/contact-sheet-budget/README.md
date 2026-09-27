# The contact sheet's tile budget

Evidence for [ADR-0095](../../adr/0095-the-sheets-budget-is-served-tile-width-and-overflow-refuses.md)
([#399](https://github.com/MBehtemam/Montagent/issues/399), on the map
[#395](https://github.com/MBehtemam/Montagent/issues/395)).

ADR-0095 decides that the sheet's budget is denominated in **served tile width** —
a 180 px target and a 140 px floor — that overflow **refuses** rather than thinning
or splitting, and that **no wall-clock budget needs to exist** because the width
floor bounds the time first.

## What is here

`check_tile_budget.py` re-derives every number the ADR spends and exits non-zero
the moment one stops holding.

```sh
python3 check_tile_budget.py                     # 28 geometry + cost claims

MONTAGENT=target/release/montagent \
PROJECT=fixtures/en-halloween-decorating/en-halloween-decorating.montagent.json \
    python3 check_tile_budget.py                 # 33, adding the wall-clock measurements
```

## How this relates to #396 and #397

This directory grounds the **policy**; the two closed tickets ground what it is
built on, and the division matters:

- [#397](https://github.com/MBehtemam/Montagent/issues/397) (`research/visual-token-cost`)
  established the cost model — `ceil(w/28) × ceil(h/28)` on **served** dimensions.
- [#396](https://github.com/MBehtemam/Montagent/issues/396)
  ([`../contact-sheet-legibility/`](../contact-sheet-legibility/FINDINGS.md))
  measured where each class of defect stops being visible, and committed the judged
  sheets verbatim because a perceptual verdict has no formula to re-run.
- This script derives the *policy* from those: which currency can bound a sheet at
  all, what tile counts the two width constants imply, and whether wall clock needs
  a budget of its own.

The cost model is duplicated here rather than imported, so that if it ever drifts
**both** scripts fail rather than one silently inheriting a wrong number from the
other.

## The one claim that is not re-derivable

The **140 px floor itself** is #396's perceptual reading, not arithmetic. This
script checks what follows from it, never that it is the right number. ADR-0095 §4
records the weakness in terms: it rests on one primed observer where the comparable
`preview` floor (ADR-0050) had three independent jurors, which is why the *target*
sits at 180 px and carries a 31% margin. A three-juror pass of ADR-0050's shape
would strengthen it and has not been run.

## Timing is hardware-bound

The wall-clock figures are bounds on one machine, and the script re-measures rather
than asserting them as constants. What the decision actually rests on is the
**ordering** — under `preview`'s 5 s at the floor's 30 tiles, over `frame`'s 500 ms
at 18, and half-scale rasterization saving far less than its pixel-area ratio
suggests — and those hold on a fast or slow machine alike.
