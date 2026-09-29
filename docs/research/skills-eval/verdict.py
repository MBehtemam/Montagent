"""Re-derive the skills eval's verdict from the committed files, and check it still follows.

    uv run docs/research/skills-eval/verdict.py [--phase verdict] [--write]

It reads only what is committed under this directory, plus the pinned commit's build:

1. **Preconditions.** For the verdict phase: every run shares one Montagent commit and one
   Claude Code version; `RUBRIC.md` and `briefs/held-out.sha256` were committed before the
   first run started; every run's brief is one of the sealed hashes; no run shows an
   isolation problem.
2. **Signals, per run** (reported, never decisive): whether a video was delivered, and its
   720p copy decodes; `validate` counts and reach (`query --census type`), re-run with the
   pinned binary on the committed workspace laid over the pinned pack; Montagent verbs used,
   `frame`/`preview` loops, whether the run looked at its own pictures; turns, wall-clock,
   tokens and cost; whether the with-skills run triggered its skills.
3. **Tally.** The human's ballots (`judging/<phase>/ballots/human.json`, plus the pairs
   decided by a missing video) mapped through `key.json` into parity and lift results, and
   the pre-registered rule in `RUBRIC.md` applied: parity passes when the reference is picked
   as better in at most 40% of parity pairs; lift passes when with-skills wins more lift
   pairs than it loses. A result that a single changed ballot would flip calls for the top-up
   to 5 runs per arm. The court's ballots are tallied the same way and set beside the
   human's, with every disagreement listed.

The result is written to `judging/<phase>/verdict.json` with `--write`. Without it, the
script compares what it derives against that committed file and exits non-zero if they
differ, or if the file is missing: the committed verdict must keep following from the
committed evidence.
"""

# /// script
# requires-python = ">=3.11"
# ///

from __future__ import annotations

import argparse
import json
import shutil
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent / "harness"))
from common import (  # noqa: E402
    EVAL, JUDGING, PHASES, REPO, RUNS, ffprobe, isolation_problems, load_pins, montagent_signals,
    pinned_build, read_json, read_jsonl, rel, sh, skill_names, transcript_signals, write_json,
)

PARITY_MAX_REFERENCE_SHARE = 0.40
TOP_UP_RUNS = 5
VOTES = ("left", "right", "equal")


# --- preconditions --------------------------------------------------------------------


def first_commit_time(path: Path) -> str | None:
    out = sh("git", "log", "--diff-filter=A", "--format=%cI", "--", path, cwd=REPO, check=False).stdout.split()
    return out[-1] if out else None


def preconditions(phase: str, manifests: list[dict], pins: dict) -> list[str]:
    problems = []
    for m in manifests:
        for p in m.get("isolation_problems", []):
            problems.append(f"{m['brief']['id']}/{m['run']}: {p}")
    if phase != "verdict":
        return problems
    for field in ("montagent_commit", "claude_code_version", "model"):
        values = sorted({m["pins"][field] for m in manifests})
        if len(values) > 1:
            problems.append(f"runs differ in {field}: {values}")
    first_run = min((m["started"] for m in manifests), default=None)
    sealed = EVAL / "briefs" / "held-out.sha256"
    for path in (EVAL / "RUBRIC.md", sealed):
        t = first_commit_time(path)
        if t is None:
            problems.append(f"{rel(path)} is not committed")
        elif first_run and t > first_run:
            problems.append(f"{rel(path)} was first committed at {t}, after the first run ({first_run})")
    hashes = set(sealed.read_text().split()) if sealed.exists() else set()
    for m in manifests:
        if m["brief"]["sha256"] not in hashes:
            problems.append(f"{m['brief']['id']}/{m['run']}: brief is not one of the sealed held-out briefs")
    return problems


# --- signals --------------------------------------------------------------------------


def run_signals(run: Path, man: dict, builds: dict, pins: dict) -> dict:
    commit = man["pins"]["montagent_commit"]
    if commit not in builds:
        builds[commit] = pinned_build(commit)
    build = builds[commit]
    ours = skill_names(build["skills"]) if man["arm"] == "with-skills" else []
    sig = transcript_signals(read_jsonl(run / "transcript.jsonl"), ours)

    render = run / "render.mp4"
    probe = ffprobe(render) if render.exists() else None
    montagent = None
    if man.get("project"):
        with tempfile.TemporaryDirectory(prefix="montagent-eval-verdict-") as tmp:
            work = Path(tmp) / "work"
            shutil.copytree(build["pack"], work)
            if (run / "workspace").is_dir():
                shutil.copytree(run / "workspace", work, dirs_exist_ok=True)
            project = work / man["project"]
            if project.exists():
                montagent = montagent_signals(build["bin"], project)

    recorded = (man.get("signals") or {}).get("montagent")
    return {
        "arm": man["arm"],
        "delivered": probe is not None,
        "render_720p": probe,
        "cap_hit": man.get("timed_out") or sig["result"]["subtype"] not in ("success", None),
        "timed_out": man.get("timed_out"),
        "isolation_problems": isolation_problems(sig, man["arm"], ours, pins["builtin_skills"]),
        "skills_triggered": sig["skills_triggered"] if man["arm"] == "with-skills" else None,
        "montagent": montagent,
        # validate reads the disk; files over the kept-size limit are not committed, so a
        # re-derived count can differ from the one taken in the run's own directory.
        "montagent_matches_run_time": None if recorded is None else recorded == montagent,
        "montagent_verbs": sig["montagent_verbs"],
        "look_loops": sig["look_loops"],
        "looked_at": sig["looked_at"],
        "result": sig["result"],
    }


# --- tally ----------------------------------------------------------------------------


def arm_of(name: str, key: dict, arms: dict) -> str:
    return arms[key[name]]


def outcomes(pairs: list[dict], votes: dict[str, str], key: dict, arms: dict) -> dict:
    """Parity and lift counts from one set of votes ({pair id: left|right|equal})."""
    parity = {"reference_better": 0, "with_skills_better": 0, "equal": 0, "pairs": 0}
    lift = {"with_skills_wins": 0, "no_skills_wins": 0, "equal": 0, "pairs": 0}
    for p in pairs:
        v = votes.get(p["id"])
        if v not in VOTES:
            continue
        a, b = arm_of(p["left"], key, arms), arm_of(p["right"], key, arms)
        winner = None if v == "equal" else (a if v == "left" else b)
        kinds = {a, b}
        if kinds == {"reference", "with-skills"}:
            parity["pairs"] += 1
            parity[{"reference": "reference_better", "with-skills": "with_skills_better", None: "equal"}[winner]] += 1
        elif kinds == {"with-skills", "no-skills"}:
            lift["pairs"] += 1
            lift[{"with-skills": "with_skills_wins", "no-skills": "no_skills_wins", None: "equal"}[winner]] += 1
        else:
            raise SystemExit(f"pair {p['id']} pits {a} against {b}, which the design never pairs")
    return {"parity": parity, "lift": lift}


def passes(o: dict) -> dict:
    par, lift = o["parity"], o["lift"]
    return {
        "parity": par["pairs"] > 0 and par["reference_better"] <= PARITY_MAX_REFERENCE_SHARE * par["pairs"],
        "lift": lift["with_skills_wins"] > lift["no_skills_wins"],
    }


def near(pairs, votes, key, arms, human_ids) -> dict:
    """Would changing any single human ballot flip either result?"""
    base = passes(outcomes(pairs, votes, key, arms))
    flips = {"parity": False, "lift": False}
    for pid in human_ids:
        for alt in VOTES:
            if alt == votes[pid]:
                continue
            changed = passes(outcomes(pairs, {**votes, pid: alt}, key, arms))
            for k in flips:
                flips[k] |= changed[k] != base[k]
    return flips


def tally(phase: str, runs: dict[str, dict]) -> dict:
    out = JUDGING / phase
    if not (out / "pairs.json").exists():
        return {"status": "unpaired"}
    key, pairs = read_json(out / "key.json"), read_json(out / "pairs.json")
    arms = {r: s["arm"] for r, s in runs.items()}
    missing_runs = sorted(set(key.values()) - set(arms))
    if missing_runs:
        raise SystemExit(f"key.json names runs that are not recorded: {missing_runs}")
    human_path = out / "ballots" / "human.json"
    human = {pid: b["vote"] for pid, b in (read_json(human_path) if human_path.exists() else {}).items()}
    auto = {p["id"]: p["auto"] for p in pairs if "auto" in p}
    votes = {**human, **auto}
    unjudged = [p["id"] for p in pairs if p["id"] not in votes]

    o = outcomes(pairs, votes, key, arms)
    result = {**o, "passes": passes(o), "auto_decided": len(auto), "unjudged": unjudged}
    runs_per_arm = {}
    for s in runs.values():
        runs_per_arm[s["arm"]] = runs_per_arm.get(s["arm"], 0) + 1
    briefs = len({p["brief"] for p in pairs}) or 1
    topped_up = all(runs_per_arm.get(a, 0) >= TOP_UP_RUNS * briefs for a in ("with-skills", "no-skills"))
    if unjudged:
        result["status"] = "incomplete"
    else:
        result["within_one_pick"] = near(pairs, votes, key, arms, list(human))
        if any(result["within_one_pick"].values()) and not topped_up:
            result["status"] = "top-up"
        else:
            result["status"] = "pass" if all(result["passes"].values()) else "fail"

    court = {}
    for path in sorted((out / "ballots" / "court").glob("*.json")):
        cv = {pid: b.get("vote") for pid, b in read_json(path).items()}
        co = outcomes(pairs, {**cv, **auto}, key, arms)
        court[path.stem] = {
            **co, "passes": passes(co),
            "disagrees_with_human": sorted(pid for pid, v in cv.items() if pid in human and v != human[pid]),
        }
    result["court"] = court
    return result


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--phase", choices=PHASES, default="verdict")
    ap.add_argument("--write", action="store_true", help="write judging/<phase>/verdict.json")
    args = ap.parse_args()

    pins = load_pins()
    manifests = {m.parent: read_json(m) for m in sorted((RUNS / args.phase).glob("*/*/manifest.json"))}
    if not manifests:
        sys.exit(f"no runs under {rel(RUNS / args.phase)}")

    builds: dict = {}
    runs = {rel(run): run_signals(run, man, builds, pins) for run, man in manifests.items()}
    verdict = {
        "phase": args.phase,
        "rule": {"parity_max_reference_share": PARITY_MAX_REFERENCE_SHARE,
                 "lift": "with-skills wins > no-skills wins", "top_up_runs_per_arm": TOP_UP_RUNS},
        "preconditions": preconditions(args.phase, list(manifests.values()), pins),
        "runs": runs,
        "tally": tally(args.phase, runs),
    }
    if verdict["preconditions"] and verdict["tally"].get("status") in ("pass", "fail", "top-up"):
        verdict["tally"]["status"] = "invalid"

    path = JUDGING / args.phase / "verdict.json"
    if args.write:
        write_json(path, verdict)
        print(f"wrote {rel(path)}")
    t = verdict["tally"]
    print(json.dumps({"status": t.get("status"), "passes": t.get("passes"),
                      "parity": t.get("parity"), "lift": t.get("lift"),
                      "preconditions": verdict["preconditions"]}, indent=2))
    if args.write:
        return
    if not path.exists():
        sys.exit(f"{rel(path)} is not committed; run with --write once the ballots are in")
    if read_json(path) != json.loads(json.dumps(verdict, sort_keys=True)):
        sys.exit(f"{rel(path)} no longer follows from the committed evidence; re-derive with --write and review the diff")
    print(f"{rel(path)} follows from the committed evidence")


if __name__ == "__main__":
    main()
