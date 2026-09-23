#!/usr/bin/env python3
"""Re-derives every number in ADR-0086 and exits non-zero if any stops holding.

Run: python3 docs/adr/recorded_intent_scan.py

Four claims, asserted in both directions where the ADR's decision turns on the
boundary rather than on the count alone:

  1. 7 of 7 Ken Burns ramps carry an exact intra-element offset of 15000 ms
     between their two `scale` keyframes, and each ramp's first keyframe sits
     exactly on its element's `start`.
  2. All 7 ramps run past their own element's `end`, so the offset is a
     same-rate claim and cannot be read as same-endpoint.
  3. 8 of 8 repeat-reading pairs carry a fixed inter-element gap, bimodal and
     exceptionless: 800 ms on all 4 word pairs, 520 ms on all 4 sentence pairs.
  4. 11 of 22 text elements overlap 2 or more audio sources (ADR-0086 carries
     this forward from #166 unchanged; it is re-derived here so the two ADRs
     cannot drift apart).

It also records the erratum ADR-0086 raises against ADR-0037: the four values
that ADR cites as its evidence appear in no committed version of the fixture.
That is asserted here against the working tree only; the git history check is
stated in ADR-0086 and reproduced by:

    for c in $(git log --format=%H --follow -- <fixture>); do \
        git show $c:<fixture> | grep -c '8000\\|16800\\|25700\\|35300'; done
"""

import json
import os
import re
import sys
from collections import Counter

FIXTURE = os.path.join(
    os.path.dirname(os.path.abspath(__file__)),
    "..",
    "..",
    "fixtures",
    "en-halloween-decorating",
    "en-halloween-decorating.montaget.json",
)

KEYFRAMED = ("scale", "rotation", "opacity", "x", "y")
RAMP_OFFSET_MS = 15000
ADR_0037_CLAIMED_VALUES = ("8000", "16800", "25700", "35300")

failures = []


def check(label, actual, expected):
    ok = actual == expected
    print(f"{'ok  ' if ok else 'FAIL'}  {label}: {actual!r}")
    if not ok:
        failures.append(f"{label}: expected {expected!r}, got {actual!r}")


def elements(doc):
    for track in doc["tracks"]:
        for el in track["elements"]:
            yield el


def keyframes(el, prop):
    v = el.get(prop)
    if isinstance(v, list) and v and isinstance(v[0], dict):
        return v
    return []


def main():
    with open(FIXTURE) as fh:
        raw = fh.read()
    doc = json.loads(raw)
    els = list(elements(doc))
    by_id = {e["id"]: e for e in els}

    # ---- Claim 1: the Ken Burns ramp is an exact intra-element fixed offset.
    ramps = [(e, keyframes(e, "scale")) for e in els if keyframes(e, "scale")]
    check("ramps carrying a scale keyframe list", len(ramps), 7)
    check(
        "ramps whose keyframe pair is exactly RAMP_OFFSET_MS apart",
        sum(1 for _, kf in ramps if len(kf) == 2 and kf[1]["t"] - kf[0]["t"] == RAMP_OFFSET_MS),
        7,
    )
    check(
        "ramps whose first keyframe sits exactly on the element's start",
        sum(1 for e, kf in ramps if kf[0]["t"] == e["start"]),
        7,
    )
    # The other direction: no ramp uses any other offset. A second offset value
    # appearing would make "one published rule argument" the wrong shape.
    check(
        "distinct ramp offsets in the fixture",
        sorted({kf[1]["t"] - kf[0]["t"] for _, kf in ramps if len(kf) == 2}),
        [RAMP_OFFSET_MS],
    )

    # ---- Claim 2: the ramp is a rate claim, not an endpoint claim.
    # If the ramp's final keyframe landed on the element's end, "same-endpoint"
    # would be an equally good reading of the author's intent and ADR-0036's
    # half-binding objection would still bite. It does not.
    check(
        "ramps whose final keyframe runs past the element's own end",
        sum(1 for e, kf in ramps if kf[-1]["t"] > e["end"]),
        7,
    )
    check(
        "ramps whose final keyframe lands exactly on the element's end",
        sum(1 for e, kf in ramps if kf[-1]["t"] == e["end"]),
        0,
    )

    # ---- Claim 3: the repeat-reading gap is fixed, bimodal and exceptionless.
    pairs = []
    for eid in by_id:
        m = re.match(r"^(.*)-a$", eid)
        if m and m.group(1) + "-b" in by_id:
            a, b = by_id[eid], by_id[m.group(1) + "-b"]
            pairs.append((eid, b["start"] - a["end"]))
    check("repeat-reading a/b pairs found", len(pairs), 8)
    check("pairs with a non-positive gap", sum(1 for _, g in pairs if g <= 0), 0)
    gaps = Counter(g for _, g in pairs)
    check("gap histogram over all pairs", dict(gaps), {800: 4, 520: 4})
    check(
        "word pairs, all at 800 ms",
        sorted(g for i, g in pairs if "-word-" in i),
        [800] * 4,
    )
    check(
        "sentence pairs, all at 520 ms",
        sorted(g for i, g in pairs if "-sentence-" in i),
        [520] * 4,
    )

    # ---- Claim 3b: every instant in those relationships is unsignatured —
    # its value appears exactly once in the whole file, so nothing records it.
    instants = []
    for e in els:
        instants += [e["start"], e["end"]]
        for prop in KEYFRAMED:
            instants += [r["t"] for r in keyframes(e, prop)]
    instants.append(doc["duration"])
    seen = Counter(instants)
    check("timeline instants in the fixture", len(instants), 135)
    pair_instants = []
    for eid, _ in pairs:
        pair_instants += [by_id[eid]["end"], by_id[eid[:-2] + "-b"]["start"]]
    check(
        "repeat-reading instants whose value appears exactly once",
        sum(1 for v in pair_instants if seen[v] == 1),
        16,
    )
    check(
        "ramp end-of-ramp instants whose value appears exactly once",
        sum(1 for _, kf in ramps if seen[kf[1]["t"]] == 1),
        7,
    )

    # ---- Claim 4: the audio axis, carried forward from #166 unchanged.
    text = [e for e in els if e["type"] == "text"]
    audio = [e for e in els if e["type"] == "audio"]
    check("text elements", len(text), 22)
    check(
        "text elements overlapping 2+ audio sources",
        sum(
            1
            for t in text
            if sum(1 for a in audio if a["start"] < t["end"] and t["start"] < a["end"]) >= 2
        ),
        11,
    )

    # ---- The ADR-0037 erratum.
    check(
        "ADR-0037's cited values present in the fixture",
        [v for v in ADR_0037_CLAIMED_VALUES if v in raw],
        [],
    )

    print()
    if failures:
        print(f"{len(failures)} claim(s) no longer hold:")
        for f in failures:
            print(f"  - {f}")
        return 1
    print("All ADR-0086 claims reproduce against the committed fixture.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
