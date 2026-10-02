"""Generate project.json for brief H5 (presenter cut, 4:5)."""
import json, math, subprocess

INK, PAPER, SIGNAL = "#101418", "#F5F0E6", "#FF5A36"
W, H = 1080, 1350

words = json.load(open("presenter/take-2.words.json"))
words[0]["word"] = "Here’s"

# Presenter placement: 1.1x the 1920x1080 take, bottom-aligned, head centred.
S = 1.1
VW, VH = round(1920 * S), round(1080 * S)
VX = 540 + round((960 - 942) * S)
VIDEO_END = 10000  # take is 9800 ms; last frame held under the close fade
CLOSE = 9600       # last word ends at 9500

CAP_Y = 1135
PAD_X, PAD_Y = 40, 24

# (first word index, last word index exclusive, start, end, size, line breaks after idx, emphasis)
chunks = [
    (0, 3, 0, 1260, 60, [], False),
    (3, 8, 1260, 2480, 60, [], False),
    (8, 13, 2480, 4168, 60, [], False),
    (13, 18, 4168, 6726, 60, [], False),
    (18, 26, 6726, VIDEO_END, 80, [20, 22], True),
]


def measure(el):
    out = subprocess.run(
        ["montagent", "measure", "project.json", "--json", "--element", json.dumps(el)],
        capture_output=True, text=True, check=True).stdout
    m = json.loads(out)["measure"]
    return m


tracks = {k: [] for k in ["bg", "presenter", "brand", "cards", "captions", "close", "close-logo", "close-text", "voice", "music"]}

tracks["presenter"].append({
    "id": "presenter", "type": "video", "start": 0, "end": VIDEO_END,
    "source": "presenter/take-2.mp4", "source_start": 0, "source_end": 9800,
    "x": VX, "y": H, "origin": "bottom-center", "width": VW, "height": VH, "fit": "contain",
    "overrun": "hold", "volume": 2.1,
    "effects": [{"name": "chroma", "color": "#00FF22", "tolerance": 0.32, "softness": 0.03, "spill": 1.0},
                {"name": "shadow", "dx": 0, "dy": 14, "radius": 36, "color": INK, "opacity": 0.22}],
})

LOCK_H = 96
LOCK_W = round(LOCK_H * 2694 / 623)
tracks["brand"].append({
    "id": "lockup-top", "type": "image", "start": 0, "end": VIDEO_END,
    "source": "brand/lockup.png", "x": 540, "y": 62, "origin": "top-center",
    "width": LOCK_W, "height": LOCK_H, "fit": "contain",
})

def adv(text, size):
    return measure({"font": "bold", "size": size, "runs": [{"text": text}]})["advance_width"]


for n, c in enumerate(chunks, 1):
    a, b, start, end, size, breaks, emph = c
    lh = 1.1 if emph else 1.2
    # split the chunk's word indices into lines
    lines, cur = [], []
    for i in range(a, b):
        cur.append(i)
        if i in breaks:
            lines.append(cur); cur = []
    lines.append(cur)
    slot = size * lh
    block_h = math.ceil(slot * len(lines))
    widths = [adv(" ".join(words[i]["word"] for i in ln), size) for ln in lines]
    tw = math.ceil(max(widths))
    card = {"id": f"card-{n}", "type": "rect", "start": start, "end": end,
            "x": 540, "y": CAP_Y, "origin": "center",
            "width": tw + 2 * PAD_X, "height": block_h + 2 * PAD_Y,
            "fill": SIGNAL if emph else INK + "E6", "radius": 28}
    if emph:
        card["scale"] = [{"t": start, "v": [0.95, 0.95]}, {"t": start + 160, "v": [1.0, 1.0], "ease": "ease-out"}]
    tracks["cards"].append(card)
    for li, ln in enumerate(lines):
        left = 540 - widths[li] / 2
        cy = CAP_Y + (li - (len(lines) - 1) / 2) * slot
        for k, i in enumerate(ln):
            w = words[i]
            prefix = " ".join(words[j]["word"] for j in ln[:k]) + (" " if k else "")
            x = left + (adv(prefix, size) if prefix else 0)
            ww = adv(w["word"], size)
            el = {"id": f"cap-{n}-{i - a + 1:02d}", "type": "text", "group": f"cap-{n}",
                  "start": max(w["start"], start), "end": end,
                  "x": round(x), "y": round(cy), "origin": "center-left",
                  "width": math.ceil(ww) + 2, "height": math.ceil(slot),
                  "font": "bold", "size": size, "line_height": lh,
                  "color": INK if emph else PAPER, "runs": [{"text": w["word"]}]}
            if not emph:
                el["runs"][0]["highlight"] = {"start": el["start"], "end": min(w["end"], end), "color": SIGNAL}
            tracks.setdefault(f"w{i - a + 1}", []).append(el)


tracks["close"].append({
    "id": "close-ground", "type": "rect", "start": CLOSE, "end": 12000,
    "x": 0, "y": 0, "origin": "top-left", "width": W, "height": H, "fill": INK,
    "opacity": [{"t": CLOSE, "v": 0.0}, {"t": CLOSE + 400, "v": 1.0, "ease": "ease-in-out"}],
})
END_W = 780
END_H = round(END_W * 623 / 2694)
tracks["close-logo"].append({
    "id": "close-lockup", "type": "image", "start": 9800, "end": 12000,
    "source": "brand/lockup-on-dark.png", "x": 540, "y": 640, "origin": "center",
    "width": END_W, "height": END_H, "fit": "contain",
    "scale": [{"t": 9800, "v": [0.94, 0.94]}, {"t": 10300, "v": [1.0, 1.0], "ease": "ease-out"}],
    "opacity": [{"t": 9800, "v": 0.0}, {"t": 10200, "v": 1.0, "ease": "ease-out"}],
})
tag = [{"text": "The video editor AI agents drive."}]
tm = measure({"font": "regular", "size": 42, "runs": tag})
tracks["close-text"].append({
    "id": "close-tagline", "type": "text", "start": 10100, "end": 12000,
    "x": 540, "y": 640 + END_H // 2 + 70, "origin": "top-center",
    "width": math.ceil(tm["extent"]["width"]) + 2, "height": tm["block_height"],
    "font": "regular", "size": 42, "color": PAPER, "align": "center", "runs": tag,
    "opacity": [{"t": 10100, "v": 0.0}, {"t": 10500, "v": 0.85, "ease": "ease-out"}],
    "caption": False,
})

tracks["music"].append({
    "id": "music-bed", "type": "audio", "start": 0, "end": 12000,
    "source": "music/bed-120bpm.wav", "source_start": 0, "source_end": 12000,
    "volume": [{"t": 0, "v": 0.0}, {"t": 250, "v": 0.18, "ease": "ease-out"},
               {"t": 9500, "v": 0.18, "ease": "step"}, {"t": 10000, "v": 0.5, "ease": "ease-in-out"},
               {"t": 11000, "v": 0.5, "ease": "step"}, {"t": 11880, "v": 0.0, "ease": "ease-in"}],
})

p = json.load(open("project.json"))
layers = {"bg": 0, "presenter": 10, "brand": 20, "cards": 30, "captions": 40, "close": 50, "close-logo": 60, "close-text": 61, "voice": 70, "music": 80}
layers.update({f"w{k}": 40 + k for k in range(1, 10)})
p["tracks"] = [{"name": k, "layer": layers[k], "elements": v} for k, v in tracks.items() if v]
json.dump(p, open("project.json", "w"), ensure_ascii=False)
subprocess.run(["montagent", "fmt", "project.json"], capture_output=True)
