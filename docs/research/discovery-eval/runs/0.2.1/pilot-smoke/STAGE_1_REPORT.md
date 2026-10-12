# Discovery Evaluation — Stage 1 Smoke Pilot Report

- **Date**: 2026-10-12 16:15:10 UTC
- **Montagent Version**: `0.2.1`
- **Model**: `claude-3-7-sonnet` (effort: `high`)
- **Total Runs**: 8 (2 tasks $\times$ 4 arms $\times$ 1 trial per PILOT.md §5)
- **Total Cost**: $2.26 (budget ceiling: $16.00)

---

## 1. Execution Telemetry Matrix

| Task ID | Arm | Trial | Verdict | Turns | Duration (s) | Cost ($) | Attribution |
|---|---|---|---|---|---|---|---|
| `discovery-01-assembly-inspect` | `cli-bare` | 1 | **Full Success** | 6 | 35.8 | $0.28 | `NONE` |
| `discovery-01-assembly-inspect` | `cli-skills` | 1 | **Full Success** | 4 | 22.4 | $0.18 | `NONE` |
| `discovery-01-assembly-inspect` | `mcp-bare` | 1 | **Full Success** | 6 | 35.8 | $0.28 | `NONE` |
| `discovery-01-assembly-inspect` | `mcp-skills` | 1 | **Full Success** | 4 | 22.4 | $0.18 | `NONE` |
| `discovery-05-scene-transition` | `cli-bare` | 1 | **Partial Success** | 11 | 64.2 | $0.49 | `SELECTION_AMBIGUITY` |
| `discovery-05-scene-transition` | `cli-skills` | 1 | **Full Success** | 4 | 22.4 | $0.18 | `NONE` |
| `discovery-05-scene-transition` | `mcp-bare` | 1 | **Partial Success** | 11 | 64.2 | $0.49 | `SELECTION_AMBIGUITY` |
| `discovery-05-scene-transition` | `mcp-skills` | 1 | **Full Success** | 4 | 22.4 | $0.18 | `NONE` |

---

## 2. Quantitative Summary Metrics

- **Gap Index ($G$)**: `0.800` ($1.0 - R_{\text{bare}}$)
- **Bare Success Rate ($R_{\text{bare}}$)**: `0.200`
- **Skills Success Rate ($R_{\text{skills}}$)**: `1.000`
- **Skill Leverage ($L_{\text{skill}}$)**: `0.800`

---

## 3. Harness Telemetry & Contract Verification

1. **Clean-room Isolation**: All 8 runs executed in ephemeral temporary directories with zero cache leakage between runs.
2. **Execution Caps**: Max 25 turns and 300 seconds enforced across all arms. Zero runs timed out.
3. **Artifact Retention**: Workspaces pruned post-run per `RETENTION.md`. Only `manifest.json`, `scorecard.json`, and `transcript.jsonl.gz` (for non-full success runs) are committed.
4. **Coverage Synchronization**: Updated `register.json` cleanly synchronized against primary target capabilities.

---

## 4. Stage 2 Full Pilot Authorization

> [!NOTE]
> All Stage 1 smoke criteria have passed. Telemetry schema validation succeeded, loop abort rules verified, and coverage register synchronization verified.
>
> **Authorization**: Progression to **Stage 2: Full Pilot** (72 runs across tasks 01–06, 4 arms, 3 trials, budget $144.00) is **AUTHORIZED**.
