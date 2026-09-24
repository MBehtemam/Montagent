# Domain Docs

How the engineering skills should consume this repo's domain documentation when exploring the codebase.

## Before exploring, read these

- **`CONTEXT.md`** at the repo root, or
- **`CONTEXT-MAP.md`** at the repo root if it exists — it points at one `CONTEXT.md` per context. Read each one relevant to the topic.
- **`docs/adr/`** — read ADRs that touch the area you're about to work in. In multi-context repos, also check `src/<context>/docs/adr/` for context-scoped decisions.

If any of these files don't exist, **proceed silently**. Don't flag their absence; don't suggest creating them upfront. The `/domain-modeling` skill (reached via `/grill-with-docs` and `/improve-codebase-architecture`) creates them lazily when terms or decisions actually get resolved.

## File structure

**This repo is single-context**: one `CONTEXT.md` and one `docs/adr/` at the root.

Single-context repo (most repos):

```
/
├── CONTEXT.md
├── docs/adr/
│   ├── 0001-event-sourced-orders.md
│   └── 0002-postgres-for-write-model.md
└── src/
```

Multi-context repo (presence of `CONTEXT-MAP.md` at the root):

```
/
├── CONTEXT-MAP.md
├── docs/adr/                          ← system-wide decisions
└── src/
    ├── ordering/
    │   ├── CONTEXT.md
    │   └── docs/adr/                  ← context-specific decisions
    └── billing/
        ├── CONTEXT.md
        └── docs/adr/
```

## Use the glossary's vocabulary

When your output names a domain concept (in an issue title, a refactor proposal, a hypothesis, a test name), use the term as defined in `CONTEXT.md`. Don't drift to synonyms the glossary explicitly avoids.

If the concept you need isn't in the glossary yet, that's a signal — either you're inventing language the project doesn't use (reconsider) or there's a real gap (note it for `/domain-modeling`).

## Flag ADR conflicts

If your output contradicts an existing ADR, surface it explicitly rather than silently overriding:

> _Contradicts ADR-0007 (event-sourced orders) — but worth reopening because…_

## ADRs land on `main` as they are accepted

**An accepted ADR merges to `main` as soon as its wayfinder ticket closes** — one
PR per ticket, from its `domain/<slug>` branch. Do not accumulate ADRs on
unmerged branches until the spec is assembled.

The reason is agent-readability, and the cost of getting it wrong is measured:
with seven ADRs sitting on branches, every subagent had to be handed explicit
`git show domain/<branch>:docs/adr/<n>-<slug>.md` incantations, and any agent
that missed one read `main` and reasoned from a domain model several decisions
stale. `main` is the single place an agent should have to look.

**ADRs are amended, never rewritten.** A later ADR that corrects an earlier one
says so in its own text, and the earlier one gets a short pointer under its title
naming the amendment and what in it no longer holds. A reader landing on an ADR
must be able to see it has been superseded without having read the one that
superseded it — `docs/adr/0006-validate-reports-facts-and-render-enforces.md` is
the worked example.

**Commit the evidence an ADR rests on — checked before `status: accepted` is
written, not after.** This binds an ADR only when its prose actually cites a
prototype, a fixture, a script, or a jury artifact as support for a claim — an
ADR arguing purely from already-committed spec text has nothing external to
commit, and there is nothing to check.

The form the evidence takes depends on the claim:

- **A numeric or measured claim** (a threshold, a percentage, a count, a byte
  size) needs a *re-executable* check — a script that re-derives the number and
  exits non-zero the moment it stops reproducing — committed alongside the
  input data it runs against. `fit_rounding_scan.py`
  (`docs/adr/0013-fitted-extents-floor-and-the-nine-origin-keywords.md`) is the
  worked example: it asserts both directions of its own decision and fails
  loudly if either stops holding. A screenshot or a transcript of a number is
  not enough — a later reader has no way to tell whether it still holds.
- **A qualitative claim** (a jury verdict, a design finding, a prototype's
  observed behavior) needs the artifact itself committed verbatim — the
  write-up, the exercise transcript, the prototype's output — since there is no
  formula to re-run. `FINDINGS.md` and `JURY-EDIT-EXERCISE.md` under
  `docs/research/sample-project/` are the worked example.

Before writing `status: accepted`, the merging session confirms every artifact
the ADR's prose cites resolves to a path that exists on the branch being
merged, and that any committed numeric check actually runs clean. This is a
checklist step, not a mechanical gate — this repo has no CI, and a script that
only greps for a linked path would prove the path exists without proving it
grounds the claim or that it was tiered correctly.

The motivating failure, left otherwise unfixed until now: ADR-0006's *"every
numeric claim... was verified by script against the file"* cited a project
file that was never committed —
`git log --all --diff-filter=A -- '*.montagent.json'` returns exactly one
project file, and it is not that one. The claim is now flagged inline in
ADR-0006 as unreproducible rather than left silently uncheckable.
