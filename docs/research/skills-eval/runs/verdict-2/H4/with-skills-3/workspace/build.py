"""Generate ad.montagent.json from the beat sheet. Re-run after any retime.

Beat sheet (120 BPM, beat = 500 ms, downbeat at 0):
  0     hook words pop, one per beat: Your / AI / agent / edits / video.(accent)
  3000  cut to Paper ground: "You ask." + terminal, prompt being typed
  4500  cut: "It edits the file." + terminal, diff lands at 5000
  7500  "It renders the video." + session-02 card pops
  9500  cut to Ink ground: claim pops; "read." turns Signal at 10500
  12500 cut to Paper ground: lockup pops, tagline at 12750, still from 13050
  14000 music fades to 0 on the last drawn frame
"""
import json, subprocess, sys

INK, PAPER, SIGNAL = "#101418", "#F5F0E6", "#FF5A36"
POP = [0.34, 1.56, 0.64, 1]
SETTLE = [0.16, 1, 0.3, 1]
LAST = 14966  # last drawn instant at 30 fps (measure --at 14999)

p = json.load(open("ad.montagent.json"))
p["tracks"] = []


def track(name, layer, *els):
    p["tracks"].append({"name": name, "layer": layer, "elements": list(els)})


def kf(*pairs):
    out = []
    for i, (t, v, *e) in enumerate(pairs):
        k = {"t": t, "v": v}
        if i:
            k["ease"] = e[0] if e else "linear"
        out.append(k)
    return out


def label(eid, text, start, end):
    return {"id": eid, "type": "text", "start": start, "end": end, "x": 540,
            "y": kf((start, 166), (start + 350, 150, SETTLE)), "origin": "center",
            "width": 900, "height": 87, "font": "bold", "size": 72, "color": INK,
            "align": "center", "runs": [{"text": text}],
            "opacity": kf((start, 0.0), (start + 250, 1.0, "ease-out")), "caption": False}


# Grounds
track("ground", 1,
      {"id": "paper-proof", "type": "rect", "start": 3000, "end": 9500, "x": 0, "y": 0,
       "origin": "top-left", "width": 1080, "height": 1080, "fill": PAPER},
      {"id": "paper-brand", "type": "rect", "start": 12500, "end": 15000, "x": 0, "y": 0,
       "origin": "top-left", "width": 1080, "height": 1080, "fill": PAPER})

# Terminal card: the session at 1.1x, seen through a rounded 972x720 window at (54, 260).
S = 1.1
W, H = round(1920 * S), round(1080 * S)
CARD_X, CARD_Y, CARD_W, CARD_H = 54, 260, 972, 720


def term(eid, start, end, src_start, src_end, sx, sy, speed=None, enter=False):
    x = CARD_X - round(sx * S)
    y = CARD_Y - round(sy * S)
    el = {"id": eid, "type": "video", "start": start, "end": end, "source": "screen/session.mp4",
          "source_start": src_start, "source_end": src_end, "x": x,
          "y": kf((start, y + 40), (start + 400, y, SETTLE)) if enter else y,
          "origin": "top-left", "width": W, "height": H, "fit": "contain"}
    if enter:
        el["opacity"] = kf((start, 0.0), (start + 300, 1.0, "ease-out"))
    if speed:
        el["speed"] = speed
    el["effects"] = [
        {"name": "mask", "shape": "rect", "x": CARD_X - x, "y": CARD_Y - y,
         "width": CARD_W, "height": CARD_H, "radius": 28},
        {"name": "shadow", "dx": 0, "dy": 24, "radius": 56, "color": INK, "opacity": 0.3},
    ]
    return el


track("term", 10,
      # typing "Add a subtitle under the greeting that fades in", 1.3x
      term("term-ask", 3000, 4500, 6300, 8250, 20, 10, speed=1.3, enter=True),
      # the agent's reasoning, then the diff scrolls in and settles at ~5000
      term("term-edit", 4500, 7500, 33100, 36100, 20, 70))

# Result: session-02 at 1.4x through a rounded 972x600 window, text block centred in it.
R = 1.4
RW, RH = round(960 * R), round(540 * R)
WIN_W, WIN_H = 972, 600
ly = round(305 * R) - WIN_H // 2  # window centred on the text block (source y ~305)
lx = (RW - WIN_W) // 2
track("result", 11,
      {"id": "result", "type": "image", "start": 7500, "end": 9500, "source": "stills/session-02.png",
       "x": 540, "y": 620 - (ly + WIN_H // 2 - RH // 2), "origin": "center", "width": RW, "height": RH,
       "fit": "contain",
       "scale": kf((7500, [0.8, 0.8]), (7950, [1.0, 1.0], POP)),
       "opacity": kf((7500, 0.0), (7650, 1.0, "ease-out")),
       "effects": [
           {"name": "mask", "shape": "rect", "x": lx, "y": ly, "width": WIN_W, "height": WIN_H, "radius": 28},
           {"name": "shadow", "dx": 0, "dy": 24, "radius": 56, "color": INK, "opacity": 0.3},
       ]})

track("label", 20,
      label("label-ask", "You ask.", 3000, 4500),
      label("label-edit", "It edits the file.", 4500, 7500),
      label("label-render", "It renders the video.", 7500, 9500))

# Claim
track("claim", 20,
      {"id": "claim", "type": "text", "start": 9500, "end": 12500, "x": 540,
       "y": kf((9500, 570), (9950, 540, POP)), "origin": "center", "width": 960, "height": 229,
       "font": "bold", "size": 104, "line_height": 1.1, "color": PAPER, "align": "center",
       "runs": [{"text": "Video your agent\ncan "},
                {"text": "read.", "highlight": {"start": 10500, "end": 12500, "color": SIGNAL}}],
       "scale": kf((9500, [0.7, 0.7]), (9950, [1.0, 1.0], POP)),
       "opacity": kf((9500, 0.0), (9650, 1.0, "ease-out")), "caption": False})

# Brand
LW, LH = 862, 199
track("brand", 20,
      {"id": "lockup", "type": "image", "start": 12500, "end": 15000, "source": "brand/lockup.png",
       "x": 540, "y": 500, "origin": "center", "width": LW, "height": LH, "fit": "contain",
       "scale": kf((12500, [0.7, 0.7]), (12950, [1.0, 1.0], POP)),
       "opacity": kf((12500, 0.0), (12650, 1.0, "ease-out"))})
track("tagline", 21,
      {"id": "tagline", "type": "text", "start": 12750, "end": 15000, "x": 540,
       "y": kf((12750, 690), (13050, 666, SETTLE)), "origin": "center", "width": 800, "height": 58,
       "font": "regular", "size": 48, "color": INK, "align": "center",
       "runs": [{"text": "The video editor AI agents drive."}],
       "opacity": kf((12750, 0.0), (13050, 1.0, "ease-out")), "caption": False})

# Music
track("music", 0,
      {"id": "bed", "type": "audio", "start": 0, "end": 15000, "source": "music/bed-120bpm.wav",
       "source_start": 0, "source_end": 15000,
       "volume": kf((14000, 0.8), (LAST, 0.0, "linear"))})

json.dump(p, open("ad.montagent.json", "w"), indent=2)

# Hook words, one per beat, set by the motion skill's script.
hook = [
    {"track": "hook1", "layer": 30, "text": "Your AI agent", "by": "word", "font": "bold", "size": 128,
     "color": PAPER, "x": 540, "y": 470, "align": "center", "times": [0, 500, 1000], "end": 3000,
     "enter": "pop", "pop": {"from": 0.6, "ms": 400, "rise": 30}},
    {"track": "hook2", "layer": 40, "text": "edits video.", "by": "word", "font": "bold", "size": 128,
     "color": PAPER, "x": 540, "y": 616, "align": "center", "times": [1500, 2000], "end": 3000,
     "enter": "pop", "pop": {"from": 0.6, "ms": 400, "rise": 30},
     "highlights": [{"unit": 2, "start": 2000, "color": SIGNAL}]},
]
for spec in hook:
    json.dump(spec, open("/tmp/claude-501/hookspec.json", "w"))
    out = subprocess.run([sys.executable, ".claude/skills/montagent-motion/scripts/type_on.py",
                          "ad.montagent.json", "/tmp/claude-501/hookspec.json"],
                         capture_output=True, text=True)
    if out.returncode:
        sys.exit(out.stderr)
    open("ad.montagent.json", "w").write(out.stdout)

subprocess.run(["montagent", "fmt", "ad.montagent.json"], check=True)
