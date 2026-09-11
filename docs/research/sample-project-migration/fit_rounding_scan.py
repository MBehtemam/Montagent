"""Re-executable evidence for ADR-0013's fitted-extent rule.

Usage:  python3 fit_rounding_scan.py

Asserts both directions of the decision, because either one alone is weak:

  1. The integer rule NEVER undercovers.  A fixture that passes proves the rule is safe
     on one geometry; only an exhaustive sweep proves it is safe in general.
  2. The float method DOES undercover.  The committed fixture cannot demonstrate this --
     1080/1536 = 45/64 is dyadic and exact in binary, so the fixture escapes the bug it
     is evidence for.  Without this scan the claim would be as uncheckable as ADR-0006's
     headline measurement, which is the whole complaint behind #46.

It also carries the stale-source case that ADR-0013's "Not settled here" hands to #48,
so that ticket inherits an artifact it can run rather than a paragraph it must trust.

Exits non-zero if any assertion fails.
"""
import math, sys

# --- the two methods ------------------------------------------------------------------

def cover_int(sw, sh, bw, bh):
    """ADR-0013. Driving axis by integer cross-multiplication, assigned verbatim;
    slack axis floors in integer arithmetic. No float anywhere."""
    if bw * sh >= bh * sw:
        return bw, (sh * bw) // sw
    return (sw * bh) // sh, bh

def cover_float(sw, sh, bw, bh):
    """The superseded implementation, kept so the defect stays reproducible."""
    f = max(bw / sw, bh / sh)
    return math.floor(sw * f), math.floor(sh * f)

def cover_int_ceil(sw, sh, bw, bh):
    """Ceil, in exact integer arithmetic -- to show that the 4.466% float-disagreement
    figure argues for INTEGER ARITHMETIC and says nothing about rounding DIRECTION."""
    if bw * sh >= bh * sw:
        return bw, -((-sh * bw) // sw)
    return -((-sw * bh) // sh), bh

# --- 1 + 2. the sweep -----------------------------------------------------------------

SW = range(50, 900, 7); SH = range(50, 900, 11)
BW = range(50, 900, 13); BH = range(50, 900, 17)

def sweep():
    n = viol = dis = 0
    first = None
    for sw in SW:
        for sh in SH:
            for bw in BW:
                for bh in BH:
                    n += 1
                    w, h = cover_int(sw, sh, bw, bh)
                    if w < bw or h < bh:
                        viol += 1
                    if (w, h) != cover_float(sw, sh, bw, bh):
                        dis += 1
                        if first is None:
                            first = (sw, sh, bw, bh, (w, h), cover_float(sw, sh, bw, bh))
    return n, viol, dis, first

print("ADR-0013 fitted-extent rule -- exhaustive sweep")
n, viol, dis, first = sweep()
print(f"  combinations tested ............ {n:,}")
print(f"  integer-rule undercoverage ..... {viol}")
print(f"  float method disagrees ......... {dis:,}  ({100*dis/n:.3f}%)")
if first:
    sw, sh, bw, bh, gi, gf = first
    print(f"  first disagreement ............. src {sw}x{sh} box {bw}x{bh}: "
          f"integer {gi} vs float {gf}")

assert viol == 0, f"ADR-0013's rule undercovered on {viol} combinations"
assert dis > 0, "the float defect did not reproduce -- the scan is no longer evidence"

# The named case from the ADR's prose, so the quoted number is checkable.
sw, sh, bw = 103, 200, 1920
assert math.floor(sw * (bw / sw)) == 1919, "the ULP case no longer reproduces"
print(f"  named ULP case ................. sw=103 bw=1920 -> float floors to "
      f"{math.floor(103 * (1920/103))}, integer gives 1920")

# --- 3. direction is not what the sweep decides ---------------------------------------

f_ = cover_int(1536, 2720, 1080, 1300)
c_ = cover_int_ceil(1536, 2720, 1080, 1300)
print("\nrounding DIRECTION is a tiebreak, not a consequence of integer arithmetic")
print(f"  floor, exact integer arithmetic  {f_}")
print(f"  ceil,  exact integer arithmetic  {c_}")
print("  both are exact, deterministic and float-free; both cover the box.")
assert f_ == (1080, 1912) and c_ == (1080, 1913)
assert f_[1] >= 1300 and c_[1] >= 1300, "both roundings must cover -- that is the point"

# --- 4. the fixture, and what ceil would have cost ------------------------------------

FIXTURE = [  # (id, source w, h, aperture w, h) -- the only two geometries in the corpus
    ("photo-05-intro", 1536, 2720, 1080, 1300), ("photo-05",      1536, 2720, 1080, 1300),
    ("photo-06",       1536, 2720, 1080, 1300), ("photo-07",      1536, 2720, 1080, 1300),
    ("photo-08",       1536, 2720, 1080, 1300), ("photo-05-quiz", 1536, 2720, 1080, 1300),
    ("photo-05-loop",  1536, 2720, 1080, 1300), ("handle-logo",    800,  800,   68,   68),
]
changed = sum(1 for _, sw, sh, bw, bh in FIXTURE
              if cover_int(sw, sh, bw, bh) != cover_int_ceil(sw, sh, bw, bh))
geoms = {(sw, sh, bw, bh) for _, sw, sh, bw, bh in FIXTURE}
needs_rounding = {g for g in geoms if (g[1] * g[2]) % g[0] or (g[0] * g[3]) % g[1]}
print(f"\nthe corpus is one geometry, counted seven times")
print(f"  distinct (source, box) geometries ...... {len(geoms)}")
print(f"  of those, needing a rounding decision .. {len(needs_rounding)}")
print(f"  elements ceil would change ............. {changed} of {len(FIXTURE)}")
assert changed == 7 and len(geoms) == 2 and len(needs_rounding) == 1

# --- 5. the case handed to #48 --------------------------------------------------------

print("\nstale-source case -- ADR-0013 'Not settled here', handed to #48")
sw, sh, bw, bh = 1536, 2200, 1080, 1300      # 06.png re-exported shorter
declared = (1080, 1912)                      # authored against the ORIGINAL 1536x2720
rule = cover_int(sw, sh, bw, bh)
covers = declared[0] >= bw and declared[1] >= bh
hf, vf = declared[0] / sw, declared[1] / sh
print(f"  source re-exported ............. {sw}x{sh}")
print(f"  declared rect (now stale) ...... {declared}")
print(f"  rule value ..................... {rule}   ({declared[1]-rule[1]} steps off)")
print(f"  covers the aperture? ........... {covers}   <- so no error fires")
print(f"  anisotropic distortion ......... {abs(vf/hf-1)*100:.1f}%")
sy = declared[1] / rule[1]
print(f"  legal deliberate spelling ...... rect {rule} + scale [1.0, {sy:.6f}]")
print(f"    ...which collides with Ken Burns: [1.08, {1.08*sy:.6f}] on 7 of 8 elements")
assert covers and rule == (1080, 1546) and declared[1] - rule[1] == 366

# --- 6. a step count is not scale-free either -----------------------------------------

a = abs(1914/1912 - 1) * 100
b = abs(70/68 - 1) * 100
print(f"\nthe same 2-step deviation, in one file: {a:.3f}% on 1912 vs {b:.3f}% on 68 "
      f"({b/a:.0f}x spread)")

print("\nall assertions passed")
sys.exit(0)
