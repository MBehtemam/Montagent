"""Frame-by-frame difference between the Montagent render and the reference, plus a
still sheet at the beats. Run: uv run --with numpy --with pillow python compare.py"""
import subprocess

import numpy as np
from PIL import Image

W, H, N = 480, 270, 315

def frames(path):
    raw = subprocess.run(["ffmpeg", "-loglevel", "error", "-i", path, "-vf", f"scale={W}:{H}:flags=area",
                          "-f", "rawvideo", "-pix_fmt", "rgb24", "-"], capture_output=True, check=True).stdout
    return np.frombuffer(raw, np.uint8).reshape(-1, H, W, 3).astype(np.float32)

m, r = frames("out/fox.mp4"), frames("out/fox-reference.mp4")
n = min(len(m), len(r))
psnr = [10 * np.log10(255 ** 2 / max(((m[i] - r[i]) ** 2).mean(), 1e-6)) for i in range(n)]
print("frames", len(m), len(r), "PSNR min %.1f median %.1f" % (min(psnr), float(np.median(psnr))))
worst = sorted(range(n), key=lambda i: psnr[i])[:5]
print("worst frames", [(i, round(psnr[i], 1)) for i in worst])

beats = [15, 60, 145, 177, 228, 255]
sheet = Image.new("RGB", (2 * W, len(beats) * H))
for row, i in enumerate(beats):
    sheet.paste(Image.fromarray(m[i].astype(np.uint8)), (0, row * H))
    sheet.paste(Image.fromarray(r[i].astype(np.uint8)), (W, row * H))
sheet.save("stills/compare.png")
