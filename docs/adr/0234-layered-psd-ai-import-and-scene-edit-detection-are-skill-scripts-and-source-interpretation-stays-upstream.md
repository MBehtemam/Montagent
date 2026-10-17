---
status: accepted
amends: 0002 (confirms inline source model and no per-element interpretation table), 0017 (confirms closed schema against proprietary PSD/AI document formats and runtime detectors), 0143 (confirms no decoder knobs; container metadata from ffprobe is trusted), 0145 (scene cuts are written as literal video elements, not runtime detectors), 0171 (extends native source format refusal to proprietary layered PSD and AI files), 0177 (extends skill-script pattern to layered PSD unpack and scene detection scripts), 0201 (unpacked PSD layers map to child elements inside a bounded nest container), 0215 (disposes of the import, source interpretation, and scene-edit detection census rows from map #896)
---

# Layered PSD and AI import and scene-edit detection are skill scripts, and source interpretation stays upstream

[How do layered PSD/AI import, source interpretation overrides and scene-edit detection enter?](https://github.com/MBehtemam/Montagent/issues/1076)
on the After Effects map ([#896](https://github.com/MBehtemam/Montagent/issues/896)).

[Census: AE import, render, export, preview and interchange](https://github.com/MBehtemam/Montagent/issues/907)
(rows 9, 10, 12–16, 20–26, 28) identified three final import, ingest, and footage interpretation capabilities from After Effects:
1. **Layered PSD and Illustrator (`.psd`, `.ai`) import**: unpacking multi-layer design documents into positioned timeline elements with preserved layer names, opacity, coordinates, and blend modes.
2. **Source interpretation overrides (*Interpret Footage*)**: per-element container overrides for alpha channel interpretation (straight vs premultiplied vs ignore), frame rate conforming, deinterlacing / field separation, and non-square pixel aspect ratio (PAR).
3. **Scene-edit detection**: automated temporal analysis of continuous footage to detect cut points and split the source into distinct shots.

[ADR-0017](0017-closed-schema-no-escape-hatch.md) established the closed schema invariant.
[ADR-0171](0171-svg-and-lottie-do-not-enter-as-sources-and-each-is-pre-rendered-through-the-skill.md) established that complex external multi-entity and vector file formats (SVG, Lottie) do not enter as native element sources, routing them to pre-processing tools.
[ADR-0177](0177-ducking-is-a-skill-script-that-writes-ordinary-volume-keyframes-and-the-format-stays-untouched.md) and [ADR-0217](0217-motion-bakes-and-keyframe-assistants-enter-in-one-keyframes-script-auto-colour-is-refused-and-a-skill-script-with-its-spec-is-a-template.md) established that authoring intelligence and media analysis enter Montagent as standalone skill scripts that emit standard literal elements and keyframes into `project.json` without touching the runtime engine.
[ADR-0201](0201-nested-sequences-are-bounded-containers-rendering-to-an-isolated-surface-on-an-absolute-timeline.md) established the `nest` element as an isolated, bounded composition canvas on an absolute timeline.
[ADR-0215](0215-after-effects-is-precedent-for-rendered-results-and-capcut-and-premiere-stay-first-for-shape.md) admitted After Effects precedent for visual results while retaining the four invariants of [ADR-0145](0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md).
[ADR-0221](0221-vibrance-hue-rotate-and-invert-enter-effects-and-twelve-skblendmode-variants-enter-blend.md) expanded `blend` to include Photoshop/AE blend modes (`multiply`, `screen`, `overlay`, `darken`, `lighten`, `color_dodge`, `color_burn`, `hard_light`, `soft_light`, `difference`, `exclusion`).

This ADR resolves all three capabilities:
1. **Layered PSD and AI files are refused as native sources**: they enter Montagent strictly via an external unpack-and-place skill script workflow that extracts cropped per-layer PNGs and generates literal `image` elements inside a `nest` (or on tracks) with exact canvas coordinates and mapped `blend` modes.
2. **In-schema source interpretation overrides are refused**: `montagent-core` continues to trust `ffprobe` container metadata directly. Non-square pixels, interlacing/fields, and frame rate conforming are normalized upstream via FFmpeg before placement. Alpha is treated as straight RGBA per ADR-0203 and ADR-0229, and matte extraction uses `effects[]` (`matte`, `luma_key`, ADR-0227).
3. **Scene-edit detection enters as an authoring skill script**: a script runs FFmpeg's `scdet` filter to find cut timestamps and writes contiguous sliced literal `video` elements (`start`, `end`, `source_start`) into `project.json`. No runtime scene detector enters the schema or render engine.

---

## Decisions

### 1. Layered PSD and AI files are refused as native sources; unpacking layers to cropped PNGs inside a `nest` is an authoring skill workflow

No Montagent element accepts a `.psd` or `.ai` file as its `source`.

1. **Format invariants and engine simplicity ([ADR-0017](0017-closed-schema-no-escape-hatch.md), [ADR-0171](0171-svg-and-lottie-do-not-enter-as-sources-and-each-is-pre-rendered-through-the-skill.md)):**
   Photoshop Document (`.psd`) and Adobe Illustrator (`.ai`, a PDF/PGF hybrid) are complex, proprietary binary formats containing private layer records, adjustment layers, text engines, clipping paths, smart filters, and vector masks. Embedding native PSD or AI decoders inside `montagent-render` would introduce enormous third-party dependencies, unpredictable cross-platform rendering differences, and high maintenance overhead, violating the byte-identical rendering invariant.
2. **Unpack-and-place skill workflow:**
   Ingesting a layered PSD or AI document is an authoring-time conversion task. A skill workflow (using Python tools such as `psd-tools` or headless CLI tools):
   - Reads the document hierarchy and canvas dimensions (`width`, `height`).
   - Exports each visible raster layer as a cropped transparent 8-bit RGBA PNG, recording its offset `(x, y)` relative to the top-left of the canvas.
   - Maps each layer's opacity and blend mode to Montagent's native properties: opacity $\to$ `opacity`, and blend modes $\to$ `blend` (using the 16 modes admitted across [ADR-0147](0147-four-blend-modes-enter-and-the-rest-wait.md) and [ADR-0221](0221-vibrance-hue-rotate-and-invert-enter-effects-and-twelve-skblendmode-variants-enter-blend.md)).
   - Emits a sequence of literal `image` elements, wrapped in a `nest` element ([ADR-0201](0201-nested-sequences-are-bounded-containers-rendering-to-an-isolated-surface-on-an-absolute-timeline.md)) whose dimensions match the PSD/AI artboard.
   - Vector layers in AI files that cannot be converted to simple Skia paths without visual loss are rasterized to per-layer PNGs or pre-rendered through `montagent-prerender` ([ADR-0171](0171-svg-and-lottie-do-not-enter-as-sources-and-each-is-pre-rendered-through-the-skill.md)).

### 2. In-schema source interpretation overrides (*Interpret Footage*) are refused; `montagent-core` trusts container metadata and normalizes upstream

Montagent does not add an `interpret` block or per-element override flags (`alpha_mode`, `deinterlace`, `par`, `assume_fps`) to `video` or `image` elements.

1. **Trust container metadata ([ADR-0002](0002-inline-source-no-asset-table.md), [ADR-0143](0143-the-output-file-is-an-mp4-and-the-encoder-has-no-knobs.md)):**
   `montagent-core` inspects media files through standard `ffprobe` / image decoder metadata. Video duration, dimensions, pixel aspect ratio, and frame rate are derived directly from the container. Adding decoder tuning knobs to `project.json` duplicates FFmpeg's ingest pipeline and turns the declarative project file into an encoder/decoder configuration manifest.
2. **Normalize upstream via FFmpeg:**
   - **Interlaced footage**: Montagent's compositor is progressive-only. Interlacing artifacts must not be resolved by running expensive deinterlacing filters (`bwdif`/`yadif`) on every frame decode inside the renderer. Interlaced media (e.g. 1080i50, 1080i59.94) must be deinterlaced upstream into progressive video (`ffmpeg -i input.mov -vf bwdif -c:v libx264 -crf 17 output.mp4`) before placement.
   - **Non-square pixel aspect ratio (PAR)**: Montagent's coordinate system is strictly square pixels ($1:1$). Anamorphic or legacy broadcast media (e.g. DVCPRO HD, PAL/NTSC DVD) is conformed to square pixels upstream (`scale=iw*sar:ih`).
   - **Frame rate conforming**: Footage with missing or non-standard container frame rates is remuxed or transcoded with explicit `-r` before placement.
   - **Alpha interpretation**: Alpha channels in Apple ProRes 4444 and PNG are decoded as straight RGBA ([ADR-0203](0203-prores-and-dnxhr-mezzanine-codecs-render-via-lgpl-encoders-and-alpha-flows-through-a-32-bit-pipe.md), [ADR-0229](0229-image-sequence-export-and-png-in-mov-are-refused-from-render-and-prores-4444-is-the-sole-alpha-deliverable.md)). For footage requiring alpha cleanup or matte extraction from color or luminance, the author uses native `effects[]`: `chroma` ([ADR-0040](0040-effect-model-attachment-and-v1-vocabulary.md)), `matte` choke/feather ([ADR-0227](0227-standalone-matte-and-luma-key-enter-effects-and-iterative-chokers-and-roughen-edges-are-refused.md)), or `luma_key` ([ADR-0227](0227-standalone-matte-and-luma-key-enter-effects-and-iterative-chokers-and-roughen-edges-are-refused.md)), rather than an in-container alpha override.

### 3. Scene-edit detection enters as an authoring skill script emitting literal cuts

Scene-edit detection enters Montagent as an authoring skill script in `montagent-footage`:

1. **Literal cuts on the timeline ([ADR-0145](0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md), [ADR-0177](0177-ducking-is-a-skill-script-that-writes-ordinary-volume-keyframes-and-the-format-stays-untouched.md)):**
   Montagent does not include a dynamic scene-cut detector in `project.json` or `montagent-render`. Instead, a script runs outside the render loop, analyzes a source video, and produces standard literal `video` elements with calculated `start`, `end`, and `source_start` timestamps.
2. **Script contract (`skills/montagent-footage/scripts/scene_detect.py`):**
   - Invocation: `python3 scripts/scene_detect.py <project> <source_path> [--threshold 0.4] [--min-scene-ms 500]`
   - Execution: Invokes FFmpeg with the `scdet` / `select='gt(scene,0.4)'` filter to locate cut timestamps across the source video.
   - Output: Generates contiguous `video` elements spanning each shot, validates the result with `montagent validate`, and writes the updated project to stdout.
   - The resulting timeline elements are ordinary editable clips that can be reordered, trimmed, graded, or transitioned individually with zero runtime overhead.

---

## Evidence: three-juror court

A three-juror court (`pro`, `flash`, `inherit`) was convened under `.agents/skills/court/SKILL.md` to evaluate all three questions. The ballots were returned verbatim:

```
🗳️ **Juror 1** (pro) — **VOTE: Q1: Option A | Q2: Option A | Q3: Option A**

**Reasoning:** Montagent consistently enforces a strict separation between complex pre-processing and core runtime rendering. By keeping `project.json` pure and explicit (favoring literal `image` or `video` elements with definitive bounds, as per ADR-0145 and ADR-0177), the rendering engine remains fast, predictable, and simple. Handling PSD/AI layers (Q1) via pre-unpacking, trusting source metadata while leaving overrides to upstream transcode (Q2), and using a script to literally slice scene boundaries (Q3) fully align with Montagent's philosophy of delegating parsing/analysis to skill scripts and representing only the explicit timeline intent in the core schema.
**Trade-offs:** Pushing logic to pre-processing scripts means the user (or their scripting pipeline) assumes the burden of generating intermediate files and transcode steps, sacrificing "drop-in" convenience for some proprietary or malformed formats. However, this protects the core schema and engine from becoming bloated with proprietary parsers, conditional overrides, and complex runtime analysis code.
```

```
🗳️ **Juror 2** (flash) — **VOTE: Q1: Option A | Q2: Option A | Q3: Option A**

**Reasoning:** 
Montagent’s core architecture prioritizes a minimal, deterministic AST in `project.json` where rendering requires no complex heuristic decoders, heavy binary format interpreters, or dynamic temporal analysis. 
- For **Q1**, unpacking PSD/AI files upstream via a skill script into standard PNGs placed in literal `nest` or timeline coordinates adheres strictly to ADR-0017 and ADR-0171, keeping `montagent-render` lean without bundling complex Adobe file parsers.
- For **Q2**, trusting container metadata and requiring upstream FFmpeg normalization for interlacing, PAR, and frame rates prevents `project.json` from becoming a dumping ground for video transcoding flags. Alpha semantics remain predictable and uniform (straight alpha per ADR-0203/ADR-0229), while non-standard source fixes belong in the ingest step.
- For **Q3**, scene detection is fundamentally an authoring-time assistance task, not a rendering concern. Emitting literal contiguous `video` slices (`start`, `end`, `source_start`) via an FFmpeg `scdet` skill script honors ADR-0145 and ADR-0177 without introducing non-deterministic runtime detection into the core engine.

**Trade-offs:** 
- **Q1:** Costs disk space for unpacked asset extraction and requires an external parsing runtime (e.g., Python/psd-tools) during ingest, but avoids bloating the core renderer with proprietary Adobe format edge cases (Option B) while avoiding the friction of forcing manual layer export on users (Option C).
- **Q2:** Requires users/ingest workflows to re-encode or transcode non-standard footage (telecine, interlaced, anamorphic) before placing it on the timeline rather than flipping an in-schema flag (Option B), but keeps the renderer pipeline pure, simple, and free of ffmpeg-filter reimplementations.
- **Q3:** Scene detection results are baked at script runtime; changes to source media require re-running the skill script rather than dynamically adapting at render time (Option B), but preserves determinism, timeline editability, and timeline inspector transparency.
```

```
🗳️ **Juror 3** (inherit) — **VOTE: Q1: Option A | Q2: Option A | Q3: Option A**

**Reasoning:** Montagent is built on a minimal, deterministic core renderer driven by explicit, declarative schema primitives (`project.json`), with high-level asset intelligence handled upstream by agent skills. 
- For Q1, native PSD/AI decoding in core introduces massive third-party format bloat and maintenance burden (ADR-0017, ADR-0171). Decomposing layers via an upstream skill script into cropped PNGs placed inside a `nest` with `blend` modes maps cleanly onto existing core primitives (ADR-0201, ADR-0221) without modifying the schema.
- For Q2, `montagent-render` should not duplicate FFmpeg's transcoding or container-fixing capabilities. Trusting `ffprobe` metadata and normalizing frame rates, field order/deinterlacing, and non-square PAR upstream guarantees uniform, predictable input streams. Alpha rules remain consistent (ADR-0203/ADR-0229), with edge-case extractions left to explicit `effects[]` (ADR-0227).
- For Q3, scene detection is an ingestion and editing workflow, not a runtime rendering behavior. Evaluating scene cuts dynamically inside the render loop violates the explicit slice model (ADR-0145, ADR-0177). Having an upstream skill script emit literal, contiguous `video` cuts keeps the timeline inspectable, reproducible, and editable.

**Trade-offs:** 
- Unpacking PSD/AI layers and normalizing non-conformant media upstream generates intermediate disk assets (PNG files, conformed video clips) and incurs an ingestion preparation step.
- Rejecting in-schema interpretation overrides means malformed container metadata or unusual alpha channels cannot be patched on-the-fly inside `project.json`; they must be re-encoded or pre-processed prior to placement.
- These trade-offs are strongly preferable to bloating `montagent-core` and `montagent-render` with binary PSD/AI parsers, runtime video analysis filters, and ad-hoc container overrides.
```

### Tally and ruling

- **Q1 (`Option A`, 3–0):** Unanimous.
- **Q2 (`Option A`, 3–0):** Unanimous.
- **Q3 (`Option A`, 3–0):** Unanimous.

---

## Consequences

- The core engine, schema, and `montagent-render` remain completely free of proprietary Adobe binary parsers (`.psd`, `.ai`), decoder interpretation flags, and temporal scene detection filters.
- Layered graphics enter via standard transparent PNGs composited in `nest` containers with native `blend` modes.
- Media ingest adheres strictly to progressive, square-pixel, straight-alpha standards with upstream FFmpeg normalization.
- Scene detection produces inspectable, editable literal `video` elements on the timeline.
- All census gaps for AE import, render, export, preview, and interchange on map #896 are fully resolved.
