"""The reference medium: the same scene graph rendered without Montagent.

Each frame is composited in PIL with sub-pixel affine transforms straight from pose(t)
(real parenting, no keyframes, no integer positions), piped to ffmpeg, and the voice
lines are mixed with adelay. Run: uv run --with pillow python ref_render.py
"""
import math
import subprocess
from functools import lru_cache
from pathlib import Path

from PIL import Image

import scene

HERE = Path(__file__).parent
OUT = HERE / "out" / "fox-reference.mp4"

@lru_cache(maxsize=None)
def source(file):
    return Image.open(HERE / file).convert("RGBA")

@lru_cache(maxsize=256)
def scaled(file, s):
    """Pre-shrink with a proper filter so the affine step never minifies (no aliasing)."""
    im = source(file)
    if s >= 1:
        return im, 1.0
    w, h = max(1, round(im.width * s)), max(1, round(im.height * s))
    return im.resize((w, h), Image.LANCZOS), im.width / w

def draw(frame, file, x, y, rot, s):
    im, back = scaled(file, round(s, 3))
    s_eff = s * back
    w, h = im.size
    cx, cy = w / 2, h / 2
    r = math.radians(rot)
    c, sn = math.cos(r), math.sin(r)
    # Screen-space bounding box of the rotated, scaled canvas.
    corners = [(x + s_eff * ((px - cx) * c - (py - cy) * sn), y + s_eff * ((px - cx) * sn + (py - cy) * c))
               for px, py in ((0, 0), (w, 0), (0, h), (w, h))]
    x0 = max(0, math.floor(min(p[0] for p in corners)))
    y0 = max(0, math.floor(min(p[1] for p in corners)))
    x1 = min(scene.W, math.ceil(max(p[0] for p in corners)))
    y1 = min(scene.H, math.ceil(max(p[1] for p in corners)))
    if x1 <= x0 or y1 <= y0:
        return
    # Output pixel (X, Y) -> input pixel, sampled at pixel centres.
    a, b = c / s_eff, sn / s_eff
    d, e = -sn / s_eff, c / s_eff
    ox, oy = x0 + 0.5 - x, y0 + 0.5 - y
    data = (a, b, cx + a * ox + b * oy - 0.5, d, e, cy + d * ox + e * oy - 0.5)
    patch = im.transform((x1 - x0, y1 - y0), Image.AFFINE, data, resample=Image.BICUBIC)
    frame.alpha_composite(patch, (x0, y0))

def main():
    OUT.parent.mkdir(exist_ok=True)
    els = sorted(scene.elements(), key=lambda e: e[3])
    inputs, filters = [], []
    for n, (f, at, src_end, _) in enumerate(scene.LINES):
        inputs += ["-i", str(HERE / f)]
        filters.append(f"[{n + 1}:a]atrim=0:{src_end / 1000},adelay={at}:all=1[a{n}]")
    mix = ";".join(filters) + ";" + "".join(f"[a{n}]" for n in range(len(scene.LINES))) + \
        f"amix=inputs={len(scene.LINES)}:normalize=0,apad,atrim=0:{scene.DURATION / 1000}[aout]"
    ff = subprocess.Popen(
        ["ffmpeg", "-y", "-loglevel", "error", "-f", "rawvideo", "-pix_fmt", "rgb24",
         "-s", f"{scene.W}x{scene.H}", "-r", str(scene.FPS), "-i", "-", *inputs,
         "-filter_complex", mix, "-map", "0:v", "-map", "[aout]",
         "-c:v", "libx264", "-pix_fmt", "yuv420p", "-crf", "16", "-c:a", "aac", "-shortest", str(OUT)],
        stdin=subprocess.PIPE)
    frames = scene.FRAMES[:-1]
    for i, t in enumerate(frames):
        pose = scene.pose(t)
        frame = Image.new("RGBA", (scene.W, scene.H), (0, 0, 0, 255))
        for el_id, file, _, _, start, end in els:
            if start <= t < end:
                x, y, rot, s = pose[scene.node_of(el_id)]
                draw(frame, file, x, y, rot, s)
        ff.stdin.write(frame.convert("RGB").tobytes())
        if i % 60 == 0:
            print(f"{i}/{len(frames)}", flush=True)
    ff.stdin.close()
    ff.wait()
    print("wrote", OUT)

if __name__ == "__main__":
    main()
