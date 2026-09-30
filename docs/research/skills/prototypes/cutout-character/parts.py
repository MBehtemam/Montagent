"""Cut the master fox into a rig: padded PNG parts whose joint sits at the canvas centre.

Run: uv run --with numpy --with scipy --with pillow python parts.py
Writes parts/*.png and parts/rig.json (each part's canvas size and its pivot in master pixels).
"""
import json
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw
from scipy import ndimage as ndi

HERE = Path(__file__).parent
OUT = HERE / "parts"
OUT.mkdir(exist_ok=True)

def load(name):
    return np.asarray(Image.open(HERE / name).convert("RGB")).astype(np.int16)

master = load("master.png")
H, W, _ = master.shape
lum = master.mean(2)
dark = lum < 90

# Background: near-white connected to the border. Interior whites (eye highlights) stay.
white = master.min(2) > 225
lab, _ = ndi.label(white)
border = set(np.unique(np.r_[lab[0], lab[-1], lab[:, 0], lab[:, -1]])) - {0}
bg = np.isin(lab, list(border))
fg = ~ndi.binary_dilation(bg, iterations=1)  # eat the light antialias fringe

# Fill regions: everything that is neither outline nor background, split by outlines.
# The raglan sleeve's inner seam has a gap at (622..632, 712..732); close it first.
closed = dark.copy()
img = Image.fromarray(closed.astype(np.uint8) * 255)
ImageDraw.Draw(img).line([(627, 700), (626, 745)], fill=255, width=9)
ImageDraw.Draw(img).line([(640, 640), (628, 705)], fill=255, width=9)
closed = np.asarray(img) > 0
fills, n = ndi.label(fg & ~closed)
cents = ndi.center_of_mass(np.ones_like(fills), fills, range(1, n + 1))
sizes = ndi.sum(np.ones_like(fills), fills, range(1, n + 1))

def comps(pred):
    ids = [i + 1 for i, (c, s) in enumerate(zip(cents, sizes)) if s > 30 and pred(c[1], c[0])]
    return np.isin(fills, ids)

# Viewer-right arm: fills right of the seam, between shoulder and hand.
def in_arm(x, y):
    seam_x = 628 + (y - 640) * (680 - 628) / (980 - 640) if y < 980 else 660
    return 640 <= y <= 1120 and x > seam_x + 4
arm_fill = comps(in_arm)
head_fill = comps(lambda x, y: y < 588 and not in_arm(x, y))
body_fill = comps(lambda x, y: True) & ~arm_fill & ~head_fill

# Every foreground pixel (outlines, pupils, finger gaps) goes to the nearest group of
# fills. A seam line shared by two groups is within SEAM of both, so both keep it.
SEAM = 6
dA, dH, dB = (ndi.distance_transform_edt(~m) for m in (arm_fill, head_fill, body_fill))
yy, xx = np.mgrid[0:H, 0:W]
arm = fg & (dA <= np.minimum(dH, dB) + SEAM)
head = fg & (dH <= np.minimum(dA, dB) + SEAM) & (yy < 620)
torso = fg & (dB <= dA + SEAM) & (dB < dH)

SHOULDER = (662, 690)
ELBOW = (678, 815)
disk = lambda c, r: (np.hypot(xx - c[0], yy - c[1]) <= r)

# Keep a shoulder cap on the torso so lifting the arm never opens a hole.
torso |= arm & disk(SHOULDER, 42)

# Split the arm across the elbow, perpendicular to shoulder->wrist.
WRIST = (722, 965)
axis = np.array(WRIST, float) - ELBOW
below = (xx - ELBOW[0]) * axis[0] + (yy - ELBOW[1]) * axis[1] > 0
JOINT_R = 38
upper = arm & (~below | disk(ELBOW, JOINT_R))
fore = arm & (below | disk(ELBOW, JOINT_R))

NECK = (512, 590)

def rgba(src, mask, soft=1.0):
    a = ndi.gaussian_filter(mask.astype(float), soft) * 255 if soft else mask * 255.0
    return np.dstack([src.clip(0, 255), a.clip(0, 255)]).astype(np.uint8)

def paint_joint(rgba_img, c, r, ring_toward=None, fill_from=None, spread=80):
    """A filled disc at the joint, so a bent joint reads as round. `ring_toward` outlines
    the arc facing that direction: at the elbow it faces the forearm (a crease at rest,
    the outside edge when bent); at the shoulder it faces away from the body."""
    im = Image.fromarray(rgba_img)
    d = ImageDraw.Draw(im)
    fc = fill_from or c
    fill = tuple(int(v) for v in np.median(master[fc[1] - 8:fc[1] + 8, fc[0] - 8:fc[0] + 8].reshape(-1, 3), 0))
    box = [c[0] - r, c[1] - r, c[0] + r, c[1] + r]
    d.ellipse(box, fill=fill + (255,))
    if ring_toward is not None:
        a = np.degrees(np.arctan2(ring_toward[1], ring_toward[0]))
        d.arc(box, a - spread, a + spread, fill=(28, 20, 24, 255), width=7)
    return np.asarray(im)

def save_centered(name, img, pivot, mask):
    """Pad/crop so `pivot` is the exact canvas centre; canvas even-sized."""
    ys, xs = np.nonzero(mask)
    rx = int(max(pivot[0] - xs.min(), xs.max() - pivot[0])) + 6
    ry = int(max(pivot[1] - ys.min(), ys.max() - pivot[1])) + 6
    canvas = np.zeros((2 * ry, 2 * rx, 4), np.uint8)
    x0, y0 = pivot[0] - rx, pivot[1] - ry
    sx0, sy0 = max(x0, 0), max(y0, 0)
    sx1, sy1 = min(pivot[0] + rx, W), min(pivot[1] + ry, H)
    canvas[sy0 - y0:sy1 - y0, sx0 - x0:sx1 - x0] = img[sy0:sy1, sx0:sx1]
    Image.fromarray(canvas).save(OUT / f"{name}.png")
    return {"file": f"parts/{name}.png", "w": 2 * rx, "h": 2 * ry, "pivot": list(pivot)}

rig = {}
rig["torso"] = save_centered("torso", rgba(master, torso), (512, 1060), torso)
up = paint_joint(rgba(master, upper), ELBOW, JOINT_R - 4, ring_toward=axis)
SHOULDER_CAP = (666, 702)
up = paint_joint(up, SHOULDER_CAP, 30, ring_toward=(1, -1.3), fill_from=(678, 745), spread=100)
rig["upper_arm"] = save_centered("upper_arm", up, SHOULDER, upper | disk(ELBOW, JOINT_R) | disk(SHOULDER_CAP, 31))
fo = paint_joint(rgba(master, fore), ELBOW, JOINT_R - 4)
rig["forearm"] = save_centered("forearm", fo, ELBOW, fore | disk(ELBOW, JOINT_R))
rig["head"] = save_centered("head", rgba(master, head), NECK, head)

# Face overlays share the head's canvas exactly, so they take the head's transform verbatim.
def overlay(variants, box, name_prefix):
    x0, y0, x1, y1 = box
    region = np.zeros((H, W), bool)
    for v in variants.values():
        diff = np.abs(v - master).sum(2) > 45
        region[y0:y1, x0:x1] |= diff[y0:y1, x0:x1]
    region = ndi.binary_opening(region, iterations=1)
    region = ndi.binary_dilation(region, iterations=10) & head
    alpha = ndi.gaussian_filter(region.astype(float), 2.5)
    for key, v in variants.items():
        img = np.dstack([v.clip(0, 255), (alpha * 255).clip(0, 255)]).astype(np.uint8)
        rig[f"{name_prefix}_{key}"] = save_centered(f"{name_prefix}_{key}", img, NECK, head)

mouths = {k: load(f"v_mouth{k}.png") for k in "AOESM"}
overlay(mouths, (400, 455, 630, 610), "mouth")
overlay({"closed": load("v_blink.png")}, (330, 270, 700, 470), "eyes")

(OUT / "rig.json").write_text(json.dumps(rig, indent=1))

# Debug: the parts reassembled at rest over grey, plus the arm split.
check = Image.new("RGBA", (W, H), (120, 130, 140, 255))
for k in ("torso", "forearm", "upper_arm", "head", "mouth_A"):
    p = rig[k]
    part = Image.open(HERE / p["file"])
    check.alpha_composite(part, (p["pivot"][0] - p["w"] // 2, p["pivot"][1] - p["h"] // 2))
check.convert("RGB").save(HERE / "check_rest.png")
for k, v in rig.items():
    print(k, v["w"], v["h"], v["pivot"])
