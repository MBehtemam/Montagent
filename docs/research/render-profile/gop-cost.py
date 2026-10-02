"""Source frames frame_at decodes for the profile projects: keyframe-at-or-before (start - 200 ms) up to the instant.
Usage: gop-cost.py <long screen> <long presenter> <short screen> <short presenter>"""
import bisect, subprocess, sys

def frames(path):
    out = subprocess.run(["ffprobe", "-v", "error", "-select_streams", "v:0", "-show_entries",
                          "frame=key_frame,pts_time", "-of", "csv=p=0", path],
                         capture_output=True, text=True, check=True).stdout
    rows = [l.split(",") for l in out.split() if l]
    return sorted(float(r[1]) for r in rows), sorted(float(r[1]) for r in rows if r[0] == "1")

def cost(path, el_start, el_end, fps=30):
    pts, keys = frames(path)
    calls = decoded = worst = 0
    for f in range(el_end * fps // 1000 + 1):
        t = f * 1000 // fps
        if not el_start <= t < el_end:
            continue
        at = (t - el_start) / 1000
        k = keys[max(0, bisect.bisect_right(keys, max(0, at - 0.2) + 1e-9) - 1)]
        d = bisect.bisect_right(pts, at + 1e-6) - bisect.bisect_left(pts, k - 1e-9)
        calls += 1; decoded += d; worst = max(worst, d)
    return calls, decoded, worst

for tag, screen, presenter in [("long GOP", sys.argv[1], sys.argv[2]), ("1 s GOP", sys.argv[3], sys.argv[4])]:
    s, p = cost(screen, 0, 55000), cost(presenter, 55000, 64000)
    print(f"{tag}: {s[0] + p[0]} calls, {s[1] + p[1]} source frames decoded "
          f"({(s[1] + p[1]) / (s[0] + p[0]):.1f} per call, worst {max(s[2], p[2])})")
