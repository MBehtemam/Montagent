---
status: accepted
amends: 0037 (its evidence does not reproduce — erratum below; and its deferred "inert provenance" need is discharged here), 0036 (designs the renderer-ignored declaration it raised and left unresolved; answers its same-rate-or-same-endpoint objection from data), 0063 (states precisely where its exact-equality scoping stops applying — narrowed, not reversed), 0012 (its zero-element-to-element-references invariant becomes zero *live* references, spendable per axis on measured proof)
---

# Recorded intent is one pattern with several fields, and the time axis instantiates it

[Ticket #324](https://github.com/MBehtemam/Montagent/issues/324), merging the map's two
**recorded intent** fog patches. Four exemplars, found independently on three axes, each
naming the same gap: **the format has no way to record what an author meant.** Every value
is a literal, deliberately, so a relationship between two values exists only in the author's
head, and an edit that breaks it is indistinguishable from an edit that never had it.

Resolved by two juries of three independent models each (Opus 5, Sonnet 5, Fable 5.1 — one
family per juror, so agreement across a round is not one model agreeing with itself).
Round 1: unanimous on the pattern, on the reference question, and on splitting violation
cost by direction; split 2–1 on how many fields ship. Round 2: twenty-three confirmations
and one overturn, adopted below. Ballots verbatim in
[`docs/research/juries/recorded-intent/`](../research/juries/recorded-intent/).

## Erratum: ADR-0037's evidence does not reproduce, and this ADR replaces it

**This must be read before anything else here, because it is the ground the ticket stood
on.** ADR-0037 states that the fixture carries *"52 hand-typed absolute timestamps"* of
which four — *"caption hold-out records at 8000, 16800, 25700, 35300 ms, each equal to
`end − 300`"* — carry no signature. It transferred that 4-of-52 ratio here as *"that fog
entry's first pricing datum."*

**None of it reproduces.** Those four values appear in **no committed version** of the
fixture — all five commits that have ever touched
`fixtures/en-halloween-decorating/en-halloween-decorating.montagent.json` return zero
matches. The file carries **135** timeline instants, not 52, and has no `highlight` anywhere
to hold a caption out. This is the defect
[`docs/agents/domain.md`](../agents/domain.md) already records as the motivating failure —
an ADR outliving the file it measured — recurring one ADR later, and it was found only
because this ADR's evidence rule required re-deriving the number before building on it.

ADR-0037's **disposition** is unaffected: no tool ships, the discriminator that killed the
emit-only helper is untouched, and its re-diagnosis of the problem as *provenance, not
computation* is the correct diagnosis and the premise of this ADR. Only its census is
withdrawn. ADR-0037 gains a banner saying so.

The replacement evidence is stronger, and it is committed:
[`recorded_intent_scan.py`](recorded_intent_scan.py) re-derives every number below and
exits non-zero the moment any stops holding.

## The evidence, re-measured

| claim | count | what it is |
| --- | --- | --- |
| Ken Burns ramps carrying a `scale` keyframe pair | **7 of 7** | every pair is exactly **15000 ms** apart, and no other offset appears |
| …whose first keyframe sits exactly on the element's `start` | **7 of 7** | an exact coincidence |
| …whose final keyframe runs **past** the element's own `end` | **7 of 7** | and **0 of 7** land on it |
| repeat-reading pairs (`*-a` → `*-b`) | **8 of 8** | a fixed inter-element gap, bimodal and exceptionless: **800 ms** on all 4 word pairs, **520 ms** on all 4 sentence pairs |
| instants in those relationships whose value appears exactly once in the file | **16 of 16** and **7 of 7** | nothing anywhere records the relationship |
| text elements overlapping 2+ audio sources | **11 of 22** | carried forward from [#166](https://github.com/MBehtemam/Montagent/issues/166), re-derived here so the two ADRs cannot drift |

**The third row settles a question ADR-0036 left open.** It called the renderer-ignored
declaration a **half-binding**, because it *"records a start instant but not whether the
intended relationship is same-rate or same-endpoint."* On the real fixture the ramps are
**same-rate, 7 of 7**: every one runs past its element's `end` and is truncated there, and
none lands on it. Had the author meant same-endpoint, the ramp would stop at `end`. The
objection was correct that the distinction matters and wrong that the document cannot
settle it — the data does, and the rule vocabulary below names the rate case explicitly
rather than leaving a reader to guess which half was meant.

## The pattern

**One gap, one rule for filling it, several fields.** The gap is singular — a literal whose
provenance is unrecorded — and the fix is singular: a **renderer-ignored declaration whose
only consumer is `validate`**, structurally [ADR-0015](./0015-fit-is-a-derivation-claim-and-gravity-retires.md)'s
`fit` transplanted off the raster axis. But the *content* of a derivation rule is not
shared: a timestamp offset, a motion coupling and an audio calibration link are not three
values of one enum.

A single generic field — *"value X was derived from value Y by rule R"* where R ranges over
arithmetic — is **rejected**, unanimously across both rounds. The moment R is an open set
the document contains a thing a reader cannot understand by reading; they must run the
arithmetic in their head to know what the document asserts. That is the original disease
with a smaller blast radius, not a cure, and it silently reopens
[ADR-0017](./0017-closed-schema-no-escape-hatch.md)'s closed schema, because the legal value
set stops being named and published and starts being generated.

### Conditions of admission

These are **obligations on every future field claiming this pattern**, not a description of
the one shipped here. A field that cannot meet all six is not an instance of this pattern
and needs its own ADR arguing why.

1. **No renderer reads it.** The literal beside it remains the sole author of what renders.
2. **`validate` is the only consumer.** See *Why not `compare`* below.
3. **The rule set is finite and published.** A new rule is an ADR carrying a census, never a
   convenience.
4. **At most one argument, its type fixed per rule by the schema.** Not "one argument of any
   type" — that is an escape hatch by another name.
5. **Rules do not compose.** No rule's argument is another rule.
6. **The declaration is optional**, and its absence means *no claim* — never a claim of
   independence.

**The step this pattern refuses, named so it cannot be taken by drift:** a second argument,
a signed argument, or a rule referencing another rule. Any of those is a new ADR, not an
extension.

### What a violation costs

The overturn adopted from round 2: **a violated declaration is always `error`.** Only the
repair class varies.

| declaration shape | class | repair |
| --- | --- | --- |
| **directional** — the document names which value is source and which derived | `error` | **advise-class**: states the re-derived integer |
| **symmetric** — names no source ("these two stay equal") | `error` | **refuse-class** `"none"` |
| **partner-only** — no arithmetic to re-derive | *not violable*; only the dangling-reference finding applies | — |

The provisional answer put the symmetric case at `review`, and a juror overturned it by
showing it contradicted this ADR's own reasoning about dangling references. `review` means
*"legal, renders, and **you must look at a frame** to know if it was meant."* A violated
symmetric declaration has already told you it was not meant — the document asserts the
inconsistency — and **no frame can say which of two numbers is stale.** Putting it at
`review` would make severity depend on whether the arithmetic happens to have one root or
two, which is not what the classes are named for. The directional case is `fit`'s logic
unchanged: the author supplied the missing determinant, so exactly one integer is legal.

**The v1 vocabulary below has no symmetric member.** The symmetric row is defined by the
pattern and is currently unexercised; it binds the first axis that proposes such a rule. A
proposed symmetric rule should be read as evidence that the axis's vocabulary is
underspecified, not as a routine case.

### References: zero *live* references, spendable per axis

[ADR-0012](./0012-flat-transform-keyframes-carried-by-their-element.md) retired
`box:"<id>"` and left the format with **zero** element-to-element references.
[ADR-0036](./0036-shift-preambles-coincident-instants-validate-and-compare-stay-out.md)
separately rejected a **live** time-anchor reference 3/3. Both stand, and neither reaches a
recorded, renderer-ignored reference — **a different object**, on all three of ADR-0036's
own grounds:

- *"Breaks read-by-reading"* — the rendered value is still the literal on the page.
- *"Degrades to a stale literal on the first shift"* — it does not degrade, it **detects**;
  going stale is the finding, which is the feature rather than the failure mode.
- *"Gives one element's rendered geometry a silent second author"* — a field no renderer
  reads authors nothing, and nothing about it is silent.

**So the invariant is restated, honestly and not finessed: zero *live* references.** That is
a weaker line than zero references and it will be cited by future proposals, so the
permission is **per-axis and earned** — granted only where an ADR carries a **measurement
that inference fails**. [#166](https://github.com/MBehtemam/Montagent/issues/166)'s 11-of-22
clears that bar for audio. Nothing else clears it today, and **this ADR grants the
permission to no axis** — it fixes the terms on which a later one may spend it.

**A dangling reference is an `error` with a refuse-class repair.** The load-bearing reason
is **referential integrity**: a closed schema that admits a reference type must require the
reference to resolve, and an unresolvable one is a malformed document. That `review` is
definitionally wrong — no frame names the intended element — is a supporting reason, not
the argument.

## The time axis instantiates the pattern: `t_from`

A pattern with no instance is untested architecture. Exactly **one** field ships.

**`t_from`** is an optional member of a keyframe record, annotating that record's own `t`.
Its value is always an object.

```json
{ "t": 3018,  "t_from": { "rule": "element-start" },                "v": [1.0, 1.0] }
{ "t": 18018, "t_from": { "rule": "after-previous", "ms": 15000 },  "v": [1.08, 1.08], "ease": "linear" }
```

### The closed rule set

| rule | argument | meaning | measured |
| --- | --- | --- | --- |
| `element-start` | none | this `t` was derived as the element's own `start` | 7 of 7 |
| `after-previous` | `ms`, a **non-negative** integer | this `t` was derived as the previous record's `t` plus `ms` | 7 of 7 |

Two rules, both measured 7-of-7, and **no rule ships without a census**. `after-previous` is
the **rate** spelling the evidence forces — it says how far the ramp runs, not where it
stops, which is exactly what the fixture does and what ADR-0036 doubted could be expressed.

**Direction is carried in the rule name, never in the sign of the argument** (`after-`), so
no rule needs signed arithmetic. *"Previous"* is well-defined because
[ADR-0082](./0082-a-keyframe-list-must-be-written-in-ascending-t.md) requires a keyframe list
to be written in ascending `t`; it is positional, not a reference, and spends none of the
per-axis permission above. Both rules are **directional**, so every v1 violation is
`error` + advise-class.

### Key order

`t_from` sits **immediately after `t`**, giving keyframe records the schema order
`t`, `t_from`, `v`, `ease` (and `t`, `t_from`, `v` on the first record, which carries no
`ease` per [ADR-0038](./0038-ease-is-required-on-every-non-first-keyframe-record.md)).
[ADR-0041](./0041-canonical-key-order-is-schema-order-validate-checks-it-fmt-splits.md)
then gives canonical order and `fmt` behaviour for free — placement does not disturb it,
and adjacency to the annotated value is the whole point of the placement.

### Why beside the value and not on the element

An element carries several timestamps, so an element-level declaration would need a field
naming **which** one it describes — and that field is an intra-element pointer, a reference
adopted specifically to avoid a reference, and a worse one: it would get no ADR, no earned
permission and no dangling-reference finding class. Beside the value there is nothing to
point at.

The cost is real and accepted: the schema grows one optional key per annotatable timestamp
site rather than one per element, and it must be done consistently or the pattern develops
holes. An author wanting one claim over several of an element's timestamps has nowhere to
put it — correct rather than regrettable, since such a claim is a composition and condition
5 forbids composition.

### The checks `validate` gains

- **`R-DERIVED-T`** — `error`, **advise-class**: the rule re-derives a value that is not the
  written `t`. The repair states the integer.

  ```
  photo-06: scale keyframe t=32472 declares after-previous ms=15000 from t=17472,
            which derives 32472. Ramp start moved to 17000; write 32000, or drop t_from.
  ```

- Schema errors, each naming the legal form: an unknown `rule`; `ms` present on a rule that
  takes no argument; `ms` absent on `after-previous`; a negative `ms`; `t_from` with
  `after-previous` on the **first** record of a list, where no previous record exists.

`element-start` overlaps [ADR-0063](./0063-compare-drift-checks-keyframe-instant-relationships.md)'s
case (a) — an element's own keyframe against its own boundary — but earns its place: that is
a `compare` predicate needing a before and an after, and it is structurally blind to a file
**born** broken, which is the case this ticket exists to close.

## Why not `compare`

`validate` is the whole consumer. Both sides of a declared derivation sit in one document,
so the check is stateless, needs no I/O, and belongs to the tool defined as
stateless-over-one-document. `compare` is defined by having an input `validate` lacks — the
earlier state — and a declared relationship needs none. Giving it the check would duplicate
`validate`'s logic for no new input, which is the discriminator
[ADR-0037](./0037-derived-time-signature-is-a-provenance-gap-not-a-tool.md) used to kill the
`derive` helper.

**ADR-0063 is narrowed, not reversed, and the distinction is load-bearing.** It refused a
fixed-offset predicate because *"a preserved offset is a hypothesis about intent inferred
from arithmetic"* and the candidate population explodes. A **declared** offset is neither
inferred nor unbounded — its population is exactly the declarations an author wrote. So that
reasoning **stops applying to declared offsets** while remaining entirely correct for
undeclared pairs, which is still every pair in every file that exists today. `compare` may
**not** infer offsets. Writing this as *"fixed-offset drift is now acceptable"* would undo
ADR-0063.

## The limits, stated rather than buried

- **The mechanism is structurally blind to undeclared relationships.** An absent declaration
  and a deliberately-unrelated literal are identical on disk — `fit`'s *"an omitted field is
  indistinguishable from a decision not to check"* transplants verbatim, and is accepted as
  a permanent hole rather than answered.

  `fit` bought completeness by being **required**, which cost nothing because 8 of 8
  elements already carried it. Here required is not free: on this fixture it would put an
  escape value on **121 of the 135** instants to check 14. On a format whose authors are
  agents that copy the nearest example, that does not produce 135 considered decisions — it
  produces 121 copies of whatever escape value appeared in the first example, and `validate`
  would then certify noise as deliberate. Optional is also the **reversible** choice:
  optional can be tightened later once real declaration density is known, where required
  cannot be loosened without stranding every file already carrying the escape value.

  **The hole must not be patched by inference.** A `note` census reporting *"this value
  happens to equal the previous one plus N"* is exactly ADR-0063's population explosion
  returning by the back door.

- **The 8-of-8 repeat-reading gaps are measured here and not solved here.** They are
  **inter-element** (one element's `end` to another's `start`), so expressing them needs the
  per-axis reference permission, which this ADR grants to no axis. They are the strongest
  fixed-offset evidence in the fixture and they graduate, with this measurement, rather than
  being spent on a rule the pattern is not yet entitled to write.

- **A shift that moves source and derived together preserves the declared relation**, so
  `validate` stays silent even if the coupling was meant to break. The declaration asserts
  the relation holds, not that it should have changed. Detecting that needs a before and an
  after and is genuinely `compare`'s shape. Not built: there is no evidence it occurs, and
  speculating the feature into existence is how ADR-0063's population problem returns.

- **The better-measured axis is held back, and the reason is architectural, not
  evidential.** Audio's 11-of-22 is the strongest number in the ticket and the time axis's
  7-of-7 covers fewer instants. Audio waits because it is the axis that **spends** the
  reference permission, and paying back ADR-0012's invariant deserves its own record with
  its own dissent rather than a rider on the ADR that establishes the pattern. Stated
  plainly because the record would otherwise read as though 7-of-7 outweighed 11-of-22,
  which is false and would corrupt the evidence bar for every later axis.

- **The jury ballots did not weigh the census above.** Both rounds were given ADR-0037's
  withdrawn figures as fact. The decisions survive the correction — the replacement evidence
  points the same way and is stronger — but no ballot should be cited as having considered
  the real numbers, and the evidence directory says so.

## Consequences

- **The map's two recorded-intent fog patches are discharged.** The second patch's blocker
  (*"frequency evidence from a wider fixture corpus"*) is answered structurally, not met: a
  pattern ADR sets the bar each axis must clear, so the corpus was never the right
  instrument.
- **ADR-0037 gains a banner**: its census is withdrawn as unreproducible; its disposition
  and its re-diagnosis stand.
- **ADR-0036's half-binding objection is discharged** — same-rate versus same-endpoint is
  settled from data, 7 of 7.
- **ADR-0063 is narrowed**, precisely as written above, and not reversed.
- **ADR-0012's invariant is restated** as zero *live* element-to-element references, with
  the permission spendable per axis on measured proof. No axis spends it here.
- **No new tool and no tenth verb.** [ADR-0011](./0011-tool-surface-reads-checks-renders.md)'s
  surface is unchanged; `validate` gains one check and a family of schema errors.
- **No renderer change**, by construction.
- **The schema, the Rust types and the fixture are not changed here.** The generated schema
  is `additionalProperties: false`, so adding `t_from` is implementation that would make the
  fixture invalid against the shipped schema until it lands. It graduates as its own ticket,
  which must land the schema change **and** the fixture's 14 declarations together —
  ADR-0015's finding that agents author by copying the nearest example makes a field absent
  from the fixture a field that does not really exist.
- **Three axes graduate as tickets** against this pattern: audio (spending the reference
  permission), the inter-element fixed-offset gaps measured here, and motion — which has no
  census of any kind and must earn one before it may propose a field.

## Evidence

[`recorded_intent_scan.py`](recorded_intent_scan.py) re-derives every number in this ADR
and exits non-zero if any stops reproducing. It asserts several claims in **both**
directions — that all 7 ramps overrun their element's `end` *and* that 0 land on it, that
only one ramp offset exists — because the decision turns on those boundaries and not on the
counts alone. It caught an arithmetic error in this ADR's own first draft.

[`docs/research/juries/recorded-intent/`](../research/juries/recorded-intent/) — both
questions and all thirty-six ballots verbatim, with the anchoring caution that qualifies
round 2's margin.
