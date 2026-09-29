---
status: accepted
amends: 0093 (corrects ruling 6 condition 1: the failed-seek predicate is **not** computable before the frame loop, because the frame grid is a property of the source's timestamps and not of either frame rate `probe` reports — what is pre-flightable is the coarser question the clamp leaves behind), 0011 (records what the quad does *not* establish: neither `r_frame_rate` nor `avg_frame_rate` is the source's frame grid, and on this repository's own reference MP4 neither is even close to it), 0020 (`overrun: "hold"`'s *"resolve to `source_end`"* now reaches a frame rather than a failed seek, which is what the rule always meant)
---

# The frame at an instant is the last one starting at or before it

[#387](https://github.com/MBehtemam/Montagent/issues/387) (MONTAGENT-2), the second of the
nine findings from the first real end-to-end build through this tool. The reported symptom
was small and the cause was not:

```
not painted  av-dialogue-01 — .../01-interviewer.mov: no frame at 3.276s
             — the seek is past the end of the source
```

on a file that is 3.280 s and 82 frames long. 3.276 s is *inside* frame 81, which covers
[3.240, 3.280). The frame rendered without the element — **one dropped frame at the end of
every line, 60 in that episode.** Invisible in a still; a flicker in motion.

## What was actually there

Montagent samples a video element at `frame_time - element.start` and hands that offset to
`ffmpeg` as `-ss`. **`ffmpeg`'s input seek returns the first frame whose timestamp is `>= t`.**
That is measured, not inferred — on a ProRes 82-frame 25 fps source, a long-GOP H.264 encode
of the same frames, and a ProRes source at 30000/1001, identifying each returned frame by its
pixels:

| asked for | frame it covers | `ffmpeg` returned |
| --- | --- | --- |
| 1.200 s | 30 | 30 |
| 1.234 s | 30 | **31** |
| 1.239 s | 30 | **31** |
| 3.240 s | 81 | 81 |
| 3.276 s | 81 | **no frame at all** |

Keyframe density makes no difference: the long-GOP encode answers identically, because an
accurate seek decodes forward from the preceding keyframe.

So the defect is **not** one dropped frame at the end. An element whose start is off the
source's frame grid is sampled off-grid at *every* instant — the offsets step by the project's
frame period plus a constant remainder — so **every frame of that element was one source frame
early**, for its whole duration. The drop is simply where "one frame early" runs out of file.
The source doc's own workaround, *"trim every video element by one frame period"*, is a
compensation for the shift; it removes the drop and leaves the shift in place.

## Why the obvious fix does not work

#387 proposed *"clamp the seek to the last frame whose start is `<= t`"*, and ADR-0093's
ruling 6 condition 1 assumed the same thing was cheap: *"the seek predicate is computable
before the frame loop"*. Both presuppose that Montagent can **compute the source's frame
grid**. It cannot, and this repository's own reference MP4 is the counterexample.

`fixtures/en-halloween-decorating/reference/en-halloween-decorating.mp4`, read frame by frame:

- its real frame timestamps start at **42.031 ms**, not at zero, and step **40 ms** — 25 fps;
- `r_frame_rate` reports **50/1**, which is the container's tick base, off by 2×;
- `avg_frame_rate` reports **1390080/55651 ≈ 24.9785**, a quotient of frames over a duration
  that includes the 42 ms offset;
- and **four gaps of 51–58 ms** sit in the middle of it, where frames were dropped.

So the file is near-CFR-25 with a nonzero origin and four irregularities, and **neither of
ADR-0011's two frame rates is its grid.** Arithmetic on either lands on the wrong frame; adding
`start_time` to compensate lands two or three frames out. ADR-0011 introduced the quad because
*"on the fixture's reference MP4 these disagree"* — this ADR records the consequence for one
specific consumer: the quad establishes durations and rates, and **it does not establish where
the frames are.**

A grid also does not *exist* for a variable-frame-rate source, which is the general case the
reference MP4 is only one step away from.

## The decision

### 1. The question goes to the authority that holds the timestamps

`frame_at` no longer computes anything about the grid. It asks `ffmpeg` for the last frame
starting at or before the instant:

```
-ss <at − 200 ms> -copyts -t <window + 1 ms> -i <source>
-vf select='lte(t,<at + 1 µs>)',scale=<w>:<h> -vsync 0
```

- **`-ss` lands one window early**, still the input seek, so `ffmpeg` jumps to the preceding
  keyframe rather than reading the file from zero — at ADR-0021's 500 ms budget that
  distinction is the whole budget.
- **`-copyts`** keeps the source's own timestamps, so the filter compares against source time
  rather than time rebased onto the seek point.
- **`select='lte(t,…)'`** keeps every frame starting at or before the instant; the **last** of
  them is the answer. A streaming filter cannot know which frame is last, so the run is read
  to its end and the final whole frame is taken.
- **`-vsync 0`** stops `ffmpeg` inventing or dropping frames to hit an output rate.

This is exact on a grid Montagent never has to know, which is the point: it is correct for
CFR, for fractional rates, for dropped frames and for VFR alike.

### 2. `-t` is load-bearing, not tidiness

`select` places no limit on the output, so without `-t` `ffmpeg` reads to **end of file**,
decoding and discarding everything after the instant. Measured at 1080p over twelve off-grid
instants:

| | ProRes all-intra | long-GOP H.264 |
| --- | --- | --- |
| the old single-frame seek | 110 ms | 116 ms |
| at-or-before, read to EOF | 247 ms | 196 ms |
| at-or-before, `-t` bounded | **142 ms** | **143 ms** |

So the correct answer costs **~20%**, not 2×, and the bounded cost does not grow with the
length of the source while the unbounded one does. The window's *width* barely enters that
figure — the cost is the seek and the spawn, not the four extra frames — which is why the
window is set from correctness and not from the budget.

`-t` is `window + 1 ms`, and the extra millisecond is measured too: at exactly `window`, the
frame sitting on the read boundary is cut and the answer is the frame before it.

### 3. The window is 200 ms, bounded by the largest gap and not by the frame period

The window must span the largest gap between two consecutive frames, because that is where the
frame being looked for might be. 200 ms spans any gap down to 5 fps. The bound is stated as a
**gap** rather than as a frame period precisely because of the reference MP4: it is nominally
25 fps and carries gaps of 51–58 ms, so a window derived from `1/fps` would have been too
narrow for its own fixture.

### 4. The threshold carries one microsecond of slack, and the slack is exact

`select` evaluates `t` as a float, so a frame whose start *is* the instant compares marginally
above it and `lte` drops it. Measured: a 25 fps source asked for 1400 ms returned frame 34
instead of 35, and a 30000/1001 source asked for 1001 ms returned 29 instead of 30 — both
exactly on a frame start.

The slack is **one microsecond**, and it is neither a guess nor a float:

| slack | 25 fps | 30000/1001 |
| --- | --- | --- |
| none | fails on every exact frame start | fails on every exact frame start |
| **1 µs** | **correct** | **correct** |
| 10 µs | correct | correct |
| 100 µs | correct | **over-includes the next frame** |

Coarser slack breaks on fractional rates, where a frame start can sit a fraction of a
millisecond *above* a whole millisecond and gets pulled in. A microsecond clears float error
by three orders of magnitude and stays three below the closest a real source's frames come to
each other. It is written as integer digits — 1.001 ms becomes `1.001001` s — so ADR-0005's
*"every time is an integer millisecond"* is still never routed through a float.

### 5. The clamp is symmetric, because a container's origin is not the author's mistake

An instant *before* the source's first frame has no frame at or before it, and the first
version of this fix therefore refused one — which would have moved MONTAGENT-2's silent
failure from the end of a source to the beginning of one. **This repository's own reference
MP4 is that case**: its frames start at 42.031 ms, so an element with `source_start: 0` asks
for instants no frame covers, and every such instant failed.

So where the window holds no frame at or before the instant, the answer is the **earliest
frame in the window**. A source that begins 42 ms late is a container's own origin, not a
statement by the author, and ADR-0020 gives them no field to declare it with.

The fallback is a second `ffmpeg` over the *same* window rather than a wider search, which is
what keeps it a clamp: an instant past the *end* of the source finds nothing in either
direction and stays an error, as below.

### 6. Past the end of the source paints the last frame; far past it is still an error

An instant after the final frame's start resolves to that frame, which is what ADR-0020's
`overrun: "hold"` always meant by *"resolve to `source_end`"* — `source_end` is a boundary,
and the frame shown at a boundary is the one before it. Previously that seek could fail.

The clamp reaches back one window, so an instant **more than 200 ms past the last frame** still
has no answer, and that stays a finding: it means the source ends well before the range the
document declares, which is a real disagreement between document and file rather than a
rounding artefact. ADR-0006 prefers the loud failure and ADR-0093 makes it an `error` that
withholds the deliverable. `E-NOT-PAINTED-UNDECODABLE` keeps the case, with wording that now
says *"no frame at or before"* rather than *"the seek is past the end"*.

### 7. ADR-0093's ruling 6 condition 1 is corrected, not abandoned

ADR-0093 justified leaving MONTAGENT-2's failed seek as an interim `error` on the grounds that
*"the seek predicate is computable before the frame loop"*. §2 above shows it is not: deciding
whether a seek will land needs the source's real frame timestamps, which `probe` does not read
and which a full-file scan would be required to get.

The ruling survives in a **weaker and correct** form, because the clamp removes the case it was
about. What remains is the coarse question — *does this source end more than a window before the
declared range?* — and that **is** answerable from `Quad::video_stream_ms` before any frame is
drawn. Whether `render` should pre-flight it is left to
[#413](https://github.com/MBehtemam/Montagent/issues/413); nothing about this ADR depends on
the answer, because the case is now an `error` either way and ADR-0093 already withholds the
deliverable.

## Scope

**This ADR is about `frame_at`, the single-frame decode.** `frames_from` — the run API behind
`measure`'s keyed-alpha series and ADR-0088's coverage reading — resamples through `fps=` and
answers a different question, *"a series at a rate"* rather than *"the state at an instant"*.
Whether that filter has the same off-by-one is not measured here and is
[#414](https://github.com/MBehtemam/Montagent/issues/414).

**It changes pictures.** Any project with a video element whose start is off its source's frame
grid now paints a different — correct — frame at every instant of that element. **No golden
frame in the repository moves, and that is an absence of coverage rather than evidence of
safety**: every committed golden comes from `en-halloween-decorating`, which ADR-0003's
asymmetry already flagged as having **zero `video` elements**, or from a synthetic text/mask
project. `fixtures/video-decode` is the only project in the repository with a video element and
it has no golden frame. So `tests/seek_clamp.rs` is not a supplement to the goldens here — it is
the only thing holding this behaviour, which is why it identifies the frame it got by a number
encoded in the pixels rather than by comparing a picture.

**It does not give `probe` a frame-timestamp reading.** That would make the grid computable and
open up a cheaper clamp, and it is a full-file scan on a cached sidecar — a larger question than
this finding, raised as [#415](https://github.com/MBehtemam/Montagent/issues/415).

## Evidence

`docs/adr/frame_at_or_before_check.sh` reproduces every number above from a generated fixture
and from the committed reference MP4: `ffmpeg`'s `>=` behaviour, the reference MP4's real grid
against both declared rates, the `-t` cost figures, and the slack table. It asserts what follows
from the mechanism, never that 200 ms or a microsecond is the only workable pair — §3 and §4
give the range each was chosen from.
