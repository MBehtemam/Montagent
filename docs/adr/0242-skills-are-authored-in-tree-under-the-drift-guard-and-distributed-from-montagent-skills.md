# 242. Skills are authored in-tree under the drift guard and distributed from `MBehtemam/montagent-skills`

Date: 2026-10-18

## Status

Accepted

## Context

`skills/` holds Montagent's six end-user Agent Skills (`montagent`, `montagent-motion`, `montagent-footage`, `montagent-character`, `montagent-prerender`, and `montagent-craft`), totaling ~260 KB of Markdown and Python helper scripts.

Installing them with `npx skills add MBehtemam/Montagent` exhibited two problems:

1. **Whole-repository clone (~921 MB unpacked / ~350–400 MB transfer):** `vercel-labs/skills` only uses GitHub's single-folder blob/API install path for four hardcoded repository owners (`BLOB_ALLOWED_OWNERS` in `src/commands/add.ts`: `vercel`, `vercel-labs`, `heygen-com`, `remotion-dev`). For every other GitHub repository — even when a subpath or `--skill` filter is passed — `cloneRepo()` runs `git clone --depth 1` without `--filter=blob:none` or `git sparse-checkout`. Because `MBehtemam/Montagent` tracks ~851 MB of evaluation transcripts and rendered media under `docs/research/` and ~60 MB of test fixtures under `fixtures/`, installing 260 KB of skills downloaded the entire `HEAD` tree into a temporary directory before copying `skills/` and deleting the checkout.
2. **Internal engineering skill leakage:** `npx skills` scans both `skills/` and `.agents/skills/` during discovery unless a skill in `.agents/skills/` is listed in `skills-lock.json` or declares `metadata: { internal: true }` in its YAML frontmatter. Any unguarded repo-engineering skill under `.agents/skills/` is offered alongside end-user Montagent skills, violating `CONTEXT.md`'s boundary that a **Skill** is end-user guidance and never guidance to agents developing Montagent itself.

At the same time, deleting `skills/` from `MBehtemam/Montagent` and moving authoring to a separate repository would sever the compile-time and CI drift guards that keep the skills accurate:
- `crates/montagent/tests/skills.rs` (#450, #513) validates every code-span name, finding code, `montagent://` URI, CLI verb/flag, fenced JSON project snippet, `skills/*/scripts/*.py` invocation, capability-map entry, and refused-feature ADR citation against the binary compiled from the same commit.
- `crates/montagent-core/tests/morph.rs` and `crates/montagent-core/tests/trim.rs` parse and render recipes directly out of `skills/montagent-motion/SKILL.md`.
- `ci/skills_workarounds.py` and `.github/workflows/skills.yml` check `<!-- workaround: #NNN -->` markers against `MBehtemam/Montagent`'s issue tracker.

## Decision

1. **In-tree authoring remains authoritative (`MBehtemam/Montagent/skills/`):**
   - `skills/` remains committed in `MBehtemam/Montagent` so format, verb, schema, ADR, and skill edits land atomically in one pull request and are gated by `cargo test --test skills`, `morph.rs`, `trim.rs`, and `ci/skills_workarounds.py`.

2. **Dedicated distribution mirror (`MBehtemam/montagent-skills`):**
   - End users and agents install Montagent skills from the standalone public repository [`MBehtemam/montagent-skills`](https://github.com/MBehtemam/montagent-skills):
     ```sh
     npx skills add MBehtemam/montagent-skills
     ```
   - `.github/workflows/skills.yml` (`publish` job) mirrors git-tracked `skills/**`, `skills/README.md` (as the root `README.md`), and `LICENSE` to `MBehtemam/montagent-skills` on every push to `main` touching `skills/**` or `LICENSE` (and on `workflow_dispatch` from `main`), authenticated via a single-repository SSH deploy key (`SKILLS_DEPLOY_KEY`).

3. **Internal repo-engineering skills carry `metadata.internal: true`:**
   - Every repo-engineering skill under `.agents/skills/*/SKILL.md` declares `metadata:\n  internal: true` in its YAML frontmatter so `npx skills` skips it during discovery unless `INSTALL_INTERNAL_SKILLS=1` is explicitly set.

## Consequences

### Positive
- `npx skills add MBehtemam/montagent-skills` transfers ~135 KiB instead of ~400 MB and finishes in ~1 second.
- `cargo test --test skills` and all in-tree recipe and workaround guards continue to run atomically on every PR in `MBehtemam/Montagent` without cross-repo CI checkouts or version skew.
- Internal `.agents/skills/` workflows are never offered when `npx skills` inspects `MBehtemam/Montagent` or a local checkout.

### Negative / Trade-offs
- Maintaining the mirror requires one repository-scoped deploy key (`SKILLS_DEPLOY_KEY`) between `MBehtemam/Montagent` and `MBehtemam/montagent-skills`.
- Direct pull requests against `MBehtemam/montagent-skills` would be overwritten by the next sync from `MBehtemam/Montagent`; `skills/README.md` (mirrored to the root `README.md`) states that `MBehtemam/Montagent/skills` is the authoring source.
