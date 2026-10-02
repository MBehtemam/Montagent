# usage: compare.py <montagent> <a.montagent.json> <b.montagent.json> <out_dir> <instant>...
# Renders both projects at each instant (full scale, PNG) and reports how far the pixels differ.
import os, subprocess, sys
import numpy as np
from PIL import Image

montagent, a, b, out = sys.argv[1:5]
instants = [int(t) for arg in sys.argv[5:] for t in arg.split()]
os.makedirs(out, exist_ok=True)
worst = (0, None)
identical = 0
for t in instants:
    pics = []
    for tag, project in (("a", a), ("b", b)):
        path = os.path.join(out, f"{t:05d}-{tag}.png")
        if not os.path.exists(path):
            subprocess.run([montagent, "frame", project, "--at", str(t), "--full", "--png",
                            "--out", path], check=True, stdout=subprocess.DEVNULL)
        pics.append(np.asarray(Image.open(path).convert("RGBA"), dtype=np.int16))
    d = np.abs(pics[0] - pics[1]).max(axis=2)
    n, m = int((d > 0).sum()), int(d.max())
    identical += n == 0
    if m > worst[0]:
        worst = (m, t)
    print(f"{t:5d} ms: {n:7d} px differ, max {m:3d}, >8: {int((d > 8).sum()):6d}", flush=True)
print(f"{identical}/{len(instants)} identical; largest channel difference {worst[0]} at {worst[1]} ms")
