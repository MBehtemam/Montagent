"""Writes hoot.base.json (everything but the owl); bake_rig.py adds the owl."""
import json

INK, PAPER, SIGNAL, TERM = "#101418", "#F5F0E6", "#FF5A36", "#1D1D2D"
POP = [0.34, 1.56, 0.64, 1]
END = 10000

VOICE_AT = 1300            # the line starts here
VOICE_LEN = 5520           # probed length of line-2.wav

# Answer beats (voice word starts + VOICE_AT)
LAPTOP_IN = (3300, 3600)   # "Just"
WIPE = (3700, 4300)        # "edit the file"
CODE_MARK = 4400           # file highlighted as "file" lands
CARD_IN = (4900, 5400)     # "Montagent" -> arrives before "renders"
PROPS_OUT = (6950, 7200)   # after the line ends (6820)
LOCKUP_IN = (7150, 7500)
TAG_IN = (7600, 7900)

# Laptop: 708x566 at 0.95, bottom on the floor
LX, LBOT, LS = 1020, 975, 0.95
LW, LH = 673, 538
L_LEFT, L_TOP = LX - LW // 2, LBOT - LH
SCR_X, SCR_Y = L_LEFT + round(120 * LS), L_TOP + round(41 * LS)
SCR_W, SCR_H = 444, 257
# session-01 crop: source (110, 295) 700x405 -> screen, k = 1218/1920
K = 1218 / 1920
CROP_X, CROP_Y = 110, 295
IMG_X, IMG_Y = SCR_X - round(CROP_X * K), SCR_Y - round(CROP_Y * K)

# Rendered-video card
CX, CY, CW, CH = 1600, 265, 480, 270


def kf(t, v, ease=None):
    k = {"t": t, "v": v}
    if ease is not None:
        k["ease"] = ease
    return k


def enter_exit(y0, start, end, dy=70):
    """Shared props move: rise in on an overshoot, sink out on ease-in."""
    a, b = LAPTOP_IN if start == LAPTOP_IN[0] else (start, start + 300)
    return {
        "y": [kf(a, y0 + dy), kf(b, y0, POP), kf(PROPS_OUT[0], y0, "linear"), kf(PROPS_OUT[1], y0 + 40, "ease-in")],
        "opacity": [kf(a, 0.0), kf(a + 200, 1.0, "ease-out"), kf(PROPS_OUT[0], 1.0, "linear"), kf(PROPS_OUT[1], 0.0, "ease-in")],
    }


def laptop_group():
    s, e = LAPTOP_IN[0], PROPS_OUT[1] + 100
    els = []
    lap = {"id": "laptop", "type": "image", "start": s, "end": e, "source": "character/laptop.png",
           "x": L_LEFT, "origin": "top-left", "width": LW, "height": LH, "fit": "contain"}
    lap.update(enter_exit(L_TOP, s, e))
    lap["effects"] = [{"name": "shadow", "dx": 0, "dy": 10, "radius": 18, "color": INK, "opacity": 0.25}]
    els.append(("laptop", 2, lap))

    code = {"id": "code", "type": "image", "start": s, "end": e, "source": "stills/session-01.png",
            "x": IMG_X, "origin": "top-left", "width": 1218, "height": 685, "fit": "contain"}
    code.update(enter_exit(IMG_Y, s, e))
    code["effects"] = [{"name": "mask", "shape": "rect", "x": round(CROP_X * K), "y": round(CROP_Y * K),
                        "width": SCR_W, "height": SCR_H}]
    els.append(("code", 3, code))

    # Covers the added (+) lines, then shrinks to the right: the edit typed in.
    cy0, cy1 = SCR_Y + round((382 - CROP_Y) * K), SCR_Y + round((640 - CROP_Y) * K)
    cover = {"id": "edit-cover", "type": "rect", "start": s, "end": WIPE[1] + 100,
             "x": SCR_X + SCR_W, "origin": "top-right", "width": SCR_W, "height": cy1 - cy0, "fill": TERM,
             "scale": [kf(WIPE[0], [1.0, 1.0]), kf(WIPE[1], [0.0, 1.0], "ease-in-out")]}
    cover.update(enter_exit(cy0, s, e))
    els.append(("edit-cover", 4, cover))
    return els


def card_group():
    s, e = CARD_IN[0], PROPS_OUT[1] + 100
    sx, sy = SCR_X + SCR_W // 2, SCR_Y + SCR_H // 2

    def fly(el, x1, y1):
        el["x"] = [kf(s, sx), kf(CARD_IN[1], x1, "ease-out")]
        el["y"] = [kf(s, sy), kf(CARD_IN[1], y1, "ease-out"), kf(PROPS_OUT[0], y1, "linear"), kf(PROPS_OUT[1], y1 + 40, "ease-in")]
        el["scale"] = [kf(s, [0.2, 0.2]), kf(CARD_IN[1], [1.0, 1.0], POP)]
        el["opacity"] = [kf(s, 0.0), kf(s + 200, 1.0, "ease-out"), kf(PROPS_OUT[0], 1.0, "linear"), kf(PROPS_OUT[1], 0.0, "ease-in")]
        return el

    frame = fly({"id": "card-frame", "type": "rect", "start": s, "end": e, "origin": "center",
                 "width": CW + 24, "height": CH + 24, "fill": INK, "radius": 14,
                 "effects": [{"name": "shadow", "dx": 0, "dy": 10, "radius": 22, "color": INK, "opacity": 0.3}]}, CX, CY)
    pic = fly({"id": "card", "type": "image", "start": s, "end": e, "source": "stills/session-02.png",
               "origin": "center", "width": CW, "height": CH, "fit": "contain"}, CX, CY)
    return [("card-frame", 5, frame), ("card", 6, pic)]


def label(id_, text, x, y, start, w, h=56, size=32, fill=INK, color=PAPER):
    a, b = start, start + 300
    common = {"start": a, "end": PROPS_OUT[1] + 100}
    pill = {"id": id_ + "-pill", "type": "rect", **common, "origin": "center", "width": w, "height": h,
            "fill": fill, "radius": h // 2}
    txt = {"id": id_, "type": "text", **common, "origin": "center", "width": w, "height": h,
           "font": "label", "size": size, "color": color, "align": "center", "runs": [{"text": text}],
           "caption": False}
    for el in (pill, txt):
        el["x"] = x
        el["y"] = [kf(a, y + 30), kf(b, y, POP), kf(PROPS_OUT[0], y, "linear"), kf(PROPS_OUT[1], y + 40, "ease-in")]
        el["opacity"] = [kf(a, 0.0), kf(a + 200, 1.0, "ease-out"), kf(PROPS_OUT[0], 1.0, "linear"), kf(PROPS_OUT[1], 0.0, "ease-in")]
    return [(id_ + "-pill", 7, pill), (id_, 8, txt)]


def marks():
    """Signal outlines: the edited line in the file, and the same words in the video."""
    out = []
    for id_, x, y, w, h, a in (("code-mark", 1052, 666, 196, 32, CODE_MARK), ("card-mark", 1600, 309, 166, 30, CARD_IN[1] + 100)):
        el = {"id": id_, "type": "rect", "start": a, "end": PROPS_OUT[1] + 100, "x": x, "origin": "center",
              "width": w, "height": h, "stroke": SIGNAL, "stroke_width": 3, "radius": 6,
              "y": [kf(a, y), kf(PROPS_OUT[0], y, "linear"), kf(PROPS_OUT[1], y + 40, "ease-in")],
              "scale": [kf(a, [1.4, 1.4]), kf(a + 200, [1.0, 1.0], "ease-out")],
              "opacity": [kf(a, 0.0), kf(a + 100, 1.0, "ease-out"), kf(PROPS_OUT[0], 1.0, "linear"), kf(PROPS_OUT[1], 0.0, "ease-in")]}
        out.append((id_, 9, el))
    return out


def arrow():
    a, b = CARD_IN[0] + 200, CARD_IN[1]
    el = {"id": "arrow", "type": "text", "start": a, "end": PROPS_OUT[1] + 100, "x": 1430, "origin": "center",
          "width": 120, "height": 120, "font": "label", "size": 110, "color": SIGNAL, "align": "center",
          "runs": [{"text": "↗"}],
          "y": [kf(a, 470), kf(b, 450, "ease-out"), kf(PROPS_OUT[0], 450, "linear"), kf(PROPS_OUT[1], 490, "ease-in")],
          "scale": [kf(a, [0.3, 0.3]), kf(b, [1.0, 1.0], POP)],
          "opacity": [kf(a, 0.0), kf(a + 200, 1.0, "ease-out"), kf(PROPS_OUT[0], 1.0, "linear"), kf(PROPS_OUT[1], 0.0, "ease-in")],
          "caption": False}
    return [("arrow", 8, el)]


def brand():
    a, b = LOCKUP_IN
    lock = {"id": "lockup", "type": "image", "start": a, "end": END, "source": "brand/lockup.png",
            "x": 1140, "y": 440, "origin": "center", "width": 860, "height": 199, "fit": "contain",
            "scale": [kf(a, [0.6, 0.6]), kf(b, [1.0, 1.0], POP)],
            "opacity": [kf(a, 0.0), kf(a + 200, 1.0, "ease-out")]}
    ta, tb = TAG_IN
    tag = {"id": "tagline", "type": "text", "start": ta, "end": END, "x": 1140, "origin": "center",
           "width": 1000, "height": 70, "font": "label", "size": 52, "color": INK, "align": "center",
           "runs": [{"text": "Edit the file. "}, {"text": "Render again.", "highlight": {"start": ta, "end": END, "color": SIGNAL}}],
           "y": [kf(ta, 640), kf(tb, 620, "ease-out")],
           "opacity": [kf(ta, 0.0), kf(tb, 1.0, "ease-out")], "caption": False}
    return [("lockup", 30, lock), ("tagline", 31, tag)]


def audio():
    voice = {"id": "voice", "type": "audio", "start": VOICE_AT, "end": VOICE_AT + VOICE_LEN,
             "source": "character/voice/line-2.wav", "source_start": 0, "source_end": VOICE_LEN, "volume": 1.0}
    bed_end = 9600
    bed = {"id": "bed", "type": "audio", "start": 0, "end": bed_end + 100, "source": "music/bed-120bpm.wav",
           "source_start": 0, "source_end": bed_end + 100,
           "volume": [kf(0, 0.0), kf(500, 0.16, "ease-out"), kf(1100, 0.16, "linear"), kf(1300, 0.07, "ease-in-out"),
                      kf(6900, 0.07, "linear"), kf(7300, 0.16, "ease-in-out"), kf(8600, 0.16, "linear"),
                      kf(bed_end, 0.0, "ease-in")]}
    return [("voice", 0, voice), ("bed", 0, bed)]


def main():
    tracks = [{"name": "set", "layer": 0, "elements": [
        {"id": "study", "type": "image", "start": 0, "end": END, "source": "character/study.png",
         "x": 0, "y": 0, "origin": "top-left", "width": 1920, "height": 1080, "fit": "contain"}]}]
    groups = laptop_group() + card_group() + label("file-label", "the file", LX, L_TOP - 50, LAPTOP_IN[0] + 100, 190) \
        + label("video-label", "the video", CX, CY - CH // 2 - 50, CARD_IN[1] - 100, 220) + arrow() + marks() + brand()
    for name, layer, el in groups:
        tracks.append({"name": name, "layer": layer, "elements": [el]})
    for name, _, el in audio():
        tracks.append({"name": name, "layer": 0, "elements": [el]})
    proj = {
        "frame": {"width": 1920, "height": 1080}, "fps": 30, "background": PAPER, "duration": END,
        "output": "deliverable.mp4",
        "fonts": {"label": [{"file": "fonts/Inter-Bold.ttf"}]},
        "fontVendor": {"fonts/Inter-Bold.ttf": {"licence": "OFL-1.1", "source": "https://github.com/rsms/inter",
                                                "sha256": "288316099b1e0a47a4716d159098005eef7c0066921f34e3200393dbdb01947f"}},
        "tracks": tracks,
    }
    json.dump(proj, open("hoot.base.json", "w"), indent=2, ensure_ascii=False)


main()
