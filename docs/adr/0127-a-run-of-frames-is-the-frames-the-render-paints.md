---
status: accepted
amends: 0096 (its at-or-before rule now governs a *run* of frames as well as a single one — `frames_from` samples the frames `frame_at` would return at `render`'s own instants, and §1's "the question goes to the authority that holds the timestamps" is carried to the run), 0088 (the coverage series is sampled at the frames the render paints, and its stated `source_fps` is `fps ÷ speed`, not `fps × speed`)
---

# A run of frames is the frames the render paints

[#414](https://github.com/MBehtemam/Montagent/issues/414). ADR-0096 measured that `ffmpeg`'s
input seek returns the first frame at or *after* an instant, and it fixed `frame_at`, the
single-frame decode, to return the frame the source is *showing*. `frames_from` was neither
measured nor changed. It is the run API behind `measure`'s keyed-alpha coverage series
(ADR-0088), and it sampled differently: `-ss <from> -vf fps=<fps × speed>`.

#414 asked whether `fps=` lands on the frames the timeline shows, and whether any error would
be a constant offset that leaves ADR-0088's *"did this key **stay**"* reading intact. It does
not land on them, and the error is not constant.

## What was measured

The oracle is the renderer's own arithmetic. Timeline frame `n` is painted at `⌊n × 1000 / fps⌋`
ms (`exact::instant_of`). The source offset is `source_start + round_half_up(elapsed × speed)`
(`exact::source_advance`), and the frame is the last one starting at or before that offset,
with ADR-0096's one microsecond of slack. The sources were:

- a 25 fps file, a 30 fps file and a 30000/1001 file, each numbering its own frames in its red
  channel (ADR-0096's generator);
- a synthetic file shaped like the reference MP4, with frames from 42 ms and a gap;
- the committed reference MP4 itself, its frames identified by matching each picture against
  its own full decode (1629 frames, 1624 distinct).

The matrix covered start points on and off the source's grid; `fps` 24, 25, 30 and 60; and
`speed` 1, 0.5, 1.5, 2 and 0.645. There were 504 runs of 30 frames each.

| | runs matching the render frame for frame |
| --- | --- |
| `-ss <from> -vf fps=<fps × speed>` (before) | **3 of 504** |
| this ADR's chain | **504 of 504** |

Every wrong frame was *later* in the source than the painted one. The run was never behind.
Three independent mechanisms caused it, and they compound. On the synthetic late-starting
source, runs were up to three frames ahead.

1. **The seek is ADR-0096's, without `-copyts`.** A start off the source's grid began the run
   on the next frame, and every frame after it inherited the shift. Without `-copyts`, `ffmpeg`
   also rebases the file onto zero. So a source whose frames start at 42.031 ms, like this
   repository's reference MP4, was read a frame ahead from its very first instant. This
   happened even from `0 ms` on the grid.
2. **`fps=` rounds to the nearest tick by default.** Wherever the project's rate and the
   source's differ, a tick closer to the *next* frame's start than to the current one's took
   the next frame. At 30 fps over a 25 fps source from 0 ms, 14 of 40 frames were a frame
   ahead, and the pattern repeats every six frames. **This is the error that is not
   constant.** It shifts some frames of a run and not others. A key edge can therefore appear
   to move by one frame in the series when it does not in the render.
3. **The rate was inverted in `speed`.** At `speed: 2`, one timeline frame moves `2 / fps`
   seconds through the source, which is `fps ÷ 2` samples per second of source time. The run
   asked `fps=` for `fps × 2`, so a speed-2 series sampled four times too densely and covered a
   quarter of the source range it named. At `speed: 0.5` it covered four times the range. The
   answer's `source_fps` field stated the same wrong rate.

So #414's hope does not hold. A constant offset might have left *"did this key stay"* intact,
but mechanisms 2 and 3 make the error vary from frame to frame, and the series is exactly the
reading where that shows.

## Decision

### 1. A run asks the renderer's question, in the renderer's integer arithmetic

`frames_from` takes the project's `fps` and the element's `speed` as the exact rational
`(numerator, denominator)` of its decimal, in a `Pace`. It no longer takes a float rate. It
asks `ffmpeg` for:

```
-ss <from − 200 ms> -copyts -i <source>
-vf settb=1/1000000,
    setpts='ceil((2*(ceil((PTS-1)/1000)-<from>)-1)*<den>/(2*<num>))*1000',
    fps=<fps>:round=up:start_time=0,
    scale=<w>:<h>
-fps_mode passthrough
```

- **`-ss` one window early with `-copyts`** is `frame_at`'s window, for `frame_at`'s reasons.
  The frame already showing at `from` is decoded, and the comparison is in source time.
- **`settb` puts every timestamp on the microsecond.** Then, per frame, `setpts` rewrites the
  source start `p` to the first **timeline** millisecond the render shows it at. That is done
  in two integer steps:
  - `q = ⌈(p − 1 µs) / 1 ms⌉` is the first source millisecond whose at-or-before query returns
    this frame, which is ADR-0096's rule with its slack.
  - `⌈(2(q − from) − 1) × den / (2 × num)⌉` inverts `round_half_up(t × speed)`, giving the
    earliest timeline millisecond `t` whose offset reaches `q`.
- **`fps=<fps>:round=up`** then assigns each frame to the first tick at or after its
  timeline start, so tick `n` shows the last frame starting at or before `n × 1000 / fps`.
  Because those starts are whole milliseconds, that is exactly *at or before
  `⌊n × 1000 / fps⌋`*, the render's instant. `start_time=0` holds the earliest frame over any
  tick before the first frame has started, which is `frame_at`'s symmetric clamp.

Every operand in the expression is an integer. The evaluator's double is exact below 2^53, so
there is no slack to choose beyond ADR-0096's microsecond, and no float stands between the run
and the render. The one float left is `measure`'s statement of `source_fps`, which nothing is
computed from.

### 2. A speed finer than six decimal places is refused, not approximated

The products `(2q − 1) × den` must stay under 2^53. With each term of `speed` in lowest terms
at most a million, which is six decimal places, that holds for every millisecond of any source
shorter than 52 days. A finer `speed` is refused with a sentence rather
than rounded, for the reason ADR-0045 refuses to round a decimal: a rounded `speed` samples
frames the render does not paint, and the reading would say nothing about why.

### 3. `source_fps` is `fps ÷ speed`

The field keeps its name and its meaning, *source frames per second of source time*. Its
value was wrong and now matches that meaning. It was never an input to the run, and it still
is not.

### 4. The run may end one frame late, and the caller stops it

`fps=` pads the run's last frame to the tick after the source ends, so a run can be one frame
longer than before. `measure` counts the frames the element's range shows and stops there
(`keyed.rs`), which it already did.

## Consequences

- `measure`'s coverage series now reports the frames the render paints. Series already read
  off off-grid, retimed, mismatched-rate or late-starting sources were one to three frames
  ahead, and a retimed element's series covered the wrong span of source.
- The cost is unchanged: 1.0–1.7 s for 700–3000 frames of the reference MP4 either way. The
  window adds at most 200 ms of decode at the start of the run.
- `crates/montagent-core/tests/seek_clamp.rs` holds the behaviour. It checks each frame of a
  run against `frame_at` at `render`'s instant, over three grids, eight
  start/rate/speed combinations and twelve frames each, plus one literal test per
  mechanism above. Each of those tests fails against the old command.
- `docs/adr/frames_from_run_check.py` reproduces the 3/504 and 504/504 measurements against
  this machine's `ffmpeg`. It takes about five minutes, so it is not in CI.
- **Not decided here:** whether the renderer's own millisecond instants are right. On a 30 fps
  QuickTime source at 30 fps, frame `n` starts at `33.33…n` ms, and the render asks for
  `⌊33.33…n⌋`. Measured through `frame_at`'s own command, timeline frames 1, 2 and 3 paint
  source frames 0, 1 and 3: one frame is held and one is skipped in every three. That is
  ADR-0005's integer millisecond meeting a source whose frames do not start on one. This ADR
  makes the run agree with the render, whichever way that question is settled; it is
  [#522](https://github.com/MBehtemam/Montagent/issues/522).
