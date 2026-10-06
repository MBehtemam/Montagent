"""PROTOTYPE #722: writes every project the named-effects prototype renders.

effects.montagent.json     the clip the owner judges (six 3-second scenes, 1920x1080, 30 fps)
identity/<case>-{on,off}   an identity value written, and the member removed
shift/<case>-<k>.json      a grain element starting k frames later
reach/<case>-{on,off}      one element alone, with and without the member (reach check)
cost/<case>.json           one full-frame 1080p element per member, 30 frames
colour-bound.json          the four ADR-0049 colour filters beside a blur (bound criterion)
"""

import copy
import json
import pathlib

HERE = pathlib.Path(__file__).parent
VIDEO = "../../fixtures/en-halloween-decorating/reference/en-halloween-decorating.mp4"
FONT = "../../fixtures/benchmark/spy-trailer/fonts/Oswald-SemiBold.ttf"
SCENE = 3000


def project(tracks, duration, output="out/effects.mp4", fonts=True):
    p = {
        "frame": {"width": 1920, "height": 1080},
        "fps": 30,
        "background": "#000000",
        "duration": duration,
        "output": output,
        "tracks": tracks,
    }
    if fonts:
        p["fonts"] = {"oswald": [{"file": FONT}]}
        p["fontVendor"] = {FONT: {
            "licence": "OFL-1.1", "source": "google/fonts ofl/oswald, instanced wght=600",
            "sha256": "442420449b66e3f8a49025fbb229a8b4b1efa5f7be9458a7c3244498d34f8de9"}}
    return p


class Clip:
    def __init__(self):
        self.tracks = []

    def add(self, e):
        self.tracks.append({"name": e["id"], "layer": len(self.tracks), "elements": [e]})
        return e


def rect(id, start, *, x, y, w, h, fill="#808080", effects=(), end=None, **extra):
    e = {"id": id, "type": "rect", "start": start, "end": end or start + SCENE, "x": x, "y": y,
         "origin": "center", "width": w, "height": h, "fill": fill}
    e.update(extra)
    if effects:
        e["effects"] = list(effects)
    return e


def video(id, start, *, x, y, w, h, effects=(), source_start=11000, end=None, **extra):
    e = {"id": id, "type": "video", "start": start, "end": end or start + SCENE, "x": x, "y": y,
         "origin": "center", "width": w, "height": h, "source": VIDEO,
         "source_start": source_start, "source_end": source_start + ((end or start + SCENE) - start),
         "fit": "literal"}
    e.update(extra)
    if effects:
        e["effects"] = list(effects)
    return e


def text(id, start, words, *, x, y, size=40, color="#BFBFBF", w=None, h=None, effects=(),
         end=None, **extra):
    e = {"id": id, "type": "text", "start": start, "end": end or start + SCENE, "x": x, "y": y,
         "origin": "center", "width": w or int(len(words) * size * 0.62) + 40,
         "height": h or int(size * 1.5), "font": "oswald", "size": size, "color": color,
         "align": "center", "runs": [{"text": words}]}
    e.update(extra)
    if effects:
        e["effects"] = list(effects)
    return e


def grain(seed, amount, size, mono):
    return {"name": "grain", "seed": seed, "amount": amount, "size": size, "mono": mono}


def glow(threshold, radius=40, intensity=1.5):
    return {"name": "glow", "threshold": threshold, "radius": radius, "intensity": intensity}


def posterize(levels):
    return {"name": "posterize", "levels": levels}


def dblur(angle, length):
    return {"name": "directional_blur", "angle": angle, "length": length}


def blur(radius):
    return {"name": "blur", "radius": radius}


GRADIENT = {"gradient": "linear", "angle": 90, "stops": [
    {"offset": 0, "color": "#0B1F4D"}, {"offset": 0.5, "color": "#E8618C"},
    {"offset": 1, "color": "#FFE08A"}]}

# ---------------------------------------------------------------------------------------------
# The clip.
c = Clip()
t = 0
# 1. grain on video: mono true/false at size 1 and 3.
for i, (mono, size) in enumerate(((True, 1), (False, 1), (True, 3), (False, 3))):
    x = 240 + i * 480
    c.add(video(f"grain-video-{i}", t, x=x, y=500, w=440, h=782,
                effects=[grain(7 + i, 0.12, size, mono)]))
    c.add(text(f"label-1-{i}", t, f"grain mono:{str(mono).lower()} size:{size}", x=x, y=1010, size=34))
t += SCENE
# 2. the texture idiom: a grey rect with grain, blended overlay over footage (left half only).
c.add(video("footage", t, x=960, y=540, w=1920, h=3413, source_start=20000))
c.add(rect("grain-texture", t, x=480, y=540, w=960, h=1080, fill="#808080",
           effects=[grain(42, 0.15, 2, True)], blend="overlay"))
c.add(rect("label-2-bar", t, x=960, y=1030, w=1920, h=60, fill="#000000B0"))
c.add(text("label-2", t, "left: grey rect + grain(size 2, mono), blend overlay   |   right: bare footage",
           x=960, y=1030, size=34))
t += SCENE
# 3. grain on a rotated and scaled element: the cells ride the transform.
c.add(rect("grain-flat", t, x=360, y=500, w=400, h=400, effects=[grain(5, 0.5, 6, True)]))
c.add(rect("grain-rotated", t, x=960, y=500, w=400, h=400, effects=[grain(5, 0.5, 6, True)],
           rotation=[{"t": t, "v": 0}, {"t": t + SCENE, "v": 45, "ease": "linear"}]))
c.add(rect("grain-scaled", t, x=1560, y=500, w=200, h=200, effects=[grain(5, 0.5, 6, True)],
           scale=[2.5, 1.5]))
c.add(text("label-3a", t, "size 6, flat", x=360, y=900, size=34))
c.add(text("label-3b", t, "rotating 0 to 45 deg", x=960, y=900, size=34))
c.add(text("label-3c", t, "scale [2.5, 1.5]", x=1560, y=900, size=34))
t += SCENE
# 4. glow on bright text over a dark background, at two thresholds.
for i, (label, effects) in enumerate((
    ("no glow", []),
    ("glow threshold 1 (empty bright-pass)", [glow(1)]),
    ("glow threshold 0.5, radius 40, intensity 1.5", [glow(0.5)]),
)):
    y = 200 + i * 320
    c.add(text(f"glow-white-{i}", t, "GLOW", x=600, y=y, size=170, color="#FFFFFF", effects=effects))
    c.add(text(f"glow-amber-{i}", t, "GLOW", x=1180, y=y, size=170, color="#FFB347", effects=effects))
    c.add(text(f"label-4-{i}", t, label, x=1650, y=y, size=30, w=520))
t += SCENE
# 5. posterize on elements that also carry blur.
c.add(video("poster-4", t, x=330, y=480, w=480, h=853, effects=[blur(4), posterize(4)]))
c.add(video("poster-256", t, x=960, y=480, w=480, h=853, effects=[blur(4), posterize(256)]))
c.add(rect("poster-gradient", t, x=[{"t": t, "v": 1560}, {"t": t + SCENE, "v": 1583, "ease": "linear"}],
           y=480, w=480, h=600, fill=GRADIENT, effects=[posterize(4), blur(6)]))
c.add(text("label-5a", t, "[blur 4, posterize 4]", x=330, y=1000, size=34))
c.add(text("label-5b", t, "[blur 4, posterize 256]", x=960, y=1000, size=34))
c.add(text("label-5c", t, "[posterize 4, blur 6], drifting", x=1570, y=1000, size=34))
t += SCENE
# 6. directional blur at 0, 45, 90 degrees, length 40 (top) and 0 (bottom).
for i, angle in enumerate((0, 45, 90)):
    x = 360 + i * 600
    for j, length in enumerate((40, 0)):
        y = 300 + j * 420
        c.add(text(f"dblur-text-{i}-{j}", t, "SMEAR", x=x, y=y - 40, size=110, color="#FFFFFF",
                   effects=[dblur(angle, length)]))
        c.add(rect(f"dblur-dot-{i}-{j}", t, x=x, y=y + 90, w=60, h=60, fill="#38BDF8",
                   effects=[dblur(angle, length)]))
        c.add(text(f"label-6-{i}-{j}", t, f"angle {angle}, length {length}", x=x, y=y + 165, size=30))
t += SCENE
CLIP = project(c.tracks, t)

# ---------------------------------------------------------------------------------------------
# Identity: written identity value against the member removed.


def ident_text(effects):
    return [{"name": "s", "layer": 0, "elements": [
        text("subject", 0, "GLOW", x=[{"t": 0, "v": 900}, {"t": 1000, "v": 913, "ease": "linear"}],
             y=540, size=170, color="#FFB347", effects=effects, end=1000)]},
        {"name": "v", "layer": 1, "elements": [
            video("under", 0, x=960, y=540, w=1080, h=1920, end=1000)]}]


def ident_video(effects, rotation=7):
    return [{"name": "v", "layer": 0, "elements": [
        video("under", 0, x=960, y=540, w=1080, h=1920, end=1000)]},
        {"name": "s", "layer": 1, "elements": [
            video("subject", 0, x=[{"t": 0, "v": 700}, {"t": 1000, "v": 717, "ease": "linear"}],
                  y=540, w=480, h=853, effects=effects, end=1000, source_start=30000,
                  rotation=rotation, opacity=0.8)]}]


IDENTITIES = {
    "glow-threshold-1": (ident_text, [glow(1)], []),
    "posterize-256": (ident_video, [blur(4), posterize(256)], [blur(4)]),
    "posterize-256-alone": (ident_video, [posterize(256)], []),
    "posterize-256-unrotated": (lambda e: ident_video(e, 0), [posterize(256)], []),
    "posterize-256-vs-brightness-0": (ident_video, [posterize(256)], [{"name": "brightness", "amount": 0}]),
    "directional-length-0": (ident_text, [dblur(30, 0)], []),
    "grain-amount-0": (ident_video, [grain(1, 0, 1, False)], []),
}

# ---------------------------------------------------------------------------------------------
# Shift: the element starting k frames later.


def shift_project(k_ms):
    start = k_ms
    els = [
        rect("g1", start, x=500, y=540, w=600, h=600, fill=GRADIENT, effects=[grain(9, 0.4, 1, False)],
             end=start + 2000),
        rect("g2", start, x=1350, y=540, w=400, h=400, fill="#808080", effects=[grain(9, 0.5, 3, True)],
             rotation=20, scale=[1.3, 0.9], end=start + 2000),
    ]
    return project([{"name": e["id"], "layer": i, "elements": [e]} for i, e in enumerate(els)],
                   4000, fonts=False)


# ---------------------------------------------------------------------------------------------
# Reach: one element alone, on black, with and without the member.
REACH = {
    "glow-r40": glow(0.3, 40, 2.0),
    "glow-r10": glow(0.3, 10, 4.0),
    "dblur-0-40": dblur(0, 40),
    "dblur-45-40": dblur(45, 40),
    "dblur-90-40": dblur(90, 40),
    "dblur-30-41.5": dblur(30, 41.5),
}


def reach_project(member):
    e = rect("subject", 0, x=960, y=540, w=300, h=200, fill="#FFFFFF",
             effects=[member] if member else [], end=100)
    return project([{"name": "s", "layer": 0, "elements": [e]}], 100, fonts=False)


# ---------------------------------------------------------------------------------------------
# Cost: one full-frame 1080p element, 30 frames.
COST = {
    "none": [],
    "blur-r40": [blur(40)],
    "grain-mono-1": [grain(3, 0.2, 1, True)],
    "grain-colour-1": [grain(3, 0.2, 1, False)],
    "grain-mono-3": [grain(3, 0.2, 3, True)],
    "glow-t05-r40": [glow(0.5, 40, 1.5)],
    "glow-t1": [glow(1, 40, 1.5)],
    "posterize-4": [posterize(4)],
    "directional-0-40": [dblur(0, 40)],
    "directional-45-40": [dblur(45, 40)],
    "directional-90-40": [dblur(90, 40)],
    "brightness": [{"name": "brightness", "amount": 0.1}],
}


def cost_project(effects):
    e = rect("subject", 0, x=960, y=540, w=1920, h=1080, fill=GRADIENT, effects=effects, end=1000)
    return project([{"name": "s", "layer": 0, "elements": [e]}], 1000, fonts=False)


# ---------------------------------------------------------------------------------------------
# The four colour filters beside a blur, drifting, for the bound criterion.
def colour_bound():
    els = []
    for i, m in enumerate(({"name": "tint", "color": "#FF0000", "amount": 0.5},
                           {"name": "saturation", "amount": 0.2},
                           {"name": "brightness", "amount": 0.3},
                           {"name": "contrast", "amount": 0.6},
                           {"name": "brightness", "amount": -0.3},
                           {"name": "contrast", "amount": -0.6})):
        x0 = 170 + i * 315
        els.append(rect(f"c{i}", 0, x=[{"t": 0, "v": x0}, {"t": 2000, "v": x0 + 17, "ease": "linear"}],
                        y=540, w=220, h=300, fill=GRADIENT, effects=[blur(10), m], end=2000,
                        rotation=[{"t": 0, "v": 0}, {"t": 2000, "v": 10, "ease": "linear"}]))
    return project([{"name": e["id"], "layer": i, "elements": [e]} for i, e in enumerate(els)],
                   2000, fonts=False)


# directional_blur with its declared-reach bound: drifting, rotating, scaled, after a blur and
# before a shadow, at angles that put the reach on both axes.
def dblur_bound():
    els = []
    cases = (
        ("d0", 0, 40, {}),
        ("d45", 45, 40, {"rotation": [{"t": 0, "v": 0}, {"t": 3000, "v": 25, "ease": "linear"}]}),
        ("d90", 90, 40, {"scale": [1.7, 0.8]}),
        ("d30", 30, 41.5, {"scale": [-1, 1]}),
        ("d135", 135, 120, {}),
        ("d-chain", 60, 30, {"effects_before": [blur(8)], "effects_after": [
            {"name": "shadow", "dx": 12, "dy": 12, "radius": 10, "color": "#38BDF8", "opacity": 0.8}]}),
    )
    for i, (id, angle, length, extra) in enumerate(cases):
        x0 = 200 + (i % 3) * 760
        y = 300 if i < 3 else 780
        before = extra.pop("effects_before", [])
        after = extra.pop("effects_after", [])
        els.append(text(id, 0, "SMEAR", x=[{"t": 0, "v": x0}, {"t": 3000, "v": x0 + 23, "ease": "linear"}],
                        y=y, size=110, color="#FFFFFF", end=3000,
                        effects=before + [dblur(angle, length)] + after, **extra))
    return project([{"name": e["id"], "layer": i, "elements": [e]} for i, e in enumerate(els)], 3000)


def write(path, data):
    path = HERE / path
    path.parent.mkdir(parents=True, exist_ok=True)
    body = json.dumps(data, indent=1) + "\n"
    if "/" in str(path.relative_to(HERE)):
        body = body.replace('"../../fixtures', '"../../../fixtures')
    path.write_text(body)


if __name__ == "__main__":
    write("effects.montagent.json", CLIP)
    for case, (make, on, off) in IDENTITIES.items():
        write(f"identity/{case}-on.json", project(make(on), 1000, fonts=True))
        write(f"identity/{case}-off.json", project(make(off), 1000, fonts=True))
    for k in (0, 100, 1000):  # 0, 3 and 30 frames at 30 fps
        write(f"shift/shift-{k}.json", shift_project(k))
    for case, member in REACH.items():
        write(f"reach/{case}-on.json", reach_project(member))
    write("reach/off.json", reach_project(None))
    for case, effects in COST.items():
        write(f"cost/{case}.json", cost_project(effects))
    write("colour-bound.json", colour_bound())
    write("dblur-bound.json", dblur_bound())
    flip = text("s", 0, "GLOW", x=[{"t": 0, "v": 900}, {"t": 1000, "v": 917, "ease": "linear"}],
                y=540, size=170, color="#FFD27A", effects=[dblur(30, 41.5)], end=1000, scale=[-1, 1])
    write("flip-dblur.json", project([{"name": "s", "layer": 0, "elements": [flip]}], 1000))
