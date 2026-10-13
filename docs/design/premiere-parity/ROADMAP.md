# Premiere Parity Implementation Architecture & Roadmap

> **Canonical architectural design and implementation roadmap for Premiere Pro feature parity in Montagent.**  
> Derived from the 2,087-feature inventory in [#882](https://github.com/MBehtemam/Montagent/issues/882) and [`docs/research/premiere-gap/INVENTORY.md`](file:///Users/mohammedehtemam/projects/github/Montagent/docs/research/premiere-gap/INVENTORY.md).  
> Resolves Wayfinder task [#984](https://github.com/MBehtemam/Montagent/issues/984), executing parent map [#978](https://github.com/MBehtemam/Montagent/issues/978).  
> Supported by research reports [#979](https://github.com/MBehtemam/Montagent/issues/979), [#980](https://github.com/MBehtemam/Montagent/issues/980), [#981](https://github.com/MBehtemam/Montagent/issues/981), [#982](https://github.com/MBehtemam/Montagent/issues/982), and [#983](https://github.com/MBehtemam/Montagent/issues/983).

---

## 1. Executive Summary & Core Invariants

Montagent is a deterministic, declarative video editor designed to be authored and manipulated by autonomous AI agents over the Model Context Protocol (MCP) and CLI, rather than operated via a direct-manipulation graphical user interface ([`CONTEXT.md`](file:///Users/mohammedehtemam/projects/github/Montagent/CONTEXT.md)).

In [#882](https://github.com/MBehtemam/Montagent/issues/882), a comprehensive census of 2,087 Adobe Premiere Pro features was conducted across 13 functional areas. The census established:
- **328 capabilities** already supported in Montagent (e.g. keyframe animation, Skia canvas compositing, audio normalization, stereo routing, HarfBuzz text layout).
- **1,392 genuine gaps** to be addressed.
- **204 refused rows** (interactive GUI docking, mouse trimming tools, floating panels, direct audio scrub gestures) that violate Montagent's foundational architecture.
- **163 deferred rows** requiring deeper prerequisite capability stabilization.

This roadmap synthesizes the architectural decisions, structural patterns, and implementation tranches required to close the 1,392 parity gaps while strictly upholding Montagent's core architectural tenets.

### 1.1 The Four Invariants ([ADR-0145](file:///Users/mohammedehtemam/projects/github/Montagent/docs/adr/0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md))

Every feature entering Montagent from Premiere Pro must satisfy all four tests without compromise:

| Invariant | Definition | Implementation Requirement |
| :--- | :--- | :--- |
| **1. Literal Values** | Every coordinate, colour, timing, gain, and parameter is an explicit, self-contained literal. | No hidden project state, implicit side-effects, relative timestamps without explicit references, or unmaterialized expressions. |
| **2. Closed Vocabulary** | Every type, property name, ease curve, blend mode, and effect kind belongs to a bounded, documented set. | Enforced by JSON schema validation (`montagent.schema.json`). No arbitrary string tags or open plugin hooks. |
| **3. Checkable by `validate`** | Every semantic constraint must be verifiable statically by reading the document and probing assets on disk. | No check requires rendering pixels, running neural network inference, or executing arbitrary scripts. |
| **4. Exact-String Replaceable** | Any agent edit can be enacted by string replacement (`replace_file_content` / `sed`) or arithmetic verbs (`shift`). | Deterministic key ordering, stable indent formatting, and localized element scopes. |

### 1.2 Model-Free Determinism ([ADR-0051](file:///Users/mohammedehtemam/projects/github/Montagent/docs/adr/0051-word-alignment-is-external-validate-and-compare-catch-drift.md), [ADR-0054](file:///Users/mohammedehtemam/projects/github/Montagent/docs/adr/0054-silence-detection-is-external-validate-and-compare-catch-drift.md), [ADR-0156](file:///Users/mohammedehtemam/projects/github/Montagent/docs/adr/0156-four-named-effects-join-the-effects-list-and-a-code-drawn-piece-enters-as-pre-rendered-footage.md))

Montagent's engine contains **zero machine learning or neural network models**. Premiere features that rely on AI (Auto Reframe, Scene Edit Detection, Speech-to-Text captions, Text-Based Editing, Enhance Speech, Warp Stabilizer) enter Montagent exclusively through the **External Analysis Protocol** ([#982](https://github.com/MBehtemam/Montagent/issues/982)):
1. Heavy inference is delegated to external CLI utilities or agent skills outside the core binary (e.g., Whisper, PySceneDetect, MediaPipe).
2. External tools emit standard, inspectable intermediate JSON artifacts (`.cuts.json`, `.words.json`, `.tracking.json`).
3. Agents translate these intermediate artifacts into literal schema structures (split element cuts, timed caption highlights, explicit keyframe coordinate lists).
4. Heavy spatial-temporal CV processing (e.g., Warp Stabilizer) enters as **pre-rendered footage** accompanied by a deterministic `.recipe/` sidecar folder ([ADR-0156](file:///Users/mohammedehtemam/projects/github/Montagent/docs/adr/0156-four-named-effects-join-the-effects-list-and-a-code-drawn-piece-enters-as-pre-rendered-footage.md)).
5. Montagent's `validate` and `render` verify and execute these project edits deterministically without access to the underlying model.

### 1.3 Agentic Tooling over GUI Panels

Premiere Pro's interface centers on mouse manipulation across specialized GUI panels (Timeline, Effects Control, Lumetri Color, Essential Sound, Essential Graphics). In Montagent, all human GUI concepts map to declarative data and MCP verbs:
- **Timeline dragging & trimming** $\rightarrow$ `shift` arithmetic with signed offsets, ripple deletion, and exact boundary replaces.
- **Effects Control & Lumetri Panels** $\rightarrow$ Declarative `effects: [...]` lists and `project.effects` arrays.
- **Timeline Scopes & Waveforms** $\rightarrow$ Structured readouts via `measure` and `query`.
- **Project Bin Management** $\rightarrow$ File system directory structures and JSON manifest inspection.

---

## 2. Foundations from Resolved Research Tickets

The roadmap integrates the normative technical conclusions established across research tickets [#979](https://github.com/MBehtemam/Montagent/issues/979) through [#983](https://github.com/MBehtemam/Montagent/issues/983):

```
       ┌────────────────────────────────────────────────────────┐
       │                Wayfinder Map #978                      │
       │    Premiere Parity Architecture & Implementation       │
       └───────────────────────────┬────────────────────────────┘
                                   │
         ┌─────────────────────────┼─────────────────────────┐
         ▼                         ▼                         ▼
  ┌──────────────┐          ┌──────────────┐          ┌──────────────┐
  │ Ticket #980  │          │ Ticket #979  │          │ Ticket #983  │
  │  Fractional  │          │Negative Shift│          │Export Codecs │
  │NTSC Timebase │          │ & Ripple Ops │          │   & Stems    │
  └──────┬───────┘          └──────┬───────┘          └──────┬───────┘
         │                         │                         │
         └─────────────────────────┼─────────────────────────┘
                                   │
         ┌─────────────────────────┴─────────────────────────┐
         ▼                                                   ▼
  ┌──────────────┐                                    ┌──────────────┐
  │ Ticket #981  │                                    │ Ticket #982  │
  │Multi-Element │                                    │   External   │
  │& Adjustments │                                    │Analysis Proto│
  └──────┬───────┘                                    └──────┬───────┘
         │                                                   │
         └─────────────────────────┬─────────────────────────┘
                                   │
                                   ▼
       ┌────────────────────────────────────────────────────────┐
       │                   Roadmap (#984)                       │
       │         Merged Master Implementation Plan              │
       └────────────────────────────────────────────────────────┘
```

### 2.1 Fractional & NTSC Timebases ([#980](https://github.com/MBehtemam/Montagent/issues/980))
- **Exact Rational Frame Rate Representation**: Project `fps` is expanded to accept exact rational fractions `{"num": 24000, "den": 1001}` (23.976 fps), `{"num": 30000, "den": 1001}` (29.97 fps), and `{"num": 60000, "den": 1001}` (59.94 fps), alongside standard integers (`24`, `25`, `30`, `60`). Floating-point rates (`fps: 23.976`) remain strictly forbidden by schema to prevent cross-platform IEEE-754 drift.
- **Absolute Milliseconds Invariant ([ADR-0005](file:///Users/mohammedehtemam/projects/github/Montagent/docs/adr/0005-absolute-integer-milliseconds.md))**: All timeline coordinates (`start`, `end`, `source_start`, `source_end`, keyframe `t`) remain **positive integer milliseconds (`i64`)**.
- **The 1001 ms Periodicity Theorem**: Because $\text{den} = 1001$, every exact 1001 ms block contains an exact integer count of frames:
  $$\text{Frames in } 1001\text{ ms} = \frac{1001 \times \text{num}}{1000 \times 1001} = \frac{\text{num}}{1000}$$
  - Exactly 24 frames per 1001 ms at 24000/1001 fps.
  - Exactly 30 frames per 1001 ms at 30000/1001 fps.
  - Exactly 60 frames per 1001 ms at 60000/1001 fps.
- **Deterministic Paint Instant**: Frame $n$ maps to timeline time via integer arithmetic:
  $$\text{instant}(n) = \left\lfloor \frac{n \times 1000 \times \text{den}}{\text{num}} \right\rfloor$$
- **Timecode Projection**: SMPTE Drop-Frame (DF) and Non-Drop-Frame (NDF) timecodes are query-time projections in `measure` and `query`, never stored as timeline keys.

### 2.2 Negative Shift & Ripple Operations ([#979](https://github.com/MBehtemam/Montagent/issues/979))
- **Unified Tool Surface ([ADR-0011](file:///Users/mohammedehtemam/projects/github/Montagent/docs/adr/0011-tool-surface-reads-checks-renders.md))**: Rather than introducing an 11th verb, `shift` is extended with signed `delta: i64` and an explicit opt-in boolean `delete: bool` (default `false`).
- **Cut-Origin Convention (Convention 2)**: For negative shifts ($\delta < 0$), `at` designates the excision boundary, removing the half-open interval $[at, at + |\delta|)$. Elements at or after $at + |\delta|$ shift left by $|\delta|$.
- **Two-Tier Safety Execution**:
  - `delete: false` (Safe Gap Closure): Excises timeline gaps. Refuses with `E-SHIFT-DESTRUCTIVE` if any element or keyframe exists in $[at, at + |\delta|)$. Requires `release` of affected slacks ([ADR-0032](file:///Users/mohammedehtemam/projects/github/Montagent/docs/adr/0032-slack-is-invariant-shift-refuses-compare-is-the-backstop.md), [ADR-0047](file:///Users/mohammedehtemam/projects/github/Montagent/docs/adr/0047-shift-releases-slack-by-boundary-instant-pairs.md)).
  - `delete: true` (Extraction / Ripple Delete): Deletes elements falling entirely within the excised span, truncating time-invariant straddlers, and returning an explicit receipt of destroyed IDs and dropped keyframes.
- **Keyframe Handling**: Keyframes on downstream elements shift isomorphically by $\delta$, preserving relative bezier slopes and timings identically. Straddling keyframes inside the cut span are pruned, with boundary states held via `step` eases.

### 2.3 Multi-Element Operations & Adjustment Layers ([#981](https://github.com/MBehtemam/Montagent/issues/981))
- **Rejection of Free-Floating `type: "adjustment"`**: Unbounded adjustment layers require mid-stack destination read-backs (`surface.image_snapshot()`), copying ~33.2 MB per frame at 4K RGBA8, stalling memory pipelines, destroying the static prefix cache ([ADR-0195](file:///Users/mohammedehtemam/projects/github/Montagent/docs/adr/0195-the-static-prefix-cache-holds-invariant-layers-at-the-bottom-of-the-stack.md), [ADR-0198](file:///Users/mohammedehtemam/projects/github/Montagent/docs/adr/0198-an-element-that-draws-nothing-neither-joins-nor-ends-the-static-prefix.md)), and creating tile seam divergence across parallel $K$-painters.
- **Three-Part Architecture**:
  1. *Master Effects (`project.effects`)*: Top-level color grading and output filtering applied once at the device boundary after all track layers are drawn. Fully preserves the static prefix cache and streaming forward rasterization.
  2. *Attribute Replication (Paste Attributes)*: Agentic tooling to replicate identical `effects: [...]` blocks across discrete elements via exact-string replace.
  3. *Bounded Group Compositing (`type: "nest"`)*: Offscreen-rendered sub-timelines filtered as a single unit under [ADR-0166](file:///Users/mohammedehtemam/projects/github/Montagent/docs/adr/0166-capcut-and-premiere-nest-with-a-transform-and-adr-0001-refuses-the-clock-not-the-shared-space.md), preserving local coordinate bounds without mid-stack canvas read-backs.

### 2.4 External Analysis Protocol ([#982](https://github.com/MBehtemam/Montagent/issues/982))
- **Contract Boundary**: External tools execute in userland/agent environments and output strictly typed JSON documents:
  - Scene Cut Detection $\rightarrow$ `.cuts.json` (timestamps in source milliseconds).
  - Speech-to-Text $\rightarrow$ `.words.json` (word-level start/end milliseconds, confidence, text).
  - Visual Motion Tracking $\rightarrow$ `.tracking.json` (per-frame bounding boxes / translation coordinates).
- **Project Translation**: Agents ingest these files and emit literal Montagent schema structures: split elements on cuts, timed caption highlights on `text` elements, and explicit `x`/`y`/`scale` keyframe lists.
- **Warp Stabilizer / Heavy CV**: Exported to pre-rendered lossless video accompanied by a deterministic `.recipe/` sidecar folder ([ADR-0156](file:///Users/mohammedehtemam/projects/github/Montagent/docs/adr/0156-four-named-effects-join-the-effects-list-and-a-code-drawn-piece-enters-as-pre-rendered-footage.md)).

### 2.5 Professional Export Codecs & Stems ([#983](https://github.com/MBehtemam/Montagent/issues/983))
- **Mezzanine Video Codecs**: Integration of FFmpeg's native LGPL `prores_ks` encoder (profiles: `proxy`, `lt`, `standard`, `hq`, `4444`, `4444xq`) and `dnxhd` (DNxHR LB, SQ, HQ, HQX, 444) in QuickTime MOV and MXF OP1a containers.
- **Alpha Channel Preservation**: ProRes 4444 with alpha uses pixel format `yuva444p10le`. Canvas pipeline switches from `Canvas::rgb` (over black) to `Canvas::rgba`, canvas clears to transparent RGBA `[0, 0, 0, 0]`, and odd dimension padding uses transparent fill.
- **Audio Stem Export**: Elements/tracks declare an optional semantic `role` (`"dialogue"`, `"music"`, `"sfx"`). Render dispatches decoupled audio-only FFmpeg passes (`-vn`) writing 24-bit 48 kHz Broadcast WAV files (`pcm_s24le`) partitioned by role.
- **LGPL Compliance & Determinism**: Software encoding via `prores_ks` provides consistent, bit-level determinism across macOS, Linux, and Windows without proprietary licensing entanglements or hardware driver variance.

---

## 3. Comprehensive Architectural Patterns Across the 13 Areas

Every parity gap in the 2,087-row census maps to one of the following architectural patterns:

```
┌────────────────────────────────────────────────────────────────────────┐
│                        Montagent Core System                           │
│                                                                        │
│  ┌───────────────────────┐  ┌─────────────────┐  ┌──────────────────┐  │
│  │ 1. Timeline Core      │  │ 2. Audio Engine │  │ 3. Skia Graphics │  │
│  │ - Flat Tracks         │  │ - 48kHz Bus     │  │ - Text Shaping   │  │
│  │ - Rational FPS        │  │ - EBU R128 Norm │  │ - Vector Paths   │  │
│  │ - Ripple / Shift      │  │ - Dynamics / EQ │  │ - Closed Effects │  │
│  └───────────┬───────────┘  └────────┬────────┘  └────────┬─────────┘  │
│              │                       │                    │            │
│              ▼                       ▼                    ▼            │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │ 4. Compositor & Master Pipeline                                  │  │
│  │ - Forward Streaming Painter (Parallel K-painters)                │  │
│  │ - Static Prefix Cache                                            │  │
│  │ - Master Effects (Device Boundary Grading & 3D LUTs)             │  │
│  │ - Bounded Nests                                                  │  │
│  └───────────────────────────────────┬──────────────────────────────┘  │
│                                      │                                 │
│                                      ▼                                 │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │ 5. Delivery & Encoding Engine                                    │  │
│  │ - Atomic Temp Promotion (Deliverable)                            │  │
│  │ - Mezzanine Encoders: ProRes 422/4444 (Alpha), DNxHR             │  │
│  │ - Audio Stem Generators (D/M/E Broadcast WAV)                    │  │
│  └──────────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────┬─────────────────────────────────┘
                                       │
        ┌──────────────────────────────┴──────────────────────────────┐
        ▼                                                             ▼
┌──────────────────────────────┐              ┌──────────────────────────────┐
│  6. Decoupled Interchange    │              │ 7. External Analysis (AI/ML) │
│  - montagent-otio (OTIO)     │              │ - Whisper (Captions / Words) │
│  - montagent-fcpxml (FCPXML) │              │ - PySceneDetect (.cuts.json) │
│  - montagent-edl (CMX 3600)  │              │ - MediaPipe Tracker (.track) │
│  - Standalone Translators    │              │ - Warp Stabilizer (Recipes)  │
└──────────────────────────────┘              └──────────────────────────────┘
```

### Area 1: Editing & Timeline Core (314 gaps)
- **Constraint-Enforced Track Lanes ([ADR-0004](file:///Users/mohammedehtemam/projects/github/Montagent/docs/adr/0004-tracks-as-constrained-lanes.md))**: Elements within a track are strictly ordered serially in time without overlaps. Overlaps are statically rejected by `validate`.
- **Negative Shift & Ripple Verb**: Signed `shift(at, delta, delete, release)` enables single-command ripple deletes and gap closures.
- **Nested Timelines (`type: "nest"`)**: Encapsulated element graphs running on absolute timeline milliseconds, transforming as a group under [ADR-0166](file:///Users/mohammedehtemam/projects/github/Montagent/docs/adr/0166-capcut-and-premiere-nest-with-a-transform-and-adr-0001-refuses-the-clock-not-the-shared-space.md).
- **Exact Rational Timebase**: Expressed via `fps: {"num": N, "den": D}`, governing frame indexing without floating-point drift.

### Area 2: Delivery & Export (172 gaps)
- **Unified `OutputConfig` Schema**: Backward-compatible untagged union (`string | OutputConfig`).
- **Mezzanine Codecs**: ProRes (Proxy through 4444 XQ), DNxHR (LB to 444), uncompressed PCM audio, QuickTime MOV and MXF wrappers.
- **Alpha Export**: RGBA Skia surface clear and transparent pipeline feeding `yuva444p10le`.
- **Audio Stem Export**: Declarative `role` tags dispatched to parallel audio-only encoding graphs.

### Area 3: Audio Processing & Mixing (248 gaps)
- **Unified 48 kHz Stereo Processing Bus**: Master loudness normalization under ITU-R BS.1770-4 and EBU R128 ([ADR-0172](file:///Users/mohammedehtemam/projects/github/Montagent/docs/adr/0172-the-master-loudness-filter-is-a-loudnorm-call-with-six-arguments-measured-by-ebur128.md), [ADR-0174](file:///Users/mohammedehtemam/projects/github/Montagent/docs/adr/0174-master-ceiling-is-a-true-peak-limiter-beside-loudness.md)).
- **Stereo Balance & Channel Routing ([ADR-0184](file:///Users/mohammedehtemam/projects/github/Montagent/docs/adr/0184-pan-is-a-linear-balance-and-channels-is-a-five-word-routing-field-both-flat-on-audio-and-video.md))**: Linear pan law (`-1.0` to `+1.0`) and discrete 5-word channel matrices (`stereo`, `mono`, `left`, `right`, `swap`).
- **Dynamics Suite**: Deterministic audio filters: Compressor, Limiter ([ADR-0180](file:///Users/mohammedehtemam/projects/github/Montagent/docs/adr/0180-the-compressor-and-the-limiter-are-two-audio-effects-with-no-defaults-and-an-rms-threshold.md)), Noise Gate ([ADR-0183](file:///Users/mohammedehtemam/projects/github/Montagent/docs/adr/0183-the-noise-gate-is-one-stackable-audio-effect-with-a-range-no-hold-and-an-rms-threshold.md)), and Parametric Equalizer.
- **Audio Transitions**: Crossfade curves (equal power, linear, exponential) implemented via FFmpeg `afade` chains ([ADR-0176](file:///Users/mohammedehtemam/projects/github/Montagent/docs/adr/0176-transitions-between-audio-elements-are-three-fade-shapes-and-render-as-afade.md)).

### Area 4: Color Grading & Lumetri Parity (156 gaps)
- **Master Stage Effects (`project.effects`)**: Output-level color grading executed post-composite without canvas read-backs.
- **ASC CDL (Color Decision List)**: Explicit `slope`, `offset`, `power`, and `saturation` color grading applied as Skia color filters.
- **3D LUT Ingestion**: Application of standard `.cube` tetrahedral interpolation LUTs as Skia custom runtime effects (`SkRuntimeEffect`).
- **Color Space Management**: Explicit project working space (`sRGB`, `Rec.709`, `Display P3`) declared on `frame.color_space`.
- **Diagnostic Color Scopes**: Histogram, Waveform (YCbCr), and Vectorscope metrics computed deterministically and queryable via `measure`.

### Area 5: Visual Effects & Compositing (185 gaps)
- **Closed Effect Vocabulary**: Strictly enumerated effect kinds (blur, color filter, drop shadow, glow, noise, sharpen, directional blur).
- **Porter-Duff & Advanced Blend Modes ([ADR-0147](file:///Users/mohammedehtemam/projects/github/Montagent/docs/adr/0147-blend-mode-is-twenty-skia-blend-modes-flat-on-every-element.md))**: 20 standard blend modes applied directly to element draw calls.
- **Vector Masking & Matte Clipping ([ADR-0154](file:///Users/mohammedehtemam/projects/github/Montagent/docs/adr/0154-mask-is-a-path-or-rect-clipping-the-element-itself-and-a-shared-matte-is-refused.md))**: Self-contained integer pixel bezier paths clipping element draw bounds.
- **Static Prefix Acceleration ([ADR-0195](file:///Users/mohammedehtemam/projects/github/Montagent/docs/adr/0195-the-static-prefix-cache-holds-invariant-layers-at-the-bottom-of-the-stack.md))**: Caching invariant bottom layers across timeline spans, skipping redundant rasterization.

### Area 6: Titles & Motion Graphics (112 gaps)
- **Rich Text Typography**: Advanced font layout via HarfBuzz and Skia (`skpath`), tracking fallback font chains ([ADR-0007](file:///Users/mohammedehtemam/projects/github/Montagent/docs/adr/0007-fonts-are-files-relative-to-the-project.md), [ADR-0010](file:///Users/mohammedehtemam/projects/github/Montagent/docs/adr/0010-font-fallback-chains-must-be-explicit.md)).
- **Text Stagger & Unit Animation ([ADR-0151](file:///Users/mohammedehtemam/projects/github/Montagent/docs/adr/0151-text-stagger-is-a-timed-window-over-units-and-a-keyframe-inside-it-is-refused.md))**: Character, word, and line timed reveal windows.
- **Vector Shapes**: Declarative geometric paths, rounded rectangles, multi-point polygons, and animated stroke trims.

### Area 7: Captions & Subtitles (68 gaps)
- **Declarative Caption Elements**: Native `text` elements with timed word `highlight` spans ([ADR-0051](file:///Users/mohammedehtemam/projects/github/Montagent/docs/adr/0051-word-alignment-is-external-validate-and-compare-catch-drift.md)).
- **SubRip (.srt) and WebVTT (.vtt) Interchange**: External conversion tools translating subtitle files to/from Montagent track elements.
- **Styling Rules**: Bounded font sizes, text outlines, background bounding boxes, and vertical margin constraints.

### Area 8: Motion & Transforms (74 gaps)
- **2D Affine Transform Matrix**: Explicit `x`, `y`, `scale` ($[s_x, s_y]$), `rotation` (degrees), `origin` (anchor point), and `opacity`.
- **Cubic Bezier Easing**: Precise $[x_1, y_1, x_2, y_2]$ unit interval control points with automatic hold/step collapse ([ADR-0012](file:///Users/mohammedehtemam/projects/github/Montagent/docs/adr/0012-flat-transform-keyframes-carried-by-their-element.md)).
- **Spatial Motion Paths**: Multi-segment keyframed spatial coordinates with linear and curved interpolation.

### Area 9: Media Import & Ingestion (45 gaps)
- **Deterministic Probing**: Comprehensive media metadata extraction (dimensions, duration, SAR/DAR, channel layout, sample rate, bit depth) via `ffprobe` ([ADR-0009](file:///Users/mohammedehtemam/projects/github/Montagent/docs/adr/0009-ffprobe-is-the-media-inspector.md), [ADR-0115](file:///Users/mohammedehtemam/projects/github/Montagent/docs/adr/0115-ffmpeg-7-1-with-libx264-is-the-floor-and-a-tool-qualification-finds-out.md)).
- **Image Sequence Support**: Numbered image sequence patterns (`frame_%04d.png`) treated as time-based video sources.
- **Vector Ingestion**: Static and animated SVG rasterization at target resolution without pixelation.

### Area 10: Timeline Interchange (52 gaps)
- **Decoupled Architecture**: Zero interchange dependencies compiled into the core renderer.
- **OpenTimelineIO (OTIO) Adapter**: External CLI tool (`montagent-otio`) converting OTIO timelines to Montagent project files and back.
- **EDL & FCPXML Support**: Standard CMX 3600 and Final Cut Pro XML translation utilities mapping cuts, transitions, and audio levels to declarative tracks.

### Area 11: AI & Computational Workflows (35 gaps)
- **External Analysis Protocol ([#982](https://github.com/MBehtemam/Montagent/issues/982))**: Whisper for speech-to-text, OpenCV/PySceneDetect for cuts, MediaPipe for face/motion tracking.
- **Pre-Rendered Recipes ([ADR-0156](file:///Users/mohammedehtemam/projects/github/Montagent/docs/adr/0156-four-named-effects-join-the-effects-list-and-a-code-drawn-piece-enters-as-pre-rendered-footage.md))**: Warp stabilization and generative fill stored as pre-rendered clips alongside reproducible recipe manifests.

### Area 12: VR & 360 Video (14 gaps)
- **Equirectangular Projection**: Spherical 360° video mapped to rectilinear viewports via specialized Skia runtime shaders or pre-rendered passes.

### Area 13: Project Architecture & Workflow (12 gaps)
- **Git-Native Versioning**: Deterministic key serialization, zero binary blobs, and exact-string replaceability.
- **Project Structure Validation**: Strict `validate` checks preventing stale references, orphan assets, or broken paths.

---

## 4. Implementation Tranches & Milestones

Implementation is sequenced into three progressive execution tranches, prioritizing foundational editing mechanics and professional mezzanine export before perceptual and format translation systems.

```
┌────────────────────────────────────────────────────────────────────────┐
│                        TRANCHE 1: FOUNDATION                           │
│                 Timeline Core & Mezzanine Delivery                     │
│                                                                        │
│  - Fractional / NTSC Rational Timebase (num/den) (#980)                │
│  - Signed Shift, Safe Gap Close & Ripple Deletion (#979)               │
│  - ProRes (Proxy to 4444 XQ with Alpha) & DNxHR Export (#983)          │
│  - Audio Stem Export (WAV 24-bit 48kHz by role) (#983)                 │
│  - External Analysis Protocol Base & JSON Specs (#982)                 │
│  - External OpenTimelineIO Adapter (montagent-otio)                    │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│                        TRANCHE 2: ENGINES                              │
│                      Audio & Color Pipeline                            │
│                                                                        │
│  - Project Master Effects (project.effects) (#981)                     │
│  - ASC CDL Color Grading (Slope, Offset, Power, Saturation)            │
│  - 3D LUT Ingestion (.cube tetrahedral SkRuntimeEffect)                │
│  - Full Dynamics (Compressor, Limiter, Noise Gate)                     │
│  - Parametric EQ (Highpass, Lowpass, Peaking bands)                    │
│  - Diagnostic Scopes in measure (Histogram, Waveform, Vectorscope)     │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│                       TRANCHE 3: COMPOSITING                           │
│               Advanced Graphics, Nests & Interchange                   │
│                                                                        │
│  - Nested Sequences (type: "nest") (#981, ADR-0166)                    │
│  - Vector Shape Elements & Path Trimming                               │
│  - Motion Tracking Literal Keyframe Pipeline (#982)                    │
│  - Word-level SRT / VTT Bidirectional Translators                      │
│  - FCPXML and CMX 3600 EDL Format Adapters                             │
│  - Pre-rendered Recipe Framework for Heavy CV (#982, ADR-0156)        │
└────────────────────────────────────────────────────────────────────────┘
```

### Tranche 1: Foundation — Timeline Core & Mezzanine Delivery
*Objective: Establish core NLE editing mechanics, eliminate frame rate rounding bugs, and unlock broadcast-quality master delivery.*

1. **Rational Timebase Engine**:
   - Update `montagent.schema.json` to accept `fps: {"num": N, "den": D}`.
   - Implement integer painted instant formula in `crates/montagent-core/src/exact.rs`.
   - Update `verify` and `render` to measure PTS intervals using exact fractional timebases.
2. **Negative Shift & Ripple Operations**:
   - Extend `shift` verb in `crates/montagent-core/src/verbs/shift.rs` with signed `delta: i64` and `delete: bool`.
   - Implement cut-origin excision interval $[at, at + |\delta|)$.
   - Enforce refusal codes `E-SHIFT-UNDERFLOW`, `E-TRACK-OVERLAP`, `E-SHIFT-DESTRUCTIVE`, and `E-SHIFT-SLACK`.
   - Implement keyframe truncation, downstream shifting, and hold-step synthesis.
3. **Mezzanine Export Pipeline**:
   - Expand `output` in `montagent.schema.json` to support `OutputConfig`.
   - Implement `prores_ks` and `dnxhd` encoder dispatch in `crates/montagent-render/src/encode.rs`.
   - Implement transparent canvas background clear and `yuva444p10le` pipeline for ProRes 4444 with alpha.
   - Implement audio stem generation (`montagent render --stems-only`) writing 24-bit 48 kHz WAV files.
4. **Interchange Foundation**:
   - Create standalone CLI crate `montagent-otio` to translate OTIO tracks and markers to Montagent projects.

### Tranche 2: Engines — Audio & Color Pipeline
*Objective: Build out professional sound mixing and cinematic color grading.*

1. **Project Master Effects**:
   - Add `effects: [...]` to root schema in `montagent.schema.json`.
   - Execute master effect chain at device boundary post-stack composite, fully preserving the static prefix cache.
2. **ASC CDL & 3D LUT Color Engine**:
   - Implement `cdl` color filter in Skia painter (`slope`, `offset`, `power`, `saturation`).
   - Implement `.cube` 3D LUT parser and Skia runtime shader tetrahedral interpolation.
   - Implement color scopes (histogram, RGB parade, vectorscope) in `montagent measure`.
3. **Audio Dynamics & Equalization**:
   - Complete implementation of parametric EQ (peaking, low shelf, high shelf).
   - Wire compressor and limiter ([ADR-0180](file:///Users/mohammedehtemam/projects/github/Montagent/docs/adr/0180-the-compressor-and-the-limiter-are-two-audio-effects-with-no-defaults-and-an-rms-threshold.md)) and noise gate ([ADR-0183](file:///Users/mohammedehtemam/projects/github/Montagent/docs/adr/0183-the-noise-gate-is-one-stackable-audio-effect-with-a-range-no-hold-and-an-rms-threshold.md)) into `Mix::of`.
   - Ensure EBU R128 loudness compliance verification in `montagent verify`.

### Tranche 3: Compositing — Advanced Graphics, Nests & Interchange
*Objective: Complete advanced graphics, complex timeline grouping, and external ecosystem interchange.*

1. **Nested Sequences (`type: "nest"`)**:
   - Implement encapsulated nested timeline element type with isolated offscreen Skia rendering.
   - Support nested transforms and group effects under ADR-0166.
2. **Vector Shapes & Advanced Motion**:
   - Add `shape` element type supporting bezier path outlines, fills, and trim paths.
   - Integrate tracking data into literal keyframe arrays (`.tracking.json` ingestion).
3. **Captions & Format Adapters**:
   - Implement standalone `montagent-subtitles` CLI for bidirectional SRT/VTT translation.
   - Implement `montagent-fcpxml` and `montagent-edl` converters.
   - Implement the `.recipe/` runner for Warp Stabilizer pre-rendered footage.

---

## 5. Specification of Follow-On Child Capability Maps

Following the completion of this master architectural roadmap, detailed execution is delegated to four focused child capability maps in GitHub issues:

### Child Map 1: Timeline Core & Mechanics
- **Scope**: Rational frame rates (`fps`), signed `shift`, safe gap closure, ripple delete, nested sequences (`type: "nest"`), and track collision validation.
- **Key ADRs to Author**:
  - `ADR-Rational-Timebase`: Rational `{num, den}` schema representation and 1001 ms periodicity indexing.
  - `ADR-Negative-Shift`: Cut-origin convention, signed `shift` signature, and keyframe hold-step synthesis.
  - `ADR-Nested-Sequences`: Offscreen buffer isolation, transform propagation, and absolute timeline clocks.

### Child Map 2: Professional Export & Mezzanine Codecs
- **Scope**: ProRes (Proxy to 4444 XQ), DNxHR (LB to 444), alpha channel canvas pipeline, structured `OutputConfig`, and Broadcast WAV stem export.
- **Key ADRs to Author**:
  - `ADR-Output-Configuration`: Schema evolution for `output` (`string | OutputConfig`).
  - `ADR-ProRes-Alpha`: Skia transparent clear, `yuva444p10le` pixel mapping, and `prores_ks` command assembly.
  - `ADR-Audio-Stems`: Role-based audio stem partitioning and parallel FFmpeg dispatch.

### Child Map 3: Color Pipeline & Scopes
- **Scope**: Master effects (`project.effects`), ASC CDL color grading, 3D LUT (.cube) parsing and shader interpolation, and diagnostic scopes in `measure`.
- **Key ADRs to Author**:
  - `ADR-Master-Effects`: Root-level effect processing at device boundaries.
  - `ADR-Color-CDL-LUT`: CDL mathematical formulation and tetrahedral LUT shader execution.
  - `ADR-Color-Scopes`: Histogram, YCbCr waveform, and vectorscope data generation in `measure`.

### Child Map 4: Timeline Interchange & External Formats
- **Scope**: Decoupled format translation tools: OpenTimelineIO (`montagent-otio`), Final Cut Pro XML (`montagent-fcpxml`), CMX 3600 EDL (`montagent-edl`), and subtitle converters (`montagent-subtitles`).
- **Key ADRs to Author**:
  - `ADR-Interchange-Boundary`: Decoupled CLI architecture and zero-renderer-dependency policy.
  - `ADR-OTIO-Mapping`: Bidirectional schema transformation between OTIO schemas and Montagent declarative tracks.
