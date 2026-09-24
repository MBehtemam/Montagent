---
status: accepted
amends: 0004 (states that its "absolute times unmissable" duty is satisfied by any
  format that states times explicitly, spatial or not), 0011 (`timeline`'s CLI output is
  not required to render a spatial axis)
---

# The agent's overview is not required to be spatial

[ADR-0004](./0004-tracks-as-constrained-lanes.md) carries an accepted requirement:
*"the published schema and `montagent timeline` output must both make absolute times
unmissable."* [#11](https://github.com/MBehtemam/Montagent/issues/11) established that
some overview halves an agent's tool-call cost on real tasks at no cost to correctness
(disjoint ranges, raw file vs. overview), but left open whether that overview has to be
*spatial* — an ASCII axis with bars and a ruler — or whether the same facts printed flat
do the same job. [#63](https://github.com/MBehtemam/Montagent/issues/63) asked for that to
be settled by measurement rather than opinion, since #11's own ablation had only n=3 per
arm and its ranges overlapped.

## Decision

**An agent-facing overview is not required to be spatial. A flat listing of the same
aggregated facts is sufficient, and nothing in this project's evidence supports paying
for an axis.**

`docs/research/prototypes/timeline-output/FINDINGS.md`, round 3: three more runs per arm
(n=6 total each, one model, blind to the ablation, harness-recorded tool counts, scored
against `rerun/GROUND-TRUTH.txt`) did not separate the spatial (Y1) and flat (Y2) arms —
they converged. Mann-Whitney U(Y1, Y2) = 10 against a critical value of 5 at α=.05,
two-tailed, for n=6,6: not significant. More tellingly, the *direction* moved against the
axis: Y1's three new runs (6, 10, 12 tool calls) pushed its own maximum past Y2's, so the
arm carrying the spatial layout produced this round's single most expensive run.
Correctness stayed 5/5 across all 30 recorded answers, in both rounds and both arms — the
overview, spatial or flat, never changed a result, only cost, and even that cost is now
statistically indistinguishable between the two forms.

This settles the question #11 raised and left open: *"whether the aggregation is doing
all the work and the axis none of it."* It is. What ADR-0004's requirement actually needs
— absolute times stated explicitly, not left to be inferred from position or array order
— is satisfied by Y2's flat form exactly as well as Y1's ruler: both print every instant
as an explicit number. The requirement was never, on its own text, a requirement for a
*picture*; ADR-0004 asked that times be unmissable, not that they be laid out on an axis.

## Evidence

`docs/research/prototypes/timeline-output/rerun/score.py`, extending round 2's dataset
with round 3's six new runs and a Mann-Whitney computation. Full narrative in
`docs/research/prototypes/timeline-output/FINDINGS.md`, "Round 3 — raising n per arm."

## Consequences

- **No spatial rendering is required of any agent-facing overview tool.** Where an
  aggregated view is built — per ADR-0011's note that the agent already reaches this
  through `query`'s census/aggregation modes — it may be implemented as a flat listing.
  This removes a real implementation cost: no ASCII bar-chart renderer, no character-width
  arithmetic, no terminal-width sensitivity, no Unicode box-drawing edge cases.
- **ADR-0004's requirement is satisfied by either form.** Its own text names the goal
  (absolute times unmissable), not the mechanism (a ruler); this ADR records that reading
  so a future implementer does not treat "spatial" as an inherited constraint.
- **`timeline` (ADR-0011, CLI-only, "the human's wide view") is unaffected.** This
  decision is scoped to what an *agent* needs from an overview, measured with an agent as
  the reader. Whether a human benefits from a spatial layout was never tested here and is
  not decided by this ADR — `timeline` remains free to render spatially if and when it is
  built, since a human reader is a different consumer with different evidence requirements.
- Round 1's original verdict — voided in `FINDINGS.md` for contamination — turns out to
  have been directionally right by accident: a spatial axis is not required. The
  corrected path to that conclusion is round 2 and round 3's measurement, not round 1's
  compromised jury, and the distinction matters: round 1 also concluded no overview should
  exist at all, which round 2 already falsified.

## Not settled here

- Whether `timeline`'s human-facing output should be spatial, flat, or something else is
  not addressed — no measurement here used a human reader.
- The exact shape of the flat aggregation (field names, grouping, whether it is `query`'s
  output verbatim or a dedicated mode) is an implementation question, not a design one, and
  is left to whoever builds it against ADR-0011's existing verb table.
