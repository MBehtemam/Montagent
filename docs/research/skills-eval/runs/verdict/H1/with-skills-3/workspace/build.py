"""Generate ad.montagent.json from the beat sheet. Run: python3 build.py"""
import json

INK, PAPER, SIGNAL = "#101418", "#F5F0E6", "#FF5A36"
HAIRLINE = "#F5F0E640"
POP = [0.34, 1.56, 0.64, 1]
SETTLE = [0.16, 1, 0.3, 1]
GENTLE = [0.25, 1, 0.5, 1]
SWEEP = [0.65, 0, 0.35, 1]
SLIDE = [0.45, 0, 0.55, 1]

BEAT = 500
def b(n):  # nth beat from the 0 ms downbeat
    return n * BEAT

def kf(*pairs):
    out = []
    for i, p in enumerate(pairs):
        t, v = p[0], p[1]
        rec = {"t": t, "v": v}
        if i:
            rec["ease"] = p[2] if len(p) > 2 else "linear"
        out.append(rec)
    return out

def text(id, start, end, runs, font, size, color, x, y, width, height, **kw):
    e = {"id": id, "type": "text", "start": start, "end": end, "x": x, "y": y,
         "origin": kw.pop("origin", "center"), "width": width, "height": height,
         "font": font, "size": size, "color": color, "align": kw.pop("align", "center"),
         "caption": False, "runs": runs}
    e.update(kw)
    return e

tracks = []
def track(name, layer, *elements):
    tracks.append({"name": name, "layer": layer, "elements": list(elements)})

# ---- 0-4 s: Timelines. -> Text. -------------------------------------------
DROP_END = 1400  # y at 2 s: the word's ink is below the frame by its last frame
track("hook-old", 20,
      text("timelines", 0, b(4), [{"text": "Timelines."}], "bold", 200, PAPER, 960,
           kf((b(3), 500), (b(4), DROP_END, "ease-in")), 1100, 240))
track("strike", 21,
      {"id": "strike", "type": "rect", "start": b(1), "end": b(4), "x": 430,
       "y": kf((b(3), 510), (b(4), 510 + DROP_END - 500, "ease-in")),
       "origin": "center-left", "width": 1060, "height": 22, "fill": SIGNAL,
       "scale": kf((b(1), [0.0, 1.0]), (b(2), [1.0, 1.0], SWEEP))})
track("hook-new", 22,
      text("text", b(4), b(8), [{"text": "Text."}], "bold", 200, SIGNAL, 960,
           kf((b(4), 540), (b(4) + 400, 500, POP), (b(7), 500), (b(7) + 300, 470, "ease-in")),
           600, 240,
           scale=kf((b(4), [0.4, 0.4]), (b(4) + 400, [1.0, 1.0], POP)),
           opacity=kf((b(7), 1.0), (b(7) + 300, 0.0, "ease-in"))))
track("hook-sub", 23,
      text("sub", b(5), b(8), [{"text": "Your agent edits video the way it edits code."}],
           "regular", 60, PAPER, 960,
           kf((b(5), 720), (b(5) + 400, 680, SETTLE), (b(7), 680), (b(7) + 300, 650, "ease-in")),
           1400, 72,
           opacity=kf((b(5), 0.0), (b(5) + 300, 1.0, "ease-out"), (b(7), 1.0), (b(7) + 300, 0.0, "ease-in"))))

# ---- 4-8 s: the session, fast-forward, with the list -----------------------
REC_W, REC_H, REC_X, REC_Y = 1120, 630, 1264, 540
rec_x = kf((b(8), 1920 + REC_W // 2 + 40), (b(9), REC_X, GENTLE))
track("rec", 30,
      {"id": "rec", "type": "video", "start": b(8), "end": b(16), "source": "screen/session.mp4",
       "source_start": 0, "source_end": 55000, "speed": 13.75,
       "x": rec_x, "y": REC_Y, "origin": "center", "width": REC_W, "height": REC_H, "fit": "contain",
       "effects": [{"name": "mask", "shape": "rect", "radius": 24},
                   {"name": "shadow", "dx": 0, "dy": 28, "radius": 70, "color": "#000000", "opacity": 0.85}]})
track("rec-edge", 31,
      {"id": "rec-edge", "type": "rect", "start": b(8), "end": b(16), "x": rec_x, "y": REC_Y,
       "origin": "center", "width": REC_W, "height": REC_H, "stroke": HAIRLINE, "stroke_width": 2, "radius": 24})

ITEMS = ["Reads the format", "Edits one file", "Checks the frames", "Renders"]
for i, label in enumerate(ITEMS):
    t0 = b(9 + 2 * i)
    y = 390 + 100 * i
    track(f"item-{i + 1}-dot", 40,
          {"id": f"dot-{i + 1}", "type": "ellipse", "start": t0, "end": b(16), "x": 110, "y": y,
           "origin": "center", "width": 16, "height": 16, "fill": SIGNAL,
           "scale": kf((t0, [0.0, 0.0]), (t0 + 300, [1.0, 1.0], POP))})
    track(f"item-{i + 1}", 41,
          text(f"item-{i + 1}", t0, b(16),
               [{"text": f"{i + 1}  ", "font": "bold"}, {"text": label}], "regular", 48, PAPER,
               kf((t0, 106), (t0 + 300, 136, SETTLE)), y, 520, 58, origin="center-left", align="start",
               opacity=kf((t0, 0.0), (t0 + 200, 1.0, "ease-out"))))

# ---- 8-10 s: split screen --------------------------------------------------
SHOT_W, SHOT_H, SHOT_Y, CAP_Y = 832, 468, 470, 770
for n, (src, cap, side, t0) in enumerate(
        [("stills/session-01.png", "One line changed.", -1, b(16)),
         ("stills/session-02.png", "One video out.", 1, b(17))], start=1):
    final_x = 960 + side * 436
    start_x = -SHOT_W // 2 - 40 if side < 0 else 1920 + SHOT_W // 2 + 40
    x = kf((t0, start_x), (t0 + 500, final_x, GENTLE))
    track(f"shot-{n}", 50 + 3 * n,
          {"id": f"shot-{n}", "type": "image", "start": t0, "end": b(20), "source": src,
           "x": x, "y": SHOT_Y, "origin": "center", "width": SHOT_W, "height": SHOT_H, "fit": "contain",
           "effects": [{"name": "mask", "shape": "rect", "radius": 20},
                       {"name": "shadow", "dx": 0, "dy": 24, "radius": 50, "color": "#000000", "opacity": 0.7}]})
    track(f"shot-{n}-edge", 51 + 3 * n,
          {"id": f"shot-{n}-edge", "type": "rect", "start": t0, "end": b(20), "x": x, "y": SHOT_Y,
           "origin": "center", "width": SHOT_W, "height": SHOT_H, "stroke": HAIRLINE, "stroke_width": 2, "radius": 20})
    track(f"shot-{n}-cap", 52 + 3 * n,
          text(f"cap-{n}", t0, b(20), [{"text": cap}], "bold", 52, PAPER, x, CAP_Y, SHOT_W, 63))

# ---- 10-14 s: light ground, the mark, the lockup ---------------------------
END = 14000
track("ground", 5,
      {"id": "light", "type": "rect", "start": b(20), "end": END, "x": 0, "y": 0, "origin": "top-left",
       "width": 1920, "height": 1080, "fill": PAPER})

TILE, STEP, RADIUS = 140, 152, 29          # 460 / 40 / 96 in the brand's 1024 box, scaled
MARK_X, MARK_Y, LOCK_X = 960, 470, 363     # mark centre while it builds; centre in the lockup
SHIFT = LOCK_X - MARK_X
# order of arrival: top-left, bottom-left, bottom-right, then the accent circle top-right
TILES = [("tile-tl", -1, -1, "rect"), ("tile-bl", -1, 1, "rect"),
         ("tile-br", 1, 1, "rect"), ("tile-tr", 1, -1, "ellipse")]
for i, (tid, cx, cy, kind) in enumerate(TILES):
    t0 = b(20 + i)
    x0 = MARK_X + cx * STEP // 2
    y0 = MARK_Y + cy * STEP // 2
    e = {"id": tid, "type": kind, "start": t0, "end": END,
         "x": kf((b(24), x0), (b(25), x0 + SHIFT, SLIDE)),
         "y": kf((t0, y0 - 220), (t0 + 467, y0, POP)),
         "origin": "center", "width": TILE, "height": TILE,
         "fill": SIGNAL if kind == "ellipse" else INK,
         "opacity": kf((t0, 0.0), (t0 + 133, 1.0, "ease-out"))}
    if kind == "rect":
        e["radius"] = RADIUS
    track(f"mark-{i + 1}", 70 + i, e)

# wordmark.png ink is x 64-1951, y 40-378 of 1998x419; scaled so the lockup keeps the brand's proportions
K = 0.5737
WM_W, WM_H = 1146, 240
WM_X = round(LOCK_X + STEP // 2 + TILE // 2 + 111 - 64 * K)
WM_Y = round(MARK_Y - (STEP + TILE) / 2 + 0.167 * (STEP + TILE) - 40 * K)
track("wordmark", 65,
      {"id": "wordmark", "type": "image", "start": b(24), "end": END, "source": "brand/wordmark.png",
       "x": kf((b(24), WM_X + SHIFT), (b(25), WM_X, SLIDE)), "y": WM_Y, "origin": "top-left",
       "width": WM_W, "height": WM_H, "fit": "contain"})
# the name slides out from behind the mark: a ground-coloured occluder rides the mark's right edge
MARK_R = MARK_X + STEP // 2 + TILE // 2
track("wordmark-occluder", 67,
      {"id": "wm-occluder", "type": "rect", "start": b(24), "end": b(25),
       "x": kf((b(24), MARK_R), (b(25), MARK_R + SHIFT, SLIDE)), "y": MARK_Y,
       "origin": "center-right", "width": 1300, "height": 320, "fill": PAPER})
track("tagline", 66,
      text("tagline", b(25), END, [{"text": "The video editor your agent drives."}], "regular", 56, INK,
           960, kf((b(25), 720), (b(25) + 400, 700, SETTLE)), 1100, 68,
           opacity=kf((b(25), 0.0), (b(25) + 400, 1.0, "ease-out"))))

# ---- music -----------------------------------------------------------------
track("music", 0,
      {"id": "bed", "type": "audio", "start": 0, "end": END, "source": "music/bed-120bpm.wav",
       "source_start": 0, "source_end": END, "volume": kf((b(26), 0.8), (13966, 0.0, "linear"))})

p = json.load(open("base.json"))
p["tracks"] = tracks
json.dump(p, open("ad.montagent.json", "w"), indent=2, ensure_ascii=False)
