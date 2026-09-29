# Generates sting.json's tracks. Run from the project directory.
import json, math

INK, PAPER, SIGNAL = "#101418", "#F5F0E6", "#FF5A36"
def t(f): return (f * 1000) // 30          # frame -> ms on the 30 fps grid
def j(o): return json.dumps(o, separators=(",", ":"), ensure_ascii=False)

# Mark: the 1024 box scaled so the visible 960 is 200 px -> tiles 96, gap 8, radius 20.
TILE, GAP, R = 96, 8, 20
MX, MY = 113, 440                           # mark's visible top-left
cx = [MX + TILE // 2, MX + TILE + GAP + TILE // 2]   # 161, 265
cy = [MY + TILE // 2, MY + TILE + GAP + TILE // 2]   # 488, 592

FLY = [0.22, 1, 0.36, 1]                    # quint-ish ease-out
SPIN = [0.33, 0.33, 0.6, 1]                 # decelerating spin, peak ~34 deg/frame
BACKIN = [0.36, 0, 0.66, -0.56]
OUT0, CLEAR_END = 150, t(165)              # clear from frame 150 (5000 ms); gone by 5500

def kf(pairs):
    out = []
    for i, (tt, v, *e) in enumerate(pairs):
        r = {"t": tt, "v": v}
        if i: r["ease"] = e[0]
        out.append(r)
    return out

def tile(id_, typ, colx, rowy, fill, move_axis, frm, t0, t1, out_t0, spin=None):
    el = {"id": id_, "type": typ, "group": "mark", "start": t0, "end": CLEAR_END}
    x, y = cx[colx], cy[rowy]
    el["x"] = kf([(t0, frm, None), (t1, x, FLY)]) if move_axis == "x" else x
    el["y"] = kf([(t0, frm, None), (t1, y, FLY)]) if move_axis == "y" else y
    el["origin"] = "center"
    el["width"] = TILE; el["height"] = TILE
    el["fill"] = fill
    if typ == "rect": el["radius"] = R
    el["scale"] = kf([(t(out_t0), [1, 1], None), (t(out_t0 + 8), [0, 0], BACKIN)])
    if spin: el["rotation"] = kf([(spin[0], 1080, None), (spin[1], 0, SPIN)])
    return el

# key order: id,type,group,start,end,layer,x,y,origin,width,height,fill,stroke,stroke_width,radius,scale,rotation
tiles = [
    ("tile-tl", tile("tile-tl", "rect",    0, 0, PAPER,  "x", -100, t(1),  t(16), OUT0)),
    ("tile-tr", tile("tile-tr", "ellipse", 1, 0, SIGNAL, "y", -100, t(6),  t(21), OUT0 + 2)),
    ("tile-bl", tile("tile-bl", "rect",    0, 1, PAPER,  "y", 1180, t(11), t(26), OUT0 + 4)),
    ("tile-br", tile("tile-br", "rect",    1, 1, PAPER,  "x", 1180, t(15), t(39), OUT0 + 6,
                     spin=(t(15), t(60)))),
]

# Word: one text element per prefix, typed every 5 frames from 3000 ms.
TX, TY, SIZE = 363, 474, 110
# `montagent measure` of each prefix at 110 px, origin top-left, y 0.
m = [{"adv": 102.48046875, "h": 132}, {"adv": 169.94140625, "h": 132}, {"adv": 238.4228515625, "h": 132}, {"adv": 278.7060546875, "h": 132}, {"adv": 343.10546875, "h": 132}, {"adv": 412.607421875, "h": 132}, {"adv": 478.134765625, "h": 132}, {"adv": 546.6162109375, "h": 132}, {"adv": 586.8994140625, "h": 132}]
word = "Montagent"
letters = [90 + 5 * i for i in range(9)]    # frame each letter lands on
TEXT_FADE = (t(150), t(158))
words = []
for i in range(9):
    s = t(letters[i]); e = t(letters[i + 1]) if i < 8 else t(159)
    el = {"id": f"word-{i+1}", "type": "text", "group": "word", "start": s, "end": e,
          "x": TX, "y": TY, "origin": "top-left", "width": math.ceil(m[i]["adv"]), "height": m[i]["h"],
          "font": "inter-bold", "size": SIZE, "color": PAPER, "runs": [{"text": word[: i + 1]}]}
    if i == 8:
        el["opacity"] = kf([(TEXT_FADE[0], 1, None), (TEXT_FADE[1], 0, "ease-in")])
    words.append(el)

# Cursor: hard on/off rects, moved ahead of each new letter.
CW, CH, CY = 10, 108, 486
def cur(id_, s, e, x):
    return {"id": id_, "type": "rect", "group": "cursor", "start": s, "end": e,
            "x": x, "y": CY, "origin": "top-left", "width": CW, "height": CH, "fill": PAPER}
def cx_after(i): return TX if i == 0 else TX + round(m[i - 1]["adv"]) + 6
cursor = [cur("cursor-blink-1", t(72), t(80), cx_after(0)),
          cur("cursor-blink-2", t(88), t(90), cx_after(0))]
for i in range(1, 10):
    s = t(letters[i - 1]); e = t(letters[i]) if i < 9 else t(136)
    cursor.append(cur(f"cursor-{i}", s, e, cx_after(i)))
cursor.append(cur("cursor-blink-3", t(144), t(152), cx_after(9)))

tracks = [("tile-tl", 10, [tiles[0][1]]), ("tile-tr", 11, [tiles[1][1]]),
          ("tile-bl", 12, [tiles[2][1]]), ("tile-br", 13, [tiles[3][1]]),
          ("word", 20, words), ("cursor", 21, cursor)]

src = open("sting.json").read()
head = src[: src.index('  "tracks"')]
lines = ['  "tracks": [']
for ti, (name, layer, els) in enumerate(tracks):
    lines.append("    {")
    lines.append(f'      "name": "{name}",')
    lines.append(f'      "layer": {layer},')
    lines.append('      "elements": [')
    lines += ["        " + j(e) + ("," if k < len(els) - 1 else "") for k, e in enumerate(els)]
    lines.append("      ]")
    lines.append("    }" + ("," if ti < len(tracks) - 1 else ""))
lines.append("  ]")
open("sting.json", "w").write(head + "\n".join(lines) + "\n}\n")
