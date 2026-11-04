# Montagent

## Agent skills

- **Issue tracker**: GitHub Issues (`MBehtemam/Montagent`) via `gh`. See `docs/agents/issue-tracker.md`.
- **Triage labels**: Five canonical triage roles (`needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`). See `docs/agents/triage-labels.md`.
- **Domain docs**: Single-context (`CONTEXT.md` + `docs/adr/` + `docs/adr/README.md`). Read `docs/agents/domain.md` before exploring or editing ADRs/schema.
- **Coding standards & review**: Read `CODING_STANDARDS.md` during `/code-review` or when touching CLI/MCP adapters, findings, caches, or schema.

## Workspace layout & fast checks

- **Verb plumbing**: `crates/montagent/src/cli.rs` (CLI) and `crates/montagent/src/mcp.rs` (MCP) wrap `crates/montagent-core/src/verbs/<verb>`, rendered via `crates/montagent-core/src/wire.rs` (JSON) and `crates/montagent-core/src/text.rs` (prose). Rasterization/FFmpeg live in `crates/montagent-render/`, text shaping in `crates/montagent-text/`.
- **Pre-commit / PR gate**: Run `ci/pre_commit.sh` (`cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `python3 docs/adr/check_amendment_banners.py`, `python3 docs/adr/predicate_reserved_names_scan.py`, `python3 ci/skills_workarounds.py --lint`) before committing or merging.
