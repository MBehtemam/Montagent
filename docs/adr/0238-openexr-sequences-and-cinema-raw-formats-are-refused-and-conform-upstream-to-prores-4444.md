---
status: accepted
amends: 0002 (container video ingest standards), 0171 (proprietary runtime engines stay upstream), 0203 (ProRes 4444 is the standard high-fidelity mezzanine), 0229 (numbered image sequences are refused)
---

# OpenEXR sequences and cinema raw formats are refused, and conform upstream to ProRes 4444

[#1111](https://github.com/MBehtemam/Montagent/issues/1111), on the map
[#1080](https://github.com/MBehtemam/Montagent/issues/1080), from the census in
[#907](https://github.com/MBehtemam/Montagent/issues/907) (rows 56–59) and
[#908](https://github.com/MBehtemam/Montagent/issues/908).

## The question

After Effects supports ingesting multi-channel OpenEXR (`.exr`) floating-point frame sequences
and proprietary camera raw formats (RED `.r3d`, Blackmagic `.braw`, ARRI raw, Sony X-OCN, Canon
Cinema RAW).

Montagent standardized on single-file video containers ([ADR-0002](0002-video-ingest-container-and-codec-whitelist.md)),
refused numbered image sequence output ([ADR-0229](0229-image-sequence-export-and-png-in-mov-are-refused-from-render-and-prores-4444-is-the-sole-alpha-deliverable.md)),
and refused proprietary binary SDK engines ([ADR-0171](0171-svg-and-lottie-do-not-enter-as-sources-and-each-is-pre-rendered-through-the-skill.md),
[ADR-0233](0233-3d-models-and-multichannel-passes-are-refused-as-native-sources-and-3d-scenes-pre-render-via-headless-recipes.md)).

Are `.exr` float sequences and proprietary camera raw formats refused as native sources in Montagent,
and how are they conformed upstream?

## The decision

### 1. OpenEXR sequences and camera raw formats are refused as native sources

1. **Vendor SDK bloat and license conflicts**: Ingesting `.braw`, `.r3d`, and `.arri` files natively
   requires embedding heavy, proprietary C++ SDKs with closed GPU acceleration drivers. This breaches
   Montagent's lightweight, standalone build invariants and introduces platform-dependent decode
   variations.
2. **Multi-file atomicity**: OpenEXR sequences scatter thousands of individual `.exr` frames across
   folders, conflicting with project portability, fast cache checks, and single-asset validation
   ([ADR-0011](0011-deliverable-is-one-mp4-and-interchange-is-an-archive.md),
   [ADR-0229](0229-image-sequence-export-and-png-in-mov-are-refused-from-render-and-prores-4444-is-the-sole-alpha-deliverable.md)).

### 2. What to do instead: Conform upstream to Apple ProRes 4444 or DNxHR HQX

Camera raw footage and EXR sequences are conformed upstream into standard 10-bit / 12-bit
Apple ProRes 4444 (`.mov`) or Avid DNxHR HQX (`.mov`) files with embedded Rec.709 tags ([ADR-0203](0203-prores-and-dnxhr-mezzanines-join-output-format.md),
[ADR-0216](0216-srgb-is-the-working-colour-space-and-video-boundaries-convert-and-tag-bt709.md)).

The recipe is documented in `skills/montagent-footage/SKILL.md`:

```bash
# Conform an EXR image sequence to 10-bit ProRes 4444 MOV with BT.709 tagging
ffmpeg -framerate 24 -i "sequence_%05d.exr" \
  -vf "format=gbrpf32le,zscale=tin=linear:pin=bt709:t=bt709:p=bt709:m=bt709,format=yuv444p10le" \
  -c:v prores_ks -profile:v 4 -vendor apl0 -pix_fmt yuv444p10le \
  -color_primaries bt709 -color_trc bt709 -colorspace bt709 \
  conformed_footage.mov
```

Conformed ProRes 4444 files load directly into `video` elements with instant seek performance and
zero external runtime dependencies.
