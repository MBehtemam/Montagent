# Court ballots: asset path resolution (#125)

Four questions on how `source` paths resolve, whether a top-level `assetRoot` is
adopted, whether absolute local paths are permitted, and what `validate`/`render`
do when a `source` file is missing. Two rounds of `/court`, independent, blind to
each other's ballots and to the author's recommendation.

## Round 1 — four questions, three jurors (Opus, Haiku, Fable)

**Q1 — What do relative `source` paths resolve against by default?**
(a) the project file's own directory (b) the process's CWD at invocation
(c) no default, always declared

**Q2 — Is a top-level `assetRoot` field adopted, and what does it do?**
(a) no `assetRoot` (b) optional `assetRoot` overriding the default base
(c) `assetRoot` mandatory

**Q3 — Are absolute local filesystem paths permitted in `source`?**
(a) forbidden (b) permitted, resolved as-is, no portability guarantee

**Q4 — What does `validate`/`render` do when a `source` file is missing?**
(a) a plain `error`, same class as any other structural defect
(b) a distinct, softer finding class for "not present yet"

| Juror | Q1 | Q2 | Q3 | Q4 |
|---|---|---|---|---|
| Opus | a | b | b | a |
| Haiku | a | b | a | a |
| Fable | a | a | b | a |

Q1 and Q4: **unanimous 3/3** for (a) on both. Q2: **2-1** for (b) (Opus, Haiku).
Q3: **2-1** for (b) (Opus, Fable).

### Reasoning (Q1, Q4 — unanimous, not relitigated in round 2)

All three jurors converged independently on the project-file-directory as the
only base a reader can resolve from the document plus its on-disk neighbours
alone, without depending on invocation state — the CWD-relative alternative
was named by every juror as a direct violation of file-as-truth. All three
also converged on treating a missing `source` as a plain ADR-0006 `error`:
the check is already promised by ADR-0002 ("`validate` must check that every
`source` resolves"), and inventing a softer severity for "not downloaded yet"
was named independently by two jurors as exactly the unforced-mechanism
pattern this project's history keeps rejecting.

### Reasoning (Q2 — split, see round 2 for the tie-break)

**Majority (Opus, Haiku) for adopting `assetRoot`:** the ticket's stated goal —
move a project between a local checkout and a remote store "without touching
any element" — cannot be met by moving a sibling folder alone once the target
is a remote store, and an optional prefix field is inert data (readable
without evaluation), so it does not reopen ADR-0002's rejection of an asset
*table*, only adds a single declared base.

**Dissent (Fable) against adopting `assetRoot`:** `source` may already be a
full URL under ADR-0002, so the stated goal is already reachable today without
a new field — a mechanical rewrite of every `source`, which this project's
own history treats as a trivial, reliably-correct agent operation (write-tool
invariant, exact-string-replace precedent). Structurally, `assetRoot` is "a
mini asset table by another name": a reader can no longer tell what `source`
resolves to by reading the element alone, which is the exact property
ADR-0002 protected — the field's *size* differs from a table, its *shape*
does not.

## Round 2 — Q2 re-run with a fourth juror (Sonnet), briefed with both round-1 positions

Sonnet was shown the round-1 majority and dissent arguments verbatim and asked
to vote independently.

| Juror | Q2 |
|---|---|
| Opus | b |
| Haiku | b |
| Fable | a |
| Sonnet | a |

**Result: 2-2, no majority.** Sonnet's ballot reinforced Fable's structural
argument rather than the majority's goal-satisfaction argument: "`assetRoot`
is functionally the asset-table indirection ADR-0002 closed the door on, just
relocated to a single field instead of a table of entries — the size of the
indirection shrinks but its structural cost (non-self-describing sources, a
second place you must read to resolve one field) does not," and noted the
majority's premise — that the goal is otherwise unreachable — is false, since
`source`-as-URL already covers it.

## Resolution

The author broke the tie for **(a) — no `assetRoot`**, on the grounds that the
majority's forcing claim doesn't survive scrutiny (the goal is already
achievable via `source`-as-URL plus a mechanical rewrite) and that two
independent jurors, briefed on the majority's own reasoning, still converged
on the same ADR-0002-consistency objection rather than merely preferring a
different ergonomics trade-off.
