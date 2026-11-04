# Coding Standards

Consulted during `/code-review` (Standards axis). Mechanical rules are enforced by `ci/pre_commit.sh`; this document holds cross-module invariants and judgement calls that static linters cannot check.

## 1. Domain & ADR Discipline

- **Glossary vocabulary (`CONTEXT.md`)**: Every domain term, format key, finding code, and report field must match `CONTEXT.md` and be backed by an accepted ADR in `docs/adr/` (`docs/agents/domain.md`). Do not invent synonyms in code, comments, or tests for terms `CONTEXT.md` explicitly avoids.
- **ADR fidelity**: Code implementing an ADR must match the ADR's exact data types, key names, and accounting rules (e.g., `render.profile` attribution, cache slot keys). If implementation reveals a cleaner shape, amend or update the ADR in the same commit so the ADR and code never diverge.
- **ADR amendment integrity**: Every ADR declaring `amends:` must add an `"Amended by"` banner to each target ADR and update both the row and opening counts in `docs/adr/README.md` (`python3 docs/adr/check_amendment_banners.py`).
- **Schema generation**: `schema/montagent.schema.json` is generated from `crates/montagent-core/src/model/` (`cargo run -p montagent-core --example schema > schema/montagent.schema.json`). Never hand-edit `schema/montagent.schema.json`.

## 2. Findings, Repair Classes, and Exit Codes

- **Document vs. Invocation boundary ([ADR-0043](docs/adr/0043-refuse-class-findings-a-repair-field-uniform-per-check-non-bypassable.md), [ADR-0073](docs/adr/0073-process-level-errors-carry-no-repair-field.md), [ADR-0083](docs/adr/0083-mcp-iserror-tracks-notaboutdocument.md))**:
  - `Repair::Refuse` (`"repair": "none"`, exit `1`) and `Repair::Advise` are strictly for findings **about a project document**.
  - Bad CLI flags, invalid MCP tool arguments, or non-document refusals (`E-INVOCATION`, `E-PARSE`, `E-READ`, `E-PROJECT-EXISTS`, `E-SHEET-OVERFLOW`) must use `RepairClass::NotAboutDocument` (no `repair` field, exit `2`/`3`/`70`, `isError: true` over MCP) and must never attach a document `.at_file(...)` location when the fault is the caller's flag.
- **Verb flag scoping in hints**: Prose hints in `crates/montagent-core/src/text.rs` (such as `--verbose` or `--quiet <CODE>`) must only print on verbs whose CLI/MCP surface actually accepts that flag.
- **Check-set honesty ([ADR-0112](docs/adr/0112-a-report-names-the-check-sets-that-ran-and-prints-no-zero-it-did-not-earn.md))**: A report records a `CheckSet` only when that check set ran to completion, and prints `0` only for classes a completed set can raise.

## 3. Caches, Sidecars, and Determinism

- **Movable project unit ([ADR-0053](docs/adr/0053-asset-path-resolution-no-assetroot.md), [ADR-0069](docs/adr/0069-probe-sidecar-is-a-per-user-json-cache-keyed-on-what-montagent-observed.md))**: Never write caches or sidecars beside the project file. All persistent caches live under the per-user cache directory (`MONTAGENT_CACHE_DIR`), write atomically via PID-suffixed temp files, remove any leftover `.tmp` file if `rename` fails, degrade silently to a cache miss on any I/O error, and are cleared by `montagent cache clear`.
- **Observed file identity ([ADR-0092](docs/adr/0092-a-probe-is-matched-on-an-observed-identity-and-guarded-by-its-contents.md), [ADR-0245](docs/adr/0245-nested-scenes-cache-their-offscreen-child-composites-by-relative-subgraph-hash.md))**: Any persistent cache keyed on local assets or fonts must guard `(canonical_path, size, mtime_ns)` with a content fingerprint (`sidecar::fingerprint`) and scope its hash strictly to the assets/fonts referenced by that unit.
- **Exact arithmetic & byte identity ([ADR-0013](docs/adr/0013-fitted-extents-floor-and-the-nine-origin-keywords.md), [ADR-0028](docs/adr/0028-text-block-arithmetic-is-exact-tenths.md), [ADR-0045](docs/adr/0045-speed-invariant-is-evaluated-in-exact-arithmetic.md), [ADR-0195](docs/adr/0195-time-invariant-bottom-layers-share-a-byte-identical-prefix-composite.md))**: Use exact integer/rational arithmetic where specified rather than IEEE `f64`. Any render optimization or cache (`prefix`, `nest_cache`, parallel `painters`) must produce byte-identical frames and identical report provenance (`painted`, `not_painted`, `painted_partially`, `declined`, `sources`, `fonts`) to an uncached single-painter run.

## 4. Architecture & Adapter Seams

- **Thin adapters ([ADR-0011](docs/adr/0011-tool-surface-reads-checks-renders.md))**: `crates/montagent/src/cli.rs` and `crates/montagent/src/mcp.rs` wrap `montagent_core` directly (MCP never shells out to CLI). Domain logic, validation, and report rendering (`wire.rs`, `text.rs`) belong in `montagent-core`.
- **Skills mirror alignment**: User-facing CLI flags, finding codes, and recipes documented in `skills/` are verified by `cargo test -p montagent --test skills` and `montagent skills check`.
