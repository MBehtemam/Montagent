---
status: accepted
amends: 0004 (tracks), 0145 (four invariants), 0199 (rational timebases), 0209 (interchange architecture)
---

# CMX 3600 EDL and FCPXML translate via SMPTE timecode and lane mapping

[#978](https://github.com/MBehtemam/Montagent/issues/978), [#1038](https://github.com/MBehtemam/Montagent/issues/1038), [#1041](https://github.com/MBehtemam/Montagent/issues/1041).

## Context

Alongside OpenTimelineIO ([ADR-0210](0210-opentimelineio-translates-bi-directionally-via-rational-time-and-sequential-tracks.md)), two interchange formats dominate professional film and broadcast workflows:
1. **CMX 3600 EDL**: The universal legacy standard for linear cut lists. Every broadcast NLE (Premiere, Avid Media Composer, DaVinci Resolve) imports and exports CMX 3600 EDLs.
2. **Apple Final Cut Pro XML (FCPXML)**: The primary multi-track interchange format used for exchanging complex sequences, connected clips, and multi-channel audio across Apple Final Cut Pro, DaVinci Resolve, and Adobe Premiere Pro (via XML translation).

Montagent must ingest both formats into valid declarative projects and export Montagent timelines back to standard CMX 3600 EDLs and FCPXML documents without external renderer dependencies ([ADR-0209](0209-timeline-interchange-is-a-decoupled-zero-renderer-conversion-engine.md)).

## Decision

1. **SMPTE Timecode Resolution**:  
   CMX 3600 EDL and FCPXML specify timeline positions via SMPTE timecodes (`HH:MM:SS:FF` for Non-Drop Frame, `HH:MM:SS;FF` for Drop Frame) or rational seconds (`<rational>s` in FCPXML).
   - Timecodes are parsed into absolute frame counts and converted to integer milliseconds using the sequence frame rate:
     $$\text{ms} = \left\lfloor \frac{\text{frame\_index} \times 1000 \times \text{den}}{\text{num}} \right\rfloor$$
   - SMPTE Drop-Frame calculation drops 2 frames per minute except every 10th minute at 29.97 fps (30000/1001), ensuring exact alignment with broadcast reference clocks.

2. **CMX 3600 EDL Ingestion & Export**:  
   - **Line Parsing**: Each EDL event line is parsed:
     `{Event}  {Reel}  {Track}  {Type}  [{TransDuration}]  {SrcIn} {SrcOut} {RecIn} {RecOut}`
   - **Channel Routing**:
     - `V` channel events are assigned to the primary video track (`name: "video"`, `layer: 1`).
     - `A`, `A2`, or `AA` channel events are mapped to audio tracks (`name: "audio_1"`, `name: "audio_2"`).
   - **Media Resolution**: Clip filenames are extracted from subsequent comment lines (`* FROM CLIP NAME: <filename>`).
   - **EDL Export**: When exporting a Montagent project to CMX 3600, each element on track 0/1 is formatted into a 3-digit event index with SMPTE timecode columns, writing `* FROM CLIP NAME:` comments for source filenames.

3. **FCPXML Ingestion & Lane Mapping**:  
   - **Resources & Format**: The `<format>` element defines `width`, `height`, and `frameDuration` (e.g. `1001/30000s`), directly establishing `project.fps` and `project.frame`.
   - **Spine & Lanes**:
     - Elements within the primary `<spine>` represent sequential cuts at layer 0.
     - Secondary connected clips (`<clip lane="1">`, `<clip lane="2">`) map to higher-indexed video tracks (`layer: 1`, `layer: 2`).
     - Audio clips with negative lanes (`<asset-clip lane="-1">`) or dedicated audio tracks map to Montagent audio tracks.
     - `<gap duration="...">` elements inside the spine advance the timeline cursor without creating elements.

4. **Validation Guarantee**:  
   Importing an EDL or FCPXML yields a canonical Montagent project document with non-overlapping tracks and explicit millisecond coordinates, guaranteed to pass `montagent validate`.
