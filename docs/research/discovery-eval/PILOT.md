# Pilot Budget, Infrastructure, and Stopping Rules Specification

This specification defines the execution parameters, resource budgets, stopping criteria, and infrastructure architecture for the discovery and selection evaluation under [Map: can a fresh external agent discover, choose, and use Montagent features?](https://github.com/MBehtemam/Montagent/issues/957).

Evaluation is benchmarked against pinned Montagent release **`v0.2.1`** across the 4 evaluation arms:
1. `cli-bare`
2. `cli-skills`
3. `mcp-bare`
4. `mcp-skills`

---

## 1. Harness Architecture and Code Reuse

The discovery evaluation requires short, focused, zero-knowledge task runs with mechanical telemetry extraction rather than long creative briefs.

### Harness Separation & Reuse Boundaries
- **Dedicated Runner**: Evaluation logic lives in `docs/research/discovery-eval/harness/run_arm.py`, maintaining complete separation from the creative output evaluation in `docs/research/skills-eval/`.
- **Shared Primitives**: The runner imports and adapts proven process isolation, file locking, and sandboxed Claude invocation patterns from `docs/research/skills-eval/harness/common.py`:
  - `one_run_at_a_time()`: POSIX file lock (`run.lock`) preventing temp file collisions in `/private/tmp/claude-<uid>`.
  - `sweep_sandbox_tmp()`: Isolation cleanup ensuring ephemeral files are swept into run-specific scratch directories.
  - Air-gapped network permissions (`permissions.deny: ["WebFetch", "WebSearch"]` and strict local-only sandbox execution).
  - Clean-room environment generation (`build_env()`, `path_without_montagent()`, isolated tool caches).

### Directory & Artifact Layout
```
docs/research/discovery-eval/
├── PILOT.md                # This specification
├── SCORING.md              # Scoring rubric, 5-dimension scorecard, attribution taxonomy
├── REGISTER.md             # Human-readable capability coverage register
├── register.json           # Machine-readable capability coverage manifest
├── register.schema.json    # JSON schema for coverage manifest
├── harness/
│   ├── pins.json           # Model pin, caps, timeout, tool settings
│   ├── run_arm.py          # Dedicated 4-arm discovery evaluation runner
│   └── score_trace.py      # Automated trace parser & scorecard evaluator
├── tasks/
│   ├── discovery-01-assembly-frame/
│   ├── discovery-02-font-licensing/
│   ├── discovery-03-chroma-key/
│   ├── discovery-04-timeline-shift/
│   ├── discovery-05-scene-transition/
│   └── discovery-06-motion-stagger/
└── runs/
    └── <phase>/<task-id>/<arm>-<trial>/
        ├── manifest.json   # Pins, timestamps, turn count, token count, spend, exit code
        ├── transcript.jsonl# Verbatim Claude stream-json output
        ├── stderr.log      # Raw process stderr
        ├── scorecard.json  # Evaluated 5-dimension scores and composite verdict
        └── workspace/      # Kept files produced by the agent (< 2 MB)
```

---

## 2. Evaluation Model Pin

- **Model**: `claude-3-7-sonnet`
- **Reasoning Effort**: `high`
- **Output Format**: `stream-json` (`--verbose`)
- **Permission Mode**: `dontAsk` (all unknown capability prompts denied, strictly sandboxed)
- **Rationale**: An instruction-tuned developer model represents realistic external agent capabilities for reading CLI manuals, discovering MCP tools, diagnosing schema errors, and writing structured JSON without paying creative video generation latencies.

---

## 3. Execution Caps per Run

Each task attempt is strictly bounded to prevent runaways and development bottlenecks:

| Parameter | Cap | Rationale |
| :--- | :--- | :--- |
| **Max Wall-Clock Time** | **300 seconds** (5 minutes) | Discovery tasks are bounded; any session exceeding 5 minutes indicates an unproductive loop or deadlock. |
| **Max Agent Turns** | **25 turns** | Settled in ticket #959. Sufficient for `--help` inspection, tool listing, attempt, error recovery, and validation. |
| **Max Context Tokens** | **100,000 tokens** | Settled in ticket #959. Enforces lean context hygiene and prevents multi-round context flooding. |
| **Max Spend Ceiling** | **$2.00 / run** | Hard budget ceiling passed via `--max-budget-usd 2.00`. Expected actual spend is $0.15–$0.40 per run. |

---

## 4. Loop Abort and Early Stopping Rules

In addition to hard ceilings, the runner monitors agent actions in real time and aborts early upon detecting unproductive failure patterns:

1. **Repetitive Error Loop**: If an agent issues **5 consecutive failed tool calls or CLI invocations with identical error signatures** (same exit code, same stderr, or same MCP error message), the runner halts execution immediately.
   - *Scoring Attribution*: Attributed to `ERGONOMIC_FRICTION` (if agent selected the correct tool but failed schema constraints) or `BUDGET_EXHAUSTION`.
2. **Hallucination Thrashing**: If an agent attempts **3 consecutive ungrounded commands or flags** without inspecting documentation or help surfaces after the first failure, the runner halts execution immediately.
   - *Scoring Attribution*: Attributed to `MODEL_HALLUCINATION`.
3. **Timeout / Turn Depletion**: Hitting the 300s wall-clock limit or 25-turn ceiling triggers graceful SIGTERM, followed after 15s by SIGKILL.
   - *Scoring Attribution*: Attributed to `BUDGET_EXHAUSTION`.

---

## 5. Two-Stage Pilot Progression

To ensure infrastructure stability before committing full evaluation budgets, execution progresses across two stages:

### Stage 1: Smoke Pilot (8 Runs)
- **Scope**: 2 representative tasks across all 4 arms ($N=1$):
  - `discovery-01-assembly-frame` (Focused inspection: single CLI/MCP capability)
  - `discovery-05-scene-transition` (Multi-feature scene narrative: multiple tracks, transition element, audio mix)
- **Total Runs**: $2 \text{ tasks} \times 4 \text{ arms} \times 1 \text{ trial} = 8 \text{ runs}$.
- **Ceiling Budget**: 8 runs $\times$ $2.00 = $16.00 max (expected actual: ~$2.50 total).
- **Objective**: Verify harness isolation, clean-room workspace setup, transcript streaming, trace event extraction, and automated scorecard generation.

### Stage 2: Full Exploration Pilot (24 Runs Total)
- **Scope**: All 6 tasks across all 4 arms ($N=1$):
  - Includes Tasks 1 through 5 plus sealed held-out Task 6 (`discovery-06-motion-stagger`).
- **Total Runs**: $6 \text{ tasks} \times 4 \text{ arms} \times 1 \text{ trial} = 24 \text{ runs}$ (16 incremental runs after Stage 1).
- **Ceiling Budget**: 24 runs $\times$ $2.00 = $48.00 max (expected actual: ~$7.50 total).
- **Objective**: Validate all task prompts, fixture assets, acceptable alternative paths, and failure attribution rules across the complete suite.

---

## 6. Repeatability, Noise, and Minimum Evidence Gating

| Evaluation Phase | Sample Size | Purpose & Authority |
| :--- | :--- | :--- |
| **Infrastructure Smoke** | $N=1$ (8 runs) | Proves harness execution and scorecard generation; uncommitted telemetry. |
| **Exploratory Pilot** | $N=1$ (24 runs) | Proves all task fixtures and scoring rubrics; produces an **uncommitted exploratory Gap Index preview**. No coverage register state transitions. |
| **Authoritative Baseline** | $N=3$ (72 runs) | Formal evaluation suite across all 4 arms ($6 \times 4 \times 3$). Mitigates LLM non-determinism. **Authorizes official coverage register state transitions** (`passed`, `failed`) and commits the official Gap Index rankings. |

### Minimum Evidence Gating Rules
1. A capability can only transition to `passed` in `register.json` if **all 3 trials** in the relevant arm achieve `Full Success` (or `Acceptable Alternative` without missed capability).
2. Any single trial resulting in `Failure` or flagged `Missed Built-In Capability` across the 3 trials marks the capability `failed` in that arm.
3. The pilot ($N=1$) provides immediate feedback on prompt clarity and friction without creating a development bottleneck, while the $N=3$ baseline protects against stochastic noise before engineering prioritization.
