"""PROTOTYPE #760 — throwaway, never to merge. Writes every project this prototype renders.

- trim.montagent.json -> out/trim.mp4: the clip for the owner (seven scenes, 23 s).
- contain/<scene>.json: each scene's stroked elements alone on black, no guides, labels or
  effects, for the ink-outside-the-box check (check.py).
- full-trimmed.json / full-plain.json: the full window under offsets, against the same
  elements with every trim field dropped (fullwin.sh).
- errors.json, schema/*.json: one element per `validate` text.
- query.json: the `query --at` captures.
- plain.json: the clip with every trim field dropped (cost.sh, side-by-side).
"""

import copy
import json
import math
import pathlib

HERE = pathlib.Path(__file__).parent
FONT = "../../fixtures/benchmark/spy-trailer/fonts/Oswald-SemiBold.ttf"
BG = "#1A1D22"
GUIDE = "#5A5F68"
INSET = "#2E5C8A"
INK = "#F2F2F2"
GHOST = "#3A404A"
MARK = "#FF3B30"
SHADOW = {"name": "shadow", "dx": 10, "dy": 10, "radius": 8, "color": "#000000", "opacity": 0.9}
OVERSHOOT = [0.34, 1.56, 0.64, 1]
TRIM_KEYS = ("trim_start", "trim_end", "trim_offset")

SCENES = [
    ("draw-on", 0, 3000),
    ("loaders", 3000, 7000),
    ("square-cap", 7000, 11000),
    ("crossing", 11000, 14000),
    ("overshoot", 14000, 17000),
    ("dashed", 17000, 20000),
    ("full", 20000, 23000),
]


def keys(*pairs):
    """[(t, v, ease)] -> a keyframe list; the first record carries no ease."""
    out = []
    for i, (t, v, *ease) in enumerate(pairs):
        r = {"t": t, "v": v}
        if i:
            r["ease"] = ease[0] if ease else "linear"
        out.append(r)
    return out


def pts(points, m):
    return [{"at": [x + m, y + m]} for x, y in points]


def path(id, *, x, y, w, h, closed, points, sw, color=INK, **extra):
    e = {"id": id, "type": "path", "x": x, "y": y, "origin": "top-left", "width": w,
         "height": h, "closed": closed, "stroke": color, "stroke_width": sw}
    e.update(extra)
    e["points"] = points  # last, after the trim fields (key order, guess 2)
    return e


def shape(id, kind, *, x, y, w, h, sw, color=INK, **extra):
    e = {"id": id, "type": kind, "x": x, "y": y, "origin": "top-left", "width": w, "height": h,
         "stroke": color, "stroke_width": sw}
    e.update(extra)
    return e


def untrim(e):
    """Drop every trim field. A closed path with no dash then cannot carry a cap
    (E-STROKE-CAP-UNDRAWN), and none draws there, so the cap goes too."""
    for k in TRIM_KEYS:
        e.pop(k, None)
    if e.get("type") == "path" and e.get("closed") and "stroke_dash" not in e:
        e.pop("stroke_cap", None)


def square_inset(w):
    m = (w + 1) // 2
    while 2 * m * m < w * w:
        m += 1
    return m


def scene_elements():
    """{scene: [(element, label, inset m or None, ghost?)]} — the stroked elements, no timing."""
    s = {}

    # 1. Draw-on under a round cap: trim_end 0 -> 1. Frame 0 must be empty (no dot).
    sw = 28
    m = (sw + 1) // 2
    wave = [{"at": [m, 300 + m], "out": [260, -300]},
            {"at": [380 + m, 150 + m], "in": [-120, 0], "out": [120, 0]},
            {"at": [760 + m, m], "in": [-260, 300]}]
    zig = pts([(0, 300), (200, 0), (400, 300), (600, 0), (760, 300)], m)
    s["draw-on"] = [
        (path("drawon-wave", x=80, y=330, w=760 + 2 * m, h=300 + 2 * m, closed=False,
              points=wave, sw=sw, stroke_cap="round",
              trim_end=keys((0, 0), (2500, 1))),
         "trim_end 0 to 1, linear, round cap", m),
        (path("drawon-zig", x=1000, y=330, w=760 + 2 * m, h=300 + 2 * m, closed=False,
              points=zig, sw=sw, stroke_cap="round",
              trim_end=keys((0, 0), (2500, 1, "ease-in-out"))),
         "trim_end 0 to 1, ease-in-out, round cap", m),
    ]

    # 2. Ring loaders: a fixed window [0, 0.25], trim_offset keyed 0 -> 3 turns, linear.
    sw = 24
    loader = dict(trim_start=0, trim_end=0.25, trim_offset=keys((0, 0), (3600, 3)))
    s["loaders"] = [
        (shape("loader-ellipse", "ellipse", x=140, y=360, w=420, h=420, sw=sw, **loader),
         "ellipse: start 3 o'clock", None),
        (shape("loader-rect", "rect", x=750, y=360, w=420, h=420, sw=sw, **loader),
         "rect: start = top-left corner", None),
        (shape("loader-rrect", "rect", x=1360, y=360, w=420, h=420, sw=sw, radius=90, **loader),
         "rect, radius 90: start = arc's end", None),
    ]

    # 3. A closed octagon under a square cap, stroke 30: inset = least m with 2m² ≥ w² = 22.
    #    Its 45° edges end on the inset box, so a square trim end on a vertex reaches
    #    √2 × 15 = 21.2 along the axis against an inset of 22.
    sw = 30
    m = square_inset(sw)
    octagon = [(150, 0), (450, 0), (600, 150), (600, 350), (450, 500), (150, 500), (0, 350),
               (0, 150)]
    diamond = [(250, 0), (500, 250), (250, 500), (0, 250)]
    s["square-cap"] = [
        (path("sq-octagon", x=120, y=320, w=600 + 2 * m, h=500 + 2 * m, closed=True,
              points=pts(octagon, m), sw=sw, stroke_cap="square", trim_start=0,
              trim_end=keys((0, 0.2), (1950, 0.45), (3900, 0.2)),
              trim_offset=keys((0, 0), (3900, 2))),
         f"closed octagon, square cap: window 0.2–0.45 orbiting 2 turns, inset {m}", m),
        (path("sq-diamond", x=1100, y=320, w=500 + 2 * m, h=500 + 2 * m, closed=True,
              points=pts(diamond, m), sw=sw, stroke_cap="square",
              trim_start=keys((0, 0), (2000, 0), (3600, 1)),
              trim_end=keys((0, 0), (1600, 1))),
         f"closed diamond, square cap: draw on, then off, inset {m}", m),
    ]

    # 4. Keyed start and end that cross (round cap): empty from ~1071 ms to ~2045 ms.
    sw = 28
    m = (sw + 1) // 2
    line = [{"at": [m, 200 + m], "out": [400, -200]}, {"at": [1500 + m, 200 + m],
                                                       "in": [-400, -200]}]
    s["crossing"] = [
        (path("crossing", x=180, y=420, w=1500 + 2 * m, h=200 + 2 * m, closed=False,
              points=line, sw=sw, stroke_cap="round",
              trim_start=keys((0, 0), (1450, 0.7), (2900, 0.2)),
              trim_end=keys((0, 1), (1450, 0.3), (2900, 0.9))),
         "trim_start 0 to 0.7 to 0.2, trim_end 1 to 0.3 to 0.9: nothing drawn from ~1.04 s to ~1.98 s",
         m),
    ]

    # 5. An overshooting bezier ease, clamped to [0, 1].
    sw = 28
    m = (sw + 1) // 2
    straight = pts([(0, 0), (1500, 0)], m)
    s["overshoot"] = [
        (path("over-end", x=180, y=380, w=1500 + 2 * m, h=2 * m, closed=False,
              points=straight, sw=sw, stroke_cap="round",
              trim_end=keys((0, 0), (2000, 1, OVERSHOOT))),
         "trim_end 0 to 1 under [0.34, 1.56, 0.64, 1]: passes 1, clamped", m),
        (path("over-start", x=180, y=640, w=1500 + 2 * m, h=2 * m, closed=False,
              points=straight, sw=sw, stroke_cap="round",
              trim_start=keys((0, 0.6), (2000, 0, OVERSHOOT))),
         "trim_start 0.6 to 0 under the same ease: passes 0, clamped", m),
    ]

    # 6. Dashes held still while the window grows (or shrinks). A dim untrimmed copy sits
    #    beneath each, so a dash that moved would show beside its ghost.
    sw = 20
    m = (sw + 1) // 2
    s["dashed"] = [
        (path("dash-drawon", x=180, y=380, w=1500 + 2 * m, h=2 * m, closed=False,
              points=pts([(0, 0), (1500, 0)], m), sw=sw, stroke_dash=[60, 30],
              trim_end=keys((0, 0), (2500, 1))),
         "[60, 30], butt: trim_end 0 to 1", m),
        (path("dot-drawoff", x=180, y=560, w=1500 + 2 * m, h=200 + 2 * m, closed=False,
              points=[{"at": [m, 200 + m], "out": [400, -200]},
                      {"at": [1500 + m, 200 + m], "in": [-400, -200]}],
              sw=sw, stroke_cap="round", stroke_dash=[0, 40],
              trim_start=keys((0, 0), (2500, 1))),
         "[0, 40], round: trim_start 0 to 1 (draw-off)", m),
    ]

    # 7. The full window under an offset: byte-identical to the untrimmed element.
    sw = 24
    m = square_inset(sw)
    full = dict(trim_start=0, trim_end=1, trim_offset=0.37)
    s["full"] = [
        (shape("full-ellipse", "ellipse", x=100, y=380, w=360, h=360, sw=sw,
               trim_start=0, trim_end=1, trim_offset=keys((0, 0), (3000, 2.5))),
         "ellipse, offset keyed 0 to 2.5", None),
        (shape("full-rect", "rect", x=540, y=380, w=360, h=360, sw=sw, **full),
         "rect, offset 0.37", None),
        (shape("full-dash-rrect", "rect", x=980, y=380, w=360, h=360, sw=sw, radius=60,
               stroke_dash=[50, 25], **full),
         "dashed rounded rect, offset 0.37", None),
        (path("full-path", x=1420, y=380, w=360 + 2 * m, h=360 + 2 * m, closed=True,
              points=pts([(0, 0), (360, 0), (180, 360)], m), sw=sw, stroke_cap="square",
              **full),
         "closed path, square cap, offset 0.37", m),
    ]
    return s


def markers(s):
    """Start points, in frame space."""
    out = {}
    for name in ("loaders", "full"):
        for e, _, _ in s[name]:
            x, y, w, h, half = e["x"], e["y"], e["width"], e["height"], e["stroke_width"] / 2
            if e["type"] == "ellipse":
                out[e["id"]] = (x + w - half, y + h / 2)
            elif e["type"] == "rect":
                out[e["id"]] = (x + e.get("radius", 0) if e.get("radius") else x + half, y + half)
            else:
                a = e["points"][0]["at"]
                out[e["id"]] = (x + a[0], y + a[1])
    for e, _, _ in s["square-cap"]:
        a = e["points"][0]["at"]
        out[e["id"]] = (e["x"] + a[0], e["y"] + a[1])
    return out


def timed(e, start, end):
    """Place an element at [start, end). A keyframe's `t` is timeline time that travels with
    the element (ADR-0012), so every keyframe list moves by `start` with it."""
    e = copy.deepcopy(e)
    for k, v in e.items():
        if isinstance(v, list) and v and all(isinstance(r, dict) and "t" in r for r in v):
            e[k] = [{**r, "t": r["t"] + start} for r in v]
    return {"id": e.pop("id"), "type": e.pop("type"), "start": start, "end": end, **e}


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
    "draw-on": "Draw-on: trim_end keyed 0 to 1 under a round cap — no dot on the first frame",
    "loaders": "Ring loaders: window [0, 0.25], trim_offset keyed 0 to 3 turns — red dot = start point",
    "square-cap": "A closed path trimmed under a square cap: the ends stay inside the box",
    "crossing": "Keyed trim_start and trim_end cross: nothing is drawn between the crossings",
    "overshoot": "An overshooting bezier ease on the trim fields, clamped to [0, 1]",
    "dashed": "Dashes drawing on: the pattern holds still, the window reveals it (dim = untrimmed)",
    "full": "The full window [0, 1] under an offset: drawn whole, byte-identical to untrimmed",
}

SHADOWED = {"drawon-wave", "loader-ellipse", "sq-octagon", "crossing", "dash-drawon",
            "full-rect"}


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
            if name == "dashed":
                ghost = {k: v for k, v in e.items() if k not in TRIM_KEYS}
                ghost = timed({**ghost, "id": f"ghost-{e['id']}", "stroke": GHOST}, start, end)
                ghost["points"] = ghost.pop("points")
                add(ghost)
            el = timed(e, start, end)
            if e["id"] in SHADOWED:
                el["effects"] = [SHADOW]
            add(el)
            add(text(f"label-{e['id']}", start, end, label, x=e["x"], y=e["y"] - 56, size=28,
                     w=max(e["width"], 620)))
            if e["id"] in marks:
                mx, my = marks[e["id"]]
                add({"id": f"mark-{e['id']}", "type": "ellipse", "start": start, "end": end,
                     "x": round(mx), "y": round(my), "origin": "center", "width": 12,
                     "height": 12, "fill": MARK})
    add(text("legend", 0, 23000, "grey = declared box   blue = inset box [m, w−m] × [m, h−m]",
             x=60, y=1000, size=26, color="#8A9099", w=1200))
    return project(tracks, 23000, "out/trim.mp4")


def contain():
    s = scene_elements()
    out = {}
    for name, start, end in SCENES:
        tracks = [{"name": e["id"], "layer": i, "elements": [timed(e, 0, end - start)]}
                  for i, (e, _, _) in enumerate(s[name])]
        out[name] = project(tracks, end - start, f"../out/contain-{name}.mp4", fonts=False,
                            background="#000000")
    return out, s


def full_pair():
    """The full-window scene's elements, plus a closed path with a keyed offset past one turn,
    and the same elements with every trim field dropped."""
    s = scene_elements()
    els = [timed(e, 0, 3000) for e, _, _ in s["full"]]
    extra = path("full-path-keyed", x=100, y=40, w=500, h=300, closed=True,
                 points=pts([(0, 0), (480, 0), (480, 280), (0, 280)], 10), sw=20,
                 stroke_dash=[30, 10], trim_start=0, trim_end=1,
                 trim_offset=keys((0, -1.25), (3000, 3.5)))
    els.append(timed(extra, 0, 3000))
    for e in els:
        e["effects"] = [SHADOW]
    plain = copy.deepcopy(els)
    for e in plain:
        untrim(e)
    mk = lambda es: project([{"name": e["id"], "layer": i, "elements": [e]}  # noqa: E731
                             for i, e in enumerate(es)], 3000, "out/x.mp4", fonts=False)
    return mk(els), mk(plain)


def seam_pair():
    """Static windows straddling the start point, [0.875, 0.125] (offset 0.875 turns) — and
    the same elements untrimmed, for check.py to compare the pixels around the start point:
    equal bytes there mean one stroke through it, with no cap and no seam."""
    sw = 24
    m = 12
    win = dict(trim_start=0, trim_end=0.5, trim_offset=0.75)
    els = [
        shape("seam-ellipse", "ellipse", x=60, y=100, w=420, h=420, sw=sw, **win),
        shape("seam-rect", "rect", x=540, y=100, w=420, h=420, sw=sw, **win),
        shape("seam-rrect", "rect", x=1020, y=100, w=420, h=420, sw=sw, radius=90, **win),
        path("seam-path-miter", x=1500, y=100, w=360, h=360, closed=True,
             points=pts([(0, 0), (312, 0), (312, 312), (0, 312)], 2 * m), sw=sw,
             stroke_join="miter", stroke_miter_limit=2, **win),
        path("seam-path-round", x=60, y=600, w=400, h=400, closed=True,
             points=pts([(0, 0), (376, 188), (0, 376)], m), sw=sw, **win),
        # A dashed crossing window: the two pieces are dashed separately (guess 9).
        # Outline 1584: [50, 30] ends the outline in a gap (1584 mod 80 = 64), [40, 20] in a
        # dash (1584 mod 60 = 24), where the untrimmed rect merges the dash across the corner.
        shape("seam-dash-rect", "rect", x=540, y=600, w=420, h=420, sw=sw, stroke_dash=[50, 30],
              **win),
        shape("seam-dash-on", "rect", x=1020, y=600, w=420, h=420, sw=sw, stroke_dash=[40, 20],
              **win),
    ]
    els = [timed(e, 0, 1000) for e in els]
    plain = copy.deepcopy(els)
    for e in plain:
        untrim(e)
    mk = lambda es: project([{"name": e["id"], "layer": i, "elements": [e]}  # noqa: E731
                             for i, e in enumerate(es)], 1000, "out/x.mp4", fonts=False,
                            background="#000000")
    starts = {}
    for e in els:
        x, y, w, h, half = e["x"], e["y"], e["width"], e["height"], e["stroke_width"] / 2
        if e["type"] == "ellipse":
            starts[e["id"]] = (x + w - half, y + h / 2)
        elif e["type"] == "rect":
            starts[e["id"]] = (x + e["radius"] if e.get("radius") else x + half, y + half)
        else:
            a = e["points"][0]["at"]
            starts[e["id"]] = (x + a[0], y + a[1])
    return mk(els), mk(plain), starts


def anchor_pair():
    """The dashed scene frozen half way: trim_end 0.5 and trim_start 0.5, against untrimmed."""
    s = scene_elements()
    a = copy.deepcopy(s["dashed"][0][0])
    a["trim_end"] = 0.5
    b = copy.deepcopy(s["dashed"][1][0])
    b["trim_start"] = 0.5
    els = [timed(a, 0, 1000), timed(b, 0, 1000)]
    plain = copy.deepcopy(els)
    for e in plain:
        untrim(e)
    mk = lambda es: project([{"name": e["id"], "layer": i, "elements": [e]}  # noqa: E731
                             for i, e in enumerate(es)], 1000, "out/x.mp4", fonts=False,
                            background="#000000")
    return mk(els), mk(plain)


def errors():
    def base(id, **kw):
        points = kw.pop("points", [{"at": [20, 20]}, {"at": [180, 180]}])
        return {"id": id, "type": "path", "start": 0, "end": 1000, "x": 0, "y": 0,
                "origin": "top-left", "width": 200, "height": 200, "closed": False,
                "stroke": INK, "stroke_width": 10, **kw, "points": points}
    tri = [{"at": [20, 20]}, {"at": [180, 20]}, {"at": [100, 180]}]
    rect = lambda id, **kw: {"id": id, "type": "rect", "start": 0, "end": 1000, "x": 300,  # noqa: E731
                             "y": 0, "origin": "top-left", "width": 200, "height": 200,
                             "stroke": INK, "stroke_width": 10, **kw}
    els = [
        # E-TRIM-EMPTY: `"trim_start": 1` alone; trim_end 0 alone; both written, crossed.
        base("start-one-alone", trim_start=1),
        {**rect("end-zero-alone", trim_end=0), "type": "ellipse"},
        rect("start-past-end", trim_start=0.6, trim_end=0.4),
        # E-TRIM-OFFSET, both cases, and both at once.
        base("offset-open", trim_start=0, trim_end=0.5, trim_offset=0.25),
        rect("offset-no-window", trim_offset=0.25),
        base("offset-open-no-window", trim_offset=0.5),
        # E-STROKE-NO-STROKE over the trim fields: no stroke; and a stroke_width of 0.
        {k: v for k, v in rect("trim-no-stroke", fill="#336699", trim_start=0.1, trim_end=0.9,
                               trim_offset=0.5).items() if k not in ("stroke", "stroke_width")},
        base("trim-zero-width", stroke_width=0, trim_end=0.5),
        # E-STROKE-CAP-UNDRAWN: still fires on a closed path with no dash and no trim...
        base("closed-cap-untrimmed", closed=True, stroke_cap="round", points=tri),
        # ...and no longer on one that carries trim_end (expected: no finding).
        base("closed-cap-trimmed", closed=True, stroke_cap="round", trim_end=0.5, points=tri),
        # E-PATH-OUTSIDE-BOX: a vertex at inset 10 = w/2, on a closed path whose square cap
        # only draws because it carries a trim (√2 inset 15).
        base("outside-square-trim", closed=True, stroke_width=20, stroke_cap="square",
             trim_end=0.5, points=[{"at": [10, 100]}, {"at": [190, 100]}, {"at": [100, 190]}]),
        # A keyed crossing is not an error (expected: no finding).
        base("keyed-crossing-ok", stroke_cap="round",
             trim_start=keys((0, 0), (900, 1)), trim_end=keys((0, 1), (900, 0))),
    ]
    tracks = [{"name": e["id"], "layer": i, "elements": [e]} for i, e in enumerate(els)]
    return project(tracks, 1000, "out/errors.mp4", fonts=False)


def schema_cases():
    r = {"id": "r", "type": "rect", "start": 0, "end": 1000, "x": 0, "y": 0,
         "origin": "top-left", "width": 200, "height": 200, "stroke": INK, "stroke_width": 10}
    cases = {
        "start-1.2": {**r, "trim_start": 1.2},
        "end-key-negative": {**r, "trim_end": keys((0, 0), (1000, -0.1))},
        "trim-on-text": {"id": "r", "type": "text", "start": 0, "end": 1000, "x": 0, "y": 0,
                         "origin": "top-left", "width": 400, "height": 60, "font": "oswald",
                         "size": 40, "color": INK, "align": "start", "trim_end": 0.5,
                         "runs": [{"text": "trim me"}]},
    }
    out = {}
    for k, e in cases.items():
        out[k] = project([{"name": "t", "layer": 0, "elements": [e]}], 1000, "out/x.mp4",
                         fonts=(e["type"] == "text"))
    return out


def query():
    s = scene_elements()
    els = [timed(s["crossing"][0][0], 0, 3000),            # drawn: empty at 1500
           timed(s["full"][1][0], 0, 3000),                # drawn: full under offset 0.37
           timed(s["full"][0][0], 0, 3000),                # drawn: full, keyed offset
           timed(s["loaders"][0][0], 0, 3600),             # a loader mid-turn
           timed(s["overshoot"][0][0], 0, 3000),           # raw trim_end past 1
           timed(s["overshoot"][1][0], 0, 3000)]           # raw trim_start below 0
    els.append(timed(shape("cross-past-turn", "rect", x=0, y=0, w=300, h=200, sw=10,
                           trim_start=0.3, trim_end=0.9, trim_offset=1.5), 0, 3000))
    els.append(timed(shape("cross-keyed-past-turn", "ellipse", x=400, y=0, w=300, h=200, sw=10,
                           trim_start=0.1, trim_end=0.6,
                           trim_offset=keys((0, 0), (3000, 3))), 0, 3000))
    tracks = [{"name": e["id"], "layer": i, "elements": [e]} for i, e in enumerate(els)]
    return project(tracks, 3000, "out/query.mp4", fonts=False)


def write(rel, p):
    path = HERE / rel
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(p, indent=1, ensure_ascii=False) + "\n")


def strip(p):
    p = copy.deepcopy(p)
    for t in p["tracks"]:
        for e in t["elements"]:
            untrim(e)
    return p


if __name__ == "__main__":
    write("trim.montagent.json", clip())
    write("plain.json", strip(clip()))
    projects, s = contain()
    for name, p in projects.items():
        write(f"contain/{name}.json", p)
    boxes = {name: [{"id": e["id"], "x": e["x"], "y": e["y"], "w": e["width"], "h": e["height"]}
                    for e, _, _ in s[name]] for name, _, _ in SCENES}
    write("contain/boxes.json", boxes)
    trimmed, plain = full_pair()
    write("full-trimmed.json", trimmed)
    write("full-plain.json", plain)
    trimmed, plain, starts = seam_pair()
    write("probe/seam-trimmed.json", trimmed)
    write("probe/seam-plain.json", plain)
    write("probe/seam-starts.json", starts)
    trimmed, plain = anchor_pair()
    write("probe/anchor-trimmed.json", trimmed)
    write("probe/anchor-plain.json", plain)
    write("errors.json", errors())
    for k, p in schema_cases().items():
        write(f"schema/{k}.json", p)
    write("query.json", query())
