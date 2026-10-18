---
status: accepted
amends: 0001 (flat element list), 0004 (tracks), 0145 (four invariants), 0199 (rational timebases)
---

# Timeline interchange is a decoupled zero-renderer conversion engine

> **Amended by [ADR-0212](./0212-standalone-subtitles-translate-via-cue-intervals-and-text-runs.md)**: Standalone subtitles translate via cue intervals and text runs.

> **Amended by [ADR-0211](./0211-cmx-3600-edl-and-fcpxml-translate-via-smpte-timecode-and-lane-mapping.md)**: CMX 3600 EDL and FCPXML translate via SMPTE timecode and lane mapping.

> **Amended by [ADR-0210](./0210-opentimelineio-translates-bi-directionally-via-rational-time-and-sequential-tracks.md)**: OpenTimelineIO translates bi-directionally via rational time and sequential tracks.

[#978](https://github.com/MBehtemam/Montagent/issues/978), [#1038](https://github.com/MBehtemam/Montagent/issues/1038), [#1039](https://github.com/MBehtemam/Montagent/issues/1039).

## Context

Professional video editing relies on cross-NLE timeline interchange formats to exchange edit decision lists, multi-track sequences, and subtitling between editors, colorists, and post-production suites (`docs/research/premiere-gap/editing.md`, `docs/research/premiere-gap/import.md`, `docs/research/premiere-gap/export.md`):
- **OpenTimelineIO (OTIO)**: ASWF standard JSON interchange for tracks, stacks, clips, gaps, transitions, and time warps.
- **CMX 3600 EDL**: The universal broadcast standard for linear cut lists.
- **Final Cut Pro XML (FCPXML)**: Widely used multi-track interchange across Premiere, DaVinci Resolve, and Final Cut Pro.
- **SubRip (SRT) & WebVTT (VTT)**: Universal timed-text subtitle formats.

In Montagent, project documents are authored declaratively and manipulated by AI agents over MCP and CLI (`CONTEXT.md`). An agent receiving an OTIO file or an EDL must be able to convert it into a valid Montagent project without running rendering engines, compiling Skia shaders, or invoking FFmpeg subprocesses.

Montagent requires a principled architectural boundary for timeline interchange that upholds the four invariants of [ADR-0145](0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md) and enforces zero-renderer dependencies.

## Decision

1. **Decoupled Crate Architecture (`crates/montagent-interchange`)**:  
   Format interchange is isolated in a dedicated workspace crate: `crates/montagent-interchange`.
   - `montagent-interchange` depends on `montagent-core` (for schema models: `Project`, `Track`, `Element`, `RationalFps`) and pure serialization crates (`serde`, `serde_json`, `quick-xml`).
   - `montagent-interchange` has **strictly zero dependencies** on `montagent-render`, `skia-safe`, OpenGL/Vulkan, or FFmpeg media subprocesses.
   - It is a pure, hermetic document transformation engine.

2. **The Zero-Renderer Policy**:  
   Interchange converters translate syntax and temporal structure:
   - Importers map foreign timeline objects into declarative tracks and literal elements with integer millisecond coordinates `[start, end]`.
   - Importers **never** probe media files, decode video streams, rasterize frames, or compute ink extents.
   - Asset existence, container compatibility, stream codec validation, and duration integrity are deferred entirely to `montagent validate` post-import.

3. **Structural Import Guarantee**:  
   Every imported project produced by `montagent-interchange` must:
   - Conform strictly to `schema/montagent.schema.json`.
   - Obey the track non-overlap invariant ([ADR-0004](0004-tracks-are-sequences-of-elements.md)) and layer ordering invariants.
   - Express frame rates using integer or rational representations (`{"num": N, "den": D}`) under [ADR-0199](0199-fractional-and-ntsc-timebases-are-exact-rational-fractions-and-timeline-points-stay-integer-milliseconds.md).
   - Pass `montagent validate` without syntax or structural errors.

4. **Foreign Feature Degradation & Diagnostic Reporting**:  
   When an external format declares features unsupported by Montagent (such as non-linear bezier speed warps, arbitrary third-party plugin metadata, or multi-angle multicam groupings):
   - The importer preserves the underlying cut points and media references losslessly.
   - Unsupported effects or metadata are dropped or mapped to nearest closed-vocabulary constructs.
   - The importer returns a structured `InterchangeReport` detailing any skipped attributes or translated elements, allowing agents and users to inspect conversion fidelity.

5. **CLI & MCP Entry Points**:  
   Interchange is exposed through uniform CLI subcommands in the `montagent` binary:
   - `montagent import <format> <input-path> -o <output-project.json>`
   - `montagent export <format> <project-file> -o <output-path>`
   Where `<format>` is one of the closed vocabulary values: `otio`, `edl`, `fcpxml`, `srt`, `vtt`.
