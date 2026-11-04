---
status: accepted
amends: 0011 (reaffirms deliverable is a single container file and chapter tracks write into that container), 0086 (reaffirms timeline intent; markers provide explicit, author-declared literal annotations on timeline and elements without soft beat-grid heuristics), 0145 (reaffirms literal values, closed vocabulary, checkable by validate, exact-string replace), 0202 (preserves output container format inference), 0210 (fulfills OTIO Marker bidirectional translation mapping to project and element markers), 0215 (disposes of After Effects marker rows 15–19 from map #896)
---

# Markers enter project and elements as render-inert annotations, and chapter markers export to container metadata

[Do project and element markers enter project.json, or stay in external sidecars under ADR-0086?](https://github.com/MBehtemam/Montagent/issues/1072)
on the After Effects map ([#896](https://github.com/MBehtemam/Montagent/issues/896)).

[ADR-0011](0011-deliverable-is-written-to-sibling-temp-and-renamed.md) established that deliverable output is written to a single atomic container file.
[ADR-0086](0086-recorded-intent-is-one-pattern-and-the-time-axis-instantiates-it.md) established the pattern for recorded intent and refused a dedicated heuristic beat-grid schema key because beats did not unlock a `validate` check at the time, leaving timeline intent in author-generated sidecars (`words.json`, `beats.json`).
[ADR-0145](0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md) established the four core invariants: literal values, closed vocabulary, checkable by `validate`, and exact-string replace.
[ADR-0202](0202-the-output-field-accepts-a-path-string-or-a-structured-output-config-with-container-inference.md) established `OutputConfig` with container inference (`mp4`, `mov`, `mxf`).
[ADR-0210](0210-opentimelineio-translates-bi-directionally-via-rational-time-and-sequential-tracks.md) §4 established OpenTimelineIO bidirectional translation, specifying that OTIO `Marker` objects on clips or timeline tracks map to literal metadata or chapter intervals.
[ADR-0215](0215-after-effects-is-precedent-for-rendered-results-and-capcut-and-premiere-stay-first-for-shape.md) admitted After Effects precedent for rendered results while keeping Premiere and CapCut first for mechanism and shape.

The After Effects censuses ([#898](https://github.com/MBehtemam/Montagent/issues/898) rows 15–19) identified that After Effects, Adobe Premiere Pro, and CapCut all provide timeline markers and clip markers to annotate beats, cue points, synchronization anchors, editorial notes, and chapter boundaries. While Montagent projects previously had no format-level representation for markers, NLE interchanges (FCPXML, EDL, OTIO) and agent-driven workflows routinely require placing and reading cue points directly in the project structure.

This ADR decides:
1. **`markers` enters schema on the root `project` and on elements as render-inert annotations**: An optional `markers` array enters `project` and element definitions with a closed vocabulary (`at`, optional `duration`, optional `label`, optional `kind`).
2. **Markers are strictly render-inert**: Markers do not alter a single pixel or audio sample during rendering, preserving byte-identical deterministic rendering.
3. **`validate` enforces structural validity and bounds only**: `validate` checks that `at >= 0`, `duration >= 0`, and `kind` matches the closed vocabulary. Fuzzy heuristics (e.g. checking whether cuts or keyframes align within a tolerance of markers) are refused from `validate`.
4. **`shift` updates marker timestamps across timeline edits**: Downstream project markers shift alongside elements when ripple operations run.
5. **Project-level `kind: "chapter"` markers export to MP4/MOV container chapter metadata**: `render` writes standard container chapter tracks for delivery, while single-frame inspection tools (`frame`, `preview`) ignore markers.

---

## Decisions

### 1. `markers` enters schema on `project` and elements

`markers` enters as an optional array on the root `project` object (timeline markers) and on all element variants (`video`, `audio`, `image`, `text`, `rect`, `ellipse`, `path`, `nest`) (element markers).

The schema is flat, literal JSON:

```json
{
  "at": 1250,
  "duration": 500,
  "label": "Chorus",
  "kind": "chapter"
}
```

- `at`: required `u64` timestamp in milliseconds. For project markers, `at` is relative to the project timeline ($0 \le \text{at} \le \text{duration}$). For element markers, `at` is relative to the element's local media or start instant.
- `duration`: optional `u64` in milliseconds representing marker span (defaulting to `0`, indicating an instant/point marker).
- `label`: optional `string` describing the marker (e.g. note, chapter title, or speech cue).
- `kind`: optional closed enum string:
  - `"comment"` (default): general editorial comment or note.
  - `"chapter"`: deliverable chapter boundary or section marker.
  - `"beat"`: musical beat or rhythmic transient marker.
  - `"cue"`: sound effect, graphic hit, or sync anchor.

### 2. Markers are strictly render-inert

Markers do not affect pixel generation or audio mixing:
- `frame`, `preview`, and `render` rasterize visual frames and mix audio stems completely independent of `markers`.
- Markers never inject overlays, subtitles, or visual indicators into the video stream.
- This preserves Montagent's byte-identical rendering invariant across all engines and platforms.

### 3. `validate` enforces structural rules, refusing heuristic tolerance checks

In accordance with [ADR-0145](0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md), `validate` checks must be deterministic, non-heuristic, and either pass or fail based on schema invariants:

- `validate` verifies:
  - `at >= 0` and `duration >= 0`.
  - For project markers: $\text{at} \le \text{project.duration}$.
  - `kind` is one of the four allowed strings (`"comment"`, `"chapter"`, `"beat"`, `"cue"`).
- `validate` does **not** check:
  - Whether cut points or transition endpoints land within $\pm \delta$ milliseconds of a `"beat"` marker.
  - Whether captions or titles align with speech cue markers.
  
Fuzzy synchronization checks belong in higher-level agent inspection scripts (`skills/montagent-motion/`) or external linters, never in the authoritative core validator.

### 4. `shift` arithmetic updates markers

`shift` operates on project and element time axes:
- When a `shift` ripple operation modifies the timeline at timestamp $T_{\text{split}}$ by delta $\Delta t$, all project-level markers with $\text{at} \ge T_{\text{split}}$ have their `at` timestamp incremented by $\Delta t$.
- Element-level markers remain anchored to their element; if an element shifts or trims, its internal markers preserve their element-relative offsets.

### 5. `render` writes MP4/MOV container chapter metadata

When project-level markers include entries with `kind: "chapter"`, `render` maps them into standard container metadata during container multiplexing:
- For MP4 containers: written as QuickTime chapter tracks or Nero chapter user-data atoms (`chpl` / standard MP4 chapter list).
- For MOV containers: written as standard QuickTime chapter tracks.
- For MKV/MXF containers: written as container chapter entries.
- Element-level markers, and project markers with `kind: "comment"`, `"beat"`, or `"cue"`, are not written to the container chapter track, keeping delivery files clean and player-compatible.

---

## Consequences

- Resolves census rows 15–19 of [#898](https://github.com/MBehtemam/Montagent/issues/898).
- `crates/montagent-core/src/model/project.rs` and element models gain an optional `markers: Option<Vec<Marker>>`.
- OpenTimelineIO import and export ([ADR-0210](0210-opentimelineio-translates-bi-directionally-via-rational-time-and-sequential-tracks.md)) can map OTIO `Marker` objects directly to Montagent `Marker` instances with zero loss of timeline annotation data.
- Video players (QuickTime Player, VLC, YouTube upload processing) display native chapter navigation markers generated from project chapter markers.
