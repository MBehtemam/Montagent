---
status: accepted
amends: 0011 (the five-code exit ladder stays five; `LAYOUT` never gates `fmt --check`'s
  exit code, exactly as it never gates `validate`'s or `render`'s), 0041 (ratifies
  `L-LAYOUT` as a second `LAYOUT` finding shape, whole-file rather than element-scoped,
  alongside `L-KEY-ORDER`; ratifies that the header's and each track's own key order are
  schema property-declaration order, the same rule ADR-0041 states for an element; and
  clarifies that a `LAYOUT` finding names only the keys a structure actually carries, never
  the ones it omits), 0006 (the `error`-only exit-code rule extends uniformly to `LAYOUT`
  wherever it appears, `validate` included — settling the identical question `validate`
  raises alongside `fmt --check`)
---

# `fmt --check` stays exit 0 on a non-canonical file; `L-LAYOUT` is ratified

**Ticket:** [#241](https://github.com/MBehtemam/Montaget/issues/241), surfaced by
implementing [#193](https://github.com/MBehtemam/Montaget/issues/193) (`fmt`, the
canonical convention, the atomic write and the key-order predicate). Three things
`crates/montaget-core/src/verbs/fmt.rs` and `crates/montaget-core/src/registry.rs` raised
inline rather than silently deciding, per `docs/agents/domain.md`'s *"if your output
contradicts an existing ADR, surface it explicitly."* Nothing in `#193`'s shipped code
changes; this ADR is where the surface it invented becomes spec.

## 1. `fmt --check` (and `validate`) stay exit 0 on a `LAYOUT`-only file

**Ratified as shipped: no sixth exit code.** ADR-0011's ladder is five codes, keyed to
*"what the caller does next,"* and *"exit non-zero only on `error`"* is not a rule about
`error` specifically — it is the whole ladder's discipline, restated by ADR-0041 for
`LAYOUT`: *"not one of the categories `render` refuses on."* A `LAYOUT` finding is true on
a file that renders byte-identical to its canonical form. Nothing about the video is
wrong, and a code the ladder does not already have would exist for exactly one condition
that is not a defect — the one case ADR-0011's own test (*"the only distinction that pays
for itself"*) rejects a code for.

Story 5's *"I want `fmt --check` to tell me the file is in canonical convention without
rewriting it, so that I can verify before I commit"* is satisfied without a new code: the
tool already tells the caller, in the one channel built for a caller who wants to gate on
something exact rather than the coarse pass/fail the ladder gives every other check —
`--json`'s counted `summary.layout` field:

```
montaget fmt p.montaget.json --check --json | jq -e '.summary.layout == 0'
```

This is the same shape ADR-0013 already established for `UNCHECKED` — a report category
that is counted and inspectable but does not move the exit code — extended to `LAYOUT`,
which ADR-0041 explicitly modeled on it.

**This settles `validate` identically, not by extension but because it is the same
question.** `validate`'s `LAYOUT` check is the same predicate as `fmt --check`'s (ADR-0041,
*"exactly one place the rule lives, not two that can disagree"*), so a `validate` run on a
non-canonical, otherwise-legal file is also exit 0, gated the identical way through its own
`summary.layout` field. A hook or CI step that wants a hard gate on canonical form composes
one `jq` check onto either tool; the ladder does not grow a code that exists to serve one
caller's polling habit when every other `LAYOUT`-adjacent category in this series already
answers that need through the summary line.

**Why not add the code anyway, given it is cheap?** Because it is not free on the read
side: ADR-0011's five rows are memorized as *"what number means what,"* and a sixth row
whose meaning is *"legal, but see the summary line for whether you care"* is a distinction
every future reader of the table has to hold, for a condition `--json` already answers
exactly. The table stays five rows.

## 2. `L-LAYOUT` is ratified as a second, whole-file `LAYOUT` finding shape

ADR-0041 specifies one finding shape — *"a `LAYOUT` finding names the element, its line,
and the fix"* — which is `L-KEY-ORDER`, element-scoped. `#193` registered a second code,
**`L-LAYOUT`**, for everything a canonicalising rewrite changes that is not one element's
own key order: line breaks, indentation, the header's and each track's own key order, the
element sort ADR-0005 requires, and the trailing newline.

**Ratified as shipped, for the reason `registry.rs`'s own comment already gives**: the
incident ADR-0041 was written about — the agent that rewrote the 154-line fixture into
1595 lines — disturbed not one key's position. A `--check` that only ever fired
`L-KEY-ORDER` would report nothing on the exact file the whole ADR exists to catch. One
finding, `L-LAYOUT`, covering the file as a whole, is `crate::checks::layout`'s measured
diff between the document as written and the document canonicalised with each element's
own key order left alone — by subtraction, not assumption, so a file whose only fault is
one element's key order still fires `L-KEY-ORDER` alone and never a redundant `L-LAYOUT`
alongside it.

**The header's and each track's key order are ratified as schema property-declaration
order** — the same rule ADR-0041 states *"within an element only"* — because
`crate::layout::Published::Project` and `Published::Track` already read both from the
identical generated-schema cache `Published::Element` does (`crates/montaget-core/src/layout.rs`),
and that is the only reading consistent with ADR-0041's own justification for the rule:
the published schema is the one order-bearing document already in the project, and nothing
about the header or a track is exempt from the reasoning that put an element's fields in
schema order.

**`L-KEY-ORDER`'s `expected` list names only the keys the element carries, never the ones
it omits.** ADR-0041's worked example — `expected id,type,group,start,end,source,x,y,
origin,width,height,fit,clip,scale` for `photo-06` — reads, out of context, as the full
published order for `image`. It is not: the committed fixture's `photo-06` carries every
key that list names, so the example is silent on the omitted-key question and was
misread as settling it. Naming an absent key in a repair message would read as an
instruction to add it, which ADR-0030 forbids `fmt` from doing on its own — a defaultable
field's absence is a declaration, not an omission `fmt` may correct. `#193`'s reading —
name only what is carried — is the one consistent with ADR-0030, and is ratified.

## Consequences

- `crates/montaget-core/src/registry.rs`'s `#241` comment above `L-LAYOUT` is replaced
  with a citation to this ADR; the code does not change.
- `crates/montaget-core/src/verbs/fmt.rs`'s doc comment on `pub fn fmt`, which stated the
  tension and raised `#241` inline, is updated to state the ratified answer instead.
- **ADR-0011** gains an "Amended by" banner pointing here.
- **ADR-0041** gains an "Amended by" banner pointing here.
- **ADR-0006** gains an "Amended by" banner pointing here, for the identical
  `validate`-side answer to the same exit-code question.
- No functional code change and no test changes: this ADR ratifies what `#193` shipped and
  what `crates/montaget-core/tests/fmt.rs`, `tests/validate.rs` and `tests/registry.rs`
  already assert.
