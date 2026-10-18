---
status: accepted
amends: 0059 (transitions bridge two visual elements across their temporal intersection), 0150 (wipe, slide, and push enter as transition kinds)
---

# 243. Native transitions between `nest` scenes render via offscreen surfaces

Date: 2026-10-18

## Context

In Montagent, transitions bridge two visual elements across their shared temporal intersection ([ADR-0059](0059-a-transition-bridges-two-elements-across-their-overlap.md), [ADR-0150](0150-wipe-slide-and-push-enter-as-transition-kinds-and-a-matte-from-another-element-is-refused.md)).
Transitions operate by computing integer pixel offsets (`pixels`) or clipping apertures (`cut`) applied to individual elements before painting.

When authors structure long-form videos (such as product sizzles or explainers) into scenes, each scene is modeled as a `nest` element referencing a child project file. In Montagent v0.4.x, bridging two `nest` elements with a native `transition` was unsupported:
1. A `nest` element contains an entire multi-track composition with its own background, layers, text, and effects.
2. Applying an opacity crossfade to individual elements inside a nested scene causes intra-scene bleed (underlying layers become visible through semi-transparent foreground elements).
3. Applying push, slide, or wipe geometric offsets directly to child element transforms within a nest breaks background framing and clip boundaries.

During production of the v0.4.0 sizzle, editors had to manually pre-render each scene to intermediate ProRes 422 files and stitch them as `video` elements on staggered tracks. This incurred a second render pass, ~58 MB of disk storage per 6-second scene, and required manual re-renders whenever a scene was tweaked.

## Decision

1. **`nest` elements are admitted as valid `from` and `to` targets for all visual transitions**:
   - `transition.from` and `transition.to` may name elements of `type: "nest"`.
   - The validation rule requiring bridged elements to be flat visual primitives (`video`, `image`, `rect`, `path`) is relaxed to include `nest`.

2. **Offscreen surface rendering during transition windows**:
   - Outside of a transition's active window `[start, end)`, a `nest` element continues to render directly into the main canvas raster stream with zero overhead.
   - During an active transition window, the render engine allocates two offscreen Skia surfaces (`Surface A` and `Surface B`) at the project's native dimensions:
     - `nest_from` is rendered completely into `Surface A`.
     - `nest_to` is rendered completely into `Surface B`.
   - The transition (`crossfade`, `wipe`, `slide`, `push`, `iris`, `blinds`) then composites `Surface A` and `Surface B` onto the main canvas as atomic texture quads.
   - Because `Surface A` and `Surface B` represent fully composited scenes:
     - Crossfades fade the whole scene as a group without intra-scene transparency leaks.
     - Pushes and slides translate the entire scene frame without tearing nested element coordinates.
     - Wipes and irises clip the composited scenes cleanly along integer pixel boundaries.

3. **Audio bridging across `nest` elements**:
   - Audio tracks inside `nest_from` and `nest_to` mix into their respective project root stems.
   - The transition's `audio` property (`cut`, `constant_power`, `constant_gain`) governs the fade curves applied to the scene audio stems across `[start, end)`.

4. **Engine scheduling & target milestone**:
   - To maintain v0.4.1 as an immediate stability, documentation, and papercut release, this engine feature is scheduled for **Montagent v0.5.0**.
   - In v0.4.1, the multi-scene ProRes handover recipe is documented in `skills/montagent-motion/SKILL.md` and `crates/montagent-core/docs/format/nest.md`.

## Consequences

### Positive
- Multi-scene video projects can use native cuts, pushes, wipes, and slides between scenes directly in `project.json` without external stitching or manual pre-rendering passes.
- Render turnaround for iterative edits is significantly accelerated by eliminating intermediate file exports.
- Group opacity and geometric clipping semantics match professional NLEs (After Effects pre-comps, Premiere nested sequences, CapCut compound clips).

### Negative / Trade-offs
- During transition windows, allocating two full-resolution offscreen surfaces increases peak GPU/RAM memory footprint by $2 \times (\text{width} \times \text{height} \times 4\text{ bytes})$ (e.g., ~16.6 MB for 1080p). Surfaces will be pooled across frames to prevent allocation churn.
