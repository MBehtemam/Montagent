#!/usr/bin/env python3
"""PROTOTYPE #764: does the ink stay inside the declared box?

Each contain/<id>.json holds one text element alone, on black, untransformed, its box at
(300, 300) in a frame 600 px larger than the box. Every frame of the element's range is
rendered (raw RGB dumped at the encoder input) and every non-zero pixel outside the box is
counted. "Least slack" is the closest the ink came to each box edge over the range (negative:
past it). ADR-0161 §7: the inset m is a frame convention, not a containment guarantee, so ink
outside the box is reported, not withdrawn.

Usage: check.py <scratch-dir>   (MONTAGENT_BIN names the binary)
"""
import glob
import json
import os
import shutil
import subprocess
import sys

from PIL import Image, ImageChops

B = os.environ.get("MONTAGENT_BIN", "montagent")
scratch = sys.argv[1]
here = os.path.dirname(os.path.abspath(__file__))
rows = []
for path in sorted(glob.glob(os.path.join(here, "contain", "*.json"))):
    doc = json.load(open(path))
    fw, fh = doc["frame"]["width"], doc["frame"]["height"]
    e = doc["tracks"][0]["elements"][0]
    box = (300, 300, 300 + e["width"], 300 + e["height"])
    sizes = [e["size"]] + [r["size"] for r in e["runs"] if "size" in r]
    strokes = [e.get("stroke_width", 0)] + [r.get("stroke_width", 0) for r in e["runs"]]
    m = max(sizes) + max(strokes)
    dump = os.path.join(scratch, "dump-" + e["id"])
    shutil.rmtree(dump, ignore_errors=True)
    os.makedirs(dump)
    subprocess.run([B, "render", path, "--from", str(e["start"]), "--to", str(e["end"]),
                    "--output", os.path.join(scratch, "contain-" + e["id"] + ".mp4")],
                   env=dict(os.environ, MONTAGENT_PROTO_DUMP=dump), capture_output=True,
                   check=True)
    files = sorted(glob.glob(os.path.join(dump, "*.rgb")),
                   key=lambda p: int(os.path.basename(p)[:-4]))
    outside_total = 0
    worst_frame = 0
    slack = [10**9] * 4  # left, top, right, bottom
    for f in files:
        img = Image.frombytes("RGB", (fw, fh), open(f, "rb").read())
        r, g, b = img.split()
        ink = ImageChops.lighter(ImageChops.lighter(r, g), b).point(lambda v: 255 if v else 0)
        bbox = ink.getbbox()
        if bbox is None:
            continue
        inside = ink.crop(box).histogram()[255]
        total = ink.histogram()[255]
        outside = total - inside
        outside_total += outside
        worst_frame = max(worst_frame, outside)
        l, t, rr, bb = bbox
        slack = [min(slack[0], l - box[0]), min(slack[1], t - box[1]),
                 min(slack[2], box[2] - rr), min(slack[3], box[3] - bb)]
    shutil.rmtree(dump)
    os.remove(os.path.join(scratch, "contain-" + e["id"] + ".mp4"))
    rows.append((e["id"], len(files), m, outside_total, worst_frame, slack))

print(f"{'element':22s} {'frames':>6s} {'m':>4s} {'ink px outside box':>19s} "
      f"{'worst frame':>11s}  least slack L/T/R/B (px; negative = past the edge)")
for id, n, m, total, worst, slack in rows:
    print(f"{id:22s} {n:6d} {m:4d} {total:19d} {worst:11d}  {slack[0]}/{slack[1]}/{slack[2]}/{slack[3]}")
