"""PROTOTYPE #722: where `posterize: 256` differs from no member, against `brightness: 0`
(an existing identity colour filter) as the control."""

import copy
import json
import pathlib
import sys

HERE = pathlib.Path(__file__).parent
sys.path.insert(0, str(HERE))
import check  # noqa: E402

base = json.loads((HERE / "identity" / "posterize-256-alone-on.json").read_text())
variants = {
    "posterize-256": [{"name": "posterize", "levels": 256}],
    "none": [],
    "brightness-0": [{"name": "brightness", "amount": 0}],
}
frames = {}
for name, effects in variants.items():
    p = copy.deepcopy(base)
    for t in p["tracks"]:
        for e in t["elements"]:
            if e["id"] == "subject":
                if effects:
                    e["effects"] = effects
                else:
                    e.pop("effects", None)
    path = HERE / "identity" / f"diag-{name}.json"
    path.write_text(json.dumps(p, indent=1))
    d = HERE / "out" / "diag" / name
    check.render(path, dump=d)
    frames[name] = check.frames(d)[0].read_bytes()
    path.unlink()


def cmp(a, b):
    n, mx, samples = 0, 0, []
    for i in range(0, len(a), 3):
        if a[i:i + 3] != b[i:i + 3]:
            n += 1
            mx = max(mx, max(abs(a[i + j] - b[i + j]) for j in range(3)))
            if len(samples) < 4:
                samples.append(((i // 3) % 1920, (i // 3) // 1920, tuple(a[i:i + 3]), tuple(b[i:i + 3])))
    return f"{n} pixels differ, max {mx} levels, e.g. {samples}"


out = [
    "frame 0 of identity/posterize-256-alone (a rotated video at opacity 0.8 over footage):",
    "posterize 256 vs no member: " + cmp(frames["posterize-256"], frames["none"]),
    "brightness 0 vs no member:  " + cmp(frames["brightness-0"], frames["none"]),
    "posterize 256 vs brightness 0: " + cmp(frames["posterize-256"], frames["brightness-0"]),
]
(HERE / "out" / "posterize-identity.txt").write_text("\n".join(out) + "\n")
print("\n".join(out))
