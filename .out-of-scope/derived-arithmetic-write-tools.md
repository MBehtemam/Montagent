# Derived-Arithmetic Write Tools (layer restack, track packing, and similar)

This project does not ship write tools whose whole job is to compute a value
that is already derivable from data sitting in the document — even when the
computation is tedious, repetitive, or error-prone to do by hand.

## Why this is out of scope

[ADR-0011](../docs/adr/0011-tool-surface-reads-checks-renders.md) fixes the
tool surface at reads, checks, and renders, plus `shift` as the sole
authorized *write*. [ADR-0037](../docs/adr/0037-derived-time-signature-is-a-provenance-gap-not-a-tool.md)
sharpened the test for admitting a tenth verb: **does the tool have an input
the agent lacks?** It was applied there to an emit-once "derive a timestamp"
tool (`end - 300`) and the answer was no — the values are computable from
`(start, end, in, out)`, already in the file. No tool shipped.

`shift` clears that bar for a different reason than "the agent could get it
wrong": it preserves a *relational invariant across many coupled values, over
time* — when a source instant moves later, `shift` re-derives every dependent
value so the invariant (slack sizes, in this case) stays intact. That is
maintenance of a live relationship, not a one-time computation. A tool that
computes a value once and writes it as a plain literal produces output
indistinguishable from something typed by hand the moment it lands in the
document — nothing records that it was derived, and nothing keeps it correct
when the inputs it was derived from move again. That is a **provenance gap,
not a tool gap**, and ADR-0037 declined to paper over it with a tool.

The same reasoning extends to any proposed write tool of this shape:

- **Layer restack** — assign every element a unique layer preserving current
  resolved stacking order. The resolved order is already obtainable from the
  document via `query --at` (anchors resolve in one hop); nothing external is
  needed. Once written, the new layer integers are plain literals — an
  overlapping edit later can retie them and nothing marks the old assignment
  as derived-and-now-stale.
- **Track packing** — bin-pack elements into the minimum number of
  non-overlapping tracks. Also fully computable from `start`/`end`, already
  in the document. Needing to *re-run* the packer whenever timing changes
  during iteration is the signature of a dead literal going stale, not
  evidence of an ongoing invariant a tool is maintaining — the opposite of
  what earns `shift` its exception.

Difficulty or tedium at scale (e.g. 84 elements across 32 tracks) is a real
ergonomic cost, but it is not an *input* the agent lacks, and ADR-0037's
discriminator is deliberately about inputs, not effort. Admitting
scale-driven exceptions would swallow the rule: nearly any tedious-but-fully-
computable transform over a large document would then clear the bar.

Layer restacking has a second, independent defect: "preserving current
resolved stacking order" requires the agent to commit to what it believes
that order to be. A tool that silently adopts the engine's own resolution
turns an ambiguity the author should confront (which element wins a tie, and
why) into a decision made invisibly by the tool — that is authorship, not
arithmetic, and ADR-0060 already routes exactly this decision back to the
author via `validate`'s layer-tie error.

If the underlying need — not re-deriving values that go stale as the
document changes — is real and recurring, the right shape for it is a
**provenance** feature (a derivation-claim field the renderer ignores but
`validate`/`compare` can check against a live recomputation), not a write
tool. That fog entry already exists, graduated from #69/ADR-0036, and is
where evidence of this need should accumulate instead of a new verb.

## Prior requests

- #318 — "Add track/layer packing as arithmetic write tools, like shift"
