# Recorded intent — jury evidence

Evidence for [ADR-0086](../../../adr/0086-recorded-intent-is-one-pattern-and-the-time-axis-instantiates-it.md),
resolving [#324](https://github.com/MBehtemam/Montagent/issues/324).

Two rounds, three jurors each, one model family per juror so that agreement
across a round is not agreement of one model with itself:

| | Juror 1 | Juror 2 | Juror 3 |
| --- | --- | --- | --- |
| backing | Claude Opus 5 | Claude Sonnet 5 | Claude Fable 5.1 |

No juror saw another's ballot, no juror was assigned a stance, and every juror
in a round received the same question text byte for byte. Jurors answered from
the question alone — no repository access — so the ballots turn on the evidence
as stated in the question, which is why the question text is committed beside
them.

## Files

- [`round-1-question.md`](round-1-question.md) — the four-part question
  (pattern shape, id references, violation cost, which axes ship).
- [`round-1-ballots.md`](round-1-ballots.md) — twelve ballots, verbatim.
- [`round-2-question.md`](round-2-question.md) — eight parts: round 1's four
  restated with their provisional answers and explicitly open to being
  overturned, plus the four shape questions (optionality, rule arguments,
  placement, consumer).
- [`round-2-ballots.md`](round-2-ballots.md) — twenty-four ballots, verbatim.

## Outcome

Round 1: unanimous on the pattern being one rule with several fields, on a
renderer-ignored id reference being a different object from the live reference
ADR-0036 rejected, and on splitting violation cost by whether the declaration
names a direction. Split 2–1 on how many fields ship.

Round 2: twenty-three confirmations and one overturn. Juror 3 overturned the
symmetric clause of the violation-cost answer, showing it contradicted the
same panel's own reasoning about dangling references — `review` means *"you
must look at a frame to know if it was meant"*, and a frame cannot adjudicate
either case. ADR-0086 adopts the overturn: a violated declaration is always
`error`, and only the repair class varies.

**A caution recorded with the evidence.** Round 2 showed jurors the round-1
answers as provisional, which invites anchoring. The single overturn came from
the juror who read those answers against each other rather than against the
evidence — the check anchoring makes hardest. The 23–1 margin should be read
with that in mind, and the overturn weighted above its share.

## What the ballots do not establish

The jurors reasoned about the evidence **as the question described it**, and
the question carried ADR-0037's `4 of 52` figure as fact. That figure does not
reproduce — see ADR-0086's erratum and
[`recorded_intent_scan.py`](../../../adr/recorded_intent_scan.py). The decisions
survive the correction and the replacement evidence is stronger, but no ballot
here should be cited as having weighed the real census, because none of them
saw it.
