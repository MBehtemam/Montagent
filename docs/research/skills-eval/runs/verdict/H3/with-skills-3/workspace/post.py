#!/usr/bin/env python3
"""Bring the left arm in front of the head while its hand is at the beak.

The rig draws the head over the arms, which would hide a hand raised to the beak. While
the thinking pose is on (the arm starts rising at SWAP_IN, is back at its side at
SWAP_OUT) the left arm's two parts are drawn above the head, keeping their own order, but
under the beak and blink overlays, so the fingers rest beneath the moving beak.
At both swaps the arm hangs at the owl's side, clear of the head, so the swap is unseen.
"""
import json, sys
SWAP_IN, SWAP_OUT = 833, 4100
FRONT = {"owl-forearm_left": 16, "owl-upper_arm_left": 17}
LIFT = {"owl-mouth": 18, "owl-blink": 19}
p = json.load(open(sys.argv[1]))
for tr in p["tracks"]:
    if tr["name"] in LIFT:
        tr["layer"] = LIFT[tr["name"]]
    if tr["name"] in FRONT:
        (e,) = tr["elements"]
        a, b, c = dict(e), dict(e), dict(e)
        a["end"] = SWAP_IN
        b["id"] = e["id"] + "-front"; b["start"] = SWAP_IN; b["end"] = SWAP_OUT; b["layer"] = FRONT[tr["name"]]
        c["id"] = e["id"] + "-after"; c["start"] = SWAP_OUT
        tr["elements"] = [a, b, c]
json.dump(p, open(sys.argv[2], "w"), indent=1, ensure_ascii=False)
