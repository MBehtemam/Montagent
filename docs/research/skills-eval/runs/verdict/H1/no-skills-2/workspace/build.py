# Generates the tracks of ad.json; header is kept as-is.
import json
INK, PAPER, SIGNAL = "#101418", "#F5F0E6", "#FF5A36"
MUTED = "#F5F0E68C"
OVER = [0.34, 1.56, 0.64, 1]   # back-out: small overshoot

def kf(*pts):
    out = []
    for i, p in enumerate(pts):
        r = {"t": p[0], "v": p[1]}
        if i: r["ease"] = p[2] if len(p) > 2 else "linear"
        out.append(r)
    return out

def text(id, s, e, x, y, runs, font, size, color=PAPER, origin="center", w=None, h=None, **kw):
    el = {"id": id, "type": "text", "start": s, "end": e, "x": x, "y": y, "origin": origin,
          "width": w, "height": h, "font": font, "size": size, "color": color, "runs": runs}
    el.update(kw); el["caption"] = False
    return el

tracks = []
def track(name, layer, els):
    # one track per element: siblings here overlap in time
    for i, el in enumerate(els):
        tracks.append({"name": el["id"], "layer": layer * 10 + i, "elements": [el]})

# ---------- music ----------
track("music", 0, [{"id": "music", "type": "audio", "start": 0, "end": 14000, "source": "music/bed-120bpm.wav",
                    "source_start": 0, "source_end": 14000,
                    "volume": kf((13000, 1.0), (13950, 0.0, "ease-in"))}])

# ---------- light ground from 10 s ----------
track("ground", 1, [{"id": "paper-ground", "type": "rect", "start": 10000, "end": 14000, "x": 0, "y": 0, "origin": "top-left",
                     "width": 1920, "height": 1080, "fill": PAPER}])

# ---------- 0-4 s: kinetic type ----------
drop = kf((1500, 540), (1950, 1420, "ease-in"))
track("type-a", 10, [
    text("timelines", 0, 2000, 960, drop, [{"text": "Timelines."}], "bold", 220, w=1160, h=264),
    text("text-word", 2000, 4000, 960,
         kf((3500, 470), (3800, 400, "ease-in")),
         [{"text": "Text."}], "bold", 220, w=600, h=264,
         scale=kf((2000, [0.55, 0.55]), (2350, [1.0, 1.0], OVER)),
         opacity=kf((3500, 1.0), (3800, 0.0, "ease-in"))),
])
bar_y = kf((1500, 548), (1950, 1428, "ease-in"))
track("strike", 11, [
    {"id": "strike-bar", "type": "rect", "start": 500, "end": 2000, "x": 389, "y": bar_y, "origin": "center-left",
     "width": 1142, "height": 24, "fill": SIGNAL, "radius": 4,
     "scale": kf((500, [0.0, 1.0]), (1000, [1.0, 1.0], "ease-in-out"))},
])
track("type-b", 12, [
    text("tagline-1", 2500, 4000, 960,
         kf((2500, 720), (2850, 660, "ease-out"), (3500, 660), (3800, 600, "ease-in")),
         [{"text": "Your agent edits video the way it edits "}, {"text": "code.", "color": SIGNAL}],
         "regular", 64, w=1400, h=78,
         opacity=kf((2500, 0.0), (2850, 1.0, "ease-out"), (3500, 1.0), (3800, 0.0, "ease-in"))),
])

# ---------- 4-8 s: screen recording + list ----------
VX, VY, VW, VH, R = 1270, 540, 1140, 642, 28
vx = kf((4000, VX + 1300), (4450, VX, [0.16, 1, 0.3, 1]))
track("screen", 20, [
    {"id": "session", "type": "video", "start": 4000, "end": 8000, "source": "screen/session.mp4",
     "source_start": 0, "source_end": 55000, "x": vx, "y": VY, "origin": "center", "width": VW, "height": VH,
     "fit": "literal", "speed": 13.75,
     "effects": [{"name": "mask", "shape": "rect", "radius": R},
                 {"name": "shadow", "dx": 0, "dy": 24, "radius": 48, "color": "#000000", "opacity": 0.75}]},
])
track("screen-edge", 21, [
    {"id": "session-edge", "type": "rect", "start": 4000, "end": 8000, "x": vx, "y": VY, "origin": "center",
     "width": VW, "height": VH, "stroke": "#F5F0E633", "stroke_width": 2, "radius": R},
])
items = ["Reads the format", "Edits one file", "Checks the frames", "Renders"]
dots, labels = [], []
for i, label in enumerate(items):
    t0 = 4500 + 1000 * i
    y = 390 + 100 * i
    op = kf((t0, 0.0), (t0 + 250, 1.0, "ease-out"))
    dots.append({"id": f"dot-{i+1}", "type": "ellipse", "start": t0, "end": 8000, "x": 96, "y": y, "origin": "center",
                 "width": 22, "height": 22, "fill": SIGNAL,
                 "scale": kf((t0, [0.0, 0.0]), (t0 + 300, [1.0, 1.0], OVER))})
    labels.append(text(f"item-{i+1}", t0, 8000, kf((t0, 170), (t0 + 300, 132, "ease-out")), y,
                       [{"text": f"{i+1}   ", "color": MUTED}, {"text": label}], "bold", 46,
                       origin="center-left", w=520, h=56, opacity=op))
track("list-dots", 22, dots)
track("list-text", 23, labels)

# ---------- 8-10 s: split screen ----------
SW, SH, SY, SR = 864, 486, 465, 24
LX, RX = 496, 1424
CAP_Y = SY + SH // 2 + 64
def still(id, src, t0, x_from, x_to):
    x = kf((t0, x_from), (t0 + 400, x_to, [0.16, 1, 0.3, 1]))
    img = {"id": id, "type": "image", "start": t0, "end": 10000, "source": src, "x": x, "y": SY, "origin": "center",
           "width": SW, "height": SH, "fit": "literal",
           "effects": [{"name": "mask", "shape": "rect", "radius": SR},
                       {"name": "shadow", "dx": 0, "dy": 20, "radius": 40, "color": "#000000", "opacity": 0.75}]}
    edge = {"id": id + "-edge", "type": "rect", "start": t0, "end": 10000, "x": x, "y": SY, "origin": "center",
            "width": SW, "height": SH, "stroke": "#F5F0E633", "stroke_width": 2, "radius": SR}
    return img, edge, x
l_img, l_edge, lx = still("shot-diff", "stills/session-01.png", 8000, -480, LX)
r_img, r_edge, rx = still("shot-render", "stills/session-02.png", 8500, 2400, RX)
track("stills", 30, [l_img, r_img])
track("stills-edge", 31, [l_edge, r_edge])
track("captions", 32, [
    text("cap-diff", 8000, 10000, lx, CAP_Y, [{"text": "One line changed."}], "bold", 48, w=600, h=58),
    text("cap-render", 8500, 10000, rx, CAP_Y, [{"text": "One video out."}], "bold", 48, w=600, h=58),
])

# ---------- 10-14 s: mark assembly + lockup ----------
TILE, GAP, TR = 115, 10, 24
MX, MY = 840, 360            # assembled mark, centred
SLIDE = -490                 # to the lockup position
def tile_x(x): return kf((12000, x), (12400, x + SLIDE, "ease-in-out"))
def tile_y(t0, y):
    return kf((t0, -130), (t0 + 260, y, "ease-in"), (t0 + 360, y - 26, "ease-out"), (t0 + 480, y, "ease-in"))
tiles = []
order = [("tile-bl", 10000, MX, MY + TILE + GAP, "rect"),
         ("tile-br", 10500, MX + TILE + GAP, MY + TILE + GAP, "rect"),
         ("tile-tl", 11000, MX, MY, "rect"),
         ("tile-tr", 11500, MX + TILE + GAP, MY, "ellipse")]
for i, (id, t0, x, y, kind) in enumerate(order):
    el = {"id": id, "type": kind, "start": t0, "end": 14000, "x": tile_x(x), "y": tile_y(t0, y), "origin": "top-left",
          "width": TILE, "height": TILE, "fill": SIGNAL if kind == "ellipse" else INK}
    if kind == "rect": el["radius"] = TR
    tracks_el = el
    tiles.append(el)
for i, el in enumerate(tiles):
    track(f"mark-{i+1}", 60 + i, [el])
WM_W, WM_H = 938, 197
track("wordmark", 50, [
    {"id": "wordmark", "type": "image", "start": 12000, "end": 14000, "source": "brand/wordmark.png",
     "x": kf((12000, 651 - SLIDE), (12400, 651, "ease-in-out")), "y": MY + TILE + GAP // 2, "origin": "center-left",
     "width": WM_W, "height": WM_H, "fit": "literal",
     "opacity": kf((12000, 0.0), (12350, 1.0, "ease-out"))},
])
track("brand-line", 51, [
    text("brand-line", 12500, 14000, 960, kf((12500, 700), (12900, 680, "ease-out")),
         [{"text": "The video editor your agent drives."}], "regular", 52, color=INK, origin="top-center", w=900, h=63,
         opacity=kf((12500, 0.0), (12900, 1.0, "ease-out"))),
])

# write
src = open("ad.json").read()
doc = json.loads(src)
doc["tracks"] = tracks
open("ad.json", "w").write(json.dumps(doc, ensure_ascii=False, indent=2) + "\n")
