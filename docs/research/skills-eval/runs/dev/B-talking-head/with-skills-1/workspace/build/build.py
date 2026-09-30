"""Write the base project (presenter, name bar, picture-in-picture, music).

Captions and the music duck are added afterwards by the footage skill's captions.py.
"""
import json
import pathlib

ROOT = pathlib.Path(__file__).resolve().parent.parent
INK, PAPER, SIGNAL = "#101418", "#F5F0E6", "#FF5A36"
POP = [0.34, 1.56, 0.64, 1]
IO = [0.65, 0, 0.35, 1]

# Presenter: 1920x1080 source scaled 1.9x, shifted left so the picture-in-picture has room.
PS = 1.9
P_W, P_H = round(1920 * PS), round(1080 * PS)
P_TOP = 150
P_CX = 477

presenter = {
    "id": "presenter", "type": "video", "start": 0, "end": 10000,
    "source": "presenter/take-1.mp4", "source_start": 0, "source_end": 9760,
    "x": P_CX, "y": P_TOP + P_H // 2, "origin": "center", "width": P_W, "height": P_H, "fit": "cover",
    "overrun": "hold", "volume": 1.0,
    "effects": [{"name": "chroma", "color": "#00FF00", "tolerance": 0.24, "softness": 0.1, "spill": 1.0}],
}

# Name bar: grows from its left edge at 1 s, text uncovered by a bar-coloured occluder;
# off from 6 s: cover grows back, then the bar shrinks to its edge.
BX, BY, BW, BH = 60, 1150, 540, 150
bar = {"id": "bar", "type": "rect", "start": 1000, "end": 6467, "x": BX, "y": BY, "origin": "center-left",
       "width": BW, "height": BH, "fill": INK, "radius": 8,
       "scale": [{"t": 1000, "v": [0.0, 1.0]}, {"t": 1300, "v": [1.0, 1.0], "ease": IO},
                 {"t": 6267, "v": [1.0, 1.0], "ease": "linear"}, {"t": 6466, "v": [0.0, 1.0], "ease": IO}]}
bar_edge = {"id": "bar-edge", "type": "rect", "start": 1000, "end": 6467, "x": BX, "y": BY, "origin": "center-left",
            "width": 12, "height": BH, "fill": PAPER}
bar_name = {"id": "bar-name", "type": "text", "start": 1300, "end": 6267, "x": BX + 44, "y": BY - 30,
            "origin": "center-left", "width": 460, "height": 70, "font": "bold", "size": 58, "color": PAPER,
            "runs": [{"text": "Montagent"}]}
bar_line = {"id": "bar-line", "type": "text", "start": 1300, "end": 6267, "x": BX + 44, "y": BY + 38,
            "origin": "center-left", "width": 460, "height": 42, "font": "bold", "size": 34, "color": PAPER,
            "runs": [{"text": "Video your agent can read"}]}
CV_L, CV_R = BX + 20, BX + BW - 20
cover_on = {"id": "bar-cover-on", "type": "rect", "start": 1300, "end": 1667, "x": CV_R, "y": BY,
            "origin": "center-right", "width": CV_R - CV_L, "height": BH - 20, "fill": INK,
            "scale": [{"t": 1300, "v": [1.0, 1.0]}, {"t": 1633, "v": [0.0, 1.0], "ease": IO}]}
cover_off = {"id": "bar-cover-off", "type": "rect", "start": 6000, "end": 6267, "x": CV_L, "y": BY,
             "origin": "center-left", "width": CV_R - CV_L, "height": BH - 20, "fill": INK,
             "scale": [{"t": 6000, "v": [0.0, 1.0]}, {"t": 6234, "v": [1.0, 1.0], "ease": IO}]}

# Picture-in-picture: the session at 1:1, masked to a 400 px circle round source point
# (400, 510), where the agent's diff of the project file lands. The element's centre is
# keyframed with the same ease as its scale, so the circle pops in place about its own centre.
C = (824, 960)
D = 400
SRC_C = (400, 510)
OFF = (SRC_C[0] - 960, SRC_C[1] - 540)
PIP_IN, PIP_PEAK, PIP_HOLD, PIP_OUT = 3400, 3800, 9500, 9733
PIP_END = 9734


def keyed(v0, v1):
    return [{"t": PIP_IN, "v": v0}, {"t": PIP_PEAK, "v": v1, "ease": POP},
            {"t": PIP_HOLD, "v": v1, "ease": "linear"}, {"t": PIP_OUT, "v": v0, "ease": "ease-in"}]


pip = {"id": "pip", "type": "video", "start": PIP_IN, "end": PIP_END, "source": "screen/session.mp4",
       "source_start": 32400, "source_end": 32400 + PIP_END - PIP_IN,
       "x": keyed(C[0], C[0] - OFF[0]), "y": keyed(C[1], C[1] - OFF[1]), "origin": "center",
       "width": 1920, "height": 1080, "fit": "cover", "scale": keyed([0.0, 0.0], [1.0, 1.0]),
       "effects": [{"name": "mask", "shape": "circle", "x": SRC_C[0] - D // 2, "y": SRC_C[1] - D // 2,
                    "width": D, "height": D}]}
ring = {"id": "pip-ring", "type": "ellipse", "start": PIP_IN, "end": PIP_END, "x": C[0], "y": C[1],
        "origin": "center", "width": D + 20, "height": D + 20, "fill": PAPER,
        "scale": keyed([0.0, 0.0], [1.0, 1.0]),
        "effects": [{"name": "shadow", "dx": 0, "dy": 16, "radius": 40, "color": INK, "opacity": 0.35}]}

music = {"id": "bed", "type": "audio", "start": 0, "end": 10000, "source": "music/bed-120bpm.wav",
         "source_start": 0, "source_end": 10000, "volume": 0.5}


def track(name, layer, *els):
    return {"name": name, "layer": layer, "elements": list(els)}


project = {
    "frame": {"width": 1080, "height": 1920}, "fps": 30, "background": SIGNAL, "duration": 10000,
    "output": "deliverable.mp4",
    "fonts": {"bold": [{"file": "fonts/Inter-Bold.ttf"}]},
    "fontVendor": json.load(open(ROOT / "social.montagent.json"))["fontVendor"],
    "tracks": [
        track("music", 0, music),
        track("presenter", 10, presenter),
        track("pip-ring", 20, ring),
        track("pip", 21, pip),
        track("bar", 40, bar),
        track("bar-edge", 41, bar_edge),
        track("bar-name", 42, bar_name),
        track("bar-line", 42, bar_line),
        track("bar-cover", 43, cover_on, cover_off),
    ],
}
json.dump(project, open(ROOT / "social.montagent.json", "w"), indent=2)
