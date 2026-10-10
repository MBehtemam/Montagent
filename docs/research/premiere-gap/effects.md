# Premiere gap: video effects, video transitions and effect controls

Research for [#885](https://github.com/MBehtemam/Montagent/issues/885), on the map
[#882](https://github.com/MBehtemam/Montagent/issues/882). Re-sourced against the full Adobe pages
for [#911](https://github.com/MBehtemam/Montagent/issues/911).

- **Date:** 2026-10-10. Re-sourced 2026-10-10 (#911).
- **Premiere side:** Adobe Premiere 26.x (26.0 through 26.5.2), from Adobe's own helpx.adobe.com
  user guide only. Every row now cites the fetched page it comes from, marked **(F)**. The pages
  are the 81 census pages assigned to this area (#883) and the three What's-new pages, all read
  in full, plus a few pages from other chapters found by searching the whole 807-page corpus
  (keyframes, masks, colour, plug-ins, troubleshooting). The anchor is still Adobe's table
  [List of effects and transitions][L], which sorts every 26.0 effect and transition into its
  bin, Legacy, Obsolete or "Removed in version 26.0 (will not render)".
- **Montagent side:** tag `v0.2.0` (`51aa4b5` = `origin/main`): `CONTEXT.md`,
  `schema/montagent.schema.json`, `crates/montagent-core/docs/format.md`, `docs/adr/`, the ten
  MCP tools in `crates/montagent/src/mcp.rs` (`validate`, `create_project`, `query`, `frame`,
  `render`, `verify`, `preview`, `measure`, `shift`, `compare`), `skills/`, and
  `docs/research/skia-reach/named-effects-candidates.md` (#703).
- **What v0.2.0 has to compare against:**
  - **12 `effects` members:** `blur`, `shadow`, `mask` (ADR-0040), `tint`, `saturation`,
    `brightness`, `contrast` (ADR-0049), `chroma` (ADR-0088), and `grain`, `glow`,
    `posterize`, `directional_blur` (ADR-0156). They apply in list order, and every numeric
    and colour parameter is animatable (ADR-0146). No visual member has an `enabled` key.
  - **Element fields:** `blend` with `normal`, `multiply`, `screen`, `overlay` and `add`
    (ADR-0147). `motion_blur` (ADR-0155). `swivel`, `tilt` and `perspective` (ADR-0167).
    `clip` and `fit`. `scale` (a negative component flips). `opacity`. `speed`.
    `source_time` (ADR-0157). `overrun: hold | loop` past the source's end. Gradient paint
    (ADR-0149).
  - **Transition kinds:** `crossfade`, `wipe`, `slide`, `push` (ADR-0059, ADR-0150), and
    `audio_crossfade` (ADR-0176).

## Counts

| Status | Before #911 | After #911 |
|---|---|---|
| have | 69 | 104 |
| have (skill) | 15 | 16 |
| partial | 50 | 72 |
| missing | 128 | 154 |
| refused | 17 | 21 |
| **total** | **279** | **367** |

One row is marked *unconfirmed* (no fetched page names it) and is counted under its status.

## How to read this

- **One row per effect and per transition**, as the ticket asks. A "(Legacy)" entry in Adobe's
  table that shares a name with a modern effect now has its own row, under
  [Legacy versions](#legacy-versions-of-renamed-effects-and-transitions). Rows that used to
  group several effects (the four removed colour correctors, the Film Impact motion bin, the
  Animation transitions) are split, one effect per row.
- **Sources.** Every link in the URL column is a page fetched in full, marked **(F)**.
  - **‡** means the page *names* the effect (usually only in Adobe's 26.0 table or the removal
    table) but does not describe it. The mechanism in that row is the earlier reading, kept but
    unconfirmed.
  - **unconfirmed** in the URL column means no fetched page names the feature. The row is kept,
    not dropped.
- **The status follows the map's vocabulary**, read by mechanism (ADR-0145):
  - `have`: an equivalent mechanism ships in v0.2.0, either as a named member or as elements
    composed in a way the format or skill documents. "Composed" names the parts.
  - `have (skill)`: a bundled skill supplies the mechanism (a recipe or a script), not a
    format field.
  - `partial`: some of the mechanism ships. The last column says exactly what is missing.
  - `missing`: nothing equivalent ships, and no ADR refuses it.
  - `refused`: an ADR, or ADR-0145's four invariants, rules it out. The ADR is cited.
- ***GUI-only*** marks a feature that serves a hand on a mouse or an eye on a panel. Montagent
  is "edited by an AI agent rather than dragged around in a GUI" (`CONTEXT.md`) and its verbs
  replace panels (ADR-0011). No ADR refuses a GUI, so these rows are `missing`, as in the
  sibling area files.
- **The pre-render route is not counted as having a filter.** ADR-0156 §7 and the
  `montagent-prerender` skill bring a look drawn in code into the project as footage. For a
  **generator**, a look with no source pixels (lens flare, lightning), that is the same
  mechanism, so the row is `have (skill)`. For a **filter** on an element's own pixels (Twirl,
  Find Edges, Mosaic), pre-rendering means processing the footage outside Montagent. The filter
  is not a parameter in the project, and it cannot be keyed. Those rows stay `missing`, and the
  last column names the workaround.
- **Transitions composed from elements.** A Montagent transition element has four visual
  kinds. Many Premiere transitions are a keyed mask, keyed transform or keyed effect on the
  two bridged elements, laid over a `crossfade` or a cut. ADR-0150 names a keyed `mask` as the
  reveal mechanism. These rows are `have` when the composition is exact, and `partial` when an
  ingredient is missing.
- **Parameters.** Adobe's 26.x library pages give one line per effect and few controls. Where a
  parameter decides the status, it is quoted from a fetched page. Otherwise the row compares the
  mechanism the page's description names.

### What changed in #911

- **Re-sourced:** all 279 earlier rows now cite fetched pages. Four citations pointed at pages
  outside the 26.x guide or at the wrong page, and were replaced: master-clip effects (now
  [About source clips][SRC]), transition settings (now [Change transition settings][CTS]),
  Convolution Kernel (cited the Adjust page, which does not list it; now the removal tables),
  and the Generate page for eight generators it no longer lists (now the removal tables, since
  all eight were removed in 26.0).
- **Mechanisms corrected (51 rows).** The main ones: Compound Blur reads another layer
  (confirmed, the hedge is gone); Motion Camera is a handheld shake, not a camera; Soft Wipe is
  a lens-blur transition, not a feathered wipe; Block Motion breaks the frame into blocks;
  Logo Cutout is the renamed Film Impact Alpha FX; Clone duplicates an element with offset
  timing; the Gradient Wipe *effect* is current, and only the Gradient Wipe *transition* was
  removed; Channel Mix, Rounded Crop, Clone, Auto Align and Stroke moved bins; Lens Flare and
  Twirl were rebuilt in 26.5. The full list is in
  [Corrections and status changes](#corrections-and-status-changes).
- **Statuses changed (15 rows),** each because the fetched page changed the mechanism. Listed
  in the same section.
- **Rows added (89):** 53 effects, transitions and controls the earlier file lacked (Fold
  Motion, Pop Motion, Frame, Louver, Wave, Shape Dissolve, the Mosaic transition, Metadata &
  Timecode Burn-in, Ghosting, Channel Mixer, Global FX Mute, Surprise Me, 32-bit and YUV
  effects, adjustment-layer resizing, Auto Reframe Sequence, the synthetic clips, transition
  handles and more), the 19 "(Legacy)" duplicates, and 17 rows from splitting grouped rows.
- **One merge.** The earlier "Alpha (Film Impact)" row and the "Logo Cutout" row were the same
  effect: Adobe's table gives Logo Cutout's old name as "FI: Alpha FX". They are now one row,
  Logo Cutout. Nothing else was dropped.

## Effect controls: how effects behave

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Fixed (intrinsic) effects: Motion, Opacity, Time Remapping | Built into every clip and always present. They cannot be removed or reordered. Motion includes anti-flicker. | [TYP][TYP], [ECP][ECP], [ECV][ECV], [REM][REM] (F) | have | Element transform fields `x`/`y`/`scale`/`rotation`/`origin` (ADR-0012), `opacity`, `source_time` (ADR-0157) | Anti-flicker filtering (an interlace concern, owned by the export area). |
| Motion effect: position, scale and rotation by number or on screen | Set Position, Scale and Rotation in Effect Controls, or drag and resize in the Program Monitor. | [MO][MO] (F) | have | `x`, `y`, `scale`, `rotation` written as numbers (ADR-0012) | Direct manipulation in a monitor is *GUI-only*. |
| Vector Motion | Transforms a vector graphic (position, scale, rotation, anchor) without rasterising it, so scaling stays sharp. | [VMO][VMO] (F) | have | Shape, path and text elements are vector and are drawn under their own transform on Skia (ADR-0144, ADR-0154) | — |
| Standard effect stack order | Standard effects render top-down in the order listed in Effect Controls. They are reordered by dragging. | [ECV][ECV], [TYP][TYP] (F) | have | `effects` is an ordered list and order is significant (ADR-0040); reordering is moving a list item | — |
| Render order: standard effects before fixed effects | "Premiere processes Fixed effects after Standard effects." | [TYP][TYP] (F) | have | A fixed, documented order: `clip`, then `effects` and mask in list order, then projection, then `scale`/`rotation`/position, then `opacity`, then `blend` (format.md; ADR-0147, ADR-0155 §4, ADR-0167) | — |
| Moving a geometry step into the stack (Transform in place of Motion, Alpha Adjust in place of Opacity) | Adobe's workaround for render order: the equivalent standard effects sit anywhere in the stack. | [TYP][TYP], [KEY][KEY] (F) | partial | Opacity and transform are element fields at fixed positions in the order | No way to put a transform or opacity step *before* an effect (e.g. blur after scaling). The only route is nesting, which v0.2.0 lacks (ADR-0166; `montagent` skill "can't do yet", #499). |
| Effect on/off toggle | "Use the effects toggle in Effect Controls panel to turn applied effects on or off." | [PERF][PERF] (F) | missing | Audio effects carry `enabled` (ADR-0179). Visual `effects` members have no such key | A visual effect can only be disabled by deleting it, which loses its parameters. |
| Global FX Mute | A Program Monitor button that mutes every applied effect at once, to ease playback; effects come back for export. | [PERF][PERF] (F) | missing | `frame`, `preview` and `render` always apply every effect | A render switch that skips effects. |
| Keyframing effect parameters | Stopwatch per property; keyframes in Effect Controls and the Timeline. | [KF][KF], [ABT][ABT], [TYP][TYP] (F) | have | Every numeric and colour effect parameter takes a keyframe list (ADR-0146) | — |
| Keyframe interpolation on effect parameters | Linear, Bezier, Auto Bezier, Continuous Bezier, Hold, Ease In, Ease Out. | [KFI][KFI] (F) | have | `Ease` vocabulary: named eases, cubic-bezier arrays, `step` = hold (schema `Ease`; ADR-0038) | Velocity-graph editing is a UI concern; the bezier is written as numbers. Spring easing is refused (ADR-0146). |
| Effect Controls panel views: timeline view, Value and Velocity graphs, Pin to Clip, filter properties, looped audio preview | Panel options for seeing and scrubbing the selected clip's effects. | [ECP][ECP] (F) | missing | `query` lists an element's effects and keyframes; `measure` reads times (ADR-0011) | *GUI-only.* |
| Adjusting a control: scrub, type, slider, angle dial | Drag the underlined value, type a number, or use a slider or angle control. | [ADJC][ADJC] (F) | have | Every value is a literal number in the JSON (ADR-0145 invariant 4) | The scrubbing gestures are *GUI-only*. |
| Eyedropper and colour picker for colour controls | Sample a colour anywhere on screen (1 px, or 5 × 5 with Ctrl/Cmd), or pick in the Adobe Color Picker. | [ADJC][ADJC], [UKA][UKA] (F) | missing | Colours are literal hex strings (ADR-0088: colour stays literal) | Sampling a colour from pixels. An agent can read a rendered `frame` and write the hex itself. |
| Reset an effect | Reset sets un-keyed properties to defaults; a keyed property is reset at the current time only, adding a keyframe if needed. | [ADJC][ADJC] (F) | have | Rewrite the literal values, or the keyframe at that time (ADR-0145 invariant 4) | A reset command. |
| Effect masks (a mask limits one effect to a region) | Rectangle, Ellipse, Polygon and Pen masks are drawn inside an effect, and "the effect is constrained within the masked area". | [MSK][MSK], [VM][VM] (F) | partial | `mask` is an `effects` member (ADR-0068), with rect/circle/ellipse/path shapes (ADR-0084, ADR-0163), `invert` and `feather` (ADR-0152) | A mask cuts the **whole element's** alpha. It cannot scope a single effect. A regional blur needs a second, masked copy of the element. Mask expansion and per-mask opacity are absent. |
| Mask tracking | Track a mask forward, backward, both ways, or one frame, by Position, Scale, Rotation or Perspective (computer vision, not AI). | [MTK][MTK] (F) | missing | Mask geometry is keyable by hand (ADR-0146) | No tracker. An agent would have to measure motion and write keyframes itself. |
| Effect presets: built-in and custom | Save one or more effects with their keyframes as a preset, in nested preset bins. Premiere ships presets (e.g. Fast Blur In). Applying one replaces fixed effects and appends standard effects; drop rules decide which selected and linked clips receive it, and audio or video effects are ignored on the wrong track type. | [EP][EP], [CPR][CPR], [APR][APR] (F) | partial | Skill recipes act as named, documented presets (`montagent-motion` trailer/logo-loop references, `montagent-footage`); an agent copies the JSON in | No preset store in the format and no apply-by-name. Indirection is avoided on purpose (ADR-0002's no-asset-table reasoning). |
| Preset keyframe retiming: Scale, Anchor to In Point, Anchor to Out Point | Chosen when saving a preset: scale keyframes to the target clip's length, or keep their distance from its In or Out point. | [CPR][CPR] (F) | missing | Keyframe times are absolute literals | Retiming pasted keyframes to a new element's length or ends. |
| Apply an effect | Drag an effect from the Effects panel onto a clip, then customise it in Effect Controls. | [AE][AE] (F) | have | Add a member to the element's `effects` list (ADR-0040) | — |
| Copy and paste effects; Paste Attributes | Copy a clip's effects and paste them, or choose effects and keyframes in Paste Attributes. Intrinsic effects are replaced, existing effects kept, new ones added; keyframes keep their positions. | [CPE][CPE] (F) | have | The `effects` list is literal JSON; copying it is an exact-string edit (ADR-0145 invariant 4) | — |
| Remove effects; Remove Attributes | Remove selected effects, or remove chosen effect types and reset intrinsic effects to defaults. Fixed effects cannot be removed. | [REM][REM] (F) | have | Delete the member, or the `effects` key | — |
| FX badges | A badge on each timeline clip shows whether effects are applied (underlined for a source effect), lists them on hover, and opens Effect Controls. | [FX][FX] (F) | missing | `query` lists each element's effects (ADR-0011) | *GUI-only.* |
| Source clip effects | A video effect applied to the source clip in the Project panel is inherited by every sequence clip made from it. Not for Motion, Opacity, Speed, Warp Stabilizer or Rolling Shutter Repair, nor for audio. | [SRC][SRC], [ASRC][ASRC], [FX][FX] (F) | refused | ADR-0002: no asset table, and each element names its file inline | There is no per-file record for a shared effect to live on. Inferred from ADR-0002; no ADR names source-clip effects. |
| Source-clip effects shown on Project panel thumbnails | Toggle "Thumbnails show effects applied" per source clip. | [THB][THB] (F) | refused | ADR-0002 (no media catalogue, so no thumbnails to decorate) | — |
| Adjustment layers | A transparent clip on a higher track. "Any effect applied to an adjustment layer affects all clips on tracks below it." | [AL][AL], [MCO][MCO] (F) | missing | No element filters the composite beneath it. `blend` mixes one element's own pixels with the backdrop (ADR-0147) | An effect over the composite below. ADR-0167 §10 calls this "group compositing" and leaves it in the map's fog, gated on nests. |
| Resize an adjustment layer | Scale and position the adjustment layer to limit its effect to part of the frame; trim it to limit when. | [RAL][RAL] (F) | missing | — | As adjustment layers. |
| Effects on a nested sequence | "To apply an effect to multiple clips at once, use a nested sequence." Warp Stabilizer also needs a nest when the clip and sequence sizes differ. | [TYP][TYP], [WS][WS] (F) | missing | ADR-0166 records that the reference class nests with a transform; no `nest` is admitted in v0.2.0 (`montagent` skill: "Parenting, groups and cameras", #499) | A container, and group compositing for its effects. |
| Clip-based video effects | Video effects (fixed and standard) apply to individual clips; only audio effects can also apply to tracks. | [TYP][TYP] (F) | have | Effects attach to elements, not tracks (ADR-0040) | — |
| Third-party effect and transition plug-ins | After Effects-compatible and other plug-ins appear in the Effects panel. A project with missing plug-ins renders without them, marked offline. Third-party transitions can draw custom UI. | [TYP][TYP], [PLG][PLG], [RN][RN] (F) | refused | ADR-0040 (vocabulary is "never a plugin architecture"), ADR-0017 (closed schema), ADR-0156 §8 (an open shader or script stays refused) | — |
| Effect Manager | Window > Effects > Manage Video Effects lists installed effects to enable or disable at next start, and names the effect suspected after a crash. | [MGR][MGR] (F) | refused | ADR-0040, ADR-0017: no plug-ins to manage; the vocabulary is closed | — |
| Finding and grouping effects | Search box, nested bins, custom bins, and filter buttons for Accelerated, 32-bit Color and YUV effects. | [FG][FG], [TYP][TYP] (F) | have | The closed vocabulary is published in the schema, each member described (ADR-0040's discoverability test) | Custom bins are *GUI-only*. |
| Markers in the Effect Controls time ruler | Clip and sequence markers shown and added in Effect Controls to place effects and cue points. | [ECM][ECM] (F) | missing | No marker key in the schema. `measure` and `query --at` read times instead | Markers as authored data. |
| Legacy, Obsolete and Removed effects | Legacy effects stay available but labelled; Obsolete ones are hidden but render in old projects; Removed ones "will no longer render". Obsolete effects already applied can still be copied and pasted. | [RO][RO], [L][L], [RM][RM] (F) | have | A different mechanism: a retired key is an unknown-key error that names the retirement, and the render is blocked (ADR-0016, ADR-0017) | — |
| GPU-accelerated effects | Accelerated effects play in real time on the GPU; Project Settings choose Mercury Playback Engine GPU or Software Only. All modern (Film Impact) effects are GPU-accelerated. | [ACC][ACC], [TYP][TYP], [MF][MF] (F) | missing | Rendering is CPU Skia on K painters (ADR-0144), byte-identical by design | GPU rendering. A performance item; the workflow area owns it. |
| 32-bit color (high-bit-depth) effects | Ten effects process at high bit depth when the sequence uses Maximum Bit Depth; one 8-bit effect drops the whole chain to 8-bit. | [B32][B32], [TYP][TYP] (F) | missing | Frames are decoded and composited as 8-bit RGBA (import area, #891) | A high-bit-depth pipeline. |
| YUV effects | Fourteen effects work on YUV values directly, without converting to RGB. | [YUV][YUV], [TYP][TYP] (F) | missing | The pipeline is RGB throughout | Not meaningful without a YUV path. Listed for completeness. |
| Surprise Me | A button on modern effects and transitions that randomises the parameters into a new look; a Seed property records it. | [MF][MF], [RN][RN] (F) | missing | `grain` takes a literal `seed` (ADR-0156 §3); no other member is seeded | A randomise operation. An agent can write varied values itself. |
| Film Impact Dashboard | Window > Extensions > Film Impact Dashboard browses the modern effects and applies them, warning when two clips have no overlapping media. | [RN][RN] (F) | missing | `validate` reports a transition whose window does not equal the overlap (ADR-0059) | *GUI-only.* |

## Synthetic clips and commonly used effects

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Black Video | A synthetic opaque black clip, sized to the sequence. Empty track areas also show black. | [BV][BV] (F) | have | The canvas background, or a full-frame `rect` with a black `fill` | — |
| Color Matte | A synthetic solid clip in a picked colour. | [CM][CM] (F) | have | A full-frame `rect` with `fill` | — |
| Transparent Video | An empty synthetic clip that hosts generator effects (Timecode, Lightning) without hiding the tracks below. | [TV][TV], [LIT][LIT] (F) | have | A generator is its own element (shape, text, or pre-rendered footage), so no empty host is needed | — |
| Bars and Tone | A synthetic colour-bars clip with a 1 kHz tone at −12 dB, gain adjustable. | [BT][BT] (F) | partial | Bars composed from `rect` elements | A tone generator. Audio comes only from files. |
| Universal Counting Leader | An 11-second leader: circular one-second wipe, lines, target circles, countdown numerals, chosen colours, and cue blips (two-pop). | [CL][CL] (F) | partial | Composed: `ellipse` and `path` elements, a stroke sweep by `trim_end` (ADR-0160), `text` numerals | The cue blips need an audio file. No one-step leader. |
| Fade in from black | A Dissolve transition at the clip's head, or Opacity keyframes from 0 to 100%. | [FI][FI] (F) | have | Keyed `opacity` (ADR-0012, ADR-0146) | — |
| Lumetri vignette | Amount (negative darkens, positive lightens), Midpoint, Roundness and Feather; applied after all other Lumetri corrections. | [LV][LV], [CC][CC] (F) | partial | Composed: a radial-gradient `rect` blended `multiply` to darken or `screen` to lighten, with stops for midpoint and feather (ADR-0147, ADR-0149) | Roundness toward a rounded rectangle. The radial gradient is circular or elliptical only. |
| Mask-based vignette | Brightness & Contrast (or Lumetri) limited by an ellipse, rectangle or pen mask, inverted, and tracked with the subject. | [VM][VM] (F) | partial | A darkened copy of the element with an inverted, feathered `mask` (ADR-0152) | The mask scopes the whole copy, not one effect, and nothing tracks it. |

## Video effects

### Adjust

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Extract | Turns the clip greyscale: pixels below the Black Input Level or above the White Input Level go black, those between grey or white. | [ADJ][ADJ], [L][L] (F) | missing | `saturation{0}` plus `contrast` approximate it (ADR-0049) | A luminance-range threshold. |
| Levels | Combines Color Balance, Gamma Correction, Brightness and Contrast, and Invert in one effect. | [ADJ][ADJ], [L][L] (F) | partial | `brightness`, `contrast` (ADR-0049) | Gamma, invert and per-channel balance. Check against ADR-0049's four-clause rule before any proposal. |
| Lighting Effects | Up to five lights: type, direction, intensity, colour, centre, spread, and a Bump Layer for texture. | [ADJ][ADJ], [L][L], [RN][RN] (F) | missing | A radial gradient rect blended `add`/`screen` gives a flat light (ADR-0147, ADR-0149) | Lights with direction, falloff and surface relief. A bump map from another track would be refused (ADR-0150). |
| ProcAmp | Brightness, contrast, hue and saturation in one effect, with a split %. 32-bit. | [ADJ][ADJ], [L][L], [B32][B32] (F) | partial | `brightness`, `contrast`, `saturation` (ADR-0049) | Hue rotation. The split-screen compare is *GUI-only*. |

### Blur & Sharpen

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Gaussian Blur (modern; was "FI: Blur FX") | Blurs and softens, horizontally, vertically or both; a Repeat Edge setting. 32-bit and YUV. | [BS][BS], [L][L], [MF][MF], [RN][RN] (F) | partial | `blur{radius}`, keyable (ADR-0040, ADR-0146) | Horizontal-only or vertical-only blur, and repeat edge pixels. A one-axis blur is approximated by `directional_blur` at 0° or 90° (ADR-0156). |
| Directional Blur (modern; was "FI: Blur FX") | Gives the illusion of motion; the blur is applied equally either side of each pixel. | [BS][BS], [L][L] (F) | have | `directional_blur{angle, length}` (ADR-0156 §4) | — |
| Bokeh Blur (was "FI: Bokeh Blur FX") | Softens the background with bokeh in circles, hexagons or custom shapes. | [L][L], [MF][MF] (F) | missing | — | Shaped bokeh kernel and highlight bloom. |
| Compound Blur (was "FI: Compound Blur FX") | "A multi-layer blur that uses any layer from your timeline (like a matte or another clip) as a blur source." | [L][L], [MF][MF] (F) | refused | ADR-0150 §4: a matte or map taken from another element is refused | — |
| Focus Blur (was "FI: Focus Blur FX") | Blur that leaves a focal region sharp. ‡ | [L][L] (F) | partial | `blur` on a copy plus an inverted, feathered `mask` gives a focal area (ADR-0152) | One member. Today it takes two elements. |
| Channel Blur (modern, 26.2.2) | Blurs each colour channel, and alpha, independently, in RGB, HSV or YUV, with a Gaussian kernel. Replaces the legacy Channel Blur. | [MF][MF], [WN][WN], [RN][RN] (F) | missing | — | Per-channel blur. |
| Reduce Interlace Flicker | Reduces high vertical frequencies for interlaced media. Also in the 26.0 removed list (see Conflicts). | [BS][BS], [L][L] (F) | missing | Output is progressive H.264 (ADR-0143) | Not applicable to progressive output. Listed for completeness. |
| Sharpen | Increases contrast where colour changes, to sharpen soft footage. | [BS][BS], [L][L] (F) | missing | — | A sharpen member. #703: Skia `matrix_convolution`. |
| Unsharp Mask | Increases contrast at edges, with Radius and Threshold. | [BS][BS], [L][L] (F) | missing | — | As Sharpen, plus a threshold. |
| Camera Blur (Legacy; Windows only) | Simulates leaving the camera's focal range, keyed for focus pulls. | [BS][BS], [L][L], [RM][RM] (F) | partial | `blur` (ADR-0040) | A lens kernel. `blur` is Gaussian. |

### Color Correction

The library page for this bin is an introduction only. The effects are described on the colour
chapter's [Available color correction effects][CC] page.

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| ASC CDL | Slope, offset and power for each of R, G, B, plus one saturation. 32-bit. | [CC][CC], [L][L] (F) | missing | `saturation` (ADR-0049) | Per-channel slope, offset and power. Test against ADR-0049's rule. |
| Brightness & Contrast | Adjusts all pixel values at once; 0.0 means no change. | [CC][CC], [L][L] (F) | have | `brightness`, `contrast` (ADR-0049) | — |
| Lumetri Color | Basic Correction, Creative (looks), Curves, Color Wheels & Match, HSL Secondary, Vignette. | [CC][CC], [LV][LV] (F) | partial | `brightness`, `contrast`, `saturation`, `tint` (ADR-0049). Vignette composed (ADR-0040). Curves and LUTs refused (ADR-0049, ADR-0040) | White balance, exposure, the tone-range controls, vibrance, colour wheels and HSL secondaries are missing. The colour area covers Lumetri in depth. |
| Look-up tables (Lumetri Input LUT and Look) | Apply a LUT from the Input LUT menu, or browse to one on disk. | [LUT][LUT] (F) | refused | ADR-0040 ("semantics live entirely in an external `.cube` file … a plugin in all but name"), ADR-0049 | — |
| Tint | Each pixel's luminance blends between Map Black To and Map White To colours, by Amount To Tint. | [CC][CC], [L][L] (F) | partial | `tint{color, amount}` pushes every pixel toward one colour (ADR-0049) | Premiere's two-colour map is a duotone, which ADR-0049 excludes. The one-colour wash ships. |
| Video Limiter | Limits RGB to a Clip Level in IRE (100, 103, 105, 109), with a compression knee and a gamut warning; usable per shot or as an export effect. | [CC][CC], [L][L] (F) | missing | — | A legal-range clamp. |
| Vignette (was "FI: Vignette FX") | Darkens the frame edges. | [L][L], [MF][MF] (F) | have | Composed: a radial-gradient `rect` to transparent, blended `multiply` (ADR-0040, ADR-0149, ADR-0147; `montagent` skill) | — |

### Distort

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Corner Pin | Shifts the four corners independently to stretch, skew, twist or mimic perspective. | [DIS][DIS], [MF][MF], [L][L] (F) | refused | ADR-0167: "a camera, depth sorting, skew and corner pin stay refused"; swivel/tilt cover rigid turns | — |
| Lens Distortion | "Simulates or corrects lens distortion." An older Lens Distortion is in the 26.0 removed list. | [DIS][DIS], [MF][MF], [L][L] (F) | missing | — | A lens warp. Workaround: `montagent-prerender`. |
| Magnify (modern, 26.3) | A loupe that enlarges part of the frame, with size, magnification, edge softness, border, shadow and position. | [DIS][DIS], [MF][MF], [WN][WN], [RN][RN] (F) | partial | Composed: a scaled copy of the element with a circular `mask`, `feather` and `shadow` | A loupe that magnifies the element's own pixels in place, without a second element. |
| Mirror | Splits the image along a line and reflects one side onto the other. | [DIS][DIS], [MF][MF], [L][L] (F) | partial | `scale: [-1, 1]` flips a whole element (format.md). A flipped, masked copy gives an axis-aligned mirror | A mirror line at an arbitrary angle within one element. |
| Spherize | Wraps a region of the image onto a sphere. | [DIS][DIS], [L][L] (F) | missing | — | A bulge warp. Workaround: `montagent-prerender`. |
| Turbulent Displace | Fractal-noise turbulence, for water, funhouse mirrors and flags. | [DIS][DIS], [MF][MF], [L][L] (F) | missing | — | A seeded noise displacement. ADR-0156 §3 has the seed and time rules it would need. |
| Twirl (rebuilt in 26.5) | Rotates pixels round a centre, sharper at the centre, with radius and angle; 26.5 adds new controls. | [DIS][DIS], [MF][MF], [L][L], [RN][RN], [WN][WN] (F) | missing | — | A twirl warp. Workaround: `montagent-prerender`. |
| Wave Warp | A wave travelling across the image (square, circular, sine…), animated at constant speed without keyframes; keyframes vary the speed. | [DIS][DIS], [L][L] (F) | missing | — | A wave member. ADR-0156 §3 says a wave takes a keyed phase rather than a time input. |
| Warp Stabilizer | Analyses the clip in the background, then smooths (Smooth Motion, Smoothness) or removes (No Motion) camera motion, by Position; Position, Scale, Rotation; Perspective; or Subspace Warp. Preserve Scale; Detailed and Fast Analysis. | [DIS][DIS], [WS][WS], [WSS][WSS], [L][L] (F) | missing | — | Motion analysis and stabilisation. |
| Warp Stabilizer border framing | Stabilize Only; Stabilize, Crop; Stabilize, Crop, Auto-scale (Maximum Scale, Action-Safe Margin, Additional Scale); Stabilize, Synthesize Edges from nearby frames (input range, edge feather, edge cropping); Crop Less ↔ Smooth More. | [WSS][WSS] (F) | missing | `scale` and `clip` can crop and refill by hand | Auto-scale from analysis, and edge synthesis from other frames. |
| Warp Stabilizer Rolling Shutter Ripple | Automatic or Enhanced Reduction of rolling-shutter ripple, with Subspace Warp or Perspective. | [WSS][WSS] (F) | missing | — | Motion analysis. |
| Rolling Shutter Repair (Obsolete) | Fixes wobbly or slanted footage caused by camera shake. | [DIS][DIS], [L][L] (F) | missing | — | Motion analysis. |

### Generate

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| 4-Color Gradient | Four effect points, each a solid-colour circle, blended; positions and colours animatable. | [GEN][GEN], [L][L] (F) | partial | Linear and radial gradient paint (ADR-0149) | A four-point gradient. |
| Gradient (modern, 26.2.2; replaces Ramp) | Linear or radial gradient with position, scale, orientation, repetition, mirroring, interpolation, feathering and grain; tone and ambient colour blending, desaturation, posterisation, and optional texture (pattern, scale, distortion, intensity). | [MF][MF], [WN][WN], [RN][RN] (F) | partial | Gradient paint with animatable angle, centre, radius and stops (ADR-0149); `grain`, `posterize`, `saturation` on the element (ADR-0156, ADR-0049) | Repetition and mirroring, an interpolation choice, and the texture layer. |
| Ramp (Legacy) | Linear or radial ramp, animatable start and end, with Ramp Scatter against banding. | [GEN][GEN], [L][L] (F) | partial | Linear and radial gradient paint (ADR-0149) | Ramp Scatter (dithering). |
| Lens Flare (Obsolete; rebuilt as a modern effect in 26.5) | Simulates bright light hitting a lens; the modern one has flare shape, streak intensity and colour. | [GEN][GEN], [L][L], [MF][MF], [WN][WN], [RN][RN] (F) | have (skill) | `montagent-prerender` names lens flare in its trigger (ADR-0156 §7) | — |
| Lightning (Obsolete), including the Jacob's Ladder look | Animated bolts between two points, without keyframes; Stability, Pull Force and Pull Direction make a Jacob's Ladder. Usually placed on Transparent Video. | [GEN][GEN], [L][L], [LIT][LIT], [JL][JL] (F) | have (skill) | `montagent-prerender` | — |

### Image Control

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Black & White | Turns the clip greyscale; cannot be keyframed. | [IC][IC], [L][L] (F) | have | `saturation{amount: 0}` (ADR-0049) | — |
| Channel Mix (was "FI: Channel Mix FX"; moved here from Tools) | Mixes and remaps colour channels across RGB, HSV and YUV. | [L][L], [MF][MF] (F) | refused | ADR-0049 excludes per-channel colour matrices | — |
| Color Pass | Turns the clip greyscale except one chosen colour. | [IC][IC], [L][L] (F) | missing | — | A colour-keyed desaturation. |
| Color Replace | Replaces a selected colour with a new one, keeping grey levels. | [IC][IC], [L][L] (F) | missing | — | Colour-range replacement. |
| Gamma Correction | Changes brightness by the midtones only; 1 to 28, default 10. | [IC][IC], [L][L] (F) | missing | ADR-0049 cites `gamma{value}` as a legitimate future scalar | A gamma member. |
| Invert (moved here in 26.0) | Inverts RGB, R, G, B, HLS, H, L, S or alpha, with Blend With Original. | [CH][CH], [L][L], [RO][RO] (F) | missing | ADR-0049 names `invert` as a request its stopping rule would test | An invert member. #703: one colour-matrix call. |
| Rounded Crop (was "FI: Rounded Crop FX") | Crops with rounded corners. | [L][L], [MF][MF] (F) | have | `mask` with `shape: "rect"` and `radius` (ADR-0084, ADR-0152) | — |

### Immersive Video

Montagent has no spherical (360°/180°) projection model, and no ADR refuses one. Each effect
is `missing` because the projection it needs is missing. The VR area (#909) covers the model.

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| VR Blur | Blur spread evenly over a 360° or 180° frame, poles and edges included. | [IMF][IMF], [IMT][IMT], [L][L] (F) | missing | `blur` is planar (ADR-0040) | Spherical wrap. |
| VR Chromatic Aberrations | Colour fringing at the edges of a 360° frame. | [IMF][IMF], [IMT][IMT], [L][L] (F) | missing | — | Spherical model, and channel offsets. |
| VR Color Gradients | Gradient fills that blend across the sphere. | [IMF][IMF], [IMT][IMT], [L][L] (F) | missing | Planar gradients only (ADR-0149) | Spherical model. |
| VR De-Noise | Noise reduction for 360° footage. | [IMF][IMF], [IMT][IMT], [L][L] (F) | missing | — | Denoise, and the spherical model. |
| VR Digital Glitch | Digital-glitch distortion across the sphere. | [IMF][IMF], [IMT][IMT], [L][L] (F) | missing | — | Spherical model. |
| VR Fractal Noise | Evolving fractal-noise textures (smoke, clouds). | [IMF][IMF], [IMT][IMT], [L][L] (F) | missing | — | Spherical model and fractal noise. |
| VR Glow | Glow round bright areas, seamless across the sphere. | [IMF][IMF], [IMT][IMT], [L][L] (F) | missing | `glow` is planar (ADR-0156) | Spherical wrap. |
| VR Plane to Sphere | Wraps flat images, text or logos onto the 360° sphere. | [IMF][IMF], [IMT][IMT], [L][L] (F) | missing | — | Equirectangular projection. |
| VR Projection | Projects 2D footage onto the spherical surface. | [IMF][IMF], [IMT][IMT], [L][L] (F) | missing | — | Spherical model. |
| VR Rotate Sphere | Rotates 360° footage about X, Y and Z, e.g. to level the horizon. | [IMF][IMF], [IMT][IMT], [L][L] (F) | missing | — | Spherical model. |
| VR Sharpen | Uniform sharpening for 360° footage. | [IMF][IMF], [IMT][IMT], [L][L] (F) | missing | — | Sharpen, and the spherical model. |

### Keying

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Ultra Key | Eyedropper key colour. Matte Generation: Transparency, Highlight, Shadow, Tolerance, Pedestal. Matte Cleanup: Choke (morphological erode), Soften (box blur), Contrast, Mid Point. Spill Suppression: Desaturate, Range, Spill, Luma. Color Correction: Saturation, Hue, Luminance. An Output menu shows the stages. GPU-accelerated. | [KEY][KEY], [UK][UK], [UKA][UKA], [L][L] (F) | partial | `chroma{color, tolerance, softness, spill}`, all keyable (ADR-0088); `montagent-footage` green-screen recipe | Choke, Pedestal, Highlight and Shadow, matte Contrast and Mid Point, spill Range and Luma, foreground hue, and an alpha-channel view. Soften maps only roughly to `softness`. |
| Color Key | Keys out every pixel like the key colour; adjusts only the alpha channel. | [KEY][KEY], [L][L] (F) | have | `chroma` with `color`, `tolerance` and `softness` (ADR-0088) | Edge Thin and Edge Feather, from the earlier reading, are not on the fetched page (unconfirmed). |
| Luma Key | Removes areas by luminance, e.g. a white background behind dark notes. 32-bit and YUV. | [KEY][KEY], [L][L] (F) | missing | — | A luminance matte. ADR-0152 defers luma-sourced masks. |
| Track Matte Key | Reveals one clip through another using a third clip on its own track as a matte; white is opaque, black transparent; the matte can move. | [KEY][KEY], [L][L] (F) | refused | ADR-0150 §4 refuses a matte from another element; the element's own `mask` is the documented substitute | — |
| Alpha Adjust | An Opacity percentage used in place of the fixed Opacity effect when the render order must change. | [KEY][KEY], [TYP][TYP], [L][L] (F) | partial | `opacity` (ADR-0012) | Opacity at a chosen point in the effect order. Ignore Alpha, Invert Alpha and Mask Only, from the earlier reading, are not on the fetched pages (unconfirmed). |
| Logo Cutout (was "FI: Alpha FX") | "Controls transparency and use for overlay transitions or layering effects." | [L][L], [MF][MF] (F) | partial | `opacity`, `mask` | Mechanism described in one line; re-check. |

### Lights & Glows

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Echo Glow (was "FI: Echo Glow FX") | A stylised glow with echoing or trailing light. | [L][L], [MF][MF] (F) | missing | — | A glow that reads earlier frames. |
| Edge Glow (was "FI: Edge Glow FX") | A glow along the outlines and contours. | [L][L], [MF][MF] (F) | missing | — | Edge-detected glow. |
| Glint (was "FI: Glint FX") | Brief sparkles or glimmers. | [L][L], [MF][MF] (F) | missing | — | A seeded, time-reading sparkle. |
| Light Leaks (was "FI: Light Leaks FX") | Glow and occasional flare, as light leaking onto film. | [L][L], [MF][MF] (F) | have | Composed: radial gradient `rect` blended `screen`/`add` (ADR-0147 names light leaks as its motive; `montagent` skill) | The "occasional" random timing is keyed by hand. |
| RGB Split (was "FI: RGB Split FX") | Splits the colour channels for chromatic distortion. | [L][L], [MF][MF] (F) | have (skill) | `montagent-motion` trailer recipe: red and cyan tinted copies blended `screen`, offset | One member on a single element. |
| Volumetric Rays (was "FI: Volumetric Rays FX") | Natural light rays. | [L][L], [MF][MF] (F) | missing | — | A radial ray shader. |
| Wonder Glow (was "FI: Wonder Glow FX") | A vibrant, radiant glow. | [L][L], [MF][MF] (F) | partial | `glow{threshold, radius, intensity}` (ADR-0156) | A glow colour. ADR-0156 left `color` out on purpose: the glow takes the element's own colours. |
| Focus Glow FX | A localised glow round focal areas. On the Modern effects page; absent from the 26.0 table. | [MF][MF] (F) | partial | `glow` on a masked copy (ADR-0156, ADR-0152) | One member. |

### Noise & Grain

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Noise (modern, 26.2.2) | Animated noise, with level spread across shadows, midtones and highlights, noise saturation, HDR-compatible blend modes (Normal, Screen, Additive, Soft Light) and preserve alpha. | [NG][NG], [MF][MF], [L][L], [RN][RN] (F) | partial | `grain{seed, amount, size, mono}`, re-rolled every frame from a literal seed and alpha-preserving (ADR-0156 §3–4) | Tonal-range weighting, and a blend mode inside the effect. A grain `rect` with `blend: screen`/`add`/`overlay` covers part of the latter; Soft Light is not a blend mode (ADR-0147). |

### Perspective

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Basic 3D | Rotates round the horizontal and vertical axes and moves nearer or farther, with an optional specular highlight from a fixed light. | [PER][PER], [L][L], [ACC][ACC] (F) | partial | `swivel`, `tilt`, `perspective`, all keyable (ADR-0167, ADR-0168) | The specular highlight. |
| Drop Shadow | A soft shadow of the alpha channel behind the clip, which can fall outside the clip's bounds; Shadow Only renders the shadow without the image. | [PER][PER], [L][L] (F) | partial | `shadow{dx, dy, radius, color, opacity}` (ADR-0040); direction and distance are polar dx and dy | Shadow Only. |
| Long Shadow (was "FI: Long Shadow FX") | Extended shadows on flat graphics for 3D-style depth. | [L][L], [MF][MF] (F) | missing | — | A long cast shadow. #703: a runtime shader. |

### Stylize

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Alpha Glow (Legacy) | Colour along the alpha edge that fades out or shifts to a second colour. | [STY][STY], [L][L] (F) | partial | `shadow` with zero offset is an alpha glow (`montagent-motion` trailer recipe; ADR-0156 §4) | A two-colour, fading glow. |
| Brush Strokes | A rough painted look with strokes scattered randomly; alters alpha and colour. | [STY][STY], [L][L] (F) | missing | — | Seeded brush strokes. |
| Color Emboss | Emboss without suppressing the image's colours. | [STY][STY], [L][L] (F) | missing | — | A relief kernel. |
| Find Edges | Emphasises edges as dark lines on white or coloured lines on black. | [STY][STY], [L][L] (F) | missing | — | A convolution kernel. |
| Mosaic (modern; was "FI: Mosaic FX") | Pixelation into solid tiles, grid or polygonal, with Softness (refined in 26.5); also for obscuring faces or as a transition. | [STY][STY], [MF][MF], [L][L], [RN][RN], [WN][WN] (F) | missing | — | A mosaic member. `montagent-prerender` names mosaic as its job, but only for footage processed outside Montagent. |
| Posterize | Number of tonal levels per channel, 2 to 255. | [STY][STY], [L][L] (F) | have | `posterize{levels}`, keyable (ADR-0156 §4) | — |
| Replicate (Legacy) | Divides the screen into tiles, each showing the whole image. | [STY][STY], [L][L] (F) | have | Composed: N² copies of the element, each scaled and placed, written out (ADR-0148 refuses a repeat construct and asks for copies) | — |
| Roughen Edges | Roughens the alpha edge, like eroded metal or typewriter text. | [STY][STY], [L][L] (F) | missing | — | Seeded edge displacement. |
| Strobe Light | An arithmetic operation on the clip (e.g. inverting colours) or full transparency, at periodic or random intervals. | [STY][STY], [L][L] (F) | partial | `montagent-motion` trailer recipe: a full-frame `rect` flash or `opacity` keyed on beats, random intervals written by a script | The arithmetic operations (invert and the like); there is no invert member. |

### Time

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Echo (Obsolete) | Combines frames from different times in the clip, for echoes, streaks and smears. | [TIME][TIME], [L][L] (F) | missing | Each element paints only its current instant; `motion_blur` averages within one shutter (ADR-0155) | Frame echoes. |
| Posterize Time | Locks the clip to a frame rate; the rate can only be keyed with Hold. | [TIME][TIME], [L][L] (F) | partial | On video, a `source_time` curve with `step` keys holds frames (ADR-0157). On graphics, `step`-eased keyframes do the same | A single frame-rate field per element. |
| Time Remapping (fixed effect) | Speed changes, freeze frames, slow and fast motion. | [TYP][TYP], [ECP][ECP] (F) | have | `source_time` literal for a freeze, curve for a ramp, falling segment for reverse (ADR-0157) | The editing area owns frame blending and optical flow. |

### Transform

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| 3D Rotate (was "FI: 3D Rotate FX") | Full-space 3D rotation. | [L][L], [MF][MF] (F) | have | Keyed `swivel`/`tilt` with `perspective` (ADR-0167) | — |
| Auto Reframe (clip) | Tracks the action and writes motion keyframes to reframe for another aspect ratio; Slower Motion, Default or Faster Motion; overwrite the path; adjust position, offset, scale, rotation. | [TR][TR], [AR][AR], [ARC][ARC], [L][L] (F) | missing | `fit`, `clip`, keyed `x`/`y`/`scale` written by hand | Subject tracking. |
| Auto Reframe Sequence | Duplicates a sequence at a target aspect ratio and custom resolution, applying Auto Reframe to every clip, optionally nesting clips to keep motion keyframes. | [ARS][ARS], [AR][AR], [RN][RN] (F) | missing | A new project at another canvas size is written by hand | Subject tracking, and a whole-project reframe. |
| Camera Shake (was "FI: Camera Shake FX") | Shakes footage like a handheld camera. | [L][L], [MF][MF], [RN][RN] (F) | have (skill) | `montagent-motion` trailer recipe: per-frame `x`/`y` keyframes written by a generator script | A seeded shake member. Today a script bakes it to literal keys. |
| Crop (Legacy) | Removes a percentage from Left, Top, Right and Bottom; Fit or Fill decides how the result fills the frame. | [TR][TR], [L][L] (F) | have | `clip` (static, frame space) and a keyable `mask` rect (ADR-0084, ADR-0152); `fit` (ADR-0015) | — |
| Edge Feather (Legacy) | A soft black border round the video, by Amount, like a vignette. | [TR][TR], [L][L] (F) | have | A full-box `mask` rect with `feather` (ADR-0152), over the black canvas | Over a non-black backdrop, a black `rect` under the element keeps the black border. |
| Grow (was "FI: Grow FX") | Animates an element growing. | [L][L], [MF][MF] (F) | have | Keyed `scale` (ADR-0012, ADR-0146) | — |
| Horizontal Flip | Reverses each frame left to right; play direction is unchanged. | [TR][TR], [L][L] (F) | have | `scale: [-1, 1]` (format.md, ADR-0012) | — |
| Move (was "FI: Move FX") | Moves an element across the frame with customisable motion. | [L][L], [MF][MF] (F) | have | Keyed `x`/`y` (ADR-0012, ADR-0146) | — |
| Offset (moved here in 26.0) | Pans the image; what leaves one side reappears on the other. | [DIS][DIS], [L][L], [RO][RO] (F) | missing | — | Wrap-around tiling. #703: an image shader with repeat tiling. |
| Shrink (was "FI: Shrink FX") | Animates an element shrinking. | [L][L], [MF][MF] (F) | have | Keyed `scale` | — |
| Spacer (was "FI: Spacer FX") | "Smooth scene separation by adding deliberate spacing between clips or elements." | [L][L], [MF][MF] (F) | missing | — | Mechanism described in one line; re-check. |
| Spin (was "FI: Spin FX") | Continuous rotation. | [L][L], [MF][MF] (F) | have | Keyed `rotation` | — |
| Transform (effect) | Position, size, rotation and opacity as a standard effect; also scale below zero, a shutter angle and a motion path. | [TR][TR], [TYP][TYP], [L][L], [RN][RN] (F) | partial | Transform fields, `opacity`, `motion_blur` (ADR-0012, ADR-0155) | There is no way to place the transform inside the effect order (see Effect controls). |
| Vertical Flip | Turns the clip upside down; cannot be keyframed. | [TR][TR], [L][L] (F) | have | `scale: [1, -1]` | — |
| Wiggle (was "FI: Wiggle FX") | Randomised, playful jitter. | [L][L], [MF][MF] (F) | have (skill) | As Camera Shake | A seeded wiggle member. |

### Transition (effects that act as transitions on one clip)

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Block Dissolve (Legacy) | The clip disappears in random blocks of set width and height. | [TE][TE], [L][L] (F) | missing | — | Seeded block pattern. |
| Gradient Wipe (effect; current) | Pixels go transparent by the luminance of the matching pixels in another video track, the gradient layer, as Transition Completion rises. | [TE][TE], [L][L], [ACC][ACC] (F) | refused | ADR-0150 §4: a matte from another element is refused | — |
| Linear Wipe | A linear wipe in a set direction. | [TE][TE], [L][L] (F) | have | A keyed `mask` (path or rect) with `feather` (ADR-0150, ADR-0152, ADR-0163) | — |

### Utility

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Auto Align (was "FI: Auto Align FX") | Positions titles or logos automatically within the layout. | [L][L], [MF][MF] (F) | partial | Nine `origin` keywords and `fit` (ADR-0013, ADR-0015); safe areas in `montagent-craft` | Automatic placement. Montagent places from literals. |
| Cineon Converter | Controls the conversion of 10-bit log Cineon frames, keyable. | [UT][UT], [L][L] (F) | missing | Colour handling is sRGB, 8-bit | Log and colour-space conversion. The colour area owns it. |
| Clone (was "FI: Clone FX") | Duplicates and animates logos, text or footage with offset timing. | [L][L], [MF][MF] (F) | have | Composed: copies of the element written out, each with keyframes offset in time (ADR-0148 asks for copies) | — |
| Metadata & Timecode Burn-in | Shows clip metadata (sound timecode, source timecode, file name, frame rate) as up to five lines of text, on a clip, adjustment layer or transparent clip. Replaces Clip Name and Timecode. | [VF][VF], [L][L], [RM][RM], [RN][RN] (F) | missing | Text is literal (ADR-0007); `query` reads timings | Text generated from metadata, and a running timecode. |
| Simple Text | Quick plain text over footage. | [VF][VF], [L][L] (F) | have | `text` element (ADR-0007) | — |
| Stroke (was "FI: Stroke FX") | Outlines any element: text, shapes or footage. | [L][L], [MF][MF] (F) | partial | `stroke` paint on shapes and text (ADR-0014, ADR-0160) | A stroke round the alpha of an image or video. |

### Video

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| SDR Conform (Obsolete in the 26.0 table) | Converts HDR footage to SDR, modulating brightness and colour for SDR displays. | [VF][VF], [L][L] (F) | missing | — | HDR handling. The colour area owns it. |

### Video effects removed in 26.0 or earlier

Adobe's 26.0 table says these "will not render". The removal page lists Adobe's suggested
alternative. Most entries are names only, so their mechanisms carry ‡.

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Anti-Alias | Removed (Windows); Adobe points to third parties. Edge smoothing. ‡ | [RM][RM] (F) | have | Skia draws anti-aliased edges by default | — |
| Arithmetic | Per-channel operators against constant values. Removed; Adobe points to Lumetri. ‡ | [L][L], [RM][RM] (F) | missing | — | Per-channel arithmetic. ADR-0049 excludes per-channel colour matrices, so any proposal starts there. |
| Auto Color | Neutralises the colour cast by analysis. Removed; Adobe points to Lumetri. ‡ | [L][L], [RM][RM] (F) | missing | — | A colour correction computed from source pixels. |
| Auto Contrast | Automatic contrast from the histogram. Removed. ‡ | [L][L], [RM][RM] (F) | missing | `contrast` exists, but only as a literal amount (ADR-0049) | The automatic analysis. |
| Auto Levels | Automatic per-channel levels. Removed. ‡ | [L][L], [RM][RM] (F) | missing | — | Per-channel levels, automatic or manual. |
| Bevel Alpha | Bevelled lighting on the alpha edge. Removed; Adobe points to a Photoshop PSD. ‡ | [L][L], [RM][RM] (F) | missing | — | Alpha-edge bevel. |
| Bevel Edges | Bevelled, lit rectangular edges. Removed; its preset bin is gone too. ‡ | [L][L], [RM][RM] (F) | missing | — | Edge bevel. |
| Blend | Blends with a clip on another track by crossfade, colour-only, tint-only, darken or lighten. Removed; Adobe points to Blend Modes. ‡ | [L][L], [RM][RM] (F) | partial | `blend` with five modes over whatever lies beneath (ADR-0147) | Naming a specific other layer is refused (ADR-0150). Colour-only, tint-only, darken and lighten modes were refused for now (ADR-0147 "Considered and refused"). |
| Calculations | Combines one channel of this clip with one of another track. Removed. ‡ | [L][L], [RM][RM] (F) | refused | ADR-0150 §4: reading another element's pixels | — |
| Camera View | Views the clip from a virtual camera (longitude, latitude, roll, focal length). Removed; Adobe points to Photoshop or After Effects. ‡ | [L][L], [RM][RM] (F) | refused | ADR-0167: "a camera … stay[s] refused" | — |
| Cell Pattern | Procedural cellular pattern. Removed. ‡ | [L][L], [RM][RM] (F) | have (skill) | `montagent-prerender`: a code-drawn piece as lossless footage with alpha (ADR-0156 §7) | — |
| Change Color | Changes hue, lightness and saturation of a colour range. Removed. ‡ | [L][L], [RM][RM] (F) | missing | — | Hue-selective correction. |
| Change to Color | Replaces a selected colour with another. Removed. ‡ | [L][L], [RM][RM] (F) | missing | — | Hue-selective replacement. |
| Channel Mixer | Mixes colour channels. Removed; Adobe points to Lumetri. ‡ | [L][L], [RM][RM] (F) | refused | ADR-0049 excludes per-channel colour matrices | — |
| Checkerboard | Procedural checkerboard. Removed. ‡ | [L][L], [RM][RM] (F) | have (skill) | `montagent-prerender`; or one `rect` per square, written out (ADR-0148) | — |
| Circle | Draws a circle or ring. Removed; Adobe points to masking tools. ‡ | [L][L], [RM][RM] (F) | have | `ellipse` element with `fill` and `stroke` | — |
| Clip Name | Overlays the clip's name. Removed; Adobe points to Metadata Burn-in. ‡ | [L][L], [RM][RM] (F) | partial | A `text` element with the name written literally | Automatic text from clip metadata. |
| Color Balance | Red, green and blue levels. Obsolete, then removed; Adobe points to Lumetri. ‡ | [L][L], [RM][RM] (F) | missing | `tint` shifts toward one colour (ADR-0049) | Independent per-channel gain. |
| Color Balance (HLS) | Hue, lightness and saturation. | unconfirmed (no fetched page names an HLS variant) | partial | `brightness`, `saturation` (ADR-0049) | Hue rotation. |
| Compound Arithmetic | Combination with a second source layer. Removed; Adobe points to Blend Modes. ‡ | [L][L], [RM][RM] (F) | refused | ADR-0150 §4 | — |
| Convolution Kernel | A user-defined 3×3 kernel with scale and offset. Removed, with its preset bin. ‡ | [L][L], [RM][RM] (F) | missing | — | An arbitrary kernel. #703 maps it to Skia `matrix_convolution`. |
| Difference Matte | Keys out pixels that match a reference clip. Removed. ‡ | [L][L], [RM][RM] (F) | refused | ADR-0150 §4: a matte computed from another element's pixels | — |
| Dust & Scratches | Median-based cleanup. Removed; Adobe points to Gaussian Blur. ‡ | [L][L], [RM][RM] (F) | missing | — | As Median. |
| Ellipse | Draws an ellipse outline. Removed; Adobe points to masking. ‡ | [L][L], [RM][RM] (F) | have | `ellipse` element | — |
| Emboss | Greyscale emboss. Removed; Adobe points to Color Emboss. ‡ | [L][L], [RM][RM] (F) | missing | — | As Color Emboss. |
| Equalize | Equalises the brightness histogram. Removed. ‡ | [L][L], [RM][RM] (F) | missing | — | Histogram equalisation. |
| Eyedropper Fill | Fills the clip with a colour sampled from the clip. Removed. ‡ | [L][L], [RM][RM] (F) | missing | Colours are literal hex strings (ADR-0088) | A colour sampled from pixels. That would fail ADR-0145's literal-values test if done at render time. |
| Fast Blur | Faster box-style blur. Removed; Adobe points to Gaussian Blur. ‡ | [L][L], [RM][RM] (F) | have | `blur` (ADR-0040) | — |
| Fast Color Corrector | Hue-balance wheel and levels. Removed; Adobe points to Lumetri. ‡ | [L][L], [RM][RM] (F) | missing | — | Colour wheels and levels. |
| Ghosting | Trails of earlier frames over the clip. Removed (Windows); Adobe points to Opacity and speed effects. ‡ | [L][L], [RM][RM] (F) | missing | `motion_blur` averages within one shutter only (ADR-0155) | Frame trails. |
| Grid | Procedural grid. Removed. ‡ | [L][L], [RM][RM] (F) | have (skill) | `montagent-prerender`; or `path` elements written out | — |
| Horizontal Hold | Simulates a slipping analogue horizontal sync. Removed (Windows); Adobe points to third parties. ‡ | [L][L], [RM][RM] (F) | missing | — | Mechanism undocumented on the fetched pages; re-check. |
| Image Matte Key | Uses a still image's alpha or luma as the matte. Removed; Adobe points to Blend Modes. ‡ | [L][L], [RM][RM] (F) | missing | ADR-0152 keeps image-sourced masks deferred, not refused | An image-sourced mask. |
| Leave Color | Desaturates everything except one colour. Removed. ‡ | [L][L], [RM][RM] (F) | missing | — | A colour-keyed desaturation. |
| Luma Corrector | Tonal-range luminance correction. Removed; Adobe points to Lumetri. ‡ | [L][L], [RM][RM] (F) | missing | `brightness`, `contrast` are global (ADR-0049) | Tonal-range correction. |
| Luma Curves | An arbitrary luminance curve. Removed; Adobe points to Lumetri. ‡ | [L][L], [RM][RM] (F) | refused | ADR-0049: "Curves, LUTs, per-channel colour matrices, and duotone-style two-colour maps are categorically excluded" | — |
| Median (Legacy) | Median filter. Removed; Adobe points to Lumetri or VR De-Noise. ‡ | [L][L], [RM][RM] (F) | missing | — | A median filter. |
| Noise Alpha | Noise in the alpha channel. Removed; Adobe points to Noise. ‡ | [L][L], [RM][RM] (F) | missing | `grain` never touches alpha (ADR-0156) | Alpha noise. |
| Noise HLS | Separate noise in hue, lightness and saturation. Removed; Adobe points to Noise. ‡ | [L][L], [RM][RM] (F) | partial | `grain` with `mono` true (luma) or false (colour) (ADR-0156) | Separate hue, lightness and saturation amounts. |
| Noise HLS Auto | As Noise HLS, animated automatically. Removed. ‡ | [L][L], [RM][RM] (F) | partial | `grain` re-rolls every frame (ADR-0156) | As Noise HLS. |
| Non Red Key | A key for blue and green screens with edge blending. Removed; Adobe points to Ultra Key. ‡ | [L][L], [RM][RM] (F) | partial | `chroma` (ADR-0088) | Mechanism undocumented on the fetched pages; re-check. |
| Paint Bucket | Flood-fills a region chosen by a point. Removed. ‡ | [L][L], [RM][RM] (F) | missing | — | A pixel-dependent fill. |
| Radial Shadow | A shadow cast from a point light onto the layer beneath. Removed; Adobe points to third parties. ‡ | [L][L], [RM][RM] (F) | missing | `shadow` is a parallel offset | A point-light projection. |
| Radial Wipe (effect) | A clock-hand wipe from a centre. Removed in 26.0. ‡ | [L][L] (F) | partial | A keyed `path` mask whose vertices sweep (ADR-0163) | A pie mask with one angle parameter. Today it is many keyed vertices. |
| Remove Matte | Removes the colour fringe from premultiplied footage. Removed; Adobe points to keyers and Blend Modes. ‡ | [L][L], [RM][RM] (F) | missing | — | Unpremultiply against a matte colour. |
| RGB Color Corrector | Per-channel tonal-range correction. Removed; Adobe points to Lumetri. ‡ | [L][L], [RM][RM] (F) | missing | — | Per-channel tonal correction. |
| RGB Curves | Arbitrary per-channel curves. Removed; now in Lumetri. | [L][L], [RM][RM], [RGBC][RGBC] (F) | refused | ADR-0049 (curves excluded) | — |
| Hue Saturation Curves (Lumetri) | Curves of hue against hue, saturation and luma. | [HSC][HSC], [CC][CC] (F) | refused | ADR-0049 (curves excluded) | — |
| Set Matte | Takes a channel of another layer as this clip's alpha. Removed; Adobe points to keyers. ‡ | [L][L], [RM][RM] (F) | refused | ADR-0150 §4: a matte from another element is refused | — |
| Shadow/Highlight | Lifts shadows and lowers highlights by local analysis. Removed; Adobe points to Lumetri. ‡ | [L][L], [RM][RM] (F) | missing | — | Tone-range-dependent lift and compression. |
| Solarize | Blends negative and positive. Removed; Adobe points to third parties. ‡ | [L][L], [RM][RM] (F) | missing | — | A solarize curve. |
| Solid Composite | A solid colour behind the clip, with opacity and a blend mode. Removed; Adobe points to generators. ‡ | [L][L], [RM][RM] (F) | have | Composed: a `rect` element on a lower track, with `fill`, `opacity` and `blend` (ADR-0147) | — |
| Texturize | Gives the clip the texture of another layer. Removed. ‡ | [L][L], [RM][RM] (F) | refused | ADR-0150 §4: reads another element's pixels | — |
| Three-Way Color Corrector | Shadow, midtone and highlight colour wheels. Removed; Adobe points to Lumetri. ‡ | [L][L], [RM][RM] (F) | missing | — | Tonal-range colour wheels. |
| Threshold | Black and white at a level. Removed; Adobe points to Lumetri and colour tools. ‡ | [L][L], [RM][RM] (F) | missing | `posterize{levels: 2}` plus `saturation{0}` is close (ADR-0156, ADR-0049) | A luminance threshold at a chosen level. |
| Timecode | Overlays a running timecode. Removed; Adobe points to Metadata Burn-in. ‡ | [L][L], [RM][RM] (F) | missing | Text is literal (ADR-0007) | A generated, running timecode. |
| Venetian Blinds (effect) | Stripes that reveal the clip. Removed in 26.0. ‡ | [L][L] (F) | missing | Several masks on one element intersect; they do not union | A union of strips. |
| Vertical Hold | Simulates a rolling analogue vertical sync. Removed (Windows); Adobe points to third parties. ‡ | [L][L], [RM][RM] (F) | missing | — | Mechanism undocumented on the fetched pages; re-check. |
| Video Limiter (Legacy) | The older legal-range limiter. Removed; Adobe points to Video Limiter. ‡ | [L][L], [RM][RM] (F) | missing | — | As Video Limiter. |
| Write-On | Paints strokes along a keyframed brush position. Removed; Adobe points to Essential Graphics or After Effects. ‡ | [L][L], [RM][RM] (F) | have | `trim_start`/`trim_end` draw a stroke along a `path` (ADR-0160) | — |

## Video transitions

### Transition behaviour

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| A transition between two clips | Placed on a cut and fed by handles from both clips; trim first so frames exist. | [TO][TO], [HAND][HAND] (F) | have | `type: "transition"` naming `from`/`to`, on two tracks (ADR-0059) | — |
| Transition duration | Drag the transition's edge, double-click for Set Transition Duration, or type a Duration in Effect Controls; the change depends on the alignment. | [TD][TD], [MC][MC] (F) | have | The window must equal the two elements' overlap (ADR-0059); duration is set by the overlap | — |
| Alignment: Center at Cut, Start at Cut, End at Cut, Custom Start | Chosen in Effect Controls, or by dragging the transition over the cut. | [ALN][ALN], [TD][TD], [MC][MC] (F) | have | Alignment is where the two elements overlap (ADR-0059) | — |
| Default transition duration | A preference for video and audio transitions (1 second by default; Morph Cut takes 30 frames); changing it does not alter existing transitions. | [TD][TD], [DT][DT], [MCA][MCA] (F) | missing | Every transition is written in full | A project-level default. |
| Single-sided transition | Applied to one clip, at a head or tail with no neighbour or no handle; Ctrl/Cmd-drag for End at Cut or Start at Cut. | [SS][SS], [TO][TO] (F) | partial | An `opacity` ramp over the backdrop (ADR-0012, ADR-0146) | A single-sided transition element. ADR-0059 requires two bridged elements. |
| Edge selectors (orientation and direction) | Arrows on the transition thumbnail change its direction, e.g. Barn Doors vertical or horizontal. | [CTS][CTS] (F) | have | `direction` on `wipe`, `slide` and `push` (ADR-0150) | — |
| Transition settings: Start and End %, border width and colour, Reverse, Anti-Aliasing Quality, Custom | Per-transition options in Effect Controls. | [CTS][CTS] (F) | missing | ADR-0150 §1: "no border, no Start/End %, no Reverse, no soft edge. Each waits for a case." | The options. They are deferred, not refused. |
| Show Actual Sources | Shows the clips' start and end frames in the transition preview. | [CTS][CTS] (F) | missing | `frame` renders any instant | *GUI-only.* |
| Reposition a transition's centre | Drag the centre point (e.g. Iris Round) in the A preview of Effect Controls. | [RC][RC] (F) | have | Composed reveals are masks placed anywhere by their `x`/`y` (ADR-0150, ADR-0084) | The four transition kinds have no centre. |
| Easing a transition | Modern transitions have adjustable easing, speed and bounce. | [MT][MT] (F) | have | Optional `ease` on `wipe`/`slide`/`push`, overshoot by a cubic bezier (ADR-0150, ADR-0146) | No `ease` on `crossfade` (ADR-0150). |
| Default transition | Cross Dissolve and Constant Power Crossfade by default (now the modern Cross Dissolve); Set Selected as Default Transition, for all projects. | [DT][DT], [RO][RO] (F) | missing | Every transition is written in full | A project-level default. |
| Apply default transitions to a selection; Automate to Sequence | Applies the default video and audio transitions at every edit where two selected clips touch. | [DT][DT] (F) | missing | An agent writes each transition | A batch apply of a default. |
| Copy and paste transitions | Paste onto the cut at the playhead or onto many selected edit points, same track type only. | [CPT][CPT] (F) | have | JSON copy, then fix `from`/`to`/`start`/`end`; `validate` checks the references (ADR-0150 §3) | — |
| Replace a transition | Drag a new one on top; alignment and duration stay, the old settings are discarded. | [RPT][RPT], [CPT][CPT] (F) | have | Change `kind` (and `direction`) in place (ADR-0145 invariant 4) | — |
| Move a cut and its transition together | Drag the cut line in Effect Controls; the clips' In and Out points change, not the sequence length. | [MOV][MOV] (F) | have | Rewrite the two elements' boundaries and the transition's window together (ADR-0145 invariant 4) | — |
| Clip handles and insufficient media | Without enough handle frames, Premiere repeats end frames as a freeze and marks the transition with warning bars. | [HAND][HAND], [TO][TO] (F) | partial | `overrun: hold` freezes the last frame past a source's end | No hold before a source's first frame, for an incoming element without a head handle. |
| Video transition handles (26.0); show or hide them (26.5) | Drag a handle at a clip's edge to apply the default transition and set its length. | [VTH][VTH], [WN][WN], [RN][RN] (F) | missing | — | *GUI-only.* |
| Multi Transitions (26.0) | Apply, edit and manage transitions on many audio and video clips at once. | [WN][WN], [RN][RN] (F) | have | One edit writes or changes any number of transition elements (ADR-0145 invariant 4) | — |
| Audio crossfades with video transitions | Audio transitions on the audio clips; Constant Power Crossfade is the default. | [TO][TO], [DT][DT] (F) | have | `audio` field on a visual transition, and the `audio_crossfade` kind (ADR-0176) | The audio area owns the detail. |

### Animation

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Block Motion | "Break the video into moving blocks that transition between scenes." | [L][L], [AN][AN], [MT][MT] (F) | partial | Composed: one masked copy per block with keyed offsets (ADR-0148) | Blocks as one parameter. Today it is N copies. |
| Flip Motion | Flips one clip into the other, vertically or horizontally. | [L][L], [AN][AN], [MT][MT] (F) | have | Keyed `swivel` or `tilt` on the two elements (ADR-0167) | — |
| Fold Motion | Folds or unfolds clips like paper, origami-style. | [L][L], [AN][AN], [MT][MT] (F) | partial | Masked halves, each keyed in `swivel` round the fold line (ADR-0167, ADR-0152) | Several folds take many copies; no depth sorting (ADR-0167). |
| Pop Motion | Clips snap in or out with a pop. | [L][L], [AN][AN], [MT][MT] (F) | have | Keyed `scale` with an overshoot bezier (ADR-0146) | — |
| Pull Motion | Pulls one scene toward the next. | [L][L], [AN][AN], [MT][MT] (F) | have | Keyed `x`/`y`/`scale` on the two elements (ADR-0012, ADR-0146) | — |
| Spin Motion | Spins the clips between shots. | [L][L], [AN][AN], [MT][MT] (F) | have | Keyed `rotation` (ADR-0012) | — |
| Spring Motion | Bouncy, spring-like movement. | [L][L], [AN][AN], [MT][MT] (F) | refused | ADR-0146 refuses spring easing (a cubic bezier with overshoot is the documented substitute) | — |
| Travel Motion | Slides one scene out while pushing the next in. | [L][L], [AN][AN], [MT][MT] (F) | have | `kind: "push"` (ADR-0150) | — |

### Text

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Text Animator | Animates titles with motion presets, without keyframing. | [L][L], [MT][MT] (F) | have | `units`, `unit`, stagger across one text element's units (ADR-0148) | — |
| Typewriter | Reveals text letter by letter. | [L][L], [AN][AN], [MT][MT] (F) | have (skill) | `montagent-motion` `type_on.py` | — |

### Dissolve

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Additive Dissolve (was "FI: Additive Dissolve Impacts") | "Adds color information from clip B to clip A, and then subtracts the color information of clip A from clip B." | [VD][VD], [L][L] (F) | have | Composed: an incoming element with `blend: "add"` and an opacity ramp over the outgoing (ADR-0147) | — |
| Blur Dissolve | Blurs while dissolving. | [L][L], [MT][MT] (F) | have | Composed: keyed `blur` radius on both elements under a `crossfade` (ADR-0146) | — |
| Blur to Color | Blurs into a solid colour, then into the next scene. | [MT][MT] (F) | have | Composed: keyed `blur` and a coloured `rect` crossfade | — |
| Burn Alpha | Parts of the image dissolve away with fiery edges. | [L][L], [MT][MT] (F) | missing | — | A burn-edge matte. |
| Cross Dissolve (modern; "Dissolve Impacts") | Fades out A while fading in B; the default video transition. | [VD][VD], [L][L], [MT][MT], [RO][RO] (F) | have | `kind: "crossfade"` (ADR-0059) | — |
| Dip to Black | Fades A to black, then black to B; it also darkens lower tracks. | [VD][VD], [L][L], [RN][RN] (F) | have | Composed: opacity ramps over the black background, or two crossfades through a black `rect` | — |
| Dip to Color | Fades through a chosen colour. | [L][L] (F) | have | Composed: through a `rect` of that colour | — |
| Dip to White | Fades A to white, then white to B. | [VD][VD], [L][L] (F) | have | Composed: as Dip to Black, through a white `rect` | — |
| Film Dissolve | Blends in a linear colour space (gamma 1.0). | [VD][VD], [L][L] (F) | missing | Blending is premultiplied sRGB; ADR-0147 refuses linear-light arithmetic for `blend` | A linear-light crossfade. ADR-0147's refusal is about `blend`, not transitions, so this is `missing`. |
| Flash | A bright flash or overexposure that hides the cut. | [L][L], [MT][MT] (F) | have (skill) | `montagent-motion` trailer flash recipe | — |
| Luma Fade | Fades by brightness, light to dark or the reverse. | [L][L], [MT][MT] (F) | missing | — | A luma-ordered reveal (ADR-0152 defers luma-sourced masks). |
| Morph Cut | Face tracking and optical-flow interpolation hide a jump cut in a talking head; background analysis restarts on every change. | [VD][VD], [MCO][MCO], [MCA][MCA], [MC][MC], [L][L] (F) | missing | — | Optical flow and face tracking. |
| Mosaic (transition; was "FI: Mosaic Impacts") | Breaks the image into geometric tiles across the cut. | [L][L], [MT][MT] (F) | missing | — | A mosaic member. |

### Grunge & Distort

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Chaos | A random distortion that collapses or explodes into the next scene. | [L][L], [MT][MT] (F) | missing | — | A seeded distortion. |
| Earthquake | A shake across the cut. ‡ | [L][L] (F) | have (skill) | `montagent-motion` trailer shake recipe, under a cut or `crossfade` | — |
| Flicker | A flicker across the cut. ‡ | [L][L] (F) | have (skill) | Per-frame `opacity` keys written by a script (`montagent-motion` trailer) | — |
| Glass | A glass-refraction distortion. ‡ | [L][L] (F) | missing | — | A refraction warp. |
| Glitch | Digital glitches, pixelation and datamoshing. ‡ | [L][L] (F) | have (skill) | `montagent-motion` trailer "Glitch cut" recipe: offset tinted copies, jitter, flashes | Pixelation and datamosh are missing (no mosaic member, no inter-frame reads). |
| Grunge | A dirty-film texture across the cut. ‡ | [L][L] (F) | partial | `grain` on a `rect` blended `overlay` (ADR-0156) | Scratches and dirt. Pre-rendered texture via `montagent-prerender`. |
| Kaleidoscope | A kaleidoscope reflection. ‡ | [L][L] (F) | missing | — | A kaleidoscope warp. |
| Liquid Distortion | A liquid warp. ‡ | [L][L] (F) | missing | — | A displacement warp. |
| TV Power | A CRT power-off squash to a line. ‡ | [L][L] (F) | have | Composed: keyed `scale` `[sx, sy]` to a line, plus a white flash `rect` | — |
| VHS Damage | VHS tracking errors and noise. ‡ | [L][L] (F) | partial | `grain`, RGB-split copies, jitter keys (`montagent-motion`) | Tracking-line displacement. |

### Lights & Blurs

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Burn Chroma | Colourful burn edges and chromatic distortion wipe between clips. | [L][L], [MT][MT] (F) | missing | — | A burn matte with channel split. |
| Chroma Leak | Colourful light leaks with chromatic flares between shots. | [L][L], [MT][MT] (F) | have | Composed: a gradient `rect` blended `screen`/`add`, keyed, over a `crossfade` (ADR-0147, ADR-0149) | — |
| Cross Zoom (Adobe lists it under Zoom on the transitions list) | Zooms into the outgoing clip and out of the incoming one. ‡ | [VT][VT], [L][L] (F) | partial | Keyed `scale` and a `crossfade` (ADR-0012, ADR-0059) | A radial zoom blur. |
| Directional Blur (transition) | Blurs the transition along a horizontal, vertical or custom direction. | [L][L], [MT][MT] (F) | have | Composed: keyed `directional_blur` on both, under a `crossfade` or `push` (ADR-0156) | — |
| Flare | Lens flares across the cut. | [L][L], [MT][MT] (F) | have (skill) | Flare pre-rendered with `montagent-prerender`, blended `screen` | — |
| Glow | A glow that intensifies, then fades into the next shot. | [L][L], [MT][MT] (F) | have | Composed: keyed `glow` intensity under a `crossfade` (ADR-0156, ADR-0146) | — |
| Lens Blur | A lens blur across the cut. ‡ | [L][L] (F) | partial | Keyed `blur` under a `crossfade` | A lens (bokeh) kernel. |
| Light Leak | Film burns and light leaks across the cut. | [L][L], [MT][MT] (F) | have | Composed: as Chroma Leak | — |
| Light Sweep | A sweeping beam of light reveals the next scene. | [L][L], [MT][MT] (F) | have | Composed: a linear gradient `rect` blended `add`, keyed across | — |
| Phosphor | A phosphorescent fade that leaves trails of light. | [L][L], [MT][MT] (F) | missing | — | Light trails, which read earlier frames. |
| Radial Blur | Spins the transition with a radial blur toward or away from the centre. | [L][L], [MT][MT] (F) | missing | — | A rotational blur. |
| Ray | Light rays burst across the transition. | [L][L], [MT][MT] (F) | missing | — | A ray shader. |
| Solarize (transition) | Inverted or high-contrast colour, like photographic solarisation. | [L][L], [MT][MT] (F) | missing | — | A solarize curve and an invert. |
| Stripe | Bands or stripes of light. | [L][L], [MT][MT] (F) | have | Composed: several linear-gradient `rect` bands blended `add`, keyed across, under a `crossfade` (ADR-0147, ADR-0149) | — |
| Zoom Blur | A zoom-like blur, as at warp speed. | [L][L], [MT][MT] (F) | missing | `directional_blur` is linear (ADR-0156) | A radial zoom blur. |

### Slide

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| 3D Roll | Rolls the footage in 3D perspective. | [L][L], [MT][MT] (F) | have | Keyed `tilt` with `perspective` on both elements (ADR-0167) | — |
| Film Roll | Mimics film rolling through a projector. | [L][L], [MT][MT] (F) | partial | Keyed `y` under a `push` | Frame borders and sprocket texture, and a motion smear (`motion_blur` covers that). |
| Push (modern) | The next clip slides over the previous one in any direction. | [VT][VT], [L][L], [MT][MT] (F) | have | `kind: "push"` with `direction` and `ease` (ADR-0150) | — |
| Roll | Rolls the outgoing shot into the incoming one with a spinning or rolling motion. | [L][L], [MT][MT] (F) | partial | Keyed `rotation`/`swivel` under a `push` | Mechanism only described in one line; re-check. |
| Slide (modern, 26.2.2) | Slides clips across the frame, with motion styles, speed, bounce and direction. | [VT][VT], [MT][MT], [WN][WN], [RN][RN] (F) | have | `kind: "slide"` with `direction` and `ease` (ADR-0150) | — |
| Split | Splits the screen into sections that slide apart or together. | [VT][VT], [L][L], [MT][MT] (F) | partial | Composed: two masked copies with keyed offsets | Sections as one parameter. Today it is N copies (ADR-0148). |
| Stretch | Stretches and pulls the footage across the screen, warping into the next clip. | [L][L], [MT][MT] (F) | partial | Keyed non-uniform `scale` `[sx, sy]` under a `push` (ADR-0012) | The warp. Scale is uniform across the element. |
| Whip | A fast push with motion blur. ‡ | [VT][VT], [L][L] (F) | have | Composed: `push` plus `motion_blur` or keyed `directional_blur` (ADR-0150, ADR-0155, ADR-0156) | — |

### Smart Tools

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Motion Camera | "Simulate handheld camera movements to make static shots feel alive." | [L][L], [MT][MT] (F) | have (skill) | `montagent-motion` trailer shake recipe: per-frame `x`/`y`/`rotation` keys written by a script | A seeded shake member. |
| Motion Tween | Automatically creates smooth in-between animation of position, scale and rotation. | [L][L], [MT][MT] (F) | have | Keyframes between literal transforms interpolate automatically (ADR-0012, ADR-0146) | — |
| Shape Dissolve (26.3) | A patterned dissolve; control shape, direction, motion, feathering and timing. | [MT][MT], [WN][WN], [RN][RN] (F) | missing | Several masks on one element intersect; they do not union | A pattern of many shapes revealing at once. |
| Shape Flow | Generates flowing shape-based animations for transitions, backgrounds or accents. | [L][L], [MT][MT] (F) | partial | Keyed shape and `path` elements, and path morphs (ADR-0162) | The generator. Each shape is written by hand. |

### Transformers

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| 3D Spin | Rotates the clip in 3D space into the next shot. | [L][L], [MT][MT] (F) | have | Keyed `swivel` (ADR-0167) | — |
| 3D Spinback (26.2.2) | A stylised 3D spin with motion blur and adjustable easing. | [MT][MT], [WN][WN], [RN][RN] (F) | have | Keyed `swivel` with an overshoot bezier and `motion_blur` (ADR-0167, ADR-0146, ADR-0155) | — |
| Frame | A frame-like border that slides, scales or animates between clips. | [L][L], [MT][MT] (F) | partial | A stroked `rect` keyed with the two elements (ADR-0014) | Mechanism described in one line; re-check. |
| Louver | Horizontal or vertical blinds open or close to reveal the next scene. | [L][L], [MT][MT] (F) | missing | Several masks on one element intersect; they do not union | A union of strips, as Venetian Blinds. |
| Mirror (transition) | Mirrored reflections during the transition. | [L][L], [MT][MT] (F) | partial | Flipped copies with `scale: [-1, 1]` | Mechanism only described in one line; re-check. |
| Page Peel (modern) | Turns the footage like a page corner peeled back. | [L][L], [MT][MT] (F) | missing | — | A page-curl warp. |
| Slice | Cuts the image into slices that slide, stagger or shift away. | [L][L], [MT][MT] (F) | partial | Masked copies with keyed, staggered offsets | Slices as one parameter. Today it is N copies (ADR-0148). |
| Wave | A rippling wave distortion morphs one scene into the next. | [L][L], [MT][MT] (F) | missing | — | A wave warp. |

### Wipe

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Clock Wipe (modern) | A rotating clock-hand wipe. | [L][L], [MT][MT] (F) | partial | A keyed `path` mask with sweeping vertices (ADR-0163) | A one-angle pie mask. |
| Linear Wipe (modern) | A straight-line wipe, horizontal, vertical or diagonal. | [L][L], [MT][MT] (F) | have | A keyed `path` mask, or `kind: "wipe"` for the four directions (ADR-0150, ADR-0163) | — |
| Neon Wipe | A wipe with glowing neon edges. | [L][L], [MT][MT] (F) | partial | A keyed mask, plus a stroked `path` with `glow` along the edge | A lit edge as part of the wipe. |
| Panel Wipe | Sliding panels open, close or cross the frame to reveal the next scene. | [L][L], [MT][MT] (F) | partial | Composed: masked copies, one per panel, with keyed offsets (ADR-0148) | Panels as one parameter. |
| Plateau Wipe | Flattens one clip into a plateau-like surface before unfolding into the next. | [L][L], [MT][MT] (F) | missing | — | A fold warp with depth. |
| Radial Wipe (modern) | A sweep round a corner. ‡ Adobe's table gives its old name as "FI: Star Wipe Impacts" (see Conflicts). | [VT][VT], [L][L] (F) | partial | A keyed `path` mask (ADR-0163) | As Clock Wipe. |
| Soft Wipe | "Apply natural lens-style blur for a soft, out-of-focus transition between clips." | [L][L], [MT][MT] (F) | partial | Keyed `blur` on both elements under a `crossfade` (ADR-0146) | A lens (bokeh) kernel. |
| Star Wipe | A star-shaped reveal or conceal. | [L][L], [MT][MT] (F) | have | A keyed star-shaped `path` mask (ADR-0163) | — |
| Stretch Wipe | Stretches and pulls the footage in a wipe, an elastic effect. | [L][L], [MT][MT] (F) | missing | — | A warp at the edge. |

### Legacy transitions without a modern twin

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Barn Doors (Legacy) | Two doors open from the centre, vertically or horizontally. | [VT][VT], [L][L], [CTS][CTS] (F) | have | Composed: one rect `mask` keyed wider from the centre (ADR-0084) | — |
| Center Split (Legacy) | The outgoing clip splits into four and moves off. ‡ | [VT][VT], [L][L] (F) | partial | Composed: four masked copies with keyed offsets | As Split. |
| Inset (Legacy) | The next clip appears in a corner box that grows. ‡ | [VT][VT], [L][L] (F) | have | Composed: a rect `mask` keyed from a corner | — |
| Iris Box (Legacy) | A growing rectangle reveals the next clip. ‡ | [VT][VT], [L][L] (F) | have | Composed: a keyed rect `mask` on the incoming element (ADR-0150 names the keyed mask as the reveal) | — |
| Iris Cross (Legacy) | A growing cross shape. ‡ | [VT][VT], [L][L] (F) | have | Composed: a keyed `path` mask (ADR-0163) | — |
| Iris Diamond (Legacy) | A growing diamond. ‡ | [VT][VT], [L][L] (F) | have | Composed: a keyed `path` mask (ADR-0163) | — |
| Iris Round (Legacy) | A growing circle round a movable centre. | [VT][VT], [L][L], [RC][RC] (F) | have | Composed: a keyed circle or ellipse `mask` (ADR-0040, ADR-0146) | — |
| Non-Additive Dissolve (Legacy) | "Blends two clips by crossfading their pixel values without brightening the result" (see Conflicts). | [VD][VD], [L][L] (F) | have | `kind: "crossfade"` (ADR-0059) | If the mechanism is the older "brighter pixel wins", it needs a `lighten` blend, refused for now (ADR-0147). |
| Wipe (Legacy) | A straight-edged wipe from a side. ‡ | [VT][VT], [L][L] (F) | have | `kind: "wipe"` with `direction` (ADR-0150) | Diagonal directions and a soft edge. A keyed `path` mask with `feather` covers both. |

### Obsolete and removed transitions

Obsolete transitions still render in old projects; removed ones do not. The removal page lists
the macOS ones as "Third party". All are names only, so their mechanisms carry ‡.

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Band Slide (Obsolete) | Bands slide in from alternate sides. ‡ | [VT][VT], [L][L], [RM][RM] (F) | partial | Composed: one copy of the incoming element per band, each with its own rect `mask` and keyed `x` | Bands as one parameter. Today it is N copies (ADR-0148). |
| Band Wipe (Obsolete) | Bands wipe in from alternate sides. ‡ | [VT][VT], [L][L], [RM][RM] (F) | missing | Masks on one element intersect | A union of band masks. |
| Checker Wipe (Obsolete) | A checker pattern wipes in. ‡ | [VT][VT], [L][L], [RM][RM] (F) | missing | — | A union of many masks. |
| CheckerBoard (Obsolete) | Two checkerboards reveal in turn. ‡ | [VT][VT], [L][L], [RM][RM] (F) | missing | — | As Checker Wipe. |
| Page Peel (Obsolete) | The page curls away. ‡ | [VT][VT], [L][L] (F) | missing | — | A page-curl warp. |
| Page Turn (Obsolete) | The page turns over. ‡ | [VT][VT], [L][L] (F) | missing | — | As Page Peel. |
| Paint Splatter (Obsolete) | Splatter shapes reveal the clip. ‡ | [VT][VT], [L][L], [RM][RM] (F) | missing | — | A splatter matte. |
| Pinwheel (Obsolete) | Pinwheel blades reveal the clip. ‡ | [VT][VT], [L][L], [RM][RM] (F) | missing | — | A union of blades. |
| Random Blocks (Obsolete) | Random blocks reveal the clip. ‡ | [VT][VT], [L][L], [RM][RM] (F) | missing | — | Seeded blocks. |
| Random Wipe (Obsolete) | Random blocks in a wipe direction. ‡ | [VT][VT], [L][L], [RM][RM] (F) | missing | — | Seeded blocks. |
| Spiral Boxes (Obsolete) | Boxes spiral in. ‡ | [VT][VT], [L][L], [RM][RM] (F) | missing | — | A union of boxes. |
| Venetian Blinds (Obsolete transition) | Blinds reveal the clip; Adobe's example uses it as a single-sided exit. ‡ | [VT][VT], [L][L], [RM][RM], [SS][SS] (F) | missing | — | A union of strips. |
| Wedge Wipe (Obsolete) | A pie wedge opens. ‡ | [VT][VT], [L][L], [RM][RM] (F) | partial | A keyed `path` mask (ADR-0163) | A one-angle wedge. |
| Zig-Zag Blocks (Obsolete) | Blocks reveal in a zig-zag. ‡ | [VT][VT], [L][L], [RM][RM] (F) | missing | — | A union of blocks. |
| Cube Spin (removed 26.0) | Clips on two faces of a rotating cube. ‡ | [L][L], [RM][RM] (F) | partial | Keyed `swivel` with `origin` at the shared edge, on two elements (ADR-0167) | No depth sorting (refused, ADR-0167), so the two faces' stacking is set by layer and may need a swap mid-turn. |
| Flip Over (removed 26.0) | The clip flips like a card to reveal the next. ‡ | [L][L], [RM][RM] (F) | have (skill) | `montagent-motion` card-flip recipe: front 0→180, back −180→0 (format.md; ADR-0167) | — |
| Gradient Wipe (transition; removed 26.0) | A wipe driven by a greyscale gradient. ‡ The current Gradient Wipe *effect* reads another track (see Transition effects). | [L][L], [RM][RM] (F) | missing | ADR-0152 keeps image-sourced masks deferred, not refused | A gradient-driven reveal. If the gradient is another element, ADR-0150 §4 refuses it. |

### Immersive Video transitions

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| VR Chroma Leaks | A chroma leak, seamless across the sphere. ‡ All immersive transitions detect a clip's VR properties. | [IMT][IMT], [VT][VT], [L][L] (F) | missing | — | Spherical model. |
| VR Gradient Wipe | A gradient-driven wipe across the sphere. ‡ | [IMT][IMT], [VT][VT], [L][L] (F) | missing | — | Spherical model. |
| VR Iris Wipe | An iris wipe on the sphere. ‡ | [IMT][IMT], [VT][VT], [L][L] (F) | missing | — | Spherical model. |
| VR Light Leaks | Light leaks across the sphere. ‡ | [IMT][IMT], [VT][VT], [L][L] (F) | missing | — | Spherical model. |
| VR Light Rays | Light rays across the sphere. ‡ | [IMT][IMT], [VT][VT], [L][L] (F) | missing | — | Spherical model. |
| VR Mobius Zoom | A Möbius zoom on the sphere. ‡ | [IMT][IMT], [VT][VT], [L][L] (F) | missing | — | Spherical model. |
| VR Random Blocks | Random blocks on the sphere. ‡ | [IMT][IMT], [VT][VT], [L][L] (F) | missing | — | Spherical model. |
| VR Spherical Blur | A blur transition on the sphere. ‡ | [IMT][IMT], [VT][VT], [L][L] (F) | missing | — | Spherical model. |

## Legacy versions of renamed effects and transitions

In 26.0 Adobe replaced these with Film Impact versions of the same name and kept the old ones
in the Legacy folder (26.5 did the same for Twirl, and 26.2.2 for Noise). Existing projects keep
the legacy version, so it still renders. Each has the status of its modern twin.

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Directional Blur (Legacy) | The pre-26.0 Directional Blur. | [L][L] (F) | have | `directional_blur` (ADR-0156) | — |
| Gaussian Blur (Legacy) | The pre-26.0 Gaussian Blur. | [L][L] (F) | partial | `blur` (ADR-0040) | One-axis blur and repeat edge pixels. |
| Linear Wipe (Legacy) | The pre-26.0 Linear Wipe effect. | [L][L] (F) | have | A keyed `mask` with `feather` (ADR-0152) | — |
| Magnify (Legacy) | The pre-26.3 Magnify. | [L][L] (F) | partial | As Magnify | As Magnify. |
| Mosaic (Legacy) | The pre-26.0 Mosaic. | [L][L] (F) | missing | — | A mosaic member. |
| Twirl (Legacy) | The pre-26.5 Twirl, "now under Legacy". | [RN][RN] (F) | missing | — | A twirl warp. |
| Noise (Legacy) | The pre-26.2.2 Noise. | [RN][RN] (F) | partial | `grain` (ADR-0156) | As Noise. |
| Additive Dissolve (Legacy) | The pre-26.0 Additive Dissolve. | [L][L] (F) | have | As Additive Dissolve | — |
| Clock Wipe (Legacy) | The pre-26.0 Clock Wipe. | [L][L], [CTS][CTS] (F) | partial | A keyed `path` mask (ADR-0163) | A one-angle pie mask. |
| Cross Dissolve (Legacy) | The pre-26.0 Cross Dissolve; old projects and FCP XML imports keep it. | [L][L], [RO][RO], [RN][RN] (F) | have | `kind: "crossfade"` (ADR-0059) | — |
| Cross Zoom (Legacy) | The pre-26.0 Cross Zoom. | [L][L] (F) | partial | Keyed `scale` and a `crossfade` | A radial zoom blur. |
| Dip to Black (Legacy) | The pre-26.0 Dip to Black. | [L][L] (F) | have | As Dip to Black | — |
| Dip to White (Legacy) | The pre-26.0 Dip to White. | [L][L] (F) | have | As Dip to White | — |
| Film Dissolve (Legacy) | The pre-26.0 Film Dissolve. | [L][L] (F) | missing | — | A linear-light crossfade. |
| Push (Legacy) | The pre-26.0 Push. | [L][L] (F) | have | `kind: "push"` (ADR-0150) | — |
| Radial Wipe (Legacy) | The pre-26.0 Radial Wipe. | [L][L] (F) | partial | A keyed `path` mask (ADR-0163) | A one-angle pie mask. |
| Slide (Legacy) | The pre-26.2.2 Slide. | [L][L] (F) | have | `kind: "slide"` (ADR-0150) | — |
| Split (Legacy) | The pre-26.0 Split. | [L][L] (F) | partial | Two masked copies with keyed offsets | Sections as one parameter. |
| Whip (Legacy) | The pre-26.0 Whip. | [L][L] (F) | have | `push` plus `motion_blur` (ADR-0150, ADR-0155) | — |

## Corrections and status changes

**Mechanisms corrected against the fetched pages** (51 rows): Levels (combines four tools);
Extract (input levels, greyscale); Gaussian Blur (Repeat Edge confirmed by the release notes);
Compound Blur (reads any timeline layer; hedge removed); Channel Blur (RGB/HSV/YUV); Lens
Distortion (simulates *or corrects*; the removed one is the older version); Magnify (rebuilt in
26.3); Twirl (rebuilt in 26.5); Gradient (texture and colour controls); Lens Flare (rebuilt in
26.5); Ramp (Ramp Scatter confirmed); Invert (Blend With Original, channel choices); Gamma
Correction (range); Channel Mix (RGB/HSV/YUV; Image Control bin); Rounded Crop, Clone, Auto
Align and Stroke (bins); Clone (offset-timed duplicates); Logo Cutout (= Alpha FX); Alpha Adjust
(an Opacity for render order; the alpha options are unconfirmed); Color Key (edge controls
unconfirmed); Ultra Key (full parameter list); Noise (blend modes named); Drop Shadow (Shadow
Only confirmed); Strobe Light (arithmetic operations); Crop (Fit/Fill, not zoom); Edge Feather
(a black border); Transform (shutter angle, motion path); the Gradient Wipe effect (current, not
removed); Radial Wipe and Venetian Blinds effects (removed in 26.0); the eight generators the
Generate page no longer lists (Cell Pattern, Checkerboard, Circle, Ellipse, Eyedropper Fill,
Grid, Paint Bucket, Write-On: removed in 26.0); Block Motion (blocks); Motion Camera (handheld
shake); Motion Tween (automatic tweening); Shape Flow, Stripe, Phosphor, Plateau Wipe, Panel
Wipe and Slice (now described); Soft Wipe (lens blur); Non-Additive Dissolve (Adobe's new
wording); effect masks (Rectangle, Ellipse, Polygon, Pen; no "4-point").

**Statuses changed** (15 rows):

| Row | Before | After | Why |
|---|---|---|---|
| Levels | missing | partial | The page defines it as Brightness & Contrast plus gamma, invert and colour balance; the first part ships. |
| Ramp (Legacy) | have | partial | Ramp Scatter is on the page and has no counterpart. |
| Drop Shadow | have | partial | Shadow Only is on the page and has no counterpart. |
| Strobe Light | have (skill) | partial | The page adds arithmetic operations such as invert, which do not ship. |
| Color Key | partial | have | The page's mechanism is a colour-similarity alpha key, which `chroma` is. The missing edge controls are not on the page. |
| Clone | missing | have | Now described: duplicates with offset timing, which copies with offset keyframes give. |
| Logo Cutout | missing | partial | It is the Alpha FX effect (transparency control); merged with the earlier Alpha row. |
| Motion Camera | refused | have (skill) | It is a handheld-shake simulation, not a 3D camera. |
| Motion Tween | partial | have | Automatic in-betweening of position, scale and rotation is keyframe interpolation. |
| Shape Flow | missing | partial | Now described as shape animations; shapes and path morphs ship, the generator does not. |
| Stripe | missing | have | Now described as bands of light; composed like Light Sweep. |
| Panel Wipe | missing | partial | Now described as sliding panels; composed from masked copies. |
| Soft Wipe | have | partial | It is a lens-blur transition; the lens kernel is missing. |
| Block Motion | have | partial | It breaks the frame into blocks; that takes N masked copies. |
| Non-Additive Dissolve | missing | have | Adobe's page now describes a crossfade "without brightening" (see Conflicts). |

Rows split from a grouped row keep the group's status, except Block Motion (above). The
Gradient Wipe effect stays `refused`, but the row now describes the current effect, not a
removed one.

## Conflicts in Adobe's pages

Recorded so the assembly does not resolve them silently:

- **Lens Distortion, Compound Blur, Noise, Channel Blur, Reduce Interlace Flicker, Radial Wipe
  and Venetian Blinds** each appear both as a current effect and in the 26.0 "Removed (will not
  render)" list. The release notes and the Modern effects page confirm the reading: the removed
  entry is the older implementation of the same name. Rows here follow the current entry.
- **Camera Blur** is Legacy (Windows only) in the 26.0 table and the Blur page, but Obsolete
  (macOS) on the removal page.
- **Lens Flare and Lightning** are on the Generate page but Obsolete in the 26.0 table. Lens
  Flare was then rebuilt as a modern effect in 26.5.
- **Radial Wipe**'s old name in the 26.0 table is "FI: Star Wipe Impacts", the same as Star
  Wipe's. One of the two is probably a typo.
- **Cross Zoom** is in the Zoom category on the transitions list and in Lights & Blurs in the
  26.0 table. **Soft Wipe** is in Wipe in the table and in Lights & Blurs on the Modern
  transitions page.
- **Non-Additive Dissolve**: the Dissolve page now says it crossfades "without brightening the
  result", which reads like a plain crossfade. The long-standing behaviour is that the brighter
  pixel of the two shows. The row follows Adobe's page.
- **Additive Dissolve**: the Dissolve page's description ("adds … then subtracts") does not
  describe an additive blend clearly. The row keeps the additive reading.
- **The accelerated list** puts Cross Dissolve, Push, Slide, Split, Whip, Wipe and Morph Cut
  under "obsolete", although all are current or Legacy. The list predates the 26.0 renames.

## Notable gaps

1. **No adjustment layer and no effect on a group.** Premiere's main way to grade or stylise
   many clips at once (and the page's advice for Morph Cut) has nothing to apply to. Group
   compositing is in the map's fog (ADR-0167 §10), behind nests (ADR-0166, #499).
2. **A mask scopes the whole element, never one effect, and nothing tracks it.** Regional
   blurs, face blurs, spot grades and mask vignettes all take a second masked copy and
   hand-keyed motion.
3. **Colour beyond four scalars.** Levels' gamma and invert, hue rotation, white balance,
   exposure, Gamma Correction, Invert, tonal-range wheels and hue-selective correction are
   missing. Curves, LUTs, channel mixing and duotone are refused (ADR-0049, ADR-0040). So
   Lumetri, Premiere's central colour tool, is only partly covered.
4. **No distortion or pixel-stylise filter.** Twirl, Wave Warp, Spherize, Lens Distortion,
   Turbulent Displace, Offset, Mosaic, Find Edges, Emboss, Sharpen and Unsharp Mask are all
   missing as members, and Adobe rebuilt Twirl and Mosaic again in 26.5. Pre-rendering outside
   the project is the only route.
5. **No analysis-driven effects.** Warp Stabilizer (with its framing and edge synthesis),
   Rolling Shutter Repair, Auto Reframe and Auto Reframe Sequence, Morph Cut and mask tracking
   all need motion or pixel analysis. Montagent has no analysis step.
6. **Transitions beyond four kinds rely on masks, and masks only intersect.** Every
   multi-shape pattern needs a union of shapes (checker, blinds, Louver, bands, random blocks,
   pinwheel, spiral, and the new Shape Dissolve). Border, Reverse and Start/End % are deferred
   (ADR-0150). There is no single-sided transition element (ADR-0059), no default transition or
   default duration, and no hold before a source's first frame.
7. **Keying stops at one chroma key.** Luma Key, Ultra Key's matte cleanup (choke, pedestal,
   highlight, shadow) and Alpha Adjust's placement in the order are missing. Track Matte Key,
   Compound Blur, the Gradient Wipe effect and every effect that reads another layer are
   refused (ADR-0150 §4).
8. **No immersive (360°/VR) model.** 11 effects and 8 transitions are all missing for the same
   reason.
9. **Pipeline properties.** No GPU path, no 32-bit (high-bit-depth) or YUV processing, no effect
   bypass flag or global FX mute, and no seeded randomise (Surprise Me) beyond `grain`.
10. **Effect controls conveniences.** No preset store or keyframe retiming, no effect-controls
    markers, and no metadata-driven text (Metadata & Timecode Burn-in). Source-clip effects have
    no home, because ADR-0002 rules out an asset table.

[L]: https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/list-of-effects-and-transitions.html
[RO]: https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/effects-and-transitions-reorganization.html
[RM]: https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/effects-and-transitions-removed-from-adobe-premiere.html
[MF]: https://helpx.adobe.com/premiere/desktop/add-video-effects/types-of-effects/effects.html
[MT]: https://helpx.adobe.com/premiere/desktop/add-video-effects/types-of-effects/transitions.html
[AN]: https://helpx.adobe.com/premiere/desktop/add-video-effects/types-of-effects/animations.html
[VF]: https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/video-effects.html
[ADJ]: https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/adjust-effects.html
[BS]: https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/blur-and-sharpen-effects.html
[CH]: https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/channel-effects.html
[CC]: https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/available-color-correction-effects.html
[LUT]: https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/add-look-up-tables.html
[RGBC]: https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/correct-color-using-rgb-curves.html
[HSC]: https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/correct-color-using-hue-and-saturation-curves.html
[LV]: https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/create-vignettes-using-lumetri-color.html
[VM]: https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/create-vignettes-using-masks.html
[DIS]: https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/distort-effects.html
[GEN]: https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/generate-effects.html
[IC]: https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/image-control-effects.html
[KEY]: https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/keying-effects.html
[UK]: https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/ultra-key-effect-parameters.html
[UKA]: https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/apply-and-customize-chromakey-using-the-ultra-key-effect.html
[NG]: https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/noise-and-grain-effects.html
[PER]: https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/perspective-effects.html
[STY]: https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/stylize-effects.html
[TIME]: https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/time-effects.html
[TR]: https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/transform-effects.html
[TE]: https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/transition-effects.html
[UT]: https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/utility-effects.html
[IMF]: https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/immersive-video-effects.html
[IMT]: https://helpx.adobe.com/premiere/desktop/edit-projects/edit-vr-content/immerse-video-effects-and-transitions.html
[VT]: https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/list-of-video-transitions.html
[VD]: https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/list-of-video-dissolve-transitions.html
[TO]: https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-transitions/transitions-overview.html
[SS]: https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-transitions/apply-single-sided-transitions.html
[TD]: https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-transitions/change-transition-duration-using-the-effect-controls-panel.html
[ALN]: https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-transitions/align-transitions.html
[CTS]: https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-transitions/change-transition-settings.html
[HAND]: https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-transitions/clip-handles-settings.html
[CPT]: https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-transitions/copy-and-paste-transitions.html
[RPT]: https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-transitions/replace-transitions.html
[MOV]: https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-transitions/move-cuts-and-transitions-simultaneously.html
[RC]: https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-transitions/reposition-the-center-of-a-transition.html
[DT]: https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-transitions/set-and-apply-default-transitions.html
[VTH]: https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-transitions/video-transitions-using-clip-handles.html
[MCO]: https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-transitions/morph-cut-overview.html
[MCA]: https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-transitions/apply-morph-cut-to-smoothen-jump-cuts.html
[MC]: https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-transitions/morph-cut-transition-options.html
[ECP]: https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-effects/about-effect-controls-panel.html
[ECV]: https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-effects/view-effects-in-the-effect-controls-panel.html
[ADJC]: https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-effects/adjust-controls-in-the-effect-controls-panel.html
[TYP]: https://helpx.adobe.com/premiere/desktop/add-video-effects/types-of-effects/types-of-effects.html
[ABT]: https://helpx.adobe.com/premiere/desktop/add-video-effects/types-of-effects/about-effects-and-effects-panel.html
[APR]: https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-effects/apply-effect-presets.html
[CPR]: https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-effects/create-effect-presets.html
[EP]: https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-effects/effect-presets.html
[AE]: https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-effects/apply-effects.html
[CPE]: https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-effects/copy-and-paste-clip-effects.html
[REM]: https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-effects/remove-effects-from-a-clip.html
[FX]: https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-effects/use-fx-badges.html
[SRC]: https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-effects/about-source-clips.html
[ASRC]: https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-effects/apply-effects-to-source-clips.html
[THB]: https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-effects/enable-or-disable-source-clips-effects-in-thumbnails.html
[AL]: https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-effects/create-adjustment-layers.html
[RAL]: https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-effects/resize-adjustment-layers.html
[MGR]: https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-effects/manage-effects-and-plug-ins.html
[FG]: https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-effects/find-and-group-effects.html
[ACC]: https://helpx.adobe.com/premiere/desktop/add-video-effects/types-of-effects/list-of-accelerated-effects.html
[B32]: https://helpx.adobe.com/premiere/desktop/add-video-effects/types-of-effects/list-of-32-bit-effects.html
[YUV]: https://helpx.adobe.com/premiere/desktop/add-video-effects/types-of-effects/list-of-yuv-effects.html
[AR]: https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/auto-reframe-overview.html
[ARC]: https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/add-auto-reframe-effect-to-a-clip.html
[ARS]: https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/add-auto-reframe-effect-to-a-sequence.html
[WS]: https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/stabilize-shaky-footage-using-warp-stabilizer.html
[WSS]: https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/warp-stabilizer-settings.html
[MO]: https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/apply-motion-effect.html
[VMO]: https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/edit-vector-graphics-using-vector-motion-effect.html
[LIT]: https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/add-lightning-effects.html
[JL]: https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/create-a-jacobs-ladder-effect.html
[BV]: https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/create-black-video-clips.html
[CM]: https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/create-color-matte-clips.html
[TV]: https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/create-transparent-video-clips.html
[BT]: https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/calibrate-audio-and-video-using-bars-and-tone.html
[CL]: https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/create-a-counting-leader.html
[FI]: https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/create-fade-in-video-effects.html
[KF]: https://helpx.adobe.com/premiere/desktop/add-video-effects/control-effects-and-transitions-using-keyframes/about-keyframes.html
[KFI]: https://helpx.adobe.com/premiere/desktop/add-video-effects/control-effects-and-transitions-using-keyframes/change-keyframe-interpolation-method.html
[MSK]: https://helpx.adobe.com/premiere/desktop/add-video-effects/create-masks-and-composites/create-masks-using-shapes.html
[MTK]: https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/mask-tracking-tools.html
[ECM]: https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/create-markers-in-the-effect-controls-panel.html
[PLG]: https://helpx.adobe.com/premiere/desktop/use-premiere-with-other-apps/plug-ins.html
[PERF]: https://helpx.adobe.com/premiere/desktop/troubleshooting/playback-issues/not-able-to-optimize-playback-performance.html
[RN]: https://helpx.adobe.com/premiere/desktop/whats-new/release-notes.html
[WN]: https://helpx.adobe.com/premiere/desktop/whats-new/whats-new.html

## Pages covered

Every census page assigned to this area, and the three What's-new pages.

| Page URL | Rows / "no feature: <reason>" |
|---|---|
| https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-effects/about-effect-controls-panel.html | Fixed (intrinsic) effects; Effect Controls panel views; Time Remapping |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-effects/about-source-clips.html | Source clip effects |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-effects/adjust-controls-in-the-effect-controls-panel.html | Adjusting a control; Eyedropper and colour picker; Reset an effect |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-effects/apply-effect-presets.html | Effect presets: built-in and custom |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-effects/apply-effects-to-source-clips.html | Source clip effects |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-effects/apply-effects.html | Apply an effect |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-effects/copy-and-paste-clip-effects.html | Copy and paste effects; Paste Attributes |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-effects/create-adjustment-layers.html | Adjustment layers |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-effects/create-effect-presets.html | Effect presets; Preset keyframe retiming |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-effects/effect-presets.html | Effect presets: built-in and custom |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-effects/enable-or-disable-source-clips-effects-in-thumbnails.html | Source-clip effects shown on Project panel thumbnails |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-effects/find-and-group-effects.html | Finding and grouping effects |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-effects/manage-effects-and-plug-ins.html | Effect Manager |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-effects/remove-effects-from-a-clip.html | Remove effects; Remove Attributes; Fixed (intrinsic) effects |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-effects/resize-adjustment-layers.html | Resize an adjustment layer |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-effects/use-fx-badges.html | FX badges; Source clip effects |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-effects/view-effects-in-the-effect-controls-panel.html | Standard effect stack order; Fixed (intrinsic) effects |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-transitions/align-transitions.html | Alignment: Center at Cut, Start at Cut, End at Cut, Custom Start |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-transitions/apply-morph-cut-to-smoothen-jump-cuts.html | Morph Cut; Default transition duration |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-transitions/apply-single-sided-transitions.html | Single-sided transition; Venetian Blinds (Obsolete transition) |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-transitions/change-transition-duration-using-the-effect-controls-panel.html | Transition duration; Alignment; Default transition duration |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-transitions/change-transition-settings.html | Edge selectors; Transition settings (Start/End %, border, Reverse, anti-aliasing, Custom); Show Actual Sources; Barn Doors |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-transitions/clip-handles-settings.html | A transition between two clips; Clip handles and insufficient media |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-transitions/copy-and-paste-transitions.html | Copy and paste transitions; Replace a transition |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-transitions/morph-cut-overview.html | Morph Cut; Adjustment layers |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-transitions/morph-cut-transition-options.html | Morph Cut; Transition duration; Alignment |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-transitions/move-cuts-and-transitions-simultaneously.html | Move a cut and its transition together |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-transitions/replace-transitions.html | Replace a transition |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-transitions/reposition-the-center-of-a-transition.html | Reposition a transition's centre; Iris Round (Legacy) |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-transitions/set-and-apply-default-transitions.html | Default transition; Apply default transitions to a selection; Default transition duration; Audio crossfades |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-transitions/transitions-overview.html | A transition between two clips; Single-sided transition; Clip handles; Audio crossfades |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-transitions/video-transitions-using-clip-handles.html | Video transition handles |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/add-auto-reframe-effect-to-a-clip.html | Auto Reframe (clip) |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/add-auto-reframe-effect-to-a-sequence.html | Auto Reframe Sequence |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/add-lightning-effects.html | Lightning; Transparent Video |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/apply-motion-effect.html | Motion effect: position, scale and rotation |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/auto-reframe-overview.html | Auto Reframe (clip); Auto Reframe Sequence |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/calibrate-audio-and-video-using-bars-and-tone.html | Bars and Tone |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/create-a-counting-leader.html | Universal Counting Leader |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/create-a-jacobs-ladder-effect.html | Lightning, including the Jacob's Ladder look |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/create-black-video-clips.html | Black Video |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/create-color-matte-clips.html | Color Matte |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/create-fade-in-video-effects.html | Fade in from black |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/create-transparent-video-clips.html | Transparent Video |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/create-vignettes-using-lumetri-color.html | Lumetri vignette; Lumetri Color |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/create-vignettes-using-masks.html | Mask-based vignette; Effect masks |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/edit-vector-graphics-using-vector-motion-effect.html | Vector Motion |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/effects-and-transitions-removed-from-adobe-premiere.html | Every row in Video effects removed in 26.0 or earlier; Obsolete and removed transitions; Legacy, Obsolete and Removed effects; Camera Blur (Legacy) |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/stabilize-shaky-footage-using-warp-stabilizer.html | Warp Stabilizer; Effects on a nested sequence |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/warp-stabilizer-settings.html | Warp Stabilizer; Warp Stabilizer border framing; Warp Stabilizer Rolling Shutter Ripple |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/adjust-effects.html | Extract; Levels; Lighting Effects; ProcAmp |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/apply-and-customize-chromakey-using-the-ultra-key-effect.html | Ultra Key; Eyedropper and colour picker |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/blur-and-sharpen-effects.html | Gaussian Blur; Directional Blur; Reduce Interlace Flicker; Sharpen; Unsharp Mask; Camera Blur (Legacy) |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/channel-effects.html | Invert |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/color-correction-effects.html | no feature: one-paragraph introduction to the Color Correction bin, naming no effect (the bin's rows cite the colour chapter's Available color correction effects page) |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/distort-effects.html | Corner Pin; Lens Distortion; Magnify; Mirror; Offset; Spherize; Turbulent Displace; Twirl; Wave Warp; Rolling Shutter Repair; Warp Stabilizer |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/effects-and-transitions-reorganization.html | Legacy, Obsolete and Removed effects; Cross Dissolve; Default transition; Invert; Offset |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/generate-effects.html | 4-Color Gradient; Lens Flare; Lightning; Ramp (Legacy) |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/image-control-effects.html | Black & White; Color Pass; Color Replace; Gamma Correction |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/immersive-video-effects.html | VR Blur through VR Sharpen (11 Immersive Video rows) |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/keying-effects.html | Alpha Adjust; Color Key; Luma Key; Track Matte Key; Ultra Key; Moving a geometry step into the stack |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/list-of-effects-and-transitions.html | Every effect and transition row (bins, Legacy versions, Obsolete, Removed in 26.0) |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/list-of-video-dissolve-transitions.html | Additive Dissolve; Cross Dissolve; Dip to Black; Dip to White; Film Dissolve; Morph Cut; Non-Additive Dissolve (Legacy) |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/list-of-video-transitions.html | Every row in Legacy transitions, Obsolete and removed transitions and Immersive Video transitions; Push; Slide; Split; Whip; Cross Zoom; Radial Wipe |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/noise-and-grain-effects.html | Noise |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/perspective-effects.html | Basic 3D; Drop Shadow |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/stylize-effects.html | Alpha Glow; Brush Strokes; Color Emboss; Find Edges; Mosaic; Posterize; Replicate; Roughen Edges; Strobe Light |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/time-effects.html | Echo; Posterize Time |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/transform-effects.html | Auto Reframe (clip); Crop; Edge Feather; Horizontal Flip; Transform; Vertical Flip |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/transition-effects.html | Block Dissolve; Gradient Wipe (effect); Linear Wipe |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/ultra-key-effect-parameters.html | Ultra Key |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/utility-effects.html | Cineon Converter |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/video-effects.html | Metadata & Timecode Burn-in; SDR Conform; Simple Text |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/types-of-effects/about-effects-and-effects-panel.html | Keyframing effect parameters; Effect presets |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/types-of-effects/animations.html | Block, Flip, Fold, Pop, Pull, Spin, Spring and Travel Motion; Typewriter |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/types-of-effects/effects.html | Every Film Impact effect row (Tools, Lights and Glows, Distort, Essentials, Motion); Gradient; Channel Blur; Noise; Surprise Me; GPU-accelerated effects |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/types-of-effects/list-of-32-bit-effects.html | 32-bit color (high-bit-depth) effects |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/types-of-effects/list-of-accelerated-effects.html | GPU-accelerated effects; Gradient Wipe (effect); Basic 3D |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/types-of-effects/list-of-yuv-effects.html | YUV effects |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/types-of-effects/transitions.html | Every Film Impact transition row (Essentials, Transformers, Animation, Lights & Blurs, Smart Tools); Shape Dissolve; 3D Spinback; Slide; Easing a transition |
| https://helpx.adobe.com/premiere/desktop/add-video-effects/types-of-effects/types-of-effects.html | Fixed (intrinsic) effects; Render order; Moving a geometry step into the stack; Clip-based video effects; Effects on a nested sequence; Third-party plug-ins; GPU-accelerated, 32-bit and YUV effects |
| https://helpx.adobe.com/premiere/desktop/whats-new/beta-features.html | no feature: the beta page's two items (AI Assistant, Color mode) belong to the AI and colour areas |
| https://helpx.adobe.com/premiere/desktop/whats-new/release-notes.html | Effects items: Lens Flare, Twirl and Twirl (Legacy), Mosaic (26.5); Surprise Me seed; Gaussian Blur Repeat Edge; Transform motion path; transition handles show/hide; Shape Dissolve, Magnify (26.3); Channel Blur, Gradient, Noise, 3D Spinback, Slide (26.2.2); Auto Reframe Sequence resolution; Film Impact Dashboard; Multi Transitions; Noise (Legacy); Cross Dissolve (Legacy); third-party plug-ins; Lighting Effects; Dip to Black |
| https://helpx.adobe.com/premiere/desktop/whats-new/whats-new.html | Effects items: Lens Flare, Twirl, Mosaic (26.5); Shape Dissolve, Magnify (26.3); Channel Blur, Gradient, Noise, 3D Spinback, Slide (26.2); 90+ effects and transitions, animation tools, video transition handles, Multi Transitions (26.0) |
