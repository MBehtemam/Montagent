# The `validate` jury — primary source for #26

Ten agent sessions across four models (Opus, two Sonnets, Haiku, Fable), run in
isolated sandboxes each holding `CONTEXT.md`, ADRs 0001–0005, the
`en-halloween-decorating` fixture media, and one project file. No sandbox contained
findings docs, ticket comments, or the author's recommendations.

The conclusions are in
[ADR-0006](../adr/0006-validate-reports-facts-and-render-enforces.md). This file
records the method, the verified numbers, and the judge errors — the things that do
not belong in a decision record but that the next exercise needs.

## The file under test

The genuinely defective project file an agent shipped in
[#9](https://github.com/MBehtemam/Montaget/issues/9)'s second exercise, at
**"very high" self-reported confidence**. Judges were not told it was defective, or
who wrote it.

It contains four defects, three of which were known:

| | defect | class |
| --- | --- | --- |
| 1 | `vo-sentence-06-{a,b}` declare `source_end: 3368` against a file that is **2568 ms** | probe-only — invisible to every no-I/O check |
| 2 | `vo-sentence-06-b`: timeline 3981 ms against `3368 / 0.645 = 5222` — **1241 ms short** | internal arithmetic |
| 3 | **800 ms hole** in `photo` and `caption` at 30603→31403 | legal gap, uncovered by any visual track |
| 4 | `sentence-quiz` alone left at `y: 1537` while its four siblings moved to 1597 | no check can see it |

**Not one of the four is an overlap.** A plain overlap checker reports the file clean.

Defect 2's direction is diagnostic and was spotted by one judge: `3981` satisfies
`2568 / 0.645`, the **true** duration — so the timeline was computed correctly from
real media and the declared `source_end` was edited afterwards to a value the file
never had.

## Method

**Round one — a scenario, not a questionnaire.** Five judges were told an editing
session had just finished on the file, given the three tasks that session had been
set, and told they were the agent who now had to decide whether it was safe to
render. They were asked for **the verbatim text they wished `montaget validate` had
printed at them** — the literal output, byte for byte, not a description of it — and
for which findings they would have missed by hand. Only then the seven design
questions, stated neutrally with the author's recommendations stripped out. One was
briefed **hostile**: to attack whether a validator should exist at all, on the
grounds that every check it performs is evidence of a format defect that should be
fixed in the format.

**Round two — three deadlocked decisions.** Five judges, options stated neutrally
with no indication of the author's preference. One was briefed to work out the
likely consensus and argue against it before committing.

## Results

| | D1 wire format | D2 fast mode | D3 version comparison |
| --- | --- | --- | --- |
| Opus | (a) JSON canonical | (a) none | (b) fold into #10 |
| Sonnet A | (a) | (a) | (b) |
| Sonnet B *(dissent-briefed)* | (a) | (a) | (b) |
| Fable | (a) | (a) | (b) |
| Haiku | (b) text canonical | (a) | (a) new ticket |
| | **4–1** | **5–0** | **4–1** |

D3 came out **against the author's recommendation**, which was a separate ticket.
The sole dissenter on both split votes was Haiku — the model that twice reported a
correct edit as broken.

Sonnet A **reversed its own round-one position** on D1: it argued text-canonical in
round one and JSON-canonical in round two, on the grounds that ADR-0005's stale-half
objection concerns a *persisted, hand-editable* file and validator output is
regenerated in one process immediately before printing.

## What the fixture actually contains — verified by script

Every number quoted in ADR-0006 was checked against the file before being written
down, because #9's jury had already established that judges are unreliable on
structure.

- **14 tracks, 60 elements.**
- **120 time values, 109 not on the 40 ms grid** at 25 fps. **48 distinct instants,
  47 misaligned.** The only aligned instant is `0`.
- **29 internal gaps.**
- **Layer ties:** 2 tracks at layer 30, 3 at layer 31 — five tracks, two values.
- **Same-source scale discontinuities:** `1.0161 → 1.0000` at 3018 and
  `1.0542 → 1.0000` at 64816. The track's three *other* discontinuities (17472,
  43563, 54656) are cuts between **different** images and are correct.
- **`sentence-quiz`** at `y: 1537`, `size: 55`, `line_height: 1.1` → text box
  ≈1507–1567 against a card top edge of **1513**: ~6 px overhang.
- **Every media source is referenced by 2–4 elements.**
  `sentence-05-cobweb.mp3` by three — including `vo-quiz-answer`, forty seconds away
  on the timeline. `images/05.png` by four.

## Judge errors

Recorded because the pattern is now three exercises old and load-bearing for how
much weight a single agent report can carry.

**Track counts came back 12, 14 and 20** for a 154-line file. Gap counts came back 26
and 29. Frame-alignment denominators came back as 109/120, 47/48 and 54/55 — the
first two are the same fact under different units and the third is wrong.
**Presence reads reliably; structure does not.**

**Opus, the strongest report of the set, carried the most confident wrong detail.**
In round one it gave 47 of 48 distinct instants — exactly right. In round two it
wrote *"120 of 120 endpoints are off-grid"*, which is wrong; the true figure is 109
of 120. The argument is unaffected — a 91% hit rate is as damning as 100% — but the
number would have entered an ADR unchallenged.

**Haiku produced nine false-positive `error` lines** in round one, claiming the
restyle had not been done and that `y` should be 1657. The restyle *had* been done:
1537 + 60 = 1597, and 1453 + 60 = 1513. It had added 60 to the already-moved value.
Its ideal validator would have told it to revert a correct edit, at `error` severity,
above the two real defects in the same report. In round two it **cited its own
hallucination as evidence** — *"five card elements all have the same error message
('should be 1573')"*.

**The hostile judge made the identical wrong inference and did not publish it.** It
concluded "0 of 5 cards moved", then named its own method — *"that is not
verification, it is archaeology"* — and deliberately kept the finding out of its
report. Same evidence, same false conclusion, opposite outcome; what separated them
was applying the fact-versus-verdict rule honestly.

**Fable and Sonnet A found defect 4** — the one place the restyle actually failed —
by census rather than inference, as did Opus.

## The setup flaw

The brief stated that `sentence-06-spider.mp3` had been re-recorded and replaced on
disk. **It had not been.** The file is 2568 ms and was 2568 ms throughout; the
project's declared `source_end: 3368` was never true of any file that existed.

This was inherited from #9's second exercise, where the same brief was given and
**four agents converged to the millisecond on 3368 on faith from the brief, and not
one probed the media.** That is itself the finding ADR-0005 predicted — *"in a real
session nobody announces that a file got 0.8 s longer"* — occurring inside the
exercise designed to test it.

It turned the probe check from an argument into a live demonstration **by accident,
not by design**, and it is recorded here as a flaw rather than presented as a test.

## Contamination controls

Applied from the lesson of #9's first jury, where committing corrections to a fixture
README *before* launching the judges let four of five read a "discovery" straight off
it:

- Sandboxes live outside the repository. The `Corrections` section was stripped from
  the fixture README (10623 → 8675 bytes).
- No findings doc, ticket body or comment was reachable. Judges were instructed that
  the sandbox is the entire world and not to run `git` or `gh`.
- Round-two options were stated with **no indication of the author's preference**,
  and round-one questions with the author's recommendations removed.
- Every judge got the identical question set. Only the model varied — except for the
  two deliberate adversarial briefs, which produced the best finding in each round.

## The two best lines

From the hostile brief, round one — the finding that produced the `NOT CHECKED`
block:

> *"The validator I designed above would print zero errors on a file that fails two
> of its session's three tasks."*

And the same conclusion reached from the opposite direction, which became
ADR-0006's organising principle:

> *"Every wrong number in this file is mutually consistent. It is a careful file
> built on one false premise — that the audio got 800 ms longer — which the disk
> contradicts and the document cannot. Every check that compares this document to
> itself, it passes. Only the checks that compare it to something outside it — the
> media, the previous version, a rendered frame — find anything."*
