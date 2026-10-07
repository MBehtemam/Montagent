#!/usr/bin/env python3
"""PROTOTYPE #764: writes every project of the text-on-a-path prototype (ADR-0161 §9).

- textpath.montagent.json  the clip: six scenes, 24 s, 1920x1080, 30 fps, all 13 cases.
- contain/<case>.json      each case's text element(s) alone, on black, untransformed,
                           in a frame with 300 px of margin round the box (containment).
- plain.json               the clip with every `path` and `path_offset` dropped (cost).
- errors/*.json            one minimal bad file per `validate` code.
- shift/keyed.json         a keyed-`points` text for `shift`.
- reverse/*.json           the arc and the circle reversed by hand (the `path_reverse` question).

Every element sits in its own track. A grey 1-px rect marks each text's declared box and a
blue one the inset box [m, w-m] x [m, h-m]; a grey 2-px `path` element draws the guide with
the same points (ADR-0161 §2: a guide that should show is a separate `path`).
"""
import copy
import json
import math
import os

HERE = os.path.dirname(os.path.abspath(__file__))
OSWALD = "../../fixtures/benchmark/spy-trailer/fonts/Oswald-SemiBold.ttf"
NASKH = "../../fixtures/letter-spacing/fonts/NotoNaskhArabic-Regular.ttf"
FPS = 30
SCENE = 4000


def head(width=1920, height=1080, duration=6 * SCENE, font_prefix=""):
    o, n = font_prefix + OSWALD, font_prefix + NASKH
    return {
        "frame": {"width": width, "height": height},
        "fps": FPS,
        "background": "#101418",
        "duration": duration,
        "fonts": {"brand": [{"file": o}, {"file": n}]},
        "fontVendor": {
            o: {"licence": "OFL-1.1", "source": "google/fonts ofl/oswald, instanced wght=600",
                "sha256": "442420449b66e3f8a49025fbb229a8b4b1efa5f7be9458a7c3244498d34f8de9"},
            n: {"licence": "OFL-1.1", "source": "notofonts/arabic NotoNaskhArabic-v2.019",
                "sha256": "eb5cde7fecba8c6a481039257fe02d5fd69b7b0e36f8afc56ee22f4b1d7e8c21"},
        },
        "tracks": [],
    }


def m_of(t):
    sizes = [t["size"]] + [r["size"] for r in t["runs"] if "size" in r]
    strokes = [t.get("stroke_width", 0)] + [r.get("stroke_width", 0) for r in t["runs"]]
    return max(sizes) + max(strokes)


def text(id, x, y, w, h, size, runs, points, closed=False, **extra):
    t = {"id": id, "type": "text", "start": 0, "end": 0, "x": x, "y": y, "origin": "top-left",
         "width": w, "height": h, "font": "brand", "size": size, "color": "#FFFFFF",
         "runs": [r if isinstance(r, dict) else {"text": r} for r in runs], "caption": False,
         "path": {"closed": closed, "points": points}}
    t.update(extra)
    return t


def arc(w, h, m, lift):
    """An arc from bottom-left to bottom-right of the inset box, bulging up by about lift."""
    b = round(lift / 0.75)
    a = round((w - 2 * m) * 0.25)
    return [{"at": [m, h - m], "out": [a, -b]}, {"at": [w - m, h - m], "in": [-a, -b]}]


def circle(cx, cy, r, clockwise=True):
    k = round(r * 0.5523)
    pts = [
        {"at": [cx, cy - r], "in": [-k, 0], "out": [k, 0]},
        {"at": [cx + r, cy], "in": [0, -k], "out": [0, k]},
        {"at": [cx, cy + r], "in": [k, 0], "out": [-k, 0]},
        {"at": [cx - r, cy], "in": [0, k], "out": [0, -k]},
    ]
    return pts if clockwise else reverse(pts)


def reverse(pts):
    """Reverse a vertex list by hand: reverse the order and swap every `in` and `out`."""
    out = []
    for v in reversed(pts):
        n = {"at": v["at"]}
        if "out" in v:
            n["in"] = v["out"]
        if "in" in v:
            n["out"] = v["in"]
        out.append(n)
    return out


def wave(w, h, m, n, amp):
    """An open wave across the inset box, n vertices, alternating up and down."""
    xs = [m + round(i * (w - 2 * m) / (n - 1)) for i in range(n)]
    cy = h // 2
    seg = (w - 2 * m) / (n - 1)
    k = round(seg * 0.4)
    pts = []
    for i, x in enumerate(xs):
        y = cy - amp if i % 2 == 0 else cy + amp
        y = max(m, min(h - m, y))
        v = {"at": [x, y]}
        if i > 0:
            v["in"] = [-k, 0]
        if i < n - 1:
            v["out"] = [k, 0]
        pts.append(v)
    return pts


def decor(t, start, end):
    """The guide, the declared box and the inset box for text `t`."""
    m = m_of(t)
    pts = t["path"]["points"]
    guide = {"id": t["id"] + "-guide", "type": "path", "start": start, "end": end,
             "x": t["x"], "y": t["y"], "origin": "top-left", "width": t["width"],
             "height": t["height"], "closed": t["path"]["closed"], "stroke": "#5A6470",
             "stroke_width": 2, "points": pts}
    box = {"id": t["id"] + "-box", "type": "rect", "start": start, "end": end, "x": t["x"],
           "y": t["y"], "origin": "top-left", "width": t["width"], "height": t["height"],
           "stroke": "#3A3F46", "stroke_width": 1}
    inset = {"id": t["id"] + "-inset", "type": "rect", "start": start, "end": end,
             "x": t["x"] + m, "y": t["y"] + m, "origin": "top-left", "width": t["width"] - 2 * m,
             "height": t["height"] - 2 * m, "stroke": "#2B4C7E", "stroke_width": 1}
    return [box, inset, guide]


def label(id, x, y, words, start, end):
    return {"id": id, "type": "text", "start": start, "end": end, "x": x, "y": y,
            "origin": "top-left", "width": 900, "height": 40, "font": "brand", "size": 26,
            "color": "#8A96A3", "runs": [{"text": words}], "caption": False}


SHADOW = {"name": "shadow", "dx": 6, "dy": 8, "radius": 6, "color": "#000000", "opacity": 0.8}


def cases():
    """[(scene, case label, [text elements])], times filled in later."""
    s = []
    # Scene 0: 1 arc, 2 wave slide (two elements: on, then off), 3 circle badge.
    w, h, size = 820, 380, 72
    t1 = text("c1-arc", 60, 90, w, h, size, ["CAPCUT-STYLE ARC"], arc(w, h, size + 0, 150),
              align="center", path_offset=0.5)
    s.append((0, "1 arc, align center, path_offset 0.5", [t1]))
    w, h, size = 980, 300, 64
    pts = wave(w, h, size, 4, 60)
    t2a = text("c2-wave-on", 900, 60, w, h, size, ["SLIDING ON"], pts, align="end",
               path_offset=[{"t": 0, "v": 0}, {"t": 2000, "v": 1, "ease": "linear"}])
    t2b = text("c2-wave-off", 900, 60, w, h, size, ["SLIDING OFF"], pts, align="start",
               path_offset=[{"t": 2000, "v": 0}, {"t": 4000, "v": 1, "ease": "linear"}])
    s.append((0, "2 wave: on from the start end, then off past the far end", [t2a, t2b]))
    w = h = 520
    size = 54
    t3 = text("c3-badge", 1100, 450, w, h, size, ["MONTAGENT * TEXT ON A PATH * "],
              circle(260, 260, 260 - size - 16), closed=True,
              path_offset=[{"t": 0, "v": 0}, {"t": 4000, "v": 1, "ease": "linear"}],
              effects=[SHADOW])
    s.append((0, "3 closed circle badge wrapping, path_offset 0 -> 1", [t3]))

    # Scene 1: 4 Arabic on an arc (defaults), 11 Arabic with Latin digits, 12 mixed sizes.
    w, h, size = 900, 380, 72
    t4 = text("c4-arabic", 60, 80, w, h, size, ["بسم الله الرحمن الرحيم"], arc(w, h, size, 140))
    s.append((1, "4 Arabic on an arc, default align start, path_offset 0", [t4]))
    t11 = text("c11-mixed", 1000, 80, w, h, size, ["عام 2026 سعيد 123"], arc(w, h, size, 140),
               align="center", path_offset=0.5)
    s.append((1, "11 Arabic with Latin digits, centred", [t11]))
    w, h, size = 1500, 500, 44
    t12 = text("c12-sizes", 210, 490, w, h, size,
               [{"text": "BIG ", "size": 110}, {"text": "small ", "size": 44},
                {"text": "MID", "size": 76}],
               wave(w, h, 110, 3, 60), align="center", path_offset=0.5)
    s.append((1, "12 runs of sizes 110, 44, 76: m = 110", [t12]))

    # Scene 2: 5 letter stagger dropping in, 7 stroke on a curve.
    w, h, size = 900, 420, 80
    t5 = text("c5-stagger", 60, 120, w, h, size, ["DROP IN"], arc(w, h, size, 170),
              align="center", path_offset=0.5, effects=[SHADOW],
              units={"by": "letter", "every": 120,
                     "y": [{"t": 8000, "v": -200}, {"t": 8700, "v": 0, "ease": "ease-out"}],
                     "opacity": [{"t": 8000, "v": 0}, {"t": 8400, "v": 1, "ease": "linear"}]})
    s.append((2, "5 by: letter stagger, y -200 -> 0 (perpendicular to the curve)", [t5]))
    w, h, size = 900, 420, 84
    t7 = text("c7-stroke", 980, 120, w, h, size, ["OUTLINED"], wave(w, h, size + 6, 3, 50),
              align="center", path_offset=0.5, stroke="#FF3366", stroke_width=6)
    s.append((2, "7 text stroke 6 on a wave: m = 84 + 6", [t7]))

    # Scene 3: 6 waving banner (keyed points), 10 keyed points that change the length.
    w, h, size = 1100, 380, 64
    base = wave(w, h, size, 4, 50)
    flip = copy.deepcopy(base)
    for i, v in enumerate(flip):
        v["at"][1] = h - v["at"][1]
    t6 = text("c6-banner", 60, 80, w, h, size, ["WAVING BANNER"], [
        {"t": 12000, "v": base}, {"t": 13000, "v": flip, "ease": "ease-in-out"},
        {"t": 14000, "v": base, "ease": "ease-in-out"}, {"t": 15000, "v": flip, "ease": "ease-in-out"},
        {"t": 16000, "v": base, "ease": "ease-in-out"}], align="center", path_offset=0.5)
    s.append((3, "6 waving banner: keyed points", [t6]))
    w, h, size = 1200, 300, 60
    short = [{"at": [size, 200], "out": [60, -60]}, {"at": [360, 200], "in": [-60, -60]}]
    long = [{"at": [size, 200], "out": [300, -60]}, {"at": [w - size, 200], "in": [-300, -60]}]
    t10 = text("c10-grow", 60, 560, w, h, size, ["THE CURVE GROWS AND SHRINKS"], [
        {"t": 12000, "v": short}, {"t": 14000, "v": long, "ease": "linear"},
        {"t": 16000, "v": short, "ease": "linear"}])
    s.append((3, "10 keyed points change the length: the hidden set changes each frame", [t10]))

    # Scene 4: 8 tight curve (r 45 < line height 120), 9 text longer than the curve.
    w, h, size = 360, 360, 100
    r = 45
    cx, cy = 180, 230
    k = round(r * 0.5523)
    semi = [{"at": [cx - r, cy], "out": [0, -k]}, {"at": [cx, cy - r], "in": [-k, 0], "out": [k, 0]},
            {"at": [cx + r, cy], "in": [0, -k]}]
    t8a = text("c8-tight-fi", 80, 120, w, h, size, ["fi"], semi, align="center", path_offset=0.5,
               effects=[SHADOW])
    t8b = text("c8-tight-arabic", 480, 120, w, h, size, ["بسم"], semi, align="center",
               path_offset=0.5)
    s.append((4, "8 tight curve r 45 < line height 120: 'fi' and the joined piece 'بسم'", [t8a, t8b]))
    w, h, size = 600, 300, 56
    t9a = text("c9-long-open", 900, 80, w, h, size, ["THIS LINE IS FAR LONGER THAN ITS CURVE"],
               arc(w, h, size, 100))
    w = h = 400
    t9b = text("c9-long-closed", 1100, 480, w, h, size,
               ["ONE LOOP ONLY * THE REST IS NOT DRAWN * SEE QUERY"], circle(200, 200, 120),
               closed=True)
    s.append((4, "9 longer than the curve: open (tail hidden), closed (past one loop hidden)",
              [t9a, t9b]))

    # Scene 5: 13 align end near 1, and a stagger under an animated path_offset.
    w, h, size = 1100, 340, 64
    t13a = text("c13-end", 60, 60, w, h, size, ["ALIGN END AT 0.97"], wave(w, h, size, 3, 60),
                align="end", path_offset=0.97)
    t13b = text("c13-stagger-slide", 60, 520, 1700, 420, size, ["STAGGER WHILE SLIDING"],
                wave(1700, 420, size, 5, 80), align="start",
                path_offset=[{"t": 20000, "v": 0}, {"t": 24000, "v": 0.55, "ease": "ease-in-out"}],
                units={"by": "letter", "every": 80,
                       "y": [{"t": 20000, "v": -160}, {"t": 20600, "v": 0, "ease": "ease-out"}],
                       "rotation": [{"t": 20000, "v": -90}, {"t": 20600, "v": 0, "ease": "ease-out"}],
                       "opacity": [{"t": 20000, "v": 0}, {"t": 20300, "v": 1, "ease": "linear"}]})
    s.append((5, "13 align end at path_offset 0.97; a letter stagger under a sliding path_offset",
              [t13a, t13b]))
    return s


def timed(t, start, end):
    t = copy.deepcopy(t)
    if t["id"] == "c2-wave-on":
        start, end = start, start + 2000
    elif t["id"] == "c2-wave-off":
        start, end = start + 2000, end
    t["start"], t["end"] = start, end
    return t


def tracks_of(elements):
    return [{"name": e["id"], "layer": i, "elements": [e]} for i, e in enumerate(elements)]


def write(name, doc):
    path = os.path.join(HERE, name)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w") as f:
        json.dump(doc, f, ensure_ascii=False, indent=1)
        f.write("\n")


def main():
    all_cases = cases()
    elements = []
    texts = []
    for scene, words, ts in all_cases:
        start, end = scene * SCENE, (scene + 1) * SCENE
        for t in ts:
            t = timed(t, start, end)
            if t["id"] != "c2-wave-off":
                elements += decor(t, start, end)
            texts.append(t)
    labels = []
    for scene in range(6):
        words = " | ".join(w for s, w, _ in all_cases if s == scene)
        labels.append(label(f"label-{scene}", 40, 1030, words, scene * SCENE, (scene + 1) * SCENE))
        labels[-1]["width"] = 1840
    doc = head()
    doc["tracks"] = tracks_of(elements + texts + labels)
    write("textpath.montagent.json", doc)

    # Plain: every `path` and `path_offset` dropped (and the flat line's box widened so the
    # flat text still validates), for the cost comparison.
    plain = copy.deepcopy(doc)
    for tr in plain["tracks"]:
        for e in tr["elements"]:
            if e["type"] == "text" and "path" in e:
                del e["path"]
                e.pop("path_offset", None)
    write("plain.json", plain)

    # Containment: each case's text element(s) alone, on black, untransformed, the box at
    # (300, 300) in a frame 600 px larger than it.
    for scene, words, ts in all_cases:
        start, end = scene * SCENE, (scene + 1) * SCENE
        group = [timed(t, start, end) for t in ts]
        bw = max(t["width"] for t in group)
        bh = max(t["height"] for t in group)
        cdoc = head(width=bw + 600, height=bh + 600, font_prefix="../")
        cdoc["background"] = "#000000"
        cdoc["duration"] = 6 * SCENE
        els = []
        for t in group:
            t = copy.deepcopy(t)
            t["x"], t["y"] = 300, 300
            t.pop("effects", None)
            els.append(t)
        cdoc["tracks"] = tracks_of(els)
        name = ts[0]["id"].split("-")[0]
        write(f"contain/{name}.json", cdoc)

    # Minimal bad files, one per code.
    def mini(t, name):
        d = head(width=1000, height=600, duration=1000, font_prefix="../")
        t = copy.deepcopy(t)
        t["start"], t["end"] = 0, 1000
        d["tracks"] = tracks_of([t])
        write(f"errors/{name}.json", d)

    good = text("t", 40, 40, 700, 300, 60, ["HELLO"], arc(700, 300, 60, 100))
    b = copy.deepcopy(good); b["runs"] = [{"text": "TWO\nLINES"}]
    mini(b, "text-path-break")
    b = copy.deepcopy(good); del b["path"]; b["path_offset"] = 0.5
    mini(b, "text-path-offset-orphan")
    b = copy.deepcopy(good); b["path"]["points"] = [{"at": [100, 200]}]
    mini(b, "path-too-few-points")
    b = copy.deepcopy(good); b["path"]["points"][0]["in"] = [-10, 0]
    mini(b, "path-dangling-handle")
    b = copy.deepcopy(good)
    p0 = b["path"]["points"]
    p1 = copy.deepcopy(p0) + [{"at": [350, 200]}]
    b["path"]["points"] = [{"t": 0, "v": p0}, {"t": 1000, "v": p1}]
    mini(b, "path-keyframe-shape")
    b = copy.deepcopy(good); b["path"]["points"][0]["at"] = [30, 240]
    mini(b, "path-outside-box")
    b = copy.deepcopy(good); b["runs"] = [{"text": "BIG", "size": 100}, {"text": "small"}]
    mini(b, "path-outside-box-run-size")
    b = copy.deepcopy(good); b["path_offset"] = 1.5
    mini(b, "path-offset-range")
    b = copy.deepcopy(good); b["path"]["closed"] = [{"t": 0, "v": False}]
    mini(b, "path-closed-keyed")
    b = copy.deepcopy(good); b["path"]["stroke"] = "#FFFFFF"
    mini(b, "path-extra-key")

    # `shift`: a text whose `path.points` is keyed 1000 -> 3000.
    sdoc = head(width=1000, height=600, duration=5000, font_prefix="../")
    a = arc(700, 300, 60, 100)
    a2 = copy.deepcopy(a)
    a2[0]["out"] = [a2[0]["out"][0], a2[0]["out"][1] + 77]
    a2[1]["in"] = [a2[1]["in"][0], a2[1]["in"][1] + 77]
    st = text("keyed", 40, 40, 700, 300, 60, ["SHIFT ME"],
              [{"t": 1000, "v": a}, {"t": 3000, "v": a2, "ease": "linear"}],
              align="center", path_offset=[{"t": 1000, "v": 0.2}, {"t": 3000, "v": 0.8, "ease": "linear"}])
    st["start"], st["end"] = 0, 5000
    sdoc["tracks"] = tracks_of([st])
    write("shift/keyed.json", sdoc)

    # `path_reverse`: the arc and the circle reversed by hand, against the originals.
    rdoc = head(width=1920, height=1080, duration=1000, font_prefix="../")
    w, h, size = 820, 380, 72
    fwd = text("arc-forward", 60, 60, w, h, size, ["ALONG THE ARC"], arc(w, h, size, 150),
               align="center", path_offset=0.5)
    rev = text("arc-reversed", 60, 560, w, h, size, ["ALONG THE ARC"],
               reverse(arc(w, h, size, 150)), align="center", path_offset=0.5)
    cw = text("circle-clockwise", 1000, 40, 480, 480, 50, ["OUTSIDE THE CIRCLE * "],
              circle(240, 240, 170), closed=True, align="center", path_offset=0.0)
    ccw = text("circle-reversed", 1000, 560, 480, 480, 50, ["INSIDE THE CIRCLE * "],
               reverse(circle(240, 240, 170)), closed=True, align="center", path_offset=0.0)
    els = []
    for t in (fwd, rev, cw, ccw):
        t["start"], t["end"] = 0, 1000
        els += decor(t, 0, 1000)
    rdoc["tracks"] = tracks_of(els + [fwd, rev, cw, ccw])
    write("reverse/reverse.json", rdoc)


if __name__ == "__main__":
    main()
