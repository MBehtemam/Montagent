---
status: accepted
amends: 0020 (states the arithmetic its rounding invariant must be evaluated in)
---

# `speed`'s rounding invariant is evaluated in exact arithmetic, not IEEE double

> **Amended by [ADR-0081](./0081-speed-literal-obligation-is-about-the-division-not-the-parser.md)**,
> which settles that the string-carriage sentence below binds the division, not
> `serde_json`'s own parse — closing [#256](https://github.com/MBehtemam/Montagent/issues/256).

[ADR-0020](./0020-speed-overrun-hold-loop.md) defined `speed`'s invariant as
`end - start == round((source_end - source_start) / speed)`, round-half-up, without
saying what arithmetic evaluates the division. [#95](https://github.com/MBehtemam/Montagent/issues/95)
measured the gap: naive IEEE-double evaluation diverges from the exact-decimal answer
at a small but real and reproducible rate — 0.0041% of cases at `speed`'s current
three-decimal precision (the fixture's own `0.645`), rising to 0.0302% at hundredths.
Minimal case: `7 / 0.560` is exactly `12.5` in decimal, so round-half-up gives `13`,
but `f64` computes `12.499999999999998`, giving `12`. This is the same shape of defect
[ADR-0028](./0028-text-block-arithmetic-is-exact-tenths.md) fixed for `line_height`
(evaluated in IEEE double, disagreeing with exact decimal near a rounding boundary).

## The fix is evaluation, not storage

`speed`'s authored literal (`0.645`) is already a finite decimal — already an exact
rational (`645/1000`). No information is lost at the file-format boundary. The
divergence is introduced entirely downstream, when the check coerces that literal
through `f64` and divides in binary floating point instead of evaluating the division
exactly. An exact tie under round-half-up is not ambiguous — it has one correct
answer — `f64` can simply land on the wrong side of it.

`speed` **does not change shape**. It stays a single authored decimal number,
unchanged in the schema, in the fixture, and in every existing project file.

**Implementation obligation**: wherever this invariant (or its inverse, `validate`'s
corrective-`speed` computation from ADR-0020) is evaluated, the implementation must
parse `speed`'s literal decimal value and perform `source_span / speed` as exact
rational/decimal arithmetic (e.g. `Decimal` or `Fraction`, constructed from the
literal's own digits — never from a `float`/`f64` intermediate) before applying
round-half-up. A JSON parser that eagerly widens numeric literals to `f64` is
insufficient on its own; the exact literal must reach the arithmetic as a string
or an exact type before any division happens.

## Why ADR-0028's fix does not transfer

ADR-0028 made `line_height × size × line_count` exact by restricting `line_height`
to one decimal digit: the product of two exact decimals is exact, and division was
never involved. `speed` feeds a **division**, and precision-restriction does not
make a division exact the way it makes a multiplication exact: `0.4 = 2/5` has an
even reduced numerator and ties every odd `source_span`, at *any* fixed decimal
precision `speed` might be restricted to. Restricting `speed`'s digits would not
have closed this gap; only evaluating the division exactly does.

## Why storing `speed` as `[num, den]` (mirroring `par`, ADR-0023) was rejected

An exact-rational pair only buys something beyond exact-literal evaluation for a
`speed` whose *true* value isn't expressible as a finite decimal at all — an exact
`1/3`, say. No such want has been stated for `speed`. `par`'s rational-pair shape
earns its cost because aspect ratios are routinely non-terminating in decimal
(`16/9 = 1.7777...`); `speed` values are authored as, and have always been, finite
decimals. Adopting `[num, den]` here would impose a schema change, a migration of
every existing project file, and a permanent authoring cost (`"speed": [645, 1000]`
against `"speed": 0.645`) to buy a capability nobody has asked for. If an exact
non-decimal speed is ever wanted, a `[num, den]` form can be added later as an
alternative accepted shape without disturbing the decimal literal this ADR fixes.

## Scope: `speed` only, not a general rule

An audit of every `round`/`ceil`/`floor` site in the format (docs/adr/*.md) found
`speed` is the **only** authored decimal field that feeds a division into a
rounding boundary. `line_height` and fitted-extent scaling (ADR-0028, ADR-0013)
feed multiplications, already covered. `par` (ADR-0023) is already an exact
integer-pair rational, never a decimal, and its one rounding point is a
multiplication (ADR-0013's floor rule), not a new division. Text-baseline
half-leading (ADR-0029) contains a division but is explicitly exempted, feeding
continuous rasterization rather than any `ceil`/`floor`/`round` boundary.

With exactly one confirmed instance, this ADR does **not** state a general rule
("any decimal field feeding round/ceil/floor through division must be evaluated
exactly") as durable format-wide policy — a rule with one instance and an
untested boundary (would it reach ADR-0029's exempted division? this ADR can't
say) would be asserted, not earned, the same trap ADR-0006 rejected for optional
fields that can silently mean nothing. The reasoning above — why division needs
exact evaluation where restricted-precision multiplication didn't — is recorded
here precisely so a future ticket that finds a second division-fed field has a
citable precedent, the same way ADR-0028's narrow `line_height` fix became this
ADR's precedent rather than the reverse.

## The fixture is unaffected

All four `speed` elements in the committed fixture (`vo-sentence-05-b` through
`08-b`) share `speed: 0.645` (`= 129/200`, odd reduced numerator) against source
spans of `2184`, `2568`, and `1992` ms. None sits on a round-half-up tie — `0.645`
never ties, at any source span — so this ADR changes zero bytes of the committed
project file. It closes a defect the fixture does not currently exhibit but
would, on different (still legal) `speed` values.

## Consequences

- `validate`'s speed-mismatch check (ADR-0020) and its corrective-`speed`
  computation must be implemented in exact rational/decimal arithmetic, never
  by parsing `speed` into `f64` before dividing.
- Any conformance suite for `validate` should include a known tie case (e.g.
  `source_span: 7, speed: 0.560`, expecting `end - start == 13`) as a regression
  guard against silently reintroducing an `f64` divide.
- `speed`'s authored shape, schema, and every existing project file are
  unchanged.
- No general division-exactness rule is adopted format-wide; this ADR is the
  precedent to cite if a second division-fed field is found.
