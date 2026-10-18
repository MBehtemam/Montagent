---
status: accepted
amends: 0005 (§2: sequence fps accepts exact rational fraction alongside integer), 0035, 0077
---

# Fractional and NTSC timebases are exact rational fractions and timeline points stay integer milliseconds

> **Amended by [ADR-0211](./0211-cmx-3600-edl-and-fcpxml-translate-via-smpte-timecode-and-lane-mapping.md)**: CMX 3600 EDL and FCPXML translate via SMPTE timecode and lane mapping.

> **Amended by [ADR-0210](./0210-opentimelineio-translates-bi-directionally-via-rational-time-and-sequential-tracks.md)**: OpenTimelineIO translates bi-directionally via rational time and sequential tracks.

> **Amended by [ADR-0209](./0209-timeline-interchange-is-a-decoupled-zero-renderer-conversion-engine.md)**: Timeline interchange is a decoupled zero-renderer conversion engine.

[#980](https://github.com/MBehtemam/Montagent/issues/980), [#986](https://github.com/MBehtemam/Montagent/issues/986), [#987](https://github.com/MBehtemam/Montagent/issues/987).

## Context

Standard cinema and North American broadcast frame rates are not whole integers:
- **23.976 fps** (24p film): exactly $24000 / 1001 = 23.\overline{976023}$ frames per second.
- **29.97 fps** (30p television / 60i): exactly $30000 / 1001 = 29.\overline{970029}$ frames per second.
- **59.94 fps** (60p high frame rate): exactly $60000 / 1001 = 59.\overline{940059}$ frames per second.

In decimal notation, these numbers are infinitely repeating. Writing a rounded decimal float such as `"fps": 23.976` causes a mathematical discrepancy of $3.6\text{ ms per hour}$ ($23.976 = 23976/1000 \ne 24000/1001$), accumulating audio/video sync drift. Furthermore, passing IEEE-754 floating-point values into boundary-sensitive ceiling, floor, or rounding operations triggers divergence across CPU platforms and optimization levels due to Unit in the Last Place (ULP) hazards ([ADR-0028](0028-text-block-arithmetic-is-exact-tenths.md), [ADR-0045](0045-speed-invariant-is-evaluated-in-exact-arithmetic.md)).

At the same time, moving timeline coordinates (`start`, `end`, `source_start`, `source_end`, keyframe `t`) from milliseconds to rational fractions, frame indices, or arbitrary ticks ($254,016,000,000/\text{sec}$) would violate [ADR-0005](0005-absolute-integer-milliseconds.md) ("reading is free") and break exact-string replacement ([ADR-0145](0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md)).

## Decision

1. **Exact Rational Schema (`fps`)**:  
   The project sequence `fps` field in `schema/montagent.schema.json` accepts either:
   - A positive integer (`24`, `25`, `30`, `50`, `60`), or
   - An exact rational object `{"num": N, "den": D}` (e.g. `{"num": 24000, "den": 1001}`), or
   - An exact rational integer pair `[num, den]` (`[24000, 1001]`), matching the pixel aspect ratio precedent of [ADR-0023](0023-video-source-dimensions-par-and-rotation.md).  
   Decimal floats (`"fps": 23.976`) are strictly rejected by JSON schema validation and `montagent validate` (`E-SCHEMA`).

2. **Timeline Points Stay Integer Milliseconds**:  
   All timeline points and durations (`start`, `end`, `source_start`, `source_end`, keyframe `t`, project `duration`) **remain positive integer milliseconds (`i64`)**.  
   - Under the **1001 ms Periodicity Theorem**, every exact $1001\text{ ms}$ interval contains an exact integer number of frames ($\text{nominal\_fps}$ frames, since $(1001 \times \text{nominal\_fps} \times 1000) / (\text{nominal\_fps} \times 1000 \times 1001 / \text{nominal\_fps}) = \text{nominal\_fps}$). There is zero cumulative fractional drift over time.
   - The presence invariant of [ADR-0077](0077-the-nine-render-readings-are-ratified.md) holds unchanged: because element boundaries are whole milliseconds, $\lfloor T_n \rfloor \ge \text{start} \iff T_n \ge \text{start}$ and $\lfloor T_n \rfloor < \text{end} \iff T_n < \text{end}$.

3. **Integer Arithmetic Engine in `exact.rs`**:  
   Frame sampling is evaluated entirely in 128-bit integer arithmetic with zero floating-point operations:
   - **Painted Whole Millisecond Instant**:
     $$\text{instant\_of}(n, \text{fps}) = \left\lfloor \frac{n \times 1000 \times \text{den}}{\text{num}} \right\rfloor$$
   - **Frame At or Before Instant $t$**:
     $$\text{frame\_at\_or\_before}(t, \text{fps}) = \left\lfloor \frac{t \times \text{num}}{1000 \times \text{den}} \right\rfloor$$
   - **First Frame At or After Instant $t$**:
     $$\text{frame\_at\_or\_after}(t, \text{fps}) = \left\lceil \frac{t \times \text{num}}{1000 \times \text{den}} \right\rceil = \left\lfloor \frac{t \times \text{num} + (1000 \times \text{den} - 1)}{1000 \times \text{den}} \right\rfloor$$
   - **Frame Immediately Before Instant $t$**:
     $$\text{frame\_before}(t, \text{fps}) = \text{frame\_at\_or\_after}(t, \text{fps}) - 1$$
   - **Total Frames in Span $[0, D)$**:
     $$N = \left\lceil \frac{D \times \text{num}}{1000 \times \text{den}} \right\rceil$$
   - **Span Emptiness Test (`holds_a_sampled_frame`)**:  
     A span $[from, to)$ contains a sampled frame if and only if:
     $$\text{first} \times 1000 \times \text{den} < to \times \text{num}, \quad \text{where } \text{first} = \text{frame\_at\_or\_after}(from, \text{fps})$$

4. **Verification in `montagent verify`**:  
   `montagent verify` measures the Presentation Time Stamps (PTS) of the encoded output video stream using `ffprobe`. For rational timebases:
   - Video stream timebase is asserted to match $\frac{\text{den}}{\text{num}}$ (or a standard container timebase multiple such as $1/90000$ or $1/24000$).
   - Measured PTS step intervals are verified against expected frame intervals without accumulating drift.
   - Total decoded frame count is asserted to equal $N = \lceil (D \times \text{num}) / (1000 \times \text{den}) \rceil$.

5. **SMPTE Timecode Projection**:  
   Timecode is strictly a projection lens queryable in `measure` and `query`, never document state:
   - Standard integer frame rates and 23.976 fps use Non-Drop Frame (NDF) formatting (`HH:MM:SS:FF`).
   - 29.97 fps and 59.94 fps default to Drop Frame (DF) formatting (`HH:MM:SS;FF`), dropping 2 frame numbers (4 frame numbers at 59.94) at the start of every minute except every 10th minute, keeping the timecode display aligned to wall-clock time within 75 ms over 24 hours.

## Consequences

- Authors and agents can write standard North American broadcast frame rates without float rounding errors.
- Existing projects specifying integer `fps` (`24`, `25`, `30`, `60`) continue to validate and parse without modification.
- Element coordinates remain human-readable integer milliseconds (`ms`), fully preserving the "reading is free" property of ADR-0005.
