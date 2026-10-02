"""Generate ad.json's tracks from the beat sheet. 120 BPM: a beat is 500 ms, a bar 2000 ms.

Beat sheet
  0      hook: "Your AI agent" (Ink ground)
  500    hook: "edits video."
  1000   accent on "video."
  2000   cut to Paper; terminal card in (real session, source 32.44 s)
  2250   label "It edits the project file."
  3000   the diff lands in the terminal (source 33.44 s)
  6000   cut: result card (session-02) + label "Then renders the result."
  8500   cut to Ink; claim line 1
  9000   claim line 2
  10000  accent on "check."
  12000  cut to Paper; lockup in
  12500  tagline in
  14000  music fade starts; 14966 last frame, silence
"""
import json, math, subprocess

P = "ad.json"
INK, PAPER, SIGNAL = "#101418", "#F5F0E6", "#FF5A36"
POP = [0.34, 1.56, 0.64, 1]
SETTLE = [0.16, 1, 0.3, 1]

proj = json.load(open(P))


def measure(specs):
    out = subprocess.run(["montagent", "measure", P, "--json", "--elements", json.dumps(specs)],
                         capture_output=True, text=True, check=True).stdout
    return [r["ok"] for r in json.loads(out)["measure"]["results"]]


def text(id, start, end, x, y, font, size, color, runs, **kw):
    return dict(id=id, type="text", start=start, end=end, x=x, y=y, origin="center",
                font=font, size=size, color=color, align="center", runs=runs, caption=False, **kw)


def pop(t, s0=0.7):
    return {"scale": [{"t": t, "v": [s0, s0]}, {"t": t + 400, "v": [1.0, 1.0], "ease": POP}]}


def rise(t, y, d=24):
    return {"y": [{"t": t, "v": y + d}, {"t": t + 300, "v": y, "ease": SETTLE}],
            "opacity": [{"t": t, "v": 0.0}, {"t": t + 300, "v": 1.0, "ease": "ease-out"}]}


texts = [
    # hook
    text("hook-1", 0, 2000, 540, 468, "bold", 120, PAPER, [{"text": "Your AI agent"}], **pop(0)),
    text("hook-2", 500, 2000, 540, 612, "bold", 120, PAPER,
         [{"text": "edits "}, {"text": "video.", "highlight": {"start": 1000, "end": 2000, "color": SIGNAL}}], **pop(500)),
    # proof / result labels
    text("label-proof", 2250, 6000, 540, 150, "bold", 64, INK, [{"text": "It edits the project file."}]),
    text("label-result", 6000, 8500, 540, 180, "bold", 64, INK, [{"text": "Then renders the result."}]),
    # claim
    text("claim-1", 8500, 12000, 540, 472, "bold", 104, PAPER, [{"text": "Video your agent"}], **pop(8500)),
    text("claim-2", 9000, 12000, 540, 608, "bold", 104, PAPER,
         [{"text": "can "}, {"text": "check.", "highlight": {"start": 10000, "end": 12000, "color": SIGNAL}}], **pop(9000)),
    # brand tagline
    text("tagline", 12500, 15000, 540, 650, "regular", 48, INK, [{"text": "The video editor AI agents drive."}]),
]
texts[2].update(rise(2250, 150))
texts[3].update(pop(6000, 0.9))
texts[6].update(rise(12500, 650))
# key order: measured box goes before font
for t, m in zip(texts, measure(texts)):
    assert m["advance_width"] <= 960, (t["id"], m["advance_width"])
    t["width"] = math.ceil(m["advance_width"]) + 24
    t["height"] = m["block_height"]

# terminal: session.mp4 at 0.85, a 920x650 window onto source x 12-1094, y 160-925
term = {"id": "terminal", "type": "video", "start": 2000, "end": 6000,
        "source": "screen/session.mp4", "source_start": 32440, "source_end": 36440,
        "x": 886, "y": 598, "origin": "center", "width": 1632, "height": 918, "fit": "contain",
        "scale": [{"t": 2000, "v": [0.94, 0.94]}, {"t": 2400, "v": [1.0, 1.0], "ease": SETTLE},
                  {"t": 6000, "v": [1.025, 1.025], "ease": "linear"}],
        "opacity": [{"t": 1900, "v": 0.0}, {"t": 2200, "v": 1.0, "ease": "ease-out"}],
        "effects": [{"name": "mask", "shape": "rect", "x": 10, "y": 136, "width": 920, "height": 650, "radius": 28},
                    {"name": "shadow", "dx": 0, "dy": 24, "radius": 48, "color": INK, "opacity": 0.3}]}

# result: session-02 at 1.3, a 920x518 (16:9) window around its text (source y 105-504)
result = {"id": "result", "type": "image", "start": 6000, "end": 8500, "source": "stills/session-02.png",
          "x": 540, "y": 560, "origin": "center", "width": 1248, "height": 702, "fit": "contain",
          "scale": [{"t": 6000, "v": [0.92, 0.92]}, {"t": 6400, "v": [1.0, 1.0], "ease": SETTLE},
                    {"t": 8500, "v": [1.02, 1.02], "ease": "linear"}],
          "opacity": [{"t": 5900, "v": 0.0}, {"t": 6200, "v": 1.0, "ease": "ease-out"}],
          "effects": [{"name": "mask", "shape": "rect", "x": 164, "y": 137, "width": 920, "height": 518, "radius": 28},
                      {"name": "shadow", "dx": 0, "dy": 24, "radius": 48, "color": INK, "opacity": 0.3}]}

lockup = {"id": "lockup", "type": "image", "start": 12000, "end": 15000, "source": "brand/lockup.png",
          "x": 540, "y": 480, "origin": "center", "width": 864, "height": 200, "fit": "contain",
          "scale": [{"t": 12000, "v": [0.9, 0.9]}, {"t": 12500, "v": [1.0, 1.0], "ease": SETTLE}],
          "opacity": [{"t": 11900, "v": 0.0}, {"t": 12300, "v": 1.0, "ease": "ease-out"}]}


def ground(id, start, end):
    return {"id": id, "type": "rect", "start": start, "end": end, "x": 0, "y": 0, "origin": "top-left",
            "width": 1080, "height": 1080, "fill": PAPER}


T = {t["id"]: t for t in texts}
proj["tracks"] = [
    {"name": "music", "layer": 0, "elements": [
        {"id": "bed", "type": "audio", "start": 0, "end": 15000, "source": "music/bed-120bpm.wav",
         "source_start": 0, "source_end": 15000,
         "volume": [{"t": 14000, "v": 0.8}, {"t": 14966, "v": 0.0, "ease": "linear"}]}]},
    {"name": "ground", "layer": 1, "elements": [ground("paper-1", 2000, 8500), ground("paper-2", 12000, 15000)]},
    {"name": "media", "layer": 10, "elements": [term, result, lockup]},
    {"name": "text-a", "layer": 20, "elements": [T["hook-1"], T["label-proof"], T["label-result"], T["claim-1"], T["tagline"]]},
    {"name": "text-b", "layer": 21, "elements": [T["hook-2"], T["claim-2"]]},
]
json.dump(proj, open(P, "w"), indent=2, ensure_ascii=False)
