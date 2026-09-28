#!/usr/bin/env python3
"""PROTOTYPE -- throwaway. Builds the doctored project #407 needs.

The pristine fixture cannot answer #407: every keyframe it carries on a *visible*
element sits exactly on a run boundary that is already tiled, and every other
keyframe it carries lies outside its element's own lifetime (the schema's
"trimmed move" spelling). So the keyframe-inclusive instant set is IDENTICAL to
the run-start set, and the two sheets the ticket asks for are the same sheet.
`classify_keyframes.py` proves that against the committed fixture.

To ask the ticket's question at all, an interior keyframe population has to be
authored. This script writes one, carrying six planted defects:

  D1  overshoot-crop   photo-06     scale 1.0 -> 1.85 -> 1.0 inside one run
  D2  off-canvas-drift sentence-07  x 540 -> 2000 -> 540 inside one run
  D3  step-snap        photo-05     scale 1.0 -> 1.08 with ease "step"
  D4  linear-control   photo-08     scale 1.0 -> 1.08 linear (NOT a defect --
                                    Juror 3's claim, here as the control)
  D5  wrong-photo      photo-07     source -> images/06.png   (the map's, presence-level)
  D6  invisible-text   sentence-08  color -> #1E344C          (the map's, presence-level)

plus seven DECORATIVE interior keyframes on the photos that host no defect, so
the tile-count cost of the flag is the cost a real animated project would pay
rather than the cost of the four defects alone.

Run:  python3 make_kf_project.py && python3 classify_keyframes.py
"""
import json, os

SP = os.path.dirname(os.path.abspath(__file__))
SRC = os.path.join(SP, "../../../fixtures/en-halloween-decorating/en-halloween-decorating.montagent.json")
OUT = os.path.join(SP, "doctored")

def kf(t, v, ease=None):
    d = {"t": t, "v": v}
    if ease: d["ease"] = ease
    return d

# (element, property, keyframe list, why)
SCALE = {
    # D3 -- step ease: holds 1.0, then snaps at 12000, interior to run 10468-17472
    "photo-05":       ([kf(3018, [1.0, 1.0]), kf(12000, [1.08, 1.08], "step")], "D3 step-snap"),
    # D1 -- overshoot: peaks at 1.85 at 20000 and returns, both interior to run 17472-22622
    "photo-06":       ([kf(17472, [1.0, 1.0]), kf(20000, [1.85, 1.85], "linear"),
                        kf(22500, [1.0, 1.0], "linear"), kf(26000, [1.02, 1.02], "linear")],
                       "D1 overshoot-crop (+1 decorative at 26000)"),
    # D4 -- the control: an honest linear ramp whose keyframe is interior to run 42763-47343
    "photo-08":       ([kf(42763, [1.0, 1.0]), kf(46000, [1.08, 1.08], "linear")], "D4 linear-control"),
    # decorative only -- cost, not signal
    "photo-05-intro": ([kf(0, [1.0, 1.0]), kf(1500, [1.03, 1.03], "linear")], "decorative"),
    "photo-07":       ([kf(30603, [1.0, 1.0]), kf(33000, [1.03, 1.03], "linear"),
                        kf(39000, [1.06, 1.06], "linear")], "decorative"),
    "photo-05-quiz":  ([kf(53856, [1.0, 1.0]), kf(55000, [1.02, 1.02], "linear"),
                        kf(62000, [1.06, 1.06], "linear")], "decorative"),
    "photo-05-loop":  ([kf(64016, [1.0, 1.0]), kf(64600, [1.02, 1.02], "linear")], "decorative"),
}
# D2 -- off-canvas drift, both keyframes interior to run 35753-42763
X = {"sentence-07": ([kf(35753, 540), kf(38000, 2000, "linear"), kf(41000, 540, "linear")],
                     "D2 off-canvas-drift")}

def main():
    d = json.load(open(SRC))
    touched = []
    for t in d["tracks"]:
        for e in t.get("elements", []):
            i = e["id"]
            if i in SCALE:
                e["scale"], why = SCALE[i][0], SCALE[i][1]; touched.append((i, "scale", why))
            if i in X:
                e["x"], why = X[i][0], X[i][1]; touched.append((i, "x", why))
            if i == "photo-07":
                e["source"] = "images/06.png"; touched.append((i, "source", "D5 wrong-photo"))
            if i == "sentence-08":
                e["color"] = "#1E344C"; touched.append((i, "color", "D6 invisible-text"))
    os.makedirs(OUT, exist_ok=True)
    for link in ("audio", "brand", "fonts", "images"):
        dst = os.path.join(OUT, link)
        if not os.path.lexists(dst):
            os.symlink(os.path.join("../../../../fixtures/en-halloween-decorating", link), dst)
    json.dump(d, open(os.path.join(OUT, "en-halloween-decorating.montagent.json"), "w"), indent=2)
    for i, p, why in touched:
        print(f"  {i:<16} {p:<8} {why}")

if __name__ == "__main__":
    main()
