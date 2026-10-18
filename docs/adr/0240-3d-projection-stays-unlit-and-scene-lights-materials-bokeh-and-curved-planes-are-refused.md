# 240. 3D projection stays unlit, and scene lights, materials, bokeh, and curved planes are refused

Date: 2026-10-18

## Status

Accepted

## Context

After Effects features a classic 3D compositing system that includes:
1. **Per-element shading**: Automatically darkening or highlighting layers as they tilt away from virtual light sources.
2. **Scene-wide 3D lights and Material Options**: Parallel, Spot, Point, and Ambient light layers that illuminate 3D layers in the composition, coupled with Material Options (diffuse, specular, shininess, metal, casts shadows).
3. **Camera Lens Blur**: Optical bokeh simulation with configurable polygonal iris blades, aperture roundness, and highlight blooming.
4. **Curved planes and cylindrical geometry**: *Curvature* geometry options and *CC Cylinder* / *CC Sphere* effects that warp flat 2D layers into cylindrical or spherical meshes.

Montagent's perspective projection model was established in `ADR-0167`: "An element may be projected, never placed". Under `ADR-0167`:
- An element may specify `swivel`, `tilt`, and `perspective` to produce a 2.5D projective transform onto Skia's 2D canvas.
- There is no `z` coordinate, no 3D camera entity, no z-depth buffer, no depth sorting, no 3D layer intersections, and no inter-element cast shadows.
- Precedent in modern short-form editing tools (CapCut and Premiere, `ADR-0145`) implements 3D tilt as a pure geometric planar projection without a full 3D rendering pipeline.
- Frame painting across 1–10 parallel CPU workers requires fast, deterministic rendering (16ms/frame budget, `ADR-0149`).

We must establish whether lighting, materials, optical bokeh, or curved geometry enter the format or are refused.

## Decision

1. **Perspective projection stays unlit, and per-element directional shading is refused (#1117):**
   - When an element turns in 3D perspective via `swivel` or `tilt`, it renders unlit (pure geometric affine/projective transformation).
   - An automatic 3D shading or normal-based lighting model is refused. Modern UI motion design (e.g. card flips, rotating badges) that requires lighting falloff or specular sheen accomplishes this deterministically by keyframing `effects: [{"name": "tint", ...}]`, `brightness`, or by applying a gradient overlay in `montagent-motion`. This keeps element rendering completely self-contained and avoids phantom light sources.

2. **Scene-wide 3D light entities and Material Options are refused (#1118):**
   - Light entities (`light` track type or element) and material options (`specular`, `diffuse`, `shininess`, `casts_shadows`) are refused under `ADR-0145` (closed vocabulary and element self-containment) and `ADR-0167`.
   - Introducing scene lights breaks element self-containment: rendering an element on track A would require querying lights scattered across tracks B, C, and D.
   - True 3D lighting, multiple bounces, and cast shadows belong to dedicated 3D engines. External 3D scenes enter Montagent exclusively as pre-rendered transparent footage via `montagent-prerender` (`ADR-0233`).

3. **Optical bokeh blur (`camera_lens_blur`) is refused (#1119):**
   - Aperture-shaped polygonal/circular bokeh convolution is computationally prohibitive on the CPU, violating the 16ms/frame render latency target across parallel painters (`ADR-0149`).
   - `camera_lens_blur` is refused from `effects[]`.
   - Separable Gaussian blur (`ADR-0040`) and directional/radial blurs remain the sole blur primitives in `effects[]`. Optical lens bokeh effects must be pre-rendered into footage if needed.

4. **Curved planes (*Curvature*, *CC Cylinder*) are refused from core render (#1120):**
   - Skia is a 2D planar canvas. Bending a planar element into a cylindrical ribbon or sphere requires a 3D polygonal triangle-mesh rasterizer, texture mapping, and depth interpolation.
   - Non-planar 3D geometry is refused from core render crates (`montagent-core`, `montagent-render`).
   - Cylindrical ribbons, spinning 3D globes, and curved card carousels are authored via headless 3D recipes in `skills/montagent-prerender/` (e.g. Three.js / Blender scripts exporting transparent ProRes 4444 `.mov` files under `ADR-0203` and `ADR-0233`).

## Consequences

### Positive
- The rendering engine remains lean, fast, and 100% planar 2D vector/bitmap math via Skia.
- Zero 3D light solvers, shadow map buffers, normal maps, or mesh rasterizers in `montagent-render`.
- Strict element self-containment is preserved: every element renders solely from its own properties and parent nest transforms without cross-element spatial coupling.
- Full compatibility with the 16ms parallel CPU painter budget.

### Negative / Trade-offs
- Subtle light falloff on rotating 2.5D cards must be explicitly animated using keyframed color/tint effects or gradient overlays rather than being automatic.
- 3D cylindrical ribbons and rotating globes cannot be generated natively in `project.json` and require an offline pre-render step.
