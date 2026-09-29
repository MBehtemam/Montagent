# Jury: what are the sheet's two opt-in flags called, and what does each promise?

Three jurors, three different models, put to the ten questions of
[#418](https://github.com/MBehtemam/Montagent/issues/418) on the map
[#395](https://github.com/MBehtemam/Montagent/issues/395). Resolved, with a second panel's
answers, as
[ADR-0106](../../../adr/0106-the-sheets-opt-ins-are-keyframes-and-infill-ceiling-and-a-keyframe-tile-is-sampled-where-its-change-first-paints.md).

Each juror received `BRIEF.md` **verbatim and identical**: no assigned stance, no persona, no sight
of each other's ballots, and none of the judge's recommendations. All three were dispatched in
parallel. Ballots are recorded here unedited.

| juror | model | ballot |
| --- | --- | --- |
| 1 | Opus 5.5 | [opus-5-5.md](opus-5-5.md) |
| 2 | Sonnet 5.5 | [sonnet-5-5.md](sonnet-5-5.md) |
| 3 | Fable 5.1 | [fable-5-1.md](fable-5-1.md) |

| | Q1 default | Q2 keyframe flag | Q3 infill flag | Q4 term | Q5 | Q6 | Q7 | Q8 counts | Q9 | Q10 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Juror 1 | (a) off | `--keyframes` | **(b) `--infill-ceiling`** | infill ceiling | (a) | (c) | (a) | tiled + untiled, **always** | (a) | agree |
| Juror 2 | (a) off | `--keyframes` | **(d) `--max-gap`** | **keep gap ceiling** | (a) | (c) | (a) | tiled + untiled, **with the flag** | (a) | agree |
| Juror 3 | (a) off | `--keyframes` | **(a) `--infill`** | infill ceiling | (a) | (c) | (a) | tiled + untiled, **always** | (a) | agree |

The human took the judge's read on every split: `--infill-ceiling`, "infill ceiling", and both
counts on every answer.

## The split

Q3 went three ways, and Q4 split with it, since only `--max-gap` kept "gap ceiling". The judge
read it like this:

- **`--max-gap` falls to the glossary**, where **Gap** already means a stretch of a track with no
  element. Juror 2 conceded the collision.
- **`--infill` had the stronger consistency case**: one word across the flag, the class token and
  `infill-evicted`. It carries "not a count" only in its help text, and the ticket's own constraint
  was that **the name** must not read as a count.
- Juror 3 said `--infill-ceiling` would be the only hyphenated flag on `frame`. That is true of
  `frame`, but the repo already has `--no-clobber`, `--no-probe` and `--enable-gpl`.

Juror 2's objection to the term is recorded in the glossary definition rather than the name:
*"the ceiling is defined over consecutive tiles of any class."* The **Infill ceiling** entry says
exactly that.

## What the panel added that the judge did not have

**Juror 1: `untiled` can be non-zero even with `--keyframes` on**, when two change points share a
painted frame or one falls between grid frames. That exposed a question neither the brief nor any
ADR had asked: *which frame is a keyframe tile sampled at?* It became the second panel,
[`../contact-sheet-keyframe-instant/`](../contact-sheet-keyframe-instant/README.md).

**Juror 3: under Q5(a), infill adds nothing on a whole-document call on the fixture**, because 18
states fill the 18 slots at 180 px. All three accepted the cost; only Juror 3 stated it. ADR-0106
records it as an honest cost.

**Jurors 1 and 3: a ceiling longer than the range is a legal no-op**, not an error. **Juror 2: a
non-numeric `<MS>` is `E-INVOCATION` too.** Both adopted.

**Juror 2: an empty identifying-field slot renders blank, never a placeholder that looks like an
id.** Adopted.

## Set aside

Juror 2's Q8 line *"if keyframes are dropped under budget, the count of dropped ones must appear
in `skipped[]`"* contradicts its own Q9 and ADR-0105 §2: keyframe tiles are never dropped, and the
call refuses instead.
