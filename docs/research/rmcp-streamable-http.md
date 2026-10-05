# Research: what rmcp's Streamable HTTP server gives Montagent, and which clients connect

Research for [#657](https://github.com/MBehtemam/Montagent/issues/657), part of the map [#656](https://github.com/MBehtemam/Montagent/issues/656).

**Date of research:** 2026-10-05. `rmcp` **3.4.0** as locked in `Cargo.lock` (source read from `~/.cargo/registry/src/index.crates.io-*/rmcp-3.4.0/`); latest upstream is 3.5.0 (2026-09-28), whose changelog has no Streamable HTTP server change beyond header/Origin fixes. n8n at `master` `e6bb9e0f` (pins `@modelcontextprotocol/sdk` **1.26.0**). Hermes agent at `main` `93c9360a` (pins Python `mcp==2.0.0`). Claude Code **2.1.284** docs. MCP specification revisions `2025-11-25` and `2026-07-28`.

**Sourcing rule:** every claim cites the SDK source at the locked version, a client's own source or first-party docs, or the MCP specification. **[verified]** = read in a primary source; **[inference]** = reasoned from verified facts, not observed. Nothing here was run against a live server; the prototype ticket of #656 should confirm the [inference] rows.

---

## 1. Answer in brief

- **One feature flag, no new framework.** Add `transport-streamable-http-server` to `rmcp`. It yields `StreamableHttpService`, a plain `tower::Service`; mounting it needs an HTTP server (axum is what upstream's examples use, `nest_service("/mcp", service)`). axum, a TCP listener and a bearer-token middleware are Montagent's to add.
- **Both protocol eras are served by one endpoint.** rmcp 3.4 knows protocol versions `2024-11-05` … `2026-07-28`. `2026-07-28` requests are always served statelessly; older (`initialize`-handshake) clients get either sessions or stateless service depending on one config flag.
- **Long calls stream.** A `tools/call` answered over SSE carries `notifications/progress` (ADR-0108 already emits them when a `progressToken` is present) and 15 s keep-alive comments. No result-size cap on the server side; only the *request* body is capped (4 MiB default).
- **All three clients speak Streamable HTTP and send a bearer header.** The thing that will bite is **tool-call timeouts**: n8n defaults to **60 s**, Hermes to **300 s**, Claude Code to a **5-minute idle** window. Only Claude Code's is progress-aware. A multi-minute `render` needs the user to raise a timeout in n8n and Hermes, or a different call shape (§5).
- **No client forces legacy HTTP+SSE.** n8n's old `MCP Client Tool` v1 is SSE-only, but v1.2+ defaults to HTTP Streamable.

---

## 2. rmcp 3.4.0 — the server side

### 2.1 Feature flags [verified]

From `rmcp-3.4.0/Cargo.toml` `[features]`:

- `transport-streamable-http-server = ["transport-streamable-http-server-session", "server-side-http", "transport-worker"]`
- `server-side-http` pulls `http`, `http-body`, `http-body-util`, `bytes`, `sse-stream`, `tower` (= `tower-service`), `uuid`, `rand`, `base64`.
- **No axum dependency.** The README's transport table lists `transport-streamable-http-server` — *"Streamable HTTP server transport"* — and names `StreamableHttpService` as the server half (`rmcp-3.4.0/README.md:39,60`).

Montagent today: `rmcp = { version = "3.4", features = ["server", "macros", "schemars", "transport-io"] }` (`crates/montagent/Cargo.toml:20`). Adding the one flag leaves stdio untouched.

### 2.2 How it mounts [verified]

`StreamableHttpService<S, M>` implements `tower_service::Service<http::Request<B>>` (`src/transport/streamable_http_server/tower.rs:1075-1100`). Constructor:

```rust
StreamableHttpService::new(
    || Ok(Montagent::new()),            // service factory, called per session / per stateless request
    LocalSessionManager::default().into(),
    StreamableHttpServerConfig::default(),
)
```

(`tower.rs:1134-1150`.) Upstream's `examples/servers/src/counter_streamhttp.rs` mounts it with `axum::Router::new().nest_service("/mcp", service)` and `axum::serve(...)` with graceful shutdown via the config's `CancellationToken`. `examples/servers/src/simple_auth_streamhttp.rs` shows bearer-token auth as an ordinary axum `middleware::from_fn` checking `Authorization: Bearer …` — rmcp itself has no server-side auth; its `auth` feature is client-side OAuth.

Fit with Montagent [verified + inference]: `Montagent::new()` holds only a `ToolRouter` (`crates/montagent/src/mcp.rs:384-394`) and the encode slot is a process-wide `static ENCODE: Semaphore` (`crates/montagent/src/mcp/dispatch.rs:66`), so a factory that builds a fresh `Montagent` per session/request keeps **one encode at a time across all remote clients** without change. Every tool already runs in `spawn_blocking` (`dispatch.rs:14,204`), so the `current_thread` runtime `serve()` builds (`mcp.rs:1098-1102`) stays responsive under HTTP too **[inference]**.

HTTP request parts (headers, axum extensions) reach tool handlers via `Extension<http::request::Parts>` (`tower.rs:984-1040` doc comment) — the hook for a per-tenant token if auth ever needs to be seen inside a tool.

### 2.3 Config and defaults [verified]

`StreamableHttpServerConfig` (`tower.rs:78-178`, defaults `tower.rs:187-201`):

| Field | Default | Meaning for Montagent |
|---|---|---|
| `sse_keep_alive` | 15 s | SSE comment pings during a silent call. |
| `sse_retry` | 3 s | Priming `retry:` for client reconnects. |
| `legacy_session_mode` | `true` | Sessions (`Mcp-Session-Id`) for pre-`2026-07-28` clients. *"Per SEP-2567, sessions are removed from the `2026-07-28` version, so requests negotiating that version are always served statelessly regardless of this setting."* |
| `json_response` | `false` | Stateless only: answer plain JSON when the handler emits nothing before its result; falls back to SSE if it does. |
| `allowed_hosts` | `localhost`, `127.0.0.1`, `::1` | **`Host` validation against DNS rebinding. A public deployment must set its hostname here or every request is refused.** |
| `allowed_origins` | empty (validation off) | Spec says servers MUST validate `Origin`; `enforce_origin_validation()` turns it on. |
| `session_store` | `None` | Optional external store so a session survives an instance restart / load balancer hop. |
| `max_request_body_bytes` | 4 MiB (`tower.rs:55`) | `413` above it. Only bounds what the client sends. |
| `stateless_protocol_metadata_required` | `false` | Strict `2026-07-28` mode; would reject older clients. Leave off. |

The `SSE` response sets `X-Accel-Buffering: no` (`src/transport/common/server_side_http.rs:176`), which the 2026-07-28 spec says servers SHOULD, for nginx-style proxies.

### 2.4 Stateful vs stateless [verified]

Doc comment on the service (`tower.rs:964-990`): with `legacy_session_mode = true` *"the server creates a session for each client that sends an `initialize` request"*; `LocalSessionManager` is in-memory, `NeverSessionManager` *"disables sessions entirely (stateless mode)"*.

`LocalSessionManager`'s `SessionConfig` (`session/local.rs:1221-1268`): `keep_alive` **5 min** of inactivity closes the session; `completed_cache_ttl` 60 s; `init_timeout` 60 s; `channel_capacity` 16. The idle timer is re-armed on *every* session event, including messages from the handler (`session/local.rs:1107-1127`), so a running `render` that emits progress keeps its session alive **[verified]**; a client that goes quiet for 5 min between calls loses its session and must re-`initialize` **[verified]**.

| | Session mode (default, `<2026-07-28` clients) | Stateless (`2026-07-28` clients, or `legacy_session_mode = false`) |
|---|---|---|
| Per-client state on the server | yes, in memory, lost on restart unless `session_store` | none |
| Client disconnect mid-call | POST stream can be resumed with `Last-Event-ID` via GET (`session/local.rs:654-690`, `966-990`); handler keeps running | **closing the stream cancels the handler** (`CancelOnDisconnect`, `tower.rs:1218-1242`; comment cites rmcp #857) — unless an `EventStore` is configured |
| Horizontal scaling | sticky sessions or a `session_store` | trivial |
| Matches 2026-07-28 spec | n/a (legacy era) | yes: *"Closing the SSE response stream MUST be treated by the server as cancellation of that request"*; *"Resumable SSE streams via `Last-Event-ID` are not supported."* ([spec, Streamable HTTP](https://modelcontextprotocol.io/specification/2026-07-28/basic/transports/streamable-http)) |

Consequence [inference]: under stateless serving, **a proxy or client that drops a long `render`'s connection cancels the render** (ADR-0109: a cancelled encode stops within one frame and publishes nothing). That is spec-correct, but it makes every idle/read timeout on the path a render-killer.

### 2.5 Streaming, progress, tasks [verified]

- Spec: on an SSE response *"the server MAY send JSON-RPC notifications — for example `notifications/progress` … before the final response"* (2026-07-28 Streamable HTTP). Progress requires a `progressToken` in the request's `_meta` ([2025-11-25 progress](https://modelcontextprotocol.io/specification/2025-11-25/basic/utilities/progress)).
- Montagent already sends `notifications/progress` plus a 30 s heartbeat when — and only when — `tools/call` carries `_meta.progressToken` (ADR-0108 §2; `mcp.rs:304`). Over HTTP these ride the request's SSE stream unchanged.
- rmcp 3.4 also ships a server-side **Tasks** runtime (`src/task_manager.rs`: *"Server-side runtime for the MCP Tasks extension (SEP-2663)"* — `tools/call` returns a task, client polls `tasks/get`, cooperative `tasks/cancel`, TTL default 5 min). This is the protocol's answer to "a call longer than the client will wait" — but only if the client speaks it (§5).

### 2.6 Result size [verified]

The server imposes no cap on response size; the only size limit in `streamable_http_server` is `max_request_body_bytes`. rmcp's *client* caps an SSE event at 16 MiB (`src/transport/common/client_side_sse.rs:18`) — irrelevant to us as a server, but a sign other clients may cap too. Practical limits are client-side: Claude Code warns at 10k tokens, caps at 25k (`MAX_MCP_OUTPUT_TOKENS`), and spills text over 50k chars to a file ([Claude Code MCP docs](https://code.claude.com/docs/en/mcp)). `frame`/`preview` return base64 images today (`mcp.rs:633-634`); n8n's `MCP Client` node converts images/audio to binary by default (`convertToBinary`, `McpClient.node.ts`). The rendered MP4 itself never travels as a tool result — which is the "no shared disk" tension already listed on #656.

---

## 3. Clients

### 3.1 n8n [verified]

Source: `packages/@n8n/nodes-langchain/nodes/mcp/` at `e6bb9e0f`.

- **Transport.** `MCP Client Tool` (the AI-Agent tool) has versions `[1, 1.1, 1.2, 1.3, 1.4]`; v1 is SSE-only (`sseEndpoint`), v1.1 offers a transport selector defaulting to `sse`, **v1.2+ defaults to `httpStreamable`** (`McpClientTool.node.ts:61,88-126`). The `MCP Client` step node also offers HTTP Streamable. Both use the official TS SDK's `StreamableHTTPClientTransport`, with an SSE fallback path (`shared/utils.ts:215-280`). Known rough edge: [n8n#35303](https://github.com/n8n-io/n8n/issues/35303) (closed) — a Cloud instance showed the old SSE-only Tool node beside a Streamable-capable step node; older workflows pinned to v1 stay SSE-only.
- **Headers.** Auth options: Bearer, Header Auth, Multiple Headers Auth, MCP OAuth2, None (`McpClientTool.node.ts`, auth block; [docs](https://docs.n8n.io/integrations/builtin/core-nodes/n8n-nodes-langchain.mcpclient/)). Headers are injected by a fetch wrapper on every request (`shared/utils.ts:311-330`).
- **Timeout.** `options.timeout`, **default 60000 ms**, *"Time in ms to wait for tool calls to finish"* (`McpClient.node.ts` options; `McpClientTool.node.ts:27,264-273`). It is passed to `client.callTool(..., { timeout, signal })` (`McpClientTool/utils.ts:82-85`) with **no `onprogress` and no `resetTimeoutOnProgress`**. In TS SDK 1.26.0, a `progressToken` is only attached when `onprogress` is set, and `resetTimeoutOnProgress` defaults to `false` (`dist/esm/shared/protocol.js:8,177,635,706`). So: **n8n sends no `progressToken`, Montagent sends no progress, and the call fails at 60 s unless the workflow author raises the timeout.** It is a hard wall clock, not an idle window.
- **Protocol era.** TS SDK 1.26.0: `LATEST_PROTOCOL_VERSION = '2025-11-25'` (`dist/esm/types.js:2`) — an `initialize`-handshake client, so rmcp serves it in session mode by default.

### 3.2 Claude Code [verified unless marked]

From [code.claude.com/docs/en/mcp](https://code.claude.com/docs/en/mcp):

- **Transport.** `claude mcp add --transport http <name> <url>`. *"The SSE transport is deprecated. Use HTTP servers instead"*; since 2.1.265 an `http` entry falls back to SSE automatically.
- **Headers.** `--header "Authorization: Bearer your-token"`; or `headersHelper` (a command printing JSON headers, re-run on connect and on `401`/`403`); OAuth 2.0 via `/mcp` or `claude mcp login`.
- **Timeouts.** Idle window **5 min for HTTP** (30 min for stdio), override `CLAUDE_CODE_MCP_TOOL_IDLE_TIMEOUT`; *"no response and no progress notification"* defines idle, so progress resets it. Hard wall clock `MCP_TOOL_TIMEOUT` (~28 h default), which progress does **not** extend. A per-server `timeout` ≥ 1000 ms is also a floor on the idle window. A per-request first-byte timer of ≥ 60 s applies to HTTP servers — rmcp's SSE headers and 15 s pings satisfy it [inference].
- **Progress token.** Still not documented whether Claude Code sends `_meta.progressToken`; its changelog implies it does (see [mcp-render-timeout.md §3.2](mcp-render-timeout.md)). ADR-0108's dispatch log line (`progressToken: present|absent`) settles it on the first remote render [inference — not yet observed].

Net: Claude Code survives a multi-minute render over HTTP **if** it sends a `progressToken` (ADR-0108 heartbeat every 30 s ≪ 5 min); otherwise the user must set a per-server `timeout`.

### 3.3 Hermes agent (Nous Research) [verified]

Source: `NousResearch/hermes-agent` at `93c9360a`; docs `website/docs/reference/mcp-config-reference.md`.

- **Transport.** A `url` entry is Streamable HTTP by default; `transport: sse` opts into legacy SSE. Python `mcp==2.0.0` (`pyproject.toml`). The client negotiates the modern era first and falls back to a legacy `initialize` once (`tools/mcp_tool_transport.py:181-210`).
- **Headers.** `headers:` mapping, with `${VAR}` / `${env:VAR}` substitution from the profile's `.env`; example `Authorization: "Bearer ***"`. OAuth (auth-code, device-code, CIMD) and mTLS (`client_cert`) also supported.
- **Timeout.** `timeout` — *"Tool call timeout in seconds (default: `300`)"*; `connect_timeout` 60 s. `_DEFAULT_TOOL_TIMEOUT = 300` (`tools/mcp_tool_common.py:41`). It is a hard wall clock around `session.call_tool(...)` (`tools/mcp_tool_handlers.py:325-335,394`), and the call passes no progress callback, so no `progressToken` and no reset. The HTTP read timeout is `httpx.Timeout(connect, read=300)` (`mcp_tool_transport.py:571`) — a between-bytes limit that rmcp's 15 s pings satisfy [inference].
- **Session expiry.** The docs warn: on *"a Streamable-HTTP server that expires idle sessions … the first unannotated call after an idle period may fail"* with `outcome_uncertain`; only tools annotated `readOnlyHint: true` are replayed transparently (`trust` row). rmcp's 5-min session `keep_alive` is exactly such an expiry.

---

## 4. Summary table

| Client | Streamable HTTP | Bearer / custom header | Default tool-call limit | Progress-aware? | Multi-minute `render` out of the box? |
|---|---|---|---|---|---|
| n8n `MCP Client Tool` v1.2+ / `MCP Client` | yes (default) | yes (Bearer, Header, Multiple Headers, OAuth2) | **60 s hard** | no (no token sent) | **no** — raise `Timeout` |
| n8n `MCP Client Tool` v1 | **SSE only** | yes | 60 s | no | no |
| Claude Code ≥ 2.1.265 | yes | yes (`--header`, `headersHelper`, OAuth) | 5 min idle; ~28 h hard | yes (resets idle) | yes if it sends `progressToken` [inference]; else set per-server `timeout` |
| Hermes agent | yes (default) | yes (`headers`, env-substituted) | **300 s hard** | no | **no** beyond 5 min — raise `timeout` |

---

## 5. Things that force a decision

1. **Long renders vs. client timeouts (blocks "Long renders over HTTP" on #656).** Two of three clients use a hard wall clock that progress cannot extend (n8n 60 s, Hermes 300 s). Options: (a) document "raise the client timeout" per client; (b) an async shape — a `render` that returns at once with a job handle plus a status tool, which works for every client; (c) MCP Tasks (rmcp 3.4 has a server runtime), which only helps clients that implement Tasks — none of the three is shown to here [inference: not checked in their source]. (b) adds tools to the surface ADR-0011 keeps small; (a) leaves n8n users failing at 60 s by default.
2. **Stateful vs. stateless.** Default session mode gives `Last-Event-ID` resumption for legacy clients but 5-min idle expiry (Hermes → `outcome_uncertain` on `render`) and in-memory state (no restarts, sticky routing). Stateless matches 2026-07-28 and scales trivially, but a dropped connection cancels the render (spec-mandated). Either way: set `SessionConfig.keep_alive` deliberately, and consider `readOnlyHint: true` annotations on the read-only tools so Hermes can replay them.
3. **`allowed_hosts` / `Origin`.** Defaults accept only loopback `Host`s. A host-neutral image needs the public hostname as configuration (env var/flag), and Origin validation should be enabled — the spec makes it a MUST.
4. **Auth is Montagent's code.** rmcp gives no server-side auth; a bearer check is a tower/axum middleware. All three clients can send a static `Authorization: Bearer`; all three also do OAuth, so "token now, OAuth later" is open — ties to the tenancy decision on #656.
5. **Dependency.** axum (or hyper directly — upstream also has `counter_hyper_streamable_http.rs`) enters the dependency tree, behind a feature or a `montagent serve --http` subcommand so stdio builds stay lean.
6. **No legacy HTTP+SSE needed.** Only n8n's `MCP Client Tool` v1 requires it; v1.2+ is the default. rmcp 3.4 has no legacy-SSE server anyway. Recommend documenting "use v1.2+" rather than building SSE.

---

## 6. Open questions (not settled here)

- Whether Claude Code sends `_meta.progressToken` on `tools/call` — ADR-0108's log line answers it on the first remote render.
- Whether any of the three clients implements MCP Tasks (SEP-2663) as a requester.
- n8n's own execution timeout (`EXECUTIONS_TIMEOUT`) and any reverse-proxy idle timeout on the host (e.g. a cloud load balancer's default) sit outside these clients and can still cut a long SSE stream; rmcp's 15 s pings should keep idle-based proxies open [inference].
