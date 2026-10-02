# /// script
# dependencies = ["numpy", "pillow"]
# ///
"""#598's #299 case: photograph MAD against #299's reference frames, per spelling.
Renders each spelling at 400 and 14000 ms (full, PNG), downsamples to the reference's
270x480, and takes the mean absolute difference over the photograph's clip [0,0,1080,1300]
(270x325 at that size), plus the arm-vs-arm max difference at full scale."""
import subprocess, sys
from pathlib import Path
import numpy as np
from PIL import Image
FX, BIN, OUT = Path(sys.argv[1]), sys.argv[2], Path(sys.argv[3])
REFS = {400: "frame-intro.png", 14000: "frame-05-at-11s.png"}
SPELL = ["en-halloween-decorating", "kb-armA-int", "kb-armB"]
full = {}
for t, ref in REFS.items():
    r = np.asarray(Image.open(FX / "reference" / ref).convert("RGB"), dtype=np.float64)
    for s in SPELL:
        png = OUT / f"{s}-{t}.png"
        subprocess.run([BIN, "frame", "--at", str(t), "--full", "--png", "--out", str(png), str(FX / f"{s}.montagent.json")],
                       check=True, capture_output=True)
        im = Image.open(png).convert("RGB"); full[s, t] = np.asarray(im, dtype=np.int16)
        small = np.asarray(im.resize((r.shape[1], r.shape[0]), Image.LANCZOS), dtype=np.float64)
        h = round(1300 * r.shape[0] / im.height)
        print(f"{t:>6} ms  {s:<26} MAD {np.abs(small[:h] - r[:h]).mean():.3f}")
    a, b = full["kb-armA-int", t], full["kb-armB", t]
    d = np.abs(a - b).max(axis=2)
    print(f"{t:>6} ms  armA-int vs armB at full scale: {int((d > 0).sum())} px differ, max {int(d.max())}/255")
