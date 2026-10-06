"""PROTOTYPE #722: glow, directional blur, grain and posterize under rotation, non-uniform
scale and a flip, with the bounds hint on and off, at one painter and at four."""

import json
import pathlib
import sys

HERE = pathlib.Path(__file__).parent
sys.path.insert(0, str(HERE))
import check  # noqa: E402
import make  # noqa: E402

CASES = {
    "glow": [make.blur(4), make.glow(0.4, 30, 2.0)],
    "glow-alone": [make.glow(0.4, 30, 2.0)],
    "directional_blur": [make.dblur(30, 41.5)],
    "grain": [make.blur(4), make.grain(3, 0.3, 2, False)],
    "posterize": [make.blur(6), make.posterize(4)],
}
TRANSFORMS = {
    "rotating": {"rotation": [{"t": 0, "v": 0}, {"t": 1000, "v": 33, "ease": "linear"}]},
    "scale[1.6,0.7]+rot": {"scale": [1.6, 0.7], "rotation": 12},
    "flip": {"scale": [-1, 1]},
}
lines = []
for tname, transform in TRANSFORMS.items():
    for cname, effects in CASES.items():
        e = make.text("s", 0, "GLOW", x=[{"t": 0, "v": 900}, {"t": 1000, "v": 917, "ease": "linear"}],
                      y=540, size=170, color="#FFD27A", effects=effects, end=1000, **transform)
        p = make.project([{"name": "s", "layer": 0, "elements": [e]}], 1000)
        path = HERE / "out" / "diag-rot.json"
        path.write_text(json.dumps(p).replace('"../../fixtures', '"../../../fixtures'))
        runs = {}
        for label, painting, extra in (
            ("hint-1", "1,1000", {}),
            ("nohint-1", "1,1000", {"MONTAGENT_PROTO_UNBOUND": "1"}),
            ("hint-4", "4,3", {}),
            ("nohint-4", "4,3", {"MONTAGENT_PROTO_UNBOUND": "1"}),
        ):
            h = HERE / "out" / f"dr-{label}.raw"
            check.render(path, hashes=h, painting=painting, extra=extra)
            runs[label] = h.read_text().split()
            h.unlink()
        base = runs["hint-1"]
        res = ", ".join(f"{k}: {sum(x == y for x, y in zip(v, base))}/{len(base)}" for k, v in runs.items()
                        if k != "hint-1")
        lines.append(f"{tname:20s} {cname:18s} vs hint at one painter: {res}")
        print(lines[-1], flush=True)
(HERE / "out" / "diag-rot.json").unlink()
import os  # noqa: E402

where = "own-hint/" if os.environ.get("MONTAGENT_PROTO_OWN_HINT") else ""
(HERE / "out" / f"{where}transforms-sweep.txt").write_text("\n".join(lines) + "\n")
