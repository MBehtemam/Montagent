"""Build h2.json: square talking-head cut. Run: python3 build.py"""
import json, math, os, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
FPS = 30
INK, PAPER, SIGNAL = "#101418", "#F5F0E6", "#FF5A36"


def on_frame(t):
    return math.floor(round(t * FPS / 1000) * 1000 / FPS)


# Cuts, all on drawn frames.
PUNCH_IN = 2600     # in the pause before "Montagent" (speech resumes 2616)
CARD_IN = on_frame(3080)    # "hands"
PUNCH_OUT = 6200    # in the pause before "the truth" (6222)
END = 9000
DUR = 10000

# Presenter framings. Source face centre ~(952, 200), 1920x1080.
SRC_W, SRC_H, FACE_X = 1920, 1080, 952
WIDE = dict(s=1.1, cx=540, top=70)
CLOSE = dict(s=1.5, cx=460, top=20)
KEY = [{"name": "chroma", "color": "#00FF00", "tolerance": 0.23, "softness": 0.08, "spill": 0.9}]


def shot(i, start, end, f, src_end=None):
    w, h = round(SRC_W * f["s"]), round(SRC_H * f["s"])
    el = {"id": f"shot-{i}", "type": "video", "start": start, "end": end,
          "source": "presenter/take-3.mp4", "source_start": start,
          "source_end": src_end if src_end is not None else end,
          "x": round(f["cx"] - FACE_X * f["s"]), "y": f["top"], "origin": "top-left",
          "width": w, "height": h, "fit": "cover"}
    if src_end is not None and src_end < end:
        el["overrun"] = "hold"
    el["volume"] = 0
    el["effects"] = KEY
    return el


def disc(i, start, end, f):
    # Behind head and shoulders; scales with the framing.
    k = f["s"] / WIDE["s"]
    face_y = f["top"] + 200 * f["s"]
    d = round(660 * k)
    return {"id": f"disc-{i}", "type": "ellipse", "start": start, "end": end,
            "x": f["cx"], "y": round(face_y + 110 * k), "origin": "center",
            "width": d, "height": d, "fill": SIGNAL}


# Product card, upper right, slides in from / out to the right edge.
CARD_W, CARD_H, BORDER = 380, 222, 10
CARD_X, CARD_Y = 852, 164
OFF_X = 1080 + CARD_W // 2 + 40
IN_LEN, OUT_LEN = 433, 300
card_end = on_frame(PUNCH_OUT + OUT_LEN) + 34
slide = [{"t": CARD_IN, "v": OFF_X},
         {"t": on_frame(CARD_IN + IN_LEN), "v": CARD_X, "ease": [0.25, 1, 0.5, 1]},
         {"t": PUNCH_OUT, "v": CARD_X, "ease": "linear"},
         {"t": on_frame(PUNCH_OUT + OUT_LEN), "v": OFF_X, "ease": "ease-in"}]
vid_w, vid_h = CARD_W - 2 * BORDER, CARD_H - 2 * BORDER
card_frame = {"id": "card-frame", "type": "rect", "start": CARD_IN, "end": card_end,
              "x": slide, "y": CARD_Y, "origin": "center", "width": CARD_W, "height": CARD_H,
              "fill": "#FFFFFF", "radius": 28,
              "effects": [{"name": "shadow", "dx": 0, "dy": 14, "radius": 32, "color": INK, "opacity": 0.28}]}
card_span = card_end - CARD_IN
card_video = {"id": "card-video", "type": "video", "start": CARD_IN, "end": card_end,
              "source": "screen/session.mp4", "source_start": 33000, "source_end": 33000 + 2 * card_span,
              "x": slide, "y": CARD_Y, "origin": "center", "width": vid_w, "height": vid_h,
              "fit": "cover", "speed": 2, "volume": 0,
              "effects": [{"name": "mask", "shape": "rect", "x": 0, "y": 0, "width": vid_w, "height": vid_h, "radius": 18}]}

# End card.
LOCK_W = 760
LOCK_H = round(LOCK_W * 623 / 2694)
lockup = {"id": "lockup", "type": "image", "start": END, "end": DUR, "source": "brand/lockup.png",
          "x": 540, "y": 540, "origin": "center", "width": LOCK_W, "height": LOCK_H, "fit": "contain",
          "scale": [{"t": END, "v": [1.0, 1.0]}, {"t": 9966, "v": [1.05, 1.05], "ease": "linear"}]}
endground = {"id": "end-ground", "type": "rect", "start": END, "end": DUR, "x": 0, "y": 0,
             "origin": "top-left", "width": 1080, "height": 1080, "fill": PAPER}

project = {
    "frame": {"width": 1080, "height": 1080}, "fps": FPS, "background": PAPER, "duration": DUR,
    "output": "deliverable.mp4",
    "fonts": {"caption": [{"file": "fonts/Inter-Bold.ttf"}]},
    "fontVendor": {"fonts/Inter-Bold.ttf": {"licence": "OFL-1.1", "source": "https://github.com/rsms/inter",
                                            "sha256": "288316099b1e0a47a4716d159098005eef7c0066921f34e3200393dbdb01947f"}},
    "tracks": [
        {"name": "disc", "layer": 5, "elements": [disc(1, 0, PUNCH_IN, WIDE), disc(2, PUNCH_IN, PUNCH_OUT, CLOSE),
                                                  disc(3, PUNCH_OUT, END, WIDE)]},
        {"name": "presenter", "layer": 10, "elements": [shot(1, 0, PUNCH_IN, WIDE), shot(2, PUNCH_IN, PUNCH_OUT, CLOSE),
                                                        shot(3, PUNCH_OUT, END, WIDE, src_end=8920)]},
        {"name": "card-frame", "layer": 20, "elements": [card_frame]},
        {"name": "card-video", "layer": 21, "elements": [card_video]},
        {"name": "end-ground", "layer": 50, "elements": [endground]},
        {"name": "lockup", "layer": 51, "elements": [lockup]},
        {"name": "voice", "layer": 0, "elements": [
            {"id": "voice", "type": "audio", "start": 0, "end": 8920, "source": "presenter/take-3.mp4",
             "source_start": 0, "source_end": 8920, "volume": 1.0}]},
        {"name": "music", "layer": 0, "elements": [
            {"id": "bed", "type": "audio", "start": 0, "end": DUR, "source": "music/bed-120bpm.wav",
             "source_start": 0, "source_end": DUR, "volume": 0.5}]},
    ],
}

# Timing check and duck from the bundled captions script; its caption pages are discarded.
SIZE, CAP_Y = 60, 905
tmp = tempfile.mkdtemp(dir=os.environ.get("TMPDIR"))
proj_tmp = os.path.join(HERE, "h2.json")
json.dump(project, open(proj_tmp, "w"))
spec = {"track": "cap-tmp", "layer": 40, "voice": "voice", "font": "caption", "size": SIZE,
        "color": INK, "highlight": SIGNAL, "x": 540, "y": CAP_Y, "width": 100000, "lines": 1,
        "duck": {"id": "bed", "under": 0.15, "over": 0.5, "ramp": 200, "lead": 100, "join": 600, "fade": 800}}
spec_path = os.path.join(tmp, "spec.json")
json.dump(spec, open(spec_path, "w"))
out = subprocess.run([sys.executable, os.path.join(HERE, ".claude/skills/montagent-footage/scripts/captions.py"),
                      proj_tmp, os.path.join(HERE, "presenter/take-3.words.json"), spec_path],
                     capture_output=True, text=True)
if out.returncode:
    sys.exit(out.stderr)
print(out.stderr, file=sys.stderr)
done = json.loads(out.stdout)
bed = next(e for t in done["tracks"] for e in t["elements"] if e["id"] == "bed")
project["tracks"][-1]["elements"][0]["volume"] = bed["volume"]
timed = [r for t in done["tracks"] if t["name"] == "cap-tmp" for e in t["elements"] for r in e["runs"] if "highlight" in r]
starts = [r["highlight"]["start"] for r in timed]
ends = [r["highlight"]["end"] for r in timed]
words = [r["text"] for r in timed]
assert len(words) == 25, words

# Pages: phrases of two to five words.
PAGES = [3, 5, 2, 4, 3, 5, 3]
ACCENT = {"mouse.", "truth"}
HOLD = 500


def measure(texts):
    batch = [{"font": "caption", "size": SIZE, "runs": [{"text": t}]} for t in texts]
    r = subprocess.run(["montagent", "measure", os.path.join(HERE, "h2.json"), "--json", "--elements", json.dumps(batch)],
                       capture_output=True, text=True)
    res = json.loads(r.stdout)["measure"]["results"]
    return [(x["ok"]["advance_width"], x["ok"]["block_height"]) for x in res]


groups, k = [], 0
for n in PAGES:
    groups.append(list(range(k, k + n)))
    k += n
assert k == 25
texts = [" ".join(words[i] for i in g) for g in groups]
# Each word is its own element, placed at its offset in the finished line, so nothing shifts.
prefixes = [" ".join(words[i] for i in g[:j]) for g in groups for j in range(1, len(g))]
m = measure(texts + prefixes + ["a a", "a"])
space = m[-2][0] - 2 * m[-1][0]
sizes, pre = m[:len(texts)], {p: w for p, (w, _) in zip(prefixes, m[len(texts):-2])}
slots = [[] for _ in range(max(PAGES))]
pills = []
for n, (g, text, (w_px, h_px)) in enumerate(zip(groups, texts, sizes)):
    start = starts[g[0]]
    end = on_frame(ends[g[-1]] + HOLD)
    if n + 1 < len(groups):
        end = min(end, starts[groups[n + 1][0]])
    end = min(end, END)
    left = 540 - w_px / 2
    for j, i in enumerate(g):
        off = 0 if j == 0 else pre[" ".join(words[k] for k in g[:j])] + space
        ww = measure([words[i]])[0][0]
        slots[j].append({"id": f"cap-{n + 1:02d}-{j + 1}", "type": "text", "start": starts[i], "end": end,
                         "x": round(left + off), "y": CAP_Y, "origin": "center-left",
                         "width": math.ceil(ww) + 2, "height": math.ceil(h_px), "font": "caption", "size": SIZE,
                         "color": SIGNAL if words[i] in ACCENT else INK, "runs": [{"text": words[i]}]})
    pills.append({"id": f"cap-bg-{n + 1:02d}", "type": "rect", "start": start, "end": end,
                  "x": 540, "y": CAP_Y, "origin": "center", "width": math.ceil(w_px) + 64, "height": math.ceil(h_px) + 32,
                  "fill": "#FFFFFF", "radius": 26,
                  "effects": [{"name": "shadow", "dx": 0, "dy": 8, "radius": 24, "color": INK, "opacity": 0.18}]})
    print(f"page {n + 1}: {start}-{end} {text!r}", file=sys.stderr)

project["tracks"].insert(4, {"name": "captions-bg", "layer": 39, "elements": pills})
for j, els in enumerate(slots):
    project["tracks"].insert(5 + j, {"name": f"captions-{j + 1}", "layer": 40 + j, "elements": els})
json.dump(project, open(os.path.join(HERE, "h2.json"), "w"), indent=1)
subprocess.run(["montagent", "fmt", os.path.join(HERE, "h2.json")], capture_output=True)
