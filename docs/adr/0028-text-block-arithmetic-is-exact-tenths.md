---
status: accepted
amends: 0007 (`line_height`'s numeric domain), 0014 (states the arithmetic its `ceil`
  formula must be evaluated in)
---

# Text-block height: exact tenths, `ceil`, and a rule for the next field

[ADR-0014](./0014-stroke-is-paint-the-text-box-is-required.md) fixed the text-block height
formula as `ceil(size × line_height × line_count)`, `size` and `line_count` both integers.
It did not say what numeric domain `line_height` draws from or what arithmetic evaluates
the product — and `line_height` is not the integer-pixel, integer-millisecond field almost
everything else in this format is. [#58](https://github.com/MBehtemam/Montaget/issues/58)
found the gap has a measured cost: evaluated in IEEE double, `size × line_height ×
line_count` disagrees with its exact-decimal value on **76 of 1197 sampled (size,
line-count) combinations at `line_height = 1.1`** — 6.35% — always by one pixel, because
`55 * 1.1` is `60.50000000000001` in double and other (size, line-count) pairs land the
other side of a rounding boundary. This is [ADR-0013](./0013-fitted-extents-floor-and-the-nine-origin-keywords.md)'s
ULP hazard again, in a field ADR-0013 never touched: that ADR measured a 4.466% divergence
in the `fit`-box calculation and banned float arithmetic from it. The committed fixture
escapes both bugs for the same reason — `1080/1536 = 45/64` is dyadic for the first, and
every committed `(size, line_height, line_count)` triple happens to fall clear of a
boundary for the second — so a correct-looking fixture is not evidence the arithmetic is
correct.

A second, independent defect sits next to it: `migrate.py`'s `r()` rounds the same product
with `Decimal(...).quantize(0, ROUND_HALF_UP)`, not `ceil`. It agrees with ADR-0014's stated
formula on all 15 currently-committed derived heights only because every fractional part
in those 15 cases happens to be `>= 0.5`, where `ceil` and round-half-up cannot disagree —
coincidence, not design.

## Decision

**`line_height` is restricted to one decimal digit (tenths): `1.0`, `1.1`, `1.2`, … — always
exactly representable as `n/10` for an integer `n`.** The block-height derivation is
evaluated as exact integer arithmetic on that representation, never IEEE double:

```
n = line_height × 10          (integer, by construction — never recovered by float parsing)
height = ceil(size × n × line_count / 10)
       = (size × n × line_count + 9) // 10     (exact integer division, positive operands)
```

`ceil` stands, matching ADR-0014's stated formula. **`migrate.py` is the thing that was
wrong**: its `r()` call at the text-block-height site is replaced with the exact-tenths
`ceil` above; `r()` itself is untouched everywhere else it's used, since its `ROUND_HALF_UP`
behaviour there rests on ADR-0012's own "ties away from zero" rule for `x`/`y` interpolation
residuals, a different question this ADR does not reopen.

Tenths, not hundredths or an integer permille field: the fixture's only two values
(`1.1`, and the `1.2` default) are both tenths, nothing in ADR-0007's typography discussion
asks for finer control, and tenths keep the field's existing human-legible decimal spelling
— `line_height: 110` would make every author divide by 100 in their head for no exactness
gain a decimal restriction doesn't already buy. If a real typographic need for hundredths
ever surfaces, the fix is widening the restriction to `n/100`; the technique — exact integer
numerator, exact integer division — is unchanged either way.

## Evidence

Three-model independent court (Claude Opus 5, Claude Haiku 4.5, Claude Fable 5.1), each
blind to the others' ballots, briefed on the measured 6.35% divergence, the `migrate.py`
inconsistency, and the newly-surfaced `speed` question below.

**Unanimous 3/3** on restricting `line_height` to exact tenths with exact integer
arithmetic (mirroring ADR-0013's technique), on scoping this decision to `line_height` alone
rather than folding in `speed`'s unmeasured exposure, and on stating a general rule (below).

**Split 2–1 on `ceil` vs. `ROUND_HALF_UP`** — the one question this court did not settle by
measurement, because both sides argue from an unmeasured product-behaviour fact: whether a
box under-allocated by a fraction of a pixel ever visibly clips rendered text, or whether the
renderer's own layout slack absorbs it. Resolved by the human as `ceil` (majority), i.e. fix
`migrate.py` to match ADR-0014's already-published formula rather than reopen the ADR: the
15 committed values are identical under both rules, so nothing changes in the shipped
fixture, and `ceil`'s guarantee — a text box is never shorter than the block it is declared
to bound — is the more defensible default for a *container* field absent a demonstrated
clipping cost. Evidence in the ticket's ballots; no separate research directory, since the
divergence itself was already measured and cited by `docs/research/juries/contain-slack/`.

## Consequences

- `line_height`'s schema gains a precision constraint: legal values are multiples of `0.1`.
  A `validate`/schema-time error names any other value (e.g. `1.15`) as out of domain.
- `migrate.py`'s text-block-height computation (the `bh = r(...)` line) is replaced with the
  exact-tenths `ceil` formula above. This changes **zero committed bytes** — verified by
  regenerating and byte-diffing the fixture, per the evidence-must-be-re-executable rule
  ([`docs/agents/domain.md`](../agents/domain.md)) — because every one of the 15 derived
  heights in the fixture has a fractional part `>= 0.5`, the exact case where `ceil` and the
  old `ROUND_HALF_UP` cannot disagree.
- **General rule, stated once so the next field doesn't re-derive this argument from
  scratch:** *any non-integer field whose value feeds a boundary-sensitive operation
  (`ceil`/`floor`/`round`) must have its legal values constrained to an exactly-representable
  domain, and that operation must be evaluated in exact integer or rational arithmetic —
  never IEEE double.* This is now the second field to trip this wire (`fit`'s box sizing in
  ADR-0013, `line_height` here); a third, `scale`/`rotation`/`opacity`/`ease` control points,
  is explicitly **exempt** — they are stored non-integer values consumed only by continuous
  interpolation and hit no discrete boundary today, so constraining them would be restriction
  without a corresponding correctness gain.

## Not settled here

- **`speed`'s rounding has the same structural shape of risk and is unmeasured.**
  [ADR-0020](./0020-speed-overrun-hold-loop.md)'s `round((source_end - source_start) /
  speed)` is an ordinary division followed by round-half-up; whether it diverges from exact
  decimal division across `speed`'s realistic value range (unlike multiplication, an
  arbitrary division isn't always made exact by restricting the divisor to tenths) has not
  been measured. Scoped out of this ADR deliberately, unanimous 3/3: `speed`'s ADR is
  already closed and accepted, and reopening it on a structural hunch rather than a
  measurement would repeat the exact mistake ADR-0013 avoided by measuring first. Graduated
  to [#95](https://github.com/MBehtemam/Montaget/issues/95), which must measure before
  proposing any fix and may need a different mechanism than tenths-restriction, since
  division doesn't degrade the same way multiplication does.
