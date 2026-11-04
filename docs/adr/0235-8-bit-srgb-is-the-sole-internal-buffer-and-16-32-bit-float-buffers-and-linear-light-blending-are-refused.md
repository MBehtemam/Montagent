---
status: accepted
amends: 0146 (retains 8-bit sRGB premultiplied colour blending), 0216 (re-affirms 8-bit display-referred sRGB as Montagent's universal internal compositing buffer)
---

# 8-bit sRGB is the sole internal buffer, and 16/32-bit float buffers and linear-light blending are refused

[#1107](https://github.com/MBehtemam/Montagent/issues/1107) and
[#1108](https://github.com/MBehtemam/Montagent/issues/1108), on the map
[#1080](https://github.com/MBehtemam/Montagent/issues/1080), from the census in
[#904](https://github.com/MBehtemam/Montagent/issues/904) (rows 1–12, 87–93) and
[#908](https://github.com/MBehtemam/Montagent/issues/908).

## The question

In After Effects, project settings provide three bit-depth choices (8 bpc integer, 16 bpc integer,
and 32 bpc floating-point `RGBA F32`) and a color-management toggle *Blend Colors Using 1.0 Gamma*
(linear-light compositing, where blending, anti-aliasing edges, gradients, and blurs operate in
linear photometric energy $\gamma = 1.0$ rather than non-linear display-encoded gamma).

Montagent composites exclusively on 8-bit `[u8; 4]` RGBA surfaces (`SkColorType::RGBA_8888`,
premultiplied alpha) in display-referred sRGB space ([ADR-0146](0146-an-animatable-property-is-one-the-schema-types-so-colour-blends-premultiplied-and-spring-easing-is-refused.md),
[ADR-0216](0216-srgb-is-the-working-colour-space-and-video-boundaries-convert-and-tag-bt709.md)).

Should Montagent admit 16-bit integer or 16/32-bit floating-point internal framebuffers (e.g. `RGBA F16` /
`F32`), and should a project-level switch `linear_blending: bool` enter `project.json`?

## The decision

### 1. 8-bit `[u8; 4]` sRGB remains the sole internal compositing buffer

Montagent refuses 16-bit integer (`RGBA_16161616`) and 16/32-bit float (`RGBA_F16` / `RGBA_F32`)
internal raster buffers from the engine:

1. **Byte-identical parallel rendering**: Montagent guarantees bit-identical output across 1–10
   worker painter threads ([ADR-0149](0149-a-gradient-is-a-paint-linear-or-radial-measured-against-the-declared-box.md),
   [ADR-0156](0156-four-named-effects-join-the-effects-list-and-a-code-drawn-piece-enters-as-pre-rendered-footage.md)).
   Floating-point arithmetic across different CPU instruction sets (x86-64 SSE/AVX vs ARM64 NEON) and
   compiler optimization passes introduces non-deterministic IEEE 754 rounding differences (e.g.
   fused multiply-add contraction and transcendental approximations). Integer 8-bit `u8` arithmetic
   in Skia is deterministic across all platforms.
2. **Throughput and memory bandwidth**: A 1080p frame at 8-bit RGBA consumes 8.29 MB. At `RGBA F16`,
   it consumes 33.18 MB; at `RGBA F32`, it consumes 66.36 MB. For a 4K frame, an `RGBA F32` buffer
   demands 265.4 MB per frame. Multiplying buffer allocations across parallel painter threads and
   nested sequence surfaces ([ADR-0201](0201-nested-sequences-are-bounded-containers-rendering-to-an-isolated-surface-on-an-absolute-timeline.md))
   would exhaust CPU L3 caches, collapse memory throughput, and breach latency budgets.
3. **Simplicity of pixel operations**: All format effects (blur, shadow, tint, saturation, CDL,
   LUT3D, matte, luma key, outline, inner shadow) operate directly on integer `[u8; 4]` samples with
   fast integer SIMD vectorization.

### 2. Linear-light blending is refused

A project-level `linear_blending: bool` toggle (or *Blend Colors Using 1.0 Gamma*) is refused:

1. **Two incompatible render modes**: In After Effects, toggling linear blending fundamentally alters
   every semi-transparent pixel, edge anti-aliasing gradient, feather mask, and blur falloff in the
   composition. Having two global compositing modes fractures the format's predictable rendering
   guarantees.
2. **Round-trip conversion overhead**: Linear compositing requires converting 8-bit non-linear sRGB
   inputs to linear floating-point values before every paint, blend, filter, and compositing pass,
   and converting back to sRGB display gamma with dithering. This adds severe compute overhead
   with negligible aesthetic gain for standard motion design and typography.
3. **Precedent**: CapCut and Premiere default timelines operate directly in display gamma space
   ([ADR-0145](0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md)).

### 3. What to do instead

1. **Precision grading**: For footage requiring heavy colour manipulation or wide-gamut transformations,
   grade upstream in a dedicated color grading tool (such as DaVinci Resolve) and export to
   ProRes 4444 or DNxHR HQX with standard Rec.709 tags ([ADR-0216](0216-srgb-is-the-working-colour-space-and-video-boundaries-convert-and-tag-bt709.md)).
2. **3D LUTs**: Apply pre-baked `.cube` 3D LUTs via `lut3d` ([ADR-0206](0206-asc-cdl-and-3d-lut-tetrahedral-shaders-grade-colour-under-closed-arithmetic.md))
   for cinematic look transforms.
