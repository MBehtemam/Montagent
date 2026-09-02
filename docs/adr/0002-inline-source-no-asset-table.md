---
status: accepted
---

# Elements name their files inline; there is no asset table

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

## Consequences

- `validate` must check that every `source` resolves. That is the failure class
  being avoided, and nothing else will catch it.
- Renaming a file is a replace-all-occurrences edit rather than a single one.
- Cached probe data (durations, dimensions) has no home in the project file. It
  belongs in a gitignored sidecar cache, where it cannot go stale in the source of
  truth or churn diffs with data nobody wrote.
- Should a table ever be added, aliases must be semantically loaded (`@intro_bg`,
  never `@a3`): identifier meaningfulness measurably affects model accuracy.
