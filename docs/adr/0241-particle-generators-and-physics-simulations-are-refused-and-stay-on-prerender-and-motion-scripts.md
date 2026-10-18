# 241. Particle generators and physics simulations are refused and stay on pre-render and motion scripts

Date: 2026-10-18

## Status

Accepted

## Context

After Effects includes 18 specialized *Simulation* effects:
1. **Analytic Overlays**: CC Rainfall, CC Snowfall, CC Star Burst, generating hundreds or thousands of falling raindrops, snowflakes, or flying starfield specks.
2. **Stateful Physics Simulations**: CC Particle World, CC Particle Systems II, Particle Playground, Foam (bubble dynamics), Wave World (water surface simulation), Caustics (refracted light physics), and Shatter (rigid-body mesh destruction).
3. **Card-Grid Choreography**: Card Dance and Card Wipe, slicing a layer into a geometric grid of 3D cards that flip and rotate according to control maps.

Montagent's architecture relies on critical core guarantees:
- `ADR-0145`: Closed minimal vocabulary, editable by exact string replace, literal values. A full-featured particle system requires dozens of sprawling parameters (birth rate, velocity, spread, turbulence, longevity, air resistance, bounce, color over life) that would bloat the schema and complicate LLM generation.
- `ADR-0149`: Ten parallel painters render frames independently, and every frame is a pure function of the clock `t`. A painter rendering frame 120 must compute its pixels instantly without sequentially stepping and accumulating physics state across frames 0 through 119.
- `ADR-0156` §7 and `ADR-0171`: Visual looks outside the closed effects list enter as pre-rendered footage with reproducible recipes.
- `ADR-0167`: Elements support per-element perspective projection (`swivel`, `tilt`, `perspective`).
- `ADR-0177`: Motion synthesis belongs in authoring scripts that emit literal keyframes into `project.json`.

We must establish whether simulation effects enter the format or are served by authoring scripts and `montagent-prerender`.

## Decision

1. **In-engine particle generators (Rainfall, Snowfall, Star Burst) are refused from `effects[]` (#1121):**
   - Built-in particle emitter effects are refused from the schema.
   - **Dense environmental overlays** (rain, snow, dust motes, starbursts) are authored via `skills/montagent-prerender/` recipes (e.g. Python Skia scripts or canvas players with fixed seeds) that export transparent straight-alpha `prores_4444` footage (`ADR-0203`), placed as standard `video` elements.
   - **Sparse celebratory bursts** (confetti showers, coin bursts, floating spark particles with 10–50 pieces) are synthesized directly into `project.json` by authoring scripts in `skills/montagent-motion/`, emitting standard `rect` or `path` elements with literal keyframes for `x`, `y`, `rotation`, and `opacity`.

2. **Stateful physics simulations (Particle World, Foam, Caustics, Shatter) are refused from Montagent's format and render engine (#1122):**
   - Emitters and solvers requiring iterative time-stepping ($S(t) = S(t - \Delta t) + \dots$) fundamentally violate `ADR-0149`'s frame-stateless parallel painter invariant and break fast random-access tile rendering (`frame --at`).
   - Rigid-body mesh destruction (Shatter), fluid dynamics (Foam, Wave World), and optical raytracing (Caustics) require dedicated simulation physics runtimes that are entirely outside Montagent's lightweight 2D compositor.
   - All stateful simulations remain strictly in `montagent-prerender` (ADR-0156 §7, ADR-0233), where headless simulation engines (Three.js, Blender, or specialized Python solvers) bake the output into transparent ProRes 4444 clips with reproducible recipe hashes.

3. **Grid-card choreography (Card Dance / Card Wipe) enters as an authoring motion pattern emitting native elements (#1123):**
   - Card Dance is refused as a monolithic engine effect.
   - Slicing an element into a grid of 3D-tilting cards is accomplished natively using Montagent's existing primitive vocabulary: an authoring script slices the graphic into a grid of `image` or `nest` elements, placing them with staggered `swivel` and `tilt` keyframes (`ADR-0167`, `ADR-0177`).
   - For ultra-dense grids (e.g. 100×100 = 10,000 cards) where element counts would degrade JSON readability or Skia DOM traversal, the animation is pre-rendered via `montagent-prerender`.

## Consequences

### Positive
- `montagent-render` retains zero physics simulation runtimes, collision detection loops, or temporal emitter caches.
- Pure random-access frame rendering (`frame --at`) remains instantaneous and byte-identical across 1–10 parallel painter threads.
- `project.json` remains clean and free of hundreds of simulation tuning dials.
- Motion designers can use any modern physics or particle library (P5.js, Three.js, Blender geometry nodes) through `montagent-prerender` without any engine restrictions.

### Negative / Trade-offs
- Environmental snow, rain, and heavy particle fields cannot be generated on the fly via a single property in `project.json` and must be imported as pre-rendered alpha video clips.
