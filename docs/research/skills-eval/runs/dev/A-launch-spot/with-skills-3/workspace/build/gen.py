"""Generate the launch spot from its beat sheet. Run from the project dir."""
import json, subprocess, sys

INK, PAPER, SIGNAL = "#101418", "#F5F0E6", "#FF5A36"
BEAT = 500
def b(n):  # nth beat from the 0 ms downbeat, on the 30 fps grid (120 BPM: whole frames)
    return n * BEAT

ENTER = [0.16, 1, 0.3, 1]
SWEEP = [0.65, 0, 0.35, 1]

# Beat sheet
HOOK_TIMES = [b(0), b(1), b(2), b(3), b(4)]   # words land 0, .5, 1, 1.5, 2
ACCENT_AT = b(4)                              # "read." turns Signal at 2.0
HOOK_EXIT = b(7)                              # clears 3.5-3.8
SHOW_IN = b(8)                                # 4.0 ground swap + shot 1
XFADE = (b(14), b(15))                        # 7.0-7.5
CHART_IN = b(18)                              # 9.0
LABELS_IN = b(18)                             # 9.5
BRAND_IN = b(20)                              # 10.0
WIPE_END = b(21)                              # 10.5
FADE_FROM = b(22)                             # 11.0
END = 12000
LAST_FRAME = 11966                            # frame 359 is drawn at 11966.667

def track(name, layer, *els):
    return {"name": name, "layer": layer, "elements": list(els)}

# One linear push over both stills, 4.0 -> 9.0
P0, P1, S0, S1 = SHOW_IN, CHART_IN, 1.0, 1.06
def push(t):
    return round(S0 + (S1 - S0) * (t - P0) / (P1 - P0), 4)

SW, SH = 1440, 810
frame_fx = [{"name": "mask", "shape": "rect", "radius": 28},
            {"name": "shadow", "dx": 0, "dy": 24, "radius": 56, "color": INK, "opacity": 0.3}]

def shot(id_, src, start, end, enter):
    el = {"id": id_, "type": "image", "start": start, "end": end, "source": src,
          "x": 960, "y": 540, "origin": "center", "width": SW, "height": SH, "fit": "contain",
          "scale": [{"t": start, "v": [push(start)] * 2}, {"t": end, "v": [push(end)] * 2, "ease": "linear"}],
          "effects": frame_fx}
    if enter:
        el["y"] = [{"t": start, "v": 590}, {"t": start + 500, "v": 540, "ease": ENTER}]
        el["opacity"] = [{"t": start, "v": 0.0}, {"t": start + 300, "v": 1.0, "ease": "ease-out"}]
    return el

shot1 = shot("shot-1", "stills/session-01.png", SHOW_IN, XFADE[1], True)
shot2 = shot("shot-2", "stills/session-02.png", XFADE[0], CHART_IN, False)

# Chart: baseline at y 700, three bars growing up from it
BASE_Y, BW = 700, 140
bars = [("Edit", 760, 150, INK, 300), ("Check", 960, 260, INK, 400), ("Render", 1160, 380, SIGNAL, 500)]
chart = [track("chart-base", 30, {"id": "baseline", "type": "rect", "start": CHART_IN, "end": BRAND_IN,
          "x": 960, "y": BASE_Y, "origin": "center", "width": 600, "height": 6, "fill": INK})]
for i, (label, x, h, fill, ms) in enumerate(bars, 1):
    chart.append(track(f"bar-{i}", 30 + i, {"id": f"bar-{i}", "type": "rect", "start": CHART_IN, "end": BRAND_IN,
        "x": x, "y": BASE_Y - 3, "origin": "bottom-center", "width": BW, "height": h, "fill": fill,
        "scale": [{"t": CHART_IN, "v": [1.0, 0.0]}, {"t": CHART_IN + ms, "v": [1.0, 1.0], "ease": ENTER}]}))
    chart.append(track(f"label-{i}", 40 + i, {"id": f"label-{i}", "type": "text", "start": LABELS_IN, "end": BRAND_IN,
        "x": x, "y": BASE_Y + 24, "origin": "top-center", "width": 240, "height": 56, "font": "body", "size": 44,
        "color": INK, "align": "center", "runs": [{"text": label}],
        "opacity": [{"t": LABELS_IN, "v": 0.0}, {"t": LABELS_IN + 200, "v": 1.0, "ease": "ease-out"}]}))

# Brand: wordmark uncovered by a Paper occluder shrinking to its right edge, Signal bar on the edge
WM_W, WM_H = 1000, 210
OCC_W, OCC_H, OCC_R = WM_W + 80, WM_H + 80, 960 + WM_W // 2 + 40
brand = [
    track("wordmark", 50, {"id": "wordmark", "type": "image", "start": BRAND_IN, "end": END,
        "source": "brand/wordmark.png", "x": 960, "y": 540, "origin": "center", "width": WM_W, "height": WM_H, "fit": "contain"}),
    track("occluder", 51, {"id": "occluder", "type": "rect", "start": BRAND_IN, "end": WIPE_END + 34,
        "x": OCC_R, "y": 540, "origin": "center-right", "width": OCC_W, "height": OCC_H, "fill": PAPER,
        "scale": [{"t": BRAND_IN, "v": [1.0, 1.0]}, {"t": WIPE_END, "v": [0.0, 1.0], "ease": SWEEP}]}),
    track("edge", 52, {"id": "edge", "type": "rect", "start": BRAND_IN, "end": WIPE_END + 234,
        "x": [{"t": BRAND_IN, "v": OCC_R - OCC_W}, {"t": WIPE_END, "v": OCC_R, "ease": SWEEP}],
        "y": 540, "origin": "center", "width": 16, "height": OCC_H + 24, "fill": SIGNAL,
        "opacity": [{"t": WIPE_END, "v": 1.0}, {"t": WIPE_END + 200, "v": 0.0, "ease": "ease-in"}]}),
]

project = {
    "frame": {"width": 1920, "height": 1080}, "fps": 30, "background": INK, "duration": END,
    "output": "deliverable.mp4",
    "fonts": {"title": [{"file": "fonts/Inter-Bold.ttf"}], "body": [{"file": "fonts/Inter-Regular.ttf"}]},
    "fontVendor": json.load(open("spot.montagent.json"))["fontVendor"],
    "tracks": [
        track("music", 0, {"id": "bed", "type": "audio", "start": 0, "end": END, "source": "music/bed-120bpm.wav",
            "source_start": 0, "source_end": END,
            "volume": [{"t": FADE_FROM, "v": 0.8}, {"t": LAST_FRAME, "v": 0.0, "ease": "linear"}]}),
        track("ground", 1, {"id": "paper", "type": "rect", "start": SHOW_IN, "end": END, "x": 0, "y": 0,
            "origin": "top-left", "width": 1920, "height": 1080, "fill": PAPER}),
        track("shot-1", 10, shot1),
        track("shot-2", 11, shot2),
        track("joins", 12, {"id": "join", "type": "transition", "start": XFADE[0], "end": XFADE[1],
            "kind": "crossfade", "from": "shot-1", "to": "shot-2"}),
        *chart, *brand,
    ],
}
json.dump(project, open("spot.base.json", "w"), indent=2)

spec = {"track": "hook", "layer": 20, "text": "Video your agent can read.", "by": "word", "font": "title",
        "size": 120, "color": PAPER, "x": 960, "y": 540, "align": "center", "times": HOOK_TIMES,
        "end": HOOK_EXIT + 334, "enter": "pop", "pop": {"from": 0.6, "ms": 400, "rise": 40},
        "highlights": [{"unit": 5, "start": ACCENT_AT, "color": SIGNAL}],
        "exit": {"at": HOOK_EXIT, "ms": 300}}
json.dump(spec, open("build/hook.spec.json", "w"), indent=2)
out = subprocess.run([sys.executable, ".claude/skills/montagent-motion/scripts/type_on.py",
                      "spot.base.json", "build/hook.spec.json"], capture_output=True, text=True)
if out.returncode:
    sys.exit(out.stderr)
sys.stderr.write(out.stderr)
open("spot.montagent.json", "w").write(out.stdout)
