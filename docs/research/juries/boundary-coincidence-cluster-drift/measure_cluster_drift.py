#!/usr/bin/env python3
"""Measures multi-partner boundary-coincidence clusters in the real fixture,
and simulates "one partner moves" to compare pairwise (C(N,2)) vs
cluster-level fact volume for a candidate compare predicate.

Run from the repo root:
  python3 docs/research/juries/boundary-coincidence-cluster-drift/measure_cluster_drift.py
"""
import json
import collections

FIXTURE = "fixtures/en-halloween-decorating/en-halloween-decorating.montagent.json"


def main():
    d = json.load(open(FIXTURE))
    tracks = d["tracks"]

    elements = []
    for tr in tracks:
        for el in tr["elements"]:
            elements.append((tr["name"], el))

    boundary_map = collections.defaultdict(list)
    for _, el in elements:
        boundary_map[el["start"]].append((el["id"], "start"))
        boundary_map[el["end"]].append((el["id"], "end"))

    coincident_instants = {t: v for t, v in boundary_map.items() if len(v) > 1}
    multi_partner = {t: v for t, v in coincident_instants.items() if len(v) > 2}

    widths = sorted(len(v) for v in multi_partner.values())
    print("--- Multi-partner cluster widths (N boundaries per coincident instant, N>=3) ---")
    print("cluster count:", len(multi_partner))
    print("widths:", widths)
    print("widest cluster:", max(widths) if widths else 0)
    print("width distribution:", collections.Counter(widths))

    print()
    print("--- Simulated 'one partner moves' per cluster: pairwise vs cluster-level fact volume ---")
    total_pairwise_before = 0
    total_pairwise_broken = 0
    total_cluster_facts = 0
    for t, members in sorted(multi_partner.items()):
        n = len(members)
        pairwise_total = n * (n - 1) // 2
        # Simulate exactly one member moving off the instant.
        broken_pairs = n - 1  # every pair involving the mover breaks
        total_pairwise_before += pairwise_total
        total_pairwise_broken += broken_pairs
        total_cluster_facts += 1  # one cluster-level fact reports the whole event
        print(
            f"  t={t}: N={n} boundaries -> C(N,2)={pairwise_total} total pairs in ref, "
            f"{broken_pairs} pairwise-drift facts if ONE partner moves, "
            f"vs 1 cluster-level fact"
        )

    print()
    print(f"Sum across all {len(multi_partner)} multi-partner clusters, one-mover-each scenario:")
    print(f"  pairwise-drift facts emitted: {total_pairwise_broken}")
    print(f"  cluster-level facts emitted:  {total_cluster_facts}")
    print(f"  ratio: {total_pairwise_broken / total_cluster_facts:.1f}x more facts under pairwise enumeration")

    # Worst case: widest cluster, only it moves fully apart (all N scatter to distinct times)
    if widths:
        widest_t = max(multi_partner, key=lambda t: len(multi_partner[t]))
        n = len(multi_partner[widest_t])
        print()
        print(f"--- Worst case: widest cluster (t={widest_t}, N={n}) fully scatters ---")
        print(f"  pairwise-drift facts (all C(N,2) pairs break): {n * (n - 1) // 2}")
        print(f"  cluster-level facts: 1")


if __name__ == "__main__":
    main()
