# Builds h3.json: set, crate, laptop, screens, tag, voice; then bakes the owl from owl.spec.json.
import json, subprocess, sys
p = json.load(open('h3.json'))
OVER = [0.34, 1.56, 0.64, 1]
LX, LY, LW, LH = 839, 433, 425, 340             # laptop, top-left; scale ~0.6
SX, SY, SW, SH = 911, 458, 280, 162             # its screen, in frame px
CX, CY = SX + SW // 2, SY + SH // 2
CRATE = (823, 773, 457, 177)                    # x, y, w, h: top meets the laptop's base
TAG_X, TAG_Y = CX, 360
def el(**k): return k
pop = lambda t0: [{"t": t0, "v": [0.0, 0.0]}, {"t": t0 + 300, "v": [1.0, 1.0], "ease": OVER}]
repop = [{"t": 5600, "v": [1.0, 1.0], "ease": "linear"}, {"t": 5700, "v": [1.12, 1.12], "ease": "ease-out"}, {"t": 5833, "v": [1.0, 1.0], "ease": "ease-in-out"}]
cx, cy, cw, ch = CRATE
tracks = [
  {"name": "set", "layer": 0, "elements": [
    el(id="study", type="image", start=0, end=8000, source="character/study.png", x=0, y=0, origin="top-left", width=1920, height=1080, fit="literal")]},
  {"name": "floor-shadow-owl", "layer": 2, "elements": [
    el(id="floor-shadow-owl", type="ellipse", start=0, end=8000, x=720, y=968, origin="center", width=270, height=34, fill="#7A4E2C55",
       scale=[{"t": 5600, "v": [1.0, 1.0]}, {"t": 5850, "v": [0.7, 0.7], "ease": "ease-out"}, {"t": 6100, "v": [1.0, 1.0], "ease": "ease-in"}])]},
  {"name": "crate-shadow", "layer": 1, "elements": [
    el(id="crate-shadow", type="ellipse", start=0, end=8000, x=cx + cw // 2, y=cy + ch - 2, origin="center", width=cw + 50, height=30, fill="#7A4E2C55")]},
  {"name": "crate", "layer": 3, "elements": [
    el(id="crate", type="rect", start=0, end=8000, x=cx, y=cy, origin="top-left", width=cw, height=ch, fill="#C98B55", stroke="#4A3226", stroke_width=6, radius=8)]},
  {"name": "crate-slat-1", "layer": 4, "elements": [
    el(id="crate-slat-1", type="rect", start=0, end=8000, x=cx + 6, y=cy + 58, origin="top-left", width=cw - 12, height=5, fill="#4A3226")]},
  {"name": "crate-slat-2", "layer": 4, "elements": [
    el(id="crate-slat-2", type="rect", start=0, end=8000, x=cx + 6, y=cy + 116, origin="top-left", width=cw - 12, height=5, fill="#4A3226")]},
  {"name": "crate-lid", "layer": 4, "elements": [
    el(id="crate-lid", type="rect", start=0, end=8000, x=cx - 10, y=cy - 6, origin="top-left", width=cw + 20, height=26, fill="#DDA36C", stroke="#4A3226", stroke_width=6, radius=6)]},
  {"name": "laptop", "layer": 5, "elements": [
    el(id="laptop", type="image", start=0, end=8000, source="character/laptop.png", x=LX, y=LY, origin="top-left", width=LW, height=LH, fit="literal")]},
  {"name": "screen", "layer": 6, "elements": [
    el(id="screen-session", type="image", start=0, end=7333, source="stills/session-02.png", x=CX, y=CY, origin="center", width=288, height=162, fit="cover", clip=[SX, SY, SW, SH]),
    el(id="screen-ground", type="rect", start=7333, end=8000, x=SX, y=SY, origin="top-left", width=SW, height=SH, fill="#F5F0E6")]},
  {"name": "screen-mark", "layer": 7, "elements": [
    el(id="screen-mark", type="image", start=7333, end=8000, source="brand/mark.png", x=CX, y=CY, origin="center", width=116, height=116, fit="literal",
       scale=[{"t": 7333, "v": [0.6, 0.6]}, {"t": 7633, "v": [1.0, 1.0], "ease": OVER}])]},
  {"name": "tag-bg", "layer": 40, "elements": [
    el(id="tag-bg", type="rect", start=3133, end=8000, x=TAG_X, y=TAG_Y, origin="center", width=320, height=90, fill="#FF5A36", radius=45, scale=pop(3133) + repop)]},
  {"name": "tag-text", "layer": 41, "elements": [
    el(id="tag-late", type="text", start=3133, end=5600, caption=False, x=TAG_X, y=TAG_Y, origin="center", width=260, height=60, font="bold", size=50, color="#101418", align="center", runs=[{"text": "+2 frames"}], scale=pop(3133)),
    el(id="tag-fixed", type="text", start=5600, end=8000, caption=False, x=TAG_X, y=TAG_Y, origin="center", width=230, height=60, font="bold", size=50, color="#101418", align="center", runs=[{"text": "0 frames"}], scale=[{"t": 5600, "v": [1.0, 1.0]}] + repop[1:])]},
  {"name": "voice", "layer": 0, "elements": [
    el(id="voice", type="audio", start=1000, end=7672, source="character/voice/line-3.wav", source_start=0, source_end=6672, volume=1.0)]},
]
p["tracks"] = tracks
json.dump(p, open('h3.base.json', 'w'), indent=1)
out = subprocess.run([sys.executable, '.claude/skills/montagent-character/scripts/bake_rig.py', 'h3.base.json', 'character/rig-h3.json', 'owl.spec.json'], capture_output=True, text=True)
sys.stderr.write(out.stderr)
if out.returncode: sys.exit(out.returncode)
open('h3.json', 'w').write(out.stdout)
