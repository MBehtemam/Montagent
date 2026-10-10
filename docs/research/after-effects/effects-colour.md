# After Effects census: colour, colour management, and the colour, blur, channel and utility effects

Research for [#904](https://github.com/MBehtemam/Montagent/issues/904), part of the map
[#896](https://github.com/MBehtemam/Montagent/issues/896).

**Date of research:** 2026-10-10. Nothing was built or run. This note scores features. It
decides nothing.

**After Effects version read:** 26.5, the September 2026 release. Adobe's [What's new][whats-new]
lists it first, and says that release moves colour management to ACES 2.0 through an updated
OpenColorIO library.

**Sources.**

- Adobe's After Effects user guide: the [effect list][list] (each effect's category, its
  8/16/32-bpc support and whether it runs on the GPU), and the category pages
  [Color Correction][cc], [Blur & Sharpen][bs], [Channel][ch], [Utility][ut] and
  [Obsolete][obs]. For colour management: [Color basics][basics],
  [Managing color][cm], [OpenColorIO and ACES][ocio] and [Enhanced HDR support][hdr].
- Montagent at `main` `51aa4b5`: `CONTEXT.md`, `docs/adr/`, `crates/montagent-core/docs/format.md`
  and its page `format/compositing.md`, `skills/*/SKILL.md`, `schema/montagent.schema.json`,
  and, for the pixel pipeline, `crates/montagent-render/src/{canvas.rs,encode.rs,decode.rs}`.
- The earlier Premiere census, [#703](https://github.com/MBehtemam/Montagent/issues/703)
  (`docs/research/skia-reach/named-effects-candidates.md` on branch
  `research/703-named-effects-candidates`), and ADR-0156's growth path.

**How Adobe was read.** The egress proxy refused `helpx.adobe.com` from this session (403 on
CONNECT), and no mirror was reachable. #703 hit the same block. So the Adobe pages were read
through search-engine extracts restricted to `helpx.adobe.com`. Every effect name, bit depth
and quoted sentence below comes from those extracts. Where an extract gave a name but no
description, the one-line description is the effect's well-known behaviour, and it is marked
†. Those lines should be checked against the page before a decision ticket leans on them.

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

**Counts, over 87 rows:** covered 8, partial 12, reachable 2, owned by an open map 0,
refused 15 (ADR-0150: 6, ADR-0049: 5, ADR-0147: 2, ADR-0040: 2), gap 50.

## Bit depth, linear light and colour management

| Feature | What it does | Adobe doc | Montagent status | Evidence |
|---|---|---|---|---|
| 8-bpc project | 8 bits per channel, values 0–255 | [Color basics][basics] | covered | RGBA8888 painter (`canvas.rs`); `yuv420p` output (ADR-0077 §7) |
| 16-bpc project | 16 bits per channel, finer gradations, less banding | [Color basics][basics] | gap | No ADR or format key; every surface is RGBA8888 |
| 32-bpc float project | Floating-point channels; values "under 0.0 and over 1.0", so HDR and overbrights survive a grade | [Color basics][basics] | gap | As above. No key stores an out-of-range value: colours are `#RRGGBB(AA)` (ADR-0014) and effect results clamp to 0–1 (`format/compositing.md`, Grain and Named effects) |
| Linearize working space | Composite and run effects in linear light (gamma 1.0), which prevents halo and fringe artefacts; Adobe advises it at 16 or 32 bpc, not at 8 | [Managing color][cm] | refused by ADR-0147 | ADR-0147 refuses "Linear-light arithmetic", because "it differs from the reference editors' default"; ADR-0146 keys colour in sRGB |
| Blend colors using 1.0 gamma | Linear-light blending between layers only | [Managing color][cm] | refused by ADR-0147 | Same refusal; `blend` "runs on the stored sRGB values" (`format/compositing.md`) |
| Working colour space (Adobe colour-managed) | Choose the space every comp is computed in; None turns colour management off | [Managing color][cm] | gap | sRGB is assumed in ADR-0146, 0147 and 0149, but no ADR chooses a working space or weighs another |
| OCIO colour-managed mode, ACES | OpenColorIO engine, ACES workflows (ACES 2.0 as of 26.5), separate working and display spaces | [OpenColorIO and ACES][ocio], [What's new][whats-new] | gap | No ADR, key or skill mentions OCIO or ACES (grep of `docs/adr`, `CONTEXT.md`, `skills`) |
| Input colour interpretation | Read a footage item's embedded profile, or assign one in Interpret Footage or a rules file | [Managing color][cm] | gap | `decode.rs` asks `ffmpeg` for `rgba` with no colour arguments; ADR-0141's gate excludes full-range sources; `probe` reports no colour fields |
| Interpret As Linear Light | Treat a footage item's profile as gamma 1.0 | [Managing color][cm] | gap | As above |
| Output colour profile | Convert the render to an output space and tag it | [Managing color][cm] | gap | `encode.rs` writes untagged `yuv420p` with `ffmpeg`'s default matrix; ADR-0077 §7 fixes `yuv420p` and says nothing on colour |
| Display colour management | Convert the working space for the viewer, separately from the render | [OpenColorIO and ACES][ocio], [HDR][hdr] | covered (trivially) | No viewer. `frame` and `preview` hand back sRGB pixels in the one space the painter has, so working, display and output agree |
| HDR project and preview | 32 bpc plus an HDR working space such as Rec.2100 PQ, previewed with display colour management | [Enhanced HDR support][hdr] | gap | 8-bit `yuv420p` H.264 only (ADR-0077 §7) |
| HDR footage in an SDR project | Tone-map or compress HDR sources into the SDR range | [Utility][ut] (HDR Compander, HDR Highlight Compression) | gap | No tone mapping; how an HDR source decodes to `rgba` is `ffmpeg`'s default and is unmeasured (ADR-0141's gate) |
| Colour values | Pick colours as numbers | [Color basics][basics] | covered | **Colour** in `CONTEXT.md`: `#RRGGBB` or `#RRGGBBAA` (ADR-0014) |

## Color Correction

| Effect | What it does | Adobe doc | Montagent status | Evidence |
|---|---|---|---|---|
| Lumetri Color | Premiere's grading panel as one effect: white balance, exposure and tone sliders, creative look LUT, curves, colour wheels, HSL secondaries, vignette† | [Color Correction][cc] | partial | `saturation`, `contrast`, `brightness` (ADR-0049); vignette is composable (capability map, "A texture or glow laid over footage"); LUT refused (ADR-0040); curves refused (ADR-0049); wheels and white balance are gaps |
| Curves | Remap tones per channel with an arbitrary curve | [Color Correction][cc] | refused by ADR-0049 | "Curves, LUTs … are categorically excluded" |
| Levels | Input black/white, gamma, output black/white | [Color Correction][cc] | partial | The black and white points are a linear map (a gain and an offset). `contrast` (a gain about 0.5) and `brightness` (an offset) compose one on paper (`canvas.rs`). If values clamp to 0–1 between members, the order has to follow the gain's direction; that is not tested. Gamma is the gap: ADR-0049 names `gamma{value}` as admissible |
| Levels (Individual Controls) | Levels with separate R, G, B and alpha controls | [Color Correction][cc] | gap | A per-channel linear map is a diagonal colour matrix. ADR-0049 excludes "per-channel colour matrices", but a fixed set of bounded scalars might pass its clauses. Not decided |
| Hue/Saturation | Master hue rotation, saturation, lightness; colorize | [Color Correction][cc] | partial | `saturation` covers saturation; colorize ≈ `saturation: 0` then `tint`; hue rotation has no member (ADR-0049 mentions a hue angle only as a possible redefinition) |
| Color Balance | Red, green and blue shifts for shadows, midtones and highlights† | [Color Correction][cc] | gap | Nine bounded scalars with an identity at zero would fit ADR-0049's clauses; no member |
| Color Balance (HLS) | Hue, lightness, saturation; "intended only to provide compatibility" with old projects | [Color Correction][cc] | partial | `saturation`, `brightness`; no hue rotation |
| Tint | Maps black to one colour and white to another, by an amount† | [Color Correction][cc] | partial | Montagent's `tint` pushes every pixel toward one colour (ADR-0049, `canvas.rs`), which is not AE's two-colour map. The two-colour map is a duotone, which ADR-0049 excludes |
| Tritone | "Mapping bright, dark, and midtone pixels to colors that you select" | [Color Correction][cc] | refused by ADR-0049 | Duotone-style colour maps excluded; colour-typed parameters beyond `tint.color` closed |
| CC Toner | Duotone, tritone or pentatone tones by luminance† | [Color Correction][cc] | refused by ADR-0049 | As Tritone |
| Colorama | Converts an attribute "to grayscale and then remaps" it through a cycling palette | [Color Correction][cc] | refused by ADR-0049 | A palette map is an arbitrary function, not a fixed scalar set |
| Change Color | Shifts the hue, lightness and saturation of one picked colour range† | [Color Correction][cc] | gap | Needs a colour-typed parameter; ADR-0049 says such a parameter "needs its own stated justification" |
| Change to Color | "Changes a color you select … to another color using hue, lightness, and saturation (HLS) values, leaving other colors unaffected" | [Color Correction][cc] | gap | Two colour-typed parameters; same ADR-0049 bar |
| Leave Color | "Desaturates all colors on a layer except colors similar to the color specified" | [Color Correction][cc] | gap | Same ADR-0049 bar. Not composable: `chroma` removes the picked colour, and nothing keeps only it |
| Exposure | Exposure in stops, offset and gamma correction, computed in linear light† | [Color Correction][cc] | partial | A pure gain *s* in sRGB composes on paper: `contrast: s − 1` then `brightness: 0.5s − 0.5` for *s* < 1, or `brightness: 0.5 − 0.5/s` then `contrast` for *s* > 1, the order chosen so that no value between members leaves 0–1 (untested). Linear-light exposure goes with linear light, which ADR-0147 refuses. Offset is `brightness`. Gamma is the gap |
| Gamma/Pedestal/Gain | Per-channel gamma, black level and gain; 8 bpc only | [Color Correction][cc], [effect list][list] | gap | Gamma has no member; ADR-0049 names `gamma{value}` as admissible. The per-channel form meets the per-channel-matrix exclusion |
| Brightness & Contrast | Brightness and contrast sliders | [Color Correction][cc] | covered | `brightness`, `contrast` (ADR-0049) |
| Vibrance | Raises saturation more where it is low, and protects skin tones† | [Color Correction][cc] | gap | One bounded scalar; not composable from `saturation`. Fits ADR-0049's clauses |
| Black & White | Greyscale with a per-hue mix (reds, yellows and so on) and an optional tint† | [Color Correction][cc] | partial | `saturation: 0` gives greyscale with fixed luma weights (ADR-0049; `LUMA` in `canvas.rs`), and `tint` adds the tint; no per-hue mix |
| Photo Filter | A warming or cooling filter colour with a density, optionally preserving luminosity† | [Color Correction][cc] | partial | `tint` with an `amount`; no preserve-luminosity |
| Selective Color | CMYK adjustments per colour range; provided "primarily to ensure fidelity with documents imported from Photoshop" | [Color Correction][cc] | gap | No member. A Photoshop-fidelity effect with no forcing case |
| Shadow/Highlight | Brightens shadows and recovers highlights by local tone mapping, automatic by default† | [Color Correction][cc] | gap | No member; the automatic amounts read rendered pixels, which ADR-0145's literal-values test does not allow |
| Auto Color | Neutralises colour casts from the frame's own histogram† | [Color Correction][cc] | gap | The result depends on rendered pixels (ADR-0145, literal values). A skill could measure the footage and write literal `brightness`, `contrast` and `tint`; none does |
| Auto Contrast | Stretches contrast from the frame's histogram† | [Color Correction][cc] | gap | As Auto Color |
| Auto Levels | Sets black and white points per channel from the histogram† | [Color Correction][cc] | gap | As Auto Color |
| Equalize | Evens out brightness from the histogram; 8 bpc only | [Color Correction][cc], [effect list][list] | gap | As Auto Color |
| Color Stabilizer | Holds brightness or colour steady across frames by sampling points† | [Color Correction][cc] | gap | It reads pixels across time. Keyed `brightness` (ADR-0146) is the literal way to write the result; no skill measures it |
| Color Link | "Colorizes one layer with the average pixel values of another layer" | [Color Correction][cc] | refused by ADR-0150 | It reads another element's pixels, which ADR-0156 refuses under ADR-0150. Workaround: sample the colour once and write a literal `tint` |
| Channel Mixer | Each output channel as a weighted mix of the input channels | [Color Correction][cc] | refused by ADR-0049 | "Per-channel colour matrices" excluded |
| PS Arbitrary Map | Applies a Photoshop arbitrary-map file; 8 bpc only | [Color Correction][cc], [effect list][list] | refused by ADR-0040 | An external file holds the function, as with LUTs (ADR-0040), and it is a curve (ADR-0049) |
| Broadcast Colors | Pulls colours into the broadcast-safe range; 8 bpc only | [Color Correction][cc], [effect list][list] | gap | No member. Output range is not decided anywhere (see the pipeline notes) |
| Video Limiter | Clamps luma and chroma to a legal range for broadcast† | [Color Correction][cc] | gap | As Broadcast Colors |
| CC Color Neutralizer | Neutralises casts with shadow, midtone and highlight colour pickers† | [Color Correction][cc] | gap | Three colour-typed parameters (ADR-0049's closed exception); a colour-wheel grade in another form |
| CC Color Offset | Rotates the hue of each RGB channel separately† | [Color Correction][cc] | gap | No hue rotation, per channel or otherwise |
| CC Kernel | A 3×3 convolution kernel with nine typed weights† | [Color Correction][cc] | gap | Skia's `matrix_convolution` (#703). ADR-0156 deferred sharpen and emboss, the named uses of a kernel; an open kernel is an arbitrary function under ADR-0049's reading |

## Blur & Sharpen

| Effect | What it does | Adobe doc | Montagent status | Evidence |
|---|---|---|---|---|
| Gaussian Blur | Gaussian blur, with horizontal, vertical or both, and Repeat Edge Pixels; GPU-accelerated | [Blur & Sharpen][bs] | covered | `blur{radius}`, σ = radius / 2 (ADR-0040, `canvas.rs`). One-axis blur and Repeat Edge Pixels are missing |
| Fast Box Blur | Box blur repeated for a number of iterations; GPU-accelerated† | [Blur & Sharpen][bs] | partial | `blur` gives the Gaussian that many iterations converge to; there is no single-pass box look and no iteration count |
| Directional Blur | "Gives a layer the illusion of motion" | [Blur & Sharpen][bs] | covered | `directional_blur{angle, length}` (ADR-0156) |
| Radial Blur | Spin or zoom blur around a centre point† | [Blur & Sharpen][bs] | gap | ADR-0156 deferred the distortions because they sample outside the pixel and their edge rule is undecided. Radial blur samples the same way |
| Camera Lens Blur | Lens-modelled blur with iris shape and highlight bloom, optionally driven by a blur-map layer | [Blur & Sharpen][bs] | gap | No bokeh member. Its blur-map mode reads another element: refused under ADR-0150, as ADR-0156 applies it |
| Compound Blur | Blurs by the brightness of a control layer | [Blur & Sharpen][bs] | refused by ADR-0150 | It reads another element's pixels (ADR-0156's Displacement Map reasoning) |
| Channel Blur | Blurs R, G, B or alpha by separate amounts | [Blur & Sharpen][bs] | gap | `blur` acts on all channels at once; no member blurs one channel |
| Bilateral Blur | Edge-preserving blur: "areas with high contrast are blurred less" | [Blur & Sharpen][bs] | gap | No member; probably a runtime shader under ADR-0156 §2 |
| Smart Blur | Edge-preserving blur that keeps edges sharper than Bilateral does | [Blur & Sharpen][bs] | gap | As Bilateral Blur |
| Sharpen | Raises contrast where colour changes | [Blur & Sharpen][bs] | gap | Deferred by ADR-0156 ("Sharpen, emboss, mosaic and RGB split … not a judgement against them"); Skia `matrix_convolution` (#703) |
| Unsharp Mask | Sharpens by edge contrast, with Amount, Radius and Threshold | [Blur & Sharpen][bs] | gap | Deferred with Sharpen (ADR-0156) |
| Camera-Shake Deblur | Rebuilds shaken frames "using sharp ones on either side" | [Blur & Sharpen][bs] | gap | It reads neighbouring source frames; no member reads time but `grain`'s seed (ADR-0156 §3) |
| CC Cross Blur | Separate horizontal and vertical blur amounts† | [Blur & Sharpen][bs] | partial | Two `directional_blur` members at 0° and 90°, though they are box-weighted, not Gaussian |
| CC Radial Blur | Spin or zoom blur with a centre† | [Blur & Sharpen][bs] | gap | As Radial Blur |
| CC Radial Fast Blur | A faster zoom blur with a centre† | [Blur & Sharpen][bs] | gap | As Radial Blur |
| CC Vector Blur | Blurs along a vector field taken from the layer's own or another layer's values† | [Blur & Sharpen][bs] | gap | No member; the other-layer form is refused under ADR-0150 |
| Reduce Interlace Flicker | A slight vertical blur for interlaced output; Obsolete in AE | [Obsolete][obs] | gap | No interlaced output exists (`yuv420p` progressive, ADR-0077 §7). No case for it |
| Gaussian Blur (Legacy) | The old Gaussian; Obsolete in AE | [Obsolete][obs] | covered | `blur` |

## Channel

| Effect | What it does | Adobe doc | Montagent status | Evidence |
|---|---|---|---|---|
| Invert | Subtracts channel values from 1.0, in RGB, one channel, HLS, YIQ or alpha | [Channel][ch] | gap | No colour inversion: `contrast` cannot go negative (its scale clamps at 0, `canvas.rs`), and ADR-0147 refused `difference` for now. Alpha inversion exists only as an inverted shape `mask` (ADR-0152) |
| Set Matte | Replaces the alpha "with a channel from another layer"; compatibility only | [Channel][ch] | refused by ADR-0150 | "A matte taken from another element" (capability map, Not by design) |
| Set Channels | Copies channels from control layers into R, G, B and A | [Channel][ch] | refused by ADR-0150 | It reads other elements' pixels |
| Shift Channels | Replaces each channel with another channel or a constant; for example, alpha from luminance | [Channel][ch] | gap | A channel permutation is a colour matrix (ADR-0049's exclusion). Alpha from the element's own luma is a matte operation, which ADR-0049 does not govern (ADR-0088); not decided |
| Arithmetic | Simple maths (add, subtract, multiply, min, max, bitwise and so on) on each RGB channel against a constant; 8 bpc | [Channel][ch] | gap | No member; per-channel constants meet the per-channel-matrix exclusion (ADR-0049) |
| Blend | Mixes the layer with a second layer by crossfade, colour only, tint only, darken only or lighten only | [Channel][ch] | partial | Crossfade is `opacity` over the lower element; darken and lighten were "refused for now" by ADR-0147; colour-only and tint-only have no blend mode |
| Calculations | "Combines channels of one layer with the channels of a second layer" | [Channel][ch] | refused by ADR-0150 | It reads another element's pixels; the whole-element form is `blend` (ADR-0147) |
| Compound Arithmetic | Combines the layer with a control layer; Adobe says "using blending modes is usually more effective" | [Channel][ch] | refused by ADR-0150 | Control layer; `blend` (ADR-0147) is the route Adobe itself points to |
| Channel Combiner | Extracts, displays and remaps values between RGB, HLS, YUV and luminance; 32 bpc and GPU | [Channel][ch] | partial | Luminance-only is `saturation: 0`; no other channel conversion |
| Minimax | Assigns each channel "the minimum or maximum value … within a specified radius", to grow or shrink a matte | [Channel][ch] | gap | Skia `dilate` and `erode` (#703 lists `dilate` for a stroke); would pass ADR-0156 §2 with a declared reach |
| Remove Color Matting | Removes halos from premultiplied channels ("previously known as the Unmultiply effect") | [Channel][ch] | gap | ADR-0089 reads whether a source carries alpha; nothing reads whether it is premultiplied |
| CC Composite | Composites the effected layer over its own original with a blend mode† | [Channel][ch] | reachable | Two copies of the element, the upper one with its effects and a `blend` (ADR-0147). ADR-0148 refuses a copy construct, so each copy is written out |
| Solid Composite | Composites the layer over a solid colour with an opacity and mode† | [Channel][ch] | reachable | A `rect` of that colour below the element, or the project `background` |

## Utility

| Effect | What it does | Adobe doc | Montagent status | Evidence |
|---|---|---|---|---|
| Apply Color LUT | Applies a `.3dl` or `.cube` lookup table | [Utility][ut] | refused by ADR-0040 | "LUTs — semantics live entirely in an external `.cube` file"; repeated in ADR-0049 |
| Color Profile Converter | Converts a layer between colour profiles† | [Utility][ut] | gap | No colour profiles at all (see Working colour space) |
| Cineon Converter | Log to linear, linear to log, or log to log for Cineon frames | [Utility][ut] | gap | No log input or interpretation; same root as Input colour interpretation |
| Grow Bounds | "Increases the layer size" for the effect after it, so a shadow is not clipped | [Utility][ut] | covered | Each member declares a reach and the painter grows the element's bounds by it (ADR-0156's bound criterion, `format/compositing.md`; "a shadow's" reach) |
| HDR Compander | Compresses HDR highlights into the 8- or 16-bpc range and expands them back, around low-dynamic-range effects | [Utility][ut] | gap | Meaningful only with 32-bpc float, which is a gap |
| HDR Highlight Compression | Compresses HDR values into 0.0–1.0 | [Utility][ut] | gap | As HDR Compander |
| CC Overbrights | Boosts highlights and shows overbright values† | [Utility][ut] | gap | Overbright values do not exist at 8 bpc |

## Not sourced

- **The exact count of the Color Correction category.** The effect list's table was reported
  as "about 39" entries, and 35 are named here. The missing ones may be old duplicates or
  entries that did not appear in the extracts. A reader with browser access should check the
  table.
- **OCIO transform effects.** No extract confirmed effects such as OCIO Color Space Transform
  in the Utility list. They are left out rather than guessed.
- **Change Color's bit depth.** The effect list marks it 32-bpc, but its own page says 8 and
  16 bpc. That makes no difference to the score.
- **Every line marked †** is described from the effect's known behaviour, not from an Adobe
  extract.
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
   HDR output, ACES and OCIO) hangs on that answer. ADR-0147 already refuses linear-light
   blending on precedent grounds, and the 8-bit painter, `#RRGGBB` colours and
   byte-identical tests all assume 8 bits. A likely outcome is a recorded no for now, with
   HDR delivery as the named revisit condition.
3. **A small grade family under ADR-0049's own clauses.** These candidates have fixed arity,
   bounded scalars and an identity, and none composes from today's four: `gamma` (which
   ADR-0049 names itself), `vibrance`, a `hue` rotation, `exposure` as a gain (it composes in
   sRGB today, which clause (d) has to weigh), and `color_balance` (shadows, midtones and
   highlights). Together they would cover the scalar core of Levels, Hue/Saturation,
   Exposure, Gamma/Pedestal/Gain, Vibrance, Color Balance and most of Lumetri's basic panel.
   Leave Color, Change Color and colour wheels need ADR-0049's closed `tint.color` exception
   reopened, so they should be a separate question.
4. **LUTs, curves and the per-channel family: confirm the refusal or reopen it.** Apply Color
   LUT, Lumetri's look, Curves, Channel Mixer, Tritone, CC Toner and Colorama are refused by
   ADR-0040 and ADR-0049. AE and Premiere both ship LUTs, so the question for the map is
   whether "a LUT is a plugin in all but name" still holds. Pre-grading footage through
   `montagent-prerender` is the workaround today, but no skill names it.
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

[whats-new]: https://helpx.adobe.com/after-effects/using/whats-new.html
[list]: https://helpx.adobe.com/after-effects/using/effect-list.html
[cc]: https://helpx.adobe.com/after-effects/using/color-correction-effects.html
[bs]: https://helpx.adobe.com/after-effects/using/blur-sharpen-effects.html
[ch]: https://helpx.adobe.com/after-effects/using/channel-effects.html
[ut]: https://helpx.adobe.com/after-effects/using/utility-effects.html
[obs]: https://helpx.adobe.com/after-effects/using/obsolete-effects.html
[basics]: https://helpx.adobe.com/after-effects/using/color-basics.html
[cm]: https://helpx.adobe.com/after-effects/using/color-management.html
[ocio]: https://helpx.adobe.com/after-effects/desktop/adjust-colors/opencolorio-and-aces-color-management/opencolorio-aces-color-management.html
[hdr]: https://helpx.adobe.com/after-effects/using/hdr-preview.html
