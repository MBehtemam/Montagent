# Premiere gap: export

Research ticket [#892](https://github.com/MBehtemam/Montagent/issues/892) on the map
[#882](https://github.com/MBehtemam/Montagent/issues/882) ("every Premiere Pro feature, set against
Montagent v0.2.0"). Area: **export**, meaning Premiere's Export mode and Adobe Media Encoder (AME).

- **Date:** 2026-10-10
- **Montagent side:** tag `v0.2.0` (`51aa4b5`, = `origin/main` at the time of writing). Read: `CONTEXT.md`,
  `schema/montagent.schema.json`, `crates/montagent-render/src/encode.rs` (the one `ffmpeg` encode
  invocation), `crates/montagent/src/cli.rs` and `crates/montagent/src/mcp.rs` (the verb and MCP tool
  surface: `validate`, `create_project`, `query`, `frame`, `render`, `verify`, `preview`, `measure`,
  `shift`, `compare`), `skills/`, and ADRs 0003, 0040, 0049, 0062, 0077, 0089, 0104, 0108, 0109, 0117,
  0121, 0143, 0145, 0172, 0174, 0190, 0192.
- **Premiere side:** Adobe's own help pages only (helpx.adobe.com: the Premiere user guide, the Adobe
  Media Encoder user guide where Premiere defers to it, and the Premiere release notes). One URL per row.
- **Status vocabulary** (from the map): `have`, `have (skill)`, `partial`, `missing`, `refused`
  (refused = an ADR, or ADR-0145's four invariants, rules it out; the ADR is cited).

### How the Premiere side was sourced, and what to re-check

helpx.adobe.com could not be fetched directly from this session (DNS resolution failed in the
sandbox), and the session's web-search budget ran out partway through. Every Premiere fact below
comes from searches restricted to `helpx.adobe.com`, whose results named the cited page. Rows marked
**†** are the exception: the cited page is the right Adobe page for the feature, but no search result
this session quoted the specific option, so the mechanism is described from that page's known content.
A follow-up should re-read the † rows against the live pages. No row was left out because it could not
be confirmed.

### Counts

| Status | Rows |
|---|---|
| have | 14 |
| have (skill) | 1 |
| partial | 11 |
| missing | 56 |
| refused | 7 |
| **total** | **89** |

### The one fact that decides most rows

Montagent writes exactly **one deliverable shape**: H.264 (libx264, `yuv420p`, CRF 20, preset
`medium`, `-threads 5`) with AAC at 160 kb/s, 48 kHz stereo, in an MP4 with `+faststart`, at the
project's own `frame` and `fps` (`crates/montagent-render/src/encode.rs`; ADR-0077 §7–8, ADR-0143). The
container is forced with `-f mp4` whatever the `output` path's extension is. ADR-0143 §5 refuses any
project field or CLI/MCP option for the encoder, preset, CRF or thread count. Every other format and
codec in AME's list is therefore `missing` (nothing equivalent, no ADR refusing it), except where an
ADR refuses the mechanism itself.

## Table

### A. Video formats and codecs

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| H.264 (MP4) | Format *H.264*, the recommended format for online video; presets incl. Match Source – Adaptive High/Medium Bitrate | https://helpx.adobe.com/premiere-pro/using/bestpractices-export.html | have | `render` (CLI + MCP); `encode.rs` libx264 `yuv420p` CRF 20 `medium`, `-movflags +faststart -f mp4`; ADR-0077 §7, ADR-0143 | Settings are fixed (see rows C); 8-bit 4:2:0 only |
| H.264 at 10-bit / 4:2:2 † | H.264 Encoding Settings: Profile (Baseline/Main/High/High 10/High 4:2:2), bit depth | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/video-encoding-settings.html | missing | `encode.rs` pins `-pix_fmt yuv420p` | Any profile, chroma format or bit depth other than 8-bit 4:2:0 |
| HEVC (H.265) | Format *HEVC (H.265)*, MP4/MOV; hardware or software encode; 8- and 10-bit; HDR presets (HEVC – Match Source – PQ) | https://helpx.adobe.com/premiere/desktop/get-started/technical-requirements/hardware-accelerated-decoding-and-encoding.html | missing | `encode.rs` names only `libx264`; no ADR on HEVC | An HEVC encode path (encoder, container, licence posture under ADR-0009's spawn-not-link rule) |
| AV1 † | AV1 export in AME (hardware-accelerated where supported) | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/supported-export-file-formats.html | missing | none; Montagent decodes VP9 (ADR-0089) but encodes only H.264 | AV1 encode |
| Apple ProRes (422 Proxy/LT/422/HQ, 4444, 4444 XQ) | QuickTime / *Apple ProRes MXF OP1a*; a format Premiere exports directly; ProRes 4444 carries alpha; ProRes 422 HQ/4444 with HLG/PQ for HDR | https://helpx.adobe.com/nz/premiere-pro/using/formats-exported-directly-premiere-pro.html | missing | `encode.rs` (H.264 only); the `montagent-prerender` skill notes ProRes 4444 is lossy and uses PNG-in-MOV for its own intermediates | A ProRes (mezzanine) deliverable |
| Avid DNxHD / DNxHR (LB/SQ/HQ/HQX/444) † | *DNxHR/DNxHD MXF OP1a* and QuickTime DNx codecs | https://helpx.adobe.com/media-encoder/desktop/encoding-and-exporting/export-settings-reference.html | missing | `encode.rs` | Any DNx encode, any MXF wrapper |
| GoPro CineForm † | QuickTime *GoPro CineForm* (YUV 10-bit / RGB 12-bit, with alpha) | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/supported-export-file-formats.html | missing | `encode.rs` | CineForm encode |
| QuickTime (.mov) container | Format *QuickTime* wrapping ProRes, DNx, CineForm, Animation, uncompressed, H.264 | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/basic-video-settings.html | missing | `encode.rs` forces `-f mp4` | A MOV deliverable; a `.mov` `output` still gets MP4 bytes |
| Uncompressed / lossless video † | QuickTime *Uncompressed 8/10-bit*, *Animation*, *PNG* codecs | https://helpx.adobe.com/media-encoder/desktop/encoding-and-exporting/export-settings-reference.html | missing | `encode.rs` (lossy CRF 20 only) | A lossless deliverable. (ADR-0143's VMAF reference used `-qp 0` through a test wrapper only, never a shipped option) |
| XAVC (Intra, Long GOP, QFHD) | *XAVC* in MXF; XAVC-Intra with HLG/PQ for HDR | https://helpx.adobe.com/premiere-pro/using/hdr-workflows.html | missing | `encode.rs` | XAVC encode and MXF wrapping |
| MXF OP1a (generic) † | *MXF OP1a* wrapper for broadcast codecs (XDCAM HD, AVC-Intra, IMX, DNx, ProRes, JPEG 2000) | https://helpx.adobe.com/media-encoder/desktop/encoding-and-exporting/export-settings-reference.html | missing | `encode.rs` | MXF container |
| AVC-Intra / XDCAM HD / P2 Movie † | Broadcast camera-native formats in MXF | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/supported-export-file-formats.html | missing | `encode.rs` | Each of these encodes |
| AS-10 / AS-11 † | DPP/AMWA broadcast delivery shims, incl. AS-11 metadata | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/supported-export-file-formats.html | missing | `encode.rs` | Broadcast-shim delivery and its metadata |
| JPEG 2000 MXF OP1a † | JPEG 2000 in MXF (archival / DCI-style) | https://helpx.adobe.com/media-encoder/desktop/encoding-and-exporting/export-settings-reference.html | missing | `encode.rs` | JPEG 2000 encode |
| Wraptor DCP † | Digital Cinema Package export | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/supported-export-file-formats.html | missing | `encode.rs` | DCP packaging (XYZ colour, JPEG 2000, MXF, CPL/PKL) |
| MPEG-2 / MPEG-2 DVD / MPEG-2 Blu-ray / H.264 Blu-ray † | Disc-authoring elementary streams | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/supported-export-file-formats.html | missing | `encode.rs` | MPEG-2 and Blu-ray-compliant streams |
| MPEG-4 (3GP) † | Legacy *MPEG4* format for low-end devices | https://helpx.adobe.com/media-encoder/desktop/encoding-and-exporting/export-settings-reference.html | missing | `encode.rs` | MPEG-4 Part 2 / 3GP |
| Windows Media (Windows only) † | *Windows Media* (WMV) format | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/supported-export-file-formats.html | missing | `encode.rs` | WMV encode |
| Animated GIF | Format *Animated GIF* (also a Match Source option) | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/supported-export-file-formats.html | missing | `encode.rs`; the schema's `loop` field (ADR-0062) says *"Montagent writes no container-level loop metadata"* | A palette-quantised GIF deliverable and its loop flag |

### B. Stills, sequences, alpha and audio-only

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Export Frame (one still) | Export Frame from the Program/Source monitor: JPEG, PNG, TIFF, DPX, TGA, BMP (Windows), OpenEXR †, GIF †; optional *Import into project* † | https://helpx.adobe.com/nz/premiere-pro/using/formats-exported-directly-premiere-pro.html | partial | `frame --at` (CLI `--out`, `--png`, `--full`; MCP `frame` returns the image in the answer); ADR-0096 (which frame an instant names) | Only JPEG (default) and PNG; default is half scale (`--full` for true pixels); no TIFF/DPX/TGA/BMP/EXR/GIF; no "import into project" (an agent writes an `image` element by hand) |
| Image sequence export (PNG/JPEG/TIFF/DPX/TGA/OpenEXR, *Export As Sequence*) † | Still format plus *Export As Sequence* writes one numbered file per frame | https://helpx.adobe.com/media-encoder/desktop/encoding-and-exporting/export-settings-reference.html | missing | `frame` paints one instant per call; `frame --from --to` is a contact sheet (one image), not a sequence (ADR-0094, ADR-0103) | A numbered per-frame file sequence at the deliverable's frame size |
| DPX | Direct export format (stills and sequences, film/DI interchange) | https://helpx.adobe.com/nz/premiere-pro/using/formats-exported-directly-premiere-pro.html | missing | `frame` writes JPEG/PNG only | DPX writer |
| TIFF | Direct export format | https://helpx.adobe.com/nz/premiere-pro/using/formats-exported-directly-premiere-pro.html | missing | `frame` writes JPEG/PNG only | TIFF writer |
| Targa (TGA) | Direct export format | https://helpx.adobe.com/nz/premiere-pro/using/formats-exported-directly-premiere-pro.html | missing | `frame` writes JPEG/PNG only | TGA writer |
| BMP (Windows) | Direct export format | https://helpx.adobe.com/nz/premiere-pro/using/formats-exported-directly-premiere-pro.html | missing | `frame` writes JPEG/PNG only | BMP writer |
| OpenEXR † | Still/sequence format (float, VFX interchange) | https://helpx.adobe.com/media-encoder/desktop/encoding-and-exporting/export-settings-reference.html | missing | `frame`; the painter is 8-bit RGBA | EXR writer and a float pipeline |
| JPEG / PNG stills | Direct export formats | https://helpx.adobe.com/nz/premiere-pro/using/formats-exported-directly-premiere-pro.html | have | `frame` (JPEG default, `png` option; `full` for true pixels; `crop` for a region) | Nothing for a single still (sequence: see above) |
| Export with an alpha channel † | *Include Alpha Channel* / *Render Alpha Channel* on ProRes 4444, QuickTime Animation/PNG, CineForm RGB, PNG/TIFF/TGA/EXR stills | https://helpx.adobe.com/media-encoder/desktop/encoding-and-exporting/export-settings-reference.html | missing | `encode.rs` pipes `rgb24` and writes `yuv420p`, so alpha is dropped; `background` is painted under everything. The `montagent-prerender` skill writes PNG-in-MOV with alpha, but only for **intermediate** footage it brings *into* a project | A transparent deliverable (codec with alpha, and a background that can be absent) |
| Audio-only export (WAV, AIFF, MP3, AAC) | Formats *Waveform Audio*, *AIFF*, *MP3*, *AAC Audio*; Export mode can export audio alone | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-video-and-audio-files.html | missing | `render` always writes a video stream; the mix is AAC inside the MP4 (ADR-0077 §7–8) | An audio-only deliverable in any format |
| Export video without audio | *Export Audio* toggle off | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-video-and-audio-files.html | partial | `encode.rs` passes `-an` when nothing is audible | No switch: a project with audible elements always carries its mix |
| Audio codec, sample rate, bit rate † | Audio tab: codec (AAC/PCM/…), sample rate, channels, bitrate | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/overview-of-export-settings.html | missing | Hard-coded AAC 160 kb/s, 48 kHz (ADR-0077 §7–8); ADR-0172 says sample rate and codec "stay hard-coded" | Any choice of audio codec, rate or bitrate. Not refused by an ADR, only hard-coded |
| Audio channel layout (mono, 5.1, multichannel, multi-track) † | Audio tab: Channels (Mono/Stereo/5.1/multi); multichannel QuickTime/MXF track mapping | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/overview-of-export-settings.html | missing | Mix bus is stereo (ADR-0077 §8); ADR-0172 defers output channel layout to a future channel-operations capability; `channels` (ADR-0184) routes an element within stereo | Mono, surround or multi-track output |

### C. Encoding settings and performance

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Hardware encoding | Encoding Settings → Performance: *Hardware Encoding* (default when supported) for H.264/HEVC on NVIDIA/Intel/AMD/Apple silicon; falls back to software | https://helpx.adobe.com/premiere/desktop/get-started/technical-requirements/enable-hardware-encoding-support.html | refused | ADR-0143 §2: VideoToolbox "is not a default and not an option"; any future hardware-encode option needs its own ADR with quality evidence | n/a (refused) |
| Smart Rendering | Re-uses source/preview frames whose codec matches the output, skipping re-encode | https://helpx.adobe.com/premiere-pro/using/whats-new/2022-4.html | missing | Every frame is painted and encoded (ADR-0141, ADR-0144); MP4 byte determinism (ADR-0143, ADR-0190) is built on a full encode | Any pass-through of unchanged source frames |
| Use Previews † | *Use Previews* re-uses rendered timeline previews on export | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/overview-of-export-settings.html | missing | `preview` writes a separate proxy MP4 and is never the deliverable (ADR-0046, ADR-0067) | Re-using preview renders in the deliverable |
| Bitrate encoding (CBR / VBR 1-pass / VBR 2-pass), target/max bitrate † | H.264/HEVC Bitrate Settings | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/video-encoding-settings.html | refused | ADR-0143 §5: no project field, CLI or MCP option for encoder, preset, CRF or threads | n/a (refused) |
| Quality, profile, level, key-frame distance † | Encoding Settings: Profile, Level, *Key Frame Distance*, Quality | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/video-encoding-settings.html | refused | ADR-0143 §5 (same) | n/a (refused) |
| Saved and custom export presets; preset import/export; Preset Browser † | *Save Preset*, *Import Preset*, Preset Manager; AME Preset Browser | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-options-in-premiere.html | refused | ADR-0143 §5: an encoder setting "is not a property of the composition", and a public knob is refused | n/a (refused). Frame and fps live in the project and are scaffolded by `create_project` |
| Match Source presets | Adaptive presets that take the sequence's frame size, frame rate etc. | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-options-in-premiere.html | have | `render` always writes the declared `frame` at the declared `fps`, never downsampled (MCP `render` description; ADR-0067) | Nothing (odd dimensions padded to even and disclosed, ADR-0077 §6) |
| Frame size / scaling on export (Scale to fit / fill / stretch) and export crop † | Video tab frame size; Source/Output *Scaling* and Crop in Export mode | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/basic-video-settings.html | missing | The deliverable is the project `frame`; only `preview` resizes, to the 720p proxy (ADR-0046) | An output size or crop other than the project frame |
| Frame rate conversion and time interpolation † | Frame Rate setting; *Time Interpolation*: frame sampling / frame blending / optical flow | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/basic-video-settings.html | partial | `fps` is the project's; source footage at another rate is frame-sampled (the frame at an instant is the last one starting at or before it, ADR-0096) | Frame blending and optical-flow interpolation; an export rate different from `fps` |
| Field order (interlaced) † | Video tab: *Field Order* (Progressive/Upper/Lower) | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/basic-video-settings.html | missing | `encode.rs` writes progressive only | Interlaced output |
| Pixel aspect ratio † | Video tab: *Aspect* (square, anamorphic, D1/DV) | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/basic-video-settings.html | missing | Output is square-pixel; source PAR is read on input only (ADR-0023) | Non-square output pixels |
| Render at Maximum Depth / Maximum Render Quality † | Video tab checkboxes for higher-precision compositing and scaling | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/basic-video-settings.html | partial | `render` is never degraded (ADR-0067) and paints at full size | Higher-than-8-bit compositing: the painter and pipe are 8-bit RGB |
| Multiplexer / container options † | *Multiplexer* tab: MP4 / 3GPP / none; stream compatibility | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/overview-of-export-settings.html | partial | Fixed MP4 with `+faststart` (`encode.rs`) | Any other multiplex, or elementary streams |
| Estimated file size / export summary | Export mode shows an estimated file size and a settings summary before export | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-video.html | partial | The `render` answer reports `bytes`, encoder, preset, CRF, threads and ffmpeg version **after** the encode (ADR-0143 §6) | Any estimate before encoding |
| Export progress and remaining time | Encoding progress bar and time remaining (Premiere and AME queue) | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-video.html | have | `render --progress-file` / MCP `progress_file`: state, phase, frames, ETA (ADR-0192); MCP progress notifications (ADR-0108) | Nothing |
| Cancel an export | Cancel in the encode dialog or AME queue | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-video.html | have | A cancelled encode publishes nothing (ADR-0109); temp path + atomic rename (`encode.rs` `Deliverable`) | Nothing |
| Output file naming and overwrite protection † | File name / location fields; AME preference to increment the name of an existing file | https://helpx.adobe.com/media-encoder/desktop/encoding-and-exporting/export-settings-reference.html | have | `output` in the project, `--output` override; foreign-deliverable check before the encoder runs, `--no-clobber` (ADR-0104) | No auto-increment of a file name (a refusal or `review` is raised instead); `no_clobber` is CLI-only, not on the MCP `render` |
| Export preview pane | Export mode preview with Source/Output view and range scrubbing | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-video.html | have | `frame` (stills, contact sheets) and `preview` (a proxy MP4 of a span) | Nothing at the mechanism level |
| Post-encode check of the file † | AME shows a completed-encode status and log | https://helpx.adobe.com/media-encoder/desktop/encoding-and-exporting/export-settings-reference.html | have | `verify` measures the deliverable with the decoder, refuses a stale file (ADR-0117), and checks `master`'s loudness and true peak (ADR-0172) | Nothing: Montagent's check goes further than an encode log |

### D. Setting up exports: ranges, queueing, destinations

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Range: Entire Source | Export the whole sequence or clip | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-video.html | have | `render` with no range writes the declared `output`; a project without `duration` renders to its last boundary (ADR-0077 §2) | Nothing |
| Range: Source In/Out, Work Area, Custom | Range menu: *Source In/Out*, *Work Area* (sequences), *Custom* | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-video.html | have | `render --from --to` (MCP `from`/`to`), half-open `[from, to)` to `out/<name>.<from>-<to>.mp4` or `--output`; world effects stop at the range and the report says so (ADR-0121); `--to` past the end is legal (ADR-0077 §4) | No in/out points stored *in the project*; the range is given on each call. A partial render can never land on the declared `output` |
| Send to Media Encoder / background encoding | *Send to Media Encoder* queues the job in AME so editing continues | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-options-in-premiere.html | partial | A render can run in the background and be polled through its progress file (ADR-0192); MCP calls run off the runtime thread (ADR-0108) | No separate encoder application or persisted job list |
| Encode queue | AME queue of many jobs, run in order | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-options-in-premiere.html | partial | MCP `render`/`preview` share one encode slot; a second call waits and its heartbeat says what it is queued behind (ADR-0108 §3) | A queue that outlives the MCP process, survives restarts, or can be reordered or inspected as a list |
| Parallel encoding † | AME *Enable Parallel Encoding* runs several queue items at once | https://helpx.adobe.com/media-encoder/desktop/encoding-and-exporting/export-settings-reference.html | refused | ADR-0108 §3: one encode slot; raising the bound above one requires a per-output-path lock it chose not to build | n/a (refused within one server) |
| Multiple outputs from one source † | One source item with several output presets/destinations in the AME queue | https://helpx.adobe.com/media-encoder/desktop/encoding-and-exporting/export-settings-reference.html | partial | Several `render` calls with different `--output` paths, queued one after another (ADR-0108) | Since every render is the same shape (ADR-0143), "multiple outputs" can differ only in range and path, never in format or size |
| Watch folders † | AME watch folders encode any file dropped into them with an assigned preset | https://helpx.adobe.com/media-encoder/desktop/encoding-and-exporting/export-settings-reference.html | missing | none | A folder-triggered encode |
| Publish to social destinations (YouTube, Vimeo, TikTok, Facebook, X, Behance) | Export mode destinations with per-platform presets; sign in and upload with title, description, tags, privacy | https://helpx.adobe.com/premiere-pro/using/exporting-web-mobile-devices.html | missing | none; ADR-0003 makes a publishing channel a fixture, not a scope, and refuses nothing here | Platform presets, authentication and upload |
| Social-platform presets (aspect and spec per destination) | Optimised settings chosen per destination | https://helpx.adobe.com/my_en/premiere/desktop/render-and-export/export-files/export-videos-for-social-media-channels.html | missing | `create_project` scaffolds a frame and fps from what it is told (ADR-0080); no per-platform table | Named platform specs |
| Frame.io upload / share for review | Upload to Frame.io from Premiere; *Export markers as comments* | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/share-for-review-using-frame-io/upload-media-to-frameio.html | missing | none | Any review-service upload; markers (Montagent has no marker concept to export) |
| Publish settings on AME (FTP, Adobe Stock, Creative Cloud) † | AME *Publish* tab: upload the encoded file to a destination | https://helpx.adobe.com/media-encoder/desktop/encoding-and-exporting/export-settings-reference.html | missing | none | Post-encode upload |

### E. What the file carries

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| HDR export (Rec. 2100 PQ / HLG) | Export colour space Rec. 2100 HLG/PQ on ProRes, XAVC-I, HEVC (*HEVC – Match Source – PQ*); SDR by default | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/create-and-export-hdr-videos-for-youtube-on-mac.html | missing | `encode.rs`: 8-bit `yuv420p`, no colour-primaries or transfer tags | 10-bit output, PQ/HLG transfer, HDR mastering metadata |
| Output colour space (Rec. 709 / Rec. 2020 / Rec. 2100) | Sequence/export *Output Color Space* menu | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/configure-sequence-output-color-space.html | missing | `encode.rs` writes no colour-space tags; no ADR on colour management | Declared output colour space and its container tags |
| HDR-to-SDR conversion on export | Effects tab HDR-to-SDR conversion | https://helpx.adobe.com/media-encoder/desktop/encoding-and-exporting/export-settings-reference.html | missing | No HDR pipeline at all | Tone mapping |
| Loudness normalisation on export † | AME Effects tab *Loudness Normalization*: standard (ITU BS.1770 / ATSC A/85 / EBU R128), target loudness, tolerance, max true peak | https://helpx.adobe.com/media-encoder/desktop/encoding-and-exporting/export-settings-reference.html | have | `master.target_lufs` (BS.1770 gated, −40 to −5) reached by one measured gain, and `master.ceiling_dbtp` with a 4× oversampled limiter promising the decoded AAC within +1 dB (ADR-0172, ADR-0174); `verify` checks both (ADR-0173) | No named-standard picker (a value is typed); it is a project setting, not an export-time option |
| Captions burned into the picture | Captions tab: *Burn Captions Into Video* | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/overview-of-export-settings.html | have | Captions are `text` elements painted into the frame (`CONTEXT.md` *Caption*; ADR-0136); caption checks in `validate` | Nothing: burn-in is the only kind |
| Captions word-by-word from a transcript | (Premiere generates caption tracks from Speech to Text; export carries them) | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/overview-of-export-settings.html | have (skill) | `montagent-footage` skill, `scripts/captions.py`: word timings → highlighted caption pages as `text` elements | n/a (speech-to-text itself belongs to another area) |
| Captions as a sidecar file (SRT, SCC, MCC, Timed Text XML, EBU STL) | Captions tab: *Create Sidecar File* in .srt, .scc, .mcc, W3C/SMPTE/EBU Timed Text .xml, EBU N19 .stl | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/supported-export-file-formats.html | missing | none: a caption is a painted `text` element, and the schema has no caption-track concept | A sidecar writer (the text and times are in the file, so it is derivable) |
| Captions embedded in the stream (CEA-608/708, open/closed) † | Captions tab: *Embed in Output File* (H.264/MXF/QuickTime) | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/overview-of-export-settings.html | missing | none | Closed-caption data in the deliverable |
| Metadata export (XMP embed or sidecar, markers, rights/description fields) † | *Metadata* export settings: embed in output file or create a sidecar XMP; include markers; edit fields | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/overview-of-export-settings.html | partial | `encode.rs` writes one `comment=<stamp>` tag, the attestation `verify` and ADR-0104 read | Title/author/copyright/description fields, XMP, sidecar, markers or chapters |
| Chapter markers in the file † | Sequence markers exported as chapter markers in metadata | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/overview-of-export-settings.html | missing | No marker concept in the schema | Markers, and writing them as MP4 chapters |
| Loop flag for looping formats | GIF loop; (container loop metadata) | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/supported-export-file-formats.html | missing | ADR-0062: `loop` is a `validate` claim only, and "Montagent writes no container-level loop metadata" | Any container loop flag (moot until GIF exists) |

### F. Effects applied on export

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Lumetri Look / LUT on export | Effects tab *Lumetri Look/LUT*: built-in look or *Select* a custom LUT; Intensity | https://helpx.adobe.com/media-encoder/desktop/encoding-and-exporting/export-settings-reference.html | refused | ADR-0040 ("LUTs: semantics live entirely in an external `.cube` file, which is a plugin in all but name"); ADR-0049's stopping rule excludes curves and LUTs | n/a (refused). The four scalar colour effects (ADR-0049) are the closest literal mechanism |
| Save / export a LUT from a grade | Lumetri *Export .cube* | https://helpx.adobe.com/in/premiere/desktop/correct-color/add-color-effects/save-or-export-look-up-tables.html | refused | ADR-0040, ADR-0049 (same refusal; no LUT concept to write) | n/a (refused) |
| Image overlay | Effects tab *Image Overlay*: image, position, offset, size, opacity | https://helpx.adobe.com/media-encoder/desktop/encoding-and-exporting/export-settings-reference.html | have | An `image` element on the top-`layer` track for the whole duration, with `x`/`y`/`scale`/`opacity` | It lives in the composition, not in an export setting, so it is in every render, `frame` and `preview` |
| Text / name overlay | Effects tab *Text Overlay* / *Name Overlay* (burns a name or text string; position, size, opacity) | https://helpx.adobe.com/media-encoder/desktop/encoding-and-exporting/export-settings-reference.html | partial | A `text` element with a literal string, `caption: false` (ADR-0136) | No derived text: the sequence/file name is not filled in automatically; the agent writes the literal |
| Timecode overlay | Effects tab *Timecode Overlay*: position, offset, size, opacity; Time Source (Media File / Generate Timecode), offset in frames | https://helpx.adobe.com/media-encoder/desktop/encoding-and-exporting/export-settings-reference.html | missing | None. A `text` element is static per element; `montagent-craft` says timecodes go in only on request | A per-frame derived timecode text (or a skill that writes one element per frame) |
| Video Limiter on export | Effects tab *Video Limiter*: clip level (IRE), compression before clipping; also an output effect | https://helpx.adobe.com/ae_en/premiere/desktop/correct-color/add-color-effects/available-color-correction-effects.html | missing | No broadcast-safe stage | Luma/chroma legalisation |
| Timecode effect (timeline) | Removed in Premiere 26.0 (no longer renders) | https://helpx.adobe.com/premiere-pro/using/metadata-effect.html | missing | none | Same as the timecode overlay |

### G. Interchange

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Final Cut Pro XML (xmeml) export | File > Export > Final Cut Pro XML, with an *FCP Translation Results* log | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-a-project-as-a-final-cut-pro-xml-file.html | missing | none; the project JSON is the only timeline file | An XML writer (timeline mapping, translation report) |
| EDL (CMX3600) export | File > Export > EDL; best with one video track and two stereo audio tracks; 32-character name option | https://helpx.adobe.com/nz/premiere-pro/using/exporting-projects-applications.html | missing | none | EDL writer |
| AAF export | File > Export > AAF (embedded or separate audio; 2 GB limit on embedded, Windows) | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-aaf-files.html | missing | none | AAF writer |
| OMF export (Pro Tools) | File > Export > OMF; ≤ 2 GB; separate audio for large projects | https://helpx.adobe.com/premiere-pro/using/exporting-omf-files-pro-tools.html | missing | none | OMF writer |
| OpenTimelineIO (OTIO) export | Import and export OTIO projects (release 26.2.2) | https://helpx.adobe.com/premiere/desktop/whats-new/release-notes.html | missing | none | OTIO writer |
| ALE export † | Avid Log Exchange named among exported interchange files in the 26.5 known issues | https://helpx.adobe.com/premiere/desktop/whats-new/release-notes.html | missing | none | ALE writer |
| Export Frame to project (still as new media) † | *Import into project* option on Export Frame | https://helpx.adobe.com/nz/premiere-pro/using/formats-exported-directly-premiere-pro.html | missing | `frame --out` writes a file; nothing adds it to the document | A one-step "still becomes an `image` element" |
| Frame.io comments ↔ markers | *Export markers as comments* on upload | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/share-for-review-using-frame-io/upload-media-to-frameio.html | missing | No marker concept | Markers and a review-service bridge |
| Project as data an agent can read | (Premiere: proprietary `.prproj`; interchange via XML/AAF/OTIO) | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-a-project-as-a-final-cut-pro-xml-file.html | have | The project *is* a documented JSON file (`schema/montagent.schema.json`, published as an MCP resource), editable by exact-string replace (ADR-0145) | No conversion into any other editor's format (see the rows above) |

## Notable gaps

1. **One deliverable shape.** H.264 8-bit 4:2:0 + AAC stereo in MP4 is the only output. Every
   mezzanine and broadcast codec (ProRes, DNxHR, CineForm, XAVC, MXF, DCP), HEVC and AV1, MOV, and
   animated GIF are missing. None is refused; ADR-0143 refuses only *knobs on the H.264 encoder*.
2. **No alpha deliverable.** The encoder takes `rgb24` and writes `yuv420p`. The prerender skill
   proves PNG-in-MOV with alpha round-trips exactly, but only for footage going *in*.
3. **No audio-only export and no channel layouts beyond stereo.** A mix cannot leave as WAV/MP3, and
   mono/5.1/multi-track output waits on a channel-operations capability (ADR-0172).
4. **No HDR or colour-space tagging.** No PQ/HLG, no 10-bit, no Rec. 2020/2100, no colour tags at all.
5. **No interchange.** FCP XML, EDL, AAF, OMF, OTIO and ALE are all missing. The project JSON is
   open, but nothing translates it.
6. **No sidecar or embedded captions.** Captions exist only as painted `text` elements, even though
   their words and times are in the file (an SRT/WebVTT writer would be derivable).
7. **No image-sequence export and only JPEG/PNG stills.** `frame` covers one instant at a time.
8. **No publishing or review destinations.** No social upload, no Frame.io, no watch folders.
9. **Refused on purpose:** hardware encoding and every encoder knob (ADR-0143 §2, §5), parallel
   encodes (ADR-0108 §3), LUTs on export (ADR-0040, ADR-0049).
10. **Where Montagent matches or exceeds:** loudness normalisation with a true-peak promise
    (ADR-0172/0174), range export (ADR-0121), progress/ETA (ADR-0192), cancel-publishes-nothing
    (ADR-0109), overwrite attestation (ADR-0104), and a post-encode `verify` with no Premiere
    equivalent (ADR-0117).
