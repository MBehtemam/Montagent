# Versioned Feature Coverage Register

This register tracks the discovery and selection evaluation coverage of Montagent capabilities across CLI and MCP interfaces, benchmarked against pinned release **`v0.2.1`** under [Map: can a fresh external agent discover, choose, and use Montagent features?](https://github.com/MBehtemam/Montagent/issues/957) and resolved in [Design the versioned feature coverage register](https://github.com/MBehtemam/Montagent/issues/962).

---

## 1. Governance & Coverage Principles

### Six-State Lifecycle Taxonomy

Every capability tracks one of six explicit states independently across each evaluation arm:

1. **`untested`**: Registered capability with zero evaluation runs under the current baseline version.
2. **`passed`**: Directly targeted in an evaluation task where all independent trials achieved `Full Success` (or `Acceptable Alternative` without missed capability).
3. **`failed`**: Directly targeted in an evaluation task where trials resulted in `Failure` (unsupported external tool workarounds, mismatched tools, persistent hallucinations, or execution exhaustion) or flagged `Missed Built-In Capability`.
4. **`stale`**: Previously passing or failing evidence invalidated by subsequent modifications to tool signatures, argument constraints, JSON schemas, discovery documentation, or release milestone bumps.
5. **`unavailable`**: Intentionally omitted from this interface due to documented architectural asymmetry (e.g. `probe`, `fmt`, or `cache clear` on MCP; in-memory schema resources on CLI). Strictly excluded from the interface's scoring denominator.
6. **`intentionally_excluded`**: Explicitly excluded from discovery evaluation scope (e.g. human-only terminal visualization `timeline`, stdio daemon launcher `montagent mcp`).

### Direct Target vs. Incidental Coverage

- **Primary Target Capabilities**: The specific, purpose-built features a task is designed to evaluate (e.g. `shift` in Task 4). A task's evaluation run can **only** update coverage status (`passed`/`failed`) for its designated primary targets.
- **Incidental Auxiliary Usage**: Tools or commands invoked purely in support of a primary goal (e.g. running `validate` or `create-project` while completing a `shift` task). Incidental invocations are captured in trace logs for auditability, but **never** satisfy or advance coverage in the register. Completing a task alone does not imply related features were tested.

### Four-Arm Evaluation Matrix

Coverage is tracked independently across the 4 clean-room evaluation arms:
- **`cli-bare`**: Clean-room workspace, `montagent` binary in `PATH`, zero skills installed, no MCP server.
- **`cli-skills`**: Clean-room workspace, `montagent` binary in `PATH`, complete Montagent skill suite installed, no MCP server.
- **`mcp-bare`**: Clean-room workspace, stdio MCP connection to Montagent server, file tools, zero skills installed, binary omitted from `PATH`.
- **`mcp-skills`**: Clean-room workspace, stdio MCP connection to Montagent server, file tools, complete Montagent skill suite installed, binary omitted from `PATH`.

### Invalidation Triggers

1. **Granular Capability Invalidation**: Modifying a command/tool's CLI flags, schema properties, validation logic, or primary discovery surfaces (`--help` docstrings, `tools/list` descriptions, resource markdown, or relevant skill files) transitions that capability's status to `stale` across affected arms.
2. **Milestone Baseline Invalidation**: Bumping the pinned Montagent semantic release version (e.g. `v0.2.1` $\to$ `v0.3.0`) transitions all capabilities across the register to `stale`, requiring a full baseline re-run.

---

## 2. Coverage Summary (Baseline `v0.2.1`)

| Evaluation Arm | Total Capabilities | Untested | Passed | Failed | Stale | Unavailable (N/A) | Excluded | Active Denominator |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| **`cli-bare`** | 31 | 20 | 0 | 0 | 0 | 9 | 2 | **20** |
| **`cli-skills`** | 31 | 20 | 0 | 0 | 0 | 9 | 2 | **20** |
| **`mcp-bare`** | 31 | 26 | 0 | 0 | 0 | 3 | 2 | **26** |
| **`mcp-skills`** | 31 | 26 | 0 | 0 | 0 | 3 | 2 | **26** |

*Note: Active Denominator = Total Capabilities $-$ Unavailable $-$ Excluded.*

---

## 3. Feature Coverage Register

| Capability ID | Category | Name & Description | CLI Surface | MCP Surface | `cli-bare` | `cli-skills` | `mcp-bare` | `mcp-skills` | Primary Target Tasks |
| :--- | :--- | :--- | :--- | :--- | :---: | :---: | :---: | :---: | :--- |
| `cap:project/create-project` | Project | Scaffolds initial project file | `montagent create-project` | `create_project` | `untested` | `untested` | `untested` | `untested` | `discovery-01-assembly-inspect` |
| `cap:inspection/validate` | Inspection | Validates schema & constraints | `montagent validate` | `validate` | `untested` | `untested` | `untested` | `untested` | `discovery-01`, `02`, `04`, `05`, `06` |
| `cap:inspection/fmt` | Inspection | Canonicalizes JSON key order/whitespace | `montagent fmt` | *None (CLI-only)* | `untested` | `untested` | `unavailable` | `unavailable` | *(Backlog)* |
| `cap:inspection/query` | Inspection | Queries timeline facts & census | `montagent query` | `query` | `untested` | `untested` | `untested` | `untested` | *(Backlog)* |
| `cap:inspection/frame` | Inspection | Rasterizes frame instant or sheet | `montagent frame` | `frame` | `untested` | `untested` | `untested` | `untested` | `discovery-01-assembly-inspect` |
| `cap:inspection/measure` | Inspection | Measures spatial, text, & alpha metrics | `montagent measure` | `measure` | `untested` | `untested` | `untested` | `untested` | *(Backlog / Incidental: 03)* |
| `cap:timeline/shift` | Timeline | Ripple edits timeline events with release | `montagent shift` | `shift` | `untested` | `untested` | `untested` | `untested` | `discovery-04-timeline-shift` |
| `cap:timeline/compare` | Timeline | Reports project drift & slack delta | `montagent compare` | `compare` | `untested` | `untested` | `untested` | `untested` | *(Backlog)* |
| `cap:rendering/preview` | Rendering | Fast proxy MP4 render | `montagent preview` | `preview` | `untested` | `untested` | `untested` | `untested` | *(Backlog / Incidental: 03)* |
| `cap:rendering/render` | Rendering | Final deliverable MP4 render | `montagent render` | `render` | `untested` | `untested` | `untested` | `untested` | `discovery-03`, `05`, `06` |
| `cap:rendering/verify` | Rendering | Decodes & checks output against project | `montagent verify` | `verify` | `untested` | `untested` | `untested` | `untested` | `discovery-01`, `05` |
| `cap:inspection/probe` | Inspection | Probes media streams, codecs, fps | `montagent probe` | *None (CLI-only)* | `untested` | `untested` | `unavailable` | `unavailable` | *(Backlog)* |
| `cap:maintenance/cache-clear` | Maintenance | Clears media probe disk cache | `montagent cache clear` | *None (CLI-only)* | `untested` | `untested` | `unavailable` | `unavailable` | *(Backlog)* |
| `cap:timeline/timeline` | Timeline | Terminal ASCII timeline view | `montagent timeline` | *None (CLI-only)* | `intentionally_excluded` | `intentionally_excluded` | `intentionally_excluded` | `intentionally_excluded` | *Operator-only debug* |
| `cap:assets/fonts-acquire` | Assets | Acquires font & license receipt | `montagent fonts acquire` | `fonts_acquire` | `untested` | `untested` | `untested` | `untested` | `discovery-02-font-licensing` |
| `cap:assets/fonts-list` | Assets | Lists host fonts & classifications | `montagent fonts list` | `fonts_list` | `untested` | `untested` | `untested` | `untested` | `discovery-02-font-licensing` |
| `cap:assets/fonts-vendor` | Assets | Vendors font behind license gate | `montagent fonts vendor` | `fonts_vendor` | `untested` | `untested` | `untested` | `untested` | `discovery-02-font-licensing` |
| `cap:maintenance/mcp-launcher` | Maintenance | Launches stdio MCP daemon | `montagent mcp` | *None (Launcher)* | `intentionally_excluded` | `intentionally_excluded` | `intentionally_excluded` | `intentionally_excluded` | *Harness daemon* |
| `cap:resources/schema-json` | Resources | Root project JSON schema resource | *None (MCP-only)* | `montagent://schema.json` | `unavailable` | `unavailable` | `untested` | `untested` | *(Backlog)* |
| `cap:resources/format-md` | Resources | Format specification & index | *None (MCP-only)* | `montagent://format.md` | `unavailable` | `unavailable` | `untested` | `untested` | *(Backlog)* |
| `cap:resources/schema-index` | Resources | Modular schema piece index | *None (MCP-only)* | `montagent://schema/index.json` | `unavailable` | `unavailable` | `untested` | `untested` | *(Backlog)* |
| `cap:resources/format-text` | Resources | Typography & text styling format | *None (MCP-only)* | `montagent://format/text.md` | `unavailable` | `unavailable` | `untested` | `untested` | *(Backlog)* |
| `cap:resources/format-compositing` | Resources | Compositing, layers, & blend modes | *None (MCP-only)* | `montagent://format/compositing.md` | `unavailable` | `unavailable` | `untested` | `untested` | *(Backlog)* |
| `cap:resources/format-transitions` | Resources | Transition elements & audio cuts | *None (MCP-only)* | `montagent://format/transitions.md` | `unavailable` | `unavailable` | `untested` | `untested` | *(Backlog)* |
| `cap:resources/format-audio-effects` | Resources | Audio effects & filtering format | *None (MCP-only)* | `montagent://format/audio-effects.md` | `unavailable` | `unavailable` | `untested` | `untested` | *(Backlog)* |
| `cap:resources/format-paths` | Resources | Vector paths & shapes format | *None (MCP-only)* | `montagent://format/paths.md` | `unavailable` | `unavailable` | `untested` | `untested` | *(Backlog)* |
| `cap:resources/format-audio-dynamics` | Resources | Sidechaining & audio dynamics format | *None (MCP-only)* | `montagent://format/audio-dynamics.md` | `unavailable` | `unavailable` | `untested` | `untested` | *(Backlog)* |
| `cap:effects/chroma-matte` | Effects | Chroma key color matte effect | `declarative:effect/chroma_matte` | `declarative:effect/chroma_matte` | `untested` | `untested` | `untested` | `untested` | `discovery-03-chroma-key` |
| `cap:primitives/transition-crossfade` | Primitives | Visual crossfade transition element | `declarative:element/transition:crossfade` | `declarative:element/transition:crossfade` | `untested` | `untested` | `untested` | `untested` | `discovery-05-scene-transition` |
| `cap:primitives/audio-transition` | Primitives | Constant-power audio crossfade | `declarative:transition:audio` | `declarative:transition:audio` | `untested` | `untested` | `untested` | `untested` | `discovery-05-scene-transition` |
| `cap:animation/stagger` | Animation | Sequential text motion stagger | `declarative:element/text:stagger` | `declarative:element/text:stagger` | `untested` | `untested` | `untested` | `untested` | `discovery-06-motion-stagger` |

---

## 4. Machine-Readable Sync & Schema Validation

The canonical state of this register is maintained in `docs/research/discovery-eval/register.json` and validated by `docs/research/discovery-eval/register.schema.json`. Any update script, evaluation runner, or PR check must ensure:
1. Every capability has an entry in both `register.json` and `REGISTER.md`.
2. Coverage states conform strictly to the 6-state taxonomy.
3. Unavailable and intentionally excluded capabilities are excluded from the evaluation denominator.
4. Tasks only update primary target capabilities upon run completion.
