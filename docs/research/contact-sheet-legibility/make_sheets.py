#!/usr/bin/env python3
"""PROTOTYPE -- throwaway. Answers issue #396: how many tiles before a contact
sheet stops showing a defect, and what grid shape a 9:16 frame wants.

Composes contact sheets at the dimensions a model is actually SERVED, so the
image on disk is the image the model sees -- no second downscale hides behind
the measurement. See FINDINGS.md.

Rebuilding the inputs from a clean checkout
-------------------------------------------
Both inputs this script expects -- `doctored/` and `tiles/` -- are derived, and
are not committed (23 MB of frames). Recreate them beside this file:

    cargo build --release

    # 1. the doctored project: the map's two planted defects, still 0 errors
    mkdir -p doctored && cd doctored
    for d in audio brand fonts images; do
        ln -s ../../../../fixtures/en-halloween-decorating/$d $d
    done
    python3 - <<'EOF'
    import json
    src = "../../../fixtures/en-halloween-decorating/en-halloween-decorating.montagent.json"
    d = json.load(open(src))
    for t in d["tracks"]:
        for e in t.get("elements", []):
            if e["id"] == "photo-07":   e["source"] = "images/06.png"   # wrong photo
            if e["id"] == "sentence-08": e["color"] = "#1E344C"         # invisible text
    json.dump(d, open("en-halloween-decorating.montagent.json", "w"), indent=2)
    EOF
    cd ..

    # 2. the 18 visual-state frames, at true scale (see FINDINGS.md on #402:
    #    --full is required, or every tile is downscaled twice)
    mkdir -p tiles
    for t in 0 3018 5316 10468 17472 22622 30603 35753 42763 47343 53856 \
             56116 57116 58116 59116 60116 61116 64016; do
        ./target/release/montagent frame \
            doctored/en-halloween-decorating.montagent.json \
            --at $t --full --png --out tiles/$t.png
    done

    python3 make_sheets.py && python3 label_test.py
"""
import math, os, sys
from PIL import Image, ImageDraw, ImageFont

SP = os.path.dirname(os.path.abspath(__file__))
TILES = os.path.join(SP, "tiles")
OUT = os.path.join(SP, "sheets")
FONT = os.path.join(SP, "../../../fixtures/en-halloween-decorating/fonts/OpenRunde-Bold.otf")

STD = (1568, 1568)          # standard tier: max long edge px, max visual tokens
HI  = (2576, 4784)          # Claude 4.7+

INSTANTS = [0, 3018, 5316, 10468, 17472, 22622, 30603, 35753, 42763,
            47343, 53856, 56116, 57116, 58116, 59116, 60116, 61116, 64016]

# What each instant is, for the label. group/what-produced-it, per the ticket.
LABEL = {
    0: "intro", 3018: "item-05 hook", 5316: "item-05 word", 10468: "item-05 sent",
    17472: "item-06 word", 22622: "item-06 sent", 30603: "item-07 word",
    35753: "item-07 sent", 42763: "item-08 word", 47343: "item-08 sent",
    53856: "quiz q", 56116: "count 5", 57116: "count 4", 58116: "count 3",
    59116: "count 2", 60116: "count 1", 61116: "quiz word", 64016: "loop hook",
}

def tok(w, h): return math.ceil(w / 28) * math.ceil(h / 28)

def resized(width, height, max_edge=1568, max_tokens=1568):
    def fits(w, h):
        return (math.ceil(w / 28) * 28 <= max_edge
                and math.ceil(h / 28) * 28 <= max_edge
                and tok(w, h) <= max_tokens)
    if fits(width, height): return (width, height)
    if height > width:
        rh, rw = resized(height, width, max_edge, max_tokens); return (rw, rh)
    ar = width / height; lo, hi = 1, width
    while lo + 1 < hi:
        mid = (lo + hi) // 2
        if fits(mid, max(round(mid / ar), 1)): lo = mid
        else: hi = mid
    return (lo, max(round(lo / ar), 1))

def served(w, h, tier=STD):
    a, b = resized(w, h, tier[0], tier[1]); return a, b, tok(a, b)

def best_grid(n, tile_w, tile_h, tier=STD):
    """The (cols, rows) giving the largest served tile for n tiles of this shape."""
    best = None
    for c in range(1, n + 1):
        r = math.ceil(n / c)
        sw, sh, t = served(c * tile_w, r * tile_h, tier)
        px = (sw / c) * (sh / r)
        if best is None or px > best[0]:
            best = (px, c, r, sw, sh, t)
    return best[1], best[2]

def build(name, instants, cols, rows, crop=None, tier=STD, label_frac=0.11,
          budget_note=""):
    src_w, src_h = (1080, 1920) if crop is None else (crop[2], crop[3])
    lab_h = int(src_h * label_frac)
    cell_w, cell_h = src_w, src_h + lab_h
    sheet_w, sheet_h = cols * cell_w, rows * cell_h
    sw, sh, t = served(sheet_w, sheet_h, tier)

    sheet = Image.new("RGB", (sheet_w, sheet_h), (24, 24, 28))
    d = ImageDraw.Draw(sheet)
    f = ImageFont.truetype(FONT, int(lab_h * 0.62))
    for i, ms in enumerate(instants):
        c, r = i % cols, i // cols
        im = Image.open(os.path.join(TILES, f"{ms}.png")).convert("RGB")
        if crop: im = im.crop((crop[0], crop[1], crop[0] + crop[2], crop[1] + crop[3]))
        x0, y0 = c * cell_w, r * cell_h
        sheet.paste(im, (x0, y0))
        txt = f"{i+1}  {ms}ms  {LABEL.get(ms,'')}"
        d.text((x0 + int(lab_h * 0.3), y0 + src_h + lab_h * 0.18), txt,
               font=f, fill=(255, 255, 255))
        d.rectangle([x0, y0, x0 + cell_w - 1, y0 + cell_h - 1],
                    outline=(90, 90, 100), width=max(2, src_w // 300))

    out = sheet.resize((sw, sh), Image.LANCZOS)
    path = os.path.join(OUT, f"{name}.png")
    out.save(path)
    tile_sw, tile_sh = sw / cols, (sh / rows) * (src_h / cell_h)
    print(f"{name:<28} {len(instants):>3} tiles  {cols}x{rows}  "
          f"authored {sheet_w}x{sheet_h}  served {sw}x{sh}  {t:>4} tok  "
          f"tile {tile_sw:.0f}x{tile_sh:.0f} ({tile_sw/src_w:.3f}x)  {budget_note}")
    return dict(name=name, n=len(instants), cols=cols, rows=rows, served=(sw, sh),
                tokens=t, tile=(round(tile_sw), round(tile_sh)),
                scale=round(tile_sw / src_w, 4))

if __name__ == "__main__":
    os.makedirs(OUT, exist_ok=True)
    rows = []
    # --- A. tile count sweep, best grid each, full frame, standard tier ---
    for n in (4, 6, 9, 12, 18, 24, 36):
        picks = INSTANTS[:n] if n <= 18 else (INSTANTS * 2)[:n]
        c, r = best_grid(n, 1080, 1920 + int(1920 * 0.11))
        rows.append(build(f"A-count-{n:02d}", picks, c, r, budget_note="sweep"))
    # --- B. grid shape at n=18 ---
    for c, r, tag in ((6, 3, "grid-6x3"), (3, 6, "grid-3x6"), (18, 1, "strip-h"),
                      (1, 18, "strip-v"), (9, 2, "grid-9x2")):
        rows.append(build(f"B-{tag}", INSTANTS, c, r, budget_note="shape"))
    # --- C. cropped band (caption + sentence card) across all 18 ---
    band = (0, 1300, 1080, 360)
    c, r = best_grid(18, band[2], band[3] + int(band[3] * 0.11))
    rows.append(build("C-crop-band-18", INSTANTS, c, r, crop=band, label_frac=0.11,
                      budget_note="crop"))
    rows.append(build("C-crop-band-36", (INSTANTS * 2)[:36], 3, 12, crop=band,
                      budget_note="crop"))
    # --- D. cheap budget: one default half-scale frame (700 tok) is not a knob we
    #        have directly; show the high-res tier instead for contrast ---
    rows.append(build("D-hires-18", INSTANTS, 6, 3, tier=HI, budget_note="4.7+ tier"))
    rows.append(build("D-hires-36", (INSTANTS * 2)[:36], 8, 5, tier=HI,
                      budget_note="4.7+ tier"))
