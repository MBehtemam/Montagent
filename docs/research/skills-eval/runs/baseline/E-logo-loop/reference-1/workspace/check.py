"""Decode deliverable.mp4 and check loop join, cursor blink and typing order."""
import subprocess
from PIL import Image, ImageChops

W = 1080
raw = subprocess.run(["ffmpeg", "-v", "error", "-i", "deliverable.mp4", "-f", "rawvideo",
                      "-pix_fmt", "rgb24", "-"], capture_output=True, check=True).stdout
n = len(raw) // (W * W * 3)
fr = [Image.frombytes("RGB", (W, W), raw[i * W * W * 3:(i + 1) * W * W * 3]) for i in range(n)]
print("frames", n, "ground px", fr[0].getpixel((5, 5)), "target (245,240,230)")

def maxdiff(a, b):
    return max(hi for lo, hi in ImageChops.difference(a, b).getextrema())

print("first vs last max diff", maxdiff(fr[0], fr[-1]))
print("empty frames", [i for i in range(n) if maxdiff(fr[i], fr[0]) <= 4])

# text/cursor region to the right of the mark
for i in range(66, 162):
    reg = fr[i].crop((320, 430, 1070, 650)).convert("L")
    bb = reg.point(lambda v: 255 if v < 120 else 0).getbbox()
    print(f"{i:3d} {i/30:.2f}s {bb}")
