# Jury: which of the sheet's outputs are findings, at what codes?

Three jurors, three different models, put to three of the six questions of
[#412](https://github.com/MBehtemam/Montagent/issues/412) on the map
[#395](https://github.com/MBehtemam/Montagent/issues/395). Resolved as
[ADR-0105](../../../adr/0105-the-sheets-refusals-are-invocation-errors-its-blind-spots-are-not-findings-and-an-unpainted-state-is-quantization.md).

Each juror received `BRIEF.md` **verbatim and identical** — no assigned stance, no persona,
no sight of each other's ballots, and none of the judge's recommendations, dispatched in
parallel. Ballots are recorded here unedited.

| juror | model | ballot |
| --- | --- | --- |
| 1 | Opus 5.5 | [opus-5-5.md](opus-5-5.md) |
| 2 | Sonnet 5.5 | [sonnet-5-5.md](sonnet-5-5.md) |
| 3 | Fable 5.1 | [fable-5-1.md](fable-5-1.md) |

## Only half the ticket went to the panel

The ticket was grilled as six questions. The human sent **Q4–Q6** to the court — the
`no-grid-frame` code, where `blind_to` lives, and the ADR-0094 §4 correction — and decided
**Q1–Q3** directly, by adopting the judge's recommendations: the overflow refusal is
`E-SHEET-OVERFLOW` (`NotAboutDocument`, exit 3), the flag-combination refusals are bare
`E-INVOCATION`, and `skipped[]` is a disclosure field of which only `no-grid-frame` entries also
raise a finding. Those three rest on argument from committed ADR text, not on a panel, and
ADR-0105 says so. After the ballots the human then took the judge's read on Q4–Q6 too.

| | Q4 code for `no-grid-frame` | Q5 `blind_to` | Q6 the §4 example |
| --- | --- | --- | --- |
| Juror 1 | (a) `N-QUANTIZATION`, review | (a) | (a) + constructed document |
| Juror 2 | **(b) new code**, review | (a) | (a) + constructed document |
| Juror 3 | (a) `N-QUANTIZATION`, review | (a) | (a) + constructed document |

## The split, and what the dissent contributes

Q4 split 2–1. The majority's argument is the code's own contract: a code is *"the identity a
future `compare` diffs on"*, so a `frame`-only code gives one document fact two identities the
day `validate` learns to see it. Juror 3's line is the sharpest on the panel: a verb-scoped code
*"would create a code whose meaning is 'frame saw it', which is provenance, not a document
fact."*

Juror 2's dissent is not answered by that and is recorded as ADR-0105's honest cost:
**suppression granularity.** *"Suppressing N-QUANTIZATION should not also hide 'a declared
combination is never shown.'"* Broadening a code widens what one suppression silences. Juror 2
also conceded the majority's main point in passing — a new code *"can be shared"* with
`validate` later — which narrows the split to what the code is called, not which verbs emit it.

## What the panel added that the judge did not have

**All three independently required a constructed test document before the code ships.** The
judge had offered the fixture's zero count as a correction to record; every juror went a step
further — a code that never fires on any committed document has an untested emission path,
prose rendering and `skipped` shape. ADR-0105 adopts it, and
`check_unpainted_runs.py` already constructs the instance.

**Juror 3: the per-tile provenance must print each tile's sampled instant**, so the
`below-tile-width` sentence's pointer at `frame --crop --at <instant>` is directly actionable —
the instant to type is on the line. ADR-0097 §8 already puts the instant on each provenance
line; ADR-0105 records it as the thing the pointer depends on.

**Juror 1: the `inside-run` sentence should point at `frame --at`** — the other blind spot with a
one-call remedy. Adopted.

## Where the panel is thinner than it looks

Three models from one family, and the unanimous questions (Q5, Q6) are the ones where the brief
leaned hardest on a precedent — the NOT CHECKED block, and #407's correction of §3. Unanimity
there is weak evidence of anything beyond the precedent being clear.
