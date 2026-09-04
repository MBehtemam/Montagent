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

**Commit the evidence an ADR rests on.** ADR-0006's headline measurement was
taken against a project file that was never committed, so no later reader can
check it. If a decision rests on a prototype, a fixture or a jury artifact, that
artifact belongs in the repo before the ADR merges.
