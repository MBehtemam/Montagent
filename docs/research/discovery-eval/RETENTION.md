# Evidence Retention, Rerun Triggers, and CI Automation Specification

This specification defines the storage hierarchy, artifact retention lifecycle, targeted rerun triggers, routine CI smoke gating, and register synchronization for the discovery and selection evaluation under [Map: can a fresh external agent discover, choose, and use Montagent features?](https://github.com/MBehtemam/Montagent/issues/957) and resolved in [Decide what evidence to retain and when to rerun tests](https://github.com/MBehtemam/Montagent/issues/964).

Evaluation is benchmarked against pinned Montagent release **`v0.2.1`** across the 4 evaluation arms:
1. `cli-bare`
2. `cli-skills`
3. `mcp-bare`
4. `mcp-skills`

---

## 1. Storage Hierarchy and Git Commit Policy

To keep the repository lightweight while preserving a complete, auditable record of discovery evaluation outcomes and failure forensics, artifacts are split across committed and local ephemeral storage:

```
docs/research/discovery-eval/
├── RETENTION.md            # This specification
├── PILOT.md                # Pilot budget, caps, and stopping rules
├── SCORING.md              # 5-dimension scorecard and failure taxonomy
├── REGISTER.md             # Human-readable coverage register
├── register.json           # Canonical machine-readable manifest
├── register.schema.json    # JSON schema for register manifest
├── harness/
│   ├── pins.json           # Model pin, caps, timeout, tool settings
│   ├── run_arm.py          # Dedicated 4-arm discovery evaluation runner
│   └── score_trace.py      # Automated trace parser, scorer, and register sync
└── runs/
    ├── <version>/<phase>/<task-id>/<arm>-<trial>/
    │   ├── manifest.json         # [Committed] Execution pins, timestamps, token/spend totals, exit code
    │   ├── scorecard.json        # [Committed] 5-dimension ratings, composite verdict, attribution
    │   └── transcript.jsonl.gz   # [Committed for Failures Only] Compressed trace of failed/aborted runs
    └── .local/                   # [Git-Ignored] Local ephemeral cache
        └── <version>/<phase>/<task-id>/<arm>-<trial>/
            ├── transcript.jsonl  # Raw uncompressed stream-json output for passing runs
            ├── stderr.log        # Raw process stderr output
            └── workspace/        # Agent-generated files and scratch edits
```

### Artifact Retention Matrix

| Artifact | Passing Runs | Failing / Flagged Runs | Location | Committed to Git? | Retention Policy |
| :--- | :--- | :--- | :--- | :---: | :--- |
| **`scorecard.json`** | Yes | Yes | `runs/<version>/...` | **Yes** | Permanent (historical record) |
| **`manifest.json`** | Yes | Yes | `runs/<version>/...` | **Yes** | Permanent (historical record) |
| **`transcript.jsonl.gz`** | No | **Yes** | `runs/<version>/...` | **Yes** | Permanent (forensic audit for failures) |
| **`transcript.jsonl` (raw)** | Yes | Yes | `runs/.local/...` | No | 14 days (pruned via CLI) |
| **`stderr.log`** | Yes | Yes | `runs/.local/...` | No | 14 days (pruned via CLI) |
| **`workspace/` scratch** | Yes | Yes | `runs/.local/...` | No | 14 days (pruned via CLI) |
| **Rendered Deliverables (`.mp4`)** | Verified inline | Verified inline | Ephemeral / Scratch | No | Purged immediately after scoring |

---

## 2. Rendered Media and Heavy Deliverables Policy

1. **Inline Programmatic Verification**:
   - The discovery evaluation evaluates whether an external agent can discover, choose, and invoke Montagent features correctly.
   - Any rendered MP4 deliverables or frame stills (`.png`) produced during a task are evaluated programmatically during scoring using native verification tools (`montagent verify`, `montagent measure`, or `probe`).
   - The verification findings and exit codes are recorded deterministically in `scorecard.json`.
2. **Exclusion from Repository**:
   - Binary media deliverables (`.mp4`, `.png`) are strictly excluded from git commits to prevent repository bloat.
   - Deliverables in temporary scratch directories are purged immediately upon scorecard generation or retained under `runs/.local/` for a maximum of 14 days for manual review.

---

## 3. Version-Scoped Run Directory Archival

To guarantee that past baseline evaluations remain accessible as immutable benchmarks across product evolutions:
1. Every run directory is prefixed by the target semantic version tag:
   ```
   runs/<version>/<phase>/<task-id>/<arm>-<trial>/
   ```
   - Example: `runs/v0.2.1/baseline/discovery-01-assembly-inspect/cli-bare-1/`
2. When Montagent moves to a new minor release (e.g. `v0.3.0`):
   - Prior version run directories (`runs/v0.2.1/`) remain permanently frozen in git.
   - The active coverage register (`register.json` and `REGISTER.md`) tracks the latest target version.
   - Historical discovery comparisons between releases are evaluated directly against committed past scorecards.

---

## 4. Rerun Triggers and Invalidation Workflows

Evaluation runs occur across three distinct operating tiers to balance cost, velocity, and statistical rigor:

### Tier 1: Targeted Feature Invalidation & Reruns (On-Demand & PRs)
- **Trigger**: When a code change or PR modifies the CLI flags, argument parsing, JSON schema, docstrings, `--help` text, or skill documentation associated with a registered capability.
- **Workflow**:
  1. A pre-commit or CI check identifies touched capabilities and marks them `stale` in `register.json` across affected arms.
  2. The developer or reviewer executes targeted reruns using the discovery harness:
     ```sh
     # Rerun only tasks targeting currently stale capabilities:
     uv run docs/research/discovery-eval/harness/run_arm.py --stale
     # Or rerun a specific task:
     uv run docs/research/discovery-eval/harness/run_arm.py --task discovery-04-timeline-shift
     ```
  3. Once 3 independent trials achieve unanimous `Full Success`, the capability transitions from `stale` back to `passed` in `register.json`.

### Tier 2: Path-Filtered CI Smoke Suite (Routine Automation)
- **Trigger**: Automatically triggered on GitHub pull requests to `main` when diffs modify discovery surfaces:
  - `crates/montagent-core/src/cli.rs`
  - `crates/montagent-core/src/commands/`
  - `crates/montagent-mcp/`
  - `skills/`
  - `schema/`
  - `docs/specs/`
- **Scope**: 4-run smoke subset ($N=1$):
  - Task 1 (`discovery-01-assembly-inspect`) and Task 5 (`discovery-05-scene-transition`)
  - Evaluated across `cli-bare` and `mcp-bare`
  - Total ceiling budget: ~$1.00 (expected actual: ~$0.30).
- **Security & Fork Gating**:
  - Executes exclusively on branches within the upstream repository or via manual `workflow_dispatch`.
  - External fork pull requests skip execution gracefully with an informational notice, safeguarding repository secrets (`ANTHROPIC_API_KEY`).

### Tier 3: Release Milestone Authoritative Baseline (Milestone Gating)
- **Trigger**: Prior to cutting a new semantic release (e.g. `v0.3.0`) or upon authoring major new capabilities.
- **Scope**: Full 72-run baseline ($6 \text{ tasks} \times 4 \text{ arms} \times 3 \text{ trials}$).
- **Output**: Authoritative state transitions in `register.json`, updated `REGISTER.md`, and official committed Gap Index ($G$) rankings.

---

## 5. Maintenance and Local Artifact Pruning

To prevent local disk exhaustion from uncommitted raw transcripts and ephemeral workspaces, the harness provides a dedicated pruning command:

```sh
# Clean uncommitted local run caches older than 14 days (default):
uv run docs/research/discovery-eval/harness/run_arm.py --clean-local

# Clean caches older than N days:
uv run docs/research/discovery-eval/harness/run_arm.py --clean-local --days 7

# Clean all local ephemeral workspaces and raw traces immediately:
uv run docs/research/discovery-eval/harness/run_arm.py --clean-local --all
```

---

## 6. Coverage Register Synchronization and CLI Reporting

The discovery harness integrates automated registry inspection and reporting tools:

```sh
# Display ANSI summary of capability states and active Gap Index:
uv run docs/research/discovery-eval/harness/run_arm.py --report

# Filter reporting to display only untested or stale capabilities:
uv run docs/research/discovery-eval/harness/run_arm.py --report --filter stale,untested

# Re-synchronize REGISTER.md and recalculate Gap Index from committed scorecards:
uv run docs/research/discovery-eval/harness/score_trace.py --sync-register
```

### Gap Index Reporting Rules
- Reports compute the Gap Index ($G = 1.0 - \text{Full Success Rate}_{\text{bare}}$) using committed scorecards in the active version directory.
- Untested or stale capabilities display with pending status flags rather than penalizing the active release score.
