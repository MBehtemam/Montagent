---
status: accepted
amends: 0020 (speed < 1.0 on video elements gains optional frame_blend: "none" | "mix" for linear frame cross-dissolve), 0036 (confirms whole-frame default and admits frame_blend: "mix" on speed-adjusted video), 0096 (confirms nearest-frame at or before instant as the default "none" rule, and specifies linear cross-dissolve between bracketing source frames under "mix"), 0145 (refuses multi-time sampling effects Echo and CC Wide Time and cross-element Time Displacement under self-containment and single-instant invariants), 0155 (reaffirms footage-internal motion blur and Pixel Motion Blur are refused from native render), 0156 (routes optical-flow retiming and temporal generators to montagent-prerender), 0157 (admits frame_blend: "none" | "mix" on remapped video carrying source_time; confirms refusal of in-engine optical flow), 0215 (disposes of After Effects Time effects and frame blending census rows from map #896)
---

# Frame mix enters video, and optical flow, echo, and posterize_time are refused

[How do frame blending (Frame Mix), optical flow (Pixel Motion) and the remaining AE Time effects enter?](https://github.com/MBehtemam/Montagent/issues/1074)
on the After Effects map ([#896](https://github.com/MBehtemam/Montagent/issues/896)).

[ADR-0020](0020-speed-overrun-hold-loop.md) established playback `speed` on media elements.
[ADR-0036](0036-shift-preambles-coincident-instants-validate-and-compare-stay-out.md) ratified frame alignment and playback boundaries.
[ADR-0096](0096-the-frame-at-an-instant-is-the-last-one-starting-at-or-before-it.md) established the discrete frame rule: the source frame shown at an instant is the last frame starting at or before it (*Whole Frames*).
[ADR-0145](0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md) established entry criteria and invariants: self-contained elements, flat literal JSON schema, and single-instant parallel paint evaluations.
[ADR-0155](0155-motion-blur-is-a-per-element-field-that-accumulates-the-element-over-a-centred-shutter.md) established `motion_blur` for authored transforms and keyed values, noting that video footage content is not blurred internally.
[ADR-0156](0156-four-named-effects-join-the-effects-list-and-a-code-drawn-piece-enters-as-pre-rendered-footage.md) established that heavy, stateful, or multi-pass processing lives on `montagent-prerender` as pre-rendered ProRes 4444 footage.
[ADR-0157](0157-a-speed-ramp-is-a-time-remap-curve-of-source-times-on-a-video-element.md) established `source_time` curves for speed ramps, evaluating the last source frame at or before the resolved source millisecond.
[ADR-0215](0215-after-effects-is-precedent-for-rendered-results-and-capcut-and-premiere-stay-first-for-shape.md) admitted After Effects precedent for visual results while keeping Premiere and CapCut first for mechanism.

The After Effects censuses ([#898](https://github.com/MBehtemam/Montagent/issues/898) rows 106–108, [#899](https://github.com/MBehtemam/Montagent/issues/899) rows 32–33, and [#905](https://github.com/MBehtemam/Montagent/issues/905) rows 132–142) compared After Effects' Time features (*Frame Blending: Frame Mix / Pixel Motion*, *Timewarp*, *Echo*, *Posterize Time*, *CC Wide Time*, *Time Difference*, *Time Displacement*) against Montagent's `video` element and `effects[]`.

This ADR decides:
1. **`frame_blend: "none" | "mix"` enters `video`**: Allows deterministic linear cross-dissolve between bracketing source frames when slow-motion playback or time-remapping samples fractional source frames.
2. **Optical-flow interpolation (*Pixel Motion*, *Timewarp*) is refused from runtime `render`**: Retiming via bidirectional optical flow (`minterpolate`) is non-local and computationally prohibitive for per-frame parallel painters; it is standardized as a `montagent-prerender` recipe.
3. **Temporal `effects[]` (*Echo*, *Posterize Time*, *CC Wide Time*, *Time Difference*, *Time Displacement*) are refused from schema**: Multi-time sampling breaks the single-instant render model, and rate quantisation is already cleanly achieved via stepped `source_time` curves and `step` keyframe easing.

---

## Decisions

### 1. `frame_blend` enters `video` as `"none" | "mix"`

A `video` element may declare an optional `frame_blend` field:

```json
{
  "type": "video",
  "src": "assets/action.mp4",
  "start": 0,
  "end": 4000,
  "speed": 0.5,
  "frame_blend": "mix"
}
```

#### Values and semantics
- **`"none"`** (default):
  - Preserves the existing behaviour of ADR-0096 and ADR-0157 §3: the decoded frame is the last source frame starting at or before the resolved source millisecond (*Whole Frames* / nearest frame).
  - Guarantees exact backward compatibility, zero decode overhead, and crisp original source frames.
- **`"mix"`**:
  - When the element plays at reduced speed (`speed < 1.0`), has a fractional `source_time` slope, or when `source_time` resolves between two source frame presentation timestamps $T_A$ and $T_B$:
    - Let $T_A$ be the presentation timestamp of the last source frame starting at or before the resolved instant $S(t)$, and $T_B$ be the presentation timestamp of the immediate next source frame ($T_A \le S(t) < T_B$).
    - If $S(t) == T_A$ or $T_B$ lies beyond the end of the video stream, frame $A$ is drawn with opacity $1.0$.
    - Otherwise, the blend factor is:
      $$w = \frac{S(t) - T_A}{T_B - T_A}$$
    - The painter decodes both frame $A$ and frame $B$ and composites them linearly:
      $$\text{Pixel} = (1 - w) \cdot \text{Frame}_A + w \cdot \text{Frame}_B$$
    - In Skia, this is evaluated by drawing $\text{Frame}_A$ followed by $\text{Frame}_B$ at opacity $w$ in `SkBlendMode::SrcOver` (premultiplied linear cross-dissolve), or via an equivalent linear blend.

#### Validation
- `frame_blend` is valid only on `type: "video"`. On any other element type (`audio`, `image`, `text`, `rect`, `nest`), it is a `validate` schema error.
- Valid values are strictly `"none"` and `"mix"`. Any other string (including `"pixel_motion"`, `"optical_flow"`, or `"frame_mix"`) produces `E-INVALID-FRAME-BLEND`.

---

### 2. Optical-flow interpolation (*Pixel Motion*, *Timewarp*) is refused from runtime `render`

After Effects provides *Pixel Motion* frame blending and the *Timewarp* effect, which estimate pixel velocity vectors across adjacent frames and morph geometry along flow trajectories.

#### Rationale for refusal from runtime `render`
1. **Parallel thread independence**: Montagent's painter threads render frame instants in parallel without cross-frame dependencies. Optical flow requires sequential multi-frame state and iterative motion-field optimisation (e.g. Farnebäck, DeepFlow, or TV-$L^1$).
2. **Latency budget**: Optical flow estimation at 1080p or 4K requires hundreds of milliseconds to several seconds per frame, completely exceeding Montagent's interactive and parallel render latency ceilings.
3. **Artifact predictability**: Optical flow creates unpredictable morphing tear artifacts on occlusion boundaries, violating Montagent's deterministic, inspectable graphics model.

#### Standard `montagent-prerender` recipe
Optical-flow slow-motion is routed to `montagent-prerender` per ADR-0156 §7:
- The agent runs FFmpeg's `minterpolate` filter as a pre-rendering step:
  ```bash
  ffmpeg -i source.mp4 -vf "minterpolate=fps=60:mi_mode=mci:mc_mode=aobmc:me_mode=bidir:vsbmc=1" -c:v prores_ks -profile:v 4 -pix_fmt yuva444p10le prerendered_slowmo.mov
  ```
- The generated lossless mezzanine is placed in the project directory and referenced as a regular `video` element with clean 1:1 frame playback.

---

### 3. Temporal effects (*Echo*, *Posterize Time*, *CC Wide Time*, *Time Difference*, *Time Displacement*) are refused from schema

The After Effects Time category effects are resolved as follows:

1. **`Posterize Time` is refused from `effects[]`**:
   - In After Effects, *Posterize Time* forces an element to update at a lower authored frame rate (e.g. 12 fps on a 24 fps timeline).
   - In Montagent, `effects[]` operates per-pixel on a rasterised snapshot at a single timeline instant; an effect inside `effects[]` does not know or control the caller's timeline sampling instant.
   - For `video` elements, rate quantisation is already achieved with exact millisecond precision via stepped `source_time` curves ([ADR-0157](0157-a-speed-ramp-is-a-time-remap-curve-of-source-times-on-a-video-element.md)) or `step` ease on keyframe points.
   - For animated properties and transforms, rate quantisation is achieved via `step` keyframe easing ([ADR-0148](0148-hold-and-cubic-bezier-enter-the-easing-model-steps-repeat-and-spring-do-not.md)) or generated via the `bake_motion` skill script ([ADR-0217](0217-motion-bakes-keyframe-assistants-and-skill-script-templates-enter-the-toolchain-and-auto-colour-is-refused.md)).
2. **`Echo` and `CC Wide Time` are refused from `effects[]`**:
   - *Echo* combines $N$ frames across past/future time offsets $t \pm k \cdot \Delta t$ with decay.
   - Evaluating *Echo* at runtime inside an element effect requires re-evaluating the entire upstream element graph (video decoding, transform resolution, text/path layout) $N$ additional times at $N$ different points in time for every rendered frame, multiplying decode and rendering load by $N+1$.
   - Trailed motion is authored cleanly by duplicating the element across tracks with offset `start` / `source_start` / `source_time` and decaying `opacity`, or pre-rendered via `montagent-prerender`.
3. **`Time Difference` is refused**:
   - Subtracts pixels from a temporally offset frame; refused for the same multi-instant sampling reasons as *Echo*. Achievable via two tracks set to `blend_mode: "difference"`.
4. **`Time Displacement` is refused**:
   - Uses a secondary layer's luminance to displace the time lookup per pixel.
   - Violates element self-containment ([ADR-0150](0150-fade-black-wipe-and-slide-enter-as-transitions-track-mattes-do-not.md) §4, [ADR-0145](0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md) §1) and requires non-uniform temporal decoding across a single frame.
5. **`Pixel Motion Blur` is refused**:
   - Reaffirmed under ADR-0155: footage-internal optical flow blur is refused from native render and handled by pre-processing footage.

---

## Graduated Tickets

This decision graduates one implementation task on [Map: what does After Effects do that Montagent doesn't, and which of it enters?](https://github.com/MBehtemam/Montagent/issues/896):
- **Implement `frame_blend: "none" | "mix"` on `video` elements per ADR-0232**:
  1. Add `frame_blend: Option<FrameBlend>` (`"none" | "mix"`, default `"none"`) to `Video` schema in `montagent-core`.
  2. Implement validation rules and error `E-INVALID-FRAME-BLEND`.
  3. In `montagent-render`, when `frame_blend == Some(FrameBlend::Mix)`, decode adjacent bounding source frames when sampling fractional source timestamps and cross-dissolve with weight $w$.
  4. Add tests verifying whole-frame identity under `"none"`, exact 50/50 blend at midpoints under `"mix"`, and schema validation.
