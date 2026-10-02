"""Writes the tracks of ad.montagent.json. Run `montagent fmt` afterwards."""
import json

P = "ad.montagent.json"
INK, PAPER, SIGNAL = "#101418", "#F5F0E6", "#FF5A36"
d = json.load(open(P))


def ramp(t0, t1, v0, v1, ease="ease-out"):
    return [{"t": t0, "v": v0}, {"t": t1, "v": v1, "ease": ease}]


def text(id, start, end, x, y, w, h, size, runs, font="bold", color=PAPER, **kw):
    e = {"id": id, "type": "text", "start": start, "end": end, "x": x, "y": y,
         "origin": "top-left", "width": w, "height": h, "font": font, "size": size,
         "line_height": 1.1 if font == "bold" else 1.2, "color": color, "align": "start",
         "runs": runs}
    e.update(kw)
    e["caption"] = False
    return e


tracks = []

# ---- grounds: Paper behind proof and result (2.5 s - 9 s); Ink is the project background
tracks.append({"name": "ground", "layer": 0, "elements": [
    {"id": "paper-ground", "type": "rect", "start": 2500, "end": 9000, "x": 0, "y": 0, "origin": "top-left",
     "width": 1080, "height": 1080, "fill": PAPER},
]})

# ---- hook, 0 - 2.5 s: three lines, one per beat
HX, HY, HL = 96, 328, 141
hook = [("hook-1", 0, [{"text": "Your AI agent"}], 860),
        ("hook-2", 500, [{"text": "edits "}, {"text": "video", "color": SIGNAL}], 720),
        ("hook-3", 1000, [{"text": "like code."}], 640)]
for i, (id, t, runs, w) in enumerate(hook):
    y = HY + i * HL
    tracks.append({"name": id, "layer": 10, "elements": [
        text(id, t, 2500, HX, ramp(t, t + 350, y + 48, y), w, HL, 128, runs,
             opacity=ramp(t, t + 200, 0.0 if t else 0.35, 1.0))]})

# ---- proof / result labels (Ink on Paper), one per section, each entering on its cut
LX, LY = 72, 134
labels = [("label-ask", 2500, 4500, "You ask.", 300),
          ("label-edit", 4500, 7000, "It edits the file.", 500),
          ("label-render", 7000, 9000, "It renders the video.", 660)]
tracks.append({"name": "labels", "layer": 20, "elements": [
    text(id, s, e, ramp(s, s + 300, LX - 32, LX), LY, w, 71, 64, [{"text": t}], color=INK,
         opacity=ramp(s, s + 200, 0.0, 1.0))
    for id, s, e, t, w in labels]})

# ---- proof: the terminal, in one card window 936 x 700 at (72, 245)
CARD = [72, 245, 936, 700]
# A: the prompt being typed (source 6.2-10.2 s at 2x), 1.15x, panning to follow the cursor
ask = {"id": "term-ask", "type": "video", "start": 2500, "end": 4500,
       "source": "screen/session.mp4", "source_start": 6200, "source_end": 10200,
       "x": [{"t": 2500, "v": 49}, {"t": 3300, "v": 49, "ease": "linear"}, {"t": 4400, "v": -880, "ease": "ease-in-out"}],
       "y": 233, "origin": "top-left", "width": 2208, "height": 1242, "fit": "literal",
       "clip": CARD, "speed": 2, "volume": 0}
# B: the diff of the project file (source 33.68-36.18 s), 1:1, showing src x 40-976, y 90-790
edit = {"id": "term-edit", "type": "video", "start": 4500, "end": 7000,
        "source": "screen/session.mp4", "source_start": 33680, "source_end": 36180,
        "x": 32, "y": 155, "origin": "top-left", "width": 1920, "height": 1080, "fit": "literal",
        "clip": CARD, "volume": 0}
tracks.append({"name": "terminal", "layer": 5, "elements": [ask, edit]})

# the one accent shape in the proof: an underline under "Video as a document" in the diff
tracks.append({"name": "accent", "layer": 6, "elements": [
    {"id": "diff-underline", "type": "rect", "start": 5500, "end": 7000, "x": 338, "y": 764,
     "origin": "center-left", "width": 292, "height": 6, "fill": SIGNAL,
     "scale": [{"t": 5500, "v": [0.0, 1.0]}, {"t": 5800, "v": [1.0, 1.0], "ease": "ease-out"}]},
]})

# ---- result: the frame the session rendered, slow push-in inside a 936 x 526 window
tracks.append({"name": "result", "layer": 5, "elements": [
    {"id": "rendered", "type": "image", "start": 7000, "end": 9000, "source": "stills/session-02.png",
     "x": 540, "y": 595, "origin": "center", "width": 960, "height": 540, "fit": "literal",
     "clip": [72, 332, 936, 526],
     "scale": [{"t": 7000, "v": [0.94, 0.94]}, {"t": 7300, "v": [1.0, 1.0], "ease": "ease-out"},
               {"t": 8966, "v": [1.14, 1.14], "ease": "linear"}],
     "opacity": ramp(7000, 7150, 0.0, 1.0)},
]})

# ---- claim, 9 - 13 s, Ink ground: two pairs of lines, on the bar and the next beat
CY, CL = 258, 141
claim = [("claim-1", 9000, [{"text": "Your video\nis a "}, {"text": "file", "color": SIGNAL}], 700, CY),
         ("claim-2", 9500, [{"text": "your agent\ncan check."}], 720, CY + 2 * CL)]
for id, t, runs, w, y in claim:
    tracks.append({"name": id, "layer": 10, "elements": [
        text(id, t, 13000, HX, ramp(t, t + 350, y + 48, y), w, 2 * CL, 128, runs,
             opacity=[{"t": t, "v": 0.0}, {"t": t + 200, "v": 1.0, "ease": "ease-out"},
                      {"t": 12716, "v": 1.0, "ease": "linear"}, {"t": 12966, "v": 0.0, "ease": "ease-in"}])]})

# ---- brand, 13 - 15 s: lockup on Ink, then the descriptor on the next beat
tracks.append({"name": "brand", "layer": 10, "elements": [
    {"id": "lockup", "type": "image", "start": 13000, "end": 15000, "source": "brand/lockup-on-dark.png",
     "x": 540, "y": 500, "origin": "center", "width": 770, "height": 178, "fit": "contain",
     "scale": [{"t": 13000, "v": [0.92, 0.92]}, {"t": 13400, "v": [1.0, 1.0], "ease": "ease-out"}],
     "opacity": ramp(13000, 13200, 0.0, 1.0)},
]})
tracks.append({"name": "tagline", "layer": 10, "elements": [
    text("tagline", 13500, 15000, 540, ramp(13500, 13800, 668, 652), 700, 53, 44,
         [{"text": "The video editor AI agents drive."}], font="regular",
         opacity=ramp(13500, 13700, 0.0, 1.0)) | {"origin": "top-center", "align": "center"},
]})

# ---- music bed under everything, faded over the held chord of the last second
tracks.append({"name": "music", "layer": 0, "elements": [
    {"id": "bed", "type": "audio", "start": 0, "end": 15000, "source": "music/bed-120bpm.wav",
     "source_start": 0, "source_end": 15000,
     "volume": [{"t": 0, "v": 1.0}, {"t": 14000, "v": 1.0, "ease": "linear"},
                {"t": 14966, "v": 0.0, "ease": "ease-in"}]},
]})

d["tracks"] = tracks
json.dump(d, open(P, "w"), indent=2, ensure_ascii=False)
