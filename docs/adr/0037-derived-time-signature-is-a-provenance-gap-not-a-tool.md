---
status: accepted
amends: 0011 (no tenth verb; the nine-tool surface holds), 0012 (derived keyframe values stay literals, no computed-field escape hatch)
---

# Derived times carry no signature, but that is a provenance gap, not a tool gap — none ships

> **Amended by [ADR-0086](0086-recorded-intent-is-one-pattern-and-the-time-axis-instantiates-it.md) — its evidence is withdrawn as unreproducible.**
> The *"52 hand-typed absolute timestamps"* of which four (`8000`, `16800`, `25700`,
> `35300`, each `end − 300`) carry no signature **do not exist**: those values appear in
> no committed version of the fixture, which carries 135 timeline instants and no
> `highlight` at all. **This ADR's disposition stands** — no tool ships, the
> input-the-agent-lacks discriminator is untouched, and the re-diagnosis of the problem as
> *provenance, not computation* is correct and is ADR-0086's premise. Only the census is
> withdrawn; ADR-0086 replaces it with a re-executable one
> ([`recorded_intent_scan.py`](recorded_intent_scan.py)). The **"inert provenance"** need
> this ADR deferred to the map's fog is designed there.

[Ticket #68](https://github.com/MBehtemam/Montagent/issues/68), re-scoped from
[#12](https://github.com/MBehtemam/Montagent/issues/12) after the preset-catalog framing was
measured and killed. What survived that kill was a discriminator — *"does the tool have an
input the agent lacks?"* — and two candidate needs it left standing. This ADR closes the
ticket by applying that same discriminator to the one candidate that reached it still alive.

## What was already settled, carried in from ADR-0035

#68 was blocked on the frame-grid question: *"a documented recipe cannot know the project's
`fps`; an expander can."* [ADR-0035](./0035-keyframe-grid-alignment-is-a-review-check-not-a-schema-rule.md)
resolved that off-grid keyframe `t` is legal and the renderer's rounding rule is published —
both conditions #68 itself named as sufficient to make this input evaporate. It has. Only the
first candidate — **derived times carry no signature** — reaches this ADR.

## The evidence

One authored project: 52 hand-typed absolute timestamps. Most cross-check against another
value already in the file — a keyframe `t` equal to its own element's `start`, for instance.
Four do not: caption hold-out records at 8000, 16800, 25700, 35300 ms, each equal to
`end − 300` for its element, with nothing in the file recording that relationship. Get one
wrong and a caption fades early or snaps off; neither `validate` nor a human reading the file
would ever say why, because the file has no way to say the value was derived at all.

## Applying #12's own discriminator

The question is not whether this is a real defect — it is. The question is whether it clears
the bar that already killed the preset framing: **does a tool here have an input the agent
lacks?** `fonts` and `measure` pass that test because the system font table and the shaper's
opinion are genuinely absent from the document. Derived-time arithmetic does not: `end − 300`
is computable from `(start, end, in, out)`, data already sitting in the file. Writing
`end - 300` in a throwaway script and typing `35300` directly into the document draw on
exactly the same knowledge. No project fact is missing.

**What's actually missing is not an input to a computation — it's a record of which values
were derived, and how.** That is a different problem than the one the discriminator was built
to detect, and a tool that only computes (an emit-only helper returning a keyframe record, on
the `measure` precedent) does not touch it: its output is a dead literal the instant it's
pasted into the document, indistinguishable from a value typed by hand. It would not have
caught the four caption-hold values, because nothing marks their output as derived rather
than authored — the gap survives the tool.

**The case considered and rejected.** An emit-only derive tool was weighed on safety grounds
— that manual timestamp arithmetic is exactly the failure mode ADR-0005 already lines up
against by making `shift` "the one edit Montagent performs instead of the agent." This doesn't
hold: `shift` earns that role because it preserves document-wide relationships across many
coupled values in one mutation, an invariant an agent cannot maintain by hand-editing.
Computing a single derived timestamp has no such invariant at write time — the invariant that
actually matters here is *over time*, after the source value (`end`) later moves and the
derived value (the hold-out) does not follow it. No emit-once tool can hold that invariant,
because it never sees the second edit. Shipping one anyway would consume one of the nine
verbs for zero durability, and let the project believe the drift hazard was handled when it
was not.

Resolved by a jury of three independent models (Opus, Sonnet, Haiku), unanimous 3/3.

## Disposition

**No authoring tool ships for derived-time computation.** The real need is a record of
derivation — provenance — not a computation the agent already has the means to perform. That
need is already open, correctly unticketed, as **"inert provenance"** in the map's fog,
graduated from [#69](https://github.com/MBehtemam/Montagent/issues/69) /
[ADR-0036](./0036-shift-preambles-coincident-instants-validate-and-compare-stay-out.md): a
renderer-ignored derivation-claim field on the time axis, structurally
[ADR-0015](./0015-fit-is-a-derivation-claim-and-gravity-retires.md)'s `fit` transplanted, left
unticketed because its shape is still underdetermined. This ADR does not resolve that shape.
It transfers evidence to it: the four `end − 300` values and the 4-of-52 unsignatured ratio
become that fog entry's first pricing datum, so the finding this ticket measured is not lost
when #68 closes.

**Doing nothing at the tool-surface level is a live answer, and it is the one adopted here.**
The nine-verb surface from ADR-0011 gains nothing; no tenth verb, no computed-field escape
hatch into ADR-0012's flat keyframe model.

## Consequences

- No new tool ships. The tool surface (ADR-0011) is unchanged.
- The "derived times carry no signature" hazard is not solved by this ADR — it is
  re-diagnosed as a provenance gap and its evidence is handed to the already-open **inert
  provenance** fog entry (graduated from #69/ADR-0036) as pricing data, not as a new ticket.
- No schema change. No renderer change.
