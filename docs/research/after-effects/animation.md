# After Effects keyframes and animation tools, scored against Montagent

Research for [#899](https://github.com/MBehtemam/Montagent/issues/899), on the map
[#896](https://github.com/MBehtemam/Montagent/issues/896). It lists what After Effects
documents about keyframes and animation tools, one feature per row, and where each feature
stands in Montagent at `main` `51aa4b5`.

## Sources and version

- **AE version.** After Effects 26.5 (September 2026) is the newest release on Adobe's
  [What's new in After Effects](https://helpx.adobe.com/uk/after-effects/desktop/what-s-new/whats-new.html)
  page (updated 28 September 2026). Neither 26.5 nor 26.3 changed keyframes or animation
  tools. Their animation-adjacent items are Curl Noise and pasting SVG as shapes, which
  belong to other censuses.
- **Adobe's pages used.** These are pages of the After Effects user guide.
  [Keyframe interpolation](https://helpx.adobe.com/after-effects/using/keyframe-interpolation.html),
  [Setting, selecting, and deleting keyframes](https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/animation-keyframes/setting-selecting-deleting-keyframes.html),
  [Editing, moving, and copying keyframes](https://helpx.adobe.com/ca/after-effects/desktop/animate-in-after-effects/animation-keyframes/editing-moving-copying-keyframes.html),
  [Speed](https://helpx.adobe.com/after-effects/using/speed.html),
  [Animation basics (Graph Editor)](https://helpx.adobe.com/after-effects/using/animation-basics-alt.html),
  [Assorted animation tools](https://helpx.adobe.com/after-effects/using/assorted-animation-tools.html),
  [Time-stretching and time-remapping](https://helpx.adobe.com/ca/after-effects/desktop/animate-in-after-effects/time-stretching-and-time-remapping/time-stretching-time-remapping.html),
  [Time effects](https://helpx.adobe.com/ca/after-effects/using/time-effects.html),
  [Animate along a path](https://helpx.adobe.com/si/after-effects/how-to/animate-along-path.html)
  and
  [Effects and animation presets overview](https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/effects-and-animation-presets/effects-animation-presets-overview.html).
- **Provenance caveat: read this first.** This session's egress proxy refused `helpx.adobe.com`,
  `www.adobe.com` and `web.archive.org` (CONNECT 403). So the Adobe text was read through the
  search engine's extracts of those pages, not fetched directly. Each claim below rests on an
  extract of the linked Adobe page, unless it is marked **(secondary)**. A secondary claim
  comes from a tutorial or forum post because no Adobe extract covered it. Some links carry
  a locale prefix (`ca/`, `si/`, `uk/`) because that is the copy the index returned; the
  content is the same as the English page.
- **Montagent side.** [`CONTEXT.md`](../../../CONTEXT.md), `docs/adr/`,
  [`format.md`](../../../crates/montagent-core/docs/format.md),
  `schema/montagent.schema.json` (`Ease`, `EaseName`, the `Keyframe*` definitions),
  [`skills/montagent/SKILL.md`](../../../skills/montagent/SKILL.md) (capability map and "Not
  by design"), and the other `skills/*`. Before marking a row "gap", the repo was grepped for
  these words: motion path, along a path, orient, roving, graph editor, speed graph, frame
  blend, optical flow, pixel motion, posterize time, exponential, motion sketch, smoother,
  amplitude, audio to keyframes, easy ease, time-reverse and sequence layers.

**Status words.** `covered`: the format spells it today. `partial`: some of it is spelled
and the rest is missing. `reachable`: a skill or a plain file edit gets the same result,
with no format support. `owned by #N`: an open map decides it. `refused by ADR-NNNN`: a
recorded no. `gap`: nothing spells it and nothing decides it.

## The census

### Temporal interpolation

| # | AE feature | What it does | Adobe doc | Montagent status | Evidence |
|---|---|---|---|---|---|
| 1 | Linear | Constant rate between two keyframes, so the motion starts and stops abruptly. | [Keyframe interpolation](https://helpx.adobe.com/after-effects/using/keyframe-interpolation.html) | covered | `"ease": "linear"`, published as `[0,0,1,1]` (`EaseName`; ADR-0012). |
| 2 | Bezier | Hand-set direction handles on each side of a keyframe, moving independently. | [Keyframe interpolation](https://helpx.adobe.com/after-effects/using/keyframe-interpolation.html) | covered | A raw `[x1,y1,x2,y2]` easing on each segment. `y` outside `[0,1]` is legal, so overshoot works (ADR-0012; `Ease` schema; capability map "with named easings or a cubic bezier (overshoot included)"). One difference: AE's handles belong to keyframes, while Montagent's easing belongs to the segment arriving at a keyframe (ADR-0012, ADR-0038). A segment's bezier holds the previous keyframe's outgoing handle and this keyframe's incoming one, so a single segment loses nothing. |
| 3 | Continuous Bezier | Handles you set yourself, locked so the rate is smooth through the keyframe. | [Keyframe interpolation](https://helpx.adobe.com/after-effects/using/keyframe-interpolation.html) | partial | Each segment's easing is independent (ADR-0012), so nothing ties the rate leaving a keyframe to the rate arriving. An agent can work out matching raw beziers on both sides, but no name spells this, no skill shows it, and `validate` doesn't check it. |
| 4 | Auto Bezier | AE sets the handles itself for a smooth rate through the keyframe. It is the default for spatial interpolation. | [Keyframe interpolation](https://helpx.adobe.com/after-effects/using/keyframe-interpolation.html) | gap | No easing name depends on neighbouring keyframes; the names are fixed curves (`EaseName`). A raw bezier could hold the computed result, but neither a skill nor a script computes it. |
| 5 | Hold | Holds the value until the next keyframe, then jumps. AE treats it as outgoing interpolation. | [Keyframe interpolation](https://helpx.adobe.com/after-effects/using/keyframe-interpolation.html) | covered | `"ease": "step"` on the keyframe arriving after the hold. It is named `step`, not `hold`, because easing is "entering" (ADR-0012). `findings.md` teaches `step` for an instant change. |

### Spatial interpolation and motion paths

| # | AE feature | What it does | Adobe doc | Montagent status | Evidence |
|---|---|---|---|---|---|
| 6 | Spatial interpolation (Linear / Bezier / Auto Bezier in space) | Position keyframes join along a curved motion path with spatial tangents, separate from timing. | [Keyframe interpolation](https://helpx.adobe.com/after-effects/using/keyframe-interpolation.html) | partial | `x` and `y` are two separate animatable properties (ADR-0012; format.md "Time"), so between two keyframes an element travels in a straight line. Different easings on `x` and `y` bend the path, but there are no spatial tangents. Anything more takes dense baked keyframes: `bake_rig.py`'s `hops` (a parabola) in `montagent-character`. |
| 7 | Motion path drawn or pasted from a path | Copy a mask or shape path into Position, and the layer travels along that drawn curve. | [Animate along a path](https://helpx.adobe.com/si/after-effects/how-to/animate-along-path.html) | gap | Only a `text` element follows a curve: its own `path` with `path_offset` (ADR-0161, ADR-0164; motion skill "Bend a title along a curve"). No other element can travel along a `path` element's points. No match for "motion path" or "along a path" in the repo. |
| 8 | Roving keyframes | Middle keyframes float in time so the speed along the path is even; only the first and last stay fixed. | [Keyframe interpolation](https://helpx.adobe.com/after-effects/using/keyframe-interpolation.html) (roving is spatial-only) | gap | Every keyframe `t` is a literal (ADR-0012), and #7 is missing too. An agent could compute even-speed times by hand, but no skill does. |
| 9 | Auto-orient (Orient Along Path) | Rotates the layer to follow the tangent of its motion path. | [Animate along a path](https://helpx.adobe.com/si/after-effects/how-to/animate-along-path.html) | gap | No field derives `rotation` from motion. Each letter of bent text sits along its curve (ADR-0161), but that orients text on its own path, not an element's travel. The workaround, `rotation` keyframes computed from the `x`/`y` slope, is written down nowhere. No match for "orient" in any relevant file. |
| 10 | Separate Dimensions | Splits Position into X, Y (and Z) properties, each with its own keyframes and curve. | [Animation basics](https://helpx.adobe.com/after-effects/using/animation-basics-alt.html) | covered | Position is always split: `x` and `y` are separate keyframe lists with their own easings (ADR-0012). AE separates Position only. Montagent's `scale` is one `[sx, sy]` property with one easing (ADR-0012), which matches AE's unseparated Scale. |

### Graph Editor, Easy Ease and keyframe velocity

| # | AE feature | What it does | Adobe doc | Montagent status | Evidence |
|---|---|---|---|---|---|
| 11 | Value graph | Plots a property's value over time, with handles to shape each segment. | [Animation basics](https://helpx.adobe.com/after-effects/using/animation-basics-alt.html) | reachable | No graph view: an agent edits the file with file tools (ADR-0011). The segment shape is the easing. `query --at` prints the value at any instant, and `frame --keyframes` shows a tile wherever a keyframe change first paints (ADR-0106). |
| 12 | Speed graph | Plots the rate of change over time. Its height is speed, not value. | [Speed](https://helpx.adobe.com/after-effects/using/speed.html) | reachable | `montagent-craft` "Durations and easing" has a table of peak speed per curve, and "Speed limits". `query --at` prints a derived `rate` only for a time remap (ADR-0157 §, format.md). No other property gets a speed readout. |
| 13 | Easy Ease, Easy Ease In, Easy Ease Out | Sets speed 0 and influence 33.33% on one or both sides of a keyframe. | [Speed](https://helpx.adobe.com/after-effects/using/speed.html) | covered | `ease-in`, `ease-out` and `ease-in-out` are named; Easy Ease is exactly the raw `[0.333,0,0.667,1]`. The named `ease-in-out` is `[0.42,0,0.58,1]` (CSS), not AE's 33%. ADR-0012; `montagent-craft` curve table. |
| 14 | Keyframe Velocity dialog (speed + influence, incoming and outgoing, Continuous toggle) | Sets the speed and influence entering and leaving a keyframe by number. | [Speed](https://helpx.adobe.com/after-effects/using/speed.html) | reachable | Speed and influence map to a segment's bezier by arithmetic: `x1` is the outgoing influence, and `y1 = speed × influence × Δt / Δv`; `x2`/`y2` mirror it. The format holds the result (ADR-0012). No skill teaches the mapping. |
| 15 | Stretch keyframe timing (Alt-drag, Graph Editor transform box) | Scales a group of keyframes' times, or values, proportionally. | [Editing, moving, and copying keyframes](https://helpx.adobe.com/ca/after-effects/desktop/animate-in-after-effects/animation-keyframes/editing-moving-copying-keyframes.html) | reachable | The agent rewrites the `t` values with file tools (ADR-0011). `shift` inserts or removes time at an instant (ADR-0012's SPLIT rule, ADR-0047) but has no proportional scale. |

### Keyframe assistants

| # | AE feature | What it does | Adobe doc | Montagent status | Evidence |
|---|---|---|---|---|---|
| 16 | Exponential Scale | Turns a linear scale into an exponential one, for a zoom whose apparent speed stays constant ("cosmic zoom"). | [Assorted animation tools](https://helpx.adobe.com/after-effects/using/assorted-animation-tools.html) | reachable | One bezier only approximates it. Linear keyframes, one per drawn frame, written by a script are exact at frame resolution; ADR-0035 makes keyframe grid alignment a review only. No skill covers it. `montagent-motion`'s "Push-in on a still" uses linear over 4–8%, where the difference can't be seen. No match for "exponential" in the skills. |
| 17 | Time-Reverse Keyframes | Reverses the selected keyframes within their own time span, across layers and properties. | [Time-stretching and time-remapping](https://helpx.adobe.com/ca/after-effects/desktop/animate-in-after-effects/time-stretching-and-time-remapping/time-stretching-time-remapping.html) | reachable | The agent mirrors the `t` values by file edit (ADR-0011: there are no field-level edit tools). Each easing moves to the other keyframe of its segment, because easing is "entering" (ADR-0012), and a raw bezier becomes `[1−x2, 1−y2, 1−x1, 1−y1]`. Neither a skill nor `format.md` says so. |
| 18 | Sequence Layers | Lays selected layers end to end, optionally overlapping with opacity crossfades. | Keyframe Assistant menu; Adobe page not returned **(secondary:** [ProVideo Coalition](https://www.provideocoalition.com/after_effects_tips_installment_3/)**)** | reachable | Every element has its own literal `start`/`end`, and cards that enter one after another are separate elements (ADR-0148; glossary **Stagger**). An overlap is a `transition` of kind `crossfade` (ADR-0059). `shift` re-times what follows (ADR-0047). |
| 19 | Convert Audio to Keyframes | Measures audio amplitude across the work area and writes it as keyframes (Left, Right, Both Channels) on a null. | [Assorted animation tools](https://helpx.adobe.com/after-effects/using/assorted-animation-tools.html) | gap | No script writes an amplitude envelope. `montagent-craft`'s `beat_grid.py` finds onsets, tempo and downbeats (workaround #547) but writes no keyframes from level. The audio map (#795) rules out beat detection as detector-driven; a per-frame RMS envelope is deterministic and isn't mentioned there. |

### Assorted animation tools

| # | AE feature | What it does | Adobe doc | Montagent status | Evidence |
|---|---|---|---|---|---|
| 20 | Motion Sketch | Records a layer's position, and how fast you draw, as a Position keyframe on every frame. Smoothing thins them out. | [Assorted animation tools](https://helpx.adobe.com/after-effects/using/assorted-animation-tools.html) | reachable | There is no live capture: an agent has no pointer, and Montagent's tools read, check and render (ADR-0011). The output, one `x`/`y` keyframe per frame from a list of points, is what the skills' generator scripts already write (`bake_rig.py`; trailer.md "write them with a generator script"). |
| 21 | Smoother | Removes bumps and extra keyframes from motion paths and value or speed curves, within a tolerance. | [Assorted animation tools](https://helpx.adobe.com/after-effects/using/assorted-animation-tools.html) | gap | No script reduces or smooths keyframes. `shift` collapses runs of three or more equal keyframes (ADR-0012 SPLIT step 7), and nothing else does. Dense lists are common: the baked owl carries 709 keyframes (#592 baseline). No match for "smoother" in the repo. |
| 22 | Wiggler | Adds random deviations between existing keyframes, at a given frequency and magnitude, as keyframes. | [Assorted animation tools](https://helpx.adobe.com/after-effects/using/assorted-animation-tools.html) | reachable | `montagent-motion` `references/trailer.md` "Shake" writes per-frame `x`/`y` offsets from a generator script. Those are literal keyframes, the same output the Wiggler writes. A procedural wiggle *field* is refused by ADR-0156 ("Camera shake and wiggle. Refused: they are procedural motion the file would not show"). |

### Time

| # | AE feature | What it does | Adobe doc | Montagent status | Evidence |
|---|---|---|---|---|---|
| 23 | Time-stretch | Speeds up or slows down a whole layer by one constant factor. | [Time-stretching and time-remapping](https://helpx.adobe.com/ca/after-effects/desktop/animate-in-after-effects/time-stretching-and-time-remapping/time-stretching-time-remapping.html) | covered | `speed` on `video` and `audio`, checked exactly (ADR-0020, ADR-0045; glossary **Speed**). |
| 24 | Time remapping | Keyframes which source frame shows, for ramps, freezes and reverse. | [Time-stretching and time-remapping](https://helpx.adobe.com/ca/after-effects/desktop/animate-in-after-effects/time-stretching-and-time-remapping/time-stretching-time-remapping.html) | partial | `source_time` on `video` is AE's own spelling: it keys the source time, not the rate (ADR-0157; glossary **Time remap**). Missing: sound that follows the curve (a remapped video must be silent, `E-REMAP-AUDIBLE`; see #38) and remapping a container (see #37). |
| 25 | Freeze Frame | Holds one source frame for the whole layer. | [Time-stretching and time-remapping](https://helpx.adobe.com/ca/after-effects/desktop/animate-in-after-effects/time-stretching-and-time-remapping/time-stretching-time-remapping.html) | covered | A literal `source_time` freezes the whole element. A flat pair of keyframes freezes a stretch. `overrun: "hold"` freezes the last frame (ADR-0157, ADR-0020). |
| 26 | Reverse a footage layer (Time-Reverse Layer) | Plays the layer backwards; its keyframes reverse too. | [Time-stretching and time-remapping](https://helpx.adobe.com/ca/after-effects/desktop/animate-in-after-effects/time-stretching-and-time-remapping/time-stretching-time-remapping.html) | covered | A falling `source_time` segment, with no `reverse` field (ADR-0157). The element's other keyframes don't flip; that is #17. |
| 27 | Frame blending: Frame Mix | Crossfades neighbouring source frames to make the in-betweens when the frame rates differ. | [Time-stretching and time-remapping](https://helpx.adobe.com/ca/after-effects/desktop/animate-in-after-effects/time-stretching-and-time-remapping/time-stretching-time-remapping.html) | gap | A slowed or remapped video shows "the last source frame starting at or before" the resolved instant (ADR-0096, ADR-0157). `motion_blur` doesn't blur footage (glossary **Motion blur**; ADR-0155). No match for "frame blend" in the repo. |
| 28 | Frame blending: Pixel Motion | Tracks pixel motion between frames and draws new in-between frames (optical flow). | [Time-stretching and time-remapping](https://helpx.adobe.com/ca/after-effects/desktop/animate-in-after-effects/time-stretching-and-time-remapping/time-stretching-time-remapping.html) | gap | As #27, with nothing for "optical flow" or "pixel motion" in the repo. `montagent-prerender` is for code-drawn looks, not reworking footage. |
| 29 | Timewarp | An effect that changes speed with keyframes and builds in-betweens with Whole Frames, Frame Mix or Pixel Motion, optionally adding motion blur. | [Time effects](https://helpx.adobe.com/ca/after-effects/using/time-effects.html) | partial | The speed part is `source_time` (ADR-0157). The in-between part is #27/#28, and the footage blur is missing (#27). #905 holds the rest of the Time effects category. |
| 30 | Posterize Time | Locks a layer to a lower frame rate, so it moves in holds. | [Time effects](https://helpx.adobe.com/ca/after-effects/using/time-effects.html) | reachable | `step` keyframes at the target rate on each animated property, from a script. For footage, `source_time` as a staircase of `step` keyframes (ADR-0012, ADR-0157). No skill shows it. Not to be confused with the colour `posterize` effect (ADR-0156). |

### Presets, copying and keyframe housekeeping

| # | AE feature | What it does | Adobe doc | Montagent status | Evidence |
|---|---|---|---|---|---|
| 31 | Animation presets (`.ffx`) | Saves keyframes, effects and expressions from a layer and applies them to other layers. | [Effects and animation presets overview](https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/effects-and-animation-presets/effects-animation-presets-overview.html) | reachable | `montagent-motion` recipes (Pop, Hold then clear, Reveal by wipe, Flip a card…) and scripts (`type_on.py`) play the preset's part. The format has no preset or template construct: the schema is closed (ADR-0017), and there is no asset table (ADR-0002). |
| 32 | Copy and paste keyframes between layers and properties | Copies keyframes to another property or layer at the current time. | [Editing, moving, and copying keyframes](https://helpx.adobe.com/ca/after-effects/desktop/animate-in-after-effects/animation-keyframes/editing-moving-copying-keyframes.html) | reachable | Copying a keyframe list is a file edit (ADR-0011). The agent offsets each `t` by the difference between the two elements' starts: `t` is written in timeline milliseconds and travels with its element (glossary **Keyframe**; ADR-0012). The closed schema rejects a list pasted onto a property of the wrong type (ADR-0017, ADR-0146). |
| 33 | Setting, selecting and deleting keyframes; keyframe navigator | Adds a keyframe at the current value without changing it, and selects or deletes keyframes. | [Setting, selecting, and deleting keyframes](https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/animation-keyframes/setting-selecting-deleting-keyframes.html) | covered | A keyframe list of `{t, v, ease}` records, in ascending `t` (ADR-0012, ADR-0082). A keyframe that changes nothing takes its value from `query --at` (ADR-0145's fourth test: "An in-between value is singled out by adding a keyframe"). |
| 34 | Any property can take keyframes (the stopwatch) | Nearly every layer, effect and mask property can be animated. | [Setting, selecting, and deleting keyframes](https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/animation-keyframes/setting-selecting-deleting-keyframes.html) | partial | The list of animatable properties is closed (ADR-0146; format.md "Time"): transform, shape size and paint, text paint and `letter_spacing`, every effect number, mask points, gradients, `source_time`, `volume`. Static by rule: the box of an `image`, `video`, `text` or `path`; `clip` (ADR-0025); enums; run and highlight paint; `origin` (ADR-0013); audio-effect parameters (#795). |
| 35 | Cycling keyframes (`loopOut`/`loopIn` cycle, ping-pong) | Repeats a span of keyframes for as long as the layer lasts. | AE does this with expressions only; see [#906](https://github.com/MBehtemam/Montagent/issues/906) | refused by ADR-0148 | "One animation started many times" is a repeat, and a repeat does not enter (ADR-0148). Each cycle is written out (`montagent-motion` `references/logo-loop.md`). |
| 36 | Expressions on keyframed properties (`wiggle()`, `valueAtTime`, links) | Computes a property's value with code, alongside or instead of its keyframes. | See [#906](https://github.com/MBehtemam/Montagent/issues/906) | refused by ADR-0145 | They fail the literal-values invariant (ADR-0145; "Not by design": "Keyframe expressions, or links between values. Write literal keyframes"). |
| 37 | Time-remapping a precomp (a nested composition's own clock) | Retimes a whole nested composition with one curve. | [Time-stretching and time-remapping](https://helpx.adobe.com/ca/after-effects/desktop/animate-in-after-effects/time-stretching-and-time-remapping/time-stretching-time-remapping.html) | owned by #591 | The provisional `nest` is space-only and "has no clock" (#632 resolution; ADR-0166). ADR-0005 allows one clock and no nested timelines. |
| 38 | Audio that follows a time remap | A remapped layer's audio speeds up, slows down and reverses with the curve. | [Time-stretching and time-remapping](https://helpx.adobe.com/ca/after-effects/desktop/animate-in-after-effects/time-stretching-and-time-remapping/time-stretching-time-remapping.html) | owned by #795 | ADR-0157 left varispeed audio for "a separate capability", and `E-REMAP-AUDIBLE` silences the element. #795's "Pitch preservation under `speed`" names this interaction. |
| 39 | Animating the Anchor Point | The pivot is a transform property that takes keyframes. | [Animation basics](https://helpx.adobe.com/after-effects/using/animation-basics-alt.html) | owned by #591 | `origin` is nine static keywords (ADR-0013). #591's free pivot `"origin": [px, py]` is provisional and static too (#596, #822). |
| 40 | Keyframe colour labels (Select Keyframe Label Group) | Colours keyframes and selects them by colour. | [Setting, selecting, and deleting keyframes](https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/animation-keyframes/setting-selecting-deleting-keyframes.html) | gap | A keyframe record is closed: `t`, `t_from`, `v`, `ease` (`Keyframe*` schema; ADR-0017). The nearest things are `group`, which labels an element and renders nothing, and `t_from`, recorded intent (ADR-0086). |

### Counts

| Status | Rows | Count |
|---|---|---|
| covered | 1, 2, 5, 10, 13, 23, 25, 26, 33 | 9 |
| partial | 3, 6, 24, 29, 34 | 5 |
| reachable by a skill or workaround | 11, 12, 14, 15, 16, 17, 18, 20, 22, 30, 31, 32 | 12 |
| owned by an open map | 37, 38 (#795), 39 | 3 |
| refused by an ADR | 35 (ADR-0148), 36 (ADR-0145) | 2 |
| gap | 4, 7, 8, 9, 19, 21, 27, 28, 40 | 9 |
| **total** | | **40** |

## Gaps worth a decision ticket

Every proposal below must first pass ADR-0145: precedent in CapCut or Premiere, or an
owner-accepted prototype, plus the four invariants. How AE counts as precedent is #897's
question.

### A. Motion along a path (rows 6, 7, 8, 9)

This is the largest hole in the area. An element can't travel along a drawn curve, can't
face along it, and can't move at even speed along it. Today all three need dense baked
keyframes. Bent text already has the parts: an inline `path`, `path_offset` measured as a
fraction of arc length, and orientation along the normal (ADR-0161, ADR-0164). The question
is whether those parts extend to every element: an inline path, a keyable fraction along
it, and an optional orient flag. That shape keeps literal values, and arc-length parameters
make roving's even speed the default. Premiere's Program Monitor draws a motion path with
spatial handles for Position keyframes. That is recalled, not checked here, and it would be
precedent for row 6. Neither tool has an element follow a separately drawn path, so row 7
probably needs a prototype under ADR-0145. Separately, spatial Bezier tangents on `x`/`y` keyframes
(row 6) would be a second spelling of the same thing. Any ticket should decide between the
two, not take both.

### B. Easing across keyframes (rows 3, 4, and 14 as teaching)

Every segment is eased on its own, so a move through three or more keyframes stalls or
kinks at each keyframe unless the agent matches the tangents by hand. Two shapes are open: a
named easing that depends on its neighbours (`auto`, `continuous`), or a skill script or
`montagent-craft` arithmetic that writes matched raw beziers. The neighbour-dependent name
must answer ADR-0145's exact-string test: editing one keyframe changes how its neighbours
render. A script avoids that. Row 14's speed and influence mapping belongs in the same
teaching.

### C. Footage in-betweens (rows 27, 28, 29)

Slow footage under `speed` < 1 or `source_time` stutters on repeated frames, and nothing
blends them. Frame Mix is a fixed, deterministic crossfade of the two neighbouring source
frames, so it could be a static per-element field next to `motion_blur`. Premiere has frame
blending and optical flow as Time Interpolation, so this has precedent. That is recalled,
not checked here; the ticket should confirm it on Adobe's Premiere page. Pixel Motion (optical
flow) raises ADR-0006's question, whether `validate` can check it, and whether it renders the
same across builds. That probably makes it a prerender route rather than a field. A ticket
should decide Frame Mix on its own.

### D. Generator scripts the skills don't ship (rows 16, 17, 19, 21, 30)

None of these needs the format to change. Each output is literal keyframes, which is how
ADR-0177 already handled ducking. The candidates:

- an amplitude-envelope script: per-frame RMS to keyframes on any animatable property
  (row 19). It is deterministic, so #795's out-of-scope line on detectors doesn't reach it;
- a keyframe reducer and smoother with a stated tolerance (row 21), for baked rigs and
  generated shake;
- recipes for exponential zoom, posterize-time stepping and keyframe time-reversal (rows 16,
  30, 17), covering the easing mirror, which is easy to get wrong.

One ticket could choose which of these get scripts and which get recipe text.

### Not worth a ticket

- Row 40, keyframe colour labels: it is a feature of AE's GUI, and `group` and `t_from`
  cover what an agent needs.
- Rows 11, 12 and 15, the Graph Editor views and stretching keyframe times: no GUI, by
  ADR-0011. `query --at`, `frame --keyframes` and file edits do the same work. Speed readouts
  beyond `source_time`'s `rate` could be a small `query` improvement if an agent ever needs
  one.
