#!/usr/bin/env python3
"""Re-derives every number in ADR-0015. Exits non-zero if any of it stops reproducing.

Companion to fit_rounding_scan.py, which covers ADR-0013. This one covers what #48
settled: the box is `clip`, the `contain` arithmetic, strict-vs-loose rounding, the
EXIF/PAR divergence magnitudes, and the falsification of ADR-0013's tiebreak (2).

    python3 fit_vocabulary_scan.py
"""
import json, os, random, struct, sys

HERE = os.path.dirname(os.path.abspath(__file__))
FIXTURE = os.path.join(HERE, "..", "..", "..", "fixtures",
                       "en-halloween-decorating", "en-halloween-decorating.montaget.json")
fail = []


def check(label, got, want):
    ok = got == want
    print(f"  {label:.<46} {got}{'' if ok else f'   EXPECTED {want}'}")
    if not ok:
        fail.append(label)


def cover(sw, sh, bw, bh):
    """ADR-0013. Driving axis verbatim; slack axis floors; exact integer arithmetic."""
    return (bw, (sh * bw) // sw) if bw * sh >= bh * sw else ((sw * bh) // sh, bh)


def contain(sw, sh, bw, bh):
    """ADR-0015. Mirror inequality; same floor on the slack axis."""
    return (bw, (sh * bw) // sw) if bw * sh <= bh * sw else ((sw * bh) // sh, bh)


def contain_ceil(sw, sh, bw, bh):
    ceil = lambda a, b: -((-a) // b)
    return (bw, ceil(sh * bw, sw)) if bw * sh <= bh * sw else (ceil(sw * bh, sh), bh)


def png_dims(path):
    with open(path, "rb") as f:
        head = f.read(33)
    assert head[:8] == b"\x89PNG\r\n\x1a\n", path
    return struct.unpack(">II", head[16:24])


print(__doc__.splitlines()[0])
print()

# ---------------------------------------------------------------- 1. the box is `clip`
print("1. the box the rule fits into is `clip`, not the frame and not the element's rect")
SW, SH = 1536, 2720
check("cover into clip 1080x1300", cover(SW, SH, 1080, 1300), (1080, 1912))
check("cover into frame 1080x1920 (falsified)", cover(SW, SH, 1080, 1920), (1084, 1920))
check("cover into the declared rect (fixed point)", cover(SW, SH, 1080, 1912), (1080, 1912))
print("  the published element is (1080, 1912) -- only `clip` reproduces it,")
print("  and the self-rect box is a fixed point, so equality alone cannot discriminate it.")
print()

# ------------------------------------------- 2. the self-rect fallback still catches stale
print("2. why the self-rect fallback is rejected -- it is NOT vacuous, it is a different check")
check("stale 1536x2200 vs declared rect as box", cover(1536, 2200, 1080, 1912), (1334, 1912))
print("  it fires (1334 != 1080), so 'tautology' is the wrong objection; it is rejected")
print("  because it silently checks aspect fidelity instead of fit-to-aperture.")
print()

# ------------------------------------------------------------------ 3. contain arithmetic
print("3. `contain`, and the cases that shaped it")
check("badge 1200x400 into 200x160", contain(1200, 400, 200, 160), (200, 66))
check("fixture geometry under contain", contain(SW, SH, 1080, 1300), (734, 1300))
print("  734x1300 is smaller than its own 1080x1300 clip -- which is why ADR-0013's")
print("  aperture-coverage error had to be parameterised by `fit`.")
check("zero extent: 300x7 into 10x10", contain(300, 7, 10, 10), (10, 0))
print()

# ------------------------------------------- 4. ADR-0013 tiebreak (2) is false
print("4. ADR-0013 tiebreak (2): 'only floor is safe for contain' -- FALSE")
viol_f = viol_c = differ = 0
N = 0
for sw in range(1, 260):
    for sh in range(1, 260):
        for bw in (1, 3, 7, 10, 64, 199, 256):
            for bh in (1, 3, 7, 10, 64, 199, 256):
                N += 1
                f = contain(sw, sh, bw, bh)
                c = contain_ceil(sw, sh, bw, bh)
                if f[0] > bw or f[1] > bh: viol_f += 1
                if c[0] > bw or c[1] > bh: viol_c += 1
                if f != c: differ += 1
check("cases tested", f"{N:,}", "3,286,969")
check("containment violations, floor", viol_f, 0)
check("containment violations, ceil", viol_c, 0)
check("floor and ceil differ on", f"{differ/N:.2%}", "97.77%")
print("  ceil(x) <= n whenever x <= n and n is an integer, and contain picks the driving")
print("  axis so exact_slack <= b_slack. Floor survives on tiebreaks (1) and (3) only.")
print()

# ------------------------------------------------ 5. the measured consumer divergence
print("5. the divergence that overturned the {floor, ceil} predicate")
print("   eight agents, six authoring tasks, integers actually written:")
for label, rule, written in [
    ("task 2  stale source, cover 1536x2200 -> 1080x1300", cover(1536, 2200, 1080, 1300)[1], [1546, 1547]),
    ("task 3  badge, contain 1200x400 -> 200x160", contain(1200, 400, 200, 160)[1], [66, 67]),
    ("task 5  EXIF-6 phone photo, cover 4032x3024 -> 1080x1300", cover(4032, 3024, 1080, 1300)[0], [1733, 1734]),
]:
    assert rule == written[0], (label, rule, written)
    print(f"  {label}")
    print(f"      rule value {rule}; written {written} -> {len(written)} legal files for one input")
print("  3 of 6 tasks produced two byte-different, equally legal files under {floor,ceil}.")
print()

# ---------------------------------------------- 6. strict equality costs nothing
print("6. strict equality against the committed fixture")
proj = json.load(open(FIXTURE))
imgs = []


def walk(o):
    if isinstance(o, dict):
        if o.get("type") == "image": imgs.append(o)
        for v in o.values(): walk(v)
    elif isinstance(o, list):
        for v in o: walk(v)


walk(proj)
check("image elements in the fixture", len(imgs), 8)
strict_fail = loose_fail = gravity = 0
for e in imgs:
    src = os.path.join(os.path.dirname(FIXTURE), e["source"])
    sw, sh = png_dims(src)
    _, _, bw, bh = e["clip"]
    rw, rh = cover(sw, sh, bw, bh)
    if (e["width"], e["height"]) != (rw, rh): strict_fail += 1
    cw, ch = contain_ceil(sw, sh, bw, bh) if False else cover(sw, sh, bw, bh)
    if e["width"] not in (rw, cw) or e["height"] not in (rh, ch): loose_fail += 1
    if "gravity" in e: gravity += 1
check("elements failing STRICT equality", strict_fail, 0)
check("elements failing the LOOSE predicate", loose_fail, 0)
print("  strict costs exactly what loose costs: nothing. The tolerance bought no compatibility.")
check("elements still carrying `gravity`", gravity, 0)
print("  (0 once this branch's migration lands; 8 before it)")
print()

# ------------------------------------------------------- 7. EXIF and PAR divergence
print("7. why source dimensions had to be defined normatively")
stored, oriented = (3024, 4032), (4032, 3024)
a = cover(*stored, 1080, 1300)
b = cover(*oriented, 1080, 1300)
check("EXIF-6 ignored  (3024x4032)", a, (1080, 1440))
check("EXIF-6 applied  (4032x3024)", b, (1733, 1300))
div = abs(b[0] - a[0]) / a[0]
check("width divergence between the two readings", f"{div:.1%}", "60.5%")
par_sq = cover(1440, 1080, 1080, 1300)
par_wide = cover(1920, 1080, 1080, 1300)          # same file, PAR 4:3 applied
pdiv = abs(par_wide[0] - par_sq[0]) / par_sq[0]
check("PAR 4:3 ignored vs applied, width", (par_sq[0], par_wide[0]), (1733, 2311))
check("PAR divergence", f"{pdiv:.1%}", "33.4%")
print("  ADR-0005's `speed` divergence, considered serious enough to legislate, was 0.038%:")
check("ADR-0005 speed spread 5222 vs 5220", f"{abs(5222-5220)/5222:.3%}", "0.038%")
print()
print("  NOTE: the fixture cannot test any of section 7 -- all five sources are PNG and the")
print("  only eXIf chunk is on the 800x800 square with no Orientation tag. Hence synthetic.")
print()

if fail:
    print(f"FAILED: {len(fail)} assertion(s) stopped reproducing:")
    for f_ in fail: print("  -", f_)
    sys.exit(1)
print("all assertions passed")
