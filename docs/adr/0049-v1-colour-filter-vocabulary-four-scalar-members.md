---
status: accepted
amends: 0040 (settles the colour-filter deferral that ADR named and excluded from its own scope)
---

# The v1 colour-filter vocabulary: four flat scalar effects, and a mechanical stopping rule

> **Amended by [ADR-0088](./0088-chroma-is-a-matte-operation-and-color-stays-literal.md)**,
> which states that the stopping rule below and its closed `tint.color` exception are scoped
> — in their own words — to **colour operations**, and so do not reach a matte operation.
> `shadow{…, color, …}` is named there as the non-scalar colour parameter that already sat
> outside them, predating this ADR. The keyer's `spill`, which *does* change pixel colour, is
> admitted on clauses (a)-(d) below rather than by citing `tint` as precedent, so the
> exception stays closed exactly as written.

[ADR-0040](./0040-effect-model-attachment-and-v1-vocabulary.md) deferred colour filters
entirely from v1's effect vocabulary, on a closed-vocabulary-erosion argument rather than
a difficulty one: "tint/grayscale/duotone/etc." is the shape of a family that does not
stop closing itself — grayscale invites sepia invites duotone invites
brightness/contrast/saturation invites curves invites LUTs. That ADR's own court was
split 2-1 (Opus for deferring outright, Haiku/Fable for shipping a small closed list),
resolved in favour of deferring until this ticket could state where the family actually
stops. [#22](https://github.com/MBehtemam/Montagent/issues/22)'s fixture-absence warning
applies again here — the committed fixture has zero colour-filter usage — and per
[ADR-0003](./0003-general-video-editor-not-channel-tooling.md)'s asymmetry that absence
is not evidence against shipping one, only an absence of a forcing case from the one
real project.

## Decision

### Shape: several flat named effects, matching `blur`/`shadow`/`mask` exactly — no `color{mode}` discriminator

Colour operations sit in the same ordered `effects` list as every other v1 effect, each
its own top-level name with its own flat parameter set. A single `color{mode, ...}`
effect with an internal discriminator was considered and rejected: it would be the only
two-level lookup in v1's vocabulary (an agent must learn "what effects exist" and, for
this one member, "what modes exist inside it"), and it complicates the one thing the
list mechanism already gives every other effect for free — stacking. `[grayscale-ish
saturation, tint]` composes exactly like `[blur, shadow]` does today; a `mode`-keyed
effect would need either a bespoke internal ordering concept or would push an agent
toward two `color` list entries that sit oddly against a singular `mode` field.

Decided 3/3 across two independent court rounds (Opus, Haiku, Fable), unanimous.

### Member list: `tint`, `saturation`, `brightness`, `contrast` — four scalar effects, `grayscale` folded into `saturation`'s zero endpoint

```json
{"name": "tint", "color": "#FF8A00", "amount": 0.4}
{"name": "saturation", "amount": 0}
{"name": "brightness", "amount": 0.15}
{"name": "contrast", "amount": 0.2}
```

- **`tint{color, amount}`** — pushes pixel colour toward `color` by `amount` (0–1).
  Grandfathered non-scalar exception, below.
- **`saturation{amount}`** — 0 = fully desaturated (equivalent to a bare `grayscale`
  effect), 1 = unchanged (identity value), >1 = oversaturated.
- **`brightness{amount}`** — signed offset from unchanged at 0.
- **`contrast{amount}`** — signed offset from unchanged at 0.

`grayscale` and `sepia` are deliberately **not** separate members, and this reverses the
6-member candidate list (`grayscale`, `sepia`, `tint`, `brightness`, `contrast`,
`saturation`) an earlier court floated on [#22](https://github.com/MBehtemam/Montagent/issues/22)
as a starting point, not a settled answer:

- **`grayscale` is not its own axis — it is `saturation{amount: 0}`.** The stopping rule
  below forbids an admitted member that is reproducible by composing other admitted
  members; keeping both `grayscale` and `saturation` violates that rule by the rule's own
  text, not by a close reading of it. A first court round split 2-1 on this exact
  question (majority: fold it; dissent: keep `grayscale` explicit for discoverability,
  matching the zero/near-zero-param shape of `blur`/`shadow`/`mask`). A second round,
  with the trade-off restated plainly, went 3/3 to fold it: the dissent never disputed
  the structural point, only the ergonomics, and a stopping rule that yields to the first
  ergonomic appeal stops nothing — the next request (`sepia`, `invert`,
  `desaturate-slightly`) would cite `grayscale` as precedent. Discoverability is a
  documentation and schema-`examples` concern (`saturation`'s description states "0 =
  full grayscale"), not a member-list concern; a missing axis is not repairable without
  reopening this ADR, while a naming gap is.
- **`sepia` is `saturation{0}` composed with a warm `tint`** — excluded on the same
  non-composability clause, without needing a separate argument.

### The stopping rule: fixed arity, bounded scalar parameters, a documented identity value, non-composability

**A colour operation is admissible in v1 only if (a) its parameter count is fixed by the
effect's name, not by user input; (b) every parameter is a bounded scalar; (c) it has a
documented identity/no-op value; and (d) it is not reproducible by composing two or more
other admitted members.** Curves, LUTs, per-channel colour matrices, and duotone-style
two-colour maps are categorically excluded under this rule — not because they are hard to
implement, but because a curve or a LUT is not a fixed parameter set, it is an arbitrary
function, and admitting one erodes the closed-vocabulary property this rule and
ADR-0040 exist to protect. LUTs were already separately rejected in ADR-0040 as an
external-file dependency; that reasoning is cross-referenced here, not re-litigated.

This rule replaces an earlier, looser candidate ("≤2 numeric parameters, a linear or
near-linear transform of pixel colour, along one perceptual axis"), retired for two
reasons a court surfaced independently in two ballots: "near-linear" has no test and gets
the sign backwards in places (saturation is not linear; sepia, which should be excluded,
is linear), and "≤2" is an arbitrary count that invites relitigation rather than
settling anything. The perceptual-axis language (hue, saturation, lightness, tint) is
kept as **commentary illustrating what the rule currently yields**, not promoted into the
rule as an independent condition a proposal must also satisfy — a 2-1 court found that
promoting it would either be redundant with the mechanical test or would wrongly reject a
future legitimate scalar operation (e.g. `gamma{value}`) that doesn't obviously sit "on"
one of today's four named axes. Freezing the v1 member set is a separate, honest decision
this ADR does make explicitly (member list, above) — it is not smuggled in as part of the
admissibility test.

**`tint.color` is the sole grandfathered exception to "parameters must be bounded
scalars."** `tint` is adopted above despite `color` not being a scalar; its identity
value (`amount: 0`) is well-defined independent of `color`'s type, which is what makes it
admissible under clause (c) despite the tension with clause (b). This exception is
closed: a future colour-op proposal does not get the same allowance by citing `tint` as
precedent, and a proposal wanting an arbitrary-colour parameter needs its own stated
justification or a scalar redefinition (e.g. a hue-angle or colour-temperature value)
against this rule, not an appeal to existing practice.

## Consequences

- The schema's `effects` discriminated union gains four new flat members:
  `tint{color, amount}`, `saturation{amount}`, `brightness{amount}`, `contrast{amount}`,
  alongside the existing `blur`/`shadow`/`mask`.
- No `grayscale` or `sepia` member exists or will be added under this rule without a new
  ADR; `saturation`'s parameter description documents `amount: 0` as the grayscale case.
- A future colour-op proposal is checked against this ADR's four-clause mechanical test
  and the current member list's non-composability, not against an open-ended "does it
  feel like a colour thing" judgement.
- `docs/research/juries/colour-filter/BALLOTS.md` (see Evidence) is the citable record
  for both the shape question and the stopping-rule question, should either be
  relitigated.

## Evidence

Two court rounds, three jurors each (Opus, Haiku, Fable via `/court`), independent, blind
to each other's ballots and to the author's recommendation.

**Round 1** — three questions put simultaneously: (Q1) shape, (Q2) exact member list
among four candidates, (Q3) endorse/tighten/loosen the stopping-rule draft.
- Q1: **unanimous 3/3** for flat named effects over a `mode`-discriminated single effect.
- Q2: **2-1** for the 4-member folded list (`tint`/`saturation`/`brightness`/`contrast`)
  over keeping `grayscale` explicit and dropping `saturation`; the dissent (Haiku)
  conceded the structural point and objected only on discoverability grounds.
- Q3: **2-1 TIGHTEN**, with Opus and Fable converging independently on the same
  underlying fix (fixed arity, scalar parameters, identity value) while diverging on
  whether the axis list belongs in the rule text and whether `tint.color` needs an
  explicit exception clause.

**Round 2** — the two splits from round 1 restated as direct two-way questions, with the
dissent's own reasoning surfaced back to the panel.
- Member-list packaging: **unanimous 3/3** for the folded 4-member list (round 1's Haiku
  dissenter flipped on re-reading the rule's own text against its stated purpose).
- Stopping-rule packaging: **2-1** for keeping the axis list as illustrative commentary
  rather than promoting it into the rule (Opus, Fable) over making it a required
  condition with an explicit `tint` exception (Haiku). The dissenting ballot's own
  synthesis — keep the mechanical-test framing, add one line grandfathering
  `tint.color` — is what this ADR adopts, closing the gap between the two positions
  without either side's full proposal.

Full ballots for both rounds, all six jurors, are committed at
`docs/research/juries/colour-filter/BALLOTS.md`.
