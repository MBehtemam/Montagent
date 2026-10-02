"""Generates h5.json: presenter cut for a feed (1080x1350, 25 fps, 12 s)."""
import json, os, subprocess, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PROJECT = os.path.join(ROOT, "h5.json")

W, H, FPS, DUR = 1080, 1350, 25, 12000
INK, PAPER, SIGNAL = "#101418", "#F5F0E6", "#FF5A36"

# chroma key
KEY, TOL, SOFT, SPILL = "#00FF22", 0.30, 0.05, 0.90
if os.environ.get("KEYCFG"):
    TOL, SOFT, SPILL = map(float, os.environ["KEYCFG"].split(","))

# presenter framing: source 1920x1080, presenter centred around x=945
S = 1.3
VW, VH = round(1920 * S), round(1080 * S)
VX, VY = round(540 - 945 * S), 190 - round(44 * S)   # hair top at y=190, below the header

CLOSE = 9800            # presenter's last word ends 9500 (speech to ~9610 ms)

words = json.load(open(os.path.join(ROOT, "presenter/take-2.words.json")))

# caption chunks: (first word idx, last word idx inclusive, emphasis, line break before word idx or None)
CHUNKS = [
    (0, 2, False, None),     # Here's the trick.
    (3, 7, False, 5),        # Every element / in the video
    (8, 12, False, None),    # is one line of text,
    (13, 17, False, None),   # so my agent can change
    (18, 20, True, None),    # exactly one thing
    (21, 25, True, 23),      # and prove / nothing else moved.
]
SIGNAL_WORDS = {19, 23}      # "one", "nothing"
CAP_Y = 1150                 # caption block centre
NORMAL_SIZE, EMPH_SIZE = 62, 78
PAD_X, PAD_Y = 40, 26


def runs_for(chunk, upto):
    """All of the chunk's words, so the layout never moves; words after `upto` are transparent."""
    a, b, emph, br = chunk
    base = INK if emph else PAPER
    out = []
    for i in range(a, b + 1):
        txt = words[i]["word"]
        if i < b:
            txt += "\n" if (br is not None and i + 1 == br) else " "
        col = SIGNAL if i in SIGNAL_WORDS else base
        out.append({"text": txt, "color": col if i <= upto else col + "00"})
    return out


def chunk_end(k):
    a, b, _, _ = CHUNKS[k]
    if k + 1 == len(CHUNKS):
        return CLOSE
    nxt = words[CHUNKS[k + 1][0]]["start"]
    last = words[b]["end"]
    return nxt if nxt - last < 800 else last + 500


def caption_elements(measure=None):
    """One card per chunk; one text element per spoken word, each revealing the chunk up to that word."""
    texts, cards = [], []
    for k, ch in enumerate(CHUNKS):
        a, b, emph, br = ch
        start, end = words[a]["start"], chunk_end(k)
        size = EMPH_SIZE if emph else NORMAL_SIZE
        mid = f"cap-{k+1:02d}-{b:02d}"   # the fully revealed step is the one measured
        tw, th = (measure or {}).get(mid, (900, 200))
        pop = [{"t": start, "v": [0.86, 0.86] if emph else [0.96, 0.96]},
               {"t": start + (200 if emph else 100), "v": [1.0, 1.0],
                "ease": [0.34, 1.56, 0.64, 1.0] if emph else "ease-out"}]
        fade = [{"t": CLOSE - 240, "v": 1.0}, {"t": CLOSE - 40, "v": 0.0, "ease": "ease-in"}]
        c = {"id": f"card-{k+1:02d}", "type": "rect", "start": start, "end": end, "x": 540, "y": CAP_Y,
             "origin": "center", "width": tw + 2 * PAD_X, "height": th + 2 * PAD_Y,
             "fill": PAPER if emph else INK, "radius": 28, "scale": pop}
        if k + 1 == len(CHUNKS):
            c["opacity"] = fade
        cards.append(c)
        for j in range(a, b + 1):
            s = start if j == a else words[j]["start"]
            e = end if j == b else words[j + 1]["start"]
            t = {"id": f"cap-{k+1:02d}-{j:02d}", "group": f"cap-{k+1:02d}", "type": "text", "start": s, "end": e,
                 "x": 540, "y": CAP_Y, "origin": "center", "width": tw, "height": th, "font": "bold",
                 "size": size, "line_height": 1.2, "align": "center", "runs": runs_for(ch, j)}
            if s < start + 240:
                t["scale"] = pop
            if k + 1 == len(CHUNKS) and j == b:
                t["opacity"] = fade
            t["caption"] = False   # a reveal step, not a standalone caption: the chunk is the caption
            texts.append(t)
    return texts, cards


def build(measure=None):
    texts, cards = caption_elements(measure)
    LW, LH = 320, 74  # lockup-on-dark 2694x623
    tracks = [
        # the mark's four shapes as a faint Paper texture behind the presenter (1024 box scaled 1.25)
        {"name": f"bg-{n}", "layer": 0, "elements": [dict(
            {"id": f"bg-{n}", "type": kind, "start": 0, "end": 10200, "x": x, "y": y, "origin": "top-left",
             "width": 575, "height": 575, "fill": PAPER},
            **({"radius": 120} if kind == "rect" else {}),
            opacity=[{"t": CLOSE - 240, "v": 0.06}, {"t": 10160, "v": 0.0, "ease": "ease-in"}])]}
        for n, kind, x, y in [("tl", "rect", -60, 90), ("tr", "ellipse", 565, 90),
                              ("bl", "rect", -60, 715), ("br", "rect", 565, 715)]
    ] + [
        {"name": "presenter", "layer": 10, "elements": [
            {"id": "take-2", "type": "video", "start": 0, "end": 9800, "source": "presenter/take-2.mp4",
             "source_start": 0, "source_end": 9800, "x": VX, "y": VY, "origin": "top-left", "width": VW, "height": VH,
             "fit": "literal",
             "opacity": [{"t": CLOSE - 240, "v": 1.0}, {"t": CLOSE - 40, "v": 0.0, "ease": "ease-in"}],
             "volume": 1.6, "effects": [{"name": "chroma", "color": KEY, "tolerance": TOL, "softness": SOFT, "spill": SPILL}]},
        ]},
        {"name": "header", "layer": 15, "elements": [
            {"id": "header-lockup", "type": "image", "start": 0, "end": CLOSE, "source": "brand/lockup-on-dark.png",
             "x": 540, "y": 100, "origin": "center", "width": LW, "height": LH, "fit": "literal",
             "opacity": [{"t": 0, "v": 0.0}, {"t": 300, "v": 1.0, "ease": "ease-out"},
                         {"t": CLOSE - 240, "v": 1.0, "ease": "step"}, {"t": CLOSE - 40, "v": 0.0, "ease": "ease-in"}]},
        ]},
        {"name": "cards", "layer": 20, "elements": cards},
        {"name": "captions", "layer": 21, "elements": texts},
        {"name": "end-mark", "layer": 30, "elements": [
            {"id": "end-mark", "type": "image", "start": CLOSE, "end": DUR, "source": "brand/mark-on-dark.png",
             "x": 540, "y": 560, "origin": "center", "width": 340, "height": 340, "fit": "literal",
             "scale": [{"t": CLOSE, "v": [0.7, 0.7]}, {"t": CLOSE + 400, "v": [1.0, 1.0], "ease": [0.34, 1.56, 0.64, 1.0]}],
             "opacity": [{"t": CLOSE, "v": 0.0}, {"t": CLOSE + 200, "v": 1.0, "ease": "ease-out"}]},
        ]},
        {"name": "end-word", "layer": 31, "elements": [
            {"id": "end-wordmark", "type": "image", "start": CLOSE + 150, "end": DUR, "source": "brand/wordmark-on-dark.png",
             "x": 540, "y": [{"t": CLOSE + 150, "v": 850}, {"t": CLOSE + 550, "v": 830, "ease": "ease-out"}],
             "origin": "center", "width": 620, "height": 130, "fit": "literal",
             "opacity": [{"t": CLOSE + 150, "v": 0.0}, {"t": CLOSE + 450, "v": 1.0, "ease": "ease-out"}]},
        ]},
        {"name": "end-tag", "layer": 32, "elements": [
            {"id": "end-tagline", "type": "text", "start": CLOSE + 350, "end": DUR, "x": 540,
             "y": [{"t": CLOSE + 350, "v": 975}, {"t": CLOSE + 750, "v": 955, "ease": "ease-out"}],
             "origin": "center", "width": (measure or {}).get("end-tagline", (900, 60))[0],
             "height": (measure or {}).get("end-tagline", (900, 60))[1], "font": "regular", "size": 40,
             "color": PAPER, "align": "center", "runs": [{"text": "The video editor AI agents drive."}],
             "opacity": [{"t": CLOSE + 350, "v": 0.0}, {"t": CLOSE + 750, "v": 1.0, "ease": "ease-out"}],
             "caption": False},
        ]},
        {"name": "music", "layer": 0, "elements": [
            {"id": "bed", "type": "audio", "start": 0, "end": DUR, "source": "music/bed-120bpm.wav",
             "source_start": 0, "source_end": DUR,
             "volume": [{"t": 0, "v": 0.0}, {"t": 300, "v": 0.14, "ease": "ease-out"},
                        {"t": 9600, "v": 0.14, "ease": "step"}, {"t": 10000, "v": 0.6, "ease": "ease-in-out"},
                        {"t": 10800, "v": 0.6, "ease": "step"}, {"t": 11850, "v": 0.0, "ease": "ease-in"}]},
        ]},
    ]
    doc = {
        "frame": {"width": W, "height": H}, "fps": FPS, "background": INK, "duration": DUR,
        "output": "deliverable.mp4",
        "fonts": {"bold": [{"file": "fonts/Inter-Bold.ttf"}], "regular": [{"file": "fonts/Inter-Regular.ttf"}]},
        "fontVendor": json.load(open(PROJECT))["fontVendor"],
        "tracks": tracks,
    }
    with open(PROJECT, "w") as f:
        json.dump(doc, f, ensure_ascii=False)
    subprocess.run(["montagent", "fmt", PROJECT], capture_output=True)


build()
res = subprocess.run(["montagent", "measure", PROJECT, "--all", "--json"], capture_output=True, text=True)
data = json.loads(res.stdout)
if "--dump" in sys.argv:
    print(json.dumps(data, indent=1)[:3000])
meas = {}
for item in data["measure"]["results"]:
    m = item["ok"]
    meas[item["id"]] = (int(-(-m["advance_width"] // 1)) + 2, m["block_height"])
    print(item["id"], m["line_count"], round(m["advance_width"]), m["block_height"])
build(meas)
