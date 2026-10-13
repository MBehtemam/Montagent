# Fractional and NTSC Timebases in Montagent: Representation, Arithmetic, and Parity

**Status:** Research Complete (Resolves [#980](https://github.com/MBehtemam/Montagent/issues/980), part of [#978](https://github.com/MBehtemam/Montagent/issues/978))  
**Date:** October 2026  
**Primary Sources:**
- `schema/montagent.schema.json`
- Architecture Decision Records: [ADR-0005](../../adr/0005-absolute-integer-milliseconds.md), [ADR-0006](../../adr/0006-validate-reports-facts-and-render-enforces.md), [ADR-0011](../../adr/0011-tool-surface-reads-checks-renders.md), [ADR-0012](../../adr/0012-flat-transform-keyframes-carried-by-their-element.md), [ADR-0023](../../adr/0023-video-source-dimensions-par-and-rotation.md), [ADR-0028](../../adr/0028-text-block-arithmetic-is-exact-tenths.md), [ADR-0035](../../adr/0035-keyframe-grid-alignment-is-a-review-check-not-a-schema-rule.md), [ADR-0041](../../adr/0041-canonical-key-order.md), [ADR-0045](../../adr/0045-speed-invariant-is-evaluated-in-exact-arithmetic.md), [ADR-0077](../../adr/0077-the-nine-render-readings-are-ratified.md), [ADR-0086](../../adr/0086-recorded-intent-is-one-pattern-and-the-time-axis-instantiates-it.md), [ADR-0096](../../adr/0096-the-frame-at-an-instant-is-the-last-one-starting-at-or-before-it.md), [ADR-0143](../../adr/0143-render-keeps-libx264-medium-crf-20-pins-five-encoder-threads-and-says-so.md), [ADR-0145](../../adr/0145-capcut-and-premiere-are-where-precedent-is-looked-for-first...md), [ADR-0157](../../adr/0157-time-remap-is-a-source-time-curve-carried-by-video.md)
- Gap Analysis: `docs/research/premiere-gap/editing.md` (Row 77), `docs/research/premiere-gap/export.md` (Rows 78, 156, 223)
- Codebase: `crates/montagent-core/src/exact.rs`, `crates/montagent-core/src/media/mod.rs`, `crates/montagent-core/src/model/mod.rs`, `crates/montagent-render/src/decode.rs`, `crates/montagent-render/src/encode.rs`, `crates/montagent-core/src/verbs/measure.rs`

---

## 1. Executive Summary

Research ticket [#980](https://github.com/MBehtemam/Montagent/issues/980) addresses a foundational gap identified in the Premiere Pro feature audit ([#882](https://github.com/MBehtemam/Montagent/issues/882), `docs/research/premiere-gap/editing.md` Row 77):
> *"Sequence timebase / frame rate: Top-level `fps` is an integer (schema). Frame $n$ is painted at $\lfloor n \times 1000 / \text{fps} \rfloor$ ms (ADR-0077). Missing: Fractional and NTSC rates: 24000/1001, 30000/1001 and 60000/1001 cannot be written."*

In North American broadcast and cinema production, the standard delivery frame rates are not whole integers:
- **23.976 fps** (NTSC film / 24p): exactly $24000 / 1001 = 23.\overline{976023}$ frames per second.
- **29.97 fps** (NTSC television / 30p / 60i): exactly $30000 / 1001 = 29.\overline{970029}$ frames per second.
- **59.94 fps** (NTSC high-frame-rate / 60p): exactly $60000 / 1001 = 59.\overline{940059}$ frames per second.

The core challenge is how to introduce support for these fractional timebases while strictly upholding the four architectural invariants of **ADR-0145** (Literal values, Closed vocabulary, Checkable by `validate`, Exact-string replace), preserving **ADR-0005**'s "reading is free" integer millisecond timeline addresses, avoiding floating-point non-determinism (**ADR-0028**, **ADR-0045**), and providing deterministic SMPTE drop-frame / non-drop-frame timecode mappings.

### Summary of Decisions
1. **Frame Rate Representation in Document**: `fps` in `schema/montagent.schema.json` must be expanded from a strict integer to a tagged union allowing either an integer (e.g. `24`, `25`, `30`, `50`, `60`) or an exact rational object `{"num": 24000, "den": 1001}` (or integer pair `[24000, 1001]`). Decimal floats (`23.976`, `29.97`) are **strictly prohibited** by schema and `validate`.
2. **Timeline Addresses**: Timeline positions (`start`, `end`, `source_start`, `source_end`, keyframe `t`) **must remain absolute integer milliseconds (`ms`)**. Moving timeline addresses to frames, rational fractions, or Premiere ticks ($254,016,000,000/\text{sec}$) is rejected: it destroys human/agent legibility, couples timeline edits to project frame rates, and breaks exact-string replacements.
3. **Discrete Frame Sampling Arithmetic**: Instant-to-frame sampling follows the exact rational arithmetic established by ADR-0035 and ADR-0077:
   $$\text{instant}(n) = \left\lfloor \frac{n \times 1000 \times \text{den}}{\text{num}} \right\rfloor \text{ ms}$$
   $$\text{frame\_at\_or\_before}(t) = \left\lfloor \frac{t \times \text{num}}{1000 \times \text{den}} \right\rfloor$$
   NTSC rates exhibit a fundamental **1001 ms periodicity theorem**: every 1001 ms contains an exact integer number of frames (24 frames at 23.976, 30 frames at 29.97, 60 frames at 59.94). Boundary presence ($[start, end)$) remains mathematically invariant: $\lfloor t \rfloor \ge \text{start} \iff t \ge \text{start}$.
4. **Timecode Display and Querying**: Timecode is a derived query/display projection, never the internal storage format. `measure` and `query` expose SMPTE 12M timecodes (`HH:MM:SS:FF` for Non-Drop Frame, `HH:MM:SS;FF` for Drop Frame). Sequences at 29.97 and 59.94 default to Drop Frame (DF) to match wall-clock time, dropping 2 frame numbers per minute except every 10th minute.

---

## 2. Primary Source Survey in Montagent

### 2.1 The Schema: `schema/montagent.schema.json`
Currently, the root project schema declares:
```json
"fps": {
  "type": "integer",
  "format": "int64"
}
```
All temporal ranges on timeline elements (`Image`, `Video`, `Audio`, `Text`, `Rect`, `Ellipse`, `Path`, `Transition`) are defined as:
```json
"start": { "type": "integer", "format": "int64" },
"end": { "type": "integer", "format": "int64" }
```
Keyframe time positions (`t`) are likewise typed as integer milliseconds.

### 2.2 Architectural Decisions on Time, Math, and ULP Hazards
- **ADR-0005 (Absolute integer milliseconds)**:
  - *Premise*: Reading is free. An agent or human inspecting a document answers "what is on screen at 6.2s" directly by comparison ($6200 \in [\text{start}, \text{end})$), with no arithmetic.
  - *Float seconds rejected*: Two test agents produced `4.400000000000002` and `14.832999999999998` in their own runs, requiring epsilon comparisons and introducing non-deterministic adjacency failures.
  - *Ticks rejected*: Premiere's 254,016,000,000 ticks/sec is exact but illegible (`start: 269257200000`), forfeiting the read-the-file property.
  - *Frames rejected*: Frames couple every number in the document to sequence fps (a 30 $\to$ 24 rebase rewrites every element).
  - *Acknowledged cost*: Milliseconds do not divide evenly into frames at 30 fps (33.333 ms); the renderer must publish a deterministic rounding rule, and `validate` notes unaligned boundaries.
- **ADR-0028 (Exact arithmetic rule & ULP hazards)**:
  - *"Any non-integer field whose value feeds a boundary-sensitive operation (`ceil`/`floor`/`round`) must have its legal values constrained to an exactly-representable domain, and that operation must be evaluated in exact integer or rational arithmetic — never IEEE double."*
  - Floating point calculations diverge on boundary conditions due to Unit in the Last Place (ULP) rounding artifacts.
- **ADR-0045 (Rational precedent)**:
  - Reaffirmed that operations feeding rounding boundaries must evaluate as exact rational/decimal arithmetic.
  - Cites `par: [num, den]` from ADR-0023 as precedent: rational pairs earn their architectural cost when an engineering quantity is non-terminating in decimal.
- **ADR-0035 & ADR-0077 (The Grid Sampling Rule)**:
  - ADR-0035 established that frame grid sampling `floor(t * fps / 1000) * 1000 / fps` must evaluate in exact rational arithmetic. Keyframe `t` remains continuous; `R-KEYFRAME-UNREACHED` fires if an animated endpoint is never painted by any sampled frame.
  - ADR-0077 Reading 1 settled that timeline frame $n$ is painted at whole millisecond:
    $$\text{instant}(n) = \lfloor n \times 1000 / \text{fps} \rfloor \text{ ms}$$
    Flooring is admissible because presence is unaffected: $\lfloor t \rfloor \ge \text{start} \iff t \ge \text{start}$ and $\lfloor t \rfloor < \text{end} \iff t < \text{end}$.
- **ADR-0096 (Instant sampling rule in FFmpeg)**:
  - "The frame at an instant is the last one starting at or before it."
  - FFmpeg seek logic uses a $1\,\mu\text{s}$ slack `lte(t, at + 1 us)` specifically to account for fractional timebase boundary rounding (e.g., 30000/1001 sources).
- **ADR-0145 (The Four Invariants)**:
  - Literal values (no runtime expressions).
  - Closed vocabulary (all keys and enum values declared in schema).
  - Checkable by `validate` (without painting frames).
  - Exact-string replace (predictable substring edits).

---

## 3. Detailed Investigation of the Four Questions

### Question 1: Expressing Fractional/NTSC Frame Rates Without Breaking Literal Values or Introducing Float Non-Determinism

#### The Problem with Float Frame Rates
Colloquial expressions like `23.976 fps`, `29.97 fps`, and `59.94 fps` are decimal approximations of fractional numbers:
$$\frac{24000}{1001} = 23.976023976023...$$
$$\frac{30000}{1001} = 29.970029970029...$$
$$\frac{60000}{1001} = 59.940059940059...$$

In decimal notation, these numbers are **infinitely repeating** with a period of 6 digits. They cannot be represented as finite decimals.

If an author writes `"fps": 23.976`:
1. **Mathematical Error**: $23.976 = \frac{23976}{1000} \ne \frac{24000}{1001}$. Over 1 hour of video (86,400 frames at 24000/1001):
   $$\text{True duration} = 86400 \times \frac{1001}{24000} = 3603.600000\text{ seconds}$$
   $$\text{Approximate duration} = \frac{86400}{23.976} = 3603.603603\text{ seconds}$$
   The sequence drifts by **3.6 milliseconds per hour**, accumulating over time and causing audio/video sync drift.
2. **IEEE-754 ULP Hazards**: In 64-bit IEEE-754 float, `23.976` is represented as `0x4037f9db22d0e560` ($23.975999999999998824...$). Frame duration $1000 / \text{fps}$ becomes `41.70837504170838...`, whereas exact NTSC 24p frame duration is $\frac{1001}{24} = 41.708333333333...$ ms. Division by an IEEE float produces divergent results across architectures, compiler optimization flags, and SIMD registers, violating ADR-0028.

#### Candidate Solutions

| Candidate | Schema Syntax | Invariant Compliance | Pros & Cons | Verdict |
| :--- | :--- | :--- | :--- | :--- |
| **A. Float Number** | `"fps": 23.976` | **Violates** ADR-0028, ADR-0045, ADR-0145 | Inaccurate; drifts 3.6 ms/hr; floating-point non-determinism. | **REJECTED** |
| **B. Closed String Enum** | `"fps": "24000/1001"` or `"fps": "23.976-ntsc"` | Complies with closed vocabulary | Closed set; but mixes types (string vs int) or forces all fps to strings. Inflexible for non-standard broadcast rates. | **REJECTED** |
| **C. Exact Rational Array** | `"fps": [24000, 1001]` | Complies with all ADR-0145 invariants | Follows ADR-0023 `par: [num, den]` precedent. Very compact. Positional (relies on order `[num, den]`). | **ACCEPTED (Alternative)** |
| **D. Exact Rational Object** | `"fps": {"num": 24000, "den": 1001}` | Complies with all ADR-0145 invariants | Self-documenting keys; eliminates transposition errors; matches `frame: {width, height}`. | **RECOMMENDED PRIMARY** |

#### The Recommended Schema Design
To maintain 100% backward compatibility with all committed project fixtures (`"fps": 25`, `"fps": 30`), `fps` in `schema/montagent.schema.json` is defined as a `oneOf` union:
- Either an integer: `{"type": "integer", "minimum": 1}`
- Or an exact rational object:
```json
{
  "fps": {
    "oneOf": [
      {
        "type": "integer",
        "minimum": 1,
        "description": "Whole frame rate in frames per second (e.g. 24, 25, 30, 50, 60)."
      },
      {
        "type": "object",
        "properties": {
          "num": { "type": "integer", "minimum": 1 },
          "den": { "type": "integer", "minimum": 1 }
        },
        "required": ["num", "den"],
        "additionalProperties": false,
        "description": "Exact rational frame rate (e.g. {\"num\": 24000, \"den\": 1001} for 23.976 fps)."
      },
      {
        "type": "array",
        "items": { "type": "integer", "minimum": 1 },
        "minItems": 2,
        "maxItems": 2,
        "description": "Exact rational pair [num, den], matching ADR-0023 PAR precedent."
      }
    ]
  }
}
```

#### Validation Rules in `validate`
1. **Positive and Non-Zero**: `num > 0` and `den > 0`.
2. **Standard Broadcast Divisor**: NTSC rates must have $\text{den} = 1001$ or $\text{den} = 1000$. A non-standard denominator (e.g. `[17, 3]`) triggers a `review`-level finding `R-FPS-NONSTANDARD`, cautioning the author that downstream video encoders and players may reject the sequence timebase.
3. **No Floats**: A float literal in JSON (e.g. `"fps": 23.976`) fails schema validation immediately with an unparseable/schema error (`E-SCHEMA`), pointing the author to the exact rational spelling.

---

### Question 2: Timeline Positions: Integer Milliseconds (`ms`) vs Exact Rational Timebases

Should timeline positions (`start`, `end`, `source_start`, `source_end`, keyframe `t`) remain integer milliseconds or move to rational timebases?

#### The Case for Preserving Absolute Integer Milliseconds
1. **Reading is Free (ADR-0005)**:
   The foundational principle of Montagent is that agents and humans read the timeline without evaluating equations.
   - *With integer milliseconds*: An agent reading `start: 12000, end: 15500` immediately knows the clip starts at 12.0 s and ends at 15.5 s.
   - *With rational timeline positions*: An agent reading `start: [288000, 24], end: [372000, 24]` or `start: {"num": 12012, "den": 1001}` must run fraction reduction and division to answer simple timeline layout questions.
   - *With Premiere ticks ($254,016,000,000/\text{sec}$)*: An agent reading `start: 3048192000000` is completely blind without a calculator.
2. **Decoupling Edits from Sequence Frame Rate**:
   In Premiere Pro, sequences are frame-based. Changing sequence frame rate from 25 fps to 24 fps shifts every cut by fractional frames or changes the edit points. In Montagent, edits are placed in real absolute time. If a video project is re-rendered at 60 fps or 23.976 fps, every audio cue, text title, and transition stays at its exact wall-clock instant.
3. **Exact String Replace (ADR-0145)**:
   Shifting a clip by 250 milliseconds with the `shift` tool is a trivial integer addition (`start: 1000` $\to$ `start: 1250`). With rational timestamps, every shift requires least common multiple arithmetic on denominators, generating multi-digit fractions.
4. **Presence Invariance is Already Solved (ADR-0077)**:
   The fear that fractional frame rates cause boundary flicker or ambiguity on integer millisecond timelines is unfounded.
   ADR-0077 proved Reading 1 for 30 fps ($33.333...$ ms step):
   Because all clip boundaries `start` and `end` are integers, flooring the frame sample instant $\lfloor T_n \rfloor$ preserves boundary presence identically:
   $$\lfloor T_n \rfloor \ge \text{start} \iff T_n \ge \text{start}$$
   $$\lfloor T_n \rfloor < \text{end} \iff T_n < \text{end}$$
   No clip ever flickers in or out of existence, regardless of whether the frame instant is $33.333$ ms or $33.3666...$ ms.

**Conclusion**: Timeline positions **must remain absolute integer milliseconds (`ms`)**.

---

### Question 3: Frame Indexing Arithmetic When Duration Does Not Divide Evenly

How does frame indexing work when duration in ms doesn't divide evenly by frame duration?

#### 3.1 The Exact Forward and Reverse Mappings
Let sequence frame rate be $R = \frac{F_{\text{num}}}{F_{\text{den}}}$. For integer rates, $F_{\text{den}} = 1$.
For 23.976 fps: $F_{\text{num}} = 24000, F_{\text{den}} = 1001$.
For 29.97 fps: $F_{\text{num}} = 30000, F_{\text{den}} = 1001$.
For 59.94 fps: $F_{\text{num}} = 60000, F_{\text{den}} = 1001$.

1. **Exact Mathematical Frame Instant ($T_n$)**:
   The exact moment in time for frame $n$ ($n \ge 0$) is:
   $$T_n = n \times \frac{1000 \times F_{\text{den}}}{F_{\text{num}}} \text{ ms}$$
2. **Painted Whole Millisecond Instant (ADR-0077)**:
   The whole millisecond that the painter paints frame $n$ at is:
   $$\text{instant}(n) = \left\lfloor \frac{n \times 1000 \times F_{\text{den}}}{F_{\text{num}}} \right\rfloor \text{ ms}$$
   In Rust (evaluated in 128-bit integer arithmetic with zero floats):
   ```rust
   pub fn instant_of(n: i64, num: i64, den: i64) -> i64 {
       ((i128::from(n) * 1000 * i128::from(den)) / i128::from(num)) as i64
   }
   ```
3. **Nearest Sampled Instant At-or-Before $t$ (ADR-0035)**:
   Given any timeline instant $t$ in milliseconds:
   $$n = \text{frame\_at\_or\_before}(t) = \left\lfloor \frac{t \times F_{\text{num}}}{1000 \times F_{\text{den}}} \right\rfloor$$
   In Rust:
   ```rust
   pub fn frame_at_or_before(t: i64, num: i64, den: i64) -> i64 {
       ((i128::from(t.max(0)) * i128::from(num)) / (1000 * i128::from(den))) as i64
   }
   ```
4. **First Frame At-or-After $t$**:
   $$n = \text{frame\_at\_or\_after}(t) = \left\lceil \frac{t \times F_{\text{num}}}{1000 \times F_{\text{den}}} \right\rceil = \left\lfloor \frac{t \times F_{\text{num}} + (1000 \times F_{\text{den}} - 1)}{1000 \times F_{\text{den}}} \right\rfloor$$
5. **Total Frame Count for Duration $D$**:
   Under Montagent's half-open timeline interval $[0, D)$, a frame $n$ is rendered if and only if $T_n < D$:
   $$n \times 1000 \times F_{\text{den}} < D \times F_{\text{num}}$$
   Thus, the total number of frames $N$ rendered for a project of duration $D$ ms is:
   $$N = \left\lceil \frac{D \times F_{\text{num}}}{1000 \times F_{\text{den}}} \right\rceil$$

#### 3.2 The 1001 ms Periodicity Theorem
Because the denominator of all NTSC fractional rates is 1001:
$$T_{n + \text{nominal\_fps}} - T_n = \text{nominal\_fps} \times \frac{1000 \times 1001}{\text{nominal\_fps} \times 1000} = 1001 \text{ ms exactly!}$$

- At **24000/1001 fps**: exactly 24 frames span exactly 1001 ms.
- At **30000/1001 fps**: exactly 30 frames span exactly 1001 ms.
- At **60000/1001 fps**: exactly 60 frames span exactly 1001 ms.

Every $1001 \times k$ milliseconds, the timeline synchronizes perfectly to an integer millisecond and an integer frame index simultaneously. There is **zero fractional drift accumulation**.

#### 3.3 Numerical Demonstration: First 10 Frames at 29.97 fps (30000/1001)

| Frame $n$ | Exact $T_n$ (ms) | Painted Instant $\lfloor T_n \rfloor$ (ms) | Frame Step (ms) | Presence in $[0, 100)$? |
| :---: | :---: | :---: | :---: | :---: |
| **0** | $0.0000$ | **0** | - | Yes |
| **1** | $33.3666...$ | **33** | 33 | Yes |
| **2** | $66.7333...$ | **66** | 33 | Yes |
| **3** | $100.1000...$ | **100** | 34 | No (at 100 ms) |
| **4** | $133.4666...$ | **133** | 33 | - |
| **5** | $166.8333...$ | **166** | 33 | - |
| **6** | $200.2000...$ | **200** | 34 | - |
| **7** | $233.5666...$ | **233** | 33 | - |
| **8** | $266.9333...$ | **266** | 33 | - |
| **9** | $300.3000...$ | **300** | 34 | - |
| ... | ... | ... | ... | ... |
| **29** | $967.6333...$ | **967** | 33 | - |
| **30** | $1001.0000$ | **1001** | 34 | - |

Notice: Frame 30 lands at **exactly 1001 ms**. If a project declares `duration: 1000`, exactly 30 frames ($n = 0 \dots 29$) are rendered. If it declares `duration: 1001`, exactly 30 frames ($n = 0 \dots 29$) are rendered because $[0, 1001)$ is half-open! If it declares `duration: 1002`, frame 30 is included, rendering 31 frames.

---

### Question 4: Timecode Display and Querying (SMPTE Non-Drop vs Drop-Frame)

How should timecode display (SMPTE non-drop vs drop-frame timecode) be rendered or queried?

#### 4.1 Broadcast Mechanics: Non-Drop Frame (NDF) vs Drop Frame (DF)
SMPTE ST 12-1 standardizes timecode formatting as `HH:MM:SS:FF` or `HH:MM:SS;FF`.
- **Non-Drop Frame (NDF)**: Separator is colon (`:`). Every second counts from frame `00` to `nominal_fps - 1`.
  - At 29.97 fps NDF, timecode runs at 30 counts per second, but actual video runs at 29.97 frames per second.
  - In 1 hour of real time: video paints 107,892 frames, but NDF timecode counts 108,000 frames. The timecode reads `00:59:56:12` after 1 hour of real clock! It is **108 frames (3.6 seconds) slow**.
- **Drop-Frame (DF)**: Separator is semicolon (`;`).
  - **No video frames are dropped.** Only timecode *display numbers* are dropped from the sequence count.
  - To drop 108 timecode numbers per hour, SMPTE 12M drops the first two frame numbers (`:00` and `:01`) at the start of every minute, **except every tenth minute** (minutes 00, 10, 20, 30, 40, 50).
  - Check: $54 \text{ minutes} \times 2 \text{ dropped frames} = 108 \text{ frames per hour}$.
  - Real clock and timecode clock stay aligned to within 75 ms over 24 hours.

#### 4.2 Standard Timecode Rules by Frame Rate

| Frame Rate | Standard Timecode Mode | Notation Example | Notes |
| :--- | :--- | :--- | :--- |
| **24 fps / 25 fps / 30 fps** | Non-Drop Frame (NDF) | `01:00:00:00` | Integer rates have zero drift against wall clock. |
| **23.976 fps (24000/1001)** | Non-Drop Frame (NDF) | `01:00:00:00` | Film standard. SMPTE ST 12-1 traditionally has no 24p DF. |
| **29.97 fps (30000/1001)** | Drop-Frame (DF) default; NDF optional | `01:00:00;00` (DF) / `01:00:00:00` (NDF) | Broadcast TV default is DF to match broadcast schedule. |
| **59.94 fps (60000/1001)** | Drop-Frame (DF) default; NDF optional | `01:00:00;00` (DF) | Drops 4 frames (`:00`–`:03`) every minute except 10th min. |

#### 4.3 Deterministic Conversion Algorithms
All conversions are implemented in integer arithmetic:

##### Frame Index to 29.97 Drop-Frame Timecode
```rust
pub fn frame_to_smpte_df_2997(frame: i64) -> String {
    // 17,982 frames per 10-minute block (10 * 1800 - 9 * 2 = 17,982)
    let d = frame / 17982;
    let m = frame % 17982;
    let drop_frames = if m > 1 {
        2 * ((m - 2) / 1798)
    } else {
        0
    };
    let adjusted = frame + 18 * d + drop_frames;
    
    let ff = adjusted % 30;
    let ss = (adjusted / 30) % 60;
    let mm = (adjusted / 1800) % 60;
    let hh = adjusted / 108000;
    format!("{:02}:{:02}:{:02};{:02}", hh, mm, ss, ff)
}
```

##### 29.97 Drop-Frame Timecode to Frame Index
```rust
pub fn smpte_df_2997_to_frame(hh: i64, mm: i64, ss: i64, ff: i64) -> i64 {
    let drop_count = 2 * (mm - mm / 10) + 18 * hh;
    let total_unadjusted = ((hh * 60 + mm) * 60 + ss) * 30 + ff;
    total_unadjusted - drop_count
}
```

#### 4.4 Architectural Placement in Montagent
Timecode is strictly an **external projection / query lens**:
1. **Never Stored on Elements**: `start`, `end`, and `t` are always integer `ms`.
2. **`measure --at <ms>` Output**:
   Expands `Instant` in `crates/montagent-core/src/verbs/measure.rs`:
   ```rust
   pub struct Instant {
       pub at: i64,
       pub fps: Rational,
       pub frame: i64,
       pub nearest: f64,
       pub timecode: String, // e.g. "00:00:01;10"
   }
   ```
3. **`query --at` and `--timecode`**:
   `query` accepts `--timecode 00:01:30;00` as an alternative to `--at 90090`. It converts the timecode to exact frame $n$ and queries state at instant $\text{instant}(n)$.
4. **Project Settings**:
   An optional `timecode` object in `Project`:
   ```json
   "timecode": {
     "start": "01:00:00:00",
     "drop_frame": true
   }
   ```
   If omitted, `drop_frame` defaults to `true` for 29.97 and 59.94, and `false` otherwise.
5. **Render Export**:
   In `crates/montagent-render/src/encode.rs`, FFmpeg receives:
   - `-r 30000/1001` or `-r 24000/1001`
   - `-timecode 01:00:00;00` (written to MP4 container and SEI metadata).

---

## 4. Impact Analysis & Migration Plan

### 4.1 Schema (`schema/montagent.schema.json`)
- Expand `fps` definition with `oneOf`: `integer`, `{"num": ..., "den": ...}`, or `[num, den]`.
- Existing project files require **zero modifications** (`fps: 25` remains completely valid).

### 4.2 Core Engine (`crates/montagent-core`)
- In `crates/montagent-core/src/model/mod.rs`: Replace `pub fps: i64` with `pub fps: FrameRate`, where `FrameRate` implements `Into<Rational>` and deserializes from integer or `{num, den}` or `[num, den]`.
- In `crates/montagent-core/src/exact.rs`: Generalize `instant_of(n, fps)` and `frame_at_or_before(t, fps)` to take `Rational` frame rate:
  ```rust
  pub fn instant_of(n: i64, rate: Rational) -> i64 {
      ((i128::from(n) * 1000 * i128::from(rate.den)) / i128::from(rate.num)) as i64
  }
  ```
- In `crates/montagent-core/src/verbs/measure.rs`: Update `Instant` to compute rational grid instants and format SMPTE timecode.

### 4.3 Render Engine (`crates/montagent-render`)
- In `decode.rs`: Pass `rate.num` and `rate.den` into the tick base:
  `tick_base = 1000 * i128::from(rate.num) / i128::from(rate.den)`
  Filter argument becomes: `fps=fps={num}/{den}:round=up:start_time=0`.
- In `encode.rs`: Format `-r` as `"{num}/{den}"` for fractional rates and `"{fps}"` for integer rates.

---

## 5. Verification & Test Suite Recommendations

1. **Exact 1001 ms Periodicity Assertion**:
   Unit tests verifying that for $24000/1001$, $30000/1001$, and $60000/1001$, instant of frame $k \times \text{nominal}$ is exactly $k \times 1001$ ms.
2. **Boundary Half-Open Test**:
   Verify that for duration 1000 ms at 29.97 fps, exactly 30 frames are emitted, and for duration 1001 ms, exactly 30 frames are emitted.
3. **SMPTE Drop-Frame Roundtrip Test**:
   Test every frame from $0$ to $108,000$ (1 hour of 29.97 fps) converting to SMPTE DF timecode and parsing back to exact frame index $n$, asserting 100% bijective correspondence.
4. **Validation Test**:
   Assert that `"fps": 23.976` fails `validate` with a clear schema error, while `"fps": {"num": 24000, "den": 1001}` validates cleanly.

---

## 6. Conclusion

Fractional and NTSC timebases (23.976, 29.97, 59.94 fps) are represented with mathematical rigor in Montagent by adopting exact rational representations in the schema, retaining integer milliseconds for timeline positions, applying discrete integer floor arithmetic for frame sampling, and calculating SMPTE drop-frame timecode as an external query projection. This design satisfies all four invariants of ADR-0145, avoids IEEE-754 floating-point drift, and achieves complete parity with Premiere Pro's sequence timebase capabilities.
