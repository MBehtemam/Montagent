"""PROTOTYPE #760: containment, empty frames, the seam at the start point, and dash anchoring.

Renders through $MONTAGENT_BIN with the raw-frame dump on, into $PROTO_SCRATCH (frames are
1920x1080 RGB, 6 MB each, deleted as they are read). Writes:

- out/containment.txt: every contain/<scene>.json holds that scene's stroked elements alone on
  black, untransformed, so the declared box in frame pixels is [x, x+w) x [y, y+h). Any
  non-zero byte outside every box is ink outside its box. Every frame at 30 fps. For the
  square-cap scene it also lists, per frame, the ink outside and each element's least slack.
- out/empty-frames.txt: which frames draw no ink at all, per element, in the draw-on,
  crossing and overshoot scenes (the first draw-on frame must be empty: no dot).
- out/seam.txt: probe/seam-trimmed.json (windows [0.75, 0.25], straddling the start point)
  against probe/seam-plain.json (the same elements untrimmed): the pixels in a square around
  each start point must be the same bytes, which is one stroke through the start point with
  the outline's own join, and no cap or seam.
- out/dash-anchor.txt: probe/anchor-trimmed.json (dashed lines trimmed at 0.5) against
  probe/anchor-plain.json: inside the window every pixel must equal the untrimmed one.
"""

import json
import math
import os
import pathlib
import shutil
import subprocess
import sys

HERE = pathlib.Path(__file__).parent
BIN = os.environ.get("MONTAGENT_BIN", "montagent")
SCRATCH = pathlib.Path(os.environ.get("PROTO_SCRATCH", "/tmp")) / "trim760-dump"
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
    empty = ["Frames with no ink at all, per element (frame n is at n/30 s from the element's start).",
             ""]
    for scene, bs in boxes.items():
        frames = render(HERE / "contain" / f"{scene}.json", SCRATCH / scene)
        union = {b["id"]: None for b in bs}
        least = {b["id"]: [10**9] * 4 for b in bs}
        blank = {b["id"]: [] for b in bs}
        per_frame = []
        out_total = 0
        for n, f in enumerate(frames):
            data = f.read_bytes()
            f.unlink()
            out_here = outside(data, bs)
            out_total += out_here
            row = [n, out_here]
            for b in bs:
                e = extent(data, b["x"] - MARGIN, b["y"] - MARGIN, b["x"] + b["w"] + MARGIN,
                           b["y"] + b["h"] + MARGIN)
                if e is None:
                    blank[b["id"]].append(n)
                    row.append(None)
                    continue
                u = union[b["id"]]
                union[b["id"]] = e if u is None else (min(u[0], e[0]), min(u[1], e[1]),
                                                       max(u[2], e[2]), max(u[3], e[3]))
                slack = (e[0] - b["x"], e[1] - b["y"], b["x"] + b["w"] - 1 - e[2],
                         b["y"] + b["h"] - 1 - e[3])
                least[b["id"]] = [min(a, s) for a, s in zip(least[b["id"]], slack)]
                row.append(min(slack))
            per_frame.append(row)
        lines.append(f"{scene}: {len(frames)} frames, non-zero pixels outside every box: {out_total}")
        for b in bs:
            u = union[b["id"]]
            ink = "no ink" if u is None else f"ink x {u[0]}..{u[2]}, y {u[1]}..{u[3]}"
            lines.append(f"  {b['id']:16s} box x {b['x']}..{b['x'] + b['w'] - 1}, y {b['y']}..{b['y'] + b['h'] - 1}"
                         f"   {ink}   least slack {tuple(least[b['id']])}")
        if scene == "square-cap":
            lines.append("  per frame: frame, ink pixels outside every box, least slack per element ("
                         + ", ".join(b["id"] for b in bs) + "; '-' = no ink)")
            for row in per_frame:
                lines.append("    " + " ".join("-" if v is None else str(v) for v in row))
        lines.append("")
        if scene in ("draw-on", "crossing", "overshoot", "dashed", "square-cap"):
            empty.append(f"{scene}: {len(frames)} frames")
            for b in bs:
                empty.append(f"  {b['id']:16s} empty frames: {ranges(blank[b['id']])}")
            empty.append("")
        shutil.rmtree(SCRATCH / scene, ignore_errors=True)
    (HERE / "out" / "containment.txt").write_text("\n".join(lines) + "\n")
    (HERE / "out" / "empty-frames.txt").write_text("\n".join(empty) + "\n")
    print("\n".join(lines[:60]))
    print("\n".join(empty))


def ranges(ns):
    if not ns:
        return "none"
    out, a, b = [], ns[0], ns[0]
    for n in ns[1:]:
        if n == b + 1:
            b = n
        else:
            out.append(f"{a}" if a == b else f"{a}-{b}")
            a = b = n
    out.append(f"{a}" if a == b else f"{a}-{b}")
    return ", ".join(out) + f"  ({len(ns)} frames)"


def diff_region(a, b, x0, y0, x1, y1):
    """(pixels that differ, greatest channel difference, inked pixels) in [x0,x1) x [y0,y1)."""
    n = worst = inked = 0
    for y in range(y0, y1):
        ra = a[(y * W + x0) * 3:(y * W + x1) * 3]
        rb = b[(y * W + x0) * 3:(y * W + x1) * 3]
        for i in range(0, len(ra), 3):
            if ra[i:i + 3] != b"\0\0\0":
                inked += 1
            if ra[i:i + 3] != rb[i:i + 3]:
                n += 1
                worst = max(worst, max(abs(ra[i + k] - rb[i + k]) for k in range(3)))
    return n, worst, inked


def seam():
    trimmed = render(HERE / "probe" / "seam-trimmed.json", SCRATCH / "seam-t")[0].read_bytes()
    plain = render(HERE / "probe" / "seam-plain.json", SCRATCH / "seam-p")[0].read_bytes()
    os.environ["MONTAGENT_PROTO_SPLIT_SEAM"] = "1"
    split = render(HERE / "probe" / "seam-trimmed.json", SCRATCH / "seam-s")[0].read_bytes()
    del os.environ["MONTAGENT_PROTO_SPLIT_SEAM"]
    starts = json.loads((HERE / "probe" / "seam-starts.json").read_text())
    project = json.loads((HERE / "probe" / "seam-trimmed.json").read_text())
    half = 40
    lines = ["Windows [0.75, 0.25] (trim_start 0, trim_end 0.5, trim_offset 0.75), straddling the",
             "start point, against the same element untrimmed, in an 80x80 square centred on the",
             "start point. A cap or a seam there shows as a large difference (a missing join",
             "wedge, or a cap's bulge); antialiasing alone stays within a few levels.",
             "",
             "control = the same window with the second piece started by a move",
             "(MONTAGENT_PROTO_SPLIT_SEAM), i.e. two ends meeting at the start point.",
             ""]
    for id, (sx, sy) in starts.items():
        x0, y0 = int(round(sx)) - half, int(round(sy)) - half
        box = (x0, y0, x0 + 2 * half, y0 + 2 * half)
        n, worst, inked = diff_region(trimmed, plain, *box)
        cn, cworst, _ = diff_region(split, plain, *box)
        lines.append(f"{id:16s} start ({sx:.0f}, {sy:.0f}): inked {inked:5d} px; vs untrimmed: "
                     f"{n:4d} px differ (max {worst:3d} levels) {'SAME' if n == 0 else 'DIFFERS'};"
                     f"  control: {cn:4d} px differ (max {cworst:3d})")
    # The ellipse away from the start point: a circle, so fraction f is the angle 2πf
    # clockwise from 3 o'clock. At f = 0.125 the square is 0.125 turn from the start point
    # and from the window's end.
    for t in project["tracks"]:
        e = t["elements"][0]
        if e["type"] != "ellipse":
            continue
        r = e["width"] / 2 - e["stroke_width"] / 2
        cx, cy = e["x"] + e["width"] / 2, e["y"] + e["height"] / 2
        px, py = cx + r * math.cos(math.pi / 4), cy + r * math.sin(math.pi / 4)
        x0, y0 = int(round(px)) - half, int(round(py)) - half
        n, worst, inked = diff_region(trimmed, plain, x0, y0, x0 + 2 * half, y0 + 2 * half)
        lines.append(f"{e['id']:16s} away from the start, at 0.125 ({px:.0f}, {py:.0f}): inked "
                     f"{inked} px; vs untrimmed {n} px differ (max {worst}): `draw_oval` against "
                     f"the cut conic outline, not a seam")
    for d in ("seam-t", "seam-p", "seam-s"):
        shutil.rmtree(SCRATCH / d, ignore_errors=True)
    (HERE / "out" / "seam.txt").write_text("\n".join(lines) + "\n")
    print("\n".join(lines))


def anchor():
    project = json.loads((HERE / "probe" / "anchor-trimmed.json").read_text())
    trimmed = render(HERE / "probe" / "anchor-trimmed.json", SCRATCH / "anc-t")[0].read_bytes()
    plain = render(HERE / "probe" / "anchor-plain.json", SCRATCH / "anc-p")[0].read_bytes()
    lines = ["Dashed lines trimmed at 0.5 against the same lines untrimmed: inside the window",
             "(the drawn half, less 30 px at the trim end) every pixel must be the same bytes.", ""]
    for t in project["tracks"]:
        e = t["elements"][0]
        x, y, w, h = e["x"], e["y"], e["width"], e["height"]
        mid = x + w // 2
        if "trim_end" in e:   # drawn half is the left one
            region = (x, y, mid - 30, y + h)
            dark = (mid + 30, y, x + w, y + h)
        else:                 # trim_start 0.5: the right half (by length; the curve is near-symmetric)
            region = (mid + 30, y, x + w, y + h)
            dark = (x, y, mid - 30, y + h)
        n, worst, inked = diff_region(trimmed, plain, *region)
        _, _, inked_dark = diff_region(trimmed, plain, *dark)
        lines.append(f"{e['id']:14s} window region x {region[0]}..{region[2]}: inked {inked} px, "
                     f"differing {n} px (max {worst}) -> {'SAME' if n == 0 else 'DIFFERS'}; "
                     f"ink in the trimmed-away half: {inked_dark} px")
    shutil.rmtree(SCRATCH / "anc-t", ignore_errors=True)
    shutil.rmtree(SCRATCH / "anc-p", ignore_errors=True)
    (HERE / "out" / "dash-anchor.txt").write_text("\n".join(lines) + "\n")
    print("\n".join(lines))


if __name__ == "__main__":
    SCRATCH.mkdir(parents=True, exist_ok=True)
    which = sys.argv[1:] or ["containment", "seam", "anchor"]
    if "seam" in which:
        seam()
    if "anchor" in which:
        anchor()
    if "containment" in which:
        containment()
