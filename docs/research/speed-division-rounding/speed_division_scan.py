#!/usr/bin/env python3
"""Measures whether ADR-0020's `round((source_end - source_start) / speed)` diverges
between IEEE double and exact-decimal division, across a realistic (source_span, speed)
grid -- the `speed` analogue of #58's `line_height` measurement
(docs/adr/0028-text-block-arithmetic-is-exact-tenths.md), for #95.

Unlike `line_height`, ADR-0020 names no fixed decimal precision for `speed` -- the
fixture's own `speed: 0.645` (vo-sentence-05-b, #9) has three decimal digits, so this
script treats thousandths as the realistic default and also sweeps tenths and
hundredths for comparison (and to test whether ADR-0028's tenths-restriction mechanism,
which fixed `line_height`, also fixes this).

Method note, load-bearing for why this sweep is fast enough to be exhaustive rather
than sampled: `round_half_up(source_span / speed)` can only diverge between the two
arithmetic paths when the TRUE value of `source_span / speed` is an exact half-integer
tie (`n + 1/2`). Away from an exact tie, the two paths agree, because a finite decimal
literal's exact fractional part is a multiple of `1/N` where `N` is the literal's
reduced numerator (e.g. `0.560 == 14/25`, `N=14`) -- the nearest non-tie fractional value
is at least `1/(2N)` from `0.5`, several orders of magnitude larger than IEEE double's
~1e-16 relative rounding error at any realistic magnitude here. This is asserted and
spot-checked below, not just claimed -- see `assert_ties_are_necessary`.

Exits non-zero if any assertion fails, so the headline numbers stay checkable.
Run from the repo root: python3 docs/research/speed-division-rounding/speed_division_scan.py
"""
import math
import sys
import time
from decimal import Decimal
from fractions import Fraction


# --- the two arithmetic paths -----------------------------------------------------

def round_half_up_exact(fr: Fraction) -> int:
    """Exact round-half-up on a positive Fraction: floor(fr + 1/2), no float anywhere."""
    shifted = fr + Fraction(1, 2)
    return shifted.numerator // shifted.denominator  # exact floor division

def round_half_up_float(x: float) -> int:
    """What `(source_span as f64 / speed as f64).round()` computes in Rust -- and what
    round-half-away-from-zero looks like for positive x in any host language."""
    return math.floor(x + 0.5)

def compare(source_span: int, speed_decimal_str: str):
    """(exact_result, float_result, diverges) for one (source_span, speed) pair, both
    starting from the exact decimal literal an author would type in the project file."""
    speed_exact = Fraction(Decimal(speed_decimal_str))
    speed_float = float(speed_decimal_str)
    exact = round_half_up_exact(Fraction(source_span) / speed_exact)
    flt = round_half_up_float(source_span / speed_float)
    return exact, flt, exact != flt


# --- ADR-0020's own worked example --------------------------------------------------

def fixture_case():
    print("ADR-0020's own worked example (vo-sentence-05-b, #9): source_span=2184, speed=0.645")
    exact, flt, d = compare(2184, "0.645")
    print(f"  exact round-half-up .............. {exact}")
    print(f"  float round-half-up ............... {flt}")
    print(f"  diverges? .......................... {d}")
    assert exact == 3386, "ADR-0020's own cited result (3386) no longer reproduces"
    assert not d, "the fixture's own committed value would have been a silent bug"


# --- a small, realistic case that DOES diverge --------------------------------------

def minimal_counterexample():
    print("\nA minimal realistic counterexample: source_span=7ms, speed=0.560")
    exact, flt, d = compare(7, "0.560")
    ev = Fraction(7) / Fraction(Decimal("0.560"))
    print(f"  true value 7 / 0.560 .............. {ev} = {float(ev)} (an exact half-integer tie)")
    print(f"  exact round-half-up (ties go up) .. {exact}")
    print(f"  IEEE double: 7 / 0.560 in Python's repr: {7/0.560!r}")
    print(f"  float round-half-up ................ {flt}")
    assert exact == 13 and flt == 12 and d
    print("  => an element with source_end-source_start=7 and speed=0.560 gets end-start=12")
    print("     under naive double division, but the invariant's exact reading demands 13.")


# --- the necessary condition: divergence requires an exact half-integer tie --------

def assert_ties_are_necessary(sample_speeds, sample_spans):
    """Spot-check (not exhaustive -- exhaustive is the sweep below) that every
    divergence found coincides with an exact tie, confirming the claim in the module
    docstring rather than just asserting it."""
    checked = 0
    for speed_str in sample_speeds:
        speed_exact = Fraction(Decimal(speed_str))
        speed_float = float(speed_str)
        for span in sample_spans:
            checked += 1
            ev = Fraction(span) / speed_exact
            is_tie = (ev - ev.numerator // ev.denominator) == Fraction(1, 2)
            exact = round_half_up_exact(ev)
            flt = round_half_up_float(span / speed_float)
            if exact != flt:
                assert is_tie, (
                    f"found a divergence that is NOT an exact tie: span={span} "
                    f"speed={speed_str} exact_value={ev} -- the module's central claim is false"
                )
    return checked


# --- the exhaustive sweep ------------------------------------------------------------

def speed_strings(lo_c: int, hi_c: int, digits: int):
    """Decimal literals with `digits` fractional digits, every count from lo_c to hi_c
    inclusive, in units of 10**-digits."""
    fmt = f"{{:.{digits}f}}"
    return [fmt.format(c / (10 ** digits)) for c in range(lo_c, hi_c + 1)]

def sweep(speeds, span_max, label):
    """Exhaustive over every (speed, source_span) in the grid -- fast because the
    per-pair check reduces to plain-integer and plain-float arithmetic (no Fraction
    object per pair), verified against the Fraction-based `compare()` separately."""
    n = dis = 0
    examples = []
    t0 = time.time()
    for speed_str in speeds:
        speed_exact = Fraction(Decimal(speed_str))
        N, M = speed_exact.numerator, speed_exact.denominator  # speed == N/M, reduced
        speed_float = float(speed_str)
        for s in range(1, span_max + 1):
            n += 1
            exact = (2 * s * M + N) // (2 * N)  # floor(s*M/N + 1/2), exact integer math
            flt = math.floor(s / speed_float + 0.5)
            if exact != flt:
                dis += 1
                if len(examples) < 8:
                    examples.append((s, speed_str, exact, flt))
    dt = time.time() - t0
    rate = 100 * dis / n
    print(f"\n{label}")
    print(f"  combinations tested ............ {n:,}  ({dt:.1f}s)")
    print(f"  divergences ..................... {dis:,}  ({rate:.4f}%)")
    if examples:
        print(f"  first divergences: {examples[:4]}")
    return n, dis, rate


def main():
    print("ADR-0020 speed-division rounding -- exhaustive-grid measurement for #95\n")

    fixture_case()
    minimal_counterexample()

    print("\nSpot-check: every divergence found coincides with an exact half-integer tie")
    checked = assert_ties_are_necessary(
        speed_strings(100, 400, 2), range(1, 2000))
    print(f"  {checked:,} (speed, source_span) pairs checked, no counterexample to the claim")

    # Realistic domain: source_span up to 60s (a long single clip/narration line, ADR-0005
    # integer ms), speed 0.100 .. 4.000 at the fixture's own thousandths precision.
    n1, d1, r1 = sweep(speed_strings(100, 4000, 3), 60_000,
                        "Sweep A: speed at thousandths (0.100 .. 4.000), source_span 1..60000ms (60s)")

    # Same domain at hundredths and tenths, to see whether coarser precision changes the
    # picture (it changes WHICH speeds can tie -- see FINDINGS -- but not that some do).
    n2, d2, r2 = sweep(speed_strings(10, 400, 2), 60_000,
                        "Sweep B: speed at hundredths (0.10 .. 4.00), source_span 1..60000ms")
    n3, d3, r3 = sweep(speed_strings(1, 40, 1), 60_000,
                        "Sweep C: speed restricted to exact tenths (0.1 .. 4.0) -- ADR-0028's mechanism")

    # A longer duration range, to confirm the rate is a property of the grid density,
    # not an artifact of a short span cutoff.
    n4, d4, r4 = sweep(speed_strings(100, 4000, 3), 300_000,
                        "Sweep D: same thousandths speeds, source_span 1..300000ms (5 min)")

    print("\n== summary ==")
    for label, (n, d, r) in [("A thousandths/60s", (n1, d1, r1)),
                              ("B hundredths/60s", (n2, d2, r2)),
                              ("C tenths/60s", (n3, d3, r3)),
                              ("D thousandths/5min", (n4, d4, r4))]:
        print(f"  {label:20s}  {n:>13,} combinations  {d:>9,} divergences  {r:7.4f}%")

    print("\n== does ADR-0028's tenths-restriction mechanism fix this? ==")
    if d3 == 0:
        print(f"  Sweep C measured ZERO divergences for exact-tenths speeds, up to 60s spans.")
        print("  This is NOT the same guarantee ADR-0028 gives line_height, and does not")
        print("  generalise: exact ties still mathematically exist at tenths (e.g. speed=0.4,")
        print("  source_span odd -> source_span/0.4 is always an exact X.5) -- unlike")
        print("  line_height, where the fix makes the underlying arithmetic EXACT so no tie")
        print("  can ever be perturbed. Here the tie exists but this script measured that its")
        print("  IEEE evaluation happens not to cross it in the tested range (spot-checked to")
        print("  50,000,000ms/~14h below with the same result) -- an empirical non-occurrence,")
        print("  not a proof, because division by a non-power-of-two decimal is never exact.")
        print("  It is also moot: ADR-0020 imposes no precision limit on speed, and the")
        print("  fixture's own speed (0.645) already needs THREE decimal digits -- at that")
        print("  precision (Sweep A) and at hundredths (Sweep B), real divergence is measured.")
    else:
        print(f"  Sweep C (speed restricted to exact tenths) still diverges on {r3:.4f}% of cases.")
        print("  CONFIRMED: restricting speed's precision does not make the DIVISION exact,")
        print("  unlike line_height's multiplication.")

    print("\n== exact-rational speed (ADR-0023's `par` precedent) closes it by construction ==")
    # If speed is declared as [num, den] and the invariant is evaluated in Fraction/
    # rational arithmetic throughout, there is no second, lossy (float) representation
    # to disagree with -- demonstrated on the exact tie cases the sweeps above found.
    for span, speed_str in [(7, "0.560"), (14, "1.120"), (21, "0.336")]:
        num, den = Fraction(Decimal(speed_str)).as_integer_ratio()
        exact_rational_result = round_half_up_exact(Fraction(span) / Fraction(num, den))
        # this IS the exact result already computed above -- there is no separate
        # "rational-path" float detour, which is exactly the point.
        assert exact_rational_result == round_half_up_exact(Fraction(span) / Fraction(Decimal(speed_str)))
    print("  speed=[num,den] evaluated as Fraction throughout reproduces the exact column")
    print("  of every sweep above by construction -- there is no float conversion step left")
    print("  to introduce the divergence.")

    # headline assertions -- keep the numbers in FINDINGS.md checkable
    assert d1 > 0, "thousandths sweep found no divergence -- re-check the method"
    assert d2 > 0, "hundredths sweep found no divergence -- re-check the method"
    assert d3 == 0, ("tenths-restricted sweep found a divergence -- FINDINGS.md's tenths-are-"
                      "empirically-clean claim in this domain no longer reproduces")
    assert 0.001 < r1 < 0.01, f"headline rate ({r1:.4f}%) moved outside the range FINDINGS.md expects"

    # spot-check that tenths' zero-divergence result is not an artifact of the 60s cutoff
    tenths_even = ["0.4", "0.8", "1.2", "1.6", "2.0", "2.4", "2.8", "3.2", "3.6", "4.0"]
    n5, d5, r5 = sweep(tenths_even, 50_000_000,
                        "Sweep E: the tie-capable exact-tenths speeds, source_span up to "
                        "50,000,000ms (~14h) -- is Sweep C's zero-divergence result a 60s artifact?")
    assert d5 == 0, "tenths diverged at a larger span range -- the 'empirically clean' claim was an artifact of the 60s cutoff"

    print("\nall assertions passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
