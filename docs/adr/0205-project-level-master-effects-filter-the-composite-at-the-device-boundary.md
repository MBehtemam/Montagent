---
status: accepted
amends: 0040 (effect model), 0145 (four invariants), 0195 (static prefix cache)
---

# Project-level master effects filter the composite at the device boundary

[#981](https://github.com/MBehtemam/Montagent/issues/981), [#1018](https://github.com/MBehtemam/Montagent/issues/1018), [#1019](https://github.com/MBehtemam/Montagent/issues/1019).

## Context

In Premiere Pro, editors frequently place an adjustment layer on the topmost video track across the entire timeline to apply a sequence-wide look—such as unified color grading, film grain, or master contrast (`docs/research/premiere-gap/effects.md:142`, `docs/research/premiere-parity/multi-element-adjustment.md`).

In Montagent, modeling multi-element filtering via a free-floating `type: "adjustment"` element was investigated in [#981](https://github.com/MBehtemam/Montagent/issues/981). While an adjustment element syntactically satisfies the four invariants of [ADR-0145](0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md), it severely degrades engine performance:
1. It requires unbounded backward backdrop snapshotting (`surface.image_snapshot()`), incurring a ~33 MB/frame memory copy penalty at 4K and creating synchronization barriers across parallel $K$-painters.
2. It breaks the **Static Prefix Cache** ([ADR-0195](0195-the-static-prefix-cache-holds-invariant-layers-at-the-bottom-of-the-stack.md), [ADR-0198](0198-an-element-that-draws-nothing-neither-joins-nor-ends-the-static-prefix.md)), forcing continuous invalidation of invariant bottom layers.
3. It violates `CONTEXT.md`'s foundational axiom: *"An effect filters the element's own pixels: there is no element that paints from nothing."*

Montagent requires a first-class sequence-level grading mechanism that satisfies the four invariants and delivers Premiere parity without mid-frame backdrop read-back penalties.

## Decision

1. **Project Root `effects` Array**:  
   The project root schema admits an optional `effects: [...]` array:
   ```json
   {
     "width": 1920,
     "height": 1080,
     "fps": 30,
     "duration": 60000,
     "background": "#000000",
     "effects": [
       { "name": "contrast", "amount": 0.08 },
       { "name": "saturation", "amount": 0.95 },
       { "name": "grain", "amount": 0.12, "seed": 42 }
     ],
     "tracks": [ ... ]
   }
   ```

2. **Closed Vocabulary & Static Validation**:  
   Every entry in `project.effects` must be a valid member of the closed `VisualEffect` vocabulary (e.g. `blur`, `color_filter`, `cdl`, `lut3d`, `grain`, `sharpen`).  
   - Any unsupported or malformed effect is refused statically by `validate` with `E-EFFECT-UNKNOWN` or `E-EFFECT-PARAM-INVALID`.
   - Element-specific parameters (such as motion paths or keyframed geometry) are forbidden at project level. All project-level effect parameters are literal numbers or strings.

3. **Execution Sequencing at the Device Boundary**:  
   Project-level master effects are executed **exactly once per frame at the device boundary**:
   - The frame painter composites the timeline stack onto the Skia canvas as normal.
   - The Static Prefix Cache ([ADR-0195](0195-the-static-prefix-cache-holds-invariant-layers-at-the-bottom-of-the-stack.md)) remains 100% active; invariant lower layers are cached without invalidation.
   - When all tracks and elements for the current instant have composited into the main canvas buffer, the master effect chain is evaluated across the canvas surface.
   - The resulting pixels are fed directly to the raw pipe transport (`canvas.rgb()` for standard export, `canvas.rgba()` for ProRes 4444 with alpha, or snapshot encoder for `frame`).

4. **Alpha Channel & Background Handling**:  
   If the project declares a transparent background (`#00000000`) and the deliverable is ProRes 4444 with alpha ([ADR-0203](0203-prores-and-dnxhr-mezzanine-codecs-render-via-lgpl-encoders-and-alpha-flows-through-a-32-bit-pipe.md)), master color effects operate on premultiplied alpha or apply color transformations preserving the source alpha channel. Unpainted transparent regions remain strictly zero alpha.

5. **Diagnostic Introspection**:  
   Project-level effects are reported in `montagent query` and inspected in `montagent measure`.
