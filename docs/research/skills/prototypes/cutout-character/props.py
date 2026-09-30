"""Key the white out of the props (border-connected near-white only) and crop to content.
Run: uv run --with numpy --with scipy --with pillow python props.py"""
import json
from pathlib import Path

import numpy as np
from PIL import Image
from scipy import ndimage as ndi

HERE = Path(__file__).parent
info = {}
for name in ("machine", "toast"):
    im = np.asarray(Image.open(HERE / f"{name}_raw.png").convert("RGB")).astype(np.int16)
    white = im.min(2) > 225
    lab, _ = ndi.label(white)
    border = set(np.unique(np.r_[lab[0], lab[-1], lab[:, 0], lab[:, -1]])) - {0}
    # White enclosed by pipe loops is background too; keep only the biggest enclosed
    # white (the machine's dial face) and small highlights.
    sizes = ndi.sum(white, lab, range(1, lab.max() + 1))
    inner = sorted((s, i + 1) for i, s in enumerate(sizes) if i + 1 not in border and s > 150)
    holes = [i for _, i in inner[:-1]] if name == "machine" else []
    fg = ~ndi.binary_dilation(np.isin(lab, list(border) + holes), iterations=1)
    fg = ndi.binary_opening(fg, iterations=2)  # drop specks
    a = ndi.gaussian_filter(fg.astype(float), 0.8) * 255
    ys, xs = np.nonzero(fg)
    y0, y1, x0, x1 = ys.min() - 2, ys.max() + 3, xs.min() - 2, xs.max() + 3
    out = np.dstack([im, a]).clip(0, 255).astype(np.uint8)[y0:y1, x0:x1]
    Image.fromarray(out).save(HERE / "parts" / f"{name}.png")
    info[name] = {"file": f"parts/{name}.png", "w": int(x1 - x0), "h": int(y1 - y0)}

bg = Image.open(HERE / "bgC.png").convert("RGB")
bg.crop((0, 4, 1920, 1084)).save(HERE / "parts" / "bg.png")
info["bg"] = {"file": "parts/bg.png", "w": 1920, "h": 1080}
(HERE / "parts" / "props.json").write_text(json.dumps(info, indent=1))
print(info)
