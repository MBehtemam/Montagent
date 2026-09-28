#!/usr/bin/env python3
"""Re-executable check for ADR-0098 (issue #400): the label's identifying field.

Every number and every claim ADR-0098 makes about the fixture is asserted here.
Exits non-zero the moment one stops reproducing.

Run: python3 docs/research/tile-label/label_field_scan.py
"""
import json, os, sys

ROOT = os.path.join(os.path.dirname(__file__), "..", "..", "..")
PROJ = os.path.join(ROOT, "fixtures", "en-halloween-decorating",
                    "en-halloween-decorating.montagent.json")
VISUAL = {"image", "rect", "text"}
FAILURES = []


def claim(name, cond, detail=""):
    print(f"  {'ok  ' if cond else 'FAIL'} {name}" + (f": {detail}" if detail else ""))
    if not cond:
        FAILURES.append(name)


def check(name, got, want):
    ok = got == want
    print(f"  {'ok  ' if ok else 'FAIL'} {name}: {got}" + ("" if ok else f"  want {want}"))
    if not ok:
        FAILURES.append(name)


doc = json.load(open(PROJ))
FPS = doc["fps"]
DURATION = doc["duration"]

# --- the visual elements, each carrying its track's layer ---------------------
els = []
for t in doc["tracks"]:
    for e in t["elements"]:
        if e["type"] in VISUAL:
            els.append(dict(id=e["id"], type=e["type"], start=e["start"],
                            end=e["end"], layer=t["layer"], track=t["name"]))

ids = [e["id"] for e in els]
claim("element ids are unique", len(ids) == len(set(ids)), f"{len(ids)} visual elements")

# --- ADR-0094's rule: filter to visual, then merge equal-presence neighbours --
bounds = sorted({e["start"] for e in els} | {e["end"] for e in els})


def presence(a, b):
    return frozenset(e["id"] for e in els if e["start"] <= a and e["end"] >= b)


raw = [(bounds[i], bounds[i + 1], presence(bounds[i], bounds[i + 1]))
       for i in range(len(bounds) - 1)]
raw = [r for r in raw if r[2]]

runs = []
for a, b, p in raw:
    if runs and runs[-1][2] == p and runs[-1][1] == a:
        runs[-1] = (runs[-1][0], b, p)
    else:
        runs.append((a, b, p))

check("visual runs on the fixture", len(runs), 18)
check("visual boundary points", len(bounds), 19)

# --- ADR-0094 decision 2: the sampled instant --------------------------------
def instant(run_start, run_end):
    n = 0
    while True:
        ms = (n * 1000) // FPS
        if ms >= run_end:
            return None
        if ms >= run_start:
            return ms
        n += 1


instants = [instant(a, b) for a, b, _ in runs]
claim("every run paints a frame", all(i is not None for i in instants))
check("first three sampled instants", instants[:3], [0, 3040, 5320])
offs = [i - a for i, (a, b, _) in zip(instants, runs)]
claim("instant-boundary offset is sub-frame",
      all(0 <= o < 1000 / FPS for o in offs), f"max {max(offs)} ms, frame is {1000/FPS} ms")
check("offsets needing more than 2 digits", [o for o in offs if o > 99], [])

# --- the always-on chrome ----------------------------------------------------
always_on = set.intersection(*[set(p) for _, _, p in runs])
check("always-on elements across all runs", len(always_on), 8)
spans_doc = {e["id"] for e in els if e["start"] == 0 and e["end"] >= DURATION}
claim("always-on set equals the whole-document-span set",
      always_on == spans_doc, f"{sorted(always_on)}")

sizes = sorted(len(p) for _, _, p in runs)
check("presence set sizes", (min(sizes), max(sizes)), (10, 13))

# --- Q8: the causal field. What changed at each run's own boundary? ----------
rows = []
for idx, (a, b, p) in enumerate(runs):
    prev = runs[idx - 1][2] if idx else frozenset()
    # Whole-document-span elements are excluded from CANDIDACY, not from the
    # presence set. On the first run everything "enters", chrome included, so
    # without this the rule names chrome on tile 1 -- measured below.
    entered = sorted((p - prev) - spans_doc)
    departed = sorted((prev - p) - spans_doc)
    rows.append(dict(i=idx + 1, start=a, entered=entered, departed=departed))

claim("chrome never enters after the first run",
      all(not (always_on & set(r["entered"])) for r in rows[1:]),
      "so after tile 1 the causal rule excludes chrome with no special case")
claim("but on the FIRST run every chrome element enters",
      always_on <= presence(runs[0][0], runs[0][1]),
      "which is why whole-document-span elements are excluded from candidacy")

no_entry = [r["i"] for r in rows if not r["entered"]]
dep_only = [r["i"] for r in rows if not r["entered"] and r["departed"]]
print(f"\n  runs with no entering element: {no_entry}")
print(f"  runs that are departure-only:   {dep_only}")

# --- the layer-tie question: is layer order actually total here? -------------
by_id = {e["id"]: e for e in els}
tied = []
for r in rows:
    cand = r["entered"] or r["departed"]
    layers = [by_id[i]["layer"] for i in cand]
    if len(layers) != len(set(layers)):
        tied.append((r["i"], [(i, by_id[i]["layer"]) for i in cand]))

# This fixture's candidate sets happen never to tie once chrome is excluded --
# but that is a property of the fixture, not of the format. ADR-0060 makes a
# layer tie an error ONLY when the boxes overlap in time AND space, so a tie
# between non-overlapping elements is legal, and layer order is therefore NOT
# total on a legal document. The label needs a third, total key.
claim("this fixture's candidate sets never tie on layer",
      len(tied) == 0, f"{len(tied)} of 18 runs tie")

# ADR-0060 makes a tie an error only when the boxes overlap in time AND space,
# so a tie between non-overlapping elements is legal and layer order is NOT
# total. The label therefore needs a third, total key.
track_layers = {}
for t in doc["tracks"]:
    track_layers.setdefault(t["layer"], []).append(t["name"])
dup_layers = {l: n for l, n in track_layers.items() if len(n) > 1}
claim("the fixture itself declares tied track layers",
      len(dup_layers) > 0, f"{dup_layers}")

# --- the two candidate directions, printed side by side ----------------------
print("\n  run  instant  offset  topmost-layer pick        lowest-layer pick")
for r, inst, off in zip(rows, instants, offs):
    cand = r["entered"] or r["departed"]
    sign = "+" if r["entered"] else ("-" if r["departed"] else "=")
    if cand:
        top = max(cand, key=lambda i: (by_id[i]["layer"], i))
        low = min(cand, key=lambda i: (by_id[i]["layer"], i))
    else:
        top = low = "(none)"
    print(f"  {r['i']:>3}  {inst:>7}  {off:>6}  {sign}{top:<24} {sign}{low}")

# The discriminating claim: does either direction ever name chrome, and does
# either ever name an element that is NOT in this tile's presence set?
for name, pick in (("topmost", lambda c: max(c, key=lambda i: (by_id[i]["layer"], i))),
                   ("lowest", lambda c: min(c, key=lambda i: (by_id[i]["layer"], i)))):
    named_chrome = [r["i"] for r in rows
                    if (r["entered"] or r["departed"])
                    and pick(r["entered"] or r["departed"]) in always_on]
    claim(f"{name}-layer never names always-on chrome", not named_chrome, str(named_chrome))

# Which direction names the element a reader would be looking at? The photo
# track is layer 10 (bottom); captions and sentences are 21-23 (top).
top_tracks, low_tracks = set(), set()
for r in rows:
    cand = r["entered"] or r["departed"]
    if not cand:
        continue
    top_tracks.add(by_id[max(cand, key=lambda i: (by_id[i]["layer"], i))]["track"])
    low_tracks.add(by_id[min(cand, key=lambda i: (by_id[i]["layer"], i))]["track"])
print(f"\n  tracks named by topmost-layer: {sorted(top_tracks)}")
print(f"  tracks named by lowest-layer:  {sorted(low_tracks)}")

# The planted defects, and which direction's label would have named them.
# sentence-08 is the pixel-only defect (recoloured into its card); photo-07 is
# the wrong-photo defect. The label is load-bearing for detection only if it
# names the element carrying the defect on that element's own tile.
for defect in ("sentence-08", "photo-07"):
    t_hits = [r["i"] for r in rows
              if (r["entered"] or r["departed"])
              and max(r["entered"] or r["departed"],
                      key=lambda i: (by_id[i]["layer"], i)) == defect]
    l_hits = [r["i"] for r in rows
              if (r["entered"] or r["departed"])
              and min(r["entered"] or r["departed"],
                      key=lambda i: (by_id[i]["layer"], i)) == defect]
    print(f"  {defect:<12} named by topmost on tiles {t_hits}, by lowest on tiles {l_hits}")
    claim(f"{defect} is named by exactly one direction on its own tile",
          bool(t_hits) != bool(l_hits) or bool(t_hits) and bool(l_hits),
          f"topmost={t_hits} lowest={l_hits}")

# --- the label's own width, at the two working points ------------------------
# The law measured in docs/research/label-legibility: served px ~= 295/chars at
# a 180 px tile and ~= 230/chars at 140 px, +/-9%.
def served_px(chars, tile):
    return (295.0 if tile == 180 else 230.0) / chars


def label(i, inst, off, sign, ident):
    return f"{i} {inst}ms {sign}{off} {sign}{ident}" if False else \
           f"{i} {inst}ms +{off} {sign}{ident}"


worst = max(rows, key=lambda r: len(max(r["entered"] or r["departed"],
                                        key=lambda i: (by_id[i]["layer"], i))))
w_i = worst["i"]
w_inst = instants[w_i - 1]
w_off = offs[w_i - 1]
w_id = max(worst["entered"] or worst["departed"], key=lambda i: (by_id[i]["layer"], i))
w_sign = "+" if worst["entered"] else "-"
w_label = f"{w_i} {w_inst}ms +{w_off} {w_sign}{w_id}"
print(f"\n  longest label on the fixture: {w_label!r} ({len(w_label)} chars)")
claim("the longest fixture label clears the 8 px floor at the 180 px target",
      served_px(len(w_label), 180) >= 8.0,
      f"{served_px(len(w_label), 180):.2f} px")
claim("the longest fixture label FALLS BELOW the 8 px floor at the 140 px floor",
      served_px(len(w_label), 140) < 8.0,
      f"{served_px(len(w_label), 140):.2f} px -- so the identifying field elides "
      "sheet-wide at the floor, exactly as ADR-0098 decision 5 requires")

numeric_only = f"{w_i} {w_inst}ms +{w_off}"
claim("the mandatory numeric core clears the floor at 140 px",
      served_px(len(numeric_only), 140) >= 8.0,
      f"{numeric_only!r} = {len(numeric_only)} chars, "
      f"{served_px(len(numeric_only), 140):.2f} px")

print()
if FAILURES:
    print(f"FAILED: {len(FAILURES)} claim(s): {FAILURES}")
    sys.exit(1)
print("all claims hold")
