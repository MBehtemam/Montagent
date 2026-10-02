"""Score #597's PiP edit runs: did the error screen go in, and what did the edit cost?

    python3 docs/research/prototypes/swaps/pip_score.py <montagent> <run dir> [...]

For each run: its spelling (the `SPELLING` file the batch wrote), cost, turns, which files
the agent edited and how, how many elements and pretty-printed lines of the project changed
against its seed, and `validate`'s counts. Then correctness, through the binary's own
`query --at` at every painted frame of the edited project: the screen showing must follow
home → tap → loading → error → done on the brief's times, and the screen's and the phone
body's scale must stay on the seed's pace (0.6 + 0.00005 ms⁻¹ · t), with no jump.
"""
import difflib
import json
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
EXPECTED = [("home", 0, 1200), ("tap", 1200, 1360), ("loading", 1360, 3360),
            ("error", 3360, 4560), ("done", 4560, 7200)]
FPS, DURATION = 25, 7200


def expected_screen(t):
    return next(name for name, a, b in EXPECTED if a <= t < b)


def pretty(d):
    return json.dumps(d, indent=1, sort_keys=True).splitlines()


def els(d):
    return {e["id"]: e for t in d["tracks"] for e in t["elements"]}


def query(montagent, project, t):
    out = subprocess.run([montagent, "query", str(project), "--at", str(t), "--json"],
                         capture_output=True, text=True)
    return json.loads(out.stdout)["query"]["stack"]


def score(montagent, run):
    m = json.loads((run / "manifest.json").read_text())
    spelling = (run / "SPELLING").read_text().strip()
    seed = json.loads((HERE / "seeds" / f"P-{spelling}" / "workspace" / "phone.montagent.json").read_text())
    ev = [json.loads(l) for l in (run / "transcript.jsonl").read_text().splitlines() if l.strip()]
    tools = [b for e in ev if e.get("type") == "assistant"
             for b in e["message"]["content"] if b.get("type") == "tool_use"]
    edits = [(b["name"], Path(b["input"].get("file_path", "")).name) for b in tools
             if b["name"] in ("Edit", "Write", "MultiEdit")]
    scripted = [b["input"].get("command", "")[:80] for b in tools
                if b["name"] == "Bash" and "montagent.json" in b["input"].get("command", "")
                and any(k in b["input"].get("command", "") for k in ("python", "jq", "sed", ">"))]
    res = m["signals"]["result"]
    out = {"run": run.name, "spelling": spelling, "cost": res.get("total_cost_usd"),
           "turns": res.get("num_turns"), "wall_s": m.get("wall_clock_s"),
           "delivered": bool(m.get("deliverable")), "warnings": m.get("delivery_warnings"),
           "edits": edits, "scripted_writes": scripted}
    project = run / "workspace" / "phone.montagent.json"
    if not project.is_file():
        out["verdict"] = "no project"
        return out
    d = json.loads(project.read_text())
    se, de = els(seed), els(d)
    diff = [l for l in difflib.unified_diff(pretty(seed), pretty(d), n=0)
            if l[:1] in "+-" and l[:3] not in ("+++", "---")]
    out.update(elements=(len(se), len(de)),
               elements_changed=sorted(i for i in set(se) | set(de) if se.get(i) != de.get(i)),
               diff_lines=len(diff), duration=d.get("duration"),
               bytes=(len(json.dumps(seed, separators=(",", ":"))), len(json.dumps(d, separators=(",", ":")))))
    v = subprocess.run([montagent, "validate", str(project)], capture_output=True, text=True)
    out["validate"] = v.stdout.splitlines()[0] if v.stdout else v.stderr[:200]

    wrong_screen, off_pace, frames = [], [], 0
    n = 0
    while (t := n * 1000 // FPS) < DURATION:
        n += 1
        frames += 1
        stack = query(montagent, project, t)
        shown = [e for e in stack if e.get("source", "") and "screens/" in (e.get("source") or "")]
        name = Path(shown[-1]["source"]).stem if shown else None
        if name != expected_screen(t):
            wrong_screen.append((t, name))
        want = 0.6 + 0.00005 * t
        if not any(e["id"] == "phone-body" for e in stack):
            off_pace.append((t, "phone-body", "absent"))
        for e in stack:
            if e["id"] == "phone-body" or e in shown[-1:]:
                scale = next((r["value"] for r in e["values"] if r["property"] == "scale"), None)
                if scale is None or abs(scale[0] - want) > 1e-3:
                    off_pace.append((t, e["id"], scale and round(scale[0], 4)))
    out.update(frames=frames, wrong_screen=wrong_screen[:6], wrong_screen_n=len(wrong_screen),
               off_pace=off_pace[:6], off_pace_n=len(off_pace))
    out["verdict"] = "landed" if (not wrong_screen and not off_pace and out["delivered"]) else "missed"
    return out


if __name__ == "__main__":
    montagent = sys.argv[1]
    for r in sys.argv[2:]:
        print(json.dumps(score(montagent, Path(r)), indent=1))
