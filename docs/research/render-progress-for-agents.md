# Notes: how an agent knows what a `render` is doing

Working notes, not a decision. Nothing here amends an ADR yet. They come from an agent's field report after a 30-minute, 13,767-frame render, from a grilling of that report against the code, and from a three-juror court on the four open questions.

**Status:** proposed. The questions below have a recommended answer. Filing issues and drafting the ADRs that amend [ADR-0011](../adr/0011-tool-surface-reads-checks-renders.md) and [ADR-0108](../adr/0108-mcp-tools-run-off-the-runtime-thread-and-a-long-call-is-heard-from-until-it-ends.md) have not happened.

## The report

The agent could see `render N/TOTAL frames SECONDS s` about every 10% of the frames and a summary at the end. It wanted to know what was in progress and could not, reliably, by itself. What it did instead: tailed a log, watched the partial `.mp4` grow, ran `ps`, and divided elapsed by frames. It lost a whole render when its own `grep | awk` pipe buffered the counter. It wanted a progress file, progress every few seconds, and a pre-flight `--estimate`.

## Facts checked in the code

- The CLI writes each progress line with `eprintln!` (`crates/montagent/src/cli.rs`), newline-terminated and unbuffered. The buffering in the report was the agent's own pipe, not Montagent.
- The core reports progress at frame 0, each tenth, and the last frame (`encode_frames` and `Progress` in `crates/montagent-core/src/verbs/render.rs`). Tests assert it (`adapters.rs`, `mcp_concurrency.rs`).
- ADR-0011 chose "Progress on stderr, coarse". ADR-0108 added MCP `notifications/progress` (each tenth, plus a 30 s heartbeat), sent only when the client supplies a `progressToken`.
- The renderer has no per-layer or per-element cost accounting.
- [ADR-0072](../adr/0072-the-render-budget-is-retired-not-replaced.md) retired the render speed budget because a rate read off few points overclaims. The report's own 0.208 to 0.238 s/frame drift is the same effect, so any ETA must be presented as an estimate.
- `skills/montagent/SKILL.md` step 6 tells the agent to render in the foreground and wait for exit. It says nothing about progress.

## Questions and recommended answers

1. **stderr cadence.** Keep the tenths lines and add a line whenever about 5 s pass without one. Every line carries an ETA from a rolling-window rate and the timeline position, and the ETA is labelled an estimate. Amends ADR-0011 and ADR-0108 §2.
2. **`render --progress-file <path>`.** A field on the shared request, so the CLI and MCP both get it. JSON written about once a second via temp file and rename: `state` (running, done, failed, cancelled), `pid`, `updated_at`, `elapsed_s`, `frames_done`, `frames_total`, `fps`, `eta_s`, `timeline_ms`. `pid` and `updated_at` let a reader tell a stale `running` from a live render. On a remote instance the path must be confined to the workspace (see [ADR-0159](../adr/0159-a-remote-instance-moves-bytes-through-a-plain-http-door-into-named-workspaces.md)); that has to be settled before any code.
3. **`render --estimate`.** A separate later issue. Naming the heaviest layer needs cost accounting that does not exist, and a sampled rate runs into ADR-0072. Until then the pre-flight is `render --from --to` on a short slice.
4. **Deliverable.** Issues first, then ADR drafts once Q1 to Q3 are confirmed. (The court split 2 to 1 between issues only and issues plus ADR drafts.)

The court was three jurors (Opus, Sonnet, Haiku): unanimous on Q1 to Q3, split on Q4. Same-family agreement is weak evidence, and the jurors saw a summary of the code, not the code.

## What this does and does not solve

Solves, for an agent that opts in: the ETA, the cadence, "still running, dead, or done", and not depending on how its pipe buffers. A progress bar is a calculation from `frames_done / frames_total`. Over MCP a client that renders `notifications/progress` can show one natively, but whether Claude Code sends a `progressToken` is not documented.

Does not solve: knowing the speed before starting (Q3), or which layer is slow.

## Discoverability: acceptance criteria

An agent only uses what it can find, and today nothing mentions progress.

- The MCP `render` tool description and parameter schema (`crates/montagent/src/mcp.rs`) name the progress file and say what it holds.
- The flag appears in `montagent render --help`.
- `skills/montagent/SKILL.md` says how to use it. This conflicts with step 6's foreground rule: the file matters mainly when the render runs in the background and the agent polls. Open point: how to reconcile the two.
- The skill recommends `render --from --to` on a short slice as the pre-flight until `--estimate` exists.
- The first stderr line of a render hints at the flag, and the final result names the progress file when one was given.
