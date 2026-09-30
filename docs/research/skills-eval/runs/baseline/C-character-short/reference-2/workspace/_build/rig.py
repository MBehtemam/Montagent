"""Cut-out rig renderer for Hoot, built on Pillow only (no numpy on this machine)."""
import json
import math
import os

from PIL import Image

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
CHAR = os.path.join(ROOT, "character")


# ---- 3x3 affine helpers (row-major tuples) --------------------------------
def mul(a, b):
    return tuple(
        tuple(sum(a[i][k] * b[k][j] for k in range(3)) for j in range(3))
        for i in range(3)
    )


def T(x, y):
    return ((1, 0, x), (0, 1, y), (0, 0, 1))


def S(sx, sy):
    return ((sx, 0, 0), (0, sy, 0), (0, 0, 1))


def R(deg):
    """Rotation; positive = clockwise on screen (y points down)."""
    r = math.radians(deg)
    c, s = math.cos(r), math.sin(r)
    return ((c, -s, 0), (s, c, 0), (0, 0, 1))


def inv(m):
    a, b, c = m[0]
    d, e, f = m[1]
    det = a * e - b * d
    ia, ib = e / det, -b / det
    id_, ie = -d / det, a / det
    return ((ia, ib, -(ia * c + ib * f)), (id_, ie, -(id_ * c + ie * f)), (0, 0, 1))


def apply(m, x, y):
    return (m[0][0] * x + m[0][1] * y + m[0][2], m[1][0] * x + m[1][1] * y + m[1][2])


def paste_affine(canvas, img, m):
    """Composite premultiplied `img` onto RGBA `canvas`, mapping img pixels by m."""
    w, h = img.size
    pts = [apply(m, x, y) for x, y in ((0, 0), (w, 0), (0, h), (w, h))]
    x0 = max(0, int(math.floor(min(p[0] for p in pts))) - 1)
    y0 = max(0, int(math.floor(min(p[1] for p in pts))) - 1)
    x1 = min(canvas.width, int(math.ceil(max(p[0] for p in pts))) + 1)
    y1 = min(canvas.height, int(math.ceil(max(p[1] for p in pts))) + 1)
    if x1 <= x0 or y1 <= y0:
        return
    im = mul(inv(m), T(x0, y0))
    coeffs = (im[0][0], im[0][1], im[0][2], im[1][0], im[1][1], im[1][2])
    patch = img.transform((x1 - x0, y1 - y0), Image.AFFINE, coeffs, resample=Image.BICUBIC)
    canvas.alpha_composite(patch.convert("RGBA"), dest=(x0, y0))


class Rig:
    MOUTHS = ["open", "round", "small", "teeth", "closed"]

    def __init__(self, scale):
        with open(os.path.join(CHAR, "rig.json")) as f:
            self.spec = json.load(f)
        self.scale = scale
        self.parts = {}
        for name, p in self.spec["parts"].items():
            im = Image.open(os.path.join(CHAR, p["file"])).convert("RGBA")
            w, h = im.size
            nw, nh = max(1, round(w * scale)), max(1, round(h * scale))
            im = im.resize((nw, nh), Image.LANCZOS).convert("RGBa")
            px, py = p["pivot"]
            # image pixel -> scaled drawing coords: centre of canvas sits on pivot
            place = T(px * scale - nw / 2, py * scale - nh / 2)
            self.parts[name] = dict(img=im, pivot=(px * scale, py * scale),
                                    parent=p["parent"], place=place)
        self.visemes = self.spec["visemes"]

    def local(self, name, deg):
        px, py = self.parts[name]["pivot"]
        return mul(T(px, py), mul(R(deg), T(-px, -py)))

    def draw(self, canvas, pose):
        """pose: x, y (screen position of feet pivot), lean, sx, sy, head,
        lift_l, bend_l, lift_r, bend_r (degrees), mouth (str|None), blink (bool)."""
        tp = self.parts["torso"]["pivot"]
        world = mul(T(pose["x"], pose["y"]),
                    mul(R(pose.get("lean", 0)),
                        mul(S(pose.get("sx", 1), pose.get("sy", 1)), T(-tp[0], -tp[1]))))
        rot = {
            "torso": 0,
            "head": pose.get("head", 0),
            "upper_arm_left": pose.get("lift_l", 0),
            "forearm_left": pose.get("bend_l", 0),
            "upper_arm_right": -pose.get("lift_r", 0),
            "forearm_right": -pose.get("bend_r", 0),
        }
        mats = {}

        def mat(name):
            if name in mats:
                return mats[name]
            p = self.parts[name]
            parent = world if p["parent"] is None else mat(p["parent"])
            m = mul(parent, self.local(name, rot.get(name, 0)))
            mats[name] = m
            return m

        mouth = pose.get("mouth")
        for name in self.spec["draw_order"]:
            if name.startswith("mouth_") and name != f"mouth_{mouth}":
                continue
            if name == "eyes_closed" and not pose.get("blink"):
                continue
            p = self.parts[name]
            paste_affine(canvas, p["img"], mul(mat(name), p["place"]))
