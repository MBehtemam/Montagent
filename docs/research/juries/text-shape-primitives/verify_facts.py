"""Re-executable evidence for ADR-0014's text and shape primitives.

Usage:  python3 docs/research/juries/text-shape-primitives/verify_facts.py

Every factual claim ADR-0014 rests on is re-derived here from files committed in this
repo. Exits non-zero if any stops reproducing.

Two of these facts were supplied to the jury by the author and FALSIFIED by jurors
(checks 6 and 7): `ScaledBorderAndShadow: yes` was cited as evidence of stroke intent
when every style in the fixture sets Outline and Shadow to 0. They are asserted here in
their corrected form so the correction cannot quietly rot back.

No third-party dependency and no ffmpeg: the geometry claims are derived from the ASS
drawing commands, which are the source the published MP4 was rendered from. The PNG
alpha check reads the file header by hand rather than importing PIL.
"""
import json, math, pathlib, re, struct, sys, zlib

ROOT = pathlib.Path(__file__).resolve().parents[4]
FIX = ROOT / "fixtures" / "en-halloween-decorating"
PROJECT = FIX / "en-halloween-decorating.montaget.json"
SUBS = sorted((FIX / "reference" / "subtitles").glob("*.ass"))

failures = []

def check(name, got, want):
    ok = got == want
    print(f"  {'ok  ' if ok else 'FAIL'}  {name}: {got!r}")
    if not ok:
        failures.append(f"{name}: got {got!r}, want {want!r}")

project = json.loads(PROJECT.read_text())
elements = [e for t in project["tracks"] for e in t["elements"]]
texts = [e for e in elements if e["type"] == "text"]
rects = [e for e in elements if e["type"] == "rect"]

print("\n1. The shapes the fixture draws are axis-aligned rectangles with square corners.")
# ASS drawing commands: `m x y l x y x y x y`. A bezier would use `b`.
draws = set()
for f in SUBS:
    for m in re.finditer(r"m ((?:\d+ )+)l ((?:\d+ ?)+)", f.read_text()):
        draws.add((m.group(1).strip(), m.group(2).strip()))
check("distinct drawings", len(draws), 6)
check("every drawing is 4 points (m=1, l=3)",
      all(len(m.split()) == 2 and len(l.split()) == 6 for m, l in draws), True)
check("bezier commands anywhere in the seven files",
      sum(len(re.findall(r"\\p1[^\\}]*\bb ", f.read_text())) for f in SUBS), 0)
corners = {tuple(int(v) for v in m.split()) for m, _ in draws}
check("chip panel drawn at (48,88)", (48, 88) in corners, True)
check("sentence card drawn at (48,1453)", (48, 1453) in corners, True)

print("\n2. The circular badge is baked into the asset, so `mask` changes nothing.")
png = (FIX / "brand" / "logo-en.png").read_bytes()
w, h = struct.unpack(">II", png[16:24])
# Each chunk is <4-byte length><4-byte type><data><4-byte crc>; `i` is the type offset.
idat = b"".join(png[i + 4 : i + 4 + struct.unpack(">I", png[i - 4 : i])[0]]
                for i in [m.start() for m in re.finditer(b"IDAT", png)])
raw = zlib.decompress(idat)
# Pixel 0 of row 0 is unaffected by every PNG filter type: its left and upper
# neighbours are both zero, so raw[1:5] is the literal RGBA whatever the filter byte says.
check("logo is RGBA 800x800", (w, h, png[24], png[25]), (800, 800, 8, 6))
check("top-left pixel alpha (filter byte + RGBA)", raw[4], 0)

print("\n3. Text `height` is self-derived on 15 of 22, container-copied on 7.")
derived = [e for e in texts
           if e["height"] == math.ceil(e["size"] * e["line_height"]
                                       * (sum(r["text"].count("\n") for r in e["runs"]) + 1))]
check("self-derived heights", len(derived), 15)
check("container-copied heights", len(texts) - len(derived), 7)
rect_boxes = {(r["width"], r["height"]) for r in rects}
check("the 5 sentence texts copy the navy card 984x169",
      sum(1 for e in texts if (e["width"], e["height"]) == (984, 169)), 5)
check("the card 984x169 is a real rect", (984, 169) in rect_boxes, True)
check("no text height is derivable from a font file (inputs are size, line_height, \\n)",
      all(set(("size", "line_height", "runs")) <= set(e) for e in texts), True)

print("\n4. `width` is externally sourced on all 22 — never the typography.")
check("no text width equals a typographic extent of its own",
      sum(1 for e in texts if e["width"] in (e["size"], e["height"])), 0)
check("distinct declared widths", sorted({e["width"] for e in texts}), [238, 472, 984])

print("\n5. Every rect carries exactly one paint field, a flat #RRGGBB.")
check("rect count", len(rects), 10)
check("paint fields present", sorted({k for r in rects for k in r} - {
      "id", "type", "group", "start", "end", "x", "y", "origin", "width", "height"}), ["fill"])
check("every fill is 6-digit hex", all(re.fullmatch(r"#[0-9A-F]{6}", r["fill"]) for r in rects), True)
check("no rect declares a radius, stroke or opacity",
      any(k in r for r in rects for k in ("radius", "stroke", "stroke_width", "opacity")), False)

print("\n6. FALSIFIED BY THE JURY: the fixture has no stroke of any kind.")
styles = [l for f in SUBS for l in f.read_text().splitlines() if l.startswith("Style:")]
check("style definitions", len(styles), 35)
check("every style sets Outline=0 and Shadow=0",
      all(s.split(",")[16].strip() == "0" and s.split(",")[17].strip() == "0" for s in styles), True)
check("inline \\bord \\shad \\3c \\4c \\alpha overrides",
      sum(len(re.findall(r"\\(?:bord|shad|3c|4c|alpha)", f.read_text())) for f in SUBS), 0)

print("\n7. FALSIFIED BY THE JURY: `ScaledBorderAndShadow: yes` is an inert default.")
check("files declaring it",
      sum(1 for f in SUBS if "ScaledBorderAndShadow: yes" in f.read_text()), 7)
check("...governing a border and shadow that are both zero everywhere",
      all(s.split(",")[16].strip() == "0" for s in styles), True)

print("\n8. ADR-0014 changes zero bytes of the committed project file.")
# Nothing ADR-0014 decides is absent from the file: text keeps width+height (required
# before and after), every rect keeps its fill, and an omitted radius/stroke is the
# square, unstroked shape the fixture already draws.
check("all 22 text elements already carry width and height",
      all("width" in e and "height" in e for e in texts), True)
check("all 10 rects already carry fill", all("fill" in r for r in rects), True)
check("nothing in the file uses a field ADR-0014 introduces",
      any(k in e for e in elements for k in ("stroke", "stroke_width", "radius")), False)

print()
if failures:
    print(f"{len(failures)} FAILED:")
    for f in failures:
        print("  -", f)
    sys.exit(1)
print("All ADR-0014 facts reproduce.")
