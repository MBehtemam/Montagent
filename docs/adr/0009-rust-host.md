---
status: accepted
---

# Montagent is a Rust MCP server

> **Amended by [ADR-0077](./0077-the-nine-render-readings-are-ratified.md)**, which ratifies the encoder
> settings the spawned `ffmpeg` is given — `libx264`, `yuv420p`, CRF 20, preset `medium`,
> `+faststart`, AAC at 160 kb/s — and confirms the `libopenh264` escape route named below
> is still not taken: an `ffmpeg` without `libx264` surfaces as exit 70 carrying its own
> sentence, never a silent fallback.

> **Amended by [ADR-0115](./0115-ffmpeg-7-1-with-libx264-is-the-floor-and-a-tool-qualification-finds-out.md)**: the `ffmpeg` the user supplies has a floor — **ffmpeg 7.1
> or newer, built with `libx264`** — stated as the three arguments that decide it rather than a
> version string, and checked by a **tool qualification** (one null encode per process). An
> `ffmpeg` below it is `E-TOOL-UNSUPPORTED`, exit 70.

Montagent is hosted in **Rust**, built on the first-party **`rmcp`** SDK.

This ADR records the **host** only — the language, the MCP SDK, the packaging
shape and the consequences that follow from them. **It does not choose the
renderer.** That half of [#7](https://github.com/MBehtemam/Montagent/issues/7)
rests on a different body of evidence, is still unmeasured, and gets its own ADR.

## Why

**Not for speed.** The axis everybody expected to decide this was measured and
came back noise. [#16](https://github.com/MBehtemam/Montagent/issues/16) timed
spawn→`tools/list` at **Rust 4.8 ms, TypeScript 103.3 ms, Python 421.6 ms** — an
88× win for Rust that **cannot be spent**. On the stdio binding the client
launches one subprocess per session, so startup is paid once: never per tool
call, never per preview. A 417 ms spread sits against ~2.8 s of headroom on
every preview and under half of FFmpeg's 0.93 s *per frame*. A future reader who
assumes this was a performance decision will draw the wrong conclusions from it.

**The tie broke by one side weakening, not by Rust strengthening.**
[#15](https://github.com/MBehtemam/Montagent/issues/15) left the choice split:
the schema story pointed at TypeScript, distribution pointed at Rust. Then #16
found the **TypeScript SDK is a protocol revision behind** — 1.30.0 is `latest`,
tops out at `2025-11-25`, and returns `-32601` for `server/discover`, while
`rmcp` and the Python `mcp` package both implement `2026-07-28`. The schema
story was TypeScript's only claim to the job.

**Distribution is the one axis Rust wins outright and can keep.** A compiled
binary with no interpreter and no `node_modules`. Node's single-executable route
is `Stability: 1.1` and untested on macOS x64; a Python package needs an
interpreter that `uvx` supplies and `pipx` deliberately will not. See
*Consequences* for the correction this ADR makes to that claim — it is smaller
than #7 originally stated.

**The text stack argues for Rust, and both surveys had this backwards.**
`renderer-survey.md` and `mcp-sdk-host.md` treated text as a renderer concern
that could not discriminate between hosts. But
[ADR-0008](0008-line-breaks-belong-to-the-agent.md) requires `measure` to emit
**break opportunities with the segmenter and data version named**, and
[ADR-0007](0007-text-runs-literal-size-declared-fonts.md) requires fonts to come
only from files the project declares. `parley`, `cosmic-text`, `fontique`,
`fontdb` and `icu_segmenter` are Rust-only, and
[#27](https://github.com/MBehtemam/Montagent/issues/27) has already run that
stack end to end. The surveys scored this axis as host-neutral; it is not.

## What this ADR does not decide

**The rasterizer.** [#6](https://github.com/MBehtemam/Montagent/issues/6)
measured `skia-canvas`, which is **Node**. No rasterizer has been measured under
a Rust host, and every downstream argument has been quietly transferring those
Node figures. `skia-safe` and `tiny-skia` have opposite build and distribution
stories and the choice between them is open.

**Whether `tiny-skia` is still disqualified.** `renderer-survey.md` ruled it out
**alone on C2**, text — "the main missing feature is text rendering". ADR-0007
and ADR-0008 have since moved line partitioning and break reporting *out of the
renderer entirely*, and #27 rasterized every Thai sample through `tiny-skia`
with an external text stack. The criterion that killed it no longer describes
this design.

**Video-clip decode.** The map lists video clips as first-class inputs. #6
rendered four stills, 73 subtitle and shape events and twelve narration
placements — **zero decoded video frames**. The FFmpeg/skia tie and the "~9×
slack" headline are both stills-only facts, and #6's one real pathology, cost
proportional to where a window starts, *came from decoding* — the thing the
fixture excluded.

## Consequences

### The schema is a file, and a validator enforces it

`rmcp` **3.2.0** advertises a schema but does not validate against it. The
escape hatch is `#[tool(input_schema = <expr>)]`, which is used verbatim when
supplied; otherwise the macro falls back to a `schemars`-derived schema. There
is no `jsonschema` dependency anywhere in the crate — enforcement is a single
`serde_json::from_value`, and the SDK says so in its own source: *"Since rust is
a strong type language, we don't need to do json schema validation here. But if
you do have to validate … you can use the `jsonschema` crate."*

So the map's principle — *schema catches malformed, agent catches wrong* —
requires Montagent to supply the missing half:

> **`element.json` is the artifact.** It is hand-written, embedded with
> `include_str!`, advertised verbatim through `input_schema`, and compiled once
> at startup with the **`jsonschema`** crate (0.53.0, MIT, 2020-12, and
> actively developed — `boon` has not shipped in ~20 months). Every call is
> validated against it before dispatch.

**Do not `derive(JsonSchema)` for the element type.** That inverts the direction
of truth: the Rust type becomes the source and the published document a
generated shadow of it. This ADR's author initially recommended exactly that,
and three independent reviewers rejected it — it *recreates* the two-artifact
split rather than closing it, which is the defect
[#15](https://github.com/MBehtemam/Montagent/issues/15) charged Python with.
The document is the contract; the Rust type is one consumer of it.

### FFmpeg is a subprocess, and the binary is not as self-contained as claimed

**The encoder carries a licence the host ADR would otherwise inherit by
accident.** `libx264` is GPL-2.0-or-later. FFmpeg's `configure` places it in
`EXTERNAL_LIBRARY_GPL_LIST`, so it requires `--enable-gpl`, and FFmpeg's own
legal page states that **"if those parts get used the GPL applies to all of
FFmpeg."** Linking `libavcodec` into Montagent via `ffmpeg-next` would make the
distributed binary a GPL combined work — the FSF is explicit that static *or*
dynamic linking produces a combined work — and the permissive licence Montagent
intends to publish under would be false. The crate's own licence is irrelevant;
the C library it links is what carries the obligation.

> **Montagent spawns a separate `ffmpeg` executable and pipes raw frames to it.**

The FSF's stated position is that pipes and fork/exec are the communication
mechanisms of *separate programs*, and raw video frames are data rather than
intimate internal structures. Recorded honestly: the FSF also frames "separate
program" as a legal question for judges, and there is no controlling case law on
ffmpeg-as-subprocess. This is the FSF's position, not settled law.

**This costs the distribution claim above, and the cost is stated rather than
softened.** #7 argued for Rust partly on a single static binary with no runtime.
That is now **"a binary, plus an `ffmpeg` the user supplies."** Bundling one in
the release archive is mere aggregation for Montagent's own code — but it makes
this project a **distributor of GPL software, owing the source-offer duties of
GPL §3/§6 for the bundled binary.** There is no option here that is both
self-contained and obligation-free.

The escape route, if the dependency ever becomes intolerable: an **LGPL** FFmpeg
built without `--enable-gpl` can still encode H.264 through `libopenh264` or the
platform hardware encoders, none of which are on the GPL list. The cost is
quality — FFmpeg documents `libopenh264` as lacking B-frames and some
main/high-profile features. Not chosen now; recorded so it is not rediscovered.

**Patent exposure is independent of all of the above.** H.264 is a Via LA
(formerly MPEG LA) pool codec. No copyright licence — GPL, BSD or otherwise —
conveys patent peace, and self-compiled OpenH264 carries no patent grant, since
Cisco's royalty payment covers only its own precompiled binaries.

### The `skia-safe` CI worry does not apply to a textlayout-free build

[#15](https://github.com/MBehtemam/Montagent/issues/15) ranked Skia's CI story
third because `skia-safe` fetches feature-hash-keyed prebuilts and can silently
fall back to a source build. A reviewer of this ADR argued the inversion — that
dropping `textlayout` (which the text decision above implies) would *cause* that
miss, since the published assets are named for textlayout combinations.

**Measured against the release assets, that is false.** Of the 152 assets
published for `skia-safe` 0.153.2, roughly half are textlayout-free, and
`skia-safe`'s own default feature set resolves to the key `jpegd-jpege-pdf`,
which is published for Linux, macOS and Windows on both x86-64 and aarch64.

The real hazard is narrower and worth carrying forward: **matching is on the
exact sorted feature set, not a subset.** Enabling any feature outside a
published combination misses the cache and triggers a full source build. Note
also that `svg` and `skottie` both imply `textlayout`, so neither is available
in a textlayout-free build.

This does not decide the rasterizer. It removes one argument against one
candidate.

### Carried into the renderer ADR

- **Text layout stands beside the rasterizer, not inside it.** Unanimous across
  three independent reviewers. ADR-0008 requires `measure` to report break
  opportunities and name its segmenter and data version; `SkParagraph` is a
  wrapping-policy engine with ICU sealed inside and exposes neither. Taking it
  would ship **two shapers**, and the one the agent measures with would not be
  the one that draws.
- **`skrifa` is the bridge**, not `swash`: `parley` 0.11.1 already shapes with
  `harfrust` and breaks with `icu_segmenter`, so `skrifa` scales the outlines
  and the rasterizer fills the paths. `swash` would duplicate a shaper the stack
  already has.
- **`tiny-skia` 0.12.0 is pure Rust with no C++ dependency**, but its README
  puts GPU rendering, image filters and blur out of scope, and it has shipped
  one release in ~2 years. Those are the real objections to it — not text.
- Do not rank `cosmic-text` below `parley` on startup: its 230 ms is `fontdb`'s
  system scan, which ADR-0007 forbids anyway.
- **The Remotion licence argument is redundant, not independent.** #6 already
  eliminated browser pipelines on preview latency. Stacking both reads as
  motivated reasoning and should not appear in the renderer ADR.
- **The budget was written for 1080×1920/30 and has never been re-derived**
  since [ADR-0003](0003-general-video-editor-not-channel-tooling.md)
  generalised the scope. "~9× slack" is not a claim about 4K.
- **Nobody has stated whether Montagent is a resident server.** #16 retired
  startup by measuring it against the 10 s preview; the per-turn
  self-verification loop is a *single frame*, where the same spread is a much
  larger share. The renderer ADR should say which mode it assumes.
