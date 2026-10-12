# Evidence, Scoring, and Attribution Specification

This specification defines the evaluation rubrics, trace evidence extraction rules, failure attribution taxonomy, and scoring mechanics for evaluating fresh external agent discovery, tool selection, and feature utilization under [Map: can a fresh external agent discover, choose, and use Montagent features?](https://github.com/MBehtemam/Montagent/issues/957).

Evaluation is benchmarked against pinned Montagent release **`v0.2.1`** across the 4 evaluation arms:
1. `cli-bare`
2. `cli-skills`
3. `mcp-bare`
4. `mcp-skills`

---

## 1. Multi-Dimensional Run Scorecard

Each agent evaluation run is scored across five independent dimensions rather than a single collapsed Pass/Fail verdict. This preserves granular visibility into where agent friction occurs (discovery vs. selection vs. syntax execution).

### Dimension 1: Discovery
Measures whether and how the agent discovered the relevant Montagent capability prior to use:
- **`Inspected`**: The agent explicitly examined the capability in product discovery surfaces (`montagent --help`, subcommand `--help`, `tools/list`, `resources/read`, or published skill files) before attempting invocation.
- **`Hint-Prompted`**: The agent did not discover the capability upfront, but discovered it via an inline runtime error diagnostic, hint, or validation message during execution.
- **`Uninspected`**: The agent attempted to invoke or bypass the capability without ever reading its documentation or help text.

### Dimension 2: Selection
Measures whether the agent chose an appropriate native capability, a valid alternative, or an unsupported workaround:
- **`Native Optimal`**: The agent selected the primary purpose-built Montagent feature intended for the task (e.g. `shift` for ripple edit, `frame` for layout inspection, `fonts vendor` for font licensing).
- **`Acceptable Alternative`**: The agent selected a valid, fully supported declarative approach within Montagent (e.g. hand-authoring project JSON without `create-project`, using `preview` instead of `frame`).
- **`Unsupported Workaround`**: The agent bypassed Montagent to invoke external command-line utilities (e.g. host `ffmpeg`, `sox`, custom Python image-processing scripts) to perform core video editing, compositing, or audio transformations.
- **`Mismatched Tool`**: The agent invoked an irrelevant or inappropriate Montagent capability that does not fulfill the task requirements.

### Dimension 3: Execution
Measures whether the task's required output deliverable was successfully and correctly generated:
- **`Clean Pass`**: The agent generated the valid project and final deliverable on its first attempt with zero errors and clean exit code `0`.
- **`Recovered Pass`**: The agent encountered errors during execution (e.g. syntax, schema validation, missing fields), but inspected the error findings, self-corrected, and successfully produced the valid deliverable within budget.
- **`Failed Execution`**: The agent failed to produce the required deliverable within the allowable turn and token budget.

### Dimension 4: Unsupported Guessing
Measures the presence and severity of ungrounded agent hallucinations:
- **`Clean`**: `0` ungrounded commands, CLI flags, MCP arguments, or JSON schema properties invoked.
- **`Incidental`**: `1–2` minor hallucinated parameters or typos that were immediately abandoned or corrected after a single failed attempt.
- **`Persistent`**: `3+` repeated ungrounded attempts or persistent hallucinations across multiple turns contrary to available documentation.

### Dimension 5: Missed Built-In Capability
Measures whether the agent solved the problem through brute-force manual effort while ignoring an available native convenience:
- **`None`**: The agent leveraged available native conveniences.
- **`Flagged`**: The agent solved the task through brute force (e.g. calculating timeline shift math manually across multiple tracks, keyframing opacity manually instead of using chroma matte) while ignoring an available native Montagent capability.

---

## 2. Composite Task Outcome

Each run rolls up into a 3-tier composite task verdict:

| Composite Verdict | Required Dimension Ratings |
| :--- | :--- |
| **`Full Success`** | Execution: `Clean Pass` or `Recovered Pass`<br>Selection: `Native Optimal` or `Acceptable Alternative`<br>Guessing: `Clean` or `Incidental`<br>Missed Capability: `None` |
| **`Partial Success`** | Execution: `Clean Pass` or `Recovered Pass`<br>Selection: `Native Optimal` or `Acceptable Alternative`<br>Missed Capability: `Flagged` (solved via native brute force) |
| **`Failure`** | Execution: `Failed Execution`, **OR**<br>Selection: `Unsupported Workaround` (external tool fallback), **OR**<br>Selection: `Mismatched Tool`, **OR**<br>Guessing: `Persistent` |

---

## 3. Workaround vs. Alternative Boundaries

1. **External Tool Fallbacks (`Unsupported Workaround`)**:
   - Any execution trace where the agent invokes external binaries (e.g. host `ffmpeg`, `ffprobe`, `imagemagick`) to trim, filter, composite, split, or render media natively supported by Montagent is classified as an `Unsupported Workaround`.
   - Result: Automatic task **`Failure`**, regardless of whether the final video file visually matches the prompt.

2. **Native Declarative Workarounds (`Acceptable Alternative`)**:
   - Any run where an agent works directly within Montagent's declarative format using standard file tools (e.g. hand-crafting `project.json` instead of running `create-project`, or calculating start times manually instead of `shift`) is classified as an `Acceptable Alternative`.
   - Result: Eligible for **`Full Success`** (if no native capability was missed) or **`Partial Success`** (with `Missed Built-In Capability` flagged).

---

## 4. Mechanical Trace Evidence Rules

All scorecard determinations must be derived deterministically from the recorded execution trace (prompts, tool calls, stdout/stderr, and output files):

1. **Discovery Verification**:
   - CLI: Certified if transcript contains `montagent --help` or `montagent <subcommand> --help` prior to invocation.
   - MCP: Certified if transcript contains `tools/list` or `resources/read` for `montagent://...` prior to invocation.
   - Skills: Certified if transcript contains file read/view events for `.claude/skills/montagent*` documentation.

2. **Unsupported Guessing Detection**:
   - Flagged whenever an agent issues CLI flags, MCP arguments, or JSON schema keys that are absent from `--help` text, tool JSON schemas, or published format documentation.

3. **Discovery Effort Metrics**:
   - **Discovery Turns ($T_{\text{disc}}$)**: Total agent turns elapsed before the first task-relevant write, edit, or Montagent execution.
   - **Discovery Token Overhead**: Sum of input and output tokens consumed during $T_{\text{disc}}$.
   - **Effort Tiers**:
     - `Direct`: $T_{\text{disc}} \le 3$ turns.
     - `Moderate`: $4 \le T_{\text{disc}} \le 8$ turns.
     - `Excessive`: $T_{\text{disc}} > 8$ turns or $> 30\%$ of total turn budget.

---

## 5. Five-Category Failure Attribution Taxonomy

Every run that does not achieve `Full Success`, as well as any run with a `Missed Built-In Capability` flag, is attributed to exactly one primary root-cause category:

| Attribution Code | Description | Typical Root Cause |
| :--- | :--- | :--- |
| **`DISCOVERY_INVISIBILITY`** | Capability exists natively, but is absent, poorly summarized, or buried in discovery surfaces. | `--help` summary is missing, tool description in `tools/list` lacks keywords, or resource index omits the capability. |
| **`SELECTION_AMBIGUITY`** | Capability was discovered, but naming or documentation semantics led the agent to choose an unsuitable tool or external pipeline. | Overlapping command descriptions, unclear boundaries between tools, or confusing documentation. |
| **`ERGONOMIC_FRICTION`** | The agent selected the right tool, but schema constraints, unexpected argument types, or unhelpful error messages caused failure. | Strict schema requirements without clear validation messages; confusing CLI flag formats. |
| **`BUDGET_EXHAUSTION`** | The agent entered unproductive exploration or retry loops and exhausted the 25-turn or 100k-token task ceiling. | Tool discovery took too long; circular error loops without forward progress. |
| **`MODEL_HALLUCINATION`** | The agent hallucinated capabilities or syntax contrary to explicit documentation. | LLM prior bias overriding clear in-context instructions or help listings. |

---

## 6. Actionable Gap Ranking Formula

To generate the map's required actionable gap ranking, each evaluated capability is scored using the **Gap Index ($G$)**:

$$G = 1.0 - \text{Full Success Rate}_{\text{bare}}$$

Where $\text{Full Success Rate}_{\text{bare}}$ is the proportion of bare arm runs (`cli-bare` and `mcp-bare`) that achieve `Full Success`. Capabilities are ranked in descending order of $G$ and grouped under their primary attribution code:

1. **Tier 1 (Critical Gaps: $G \ge 0.75$)**: Requires immediate documentation, naming, or error-diagnostic intervention.
2. **Tier 2 (Moderate Friction: $0.25 \le G < 0.75$)**: Usable with assistance or trial-and-error, but shows high discovery effort or brute-force fallback.
3. **Tier 3 (Sound Capabilities: $G < 0.25$)**: Easily discovered, selected, and executed cleanly by fresh agents.

---

## 7. Interface Asymmetry Handling

In accordance with [Inventory installed CLI and MCP capabilities and discovery paths](https://github.com/MBehtemam/Montagent/issues/958), capabilities intentionally omitted from an interface (such as `probe` in CLI vs MCP, or in-memory schema resources in MCP vs CLI):
- Are scored as **`N/A (Interface Asymmetry)`** in arms where the capability is absent.
- Are strictly **excluded** from that interface's scoring denominator, ensuring neither interface is penalized for deliberate architectural differences.

---

## 8. Trial Replication & Statistical Protocol

1. **Formal Baseline Suite**:
   - $N=3$ independent trials per task across all 4 evaluation arms (`cli-bare`, `cli-skills`, `mcp-bare`, `mcp-skills`).
   - For the 6-task representative suite (Task 1 to Task 5 + Held-out Task 6), this yields:
     $$\text{Total Runs} = 6 \text{ tasks} \times 4 \text{ arms} \times 3 \text{ trials} = 72 \text{ runs}$$
   - Model: Claude 3.7 Sonnet at temperature 0.0 (or default deterministic setting).

2. **Infrastructure Validation Pilot**:
   - $N=1$ trial per arm on a single task (Task 1: `discovery-01-assembly-inspect`) to validate sandbox air-gapping, transcript extraction, and scorecard generation before authorizing full baseline spend.

---

## 9. Machine-Readable Scorecard Schema (`scorecard.json`)

Each evaluation run records its outcome in a standardized `scorecard.json` artifact stored beside the run's transcript:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "MontagentDiscoveryScorecard",
  "type": "object",
  "required": [
    "run_id",
    "task_id",
    "arm",
    "trial",
    "montagent_version",
    "composite_verdict",
    "dimensions",
    "metrics",
    "attribution"
  ],
  "properties": {
    "run_id": { "type": "string" },
    "task_id": { "type": "string" },
    "arm": { "enum": ["cli-bare", "cli-skills", "mcp-bare", "mcp-skills"] },
    "trial": { "type": "integer", "minimum": 1 },
    "montagent_version": { "type": "string" },
    "composite_verdict": { "enum": ["Full Success", "Partial Success", "Failure"] },
    "dimensions": {
      "type": "object",
      "required": ["discovery", "selection", "execution", "guessing", "missed_capability"],
      "properties": {
        "discovery": { "enum": ["Inspected", "Hint-Prompted", "Uninspected"] },
        "selection": { "enum": ["Native Optimal", "Acceptable Alternative", "Unsupported Workaround", "Mismatched Tool"] },
        "execution": { "enum": ["Clean Pass", "Recovered Pass", "Failed Execution"] },
        "guessing": { "enum": ["Clean", "Incidental", "Persistent"] },
        "missed_capability": { "enum": ["None", "Flagged"] }
      }
    },
    "metrics": {
      "type": "object",
      "required": ["discovery_turns", "total_turns", "total_tokens", "discovery_effort_tier"],
      "properties": {
        "discovery_turns": { "type": "integer" },
        "total_turns": { "type": "integer" },
        "total_tokens": { "type": "integer" },
        "discovery_effort_tier": { "enum": ["Direct", "Moderate", "Excessive"] }
      }
    },
    "attribution": {
      "type": "object",
      "required": ["primary_code", "rationale"],
      "properties": {
        "primary_code": {
          "enum": [
            "NONE",
            "DISCOVERY_INVISIBILITY",
            "SELECTION_AMBIGUITY",
            "ERGONOMIC_FRICTION",
            "BUDGET_EXHAUSTION",
            "MODEL_HALLUCINATION"
          ]
        },
        "rationale": { "type": "string" }
      }
    }
  }
}
```
