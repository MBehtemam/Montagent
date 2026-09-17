---
status: accepted
amends: 0006 (adds `R-BOX-SLACK` to the check list), 0014 (closes its recorded residual: "an over-large `height` disables its own tripwire... nothing in this design catches it")
---

# `validate` gains `R-BOX-SLACK`: a `note`-level, height-only, census-carrying check for an oversized text box

[ADR-0014](./0014-stroke-is-paint-the-text-box-is-required.md) required every text
element to declare `width`/`height`, then recorded the gap it left open: a box much
bigger than the text it bounds disables the overflow check exactly as effectively as
omitting the box would, and nothing in the design distinguished the two. This ADR
gives that gap a check.

## The evidence

Re-derived against the committed fixture using [ADR-0028](./0028-text-block-arithmetic-is-exact-tenths.md)'s
exact-integer formula (`height = (size × line_height×10 × line_count + 9) // 10`),
not the float `ceil` this map's decisions record elsewhere had cited. That
re-derivation is itself a small correction: ADR-0028 already eliminated the
floating-point residual ADR-0014's "≤0.5px" language described, so all 22 text
elements now split cleanly into two populations with **no scattered middle**:

| element | computed | declared | slack | % | sibling match on declared extent |
| --- | --- | --- | --- | --- | --- |
| `sentence-05` | 61 | 169 | 108 | 177% | `card-05`, `card-06`, `card-07`, `card-08`, `card-quiz`, `sentence-06`, `sentence-07`, `sentence-08`, `sentence-quiz` |
| `sentence-08` | 61 | 169 | 108 | 177% | (same set) |
| `sentence-quiz` | 61 | 169 | 108 | 177% | (same set) |
| `handle-text` | 38 | 84 | 46 | 121% | `chip-panel`, `chip-text`, `handle-panel` |
| `sentence-06` | 126 | 169 | 43 | 34% | `card-05..08`, `card-quiz`, other `sentence-*` |
| `sentence-07` | 126 | 169 | 43 | 34% | (same set) |
| `chip-text` | 58 | 84 | 26 | 45% | `chip-panel`, `handle-panel`, `handle-text` |
| *(remaining 15 of 22)* | — | — | **0** | 0% | — |

The 15 exact matches are elements whose declared `height` is the arithmetic result
itself. The 7 with slack are every text element that shares a card/chip/panel — each
declares the *container's* height verbatim rather than its own computed block
height. There is no element anywhere in the fixture with slack between 0 and 26 —
no "natural padding" case exists to calibrate against, only exact derivation or
apparent copy.

## Decision

Put to a three-model court (Claude Opus 5, Claude Haiku 4.5, Claude Fable 5.1),
independent ballots, no persona, blind to each other and to any recommendation;
severity was then put to the human directly after the court split.

### Severity: `note`

Court split 1–2 (Opus: `review`; Haiku, Fable: `note`); **the human resolved it as
`note`.** The deciding reading of ADR-0006's own test: `review` means "legal,
renders, and you must look at a frame to know if it was meant" — but an oversized
box has zero rendering effect, so there is no frame that would show anything either
way. The harm this check names is deferred (a future edit that grows the text past
its true bound, silently absorbed by leftover slack) rather than present, which is
exactly ADR-0006's `note` case: "a fact you may want and will not act on today."
Consequence: on a clean run this collapses into the counted notes line rather than
printing in full, per ADR-0006's noise-budget rule.

### Threshold: `slack > max(2px, 10% of computed height)`

Unanimous 3/3 on the hybrid shape, unanimous that the exact constants are
**conventional, not measured** — the fixture's only two populations (0 and
26–108) are separated by any value in a two-order-of-magnitude gap, so no number in
that range is more justified by evidence than another. The shape itself is not
arbitrary: a pure percentage misfires at the small end (10% of a short single-line
caption can be smaller than a single pixel), and a pure fixed value doesn't scale
across a format with no assumed font size. `2px` is kept as a floor for small text
even though ADR-0028 has since removed the float-rounding reason ADR-0014 would
have cited for it — a trivial-padding floor, not a noise floor.

On the fixture: all 7 non-exact elements clear the threshold at every plausible
constant in the justified range (minimum observed slack 26px / 45%, well above
`max(2px, 10%)` for any of their sizes); all 15 exact elements read 0 and never
fire.

### Finding code and message: `R-BOX-SLACK`, with a sibling census

2 of 3 jurors independently proposed `R-BOX-SLACK`, matching the established
`R-<subject>-<symptom>` shape (`R-VISUAL-GAP`, `R-EASE-INERT`) — a symptom noun,
never a verdict. Rejected in the process: anything containing `COPY` or `STALE`,
which would bake a causal claim into the identifier itself, not just the message.

**Include the sibling census, unanimous 3/3.** When the declared height exactly
equals another element's declared extent, state that coincidence as a bare fact —
never "copied from", never "should be". It is the one fact that tells the two
populations apart (every non-exact case in the fixture has a match; no exact case
does), and per ADR-0006 a finding may state any fact derivable from the document,
including a census of its siblings.

```
R-BOX-SLACK: text "sentence-05" declares height 169; computed block height is 61
(size 55 × line_height 1.1 × 1 line) — slack 108 (177%). height 169 also declared
by: card-05, card-06, card-07, card-08, card-quiz, sentence-06, sentence-07,
sentence-08, sentence-quiz.
```

The census clause is omitted, not replaced with "no match found", when nothing
matches — stating the negative would itself imply the copy hypothesis was tested
and ruled out, which is more than the document supports.

### Scope: height-only

Unanimous 3/3. `height`'s expected value is arithmetically derivable from fields
already in the document (`size`, `line_height`, mandatory-break count); `width`'s
is not — [ADR-0014](./0014-stroke-is-paint-the-text-box-is-required.md) already
parked `width` overflow-checking as `UNCHECKED` pending a `measure` call, because
width needs the font binary and shaper, not arithmetic on the element. A width
slack check is a different check with a different dependency, not a wider version
of this one, and reopening it here would smuggle a settled decision's reversal
into a ticket whose evidence is entirely about height. Recorded residual: a
copy-pasted box has almost certainly copied `width` too, and this check stays
blind to that half — tolerable because the height census already surfaces the
element most likely to be the copy's source, which typically carries both
dimensions.

## Consequences

- **`validate` gains `R-BOX-SLACK`** (`note`, height-only, `slack > max(2px, 10%
  of computed height)`, sibling census on exact match), computed from `size`,
  `line_height`, `runs` and `height` with no I/O.
- **Closes ADR-0014's recorded residual** for the height axis; the width axis
  stays exactly where ADR-0014 left it (`UNCHECKED` pending `measure`).
- **Corrects ADR-0014's "≤0.5px" language**: re-derived under
  [ADR-0028](./0028-text-block-arithmetic-is-exact-tenths.md)'s exact-integer
  formula, the residual is exactly 0 on all 15 derived elements — there was never
  float noise to set a floor against, only a defensive one for text sizes this
  fixture doesn't exercise.
- No schema change — `height` was already required by ADR-0014; this is a new
  read, not a new field.

## Evidence

Three-juror court on the four sub-questions (severity, threshold shape, finding
code + census, scope), ballots recorded in the ticket
([#132](https://github.com/MBehtemam/Montaget/issues/132)); severity resolved by
the human after a 1–2 split. Per-element table above re-derived directly from
`fixtures/en-halloween-decorating/en-halloween-decorating.montaget.json` against
ADR-0028's formula.
