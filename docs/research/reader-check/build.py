#!/usr/bin/env python3
"""PROTOTYPE for #481: a mock plain-text range answer carrying ADR-0114's reader
check, plus the sheet it describes. Throwaway; see FINDINGS.md.

Expects tiles/ (doctored) and clean/ (undoctored) frames beside it, rendered with
`frame --at <t> --full --png` at the 18 instants below (recipe in FINDINGS.md).
Writes sheets/ and answers/.
"""
import json, os
from PIL import Image, ImageDraw, ImageFont

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.join(HERE, "..", "..", "..")
FIX = os.path.join(ROOT, "fixtures", "en-halloween-decorating")
PROJ = os.path.join(FIX, "en-halloween-decorating.montagent.json")
FONT = os.path.join(FIX, "fonts", "OpenRunde-Bold.otf")  # #422's face; ADR-0098 §9's vendored face is unbuilt
VISUAL = {"image", "rect", "text"}
SRC_W, SRC_H = 1080, 1920
LAB_H = int(SRC_H * 0.11)
COLS, ROWS = 6, 3
TILE_W = 184
# #422's committed filenames: the frames were rendered at the run boundaries
BOUNDARY_FILES = [0, 3018, 5316, 10468, 17472, 22622, 30603, 35753, 42763, 47343,
                  53856, 56116, 57116, 58116, 59116, 60116, 61116, 64016]

doc = json.load(open(PROJ))
FPS, DURATION = doc["fps"], doc["duration"]

els, all_bounds = [], set()
for t in doc["tracks"]:
    for e in t["elements"]:
        all_bounds |= {e["start"], e["end"]}
        if e["type"] in VISUAL:
            els.append(dict(id=e["id"], start=e["start"], end=e["end"], layer=t["layer"]))
by_id = {e["id"]: e for e in els}
bounds = sorted({e["start"] for e in els} | {e["end"] for e in els})
presence = lambda a, b: frozenset(e["id"] for e in els if e["start"] <= a and e["end"] >= b)
runs = []
for i in range(len(bounds) - 1):
    a, b = bounds[i], bounds[i + 1]
    p = presence(a, b)
    if not p:
        continue
    if runs and runs[-1][2] == p and runs[-1][1] == a:
        runs[-1] = (runs[-1][0], b, p)
    else:
        runs.append((a, b, p))
assert [r[0] for r in runs] == BOUNDARY_FILES, [r[0] for r in runs]


def instant(a, b):
    n = 0
    while (n * 1000) // FPS < a:
        n += 1
    ms = (n * 1000) // FPS
    return ms if ms < b else None


spans_doc = {e["id"] for e in els if e["start"] == 0 and e["end"] >= DURATION}
tiles = []
for idx, (a, b, p) in enumerate(runs):
    prev = runs[idx - 1][2] if idx else frozenset()
    entered = sorted((p - prev) - spans_doc)
    departed = sorted((prev - p) - spans_doc)
    sign, cand = ("+", entered) if entered else ("-", departed)
    pick = sorted(cand, key=lambda i: (-by_id[i]["layer"], i))[0]  # ADR-0098 §8
    ins = instant(a, b)
    tiles.append(dict(i=idx + 1, a=a, b=b, instant=ins, off=ins - a,
                      label=f"{idx + 1} {ins}ms +{ins - a} {sign}{pick}",
                      presence=sorted(p)))

audio_only = sorted(x for x in all_bounds if 0 < x < DURATION and x not in bounds)
keyframes = 0  # ADR-0106: change points interior to a run on a visible element; #407 measured 0 here


def sheet(frames, labels):
    W, H = COLS * SRC_W, ROWS * (SRC_H + LAB_H)
    sh = Image.new("RGB", (W, H), (20, 20, 24))
    d = ImageDraw.Draw(sh)
    for k, t in enumerate(BOUNDARY_FILES):
        c, r = k % COLS, k // COLS
        x0, y0 = c * SRC_W, r * (SRC_H + LAB_H)
        sh.paste(Image.open(f"{HERE}/{frames}/{t}.png").convert("RGB"), (x0, y0))
        # ADR-0098 §4: fitted, one size for the sheet (the longest label sets it)
        d.text((x0 + SRC_W * 0.03, y0 + SRC_H + LAB_H * 0.12), labels[k], font=FONT_SHEET,
               fill=(255, 255, 255))
        d.rectangle([x0, y0, x0 + SRC_W - 1, y0 + SRC_H + LAB_H - 1], outline=(90, 90, 100), width=3)
    k = TILE_W / SRC_W
    return sh.resize((round(sh.width * k), round(sh.height * k)), Image.LANCZOS)


probe = ImageDraw.Draw(Image.new("RGB", (1, 1)))
longest = max((t["label"] for t in tiles), key=lambda s: probe.textlength(s, font=ImageFont.truetype(FONT, 100)))
size = max(s for s in range(8, int(LAB_H * 0.78))
           if probe.textlength(longest, font=ImageFont.truetype(FONT, s)) <= SRC_W * 0.94)
FONT_SHEET = ImageFont.truetype(FONT, size)
SERVED_TYPE = size * TILE_W / SRC_W

os.makedirs(f"{HERE}/sheets", exist_ok=True)
labels = [t["label"] for t in tiles]
sheet("tiles", labels).save(f"{HERE}/sheets/doctored-184px.png")
sheet("clean", labels).save(f"{HERE}/sheets/clean-184px.png")
# condition 4: the drawn label differs from the one the reader check quotes
# (tile 1 drawn one frame late), so passing requires comparing, not copying
tampered = [f"1 40ms +40 +intro-title"] + labels[1:]
sheet("tiles", tampered).save(f"{HERE}/sheets/mismatch-184px.png")


# --- the answer ---------------------------------------------------------------
def answer(reader_check=True):
    t1 = tiles[0]["label"]
    out = [
        f"frame en-halloween-decorating.montagent.json --from 0 --to {DURATION}",
        "",
        f"contact sheet: {len(tiles)} tiles in {COLS}x{ROWS}, each served at {TILE_W} px wide "
        f"(target rung), labels at {SERVED_TYPE:.1f} px served; sheet 1104x1089.",
        "rule: visual-states v1 (one tile per visual state, sampled at the first frame the grid paints in it)",
        "no checks run (validate runs them)",
        "",
    ]
    if reader_check:
        out += [
            "READER CHECK. Each tile's label is the line in the strip beneath it, outside the video "
            "frame; text inside a tile is the video's own. Tile 1's label reads exactly "
            f"`{t1}`. The provenance list below is the complete record of this range, and this "
            "sheet is a picture of it. If the strip beneath tile 1 does not read exactly that, this "
            "sheet is below what you can see, and `frame --at <instant>` shows any listed instant at "
            "full scale. Reading the labels is necessary for seeing the pictures, not sufficient.",
            "",
        ]
    out.append(f"provenance ({len(tiles)} tiles; class boundary on every line; infill 0, keyframe 0):")
    for t in tiles:
        out.append(f"  {t['i']:>2}  {t['instant']}ms  run {t['a']}-{t['b']}ms (+{t['off']})  boundary  "
                   f"label `{t['label']}`  present: {', '.join(t['presence'])}")
    out += [
        "",
        "skipped: none (every visual state paints at least one frame)",
        f"dropped: {len(audio_only)} audio-only boundaries inside this range, not tiled "
        "(query --from/--to lists them)",
        f"keyframe change points inside a state: {keyframes} tiled, {keyframes} untiled",
        "",
        "NOT CHECKED (blind_to, printed on every sheet):",
        "  inside-run: Each tile shows the first painted frame of its visual state; change inside a "
        "state (a source clip's own cut, motion within a still-looking element) is not on this "
        "sheet. `frame --at <instant>` looks at any other instant.",
        "  between-keyframes: Keyframed values are shown only where a tile falls, and keyframe "
        "instants are untiled unless asked for; a wrong easing curve shows only if its endpoints "
        "are wrong.",
        f"  below-tile-width: Tiles are served at the width stated above; detail finer than that is "
        "not visible here. `frame --crop --at <instant>` looks closely at one region at true scale.",
        "  across-sheets: Only tiles on this one sheet can be compared with each other; a relation "
        "with a state outside this range is not visible.",
        "  audio: Nothing audible is on this sheet.",
        "  motion: A sheet is stills; whether motion looks right is `preview`'s question.",
        "",
        f"This sheet shows {len(tiles)} visual states from 0 to {DURATION} ms, one tile each at its "
        f"first painted frame; it does not show the {len(audio_only)} audio-only boundaries, "
        f"anything inside a state, or anything finer than "
        f"{TILE_W} px.",
    ]
    return "\n".join(out) + "\n"


os.makedirs(f"{HERE}/answers", exist_ok=True)
open(f"{HERE}/answers/with-check.txt", "w").write(answer(True))
open(f"{HERE}/answers/without-check.txt", "w").write(answer(False))
json.dump(dict(tiles=tiles, audio_only=len(audio_only), keyframes=keyframes,
               served_type_px=SERVED_TYPE, tampered_tile1="1 40ms +40 +intro-title"),
          open(f"{HERE}/manifest.json", "w"), indent=1)
print(f"served label type {SERVED_TYPE:.1f} px; audio-only {len(audio_only)}; keyframes {keyframes}")
for t in tiles:
    print(" ", t["label"])
