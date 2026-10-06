"""PROTOTYPE #722: the identity, shift and reach checks.

Each renders through the prototype binary ($MONTAGENT_BIN) with the raw-frame probes on and
writes its result to out/<check>.txt.
"""

import hashlib
import math
import os
import pathlib
import shutil
import subprocess
import sys

HERE = pathlib.Path(__file__).parent
BIN = os.environ.get("MONTAGENT_BIN", "montagent")
W, H = 1920, 1080


def render(project, *, dump=None, hashes=None, painting="1,1000", extra=None):
    env = dict(os.environ)
    env["MONTAGENT_PAINTING"] = painting
    if dump:
        shutil.rmtree(dump, ignore_errors=True)
        dump.mkdir(parents=True)
        env["MONTAGENT_PROTO_DUMP"] = str(dump)
    if hashes:
        hashes.unlink(missing_ok=True)
        env["MONTAGENT_PROTO_HASHES"] = str(hashes)
    env.update(extra or {})
    out = HERE / "out" / "tmp.mp4"
    r = subprocess.run([BIN, "render", str(project), "--output", str(out)], env=env,
                       capture_output=True, text=True)
    out.unlink(missing_ok=True)
    if r.returncode != 0:
        sys.exit(f"render failed: {project}\n{r.stderr[-2000:]}")
    return r.stderr


def frames(dump):
    return sorted(dump.glob("*.rgb"), key=lambda p: int(p.stem))


def identity():
    lines = []
    for on in sorted((HERE / "identity").glob("*-on.json")):
        case = on.name[: -len("-on.json")]
        off = on.with_name(f"{case}-off.json")
        results = []
        for painting in ("1,1000", "4,3"):
            ha = HERE / "out" / "identity" / f"{case}-on-{painting.replace(',', 'x')}.raw"
            hb = HERE / "out" / "identity" / f"{case}-off-{painting.replace(',', 'x')}.raw"
            ha.parent.mkdir(parents=True, exist_ok=True)
            render(on, hashes=ha, painting=painting)
            render(off, hashes=hb, painting=painting)
            a, b = ha.read_text().split(), hb.read_text().split()
            same = sum(x == y for x, y in zip(a, b))
            results.append(f"{painting}: {same}/{len(a)} frames identical")
        lines.append(f"{case:24s} " + "; ".join(results))
    return lines


def shift():
    base = HERE / "out" / "shift" / "0"
    render(HERE / "shift" / "shift-0.json", dump=base)
    a = frames(base)
    lines = []
    # The grain re-rolls: consecutive frames of the unshifted element differ.
    rerolled = sum(a[i].read_bytes() != a[i + 1].read_bytes() for i in range(59))
    lines.append(f"unshifted: {rerolled}/59 consecutive frame pairs differ (grain re-rolls every frame)")
    for ms, k in ((100, 3), (1000, 30)):
        d = HERE / "out" / "shift" / str(ms)
        render(HERE / "shift" / f"shift-{ms}.json", dump=d)
        b = frames(d)
        same = sum(a[n].read_bytes() == b[n + k].read_bytes() for n in range(60))
        lines.append(f"start +{ms} ms ({k} frames): frame n vs frame n+{k}: {same}/60 identical")
    shutil.rmtree(HERE / "out" / "shift")
    return lines


def bbox(mask_fn, data):
    xs0, ys0, xs1, ys1 = W, H, -1, -1
    for y in range(H):
        row = data[y * W * 3:(y + 1) * W * 3]
        for x in range(W):
            if mask_fn(x, y):
                xs0, ys0, xs1, ys1 = min(xs0, x), min(ys0, y), max(xs1, x), max(ys1, y)
    return xs0, ys0, xs1, ys1


def changed_bbox(a, b):
    """The bounding box of the pixels that differ between two raw RGB frames."""
    x0, y0, x1, y1 = W, H, -1, -1
    for y in range(H):
        ra, rb = a[y * W * 3:(y + 1) * W * 3], b[y * W * 3:(y + 1) * W * 3]
        if ra == rb:
            continue
        y0, y1 = min(y0, y), max(y1, y)
        xs = [x for x in range(W) if ra[x * 3:x * 3 + 3] != rb[x * 3:x * 3 + 3]]
        x0, x1 = min(x0, xs[0]), max(x1, xs[-1])
    return x0, y0, x1, y1


def nonzero_bbox(a):
    return changed_bbox(a, bytes(len(a)))


def reach():
    import make  # noqa: E402

    off_dir = HERE / "out" / "reach" / "off"
    render(HERE / "reach" / "off.json", dump=off_dir)
    off = frames(off_dir)[0].read_bytes()
    drawn = nonzero_bbox(off)
    lines = [f"element drawn (no member): x {drawn[0]}..{drawn[2]}, y {drawn[1]}..{drawn[3]}"]
    for case, member in make.REACH.items():
        d = HERE / "out" / "reach" / case
        render(HERE / "reach" / f"{case}-on.json", dump=d)
        on = frames(d)[0].read_bytes()
        t = changed_bbox(off, on)
        if member["name"] == "glow":
            rx = ry = 3 * member["radius"] / 2
            formula = "3σ = 1.5 × radius"
        else:
            th = math.radians(member["angle"])
            rx = abs(math.cos(th)) * member["length"] / 2
            ry = abs(math.sin(th)) * member["length"] / 2
            formula = "(|cos θ|·L/2, |sin θ|·L/2)"
        past = (drawn[0] - t[0], drawn[1] - t[1], t[2] - drawn[2], t[3] - drawn[3])
        inside = max(past[0], past[2]) <= math.ceil(rx) and max(past[1], past[3]) <= math.ceil(ry)
        lines.append(
            f"{case:14s} declared reach {formula} = ({rx:.2f}, {ry:.2f}) px; touched pixels reach "
            f"past the drawn box by left {past[0]}, top {past[1]}, right {past[2]}, bottom {past[3]} "
            f"-> {'inside' if inside else 'OUTSIDE'} the declared reach"
        )
        shutil.rmtree(d)
    shutil.rmtree(off_dir)
    return lines


if __name__ == "__main__":
    sys.path.insert(0, str(HERE))
    which = sys.argv[1:] or ["identity", "shift", "reach"]
    for name in which:
        lines = globals()[name]()
        text = "\n".join(lines) + "\n"
        (HERE / "out" / f"{name}.txt").write_text(text)
        print(f"== {name}\n{text}")
