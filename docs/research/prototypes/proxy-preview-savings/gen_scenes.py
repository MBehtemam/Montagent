#!/usr/bin/env python3
"""Generate scene-4k.json and scene-8k.json for #87's proxy-preview-savings harness.
Not part of run.sh -- one-off generator, kept for provenance. Re-run only if the
scene design changes.
"""
import json, os

REPO = "/Users/mohammedehtemam/projects/github/Montagent"
BASE = f"{REPO}/docs/research/prototypes/proxy-preview-savings"
BADGE = f"{REPO}/fixtures/en-halloween-decorating/brand/logo-en.png"

# events lifted from rust-rasterizer/scene-video.json's t=[30.6,42.76] block
# (title-card-style badges + a big bottom caption), times shifted to our
# window and coordinates/sizes scaled by `factor` relative to the 1080x1920
# scene they were authored for.
RAW_EVENTS = [
    {"kind": "rect", "layer": 0, "start": 30.6, "end": 42.76, "x": 48.0, "y": 88.0, "w": 372.0, "h": 84.0, "colour": "#FBF3E3"},
    {"kind": "rect", "layer": 1, "start": 30.6, "end": 42.76, "x": 82.0, "y": 103.0, "w": 78.0, "h": 54.0, "colour": "#00246B"},
    {"kind": "rect", "layer": 1, "start": 30.6, "end": 42.76, "x": 82.0, "y": 124.0, "w": 78.0, "h": 12.0, "colour": "#FFFFFF"},
    {"kind": "rect", "layer": 1, "start": 30.6, "end": 42.76, "x": 115.0, "y": 103.0, "w": 12.0, "h": 54.0, "colour": "#FFFFFF"},
    {"kind": "text", "layer": 2, "start": 30.6, "end": 42.76, "x": 182.0, "y": 130.0, "align": 4, "colour": "#245C8C", "size": 52.0, "font": "SF Pro Rounded", "bold": False, "lines": ["English"]},
    {"kind": "rect", "layer": 0, "start": 30.6, "end": 42.76, "x": 438.0, "y": 88.0, "w": 594.0, "h": 84.0, "colour": "#FBF3E3"},
    {"kind": "text", "layer": 2, "start": 30.6, "end": 42.76, "x": 560.0, "y": 130.0, "align": 4, "colour": "#245C8C", "size": 34.0, "font": "SF Pro Rounded", "bold": False, "lines": ["@FluencyInActionEnglish"]},
    {"kind": "text", "layer": 1, "start": 30.6, "end": 42.76, "x": 540.0, "y": 1373.0, "align": 5, "colour": "#245C8C", "size": 80.0, "font": "SF Pro Rounded", "bold": False, "lines": ["skeleton  -  skeleton"]},
    {"kind": "rect", "layer": 0, "start": 35.75, "end": 42.76, "x": 48.0, "y": 1453.0, "w": 984.0, "h": 169.0, "colour": "#1E344C"},
    {"kind": "text", "layer": 1, "start": 35.75, "end": 42.76, "x": 540.0, "y": 1537.0, "align": 5, "colour": "#FFF8E8", "size": 57.0, "font": "SF Pro Rounded", "bold": False, "lines": ["I hang the skeleton", "up by the door."]},
]
TIME_SHIFT = -30.6 + 5.0  # move the 30.6..42.76 block to start at t=5


def make_scene(name, canvas_w, canvas_h, card_h, factor, video_src, still_src, duration=25.0, fps=30.0):
    events = []
    for e in RAW_EVENTS:
        ne = dict(e)
        ne["start"] = round(e["start"] + TIME_SHIFT, 3)
        ne["end"] = round(e["end"] + TIME_SHIFT, 3)
        for k in ("x", "y", "w", "h", "size"):
            if k in ne:
                ne[k] = ne[k] * factor
        events.append(ne)

    scene = {
        "width": canvas_w,
        "height": canvas_h,
        "fps": fps,
        "duration": duration,
        "cardH": card_h,
        "background": "#FBF3E3",
        "spans": [
            {
                "image": still_src,
                "start": 0.0,
                "end": duration,
                "frames": int(duration * fps),
                "cropW": canvas_w,
                "cropH": card_h,
                "zoomFrom": 1.0001,
                "zoomTo": 1.12,
                "zoomStep": 0.0009,
            }
        ],
        "badge": {"src": BADGE, "x": 470 * factor, "y": 104 * factor, "w": 64 * factor, "h": 64 * factor},
        "events": events,
        "audio": [],
        "clips": [
            {
                "src": video_src,
                "start": 5.0,
                "end": 25.0,
                "srcStart": 5.0,
                "dx": 0.0,
                "dy": 0.0,
                "dw": canvas_w,
                "dh": card_h,
            }
        ],
    }
    out = f"{BASE}/{name}"
    with open(out, "w") as f:
        json.dump(scene, f, indent=1)
    print("wrote", out)


make_scene(
    "scene-4k.json",
    canvas_w=2160, canvas_h=3840, card_h=2600, factor=2.0,
    video_src=f"{BASE}/media/4k-video.mp4",
    still_src=f"{BASE}/media/4k-still.png",
)
make_scene(
    "scene-8k.json",
    canvas_w=4320, canvas_h=7680, card_h=5200, factor=4.0,
    video_src=f"{BASE}/media/8k-video.mp4",
    still_src=f"{BASE}/media/8k-still.png",
)
