"""Prepare the owl's set and prop from their FLUX drawings.

Run from `assets/`:
`uv run --python 3.12 --with numpy --with scipy --with pillow python ../pack-src/character/make_props.py`.

Reads `study-raw.png` (1920×1088) and `laptop-raw.png` (1024×1024, on white) beside this
script, both drawn by `flux.py`. Writes `character/study.png`, cropped to 1920×1080, and
`character/laptop.png`, with the white keyed out and cropped to the laptop. It prints the
laptop's screen rectangle, which the pack's README records.
"""

from pathlib import Path

import numpy as np
from PIL import Image
from scipy import ndimage as ndi

SRC = Path(__file__).parent
OUT = Path("character")
OUT.mkdir(exist_ok=True)

Image.open(SRC / "study-raw.png").convert("RGB").crop((0, 4, 1920, 1084)).save(OUT / "study.png", optimize=True)

im = np.asarray(Image.open(SRC / "laptop-raw.png").convert("RGB")).astype(np.int16)
white = im.min(2) > 225
lab, _ = ndi.label(white)
border = set(np.unique(np.r_[lab[0], lab[-1], lab[:, 0], lab[:, -1]])) - {0}
fg = ~ndi.binary_dilation(np.isin(lab, list(border)), iterations=1)
fg = ndi.binary_opening(fg, iterations=2)  # drop specks
alpha = ndi.gaussian_filter(fg.astype(float), 0.8) * 255
ys, xs = np.nonzero(fg)
y0, y1, x0, x1 = ys.min() - 2, ys.max() + 3, xs.min() - 2, xs.max() + 3
Image.fromarray(np.dstack([im, alpha]).clip(0, 255).astype(np.uint8)[y0:y1, x0:x1]).save(OUT / "laptop.png", optimize=True)

# The screen: the biggest flat dark-grey region, in the cropped PNG's pixels.
screen_px = (np.abs(im - im[380, 512]).sum(2) < 24) & fg
slab, n = ndi.label(screen_px)
biggest = 1 + int(np.argmax(ndi.sum(screen_px, slab, range(1, n + 1))))
sy, sx = np.nonzero(slab == biggest)
print(f"laptop.png {x1 - x0}x{y1 - y0}; screen x {sx.min() - x0}..{sx.max() - x0}, y {sy.min() - y0}..{sy.max() - y0}")
