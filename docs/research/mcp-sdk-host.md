# Survey: MCP SDK host language — TypeScript, Python or Rust

Research for [#15](https://github.com/MBehtemam/Montaget/issues/15), part of the map [#2](https://github.com/MBehtemam/Montaget/issues/2).

**Date of research:** 2026-09-03. Version numbers, release dates, download counts and issue counts are as of that date.

**This document does not pick a winner.** [#7 "Choose the renderer and host language"](https://github.com/MBehtemam/Montaget/issues/7) makes that call. A prior direction on #7 leaned Rust and was recorded as provisional precisely so this ticket could overturn it; this survey was written without writing toward it, and the findings below do not support it as cleanly as that lean assumed.

**Sourcing rule applied:** every claim cites a primary source (the SDK repository itself, its releases and tags, the MCP specification, or a registry API — npm, PyPI, crates.io). Where a figure is not published, this document says **not found** rather than estimating. Where a fact could only be reached through a search-engine summary because the primary URL failed, it is marked **[secondary]**, as in the [renderer survey](renderer-survey.md).

> [!WARNING]
> **Scope correction — H7's conditional has resolved.**
>
> This survey reports text shaping (H7) as Latin-only and full-bidi *separately*, and concludes that Rust's bidi lead is "only load-bearing if the channel goes RTL or CJK" — leaving a large part of Rust's case contingent on a fact about one YouTube channel.
>
> [ADR-0003](../adr/0003-general-video-editor-not-channel-tooling.md) settles that Montaget is a **general, open-source video editor** whose first consumer merely happens to be that channel. See [#19](https://github.com/MBehtemam/Montaget/issues/19). The conditional therefore resolves: **full bidi is load-bearing, unconditionally.** Strangers will feed Montaget Arabic, Hebrew, Persian and Chinese regardless of what that channel publishes.
>
> How that changes the reading:
>
> - **The "H7 text — Latin only" row should be disregarded as a scoring row.** It was a tie worth nothing under either reading; only the full-bidi row scores.
> - **Rust's case is stronger than the conclusion states.** It rests on H8 (distribution), H9 (unscored by anyone) *and* H7 — and H7 is now unconditional rather than contingent.
> - **Python's named blocker is on the critical path.** No `TextDirection` binding in `skia-python` is still a well-defined upstream PR rather than a capability gap, but it can no longer be discounted as hypothetical.
> - **`skia-canvas`'s undocumented bidi behaviour must be established empirically** if Node stays a candidate. The ~30-minute test this survey describes as conditional is now required.
>
> The **facts** are unaffected; the **weighting** is not. [#7](https://github.com/MBehtemam/Montaget/issues/7) owns the re-score.

---

## 1. What is being scored

Montaget is an MCP server before it is a renderer. Per [`CONTEXT.md`](../../CONTEXT.md) and the map, Montaget contains no model; its entire external surface is the MCP tools an agent calls. So the criteria below weight the MCP layer, the schema story and the packaging story far above per-frame rasterizer cost — all three languages bind the *same* C++ Skia, and [#6](https://github.com/MBehtemam/Montaget/issues/6) already established that the incumbent pipeline clears the throughput budget ~10x while the preview loop is **startup-dominated**.

| # | Criterion |
|---|---|
| **H1** | **First-party SDK** — is it under the `modelcontextprotocol` org, or community-maintained? |
| **H2** | **Maturity** — version, release cadence, last commit, open-issue posture, absence of the "Revideo pattern" (a stated pivot plus stalling commits). |
| **H3** | **Spec coverage** — transports, tools/resources/prompts, and which MCP revision it targets. |
| **H4** | **Schema story** *(weighted heaviest)* — how tool input schemas are declared and emitted, and how faithfully a large nested element object survives the trip. |
| **H5** | **Ergonomics against our shape** — what one tool taking a whole element object actually costs to declare. |
| **H6** | **Skia binding health** — prebuilt binaries for macOS arm64 and linux x64, or a from-source Skia build. |
| **H7** | **Text shaping** — Latin-only and full-bidi reported *separately*. |
| **H8** | **Distribution** — how Montaget gets installed and run. |
| **H9** | **Startup cost** — the axis #6 says is most likely to separate them in practice. |

The load-bearing shape throughout is the map's settled write-tool invariant: **a tool that writes takes a complete element as a schema-shaped object, never a field name and never an element id**, and the format is discoverable through its *published JSON Schema*. So the question under H4 is narrower than "does this SDK do schemas": it is **can the one published JSON Schema document be both the file-validation artifact and the tool's advertised `inputSchema`, without being retyped in a second language?**

---

## 2. Scorecard

Legend: **Y** meets it, **~** partial or with caveats, **N** does not.

| Axis | TypeScript | Python | Rust |
|---|---|---|---|
| **H1 first-party SDK** | **Y** `modelcontextprotocol/typescript-sdk` | **Y** `modelcontextprotocol/python-sdk` | **Y** `modelcontextprotocol/rust-sdk` (crate `rmcp`; **not** `rust-mcp-sdk`, **not** `mcp-core`) |
| **H2 maturity** | **~** 2.0.0, active (106 commits/3mo), but **305 open issues + 299 open PRs** and a standing "1 PR per new contributor" freeze | **Y** 2.1.1, most active (176 commits/3mo), most-starred (24.2k), v1 line still patched | **Y** 3.2.0, most commits (192/3mo), smallest backlog (23 issues / 34 PRs), smallest community (3.9k stars) |
| **H3 spec coverage** | **Y** 2026-07-28; stdio + Streamable HTTP; legacy SSE only in `server-legacy` | **Y** 2026-07-28; the only one still shipping legacy HTTP+SSE server-side | **Y** 2026-07-28; **N** on legacy HTTP+SSE, called "a deliberate non-goal" |
| **H4 schema story** *(weighted heaviest)* | **Y** — `fromJsonSchema()` advertises the published document **unchanged** and validates it with AJV, at the high level. Caveat: Zod's default `reused: "inline"` bloats a nested union if you go the Zod route instead | **~** — best-in-class derivation from type hints, but **no schema override on `@mcp.tool()`**; exact schemas require the low-level `Server`, where "`input_schema` is *advertised*… never *applied*". Emits per-property `title` noise and an OpenAPI `discriminator` keyword | **~** — cleanest emitted document (2020-12, `$defs`/`$ref`, top-level noise stripped, no OpenAPI-isms, schema cached per type). `.parameters_value()` can advertise a raw document, but nothing validates against it |
| **H5 ergonomics** | **Y** — one `registerTool` call; either derived TS types **or** the published schema verbatim, not both | **Y** — shortest code of the three; a decorator and a type hint | **~** — every element variant needs `Deserialize + JsonSchema` derives and doc comments; most ceremony, best output, and an error model (`CallToolResult::error` vs `Err(McpError)`) that matches "schema catches malformed, agent catches wrong" |
| **H6 Skia prebuilts (macOS arm64 + linux x64)** | **~** both covered, but via a **postinstall script hitting GitHub releases**, not npm platform packages; needs github.com egress and pnpm build approval. **Binding is the most stalled artifact in this survey** (no npm release in ~11 months, Skia m140) | **Y** — 45 real PyPI wheels, cp38–3.14 incl. free-threaded; `pip install` is a plain binary fetch. No sdist, so an uncovered platform fails outright | **~** both covered, but fetched by `build.rs` from a **separate repo**, matched by **feature-set hash**; an off-menu combo silently falls into an LLVM/Python/Ninja Skia build. crates.io static since 2026-06-19 while master runs 3 milestones ahead |
| **H7 text — Latin only** | **Y** `ctx.textWrap` + `fillText(…, width)` | **Y** `textlayout_ParagraphBuilder` | **Y** `textlayout::ParagraphBuilder` (feature `textlayout`, prebuilt, ICU embedded) |
| **H7 text — full bidi + fallback + shaping** | **~** base direction wired to `TextDirection` **in source only**; mixed-direction UBA reordering **not found** in any project doc. Fallback documented via `.runs` | **N as exposed** — **no `TextDirection` binding at all**; ICU/HarfBuzz are in the wheels but the knob is unbound. Needs an upstream PR or a fork | **Y** — HarfBuzz + ICU through `textlayout`; or `cosmic-text` (unicode-bidi + harfrust + per-character fallback on the three desktop OSes) at the cost of writing the Skia draw bridge |
| **H8 distribution** | **~** npm/`npx` is frictionless; SEA is `Stability: 1.1` and **untested on macOS x64** | **~** `uvx` can supply the interpreter itself; `pipx` deliberately will not | **Y** — static musl (Tier 2 w/ host tools), and the MCP docs' own Rust example is a bare path to a binary with **no args** |
| **H9 startup cost** | **Not published** (only qualitative snapshot claims) | **Not published** (only "10-15% faster" 3.11 vs 3.10) | **Not published** (structural only) |

**The single most important finding:** *the axis expected to decide this — startup cost — publishes no absolute figure for any of the three runtimes.* Exactly as with the renderer survey's throughput column, H9 cannot be filled from primary sources and must be measured before [#7](https://github.com/MBehtemam/Montaget/issues/7) rules on it. Python's "10–15% faster" is relative to Python 3.10 and composes with nothing; V8's headline snapshot numbers measure context creation, not process startup; Rust's advantage is structural and therefore unpublished because it would be meaningless to publish.

**The second:** *the heaviest-weighted axis, H4, points at TypeScript, not Rust.* Montaget's settled principle is that the format is discoverable through its **published JSON Schema** and that a write tool takes a complete schema-shaped element. Only the TypeScript SDK offers a first-party path — `fromJsonSchema()` — that advertises that exact document **and** validates against it, from one artifact, at the high level. Rust can advertise it but not enforce it; Python can enforce a schema or advertise an exact one, but not both.

**The third, and the one that most cuts against a Rust lean:** *of the cross-cutting axes that are decidable from published sources at all, Rust wins one outright (distribution), ties one (Latin text), leads a contested one (full bidi), and comes **third** on the CI story.* `skia-python` ships real wheels; `skia-safe` ships a build script that fetches feature-hash-keyed tarballs from a second repository and quietly compiles Skia when it misses.

---

## 3. The three SDKs

### 3.0 All three are first-party (H1)

All three candidates are published under the `modelcontextprotocol` GitHub organisation. This was verified per repository rather than assumed, and there is no ambiguity in any of the three:

| | Repository | Description string from the GitHub API |
|---|---|---|
| TypeScript | [`modelcontextprotocol/typescript-sdk`](https://github.com/modelcontextprotocol/typescript-sdk) | "The official TypeScript SDK for Model Context Protocol servers and clients" |
| Python | [`modelcontextprotocol/python-sdk`](https://github.com/modelcontextprotocol/python-sdk) | "The official Python SDK for Model Context Protocol servers and clients" |
| Rust | [`modelcontextprotocol/rust-sdk`](https://github.com/modelcontextprotocol/rust-sdk) | "The official Rust SDK for the Model Context Protocol" |

Source: `GET /repos/{owner}/{repo}` on the GitHub API for each, and the org's repository listing (`GET /orgs/modelcontextprotocol/repos`), which also carries first-party Go, Java, Kotlin, C#, Ruby, PHP and Swift SDKs. None of the three is archived.

**Which Rust crate is current — the question the ticket asks explicitly.** The current crate is **`rmcp`**, published from `modelcontextprotocol/rust-sdk` ([crates.io](https://crates.io/api/v1/crates/rmcp) reports `repository: https://github.com/modelcontextprotocol/rust-sdk/`), together with its companion proc-macro crate `rmcp-macros`. Two other crates could plausibly be mistaken for it and are **not** the official SDK:

- **`rust-mcp-sdk`** — 2.0.0, updated 2026-08-27, 254,669 all-time downloads, repo [`rust-mcp-stack/rust-mcp-sdk`](https://github.com/rust-mcp-stack/rust-mcp-sdk). Community-maintained, actively developed, built on its own `rust-mcp-schema` crate. A real alternative, but not first-party.
- **`mcp-core`** — 0.1.50, last updated **2025-05-01**, repo [`stevohuncho/mcp-core`](https://github.com/stevohuncho/mcp-core). Dormant for ~16 months. Not a candidate.

`rmcp` dominates on adoption: **23,813,092 all-time downloads and 12,103,107 in the recent window**, against `rust-mcp-sdk`'s 254,669 — roughly 90x ([crates.io API](https://crates.io/api/v1/crates/rmcp)). The Rust crate question resolves cleanly.

### 3.1 Maturity (H2)

Everything below is from the GitHub and registry APIs on 2026-09-03.

| | TypeScript | Python | Rust |
|---|---|---|---|
| **Current release** | `@modelcontextprotocol/server` **2.0.0** (2026-07-27); legacy `@modelcontextprotocol/sdk` **1.30.0** (2026-07-27) | `mcp` **2.1.1** (2026-08-25); v1 line still shipping (`1.29.1`, 2026-08-24) | `rmcp` **3.2.0** (2026-08-31) |
| **Pre-1.0?** | No | No | No |
| **Releases published** | 79 versions of `@modelcontextprotocol/sdk` since 2024-11-11; 10 of `@modelcontextprotocol/server` since 2026-04-01 | 70 releases of `mcp` | 63 versions of `rmcp` since 2025-03-16 |
| **Cadence, last ~6 months** | 1.26.0 (Feb 4) → 1.27.0 (Feb 16) → 1.28.0 (Mar 25) → 1.29.0 (Mar 30) → 1.30.0 (Jul 27), plus the v2 alpha/beta train Apr–Jul | 2.0.0a1 (Jun 11) → … → 2.0.0 (Jul 28) → 2.1.0 (Aug 24) → 2.1.1 (Aug 25) | 2.1.0 (Jul 2) → 2.2.0 (Jul 8) → 3.0.0 (Jul 28) → 3.1.x (Jul 31–Aug 20) → 3.2.0 (Aug 31) |
| **Last commit** | 2026-08-31 | **2026-09-02** | 2026-09-01 |
| **Commits in last 3 months** | 106 | 176 | **192** |
| **Stars** | 13,311 | **24,189** | 3,865 |
| **Open issues / open PRs** | **305 / 299** | 206 / 186 | **23 / 34** |
| **Licence** | In transition: repo `LICENSE` states the MCP project is moving from MIT to **Apache-2.0**; the published npm package still declares MIT | MIT (GitHub API and PyPI metadata) | Same MIT→Apache-2.0 transition text in the repo `LICENSE`; crates.io metadata for 3.2.0 declares **Apache-2.0** |
| **Adoption** | 203,972,638 downloads of `@modelcontextprotocol/sdk` and 14,466,180 of `@modelcontextprotocol/server` in the 30 days to 2026-08-29 ([npm downloads API](https://api.npmjs.org/downloads/point/last-month/@modelcontextprotocol/sdk)) | **not found** — PyPI's own API publishes no download counts, and pypistats.org (a secondary source) returned HTTP 429 on both attempts | 23,813,092 all-time / 12,103,107 recent ([crates.io API](https://crates.io/api/v1/crates/rmcp)) |

**No candidate shows the Revideo pattern.** The renderer survey's signature was *a stated pivot to a commercial product plus stalling commits*. Here all three repositories received commits within 48 hours of this research, all three shipped a major version in the last six weeks, and none is archived or carries a deprecation banner. There is no commercial fork or pivot statement in any of the three READMEs.

**Two real maturity caveats, both about churn rather than decay:**

1. **All three are mid-major-version transition simultaneously**, because all three shipped a rewrite alongside the 2026-07-28 spec. Python ships v1.x and v2.x in parallel and warns that `pip install mcp` now resolves to 2.x, advising a `<2` pin until you migrate ([README](https://github.com/modelcontextprotocol/python-sdk/blob/main/README.md)). TypeScript split one package into a monorepo of `@modelcontextprotocol/{server,client,core,node,express,fastify,hono}` plus `server-legacy` ([README](https://github.com/modelcontextprotocol/typescript-sdk/blob/main/README.md), [`packages/`](https://github.com/modelcontextprotocol/typescript-sdk/tree/main/packages)). Rust went 2.x → 3.x with a [migration guide in a GitHub discussion](https://github.com/modelcontextprotocol/rust-sdk/discussions/969). Starting a project today means starting on a release line that is weeks old in every language.
2. **The TypeScript repo is currently restricting contribution.** Its README carries a standing warning: *"We're limiting pull requests to 1 per new contributor while v2 settles after the 2026-07-28 spec release … Issues are the most useful feedback right now — we'll reopen PRs as v2 stabilizes"* ([README](https://github.com/modelcontextprotocol/typescript-sdk/blob/main/README.md)). Paired with 305 open issues and 299 open PRs — the largest backlog of the three by an order of magnitude against Rust's 23/34 — this is the one place a "the official SDK will fix it" assumption is weakest.

### 3.2 Spec coverage (H3)

**The current spec revision is `2026-07-28`.** The specification repository publishes revisions as directories under [`docs/specification/`](https://github.com/modelcontextprotocol/modelcontextprotocol/tree/main/docs/specification) and [`schema/`](https://github.com/modelcontextprotocol/modelcontextprotocol/tree/main/schema): `2024-11-05`, `2025-03-26`, `2025-06-18`, `2025-11-25`, `2026-07-28`, `draft`. So `2026-07-28` is the newest released revision and `2025-11-25` the one before it.

**All three SDKs implement `2026-07-28`. Spec coverage does not separate them.**

- **Rust** — the README states plainly: *"This SDK implements the stable MCP `2026-07-28` specification while remaining fully compatible with the `2025-11-25` release and earlier versions"*, and enumerates the new features it covers: server discovery & negotiation, transport-neutral subscriptions, long-running tasks, response caching, multi-round-trip requests, standard HTTP routing headers ([README](https://github.com/modelcontextprotocol/rust-sdk/blob/main/README.md)). `ProtocolVersion::KNOWN_VERSIONS` lists all five revisions ([`crates/rmcp/src/model.rs`](https://github.com/modelcontextprotocol/rust-sdk/blob/main/crates/rmcp/src/model.rs)).
- **TypeScript** — the README's first line: *"This is the `main` branch — v2 of the SDK … implementing the [2026-07-28 MCP spec]"*, and the repo contains both `spec.types.2025-11-25.ts` and `spec.types.2026-07-28.ts` plus the 2026-era `_meta` envelope keys in [`packages/core/src/constants.ts`](https://github.com/modelcontextprotocol/typescript-sdk/blob/main/packages/core/src/constants.ts).
- **Python** — the README: *"This is v2 of the MCP Python SDK, the current stable release line … to support the [2026-07-28 MCP specification](https://modelcontextprotocol.io/specification/2026-07-28) (and every earlier revision)"*.

A subtlety worth recording, because it looks like a lag and is not: in all three, the constant named "latest" is **`2025-11-25`**, not `2026-07-28`. That is deliberate. `2026-07-28` introduced a **stateless per-request envelope** reached through a `server/discover` probe rather than through the `initialize` handshake, so the newest *handshake-reachable* revision and the newest revision overall are different things. Python names the split explicitly — `HANDSHAKE_PROTOCOL_VERSIONS` ends at `2025-11-25`, `MODERN_PROTOCOL_VERSIONS` is `("2026-07-28",)`, and `LATEST_PROTOCOL_VERSION` is the newest of *either* ([`src/mcp-types/mcp_types/version.py`](https://github.com/modelcontextprotocol/python-sdk/blob/main/src/mcp-types/mcp_types/version.py)). Rust encodes the same split as `ProtocolVersion::LATEST = V_2025_11_25` alongside `ClientLifecycleMode::{Discover, Auto}` for the newer path ([`model.rs`](https://github.com/modelcontextprotocol/rust-sdk/blob/main/crates/rmcp/src/model.rs), [README](https://github.com/modelcontextprotocol/rust-sdk/blob/main/README.md)). TypeScript sets `LATEST_PROTOCOL_VERSION = '2025-11-25'` with `DEFAULT_NEGOTIATED_PROTOCOL_VERSION = '2025-03-26'` in the same file that defines the 2026-07-28 `_meta` keys.

**Transports.** All three do stdio and Streamable HTTP; they differ only on whether the deprecated two-endpoint HTTP+SSE transport from `2024-11-05` is still shipped.

| Transport | TypeScript | Python | Rust |
|---|---|---|---|
| **stdio** (server) | Y — [`packages/server/src/stdio.ts`](https://github.com/modelcontextprotocol/typescript-sdk/blob/main/packages/server/src/stdio.ts), `packages/server/src/server/serveStdio.ts` | Y — [`src/mcp/server/stdio.py`](https://github.com/modelcontextprotocol/python-sdk/blob/main/src/mcp/server/stdio.py); README: "Speak every standard transport: stdio, Streamable HTTP, and SSE" | Y — feature `transport-io` ([README transports table](https://github.com/modelcontextprotocol/rust-sdk/blob/main/README.md)) |
| **Streamable HTTP** (server) | Y — `packages/server/src/server/streamableHttp.ts`, plus thin framework adapters for Node `http`, Express, Fastify and Hono | Y — [`src/mcp/server/streamable_http.py`](https://github.com/modelcontextprotocol/python-sdk/blob/main/src/mcp/server/streamable_http.py), `streamable_http_manager.py`, `_streamable_http_modern.py` | Y — feature `transport-streamable-http-server`, exposed as a **Tower service** mountable on any axum/hyper router |
| **Legacy HTTP+SSE** (`2024-11-05`) | ~ — only in the compatibility package [`packages/server-legacy/src/sse`](https://github.com/modelcontextprotocol/typescript-sdk/tree/main/packages/server-legacy/src); not in v2's `@modelcontextprotocol/server` | Y — still shipped as [`src/mcp/server/sse.py`](https://github.com/modelcontextprotocol/python-sdk/blob/main/src/mcp/server/sse.py) | **N — deliberately.** The README calls it "a **deliberate non-goal**" and "a supported-surface decision, not a missing feature", directing legacy peers to a proxy |
| **Child-process client** | Y | Y | Y — feature `transport-child-process` |

Tools, resources and prompts are covered in all three, with matching handler shapes; Rust additionally documents sampling, elicitation (form *and* URL mode), roots, logging, completions, subscriptions, tasks, caching and pagination as first-class README sections.

**None of this matters much for Montaget**, which will be launched over **stdio** by an agent — but it is worth noting that Rust's refusal to ship the legacy SSE transport is the only place any of the three has *less* surface, and it is a surface Montaget does not need.

### 3.3 Schema story (H4) — the axis that actually separates them

This is the heaviest-weighted criterion, and it is where the three genuinely diverge. The question is not "can it emit JSON Schema" — all three emit **JSON Schema draft 2020-12**, the dialect the MCP spec fixes (a schema with no `$schema` key is 2020-12; see the [spec's JSON Schema usage section](https://modelcontextprotocol.io/specification/latest/basic#json-schema-usage), cited by the Python SDK's own [low-level server docs](https://github.com/modelcontextprotocol/python-sdk/blob/main/docs/advanced/low-level-server.md)). The question is **whether Montaget's one published element schema can be the tool's advertised `inputSchema` verbatim, while still validating incoming calls.**

#### The generation path in each

| | TypeScript | Python | Rust |
|---|---|---|---|
| **Mechanism** | **Standard Schema** — `inputSchema` accepts any [Standard Schema](https://standardschema.dev/) that can emit JSON Schema. Zod v4 is the documented default and a hard dependency of `@modelcontextprotocol/server` (`zod: ^4.2.0`); ArkType works unwrapped; Valibot needs `toStandardJsonSchema` | Type hints + **Pydantic**. `@mcp.tool()` reads the function's signature; `Annotated[..., Field(...)]` adds descriptions and constraints; a Pydantic `BaseModel` parameter is the documented way to take a structured body | `#[derive(schemars::JsonSchema)]` on a params struct + the `#[tool]` / `#[tool_router]` macros. **schemars 1.2.2** |
| **Emitted dialect** | 2020-12 by default ([Zod docs](https://zod.dev/json-schema)) | 2020-12; Pydantic writes it and omits the `$schema` key, which the SDK's docs note relies on MCP's default | 2020-12, set explicitly: `SchemaSettings::draft2020_12()` in [`crates/rmcp/src/handler/server/common.rs`](https://github.com/modelcontextprotocol/rust-sdk/blob/main/crates/rmcp/src/handler/server/common.rs) |
| **Nested types** | **Inlined by default** — `reused: "inline"` repeats a duplicated subschema in every property; `reused: "ref"` is opt-in to get `$defs`/`$ref` ([Zod docs](https://zod.dev/json-schema)) | `$defs` + `$ref`. The SDK's own docs: "The `Book` schema is nested inside the tool's input schema (as a `$defs` reference)" ([tools.md](https://github.com/modelcontextprotocol/python-sdk/blob/main/docs/servers/tools.md)) | `$defs` + `$ref` — schemars' default; the [schemars README](https://github.com/GREsau/schemars) shows `"$ref": "#/$defs/MyEnum"` in its headline example |
| **Noise in the emitted schema** | Zod's `.describe()` becomes `description`; no automatic per-property `title` | **A `title` on every property**, and a `"title": "<fn>Arguments"` at the root. The SDK's own docs call these "Pydantic artifacts" and show them in the emitted schema | Cleanest: rmcp explicitly **strips top-level `title` and `description`** — the code comment says they are "the wrapper type name and doc, which are noise to the LLM" — and schemars puts no `title` on individual properties |
| **Discriminated unions** (Montaget's element *is* one, keyed on `type`) | `z.discriminatedUnion` → union constructs; Zod's docs do not spell out the emitted shape — **not found** | `Field(discriminator=...)` → Pydantic docs state it "means the generated JSON schema implements the `discriminator` attribute from the **OpenAPI specification**" ([Pydantic unions docs](https://pydantic.dev/docs/validation/latest/concepts/unions/)) — an OpenAPI keyword, not JSON Schema 2020-12 vocabulary | serde-tagged enums via schemars; no OpenAPI-isms. rmcp deliberately does **not** enable schemars' `AddNullable`, with the code comment: "the `nullable` keyword is an OpenAPI 3.0 extension, not part of JSON Schema 2020-12. Using it would cause validation failures with strict JSON Schema validators" |

#### Can the published schema be used verbatim?

This is the decisive sub-question, and the answer differs sharply:

- **TypeScript — yes, and with validation, at the high level.** `@modelcontextprotocol/server` exports **`fromJsonSchema()`**, which wraps a plain JSON Schema document as a Standard Schema so it can be passed straight to `registerTool`. Its `jsonSchema.input()` returns *the document you passed*, and its `validate()` runs **AJV** on Node ([`packages/core-internal/src/validators/fromJsonSchema.ts`](https://github.com/modelcontextprotocol/typescript-sdk/blob/main/packages/core-internal/src/validators/fromJsonSchema.ts)). The docs confirm the round trip: *"`tools/list` advertises the document you passed, unchanged"* ([schema-libraries.md](https://github.com/modelcontextprotocol/typescript-sdk/blob/main/docs/advanced/schema-libraries.md)). **This is exactly Montaget's shape**: one schema file, published for the agent to read, fed unchanged into the tool, and enforced on every call. The cost is that the handler's arguments are typed `unknown` unless you supply the generic parameter (`fromJsonSchema<Element>(...)`), i.e. you assert the TypeScript type rather than deriving it.
- **Rust — yes for the advertised document, but validation is not tied to it.** `ToolRoute` exposes `.parameters_value(schema: serde_json::Value)` alongside the type-derived `.parameters::<T>()` ([`crates/rmcp/src/handler/server/router/tool.rs`](https://github.com/modelcontextprotocol/rust-sdk/blob/main/crates/rmcp/src/handler/server/router/tool.rs)), so a hand-supplied schema can be advertised. But the handler still receives arguments through serde deserialization into its own parameter type, so the advertised document and the enforced shape are two artifacts that can drift. No JSON-Schema validator is wired to `parameters_value`; **not found** in the repository.
- **Python — no, not without losing validation.** `MCPServer.tool()` takes `name`, `title`, `description`, `annotations`, `icons`, `meta` and `structured_output`, and **no input-schema override** ([`src/mcp/server/mcpserver/server.py`](https://github.com/modelcontextprotocol/python-sdk/blob/main/src/mcp/server/mcpserver/server.py)). To emit an exact schema you must drop to the low-level `Server`, and its own documentation is blunt about the consequence: *"Your `input_schema` is **advertised** to the client; it is never **applied** to `params.arguments`"* — a missing argument becomes a `KeyError` surfacing as a generic JSON-RPC `-32603`, so *"the model never finds out what it did wrong, so it can't retry"* ([low-level-server.md](https://github.com/modelcontextprotocol/python-sdk/blob/main/docs/advanced/low-level-server.md)). Python's choice is therefore **either** a Pydantic-derived schema with free validation and `title` noise, **or** the exact published document with validation you re-implement by hand. The two-artifact drift the map's file-as-truth principle exists to prevent is the default outcome.

#### How faithfully does each render a large nested object?

Montaget's element is a discriminated union with a shared spine (`type`, time range, `layer`, optional `group`, `source`, optional source range) and per-type bodies — a genuinely large nested object.

- **Rust** produces the most faithful and most compact rendering: 2020-12, `$defs`/`$ref` so the shared spine appears once, no OpenAPI keywords, and top-level noise stripped. rmcp also **caches the generated schema per type** in a thread-local map keyed by `TypeId`, so the cost is paid once.
- **Python** is faithful in structure (`$defs`/`$ref`) but noisier per property, and its discriminated-union path emits an OpenAPI `discriminator` keyword into a document the MCP spec pins to JSON Schema 2020-12. That is not fatal — 2020-12 ignores unknown keywords — but it is vocabulary the reading agent has no reason to see, and the map's own principle is that *every tool schema occupies the agent's context*.
- **TypeScript** is the outlier on size: Zod's default `reused: "inline"` **repeats** duplicated subschemas rather than referencing them. For a union of element types that all share the same spine, that means the spine is emitted once per variant. `reused: "ref"` fixes it, but it is opt-in and the MCP SDK's own docs never mention it. Alternatively `fromJsonSchema` sidesteps Zod entirely and emits whatever the published document says — which for Montaget is the better path anyway.

### 3.4 Ergonomics against our shape (H5)

What one write tool — `add_element(path, element)`, taking a complete element object — costs to declare. These sketches follow the SDKs' own documented examples ([TS tools.md](https://github.com/modelcontextprotocol/typescript-sdk/blob/main/docs/servers/tools.md), [Python tools.md](https://github.com/modelcontextprotocol/python-sdk/blob/main/docs/servers/tools.md), [rmcp README](https://github.com/modelcontextprotocol/rust-sdk/blob/main/README.md)).

**Python — shortest to write, and the schema is a by-product.**

```python
from mcp.server import MCPServer

mcp = MCPServer("montaget")

@mcp.tool()
def add_element(path: str, element: Element) -> str:
    """Append one complete element to a project's elements array."""
    ...
```

`Element` is a Pydantic model (or a discriminated `Annotated[Union[...], Field(discriminator="type")]`); the handler receives a validated instance, and bad arguments are rejected before it runs with a message the model reads and retries against. This is the least code of the three by a wide margin. The price is the one named in §3.3: the *published* schema and the *Pydantic* schema are separate artifacts, and there is no supported way to collapse them at the high level.

**TypeScript — two honest options, and one of them is a direct fit.**

```ts
import { McpServer, fromJsonSchema } from '@modelcontextprotocol/server';
import elementSchema from './schema/element.json' with { type: 'json' };

server.registerTool(
    'add_element',
    { description: '…', inputSchema: fromJsonSchema<AddElementArgs>(elementSchema) },
    async args => { /* args typed by the generic, validated by AJV */ }
);
```

or the Zod path, where `inputSchema: z.object({ path: z.string(), element: elementSchema })` gives derived TypeScript types for free but makes the Zod schema the source of truth and the JSON Schema a derivative. The first form is the one that matches file-as-truth; the second is the one that matches idiomatic TypeScript. They are mutually exclusive per tool.

**Rust — most ceremony, best output.**

```rust
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct AddElementParams {
    /// Path to the project file.
    path: String,
    /// The complete element to append.
    element: Element,
}

#[tool_router(server_handler)]
impl Montaget {
    #[tool(description = "Append one complete element to a project's elements array")]
    async fn add_element(&self, Parameters(p): Parameters<AddElementParams>) -> Result<CallToolResult, McpError> { … }
}
```

`Element` must derive `Deserialize + JsonSchema` throughout the union — which for a large nested type means every variant and every nested struct carries two derives and doc comments that become `description`s. That is real, repetitive declaration cost. In exchange the emitted schema is the cleanest of the three, the deserialization is checked at compile time, and rmcp's error model documents the split Montaget needs anyway: `Ok(CallToolResult::error(...))` for "the tool ran and it didn't work" (which the agent sees and can act on) versus `Err(McpError)` for "the server can't route this" (which clients render opaquely). The map's *"schema catches malformed, agent catches wrong"* principle maps onto that split directly.

**A note that cuts against all three equally:** MCP fixes tool `inputSchema` root type to `object`. rmcp enforces this at generation time and returns an error string for a non-object root ([`common.rs`](https://github.com/modelcontextprotocol/rust-sdk/blob/main/crates/rmcp/src/handler/server/common.rs)); the others rely on you not doing it. As of `2026-07-28` (SEP-2106) `outputSchema` may be any JSON Schema type and `structuredContent` any JSON value, which all three document.

---

## 4. Skia binding health (H6)

All three bind the *same* C++ Skia, so the question is entirely about the CI story: does the current release ship a prebuilt binary covering **macOS arm64** and **linux x64**, or does someone have to compile Skia?

| | `skia-safe` (Rust) | `skia-canvas` (Node) | `skia-python` (Python) |
|---|---|---|---|
| **Repo** | [rust-skia/rust-skia](https://github.com/rust-skia/rust-skia) | [samizdatco/skia-canvas](https://github.com/samizdatco/skia-canvas) | [skia-python/skia-python](https://github.com/skia-python/skia-python) — `kyamagu/skia-python` now 301-redirects here |
| **Current release** | **0.99.0**, 2026-06-19 ([crates.io](https://crates.io/api/v1/crates/skia-safe)) | **3.0.8**, 2025-09-25 ([npm](https://registry.npmjs.org/skia-canvas)) | **144.0.post2**, wheels 2026-03-19 ([PyPI](https://pypi.org/pypi/skia-python/json)) |
| **Releases in last 12 mo** | ~9 (roughly monthly), but **nothing since 2026-06-19** while master sits at an unreleased 0.153.2 | **Zero.** All of 3.0.x shipped in one burst Aug–Sep 2025 | 3, with a **~9-month silent gap** between 138.0 (2025-06-09) and 144.0 (2026-03-05) |
| **Last commit** | 2026-08-31 (master active) | **2025-09-26 on `main`** — ~11 months stale; recent pushes are to side branches (`version-4`, `gpu/surface-cache`, …) for an unreleased v4 | 2026-08-19 |
| **Open issues (excl. PRs)** | 41 | 23 | 45 |
| **Stars / licence** | 1,808 / MIT | 2,601 / MIT | 322 / BSD-3-Clause |
| **Binding style** | Thin safe bindings over C++ Skia, vendored as a submodule of rust-skia's Skia fork | **Not thin** — a Canvas2D/DOM *implementation* written in Rust on top of `skia-safe`, exposed to Node via Neon (N-API v8) | Thin pybind11 binding; Skia vendored straight from `skia.googlesource.com` |
| **Skia milestone** | **m153** on master; **m150** in released 0.99.0 | **m140**, only derivable transitively via its pin of `skia-safe = "0.88.0"` — two milestones behind | **m144** |
| **macOS arm64 prebuilt?** | **Y** — 14 assets for `aarch64-apple-darwin` in [skia-binaries 0.99.0](https://github.com/rust-skia/skia-binaries/releases/tag/0.99.0) | **Y** — `darwin-arm64.gz` in [release v3.0.8](https://github.com/samizdatco/skia-canvas/releases/tag/v3.0.8) | **Y** — `macosx_11_0_arm64` wheels for cp38–cp314 incl. cp314t |
| **linux x64 prebuilt?** | **Y** — 13 assets for `x86_64-unknown-linux-gnu` | **Y** — `linux-x64-glibc.gz` **and** `linux-x64-musl.gz` | **Y** — `manylinux_2_28_x86_64` for cp38–cp314 incl. cp314t |
| **How the binary is delivered** | **Not** on the crate or on rust-skia's own releases (which carry **0 assets**). `skia-bindings`' `build.rs` `curl`s a tarball from the *separate* [rust-skia/skia-binaries](https://github.com/rust-skia/skia-binaries/releases) repo, keyed by a **hash of the enabled feature set** | **Not** npm platform packages — `optionalDependencies` is absent and no `@skia-canvas/*` scoped packages exist (404 on `@skia-canvas/darwin-arm64`). An `install` lifecycle script (`node lib/prebuild.mjs download --or-compile`) fetches a `.gz` from GitHub releases | **Real PyPI wheels.** `pip install` is a plain binary fetch from the index |
| **From-source fallback** | LLVM/Clang + Python 3 + Ninja + recent `git`; Xcode CLT on macOS; `pkg-config`/`libssl-dev` on Linux ([README "Building"](https://github.com/rust-skia/rust-skia/blob/master/README.md)) | rustup + a C toolchain + Python 3 + Ninja, plus Fontconfig/OpenSSL on Linux — "a fairly lengthy compilation process" ([README](https://github.com/samizdatco/skia-canvas/blob/main/README.md)) | **None — and that cuts both ways.** 144.0.post2 publishes **45 wheels and zero sdist**, so an uncovered platform fails outright rather than compiling |
| **CI friction** | Feature-hash miss ⇒ silent fall-through to a full Skia compile. README only promises prebuilts for "**most** feature combinations" | Needs **github.com egress** at install time, and under pnpm the install script is blocked by default (`pnpm approve-builds` / `onlyBuiltDependencies`) | Needs Linux runtime libs at *import*: OpenGL, **libEGL**, fontconfig (`libegl1`, `libgl1-mesa-*`, `libglvnd0`) |

**Reading.** This is the axis where a Rust-lean is weakest on its own terms. `skia-python` has the cleanest CI story of the three by some distance — wheels on the index, no build script, no external host to reach, `pip install` and done. `skia-safe`'s prebuilts genuinely cover both target triples, but they are fetched by a build script from a second repository and matched by *feature-set hash*, so an unusual feature combination drops into an LLVM/Python/Ninja Skia build with no warning; and its published crate has been static for ~2.5 months while master runs three Skia milestones ahead. `skia-canvas` is the one binding in this survey that shows the renderer survey's **Revideo pattern in miniature**: no npm release in ~11 months, `main` untouched since 2025-09-26, two Skia milestones behind, and its visible work parked on unreleased `version-*` branches. It is not archived and not abandoned — but it is the only component here whose *released* artifact is a year old.

---

## 5. Text shaping (H7)

The domain is bilingual subtitles, and the map carries an open question about whether RTL or CJK is in the channel's future. Latin-only and full-bidi are therefore reported **separately**, because they separate the stacks and nothing else in this survey does so as sharply.

**One correction to the ticket's framing, up front:** `cosmic-text` is not the natural Rust choice here. It has **no Skia dependency at all** — its only rasterizer is `swash`, and `tiny-skia` appears solely as a dev-dependency ([Cargo.toml](https://github.com/pop-os/cosmic-text/blob/main/Cargo.toml)). Using it with Skia means writing the glyph-run bridge yourself. The Rust stack that actually parallels the Node and Python ones is **`skia-safe`'s own `textlayout` feature**, which the README describes as enabling "text shaping with Harfbuzz and ICU by providing bindings to the Skia modules skshaper and skparagraph" ([skia-safe README](https://github.com/rust-skia/rust-skia/blob/master/skia-safe/README.md)). Both Rust options are given below.

### Latin-only: single font, LTR, multi-line wrap

| Stack | Works out of the box? | The API |
|---|---|---|
| **Rust — `skia-safe` `textlayout`** | **Y** | `textlayout::{FontCollection, ParagraphBuilder, ParagraphStyle, TextStyle}` → `paragraph.layout(w)` → `paragraph.paint(canvas, pt)`. Needs `features = ["textlayout"]`; prebuilt binaries exist for that combo (e.g. `…-aarch64-apple-darwin-jpegd-jpege-pdf-textlayout.tar.gz` in skia-binaries 0.153.2), so **no source build**. ICU is handled by the default `embed-icudtl` feature, which embeds `icudtl.dat` in the binary and initialises it automatically |
| **Rust — `cosmic-text`** | **~** — layout yes, drawing is yours | `FontSystem` → `Buffer::new(metrics)` → `set_size` → `set_text(.., Shaping::Advanced, ..)` → iterate layout runs, then rasterize via `SwashCache` or hand glyph IDs to Skia ([docs.rs](https://docs.rs/cosmic-text/0.19.0/cosmic_text/)) |
| **Node — `skia-canvas`** | **Y**, and the shortest of the three | `ctx.textWrap = true` (a documented extension: it honours `"\n"`, treats the `width` argument to `fillText`/`measureText` as a wrap column, and uses the font's line-height for leading), then `ctx.fillText(str, x, y, columnWidth)` ([context.md](https://github.com/samizdatco/skia-canvas/blob/main/docs/api/context.md)) |
| **Python — `skia-python`** | **Y** | `skia.textlayout_ParagraphBuilder` + `ParagraphStyle` + `TextStyle` + `FontCollection` → `paragraph.layout(w)` → `paragraph.paint(canvas, x, y)`. The wheels ship the module ([reference](https://skia-python.github.io/skia-python/reference/skia.html)) |

**On Latin-only, all three are fine.** If the channel stays Latin, this axis carries no weight at all.

### Full bidi + per-character font fallback + complex shaping

| Stack | (a) Bidi / RTL | (b) Per-character fallback | (c) Complex shaping | Extra work |
|---|---|---|---|---|
| **Rust — `skia-safe` `textlayout`** | **Y** — HarfBuzz + ICU; `ParagraphStyle::text_direction` | **Y** — `FontCollection::set_default_font_manager` | **Y** — HarfBuzz | Enable one Cargo feature |
| **Rust — `cosmic-text`** | **Y** — depends on `unicode-bidi`; `Direction` detects each paragraph's base direction from its first strong character, per-span `unicode_bidi::Level`; README ticks "Bidirectional rendering" with an Arabic UDHR screenshot | **Y, per-character** (README roadmap: "Per-character granularity ✓"), **on Linux/macOS/Windows only** — `src/font/fallback/other.rs` returns empty lists, so any other target (including Android) gets no fallback unless you supply a `Fallback` impl | **Y** — shaping is now **`harfrust`** (not `rustybuzz`, as the ticket assumed) via `Shaping::Advanced` | You write the cosmic-text → Skia draw bridge |
| **Node — `skia-canvas`** | **~** — **base** paragraph direction only, and only verifiable from source: `get_direction`/`set_direction` in `src/context/api.rs` map `"ltr"`/`"rtl"` onto `textlayout::TextDirection`. **Mixed-direction UBA reordering: not found** — the project's README and API docs never mention bidi, RTL or shaping at all | **Y**, and documented: `measureText().lines[].runs` decomposes a line "into all its single-font ranges of characters… useful in cases where a fallback font is providing character glyphs not present in the 'main' font"; `FontLibrary.use()` registers local faces | **Y** — inherited, since its `Cargo.toml` enables `skia-safe`'s `textlayout` | None, but you are trusting undocumented inherited behaviour |
| **Python — `skia-python`** | **N as exposed.** There is **no `TextDirection` enum and no `ParagraphStyle.setTextDirection` binding** in `src/skia/Paragraph.cpp` or in the published reference. ICU and HarfBuzz *are* compiled into the wheels (`setup.py` links `skparagraph`, `skshaper`, `skunicode_icu`, `skunicode_core`; the build script sets `skia_use_system_icu=false`), so the engine can do it — the knob is simply not bound. The only direction control anywhere is `TextBlob.MakeFromShapedText(..., leftToRight: bool)`, a single-run escape hatch | **Y** — `TextStyle.setFontFamilies([...])` as a fallback chain + `FontCollection.setDefaultFontManager` | **Y** — HarfBuzz bundled into the wheels | **A PR to skia-python, or a fork** |

**Reading.** Latin-only is a three-way tie. Full bidi is not: **Rust leads, Node is undocumented-but-probably-fine, and Python is the one stack with a concrete, named blocker** — a missing pybind11 binding, not a missing capability. That inverts the Skia-binding result from §4, where Python had the cleanest story. It also means the map's open question about RTL/CJK is a genuine input to #7 and not a hypothetical: it is the difference between "any of the three" and "not Python without upstream work".

Note the direction of travel on the Rust side too: `rustybuzz` (which the ticket named) was last published **2024-11-12**, and the same GitHub org's newer `harfrust` (0.13.3, 2026-08-25) is what cosmic-text 0.19.0 now depends on.

---

## 6. Distribution (H8)

The MCP config contract is, in every documented example, **an executable plus argv**. The official ["build a server"](https://modelcontextprotocol.io/docs/develop/build-server) and ["connect local servers"](https://modelcontextprotocol.io/docs/develop/connect-local-servers) pages show:

- Node: `"command": "npx", "args": ["-y", "@modelcontextprotocol/server-filesystem", …]`, glossed as "Uses Node.js's npx tool to run the server" and "`-y`: Automatically confirms the installation of the server package"; or `"command": "node", "args": ["/ABSOLUTE/PATH/…/build/index.js"]`
- Python: `"command": "uv", "args": ["--directory", "/ABSOLUTE/PATH/…", "run", "weather.py"]`
- Rust: `"command": "/ABSOLUTE/PATH/…/target/release/weather"` — **with no `args` at all.** The compiled binary *is* the command.

| | Rust | Node | Python |
|---|---|---|---|
| **Install** | `cargo install` builds and installs into `~/.cargo/bin` by default ([cargo-install](https://doc.rust-lang.org/cargo/commands/cargo-install.html)) — but that requires the *user* to have a Rust toolchain. A prebuilt binary requires nothing at all | `npm install` / `npx` — npx "installs them to a cache folder and adds them to the PATH" if not already present ([npx docs](https://docs.npmjs.com/cli/v11/commands/npx)) | `pip`/`pipx`/`uvx`. `pipx` "installs and runs end-user Python applications in isolated environments. It fills the same role as macOS's brew, JavaScript's npx…" ([pipx](https://pipx.pypa.io/stable/)); `uvx X` is "exactly equivalent to `uv tool run X`" ([uv tools](https://docs.astral.sh/uv/guides/tools/)) |
| **Self-contained artifact** | **Y.** `x86_64-unknown-linux-musl` is **Tier 2 with Host Tools** ([platform support](https://doc.rust-lang.org/rustc/platform-support.html)) and is one of the targets that "are static by default", linking the C runtime statically ([Reference — linkage](https://doc.rust-lang.org/reference/linkage.html)) | **~.** Single Executable Applications are **`Stability: 1.1 — Active development`**, and the docs state CI tests cover Windows, **macOS arm64 only (x64 "is not currently supported and is skipped in the tests")**, and Linux except Alpine ([SEA docs](https://nodejs.org/api/single-executable-applications.html)) | **N by itself**, but `uv` closes most of the gap: "uv automatically installs missing Python versions as needed — you don't need to install Python to get started", using python-build-standalone distributions ([uv](https://docs.astral.sh/uv/guides/install-python/)). `pipx`, by contrast, "prefers a real system Python and only downloads a standalone build as a fallback", and its default `--fetch-python=never` "keeps pipx offline and errors out instead" ([pipx internals](https://pipx.pypa.io/stable/explanation/how-pipx-works.html)) |
| **What runs on every server start** | The binary | A launcher (`npx` or `node`) plus module resolution | A launcher (`uv`/`uvx`/`pipx`) plus interpreter bootstrap. `uvx` environments are "stored in the uv cache directory and… treated as disposable"; note the documented failure mode: "If the Python version used by a tool is *uninstalled*, the tool environment will be broken" ([uv tools concepts](https://docs.astral.sh/uv/concepts/tools/)) |

**Reading.** **Rust leads this axis cleanly, and it is the one axis where the conventional Rust argument fully survives contact with the sources.** The MCP docs' own Rust example — a bare path to a binary with no arguments — is the entire case. Node's SEA is documented as not yet stable and *not tested on macOS x64*; Python needs an interpreter that `uv` can supply but `pipx` deliberately will not. This axis also directly narrows the map's **Packaging and distribution** fog, which #7 is holding.

---

## 7. Startup cost (H9) — and the finding that this axis cannot be read

[#6](https://github.com/MBehtemam/Montaget/issues/6) established that the preview loop is **startup-dominated**, with FFmpeg's cold start at ~0.93s for a full frame. This survey went looking for published startup figures for the three runtimes. Here is everything that exists, and it is much less than expected.

| | Rust | Node.js | Python |
|---|---|---|---|
| **Published absolute cold-start figure** | **Not found** | **Not found** | **Not found** |
| **Published relative claim** | **Not found** | Qualitative only: the v18 announcement says a snapshot-built binary "can be initialized **faster**" and "improves startup time" ([v18 release announcement](https://nodejs.org/en/blog/announcements/v18-release-announce)); the SEA docs say `useCodeCache` "would improve the startup performance" — **no numbers anywhere** | **"Interpreter startup is now 10-15% faster in Python 3.11"**, by freezing core startup modules so module execution goes from *read `__pycache__` → unmarshal → heap-allocate → evaluate* to *statically allocated code object → evaluate* ([What's New in 3.11](https://docs.python.org/3/whatsnew/3.11.html)). This is the single strongest startup claim found in any of the three ecosystems |
| **Later releases** | n/a | n/a | 3.12: **no startup claim of any kind**. 3.13: only per-module import cost — "the import time of the `typing` module has been reduced by around a third". 3.14: no interpreter-startup claim, only the new `-X importtime=2` tracing of already-cached modules |
| **Structural fact instead** | A compiled native binary has no interpreter or runtime to bootstrap; the citable facts are the static-by-default musl target and Tier 2 with Host Tools status (see §6) | Startup snapshots exist and are **no longer experimental as of v25.4.0 / v24.13.1** (`--build-snapshot` / `--snapshot-blob`), but "the snapshot currently only supports loading a single entrypoint… which can load built-in modules, but not additional user-land modules" — you must bundle first ([CLI docs](https://nodejs.org/api/cli.html)) | `-X importtime` is the documented instrument for finding where startup goes ([command line docs](https://docs.python.org/3/using/cmdline.html)) |
| **Nearby number that must not be misused** | — | V8's own blog reports custom startup snapshots taking context creation "from **40 ms down to less than 2 ms**" on desktop and "**270 ms** and **10 ms**" on mobile ([v8.dev](https://v8.dev/blog/custom-startup-snapshots)). **This is V8 context creation, not Node process startup.** It is cited here only as evidence that snapshot deserialization is a real, large-magnitude mechanism — quoting it as "Node starts in 2 ms" would be a fabrication | PEP 690 (Lazy Imports) measured "startup time improvements up to 70%… on real-world Python CLIs" — but PEP 690 was **Rejected** ([PEP 690](https://peps.python.org/pep-0690/)). It is evidence that *import cost dominates CLI startup*, not a property CPython has |

**This is the survey's single most important finding (§2), and the evidence for it is the table above:** the axis that #6 identified as the one most likely to separate the three hosts **cannot be filled from published primary sources for any of them.** Not one of the three runtimes publishes an absolute cold-start figure. This is the exact same shape as the renderer survey's C6 column, and it has the same remedy: **#7 must measure it, not read it** — three hello-world binaries plus a minimal `initialize` handshake, timed the way #6 timed FFmpeg's 0.93s.

---

## 8. Where each host leads

The full scorecard is in §2. This is the shorter question #7 actually has to answer: on each axis, who leads, and how much daylight is there?

| Axis | Leads | Margin |
|---|---|---|
| **H1 first-party SDK** | *tie* | None. All three are under the `modelcontextprotocol` org, all actively maintained, none archived. The only thing to get wrong here is picking the wrong Rust crate. |
| **H2 maturity** | **Python**, narrowly, on **Rust** | Small. Python has the most stars and near-top commit volume; Rust has the smallest backlog by an order of magnitude. TypeScript trails only because of its 305/299 issue-and-PR backlog and its contribution freeze. |
| **H3 spec coverage** | *tie* | None worth naming. All three implement `2026-07-28`. The only difference — legacy HTTP+SSE — is a transport Montaget will never use. |
| **H4 schema story** *(heaviest)* | **TypeScript** | **Large, and decisive on our stated principle.** `fromJsonSchema()` is the only first-party path that makes the published schema and the enforced schema one artifact. |
| **H5 ergonomics** | **Python** | Moderate on line count, but the win is partly cancelled by H4: Python's brevity comes from deriving the schema, which is the thing Montaget does not want derived. |
| **H6 Skia CI story** | **Python** | Moderate. Real wheels beat a `build.rs` download beat a postinstall script. Rust is third here, which is not where a Rust-lean expects it. |
| **H7 text — Latin only** | *tie* | None. If the channel stays Latin, this axis is worth nothing. |
| **H7 text — full bidi** | **Rust** | **Large, and it is the axis the map's open RTL/CJK question turns on.** Python has a named blocker (no `TextDirection` binding); Node's behaviour is undocumented. |
| **H8 distribution** | **Rust** | **Large.** A static binary against a launcher that runs on every server start. The MCP docs' own Rust example makes the case without commentary. |
| **H9 startup cost** | **unknown** | Unmeasurable from published sources. Structurally Rust should lead; nothing published proves it. |

Read together: **Rust's case rests on H8 and H9** — one axis it wins convincingly and one nobody can score yet — **plus H7 full-bidi, which is only load-bearing if the channel goes RTL or CJK.** **TypeScript's case rests almost entirely on H4**, the axis this survey was told to weight heaviest, where it is the only candidate that satisfies the map's file-as-truth principle without a second artifact. **Python's case is breadth**: it leads or ties on five axes and is only decisively behind on one — but that one is full bidi, and it is behind for a reason that a single upstream PR would remove.

If #7 wants the trade in one sentence: **Rust buys the best distribution and the best text stack at the cost of the worst schema fit and the most declaration ceremony; TypeScript buys the only clean schema fit at the cost of the weakest packaging story and the most stalled Skia binding; Python buys the least code and the best CI at the cost of an RTL blocker and a schema story that forces two artifacts.**

---

## 9. Open items this survey could not close

1. **No absolute cold-start figure exists for any of the three runtimes.** H9 is unfillable from published sources. #7 must measure — three hello-worlds plus a minimal `initialize` handshake, against #6's 0.93s FFmpeg baseline.
2. **Python download counts: not found.** PyPI's own API publishes none, and pypistats.org (secondary) returned HTTP 429 on both attempts. The TypeScript/Rust adoption figures in §3.1 therefore have no Python column to compare against.
3. **`skia-canvas` bidi behaviour is undocumented.** Base direction is wired to `TextDirection` in the Rust source, but no project doc mentions bidi, RTL, shaping or fallback at all. Whether mixed-direction UBA reordering works is **not confirmed** — recorded as unknown, not as absent. If Node is a serious candidate and RTL is in the channel's future, this is a 30-minute empirical test, not a reading exercise.
4. **`skia-canvas` pins `skia-safe = "0.88.0"` while current skia-safe is `0.153.2`.** No project statement explaining the gap was found, and it is unclear whether the unreleased `version-4` branch closes it.
5. **`skia-python`'s missing `TextDirection` binding** is a small, well-defined upstream contribution rather than a capability gap — the ICU and HarfBuzz machinery is already compiled into the wheels. Its cost should be estimated, not assumed prohibitive, if Python is otherwise attractive.
6. **Zod's emitted shape for `z.discriminatedUnion` is not documented** on the JSON Schema page — **not found**. Since Montaget's element is exactly a discriminated union, this matters if the Zod route is taken rather than `fromJsonSchema`.
7. **rmcp's `parameters_value()` has no attached validator.** No JSON-Schema validation of incoming arguments against a hand-supplied document was found in the repository. If Rust is chosen and the published schema is to be advertised verbatim, that validation is Montaget's to write.
8. **The MIT → Apache-2.0 licence transition** affecting the TypeScript and Rust SDK repositories is stated in their `LICENSE` files but its completion state is not published; the npm package still declares MIT while the `rmcp` crate declares Apache-2.0. Cosmetic for our purposes, but worth a glance before shipping.
9. **All three SDKs are weeks into a major-version transition.** Whichever is chosen, Montaget starts on a release line that is younger than this survey's own research window.

