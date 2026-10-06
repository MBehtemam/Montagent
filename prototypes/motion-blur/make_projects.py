#!/usr/bin/env python3
"""PROTOTYPE #718 — throwaway. Writes the motion-blur clip project, its no-blur twin, and
the cost projects, next to this file.

    python3 prototypes/motion-blur/make_projects.py
"""
import copy
import json
import pathlib

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parent.parent
FONT = "../../fixtures/benchmark/spy-trailer/fonts/Cinzel-Bold.ttf"
CLIP = "../../fixtures/skills/media/clip.mp4"

W, H, FPS = 1920, 1080, 30
SCENE = 3000


def key(*records):
    """[(t, v, ease?)] -> keyframe list."""
    out = []
    for r in records:
        rec = {"t": r[0], "v": r[1]}
        if len(r) > 2:
            rec["ease"] = r[2]
        elif out:
            rec["ease"] = "linear"
        out.append(rec)
    return out


def label(id_, text, x, y, start, end, size=30, colour="#C8CCD4"):
    return {
        "id": id_, "type": "text", "start": start, "end": end, "x": x, "y": y,
        "origin": "center-left", "width": 900, "height": size + 16, "font": "cinzel",
        "size": size, "color": colour, "align": "start", "runs": [{"text": text}],
        "caption": False,
    }


def mb(shutter=180, samples=16):
    return {"shutter": shutter, "samples": samples}


def clip_elements():
    els = []
    # ---- Scene 1: a fast slide on ease-out, sharp / 180 / 360 ----------------------
    s, e = 0, SCENE
    slide = key((200, 260), (650, 1660, "ease-out"), (1500, 1660), (1950, 260, "ease-out"))
    for i, (name, blur) in enumerate([
        ("sharp (no motion_blur)", None),
        ("shutter 180, samples 16", mb(180, 16)),
        ("shutter 360, samples 16", mb(360, 16)),
    ]):
        y = 300 + i * 260
        r = {"id": f"slide{i}", "type": "rect", "start": s, "end": e,
             "x": [dict(k, t=k["t"] + s) for k in slide], "y": y, "origin": "center",
             "width": 180, "height": 120, "radius": 16, "fill": "#F2B134"}
        if blur:
            r["motion_blur"] = blur
        els.append(r)
        els.append(label(f"slide{i}-label", name, 80, y - 100, s, e))
    els.append(label("s1", "1  FAST SLIDE, EASE-OUT", 80, 60, s, e, 40, "#FFFFFF"))

    # ---- Scene 2: a spinning bar, and a keyed size / radius / colour ---------------
    s, e = SCENE, 2 * SCENE
    spin = key((s, 0), (e, 1080, "linear"))
    for i, (x, blur, name) in enumerate([(480, None, "spin, sharp"), (1440, mb(180, 16), "spin, 180 / 16")]):
        r = {"id": f"bar{i}", "type": "rect", "start": s, "end": e, "x": x, "y": 360,
             "origin": "center", "width": 400, "height": 40, "radius": 22, "fill": "#7FD1FF",
             "rotation": spin}
        if blur:
            r["motion_blur"] = blur
        els.append(r)
        els.append(label(f"bar{i}-label", name, x - 440, 140, s, e, 26))
    grow = lambda a, b: key((s + 300, a), (s + 700, b, "ease-in-out"), (s + 1500, b),
                             (s + 1900, a, "ease-in-out"))
    for i, (x, blur, name) in enumerate([(480, None, "keyed size, radius, colour: sharp"),
                                          (1440, mb(180, 16), "keyed size, radius, colour: 180 / 16")]):
        r = {"id": f"morph{i}", "type": "rect", "start": s, "end": e, "x": x, "y": 820,
             "origin": "center", "width": grow(80, 560), "height": grow(80, 260),
             "radius": grow(0, 130),
             "fill": key((s + 300, "#FF3B5C"), (s + 700, "#3B7BFF", "ease-in-out"),
                         (s + 1500, "#3B7BFF"), (s + 1900, "#FF3B5C", "ease-in-out"))}
        if blur:
            r["motion_blur"] = blur
        els.append(r)
        els.append(label(f"morph{i}-label", name, x - 400, 600, s, e, 26))
    els.append(label("s2", "2  SPIN (ARC) AND KEYED VALUES", 80, 60, s, e, 40, "#FFFFFF"))

    # ---- Scene 3: a units stagger, and a still element carrying motion_blur --------
    s, e = 2 * SCENE, 3 * SCENE
    units = {"by": "letter", "every": 70,
             "y": key((s, -260), (s + 260, 0, "ease-out")),
             "x": key((s, 160), (s + 260, 0, "ease-out")),
             "opacity": key((s, 0), (s + 120, 1, "linear"))}
    for i, (y, blur, name) in enumerate([(330, None, "units stagger, sharp"),
                                         (700, mb(180, 16), "units stagger, 180 / 16")]):
        t = {"id": f"title{i}", "type": "text", "start": s, "end": e, "x": 960, "y": y,
             "origin": "center", "width": 1500, "height": 180, "font": "cinzel", "size": 150,
             "color": "#FFFFFF", "align": "center", "runs": [{"text": "MOTION"}],
             "units": copy.deepcopy(units), "caption": False}
        if blur:
            t["motion_blur"] = blur
        els.append(t)
        els.append(label(f"title{i}-label", name, 80, y - 140, s, e))
    els.append({"id": "still", "type": "rect", "start": s, "end": e, "x": 1700, "y": 980,
                "origin": "center", "width": 240, "height": 90, "radius": 12,
                "fill": "#5AD17A", "rotation": 8, "motion_blur": mb(360, 32)})
    els.append(label("still-label", "still, motion_blur 360 / 32", 1100, 980, s, e, 26))
    els.append(label("s3", "3  TITLE STAGGER AND A STILL ELEMENT", 80, 60, s, e, 40, "#FFFFFF"))

    # ---- Scene 4: moving video; effects + mask + blend ----------------------------
    s, e = 3 * SCENE, 4 * SCENE
    vx = key((s + 200, 360), (s + 900, 1560, "ease-in-out"), (s + 1700, 1560),
             (s + 2400, 360, "ease-in-out"))
    for i, (y, blur, name) in enumerate([(330, None, "video, sharp"), (760, mb(180, 16), "video, 180 / 16 (footage held)")]):
        v = {"id": f"video{i}", "type": "video", "start": s, "end": e, "source": CLIP,
             "source_start": 0, "source_end": SCENE, "x": vx, "y": y, "origin": "center",
             "width": 480, "height": 270, "fit": "cover"}
        if blur:
            v["motion_blur"] = blur
        els.append(v)
        els.append(label(f"video{i}-label", name, 80, y - 165, s, e, 26))
    els.append(label("s4", "4  MOVING VIDEO", 80, 60, s, e, 40, "#FFFFFF"))

    s, e = 4 * SCENE, 5 * SCENE
    # A striped ground so `screen` shows.
    for j in range(8):
        els.append({"id": f"stripe{j}", "type": "rect", "start": s, "end": e,
                    "x": 120 + j * 240, "y": 540, "origin": "center", "width": 120, "height": 1080,
                    "fill": "#2B4C7E" if j % 2 else "#7E2B4C"})
    ox = key((s + 200, 300), (s + 1000, 1620, "ease-out"), (s + 1700, 1620),
             (s + 2500, 300, "ease-out"))
    for i, (y, blur, name) in enumerate([(300, None, "shadow + mask + screen, sharp"),
                                         (780, mb(180, 16), "shadow + mask + screen, 180 / 16")]):
        el = {"id": f"fx{i}", "type": "rect", "start": s, "end": e, "x": ox, "y": y,
              "origin": "center", "width": 320, "height": 220, "fill": "#FFD24A",
              "rotation": key((s + 200, -20), (s + 1000, 25, "ease-out"), (s + 1700, 25),
                              (s + 2500, -20, "ease-out")),
              "blend": "screen",
              "effects": [
                  {"name": "mask", "shape": "ellipse"},
                  {"name": "shadow", "dx": 0, "dy": 0, "radius": 24, "color": "#FF6A00",
                   "opacity": 0.9},
                  {"name": "blur", "radius": 2},
              ]}
        if blur:
            el["motion_blur"] = blur
        els.append(el)
        els.append(label(f"fx{i}-label", name, 80, y - 180, s, e, 26, "#FFFFFF"))
    els.append(label("s5", "5  EFFECTS, MASK AND BLEND", 80, 60, s, e, 40, "#FFFFFF"))
    return els


def project(elements, output, duration):
    tracks = [{"name": f"t{i}", "layer": i, "elements": [el]} for i, el in enumerate(elements)]
    return {
        "frame": {"width": W, "height": H}, "fps": FPS, "background": "#14171C",
        "duration": duration, "output": output,
        "fonts": {"cinzel": [{"file": FONT}]},
        "fontVendor": {FONT: {
            "licence": "OFL-1.1",
            "source": "google/fonts ofl/cinzel, instanced wght=700",
            "sha256": "c9ac320a5f48ecb57db76bc0b942ccc39eb89994e37fb182a454ec273ee43e02"}},
        "tracks": tracks,
    }


def strip(elements):
    out = copy.deepcopy(elements)
    for el in out:
        el.pop("motion_blur", None)
    return out


def write(name, doc):
    (HERE / name).write_text(json.dumps(doc, indent=1) + "\n")


def main():
    els = clip_elements()
    write("motion-blur.montagent.json", project(els, "out/motion-blur.mp4", 5 * SCENE))
    write("no-blur.montagent.json", project(strip(els), "out/no-blur.mp4", 5 * SCENE))

    # Cost: one 1080p frame-filling scene, four fast-moving elements, 60 frames.
    def cost(samples):
        els = []
        for i in range(4):
            r = {"id": f"m{i}", "type": "rect", "start": 0, "end": 2000,
                 "x": key((0, 200), (2000, 1700, "linear")), "y": 150 + i * 260,
                 "origin": "center", "width": 240, "height": 160, "radius": 20,
                 "fill": "#F2B134", "rotation": key((0, 0), (2000, 90, "linear"))}
            if samples:
                r["motion_blur"] = mb(180, samples)
            els.append(r)
        return project(els, f"out/cost-{samples or 0}.mp4", 2000)

    def cost_fx(samples):
        doc = cost(samples)
        for t in doc["tracks"]:
            t["elements"][0]["effects"] = [{"name": "shadow", "dx": 0, "dy": 8, "radius": 16,
                                           "color": "#000000", "opacity": 0.6}]
        doc["output"] = f"out/cost-fx-{samples or 0}.mp4"
        return doc

    def cost_text(samples):
        doc = cost(samples)
        for t in doc["tracks"]:
            el = t["elements"][0]
            for k in ("radius", "fill"):
                el.pop(k)
            el.update({"type": "text", "font": "cinzel", "size": 96, "color": "#FFFFFF",
                       "runs": [{"text": "MOTION"}], "width": 600, "height": 130,
                       "align": "center", "caption": False})
        doc["output"] = f"out/cost-text-{samples or 0}.mp4"
        return doc

    # Still: elements carrying motion_blur that do not move, against the same with none.
    still = [
        {"id": "r", "type": "rect", "start": 0, "end": 1000, "x": 500, "y": 400,
         "origin": "center", "width": 300, "height": 160, "radius": 24, "fill": "#5AD17A",
         "rotation": 8, "motion_blur": mb(360, 32)},
        {"id": "e", "type": "ellipse", "start": 0, "end": 1000, "x": 1300, "y": 400,
         "origin": "center", "width": 260, "height": 180, "fill": "#7FD1FF",
         "effects": [{"name": "shadow", "dx": 0, "dy": 0, "radius": 20, "color": "#FF6A00",
                      "opacity": 0.9}], "blend": "screen", "motion_blur": mb(180, 8)},
        # Keyed, but every key lies before the element's life: clamped, so still.
        {"id": "k", "type": "rect", "start": 500, "end": 1000,
         "x": key((0, 200), (400, 960, "ease-out")), "y": 800, "origin": "center",
         "width": 200, "height": 100, "fill": "#F2B134", "motion_blur": mb(180, 16)},
        dict(label("txt", "A STILL TITLE", 1100, 800, 0, 1000, 60, "#FFFFFF"),
             motion_blur=mb(180, 16)),
    ]
    write("still-on.montagent.json", project(still, "out/still-on.mp4", 1000))
    write("still-off.montagent.json", project(strip(still), "out/still-off.mp4", 1000))

    for n in (0, 8, 16, 32):
        write(f"cost-{n}.montagent.json", cost(n))
        write(f"cost-fx-{n}.montagent.json", cost_fx(n))
        write(f"cost-text-{n}.montagent.json", cost_text(n))


if __name__ == "__main__":
    main()
