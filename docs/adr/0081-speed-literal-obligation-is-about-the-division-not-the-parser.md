---
status: accepted
amends: 0045 (settles what it left silent: whether the string-carriage sentence binds the JSON parser or only the division)
---

# ADR-0045's string-carriage sentence discharges at the division, not at the parser

**Ticket:** [#256](https://github.com/MBehtemam/Montagent/issues/256), raised by
`crates/montagent-core/src/exact.rs` (shipped for [#197](https://github.com/MBehtemam/Montagent/issues/197)),
per `docs/agents/domain.md`'s *"if your output contradicts an existing ADR, surface it
explicitly."* Nothing in `#197`'s shipped code changes; this ADR is where the gap it
found becomes settled.

## The literal reading and why it was never fully met

ADR-0045 says *"the exact literal must reach the arithmetic as a string or an exact type
before any division happens,"* and calls a parser that *"eagerly widens numeric literals
to `f64`"* insufficient **on its own**. `serde_json` is built without
`arbitrary_precision` — required because `crate::model::keyframe` uses
`#[serde(untagged)]` for its two keyframe record shapes, and the two features are
documented as incompatible — so every non-integer literal, `speed` included, has already
become an `f64` by the time any Montagent code runs. `#197`'s division is exact
(`i128`, no `f64` in the arithmetic), but the *value* it divides is whatever `f64` could
represent of the authored literal: the shortest round-tripping decimal, via the
`Number`'s `Display`.

## Ratified: the obligation is about the division, never the parser

The measured defect ADR-0045 exists to fix — `7 / 0.560` landing on the wrong side of an
exact tie under `f64` division — is the one class of error that matters, and it is fully
closed: `crates/montagent-core/src/exact.rs` divides on `i128`, and both `exact.rs` and
`tests/time.rs` carry a regression test for exactly this shape. The residual gap — a
literal with more significant digits than `f64` can distinguish, e.g.
`"speed": 0.6450000000000000001` reaching the check as `0.645` — is information already
lost *before* Montagent's own code runs, on a literal no author has ever written and
which the committed fixture does not contain (all four `speed` elements round-trip
`f64` exactly, per ADR-0045 itself). ADR-0045's string-carriage sentence is read, as of
this ADR, as binding the **division** — *no `float`/`f64` intermediate between the
literal (as `serde_json` hands it over) and the rounding boundary* — not as a demand that
the JSON parser itself preserve arbitrary decimal precision ahead of that boundary.

This is a narrower reading than ADR-0045's prose taken alone would suggest, chosen over
building a string-preserving second read path for `speed` (or dropping `untagged` from
the keyframe model) because:

- **The residual has no observed instance.** No project file, fixture, or bug report has
  ever carried a `speed` literal with more digits than `f64` distinguishes; the gap is
  theoretical against every corpus that exists.
- **Both fixes cost real correctness surface for zero measured defects.** A second read
  path duplicates the numeric-literal parse Montagent already trusts `serde_json` for, on
  one field, forever out of sync with it. Dropping `untagged` breaks the strict parse of
  every keyframe in the committed fixture and is a schema-model change with no other
  motivation.
- **ADR-0045's own precedent is narrow-scope-until-earned**: it declined a general
  division-exactness rule with "exactly one confirmed instance" of the problem it *did*
  fix. The same discipline applies here — a parser-precision fix with *zero* confirmed
  instances is not earned by this ticket.

## Consequences

- ADR-0045 gains an "Amended by" banner pointing here.
- `crates/montagent-core/src/exact.rs`'s `#256` comment, if any, is replaced with a
  citation to this ADR; no functional code changes.
- If a `speed` literal exceeding `f64`'s representable precision is ever observed in the
  wild, that is new evidence and reopens the question this ADR closes on zero instances —
  it does not retroactively make this reading wrong.
