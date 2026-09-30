"""Prototype: SILENT PROTOCOL, a spy-thriller trailer prototype in Montagent.

Run: uv run --with fonttools python scene.py -> trailer.montagent.json (keeps the fontVendor
table that `montagent fonts vendor` wrote into it).
"""
import json
import math
import random
from pathlib import Path

from fontTools.ttLib import TTFont

import plan as P

HERE = Path(__file__).parent
W, H, FPS = 1920, 1080, 30
FR = 1000 / FPS
rnd = random.Random(5)

OVERSHOOT = [0.34, 1.56, 0.64, 1.0]
EXPO_OUT = [0.16, 1.0, 0.3, 1.0]
EXPO_IN = [0.7, 0.0, 0.84, 0.0]
IN_OUT = [0.65, 0.0, 0.35, 1.0]

GOLD, IVORY, RED, CYAN = "#E3C067", "#E8E2D0", "#FF2A2A", "#22E0FF"
NIGHT = [{"name": "contrast", "amount": 0.15}, {"name": "saturation", "amount": 0.85},
         {"name": "tint", "color": "#1E5A6E", "amount": 0.14}]
WARM = [{"name": "contrast", "amount": 0.12}, {"name": "tint", "color": "#FF8A3D", "amount": 0.08}]
COLD = [{"name": "contrast", "amount": 0.1}, {"name": "tint", "color": "#3A6EA5", "amount": 0.16}]
GRADE = {"rooftop": NIGHT, "stairs": WARM, "cards": WARM, "lair": COLD, "fire": WARM, "boat": WARM}


def glow(color, radius, opacity):
    return {"name": "shadow", "dx": 0, "dy": 0, "radius": radius, "color": color, "opacity": opacity}


# ---- keyframes ------------------------------------------------------------------------

def kf(points):
    """[(t, v) or (t, v, ease)] -> keyframe records; the first carries no ease."""
    out = []
    for n, p in enumerate(points):
        t, v = p[0], p[1]
        rec = {"t": int(round(t)), "v": v}
        if n:
            rec["ease"] = p[2] if len(p) > 2 else "linear"
        out.append(rec)
    return out


def s2(v):
    return [round(v, 4), round(v, 4)]


def frames(a, b):
    n = int((b - a) / FR)
    return [a + i * FR for i in range(n + 1)]


def shake(t0, dur, amp):
    """[(t, dx, dy)] every frame, decaying."""
    out = [(t0, 0.0, 0.0)]
    for t in frames(t0 + FR, t0 + dur):
        k = amp * (1 - (t - t0) / dur) ** 2
        out.append((t, rnd.uniform(-k, k), rnd.uniform(-k, k)))
    out.append((t0 + dur + FR, 0.0, 0.0))
    return out


# ---- the document ---------------------------------------------------------------------

tracks = {}
BANDS = {"shots": 100, "iris": 200, "hud": 300, "text": 400, "title": 500, "fx": 600, "top": 700}
counters = dict(BANDS)


def put(band, el, track=None):
    """Each call without a track name opens a new track, so overlaps never collide."""
    if track is None or track not in tracks:
        name = track or f"{band}-{counters[band]}"
        tracks[name] = {"name": name, "layer": counters[band], "elements": []}
        counters[band] += 1
        track = name
    tracks[track]["elements"].append(el)
    return el


def image(id, src, start, end, w=1920, h=1088, x=960, y=540, **kw):
    el = {"id": id, "type": "image", "start": int(start), "end": int(end), "source": src,
          "x": x, "y": y, "origin": kw.pop("origin", "center"), "width": w, "height": h, "fit": "literal"}
    el.update(kw)
    return el


def rect(id, start, end, w, h, fill, x=0, y=0, origin="top-left", **kw):
    el = {"id": id, "type": "rect", "start": int(start), "end": int(end), "x": x, "y": y, "origin": origin,
          "width": w, "height": h, "fill": fill}
    el.update(kw)
    return el


def ellipse(id, start, end, w, h, x, y, **kw):
    el = {"id": id, "type": "ellipse", "start": int(start), "end": int(end), "x": x, "y": y, "origin": "center",
          "width": w, "height": h}
    el.update(kw)
    return el


FONTS = {"cinzel-bold": "fonts/Cinzel-Bold.ttf", "cinzel": "fonts/Cinzel-Regular.ttf", "oswald": "fonts/Oswald-SemiBold.ttf"}
_tt = {k: TTFont(HERE / v) for k, v in FONTS.items()}


def advance(font, ch, size):
    f = _tt[font]
    glyph = f.getBestCmap().get(ord(ch))
    return f["hmtx"][glyph][0] * size / f["head"].unitsPerEm if glyph else size * 0.3


def text(id, start, end, s, font, size, color, x, y, w=None, h=None, origin="center", align="center", **kw):
    el = {"id": id, "type": "text", "start": int(start), "end": int(end), "x": x, "y": y, "origin": origin,
          "width": w or int(sum(advance(font, c, size) for c in s)) + 80, "height": h or int(size * 1.5),
          "font": font, "size": size, "color": color, "align": align, "runs": [{"text": s}]}
    el.update(kw)
    return el


def letter_xs(s, font, size, spacing, cx=960):
    advs = [advance(font, c, size) for c in s]
    total = sum(advs) + spacing * (len(s) - 1)
    x, out = cx - total / 2, []
    for a in advs:
        out.append(x + a / 2)
        x += a + spacing
    return out


def flash(t, dur=260, peak=1.0):
    put("top", rect(f"flash-{int(t)}", t, t + dur, W, H, "#FFFFFF",
                    opacity=kf([(t, peak), (t + dur, 0.0, "ease-out")])), "flash")


# ---- cold open: the gun-barrel homage -------------------------------------------------

for i, ms in enumerate(P.DOTS):
    last = i == len(P.DOTS) - 1
    end = P.IRIS + 80 if last else ms + 800
    xs = kf([(ms, -60), (1600, 960, EXPO_OUT)]) if last else kf([(ms, -60), (ms + 800, 1980)])
    put("iris", ellipse(f"dot-{i}", ms, end, 64, 64, xs, 540, fill="#F2F2F2",
                        effects=[glow("#FFFFFF", 12, 0.6)]))

put("iris", image("rifling", "fx/rifling.png", P.IRIS, P.BOOM + 350, 1080, 1080,
                  scale=kf([(P.IRIS, s2(0.06)), (P.IRIS + 450, s2(0.62), OVERSHOOT), (P.BOOM - 150, s2(0.7)),
                            (P.BOOM + 350, s2(4.2), EXPO_IN)]),
                  rotation=kf([(P.IRIS, 0), (P.BOOM + 350, -220, "ease-in")]),
                  opacity=kf([(P.BOOM + 100, 1.0), (P.BOOM + 350, 0.0)]),
                  effects=[{"name": "mask", "shape": "circle"}]))
IRIS_IN = P.IRIS + 500
IRIS_OUT = P.BOOM + 350
put("iris", image("iris-rooftop", "img/rooftop.png", IRIS_IN, IRIS_OUT,
                  scale=kf([(IRIS_IN, s2(0.2)), (P.BOOM - 150, s2(0.26)), (IRIS_OUT, s2(2.1), EXPO_IN)]),
                  opacity=kf([(IRIS_IN, 0.0), (IRIS_IN + 300, 1.0)]),
                  effects=NIGHT + [{"name": "mask", "shape": "circle"}]))
flash(P.BOOM, 320)

# ---- the shots ------------------------------------------------------------------------

AB = ["shots-a", "shots-b"]


def shot(id, name, start, end, s0, s1, track, x0=960, x1=960, fade_in=0, shake_at=None, amp=0, ease="linear"):
    xs = kf([(start, x0), (end, x1)]) if x0 != x1 else x0
    ys = 540
    if shake_at is not None:
        sh = shake(shake_at, 600, amp)
        xs = kf([(start, x0)] + [(t, round(x0 + dx)) for t, dx, _ in sh[1:]])
        ys = kf([(start, 540)] + [(t, round(540 + dy)) for t, _, dy in sh[1:]])
    el = image(id, f"img/{name}.png", start, end, x=xs, y=ys,
               scale=kf([(start, s2(s0)), (end, s2(s1), ease)]), effects=GRADE[name])
    if fade_in:
        el["opacity"] = kf([(start, 0.0), (start + fade_in, 1.0)])
    put("shots", el, track)


# s1: the iris blows open onto the rooftop, then pulls out.
put("shots", image("s1", "img/rooftop.png", IRIS_OUT, 5300,
                   scale=kf([(IRIS_OUT, s2(2.1)), (IRIS_OUT + 1300, s2(1.15), EXPO_OUT), (5300, s2(1.1))]),
                   effects=NIGHT), "shots-a")
shot("s2", "stairs", 5000, 6800, 1.0, 1.12, "shots-b", fade_in=300)
shot("s3", "cards", 7900, 10900, 1.02, 1.16, "shots-a", x0=1000, x1=960)
shot("s4", "lair", 12000, 16300, 1.02, 1.14, "shots-b")
shot("s6", "rooftop", 17400, 19700, 1.55, 1.7, "shots-a")
shot("s7", "fire", P.HIT, 30400, 1.3, 1.08, "shots-b", shake_at=P.HIT, amp=34, ease=EXPO_OUT)

# snow over the lair: the ember plate, whitened, falling
put("fx", image("snow", "fx/embers.png", 12000, 16300, 1920, 2400, y=kf([(12000, -300), (16300, 700)]),
                opacity=0.8, effects=[{"name": "saturation", "amount": 0.0}, {"name": "brightness", "amount": 0.4}]))
# embers over the fireball
put("fx", image("embers-fire", "fx/embers.png", P.HIT, 30400, 1920, 2400, y=kf([(P.HIT, 1500), (30400, 300)])))

# ---- text cards -------------------------------------------------------------------------

for cid, s, start, end, style in P.CARDS:
    if style == "track":  # letters drift apart as the words fade up
        for i, (c, xa, xb) in enumerate(zip(s, letter_xs(s, "oswald", 120, 8), letter_xs(s, "oswald", 120, 34))):
            if c == " ":
                continue
            put("text", text(f"{cid}-{i}", start, end, c, "oswald", 120, IVORY,
                             kf([(start, round(xa)), (end, round(xb))]), 540,
                             opacity=kf([(start, 0.0), (start + 350, 1.0), (end - 200, 1.0), (end, 0.0)]),
                             effects=[glow("#FFD58A", 16, 0.35)]))
    elif style == "slam":  # hits the frame, overshoots, shakes
        sh = shake(start + 60, 380, 16)
        put("text", text(cid, start, end, s, "oswald", 140, GOLD,
                         kf([(start, 960)] + [(t, round(960 + dx)) for t, dx, _ in sh[1:]]),
                         kf([(start, 540)] + [(t, round(540 + dy)) for t, _, dy in sh[1:]]),
                         scale=kf([(start, s2(2.6)), (start + 260, s2(1.0), OVERSHOOT), (end, s2(1.06))]),
                         opacity=kf([(start, 0.0), (start + 90, 1.0), (end - 150, 1.0), (end, 0.0)]),
                         effects=[glow("#FF9F2E", 26, 0.6)]))
        flash(start, 180, 0.7)
    elif style == "split":  # red and cyan copies converge into white: a chromatic snap
        for col, off, op in ((RED, -46, 0.75), (CYAN, 46, 0.75)):
            put("text", text(f"{cid}-{col[1:]}", start, end, s, "oswald", 110, col,
                             kf([(start, 960 + off), (start + 420, 960, EXPO_OUT), (end - 250, 960),
                                 (end, 960 - off // 3, "ease-in")]), 540,
                             opacity=kf([(start, op), (start + 500, 0.0)])))
        put("text", text(cid, start, end, s, "oswald", 110, "#FFFFFF", 960, 540,
                         opacity=kf([(start, 0.0), (start + 380, 1.0), (end - 150, 1.0), (end, 0.0)]),
                         effects=[glow("#FFFFFF", 14, 0.35)]))
        flash(start, 180, 0.6)

flash(6800, 160, 0.5)

# ---- the HUD shot: a control room built from shapes -------------------------------------

h0, h1 = P.HUD
put("hud", image("hud-lair", "img/lair.png", h0, h1, opacity=0.3, scale=kf([(h0, s2(1.12)), (h1, s2(1.18))]),
                 effects=[{"name": "tint", "color": "#FF2020", "amount": 0.6}, {"name": "brightness", "amount": -0.3}]))
put("hud", image("hud-map", "fx/hud.png", h0, h1, 1920, 1080, scale=kf([(h0, s2(1.0)), (h1, s2(1.05))]),
                 opacity=kf([(h0, 0.0), (h0 + 60, 1.0, "step"), (h0 + 100, 0.3, "step"), (h0 + 140, 1.0, "step")])))
for k, (ow, oh, rot, speed) in enumerate(((700, 250, -14, 1.3), (960, 330, 9, -0.9), (1220, 420, 22, 0.6))):
    put("hud", ellipse(f"orbit-{k}", h0 + 80 * k, h1, ow, oh, 960, 560, fill="#00000000", stroke="#FF3B30",
                       stroke_width=3, rotation=rot, opacity=0.8,
                       scale=kf([(h0 + 80 * k, s2(0.6)), (h0 + 80 * k + 400, s2(1.0), EXPO_OUT)])))
    ts = frames(h0 + 80 * k, h1)
    pts = []
    for t in ts:
        th = speed * (t - h0) / 1000 * 2 * math.pi + k
        px, py = ow / 2 * math.cos(th), oh / 2 * math.sin(th)
        r = math.radians(rot)
        pts.append((t, 960 + px * math.cos(r) - py * math.sin(r), 560 + px * math.sin(r) + py * math.cos(r)))
    put("hud", ellipse(f"sat-{k}", ts[0], h1, 18, 18, kf([(t, round(x)) for t, x, _ in pts]),
                       kf([(t, round(y)) for t, _, y in pts]), fill="#FF8A80", effects=[glow("#FF3B30", 10, 0.9)]))

# typed readout, one element per keystroke on a single track
for line, (s, y, col, t0) in enumerate((("SATELLITE UPLINK: ACQUIRED", 200, "#FF4A3D", h0 + 150),
                                         ("TARGET: E. VALE", 262, "#F2E6E0", h0 + 150 + 26 * 28 + 80))):
    for n in range(1, len(s) + 1):
        a = t0 + (n - 1) * 28
        b = t0 + n * 28 if n < len(s) else h1
        if b <= a:
            continue
        put("hud", text(f"type-{line}-{n}", a, b, s[:n], "oswald", 46, col, 190, y, w=1200, h=64,
                        origin="top-left", align="start"), f"type-{line}")
put("hud", rect("cursor", h0 + 150, h1, 20, 46, "#FF4A3D",
                x=round(190 + sum(advance("oswald", c, 46) for c in "TARGET: E. VALE") + 10), y=270,
                opacity=kf([(h0 + 150, 1.0)] + [(t, float(i % 2 == 0), "step") for i, t in
                                                  enumerate(range(h0 + 250, h1, 120))])))


def glitch(id, src, t0, dur, grade):
    """Horizontal slices of a picture, each jumping sideways every frame, two colour-split."""
    ys = [0, 150, 260, 420, 520, 700, 820, 960, 1088]
    for k, (a, b) in enumerate(zip(ys, ys[1:])):
        fx = [{"name": "tint", "color": [RED, CYAN, "#FFFFFF"][k % 3], "amount": 0.35}] if k % 3 < 2 else []
        xs = [(t0, 960 + rnd.randint(-90, 90))] + [(t, 960 + rnd.randint(-90, 90), "step") for t in frames(t0 + FR, t0 + dur)]
        put("hud", image(f"{id}-{k}", src, t0, t0 + dur, x=kf(xs),
                         effects=grade + fx + [{"name": "mask", "shape": "rect", "x": 0, "y": a, "width": 1920, "height": b - a}]))


glitch("glitch-in", "img/lair.png", h0, 140, COLD)
glitch("glitch-out", "img/rooftop.png", h1 - 130, 160, NIGHT)

# ---- the montage ------------------------------------------------------------------------

m0, m1 = P.MONTAGE
step = (m1 - m0) / len(P.MONTAGE_SHOTS)
for i, name in enumerate(P.MONTAGE_SHOTS):
    a, b = m0 + i * step, m0 + (i + 1) * step
    shot(f"m{i}", name, a, b, 1.04, 1.16 + 0.02 * i, AB[i % 2], x0=960 + (30 if i % 2 else -30), x1=960,
         shake_at=a if name == "fire" else None, amp=24)
    flash(a, 110, 0.55)
# a single-word flash cut halfway through
put("text", text("m-vale", m0 + 4 * step - 140, m0 + 4 * step, "VALE", "cinzel-bold", 220, GOLD, 960, 540,
                 effects=[glow("#FF9F2E", 30, 0.7)]))

# ---- the drop: "This winter..." ----------------------------------------------------------

d0, d1 = P.DROP
s = "THIS WINTER"
for i, (c, xa, xb) in enumerate(zip(s, letter_xs(s, "cinzel", 96, 6), letter_xs(s, "cinzel", 96, 22))):
    if c == " ":
        continue
    t0 = P.VO["n3"] + 80 + 70 * i
    put("text", text(f"winter-{i}", t0, d1, c, "cinzel", 96, IVORY, kf([(t0, round(xa)), (d1, round(xb))]), 540,
                     opacity=kf([(t0, 0.0), (t0 + 450, 1.0)]), effects=[glow("#FFE3A8", 14, 0.3)]))
flash(P.HIT, 380)

# ---- the title ------------------------------------------------------------------------------

t0, t1 = P.TITLE
s = "SILENT PROTOCOL"
xs = letter_xs(s, "cinzel-bold", 150, 16)
mid = (len(s) - 1) / 2
SET = t0 + 1300  # every letter has landed (the outermost start 550 ms in and land 650 ms later)
for i, (c, xf) in enumerate(zip(s, xs)):
    if c == " ":
        continue
    ti = t0 + 200 + 50 * abs(i - mid)
    spread = 960 + (xf - 960) * 1.5

    def path(g_end=1.045):
        # land, then a slow push over the rest of the title
        return kf([(ti, round(spread)), (ti + 650, round(xf), EXPO_OUT), (SET, round(xf)),
                   (t1, round(960 + (xf - 960) * g_end))])

    sc = kf([(ti, s2(1.4)), (ti + 650, s2(1.0), EXPO_OUT), (SET, s2(1.0)), (t1, s2(1.045))])
    put("title", text(f"title-soft-{i}", ti, ti + 700, c, "cinzel-bold", 150, GOLD, path(), 540, scale=sc,
                      opacity=kf([(ti, 0.0), (ti + 120, 0.9), (ti + 700, 0.0, "ease-out")]),
                      effects=[{"name": "blur", "radius": 14}]))
    put("title", text(f"title-{i}", ti, P.FADE[1], c, "cinzel-bold", 150, GOLD, path(), 540, scale=sc,
                      opacity=kf([(ti, 0.0), (ti + 380, 1.0, "ease-in")]),
                      effects=[glow("#FF9F2E", 28, 0.55)]))

put("fx", image("flare-1", "fx/flare.png", t0 + 150, t0 + 1300, 2400, 240,
                scale=kf([(t0 + 150, [0.3, 1.0]), (t0 + 1300, [1.6, 1.0], EXPO_OUT)]),
                opacity=kf([(t0 + 150, 1.0), (t0 + 1300, 0.0, "ease-in")])))
put("fx", image("sweep", "fx/sweep.png", 31900, 32900, 400, 600, x=kf([(31900, 250), (32900, 1680, IN_OUT)])))
put("fx", image("flare-2", "fx/flare.png", 32300, 33100, 2400, 240, x=kf([(32300, 700), (33100, 1300)]),
                scale=[0.6, 0.6], opacity=kf([(32300, 0.0), (32550, 0.8), (33100, 0.0)])))
put("fx", image("embers-a", "fx/embers.png", t0, P.FADE[1], 1920, 2400, y=kf([(t0, 1740), (P.FADE[1], 340)]),
                opacity=0.9))
put("fx", image("embers-b", "fx/embers.png", t0, P.FADE[1], 1920, 2400, x=1060, y=kf([(t0, 2100), (P.FADE[1], 100)]),
                scale=[1.5, 1.5], opacity=0.5))
s = "COMING SOON"
for i, (c, xa, xb) in enumerate(zip(s, letter_xs(s, "oswald", 44, 10), letter_xs(s, "oswald", 44, 22))):
    if c == " ":
        continue
    put("text", text(f"soon-{i}", P.COMING, P.FADE[1], c, "oswald", 44, "#BFB8A5",
                     kf([(P.COMING, round(xa)), (P.FADE[1], round(xb))]), 700,
                     opacity=kf([(P.COMING, 0.0), (P.COMING + 700, 1.0)])))

# ---- frame furniture: letterbox, vignette, grain, the final fade -----------------------------

LB = 138
open_ = kf([(P.BOOM, [1.0, 0.0]), (P.BOOM + 450, [1.0, 1.0], EXPO_OUT)])
put("top", rect("bar-top", P.BOOM, P.DURATION, W, LB, "#000000", scale=open_))
put("top", rect("bar-bottom", P.BOOM, P.DURATION, W, LB, "#000000", y=H, origin="bottom-left", scale=open_))
put("top", image("vignette", "fx/vignette.png", P.BOOM, P.DURATION, 1920, 1080))
gx = [(0, 960, None)] + [(t, 960 + rnd.randint(-220, 220), "step") for t in frames(2 * FR, P.DURATION - FR)[::2]]
gy = [(0, 540, None)] + [(t, 540 + rnd.randint(-150, 150), "step") for t in frames(2 * FR, P.DURATION - FR)[::2]]
put("top", image("grain", "fx/grain.png", 0, P.DURATION, 2400, 1400,
                 x=kf([p if p[2] else p[:2] for p in gx]), y=kf([p if p[2] else p[:2] for p in gy])))
put("top", rect("fade", P.FADE[0], P.DURATION, W, H, "#000000", opacity=kf([(P.FADE[0], 0.0), (P.FADE[1], 1.0)])))

# ---- sound ------------------------------------------------------------------------------------

vo = {k: json.loads((HERE / f"vo/{k}.json").read_text())["duration_ms"] for k in P.VO}
duck = [(0, 1.0)]
for k, at in sorted(P.VO.items(), key=lambda kv: kv[1]):
    duck += [(at - 150, 1.0), (at, 0.42), (at + vo[k], 0.42), (at + vo[k] + 300, 1.0)]
audio = [{"id": "score", "type": "audio", "start": 0, "end": P.DURATION, "source": "score.wav", "source_start": 0,
          "source_end": P.DURATION, "volume": kf(duck)}]
voice = [{"id": f"vo-{k}", "type": "audio", "start": at, "end": at + vo[k], "source": f"vo/{k}.wav",
          "source_start": 0, "source_end": vo[k], "volume": 1.25} for k, at in P.VO.items()]

doc_path = HERE / "trailer.montagent.json"
vendor = json.loads(doc_path.read_text()).get("fontVendor") if doc_path.exists() else None
doc = {"frame": {"width": W, "height": H}, "fps": FPS, "background": "#000000", "duration": P.DURATION,
       "output": "out/silent-protocol.mp4", "fonts": {k: [{"file": v}] for k, v in FONTS.items()}}
if vendor:
    doc["fontVendor"] = vendor
doc["tracks"] = list(tracks.values()) + [
    {"name": "score", "layer": 900, "elements": audio},
    {"name": "voice", "layer": 901, "elements": voice}]
doc_path.write_text(json.dumps(doc, indent=1))
print("tracks", len(doc["tracks"]), "elements", sum(len(t["elements"]) for t in doc["tracks"]))
