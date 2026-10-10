# Premiere gap: import, supported formats and media handling

Research ticket [#891](https://github.com/MBehtemam/Montagent/issues/891) on the map
[#882](https://github.com/MBehtemam/Montagent/issues/882). Area slug: **import**.

- **Date:** 2026-10-10
- **Montagent side:** tag `v0.2.0` (`51aa4b5`, = `origin/main` at writing). Read: `CONTEXT.md`,
  `schema/montagent.schema.json`, ADR-0002, 0004, 0005, 0006, 0009, 0011, 0015, 0017, 0020, 0023,
  0027, 0045, 0046, 0050, 0053, 0056, 0065, 0069, 0070, 0077, 0080, 0089, 0091, 0092, 0093, 0096,
  0115, 0131, 0140, 0145, 0157, 0159, 0171, 0184, 0186; the MCP surface (`crates/montagent/src/mcp.rs`: `validate`, `create_project`,
  `query`, `frame`, `render`, `verify`, `preview`, `measure`, `shift`, `compare`; stdio transport
  only) and the CLI-only verbs (`probe`, `timeline`, `fmt`, `cache clear`, `fonts`); the decode
  code (`crates/montagent-render/src/decode.rs`, `canvas.rs` `Raster::decode`,
  `crates/montagent-core/src/media/*`); the Skia pin in the root `Cargo.toml`
  (`binary-cache`, `embed-icudtl`, `pdf`, `jpeg`) and `THIRD-PARTY.md`; the skills
  `montagent-footage` and `montagent-prerender`.
- **Premiere side:** Adobe's own helpx.adobe.com pages only. **Caveat, stated plainly:** this
  session's network policy refused every connection to `helpx.adobe.com` (proxy `403` on
  CONNECT), so no Adobe page could be fetched in full. The Premiere mechanisms below come from
  Adobe search-result extracts of helpx pages (format list, VFR note, raw-format KB articles,
  interpret-frame-rate page) and from the Premiere user guide as known before this session.
  URLs are marked:
  - no mark: the URL was returned by a helpx.adobe.com-restricted search in this session, or is
    already cited in this repository;
  - **†**: the page path is recalled from the Premiere user guide and was **not** fetched or
    returned this session. Check each † URL against the census ticket
    [#883](https://github.com/MBehtemam/Montagent/issues/883) before relying on it.
- **Status vocabulary** (map #882): `have`, `have (skill)`, `partial`, `missing`, `refused`.

## Counts

| Status | Rows |
| --- | --- |
| have | 38 |
| have (skill) | 3 |
| partial | 21 |
| missing | 48 |
| refused | 9 |
| **total** | **119** |

## Three facts that decide most rows

1. **Video and audio decode through the user's own `ffmpeg`** (ADR-0009, floor ffmpeg 7.1 per
   ADR-0115, a missing one is `E-TOOL-MISSING` per ADR-0091), and every source is probed by
   `ffprobe` (ADR-0011). So a codec "is supported" exactly when the user's `ffmpeg` decodes it.
   The repository tests H.264 MP4, ProRes (incl. 4444) and VP9-alpha WebM
   (`docs/research/alpha-decode/`, ADR-0096); other ffmpeg codecs marked `have` below ship the
   mechanism but no repo test exercises them. Every decoded frame becomes **8-bit RGBA**
   (`decode.rs`: `-pix_fmt rgba`), with no colour management, tone mapping or deinterlacing.
2. **Stills decode through Skia** (`Raster::decode` → `Image::from_encoded`), pinned to the
   codecs in the prebuilt key: PNG, JPEG and GIF (wuffs) are named in `THIRD-PARTY.md`. WebP,
   HEIF, TIFF, TGA, DPX, PSD and EXR are not in the pin. A still Skia cannot decode is only
   caught at paint time as `E-NOT-PAINTED-UNDECODABLE` (ADR-0093), not by `validate`.
3. **There is no media catalogue.** `source` is written inline on each element and "asset" is
   not in the vocabulary; ADR-0002 refuses an asset table ("`source` never gets a table") and
   ADR-0053 refuses even an `assetRoot`. Every Premiere feature whose object is a Project-panel
   item (bins, subclips, freeform boards) therefore has nothing to attach to.

Two accepted ADRs in this area are **not in the v0.2.0 schema**: ADR-0023's `par` key and
ADR-0140's `swaps` (neither key appears in `schema/montagent.schema.json` or the element model).
ADR-0184 (`channels`, `pan`) is still `proposed`. ADR-0159's remote instance is accepted but
v0.2.0 binds the MCP stdio transport only (`mcp.rs`). Rows that lean on these say so.

## Inventory

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
| --- | --- | --- | --- | --- | --- |
| **Formats ›** H.264 / AVC (MP4, MOV, M4V) | Native import; hardware-accelerated decode on supported GPUs | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html | have | ffmpeg decode (ADR-0009); reference MP4 fixture and long-GOP H.264 measured in ADR-0096 | — |
| **Formats ›** HEVC / H.265 | Native import, hardware decode where available | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html | have | ffmpeg decode (ADR-0009) | No repo test; 10-bit HEVC is truncated to 8-bit (see high-bit-depth row) |
| **Formats ›** Apple ProRes 422 family | Native import (built-in decoder, both OSes) | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html | have | ffmpeg decode; ProRes measured in ADR-0096 | — |
| **Formats ›** ProRes 4444 / 4444 XQ with alpha | Native import; alpha read as straight or premultiplied per Interpret Footage | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html | have | ADR-0089 (`alpha` from pixel format, `yuva444p12le` measured); `probe` reports `alpha.carries` | — |
| **Formats ›** Avid DNxHD / DNxHR (MXF, MOV) | Native import | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html | have | ffmpeg `dnxhd` decoder via ADR-0009 path | No repo test |
| **Formats ›** GoPro CineForm | Native import (also a proxy/intermediate codec) | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html | have | ffmpeg `cfhd` decoder via ADR-0009 path | No repo test |
| **Formats ›** AV1 | Native import (decode) | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html † | partial | ffmpeg decode path (ADR-0009) | ADR-0115's floor (ffmpeg 7.1 + libx264) does not require a software AV1 decoder (libdav1d/libaom), so it depends on the user's build; no repo test |
| **Formats ›** MPEG-1 / MPEG-2 (MPEG, M2V, M2TS, MTS, VOB, MOD) | Native import | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html | have | ffmpeg decode (ADR-0009) | No repo test |
| **Formats ›** MPEG-4 Part 2 / 3GP | Native import | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html † | have | ffmpeg decode (ADR-0009) | No repo test |
| **Formats ›** DV / DVCPRO / DVCPRO HD | Native import (also in P2 MXF) | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html | have | ffmpeg `dvvideo` decoder (ADR-0009) | Interlaced DV is not deinterlaced (see field-order row) |
| **Formats ›** HDV | Native import (MPEG-2 transport stream) | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html | have | ffmpeg decode (ADR-0009) | Interlaced, no deinterlace (see field-order row) |
| **Formats ›** AVCHD (.mts) | Native camera format; Media Browser reads the card's folder structure | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html | have | Each `.mts` stream decodes through ffmpeg (ADR-0009) | Card-structure reading is a separate row (Media Browser) |
| **Formats ›** Sony XAVC / XAVC S / XAVC HS / XDCAM | Native camera formats (H.264/HEVC/MPEG-2 in MXF or MP4) | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html | have | ffmpeg decode of the essence (ADR-0009) | No repo test |
| **Formats ›** Canon XF (MXF) | Native camera format | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html | have | ffmpeg decode of the essence (ADR-0009) | No repo test |
| **Formats ›** Panasonic P2 (AVC-Intra, AVC-LongG, DVCPRO in MXF OP-Atom) | Native camera format; picture and each audio channel are separate MXF files that Premiere joins into one clip from card metadata | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html | partial | Each MXF essence file decodes through ffmpeg; one `source` per element (ADR-0002) | Nothing joins OP-Atom picture and audio files into one clip: the author must place one `video` plus one `audio` element per file and align them by hand |
| **Formats ›** MXF OP1a | Native import | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html | have | ffmpeg MXF demuxer (ADR-0009) | No repo test |
| **Formats ›** QuickTime intra codecs (Animation, PNG, Uncompressed) | Native import (MOV) | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html | have | PNG-in-MOV is the format `montagent-prerender` writes and verifies byte-exact; `qtrle`/raw via ffmpeg | — |
| **Formats ›** AVI | Native import (codec-dependent) | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html | have | ffmpeg AVI demuxer (ADR-0009) | — |
| **Formats ›** WMV / ASF | Import, Windows only | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html | have | ffmpeg `wmv*` decoders, every OS (ADR-0009) | No repo test |
| **Formats ›** Animated GIF | Import as an animated clip | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html | partial | As `image`: Skia GIF decoder (wuffs, `THIRD-PARTY.md`) | `Raster::decode` keeps the first frame only; a GIF as a `video` source would go through ffmpeg's `gif` decoder, but no doc, skill or test establishes that route |
| **Formats ›** Container vs codec ("MOV, AVI, MXF are containers; readable only if the codec is installed") | Import depends on the installed codec set | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html | have | Same model: the user's ffmpeg is the codec set (ADR-0009); `probe` reports `codec_name` (ADR-0089); no ffmpeg is `E-TOOL-MISSING` (ADR-0091); an undecodable source is `E-NOT-PAINTED-UNDECODABLE` (ADR-0093) | — |
| **Camera raw ›** RED R3D | Native decode with RED Source Settings | https://helpx.adobe.com/premiere-pro/using/supported-file-formats.html | missing | ffmpeg has no R3D decoder; no ADR | Any R3D decode; must be transcoded outside Montagent |
| **Camera raw ›** ARRIRAW (incl. ALEXA 35, MXF ARRIRAW) | Native decode with ARRI source settings | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html | missing | ffmpeg has no ARRIRAW decoder; no ADR | Any ARRIRAW decode (ALEXA 35 ProRes files are covered by the ProRes row) |
| **Camera raw ›** Blackmagic RAW (.braw) | Through Blackmagic's own plugin | https://helpx.adobe.com/be_en/x-productkb/multi/braw-support-adobe-video-applications.html | missing | No plugin model (ADR-0017 closed schema); ffmpeg has no BRAW decoder | Any BRAW decode |
| **Camera raw ›** Apple ProRes RAW | Needs Apple's Pro Video Formats (macOS) or ProRes RAW Decoder (Windows) | https://helpx.adobe.com/uk/x-productkb/multi/prores-raw-support-adobe-video-applications.html | missing | ffmpeg 7.1 (ADR-0115 floor) has no ProRes RAW decoder | Any ProRes RAW decode and its raw controls |
| **Camera raw ›** Canon Cinema RAW Light (.crm) / Canon RAW | Native decode | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html | missing | ffmpeg has no CRM decoder | Any CRM decode |
| **Camera raw ›** Sony RAW / X-OCN (F65, F55, VENICE) | Native decode | https://helpx.adobe.com/premiere-pro/using/supported-file-formats.html | missing | ffmpeg has no Sony RAW / X-OCN decoder | Any Sony RAW decode |
| **Camera raw ›** CinemaDNG | Import as a raw image sequence | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html † | missing | No DNG codec in the Skia pin; not an ffmpeg-supported raw route | Any CinemaDNG decode |
| **Camera raw ›** Raw Source Settings (ISO, white balance, exposure, debayer resolution) | Per-clip decode parameters in the Effect Controls "Source Settings" | https://helpx.adobe.com/premiere-pro/using/supported-file-formats.html † | missing | No raw decode, so no decode parameters | The whole raw-parameter surface |
| **Audio ›** WAV / Broadcast WAV | Native import | https://helpx.adobe.com/my_en/premiere/desktop/organize-media/import-files/importing-digital-audio.html | have | ffmpeg decode, mixed by `render` (ADR-0077 §8) | BWF timecode/metadata is not read (see timecode row) |
| **Audio ›** AIFF / AIF | Native import | https://helpx.adobe.com/my_en/premiere/desktop/organize-media/import-files/importing-digital-audio.html | have | ffmpeg decode (ADR-0009) | — |
| **Audio ›** MP3 | Native import | https://helpx.adobe.com/my_en/premiere/desktop/organize-media/import-files/importing-digital-audio.html | have | ffmpeg decode; the committed fixture uses `audio/intro-2.mp3` (ADR-0053) | — |
| **Audio ›** AAC / M4A | Native import | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html | have | ffmpeg decode (ADR-0009) | — |
| **Audio ›** Sample-rate conform | Imported audio is conformed to the project's audio (cached `.cfa` files) | https://helpx.adobe.com/my_en/premiere/desktop/organize-media/import-files/importing-digital-audio.html | have | Every input is resampled to the 48 kHz stereo bus at mix time (ADR-0077 §8) | No conform cache; the bus rate is fixed, not a project setting |
| **Audio ›** Audio channel interpretation (Modify > Audio Channels: mono, stereo, 5.1, adaptive, channel-to-track mapping) | Per-clip channel format and mapping | https://helpx.adobe.com/my_en/premiere/desktop/organize-media/import-files/importing-digital-audio.html † | partial | Every input is downmixed to stereo by `aformat` (ADR-0077 §8) | No channel choice or remap per source; ADR-0184's `channels` routing field is `proposed` and absent from the v0.2.0 schema; no 5.1 bus |
| **Audio ›** Multiple audio streams in one file | Each stream appears as its own channel/track on import | https://helpx.adobe.com/my_en/premiere/desktop/organize-media/import-files/importing-digital-audio.html † | partial | ffmpeg's default stream selection decides | No key selects a stream by index; `probe`'s `Audio` reports one stream |
| **Audio ›** Use a video file's audio | A/V clip's audio linked on the timeline | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html † | have | A `video` element mixes its audio (`volume`, `audio_effects` in schema); muted with `volume: 0` (footage and prerender skills) | — |
| **Stills ›** PNG (incl. transparency) | Native import | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html | have | Skia PNG codec (vendored libpng, `THIRD-PARTY.md`); fixture `images/06.png` | — |
| **Stills ›** JPEG (EXIF orientation honoured) | Native import | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html | have | Skia `jpeg` feature in the pin; EXIF orientation applied once (`canvas.rs`, test `a_stills_exif_orientation_is_applied_before_it_is_painted`; ADR-0015/0023) | — |
| **Stills ›** GIF (still) | Native import | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html | have | Skia GIF decoder (wuffs, `THIRD-PARTY.md`) | — |
| **Stills ›** BMP / DIB / ICO | Native import | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html | partial | Skia's built-in codecs; not named in `THIRD-PARTY.md` or the pin | Unverified: no test or doc establishes that the pinned prebuilt decodes BMP/ICO |
| **Stills ›** TIFF | Native import | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html | missing | No TIFF codec in the Skia pin | Decode; `validate` passes it (ffprobe reads it) and `render` refuses at `E-NOT-PAINTED-UNDECODABLE` |
| **Stills ›** Targa (TGA) | Native import | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html | missing | No TGA codec in the Skia pin | Decode |
| **Stills ›** DPX | Native import (single frames and sequences) | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html | missing | No DPX codec in the Skia pin | Decode |
| **Stills ›** HEIC / HEIF | Import of HEIF stills | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html † | missing | No HEIF codec in the Skia pin | Decode |
| **Stills ›** OpenEXR | Import (multichannel EXR, layer selection) | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html † | missing | No EXR codec in the Skia pin (`exr` appears only as a test-only crate, `about.toml`) | Decode, float pixels, channel selection |
| **Stills ›** Very large frames (max 256 MP, 32,768 px per side) | Documented import ceiling | https://helpx.adobe.com/premiere-pro/using/supported-file-formats.html | partial | 4K and 8K sources measured (ADR-0046, #87); fixture stills 1536×2720 | No stated or checked maximum source size |
| **Stills ›** Default still-image duration (preference) | Preference sets the length of each imported still | https://helpx.adobe.com/premiere-pro/using/set-preferences.html † | have | Every `image` element states its own `start`/`end` (schema); no implicit default by design (ADR-0017) | — |
| **Stills ›** Default media scaling (Scale to frame size / Set to frame size) | Preference or per-clip command that fits media to the sequence frame | https://helpx.adobe.com/premiere-pro/using/set-preferences.html † | have | `fit`: `cover` / `contain` / `literal` against the declared box (ADR-0015); source dimensions per ADR-0023 | — |
| **Image sequences ›** Import numbered stills as one clip | Select the first file, tick "Image Sequence"; each still becomes one frame | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/import-images-as-image-sequence.html | have (skill) | `montagent-prerender` with `"input": "png"` encodes `f00000.png…` into one PNG-in-MOV, verified frame-exact, placed as a `video` | No element takes a sequence as its `source`; the spec's `render` command must place the frames under the skill's naming; DPX/TIFF/EXR sequences need converting to PNG first. ADR-0140 `swaps` (stepped stills on one image) is accepted but not in the v0.2.0 schema |
| **Image sequences ›** Sequence frame rate (Indeterminate Media Timebase preference) | Preference sets the fps of imported sequences | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/import-images-as-image-sequence.html | have (skill) | `montagent-prerender` spec `fps`, rational allowed (`"30000/1001"`) | — |
| **Source properties ›** Variable frame rate (VFR) media | Imported and conformed; Adobe advises transcoding VFR (phone, screen recordings) to constant rate | https://helpx.adobe.com/premiere-pro/using/supported-file-formats.html | have | ADR-0096: the frame at an instant is the last one starting at or before it, asked of the decoder over a bounded window, exact for constant, fractional, dropped-frame and variable rates (CONTEXT "Sampled frame") | — |
| **Source properties ›** Alpha channels in sources | Read from the file; straight/premultiplied per Interpret Footage | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html | have | ADR-0089: `alpha` is the file's reading (pixel format or container declaration), VP9 side-stream alpha forces `libvpx-vp9`; sidecar v2 | — |
| **Source properties ›** Rotation / orientation metadata | Phone footage shown upright | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html † | have | ADR-0023: container track-level display transform applied; `validate` prints dimensions used; ffmpeg applies it at decode (`decode.rs`) | No fixture exercises video rotation (ADR-0023's own recorded gap) |
| **Source properties ›** HDR sources (HLG, PQ, Rec. 2100) | HDR media detected and handled through colour management | https://helpx.adobe.com/premiere-pro/using/color-management.html † | missing | `decode.rs` converts every frame to 8-bit RGBA with ffmpeg's default conversion; no transfer-function handling | HDR detection, tone mapping, HDR working space (overlaps colour ticket #886) |
| **Source properties ›** High bit depth (10/12-bit, float) | Decoded at source precision into a high-bit pipeline | https://helpx.adobe.com/premiere-pro/using/color-management.html † | partial | 10/12-bit sources decode through ffmpeg | Precision is lost: every frame is RGBA8 (`-pix_fmt rgba`) |
| **Source properties ›** Log / wide-gamut source colour detection | Automatic media colour-space detection, camera log recognition | https://helpx.adobe.com/premiere-pro/using/color-management.html † | missing | No colour-space reading in `probe` (`Probe` fields: quad, dimensions, alpha, codec, audio) | Detection and conversion (overlaps #886) |
| **Source properties ›** Source timecode | Clip start timecode read from file (also BWF/MXF) and shown everywhere | https://helpx.adobe.com/premiere-pro/using/supported-file-formats.html † | missing | Source time is milliseconds from the file's start (`source_start`, ADR-0005); `probe` reports `start_time_ms` only | Timecode reading, display or addressing |
| **Source properties ›** Embedded captions (CEA-608/708) | Read from the video stream on import | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html † | missing | Nothing reads caption streams | Extraction (overlaps captions ticket #888) |
| **Layered & vector ›** Photoshop PSD, merged layers | Import the flattened composite as one still | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/import-photoshop-and-illustrator-files.html | missing | No PSD codec in the Skia pin | Decode; workaround is exporting PNG |
| **Layered & vector ›** Photoshop PSD, individual layers | Choose layers; each becomes its own clip in a bin | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/import-photoshop-and-illustrator-files.html | missing | No PSD decode; no bins (ADR-0002) | Layer extraction (the bin half is refused with the catalogue) |
| **Layered & vector ›** Photoshop PSD as a sequence | Layers become stacked tracks in a new sequence sized to the document | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/import-photoshop-and-illustrator-files.html | missing | Tracks and layers exist (ADR-0004) but nothing builds them from a PSD | PSD layer decode and layout into elements |
| **Layered & vector ›** Illustrator AI / EPS | Import, rasterised (continuously with Vector Motion) | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/import-photoshop-and-illustrator-files.html | refused | ADR-0027: vector sources out of scope (amended by ADR-0171) | — (route: rasterise outside, place PNG; the prerender skill's SVG route takes SVG only) |
| **Layered & vector ›** Vector Motion (continuous rasterisation of vector layers) | Effect that re-rasterises vector media at any scale | https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/edit-vector-graphics-using-vector-motion-effect.html | refused | ADR-0027 / ADR-0171: no vector source enters | — |
| **Layered & vector ›** SVG | Import of SVG graphics (ADR-0171 found no native Premiere SVG import in any source it read) | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html | refused | ADR-0171 §1 refuses SVG as a source; `montagent-prerender` SVG route rasterises once to a PNG `image` | Resolution independence (a PNG is fixed at the largest shown size) |
| **Interpret footage ›** Frame rate ("Assume this frame rate") | Per-clip override of the source rate; changes duration | https://helpx.adobe.com/premiere/desktop/edit-projects/modify-clip-properties/change-the-frame-rate-of-a-clip.html | partial | `speed` retimes a `video`/`audio` element (ADR-0020), evaluated exactly (ADR-0045); `source_time` remap (ADR-0157) | No per-source declared rate: a conform is spelled as a `speed` ratio on every element using the file; project `fps` is an integer (schema), so 23.976/29.97 projects cannot be stated |
| **Interpret footage ›** Pixel aspect ratio | Per-clip PAR override (anamorphic, DV, HDV) | https://helpx.adobe.com/premiere-pro/using/interpreting-footage.html † | partial | `probe` reads the file's PAR into source dimensions and caches it (ADR-0023, ADR-0069); `fit: literal` with a desqueezed box stretches correctly | ADR-0023's `par` override key is accepted but absent from the v0.2.0 schema; `decode.rs` does not apply PAR (open as #274) |
| **Interpret footage ›** Field order / deinterlace | Per-clip field order (upper, lower, none) and field options | https://helpx.adobe.com/premiere-pro/using/interpreting-footage.html † | missing | `decode.rs` passes only `scale=`; no deinterlace filter, no field reading in `probe` | Field detection, override and deinterlacing |
| **Interpret footage ›** Remove pulldown (3:2) | Removes 3:2 / 24pA pulldown on interpretation | https://helpx.adobe.com/premiere-pro/using/interpreting-footage.html † | missing | No inverse-telecine filter | Pulldown removal |
| **Interpret footage ›** Alpha: ignore, invert, premultiplied | Per-clip alpha interpretation | https://helpx.adobe.com/premiere-pro/using/interpreting-footage.html † | partial | Alpha is read and composited (ADR-0089) | No key to ignore, invert or declare premultiplied alpha on a source |
| **Interpret footage ›** Colour space override / input LUT | Override media colour space, apply an input LUT per clip | https://helpx.adobe.com/premiere-pro/using/color-management.html † | missing | No colour management in decode | Override and LUT (overlaps #886) |
| **Interpret footage ›** Modify > Timecode | Set or override a clip's start timecode | https://helpx.adobe.com/premiere-pro/using/interpreting-footage.html † | missing | No timecode model (ADR-0005 milliseconds) | Timecode override |
| **Ingest & proxies ›** Import by path, drag, File > Import | Brings a file into the project catalogue | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html † | have | Naming the file in `source` is the import (ADR-0002); relative to the project file, absolute allowed (ADR-0053); a URL is probed (ADR-0056) | — |
| **Ingest & proxies ›** Ingest: copy media on import, with checksum verification | Ingest settings copy files to a destination and verify | https://helpx.adobe.com/premiere-pro/using/ingest-proxy-workflow.html † | missing | None over stdio; ADR-0159's sha256 report is for a remote instance and is not shipped | Copy-and-verify |
| **Ingest & proxies ›** Ingest: transcode on import | Ingest preset transcodes to a mezzanine codec | https://helpx.adobe.com/premiere-pro/using/ingest-proxy-workflow.html † | missing | `montagent-prerender` encodes only frames its spec draws | Source transcoding |
| **Ingest & proxies ›** Create proxies | Generates low-res proxy files (presets: ProRes, H.264, CineForm) attached to full-res clips | https://helpx.adobe.com/premiere-pro/using/ingest-proxy-workflow.html † | missing | Montagent's "proxy" is a render resolution tier, not a file (ADR-0046, ADR-0065) | Proxy media files of any kind |
| **Ingest & proxies ›** Attach / reattach proxies | Link existing low-res files as a clip's proxy | https://helpx.adobe.com/premiere-pro/using/ingest-proxy-workflow.html † | missing | No proxy files to attach | Attach, reattach |
| **Ingest & proxies ›** Toggle proxies for playback | Button switches playback between proxy and full-res | https://helpx.adobe.com/premiere-pro/using/ingest-proxy-workflow.html † | partial | `preview` degrades to a 720p tier then 540p, disclosed, hard floor (ADR-0046, ADR-0065, ADR-0050); proxy tiers sample one mip level (ADR-0186) | Decode still reads full-size sources (ADR-0065: 8K is decode-bound), so the decode saving a proxy file buys is absent; no author toggle beyond `--full` |
| **Ingest & proxies ›** Use proxies on export | Export setting renders from proxies | https://helpx.adobe.com/premiere-pro/using/ingest-proxy-workflow.html † | refused | ADR-0186: the deliverable (`render`, `frame`, `preview --full`) always paints true pixels | — |
| **Ingest & proxies ›** Media Browser | Panel browsing disks and cards; reads camera folder structures (P2, XDCAM, AVCHD), merges spanned clips, previews before import | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html † | missing | ADR-0011: the agent brings its own file tools; no card-structure reader | Card-structure reading and spanned-clip joining |
| **Ingest & proxies ›** Spanned clips | A recording split across files plays as one clip | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html † | missing | One element, one file (ADR-0002) | Joining; today the author abuts elements by hand |
| **Ingest & proxies ›** Import mode (2023+ import workspace) | Full-screen import UI: recent locations, copy media, new bin, create sequence | https://helpx.adobe.com/premiere-pro/using/import-mode.html † | missing | No GUI (ADR-0011) | The import workspace; its sequence half is the next row |
| **Ingest & proxies ›** New sequence from clip (match settings) | Sequence takes the clip's frame size and rate | https://helpx.adobe.com/premiere-pro/using/import-mode.html † | partial | `probe` reports size and rates; `create_project` writes the header it is told (ADR-0080) | No derive-from-source; integer `fps` cannot match a 29.97 source |
| **Ingest & proxies ›** Hardware-accelerated decoding | GPU decode of H.264/HEVC | https://helpx.adobe.com/premiere/desktop/get-started/technical-requirements/enable-hardware-accelerated-decoding-support.html | missing | `decode.rs` passes no `-hwaccel` | GPU decode (overlaps performance ticket #894) |
| **Ingest & proxies ›** Remote / cloud-hosted media | Media in cloud storage (Creative Cloud, Frame.io) brought into projects | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html † | partial | A URL `source` is legal and probed per session (ADR-0002, ADR-0056); `validate` errors it under `render`'s codes (ADR-0131) | `render`/`frame` use local sources only (ADR-0131); ADR-0159's remote instance with download cache is accepted but v0.2.0 is stdio-only |
| **Ingest & proxies ›** Import After Effects compositions (Dynamic Link) | Live link to an AE comp | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html † | missing | `montagent-prerender` takes Lottie or code, not AE projects | Dynamic link; nearest route is a rendered MOV |
| **Ingest & proxies ›** Import projects and sequences (PRPROJ, FCP XML, AAF, EDL) | Imports another project's items/sequences | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html † | missing | No interchange reader | All interchange import (overlaps #892) |
| **Ingest & proxies ›** Caption sidecar import (SRT, SCC, MCC, STL, DFXP) | Import caption files as caption tracks | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html | missing | `montagent-footage`'s `captions.py` takes a word-timings JSON, not caption files | Caption-file parsing (overlaps #888) |
| **Ingest & proxies ›** Creative Cloud Libraries / Adobe Stock | Import assets from Libraries panel and Stock | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html † | missing | No library or stock connector | All of it |
| **Ingest & proxies ›** Motion Graphics templates (.mogrt) import | Install and use templates | https://helpx.adobe.com/premiere/desktop/add-text-images/use-motion-graphics-templates/about-motion-graphics-templates.html | missing | No template format | All of it (overlaps #887) |
| **Keeping linked ›** Offline media detection | Missing files show as offline in the Project panel | https://helpx.adobe.com/premiere-pro/using/set-preferences.html † | have | A missing `source` is a plain `error`, `E-SOURCE-MISSING` (ADR-0053); `validate` probes every source on every run (ADR-0006) | — |
| **Keeping linked ›** Link Media / relink | Dialog finds moved files by name, extension, media start, tape name; "relink others automatically" | https://helpx.adobe.com/premiere-pro/using/set-preferences.html † | partial | Relinking is rewriting `source` strings, an exact-string edit (ADR-0002, ADR-0145 invariant 4) | No search for a moved file by name or attributes, and no relink-all; ADR-0002 leaves replace-all to tooling that does not exist |
| **Keeping linked ›** Media Offline placeholder in playback and export | Missing media renders a "Media Offline" frame | https://helpx.adobe.com/premiere-pro/using/set-preferences.html † | refused | ADR-0093: an `error` withholds the deliverable; ADR-0053: missing source is an `error` | — |
| **Keeping linked ›** Replace footage / make offline | Swap a clip's underlying file | https://helpx.adobe.com/premiere/desktop/render-and-export/render-sequences-for-playback/render-and-replace-media-in-a-sequence.html † | have | Edit `source` (ADR-0002); `probe` re-reads the new file | — |
| **Keeping linked ›** Relative-path relink when a project moves | Premiere searches relative locations | https://helpx.adobe.com/premiere-pro/using/set-preferences.html † | have | Relative paths resolve against the project file's directory (ADR-0053) | — |
| **Keeping linked ›** Changed-on-disk detection | Re-reads media changed outside Premiere | https://helpx.adobe.com/premiere-pro/using/set-preferences.html † | have | Probe sidecar keyed on `(canonical path, size, mtime)` plus a content fingerprint (ADR-0069, ADR-0092) | — |
| **Keeping linked ›** Media cache (conformed audio, peak files, index files) | Cache files and database speeding playback | https://helpx.adobe.com/premiere-pro/using/set-preferences.html † | partial | Per-user JSON probe sidecar (ADR-0069, version 2 per ADR-0089) | Caches probe facts only: no conformed audio, waveform peaks or decode indexes |
| **Keeping linked ›** Media cache management (location, clean, auto-delete by age/size) | Preferences > Media Cache | https://helpx.adobe.com/premiere-pro/using/set-preferences.html † | partial | `montagent cache clear` (CLI-only, ADR-0092) | No location, size or age setting; not an MCP tool |
| **Keeping linked ›** Project Manager (collect files, consolidate and transcode) | Copies or trims used media to a new location | https://helpx.adobe.com/premiere-pro/using/project-manager.html † | missing | ADR-0159's "upload what is missing or different" is remote-only and not shipped | Collect, consolidate, trim-to-used |
| **Keeping linked ›** Render and replace | Replaces a clip with a rendered flattened file | https://helpx.adobe.com/premiere/desktop/render-and-export/render-sequences-for-playback/render-and-replace-media-in-a-sequence.html | missing | `montagent-prerender` flattens only code/Lottie/SVG pieces | Flattening an element or range back to a source |
| **Keeping linked ›** Merge clips (sync separate audio and video) | Combines A/V into one clip, synced by timecode or audio | https://helpx.adobe.com/premiere-pro/using/merge-clips.html † | partial | Separate `video` and `audio` elements can share a timeline range | No synchronisation by timecode or waveform; no merged item |
| **Organising ›** Project panel (catalogue of master clips) | Every imported file is a project item | https://helpx.adobe.com/premiere-pro/using/organizing-assets-project-panel.html † | refused | ADR-0002: no asset table; ADR-0053: no `assetRoot` | — |
| **Organising ›** Bins and nested bins | Folders inside the catalogue | https://helpx.adobe.com/premiere-pro/using/organizing-assets-project-panel.html † | refused | ADR-0002 (no catalogue to hold bins) | — (filesystem folders stand in) |
| **Organising ›** Subclips | Named, reusable in/out ranges of a master clip | https://helpx.adobe.com/premiere-pro/using/organizing-assets-project-panel.html † | refused | ADR-0002: no named indirection; each element carries its own `source_start`/`source_end` | — |
| **Organising ›** Freeform view (storyboarding in the Project panel) | Arrange catalogue thumbnails freely | https://helpx.adobe.com/premiere-pro/using/organizing-assets-project-panel.html † | refused | ADR-0002 (no catalogue items to arrange) | — |
| **Organising ›** Thumbnails, hover scrub, Source Monitor preview | View a source's content before using it | https://helpx.adobe.com/premiere-pro/using/organizing-assets-project-panel.html † | have (skill) | `montagent-footage` "Start from the footage": put the clip alone in a scratch project, `frame --from 0 --to <len> --infill-ceiling`, then `frame --at` | — |
| **Organising ›** File properties (Properties dialog) | Shows codec, size, rates, duration | https://helpx.adobe.com/premiere-pro/using/organizing-assets-project-panel.html † | have | `montagent probe` (ADR-0011 quad, dimensions, alpha, `codec_name`, audio) | CLI-only (ADR-0011) |
| **Organising ›** Metadata panel / XMP | View and edit descriptive and technical metadata, custom schemas | https://helpx.adobe.com/premiere-pro/using/organizing-assets-project-panel.html † | partial | Technical facts via `probe` | No descriptive or XMP metadata read or write |
| **Organising ›** Labels (colour labels) | Colour-code items and clips | https://helpx.adobe.com/premiere-pro/using/organizing-assets-project-panel.html † | partial | `group`: free-text label on elements (CONTEXT "Group") | No colour; nothing labels a source |
| **Organising ›** Search / filter the Project panel | Find items by name or metadata | https://helpx.adobe.com/premiere-pro/using/organizing-assets-project-panel.html † | partial | `query --where` conjunction predicates over elements (ADR-0070) | No source catalogue to search; no metadata search |
| **Organising ›** Search bins (saved searches) | Bins that update from a saved query | https://helpx.adobe.com/premiere-pro/using/organizing-assets-project-panel.html † | missing | `query` predicates are not stored | Saved queries |
| **Organising ›** Productions | Multi-project folder with shared settings, cross-project references and locking | https://helpx.adobe.com/premiere-pro/using/productions.html † | missing | Each project is one file (ADR-0011) | Cross-project references, shared cache, locking |
| **Organising ›** Media Intelligence / visual search | Search footage by visual content and transcript | https://helpx.adobe.com/premiere-pro/using/organizing-assets-project-panel.html † | missing | Nothing indexes content | Content search (overlaps AI ticket #893) |
| **Immersive ›** 360° monoscopic (equirectangular) import | Interpret as VR: projection, layout, captured view | https://helpx.adobe.com/premiere-pro/using/VRSupport.html † | missing | No projection metadata or VR reading | VR interpretation |
| **Immersive ›** Stereoscopic VR (over/under, side-by-side) | Stereo layout in VR properties | https://helpx.adobe.com/premiere-pro/using/VRSupport.html † | missing | Nothing reads stereo layout | Stereo handling |
| **Immersive ›** VR video viewing (Monitor VR mode, headset) | Navigable 360 view while editing | https://helpx.adobe.com/premiere-pro/using/VRSupport.html † | missing | `frame`/`preview` draw flat frames | VR viewing |
| **Immersive ›** VR metadata on export | Writes spherical metadata for platforms | https://helpx.adobe.com/premiere-pro/using/VRSupport.html † | missing | `render` writes no spherical metadata | Spherical metadata (overlaps #892) |
| **Immersive ›** Ambisonics audio | Imports first-order ambisonic audio | https://helpx.adobe.com/premiere-pro/using/VRSupport.html † | missing | Stereo bus only (ADR-0077 §8) | Ambisonic channel handling |
| **Immersive ›** Stereoscopic 3D (non-VR) | Legacy S3D clips | https://helpx.adobe.com/premiere-pro/using/VRSupport.html † | missing | None | Stereo pairs |

## Notable gaps

- **Camera raw is entirely missing**: R3D, ARRIRAW, BRAW, ProRes RAW, Canon CRM, Sony RAW/X-OCN and
  CinemaDNG. ffmpeg (ADR-0009, ADR-0115) has no decoder for any of them, so the only route is
  transcoding outside Montagent.
- **Stills beyond PNG, JPEG and GIF**: TIFF, TGA, DPX, PSD, HEIF and EXR are missing from the Skia
  pin. Worse, `validate` passes them (ffprobe reads them) and only `render` refuses
  (`E-NOT-PAINTED-UNDECODABLE`), which is the kind of validate/render split ADR-0131 closed for
  remote sources.
- **Decode is 8-bit RGBA with no colour pipeline**: no HDR, high-bit-depth, log or colour-space
  handling, no deinterlace or pulldown removal. Interlaced DV/HDV/XDCAM is drawn combed.
- **Interpret footage is mostly unshipped**: ADR-0023's `par` key is accepted but not in the v0.2.0
  schema, and `decode.rs` skips PAR (#274). The frame-rate override exists only as a per-element
  `speed`. The integer project `fps` cannot state 23.976 or 29.97.
- **Proxies mean something else**: Montagent's 720p/540p `preview` tiers (ADR-0046/0065/0186) cut
  paint cost but not decode cost, and there are no proxy files to create, attach or toggle.
- **No ingest or card handling**: no copy-and-verify, transcode-on-ingest, Media Browser
  card-structure reading, spanned-clip joining or OP-Atom picture/audio assembly.
- **Audio channels**: every input is downmixed to the 48 kHz stereo bus. ADR-0184's `channels`
  routing is still `proposed`, and there is no stream selection or 5.1/ambisonics.
- **Relink is a manual string edit**: offline detection is strong (`E-SOURCE-MISSING`, the content
  guard of ADR-0092), but nothing searches for a moved file.
- **Refused by design**: the Project panel, bins, subclips and freeform view (ADR-0002, no asset
  table), AI/EPS, SVG and Vector Motion (ADR-0027, ADR-0171; the SVG route is the prerender
  skill), Media Offline frames in a deliverable (ADR-0093), and exporting from proxies (ADR-0186).
- **Accepted but unshipped in v0.2.0**: ADR-0023 `par`, ADR-0140 `swaps`, and ADR-0159's remote
  instance (stdio only). A URL source is validated but never rendered (ADR-0131).
- **Premiere sources not fetched**: helpx.adobe.com was unreachable from this session. Every †
  URL, and the mechanism text on † rows, should be checked against the census
  [#883](https://github.com/MBehtemam/Montagent/issues/883) before assembly in #895.
