# Protocol for External Analysis Tools Emitting Montagent Project Edits

- **Status:** Research Specification / Protocol Definition
- **Related Issue:** [#982](https://github.com/MBehtemam/Montagent/issues/982) (External Analysis Protocol)
- **Context & Precedents:**
  - `CONTEXT.md` (**Montagent** model-free determinism, **Element**, **Transform**, **Keyframe**, **Pre-rendered footage**)
  - [ADR-0011](../../adr/0011-tool-surface-reads-checks-renders.md) (Tool surface reads, checks, renders; write-tool invariant)
  - [ADR-0033](../../adr/0033-split-element-operation.md) (Split element operation and cut continuity)
  - [ADR-0038](../../adr/0038-ease-is-required-on-every-non-first-keyframe-record.md) (Ease required on every non-first keyframe record)
  - [ADR-0048](../../adr/0048-per-word-highlighting-is-a-timed-window-on-the-run.md) (Per-word highlighting is a timed window on the run)
  - [ADR-0051](../../adr/0051-word-alignment-is-external-validate-and-compare-catch-drift.md) (Word alignment is external; validate and compare catch drift)
  - [ADR-0054](../../adr/0054-caption-audio-backing-and-minimum-duration-checks.md) (Caption audio backing and minimum duration checks; classification model exclusion)
  - [ADR-0082](../../adr/0082-keyframe-time-monotonicity.md) (Keyframe time strictly monotonically increasing)
  - [ADR-0086](../../adr/0086-recorded-intent-source-clock-derivation.md) (Recorded intent: source clock derivation)
  - [ADR-0145](../../adr/0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md) (Four invariants of Montagent capabilities)
  - [ADR-0156](../../adr/0156-four-named-effects-join-the-effects-list-and-a-code-drawn-piece-enters-as-pre-rendered-footage.md) (Pre-rendered footage and sidecar recipes)
  - [ADR-0177](../../adr/0177-ducking-is-a-skill-script-that-writes-ordinary-volume-keyframes-and-the-format-stays-untouched.md) (Ducking as an external skill script writing ordinary keyframes)
  - Gap analyses: [`docs/research/premiere-gap/ai.md`](../premiere-gap/ai.md), [`captions.md`](../premiere-gap/captions.md), [`motion.md`](../premiere-gap/motion.md)

---

## 1. Executive Summary & Problem Statement

Montagent is architecturally and constitutionally **model-free and deterministic**:
> *"Montagent is deterministic and contains no model: given the same project and the same files it produces the same video every time."* (`CONTEXT.md`)

In commercial NLEs like Adobe Premiere Pro and CapCut, advanced media intelligence features—such as **Scene Edit Detection**, **Auto Reframe / Motion Tracking**, **Speech-to-Text / Captions**, and **Auto Ducking**—are built into the GUI or background services, often relying on proprietary neural networks (Adobe Sensei, Firefly), local ONNX models, or heuristics.

Montagent firmly excludes neural network models, forced aligners, diarizers, and non-deterministic signal classifiers from its core binary (`crates/montagent-core`, `crates/montagent`). Introducing a model into the core binary would destroy bit-for-bit rendering reproducibility, inflate binary footprint, introduce hardware and runtime nondeterminism, and couple editing documents to fleeting model weights.

Consequently, all analysis and perception tools (Whisper, PySceneDetect, OpenCV/MediaPipe trackers, ffmpeg detectors) must execute **strictly external** to Montagent. 

This document defines the **Montagent External Analysis Tool Contract**:
1. How external CLI tools and agent skills must structure, normalize, and emit their findings.
2. The exact JSON schema structures receiving the output:
   - **Scene Edit Detection** $\rightarrow$ Split video/audio elements with contiguous source slicing.
   - **Motion Tracking / Auto Reframe** $\rightarrow$ Literal keyframe arrays on `x`, `y`, `scale`.
   - **Speech-to-Text / Word Alignment** $\rightarrow$ Paged `text` caption elements with timed `runs` and `highlight` windows.
   - **Audio Analysis / Ducking** $\rightarrow$ Literal `volume` keyframe ramps in linear amplitudes.
3. How `montagent validate` guarantees the syntactic, temporal, and structural validity of these edits without invoking or requiring the original analysis model.
4. The orchestration lifecycle and staleness/drift detection protocols across agent skills.

---

## 2. The Analysis Tool Contract: Four Constitutional Principles

All external analysis tools and agent skills interfacing with Montagent must adhere to the four invariants defined in [ADR-0145](../../adr/0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md) and established in [ADR-0051](../../adr/0051-word-alignment-is-external-validate-and-compare-catch-drift.md), [ADR-0156](../../adr/0156-four-named-effects-join-the-effects-list-and-a-code-drawn-piece-enters-as-pre-rendered-footage.md), and [ADR-0177](../../adr/0177-ducking-is-a-skill-script-that-writes-ordinary-volume-keyframes-and-the-format-stays-untouched.md).

```
+-------------------------------------------------------------------------------+
|                       EXTERNAL AGENT SKILL / CLI TOOL                         |
|  (Whisper ASR, PySceneDetect, OpenCV Trackers, ffmpeg silencedetect, etc.)    |
+---------------------------------------+---------------------------------------+
                                        |
                                        v
                 [Intermediate Interchange JSON (Source Time)]
                   e.g., [{word, start, end}], [cuts], [tracks]
                                        |
                                        v
                 [Skill Mapping Script (e.g., captions.py, duck.py)]
                   - Reads project via `query` / `measure`
                   - Maps source times to timeline clock (start, speed, trim)
                   - Applies layout & creates literal schema structures
                                        |
                 +----------------------+----------------------+
                 |                                             |
                 v (Edits)                                     v (Pre-rendered Footage)
       [Project JSON Document]                        [Lossless MOV / Alpha]
       - Split Elements                               + [<clip>.recipe/ Sidecar]
       - Literal Keyframe Arrays                      - Pinned versions & seed
       - Text Elements + Highlight Runs               - Decoded frame SHA-256
                 |                                             |
                 +----------------------+----------------------+
                                        |
                                        v
                      +-----------------------------------+
                      |   `montagent validate` (Binary)   |
                      |   - Static fact verification      |
                      |   - Zero model dependencies       |
                      |   - Zero audio/video I/O checks   |
                      +-----------------------------------+
```

### Principle 1: Literal Values (No Live Relational References)
Analysis findings must be translated into explicit, literal schema values before being written to the project document:
- All timeline and source coordinates are expressed as absolute integer milliseconds (`int64`).
- All spatial positions are integer pixels (`int64`).
- All scales, rotations, and audio volumes are literal floating-point numbers (`f64`).
- No project document may ever contain a reference to an external model file, an alignment checkpoint, a live relational ducking link (e.g. `duck_under: "dialogue"`), or an unbaked tracking constraint (ADR-0055, ADR-0177).

### Principle 2: Closed Vocabulary Conformance
An analysis tool cannot invent new properties, flags, or metadata in the Montagent schema:
- Tracking cannot add a `"tracked": true` field or a `"tracker_model": "yolo"` attribute.
- Speech-to-text cannot add a `"transcript"` block or `"speaker_label"` to the track (ADR-0054).
- The output must project entirely into the 12 existing element types and the closed animatable transform/effect properties defined in `schema/montagent.schema.json`.

### Principle 3: Checkable by `validate`
The validity of the resulting project must be strictly verifiable by `montagent validate` using pure static analysis. The core engine never reads source audio or executes model inference during validation; it enforces invariants (temporal bounds, non-overlap, keyframe monotonicity, audio backing) through zero-I/O document inspection.

### Principle 4: The Residue vs. Recipe Separation
Montagent strictly distinguishes between the **residue** (what enters the project) and the **recipe** (how it was produced):
- **Residue:** The project file records only the final state—ordinary elements, split source intervals, ordinary keyframes.
- **Recipe:** Sidecar files (e.g., `<asset>.recipe/recipe.json`, `.words.json`, `.spec.json`) stored outside the project record the tool version, command invocations, hashes, prompts, and raw analysis data (ADR-0037, ADR-0086, ADR-0156 §7, ADR-0177 §5).
- If the recipe is deleted, the project remains 100% valid, editable, and renderable.

---

## 3. Intermediate Representation: Standardized Analysis Artifacts

External CLI tools and skills should not write directly to `project.montagent.json` in their raw analysis loops. Instead, they must emit findings into standardized intermediate JSON formats. The skill script then ingests these artifacts, projects them onto the timeline, and updates the project file.

### 3.1. Scene Edit Detection (`.cuts.json`)
Emitted by tools such as PySceneDetect, OpenCV content detectors, or ffmpeg `select='gt(scene,0.4)'`.
- **Clock:** Milliseconds in the source media's own internal presentation time.
- **Format:**
```json
{
  "source": "media/raw_footage.mp4",
  "fps": 29.97,
  "duration_ms": 125400,
  "cuts": [
    0,
    3420,
    8950,
    14200,
    22850
  ]
}
```
*Rule:* The list must include boundary cuts or strictly internal cut points. Timestamps must be sorted in ascending order.

### 3.2. Speech-to-Text & Word Alignment (`.words.json`)
Emitted by Whisper, Kaldi, Wav2Vec2, Gentle, or Montreal Forced Aligner (MFA), conforming to the convention established in ADR-0051:
- **Clock:** Integer milliseconds relative to the start of the voice source file (`t = 0`).
- **Format:**
```json
[
  {"word": "Welcome", "start": 350, "end": 780},
  {"word": "to", "start": 790, "end": 920},
  {"word": "Montagent,", "start": 950, "end": 1400},
  {"word": "deterministic", "start": 1620, "end": 2300},
  {"word": "video", "start": 2320, "end": 2680},
  {"word": "editing.", "start": 2700, "end": 3150}
]
```
*Rule:* Non-speech tokens, punctuation, and silence intervals are handled per ADR-0051: words must be ordered chronologically; gaps between words represent pauses.

### 3.3. Motion Tracking & Bounding Boxes (`.tracking.json`)
Emitted by point trackers, CSRT, KCF, ByteTrack, or MediaPipe pose/face detection models:
- **Format:** Normalized or source-pixel bounding boxes sampled over time.
```json
{
  "source": "media/interview.mp4",
  "source_width": 3840,
  "source_height": 2160,
  "subject": "face_primary",
  "samples": [
    {"t": 0, "bbox": [1820, 450, 420, 520], "confidence": 0.98},
    {"t": 100, "bbox": [1825, 452, 420, 520], "confidence": 0.97},
    {"t": 200, "bbox": [1838, 456, 422, 522], "confidence": 0.99}
  ]
}
```

### 3.4. Speech / Acoustic Span Detection (`.spans.json`)
Used for silence detection, VAD, or ducking triggers without full speech recognition (ADR-0177 §2):
- **Format:**
```json
[
  {"start": 1200, "end": 4500},
  {"start": 5800, "end": 9200}
]
```

---

## 4. Exact Target Schema Mappings & Edits

An external tool or skill script consumes the intermediate JSON and applies edits to the Montagent project document. Below are the precise schema requirements and transformations for the four primary analysis workflows.

---

### 4.1. Scene Edit Detection $\rightarrow$ Split Elements & Project Cuts

When Scene Edit Detection detects shot transitions inside a long source file, the skill splits the continuous element into multiple distinct sibling elements.

#### Architectural Requirements
1. **Source Continuity Invariant (ADR-0033):** Slicing an element $[T_{start}, T_{end})$ at cut timestamp $T_{cut}$ produces two contiguous elements:
   - Head: $[T_{start}, T_{cut})$ with source range $[S_{start}, S_{start} + (T_{cut} - T_{start}) \cdot \text{speed})$.
   - Tail: $[T_{cut}, T_{end})$ with source range $[S_{start} + (T_{cut} - T_{start}) \cdot \text{speed}, S_{end})$.
2. **Audio/Video Synchrony:** If the source footage contains both picture and sound, both the `video` and matching `audio` tracks must be split at the exact same timeline instant.
3. **Element IDs:** Sibling segments must receive unique IDs, following the convention `<original_id>-01`, `<original_id>-02`, or semantic scene tags.

#### Schema Example: Before and After Scene Detection

**Before (Single 10-second clip on timeline):**
```json
{
  "id": "clip-raw",
  "type": "video",
  "start": 1000,
  "end": 11000,
  "source": "media/interview.mp4",
  "source_start": 0,
  "source_end": 10000,
  "width": 1920,
  "height": 1080
}
```

**After (External tool detects cut at source time 4200 ms):**
Timeline cut position: $T_{cut} = 1000 + 4200 = 5200\text{ ms}$.

```json
[
  {
    "id": "clip-raw-01",
    "type": "video",
    "start": 1000,
    "end": 5200,
    "source": "media/interview.mp4",
    "source_start": 0,
    "source_end": 4200,
    "width": 1920,
    "height": 1080
  },
  {
    "id": "clip-raw-02",
    "type": "video",
    "start": 5200,
    "end": 11000,
    "source": "media/interview.mp4",
    "source_start": 4200,
    "source_end": 10000,
    "width": 1920,
    "height": 1080
  }
]
```

---

### 4.2. Motion Tracking / Auto Reframe $\rightarrow$ Transform Keyframe Arrays

Auto Reframe and subject tracking adjust the camera framing (e.g. keeping a speaker centered when converting 16:9 landscape footage into a 9:16 vertical 1080×1920 video).

#### Schema Requirements
1. **Target Properties:** `x`, `y` (`Animatableint64`) and `scale` (`AnimatableArray_size_2_of_double`).
2. **First Keyframe Record (ADR-0038, `FirstKeyframeint64`):**
   - Must contain `"t"` and `"v"`.
   - **Must NOT contain `"ease"`**. The presence of `"ease"` on the first record is a hard schema violation.
3. **Subsequent Keyframe Records (ADR-0038, `Keyframeint64`):**
   - Must contain `"t"`, `"v"`, and **must contain `"ease"`** (e.g., `"linear"`, `"ease-in-out"`, or a 4-component cubic bezier `[x1, y1, x2, y2]`).
4. **Strict Monotonicity (ADR-0082):**
   - $t_{i+1} > t_i$ for all $i$. Equal or descending timestamps violate schema.
5. **Decimation and Smoothing (Avoiding `R-EASE-INERT`):**
   - Raw frame-by-frame tracker output (e.g. 60 samples per second) must not be dumped raw into the project. It should be filtered (moving average or Kalman filter) and decimated to salient change points.
   - Per ADR-0052 and ADR-0177 §5, consecutive identical values (`v_1 == v_2`) with an easing function trigger review finding `R-EASE-INERT`. Flat holds between motion pans should only place keyframes at the start and end of the hold.

#### Schema Example: Auto Reframe Tracking Keyframes

Placing a 3840×2160 video inside a 1080×1920 project canvas, tracking a walking speaker across the frame:

```json
{
  "id": "speaker-reframed",
  "type": "video",
  "start": 0,
  "end": 6000,
  "source": "media/walk.mp4",
  "source_start": 0,
  "source_end": 6000,
  "width": 3413,
  "height": 1920,
  "fit": "cover",
  "origin": "center",
  "x": [
    {"t": 0, "v": 540},
    {"t": 1200, "v": 620, "ease": [0.33, 0, 0.67, 1]},
    {"t": 2800, "v": 410, "ease": [0.33, 0, 0.67, 1]},
    {"t": 4500, "v": 410, "ease": "linear"},
    {"t": 6000, "v": 540, "ease": [0.65, 0, 0.35, 1]}
  ],
  "y": 960,
  "scale": [
    {"t": 0, "v": [1.0, 1.0]},
    {"t": 2800, "v": [1.15, 1.15], "ease": "ease-out"},
    {"t": 6000, "v": [1.0, 1.0], "ease": "ease-in"}
  ]
}
```

---

### 4.3. Speech-to-Text $\rightarrow$ Text Elements with Timed Highlights

Captions generated from ASR models (e.g. Whisper) must be grouped into readable pages and styled with word-by-word highlights following `captions.py` and ADR-0048.

#### Schema Requirements
1. **Caption Recognition (`CONTEXT.md`, ADR-0136):**
   - Any `text` element without `"caption": false` is evaluated as a caption by `montagent validate`.
2. **Highlight Structure (`Highlight`, ADR-0048):**
   - Timed window on individual runs: `{"text": "word", "highlight": {"start": <ms>, "end": <ms>, "color": "#hex"}}`.
   - Each highlighted word is its own entry in `runs`.
   - Whitespace and line breaks (`\n`, ` `) are separate unhighlighted runs.
3. **Temporal Containment Invariants:**
   - For every run: $T_{\text{element.start}} \le T_{\text{highlight.start}} < T_{\text{highlight.end}} \le T_{\text{element.end}}$.
   - No two highlight windows may overlap in time within the same element (ADR-0051 §3).
4. **Audio Backing & Duration Checks (ADR-0054):**
   - **`R-CAPTION-NO-AUDIO`:** Every caption element must temporally overlap at least one active `audio` element or video element with sound.
   - **`R-CAPTION-MIN-DURATION`:** Caption element duration must be $\ge 834\text{ ms}$ (25 frames at 30 fps) to guarantee human visual legibility.
5. **Layout & Box Dimensions:**
   - Width and height must be measured against the project's actual font using `montagent measure` before authoring, ensuring text does not overflow the canvas or its background pill.

#### Schema Example: Paged Caption with Spoken Word Highlight

```json
{
  "id": "caption-01",
  "type": "text",
  "start": 1000,
  "end": 3200,
  "x": 540,
  "y": 1400,
  "origin": "center",
  "width": 840,
  "height": 120,
  "font": "Inter-Bold",
  "size": 52,
  "color": "#F5F0E6",
  "align": "center",
  "runs": [
    {
      "text": "Welcome",
      "highlight": {"start": 1050, "end": 1450, "color": "#FF5A36"}
    },
    {"text": " "},
    {
      "text": "to",
      "highlight": {"start": 1460, "end": 1600, "color": "#FF5A36"}
    },
    {"text": " "},
    {
      "text": "Montagent",
      "highlight": {"start": 1620, "end": 2200, "color": "#FF5A36"}
    }
  ]
}
```

---

### 4.4. Audio Signal Analysis $\rightarrow$ Volume Keyframes (Ducking)

Per [ADR-0177](../../adr/0177-ducking-is-a-skill-script-that-writes-ordinary-volume-keyframes-and-the-format-stays-untouched.md), ducking under dialogue is emitted as ordinary linear `volume` keyframes on the background music track.

#### Schema Requirements
1. **Amplitude Mapping:**
   - Values written to the project JSON are linear gains rounded to 4 decimal places:
     $$\text{gain} = \text{round}(10^{(\text{dB} / 20)}, 4)$$
   - Standard defaults:
     - Under voice: $-15\text{ dB} \rightarrow 0.1778$
     - Between phrases (over): $-6\text{ dB} \rightarrow 0.5012$
     - End card / lead-in: $-1.5\text{ dB} \rightarrow 0.8414$
2. **Timing Ramps:**
   - Lead ramp: Starts $100\text{ ms}$ ahead of voice onset (`lead_ms = 100`).
   - Ramp duration: $200\text{ ms}$ (`ramp_ms = 200`).
   - Pause threshold: Only pauses $\ge 600\text{ ms}$ return to over-level (`join_ms = 600`) to avoid pumping.

#### Schema Example: Ducked Music Element
```json
{
  "id": "music-bed",
  "type": "audio",
  "start": 0,
  "end": 20000,
  "source": "audio/bed.mp3",
  "source_start": 0,
  "source_end": 20000,
  "volume": [
    {"t": 0, "v": 0.8414},
    {"t": 900, "v": 0.8414, "ease": "linear"},
    {"t": 1100, "v": 0.1778, "ease": [0.65, 0, 0.35, 1]},
    {"t": 4500, "v": 0.1778, "ease": "linear"},
    {"t": 4700, "v": 0.5012, "ease": [0.65, 0, 0.35, 1]},
    {"t": 5800, "v": 0.5012, "ease": "linear"},
    {"t": 6000, "v": 0.1778, "ease": [0.65, 0, 0.35, 1]}
  ]
}
```

---

## 5. Model-Free Verification: How `validate` Enforces Well-Formedness

When an external tool emits project edits, `montagent validate` inspects the output. Because Montagent contains no neural models or signal analysis runtimes, validation operates through deterministic, static checks.

### 5.1. Static Document Invariants
1. **Type & Bounds Discrimination:**
   - Field sets are strict functions of `type` (ADR-0012). An external tool writing `"highlight"` on a `rect` or `"x"` on an `audio` element fails immediately with a schema violation.
2. **Keyframe Grammar Checks:**
   - The first keyframe of any animatable list must not carry `"ease"` (ADR-0038).
   - Every non-first keyframe must carry `"ease"` (ADR-0038).
   - All keyframe instants $t$ must be strictly ascending: $t_0 < t_1 < \dots < t_n$ (ADR-0082).
3. **Temporal Containment (`Highlight`):**
   - `montagent validate` asserts that every `highlight.start` and `highlight.end` lies within $[element.start, element.end)$ (ADR-0051).
   - It asserts that sibling highlights within a single `text` element do not overlap.
4. **Zero-I/O Relational Document Checks (ADR-0054):**
   - **`R-CAPTION-NO-AUDIO`:** Validates that for every caption element, there exists at least one audio or video element spanning an overlapping interval $[t_a, t_b)$ where $t_a < \text{caption.end}$ and $t_b > \text{caption.start}$.
   - **`R-CAPTION-MIN-DURATION`:** Asserts $\text{end} - \text{start} \ge 834\text{ ms}$.
5. **Spatial and Canvas Integrity:**
   - Validates that text bounding boxes measured with project fonts do not overflow canvas bounds or create unpainted frames (ADR-0134).

### 5.2. Review Findings vs. Hard Errors
- **Hard Schema Errors (Execution Halted):** Missing required fields, out-of-order keyframes, overlapping track elements, highlight outside parent bounds.
- **Review Findings (Warnings Printed, Return Code 0 unless `--strict`):**
  - `R-EASE-INERT`: Triggered when two adjacent keyframes have identical literal values $v$ (e.g. held levels in ducking curves). Skills document this as expected on ducking curves (ADR-0177 §5).
  - `R-TRANSITION-VOLUME-STACK`: Triggered when an audio transition crossfade overlaps volume keyframes.

---

## 6. Pre-Rendered Footage: The External Recipe Contract

For heavy computer-vision operations that cannot be represented as simple affine transforms or closed effects (such as **Warp Stabilizer**, complex AI background removal, or optical flow interpolation), Montagent provides the **Pre-rendered Footage Pattern** ([ADR-0156](../../adr/0156-four-named-effects-join-the-effects-list-and-a-code-drawn-piece-enters-as-pre-rendered-footage.md), [`skills/montagent-prerender`](../../../skills/montagent-prerender/SKILL.md)).

```
+-----------------------------------------------------------------------------------+
|                            EXTERNAL BAKING PIPELINE                               |
|                                                                                   |
|  Source Clip (raw.mp4) ---> [External CV/AI Stabilizer] ---> Lossless MOV (PNG)   |
|                                                                                   |
|  Generates: media/stabilized.mov                                                  |
|  Generates: media/stabilized.recipe/                                              |
|             ├── recipe.json           (tool versions, params, decoded frame hashes)|
|             ├── source/               (copied script and config)                  |
|             └── source_sha256.txt     (source footage hash)                       |
+-----------------------------------------------------------------------------------+
                                        |
                                        v
+-----------------------------------------------------------------------------------+
|                        MONTAGENT PROJECT SPECIFICATION                            |
|                                                                                   |
|  {                                                                                |
|    "type": "video",                                                               |
|    "source": "media/stabilized.mov",      <-- Ordinary element reference          |
|    "start": 0, "end": 5000                                                        |
|  }                                                                                |
|  * The recipe folder is NEVER referenced inside the project JSON.                 |
+-----------------------------------------------------------------------------------+
```

### Pre-Render Contract Rules
1. **Format:** Pre-rendered footage enters as an ordinary `video` element pointing to a lossless ProRes 4444 or PNG-in-MOV file with alpha channel support (ADR-0089).
2. **Recipe Sidecar (`<name>.recipe/`):**
   - Accompanies the generated asset outside the project JSON.
   - Pinned dependencies: Python, OpenCV, ffmpeg versions.
   - Decoded frame SHA-256 hashes (never comparing container bytes, which vary across ffmpeg builds).
3. **Acceptance Thresholds:**
   - Stabilization recipes must enforce measured drift thresholds (e.g. residual drift $< 1\%$ and no pure black perimeter edge pixels, as defined in `skills/montagent-prerender/references/stabilize.md`).

---

## 7. Skill Orchestration Lifecycle & CLI Contract

To integrate external tools seamlessly into Montagent workflows, skills and scripts must implement a standardized three-phase lifecycle:

```
[Phase 1: External Perception]  ==>  [Phase 2: Translation & Layout]  ==>  [Phase 3: Montagent Validation]
(Runs CLI / Model outside)          (Maps spans, keyframes, elements)      (montagent validate & frame)
```

### 7.1. Standard CLI Flags for Skill Scripts
Every skill script that modifies a project from external analysis (e.g., `duck.py`, `captions.py`, `scenedetect.py`) must implement:
1. `--check`: Read-only verification. Computes what the tool would write, compares it with the project's current elements/keyframes, and exits:
   - `0` if the document already matches the analysis.
   - Non-zero naming the first instant and value that differs if stale or unapplied.
2. `--replace`: Overwrite guard. Refuses to overwrite existing keyframe curves or edited elements unless explicitly instructed.
3. Clean Stdio Protocol:
   - `stdout`: Emits the updated project JSON (or diff).
   - `stderr`: Emits human-readable logging, repair summaries, and validation verdicts.
4. Embedded Validation: The script concludes by running `montagent validate` on its own generated output and echoing the engine's findings (ADR-0177 §5).

### 7.2. Drift Detection via `montagent compare`
When transcripts or footage are updated:
- `montagent compare` flags text changed under unchanged timing (ADR-0051).
- Script `--check` flags timing drift when source media is modified or retimed.

---

## 8. Summary Table: Feature Mapping (Premiere / CapCut $\rightarrow$ Montagent)

| Feature | Premiere / CapCut Mechanism | Montagent Protocol | Output Schema Representation | Validation Guarantee |
|---|---|---|---|---|
| **Scene Edit Detection** | Sensei cut detector / subclips | External PySceneDetect / OpenCV script | Contiguous split `video`/`audio` elements with sliced `source_start`/`source_end` | Track non-overlap; source slice continuity (ADR-0033) |
| **Auto Reframe** | Continuous face/action tracking effect | External tracker (CSRT/MediaPipe) $\rightarrow$ Decimator script | Literal keyframe arrays on `x`, `y`, `scale` | Keyframe monotonicity (ADR-0082); Ease syntax (ADR-0038) |
| **Speech-to-Text** | Sensei / Whisper with internal transcript | External Whisper/MFA $\rightarrow$ `captions.py` | `text` elements with timed `runs` and `highlight` windows | `R-CAPTION-NO-AUDIO`, `R-CAPTION-MIN-DURATION`, containment |
| **Auto Ducking** | Live dynamic sidechain compressor | External span parser $\rightarrow$ `duck.py` | Literal linear `volume` keyframes with ramp/lead/join | Float range $[0.0, 1.0]$; monotonic $t$; review `R-EASE-INERT` |
| **Warp Stabilizer** | Subspace warp background effect | External CV bake $\rightarrow$ `prerender.py` | Pre-rendered lossless MOV with sidecar `<clip>.recipe/` | Pixel integrity; decoded frame hash; drift $< 1\%$ |
| **Text-Based Editing** | Ripple edit sequence via transcript text | External words file $\rightarrow$ Agent source trim & `shift` | Modified `source_start`/`source_end` and `shift` ripple | Slack invariant; boundary instant pair check (ADR-0047) |

---

## 9. Conclusion

Montagent resolves the tension between modern AI perception and deterministic video rendering by maintaining a strict, verifiable boundary:
- **Models belong to the agent and the environment;**
- **The engine handles only mathematics, layout, and rendering;**
- **The document stores only explicit literals.**

External analysis tools do not require specialized hooks or binary modifications. By formatting findings into normalized intermediate JSON and compiling them into Montagent's closed vocabulary of elements and keyframes, any external analysis capability—from cutting-edge neural models to classic signal algorithms—can participate in the Montagent workflow while maintaining bit-for-bit reproducible output.
