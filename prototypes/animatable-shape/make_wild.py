#!/usr/bin/env python3
"""PROTOTYPE (#668), throwaway: writes wild.montagent.json, 36 rects that morph through
five acts. Every value in the output is a literal; this script is only the typist."""
import colorsys, json, math

END = 8000
OVER = [0.34, 1.56, 0.64, 1]
N = 6
PITCH = 160

def hexc(h, s, v, a=255):
    r, g, b = colorsys.hsv_to_rgb(h % 1.0, s, v)
    rgb = "#%02X%02X%02X" % (round(r * 255), round(g * 255), round(b * 255))
    return rgb if a == 255 else rgb + "%02X" % a      # the schema refuses an opaque eight-digit colour

class Keys:
    def __init__(self, v):
        self.k = [{"t": 0, "v": v}]
    def to(self, start, end, v, ease="ease-in-out"):
        last = self.k[-1]
        if start > last["t"]:
            self.k.append({"t": start, "v": last["v"], "ease": "linear"})
        assert end > self.k[-1]["t"], (start, end, self.k[-1])
        self.k.append({"t": end, "v": v, "ease": ease})
        return self

# The equalizer: one level (0..6 lit segments) per column, per beat.
BEATS = [4100, 4350, 4600, 4850, 5100, 5350]
LEVELS = [
    [2, 5, 3, 6, 4, 1],
    [4, 3, 6, 2, 5, 3],
    [6, 2, 4, 5, 1, 4],
    [3, 6, 2, 4, 6, 2],
    [5, 4, 5, 1, 3, 6],
    [1, 6, 3, 5, 2, 5],
]

tracks = []
for r in range(N):
    for c in range(N):
        k = r * N + c
        cx = round(540 + (c - 2.5) * PITCH)
        cy = round(540 + (r - 2.5) * PITCH)
        ring = max(abs(r - 2.5), abs(c - 2.5)) - 0.5          # 0, 1, 2 from the centre
        w, h, rad = Keys(4), Keys(4), Keys(70)   # never 0: render refuses an element with no extent
        x, y = Keys(cx), Keys(cy)
        fill, stroke = Keys("#FFFFFF"), Keys(hexc(k / 36, 0.9, 1, 0))

        # Act 1: a diagonal wave of dots pops into rounded squares.
        s = (r + c) * 60
        cool = hexc(0.5 + (r + c) / 10 * 0.35, 0.85, 1)
        w.to(100 + s, 500 + s, 140, OVER); h.to(100 + s, 500 + s, 140, OVER)
        fill.to(100 + s, 500 + s, cool)
        rad.to(500 + s, 800 + s, 24)

        # Act 2: a ripple from the centre shrinks them to hot circles, then they swell.
        s = round(ring * 140)
        hot = hexc(0.14 - ring * 0.07, 0.95, 1)
        w.to(1700 + s, 2200 + s, 64, OVER); h.to(1700 + s, 2200 + s, 64, OVER)
        rad.to(1700 + s, 2200 + s, 32)
        fill.to(1700 + s, 2200 + s, hot)
        w.to(2500 + s, 2900 + s, 110, OVER); h.to(2500 + s, 2900 + s, 110, OVER)
        rad.to(2500 + s, 2900 + s, 55)

        # Act 3: column by column they become the segments of a level meter, and it plays.
        s = c * 70
        level_from_bottom = N - r                              # 1 at the bottom row
        led = hexc(0.33 - (level_from_bottom - 1) / 5 * 0.33, 0.9, 1)
        lit = 1
        w.to(3300 + s, 3700 + s, 124); h.to(3300 + s, 3700 + s, 124)
        rad.to(3300 + s, 3700 + s, 14)
        fill.to(3300 + s, 3700 + s, led)
        for beat, levels in zip(BEATS, LEVELS):
            want = 1 if level_from_bottom <= levels[c] else 0
            if want != lit:
                h.to(beat, beat + 140, 124 if want else 10, "ease-out")
                w.to(beat, beat + 140, 124 if want else 40, "ease-out")
                lit = want
        if not lit:
            h.to(5560, 5600, 124, "linear"); w.to(5560, 5600, 124, "linear")

        # Act 4: everything flies to the centre and opens into 36 concentric rings.
        s = k * 12
        size = 44 + k * 28
        x.to(5700 + s, 6300 + s, 540); y.to(5700 + s, 6300 + s, 540)
        w.to(5700 + s, 6300 + s, size, OVER); h.to(5700 + s, 6300 + s, size, OVER)
        rad.to(5700 + s, 6300 + s, size // 2 + 1)
        fill.to(5700 + s, 5860 + s, led[:7] + "00")        # empty out first, or 36 big squares flood the frame
        stroke.to(5700 + s, 5860 + s, hexc(k / 36, 0.9, 1))
        # ...and a pulse runs outward through them, shifting their colour.
        p = 6760 + k * 10
        w.to(p, p + 140, round(size * 1.12), "ease-out"); h.to(p, p + 140, round(size * 1.12), "ease-out")
        stroke.to(p, p + 140, "#FFFFFF", "ease-out")
        w.to(p + 140, p + 300, size, "ease-in"); h.to(p + 140, p + 300, size, "ease-in")
        stroke.to(p + 140, p + 300, hexc(k / 36 + 0.5, 0.9, 1), "ease-in")

        # Act 5: the rings collapse into the pill the first prototype began with.
        w.to(7450, 7800, 360, OVER); h.to(7450, 7800, 96, OVER)
        rad.to(7450, 7800, 48)
        fill.to(7450, 7800, "#3B82F6")
        stroke.to(7450, 7700, "#3B82F600")

        tracks.append({"name": f"t{k:02d}", "layer": k, "elements": [{
            "id": f"tile-{k:02d}", "type": "rect", "start": 0, "end": END,
            "x": x.k, "y": y.k, "origin": "center",
            "width": w.k, "height": h.k,
            "fill": fill.k, "stroke": stroke.k, "stroke_width": 8, "radius": rad.k,
        }]})

project = {"frame": {"width": 1080, "height": 1080}, "fps": 30, "background": "#070A14",
           "output": "out/wild.mp4", "duration": END, "tracks": tracks}
with open("wild.montagent.json", "w") as f:
    json.dump(project, f, separators=(",", ":"))
print("elements", len(tracks), "keyframes",
      sum(len(v) for t in tracks for v in t["elements"][0].values() if isinstance(v, list)))
