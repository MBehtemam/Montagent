"""Build project.json for the Montagent logo loop from one beat sheet."""
import json, subprocess, sys, os

FPS = 30
def snap(t):  # nearest drawn frame, in ms
    return int(round(t * FPS / 1000) * 1000 // FPS)

INK, PAPER, SIGNAL = "#101418", "#F5F0E6", "#FF5A36"
W = H = 1080

# --- layout: the mark at 0.3 of its 1024 construction (tile 138, gap 12, radius 29) ---
S = 0.3
MARK = 288                               # visible extent: 2 tiles + gap
TEXT_SIZE = 104
TEXT_W = 5.33545 * TEXT_SIZE            # measured advance of "Montagent" at size 100, scaled
GAP = 50
CURSOR_TAIL = 8 + 8                      # cursor gap + width after the last letter
total = MARK + GAP + TEXT_W + CURSOR_TAIL
left = round((W - total) / 2)
top = H // 2 - MARK // 2
tile = round(460 * S)
def centre(col, row):
    return (left + col * 150 + tile // 2, top + row * 150 + tile // 2)
TEXT_X = left + MARK + GAP

# --- beat sheet (ms) ---
OVER = [0.34, 1.25, 0.64, 1]    # gentle overshoot settle
SPIN = [0.3, 0.3, 0.5, 1]       # max 44 deg/frame: no strobing on a square
FLY = 600
parts = [
    # id, kind, col, row, enter-from, start
    ("tile-tl", "rect", 0, 0, "left", 200),
    ("dot-tr", "ellipse", 1, 0, "top", 367),
    ("tile-bl", "rect", 0, 1, "bottom", 533),
    ("tile-br", "rect", 1, 1, "right", 700),   # the spinner, lands last at 2000
]
SPIN_END = 2000
CLEAR = {"tile-br": 4967, "tile-bl": 5067, "dot-tr": 5167, "tile-tl": 5267}   # last in, first out
CLEAR_MS = 300

CURSOR_FROM, CURSOR_TO = 2500, 4833
LETTER_OFFS = [0, 130, 290, 400, 560, 680, 830, 970, 1100]   # irregular, reads typed
TYPE_START = 3050
TEXT_EXIT = 4900

def kf(t, v, ease=None):
    d = {"t": snap(t), "v": v}
    if ease is not None:
        d["ease"] = ease
    return d

tracks = []
for i, (pid, kind, col, row, frm, t0) in enumerate(parts):
    cx, cy = centre(col, row)
    spinner = pid == "tile-br"
    t1 = SPIN_END if spinner else t0 + FLY
    ease = SPIN if spinner else OVER
    x, y = cx, cy
    if frm == "left":
        x = [kf(t0, -tile), kf(t1, cx, ease)]
    elif frm == "right":
        x = [kf(t0, W + tile), kf(t1, cx, ease)]
    elif frm == "top":
        y = [kf(t0, -tile), kf(t1, cy, ease)]
    elif frm == "bottom":
        y = [kf(t0, H + tile), kf(t1, cy, ease)]
    c = CLEAR[pid]
    el = {"id": pid, "type": kind, "start": snap(t0), "end": snap(c + CLEAR_MS + 34),
          "x": x, "y": y, "origin": "center", "width": tile, "height": tile,
          "fill": SIGNAL if kind == "ellipse" else INK}
    if kind == "rect":
        el["radius"] = round(96 * S)
    if spinner:
        el["rotation"] = [kf(t0, 0), kf(t1, 1080, SPIN)]
    el["scale"] = [kf(c, [1.0, 1.0]), kf(c + CLEAR_MS, [0.0, 0.0], "ease-in")]
    tracks.append({"name": pid, "layer": 10 + i, "elements": [el]})

proj = {
    "frame": {"width": W, "height": H}, "fps": FPS, "background": PAPER, "duration": 6000,
    "loop": True, "output": "deliverable.mp4",
    "fonts": {"title": [{"file": "fonts/Inter-Bold.ttf"}]},
    "fontVendor": json.load(open("project.json"))["fontVendor"],
    "tracks": tracks,
}
json.dump(proj, open("base.json", "w"), indent=2)

spec = {
    "track": "name", "layer": 30, "text": "Montagent", "by": "letter", "font": "title",
    "size": TEXT_SIZE, "color": INK, "x": TEXT_X, "y": H // 2, "align": "left",
    "times": [snap(TYPE_START + o) for o in LETTER_OFFS], "end": snap(TEXT_EXIT + 300 + 34),
    "enter": "cut", "exit": {"at": snap(TEXT_EXIT), "ms": 300},
    "cursor": {"from": snap(CURSOR_FROM), "to": snap(CURSOR_TO), "blink": 500, "width": 8,
               "color": INK, "height": 100, "gap": 8},
}
json.dump(spec, open("type.spec.json", "w"), indent=2)
here = os.path.dirname(os.path.abspath(__file__))
out = subprocess.run([sys.executable, os.path.join(here, ".claude/skills/montagent-motion/scripts/type_on.py"),
                      "base.json", "type.spec.json"], capture_output=True, text=True, check=True).stdout
open("project.json", "w").write(out)
print("left", left, "text_x", TEXT_X, "centres", [centre(c, r) for c in (0, 1) for r in (0, 1)])
