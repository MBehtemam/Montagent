# Jury: what does a report say about the checks it did not run?

Three panels of three jurors, three different models each time, put to
[#403](https://github.com/MBehtemam/Montagent/issues/403) (*timeline prints a check-engine
scoreboard while running no checks*), found while evaluating
[#395](https://github.com/MBehtemam/Montagent/issues/395). Resolved as
[ADR-0112](../../../adr/0112-a-report-names-the-check-sets-that-ran-and-prints-no-zero-it-did-not-earn.md).

For each round, every juror received that round's brief **verbatim and identical**, apart from
their juror number. They got no assigned stance, no persona, no sight of each other's ballots,
and none of the judge's recommendations. Jurors were dispatched in parallel. Ballots are
recorded here unedited.

| Round | Brief | Opus 5.5 | Sonnet 5.5 | Fable 5.1 |
| --- | --- | --- | --- | --- |
| 1 — Q2–Q4 | [BRIEF.md](BRIEF.md) | [opus-5-5.md](opus-5-5.md) | [sonnet-5-5.md](sonnet-5-5.md) | [fable-5-1.md](fable-5-1.md) |
| 2 — Q5–Q9 | [BRIEF-2.md](BRIEF-2.md) | [round 2](opus-5-5-round-2.md) | [round 2](sonnet-5-5-round-2.md) | [round 2](fable-5-1-round-2.md) |
| 3 — Q10 | [BRIEF-3.md](BRIEF-3.md) | [round 3](opus-5-5-round-3.md) | [round 3](sonnet-5-5-round-3.md) | [round 3](fable-5-1-round-3.md) |

| | Q2 JSON? | Q3 tiers | Q4 NOT CHECKED | Q5 sets/classes | Q6 recorded | Q7 names | Q8 zeros | Q9 ADR, term | Q10 stopped run |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Juror 1 | B | B | B | B | B | A | A | yes, Check set | **C, `validate:partial`** |
| Juror 2 | B | B | B | B | B | A | A | yes, Check set | **C, split** |
| Juror 3 | B | B | B | B | B | A | A | yes, Check set | **C, split** |

Q1 (keep a header that says whose checks ran) was not put to a panel. All three round-1
jurors assumed it, and the human adopted it with their votes.

The human took the jury on every unanimous question and the judge's read on Q10's one split:
**split `validate` into `document` and `disk`**, with no `validate` alias.

## What each round changed

- **Round 1** settled that the fix is data, not prose, and reaches NOT CHECKED too.
- **Round 2** settled *what* the data names. All three rejected finding classes on the same
  fact, that a refusal is a finding without being a check. All three reached the same
  set-to-class table without being prompted.
- **Round 3** exists because the judge's round-2 brief was wrong. It described a missing
  `ffprobe` as simply an internal-failure exit, and Fable concluded that no partial state
  existed. In fact the document half's findings stay in the report. The judge's
  recommendation for that case (record `[]`) got **zero votes**. All three jurors said it
  contradicts the ratified rule that the field records checks as they execute. Opus's
  objection to the split (a no-media run would look stopped) was answered independently by
  Sonnet and Fable: a no-media run completes `disk` trivially.
