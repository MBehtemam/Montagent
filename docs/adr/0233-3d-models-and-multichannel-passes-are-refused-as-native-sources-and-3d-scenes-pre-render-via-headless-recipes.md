---
status: accepted
amends: 0040 (reaffirms 3D spatial scene graph, depth sorting and cameras are refused), 0167 (confirms 3D model container formats are refused as native sources and projection is limited to flat 2D planes), 0171 (extends native source format refusal to 3D model containers .glb/.gltf/.obj/.fbx/.c4d/.blend/.usd and multichannel passes), 0156 (routes 3D scenes and multichannel passes to pre-rendered ProRes 4444 with alpha on montagent-prerender), 0203 (affirms ProRes 4444 with alpha as the delivery format for pre-rendered 3D scenes), 0215 (disposes of After Effects 3D model import and multichannel pass census rows from map #896)
---

# 3D models and multichannel passes are refused as native sources, and 3D scenes pre-render via headless recipes

[Are 3D model files and multichannel 3D passes refused as native sources, and what is the montagent-prerender 3D-scene recipe?](https://github.com/MBehtemam/Montagent/issues/1075)
on the After Effects map ([#896](https://github.com/MBehtemam/Montagent/issues/896)).

[ADR-0040](0040-effect-model-attachment-and-v1-vocabulary.md) refused "3D / perspective distortion" on model grounds because the format's spatial canvas is 2D.
[ADR-0167](0167-an-element-may-be-projected-never-placed-swivel-tilt-and-perspective-join-the-transform.md) established the core invariant that an element may be *projected, never placed*: swivel, tilt, and perspective act on flat 2D layers, while a 3D camera, spatial scene graph, depth sorting, and 3D layer intersections remain strictly refused.
[ADR-0171](0171-svg-and-lottie-do-not-enter-as-sources-and-each-is-pre-rendered-through-the-skill.md) established that complex external vector formats (SVG, Lottie) do not enter as native sources due to heavyweight external runtimes, lack of deterministic CPU guarantees across platforms, and clock reconciliation complexities, routing them to `montagent-prerender`.
[ADR-0156](0156-four-named-effects-join-the-effects-list-and-a-code-drawn-piece-enters-as-pre-rendered-footage.md) §7 and [ADR-0203](0203-prores-and-dnxhr-mezzanine-codecs-render-via-lgpl-encoders-and-alpha-flows-through-a-32-bit-pipe.md) established that generative and code-drawn visuals enter Montagent as pre-rendered footage with straight alpha in Apple ProRes 4444 or PNG-in-MOV.
[ADR-0215](0215-after-effects-is-precedent-for-rendered-results-and-capcut-and-premiere-stay-first-for-shape.md) admitted After Effects precedent for visual results while keeping Premiere and CapCut first for mechanism and shape.

The After Effects censuses ([#902](https://github.com/MBehtemam/Montagent/issues/902) rows 33–45 and [#907](https://github.com/MBehtemam/Montagent/issues/907) rows 18–19) noted that After Effects natively imports 3D model files (`.glb`, `.gltf`, `.obj`, `.fbx`, `.c4d`) with Advanced 3D / Mercury 3D render engines, and compositing workflows frequently sample multichannel 3D render passes (depth `Z`, world normals, position, Cryptomatte, motion vectors).

This ADR decides:
1. **3D model files and containers are refused as native element sources**: Files in `.glb`, `.gltf`, `.obj`, `.fbx`, `.c4d`, `.blend`, `.usd`, and `.usdz` are refused by schema and runtime painters.
2. **Multichannel 3D render passes are refused from native composition**: Non-RGBA 3D auxiliary buffers (depth `Z`, camera-space normals, Cryptomatte, UV passes) are refused from native `effects[]` and layer pipelines.
3. **3D scenes enter strictly as pre-rendered footage via `montagent-prerender`**: Deterministic headless 3D render recipes (Blender CLI or headless Three.js) render transparent RGBA video (ProRes 4444 with alpha or PNG-in-MOV) or still PNGs outside the project render, preserving byte-identical painter execution.

---

## Decisions

### 1. 3D model containers are refused as native element sources

No Montagent element accepts a 3D model file (`.glb`, `.gltf`, `.obj`, `.fbx`, `.c4d`, `.blend`, `.usd`, `.usdz`) as its `source`.

The grounds stand on the core invariants of the format:
- **No 3D scene engine or camera**: A 3D model has no intrinsic 2D appearance without a camera viewpoint, field of view, lighting setup, environment map, material shader, and rasterizer. Introducing a 3D model loader would require embedding an entire 3D scenegraph engine (e.g. Filament, wgpu, or Three.js) into `montagent-render`.
- **Model contradiction with ADR-0167**: ADR-0167's central thesis is that an element may be *projected, never placed*. Objects have no $z$-depth coordinate, do not cast spatial shadows on other layers, and cannot intersect in depth. Native 3D models require inter-object depth sorting, violating this boundary.
- **Cross-platform determinism**: Real-time GPU rendering and PBR shaders (Cook-Torrance, metallic/roughness, anisotropic filtering, raster precision across Apple Silicon and NVIDIA/AMD/Intel GPUs) cannot guarantee byte-identical rendered frames across platforms.
- **Reference-class alignment**: Neither CapCut nor Premiere natively embeds interactive arbitrary 3D model files as raw timeline clip elements without conversion or pre-rendering. Under ADR-0145 and ADR-0215, AE's recent 3D model engine does not override the fundamental 2D transform model.

### 2. Multichannel 3D render passes are refused from native composition

Multichannel 3D scene passes—such as 32-bit float OpenEXR depth (`Z`), world/camera normals, object/material IDs, Cryptomatte, motion vectors, and position passes—are refused from the native element schema and runtime `effects[]`.

- **Single-instant 2D composite**: Montagent's compositor operates strictly on 8-bit sRGB / Rec.709 display-referred RGBA buffers (ADR-0216).
- **No camera-space shaders**: Depth-of-field shaders driven by depth passes (*Lens Blur*), relighting shaders driven by normal passes, and matte isolation driven by Cryptomatte require full 3D spatial metadata and camera projection matrices. These composite steps belong upstream in the 3D rendering pipeline, not inside the 2D video timeline.

### 3. 3D scenes enter via `montagent-prerender`

3D models and animated 3D scenes enter Montagent through `skills/montagent-prerender/SKILL.md` as pre-rendered footage:
- **Deliverable**: Apple ProRes 4444 (`.mov`) with straight alpha (`alpha: true` per ADR-0203 and ADR-0229) for animated sequences, or PNG-in-MOV for internal caching, or a single PNG still for static 3D product renders.
- **Execution**: Rendered completely outside `montagent-render` using deterministic headless command-line pipelines (Blender CLI or headless Three.js / Puppeteer).
- **Placement**: Placed into `project.json` as an ordinary `video` or `image` element, fully compositable with native 2D transforms (`x`, `y`, `scale`, `rotation`, `swivel`, `tilt`), blend modes, and 2D `effects[]`.

---

## The `montagent-prerender` 3D-scene recipe contract

A new **3D scene route** is specified in `skills/montagent-prerender/SKILL.md`:

1. **Headless toolchain**:
   - **Blender CLI**: `blender -b <scene.blend> -P <script.py> -o <out_dir>/frame_######.png -s <start> -e <end> -a`
   - Fixed rendering engine (`CYCLES` with pinned CPU/GPU device and fixed sampling seed, or deterministic `BLENDER_EEVEE_NEXT`).
   - Pinned resolution, frame rate, and duration matching the Montagent project target.
   - RGBA 8-bit or 16-bit PNG frame export with transparent background (`render.film_transparent = True`).
2. **Deterministic encoding**:
   - Frames are encoded into transparent Apple ProRes 4444 MOV (`-c:v prores_ks -profile:v 4 -pix_fmt yuva444p10le`) or PNG-in-MOV (`-c:v png -pix_fmt rgba`) via `python3 scripts/prerender.py build <spec>`.
3. **Recipe artifact**:
   - `<name>.recipe/` stores the build spec, the 3D asset/script, pinned toolchain versions (`blender --version`), render arguments, and frame hashes.

---

## Consequences

- Schema and runtime painters remain entirely free of 3D scenegraph runtimes, PBR engines, and 3D mesh loaders.
- Any 3D content in a Montagent video is deterministic, byte-identical across painters, and decoded as standard high-performance video frames.
- Census rows for 3D model import and multichannel passes on map #896 are fully resolved.
