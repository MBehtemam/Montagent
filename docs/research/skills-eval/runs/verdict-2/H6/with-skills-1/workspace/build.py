"""Build hoot.montagent.json: the set, props, brand and sound, then bake the owl from owl.spec.json."""
import json
import subprocess
import sys

BEZ = [0.34, 1.56, 0.64, 1]
VOICE_AT = 1200          # voice starts on frame 36
END = 10000

# Laptop on the floor right of the owl, scale 1.0 (708x566), top-left (800, 392).
LAP_X, LAP_Y = 860, 392
SCR_CX, SCR_CY, SCR_W, SCR_H = LAP_X + 354, LAP_Y + 177, 467, 271   # screen centre (120..587, 41..312)


def snap(t):
    """The drawn instant nearest t, in whole ms."""
    return int(round(t * 30 / 1000) * 1000 / 30)


LAP_IN, LAP_LAND = snap(3225), snap(3525)          # pops on "Just"
DIFF_IN, DIFF_DONE = snap(3566), snap(3866)        # the edit writes itself in on "edit"
BAR_IN, BAR_DONE = snap(4766), snap(5400)          # render progress over "Montagent"
CARD_IN, CARD_LAND = snap(5400), snap(5766)        # the rendered video flies out on "renders"
OUT_A, OUT_B = snap(6900), snap(7166)              # props leave
GONE = snap(OUT_B + 33)                              # props' end: the frame after they reach 0
BRAND_IN, BRAND_LAND = snap(7100), snap(7400)      # lockup lands

CARD_CX, CARD_CY = SCR_CX, 200
CARD_W, CARD_H = 448, 252


def kf(pairs, ease="ease-out"):
    out = []
    for i, (t, v) in enumerate(pairs):
        k = {"t": t, "v": v}
        if i:
            k["ease"] = ease if not isinstance(v, tuple) else ease
        out.append(k)
    return out


def keys(*recs):
    out = []
    for i, r in enumerate(recs):
        k = {"t": r[0], "v": r[1]}
        if i:
            k["ease"] = r[2]
        out.append(k)
    return out


fade_out = keys((OUT_A, 1.0), (OUT_B, 0.0, "ease-in"))

tracks = [
    {"name": "set", "layer": 0, "elements": [
        {"id": "study", "type": "image", "start": 0, "end": END, "source": "character/study.png",
         "x": 0, "y": 0, "origin": "top-left", "width": 1920, "height": 1080, "fit": "literal"}]},
    {"name": "laptop", "layer": 30, "elements": [
        {"id": "laptop", "type": "image", "start": LAP_IN, "end": GONE, "source": "character/laptop.png",
         "x": SCR_CX, "y": LAP_Y + 566, "origin": "bottom-center", "width": 708, "height": 566, "fit": "literal",
         "scale": keys((LAP_IN, [0.0, 0.0]), (LAP_LAND, [1.0, 1.0], BEZ)), "opacity": fade_out}]},
    {"name": "diff", "layer": 31, "elements": [
        {"id": "diff", "type": "image", "start": DIFF_IN, "end": GONE, "source": "media/diff-crop.png",
         "x": SCR_CX, "y": SCR_CY, "origin": "center", "width": SCR_W, "height": SCR_H, "fit": "literal",
         "opacity": fade_out}]},
    # A screen-coloured cover that shrinks upwards from the bottom, so the edit's lines appear top-down.
    {"name": "diff-wipe", "layer": 32, "elements": [
        {"id": "diff-wipe", "type": "rect", "start": DIFF_IN, "end": snap(DIFF_DONE + 33),
         "x": SCR_CX, "y": SCR_CY + SCR_H // 2, "origin": "bottom-center", "width": SCR_W, "height": SCR_H,
         "fill": "#4F535C", "scale": keys((DIFF_IN, [1.0, 1.0]), (DIFF_DONE, [1.0, 0.0], "ease-in-out"))}]},
    {"name": "render-track", "layer": 33, "elements": [
        {"id": "render-track", "type": "rect", "start": BAR_IN, "end": GONE,
         "x": SCR_CX - 200, "y": SCR_CY + SCR_H // 2 - 22, "origin": "center-left", "width": 400, "height": 14,
         "fill": "#101418", "radius": 7, "opacity": keys((BAR_IN, 0.0), (snap(BAR_IN + 133), 0.85, "ease-out"),
                                                         (OUT_A, 0.85, "linear"), (OUT_B, 0.0, "ease-in"))}]},
    {"name": "render-bar", "layer": 34, "elements": [
        {"id": "render-bar", "type": "rect", "start": BAR_IN, "end": GONE,
         "x": SCR_CX - 200, "y": SCR_CY + SCR_H // 2 - 22, "origin": "center-left", "width": 400, "height": 14,
         "fill": "#FF5A36", "radius": 7, "scale": keys((BAR_IN, [0.0, 1.0]), (BAR_DONE, [1.0, 1.0], "ease-in-out")),
         "opacity": fade_out}]},
    {"name": "card-border", "layer": 40, "elements": [
        {"id": "card-border", "type": "rect", "start": CARD_IN, "end": GONE,
         "x": keys((CARD_IN, SCR_CX), (CARD_LAND, CARD_CX, BEZ)),
         "y": keys((CARD_IN, SCR_CY), (CARD_LAND, CARD_CY, BEZ)), "origin": "center",
         "width": CARD_W + 16, "height": CARD_H + 16, "fill": "#F5F0E6", "radius": 12,
         "scale": keys((CARD_IN, [0.3, 0.3]), (CARD_LAND, [1.0, 1.0], BEZ)), "opacity": fade_out,
         "effects": [{"name": "shadow", "dx": 0, "dy": 8, "radius": 16, "color": "#101418", "opacity": 0.3}]}]},
    {"name": "card", "layer": 41, "elements": [
        {"id": "card", "type": "image", "start": CARD_IN, "end": GONE, "source": "stills/session-02.png",
         "x": keys((CARD_IN, SCR_CX), (CARD_LAND, CARD_CX, BEZ)),
         "y": keys((CARD_IN, SCR_CY), (CARD_LAND, CARD_CY, BEZ)), "origin": "center",
         "width": CARD_W, "height": CARD_H, "fit": "contain",
         "scale": keys((CARD_IN, [0.3, 0.3]), (CARD_LAND, [1.0, 1.0], BEZ)), "opacity": fade_out}]},
]

# An upward Signal arrow from the laptop to the video it rendered.
ARROW_TIP = CARD_CY + CARD_H // 2 + 8 + 16
ARROW_BASE = LAP_Y - 8
arrow_op = keys((snap(CARD_LAND - 100), 0.0), (snap(CARD_LAND + 100), 1.0, "ease-out"), (OUT_A, 1.0, "linear"), (OUT_B, 0.0, "ease-in"))
tracks.append({"name": "arrow", "layer": 35, "elements": [
    {"id": "arrow-shaft", "type": "rect", "start": snap(CARD_LAND - 100), "end": GONE,
     "x": CARD_CX, "y": ARROW_BASE, "origin": "bottom-center", "width": 12, "height": ARROW_BASE - ARROW_TIP,
     "fill": "#FF5A36", "radius": 6, "opacity": arrow_op}]})
tracks.append({"name": "arrow-head-l", "layer": 36, "elements": [
    {"id": "arrow-head-l", "type": "rect", "start": snap(CARD_LAND - 100), "end": GONE,
     "x": CARD_CX, "y": ARROW_TIP, "origin": "top-center", "width": 12, "height": 30,
     "fill": "#FF5A36", "radius": 6, "rotation": 45.0, "opacity": arrow_op}]})
tracks.append({"name": "arrow-head-r", "layer": 37, "elements": [
    {"id": "arrow-head-r", "type": "rect", "start": snap(CARD_LAND - 100), "end": GONE,
     "x": CARD_CX, "y": ARROW_TIP, "origin": "top-center", "width": 12, "height": 30,
     "fill": "#FF5A36", "radius": 6, "rotation": -45.0, "opacity": arrow_op}]})

# The brand: lockup at 0.28 (754x174, the file's 2694x623 at one scale), centred on the open wall.
tracks.append({"name": "brand", "layer": 50, "elements": [
    {"id": "lockup", "type": "image", "start": BRAND_IN, "end": END, "source": "brand/lockup.png",
     "x": 1200, "y": 430, "origin": "center", "width": 754, "height": 174, "fit": "literal",
     "scale": keys((BRAND_IN, [0.0, 0.0]), (BRAND_LAND, [1.0, 1.0], BEZ))}]})

# Sound: the line whole, and the bed well under it, faded out before 10 s.
tracks.append({"name": "voice", "layer": 1, "elements": [
    {"id": "voice", "type": "audio", "start": VOICE_AT, "end": VOICE_AT + 5520, "source": "character/voice/line-2.wav",
     "source_start": 0, "source_end": 5520}]})
tracks.append({"name": "music", "layer": 2, "elements": [
    {"id": "bed", "type": "audio", "start": 0, "end": END, "source": "music/bed-120bpm.wav",
     "source_start": 0, "source_end": END,
     "volume": keys((0, 0.0), (400, 0.3, "ease-out"), (1000, 0.12, "ease-in-out"), (6720, 0.12, "linear"),
                    (7200, 0.3, "ease-in-out"), (8800, 0.3, "linear"), (9800, 0.0, "ease-in"))}]})

project = {
    "frame": {"width": 1920, "height": 1080},
    "fps": 30,
    "background": "#F5F0E6",
    "duration": END,
    "output": "deliverable.mp4",
    "tracks": tracks,
}

with open("base.montagent.json", "w") as f:
    json.dump(project, f, indent=2)

baked = subprocess.run([sys.executable, ".claude/skills/montagent-character/scripts/bake_rig.py",
                        "base.montagent.json", "character/rig.json", "owl.spec.json"],
                       capture_output=True, text=True)
sys.stderr.write(baked.stderr)
if baked.returncode:
    sys.exit(baked.returncode)
with open("hoot.montagent.json", "w") as f:
    f.write(baked.stdout)
