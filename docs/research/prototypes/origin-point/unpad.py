# usage: unpad.py <rig.json> <out_dir> <grid> <margin> [zero]
# Trims each padded part to its art's alpha bbox (+margin), aligned to `grid` source px
# measured from the part's centre (its joint), or from the PNG's top-left with `zero`, and
# writes a rig whose parts carry `joint`: the joint's position in the trimmed PNG, in
# source px.
#   5 0        the score's owl: boxes and joints stay exact integers at the baker's 0.6
#   2 4 zero   the equivalence pair: even offsets keep Skia's mip level 1 aligned, and the
#              transparent margin keeps edge clamping from bleeding art outward
import json, os, sys
from PIL import Image

rig_path, out_dir, grid, margin = sys.argv[1], sys.argv[2], int(sys.argv[3]), int(sys.argv[4])
anchor_zero = len(sys.argv) > 5 and sys.argv[5] == "zero"
rig = json.load(open(rig_path))
rig_dir = os.path.dirname(os.path.abspath(rig_path))
os.makedirs(os.path.join(out_dir, "parts"), exist_ok=True)

def down(v, c):   # largest value <= v on the grid through c
    return c - ((c - v + grid - 1) // grid) * grid
def up(v, c):     # smallest value >= v on the grid through c
    return c + ((v - c + grid - 1) // grid) * grid

for name, part in rig["parts"].items():
    im = Image.open(os.path.join(rig_dir, part["file"])).convert("RGBA")
    W, H = im.size
    assert (W, H) == (part["width"], part["height"]), name
    assert W % 2 == 0 and H % 2 == 0, name
    cx, cy = W // 2, H // 2
    ax, ay = (0, 0) if anchor_zero else (cx, cy)
    l, t, r, b = im.getchannel("A").getbbox()
    # Not clamped to the PNG: a crop past its edge is filled with transparency, which is
    # what the padding there already was.
    l, t = down(l - margin, ax), down(t - margin, ay)
    r, b = up(r + margin, ax), up(b + margin, ay)
    art = im.getchannel("A").getbbox()
    assert l <= art[0] and t <= art[1] and r >= art[2] and b >= art[3], \
        f"{name}: the grid cuts into the art; use a finer grid"
    crop = im.crop((l, t, r, b))
    file = f"parts/{name}.png"
    crop.save(os.path.join(out_dir, file))
    part.update(file=file, width=r - l, height=b - t, joint=[cx - l, cy - t])
    print(f"{name}: {W}x{H} -> {r-l}x{b-t} at ({l},{t}), joint {part['joint']}", file=sys.stderr)

json.dump(rig, open(os.path.join(out_dir, "rig.json"), "w"), indent=1)
