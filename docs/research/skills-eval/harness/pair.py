"""Pair a phase's runs for blind judging.

    uv run docs/research/skills-eval/harness/pair.py [--phase verdict]

Per brief it makes the **parity** pairs (the reference against each with-skills run) and the
**lift** pairs (every with-skills run against every no-skills run). Every run gets a random
name, every pair a random left/right order, and the new pairs are shuffled together across
kinds and briefs. It writes, under `judging/<phase>/`:

- `pairs.json`: the order to judge in, by random name only. Nothing in it says which arm a
  name is, so the judging page and the court can read it without unblinding anyone.
- `key.json`: which run each name is. The judge does not open it until judging is done.

A run that delivered no video has no clip to show; each of its pairs is decided against it
here (`"auto"`), as the rubric pre-registers, and never reaches a judge.

Re-running adds pairs only for runs not yet paired (the top-up to 5 runs per arm) and never
touches existing pairs or names, so ballots already cast stay valid.
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
from common import JUDGING, PHASES, REPO, RUNS, read_json, rel, write_json  # noqa: E402

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


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--phase", choices=PHASES, default="verdict")
    args = ap.parse_args()

    runs = inventory(args.phase)
    if not runs:
        sys.exit(f"no runs under {rel(RUNS / args.phase)}")
    if args.phase == "verdict":
        commits = {read_json(r / "manifest.json")["pins"]["montagent_commit"]
                   for arms in runs.values() for rs in arms.values() for r in rs}
        if len(commits) != 1:
            sys.exit(f"verdict runs span {len(commits)} Montagent commits; they must share one pinned build")

    out = JUDGING / args.phase
    key_path, pairs_path = out / "key.json", out / "pairs.json"
    key: dict[str, str] = read_json(key_path) if key_path.exists() else {}
    pairs: list[dict] = read_json(pairs_path) if pairs_path.exists() else []
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

    new = []
    for brief, arms in sorted(runs.items()):
        refs, withs, nos = arms.get("reference", []), arms.get("with-skills", []), arms.get("no-skills", [])
        if len(refs) != 1:
            sys.exit(f"{brief}: {len(refs)} reference runs; the design has exactly one build per brief")
        for a, b in [(refs[0], w) for w in withs] + list(product(withs, nos)):
            pair = [name(a), name(b)]
            if frozenset(pair) in seen:
                continue
            RNG.shuffle(pair)
            left, right = pair
            entry = {"brief": brief, "left": left, "right": right}
            runs_lr = [key[left], key[right]]
            ok = [delivered(REPO / r) for r in runs_lr]
            if not all(ok):
                entry["auto"] = "equal" if not any(ok) else ("left" if ok[0] else "right")
            new.append(entry)
    RNG.shuffle(new)
    for i, entry in enumerate(new, start=len(pairs) + 1):
        pairs.append({"id": f"p{i:03d}", **entry})

    write_json(key_path, key)
    write_json(pairs_path, pairs)
    judged = sum("auto" not in p for p in new)
    print(f"{len(new)} new pairs ({judged} to judge, {len(new) - judged} decided by a missing video); "
          f"{len(pairs)} in {rel(pairs_path)}")


if __name__ == "__main__":
    main()
