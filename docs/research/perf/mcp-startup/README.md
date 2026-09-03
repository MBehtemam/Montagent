# Measured: MCP server cold start across TypeScript, Python and Rust

Prototype for [#16](https://github.com/MBehtemam/Montaget/issues/16), part of the map
[#2](https://github.com/MBehtemam/Montaget/issues/2).

[#15](https://github.com/MBehtemam/Montaget/issues/15) named startup as the axis expected to
decide [#7](https://github.com/MBehtemam/Montaget/issues/7) and found that **no absolute
cold-start figure is published for any of the three runtimes**. Same shape as the renderer
survey's unfillable throughput column, same remedy: measure it.

**This is a throwaway.** Nothing here is a proposed design. `element-schema.json` is a
*stand-in* built to be realistically large — it is not a candidate schema for the element
format and should not be read as one.

## The spec question comes first

#16 asked to confirm the process model before measuring, because a per-session process
would shrink the axis on its own. It does. From the
[stdio binding](https://modelcontextprotocol.io/specification/2026-07-28/basic/transports/stdio):

> In the **stdio** transport, the client launches the MCP server as a subprocess.

One subprocess, serving every request over its standard streams for as long as the client
keeps it, shut down by closing its stdin:

> The client **SHOULD** initiate shutdown by: 1. Closing the input stream to the child
> process (the server). […] Servers **SHOULD** exit promptly when their standard input is
> closed or reads return end-of-file.

So **startup is paid once per server process, not per tool call and not per preview.** Every
number below is a one-time cost per agent session.

Two further facts fell out of reading the spec, both of which changed how this was measured:

- **The current revision has no `initialize` handshake.** `2026-07-28` carries version,
  identity and capabilities as per-request `_meta`; "There is no negotiation handshake."
  The handshake #16 asked about is the *legacy* shape (`2025-11-25` and earlier). The
  modern equivalent of "start through a handshake" is `server/discover` then `tools/list`.
- **`server/discover` does not carry tool schemas** — it returns supported versions,
  capabilities and identity. `tools/list` stays separate, so the schema cost still lands on
  a second round trip either way.

## Method

Three minimal stdio servers, one per host, each using that language's first-party SDK, each
advertising **one** tool (`add_element`, the shape the map's tool-surface invariant permits)
whose `inputSchema` is **byte-identical across all three** — confirmed at 41,391 bytes
minified in every run.

`measure.py` drives each one exactly as a client does: timers start immediately *before*
`spawn`, and each request is written only after the previous response is read. No
pipelining. Two intermediate times are captured per run — spawn→handshake complete, and
spawn→`tools/list` complete.

Each cell is **30 iterations**; medians reported, with p10/p90 spread and the first run
called out separately in `results.json`.

Four axes were crossed:

- **host** — TypeScript, Python, Rust.
- **revision** — legacy `2025-11-25` (the newest all three implement) and modern
  `2026-07-28`.
- **schema** — the 41 KB element schema vs a trivial 181-byte one, to separate schema cost
  from startup cost.
- **stage** — bare runtime, runtime with the SDK imported, and the full protocol exchange,
  to attribute where the time goes.

## Machine

Apple **M1 Pro**, 10 cores, 16 GB, macOS **26.5.2**. Node **v22.23.2**, Python **3.14.6**,
rustc **1.95.0**. SDKs: `@modelcontextprotocol/sdk` **1.30.0**, `mcp` **2.1.1**, `rmcp`
**3.2.0** (release build, `lto = true`). Same machine as the
[FFmpeg baseline](../ffmpeg-baseline/README.md) and the
[renderer candidates](../candidates/README.md). Wall-clock, not published benchmarks.

## Results

### Spawn → `tools/list` complete — the whole cost an agent pays, once per session

Legacy revision, so all three hosts are comparable. Medians of 30.

| Host | Handshake | **+ `tools/list`** | p10 – p90 | vs Rust |
| --- | --- | --- | --- | --- |
| **Rust** (`rmcp`) | 4.3 ms | **4.8 ms** | 4.6 – 5.3 | — |
| **TypeScript** (`@modelcontextprotocol/sdk`) | 101.6 ms | **103.3 ms** | 102.2 – 105.0 | 22× |
| **Python** (`mcp`) | 419.6 ms | **421.6 ms** | 417.2 – 427.0 | 88× |

**Full spread: 417 ms.** The ratios are large; the absolute numbers are not. Spread within
each host is tight — p90 is within 5% of the median everywhere — so these are not
single-run artefacts.

### Where the time goes

| Stage | TypeScript | Python | Rust |
| --- | --- | --- | --- |
| Bare runtime, no SDK | 29.5 ms | 21.1 ms | 2.8 ms |
| Runtime + SDK imported | 93.9 ms | 410.1 ms | — |
| Full exchange (above) | 103.3 ms | 421.6 ms | 4.8 ms |
| *(runtime teardown, at session end)* | *6.4 ms* | *71.3 ms* | *0.2 ms* |

Every row is timed to the same point — the moment the process emits its first output — so
the stages compose. Teardown is listed for completeness: an stdio server pays it once when
the client closes its stdin, not at startup.

Startup *is* the cost. The handshake and `tools/list` together add **1–2 ms** in every
host — the protocol work is free, and what separates the three is entirely getting the
runtime and its SDK loaded.

The bare runtimes are within 28 ms of each other. **The spread is the SDK import, not the
language**: ~73 ms for TypeScript, ~500 ms for Python.

### The 41 KB schema is free

| Host | 181-byte schema | 41 KB schema | Delta |
| --- | --- | --- | --- |
| Rust | 3.7 ms | 4.8 ms | +1.1 ms |
| TypeScript | 102.0 ms | 103.3 ms | +1.3 ms |
| Python | 421.2 ms | 421.6 ms | +0.4 ms |

#16 asked for "a realistically large nested element schema" on the assumption its size might
matter. It does not — a 228× larger schema costs about a millisecond. **Schema size is not
a startup consideration in any host**, which also means the element format can be as
explicit as it needs to be without a startup penalty.

### Python's cost is a packaging accident, not Python

`python -X importtime` on the server, cumulative (the instrumented run is slightly slower
than the timed one, so read these as proportions):

| Module | Cumulative |
| --- | --- |
| `mcp` → `mcp.server` → `mcp.server.stdio` (total) | 448.7 ms |
| ⤷ of which `mcp.client._input_required` → `mcp.client` | **195.8 ms** |
| ⤷ of which `mcp.types` / `mcp_types` | 126.5 ms |
| ⤷ of which `sse_starlette.sse` → `starlette.*` | 47.8 ms |

Two of those a stdio server has no use for. **~196 ms — roughly 44% of Python's total — is
importing `mcp.client`**, which `mcp.server` reaches through `mcp.client._memory`; and a
further ~48 ms is Starlette, pulled in for the HTTP transport this server never opens.
Both are upstream packaging accidents, fixable upstream and not by Montaget — so Python's
figure is the least intrinsic of the three, but it is also not something to plan around
today.

### Modern revision: TypeScript cannot speak it

| Host | `2026-07-28` (`server/discover` + `tools/list`) |
| --- | --- |
| Rust | 4.9 ms |
| Python | 422.6 ms |
| TypeScript | **`-32601` Method not found** |

`@modelcontextprotocol/sdk` **1.30.0** — `latest` on npm, no prerelease tag — declares
`LATEST_PROTOCOL_VERSION = '2025-11-25'` and
`SUPPORTED_PROTOCOL_VERSIONS = ['2025-11-25', '2025-06-18', '2025-03-26', '2024-11-05',
'2024-10-07']`. It has no `server/discover`. `mcp` 2.1.1 reports
`LATEST_PROTOCOL_VERSION = '2026-07-28'` and ships `DiscoverRequest`/`DiscoverResult`;
`rmcp` 3.2.0 has `ProtocolVersion::V_2026_07_28`. Both answer the modern exchange, and both
are **dual-era** — they answered `initialize` too.

Dropping the handshake buys nothing measurable (Python 421.6 → 422.6 ms, Rust unchanged):
it removes a notification, not a round trip, since `tools/list` is still separate.

## What this settles

**The spread is noise, not a constraint.** 417 ms separates the fastest host from the
slowest, paid **once per session**, against a preview budget of 5 s that
[#6](https://github.com/MBehtemam/Montaget/issues/6) measured skia-canvas hitting in
**2.06–2.18 s** — roughly 2.8 s of headroom on *every* preview. The worst host spends 15%
of one preview's budget, once, and then nothing for the rest of the session. Even the 88×
Rust-vs-Python ratio cannot be spent on anything: there is no loop to amortise it over,
because the process does not restart.

Set against the renderer's own **0.93 s** cold start for a single FFmpeg frame, the entire
host spread is under half of what the renderer pays *per frame*.

So, in #16's own terms: **it is noise, and this axis stops carrying weight in
[#7](https://github.com/MBehtemam/Montaget/issues/7).** The host decision falls to the
schema story and distribution.

## Two observations for #7, not resolved here

Both are #7's to weigh; recorded because this prototype ran into them directly.

1. **TypeScript is now a protocol revision behind.** #15 concluded all three SDKs were "on
   the current `2026-07-28` spec revision"; that is no longer true of TypeScript, measured
   above. #15's heaviest axis — the schema story — pointed *at* TypeScript, so #7 should
   not read that conclusion without re-checking it.
2. **Python's low-level `Server` does advertise an exact schema.** #15 concluded "Python's
   `@mcp.tool()` cannot advertise an exact schema at all." That holds for the FastMCP
   decorator, but `mcp.server.lowlevel.Server` takes `types.Tool(inputSchema=...)` and
   advertised the 41 KB schema **byte-identically** to the other two hosts. Whether it also
   *validates* against it was not tested and is the question that actually matters.

## Files

| File | |
| --- | --- |
| `element-schema.json` | the shared 41 KB stand-in schema, byte-identical in all three servers |
| `measure.py` | the harness — spawns, drives the exchange, summarises |
| `results.json` | full output: medians, p10/p90, min/max, first run, per cell |
| `ts/`, `py/`, `rs/` | the three minimal servers, plus `bare` / `import-only` stages |

Reproduce with `python3 measure.py -n 30` after `npm install` in `ts/`, `pip install mcp`
into `.venv`, and `cargo build --release` in `rs/`.
