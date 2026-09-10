"""Migrate the #9 prototype project onto accepted ADRs 0001-0012.

Reads old.json (the prototype, on branch prototype/sample-project-file) and emits
the migrated file. Every transformation below is traceable to a specific ADR; the
script exists so the mapping is checkable rather than asserted.
"""
import json, collections, decimal, sys

SRC_DIMS = {  # ffprobe, front-loaded per source (ADR-0005: probe is per-source)
    "images/05.png": (1536, 2720), "images/06.png": (1536, 2720),
    "images/07.png": (1536, 2720), "images/08.png": (1536, 2720),
    "brand/logo-en.png": (800, 800),
}

def r(x):  # ADR-0012: ties away from zero
    return int(decimal.Decimal(x).quantize(0, rounding=decimal.ROUND_HALF_UP))

def cover(sw, sh, bw, bh):
    """ADR-0013: fitted extents floor, in exact integer arithmetic.

    Exact cover for the photos is 1080 x 1912.5, so a rounding rule is unavoidable and
    ADR-0012 declined to publish one ("The renderer must publish a sampling rule").
    ADR-0013 supplies it and scopes ADR-0012's "ties away from zero" back to x/y
    interpolation residuals under SPLIT, where it belongs -- so the element published in
    ADR-0012 (1912) was correct and its stated rule did not reach here. See README D1.

    Three clauses, and the integer ones are load-bearing rather than stylistic:
      - the driving axis is chosen by integer cross-multiplication, never by comparing
        bw/sw against bh/sh as floats;
      - the driving axis takes the box dimension VERBATIM -- it is exact by construction
        and must not survive a round trip through a float;
      - the slack axis floors in integer arithmetic.

    The previous implementation was `math.floor(sw * f)` with `f = max(bw/sw, bh/sh)`.
    That loses the driving axis to a ULP -- sw=103, bw=1920 gives 1919.9999999999998,
    floors to 1919, and cover fails by a visible pixel on the axis that is exact by
    construction. It disagrees with this function on 4.466% of 31,402,800 combinations;
    see fit_rounding_scan.py. The fixture escaped it only because 1080/1536 = 45/64 is
    dyadic, so this correction changes zero bytes of the committed file.
    """
    if bw * sh >= bh * sw:                 # width drives
        return bw, (sh * bw) // sw
    return (sw * bh) // sh, bh

# ADR-0012: text gains a literal box. Only 7 of 22 have a rect behind them to measure.
CARD = (984, 169)                     # card-05..quiz: [48,1453,984,169]
TEXT_BOX = {
    "sentence-05": CARD, "sentence-06": CARD, "sentence-07": CARD,
    "sentence-08": CARD, "sentence-quiz": CARD,
    "chip-text":   (420 - 182, 84),     # inside chip-panel   [48,88,372,84]
    "handle-text": (1032 - 560, 84),    # inside handle-panel [438,88,594,84]
}
MEASURED_BOX = set(TEXT_BOX)
DESIGN_WIDTH = 984                    # every panel and card in the ASS runs x=48..1032

# ASS alignment recovered from reference/subtitles/*.ass
ORIGIN_FROM_ALIGN = {"center": "center", "left": "center-left"}   # \an5, \an4
ALIGN_MIGRATE     = {"center": "center", "left": "start"}         # ADR-0007 start/center/end

KEY_ORDER = ["id","type","group","start","end","source","source_start","source_end",
             "speed","x","y","origin","width","height","fit","gravity","clip","mask",
             "fill","font","size","line_height","color","align","runs","scale"]

def ordered(d):
    out = collections.OrderedDict()
    for k in KEY_ORDER:
        if k in d: out[k] = d[k]
    unknown = [k for k in d if k not in KEY_ORDER]
    if unknown: raise SystemExit(f"unhandled key(s) {unknown} on {d.get('id')}")
    return out

def migrate(el, ease):
    e = dict(el); t = e["type"]; out = {}
    for k in ("id","type","group","start","end","source","source_start","source_end","speed"):
        if k in e: out[k] = e.pop(k)

    if t == "audio":
        # ADR-0012: a transform field on audio is a schema error, so nothing is added.
        pass

    elif t in ("image", "rect"):
        x, y, w, h = e.pop("box")
        out["x"], out["y"], out["origin"] = x, y, "top-left"
        if t == "image":
            sw, sh = SRC_DIMS[out["source"]]
            dw, dh = cover(sw, sh, w, h)          # the drawn rect, ADR-0012
            out["width"], out["height"] = dw, dh
            out["fit"] = e.pop("fit")
            out["gravity"] = e.pop("align")       # ADR-0012: align -> gravity on images
            out["clip"] = [x, y, w, h]            # the aperture the old box was doing silently
            if "mask" in e: out["mask"] = e.pop("mask")   # still #22, unchanged
        else:
            out["width"], out["height"] = w, h
            out["fill"] = e.pop("fill")           # still #13, unchanged

    elif t == "text":
        assert e.pop("font") == "SF Pro Rounded" and e.pop("weight") == "bold"
        text = e.pop("text"); al = e.pop("align"); size = e.pop("size"); lh = e.pop("line_height")
        out["x"], out["y"] = e.pop("x"), e.pop("y")
        out["origin"] = ORIGIN_FROM_ALIGN[al]
        if out["id"] in MEASURED_BOX:
            bw, bh = TEXT_BOX[out["id"]]
        else:
            bw, bh = DESIGN_WIDTH, None
        # ADR-0007's own block formula: lines x max size x line_height.
        if bh is None:
            # No rect behind this element, and the fixture has no other source for a
            # height. ADR-0007's own block formula is the only value derivable from the
            # document -- which makes ADR-0006's overflow check tautological here.
            # See FINDINGS D2.
            bh = r((text.count("\n") + 1) * size * lh)
        out["width"], out["height"] = bw, bh
        out["font"] = "brand"                     # ADR-0007 declared fonts table
        out["size"], out["line_height"], out["color"] = size, lh, e.pop("color")
        out["align"] = ALIGN_MIGRATE[al]
        out["runs"] = [{"text": text}]            # ADR-0007: runs, always an array

    if "scale" in e:  # ADR-0012 keyframe records; v is always a pair, never a scalar
        recs = []
        for i, (kt, kv) in enumerate(e.pop("scale")):
            rec = {"t": kt, "v": [kv, kv]}
            if i: rec["ease"] = ease             # ease on the first record is a schema error
            recs.append(rec)
        out["scale"] = recs
    leftover = [k for k in e if k not in ("box",)]
    if leftover: raise SystemExit(f"unconsumed {leftover} on {out['id']}")
    return ordered(out)

def main(ease):
    old = json.load(open(sys.argv[2] if len(sys.argv) > 2 else "old.json"))
    n = 0
    tracks = []
    for tr in old["tracks"]:
        els = [migrate(el, ease) for el in tr["elements"]]
        n += len(els)
        tracks.append(collections.OrderedDict(
            [("name", tr["name"]), ("layer", tr["layer"]), ("elements", els)]))
    doc = collections.OrderedDict()
    for k in ("frame","fps","background","duration","output"):
        doc[k] = old[k]
    doc["fonts"] = {"brand": [{"file": "fonts/SFProRounded-Bold.ttf"}]}
    doc["tracks"] = tracks

    # ADR-0005/0007 writing convention: one element per line, stable key order.
    lines = ["{"]
    head = [f'  {json.dumps(k)}: {json.dumps(doc[k])}' for k in
            ("frame","fps","background","duration","output","fonts")]
    lines.append(",\n".join(head) + ",")
    lines.append('  "tracks": [')
    tblocks = []
    for tr in tracks:
        b  = "    {\n"
        b += f'      "name": {json.dumps(tr["name"])},\n      "layer": {tr["layer"]},\n'
        b += '      "elements": [\n'
        b += ",\n".join("        " + json.dumps(el, ensure_ascii=False, separators=(",", ":"))
                        for el in tr["elements"])
        b += "\n      ]\n    }"
        tblocks.append(b)
    lines.append(",\n".join(tblocks))
    lines.append("  ]")
    lines.append("}")
    open(sys.argv[3] if len(sys.argv) > 3 else "new.json","w").write("\n".join(lines) + "\n")
    print(f"{len(tracks)} tracks, {n} elements", file=sys.stderr)

main(sys.argv[1] if len(sys.argv) > 1 else "linear")
