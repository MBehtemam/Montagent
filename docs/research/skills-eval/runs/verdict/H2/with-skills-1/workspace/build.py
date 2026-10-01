"""Build h2.json: square talking-head cut (brief H2)."""
import json, math, subprocess, sys

FPS = 30
INK, PAPER, SIGNAL = "#101418", "#F5F0E6", "#FF5A36"

def drawn(t):
    """First drawn frame at or after t, as the integer ms that shows on it."""
    n = math.ceil(t * FPS / 1000 - 1e-9)
    return math.floor(n * 1000 / FPS)

def drawn_le(t):
    """Last drawn frame at or before t."""
    n = math.floor(t * FPS / 1000 + 1e-9)
    return math.floor(n * 1000 / FPS)

proj = json.load(open("h2.json"))
proj["tracks"] = []
tracks = proj["tracks"]

def track(name, layer, els):
    tracks.append({"name": name, "layer": layer, "elements": els})

# ---- shots: source px (cx, top) of the presenter -> frame placement -------------
SRC_W, SRC_H = 1920, 1080
SRC_CX, SRC_HAIR = 950, 40          # presenter's centre line and hair top in the source
SRC_DISC_Y = 255                    # neck / upper chest in the source: the disc's centre
PUNCH_IN, CARD_IN, PUNCH_OUT, END = 2633, 3066, 6200, 9000  # 2633: speech resumes at 2616
SHOTS = [  # (id, start, end, scale, frame x of centre line, frame y of hair top)
    ("wide-a", 0, PUNCH_IN, 1.4, 540, 92),
    ("close", PUNCH_IN, PUNCH_OUT, 1.85, 400, 64),
    ("wide-b", PUNCH_OUT, END, 1.4, 540, 92),
]
KEY = {"name": "chroma", "color": "#00FF00", "tolerance": 0.2, "softness": 0.08, "spill": 0.9}
DISC_D = 600  # disc diameter in the wide shot

shots, discs = [], []
for sid, a, b, s, fx, fy in SHOTS:
    w, h = round(SRC_W * s), round(SRC_H * s)
    x0 = round(fx - SRC_CX * s)
    y0 = round(fy - SRC_HAIR * s)
    src_end = min(b, 8920)
    el = {"id": f"shot-{sid}", "type": "video", "start": a, "end": b,
          "source": "presenter/take-3.mp4", "source_start": a, "source_end": src_end}
    if src_end < b:
        el["overrun"] = "hold"
    el.update({"volume": 0, "x": x0, "y": y0, "origin": "top-left", "width": w, "height": h,
               "fit": "contain", "effects": [KEY]})
    shots.append(el)
    d = round(DISC_D * s / 1.4)
    discs.append({"id": f"disc-{sid}", "type": "ellipse", "start": a, "end": b,
                  "x": fx, "y": round(fy + (SRC_DISC_Y - SRC_HAIR) * s), "origin": "center",
                  "width": d, "height": d, "fill": SIGNAL})
track("disc", 5, discs)
track("presenter", 10, shots)

# ---- product card ---------------------------------------------------------------
CARD_W, CARD_H, CARD_R = 360, 290, 28
CARD_X, CARD_Y = 1080 - 44 - CARD_W, 64           # rest position, top-left
OFF_X = 1080 + 40                                  # fully off the right edge
IN_END = drawn(CARD_IN + 500)
OUT_END = drawn(PUNCH_OUT + 333)
FRAME_PAD = 10
def slide(x_rest, x_off):
    return [{"t": CARD_IN, "v": x_off}, {"t": IN_END, "v": x_rest, "ease": [0.25, 1, 0.5, 1]},
            {"t": PUNCH_OUT, "v": x_rest, "ease": "linear"}, {"t": OUT_END, "v": x_off, "ease": "ease-in"}]
SRC_SCALE = 0.5   # the session at half size; the mask shows its top-left
track("card-frame", 40, [{"id": "card-frame", "type": "rect", "start": CARD_IN, "end": OUT_END + 34,
    "x": slide(CARD_X - FRAME_PAD, OFF_X - FRAME_PAD), "y": CARD_Y - FRAME_PAD, "origin": "top-left",
    "width": CARD_W + 2 * FRAME_PAD, "height": CARD_H + 2 * FRAME_PAD, "fill": INK, "radius": CARD_R + FRAME_PAD,
    "effects": [{"name": "shadow", "dx": 0, "dy": 14, "radius": 36, "color": INK, "opacity": 0.3}]}])
SESSION_START = 27500
card_len = OUT_END + 34 - CARD_IN
track("card", 41, [{"id": "card", "type": "video", "start": CARD_IN, "end": CARD_IN + card_len,
    "source": "screen/session.mp4", "source_start": SESSION_START, "source_end": SESSION_START + card_len,
    "volume": 0, "x": slide(CARD_X, OFF_X), "y": CARD_Y, "origin": "top-left",
    "width": round(1920 * SRC_SCALE), "height": round(1080 * SRC_SCALE), "fit": "contain",
    "effects": [{"name": "mask", "shape": "rect", "x": 24, "y": 0, "width": CARD_W, "height": CARD_H, "radius": CARD_R}]}])

# ---- captions -------------------------------------------------------------------
words = json.load(open(sys.argv[1] if len(sys.argv) > 1 else "presenter/take-3.words.json"))
PAGES = [3, 5, 2, 4, 3, 5, 3]   # words per page: phrases, never more than five
ACCENT_WORDS = {"mouse.", "truth"}
CAP_Y, CAP_SIZE, PAD_X, PAD_Y = 900, 64, 34, 18
pages, i = [], 0
for n in PAGES:
    pages.append(words[i:i + n]); i += n
assert i == len(words)
words[8]["start"] = 2616  # "Montagent": silencedetect hears it from 2616, not 2720
texts = [" ".join(w["word"] for w in pg) for pg in pages]
specs = [{"runs": [{"text": t}], "font": "bold", "size": CAP_SIZE} for t in texts]
m = json.loads(subprocess.run(["montagent", "measure", "h2.json", "--elements", json.dumps(specs), "--json"],
                              capture_output=True, text=True).stdout)["measure"]["results"]
prefixes = []   # for each page, the advance of each word's prefix (with its trailing space)
for pg in pages:
    for j in range(len(pg)):
        prefixes.append(" ".join(w["word"] for w in pg[:j]) + (" " if j else ""))
    for w in pg:
        prefixes.append(w["word"])
specs = [{"runs": [{"text": t or " "}], "font": "bold", "size": CAP_SIZE} for t in prefixes]
mw = json.loads(subprocess.run(["montagent", "measure", "h2.json", "--elements", json.dumps(specs), "--json"],
                               capture_output=True, text=True).stdout)["measure"]["results"]
adv = iter([r["ok"]["advance_width"] for r in mw])
word_tracks = [[] for _ in range(max(PAGES))]
pills = []
for k, pg in enumerate(pages):
    a = drawn(pg[0]["start"])
    nxt = drawn(pages[k + 1][0]["start"]) if k + 1 < len(pages) else END
    b = min(nxt, drawn(pg[-1]["end"] + 600))
    width = m[k]["ok"]["advance_width"]
    left = 540 - width / 2
    offs = [next(adv) for _ in pg]
    offs[0] = 0
    wws = [next(adv) for _ in pg]
    for j, w in enumerate(pg):
        col = SIGNAL if w["word"] in ACCENT_WORDS else INK
        word_tracks[j].append({"id": f"cap-{k + 1}-{j + 1}", "type": "text", "start": max(a, drawn(w["start"])),
            "end": b, "x": round(left + offs[j]), "y": CAP_Y, "origin": "center-left",
            "width": math.ceil(wws[j]) + 2, "height": 80, "font": "bold", "size": CAP_SIZE, "color": col,
            "runs": [{"text": w["word"]}]})
    starts = [max(a, drawn(w["start"])) for w in pg] + [b]
    for j in range(len(pg)):   # the pill hugs the words revealed so far
        if starts[j] == starts[j + 1]:
            continue
        right = offs[j] + wws[j]
        pills.append({"id": f"cap-pill-{k + 1}-{j + 1}", "type": "rect", "start": starts[j], "end": starts[j + 1],
                      "x": round(left - PAD_X), "y": CAP_Y, "origin": "center-left",
                      "width": math.ceil(right) + 2 * PAD_X, "height": 77 + 2 * PAD_Y, "fill": "#FFFFFF", "radius": 26,
                      "effects": [{"name": "shadow", "dx": 0, "dy": 8, "radius": 24, "color": INK, "opacity": 0.18}]})
track("captions-bg", 49, pills)
for j, els in enumerate(word_tracks):
    track(f"captions-w{j + 1}", 50, els)
caps = pills

# ---- end card -------------------------------------------------------------------
LW = 760; LH = round(LW * 623 / 2694)
track("endcard", 60, [{"id": "end-lockup", "type": "image", "start": END, "end": 10000,
    "source": "brand/lockup.png", "x": 540, "y": 540, "origin": "center", "width": LW, "height": LH, "fit": "contain",
    "scale": [{"t": END, "v": [1.0, 1.0]}, {"t": 9966, "v": [1.04, 1.04], "ease": "linear"}]}])
track("endcard-ground", 55, [{"id": "end-ground", "type": "rect", "start": END, "end": 10000,
    "x": 0, "y": 0, "origin": "top-left", "width": 1080, "height": 1080, "fill": PAPER}])

# ---- sound ----------------------------------------------------------------------
track("voice", 0, [{"id": "voice", "type": "audio", "start": 0, "end": 8960, "source": "presenter/take-3.mp4",
    "source_start": 0, "source_end": 8960}])
track("music", 0, [{"id": "bed", "type": "audio", "start": 0, "end": 10000, "source": "music/bed-120bpm.wav",
    "source_start": 0, "source_end": 10000, "volume": 0.5}])

json.dump(proj, open("h2.json", "w"), indent=2)

# ---- duck the bed under the voice with the footage skill's script ---------------
out = subprocess.run(["python3", ".claude/skills/montagent-footage/scripts/captions.py", "h2.json",
                      "presenter/take-3.words.json", "scratch/duck.spec.json"], capture_output=True, text=True, check=True)
print(out.stderr, file=sys.stderr)
ducked = json.loads(out.stdout)
vol = next(e["volume"] for t in ducked["tracks"] for e in t["elements"] if e["id"] == "bed")
for t in proj["tracks"]:
    for e in t["elements"]:
        if e["id"] == "bed":
            e["volume"] = vol
json.dump(proj, open("h2.json", "w"), indent=2)
print("pages:", [(c["id"], c["start"], c["end"]) for c in caps], file=sys.stderr)
