---
status: accepted
amends: 0216 (re-affirms Rec.709 1-1-1 video boundaries; HDR deliverables and tone mapping remain downstream)
---

# In-engine HDR encoding and tone mapping are refused from render, and HDR mastering stays downstream

[#1109](https://github.com/MBehtemam/Montagent/issues/1109), on the map
[#1080](https://github.com/MBehtemam/Montagent/issues/1080), from the census in
[#904](https://github.com/MBehtemam/Montagent/issues/904) (rows 17, 29, 30) and
[#908](https://github.com/MBehtemam/Montagent/issues/908).

## The question

After Effects supports exporting High Dynamic Range (HDR) video deliverables in Rec.2100 PQ
(`smpte2084`) and HLG (`arib-std-b67`) color spaces with 10-bit / 12-bit encoding and embedded
CICP metadata tags, alongside in-engine tone mapping effects.

Montagent standardizes on 8-bit display-referred sRGB compositing and encodes deliverables tagged
Rec.709 `1-1-1` ([ADR-0216](0216-srgb-is-the-working-colour-space-and-video-boundaries-convert-and-tag-bt709.md))
across H.264, Apple ProRes 4444, and Avid DNxHR ([ADR-0203](0203-prores-and-dnxhr-mezzanines-join-output-format.md),
[ADR-0229](0229-image-sequence-export-and-png-in-mov-are-refused-from-render-and-prores-4444-is-the-sole-alpha-deliverable.md)).

Do HDR deliverables (`rec2100_pq`, `rec2100_hlg`) and in-engine tone mapping enter `output.video`
in `render`, or does HDR mastering remain downstream from lossless ProRes 4444 masters?

## The decision

### 1. In-engine HDR encoding and tone mapping are refused from `render`

1. **No super-whites on canvas**: Montagent's canvas operates within a standard SDR gamut and dynamic
   range (0..255 peak, nominal 100 nits). Synthesizing HDR signals directly from an SDR canvas
   requires either artificial tone expansion (inverse tone mapping) or complex scene-referred metadata
   systems that violate Montagent's flat literal values ([ADR-0145](0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md)).
2. **Deterministic encoding boundary**: `render` relies on deterministic FFmpeg encoder flags
   ([ADR-0143](0143-the-only-deliverable-is-a-yuv420p-mp4-and-encoder-settings-do-not-enter.md),
   [ADR-0203](0203-prores-and-dnxhr-mezzanines-join-output-format.md),
   [ADR-0216](0216-srgb-is-the-working-colour-space-and-video-boundaries-convert-and-tag-bt709.md)).
   Introducing PQ/HLG EOTF curves, MaxFALL / MaxCLL mastering metadata injection, and tone mapping
   algorithms into `montagent-render` bloats the deliverable pipeline with specialized color science
   better handled by dedicated finishing suites.

### 2. What to do instead: Downstream HDR mastering via FFmpeg recipes

HDR finishing is performed downstream on Montagent's lossless Apple ProRes 4444 deliverables
(`prores_4444`, `alpha: false`, 10/12-bit mezzanine per ADR-0203).

A recipe in `skills/montagent-footage/SKILL.md` documents standard FFmpeg conversion and tagging for
Rec.2100 HLG and PQ deliverables:

```bash
# Conform ProRes 4444 master to 10-bit HLG (Rec.2100 HLG, BT.2020 primaries)
ffmpeg -i input_prores.mov \
  -vf "zscale=tin=srgb:pin=bt709:rin=limited:t=arib-std-b67:p=bt2020:m=bt2020nc:r=limited,format=yuv420p10le" \
  -c:v libx265 -tag:v hvc1 -crf 18 -preset slow \
  -color_primaries bt2020 -color_trc arib-std-b67 -colorspace bt2020nc \
  output_hlg.mp4
```
