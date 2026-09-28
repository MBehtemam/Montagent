#!/usr/bin/env python3
"""Re-executable check for #407's two numeric claims, plus the instant sets the
sheets are built from.

Claim A (about the COMMITTED fixture, and the whole reason this ticket changed
shape): of the 14 keyframe change points `fixtures/en-halloween-decorating`
declares, 7 sit exactly on a run boundary that the run-start rule already tiles,
and 7 lie outside the lifetime of the element that declares them. The number
INTERIOR TO A RUN ON A VISIBLE ELEMENT is 0 -- so ADR-0094's keyframe flag adds
no tile to this fixture, and the "count of untiled keyframe change points" its
disclosure mandates would print 14 where the honest count is 0.

Claim C: ADR-0095's 140 px floor admits 30 whole-frame tiles, so an 18-state
document has room for 12 keyframe tiles; the doctored population's 13 puts the
sheet at 138.4 px and the call REFUSES.

Claim B (about `doctored/`, written by make_kf_project.py): the authored interior
population is 13 keyframe instants on top of 18 run-start instants = 31 tiles,
which crosses ADR-0095's 30-tile / 140 px floor.

Exits non-zero the moment either stops holding.
"""
import json, math, os, subprocess, sys

SP = os.path.dirname(os.path.abspath(__file__))
FIXTURE = os.path.join(SP, "../../../fixtures/en-halloween-decorating/en-halloween-decorating.montagent.json")
DOCTORED = os.path.join(SP, "doctored/en-halloween-decorating.montagent.json")
BIN = os.path.join(SP, "../../../target/release/montagent")
VISUAL = {"image", "text", "rect", "video"}

def cuts(project):
    out = subprocess.run([BIN, "query", project, "--from", "0", "--to",
                          str(json.load(open(project))["duration"]), "--json"],
                         capture_output=True, text=True, check=True).stdout
    return json.loads(out)["query"]["intervals"]

def runs(project):
    """ADR-0094 decision 1: filter to visual types, then re-merge on set equality."""
    out = []
    for i in cuts(project):
        s = frozenset(e["id"] for e in i["present"] if e["type"] in VISUAL)
        if out and out[-1][2] == s: out[-1][1] = i["end"]
        else: out.append([i["start"], i["end"], s])
    return out

def first_painted(a, b, fps):
    """ADR-0094 decision 2: least frame index whose painted ms falls inside [a, b)."""
    n = math.ceil(a * fps / 1000)
    while True:
        ms = math.floor(n * 1000 / fps)
        if ms >= b: return None
        if ms >= a: return ms
        n += 1

def keyframes(project):
    """(element id, property, t, present-at-t) for every keyframe in the document."""
    d = json.load(open(project))
    out = []
    for t in d["tracks"]:
        for e in t.get("elements", []):
            for k, v in e.items():
                if isinstance(v, list) and v and isinstance(v[0], dict) and "t" in v[0]:
                    for f in v:
                        out.append((e["id"], k, f["t"], e["start"] <= f["t"] < e["end"]))
    return out

def classify(project):
    d = json.load(open(project)); fps = d["fps"]
    rs = runs(project)
    bounds = {a for a, b, _ in rs}
    boundary_instants = [i for i in (first_painted(a, b, fps) for a, b, _ in rs) if i is not None]
    on_boundary, outside, interior = [], [], []
    for eid, prop, t, present in keyframes(project):
        if not present: outside.append((eid, prop, t))
        elif t in bounds: on_boundary.append((eid, prop, t))
        else:
            run = next(((a, b) for a, b, _ in rs if a < t < b), None)
            inst = first_painted(max(t, run[0]), run[1], fps) if run else None
            # the tile for a keyframe is the first painted frame at or after it
            n = math.ceil(t * fps / 1000); inst = math.floor(n * 1000 / fps)
            interior.append((eid, prop, t, inst if run and inst < run[1] else None))
    return dict(fps=fps, runs=rs, boundary_instants=boundary_instants,
                on_boundary=on_boundary, outside=outside, interior=interior)

def write_runs(project, path):
    """runs.json for make_sheets.py: each run's bounds plus ADR-0098 d.8's
    identifying field -- entered else departed, whole-document-span excluded,
    highest layer (latest declaration), id tie-break."""
    d = json.load(open(project)); dur = d["duration"]
    order, span, n = {}, {}, 0
    for t in d["tracks"]:
        for e in t.get("elements", []):
            order[e["id"]], span[e["id"]], n = n, (e["start"], e["end"]), n + 1
    out, prev = [], frozenset()
    for a, b, s in runs(project):
        cand = [(i, "+") for i in (s - prev)] or [(i, "-") for i in (prev - s)]
        cand = [(i, g) for i, g in cand if span[i] != (0, dur)] or cand
        i, g = max(cand, key=lambda c: (order[c[0]], c[0])) if cand else ("", "")
        out.append([a, b, f"{g}{i}"]); prev = s
    json.dump(out, open(path, "w"), indent=1)

def main():
    ok = True
    print("=== A. the committed fixture ===")
    c = classify(FIXTURE)
    print(f"visual states (runs): {len(c['runs'])}   run-start instants: {len(c['boundary_instants'])}")
    total = len(c["on_boundary"]) + len(c["outside"]) + len(c["interior"])
    print(f"keyframe change points declared: {total}")
    print(f"  on a run boundary, already tiled : {len(c['on_boundary'])}")
    print(f"  outside the declaring element    : {len(c['outside'])}   (schema's 'trimmed move')")
    print(f"  interior to a run, element visible: {len(c['interior'])}   <-- the only class a keyframe tile adds")
    for eid, prop, t in sorted(c["outside"], key=lambda r: r[2]):
        print(f"      trimmed: {eid}.{prop} t={t}")
    for a, e, n in (("runs", 18, len(c["runs"])), ("declared keyframes", 14, total),
                    ("on-boundary", 7, len(c["on_boundary"])), ("trimmed", 7, len(c["outside"])),
                    ("interior-on-visible", 0, len(c["interior"]))):
        if e != n: print(f"  FAIL claim A: {a} expected {e}, got {n}"); ok = False

    print("\n=== B. the doctored project ===")
    d = classify(DOCTORED)
    kf_instants = sorted({i for *_ , i in d["interior"] if i is not None})
    union = sorted(set(d["boundary_instants"]) | set(kf_instants))
    print(f"run-start instants          : {len(d['boundary_instants'])}")
    print(f"interior keyframes, visible : {len(d['interior'])}  -> {len(kf_instants)} distinct painted instants")
    for eid, prop, t, i in sorted(d["interior"], key=lambda r: r[2]):
        print(f"      {eid}.{prop} t={t} -> tile at {i}ms")
    print(f"run-start U keyframe        : {len(union)} tiles")
    for a, e, n in (("run-start", 18, len(d["boundary_instants"])),
                    ("interior keyframes", 13, len(d["interior"])),
                    ("union tile count", 31, len(union))):
        if e != n: print(f"  FAIL claim B: {a} expected {e}, got {n}"); ok = False

    print("\n=== C. what the flag costs, against ADR-0095's 140 px floor ===")
    sys.path.insert(0, SP)
    from make_sheets import best_grid, served, SRC_W, SRC_H, LABEL_FRAC
    cw, ch = SRC_W, SRC_H + int(SRC_H * LABEL_FRAC)
    widths = {}
    for n in range(16, 40):
        c, r = best_grid(n, cw, ch); sw, sh, _ = served(c * cw, r * ch)
        widths[n] = sw / c
    last_ok = max(n for n, w in widths.items() if w >= 140)
    print(f"  18 run-start tiles      -> {widths[18]:.1f} px  (ADR-0095's 180 px target)")
    print(f"  {len(union)} run-start U keyframe  -> {widths[len(union)]:.1f} px  "
          f"({'REFUSES' if widths[len(union)] < 140 else 'admissible'})")
    print(f"  last admissible count   -> {last_ok} tiles at {widths[last_ok]:.1f} px")
    print(f"  so the keyframe budget on an 18-state document is {last_ok - 18} keyframe tiles")
    for a, e, n in (("last admissible count", 30, last_ok),
                    ("union width < 140", True, widths[len(union)] < 140)):
        if e != n: print(f"  FAIL claim C: {a} expected {e}, got {n}"); ok = False

    write_runs(DOCTORED, os.path.join(SP, "runs.json"))
    json.dump(dict(boundary=d["boundary_instants"], keyframe=kf_instants, union=union,
                   interior=[(e, p, t, i) for e, p, t, i in d["interior"]]),
              open(os.path.join(SP, "instants.json"), "w"), indent=1)
    print("\nOK" if ok else "\nFAILED"); sys.exit(0 if ok else 1)

if __name__ == "__main__":
    main()
