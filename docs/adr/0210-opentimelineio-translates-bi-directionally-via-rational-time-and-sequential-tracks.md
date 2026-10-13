---
status: accepted
amends: 0004 (tracks), 0145 (four invariants), 0199 (rational timebases), 0209 (interchange architecture)
---

# OpenTimelineIO translates bi-directionally via rational time and sequential tracks

[#978](https://github.com/MBehtemam/Montagent/issues/978), [#1038](https://github.com/MBehtemam/Montagent/issues/1038), [#1040](https://github.com/MBehtemam/Montagent/issues/1040).

## Context

OpenTimelineIO (OTIO) is the open-source interchange standard fostered by the Academy Software Foundation (ASWF) and adopted by major NLEs and studios (including Adobe Premiere Pro 26.x, DaVinci Resolve, Final Cut Pro, and Blender).

In [#978](https://github.com/MBehtemam/Montagent/issues/978) and `docs/design/premiere-parity/ROADMAP.md:229`, Montagent committed to providing an OTIO adapter translating OTIO JSON timelines to Montagent declarative projects and back.

OTIO represents timelines as a hierarchical tree of schema objects:
- `Timeline`: Root container holding metadata and a top-level `Stack`.
- `Stack`: Evaluates child tracks concurrently in compositing layer order.
- `Track`: Evaluates child items sequentially in time (`kind: "Video"` or `"Audio"`).
- `Clip`: Media slice referencing an external asset, with a `source_range` (`RationalTime` start and duration).
- `Gap`: Empty temporal duration between clips.
- `Transition`: Crossfade or cut transition between adjacent track items.
- `LinearTimeWarp`: Linear retiming effect modifying playback speed.

Montagent models projects as a collection of non-overlapping tracks containing sequential elements positioned at absolute integer millisecond coordinates `[start, end]` ([ADR-0004](0004-tracks-are-sequences-of-elements.md), [ADR-0005](0005-absolute-integer-milliseconds.md), [ADR-0199](0199-fractional-and-ntsc-timebases-are-exact-rational-fractions-and-timeline-points-stay-integer-milliseconds.md)).

A bidirectional mapping must reconcile OTIO's sequential gap-based model and rational time representations with Montagent's absolute millisecond timeline coordinates without loss or temporal drift.

## Decision

1. **Temporal Mapping & Rational Time Translation**:  
   OTIO expresses time as `RationalTime { value: f64, rate: f64 }`.  
   - When importing from OTIO, time is converted to integer milliseconds using exact 128-bit integer arithmetic:
     $$\text{ms} = \left\lfloor \frac{\text{value} \times 1000}{\text{rate}} + 0.5 \right\rfloor$$
   - When `rate` is fractional (e.g. NTSC $23.976$ or $29.97$ fps), the rate is matched to Montagent's standardized rational fractions ($24000/1001$, $30000/1001$, $60000/1001$) under [ADR-0199](0199-fractional-and-ntsc-timebases-are-exact-rational-fractions-and-timeline-points-stay-integer-milliseconds.md), and sampled using the 1001 ms periodicity theorem.
   - When exporting to OTIO, integer milliseconds are converted to `RationalTime`:
     $$\text{value} = \frac{\text{ms} \times \text{project.fps}}{1000}$$

2. **Track & Hierarchy Ingestion**:  
   - An OTIO `Timeline` maps to a Montagent project root document.
   - The top-level OTIO `Stack` maps to Montagent's `tracks` array.
   - OTIO `Track` objects with `kind: "Video"` map to Montagent video tracks, ordered by layer index.
   - OTIO `Track` objects with `kind: "Audio"` map to Montagent audio tracks.
   - Sequential items within an OTIO track are accumulated along a temporal timeline cursor:
     - `Gap`: Advances the timeline cursor by `gap.duration`. No element is emitted, naturally preserving empty space between elements.
     - `Clip`: Emits a Montagent element (`video` or `audio`) with:
       - `start = cursor`
       - `end = cursor + clip.duration`
       - `source = clip.media_reference.target_url` (normalized relative to project directory)
       - `source_start = clip.source_range.start_time`
       - `source_end = clip.source_range.start_time + clip.duration`
       - Advances cursor by `clip.duration`.
     - `LinearTimeWarp`: Translated directly to `speed: clip.time_scalar`.

3. **Export Gap Synthesis**:  
   In Montagent, gaps between elements on a track are implicit (the absence of an element between $t_1$ and $t_2$).  
   When exporting a Montagent track to OTIO:
   - If the first element starts at $t > 0$, an initial OTIO `Gap` of duration $t$ is emitted.
   - For every pair of adjacent elements where $E_n.\text{end} < E_{n+1}.\text{start}$, an OTIO `Gap` of duration $E_{n+1}.\text{start} - E_n.\text{end}$ is inserted between them.
   - This ensures the exported OTIO sequence is continuous and non-overlapping.

4. **Transitions & Markers**:  
   - OTIO `Transition` objects between clips map to Montagent `transition` properties or audio crossfades.
   - OTIO `Marker` objects on clips or timeline tracks map to literal metadata or chapter intervals.

5. **Round-Trip Lossless Invariant**:  
   A legal Montagent project exported to OTIO and re-imported back to Montagent produces an identical set of tracks, elements, and millisecond time intervals, satisfying `montagent validate`.
