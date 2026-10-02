"""#598's #299 case: write arm A (x/y keyframes) and arm B (origin point) beside the fixture."""
import json, copy, sys
from pathlib import Path
FX = Path(sys.argv[1])  # fixtures/en-halloween-decorating
base = json.loads((FX / "en-halloween-decorating.montagent.json").read_text())
P = (434, 920); C = (540, 956)
def photos(d):
    return [e for t in d["tracks"] for e in t["elements"] if e.get("type") == "image" and e["id"].startswith("photo-")]
def arm_b(d):
    for e in photos(d):
        e["origin"] = list(P); e["x"] = e["x"] - C[0] + P[0]; e["y"] = e["y"] - C[1] + P[1]
def arm_a(d, rnd):
    for e in photos(d):
        k0, k1 = e["scale"]
        for ax, c, p in (("x", C[0], P[0]), ("y", C[1], P[1])):
            v1 = e[ax] + (1 - k1["v"][0]) * (p - c)
            e[ax] = [dict(k0, v=e[ax]), dict(k1, v=round(v1) if rnd else round(v1, 3))]
for name, f in (("armB", arm_b), ("armA", lambda d: arm_a(d, False)), ("armA-int", lambda d: arm_a(d, True))):
    d = copy.deepcopy(base); f(d)
    (FX / f"kb-{name}.montagent.json").write_text(json.dumps(d, indent=1))
print(json.dumps([e for e in photos(json.loads((FX / "kb-armA.montagent.json").read_text())) if e["id"] == "photo-05"][0]))
