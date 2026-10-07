"""PROTOTYPE #750 — throwaway, never to merge. Writes every project this prototype renders.

- strokes.montagent.json -> out/strokes.mp4: the clip for the owner (seven scenes, 22 s).
- contain/<scene>.json: the same stroked elements alone on black, no guides, labels or
  effects, for the ink-outside-the-box check (check.py).
- errors.json, schema-*.json: one project per `validate` text.
- query.json: dashed shapes and a keyed offset, for `query --at`.
"""

import json
import math
import pathlib

HERE = pathlib.Path(__file__).parent
FONT = "../../fixtures/benchmark/spy-trailer/fonts/Oswald-SemiBold.ttf"
BG = "#1A1D22"
GUIDE = "#5A5F68"
INSET = "#2E5C8A"
INK = "#F2F2F2"
MARK = "#FF3B30"
SHADOW = {"name": "shadow", "dx": 10, "dy": 10, "radius": 8, "color": "#000000", "opacity": 0.9}

# (scene name, start ms, end ms)
SCENES = [
    ("miter", 0, 3000),
    ("caps", 3000, 6000),
    ("keyed-miter", 6000, 10000),
    ("dashed-shapes", 10000, 13000),
    ("seam", 13000, 16000),
    ("dotted", 16000, 19000),
    ("ants", 19000, 22000),
]


def pts(points, m):
    return [{"at": [x + m, y + m]} for x, y in points]


def path(id, *, x, y, w, h, closed, points, sw, color=INK, **extra):
    e = {"id": id, "type": "path", "x": x, "y": y, "origin": "top-left", "width": w,
         "height": h, "closed": closed, "stroke": color, "stroke_width": sw}
    e.update(extra)
    e["points"] = e.pop("points_", None) or points
    return e


def shape(id, kind, *, x, y, w, h, sw, color=INK, **extra):
    e = {"id": id, "type": kind, "x": x, "y": y, "origin": "top-left", "width": w, "height": h,
         "stroke": color, "stroke_width": sw}
    e.update(extra)
    return e


def inset_ceil(k, w):
    return math.ceil(k * w / 2 - 1e-9)


def square_inset(w):
    m = (w + 1) // 2
    while 2 * m * m < w * w:
        m += 1
    return m


def scene_elements():
    """{scene: [(element, label, inset m or None)]} — the stroked elements, no timing."""
    s = {}

    # 1. Miter joins at limits 1, 4 and 10 on one zigzag. The corner at (41, 0) is just
    #    inside limit 10 (1/sin(half-angle) = 9.81), so its tip nearly reaches the box edge.
    zig = [(0, 400), (41, 0), (82, 400), (200, 0), (300, 400), (480, 0)]
    sw = 16
    row, x = [], 60
    for limit in (1, 4, 10):
        m = inset_ceil(limit, sw)
        e = path(f"miter-{limit}", x=x, y=240, w=480 + 2 * m, h=400 + 2 * m, closed=False,
                 points=pts(zig, m), sw=sw, stroke_join="miter", stroke_miter_limit=limit)
        row.append((e, f"miter, limit {limit}: inset {m}", m))
        x += 480 + 2 * m + 60
    s["miter"] = row

    # 2. Caps on a 45-degree open line, stroke 40: butt, round, square; and a level square line.
    sw = 40
    row, x = [], 60
    for cap in ("butt", "round", "square"):
        m = square_inset(sw) if cap == "square" else inset_ceil(1, sw)
        e = path(f"cap-{cap}", x=x, y=300, w=300 + 2 * m, h=300 + 2 * m, closed=False,
                 points=pts([(0, 300), (300, 0)], m), sw=sw, stroke_cap=cap)
        row.append((e, f"{cap} cap, 45°: inset {m}", m))
        x += 300 + 2 * m + 80
    m = square_inset(sw)
    e = path("cap-square-level", x=x, y=420, w=300 + 2 * m, h=2 * m, closed=False,
             points=pts([(0, 0), (300, 0)], m), sw=sw, stroke_cap="square")
    row.append((e, f"square cap, level: inset {m}", m))
    s["caps"] = row

    # 3. Keyed points: the two arms swap sides, so the frames in between close the corner from
    #    41 degrees to 0 and back. Every keyframe's corner is 41 degrees.
    sw, limit = 16, 10
    m = inset_ceil(limit, sw)
    k0 = pts([(0, 0), (400, 150), (0, 300)], m)
    k1 = pts([(0, 300), (400, 150), (0, 0)], m)
    e = path("keyed-miter", x=640, y=280, w=400 + 2 * m, h=300 + 2 * m, closed=False,
             points=None, sw=sw, stroke_join="miter", stroke_miter_limit=limit,
             points_=[{"t": 0, "v": k0}, {"t": 4000, "v": k1, "ease": "linear"}])
    s["keyed-miter"] = [(e, f"keyed points, miter limit 10: inset {m}", m)]

    # 4. Dashed rect, rounded rect and ellipse, stroke 12, pattern [60, 20, 10, 20].
    dash = [60, 20, 10, 20]
    sw = 12
    s["dashed-shapes"] = [
        (shape("dash-rect", "rect", x=60, y=380, w=480, h=320, sw=sw, stroke_dash=dash),
         "rect: from the top-left corner, clockwise", None),
        (shape("dash-rrect", "rect", x=720, y=380, w=480, h=320, sw=sw, radius=80,
               stroke_dash=dash), "rect, radius 80: from the top-left arc's end", None),
        (shape("dash-ellipse", "ellipse", x=1380, y=380, w=480, h=320, sw=sw, stroke_dash=dash),
         "ellipse: from 3 o'clock, clockwise", None),
    ]

    # 5. The seam at points[0] on closed paths, pattern [50, 30] (total 80), miter limit 2.
    sw, dash = 12, [50, 30]
    m = inset_ceil(2, sw)
    sq = lambda side: [(0, 0), (side, 0), (side, side), (0, side)]  # noqa: E731
    r, c = 160, 0.5523 * 160
    circle = [{"at": [2 * r, r], "in": [0, -round(c)], "out": [0, round(c)]},
              {"at": [r, 2 * r], "in": [round(c), 0], "out": [-round(c), 0]},
              {"at": [0, r], "in": [0, round(c)], "out": [0, -round(c)]},
              {"at": [r, 0], "in": [-round(c), 0], "out": [round(c), 0]}]
    circle = [{**v, "at": [v["at"][0] + m, v["at"][1] + m]} for v in circle]
    s["seam"] = [
        (path("seam-310", x=80, y=330, w=310 + 2 * m, h=310 + 2 * m, closed=True,
              points=pts(sq(310), m), sw=sw, stroke_join="miter", stroke_miter_limit=2,
              stroke_dash=dash), "square 310: length 1240 = 15.5 × 80", m),
        (path("seam-333", x=560, y=330, w=333 + 2 * m, h=333 + 2 * m, closed=True,
              points=pts(sq(333), m), sw=sw, stroke_join="miter", stroke_miter_limit=2,
              stroke_dash=dash), "square 333: length 1332 = 16 × 80 + 52", m),
        (path("seam-circle", x=1080, y=330, w=2 * r + 2 * m, h=2 * r + 2 * m, closed=True,
              points=circle, sw=sw, stroke_join="miter", stroke_miter_limit=2,
              stroke_dash=dash), "circle of 4 cubics, r 160, from 3 o'clock", m),
    ]

    # 6. Dots: zero-length dashes under round caps, and one row under square caps.
    sw = 16
    m = inset_ceil(1, sw)
    ms = square_inset(sw)
    s["dotted"] = [
        (path("dots-line", x=100, y=300, w=760 + 2 * m, h=2 * m, closed=False,
              points=pts([(0, 0), (760, 0)], m), sw=sw, stroke_cap="round",
              stroke_dash=[0, 32]), "[0, 32], round cap", m),
        (path("dots-curve", x=100, y=420, w=760 + 2 * m, h=300 + 2 * m, closed=False,
              points=[{"at": [m, 300 + m], "out": [250, -300]},
                      {"at": [760 + m, m], "in": [-250, 300]}],
              sw=sw, stroke_cap="round", stroke_dash=[0, 32]), "[0, 32] on a cubic, round cap", m),
        (path("squares-diag", x=1060, y=300, w=700 + 2 * ms, h=400 + 2 * ms, closed=False,
              points=pts([(0, 400), (700, 0)], ms), sw=sw, stroke_cap="square",
              stroke_dash=[0, 40]), "[0, 40], square cap, on a diagonal", ms),
    ]

    # 7. Marching ants: [24, 16] (total 40), offset 0 -> -240 and 0 -> +240, linear, 3 s.
    sw, dash = 10, [24, 16]
    ants = lambda v: [{"t": 0, "v": 0}, {"t": 3000, "v": v, "ease": "linear"}]  # noqa: E731
    m = inset_ceil(1, sw)
    s["ants"] = [
        (shape("ants-minus", "rect", x=80, y=380, w=480, h=320, sw=sw, stroke_dash=dash,
               stroke_dash_offset=ants(-240)), "offset 0 to −6 × 40", None),
        (shape("ants-plus", "rect", x=720, y=380, w=480, h=320, sw=sw, radius=40,
               stroke_dash=dash, stroke_dash_offset=ants(240)), "offset 0 to +6 × 40", None),
        (path("ants-path", x=1380, y=380, w=440 + 2 * m, h=300 + 2 * m, closed=True,
              points=pts([(0, 300), (220, 0), (440, 300)], m), sw=sw, stroke_dash=dash,
              stroke_dash_offset=ants(-240)), "closed path, 0 to −6 × 40", m),
    ]
    return s


# Start markers for the dashed shapes and seams, in frame space: (x, y).
def markers(s):
    out = {}
    for e, _, m in s["dashed-shapes"]:
        x, y, w, h, half = e["x"], e["y"], e["width"], e["height"], e["stroke_width"] / 2
        if e["type"] == "ellipse":
            out[e["id"]] = (x + w - half, y + h / 2)
        elif e.get("radius"):
            out[e["id"]] = (x + e["radius"], y + half)
        else:
            out[e["id"]] = (x + half, y + half)
    for e, _, m in s["seam"]:
        a = e["points"][0]["at"]
        out[e["id"]] = (e["x"] + a[0], e["y"] + a[1])
    return out


def timed(e, start, end):
    e = dict(e)
    e = {"id": e.pop("id"), "type": e.pop("type"), "start": start, "end": end, **e}
    return e


def text(id, start, end, words, *, x, y, size=30, color="#C8CCD2", w=None):
    return {"id": id, "type": "text", "start": start, "end": end, "x": x, "y": y,
            "origin": "top-left", "width": w or 900, "height": int(size * 1.6),
            "font": "oswald", "size": size, "color": color, "align": "start",
            "runs": [{"text": words}]}


def project(tracks, duration, output, *, fonts=True, background=BG, frame=(1920, 1080)):
    p = {"frame": {"width": frame[0], "height": frame[1]}, "fps": 30, "background": background,
         "duration": duration, "output": output, "tracks": tracks}
    if fonts:
        p["fonts"] = {"oswald": [{"file": FONT}]}
        p["fontVendor"] = {FONT: {
            "licence": "OFL-1.1", "source": "google/fonts ofl/oswald, instanced wght=600",
            "sha256": "442420449b66e3f8a49025fbb229a8b4b1efa5f7be9458a7c3244498d34f8de9"}}
    return p


TITLES = {
    "miter": "Miter joins at stroke_miter_limit 1, 4, 10 — stroke 16, sharpest corner on the inset edge",
    "caps": "stroke_cap on a 45° open line — stroke 40; square reaches √2 × w/2 along an axis",
    "keyed-miter": "Keyed points, miter limit 10: the corner closes from 41° to 0° and back between the keys",
    "dashed-shapes": "stroke_dash [60, 20, 10, 20] on rect, rounded rect, ellipse — red dot = start",
    "seam": "The seam at points[0] (red dot), stroke_dash [50, 30], miter limit 2",
    "dotted": "Zero-length dashes: dots under a round cap, squares under a square cap",
    "ants": "Marching ants: stroke_dash_offset keyed linearly, [24, 16], both signs",
}

SHADOWED = {"miter-10", "keyed-miter", "dash-rrect", "ants-minus", "seam-circle"}


def clip():
    s = scene_elements()
    marks = markers(s)
    tracks = []

    def add(e):
        tracks.append({"name": e["id"], "layer": len(tracks), "elements": [e]})

    for name, start, end in SCENES:
        add(text(f"title-{name}", start, end, TITLES[name], x=60, y=60, size=40, color="#FFFFFF",
                 w=1800))
        for e, label, m in s[name]:
            add({"id": f"box-{e['id']}", "type": "rect", "start": start, "end": end,
                 "x": e["x"], "y": e["y"], "origin": "top-left", "width": e["width"],
                 "height": e["height"], "stroke": GUIDE, "stroke_width": 1})
            if m:
                add({"id": f"inset-{e['id']}", "type": "rect", "start": start, "end": end,
                     "x": e["x"] + m, "y": e["y"] + m, "origin": "top-left",
                     "width": max(e["width"] - 2 * m, 1), "height": max(e["height"] - 2 * m, 1),
                     "stroke": INSET, "stroke_width": 1})
            el = timed(e, start, end)
            if e["id"] in SHADOWED:
                el["effects"] = [SHADOW]
            add(el)
            add(text(f"label-{e['id']}", start, end, label, x=e["x"], y=e["y"] - 56, size=28,
                     w=max(e["width"], 520)))
            if e["id"] in marks:
                mx, my = marks[e["id"]]
                add({"id": f"mark-{e['id']}", "type": "ellipse", "start": start, "end": end,
                     "x": round(mx), "y": round(my), "origin": "center", "width": 14,
                     "height": 14, "fill": MARK})
    add(text("legend", 0, 22000, "grey = declared box   blue = inset box [m, w−m] × [m, h−m]",
             x=60, y=1000, size=26, color="#8A9099", w=1200))
    return project(tracks, 22000, "out/strokes.mp4")


def contain():
    """One project per scene: the stroked elements alone, on black, for check.py."""
    s = scene_elements()
    out = {}
    for name, start, end in SCENES:
        tracks = [{"name": e["id"], "layer": i, "elements": [timed(e, 0, end - start)]}
                  for i, (e, _, _) in enumerate(s[name])]
        out[name] = project(tracks, end - start, f"../out/contain-{name}.mp4", fonts=False,
                            background="#000000")
    return out, s


def errors():
    """One element per new `validate` error."""
    base = lambda id, **kw: {"id": id, "type": "path", "start": 0, "end": 1000, "x": 0,  # noqa: E731
                             "y": 0, "origin": "top-left", "width": 200, "height": 200,
                             "closed": False, "stroke": INK, "stroke_width": 10,
                             "points": [{"at": [20, 20]}, {"at": [180, 180]}], **kw}
    els = [
        # E-STROKE-NO-STROKE: no stroke at all; and a stroke_width of 0.
        {k: v for k, v in base("no-stroke", stroke_cap="round").items()
         if k not in ("stroke", "stroke_width")},
        base("zero-width", stroke_width=0, stroke_dash=[10, 5]),
        # E-STROKE-MITER-LIMIT, both ways.
        base("miter-no-limit", stroke_join="miter"),
        base("limit-on-round", stroke_join="round", stroke_miter_limit=4),
        # E-STROKE-CAP-UNDRAWN.
        {**base("closed-cap", closed=True, stroke_cap="butt"),
         "points": [{"at": [20, 20]}, {"at": [180, 20]}, {"at": [100, 180]}]},
        # E-DASH-SHAPE, both ways.
        base("odd-dash", stroke_dash=[10, 5, 3]),
        base("zero-dash", stroke_dash=[0, 0]),
        # E-DASH-ZERO-BUTT on a path and on a rect.
        base("zero-butt", stroke_dash=[0, 10]),
        {"id": "zero-butt-rect", "type": "rect", "start": 0, "end": 1000, "x": 300, "y": 0,
         "origin": "top-left", "width": 200, "height": 200, "stroke": INK, "stroke_width": 10,
         "stroke_dash": [0, 10]},
        # E-PATH-OUTSIDE-BOX naming k: miter limit 10, and a square cap.
        {**base("outside-miter", stroke_width=3, stroke_join="miter", stroke_miter_limit=10),
         "points": [{"at": [10, 20]}, {"at": [100, 190]}, {"at": [190, 20]}]},
        base("outside-square", stroke_width=20, stroke_cap="square",
             points=[{"at": [10, 100]}, {"at": [190, 100]}]),
        # The guessed code: an offset with no dash.
        base("offset-alone", stroke_dash_offset=5),
    ]
    tracks = [{"name": e["id"], "layer": i, "elements": [e]} for i, e in enumerate(els)]
    return project(tracks, 1000, "out/errors.mp4", fonts=False)


def schema_cases():
    """Field-set errors the schema reports, one project each."""
    rect = {"id": "r", "type": "rect", "start": 0, "end": 1000, "x": 0, "y": 0,
            "origin": "top-left", "width": 200, "height": 200, "stroke": INK, "stroke_width": 10}
    cases = {
        "join-on-rect": {**rect, "stroke_join": "miter"},
        "cap-on-ellipse": {**rect, "type": "ellipse", "stroke_cap": "round"},
        "limit-11": {**rect, "type": "path", "closed": False, "stroke_join": "miter",
                     "stroke_miter_limit": 11, "points": [{"at": [20, 20]}, {"at": [180, 180]}]},
        "dash-one-entry": {**rect, "stroke_dash": [10]},
        "dash-keyed": {**rect, "stroke_dash": [{"t": 0, "v": [10, 5]}]},
    }
    return {k: project([{"name": "t", "layer": 0, "elements": [e]}], 1000, "out/x.mp4",
                       fonts=False) for k, e in cases.items()}


def query():
    s = scene_elements()
    els = [timed(e, 0, 3000) for e, _, _ in s["dashed-shapes"] + s["ants"] + s["seam"]]
    els += [timed(s["miter"][2][0], 0, 3000), timed(s["caps"][2][0], 0, 3000)]
    tracks = [{"name": e["id"], "layer": i, "elements": [e]} for i, e in enumerate(els)]
    return project(tracks, 3000, "out/query.mp4", fonts=False)


def loop():
    """The ants held past their last key, so the frame at the last key can be compared with
    the frame at the first: a whole number of pattern lengths moves it back onto itself."""
    s = scene_elements()
    els = [timed(e, 0, 4000) for e, _, _ in s["ants"]]
    tracks = [{"name": e["id"], "layer": i, "elements": [e]} for i, e in enumerate(els)]
    return project(tracks, 4000, "out/loop.mp4", fonts=False, background="#000000")


def write(rel, p):
    path = HERE / rel
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(p, indent=1, ensure_ascii=False) + "\n")


if __name__ == "__main__":
    write("strokes.montagent.json", clip())
    projects, s = contain()
    for name, p in projects.items():
        write(f"contain/{name}.json", p)
    boxes = {name: [{"id": e["id"], "x": e["x"], "y": e["y"], "w": e["width"], "h": e["height"]}
                    for e, _, _ in s[name]] for name, _, _ in SCENES}
    write("contain/boxes.json", boxes)
    write("errors.json", errors())
    for k, p in schema_cases().items():
        write(f"schema/{k}.json", p)
    write("query.json", query())
    write("loop.json", loop())
    # For cost.sh: the same clip with every new field dropped (round join, butt cap, no dash).
    plain = clip()
    for t in plain["tracks"]:
        for e in t["elements"]:
            for k in ("stroke_join", "stroke_miter_limit", "stroke_cap", "stroke_dash",
                      "stroke_dash_offset"):
                e.pop(k, None)
    write("plain.json", plain)
    # For gap0.sh: a pattern with a zero gap, [10, 0], against no pattern at all, on each shape.
    s = scene_elements()
    picks = s["dashed-shapes"] + [s["seam"][0], s["miter"][2]]
    for name, dash in (("gap0-dashed", [10, 0]), ("gap0-plain", None)):
        els = []
        places = [(60, 20), (720, 20), (1380, 20), (60, 420), (720, 420)]  # no overlaps
        for (e, _, _), (px, py) in zip(picks, places):
            e = {k: v for k, v in e.items() if k != "stroke_dash"} | {"x": px, "y": py}
            if dash:
                e = {k: v for k, v in e.items() if k != "points"} | {"stroke_dash": dash,
                                                                      "points": e["points"]} \
                    if e["type"] == "path" else {**e, "stroke_dash": dash}
            els.append(timed(e, 0, 1000))
        tracks = [{"name": e["id"], "layer": i, "elements": [e]} for i, e in enumerate(els)]
        write(f"{name}.json", project(tracks, 1000, "out/x.mp4", fonts=False))
