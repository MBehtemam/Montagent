"""Writes hoot.base.json (set, props, brand, sound) and hoot.spec.json (the owl).

Then: python3 .claude/skills/montagent-character/scripts/bake_rig.py \
        hoot.base.json character/rig.json hoot.spec.json > hoot.montagent.json
"""
import json

FPS = 30


def f(ms):
    """Snap a time to its drawn frame."""
    return int(round(ms * FPS / 1000) * 1000 // FPS)


END = 10000
POP = [0.34, 1.56, 0.64, 1]
SETTLE = [0.25, 1, 0.5, 1]

# Laptop on the floor, right of the owl. 708x566, bottom-center at (1150, 965).
LAP_X, LAP_Y = 1200, 965
# Screen centre in frame: image screen centre (353.5, 176.5) of 708x566 -> offset from bottom-center.
SCR_X, SCR_Y = LAP_X + int(353.5 - 354), LAP_Y - (566 - 176)
CARD_X, CARD_Y, CARD_W, CARD_H = 1200, 222, 576, 324

lap_pop_start, lap_pop_end = f(2733), f(3033)
type_times = [f(3400), f(3500), f(3600), f(3700), f(3800), f(3900)]
exit_start, exit_end = f(6566), f(6800)
card_start, card_land = f(5000), f(5566)
bar_start, bar_end = f(5600), f(6200)
logo_start, logo_land = f(6700), f(7000)


def laptop(k, start, end, scale=None):
    e = {"id": f"laptop-{k}", "type": "image", "start": start, "end": end,
         "source": f"media/laptop-{k}.png", "x": LAP_X, "y": LAP_Y, "origin": "bottom-center",
         "width": 708, "height": 566, "fit": "literal"}
    if scale:
        e["scale"] = scale
    return e


laptops = []
bounds = [lap_pop_start] + type_times + [exit_end]
for k in range(7):
    scale = None
    if k == 0:
        scale = [{"t": lap_pop_start, "v": [0.0, 0.0]}, {"t": lap_pop_end, "v": [1.0, 1.0], "ease": POP}]
    if k == 6:
        scale = [{"t": exit_start, "v": [1.0, 1.0]}, {"t": f(exit_end - 33), "v": [0.0, 0.0], "ease": "ease-in"}]
    laptops.append(laptop(k, bounds[k], bounds[k + 1], scale))

card = {"id": "card", "type": "image", "start": card_start, "end": exit_end,
        "source": "stills/session-02.png",
        "x": [{"t": card_start, "v": SCR_X}, {"t": card_land, "v": CARD_X, "ease": SETTLE}],
        "y": [{"t": card_start, "v": SCR_Y}, {"t": card_land, "v": CARD_Y, "ease": SETTLE}],
        "origin": "center", "width": CARD_W, "height": CARD_H, "fit": "literal",
        "scale": [{"t": card_start, "v": [0.25, 0.25]}, {"t": card_land, "v": [1.0, 1.0], "ease": POP},
                  {"t": exit_start, "v": [1.0, 1.0], "ease": "linear"},
                  {"t": f(exit_end - 33), "v": [0.0, 0.0], "ease": "ease-in"}],
        "effects": [{"name": "shadow", "dx": 0, "dy": 10, "radius": 18, "color": "#101418", "opacity": 0.35}]}

BAR_X, BAR_Y, BAR_W = CARD_X - CARD_W // 2 + 24, CARD_Y + CARD_H // 2 - 22, CARD_W - 48
bar_track = {"id": "bar-track", "type": "rect", "start": card_land, "end": exit_start,
             "x": BAR_X, "y": BAR_Y, "origin": "center-left", "width": BAR_W, "height": 8,
             "fill": "#F5F0E64D", "radius": 4}
bar_fill = {"id": "bar-fill", "type": "rect", "start": bar_start, "end": exit_start,
            "x": BAR_X, "y": BAR_Y, "origin": "center-left", "width": BAR_W, "height": 8,
            "fill": "#FF5A36", "radius": 4,
            "scale": [{"t": bar_start, "v": [0.0, 1.0]}, {"t": bar_end, "v": [1.0, 1.0], "ease": "ease-in-out"}]}

logo = {"id": "lockup", "type": "image", "start": logo_start, "end": END,
        "source": "brand/lockup.png", "x": 1215, "y": 470, "origin": "center",
        "width": 778, "height": 180, "fit": "contain",
        "scale": [{"t": logo_start, "v": [0.0, 0.0]}, {"t": logo_land, "v": [1.0, 1.0], "ease": POP}]}

VOICE_AT = f(1000)
voice = {"id": "voice", "type": "audio", "start": VOICE_AT, "end": VOICE_AT + 5520,
         "source": "character/voice/line-2.wav", "source_start": 0, "source_end": 5520}
bed = {"id": "bed", "type": "audio", "start": 0, "end": END,
       "source": "music/bed-120bpm.wav", "source_start": 0, "source_end": END,
       "volume": [{"t": 0, "v": 0.5}, {"t": 900, "v": 0.12, "ease": "linear"},
                  {"t": 6500, "v": 0.12, "ease": "linear"}, {"t": 7000, "v": 0.5, "ease": "linear"},
                  {"t": 8500, "v": 0.5, "ease": "linear"}, {"t": 9900, "v": 0.0, "ease": "linear"}]}

project = {
    "frame": {"width": 1920, "height": 1080},
    "fps": FPS,
    "background": "#F5F0E6",
    "duration": END,
    "output": "deliverable.mp4",
    "tracks": [
        {"name": "set", "layer": 0, "elements": [
            {"id": "set", "type": "image", "start": 0, "end": END, "source": "character/study.png",
             "x": 0, "y": 0, "origin": "top-left", "width": 1920, "height": 1080, "fit": "literal"}]},
        {"name": "laptop", "layer": 5, "elements": laptops},
        {"name": "card", "layer": 6, "elements": [card]},
        {"name": "bar-track", "layer": 7, "elements": [bar_track]},
        {"name": "bar-fill", "layer": 8, "elements": [bar_fill]},
        {"name": "brand", "layer": 6, "elements": [logo]},
        {"name": "voice", "layer": 0, "elements": [voice]},
        {"name": "music", "layer": 0, "elements": [bed]},
    ],
}
json.dump(project, open("hoot.base.json", "w"), indent=2)

DOWN = {"upper_arm_left": -50, "upper_arm_right": 50, "forearm_left": -10, "forearm_right": 10,
        "head": 0, "torso": 0}
spec = {
    "prefix": "owl", "layer": 10, "start": 0, "end": END, "scale": 0.56,
    "place": [{"t": 0, "x": -320, "y": 965}, {"t": f(900), "x": 520, "ease": "ease-out"}],
    "hops": [{"from": 0, "to": f(300), "height": 60}, {"from": f(300), "to": f(600), "height": 38},
             {"from": f(600), "to": f(900), "height": 20}],
    "poses": {
        "down": DOWN,
        # A question to the viewer: an open-handed shrug, palms up, head cocked, a slight lean.
        "ask": {"upper_arm_left": 10, "forearm_left": 38, "upper_arm_right": -10, "forearm_right": -38,
                "head": -8, "torso": -2},
        "ask2": {"forearm_left": 48, "forearm_right": -48, "head": -11},
        "settle": {"upper_arm_left": -35, "forearm_left": -5, "upper_arm_right": 35, "forearm_right": 5,
                   "head": -2, "torso": 0},
        # Towards the laptop.
        "point": {"upper_arm_left": -50, "forearm_left": -10,
                  "upper_arm_right": -28, "forearm_right": -2, "head": 5, "torso": 2},
        "nod": {"head": 9}, "nodback": {"head": 4},
        # Up towards the rendered video.
        "lookup": {"upper_arm_right": -60, "forearm_right": -12, "head": 8, "torso": 1},
        "nod2": {"head": 11}, "nod2back": {"head": 6},
        "happy": {"upper_arm_left": -40, "upper_arm_right": 40, "forearm_left": -5, "forearm_right": 5,
                  "head": 3, "torso": 0},
        "cheer": {"upper_arm_left": 46, "upper_arm_right": -46, "forearm_left": 26, "forearm_right": -26,
                  "head": 0},
        "tiltl": {"head": -3}, "tiltr": {"head": 3},
    },
    "moves": [
        {"t": 0, "pose": "down"},
        {"t": f(1300), "pose": "ask", "in": 300, "ease": POP},
        {"t": f(2133), "pose": "ask2", "in": 250, "ease": POP},
        {"t": f(2933), "pose": "settle", "in": 350},
        {"t": f(3366), "pose": "point", "in": 350, "ease": POP},
        {"t": f(3766), "pose": "nod", "in": 200},
        {"t": f(4066), "pose": "nodback", "in": 300},
        {"t": f(5166), "pose": "lookup", "in": 400, "ease": POP},
        {"t": f(5733), "pose": "nod2", "in": 200},
        {"t": f(6033), "pose": "nod2back", "in": 300},
        {"t": f(6433), "pose": "happy", "in": 350},
        {"t": f(7000), "pose": "cheer", "in": 300, "ease": POP},
        {"t": f(8000), "pose": "happy", "in": 400},
        {"t": f(8800), "pose": "tiltl", "in": 450},
        {"t": f(9600), "pose": "tiltr", "in": 450},
    ],
    "swings": [
        {"joint": "torso", "from": f(900), "to": END, "degrees": 1.0, "period": 2400},
        {"joint": "forearm_left", "from": f(7100), "to": f(7900), "degrees": 10, "period": 400},
        {"joint": "forearm_right", "from": f(7100), "to": f(7900), "degrees": 10, "period": 400},
    ],
    "voice": {"id": "voice", "visemes": "character/voice/line-2.json"},
    "blinks": {"part": "eyes_closed", "at": [f(2766), f(4266), f(6566), f(8366), f(9500)], "frames": 3},
}
json.dump(spec, open("hoot.spec.json", "w"), indent=2)
