# After Effects' generate, distort, stylize, simulation, transition, time, noise, perspective and matte effects, scored against Montagent

Research for [#905](https://github.com/MBehtemam/Montagent/issues/905), part of the map
[#896](https://github.com/MBehtemam/Montagent/issues/896). **This note decides nothing.** It scores
every effect in nine After Effects categories against Montagent's `main` (commit `51aa4b5`). It
names the Audio, Obsolete and Immersive Video categories, and it ends with the gaps worth a
decision ticket.

## Sources, version and limits

- **Adobe's effect list and its category pages** ([effect list][list]; the category pages are
  linked in each table). The search index dates the category pages 26–27 June 2025 and the
  Noise and Grain page 17 June 2026. That puts them at After Effects 25.x–26.x, the current
  user guide.
- **Adobe's pages could not be fetched from this session.** `helpx.adobe.com` and
  `learn.adobe.com` got a proxy 403 or a DNS failure, and #703 hit the same 403. The category
  URLs, update dates and per-effect facts come from search-index excerpts of those pages. The
  membership of each category comes from those excerpts plus the AE 25 Effects menu as known
  before this session. Rows not confirmed by an excerpt are marked **(menu)**. A reader with a
  browser should check each **(menu)** row against its category page before a decision ticket
  quotes it.
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
| Bezier Warp | Reshapes the layer by a 12-handle closed curve on its edge | [Distort][dist] | reachable | prerender (distortions); free warp undecided |
| Bulge | Lens-like swelling around a point | [Distort][dist] | reachable | prerender; spherize deferred in ADR-0156 (edge rule) |
| CC Bend It | Bends the layer between two points | [Distort][dist] (menu) | reachable | prerender |
| CC Bender | Bends the layer along an axis | [Distort][dist] (menu) | reachable | prerender |
| CC Blobbylize | Blobby, liquid relief from a map | [Distort][dist] (menu) | reachable | prerender; reading a map layer would be refused by ADR-0150 §4 |
| CC Flo Motion | Pinch/stretch around two knots | [Distort][dist] (menu) | reachable | prerender |
| CC Griddler | Cuts the layer into tiles offset in a grid | [Distort][dist] (menu) | reachable | prerender |
| CC Lens | Fisheye lens | [Distort][dist] (menu) | reachable | prerender; spherize deferred ADR-0156 |
| CC Page Turn | Page curl showing a back side | [Distort][dist] (menu) | reachable | prerender; a flat flip is `swivel` (ADR-0167) |
| CC Power Pin | Four-corner pin with perspective | [Distort][dist] (menu) | refused ADR-0167 | "Skew, corner pin … still refused" |
| CC Ripple Pulse | Ripple driven by a pulse | [Distort][dist] (menu) | reachable | prerender; ripple deferred ADR-0156 |
| CC Slant | Slants (skews) the layer | [Distort][dist] (menu) | refused ADR-0167 | skew refused (ADR-0012, ADR-0167) |
| CC Smear | Drags pixels from one point to another | [Distort][dist] (menu) | reachable | prerender |
| CC Split | Splits the layer open along a line | [Distort][dist] (menu) | reachable | two copies with `mask` and keyed `x`/`y` (ADR-0148 copies) |
| CC Split 2 | Split with separate strengths per side | [Distort][dist] (menu) | reachable | as CC Split |
| CC Tiler | Tiles a scaled copy of the layer across it | [Distort][dist] (menu) | reachable | copies as elements (ADR-0148) |
| Corner Pin | Moves four corners for a perspective distort | [Distort][dist] | refused ADR-0167 | corner pin refused; ADR-0156 sent it to the 2.5D decision |
| Detail-preserving Upscale | Upscale that keeps edges sharp | [Distort][dist] (menu) | gap | resampling fixed by ADR-0132; no upscaler member |
| Displacement Map | Shifts pixels by another layer's channels | [Distort][dist] | refused ADR-0156 | "reads another element's pixels" (ADR-0150 §4) |
| Liquify | Brush-painted mesh warp | [Distort][dist] (menu) | reachable | prerender |
| Magnify | Circular or square loupe | [Distort][dist] (menu) | reachable | prerender; magnify deferred ADR-0156 |
| Mesh Warp | Warps by a grid of Bezier patches | [Distort][dist] (menu) | reachable | prerender |
| Mirror | Reflects about a line | [Distort][dist] (menu) | reachable | copy with `scale: [-1, 1]` + `mask`; mirror deferred ADR-0156 |
| Offset | Pans the layer within itself, wrapping | [Distort][dist] (menu) | reachable | prerender; wrap is the undecided edge rule (ADR-0156) |
| Optics Compensation | Adds or removes lens barrel distortion | [Distort][dist] (menu) | reachable | prerender |
| Polar Coordinates | Rect ↔ polar mapping | [Distort][dist] (menu) | reachable | prerender |
| Reshape | Morphs one mask shape to another, dragging pixels | [Distort][dist] (menu) | reachable | prerender; a shape morph (no pixels) is `path` `points` (ADR-0162) |
| Ripple | Concentric ripples from a centre | [Distort][dist] (menu) | reachable | prerender; deferred ADR-0156 (keyed phase, §3) |
| Rolling Shutter Repair | Fixes CMOS skew in footage | [Distort][dist] (menu) | gap | no footage-analysis route; grep empty |
| Smear | Moves a masked region, stretching the rest | [Distort][dist] (menu) | reachable | prerender |
| Spherize | Wraps the layer on a sphere | [Distort][dist] (menu) | reachable | prerender; deferred ADR-0156 |
| Transform | Anchor, position, scale, skew, rotation, opacity, shutter | [Distort][dist] (menu) | partial | flat transform (ADR-0012), `motion_blur` (ADR-0155); skew refused ADR-0167 |
| Turbulent Displace | Fractal-noise displacement | [Distort][dist] (menu) | reachable | prerender; seed + time (ADR-0156 §3) |
| Twirl | Rotates more at the centre than the edge | [Distort][dist] (menu) | reachable | prerender; deferred ADR-0156 |
| Warp | Arc, bulge, flag, wave and other preset warps | [Distort][dist] (menu) | reachable | prerender |
| Warp Stabilizer | Analyses and stabilises shaky footage | [Distort][dist] (menu) | gap | no tracking or analysis route; grep empty |
| Wave Warp | Travelling wave across the layer | [Distort][dist] (menu) | reachable | prerender; deferred ADR-0156 (keyed phase) |

## Generate (26)

| Effect | What it does | Adobe | Status | Evidence |
|---|---|---|---|---|
| 4-Color Gradient | Four-point colour blend | [Generate][gen] | partial | `gradient` is linear or radial only (ADR-0149); stacked radial rects approximate it |
| Advanced Lightning | Branching lightning bolt | [Generate][gen] | reachable | prerender (procedural) |
| Audio Spectrum | Frequency bars or lines from an audio layer | [Generate][gen] | reachable | prerender, or keyed `rect` heights from a script (as ducking, ADR-0177); no shipped script |
| Audio Waveform | Waveform drawn from an audio layer | [Generate][gen] | reachable | as Audio Spectrum; keyed `path` `points` |
| Beam | Laser beam between two points | [Generate][gen] | partial | `path` stroke + `trim_*` + `glow`; no tapered core |
| CC Glue Gun | Glue-like stroke along a path | [Generate][gen] | reachable | prerender |
| CC Light Burst 2.5 | Radial light streaks from bright pixels | [Generate][gen] | reachable | prerender |
| CC Light Rays | God rays from a point through the layer | [Generate][gen] | reachable | prerender |
| CC Light Sweep | Light band sweeping across the layer | [Generate][gen] | reachable | `trailer.md` "Light sweeps": gradient `rect`, `blend: screen` |
| CC Threads | Woven-thread texture | [Generate][gen] | reachable | prerender (procedural) |
| Cell Pattern | Procedural cellular (Voronoi) texture | [Generate][gen] | reachable | prerender; would need seed (ADR-0156 §3) |
| Checkerboard | Checker pattern | [Generate][gen] | reachable | `rect` copies (ADR-0148), or prerender |
| Circle | Filled or ring circle | [Generate][gen] | covered | `ellipse`, `stroke`, `mask` `circle` |
| Ellipse | Stroked ellipse with glow | [Generate][gen] | covered | `ellipse` + `stroke` + `glow`/`shadow` |
| Eyedropper Fill | Fills with a colour sampled from the layer | [Generate][gen] | reachable | agent reads the colour with `frame` and writes a literal `tint` (literal values, ADR-0145) |
| Fill | Fills alpha with a colour | [Generate][gen] | covered | `tint` with `amount: 1` (ADR-0049) |
| Fractal | Mandelbrot/Julia set | [Generate][gen] | reachable | prerender |
| Gradient Ramp | Linear or radial two-colour ramp | [Generate][gen] | covered | `gradient` linear/radial on `fill` (ADR-0149) |
| Grid | Grid lines | [Generate][gen] | reachable | `rect`/`path` copies (ADR-0148), `stroke_dash` |
| Lens Flare | Simulated lens flare | [Generate][gen] | reachable | `trailer.md` "flares" (gradients, PNG plates, `screen`); prerender |
| Paint Bucket | Flood fill by tolerance | [Generate][gen] | gap | reads pixels for a region; no route documented |
| Radio Waves | Rings emanating from a point | [Generate][gen] | reachable | `ellipse` copies with keyed `scale`/`opacity` (ADR-0148) |
| Scribble | Scribbled fill of a mask | [Generate][gen] | partial | `path` + `trim_end` draws on (ADR-0160); zig-zag points written by hand |
| Stroke | Strokes a mask path, with start/end | [Generate][gen] | covered | `path` `stroke`, `trim_start`/`trim_end` (ADR-0160) |
| Vegas | Marching dashes along a path or edges | [Generate][gen] | covered | `stroke_dash` + keyed `stroke_dash_offset` (ADR-0158) |
| Write-on | Paints a stroke following a keyed brush | [Generate][gen] | covered | `path` + keyed `trim_end` (ADR-0160) |

## Noise & Grain (11)

| Effect | What it does | Adobe | Status | Evidence |
|---|---|---|---|---|
| Add Grain | Synthesises film grain with presets | [Noise & Grain][noise] | partial | `grain` (seed, amount, size, mono; ADR-0156); no stock presets, softness or per-channel intensity |
| Dust & Scratches | Median-based cleanup within a radius | [Noise & Grain][noise] (menu) | gap | no median member; grep empty |
| Fractal Noise | Fractal turbulence, Evolution animates | [Noise & Grain][noise] (menu) | reachable | prerender; #703: Evolution needs a time input ADR-0156 §3 refuses |
| Match Grain | Copies grain from a source layer | [Noise & Grain][noise] | refused ADR-0150 | samples another element's pixels (§4) |
| Median | Median filter | [Noise & Grain][noise] (menu) | gap | grep empty |
| Noise | Random per-pixel noise each frame | [Noise & Grain][noise] (menu) | covered | `grain`, `size: 1`, `mono: false`; re-rolls every frame (ADR-0156 §3) |
| Noise Alpha | Noise on the alpha channel | [Noise & Grain][noise] (menu) | gap | `grain` never changes alpha (compositing page) |
| Noise HLS | Noise per hue, lightness, saturation | [Noise & Grain][noise] (menu) | partial | `grain` offsets RGB only |
| Noise HLS Auto | Noise HLS that animates by itself | [Noise & Grain][noise] (menu) | partial | as Noise HLS; `grain` re-rolls per frame |
| Remove Grain | Denoise footage | [Noise & Grain][noise] | gap | no denoise member (audio has one, ADR-0181); grep empty |
| Turbulent Noise | Faster fractal noise | [Noise & Grain][noise] (menu) | reachable | prerender; time input as Fractal Noise |

## Perspective (10)

| Effect | What it does | Adobe | Status | Evidence |
|---|---|---|---|---|
| 3D Camera Tracker | Solves a camera from footage | [Perspective][persp] (menu) | refused ADR-0167 | "a camera and depth sorting are After Effects-only"; the tracking data itself is not decided (map #896) |
| 3D Glasses | Stereo anaglyph from two views | [Perspective][persp] (menu) | gap | grep empty; anaglyph partly reachable with `tint` + `screen` copies |
| Bevel Alpha | Bevelled edge lit from the alpha | [Perspective][persp] (menu) | gap | no lighting member; grep empty |
| Bevel Edges | Bevelled rectangle edges | [Perspective][persp] (menu) | gap | gradient `rect` frames approximate it; no member |
| CC Cylinder | Wraps the layer on a cylinder | [Perspective][persp] (menu) | reachable | prerender (distortion) |
| CC Environment | Maps an environment onto the layer | [Perspective][persp] (menu) | refused ADR-0150 | reads another layer as the map |
| CC Sphere | Wraps the layer on a lit sphere | [Perspective][persp] (menu) | reachable | prerender |
| CC Spotlight | Spotlight pool and falloff | [Perspective][persp] (menu) | reachable | radial-gradient `rect`, `blend: multiply` (vignette line in capability map) |
| Drop Shadow | Offset soft shadow | [Perspective][persp] (menu) | covered | `shadow` (ADR-0040) |
| Radial Shadow | Shadow cast from a point light | [Perspective][persp] (menu) | partial | `shadow` with keyed `dx`/`dy`; no projection from a light point |

## Simulation (18)

Each of these is particles or physics. The capability map routes "particles" to
`montagent-prerender`. The craft skill ("particle bursts belong in a trailer") is the only other
mention.

| Effect | What it does | Adobe | Status | Evidence |
|---|---|---|---|---|
| Card Dance | Layer split into cards driven by a gradient layer | [Simulation][sim] (menu) | reachable | prerender; a gradient driver layer is refused by ADR-0150 §4 |
| Caustics | Water-surface light caustics | [Simulation][sim] (menu) | reachable | prerender |
| CC Ball Action | Layer turned into a grid of 3D balls | [Simulation][sim] (menu) | reachable | prerender |
| CC Bubbles | Bubbles from the layer's colours | [Simulation][sim] (menu) | reachable | prerender |
| CC Drizzle | Raindrop ripples | [Simulation][sim] (menu) | reachable | prerender |
| CC Hair | Hair or fur grown from the layer | [Simulation][sim] (menu) | reachable | prerender |
| CC Mr. Mercury | Liquid blobs | [Simulation][sim] (menu) | reachable | prerender |
| CC Particle Systems II | 2D particle emitter | [Simulation][sim] (menu) | reachable | prerender |
| CC Particle World | 3D particle emitter | [Simulation][sim] | reachable | prerender |
| CC Pixel Polly | Shatters the layer into polygons | [Simulation][sim] (menu) | reachable | prerender |
| CC Rainfall | Rain | [Simulation][sim] (menu) | reachable | prerender |
| CC Scatterize | Scatters the layer into pixels | [Simulation][sim] (menu) | reachable | prerender |
| CC Snowfall | Snow | [Simulation][sim] (menu) | reachable | prerender |
| CC Star Burst | Star field rushing at the viewer | [Simulation][sim] (menu) | reachable | prerender |
| Foam | Bubbles and foam | [Simulation][sim] (menu) | reachable | prerender |
| Particle Playground | Particles, optionally from other layers | [Simulation][sim] (menu) | reachable | prerender; layer-sourced particles would be refused in-format (ADR-0150 §4) |
| Shatter | Explodes the layer into 3D shards | [Simulation][sim] | reachable | prerender |
| Wave World | Water-surface height map | [Simulation][sim] (menu) | reachable | prerender |

## Stylize (25)

| Effect | What it does | Adobe | Status | Evidence |
|---|---|---|---|---|
| Brush Strokes | Painterly brush look | [Stylize][styl] (menu) | reachable | prerender |
| Cartoon | Flattened fills with ink edges | [Stylize][styl] (menu) | partial | `posterize` (ADR-0156) gives fills; no edges (see Find Edges) |
| CC Block Load | Progressive-JPEG-style blocky reveal | [Stylize][styl] (menu) | reachable | prerender (mosaic family) |
| CC Burn Film | Burning-film holes | [Stylize][styl] (menu) | reachable | prerender |
| CC Glass | Glass relief from a map | [Stylize][styl] (menu) | reachable | prerender; a separate map layer refused (ADR-0150 §4) |
| CC HexTile | Hexagonal tiling of the layer | [Stylize][styl] (menu) | reachable | prerender |
| CC Kaleida | Kaleidoscope | [Stylize][styl] (menu) | reachable | prerender; mirror deferred ADR-0156 |
| CC Mr. Smoothie | Smooth colour bands sampled from the layer | [Stylize][styl] (menu) | reachable | prerender |
| CC Plastic | Plastic relief, lit | [Stylize][styl] (menu) | gap | lighting/emboss family; deferred-family ADR-0156, no documented route |
| CC RepeTile | Repeats the layer outward | [Stylize][styl] (menu) | reachable | copies as elements (ADR-0148) |
| CC Threshold | One-level threshold | [Stylize][styl] | partial | `saturation: 0` + `brightness` + `posterize` `levels: 2`; threshold fixed at mid-grey, shifted by brightness |
| CC Threshold RGB | Threshold per channel | [Stylize][styl] (menu) | partial | `posterize` `levels: 2` per channel (fixed 0.5 cut) |
| CC Vignette | Darkened edges | [Stylize][styl] (menu) | reachable | capability map: "a vignette … a `rect` carrying an effect and a blend" (ADR-0040 composable) |
| Color Emboss | Emboss keeping colour | [Stylize][styl] (menu) | gap | emboss deferred by ADR-0156 |
| Emboss | Grey relief | [Stylize][styl] (menu) | gap | deferred by ADR-0156 |
| Find Edges | Edge detection | [Stylize][styl] (menu) | gap | convolution member absent; grep empty (#703: `matrix_convolution`) |
| Glow | Bloom from bright pixels, colourable | [Stylize][styl] (menu) | covered | `glow` (ADR-0156); colour from alpha is a zero-offset `shadow` |
| Mosaic | Blocks of averaged colour | [Stylize][styl] (menu) | reachable | prerender (capability map names mosaic); member deferred ADR-0156 |
| Motion Tile | Tiles the layer, mirrored edges, phase | [Stylize][styl] (menu) | reachable | copies as elements, `scale: [-1, 1]` for mirrored (ADR-0148) |
| Posterize | Fewer levels per channel | [Stylize][styl] (menu) | covered | `posterize` `levels` 2–256 (ADR-0156) |
| Roughen Edges | Noisy, eroded alpha edge | [Stylize][styl] (menu) | gap | needs seeded noise on alpha; grep empty |
| Scatter | Scatters pixels randomly | [Stylize][styl] (menu) | gap | seeded per-pixel offset; no member; grep empty |
| Strobe Light | Periodic flash or transparency | [Stylize][styl] (menu) | reachable | `trailer.md` "Flash": `rect` with keyed `opacity`, `step` ease |
| Texturize | Lights the layer with another layer's texture | [Stylize][styl] (menu) | refused ADR-0150 | reads another element's pixels (§4) |
| Threshold | Pixels to black or white at a level | [Stylize][styl] (menu) | partial | as CC Threshold |

## Time (8)

| Effect | What it does | Adobe | Status | Evidence |
|---|---|---|---|---|
| CC Force Motion Blur | Sub-frame samples of the layer's animation | [Time][time] | covered | `motion_blur` `shutter`/`samples` (ADR-0155) |
| CC Wide Time | Blends frames before and after | [Time][time] | reachable | copies offset in time with lower `opacity` (ADR-0148); `motion_blur` is not this (CONTEXT avoids "echo") |
| Echo | Composites frames from other times | [Time][time] | reachable | as CC Wide Time; a video copy uses its own `source_time` offset (ADR-0157) |
| Pixel Motion Blur | Optical-flow blur of footage content | [Time][time] | gap | "a video's footage is not blurred, only its keyed values" (ADR-0155) |
| Posterize Time | Locks a layer to a lower frame rate | [Time][time] | reachable | stepped keys: `step` ease, a stepped `source_time` curve (ADR-0157) |
| Time Difference | Difference between a layer and a target layer | [Time][time] (menu) | refused ADR-0150 | reads a second element's pixels (§4) |
| Time Displacement | Per-pixel time offset from a map layer | [Time][time] (menu) | refused ADR-0150 | map layer = another element (§4) |
| Timewarp | Speed change with frame blending or optical flow | [Time][time] | partial | `source_time` speed ramps (ADR-0157); last frame at or before the instant (ADR-0096), no blending or flow |

## Transition (17)

Montagent's `transition` has five kinds: `crossfade`, `wipe`, `slide`, `push` and `audio_crossfade`.
`direction` is one of four, an `ease` is optional, and edges are hard whole pixels (ADR-0150, ADR-0176).
It has no border, no soft edge, no angle and no Start/End % (ADR-0150 §1, "Nothing else
enters"). A reveal on one element is a keyed `mask` (ADR-0146). The transition rows below name
the AE transition, so "reachable" usually means keyed masks or copies on the incoming element,
not a `transition` kind.

| Effect | What it does | Adobe | Status | Evidence |
|---|---|---|---|---|
| Block Dissolve | Random blocks reveal the layer below | [Transition][trans] | gap | needs a seed and blocks; no kind; grep empty |
| Card Wipe | Grid of cards flip to the next picture | [Transition][trans] | gap | per-card copies with `swivel` need a pivot per card (#591 free pivot); no kind |
| CC Glass Wipe | Glassy melt using a map layer | [Transition][trans] | refused ADR-0150 | map layer (§4) |
| CC Grid Wipe | Wipe in a grid of shapes | [Transition][trans] | reachable | masks on copies of the incoming element (ADR-0148); masks in one list intersect, so no single-element union |
| CC Image Wipe | Wipe ordered by another layer's luminance | [Transition][trans] | refused ADR-0150 | "matte from another element" (§4) |
| CC Jaws | Zig-zag jaws close or open | [Transition][trans] | reachable | keyed `mask` `shape: "path"` with zig-zag `points` (ADR-0163) |
| CC Light Wipe | Circular wipe with a bright edge | [Transition][trans] | partial | keyed `mask` circle with `feather` (ADR-0152); no edge light |
| CC Line Sweep | Straight-edge wipe at any angle | [Transition][trans] | partial | `wipe` is four directions only; any angle via a keyed path `mask` (ADR-0163) |
| CC Radial ScaleWipe | Radial wipe that stretches the image | [Transition][trans] | reachable | prerender (distortion) |
| CC Scale Wipe | Stretch-off wipe | [Transition][trans] | reachable | prerender (distortion) |
| CC Twister | Twisting card-like turn | [Transition][trans] | reachable | prerender (distortion) |
| CC WarpoMatic | Luminance-driven warp transition | [Transition][trans] | reachable | prerender; a driver from another element refused (ADR-0150 §4) |
| Gradient Wipe | Wipe ordered by a gradient layer's luminance | [Transition][trans] | refused ADR-0150 | the gradient layer is another element (§4) |
| Iris Wipe | Polygon or circle iris opens | [Transition][trans] (menu) | reachable | keyed `mask` (`ellipse`, `rect`, `path`) with `feather` (ADR-0146, ADR-0152, ADR-0163) |
| Linear Wipe | Straight edge at any angle, feathered | [Transition][trans] | partial | `wipe` + `direction` (ADR-0150): 4 directions, hard edge; angle and feather via keyed path `mask` |
| Radial Wipe | Clock sweep around a centre | [Transition][trans] (menu) | reachable | keyed path `mask` drawing the sector (ADR-0163); no kind |
| Venetian Blinds | Striped reveal in bands | [Transition][trans] (menu) | reachable | one copy of the incoming element per band, each with a keyed `mask` (ADR-0148) |

## Matte (4)

| Effect | What it does | Adobe | Status | Evidence |
|---|---|---|---|---|
| Matte Choker | Chokes and spreads a matte in stages to fill holes | [Matte][matte] | gap | no alpha choke; `chroma` `tolerance`/`softness` act before the matte (ADR-0088) |
| Refine Hard Matte | Cleans a hard keyed edge, decontaminates | [Matte][matte] | partial | `chroma` `softness` + `spill` (ADR-0088); no edge refinement |
| Refine Soft Matte | Keeps hair and motion-blur detail at the edge | [Matte][matte] | partial | as above |
| Simple Choker | Grows or shrinks alpha by a few pixels | [Matte][matte] (menu) | gap | `mask` `feather` softens only its own shape (ADR-0152); grep "choke" empty |

The ticket names "Refine Matte". The current Matte menu holds Refine Hard Matte and Refine Soft
Matte, which replaced it (ProVideo Coalition's keying series, and Adobe's Matte page excerpts).
The page excerpts did not confirm a "mocha shape" entry, so it is not listed.

## Categories named, not scored row by row

- **Audio** ([Audio][audio], (menu) list): Backwards, Bass & Treble, Delay, Flange & Chorus,
  High-Low Pass, Modulator, Parametric EQ, Reverb, Stereo Mixer, Tone. The audio map
  [#795](https://github.com/MBehtemam/Montagent/issues/795) owns them all:
  - **Already on the capability map:** Bass & Treble (`shelf`), High-Low Pass (`highpass`, `lowpass`), Parametric EQ (`bell`). These come from ADR-0179, still marked proposed while its test table is committed.
  - **Open on #795:** Delay (ADR-0185 `echo`, draft), Stereo Mixer (ADR-0184 `pan`/`channels`, draft), Reverb (out of ADR-0185, prototype in `docs/research/audio-effects/echo-reverb/`), Flange & Chorus, Modulator, Tone.
  - **Backwards:** `source_time` on `audio` is a schema error (ADR-0157). Reversed sound is #795's to decide.
- **Obsolete** ([Obsolete][obs]): Adobe keeps these effects "for compatibility with projects
  created with previous versions" and says to use alternatives. The (menu) list: Basic 3D, Basic Text,
  Color Key, Fast Blur (Legacy), Gaussian Blur (Legacy), Lightning, Luma Key, Path Text,
  Spill Suppressor. Not scored. Their replacements are scored elsewhere: projection
  (ADR-0167), `text`, `chroma`, `blur`, Advanced Lightning, text on a path (ADR-0161).
- **Immersive Video** ([VR effects][vr], (menu) list): VR Blur, VR Chromatic Aberrations,
  VR Color Gradients, VR Converter, VR De-Noise, VR Digital Glitch, VR Fractal Noise, VR Glow,
  VR Plane to Sphere, VR Rotate Sphere, VR Sharpen, VR Sphere to Plane. Not scored:
  Montagent has no equirectangular or stereo frame model (grep "equirect" and "VR" find nothing
  in the ADRs). Whether 360° video is in scope at all is its own question for #896.

## Counts

Over the 156 rows scored in the nine categories:

| Status | Count |
|---|---|
| covered | 12 |
| partial | 18 |
| reachable | 91 (prerender first: 68; copies, keyed masks, skill recipes or a script: 23) |
| owned by an open map | 0 (the 10 Audio effects all go to #795 and are not counted) |
| refused | 13 (ADR-0150: 8, ADR-0167: 4, ADR-0156: 1) |
| gap | 22 |

By category (covered / partial / reachable / refused / gap): Distort 0/1/29/4/3, Generate
7/3/15/0/1, Noise & Grain 1/3/2/1/4, Perspective 1/1/3/2/3, Simulation 0/0/18/0/0, Stylize
2/4/12/1/6, Time 1/1/3/2/1, Transition 0/3/9/3/2, Matte 0/2/0/0/2.

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
4. **Generators and noise with a time axis.** Fractal Noise, Turbulent Noise, Cell Pattern and
   Turbulent Displace. ADR-0156 §3 allows a seed but no time input, and Evolution is time
   (#703). The questions: a keyed `evolution` parameter (like a keyed phase), or prerender for
   good? 4-Color Gradient is a separate question: a third `gradient` kind (bilinear corners)
   under ADR-0149.
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
   Detail-preserving Upscale, Remove Grain, Dust & Scratches and Median. None has a route. The
   tracking and stabilise results could be literal keyframes written by a skill (the
   "reachable by a skill" line on #896). Denoise and median are filter members.
9. **Lighting and 3D surface looks.** Bevel Alpha, Bevel Edges, Radial Shadow's light point and
   CC Sphere/Cylinder lighting. These are low priority. Skia's lit filters
   (`distant_lit_diffuse`, #703) are the route if any of them is wanted.

Already decided, and not to reopen: everything that reads a second element (Displacement Map,
Gradient Wipe, CC Image Wipe, CC Glass Wipe, Texturize, Match Grain, Time Difference, Time
Displacement, CC Environment) is refused by ADR-0150 §4. So are corner pin and skew (CC Power
Pin, Corner Pin, CC Slant; ADR-0167). An open shader file stays refused (ADR-0156 §8,
ADR-0017).
