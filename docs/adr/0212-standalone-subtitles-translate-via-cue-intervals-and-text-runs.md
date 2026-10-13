---
status: accepted
amends: 0054 (captions), 0145 (four invariants), 0209 (interchange architecture)
---

# Standalone subtitles translate via cue intervals and text runs

[#978](https://github.com/MBehtemam/Montagent/issues/978), [#1038](https://github.com/MBehtemam/Montagent/issues/1038), [#1042](https://github.com/MBehtemam/Montagent/issues/1042).

## Context

Subtitles and closed captions are essential deliverables in modern video editing (`docs/research/premiere-gap/captions.md`). Premiere Pro and other NLEs support importing, editing, and exporting standalone subtitle files:
- **SubRip (`.srt`)**: Numbered blocks with comma-delimited millisecond timestamps (`HH:MM:SS,mmm --> HH:MM:SS,mmm`).
- **WebVTT (`.vtt`)**: W3C standard with `WEBVTT` header, dot-delimited millisecond timestamps (`HH:MM:SS.mmm --> HH:MM:SS.mmm`), and optional cue placement settings.

In Montagent, captions are represented declaratively as `type: "text"` elements carrying text `runs: [...]` on timeline tracks ([ADR-0054](0054-silence-detection-is-external-validate-and-compare-catch-drift.md)). 

Montagent requires a bidirectional translator between standalone `.srt` / `.vtt` files and declarative caption tracks without requiring external rendering or font shaping ([ADR-0209](0209-timeline-interchange-is-a-decoupled-zero-renderer-conversion-engine.md)).

## Decision

1. **Timestamp Parsing & Formatting**:  
   - SubRip (`.srt`): Parses `HH:MM:SS,mmm` into integer milliseconds:
     $$\text{ms} = (H \times 3600 + M \times 60 + S) \times 1000 + mmm$$
   - WebVTT (`.vtt`): Parses both `HH:MM:SS.mmm` and `MM:SS.mmm` formats into integer milliseconds.
   - Formatting formats integer milliseconds back to standard strings with leading zeros.

2. **Subtitle Ingestion into Caption Tracks**:  
   When importing an `.srt` or `.vtt` file:
   - A dedicated track is created: `{"name": "subtitles", "layer": <top_layer>, "elements": [...]}`.
   - Each subtitle cue becomes a `type: "text"` element:
     ```json
     {
       "id": "sub_<n>",
       "type": "text",
       "start": <cue_start_ms>,
       "end": <cue_end_ms>,
       "x": 0,
       "y": <bottom_third_y>,
       "width": <project_width>,
       "height": 200,
       "align": "center",
       "font": "default",
       "size": 48,
       "runs": [
         { "text": "<cue_text>" }
       ]
     }
     ```
   - Basic HTML tags (e.g. `<b>`, `<i>`, `<font>`) in cues are stripped or normalized to plain text runs.

3. **Subtitle Extraction & Export**:  
   When exporting a Montagent project to `.srt` or `.vtt`:
   - Elements on tracks named `"subtitles"`, `"captions"`, or elements of `type: "text"` sorted by `start` time are collected.
   - Adjacent or overlapping text runs are joined.
   - Serialized into sequential, numbered cues formatted with canonical `.srt` or `.vtt` headers and timestamps.

4. **Integration with External Analysis Protocol**:  
   This translator integrates directly with external speech-to-text models (e.g. Whisper CLI) via the External Analysis Protocol ([#982](https://github.com/MBehtemam/Montagent/issues/982)): external transcription tools output `.srt` or `.vtt`, which Montagent imports into literal caption tracks.
