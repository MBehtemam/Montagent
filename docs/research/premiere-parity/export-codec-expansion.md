# Research: Export Codec Expansion (ProRes, DNxHR, Alpha Channel, and Audio Stems)

**Ticket:** [#983](https://github.com/MBehtemam/Montagent/issues/983)  
**Parent Initiative:** [#978](https://github.com/MBehtemam/Montagent/issues/978) (Premiere Pro Parity Milestone)  
**Related Documents:**
- [`docs/research/premiere-gap/export.md`](../premiere-gap/export.md) (Export Gap Analysis)
- [`docs/adr/0009-ffmpeg-is-spawned-never-linked.md`](../../adr/0009-ffmpeg-is-spawned-never-linked.md)
- [`docs/adr/0011-deliverable-is-written-to-sibling-temp-and-renamed.md`](../../adr/0011-deliverable-is-written-to-sibling-temp-and-renamed.md)
- [`docs/adr/0021-odd-frame-dimension-is-padded-to-even.md`](../../adr/0021-odd-frame-dimension-is-padded-to-even.md)
- [`docs/adr/0077-the-nine-render-readings-are-ratified.md`](../../adr/0077-the-nine-render-readings-are-ratified.md)
- [`docs/adr/0104-attestation-stamps-the-container.md`](../../adr/0104-attestation-stamps-the-container.md)
- [`docs/adr/0115-ffmpeg-7-1-with-libx264-is-the-floor-and-a-tool-qualification-finds-out.md`](../../adr/0115-ffmpeg-7-1-with-libx264-is-the-floor-and-a-tool-qualification-finds-out.md)
- [`docs/adr/0143-render-keeps-libx264-medium-crf-20-pins-five-encoder-threads-and-says-so.md`](../../adr/0143-render-keeps-libx264-medium-crf-20-pins-five-encoder-threads-and-says-so.md)
- [`docs/adr/0172-the-master-stage-is-a-top-level-loudness-target-and-true-peak-ceiling-reached-by-one-measured-gain.md`](../../adr/0172-the-master-stage-is-a-top-level-loudness-target-and-true-peak-ceiling-reached-by-one-measured-gain.md)
- [`docs/adr/0184-pan-is-a-linear-balance-and-channels-is-a-five-word-routing-field-both-flat-on-audio-and-video.md`](../../adr/0184-pan-is-a-linear-balance-and-channels-is-a-five-word-routing-field-both-flat-on-audio-and-video.md)

---

## 1. Executive Summary

Montagent currently renders to exactly one deliverable shape: **H.264 (`libx264`), 8-bit `yuv420p`, CRF 20, preset `medium`, pinned 5 threads, AAC 160 kb/s 48 kHz stereo, in an MP4 container with `+faststart`** (ADR-0077, ADR-0143).

While this provides an optimal baseline for web distribution, it creates four severe architectural ceilings when integrating with professional post-production workflows (Premiere Pro, DaVinci Resolve, Final Cut Pro, Pro Tools):
1. **No Mezzanine / Intermediate Codecs:** Mastering and archival require intra-frame codecs with minimal compression artifacts (Apple ProRes 422 / 422 HQ, Avid DNxHR HQ / HQX).
2. **No Alpha Channel Export:** Motion graphics, lower thirds, titles, and compositing overlays cannot be exported with transparency; Montagent composites over opaque black (`Canvas::BLACK`) and strips alpha to 24-bit RGB (`rgb24`) before piping to FFmpeg.
3. **No Audio Stem Separation:** Delivery specifications require isolated Dialogue (DX), Music (MX), and Sound Effects (FX) audio tracks or standalone 24-bit 48 kHz WAV files.
4. **Rigid Deliverable Configuration:** The schema defines `"output": "path/to/file.mp4"` as a bare string, offering no declarative vocabulary for codec, profile, pixel format, container, or auxiliary deliverables.

This research paper resolves **Ticket #983** by establishing the architectural roadmap for codec expansion, schema evolution, alpha pipeline support, audio stem dispatching, and performance/licensing governance.

### Core Recommendations Matrix

| Dimension | Current Architecture | Proposed Architecture |
|---|---|---|
| **Video Codecs** | `libx264` (H.264 only) | `libx264`, `prores_ks` (ProRes 422 / 4444), `dnxhd` (DNxHR) |
| **Pixel Formats** | `yuv420p` (8-bit, 4:2:0) | `yuv420p`, `yuv422p10le` (10-bit 4:2:2), `yuva444p10le` (10-bit 4:4:4:4 with alpha) |
| **Canvas Readback Pipe** | 8-bit `rgb24` over pipe | 8-bit `rgb24` (standard) or 8-bit `rgba` (when alpha requested) |
| **Alpha Channel** | Composited over black (`Rgba::BLACK`) | True transparency (unpremultiplied/straight or premultiplied alpha tagged in MOV container) |
| **Audio Codecs** | AAC 160 kb/s stereo | AAC 160 kb/s (MP4), 24-bit LPCM (`pcm_s24le` in MOV / WAV) |
| **Audio Stems** | Single stereo `[mix]` bus | Element-level / Track-level `role` tagging; decoupled audio-only WAV stem export |
| **Schema `output`** | Bare string (`"out/video.mp4"`) | Untagged union: bare string (backward compatible) OR structured `OutputConfig` object |
| **Primary Encoder Backend** | Spawned FFmpeg CLI (GPL) | Spawned FFmpeg CLI (ProRes, DNxHR, and PCM are native LGPL) |

---

## 2. Current State: The Montagent Export & Encoding Pipeline

### 2.1 The Spawned-Never-Linked Boundary (ADR-0009)
Montagent deliberately links neither `libavcodec` nor `libx264`. To eliminate GPL license contagion while ensuring robust process isolation, every render spawns an external `ffmpeg` binary (qualified at startup against the FFmpeg 7.1 floor via ADR-0115). Communication occurs exclusively over standard OS pipes (`pipe:0` for stdin).

### 2.2 The Canvas Readback & Pixel Stripping Bottleneck
The video encoding loop operates in `crates/montagent-render/src/encode.rs` and `crates/montagent-core/src/verbs/render.rs`:
```
Skia Canvas (premultiplied RGBA8888)
       │
       ▼
Canvas::rgb() ──► strips alpha byte, composites over black
       │
       ▼
Pipe (rawvideo, rgb24, width * height * 3 bytes/frame)
       │
       ▼
FFmpeg stdin (pipe:0)
       │
       ▼
libx264 (pix_fmt yuv420p, crf 20, preset medium, threads 5)
       │
       ▼
Deliverable Temp File (.out.mp4.XXXXXX) ──► atomic rename (ADR-0011)
```

In `crates/montagent-render/src/canvas.rs` (lines 1485–1502):
```rust
pub fn rgb(&mut self) -> Option<Vec<u8>> {
    Some(
        self.rgba()?
            .chunks_exact(4)
            .flat_map(|px| [px[0], px[1], px[2]])
            .collect(),
    )
}
```
The canvas readback method explicitly discards the 4th byte (alpha) to save 1 byte per pixel over the subprocess pipe. Furthermore, if the project does not declare a background, `Canvas::BLACK` (`#000000FF`) is painted underneath every frame. Even if an element has opacity or transparency, the deliverable is forced opaque.

In `crates/montagent-render/src/encode.rs` (lines 268–279, 298–310):
- Input arguments: `-f rawvideo -pix_fmt rgb24 -s {width}x{height} -r {fps} -i pipe:0`
- Video filtering: `-vf pad={out_width}:{out_height}:0:0:color=0x{r:02X}{g:02X}{b:02X}` (ADR-0021)
- Encoder arguments: `-c:v libx264 -preset medium -crf 20 -threads 5 -pix_fmt yuv420p -r {fps}`
- Output container: `-movflags +faststart -f mp4`
- Mix bus: `-map [mix] -c:a aac -b:a 160k`

### 2.3 The Single Deliverable Contract
The document model (`crates/montagent-core/src/model/mod.rs`) defines `output` as:
```rust
pub output: Option<String>,
```
Every verb (`render`, `preview`, `verify`) assumes `output` points to a single local filesystem path terminating in `.mp4`.

---

## 3. Question 1: Encoder Backends & Pixel Formats for ProRes 4444 with Alpha

### 3.1 FFmpeg Encoder Backends Survey

FFmpeg contains multiple encoder backends for intermediate mezzanine video formats:

#### 1. Apple ProRes Encoders
| Encoder | Threading | Supported Pixel Formats | Supported Profiles | Licensing | Assessment |
|---|---|---|---|---|---|
| **`prores_ks`** (Kostya) | Frame & Slice | `yuv422p10le`, `yuv444p10le`, `yuva444p10le` | `proxy` (0), `lt` (1), `standard` (2), `hq` (3), `4444` (4), `4444xq` (5) | **LGPL v2.1+** | **Recommended Default.** Excellent multi-threaded slice encoding; supports 10-bit alpha plane via `yuva444p10le`. |
| **`prores`** (Anatoly) | Frame only | `yuv422p10le`, `yuv444p10le`, `yuva444p10le` | `proxy`, `lt`, `standard`, `hq`, `4444` | **LGPL v2.1+** | Slower slice parallelism than `prores_ks`. |
| **`prores_aw`** | Intra | `yuv422p10le` | Standard 422 profiles | LGPL v2.1+ | Deprecated / limited profile support. |
| **`prores_videotoolbox`** | Hardware | `p010le`, `nv12`, `ayuv64le`, `bgra` | `proxy`, `lt`, `standard`, `hq`, `4444`, `xq` | Apple Proprietary Framework | macOS only. Breaks byte determinism across chip generations (ADR-0143). |

#### 2. Avid DNxHR / DNxHD Encoders
| Encoder | Threading | Supported Pixel Formats | Supported Profiles | Licensing | Assessment |
|---|---|---|---|---|---|
| **`dnxhd`** | Frame & Slice | `yuv422p`, `yuv422p10le`, `yuv444p10le`, `gbrp10le` | `dnxhd`, `dnxhr_lb`, `dnxhr_sq`, `dnxhr_hq`, `dnxhr_hqx`, `dnxhr_444` | **LGPL v2.1+** | **Recommended for Avid/Broadcast.** High-performance multi-slice encoder. |

> [!IMPORTANT]
> **DNxHR Alpha Support Limitation:**
> In FFmpeg's `dnxhd` encoder, there is **no pixel format with an alpha channel** (`yuva444p10le` is not supported). The SMPTE VC-3 standard and Avid DNxHR specifications do not encode an interleaved alpha channel inside the primary video stream like ProRes 4444 does. In post-production, Avid workflows handle alpha either via uncompressed QuickTime Animation (RLE), Apple ProRes 4444, or by exporting a separate grayscale alpha matte file (`<name>_matte.mov`).

---

### 3.2 Pixel Format Architecture for ProRes 4444 with Alpha

To export ProRes 4444 with alpha, three stages in Montagent's pipeline must be updated:

```
[Skia Canvas]
      │  Renders with transparent background (Rgba([0, 0, 0, 0]))
      ▼
[Readback Pipe]
      │  Canvas::rgba() yields packed 32-bit RGBA (8-bit/channel premultiplied)
      ▼
[Pipe Input to FFmpeg]
      │  -f rawvideo -pix_fmt rgba -s WxH -r FPS -i pipe:0
      ▼
[FFmpeg Filter & Swscale]
      │  pad=...:color=0x00000000 (preserves alpha)
      │  swscale converts rgba -> yuva444p10le
      ▼
[Encoder & Container]
      │  -c:v prores_ks -profile:v 4444 -pix_fmt yuva444p10le
      │  -vendor apl0 (Apple compatibility)
      ▼
[QuickTime MOV Container (.mov)]
```

#### Detailed Stage Specifications:

1. **Input Pipe Pixel Format:**
   - **Current:** `-pix_fmt rgb24` (24 bits/pixel).
   - **Alpha Export:** `-pix_fmt rgba` (32 bits/pixel, row-major).
   - *Data Volume:* At 1920×1080 @ 30 fps, `rgba` transfers ~248 MB/s over the local anonymous OS pipe (`pipe:0`). Local pipe bandwidth on Unix kernels easily exceeds 2–4 GB/s, so this poses zero IPC bottleneck.
   - *High Bit-Depth Future Note:* If Skia surfaces are ever configured for 10-bit or 16-bit wide color (e.g. `ColorType::RGBAF16` or `ColorType::R10x6G10x6B10x6A10x6`), the pipe format can transition to `rgba64le`. However, since Montagent currently renders in 8-bit sRGB, piping `rgba` and having FFmpeg's `swscale` map 8-bit RGBA to 10-bit `yuva444p10le` is bit-exact and optimal.

2. **Canvas Background Clear:**
   - In `crates/montagent-render/src/canvas.rs`:
     - Standard renders: Default background is `Rgba::BLACK` (`#000000FF`).
     - Alpha renders: When `alpha: true` is configured, the canvas must initialize with transparent clear (`Rgba([0, 0, 0, 0])`), and any omitted project `background` must not paint an opaque backing layer.

3. **Odd Dimension Padding (ADR-0021):**
   - For H.264 `yuv420p`, dimensions must be even.
   - For ProRes 4444 (`yuva444p10le`), chroma subsampling is 4:4:4, meaning odd dimensions are technically permissible by the codec, but macroblock slice alignment (macroblocks are 16×16) benefits from even dimensions.
   - If padding is required, the pad color must be transparent (`color=0x00000000`), rather than the project's background color:
     `-vf pad={out_width}:{out_height}:0:0:color=0x00000000`

4. **Premultiplied vs Straight (Unpremultiplied) Alpha in NLEs:**
   - Skia natively stores and composites pixels as **premultiplied alpha** (`AlphaType::Premul`).
   - In QuickTime MOV containers, NLEs like Adobe Premiere Pro, Apple Final Cut Pro, and DaVinci Resolve interpret alpha based on container atom metadata (`colr`, `clap`, and the video sample description extension).
   - If a premultiplied file is imported as straight alpha, dark fringing occurs around semi-transparent edges.
   - *Architecture Decision:* Montagent should either:
     - Tag the QuickTime container with premultiplied alpha metadata (`-movflags write_colr` and alpha mode atoms), or
     - Pass the stream through FFmpeg's `unpremultiply` filter (`-vf unpremultiply=inplace=1`) before encoding to `yuva444p10le`. Premiere Pro defaults to straight alpha for ProRes 4444 unless tagged otherwise; providing clean unpremultiplied RGB channels is the gold standard for compositing exchange.

5. **QuickTime Compatibility Tagging:**
   - By default, FFmpeg's `prores_ks` writes vendor ID `"Lavc"`.
   - Apple QuickTime and Final Cut Pro can reject hardware acceleration or display warning badges for non-Apple vendor tags.
   - Using `-vendor apl0` ensures maximum compatibility across the Apple post-production ecosystem.

---

## 4. Question 2: Schema Evolution for `output` (Zero-Breaking-Change Design)

### 4.1 Backward Compatibility Principles
Montagent's existing test fixtures, user scripts, and benchmarks declare output as a single string:
```json
{
  "frame": { "width": 1920, "height": 1080 },
  "fps": 30,
  "output": "out/final.mp4",
  "tracks": [ ... ]
}
```
Under **zero-breaking-change** constraints:
1. Every existing project declaring `"output": "out/final.mp4"` must continue to validate against `schema/montagent.schema.json` with zero warnings or errors.
2. An absent codec configuration must produce the exact, ratified H.264 / AAC deliverable (ADR-0077, ADR-0143).
3. The CLI argument `montagent render <project> --output <path>` must continue to accept path strings.

### 4.2 Proposed Schema Specification

In `schema/montagent.schema.json`, `output` is expanded from `{"type": "string"}` to a schema union using `oneOf`:

```json
"output": {
  "oneOf": [
    {
      "type": "string",
      "description": "Shorthand deliverable path. Defaults to H.264/AAC in MP4 container."
    },
    {
      "$ref": "#/$defs/OutputConfig"
    }
  ]
}
```

Under `$defs`:

```json
"$defs": {
  "OutputConfig": {
    "type": "object",
    "additionalProperties": false,
    "properties": {
      "path": {
        "type": "string",
        "description": "Primary deliverable file path, relative to the project directory."
      },
      "container": {
        "type": "string",
        "enum": ["mp4", "mov", "mxf"],
        "description": "Container format. If omitted, inferred from `path` extension."
      },
      "video": {
        "$ref": "#/$defs/VideoExportConfig"
      },
      "audio": {
        "$ref": "#/$defs/AudioExportConfig"
      },
      "stems": {
        "type": "array",
        "items": {
          "$ref": "#/$defs/StemExportConfig"
        },
        "description": "Auxiliary audio stem deliverables to export alongside the master."
      }
    },
    "required": ["path"]
  },

  "VideoExportConfig": {
    "type": "object",
    "additionalProperties": false,
    "properties": {
      "codec": {
        "type": "string",
        "enum": ["h264", "prores", "dnxhr"]
      },
      "profile": {
        "type": "string",
        "description": "Codec profile: for prores: proxy, lt, standard, hq, 4444, 4444xq; for dnxhr: lb, sq, hq, hqx, 444; for h264: high, main."
      },
      "alpha": {
        "type": "boolean",
        "default": false,
        "description": "Export alpha (transparency) channel. Requires prores 4444 in MOV container."
      }
    },
    "required": ["codec"]
  },

  "AudioExportConfig": {
    "type": "object",
    "additionalProperties": false,
    "properties": {
      "codec": {
        "type": "string",
        "enum": ["aac", "pcm_s16le", "pcm_s24le"]
      },
      "bitrate": {
        "type": "string",
        "description": "Compressed bitrate for AAC (e.g. '160k', '320k'). Ignored for uncompressed PCM."
      }
    },
    "required": ["codec"]
  },

  "StemExportConfig": {
    "type": "object",
    "additionalProperties": false,
    "properties": {
      "stem": {
        "type": "string",
        "description": "Name of the stem matching element/track roles: 'dialogue', 'music', 'sfx', 'm_and_e', or 'mix'."
      },
      "path": {
        "type": "string",
        "description": "Output path for this stem WAV file, relative to the project directory."
      },
      "codec": {
        "type": "string",
        "enum": ["pcm_s16le", "pcm_s24le"],
        "default": "pcm_s24le"
      }
    },
    "required": ["stem", "path"]
  }
}
```

### 4.3 Rust Data Model Mapping

In `crates/montagent-core/src/model/mod.rs`:
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OutputDeclaration {
    Path(String),
    Config(Box<OutputConfig>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutputConfig {
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub container: Option<ContainerFormat>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub video: Option<VideoExportConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: Option<AudioExportConfig>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stems: Vec<StemExportConfig>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContainerFormat {
    Mp4,
    Mov,
    Mxf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VideoExportConfig {
    pub codec: VideoCodec,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
    #[serde(default)]
    pub alpha: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VideoCodec {
    H264,
    ProRes,
    DnxHr,
}
```

### 4.4 Static Validation Rules (`montagent validate`)

The validation engine (`crates/montagent-core/src/checks/`) should enforce strict consistency:
- `E-OUTPUT-ALPHA-UNSUPPORTED`: `alpha: true` is requested on an incompatible codec or container (e.g. `codec: "h264"` or `codec: "dnxhr"` or `container: "mp4"`). Alpha is valid exclusively with `codec: "prores"` and `profile: "4444"` or `"4444xq"` in a `.mov` container.
- `E-OUTPUT-CODEC-CONTAINER-MISMATCH`: ProRes or DNxHR cannot be muxed into an MP4 container (must be MOV or MXF).
- `E-OUTPUT-STEM-UNKNOWN`: A stem specifies a role name (e.g. `"foley"`) that matches no audio elements or tracks in the project.
- `E-OUTPUT-STEM-PATH-COLLISION`: Two stems or a stem and the master deliverable target the identical filesystem path.

### 4.5 Impact on `montagent verify`

Currently, `montagent-core::verbs::verify` verifies:
- Attestation stamp in container comment (ADR-0104)
- Frame dimensions (padded to even)
- Video PTS interval consistency and frame count
- Mix bus loudness and true peak

When extended:
1. **Container & Codec Verification:** `Structure::of` probes the deliverable's video format. For ProRes deliverables, it verifies stream codec `prores`, expected profile ID, 10-bit pixel format, and QuickTime atom tags.
2. **Alpha Verification:** For `alpha: true` exports, verify probes the video stream's pixel format (`yuva444p10le`) and asserts the existence of non-255 alpha values in sampled decoded frames.
3. **Audio Format Verification:** Verifies 24-bit PCM (`pcm_s24le`) when configured instead of defaulting to AAC.
4. **Stem Deliverable Verification:** When `stems` are declared, `verify` independently validates that each stem WAV file exists, carries uncorrupted LPCM 48 kHz audio, and matches the project's exact extent.

---

## 5. Question 3: Audio-Only Stem Export Declaration and Dispatch

### 5.1 The Post-Production Need for Stems

In professional post-production workflows:
- **Dialogue (DX):** Spoken voices, narration, VO, ADR.
- **Music (MX):** Score, source music, background cues.
- **Sound Effects (FX / SFX):** Foley, ambient backgrounds, hard effects.
- **M&E (Music & Effects):** The complete mix with dialogue removed, essential for foreign language localization.

Premiere Pro, Pro Tools, and DaVinci Resolve exchange stems as standalone **Broadcast WAV (BWF) files, 24-bit, 48 kHz stereo or 5.1 multichannel**.

---

### 5.2 Declarative Stem Assignment

To associate elements with stems, Montagent should introduce an optional `role` field on `audio` and `video` elements (and optionally at the `track` level for bulk assignment):

```json
{
  "type": "audio",
  "id": "narration_take_3",
  "source": "audio/vo.wav",
  "role": "dialogue",
  "start": 1000,
  "end": 6500,
  "volume": 0.9
}
```

```json
{
  "type": "audio",
  "id": "intro_music",
  "source": "audio/bed.mp3",
  "role": "music",
  "start": 0,
  "end": 15000,
  "volume": 0.4
}
```

**Default Rules:**
- If an element specifies an explicit `role`, that role is used.
- If an element omits `role`, it inherits the `role` declared on its enclosing `track`.
- If neither declares a role, it defaults to `"sfx"`.
- The special stem name `"m_and_e"` automatically pools all elements whose role is NOT `"dialogue"`.
- The special stem name `"mix"` generates the full mix sum as an uncompressed WAV.

---

### 5.3 Dispatch Architecture: Single-Pass vs Decoupled Audio Passes

Two candidate dispatch architectures were evaluated:

```
┌────────────────────────────────────────────────────────────────────────┐
│ Option A: Single FFmpeg Process with Multi-Output Filtergraph          │
│                                                                        │
│  [Video pipe:0] ───────────────► Video Filter ──► master.mov           │
│  [Audio inputs 1..N] ──► Filter ┬───────────────► (master audio)       │
│                                 ├─► [dialogue]  ──► dialogue.wav       │
│                                 ├─► [music]     ──► music.wav          │
│                                 └─► [sfx]       ──► sfx.wav            │
└────────────────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────────────────┐
│ Option B: Decoupled Passes (Recommended)                               │
│                                                                        │
│  Pass 1: Main Deliverable (Video + Master Audio Mix)                   │
│          Skia Painter ──► pipe:0 ──► FFmpeg ──► master.mov             │
│                                                                        │
│  Pass 2..M: Audio Stem Dispatches (Audio-Only, No Skia Painter)         │
│          Audio Inputs ──► FFmpeg (-vn) ──► dialogue.wav                │
│          Audio Inputs ──► FFmpeg (-vn) ──► music.wav                   │
│          Audio Inputs ──► FFmpeg (-vn) ──► sfx.wav                     │
└────────────────────────────────────────────────────────────────────────┘
```

#### Evaluation:

| Criterion | Option A: Monolithic Multi-Output | Option B: Decoupled Passes |
|---|---|---|
| **Atomic Rename (`Deliverable`)** | Extremely complex; single failure rolls back video and all stems. | Clean: each stem and the video use standard [`Deliverable`] temporary siblings independently. |
| **Render Speed** | Slightly faster disk read; decodes audio inputs once. | Audio mixing takes < 0.5s total; difference is undetectable on wall-clock. |
| **Process Isolation** | A crash or syntax error in stem filtergraph halts the entire video render. | Total isolation: master video render is unaffected by stem errors. |
| **Selective Execution** | Cannot render stems without rendering all video frames. | Supports `montagent render --stems-only`, generating stems in seconds without Skia rasterization! |

### 5.4 Recommended Implementation: Decoupled Stem Dispatch

Because audio mixing in Montagent (`crates/montagent-core/src/verbs/render.rs`) is already fully isolated into `Mix::of`, generating a stem is straightforward:
1. Filter the document elements where `element.role == target_stem` (or `role != "dialogue"` for M&E).
2. Call `Mix::of` on the filtered element subset.
3. Spawn an audio-only FFmpeg invocation:
   ```bash
   ffmpeg -hide_banner -loglevel error -y \
     -i input1.wav -i input2.mp3 ... \
     -filter_complex_script .filters.stem.tmp \
     -map "[mix]" \
     -c:a pcm_s24le \
     -ar 48000 \
     out/.dialogue.wav.XXXXXX
   ```
4. Atomically commit each stem via `Deliverable::commit()`.
5. Report individual stem byte counts, paths, and durations in the render response JSON.

---

## 6. Question 4: Performance, Hardware Acceleration, and Licensing Implications

### 6.1 Licensing Implications

| Component | License | GPL Contagion Risk | Notes |
|---|---|---|---|
| **`libx264` (H.264)** | **GPL v2+** | Isolated via process pipe (ADR-0009). | Requires GPL-enabled FFmpeg build. |
| **`prores_ks` (ProRes)** | **LGPL v2.1+** | **Zero.** Native FFmpeg C implementation. | Available in standard LGPL FFmpeg builds. |
| **`dnxhd` (DNxHR)** | **LGPL v2.1+** | **Zero.** Native FFmpeg C implementation. | Available in standard LGPL FFmpeg builds. |
| **`pcm_s24le` (WAV)** | **LGPL v2.1+** | **Zero.** Native FFmpeg raw audio muxer. | Universally available in all builds. |

#### Apple ProRes Trademark & Patent Governance:
- Apple owns patents covering the ProRes bitstream and maintains the "Apple ProRes Authorized Products" licensing program.
- FFmpeg's `prores_ks` was reverse-engineered by open-source contributors without Apple's proprietary source code.
- Because Montagent spawns FFmpeg as an independent CLI tool rather than distributing linked proprietary or GPL libraries (ADR-0009), Montagent assumes zero direct copyright or licensing liabilities.
- To prevent compatibility friction in broadcast post-production facilities, setting the QuickTime vendor tag `-vendor apl0` ensures that Apple software treats files as standard ProRes.

---

### 6.2 Performance: Software Intra-Frame Encoding vs Inter-Frame H.264

A widespread misconception is that intermediate codecs like ProRes and DNxHR are computationally heavier than distribution codecs like H.264. In reality, the reverse is true:

```
Computational Complexity per Frame:

H.264 (libx264 crf 20 medium):
[DCT] ──► [Motion Estimation] ──► [Motion Compensation] ──► [Rate Control / CABAC]
(Very high CPU usage: ~15-30ms per 1080p frame)

ProRes 422 / 4444 (prores_ks):
[DCT] ──► [Fixed Quantization Matrix] ──► [Variable Length Coding]
(Extremely low CPU usage: ~2-4ms per 1080p frame, purely intra-frame)
```

- **CPU Utilization:** `prores_ks` encodes 4–6× faster than `libx264` at preset `medium` because it performs no temporal motion search between frames.
- **I/O Bottleneck Shift:** Because ProRes 4444 bitrates are substantial (~330 Mbps for 1080p 30fps; ~1.3 Gbps for 4K 30fps), the rendering bottleneck shifts completely away from CPU encoding to:
  1. Skia canvas draw call execution and readback.
  2. Memory throughput for frame allocation and pipe writing.
  3. Disk write bandwidth for the output deliverable.

---

### 6.3 Hardware Acceleration: VideoToolbox (`prores_videotoolbox`)

On macOS, Apple Silicon chips (M1/M2/M3/M4 Pro/Max/Ultra) include dedicated hardware ProRes encoding and decoding engines accessible via Apple's VideoToolbox framework.

#### Evaluation against ADR-0143:
In ADR-0143 §2, VideoToolbox was rejected for H.264 encoding due to:
1. Lack of CRF mode (bitrate-based rate control yields fluctuating quality).
2. Substantially lower VMAF quality scores compared to `libx264`.
3. Complete lack of byte determinism across macOS versions and Apple Silicon revisions.
4. macOS-only availability, creating an asymmetric parity gap with Linux and Windows render nodes.

#### Does this apply to ProRes?
- Unlike H.264, **ProRes is an intra-frame DCT format with fixed quantization targets per profile**. Quality between `prores_videotoolbox` and `prores_ks` is virtually identical.
- However, **byte determinism** remains an issue: bitstreams produced by `prores_videotoolbox` will vary across macOS 14, 15, and between M1 and M4 hardware blocks.
- Furthermore, because software `prores_ks` is already so fast that it outpaces Skia's rasterization speed, hardware offloading provides negligible real-world throughput gain while introducing platform fragmentation.

**Recommendation:**
1. Use **`prores_ks` as the universal, deterministic, cross-platform software encoder floor** across macOS, Linux, and Windows.
2. Pin encoder thread parameters in `montagent-render::encode` consistently with ADR-0143.
3. Reserve hardware acceleration as an optional future CLI optimization flag (`--hwaccel videotoolbox`), never as an untracked default.

---

## 7. Recommended Implementation Roadmap & Spec

### Phase 1: Alpha-Capable Canvas & Readback (`montagent-render`)
1. Add `Canvas::rgba_straight()` or update `Canvas::rgba()` to provide clean, uncomposited 32-bit buffers.
2. Update `montagent-render::canvas` to support transparent canvas clearing (`Rgba([0, 0, 0, 0])`) when alpha is requested.
3. Update `montagent-render::encode`:
   - Accept `pix_fmt: "rgba"` on stdin.
   - Adjust odd-dimension padding to use `color=0x00000000`.

### Phase 2: Codec & Profile Abstraction in Encoder (`montagent-render`)
1. Expand `encode::Settings`:
   ```rust
   pub enum CodecSettings {
       Libx264 { preset: &'static str, crf: u8, threads: NonZeroU32 },
       ProRes { profile: ProResProfile, alpha: bool, vendor: &'static str },
       DnxHr { profile: DnxHrProfile },
   }
   ```
2. Implement argument builder for `prores_ks`:
   `-c:v prores_ks -profile:v 4 -pix_fmt yuva444p10le -vendor apl0`
3. Support `.mov` container output with QuickTime faststart atoms (`-movflags +faststart`).

### Phase 3: Project Schema & Model Expansion (`montagent-core`)
1. Update `schema/montagent.schema.json` with polymorphic `output` (`OutputConfig`).
2. Update `crates/montagent-core/src/model/mod.rs` with `OutputDeclaration`.
3. Add validation findings:
   - `E-OUTPUT-ALPHA-UNSUPPORTED`
   - `E-OUTPUT-CODEC-CONTAINER-MISMATCH`
   - `E-OUTPUT-STEM-UNKNOWN`

### Phase 4: Audio Stem Dispatcher (`montagent-core`)
1. Add `role` field to `audio` and `video` elements in `montagent.schema.json`.
2. Implement `StemProducer` in `crates/montagent-core/src/verbs/render.rs`:
   - Partitions audio graph by stem role.
   - Renders 24-bit 48 kHz WAV files (`pcm_s24le`) in fast audio-only FFmpeg passes.
   - Discloses generated stem files in `render` answer report.

### Phase 5: Verification & Deliverable Measurement (`montagent verify`)
1. Extend `crates/montagent-core/src/verbs/verify.rs` to probe:
   - QuickTime MOV metadata and ProRes profile tags.
   - Decoded alpha plane values for alpha exports.
   - Stem file existence, formatting, duration, and peak levels.

---

## 8. Conclusion

Expanding Montagent's export architecture beyond H.264 is technically straightforward and aligns cleanly with Montagent's foundational ADRs:
- **`prores_ks` and `dnxhd` are native LGPL FFmpeg components**, preserving the clean legal separation established in ADR-0009.
- **`rgba` pipe transmission** leverages existing Skia surface capabilities while maintaining high IPC efficiency.
- **Untagged schema polymorphism** guarantees 100% backward compatibility for all existing projects.
- **Decoupled stem dispatch** ensures fast, isolated, and robust audio asset mastering.

This architecture bridges the final gap between Montagent's declarative automation engine and professional NLE/DAW post-production finishing pipelines.
