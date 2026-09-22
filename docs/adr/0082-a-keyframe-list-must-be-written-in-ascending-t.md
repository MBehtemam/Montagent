---
status: accepted
amends: 0012 (a keyframe list gains an ordering constraint at the schema, not only a
  resolution rule), 0038 (`ease`'s positional presence rule now coincides with clock
  order by construction, closing the divergence #270 found)
---

# A keyframe list must be written in ascending `t`; `E-SCHEMA` names the first inversion

**Ticket:** [#270](https://github.com/MBehtemam/Montaget/issues/270), found while
implementing [#208](https://github.com/MBehtemam/Montaget/issues/208) (the keyframe
resolver and `query --at`). Neither ADR-0012 nor ADR-0038 states whether a keyframe list
must be written in the clock order its `t` values imply, and the two consumers that
existed read the list two different ways — `Animatable`'s deserializer by array
position, `resolve::at` by `t` after sorting — which coincide on every list anyone has
written and come apart on

```json
"opacity": [{"t": 800, "v": 1}, {"t": 0, "v": 0, "ease": "linear"}]
```

where the positional rule is satisfied (index 0 carries no `ease`, index 1 does) but the
record arriving *first on the clock* is the one with no stated easing — a silent
divergence between what `validate` accepts and what the resolver actually does.

## Ratified: ascending `t` is a schema rule

A keyframe list's records must be written with strictly increasing `t`. `#208` did not
take this because no ADR stated it; it is new format law, adopted here as the smallest
change that removes the divergence entirely, per #270's own framing: it "makes the two
readings identical by construction." Once array position and clock position always
agree, ADR-0038's positional `ease` rule and `resolve::at`'s clock-based read are the
same statement, not two that can be fed a file where they disagree.

**Why not "re-read ADR-0038 as the earliest record on the clock" instead.** That reading
keeps unordered lists legal, but the published JSON Schema can only express
`ease`-presence positionally (`prefixItems`), so schema and binary would agree on every
list anyone happens to write in ascending order and silently disagree on one that is not
— the exact defect this ADR exists to close, relocated rather than removed. It also adds
a second thing every consumer must do (sort before reasoning about presence) where the
schema rule adds nothing at read time: array order already is clock order.

**Why not leave it as a `review` finding instead.** A `review` finding (the third
candidate #270 named, "not illegal, and a human must look") is the right shape for a
question the document's authored intent leaves genuinely open — this is not that. Nothing
about an unordered keyframe list is ever intended; it is either a mistake or an artifact
of a script that assembled the list without sorting it. ADR-0006's own alarm-fatigue
test cuts the other way here: a `review` finding on every unordered list a buggy script
produces is not the honest middle ground `R-KEYFRAME-UNREACHED`-style checks earn where
the document really might mean it either way. A schema error, catching the mistake at
the moment it is written, is the check that matches what an unordered list actually is.

## What this settles

- **Schema.** `E-SCHEMA` fires on the first `t` that is not strictly greater than its
  predecessor in the array, naming the index and both `t` values — the same shape
  ADR-0016 already gives other malformed-value schema errors. Two records sharing one
  `t` are also an inversion under this rule (`t` is not *strictly* increasing) and fire
  the same code; ADR-0012 and ADR-0038 are silent on same-`t` records and this ADR does
  not additionally resolve that question.
- **The published JSON Schema.** Gains the ascending-`t` constraint (an
  array-of-objects ordering check, expressed the way the schema already expresses
  `prefixItems`-based positional constraints) alongside the existing `ease`-presence
  rule.
- **The resolver.** `resolve::at` needs no `t`-sort once every keyframe list reaching it
  has already passed the schema check — the sort ADR-0012 implied for a general list
  becomes provably a no-op and may be removed or kept as a defensive assertion, at the
  implementer's discretion.
- **`shift`'s SPLIT ([#220](https://github.com/MBehtemam/Montaget/issues/220)).** Step
  7's "any run of three or more consecutive records" now unambiguously means consecutive
  in the array, because array order and clock order are the same order by construction.
- **The committed fixture.** Every keyframe list in it is already ascending (#270 states
  this); this ADR changes zero bytes of the fixture.

## Consequences

- ADR-0012 and ADR-0038 each gain an "Amended by" banner pointing here.
- A new schema-error code (or a documented reuse of `E-SCHEMA`, at the implementer's
  discretion per ADR-0016's split) enforces strictly ascending `t` within a keyframe
  list.
- A regression test should cover the `#270` divergence case above directly: it must be
  rejected by `validate`, not merely handled correctly by the resolver.
