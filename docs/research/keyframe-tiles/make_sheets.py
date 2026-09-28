#!/usr/bin/env python3
"""PROTOTYPE -- throwaway. #407: does a keyframe tile catch anything the
run-start tile misses, and does it still catch it after the tile-count increase
shrinks every tile?

Three sheets over the same range at the same one-image budget, from the same
doctored document (`make_kf_project.py`), composed at the size the API actually
SERVES -- the #396 method, so the file on disk is the file that was judged:

  S1  run-start-18   the 18 run-start instants                    (ADR-0094 default)
  S2  keyframe-22    + the 4 keyframes that host a defect         (diagnostic: the
                     best case a keyframe tile could ever have -- not a policy, the
                     tool cannot know which keyframes host defects)
  S3  keyframe-31    + all 13 interior keyframes                  (what the flag gives you)

Run: python3 make_kf_project.py && python3 classify_keyframes.py && python3 make_sheets.py
(the 31 frames under tiles/ are derived and not committed -- see FINDINGS.md)
"""
import json, math, os
from PIL import Image, ImageDraw, ImageFont

SP = os.path.dirname(os.path.abspath(__file__))
TILES, OUT = os.path.join(SP, "tiles"), os.path.join(SP, "sheets")
FONT = os.path.join(SP, "../../../fixtures/en-halloween-decorating/fonts/OpenRunde-Bold.otf")
STD = (1568, 1568)          # standard tier: max long edge px, max visual tokens
SRC_W, SRC_H = 1080, 1920
LABEL_FRAC = 0.11           # ADR-0098 decision 1
FLOOR_PX = 8                # ADR-0098 decision 4: 8 px of served type

def tok(w, h): return math.ceil(w / 28) * math.ceil(h / 28)

def resized(width, height, max_edge=1568, max_tokens=1568):
    def fits(w, h):
        return (math.ceil(w / 28) * 28 <= max_edge and math.ceil(h / 28) * 28 <= max_edge
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

def served(w, h): a, b = resized(w, h, *STD); return a, b, tok(a, b)

def best_grid(n, tile_w, tile_h):
    """#396: the (cols, rows) giving the largest served tile -- nearest square."""
    best = None
    for c in range(1, n + 1):
        r = math.ceil(n / c)
        sw, sh, t = served(c * tile_w, r * tile_h)
        px = (sw / c) * (sh / r)
        if best is None or px > best[0]: best = (px, c, r)
    return best[1], best[2]

def build(name, tiles):
    """tiles: list of (instant_ms, label, is_keyframe)."""
    n = len(tiles)
    lab_h = int(SRC_H * LABEL_FRAC)
    cell_w, cell_h = SRC_W, SRC_H + lab_h
    cols, rows = best_grid(n, cell_w, cell_h)
    sheet_w, sheet_h = cols * cell_w, rows * cell_h
    sw, sh, t = served(sheet_w, sheet_h)
    scale = sw / sheet_w
    tile_sw = sw / cols
    # ADR-0098 d.4: fit to tile width, floored at 8 px SERVED -> authored floor / scale
    authored_floor = FLOOR_PX / scale

    sheet = Image.new("RGB", (sheet_w, sheet_h), (24, 24, 28))
    d = ImageDraw.Draw(sheet)
    for i, (ms, text, is_kf) in enumerate(tiles):
        c, r = i % cols, i // cols
        x0, y0 = c * cell_w, r * cell_h
        sheet.paste(Image.open(os.path.join(TILES, f"{ms}.png")).convert("RGB"), (x0, y0))
        # ADR-0098 d.6: the keyframe class carries a visual mark on the sheet
        gutter = (86, 52, 16) if is_kf else (24, 24, 28)
        d.rectangle([x0, y0 + SRC_H, x0 + cell_w - 1, y0 + cell_h - 1], fill=gutter)
        size = int(lab_h * 0.62)
        while size > authored_floor:
            f = ImageFont.truetype(FONT, size)
            if d.textlength(text, font=f) <= cell_w - lab_h * 0.5: break
            size -= 2
        f = ImageFont.truetype(FONT, max(size, int(authored_floor)))
        d.text((x0 + int(lab_h * 0.3), y0 + SRC_H + lab_h * 0.18), text, font=f,
               fill=(255, 222, 170) if is_kf else (255, 255, 255))
        d.rectangle([x0, y0, x0 + cell_w - 1, y0 + cell_h - 1], outline=(90, 90, 100),
                    width=max(2, SRC_W // 300))
    os.makedirs(OUT, exist_ok=True)
    sheet.resize((sw, sh), Image.LANCZOS).save(os.path.join(OUT, f"{name}.png"))
    served_type = max(size, int(authored_floor)) * scale
    print(f"{name:<16} {n:>3} tiles  {cols}x{rows}  served {sw}x{sh}  {t:>4} tok  "
          f"tile {tile_sw:.0f} px wide  smallest label type {served_type:.1f} px served")
    return dict(name=name, n=n, cols=cols, rows=rows, tile_w=round(tile_sw), tokens=t,
                type_px=round(served_type, 1))

def labels(instants, boundary, interior, runs):
    """ADR-0098 d.2 shape: `<index> <sigil> <instant>ms <+-offset> <+-id>`."""
    kf_at = {}
    for eid, prop, t, inst in interior:
        kf_at.setdefault(inst, []).append(f"{eid}.{prop}")
    out = []
    for i, ms in enumerate(instants):
        run = next((r for r in runs if r[0] <= ms < r[1]), None)
        off = ms - run[0] if run else 0
        if ms in kf_at and ms not in boundary:
            out.append((ms, f"{i+1} ◆ {ms}ms {off:+d} {kf_at[ms][0]}", True))
        else:
            out.append((ms, f"{i+1} {ms}ms {off:+d} {run[2] if run else ''}", False))
    return out

if __name__ == "__main__":
    ins = json.load(open(os.path.join(SP, "instants.json")))
    runs_raw = json.load(open(os.path.join(SP, "runs.json")))
    boundary, interior = set(ins["boundary"]), ins["interior"]
    DEFECT_KF = [12000, 20000, 38000, 46000]   # D3, D1, D2, D4 -- see make_kf_project.py
    rows = []
    for name, sel in (("S1-run-start-18", sorted(boundary)),
                      ("S2-keyframe-22", sorted(boundary | set(DEFECT_KF))),
                      ("S3-keyframe-31", ins["union"])):
        rows.append(build(name, labels(sel, boundary, interior, runs_raw)))
    json.dump(rows, open(os.path.join(SP, "geometry.json"), "w"), indent=1)
