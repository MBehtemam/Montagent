"""Cut the owl into a rig: padded PNG parts whose pivot is the canvas centre.

Run from `assets/`:
`uv run --python 3.12 --with numpy --with scipy --with pillow python ../pack-src/character/cut_rig.py`.

Reads `master.png` (the one full-body drawing) and `blink.png` (a FLUX edit of it with the
eyes shut, registered at shift 0,0) beside this script, both drawn by `flux.py`. Writes
`character/parts/*.png`, `character/rig.json` and `character/rest.png`, and writes the same
bytes on every run.

How the cut works. The white that touches the border is background. Everything else is
split by its dark outlines into fills; each fill goes to a group (head, torso, left arm,
right arm) by where its centre lies, and each outline pixel goes to the nearest group. The
sleeves join the cardigan with no outline between them, so a seam is drawn across each
shoulder before the split. Each arm is cut across its middle into an upper arm and a
forearm, and round caps are painted at the elbow and shoulder so a bent joint reads as
round. A flat grey neck is painted on the torso under the head so tilting the head never
opens a hole. Every coordinate below was read off a gridded zoom of `master.png`.

The five mouths are drawn here, not by FLUX: FLUX redrew the beak at a different size for
every mouth it was asked for, and a mouth set that changes size flickers. Each mouth is
drawn behind the master's own upper beak, so the beak never moves.
"""

import json
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw
from scipy import ndimage as ndi

SRC = Path(__file__).parent
OUT = Path("character")
(OUT / "parts").mkdir(parents=True, exist_ok=True)


def load(name):
    return np.asarray(Image.open(SRC / name).convert("RGB")).astype(np.int16)


master = load("master.png")
H, W, _ = master.shape
yy, xx = np.mgrid[0:H, 0:W]
dark = master.mean(2) < 90
INK = (48, 36, 38)
GREY = (156, 137, 127)
OUTLINE = 11  # the drawing's stroke width, in pixels

# Background: near-white connected to the border. Interior whites (eye rings) stay.
white = master.min(2) > 225
lab, _ = ndi.label(white)
border = set(np.unique(np.r_[lab[0], lab[-1], lab[:, 0], lab[:, -1]])) - {0}
fg = ~ndi.binary_dilation(np.isin(lab, list(border)), iterations=1)  # eat the light fringe

# Seams across each shoulder, from where the head meets the sleeve to the armpit.
SEAM_L = ((355, 660), (333, 786))
SEAM_R = ((665, 660), (687, 786))
img = Image.fromarray(dark.astype(np.uint8) * 255)
for seam in (SEAM_L, SEAM_R):
    ImageDraw.Draw(img).line(seam, fill=255, width=9)
fills, n = ndi.label(fg & ~(np.asarray(img) > 0))
cents = ndi.center_of_mass(np.ones_like(fills), fills, range(1, n + 1))
sizes = ndi.sum(np.ones_like(fills), fills, range(1, n + 1))


def seam_x(seam, y):
    (x0, y0), (x1, y1) = seam
    y = min(max(y, y0), y1)
    return x0 + (y - y0) * (x1 - x0) / (y1 - y0)


def comps(pred):
    ids = [i + 1 for i, (c, s) in enumerate(zip(cents, sizes)) if s > 30 and pred(c[1], c[0])]
    return np.isin(fills, ids)


in_left = lambda x, y: 640 <= y <= 1000 and x < seam_x(SEAM_L, y) - 4
in_right = lambda x, y: 640 <= y <= 1000 and x > seam_x(SEAM_R, y) + 4
left_fill = comps(in_left)
right_fill = comps(in_right)
head_fill = comps(lambda x, y: y < 640 and not in_left(x, y) and not in_right(x, y))
torso_fill = comps(lambda x, y: True) & ~left_fill & ~right_fill & ~head_fill

# Every foreground pixel (outlines, pupils, finger gaps) goes to the nearest group of
# fills. A seam line shared by two groups is within SEAM of both, so both keep it.
SEAM = 6
dL, dR, dH, dB = (ndi.distance_transform_edt(~m) for m in (left_fill, right_fill, head_fill, torso_fill))
left = fg & (dL <= np.minimum.reduce([dR, dH, dB]) + SEAM)
right = fg & (dR <= np.minimum.reduce([dL, dH, dB]) + SEAM)
head = fg & (dH <= np.minimum.reduce([dL, dR, dB]) + SEAM) & (yy < 700)
torso = fg & (dB <= np.minimum(dL, dR) + SEAM) & (dB < dH)

disk = lambda c, r: np.hypot(xx - c[0], yy - c[1]) <= r

# Pivots, in master pixels. Left and right are the viewer's.
NECK = (512, 648)
FEET = (512, 1322)
SHOULDER = {"left": (356, 736), "right": (666, 736)}
WRIST = {"left": (172, 815), "right": (840, 820)}
ARM_R = 48  # half the sleeve's width, to the middle of its outline
arms = {"left": left, "right": right}

# The shoulders overlap, as in any cut-out puppet. The torso keeps a strip of each sleeve
# next to the seam, so moving an arm never opens a hole; the upper arm keeps only the part
# of that strip inside a disc round its pivot, which looks the same at every angle, so its
# cut end never swings out over the cardigan.
STRIP = 30
SHOULDER_R = 40


def beyond_seam(seam, side):
    """Distance past the seam line, towards the arm; negative on the body's side."""
    (x0, y0), (x1, y1) = seam
    nx, ny = y1 - y0, -(x1 - x0)
    nx, ny = nx / np.hypot(nx, ny), ny / np.hypot(nx, ny)
    d = (xx - x0) * nx + (yy - y0) * ny
    return -d if side == "left" else d


seam_strip = {}
for side, seam in (("left", SEAM_L), ("right", SEAM_R)):
    past = beyond_seam(seam, side)
    seam_strip[side] = arms[side] & (past < STRIP)
    torso |= seam_strip[side]
    arms[side] = arms[side] & ~(seam_strip[side] & ~disk(SHOULDER[side], SHOULDER_R))


# Pixels well inside the drawing, away from the background. A part is feathered only at
# the background's edge: feathered at a cut, two neighbouring parts would each be
# half-transparent there, and the background would show through the join.
inner = ndi.binary_erosion(fg, iterations=2)


def rgba(mask, src=master, soft=1.0):
    a = ndi.gaussian_filter(mask.astype(float), soft) if soft else mask.astype(float)
    a = np.where(mask & inner, 1.0, a) * 255
    return np.dstack([src.clip(0, 255), a.clip(0, 255)]).astype(np.uint8)


def paint_joint(img, c, r, fill_at, ring_toward=None, spread=90):
    """A filled disc at the joint, so a bent joint reads as round. `ring_toward` outlines
    the arc facing that way: at the elbow it faces the forearm (a crease at rest, the
    outside edge when bent); at the shoulder it faces away from the body."""
    fx, fy = fill_at
    fill = tuple(int(v) for v in np.median(master[fy - 6:fy + 6, fx - 6:fx + 6].reshape(-1, 3), 0))
    box = [c[0] - r - OUTLINE // 2, c[1] - r - OUTLINE // 2, c[0] + r + OUTLINE // 2, c[1] + r + OUTLINE // 2]
    # The disc goes under the part's own pixels, so it only fills what the cut took away.
    im = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    ImageDraw.Draw(im).ellipse(box, fill=fill + (255,))
    im.alpha_composite(Image.fromarray(img))
    d = ImageDraw.Draw(im)
    if ring_toward is not None:
        a = np.degrees(np.arctan2(ring_toward[1], ring_toward[0]))
        d.arc(box, a - spread, a + spread, fill=INK + (255,), width=OUTLINE)
    return np.asarray(im)


def save_centered(name, img, pivot, mask):
    """Pad or crop so `pivot` is the exact canvas centre; the canvas is even-sized."""
    ys, xs = np.nonzero(mask)
    rx = int(max(pivot[0] - xs.min(), xs.max() - pivot[0])) + 6
    ry = int(max(pivot[1] - ys.min(), ys.max() - pivot[1])) + 6
    canvas = np.zeros((2 * ry, 2 * rx, 4), np.uint8)
    x0, y0 = pivot[0] - rx, pivot[1] - ry
    sx0, sy0 = max(x0, 0), max(y0, 0)
    sx1, sy1 = min(pivot[0] + rx, W), min(pivot[1] + ry, H)
    canvas[sy0 - y0:sy1 - y0, sx0 - x0:sx1 - x0] = img[sy0:sy1, sx0:sx1]
    canvas[canvas[..., 3] == 0] = 0  # nothing under full transparency, so the PNG compresses
    Image.fromarray(canvas).save(OUT / "parts" / f"{name}.png", optimize=True)
    return {"file": f"parts/{name}.png", "width": 2 * rx, "height": 2 * ry, "pivot": list(pivot)}


parts = {}

# Torso, with legs and feet, pivoting at the feet so a lean rocks the whole body. A flat
# grey neck sits under the head's lower half.
# A rounded neck from inside the head down to the torso's top edge. The drawing leaves a
# sliver of white between the two, enclosed and so not background; the neck replaces it.
neck = ((xx - NECK[0]) / 120) ** 2 + ((yy - NECK[1] - 6) / 30) ** 2 <= 1
torso &= ~(neck & (master.min(2) > 235))
torso_img = rgba(torso)
im =Image.fromarray(np.dstack([np.broadcast_to(np.array(GREY, np.uint8), (H, W, 3)), neck * np.uint8(255)]))
im.alpha_composite(Image.fromarray(torso_img))  # the neck goes under the torso's feathered top edge
torso_img = np.asarray(im).copy()
# The sleeve strips' cut ends get an outline, as the shoulder's edge when an arm moves away.
for side, seam in (("left", SEAM_L), ("right", SEAM_R)):
    (sx0, sy0), (sx1, sy1) = seam
    # Just past the strip, so at rest the upper arm covers it.
    t = (STRIP + OUTLINE // 2 + 1) / np.hypot(sx1 - sx0, sy1 - sy0)
    ox, oy = (sy1 - sy0) * t, -(sx1 - sx0) * t
    if side == "left":
        ox, oy = -ox, -oy
    ImageDraw.Draw(im).line([(sx0 + ox, sy0 + oy), (sx1 + ox, sy1 + oy)], fill=INK + (255,), width=OUTLINE)
edge = (np.asarray(im)[..., :3] != torso_img[..., :3]).any(2) & fg
torso_img = np.asarray(im).copy()
torso_img[~(torso | neck | edge), 3] = 0
parts["torso"] = save_centered("torso", torso_img, FEET, torso | neck | edge)

for side, arm in arms.items():
    sh, wr = np.array(SHOULDER[side], float), np.array(WRIST[side], float)
    axis = wr - sh
    elbow = tuple(int(v) for v in np.round(sh + axis * 0.5))
    below = (xx - elbow[0]) * axis[0] + (yy - elbow[1]) * axis[1] > 0
    upper = arm & (~below | disk(elbow, ARM_R))
    fore = arm & (below | disk(elbow, ARM_R))
    fill_at = tuple(int(v) for v in np.round(sh + axis * 0.4))
    up = paint_joint(rgba(upper), elbow, ARM_R - OUTLINE // 2, fill_at, ring_toward=axis)
    up = paint_joint(up, SHOULDER[side], SHOULDER_R - OUTLINE // 2, fill_at)
    parts[f"upper_arm_{side}"] = save_centered(
        f"upper_arm_{side}", up, SHOULDER[side], upper | disk(elbow, ARM_R + 1) | disk(SHOULDER[side], SHOULDER_R + 1))
    fo = paint_joint(rgba(fore), elbow, ARM_R - OUTLINE // 2, fill_at)
    parts[f"forearm_{side}"] = save_centered(f"forearm_{side}", fo, elbow, fore | disk(elbow, ARM_R + 1))

parts["head"] = save_centered("head", rgba(head), NECK, head)

# ---- face overlays: on the head's canvas, so they take the head's transform verbatim ----

BEAK_BOX = (452, 484, 566, 600)
x0, y0, x1, y1 = BEAK_BOX
cream = np.abs(master - master[610, 508]).sum(2) < 60
beak_lab, _ = ndi.label(~cream[y0:y1, x0:x1])
beak = np.zeros((H, W), bool)
beak[y0:y1, x0:x1] = beak_lab == beak_lab[540 - y0, 508 - x0]
BEAK = tuple(int(v) for v in master[525, 508])
MOUTH = (92, 34, 44)
TONGUE = (238, 128, 140)
TEETH = (255, 252, 244)


def ell(d, cx, cy, rx, ry, fill, outline=True, s=4):
    box = [(cx - rx) * s, (cy - ry) * s, (cx + rx) * s, (cy + ry) * s]
    d.ellipse(box, fill=fill, outline=INK if outline else None, width=OUTLINE * s if outline else 0)


def rrect(d, cx, cy, rx, ry, fill, s=4):
    d.rounded_rectangle([(cx - rx) * s, (cy - ry) * s, (cx + rx) * s, (cy + ry) * s], radius=ry * s,
                        fill=fill, outline=INK, width=max(1, OUTLINE * s // 2))


# Each mouth: shapes drawn in order, behind the upper beak. Centred on the beak's axis.
CX = 508
MOUTHS = {
    "open": lambda d: (ell(d, CX, 590, 36, 44, BEAK), ell(d, CX, 584, 23, 30, MOUTH, outline=False),
                       ell(d, CX, 604, 14, 9, TONGUE, outline=False)),
    "round": lambda d: (ell(d, CX, 596, 26, 26, BEAK), ell(d, CX, 598, 12, 12, MOUTH, outline=False)),
    "small": lambda d: (ell(d, CX, 588, 28, 26, BEAK), ell(d, CX, 582, 16, 15, MOUTH, outline=False)),
    "teeth": lambda d: (ell(d, CX, 586, 42, 22, BEAK), rrect(d, CX, 586, 30, 8, TEETH)),
    "closed": lambda d: (ell(d, CX, 582, 30, 17, BEAK),),
}
M_BOX = (440, 520, 580, 650)


def draw_mouth(shapes):
    """The master with this mouth drawn behind its upper beak, and the drawn region."""
    mx0, my0, mx1, my1 = M_BOX
    s = 4
    layer = Image.new("RGBA", ((mx1 - mx0) * s, (my1 - my0) * s), (0, 0, 0, 0))
    shapes(_Shifted(ImageDraw.Draw(layer), mx0 * s, my0 * s))
    layer = layer.resize((mx1 - mx0, my1 - my0), Image.LANCZOS)
    face = Image.fromarray(master.astype(np.uint8)).convert("RGBA")
    face.alpha_composite(layer, (mx0, my0))
    out = np.asarray(face.convert("RGB")).astype(np.int16).copy()
    out[beak] = master[beak]
    region = np.zeros((H, W), bool)
    region[my0:my1, mx0:mx1] = np.asarray(layer)[..., 3] > 8
    return out, region & ~beak


class _Shifted:
    """ImageDraw that takes coordinates in supersampled master pixels."""

    def __init__(self, d, dx, dy):
        self.d, self.dx, self.dy = d, dx, dy

    def _box(self, b):
        return [b[0] - self.dx, b[1] - self.dy, b[2] - self.dx, b[3] - self.dy]

    def ellipse(self, box, **kw):
        self.d.ellipse(self._box(box), **kw)

    def rounded_rectangle(self, box, **kw):
        self.d.rounded_rectangle(self._box(box), **kw)


def overlay(name, img, region):
    region = ndi.binary_dilation(region, iterations=3) & head
    alpha = ndi.gaussian_filter(region.astype(float), 1.2)
    img = np.dstack([img.clip(0, 255), (alpha * 255).clip(0, 255)]).astype(np.uint8)
    parts[name] = save_centered(name, img, NECK, head)


for key, shapes in MOUTHS.items():
    overlay(f"mouth_{key}", *draw_mouth(shapes))

blink = load("blink.png")
eyes = np.zeros((H, W), bool)
ex0, ey0, ex1, ey1 = 280, 330, 750, 560
eyes[ey0:ey1, ex0:ex1] = (np.abs(blink - master).sum(2) > 45)[ey0:ey1, ex0:ex1]
eyes = ndi.binary_dilation(ndi.binary_opening(eyes, iterations=1), iterations=8)
overlay("eyes_closed", blink, eyes)

# ---- the manifest ----------------------------------------------------------------------

PARENT = {"torso": None, "head": "torso",
          "upper_arm_left": "torso", "forearm_left": "upper_arm_left",
          "upper_arm_right": "torso", "forearm_right": "upper_arm_right"}
ORDER = ["torso", "forearm_left", "forearm_right", "upper_arm_left", "upper_arm_right", "head",
         "mouth_open", "mouth_round", "mouth_small", "mouth_teeth", "mouth_closed", "eyes_closed"]
for name, p in parts.items():
    p["parent"] = PARENT.get(name, "head")
VISEMES = {}
for ids, mouth in (((1, 2, 9, 11), "open"), ((3, 7, 8, 10, 13, 16), "round"),
                   ((4, 5, 6, 12, 14, 17, 19, 20), "small"), ((15, 18), "teeth"), ((21,), "closed")):
    for i in ids:
        VISEMES[str(i)] = mouth
VISEMES["0"] = None
part_lines = ",\n  ".join(f'"{k}": {json.dumps(parts[k])}' for k in ORDER)
(OUT / "rig.json").write_text(
    f'{{\n "drawing": {json.dumps({"width": W, "height": H})},\n'
    f' "draw_order": {json.dumps(ORDER)},\n'
    f' "parts": {{\n  {part_lines}\n }},\n'
    f' "visemes": {json.dumps({k: VISEMES[k] for k in sorted(VISEMES, key=int)})}\n}}\n')

# The parts reassembled at rest, as one picture: what the rig looks like undeformed.
rest = Image.new("RGBA", (W, H), (0, 0, 0, 0))
for k in ORDER[:6]:
    p = parts[k]
    rest.alpha_composite(Image.open(OUT / p["file"]), (p["pivot"][0] - p["width"] // 2, p["pivot"][1] - p["height"] // 2))
rest.crop(rest.getbbox()).save(OUT / "rest.png", optimize=True)
for k, v in parts.items():
    print(k, v["width"], v["height"], v["pivot"], v["parent"])
