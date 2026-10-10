# After Effects census: compositions, layers and compositing

Research for [#898](https://github.com/MBehtemam/Montagent/issues/898), on the map
[#896](https://github.com/MBehtemam/Montagent/issues/896). It lists what After Effects documents
about compositions, layers and compositing, one feature per row, and scores each one against
Montagent at `main` `51aa4b5`.

## Sources and version

- **AE version read:** After Effects **26.5**, the September 2026 release. Adobe's
  [What's new][wn] page names it as current (updated 18 Sep 2026).
- **Primary source:** Adobe's After Effects user guide on helpx.adobe.com. The pages used are
  [Composition basics][cb], [Precomposing, nesting, and pre-rendering][pc], [Creating layers][cl],
  [Managing layers][ml], [Selecting and arranging layers][sa], [Layer properties][lp],
  [Blending modes and layer styles][bm], [Track mattes and traveling mattes][tm],
  [Layer markers and composition markers][mk], [Responsive design][rd],
  [Time-stretching and time-remapping][ts], [Cameras, lights, and points of interest][cam],
  [Enable lights to cast shadows][shd], [Assorted animation tools][mb] (motion blur),
  [Previewing][pv], [Modifying and using views][vw], [Create Nulls for positional
  properties and paths][nc] and [Composition Profiler][cpr].
- **How the pages were read:** the first draft was written from search-engine extracts while
  helpx.adobe.com was blocked. This file was then **verified against the live pages on
  2026-10-10**: every row's link was opened, followed through its redirects and replaced
  with the page that documents the feature today; every description was checked against
  Adobe's text; and the user guide's table of contents for compositions, layers and
  compositing was walked for features the draft missed. No row is left unverified, and no
  secondary source decides a row.
- **Montagent evidence:** `CONTEXT.md`, `docs/adr/` (grepped for every feature before writing
  "gap"), `crates/montagent-core/docs/format.md` and its pages (`format/compositing.md`
  above all), `schema/montagent.schema.json`, `skills/montagent/SKILL.md` (the capability map)
  and the other `skills/*`.
- **Not in this file:** the detail of shape layers ([#901](https://github.com/MBehtemam/Montagent/issues/901)),
  text ([#900](https://github.com/MBehtemam/Montagent/issues/900)), 3D, cameras and lights
  ([#902](https://github.com/MBehtemam/Montagent/issues/902)), keyframes and motion paths
  ([#899](https://github.com/MBehtemam/Montagent/issues/899)), the colour effects and colour
  management ([#904](https://github.com/MBehtemam/Montagent/issues/904)), expressions
  ([#906](https://github.com/MBehtemam/Montagent/issues/906)) and import/export, including PSD
  and AI layers and marker export ([#907](https://github.com/MBehtemam/Montagent/issues/907)).
  Each of those layer types gets one row here and points on.

## Statuses

- **covered**: Montagent reaches the result with literal values in the format.
- **partial**: part of the feature is reached and the rest is not.
- **reachable**: reached by a skill script or a documented workaround, not by the format.
- **#591** / **#795**: owned by an open map, either
  [the transforms map](https://github.com/MBehtemam/Montagent/issues/591) (nest, free pivot,
  swaps) or [the audio map](https://github.com/MBehtemam/Montagent/issues/795).
- **refused (ADR-NNNN)**: a recorded decision says no. "For now" marks a refusal the ADR
  itself leaves open to a forcing case.
- **gap**: nothing in the repo reaches it or rules on it.
- **app**: a GUI or workflow feature, which #896 puts out of scope. Montagent has no GUI, and
  the agent stands in for the app.

## 1. Composition settings

| Feature | What it does | Adobe doc | Montagent status | Evidence |
|---|---|---|---|---|
| Frame size | The composition's width and height in pixels | [Composition basics][cb] | covered | The project's `frame` `{width, height}` (schema `Frame`) |
| Pixel aspect ratio | Non-square pixels for the composition | [Composition basics][cb] | partial | The project is square-pixel. A source's own PAR is resolved on read ([ADR-0023](../../adr/0023-video-source-dimensions-par-and-rotation.md)), but a non-square output cannot be written |
| Frame rate | The composition's frames per second, fractional rates included (29.97, 23.976) | [Composition basics][cb] | partial | `fps` is an integer in the schema, so 30000/1001 and 24000/1001 cannot be written. Fractional *sources* are handled ([ADR-0096](../../adr/0096-the-frame-at-an-instant-is-the-last-one-starting-at-or-before-it.md)) |
| Resolution (Full, Half, Third, Quarter) | Renders preview pixels at a fraction of the frame | [Modifying and using views][vw] | covered | Proxy tiers for `preview` and `frame` ([ADR-0046](../../adr/0046-proxy-preview-target-is-720p-long-edge-capped.md), [ADR-0186](../../adr/0186-a-proxy-tier-reads-a-shrunk-image-through-one-mip-level-and-the-deliverable-keeps-adr-0132s-rule.md)). `render` is always full size |
| Start timecode | Labels the composition's first frame with a timecode other than 0 | [Composition basics][cb] | gap | Every time is integer ms from 0 ([ADR-0005](../../adr/0005-absolute-integer-milliseconds.md)). `CONTEXT.md` **Timeline range** lists "timecode" under _Avoid_. No ADR rules on a display offset |
| Duration | The composition's length | [Composition basics][cb] | covered | The project's optional `duration` (format.md, ADR-0030) |
| Background colour | The colour behind every layer | [Composition basics][cb] | covered | The project's `background` (format/compositing.md: "for a gradient background, put a full-frame `rect`…") |
| Work area | The span of the composition that previews and renders | [Previewing][pv] | covered | `render --from --to`, `preview` and `frame --from --to` ([ADR-0097](../../adr/0097-the-range-is-from-to-on-both-surfaces-and-the-caption-becomes-an-attribution-obligation.md), [ADR-0121](../../adr/0121-a-partial-render-s-world-effects-stop-at-its-range-and-its-report-says-so.md)), or editing `duration` |
| Region of interest / crop comp to ROI | Previews, or crops the composition or the output to, a rectangle | [Previewing][pv] | covered | `frame`'s true-scale crop for looking ([ADR-0101](../../adr/0101-a-crop-is-served-at-true-scale.md), [ADR-0103](../../adr/0103-the-sheet-is-never-cropped-and-the-crop-stays-a-single-frame-instrument.md)). To crop the output, edit `frame` and the elements' `x`/`y` |
| Preserve frame rate / resolution when nested | A nested composition keeps its own frame rate or resolution inside its parent | [Precomposing][pc] | #591 | A nest is "space-only… it has no clock" (#632's ruling on #591), and [ADR-0166](../../adr/0166-capcut-and-premiere-nest-with-a-transform-and-adr-0001-refuses-the-clock-not-the-shared-space.md) §2 refuses any local clock |
| Motion blur settings: shutter angle, shutter phase, samples per frame, adaptive sample limit | Composition-wide motion blur controls (Advanced tab) | [Composition basics][cb] | partial | Per-element `motion_blur` `{shutter, samples}` ([ADR-0155](../../adr/0155-motion-blur-is-a-per-element-field-that-accumulates-the-element-over-a-centred-shutter.md)). ADR-0155 refuses a phase and a project-wide setting ("there is no `phase` and no project-wide setting") |
| 3D renderer (Classic 3D, Cinema 4D, Advanced 3D) | Chooses the engine that draws 3D layers, lights and shadows | [Composition basics][cb] | refused (ADR-0167) | "An element may be projected, never placed": no depth, no sorting, no camera ([ADR-0167](../../adr/0167-an-element-may-be-projected-never-placed-swivel-tilt-and-perspective-join-the-transform.md)). Detail goes to #902 |
| Blend colours using 1.0 gamma (linear blending) | Blends in linear light instead of gamma-encoded values | [Blending modes][bm] | refused (ADR-0147) | "Linear-light arithmetic. Refused" ([ADR-0147](../../adr/0147-a-blend-mode-is-a-flat-static-field-of-five-values-and-the-finished-element-blends-last.md)). Colour management goes to #904 |
| Crop comp to selected layer bounds | Shrinks or grows the composition to the visible bounds of the selected 2D layers (26.2) | [Composition basics][cb] | covered | Edit `frame` and shift every element's `x`/`y` by the same offset: literal values. No verb computes the bounds; `frame`'s report and the declared boxes give them |
| Advanced: Anchor | Pins layers to a corner or edge while the composition is resized | [Composition basics][cb] | reachable | The agent shifts `x`/`y` when it edits `frame`. Every element is placed in frame pixels ([ADR-0013](../../adr/0013-fitted-extents-floor-and-the-nine-origin-keywords.md)), so there is nothing to pin |
| Scale Composition script | Scales a whole composition and its layers by one factor | [Composition basics][cb] | reachable | Edit `frame`, every element's box, `x`/`y` and size keyframes. No shipped script does it |
| Composition presets | Fills frame size, pixel aspect and frame rate from a named preset, social formats included | [Composition basics][cb] | app | A dialog. The agent writes `frame` and `fps` itself; `skills/montagent/references/starter-project.md` is the nearest thing to a preset |
| Poster time (composition thumbnail) | Picks the frame shown as the composition's thumbnail in the Project panel | [Composition basics][cb] | app | No Project panel. `frame` shows any instant on demand |
| Composition Navigator, Mini-Flowchart and Flowchart | Moves around a network of nested compositions | [Precomposing][pc] | app | GUI only. A nest is listed by `timeline` (ADR-0031); nesting itself is #591 |
| Composition Profiler | Shows how long each layer and effect takes to render | [Composition Profiler][cpr] | app | A panel. `render` reports its own findings (ADR-0011), but nothing times each element |

## 2. Layer types

| Feature | What it does | Adobe doc | Montagent status | Evidence |
|---|---|---|---|---|
| Footage layer (video, still image) | Places an imported file in the composition | [Creating layers][cl] | covered | `video` and `image` elements with an inline `source` ([ADR-0002](../../adr/0002-inline-source-no-asset-table.md)) |
| Audio layer | Places a sound file, with levels and waveform | [Creating layers][cl] | #795 | The `audio` element exists (`volume`, `audio_effects`, capability map "Audio"). The rest of AE's audio belongs to the audio map |
| Solid layer | A flat-coloured rectangle of any size | [Creating layers][cl] | covered | A `rect` with `fill`. A full-frame one is the documented gradient-background recipe (format/compositing.md) |
| Null object | An invisible layer used as a parent or as a control layer for other layers | [Layer properties][lp] | #591 | Its job is parenting, which is the nest's job on #591. Capability map, "What it can't do yet": "Parenting, groups and cameras… (#499)" |
| Adjustment layer | Applies its effects to the composite of every layer beneath it | [Creating layers][cl] | gap | An effect filters its own element's pixels ([ADR-0040](../../adr/0040-effect-model-attachment-and-v1-vocabulary.md), `CONTEXT.md` **Effect**). #632 refused "the adjustment-layer shape" only as the group transform's shape, not as an effects carrier. Only the blend-mode texture recipe comes near (capability map: "a `rect` carrying an effect and a blend") |
| Shape layer | Vector shapes with fills, strokes and path operators | [Shape attributes][sh] | partial | `rect`, `ellipse` and `path`, with dash, trim and join ([ADR-0154](../../adr/0154-a-point-list-enters-as-one-path-element-in-integer-pixels-from-the-declared-box.md), [ADR-0158](../../adr/0158-a-path-chooses-its-stroke-join-and-cap-and-every-shape-takes-a-dash-pattern.md), [ADR-0160](../../adr/0160-a-stroke-draws-a-window-of-its-outline-measured-in-fractions-of-its-length.md)). Repeater refused ([ADR-0148](../../adr/0148-a-repeat-does-not-enter-and-a-stagger-enters-only-across-the-units-of-one-text-element.md)). Operator detail goes to #901 |
| Text layer | Point and paragraph text with animators | [Creating layers][cl] | covered | The `text` element (format/text.md). Animator detail goes to #900 |
| Camera layer | Views 3D layers from a moving point with lens and depth of field | [Cameras, lights][cam] | refused (ADR-0167) | "Skew, corner pin, a camera, depth sorting. Still refused" ([ADR-0167](../../adr/0167-an-element-may-be-projected-never-placed-swivel-tilt-and-perspective-join-the-transform.md)). #591 treats a camera move as a group transform over everything |
| Light layer (parallel, spot, point, ambient, environment) | Lights 3D layers and casts shadows | [Cameras, lights][cam], [Shadows][shd] | gap | Nothing names lights. A light needs placed 3D layers, which ADR-0167 refuses, so it probably falls with that refusal. Detail goes to #902 |
| Precomposition layer | A composition used as a layer in another | [Precomposing][pc] | #591 | The `nest` (#632's ruling, prototype #780, score #781). Capability map: "Parenting, groups and cameras" |
| Guide layer | A reference layer shown while you work and left out of the render unless the render settings include it | [Layer properties][lp] | gap | Every element paints, and the format has no render-excluded element. [ADR-0150](../../adr/0150-wipe-slide-and-push-enter-as-transition-kinds-and-a-matte-from-another-element-is-refused.md) §4 counts "an element… in a track without painting" against a matte |
| Stacking order | Which layer draws in front, changed by dragging or Bring Forward / Send Backward | [Selecting and arranging layers][sa] | covered | `layer` integers, on the track or the element, and one-hop anchors such as `{"below": "title"}` (format.md "Tracks, elements and stacking", [ADR-0019](../../adr/0019-layer-anchor-gets-an-id-a-validate-check-and-one-hop.md), [ADR-0060](../../adr/0060-layer-tie-is-an-error-array-order-stays-meaningless.md)) |
| Duplicate or copy a layer | Copies a layer with its properties and keyframes | [Selecting and arranging layers][sa] | covered | Copy the element's record under a new `id`. Edits are file edits ([ADR-0011](../../adr/0011-tool-surface-reads-checks-renders.md)) |
| Replace layer source | Swaps the footage under a layer and keeps its properties | [Creating layers][cl] | covered | Edit the element's inline `source` ([ADR-0002](../../adr/0002-inline-source-no-asset-table.md)) |

## 3. Layer transform properties

| Feature | What it does | Adobe doc | Montagent status | Evidence |
|---|---|---|---|---|
| Anchor point | The point the layer is positioned by and transforms about, anywhere | [Layer properties][lp] | partial | Nine `origin` keywords ([ADR-0013](../../adr/0013-fitted-extents-floor-and-the-nine-origin-keywords.md)), the same nine points AE's beta Quick Set Anchor offers. The free pivot `"origin": [px, py]` belongs to #591 (#596, #822) |
| Position, Scale, Rotation, Opacity | The layer's 2D transform, all keyable | [Layer properties][lp] | covered | `x`, `y`, `scale`, `rotation`, `opacity` ([ADR-0012](../../adr/0012-flat-transform-keyframes-carried-by-their-element.md), [ADR-0146](../../adr/0146-an-animatable-property-is-one-the-schema-types-so-colour-blends-premultiplied-and-spring-easing-is-refused.md)) |
| Separate dimensions | Keys X and Y position on their own | [Selecting and arranging layers][sa] | covered | `x` and `y` are separate animatable fields (format.md "Time") |
| Auto-orient along path | Turns the layer to face along its motion path (the 3D options face the camera or a point of interest, #902) | [Selecting and arranging layers][sa] | gap | Montagent has no spatial motion path (`x` and `y` are keyed apart), and no ADR mentions orientation. Motion paths go to #899 |
| Center anchor point in layer content | Moves the anchor to the centre of the layer's visible content, keeping the layer in place | [Layer properties][lp] | partial | `origin: "center"` is the centre of the declared box, which is the content for shapes and fitted media. Text's or a masked region's visible centre is not computed, and re-placing `x`/`y` is the agent's arithmetic |
| Flip horizontal / vertical | Multiplies one scale component by −1, about the anchor | [Layer properties][lp] | covered | `scale: [-1, 1]` (format.md "Geometry": "a mirrored back is a second element… with `scale: [-1, 1]`") |
| Fit to comp, fit to comp width or height | Scales and centres a layer to the frame, keeping or ignoring its aspect | [Layer properties][lp] | covered | The agent computes `width`/`height`; `fit: "cover"`/`"contain"` records the claim and `validate` checks the arithmetic ([ADR-0015](../../adr/0015-fit-is-a-derivation-claim-and-gravity-retires.md)) |
| Align and distribute (Align panel) | Lines up or evenly spaces layers to the selection or the composition | [Selecting and arranging layers][sa] | reachable | The agent writes `x`/`y` with `origin` keywords ([ADR-0013](../../adr/0013-fitted-extents-floor-and-the-nine-origin-keywords.md)). No shipped script distributes |

## 4. Layer switches

| Feature | What it does | Adobe doc | Montagent status | Evidence |
|---|---|---|---|---|
| Video switch (eye) | Hides a layer from previews and renders | [Managing layers][ml] | reachable | Remove the element or set `opacity: 0`. Edits are file edits ([ADR-0011](../../adr/0011-tool-surface-reads-checks-renders.md)) |
| Audio switch | Mutes a layer | [Managing layers][ml] | reachable | `volume: 0` (`CONTEXT.md` **Volume**) |
| Solo | Previews and renders only the soloed layers of that type | [Managing layers][ml] | reachable | `frame` and `preview` have no element filter (`crates/montagent/src/cli.rs`). Look at a copy of the project with the other elements removed |
| Lock | Stops a layer from being selected or edited | [Managing layers][ml] | app | GUI only. Edits go through file tools ([ADR-0011](../../adr/0011-tool-surface-reads-checks-renders.md)) |
| Shy / Hide Shy Layers | Hides layers from the Timeline panel | [Managing layers][ml] | app | GUI only. `timeline` is a flat listing ([ADR-0031](../../adr/0031-timeline-overview-is-not-required-to-be-spatial.md)) |
| Label colour and comment | Tags layers for organising | [Managing layers][ml] | covered | `group`, a render-inert label (`CONTEXT.md` **Group**) |
| Collapse transformations (precomp) | Draws a nested composition's layers straight into the parent, in its 3D space and blend context | [Managing layers][ml] | #591 | A nest is "a pure matrix". Group compositing ("a nest that composites its children as one") is in #591's _Not yet specified_ |
| Continuously rasterize (vector, text, shape) | Rasterises vectors after the transform, so they stay sharp when scaled | [Managing layers][ml] | covered | Shapes, paths and text are drawn through the element's transform. The renderer scales the canvas before drawing (`crates/montagent-render/src/canvas.rs`, [ADR-0012](../../adr/0012-flat-transform-keyframes-carried-by-their-element.md)) |
| Quality and sampling (Best/Draft/Wireframe, bilinear/bicubic) | Trades image quality for speed, and picks the resampling filter | [Managing layers][ml] | covered | One fixed sampling rule ([ADR-0132](../../adr/0132-an-image-is-sampled-by-what-its-draw-does-to-it.md)). Draft pixels are the proxy tiers ([ADR-0186](../../adr/0186-a-proxy-tier-reads-a-shrunk-image-through-one-mip-level-and-the-deliverable-keeps-adr-0132s-rule.md)) |
| Effects switch (fx) | Turns all of a layer's effects on or off | [Managing layers][ml] | partial | `"enabled": false` bypasses an `audio_effects` member ([ADR-0169](../../adr/0169-audio-effects-are-an-ordered-list-before-volume-and-gain-and-routing-stay-flat.md)). Visual `effects` members have no bypass (schema `Effect`) |
| Frame blending (Frame Mix, Pixel Motion) | Builds in-between frames for footage played at another rate | [Time-stretching][ts] | gap | A video shows the last source frame at or before the instant ([ADR-0096](../../adr/0096-the-frame-at-an-instant-is-the-last-one-starting-at-or-before-it.md), [ADR-0157](../../adr/0157-a-speed-ramp-is-a-time-remap-curve-of-source-times-on-a-video-element.md)). No ADR mentions blending frames. Listed among #896's candidates |
| Motion blur switch | Turns motion blur on for one layer | [Motion blur][mb] | covered | `motion_blur` on `rect`, `ellipse`, `path`, `text`, `image`, `video` ([ADR-0155](../../adr/0155-motion-blur-is-a-per-element-field-that-accumulates-the-element-over-a-centred-shutter.md)) |
| Adjustment layer switch | Turns any layer into an adjustment layer | [Managing layers][ml] | gap | Same as the adjustment layer in §2 |
| 3D layer switch | Gives a layer a Z position, orientation and material options | [Cameras, lights][cam] | partial | A per-element projection: `swivel`, `tilt`, `perspective` ([ADR-0167](../../adr/0167-an-element-may-be-projected-never-placed-swivel-tilt-and-perspective-join-the-transform.md)). Depth, sorting and materials are refused by the same ADR |

## 5. Parenting

| Feature | What it does | Adobe doc | Montagent status | Evidence |
|---|---|---|---|---|
| Parenting (Parent & Link column) | A child layer takes its parent's position, scale and rotation (not opacity) | [Layer properties][lp] | #591 | The nest is the container form. A sibling parent link was ruled AE-only and out of scope by #594, which [#897](https://github.com/MBehtemam/Montagent/issues/897) may re-read. Today it is baked by `skills/montagent-character/scripts/bake_rig.py` (capability map, workaround #499) |
| Parent pick whip | Sets a parent by dragging to it | [Layer properties][lp] | #591 | Same as parenting. The pick whip is its GUI |
| Property pick whip (link a value to another) | Writes an expression that ties one property to another | [Layer properties][lp] | refused (ADR-0145) | Fails the literal-values invariant ([ADR-0145](../../adr/0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md)). Capability map "Not by design": "Keyframe expressions, or links between values" |
| Null controllers for positional properties and path points | Creates nulls whose positions drive a layer's position, anchor or path vertices through expressions | [Create Nulls for positional properties and paths][nc] | refused (ADR-0145) | The link is an expression, which fails the literal-values invariant ([ADR-0145](../../adr/0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md)). Path vertices go to #901 |

## 6. Precomposing and nested compositions

| Feature | What it does | Adobe doc | Montagent status | Evidence |
|---|---|---|---|---|
| Precompose: leave all attributes | Moves a layer into a new comp and keeps its transform and effects on the precomp layer | [Precomposing][pc] | #591 | The nest (#632). The transform the nest carries is `x`, `y`, `rotation`, `scale` and `pivot` |
| Precompose: move all attributes | Moves the layers with their properties into a new comp the size of the original | [Precomposing][pc] | #591 | The nest (#632) |
| A nested comp's own clock | A precomp keeps local time, so it can be moved, time-stretched or time-remapped as a unit | [Precomposing][pc] | refused (ADR-0166) | "The clock refusal stands, unchanged. No element's time is relative to another element or to a container" ([ADR-0166](../../adr/0166-capcut-and-premiere-nest-with-a-transform-and-adr-0001-refuses-the-clock-not-the-shared-space.md) §2, [ADR-0001](../../adr/0001-flat-element-list.md)) |
| Effects, masks and opacity on the precomp as one picture | The nested comp composites once, then takes effects and a blend | [Precomposing][pc] | #591 | Group opacity and group compositing were refused from the nest by #632 and are still open in #591's _Not yet specified_. [ADR-0147](../../adr/0147-a-blend-mode-is-a-flat-static-field-of-five-values-and-the-finished-element-blends-last.md) leaves "isolation or a group backdrop" to #591 |
| Pre-render a nested comp | Renders a precomp to a file and swaps it in as footage | [Precomposing][pc] | reachable | `montagent-prerender`'s pre-rendered footage ([ADR-0156](../../adr/0156-four-named-effects-join-the-effects-list-and-a-code-drawn-piece-enters-as-pre-rendered-footage.md)), or render a second Montagent project and place it as a `video` |

## 7. Track mattes, mattes from the stack, and alpha modes

| Feature | What it does | Adobe doc | Montagent status | Evidence |
|---|---|---|---|---|
| Alpha matte | A layer shows only where another layer's alpha is opaque | [Track mattes][tm] | refused (ADR-0150) | §4 "A matte from another element is refused" ([ADR-0150](../../adr/0150-wipe-slide-and-push-enter-as-transition-kinds-and-a-matte-from-another-element-is-refused.md)). The element's own `mask` replaces it (capability map "Not by design") |
| Alpha inverted matte | Shows where the matte's alpha is transparent | [Track mattes][tm] | refused (ADR-0150) | As above. An element-local inverted mask is `invert` ([ADR-0152](../../adr/0152-a-mask-gains-invert-and-feather-and-takes-no-text-shape-or-image-source.md)) |
| Luma matte | Shows where the matte layer is bright | [Track mattes][tm] | refused (ADR-0150) | §4 refuses "one element's alpha or luminance deciding where another shows" |
| Luma inverted matte | Shows where the matte layer is dark | [Track mattes][tm] | refused (ADR-0150) | As above |
| Track matte menu and pick whip (any layer as the matte, one matte shared by several layers) | Picks the matte layer per layer from the Modes column, in any stack position, with Alpha/Luma and inverted toggles | [Track mattes][tm] | refused (ADR-0150) | As above. `CONTEXT.md` **Transition**: "track matte (refused…)" |
| Text or a shape as a traveling matte | Text or a shape reveals footage through it | [Track mattes][tm] | refused (ADR-0152) | A mask "takes no text shape or image source" ([ADR-0152](../../adr/0152-a-mask-gains-invert-and-feather-and-takes-no-text-shape-or-image-source.md)). A closed path mask carries its own copy of the `points` ([ADR-0163](../../adr/0163-a-mask-takes-a-closed-path-inline-measured-from-its-own-rect.md)) |
| Preserve Underlying Transparency (T switch) | A layer shows only where the layers beneath it are opaque | [Track mattes][tm] | refused (ADR-0150) | An inference: ADR-0150 §4's wording ("one element's alpha… deciding where another shows") covers it, but the ADR does not name it. Over Montagent's opaque `background` the backdrop is always opaque, so it would matter only inside group compositing (#591) |
| Stencil Alpha | Cuts every layer below to this layer's alpha | [Blending modes][bm] | refused (ADR-0150) | ADR-0150 §4: another element's alpha decides where the stack shows |
| Stencil Luma | Cuts every layer below to this layer's luminance | [Blending modes][bm] | refused (ADR-0150) | As above |
| Silhouette Alpha | Punches this layer's alpha out of every layer below | [Blending modes][bm] | refused (ADR-0150) | As above |
| Silhouette Luma | Punches this layer's luminance out of every layer below | [Blending modes][bm] | refused (ADR-0150) | As above |
| Alpha Add | Adds alpha so two complementary edges join without a seam | [Blending modes][bm] | gap | AE-only and outside Premiere's list, so ADR-0147's refusal doesn't reach it. [ADR-0165](../../adr/0165-a-hard-edged-inverted-mask-is-the-complement-off-the-edge-and-a-feathered-pair-sums-within-one-level.md) notes that stacked complementary masks show the backdrop by c·(1 − c), which is the seam Alpha Add removes |
| Luminescent Premul | Stops colour values above the alpha from clipping after compositing, for premultiplied light effects such as lens flares | [Blending modes][bm] | gap | AE-only. No ADR names it |

## 8. Blending modes

[ADR-0147](../../adr/0147-a-blend-mode-is-a-flat-static-field-of-five-values-and-the-finished-element-blends-last.md)
admits five values (`normal`, `multiply`, `screen`, `overlay`, `add`). It refuses "a wider
vocabulary (`darken`, `lighten`, `soft_light`, `hard_light`, `color_dodge`, `color_burn`,
`difference`, or Premiere's list)" **for now**, because none has a forcing case. So every mode
on Premiere's list is refused for now. The modes only AE has were never weighed. Adobe's page
says AE's modes are "identical to blending modes in Adobe Photoshop", except the AE-only
modes. Stencil, silhouette, Alpha Add and Luminescent Premul are in §7.

| AE mode (category) | What it does | Adobe doc | Montagent status | Evidence |
|---|---|---|---|---|
| Normal (Normal) | Source over the stack | [Blending modes][bm] | covered | `blend` omitted or `"normal"` (ADR-0147) |
| Dissolve (Normal) | Randomly makes source pixels transparent by opacity | [Blending modes][bm] | refused (ADR-0147, for now) | On Premiere's list |
| Dancing Dissolve (Normal) | Dissolve with a pattern that changes every frame | [Blending modes][bm] | gap | AE-only. Not on Premiere's list |
| Darken (Subtractive) | Keeps the darker of each channel | [Blending modes][bm] | refused (ADR-0147, for now) | Named in the refusal |
| Multiply (Subtractive) | Multiplies the colours | [Blending modes][bm] | covered | `"blend": "multiply"` |
| Color Burn (Subtractive) | Darkens the stack by raising contrast | [Blending modes][bm] | refused (ADR-0147, for now) | Named (`color_burn`) |
| Classic Color Burn (Subtractive) | The Color Burn of AE 5.0 and earlier, kept for old projects | [Blending modes][bm] | gap | AE-only |
| Linear Burn (Subtractive) | Darkens by lowering brightness | [Blending modes][bm] | refused (ADR-0147, for now) | On Premiere's list |
| Darker Color (Subtractive) | Keeps the darker colour as a whole | [Blending modes][bm] | refused (ADR-0147, for now) | On Premiere's list |
| Add (Additive) | Sums the colours | [Blending modes][bm] | covered | `"blend": "add"` |
| Lighten (Additive) | Keeps the lighter of each channel | [Blending modes][bm] | refused (ADR-0147, for now) | Named |
| Screen (Additive) | Inverts, multiplies and inverts again | [Blending modes][bm] | covered | `"blend": "screen"` |
| Color Dodge (Additive) | Brightens the stack by lowering contrast | [Blending modes][bm] | refused (ADR-0147, for now) | Named (`color_dodge`) |
| Classic Color Dodge (Additive) | The Color Dodge of AE 5.0 and earlier, kept for old projects | [Blending modes][bm] | gap | AE-only |
| Linear Dodge (Additive) | Brightens by raising brightness | [Blending modes][bm] | partial | The same clamped sum as `add` on opaque pixels. CONTEXT **Blend mode** lists "linear dodge" under _Avoid_. How it treats alpha differs |
| Lighter Color (Additive) | Keeps the lighter colour as a whole | [Blending modes][bm] | refused (ADR-0147, for now) | On Premiere's list |
| Overlay (Complex) | Multiplies or screens, by the stack's brightness | [Blending modes][bm] | covered | `"blend": "overlay"` |
| Soft Light (Complex) | A gentle Overlay, by the source's brightness | [Blending modes][bm] | refused (ADR-0147, for now) | Named (`soft_light`) |
| Hard Light (Complex) | Multiplies or screens, by the source's brightness | [Blending modes][bm] | refused (ADR-0147, for now) | Named (`hard_light`) |
| Linear Light (Complex) | Burns or dodges by brightness | [Blending modes][bm] | refused (ADR-0147, for now) | On Premiere's list |
| Vivid Light (Complex) | Burns or dodges by contrast | [Blending modes][bm] | refused (ADR-0147, for now) | On Premiere's list |
| Pin Light (Complex) | Replaces colours, by the source's brightness | [Blending modes][bm] | refused (ADR-0147, for now) | On Premiere's list |
| Hard Mix (Complex) | Raises the contrast of the stack seen beneath a mask on the source | [Blending modes][bm] | refused (ADR-0147, for now) | On Premiere's list |
| Difference (Difference) | The absolute difference of the colours | [Blending modes][bm] | refused (ADR-0147, for now) | Named |
| Classic Difference (Difference) | The Difference of AE 5.0 and earlier, kept for old projects | [Blending modes][bm] | gap | AE-only |
| Exclusion (Difference) | A lower-contrast Difference | [Blending modes][bm] | refused (ADR-0147, for now) | On Premiere's list |
| Subtract (Difference) | Subtracts the source from the stack | [Blending modes][bm] | refused (ADR-0147, for now) | On Premiere's list |
| Divide (Difference) | Divides the stack by the source | [Blending modes][bm] | refused (ADR-0147, for now) | On Premiere's list |
| Hue (HSL) | The source's hue with the stack's saturation and luminosity | [Blending modes][bm] | refused (ADR-0147, for now) | On Premiere's list |
| Saturation (HSL) | The source's saturation | [Blending modes][bm] | refused (ADR-0147, for now) | On Premiere's list |
| Color (HSL) | The source's hue and saturation | [Blending modes][bm] | refused (ADR-0147, for now) | On Premiere's list |
| Luminosity (HSL) | The source's luminosity | [Blending modes][bm] | refused (ADR-0147, for now) | On Premiere's list |

## 9. Markers

| Feature | What it does | Adobe doc | Montagent status | Evidence |
|---|---|---|---|---|
| Layer markers | Notes, a duration and metadata pinned to a time on one layer | [Markers][mk] | gap | No marker construct. A grep of `docs/adr` and `CONTEXT.md` for "marker" finds only unrelated uses. The nearest pieces are `t_from`'s recorded intent on keyframes ([ADR-0086](../../adr/0086-recorded-intent-is-one-pattern-and-the-time-axis-instantiates-it.md)) and `group` |
| Composition markers | Notes and durations on the composition's time ruler. A nested comp's markers become layer markers in its parent | [Markers][mk] | gap | As above. A beat grid is found by `skills/montagent-craft/scripts/beat_grid.py` and held in the agent's notes, not in the project |
| Markers rendered as chapters, cue points or web links | Exports markers into the output file's metadata | [Markers][mk] | gap | Montagent writes no container metadata (schema `loop`: "Montagent writes no container-level loop metadata"). Export goes to #907 |
| Protected regions | A composition-marker span that Responsive Design – Time keeps from stretching when the comp is nested and stretched | [Responsive design][rd] | gap | No time-stretch of a nested piece exists to protect against (§6, ADR-0166) |

## 10. Time: stretch, reverse, freeze, trim, split, sequence

| Feature | What it does | Adobe doc | Montagent status | Evidence |
|---|---|---|---|---|
| Time stretch | Changes a whole layer's speed by one factor, keyframes and audio included | [Time-stretching][ts] | partial | A static `speed` on `video` and `audio` ([ADR-0020](../../adr/0020-speed-overrun-hold-loop.md)). Keyframes do not stretch with it: `t` is the element's own geometry ([ADR-0012](../../adr/0012-flat-transform-keyframes-carried-by-their-element.md)), so they are rewritten by hand. `image`, `text` and shapes have no source to stretch |
| Time-stretch a layer but not its keyframes | Changes the layer's speed while its keyframes keep their times (by cutting and pasting them back) | [Time-stretching][ts] | covered | This is what `speed` does: keyframe `t` is the element's own geometry and does not follow `speed` ([ADR-0020](../../adr/0020-speed-overrun-hold-loop.md), [ADR-0012](../../adr/0012-flat-transform-keyframes-carried-by-their-element.md)) |
| Time-reverse layer | Plays a layer backwards | [Time-stretching][ts] | partial | Video: a falling `source_time` curve ([ADR-0157](../../adr/0157-a-speed-ramp-is-a-time-remap-curve-of-source-times-on-a-video-element.md)). A remapped video is silent, and `source_time` on `audio` is a schema error. Reversed sound belongs to #795 |
| Freeze frame | Holds one frame of a layer | [Time-stretching][ts] | covered | A literal `source_time`, or `overrun: "hold"` at the end ([ADR-0157](../../adr/0157-a-speed-ramp-is-a-time-remap-curve-of-source-times-on-a-video-element.md), [ADR-0020](../../adr/0020-speed-overrun-hold-loop.md)) |
| Time remapping | A keyable curve of source time: ramps, reverse, holds | [Time-stretching][ts] | covered | `source_time` on `video` ([ADR-0157](../../adr/0157-a-speed-ramp-is-a-time-remap-curve-of-source-times-on-a-video-element.md)). Capability map "Time". Video only |
| Time-remap audio pitch | A remapped audio layer changes pitch with its speed | [Time-stretching][ts] | #795 | A remapped video is silent (format.md, [ADR-0157](../../adr/0157-a-speed-ramp-is-a-time-remap-curve-of-source-times-on-a-video-element.md)) and `source_time` on `audio` is a schema error. Belongs to the audio map |
| Trim or extend in and out points | Shortens or lengthens a layer at either end | [Selecting and arranging layers][sa] | covered | `start`/`end`, and `source_start`/`source_end` (`CONTEXT.md` **Timeline range**, **Source range**) |
| Slip edit | Moves the source under a fixed timeline span | [Selecting and arranging layers][sa] | covered | Edit `source_start`/`source_end` only (ADR-0020's agreement rule) |
| Split layer | Cuts a layer into two at the current time | [Selecting and arranging layers][sa] | covered | Two elements with adjacent half-open ranges and their own source ranges. Keyframes copy unchanged, because `t` travels with the element and may sit outside its range (format.md "Time"). There is no edit verb by design ([ADR-0011](../../adr/0011-tool-surface-reads-checks-renders.md)) |
| Overlay edit and ripple insert edit | Places trimmed footage at the current time, on top or pushing every later layer along | [Creating layers][cl] | covered | Overlay: a new element in a free track. Ripple insert: `shift --at --delta` moves every later time and carries keyframes ([ADR-0032](../../adr/0032-slack-is-invariant-shift-refuses-compare-is-the-backstop.md), `CONTEXT.md` **Shift**), then the new element goes in |
| Lift and extract work area | Removes a span from layers, leaving a gap (lift) or closing it (extract) | [Selecting and arranging layers][sa] | reachable | Lift: split and trim the elements by hand. Extract also needs later times pulled back, and `shift` takes only a positive `delta` (`crates/montagent/src/cli.rs`), so the agent rewrites them |
| Sequence layers (keyframe assistant, with overlap and crossfade) | Places layers end to end, optionally overlapped with a crossfade | [Selecting and arranging layers][sa] | reachable | The agent writes each range, or generates them with a script (the router's loop step 3). An overlap is a `transition` across two tracks ([ADR-0059](../../adr/0059-transitions-element-type-crossfade-only-exact-window.md)). No shipped script does it |
| Quick Offset | Staggers selected layers (or keyframes) evenly in time by dragging, keyframes moving with their layers | [Selecting and arranging layers][sa] | reachable | The agent writes each element's `start`/`end` and its keyframe `t`s. A stagger is admitted only across the units of one text element ([ADR-0148](../../adr/0148-a-repeat-does-not-enter-and-a-stagger-enters-only-across-the-units-of-one-text-element.md)) |

## 11. Layer styles

Adobe's page says After Effects keeps every Photoshop layer style on import but can only create
and edit some of them. Importing PSD styles goes to #907.

| Feature | What it does | Adobe doc | Montagent status | Evidence |
|---|---|---|---|---|
| Drop Shadow | A shadow cast from the layer's alpha | [Layer styles][bm] | covered | The `shadow` effect `{color, dx, dy, radius, opacity}` (ADR-0040, ADR-0146) |
| Inner Shadow | A shadow inside the layer's contents, for a recessed look | [Layer styles][bm] | gap | No effect, and no ADR names it |
| Outer Glow | A soft colour halo around the alpha | [Layer styles][bm] | covered | "A coloured glow from the alpha is a zero-offset `shadow`" (format/compositing.md, [ADR-0156](../../adr/0156-four-named-effects-join-the-effects-list-and-a-code-drawn-piece-enters-as-pre-rendered-footage.md)) |
| Inner Glow | A glow inward from the edges or the centre | [Layer styles][bm] | gap | No effect, and no ADR names it |
| Bevel and Emboss | Highlights and shadows that give the layer relief | [Layer styles][bm] | gap | No effect. "bevel" in the ADRs is only the stroke join ([ADR-0158](../../adr/0158-a-path-chooses-its-stroke-join-and-cap-and-every-shape-takes-a-dash-pattern.md)) |
| Satin | Interior shading with a satiny finish | [Layer styles][bm] | gap | No effect, and no ADR names it |
| Color Overlay | Fills the layer with one colour, keeping its alpha | [Layer styles][bm] | covered | `tint` with `amount: 1` "pushes pixel colour toward `color`" ([ADR-0049](../../adr/0049-v1-colour-filter-vocabulary-four-scalar-members.md)) |
| Gradient Overlay | Fills the layer with a gradient, keeping its alpha | [Layer styles][bm] | partial | A gradient paint on `rect`, `ellipse`, `path` and `text` ([ADR-0149](../../adr/0149-a-gradient-is-a-paint-linear-or-radial-measured-against-the-declared-box.md)). There is no gradient over an `image` or `video` |
| Stroke (style) | An outline around the layer's alpha | [Layer styles][bm] | partial | `stroke` on shapes and text ([ADR-0014](../../adr/0014-stroke-is-paint-the-text-box-is-required.md)). There is no outline around an image's or video's alpha |
| Blending Options (fill opacity, blend interior styles as group, global light, blend ranges) | Opacity of the contents apart from the styles, interior styles taking the layer's blend mode, one light angle for every style | [Layer styles][bm] | partial | A fill's own alpha (`#RRGGBBAA`) dims the fill apart from the stroke and `shadow`. Blending interior styles as a group and a shared light have no equivalent |
| Exclude channels from blending (Advanced Blending) | Leaves the red, green or blue channel out of the layer's blend, keyable | [Layer styles][bm] | gap | `blend` is one static value for the whole pixel ([ADR-0147](../../adr/0147-a-blend-mode-is-a-flat-static-field-of-five-values-and-the-finished-element-blends-last.md)). No ADR names per-channel blending |

## Counts

| Status | Rows |
|---|---|
| covered | 36 |
| partial | 14 |
| reachable | 10 |
| owned by #591 | 9 |
| owned by #795 | 2 |
| refused | 39: ADR-0147 "for now" 22, ADR-0150 10, ADR-0167 2, ADR-0145 2, and one each for ADR-0147 (linear blending), ADR-0152 and ADR-0166 |
| gap | 22 |
| app | 6 |
| **total** | **138** |

## Gaps worth a decision ticket

Grouped by what would decide them. Each group is a candidate ticket for
[#908](https://github.com/MBehtemam/Montagent/issues/908) to order.

**Effects on what lies below.**
- **Adjustment layer**: effects applied to the composite under a span, such as a blur or a
  grade over everything. ADR-0040 keeps every effect element-local. Neither ADR-0147 (blend
  reads the backdrop, but once) nor #632 (refused it only as the group transform's shape)
  rules on it. This is the largest gap in the area.
- Its neighbours are group compositing and isolation, which are owned by #591 and not gaps
  here.

**Time.**
- **Frame blending** (Frame Mix, Pixel Motion) for slowed or rate-converted footage. It is
  already one of #896's candidates.
- **Time stretch that carries keyframes**. Today `speed` leaves `t` alone, so every key is
  rewritten by hand. A skill script could do it (ADR-0177's pattern), with no format change.
- **Sequence layers** and **Quick Offset** as a skill script that writes ranges, keyframe
  times and transitions.
- **Extract** (ripple delete): `shift` takes only a positive `delta`, so closing a removed
  span is hand arithmetic.
- Reversed sound goes to #795.

**Markers.**
- **Layer and composition markers** as render-inert annotations: notes, beat marks and
  chapter spans that `validate` could check against cuts. They would sit next to recorded
  intent (ADR-0086), and the beat grid (#547) would have somewhere in the project to live.
  Export as chapters or cue points goes to #907.

**Layer styles under ADR-0156's growth path.**
- **Inner Shadow, Inner Glow, Bevel and Emboss, Satin**. All four are alpha-derived and
  element-local. Each would join the effects list by its own ADR.
- **Stroke and Gradient Overlay on `image` and `video`**: an outline or a gradient on a
  raster's alpha.

**Small consistency gaps.**
- **A bypass for visual `effects` members** (`enabled: false`, as ADR-0169 gives audio).
- **Per-channel blending** (AE's Advanced Blending, which leaves R, G or B out of a blend).
  ADR-0147's static `blend` field does not reach it. Low value.
- **A guide element** that is left out of the render. ADR-0150's dislike of "an element…
  in a track without painting" weighs against it.
- **Fractional frame rates** (29.97, 23.976): `fps` is an integer. This probably belongs with
  #907.
- **Start timecode**: low value.

**Blend modes: a forcing-case question, not a census gap.**
- ADR-0147 refuses Premiere's list "for now", which covers 22 of AE's modes. The census adds
  no forcing case. If [#897](https://github.com/MBehtemam/Montagent/issues/897) makes AE
  precedent, the AE-only modes could also be weighed: Dancing Dissolve, the three Classic
  modes, Alpha Add and Luminescent Premul. Alpha Add matters only once group compositing
  exists.

**Refusals that [#897](https://github.com/MBehtemam/Montagent/issues/897) /
[#910](https://github.com/MBehtemam/Montagent/issues/910) should list.**
- Track mattes, stencil and silhouette modes, and Preserve Underlying Transparency are
  refused on ADR-0150 §4's invariant reasoning: `validate` cannot follow a dependency that
  lives in pixels. That reasoning is not about the reference class, so AE becoming precedent
  does not by itself reopen them.
- Preserve Underlying Transparency's refusal is inferred from §4's wording, not named.
- Camera and the 3D renderer are refused by ADR-0167 as "After Effects-only", which *is* a
  reference-class reason, so they belong on #910's list.
- Lights come with them (#902).

[wn]: https://helpx.adobe.com/after-effects/desktop/what-s-new/whats-new.html
[cb]: https://helpx.adobe.com/after-effects/desktop/work-with-compositions/composition-settings/composition-basics.html
[pc]: https://helpx.adobe.com/after-effects/desktop/work-with-compositions/precomposing-and-nesting/precomposing-nesting-pre-rendering.html
[cl]: https://helpx.adobe.com/after-effects/desktop/work-with-layers/create-layers/creating-layers.html
[ml]: https://helpx.adobe.com/after-effects/desktop/work-with-layers/manage-layers/layers.html
[lp]: https://helpx.adobe.com/after-effects/desktop/work-with-layers/layer-properties/layer-properties.html
[bm]: https://helpx.adobe.com/after-effects/desktop/work-with-layers/work-with-layer-blending-modes/blending-modes-layer-styles.html
[tm]: https://helpx.adobe.com/after-effects/desktop/work-with-transparency-and-compositing/work-with-track-mattes-and-traveling-mattes/track-mattes-and-traveling-mattes.html
[mk]: https://helpx.adobe.com/after-effects/desktop/work-with-layers/work-with-layer-and-composition-markers/layer-markers-composition-markers.html
[ts]: https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/time-stretching-and-time-remapping/time-stretching-time-remapping.html
[cam]: https://helpx.adobe.com/after-effects/desktop/work-with-layers/camera-layer/cameras-lights-points-interest.html
[shd]: https://helpx.adobe.com/after-effects/desktop/work-with-3d-composition/cast-and-accept-shadows/enable-lights-to-cast-shadows.html
[sh]: https://helpx.adobe.com/after-effects/desktop/drawing-painting-and-paths/shapes-and-shape-attributes/shape-attributes-paint-operations-path.html
[mb]: https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/assorted-animation-tools/assorted-animation-tools.html#motion_blur
[sa]: https://helpx.adobe.com/after-effects/desktop/work-with-layers/select-and-arrange-layers/selecting-arranging-layers.html
[pv]: https://helpx.adobe.com/after-effects/desktop/view-and-preview/preview-video-and-audio/previewing.html
[vw]: https://helpx.adobe.com/after-effects/desktop/view-and-preview/preview-video-and-audio/modifying-using-views.html
[rd]: https://helpx.adobe.com/after-effects/desktop/motion-graphics/add-responsive-design/responsive-design.html
[nc]: https://helpx.adobe.com/after-effects/desktop/drawing-painting-and-paths/null-controllers/create-nulls-for-positional-properties-and-paths.html
[cpr]: https://helpx.adobe.com/after-effects/desktop/work-with-compositions/composition-profiler/composition-profiler.html
