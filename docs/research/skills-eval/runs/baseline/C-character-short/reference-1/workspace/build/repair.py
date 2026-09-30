"""Repair the rig's shoulders so the arms can rotate without coming apart.

As delivered, `upper_arm_*` is only a shoulder disc and an elbow cap; the sleeve between
them is painted on the torso, together with a cut outline and a small wedge under the arm.
Any real rotation of the upper arm opens a gap there. This writes, into build/parts/:

- torso.png: the same torso with the shoulder stub and underarm wedge trimmed off and a
  clean ink contour drawn down each side of the body.
- upper_arm_{left,right}.png: a whole sleeve, round shoulder cap to round elbow cap, in the
  owl's own coral and ink, on a canvas centred on the same shoulder pivot.
"""
import json, math, os
import numpy as np
from PIL import Image, ImageDraw, ImageFilter

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CH = os.path.join(ROOT, "character")
OUT = os.path.join(ROOT, "build", "parts")
os.makedirs(OUT, exist_ok=True)
rig = json.load(open(os.path.join(CH, "rig.json")))

INK = (53, 39, 41, 255)
CORAL = (241, 130, 110, 255)
STROKE = 14
SS = 4  # supersampling
MIRROR = 1022  # right side x = MIRROR - left side x


def catmull(pts, n=12):
    out = []
    P = [pts[0]] + pts + [pts[-1]]
    for i in range(1, len(P) - 2):
        p0, p1, p2, p3 = P[i - 1], P[i], P[i + 1], P[i + 2]
        for k in range(n):
            t = k / n
            out.append(tuple(0.5 * ((2 * p1[j]) + (-p0[j] + p2[j]) * t
                                    + (2 * p0[j] - 5 * p1[j] + 4 * p2[j] - p3[j]) * t * t
                                    + (-p0[j] + 3 * p1[j] - 3 * p2[j] + p3[j]) * t ** 3)
                             for j in range(2)))
    out.append(pts[-1])
    return out


def thick_line(d, pts, w, fill):
    """A round-capped stroke, stamped as discs so its edge stays smooth."""
    r = w / 2
    for (x0, y0), (x1, y1) in zip(pts, pts[1:]):
        n = max(1, int(math.hypot(x1 - x0, y1 - y0) / 2))
        for k in range(n + 1):
            x, y = x0 + (x1 - x0) * k / n, y0 + (y1 - y0) * k / n
            d.ellipse([x - r, y - r, x + r, y + r], fill=fill)


# ------------------------------------------------------------------ torso
tp = rig["parts"]["torso"]
torso = Image.open(os.path.join(CH, tp["file"])).convert("RGBA")
ox = tp["pivot"][0] - tp["width"] / 2
oy = tp["pivot"][1] - tp["height"] / 2

# left-side contour, drawing coords: from the shoulder-top outline down into the body line
CURVE_L = [(336, 685), (318, 693), (308, 712), (304, 745), (304, 790), (306, 825), (307, 852)]
curve_l = catmull(CURVE_L)


def side(curve, mirror):
    pts = [((MIRROR - x) if mirror else x, y) for x, y in curve]
    outer = 240 if not mirror else MIRROR - 240
    erase = [(pts[0][0], 650)] + pts + [(outer, pts[-1][1]), (outer, 650)]
    return pts, erase


big_size = (torso.width * SS, torso.height * SS)
mask = Image.new("L", big_size, 0)
ink = Image.new("L", big_size, 0)
under = Image.new("L", big_size, 0)
md, idr, ud = ImageDraw.Draw(mask), ImageDraw.Draw(ink), ImageDraw.Draw(under)
to_c = lambda p: ((p[0] - ox) * SS, (p[1] - oy) * SS)
for mirror in (False, True):
    pts, erase = side(curve_l, mirror)
    md.polygon([to_c(p) for p in erase], fill=255)
    thick_line(idr, [to_c(p) for p in pts], STROKE * SS, 255)
    # inside the new contour: the underarm pocket was transparent (hidden by the arm)
    inner_x = 362 if not mirror else MIRROR - 362
    ud.polygon([to_c(p) for p in pts + [(inner_x, pts[-1][1]), (inner_x, 690)]], fill=255)
mask = mask.resize(torso.size, Image.LANCZOS)
ink = ink.resize(torso.size, Image.LANCZOS)
under = under.resize(torso.size, Image.LANCZOS)

# back-fill the pocket with coral, then erase outside the contour
fill = Image.new("RGBA", torso.size, CORAL[:3] + (0,))
fill.putalpha(under)
torso = Image.alpha_composite(fill, torso)
a = torso.getchannel("A")
a = Image.composite(Image.new("L", torso.size, 0), a, mask)
torso.putalpha(a)
# lay the ink contour on top
ink_layer = Image.new("RGBA", torso.size, INK)
ink_layer.putalpha(ink)
torso = Image.alpha_composite(torso, ink_layer)
torso.save(os.path.join(OUT, "torso.png"))


# ------------------------------------------------------------------ head
# The head's lower rim carries a few cut-off outline spurs; an opening of its alpha along
# the bottom (below the eyes) removes them without touching the ear tufts.
head = Image.open(os.path.join(CH, rig["parts"]["head"]["file"])).convert("RGBA")
a = head.getchannel("A")
opened = a.filter(ImageFilter.MinFilter(11)).filter(ImageFilter.MaxFilter(11))
band = Image.new("L", head.size, 0)
ImageDraw.Draw(band).rectangle([0, 470, head.width, head.height], fill=255)
a = Image.composite(Image.fromarray(np.minimum(np.array(a), np.array(opened))), a, band)
# ...and the two little ledges left at the sides fall outside the rim's own ellipse
# (fitted to the head's bottom contour: centre (305, 320), semi-axes 301 x 222 on the canvas)
ell = Image.new("L", (head.width * SS, head.height * SS), 0)
ImageDraw.Draw(ell).ellipse([(305 - 301) * SS, (320 - 222) * SS, (305 + 301) * SS, (320 + 222) * SS],
                            fill=255)
ell = ell.resize(head.size, Image.LANCZOS)
keep = Image.composite(ell, Image.new("L", head.size, 255), band)
head.putalpha(Image.fromarray(np.minimum(np.array(a), np.array(keep))))
head.save(os.path.join(OUT, "head.png"))

# ------------------------------------------------------------------ upper arms
def sleeve(shoulder, elbow, size, r_sh=50, r_arm=46, r_el=53):
    w, h = size
    cx, cy = w / 2, h / 2
    S = (cx, cy)
    E = (cx + elbow[0] - shoulder[0], cy + elbow[1] - shoulder[1])

    def shape(grow):
        m = Image.new("L", (w * SS, h * SS), 0)
        d = ImageDraw.Draw(m)
        s = lambda p: (p[0] * SS, p[1] * SS)
        for (px, py), r in ((S, r_sh), (E, r_el)):
            r = (r + grow) * SS
            d.ellipse([px * SS - r, py * SS - r, px * SS + r, py * SS + r], fill=255)
        dx, dy = E[0] - S[0], E[1] - S[1]
        L = math.hypot(dx, dy)
        nx, ny = -dy / L, dx / L
        r = r_arm + grow
        quad = [(S[0] + nx * r, S[1] + ny * r), (E[0] + nx * r, E[1] + ny * r),
                (E[0] - nx * r, E[1] - ny * r), (S[0] - nx * r, S[1] - ny * r)]
        d.polygon([s(p) for p in quad], fill=255)
        return m.resize((w, h), Image.LANCZOS)

    outer = shape(0)
    inner = shape(-STROKE)
    im = Image.new("RGBA", (w, h), INK)
    im.putalpha(outer)
    fill = Image.new("RGBA", (w, h), CORAL)
    fill.putalpha(inner)
    return Image.alpha_composite(im, fill)


meta = {}
for side_name, fore in (("left", "forearm_left"), ("right", "forearm_right")):
    up = rig["parts"]["upper_arm_" + side_name]
    el = rig["parts"][fore]["pivot"]
    size = (320, 230)
    im = sleeve(up["pivot"], el, size)
    im.save(os.path.join(OUT, f"upper_arm_{side_name}.png"))
    meta["upper_arm_" + side_name] = {"width": size[0], "height": size[1]}
json.dump(meta, open(os.path.join(OUT, "sizes.json"), "w"))
print("repaired parts written to", OUT)
