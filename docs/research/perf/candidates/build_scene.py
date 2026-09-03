#!/usr/bin/env python3
"""Derive one declarative scene.json from the fixture.

All three candidate backends render *this* file, so their numbers are
comparable to each other and to the FFmpeg baseline, which rendered the same
content from merged.ass + the spans/audio table in ../ffmpeg-baseline/render.py.

Throwaway. The shape here is not a proposed project format.
"""
import re, json, pathlib

ROOT = pathlib.Path("/Users/mohammedehtemam/projects/github/Montaget")
FIX = ROOT / "fixtures/en-halloween-decorating"
MERGED = ROOT / "docs/research/perf/ffmpeg-baseline/merged.ass"

W, H, FPS = 1080, 1920, 30
CARD_H = 1300
CREAM = "#FBF3E3"

SPANS = [("05", 17.472), ("06", 13.131), ("07", 12.160), ("08", 11.093), ("05", 11.360)]
AUDIO = [
    ("intro-2.mp3", 0.0), ("hook-2.mp3", 3.02),
    ("05-cobweb.mp3", 5.32), ("sentence-05-cobweb.mp3", 10.47),
    ("06-spider.mp3", 17.47), ("sentence-06-spider.mp3", 22.62),
    ("07-skeleton.mp3", 30.6), ("sentence-07-skeleton.mp3", 35.75),
    ("08-lights.mp3", 42.76), ("sentence-08-lights.mp3", 47.33),
    ("quiz-2.mp3", 53.85), ("sentence-05-cobweb.mp3", 61.11),
]

def ass_colour(v):
    """&HAABBGGRR& -> #RRGGBB"""
    h = v.strip("&").lstrip("Hh").rstrip("&")
    h = h.zfill(8)[-6:]          # BBGGRR
    return "#" + h[4:6] + h[2:4] + h[0:2]

def parse_ts(t):
    hh, mm, ss = t.split(":")
    return int(hh) * 3600 + int(mm) * 60 + float(ss)

styles = {}
events = []
section = None
for line in MERGED.read_text().splitlines():
    st = line.strip()
    if st.startswith("["):
        section = st.lower(); continue
    if section == "[v4+ styles]" and st.startswith("Style:"):
        f = st[len("Style:"):].split(",")
        styles[f[0].strip()] = {
            "font": f[1].strip(), "size": float(f[2]),
            "colour": ass_colour(f[3]), "bold": f[8].strip() == "-1",
        }
    elif section == "[events]" and st.startswith("Dialogue:"):
        m = re.match(r"Dialogue:\s*(\d+),([^,]+),([^,]+),([^,]+),[^,]*,[^,]*,[^,]*,[^,]*,[^,]*,(.*)", st)
        layer, start, end, style, text = int(m.group(1)), parse_ts(m.group(2)), parse_ts(m.group(3)), m.group(4).strip(), m.group(5)

        tags = "".join(re.findall(r"\{([^}]*)\}", text))
        an = int(re.search(r"\\an(\d)", tags).group(1)) if re.search(r"\\an(\d)", tags) else 2
        pos = re.search(r"\\pos\((-?[\d.]+),(-?[\d.]+)\)", tags)
        x, y = (float(pos.group(1)), float(pos.group(2))) if pos else (W / 2, H / 2)
        colr = re.search(r"\\c(&H[0-9A-Fa-f]+&)", tags)
        colour = ass_colour(colr.group(1)) if colr else styles[style]["colour"]
        fs = re.search(r"\\fs([\d.]+)", tags)
        size = float(fs.group(1)) if fs else styles[style]["size"]

        body = re.sub(r"\{[^}]*\}", "", text)

        if "\\p1" in tags:
            # ASS drawing: "m X Y l X Y X Y X Y" — every shape in this fixture is a rect
            nums = [float(v) for v in re.findall(r"-?[\d.]+", body)]
            xs, ys = nums[0::2], nums[1::2]
            events.append({
                "kind": "rect", "layer": layer, "start": start, "end": end,
                "x": min(xs), "y": min(ys),
                "w": max(xs) - min(xs), "h": max(ys) - min(ys),
                "colour": colour,
            })
        else:
            events.append({
                "kind": "text", "layer": layer, "start": start, "end": end,
                "x": x, "y": y, "align": an, "colour": colour, "size": size,
                "font": styles[style]["font"], "bold": styles[style]["bold"],
                "lines": body.split("\\N"),
            })

# Ken Burns spans, resolved to explicit source rects per span
crop_h = round(1536 * CARD_H / W)          # 1850 — the window the baseline crops to
spans, t = [], 0.0
for i, (img, dur) in enumerate(SPANS):
    frames = max(1, round(dur * FPS))
    spans.append({
        "image": str(FIX / f"images/{img}.png"),
        "start": t, "end": t + dur, "frames": frames,
        "cropW": 1536, "cropH": crop_h,
        # zoom over the span, matching the baseline's zoompan expressions exactly
        "zoomFrom": 1.0001 if i % 2 == 0 else 1.12,
        "zoomTo":   1.12   if i % 2 == 0 else 1.0001,
        "zoomStep": 0.0009,
    })
    t += dur

scene = {
    "width": W, "height": H, "fps": FPS, "duration": round(t, 3),
    "cardH": CARD_H, "background": CREAM,
    "spans": spans,
    "badge": {"src": str(FIX / "brand/logo-en.png"), "x": 470, "y": 104, "w": 64, "h": 64},
    "events": events,
    "audio": [{"src": str(FIX / "audio" / f), "at": at} for f, at in AUDIO],
}

out = pathlib.Path(__file__).parent / "scene.json"
out.write_text(json.dumps(scene, indent=1))
print(f"scene.json: {len(spans)} spans, {len(events)} events "
      f"({sum(1 for e in events if e['kind']=='text')} text, "
      f"{sum(1 for e in events if e['kind']=='rect')} rect), "
      f"{len(scene['audio'])} audio, {scene['duration']}s")
