---
status: accepted
amends: 0032 (closes the pairwise gap named but not mechanized there), 0063 (the new predicate is sibling to, not an extension of, the one shipped there)
---

# Boundary-coincidence-cluster drift is its own `compare` predicate, scoped to all N≥2 and suppressing slack-drift's zero-distance case

[Ticket #162](https://github.com/MBehtemam/Montaget/issues/162), graduated from
the map's fog, originally named as unmechanized residue in
[ADR-0063](./0063-compare-drift-checks-keyframe-instant-relationships.md).
Resolved by a jury of three independent models (Opus, Sonnet, Haiku) plus a
fixture measurement pass against the real, currently-committed fixture
(`fixtures/en-halloween-decorating/en-halloween-decorating.montaget.json`).
Evidence in
[`docs/research/juries/boundary-coincidence-cluster-drift/`](../research/juries/boundary-coincidence-cluster-drift/).

## The gap

[ADR-0032](./0032-slack-is-invariant-shift-refuses-compare-is-the-backstop.md)
gave `compare` a slack-drift check: it reports when a boundary's distance to
its *nearest* neighbour changes between two file versions. [ADR-0063](./0063-compare-drift-checks-keyframe-instant-relationships.md)
added a second predicate — exact-equality drift over keyframe-involving
pairs — but explicitly left boundary-vs-boundary pairs to slack-drift,
reasoning "a coincidence is slack of size zero."

That reasoning holds for a two-way tie and fails for a wider one. Slack is a
*minimum over the nearest neighbour* — one number per boundary. Coincidence
is *pairwise*. When an instant has three or more boundaries on it, one
specific pairing can separate while a different partner keeps the moved
boundary's nearest-neighbour distance at zero, so slack-drift reports
nothing. This is not a rare shape: **14 of the real fixture's 19 coincident
instants have three or more boundaries on them** (widths 3–11), confirmed by
[`count_coincidences.py`](../research/juries/compare-drift-checks/count_coincidences.py).

## The predicate: new, not an extension of ADR-0063's

**Boundary-coincidence-cluster drift** is a fourth `compare` predicate,
distinct from ADR-0063's exact-equality-drift predicate rather than a widened
version of it. ADR-0063's predicate is identified by requiring *at least one
keyframe* on one side of the pair, and its fact shape is pairwise: "two
instants equal in ref, unequal in current." Once a cluster's fact needs to
name a moved-set and a stayed-set together (see below), that shape no longer
fits ADR-0063's pairwise convention — reusing that predicate's output would
be self-inconsistent with the convention it defines. A new, sibling
predicate keeps both legible: ADR-0063's stays "at least one keyframe
involved"; this one is "no keyframe involved, boundary only."

**Scope: all N≥2 boundary coincidences, not just N≥3.** Restricting the new
predicate to multi-partner (3+) clusters would leave two predicates
partially covering the same 2-way case (slack-drift and the new predicate
both watching an ordinary tie), reopening exactly the ambiguity this ADR
exists to close. Covering every boundary coincidence uniformly is what makes
suppression (below) safe: the new predicate has no coverage gap relative to
slack-drift's zero-distance reporting, so nothing slack-drift used to catch
is silently dropped.

## Candidate granularity: cluster-level, not pairwise

Measured, not assumed. `measure_cluster_drift.py` simulates "one partner
moves" against the fixture's 14 multi-partner clusters:

```
pairwise-drift facts emitted: 68
cluster-level facts emitted:  14
ratio: 4.9x more facts under pairwise enumeration
```

and the worst case — the widest cluster (N=11) fully scattering — is **55
pairwise facts against 1** cluster-level fact. Pairwise enumeration
(`C(N,2)` candidates per cluster) floods output on exactly the common case:
most of the fixture's coincident instants are already 3+-way. It also has no
natural way to say "and these stayed coincident" — the fact that makes a
report actionable — without emitting the stayed-pairs as further candidates,
compounding the blow-up.

**One fact per destroyed cluster.** A cluster candidate names every boundary
that shared the instant in `ref`, and partitions them into moved and stayed
in `current`:

```
boundaries A, B, C, D, E were coincident at 6000 in <ref>; in <current>
D moved to 6300; A, B, C, E still coincide.
```

A cluster where nothing moved produces no fact, same as any other `compare`
predicate.

## Relationship to ADR-0032's slack-drift: suppress the zero-distance case

Because the new predicate covers all N≥2 boundary coincidences, it strictly
generalises what slack-drift would report at distance zero — naming which
specific partner moved, rather than only that some nearest-neighbour
distance changed. Reporting both would duplicate the same drift under two
predicates for every ordinary two-way tie breaking, which is the common
case, not an edge case.

**Slack-drift suppresses its own report when a boundary's zero-distance
slack changes** — that case is now exclusively the new predicate's to
report. Slack-drift keeps its existing, unchanged role for every
**nonzero**-distance change (gaps, lead-outs), which the new predicate does
not touch at all: the two are disjoint above distance zero, so nothing else
about ADR-0032 changes.

This was chosen over leaving both active un-suppressed: cross-predicate
duplication is a `compare`-output-quality problem this project has not
previously accepted elsewhere (compare's grouped-deltas-as-facts convention,
[ADR-0011](./0011-tool-surface-reads-checks-renders.md), aims for one fact
per event). It was also chosen over reframing slack itself into a
multi-valued relation — that would touch a load-bearing, already-shipped
invariant (`shift`'s refusal logic, ADR-0032) for a problem that is about
compare's *reporting*, not about what slack *means*.

## Output shape

Reuses `compare`'s existing grouped-deltas-as-facts convention
([ADR-0011](./0011-tool-surface-reads-checks-renders.md)), the same shape as
slack-drift and ADR-0063's predicate: a population, a common outcome, the
exceptions named. A grouping key distinguishes this population
(`boundary-coincidence-cluster`) from ADR-0063's three keyframe-involving
populations. **No severity** — same rule as ADR-0063: `compare` stays purely
descriptive, and a destroyed coincidence is frequently the entire point of
an edit, so assigning severity would require judging intent `compare` cannot
see.

## Consequences

- **`compare` gains a fourth predicate**: boundary-coincidence-cluster
  drift, covering all N≥2 boundary-vs-boundary coincidences, reporting one
  fact per destroyed cluster (moved-set vs. stayed-set), no severity.
- **ADR-0032's slack-drift suppresses its zero-distance report**, now fully
  superseded there by the new predicate; its nonzero-distance role
  (gaps, lead-outs) is unchanged.
- **ADR-0063's exact-equality-drift predicate is unchanged** — it keeps its
  "at least one keyframe" scope; the new predicate is a sibling, not an
  extension.
- **The map's Not yet specified entry for this gap is resolved**, recorded
  in [#162](https://github.com/MBehtemam/Montaget/issues/162).
- **No schema change, no renderer change.**
