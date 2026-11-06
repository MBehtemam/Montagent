# `render --estimate` (Dedicated Pre-Flight Speed Estimate Flag/Verb)

This project does not add a separate `montagent render --estimate` flag or
`estimate` verb. Pre-flight speed estimation and heaviest-element attribution
are performed with `montagent render --from <start_ms> --to <end_ms> --profile`
(or `from`, `to`, and `profile: true` over MCP).

## Why this is out of scope

- **A few sampled frames across a timeline overclaim a single rate.**
  [ADR-0072](../docs/adr/0072-the-render-budget-is-retired-not-replaced.md)
  retired the single-number render rate claim because render cost varies with
  track density, active effects, and static-prefix reuse across the timeline.
- **`render --from <t> --to <t> --profile` already measures the real pipeline.**
  Combining range rendering (`--from` / `--to`) with `--profile` (#931) and
  `prefix_ended_by` (#975) runs the exact decode, paint, composite, and encode
  path on a representative dense slice and reports:
  - actual elapsed time and frame count for the slice (`s/frame` and `fps`),
  - per-element CPU attribution (`id`, `type`, `effect`, `frames_painted`,
    `frames_reused`, `cpu_ms`),
  - `prefix_ended_by` identifying which element broke static-prefix reuse, and
  - per-stage CPU attribution.
- **Live long renders already report rolling-window `fps` and `eta_s`.**
  [ADR-0192](../docs/adr/0192-render-progress-is-an-opt-in-status-file-with-rolling-window-eta-and-honest-phase-boundaries.md)
  (`--progress-file`) reports rolling-window rate and ETA during the full
  render without needing a second estimator.

## Prior requests

- #873 — "`render --estimate`: a pre-flight speed estimate before committing to a long render"
