---
status: accepted
amends: 0011 (*"Progress on stderr, coarse"* was written for the CLI and is wrong for the MCP surface: over MCP, progress also goes to the client as `notifications/progress` whenever the call carries a `progressToken`, with a time-based heartbeat, and the stderr line stays)
---

# MCP tools run off the runtime thread, and a long call is heard from until it ends

> **Amended by [ADR-0192](0192-a-render-writes-a-progress-file-and-says-its-phase-and-eta-or-why-it-has-none.md).**
> §2 gains phases and a 5 s stderr fill, and a render with no `progressToken` can now be followed through
> a progress file. The 30 s heartbeat is unchanged.

> **Amended by [ADR-0109](0109-a-cancelled-encode-publishes-nothing.md).** §4 no longer
> holds: a cancelled `render`/`preview` stops within one frame, publishes nothing, and
> releases the encode slot only once its work has actually stopped.

[#439](https://github.com/MBehtemam/Montagent/issues/439), building what
[#393](https://github.com/MBehtemam/Montagent/issues/393) (MONTAGENT-9) found. Part of
[#383](https://github.com/MBehtemam/Montagent/issues/383). The research is
`docs/research/mcp-render-timeout.md`.

## What was there

Two `render` calls over MCP were aborted by Claude Code after *"no response or progress for
1800s"*, and the files turned out complete and correct. The 1800 s is Claude Code's documented
stdio idle window, and progress resets it. The fault was Montagent's server:

- `serve()` builds a **`current_thread`** runtime.
- Every tool was a **synchronous `fn`**, and `rmcp` calls a sync tool *inside the poll*.

So a `render` held the only thread for its whole encode. The service loop could not read the
next request, write a finished response, or send a notification. A `frame` issued behind a
render waited *unread*, with its client clock already running since it was sent. The server
never sent a progress notification, so a queued call and a hung one looked identical. A comment
in `mcp.rs` justified the silence with *"the protocol has no stream for it"*, which is false:
`notifications/progress` is that stream.

Measured on the commit before this ADR: a `frame` sent right after a `render` came back
**160 µs after the render finished**. It was dispatched in the render's wake, not beside it.

## Decision

### 1. Every tool runs in `spawn_blocking`

All nine tools are `async fn`s that hand their core call to `tokio::task::spawn_blocking`
through one function, `mcp::dispatch::run`. The service loop keeps reading, dispatching and
writing for the whole of an encode. **`current_thread` is kept**, because once no handler
blocks it, one thread is enough to shuttle messages. The CPU work runs on tokio's blocking
pool.

On principle it is every tool, not just the three long ones. `validate` probes media and can
take seconds on a cold cache, and a rule of *"long tools only"* is a rule someone has to
remember when adding the tenth.

### 2. Progress goes on the wire, and a heartbeat covers what the core does not report

When `tools/call` carries `_meta.progressToken`, the call sends `notifications/progress`:

- **on each progress step the core reports**: `render`/`preview` at frame 0, each tenth, and
  the last. `progress` counts frames drawn and `total` is the frame count;
- **on a heartbeat every 30 s** in which nothing else was sent, covering pre-flight (checks,
  probes), `seal` after the last frame, and waiting for the encode slot (§3). Every
  notification carries a `message` saying which phase it is in.

**30 s** is Anthropic's own choice for `claude mcp serve` (Claude Code 2.1.271). It is a tenth
of the *shortest* idle window Claude Code applies to any transport, and a notification is one
short line.

**`progress` strictly increases**, as the spec requires. A notification that brings no new
frame advances it by **1/1024 of a frame**, an exact binary fraction, so the bar stays honest
to within one frame for 1024 heartbeats (8.5 hours). `total` is omitted from any notification
whose `progress` would exceed it, so no client is told it is past 100%.

**With no `progressToken`, no notifications are sent.** The spec allows notifications only
against a token the client supplied, so the stderr lines are the whole report. The loop is
still unblocked, which is the part of the fix that does not depend on the client.

**The stderr lines stay** (ADR-0011's coarse `render  N/M frames  T s`). They gain one line
per call at dispatch, at start, and at finish, carrying the request id and seconds since
dispatch:

```
mcp  #2 render  dispatched  progressToken: present
mcp  #2 render  started  0.0 s after dispatch
mcp  #2 render  finished  2.4 s after dispatch
```

The dispatch line **records whether the client sent a `progressToken`**. Claude Code's
documentation does not say whether it does, and its changelog implies it must. The next
incident can then be attributed from the log, without argument.

### 3. Encodes take one slot, and a queued call says what it waits behind

`render` and `preview` share **one** encode slot, a semaphore of one. A second encode waits,
and its heartbeat says `queued behind render of <project>`, which is the *"visible queue
position"* the source doc asked for. Every other tool runs as soon as it arrives.

**Why one, and why `preview` shares it:**

- Two encodes compete for the same cores, so running them in parallel buys neither of them
  wall clock.
- A `preview` running beside a 4K `render` would miss ADR-0021's scrub budget because of
  contention the project did not cause, then degrade or refuse and blame the project. Queued,
  it runs at the speed its budget was measured at.
- **It closes ADR-0104's race by construction.** Two in-flight renders to one `output` would
  each pass the foreign-output pre-flight against the same disk state. Only one publishing verb
  is ever in flight in this process, so the pre-flight and the publish cannot interleave. The
  ticket's per-output-path lock is therefore not built. **Raising the bound above one requires
  it**, and this ADR is where that is written down.

`frame` is not in the slot. It paints in-process and writes nothing, and answering it during an
encode is this ADR's acceptance test.

**Out of scope:** two `montagent` *processes* rendering to one path. The slot is per server.

### 4. What this does not change

- **An aborted call still runs to completion and publishes.** Honouring the request's
  `CancellationToken` is [#440](https://github.com/MBehtemam/Montagent/issues/440), a separate
  decision about what the disk should hold when the caller was told the call failed. This ADR
  makes the abort much rarer; it does not change what an abort does.
- **No tool schema changes.** Progress is a transport fact, not an argument.
- **The CLI is unchanged.** It already had stderr, which is a stream.

## Test seam

`MONTAGENT_MCP_HEARTBEAT_MS` shortens the heartbeat. It exists so *"progress at least every N s
from dispatch to result"* can be proved in a test's run time, and it is read but not
advertised. The production value is the same code path with a different constant.

## Evidence

- `crates/montagent/tests/mcp_concurrency.rs`:
  - `frame` issued behind `render` is answered **while the render is still encoding**;
  - the render's progress never falls silent for longer than the tolerance between dispatch
    and result;
  - `progress` strictly increases and `total` is never exceeded;
  - the stderr dispatch line records the token as present or absent;
  - a second `render` names the one it is queued behind, and both deliverables land;
  - a call without a token receives no notification.

  Against the commit before this ADR, the first two fail: the `frame` comes back 160 µs *after*
  the render, and the queued render says nothing.
- `docs/adr/mcp_calls_are_heard_from_check.sh` asserts the ordering, the progress stream and
  the dispatch record over the built binary, using a shell and `python3` and no test
  harness, so it can be run against any commit. Against the commit before this ADR, three of its five
  assertions fail.
