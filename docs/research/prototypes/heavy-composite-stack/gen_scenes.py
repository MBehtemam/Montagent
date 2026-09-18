#!/usr/bin/env python3
"""Generate scene-4k.json and scene-8k.json for #159's heavy-composite-stack
harness. Not part of run.sh -- one-off generator, kept for provenance.

The "heavy composite stack" ADR-0021 names, built as the direct plural of
#87's single-clip/single-still/single-overlay-block scene: TWO simultaneous
Ken Burns stills (main full-card + a PiP inset), TWO simultaneous video
clips (main full-card + a PiP inset), and TWO simultaneous overlay blocks
(translation-card style, the shape #87 measured once) all live in the same
[10, 20) window that run.sh benchmarks -- so #87's numbers and this
prototype's are read against the same window and the same budget.
"""
import json

REPO = "/Users/mohammedehtemam/projects/github/Montaget"
BASE = f"{REPO}/docs/research/prototypes/heavy-composite-stack"
BADGE = f"{REPO}/fixtures/en-halloween-decorating/brand/logo-en.png"

# One translation-card overlay block (rect x3 + text x2), the same shape
# #87 measured once. Positioned by (x0, y0) so two can be placed without
# overlapping; sized by `factor` relative to the 1080x1920 scene it was
# authored for (matches #87's convention).
def card_block(x0, y0, factor, start, end, english_label, translation_line, sub_lines):
    def sx(v):
        return round(v * factor, 2)

    return [
        {"kind": "rect", "layer": 0, "start": start, "end": end,
         "x": x0 + sx(48.0), "y": y0 + sx(88.0), "w": sx(372.0), "h": sx(84.0), "colour": "#FBF3E3"},
        {"kind": "rect", "layer": 1, "start": start, "end": end,
         "x": x0 + sx(82.0), "y": y0 + sx(103.0), "w": sx(78.0), "h": sx(54.0), "colour": "#00246B"},
        {"kind": "rect", "layer": 1, "start": start, "end": end,
         "x": x0 + sx(82.0), "y": y0 + sx(124.0), "w": sx(78.0), "h": sx(12.0), "colour": "#FFFFFF"},
        {"kind": "rect", "layer": 1, "start": start, "end": end,
         "x": x0 + sx(115.0), "y": y0 + sx(103.0), "w": sx(12.0), "h": sx(54.0), "colour": "#FFFFFF"},
        {"kind": "text", "layer": 2, "start": start, "end": end,
         "x": x0 + sx(182.0), "y": y0 + sx(130.0), "align": 4, "colour": "#245C8C",
         "size": sx(52.0), "font": "SF Pro Rounded", "bold": False, "lines": [english_label]},
        {"kind": "rect", "layer": 0, "start": start, "end": end,
         "x": x0 + sx(438.0), "y": y0 + sx(88.0), "w": sx(594.0), "h": sx(84.0), "colour": "#FBF3E3"},
        {"kind": "text", "layer": 2, "start": start, "end": end,
         "x": x0 + sx(560.0), "y": y0 + sx(130.0), "align": 4, "colour": "#245C8C",
         "size": sx(34.0), "font": "SF Pro Rounded", "bold": False, "lines": ["@FluencyInActionEnglish"]},
        {"kind": "text", "layer": 1, "start": start, "end": end,
         "x": x0 + sx(540.0), "y": y0 + sx(1373.0), "align": 5, "colour": "#245C8C",
         "size": sx(80.0), "font": "SF Pro Rounded", "bold": False, "lines": [translation_line]},
        {"kind": "rect", "layer": 0, "start": start + 5.15, "end": end,
         "x": x0 + sx(48.0), "y": y0 + sx(1453.0), "w": sx(984.0), "h": sx(169.0), "colour": "#1E344C"},
        {"kind": "text", "layer": 1, "start": start + 5.15, "end": end,
         "x": x0 + sx(540.0), "y": y0 + sx(1537.0), "align": 5, "colour": "#FFF8E8",
         "size": sx(57.0), "font": "SF Pro Rounded", "bold": False, "lines": sub_lines},
    ]


def make_scene(name, canvas_w, canvas_h, card_h, factor, a_video, a_still, b_video, b_still, b_still_px, duration=25.0, fps=30.0):
    # PiP inset geometry, same fraction of the card at either resolution:
    # top-right quadrant, roughly 1/3 of the card's width/height.
    inset_w = round(canvas_w / 3)
    inset_h = round(card_h / 3)
    inset_x = canvas_w - inset_w - round(60 * factor)
    inset_y_still = round(60 * factor)
    inset_y_clip = inset_y_still + inset_h + round(40 * factor)

    # Two overlay blocks live in the same window -- ADR-0021's "one overlay
    # block" made plural. They deliberately overlap in y (offset by a third
    # of the block's own height): this is a synthetic stress scene sized to
    # answer a raster-cost question, not a legible composition, and the two
    # sources are distinguishable in the committed correctness frames by hue.
    events = []
    events += card_block(0, 0, factor, 5.0, 17.16, "English", "skeleton  -  skeleton",
                          ["I hang the skeleton", "up by the door."])
    events += card_block(0, round(500 * factor), factor, 5.0, 17.16, "Deutsch", "Skelett  -  Skelett",
                          ["Ich hänge das Skelett", "an die Tür."])

    scene = {
        "width": canvas_w,
        "height": canvas_h,
        "fps": fps,
        "duration": duration,
        "cardH": card_h,
        "background": "#FBF3E3",
        "spans": [
            {
                "image": a_still,
                "start": 0.0,
                "end": duration,
                "cropW": canvas_w,
                "cropH": card_h,
                "zoomFrom": 1.0001,
                "zoomTo": 1.12,
                "zoomStep": 0.0009,
            },
            {
                "image": b_still,
                "start": 0.0,
                "end": duration,
                "cropW": b_still_px,
                "cropH": b_still_px,
                "zoomFrom": 1.0001,
                "zoomTo": 1.15,
                "zoomStep": 0.0012,
                "dx": inset_x,
                "dy": inset_y_still,
                "dw": inset_w,
                "dh": inset_h,
            },
        ],
        "badge": {"src": BADGE, "x": 470 * factor, "y": 104 * factor, "w": 64 * factor, "h": 64 * factor},
        "events": events,
        "audio": [],
        "clips": [
            {
                "src": a_video,
                "start": 5.0,
                "end": 25.0,
                "srcStart": 5.0,
                "dx": 0.0,
                "dy": 0.0,
                "dw": canvas_w,
                "dh": card_h,
            },
            {
                "src": b_video,
                "start": 5.0,
                "end": 25.0,
                "srcStart": 0.0,
                "dx": inset_x,
                "dy": inset_y_clip,
                "dw": inset_w,
                "dh": inset_h,
            },
        ],
    }
    out = f"{BASE}/{name}"
    with open(out, "w") as f:
        json.dump(scene, f, indent=1)
    print(f"wrote {out}: {len(scene['spans'])} spans, {len(scene['clips'])} clips, {len(scene['events'])} events")


make_scene(
    "scene-4k.json", 2160, 3840, 2600, 1.0,
    f"{BASE}/media/a-4k-video.mp4", f"{BASE}/media/a-4k-still.png",
    f"{BASE}/media/b-4k-video.mp4", f"{BASE}/media/b-4k-still.png", 800,
)
make_scene(
    "scene-8k.json", 4320, 7680, 5200, 2.0,
    f"{BASE}/media/a-8k-video.mp4", f"{BASE}/media/a-8k-still.png",
    f"{BASE}/media/b-8k-video.mp4", f"{BASE}/media/b-8k-still.png", 1600,
)
