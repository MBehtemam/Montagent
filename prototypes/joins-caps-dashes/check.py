"""PROTOTYPE #750: containment, the seam join, and the marching-ants loop.

Renders through $MONTAGENT_BIN with the raw-frame dump on, into $PROTO_SCRATCH (frames are
1920x1080 RGB, 6 MB each, deleted as they are read). Writes out/containment.txt and
out/seam-join.txt.

Containment: every contain/<scene>.json holds that scene's stroked elements alone on black,
with no transform, so the declared box in frame pixels is [x, x+w) x [y, y+h). Any non-zero
byte outside every box is ink outside its box. Per element it also reports the union of its
ink's bounding box over all frames and the least slack to each box edge.
"""

import json
import os
import pathlib
import shutil
import subprocess
import sys

HERE = pathlib.Path(__file__).parent
BIN = os.environ.get("MONTAGENT_BIN", "montagent")
SCRATCH = pathlib.Path(os.environ.get("PROTO_SCRATCH", "/tmp")) / "jc750-dump"
W, H = 1920, 1080
MARGIN = 36


def render(project, dump):
    shutil.rmtree(dump, ignore_errors=True)
    dump.mkdir(parents=True)
    env = dict(os.environ, MONTAGENT_PROTO_DUMP=str(dump), MONTAGENT_PAINTING="1,1000")
    out = SCRATCH / "tmp.mp4"
    r = subprocess.run([BIN, "render", str(project), "--output", str(out)], env=env,
                       capture_output=True, text=True)
    out.unlink(missing_ok=True)
    if r.returncode != 0:
        sys.exit(f"render failed: {project}\n{r.stderr[-3000:]}")
    return sorted(dump.glob("*.rgb"), key=lambda p: int(p.stem))


def extent(data, x0, y0, x1, y1):
    """Bounding box of non-zero pixels in [x0, x1) x [y0, y1), or None."""
    x0, y0, x1, y1 = max(x0, 0), max(y0, 0), min(x1, W), min(y1, H)
    bx0 = by0 = 10**9
    bx1 = by1 = -1
    for y in range(y0, y1):
        row = data[(y * W + x0) * 3:(y * W + x1) * 3]
        left = len(row) - len(row.lstrip(b"\0"))
        if left == len(row):
            continue
        right = len(row.rstrip(b"\0"))
        bx0, bx1 = min(bx0, x0 + left // 3), max(bx1, x0 + (right - 1) // 3)
        by0, by1 = min(by0, y), max(by1, y)
    return None if bx1 < 0 else (bx0, by0, bx1, by1)


def outside(data, boxes):
    """Count of non-zero pixels outside every box."""
    count = 0
    for y in range(H):
        row = bytearray(data[y * W * 3:(y + 1) * W * 3])
        for b in boxes:
            if b["y"] <= y < b["y"] + b["h"]:
                row[b["x"] * 3:(b["x"] + b["w"]) * 3] = bytes(b["w"] * 3)
        if row.strip(b"\0"):
            count += sum(1 for i in range(0, len(row), 3) if row[i:i + 3] != b"\0\0\0")
    return count


def containment():
    boxes = json.loads((HERE / "contain" / "boxes.json").read_text())
    lines = ["Ink outside the declared box, per scene, on every frame rendered at 30 fps.",
             "slack = least distance in whole pixels from the ink's bounding box to the box edge",
             "(left, top, right, bottom); 0 means ink on the box's outermost pixel row.", ""]
    for scene, bs in boxes.items():
        frames = render(HERE / "contain" / f"{scene}.json", SCRATCH / scene)
        union = {b["id"]: None for b in bs}
        least = {b["id"]: [10**9] * 4 for b in bs}
        per_frame = []
        out_total = 0
        for f in frames:
            data = f.read_bytes()
            f.unlink()
            out_total += outside(data, bs)
            for b in bs:
                e = extent(data, b["x"] - MARGIN, b["y"] - MARGIN, b["x"] + b["w"] + MARGIN,
                           b["y"] + b["h"] + MARGIN)
                if e is None:
                    continue
                u = union[b["id"]]
                union[b["id"]] = e if u is None else (min(u[0], e[0]), min(u[1], e[1]),
                                                       max(u[2], e[2]), max(u[3], e[3]))
                slack = (e[0] - b["x"], e[1] - b["y"], b["x"] + b["w"] - 1 - e[2],
                         b["y"] + b["h"] - 1 - e[3])
                least[b["id"]] = [min(a, s) for a, s in zip(least[b["id"]], slack)]
                if scene == "keyed-miter":
                    per_frame.append(slack)
        lines.append(f"{scene}: {len(frames)} frames, non-zero pixels outside every box: {out_total}")
        for b in bs:
            lines.append(f"  {b['id']:18s} box x {b['x']}..{b['x'] + b['w'] - 1}, y {b['y']}..{b['y'] + b['h'] - 1}"
                         f"   ink x {union[b['id']][0]}..{union[b['id']][2]}, y {union[b['id']][1]}..{union[b['id']][3]}"
                         f"   least slack {tuple(least[b['id']])}")
        if per_frame:
            right = [s[2] for s in per_frame]
            lines.append(f"  keyed-miter, right-edge slack per frame (the miter tip's side): min {min(right)} "
                         f"at frame {right.index(min(right))}; frames with slack < 0: "
                         f"{sum(1 for s in per_frame if min(s) < 0)} of {len(per_frame)}")
            lines.append("  " + " ".join(str(s) for s in right))
        lines.append("")
        shutil.rmtree(SCRATCH / scene, ignore_errors=True)
    (HERE / "out" / "containment.txt").write_text("\n".join(lines) + "\n")
    print("\n".join(lines))


def seam_join():
    """Does Skia join the last dash to the first across a closed outline's start point?

    At a right-angled start corner the outer quadrant (outside both edges, inside the miter)
    is inked only by a join: two butt-ended dashes leave it empty. Read on the containment
    renders of the seam and dashed-shape scenes, plus an undashed control.
    """
    boxes = json.loads((HERE / "contain" / "boxes.json").read_text())
    lines = []
    for scene, ids in (("seam", ("seam-310", "seam-333")), ("dashed-shapes", ("dash-rect",))):
        frames = render(HERE / "contain" / f"{scene}.json", SCRATCH / scene)
        data = frames[0].read_bytes()
        for b in boxes[scene]:
            if b["id"] not in ids:
                continue
            m = 12 if b["id"].startswith("seam") else 6  # the corner sits at (x+m, y+m)
            half = 6
            cx, cy = b["x"] + m, b["y"] + m
            quad = extent(data, cx - half + 1, cy - half + 1, cx - 1, cy - 1)
            lines.append(f"{b['id']}: start corner at ({cx}, {cy}); outer quadrant "
                         f"[{cx - half + 1}, {cx - 2}] x [{cy - half + 1}, {cy - 2}] inked: {quad is not None}")
        shutil.rmtree(SCRATCH / scene, ignore_errors=True)
    (HERE / "out" / "seam-join.txt").write_text("\n".join(lines) + "\n")
    print("\n".join(lines))


if __name__ == "__main__":
    SCRATCH.mkdir(parents=True, exist_ok=True)
    which = sys.argv[1:] or ["containment", "seam"]
    if "containment" in which:
        containment()
    if "seam" in which:
        seam_join()
