# Which named effects Premiere and CapCut ship that Montagent lacks, and which Skia could paint

Research for [#703](https://github.com/MBehtemam/Montagent/issues/703), part of the map
[#663](https://github.com/MBehtemam/Montagent/issues/663).

**Date of research:** 2026-10-06. Nothing was built or run. This is a reading of vendor help
pages, the `skia-safe` 0.153.2 source in the cargo registry, and Skia at the pinned commit.

**This note decides nothing.** It gives the shader ticket a candidate list. It builds on two
earlier notes and does not repeat them:

- [#664](https://github.com/MBehtemam/Montagent/issues/664): `docs/research/skia-reach/capcut-premiere-mechanisms.md`
  on branch [`research/664-capcut-premiere-mechanisms`](https://github.com/MBehtemam/Montagent/blob/research/664-capcut-premiere-mechanisms/docs/research/skia-reach/capcut-premiere-mechanisms.md).
  It found that a closed library of named effects with typed parameters is the
  reference-class mechanism, and that CapCut publishes no effect list.
- [#665](https://github.com/MBehtemam/Montagent/issues/665): `docs/research/skia-reach/skia-safe-exposure.md`
  on branch [`research/665-skia-safe-exposure`](https://github.com/MBehtemam/Montagent/blob/research/665-skia-safe-exposure/docs/research/skia-reach/skia-safe-exposure.md).
  It found that every image filter, runtime effect and the Perlin shaders are in the pinned
  prebuilt with no feature change, and named the determinism risks.

## Answer

- **Premiere ships a long named list; CapCut does not publish one.** Premiere 26.0's table
  names about 90 video effects by category
  ([List of effects and transitions][pr-list]). CapCut has no reference manual. Its own pages
  confirm a searchable Effects panel with named presets and a strength slider, but no list and
  no parameter names ([Vignette effect][cc-vignette]).
- **Parameters are mostly undocumented for Premiere 26.** Adobe's new Premiere pages give a
  one-line description per effect and almost no controls. The parameter names below come
  from After Effects' pages for the same-named effect. That is marked "AE" in each row. That
  Premiere uses the same names is an inference, not a quote.
- **Montagent already covers more than its eight `effects` members.** Gradients are a paint
  ([ADR-0149](../../adr/0149-a-gradient-is-a-paint-linear-or-radial-measured-against-the-declared-box.md))
  and `screen`/`add`/`multiply`/`overlay` are a blend field
  ([ADR-0147](../../adr/0147-a-blend-mode-is-a-flat-static-field-of-five-values-and-the-finished-element-blends-last.md)).
  So a vignette, a simple light leak and an alpha glow can already be built from elements.
  Black & White, Tint and Brightness & Contrast are ADR-0049 members.
- **Most candidates are one Skia call or one small runtime shader.** Posterize, Invert,
  Gamma, Sharpen, Find Edges, Emboss, Magnify, Offset and Corner Pin map to a built-in filter
  or matrix. Twirl, Wave Warp, Ripple, Spherize, Mirror, Mosaic and RGB Split need a
  `RuntimeEffect` shader. Glow is a built-in filter graph.
- **Noise is the dividing line for seed and time.** Every grain, fractal-noise, turbulent or
  "random" effect needs a seed. Every effect Adobe says animates "without keyframes" (Wave
  Warp, Ripple, Turbulent Displace's Evolution, animated Noise) needs the frame instant as an
  input. Skia's built-in Perlin shader takes a seed but has no time axis; Skia's own After
  Effects importer builds Evolution in SkSL instead.
- **Pre-rendered motion graphics have precedent in Premiere only.** A `.mogrt` is a file made
  in Premiere or After Effects with exposed controls
  ([Motion Graphics templates][pr-mogrt]). CapCut's templates live in an online library and
  depend on "mobile-only rendering engines"; they are not footage files
  ([Template shows one frame][cc-template-pc]).

## How to read the tables

- **Premiere** names the effect's category in the 26.0 table ([List][pr-list]). "Modern" means
  it is on the Modern effects page ([Modern effects][pr-modern]). "Legacy" and "Obsolete" are
  Adobe's own labels from the 26.0 table.
- **CapCut** is "named" only when a capcut.com page names it. "Not found" means no capcut.com
  page I reached names it. It does not mean CapCut lacks it.
- **Parameters** are quoted names. "AE" marks names taken from After Effects' page. "None
  documented" means neither Premiere's page nor an AE page for that effect gave controls.
- **Skia route** names the `skia-safe` 0.153.2 call. Paths are under
  `~/.cargo/registry/src/index.crates.io-*/skia-safe-0.153.2/src/`. Three routes:
  - **filter:** a member of `effects/image_filters.rs` or `core/color_filter.rs` (`color_filters`).
  - **perlin:** `effects/perlin_noise_shader.rs` (`fractal_noise`, `turbulence`).
  - **runtime:** an SkSL shader through `RuntimeEffect::make_for_shader`
    (`effects/runtime_effect.rs:163`) applied to the element with
    `image_filters::runtime_shader` (`effects/image_filters.rs:489`).
- **Seed / time.** "seed" means the look depends on a random source, which must be a literal
  in the document to stay deterministic. "time" means the effect changes with no keyframes,
  so it must read the element-local instant. Keyframed parameters
  ([ADR-0146](../../adr/0146-an-animatable-property-is-one-the-schema-types-so-colour-blends-premultiplied-and-spring-easing-is-refused.md))
  are not a time input.

## Generators

| Effect | Premiere | CapCut | Parameters | In Montagent? | Skia route | Seed / time |
|---|---|---|---|---|---|---|
| **Noise / grain** | Noise & Grain: Noise (Modern) | not found | Premiere: overall level; spread over shadows, midtones, highlights; noise saturation; blend Normal, Screen, Additive, Soft Light; preserve alpha ([Modern effects][pr-modern]). AE Noise: Amount of Noise, Use Color Noise, Clipping ([AE Noise & Grain][ae-noise]) | No | **runtime.** Per-pixel hash of (x, y, seed, frame). The Perlin shaders are smooth noise, not per-pixel grain | seed and time. Premiere calls it "controlled, animated noise" |
| **Fractal noise** | Only VR Fractal Noise (Immersive). Turbulent Displace uses it internally | not found | AE Fractal Noise: Fractal Type, Noise Type, Invert, Contrast, Overflow, Transform, Complexity, Sub Settings, Evolution, Evolution Options, Random Seed, Opacity, Blending Mode ([AE Noise & Grain][ae-noise]) | No | **perlin** `fractal_noise(base_frequency, num_octaves, seed, tile_size)` or `turbulence(…)`. No Evolution axis: Skia's Skottie builds Evolution as SkSL ([FractalNoiseEffect.cpp][sk-fractal]) | seed. time if Evolution animates |
| **Gradient / Ramp / 4-Color Gradient** | Generate: 4-Color Gradient; Ramp (Legacy); Gradient (Modern) | gradient text and fills (#664 §3) | 4-Color Gradient: "Positions and Colors" of four points; Ramp: start and end, colours, Ramp Scatter ([Generate effects][pr-generate]); Gradient: position, scale, orientation, repetition, mirroring, interpolation, feathering, grain ([Modern effects][pr-modern]) | **Yes**, linear and radial paint (ADR-0149). Four-point gradient: no | linear/radial: already drawn. 4-colour: **runtime** (inverse-distance blend of four points); no single Skia gradient does it | none. "Ramp Scatter" and Gradient's "grain" add seed |
| **Light leaks** | Lights & Glows: Light Leaks (Modern) | not found | "Add glow and occasional flare" ([Modern effects][pr-modern]). Controls: none documented | **Partly.** A radial gradient element with `blend: screen` or `add` (ADR-0147 names light leaks as its motive) | Composition of `radial_gradient` shaders and `BlendMode::Screen`/`Plus`. Or **runtime** | "occasional" implies time and seed. A keyframed composition needs neither |
| **Lens flare** | Lens Flare FX (Modern). The old Lens Flare is Obsolete ([List][pr-list]) | not found | "flare shape, streak intensity, and color" ([Modern effects][pr-modern]) | No | **runtime**, or several radial gradients blended `Plus` | none |
| **Volumetric rays, Glint** | Lights & Glows (Modern) | not found | None documented. Glint: "brief sparkling" ([Modern effects][pr-modern]) | No | **runtime** | Glint: time and likely seed |
| **Lightning** | Generate page lists it; the 26.0 table marks it Obsolete | not found | "animates automatically ... without keyframes" ([Generate effects][pr-generate]) | No | **runtime** | seed and time |

## Stylisers

| Effect | Premiere | CapCut | Parameters | In Montagent? | Skia route | Seed / time |
|---|---|---|---|---|---|---|
| **Glow** | Lights & Glows: Echo Glow, Edge Glow, Wonder Glow, Focus Glow (Modern); Alpha Glow (Legacy) | not found | Premiere: none documented. AE Glow: Glow Based On, Threshold, Radius, Intensity, Composite Original, Glow Colors, Color Looping, Color Loops, Color Phase, A & B Midpoint, Glow Dimensions ([AE Stylize][ae-stylize]) | **Partly.** `shadow` with `dx = dy = 0` is an alpha-based outer glow. A brightness-threshold glow is not | **filter** graph: `color_filter` (threshold matrix) → `blur` → `blend`/`merge` with source. Skia's Skottie builds its glow the same way ([GlowStyles.cpp][sk-glow]) | none. Echo Glow's "trailing" may read earlier frames: not documented |
| **Vignette** | Color Correction: Vignette (Modern) | **named:** Effects panel, search "vignette", strength slider ([Vignette effect][cc-vignette]) | Premiere: "darken the edges" only ([Modern effects][pr-modern]). CapCut: strength | **Composable** today: a radial gradient element to transparent with `blend: multiply` | `radial_gradient` + `BlendMode::Multiply`, or **runtime** | none |
| **Posterize** | Stylize: Posterize | not found | "number of tonal levels ... Values range from 2 to 255" ([Stylize][pr-stylize]); AE: Level ([AE Stylize][ae-stylize]) | No | **filter** `color_filters::table` / `table_argb` (`core/color_filter.rs:201`, `:209`): a 256-entry lookup per channel, exact | none |
| **Invert, Gamma Correction** | Image Control | not found | None documented | No | **filter** `color_filters::matrix` (invert) or `table` (gamma) | none |
| **Mosaic / pixelate** | Stylize: Mosaic (Modern) | not found | Premiere: "polygonal or grid-based segmentation", Softness ([Modern effects][pr-modern]). AE: Horizontal Blocks, Vertical Blocks, Sharp Colors ([AE Stylize][ae-stylize]) | No | **runtime**: sample the child at each block's centre. The polygonal mode needs a cell function, also runtime | grid: none. polygonal cells likely seed |
| **RGB split / chromatic aberration** | Lights & Glows: RGB Split (Modern) | glitch preset, marketing article only ([Glitch effect][cc-glitch]) | "Split color channels" ([Modern effects][pr-modern]); none documented | No | **filter**: three `offset` copies, each through a channel `color_filters::matrix`, joined by `blend` with `Plus`. Or **runtime** | none. CapCut glitch: "intensity, frequency, and color" suggests time |
| **Find Edges, Emboss, Sharpen** | Stylize: Find Edges, Color Emboss; Blur & Sharpen: Sharpen, Unsharp Mask | not found | None documented on Premiere's pages | No | **filter** `matrix_convolution` (`effects/image_filters.rs:380`). Skottie's Sharpen uses a 3×3 kernel ([SharpenEffect.cpp][sk-sharpen]). Emboss can also use `distant_lit_diffuse` (`:639`) | none |
| **Stroke (outline on any element)** | Utility: Stroke (Modern) | not found | "customizable outlines to any element" ([Modern effects][pr-modern]) | **Partly.** Stroke is paint on shapes and text ([ADR-0014](../../adr/0014-stroke-is-paint-the-text-box-is-required.md)), not on images or video | **filter** `dilate` (`:593`) on alpha → colour fill → under source | none |
| **Long Shadow** | Perspective: Long Shadow (Modern) | not found | "extended shadows to flat graphics" ([Modern effects][pr-modern]) | No | **runtime** (march along a direction), or many `offset` copies merged | none |
| **Strobe Light** | Stylize | not found | "at periodic or random intervals" ([Stylize][pr-stylize]) | No | `color_filters` on a schedule. Not a shader problem | time; random mode needs seed |
| **Roughen Edges, Brush Strokes** | Stylize | not found | Brush Strokes strokes "are scattered randomly by a small amount" ([Stylize][pr-stylize]) | No | **perlin** turbulence → `displacement_map` on alpha (Roughen); Brush Strokes: **runtime** | seed |

## Distortions

| Effect | Premiere | CapCut | Parameters | In Montagent? | Skia route | Seed / time |
|---|---|---|---|---|---|---|
| **Wave Warp** | Distort: Wave Warp | not found | AE: Wave Type, Wave Height, Wave Width, Direction, Wave Speed, Pinning, Phase, Antialiasing ([AE Distort][ae-distort]). Premiere: "automatically animated at a constant speed ... without keyframes" ([Distort][pr-distort]) | No | **runtime** with `runtime_shader_with_options(sample_radius = wave height)` (`:521`) | **time** (Wave Speed). Phase keyframed instead needs none |
| **Ripple** | **not in Premiere 26** | not found | AE only: Radius, Center Of Ripple, Type Of Conversion, Wave Speed, Wave Width, Wave Height, Ripple Phase ([AE Distort][ae-distort]) | No | **runtime** | time (Wave Speed) |
| **Twirl** | Distort: Twirl (Modern) | not found | AE: Angle, Twirl Radius ([AE Distort][ae-distort]); Premiere: "radius and angle" around a "center point" ([Modern effects][pr-modern]) | No | **runtime** | none |
| **Turbulent Displace** | Distort: Turbulent Displace (Modern) | not found | AE: Displacement, Amount, Size, Offset, Complexity, Evolution, Evolution Options, Cycle Evolution, Cycle, Random Seed, Pinning, Resize Layer ([AE Distort][ae-distort]) | No | **perlin** `turbulence` as a shader filter → **filter** `displacement_map` (`:212`). Both built in. Evolution needs **runtime** | **seed** (Random Seed); **time** if Evolution animates |
| **Spherize / Bulge** | Distort: Spherize | not found | Premiere: none documented. AE Bulge: Horizontal and Vertical Radius, Bulge Height, Taper Radius, Antialiasing, Pin All Edges ([AE Distort][ae-distort]) | No | **runtime**. Skottie's Bulge and Sphere are SkSL ([BulgeEffect.cpp][sk-bulge]) | none |
| **Lens Distortion** | Distort | not found | None documented | No | **runtime** | none |
| **Magnify** | Distort: Magnify (Modern); Magnify (Legacy) | not found | Premiere: "lens size, magnification, edge softness, border, shadow, and position" ([Modern effects][pr-modern]) | No | **filter** `magnifier(lens_bounds, zoom_amount, inset, …)` (`:344`), round lens via **runtime** | none |
| **Mirror** | Distort: Mirror | not found | AE: Reflection Center, Reflection Angle ([AE Distort][ae-distort]) | No | **runtime**, or a clip plus a reflected redraw | none |
| **Offset (wrap-around pan)** | Transform: Offset | not found | AE: Shift Center To, Blend With Original ([AE Distort][ae-distort]) | No | `Image::to_shader` with `TileMode::Repeat` and a translate (`core/image.rs:628`) | none |
| **Corner Pin** | Distort: Corner Pin | not found | four corners ([Distort][pr-distort]) | No | `Matrix::from_poly_to_poly` (`core/matrix.rs:591`), a perspective matrix: #665's 2.5D risks apply | none |
| **Displacement Map** | **not in Premiere 26** | not found | AE only: Displacement Map Layer, channel selectors, max displacement, Wrap Pixels Around ([AE Distort][ae-distort]) | No | **filter** `displacement_map` | none, but it reads another layer: [ADR-0150](../../adr/0150-wipe-slide-and-push-enter-as-transition-kinds-and-a-matte-from-another-element-is-refused.md) refuses a matte from another element for the same reason |
| **Camera Shake, Wiggle** | Transform (Modern) | "shake" not found | "randomized" motion ([Modern effects][pr-modern]) | No | Not Skia. A transform per frame, resolved in `montagent-core` | seed and time |

## Already in Montagent

| Montagent | Premiere counterpart | Source |
|---|---|---|
| `blur` | Gaussian Blur, Blur FX | [ADR-0040](../../adr/0040-effect-model-attachment-and-v1-vocabulary.md) |
| `shadow` | Drop Shadow | ADR-0040 |
| `mask` (rect, circle, ellipse) | Rounded Crop, shape masks | ADR-0040, [ADR-0084](../../adr/0084-the-mask-rect-is-one-shape-independent-parameter-set.md) |
| `tint`, `saturation`, `brightness`, `contrast` | Tint, Black & White (saturation 0), Brightness & Contrast | [ADR-0049](../../adr/0049-v1-colour-filter-vocabulary-four-scalar-members.md) |
| `chroma` | Ultra Key, Color Key | [ADR-0088](../../adr/0088-chroma-is-a-matte-operation-and-color-stays-literal.md) |
| linear and radial gradient paint | Ramp, Gradient | ADR-0149 |
| `blend` of five values | Opacity blend modes | ADR-0147 |
| `crossfade`, wipe, slide, push | Cross Dissolve, wipes, Push, Slide | ADR-0150 |

The vocabulary lives in `crates/montagent-core/src/model/effects.rs` (`enum Effect`).

## Seed and time: what the sources say

- **Skia's Perlin seed is deterministic.** `SkPerlinNoiseShaderImpl.h` truncates the seed to an
  integer "According to the SVG spec" and clamps it, then runs a fixed linear congruential
  generator ([SkPerlinNoiseShaderImpl.h, lines 124-132][sk-perlin-impl]). The header says the
  algorithm is the SVG `feTurbulence` one ([SkPerlinNoiseShader.h][sk-perlin-h]). So a literal
  seed in the document gives the same noise on every painter. **Inferred from code.**
- **Skia's Perlin noise is 2-D.** `MakeFractalNoise` and `MakeTurbulence` take base frequency,
  octaves, seed and tile size, and nothing else ([SkPerlinNoiseShader.h][sk-perlin-h]). There is
  no third axis to animate smoothly. Changing the seed per frame flickers, which is what AE
  warns of: animating Random Seed "results in flashing" ([AE Noise & Grain][ae-noise]).
- **Smooth evolution needs SkSL.** Skottie's Fractal Noise computes "noise planes ... based on
  evolution params" with a hash in SkSL, not with the Perlin shader
  ([FractalNoiseEffect.cpp, lines 66-105][sk-fractal]). Skottie is not in the pin (#665), but its
  SkSL source is a worked example a Montagent runtime shader could follow.
- **"Animates without keyframes" is a time input.** Adobe says this of Wave Warp
  ([Distort][pr-distort]), Ripple's Wave Speed and Turbulent Displace's Evolution
  ([AE Distort][ae-distort]), and Lightning ([Generate][pr-generate]). In Montagent, the value must
  come from the element-local instant, so #653's rule (a frame's bytes depend on the document and
  the instant only) holds. No current member reads the instant.
- **A literal phase avoids the time input.** Wave Warp and Ripple both document a Phase
  parameter and say: to vary speed, set speed to 0 and keyframe Phase
  ([AE Distort][ae-distort]). With ADR-0146's keyframes, a Phase parameter covers the motion
  without the effect reading time.

## Pre-rendered motion graphics

**Premiere: a `.mogrt` file with exposed controls.** "A Motion Graphics template is a file type
(.mogrt) that can be created in Adobe Premiere or Adobe After Effects." It is "Packaged as
templates with easy-to-use controls designed to be customized in Premiere" and comes from a
local folder, Creative Cloud Libraries or Adobe Stock ([Motion Graphics templates][pr-mogrt]).
#664 also cites Dynamic Link and Render and Replace, which bake an After Effects composition into
a clip.

What I did not find: whether Premiere renders an After Effects-made `.mogrt` with an After
Effects engine or as footage. The overview page does not say.

**CapCut: templates are not footage.** Templates are browsed and opened with "Use Template"
inside the app ([Newest CapCut templates][cc-new-templates]). On desktop, a project from a
mobile template may show "a single static image or blank/gray frames", because desktop "cannot
fetch dynamic template assets (like motion graphics, transitions, or placeholder animations)" and
"some visual elements rely on mobile-only rendering engines" ([Template shows one frame][cc-template-pc]).
So a CapCut template is a project tied to a renderer and an online library, not a portable clip.

**For the pre-render skill:** Premiere's model (a piece built elsewhere, arriving with a few
exposed controls, or baked to a clip) is the precedent. CapCut's is a counter-example: its
templates break when the renderer that made them is absent.

## Could not be sourced

- **CapCut's effect list and every CapCut parameter.** CapCut publishes no reference manual. The
  only named effect with a documented control is the vignette's strength slider. The glitch
  parameters come from a marketing article ([Glitch effect][cc-glitch]). Pages for glow, grain,
  light leak, shake and noise returned 404 at the URLs tried.
- **Premiere 26 parameters for almost every effect.** The new pages carry one line each. Names
  above marked "AE" are from After Effects pages.
- **Premiere's Vignette controls, Glow controls (Echo, Edge, Wonder, Focus), Light Leaks, Lens
  Flare, RGB Split and Volumetric Rays.** None documented on the pages read.
- **Whether Premiere's Modern Noise is seeded, and whether it changes every frame by default.**
  The page says "animated" but names no seed control.
- **After Effects' Gradient Ramp and 4-Color Gradient controls.** The AE Generate page did not
  yield them in this session. Gradients are already in Montagent, so this was not pursued.
- **An inconsistency in Adobe's pages.** The Generate page still lists Lens Flare, Lightning and
  Ramp. The 26.0 table marks Lens Flare and Lightning Obsolete and Ramp Legacy.
- **Access.** Adobe's help server returned HTTP 403 to plain HTTP clients from this machine, and
  the third-party page reader was out of credit. All Adobe and CapCut pages were read in a
  browser tab. Nothing came from search summaries.
- **No byte checks.** Every "Skia route" is read from source. None was run under K painters.

## Sources

**Adobe, read 2026-10-06**

- [Premiere: List of effects and transitions (26.0)][pr-list]
- [Premiere: Modern effects][pr-modern]
- [Premiere: Generate effects][pr-generate]
- [Premiere: Distort effects][pr-distort]
- [Premiere: Stylize effects][pr-stylize]
- [Premiere: Noise and Grain effects](https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/noise-and-grain-effects.html) (one line: Noise "adds random pixels")
- [Premiere: Overview of Motion Graphics templates][pr-mogrt]
- [After Effects: Distort effects][ae-distort]
- [After Effects: Noise and Grain effects][ae-noise]
- [After Effects: Stylize effects][ae-stylize]

**CapCut, read 2026-10-06**

- [Vignette effect][cc-vignette] (feature page)
- [Where can I find the newest templates][cc-new-templates] (help article)
- [Why does a downloaded template only show one frame][cc-template-pc] (help article; the misspelt URL is CapCut's)
- [Add a glitch effect][cc-glitch] (marketing article, weaker)

**Skia at [`c9c3c5a9`](https://github.com/rust-skia/skia/tree/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6), the commit the pin builds**

- [`include/effects/SkPerlinNoiseShader.h`][sk-perlin-h]
- [`src/shaders/SkPerlinNoiseShaderImpl.h`, lines 124-132][sk-perlin-impl]
- [`modules/skottie/src/effects/FractalNoiseEffect.cpp`, lines 66-105][sk-fractal]
- [`modules/skottie/src/effects/GlowStyles.cpp`, lines 91-131][sk-glow]
- [`modules/skottie/src/effects/SharpenEffect.cpp`, line 55][sk-sharpen]
- [`modules/skottie/src/effects/BulgeEffect.cpp`, lines 90-92][sk-bulge]
- The full Skottie effects folder at this commit lists Bulge, DisplacementMap, FractalNoise, Glow, Sharpen, Sphere, Threshold and others, each an After Effects effect built on Skia.

**`skia-safe` 0.153.2** (cargo registry copy; same files at
[tag `0.153.2`](https://github.com/rust-skia/rust-skia/tree/0.153.2/skia-safe/src))

- `src/effects/image_filters.rs`: `displacement_map` :212, `magnifier` :344,
  `matrix_convolution` :380, `runtime_shader` :489, `runtime_shader_with_options` :521,
  `shader` :555, `tile` :573, `dilate` :593, `erode` :614, `distant_lit_diffuse` :639.
- `src/effects/perlin_noise_shader.rs`: `fractal_noise` :3, `turbulence` :12.
- `src/effects/runtime_effect.rs`: `make_for_color_filter` :149, `make_for_shader` :163.
- `src/core/color_filter.rs`: `color_filters::matrix` :147, `table` :201, `table_argb` :209.
- `src/core/matrix.rs`: `from_poly_to_poly` :591. `src/core/image.rs`: `to_shader` :628.

[pr-list]: https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/list-of-effects-and-transitions.html
[pr-modern]: https://helpx.adobe.com/premiere/desktop/add-video-effects/types-of-effects/effects.html
[pr-generate]: https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/generate-effects.html
[pr-distort]: https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/distort-effects.html
[pr-stylize]: https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/stylize-effects.html
[pr-mogrt]: https://helpx.adobe.com/premiere/desktop/add-text-images/use-motion-graphics-templates/about-motion-graphics-templates.html
[ae-distort]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/distort-effects.html
[ae-noise]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/noise-grain-effects.html
[ae-stylize]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/stylize-effects.html
[cc-vignette]: https://www.capcut.com/tools/vignette-effect
[cc-new-templates]: https://www.capcut.com/help/new-templates
[cc-template-pc]: https://www.capcut.com/help/download-tempalte-problem
[cc-glitch]: https://www.capcut.com/resource/glitch-effect
[sk-perlin-h]: https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/include/effects/SkPerlinNoiseShader.h
[sk-perlin-impl]: https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/src/shaders/SkPerlinNoiseShaderImpl.h#L124-L132
[sk-fractal]: https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/modules/skottie/src/effects/FractalNoiseEffect.cpp#L66-L105
[sk-glow]: https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/modules/skottie/src/effects/GlowStyles.cpp#L91-L131
[sk-sharpen]: https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/modules/skottie/src/effects/SharpenEffect.cpp#L55
[sk-bulge]: https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/modules/skottie/src/effects/BulgeEffect.cpp#L90-L92
