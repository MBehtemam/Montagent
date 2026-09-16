---
status: accepted
amends: 0005 (its writing-convention sentence stops being folklore and becomes a checked MUST, with a stated order), 0006 (`validate` gains a fourth report category, `LAYOUT`, alongside `error`/`review`/`note`/`UNCHECKED`), 0011 (`fmt` gains a non-destructive `--check` mode; the write-tool invariant is restated for key order), 0016 (closes the "published key order" entry left unsettled there)
---

# Canonical key order is schema property order; `validate` checks it; `fmt` splits into `--check` and write

[Ticket #73](https://github.com/MBehtemam/Montaget/issues/73), from [#12](https://github.com/MBehtemam/Montaget/issues/12)
and sharpened by [#8](https://github.com/MBehtemam/Montaget/issues/8) and
[#16](https://github.com/MBehtemam/Montaget/issues/16). [ADR-0005](./0005-absolute-integer-milliseconds.md)
already required elements to be written "sorted by `start` within a track, one element per
line, stable key order," and said this is load-bearing *because agents edit by exact-string
replace* — but it published no actual order and nothing checked it. Two incidents followed.
One agent predicted the failure live: *"if a tool reorders my keys, my next exact-string
replace gets zero hits."* A different agent then demonstrated it by accident — asked for a
routine edit, it rewrote the real 154-line fixture into 1595 lines, silently pretty-printing
the whole file and destroying the one-element-per-line convention, reporting "no schema
errors." By the letter of every existing rule it was correct: **nothing forbade what it
did.** [ADR-0016](./0016-no-format-version-the-unknown-key-error-is-the-mechanism.md) had
already logged this as unsettled, with independent corroboration: 3 of 9 agents in an
unrelated exercise also pretty-printed the fixture (154 → 1146/1165/1243 lines), unprompted,
while doing otherwise careful work.

Resolved by a jury of three independent models (Opus, Sonnet, Haiku) on four sub-questions,
run from a brief that gave the incident evidence and this project's governing principles
(`validate`'s noise budget, its single-document/no-intent-assertion scope, `compare`'s
before/after scope) without steering toward an answer. **Unanimous 3/3 on three of the four
questions; split 2–1 on the fourth.** Ballots committed in full at
[`docs/research/juries/key-order-and-fmt/`](../research/juries/key-order-and-fmt/README.md).

## Decision

### 1. The convention is a published `MUST`, and something checks it (unanimous 3/3)

Not folklore a formatter happens to produce, not a documented `SHOULD` with no tooling —
a requirement `validate` and `fmt --check` both verify. The incident is decisive on its own:
the convention already existed, was already predicted to break, and broke anyway, because
nothing checked it. A rule every future edit structurally depends on but that nothing
verifies is not a convention, it is a latent outage.

### 2. Canonical key order is a universal prefix, then each type's property order in the published schema (2/3)

Every element is written `id, type, group, start, end, ...`, in that order, `group` omitted
entirely (not written as `null`) when the element carries none — matching what every element
in the committed fixture already does and what
[CONTEXT.md](../../CONTEXT.md)/[ADR-0004](./0004-tracks-as-constrained-lanes.md) establish as
the fields every element shares regardless of type.

After the prefix, **the order is whatever order that element `type`'s properties are declared
in the published JSON Schema** — not a second, hand-maintained list. This was the closer of
two votes (Opus and Sonnet for a per-type order — Opus specifically for a schema-tied one;
Haiku for a single global order applied to whatever keys are present). The deciding argument,
from Opus's ballot: a bare global sequence is "defined over a set the schema does not close,"
so adding one field to one type forces a decision that silently reshuffles every other type's
line. Tying order to the schema instead of a parallel document means there is exactly one
place key order can go stale — the schema itself — rather than two artifacts that can drift
apart, which is the identical failure this map has now found twice in jury evidence
([#72](https://github.com/MBehtemam/Montaget/issues/72)) and once in `validate`'s own
measured facts ([ADR-0006](./0006-validate-reports-facts-and-render-enforces.md)'s stale
severity claims). It also matches this project's own standing principle that **the format is
discoverable through its published schema**, not through a document that merely describes it.

**Measured against the committed fixture, this costs zero bytes.** The four element types the
fixture actually uses are already internally consistent — every element of a given type
already shares one exact key sequence (with `speed` on audio and `mask` on one image
appending after their type's core fields, still in a stable relative position) — so
formalizing "the order is the schema's order" and setting the not-yet-written schema to match
what is already on disk changes nothing:

| type | order (prefix already covered above) |
|---|---|
| `image` | `source, x, y, origin, width, height, fit, clip, scale` |
| `rect` | `x, y, origin, width, height, fill` |
| `text` | `x, y, origin, width, height, font, size, line_height, color, align, runs` |
| `audio` | `source, source_start, source_end` |

`video` and `ellipse` have no committed instance to measure yet; per the rule above, whichever
ADR first fixes their full property set (they inherit `image`'s and `rect`'s shape
respectively) fixes their order too, at the point they're first authored — not invented here
speculatively, the same discipline this map applies everywhere else (evidence before rule).
Any transform, paint or effect property this ADR doesn't enumerate — `rotation`, `opacity`,
`layer` (the per-element stacking override), `stroke`, `effects` — gets its position from the
ADR that introduces it into a given type's schema, in the order that ADR adds it; nothing
here freezes a field's position ahead of the decision that creates the field.

### 3. Checked in both `validate` (unconditional) and `fmt --check` (unanimous 3/3)

`fmt --check`-only was rejected outright: it is exactly the opt-in shape this project has
already rejected every time it's come up ([ADR-0032](./0032-slack-is-invariant-shift-refuses-compare-is-the-backstop.md),
[ADR-0036](./0036-shift-preambles-coincident-instants-validate-and-compare-stay-out.md),
[ADR-0039](./0039-group-keyframe-time-check-retracted-from-validate-reassigned-to-compare.md)) —
the incident agent was not running a formatter, it believed it was making a routine edit and
had no reason to invoke one. `validate` already runs unconditionally, including on files
nobody ever ran `fmt` over, which is exactly the file the incident produced.

This does not reopen `validate`'s noise budget, because layout compliance is a deterministic
property of the bytes, not a judgment call — it fires on exactly the non-compliant files and
never on a correct one, clearing the "an opt-in check is worth nothing, a noisy one is worse"
bar cleanly in both directions.

It does, honestly, widen what `validate` reports: layout is not "internally legal" in the
semantic sense [ADR-0006](./0006-validate-reports-facts-and-render-enforces.md) defined, and
it never makes a render *wrong* — the `error`/`review`/`note` ladder is keyed to render
correctness (`error`: "the render is refused or is guaranteed wrong"; `review`: "you must look
at a frame to know if it was meant"; `note`: "a fact you may want and will not act on today"),
and a key-order violation fits none of those, since the video renders identically either way.
Rather than force it into that ladder, `validate` gains a **fourth report category, `LAYOUT`**,
the same shape as [ADR-0013](./0013-fitted-extents-floor-and-the-nine-origin-keywords.md)'s
`UNCHECKED` — validate-only, counted in the summary line, and **not one of the categories
`render` refuses on**. `render` keeps refusing only on `error`; a layout-noncompliant file
still renders, because nothing about key order or line breaks changes what a frame looks
like. What `LAYOUT` buys is that the file stops being silently unsafe to edit, not that it
stops being renderable.

A `LAYOUT` finding names the element, its line, and the fix:

> `LAYOUT` — `photo-06` (line 42): key order does not match the schema for `image`; expected
> `id,type,group,start,end,source,x,y,origin,width,height,fit,clip,scale`. Run `montaget fmt`.

### 4. `fmt` splits into `--check` and its existing write mode (unanimous 3/3)

[#61](https://github.com/MBehtemam/Montaget/issues/61) found `fmt` may also materialize (or,
per [ADR-0030](./0030-defaultable-field-presence-is-content-fmt-leaves-it-alone.md), now must
*not* materialize) defaulted fields. Bundling a key reorder and any other rewrite into one
silent pass produces a diff whose size and content conflate independent, separately-arguable
changes — precisely what happened in the incident, just with two hazards instead of one.
`fmt --check` reports what a run would change without writing anything; `fmt` (bare) writes.
Both share one implementation of "what does canonical form look like" — the same rule
`validate`'s `LAYOUT` check reads — so there is exactly one place the rule lives, not two that
can disagree.

## Consequences

- **`validate` gains a `LAYOUT` category**, `validate`-only like `UNCHECKED`, never gating
  `render`'s refusal — refusal stays keyed to `error` alone.
- **`fmt` gains `--check`**, non-destructive, sharing its comparison logic with `validate`'s
  `LAYOUT` check.
- **No schema file exists yet to carry the per-type property order this ADR ties key order
  to** — this project is pre-implementation. When the JSON Schema is written, its `properties`
  declaration order for each type *is* the canonical key order by this ADR's rule; the table
  above is what that order must be for `image`, `rect`, `text` and `audio`, verified against
  the one committed project file.
- **Zero bytes change in the committed fixture.** The order this ADR requires is the order
  already on disk.
- **The universal prefix (`id, type, group, start, end`) and per-type tail apply within an
  element only** — this does not touch element-to-element ordering, which
  [ADR-0005](./0005-absolute-integer-milliseconds.md) already fixed (sorted by `start` within
  a track) and this ADR does not reopen.
- **`video` and `ellipse` key order is deferred, not invented**, to whichever future ADR first
  fixes either type's full property set.
