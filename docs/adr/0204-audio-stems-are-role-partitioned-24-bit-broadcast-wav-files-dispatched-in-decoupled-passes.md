---
status: accepted
amends: 0011 (deliverables), 0172 (master stage), 0184 (audio routing)
---

# Audio stems are role-partitioned 24-bit broadcast WAV files dispatched in decoupled passes

[#983](https://github.com/MBehtemam/Montagent/issues/983), [#1002](https://github.com/MBehtemam/Montagent/issues/1002), [#1005](https://github.com/MBehtemam/Montagent/issues/1005).

## Context

Broadcasters, streaming distributors, and international localization workflows require separate audio stem deliverables: Dialogue (DX), Music (MX), and Sound Effects (FX / Foley). Montagent currently sums all audio elements into a single stereo `[mix]` bus, rendering a single interleaved AAC or PCM soundtrack alongside video ([ADR-0077](0077-the-nine-render-readings-are-ratified.md), [ADR-0184](0184-pan-is-a-linear-balance-and-channels-is-a-five-word-routing-field-both-flat-on-audio-and-video.md)).

Generating separate audio stems by manually creating multiple duplicated projects is error-prone, unmaintainable, and breaks atomic versioning.

## Decision

1. **Semantic Audio `role` Vocabulary**:  
   Audio elements (`audio`, `video`) and `tracks` admit an optional `role` property belonging to a closed vocabulary:
   - `"dialogue"`: Spoken speech, character dialogue, interview audio.
   - `"voiceover"`: Narration, off-screen commentary, host voice.
   - `"music"`: Background music, score, source music.
   - `"sfx"`: Sound effects, ambient environment beds, impacts.
   - `"foley"`: Synchronous human Foley movements, footsteps, props.
   
   *Inheritance*: A track-level `role` establishes the default semantic role for all elements on that track. An element-level `role` explicitly overrides the track default.

2. **Declarative Stem Deliverables (`output.stems`)**:  
   Projects specify stem exports within `output.stems`:
   ```json
   {
     "output": {
       "path": "renders/master.mov",
       "stems": [
         { "role": "dialogue", "path": "stems/dialogue.wav" },
         { "role": "voiceover", "path": "stems/voiceover.wav" },
         { "role": "music", "path": "stems/music.wav" },
         { "role": "sfx", "path": "stems/sfx.wav" }
       ]
     }
   }
   ```
   Each stem item requires:
   - `role`: The semantic role to isolate.
   - `path`: Relative filesystem destination, terminating in `.wav`.

3. **Decoupled Audio Export Passes**:  
   Stem generation is decoupled from visual video rendering:
   - **Full Render (`montagent render`)**: First completes the composite master deliverable. Subsequently, dispatches audio-only passes (`-vn`) for each configured stem.
   - **Stems-Only Flag (`montagent render --stems-only`)**: Bypasses the Skia canvas rasterizer entirely, synthesizing only the requested audio stems in parallel in seconds.
   - **Broadcast WAV Format**: Stems are encoded as 24-bit uncompressed linear PCM at 48 kHz stereo (`-c:a pcm_s24le -ar 48000`) in the Broadcast Wave Format (BWF/RIFF).
   - **Master Effects Isolation**: Project master loudness targets ([ADR-0172](0172-the-master-stage-is-a-top-level-loudness-target-and-true-peak-ceiling-reached-by-one-measured-gain.md)) are applied to the full mix, not individual stems, preserving mix headroom for downstream dubbing re-recording.

4. **Static Validation Invariants (`montagent validate`)**:  
   - **`E-STEM-PATH-COLLISION`**: Stem destination paths must be distinct from the primary deliverable and from all other stem paths.
   - **`E-STEM-FORMAT-INVALID`**: Stem files must carry the `.wav` file extension.
   - **`E-STEM-ROLE-EMPTY`**: If a project declares a stem for a role that contains zero active elements, `validate` emits an advisory finding.

## Consequences

- Professional delivery packages (master video + isolated audio stems) are declared in a single atomic project file.
- Audio post-production and localization workflows can re-render dialogue or music stems instantly without re-rasterizing video frames.
- Strict closed semantic roles prevent arbitrary, non-standard track tags.
