---
status: accepted
amends: 0002 (settles the resolution-base, assetRoot, absolute-path and missing-source questions that ADR left open)
---

> **Amended by [ADR-0159](0159-a-remote-instance-moves-bytes-through-a-plain-http-door-into-named-workspaces.md)**, for a remote instance only. An absolute path, or a `..`
> that leaves the workspace, is refused as a named `validate` finding. Over stdio, absolute
> paths stay permitted as below.

> **Amended by [ADR-0131](./0131-render-and-frame-use-local-sources-only-and-validate-says-so.md)**,
> which records that `render` and `frame` use local sources only. Rewriting `source` to URLs
> is still how a project refers to a remote store, but that project does not render until
> each source is a local copy again. `validate` probes the URLs and says so.

> **Amended by [ADR-0056](./0056-remote-source-probe-session-scoped-no-persistent-cache.md)**,
> which discharges the four remote-specific questions this ADR deferred: probe
> cadence, what a remote probe fetches, cache key, and what a probe *failure*
> (as distinct from a probe finding this ADR's plain `error`) means.

# Asset paths resolve against the project file's directory; no `assetRoot`; absolute paths permitted; a missing source is a plain error

[ADR-0002](./0002-inline-source-no-asset-table.md) settled that `source` is a path
or URL written inline on the element, with no asset table. It left four questions
unanswered: what a relative path resolves against, whether a top-level `assetRoot`
field is adopted so a project can be relocated without touching every element,
whether an absolute local path is permitted, and what `validate`/`render` do when
a `source` file is simply missing. Graduated from [#4](https://github.com/MBehtemam/Montagent/issues/4)
via the map's "Asset resolution" fog entry.

## Decision

### Relative `source` paths resolve against the project file's own directory

The only base a reader can resolve from the document plus its on-disk
neighbours, with no dependency on where the tool happened to be invoked from.
The one committed fixture already assumes this — `"images/05.png"`,
`"audio/intro-2.mp3"` and `"brand/logo-en.png"` all sit as siblings of the
`.montagent.json` file itself. Resolving against the process's current working
directory was rejected: it makes the same file mean different things depending
on invocation, which is a direct violation of file-as-truth. Requiring an
explicit base on every project was rejected as ceremony forced on every author
to serve a case the fixture doesn't exhibit.

Decided **unanimous 3/3** (Opus, Haiku, Fable).

### No `assetRoot` field

A project relocating to a remote store does so by rewriting `source` to a full
URL on the affected elements (already permitted under ADR-0002) or by moving
the local sibling folder as a unit; there is no top-level field that changes
how every `source` in the document is interpreted.

`assetRoot` was considered and rejected. The case for it — stated by the
ticket that graduated this decision as "a project can move between a local
checkout and a remote store without touching any element" — does not survive
scrutiny: `source`-as-URL already reaches every remote store ADR-0002
contemplated, so the field is not forced by anything unreachable today, only
by ergonomics on a migration that is rare and, per this project's own
write-tool-invariant precedent, a mechanical and reliably-correct edit for an
agent to make. Structurally, `assetRoot` is the asset-table indirection
ADR-0002 rejected, relocated from a per-file table to a single top-level
field: a reader can no longer tell what `"images/05.png"` resolves to by
reading the element alone, and must first check whether a distant field
exists and what it currently says. The field's *size* differs from a table;
its *shape* does not.

Decided by two rounds of `/court`. Round 1 (Opus, Haiku, Fable) split **2-1**
for adopting `assetRoot`. Round 2 added a fourth juror (Sonnet), briefed with
both round-1 positions verbatim, and split the panel **2-2** — the added
juror independently reproduced the dissent's structural objection rather than
the majority's goal-satisfaction argument. The author broke the tie for
rejection: the majority's forcing premise is false (the goal is already
reachable), and two jurors who saw the majority's own reasoning still
converged on the same ADR-0002-consistency objection rather than merely
preferring different ergonomics.

### Absolute local filesystem paths are permitted in `source`

`source` may be a relative path, an absolute local path, or a URL. `validate`
does not police portability as a property of the format — an agent pointing
at `/Volumes/footage/take3.mov` on its own machine, or a shared NAS mount, is
a legitimate authoring state, and the project takes on non-portability
knowingly rather than the format refusing it. Forbidding absolute paths was
rejected as `validate` asserting something beyond the document's own
internal and on-disk consistency — a portability *wish* enforced as a *ban*,
costing real authoring cases and buying nothing an author cannot already
achieve by choosing a relative path.

Decided **2-1** (Opus, Fable for permitting; Haiku dissenting for forbidding,
on the grounds that "no guarantee" invites silent breakage in a file-as-truth
format). The dissent is recorded rather than adopted: a project that becomes
unportable because of an absolute path fails as a clean missing-source
`error` on the next machine, which is the honest and sufficient symptom
[ADR-0006](./0006-validate-reports-facts-and-render-enforces.md) already
provides for — no new mechanism is needed to make that failure legible.

### A missing `source` file is a plain `error`

Same severity class as any other structural defect under ADR-0006 — it
blocks `render`'s refusal-on-error behaviour, with no special case for
"don't have the file yet" during early authoring. This is exactly the
on-disk-consistency check ADR-0002 already promised ("`validate` must check
that every `source` resolves"), and ADR-0006 already has the right severity
for a defect `render` genuinely cannot proceed past. Inventing a softer class
for early-authoring convenience was rejected as the same unforced-mechanism
pattern this project's decision history repeatedly rejects elsewhere.

Decided **unanimous 3/3** (Opus, Haiku, Fable).

## Consequences

- No schema change: `source` keeps its ADR-0002 shape (a path or URL on the
  element), and no top-level `assetRoot` field is added.
- A project is a movable unit: the `.montagent.json` file plus its relative
  media siblings. Relocating to a remote store is a mechanical rewrite of
  `source` values to URLs, not a one-line edit.
- `validate`'s existing `error`-class "every `source` must resolve" check
  (ADR-0002/ADR-0006) covers both a missing local file and, by extension, an
  absolute path that doesn't resolve on the current machine — no new finding
  code is introduced here.
- Remote (URL) `source` failure modes — network timeouts, caching, probe
  cost for a non-local file — are explicitly out of scope for this decision
  and remain [#127](https://github.com/MBehtemam/Montagent/issues/127)'s to
  design.
- Font paths (the `fonts` table) are governed by ADR-0007's separate carve-out,
  not by this decision; [#128](https://github.com/MBehtemam/Montagent/issues/128)
  should not assume `assetRoot` exists as a mechanism it can reuse.

## Evidence

Two rounds of `/court`, four jurors across Opus, Sonnet, Haiku and Fable,
independent, blind to each other's ballots. Full ballots at
[`docs/research/juries/asset-path-resolution/BALLOTS.md`](../research/juries/asset-path-resolution/BALLOTS.md).
