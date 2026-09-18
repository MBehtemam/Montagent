---
status: accepted
amended-by: 0007 (fonts carve-out), 0053 (resolution base, no assetRoot, absolute paths permitted, missing-source severity)
---

# Elements name their files inline; there is no asset table

> **Amended by two later ADRs.** Read them before relying on anything below.
>
> - [ADR-0007](0007-text-runs-literal-size-declared-fonts.md) — fonts carve-out
> - [ADR-0053](0053-asset-path-resolution-no-assetroot.md) — settles the resolution-base,
>   assetRoot, absolute-path and missing-source questions that ADR left open

An element's `source` is written on the element itself — a relative path today, a
URL where the file lives elsewhere. Montaget has no `assets` block declaring
files once and referring to them by id, and "asset" is not part of its
vocabulary.

The alternative is FCPXML's `resources` + `IDREF` pattern: declare each file once
under a name, then write `"source": "@img05"` at each use. It is genuine prior
art and the project's renderer survey recommends stealing it. It is rejected here
anyway.

## Why

**Nobody authoring for machines does it.** All four declarative video APIs
surveyed — Shotstack, Creatomate, Editly, JSON2Video — write the file location
directly on the element. None has an asset table. All four are HTTP services, so
they had the strongest possible reason to want one.

**The lookup is a tax paid on every read.** `"@img05"` tells a reader nothing
until they go and resolve it. `"images/05.png"` is the answer. This is the
project's inert-data principle applied at the smallest scale.

**Indirection creates an error class that inline cannot produce.** A dangling
reference passes JSON Schema validation; a wrong path fails at render with an
obvious cause. Agent-hallucination research names "parameter hallucination" —
asserting plausible ids that do not exist — as a common, distinct failure mode.

**Anthropic's own guidance on writing tools for agents** says to "eschew
low-level technical identifiers" and reports that resolving opaque ids to
meaningful language "significantly improves Claude's precision … by reducing
hallucinations". Their context-engineering guidance recommends the opposite —
lightweight references, loaded just in time — but for a different scope:
references are for data *not* in context. An asset table sits in the same
document the model is already reading, so it buys nothing and costs a hop.

**Deduplication mostly evaporated** once the timeline became flat. One image on
screen for fourteen seconds is one element, not four references.

## The strongest argument against, recorded honestly

Refactoring benchmarks find incomplete edits — agents updating some occurrences
and missing others — to be a dominant agent failure mode. If one path appears at
a dozen positions, a table needs one edit and inline needs twelve correct ones.
For the *editing* task specifically this is better evidence than anything on the
inline side.

Two things answer it. The risk scales with occurrence count, and the flat model
keeps counts low. And the fix belongs in tooling — a replace-all-occurrences
operation, or a normalising pass — rather than in a format that taxes every read
to serve an occasional edit.

Note also that no published study tests this question directly. The verdict rests
on inference from adjacent results, at moderate confidence. The cheap way to
settle it properly is to build one fixture in both encodings and run a matched
eval on reading, generation and editing.

**Correction, from [ADR-0007](./0007-text-runs-literal-size-declared-fonts.md):
the rebuttal above is true for media and false for fonts, measured in this very
fixture.** "The risk scales with occurrence count, and the flat model keeps counts
low" holds for `source` — 16 distinct media files, 28 references, **mean 1.75**, max
4. It fails for fonts: **one font, referenced by all 22 text elements**. A flat
timeline reduces media reference counts because one image on screen for fourteen
seconds is one element; it does nothing to a property that recurs on *every* text
element regardless of duration. Read the sentence as scoped to media, not as general.

## What this ADR does not cover: fonts

**The evidence base was gathered about media locations and contains no font data.**
`docs/research/declarative-video-api-models.md` — the survey this ADR's "nobody
authoring for machines does it" argument rests on — contains the string "font"
**zero times**. Surveyed afresh, the same reference class points the other way:
Shotstack (`timeline.fonts`), Creatomate, CSS `@font-face`, Remotion `loadFont` and
ASS's own `Style:` block all *declare* fonts centrally. The one system that inlined
the location into the name slot, JSON2Video, **forfeited ordered fallback as a
structural consequence** — a slot that may hold a URL cannot also hold a chain.

Three of this ADR's five arguments still transfer to fonts intact (read tax,
dangling references, id hallucination); the dedup argument does not, and hermeticity
has nowhere to live under the inline form — "the set of files the renderer may open"
would exist only as the union of 22 element declarations. So ADR-0007 carves fonts
out **without disturbing `source`**, under a discriminant that keeps the format at
one rule:

> A value is written inline unless it **(i)** names bytes outside the document, **and
> (ii)** belongs to a value set that is closed, small, and scaled by policy rather
> than by content.

Media source fails (ii) — open, content-scaled, and growing. Colour, size and
position fail (i). **Font passes both, and today it is the only thing that does** —
which is the point, and what stops the slide into templating.
`source` never gets a table.

Consequence #4 below is thereby **discharged rather than overridden**: it
pre-authorised the shape, and ADR-0007 binds it — alias keys are author-chosen
semantic names (`brand`, `brand+fa`), never `f1`, and never the family name embedded
in the font binary, which would reintroduce the system-font nondeterminism the whole
decision exists to kill.

## Consequences

- `validate` must check that every `source` resolves. That is the failure class
  being avoided, and nothing else will catch it.
- Renaming a file is a replace-all-occurrences edit rather than a single one.
- Cached probe data (durations, dimensions) has no home in the project file. It
  belongs in a gitignored sidecar cache, where it cannot go stale in the source of
  truth or churn diffs with data nobody wrote.
- Should a table ever be added, aliases must be semantically loaded (`@intro_bg`,
  never `@a3`): identifier meaningfulness measurably affects model accuracy.
