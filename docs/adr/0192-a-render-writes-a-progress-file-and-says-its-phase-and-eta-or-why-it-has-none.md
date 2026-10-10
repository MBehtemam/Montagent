---
status: accepted
amends: 0011 (*"Progress on stderr, coarse"* no longer holds: stderr also prints a line whenever about 5 s pass without one, and every line names its phase, the timeline position and an ETA labelled an estimate, or says why there is none; and `render` gains one optional request field, `progress_file`, so the nine-verb surface is unchanged), 0108 (§2's *"on each progress step the core reports"* now includes the phases the core reports and the 5 s fill; the 30 s heartbeat is unchanged and the progress file is independent of both channels, so a client that sends no `progressToken` is no longer left with stderr alone)
---

# A render writes a progress file, and says its phase and its ETA, or why it has none

[#872](https://github.com/MBehtemam/Montagent/issues/872). An agent ran a 30-minute,
13,767-frame `render` and could not tell, by itself, what was in progress. It saw about ten
`render N/TOTAL frames SECONDS s` lines and a summary at the end. It tailed a log, watched the
partial `.mp4` grow, ran `ps`, and divided elapsed by frames. It lost a whole render to its own
`grep | awk` pipe. A later session showed the same gap from the other side: a wrapper script
printed *"no progress line yet; no ETA until then"*, and the agent could not tell loading from
hung.

Montagent was not the thing buffering: the CLI line is written with `eprintln!`, newline
terminated. What it lacked was anything to say *between* the tenths, and anything an agent can
read without holding the call open.

## What was there

- The core reports progress at frame 0, each tenth and the last frame, and only after
  pre-flight (checks, mix, encoder start). Before that there is silence. `Progress` carries
  `done`, `of` and `elapsed`, with no phase.
- [ADR-0011](0011-tool-surface-reads-checks-renders.md): *"Progress on stderr, coarse"*, because
  *"a spinner is worth nothing to an agent"*.
- [ADR-0108](0108-mcp-tools-run-off-the-runtime-thread-and-a-long-call-is-heard-from-until-it-ends.md)
  §2: MCP `notifications/progress` on each core step plus a 30 s heartbeat, **only** when the call
  carries a `progressToken`. With none, stderr is the whole report.
- [ADR-0072](0072-the-render-budget-is-retired-not-replaced.md): a rate read off few points
  overclaims. The reported render's own rate drifted 0.208 to 0.238 s/frame.

The coarse cadence was right against a spinner. It is wrong for an agent that must decide
whether a long render is alive, and no rule here makes progress chatty: the new lines carry
facts the agent cannot otherwise get.

## Decision

### 1. The principle: Montagent states the status, the agent never infers it

Every surface says its phase. An absent number says why it is absent. An agent never reads
meaning into silence, a missing field or an unchanged file without a rule written here.

### 2. stderr

Keep the tenths lines. Add a line whenever about 5 s pass without one, **including during
pre-flight**. Every line carries:

- the **phase**: `preparing` (checks, probing, mix, encoder start), `rendering`, `finishing`
  (encoder flush and rename);
- the **timeline position**;
- the **ETA from a rolling-window rate, labelled an estimate** (per ADR-0072), or the words
  saying why there is none (*no frames rendered yet*).

Pre-flight (checks, probing, mix) runs on one thread today and reports nothing, so the 5 s line
there needs a timer that does not depend on the core reaching a step: the same shape as the MCP
adapter's 30 s heartbeat. This is an implementation requirement, not a detail. The first line is
printed when the run starts. It names the phase, the progress file when one
was given (a hint to the flag when none was), and when the next line is due.

### 3. `render`'s `progress_file`

One optional field on the shared request, so the CLI (`--progress-file <path>`) and MCP both
get it. The MCP parameter is `progress_file`, matching `RenderParams`' other fields
(`no_clobber`-style snake case); there is no camelCase in that struct. The path resolves like `output`: relative to the project file's folder, and an absolute
path or a `..` that leaves the workspace is refused as a named finding
([ADR-0053](0053-asset-path-resolution-no-assetroot.md),
[ADR-0159](0159-a-remote-instance-moves-bytes-through-a-plain-http-door-into-named-workspaces.md)
§4). JSON, replaced through a temporary name and a rename, so a reader never sees half a file:

| field | meaning |
| --- | --- |
| `state` | `running`, `done`, `failed`, `cancelled` |
| `phase` | `preparing`, `rendering`, `finishing`; on a terminal `state`, the phase it stopped in |
| `started_at`, `updated_at` | this run's start, and the time of this write |
| `write_interval_s` | how often it is rewritten |
| `pid` | the process id, **for local use only**; a remote agent cannot check it |
| `elapsed_s`, `frames_done`, `frames_total`, `fps`, `timeline_ms` | what has been done |
| `eta_s`, `eta_note` | the ETA in seconds, or `null` with a note saying why |

A progress bar is `frames_done / frames_total`.

### 4. The file is the run's own, from its first moment

A fresh `running` record with `started_at` is written **before project loading and validation**.
`failed` is written if validation fails. A file left by an earlier run at the same path, which
Montagent never deletes, is therefore replaced at once, and an agent polling right after launch
cannot read a stale `done` as this run's. The writer runs on its own timer, not on frame
completion, so one slow frame is not mistaken for a crash. A failed write of the progress file
(a full disk) never fails the render.

A final write happens on every exit path Montagent controls: `done`, `failed`, `cancelled`.

### 5. Staleness is read from the file, not from the agent's clock

A `running` file whose `updated_at` has **not changed across polls spanning at least
`max(3 × write_interval_s, 10 s)`** means the render died: a killed process, an out-of-memory kill, or a
Ctrl-C on the CLI (which sends a signal and gives the core no chance to write; no signal handler
is added). The agent compares the file with itself between polls, never `updated_at` with its
own clock, because a remote agent's clock can differ from the instance's.

### 6. Remote and MCP

No new door. The progress file is a workspace file, so the byte door's `GET /files/<workspace
path>` serves it. The URL is always `/files/` plus the workspace-relative path the agent chose,
and that rule is stated up front, in the skill and in the first stderr line, because the
`render` result only arrives at the end. The result also names the progress file (path and URL)
on success, failure and cancel.

The 30 s MCP heartbeat stays as it is. The file does not depend on a `progressToken`, which is
what lets an MCP client that sends none finally see progress. Backgrounding a render on MCP
needs a client that can make a second call while `render` is open; one that cannot relies on
`notifications/progress`.

### 7. The skill

[`skills/montagent/SKILL.md`](../../skills/montagent/SKILL.md) step 6 keeps foreground as the
default and keeps *end your turn only after it exits*. It gains one exception, a calculation and
not a judgement: render a short `--from --to` slice first; if `frames_total / slice_fps` is above
about 120 s, pass `--progress-file` with a fresh path, run the render in the background, and
poll every 10 to 30 s until `state` is terminal or the file is stale by section 5. Do not end
the turn before then. Read `phase`, `state` and `eta_note` from the file; do not write wrapper
scripts that guess. The pre-flight slice stays the answer until
[`render --estimate`](https://github.com/MBehtemam/Montagent/issues/873) exists.

## Consequences

- ADR-0011 and ADR-0108 §2 are amended, with banners and index rows
  (`check_amendment_banners.py`). The nine-verb surface and the 30 s heartbeat are unchanged.
- The tests that assert tenths (`adapters.rs`, `mcp_concurrency.rs`) change, and new ones pin
  the early write, the replace-a-stale-`done` rule, the phases and the `eta_note`.
- The core `Progress` gains a phase, an ETA and a timeline position. CLI, MCP and the file read
  one source.
- `--progress-file`, the MCP `render` description and schema, and `SKILL.md` all describe the
  file, because a feature nobody can find does not solve the problem.

## Considered and not chosen

- **Always write a progress file.** One recipe and no branch, but it leaves a stray file beside
  every render. Passing `--progress-file` is cheap, and the skill's calculated threshold already
  removes the judgement call.
- **A signal handler for `cancelled` on Ctrl-C.** A hard-to-test path that gains little, because
  section 5 already tells a reader the render died.
- **Comparing `updated_at` with the agent's clock.** Clock skew gives false verdicts. Comparing
  the file with itself does not.
- **Making stderr chatty with a spinner.** Not chosen. The 5 s line exists to carry a phase and
  an ETA, not to show motion.

## Owner calls (confirmed by the owner)

1. A Ctrl-C on the CLI leaves `running`, and section 5 is the whole remedy. A handler that writes
   `cancelled` is the alternative.
2. The 120 s threshold in the skill is a starting point from one reported render. It is a
   constant to revisit, not a finding.
3. The `max(3 × write_interval_s, 10 s)` stale window is a choice, not a measurement. The 10 s floor
   stops a once-a-second writer from being declared dead after about 3 s; it costs slower detection
   of a real death.
4. Whether the byte door should send `Last-Modified` on a workspace file. This ADR does not need
   it; section 5 avoids the clock entirely.
