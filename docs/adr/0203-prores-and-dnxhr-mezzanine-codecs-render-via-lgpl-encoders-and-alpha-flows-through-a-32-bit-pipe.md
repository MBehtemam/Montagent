---
status: accepted
amends: 0009 (spawned ffmpeg), 0021 (odd dimension padding), 0115 (ffmpeg qualification), 0143 (encoder parameters)
---

# ProRes and DNxHR mezzanine codecs render via LGPL encoders and alpha flows through a 32-bit pipe

[#983](https://github.com/MBehtemam/Montagent/issues/983), [#1002](https://github.com/MBehtemam/Montagent/issues/1002), [#1004](https://github.com/MBehtemam/Montagent/issues/1004).

## Context

Montagent currently strips the alpha channel byte during Skia canvas readback (`Canvas::rgb`) and pipes 24-bit `rgb24` rawvideo frames into `libx264` in an MP4 container ([ADR-0077](0077-the-nine-render-readings-are-ratified.md), [ADR-0143](0143-render-keeps-libx264-medium-crf-20-pins-five-encoder-threads-and-says-so.md)). Any transparent region is composited over black (`#000000FF`).

In motion graphics, VFX, and broadcast editing, deliverables must frequently carry full transparency (alpha channel) in Apple ProRes 4444 or mastering quality in Avid DNxHR. Furthermore, Montagent's spawned FFmpeg boundary ([ADR-0009](0009-ffmpeg-is-spawned-never-linked.md)) must preserve cross-platform byte determinism without proprietary licensing entanglements or hardware driver variance.

## Decision

1. **Native LGPL Encoder Backends**:  
   Montagent renders mezzanine deliverables using FFmpeg's native, built-in LGPL encoders:
   - **Apple ProRes**: `prores_ks` (Kostya multi-slice encoder).
     - Valid profiles: `proxy`, `lt`, `standard` (default), `hq`, `4444`, `4444xq`.
     - Pixel format: 10-bit `yuv422p10le` for 422 profiles; 10-bit `yuva444p10le` for 4444 profiles with alpha.
     - Compatibility: QuickTime container tagged with `-vendor apl0` for native macOS and Final Cut Pro / Premiere acceleration.
   - **Avid DNxHR**: `dnxhd` encoder.
     - Valid profiles: `dnxhr_lb`, `dnxhr_sq`, `dnxhr_hq` (default), `dnxhr_hqx` (10-bit), `dnxhr_444` (10-bit).
     - Container: QuickTime MOV or MXF OP1a (`.mxf`).

2. **32-bit Alpha Channel Pipeline**:  
   When `output.video.alpha: true` is configured:
   - **Canvas Initialization**: The Skia canvas clears to transparent `Rgba([0, 0, 0, 0])`. If the project declares no `background`, no opaque layer is drawn.
   - **32-bit Pipe Transport**: Canvas readback switches from `Canvas::rgb` to `Canvas::rgba`, transmitting uncompressed 32-bit premultiplied RGBA over `pipe:0`.
   - **FFmpeg Input**: Spawns with `-f rawvideo -pix_fmt rgba -s {width}x{height} -r {fps} -i pipe:0`.
   - **Chroma & Alpha Conversion**: FFmpeg maps `rgba` to 10-bit `yuva444p10le` using `swscale`.
   - **Odd Dimension Padding ([ADR-0021](0021-odd-frame-dimension-is-padded-to-even.md))**: If padding is required, the pad color is set to transparent: `-vf pad={w}:{h}:0:0:color=0x00000000`.

3. **Software Determinism & Zero Hardware Inconsistency**:  
   Hardware-accelerated encoders (e.g. Apple VideoToolbox `prores_videotoolbox` or NVENC) are **refused**. Software encoding via `prores_ks` and `dnxhd` guarantees identical, bit-level determinism across Linux, macOS (Apple Silicon and Intel), and Windows.

4. **Static Validation Invariants (`montagent validate`)**:  
   - **`E-CODEC-ALPHA-UNSUPPORTED`**: Emitted if `alpha: true` is requested on an encoder that does not support an interleaved alpha channel (e.g. `h264`, `dnxhr`, or ProRes profiles other than `4444` and `4444xq`).
   - **`E-CODEC-PROFILE-INVALID`**: Emitted if the requested `profile` is not a valid profile for the specified `codec`.
   - **`E-CONTAINER-UNSUPPORTED`**: Emitted if the combination of container and codec is invalid (e.g. ProRes inside an MP4 container).

## Consequences

- Motion graphics overlays, titles, and lower thirds can be exported with transparent backgrounds for compositing in NLEs.
- Broadcasters and post-production houses can receive master ProRes and DNxHR deliverables directly from Montagent.
- Licensing remains clean under LGPL v2.1+, requiring no commercial Apple or Avid royalties.
