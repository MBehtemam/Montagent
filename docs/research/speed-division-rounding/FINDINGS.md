# `speed`'s `round(source_span / speed)`: measured, and it diverges — [#95](https://github.com/MBehtemam/Montaget/issues/95)

Resolves #95, graduated from [ADR-0028](../../adr/0028-text-block-arithmetic-is-exact-tenths.md)'s
"Not settled here" clause. `speed_division_scan.py` re-derives every number below and exits
non-zero if any of it stops reproducing. Run it from the repo root.

## The headline

**It diverges, at a measurable rate, and a 7ms/0.560 element is enough to show it:**

```
source_span = 7ms, speed = 0.560
exact:  7 / 0.560 = 25/2 = 12.5 exactly -> round-half-up -> 13
double: 7 / 0.560 = 12.499999999999998   -> round-half-up -> 12
```

`validate` would accept a project where `end - start` is `12`, but the exact reading of
ADR-0020's own invariant demands `13`. This is the same shape of bug ADR-0028 found for
`line_height` (`ceil` disagreeing with exact decimal by one pixel), on the field ADR-0028
scoped out.

Across a full grid at the fixture's own precision (`speed: 0.645`, three decimal digits,
per #9's `vo-sentence-05-b`):

| speed precision | grid (speed x source_span, 1..60000ms) | divergences | rate |
| --- | --- | --- | --- |
| thousandths (0.100-4.000) | 234,060,000 | 9,687 | **0.0041%** |
| hundredths (0.10-4.00) | 23,460,000 | 7,074 | **0.0302%** |
| exact tenths (0.1-4.0) | 2,400,000 | 0 | 0.0000% (see below -- not a fix) |

The rate is stable as the source-span range grows: sweeping the same thousandths grid out
to 300,000ms (5 minutes) gives 51,086 / 1,170,300,000 = 0.0044%, matching the 60-second
figure. This is a real, reproducible exposure, not a sampling artifact -- smaller than
`line_height`'s 6.35% (ADR-0028) and `fit`'s 4.466% (ADR-0013), but nonzero, and #95 asked
only whether it manifests, not how it compares in size.

## Why it diverges: the mechanism is different from `line_height`'s

`line_height`'s bug fires whenever the float product lands close to an *integer*, because
`ceil`'s boundary is at every integer -- which is common. `round-half-up`'s boundary is at
every *half-integer*, which is rarer: **a divergence can only occur when the true value of
`source_span / speed` is an exact tie (`n + 1/2`).** Away from an exact tie, IEEE double's
relative error (~1e-16) is many orders of magnitude smaller than the distance to the nearest
tie for any realistic precision, so the two paths agree. `speed_division_scan.py`'s
`assert_ties_are_necessary` spot-checks this against 601,699 sampled pairs with no
counterexample found, and it is why the exhaustive sweep can be exhaustive at all: only
ties are candidates, and IEEE division is correctly-rounded, so whether a given tie survives
is a fixed, checkable fact per `(speed, source_span mod N)` pair, not something that needs
sampling to approximate.

An exact tie exists only when `speed`'s reduced numerator is **even** -- e.g. `0.560 = 14/25`
(numerator 14, even) has one every 14 spans; `0.645 = 129/200` (numerator 129, odd) has
**no exact ties at all**, which is why ADR-0020's own worked example (`2184 / 0.645`) does
not diverge -- it was never a candidate, not evidence the invariant is safe in general.
Whether a tie that exists actually flips depends on which side of the tie IEEE
round-to-nearest lands the corrected quotient, which varies per `source_span` even for the
same `speed` (measured: 5,395 of 7,143 ties diverge for `speed=0.560` over spans 1-100,000 --
75.5%, not 100% and not 0%).

## Does ADR-0028's tenths-restriction mechanism transfer? No -- for two independent reasons

The ticket already argued this from the algebra (`1/0.3` has no finite decimal expansion at
any precision); this script confirms it, and adds a fact the algebra alone doesn't give:

**1. It isn't a structural fix here, unlike for `line_height`.** ADR-0028's restriction
works because `n/10 x n/10 x integer` stays an *exact* rational at every step -- there is no
longer any tie for float rounding to perturb, because there is no longer any float rounding
at all. Restricting `speed` to tenths does not do the analogous thing: `source_span / (n/10)`
is `source_span x 10 / n`, and whether **that** result is itself a tie depends on the parity
of `n`, exactly as before restriction -- tenths do not remove ties, they just change which
values can produce them (`speed = 0.4 = 2/5` still has an even numerator and still ties on
every odd `source_span`).

**2. Measured anyway, restricting `speed` to the ten tie-capable exact-tenths values
(`0.4, 0.8, 1.2, 1.6, 2.0, 2.4, 2.8, 3.2, 3.6, 4.0`) produced *zero* divergences** -- checked
up to 60,000ms, and re-checked up to 50,000,000ms (~14 hours) to rule out a range artifact
(Sweep E in the script). **This is a real, measured fact, and it is not a guarantee.** Two
of the ten (`2.0`, `4.0`) are exact powers of two, where binary division is exact by
construction. The other eight are not, and their zero-divergence result is an empirical
non-occurrence within the tested domain, not a proof -- nothing in the tenths restriction
makes the division exact the way it makes `line_height`'s multiplication exact, so a
counterexample at some untested magnitude cannot be ruled out the way it can for
`line_height`.

**It is moot regardless: ADR-0020 imposes no precision limit on `speed`, and the fixture's
own `speed: 0.645` already needs three decimal digits.** A tenths restriction would reject
the committed fixture value that motivated ADR-0020 in the first place. The realistic
domain is thousandths (or finer), where the measured rate is 0.0041-0.0302% and nonzero.

## What would fix it: exact-rational `speed`, per ADR-0023's `par` precedent

[ADR-0023](../../adr/0023-video-source-dimensions-par-and-rotation.md) already solved this
exact problem for a different field: `par` is declared as `[num, den]`, exact integers, and
consumed as an exact rational rather than a decimal. Applied to `speed`:

- Declare `speed` as `[num, den]` (or keep the decimal spelling for authoring and parse it
  to an exact `Fraction`/rational internally -- either is fine as long as the *invariant* is
  evaluated in rational arithmetic throughout, never converted to `f64` at any step).
- Evaluate `round_half_up(Fraction(source_span) / Fraction(num, den))` with exact integer
  arithmetic (`(2*source_span*den + num) // (2*num)`, mirroring ADR-0028's
  `(size*n*line_count + 9) // 10` technique).

This closes the divergence **by construction, not by measurement**: there is no second,
lossy (float) representation of the quotient to disagree with, because there is only one
representation. `speed_division_scan.py` demonstrates this on three of the measured
divergent cases (`(7, 0.560)`, `(14, 1.120)`, `(21, 0.336)`) -- evaluated as an exact
`Fraction` throughout, each reproduces the "exact" column exactly, which is the only
column that exists in that scheme.

Unlike `line_height`, restricting `speed`'s *decimal precision* is not the fix -- the
mechanism has to change from "restrict the input's precision" to "change what arithmetic the
invariant is evaluated in," because division doesn't have a precision at which it becomes
exact the way multiplication does at tenths. This confirms the ticket's own prediction.

## What this does not decide

- **Whether to change `speed`'s wire format to `[num, den]`, or keep the decimal spelling
  and parse it to an exact rational at read time.** Both close the measured divergence
  identically; which is more legible to an authoring agent is a design question, not a
  measurement one, and is left to whatever ADR takes this up.
- **Whether 0.0041-0.0302% is worth fixing at all**, relative to `line_height`'s 6.35% or
  `fit`'s 4.466%. This ticket's job was only to measure whether the exposure manifests
  (ADR-0028's explicit condition for reopening), not to re-litigate ADR-0020's acceptance.
