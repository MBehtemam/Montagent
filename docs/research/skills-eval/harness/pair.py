"""Pair a phase's runs for blind judging.

    uv run docs/research/skills-eval/harness/pair.py [--phase verdict]

Per brief it makes the **parity** pairs (the reference against each with-skills run) and the
**lift** pairs. Every run gets a random name and every pair a random left/right order. It
writes, under `judging/<phase>/`:

- `pairs.json`: the order to judge in, by random name only. Nothing in it says which arm a
  name is, so the judging page and the court can read it without unblinding anyone.
- `key.json`: which run each name is. The judge does not open it until judging is done.

A run that delivered no video has no clip to show; each of its pairs is decided against it
here (`"auto"`), as the rubric pre-registers, and never reaches a judge.

**Every pair** (the first verdict and earlier phases): lift is every with-skills run against
every no-skills run, and the new pairs are shuffled together across kinds and briefs.
Re-running adds pairs only for runs not yet paired (the top-up to 5 runs per arm) and never
touches existing pairs or names, so ballots already cast stay valid.

**The cyclic sample** (`verdict-2`, whose pins set `lift_sample.design` to `cyclic`): a brief
is paired once, when all its runs are recorded (`runs_per_brief`). The with-skills runs are
put in a random order W[0..n-1] and, independently, the no-skills runs in N[0..n-1]; W[i]
meets N[i] and N[(i+1) % n], so every Montagent run is in exactly 2 lift pairs. That brief's
parity and lift pairs are shuffled together and appended as one block, its sitting. The
orders, by random name, go to `sample.json`, which names arms: keep it closed like the key.
Re-running pairs only briefs that have newly completed.
"""

# /// script
# requires-python = ">=3.11"
# ///

from __future__ import annotations

import argparse
import random
import secrets
import sys
from itertools import product
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from common import JUDGING, PHASES, REPO, RUNS, VERDICT_PHASES, load_pins, read_json, rel, write_json  # noqa: E402

RNG = random.SystemRandom()


def inventory(phase: str) -> dict[str, dict[str, list[Path]]]:
    """{brief: {arm: [run dirs]}} for every recorded run of the phase."""
    out: dict[str, dict[str, list[Path]]] = {}
    for m in sorted((RUNS / phase).glob("*/*/manifest.json")):
        man = read_json(m)
        out.setdefault(man["brief"]["id"], {}).setdefault(man["arm"], []).append(m.parent)
    return out


def delivered(run: Path) -> bool:
    return (run / "render.mp4").exists()


def cyclic(withs: list, nos: list) -> tuple[list, list, list[tuple]]:
    """Shuffle each arm and pair W[i] with N[i] and N[(i+1) % n]."""
    w, n = list(withs), list(nos)
    RNG.shuffle(w)
    RNG.shuffle(n)
    return w, n, [(w[i], n[(i + k) % len(n)]) for i in range(len(w)) for k in (0, 1)]


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--phase", choices=PHASES, default="verdict")
    args = ap.parse_args()

    pins = load_pins(args.phase)
    sampled = (pins.get("lift_sample") or {}).get("design") == "cyclic"
    runs = inventory(args.phase)
    if not runs:
        sys.exit(f"no runs under {rel(RUNS / args.phase)}")
    if args.phase in VERDICT_PHASES:
        commits = {read_json(r / "manifest.json")["pins"]["montagent_commit"]
                   for arms in runs.values() for rs in arms.values() for r in rs}
        if len(commits) != 1:
            sys.exit(f"{args.phase} runs span {len(commits)} Montagent commits; they must share one pinned build")

    out = JUDGING / args.phase
    key_path, pairs_path, sample_path = out / "key.json", out / "pairs.json", out / "sample.json"
    key: dict[str, str] = read_json(key_path) if key_path.exists() else {}
    pairs: list[dict] = read_json(pairs_path) if pairs_path.exists() else []
    sample: dict = read_json(sample_path) if sample_path.exists() else {}
    name_of = {run: name for name, run in key.items()}
    seen = {frozenset((p["left"], p["right"])) for p in pairs}

    def name(run: Path) -> str:
        r = rel(run)
        if r not in name_of:
            n = secrets.token_hex(4)
            while n in key:
                n = secrets.token_hex(4)
            key[n], name_of[r] = r, n
        return name_of[r]

    def entry(brief: str, a: Path, b: Path) -> dict:
        pair = [name(a), name(b)]
        RNG.shuffle(pair)
        left, right = pair
        e = {"brief": brief, "left": left, "right": right}
        ok = [delivered(REPO / key[left]), delivered(REPO / key[right])]
        if not all(ok):
            e["auto"] = "equal" if not any(ok) else ("left" if ok[0] else "right")
        return e

    new = []
    for brief, arms in sorted(runs.items()):
        refs, withs, nos = arms.get("reference", []), arms.get("with-skills", []), arms.get("no-skills", [])
        if not sampled:
            if len(refs) != 1:
                sys.exit(f"{brief}: {len(refs)} reference runs; the design has exactly one build per brief")
            for a, b in [(refs[0], w) for w in withs] + list(product(withs, nos)):
                if frozenset((name(a), name(b))) not in seen:
                    new.append(entry(brief, a, b))
            continue
        if brief in sample:
            continue
        want = pins["runs_per_brief"]
        have = {"reference": len(refs), "with-skills": len(withs), "no-skills": len(nos)}
        if have != want:
            print(f"{brief}: not paired yet, runs {have} of {want}")
            continue
        w, n, lift = cyclic(withs, nos)
        block = [entry(brief, refs[0], x) for x in w] + [entry(brief, a, b) for a, b in lift]
        RNG.shuffle(block)
        sample[brief] = {"with-skills": [name(x) for x in w], "no-skills": [name(x) for x in n]}
        new += block
    if not sampled:
        RNG.shuffle(new)
    for i, e in enumerate(new, start=len(pairs) + 1):
        pairs.append({"id": f"p{i:03d}", **e})

    write_json(key_path, key)
    write_json(pairs_path, pairs)
    if sampled:
        write_json(sample_path, sample)
    judged = sum("auto" not in p for p in new)
    print(f"{len(new)} new pairs ({judged} to judge, {len(new) - judged} decided by a missing video); "
          f"{len(pairs)} in {rel(pairs_path)}")


if __name__ == "__main__":
    main()
