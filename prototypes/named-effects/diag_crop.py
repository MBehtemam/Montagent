"""PROTOTYPE #722: where the directional blur's crop changes pixels, per element region."""

import os
import pathlib
import subprocess

HERE = pathlib.Path(__file__).parent
BIN = os.environ.get("MONTAGENT_BIN", "montagent")
W, H = 1920, 1080
project = os.environ.get("PROJECT", "dblur-bound.json")
at = os.environ.get("AT", "1500")


def frame(extra):
    png = HERE / "out" / "diag.png"
    subprocess.run([BIN, "frame", project, "--at", at, "--full", "--png", "--out", str(png)],
                   env=dict(os.environ, **extra), cwd=HERE, capture_output=True, check=True)
    raw = subprocess.run(["ffmpeg", "-v", "error", "-i", str(png), "-f", "rawvideo", "-pix_fmt",
                          "rgb24", "-"], capture_output=True, check=True).stdout
    png.unlink()
    return raw


a = frame({})
b = frame({os.environ.get("AGAINST", "MONTAGENT_PROTO_DBLUR_NO_CROP"): "1"})
diffs = []
for i in range(0, len(a), 3):
    if a[i:i + 3] != b[i:i + 3]:
        p = i // 3
        diffs.append((p % W, p // W, tuple(a[i:i + 3]), tuple(b[i:i + 3])))
print(f"{len(diffs)} pixels differ")
if diffs:
    xs = [d[0] for d in diffs]
    ys = [d[1] for d in diffs]
    print("bbox", min(xs), min(ys), max(xs), max(ys))
    print("max level diff", max(max(abs(x - y) for x, y in zip(d[2], d[3])) for d in diffs))
    for d in diffs[:10]:
        print(d)
