# Survey: programmatic and declarative video renderers

Research for [#5](https://github.com/MBehtemam/Montagent/issues/5), part of the map [#2](https://github.com/MBehtemam/Montagent/issues/2).

**Date of research:** 2026-09-02. Version numbers, release dates and star counts are as of that date.

**This document does not recommend a winner.** [#7 "Choose the renderer and host language"](https://github.com/MBehtemam/Montagent/issues/7) makes that call, informed by measurements from [#6](https://github.com/MBehtemam/Montagent/issues/6). What follows is the landscape and the scores.

**Sourcing rule applied:** every claim cites a primary source (official docs, repo, LICENSE, registry API). Where a figure is not published, this document says so explicitly rather than estimating. A handful of Remotion claims could only be recovered through search-engine summaries because the doc URLs 404'd on direct fetch; those are marked **[secondary]**.

> [!WARNING]
> **Scope correction — the criteria below were written under a narrower scope than now stands.**
>
> [ADR-0003](../adr/0003-general-video-editor-not-channel-tooling.md) settles that Montagent is a **general, open-source, agent-first video editor** in the CapCut/Premiere class. The `youtube_language_learning` channel is a **fixture and regression guard, never a scope boundary**. See [#19](https://github.com/MBehtemam/Montagent/issues/19).
>
> Two things below were written against the narrow scope and must not be read at face value:
>
> - **C2 says "bilingual subtitle text."** Read it as **text and typography generally** — multi-script, arbitrary fonts, styled runs, full shaping. A video editor that strangers run cannot be Latin-only by construction, whatever any one channel publishes. Note this makes tiny-skia's disqualification on C2 stand *more* firmly, not less.
> - **"Target format throughout: 1080x1920, 30fps"** was the *channel's* format, not a constraint on Montagent. Frame size and rate are project properties, and **arbitrary aspect ratios are in scope**.
>
> The **facts** in this survey are unaffected — they were gathered against primary sources and remain good. Only the **weighting** changes. [#7](https://github.com/MBehtemam/Montagent/issues/7) must re-score against general criteria rather than reading the scores here at face value.

---

## 1. The constraints being scored against

From the map:

| # | Constraint |
|---|---|
| **C1** | A single **declarative, inert (non-executing)** project file is the source of truth. Readable without evaluation. |
| **C2** | **Text and typography** — bilingual subtitle text, web fonts, multi-line layout. |
| **C3** | **Shapes and keyframe animation** expressible in data. |
| **C4** | **Single-frame render** is first-class (agent self-verification). |
| **C5** | **Partial-range render** (`--from`/`--to`) is first-class. |
| **C6** | **Budget** — 60s renders in under 2 min; 10s preview in under 5s. |
| **C7** | **Licensing** — usable without a commercial trap. |
| **C8** | **Project health and dependency weight.** |
| **C9** | **No GUI** — headless CLI, scriptable. |

Target format throughout: **1080x1920, 30fps**, images + narration audio + bilingual subtitle text.

---

## 2. Scorecard

Legend: **Y** meets it, **~** partial or with caveats, **N** does not.

| Candidate | C1 inert | C2 text | C3 anim | C4 still | C5 range | C6 speed | C7 licence | C8 health | C9 headless |
|---|---|---|---|---|---|---|---|---|---|
| **Remotion** | N (React code; `inputProps` is a data *seam*, not the truth) | Y (Chromium + CSS) | Y (`interpolate`/`spring`/CSS) | **Y** (`remotion still --frame`) | **Y** (`--frames=a-b`) | Not published | **~** free only under 4 employees | Y (very active) | Y |
| **Revideo** | N (generator functions) | Y (Chromium + CSS) | ~ (imperative tweens) | Not found | ~ (`renderPartialVideo`, worker-sharded) | "faster than realtime" only | **Y** MIT | **N** — team pivoted to commercial Midrender | Y |
| **Motion Canvas** | N (generator functions) | Y (DOM-measured) | ~ (imperative tweens) | ~ (GUI Snapshot only) | ~ (GUI Range only) | Not published | Y MIT | ~ (active repo, alpha tag) | **N** — no supported headless render |
| **Editly** | **~** JSON5 edit spec, but escapes to JS | ~ (canvas + `fontPath`) | ~ (fixed transitions + Ken Burns) | **N** | **N** (only per-clip `cutFrom/cutTo`) | Not published | Y MIT | **N** (RC only, stalled since 2025) | Y |
| **Headless Chrome + FFmpeg** | **Y** — you define the format | Y (full browser) | Y (you define it) | **Y** (you own the frame loop) | **Y** | Not published | Y (FFmpeg LGPL/GPL) | Y (Chrome/CDP) | Y |
| **Skia bindings** | **Y** — you define the format | Y w/ right stack (see §5) | **N** built-in (immediate mode) | **Y** | **Y** | Micro-benchmarks only | Y (MIT / BSD-3) | Y | Y |
| **FFmpeg filtergraphs** (incumbent) | **~** a DSL string, not a document | **N** (no wrapping; escaping hell) | **N** (expression soup) | **Y** (`-frames:v 1`) | **Y** (`-ss`/`-to`) | Not published | ~ LGPL, GPL if x264 | Y | Y |
| **OTIO** | **Y** (JSON) | **N** (no text model) | **N** (opaque effects) | n/a — not a renderer | n/a | n/a | Y Apache-2.0 | Y (ASWF) | n/a |
| **FCPXML** | ~ (XML, but indirection) | **N** (Motion templates) | ~ (`keyframeAnimation`) | n/a — not a renderer | n/a | Apple terms | Y (Apple-maintained) | **N** (macOS/FCP) | n/a |

**The single most important cross-cutting finding:** *no candidate in this survey publishes a throughput figure for 1080x1920 at 30fps.* Not one. The closest published claims are Revideo's qualitative "almost always faster than real-time" and per-op canvas micro-benchmarks from the Node Skia bindings. This is precisely why [#6](https://github.com/MBehtemam/Montagent/issues/6) exists as a separate measurement ticket — the C6 column above cannot be filled from published sources at all.

**The second:** every *renderer* in this survey fails C1 (Remotion, Revideo, Motion Canvas) or fails C2/C3 (FFmpeg, Skia), while every candidate that passes C1 cleanly (OTIO, headless Chrome, Skia) is not a complete renderer. Nothing off the shelf is both an inert declarative format and a capable renderer. That gap is the shape of the project.

---

## 3. Browser-based compositors

### Remotion

- **Host language / runtime.** React components in TypeScript. Node ≥16 per the getting-started guide ([docs](https://www.remotion.dev/docs/)); v5.0 raises the minimum ([v5.0 migration](https://www.remotion.dev/docs/5-0-migration)); `@remotion/media-parser` needs Node ≥20 ([runtime support](https://www.remotion.dev/docs/media-parser/runtime-support)).
- **Runtime dependencies.** Auto-installs **Chrome Headless Shell** into `node_modules/.remotion/chrome-headless-shell/` (pinned build, e.g. 149.0.7790.0 as of 4.0.452); `npx remotion browser ensure` verifies it ([Chrome Headless Shell](https://www.remotion.dev/docs/miscellaneous/chrome-headless-shell)). Linux needs extra shared libraries ([Linux dependencies](https://www.remotion.dev/docs/miscellaneous/linux-dependencies)). Requires ffmpeg/ffprobe ≥4.1 and auto-installs if missing **[secondary]**.
- **C1 — inert file: NO.** A composition *is* a React component tree. `calculateMetadata()` is a callback that can transform props and metadata at render time ([calculateMetadata](https://www.remotion.dev/docs/calculate-metadata)). Knowing what is on screen at t=6s requires executing the tree. This is the exact disease the map names.
  - **But there is a real data seam.** `getInputProps()` reads CLI `--props` JSON at render time; input props must be JSON-serializable; a Zod `z.object()` schema can be attached to a composition to validate them ([schemas](https://www.remotion.dev/docs/schemas), [getInputProps](https://www.remotion.dev/docs/get-input-props)). A Montagent project file could be passed wholesale as input props to a *fixed, general* Remotion composition that interprets it. The inert file would then be Montagent's, and Remotion would be a rendering backend rather than the authoring format. This is an architectural option worth naming, not a property Remotion has out of the box.
- **C2 — text.** Chromium's layout engine: full CSS/DOM text layout, line wrapping, bidi, everything a browser does. `@remotion/google-fonts` loads Google Fonts without hand-written CSS; manual `FontFace` loading pairs with `delayRender()`/`continueRender()`. From v2.2 Remotion automatically waits for CSS-imported fonts to load, which defuses the classic font-load race **[secondary]**. Best-in-survey for bilingual text.
- **C3 — animation.** `interpolate()` with `extrapolateLeft/Right: 'clamp'` and easing ([interpolate](https://www.remotion.dev/docs/interpolate)); `spring()` for physics-based 0→1 ([spring](https://www.remotion.dev/docs/spring)); `Easing` module ([easing](https://www.remotion.dev/docs/easing)); `@remotion/shapes` for SVG shape generation ([shapes](https://www.remotion.dev/docs/shapes)). Plus all of CSS and SVG.
- **C4 — single frame: YES, first-class.** `npx remotion still <serve-url> [composition-id] [output]` with `--frame`, `--image-format` (PNG/JPEG), `--scale`, `--timeout` (default 30000ms for `delayRender`) ([CLI still](https://www.remotion.dev/docs/cli/still)).
- **C5 — partial range: YES, first-class.** `--frames=a-b` inclusive; comma-separated ranges concatenate, e.g. `--frames=0-99,150-199`, and mixed forms like `--frames=0,30-59,90-` work. Available from **4.0.502**. `--sequence` emits an image sequence instead of a video ([CLI render](https://www.remotion.dev/docs/cli/render)).
- **C6 — speed: NOT PUBLISHED.** No fps or render-time figures found in the official docs. Remotion ships `npx remotion benchmark` so you measure your own ([CLI benchmark](https://www.remotion.dev/docs/cli/benchmark)); Lambda concurrency docs discuss `framesPerLambda` trade-offs qualitatively **[secondary]** but state no numbers.
- **C7 — licence: NOT MIT. The significant catch in this survey.** Per [LICENSE.md](https://github.com/remotion-dev/remotion/blob/main/LICENSE.md): the **Free License** covers individuals, for-profit orgs with **up to 3 employees**, non-profits, and orgs evaluating Remotion — commercial use permitted, but you may not resell/relicense/sublicense a derivative of Remotion itself. A **Company License** is required for for-profit orgs of 4+ people. Pricing at [remotion.pro/license](https://www.remotion.pro/license): "Remotion for Automators" $0.01/render with a $100/month minimum; "Remotion for Creators" $25/month/seat; Enterprise from $500/month. LICENSE.md notes the licence "will slightly change" in v5.0, linking [PR #3750](https://github.com/remotion-dev/remotion/pull/3750) — a pending change, not yet reviewed here.
  - Two flags for #7. First, the **"may not resell a derivative"** clause deserves reading closely if Montagent is ever distributed as a tool that embeds Remotion — that is a different posture from using it to render your own channel's videos. Second, the per-render Automators tier is a *usage-metered* model, which interacts badly with a design where a render is cheap and frequent (single-frame self-verification on every agent turn).
- **C8 — health: excellent.** npm `remotion` **4.0.520**, published 2026-09-01 ([registry](https://registry.npmjs.org/remotion)). GitHub `remotion-dev/remotion`: **58,098 stars**, last push 2026-09-02 (same day as this research), not archived. Company-backed by Remotion GmbH.
- **C9 — headless: yes**, CLI-driven throughout.
- **Fit for this project's shape.** `@remotion/captions` provides `createTikTokStyleCaptions()`, grouping caption tokens into pages via `combineTokensWithinMilliseconds` — low values give word-by-word TikTok-style display ([createTikTokStyleCaptions](https://www.remotion.dev/docs/captions/create-tiktok-style-captions), [API](https://www.remotion.dev/docs/captions/api)). Images and audio are `<Img>`/`<Audio>`; 9:16 is just composition width/height. This is the most directly on-target feature set in the survey.

### Revideo

A fork of Motion Canvas adding headless rendering, audio and a library-first API.

- **Host language / runtime.** TypeScript, Node ≥16 ([installation-and-setup.mdx](https://raw.githubusercontent.com/midrender/revideo/main/packages/docs/src/content/guide/installation-and-setup.mdx)).
- **Execution model.** Frames are drawn to an HTML `<canvas>` in a headless Chromium and encoded via the browser's **WebCodecs `VideoEncoder`** into a muted MP4; a separate Node backend runs FFmpeg only to extract and merge audio ([rendering-videos.mdx](https://raw.githubusercontent.com/midrender/revideo/main/packages/docs/src/content/guide/rendering-videos.mdx)). FFmpeg is **bundled** via `@ffmpeg-installer/ffmpeg` (v6) and `@ffprobe-installer/ffprobe` ([package.json](https://raw.githubusercontent.com/midrender/revideo/main/packages/ffmpeg/package.json)); overridable via `FFMPEG_PATH`. Linux users must `apt-get install nscd` to avoid an FFmpeg segfault with remote media URLs ([ffmpeg.mdx](https://raw.githubusercontent.com/midrender/revideo/main/packages/docs/src/content/common-issues/ffmpeg.mdx)) — a real native-dependency wart.
- **C1 — inert file: NO.** Scenes are **generator functions** passed to `makeScene2D()`, with each `yield`/`yield*` advancing frames ([understanding-scene-flow.mdx](https://raw.githubusercontent.com/midrender/revideo/main/packages/docs/src/content/guide/understanding-scene-flow.mdx)). State at t=6s exists only after stepping the generator.
  - **Data seam:** `useScene().variables.get('name', default)` reads project variables injected via `renderVideo({variables})` or `makeProject({variables})` ([parameterized-video.mdx](https://raw.githubusercontent.com/midrender/revideo/main/packages/docs/src/content/guide/parameterized-video.mdx)). Same architectural pattern as Remotion's input props, less developed.
- **C2 — text.** `<Txt fontFamily>`; web fonts via `@import url(...)` in a `global.css` imported into `project.ts`, or `@font-face` over files in `public/fonts/` ([custom-font.mdx](https://raw.githubusercontent.com/midrender/revideo/main/packages/docs/src/content/animations/custom-font.mdx)). **Documented font-load race:** adding a `<Txt>` node can create a promise because Revideo waits for `document.fonts.ready`; you must `yield` every `.add()` or you get "Tried to access an asynchronous property before the node was ready" ([scene-flow](https://raw.githubusercontent.com/midrender/revideo/main/packages/docs/src/content/guide/understanding-scene-flow.mdx)).
- **C3 — animation.** Motion-Canvas-style imperative tweens: call a property as a function with target + duration, `myCircle().fill('#e6a700', 1)`; `all()` parallel, `chain()` sequential. Components: `<Circle>`, `<Rect>`, `<Img>`, `<Video>`, `<Audio>`, `<Txt>`.
- **C4 — single frame: NOT FOUND.** No still-frame API located in the fetched docs. Recorded as *not confirmed*, not as *does not exist*.
- **C5 — partial range: partial.** `renderPartialVideo()` renders a chunk given a worker id and worker count — designed for sharding across processes or Lambda, not for "render me seconds 10 to 20". `settings.workers` parallelizes in-process ([rendering-videos.mdx](https://raw.githubusercontent.com/midrender/revideo/main/packages/docs/src/content/guide/rendering-videos.mdx)). Semantically this is parallelism, not the `--from`/`--to` the map asks for, though it could likely be bent to it.
- **C6 — speed.** The one quantitative-ish published claim in the whole survey: since v0.4.6 "rendering videos is almost always faster than real-time (meaning that rendering a 1-minute video takes less than one minute)" ([rendering-videos.mdx](https://raw.githubusercontent.com/midrender/revideo/main/packages/docs/src/content/guide/rendering-videos.mdx)). Relative, not absolute; no 1080x1920/30fps number. **Note this would satisfy the 60s-under-2min half of the budget if it holds, but says nothing about the 10s-preview-under-5s half, which is startup-dominated.**
- **C7 — licence: MIT, unrestricted.** [LICENSE](https://raw.githubusercontent.com/midrender/revideo/main/LICENSE) — "MIT License, Copyright (c) 2022 motion-canvas", standard text, no thresholds. Cleanest licence among the capable browser compositors.
- **C8 — health: the concern.** The repo **moved** from `redotvideo/revideo` to `midrender/revideo`. 4,019 stars, not archived, last push **2026-07-15** — ~7 weeks stale at time of research, against Remotion's same-day activity. **No GitHub Releases at all** (empty API list); latest npm `@revideo/core` is **0.11.0**, published 2026-07-10 ([registry](https://registry.npmjs.org/@revideo/core)). `re.video/docs` now 308-redirects to `midrender.com/revideo`, and the docs there state the team "now primarily works on Midrender" — a closed commercial editor built on the Revideo engine — and that "recent changes have not yet been upstreamed to the open-source repository" **[secondary]**. No archived/deprecated banner, but a pre-1.0 version number plus a stated pivot to a commercial product plus stalling commits is the classic signature of an OSS project going quiet.
- **C9 — headless: yes**, this is Revideo's whole reason for existing over Motion Canvas.
- **Shorts fit.** No first-party captions package; the pattern is demonstrated by the community example [redotvideo/examples/youtube-shorts](https://github.com/redotvideo/examples/tree/main/youtube-shorts), passing word-level timestamped subtitles in as project variables.

### Motion Canvas

- **Host language / runtime.** TypeScript library plus a **browser-based editor** built on Vite; Node ≥16 ([quickstart.mdx](https://github.com/motion-canvas/motion-canvas/blob/main/packages/docs/docs/getting-started/quickstart.mdx)).
- **C1 — inert file: NO,** and emphatically so. The docs' own words: "scenes are declared using generator functions - they serve as a description of how the animation should play out. By yielding different instructions we can tell the scene animation to do different things" ([quickstart.mdx](https://github.com/motion-canvas/motion-canvas/blob/main/packages/docs/docs/getting-started/quickstart.mdx)). Scene logic is arbitrary imperative TS executed step by step. A data file could at best be *read by* a generator; the generator must be code.
- **C2 — text.** `Txt`/`TxtLeaf` measure text with a hidden DOM `<span>` and `document.createRange()`, then draw via `CanvasRenderingContext2D` using `context.font`/`context.measureText` ([TxtLeaf.ts](https://github.com/motion-canvas/motion-canvas/blob/main/packages/2d/src/lib/components/TxtLeaf.ts)). Genuinely DOM-measured layout plus canvas drawing, so CSS `@font-face` web fonts work.
- **C3 — animation.** Reactive **signals**; tweens created by calling a property as a setter with a duration — `myCircle().fill('#e6a700', 1).to('#e13238', 1)` — composed with `all()`. Imperative code, no keyframe data schema.
- **C4 / C5 — single frame and range: GUI-only.** Rendering is driven from the browser editor's Video Settings panel via a RENDER button, output to `/output` ([rendering/index.mdx](https://github.com/motion-canvas/motion-canvas/blob/main/packages/docs/docs/getting-started/rendering/index.mdx)). A **Range** control renders a sub-span ("If set to a smaller range than the entire length of the video, it will only render a portion of the frames"), and a **Snapshot** camera icon exports a single still to `/still` — same doc. So both capabilities *exist* but only behind the GUI, which C9 rules out.
- **C9 — headless: NO, and this is disqualifying against the map's no-GUI principle.** There is no documented CLI or headless render path. Issue [#415](https://github.com/motion-canvas/motion-canvas/issues/415) ("Render projects headlessly", opened 2023) and issue [#1218](https://github.com/motion-canvas/motion-canvas/issues/1218) ("Headless rendering without browser for automated pipeline") are both still open — the latter carrying an unanswered "Did you ever figure this out?" comment dated 2026-04-14. Maintainer `aarthificial` said in 2023 it would ship as a separate opt-in package to avoid forcing headless Chrome on ordinary users; that was never completed. The internal `e2e` package does drive the `Renderer` class with Puppeteer, but that is not a supported public API.
- **Video output.** Default exporter emits an image sequence; the optional `@motion-canvas/ffmpeg` package produces finished video and installs FFmpeg automatically — "You do not need to install FFmpeg yourself" ([video.mdx](https://github.com/motion-canvas/motion-canvas/blob/main/packages/docs/docs/getting-started/rendering/video.mdx)).
- **C6 — speed: NOT PUBLISHED.**
- **C7 — licence:** MIT ([LICENSE](https://github.com/motion-canvas/motion-canvas/blob/main/LICENSE)).
- **C8 — health.** 19,032 stars, 173 open issues, not archived, last push 2026-07-02. Latest tag is `v3.18.0-alpha.0` from 2025-02-16 — still a prerelease; `packages/core` reads 3.17.2. Repo receives commits, but a 2023 headless-rendering request unresolved in 2026 is a fair read on priorities for the one feature this project needs most.

---

## 4. Editly

- **Host language / runtime.** Node.js, **ESM-only**. README states Node "v12.16.2 or newer; latest LTS recommended". **FFmpeg and FFprobe must be installed externally** and on `PATH` — not bundled ([README](https://github.com/mifi/editly)).
- **Native dependency weight — the heaviest in the survey.** `gl` (^8.1.6, headless-gl), `gl-shader`, `gl-texture2d`, `gl-buffer`, `gl-transition`/`gl-transitions`, `canvas` (^2.11.2, node-canvas), and `fabric` (^6.5.4) ([package.json](https://github.com/mifi/editly/blob/master/package.json)). On Linux headless-gl needs extra system libraries. Two native compilation targets plus an external FFmpeg is a rough install story.
- **C1 — inert file: the closest thing to a match among the renderers, but leaky.** The edit is a **JSON5 (or plain JS) "edit spec"** — clips, layers, transitions, audio tracks, output params — passed to CLI or Node API. That part is genuine declarative data. **But** it explicitly permits arbitrary JavaScript: `type: 'canvas'` layers take a `func` callback for custom drawing, `type: 'fabric'` layers likewise, and `type: 'gl'` layers take custom GLSL shaders ([README](https://github.com/mifi/editly)). The moment any of those appear, the spec stops being JSON-serializable and stops being inert. Editly demonstrates that a declarative video spec is workable while also demonstrating the exact failure mode the map's "inert data, no evaluation" principle guards against: an escape hatch to code that becomes the path of least resistance.
- **C2 — text.** Title/subtitle layers render through **node-canvas**, not FFmpeg drawtext; fonts selected via a `fontPath` pointing at a `.ttf`, with a system-font fallback ([README](https://github.com/mifi/editly)). Workable but thin — no documented rich typography or styled-run model for bilingual text.
- **C3 — animation.** A **fixed menu**, not a general model: transitions from the gl-transitions gallery plus built-in `directional-left/right/up/down`, `random`, `dummy`; and Ken Burns via per-clip `zoomDirection` (`in`/`out`/`left`/`right`/`null`) and `zoomAmount` (default `0.1`) ([README](https://github.com/mifi/editly)). No arbitrary keyframing of arbitrary properties.
- **C4 — single frame: NO.** No documented single-output-frame capability.
- **C5 — partial range: NO.** Per-clip **source** trimming exists via `cutFrom`/`cutTo`, but with a surprising semantic — "If cutFrom/cutTo is set, the resulting segment (cutTo-cutFrom) will be slowed/sped-up to fit clip.duration" ([README](https://github.com/mifi/editly)) — that is time-stretching, not the timeline-range render the map wants. The only preview affordance is `--fast`/`-f`, "Fast mode (low resolution and FPS, useful for getting a quick preview)", which trades quality across the whole video rather than rendering a range. **Both C4 and C5, which the map calls first-class, are absent.**
- **C6 — speed: NOT PUBLISHED.**
- **C7 — licence:** MIT ([LICENSE](https://github.com/mifi/editly/blob/master/LICENSE), (c) 2020 Mikael Finstad).
- **C8 — health: weak.** 5,482 stars, 80 open issues, not archived. Last push 2025-05-12; last commit 2025-02-20. Latest release is **`v0.15.0-rc.1`** from 2025-01-19 — a *release candidate*, whose own notes read: "📢 After a couple years of inactivity, editly now has a new maintainer and a new release (candidate)" ([release](https://github.com/mifi/editly/releases/tag/v0.15.0-rc.1)). So: multi-year dormancy, a 2025 revival that never reached a stable release, and no activity in the ~16 months since. Two dormancy periods is a pattern.

---

## 5. Build-your-own substrates

These are not products; they are the materials you would use if Montagent renders frames itself. They score perfectly on C1, C4 and C5 for the trivial reason that *you* define the file format and *you* own the frame loop — the cost is that C2 and C3 become your problem.

### Headless Chrome + FFmpeg muxing

The DIY version of what Remotion and Revideo package.

- **Deterministic virtual time.** CDP `Emulation.setVirtualTimePolicy` "Turns on virtual time for all frames (replacing real-time with a synthetic time source)", with modes `advance`/`pause`/`pauseIfNetworkFetchesPending`, an optional `budget` in ms that fires `virtualTimeBudgetExpired`, and `maxVirtualTimeTaskStarvationCount` to avoid deadlock ([CDP Emulation](https://chromedevtools.github.io/devtools-protocol/tot/Emulation/#method-setVirtualTimePolicy)). This is what makes render output frame-exact and reproducible rather than wall-clock-dependent.
- **Frame-accurate capture.** CDP `HeadlessExperimental.beginFrame` "Sends a BeginFrame to the target and returns when the frame was completed. Optionally captures a screenshot from the resulting frame." Requires a target created with BeginFrameControl, is intended to be paired with the `--run-all-compositor-stages-before-draw` Chrome flag, takes `frameTimeTicks`, `interval` (default ~16.666ms = 60fps), `noDisplayUpdates`, and an optional `screenshot` param; returns `hasDamage` and base64 `screenshotData` ([CDP HeadlessExperimental](https://chromedevtools.github.io/devtools-protocol/tot/HeadlessExperimental/#method-beginFrame)).
- **Higher-level alternative.** Puppeteer `page.screenshot()` returns base64 or `Uint8Array` depending on `encoding` ([pptr.dev](https://pptr.dev/api/puppeteer.page.screenshot)) — but it wraps `Page.captureScreenshot` and is wall-clock, so a serious pipeline drops to raw CDP for the two primitives above.
- **Muxing.** FFmpeg's `pipe` protocol reads via `pipe:[fd]` (stdin default); pipes are non-seekable, so formats needing seekable output (e.g. MOV) can't be written that way ([ffmpeg-protocols](https://ffmpeg.org/ffmpeg-protocols.html#pipe)). The `image2` demuxer reads image sequences with a `framerate` option (default 25) and `pattern_type`; all images must share size, pixel format and file format ([ffmpeg-formats](https://ffmpeg.org/ffmpeg-formats.html#image2-1)). `-f image2pipe -i -` is the standard way to stream PNG frames from a capture loop into FFmpeg.
- **Scores.** C1 **Y** (format is yours), C2 **Y** (a full browser — same typography ceiling as Remotion), C3 **Y** (yours to define), C4/C5 **Y** (you drive the frame loop, so "render frame 180" and "render frames 300-600" are the same code path as anything else), C7 **Y**, C9 **Y**. C6 **not published** — no CDP or Puppeteer page states any capture-throughput figure. C8: the dependency is Chrome itself, which is healthy, but `HeadlessExperimental` is a *experimental* CDP domain by name and the maintenance burden is entirely yours.
- **The honest trade.** This is Remotion minus the licence, minus the ecosystem (`@remotion/captions`, `@remotion/shapes`, the font-loading race fixes), plus a substantial amount of work you own forever. The map's preview budget (10s in under 5s) makes browser startup cost the crux, and that is exactly the part a DIY pipeline must solve itself.

### Skia bindings

**Rust:**

- **`skia-safe` (rust-skia).** Bindings to Google Skia. Prebuilt binaries auto-download for most platform/feature combos; unsupported configs build from source needing LLVM/Clang, Python 3, Ninja and a Skia checkout (tracks `chrome/m153`) ([repo](https://github.com/rust-skia/rust-skia)). No dedicated text-shaping module is called out in the README. Ships **Skottie/Lottie** support — Skia's declarative animation player. MIT ([crates.io](https://crates.io/api/v1/crates/skia-safe)); latest **0.99.0**, 2026-06-19.
- **`tiny-skia`.** Pure Rust, a subset of Skia, no external deps, compiles in under 5s, ~200KiB binary growth. **Explicitly has no text rendering**: "The main missing feature is text rendering", listed under "Out of scope" ([repo](https://github.com/linebender/tiny-skia)). Pure path/gradient/pattern rasterizer. BSD-3-Clause; latest **0.12.0**, 2026-02-02 ([crates.io](https://crates.io/api/v1/crates/tiny-skia)). **Disqualified alone on C2** — subtitles are the point of this project.
- **The text stack you would have to assemble.** `resvg`/`usvg` render SVG on top of tiny-skia and note "No native text rendering" in some build configs by design, relying on `fontdb` for font enumeration/matching and `rustybuzz` (pure-Rust HarfBuzz port) for shaping ([resvg](https://github.com/linebender/resvg)). resvg **0.48.1** (2026-08-02, Apache-2.0 OR MIT); `rustybuzz` **0.20.1** (2024-11-12, MIT); `fontdb` **0.24.0** (2026-07-29, MIT). The strongest option is **`cosmic-text`** — "Pure Rust multi-line text handling", full shaping via HarfRust, **bidi**, per-character and per-line font fallback, and multiple wrap modes (simple, indented, none, ellipsize), loading fonts through `fontdb` ([repo](https://github.com/pop-os/cosmic-text)). Apache-2.0 OR MIT, **0.19.0** (2026-04-22), maintained by System76. For bilingual subtitles with mixed scripts, `cosmic-text`'s bidi and font-fallback are the features that matter.

**Node:**

- **`skia-canvas`.** Native N-API addon (N-API v8, Node ≥12.22) wrapping Skia in Rust; prebuilt binaries auto-download for Linux (glibc 2.28+, musl/Alpine), macOS, Windows arm64/x64 ([repo](https://github.com/samizdatco/skia-canvas)). **The best text story of the Node bindings:** multi-line **word-wrapped** text, line-by-line metrics, small-caps/ligatures/OpenType features, proportional letter/word spacing and leading, variable fonts, and loading non-system fonts from local files. Exports sync or async to PNG/JPEG/WEBP/PDF/SVG, including numbered image-sequence export. MIT; **3.0.8**, 2025-09-25 ([npm](https://registry.npmjs.org/skia-canvas)). Published micro-benchmarks: async export 28ms vs 137ms serial for a Bezier render, 4ms vs 21ms for a text render — *the project's own micro-benchmarks, not a 1080x1920/30fps figure*.
- **`@napi-rs/canvas`.** Node-API bindings to Skia; prebuilt binaries for x64/arm64/armv7, "zero system dependencies" for consumers; source builds need Clang and glibc ≥2.18 ([repo](https://github.com/Brooooooklyn/canvas)). Fonts via `GlobalFonts.registerFromPath()`. **Line wrapping is not documented** — the caller does it with `measureText`, which is exactly the work you do not want to hand-roll for bilingual subtitles. MIT; **1.0.8**, 2026-08-24 ([npm](https://registry.npmjs.org/@napi-rs/canvas)). Published micro-benchmark (Apple M3 Max, draw-house-and-export-PNG): ~14.7ms/op (68 ops/sec) vs skia-canvas ~21.2ms vs node-canvas ~16.6ms — again a relative micro-benchmark, not a throughput figure. Maintainer LongYinan also co-maintains rust-skia.
- **`CanvasKit`** (Skia in WASM, npm `canvaskit-wasm`). Exposes an `SkParagraph`-based **paragraph shaping** API for multi-line styled text layout, and bundles **Skottie** for Lottie playback ([skia.org](https://skia.org/docs/user/modules/canvaskit/)). BSD-3-Clause; **0.42.0**, 2026-08-18 ([npm](https://registry.npmjs.org/canvaskit-wasm)).
- **Scores across all Skia bindings.** C1 **Y** and C4/C5 **Y** for the same reason as above — these are immediate-mode APIs, you own the frame loop, so a single frame and an arbitrary range are the natural unit of work. C2 ranges from **N** (tiny-skia) to **Y** (skia-canvas, cosmic-text, CanvasKit paragraph). **C3 is the hard N:** none of these provides a keyframe or interpolation timeline for arbitrary shapes. You would build easing, interpolation and a timeline model yourself. The one exception is **Skottie** (in skia-safe and CanvasKit), which *is* a declarative animation player — but it consumes **Lottie JSON**, an After Effects export format, which is a different bet entirely and was not in scope for this ticket. C6 **not published** at 1080x1920/30fps for any of them.

---

## 6. FFmpeg filtergraphs — the incumbent baseline

This is what the project uses today via agent-written throwaway Python. Understanding precisely *why* it hurts is the most useful output of this section.

- **Licence.** Base is **LGPL v2.1 or later**; several optional components (notably libx264) are GPL v2-or-later, and "If those parts get used the GPL applies to all of FFmpeg" ([ffmpeg.org/legal](https://ffmpeg.org/legal.html)). Since H.264 output is the point, the practical licence is **GPL**.
- **C2 — text: the core pain.**
  - `drawtext` needs a build with `--enable-libfreetype --enable-libharfbuzz`; the `font` option and default-font fallback need `--enable-libfontconfig`; `text_shaping` needs `--enable-libfribidi` ([ffmpeg-filters](https://ffmpeg.org/ffmpeg-filters.html#drawtext-1)). So text capability depends on how the local FFmpeg binary was compiled — a portability hazard.
  - `text_shaping`: "If set to 1, attempt to shape the text (for example, reverse the order of right-to-left text and join Arabic characters) before drawing it... By default 1 (if supported)". The "(if supported)" is doing real work for a bilingual project.
  - **No automatic line wrapping exists.** Only `line_spacing` (pixel gap between explicit lines) and `tabsize`. Line breaks must be inserted manually into the text. For subtitles this means the caller computes wrapping — with no text metrics available to it.
  - **Escaping is documented as three-level and is genuinely awful.** Level 1 escapes `'` and `:` inside the option value; level 2 additionally escapes `,` when embedded in a filtergraph; level 3 escapes again per the shell's rules. FFmpeg's own example: `-vf "drawtext=text=this is a \\\\\\'string\\\\\\'\\\\: may contain one\\, or more\\, special characters"` ([Notes on filtergraph escaping](https://ffmpeg.org/ffmpeg-filters.html)). The docs themselves recommend `textfile` over inline `text` to escape the escaping. **For an agent generating bilingual subtitle text mechanically, this is a correctness minefield.**
  - The better path is `subtitles`/`ass` via **libass** (needs `--enable-libass`): options include `force_style` ASS overrides, `wrap_unicode` (Unicode Line Breaking Algorithm, needs libass ≥0.17.0 with libunibreak, on by default except native ASS), and `shaping` (`auto`/`simple`/`complex`, where complex — required for Arabic/Hebrew/Devanagari/Thai — needs libass built with HarfBuzz) ([subtitles](https://ffmpeg.org/ffmpeg-filters.html#subtitles), [ass](https://ffmpeg.org/ffmpeg-filters.html#ass-1)). This works, but it means the real text format is ASS, and Montagent's project file would be generating a second file format as an intermediate.
- **C3 — animation: expression soup.**
  - Timeline gating via `enable='between(t,10,3*60)'` on filters supporting it ([timeline editing](https://ffmpeg.org/ffmpeg-filters.html)).
  - `drawtext` `x`/`y` accept expressions over `t`, `n`, `w`/`h`, `text_w`/`text_h`.
  - `zoompan` takes `zoom`/`z`, `x`, `y`, `d` as expressions: `zoompan=z='min(zoom+0.0015,1.5)':d=700:x='iw/2-(iw/zoom/2)':y='ih/2-(ih/zoom/2)'` ([zoompan](https://ffmpeg.org/ffmpeg-filters.html#zoompan)) — that is a Ken Burns move, and it is already unreadable.
  - `xfade` gives a fixed set of named transitions plus a `custom` per-pixel expression mode ([xfade](https://ffmpeg.org/ffmpeg-filters.html#xfade)).
  - `sendcmd`/`asendcmd` provide an interval DSL — `START[-END] [FLAGS] TARGET COMMAND ARG;` — to change filter parameters at times or frame events ([sendcmd](https://ffmpeg.org/ffmpeg-filters.html#sendcmd_002c-asendcmd)).
  - **There is no keyframe or curve model.** Anything beyond single-parameter easing becomes long single-line math expressions or chained `sendcmd` intervals. Multi-property choreography degrades into strings no one — human or agent — can read back.
- **C4 — single frame: YES.** `-frames:v 1` — "Stop writing to the stream after framecount frames" ([ffmpeg.html §5.5](https://ffmpeg.org/ffmpeg.html)) — with the image2 muxer's `update` option: "If set to 1, the filename will always be interpreted as just a filename, not a pattern, and the corresponding file will be continuously overwritten" ([ffmpeg-formats](https://ffmpeg.org/ffmpeg-formats.html#image2_002c-image2pipe)).
- **C5 — partial range: YES.** `-ss` as an input option seeks to the nearest keyframe before the position then decodes and discards the gap with `-accurate_seek` (default on); as an output option it discards decoded frames until the timestamp. `-to`/`-t` stop at an absolute position or duration, mutually exclusive with `-t` taking priority. `-copyts` keeps input timestamps unsanitized, paired with `-start_at_zero` ([ffmpeg.html §5.4](https://ffmpeg.org/ffmpeg.html)). The `trim`/`atrim` filters take `start`/`end`/`start_pts`/`end_pts`/`duration`/`start_frame`/`end_frame` and do not reset timestamps — chain `setpts` after ([trim](https://ffmpeg.org/ffmpeg-filters.html#trim)). **Note the input-vs-output seeking distinction is itself a classic source of off-by-frame errors.**
- **C1 — inert file: a DSL string, not a document.** A filtergraph is a single-line DSL passed via `-vf`/`-filter_complex`, with the three-level escaping above. It *can* be stored externally — `ffmpeg -i INPUT -/filter:v filter.script OUTPUT` "will load a filtergraph description from the file named filter.script" ([per-option syntax](https://ffmpeg.org/ffmpeg.html)), the modern form of what was `-filter_complex_script`. But externalizing it does not make it structured, diffable, or queryable. You cannot ask "what is on screen at 6s" of a filtergraph without simulating it — which is the same disease as executing React, in uglier syntax.
- **C6 — speed: NOT PUBLISHED.** No official 1080x1920/30fps throughput figure exists in the FFmpeg documentation; those pages document behavior and options, not performance.
- **Verdict as baseline.** FFmpeg wins C4 and C5 outright and loses C1, C2 and C3 badly. Every candidate in this survey is, in effect, a proposal for what to put *on top of* FFmpeg — note that Remotion, Revideo, Motion Canvas and Editly all still shell out to FFmpeg for encoding. **FFmpeg is not really a competitor to the others; it is the encode layer underneath all of them.** The question is only what generates the frames.

---

## 7. Interchange formats as prior art for the project file

Neither OTIO nor FCPXML renders anything. They are in this survey as **prior art for the file format**, and that is how they are read here.

### OpenTimelineIO

- **What it is.** "An API and interchange format for editorial cut information", like "a modern Edit Decision List (EDL) that also includes an API for reading, writing, and manipulating editorial data" ([docs](https://opentimelineio.readthedocs.io/en/latest/index.html)). **It renders nothing:** it "supports clips, timing, tracks, transitions, markers, metadata, etc. but not embedded video or audio. Video and audio media are referenced externally" (same page). To get pixels you need an external renderer that resolves `target_url` and interprets the edit graph.
- **Runtime.** C++ core with Python bindings, `pip install opentimelineio` ([quickstart](https://opentimelineio.readthedocs.io/en/latest/tutorials/quickstart.html)); targets Python 3.9–3.12 and VFX Reference Platform 2022–2025 ([repo](https://github.com/AcademySoftwareFoundation/OpenTimelineIO)). Default install ships only core adapters (`.otio` JSON, `.otiod`/`.otioz` bundles); EDL/AAF/ALE/FCPXML adapters need the separate `OpenTimelineIO-Plugins` package.
- **Data model.** Timeline → Stack → Track → {Clip, Gap, Transition}, with Markers. Track "holds composable children in sequence"; **Gap "is meant to be transparent"** — an explicit object, not implicit absence; Transition blends neighbours via `in_offset`/`out_offset` ([timeline structure](https://opentimelineio.readthedocs.io/en/latest/tutorials/otio-timeline-structure.html)).
- **Time model.** `RationalTime(value, rate)` — `RationalTime(7,24)` is frame 7 at 24fps; `TimeRange` is `start_time` + `duration`, both `RationalTime` (same page). No floats.
- **Serialization and versioning.** `.otio` files are **indented, human-readable JSON** (gzip recommended over minification for size). Every object carries `"OTIO_SCHEMA"` with an embedded integer version — `"Timeline.1"`, `"Clip.5"` ([file format spec](https://opentimelineio.readthedocs.io/en/latest/tutorials/otio-file-format-specification.html)). Upgrade functions are registered per (type, target version) and **chained automatically** on deserialize (1→2→3→4); downgrade functions run the inverse on serialize via `schema_version_targets`; label sets like `CORE_VERSION_MAP` keyed by release let plugins target a whole schema family ([versioning schemas](https://opentimelineio.readthedocs.io/en/latest/tutorials/versioning-schemas.html)).
- **Text: none.** No `Title`/`Text` class exists in the core schema. Anything textual lives in the generic `metadata` dict ([architecture](https://opentimelineio.readthedocs.io/en/latest/tutorials/architecture.html)).
- **Effects: opaque.** `Effect` exposes only `effect_name`, `name`, `enabled`, `metadata` — no parameter animation, no keyframes ([Python schema API](https://opentimelineio.readthedocs.io/en/latest/api/python/opentimelineio.schema.html)).
- **Ranges.** Every Clip has a `source_range`; `trimmed_range()`/`TimeRange` give exact rational addressing of any sub-range without rendering.
- **Licence:** Apache-2.0 ([LICENSE.txt](https://raw.githubusercontent.com/AcademySoftwareFoundation/OpenTimelineIO/main/LICENSE.txt)).
- **Health.** Latest release v0.18.1 ("Beta 18.1 – Python Wheel Fixes"), 2024-11-09 ([releases](https://github.com/AcademySoftwareFoundation/OpenTimelineIO/releases)). Under AcademySoftwareFoundation, ~2.0k stars, docs describe it as "a mature framework widely deployed" with a stable API. ASWF Landscape currently lists it as **Incubating** ([landscape](https://landscape.aswf.io/?item=aswf-projects--incubating--opentimelineio)); a 2019 TAC thread records a unanimous graduation motion recommended to the board ([TAC thread](https://lists.aswf.io/g/tac/topic/opentimelineio_aswf_project/32497878)), but since the landscape entry still says Incubating, **graduated status is not confirmed**.

**What to steal:**

- **Exact rational time with explicit rate everywhere** — `RationalTime(value, rate)` rather than floats. Frame-accurate, no accumulation error, and an agent reading `{"value": 180, "rate": 30}` knows exactly which frame that is. Directly relevant to [#8 "Decide the time model"](https://github.com/MBehtemam/Montagent/issues/8).
- **Gaps as explicit first-class objects.** Silence and blank space are things you can point at, name and edit — not the absence of a thing. Enormously easier for an agent to reason about and to diff.
- **Per-object embedded schema version** (`"OTIO_SCHEMA": "Clip.5"`) with automatic chained upgrades. A versioning story that lives in the file rather than in a sidecar.
- **Plain indented human-readable JSON as the canonical form**, explicitly choosing gzip over minification when size matters — i.e. never trading readability for bytes.
- **Clean composition primitives** (Track/Stack/Clip/Gap/Transition) that map onto how anyone actually thinks about an edit.
- **One namespaced `metadata` escape hatch per object** so unknown fields never break the schema.

**What makes it a poor fit for hand-editing by an agent:**

- **No instancing** — "OTIO does not support instancing"; reused content is fully duplicated in the JSON ([file format spec](https://opentimelineio.readthedocs.io/en/latest/tutorials/otio-file-format-specification.html)). For a file the agent edits by hand, duplication means drift: change one copy, forget the other. (Note the map already accepts a version of this cost by deferring templating.)
- **Effects and Markers are opaque bags.** With only `effect_name` + `metadata` and no standard parameter vocabulary, all real animation becomes ad hoc tool-specific metadata that no agent can reason about generically. For a project whose whole point is expressive text and shape animation, this is the fatal gap — and it is why OTIO cannot be adopted wholesale as the project format.
- **No text or typography model at all.** Titles must be smuggled through `metadata` — untyped indirection, exactly what a hand-editable format should avoid.
- **The versioning machinery needs a library.** Upgrade-function chains, label sets and `plugin_manifest.json` are powerful but mean you cannot safely hand-write or hand-migrate a file without the tooling. A format an agent edits with ordinary file tools should be writable correctly *by reading the schema alone*.

### FCPXML

- **What it is.** Apple: documents that describe "the data your app or workflow extension exchanges with Final Cut Pro" — assets, projects and metadata in; rendered media, timeline sequences and editing decisions out. And explicitly: "FCPXML describes certain, but not all, aspects of projects and events useful for other applications. It is **not a substitute for native Final Cut Pro project and event data** organized in a library bundle" ([FCPXML Reference](https://developer.apple.com/tutorials/data/documentation/professional-video-applications/fcpxml-reference.json)). Pixels require Final Cut Pro plus the referenced media plus any Motion templates.
- **Runtime.** None of its own — plain XML validated against a DTD, consumed by importing into Final Cut Pro. "Even though an imported FCPXML document matches this DTD, import errors may still occur due to invalid data. If the document does not match the DTD, Final Cut Pro X rejects the import operation completely" ([FCPXML v1.7 DTD](https://developer.apple.com/library/archive/documentation/Miscellaneous/Conceptual/LegacyDTDsFinalCutPro/FCPXMLDTDv1.7/FCPXMLDTDv1.7.html)). macOS/FCP-bound by nature.
- **Structure.** Root `<!ELEMENT fcpxml (import-options?, resources?, (library | event* | (%event_item;)*))>` with `version` fixed per DTD. `resources` holds shared ID-keyed objects (`format`, `asset`, `effect`) referenced by `IDREF` — the DTD calls these "project elements potentially referenced by other project elements". Hierarchy: `library` → `event*` → `project` → `sequence` → `spine`. `asset-clip` references an `asset` by `ref`, with `start`/`duration`/`offset` as `%time;` values (same DTD). As of FCPXML 1.9 only `resources` is strictly required at root; organizational elements became optional ([FCPXML overview](https://developer.apple.com/tutorials/data/documentation/professional-video-applications/fcpxml.json)).
- **Time.** Rational seconds as strings — `"1001/30000s"` for the NTSC frame duration, or `"5s"`. DTD comment: "'time' attributes are expressed as a rational number of seconds" (same DTD).
- **Text.** `<!ATTLIST title ref IDREF #REQUIRED>` — a title *must* reference an `effect` resource that is a **Motion template**. Text lives in `text`/`text-style-def`/`text-style` children (`font`, `fontSize`, `fontColor`, `bold`, `italic`, `alignment`) applied *on top of* whatever the template defines. **The base styling and rendering behavior is not in the XML** (same DTD).
- **Keyframes.** `param` elements carry `keyframeAnimation` containing `keyframe` elements with `time`, `value`, `interp` (linear/ease/easeIn/easeOut) and `curve` (linear/smooth); `fadeIn`/`fadeOut` give built-in fades (same DTD). Expressible only in the context of a named `param` on a referenced effect — the XML records the animated *value*, never what it visually does.
- **Effects by opaque UID.** `<!ATTLIST effect uid CDATA #REQUIRED>` — "FCP-assigned unique ID for effect", separate from the local `id` used for IDREF wiring (same DTD).
- **Licensing / health.** No open-source licence; Apple documentation under Apple's site terms ("Copyright © 2026 Apple Inc. All rights reserved") ([overview](https://developer.apple.com/tutorials/data/documentation/professional-video-applications/fcpxml.json)). Apple documents FCPXML alongside current FCP releases (1.9 requires FCP 10.4.9+) and archives every legacy DTD from 1.0 onward. **Versions beyond 1.9 could not be confirmed from a primary Apple source** during this research; community sources mention 1.13/1.14 with Final Cut Pro 11 but were not verified, so treat the current version as not established here.

**What to steal:**

- **Explicit rational-second time strings** (`"1001/30000s"`) — unambiguous, and notably a *different* encoding of the same good idea as OTIO's `RationalTime`. Two independent formats converging on exact rational time is the strongest signal in this whole section for [#8](https://github.com/MBehtemam/Montagent/issues/8).
- **Separating shared `resources` from the timeline that references them.** Asset and format definitions are declared once and referenced by id, rather than repeated at every use — the fix for OTIO's no-instancing duplication problem.
- **The *shape* of `param` / `keyframeAnimation` / `keyframe`** — a generic vocabulary of (name, value, time, interp, curve). Even discarding Motion's semantics entirely, that tuple is a good starting vocabulary for [#12 "Design the animation model"](https://github.com/MBehtemam/Montagent/issues/12), and it is genuinely declarative data, unlike an FFmpeg expression string or a Remotion `interpolate()` call.
- **`start`/`duration`/`offset` on every clip** — simple, explicit, addressable.

**What makes it a poor fit for hand-editing by an agent:**

- **Two-level indirection into an opaque binary.** A `title` references an `effect` by `IDREF`, which carries an Apple-assigned `uid` pointing at a Motion template file. **None of the actual visual or text behavior is in the document.** An agent editing this file cannot know what the title will look like — the precise failure the map's "understandable by reading it" principle exists to prevent. This is worse than executable code: at least code can be read.
- **No self-contained styling model.** `text-style` only styles what the Motion template exposes, and the typography engine is macOS/FCP-only.
- **Brittle all-or-nothing validation.** Import "fails completely" on any DTD mismatch — no partial success, no graceful degradation, no useful partial render. Compare the map's preference (schema catches malformed, agent catches wrong) which wants *actionable* errors.
- **Verbose XML requiring DTD knowledge to write.** Nested `%entity;` groups and clip-item unions (`%clip_item;`, `%anchor_item;`, `%marker_item;`), many optional attributes, and no self-describing schema. An agent must know the DTD out-of-band to produce a valid document — whereas JSON Schema can travel with the file.
- **No versioned upgrade mechanism** comparable to OTIO's. Compatibility is tied to which Final Cut Pro release is installed.

### Combined lesson for the project file

Read side by side, the two formats agree on the structural questions and both fail on the expressive ones:

| Question | OTIO | FCPXML | Lesson |
|---|---|---|---|
| Time | `RationalTime(value, rate)` | `"1001/30000s"` | **Both chose exact rational time.** Do the same. |
| Reuse | none (duplicates) | `resources` + `IDREF` | Take FCPXML's id/reference table. |
| Versioning | per-object schema version + upgrade chain | tied to FCP release | Take OTIO's idea, simplify so it stays hand-writable. |
| Gaps | explicit `Gap` object | implicit via `offset` | Take OTIO's explicit gaps. |
| Text | none | Motion template by UID | **Neither is usable. This is Montagent's to invent** ([#13](https://github.com/MBehtemam/Montagent/issues/13)). |
| Animation | opaque effects | `keyframeAnimation` | Take FCPXML's tuple shape, define real semantics ([#12](https://github.com/MBehtemam/Montagent/issues/12)). |
| Readability | indented JSON | verbose XML + DTD | Take OTIO's JSON posture. |

Both are *interchange* formats — designed to move an edit between applications that already know how to render, where the receiving app supplies all the meaning. Montagent's file is an *authoring* format that must carry its own meaning, because the thing reading it is an agent with no application-specific knowledge to supply. That difference explains every one of the "poor fit" points above.

---

## 8. Open items this survey could not close

1. **No throughput figures exist, anywhere, for any candidate at 1080x1920/30fps.** The C6 column is unfillable from published sources. [#6](https://github.com/MBehtemam/Montagent/issues/6) must measure.
2. **Revideo single-frame render** — no API found, but recorded as *not confirmed* rather than *absent*. Worth 10 minutes of source-reading before #7 decides.
3. **Remotion v5.0 licence change** ([PR #3750](https://github.com/remotion-dev/remotion/pull/3750)) not reviewed. If Remotion is a serious candidate, read it.
4. **Remotion's exact v5.0 minimum Node version** is templated as `<MinNodeVersion/>` in the migration doc and never resolves to a literal in the published page.
5. **FCPXML current version** beyond 1.9 not confirmed from an Apple primary source.
6. **OTIO ASWF maturity tier** — landscape says Incubating, a 2019 TAC motion recommended graduation. Cosmetic for our purposes.
7. Several Remotion claims are **[secondary]** (search summaries, because doc URLs 404'd on direct fetch): FFmpeg auto-install and version floor, the font-loading-race behavior from v2.2, and the Lambda concurrency discussion.
