"""Prototype: procedural VFX plates for the trailer.
Run: uv run --with numpy --with pillow --with scipy python plates.py"""
from pathlib import Path

import numpy as np
from PIL import Image
from scipy import ndimage as ndi

OUT = Path("fx")
OUT.mkdir(exist_ok=True)
rng = np.random.default_rng(7)


def save(name, rgb, a):
    rgb = np.broadcast_to(np.asarray(rgb, float), a.shape + (3,)) if np.ndim(rgb) == 1 else rgb
    Image.fromarray(np.dstack([np.clip(rgb, 0, 255), np.clip(a * 255, 0, 255)]).astype(np.uint8)).save(OUT / name)


# The rifling: the inside of a barrel, grooves spiralling towards a bright centre.
N = 1080
yy, xx = np.mgrid[0:N, 0:N] - N / 2 + 0.5
r = np.hypot(xx, yy) / (N / 2)
th = np.arctan2(yy, xx)
spiral = 0.5 + 0.5 * np.cos(8 * (th + 2.2 * np.log(r + 0.05)))
grooves = ndi.gaussian_filter(np.clip((spiral - 0.35) * 3, 0, 1), 1.2)
lum = 22 + 70 * grooves * r ** 0.6 + 40 * np.exp(-((r - 0.18) ** 2) / 0.004)  # a lit ring near the muzzle
lum *= np.clip(1.15 - r * 0.5, 0, 1)
save("rifling.png", np.dstack([lum * 0.95, lum, lum * 1.05]), (r <= 1.0).astype(float))

# Vignette: black, transparent in the middle.
yy, xx = np.mgrid[0:1080, 0:1920]
d = np.hypot((xx - 960) / 960, (yy - 540) / 540)
save("vignette.png", [0, 0, 0], np.clip((d - 0.55) / 0.75, 0, 1) ** 1.6 * 0.85)

# Grain: a plate larger than the frame, jittered every couple of frames.
g = rng.normal(0, 1, (1400, 2400))
g = ndi.gaussian_filter(g, 0.7)
g = (g - g.min()) / (g.max() - g.min())
save("grain.png", np.dstack([g * 255] * 3), np.full(g.shape, 0.10))

# Anamorphic flare: a thin blue-white horizontal streak with a hot core.
yy, xx = np.mgrid[0:240, 0:2400]
core = np.exp(-((yy - 120) / 5) ** 2) * np.exp(-((xx - 1200) / 700) ** 2)
halo = np.exp(-((yy - 120) / 28) ** 2) * np.exp(-((xx - 1200) / 260) ** 2) * 0.6
a = np.clip(core + halo, 0, 1)
save("flare.png", np.dstack([170 + 85 * a, 210 + 45 * a, np.full(a.shape, 255.0)]), a)

# Light sweep: a soft diagonal band of warm white.
yy, xx = np.mgrid[0:600, 0:400]
band = np.exp(-((xx - 200 + (yy - 300) * 0.35) / 45) ** 2)
save("sweep.png", [255, 240, 205], band * 0.55)

# Embers: glowing orange specks on a tall plate that drifts upward.
H, W = 2400, 1920
ember = np.zeros((H, W))
for _ in range(260):
    x, y, s = rng.integers(0, W), rng.integers(0, H), rng.uniform(1.0, 3.2)
    ember[y, x] = s
glow = ndi.gaussian_filter(ember, 2.2) * 60 + ndi.gaussian_filter(ember, 0.8) * 6
glow = np.clip(glow, 0, 1)
save("embers.png", np.dstack([255 * np.ones_like(glow), 120 + 110 * glow, 40 + 80 * glow]), glow)

# HUD: a dim red lat/long grid with a crude continent scatter, for the control-room shot.
yy, xx = np.mgrid[0:1080, 0:1920]
grid = ((xx % 80) < 2) | ((yy % 80) < 2)
land = ndi.gaussian_filter(rng.random((27, 48)), 1.4) > 0.53
land = np.kron(land, np.ones((40, 40), bool))[:1080, :1920]
dots = land & ((xx % 16) < 6) & ((yy % 16) < 6)
a = grid * 0.18 + dots * 0.75
save("hud.png", [255, 50, 40], a)
print("plates:", sorted(p.name for p in OUT.iterdir()))
