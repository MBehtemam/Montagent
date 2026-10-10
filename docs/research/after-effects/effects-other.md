# After Effects' generate, distort, stylize, simulation, transition, time, noise, perspective and matte effects, scored against Montagent

Research for [#905](https://github.com/MBehtemam/Montagent/issues/905), part of the map
[#896](https://github.com/MBehtemam/Montagent/issues/896). **This note decides nothing.** It scores
every effect in nine After Effects categories against Montagent's `main` (commit `51aa4b5`). It
names the Audio, Obsolete and Immersive Video categories, and it ends with the gaps worth a
decision ticket.

## Sources, version and limits

- **Verified against Adobe's live pages on 2026-10-10.** Membership comes from the
  [effect list][list] (updated 13 May 2026), which lists every effect by category. Each category
  page was read too, and each row's description was checked against Adobe's text. The pages
  are dated: Distort, Perspective, Matte and Obsolete 27 June 2025; Generate, Simulation, Time
  and Transition 26 June 2025; Noise and Grain 17 June 2026; Stylize 7 October 2026; Audio
  11 September 2026; Immersive Video 27 June 2025. The CC effects are described on Adobe's
  [Cycore plugins][cc] page (16 June 2025), not on the category pages, which only name them, so
  CC rows link there. That puts the list at After Effects 26.x, the current user guide.
- **Where the effect list and a category page disagree**, this note follows the list and says
  so:
  - The Noise and Grain page documents **Curl Noise**, which the older effect list does not
    have yet. It is scored.
  - The Matte page has no section for **Mocha shape** or **Refine Hard Matte**. The list has
    both, so both are scored.
  - The Stylize page shows **Contour Paint**, but only as an After Effects (beta) feature. It
    is not in the list and is not scored.
  - The Time page's note names **CC Time Blend** and **CC Time Blend FX**. Neither the list nor
    the Cycore page has them, so they are not scored.
  - The Obsolete page and the list differ by one entry each (Fast Blur, Reduce Interlace
    Flicker; see below).
- **Still unverified:**
  - **CC Ball Action** (Simulation), **CC Glass Wipe** (Transition) and **Refine Hard Matte**
    (Matte) are listed by Adobe, but no helpx page describes them. Their one-line descriptions
    are this note's own.
  - **Mocha shape** is scored from one sentence on Adobe's compositing overview ("converts
    paths from mocha-AE into mattes").
- **Anchors:** a row links to its effect's section when the page has a stable anchor for it.
  Displacement Map and Echo link to the page top. Displacement Map's anchor id is reused by the
  Detail-preserving Upscale section, and Echo has no anchor.
- **The Montagent side** comes from the repo: `CONTEXT.md` (**Effect**, **Transition**,
  **Direction**, **Motion blur**, **Time remap**, **Seed**, **Pre-rendered footage**),
  `crates/montagent-core/docs/format.md` with its pages `format/compositing.md` and
  `format/transitions.md`, `skills/montagent/SKILL.md` (the capability map and "Not by design"),
  `skills/montagent-prerender/SKILL.md`, `skills/montagent-motion/references/trailer.md`, and
  the ADRs. Each "gap" was grepped across `docs/adr`, `CONTEXT.md`, `skills`, the format pages
  and `schema` first.
- **Earlier census:** [#703](https://github.com/MBehtemam/Montagent/issues/703),
  `docs/research/skia-reach/named-effects-candidates.md`. It holds the Skia route, parameters and
  seed/time needs for about 20 of these effects. They are not repeated here.

## How a row is scored

| Status | Meaning | Typical evidence |
|---|---|---|
| **covered** | A native key gives the result | `format.md` key, ADR |
| **partial** | A native key gives part of it; the missing part is named | key + what is missing |
| **reachable** | No native key, but a shipped skill or a documented recipe reaches it | `montagent-prerender` (its description names "distortions, mosaic, lens flare, a particle or shader piece" and "a procedural pattern"), `trailer.md`, copies written out (ADR-0148) |
| **map #N** | An open map owns the decision | #591 transforms, #795 audio |
| **refused ADR-NNNN** | An accepted ADR refuses it, or the mechanism it needs | ADR number |
| **gap** | None of the above, and no skill documents a route | the grep that came back empty |

Three facts settle most rows:

1. **The `effects` vocabulary is twelve members:** `blur`, `shadow`, `mask`, `chroma`, `tint`,
   `saturation`, `brightness`, `contrast`, `grain`, `posterize`, `glow`, `directional_blur`
   (CONTEXT **Effect**; ADR-0040, ADR-0049, ADR-0088, ADR-0156). A new member joins by its own
   ADR if it passes three tests: it can't be composed, it is one Skia filter or a small shader,
   and its reach is declared (ADR-0156 §2).
2. **No generator element exists.** A texture is a `rect` with an effect and a `blend`
   (ADR-0156 §1). A look outside the vocabulary is pre-rendered footage (ADR-0156 §7,
   `montagent-prerender`). ADR-0156 explicitly **defers** the distortions (wave warp, ripple,
   twirl, spherize, mirror, magnify) because their edge rule is undecided. It defers sharpen,
   emboss, mosaic and RGB split to keep the first batch small. It **refuses** Displacement Map,
   camera shake and wiggle.
3. **No element's pixels decide another's** (ADR-0150 §4). That refuses track mattes. It also
   refuses every effect that reads a second layer: a displacement, gradient, texture or
   time map.

A prerender piece is drawn outside the project. Its code may read a source file (its spec's
`files`), so it can distort footage. The footage is fixed, though: changing the look means
running the recipe again.

[list]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/effects-and-animation-presets/effect-list.html
[cc]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/cycore-plugins.html
[dist]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/distort-effects.html
[gen]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/generate-effects.html
[noise]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/noise-grain-effects.html
[persp]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/perspective-effects.html
[sim]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/simulation-effects.html
[styl]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/stylize-effects.html
[time]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/time-effects.html
[trans]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/transition-effects.html
[matte]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/matte-effects.html
[audio]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/audio-effects.html
[obs]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/obsolete-effects.html
[vr]: https://helpx.adobe.com/after-effects/desktop/immersive-video-and-vr/immersive-video-effects/vr-effects.html

## Distort (37)

| Effect | What it does | Adobe | Status | Evidence |
|---|---|---|---|---|
| Bezier Warp | Reshapes the layer by a closed Bezier curve on its boundary (four segments, 12 points) | [Distort][d-bezier] | reachable | prerender (distortions); free warp undecided |
| Bulge | Bulges the image toward or away from the viewer around a point | [Distort][d-bulge] | reachable | prerender; spherize deferred in ADR-0156 (edge rule) |
| CC Bend It | Bends the layer between two points, like a flexed rod | [Cycore][cc] | reachable | prerender |
| CC Bender | Curves the layer along its vertical axis from one control point | [Cycore][cc] | reachable | prerender |
| CC Blobbylize | Soft, jelly-like liquid look | [Cycore][cc] | reachable | prerender; its blob-layer input would be refused in-format by ADR-0150 §4 |
| CC Flo Motion | Smooth mesh warp around two control points | [Cycore][cc] | reachable | prerender |
| CC Griddler | Cuts the layer into a grid of tiles | [Cycore][cc] | reachable | prerender |
| CC Lens | Fisheye lens from a centre point | [Cycore][cc] | reachable | prerender; spherize deferred ADR-0156 |
| CC Page Turn | Page curl or flip, like turning a sheet of paper | [Cycore][cc] | reachable | prerender; a flat flip is `swivel` (ADR-0167) |
| CC Power Pin | Pins four corners for perspective and shape changes | [Cycore][cc] | refused ADR-0167 | "Skew, corner pin … still refused" |
| CC Ripple Pulse | Circular ripple waves from the centre | [Cycore][cc] | reachable | prerender; ripple deferred ADR-0156 |
| CC Slant | Skews the layer diagonally | [Cycore][cc] | refused ADR-0167 | skew refused (ADR-0012, ADR-0167) |
| CC Smear | Drags part of the layer between two points | [Cycore][cc] | reachable | prerender |
| CC Split | Splits the layer down the middle and moves the halves apart | [Cycore][cc] | reachable | two copies with `mask` and keyed `x`/`y` (ADR-0148 copies) |
| CC Split 2 | CC Split with angle and edge-softness controls | [Cycore][cc] | reachable | as CC Split; a soft edge is `mask` `feather` (ADR-0152) |
| CC Tiler | Repeats the layer across the composition as tiles | [Cycore][cc] | reachable | copies as elements (ADR-0148) |
| Corner Pin | Moves four corners to stretch, skew or simulate perspective | [Distort][d-corner] | refused ADR-0167 | corner pin refused; ADR-0156 sent it to the 2.5D decision |
| Detail-preserving Upscale | Scales a layer up while keeping sharpness and detail | [Adobe page][d-dpu] | gap | resampling fixed by ADR-0132; no upscaler member |
| Displacement Map | Shifts pixels by a control layer's colour values | [Distort][dist] | refused ADR-0156 | "reads another element's pixels" (ADR-0150 §4) |
| Liquify | Brush tools push, pull, twirl, pucker and bloat areas | [Distort][d-liquify] | reachable | prerender |
| Magnify | Enlarges part of the image like a loupe, or all of it | [Distort][d-magnify] | reachable | prerender; magnify deferred ADR-0156 |
| Mesh Warp | Warps by a grid of Bezier patches | [Distort][d-mesh] | reachable | prerender |
| Mirror | Reflects one side onto the other about a line | [Distort][d-mirror] | reachable | copy with `scale: [-1, 1]` + `mask`; mirror deferred ADR-0156 |
| Offset | Pans the image within the layer, wrapping at the edges | [Distort][d-offset] | reachable | prerender; wrap is the undecided edge rule (ADR-0156) |
| Optics Compensation | Adds or removes camera lens distortion | [Distort][d-optics] | reachable | prerender |
| Polar Coordinates | Maps pixels between rectangular and polar coordinates | [Distort][d-polar] | reachable | prerender |
| Reshape | Morphs one mask shape into another, dragging the image | [Distort][d-reshape] | reachable | prerender; a shape morph (no pixels) is `path` `points` (ADR-0162) |
| Ripple | Concentric ripples moving from a centre point | [Distort][d-ripple] | reachable | prerender; deferred ADR-0156 (keyed phase, §3) |
| Rolling Shutter Repair | Removes rolling-shutter (CMOS) distortion from footage | [Adobe page][d-rsr] | gap | no footage-analysis route; grep empty |
| Smear | Moves a masked area, stretching the image around it | [Distort][d-smear] | reachable | prerender |
| Spherize | Wraps a region of the image onto a sphere | [Distort][d-spherize] | reachable | prerender; deferred ADR-0156 |
| Transform | Anchor, position, scale, skew, rotation, opacity, shutter angle | [Distort][d-transform] | partial | flat transform (ADR-0012), `motion_blur` (ADR-0155); skew refused ADR-0167 |
| Turbulent Displace | Fractal-noise displacement, Evolution animates | [Distort][d-turb] | reachable | prerender; seed + time (ADR-0156 §3) |
| Twirl | Rotates more sharply at the centre than at the edges | [Distort][d-twirl] | reachable | prerender; deferred ADR-0156 |
| Warp | Arc, bulge, flag and other preset warps, as in Illustrator | [Distort][d-warp] | reachable | prerender |
| Warp Stabilizer | Analyses and stabilises shaky footage | [Adobe page][d-ws] | gap | no tracking or analysis route; grep empty |
| Wave Warp | A wave travelling across the image, self-animating | [Distort][d-wave] | reachable | prerender; deferred ADR-0156 (keyed phase) |

[d-bezier]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/distort-effects.html#bezier_warp_effect
[d-bulge]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/distort-effects.html#bulge_effect
[d-corner]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/distort-effects.html#corner_pin_effect
[d-dpu]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/detail-preserving-upscale-effect.html
[d-liquify]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/distort-effects.html#liquify_effect
[d-magnify]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/distort-effects.html#magnify_effect
[d-mesh]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/distort-effects.html#mesh_warp_effect
[d-mirror]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/distort-effects.html#mirror_effect
[d-offset]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/distort-effects.html#offset_effect
[d-optics]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/distort-effects.html#optics_compensation_effect
[d-polar]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/distort-effects.html#polar_coordinates_effect
[d-reshape]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/distort-effects.html#reshape_effect
[d-ripple]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/distort-effects.html#ripple_effect
[d-rsr]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/rolling-shutter-repair.html
[d-smear]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/distort-effects.html#smear_effect
[d-spherize]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/distort-effects.html#spherize_effect
[d-transform]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/distort-effects.html#transform_effect
[d-turb]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/distort-effects.html#turbulent_displace_effect
[d-twirl]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/distort-effects.html#twirl_effect
[d-warp]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/distort-effects.html#warp_effect
[d-ws]: https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/track-motion/tracking-stabilizing-motion-cs5.html#stabilize_motion_with_the_warp_stabilizer_effect
[d-wave]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/distort-effects.html#wave_warp_effect

## Generate (26)

| Effect | What it does | Adobe | Status | Evidence |
|---|---|---|---|---|
| 4-Color Gradient | Four-colour gradient from four animatable points | [Generate][g-4c] | partial | `gradient` is linear or radial only (ADR-0149); stacked radial rects approximate it |
| Advanced Lightning | Electrical discharge; animated by its properties, not by itself | [Generate][g-lightning] | reachable | prerender (procedural) |
| Audio Spectrum | Frequency magnitudes of an audio layer, drawn as bars or lines | [Generate][g-spectrum] | reachable | prerender, or keyed `rect` heights from a script (as ducking, ADR-0177); no shipped script |
| Audio Waveform | Waveform of an audio layer, optionally along a mask path | [Generate][g-waveform] | reachable | as Audio Spectrum; keyed `path` `points` |
| Beam | Laser-like beam that shoots or stays fixed at one end | [Generate][g-beam] | partial | `path` stroke + `trim_*` + `glow`; no tapered core |
| CC Glue Gun | Hot-glue strokes that drip, flow and trail | [Cycore][cc] | reachable | prerender |
| CC Light Burst 2.5 | Burst of light from a single point, like a flash | [Cycore][cc] | reachable | prerender |
| CC Light Rays | Beams of light radiating from a source | [Cycore][cc] | reachable | prerender |
| CC Light Sweep | Light streak moving across the layer | [Cycore][cc] | reachable | `trailer.md` "Light sweeps": gradient `rect`, `blend: screen` |
| CC Threads | Woven strands or fibres | [Cycore][cc] | reachable | prerender (procedural) |
| Cell Pattern | Cellular-noise texture, static or moving | [Generate][g-cell] | reachable | prerender; would need seed (ADR-0156 §3) |
| Checkerboard | Checker pattern of rectangles, half transparent | [Generate][g-checker] | reachable | `rect` copies (ADR-0148), or prerender |
| Circle | Solid disk or ring | [Generate][g-circle] | covered | `ellipse`, `stroke`, `mask` `circle` |
| Ellipse | Draws an ellipse (a soft-edged ring) | [Generate][g-ellipse] | covered | `ellipse` + `stroke` + `glow`/`shadow` |
| Eyedropper Fill | Fills with a colour sampled from the layer | [Generate][g-eyedrop] | reachable | agent reads the colour with `frame` and writes a literal `tint` (literal values, ADR-0145) |
| Fill | Fills masks (or the layer) with a colour | [Generate][g-fill] | covered | `tint` with `amount: 1` (ADR-0049), with `mask` for a masked area |
| Fractal | Mandelbrot or Julia set | [Generate][g-fractal] | reachable | prerender |
| Gradient Ramp | Linear or radial colour ramp | [Generate][g-ramp] | covered | `gradient` linear/radial on `fill` (ADR-0149) |
| Grid | Customisable grid, as colour or alpha | [Generate][g-grid] | reachable | `rect`/`path` copies (ADR-0148), `stroke_dash` |
| Lens Flare | Simulated refraction of a bright light in the lens | [Generate][g-flare] | reachable | `trailer.md` "flares" (gradients, PNG plates, `screen`); prerender |
| Paint Bucket | Fills an area with a solid colour by tolerance | [Generate][g-bucket] | gap | reads pixels for a region; no route documented |
| Radio Waves | Waves radiating from a point | [Generate][g-radio] | reachable | `ellipse` copies with keyed `scale`/`opacity` (ADR-0148) |
| Scribble | Hand-drawn zig-zag fill or stroke of a closed mask | [Generate][g-scribble] | partial | `path` + `trim_end` draws on (ADR-0160); zig-zag points written by hand |
| Stroke | Strokes the path of one or more masks | [Generate][g-stroke] | covered | `path` `stroke`, `trim_start`/`trim_end` (ADR-0160) |
| Vegas | Running lights and pulses along a path or edges | [Generate][g-vegas] | covered | `stroke_dash` + keyed `stroke_dash_offset` (ADR-0158) |
| Write-on | Animates a brush stroke, like handwriting | [Generate][g-writeon] | covered | `path` + keyed `trim_end` (ADR-0160) |

[g-4c]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/generate-effects.html#4_color_gradient_effect
[g-lightning]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/generate-effects.html#advanced_lightning_effect
[g-spectrum]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/generate-effects.html#audio_spectrum_effect
[g-waveform]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/generate-effects.html#audio_waveform_effect
[g-beam]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/generate-effects.html#beam_effect
[g-cell]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/generate-effects.html#cell_pattern_effect
[g-checker]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/generate-effects.html#checkerboard_effect
[g-circle]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/generate-effects.html#circle_effect
[g-ellipse]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/generate-effects.html#ellipse_effect
[g-eyedrop]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/generate-effects.html#eyedropper_fill_effect
[g-fill]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/generate-effects.html#fill_effect
[g-fractal]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/generate-effects.html#fractal_effect
[g-ramp]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/generate-effects.html#GradientRampeffectCC
[g-grid]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/generate-effects.html#grid_effect
[g-flare]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/generate-effects.html#lens_flare_effect
[g-bucket]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/generate-effects.html#paint_bucket_effect
[g-radio]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/generate-effects.html#radio_waves_effect
[g-scribble]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/generate-effects.html#scribble_effect
[g-stroke]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/generate-effects.html#stroke_effect
[g-vegas]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/generate-effects.html#vegas_effect
[g-writeon]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/generate-effects.html#write_on_effect

## Noise & Grain (13)

| Effect | What it does | Adobe | Status | Evidence |
|---|---|---|---|---|
| Add Grain | Synthesises film grain with presets | [Noise & Grain][n-add] | partial | `grain` (seed, amount, size, mono; ADR-0156); no stock presets, softness or per-channel intensity |
| Curl Noise | Smooth, fluid animated 2D noise; input from itself, the layer or another layer | [Noise & Grain][n-curl] | reachable | prerender (procedural); a time axis as Fractal Noise; its Other Layer input would be refused in-format (ADR-0150 §4); grep "curl" empty |
| Dust & Scratches | Makes dissimilar pixels within a radius like their neighbours | [Noise & Grain][n-dust] | gap | no median member; grep empty |
| Fractal Noise | Perlin-noise greyscale texture, Evolution animates | [Noise & Grain][n-fractal] | reachable | prerender; #703: Evolution needs a time input ADR-0156 §3 refuses |
| Match Grain | Matches the noise of another image | [Noise & Grain][n-match] | refused ADR-0150 | samples another element's pixels (§4) |
| Median | Replaces each pixel with the median of its neighbours | [Noise & Grain][n-median] | gap | grep empty |
| Median (Legacy) | Older median filter, with an option to work on alpha | [Noise & Grain][n-medianl] | gap | as Median |
| Noise | Random pixel values; colour or same-value noise | [Noise & Grain][n-noise] | covered | `grain`, `size: 1`, `mono: false`; re-rolls every frame (ADR-0156 §3) |
| Noise Alpha | Noise on the alpha channel | [Noise & Grain][n-alpha] | gap | `grain` never changes alpha (compositing page) |
| Noise HLS | Noise on hue, lightness and saturation | [Noise & Grain][n-hls] | partial | `grain` offsets RGB only |
| Noise HLS Auto | Noise HLS that animates by itself | [Noise & Grain][n-hls] | partial | as Noise HLS; `grain` re-rolls per frame |
| Remove Grain | Removes grain or noise from footage | [Noise & Grain][n-remove] | gap | no denoise member (audio has one, ADR-0181); grep empty |
| Turbulent Noise | Faster Perlin-noise texture like Fractal Noise | [Noise & Grain][n-turb] | reachable | prerender; time input as Fractal Noise |

[n-add]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/noise-grain-effects.html#add_grain_effect
[n-curl]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/noise-grain-effects.html#curl-noise
[n-dust]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/noise-grain-effects.html#dust-scratches
[n-fractal]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/noise-grain-effects.html#fractal_noise_effect
[n-match]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/noise-grain-effects.html#match_grain_effect
[n-median]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/noise-grain-effects.html#median
[n-medianl]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/noise-grain-effects.html#median-legacy
[n-noise]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/noise-grain-effects.html#noise_effect
[n-alpha]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/noise-grain-effects.html#noise_alpha_effect
[n-hls]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/noise-grain-effects.html#noise_hls_effect_and_noise_hls_auto_effect
[n-remove]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/noise-grain-effects.html#remove_grain_effect
[n-turb]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/noise-grain-effects.html#turbulent_noise_effect

## Perspective (10)

| Effect | What it does | Adobe | Status | Evidence |
|---|---|---|---|---|
| 3D Camera Tracker | Extracts camera movement from 2D footage | [Perspective][p-cam] | refused ADR-0167 | "a camera and depth sorting are After Effects-only"; the tracking data itself is not decided (map #896) |
| 3D Glasses | Combines left and right views into one 3D image | [Perspective][p-glasses] | gap | grep empty; anaglyph partly reachable with `tint` + `screen` copies |
| Bevel Alpha | Chiselled, lit edge on the alpha boundary | [Perspective][p-bevela] | gap | no lighting member; grep empty |
| Bevel Edges | Chiselled, lit rectangular edges | [Perspective][p-bevele] | gap | gradient `rect` frames approximate it; no member |
| CC Cylinder | Wraps the layer around a 3D cylinder | [Cycore][cc] | reachable | prerender (distortion) |
| CC Environment | Reflective spherical environment map | [Cycore][cc] | refused ADR-0150 | reads another layer as the map |
| CC Sphere | Turns the layer into a 3D sphere | [Cycore][cc] | reachable | prerender |
| CC Spotlight | Spotlight shining on the layer | [Cycore][cc] | reachable | radial-gradient `rect`, `blend: multiply` (vignette line in capability map) |
| Drop Shadow | Soft shadow behind the layer, shaped by its alpha | [Perspective][p-drop] | covered | `shadow` (ADR-0040) |
| Radial Shadow | Shadow cast from a point light, not an infinite one | [Perspective][p-radial] | partial | `shadow` with keyed `dx`/`dy`; no projection from a light point |

[p-cam]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/perspective-effects.html#camera-tracker
[p-glasses]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/perspective-effects.html#3d_glasses_effect
[p-bevela]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/perspective-effects.html#bevel_alpha_effect
[p-bevele]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/perspective-effects.html#bevel_edges_effect
[p-drop]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/perspective-effects.html#drop_shadow_effect
[p-radial]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/perspective-effects.html#radial_shadow_effect

## Simulation (18)

Each of these is particles or physics. The capability map routes "particles" to
`montagent-prerender`. The craft skill ("particle bursts belong in a trailer") is the only other
mention.

| Effect | What it does | Adobe | Status | Evidence |
|---|---|---|---|---|
| Card Dance | Divides the layer into cards driven by a second layer | [Simulation][s-card] | reachable | prerender; a driver layer is refused by ADR-0150 §4 |
| Caustics | Light caustics at the bottom of water | [Simulation][s-caustics] | reachable | prerender |
| CC Ball Action | Turns the layer into a grid of 3D balls (no Adobe description) | [Simulation][sim] | reachable | prerender |
| CC Bubbles | Bubbles rising through water | [Cycore][cc] | reachable | prerender |
| CC Drizzle | Light rain, a gentle drizzle | [Cycore][cc] | reachable | prerender |
| CC Hair | Hair or fibres that move with wind or motion | [Cycore][cc] | reachable | prerender |
| CC Mr. Mercury | Flowing, dripping liquid like mercury | [Cycore][cc] | reachable | prerender |
| CC Particle Systems II | Particles for fire, smoke, explosions or dust | [Cycore][cc] | reachable | prerender |
| CC Particle World | Particles moving freely in 3D | [Cycore][cc] | reachable | prerender |
| CC Pixel Polly | Breaks the layer into pieces, disintegrating | [Cycore][cc] | reachable | prerender |
| CC Rainfall | Falling rain | [Cycore][cc] | reachable | prerender |
| CC Scatterize | Scatters the layer into fragments | [Cycore][cc] | reachable | prerender |
| CC Snowfall | Falling snow | [Cycore][cc] | reachable | prerender |
| CC Star Burst | Particles shot outward from a centre point | [Cycore][cc] | reachable | prerender |
| Foam | Bubbles that flow, cling and pop | [Simulation][s-foam] | reachable | prerender |
| Particle Playground | Many similar objects animated independently: dots, layers or text | [Simulation][s-pp] | reachable | prerender; layer-sourced particles would be refused in-format (ADR-0150 §4) |
| Shatter | Explodes the layer into pieces | [Simulation][s-shatter] | reachable | prerender |
| Wave World | Greyscale displacement map from simulated liquid waves | [Simulation][s-wave] | reachable | prerender |

[s-card]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/simulation-effects.html#card_dance_effect
[s-caustics]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/simulation-effects.html#caustics_effect
[s-foam]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/simulation-effects.html#foam_effect
[s-pp]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/simulation-effects.html#particle_playground_effect
[s-shatter]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/simulation-effects.html#shatter_effect
[s-wave]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/simulation-effects.html#wave_world_effect

## Stylize (25)

| Effect | What it does | Adobe | Status | Evidence |
|---|---|---|---|---|
| Brush Strokes | Rough, painted look | [Stylize][y-brush] | reachable | prerender |
| Cartoon | Simplified shading and colour with strokes on the edges | [Stylize][y-cartoon] | partial | `posterize` (ADR-0156) gives fills; no edges (see Find Edges) |
| CC Block Load | Image loading in large blocky segments | [Cycore][cc] | reachable | prerender (mosaic family) |
| CC Burn Film | Burn marks and scratches of damaged film | [Cycore][cc] | reachable | prerender |
| CC Glass | Light bent through textured glass | [Cycore][cc] | reachable | prerender; a separate map layer refused (ADR-0150 §4) |
| CC HexTile | Hexagonal tiles, like a honeycomb | [Cycore][cc] | reachable | prerender |
| CC Kaleida | Kaleidoscope of reflected sections | [Cycore][cc] | reachable | prerender; mirror deferred ADR-0156 |
| CC Mr. Smoothie | Softens sharp edges into a smooth, stylised look | [Cycore][cc] | reachable | prerender |
| CC Plastic | Shiny, glossy plastic look | [Cycore][cc] | gap | lighting/emboss family; deferred-family ADR-0156, no documented route |
| CC RepeTile | Repeats the layer in a tiled grid | [Cycore][cc] | reachable | copies as elements (ADR-0148) |
| CC Threshold | High-contrast black and white | [Cycore][cc] | partial | `saturation: 0` + `brightness` + `posterize` `levels: 2`; threshold fixed at mid-grey, shifted by brightness |
| CC Threshold RGB | Threshold per red, green and blue channel | [Cycore][cc] | partial | `posterize` `levels: 2` per channel (fixed 0.5 cut) |
| CC Vignette | Darkens or lightens the edges | [Cycore][cc] | reachable | capability map: "a vignette … a `rect` carrying an effect and a blend" (ADR-0040 composable) |
| Color Emboss | Emboss keeping the original colours | [Stylize][y-cemboss] | gap | emboss deferred by ADR-0156 |
| Emboss | Sharpens edges and suppresses colour, lit from an angle | [Stylize][y-emboss] | gap | deferred by ADR-0156 |
| Find Edges | Finds and emphasises edges | [Stylize][y-edges] | gap | convolution member absent; grep empty (#703: `matrix_convolution`) |
| Glow | Brightens bright areas and their surroundings into a glow | [Stylize][y-glow] | covered | `glow` (ADR-0156); colour from alpha is a zero-offset `shadow` |
| Mosaic | Solid-colour rectangles, pixelating the image | [Stylize][y-mosaic] | reachable | prerender (capability map names mosaic); member deferred ADR-0156 |
| Motion Tile | Replicates the source across the output, with mirrored edges and phase | [Stylize][y-mtile] | reachable | copies as elements, `scale: [-1, 1]` for mirrored (ADR-0148) |
| Posterize | Fewer tonal levels per channel | [Stylize][y-posterize] | covered | `posterize` `levels` 2–256 (ADR-0156) |
| Roughen Edges | Roughens the alpha edge, optionally with corrosion colour | [Stylize][y-roughen] | gap | needs seeded noise on alpha; grep empty |
| Scatter | Redistributes pixels randomly nearby | [Stylize][y-scatter] | gap | seeded per-pixel offset; no member; grep empty |
| Strobe Light | Periodic or random flash or transparency | [Stylize][y-strobe] | reachable | `trailer.md` "Flash": `rect` with keyed `opacity`, `step` ease |
| Texturize | Gives the layer another layer's texture | [Stylize][y-texturize] | refused ADR-0150 | reads another element's pixels (§4) |
| Threshold | Pixels to black or white at a level | [Stylize][y-threshold] | partial | as CC Threshold |

[y-brush]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/stylize-effects.html#brush_strokes_effect
[y-cartoon]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/stylize-effects.html#cartoon_effect
[y-cemboss]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/stylize-effects.html#color_emboss_effect
[y-emboss]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/stylize-effects.html#emboss_effect
[y-edges]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/stylize-effects.html#find_edges_effect
[y-glow]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/stylize-effects.html#glow_effect
[y-mosaic]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/stylize-effects.html#mosaic_effect
[y-mtile]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/stylize-effects.html#motion_tile_effect
[y-posterize]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/stylize-effects.html#posterize_effect
[y-roughen]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/stylize-effects.html#roughen_edges_effect
[y-scatter]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/stylize-effects.html#scatter_effect
[y-strobe]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/stylize-effects.html#strobe_light_effect
[y-texturize]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/stylize-effects.html#texturize_effect
[y-threshold]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/stylize-effects.html#threshold_effect

## Time (8)

| Effect | What it does | Adobe | Status | Evidence |
|---|---|---|---|---|
| CC Force Motion Blur | Motion blur on the layer's own movement | [Cycore][cc] | covered | `motion_blur` `shutter`/`samples` (ADR-0155) |
| CC Wide Time | Blends multiple frames into a ghostly trail | [Cycore][cc] | reachable | copies offset in time with lower `opacity` (ADR-0148); `motion_blur` is not this (CONTEXT avoids "echo") |
| Echo | Combines frames from different times in a layer | [Time][time] | reachable | as CC Wide Time; a video copy uses its own `source_time` offset (ADR-0157) |
| Pixel Motion Blur | Motion blur synthesised from footage motion vectors | [Time][t-pmb] | gap | "a video's footage is not blurred, only its keyed values" (ADR-0155) |
| Posterize Time | Locks a layer to a specific frame rate | [Time][t-post] | reachable | stepped keys: `step` ease, a stepped `source_time` curve (ADR-0157) |
| Time Difference | Colour difference between two layers | [Time][t-diff] | refused ADR-0150 | reads a second element's pixels (§4) |
| Time Displacement | Shifts pixels across time by a map layer | [Time][t-disp] | refused ADR-0150 | map layer = another element (§4) |
| Timewarp | Speed change with interpolation, motion blur and cropping | [Time][t-warp] | partial | `source_time` speed ramps (ADR-0157); last frame at or before the instant (ADR-0096), no blending or flow |

[t-pmb]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/time-effects.html#pixel-motion-blur
[t-post]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/time-effects.html#posterize-time
[t-diff]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/time-effects.html#time-difference
[t-disp]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/time-effects.html#time-displacement
[t-warp]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/time-effects.html#timewarp

## Transition (17)

Montagent's `transition` has five kinds: `crossfade`, `wipe`, `slide`, `push` and `audio_crossfade`.
`direction` is one of four, an `ease` is optional, and edges are hard whole pixels (ADR-0150, ADR-0176).
It has no border, no soft edge, no angle and no Start/End % (ADR-0150 §1, "Nothing else
enters"). A reveal on one element is a keyed `mask` (ADR-0146). The transition rows below name
the AE transition, so "reachable" usually means keyed masks or copies on the incoming element,
not a `transition` kind.

| Effect | What it does | Adobe | Status | Evidence |
|---|---|---|---|---|
| Block Dissolve | Layer disappears in random blocks | [Transition][r-block] | gap | needs a seed and blocks; no kind; grep empty |
| Card Wipe | Grid of cards flip to a back layer | [Transition][r-card] | gap | per-card copies with `swivel` need a pivot per card (#591 free pivot); no kind |
| CC Glass Wipe | Glassy melt using a map layer (no Adobe description) | [Transition][trans] | refused ADR-0150 | map layer (§4) |
| CC Grid Wipe | Grid pattern that uncovers the next picture | [Cycore][cc] | reachable | masks on copies of the incoming element (ADR-0148); masks in one list intersect, so no single-element union |
| CC Image Wipe | Wipe that uses an image or texture as its mask | [Cycore][cc] | refused ADR-0150 | "matte from another element" (§4) |
| CC Jaws | Teeth-like "chomp" that reveals the next scene | [Cycore][cc] | reachable | keyed `mask` `shape: "path"` with zig-zag `points` (ADR-0163) |
| CC Light Wipe | Light source sweeping across, wiping the scene away | [Cycore][cc] | partial | keyed `mask` circle with `feather` (ADR-0152); no edge light |
| CC Line Sweep | A single moving line reveals the next clip | [Cycore][cc] | partial | `wipe` is four directions only; any angle via a keyed path `mask` (ADR-0163) |
| CC Radial ScaleWipe | Wipe with a zoom from the centre or edge | [Cycore][cc] | reachable | prerender (distortion) |
| CC Scale Wipe | Swipe and zoom into the next scene | [Cycore][cc] | reachable | prerender (distortion) |
| CC Twister | Swirling, twisting distortion into the next scene | [Cycore][cc] | reachable | prerender (distortion) |
| CC WarpoMatic | Stretchy warp that bends the scene into the next | [Cycore][cc] | reachable | prerender; a driver from another element refused (ADR-0150 §4) |
| Gradient Wipe | Transparency ordered by a gradient layer's luminance | [Transition][r-grad] | refused ADR-0150 | the gradient layer is another element (§4) |
| Iris Wipe | Radial iris of 6–32 points reveals the layer below | [Transition][r-iris] | reachable | keyed `mask` (`ellipse`, `rect`, `path`) with `feather` (ADR-0146, ADR-0152, ADR-0163) |
| Linear Wipe | Simple linear wipe at an angle, feathered | [Transition][r-linear] | partial | `wipe` + `direction` (ADR-0150): 4 directions, hard edge; angle and feather via keyed path `mask` |
| Radial Wipe | Wipe circling around a point | [Transition][r-radial] | reachable | keyed path `mask` drawing the sector (ADR-0163); no kind |
| Venetian Blinds | Strips of set direction and width reveal the layer below | [Transition][r-blinds] | reachable | one copy of the incoming element per band, each with a keyed `mask` (ADR-0148) |

[r-block]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/transition-effects.html#block_dissolve_effect
[r-card]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/transition-effects.html#card_wipe_effect
[r-grad]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/transition-effects.html#gradient_wipe_effect
[r-iris]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/transition-effects.html#iris_wipe_effect
[r-linear]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/transition-effects.html#linear_wipe_effect
[r-radial]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/transition-effects.html#radial_wipe_effect
[r-blinds]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/transition-effects.html#venetian_blinds_effect

## Matte (5)

| Effect | What it does | Adobe | Status | Evidence |
|---|---|---|---|---|
| Matte Choker | Chokes and spreads a matte repeatedly to fill holes | [Matte][m-mchoker] | gap | no alpha choke; `chroma` `tolerance`/`softness` act before the matte (ADR-0088) |
| Mocha shape | Turns paths tracked in Mocha AE into mattes | [Adobe page][m-mocha] | partial | a keyed path `mask` (ADR-0146, ADR-0163) holds a shape; no route brings in Mocha's tracked paths; grep "mocha" empty |
| Refine Hard Matte | Cleans a hard keyed edge (no Adobe description) | [Refine Matte][m-refine] | partial | `chroma` `softness` + `spill` (ADR-0088); no edge refinement |
| Refine Soft Matte | Refines soft edges of a keyed matte, removes spill and noise | [Matte][m-soft] | partial | as above |
| Simple Choker | Shrinks or expands matte edges in small steps | [Matte][m-schoker] | gap | `mask` `feather` softens only its own shape (ADR-0152); grep "choke" empty |

[m-mchoker]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/matte-effects.html#matte_choker_effect
[m-mocha]: https://helpx.adobe.com/after-effects/desktop/work-with-transparency-and-compositing/compositing-in-after-effects/compositing-transparency-overview-resources.html
[m-refine]: https://helpx.adobe.com/after-effects/desktop/roto-brush-and-refine-matte/roto-brush/roto-brush-refine-matte.html#roto-brush-effect-refine-matte-effect
[m-soft]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/matte-effects.html#refine-soft-matte
[m-schoker]: https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/matte-effects.html#simple-choker-effect

The ticket names "Refine Matte". The current Matte category holds Refine Hard Matte and Refine
Soft Matte instead. The Matte page sends Refine Matte readers to the Roto Brush and Refine Matte
reference.

## Categories named, not scored row by row

- **Audio** ([Audio][audio]): the effect list has Backwards, Bass & Treble, Delay, Flange &
  Chorus, High-Low Pass, Modulator, Parametric EQ, Reverb, Stereo Mixer and Tone. The Audio page
  (11 September 2026) adds Compressor, Distortion and Gate, plus De-esser as a beta feature. The
  audio map [#795](https://github.com/MBehtemam/Montagent/issues/795) owns them all:
  - **Already on the capability map:** Bass & Treble (`shelf`), High-Low Pass (`highpass`, `lowpass`), Parametric EQ (`bell`). These come from ADR-0179, still marked proposed while its test table is committed.
  - **Open on #795:** Delay (ADR-0185 `echo`, draft), Stereo Mixer (ADR-0184 `pan`/`channels`, draft), Reverb (out of ADR-0185, prototype in `docs/research/audio-effects/echo-reverb/`), Flange & Chorus, Modulator, Tone, and the page's new Compressor, Distortion, Gate and De-esser. ADR-0169 already uses a compressor as its example of why an audio effect's order matters.
  - **Backwards:** `source_time` on `audio` is a schema error (ADR-0157). Reversed sound is #795's to decide.
- **Obsolete** ([Obsolete][obs]): Adobe keeps these effects "for compatibility with projects
  created with previous versions" and says to use alternatives. The effect list has Basic 3D,
  Basic Text, Color Key, Gaussian Blur (Legacy), Lightning, Luma Key, Path Text, Reduce Interlace
  Flicker and Spill Suppressor. The Obsolete page also documents Fast Blur, which the list no
  longer has. Not scored. Their replacements are scored elsewhere: projection (ADR-0167),
  `text`, `chroma`, `blur`, Advanced Lightning, text on a path (ADR-0161). Reduce Interlace
  Flicker has no counterpart; ADR-0141's equivalence test did not cover interlaced sources.
- **Immersive Video** ([VR effects][vr]): VR Blur, VR Chromatic Aberrations, VR Color Gradients,
  VR Converter, VR De-Noise, VR Digital Glitch, VR Fractal Noise, VR Glow, VR Plane to Sphere,
  VR Rotate Sphere, VR Sharpen, VR Sphere to Plane. Not scored:
  Montagent has no equirectangular or stereo frame model (grep "equirect" and "VR" find nothing
  in the ADRs). Whether 360° video is in scope at all is its own question for #896.

## Counts

Over the 159 rows scored in the nine categories:

| Status | Count |
|---|---|
| covered | 12 |
| partial | 19 |
| reachable | 92 (prerender first: 69; copies, keyed masks, skill recipes or a script: 23) |
| owned by an open map | 0 (the Audio effects all go to #795 and are not counted) |
| refused | 13 (ADR-0150: 8, ADR-0167: 4, ADR-0156: 1) |
| gap | 23 |

By category (covered / partial / reachable / refused / gap): Distort 0/1/29/4/3, Generate
7/3/15/0/1, Noise & Grain 1/3/3/1/5, Perspective 1/1/3/2/3, Simulation 0/0/18/0/0, Stylize
2/4/12/1/6, Time 1/1/3/2/1, Transition 0/3/9/3/2, Matte 0/3/0/0/2.

Verification added three rows: Curl Noise (reachable), Median (Legacy) (gap) and Mocha shape
(partial). No row was removed, and no existing row changed status.

"Reachable" is large because `montagent-prerender` documents distortions, mosaic, lens flare,
particles and procedural patterns. Those rows are reachable as fixed footage, not as parameters
an agent edits in the project.

## Gaps worth a decision ticket

Grouped. Each group says why it can't simply be pre-rendered or composed, or why a named
member would beat the prerender route.

1. **Distortions waiting on an edge rule.** Wave Warp, Ripple, Twirl, Bulge/Spherize, Mirror,
   Magnify, Offset, Polar Coordinates, CC Lens, CC Kaleida and the CC Split/Tiler family.
   ADR-0156 deferred the first six until the edge rule (clamp, transparent or wrap) is decided.
   Today all of them are pre-rendered footage, which bakes the source in. One ticket deciding
   the edge rule unblocks a batch of members under ADR-0156 §2. Wave Warp and Ripple fit §3 as
   is, through a keyed phase.
2. **Deferred filter members.** Emboss/Color Emboss, Find Edges, Sharpen (Blur & Sharpen,
   another census), Mosaic, CC Plastic and Cartoon's ink edges. These are convolution or block
   filters with Skia routes in #703, so they are the cheapest ADR-0156 growth.
3. **Alpha operations.** Simple Choker, Matte Choker, Roughen Edges, Noise Alpha and Scatter.
   No member reads or changes alpha except `mask` and `chroma`. A keyed-footage pipeline wants
   a choke after `chroma`. A choke stays inside the element, so ADR-0150 §4 does not bar it.
4. **Generators and noise with a time axis.** Fractal Noise, Turbulent Noise, Curl Noise, Cell
   Pattern and Turbulent Displace. ADR-0156 §3 allows a seed but no time input, and Evolution is
   time (#703). Curl Noise's drift (Speed and Direction) is the same question. The questions: a
   keyed `evolution` parameter (like a keyed phase), or prerender for good? 4-Color Gradient is
   a separate question: a third `gradient` kind (bilinear corners) under ADR-0149.
5. **Transition kinds.** Iris, Radial (clock), Venetian Blinds, Block Dissolve, Card Wipe and an
   angled or feathered Linear Wipe. Today they are hand-keyed masks on copies. That is the
   "scattered state `validate` cannot tell is one transition" that ADR-0150's "Why" argued
   against for slide and push. Premiere ships Iris and Clock wipes (precedent per ADR-0145).
   Block Dissolve would need a seed.
6. **Time effects on footage.** Pixel Motion Blur and Timewarp's frame blending or optical flow.
   ADR-0155 blurs keyed values only, and ADR-0157 shows the last frame at or before. Footage
   interpolation is the charting list's "frame blending and pixel motion". Echo and CC Wide Time
   as a native field, not copies, belong to the same ticket.
7. **Particles and simulation.** All 18 Simulation effects. They are reachable only as baked
   footage. The ticket the map already lists ("particles and simulation") should decide whether
   anything beyond the prerender route enters. A seeded emitter would be a generator element,
   which ADR-0156 refused but left open to "its own ADR".
8. **Footage analysis.** Warp Stabilizer, Rolling Shutter Repair, 3D Camera Tracker,
   Detail-preserving Upscale, Remove Grain, Dust & Scratches, Median and Median (Legacy). None
   has a route. The tracking and stabilise results could be literal keyframes written by a
   skill (the "reachable by a skill" line on #896). Mocha shape's tracked paths are the same
   kind of result: a skill could write them as keyed path `mask` points. Denoise and median are
   filter members.
9. **Lighting and 3D surface looks.** Bevel Alpha, Bevel Edges, Radial Shadow's light point and
   CC Sphere/Cylinder lighting. These are low priority. Skia's lit filters
   (`distant_lit_diffuse`, #703) are the route if any of them is wanted.

Already decided, and not to reopen: everything that reads a second element (Displacement Map,
Gradient Wipe, CC Image Wipe, CC Glass Wipe, Texturize, Match Grain, Time Difference, Time
Displacement, CC Environment) is refused by ADR-0150 §4. So are corner pin and skew (CC Power
Pin, Corner Pin, CC Slant; ADR-0167). An open shader file stays refused (ADR-0156 §8,
ADR-0017).
