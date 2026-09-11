# Jury and consumer evidence for ADR-0015 (`fit`)

The evidence [ADR-0015](../../../adr/0015-fit-is-a-derivation-claim-and-gravity-retires.md)
rests on, committed per `docs/agents/domain.md`'s *"commit the evidence an ADR rests on."*
Twenty-four agents across four models: sixteen deciding, eight **using**.

The headline is that those two groups disagreed, and the users were right.

## `round-1-design/` — eight jurors, six questions

Opus ×2, Sonnet ×2, Fable ×2, Haiku ×2, on one brief, blind to each other and to the
author's write-up. Settled: `fit` is a derivation claim (7–1), the box is `clip` (8–0), `fit`
is required (7–1), `gravity` retires (8–0), the deviation check becomes an `error` (8–0).

Two findings came from here that no single reader had produced: that admitting `contain`
**forces** ADR-0013's aperture-coverage error to be parameterised, and that *"the source's
dimensions"* was undefined and `speed`-shaped.

## `round-2-design/` — eight fresh jurors, five questions

`BRIEF.md` is what they were given. Round 1's outcomes appear there as flat premises with **no
vote counts** — a juror told a finding carried 7–1 weighs it differently from one told it is
settled — and R2-Q1's candidate names are listed alphabetically so the plurality did not sit
first and collect the anchoring vote.

One juror falsified **ADR-0013's own tiebreak (2)**: ceil preserves containment exactly as
trivially as floor preserves coverage. Verified independently; floor survives on the other two
tiebreaks. The ADR predicted `contain` would vindicate a reason that `contain` refutes.

## `consumer-exercise/` — eight agents authoring real elements

`SPEC-declared.md` and `SPEC-literal.md` are byte-identical but for the escape value's name;
`TASKS.md` is six authoring jobs with checkable answers. Consumers were barred from
`docs/adr/` and worked only from a published-style reference, matching ADR-0011, where agents
receive the schema and format docs as resources and never the decision records.

**This round overturned both design juries.** All eight authored correctly — every one found
`contain`, found the escape value, applied the EXIF transposition, and left the `scale`
keyframes untouched. But under the `{floor, ceil}` predicate that sixteen design jurors had
endorsed 16–0, **three of six tasks produced two byte-different, equally legal files**
(1546/1547, 66/67, 1733/1734). That is #44's founding complaint reproduced under the design
commissioned to end it. The agent that diverged on two tasks had independently named
floor-versus-ceil the vocabulary's worst flaw, and changed its position on seeing the data.

Two further things only this round could show:

- **8 of 8 reached for `gravity` or `align`**, several copying `"gravity":"top"` out of the
  shipped fixture before reading the spec. Agents author by copying the nearest example — which
  is why the retirement and the fixture migration are one commit.
- **`contain` leaves slack nobody places.** On the badge task, 4 agents centred and 4
  top-anchored. Both juries reasoned about a corpus containing only `cover`, where no slack
  exists, so neither could have seen it. Recorded in ADR-0015's *Not settled here*.

## Reading these honestly

These are agent outputs, not verified findings. At least three claims in them are **wrong** and
were caught by computation before reaching the ADR:

- a round-2 juror's assertion that ceil can overflow the box under `contain` (it cannot —
  0 of 3,286,969 cases);
- a juror's EXIF and PAR divergence figures, which did not reproduce; the ADR publishes the
  scan's own numbers instead;
- a consumer's proposal to cache computed fit values **in the project file**, which contradicts
  ADR-0002.

Every number in ADR-0015 is re-derived by
`docs/research/sample-project-migration/fit_vocabulary_scan.py`, which exits non-zero if any of
it stops reproducing. Prefer the script to anything asserted in these transcripts.
