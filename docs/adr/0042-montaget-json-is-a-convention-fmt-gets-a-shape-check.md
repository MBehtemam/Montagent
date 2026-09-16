---
status: accepted
amends: 0011 (adds a precondition to `fmt`), 0016 (discharges its file-identification deferral)
---

# `.montaget.json` is a documented convention, not an enforced or in-document marker; `fmt` gets a shape check

[ADR-0016](./0016-no-format-version-the-unknown-key-error-is-the-mechanism.md) settled that a
project file carries no version integer, and explicitly left file *identification* open as a
separate decision — constrained only by "if a marker is ever added it must not be an integer."
Both project files on `main` are named `*.montaget.json`, but the convention appears in zero
accepted ADRs and zero mentions in `CONTEXT.md`. [#14](https://github.com/MBehtemam/Montaget/issues/14)
found no evidenced consumer for identification at all — every verb in
[ADR-0011](./0011-tool-surface-reads-checks-renders.md) takes a caller-chosen path, and Montaget
never sweeps a directory — while separately flagging that `fmt`'s write bit has no precondition:
pointed at a mis-handed, unrelated JSON file (a transcript export, a beats export), it will
rewrite it. [#77](https://github.com/MBehtemam/Montaget/issues/77) asked three questions: is the
extension normative, does the file need an in-document marker, and what precondition `fmt` needs.

## Decision

**The extension is a documented convention, never enforced. No in-document identifying marker
exists. `fmt` (and, by the same predicate, `validate`/`render`) refuses to act when the file is
not project-shaped — missing `tracks`, `fps`, or `frame` — but proceeds regardless of any
`error`/`review`/`note` findings on a file that is recognizably a project.**

1. **Extension: convention, not a gate.** `.montaget.json` is written down as the naming
   convention for new files and the name tools emit by default. No tool inspects or enforces
   it. A caller has already asserted "this is a Montaget project" by choosing the path; only
   the contents can test that assertion, and a filename check either duplicates the schema
   check or wrongly rejects valid data with an unconventional name (a pipe, a temp path, a
   file produced by another tool).

2. **No in-document marker.** The schema's required top-level keys are already a stronger,
   redundant-free discriminator than a fixed marker string: a transcript or beats export
   fails them immediately, and a marker would only ever fire in cases that check already
   catches, or never fire in a case it misses. A required `"format"`-style field also carries
   its own version-number-shaped risk — agents treating `"montaget"` vs. a hypothetical
   `"montaget-2"` as a versioning axis, reopening exactly the confusion ADR-0016 closed. The
   actual gap #77 found is message quality, not detection: a file missing its required keys
   wholesale should get a message naming the likely mismatch ("this does not look like a
   Montaget project file — no `tracks`/`fps`/`frame`"), not a bare schema-mismatch dump. That
   is a wording fix to `validate`'s and `render`'s error text, not a format change.

3. **`fmt` gets a structural shape check, not a correctness gate.** `fmt`'s job is
   reformatting, not correctness-checking — ADR-0006 already separates "is this legal" from
   "does it say what you meant," and a `note`- or `review`-level finding does not make a file
   any less a legitimate, safely-formattable Montaget project. The precondition is narrow:
   refuse only when the document is missing the required top-level keys that make it
   recognizable as a project at all. `fmt` already needs to know the document's shape to order
   keys canonically, so this is a refusable version of a check it was implicitly performing —
   not new machinery. `validate` and `fmt` should share one "is this a Montaget project
   document" structural predicate rather than each reimplementing it.

## Evidence

Three separate three-model courts (Claude Opus, Claude Haiku 4.5, Claude Fable 5.1 — each
blind to the others' ballots), one per question. **Unanimous 3/3 on all three questions**,
each panel independently reaching the recommended answer by its own route.

- **Extension normativity**: 3/3 for convention-only. Independently-named reasoning: filename
  checks earn their keep only in directory-sweep discovery flows, which ADR-0011 already
  removed; a filename check adds no true positives over the schema check and only false
  negatives; and enforcement would make the filename a load-bearing identity signal, the exact
  out-of-band-metadata pattern ADR-0016 eliminated for version numbers.
- **In-document marker**: 3/3 against. Independently-named reasoning: no evidenced consumer
  (confirming #14's finding from the producer side); the schema's required keys already
  discriminate with full precision on the sibling files that motivate the concern; and a fixed
  marker string risks becoming a de facto version field the moment a second value is ever
  proposed.
- **`fmt` precondition**: 3/3 for the structural-shape-only refusal. Independently-named
  reasoning: the reported hazard is wrong-file destruction, not wrong-content tolerance, so the
  proportionate fix is an identity check, not a correctness gate; gating on a clean `validate`
  pass (the rejected third option) would make `fmt` unusable exactly when most wanted — mid-authoring,
  with open `review` findings — and would let "does it say what you meant" hold a byte-level
  rewrite hostage, which ADR-0006's separation forbids.

## Consequences

- `CONTEXT.md`/authoring docs should recommend `.montaget.json` for new files without any tool
  ever checking it.
- `validate` and `render` gain improved error text for the wholesale-missing-required-keys
  case, naming the likely mismatch — a wording change, not a schema or tool-surface change.
- `fmt` gains a refusal path on documents missing `tracks`/`fps`/`frame`, implemented as (or
  sharing) the same structural predicate `validate`'s schema layer already uses. It does not
  gain any dependency on `validate`'s severity levels.
- ADR-0016's file-identification deferral is discharged: no marker, ever, unless a real
  consumer is later measured — at which point adding an optional, non-integer field is a
  backward-compatible schema change, not a reopening of this ADR.
- No schema change. No new tool-surface entries. ADR-0011's eleven-command surface stands.

## Not settled here

- The unknown-key policy itself ([#75](https://github.com/MBehtemam/Montaget/issues/75)), which
  the structural predicate here is independent of.
- The exact wording of the improved error message — left to implementation, constrained only by
  "name the likely mismatch, don't dump the raw schema error."
