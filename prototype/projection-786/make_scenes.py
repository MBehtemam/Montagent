#!/usr/bin/env python3
"""Prototype #786: write the nine ADR-0167 scene projects, 1280x720 at 30 fps.

Run from the repo root:  python3 prototype/projection-786/make_scenes.py
Media are referenced from the repo's own fixtures, copied next to the projects.
"""
import json
import math
import os
import shutil

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.normpath(os.path.join(HERE, "..", ".."))
OUT = os.path.join(HERE, "projects")
W, H = 1280, 720

FONT = {
    "fonts": {"cinzel-bold": [{"file": "media/Cinzel-Bold.ttf"}]},
    "fontVendor": {"media/Cinzel-Bold.ttf": {
        "licence": "OFL-1.1",
        "source": "google/fonts ofl/cinzel, instanced wght=700",
        "sha256": "c9ac320a5f48ecb57db76bc0b942ccc39eb89994e37fb182a454ec273ee43e02"}},
}


def keys(*pairs, ease="ease-in-out"):
    out = []
    for i, (t, v) in enumerate(pairs):
        record = {"t": t, "v": v}
        if i > 0:
            record["ease"] = ease
        out.append(record)
    return out


def project(name, elements, duration=3000, background="#14161C", fonts=False):
    tracks = [{"name": f"t{i}", "layer": i, "elements": [e]} for i, e in enumerate(elements)]
    doc = {"frame": {"width": W, "height": H}, "fps": 30, "background": background,
           "duration": duration, "output": f"out/{name}.mp4"}
    if fonts:
        doc.update(FONT)
    doc["tracks"] = tracks
    return doc


def card(id_, w, h, **kw):
    e = {"id": id_, "type": "image", "start": 0, "end": kw.pop("end", 3000),
         "source": "media/cards.jpg", "x": W // 2, "y": H // 2, "width": w, "height": h,
         "fit": "literal"}
    e.update(kw)
    return e


def rect(id_, w, h, fill, **kw):
    e = {"id": id_, "type": "rect", "start": 0, "end": kw.pop("end", 3000),
         "x": W // 2, "y": H // 2, "width": w, "height": h, "fill": fill}
    e.update(kw)
    return e


def mask(radius):
    return {"name": "mask", "shape": "rect", "radius": radius}


def shadow():
    return {"name": "shadow", "dx": 18, "dy": 26, "radius": 24, "color": "#000000", "opacity": 0.75}


def blur(radius):
    return {"name": "blur", "radius": radius}


def scenes():
    s = {}
    # 1. A card flip: front 0 -> 180, back -180 -> 0, each gone past 90.
    s["01-card-flip"] = project("01-card-flip", [
        card("front", 480, 300, swivel=keys((300, 0), (2700, 180)), perspective=1200,
             effects=[mask(28)]),
        rect("back", 480, 300, {"gradient": "linear", "angle": 135, "stops": [
            {"offset": 0, "color": "#2B5CE6"}, {"offset": 1, "color": "#9B2BE6"}]},
             radius=28, stroke="#FFFFFF", stroke_width=10,
             swivel=keys((300, -180), (2700, 0)), perspective=1200),
    ])
    # 2. A door swing about center-left.
    s["02-door-swing"] = project("02-door-swing", [
        rect("frame", 420, 520, "#3A2A1A", x=440 - 10, y=H // 2, origin="center-left"),
        card("door", 400, 500, x=440, origin="center-left",
             swivel=keys((300, 0), (1500, -75), (2700, 0)), perspective=900),
    ])
    # 3. A tilted screen: a video with tilt and swivel both keyed.
    s["03-tilted-screen"] = project("03-tilted-screen", [
        {"id": "screen", "type": "video", "start": 0, "end": 3000, "source": "media/clip.mp4",
         "source_start": 0, "source_end": 3000, "x": W // 2, "y": H // 2,
         "width": 640, "height": 360, "fit": "literal",
         "tilt": keys((0, 0), (1500, 28), (3000, 0)),
         "swivel": keys((0, -30), (3000, 30), ease="linear"), "perspective": 1100,
         "effects": [mask(16)]},
    ])
    # 4. A projected card with blur, shadow and mask: the shadow tilts with the card.
    s["04-blur-shadow-mask"] = project("04-blur-shadow-mask", [
        rect("floor", W, 200, "#5D6577", y=H - 100),
        card("card", 520, 325, swivel=keys((0, -40), (3000, 40), ease="linear"),
             tilt=keys((0, 25), (1500, -15), (3000, 25)), perspective=1100,
             effects=[mask(36), shadow(), blur(3)]),
    ], background="#8E97AA")
    # 5. The same card with grain and directional_blur.
    s["05-grain-directional"] = project("05-grain-directional", [
        rect("floor", W, 200, "#5D6577", y=H - 100),
        card("card", 520, 325, swivel=keys((0, -40), (3000, 40), ease="linear"),
             tilt=keys((0, 25), (1500, -15), (3000, 25)), perspective=1100,
             effects=[mask(36), shadow(),
                      {"name": "grain", "seed": 7, "amount": 0.18, "size": 3, "mono": True},
                      {"name": "directional_blur", "angle": 0, "length": 24}]),
    ], background="#8E97AA")
    # 6. Projected text and a projected path.
    s["06-text-and-path"] = project("06-text-and-path", [
        {"id": "title", "type": "text", "start": 0, "end": 3000, "font": "cinzel-bold",
         "size": 150, "color": "#E3C067", "align": "center", "runs": [{"text": "SPY"}],
         "x": 380, "y": H // 2, "width": 420, "height": 200,
         "swivel": keys((0, -60), (1500, 60), (3000, -60)), "perspective": 900},
        {"id": "star", "type": "path", "start": 0, "end": 3000, "x": 920, "y": H // 2,
         "width": 320, "height": 300, "closed": True, "fill": "#FFCC00",
         "stroke": "#FFFFFF", "stroke_width": 6,
         "points": [{"at": [160, 10]}, {"at": [224, 208]}, {"at": [56, 86]},
                    {"at": [264, 86]}, {"at": [96, 208]}],
         "tilt": keys((0, -55), (1500, 55), (3000, -55)), "perspective": 900},
    ], fonts=True)
    # 7. Projected, then rotated and scaled: the projection applies first, so the
    # trapezoid turns in the frame as a picture.
    s["07-project-then-rotate"] = project("07-project-then-rotate", [
        card("ghost", 400, 250, opacity=0.25, rotation=keys((0, 0), (3000, 90), ease="linear"),
             scale=keys((0, [1, 1]), (3000, [1.4, 0.8]), ease="linear")),
        card("card", 400, 250, swivel=50, perspective=800,
             rotation=keys((0, 0), (3000, 90), ease="linear"),
             scale=keys((0, [1, 1]), (3000, [1.4, 0.8]), ease="linear")),
    ])
    # 8. Motion blur: the card passes 90 within one frame (27 degrees a frame).
    s["08-motion-blur-flip"] = project("08-motion-blur-flip", [
        card("front", 480, 300, swivel=keys((1000, 0), (1200, 180), ease="linear"),
             perspective=1200, effects=[mask(28)],
             motion_blur={"shutter": 360, "samples": 16}),
        rect("back", 480, 300, "#2B5CE6", radius=28, stroke="#FFFFFF", stroke_width=10,
             swivel=keys((1000, -180), (1200, 0), ease="linear"), perspective=1200,
             motion_blur={"shutter": 360, "samples": 16}),
    ])
    # 9. A perspective just above the eye bound: r = |corner - origin|, here 183.58, so perspective 184.
    w9, h9 = 320, 180
    r = math.hypot(w9 / 2, h9 / 2)
    s["09-eye-bound"] = project("09-eye-bound", [
        card("card", w9, h9, swivel=keys((0, -70), (1500, 70), (3000, -70)),
             perspective=math.floor(r) + 1),
    ])
    return s


def main():
    os.makedirs(os.path.join(OUT, "media"), exist_ok=True)
    for src in ["fixtures/benchmark/spy-trailer/img/cards.jpg",
                "fixtures/skills/media/clip.mp4",
                "fixtures/benchmark/spy-trailer/fonts/Cinzel-Bold.ttf"]:
        shutil.copyfile(os.path.join(ROOT, src), os.path.join(OUT, "media", os.path.basename(src)))
    for name, doc in scenes().items():
        with open(os.path.join(OUT, f"{name}.montagent.json"), "w") as f:
            json.dump(doc, f, indent=1)
            f.write("\n")
    print("wrote", len(scenes()), "projects to", os.path.relpath(OUT, ROOT))


if __name__ == "__main__":
    main()
