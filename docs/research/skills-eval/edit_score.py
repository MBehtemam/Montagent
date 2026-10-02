"""Score edit-test runs (#593): what the agent changed against the seed, and how.

    python3 docs/research/skills-eval/edit_score.py <run dir> [...]
    EDIT_SCORE_SEED=<seed project> python3 ... (#598's arm B, seeded from the unpadded owl)

Run dirs are relative to the repo root. For each run it prints cost, turns, how many times
the agent ran bake_rig.py, which files it edited with Edit/Write, how many elements and
pretty-printed lines of the project changed against the seed, when each arm first leaves its
rest angle (seed, now), the voice element, the mouth count and last mouth end, the duration,
and which non-owl elements changed.
"""
import json, os, sys, difflib
from pathlib import Path
WT = Path(__file__).resolve().parents[3]
SEED = WT / os.environ.get("EDIT_SCORE_SEED", "docs/research/skills-eval/runs/dev/C-character-short/with-skills-1/workspace/hoot.montagent.json")
def els(d): return {e["id"]: e for t in d["tracks"] for e in t["elements"]}
def first_move(e, rest_tol=0.5):
    v = e.get("rotation")
    if not isinstance(v, list): return None
    base = v[0]["v"]
    for k in v:
        if abs(k["v"] - base) > rest_tol: return k["t"]
def lines(d): return json.dumps(d, indent=1, sort_keys=True).splitlines()
seed = json.load(open(SEED)); se = els(seed)
for run in sys.argv[1:]:
    r = WT / run; m = json.load(open(r / "manifest.json"))
    ev = [json.loads(l) for l in (r / "transcript.jsonl").read_text().splitlines() if l.strip()]
    cmds = [b["input"].get("command", "") for e in ev if e.get("type") == "assistant"
            for b in e["message"]["content"] if b.get("type") == "tool_use" and b["name"] == "Bash"]
    edits = [b["input"].get("file_path", "") for e in ev if e.get("type") == "assistant"
             for b in e["message"]["content"] if b.get("type") == "tool_use" and b["name"] in ("Edit", "Write", "MultiEdit")]
    proj = r / "workspace" / (m.get("project") or "hoot.montagent.json")
    out = {"run": run, "cost": m["signals"]["result"]["total_cost_usd"], "turns": m["signals"]["result"]["num_turns"],
           "wall_s": m["wall_clock_s"], "delivered": bool(m["deliverable"]), "project": m.get("project"),
           "rebakes": sum("bake_rig.py" in c for c in cmds),
           "hand_edits": sorted(set(Path(p).name for p in edits)),
           "warnings": m["delivery_warnings"], "killed": m["signals"].get("background_tasks_killed")}
    if proj.is_file():
        d = json.load(open(proj)); de = els(d)
        changed = sorted(i for i in set(se) | set(de) if se.get(i) != de.get(i))
        diff = [l for l in difflib.unified_diff(lines(seed), lines(d), n=0) if l[:1] in "+-" and l[:3] not in ("+++", "---")]
        out.update(elements_changed=len(changed), elements_now=len(de), diff_lines=len(diff),
                   changed_sample=changed[:8])
        for arm in ("owl-forearm_left", "owl-forearm_right", "owl-upper_arm_left", "owl-upper_arm_right"):
            if arm in de: out[f"move:{arm}"] = (first_move(se[arm]), first_move(de[arm]))
        v0, v1 = se["voice"], de.get("voice", {})
        out["voice"] = ((v0["start"], v0["end"], v0.get("source")), (v1.get("start"), v1.get("end"), v1.get("source")))
        mouths = [e for i, e in de.items() if i.startswith("owl-mouth")]
        out["mouths"] = (len([i for i in se if i.startswith("owl-mouth")]), len(mouths),
                         max((e["end"] for e in mouths), default=None))
        out["duration"] = (seed.get("duration"), d.get("duration"))
        static = [i for i in changed if i in de and i in se and not i.startswith(("owl-", "voice"))]
        out["non_owl_changed"] = static
    print(json.dumps(out, indent=1))
