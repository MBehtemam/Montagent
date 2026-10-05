# What the pinned `skia-safe` build exposes for each capability, and which parts stay byte-identical

Research for [#665](https://github.com/MBehtemam/Montagent/issues/665), part of the map [#663](https://github.com/MBehtemam/Montagent/issues/663).

**Date of research:** 2026-10-05. Nothing was built or run for this note. It is a reading of sources.

**This note decides nothing.** It says what the pinned build can draw and what is known about its bytes. Each capability's ticket on the map makes its own call.

## Answer

- **Ten of the twelve capabilities need no change to the Skia pin.** Blend modes, gradients, path effects (trim, dash), path measuring and interpolation, `M44` perspective, runtime effects (SkSL), clip shaders and every image filter are in Skia's core and effects libraries, which the pinned prebuilt (`jpegd-jpege-pdf`) already contains. `skia-safe` puts none of them behind a Cargo feature.
- **Vector sources are the one capability that moves the pin.** `svg` and `skottie` are Cargo features that link extra Skia modules and pull in `textlayout`.
  - **`svg`:** no CPU-only prebuilt exists on all six targets. One exists for macOS only. Linux and Windows have an `svg` prebuilt only together with a GPU backend.
  - **`skottie`:** a prebuilt exists for Linux only, in one key that also carries GL, EGL, Vulkan, X11 and Wayland. On macOS and Windows it is a source build.
- **Skia documents no byte-determinism guarantee.** I found no statement in Skia's headers or site docs that the raster backend gives the same bytes for the same input. Every determinism statement below is therefore read from code, or measured by this repo, or marked unknown.
- **No capability was found to be non-deterministic across painters.** Nothing read in the pinned source makes a frame's bytes depend on which thread paints it or what that thread painted before. That is an inference from code, not a tested fact, for every capability the painter does not use today.
- **The known risks are narrower than "threads":**
  1. Anything drawn inside a layer changes bytes if the layer's top-left corner moves (measured by [#649](https://github.com/MBehtemam/Montagent/issues/649)). New capabilities that add layers, or that want a bound, inherit this.
  2. Perspective is excluded from the byte-identical filter bound of [#652](https://github.com/MBehtemam/Montagent/issues/652), and Skia picks an approximate filter scale under perspective.
  3. SVG and Lottie text reaches a font manager and a shaper that Montagent's own text stack was built to avoid.
  4. Bytes differ between machines (SIMD path chosen at run time on x86). The repo already knows this and does not assert bytes across targets.

## Contents

- [Summary table](#summary-table)
- [How to read the determinism column](#how-to-read-the-determinism-column)
- [What "byte-identical" has to mean here](#what-byte-identical-has-to-mean-here)
- [The build: what the pinned prebuilt contains](#the-build-what-the-pinned-prebuilt-contains)
- [Determinism facts that apply to every capability](#determinism-facts-that-apply-to-every-capability)
- [Per capability](#per-capability)
- [Not found, and not checked](#not-found-and-not-checked)
- [Sources](#sources)

## Summary table

"In the pin" means the pinned prebuilt already contains the code and `skia-safe` exposes it with today's feature list.

| # | Capability | Skia API that draws it | In the pin? | Determinism across painters | Risk to watch |
|---|---|---|---|---|---|
| 1 | Keyframable non-transform properties, spring easing | None needed. Values are arguments to draws the painter already makes (`RRect`, `Paint` colour, filter parameters). `CubicMap` exists for cubic-bezier easing | Yes | **Inferred from code:** same arguments, same draw | Animated blur σ crosses the σ ≤ 135 limit of the #652 bound |
| 2 | Blend modes | `BlendMode` (29 modes) on `Paint::set_blend_mode`, or on the `SaveLayerRec` paint; `Blender`, `blenders::arithmetic` | Yes | **Inferred from code.** `BlendMode::Clear` is already **measured** | A blended layer must sit where it does not disturb the #652 bound's preconditions |
| 3 | Gradients | `gradient::shaders::{linear,radial,two_point_conical,sweep}_gradient`, with `Gradient`, `Colors`, `Interpolation` | Yes | **Inferred from code** | Dither is off unless asked for; if turned on it is positional (ordered 8×8), so it moves with a layer's origin |
| 4 | Letter spacing, per-letter animation | Not Skia. `parley` has `StyleProperty::LetterSpacing`; the painter fills glyph outlines as paths. Skia's own route (`TextBlob` positioned runs, `Canvas::draw_glyphs_at`) is in the pin but unused by design (ADR-0010) | Yes (no Skia change) | **Measured** for path-filled glyphs today; spacing only moves the paths | None from Skia |
| 5 | Repeat or stagger | Not Skia. Repeated draws. `Picture` recording exists if one recording is replayed | Yes | **Inferred from code** | None from Skia |
| 6 | Paths, lines, trim, stroke, dash, text on a path, morphing | `PathBuilder`; `Paint` stroke width, cap, join, miter; `PathEffect::trim`, `PathEffect::dash`; `ContourMeasure::pos_tan` / `segment`; `Path::interpolate` + `is_interpolatable` | Yes | **Inferred from code** | `PathEffect::discrete` is random by a caller-supplied seed; keep it out or pin the seed |
| 7 | Animated mattes, wipe/slide/push, animated `clip` | `clip_rect` / `clip_rrect` / `clip_path` (animated), `clip_shader`, `save_layer` + `BlendMode::DstIn`/`DstOut`, `luma_color_filter`, gradient shaders as soft wipes | Yes | **Inferred from code.** Layer + `Clear` mask is **measured** | Same layer-origin rule as #2 |
| 8 | Motion blur | No primitive. Three constructions: (a) paint N sub-frame instants and average them (`BlendMode::Plus`, or accumulate outside Skia); (b) directional blur: `image_filters::matrix_transform` rotate → `image_filters::blur` on one axis → rotate back; (c) an SkSL shader | Yes | (a) **inferred from code**; (b) **unknown**; (c) **inferred from code** | (b) resamples twice and is outside the #652 derivation |
| 9 | Shader vocabulary | `RuntimeEffect::make_for_shader` / `make_for_color_filter` / `make_for_blender`, `RuntimeShaderBuilder`, `image_filters::runtime_shader`; built-in `perlin_noise_shader::{fractal_noise,turbulence}` | Yes | **Inferred from code.** The chroma keyer (a runtime colour filter) is **measured** | `RuntimeEffect` is not `Send`/`Sync` in `skia-safe`: one per painter. Noise takes an explicit seed |
| 10 | 2.5D perspective | `M44` (`M44::perspective`, `rotate`, `translate`), `Canvas::concat_44`, `Canvas::set_matrix(&M44)`; `utils::Camera3D` also bound | Yes | **Inferred from code** for the draw. **Unknown** with blur, shadow and mask | The #652 bound gives no hint under perspective; Skia picks an approximate filter scale under perspective |
| 11 | Speed ramps, time remapping | Not Skia. Decides which source frame is asked for | Yes | Not a Skia question | None from Skia |
| 12 | Vector sources: SVG, Lottie | `svg::Dom` (`from_bytes`, `set_container_size`, `render`); `skottie::Animation` (`from_str`, `seek_frame`, `render`) | **No.** `svg` and `skottie` are Cargo features; both move the prebuilt key. See [the build](#the-build-what-the-pinned-prebuilt-contains) | **Unknown.** Shapes: inferred to be ordinary draws. Text: reaches a font manager and shaper | Fonts, external images, `Dom` not `Send`/`Sync`, Lottie expressions ignored by default |

## How to read the determinism column

Every determinism statement in this note carries one of four marks.

- **Documented:** a Skia header or doc page says it.
- **Inferred from code:** read from the pinned Skia source or the `skia-safe` 0.153.2 source. Nobody ran it.
- **Measured:** a test or research run in this repo compared bytes.
- **Unknown:** no source found.

No row is marked "documented" for determinism, because Skia documents none. The documented facts are about what an API does, and they are quoted in the detail sections.

## What "byte-identical" has to mean here

The rule on the map is: output stays byte-identical across parallel painters. The render ADRs fix what that means.

- **A paint chunk is a run of frames, not a region of a frame.** ADR-0144 ([#653](https://github.com/MBehtemam/Montagent/issues/653), on its own branch at the time of writing) cuts a span into chunks of C = 2 frames and hands them to K painters. Each painter owns a whole canvas and paints whole frames.
- **So the rule is: a frame's bytes must not depend on which painter painted it, or on what that painter painted before.** #653's `painters.rs` lists everything a painter carries between frames and argues none of it reaches a pixel. Its CI guard paints a blur-and-glow project with K = 3, C = 2 and requires the same frame hashes as one painter.
- **There is a second, separate rule from [#652](https://github.com/MBehtemam/Montagent/issues/652):** a `bounds` hint on a filter layer must not change bytes. [#649](https://github.com/MBehtemam/Montagent/issues/649) measured that it does unless the layer's top-left corner stays put. This is not about threads, but any new capability that draws inside a layer, or wants a smaller layer for speed, meets it.
- **Bytes across machines are not part of the rule.** `crates/montagent-core/tests/golden_frames.rs` says so: "the same rasterizer on a different tier-1 target takes a different SIMD path", and goldens compare by SSIM, not bytes. ADR-0143 pins the encoder's threads so one machine's bytes do not depend on its core count.

So a capability breaks the rule if it reads any of these:

1. state shared between threads that changes a result (not just its speed);
2. state a painter keeps from an earlier frame;
3. a clock or an unseeded random source;
4. a layer whose origin or size differs between two paints of the same frame.

The per-capability sections check each API against these four.

## The build: what the pinned prebuilt contains

### The pin

- The workspace pins `skia-safe = "=0.153.2"` with `default-features = false` and features `binary-cache`, `embed-icudtl`, `pdf`, `jpeg` (`Cargo.toml`).
- That resolves to the prebuilt key **`jpegd-jpege-pdf`**, recorded in `workspace.metadata.skia.prebuilt-key` and asserted by `crates/montagent-render/tests/skia_pin.rs`.
- `crates/montagent-render` turns on `skia-safe/no-compile` by default, so a prebuilt miss fails the build instead of compiling Skia ([ADR-0010](../../adr/0010-skia-safe-rasterizer-text-beside-it.md)).
- `skia-bindings` 0.153.2 builds Skia tag `m153-0.101.1`, which is commit [`c9c3c5a9`](https://github.com/rust-skia/skia/tree/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6) of `rust-skia/skia` (`package.metadata.skia` in the crate manifest; tag resolved through the GitHub API).

### How the key is made

The key is the sorted list of Skia-affecting features, joined by `-` ([`build_support/features.rs`](https://github.com/rust-skia/rust-skia/blob/0.153.2/skia-bindings/build_support/features.rs), `to_key`). The download name is `skia-binaries-{hash}-{target}-{key}.tar.gz` ([`binary_cache/binaries.rs`](https://github.com/rust-skia/rust-skia/blob/0.153.2/skia-bindings/build_support/binary_cache/binaries.rs), `key`). A feature set with no published archive is a source build.

### Which features gate which code

Read from the `skia-safe` 0.153.2 manifest and `src/modules.rs`:

| Cargo feature | What it adds | Implies |
|---|---|---|
| `textlayout` | `skparagraph`, `skshaper`, `skunicode` libraries; the `textlayout` and `shaper` modules | – |
| `svg` | the `svg` library and `skresources`; the `skia_safe::svg` module | `textlayout` (in `skia-bindings`: `svg = ["textlayout"]`) |
| `skottie` | `skottie`, `sksg`, `jsonreader`, `skresources`; the `skia_safe::skottie` module | `textlayout` |
| `webp`, `gl`, `vulkan`, `metal`, `d3d`, `ganesh`, `graphite`, … | codecs and GPU backends | – |

**Everything else on the map is ungated.** `src/core.rs`, `src/effects.rs` and `src/utils.rs` carry no `cfg(feature)` line for blend modes, gradients, path effects, `M44`, runtime effects, image filters, contour measuring or `Camera3D`. They compile with the feature list the workspace already has, and their C++ lives in the one `skia` library every prebuilt archive contains ([`binaries_config.rs`](https://github.com/rust-skia/rust-skia/blob/0.153.2/skia-bindings/build_support/binaries_config.rs): `svg`, `skottie` and `textlayout` are the only features that add libraries).

### Which `svg` and `skottie` prebuilts are published

Listed from the release assets of [`rust-skia/skia-binaries` tag `0.153.2`](https://github.com/rust-skia/skia-binaries/releases/tag/0.153.2) (152 archives) on 2026-10-05. The six targets are the ones ADR-0064 commits to.

| Key | macOS (arm64, x86_64) | Linux gnu (arm64, x86_64) | Windows msvc (arm64, x86_64) |
|---|---|---|---|
| `jpegd-jpege-pdf` (today's pin) | yes, yes | yes, yes | yes, yes |
| `jpegd-jpege-pdf-textlayout` | yes, yes | yes, yes | yes, yes |
| `jpegd-jpege-pdf-svg-textlayout-webpd-webpe` (CPU-only `svg`) | **yes, yes** | **no, no** | **no, no** |
| `ganesh-jpegd-jpege-pdf-svg-textlayout-vulkan-webpd-webpe` | no, no | yes, yes | yes, yes |
| `ganesh-jpegd-jpege-metal-pdf-svg-textlayout-webpd-webpe` | yes, yes | – | – |
| `egl-ganesh-gl-jpegd-jpege-pdf-skottie-svg-textlayout-vulkan-wayland-webpd-webpe-x11` (the only `skottie` key) | – | yes, yes | – |

What follows from the table:

- **`svg` with no GPU backend is published for macOS only.** There is no single `svg` key on all six targets.
- **Every `svg` key off macOS also turns on `ganesh` with `gl` or `vulkan`.** That contradicts the "CPU-only, no `ganesh`, no `gl`" line ADR-0010 wrote into the key. The table shows the one such key that Linux and Windows share on both architectures; a few more exist per target, all with `ganesh`.
- **The CPU-only `svg` key also carries `webp`**, so taking it adds the WebP codec as well.
- **`skottie` is published for Linux only**, in one key with five windowing and GPU features. macOS and Windows have none.
- **A per-target feature list could in principle pick a different published key per target.** That is an inference from how Cargo features and the key work; nobody tried it, and it would make the six targets link different Skia builds.
- **Otherwise `svg` or `skottie` means a source build** of Skia (LLVM/Clang, Python, Ninja), which is the cost ADR-0010 accepted the C++ dependency to avoid, or hosting our own archives through `SKIA_BINARIES_URL` (documented in the [`skia-bindings` README](https://github.com/rust-skia/rust-skia/blob/0.153.2/skia-bindings/README.md), "Prebuilt Binaries in an Offline Environment").

This corrects one line in [ADR-0027](../../adr/0027-vector-sources-out-of-scope.md)'s evidence: its research note left "whether `svg` forces `textlayout`" unconfirmed. It does, in `skia-bindings`' manifest.

## Determinism facts that apply to every capability

**D1. Skia's docs promise no determinism. (Not found.)** I searched the pinned tree's public headers for `SkCanvas`, `SkSurface`, `SkPaint`, `SkPath`, `SkImage`, `SkShader`, `SkPicture`, `SkRuntimeEffect`, `SkGraphics`, and the site pages `user/sksl.md`, `user/tips.md`, `user/api/skcanvas_overview.md`, `user/api/skpaint_overview.md`, `user/modules/skottie.md`. None says that the same draw gives the same bytes, across threads or otherwise.

**D2. The CPU backend runs no threads of its own unless asked. (Inferred from code.)** [`SkExecutor.cpp`](https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/src/core/SkExecutor.cpp#L37-L61): "The default default SkExecutor is an SkTrivialExecutor, which just runs the work right away." Montagent never sets another. So a draw runs wholly on the painter's thread.

**D3. The SIMD path is chosen once per process, not per thread. (Inferred from code.)** [`SkOpts.cpp`](https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/src/core/SkOpts.cpp#L50-L76) replaces the raster pipeline's function pointers once, behind a function-local static, from the CPU's features (`SkCpu::Supports`). On x86 that choice depends on the machine (AVX2-level, optionally AVX-512). On ARM64 there is no run-time choice in that file. Consequences:
- every painter in one process runs the same code path;
- two x86 machines can differ, which matches what `golden_frames.rs` records.

**D4. Shared Skia objects are immutable or guarded. (Partly documented, partly inferred from code.)**
- **Documented:** "SkImage cannot be modified after it is created" ([`SkImage.h`](https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/include/core/SkImage.h#L254)); the global resource cache's static methods are "thread-safe wrappers around a global instance" ([`SkResourceCache.h`](https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/src/core/SkResourceCache.h#L141)).
- **Inferred from code:** `skia-safe` asserts which wrappers are `Send + Sync` in [`tests/send_sync.rs`](https://github.com/rust-skia/rust-skia/blob/0.153.2/skia-safe/tests/send_sync.rs): `Image`, `Shader`, `PathEffect`, `Picture`, `Blender` are; `Canvas`, `Surface`, `RuntimeEffect` and `svg::Dom` are not. A painter per thread with its own canvas, which is ADR-0144's shape, is the arrangement these assertions allow.
- **Unknown:** whether a global cache hit always returns the bytes a fresh computation would. It is the design intent of a cache, and #653's K = 3 guard passes with image mipmaps and filters in play, but no source states it.

**D5. A layer's origin reaches the pixels. (Measured.)** [#649](https://github.com/MBehtemam/Montagent/issues/649)'s findings (`docs/research/filter-bound/FINDINGS.md` on branch `research/649-filter-bound`): moving a layer's top-left corner changes antialiased coverage by ±1 on a few edge pixels, because the integer shift is applied in float. "No bound that shrinks the layer on all four sides is byte-identical." The shipped bound (`crates/montagent-render/src/canvas/layer_bound.rs`) pins left and top and gives no hint when any effect is a colour filter, when there is perspective, or when layer-space σ > 135.

**D6. Two paints of one frame are the same sequence of calls. (Measured for today's vocabulary.)** #653's chunk-start argument and its CI guard. A new capability keeps this only if the values it passes to Skia come from the document and the frame instant alone.

## Per capability

### 1. Keyframable non-transform properties, and spring easing

- **API.** Nothing new. A keyframed radius, colour or effect parameter is a different number passed to draws the painter already makes: `RRect`, `Paint::set_color4f`, `image_filters::blur` σ, the colour-filter matrices, the keyer's uniforms.
- **Easing.** Skia has `CubicMap` (`CubicMap::new(p1, p2)`, `compute_y_from_x`) for cubic-bezier curves. It has no spring. A spring is arithmetic in `montagent-core`, where ADR-0012's keyframes already resolve.
- **In the pin.** Yes.
- **Determinism.** **Inferred from code:** the draw is the same code with different arguments. If the resolved value is computed in `f64`/`f32` from the document and the instant only, D6 holds.
- **Risks.**
  - An animated blur radius can cross layer-space σ = 135, where the #652 bound switches off (D5). That changes speed, not bytes, because the bound declines rather than approximates.
  - An animated parameter makes the filtered result differ every frame, which is the case #653 already measured as uncacheable (6.7% hits).

### 2. Blend modes

- **API.** `BlendMode` is Skia's `SkBlendMode`, 29 modes from `Clear` to `Luminosity` ([`SkBlendMode.h`](https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/include/core/SkBlendMode.h)). Two places take one:
  - `Paint::set_blend_mode` on a single draw;
  - the paint of a `SaveLayerRec`, which blends a whole element at `restore`. The painter already composites each element through a layer for `opacity`, so a per-element blend mode belongs there.
  - `Paint::set_blender` with `blenders::arithmetic`, or a runtime blender, covers anything outside the 29.
- **In the pin.** Yes. The painter already imports `BlendMode` and draws with `BlendMode::Clear`.
- **Determinism.** **Inferred from code** for the 28 modes not yet used: each is a fixed raster-pipeline stage, with no state. **Measured** for `Clear` under K painters.
- **Risks.**
  - #649 read in `SkCanvas.cpp` that a `bounds` hint is honoured only when the restore is trivial, and it lists "no blender" among the conditions. So a blend mode placed on a blur or shadow layer changes whether the #652 hint applies. A blend mode on its own outer layer leaves the filter layers as they are. **Inferred from code**, not tested.
  - The non-separable and the dodge/burn modes divide; results are still a pure function of the two pixels. **Inferred from code.**

### 3. Gradients

- **API.** `skia_safe::gradient::shaders`: `linear_gradient`, `radial_gradient`, `two_point_conical_gradient`, `sweep_gradient`. Each takes a `Gradient` built from `Colors` (stops as `Color4f`, optional positions, a colour space, a `TileMode`) and an `Interpolation` (premultiplied or not, colour space, hue method). The older `gradient_shader::linear` family is still exported. The shader goes on `Paint::set_shader`.
- **In the pin.** Yes.
- **Determinism.** **Inferred from code:** a gradient is evaluated per pixel from local coordinates by raster-pipeline stages.
- **Risks.**
  - **Dither.** `SkPaint::setDither` "requests, but does not require, to distribute color error" ([`SkPaint.h`](https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/include/core/SkPaint.h#L179-L182)). The painter never sets it. If it is ever set to hide banding, the dither is an ordered 8×8 matrix indexed by device x and y ([`SkRasterPipeline_opts.h`](https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/src/opts/SkRasterPipeline_opts.h#L2316-L2338)): the same for every painter, but tied to the layer's pixel grid, so it falls under D5.
  - Animated stops are capability 1's case: new arguments each frame.
  - **Unknown:** whether a gradient shader's local coordinates round differently inside a layer than outside one. D5 suggests the same float-translation effect could apply; nobody measured it for shaders.

### 4. Letter spacing and per-letter animation

- **API.** Not a Skia capability in this repo. ADR-0010 puts text beside the rasterizer: `parley` shapes, `skrifa` scales outlines, and the painter fills paths. `parley` 0.11.1 has `StyleProperty::LetterSpacing(f32)` and `WordSpacing(f32)`. Per-letter animation is a transform per glyph path, which the painter can already apply.
- **Skia's own routes, for the record.** In the pin and unused: `TextBlobBuilder::alloc_run_pos` / `alloc_run_rsxform`, `TextBlob::from_pos_text`, `Canvas::draw_glyphs_at`. They need a Skia `Font` and typeface, which ADR-0010 keeps out. `TextStyle::set_letter_spacing` is in `skparagraph`, behind `textlayout`, which moves the key.
- **In the pin.** Yes, with no Skia change.
- **Determinism.** **Measured** for path-filled glyphs under K painters (the trailer is mostly text). Letter spacing changes where paths sit, not how they fill.
- **Risks.** None from Skia. The font registry is a per-painter cache that #653 argues cannot change shaping.

### 5. Repeat or stagger

- **API.** Not a Skia capability. N copies are N draws with N transforms. If one recording is replayed, `PictureRecorder` and `Canvas::draw_picture` exist; the #652 bound already records a picture per filtered element.
- **In the pin.** Yes.
- **Determinism.** **Inferred from code.**
- **Risks.** `SkCanvas.h` notes that drawing a picture through a layer can differ in appearance "if there are any non-associative blendModes inside any of the pictures elements". That matters only if copies are replayed with a paint. **Documented** as a behaviour note, not as a determinism statement.

### 6. Paths, lines, trim-path, stroke properties, text on a path, morphing

- **Paths and lines.** `PathBuilder` and `Path`, already used for glyphs, rounded rects and ellipses.
- **Stroke properties.** `Paint::set_stroke_width`, `set_stroke_cap`, `set_stroke_join`, `set_stroke_miter`. The painter already strokes with a round join.
- **Dash.** `PathEffect::dash(intervals, phase)`. **Documented** ([`SkDashPathEffect.h`](https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/include/effects/SkDashPathEffect.h)): intervals are on/off lengths, an even count ≥ 2; phase is an offset "mod the sum of all of the intervals"; "only affects stroked paths".
- **Trim.** `PathEffect::trim(start_t, stop_t, mode)`. **Documented** ([`SkTrimPathEffect.h`](https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/include/effects/SkTrimPathEffect.h)): `t` is 0..1 and pinned to that range; NaN returns null; "The trim values apply to the entire path, so if it contains several contours, all of them are including in the calculation." `Mode::Inverted` returns the complement.
- **Text on a path.** Skia has no draw-text-on-path call. The construction is `ContourMeasure::pos_tan(distance)` for a position and tangent per glyph, then a transform per glyph path. That fits the painter, which already holds one path per glyph.
- **Morphing.** `Path::interpolate(ending, weight)` with `Path::is_interpolatable`. **Documented** ([`SkPath.h`](https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/include/core/SkPath.h#L195-L232)): it needs "equal verbs and equal weights", and returns an empty path otherwise. The header adds that these methods "could trivially be implemented directly by the client". So Skia morphs only between paths of identical structure; matching two arbitrary shapes is the format's problem, not Skia's.
- **Also bound:** `PathEffect::corner_path`, `path_1d` (stamp a path along a path), `pathops::op` and `simplify`.
- **In the pin.** Yes, all of it.
- **Determinism.** **Inferred from code:** dash, trim and interpolate turn one path into another by arithmetic on its points, then the ordinary scan converter fills it.
- **Risks.**
  - `PathEffect::discrete` moves points randomly. **Documented:** the seed is caller-supplied and defaults to 0, "in which case filtering a path multiple times will result in the same set of segments". It is deterministic if the seed is fixed; it should stay out of the vocabulary or carry a literal seed.
  - **Unknown:** whether a path effect's curve subdivision depends on the canvas matrix in a way that differs inside a layer. Nobody measured it.

### 7. Animated mattes, more transition kinds, and the animated `clip`

- **Animated `clip`.** `Canvas::clip_rect`, `clip_rrect`, `clip_path`, each with an antialias flag. Animating one is capability 1's mechanism. The painter already calls `clip_path` with `ClipOp::Difference`.
- **Mattes.**
  - Alpha matte: draw the element in a `save_layer`, then draw the matte with `BlendMode::DstIn` (or `DstOut` to invert). The painter's `mask` effect already does the same shape of thing with `BlendMode::Clear`.
  - Luma matte: the same, with `luma_color_filter::new()` on the matte.
  - `Canvas::clip_shader` clips by a shader's alpha, which gives a soft-edged or gradient clip in one call.
- **Wipe, slide, push.** A wipe is an animated clip or a gradient used as a matte. Slide and push are transforms on the two bridged elements, which is the kind of resolution `canvas.rs` already says belongs to `montagent-core`, as `crossfade` does.
- **In the pin.** Yes.
- **Determinism.** **Inferred from code** for `clip_shader`, `DstIn`/`DstOut` and the luma filter. **Measured** for layer-plus-`Clear` masks and antialiased `clip_path`.
- **Risks.** A matte layer is one more layer, so D5 applies if it is ever bounded. The #652 bound's derivation covers `mask` only as it exists today.

### 8. Motion blur

Skia has no motion-blur draw or filter. I found none in the public headers, and Skottie's effects folder at the pinned commit has no motion-blur file either (it has `DirectionalBlur.cpp`, `GaussianBlurEffect.cpp`, `MotionTileEffect.cpp`). Three constructions are available with the pin.

- **(a) Sub-frame accumulation.** Paint the element, or the frame, at N instants inside the shutter interval and average. In Skia: each sample into a layer at 1/N weight with `BlendMode::Plus`, or N surfaces averaged outside Skia.
  - **Determinism: inferred from code.** Each sample is an ordinary paint at a derived instant, so D6 carries over if the N instants come from the frame number alone. Averaging in 8-bit with `Plus` rounds per sample; averaging in a float buffer outside Skia avoids order effects. Nobody measured either.
  - **Cost:** N paints per blurred frame. The map asks for this number; this note has none.
- **(b) Directional blur.** Rotate, blur on one axis, rotate back: `image_filters::matrix_transform` → `image_filters::blur((σ, 0), …)` → `matrix_transform`. This is how Skottie builds its directional blur ([`DirectionalBlur.cpp`](https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/modules/skottie/src/effects/DirectionalBlur.cpp)).
  - **Determinism: unknown.** It resamples the layer twice and is outside what #649 derived and #652 guards. It also blurs by a fixed vector, so it is a look, not a sample of the motion.
- **(c) An SkSL shader** that takes the element as a child and samples along a velocity uniform, through `image_filters::runtime_shader`.
  - **Determinism: inferred from code**, as capability 9.

### 9. A shader vocabulary

- **API.**
  - `RuntimeEffect::make_for_shader(sksl, options)`, `make_for_color_filter`, `make_for_blender`.
  - `RuntimeEffect::make_shader(uniforms, children, local_matrix)`, or `RuntimeShaderBuilder` with `set_uniform_float` / `set_uniform_int`.
  - `image_filters::runtime_shader(builder, child_name, input)` runs a shader over an element's own pixels, which is what an effect needs.
  - Built-in procedural shaders: `perlin_noise_shader::fractal_noise` and `turbulence`, with a `seed` argument.
- **In the pin.** Yes. The chroma keyer is already `RuntimeEffect::make_for_color_filter`.
- **How it runs on the CPU. Inferred from code:** the SkSL is compiled to a raster-pipeline program, lazily, once per effect: "By using an SkOnce, we avoid thread hazards" ([`SkRuntimeEffect.cpp`](https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/src/core/SkRuntimeEffect.cpp#L218-L222)). No GPU and no external compiler is involved.
- **Determinism.** **Inferred from code:** the program is a pure function of pixel coordinates, uniforms and child samples. **Measured** for one colour-filter effect (the keyer) under K painters. A colour filter reads no coordinates, so that measurement does not cover a shader that does.
- **Risks.**
  - `skia-safe` asserts `RuntimeEffect` is **not** `Send` or `Sync` (`tests/send_sync.rs`). Each painter must compile its own, as each already owns its own canvas.
  - A shader that reads coordinates inside a layer falls under D5: its coordinates are the layer's.
  - The #652 bound gives no hint when any effect is not blur, shadow or mask, so a shader effect on an element turns the bound off for that element.
  - **Unknown:** whether SkSL transcendental functions (`sin`, `pow`, `exp`) give the same bits on every SIMD path. Inside one process they run one path (D3), so this is a cross-machine question only.
  - [`sksl.md`](https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/site/docs/user/sksl.md) documents colour-space handling of uniforms and coordinates conventions; it says nothing about precision or determinism on the CPU.

### 10. A 2.5D perspective transform

- **API.** `M44` with `M44::perspective(near, far, angle)`, `M44::rotate(axis, radians)`, `M44::translate`, `look_at`, `set_concat`; applied by `Canvas::concat_44(&M44)` or `Canvas::set_matrix(&M44)`. `utils::Camera3D` and `Patch3D` are also bound. A per-element tilt and a perspective distance is one `M44` per element, with no camera object and no depth sort.
- **In the pin.** Yes.
- **Determinism.** **Inferred from code** for the draw itself: a matrix is a draw argument. `canvas.rs` already has a sampling rule that names perspective ("a perspective matrix, which this crate never builds, lands here too") and a test for it.
- **With blur, shadow and mask: unknown.** Two facts are known:
  - **Inferred from code:** under perspective Skia does not filter in the element's own space. It says so: "Perspective, which has a non-uniform scaling effect on the filter. Pick a single scale factor that best matches where the filter will be evaluated" ([`SkImageFilterTypes.cpp`](https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/src/core/SkImageFilterTypes.cpp#L93-L101)). So a blur under a tilt is an approximation chosen from one representative point.
  - **Measured scope:** the #652 bound gives no hint under perspective, so tilted blurred elements paint unbounded layers. Their bytes across painters are not covered by any test; nothing read suggests they would differ.
- **Risks.** The map's note that 2.5D "needs its interaction with blur, shadow and masks settled" is exactly the unknown above. Where the perspective matrix sits relative to the effect layers decides it: outside the layers, effects are computed flat and then tilted; inside, Skia's approximation applies.

### 11. Speed ramps and time remapping

- **API.** None in Skia. The map says so: "an editing feature, not a Skia one". It changes which source instant a frame asks the supplier for (ADR-0141's feeds, ADR-0127's frame arithmetic).
- **In the pin.** Not applicable.
- **Determinism.** Not a Skia question. If a ramp ever blends two source frames, that blend is a `Plus` or a cross-fade of two images, which is capability 2's case.

### 12. Vector sources: SVG and Lottie

- **API.**
  - SVG: `skia_safe::svg::Dom` with `Dom::from_bytes` / `from_str` / `read`, `set_container_size`, `render(&Canvas)`.
  - Lottie: `skia_safe::skottie::Animation` with `from_str` / `from_bytes` / `from_file`, `duration`, `fps`, `size`, `seek`, `seek_frame`, `seek_frame_time`, `render(&Canvas, dst)`; `skottie::Builder` for a font manager and a resource provider.
- **In the pin.** **No.** See [the build](#the-build-what-the-pinned-prebuilt-contains): both are Cargo features, both imply `textlayout`, CPU-only `svg` is published for macOS only, and `skottie` for Linux only inside a GPU-and-windowing key.
- **Determinism: unknown.** Neither module's header or doc page says anything about it. What the sources do say:
  - **SVG text needs a font manager. Documented** ([`SkSVGDOM.h`](https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/modules/svg/include/SkSVGDOM.h)): "Specify a font manager for loading fonts (e.g. from the system) to render `<text>`". `skia-safe`'s `Dom` doc gives the two choices: `FontMgr::new_empty()` for no fonts, `FontMgr::default()` for the system's. The system's breaks "a project that renders on my machine renders on a clean one"; the empty one drops text.
  - **SVG images need a resource provider. Documented** in the same header. `skia-safe`'s default one handles typefaces and `data:` URLs; a second one fetches `http(s)://` resources behind the `ureq` feature, which the renderer must not use.
  - **`svg::Dom` is not `Send` or `Sync`** (`tests/send_sync.rs`). Each painter parses its own. `skottie::Animation` is marked `Send + Sync` in `skia-safe`, but `seek` mutates it, so one per painter is still the safe shape.
  - **Lottie time is explicit. Documented:** an animation is positioned by `seek`, `seekFrame` or `seekFrameTime` before `render` ([`Skottie.h`](https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/modules/skottie/include/Skottie.h)). No clock is read. So a chunk start can seek to its own frame.
  - **Lottie expressions are ignored by default. Documented:** "If unspecified, expressions in the animation JSON will be ignored." A file that relies on them renders differently from After Effects, silently.
  - **Unknown:** whether `seek` to frame f gives the same scene after any earlier seek as after none. That is rule 2 of [what byte-identical has to mean](#what-byte-identical-has-to-mean-here), and no source answers it.
  - **Shapes and fills: inferred from code** to be ordinary canvas draws, with the determinism of capabilities 2, 3 and 6.
- **Risks.** The build cost above; fonts; external resources; the unanswered seek question; and an SVG with no absolute size reports zero, as ADR-0027 already records.

## Not found, and not checked

- **No Skia determinism statement.** Not found in the headers and doc pages listed under D1. Skia's wider site and its issue tracker were not searched exhaustively.
- **No measurement in this note.** Every "inferred from code" row is a candidate for the same check #653 uses: paint with K = 3, C = 2 and require one painter's frame hashes. The map's rule already makes each prototype do this.
- **Cache hits versus fresh results (D4).** Not sourced.
- **Path effects, gradient shaders and SkSL coordinates inside a layer whose origin moves (D5).** Not measured for any of them; #649 measured filled paths, images and text under blur and shadow only.
- **Perspective with blur, shadow and mask.** Not measured.
- **Skottie's seek-order independence, and SVG or Lottie byte stability in general.** Not sourced, and not measurable here without changing the pin.
- **Cross-machine bit equality of SkSL math functions.** Not sourced. Outside the rule, which is about painters on one machine.
- **docs.rs was not fetched.** The API names come from the `skia-safe` 0.153.2 crate source itself, which is what docs.rs renders. Links to `rust-skia` files use the `0.153.2` tag, whose commit (`b0260d93e484…`) matches the hash in the prebuilt archive names.
- **No web source returned a 429.**

## Sources

**This repo**

- `Cargo.toml` (the pin and `workspace.metadata.skia`), `crates/montagent-render/Cargo.toml` (`no-skia-source-build`), `crates/montagent-render/tests/skia_pin.rs`.
- `crates/montagent-render/src/canvas.rs` and `crates/montagent-render/src/canvas/layer_bound.rs`.
- `crates/montagent-core/tests/golden_frames.rs` (why bytes are not asserted across targets).
- [ADR-0010](../../adr/0010-skia-safe-rasterizer-text-beside-it.md), [ADR-0027](../../adr/0027-vector-sources-out-of-scope.md), [ADR-0141](../../adr/0141-render-reads-each-video-element-through-a-feed-and-the-painter-takes-its-pixels-from-a-supplier.md), [ADR-0142](../../adr/0142-render-has-one-speed-target-the-benchmark-project-in-three-minutes.md), [ADR-0143](../../adr/0143-render-keeps-libx264-medium-crf-20-pins-five-encoder-threads-and-says-so.md).
- ADR-0144 and `crates/montagent-core/src/verbs/render/painters.rs`, as written for [#653](https://github.com/MBehtemam/Montagent/issues/653) (not on `main` at the time of writing).
- [#649](https://github.com/MBehtemam/Montagent/issues/649)'s `docs/research/filter-bound/FINDINGS.md` on branch `research/649-filter-bound`; [#652](https://github.com/MBehtemam/Montagent/issues/652).

**`rust-skia` 0.153.2** (the published crates, and the same files at tag [`0.153.2`](https://github.com/rust-skia/rust-skia/tree/0.153.2))

- `skia-safe/Cargo.toml` and `skia-bindings/Cargo.toml`: features, and `package.metadata.skia = "m153-0.101.1"`.
- `skia-safe/src/modules.rs`, `src/effects/*.rs`, `src/core/{blend_mode,canvas,m44,paint,path,contour_measure,cubic_map,text_blob}.rs`, `src/utils/camera.rs`, `src/modules/svg/dom.rs`, `src/modules/skottie.rs`.
- `skia-safe/tests/send_sync.rs`.
- `skia-bindings/build_support/{features.rs,binaries_config.rs,binary_cache/binaries.rs,binary_cache/env.rs}` and `skia-bindings/README.md`.
- Release assets of [`rust-skia/skia-binaries` `0.153.2`](https://github.com/rust-skia/skia-binaries/releases/tag/0.153.2).

**Skia at [`c9c3c5a9`](https://github.com/rust-skia/skia/tree/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6)** (the commit the pin builds)

- `include/core/`: `SkBlendMode.h`, `SkCanvas.h`, `SkExecutor.h`, `SkGraphics.h`, `SkImage.h`, `SkM44.h`, `SkPaint.h`, `SkPath.h`, `SkPicture.h`, `SkShader.h`, `SkSurface.h`.
- `include/effects/`: `SkDashPathEffect.h`, `SkDiscretePathEffect.h`, `SkGradient.h`, `SkPerlinNoiseShader.h`, `SkRuntimeEffect.h`, `SkTrimPathEffect.h`.
- `src/core/`: `SkOpts.cpp`, `SkExecutor.cpp`, `SkRuntimeEffect.cpp`, `SkImageFilterTypes.cpp`, `SkResourceCache.h`; `src/opts/SkRasterPipeline_opts.h`.
- `modules/svg/include/SkSVGDOM.h`, `modules/skottie/include/Skottie.h`, `modules/skottie/src/effects/` (listing, and `DirectionalBlur.cpp`).
- `site/docs/user/`: `sksl.md`, `tips.md`, `api/skcanvas_overview.md`, `api/skpaint_overview.md`, `modules/skottie.md`.
