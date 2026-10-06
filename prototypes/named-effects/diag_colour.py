"""PROTOTYPE #722: which colour filter changes bytes when let into the bound, and where.

Each member alone on one drifting gradient rect after a `blur`, rotating and not; the bound
with the colour filters let in (`MONTAGENT_PROTO_COLOUR_BOUND`) against the hint off.
"""

import copy
import json
import pathlib
import sys

HERE = pathlib.Path(__file__).parent
sys.path.insert(0, str(HERE))
import check  # noqa: E402
import make  # noqa: E402
import os  # noqa: E402

WHERE = "own-hint/" if os.environ.get("MONTAGENT_PROTO_OWN_HINT") else ""

MEMBERS = {
    "tint": {"name": "tint", "color": "#FF0000", "amount": 0.5},
    "saturation-0.2": {"name": "saturation", "amount": 0.2},
    "brightness+0.3": {"name": "brightness", "amount": 0.3},
    "brightness-0.3": {"name": "brightness", "amount": -0.3},
    "contrast+0.6": {"name": "contrast", "amount": 0.6},
    "contrast-0.6": {"name": "contrast", "amount": -0.6},
    "posterize-4": {"name": "posterize", "levels": 4},
}
lines = []
for rotating in (False, True):
    for name, member in MEMBERS.items():
        e = make.rect("c", 0, x=[{"t": 0, "v": 900}, {"t": 1000, "v": 917, "ease": "linear"}],
                      y=540, w=220, h=300, fill=make.GRADIENT, effects=[make.blur(10), member],
                      end=1000)
        if rotating:
            e["rotation"] = [{"t": 0, "v": 0}, {"t": 1000, "v": 10, "ease": "linear"}]
        p = make.project([{"name": "c", "layer": 0, "elements": [e]}], 1000, fonts=False)
        path = HERE / "out" / "diag-colour.json"
        path.write_text(json.dumps(p))
        on, off = HERE / "out" / "dc-on.raw", HERE / "out" / "dc-off.raw"
        check.render(path, hashes=on, extra={"MONTAGENT_PROTO_COLOUR_BOUND": "1"})
        check.render(path, hashes=off, extra={"MONTAGENT_PROTO_UNBOUND": "1"})
        a, b = on.read_text().split(), off.read_text().split()
        same = sum(x == y for x, y in zip(a, b))
        dump_on, dump_off = HERE / "out" / "dc-don", HERE / "out" / "dc-doff"
        detail = ""
        if same < len(a):
            check.render(path, dump=dump_on, extra={"MONTAGENT_PROTO_COLOUR_BOUND": "1"})
            check.render(path, dump=dump_off, extra={"MONTAGENT_PROTO_UNBOUND": "1"})
            i = next(i for i, (x, y) in enumerate(zip(a, b)) if x != y)
            fa = check.frames(dump_on)[i].read_bytes()
            fb = check.frames(dump_off)[i].read_bytes()
            n = sum(fa[j:j + 3] != fb[j:j + 3] for j in range(0, len(fa), 3))
            mx = max(abs(x - y) for x, y in zip(fa, fb))
            bb = check.changed_bbox(fa, fb)
            nz = check.nonzero_bbox(fb)
            detail = (f"; first differing frame {i}: {n} pixels, max {mx} levels, in x {bb[0]}..{bb[2]} "
                      f"y {bb[1]}..{bb[3]} (element's painted pixels x {nz[0]}..{nz[2]} y {nz[1]}..{nz[3]})")
            for d in (dump_on, dump_off):
                for f in d.glob("*.rgb"):
                    f.unlink()
                d.rmdir()
        lines.append(f"{'rotating' if rotating else 'flat    '} [blur 10, {name:15s}] bound vs no hint: "
                     f"{same}/{len(a)} frames identical{detail}")
        print(lines[-1], flush=True)
for f in ("diag-colour.json", "dc-on.raw", "dc-off.raw"):
    (HERE / "out" / f).unlink(missing_ok=True)
(HERE / "out" / f"{WHERE}colour-bound-diag.txt").write_text("\n".join(lines) + "\n")
