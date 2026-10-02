import json
INK, PAPER, SIGNAL = "#101418", "#F5F0E6", "#FF5A36"
P = json.load(open("ad.montagent.json"))

def kf(*pts):
    out = []
    for i, p in enumerate(pts):
        r = {"t": p[0], "v": p[1]}
        if i: r["ease"] = p[2] if len(p) > 2 else "ease-out"
        out.append(r)
    return out

def text(id, start, end, x, y, runs, font="bold", size=52, color=PAPER, w=972, h=None, origin="top-left", align=None, rise=30, fade=300, lh=1.2):
    import math
    e = {"id": id, "type": "text", "start": start, "end": end,
         "x": x, "y": kf((start, y + rise), (start + fade, y)) if rise else y,
         "origin": origin, "width": w, "height": h or math.ceil(size * lh) + 2,
         "font": font, "size": size, "line_height": lh, "color": color}
    if align: e["align"] = align
    e["runs"] = runs
    e["opacity"] = kf((start, 0.0), (start + fade, 1.0))
    e["caption"] = False
    return e

CARD = [60, 260, 960, 720]
tracks = []
def track(name, layer, *els): tracks.append({"name": name, "layer": layer, "elements": list(els)})

# --- Hook 0-3 s: three lines landing on beats 0.0 / 0.5 / 1.0
track("hook-1", 10, text("hook-1", 0, 3000, 90, 300, [{"text": "Your AI agent"}], size=124, lh=1.1, w=900, rise=40, fade=350))
track("hook-2", 11, text("hook-2", 500, 3000, 90, 440, [{"text": "edits "}, {"text": "video", "color": SIGNAL}], size=124, lh=1.1, w=900, rise=40, fade=350))
track("hook-3", 12, text("hook-3", 1000, 3000, 90, 580, [{"text": "like code."}], size=124, lh=1.1, w=900, rise=40, fade=350))

# --- Proof 3-7.5 s, result 7.5-10 s
track("kicker", 10, text("kicker", 3000, 9750, 60, 108, [{"text": "A REAL CLAUDE CODE SESSION"}], font="regular", size=28, color="#F5F0E6A6", w=600, rise=0, fade=250))
track("label", 11,
      text("label-ask", 3000, 5000, 60, 156, [{"text": "You ask for a subtitle."}], rise=20, fade=250),
      text("label-edit", 5000, 7500, 60, 156, [{"text": "The agent edits the file."}], rise=20, fade=250),
      text("label-render", 7500, 9750, 60, 156, [{"text": "Montagent renders the result."}], rise=20, fade=250))
S = 0.85
VW, VH = 1632, 918
track("screen", 2,
      {"id": "shot-ask", "type": "video", "start": 3000, "end": 5000, "source": "screen/session.mp4", "source_start": 6000, "source_end": 9800,
       "x": kf((3000, 60), (4150, 60), (4950, -650, "linear")), "y": 380, "origin": "top-left", "width": 1920, "height": 1080, "fit": "literal", "clip": [60, 380, 960, 480],
       "opacity": kf((3000, 0.0), (3250, 1.0)), "speed": 1.9},
      {"id": "shot-edit", "type": "video", "start": 5000, "end": 7500, "source": "screen/session.mp4", "source_start": 33600, "source_end": 36100,
       "x": kf((5000, -8), (5250, -8), (7450, -416, "ease-in-out")), "y": 188, "origin": "top-left", "width": VW, "height": VH, "fit": "literal", "clip": CARD},
      {"id": "result", "type": "image", "start": 7500, "end": 10000, "source": "stills/session-02.png", "x": 540, "y": 620, "origin": "center",
       "width": 960, "height": 540, "fit": "literal", "scale": kf((7500, [0.86, 0.86]), (7850, [0.94, 0.94])), "opacity": kf((7500, 0.0), (7700, 1.0))})
track("result-path", 12, text("result-path", 7750, 9750, 540, 906, [{"text": "out/hello-text.mp4"}], font="regular", size=28, color="#F5F0E6A6", w=250, origin="top-center", rise=0, fade=250))
track("screen-edge", 3,
      {"id": "screen-edge-ask", "type": "rect", "start": 3000, "end": 5000, "x": 60, "y": 380, "origin": "top-left", "width": 960, "height": 480,
       "stroke": "#F5F0E633", "stroke_width": 2, "opacity": kf((3000, 0.0), (3250, 1.0))},
      {"id": "screen-edge-edit", "type": "rect", "start": 5000, "end": 7500, "x": 60, "y": 260, "origin": "top-left", "width": 960, "height": 720,
       "stroke": "#F5F0E633", "stroke_width": 2},
      {"id": "result-edge", "type": "rect", "start": 7500, "end": 10000, "x": 540, "y": 620, "origin": "center", "width": 968, "height": 548,
       "stroke": SIGNAL, "stroke_width": 4, "scale": kf((7500, [0.86, 0.86]), (7850, [0.94, 0.94])), "opacity": kf((7500, 0.0), (7700, 1.0))})

# --- Paper ground wipes up, landing on the 10.0 s downbeat
track("paper", 20, {"id": "paper", "type": "rect", "start": 9750, "end": 15000, "x": 0, "y": kf((9750, 1080), (10000, 0, "ease-in-out")),
                    "origin": "top-left", "width": 1080, "height": 1080, "fill": PAPER})

# --- Claim 10-13 s, brand 13-15 s
track("line-1", 30,
      text("claim-1", 10000, 13000, 540, 420, [{"text": "Your video is a "}, {"text": "file", "color": SIGNAL}, {"text": "."}], size=84, color=INK, w=780, origin="top-center", align="center", rise=30, fade=300),
      {"id": "lockup", "type": "image", "start": 13000, "end": 15000, "source": "brand/lockup.png", "x": 540, "y": 500, "origin": "center",
       "width": 780, "height": 180, "fit": "literal", "scale": kf((13000, [0.94, 0.94]), (13400, [1.0, 1.0])), "opacity": kf((13000, 0.0), (13300, 1.0))})
track("line-2", 31,
      text("claim-2", 10500, 13000, 540, 540, [{"text": "Your agent can edit it."}], size=84, color=INK, w=900, origin="top-center", align="center", rise=30, fade=300),
      text("tagline", 13500, 15000, 540, 650, [{"text": "The video editor AI agents drive."}], font="regular", size=40, color=INK, w=620, origin="top-center", align="center", rise=16, fade=300))

# --- Music under everything, fading over the held final chord
track("music", 0, {"id": "music", "type": "audio", "start": 0, "end": 15000, "source": "music/bed-120bpm.wav", "source_start": 0, "source_end": 15000,
                   "volume": kf((0, 1.0), (14000, 1.0, "linear"), (14950, 0.0, "linear"))})

P["tracks"] = tracks
json.dump(P, open("ad.montagent.json", "w"), ensure_ascii=False)
