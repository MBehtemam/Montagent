# 239. Footage analysis and cleanup run offline and emit literal keyframes or pre-rendered alpha footage

Date: 2026-10-18

## Status

Accepted

## Context

After Effects provides extensive footage-analysis and cleanup toolsets:
1. **2D Point and Two-Point Tracking**: Tracking features across frames to pin text, callouts, or graphics to moving objects.
2. **Mask and Face Tracking**: Propagating polygon mask vertices across frames to isolate or censor faces and objects.
3. **Motion Stabilization**: Warp Stabilizer and 2D stabilizer analyzing camera jitter and smoothing camera motion.
4. **Rotoscoping / Subject Segmentation**: Roto Brush isolating foreground subjects without a green screen.
5. **Content-Aware Fill**: Video inpainting removing unwanted objects or boom mics across moving footage.
6. **Video Denoise / Grain Removal**: Temporal noise filters (Remove Grain) averaging noise across adjacent frames.

Montagent's architecture is governed by strict invariants:
- `ADR-0145`: Literal values, closed schema vocabulary, checkable by `validate`, editable by exact string replace. No hidden solvers, optical flow runtimes, or detector models inside `montagent render`.
- `ADR-0149` and `ADR-0156`: Stateless, embarrassingly parallel frame painting across 1–10 threads. A painter rendering frame 42 cannot depend on the internal state, optical flow vectors, or temporal buffers of frame 41.
- `ADR-0167` and `ADR-0215`: 3D camera tracking and 4-point perspective corner pins are refused.
- `ADR-0177`: Motion synthesis belongs in authoring scripts that write literal keyframes into `project.json` ahead of render.
- `ADR-0203`: Straight-alpha ProRes 4444 / DNxHR HQX is the sole lossless format for pre-rendered video clips carrying alpha.

We must decide how each footage-analysis and cleanup capability enters Montagent without violating engine determinism or bloating the core runtime with machine learning runtimes (OpenCV, ONNX, PyTorch).

## Decision

1. **All footage analysis and cleanup tools run strictly offline during authoring, never inside `montagent render`.**
   No ML models, optical flow algorithms, or temporal frame-accumulation buffers are admitted to the core Rust crates (`montagent-core`, `montagent-render`). `montagent render` remains a pure, frame-stateless function of `project.json` and static media assets.

2. **2D Point and Two-Point Tracking enters as an offline authoring script emitting literal keyframes (#1112):**
   - Feature tracking is performed ahead of time by an authoring script (`skills/montagent-footage/scripts/track.py` or OpenCV equivalent).
   - The script tracks a target feature point or bounding box across source video frames and writes literal `x`, `y`, and optional `rotation` / `scale` keyframes directly onto the target element (`text`, `image`, `nest`, `rect`) in `project.json`.
   - **Douglas-Peucker Simplification**: The tracking script MUST apply Douglas-Peucker trajectory simplification (default tolerance `0.5` pixels) to collapse redundant linear segments and prevent per-frame keyframe flooding, maintaining human-readability and string-replace editability (`ADR-0145`).

3. **Per-frame polygon mask vertex tracking is refused; affine tracking and pre-rendered mattes enter (#1113):**
   - Emitting 30–60 arrays of polygon coordinates per second for deforming masks into `project.json` creates massive JSON files that violate the "editable by exact-string replace" invariant and makes Git diffs unreadable.
   - Rigid or affine tracking (e.g. blurring a moving face, censoring a license plate) is achieved by tracking the center and scale onto a `nest` element or parent element transform while keeping the `mask` polygon static relative to its element.
   - Complex non-rigid deforming contours (human bodies, moving fabric) are refused as polygon keyframe tracks in `project.json`. They enter exclusively as pre-rendered alpha video mattes (Decision 5 below).

4. **Camera Motion Stabilization pre-renders via FFmpeg `vidstab` recipes; in-engine warp stabilizers are refused (#1114):**
   - Runtime mesh-warp stabilization is refused from `render`.
   - General camera stabilization is performed offline via the two-pass FFmpeg `vidstabdetect` and `vidstabtransform` filter pipeline documented in `skills/montagent-footage/SKILL.md` (or the specialized ink-drawing reference script in `skills/montagent-prerender/references/stabilize.md`). The resulting stabilized clip is encoded as ProRes 4444 or high-bitrate MP4 and placed as an ordinary `video` element.
   - For simple translation-only horizon pinning, counter-transform keyframes (`x`, `y` inverted from a tracker) may be written onto the `video` element, provided the element's `width`/`height` or `scale` is enlarged by at least 10% to prevent unpainted black borders during canvas jitter compensation.

5. **AI Rotoscoping / Subject Segmentation enters as an offline recipe emitting straight-alpha ProRes 4444 (#1115):**
   - Roto Brush and AI foreground segmentation models (SAM 2, BiRefNet, rembg) run in external Python/CLI environments outside the Montagent toolchain.
   - The segmented subject is exported as a single-file, straight-alpha `prores_4444` `.mov` file with Rec.709 color tags (`ADR-0203`, `ADR-0238`).
   - The resulting clip is placed in `project.json` as an ordinary `video` element (`probe` reports `carries: true` for alpha).

6. **Content-Aware Fill (video inpainting) and temporal video denoise are refused from runtime `effects[]` and enter as offline recipes (#1116):**
   - Temporal inpainting (Content-Aware Fill) and multi-frame temporal video denoise (`hqdn3d`, `nlmeans`, `bm3d`) violate the frame-stateless parallel painter invariant (`ADR-0149`).
   - They are refused as runtime effects in `effects[]`.
   - Unwanted objects, boom mics, or high-ISO sensor noise are cleaned upstream via FFmpeg filters (`hqdn3d`) or external inpainting tools (ProPainter) before importing the cleaned footage into Montagent.

## Consequences

### Positive
- Zero ML runtime dependencies (ONNX, PyTorch, LibTorch, OpenCV C++) in `montagent-render` or `montagent-core`.
- Multi-threaded rendering remains strictly frame-stateless, embarrassingly parallel, and bit-identical across runs.
- `project.json` remains concise, readable, and easily diffed or manipulated by LLM agents.
- Producers and editors retain access to state-of-the-art AI segmentation and stabilization tools without coupling Montagent's core to fast-moving ML research models.

### Negative / Trade-offs
- Tracking and rotoscoping require an explicit upstream authoring step rather than a single check-box inside `project.json`.
- Large deforming subject mattes require disk storage for ProRes 4444 footage files rather than analytic vector outlines.
