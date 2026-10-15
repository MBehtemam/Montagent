---
status: accepted
amends: 0011 (reaffirms deliverable is a single container file), 0023 (reaffirms frame is the verb for single static images), 0029 (reaffirms preview is the verb for contact sheets), 0143 (reaffirms render produces a single multiplexed file), 0156 (confirms PNG-in-MOV remains reserved for montagent-prerender caching), 0202 (preserves OutputConfig schema vocabulary), 0203 (confirms ProRes 4444 is the sole alpha deliverable for render), 0215 (disposes of After Effects export and render queue sequence rows from map #896)
---

# Image sequence export and PNG-in-MOV are refused from render, and ProRes 4444 remains the sole alpha video deliverable

[Does render emit numbered image sequences, and is lossless PNG-in-MOV needed beside ProRes 4444?](https://github.com/MBehtemam/Montagent/issues/1071)
on the After Effects map ([#896](https://github.com/MBehtemam/Montagent/issues/896)).

[ADR-0011](0011-deliverable-is-written-to-sibling-temp-and-renamed.md) established that a render's deliverable is written to an atomic sibling temporary file and renamed into place.
[ADR-0023](0023-video-source-dimensions-par-and-rotation.md) established `frame` as the tool for rendering an individual video frame to an image file.
[ADR-0029](0029-the-nine-preview-readings-are-ratified.md) established `preview` as the tool for rendering contact sheets.
[ADR-0104](0104-attestation-stamps-the-container.md) established attestation metadata stamps written into deliverable containers.
[ADR-0143](0143-render-keeps-libx264-medium-crf-20-pins-five-encoder-threads-and-says-so.md) established standard H.264 encode parameters.
[ADR-0156](0156-four-named-effects-join-the-effects-list-and-a-code-drawn-piece-enters-as-pre-rendered-footage.md) §7 and [#723](https://github.com/MBehtemam/Montagent/issues/723) established that `montagent-prerender` uses 8-bit lossless RGBA PNG-in-MOV (`-c:v png -pix_fmt rgba`) for pixel-exact caching of slow effects.
[ADR-0202](0202-the-output-field-accepts-a-path-string-or-a-structured-output-config-with-container-inference.md) established `OutputConfig` with container inference (`mp4`, `mov`, `mxf`).
[ADR-0203](0203-prores-and-dnxhr-mezzanine-codecs-render-via-lgpl-encoders-and-alpha-flows-through-a-32-bit-pipe.md) admitted Apple ProRes (`prores_ks`) and Avid DNxHR (`dnxhd`), and established 10-bit Apple ProRes 4444 (`alpha: true`, `yuva444p10le`) as the mastering deliverable for video with alpha channels.
[ADR-0215](0215-after-effects-is-precedent-for-rendered-results-and-capcut-and-premiere-stay-first-for-shape.md) admitted After Effects precedent for rendered results while keeping Premiere and CapCut first for mechanism and shape.

The After Effects censuses ([#907](https://github.com/MBehtemam/Montagent/issues/907) rows 45, 48, 52) flagged two potential output capabilities:
1. Numbered image-sequence export (rendering frames to `frame_%05d.png` / `.webp` / `.exr`).
2. 8-bit lossless RGBA in `.mov` (`PNG`-in-MOV / `qtrle` QuickTime Animation) alongside 10-bit ProRes 4444.

This ADR decides:
1. **Numbered image sequence output is refused from `render`**: `render` produces exactly one container file per invocation; directory creation, frame numbering patterns, and multi-file sequence output are refused.
2. **Lossless `PNG`-in-MOV and `qtrle` are refused from `output.video.codec` in `render`**: Apple ProRes 4444 (`prores` profile `4444` / `4444xq`) remains the sole video alpha deliverable in `render`. PNG-in-MOV remains dedicated to internal `montagent-prerender` caching.
3. **Sequence export and alternative formats are handled via downstream tools**: Frame extraction from ProRes 4444 / MP4 or repeated calls to `frame` serve any workflow needing separate image sequences.

---

## Decisions

### 1. Numbered image sequence export is refused from `render`

After Effects and traditional 3D DCC tools support rendering to numbered image folders (`name_[#####].png`). This is refused from Montagent's `render` verb:

1. **Single deliverable atomicity (ADR-0011)**: `render` writes to a sibling temporary file and atomically renames it into place. A directory of hundreds or thousands of files breaks this atomic guarantee, introducing partial writes on process termination or disk exhaustion.
2. **Attestation stamping (ADR-0104)**: `render` stamps container metadata (`user_data` / `com.apple.quicktime.description`) to guarantee provenance and detect external tampering. Individual image files lack a uniform, container-level tamper-evident attestation mechanism.
3. **Clear verb division**: Montagent separates responsibilities cleanly:
   - `frame`: renders a single timestamp to an exact image file ([ADR-0023](0023-video-source-dimensions-par-and-rotation.md)).
   - `preview`: renders a contact sheet grid for visual timeline inspection ([ADR-0029](0029-the-nine-preview-readings-are-ratified.md)).
   - `render`: compiles video, audio, and stems into single deliverable containers ([ADR-0077](0077-the-nine-render-readings-are-ratified.md), [ADR-0202](0202-the-output-field-accepts-a-path-string-or-a-structured-output-config-with-container-inference.md)).
4. **Agent and shell composability**: Extracting an image sequence from Montagent's master ProRes 4444 or MP4 deliverable is trivially achieved with an external FFmpeg command (`ffmpeg -i master.mov frames/f_%05d.png`). Adding directory path expansion, zero-padding pattern syntax (`%05d`, `[#####]`), and sequence validation into Montagent's core schema would clutter `OutputConfig` with redundant complexity.

### 2. ProRes 4444 remains the sole video alpha deliverable; PNG-in-MOV and qtrle are refused from `render`

While [#723](https://github.com/MBehtemam/Montagent/issues/723) demonstrated that `png` in `.mov` (`-c:v png -pix_fmt rgba`) round-trips pixel-exact RGBA without chroma conversion, it is refused as a user-facing deliverable codec:

1. **Industry standard for NLE interchange**: Apple ProRes 4444 is the universal standard for motion graphics overlays, title templates, and VFX elements across Adobe Premiere Pro, Final Cut Pro, DaVinci Resolve, and Avid Media Composer. It supports hardware-accelerated decoding across modern workstation GPUs and Apple Silicon Media Engines.
2. **Lack of hardware support for intra PNG / Animation**: QuickTime Animation (`qtrle`) is an obsolete legacy codec from the 1990s. PNG-in-MOV lacks hardware decode pipelines in NLEs, leading to stuttering playback and timeline dropping during editing.
3. **Internal vs Deliverable separation**:
   - `montagent-prerender` uses PNG-in-MOV internally ([ADR-0156](0156-four-named-effects-join-the-effects-list-and-a-code-drawn-piece-enters-as-pre-rendered-footage.md) §7, [#723](https://github.com/MBehtemam/Montagent/issues/723)) because it requires byte-for-byte exactness across re-runs to validate cache validity.
   - Deliverables emitted by `render` serve human editors, client delivery, and broadcast networks, where ProRes 4444 (`alpha: true`) is the accepted master format.

`output.video.codec` remains strictly constrained to `"h264"`, `"prores"`, and `"dnxhr"`.

---

## Consequences

- No changes are required in `crates/montagent-core/src/model/output.rs` or `crates/montagent-render/src/encode.rs`.
- `E-OUTPUT-EXTENSION-UNKNOWN` continues to reject non-container extensions (`.png`, `.webp`, `.exr`, `.jpg`).
- `E-CODEC-ALPHA-UNSUPPORTED` continues to enforce that `alpha: true` is valid only with ProRes `4444` and `4444xq`.
- Workflows requiring image sequences generate ProRes 4444 or MP4 masters and extract PNGs using standard FFmpeg or image tooling.
