# Multi-Element Operations and Adjustment Layers Under ADR-0145

**Research Ticket**: [#981](https://github.com/MBehtemam/Montagent/issues/981)  
**Parent Map**: [#978](https://github.com/MBehtemam/Montagent/issues/978) (Premiere Parity Implementation Architecture & Roadmap)  
**Status**: Complete  
**Date**: October 2026  

---

## 1. Executive Summary

This investigation resolves research ticket [#981](https://github.com/MBehtemam/Montagent/issues/981): *"What is the architectural model for multi-element operations and adjustment layers under ADR-0145?"*

The research examines how operations that filter or affect multiple elements—such as adjustment layers, group-level color grading, or sequence-wide looks—can be modeled in Montagent without breaking element-local rendering, the closed-vocabulary invariant, or single-pass rendering performance.

### Key Conclusions:

1. **A first-class `type: "adjustment"` element can satisfy all four invariants of ADR-0145 syntactically.**  
   It uses literal parameters, draws exclusively from the closed effect vocabulary, is statically checkable by `validate` (schema, bounds, and stacking without painting), and is directly editable by exact-string replacement.
2. **However, an `adjustment` element severely disrupts the rendering engine's operational assumptions.**  
   In a single-pass streaming Skia rasterizer, an adjustment layer demands **unbounded backward backdrop read-back** (`surface.image_snapshot()`). At 4K/8K resolutions, this imposes massive memory bandwidth penalties (~33 MB/frame copy at 4K), breaks the static prefix cache ([ADR-0195](../../adr/0195-the-static-prefix-cache-holds-invariant-layers-at-the-bottom-of-the-stack.md), [ADR-0198](../../adr/0198-an-element-that-draws-nothing-neither-joins-nor-ends-the-static-prefix.md)), creates synchronization barriers across parallel $K$-painters ([#653](https://github.com/MBehtemam/Montagent/issues/653)), and introduces ambiguous spatial boundary reach problems for kernel filters like blur and grain.
3. **Premiere precedent distinguishes clip, track, and container mechanisms.**  
   Primary source audit of Premiere Pro demonstrates that video effects apply **only to individual clips or adjustment layers**—*Premiere has no video track effects* (`docs/research/premiere-gap/effects.md` row 145). Multi-clip filtering in Premiere is accomplished either via **Adjustment Layers**, **Nested Sequences** (`effects.md` row 144), or **Paste Attributes** (`effects.md` row 137).
4. **Architectural Recommendation: Do not implement a free-floating `adjustment` layer.**  
   Instead, adopt a two-tier architectural model that delivers full Premiere parity while preserving single-pass forward compositing:
   - **Tier 1 (Sequence / Global Grades): Project-Level Master Effects (`project.effects`).** A root-level ordered `effects: [...]` list evaluated once on the finished canvas prior to final encoding. This requires zero backdrop snapshotting, preserves the static prefix cache, and covers the dominant real-world use case of adjustment layers (overall sequence color grading and film grain).
   - **Tier 2 (Regional / Multi-Element Sub-Compositing): Bounded Group Compositing (`nest`).** Rather than reading backwards into an open backdrop, nested child elements render forward into an isolated buffer, undergo group-level `effects` and `blend`, and composite onto the parent canvas. This respects [ADR-0166](../../adr/0166-capcut-and-premiere-nest-with-a-transform-and-adr-0001-refuses-the-clock-not-the-shared-space.md)'s shared spatial coordinate model without compromising timeline clocks.
   - **Complementary Workflow: Agentic Clip-Scoped Replication (Paste Attributes).** For disparate clips sharing a grade without spatial coupling, AI agents replicate literal `effects: [...]` blocks across target elements via exact-string replace, incurring zero rendering architectural overhead.

---

## 2. Investigation of Primary Sources

### 2.1 CONTEXT.md Foundations

`CONTEXT.md` establishes foundational domain definitions that constrain any multi-element design:

- **Element** (`CONTEXT.md:63-70`):  
  *"One thing placed on the timeline — an image, a video, an audio file, a piece of text or a shape. Every element has a type, a required unique `id`, a time range and an optional `group`... It sits in a track, which supplies its stacking position unless the element overrides it."*
- **Layer** (`CONTEXT.md:84-94`):  
  *"A place in the stack, as an integer — higher draws in front... Two elements resolving to the same layer is legal only while their boxes never overlap in time and space; once they do, the document does not say which draws in front and `validate` errors... until the author states an order explicitly."*
- **Effect** (`CONTEXT.md:436-460`):  
  *"A member of a closed, named, parameterised vocabulary in `effects: [...]` on an element — never an open plugin architecture... **An effect filters the element's own pixels: there is no element that paints from nothing, so a texture is a `rect` carrying an effect and a blend. Effects attach to whole elements... and it stays inside the element's box or a reach it declares.**"*
- **Blend Mode** (`CONTEXT.md:605-618`):  
  *"How a finished element is composited into its **backdrop**, written as the element's flat `blend` field beside `opacity`... **The backdrop is everything painted below the element in the stack, down to the `background` — it is not the `background`, which is only its bottom.** The finished element is what blends: its effects, masks and shadow, then its `opacity`, composited once. Not an `effects` member, because an effect is element-local and a blend mode reads what is under the element."*

### 2.2 Relevant Architectural Decision Records (ADRs)

- **ADR-0040 (Effect Model and Attachment)**:  
  Fixed effects as an ordered list field on the element (`"effects": [...]`). It explicitly rejected generator/synthetic effects (*"an effect filters an existing element; it never authors new visual content. This is the line that keeps the model from becoming a rendering language"*).
- **ADR-0049 (Colour Filter Vocabulary)**:  
  Established the four-clause admissibility stopping rule: (a) fixed parameter count, (b) bounded scalars, (c) documented identity value, and (d) non-composability. Admitted `tint`, `saturation`, `brightness`, and `contrast`.
- **ADR-0145 (The Four Invariants and Reference Class)**:  
  Established that CapCut and Premiere are the first reference class, read by mechanism. Every capability must uphold the four invariants:
  1. *Literal values*: Fixed, schema-documented arithmetic only; no runtime expressions.
  2. *Closed vocabulary*: Every key and enum listed in schema.
  3. *Checkable by `validate`*: Malformed or out-of-range values identified statically without painting a frame.
  4. *Exact-string replace*: Every authored choice has its own string.
- **ADR-0147 (Blend Modes)**:  
  Defined the compositing pipeline:
  1. The element is drawn through its ordered `effects`;
  2. `opacity` scales the result;
  3. The result is composited **once** into its backdrop using `blend` inside `clip`.  
  Crucially noted: *"An effect is element-local, and a blend mode reads what is under the element."*
- **ADR-0166 & ADR-0167 (Containers, Shared Space, and Group Compositing)**:  
  ADR-0166 corrected ADR-0003's assertion that the reference class was purely flat: Premiere nested sequences and CapCut compound clips group children under a shared transform. ADR-0001 only rejected local clocks, not shared spatial coordinates.  
  ADR-0167 §10 expressly identified the core hurdle of group-level effects: *"A projected nest, one group turned as one card, means flattening the group into one layer first. That is group compositing, which nests refuse. It is left in the map's fog, gated on the nest's score and on group compositing."*

### 2.3 Premiere Pro Gap Research

The Premiere inventory documents the precise state of multi-element operations:

| Capability | Inventory Ref | Current Status in Montagent | Mechanism in Reference Class / Montagent |
|---|---|---|---|
| **Adjustment layers** | `effects.md:142`, `editing.md:226` | Missing / Partial | In Premiere: A transparent synthetic clip on a higher track whose applied effects filter all tracks beneath it. In Montagent: A full-frame `rect` with `blend` mode gives a color wash/tint, but cannot filter backdrop pixels. |
| **Resize adjustment layer** | `effects.md:143` | Missing | In Premiere: Scaling and positioning an adjustment layer restricts its effect to a spatial sub-region of the frame. |
| **Nested sequence effects** | `effects.md:144` | Missing | In Premiere: Applying effects to a nested sequence filters the flattened composite of the enclosed clips. |
| **Clip-based video effects** | `effects.md:145` | Have | In Premiere: Video effects apply to individual clips. **Video effects do NOT exist on tracks in Premiere**; only audio effects can be placed on tracks. |
| **Paste Attributes** | `effects.md:137` | Have | In Premiere: Copying effects from one clip and applying them across selected clips. In Montagent: Literal JSON `effects` lists are duplicated across elements via exact-string replace. |
| **Render and Replace** | `editing.md:227` | Partial | Flattens timeline spans to intermediate media. Handled in Montagent via `render --from --to` and `montagent-prerender`. |

---

## 3. Analysis of Montagent's Current Rendering Architecture

An examination of `crates/montagent-render/src/canvas.rs` and `crates/montagent-core/src/verbs/frame/mod.rs` reveals the exact operational mechanics of frame generation:

### 3.1 Single-Pass Streaming Loop

In `crates/montagent-core/src/verbs/frame/mod.rs:1317-1338`, the frame painter executes sequentially:
```rust
pub(crate) fn paint(&mut self, canvas: &mut Canvas, view: &At) {
    self.resolve_transitions();
    canvas.background(self.background());

    let prefix = self.prefix_split(view);
    if !prefix.drawn.is_empty() {
        self.paint_prefix(canvas, &prefix.drawn);
    }
    for present in prefix.drawless {
        self.paint_present(canvas, present);
    }
    for present in &view.stack[prefix.len..] {
        self.paint_present(canvas, present);
    }
}
```

1. The canvas is initialized with the project's background color.
2. Invariant bottom layers are restored from the **Static Prefix Cache** (`cached.pixels`).
3. Remaining elements in `view.stack` (sorted by ascending resolved integer `layer`) are painted sequentially front-to-back.
4. Each element executes `paint_present` $\rightarrow$ `self.sharp` $\rightarrow$ rasterizer dispatch (`shape`, `text`, `raster`, `path`).

### 3.2 Element-Local Layer Isolation

Inside `crates/montagent-render/src/canvas.rs:1900-1951`, each visual element opens an isolated layer for its opacity and blend mode:
```rust
let layered = transform.opacity < 1.0 || transform.blend != Blend::Normal;
if transform.blend == Blend::Normal {
    if layered { canvas.save_layer_alpha_f(None, transform.opacity as f32); }
} else {
    let mut paint = SkPaint::default();
    paint.set_alpha_f(transform.opacity.min(1.0) as f32);
    paint.set_blend_mode(transform.blend.mode());
    canvas.save_layer(&SaveLayerRec::default().paint(&paint));
}
```
Inside this layer, `Canvas::through` (`canvas.rs:1980-2046`) nests `save_layer` calls for each member of the element's ordered `effects` list.

**Crucial Architectural Invariant**:  
At no point does `Canvas::through` or any rasterizer read from the destination canvas surface. The element draws its **own isolated geometry/pixels**, filters **its own layer**, and composites the finished result **forward** onto the canvas via Skia's Porter-Duff blend modes. The destination canvas is strictly write-only during element execution.

---

## 4. What Makes Adjustment Layers Hard in a Declarative Single-Pass Renderer?

Implementing a traditional adjustment layer—defined as an element that applies an `effects` chain to the accumulated composite of everything rendered beneath it—introduces three major architectural and engineering conflicts:

```
[Layer 3: Title Text]       ──> Drawn over adjusted backdrop
─────────────────────────────────────────────────────────────────────────────
[Layer 2: Adjustment Layer] ──> REQUIRES READ-BACK of Canvas (Layers 0 + 1)
                                1. Snapshot destination surface
                                2. Pass snapshot through ImageFilter chain
                                3. Composite filtered snapshot over canvas
─────────────────────────────────────────────────────────────────────────────
[Layer 1: B-Roll Video]     ──┐
[Layer 0: Background Plate] ──┴─> Accumulated Backdrop
```

### 4.1 Backdrop Read-Back and Canvas Pipeline Disruption

1. **Surface Read-Back / Snapshot Overhead**:  
   In Skia, an active `SkSurface` cannot be sampled as an `SkImage` while simultaneously serving as the draw target without an explicit copy or snapshot (`surface.image_snapshot()`).  
   - In CPU software rendering (`skia-safe` without GPU), taking an image snapshot of the active canvas forces an allocation and memcpy of the full device bitmap.
   - For a 4K UHD frame ($3840 \times 2160$ premultiplied RGBA8), each snapshot consumes **33.18 MB**. For an 8K frame, it consumes **132.7 MB**.
   - If a timeline includes multiple adjustment layers (e.g. base grade on layer 10, vignette adjustment on layer 20), memory bandwidth escalates dramatically, destroying scrub-preview throughput budgets (see `docs/research/prototypes/heavy-composite-stack/FINDINGS.md`).
2. **Static Prefix Cache Invalidation ([ADR-0195](../../adr/0195-the-static-prefix-cache-holds-invariant-layers-at-the-bottom-of-the-stack.md))**:  
   Montagent accelerates rendering by caching the invariant bottom-most layers across consecutive frames (`paint_prefix`). If an adjustment layer has animated effect parameters (e.g., keyed contrast or brightness), it cannot join the static prefix. More critically, if an adjustment layer spans above the prefix, the cached prefix pixels cannot simply be restored; the backdrop must be read back and re-filtered every frame, invalidating downstream cached states.
3. **Multi-Painter Concurrency Barriers ($K$-Painters)**:  
   Montagent renders frames in parallel across CPU worker threads ([#653](https://github.com/MBehtemam/Montagent/issues/653)). While frame-level parallelism is decoupled, proxy canvas scaling ([ADR-0186](../../adr/0186-proxy-sampling-reads-one-mip-level.md)) and tiled evaluation rely on strict determinism. Skia's `SaveLayerRec::backdrop` filter behaves inconsistently across software canvas tiling configurations, creating potential byte-level divergence.

### 4.2 Layer Ordering and Timeline Stacking Semantics

1. **Flat Integer Layers vs. Track Containers**:  
   In Premiere Pro, tracks are physical containers; an adjustment layer on video track $V_3$ automatically filters $V_1$ and $V_2$ across its entire temporal extent.  
   In Montagent, tracks provide stacking defaults, but elements carry absolute time spans and explicit integer layers (`layer: i64`), with ties strictly forbidden when boxes overlap in space and time ([ADR-0060](../../adr/0060-layer-tie-is-an-error-array-order-stays-meaningless.md)).  
   - If an adjustment element sits at resolved `layer: 50`, its backdrop is defined as all elements with resolved `layer < 50` intersecting its spatial and temporal footprint.
2. **Interaction with Layer Anchoring ([ADR-0019](../../adr/0019-layer-anchor-gets-an-id-a-validate-check-and-one-hop.md))**:  
   If an element specifies `layer: {"below": "adjust-01"}`, it is filtered by the adjustment layer. If another element specifies `layer: {"above": "adjust-01"}`, it is exempt. Because anchor resolution is strictly one hop, any subtle restructuring of element IDs can inadvertently shift an element into or out of the filtered backdrop.
3. **Interaction with Transitions ([ADR-0059](../../adr/0059-transitions-element-type-crossfade-only-exact-window.md), [ADR-0150](../../adr/0150-wipe-slide-and-push-enter-as-transition-kinds-and-a-matte-from-another-element-is-refused.md))**:  
   Transitions bridge two elements and resolve into cuts, opacities, and offsets before painting. If an adjustment layer's layer falls between the outgoing element's layer and the incoming element's layer, the transition crossfade is broken across the filter boundary: one half of the dissolve is graded, while the other is raw, producing severe perceptual flicker.

### 4.3 Spatial Bounds and Boundary Reach

1. **Full-Frame vs. Sub-Region Adjustments**:  
   In Premiere, an adjustment layer can be scaled or cropped to affect only a portion of the screen (e.g. localized face blur or lower-third grade).  
   In Montagent, if an `adjustment` element defines `width: 800, height: 600`, what are its sampling boundaries?
2. **Kernel Filter Reach Over Backdrops**:  
   For a local element (e.g., a `rect`), [ADR-0040](../../adr/0040-effect-model-attachment-and-v1-vocabulary.md) and `crates/montagent-render/src/canvas/layer_bound.rs` compute explicit bounds hints based on blur radius and shadow offsets.  
   If an adjustment layer applies `blur{radius: 20}` to a sub-rectangle of the canvas:
   - Does it sample backdrop pixels *outside* its bounds? If so, the read-back region must be expanded by the kernel reach (20 px outward in all directions), requiring a margin read-back.
   - If the read-back is clamped to the element's box, the edges will fade or clamp abruptly against the unfiltered background, creating harsh seam artifacts.
3. **Coverage and Lint Analysis**:  
   Montagent's query engine computes `NOT COVERED` ([ADR-0011](../../adr/0011-tool-surface-reads-checks-renders.md)) from element bounding boxes. An adjustment layer does not generate opaque pixels; it alters existing pixels. If placed over an empty or background-only area, it produces no meaningful output. An adjustment layer would require dedicated static check rules (`R-ADJUSTMENT-BACKGROUND-ONLY`, analogous to [ADR-0147](../../adr/0147-a-blend-mode-is-a-flat-static-field-of-five-values-and-the-finished-element-blends-last.md)'s `R-BLEND-BACKGROUND-ONLY`).

---

## 5. Can an Element Type `adjustment` Satisfy the Four Invariants of ADR-0145?

Suppose Montagent specified an element of type `adjustment`:

```json
{
  "type": "adjustment",
  "id": "grade-cinematic",
  "start": 0,
  "end": 15000,
  "track": "grading",
  "layer": 100,
  "x": 960,
  "y": 540,
  "width": 1920,
  "height": 1080,
  "effects": [
    {"name": "saturation", "amount": 0.85},
    {"name": "contrast", "amount": 0.1},
    {"name": "grain", "amount": 0.15, "seed": 42}
  ],
  "opacity": 1.0,
  "blend": "normal"
}
```

We evaluate this candidate against the normative tests in ADR-0145 §2:

| Invariant | Normative Test | Evaluation for `type: "adjustment"` | Verdict |
|---|---|---|---|
| **Literal values** | *Every value is a number, string, boolean, list or object written in the file. The renderer may apply fixed, schema-documented arithmetic, never an author-written expression or a reference to another value.* | All properties (`start`, `end`, `width`, `height`, `x`, `y`, `effects`, `opacity`, `blend`) are literal numbers, strings, or lists. Keyframed parameters use literal `[t, value]` pairs. Effect arithmetic is fixed in Skia. | **PASSES** |
| **Closed vocabulary** | *Every key and enum value is listed in the schema. A path may name media to decode, never a program to run.* | `type: "adjustment"` is an enumerated member of the closed `type` union. Its allowed keys are enumerated in schema. It accepts only members of the closed 12-member `effects` vocabulary. | **PASSES** |
| **Checkable by `validate`** | *A malformed or out-of-range value is a finding from the file and its probed assets alone, without painting a frame.* | Checked without painting: schema validation, integer layer conflict detection ([ADR-0060](../../adr/0060-layer-tie-is-an-error-array-order-stays-meaningless.md)), spatial bounds intersection, and static background-only checks (`R-ADJUSTMENT-BACKGROUND-ONLY`). | **PASSES** |
| **Exact-string replace** | *Every authored choice has its own string. A derived thing is changed through the literal that generates it, and the schema documents how to single one out.* | An adjustment layer's parameters, time bounds, and individual effect entries are directly targeted and edited in place via standard JSON string replacement. Removing an effect is deleting its JSON object. | **PASSES** |

### Crucial Finding: Invariant Compliance vs. Architectural Viability

While `type: "adjustment"` **passes all four invariants of ADR-0145**, passing the invariants is a **necessary, not sufficient** condition for admission. As ADR-0145 §3 explicitly mandates:
> *"Acceptance is recorded as the prototype's commit pin, the linked render and the owner's quoted words. It is necessary, not sufficient: the capability's own ADR decides the shape and may still say no, recorded with its reason."*

An adjustment element passes the document-level invariants but violates the **engine-level architectural axioms**:
1. It invalidates `CONTEXT.md`'s foundational rule: *"An effect filters the element's own pixels: there is no element that paints from nothing."*
2. It breaks the single-pass forward streaming model of the rasterizer.
3. It incurs prohibitive memory copy costs during multi-painter rendering.

---

## 6. Architectural Alternatives

To address multi-element operations under ADR-0145 without the penalties of open backdrop read-back, four structural alternatives exist:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Architectural Patterns for Multi-Element Operations                         │
├─────────────────────────────────────────────────────────────────────────────┤
│ 1. Project-Level Master Effects (Sequence Grade)                            │
│    Root "effects": [...] applied ONCE at end of frame paint.                │
│    ✓ Zero mid-frame read-backs  ✓ Static prefix preserved  ✓ 100% Invariants│
├─────────────────────────────────────────────────────────────────────────────┤
│ 2. Bounded Group Compositing (Nest Container)                              │
│    Child elements render forward into isolated buffer, filtered, composited.│
│    ✓ Strict spatial bounds      ✓ Clean subpass isolation  ✓ 100% Invariants│
├─────────────────────────────────────────────────────────────────────────────┤
│ 3. Agentic Clip-Scoped Duplication (Paste Attributes)                       │
│    AI Agent writes identical literal effects blocks across target clips.    │
│    ✓ Zero renderer changes      ✓ Perfect performance      ✓ 100% Invariants│
├─────────────────────────────────────────────────────────────────────────────┤
│ 4. Pre-Rendered Footage Passes (montagent-prerender)                        │
│    Timeline span exported to intermediate video and imported as single clip.│
│    ✓ Decoupled heavy processing ✓ Lossless intermediate    ✓ 100% Invariants│
└─────────────────────────────────────────────────────────────────────────────┘
```

### Alternative 1: Project-Level Master Effects (`project.effects`)

In Premiere, users frequently apply an adjustment layer across the entire sequence on the top track to establish a sequence-wide look (e.g. master LUT, film grain, or unified contrast).

In Montagent, this is modeled cleanly as a top-level `effects` list on the project root:
```json
{
  "width": 1920,
  "height": 1080,
  "fps": 30,
  "duration": 60000,
  "background": "#000000",
  "effects": [
    {"name": "contrast", "amount": 0.08},
    {"name": "saturation", "amount": 0.92},
    {"name": "grain", "amount": 0.12, "seed": 101}
  ],
  "tracks": [ ... ]
}
```

- **Rendering Implementation**: Applied exactly once per frame *after* all tracks and elements are painted onto the canvas, immediately before `canvas.rgb()` is fed to the video encoder or snapshot engine.
- **Backdrop Impact**: Zero mid-frame read-backs. The canvas is simply filtered in place at the conclusion of the pass.
- **Prefix Cache Impact**: The static prefix cache remains fully functional; lower layers are cached normally, dynamic elements are composited, and the final sequence filter runs at the device boundary.

### Alternative 2: Bounded Group Compositing (`type: "nest"`)

When multiple elements must be filtered together over a specific spatial region or timeline window (e.g., blurring a video background and graphic card together before laying text over them), the clean architectural pattern is **Bounded Group Compositing** via a container element:

```json
{
  "type": "nest",
  "id": "composite-group-01",
  "start": 1000,
  "end": 8000,
  "track": "graphics",
  "layer": 20,
  "width": 1280,
  "height": 720,
  "effects": [
    {"name": "blur", "radius": 15}
  ],
  "elements": [
    { "type": "video", "id": "inset-vid", ... },
    { "type": "rect", "id": "inset-border", ... }
  ]
}
```

- **Rendering Implementation**: A temporary offscreen canvas (`Canvas::layer()`) of extent $1280 \times 720$ is allocated. The enclosed child elements are painted into this buffer. The group's `effects` chain is applied to the buffer via `Canvas::through`. The finished buffer is composited into the main canvas *once* in the nest's blend mode and opacity.
- **Backdrop Impact**: Zero read-back of the parent canvas. Rendering remains strictly feed-forward.
- **ADR-0166 Alignment**: ADR-0166 established that Premiere nested sequences and CapCut compound clips share a spatial coordinate transform, and that ADR-0001 only rejected local clocks. By maintaining all child element `start`/`end` timestamps in absolute timeline milliseconds, the container enforces spatial grouping without relative timing drift.

### Alternative 3: Agentic Clip-Scoped Duplication (Paste Attributes)

Because Montagent is designed primarily for **AI agent manipulation over MCP** rather than mouse-driven GUI interaction (`CONTEXT.md:78-81`, `docs/research/inline-vs-indirection-for-agents.md`), human GUI convenience features take on different trade-offs:

- In Premiere, humans use adjustment layers because manually selecting 50 clips and pasting effects is tedious and error-prone to maintain.
- For an AI agent authoring via MCP (`replace_file_content` or domain tools), propagating an `effects` array across multiple elements in JSON is an instantaneous, exact-string operation.
- **Rendering Implementation**: Leaves the renderer 100% untouched. Each element evaluates its own effects locally.
- **Limitations**: Cannot perform cross-element spatial blurring (where pixel colors from one element bleed into another) and cannot filter a transition crossfade.

### Alternative 4: Pre-Rendered Footage Passes (`montagent-prerender`)

For complex, computationally expensive multi-element looks (e.g. multi-layered motion graphics with stacked heavy blurs and grain):
- The agent renders the sub-span via `render --from --to` into a mezzanine video asset with alpha.
- The rendered asset is re-imported into the timeline as a single `video` element with a companion recipe ([ADR-0156 §7](../../adr/0156-four-named-effects-join-the-effects-list-and-a-code-drawn-piece-enters-as-pre-rendered-footage.md)).
- This preserves Montagent's model-free determinism while isolating heavy compositing from the live editing loop.

### Alternative 5: Video Track Effects (Refused by Precedent)

Could effects be attached to a `track` object?
```json
{
  "name": "video-track-1",
  "layer": 10,
  "effects": [ {"name": "saturation", "amount": 0.5} ],
  "elements": [ ... ]
}
```
- **Precedent Audit**: **Refused**. Premiere Pro row 145 (`docs/research/premiere-gap/effects.md`) confirms: *"Video effects (fixed and standard) apply to individual clips; only audio effects can also apply to tracks."* Premiere has no concept of video track effects.
- **Model Inconsistency**: In Montagent, tracks supply stacking defaults, not pixel containment or timing bounds (`CONTEXT.md:55-61`). Adding visual filters to tracks would confuse the separation between tracks (structural stacking) and elements (renderable items).

---

## 7. Comparative Trade-off Matrix

| Criterion | Free-Floating `adjustment` Layer | Project Master Effects (`project.effects`) | Bounded Nest (`nest` + effects) | Clip-Scoped Duplication (Paste Attributes) |
|---|---|---|---|---|
| **ADR-0145 Invariants** | **Passes** (all 4) | **Passes** (all 4) | **Passes** (all 4) | **Passes** (all 4) |
| **Renderer Flow** | **Backward Read-back** (stalls pipeline) | **Single-Pass Forward** (applied at end) | **Single-Pass Forward** (subpass layer) | **Single-Pass Forward** (strictly local) |
| **Memory Bandwidth** | **Severe** (~33 MB/frame copy at 4K) | **Negligible** (in-place canvas filter) | **Bounded** (size of group box) | **Zero** additional overhead |
| **Prefix Cache ([ADR-0195](../../adr/0195-the-static-prefix-cache-holds-invariant-layers-at-the-bottom-of-the-stack.md))** | **Breaks** or fragments cache | **Preserves** full prefix cache | **Preserves** prefix outside group | **Preserves** full prefix cache |
| **Multi-Painter ($K$-Painters)** | High risk of tile edge divergence | 100% deterministic | 100% deterministic | 100% deterministic |
| **Spatial Cross-Element Filtering** | Yes (across all lower layers) | Yes (entire frame) | Yes (inside group) | No (element-isolated) |
| **Regional Scoping** | Yes (via element bounds/clip) | No (full frame only) | Yes (via nest bounds/clip) | Yes (per clip) |
| **Premiere Precedent** | Direct (`effects.md:142`) | Direct (Sequence Lumetri) | Direct (`effects.md:144`) | Direct (`effects.md:137`) |
| **Agent Usability** | High cognitive overhead (stack order) | Extremely simple (one root field) | High structural clarity | Simple exact-string replace |

---

## 8. Architectural Recommendation & Sequencing

Based on this investigation against primary sources and rendering mechanics under ADR-0145, we recommend **rejecting the free-floating `type: "adjustment"` element** and adopting a phased three-part architecture:

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ Recommended Roadmap for Multi-Element Operations                            │
├──────────────────────────────────────────────────────────────────────────────┤
│ Phase 1: Tranche A (Roadmap #978)                                            │
│ ➔ Project Master Effects ("effects": [...] on root document)                 │
│   • Instant Premiere parity for sequence-wide color grades and film looks.   │
│   • Minimal schema addition; zero renderer pipeline stalls.                 │
├──────────────────────────────────────────────────────────────────────────────┤
│ Phase 2: Agent Skill & CLI Verbs                                             │
│ ➔ Exact-String "Paste Attributes" Tooling                                    │
│   • MCP tool / skill recipe to propagate effects across selected clip IDs.   │
│   • Zero schema impact; fully preserves element-local rendering.             │
├──────────────────────────────────────────────────────────────────────────────┤
│ Phase 3: Tranche B / Capability Map (Transform & Compositing)                │
│ ➔ Bounded Group Compositing via Container ("type": "nest")                   │
│   • Enclosed elements render forward into isolated buffer.                   │
│   • Supports regional multi-element blur, localized grades, and group masks. │
│   • Strictly maintains absolute timeline millisecond timestamps (ADR-0166).  │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 8.1 Specification for Phase 1: Project Master Effects

1. **Schema Update**:  
   Extend the root project JSON schema (`schema/project.json`) to allow an optional `effects` list:
   ```json
   {
     "type": "object",
     "properties": {
       "effects": {
         "$ref": "#/$defs/effects",
         "description": "Ordered visual effects applied to the final composite of the entire frame before encoding."
       }
     }
   }
   ```
2. **Painter Execution**:  
   In `crates/montagent-core/src/verbs/frame/mod.rs`, modify `paint` to execute root effects after all stack elements finish painting:
   ```rust
   // After painting prefix and remaining stack:
   if let Some(master_effects) = self.document.master_effects() {
       canvas.apply_master_effects(master_effects);
   }
   ```
3. **Check Coverage**:  
   - Unknown effects or malformed values are reported as standard schema errors by `validate`.
   - All existing effect stopping rules ([ADR-0049](../../adr/0049-v1-colour-filter-vocabulary-four-scalar-members.md)) apply without modification.

### 8.2 Specification for Phase 3: Bounded Nest Compositing

When regional multi-element operations are scheduled under Roadmap #978:
1. Admit `type: "nest"` under the container rules clarified in ADR-0166.
2. Require `width` and `height` to establish an explicit offscreen surface bound, avoiding full-frame memory allocations where a small sub-region is needed.
3. Enforce that all child elements carry absolute timeline timestamps (`start`, `end`, and keyframe `t`), strictly upholding ADR-0001's rejection of relative clocks.
4. Render children forward into the nest's isolated buffer, execute the nest's `effects` list, and composite into the parent stack once in the nest's `blend` mode.

---

## 9. Conclusion

The demand for multi-element operations represented by Premiere Pro adjustment layers is fundamentally two distinct needs:
1. **Sequence-wide color grading and look development**: Solved cleanly, deterministically, and with zero performance penalty by **Project Master Effects**.
2. **Coordinated multi-element grouping and regional filtering**: Solved without backward backdrop read-back by **Bounded Group Compositing (`nest`)**.

By addressing these needs through forward-composited primitives rather than an open-ended backward-reading adjustment layer, Montagent achieves complete Premiere Pro capability parity while strictly safeguarding the four invariants of ADR-0145 and the performance of its single-pass streaming engine.
