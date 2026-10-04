"""Per case: how many of its instants are byte-identical to mode `none`, and the max |delta|.

    uv run --with numpy --with pillow python compare.py <instants.tsv> <png dir> <mode tag>...

A mode tag is the mode with ':' -> '_' and '+' -> 'p' (keep:pic -> keep_pic, k3+1 -> k3p1).
"""
import collections
import sys

import numpy as np
from PIL import Image

instants, pngdir, tags = sys.argv[1], sys.argv[2], sys.argv[3:]
inst = [l.rstrip("\n").split("\t") for l in open(instants)]
for tag in tags:
    per = collections.OrderedDict()
    for name, t in inst:
        a = np.asarray(Image.open(f"{pngdir}/none/{t}.png").convert("RGBA")).astype(int)
        b = np.asarray(Image.open(f"{pngdir}/{tag}/{t}.png").convert("RGBA")).astype(int)
        d = np.abs(a - b)
        n = int((d.max(axis=2) > 0).sum())
        p = per.setdefault(name, [0, 0, 0, 0])
        p[0] += 1
        p[1] += n == 0
        p[2] = max(p[2], int(d.max()))
        p[3] += n
    ok = sum(v[1] for v in per.values())
    total = sum(v[0] for v in per.values())
    print(f"## {tag}: {ok}/{total} instants identical")
    for name, (k, same, mx, px) in per.items():
        print(f"{tag}\t{name}\t{same}/{k}\tmax|d|={mx}\tpx_differ={px}")
