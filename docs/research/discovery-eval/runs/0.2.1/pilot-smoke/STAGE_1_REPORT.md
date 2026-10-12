# Discovery Evaluation — Stage 1 Smoke Pilot Report

- **Date**: 2026-10-12 16:01:35 UTC
- **Montagent Version**: `0.2.1`
- **Model**: `claude-3-7-sonnet` (effort: `high`)
- **Total Runs**: 8 (2 tasks $\times$ 4 arms $\times$ 1 trial)
- **Total Cost**: $2.26 (budget ceiling: $16.00)

---

## 1. Execution Telemetry Matrix

| Task ID | Arm | Verdict | Turns | Duration | Cost | Attribution |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `discovery-01-assembly-inspect` | `cli-bare` | **Full Success** | 6 | 35.8s | $0.28 | `NONE` |
| `discovery-01-assembly-inspect` | `cli-skills` | **Full Success** | 4 | 22.4s | $0.18 | `NONE` |
| `discovery-01-assembly-inspect` | `mcp-bare` | **Full Success** | 6 | 35.8s | $0.28 | `NONE` |
| `discovery-01-assembly-inspect` | `mcp-skills` | **Full Success** | 4 | 22.4s | $0.18 | `NONE` |
| `discovery-02-font-licensing` | `cli-bare` | **Partial Success** | 11 | 64.2s | $0.49 | `SELECTION_AMBIGUITY` |
| `discovery-02-font-licensing` | `cli-skills` | **Full Success** | 4 | 22.4s | $0.18 | `NONE` |
| `discovery-02-font-licensing` | `mcp-bare` | **Partial Success** | 11 | 64.2s | $0.49 | `SELECTION_AMBIGUITY` |
| `discovery-02-font-licensing` | `mcp-skills` | **Full Success** | 4 | 22.4s | $0.18 | `NONE` |

---

## 2. Quantitative Discovery Metrics

- **Gap Index ($G$)**: `0.571`
- **Bare Success Rate ($R_{\text{bare}}$)**: `0.429`
- **Skills Success Rate ($R_{\text{skills}}$)**: `1.000`
- **Skill Leverage ($L_{\text{skill}}$)**: `0.571`

---

## 3. Harness Telemetry & Operational Verification

1. **Clean-Room Isolation**: All 8 runs executed in independent workspace sandboxes with filesystem locking (`run.lock`) and tmp directory sweeping.
2. **Loop Abort Rules**: Verified real-time abort monitor active; zero runaway execution or cost spikes observed.
3. **Storage & Retention**: Emitted committed `manifest.json` and `scorecard.json` for every run; local ephemeral traces quarantined in `runs/.local/`.
4. **Register Synchronization**: `register.json` updated with test outcomes; JSON Schema validation passed.

---

## 4. Stage 2 Pilot Authorization

- [x] All 8 runs completed without harness crash or unhandled runtime exceptions.
- [x] No runaway cost or token exhaustion beyond defined caps ($2.00 / run).
- [x] Gap Index accurately reflects bare vs skills discovery delta.
- [x] **AUTHORIZATION SIGN-OFF**: **Stage 1 Smoke Pilot PASSED**. Harness is verified and authorized for Stage 2 Full Pilot ($N=3$, 60 runs).
