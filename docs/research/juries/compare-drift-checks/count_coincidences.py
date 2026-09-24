#!/usr/bin/env python3
"""Counts boundary and keyframe-time coincidences in the real fixture.

Regenerates the numbers ADR-0063 cites: 92/120 coincident boundary endpoints
(76.7%), and the fact that only 7 elements carry any keyframed transform
property, all `scale`, with zero cross-element same-property keyframe-time
matches and 7/7 self-element (keyframe == own start) matches.

Run from the repo root: python3 docs/research/juries/compare-drift-checks/count_coincidences.py
"""
import json
import collections
import sys

FIXTURE = "fixtures/en-halloween-decorating/en-halloween-decorating.montagent.json"
TRANSFORM_PROPS = ["x", "y", "scale", "rotation", "opacity"]


def main():
    d = json.load(open(FIXTURE))
    tracks = d["tracks"]

    elements = []
    for tr in tracks:
        for el in tr["elements"]:
            elements.append((tr["name"], el))

    print("total elements:", len(elements))

    # Boundary coincidences
    boundary_map = collections.defaultdict(list)
    for _, el in elements:
        boundary_map[el["start"]].append((el["id"], "start"))
        boundary_map[el["end"]].append((el["id"], "end"))

    coincident_instants = {t: v for t, v in boundary_map.items() if len(v) > 1}
    total_boundaries = sum(len(v) for v in boundary_map.values())
    participating = sum(len(v) for v in coincident_instants.values())
    print("\n--- Boundary coincidences ---")
    print("distinct instants:", len(boundary_map))
    print("coincident instants (2+ boundaries):", len(coincident_instants))
    print("boundary endpoints participating in a coincidence:", participating, "/", total_boundaries)
    multi_partner = {t: v for t, v in coincident_instants.items() if len(v) > 2}
    print("instants with 3+ boundaries (multi-partner clusters):", len(multi_partner))

    # Keyframe coincidences: self (own element), cross (different elements)
    kf_map = collections.defaultdict(list)  # (prop, t) -> [(el_id, group)]
    elements_with_kf = []
    for _, el in elements:
        group = el.get("group")
        has_kf = False
        for prop in TRANSFORM_PROPS:
            val = el.get(prop)
            if isinstance(val, list) and val and isinstance(val[0], dict) and "t" in val[0]:
                has_kf = True
                for kf in val:
                    kf_map[(prop, kf["t"])].append((el["id"], group))
        if has_kf:
            elements_with_kf.append(el)

    print(f"\n--- Elements with any keyframed transform property: {len(elements_with_kf)} ---")

    self_matches = 0
    for el in elements_with_kf:
        for prop in TRANSFORM_PROPS:
            val = el.get(prop)
            if isinstance(val, list) and val and isinstance(val[0], dict):
                for kf in val:
                    if kf["t"] == el["start"] or kf["t"] == el["end"]:
                        self_matches += 1
                        print(f"  self-coincidence: {el['id']}.{prop} t={kf['t']} == own boundary")

    cross_element_hits = []
    for (prop, t), lst in kf_map.items():
        ids = set(x[0] for x in lst)
        if len(ids) > 1:
            cross_element_hits.append((prop, t, lst))

    print(f"\n--- Same-property keyframe-time coincidences (cross-element) ---")
    print("total cross-element same-property coincidences:", len(cross_element_hits))
    print("total self-element keyframe-to-own-boundary coincidences:", self_matches)


if __name__ == "__main__":
    sys.exit(main())
