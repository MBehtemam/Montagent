import subprocess, sys
P = "social.montagent.json"
W, H = 1080, 1920
for t in sys.argv[1:]:
    png = f"/tmp/claude-501/mb/g{t}.png"
    subprocess.run(["montagent", "frame", P, "--at", t, "--full", "--png", "--out", png], check=True, capture_output=True)
    raw = subprocess.run(["ffmpeg", "-loglevel", "error", "-i", png, "-f", "rawvideo", "-pix_fmt", "rgb24", "-"], check=True, capture_output=True).stdout
    greenish = olive = 0; pts = []
    for y in range(0, H):
        row = y * W * 3
        for x in range(W):
            if (x - 824) ** 2 + (y - 960) ** 2 < 215 ** 2: continue
            p = row + x * 3; r, g, b = raw[p], raw[p + 1], raw[p + 2]
            if g > r + 8 and g > b + 8:
                greenish += 1
                if len(pts) < 8: pts.append((x, y, r, g, b))
    print(t, "green-dominant px:", greenish, pts)
