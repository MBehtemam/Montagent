#!/usr/bin/env python3
"""Re-derive the edit sizes ADR-0140 rests on, and fail if any stops reproducing.

    python3 docs/research/prototypes/swaps/edit_size_check.py

The request (`P-edit-error.md`, #597): insert `screens/error.png` between loading and done
for 1.2 s, keep done's length, and keep the push-in's pace (0.6 + 0.00005 ms⁻¹ · t).

Two sets of numbers, both counted as #597's `pip_score.py` counts them: changed lines of the
project pretty-printed with `indent=1, sort_keys=True`, against its seed.

1. **The nine agent runs** (`runs/`, the final project of each, copied from
   `prototype/swaps` at `7fd08e26`). Split 68 ×3; swaps with `end` optional 30, 26, 26;
   swaps with `end` required 29 ×3. Whether each run *landed* needs the prototype binary's
   `query --at`, so that verdict is the committed `runs/scores.txt`, not re-derived here.
2. **The minimum correct edit, by hand**, on four spellings of the same seed: the split, the
   split inside a container that carries the push-in (`seeds/group.hypothetical.json`, a
   shape #632 has not decided and no build reads), and the two swaps variants. The point of
   the container row is that swaps is not credited with the container's win. Split 66,
   container 35, swaps 24 (`end` optional) and 27 (`end` required); #597 reported 66, 24 and
   27 for the three it measured.
"""
import copy
import difflib
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
END, ERR, DONE = 7200, 3360, 4560
LAST_KEY = END - 40


def pace(t):
    return round(0.6 + 0.00005 * t, 3)


def lines(d):
    return json.dumps(d, indent=1, sort_keys=True).splitlines()


def changed(a, b):
    return sum(1 for l in difflib.unified_diff(lines(a), lines(b), n=0)
               if l[:1] in "+-" and l[:3] not in ("+++", "---"))


def load(name):
    return json.loads((HERE / name).read_text())


def els(d):
    return {e["id"]: e for t in d["tracks"] for e in t["elements"]}


def lengthen(e):
    """Stretch an element that runs to the old end, and its push-in, to the new end."""
    e["end"] = END
    if "scale" in e:
        e["scale"][-1].update(t=LAST_KEY, v=[pace(LAST_KEY)] * 2)


def ramp(a, b):
    return [{"t": a, "v": [pace(a)] * 2}, {"t": b - 40, "v": [pace(b - 40)] * 2, "ease": "linear"}]


def edit_split(d):
    d["duration"] = END
    e = els(d)
    lengthen(e["phone-body"])
    err = copy.deepcopy(e["screen-loading"])
    err.update(id="screen-error", start=ERR, end=DONE, source="screens/error.png", scale=ramp(ERR, DONE))
    screens = d["tracks"][1]["elements"]
    screens.insert(screens.index(e["screen-done"]), err)
    e["screen-done"].update(start=DONE, end=END, scale=ramp(DONE, END))


def edit_group(d):
    d["duration"] = END
    group = d["tracks"][0]["elements"][0]
    lengthen(group)
    kids = {e["id"]: e for e in group["elements"]}
    lengthen(kids["phone-body"])
    err = copy.deepcopy(kids["screen-loading"])
    err.update(id="screen-error", start=ERR, end=DONE, source="screens/error.png")
    group["elements"].insert(group["elements"].index(kids["screen-done"]), err)
    kids["screen-done"].update(start=DONE, end=END)


def edit_swaps(d):
    d["duration"] = END
    e = els(d)
    lengthen(e["phone-body"])
    lengthen(e["screen"])
    swaps = e["screen"]["swaps"]
    done = next(s for s in swaps if s["source"] == "screens/done.png")
    i = swaps.index(done)
    err = {"start": ERR, "source": "screens/error.png"}
    if "end" in done:
        err["end"] = DONE
        done["end"] = END
    swaps.insert(i, err)
    done["start"] = DONE


HAND = [
    ("split", "seeds/split.montagent.json", edit_split, 66),
    ("split in a container", "seeds/group.hypothetical.json", edit_group, 35),
    ("swaps, end optional", "seeds/swaps.montagent.json", edit_swaps, 24),
    ("swaps, end required", "seeds/swaps-ended.montagent.json", edit_swaps, 27),
]
RUNS = {1: 68, 2: 30, 3: 29, 4: 68, 5: 26, 6: 29, 7: 68, 8: 26, 9: 29}

failures = []
print("agent runs (changed lines against the run's seed):")
for n, want in RUNS.items():
    run = next(HERE.glob(f"runs/run-{n}.*.montagent.json"))
    spelling = run.name.split(".")[1]
    got = changed(load(f"seeds/{spelling}.montagent.json"), load(f"runs/{run.name}"))
    print(f"  run {n} {spelling:12} {got:3}  (claimed {want})")
    if got != want:
        failures.append(f"run {n}: {got} != {want}")

print("minimum correct edit, by hand:")
for label, seed, edit, want in HAND:
    before = load(seed)
    after = copy.deepcopy(before)
    edit(after)
    got = changed(before, after)
    print(f"  {label:22} {got:3}  (claimed {want})")
    if got != want:
        failures.append(f"{label}: {got} != {want}")

if failures:
    sys.exit("edit_size_check: does not reproduce: " + "; ".join(failures))
print("edit_size_check: all figures reproduce")
