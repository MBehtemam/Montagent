#!/usr/bin/env python3
"""Generate social.json (Brief B: talking-head social cut) and write it in canonical form."""
import json, subprocess, math

PROJ = "social.json"
INK, PAPER, SIGNAL = "#101418", "#F5F0E6", "#FF5A36"
W, H = 1080, 1920
END = 10000

words = json.load(open("presenter/take-1.words.json"))


def measure(el):
    out = subprocess.run(["montagent", "measure", PROJ, "--json", "--element", json.dumps(el)],
                         capture_output=True, text=True, check=True).stdout
    return json.loads(out)["measure"]


def E(**kw):
    return kw


# ---------------------------------------------------------------- presenter
presenter = E(id="presenter", type="video", start=0, end=END, source="presenter/take-1.mp4",
              source_start=0, source_end=9760, x=540, y=960, origin="center", width=3413, height=1920,
              fit="cover", clip=[0, 0, W, H], overrun="hold",
              effects=[{"name": "chroma", "color": "#00FF22", "tolerance": 0.25, "softness": 0.05, "spill": 1.0}])

# ---------------------------------------------------------------- captions
# Two-line chunks by index into words, with the line break before word `brk`; each chunk
# shows from its first word until the next chunk starts.
chunks = [(0, 7, 4), (7, 16, 12), (16, 22, 19), (22, 25, None)]
CAP_SIZE, CAP_Y = 60, 1600  # caption block centre; face sits above y≈600
PAD_X, PAD_Y = 44, 26
cap_text, cap_pill = [], []
for ci, (a, b, brk) in enumerate(chunks):
    start = 0 if ci == 0 else words[a]["start"]
    end = words[chunks[ci + 1][0]]["start"] if ci + 1 < len(chunks) else END
    runs = []
    for i in range(a, b):
        w = words[i]
        if i > a:
            runs.append({"text": "\n" if i == brk else " "})
        runs.append({"text": w["word"], "highlight": {"start": w["start"], "end": w["end"], "color": SIGNAL}})
    base = {"font": "bold", "size": CAP_SIZE, "line_height": 1.2, "runs": runs, "y": CAP_Y, "origin": "center"}
    m = measure(base)
    tw = math.ceil(m["extent"]["width"])
    th = m["block_height"]
    ink_mid = (m["ink_top"] + m["ink_bottom"]) / 2  # already in frame space: measured at y=CAP_Y
    cap_text.append(E(id=f"cap-{ci+1:02d}", type="text", start=start, end=end, x=540, y=CAP_Y, origin="center",
                      width=tw + 4, height=th, font="bold", size=CAP_SIZE, line_height=1.2, color=PAPER,
                      align="center", runs=runs))
    cap_pill.append(E(id=f"cap-pill-{ci+1:02d}", type="rect", start=start, end=end, x=540,
                      y=round(ink_mid), origin="center", width=tw + 2 * PAD_X,
                      height=round(m["ink_bottom"] - m["ink_top"]) + 2 * PAD_Y, fill=INK, radius=28,
                      effects=[{"name": "shadow", "dx": 0, "dy": 8, "radius": 24, "color": INK, "opacity": 0.35}]))

# ---------------------------------------------------------------- name bar (lower third)
# Wipe: the Ink bar grows left→right, then an Ink cover over the content shrinks
# towards the right, revealing the content left→right. Off: the reverse, still left→right.
BX, BY, BW, BH = 60, 1190, 760, 176
ON0, ON1, ON2 = 1000, 1250, 1500      # bar in, content in
OFF0, OFF1, OFF2 = 6000, 6250, 6500   # content out, bar out
EASE = [0.65, 0.0, 0.35, 1.0]
bar_in = E(id="nb-bar-in", type="rect", start=ON0, end=OFF1, x=BX, y=BY, origin="top-left", width=BW, height=BH,
           fill=INK, scale=[{"t": ON0, "v": [0.0, 1.0]}, {"t": ON1, "v": [1.0, 1.0], "ease": EASE}])
bar_out = E(id="nb-bar-out", type="rect", start=OFF1, end=OFF2 + 34, x=BX + BW, y=BY, origin="top-right", width=BW,
            height=BH, fill=INK, scale=[{"t": OFF1, "v": [1.0, 1.0]}, {"t": OFF2, "v": [0.0, 1.0], "ease": EASE}])
accent = E(id="nb-accent", type="rect", start=ON1, end=OFF1, x=BX, y=BY, origin="top-left", width=12, height=BH,
           fill=SIGNAL)
MARK = 112
mark = E(id="nb-mark", type="image", start=ON1, end=OFF1, source="brand/mark-on-dark.png", x=BX + 44,
         y=BY + BH // 2, origin="center-left", width=MARK, height=MARK, fit="contain")
TX = BX + 44 + MARK + 32
name_el = {"font": "bold", "size": 68, "line_height": 1.1, "runs": [{"text": "Montagent"}]}
tag_el = {"font": "regular", "size": 36, "line_height": 1.2, "runs": [{"text": "Video your agent can read"}]}
mn, mt = measure(name_el), measure(tag_el)
name = E(id="nb-name", type="text", start=ON1, end=OFF1, x=TX, y=BY + 30, origin="top-left",
         width=math.ceil(mn["extent"]["width"]) + 4, height=mn["block_height"], font="bold", size=68,
         line_height=1.1, color=PAPER, runs=name_el["runs"])
tag = E(id="nb-tagline", type="text", start=ON1, end=OFF1, x=TX, y=BY + 30 + mn["block_height"] + 6,
        origin="top-left", width=math.ceil(mt["extent"]["width"]) + 4, height=mt["block_height"], font="regular",
        size=36, line_height=1.2, color=PAPER, runs=tag_el["runs"])
assert TX + name["width"] < BX + BW and TX + tag["width"] < BX + BW, (name["width"], tag["width"])
cover_in = E(id="nb-cover-in", type="rect", start=ON1, end=ON2 + 34, x=BX + BW, y=BY, origin="top-right", width=BW,
             height=BH, fill=INK, scale=[{"t": ON1, "v": [1.0, 1.0]}, {"t": ON2, "v": [0.0, 1.0], "ease": EASE}])
cover_out = E(id="nb-cover-out", type="rect", start=OFF0, end=OFF1, x=BX, y=BY, origin="top-left", width=BW,
              height=BH, fill=INK, scale=[{"t": OFF0, "v": [0.0, 1.0]}, {"t": OFF1 - 50, "v": [1.0, 1.0], "ease": EASE}])

# ---------------------------------------------------------------- picture-in-picture
# Pops in on "video" (3420 ms), out before the sign-off ends.
PIN, PSET, POUT0, POUT1 = 3420, 3820, 9150, 9400
D = 400                      # circle diameter on screen
SRC_D = 620                  # how much of the 1920×1080 source the circle shows
SRC_C = (360, 412)           # source point the circle centres on: the Update(...) call and its diff
K = D / SRC_D
PW, PH = round(1920 * K), round(1080 * K)
CX, CY = round(SRC_C[0] * K), round(SRC_C[1] * K)   # circle centre, element-local
DX, DY = round(PW / 2 - CX), round(PH / 2 - CY)     # element centre minus circle centre
PX, PY = 840, 900            # circle centre on screen
OVERSHOOT = [0.34, 1.7, 0.64, 1.0]
POP_OUT = [0.5, -0.6, 0.8, 0.4]
pop = [{"t": PIN, "v": [0.0, 0.0]}, {"t": PSET, "v": [1.0, 1.0], "ease": OVERSHOOT},
       {"t": POUT0, "v": [1.0, 1.0], "ease": "linear"}, {"t": POUT1, "v": [0.0, 0.0], "ease": POP_OUT}]


def follow(c, d):
    """The element pivots on its own centre, not the circle's; move it with the same curve so the circle
    centre stays at `c`: pos = c + d * s."""
    return [{"t": PIN, "v": c}, {"t": PSET, "v": c + d, "ease": OVERSHOOT},
            {"t": POUT0, "v": c + d, "ease": "linear"}, {"t": POUT1, "v": c, "ease": POP_OUT}]


RING = 16
ring = E(id="pip-ring", type="ellipse", start=PIN, end=POUT1 + 34, x=PX, y=PY, origin="center", width=D + 2 * RING,
         height=D + 2 * RING, fill=PAPER, scale=pop,
         effects=[{"name": "shadow", "dx": 0, "dy": 12, "radius": 30, "color": INK, "opacity": 0.4}])
# The diff holds still on screen from 33.5 s to 37.0 s; slow that window to fill the PiP.
SRC0, SRC1 = 33500, 37000
dur = POUT1 + 34 - PIN
speed = round((SRC1 - SRC0) / dur, 4)
assert round((SRC1 - SRC0) / speed) == dur, speed
pip = E(id="pip", type="video", start=PIN, end=POUT1 + 34, source="screen/session.mp4", source_start=SRC0,
        source_end=SRC1, x=follow(PX, DX), y=follow(PY, DY), origin="center", width=PW, height=PH, fit="cover",
        scale=pop, speed=speed,
        effects=[{"name": "mask", "shape": "circle", "x": CX - D // 2, "y": CY - D // 2, "width": D, "height": D}])

# ---------------------------------------------------------------- music
speech = [(0, 1740), (2420, 4360), (5621, 6740), (7164, 7820), (8400, 9480)]
LO, HI = 0.2, 0.55
vol = [{"t": 0, "v": LO},
       {"t": 1780, "v": LO, "ease": "linear"}, {"t": 2000, "v": HI - 0.1, "ease": "ease-out"},
       {"t": 2200, "v": HI - 0.1, "ease": "linear"}, {"t": 2400, "v": LO, "ease": "ease-in"},
       {"t": 4400, "v": LO, "ease": "linear"}, {"t": 4700, "v": HI, "ease": "ease-out"},
       {"t": 5300, "v": HI, "ease": "linear"}, {"t": 5600, "v": LO, "ease": "ease-in"},
       {"t": 7820, "v": LO, "ease": "linear"}, {"t": 8000, "v": 0.4, "ease": "ease-out"},
       {"t": 8200, "v": 0.4, "ease": "linear"}, {"t": 8390, "v": LO, "ease": "ease-in"},
       {"t": 9480, "v": LO, "ease": "linear"}, {"t": 9600, "v": 0.35, "ease": "ease-out"},
       {"t": 9900, "v": 0.0, "ease": "ease-in"}]
music = E(id="music", type="audio", start=0, end=END, source="music/bed-120bpm.wav", source_start=0,
          source_end=END, volume=vol)

tracks = [
    {"name": "presenter", "layer": 10, "elements": [presenter]},
    {"name": "pip-ring", "layer": 20, "elements": [ring]},
    {"name": "pip", "layer": 21, "elements": [pip]},
    {"name": "namebar", "layer": 30, "elements": [bar_in, bar_out]},
    {"name": "namebar-accent", "layer": 31, "elements": [accent]},
    {"name": "namebar-mark", "layer": 32, "elements": [mark]},
    {"name": "namebar-name", "layer": 32, "elements": [name]},
    {"name": "namebar-tagline", "layer": 32, "elements": [tag]},
    {"name": "namebar-cover", "layer": 33, "elements": [cover_in, cover_out]},
    {"name": "caption-pill", "layer": 40, "elements": cap_pill},
    {"name": "captions", "layer": 41, "elements": cap_text},
    {"name": "music", "layer": 0, "elements": [music]},
]

doc = json.load(open(PROJ))
doc["tracks"] = tracks
json.dump(doc, open(PROJ, "w"), indent=2, ensure_ascii=False)
subprocess.run(["montagent", "fmt", PROJ], check=True, capture_output=True)
print("written")
