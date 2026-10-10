# Premiere gap: motion, keyframing and compositing

Research ticket [#890](https://github.com/MBehtemam/Montagent/issues/890), on the map
[#882](https://github.com/MBehtemam/Montagent/issues/882) ("every Premiere Pro feature, set
against Montagent v0.2.0").

- **Date:** 2026-10-10
- **Montagent side:** tag `v0.2.0` (`51aa4b5`, = `origin/main` when this was written).
  Read: `CONTEXT.md`, `schema/montagent.schema.json`, `crates/montagent-core/docs/format.md`,
  the ADRs named per row, the MCP tool surface (`crates/montagent/src/mcp*`: `compare`,
  `create_project`, `frame`, `measure`, `preview`, `query`, `render`, `shift`, `validate`,
  `verify`, `where`), and `skills/`.
- **Premiere side:** Adobe's own Premiere user guide on helpx.adobe.com (the current
  `/premiere/desktop/` guide, 26.x, plus older `/premiere-pro/using/` pages where the new guide
  has no page). Only Adobe pages are cited. **Method caveat:** this session's network proxy
  refused direct connections to helpx.adobe.com, so the pages were read through the search
  engine's extracts of them (restricted to `helpx.adobe.com`), not fetched whole. The search
  budget ran out before three pages could be looked up (nested sequences, adjustment layers,
  Alpha Adjust's controls); those rows say so in the URL column. Every other URL was returned by
  a search for that feature.
- **Status vocabulary** (the map's): `have` · `have (skill)` · `partial` · `missing` ·
  `refused` (an ADR rules it out; cited).

## Counts

| Status | Rows |
| --- | --- |
| have | 50 |
| have (skill) | 2 |
| partial | 11 |
| missing | 36 |
| refused | 31 |
| **total** | **130** |

Rows are one per mechanism: one per blend mode, per interpolation type, per mask control, per
Warp Stabilizer option and per keying effect. The 22 blend modes counted `refused` are
refused "for now" by ADR-0147, which needs a forcing case per added mode. They are not refused
on model grounds.

## The table

### 1. Motion (intrinsic effect)

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
| --- | --- | --- | --- | --- | --- |
| Motion: Position | Fixed effect on every clip; X/Y position, keyframeable, set in Effect Controls or by dragging in the Program Monitor | https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/apply-motion-effect.html | have | `x`, `y` on every visual element, `Animatableint64` (ADR-0012) | Positions are integer pixels; Premiere takes fractional values. A keyed position still resolves unrounded between keys. |
| Motion: Scale (uniform) | Scale slider with Uniform Scale checked | https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/apply-motion-effect.html | have | `scale` `[sx, sy]`, animatable (ADR-0012); equal components = uniform | — |
| Motion: Scale Width (non-uniform) | Uniform Scale unchecked, giving Scale Height and Scale Width | https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/apply-motion-effect.html | have | `scale` is a two-component array (`AnimatableArray_size_2_of_double`) | — |
| Motion: Rotation | Degrees, keyframeable | https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/apply-motion-effect.html | have | `rotation`, `Animatabledouble`, never normalised (ADR-0012) | — |
| Motion: Anchor Point | Free pivot for position/scale/rotation, at any point in clip-local coordinates, keyframeable; snaps to corners/midpoints | https://helpx.adobe.com/ph_fil/premiere-pro/how-to/move-anchor-point.html | partial | `origin`: nine keywords, static (ADR-0013; `Origin` in schema) | An arbitrary pivot point (any x/y inside or outside the box) and a keyed pivot. Only the nine snap points ship, and they are static. |
| Motion: Anti-flicker Filter | Softening filter against interlace flicker on thin lines, keyframeable strength | https://helpx.adobe.com/premiere/desktop/troubleshooting/media-issues/eliminate-flicker.html | missing | No field; output is progressive (no interlace model anywhere in the schema) | The whole mechanism. |
| Composite with other clips (Motion's stacking role) | Clips on higher tracks composite over lower ones | https://helpx.adobe.com/premiere/desktop/add-video-effects/types-of-effects/types-of-effects.html | have | Tracks with `layer` stacking (ADR-0004); stack order resolved by `query --at` | — |

### 2. Opacity (intrinsic effect)

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
| --- | --- | --- | --- | --- | --- |
| Opacity | Fixed effect, 0–100 %, keyframeable (rubber band or Effect Controls) | https://helpx.adobe.com/premiere/desktop/add-video-effects/create-masks-and-composites/specify-clip-opacity.html | have | `opacity`, `Animatabledouble` (ADR-0012, ADR-0146) | — |
| Opacity fade in/out | Two opacity keyframes, optionally Bezier | https://helpx.adobe.com/premiere/desktop/add-video-effects/create-masks-and-composites/specify-clip-opacity.html | have | Keyed `opacity` with `ease`; `crossfade` transition between two elements (ADR-0059) | — |
| Opacity render order | Video Effects list first, then Motion, then Opacity; reorder by using Alpha Adjust | https://helpx.adobe.com/premiere-pro/using/compositing-alpha-channels-adjusting-clip.html | partial | Fixed order: `effects` → `opacity` → `blend` (ADR-0147 §2) | Moving opacity earlier or later in the chain. The order is one fixed rule. |
| Alpha Adjust effect | Effect version of opacity placed anywhere in the effect stack (the compositing page names it for reordering opacity) | URL for its own page not retrieved (search budget); named on https://helpx.adobe.com/premiere-pro/using/compositing-alpha-channels-adjusting-clip.html | missing | No `effects` member for alpha (ADR-0040 vocabulary; schema `Effect` has 12 members, none alpha) | An opacity step placed in the effect chain, and any alpha inversion. |
| Opacity masks (ellipse, 4-point polygon, free-draw Bezier under Opacity) | Masks on the Opacity effect cut the clip's alpha | https://helpx.adobe.com/premiere/desktop/add-video-effects/create-masks-and-composites/create-masks-using-shapes.html | have | `mask` member of `effects` (ADR-0068, ADR-0084, ADR-0163) | See the Masks section for the controls one by one. |
| Blend Mode menu (in Opacity) | One enumerated mode per clip beside opacity, 27 modes, static | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-composites/combine-multiple-videos-or-stil-images-using-blend-modes.html | partial | `blend`, five values, static (ADR-0147) | 22 of the 27 modes (rows in section 6). |

### 3. Time Remapping and time interpolation

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
| --- | --- | --- | --- | --- | --- |
| Time Remapping: speed keyframes | Speed rubber band on the clip; Ctrl/Cmd-click adds speed keyframes, drag the band to change speed | https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-speed/change-clip-speed-and-duration-using-time-remapping.html | have | `source_time` keyframe list on `video` (ADR-0157); `query --at` reports derived `rate` | Montagent keys the source position, not the speed. The rate is the curve's slope, so the mechanism is equivalent. |
| Time Remapping: speed transitions (ramped) | Split speed keyframe with Bezier handles to ease between speeds | https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-speed/change-clip-speed-and-duration-using-time-remapping.html | have | `ease` on `source_time` records (ADR-0157, ADR-0012) | — |
| Time Remapping: freeze inside a clip | Drag a speed keyframe to 0 % | https://helpx.adobe.com/au/premiere/desktop/organize-media/file-organization/freeze-a-frame-for-a-portion-of-a-clip-using-time-remapping.html | have | A flat pair of `source_time` keys (ADR-0157 §2) | — |
| Frame Hold (whole clip) | Freeze one frame for the clip's duration | https://helpx.adobe.com/premiere-pro/using/freeze-frame.html | have | Literal `source_time` (ADR-0157: "That is Premiere's Frame Hold") | — |
| Time Remapping: reverse | Reverse segment inside a remap | https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-speed/change-clip-speed-and-duration-using-time-remapping.html | have | Falling `source_time` segment (ADR-0157; a `reverse` field is refused there) | — |
| Constant speed (Speed/Duration) | Percentage speed for the whole clip | https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-speed/change-clip-speed-using-the-speedduration-option.html | have | `speed` (ADR-0020, ADR-0045) | — |
| Time interpolation: Frame Sampling | Repeat or drop whole frames to reach the speed | https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-speed/apply-time-interpolation-methods-to-adjust-clip-speed.html | have | The frame shown is the last source frame starting at or before the resolved instant (ADR-0157 §3, ADR-0096) | — |
| Time interpolation: Frame Blending | Mix neighbouring frames for smoother slow motion | https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-speed/apply-time-interpolation-methods-to-adjust-clip-speed.html | missing | No field; ADR-0157 §3 fixes nearest-earlier sampling | The whole mechanism. |
| Time interpolation: Optical Flow | Synthesises in-between frames from motion estimation | https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-speed/apply-time-interpolation-methods-to-adjust-clip-speed.html | missing | No field; no pre-render recipe for it in `skills/montagent-prerender` | The whole mechanism. A skill could pre-render it the way ADR-0171 does for Lottie. |

### 4. Keyframes, interpolation and graphs

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
| --- | --- | --- | --- | --- | --- |
| Keyframe any effect property | Stopwatch on any property in Effect Controls | https://helpx.adobe.com/premiere/desktop/add-video-effects/control-effects-and-transitions-using-keyframes/about-keyframes.html | have | Every property the schema types as animatable, incl. every numeric and colour effect parameter (ADR-0146) | Booleans and enums stay static by rule (ADR-0146). |
| Add / set keyframes | Toggle animation, add keyframe at playhead, or Pen tool on rubber band | https://helpx.adobe.com/premiere/desktop/add-video-effects/control-effects-and-transitions-using-keyframes/add-keyframes.html | have | `{t, v, ease}` records (ADR-0012, ADR-0038, ADR-0082), written by exact-string edit | — |
| Select / move / delete keyframes | Select, drag in time, delete | https://helpx.adobe.com/premiere/desktop/add-video-effects/control-effects-and-transitions-using-keyframes/select-keyframes.html | have | Record edits; `shift` moves an element with its keyframes and splits lists (ADR-0012, ADR-0146) | — |
| Keyframe navigation (go to previous/next keyframe) | Panel buttons step the playhead between keyframes | https://helpx.adobe.com/premiere/desktop/add-video-effects/control-effects-and-transitions-using-keyframes/view-keyframes-in-the-effect-controls-panel.html | partial | `query --at` reads resolved values at any instant; the contact sheet's keyframe tiles (docs/research/keyframe-tiles) | A "next keyframe" step across all lists. An agent reads the `t` values directly. |
| Copy and paste keyframes | Copy keyframes to another property or clip | https://helpx.adobe.com/premiere/desktop/add-video-effects/control-effects-and-transitions-using-keyframes/keyframes-and-graphs-in-panels.html | have | Keyframe lists are plain JSON copied by exact-string replace (ADR-0145 invariant 4) | — |
| Interpolation: Linear | Constant rate between keyframes; default | https://helpx.adobe.com/premiere/desktop/add-video-effects/control-effects-and-transitions-using-keyframes/change-keyframe-interpolation-method.html | have | `ease: "linear"` (`EaseName`, ADR-0012) | — |
| Interpolation: Bezier | Independent direction handles each side of a keyframe; influence handles | https://helpx.adobe.com/premiere/desktop/add-video-effects/control-effects-and-transitions-using-keyframes/control-effect-changes-using-bezier-keyframe-interpolation.html | have | `ease: [x1, y1, x2, y2]` per segment; `y` outside [0, 1] for overshoot (`Ease`, ADR-0012) | Handles live per segment, not per keyframe, so the two sides of one keyframe sit on two records. |
| Interpolation: Auto Bezier | Smooth rate through a keyframe, handles set automatically | https://helpx.adobe.com/premiere/desktop/add-video-effects/control-effects-and-transitions-using-keyframes/change-keyframe-interpolation-method.html | partial | Any curve it makes can be written as per-segment beziers (ADR-0012) | Automatic tangents. The author must compute handles that make the rate continuous through each key. No ease name does it. |
| Interpolation: Continuous Bezier | Smooth through a keyframe, handles adjustable but locked collinear | https://helpx.adobe.com/premiere/desktop/add-video-effects/control-effects-and-transitions-using-keyframes/change-keyframe-interpolation-method.html | partial | Per-segment beziers (ADR-0012) | A constraint tying one segment's outgoing handle to the next segment's incoming handle. Continuity is the author's job, and `validate` does not check it. |
| Interpolation: Hold | Value jumps at the next keyframe; flat graph | https://helpx.adobe.com/premiere/desktop/add-video-effects/control-effects-and-transitions-using-keyframes/change-keyframe-interpolation-method.html | have | `ease: "step"` (ADR-0012 names it `step`, not `hold`) | — |
| Interpolation: Ease In | Converts to Bezier decelerating into the keyframe | https://helpx.adobe.com/premiere/desktop/add-video-effects/control-effects-and-transitions-using-keyframes/change-keyframe-interpolation-method.html | have | `ease: "ease-in"` / any bezier (`EaseName`) | Montagent's names are CSS curves, not Premiere's exact handle values. |
| Interpolation: Ease Out | Converts to Bezier accelerating out of the keyframe | https://helpx.adobe.com/premiere/desktop/add-video-effects/control-effects-and-transitions-using-keyframes/change-keyframe-interpolation-method.html | have | `ease: "ease-out"` / any bezier (`EaseName`) | As above. |
| Spatial interpolation (motion path) | Position keyframes form a path in the Program Monitor; corners rounded (Bezier) or angular (linear) | https://helpx.adobe.com/premiere/desktop/add-video-effects/control-effects-and-transitions-using-keyframes/edit-keyframes-graphs.html | missing | `x` and `y` are separate scalar lists (ADR-0012); a text-on-path exists only for a text's own glyphs (ADR-0161) | A spatial curve between position keys with its own handles. A curved move today is two decoupled per-axis eases or many keys. |
| Value graph | Graph of the property's value, edited with handles | https://helpx.adobe.com/premiere/desktop/add-video-effects/control-effects-and-transitions-using-keyframes/keyframes-and-graphs-in-panels.html | partial | Same curves are written as numbers; `preview` / contact sheet show the result | An interactive graph view. Montagent has no GUI. |
| Velocity graph / influence | Graph of rate of change; direction and influence handles; Ctrl/Cmd splits in/out | https://helpx.adobe.com/premiere/desktop/add-video-effects/control-effects-and-transitions-using-keyframes/adjust-effect-speed.html | partial | Bezier `x1`/`x2` are the influence and `y1`/`y2` the speed (ADR-0012) | A velocity view and velocity in units per second. An agent derives speed from the bezier itself. |
| Timeline rubber-band keyframing | Keyframes shown and dragged on the clip in the Timeline | https://helpx.adobe.com/premiere/desktop/add-video-effects/control-effects-and-transitions-using-keyframes/keyframes-and-graphs-in-panels.html | missing | No timeline GUI; `where` and `query` are the read-out | The GUI. |

### 5. Masks and mask tracking

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
| --- | --- | --- | --- | --- | --- |
| Ellipse mask | Ellipse tool in Effect Controls | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/rectangle-ellipse-and-pen-masks.html | have | `mask` `shape: "ellipse"` / `"circle"` (ADR-0084) | — |
| Rectangle mask (incl. 26.0 rounded corners) | Rectangle tool; 26.0 adds rounded-corner control, numeric editing | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/rectangle-ellipse-and-pen-masks.html | have | `shape: "rect"` with `radius` (ADR-0084) | — |
| Polygon mask | Polygon tool | https://helpx.adobe.com/premiere/desktop/add-video-effects/create-masks-and-composites/create-masks-using-shapes.html | have | `shape: "path"` with straight-segment `points` (ADR-0163) | No polygon preset with a side count; vertices are written out. |
| Pen / Free Draw Bezier mask | Pen tool; Alt/Option-click adds Bezier handles; close on first point | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/modifiers-and-conditions-for-masking.html | have | `shape: "path"`, `points` with `in`/`out` handles, always closed (ADR-0163) | — |
| Mask Path keyframes | Mask path animated per frame (by hand or by tracking) | https://helpx.adobe.com/premiere/desktop/add-video-effects/create-masks-and-composites/adjust-masks-at-the-clip-or-frame-level.html | have | Mask rect, `radius` and `points` are animatable (ADR-0146, ADR-0163) | Path keys must keep matching vertex counts (ADR-0162). |
| Clip-level vs frame-level mask adjust | Whole-clip change vs. a change at the current frame that auto-creates keyframes | https://helpx.adobe.com/premiere/desktop/add-video-effects/create-masks-and-composites/adjust-masks-at-the-clip-or-frame-level.html | have | Static literal vs. keyframe list on the same field (ADR-0146) | — |
| Mask Feather | Softens the edge; handle in Program Monitor | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/adjust-mask-properties.html | have | `feather`, animatable integer (ADR-0152 §2) | Only a centred feather; inward-only or outward-only feather is refused (ADR-0152). |
| Mask Expansion | Grows or shrinks the mask edge by pixels | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/refining-and-combining-masks.html | refused | ADR-0152 §3: "a second knob for one edge" | An outline offset for `path` masks, which the rect alone cannot express. |
| Mask Opacity | Partial erase of the masked region | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/refining-and-combining-masks.html | refused | ADR-0152 §3: "Not until a case asks" | — (refused until a case asks) |
| Invert mask | Swap masked and unmasked areas | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/refining-and-combining-masks.html | have | `invert` (ADR-0152 §1; edge behaviour ADR-0165) | — |
| Mask combine: Intersect | Mask blend mode, Boolean over the masks above it | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/refining-and-combining-masks.html | have | Masks in one `effects` list multiply (intersect) (ADR-0152 §1) | — |
| Mask combine: Subtract | Mask blend mode | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/refining-and-combining-masks.html | have | An `invert: true` mask intersected with the others (ADR-0152 §1, "keeps a ring") | — |
| Mask combine: Add (union) | Mask blend mode | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/refining-and-combining-masks.html | missing | Masks only intersect (ADR-0152 §1) | A union of two shapes on one element. The workaround is two copies of the element. |
| Mask scoped to an effect (effect applies only inside the mask) | A mask belongs to an effect, e.g. blur only a face | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/isolated-adjustments-using-masks.html | missing | A `mask` cuts the element's alpha; no effect takes a mask scope (ADR-0040, ADR-0084) | Scoping a blur, colour or other effect to a region. The workaround is a masked duplicate element. |
| Unassigned masks / apply effects to existing masks | Masks drawn first, then assigned to effects | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/apply-effects-to-existing-unassigned-masks.html | missing | No mask object outside an element's `effects` list | Masks as reusable objects. Copies are written per element (ADR-0163 §1). |
| Mask with Text/Shape (graphic as matte) | A text or shape layer masks what lies beneath it | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/rectangle-ellipse-and-pen-masks.html | refused | ADR-0152 §3 (text and shape sources refused inside `mask`); ADR-0150 §4 (matte from another element) | — |
| Object Masking (AI, beta) | Click or lasso a subject; model generates a mask; Sharp/Smooth edge modes; +/- refine; tracked | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/object-masking.html | missing | No AI segmentation; an image- or luma-sourced mask is deferred (ADR-0152 §3, ADR-0040) | Segmentation, and the per-frame alpha-matte source it would need. |
| Object mask Sharp / Smooth modes | Edge quality modes; Smooth has no expansion | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/apply-sharp-and-smooth-modes-in-object-masking.html | missing | — | The whole mechanism (depends on Object Masking). |
| Mask tracking: forward / backward / bi-directional / one frame | Tracker analyses the clip and writes mask-path keyframes | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/track-masks.html | missing | No tracker in any tool or skill; masks are keyable, so a tracker's output could be written (ADR-0146) | An analysis tool that writes the mask keyframes. No ADR refuses it. |
| Mask tracking method: Position | Track the mask's position only | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/mask-tracking-tools.html | missing | — | As above. |
| Mask tracking method: Position, Scale and Rotation | Track position, scale and rotation | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/mask-tracking-tools.html | missing | — | As above. |
| Mask tracking method: Perspective | Track with perspective (vector masks only) | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/mask-tracking-tools.html | missing | — | As above, plus a per-vertex result; a tracked `path` could carry it. |

### 6. Blend modes (one row per mode)

All 27 from Adobe's list. Montagent's `blend` is a flat, static field per visual element,
applied last, once, on stored sRGB values (ADR-0147 §1–2). Premiere's mode is static too.
ADR-0147's "Considered and refused" names "a wider vocabulary (`darken`, `lighten`,
`soft_light`, `hard_light`, `color_dodge`, `color_burn`, `difference`, or Premiere's list)" as
"refused for now: none has a forcing case". So every mode outside the five is `refused`, and
each would enter with its own forcing case and byte-identity test. URL for every row:
https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-composites/blend-mode-options.html

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
| --- | --- | --- | --- | --- | --- |
| Normal | Source over backdrop | (above) | have | `blend: "normal"` (default) | — |
| Dissolve | Random pixels of the source become transparent by opacity | (above) | refused | ADR-0147 (wider vocabulary refused for now) | Needs a forcing case. It would also need a seeded noise rule like `grain`'s (ADR-0156). |
| Darken | Keeps the darker of source and backdrop per channel | (above) | refused | ADR-0147 (`darken` named) | Forcing case. |
| Multiply | s·d | (above) | have | `blend: "multiply"` | — |
| Color Burn | Darkens backdrop by increasing contrast | (above) | refused | ADR-0147 (`color_burn` named) | Forcing case. |
| Linear Burn | Darkens backdrop by decreasing brightness | (above) | refused | ADR-0147 | Forcing case. |
| Darker Color | Keeps whichever whole colour has lower value | (above) | refused | ADR-0147 | Forcing case. |
| Lighten | Keeps the lighter per channel | (above) | refused | ADR-0147 (`lighten` named) | Forcing case. |
| Screen | s + d − s·d | (above) | have | `blend: "screen"` | — |
| Color Dodge | Brightens backdrop by decreasing contrast | (above) | refused | ADR-0147 (`color_dodge` named) | Forcing case. |
| Linear Dodge (Add) | min(1, s + d) | (above) | have | `blend: "add"` (Skia `Plus`; `linear_dodge` is not a spelling, ADR-0147 §1) | — |
| Lighter Color | Keeps whichever whole colour has higher value | (above) | refused | ADR-0147 | Forcing case. |
| Overlay | Multiply on dark backdrop, screen on light | (above) | have | `blend: "overlay"` | — |
| Soft Light | Gentle darken/lighten by source | (above) | refused | ADR-0147 (`soft_light` named) | Forcing case. |
| Hard Light | Multiply or screen by source | (above) | refused | ADR-0147 (`hard_light` named) | Forcing case. |
| Vivid Light | Burn or dodge by contrast, by source | (above) | refused | ADR-0147 | Forcing case. |
| Linear Light | Burn or dodge by brightness, by source | (above) | refused | ADR-0147 | Forcing case. |
| Pin Light | Replaces colours depending on source lightness | (above) | refused | ADR-0147 | Forcing case. |
| Hard Mix | Posterises the sum to primaries | (above) | refused | ADR-0147 | Forcing case. |
| Difference | abs(s − d) | (above) | refused | ADR-0147 (`difference` named) | Forcing case. |
| Exclusion | Lower-contrast difference | (above) | refused | ADR-0147 | Forcing case. |
| Subtract | d − s | (above) | refused | ADR-0147 | Forcing case. |
| Divide | d ÷ s | (above) | refused | ADR-0147 | Forcing case. |
| Hue | Source hue with backdrop saturation and luminosity | (above) | refused | ADR-0147 | Forcing case. A non-separable HSL mode also needs its own arithmetic stated. |
| Saturation | Source saturation | (above) | refused | ADR-0147 | As Hue. |
| Color | Source hue and saturation | (above) | refused | ADR-0147 | As Hue. |
| Luminosity | Source luminosity | (above) | refused | ADR-0147 | As Hue. |

### 7. Motion tracking, stabilisation and reframing

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
| --- | --- | --- | --- | --- | --- |
| Warp Stabilizer (effect) | Analyses camera motion and counter-transforms frames | https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/stabilize-shaky-footage-using-warp-stabilizer.html | missing | No effect, field, tool or skill; no ADR refuses it | The whole mechanism. Render-time analysis would break ADR-0146 criterion 2 (nothing carried between frames), so it fits a pre-render skill or a tool that writes keyframes. |
| Warp Stabilizer Result: Smooth Motion + Smoothness | Keeps eased camera motion; higher smoothness crops more | https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/warp-stabilizer-settings.html | missing | — | As above. |
| Warp Stabilizer Result: No Motion | Removes all camera motion | https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/warp-stabilizer-settings.html | missing | — | As above. |
| Warp Stabilizer Method: Position; Position, Scale, Rotation; Perspective; Subspace Warp | Model of the correction, from translation to per-region warp | https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/warp-stabilizer-settings.html | missing | — | As above. The first two could land as `x`/`y`/`scale`/`rotation` keyframes; Perspective and Subspace Warp could not (corner pin refused, ADR-0167). |
| Warp Stabilizer Framing: Stabilize Only; Stabilize, Crop; Stabilize, Crop, Auto-scale; Stabilize, Synthesize Edges | How moving edges are handled | https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/warp-stabilizer-settings.html | missing | `clip` and `scale` could express crop + auto-scale by hand (ADR-0025, ADR-0012) | The analysis, and edge synthesis from neighbouring frames. |
| Warp Stabilizer Advanced: Crop Less ↔ Smooth More | Trade crop against smoothness without re-analysis | https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/warp-stabilizer-settings.html | missing | — | As above. |
| Auto Reframe (sequence) | AI follows the subject and writes position keyframes into a new sequence at a target aspect ratio | https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/add-auto-reframe-effect-to-a-sequence.html | missing | `frame` width/height is literal; a reframe is hand-keyed `x`/`y` | Subject tracking and a new-aspect copy of the project. |
| Auto Reframe (clip effect) | Same, on one clip in Effect Controls | https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/add-auto-reframe-effect-to-a-clip.html | missing | — | As above. |
| Auto Reframe Motion Tracking presets: Slower / Default / Faster | Trade stability against following fast action (fewer or more keyframes) | https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/add-auto-reframe-effect-to-a-sequence.html | missing | — | As above. |

### 8. Keying

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
| --- | --- | --- | --- | --- | --- |
| Ultra Key: key colour + Matte Generation | Chroma key with matte generation controls | https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/ultra-key-effect-parameters.html | partial | `chroma` effect: `color`, `tolerance`, `softness` (ADR-0088); recipe in `skills/montagent-footage` "Green-screen key" | Matte Generation's separate controls beyond tolerance and softness. Only one distance threshold and one soft band ship. |
| Ultra Key: Matte Cleanup | Choke/soften/contrast the generated matte | https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/ultra-key-effect-parameters.html | missing | No matte-cleanup parameter on `chroma` (schema `Effect` `chroma`: color, softness, spill, tolerance) | Choke or shrink, and soften the matte apart from the key. |
| Ultra Key: Spill Suppression | Removes key-colour spill from the subject | https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/ultra-key-effect-parameters.html | have | `chroma.spill` (ADR-0088) | One amount only. |
| Ultra Key: Color Correction | Saturation/hue/luminance correction inside the key | https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/ultra-key-effect-parameters.html | have | `saturation`, `tint`, `brightness`, `contrast` effects after `chroma` (ADR-0049); order checked by `R-CHROMA-AFTER-COLOUR` (ADR-0088) | Hue rotation is not an effect member. |
| Ultra Key: Output (Composite / Alpha / Status views) | Diagnostic views of the matte | https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/apply-and-customize-chromakey-using-the-ultra-key-effect.html | partial | `measure` keyed-alpha coverage per frame (ADR-0088); `frame --crop` for edges (skill workflow) | A rendered alpha-only (matte) view and a status view. Coverage is reported as numbers. |
| Color Key | Keys out pixels matching a colour; alpha only | https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/keying-effects.html | have | `chroma` (ADR-0088) keys any `color` | — |
| Luma Key | Transparency from brightness | https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/keying-effects.html | missing | No luma member; ADR-0088 admits a colour-distance key only | The whole mechanism. No ADR refuses it. |
| Track Matte Key | Another track's alpha or luma is the matte | https://helpx.adobe.com/ph_fil/premiere-pro/how-to/add-track-matte-key.html | refused | ADR-0150 §4: "A matte from another element is refused" | — |
| Image Matte Key | A still image's alpha or luma is the matte | https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/keying-effects.html | missing | Image or luma source in `mask` deferred (ADR-0152 §3, ADR-0040), not refused | An asset-referenced matte. |
| Difference Matte | Keys pixels matching a reference clip | https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/keying-effects.html | missing | — | The whole mechanism (it needs a second source inside an effect, the same block ADR-0152 §3 names). |

### 9. Picture-in-picture, crop, flip

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
| --- | --- | --- | --- | --- | --- |
| Picture-in-Picture presets | Presets bin entries that scale/position a clip over another | https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-effects/effect-presets.html | have (skill) | `skills/montagent-footage` "Circular picture-in-picture" recipe (video + circle `mask` + ring + pop) | A library of corner placements. The recipe gives one. |
| Effect presets (save / apply custom) | Save effect settings and keyframes as a preset | https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-effects/effect-presets.html | have (skill) | Skill recipes in `skills/montagent-motion` and `skills/montagent-footage` are copyable element JSON | A user-defined preset store. The format has none, by the closed-schema rule (ADR-0017). |
| Crop effect (Left/Top/Right/Bottom %, Zoom; Legacy in 26.0) | Removes edge pixels by percentage, keyframeable | https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/transform-effects.html | have | Element-local, keyable `mask` `shape: "rect"` rides the transform (ADR-0084); static frame-space `clip` (ADR-0025) | Crop values are pixels, not percentages. Crop-then-zoom-to-fill is computed by hand. |
| Edge Feather (Legacy in 26.0) | Soft edge on the frame border | https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/list-of-effects-and-transitions.html | have | `mask` `rect` with `feather` (ADR-0152) | — |
| Horizontal Flip | Mirrors each frame left to right | https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/transform-effects.html | have | `scale: [-1, 1]` (`crates/montagent-core/docs/format.md`; renderer test `crates/montagent-render/src/canvas.rs`) | — |
| Vertical Flip | Turns the clip upside down; not keyframeable | https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/transform-effects.html | have | `scale: [1, -1]` (as above) | — |

### 10. 3D / perspective, distortion of the plane, motion blur

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
| --- | --- | --- | --- | --- | --- |
| Basic 3D: Swivel | Rotation about the vertical axis | https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/perspective-effects.html | have | `swivel`, animatable (ADR-0167, ADR-0168) | — |
| Basic 3D: Tilt | Rotation about the horizontal axis | https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/perspective-effects.html | have | `tilt`, animatable (ADR-0167, ADR-0168) | — |
| Basic 3D: Distance to Image | Moves the image nearer or farther (a size change) | https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/perspective-effects.html | refused | ADR-0167 "Considered": a second size control; `perspective` keeps CSS's meaning | — |
| Basic 3D: Specular Highlight | Light glint from a fixed overhead-left light on the turned plane | https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/perspective-effects.html | missing | No lighting model; no ADR names it | The whole mechanism. |
| Corner Pin | Moves the four corners freely (perspective, skew, hinge) | https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/distort-effects.html | refused | ADR-0167 "Skew, corner pin, a camera, depth sorting. Still refused." | — |
| Transform effect: Skew / Skew Axis | Skews the clip along an axis | https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/transform-effects.html | refused | ADR-0012, ADR-0040 ("Skew — rejected on model grounds"), ADR-0167 | — |
| Transform effect (a second transform in the effect stack) | Position/scale/rotation/opacity as an effect, rendered in effect order | https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/transform-effects.html | refused | One transform per element, "never nested, never composed" (CONTEXT.md **Transform**, ADR-0012) | — |
| Motion blur | Premiere's per-clip motion blur is the Transform effect's shutter angle. Adobe's Premiere pages found here do not confirm it (ADR-0155 notes it is sourced only from Adobe Community posts) | https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/transform-effects.html | have | `motion_blur: {shutter, samples}` on every visual element, sub-frame accumulation (ADR-0155) | — (the Premiere side is unconfirmed, not the Montagent side) |

### 11. Nesting and Dynamic Link to After Effects

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
| --- | --- | --- | --- | --- | --- |
| Nested sequence with its own Motion transform | A sequence placed as a clip, transformed as one | Not retrieved (search budget); mechanism read in ADR-0166 §1 | missing | `group` is a render-inert label (ADR-0001); ADR-0166: a container is "not ruled out, and not admitted" | A space-only container (group transform). Its ruling (#632) is still open. |
| Nested sequence's local clock | Children timed relative to the nest | Not retrieved (search budget) | refused | ADR-0001 clock refusal, restated by ADR-0166 §2 | — |
| Dynamic Link: create new After Effects composition | Live-linked comp created from Premiere | https://helpx.adobe.com/premiere/desktop/use-premiere-with-other-apps/working-with-other-adobe-applications/create-a-dynamically-linked-composition.html | missing | No external-app source; ADR-0145 bars AE as a reason to enter | A live link. |
| Dynamic Link: replace clips with an After Effects composition | Selected clips become one linked comp; edits in AE update Premiere without rendering | https://helpx.adobe.com/premiere/desktop/use-premiere-with-other-apps/working-with-other-adobe-applications/replace-clips-with-a-dynamically-linked-after-effects-composition.html | missing | — | As above. |
| Dynamic Link: import an After Effects composition | Link an existing comp into the project | https://helpx.adobe.com/premiere/desktop/use-premiere-with-other-apps/working-with-other-adobe-applications/import-an-after-effects-composition.html | partial | `skills/montagent-prerender`: a Lottie export of an AE comp is pre-rendered to footage and placed as a `video` (ADR-0171) | The live link, and comps that do not export to Lottie. Lottie expressions are refused by the skill. |
| Dynamic Link: share assets between AE and Premiere | Two-way linked assets | https://helpx.adobe.com/premiere/desktop/use-premiere-with-other-apps/working-with-other-adobe-applications/share-assets-between-after-effects-and-premiere-using-dynamic-link.html | missing | — | As above. |
| Dynamic Link: relink offline compositions | Reconnect a moved comp | https://helpx.adobe.com/premiere/desktop/use-premiere-with-other-apps/working-with-other-adobe-applications/relink-a-dynamically-linked-composition.html | missing | — | As above (no link to relink). |
| Dynamic Link: make composition offline | Drop the live link for performance | https://helpx.adobe.com/premiere/desktop/use-premiere-with-other-apps/working-with-other-adobe-applications/make-a-dynamically-linked-composition-offline.html | missing | — | As above. |
| Render and Replace (linked comp → rendered media) | Bake a linked comp to a media file in the sequence | https://helpx.adobe.com/be_en/premiere/desktop/render-and-export/render-sequences-for-playback/render-and-replace-media-in-a-sequence.html | have | `skills/montagent-prerender` `build` + `place`, with a rebuild recipe and frame hashes (ADR-0156, ADR-0171) | — |
| Copy layers from After Effects to Premiere | Paste AE layers into a Premiere sequence | https://helpx.adobe.com/premiere/desktop/use-premiere-with-other-apps/working-with-other-adobe-applications/copy-from-after-effects-to-premiere.html | missing | — | The whole mechanism. |

## Notable gaps

1. **No tracker anywhere.** Mask tracking (four directions, three methods), Warp Stabilizer
   and Auto Reframe all rest on frame-to-frame motion analysis, and no tool or skill does it.
   No ADR refuses it. Render-time analysis would break ADR-0146 criterion 2, so the natural
   shape is a tool or skill that writes ordinary keyframes (`x`/`y`/`scale`/`rotation`, mask
   rect or `points`), the way the ducking skill writes volume keyframes (ADR-0177).
2. **Mask union and effect-scoped masks.** Masks only intersect, so Premiere's Add mode is
   missing. A mask cuts the whole element and cannot scope an effect, e.g. blur only a face.
   Both need a duplicate element today.
3. **22 of 27 blend modes**, refused "for now" by ADR-0147 until each has a forcing case.
   The likeliest next ones are `darken`/`lighten`, `soft_light` and `difference`, which
   ADR-0147 names.
4. **Anchor point is nine snap keywords.** There is no arbitrary pivot and no keyed pivot.
   Premiere's Anchor Point is free and keyframeable.
5. **Auto/Continuous Bezier and spatial interpolation.** Easing is per segment, so tangent
   continuity through a key is hand-computed and unchecked. A curved motion path has no
   spatial handles: `x` and `y` ease separately.
6. **Frame Blending and Optical Flow** are missing. Remapped or slowed video only samples the
   nearest earlier frame (ADR-0157 §3).
7. **Keying beyond chroma.** Luma Key, Image Matte Key, Difference Matte and Ultra Key's Matte
   Cleanup (choke/soften) are missing. Track Matte is refused (ADR-0150 §4).
8. **Object Masking (AI segmentation)** is missing, and so is the alpha-matte source it would
   need (deferred by ADR-0152 §3).
9. **Nesting with a group transform** is still undecided (ADR-0166; ruling on #632). A nest's
   local clock is refused (ADR-0001).
10. **Dynamic Link** has no live equivalent. The pre-render skill covers "render and replace"
    and Lottie-exported comps only.
