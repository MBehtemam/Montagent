#!/usr/bin/env python3
"""Generate ad.json from the beat sheet. Run, then `montagent fmt ad.json`."""
import json

INK, PAPER, SIGNAL = "#101418", "#F5F0E6", "#FF5A36"
FRAME_EDGE = "#3A4250"
POP = [0.34, 1.56, 0.64, 1]
SETTLE = [0.25, 1, 0.5, 1]
LONG = [0.16, 1, 0.3, 1]
SWEEP = [0.65, 0, 0.35, 1]


def B(n):
    """The nth beat at 120 BPM, downbeat 0 (500 ms = 15 frames, always drawn)."""
    return int(n * 500)


def kf(*pairs):
    out = []
    for i, p in enumerate(pairs):
        rec = {"t": p[0], "v": p[1]}
        if i:
            rec["ease"] = p[2]
        out.append(rec)
    return out


def track(name, layer, *els):
    return {"name": name, "layer": layer, "elements": list(els)}


def text(id, start, end, x, y, size, font, runs, color=PAPER, origin="center", w=None, h=None, **kw):
    e = {"id": id, "type": "text", "start": start, "end": end, "x": x, "y": y, "origin": origin,
         "width": w, "height": h, "font": font, "size": size, "color": color, "align": "center" if origin == "center" else "start",
         "runs": runs, "caption": False}
    e.update(kw)
    return e


tracks = []

# ---------- ground ----------
tracks.append(track("ground", 1, {"id": "light", "type": "rect", "start": B(20), "end": B(28), "x": 0, "y": 0,
                                    "origin": "top-left", "width": 1920, "height": 1080, "fill": PAPER}))

# ---------- hook: Timelines. -> Text. (0-4 s) ----------
HY = 500
tl = text("timelines", 0, B(4), 960, kf((B(3), HY), (B(4) - 100, 1440, "ease-in")), 200, "bold",
          [{"text": "Timelines."}], w=1100, h=240)
strike = {"id": "strike", "type": "rect", "start": B(1), "end": B(4), "x": 430,
          "y": kf((B(3), HY + 12), (B(4) - 100, 1440 + 12, "ease-in")), "origin": "center-left",
          "width": 1060, "height": 22, "fill": SIGNAL,
          "scale": kf((B(1), [0.0, 1.0]), (B(2), [1.0, 1.0], SWEEP))}
txt = text("text-word", B(4), B(8), 960, kf((B(7), HY), (B(7) + 300, HY - 30, "ease-in")), 200, "bold",
           [{"text": "Text."}], w=600, h=240,
           scale=kf((B(4), [0.6, 0.6]), (B(4) + 400, [1.0, 1.0], POP)),
           opacity=kf((B(7), 1.0), (B(7) + 300, 0.0, "ease-in")))
sub = text("subline", B(5), B(8), 960,
           kf((B(5), 700), (B(5) + 400, 660, LONG), (B(7), 660, "linear"), (B(7) + 300, 630, "ease-in")),
           60, "reg", [{"text": "Your agent edits video the way it edits code."}], w=1300, h=72,
           opacity=kf((B(5), 0.0), (B(5) + 300, 1.0, "ease-out"), (B(7), 1.0, "linear"), (B(7) + 300, 0.0, "ease-in")))
tracks.append(track("headline", 20, tl, txt))
tracks.append(track("strike", 21, strike))
tracks.append(track("subline", 22, sub))

# ---------- show: screen recording + list (4-8 s) ----------
RX, RY, RW, RH = 1248, 540, 1152, 648
rec_x = kf((B(8), RX + 1300), (B(9), RX, SETTLE))
tracks.append(track("rec-frame", 30, {
    "id": "rec-frame", "type": "rect", "start": B(8), "end": B(16), "x": rec_x, "y": RY, "origin": "center",
    "width": RW + 8, "height": RH + 8, "fill": FRAME_EDGE, "radius": 32,
    "effects": [{"name": "shadow", "dx": 0, "dy": 28, "radius": 56, "color": "#000000", "opacity": 0.55}]}))
tracks.append(track("rec", 31, {
    "id": "rec", "type": "video", "start": B(8), "end": B(16), "source": "screen/session.mp4",
    "source_start": 0, "source_end": 55000, "x": rec_x, "y": RY, "origin": "center",
    "width": RW, "height": RH, "fit": "contain", "speed": 13.75,
    "effects": [{"name": "mask", "shape": "rect", "radius": 28}]}))

items = ["Reads the format", "Edits one file", "Checks the frames", "Renders"]
for i, label in enumerate(items):
    t = B(9 + 2 * i)
    y = 390 + 100 * i
    tracks.append(track(f"dot-{i+1}", 40, {
        "id": f"dot-{i+1}", "type": "ellipse", "start": t, "end": B(16), "x": 116, "y": y, "origin": "center",
        "width": 18, "height": 18, "fill": SIGNAL, "scale": kf((t, [0.0, 0.0]), (t + 300, [1.0, 1.0], POP))}))
    tracks.append(track(f"item-{i+1}", 41, text(
        f"item-{i+1}", t, B(16), kf((t, 124), (t + 300, 144, LONG)), y, 46, "reg",
        [{"text": f"{i+1}", "font": "bold"}, {"text": f"   {label}"}], origin="center-left", w=480, h=56,
        opacity=kf((t, 0.0), (t + 250, 1.0, "ease-out")))))

# ---------- split screen (8-10 s) ----------
SW, SH, SY, CY = 816, 459, 470, 790
for n, (src, cap, t, x0, x1) in enumerate([
        ("stills/session-01.png", "One line changed.", B(16), -440, 504),
        ("stills/session-02.png", "One video out.", B(17), 2360, 1416)], start=1):
    xs = kf((t, x0), (t + 500, x1, SETTLE))
    tracks.append(track(f"shot-{n}-frame", 50, {
        "id": f"shot-{n}-frame", "type": "rect", "start": t, "end": B(20), "x": xs, "y": SY, "origin": "center",
        "width": SW + 8, "height": SH + 8, "fill": FRAME_EDGE, "radius": 30,
        "effects": [{"name": "shadow", "dx": 0, "dy": 28, "radius": 56, "color": "#000000", "opacity": 0.55}]}))
    tracks.append(track(f"shot-{n}", 51, {
        "id": f"shot-{n}", "type": "image", "start": t, "end": B(20), "source": src, "x": xs, "y": SY,
        "origin": "center", "width": SW, "height": SH, "fit": "contain",
        "effects": [{"name": "mask", "shape": "rect", "radius": 26}]}))
    tracks.append(track(f"cap-{n}", 52, text(f"cap-{n}", t, B(20), xs, CY, 54, "bold", [{"text": cap}], w=700, h=65)))

# ---------- brand (10-14 s) ----------
T, G = 184, 16                  # tile size and gap at 0.4 x the 1024 construction
OFF = (T + G) // 2              # 100: tile centre from the mark's centre
AX, AY = 960, 520               # mark centre while it assembles
LS = 0.75                       # lockup scale
LX, LY = 372, 470               # mark centre in the lockup
tiles = [  # (name, col, row, shape, colour), in arrival order: accent last
    ("tl", -1, -1, "rect", INK), ("bl", -1, 1, "rect", INK), ("br", 1, 1, "rect", INK), ("tr", 1, -1, "ellipse", SIGNAL)]
for i, (name, cx, cy, shape, col) in enumerate(tiles):
    t = B(20 + i)
    ax, ay = AX + cx * OFF, AY + cy * OFF
    lx, ly = int(LX + cx * OFF * LS), int(LY + cy * OFF * LS)
    e = {"id": f"tile-{name}", "type": shape, "start": t, "end": B(28),
         "x": kf((B(24), ax), (B(25), lx, SWEEP)),
         "y": kf((t, ay - 220), (t + 400, ay, POP), (B(24), ay, "linear"), (B(25), ly, SWEEP)),
         "origin": "center", "width": T, "height": T, "fill": col}
    if shape == "rect":
        e["radius"] = 38
    e["scale"] = kf((B(24), [1.0, 1.0]), (B(25), [LS, LS], SWEEP))
    e["opacity"] = kf((t, 0.0), (t + 133, 1.0, "linear"))
    tracks.append(track(f"tile-{name}", 60 + i, e))

WM_W, WM_H = 1129, 237
WM_X = LX + 144 + 109 - 36      # mark right edge + gap - wordmark's left side bearing
tracks.append(track("wordmark", 70, {
    "id": "wordmark", "type": "image", "start": B(24), "end": B(28), "source": "brand/wordmark.png",
    "x": kf((B(24), AX + T + G // 2 + 109 - 36), (B(25), WM_X, SWEEP)), "y": kf((B(24), AY), (B(25), LY, SWEEP)), "origin": "center-left",
    "width": WM_W, "height": WM_H, "fit": "contain",
    "opacity": kf((B(24), 0.0), (B(24) + 300, 1.0, "ease-out"))}))
tracks.append(track("tagline", 71, text(
    "tagline", B(25), B(28), 960, kf((B(25), 730), (B(25) + 500, 710, LONG)), 56, "reg",
    [{"text": "The video editor your agent drives."}], color=INK, w=1000, h=68,
    opacity=kf((B(25), 0.0), (B(25) + 400, 1.0, "ease-out")))))

# ---------- music ----------
tracks.append(track("music", 0, {
    "id": "bed", "type": "audio", "start": 0, "end": B(28), "source": "music/bed-120bpm.wav",
    "source_start": 0, "source_end": 14000, "volume": kf((B(26), 0.8), (13966, 0.0, "linear"))}))

p = json.load(open("ad.json"))
p["tracks"] = tracks
json.dump(p, open("ad.json", "w"), indent=2)
