# Premiere gap: video effects, video transitions and effect controls

Research for [#885](https://github.com/MBehtemam/Montagent/issues/885), on the map
[#882](https://github.com/MBehtemam/Montagent/issues/882).

- **Date:** 2026-10-10.
- **Premiere side:** Adobe Premiere 26.x (26.0 through 26.5), from Adobe's own helpx.adobe.com
  user guide only. The anchor is Adobe's table
  [Effects and transitions changes in Premiere 26.0][L]. That table lists every effect and
  transition as current, Legacy, Obsolete or "Removed in version 26.0 (will not render)". The
  per-bin library pages, the [Modern effects][MF] and [Modern transitions][MT] pages, the
  [Immersive video effects][IMF] page and the apply/controls pages fill it in. Page URLs match
  the census on `research/premiere-gap-census` (#883) where it has them.
- **Montagent side:** tag `v0.2.0` (`51aa4b5` = `origin/main`): `CONTEXT.md`,
  `schema/montagent.schema.json`, `crates/montagent-core/docs/format.md`, `docs/adr/`, the ten
  MCP tools in `crates/montagent/src/mcp.rs` (`validate`, `create_project`, `query`, `frame`,
  `render`, `verify`, `preview`, `measure`, `shift`, `compare`), `skills/`, and
  `docs/research/skia-reach/named-effects-candidates.md` (#703).
- **What v0.2.0 has to compare against:**
  - **12 `effects` members:** `blur`, `shadow`, `mask` (ADR-0040), `tint`, `saturation`,
    `brightness`, `contrast` (ADR-0049), `chroma` (ADR-0088), and `grain`, `glow`,
    `posterize`, `directional_blur` (ADR-0156). They apply in list order, and every numeric
    and colour parameter is animatable (ADR-0146).
  - **Element fields:** `blend` with `normal`, `multiply`, `screen`, `overlay` and `add`
    (ADR-0147). `motion_blur` (ADR-0155). `swivel`, `tilt` and `perspective` (ADR-0167).
    `clip` and `fit`. `scale` (a negative component flips). `opacity`. `speed`.
    `source_time` (ADR-0157). Gradient paint (ADR-0149).
  - **Transition kinds:** `crossfade`, `wipe`, `slide`, `push` (ADR-0059, ADR-0150), and
    `audio_crossfade` (ADR-0176).

## Counts

| Status | Rows |
|---|---|
| have | 69 |
| have (skill) | 15 |
| partial | 50 |
| missing | 128 |
| refused | 17 |
| **total** | **279** |

## How to read this

- **One row per effect and per transition**, as the ticket asks. Adobe's 26.0 table both
  renames older effects and adds about 90 Film Impact–derived ones. When a modern effect has
  the same name as an older one (Gaussian Blur, Cross Dissolve, Mosaic), the two share a row
  and the mechanism column says so. Legacy, Obsolete and Removed entries keep their own rows,
  so nothing Adobe lists is dropped. Adobe's 26.0 state is written in the mechanism column.
- **The status follows the map's vocabulary**, read by mechanism (ADR-0145):
  - `have`: an equivalent mechanism ships in v0.2.0, either as a named member or as elements
    composed in a way the format or skill documents. "Composed" names the parts.
  - `have (skill)`: a bundled skill supplies the mechanism (a recipe or a script), not a
    format field.
  - `partial`: some of the mechanism ships. The last column says exactly what is missing.
  - `missing`: nothing equivalent ships, and no ADR refuses it.
  - `refused`: an ADR, or ADR-0145's four invariants, rules it out. The ADR is cited.
- **The pre-render route is not counted as having a filter.** ADR-0156 §7 and the
  `montagent-prerender` skill bring a look drawn in code into the project as footage. For a
  **generator**, a look with no source pixels (lens flare, lightning, cell pattern), that is the
  same mechanism, so the row is `have (skill)`. For a **filter** on an element's own pixels
  (Twirl, Find Edges, Mosaic), pre-rendering means processing the footage outside Montagent.
  The filter is not a parameter in the project, and it cannot be keyed. Those rows stay
  `missing`, and the last column names the workaround.
- **Transitions composed from elements.** A Montagent transition element has four visual
  kinds. Many Premiere transitions are a keyed mask, keyed transform or keyed effect on the
  two bridged elements, laid over a `crossfade` or a cut. ADR-0150 names a keyed `mask` as the
  reveal mechanism. These rows are `have` when the composition is exact, and `partial` when an
  ingredient is missing.
- **Parameters.** Adobe's 26.x pages give one line per effect and almost no controls (#703
  found the same). Where a parameter decides the status, it is quoted from an Adobe page.
  Otherwise the row compares the mechanism the one-line description names.

### Limits of this research

- `helpx.adobe.com` was blocked by this session's egress policy: the proxy refused CONNECT,
  and WebFetch could not resolve the host. Every Adobe fact came through web searches
  restricted to `helpx.adobe.com`, which return Adobe's own page text as snippets. Each row
  links the Adobe page the fact came from. Where the search snippets disagreed, both readings
  are given (see [Conflicts in Adobe's pages](#conflicts-in-adobes-pages)).
- The session's search budget ran out before four pages could be read: the Premiere
  adjustment-layer page, and the Channel, Utility and Video library pages. Their rows cite
  the 26.0 table, the Video effects index or the Types of effects page instead, and say so.
  #703's note (2026-10-06, read in a browser) was used to cross-check the effect list.
- A few modern names (Spacer FX, Clone FX, Alpha FX, Motion Tween, Slice, Stripe) appear on
  Adobe's pages with no description that names a mechanism. Their status is the best reading
  of the name, and the row says so. These are candidates for the follow-up the map's "Not yet
  specified" anticipates.

## Effect controls: how effects behave

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Fixed (intrinsic) effects: Motion, Opacity, Time Remapping | Built into every clip and always present. They cannot be removed or reordered. | [ECP][ECP], [TYP][TYP] | have | Element transform fields `x`/`y`/`scale`/`rotation`/`origin` (ADR-0012), `opacity`, `source_time` (ADR-0157) | — |
| Standard effect stack order | Standard effects render top-down in the order listed in Effect Controls. They are reordered by dragging. | [ECV][ECV] | have | `effects` is an ordered list and order is significant (ADR-0040); reordering is moving a list item | — |
| Render order: standard effects before fixed effects | Premiere renders standard effects first, then Motion, Opacity and Time Remapping. | [ECV][ECV] | have | A fixed, documented order: `clip`, then `effects` and mask in list order, then projection, then `scale`/`rotation`/position, then `opacity`, then `blend` (format.md; ADR-0147, ADR-0155 §4, ADR-0167) | — |
| Moving a geometry step into the stack (Transform effect in place of Motion, Alpha Adjust in place of Opacity) | Adobe's workaround for render order: a standard Transform or Alpha Adjust effect sits anywhere in the stack. | [ECV][ECV] | partial | Opacity and transform are element fields at fixed positions in the order | No way to put a transform or opacity step *before* an effect (e.g. blur after scaling). The only route is nesting, which v0.2.0 lacks (ADR-0166; `montagent` skill "can't do yet", #499). |
| Effect on/off toggle (fx bypass) | Each effect has an enable toggle in Effect Controls. | [ECP][ECP] | missing | Audio effects carry `enabled` (ADR-0179). Visual `effects` members have no such key | A visual effect can only be disabled by deleting it, which loses its parameters. |
| Keyframing effect parameters | Stopwatch per parameter; keyframes in Effect Controls and the Timeline. | [KF][KF] | have | Every numeric and colour effect parameter takes a keyframe list (ADR-0146) | — |
| Keyframe interpolation on effect parameters | Linear, Bezier, Auto Bezier, Continuous Bezier, Hold, Ease In/Out. | [KFI][KFI] | have | `Ease` vocabulary: named eases, cubic-bezier arrays, `step` = hold (schema `Ease`; ADR-0038) | Velocity-graph editing is a UI concern; the bezier is written as numbers. Spring easing is refused (ADR-0146). |
| Effect masks (a mask limits one effect to a region) | Ellipse, 4-point and pen masks sit inside an effect, so e.g. a blur applies only inside the mask. | [MSK][MSK] | partial | `mask` is an `effects` member (ADR-0068), with rect/circle/ellipse/path shapes (ADR-0084, ADR-0163), `invert` and `feather` (ADR-0152) | A mask cuts the **whole element's** alpha. It cannot scope a single effect. A regional blur needs a second, masked copy of the element. Mask expansion and per-mask opacity are absent. |
| Mask tracking | Track a mask forward or back, by position, scale, rotation or perspective. | [MTK][MTK] | missing | Mask geometry is keyable by hand (ADR-0146) | No tracker. An agent would have to measure motion and write keyframes itself. |
| Effect presets: built-in and custom | Save an effect stack with its keyframes as a preset. Apply it by drag, with Anchor to In/Out or Scale. Ships with preset bins. | [CPR][CPR], [APR][APR] | partial | Skill recipes act as named, documented presets (`montagent-motion` trailer/logo-loop references, `montagent-footage`); an agent copies the JSON in | No preset store in the format, no apply-by-name, and no anchor-to-in/out or scale-to-clip retiming of the pasted keyframes. Indirection is avoided on purpose (ADR-0002's no-asset-table reasoning). |
| Copy and paste effects; Paste Attributes | Copy a clip's effects and paste them onto other clips, with a choice of attributes. | [CPE][CPE] | have | The `effects` list is literal JSON; copying it is an exact-string edit (ADR-0145 invariant 4) | — |
| Remove effects; Remove Attributes | Remove one or all effects from selected clips. | [REM][REM] | have | Delete the member, or the `effects` key | — |
| Source (master) clip effects | An effect applied to the source clip ripples to every sequence clip made from it (shown by a red line under the fx badge). Video effects only. | [MCE][MCE] | refused | ADR-0002: no asset table, and each element names its file inline | There is no per-file record for a shared effect to live on. Inferred from ADR-0002; no ADR names master-clip effects. |
| Adjustment layers | A transparent clip on a higher track. Its effects apply to the composite of every track beneath it. | [TYP][TYP] (adjustment-layer page not reached) | missing | No element filters the composite beneath it. `blend` mixes one element's own pixels with the backdrop (ADR-0147) | An effect over the composite below. ADR-0167 §10 calls this "group compositing" and leaves it in the map's fog, gated on nests. |
| Effects on a nested sequence | A nest is a clip, so it takes effects that apply to the flattened group. | [TYP][TYP] | missing | ADR-0166 records that the reference class nests with a transform; no `nest` is admitted in v0.2.0 (`montagent` skill: "Parenting, groups and cameras", #499) | A container, and group compositing for its effects. |
| Third-party effect plug-ins | After Effects-compatible and other plug-ins appear in the Effects panel. | [PLG][PLG] | refused | ADR-0040 (vocabulary is "never a plugin architecture"), ADR-0017 (closed schema), ADR-0156 §8 (an open shader or script stays refused) | — |
| Finding and grouping effects | Search box, custom bins, accelerated/32-bit/YUV badges. | [FG][FG] | have | The closed vocabulary is published in the schema, each member described (ADR-0040's discoverability test) | Custom bins are UI and have no counterpart. |
| Markers in the Effect Controls time ruler | Clip markers shown in Effect Controls to time keyframes. | [ECM][ECM] | missing | No marker key in the schema. `measure` and `query --at` read times instead | Markers as authored data. |
| Legacy and Obsolete effects keep rendering in old projects | Obsolete effects are hidden but still render. Removed ones (26.0) "will not render". | [RO][RO], [L][L] | have | A different mechanism: a retired key is an unknown-key error that names the retirement, and the render is blocked (ADR-0016, ADR-0017) | — |
| GPU-accelerated and 32-bit effects | Effects marked accelerated render on the GPU in real time. | [ACC][ACC] | missing | Rendering is CPU Skia on K painters (ADR-0144), byte-identical by design | GPU rendering. A performance item; the performance research area owns it. |

## Video effects

### Adjust

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Auto Color | Analyses the frame and neutralises the colour cast. Removed in 26.0; Adobe points to Lumetri. | [L][L], [RM][RM] | missing | — | A colour correction computed from source pixels. |
| Auto Contrast | Automatic contrast from the frame's histogram. Removed in 26.0. | [L][L], [RM][RM] | missing | `contrast` exists, but only as a literal amount (ADR-0049) | The automatic analysis. |
| Auto Levels | Automatic per-channel levels. Removed in 26.0. | [L][L], [RM][RM] | missing | — | Per-channel levels, automatic or manual. |
| Convolution Kernel | A user-defined 3×3 kernel with scale and offset. | [ADJ][ADJ] | missing | — | An arbitrary kernel. #703 maps it to Skia `matrix_convolution`. |
| Extract | Keeps a luminance range and renders it black and white, with softness. | [ADJ][ADJ], [L][L] | missing | `saturation{0}` plus `contrast` approximate it (ADR-0049) | A luminance-range threshold with softness. |
| Levels | Input and output black and white points and gamma, per channel or RGB. | [ADJ][ADJ], [L][L] | missing | `brightness`/`contrast` are single offsets (ADR-0049) | Black and white points, gamma, and per-channel levels. Check against ADR-0049's four-clause rule before any proposal. |
| Lighting Effects | Up to five spot, omni or directional lights, with a bump layer. | [ADJ][ADJ], [L][L] | missing | A radial gradient rect blended `add`/`screen` gives a flat light (ADR-0147, ADR-0149) | Lights with direction, falloff and surface relief. A bump map from another track would be refused (ADR-0150). |
| ProcAmp | Brightness, contrast, hue and saturation in one effect, with a split-screen view. | [ADJ][ADJ], [L][L] | partial | `brightness`, `contrast`, `saturation` (ADR-0049) | Hue rotation. The split-screen compare is UI. |
| Shadow/Highlight | Lifts shadows and lowers highlights by local analysis. Obsolete or removed; Adobe points to Lumetri. | [RM][RM] | missing | — | Tone-range-dependent lift and compression. |

### Blur & Sharpen

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Gaussian Blur (Film Impact "Blur FX"; old version is Legacy) | Gaussian blur with Blurriness, Blur Dimensions (horizontal, vertical, both) and Repeat Edge Pixels. | [BS][BS], [L][L], [MF][MF] | partial | `blur{radius}`, keyable (ADR-0040, ADR-0146) | Horizontal-only or vertical-only blur, and repeat edge pixels. A one-axis blur is approximated by `directional_blur` at 0° or 90° (ADR-0156). |
| Directional Blur (Film Impact; old version is Legacy) | Smear along an angle by a length. | [BS][BS], [L][L] | have | `directional_blur{angle, length}` (ADR-0156 §4) | — |
| Camera Blur (Legacy) | A lens-style defocus by a percentage. | [L][L], [RM][RM] | partial | `blur` (ADR-0040) | A lens kernel. `blur` is Gaussian. |
| Bokeh Blur (Film Impact, also listed under Lights & Glows) | Defocus with bokeh highlights. | [L][L], [MF][MF] | missing | — | Shaped bokeh kernel and highlight bloom. |
| Compound Blur (Film Impact; the old one is Obsolete) | The classic effect blurs by the luminance of a "blur layer" on another track. | [L][L], [RM][RM], [MF][MF] | refused | ADR-0150 §4: a matte or map taken from another element is refused | Adobe's page does not say whether the modern Compound Blur FX still reads another layer. If it does not, re-read this row as `missing`. |
| Focus Blur (Film Impact) | Blur that leaves a focal region sharp. | [L][L] | partial | `blur` on a copy plus an inverted, feathered `mask` gives a focal area (ADR-0152) | One member. Today it takes two elements. |
| Channel Blur (re-added in 26.2.2 as Film Impact) | Separate blur amounts for R, G, B and alpha. | [RN][RN], [RM][RM] | missing | — | Per-channel blur. |
| Fast Blur | Faster box-style blur. Removed in 26.0. | [L][L] | have | `blur` (ADR-0040) | — |
| Reduce Interlace Flicker | Vertical softening against interlace twitter. | [BS][BS], [L][L] | missing | Output is progressive H.264 (ADR-0143) | Not applicable to progressive output. Listed for completeness. |
| Sharpen | Increases contrast where colour changes. | [BS][BS], [L][L] | missing | — | A sharpen member. #703: Skia `matrix_convolution`. |
| Unsharp Mask | Sharpen with amount, radius and threshold. | [BS][BS], [L][L] | missing | — | As Sharpen, plus a threshold. |

### Channel

The Channel library page was not reached. These rows cite the 26.0 table and the removal page.

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Arithmetic | Per-channel operators (and, or, add, multiply…) against constant values. Removed in 26.0. | [L][L], [RM][RM] | missing | — | Per-channel arithmetic. ADR-0049 excludes per-channel colour matrices, so any proposal starts there. |
| Blend | Blends the clip with a clip on another track by crossfade, colour-only, tint-only, darken or lighten. Removed in 26.0; Adobe points to Blend Modes. | [L][L], [RM][RM] | partial | `blend` with five modes over whatever lies beneath (ADR-0147) | Naming a specific other layer is refused (ADR-0150). Colour-only, tint-only, darken and lighten modes were refused for now (ADR-0147 "Considered and refused"). |
| Calculations | Combines one channel of this clip with one channel of another track. Removed in 26.0. | [L][L], [RM][RM] | refused | ADR-0150 §4: reading another element's pixels | — |
| Compound Arithmetic | Mathematical combination with a second source layer. Removed in 26.0. | [L][L], [RM][RM] | refused | ADR-0150 §4 | — |
| Invert (moved to Image Control in 26.0) | Inverts RGB, HLS, YIQ or alpha channels. | [IC][IC], [RO][RO] | missing | ADR-0049 names `invert` as a request its stopping rule would test | An invert member. #703: one colour-matrix call. |
| Set Matte | Takes a channel of another layer as this clip's alpha. Obsolete. | [RM][RM] | refused | ADR-0150 §4: a matte from another element is refused | — |
| Solid Composite | Composites a solid colour behind the clip, with opacity and a blend mode. Obsolete; Adobe points to generators. | [RM][RM] | have | Composed: a `rect` element on a lower track, with `fill`, `opacity` and `blend` (ADR-0147) | — |

### Color Correction

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| ASC CDL | Slope, offset and power per channel, plus saturation. | [CC][CC], [L][L] | missing | `saturation` (ADR-0049) | Per-channel slope, offset and power. Test against ADR-0049's rule. |
| Brightness & Contrast | Brightness and contrast sliders. | [CC][CC], [L][L] | have | `brightness`, `contrast` (ADR-0049) | — |
| Lumetri Color | Basic correction (white balance, exposure, contrast, highlights, shadows, whites, blacks, saturation), Creative (Look/LUT, faded film, sharpen, vibrance), Curves (RGB and Hue Saturation), Color Wheels, HSL Secondary, Vignette. | [CC][CC], [LUT][LUT], [LV][LV] | partial | `brightness`, `contrast`, `saturation`, `tint` (ADR-0049). Vignette composed (ADR-0040). Curves and LUTs refused (ADR-0049, ADR-0040) | White balance, exposure, the tone-range controls, vibrance, sharpen, colour wheels and HSL secondaries are missing. The colour area ticket covers Lumetri in depth. |
| Look-up tables (Lumetri Input LUT and Look) | Apply a `.cube` or `.look` file. | [LUT][LUT] | refused | ADR-0040 ("semantics live entirely in an external `.cube` file … a plugin in all but name"), ADR-0049 | — |
| RGB Curves, Luma Curve, Hue Saturation Curves | Arbitrary tone curves. RGB Curves and Luma Curve were removed earlier; the curves now live in Lumetri. | [RM][RM], [CC][CC] | refused | ADR-0049: "Curves, LUTs, per-channel colour matrices, and duotone-style two-colour maps are categorically excluded" | — |
| Tint | Maps black to one colour and white to another, by an amount. | [CC][CC], [L][L] | partial | `tint{color, amount}` pushes every pixel toward one colour (ADR-0049) | Premiere's two-colour map (Map Black To, Map White To) is a duotone, which ADR-0049 excludes. The one-colour wash ships. |
| Video Limiter | Clamps luma and chroma to broadcast-legal range. | [CC][CC], [L][L] | missing | — | A legal-range clamp. |
| Vignette (Film Impact "Vignette FX") | Darkens the frame edges. | [CC][CC], [MF][MF] | have | Composed: a radial-gradient `rect` to transparent, blended `multiply` (ADR-0040, ADR-0149, ADR-0147; `montagent` skill) | — |
| Change Color | Changes the hue, lightness and saturation of a selected colour range. Removed in 26.0. | [L][L] | missing | — | Hue-selective correction. |
| Change to Color | Replaces a selected colour with another. Removed in 26.0. | [L][L] | missing | — | Hue-selective replacement. |
| Color Balance (RGB) | Red, green and blue levels. Obsolete in 26.0. | [L][L], [RM][RM] | missing | `tint` shifts toward one colour (ADR-0049) | Independent per-channel gain. |
| Color Balance (HLS) | Hue, lightness and saturation. | [RM][RM] | partial | `brightness`, `saturation` (ADR-0049) | Hue rotation. |
| Leave Color | Desaturates everything except one colour. Removed in 26.0. | [L][L] | missing | — | A colour-keyed desaturation. |
| Equalize | Equalises the brightness histogram. | [RM][RM] | missing | — | Histogram equalisation. |
| Three-Way Color Corrector, Fast Color Corrector, RGB Color Corrector, Luma Corrector | Colour wheels and tonal-range correction. Removed; replaced by Lumetri. | [L][L], [RM][RM] | missing | — | Tonal-range colour wheels. |

### Distort

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Corner Pin | Moves the four corners independently, a free perspective quad. | [DIS][DIS], [L][L] | refused | ADR-0167: "a camera, depth sorting, skew and corner pin stay refused"; swivel/tilt cover rigid turns | — |
| Lens Distortion | Barrel and pincushion distortion. Listed as current and also as removed in 26.0 (see Conflicts). | [DIS][DIS], [L][L] | missing | — | A lens warp. Workaround: `montagent-prerender`. |
| Magnify (Film Impact "Magnify FX"; old version is Legacy) | A loupe with size, magnification, edge softness, border and shadow. | [DIS][DIS], [MF][MF] | partial | Composed: a scaled copy of the element with a circular `mask`, `feather` and `shadow` | A loupe that magnifies the element's own pixels in place, without a second element. |
| Mirror | Reflects the image across a line at an angle. | [DIS][DIS], [MF][MF] | partial | `scale: [-1, 1]` flips a whole element (format.md). A flipped, masked copy gives an axis-aligned mirror | A mirror line at an arbitrary angle within one element. |
| Offset (moved to Transform in 26.0) | Pans the image with wrap-around. | [TR][TR], [RO][RO] | missing | — | Wrap-around tiling. #703: an image shader with repeat tiling. |
| Rolling Shutter Repair | Analyses and corrects rolling-shutter skew. Obsolete in 26.0. | [DIS][DIS], [L][L] | missing | — | Motion analysis. |
| Spherize | Wraps the image round a sphere. | [DIS][DIS], [L][L] | missing | — | A bulge warp. Workaround: `montagent-prerender`. |
| Transform (effect) | Anchor, position, scale, skew, rotation, opacity and its own shutter angle, as a standard effect. | [DIS][DIS], [TR][TR] | partial | Transform fields, `opacity`, `motion_blur` (ADR-0012, ADR-0155) | Skew is refused (ADR-0012, ADR-0040, ADR-0167). There is no way to place the transform inside the effect order (see Effect controls). |
| Turbulent Displace | Procedural turbulence displacement, with Evolution and Random Seed. | [DIS][DIS], [MF][MF] | missing | — | A seeded noise displacement. ADR-0156 §3 has the seed and time rules it would need. |
| Twirl | Rotates pixels round a centre, with radius and angle. | [DIS][DIS], [MF][MF] | missing | — | A twirl warp. Workaround: `montagent-prerender`. |
| Warp Stabilizer | Analyses camera motion and smooths or locks it, with methods from Position to Subspace Warp, and border options. | [WS][WS], [WSS][WSS] | missing | — | Motion analysis and stabilisation. |
| Wave Warp | A travelling wave that animates without keyframes; type, height, width, direction, speed and phase. | [DIS][DIS], [L][L] | missing | — | A wave member. ADR-0156 §3 says a wave takes a keyed phase rather than a time input. |

### Generate

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| 4-Color Gradient | A gradient between four coloured points. | [GEN][GEN], [L][L] | partial | Linear and radial gradient paint (ADR-0149) | A four-point gradient. |
| Gradient (Film Impact, added 26.2.2) | Gradient with position, scale, orientation, repetition, mirroring, interpolation, feathering and grain. | [RN][RN], [MF][MF] | partial | Gradient paint with animatable angle, centre, radius and stops (ADR-0149); `grain` (ADR-0156) | Repetition and mirroring, and an interpolation choice. |
| Ramp (Legacy) | Linear or radial ramp with Ramp Scatter. | [GEN][GEN], [L][L] | have | Linear and radial gradient paint (ADR-0149) | Ramp Scatter (dithering). |
| Cell Pattern | Procedural cellular pattern. | [GEN][GEN] | have (skill) | `montagent-prerender`: a code-drawn piece as lossless footage with alpha (ADR-0156 §7) | — |
| Checkerboard | Procedural checkerboard. | [GEN][GEN] | have (skill) | `montagent-prerender`; or one `rect` per square, written out (ADR-0148) | — |
| Circle | Draws a circle or ring. | [GEN][GEN] | have | `ellipse` element with `fill` and `stroke` | — |
| Ellipse | Draws an ellipse outline. | [GEN][GEN] | have | `ellipse` element | — |
| Eyedropper Fill | Fills the clip with a colour sampled from the clip. | [GEN][GEN] | missing | Colours are literal hex strings (ADR-0014) | A colour sampled from pixels. That would fail ADR-0145's literal-values test if done at render time. |
| Grid | Procedural grid. | [GEN][GEN] | have (skill) | `montagent-prerender`; or `path` elements written out | — |
| Lens Flare (Obsolete); Lens Flare FX (Film Impact) | A flare with shape, streak intensity and colour. | [GEN][GEN], [L][L], [MF][MF] | have (skill) | `montagent-prerender` names lens flare in its trigger (ADR-0156 §7) | — |
| Lightning (Obsolete) | Animated lightning between two points, without keyframes. | [GEN][GEN], [L][L] | have (skill) | `montagent-prerender` | — |
| Paint Bucket | Flood-fills a region chosen by a point. | [GEN][GEN] | missing | — | A pixel-dependent fill. |
| Write-on | Paints strokes along a keyframed brush position. | [GEN][GEN] | have | `trim_start`/`trim_end` draw a stroke along a `path` (ADR-0160) | — |

### Image Control

Invert moved here in 26.0. It has one row, under Channel.

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Black & White | Converts to greyscale. | [IC][IC], [L][L] | have | `saturation{amount: 0}` (ADR-0049) | — |
| Color Pass | Keeps one colour and desaturates the rest. | [IC][IC] | missing | — | A colour-keyed desaturation. |
| Color Replace | Replaces a chosen colour, with a similarity range. | [IC][IC] | missing | — | Colour-range replacement. |
| Gamma Correction | Adjusts gamma. | [IC][IC] | missing | ADR-0049 cites `gamma{value}` as a legitimate future scalar | A gamma member. |

### Immersive Video

Montagent has no spherical (360°/180°) projection model, and no ADR refuses one. Each effect
is `missing` because the projection it needs is missing. The census (#883) marks immersive
pages UNCOVERED.

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| VR Blur | Blur that wraps seamlessly across a 360° frame. | [IMF][IMF] | missing | `blur` is planar (ADR-0040) | Spherical wrap. |
| VR Chromatic Aberrations | Colour fringing at the edges of a 360° frame. | [IMF][IMF] | missing | — | Spherical model, and channel offsets. |
| VR Color Gradients | Gradient fills that blend across the sphere. | [IMF][IMF] | missing | Planar gradients only (ADR-0149) | Spherical model. |
| VR De-Noise | Noise reduction for 360° footage. | [IMF][IMF] | missing | — | Denoise, and the spherical model. |
| VR Digital Glitch | Digital-glitch distortion across the sphere. | [IMF][IMF] | missing | — | Spherical model. |
| VR Fractal Noise | Fractal-noise textures over the sphere. | [IMF][IMF] | missing | — | Spherical model and fractal noise. |
| VR Glow | Glow round bright areas, seamless across the sphere. | [IMF][IMF] | missing | `glow` is planar (ADR-0156) | Spherical wrap. |
| VR Plane to Sphere | Projects a flat layer (logo, title) onto equirectangular footage. | [IMF][IMF] | missing | — | Equirectangular projection. |
| VR Projection | Re-projects and rotates 360° footage. | [IMF][IMF] | missing | — | Spherical model. |
| VR Rotate Sphere | Rotates 360° footage about X, Y and Z. | [IMF][IMF] | missing | — | Spherical model. |
| VR Sharpen | Sharpening for 360° footage. | [IMF][IMF] | missing | — | Sharpen, and the spherical model. |

### Keying

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Ultra Key | Sampled key colour. Matte generation (transparency, highlight, shadow, tolerance, pedestal). Matte cleanup (choke, soften, contrast, mid point). Spill suppression. Colour correction. Output views (composite, alpha, colour channel). | [KEY][KEY], [UK][UK] | partial | `chroma{color, tolerance, softness, spill}`, all keyable (ADR-0088); `montagent-footage` green-screen recipe | Choke, pedestal, highlight and shadow controls, matte contrast and mid point, and an alpha-channel view. Contrast and softness map only roughly. |
| Color Key | Keys out one colour by tolerance, with edge thin and edge feather. | [KEY][KEY] | partial | `chroma` with `tolerance` and `softness` (ADR-0088) | Edge thin (choke). |
| Luma Key | Keys out by luminance threshold and cutoff. | [KEY][KEY], [L][L] | missing | — | A luminance matte. ADR-0152 defers luma-sourced masks. |
| Track Matte Key | Uses another track's alpha or luma as this clip's matte. | [KEY][KEY], [L][L] | refused | ADR-0150 §4 refuses a matte from another element; the element's own `mask` is the documented substitute | — |
| Alpha Adjust | Opacity, Ignore Alpha, Invert Alpha, Mask Only. | [KEY][KEY] | partial | `opacity`; `mask` with `invert` (ADR-0152) | Ignore Alpha, Invert Alpha on the source's own alpha, and Mask Only. |
| Logo Cutout | Named in Adobe's keying list, with no mechanism described. | [KEY][KEY] | missing | — | Mechanism undocumented on the page reached; re-check. |
| Image Matte Key | Uses a still image's alpha or luma as the matte. Removed earlier. | [RM][RM] | missing | ADR-0152 keeps image-sourced masks deferred, not refused | An image-sourced mask. |
| Difference Matte | Keys out pixels that match a reference clip. Removed earlier. | [RM][RM] | refused | ADR-0150 §4: a matte computed from another element's pixels | — |
| Remove Matte | Removes the colour fringe from premultiplied footage. | [RM][RM] | missing | — | Unpremultiply against a matte colour. |

### Noise & Grain

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Noise (Film Impact in 26.2.2) | Animated noise, with level spread across shadows, midtones and highlights, noise saturation, blend mode and preserve alpha. | [NG][NG], [MF][MF], [RN][RN] | partial | `grain{seed, amount, size, mono}`, re-rolled every frame from a literal seed and alpha-preserving (ADR-0156 §3–4) | Tonal-range weighting, and a blend mode inside the effect. A grain `rect` with `blend: overlay` covers the latter. |
| Noise Alpha | Noise in the alpha channel. Obsolete or removed. | [RM][RM] | missing | `grain` never touches alpha (ADR-0156) | Alpha noise. |
| Noise HLS, Noise HLS Auto | Separate noise in hue, lightness and saturation. Obsolete or removed; Adobe points to Noise. | [RM][RM] | partial | `grain` with `mono` true (luma) or false (colour) (ADR-0156) | Separate hue, lightness and saturation amounts. |
| Median | Median filter. Legacy, and in the obsolete table. | [NG][NG], [RM][RM] | missing | — | A median filter. |
| Dust & Scratches | Median-based cleanup. Removed in 26.0; Adobe points to Gaussian Blur. | [L][L], [RM][RM] | missing | — | As Median. |

### Perspective

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Basic 3D | Swivel, tilt and distance to image, with a specular highlight. | [PER][PER], [L][L] | partial | `swivel`, `tilt`, `perspective`, all keyable (ADR-0167, ADR-0168) | The specular highlight. |
| Bevel Alpha | Bevelled edge lighting on the alpha boundary. Obsolete; Adobe points to a Photoshop PSD. | [RM][RM] | missing | — | Alpha-edge bevel. |
| Bevel Edges | Bevelled, lit rectangular edges. | [PER][PER] | missing | — | Edge bevel. |
| Drop Shadow | Shadow colour, opacity, direction, distance and softness. | [PER][PER], [L][L] | have | `shadow{dx, dy, radius, color, opacity}` (ADR-0040); direction and distance are polar dx and dy | Shadow Only (shadow without the element) is absent. |
| Radial Shadow | A shadow cast from a point light onto the layer beneath. Obsolete. | [RM][RM] | missing | `shadow` is a parallel offset | A point-light projection. |
| Long Shadow (Film Impact) | An extended shadow trailing from flat graphics. | [MF][MF] | missing | — | A long cast shadow. #703: a runtime shader. |

### Stylize (and Film Impact Lights & Glows, Essentials)

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Alpha Glow (Legacy) | Glow round the alpha edge, with start and end colours and fade. | [L][L], [STY][STY] | partial | `shadow` with zero offset is an alpha glow (`montagent-motion` trailer recipe; ADR-0156 §4) | A two-colour, fading glow. |
| Brush Strokes | A painted look with randomly scattered strokes. | [STY][STY], [L][L] | missing | — | Seeded brush strokes. |
| Color Emboss | Emboss that keeps colour. | [STY][STY], [L][L] | missing | — | A relief kernel. |
| Emboss | Greyscale emboss. Removed in 26.0; Adobe points to Color Emboss. | [L][L], [RM][RM] | missing | — | As Color Emboss. |
| Find Edges | Edge detection. | [STY][STY], [L][L] | missing | — | A convolution kernel. |
| Mosaic (Film Impact "Mosaic FX"; old version is Legacy) | Grid or polygonal pixelation, with softness. | [STY][STY], [MF][MF], [L][L] | missing | — | A mosaic member. `montagent-prerender` names mosaic as its job, but only for footage processed outside Montagent. |
| Posterize | Reduces tonal levels per channel (2–255). | [STY][STY] | have | `posterize{levels}`, keyable (ADR-0156 §4) | — |
| Replicate (Legacy) | Tiles the clip N×N inside its frame. | [L][L] | have | Composed: N² copies of the element, each scaled and placed, written out (ADR-0148 refuses a repeat construct and asks for copies) | — |
| Roughen Edges | Erodes the alpha edge with fractal noise. | [STY][STY], [L][L] | missing | — | Seeded edge displacement. |
| Solarize | Blends negative and positive. Removed in 26.0; Adobe points to third parties. | [L][L], [RM][RM] | missing | — | A solarize curve. |
| Strobe Light | Flashes a colour or makes the layer transparent at periodic or random intervals. | [STY][STY], [L][L] | have (skill) | `montagent-motion` trailer recipe: a full-frame `rect` flash keyed on beats | Random intervals need a seed. The skill writes them from a script. |
| Texturize | Gives the clip the texture of another layer. Removed in 26.0. | [L][L], [RM][RM] | refused | ADR-0150 §4: reads another element's pixels | — |
| Threshold | Converts to black and white at a level. Removed in 26.0; Adobe points to Lumetri. | [L][L], [RM][RM] | missing | `posterize{levels: 2}` plus `saturation{0}` is close (ADR-0156, ADR-0049) | A luminance threshold at a chosen level. |
| RGB Split (Film Impact) | Splits and offsets the colour channels. | [MF][MF] | have (skill) | `montagent-motion` trailer recipe: red and cyan tinted copies blended `screen`, offset | One member on a single element. |
| Glint (Film Impact) | Brief sparkles. | [MF][MF] | missing | — | A seeded, time-reading sparkle. |
| Light Leaks (Film Impact) | Glow and occasional flare. | [MF][MF] | have | Composed: radial gradient `rect` blended `screen`/`add` (ADR-0147 names light leaks as its motive; `montagent` skill) | The "occasional" random timing is keyed by hand. |
| Echo Glow (Film Impact) | A stylised trailing glow. | [MF][MF] | missing | — | A glow that reads earlier frames. |
| Edge Glow (Film Impact) | Glow along edges. | [MF][MF] | missing | — | Edge-detected glow. |
| Wonder Glow (Film Impact) | A vibrant radiant glow. | [MF][MF] | partial | `glow{threshold, radius, intensity}` (ADR-0156) | A glow colour. ADR-0156 left `color` out on purpose: the glow takes the element's own colours. |
| Focus Glow (Film Impact) | A glow localised round focal areas. | [MF][MF] | partial | `glow` on a masked copy (ADR-0156, ADR-0152) | One member. |
| Volumetric Ray (Film Impact) | Light rays. | [MF][MF] | missing | — | A radial ray shader. |
| Stroke (Film Impact; Adobe and #703 file it under Essentials or Utility) | An outline round any element, including images and video. | [MF][MF] | partial | `stroke` paint on shapes and text (ADR-0014, ADR-0160) | A stroke round the alpha of an image or video. |
| Rounded Crop (Film Impact) | Crop with rounded corners. | [MF][MF] | have | `mask` with `shape: "rect"` and `radius` (ADR-0084, ADR-0152) | — |
| Alpha (Film Impact) | Named under Essentials, with no mechanism described. | [MF][MF] | partial | `opacity`, `mask` | Mechanism undocumented; re-check. |

### Time

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Echo | Composites earlier and later frames of the clip (trails). Obsolete in 26.0. | [TIME][TIME], [L][L] | missing | Each element paints only its current instant; `motion_blur` averages within one shutter (ADR-0155) | Frame echoes. |
| Posterize Time | Locks the clip to a lower frame rate. | [TIME][TIME], [L][L] | partial | On video, a `source_time` curve with `step` keys holds frames (ADR-0157). On graphics, `step`-eased keyframes do the same | A single frame-rate field per element. |
| Time Remapping (fixed effect) | Speed keyframes, ramps, freeze frames and reverse. | [ECP][ECP] | have | `source_time` literal for a freeze, curve for a ramp, falling segment for reverse (ADR-0157) | The speed area owns frame blending and optical flow. |

### Transform

Transform and Offset sit in this bin in 26.0. Each has one row, under Distort.

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Auto Reframe | ML subject tracking that reframes for a new aspect ratio. | [AR][AR], [L][L] | missing | `fit`, `clip`, keyed `x`/`y`/`scale` written by hand | Subject tracking. |
| Crop | Crops left, top, right and bottom by percentage, with Zoom and Edge Feather. | [TR][TR] | have | `clip` (static, frame space) and a keyable `mask` rect with `feather` (ADR-0084, ADR-0152); `fit: cover` for zoom (ADR-0015) | — |
| Edge Feather | Feathers the clip's edges. | [TR][TR] | have | A full-box `mask` rect with `feather` (ADR-0152) | — |
| Horizontal Flip | Mirrors left to right. | [TR][TR], [L][L] | have | `scale: [-1, 1]` (format.md, ADR-0012) | — |
| Vertical Flip | Mirrors top to bottom. | [TR][TR], [L][L] | have | `scale: [1, -1]` | — |
| Camera View | Views the clip from a virtual camera (longitude, latitude, roll, focal length). | [RM][RM] | refused | ADR-0167: "a camera … stay[s] refused" | — |
| Film Impact Motion bin: 3D Rotate, Spin, Move, Grow, Shrink | Preset motions applied as an effect. | [MF][MF] | have | Keyed `rotation`, `x`/`y`, `scale`, `swivel`/`tilt` (ADR-0012, ADR-0146, ADR-0167) | — |
| Camera Shake FX, Wiggle FX (Film Impact) | Randomised positional motion. | [MF][MF] | have (skill) | `montagent-motion` trailer recipe: per-frame `x`/`y` keyframes written by a generator script | A seeded shake member. Today a script bakes it to literal keys. |
| Spacer FX (Film Impact) | Named under Motion, with no mechanism described. | [MF][MF] | missing | — | Mechanism undocumented; re-check. |
| Auto Align FX (Film Impact Tools) | Positions titles or logos automatically within the layout. | [MF][MF] | partial | Nine `origin` keywords and `fit` (ADR-0013, ADR-0015); safe areas in `montagent-craft` | Automatic placement. Montagent places from literals. |
| Channel Mix FX (Film Impact Tools) | Mixes colour channels. | [MF][MF] | refused | ADR-0049 excludes per-channel colour matrices | — |
| Clone FX (Film Impact Tools) | Named under Tools, with no mechanism described. | [MF][MF] | missing | — | Mechanism undocumented; re-check. |

### Transition (effects that act as transitions on one clip)

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Block Dissolve | Random blocks dissolve the clip away. | [VF][VF] | missing | — | Seeded block pattern. |
| Gradient Wipe | A wipe driven by the luminance of another layer. Removed in 26.0. | [L][L], [RM][RM] | refused | ADR-0150 §4 | — |
| Linear Wipe | A straight wipe at any angle, with feather. | [VF][VF] | have | A keyed `mask` (path or rect) with `feather` (ADR-0150, ADR-0152, ADR-0163) | — |
| Radial Wipe | A clock-hand wipe from a centre. | [VF][VF] | partial | A keyed `path` mask whose vertices sweep (ADR-0163) | A pie mask with one angle parameter. Today it is many keyed vertices. |
| Venetian Blinds | Stripes that reveal the clip. | [VF][VF], [L][L] | missing | Several masks on one element intersect; they do not union | A union of strips. |

### Utility and Video

The Utility and Video library pages were not reached. These rows cite the 26.0 table and the
Video effects index.

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Cineon Converter | Log-to-linear conversion for Cineon and DPX footage. | [VF][VF] | missing | Colour handling is sRGB (ADR-0146 premultiplied blending) | Log and colour-space conversion. The colour area owns it. |
| Clip Name | Overlays the clip's name. | [VF][VF] | partial | A `text` element with the name written literally | Automatic text from clip metadata. |
| Timecode | Overlays a running timecode. | [VF][VF] | missing | Text is literal (ADR-0007) | A generated, running timecode. |
| Simple Text | Basic text overlay as an effect. | [VF][VF] | have | `text` element (ADR-0007) | — |
| SDR Conform | Tone-maps HDR to SDR. Obsolete in 26.0. | [L][L] | missing | — | HDR handling. The colour area owns it. |

## Video transitions

### Transition behaviour

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| A transition between two clips | Placed on an edit point and uses handles from both clips. | [TO][TO] | have | `type: "transition"` naming `from`/`to`, on two tracks (ADR-0059) | — |
| Duration and alignment (Center at Cut, Start at Cut, End at Cut) | Set in Effect Controls A/B view or by dragging. | [TD][TD] | have | The window must equal the two elements' overlap (ADR-0059); alignment is where the elements overlap | — |
| Single-sided transition (to or from nothing, e.g. fade from black) | A transition at a clip's head or tail with no neighbour. | [SS][SS] | partial | An `opacity` ramp over the backdrop (ADR-0012, ADR-0146) | A single-sided transition element. ADR-0059 requires two bridged elements. |
| Transition settings: border width and colour, Reverse, Start/End %, anti-aliasing, custom | Per-transition options in Effect Controls. | [MOD][MOD] | missing | ADR-0150 §1: "no border, no Start/End %, no Reverse, no soft edge. Each waits for a case." | The four options. They are deferred, not refused. |
| Easing a transition | Some modern transitions ease. | [MT][MT] | have | Optional `ease` on `wipe`/`slide`/`push` (ADR-0150) | No `ease` on `crossfade` (ADR-0150). |
| Default transition, and applying it to a selection | A default video transition and duration, applied with a shortcut. | [DT][DT] | missing | Every transition is written in full | A project-level default. |
| Copy and paste transitions | Paste onto other edit points. | [CPT][CPT] | have | JSON copy, then fix `from`/`to`/`start`/`end`; `validate` checks the references (ADR-0150 §3) | — |
| Replace a transition | Drag a new transition on top, keeping duration and alignment. | [RPT][RPT] | have | Change `kind` (and `direction`) in place (ADR-0145 invariant 4) | — |
| Audio crossfades with video transitions | Separate audio transitions on the audio clips. | [TO][TO] | have | `audio` field on a visual transition, and the `audio_crossfade` kind (ADR-0176) | The audio area owns the detail. |

### 3D Motion

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Cube Spin | Clips on two faces of a rotating cube. Removed in 26.0. | [VT][VT], [L][L] | partial | Keyed `swivel` with `origin` at the shared edge, on two elements (ADR-0167) | No depth sorting (refused, ADR-0167), so the two faces' stacking is set by layer and may need a swap mid-turn. |
| Flip Over | The clip flips like a card to reveal the next. Removed in 26.0. | [VT][VT], [L][L] | have (skill) | `montagent-motion` card-flip recipe: front 0→180, back −180→0 (format.md; ADR-0167) | — |

### Dissolve

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Cross Dissolve (Film Impact; the old one is "Cross Dissolve (Legacy)") | Linear opacity crossfade. | [VD][VD], [L][L] | have | `kind: "crossfade"` (ADR-0059) | — |
| Additive Dissolve (Film Impact) | Crossfade that adds the two images. | [VD][VD], [L][L] | have | Composed: an incoming element with `blend: "add"` and an opacity ramp over the outgoing (ADR-0147) | — |
| Non-Additive Dissolve | Shows the brighter pixel of the two as it fades. | [VD][VD] | missing | `lighten` blend refused for now (ADR-0147) | A max-of-two dissolve. |
| Film Dissolve (Film Impact) | Crossfade in linear light. | [VD][VD], [L][L] | missing | Blending is premultiplied sRGB; ADR-0147 refuses linear-light arithmetic for `blend` | A linear-light crossfade. ADR-0147's refusal is about `blend`, not transitions, so this is `missing`. |
| Dip to Black (Film Impact) | Fade out to black, then in. | [VD][VD], [L][L] | have | Composed: opacity ramps over the black background, or two crossfades through a black `rect` | — |
| Dip to White (Film Impact) | Fade through white. | [VD][VD], [L][L] | have | Composed: as Dip to Black, through a white `rect` | — |
| Dip to Color (Film Impact) | Fade through a chosen colour. | [L][L] | have | Composed: through a `rect` of that colour | — |
| Blur Dissolve (Film Impact) | Blurs out of one clip and into the next. | [L][L], [MT][MT] | have | Composed: keyed `blur` radius on both elements under a `crossfade` (ADR-0146) | — |
| Burn Alpha (Film Impact) | An alpha-channel burn, with fiery edges eating the image. | [L][L], [MT][MT] | missing | — | A burn-edge matte. |
| Luma Fade (Film Impact) | Reveals by luminance order. | [L][L], [MT][MT] | missing | — | A luma-ordered reveal (ADR-0152 defers luma-sourced masks). |
| Blur to Color (Film Impact, Essentials) | Blurs into a colour, then out. | [MT][MT] | have | Composed: keyed `blur` and a coloured `rect` crossfade | — |
| Flash (Film Impact; also in Lights & Blurs) | A flash frame at the cut. | [MT][MT], [L][L] | have (skill) | `montagent-motion` trailer flash recipe | — |
| Morph Cut | Face tracking and optical-flow interpolation hide a jump cut. | [MC][MC], [VD][VD] | missing | — | Optical flow and face tracking. |

### Iris

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Iris Box | A growing rectangle reveals the next clip. | [VT][VT] | have | Composed: a keyed rect `mask` on the incoming element (ADR-0150 names the keyed mask as the reveal) | — |
| Iris Cross | A growing cross shape. | [VT][VT] | have | Composed: a keyed `path` mask (ADR-0163) | — |
| Iris Diamond | A growing diamond. | [VT][VT] | have | Composed: a keyed `path` mask (ADR-0163) | — |
| Iris Round | A growing circle. | [VT][VT] | have | Composed: a keyed circle or ellipse `mask` (ADR-0040, ADR-0146) | — |

### Page Peel

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Page Peel | The page curls away. Obsolete or removed in 26.0; a modern Page Peel Impacts exists. | [VT][VT], [L][L], [MT][MT] | missing | — | A page-curl warp. |
| Page Turn | The page turns over. Obsolete in 26.0. | [VT][VT], [L][L] | missing | — | As Page Peel. |

### Slide

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Push (Film Impact; old one is "Push (Legacy)") | The incoming clip pushes the outgoing one off. | [VT][VT], [L][L] | have | `kind: "push"` with `direction` and `ease` (ADR-0150) | — |
| Slide | The incoming clip slides over the outgoing one. | [VT][VT] | have | `kind: "slide"` (ADR-0150) | — |
| Band Slide | Bands slide in from alternate sides. Obsolete in 26.0. | [VT][VT], [L][L] | partial | Composed: one copy of the incoming element per band, each with its own rect `mask` and keyed `x` | Bands as one parameter. Today it is N copies (ADR-0148). |
| Center Split | The outgoing clip splits into four and moves off. | [VT][VT] | partial | Composed: four masked copies with keyed offsets | As Band Slide. |
| Split (Film Impact) | The outgoing clip splits in two and opens. | [VT][VT], [L][L] | partial | Composed: two masked copies with keyed offsets | As Band Slide. |
| Whip (Film Impact) | A fast push with motion blur. | [VT][VT], [L][L] | have | Composed: `push` plus `motion_blur` or keyed `directional_blur` (ADR-0150, ADR-0155, ADR-0156) | — |
| Roll (Film Impact) | The outgoing shot rolls into the incoming one. | [L][L], [MT][MT] | partial | Keyed `rotation`/`swivel` under a `push` | Mechanism only described in one line; re-check. |
| Stretch (Film Impact) | Pulls and warps the footage across the screen into the next clip. | [L][L], [MT][MT] | partial | Keyed non-uniform `scale` `[sx, sy]` under a `push` (ADR-0012) | The warp. Scale is uniform across the element. |

### Wipe

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Wipe | A straight-edged wipe from a side. | [VT][VT] | have | `kind: "wipe"` with `direction` (ADR-0150) | Diagonal directions and a soft edge. A keyed `path` mask with `feather` covers both. |
| Band Wipe | Bands wipe in from alternate sides. Obsolete in 26.0. | [VT][VT], [L][L] | missing | Masks on one element intersect | A union of band masks. |
| Barn Doors | Two doors open from the centre. | [VT][VT] | have | Composed: one rect `mask` keyed wider from the centre (ADR-0084) | — |
| Checker Wipe | A checker pattern wipes in. Obsolete in 26.0. | [VT][VT], [L][L] | missing | — | A union of many masks. |
| CheckerBoard | Two checkerboards reveal in turn. Obsolete in 26.0. | [VT][VT], [L][L] | missing | — | As Checker Wipe. |
| Clock Wipe (Film Impact) | A clock-hand sweep. | [VT][VT], [L][L] | partial | A keyed `path` mask with sweeping vertices (ADR-0163) | A one-angle pie mask. |
| Inset | The next clip appears in a corner box that grows. | [VT][VT] | have | Composed: a rect `mask` keyed from a corner | — |
| Paint Splatter | Splatter shapes reveal the clip. Obsolete in 26.0. | [VT][VT], [L][L] | missing | — | A splatter matte. |
| Pinwheel | Pinwheel blades reveal the clip. Obsolete in 26.0. | [VT][VT], [L][L] | missing | — | A union of blades. |
| Radial Wipe (Film Impact) | A sweep round a corner. Also listed as removed (see Conflicts). | [VT][VT], [L][L] | partial | A keyed `path` mask (ADR-0163) | As Clock Wipe. |
| Random Blocks | Random blocks reveal the clip. Obsolete in 26.0. | [VT][VT], [L][L] | missing | — | Seeded blocks. |
| Random Wipe | Random blocks in a wipe direction. Obsolete in 26.0. | [VT][VT], [L][L] | missing | — | Seeded blocks. |
| Spiral Boxes | Boxes spiral in. Obsolete in 26.0. | [VT][VT], [L][L] | missing | — | A union of boxes. |
| Venetian Blinds | Blinds reveal the clip. Obsolete in 26.0. | [VT][VT], [L][L] | missing | — | A union of strips. |
| Wedge Wipe | A pie wedge opens. Obsolete in 26.0. | [VT][VT], [L][L] | partial | A keyed `path` mask (ADR-0163) | A one-angle wedge. |
| Zig-Zag Blocks | Blocks reveal in a zig-zag. Obsolete in 26.0. | [VT][VT], [L][L] | missing | — | A union of blocks. |
| Linear Wipe (Film Impact) | A wipe at any angle. | [L][L] | have | A keyed `path` mask, or `kind: "wipe"` for the four directions (ADR-0150, ADR-0163) | — |
| Neon Wipe (Film Impact) | A wipe with a glowing edge. | [L][L] | partial | A keyed mask, plus a stroked `path` with `glow` along the edge | A lit edge as part of the wipe. |
| Panel Wipe (Film Impact) | Panels wipe in. | [L][L], [MT][MT] | missing | — | A union of panels. |
| Plateau Wipe (Film Impact) | Named in the Wipe group, with no mechanism described. | [L][L], [MT][MT] | missing | — | Mechanism undocumented; re-check. |
| Soft Wipe (Film Impact) | A wipe with a feathered edge. | [L][L] | have | A keyed `mask` with `feather` (ADR-0152) | ADR-0150 refuses a soft edge on `kind: "wipe"` for now. |
| Star Wipe (Film Impact) | A growing star reveals the clip. | [L][L] | have | A keyed star-shaped `path` mask (ADR-0163) | — |
| Stretch Wipe (Film Impact) | A wipe that stretches the footage. | [L][L], [MT][MT] | missing | — | A warp at the edge. |

### Zoom

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Cross Zoom (Film Impact; Adobe files it under Zoom and under Lights & Blurs) | Zooms into the outgoing clip and out of the incoming one, blurred. | [VT][VT], [L][L] | partial | Keyed `scale` and a `crossfade` (ADR-0012, ADR-0059) | A radial zoom blur. |
| Zoom Blur (Film Impact) | A zoom-like radial blur across the cut. | [L][L], [MT][MT] | missing | `directional_blur` is linear (ADR-0156) | A radial zoom blur. |

### Film Impact: Lights & Blurs

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Burn Chroma | Colourful burn edges and chromatic distortion. | [L][L], [MT][MT] | missing | — | A burn matte with channel split. |
| Chroma Leak | A coloured leak across the cut. | [L][L], [MT][MT] | have | Composed: a gradient `rect` blended `screen`/`add`, keyed, over a `crossfade` (ADR-0147, ADR-0149) | — |
| Directional Blur (transition) | A smear across the cut. | [L][L] | have | Composed: keyed `directional_blur` on both, under a `crossfade` or `push` (ADR-0156) | — |
| Flare | A flare across the cut. | [L][L], [MT][MT] | have (skill) | Flare pre-rendered with `montagent-prerender`, blended `screen` | — |
| Glow | A bloom across the cut. | [L][L], [MT][MT] | have | Composed: keyed `glow` intensity under a `crossfade` (ADR-0156, ADR-0146) | — |
| Lens Blur | A lens blur across the cut. | [L][L] | partial | Keyed `blur` under a `crossfade` | A lens (bokeh) kernel. |
| Light Leak | Film burn and light leak across the cut. | [L][L], [MT][MT] | have | Composed: as Chroma Leak | — |
| Light Sweep | A light band sweeping across. | [L][L] | have | Composed: a linear gradient `rect` blended `add`, keyed across | — |
| Phosphor | Named in the group, with no mechanism described. | [L][L] | missing | — | Mechanism undocumented; re-check. |
| Radial Blur | A spin blur across the cut. | [L][L] | missing | — | A rotational blur. |
| Ray | Light rays across the cut. | [L][L] | missing | — | A ray shader. |
| Solarize | A solarize flash across the cut. | [L][L] | missing | — | A solarize curve. |
| Stripe | Named in the group, with no mechanism described. | [L][L] | missing | — | Mechanism undocumented; re-check. |

### Film Impact: Grunge & Distort

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Chaos | A random distortion that collapses or explodes into the next scene. | [L][L], [MT][MT] | missing | — | A seeded distortion. |
| Earthquake | A shake across the cut. | [L][L] | have (skill) | `montagent-motion` trailer shake recipe, under a cut or `crossfade` | — |
| Flicker | A flicker across the cut. | [L][L] | have (skill) | Per-frame `opacity` keys written by a script (`montagent-motion` trailer) | — |
| Glass | A glass-refraction distortion. | [L][L] | missing | — | A refraction warp. |
| Glitch | Digital glitches, pixelation and datamoshing. | [L][L], [MT][MT] | have (skill) | `montagent-motion` trailer "Glitch cut" recipe: offset tinted copies, jitter, flashes | Pixelation and datamosh are missing (no mosaic member, no inter-frame reads). |
| Grunge | A dirty-film texture across the cut. | [L][L] | partial | `grain` on a `rect` blended `overlay` (ADR-0156) | Scratches and dirt. Pre-rendered texture via `montagent-prerender`. |
| Kaleidoscope | A kaleidoscope reflection. | [L][L] | missing | — | A kaleidoscope warp. |
| Liquid Distortion | A liquid warp. | [L][L] | missing | — | A displacement warp. |
| TV Power | A CRT power-off squash to a line. | [L][L] | have | Composed: keyed `scale` `[sx, sy]` to a line, plus a white flash `rect` | — |
| VHS Damage | VHS tracking errors and noise. | [L][L] | partial | `grain`, RGB-split copies, jitter keys (`montagent-motion`) | Tracking-line displacement. |

### Film Impact: Transformers, Animation, Text and Smart Tools

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| 3D Roll | A 3D roll into the next clip. | [L][L], [MT][MT] | have | Keyed `tilt` with `perspective` on both elements (ADR-0167) | — |
| 3D Spin | A 3D spin. | [L][L], [MT][MT] | have | Keyed `swivel` (ADR-0167) | — |
| 3D Spinback | A 3D spin that springs back. | [MT][MT] | have | Keyed `swivel` with an overshoot bezier (ADR-0167, ADR-0146) | — |
| Film Roll | Frames roll past like film. | [L][L], [MT][MT] | partial | Keyed `y` under a `push` | Frame borders and sprocket texture, and a motion smear (`motion_blur` covers that). |
| Mirror (Impacts) | Mirrored copies across the cut. | [MT][MT] | partial | Flipped copies with `scale: [-1, 1]` | Mechanism only described in one line; re-check. |
| Page Peel Impacts | A modern page curl. | [MT][MT] | missing | — | A page-curl warp. |
| Slice | Named in Transformers, with no mechanism described. | [MT][MT] | partial | Masked copies with keyed offsets | Mechanism undocumented; re-check. |
| Block Motion, Flip Motion, Pull Motion, Spin Motion, Travel Motion | Preset transform animations into the next clip. | [L][L], [MT][MT] | have | Keyed `x`/`y`/`scale`/`rotation`/`swivel` on the two elements (ADR-0012, ADR-0146, ADR-0167) | — |
| Spring Motion | A spring-physics move into the next clip. | [L][L] | refused | ADR-0146 refuses spring easing (a cubic bezier with overshoot is the documented substitute) | — |
| Text Animator | Text animates in letter by letter or word by word. | [L][L] | have | `units`, `unit`, stagger across one text element's units (ADR-0148) | — |
| Typewriter | Text types on. | [L][L], [MT][MT] | have (skill) | `montagent-motion` `type_on.py` | — |
| Motion Camera | A camera move across the cut. | [L][L] | refused | ADR-0167: a camera stays refused | — |
| Motion Tween | Tweens between the two clips' transforms. | [L][L] | partial | Keyframes between literal transforms (ADR-0012) | Automatic matching of the two clips' transforms. |
| Shape Flow | Named in Smart Tools, with no mechanism described. | [L][L] | missing | — | Mechanism undocumented; re-check. |

### Immersive Video transitions

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| VR Chroma Leaks | A chroma leak, seamless across the sphere. | [IMT][IMT], [VT][VT] | missing | — | Spherical model. |
| VR Gradient Wipe | A gradient-driven wipe across the sphere. | [IMT][IMT], [VT][VT] | missing | — | Spherical model. |
| VR Iris Wipe | An iris wipe on the sphere. | [IMT][IMT], [VT][VT] | missing | — | Spherical model. |
| VR Light Leaks | Light leaks across the sphere. | [IMT][IMT], [VT][VT] | missing | — | Spherical model. |
| VR Light Rays | Light rays across the sphere. | [IMT][IMT], [VT][VT] | missing | — | Spherical model. |
| VR Mobius Zoom | A Möbius zoom on the sphere. | [IMT][IMT], [VT][VT] | missing | — | Spherical model. |
| VR Random Blocks | Random blocks on the sphere. | [IMT][IMT], [VT][VT] | missing | — | Spherical model. |
| VR Spherical Blur | A blur transition on the sphere. | [IMT][IMT], [VT][VT] | missing | — | Spherical model. |

## Conflicts in Adobe's pages

Recorded so the assembly does not resolve them silently:

- **Lens Distortion, Compound Blur, Noise, Radial Wipe and Venetian Blinds** each appear both as
  a current or Obsolete entry and in the 26.0 "Removed (will not render)" list. The likeliest
  reading is that the removed entry is the older implementation of the same name. Rows here
  follow the current entry.
- **Camera Blur** is Legacy in the 26.0 table, but Obsolete (macOS) on the older removal page.
- **Lens Flare, Lightning and Ramp** are still on the Generate page, while the 26.0 table marks
  them Obsolete, Obsolete and Legacy. #703 found the same.
- **Cross Zoom** is in the Zoom category on the transitions list and in Lights & Blurs in the
  26.0 table.
- **Channel Blur, Gradient and Noise** came back as Film Impact effects in 26.2.2 (release
  notes), after the older versions were listed obsolete.

## Notable gaps

1. **No adjustment layer and no effect on a group.** Premiere's main way to grade or stylise
   many clips at once has nothing to apply to. Group compositing is in the map's fog
   (ADR-0167 §10), behind nests (ADR-0166, #499).
2. **A mask scopes the whole element, never one effect, and nothing tracks it.** Regional
   blurs, face blurs and spot grades all take a second masked copy and hand-keyed motion.
3. **Colour beyond four scalars.** Levels, hue rotation, white balance, exposure, gamma,
   invert, tonal-range wheels and hue-selective correction are missing. Curves, LUTs, channel
   mixing and duotone are refused (ADR-0049, ADR-0040). So Lumetri, Premiere's central colour
   tool, is only partly covered.
4. **No distortion or pixel-stylise filter.** Twirl, Wave Warp, Spherize, Lens Distortion,
   Turbulent Displace, Offset, Mosaic, Find Edges, Emboss, Sharpen and Unsharp Mask are all
   missing as members. Pre-rendering outside the project is the only route. #703 maps most of
   them to one Skia call or one small runtime shader under ADR-0156 §2's entry test.
5. **No analysis-driven effects.** Warp Stabilizer, Rolling Shutter Repair, Auto Reframe, Morph
   Cut and the Auto colour effects all need motion or pixel analysis. Montagent has no
   analysis step.
6. **Transitions beyond four kinds rely on masks, and masks only intersect.** Every
   multi-shape pattern wipe needs a union of shapes (checker, blinds, bands, random blocks,
   pinwheel, spiral). Border, Reverse and Start/End % are deferred (ADR-0150). There is no
   single-sided transition element (ADR-0059).
7. **Keying stops at one chroma key.** Luma Key, Ultra Key's matte cleanup (choke, pedestal)
   and Alpha Adjust's alpha options are missing. Track Matte Key and every effect that reads
   another layer are refused (ADR-0150 §4).
8. **No immersive (360°/VR) model.** 11 effects and 8 transitions are all missing for the same
   reason.
9. **Effect controls conveniences.** There is no visual-effect bypass flag (audio has
   `enabled`), no preset store or anchor-to-in/out retiming, and no effect-controls markers.
   Source-clip effects have no home, because ADR-0002 rules out an asset table.

[L]: https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/list-of-effects-and-transitions.html
[RO]: https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/effects-and-transitions-reorganization.html
[RM]: https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/effects-and-transitions-removed-from-adobe-premiere.html
[MF]: https://helpx.adobe.com/premiere/desktop/add-video-effects/types-of-effects/effects.html
[MT]: https://helpx.adobe.com/premiere/desktop/add-video-effects/types-of-effects/transitions.html
[VF]: https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/video-effects.html
[ADJ]: https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/adjust-effects.html
[BS]: https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/blur-and-sharpen-effects.html
[CC]: https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/available-color-correction-effects.html
[LUT]: https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/add-look-up-tables.html
[LV]: https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/create-vignettes-using-lumetri-color.html
[DIS]: https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/distort-effects.html
[GEN]: https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/generate-effects.html
[IC]: https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/image-control-effects.html
[KEY]: https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/keying-effects.html
[UK]: https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/ultra-key-effect-parameters.html
[NG]: https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/noise-and-grain-effects.html
[PER]: https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/perspective-effects.html
[STY]: https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/stylize-effects.html
[TIME]: https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/time-effects.html
[TR]: https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/transform-effects.html
[IMF]: https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/immersive-video-effects.html
[IMT]: https://helpx.adobe.com/premiere/desktop/edit-projects/edit-vr-content/immerse-video-effects-and-transitions.html
[VT]: https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/list-of-video-transitions.html
[VD]: https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/list-of-video-dissolve-transitions.html
[TO]: https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-transitions/transitions-overview.html
[SS]: https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-transitions/apply-single-sided-transitions.html
[TD]: https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-transitions/change-transition-duration-using-the-effect-controls-panel.html
[CPT]: https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-transitions/copy-and-paste-transitions.html
[RPT]: https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-transitions/replace-transitions.html
[DT]: https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-transitions/set-and-apply-default-transitions.html
[MC]: https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-transitions/morph-cut-transition-options.html
[MOD]: https://helpx.adobe.com/be_en/premiere-pro/using/modifying-customizing-transitions.html
[ECP]: https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-effects/about-effect-controls-panel.html
[ECV]: https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-effects/view-effects-in-the-effect-controls-panel.html
[TYP]: https://helpx.adobe.com/premiere/desktop/add-video-effects/types-of-effects/types-of-effects.html
[APR]: https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-effects/apply-effect-presets.html
[CPR]: https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-effects/create-effect-presets.html
[CPE]: https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-effects/copy-and-paste-clip-effects.html
[REM]: https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-effects/remove-effects-from-a-clip.html
[MCE]: https://helpx.adobe.com/premiere-pro/using/master-clip-effects.html
[AR]: https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/auto-reframe-overview.html
[WS]: https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/stabilize-shaky-footage-using-warp-stabilizer.html
[WSS]: https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/warp-stabilizer-settings.html
[MSK]: https://helpx.adobe.com/premiere/desktop/add-video-effects/create-masks-and-composites/create-masks-using-shapes.html
[MTK]: https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/mask-tracking-tools.html
[KF]: https://helpx.adobe.com/premiere/desktop/add-video-effects/control-effects-and-transitions-using-keyframes/about-keyframes.html
[KFI]: https://helpx.adobe.com/premiere/desktop/add-video-effects/control-effects-and-transitions-using-keyframes/change-keyframe-interpolation-method.html
[ECM]: https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/create-markers-in-the-effect-controls-panel.html
[PLG]: https://helpx.adobe.com/premiere/desktop/use-premiere-with-other-apps/plug-ins.html
[FG]: https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-effects/find-and-group-effects.html
[ACC]: https://helpx.adobe.com/premiere/desktop/add-video-effects/types-of-effects/list-of-accelerated-effects.html
[RN]: https://helpx.adobe.com/premiere/desktop/whats-new/release-notes.html
