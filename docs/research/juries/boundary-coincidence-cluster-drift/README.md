# Court: closing the boundary-coincidence-cluster gap in `compare`'s drift checks

Evidence backing [ticket #162](https://github.com/MBehtemam/Montagent/issues/162),
graduated from the map's ("Not yet specified") fog, originally named in
[ADR-0063](../../adr/0063-compare-drift-checks-keyframe-instant-relationships.md).

Three independent jurors (Opus, Sonnet, Haiku), blind to each other's ballots,
voted on four sub-questions: the detection mechanism, candidate-fact
granularity for a multi-partner cluster, this check's relationship to
ADR-0032's slack-drift, and whether the design was ticketable without a
fixture measurement pass. Splits: 2-1 for extending ADR-0063's predicate
(mechanism), 2-1 for cluster-level facts over pairwise enumeration, 2-1 for
suppressing slack-drift's zero-distance report, unanimous for running a
measurement pass before finalizing.

`measure_cluster_drift.py` runs that pass against the real, currently-committed
fixture (`fixtures/en-halloween-decorating/en-halloween-decorating.montagent.json`),
extending `count_coincidences.py`'s cluster count with a "one partner moves"
simulation to compare pairwise (`C(N,2)`) vs. cluster-level fact volume:

```
$ python3 measure_cluster_drift.py
cluster count: 14
widths: [3, 3, 3, 3, 3, 5, 5, 6, 7, 7, 8, 8, 10, 11]
widest cluster: 11
...
pairwise-drift facts emitted: 68
cluster-level facts emitted:  14
ratio: 4.9x more facts under pairwise enumeration
...
Worst case: widest cluster (t=0, N=11) fully scatters
  pairwise-drift facts (all C(N,2) pairs break): 55
  cluster-level facts: 1
```

The measurement settled the split on candidate granularity: pairwise
enumeration produces 4.9x the facts of cluster-level reporting on a
single-mover edit, and 55x on the worst-case full scatter of the fixture's
widest cluster (N=11) — flooding output on exactly the common case, since
14 of the fixture's 19 coincident instants are 3+-way.

That result also resolved the two questions the data didn't directly
measure: a cluster-shaped fact (moved-set vs. stayed-set) doesn't fit
ADR-0063's existing pairwise "equal-in-ref, unequal-in-current" fact
convention, so the mechanism decision moved to a new predicate rather than
extending ADR-0063's. And scoping that new predicate to all N≥2 boundary
coincidences (not just 3+) closes the correctness gap one juror raised
against suppression — the new predicate then fully subsumes slack-drift's
zero-distance case, so suppressing it introduces no reporting hole.

See [ticket #162](https://github.com/MBehtemam/Montagent/issues/162) for the
full ballots and the resulting design.
