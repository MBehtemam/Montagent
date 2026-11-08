---
status: accepted
amends: 0096 (maps timeline frame n through its exact rational frame time n / fps seconds and sub-millisecond source microseconds when selecting the source video frame showing at that frame, while keeping the floored integer millisecond for element presence, keyframe interpolation, and query --at), 0127 (updates the run-of-frames setpts filter and offset_us oracle to invert the microsecond source offset of timeline frame n's exact rational time rather than its floored millisecond)
---

# Source video frame selection uses exact rational frame time

[#522](https://github.com/MBehtemam/Montagent/issues/522).
Under [ADR-0077](0077-the-nine-render-readings-are-ratified.md), [ADR-0096](0096-the-frame-at-an-instant-is-the-last-one-starting-at-or-before-it.md), and [ADR-0127](0127-a-run-of-frames-is-the-frames-the-render-paints.md), timeline frame `n` was sampled at the floored integer millisecond `t_n = ⌊n × 1000 / fps⌋` and mapped to an integer-millisecond source offset before asking `ffmpeg` for the last source frame with `pts_time <= offset_ms + 0.000001`.

When `1000` is not divisible by `fps` (such as 30 fps, 60 fps, or 24 fps), flooring `n × 1000 / fps` to an integer millisecond lands strictly before the source video's own `n / fps` timestamp whenever `(n × 1000) mod fps > 0`:

- At 30 fps (`1000 / 30 = 33.333... ms`), timeline frame 1 has floored instant `t_1 = 33 ms`, whereas frame 1 of a 30 fps source sits at `1 / 30 s = 33,333 µs = 33.333 ms`.
- Asking `select='lte(t, 0.033001)'` at `t_1 = 33 ms` therefore misses source frame 1 (`0.033333 s`) and re-selects source frame 0 (`0.000000 s`); source frame 1 is painted at timeline frame 2 (`66 ms >= 33.333 ms`), and only every third timeline frame (`n ≡ 0 (mod 3)`, where `n × 1000 / 30` is an exact integer millisecond) catches up by skipping a source frame.
- Consequently, a 30 fps source in a 30 fps project at `speed = 1` with `start == from == 0` repeated every `3k`-th source frame twice and dropped every `(3k + 2)`-th source frame across `render`, `frame`, and `measure`.

This ADR amends [ADR-0096](0096-the-frame-at-an-instant-is-the-last-one-starting-at-or-before-it.md) and [ADR-0127](0127-a-run-of-frames-is-the-frames-the-render-paints.md) so that source video frame selection maps timeline frame `n` through its exact rational timeline time `n × 1000 / fps` ms to a microsecond source offset `offset_us`.

---

## Decisions

### 1. Exact rational frame time for source video frame selection (amends [ADR-0096](0096-the-frame-at-an-instant-is-the-last-one-starting-at-or-before-it.md))

When painting timeline frame `n` at project frame rate `fps`, an active non-remapped video element (`origin = (timeline_ms, source_ms)`, `pace = (fps, speed = num / den)`) maps frame `n`'s exact rational timeline time `1000 × n / fps` ms to a microsecond source offset:

$$\text{offset\_us}(n) = 1000 \times \text{source\_ms} + \left\lfloor \frac{2000 \times (1000 \times n - \text{timeline\_ms} \times \text{fps}) \times \text{num} + \text{fps} \times \text{den}}{2 \times \text{fps} \times \text{den}} \right\rfloor$$

Single-frame decode (`decode::frame_at_us`) seeks a 200 ms window before `offset_us` with `-copyts` and selects the last decoded frame satisfying `lte(t, (offset_us + 1) / 1_000_000)`, preserving [ADR-0096](0096-the-frame-at-an-instant-is-the-last-one-starting-at-or-before-it.md)'s 1 µs slack at microsecond resolution.

- When `frame --at <instant>` or a contact sheet samples an instant that equals the painted millisecond `⌊n × 1000 / fps⌋` of timeline frame `n`, `Painter` supplies `frame_number = Some(n)` so `frame --at`, contact sheets, `render`, and `measure` select the identical source video frame.
- Off-grid `frame --at` instants that do not correspond to any timeline frame `⌊n × 1000 / fps⌋` (and keyframed `remap` source-time evaluations) continue to evaluate at `offset_ms × 1000` µs via `decode::frame_at`.
- Element presence `[start, end)`, keyframe interpolation, `motion_blur` shutter centres, and `query --at` remain on the integer-millisecond clock (`⌊n × 1000 / fps⌋`).

### 2. Run-of-frames `setpts` inversion on exact rational frame time (amends [ADR-0127](0127-a-run-of-frames-is-the-frames-the-render-paints.md))

A source frame with microsecond timestamp $P = \lfloor 10^6 \times \text{pts\_time} \rfloor$ is eligible for timeline frame $n$ iff $P \le \text{offset\_us}(n) + 1$. Because $\text{offset\_us}(n)$ is monotone in $n$, $P \le \text{offset\_us}(n) + 1$ holds iff

$$1000 \times n - \text{timeline\_ms} \times \text{fps} \ge \left\lceil \frac{(2 P - \text{bias}) \times \text{den}}{2 \times \text{num}} \right\rceil$$

where $\text{bias} = \text{fps} \times (2 \times (1000 \times \text{source\_ms} + 1) + 1)$.

In `decode::frames_at`, the filter graph therefore converts timestamps first to microseconds (`settb=1/1000000`) so `PTS` is integer microseconds $P$, then to tick base $\text{tick\_base} = 1\,000\,000 \times \text{fps}$ (`settb=1/{tick_base}`), and maps each source frame to the earliest tick at which it is eligible relative to the run's first timeline frame `first`:

```text
settb=1/1000000,settb=1/{tick_base},setpts='ceil((2*PTS-{bias})*{den}/(2*{num})){shift:+}',fps={fps}:round=up:start_time=0,{scale}
```

where $\text{shift} = 1000 \times \text{timeline\_ms} \times \text{fps} - 1\,000\,000 \times \text{first}$. On this tick base, timeline frame $n = \text{first} + k$ sits at tick $1\,000\,000 \times k$, and `fps={fps}:round=up:start_time=0` selects the last source frame whose mapped tick is at or before $1\,000\,000 \times k$ — matching `decode::frame_at_us(offset_us(n))` frame for frame across `render`, `frame`, and `measure`.

---

## Consequences

- A 30 fps source in a 30 fps project at `speed: 1` with `start == from` steps 1:1 through source frames `0, 1, 2, …` across `render`, `frame`, and `measure`.
- `decode::offset_us` and `decode::frame_at_us` provide the exact microsecond single-frame oracle beside `decode::offset_at` and `decode::frame_at`.
- `crates/montagent-core/tests/seek_clamp.rs` verifies both the 504-case run-vs-single-frame equivalence matrix and the 1:1 30-fps-in-30-fps stepping across `render`, `frame`, and `measure`.
