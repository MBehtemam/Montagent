# Premiere Pro gap: colour

Research ticket [#886](https://github.com/MBehtemam/Montagent/issues/886), on the map
[#882](https://github.com/MBehtemam/Montagent/issues/882). Area slug **colour**.

- **Date:** 2026-10-10
- **Montagent side:** tag `v0.2.0` (`51aa4b5`, equal to `origin/main` when read). Sources:
  `CONTEXT.md` (**Effect**, **Colour filter**, **Matte operation**, **Chroma**, **Colour**,
  **Paint**, **Gradient**, **Blend mode**), `schema/montagent.schema.json` (`/$defs/Effect`,
  `MaskShape`, `Blend`, `Paint`, `Gradient`, the top-level keys), ADR-0017, 0040, 0049, 0077,
  0088, 0143, 0146, 0147, 0149, 0150, 0152, 0156, the MCP tool surface in
  `crates/montagent/src/mcp.rs` (`validate`, `create_project`, `query`, `frame`, `render`,
  `verify`, `preview`, `measure`, `shift`, `compare`), the render crate
  (`crates/montagent-render/src/{decode,encode,canvas}.rs`) for what the pipeline actually does
  with colour, and `skills/`.
- **Premiere side:** Adobe's own `helpx.adobe.com` Premiere user guide only. **Caveat on how it
  was read:** this session's egress proxy refuses `helpx.adobe.com` (HTTP 403 on CONNECT), so no
  page body could be fetched directly. Every URL below is a helpx page that Adobe's search index
  returned, and the mechanism text comes from the search engine's extracts of those pages. A
  mechanism marked **†** is named from knowledge of Premiere and was **not** confirmed against an
  extract. A row whose URL reads *not retrieved* had no helpx page surface before the shared
  web-search budget ran out. Both kinds of row need a follow-up read of the page itself.
- **Premiere 2026 context:** Adobe's docs describe a new **Color mode** (beta) with a **Color
  Management panel** and a **Video Scopes** panel. The Lumetri *panel* is to be retired in
  Premiere 27.0, and the Lumetri Color *effect* stays
  ([Color mode basics](https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/color-mode-basics.html)).
  Both generations are listed.

### Counts

| Status | Rows |
| --- | --- |
| have | 9 |
| have (skill) | 1 |
| partial | 22 |
| missing | 60 |
| refused | 17 |
| **total** | **109** |

### How Montagent handles colour at v0.2.0, in one paragraph

Colour exists only as **per-element `effects`**: four colour-filter scalars (`tint`, `saturation`,
`brightness`, `contrast`, ADR-0049), two matte operations (`mask`, `chroma`, ADR-0088/0152), and
`posterize`/`glow`/`grain` (ADR-0156). All are animatable (ADR-0146). There is no adjustment
layer, so a grade cannot be applied to a track or to the whole frame. **There is no colour
management at all.** The decoder asks `ffmpeg` for `-pix_fmt rgba` (8-bit) and accepts its
default YUV→RGB conversion (`decode.rs`). The painter does all arithmetic on stored sRGB-encoded
values (ADR-0146 §2, ADR-0147 §2). The encoder writes 8-bit `yuv420p` H.264 with no
`-colorspace`/`-color_primaries`/`-color_trc`/`-color_range` tags (`encode.rs`,
`Settings::args`), and ADR-0143 §5 exposes no encoder option. No tool reads luma or chroma back
out of a frame. `measure` reads keyed-alpha coverage (ADR-0088) and `verify` reads size, timing
and audio, but neither reads colour.

## Inventory

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
| --- | --- | --- | --- | --- | --- |
| **Lumetri Color: Basic Correction** | | | | | |
| Input LUT (Basic Correction) | Choose a preset LUT or browse to a custom one as the starting point of the grade. | https://helpx.adobe.com/premiere/desktop/correct-color/color-correction-fundamentals/basic-color-correction-options.html | refused | ADR-0040 *Out of scope*: "LUTs — semantics live entirely in an external `.cube` file, which is a plugin in all but name". ADR-0049: "Curves, LUTs, per-channel colour matrices … are categorically excluded". ADR-0088 *Consequences* confirms the LUT refusal stands. | — |
| White balance selector (eyedropper) | Sample a neutral area, and Temperature and Tint are set automatically. | https://helpx.adobe.com/premiere/desktop/correct-color/color-correction-fundamentals/basic-color-correction-options.html | missing | No tool samples pixels to derive a parameter. `frame` returns an image and computes nothing from it. | Sampling a frame to a white-balance correction. |
| Temperature | Slider that shifts the image cooler or warmer, along the blue to orange axis. | https://helpx.adobe.com/premiere/desktop/correct-color/color-correction-fundamentals/basic-color-correction-options.html | missing | `tint{color, amount}` lerps every pixel toward one colour (`canvas.rs`, ADR-0049), so it is an approximation, not a white-balance shift. ADR-0049 names "a colour-temperature value" as an admissible future scalar, so this is not refused. | A luminance-preserving per-channel gain on a temperature axis. |
| Tint (white balance) | Slider that compensates for green or magenta casts. | https://helpx.adobe.com/premiere/desktop/correct-color/color-correction-fundamentals/basic-color-correction-options.html | missing | As above: `tint` lerps to a colour and is not a cast correction. | A green to magenta balance axis. |
| Exposure | Luminance adjustment of the overall brightness, in stops †. | https://helpx.adobe.com/premiere/desktop/correct-color/color-correction-fundamentals/basic-color-correction-options.html | partial | `brightness{amount}` is a signed additive offset per channel (`canvas.rs` matrix, ADR-0049). | A multiplicative gain (exposure in stops). An offset lifts blacks, and exposure does not. |
| Contrast | Luminance contrast slider. | https://helpx.adobe.com/premiere/desktop/correct-color/color-correction-fundamentals/basic-color-correction-options.html | have | `contrast{amount}`, pivoted on mid-grey: `c' = (c − 0.5)(1 + amount) + 0.5` (`canvas.rs`, ADR-0049). Animatable (ADR-0146). | — (Premiere's exact contrast curve is not documented, so this is equivalent by mechanism, not by pixel.) |
| Highlights | Adjusts the bright areas only. | https://helpx.adobe.com/premiere/desktop/correct-color/color-correction-fundamentals/basic-color-correction-options.html | missing | Every colour filter applies uniformly to every pixel. There is no tonal-range selection. | A luminance-range-weighted adjustment. |
| Shadows | Adjusts the dark areas only. | https://helpx.adobe.com/premiere/desktop/correct-color/color-correction-fundamentals/basic-color-correction-options.html | missing | As above. | As above. |
| Whites | Sets the white clipping point. | https://helpx.adobe.com/premiere/desktop/correct-color/color-correction-fundamentals/basic-color-correction-options.html | partial | `contrast` followed by `brightness` composes any affine map `c' = s·c + t`, so either end point can be placed, but the two ends move together. | An independent white point with its own soft roll-off. |
| Blacks | Sets the black clipping point. | https://helpx.adobe.com/premiere/desktop/correct-color/color-correction-fundamentals/basic-color-correction-options.html | partial | As above. | An independent black point with its own soft roll-off. |
| Saturation (Basic) | Changes the saturation of every colour equally. | https://helpx.adobe.com/premiere/desktop/correct-color/color-correction-fundamentals/basic-color-correction-options.html | have | `saturation{amount}`: 0 is greyscale, 1 is identity, above 1 oversaturates. Skia luma weights (`canvas.rs`, ADR-0049). | — |
| Auto (tone) | Sets the sliders to maximise tonal range and minimise highlight and shadow clipping. | https://helpx.adobe.com/premiere/desktop/correct-color/color-correction-fundamentals/basic-color-correction-options.html | missing | No tool analyses footage to write parameters. | Footage analysis that writes literal `brightness`/`contrast` values. |
| **Lumetri Color: Creative** | | | | | |
| Look (preset) | Menu of preset looks applied as a starting grade. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/apply-lumetri-looks.html | refused | A Lumetri look is a LUT/`.look` file †. ADR-0040 and ADR-0049 refuse LUTs. | — |
| Look Intensity | Slider that scales how strongly the look applies. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/apply-lumetri-looks.html | refused | Falls with the look (ADR-0040, ADR-0049). | — |
| Faded Film | Slider that gives a faded, vintage look by lifting blacks †. | https://helpx.adobe.com/africa/premiere-pro/using/creative-lumetri-looks.html | missing | Negative `contrast` plus positive `brightness` lifts blacks, but it also pulls whites down. | A black-lift that leaves highlights alone. |
| Sharpen | Slider for edge definition. Right sharpens, left softens, 0 is off. | https://helpx.adobe.com/africa/premiere-pro/using/creative-lumetri-looks.html | missing | ADR-0156 *Considered and refused*: "Sharpen … Deferred to keep the first batch small. This is not a judgement against them." Softening is `blur`. | A `sharpen` member. It is deferred, not refused. |
| Vibrance | Saturation that favours less-saturated colours, reduces clipping near full saturation and protects skin tones. | https://helpx.adobe.com/africa/premiere-pro/using/creative-lumetri-looks.html | missing | `saturation` is uniform. | Saturation weighted by each pixel's own saturation. |
| Saturation (Creative) | Global saturation. | https://helpx.adobe.com/africa/premiere-pro/using/creative-lumetri-looks.html | have | `saturation{amount}` (ADR-0049). | — |
| Shadow Tint / Highlight Tint (split toning) | Colour wheels that tint shadows and highlights separately †. | https://helpx.adobe.com/africa/premiere-pro/using/creative-lumetri-looks.html | missing | `tint` is global, so it cannot be split by tone. | Tone-range-weighted tinting. |
| Tint Balance | Balances the shadow tint against the highlight tint †. | https://helpx.adobe.com/africa/premiere-pro/using/creative-lumetri-looks.html | missing | Nothing to balance (see the row above). | As above. |
| **Lumetri Color: Curves** | | | | | |
| RGB Curves: master (luma) curve | A freeform curve on all three channels together. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/correct-color-using-rgb-curves.html | refused | ADR-0049: "a curve or a LUT is not a fixed parameter set, it is an arbitrary function", so curves are "categorically excluded". `CONTEXT.md` **Colour filter** *Avoid*: "curve". | — |
| RGB Curves: per-channel R, G, B | A separate curve per channel. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/correct-color-using-rgb-curves.html | refused | ADR-0049 (curves, and per-channel colour matrices). | — |
| Hue vs Sat curve | Saturation within a chosen hue range. The eyedropper can isolate one colour. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/correct-color-using-hue-and-saturation-curves.html | refused | ADR-0049 (curves). | — |
| Hue vs Hue curve | Shifts one hue range into another. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/correct-color-using-hue-and-saturation-curves.html | refused | ADR-0049 (curves). | — |
| Hue vs Luma curve | Brightness per hue range. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/correct-color-using-hue-and-saturation-curves.html | refused | ADR-0049 (curves). | — |
| Luma vs Sat curve | Saturation by tonal range. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/correct-color-using-hue-and-saturation-curves.html | refused | ADR-0049 (curves). | — |
| Sat vs Sat curve | Remaps a saturation range, for example under a broadcast-safe ceiling. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/correct-color-using-hue-and-saturation-curves.html | refused | ADR-0049 (curves). | — |
| **Lumetri Color: Color Wheels & Match** | | | | | |
| Shadows / Midtones / Highlights colour wheels | Three wheels, each adjusting hue, saturation and brightness for its tonal range. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/correct-color-using-color-wheel.html | missing | No tonal-range operation. ADR-0049's four-clause test would govern a proposal: the arity is fixed, but a colour-typed wheel parameter needs its own justification (the `tint.color` exception is closed). ADR-0049 names `gamma{value}` as admissible in principle. | Lift/gamma/gain by tonal range, with a hue offset per range. |
| Color Match (auto) | Matches the current shot to a reference frame, writing the wheels and Saturation. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/match-color-between-shots.html | missing | No analysis tool. `compare` diffs timelines, not pixels. | A frame-to-frame colour statistic and a tool that writes literal effect values from it. |
| Face Detection (in Color Match) | Weights the match toward facial regions to protect skin tones. On by default. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/match-color-between-shots.html | missing | No face detection anywhere. | As above, plus face detection. |
| Apply Match / fine-tune after match | The match lands as editable wheel and Saturation values, then fine-tuned by hand. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/match-color-between-shots.html | missing | — | The wheels it writes into, and the match. |
| **Lumetri Color: HSL Secondary** | | | | | |
| Key: Set colour eyedropper, plus and minus eyedroppers | Sample a colour to start a qualifier, then add or remove sampled areas. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/correct-color-using-hsl-secondary-controls.html | partial | `chroma{color, tolerance, softness}` keys one literal `#RRGGBB` by distance in the chroma plane (ADR-0088). The colour is typed, not sampled. | Sampling, and growing or shrinking the key with more samples. |
| Key: H, S, L range sliders (range and softness triangles) | Separate hue, saturation and lightness ranges, each with a softness falloff. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/correct-color-using-hsl-secondary-controls.html | partial | `chroma` has one `tolerance` and one `softness` around one colour. ADR-0088 measured hue-angle keying and chose `color` (it does not refuse an HSL qualifier). | Independent H, S and L ranges and softnesses. A luminance-only qualifier. |
| Color/Gray, Color/Black, White/Black mask preview | Shows only the selected range, or the matte, while keying. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/correct-color-using-hsl-secondary-controls.html | missing | `measure` reports keyed coverage numerically (opaque, transparent and partial fractions per frame, ADR-0088). No tool draws the matte. | A matte view from `frame`. |
| Refine: Denoise, Blur | Smooths noise in the selection and softens the matte edge. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/correct-color-using-hsl-secondary-controls.html | partial | `chroma.softness` widens the partial-alpha band (ADR-0088). | A matte blur and a matte denoise. ADR-0088 says edge quality is "specified but untested". |
| Invert mask (HSL Secondary) | Applies the correction outside the key †. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/correct-color-using-hsl-secondary-controls.html | missing | `mask` has `invert` (ADR-0152), but `chroma` has none. | An inverted colour key. |
| Correction on the selection (Temperature, Tint, Contrast, Sharpen, Saturation, wheel) | The correction applies only inside the keyed range. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/correct-color-using-hsl-secondary-controls.html | partial | `chroma` outputs transparency only, and its retained RGB is the RGB it was given (CONTEXT **Matte operation**). Stacking a corrected copy *under* a keyed copy corrects everything **except** the key. | Correcting **inside** the key, which needs an inverted key or a key that gates a correction. Two elements decode the same source. |
| Isolate one colour (colour pass, "Isolate and change a color" how-to) | Hue vs Sat eyedropper drops the saturation of the other hues to zero. | https://helpx.adobe.com/ph_fil/premiere-pro/how-to/isolate-color.html | partial | Composable: the original `video` below, a copy above with `saturation{0}` then `chroma{color: <kept colour>}`. The keyed region shows the colour beneath. `R-CHROMA-AFTER-COLOUR` fires on that order (ADR-0088), and it is review-class, so this is a deliberate exception. | A single-element mechanism keyed by hue range. Today it takes a duplicated element and draws a review finding. |
| **Lumetri Color: Vignette** | | | | | |
| Vignette: Amount, Midpoint, Roundness, Feather | An edge darken or lighten applied after all other Lumetri corrections. | https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/create-vignettes-using-lumetri-color.html | have | ADR-0040 names vignette as composable. A `rect` over the frame with a `radial` gradient fill (ADR-0149: `center`, `radius`, stops with alpha), blended `multiply` or `normal` (or `screen`/`add` to lighten, ADR-0147). Amount is stop alpha, Midpoint is stop offset, Feather is stop spacing, Roundness is the box aspect. Taught in `skills/montagent/SKILL.md` ("the way a vignette or a light leak is built"). | — (The falloff curve is the author's stops, not Premiere's.) |
| **Lumetri Color: structure** | | | | | |
| Fixed section order (Basic, Creative, Curves, Wheels, HSL, Vignette) | Sections process in a fixed order, and HS curves sample after the earlier sections. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/correct-color-using-hue-and-saturation-curves.html | have | `effects` is an ordered list, and order is semantic (ADR-0040). The author chooses the order. | — |
| Multiple Lumetri Color instances on one clip | Stack several Lumetri effects and edit each one. | https://helpx.adobe.com/premiere/desktop/correct-color/color-correction-fundamentals/create-and-edit-multiple-lumetri-color-effects.html | have | Two effects of the same name are legal and apply in order (ADR-0040, ADR-0088 *render reading*). | — |
| Lumetri on an adjustment layer (grade a whole track or sequence) | Effects on an adjustment layer apply to everything below it †. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/correct-color-using-hsl-secondary-controls.html | missing | An effect "filters the element's own pixels" (CONTEXT **Effect**, ADR-0156 §1). There is no element that filters what is beneath it. No ADR refuses one. | An adjustment element, or a track- or project-level `effects`. Today a grade is repeated on every element. |
| Keyframed Lumetri parameters | Effect parameters animate with keyframes. | https://helpx.adobe.com/premiere/desktop/correct-color/color-correction-fundamentals/basic-color-correction-options.html | have | Every numeric and colour effect parameter is animatable (ADR-0146 §3 slice 2). | — |
| **Masks for colour** | | | | | |
| Lumetri effect masks (shape masks, invert, feather, tracking) | Limit a correction to a drawn region, tracked over time †. | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/isolated-adjustments-using-masks.html | partial | A corrected copy of the element carrying a `mask` (`circle`/`rect`/`ellipse`/`path`, `invert`, `feather`, keyed rect; ADR-0084, 0152, 0163, 0165), stacked over the original. | Tracking: the mask is keyed by hand. A mask on the *correction* rather than on a duplicate element. |
| HSL mask (mask from a sampled colour) | Generate a mask from a hue, saturation and lightness selection. | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/create-masks-using-hsl-and-luminance.html | partial | `chroma` makes a colour-distance matte (ADR-0088). | Sampling, H/S/L ranges, an inverted key, and using it as a mask for a correction (see the HSL Secondary rows). |
| Luminance mask | Generate a mask from a brightness range. | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/create-masks-using-hsl-and-luminance.html | missing | ADR-0152: "an image- or luma-sourced mask stays deferred". ADR-0150 refuses a matte taken from *another* element. | A self-luma matte (deferred, not refused). |
| Refine and combine masks | Feather, expand, and add, subtract or intersect masks †. | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/refining-and-combining-masks.html | partial | `feather`, `invert`. Several `mask` members in one list intersect by applying in order (ADR-0040 ordering). | Expansion and choke. Add (union) and subtract between masks. |
| **Color mode (2026, beta)** | | | | | |
| Color mode workspace and Color Management panel (replaces the Lumetri panel in 27.0) | A dedicated grading mode. The Lumetri panel is retired, and the Lumetri effect stays. | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/color-mode-basics.html | missing | Montagent has no GUI. Grading is effect members written in the file. | A grading surface. GUI-only, so it is outside the format's reach, but no ADR refuses it. |
| Adjust colour and contrast (Color mode's default Adjust operation) | Default adjust operation for colour and contrast in Color mode †. | https://helpx.adobe.com/in/premiere/desktop/correct-color/color-mode-fundamentals/default-adjust-operation.html | partial | `brightness`, `contrast`, `saturation`, `tint` (ADR-0049). | Whatever Color mode adds beyond these four scalars. The page body was not read. |
| Color mode with Color Management (inverse tone mapping, beta) | "Apply Inverse Tone Mapping" (public beta), which maps SDR into an HDR working space. | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/using-color-mode-with-color-management.html | missing | No HDR path at all (see Colour management). | Inverse tone mapping. |
| **Auto colour** | | | | | |
| Auto Color (AI) | Automatic colour correction of a clip (Sensei) †. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/use-auto-color.html | missing | No footage analysis. The four invariants of ADR-0145 would accept it if it wrote literal values. | Analysis that writes literal `effects`. |
| **LUTs** | | | | | |
| Creative LUT / custom look file (`.cube`, `.look`) | Browse to a creative LUT in the Creative section. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/apply-lumetri-looks.html | refused | ADR-0040 (external `.cube` is "a plugin in all but name"). ADR-0049. ADR-0017 (closed schema, no escape hatch). | — |
| Input LUT / source-clip LUT in colour management (Modify Clip › Color) | Assign a LUT to media as its input transform †. | https://helpx.adobe.com/premiere/desktop/correct-color/color-correction-fundamentals/color-settings.html | refused | ADR-0040, ADR-0049. | — |
| Export the grade as a LUT (`.cube` / `.look`) | Lumetri panel menu writes the current grade out as a LUT †. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/apply-lumetri-looks.html | missing | Writing a LUT out is not what ADR-0040 refuses (it refuses LUTs as *input*), so this reads as missing rather than refused. | A tool that bakes an element's colour effects to a LUT file. |
| LUT applied at export (Export › Effects › Lumetri Look/LUT) | Burn a LUT into the encode †. | not retrieved | refused | ADR-0040, ADR-0049. ADR-0143 §5 also exposes no export options. | — |
| LUT interpolation preference (trilinear / tetrahedral) | Preference for how 3D LUTs interpolate †. | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/color-management-options.html | refused | No LUTs (ADR-0040, ADR-0049). | — |
| **Lumetri Scopes (Edit mode)** | | | | | |
| Vectorscope YUV | Hue by angle and saturation by distance from the centre. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/display-lumetri-scopes.html | missing | No colour reading. `frame` returns pixels with no statistics. `measure` reads text extents and chroma coverage. `verify` reads size, timing and audio (`mcp.rs`). | A per-frame chroma distribution reading. No ADR refuses one, and ADR-0024's "derivation, no verdict" shape fits it. |
| Vectorscope HLS | Hue, lightness and saturation vectorscope †. | https://helpx.adobe.com/be_en/premiere/desktop/correct-color/add-color-effects/available-lumetri-scopes.html | missing | As above. | As above. |
| Histogram | Distribution of RGB values, with CIN output-range markers (95–685). | https://helpx.adobe.com/be_en/premiere/desktop/correct-color/add-color-effects/available-lumetri-scopes.html | missing | As above. | A per-frame histogram reading. |
| Parade (RGB, YUV, RGB-White, YUV-White) | Per-component waveforms side by side. | https://helpx.adobe.com/be_en/premiere/desktop/correct-color/add-color-effects/available-lumetri-scopes.html | missing | As above. | A per-channel level reading. |
| Waveform (RGB, Luma IRE −20 to 120, YC, YC no chroma) | Signal level across the frame's width. | https://helpx.adobe.com/be_en/premiere/desktop/correct-color/add-color-effects/available-lumetri-scopes.html | missing | As above. | A luma and chroma level reading, for example min, max and clip counts. |
| Scope bit depth (8-bit, 10-bit, float, HDR 0–10000 nits) | Chooses the scale the scopes read in. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/display-lumetri-scopes.html | missing | 8-bit RGBA end to end (`decode.rs` `-pix_fmt rgba`, `encode.rs` `rgb24`→`yuv420p`). | Any scope, and any depth above 8 bits. |
| Clamp Signal | Scope display clips to 0–100 IRE. It does not change output. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/display-lumetri-scopes.html | missing | No scopes. | Falls with the scopes. |
| **Video Scopes (Color mode)** | | | | | |
| Video Scopes panel: Vectorscope, Waveform Monitor, RGB Parade, RGB Overlay | Updated scope set for Color mode. | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/working-with-video-scopes.html | missing | As in the Lumetri Scopes rows. | As in the Lumetri Scopes rows. |
| **Comparison view** | | | | | |
| Comparison View: reference frame vs current frame | Pick a reference position (slider, timecode or edit-point arrows) and view it beside the current clip. | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/about-comparison-view.html | partial | `frame` rasterises any instant, and two calls give the two frames. The `frame --from/--to` contact sheet tiles visual states (`mcp.rs`). | A single picture with reference and current composed together. A pinned reference. |
| Comparison layouts: side by side, vertical split, horizontal split, draggable split line | Display modes for the comparison. | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/about-comparison-view.html | missing | — | A split or side-by-side output from `frame`. |
| Shot comparison: before/after on the current frame | Compare the current frame with and without its correction †. | https://helpx.adobe.com/mena_en/premiere-pro/using/color-match-comparison-view.html | partial | `frame` before and after an edit to the file, or `compare` for the timeline diff. There is no bypass. | An effects-bypass flag on `frame`, so the agent need not edit the file to see "before". |
| **Colour management** | | | | | |
| Colour Management on/off (pass-through for LUT workflows) | Turn automated colour management off for pass-through or LUT workflows. | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/about-color-management.html | partial | Montagent is permanently in what amounts to pass-through: `ffmpeg`'s default YUV→RGBA conversion in, sRGB-encoded arithmetic, untagged `yuv420p` out. | A *managed* mode. Also, nothing records or declares the pass-through, so the conversions are whatever the `ffmpeg` build defaults to. |
| Working colour space | The space all correction and effects are computed in. Adobe advises keeping it fixed during a grade. | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/about-color-management.html | partial | One fixed working space: sRGB-encoded, 8-bit, premultiplied (ADR-0146 §2, ADR-0147 §2 "the arithmetic runs on the stored sRGB values, with no linearisation"). ADR-0147 refuses linear-light blending, and ADR-0146 refuses Oklab interpolation. | A choice of working space, and any wide-gamut or linear space. A linear working space would have to amend ADR-0147. |
| Output colour space (SDR, HDR PQ, HLG) for monitoring and delivery | Chooses the monitoring and delivery space. | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/about-color-management.html | missing | Fixed `yuv420p` 8-bit H.264. ADR-0143 §5: "No project field, and no CLI or MCP option" for encoder settings (it names encoder, preset, CRF and threads, not colour space). | Any output space other than the implicit SDR one. |
| Colour setup preset: Direct 709 (SDR), the default | SDR workflow preset. | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/color-management-options.html | partial | The pipeline is SDR and 8-bit, so it is closest to Direct 709 by behaviour. | Rec.709 is never declared or tagged (no colour tags in `encode.rs`), and decode conversion is not chosen by Montagent. |
| Colour setup preset: Direct HDR (HLG) | HLG workflow preset. | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/color-management-options.html | missing | No HDR path. | HLG decode, working space and encode. |
| Colour setup preset: Direct PQ (HDR) | PQ workflow preset. | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/color-management-options.html | missing | No HDR path. | PQ decode, working space and encode. |
| Colour setup presets: Wide Gamut (Tone Mapped, Minimal Tone Mapping, No Tone Mapping) | Wide-gamut working space with three tone-mapping strengths. | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/color-management-options.html | missing | No wide-gamut path. | A wide-gamut working space and tone mapping. |
| Customise colour setup presets for new or existing sequences | Save and apply custom colour setups. | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/customize-color-setup-presets-for-new-or-existing-sequences.html | missing | No colour settings exist to preset. | Falls with the settings. |
| Sequence colour management settings | Per-sequence working and output space and tone mapping. | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/configuring-sequence-color-management.html | missing | The project's top-level keys are `frame`, `fps`, `background`, `duration`, `loop`, `output`, `master`, `fonts`, `fontVendor` and `tracks` (schema). There is no colour key. | A project-level colour declaration. |
| Log footage handling (ARRI LogC, Canon Log, Sony S-Log, Panasonic V-Log; auto-detect) | Log media transformed automatically to the working space. | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/about-color-management.html | missing | Log footage decodes as flat, display-referred RGB with no transform (`decode.rs`). The only correction route, a LUT, is refused (ADR-0040). | A log-to-display transform, either as a named, closed set of camera transforms or as a source field. |
| Raw camera media colour handling | Supported raw formats are colour-managed automatically. | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/about-color-management.html | missing | Sources are whatever `ffmpeg` decodes. No raw-specific colour handling exists. | Raw debayer and colour transform. |
| Override media colour space (per clip, Modify Clip › Color) | Override the colour space Premiere detected for a source †. | https://helpx.adobe.com/premiere/desktop/correct-color/color-correction-fundamentals/color-settings.html | missing | `probe` reads alpha and dimensions (`media/probe.rs`), not colour space. A `video` element has no colour-space field. | Reading and overriding a source's primaries, transfer, matrix and range. |
| Input tone mapping (per clip) | Tone-maps a clip before effects. The algorithm can differ per clip, and highlight detail may be lost. | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/tone-mapping-in-premiere.html | missing | No HDR sources are understood. An HDR file is converted by `ffmpeg`'s default to 8-bit RGBA without tone mapping. | Per-source tone mapping. |
| Output tone mapping (sequence) | Tone-maps after all effects with one algorithm for the sequence. | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/tone-mapping-in-premiere.html | missing | — | Output tone mapping. |
| Tone-mapping algorithms: By Channel, Highlight Saturation, Hue Preservation, Max RGB | The four algorithms named. Adobe calls Hue Preservation one of the best all-round choices. | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/color-management-options.html | missing | — | A closed enum of algorithms (it would fit ADR-0145's closed-vocabulary invariant). |
| Gamut compression | Fits a wider gamut into a narrower one, helping saturated highlights. | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/color-management-options.html | missing | No gamut is modelled. | Gamut mapping. |
| HDR editing (Rec.2100 PQ/HLG timelines, HDR scopes) | Edit and monitor HDR end to end. | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/tone-mapping-in-premiere.html | missing | 8-bit SDR end to end. | Decode above 8 bits, an HDR working space, HDR encode and metadata. |
| Lumetri and colour-management interplay (Lumetri enhancements) | Lumetri controls adapted to colour-managed and HDR working spaces †. | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/lumetri-enhancements.html | missing | The colour filters have fixed sRGB 0–1 arithmetic (`canvas.rs`). | Space-aware colour filters. |
| Display colour management (GPU-accelerated, monitor profile) | The Program Monitor is shown through the display's ICC profile †. | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/color-management-options.html | missing | There is no viewer. `frame` and `preview` hand files to the agent's own viewer. | Untagged output means the viewer guesses. See the export rows. |
| Viewer gamma (1.96 / 2.2 / 2.4) | Preference for the viewer's transfer function †. | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/color-management-options.html | missing | No viewer. | — |
| HDR graphics white (nits) | Maps graphics white to a nit level in HDR †. | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/color-management-options.html | missing | No HDR. | — |
| **Colour-correction effects (Effects panel)** | | | | | |
| Lumetri Color (as an effect, with presets) | The same controls as the panel, applied as an effect. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/available-color-correction-effects.html | partial | The four colour filters together with `posterize` are the whole colour vocabulary (CONTEXT **Effect**). | Every missing or refused row above. |
| Brightness & Contrast | Two-slider tonal adjustment †. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/available-color-correction-effects.html | have | `brightness`, `contrast` (ADR-0049). | — |
| Black & White | Converts to greyscale †. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/available-color-correction-effects.html | have | `saturation{amount: 0}`, deliberately the grayscale member (ADR-0049). | — |
| Tint (effect: map black to a colour and white to a colour) | A two-colour map with an amount †. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/available-color-correction-effects.html | refused | ADR-0049: "duotone-style two-colour maps are categorically excluded". Montagent's `tint` is the one-colour lerp. | — |
| Leave Color | Desaturates everything except one colour †. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/available-color-correction-effects.html | partial | The colour-pass composition (see "Isolate one colour"). | A single-element member. |
| Change Color / Change to Color | Selects a hue range and shifts its hue, lightness and saturation †. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/available-color-correction-effects.html | missing | No hue-rotation operation, and `chroma` only removes. Not refused: fixed arity is possible. | A hue-range qualifier plus a hue shift. |
| Color Replace | Replaces a sampled colour with another †. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/available-color-correction-effects.html | have (skill) | Composable and taught: a `chroma` key on the element above a flat `rect` of the replacement colour is the green-screen-to-flat-ground recipe in `skills/montagent-footage` (talking-head "Ground"). | — (Only for a uniform target colour, and the replaced area takes a flat colour, not shaded.) |
| Channel Mixer | Each output channel as a weighted mix of the inputs †. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/available-color-correction-effects.html | refused | ADR-0049: "per-channel colour matrices … are categorically excluded". | — |
| Color Balance (shadow, midtone and highlight R/G/B) | Per-tonal-range, per-channel balance †. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/available-color-correction-effects.html | missing | No tonal-range or per-channel operation. | As for the colour wheels. |
| Color Balance (HLS) | Hue rotate, lightness and saturation †. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/available-color-correction-effects.html | partial | `saturation`, and `brightness` for lightness. | Hue rotation. |
| ASC CDL | Slope, offset and power per channel, plus saturation †. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/available-color-correction-effects.html | refused | Read against ADR-0049: per-channel slope and offset form a per-channel colour matrix, which is excluded by name. Master slope and offset already compose from `contrast`+`brightness`. | — (A master-only `gamma` or power is not refused, see Gamma Correction.) |
| Gamma Correction | One gamma value †. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/available-color-correction-effects.html | missing | ADR-0049 names `gamma{value}` as an example of a legitimate future scalar that the rule must not wrongly reject. | A `gamma` member (it would need its own ADR). |
| Levels / ProcAmp | Input and output black and white points with gamma; brightness, contrast, hue and saturation †. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/available-color-correction-effects.html | partial | The affine part composes from `contrast`+`brightness`, plus `saturation`. | Gamma, per-channel levels, and hue. |
| Video Limiter / Broadcast-safe | Clamps luma and chroma to legal broadcast ranges †. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/available-color-correction-effects.html | missing | No range model. The encode does not declare a range (`encode.rs`). | Legal-range clamping and a declared range. |
| SDR Conform | Conforms HDR material to SDR †. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/available-color-correction-effects.html | missing | No HDR. | — |
| Ultra Key (colour-based keying) | Key with matte generation, cleanup, spill suppression and colour correction †. | https://helpx.adobe.com/ph_fil/premiere-pro/how-to/ultra-key-effect.html | partial | `chroma{color, tolerance, softness, spill}` (ADR-0088). `measure` coverage series. `skills/montagent-footage` key recipe. | Matte choke and contrast, a matte view, and a colour-correction subsection. The keying area is owned by the effects ticket, and this row is a cross-reference. |
| **Colour on export** | | | | | |
| Export colour space (Rec.709, Rec.2100 HLG/PQ) | Chooses the delivery colour space in Export settings †. | not retrieved | missing | `encode.rs`: `rgb24` → `-pix_fmt yuv420p`, libx264, CRF 20, no colour tags. ADR-0143 §5: no encoder options. | Any choice. Even the one fixed output is not declared. |
| Colour metadata in the file (primaries, transfer, matrix, range) | Export writes colour description metadata †. | not retrieved | missing | `encode.rs` `Settings::args` and `Spec` pass no `-colorspace`, `-color_primaries`, `-color_trc` or `-color_range`. | Tagging the output, which is what lets players show the render as painted. |
| Maximum bit depth / Render at Maximum Depth | Renders and encodes above 8 bits †. | not retrieved | missing | 8-bit RGBA painter and 8-bit `yuv420p` encode (ADR-0077 §7, ADR-0143). | Higher-depth paint and encode. |
| HDR export (HDR10 metadata, mastering display, MaxCLL/MaxFALL) | HDR delivery metadata †. | not retrieved | missing | No HDR. | — |
| Tone mapping on export | Tone-maps HDR sequences to SDR deliverables †. | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/tone-mapping-in-premiere.html | missing | No HDR. | — |

## Notable gaps

1. **No colour management at all, and the one fixed path is undeclared.** Decode goes through
   `ffmpeg`'s default YUV→RGBA, and the render is 8-bit `yuv420p` with **no colour tags**. A
   player guesses the matrix, transfer and range, so a render can shift on playback. Of everything
   here, this is the most concrete defect and the cheapest to close: a fixed tag, with no new
   field.
2. **Log, raw and HDR footage cannot be handled.** No input transform, no tone mapping and no
   above-8-bit path exist, and the LUT route Premiere users take instead is refused (ADR-0040,
   ADR-0049). Log footage stays flat with no correct remedy.
3. **No adjustment layer.** Every colour effect filters one element's own pixels, so a grade is
   repeated on every element and cannot cover a track or the frame. No ADR refuses an adjustment
   element.
4. **No scopes and no colour reading.** An agent cannot measure exposure, clipping or a cast. It
   can only look at `frame` images. A per-frame luma and chroma reading fits ADR-0024's
   "derivation, no verdict" shape for `measure`, and nothing refuses it.
5. **No tonal-range operations.** Highlights, Shadows, the three wheels, split toning and Color
   Balance are all missing. ADR-0049's test does not rule them out in principle, but the closed
   `tint.color` exception means a colour-typed wheel parameter needs its own justification.
6. **Secondary correction is half-built.** `chroma` gives a colour key, but it has no inverted key
   and cannot gate a correction, so you can correct everything **except** a colour and never
   **only** that colour.
7. **Curves, LUTs, channel mixing, duotone and CDL are refused by ADR-0049 and ADR-0040.**
   They make up most of Premiere's power-grading surface (17 refused rows).
8. **Small, unrefused scalar members:** `gamma`, `sharpen` (deferred by ADR-0156), `vibrance`, a
   temperature axis (named admissible in ADR-0049), and exposure as gain.
9. **Comparison is partial.** `frame` can show two instants, but it has no split or side-by-side
   output and no effects-bypass "before" view.

### Cross-area notes (not counted)

- **Blend modes (ADR-0147):** 5 of Premiere's 27. The colour-family modes (Color, Hue,
  Saturation, Luminosity, Soft Light and others) are refused *for now* by ADR-0147 *Considered
  and refused*. They are owned by the compositing ticket.
- **Gradients (ADR-0149):** linear and radial paint ship. Repeat and mirror ("Premiere's newer
  Gradient *effect*") are refused for now, and sweep and conical need a prototype. They are owned
  by the graphics ticket. The radial gradient is what makes the vignette row `have`.

### Follow-up for assembly

Rows marked † and the four *not retrieved* export rows were not confirmed against a helpx page
body, because this session could not reach `helpx.adobe.com` and the shared search budget ran
out. Re-read the following before the inventory is merged:
[available-color-correction-effects](https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/available-color-correction-effects.html),
[color-management-options](https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/color-management-options.html),
[color-settings](https://helpx.adobe.com/premiere/desktop/correct-color/color-correction-fundamentals/color-settings.html),
[lumetri-enhancements](https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/lumetri-enhancements.html),
[color-mode-basics](https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/color-mode-basics.html),
and Premiere's Export-settings colour page (not found).
