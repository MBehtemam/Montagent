"""Write the fixtures #597 scores, on the `prototype/swaps` branch (#614).

    python3 docs/research/prototypes/swaps/make_fixtures.py

Deterministic, and re-run only to regenerate; the committed JSON is what gets scored.
The owl's swaps are derived from the pinned baked owl rather than typed in, so the two
can be held to pixel equality. The PiP files are written out here in full.
"""

import json
import pathlib

from PIL import Image, ImageDraw

HERE = pathlib.Path(__file__).resolve().parent
RESEARCH = HERE.parents[1]
BAKED = (
    RESEARCH
    / "skills-eval/runs/dev/C-character-short/with-skills-1/workspace/hoot.montagent.json"
)
BASE = "character/parts/mouth_closed.png"


def dump(path, document):
    path.write_text(json.dumps(document, indent=2, ensure_ascii=False) + "\n")


# ---- The owl: 39 mouth elements become one element with swaps. ---------------------


def baked_rotation(mouths):
    """The 39 baked mouths' own rotation records, joined into one list.

    Not the head's records, though every mouth copies them: the baker rounds each copy,
    and the copies drift from the head by up to 0.007° at painted frames — sub-pixel, but
    enough to move anti-aliased edge pixels. Joining the copies keeps the swap owl
    pixel-equal to the baked one, so any difference the score finds is authoring.

    A mouth holds its last record until its `end` (clamping). Where the next mouth starts
    on a different value, its first record takes `step`, which holds the earlier value up
    to that instant and then jumps — exactly what the cut between two elements did.
    """
    joined = []
    for mouth in mouths:
        own = mouth.get("rotation", 0)
        records = own if isinstance(own, list) else [{"t": mouth["start"], "v": own}]
        for i, record in enumerate(records):
            record = dict(record)
            if i == 0 and joined:
                if joined[-1]["t"] == record["t"]:
                    assert joined[-1]["v"] == record["v"], record
                    continue
                if joined[-1]["v"] != record["v"]:
                    record["ease"] = "step"
                else:
                    record["ease"] = "linear"
            joined.append(record)
    return joined


def owl(ended):
    document = json.loads(BAKED.read_text())
    tracks = {track["name"]: track for track in document["tracks"]}
    mouths = tracks["owl-mouth"]["elements"]
    start, end = mouths[0]["start"], mouths[-1]["end"]

    rotation = baked_rotation(mouths)

    # One swap per baked mouth, except where the baked mouth *is* the base: there the
    # swap before it ends instead. A swap before a pause ends too, so the base shows in
    # the pause (the baked owl shows no mouth there; see the README).
    swaps = []
    for i, mouth in enumerate(mouths):
        following = mouths[i + 1] if i + 1 < len(mouths) else None
        if mouth["source"] == BASE:
            continue
        swap = {"start": mouth["start"]}
        next_is_this_swaps_end = following is not None and following["start"] == mouth["end"]
        ends_on_base = following is not None and following["source"] == BASE
        if ended or (not next_is_this_swaps_end and following is not None) or ends_on_base:
            swap["end"] = mouth["end"]
        swap["source"] = mouth["source"]
        swaps.append(swap)

    first = mouths[0]
    tracks["owl-mouth"]["elements"] = [
        {
            "id": "owl-mouth",
            "type": "image",
            "start": start,
            "end": end,
            "source": BASE,
            "x": first["x"],
            "y": first["y"],
            "origin": first["origin"],
            "width": first["width"],
            "height": first["height"],
            "fit": first["fit"],
            "rotation": rotation,
            "swaps": swaps,
        }
    ]
    return document


# ---- The PiP phone mock-up: 4 screens under one push-in. ----------------------------

W, H = 1080, 2340  # every screenshot but one
ODD = (1080, 2400)  # `done`, deliberately a different pixel size
# On the 25 fps grid (a frame every 40 ms), so every step starts on a painted frame and
# every keyframe sits on one: a ~160 ms tap, ~2 s of loading.
FPS = 25
TIMES = [("home", 0), ("tap", 1200), ("loading", 1360), ("done", 3360)]
DURATION = 6000
LAST = DURATION - 1000 // FPS  # the last painted frame
RATE = 0.00005  # the push-in's scale per ms, linear, so every split boundary is exact
X, Y = 960, 540
SCREEN = (498, 1080)  # `contain` of a 1080x2340 source into the 1920x1080 clip


def screen(name, size):
    w, h = size
    image = Image.new("RGB", size, "#F4F1EA")
    draw = ImageDraw.Draw(image)
    draw.rectangle([0, 0, w, 220], fill="#1F3A5F")
    draw.rectangle([80, 90, 520, 150], fill="#F4F1EA")
    for row in range(5):
        y = 360 + row * 300
        draw.rounded_rectangle([80, y, w - 80, y + 220], radius=36, fill="#FFFFFF", outline="#D8D2C4", width=6)
        draw.rectangle([140, y + 60, 140 + 100, y + 160], fill="#C9B79C")
        draw.rectangle([300, y + 70, w - 160, y + 100], fill="#8A8F98")
        draw.rectangle([300, y + 130, w - 360, y + 150], fill="#B5BAC2")
    button = [140, h - 420, w - 140, h - 240]
    if name == "home":
        draw.rounded_rectangle(button, radius=90, fill="#E07A3F")
    elif name == "tap":
        draw.rounded_rectangle(button, radius=90, fill="#A8521F")
        draw.ellipse([w // 2 - 90, h - 420, w // 2 + 90, h - 240], fill="#FFFFFF")
    elif name == "loading":
        draw.rectangle([0, 220, w, h], fill="#E9E5DC")
        cx, cy = w // 2, h // 2
        draw.arc([cx - 160, cy - 160, cx + 160, cy + 160], 0, 270, fill="#1F3A5F", width=40)
    elif name == "done":
        draw.rectangle([0, 220, w, h], fill="#2E7D4F")
        cx, cy = w // 2, h // 2
        draw.line([(cx - 200, cy), (cx - 50, cy + 160), (cx + 230, cy - 170)], fill="#FFFFFF", width=70)
    elif name == "error":
        draw.rectangle([0, 220, w, h], fill="#B3261E")
        cx, cy = w // 2, h // 2
        draw.line([(cx - 170, cy - 170), (cx + 170, cy + 170)], fill="#FFFFFF", width=70)
        draw.line([(cx - 170, cy + 170), (cx + 170, cy - 170)], fill="#FFFFFF", width=70)
    return image


def scale_at(t):
    return round(0.6 + RATE * t, 6)


def push_in(start, end):
    """The push-in over `[start, end)`, its last record on the last painted frame —
    a record at the exclusive `end` is one no frame reaches (`R-KEYFRAME-UNREACHED`)."""
    last = end - 1000 // FPS
    return [
        {"t": start, "v": [scale_at(start)] * 2},
        {"t": last, "v": [scale_at(last)] * 2, "ease": "linear"},
    ]


def pip_header():
    return {
        "frame": {"width": 1920, "height": 1080},
        "fps": FPS,
        "background": "#20242B",
        "duration": DURATION,
        "output": "pip.mp4",
    }


def phone_body():
    return {
        "id": "phone-body",
        "type": "rect",
        "start": 0,
        "end": DURATION,
        "x": X,
        "y": Y,
        "origin": "center",
        "width": SCREEN[0] + 44,
        "height": SCREEN[1] + 88,
        "fill": "#0E0F12",
        "radius": 56,
        "scale": push_in(0, DURATION),
    }


def screen_box():
    return {
        "x": X,
        "y": Y,
        "origin": "center",
        "width": SCREEN[0],
        "height": SCREEN[1],
        "fit": "contain",
        "clip": [0, 0, 1920, 1080],
    }


def ranges():
    ends = [t for _, t in TIMES[1:]] + [DURATION]
    return [(name, start, end) for (name, start), end in zip(TIMES, ends)]


def pip_split(scaled=True):
    elements = []
    for name, start, end in ranges():
        element = {"id": f"screen-{name}", "type": "image", "start": start, "end": end,
                   "source": f"screens/{name}.png", **screen_box()}
        if scaled:
            element["scale"] = push_in(start, end)
        elements.append(element)
    return elements


def pip(spelling):
    document = pip_header()
    if spelling == "split":
        document["tracks"] = [
            {"name": "phone", "layer": 1, "elements": [phone_body()]},
            {"name": "screen", "layer": 2, "elements": pip_split()},
        ]
    elif spelling in ("swaps", "swaps-ended"):
        swaps = []
        for name, start, end in ranges()[1:]:
            swap = {"start": start}
            if spelling == "swaps-ended":
                swap["end"] = end
            swap["source"] = f"screens/{name}.png"
            swaps.append(swap)
        document["tracks"] = [
            {"name": "phone", "layer": 1, "elements": [phone_body()]},
            {"name": "screen", "layer": 2, "elements": [{
                "id": "screen", "type": "image", "start": 0, "end": DURATION,
                "source": "screens/home.png", **screen_box(),
                "scale": push_in(0, DURATION), "swaps": swaps,
            }]},
        ]
    elif spelling == "group":
        # HYPOTHETICAL — no build reads this. The split, inside a group that carries the
        # push-in once, so the score can credit #499's container with its own win rather
        # than the swap. The `group` element type and its `elements` key do not exist.
        body = phone_body()
        del body["scale"]
        document["tracks"] = [{"name": "phone", "layer": 1, "elements": [{
            "id": "phone", "type": "group", "start": 0, "end": DURATION,
            "x": X, "y": Y, "origin": "center", "scale": push_in(0, DURATION),
            "elements": [body] + pip_split(scaled=False),
        }]}]
    return document


def main():
    owl_dir = HERE / "owl"
    owl_dir.mkdir(exist_ok=True)
    dump(owl_dir / "hoot-swaps.montagent.json", owl(ended=False))
    dump(owl_dir / "hoot-swaps-ended.montagent.json", owl(ended=True))

    pip_dir = HERE / "pip"
    (pip_dir / "screens").mkdir(parents=True, exist_ok=True)
    for name in ["home", "tap", "loading", "done", "error"]:
        screen(name, ODD if name == "done" else (W, H)).save(
            pip_dir / "screens" / f"{name}.png", optimize=True
        )
    dump(pip_dir / "pip-split.montagent.json", pip("split"))
    dump(pip_dir / "pip-swaps.montagent.json", pip("swaps"))
    dump(pip_dir / "pip-swaps-ended.montagent.json", pip("swaps-ended"))
    dump(pip_dir / "pip-group.hypothetical.json", pip("group"))


if __name__ == "__main__":
    main()


# ---- Seeds for #597's edit test: a finished PiP, one per spelling. ------------------


def seeds():
    """`--seed` directories for `run_arm.py`: each `workspace/` holds `phone.montagent.json`
    in one spelling and the five screens. Every screen is 1080x2340 here, so the seed
    validates and renders clean; the odd-sized `done.png` above exists only to prove the
    fit check, and in a seed it would be a defect the agent stops to fix."""
    for spelling in ("split", "swaps", "swaps-ended"):
        work = HERE / "seeds" / f"P-{spelling}" / "workspace"
        (work / "screens").mkdir(parents=True, exist_ok=True)
        for name in ["home", "tap", "loading", "done", "error"]:
            screen(name, (W, H)).save(work / "screens" / f"{name}.png", optimize=True)
        dump(work / "phone.montagent.json", pip(spelling))


if __name__ == "__main__":
    seeds()
