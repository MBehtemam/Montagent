# Generates the tracks of logo-loop.json. Frame-aligned: a boundary at frame f is floor(f*1000/30) ms.
import json, math
F = lambda f: (f * 1000) // 30
PAPER, SIGNAL = "#F5F0E6", "#FF5A36"
EASE_IN = [0.16, 1, 0.3, 1]          # fast arrival, long settle
END = F(165)                          # 5500: everything gone
FADE = [{"t": F(148), "v": 1.0}, {"t": F(164), "v": 0.0, "ease": "ease-in-out"}]  # 0 on the last sampled frame

# Mark: 2x2 grid, tile 92, gap 8 (the pack's 460:40), corner radius 19 (96/460 of 92).
T, G, R = 92, 8, 19
MX, MY = 115, 444                     # mark top-left
cx = [MX + T // 2, MX + T + G + T // 2]
cy = [MY + T // 2, MY + T + G + T // 2]

def kf(t0, v0, t1, v1, ease=EASE_IN):
    return [{"t": t0, "v": v0}, {"t": t1, "v": v1, "ease": ease}]

tiles = [
  # id, type, x, y, extra ; arrival order: TL (from left), TR circle (from top), BL (from bottom), BR spinner (from right)
  {"id": "tile-tl", "type": "rect", "x": kf(F(0), -120, F(21), cx[0]), "y": cy[0], "fill": PAPER, "radius": R},
  {"id": "tile-tr", "type": "ellipse", "x": cx[1], "y": kf(F(9), -120, F(30), cy[0]), "fill": SIGNAL},
  {"id": "tile-bl", "type": "rect", "x": cx[0], "y": kf(F(18), 1200, F(39), cy[1]), "fill": PAPER, "radius": R},
  # The spinner travels longest so its three turns stay under ~42 deg/frame (a square reads backwards past 45).
  {"id": "tile-br", "type": "rect", "x": kf(F(12), 1200, F(60), cx[1], [0.2, 0.7, 0.3, 1]), "y": cy[1], "fill": PAPER, "radius": R,
   "rotation": kf(F(12), 0.0, F(60), 1080.0, [0.35, 0.5, 0.4, 1])},
]
def shape(t):
    e = {"id": t["id"], "type": t["type"], "group": "mark", "start": 0, "end": END,
         "x": t["x"], "y": t["y"], "origin": "center", "width": T, "height": T, "fill": t["fill"]}
    if "radius" in t: e["radius"] = t["radius"]
    if "rotation" in t: e["rotation"] = t["rotation"]
    e["opacity"] = FADE
    return e

# Text: Inter Bold 112; advances measured at 128 and scaled.
SIZE = 112
ADV128 = [119.25, 197.75, 277.4375, 324.3125, 399.25, 480.125, 556.375, 636.0625, 682.9375]
ADV = [a * SIZE / 128 for a in ADV128]
WORD = "Montagent"
TX, TY = MX + 2 * T + G + 44, 540
CUR_W, CUR_H, CUR_GAP = 10, 104, 6
TYPE0, STEP = 90, 5                   # first letter at frame 90 (3.0 s), one every 5 frames
letter_f = [TYPE0 + STEP * k for k in range(9)]

texts = []
for k in range(9):
    s, e = F(letter_f[k]), (F(letter_f[k + 1]) if k < 8 else END)
    el = {"id": f"type-{k+1}", "type": "text", "group": "name", "start": s, "end": e,
          "x": TX, "y": TY, "origin": "center-left", "width": math.ceil(ADV[k]) + 2, "height": SIZE,
          "font": "bold", "size": SIZE, "line_height": 1.0, "color": PAPER, "runs": [{"text": WORD[:k + 1]}]}
    if k == 8: el["opacity"] = FADE
    texts.append(el)

def cursor(i, f0, f1, adv):
    return {"id": f"cursor-{i:02d}", "type": "rect", "group": "cursor", "start": F(f0), "end": F(f1),
            "x": round(TX + adv + CUR_GAP), "y": TY, "origin": "center-left", "width": CUR_W, "height": CUR_H, "fill": PAPER}
# Hard blink, 9 frames (300 ms) on / 9 off; solid while typing.
# Appears at 2.2 s: on 66-75, off 75-84, on 84-90 (all before the first letter), then solid while typing.
cur = [cursor(0, 66, 75, 0)]
cur.append(cursor(11, 84, TYPE0, 0))
for k in range(9):
    cur.append(cursor(k + 1, letter_f[k], letter_f[k + 1] if k < 8 else letter_f[8] + 9, ADV[k]))
# After the last letter: on 130-139, then off at 139 and it stays off; the clear starts at 148.

def dump(e): return json.dumps(e, separators=(",", ":"), ensure_ascii=False)
tracks = [{"name": f"mark-{t['id'][5:]}", "layer": 10 + i, "elements": [shape(t)]} for i, t in enumerate(tiles)]
tracks += [{"name": "name", "layer": 20, "elements": texts}, {"name": "cursor", "layer": 21, "elements": cur}]

p = "logo-loop.json"
doc = json.load(open(p))
doc["tracks"] = tracks
json.dump(doc, open(p, "w"), ensure_ascii=False)
