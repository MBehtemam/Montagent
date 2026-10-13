---
status: accepted
amends: 0011 (deliverables), 0077 (render readings), 0143 (H.264 default parameters)
---

# The output field accepts a path string or a structured output config with container inference

[#983](https://github.com/MBehtemam/Montagent/issues/983), [#1002](https://github.com/MBehtemam/Montagent/issues/1002), [#1003](https://github.com/MBehtemam/Montagent/issues/1003).

## Context

Montagent currently defines project output as a bare filesystem string in `schema/montagent.schema.json`:
```json
"output": "out/final.mp4"
```
Every render assumes this deliverable is an 8-bit `yuv420p` H.264 video with AAC stereo audio in an MP4 container ([ADR-0077](0077-the-nine-render-readings-are-ratified.md), [ADR-0143](0143-render-keeps-libx264-medium-crf-20-pins-five-encoder-threads-and-says-so.md)). While optimal for web playback and lightweight distribution, professional post-production workflows require mastering codecs (Apple ProRes, Avid DNxHR), alpha channel transparency, and auxiliary deliverables such as audio stems.

We must expand the deliverable vocabulary without breaking existing project documents, test fixtures, or CLI invocations.

## Decision

1. **Backward-Compatible Untagged Union**:  
   The `output` property in `schema/montagent.schema.json` is expanded via a schema `oneOf` union:
   - A shorthand **string** path: `"output": "out/final.mp4"` (preserves 100% backward compatibility).
   - A structured **`OutputConfig`** object declaring explicit video, audio, container, and auxiliary stem settings.

2. **Structure of `OutputConfig`**:  
   ```json
   {
     "output": {
       "path": "renders/master.mov",
       "container": "mov",
       "video": {
         "codec": "prores",
         "profile": "4444",
         "alpha": true
       },
       "audio": {
         "codec": "pcm",
         "sample_rate": 48000
       },
       "stems": [
         { "role": "dialogue", "path": "stems/dialogue.wav" },
         { "role": "music", "path": "stems/music.wav" },
         { "role": "sfx", "path": "stems/sfx.wav" }
       ]
     }
   }
   ```
   - `path`: Required string. Deliverable destination relative to the project file.
   - `container`: Optional enum (`"mp4"`, `"mov"`, `"mxf"`). If omitted, inferred automatically from `path`'s file extension (`.mp4` $\rightarrow$ `"mp4"`, `.mov` $\rightarrow$ `"mov"`, `.mxf` $\rightarrow$ `"mxf"`).
   - `video`: Optional object defining video encoding parameters (`codec`, `profile`, `crf`, `alpha`).
   - `audio`: Optional object defining audio encoding parameters (`codec`, `sample_rate`, `bitrate`).
   - `stems`: Optional array of auxiliary audio stem deliverables.

3. **Semantic Defaults**:  
   When `output` is stated as a bare string `"out.mp4"`, or when `OutputConfig` omits sub-fields, Montagent defaults to the established H.264 baseline:
   - Video: `codec: "h264"`, `crf: 20`, `preset: "medium"`, `threads: 5`, `alpha: false`.
   - Audio: `codec: "aac"`, `bitrate: "160k"`, `sample_rate: 48000`.
   - Container: `"mp4"`.

4. **Static Validation Invariants (`montagent validate`)**:  
   - **`E-OUTPUT-EXTENSION-MISMATCH`**: If both `container` and `path` are specified and contradict each other (e.g. `container: "mov"` with `path: "video.mp4"`), validate refuses with an explicit error.
   - **`E-OUTPUT-UNKNOWN-KEY`**: Statically enforced closed vocabulary on all `OutputConfig`, `VideoExportConfig`, and `AudioExportConfig` fields.
   - **`E-OUTPUT-EMPTY`**: Output path must be a non-empty relative path ending with a recognized extension.

## Consequences

- Existing projects continue to validate and render without any modification.
- Authors and AI agents can target broadcast and editing intermediaries with explicit declarative configuration.
- Primary and auxiliary deliverables are stated in a single atomic project document.
