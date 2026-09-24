# Jury: `compare`'s mechanics for destroyed-coincidence and coupled-motion-drift

Evidence backing [ADR-0063](../../adr/0063-compare-drift-checks-keyframe-instant-relationships.md),
resolving [ticket #151](https://github.com/MBehtemam/Montagent/issues/151).

Two rounds, three independent models each (Opus, Sonnet — the judge/moderator,
not a juror — Haiku, Fable), all blind to each other's ballots within a round.

- **Round 1** ([opus](round-1/opus.md), [haiku](round-1/haiku.md), [fable](round-1/fable.md)):
  scope of "held" (exact coincidence vs. fixed offset), output format, severity,
  and whether the first-authoring blind spot stays out of scope. Unanimous 3/3 on
  all four questions.
- **Round 2** ([opus](round-2/opus.md), [haiku](round-2/haiku.md), [fable](round-2/fable.md)):
  whether to narrow the ticket to keyframe-involving relationships only, whether
  self-element keyframe-to-own-boundary pairs are in scope, and how much to
  design the (zero-live-example) coupled-motion case. Splits 2-1 on two of three
  sub-questions — resolved in the ADR by taking the substantive finding all three
  independently reached (slack and coincidence are not the same predicate) and
  the majority's disposition (narrow the ticket, name the residue as fog; include
  self-pairs).

`count_coincidences.py` regenerates every number both rounds cite against the
real, currently-committed fixture
(`fixtures/en-halloween-decorating/en-halloween-decorating.montagent.json`):

```
$ python3 count_coincidences.py
total elements: 60
coincident instants (2+ boundaries): 19
boundary endpoints participating in a coincidence: 92 / 120
instants with 3+ boundaries (multi-partner clusters): 14
Elements with any keyframed transform property: 7
total cross-element same-property keyframe-time coincidences: 0
total self-element keyframe-to-own-boundary coincidences: 7
```

The 92/120 figure matches ADR-0036's original historical measurement exactly,
confirming the pattern held unchanged through the file's later migration
([#42](https://github.com/MBehtemam/Montagent/issues/42)). The 14 multi-partner
clusters is the concrete count behind round 2's key finding: slack-drift
(ADR-0032) reports the *nearest*-partner distance, so a coincidence pairing can
be destroyed inside one of these 14 clusters while another partner keeps the
minimum at zero — invisible to slack-drift. That residue is recorded as fog in
ADR-0063 rather than mechanized in this ticket.
