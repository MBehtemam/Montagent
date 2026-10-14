---
status: accepted
amends: 0077 (reading 7's H.264 encode pipeline converts RGB to YUV with the BT.709 matrix and studio-swing range and tags output streams as `bt709`/`tv` instead of `swscale`'s untagged Rec.601 default), 0143 (records the one-time deliverable byte change from switching `encode.rs`'s RGB-to-YUV conversion from `bt601` to `bt709` and writing VUI/`colr` tags), 0203 (extends explicit BT.709 matrix conversion and `bt709`/`tv` stream and container tagging across `prores_ks` MOV and `dnxhd` MOV/MXF mezzanine deliverables)
---

# sRGB is the working colour space, and video boundaries convert and tag BT.709

[#1057](https://github.com/MBehtemam/Montagent/issues/1057), on the map
[#896](https://github.com/MBehtemam/Montagent/issues/896), from the colour-management census in
[#904](https://github.com/MBehtemam/Montagent/issues/904) and
[#907](https://github.com/MBehtemam/Montagent/issues/907).

## The gap

Montagent's compositor (`crates/montagent-render/src/canvas.rs`) paints into an 8-bit RGBA raster
surface (`ColorType::RGBA8888`, `AlphaType::Premul`, `color_space: None`), and every hex literal
(`#RRGGBB` / `#RRGGBBAA`), colour filter ([ADR-0049](0049-v1-colour-filter-vocabulary-four-scalar-members.md)),
gradient ([ADR-0149](0149-a-gradient-is-a-paint-linear-or-radial-measured-against-the-declared-box.md)),
blend mode ([ADR-0147](0147-a-blend-mode-is-a-flat-static-field-of-five-values-and-the-finished-element-blends-last.md)),
ASC-CDL / 3D LUT shader ([ADR-0206](0206-asc-cdl-and-3d-lut-tetrahedral-shaders-grade-colour-under-closed-arithmetic.md)),
and scope ([ADR-0207](0207-parametric-eq-dynamics-processors-and-perceptual-scopes-provide-mixing-and-inspection.md))
operates on 8-bit sRGB code values. Yet neither the working colour space nor the Y'CbCr
conversions at the video decode and encode boundaries were specified:

1. **Untagged Rec.601 encode drift (`crates/montagent-render/src/encode.rs`):** `Encoder::spawn_inner`
   pipes `-f rawvideo -pix_fmt rgb24` (or `rgba` when `alpha` is true under
   [ADR-0203](0203-prores-and-dnxhr-mezzanine-codecs-render-via-lgpl-encoders-and-alpha-flows-through-a-32-bit-pipe.md))
   into `ffmpeg` with no colour-matrix filter and no colour flags. Because `rawvideo` carries no
   colour metadata, `swscale`'s auto-inserted RGB → Y'CbCr conversion defaults to the **ITU-R
   BT.601 (`bt601`)** matrix, and the written MP4, MOV, or MXF stream carries `unknown` for
   `color_space`, `color_primaries`, `color_transfer`, and `color_range`. When an HD player or NLE
   (QuickTime, browsers, Premiere Pro, After Effects) plays that untagged HD deliverable, it
   assumes **Rec.709** on decode while `swscale` encoded the stream with **Rec.601**—shifting an
   authored `#DC3C28` (`RGB(220, 60, 40)`) swatch to `RGB(235, 74, 36)` (`+15` red, `+14` green,
   `-4` blue).
2. **Filtergraph `unspecified` overwrite on `pipe:0`:** In `ffmpeg` 9.0.2, passing
   `-colorspace bt709 -color_primaries bt709 -color_trc bt709 -color_range tv` on the output side
   alone (without setting frame parameters in `-vf`) still leaves `color_primaries` and
   `color_transfer` as `unknown` in the output stream, because the filtergraph negotiates
   `unspecified` primaries and transfer characteristics from the `rawvideo` `pipe:0` input and
   overwrites the encoder context.
3. **Untagged video decode fallback (`crates/montagent-render/src/decode.rs`):** `decode::frame_at`
   and `decode::frames_at` end their `-vf` chain with bare `scale={width}:{height}` and
   `-pix_fmt rgba`. When a video source is tagged (`bt709`, `smpte170m`, `bt470bg`, `tv` or `pc`),
   `scale` honours the stream's matrix and range. When a video source is **untagged**
   (`color_space` is `unknown` / `unspecified`, even at 1920×1080), `swscale` falls back to
   `SWS_CS_DEFAULT` (`bt601`), shifting an untagged Rec.709 source from `(220, 60, 40)` to
   `(206, 45, 43)`. Conversely, passing `in_color_matrix=bt709` unconditionally on every source
   overrides genuinely tagged Rec.601 (`smpte170m` / `bt470bg`) footage and shifts `(220, 60, 40)`
   to `(235, 75, 36)`.
4. **ICC-tagged still images (`crates/montagent-render/src/canvas.rs`):** `Raster::decode` calls
   `Image::from_encoded`, which parses embedded ICC profiles, but `Canvas::new` creates its Skia
   surface with `color_space: None` (legacy untagged mode), causing Skia to ignore embedded ICC
   profiles (such as Display P3 screenshots) when painting onto the canvas.

All four measurements are reproduced by
[`docs/adr/srgb_rec709_colour_check.py`](srgb_rec709_colour_check.py).

## Decision

### 1. Working colour space and transfer-curve pass-through

1. **8-bit display-referred sRGB:** Montagent's working colour space is **8-bit sRGB** (`[u8; 4]`,
   `ColorType::RGBA8888`): premultiplied (`AlphaType::Premul`) on the Skia `Canvas` surface,
   unpremultiplied (`AlphaType::Unpremul`) at the `decode` and `frame` PNG boundaries, and 24-bit
   RGB composited over opaque black (`rgb24`) or 32-bit unpremultiplied/premultiplied RGBA (`rgba`,
   per [ADR-0203](0203-prores-and-dnxhr-mezzanine-codecs-render-via-lgpl-encoders-and-alpha-flows-through-a-32-bit-pipe.md))
   at the encoder pipe.
2. **Direct transfer-curve pass-through to Rec.709:** sRGB and ITU-R BT.709 share identical
   primaries and the D65 white point. Following standard 8-bit display-referred NLE practice
   (Premiere Pro, DaVinci Resolve, Final Cut Pro, and After Effects with `Working Space: None`),
   8-bit sRGB code values map directly to Rec.709 Y'CbCr without a $\gamma \approx 2.2 \leftrightarrow 2.4$
   transfer-curve re-quantization LUT on decode or encode. Video decode and encode convert
   strictly through the **ITU-R BT.709 luma/chroma matrix** ($K_r = 0.2126, K_b = 0.0722$) and
   studio-swing (`tv`, 16–235 luma / 16–240 chroma in 8-bit; 64–940 / 64–960 in 10-bit) range
   scaling, preserving `#RRGGBB` hex swatches across render-and-decode round trips without 8-bit
   LUT banding.
3. **Scope boundary:** 16-bit and 32-bit float buffers, linear-light compositing ($\gamma = 1.0$),
   OpenColorIO (OCIO) configurations, and HDR transfer functions (`PQ`, `HLG`) are out of scope
   here and belong to sibling map
   [#1080](https://github.com/MBehtemam/Montagent/issues/1080).

### 2. Decode normalisation for `video` and `image` sources

1. **Video sources (`crates/montagent-core/src/media/probe.rs` and `crates/montagent-render/src/decode.rs`):**
   - `probe::probe` reads `color_space` from `ffprobe`'s video stream JSON and records whether the
     source carries an explicit colour-matrix tag.
   - When the source stream carries a known `color_space` tag (such as `bt709`, `smpte170m`,
     `bt470bg`, `bt2020nc`, or `gbr`), `decode::frame_at` and `decode::frames_at` end `-vf` with
     `scale={width}:{height}:out_range=pc`, leaving `in_color_matrix=auto` and `in_range=auto` so
     `swscale` honours the source stream's own tagged matrix and range (`tv` or `pc`, including
     `yuvj*` pixel formats).
   - When the source stream's `color_space` is missing, `unknown`, or `unspecified` (or when
     `Decoder` carries no tagged matrix), `decode::frame_at` and `decode::frames_at` pass
     `scale={width}:{height}:in_color_matrix=bt709:out_range=pc`, defaulting untagged Y'CbCr
     streams to **ITU-R BT.709** (and limited `tv` range unless tagged `pc`) at every resolution.
2. **Still-image sources (`crates/montagent-render/src/canvas.rs`):**
   - In `Raster::decode`, when `Image::from_encoded` produces an image with an embedded ICC colour
     space that is not already sRGB (`image.color_space()` is `Some(cs)` and `!cs.is_srgb()`), its
     pixels are converted once at decode time into `skia_safe::ColorSpace::new_srgb()`
     (`ColorType::RGBA8888`, `AlphaType::Premul`) and wrapped in a `Raster` with
     `color_space: None`.
   - Untagged still images and still images already tagged sRGB keep their decoded 8-bit pixel
     bytes untouched. `Canvas::new` remains an untagged (`color_space: None`) raster surface so
     painting untagged assets incurs zero colour-management overhead and existing golden frames of
     untagged assets remain byte-identical.

### 3. Encode matrix conversion and stream/container colour tagging

Across every Y'CbCr deliverable encoder (`libx264` in `mp4`/`mov`, `prores_ks` in `mov`, and
`dnxhd` in `mov`/`mxf`), `Encoder::spawn_inner` in `crates/montagent-render/src/encode.rs`
applies:

1. **Filtergraph (`-vf`):**
   ```text
   scale=in_range=pc:out_color_matrix=bt709:out_range=tv,setparams=colorspace=bt709:color_primaries=bt709:color_trc=bt709:range=tv
   ```
   When odd-dimension padding is active ([ADR-0077](0077-the-nine-render-readings-are-ratified.md)
   §6), `pad={width}:{height}:0:0:color={pad_color}` precedes `scale` in the same `-vf` chain so
   the background pad colour is converted through the identical BT.709 matrix.
2. **Output stream and container flags:**
   ```text
   -colorspace bt709 -color_primaries bt709 -color_trc bt709 -color_range tv
   ```
   Together with `setparams`, this writes `color_space = bt709`, `color_primaries = bt709`,
   `color_transfer = bt709` (`nclx` `1-1-1`), and `color_range = tv` into the H.264 SPS VUI, the
   MP4/MOV `colr` atom, and the MXF descriptor.

### 4. Acceptance checks

1. **Stream and container tag assertion:** `ffprobe -select_streams v:0 -show_entries stream=color_space,color_primaries,color_transfer,color_range`
   on rendered deliverables across `h264` (`mp4`, `mov`), `prores` (`mov`), and `dnxhr` (`mov`,
   `mxf`) must report:
   - `color_space`: `"bt709"`
   - `color_primaries`: `"bt709"`
   - `color_transfer`: `"bt709"`
   - `color_range`: `"tv"`
2. **Pixel round-trip assertion:** Rendering a project with a solid `#DC3C28` (`RGB(220, 60, 40)`)
   swatch and decoding the deliverable back through `decode::frame_at` must recover every RGB
   channel within **`±2/255`** (the 8-bit 4:2:0 / 10-bit 4:2:2 integer quantization ceiling,
   rejecting the `15/255` drift of a `bt601` ↔ `bt709` mismatch).

## Measurements

From [`docs/adr/srgb_rec709_colour_check.py`](srgb_rec709_colour_check.py) on `ffmpeg` 9.0.2 with
swatch `#DC3C28` (`RGB(220, 60, 40)`):

| Path / case | `ffprobe` tags (`space / primaries / trc / range`) | Decoded RGB | Max channel $\|\Delta\|$ |
|---|---|---|---|
| Old untagged `libx264` encode → Rec.709 player decode | `unknown / unknown / unknown / unknown` | `(235, 74, 36)` | `15 / 255` |
| Output flags only (no `setparams` in `-vf`) | `bt709 / unknown / unknown / tv` | `(220, 60, 40)` | `0 / 255` (missing tags) |
| New encode chain (`libx264` MP4) | `bt709 / bt709 / bt709 / tv` | `(220, 60, 40)` | `0 / 255` |
| New encode chain (`prores_ks` MOV) | `bt709 / bt709 / bt709 / tv` | `(218, 59, 38)` | `2 / 255` |
| New encode chain (`dnxhd` MOV) | `bt709 / bt709 / bt709 / tv` | `(220, 60, 40)` | `0 / 255` |
| New encode chain (`dnxhd` MXF) | `bt709 / bt709 / bt709 / tv` | `(220, 60, 40)` | `0 / 255` |
| Bare `scale=W:H` decode on untagged Rec.709 clip | `unknown / unknown / unknown / unknown` | `(206, 45, 43)` | `15 / 255` |
| Unconditional `in_color_matrix=bt709` decode on tagged `smpte170m` clip | `smpte170m / smpte170m / smpte170m / tv` | `(235, 75, 36)` | `15 / 255` |
| Tag-aware decode on tagged `bt709` (`tv`), tagged `bt709` (`pc`), tagged `smpte170m` (`tv`), and untagged Rec.709 (`tv`) | honoured when tagged; `bt709` fallback when untagged | `(220, 61, 40)` / `(219, 61, 39)` / `(221, 60, 39)` / `(220, 61, 40)` | `1 / 255` |

## Considered and refused

- **Apply a transfer-characteristic LUT between sRGB piece-wise $\gamma \approx 2.2$ and BT.709 /
  BT.1886 $\gamma \approx 2.4$ on 8-bit decode and encode.** Refused: in an 8-bit integer pipeline,
  re-mapping the transfer curve quantizes dark code values twice, introduces visible 8-bit banding,
  and causes an authored `#RRGGBB` hex solid in Montagent to decode back with shifted RGB numbers
  and disagree with standard NLEs (Premiere Pro, DaVinci Resolve, Final Cut Pro, and After Effects
  with `Working Space: None`), which pass 8-bit display-referred code values directly into the
  BT.709 matrix.
- **Tag `-color_trc iec61966-2-1` (`nclx` `1-13-1`) instead of `-color_trc bt709` (`1-1-1`) on
  encoded video deliverables.** Refused: QuickTime Player / macOS AVFoundation and broadcast NLEs
  expect `1-1-1` (`bt709` primaries, `bt709` transfer, `bt709` matrix) on standard Rec.709 H.264,
  ProRes, and DNxHR video; `1-13-1` triggers divergent ColorSync display gamma adjustments across
  players.
- **Force `in_color_matrix=bt709` unconditionally on every video decode regardless of stream tags.**
  Refused: FFmpeg's `scale` filter already honours embedded matrix tags when `in_color_matrix=auto`
  is left intact, whereas forcing `in_color_matrix=bt709` overrides genuinely tagged Rec.601
  (`smpte170m` / `bt470bg`) sources and introduces a `15/255` colour shift. Falling back to
  `in_color_matrix=bt709` only when `color_space` is `unknown` / `unspecified` fixes untagged
  sources without breaking tagged ones.
- **Resolution-dependent decode fallback (`bt601` below 720p, `bt709` at 720p and above) for
  untagged video.** Refused: Montagent's own proxy tiers ([ADR-0065](0065-preview-proxy-target-720p-540p-floor-disclosed-not-certified.md)),
  skill media fixtures (`320×180`), and test fixtures (`64×64`) frequently sit below 720p while
  originating from sRGB/Rec.709 renders; a resolution threshold would make an untagged clip change
  colour when downscaled below 720p.

## Consequences

- [ADR-0077](0077-the-nine-render-readings-are-ratified.md),
  [ADR-0143](0143-render-keeps-libx264-medium-crf-20-pins-five-encoder-threads-and-says-so.md),
  and
  [ADR-0203](0203-prores-and-dnxhr-mezzanine-codecs-render-via-lgpl-encoders-and-alpha-flows-through-a-32-bit-pipe.md)
  gain amendment banners pointing here.
- `CONTEXT.md`'s **Colour** entry records 8-bit display-referred sRGB as the working colour space
  and cites this ADR for the BT.709 decode/encode boundary rules and ICC still conversion.
- Every deliverable `render` and `preview` write changes bytes once when the implementation ticket
  switches `encode.rs` from `swscale`'s untagged `bt601` default to explicit `bt709` matrix
  conversion and `1-1-1` stream/container tags; the implementation ticket re-takes
  `fixtures/benchmark/spy-trailer/frames.framemd5` (and any committed rendered MP4 fixtures) under
  the new encoder arguments.
