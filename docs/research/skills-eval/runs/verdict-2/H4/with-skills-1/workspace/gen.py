#!/usr/bin/env python3
"""Generate ad.montagent.json from the beat sheet below.

Beat sheet (120 BPM, beat = 500 ms, bar = 2000 ms, downbeat at 0):
  0      hook line 1 pops ("Your AI agent")
  500    hook line 2 pops ("edits video.")
  1000   accent on "video"
  2000   CUT  proof 1: the diff (session 33.02 s; diff lands at 2500)
  4500   CUT  proof 2: checks frames, "Rendering now." (session 40.0 s)
  6500   CUT  result on Paper: session-02, rendered video
  9000   CUT  claim on Ink
  9500   accent on "change"
  12000  claim moves down, accent clears
  12500  lockup pops; still from 12900 to 15000
  14000  music fade starts, 0 on the last frame
"""
import json

INK, PAPER, SIGNAL = "#101418", "#F5F0E6", "#FF5A36"
POP = [0.34, 1.56, 0.64, 1]
SETTLE = [0.16, 1, 0.3, 1]
MOVE = [0.65, 0, 0.35, 1]

T_P1, T_P2, T_RES, T_CLAIM, T_BRAND_MOVE, T_BRAND, END = 2000, 4500, 6500, 9000, 12000, 12500, 15000

proj = json.load(open("ad.montagent.json"))
proj["fonts"] = {"bold": [{"file": "fonts/Inter-Bold.ttf"}], "regular": [{"file": "fonts/Inter-Regular.ttf"}]}
tracks = []


def track(name, layer, *els):
    tracks.append({"name": name, "layer": layer, "elements": list(els)})


def text(id, start, end, x, y, size, runs, color=PAPER, w=980, h=None, font="bold", **kw):
    lines = sum(r["text"].count("\n") for r in runs) + 1
    el = {"id": id, "type": "text", "start": start, "end": end, "x": x, "y": y, "origin": "center",
          "width": w, "height": h or int(size * 1.2 * lines + 0.999), "font": font, "size": size,
          "color": color, "align": "center", "runs": runs}
    el.update(kw)
    el["caption"] = False
    return el


def pop_in(t, s0=0.7, fade=True, dur=350):
    out = {"scale": [{"t": t, "v": [s0, s0]}, {"t": t + dur, "v": [1.0, 1.0], "ease": POP}]}
    if fade:
        out["opacity"] = [{"t": t, "v": 0.0}, {"t": t + 150, "v": 1.0, "ease": "ease-out"}]
    return out


def card(id, start, end, source, src_w, src_h, s, crop, centre, k_keys, video=None):
    """A crop of a source, shown as a rounded card whose centre stays put while it scales.

    crop: (x, y, w, h) in source pixels. centre: card centre in frame px.
    k_keys: scale keyframes [(t, k, ease)]; x and y follow the same curve so the
    card's centre does not drift.
    """
    W, H = round(src_w * s), round(src_h * s)
    mx, my, mw, mh = (round(v * s) for v in crop)
    dx, dy = mx + mw / 2 - W / 2, my + mh / 2 - H / 2
    def keys(f):
        out = []
        for i, (t, k, e) in enumerate(k_keys):
            r = {"t": t, "v": f(k)}
            if i:
                r["ease"] = e
            out.append(r)
        return out if len({json.dumps(r["v"]) for r in out}) > 1 else out[0]["v"]
    el = {"id": id, "type": "video" if video else "image", "start": start, "end": end, "source": source}
    if video:
        el.update(video)
    el.update({"x": keys(lambda k: round(centre[0] - k * dx)), "y": keys(lambda k: round(centre[1] - k * dy)),
               "origin": "center", "width": W, "height": H, "fit": "contain",
               "scale": keys(lambda k: [k, k])})
    el["effects"] = [{"name": "mask", "shape": "rect", "x": mx, "y": my, "width": mw, "height": mh, "radius": 24}]
    return el, (mw, mh)


# ---- grounds -------------------------------------------------------------
track("ground", 1,
      {"id": "paper-ground", "type": "rect", "start": T_RES, "end": T_CLAIM, "x": 0, "y": 0, "origin": "top-left",
       "width": 1080, "height": 1080, "fill": PAPER})

# ---- hook ----------------------------------------------------------------
track("hook-1", 20, text("hook-1", 0, T_P1, 540, 470, 124, [{"text": "Your AI agent"}],
                         scale=[{"t": 0, "v": [0.8, 0.8]}, {"t": 350, "v": [1.0, 1.0], "ease": POP}]))
track("hook-2", 21, text("hook-2", 500, T_P1, 540, 620, 124,
                         [{"text": "edits "}, {"text": "video", "highlight": {"start": 1000, "end": T_P1, "color": SIGNAL}}, {"text": "."}],
                         **pop_in(500)))

# ---- proof ---------------------------------------------------------------
LABEL_Y, LABEL_SIZE = 150, 60
CARD_C, CARD_W, CARD_H = (540, 615), 936, 744
S_TERM = 1.2
crop_w, crop_h = CARD_W / S_TERM, CARD_H / S_TERM

p1, _ = card("proof-edit", T_P1, T_P2, "screen/session.mp4", 1920, 1080, S_TERM,
             (30, 172, crop_w, crop_h), CARD_C,
             [(T_P1, 1.0, None), (T_P2, 1.03, "linear")],
             video={"source_start": 33020, "source_end": 33020 + (T_P2 - T_P1)})
p2, _ = card("proof-check", T_P2, T_RES, "screen/session.mp4", 1920, 1080, S_TERM,
             (30, 128, crop_w, crop_h), CARD_C,
             [(T_P2, 1.0, None), (T_RES, 1.03, "linear")],
             video={"source_start": 40000, "source_end": 40000 + (T_RES - T_P2)})
track("proof", 10, p1, p2)
track("proof-label", 20,
      text("label-edit", T_P1, T_P2, 540, LABEL_Y, LABEL_SIZE, [{"text": "It edits the project file."}], **pop_in(T_P1)),
      text("label-check", T_P2, T_RES, 540, LABEL_Y, LABEL_SIZE, [{"text": "Checks the frames. Renders."}], **pop_in(T_P2)))

# ---- result --------------------------------------------------------------
S_RES = 1.5
res, _ = card("result", T_RES, T_CLAIM, "stills/session-02.png", 960, 540, S_RES,
              (168, 106, 624, 400), (540, 620),
              [(T_RES, 0.92, None), (T_RES + 400, 1.0, POP), (T_CLAIM, 1.03, "linear")])
res["opacity"] = [{"t": T_RES, "v": 0.0}, {"t": T_RES + 150, "v": 1.0, "ease": "ease-out"}]
res["effects"].append({"name": "shadow", "dx": 0, "dy": 24, "radius": 48, "color": INK, "opacity": 0.3})
track("result", 10, res)
track("result-label", 20,
      text("label-result", T_RES, T_CLAIM, 540, LABEL_Y, LABEL_SIZE, [{"text": "Out comes the video."}], color=INK, **pop_in(T_RES)))

# ---- claim + brand -------------------------------------------------------
claim = text("claim", T_CLAIM, END, 540, [{"t": T_BRAND_MOVE, "v": 540}, {"t": T_BRAND, "v": 730, "ease": MOVE}], 76,
             [{"text": "Video your agent can\nread, check and "},
              {"text": "change", "highlight": {"start": T_CLAIM + 500, "end": T_BRAND_MOVE, "color": SIGNAL}},
              {"text": "."}])
claim["scale"] = [{"t": T_CLAIM, "v": [0.7, 0.7]}, {"t": T_CLAIM + 350, "v": [1.0, 1.0], "ease": POP},
                  {"t": T_BRAND_MOVE, "v": [1.0, 1.0], "ease": "linear"}, {"t": T_BRAND, "v": [0.68, 0.68], "ease": MOVE}]
claim["opacity"] = [{"t": T_CLAIM, "v": 0.0}, {"t": T_CLAIM + 150, "v": 1.0, "ease": "ease-out"}]
track("claim", 20, claim)

LOCK_W, LOCK_H = 800, 185
lock = {"id": "lockup", "type": "image", "start": T_BRAND, "end": END, "source": "brand/lockup-on-dark.png",
        "x": 540, "y": 450, "origin": "center", "width": LOCK_W, "height": LOCK_H, "fit": "contain"}
lock.update(pop_in(T_BRAND, s0=0.7, dur=400))
track("brand", 22, lock)

# ---- music ---------------------------------------------------------------
track("music", 0, {"id": "bed", "type": "audio", "start": 0, "end": END, "source": "music/bed-120bpm.wav",
                   "source_start": 0, "source_end": END,
                   "volume": [{"t": 0, "v": 0.8}, {"t": 14000, "v": 0.8, "ease": "linear"},
                              {"t": 14966, "v": 0.0, "ease": "linear"}]})

proj["tracks"] = tracks
json.dump(proj, open("ad.montagent.json", "w"), indent=2)
