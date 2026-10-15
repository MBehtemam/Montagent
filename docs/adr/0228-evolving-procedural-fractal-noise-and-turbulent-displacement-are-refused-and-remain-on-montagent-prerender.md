---
status: accepted
amends: 0040 (confirms closed effects vocabulary), 0156 (reaffirms grain is discrete frame-hashed film noise without evolution, and confirms procedural fractal noise generators stay on montagent-prerender), 0215 (disposes of After Effects Noise & Grain rows 143-149 and Distort procedural rows from map #896), 0225 (confirms refusal of turbulent_displace from effects[])
---

# Evolving procedural fractal noise and turbulent displacement are refused and remain on montagent-prerender

[Does evolving procedural noise enter grain / effects[] under ADR-0156, or stay on montagent-prerender?](https://github.com/MBehtemam/Montagent/issues/1070)
on the After Effects map ([#896](https://github.com/MBehtemam/Montagent/issues/896)).

[ADR-0040](0040-effect-model-attachment-and-v1-vocabulary.md) established the ordered `effects[]` list filtering an element's own pixels.
[ADR-0145](0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md) established the four core invariants: literal values, closed vocabulary, checkable by `validate`, and exact-string replace.
[ADR-0156](0156-four-named-effects-join-the-effects-list-and-a-code-drawn-piece-enters-as-pre-rendered-footage.md) admitted `grain` as a discrete film-noise offset re-rolling per output frame from a required static `seed` and the element's local frame (§3, §4), refused open shaders or script files within the format (§8), established the byte-identity gate across parallel painters (§6), and established the `montagent-prerender` route for code-drawn and procedural looks (§7).
[ADR-0215](0215-after-effects-is-precedent-for-rendered-results-and-capcut-and-premiere-stay-first-for-shape.md) admitted After Effects precedent for rendered results while keeping Premiere and CapCut first for mechanism and shape.
[ADR-0225](0225-edge-sampling-is-clamp-within-bounds-mosaic-radial-blur-sharpen-find-edges-emboss-and-four-distortions-enter-effects.md) admitted closed parametric coordinate distortions (`twirl`, `bulge`, `optics_compensation`, `offset`) while deferring procedural noise distortions (*Turbulent Displace*, *Wave Warp*) to `montagent-prerender` or ticket #1070.

The After Effects censuses ([#905](https://github.com/MBehtemam/Montagent/issues/905) rows 143–149) identified that After Effects provides procedural noise with continuous temporal variation (*Evolution*) in *Fractal Noise*, *Turbulent Noise*, and *Curl Noise*, whereas Montagent's `grain` is discrete per-frame white noise with no continuous evolution or rate parameter.

This ADR decides:
1. **`grain` remains discrete per-frame hashed film noise without an `evolution` parameter**: `grain` preserves its ADR-0156 §3–§4 model, re-rolling discrete white noise per output frame based on `(seed, local_frame)`.
2. **Continuous evolving fractal noise generators (*Fractal Noise*, *Turbulent Noise*, *Curl Noise*) are refused from `effects[]`**: Multi-octave 3D/4D procedural noise generators remain on `montagent-prerender` and do not enter the runtime schema.
3. **Procedural noise-driven coordinate distortions (*Turbulent Displace*) are refused from `effects[]`**: Coordinate remapping remains strictly parametric and closed-form; vector displacements driven by procedural noise remain on `montagent-prerender`.
4. **Evolving noise and procedural turbulence are reached via standard `montagent-prerender` recipes**: Code-drawn and shader-generated textures enter as lossless ProRes 4444 video with alpha and a deterministic sidecar recipe per ADR-0156 §7.

---

## Decisions

### 1. `grain` preserves its discrete per-frame model

Physical film grain consists of silver halide crystals exposed per frame. In photographic and cinematic pipelines, grain is spatially independent from one frame to the next.

- Adding an `evolution` or continuous morphing parameter to `grain` simulates fluid, smoke, or crawling gelatin rather than film grain, confusing the domain model.
- `grain` continues to take a required static `seed`, static `size` (1–8), static `mono` flag, and an animatable `amount` (`[0.0, 1.0]`).
- On every output frame, `grain` re-rolls deterministically by hashing `(seed, local_frame, x, y)`.
- No `evolution`, `rate`, or temporal smoothing parameter enters `grain`.

### 2. Evolving fractal noise generators are refused from `effects[]`

Procedural noise generators such as After Effects' *Fractal Noise*, *Turbulent Noise*, and *Curl Noise* evaluate fractional Brownian motion (fBm) or multi-octave 3D/4D Simplex/Perlin noise grids:

1. **Byte-identity failure risk**: Multi-octave 3D float math, trigonometric permutation hashes, and gradient interpolation in runtime shaders (SkSL / GPU / CPU) risk platform- and compiler-specific floating-point divergence across parallel painter threads, violating ADR-0156 §6.
2. **Latency budget**: Evaluating 4 to 8 octaves of 3D Simplex noise per pixel at 1080p or 4K resolution severely degrades Montagent's single-frame preview and parallel render throughput.
3. **Format boundaries**: ADR-0156 §1 established that `effects[]` members filter existing element pixels; there is no generator element that paints from nothing.

Evolving fractal noise is therefore refused from the format's `effects[]` list and remains on `montagent-prerender`.

### 3. Procedural coordinate distortions (*Turbulent Displace*) are refused from `effects[]`

ADR-0225 admitted closed, analytic coordinate remapping filters (`twirl`, `bulge`, `optics_compensation`, `offset`) whose transformation vector is a direct, closed-form function of element geometry.

- Procedural coordinate distortions like *Turbulent Displace* require sampling a 2D/3D procedural noise gradient field to derive horizontal and vertical pixel offset vectors.
- Because the underlying multi-octave noise field is refused from runtime shaders under §2, *Turbulent Displace* and *Wave Warp* are refused from `effects[]`.
- They remain on `montagent-prerender`.

### 4. Evolving noise is reached via `montagent-prerender` recipes

Montagent's answer to animated organic textures (clouds, mist, fire, fluid turbulence, water caustics) remains ADR-0156 §7:

- The agent or user renders the generative pattern outside the project via a deterministic script or shader (e.g. using Python, ffmpeg, or an offline shader renderer).
- The output is exported to a lossless video file with alpha (`prores_4444`).
- Beside the video, a sidecar recipe (`.recipe.json`) records the source script, command, parameters, frame rate, duration, and frame hashes.
- The project references the resulting video as an ordinary `video` element with standard `blend` modes (`overlay`, `screen`, `soft_light`) or as a `mask`.

---

## Consequences

- No changes to `crates/montagent-core` model types or `schema/montagent.schema.json`.
- `grain` remains strictly defined by ADR-0156 §3–§4.
- Census rows 143–149 of [#905](https://github.com/MBehtemam/Montagent/issues/905) for *Fractal Noise*, *Turbulent Noise*, *Curl Noise*, and *Turbulent Displace* are resolved as refused from the core format and reachable via `montagent-prerender`.
