#!/usr/bin/env python3
"""Generate ad.montagent.json from the beat sheet. Re-run after any retime."""
import json

INK, PAPER, SIGNAL = "#101418", "#F5F0E6", "#FF5A36"
SETTLE = [0.16, 1, 0.3, 1]
POP = [0.34, 1.56, 0.64, 1]
MOVE = [0.65, 0, 0.35, 1]


def beat(n):
    return 500 * n  # 120 BPM, downbeat at 0; every beat is a whole frame at 30 fps


def kf(*pairs):
    out = []
    for i, p in enumerate(pairs):
        t, v = p[0], p[1]
        rec = {"t": t, "v": v}
        if i:
            rec["ease"] = p[2]
        out.append(rec)
    return out


def text(id, start, end, x, y, s, font, size, color, height, width, origin="center", align="center", **kw):
    e = {"id": id, "type": "text", "start": start, "end": end, "x": x, "y": y, "origin": origin,
         "width": width, "height": height, "font": font, "size": size, "color": color,
         "align": align, "runs": [{"text": s}]}
    e.update(kw)
    e["caption"] = False
    return e


def track(name, layer, *els):
    return {"name": name, "layer": layer, "elements": list(els)}


def framed(prefix, start, end, x, y, w, h, radius, media, xs):
    """A rounded frame with a soft shadow: a hairline ring rect behind the media."""
    ring = {"id": f"{prefix}-ring", "type": "rect", "start": start, "end": end, "x": xs, "y": y,
            "origin": "center", "width": w + 4, "height": h + 4, "fill": "#F5F0E633",
            "radius": radius + 2,
            "effects": [{"name": "shadow", "dx": 0, "dy": 24, "radius": 56, "color": "#000000", "opacity": 0.55}]}
    media.update({"x": xs, "y": y, "origin": "center", "width": w, "height": h})
    media["effects"] = [{"name": "mask", "shape": "rect", "radius": radius}]
    return ring, media


tracks = []

# ---------- Section 1: Timelines. -> Text. (0-4 s, dark) ----------
WY = 540
DROP = 760
tracks.append(track("word-old", 20,
    text("timelines", 0, beat(4), 960,
         kf((beat(3), WY), (1966, WY + DROP, "ease-in")),
         "Timelines.", "bold", 180, PAPER, 216, 960)))
# strike bar sits mid x-height; word's advance is 918 px centred on 960
tracks.append(track("strike", 21,
    {"id": "strike", "type": "rect", "start": beat(1), "end": beat(4), "x": 488,
     "y": kf((beat(3), 554), (1966, 554 + DROP, "ease-in")), "origin": "center-left",
     "width": 944, "height": 18, "fill": SIGNAL, "radius": 4,
     "scale": kf((beat(1), [0.0, 1.0]), (beat(2), [1.0, 1.0], SETTLE))}))
tracks.append(track("word-new", 20,
    text("text", beat(4), beat(8), 960,
         kf((beat(4), WY + 40), (beat(4) + 400, WY, POP), (beat(7), WY, "linear"), (beat(7) + 300, WY - 30, "ease-in")),
         "Text.", "bold", 180, PAPER, 216, 500,
         scale=kf((beat(4), [0.5, 0.5]), (beat(4) + 400, [1.0, 1.0], POP)),
         opacity=kf((beat(7), 1.0), (beat(7) + 300, 0.0, "ease-in")))))
tracks.append(track("subline", 22,
    text("subline", beat(5), beat(8), 960,
         kf((beat(5), 740), (beat(5) + 400, 700, SETTLE), (beat(7), 700, "linear"), (beat(7) + 300, 670, "ease-in")),
         "Your agent edits video the way it edits code.", "regular", 52, PAPER, 63, 1160,
         opacity=kf((beat(5), 0.0), (beat(5) + 400, 1.0, SETTLE), (beat(7), 1.0, "linear"), (beat(7) + 300, 0.0, "ease-in")))))

# ---------- Section 2: screen recording + list (4-8 s, dark) ----------
RW, RH = 1120, 630
RX = 1824 - RW // 2
rx = kf((beat(8), 1920 + RW // 2 + 80), (beat(8) + 600, RX, SETTLE))
rec = {"id": "rec", "type": "video", "start": beat(8), "end": beat(16), "source": "screen/session.mp4",
       "source_start": 0, "source_end": 55000, "fit": "contain", "speed": 13.75}
ring, rec = framed("rec", beat(8), beat(16), RX, 540, RW, RH, 24, rec, rx)
tracks.append(track("rec-ring", 30, ring))
tracks.append(track("rec", 31, rec))

items = ["Reads the format", "Edits one file", "Checks the frames", "Renders"]
for i, label in enumerate(items):
    t0 = beat(9 + 2 * i)
    y = 396 + 96 * i
    op = kf((t0, 0.0), (t0 + 300, 1.0, SETTLE))
    dx = lambda x: kf((t0, x - 40), (t0 + 300, x, SETTLE))
    tracks.append(track(f"item-{i+1}", 32,
        {"id": f"dot-{i+1}", "type": "ellipse", "start": t0, "end": beat(16), "x": dx(116), "y": y,
         "origin": "center", "width": 16, "height": 16, "fill": SIGNAL, "opacity": op}))
    tracks.append(track(f"num-{i+1}", 32,
        text(f"num-{i+1}", t0, beat(16), dx(144), y, str(i + 1), "bold", 46, "#F5F0E68C", 56, 40,
             origin="center-left", align="start", opacity=op)))
    tracks.append(track(f"label-{i+1}", 32,
        text(f"label-{i+1}", t0, beat(16), dx(190), y, label, "bold", 46, PAPER, 56, 440,
             origin="center-left", align="start", opacity=op)))

# ---------- Section 3: split screen (8-10 s, dark) ----------
SW, SH = 816, 459
SY, CY = 466, 770
for n, (src, cap, xf, t0, layer) in enumerate([
        ("stills/session-01.png", "One line changed.", (-460, 520), beat(16), 40),
        ("stills/session-02.png", "One video out.", (2380, 1400), beat(17), 42)], start=1):
    xs = kf((t0, xf[0]), (t0 + 500, xf[1], SETTLE))
    img = {"id": f"shot-{n}", "type": "image", "start": t0, "end": beat(20), "source": src, "fit": "contain"}
    ring, img = framed(f"shot-{n}", t0, beat(20), None, SY, SW, SH, 20, img, xs)
    tracks.append(track(f"shot-{n}-ring", layer, ring))
    tracks.append(track(f"shot-{n}", layer + 1, img))
    tracks.append(track(f"cap-{n}", layer + 1,
        text(f"cap-{n}", t0, beat(20), kf((t0, xf[0]), (t0 + 500, xf[1], SETTLE)), CY, cap,
             "bold", 48, PAPER, 58, 600)))

# ---------- Section 4: brand (10-14 s, light) ----------
END = 14000
tracks.append(track("ground", 5,
    {"id": "light", "type": "rect", "start": beat(20), "end": END, "x": 0, "y": 0, "origin": "top-left",
     "width": 1920, "height": 1080, "fill": PAPER}))

# mark at 0.3 of its 1024 box: tiles 138, gap 12, radius 29; lockup centred on 960
T, G, R = 138, 12, 29
LEFT, TOP = 227, 336            # mark's ink box in the final lockup
SHIFT = 960 - (LEFT + T + G // 2)  # mark alone sits centred on 960
tiles = [  # (id, col, row, beat, shape)
    ("tile-tl", 0, 0, 20, "rect"),
    ("tile-bl", 0, 1, 21, "rect"),
    ("tile-br", 1, 1, 22, "rect"),
    ("tile-tr", 1, 0, 23, "ellipse"),
]
for i, (tid, c, r, b, shape) in enumerate(tiles):
    t0 = beat(b)
    cx = LEFT + c * (T + G) + T // 2
    cy = TOP + r * (T + G) + T // 2
    e = {"id": tid, "type": shape, "start": t0, "end": END,
         "x": kf((beat(24), cx + SHIFT), (beat(25), cx, MOVE)),
         "y": kf((t0, cy - 160), (t0 + 400, cy, POP)), "origin": "center",
         "width": T, "height": T, "fill": SIGNAL if shape == "ellipse" else INK}
    if shape == "rect":
        e["radius"] = R
    e["opacity"] = kf((t0, 0.0), (t0 + 133, 1.0, "ease-out"))
    tracks.append(track(tid, 60 + i, e))  # each arriving tile lands in front

# wordmark rides with the mark as it slides; lockup.png spacing scaled to the mark (k = 288/509)
WM_W, WM_H = 1130, 237
WM_X, WM_Y = LEFT + 361, TOP + 25
tracks.append(track("wordmark", 50,
    {"id": "wordmark", "type": "image", "start": beat(24), "end": END, "source": "brand/wordmark.png",
     "x": kf((beat(24), WM_X + SHIFT), (beat(25), WM_X, MOVE)), "y": WM_Y, "origin": "top-left",
     "width": WM_W, "height": WM_H, "fit": "contain",
     "opacity": kf((beat(24), 0.0), (beat(25), 1.0, MOVE))}))
tracks.append(track("tagline", 50,
    text("tagline", beat(25), END, 960, kf((beat(25), 750), (beat(25) + 400, 730, SETTLE)),
         "The video editor your agent drives.", "regular", 52, INK, 63, 960,
         opacity=kf((beat(25), 0.0), (beat(25) + 400, 1.0, SETTLE)))))

# ---------- Music ----------
tracks.append(track("music", 0,
    {"id": "bed", "type": "audio", "start": 0, "end": END, "source": "music/bed-120bpm.wav",
     "source_start": 0, "source_end": END,
     "volume": kf((13000, 0.85), (13966, 0.0, "linear"))}))

p = json.load(open("ad.montagent.json"))
p["tracks"] = tracks
json.dump(p, open("ad.montagent.json", "w"), ensure_ascii=False)
