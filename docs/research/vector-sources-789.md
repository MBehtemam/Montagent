# Vector sources: which renderer could draw SVG or Lottie on all six targets, CPU-only and byte-identical?

Research for [#789](https://github.com/MBehtemam/Montagent/issues/789), part of the map [#663](https://github.com/MBehtemam/Montagent/issues/663). It feeds the grilling [#790](https://github.com/MBehtemam/Montagent/issues/790), which reopens [ADR-0027](../adr/0027-vector-sources-out-of-scope.md). It builds on [#665](https://github.com/MBehtemam/Montagent/issues/665) (`docs/research/skia-reach/skia-safe-exposure.md` on branch `research/665-skia-safe-exposure`).

Nothing was built or run. This is a reading of published sources, crate manifests and source files, and CI metadata, done on 2026-10-07. Every version read is pinned in [Sources](#sources).

**Evidence labels**, as in #665:

- **Documented**: the owner's docs or headers say so.
- **Inferred from code**: read from the source, not run.
- **Measured**: someone ran it. Only Montagent's own earlier measurements, and CI timings published by GitHub Actions, fall here.
- **Not sourced**: looked for and not found.

## Answer in one table

| | Skia `svg` / `skottie` (rust-skia 0.153.2) | `resvg` 0.48.1 (SVG) | `velato` 0.12.0 (Lottie) | `rasterlottie` 0.2.4 (Lottie) | ThorVG 1.1.2 (Lottie, C++) | `rlottie` (C++) |
|---|---|---|---|---|---|---|
| All six targets, CPU-only | Only by source build or self-hosted archives. No published key covers six. Also turns on `textlayout`, which Montagent's pin test forbids | Pure Rust. Builds wherever Rust does (documented) | Pure Rust, but only renders through Vello (GPU) or your own `RenderSink` | Pure Rust, on `tiny-skia` 0.12 | C++, a meson build. Not a Cargo dependency | Archived, unmaintained |
| Byte-identical across painters | Unknown. Shapes are ordinary Skia draws | Inferred yes for shapes and gradients (no runtime CPU dispatch). Filters that use `powf`/`sin`/`cos` are weaker | Depends on the sink. Through a Skia sink, it is Skia draws | Not checked. It has an internal cache (`RefCell`) per prepared animation | Not checked. Has its own thread pool | n/a |
| Seek is a pure function of t | Inferred yes (Skottie) | n/a | Inferred yes: frame is an argument | Inferred yes from the API shape. The cache is not checked | **No guarantee**: `frame()` ignores a change of less than 0.001 | n/a |
| Licence | BSD-3 (Skia) | Apache-2.0 OR MIT | Apache-2.0 OR MIT | MIT OR Apache-2.0 | MIT | MIT, plus FTL, MPL and others |

**Short answers by bullet:**

1. **Skia features.** A source build costs about 18 to 47 minutes per target on GitHub-hosted runners, judged from rust-skia's own release jobs. rust-skia publishes a key when a downstream project asks for it, so a CPU-only `svg` key could be requested through a pull request. But `svg` and `skottie` both pull in `textlayout`, and Montagent's `skia_pin.rs` asserts `textlayout` is off, for an ADR reason that is separate from cost.
2. **`resvg`.** It covers static SVG 1.1 and part of SVG 2, with no animation or script. It is the strongest SVG candidate, and it already shares `tiny-skia` 0.12.0, `harfrust` 0.12.0 and `skrifa` 0.44.0 with Montagent's lockfile. Its "identical on every platform" claim is not tested byte-exact: its CI runs the tests on Linux only, with a tolerance of ±1 per channel.
3. **Other Lottie renderers.** `velato` (Linebender) can hand paths to any renderer through `RenderSink`, but has no text, image or dash support. `rasterlottie` is a young pure-Rust CPU renderer that only accepts a strict subset. ThorVG is a mature C++ CPU renderer. `rlottie` is archived.
4. **Lottie seeks.** Skottie: inferred pure in t. `velato`: pure by construction. ThorVG: documented to drop a frame change of less than 0.001, so its seeks depend on history.
5. **The size rule's inputs.** Skia gives 0×0 whenever a root dimension is a percentage, and ignores `viewBox`. `usvg` takes the size from `viewBox`, or falls back to 100×100. Lottie requires an integer `w` and `h`.
6. **Text.** SVG text needs fonts by family name plus shaping. Lottie text is outside the published Lottie spec (1.0.1). Both can be made deterministic only with vendored fonts and no system font lookup.
7. **Pre-rendering.** An SVG still needs no footage: it is one PNG in an `image` element. Animated Lottie becomes PNG-in-MOV (measured lossless by #723). The cost is a fixed resolution.
8. **The reference class.** Premiere imports neither SVG nor Lottie JSON. Lottie reaches Premiere only as MP4 or MOV through a LottieFiles extension. CapCut's published format lists name neither format. Adobe's official format page refused the fetch (HTTP 403).

---

## 1. Skia features: `svg` and `skottie` on six targets

### What is published (Documented, re-checked against #665)

These are the release assets of [`rust-skia/skia-binaries` 0.153.2](https://github.com/rust-skia/skia-binaries/releases/tag/0.153.2), listed again on 2026-10-07. The facts in #665 still hold:

- **CPU-only `svg`** (`jpegd-jpege-pdf-svg-textlayout-webpd-webpe`) is published for `aarch64-apple-darwin` and `x86_64-apple-darwin` only.
- Linux gnu (both architectures) and Windows msvc (both architectures) share one `svg` key: `ganesh-jpegd-jpege-pdf-svg-textlayout-vulkan-webpd-webpe`. It contains GPU code.
- **`skottie`** appears only in `egl-ganesh-gl-jpegd-jpege-pdf-skottie-svg-textlayout-vulkan-wayland-webpd-webpe-x11`, for `x86_64-unknown-linux-gnu` and `aarch64-unknown-linux-gnu`.

### Could a CPU-only key be published? (Documented)

- rust-skia generates its release matrix in [`mk-workflows/src/config.rs`](https://github.com/rust-skia/rust-skia/blob/0.153.2/mk-workflows/src/config.rs). Besides a base list, it has one function per downstream project that asked for binaries, each with a link to the request:
  - `skia_canvas_binaries_features` adds `textlayout,webp,svg` on macOS, which is where the CPU-only `svg` key comes from. It links to the [request in rust-skia#1068](https://github.com/rust-skia/rust-skia/pull/1068#issuecomment-2518894492).
  - There are similar functions for Freya, Vizia, Grida ([#1205](https://github.com/rust-skia/rust-skia/issues/1205)), Neovide and Slint.
- In a release workflow, the exact listed set is what gets published. `effective_features` adds platform features only for QA runs ([`mk-workflows/src/main.rs`](https://github.com/rust-skia/rust-skia/blob/0.153.2/mk-workflows/src/main.rs), "if we are releasing binaries, we want the exact set of features specified").
- **So the route to a CPU-only key is a pull request to `config.rs`** that adds, say, `textlayout,svg` (and `textlayout,skottie`) for every host. That would publish `jpegd-jpege-pdf-svg-textlayout` on all six targets in a later release. Whether upstream accepts it, and for which version, is **not sourced**.
- **The alternative is to self-host archives** via `SKIA_BINARIES_URL` ([`skia-bindings` README](https://github.com/rust-skia/rust-skia/blob/0.153.2/skia-bindings/README.md), cited in #665). Montagent would then build six archives itself.
- **Binary size.** The Slint entry in the same file says `textlayout` "embeds Skia's ICU data and adds about 10 MB to the resulting binary".

### What a source build costs per target (Measured by rust-skia's CI; not run here)

- **Requirements (documented).** [rust-skia's README](https://github.com/rust-skia/rust-skia/blob/0.153.2/README.md#building) requires LLVM/Clang, Python 3 and Ninja. Windows also needs Visual Studio 2022 Build Tools.
- **Build time (not documented).** The README says only that building Skia "takes a lot of time". As a stand-in, the table below gives the wall-clock time of rust-skia's own release jobs on GitHub-hosted runners. Each job builds one key from source and uploads it. The jobs run in a concurrent matrix, so read the times as an order of magnitude, not as a benchmark.

| Target | Key built | Job time | Run |
|---|---|---|---|
| Linux x86_64 | `svg,textlayout,vulkan,webp` | 44 min 51 s | [27671558692](https://github.com/rust-skia/rust-skia/actions/runs/27671558692) (2026-06-17) |
| Linux x86_64 | the `skottie` key above | 41 min 57 s | same |
| Linux x86_64 | `textlayout` only | 41 min 25 s | same |
| Linux aarch64 | `svg,textlayout,vulkan,webp` | 17 min 38 s | same |
| macOS aarch64 | `svg,textlayout,webp` (the CPU-only one) | 19 min 47 s | [33868798001](https://github.com/rust-skia/rust-skia/actions/runs/33868798001) (2026-09-04) |
| macOS x86_64 | `svg,textlayout,webp` | 7 min 07 s | same |
| Windows x86_64 | `ganesh,svg,textlayout,vulkan,webp` | 47 min 02 s | [33868798088](https://github.com/rust-skia/rust-skia/actions/runs/33868798088) (2026-09-04) |
| Windows aarch64 | `ganesh,svg,textlayout,vulkan,webp` | 37 min 50 s | [33868798199](https://github.com/rust-skia/rust-skia/actions/runs/33868798199) (2026-09-04) |

For comparison, ADR-0010 measured the prebuilt fetch at 15.8 s.

These runs are on rust-skia's `release` branch, and the Skia version each one built was not resolved. Treat the times as typical of the 0.15x line, not of 0.153.2 exactly.

### Two repo constraints the grilling meets (Documented, in this repo)

- **`textlayout` is forbidden for an ADR reason, not only a cost reason.** `crates/montagent-render/tests/skia_pin.rs` (`the_pinned_build_is_cpu_only`) asserts `textlayout` is off. ADR-0010 rules out SkParagraph, because ADR-0008 needs break opportunities and a named segmenter: "Taking it would ship two shapers, and the one `measure` reports would not be the one that draws."
  - [`binaries_config.rs`](https://github.com/rust-skia/rust-skia/blob/0.153.2/skia-bindings/build_support/binaries_config.rs) shows `textlayout` links `skparagraph`, `skshaper`, `skunicode_core` and `skunicode_icu`. It also shows `svg` adds `svg` and `skresources`, and `skottie` adds `skottie`, `sksg`, `jsonreader` and `skresources`.
  - #665 confirmed that `skia-bindings` declares `svg = ["textlayout"]`. So either Skia route reopens ADR-0008 and ADR-0010 as well as ADR-0027.
- **A per-target pin is guarded against.** #665 suggested a different published key per target. The same test file checks `[target.'cfg(...)'.dependencies]` tables, because "it is the shape a second pin would realistically take". This is flagged here, not decided.

## 2. `resvg`

All files are read at tag [`v0.48.1`](https://github.com/linebender/resvg/tree/v0.48.1) (`68b14c4c`). This is the latest release, published on crates.io on 2026-08-02.

### SVG coverage (Documented)

- **Scope.** The [README](https://github.com/linebender/resvg/blob/v0.48.1/README.md) says: "`resvg` aims to only support the static SVG subset; i.e. no `a`, `script`, `view` or `cursor` elements, no events and no animations". It has no plans for animation. SVG 2 support is "being worked on". SVG Tiny 1.2 is not supported.
- **Results chart.** The chart embedded in the README at v0.48.1 ([`.github/chart.svg`](https://github.com/linebender/resvg/blob/v0.48.1/.github/chart.svg)) is labelled resvg **0.45.1**. It plots, in label order:
  - resvg 0.45.1: 1520
  - Chrome 123: 1423
  - Firefox 124: 1385
  - Safari 17.3.1: 1341
  - librsvg 2.58.0: 1168
  - Inkscape 1.3.2: 985
  - Batik 1.17: 976
  - SVG.NET 3.2.3: 530
  - QtSvg 6.7.0: 591

  The axis runs to 1616. Mapping each value to its label by order is my reading of the SVG.
- **Feature table.** The per-feature [support table](https://linebender.org/resvg-test-suite/svg-support-table.html) is older still: it is for resvg 0.40.0. It shows 0% for `direction`, `font-size-adjust`, `glyph-orientation-*`, `unicode-bidi` and `enable-background`. It shows 94% for `clipPath` and 97% for `mask`.
- **Both coverage figures lag the crate.** No coverage figure for 0.48.1 itself was found.

### How its output would enter the painter (Inferred from code)

There are two routes, both exposed in the public API:

- **(a) As a raster.** [`resvg::render(tree, transform, &mut tiny_skia::PixmapMut)`](https://github.com/linebender/resvg/blob/v0.48.1/crates/resvg/src/lib.rs) fills a `tiny-skia` pixmap. A pixmap is "a container that owns premultiplied RGBA pixels" ([`tiny-skia` `pixmap.rs`](https://github.com/linebender/tiny-skia/blob/v0.12.0/src/pixmap.rs)). Skia could take those bytes as an RGBA8888 premultiplied image. That conversion step is inferred, not read.
  - The SVG is then a bitmap at one resolution, rasterised once for each size or transform it is drawn at.
- **(b) As paths handed to Skia.** `usvg` (the parser) depends only on `tiny-skia-path`, not on the rasteriser ([`crates/usvg/Cargo.toml`](https://github.com/linebender/resvg/blob/v0.48.1/crates/usvg/Cargo.toml)).
  - Its `Tree` holds groups, paths, fills, strokes, gradients, patterns, clip paths, masks and filters.
  - `Text::flattened()` gives text as path groups ([`tree/text.rs`](https://github.com/linebender/resvg/blob/v0.48.1/crates/usvg/src/tree/text.rs)).
  - The README: "you can easily write your own renderer on top of `usvg` using any 2D library of your liking."
  - **The catch:** SVG filters are not paths. resvg implements them on pixmaps ([`crates/resvg/src/filter/`](https://github.com/linebender/resvg/tree/v0.48.1/crates/resvg/src/filter)), so a Skia-side renderer would have to re-implement or refuse them.
  - **A related option, inferred and not checked:** since #768 the format has a `path` element. A skill could convert `usvg` paths into `path` elements at import time, so no vector source exists at render time.

**Repo constraint on route (a):** the workspace pins `tiny-skia = "=0.12.0"` with the comment "The named exit if the prebuilt story breaks (ADR-0010), and the second arm of the golden-frame oracle. **Never a parallel backend.**" Rendering SVG through resvg is a second raster backend in exactly that sense. resvg 0.48.1 requires `tiny-skia = "0.12.0"`, so no second copy is linked, but the policy line is what the grilling has to address.

### Determinism across threads and platforms

- **Claim (documented).** The README says: "Since `resvg` doesn't rely on any system libraries it allows us to have reproducible results on all supported platforms. Meaning if you render an SVG file on x86 Windows and then render it on ARM macOS - the produced image will be identical. Each pixel would have the same value."
- **How far CI tests that claim (inferred from code).**
  - [`main.yml`](https://github.com/linebender/resvg/blob/v0.48.1/.github/workflows/main.yml) runs `cargo test` on `ubuntu-latest` only. The Windows job only builds.
  - The regression comparison allows a difference: `DIFF_THRESHOLD: u8 = 1`, and `is_pix_diff` passes a pixel whose channels differ by ≤ 1 ([`tests/integration/main.rs`](https://github.com/linebender/resvg/blob/v0.48.1/crates/resvg/tests/integration/main.rs)).
  - **So "identical across platforms" is a stated design goal, not a byte-exact test.**
- **Threads (documented and inferred).** tiny-skia's [README](https://github.com/linebender/tiny-skia/blob/v0.12.0/README.md) says it does not support "dynamic CPU detection". SIMD is chosen at compile time: SSE2 by default on x86, or AVX with `-Ctarget-cpu`, and NEON on AArch64. One binary therefore runs one code path on every thread. Compare Skia, which picks its raster-pipeline functions once per process from the CPU's features (#665 D3).
  - The README also says: "Unless there is a bug, `tiny-skia` must produce exactly the same results as Skia." That is a goal about algorithms, not a statement about bytes.
- **Floating point in filters (documented and inferred).** Rust's [`f32` docs](https://doc.rust-lang.org/std/primitive.f32.html) say of `powf`, `sin` and `cos`: "The precision of this function is non-deterministic. This means it varies by platform, Rust version, and can even differ within the same execution from one invocation to the next." `sqrt` is "guaranteed to be the rounded infinite-precision result".
  - resvg uses `powf` in `feComponentTransfer` gamma ([`component_transfer.rs`](https://github.com/linebender/resvg/blob/v0.48.1/crates/resvg/src/filter/component_transfer.rs), line 68).
  - It uses `powf`, `sin` and `cos` in the lighting filters ([`lighting.rs`](https://github.com/linebender/resvg/blob/v0.48.1/crates/resvg/src/filter/lighting.rs)).
  - Blur uses a box blur or an IIR blur depending on σ (`filter/mod.rs`).
  - **So shapes, strokes and gradients sit under the reproducibility claim, while those filter primitives have no language-level guarantee.** Across painters in one process the same libm runs, so a difference is unlikely, but it is not promised.
- **Thread safety (not checked).** Whether `usvg::Tree` is `Send + Sync` was not checked. The resolver callbacks in `Options` are declared `Send + Sync` ([`parser/image.rs`](https://github.com/linebender/resvg/blob/v0.48.1/crates/usvg/src/parser/image.rs)).

### Licence (Documented)

- resvg and usvg: Apache-2.0 OR MIT (README; crates.io metadata for 0.48.1).
- tiny-skia: BSD-3-Clause, as Skia ([README](https://github.com/linebender/tiny-skia/blob/v0.12.0/README.md#license)). Montagent already ships it.

### Dependency overlap (Inferred from manifests)

- **Already shared with Montagent.** `resvg` 0.48.1 depends on `tiny-skia 0.12.0` and `usvg`. `usvg` with `text` depends on `harfrust 0.12.0`, `skrifa 0.44` and `fontdb 0.24`.
  - Montagent's `Cargo.lock` already holds `tiny-skia 0.12.0`, `harfrust 0.12.0` and `skrifa 0.44.0`. The last two come in through `parley 0.11.1`, whose crates.io dependencies are `harfrust ^0.12.0` and `skrifa ^0.44.0`. Montagent also pins `skrifa =0.46.2` directly.
- **New crates** would include `usvg`, `roxmltree`, `simplecss`, `svgtypes`, `kurbo` and `fontdb`.
- **MSRV** is 1.85.

## 3. Other Lottie renderers

| Renderer | Language, CPU path | State | What it lacks | Source |
|---|---|---|---|---|
| **velato** 0.12.0 (Linebender) | Pure Rust. Builds a `vello::Scene` (GPU compute), **or** calls a `RenderSink` trait you implement: `push_layer`, `push_clip_layer`, `draw(stroke, transform, brush, shape)`, `draw_image` | Released 2026-09-10. MIT or Apache-2.0 | README "Missing features": text, image embedding, stroke dash, zig-zag, most layer effects, correct colour stops, split rotations, split positions | [README](https://github.com/linebender/velato), [`runtime/render.rs` @ v0.12.0](https://github.com/linebender/velato/blob/v0.12.0/src/runtime/render.rs) |
| **rasterlottie** 0.2.4 | Pure Rust, on `tiny-skia 0.12` | Young (released 2026-09-23). Describes itself as "Pure Rust, headless Lottie rasterizer for deterministic server-side rendering". MIT or Apache-2.0 | Its default profile rejects text animators, layer effects, expressions and non-scalar time remap. Text only from embedded glyphs | [README](https://github.com/neodyland/rasterlottie), [crates.io](https://crates.io/crates/rasterlottie) |
| **ThorVG** 1.1.2 | C++, software rasteriser ("CPU/SIMD") | Active. Released 2026-09-18. MIT. Used by LottieFiles' dotLottie player, Canva on iOS, Camtasia | Expressions are off by default (`-Dextra=lottie_exp`). A meson C++ build, so a second C++ toolchain story next to Skia. Rust use is through a third-party binding (`thorvg` 0.5.1) | [README](https://github.com/thorvg/thorvg), [`thorvg.h` @ v1.1.2](https://github.com/thorvg/thorvg/blob/v1.1.2/inc/thorvg.h) |
| **rlottie** (Samsung) | C++, CPU | README: "This project has been archived and is no longer maintained" | Its own table marks text glyphs, fonts, merge paths and layer effects ⛔ | [README](https://github.com/Samsung/rlottie) |

- **velato through a Skia sink.** velato's `RenderSink` makes it the Lottie counterpart of `usvg` route (b): it evaluates the animation and hands over kurbo shapes and brushes. A Montagent sink could draw them with Skia's core API, which is already in the pin, without `skottie`. **Inferred**: nobody has built such a sink here.
- **Another C++ engine.** ThorVG is the only mature CPU-only Lottie engine outside Skia. Taking it means adding a second C++ engine.

## 4. Lottie seeks: is a seek to t a pure function of t?

- **Skottie (inferred from code, at Skia [`c9c3c5a9`](https://github.com/rust-skia/skia/tree/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6), the commit the pin builds):**
  - `Animation::seekFrame` clamps `fInPoint + t` to the in and out points, calls `seek(comp_time)` on every animator, then calls `fSceneRoot->revalidate` ([`Skottie.cpp`](https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/modules/skottie/src/Skottie.cpp), lines 506–521).
  - **Keyframes.** `KeyframeAnimator` keeps a cached segment, `fCurrentSegment`, re-searched whenever `!contains(t)`. `contains` is half-open (`kf0->t <= t && t < kf1->t`), so any t lies in exactly one segment, and the cache only speeds up the lookup ([`KeyframeAnimator.cpp`](https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/modules/skottie/src/animator/KeyframeAnimator.cpp), [`.h`](https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/modules/skottie/src/animator/KeyframeAnimator.h)).
  - **Sync.** `AnimatablePropertyContainer::onSeek` re-syncs only when some animator's value changed, and always on the first seek ([`Animator.cpp`](https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/modules/skottie/src/animator/Animator.cpp)). Each stored value equals f(t) after a seek, so skipping a sync leaves the same state.
  - **Text "randomize order".** It re-seeds `std::mt19937` with a fixed `kSeed = 42` on every rebuild ([`RangeSelector.cpp`](https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/modules/skottie/src/text/RangeSelector.cpp)), so it does not depend on history. But it shuffles with `std::shuffle`, whose algorithm the C++ standard library implementation chooses ([cppreference, `std::shuffle`](https://en.cppreference.com/w/cpp/algorithm/random_shuffle), a secondary reference to the standard). The order could differ between libc++ and the MSVC STL, and so between targets. Not checked further.
  - **Verdict:** pure in t for keyframed properties, inferred and not measured. Each painter needs its own `Animation`, because seeking mutates it.
  - **Not checked:** each effect adapter's internal state, sksg's invalidation caching, and expressions (ignored unless an `ExpressionManager` is set; [`Skottie.h`](https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/modules/skottie/include/Skottie.h)).
- **velato (inferred from code).** `Renderer::append(&Composition, frame, transform, alpha, sink)` takes the frame as an argument and the composition by shared reference. The `Renderer` holds only scratch buffers (`batch`, `mask_elements`), and `append` clears them first. So it is pure by construction.
- **rasterlottie (inferred from the API).** `Renderer::render_frame(&animation, frame, config)` takes the frame as an argument. A `PreparedAnimation` holds caches in `RefCell`s (`timeline_sample_cache`, `static_path_cache`, and others; [`render/renderer_types.rs`](https://github.com/neodyland/rasterlottie/blob/v0.2.4/src/render/renderer_types.rs)). Those caches make it not `Sync`. Whether a cache hit always equals a fresh result is not checked.
- **ThorVG (documented).** From `Animation::frame(float no)`: "For efficiency, ThorVG ignores updates to the new frame value if the difference from the current frame value is less than 0.001 … Values less than 0.001 may be disregarded and may not be accurately retained by the Animation" ([`thorvg.h` @ v1.1.2](https://github.com/thorvg/thorvg/blob/v1.1.2/inc/thorvg.h)). **A seek therefore depends on the previous seek** for nearby values. It would be harmless only if every requested frame differs from the last by at least 0.001, or if each seek starts from a fresh `Animation`.
- **The spec (documented).** [Lottie spec 1.0.1, properties](https://github.com/lottie/lottie-spec/blob/1.0.1/docs/specs/properties.md) defines a property's value from its keyframes at a frame number. It also says: "If two keyframes share the `t` value, the implementation MUST render one of the two values at the given frame". So which value appears at a shared `t` is left to the implementation, but it must not depend on history.

## 5. The size rule's inputs: no absolute `width`/`height`, and `viewBox`

- **Skia `SkSVGDOM` (inferred from code, at `c9c3c5a9`).**
  - The root's `width` and `height` default to `100%` ([`SkSVGSVG.h`](https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/modules/svg/include/SkSVGSVG.h), lines 33–34).
  - `SkSVGSVG::intrinsicSize` returns `(0, 0)` if *either* dimension is a percentage, and never reads `viewBox` ([`SkSVGSVG.cpp`](https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/modules/svg/src/SkSVGSVG.cpp), lines 106–115). So `width="200"` alone, with a `viewBox`, still reports 0×0.
  - `setContainerSize` supplies the viewport for relative units ([`SkSVGDOM.h`](https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/modules/svg/include/SkSVGDOM.h)).
- **usvg (inferred from code, [`parser/converter.rs` `resolve_svg_size`](https://github.com/linebender/resvg/blob/v0.48.1/crates/usvg/src/parser/converter.rs), lines 523–595):**
  - A missing `width` or `height` means `100%`.
  - **With a `viewBox`:** a percentage dimension is that percentage of the viewBox's width or height. If exactly one of `width` and `height` is given, the other is derived from the viewBox's aspect ratio. So a root with only a `viewBox` gets the viewBox's size.
  - **Without a `viewBox`:** a percentage applies to `Options::default_size`, which is 100×100 by default ([`parser/options.rs`](https://github.com/linebender/resvg/blob/v0.48.1/crates/usvg/src/parser/options.rs)).
  - A zero or invalid size is `Error::InvalidSize`.
  - This refines ADR-0027's "usvg invents 100×100": that happens only when there is no `viewBox`.
- **Lottie (documented).** The [1.0.1 schema](https://github.com/lottie/lottie-spec/blob/1.0.1/schema/composition/animation.json) requires `w`, `h`, `fr`, `ip` and `op`. `w` and `h` are integers with `minimum: 0`, so a Lottie file always states a pixel size, though it may legally be 0. Skottie's `render(canvas, dst)` scales the animation into `dst` with `kCenter_ScaleToFit` (`Skottie.cpp`, line 489).
- **Other candidates.** velato's `append` clips to `animation.width`×`height`. ThorVG was not checked.

## 6. Text

- **SVG.**
  - **What it needs.** `<text>` names fonts by CSS `font-family` and needs shaping.
  - **Skia.** `SkSVGDOM::Builder::setFontManager`: "If this is not set, but a font is required as part of rendering, the text will not be displayed". `setTextShapingFactory` supplies the shaper ([`SkSVGDOM.h`](https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/modules/svg/include/SkSVGDOM.h)). That shaper is Skia's `skshaper`, brought in by `textlayout`.
  - **usvg: its own path.** It shapes with `harfrust` and reads outlines with `skrifa`. `Options::fontdb` defaults to an **empty** `fontdb::Database`. System fonts are an opt-in Cargo feature (`system-fonts` = `fontdb/fs`, `fontdb/fontconfig`; [`usvg/Cargo.toml`](https://github.com/linebender/resvg/blob/v0.48.1/crates/usvg/Cargo.toml)). That mirrors Montagent's `parley` without `system`.
  - **usvg: a missing family.** The default selector logs "No match for '…' font-family." and returns `None` ([`text/mod.rs`](https://github.com/linebender/resvg/blob/v0.48.1/crates/usvg/src/text/mod.rs)). Layout then skips that span (`None => continue`, [`text/layout.rs`](https://github.com/linebender/resvg/blob/v0.48.1/crates/usvg/src/text/layout.rs), line 262). So a missing font drops text silently, and Montagent would have to turn that into an error.
  - **usvg: fallback.** The fallback selector walks `fontdb.faces()` in database order, which is deterministic if the fonts are loaded in a fixed order.
  - **resvg's own tests** load a vendored `tests/fonts` directory and fix the generic families.
  - **Can it be byte-identical?** Plausibly, with vendored fonts, no system fonts, and text flattened to paths (`Text::flattened`). Inferred, not measured. It would be the same `harfrust` and `skrifa` versions Montagent's own text uses today.
- **Lottie.**
  - **Not in the spec.** The published [Lottie spec 1.0.1](https://github.com/lottie/lottie-spec/blob/1.0.1/docs/specs/layers.md) lists five layer types: shape, image, null, solid and precomposition. **There is no text layer**, so Lottie text is defined only by what After Effects exports and players accept.
  - **Skottie** takes a font manager (`Builder::setFontManager`). If no shaping factory is set, "text will be shaped with primitive shaping" ([`Skottie.h`](https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/modules/skottie/include/Skottie.h)). It also builds a `CustomFont` from glyph paths embedded in the file (`chars`; [`text/Font.h`](https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/modules/skottie/src/text/Font.h)).
  - **Other players.** velato has no text. rasterlottie handles only glyph-backed text. rlottie has none.
  - **Can it be byte-identical?** A Lottie with embedded glyph paths needs no font lookup and no shaper, so it is the deterministic case. A Lottie that names fonts needs the same vendored-font discipline as SVG.

## 7. Pre-rendering through the existing skill (ADR-0156 §7)

- **The skill as it stands (documented, in this repo).**
  - [`skills/montagent-prerender/SKILL.md`](../../skills/montagent-prerender/SKILL.md): code writes RGBA frames, `prerender.py build` encodes **PNG-in-MOV** (`-c:v png -pix_fmt rgba`), decodes it, and fails on the first frame that differs.
  - A recipe beside the footage holds the code, the commands, the tool versions and the decoded-frame hashes.
- **Codec cost (measured, by #723).** This is the [codec task's result](https://github.com/MBehtemam/Montagent/issues/723).
  - PNG-in-MOV RGBA round-trips with **0** differing pixels, alpha included.
  - At 1080p and 25 fps it takes about **6.0 MB per second** of footage on smooth content and **141 MB/s** on noisy content.
  - Decoding 50 frames took 0.14 s (smooth) and 0.39 s (noisy). The machine was loaded, so the timings are indicative.
- **An SVG still needs no footage.** The schema already has an `image` element. One lossless PNG and a recipe are enough.
  - The format change is nil, and every painter decodes the same PNG, so painting stays byte-identical by construction.
  - The rasteriser (resvg CLI, a browser, Inkscape) runs once, outside the render, and its version goes in the recipe.
- **An animated Lottie becomes PNG-in-MOV** through the skill unchanged, rendered by any player that can write RGBA frames.
  - ThorVG's CLI tools write GIF (`lottie2gif`) per its [README](https://github.com/thorvg/thorvg), and GIF is not lossless RGBA, so it would need a frame-writing harness.
  - The player's seek behaviour (§4) affects only the one pre-render, not the parallel painters.
- **What is lost.**
  - **Resolution is fixed at pre-render time.** Any later `scale`, `fit` or zoom resamples a raster, so a piece must be rendered at its largest displayed size.
  - **A recipe toolchain becomes a dependency** of reproducing the piece, though not of rendering the project.
  - **Size.** A long noisy piece is large: 141 MB per second at 1080p.
- **Per-frame cost of the SVG rasterisation itself:** not measured. This ticket is reading only.

## 8. The reference class today

- **Premiere Pro and SVG.**
  - Adobe's [Premiere Pro supported file formats page](https://helpx.adobe.com/premiere-pro/using/supported-file-formats/supported-file-formats.html) **refused the fetch with HTTP 403** on 2026-10-07. Per the brief, no other route to it (another locale, an archive) was tried, so the 2026 text of Adobe's own format list is **not sourced here**.
  - The open Adobe feature request [Add the ability to work directly with vector files (SVG, AI, EPS)](https://community.adobe.com/feature-requests-730/feature-request-add-the-ability-to-work-directly-with-vector-files-svg-ai-eps-1330085) is still "Open for Voting". It was filed on 2023-05-26, and its last comment is from December 2024. No Adobe staff reply was shown.
- **Premiere Pro and Lottie.**
  - On Adobe's community forum, a Community Expert answered "Short answer: no" to importing a Lottie JSON (2024-01-31, [thread](https://community.adobe.com/t5/premiere-pro-discussions/importing-a-lottie-animation-json-file-into-premiere-pro/td-p/14392672)).
  - LottieFiles' own [Premiere Pro extension post](https://lottiefiles.com/blog/working-with-lottie-animations/elevate-video-projects-lottiefiles-in-adobe-premiere-pro.md) (2025-04-02) says to "Choose between MP4 or transparent MOV files". So Lottie reaches Premiere as **pre-rendered footage**, which is the same shape as §7.
- **CapCut.**
  - CapCut's help page [Why can't local assets be recognized during import?](https://www.capcut.com/help/can-not-local-assets-be-recognized-during-import) (dated 2025-12-26) lists formats for iOS and Android: BMP, JPEG, PNG, WebP, HEIF, then video. **It names no SVG, Lottie or JSON.**
  - The desktop list on [Why can't I import MP4 and JPG files?](https://www.capcut.com/help/can-not-import-mp4-and-jpg-files) (2025-12-30) is an image, not text, so the desktop formats are **not sourced**.
  - CapCut's [SVG-to-PNG article](https://www.capcut.com/resource/svg-to-png) converts SVG with other tools before editing in CapCut Desktop, and lists "PNG, JPEG, and GIF".
- **Conclusion.** Neither reference vendor documents native SVG or Lottie import as of the sources read. The one documented Lottie path into Premiere is pre-rendered footage.

## Not sourced

- The current text of Adobe's Premiere Pro supported-formats page (HTTP 403).
- CapCut desktop's import format list (published only as an image).
- Whether rust-skia would accept a CPU-only `svg` or `skottie` key, and the Skia version the timed CI runs built.
- Byte-exact cross-platform equality for resvg. Its CI tests on Linux only, with a ±1 tolerance.
- Whether `usvg::Tree` and `velato::Composition` are `Send + Sync`.
- Whether rasterlottie's caches always return what a fresh evaluation would.
- Skottie's per-effect state and sksg's invalidation caching beyond the keyframe path. The platform dependence of `std::shuffle` in "randomize order" text.
- ThorVG's Lottie text support and its single-threaded determinism.
- What a Vulkan-carrying `svg` key costs at link time or at runtime on a machine without Vulkan.
- Any measurement of SVG or Lottie byte stability under Montagent's K painters.

## Sources

Read on 2026-10-07.

**This repository:** `Cargo.toml` (the Skia pin, `tiny-skia` and `parley` comments), `Cargo.lock`, `crates/montagent-render/tests/skia_pin.rs`, `schema/montagent.schema.json`, `skills/montagent-prerender/SKILL.md`, ADR-0010, ADR-0027 and ADR-0156, and issues [#665](https://github.com/MBehtemam/Montagent/issues/665), [#723](https://github.com/MBehtemam/Montagent/issues/723) and [#726](https://github.com/MBehtemam/Montagent/issues/726).

**rust-skia [`0.153.2`](https://github.com/rust-skia/rust-skia/tree/0.153.2):**

- `README.md`
- `skia-bindings/build_support/binaries_config.rs`
- `mk-workflows/src/config.rs` and `main.rs`
- the release assets of [`skia-binaries` 0.153.2](https://github.com/rust-skia/skia-binaries/releases/tag/0.153.2)
- Actions runs [27671558692](https://github.com/rust-skia/rust-skia/actions/runs/27671558692), [33868798001](https://github.com/rust-skia/rust-skia/actions/runs/33868798001), [33868798088](https://github.com/rust-skia/rust-skia/actions/runs/33868798088) and [33868798199](https://github.com/rust-skia/rust-skia/actions/runs/33868798199) (job timings through the GitHub API)

**Skia at [`c9c3c5a9`](https://github.com/rust-skia/skia/tree/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6):**

- `modules/svg/include/SkSVGDOM.h`, `SkSVGSVG.h`, `modules/svg/src/SkSVGSVG.cpp`
- `modules/skottie/include/Skottie.h`, `src/Skottie.cpp`
- `src/animator/{Animator.cpp, KeyframeAnimator.cpp, KeyframeAnimator.h}`
- `src/text/{RangeSelector.cpp, Font.h}`, and the `src/effects/` listing

**resvg [`v0.48.1`](https://github.com/linebender/resvg/tree/v0.48.1) (`68b14c4c`):**

- `README.md`, `.github/chart.svg`, `.github/workflows/main.yml`
- `crates/resvg/{Cargo.toml, src/lib.rs, src/filter/*, tests/integration/main.rs}`
- `crates/usvg/{Cargo.toml, src/parser/{converter.rs, options.rs, image.rs}, src/tree/{mod.rs, text.rs}, src/text/{mod.rs, layout.rs}}`
- [crates.io `resvg`](https://crates.io/crates/resvg)
- the [support table](https://linebender.org/resvg-test-suite/svg-support-table.html) (resvg 0.40.0)

**Others:**

- tiny-skia [`v0.12.0`](https://github.com/linebender/tiny-skia/tree/v0.12.0): `README.md` and `src/pixmap.rs`
- Rust [`f32` documentation](https://doc.rust-lang.org/std/primitive.f32.html)
- [crates.io `parley` 0.11.1 dependencies](https://crates.io/crates/parley/0.11.1/dependencies)
- velato [`v0.12.0`](https://github.com/linebender/velato/tree/v0.12.0): `src/runtime/render.rs`, `Cargo.toml`. The README was read from the default branch.
- rasterlottie [`v0.2.4`](https://github.com/neodyland/rasterlottie/tree/v0.2.4): `src/render/renderer_types.rs`. The README and `Cargo.toml` were read from the default branch. Also [crates.io](https://crates.io/crates/rasterlottie).
- ThorVG [`v1.1.2`](https://github.com/thorvg/thorvg/tree/v1.1.2): `inc/thorvg.h`. The README was read from the default branch.
- [Samsung/rlottie](https://github.com/Samsung/rlottie): `README.md`, `COPYING`, `inc/rlottie.h`, all from the default branch.
- [Lottie spec 1.0.1](https://github.com/lottie/lottie-spec/tree/1.0.1): `docs/specs/{layers.md, properties.md, composition.md}` and `schema/composition/animation.json`
- [cppreference, `std::shuffle`](https://en.cppreference.com/w/cpp/algorithm/random_shuffle) (secondary)

**Vendors:** the Adobe community pages and LottieFiles blog post linked in §8, and the CapCut help and resource pages linked in §8.
