# After Effects census: colour, colour management, and the colour, blur, channel and utility effects

Research for [#904](https://github.com/MBehtemam/Montagent/issues/904), part of the map
[#896](https://github.com/MBehtemam/Montagent/issues/896).

**Date of research:** 2026-10-10. Nothing was built or run. This note scores features. It
decides nothing.

**After Effects version read:** 26.5, the September 2026 release. Adobe's [What's new][whats-new]
lists it first, and says that release brings native ACES 2.0 support through OpenColorIO 2.5.1.

**Sources.**

- Adobe's After Effects user guide: the [effect list][list] (each effect's category, its
  8/16/32-bpc support and whether it runs on the GPU), the category pages
  [Color Correction][cc], [Blur & Sharpen][bs], [Channel][ch], [Utility][ut] and
  [Obsolete][obs], and [Cycore plugins][cyc], which is Adobe's only text on the CC effects. For
  colour management, the user guide's colour chapter: [Color basics][basics],
  [Color management][cm], [OpenColorIO and ACES][ocio], [Enhanced HDR support][hdr] and
  [Import and export HDR content][hdrio]. For Lumetri Color's panel, Premiere's
  [Basic color correction options][pr-lumetri].
- Montagent at `main` `51aa4b5`: `CONTEXT.md`, `docs/adr/`, `crates/montagent-core/docs/format.md`
  and its page `format/compositing.md`, `skills/*/SKILL.md`, `schema/montagent.schema.json`,
  and, for the pixel pipeline, `crates/montagent-render/src/{canvas.rs,encode.rs,decode.rs}`.
- The earlier Premiere census, [#703](https://github.com/MBehtemam/Montagent/issues/703)
  (`docs/research/skia-reach/named-effects-candidates.md` on branch
  `research/703-named-effects-candidates`), and ADR-0156's growth path.

**How Adobe was read.** The first draft was written while the egress proxy refused
`helpx.adobe.com`, from search-engine extracts. This version was verified against the live
pages on 2026-10-10. Every row links to the section of the page it was checked against, and
every bit depth and quoted sentence comes from those pages. Still unverified, and marked †, are
the parts of five CC descriptions that Adobe's one-line Cycore entries do not state: the centre
point of CC Radial Blur and CC Radial Fast Blur, CC Vector Blur's vector field, CC Composite
compositing over its own original, and CC Kernel being a 3×3 kernel. Where the effect list and
an effect's own page disagree on bit depth, both are given under Not sourced.

**Membership, reconciled against the effect list.** Color Correction: the list's table has 35
entries and all 35 are here. Five OCIO transform effects are documented on the Color Correction
page and the OpenColorIO page ("The OCIO Look Transform effect under the Color Correction
category"), but they are missing from the table. They are added here, which makes 40. Blur &
Sharpen: 16 in the table, all here, plus two blurs from Obsolete. The page's "Box Blur effect"
section is Fast Box Blur under its old name (match name `ADBE Box Blur`), not a seventeenth
effect. Channel: 13, all here. Utility: 7, all here. No OCIO effect is in Utility.

## How Montagent's colour pipeline stands today

These facts come from the repo. They frame every row below.

- **Everything is 8-bit sRGB, and no document names a working space.** The painter draws into
  `ColorType::RGBA8888` surfaces (`crates/montagent-render/src/canvas.rs`). A keyed colour
  interpolates "in sRGB with premultiplied alpha" (ADR-0146). Gradients use the same rule
  (ADR-0149). `blend` "runs on the stored sRGB values, with no linearisation" (ADR-0147). No
  ADR decides a working colour space. sRGB is assumed everywhere, never chosen.
- **Footage comes in through `ffmpeg`, as `rgba`, with no colour rules of Montagent's own.** The
  decoder asks `ffmpeg` for `-f rawvideo -pix_fmt rgba` (`decode.rs`). Montagent passes no
  colour matrix, range or transfer arguments. ADR-0141's parity gate "did not cover VP9 with
  alpha, interlaced sources or full-range sources", and covered one 10-bit codec (ProRes 422).
- **The deliverable is 8-bit `yuv420p` H.264, untagged.** The encoder pipes `rgb24` into
  `libx264` with `-pix_fmt yuv420p` (ADR-0077 §7, `encode.rs`). It writes no `-colorspace`,
  `-color_primaries`, `-color_trc` or `-color_range`. So the RGB-to-YUV matrix is whatever
  `ffmpeg` defaults to, and players have to guess it. No ADR names this.
- **The colour-filter vocabulary is closed by a stopping rule.** ADR-0049 admits a colour
  operation only if (a) its parameter count is fixed by its name, (b) every parameter is a
  bounded scalar, (c) it has a documented identity value, and (d) it cannot be composed from
  two or more admitted members. "Curves, LUTs, per-channel colour matrices, and duotone-style
  two-colour maps are categorically excluded." `tint.color` is the only colour-typed parameter
  it allows, and that exception "is closed". ADR-0049 names `gamma{value}` as an example of a
  legitimate future scalar that the rule would admit.
- **ADR-0040 refuses LUTs separately**, because their "semantics live entirely in an external
  `.cube` file, which is a plugin in all but name".
- **ADR-0156's growth path** lets a new `effects` member in, one ADR at a time, if it passes
  three tests: it cannot already be composed, it is "one built-in Skia filter, or one small
  runtime shader", and it stays inside its box or a reach it declares. ADR-0156 deferred
  sharpen (with emboss, mosaic and RGB split) "to keep the first batch small", and deferred
  the distortions. It refused Displacement Map because "it reads another element's pixels",
  citing ADR-0150.
- **The pre-render route** (`montagent-prerender`, ADR-0156 §7) bakes any look drawn in code
  into lossless footage with alpha. The code can read a source file too. So, in principle,
  every per-pixel gap below that filters footage can be baked this way, at the cost of large
  files and an edit loop outside the project. The table marks a row "reachable" only where
  the capability map or a skill names that look. It does not mark every gap reachable on this
  general ground.

**Status words.** **covered**: a Montagent key gives the AE result. **partial**: some of it.
**reachable**: by a named skill or a documented workaround. **refused by ADR-NNNN**: an
accepted ADR says no. **gap**: nothing covers it, and no ADR or open map owns it. No row here
is owned by an open map. The transforms map
([#591](https://github.com/MBehtemam/Montagent/issues/591)) refused the adjustment-layer shape
of a nest, which is how AE usually applies a grade to everything below it. That bears on
how a grade would be applied, not on any one effect.

**Counts, over 93 rows:** covered 8, partial 12, reachable 2, owned by an open map 0,
refused 16 (ADR-0150: 6, ADR-0049: 5, ADR-0040: 3, ADR-0147: 2), gap 55.

## Bit depth, linear light and colour management

| Feature | What it does | Adobe doc | Montagent status | Evidence |
|---|---|---|---|---|
| 8-bpc project | 8 bits per channel, values 0–255 | [Color basics][basics-depth] | covered | RGBA8888 painter (`canvas.rs`); `yuv420p` output (ADR-0077 §7) |
| 16-bpc project | 16 bits per channel (0–32,768); higher precision that "greatly reduces quantization artifacts, such as banding in gradients" | [Color basics][basics-depth] | gap | No ADR or format key; every surface is RGBA8888 |
| 32-bpc float project | Floating-point channels that "can have values under 0.0 and values over 1.0", so HDR and overbrights survive a grade | [Color basics][basics-depth] | gap | As above. No key stores an out-of-range value: colours are `#RRGGBB(AA)` (ADR-0014) and effect results clamp to 0–1 (`format/compositing.md`, Grain and Named effects) |
| Linearize working space | All colour operations in linear light, which can "prevent certain edge and halo artifacts"; Adobe says it "works best with higher color depths—16 bpc and 32 bpc—and is not recommended for 8-bpc color" | [Color management][cm-linear] | refused by ADR-0147 | ADR-0147 refuses "Linear-light arithmetic", because "it differs from the reference editors' default"; ADR-0146 keys colour in sRGB |
| Blend colors using 1.0 gamma | Linear-light blending that "affects only blending between layers" | [Color management][cm-linear] | refused by ADR-0147 | Same refusal; `blend` "runs on the stored sRGB values" (`format/compositing.md`) |
| Working colour space (Adobe colour-managed) | The space "all of its color operations" run in; setting it to None turns colour management off | [Color management][cm-working] | gap | sRGB is assumed in ADR-0146, 0147 and 0149, but no ADR chooses a working space or weighs another |
| OCIO colour-managed mode, ACES | OpenColorIO engine, ACES workflows (ACES 2.0 and OCIO 2.5.1 as of 26.5), separate working and display spaces | [OpenColorIO and ACES][ocio], [What's new][whats-new] | gap | No ADR, key or skill mentions OCIO or ACES (grep of `docs/adr`, `CONTEXT.md`, `skills`) |
| Input colour interpretation | Read a footage item's embedded profile or colour space, or assign one in Interpret Footage or the interpretation rules file | [Color management][cm-input] | gap | `decode.rs` asks `ffmpeg` for `rgba` with no colour arguments; ADR-0141's gate excludes full-range sources; `probe` reports no colour fields |
| Interpret As Linear Light | Treat a footage item's profile as gamma 1.0; works even with colour management off | [Color management][cm-input] | gap | As above |
| Output colour profile | Convert the render to an output space, and embed the colour space in the file | [Color management][cm-output] | gap | `encode.rs` writes untagged `yuv420p` with `ffmpeg`'s default matrix; ADR-0077 §7 fixes `yuv420p` and says nothing on colour |
| Display colour management | Convert the working space for the viewer, "separate from the render output" | [OpenColorIO and ACES][ocio], [HDR][hdr] | covered (trivially) | No viewer. `frame` and `preview` hand back sRGB pixels in the one space the painter has, so working, display and output agree |
| HDR project and preview | 32 bpc plus an HDR working space such as Rec.2100 PQ, previewed with display colour management | [Enhanced HDR support][hdr-create] | gap | 8-bit `yuv420p` H.264 only (ADR-0077 §7) |
| HDR export with CICP metadata | Render to an HDR output space such as Rec 2100 PQ with CICP code points embedded, and PNG sequences with HDR10 mastering metadata | [Import and export HDR content][hdrio-export] | gap | No HDR output, and no colour tags of any kind on the encode (`encode.rs`, ADR-0077 §7) |
| HDR footage in an SDR project | Tone-map or compress HDR sources into the SDR range | [Utility][ut-hdrc] (HDR Compander, HDR Highlight Compression) | gap | No tone mapping; how an HDR source decodes to `rgba` is `ffmpeg`'s default and is unmeasured (ADR-0141's gate) |
| Colour values | Pick colours as numbers | [Color basics][basics] | covered | **Colour** in `CONTEXT.md`: `#RRGGBB` or `#RRGGBBAA` (ADR-0014) |

## Color Correction

| Effect | What it does | Adobe doc | Montagent status | Evidence |
|---|---|---|---|---|
| Lumetri Color | Premiere's grading tools as one effect: white balance, exposure and tone sliders, an input LUT and Lumetri Looks, RGB and hue/saturation curves, colour wheels, HSL secondaries and vignettes | [Color Correction][cc-lumetri], [Premiere][pr-lumetri] | partial | `saturation`, `contrast`, `brightness` (ADR-0049); vignette is composable (capability map, "A texture or glow laid over footage"); LUT refused (ADR-0040); curves refused (ADR-0049); wheels and white balance are gaps |
| Curves | "Arbitrarily map input values to output values using a curve defined by 256 points", for RGB or one channel | [Color Correction][cc-curves] | refused by ADR-0049 | "Curves, LUTs … are categorically excluded" |
| Levels | Input black/white, gamma, output black/white, for colour or alpha; GPU-accelerated | [Color Correction][cc-levels] | partial | The black and white points are a linear map (a gain and an offset). `contrast` (a gain about 0.5) and `brightness` (an offset) compose one on paper (`canvas.rs`). If values clamp to 0–1 between members, the order has to follow the gain's direction; that is not tested. Gamma is the gap: ADR-0049 names `gamma{value}` as admissible |
| Levels (Individual Controls) | Levels with separate R, G, B and alpha controls | [Color Correction][cc-levels-ic] | gap | A per-channel linear map is a diagonal colour matrix. ADR-0049 excludes "per-channel colour matrices", but a fixed set of bounded scalars might pass its clauses. Not decided |
| Hue/Saturation | Master hue rotation (±180°), saturation and lightness, the same per colour range, and Colorize | [Color Correction][cc-huesat] | partial | `saturation` covers saturation; colorize ≈ `saturation: 0` then `tint`; hue rotation has no member (ADR-0049 mentions a hue angle only as a possible redefinition) |
| Color Balance | "Changes the amount of red, green, and blue in the shadows, midtones, and highlights", with Preserve Luminosity | [Color Correction][cc-balance] | gap | Nine bounded scalars with an identity at zero would fit ADR-0049's clauses; no member |
| Color Balance (HLS) | Hue, lightness, saturation; "intended only to provide compatibility" with old projects | [Color Correction][cc-balance-hls] | partial | `saturation`, `brightness`; no hue rotation |
| Tint | Replaces each pixel with a value between Map Black To and Map White To, by an Amount To Tint; GPU-accelerated | [Color Correction][cc-tint] | partial | Montagent's `tint` pushes every pixel toward one colour (ADR-0049, `canvas.rs`), which is not AE's two-colour map. The two-colour map is a duotone, which ADR-0049 excludes |
| Tritone | "Mapping bright, dark, and midtone pixels to colors that you select" | [Color Correction][cc-tritone] | refused by ADR-0049 | Duotone-style colour maps excluded; colour-typed parameters beyond `tint.color` closed |
| CC Toner | "Add a color tint by adjusting the shadows, midtones, and highlights of a layer" | [Cycore plugins][cyc-cc] | refused by ADR-0049 | As Tritone |
| Colorama | Converts an attribute "to grayscale and then remaps" it through a cycling palette | [Color Correction][cc-colorama] | refused by ADR-0049 | A palette map is an arbitrary function, not a fixed scalar set |
| Change Color | "Adjusts the hue, lightness, and saturation of a range of colors" around one picked colour | [Color Correction][cc-change] | gap | Needs a colour-typed parameter; ADR-0049 says such a parameter "needs its own stated justification" |
| Change to Color | "Changes a color you select … to another color using hue, lightness, and saturation (HLS) values, leaving other colors unaffected" | [Color Correction][cc-change-to] | gap | Two colour-typed parameters; same ADR-0049 bar |
| Leave Color | "Desaturates all colors on a layer except colors similar to the color specified" | [Color Correction][cc-leave] | gap | Same ADR-0049 bar. Not composable: `chroma` removes the picked colour, and nothing keeps only it |
| Exposure | Exposure in f-stops, offset and gamma correction, computed "in a linear color space" unless Bypass Linear Light Conversion is set | [Color Correction][cc-exposure] | partial | A pure gain *s* in sRGB composes on paper: `contrast: s − 1` then `brightness: 0.5s − 0.5` for *s* < 1, or `brightness: 0.5 − 0.5/s` then `contrast` for *s* > 1, the order chosen so that no value between members leaves 0–1 (untested). That is AE's Exposure with the bypass on. Linear-light exposure goes with linear light, which ADR-0147 refuses. Offset is `brightness`. Gamma is the gap |
| Gamma/Pedestal/Gain | Per-channel response curve: Black Stretch, gamma, and pedestal and gain as the lowest and highest output; 8 bpc only | [Color Correction][cc-gpg], [effect list][list] | gap | Gamma has no member; ADR-0049 names `gamma{value}` as admissible. The per-channel form meets the per-channel-matrix exclusion |
| Brightness & Contrast | Brightness and contrast of the whole layer, identity at 0.0 | [Color Correction][cc-bc] | covered | `brightness`, `contrast` (ADR-0049) |
| Vibrance | Raises saturation more where it is low, and affects hues "from magenta to orange" less, to protect skin tones | [Color Correction][cc-vibrance] | gap | One bounded scalar; not composable from `saturation`. Fits ADR-0049's clauses |
| Black & White | Greyscale with a per-colour mix and an optional tint, after Photoshop's Black & White | [Color Correction][cc-bw] | partial | `saturation: 0` gives greyscale with fixed luma weights (ADR-0049; `LUMA` in `canvas.rs`), and `tint` adds the tint; no per-colour mix |
| Photo Filter | A warming, cooling or custom filter colour with a Density, and Preserve Luminosity | [Color Correction][cc-photo] | partial | `tint` with an `amount`; no preserve-luminosity |
| Selective Color | CMYK adjustments per colour range; provided "primarily to ensure fidelity with documents imported from Photoshop" | [Color Correction][cc-selective] | gap | No member. A Photoshop-fidelity effect with no forcing case |
| Shadow/Highlight | Brightens shadows and darkens highlights "based on the surrounding pixels", with automatic amounts and temporal smoothing | [Color Correction][cc-sh] | gap | No member; the automatic amounts read rendered pixels, which ADR-0145's literal-values test does not allow |
| Auto Color | Adjusts contrast and colour "after analyzing the shadows, midtones, and highlights", with Snap Neutral Midtones and temporal smoothing over adjacent frames | [Color Correction][cc-auto] | gap | The result depends on rendered pixels (ADR-0145, literal values). A skill could measure the footage and write literal `brightness`, `contrast` and `tint`; none does |
| Auto Contrast | Maps the lightest and darkest pixels to white and black without adjusting channels individually | [Color Correction][cc-auto] | gap | As Auto Color |
| Auto Levels | Maps the lightest and darkest values "in each color channel" to white and black | [Color Correction][cc-autolevels] | gap | As Auto Color |
| Equalize | Redistributes brightness or colour values for a more even distribution, like Photoshop's Equalize; 8 bpc only | [Color Correction][cc-equalize], [effect list][list] | gap | As Auto Color |
| Color Stabilizer | Samples one to three points on a pivot frame and holds their values constant across the layer | [Color Correction][cc-stabilizer] | gap | It reads pixels across time. Keyed `brightness` (ADR-0146) is the literal way to write the result; no skill measures it |
| Color Link | "Colorizes one layer with the average pixel values of another layer" | [Color Correction][cc-link] | refused by ADR-0150 | It reads another element's pixels, which ADR-0156 refuses under ADR-0150. Workaround: sample the colour once and write a literal `tint` |
| Channel Mixer | Each output channel as a percentage mix of the input channels plus a constant | [Color Correction][cc-mixer] | refused by ADR-0049 | "Per-channel colour matrices" excluded |
| PS Arbitrary Map | Applies a Photoshop arbitrary-map file; compatibility only ("For new work, use the Curves effect") | [Color Correction][cc-psmap], [effect list][list] | refused by ADR-0040 | An external file holds the function, as with LUTs (ADR-0040), and it is a curve (ADR-0049) |
| Broadcast Colors | Alters colours "to keep signal amplitudes within the range allowed for broadcast television"; 8 bpc only | [Color Correction][cc-broadcast], [effect list][list] | gap | No member. Output range is not decided anywhere (see the pipeline notes) |
| Video Limiter | "Clips video signals to legal ranges in the project working space", luma and chroma, in IRE; GPU-accelerated; the successor to Broadcast Colors | [Color Correction][cc-limiter] | gap | As Broadcast Colors |
| CC Color Neutralizer | "Adjust red, green, and blue colors of a layer separately to fix or change its overall color" | [Cycore plugins][cyc-cc] | gap | Separate R, G and B adjustments meet ADR-0049's per-channel question, left undecided at Levels (Individual Controls) |
| CC Color Offset | "Shift the red, green, and blue channels", by a phase per channel | [Cycore plugins][cyc-cc] | gap | No member shifts one channel; per-channel parameters meet the same per-channel question |
| CC Kernel | "Change how pixels interact with nearby pixels to create effects like blur or pixelation", through a 3×3 convolution kernel† | [Cycore plugins][cyc-cc] | gap | Skia's `matrix_convolution` (#703). ADR-0156 deferred sharpen and emboss, the named uses of a kernel; an open kernel is an arbitrary function under ADR-0049's reading |
| OCIO Color Space Transform | Converts a layer from one colour space to another with the OpenColorIO engine; works without OCIO colour management turned on | [Color Correction][cc-ocio], [OpenColorIO][ocio-effects] | gap | No colour spaces at all (see Working colour space and OCIO colour-managed mode) |
| OCIO File Transform | "Used to apply a LUT or CDL" file, usually between two Color Space Transforms | [Color Correction][cc-ocio], [OpenColorIO][ocio-effects] | refused by ADR-0040 | The LUT's function lives in an external file, which ADR-0040 calls "a plugin in all but name"; repeated in ADR-0049 |
| OCIO Display Transform | Applies an OCIO display and view transform, or its inverse, to a layer | [Color Correction][cc-ocio], [OpenColorIO][ocio-display] | gap | No display or view transforms; Montagent's one space is also its display space |
| OCIO Look Transform | Applies a Look named in the OCIO config, or its inverse, between an input and an output colour space | [Color Correction][cc-ocio], [OpenColorIO][ocio-look] | gap | No OCIO config. A Look is defined in the external config, which meets ADR-0040's external-file reasoning, but no ADR rules on it |
| OCIO CDL Transform | Applies a CDL from a `.ccc` file (one CCCID from the collection), or its inverse | [Color Correction][cc-ocio], [OpenColorIO][ocio-effects] | gap | The file-held form meets ADR-0040's external-file reasoning. The numbers inside a CDL (slope, offset and power per channel, and a saturation) meet ADR-0049's per-channel question. Not decided |

## Blur & Sharpen

| Effect | What it does | Adobe doc | Montagent status | Evidence |
|---|---|---|---|---|
| Gaussian Blur | "Blurs and softens the image and eliminates noise", with Repeat Edge Pixels; GPU-accelerated | [Blur & Sharpen][bs-gauss] | covered | `blur{radius}`, σ = radius / 2 (ADR-0040, `canvas.rs`). Repeat Edge Pixels is missing |
| Fast Box Blur | A box blur with an Iterations control (about 3 iterations resembles Fast Blur); replicates Fast Blur (Legacy) faster, on the GPU | [Blur & Sharpen][bs-fastbox] | partial | `blur` gives the Gaussian that many iterations converge to; there is no single-pass box look and no iteration count |
| Directional Blur | "Gives a layer the illusion of motion"; GPU-accelerated | [Blur & Sharpen][bs-directional] | covered | `directional_blur{angle, length}` (ADR-0156) |
| Radial Blur | Spin or zoom blur around a Center point, "simulating the result of a zooming or rotating camera" | [Blur & Sharpen][bs-radial] | gap | ADR-0156 deferred the distortions because they sample outside the pixel and their edge rule is undecided. Radial blur samples the same way |
| Camera Lens Blur | Lens-modelled blur with iris shape and highlight bloom, optionally driven by a blur-map layer | [Blur & Sharpen][bs-lens] | gap | No bokeh member. Its blur-map mode reads another element: refused under ADR-0150, as ADR-0156 applies it |
| Compound Blur | Blurs "based on the luminance values of a control layer" | [Blur & Sharpen][bs-compound] | refused by ADR-0150 | It reads another element's pixels (ADR-0156's Displacement Map reasoning) |
| Channel Blur | "Individually blurs the red, green, blue, or alpha channels" | [Blur & Sharpen][bs-channel] | gap | `blur` acts on all channels at once; no member blurs one channel |
| Bilateral Blur | Edge-preserving blur: "areas with high contrast … are blurred less" | [Blur & Sharpen][bs-bilateral] | gap | No member; probably a runtime shader under ADR-0156 §2 |
| Smart Blur | Blurs "while preserving lines and edges", which Bilateral Blur still softens slightly | [Blur & Sharpen][bs-smart] | gap | As Bilateral Blur |
| Sharpen | "Increases the contrast where color changes occur"; GPU-accelerated | [Blur & Sharpen][bs-sharpen] | gap | Deferred by ADR-0156 ("Sharpen, emboss, mosaic and RGB split … not a judgement against them"); Skia `matrix_convolution` (#703) |
| Unsharp Mask | "Increases the contrast between colors that define an edge", with Radius and Threshold | [Blur & Sharpen][bs-unsharp] | gap | Deferred with Sharpen (ADR-0156) |
| Camera-Shake Deblur | Rebuilds blurry frames "by using sharp ones on either side", with optical flow | [Blur & Sharpen][bs-deblur] | gap | It reads neighbouring source frames; no member reads time but `grain`'s seed (ADR-0156 §3) |
| CC Cross Blur | "Blurs the image in both horizontal and vertical directions" | [Cycore plugins][cyc-bs] | partial | Two `directional_blur` members at 0° and 90°, though they are box-weighted, not Gaussian |
| CC Radial Blur | Adobe says only that it "blurs along the direction of motion"; a spin or zoom blur about a centre† | [Cycore plugins][cyc-bs] | gap | As Radial Blur |
| CC Radial Fast Blur | A faster version of CC Radial Blur ("It renders faster than CC Radial Blur"), about a centre† | [Cycore plugins][cyc-bs] | gap | As Radial Blur |
| CC Vector Blur | Adobe says only that it "blurs along the direction of motion"; along a vector field taken from the layer's own or another layer's values† | [Cycore plugins][cyc-bs] | gap | No member; the other-layer form would be refused under ADR-0150 |
| Reduce Interlace Flicker | "A slight vertical blur to reduce flicker" in interlaced footage; Obsolete in AE | [Obsolete][obs-rif] | gap | No interlaced output exists (`yuv420p` progressive, ADR-0077 §7). No case for it |
| Gaussian Blur (Legacy) | The old Gaussian, with "slightly different render result" from Gaussian Blur; Obsolete in AE | [Obsolete][obs-gauss] | covered | `blur` |

## Channel

| Effect | What it does | Adobe doc | Montagent status | Evidence |
|---|---|---|---|---|
| Invert | Inverts the colour information, in RGB, one channel, HLS, YIQ or alpha; GPU-accelerated | [Channel][ch-invert] | gap | No colour inversion: `contrast` cannot go negative (its scale clamps at 0, `canvas.rs`), and ADR-0147 refused `difference` for now. Alpha inversion exists only as an inverted shape `mask` (ADR-0152) |
| Set Matte | Replaces the alpha "with a channel from another layer above it"; compatibility only | [Channel][ch-setmatte] | refused by ADR-0150 | "A matte taken from another element" (capability map, Not by design) |
| Set Channels | "Copies channels from control layers (source layers) to the red, green, blue, and alpha channels" | [Channel][ch-setchannels] | refused by ADR-0150 | It reads other elements' pixels |
| Shift Channels | "Replaces red, green, blue, and alpha channels in the image with values from other channels" | [Channel][ch-shift] | gap | A channel permutation is a colour matrix (ADR-0049's exclusion). Alpha from the element's own luma is a matte operation, which ADR-0049 does not govern (ADR-0088); not decided |
| Arithmetic | Simple operations (bitwise, add, subtract, multiply, difference, min, max, block, slice, screen) on each RGB channel against a value; 8 bpc | [Channel][ch-arith] | gap | No member; per-channel constants meet the per-channel-matrix exclusion (ADR-0049) |
| Blend | Mixes the layer with a second layer by Crossfade, Color Only, Tint Only, Darken Only or Lighten Only; animatable, unlike blending modes | [Channel][ch-blend] | partial | Crossfade is `opacity` over the lower element, which Adobe notes is wrong where the front layer has transparent areas; darken and lighten were "refused for now" by ADR-0147; colour-only and tint-only have no blend mode |
| Calculations | "Combines channels of one layer with the channels of a second layer" | [Channel][ch-calc] | refused by ADR-0150 | It reads another element's pixels; the whole-element form is `blend` (ADR-0147) |
| Compound Arithmetic | Combines the layer with a control layer; Adobe says "using blending modes is usually more effective" | [Channel][ch-comparith] | refused by ADR-0150 | Control layer; `blend` (ADR-0147) is the route Adobe itself points to |
| Channel Combiner | "Extracts, displays, and adjusts channel values", for example any channel as greyscale, from RGB, HLS, YUV and luminance; optionally from a second layer; 32 bpc and GPU | [Channel][ch-combiner] | partial | Luminance-only is `saturation: 0`; no other channel conversion; the second-layer form reads another element (ADR-0150) |
| Minimax | Assigns each channel "the minimum or maximum value for that channel found within a specified radius", to grow or shrink a matte | [Channel][ch-minimax] | gap | Skia `dilate` and `erode` (#703 lists `dilate` for a stroke); would pass ADR-0156 §2 with a declared reach |
| Remove Color Matting | Removes halos from premultiplied channels ("previously known as the Unmultiply effect") | [Channel][ch-rcm] | gap | ADR-0089 reads whether a source carries alpha; nothing reads whether it is premultiplied |
| CC Composite | Lets you "blend layers more flexibly"; composites the effected layer over its own original with a blend mode† | [Cycore plugins][cyc-ch] | reachable | Two copies of the element, the upper one with its effects and a `blend` (ADR-0147). ADR-0148 refuses a copy construct, so each copy is written out |
| Solid Composite | Composites "a new color solid behind the original source layer", with source opacity, solid opacity and a blending mode | [Channel][ch-solid] | reachable | A `rect` of that colour below the element, or the project `background` |

## Utility

| Effect | What it does | Adobe doc | Montagent status | Evidence |
|---|---|---|---|---|
| Apply Color LUT | Applies a `.3dl` or `.cube` lookup table (also `.look` and `.csp`) | [Utility][ut-lut] | refused by ADR-0040 | "LUTs — semantics live entirely in an external `.cube` file"; repeated in ADR-0049 |
| Color Profile Converter | "Converts a layer from one color space to another by specifying input and output profiles", with a rendering intent | [Utility][ut-cpc] | gap | No colour profiles at all (see Working colour space) |
| Cineon Converter | Log to linear, linear to log, or log to log for Cineon frames | [Utility][ut-cineon] | gap | No log input or interpretation; same root as Input colour interpretation |
| Grow Bounds | "Increases the layer size for the effect that directly follows it", so a shadow is not clipped; not needed for 32-bpc effects | [Utility][ut-grow] | covered | Each member declares a reach and the painter grows the element's bounds by it (ADR-0156's bound criterion, `format/compositing.md`; "a shadow's" reach) |
| HDR Compander | Compresses HDR highlights into the 8- or 16-bpc range and expands them back, around low-dynamic-range effects | [Utility][ut-hdrc] | gap | Meaningful only with 32-bpc float, which is a gap |
| HDR Highlight Compression | Compresses HDR values into 0.0–1.0 by an Amount | [Utility][ut-hdrhc] | gap | As HDR Compander |
| CC Overbrights | "Boosts the brightness of highlight areas, producing an overexposed or glowing look while keeping mid tones and shadows unchanged" | [Cycore plugins][cyc-ut] | gap | No member boosts highlights alone; at 8 bpc a boost clips at white, so only the overexposed look would survive |

## Not sourced

- **Bit depth where the effect list and the effect's page disagree.** Change Color: the list
  marks 32 bpc, the page says 8 and 16. PS Arbitrary Map: the list marks 8 only, the page says 8
  and 16. Channel Combiner: the list leaves 8 bpc blank and marks 16 and 32, the page says 32.
  Leave Color's page states no bit depth; the list marks 8 only. None of these changes a score.
- **The OCIO effects' bit depth and GPU support.** They are not in the effect list's table, and
  their sections do not say.
- **Every line marked †.** Adobe documents the CC effects only in one line each on the Cycore
  page. The † parts (CC Radial Blur's and CC Radial Fast Blur's centre point, CC Vector Blur's
  vector field, CC Composite compositing over its own original, CC Kernel's 3×3 kernel) come
  from the effects' known behaviour. CC Composite's "reachable" score rests on its †.
- **What `ffmpeg` actually does to an untagged, full-range or 10-bit source** on the way to
  `rgba`, and which matrix the encoder uses. These are read from the code, not measured.

## Gaps worth a decision ticket

Grouped so that each group could be one ticket. Each group names the ADR it must argue
against.

1. **Colour management and the delivery tags.** The cheapest group, and it fixes a silent
   risk. Today the deliverable is untagged `yuv420p` converted with `ffmpeg`'s default matrix,
   and footage decodes with `ffmpeg`'s defaults. Decide: write Rec.709 (or sRGB) tags and an
   explicit matrix and range on encode; say how a tagged, full-range or BT.2020 source is
   read; and record in an ADR that the working space is sRGB, which ADR-0146, 0147 and 0149
   already assume. This is a measuring prototype plus an ADR, with no new key. It is
   prerequisite to everything below.
2. **16- and 32-bit float, linear light and HDR.** One question: does Montagent ever leave
   8-bit sRGB? Everything AE does with 32 bpc (overbrights, linear blending, HDR Compander,
   HDR output with CICP tags, ACES, OCIO and the OCIO colour-space, display and look
   transforms) hangs on that answer. ADR-0147 already refuses linear-light blending on
   precedent grounds, and the 8-bit painter, `#RRGGBB` colours and byte-identical tests all
   assume 8 bits. A likely outcome is a recorded no for now, with HDR delivery as the named
   revisit condition.
3. **A small grade family under ADR-0049's own clauses.** These candidates have fixed arity,
   bounded scalars and an identity, and none composes from today's four: `gamma` (which
   ADR-0049 names itself), `vibrance`, a `hue` rotation, `exposure` as a gain (it composes in
   sRGB today, which clause (d) has to weigh), and `color_balance` (shadows, midtones and
   highlights). Together they would cover the scalar core of Levels, Hue/Saturation,
   Exposure, Gamma/Pedestal/Gain, Vibrance, Color Balance and most of Lumetri's basic panel.
   Leave Color, Change Color and colour wheels need ADR-0049's closed `tint.color` exception
   reopened, so they should be a separate question.
4. **LUTs, curves and the per-channel family: confirm the refusal or reopen it.** Apply Color
   LUT, OCIO File Transform, Lumetri's look, Curves, Channel Mixer, Tritone, CC Toner and
   Colorama are refused by ADR-0040 and ADR-0049. OCIO CDL Transform and OCIO Look Transform
   are file-held functions of the same kind, which no ADR names yet. AE and Premiere both ship
   LUTs, so the question for the map is whether "a LUT is a plugin in all but name" still
   holds. Pre-grading footage through `montagent-prerender` is the workaround today, but no
   skill names it.
5. **Sharpen and the edge-aware blurs, under ADR-0156's growth path.** Sharpen and Unsharp
   Mask (deferred by ADR-0156), Bilateral or Smart Blur, Channel Blur and Minimax (Skia
   `dilate` and `erode`). Each is one built-in filter or one small shader with a declared
   reach, so each can join with its own ADR. Sharpen has the strongest case: every editor
   ships it.
6. **Radial and zoom blur, with the distortions.** Radial Blur and CC Radial (Fast) Blur sample
   outside the pixel, so they wait on the edge rule ADR-0156 left undecided for the
   distortions. Route them to whichever ticket takes the distortions.
7. **A colour inversion and the missing blend modes.** Invert has no spelling: `contrast`
   clamps, and ADR-0147 refused `difference`, `darken` and `lighten` "for now", with no forcing
   case. Channel's Blend effect and the AE blend-mode census (another #896 ticket) bear on the
   same question.
8. **Measure-then-write skills for the automatic effects.** Auto Color, Auto Contrast, Auto
   Levels, Equalize, Color Stabilizer and Shadow/Highlight's automatic amounts read rendered
   pixels, so they can never be format keys under ADR-0145. A skill script could measure
   footage and write literal `brightness`, `contrast` and `tint` keyframes, the way ducking
   writes `volume`. This belongs with the map's "what reachable by a skill covers" question.

Not worth a ticket: Selective Color, Broadcast Colors, Video Limiter (until delivery range is
decided in group 1), Reduce Interlace Flicker, Camera-Shake Deblur, Arithmetic, and Remove
Color Matting (until a premultiplied source turns up).

[whats-new]: https://helpx.adobe.com/after-effects/desktop/what-s-new/whats-new.html
[list]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/effects-and-animation-presets/effect-list.html
[basics]: https://helpx.adobe.com/after-effects/desktop/adjust-colors/color-basics/color-basics.html
[basics-depth]: https://helpx.adobe.com/after-effects/desktop/adjust-colors/color-basics/color-basics.html#comparative_advantages_of_each_color_depth
[cm]: https://helpx.adobe.com/after-effects/desktop/adjust-colors/color-management/color-management.html
[cm-working]: https://helpx.adobe.com/after-effects/desktop/adjust-colors/color-management/color-management.html#choose_a_working_color_space_and_enable_color_management
[cm-linear]: https://helpx.adobe.com/after-effects/desktop/adjust-colors/color-management/color-management.html#linearize_working_space_and_enable_linear_blending
[cm-input]: https://helpx.adobe.com/after-effects/desktop/adjust-colors/color-management/color-management.html#interpret_a_footage_item_by_assigning_an_input_color_profile
[cm-output]: https://helpx.adobe.com/after-effects/desktop/adjust-colors/color-management/color-management.html#assign_an_output_color_profile
[ocio]: https://helpx.adobe.com/after-effects/desktop/adjust-colors/opencolorio-and-aces-color-management/opencolorio-aces-color-management.html
[ocio-effects]: https://helpx.adobe.com/after-effects/desktop/adjust-colors/opencolorio-and-aces-color-management/opencolorio-aces-color-management.html#layer-effects-opencolorio
[ocio-display]: https://helpx.adobe.com/after-effects/desktop/adjust-colors/opencolorio-and-aces-color-management/opencolorio-aces-color-management.html#ocio-display-transform
[ocio-look]: https://helpx.adobe.com/after-effects/desktop/adjust-colors/opencolorio-and-aces-color-management/opencolorio-aces-color-management.html#ocio-look-transform
[hdr]: https://helpx.adobe.com/after-effects/desktop/adjust-colors/enhanced-hdr-support/hdr-preview.html
[hdr-create]: https://helpx.adobe.com/after-effects/desktop/adjust-colors/enhanced-hdr-support/hdr-preview.html#create-an-hdr-project
[hdrio]: https://helpx.adobe.com/after-effects/desktop/adjust-colors/hdr-import-and-export/hdr-import-and-export.html
[hdrio-export]: https://helpx.adobe.com/after-effects/desktop/adjust-colors/hdr-import-and-export/hdr-import-and-export.html#export
[pr-lumetri]: https://helpx.adobe.com/premiere/desktop/correct-color/color-correction-fundamentals/basic-color-correction-options.html
[cc]: https://helpx.adobe.com/after-effects/desktop/adjust-colors/work-with-color-correction-effects/color-correction-effects.html
[cc-auto]: https://helpx.adobe.com/after-effects/desktop/adjust-colors/work-with-color-correction-effects/color-correction-effects.html#auto_color_and_auto_contrast_effects
[cc-autolevels]: https://helpx.adobe.com/after-effects/desktop/adjust-colors/work-with-color-correction-effects/color-correction-effects.html#auto_levels_effect
[cc-bw]: https://helpx.adobe.com/after-effects/desktop/adjust-colors/work-with-color-correction-effects/color-correction-effects.html#black_white_effect
[cc-bc]: https://helpx.adobe.com/after-effects/desktop/adjust-colors/work-with-color-correction-effects/color-correction-effects.html#brightness_contrast_effect
[cc-broadcast]: https://helpx.adobe.com/after-effects/desktop/adjust-colors/work-with-color-correction-effects/color-correction-effects.html#broadcast_colors_effect
[cc-change]: https://helpx.adobe.com/after-effects/desktop/adjust-colors/work-with-color-correction-effects/color-correction-effects.html#change_color_effect
[cc-change-to]: https://helpx.adobe.com/after-effects/desktop/adjust-colors/work-with-color-correction-effects/color-correction-effects.html#change_to_color_effect
[cc-mixer]: https://helpx.adobe.com/after-effects/desktop/adjust-colors/work-with-color-correction-effects/color-correction-effects.html#channel_mixer_effect
[cc-balance]: https://helpx.adobe.com/after-effects/desktop/adjust-colors/work-with-color-correction-effects/color-correction-effects.html#color_balance_effect
[cc-balance-hls]: https://helpx.adobe.com/after-effects/desktop/adjust-colors/work-with-color-correction-effects/color-correction-effects.html#color_balance_hls_effect
[cc-link]: https://helpx.adobe.com/after-effects/desktop/adjust-colors/work-with-color-correction-effects/color-correction-effects.html#color_link_effect
[cc-stabilizer]: https://helpx.adobe.com/after-effects/desktop/adjust-colors/work-with-color-correction-effects/color-correction-effects.html#color_stabilizer_effect
[cc-colorama]: https://helpx.adobe.com/after-effects/desktop/adjust-colors/work-with-color-correction-effects/color-correction-effects.html#colorama_effect
[cc-curves]: https://helpx.adobe.com/after-effects/desktop/adjust-colors/work-with-color-correction-effects/color-correction-effects.html#curves_effect
[cc-equalize]: https://helpx.adobe.com/after-effects/desktop/adjust-colors/work-with-color-correction-effects/color-correction-effects.html#equalize_effect
[cc-exposure]: https://helpx.adobe.com/after-effects/desktop/adjust-colors/work-with-color-correction-effects/color-correction-effects.html#exposure_effect
[cc-gpg]: https://helpx.adobe.com/after-effects/desktop/adjust-colors/work-with-color-correction-effects/color-correction-effects.html#gamma_pedestal_gain_effect
[cc-huesat]: https://helpx.adobe.com/after-effects/desktop/adjust-colors/work-with-color-correction-effects/color-correction-effects.html#hue_saturation_effect
[cc-leave]: https://helpx.adobe.com/after-effects/desktop/adjust-colors/work-with-color-correction-effects/color-correction-effects.html#leave_color_effect
[cc-levels]: https://helpx.adobe.com/after-effects/desktop/adjust-colors/work-with-color-correction-effects/color-correction-effects.html#levels_effect
[cc-levels-ic]: https://helpx.adobe.com/after-effects/desktop/adjust-colors/work-with-color-correction-effects/color-correction-effects.html#levels_individual_controls_effect
[cc-lumetri]: https://helpx.adobe.com/after-effects/desktop/adjust-colors/work-with-color-correction-effects/color-correction-effects.html#lumetri
[cc-ocio]: https://helpx.adobe.com/after-effects/desktop/adjust-colors/work-with-color-correction-effects/color-correction-effects.html
[cc-photo]: https://helpx.adobe.com/after-effects/desktop/adjust-colors/work-with-color-correction-effects/color-correction-effects.html#photo_filter_effect
[cc-psmap]: https://helpx.adobe.com/after-effects/desktop/adjust-colors/work-with-color-correction-effects/color-correction-effects.html#ps_arbitrary_map_effect
[cc-selective]: https://helpx.adobe.com/after-effects/desktop/adjust-colors/work-with-color-correction-effects/color-correction-effects.html#selective_color_effect
[cc-sh]: https://helpx.adobe.com/after-effects/desktop/adjust-colors/work-with-color-correction-effects/color-correction-effects.html#shadow_highlight_effect
[cc-tint]: https://helpx.adobe.com/after-effects/desktop/adjust-colors/work-with-color-correction-effects/color-correction-effects.html#tint_effect
[cc-tritone]: https://helpx.adobe.com/after-effects/desktop/adjust-colors/work-with-color-correction-effects/color-correction-effects.html#tritone_effect
[cc-vibrance]: https://helpx.adobe.com/after-effects/desktop/adjust-colors/work-with-color-correction-effects/color-correction-effects.html#vibrance_effect
[cc-limiter]: https://helpx.adobe.com/after-effects/desktop/adjust-colors/work-with-color-correction-effects/color-correction-effects.html#VideoLimitereffect
[bs]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/blur-sharpen-effects.html
[bs-bilateral]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/blur-sharpen-effects.html#bilateral_blur_effect
[bs-lens]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/blur-sharpen-effects.html#camera_lens_blur_effect
[bs-deblur]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/blur-sharpen-effects.html#camera_shake_deblur
[bs-channel]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/blur-sharpen-effects.html#channel_blur_effect
[bs-compound]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/blur-sharpen-effects.html#compound_blur_effect
[bs-directional]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/blur-sharpen-effects.html#directional_blur_effect
[bs-fastbox]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/blur-sharpen-effects.html#FastBoxBlureffect
[bs-gauss]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/blur-sharpen-effects.html#gaussian_blur_effect
[bs-radial]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/blur-sharpen-effects.html#radial_blur_effect
[bs-sharpen]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/blur-sharpen-effects.html#sharpen_effect
[bs-smart]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/blur-sharpen-effects.html#smart_blur_effect
[bs-unsharp]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/blur-sharpen-effects.html#unsharp_mask_effect
[ch]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/channel-effects.html
[ch-arith]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/channel-effects.html#arithmetic_effect
[ch-blend]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/channel-effects.html#blend_effect
[ch-calc]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/channel-effects.html#calculations_effect
[ch-combiner]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/channel-effects.html#channel_combiner_effect
[ch-comparith]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/channel-effects.html#compound_arithmetic_effect
[ch-invert]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/channel-effects.html#invert_effect
[ch-minimax]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/channel-effects.html#minimax_effect
[ch-rcm]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/channel-effects.html#remove_color_matting_effect
[ch-setchannels]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/channel-effects.html#set_channels_effect
[ch-setmatte]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/channel-effects.html#set_matte_effect
[ch-shift]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/channel-effects.html#shift_channels_effect
[ch-solid]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/channel-effects.html#solid_composite_effect
[ut]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/utility-effects.html
[ut-lut]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/utility-effects.html#apply_color_lut_effect
[ut-cineon]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/utility-effects.html#cineon_converter_effect
[ut-cpc]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/utility-effects.html#color_profile_converter_effect
[ut-grow]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/utility-effects.html#grow_bounds_effect
[ut-hdrc]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/utility-effects.html#hdr_compander_effect
[ut-hdrhc]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/utility-effects.html#hdr_highlight_compression_effect
[obs]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/obsolete-effects.html
[obs-gauss]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/obsolete-effects.html#GaussianBlurLegacyeffect
[obs-rif]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/obsolete-effects.html#reduce-interlace-flicker
[cyc]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/cycore-plugins.html
[cyc-bs]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/cycore-plugins.html#blur-sharpen
[cyc-ch]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/cycore-plugins.html#channel
[cyc-cc]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/cycore-plugins.html#color-correction
[cyc-ut]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/cycore-plugins.html#utility
