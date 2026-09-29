# Research: the MCP `render` timeout — Montagent's server or Claude Code's transport?

Research for [#393](https://github.com/MBehtemam/Montagent/issues/393) (MONTAGENT-9), part of the map [#383](https://github.com/MBehtemam/Montagent/issues/383).

**Date of research:** 2026-09-29. Montagent at `4628831d`; `rmcp` **3.4.0** and `tokio` **1.53.1** as pinned in `Cargo.lock` (read from `~/.cargo/registry/src/index.crates.io-*/`); Claude Code **2.1.284** installed; MCP specification revisions `2025-11-25` and `2026-07-28`.

**Sourcing rule applied:** every claim cites Montagent's own source (`file:line`), the SDK source at the exact locked version, the MCP specification, or Claude Code's official documentation and first-party changelog. Each finding is tagged **[verified]** (read in a primary source) or **[inference]** (reasoned from verified facts, not observed). Where Claude Code's behaviour is not documented, this note says **not documented** rather than guessing. The source incident is `research/montagent-findings.md` § MONTAGENT-9 in the `youtube_language_learning` repository, not in this one.

---

## 1. Answer

**Montagent's server owns the fix.** Claude Code's client is behaving exactly as documented and as the MCP specification permits: it aborts a stdio tool call that sends *"no response and no progress notification"* for 30 minutes, and it resets that clock on progress. Montagent's server never sends progress, and it **serialises every request onto one OS thread** — so a call issued behind a running `render` sits unread, with its client-side clock already running, for the whole of the other call's encode. Nothing on the wire distinguishes that from a hang, because nothing is on the wire at all.

Two implementation tickets should graduate, both in this repo (§7). Nothing needs to be raised with Claude Code, beyond an optional request to document two unstated details (§6).

The ticket's third question — should a failed `render` report what is at `output`? — **cannot help with this failure**: once the client aborts, no result from the server reaches the caller. The honest after-the-fact answer is `verify` ([#436](https://github.com/MBehtemam/Montagent/issues/436)), which already has an MCP slot, not a `probe` bolted onto `render`'s failure branch (§5).

---

## 2. What the incident actually showed

From the source doc [verified — `youtube_language_learning/research/montagent-findings.md`, § MONTAGENT-9]:

- The error text: *`MCP server "montagent" tool "render" sent no response or progress for 1800s; aborting. If this server is configured in your MCP settings, set a per-server "timeout" (ms) … otherwise set CLAUDE_CODE_MCP_TOOL_IDLE_TIMEOUT (ms) globally (0 disables).`*
- Unstacked renders in the same session took **830–990 s** of wall time.
- Both timeouts had another `montagent` call — a `frame` on a different project, or a second `render` — issued close to them.
- `ps aux` at the failure showed `montagent mcp` running **one** `ffmpeg`, mid-encode, *"for the request that had supposedly timed out"*.
- The files later turned out complete and correct.

---

## 3. Question 1 — whose clock is the 1800 s?

### 3.1 It is Claude Code's client, and it is documented [verified]

- *"A tool call to an MCP server that sends no response and no progress notification for the idle window aborts with an error instead of waiting for the wall-clock limit. … The idle window defaults to five minutes for HTTP, SSE, WebSocket, and claude.ai connector servers, and to 30 minutes for stdio servers. Before v2.1.203, stdio servers were exempt from the idle timeout."* — [code.claude.com/docs/en/mcp](https://code.claude.com/docs/en/mcp) (MCP timeout paragraphs under the `claude mcp add` tips).
- `CLAUDE_CODE_MCP_TOOL_IDLE_TIMEOUT`: *"Overrides the per-transport defaults of 300000 (5 minutes) for network servers and 1800000 (30 minutes) for stdio servers. Set to `0` to disable the idle check. … capped at the effective `MCP_TOOL_TIMEOUT`. A per-server `timeout` in `.mcp.json` of at least 1000 raises that server's idle window to at least the `timeout` value."* — [code.claude.com/docs/en/env-vars](https://code.claude.com/docs/en/env-vars).
- The per-server `timeout` is a separate **hard wall-clock** limit that *"progress notifications from the server don't extend"*. With `MCP_TOOL_TIMEOUT` unset that limit is about 28 hours — [mcp docs](https://code.claude.com/docs/en/mcp), [env-vars](https://code.claude.com/docs/en/env-vars).
- Changelog history: the idle timeout arrived in 2.1.187 for remote servers (*"override with `CLAUDE_CODE_MCP_TOOL_IDLE_TIMEOUT`"*); per-call watchdogs were fixed in 2.1.113 so that *"a message for one tool call could silently disarm another call's watchdog"* no longer happens — [CHANGELOG.md](https://github.com/anthropics/claude-code/blob/main/CHANGELOG.md).

1800 s is exactly the documented stdio default. The server has no timeout of its own: `serve()` sets none (`crates/montagent/src/mcp.rs:937-954`), and nothing in `rmcp`'s server loop times a request (`rmcp-3.4.0/src/service.rs:1414-1466`).

### 3.2 Progress notifications do reset it [verified]

The docs define idleness as *"no response **and no progress notification**"*, and the error text itself says *"no response or progress"*. Three first-party changelog entries corroborate that Claude Code consumes MCP progress:

- 2.1.271: *"Improved `claude mcp serve`: a running tool call now sends a progress update every 30 seconds, so clients show it is still running and idle timeouts don't abort a long command that prints nothing."* Anthropic applied the same fix to its own server.
- 2.1.153: *"Fixed MCP tool progress notifications not rendering in the collapsed tool view."*
- 2.1.283: *"Fixed MCP progress notifications being discarded once a long-running tool call moved to the background."*

This is the behaviour the spec allows: *"Implementations **MAY** choose to reset the timeout clock when receiving a progress notification corresponding to the request … However, implementations **SHOULD** always enforce a maximum timeout"* — [2026-07-28 cancellation § Timeouts](https://modelcontextprotocol.io/specification/2026-07-28/basic/patterns/cancellation); the same text is in [2025-11-25 lifecycle § Timeouts](https://modelcontextprotocol.io/specification/2025-11-25/basic/lifecycle#timeouts).

### 3.3 When does the clock start? Not documented, but only one answer is possible

The docs do not say when the idle window starts. **[inference]** It can only start when the client sends the request. A stdio client has no signal that a server has *begun* a request: the spec defines no acknowledgement message, and the only request-scoped server→client traffic before the result is `notifications/progress` ([2026-07-28 progress](https://modelcontextprotocol.io/specification/2026-07-28/basic/patterns/progress)). So server-side queueing counts against the window whatever the client does, and the only way a server can tell the client *"queued, not hung"* is a progress notification.

### 3.4 Why calls overlap at all: automatic backgrounding [verified]

Since 2.1.212, *"An MCP tool call in the main conversation that is still running after two minutes moves to a background task instead of blocking the session. Claude receives the task ID immediately and keeps working"* — and *"the per-call limits still apply while the call runs in the background: … the idle timeout set by `CLAUDE_CODE_MCP_TOOL_IDLE_TIMEOUT`"* — [mcp docs, § Automatic backgrounding of long tool calls](https://code.claude.com/docs/en/mcp#automatic-backgrounding-of-long-tool-calls).

**[inference]** A 15-minute `render` is therefore backgrounded at 2 minutes, and the agent is *invited* to keep calling tools — including `frame` or `render` on the same `montagent mcp` process. Concurrent calls to one stdio server are the ordinary case in this client, not an edge case. That is why the source doc saw overlap *"from a scheduled wake-up"*.

---

## 4. Question 2 — does `montagent mcp` serialise, and could it say "queued"?

### 4.1 `rmcp` is concurrent; Montagent's runtime and handlers make it serial [verified]

- **The SDK spawns every request as its own task.** In the service loop, each incoming `JsonRpcMessage::Request` goes to `spawn_service_task(async move { … service.handle_request(request, context) … sink.send(response) })` (`rmcp-3.4.0/src/service.rs:1564-1625`), and `spawn_service_task` is `tokio::spawn` (`service.rs:1327-1334`). Nothing in `rmcp` queues one tool call behind another.
- **Montagent runs that on a single thread.** `serve()` builds `tokio::runtime::Builder::new_current_thread()` (`crates/montagent/src/mcp.rs:938`). The crate does not even enable `rt-multi-thread`: `tokio = { version = "1", features = ["rt", "macros", "io-std"] }` (`crates/montagent/Cargo.toml:22`). Every spawned task, including the service loop that reads stdin and writes stdout, shares that one thread. Its run queue is a FIFO `VecDeque` (`tokio-1.53.1/src/runtime/scheduler/current_thread/mod.rs:65`).
- **Every tool is a synchronous `fn` that runs to completion inside the poll.** `render` is `fn render(&self, …) -> Result<CallToolResult, ErrorData>` (`mcp.rs:569-614`), and the same holds for all nine tools. For a sync tool, `rmcp` calls the function **eagerly** and wraps its return value in an already-ready future: `Box::pin(std::future::ready(self(context.service, $($Tn,)*).into_call_tool_result()))` (`rmcp-3.4.0/src/handler/server/tool.rs:375`, and `:400` for the context-less arity). There is no `spawn_blocking` or `block_in_place` anywhere in `rmcp`'s server path (the only `spawn_blocking` hit in the crate is `transport/common/unix_socket.rs:118`), and none in `mcp.rs`.
- **So the whole encode blocks the only thread.** `montagent_core::verbs::render::render` paints each frame in-process and pushes it to `ffmpeg` in a loop (`crates/montagent-core/src/verbs/render.rs:755-808`). While it runs, the service loop cannot run: it cannot read the next request off stdin, write a response, or send a notification.

**Consequence [verified in code, inference as to timing]:** a second `tools/call` that arrives during a `render` is not dispatched until the render returns. Its client clock started at send (§3.3). If the call ahead of it is a render with ~900 s left and the queued call is itself a ~900 s render, the queued one passes 1800 s mid-encode. That matches the `ps` evidence: one `ffmpeg`, mid-encode, *for the request that timed out*.

### 4.2 A second, probabilistic path: a finished response can wait behind the next call [verified in code]

When a handler finishes, it hands the response to the loop over an mpsc channel (`service.rs:1623`). The loop turns it into a transport write, and that write is itself a **newly spawned task** in `response_send_tasks` (`service.rs:1503-1522`). The loop's `tokio::select!` has no `biased;` (`service.rs:1418-1466`), and tokio documents that *"By default, `select!` randomly picks a branch to check first"* (`tokio-1.53.1/src/macros/select.rs:63-67`).

So if the next request is already buffered when a render finishes, the loop may pick up that request first and spawn its handler task **ahead of** the response-write task in the FIFO queue. The next handler then blocks the thread before the finished response is written. tokio's `Stdout` does its actual `write` on a blocking-pool thread (`tokio-1.53.1/src/io/blocking.rs:109-143`), so once the write task has been polled the bytes get out on their own. The window is between the handler finishing and the write task's first poll.

**[inference]** This path turns *"render A finished"* into *"A's answer arrives when B finishes"*. For A to pass 1800 s this way, B must itself run for roughly as long as A, so it can explain the `render`-behind-`render` incident. It **cannot** explain the `frame` incident alone, because a `frame` takes seconds. It also fits the `ps` evidence worse than §4.1: here the timed-out render's own `ffmpeg` would already have exited. The earlier comment on #393 treats this path as *the* mechanism. It is real, but §4.1 is the better fit to the evidence, and one fix removes both.

### 4.3 An abort does not stop the work, which makes the queue worse [verified]

- The spec: on stdio, when a request times out the sender *"SHOULD cancel the request"* by *"sending a `notifications/cancelled` notification"* ([2026-07-28 cancellation § Timeouts](https://modelcontextprotocol.io/specification/2026-07-28/basic/patterns/cancellation)). Servers *"SHOULD … stop processing the cancelled request"*, and the client *"SHOULD ignore any response to the cancelled request that arrives afterward"* (same page, § Behavior Requirements).
- `rmcp` on receipt cancels the request's `CancellationToken` and removes it from `local_ct_pool` (`service.rs:1654-1659`). When that request's response later arrives, it is dropped: `tracing::debug!(%id, "dropping response for cancelled request")` (`service.rs:1509-1511`).
- Montagent's sync handlers never look at the token. `CancellationToken` is available to a tool as a parameter (`rmcp-3.4.0/src/handler/server/common.rs:161`), but no tool takes it. So an aborted `render` **keeps encoding to completion and publishes** (ADR-0093's *"a file at the output path is a render with zero errors"*, `docs/adr/0093-renders-world-effects-are-findings-and-an-error-withholds-the-deliverable.md:212`). That is why the files were correct, and why the caller's picture (*failed*) and the disk (*succeeded*) disagreed.
- **[inference]** Every aborted-but-still-running render keeps blocking the one thread, so every call issued after it queues behind work nobody is waiting for. One timeout makes the next more likely. That also accounts for the `frame` incident: a `frame` queued behind a render nobody was awaiting any more.
- **Not documented:** whether Claude Code actually sends `notifications/cancelled` when its idle timer fires. The docs say only that the call *"aborts with an error"*. Either way the result is lost: dropped by `rmcp` if the cancel arrives, ignored by the client if it does not.

### 4.4 Could the server send progress while queued or encoding? Yes, but not as the code stands [verified]

- **The protocol has a stream for it.** The comment at `mcp.rs:578-579` says *"the protocol has no stream for it, and the result is the answer"*. That is wrong: `notifications/progress` with the request's `progressToken` is exactly that stream ([2026-07-28 progress](https://modelcontextprotocol.io/specification/2026-07-28/basic/patterns/progress)). The server *"MAY … send notifications at whatever frequency they deem appropriate"*, `progress` *"MUST increase with each notification, even if the total is unknown"*, and notifications *"MUST only reference tokens that were provided in an active request"*. The comment traces back to ADR-0011's *"Progress on stderr, coarse"* (`docs/adr/0011-tool-surface-reads-checks-renders.md:493`), which was written with the CLI in mind.
- **`rmcp` has the API.** `Peer<RoleServer>::notify_progress(ProgressNotificationParam)` (`rmcp-3.4.0/src/service/server.rs:908`; the param type is at `model.rs:1592-1604`). A tool can take `Peer<RoleServer>` (`common.rs:203`) and the request's `RequestMetaObject` (`common.rs:212`) as parameters, and read the token with `RequestMetaObject::get_progress_token()` (`model/meta.rs:436`).
- **A notification needs the service loop to run.** It travels `peer_tx → peer_rx → Event::ProxyMessage → transport.send` (`service.rs:1435-1441`, `1543-1563`). On a current-thread runtime blocked by a sync handler, a notification sent from inside `render`'s progress closure **would not leave the process until the render returned**. So *"just emit progress"* does nothing on its own; it only works once the handler is off the runtime thread.
- **A queued call cannot report itself today.** It has not been read off stdin yet (§4.1). Even a heartbeat for queued calls needs the loop to be free to read and dispatch them, which again means the blocking work is off the runtime thread.
- **Frame-level cadence alone is not enough.** `render` reports progress at frame 0 and at each tenth (`render.rs:750-808`), roughly every 90 s on a 15-minute render. But nothing is reported during pre-flight (checks, probes) or after the last frame (`seal`, `render.rs:817-847`), and a 90-minute render would report every 9 minutes. A **time-based heartbeat** (Anthropic chose 30 s for `claude mcp serve`, changelog 2.1.271) covers every phase, including waiting.
- **Does Claude Code send a `progressToken`?** Not stated in the docs. **[inference]** It must, for the three changelog entries in §3.2 to make sense, since a server may only send progress against a token the client supplied. The ticket's first acceptance step should log whether `tools/call` carries `_meta.progressToken`, and fall back to stderr-only when it does not.

### 4.5 Concurrency hazards to settle in the fix [verified where cited]

- The encoder's temp file is unique per process **and** per nanosecond: `.{stem}.montagent-partial-{pid}-{nanos}` (`crates/montagent-render/src/encode.rs:426-433`). Two in-process renders never share a temp file.
- **[inference]** Two concurrent renders to the **same** `output` would each pass ADR-0104's foreign-output pre-flight against the same disk state, and the last `rename` would win. Parallel dispatch needs a per-output-path lock (or a refusal) so that ADR-0104's check and the publish are not a race.
- **[inference]** Two full encodes in parallel compete for the same CPU. Running long verbs on a bounded pool while *announcing* the wait (a `"queued behind render of <path>"` progress message) gives the source doc's *"visible queue position"* without doubling every render's wall time.

---

## 5. Question 3 — should a failed `render` report what is at `output`?

**Not as a fix for this incident.** Once Claude Code aborts, the server's result never reaches the caller. `rmcp` drops a response to a cancelled request (`service.rs:1509-1511`), and the spec tells the client to ignore one that arrives late (§4.3). No content in a `CallToolResult` can fix a result that is never delivered. So the only server-side remedies are:

1. **Prevent the abort** — progress heartbeats while running *and* while queued (§4.4). This is the real fix.
2. **Make the disk and the verdict agree when an abort still happens** — honour the `CancellationToken` between frames, so a cancelled render withholds (ADR-0093 ruling 6's walk-away, `render.rs:668-672`) instead of publishing a file the caller has been told failed. **[inference]** This is an observable-behaviour change, so under ADR-0031 it needs its ADR, and it trades away finished work. It is not the obvious default, but the current state — *reported failed, silently published* — is the MONTAGENT-9 trap itself.
3. **Let the caller ask afterwards, with a verb that can tell *this* render from an older one.** A `probe` of `output` reports duration and codecs, but it cannot tell a fresh deliverable from a stale file of the same length. That is the confusion ADR-0104 and [#436](https://github.com/MBehtemam/Montagent/issues/436) exist for: two language cuts of one timeline agree to the frame. `verify` reads the stamp and the staleness digest, and #436 already gives it an MCP slot *"since it is MONTAGENT-9's question asked over MCP"* (#383 map). **That is the answer to the source doc's item 3**, and it needs no change to `render`.

For `render` failures that *do* reach the caller (refusals, internal errors), ADR-0093 already guarantees the deliverable was withheld. Whatever sits at `output` was there before this call and is not this call's result. Reporting its contents from `render` would invite exactly the misreading #436 guards against. At most, the refusal text could say *"the declared output was not written by this call"*, which is a one-line wording question, not a ticket.

---

## 6. What, if anything, to raise with Claude Code

Nothing is a defect. Claude Code's behaviour is documented, consistent with the spec's MAY/SHOULD, and has been deliberately hardened (2.1.113, 2.1.203, 2.1.283). Two points are **not documented** and would be worth a docs request, not a bug:

- whether the idle window starts at send (§3.3 — the only possible reading, but unstated);
- whether an idle abort sends `notifications/cancelled` on stdio (§4.3), which the 2026-07-28 spec makes a **MUST** for stdio cancellation.

**User-side mitigation today [verified]:** add a per-server `"timeout"` to montagent's `.mcp.json` entry (e.g. `7200000`). It *"acts as a floor on the idle timeout"* (v2.1.203+), but it is also a hard wall-clock cap on every call. Alternatively set `CLAUDE_CODE_MCP_TOOL_IDLE_TIMEOUT=0` to disable the idle check globally ([mcp docs](https://code.claude.com/docs/en/mcp), [env-vars](https://code.claude.com/docs/en/env-vars)). Neither mitigation stops a queued call from waiting silently. Both only stop the client from giving up on it.

---

## 7. Tickets to graduate

**T1 — Move the blocking verbs off the runtime thread and send MCP progress, heartbeat included.** Server-side, `crates/montagent/src/mcp.rs`.
- Make the long verbs (`render`, `preview`, `frame`, and on principle every tool) `async fn` that run the core call in `tokio::task::spawn_blocking`, so the service loop keeps reading, dispatching and writing. A `current_thread` runtime is fine once no handler blocks it. `rt-multi-thread` is optional.
- Take `Peer<RoleServer>` and `RequestMetaObject` as tool parameters. When a `progressToken` is present, send `notifications/progress` with a monotonically increasing `progress` (frames done, `total` = frames), the core `Progress` callback bridged over a channel, **plus a time-based heartbeat** (~30 s) that covers pre-flight, `seal`, and waiting for a slot. Keep the stderr lines.
- Bound long encodes with a semaphore or pool and say so in the heartbeat (`message: "queued behind render of …"`). Add a per-output-path lock so ADR-0104's pre-flight and the publish are not a race (§4.5).
- Acceptance: a stdio harness issues `render` and then `frame` concurrently; the `frame` result arrives while the render is still encoding; the render's call receives progress at least every N s from dispatch to result; and a log line confirms whether Claude Code supplies `_meta.progressToken`.
- Needs an ADR amending ADR-0011's *"Progress on stderr, coarse"* (`docs/adr/0011-tool-surface-reads-checks-renders.md:493`), and the false comment at `mcp.rs:578-579` goes.

**T2 — Honour cancellation in `render`/`preview`.** Server-side, core and adapter.
- Pass the request's `CancellationToken` into the verb, check it between frames, and on cancel walk away (withhold, ADR-0093 ruling 6) rather than publish.
- Needs its own ADR, because it decides what *"the caller was told it failed"* means for the disk (§5 point 2). It is independent of T1 and lower priority: with T1, idle aborts should stop happening, and T2 covers the user pressing stop in `/tasks`.

**No ticket** for reporting `output`'s contents on a failed `render`: [#436](https://github.com/MBehtemam/Montagent/issues/436) (`verify` over MCP) is the answer, and it is already open.

---

## 8. Open items this research could not close

1. **Whether Claude Code sends `progressToken` on `tools/call`** — implied by the changelog, not documented. T1's acceptance test settles it empirically.
2. **Whether Claude Code sends `notifications/cancelled` on idle abort** — not documented. It changes nothing for T1, and only matters to T2's cost/benefit.
3. **Which of §4.1 and §4.2 produced each of the two incidents** — the source doc records no per-call timestamps. §4.1 fits the `ps` evidence better, and T1 removes both mechanisms. T1's stderr lines should carry the request id and dispatch/finish times, so the next incident is attributable without argument.
4. **MCP tasks** ([2026-07-28 changelog item 6](https://modelcontextprotocol.io/specification/2026-07-28/changelog), SEP-2663: an official `io.modelcontextprotocol/tasks` extension with `tasks/get` polling) would be the protocol-native shape for a 15-minute call. Claude Code's docs and changelog do not mention client support for it, so it is not a basis for a ticket today.
