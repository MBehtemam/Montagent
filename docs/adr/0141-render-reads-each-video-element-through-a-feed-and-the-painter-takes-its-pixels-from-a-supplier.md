---
status: proposed
amends: 0021 ("one painter" is one place that decides visibility, offset, extent and findings; the pixels come through a frame supplier, per frame for `frame` and from feeds for `render` and `preview`), 0127 ("run of frames" becomes **feed**; the run now supplies `render`, anchored at the element's own origin so it can open partway through one; `tests/feeds.rs`'s byte-equality and spawn-count test joins `seek_clamp.rs`)
---

# `render` reads each video element through a feed, and the painter takes its pixels from a supplier

[#532](https://github.com/MBehtemam/Montagent/issues/532), settled in
[#626](https://github.com/MBehtemam/Montagent/issues/626) over three grilling rounds, each put
to a court of three independent jurors; the decisions are the jurors' consensus as amended by
the Judge's read, which the dev adopted. Three later tickets add to it:
[#635](https://github.com/MBehtemam/Montagent/issues/635) (`preview`),
[#636](https://github.com/MBehtemam/Montagent/issues/636) (the feed budget) and
[#637](https://github.com/MBehtemam/Montagent/issues/637) (hardware decode). The spawn-count
test is [#625](https://github.com/MBehtemam/Montagent/issues/625) §4's, and the produce/push
split is [#627](https://github.com/MBehtemam/Montagent/issues/627) §7's.

## The gap

A real six-minute project with three videos on screen took about an hour to render. Every
`video` element was decoded through one `frame_at` per frame: one `ffmpeg` spawn, a 200 ms
seek window, one frame kept. ADR-0077 recorded that as a cost and left it unmeasured, because
the committed fixture has no `video` element. #623's profile put it at 93–113 ms per frame per
visible video element: about 95% of the hour.

`frames_from` already existed, as `measure`'s coverage series, and ADR-0127 had made it return
exactly the frames the render paints.

**The gate passed before anything was built.** A throwaway test, since deleted, compared each
`frames_from` frame's RGBA bytes with `frame_at` at `render`'s own instants: **17,280 of
17,280 frames were identical, and the largest per-channel difference was 0** (ffmpeg 9.0.2).
It covered H.264 at 30, 25 and 30000/1001 fps, ProRes 422 10-bit and the reference MP4 (frames
from 42 ms, real dropped frames); the source's own size, three downscales and an odd-aspect
upscale; fps 24, 30 and 60; speed 1, 0.5, 2 and 0.645; starts on and off the grid. It did not
cover VP9 with alpha, interlaced sources or full-range sources.

## Decision

### 1. The painter takes its pixels from a supplier

The `Painter` owns a `Box<dyn FrameSupplier>`, passed to its constructor.

- `frame` (and its sheet) passes **`PerFrame`**: one `frame_at` per request, which is what
  the painter did before. `frame`'s behaviour is unchanged.
- `render` and `preview` pass **`Feeds`**, through `encode_span`.

Nothing in the painter branches on which verb it serves. A request carries the element's
key (its index in the painter's element table, stable for the painter's life), the timeline
frame `n`, the offset into the source the painter already computed, where the current pass
over the source began, the declared `w×h`, and the project's `fps` and the element's `speed`
from the same element table. Both suppliers answer with the frame `frame_at` returns at that
offset.

`supply` returns a frame or a typed `Failure`: `frame_at`'s own sentence, or a feed's source,
starting offset, frames delivered and `ffmpeg`'s stderr. **Only the painter turns a failure
into a finding** (`E-NOT-PAINTED-UNDECODABLE`), so ADR-0093's classes (`review` for `frame`,
`error` for `render`) stay in one place.

`begin_frame(n)` closes every feed that was not asked for during frame `n − 1`. It is there
rather than in an `end_frame`, so an early return from `paint` cannot leak a child process
past the next frame. The span's end, a cancel and an abandoned `preview` rung close the rest
through `Drop`: `Frames`' own `Drop` kills and reaps its `ffmpeg`.

The rejected alternative was having `render` pre-feed rasters to the painter. That would put a
second copy of the visibility and offset logic outside the painter.

**This amends ADR-0021.** *"One painter"* is one place that decides visibility, the offset
into the source, the extent and the findings. Where its pixels come from is the supplier's.

### 2. A feed opens, serves, reopens and closes

A **feed** is one `ffmpeg` decoding one video element's frames in timeline order, at the
element's declared size and a constant pace (`CONTEXT.md`). One feed per element: two elements
on one file get two. Sharing one is a later optimisation.

- **Open** lazily, at the first frame that paints the element, from that frame's offset. That
  one rule covers a partial range, an element starting partway through the span, and
  re-entry.
- **Serve**, in this order (the order is the rule):
  1. **The feed's predicted next frame.** The request is for the feed's next timeline frame,
     at `decode::offset_at` of it. Read the next frame off the feed.
  2. **A hold.** The same offset as the last frame served. Serve the cached frame. This comes
     after step 1, because at a slow `speed` the predicted next offset can equal the last
     one, and checking the hold first would leave the feed a frame behind for good. Measured:
     at `speed: 0.02` (0.8 source ms per 25 fps frame), hold-first reopened 20 times in 100
     frames; prediction-first reopens 0 times.
  3. **Anything else**: a loop wrap, a skip. Drop the feed and reopen it at this frame.
- **Close** at the first frame that does not paint the element (in `begin_frame`, §1), and at
  the span's end or cancel.
- **No runtime guard against churn.** Reopens follow how often the document loops, re-enters
  or starts partway, never the frame count. A feed reopened on every frame costs about what a
  `frame_at` per frame cost, so a bug can lose the speed-up but cannot make rendering slower
  than before. The spawn-count tests (§7) catch it. A heuristic fallback would be a second
  path, and it would hide the bug.

**A feed is measured from the element's own origin, not from wherever it opened.** This is a
correction to #626 §3's formula, `o0 + source_advance(instant_of(n0 + k) − instant_of(n0),
speed)`. The render's offset is `source_start + round_half_up((instant − start) × speed)`,
and `round_half_up` does not distribute over a sum, so a feed measured from its own first frame
disagrees with the render by a millisecond wherever the two roundings differ. Measured on the
standing test's fixture: opened at 280 ms into a `speed: 0.645` element, it reopened on **17 of
27** frames. So:

- the caption (`query::at`) states, beside each offset, where the current pass began: the
  element's `start` and `source_start`, or after a loop wrap the instant that pass began. It
  is read off the same arithmetic as the offset and is not part of any answer;
- `decode::frames_at(origin, first, pace)` opens a run anchored there at timeline frame
  `first`. Its `k`-th frame is the last source frame starting at or before
  `decode::offset_at(origin, pace, first + k)`, which is the render's own offset;
- prediction asks `offset_at`, the function the filter inverts, so the two cannot drift;
- `frames_from` is `frames_at` anchored at `(0, from_ms)`, unchanged for `measure`.

The filter keeps ADR-0127's integer chain and adds two steps. A second `settb` moves each
frame's timeline millisecond onto a clock of `1 / (1000 × fps)` seconds, where it is exactly
`t × fps`; a second `setpts` subtracts `first × 1000`. Timeline frame `first + k` is then
tick `k`, with `fps=:start_time=0` as before. A `start_time` of `first / fps` seconds was
rejected: at most rates it is no whole microsecond, and by our reading of the `fps` filter's
start-time arithmetic (not measured) a microsecond's truncation moves a frame that starts
exactly on a tick onto the next one — at 30 fps, opened at frame 1, the frame at 100 ms onto
tick 4 instead of 3. The integer clock leaves nothing to round.

At every `speed` in the standing test (2, 2.5, 0.5, 0.645, 0.02), from the element's start and
from partway through, one feed and no reopen.

### 3. When a feed ends or fails

- **The feed's `ffmpeg` fails** (non-zero exit, or a frame cut short): it is refused with
  `E-NOT-PAINTED-UNDECODABLE`, naming the source, where the feed opened, the frames delivered
  and `ffmpeg`'s stderr. **Nothing retries through `frame_at`.** That is ADR-0113, which
  ADR-0115 made an invariant over every spawn, applied as it stands.
- **The feed ends cleanly** (`Ok(None)`): the end is accepted only if the asked offset is at or
  past the **probed video-stream end**, less one source frame. That end is `start_time +
  video_stream_ms` (`media/probe.rs`), never the container's duration where the stream states
  its own. One source frame is the longer of the two probed frame periods, rounded up. Where
  the end is accepted, the supplier makes **one** `frame_at` at that offset and keeps it as the
  element's terminal frame; every later offset at or past it gets that frame with no further
  spawn. ADR-0096's at-or-before rule is unchanged, and `frame_at` stays the single authority at
  the edge.
- **New rule: a feed that ends cleanly inside the source is refused**, with
  `E-NOT-PAINTED-UNDECODABLE` naming the source, where the feed opened, the frames delivered,
  the offset it ended at, the stream end and `ffmpeg`'s stderr. A feed that stops early because
  of a bug must fail loudly, never freeze on a frame.

Two readings the implementation had to make:

- **Matroska and WebM state no stream duration**, so for them the end is the container's. It
  is the only end there is; for the VP9-with-alpha sources ADR-0089 exists for, it is the
  only one available.
- **A source whose probe states no end at all** accepts a clean end, and `frame_at` decides:
  it answers the frame at or before the offset, or refuses where there is none.

### 4. The feed budget

Feeds made peak memory one `ffmpeg` per visible video, where `frame_at` was one process at a
time. A 4×4 video wall could swap a 16 GB laptop. Refusing such a document would reject one
that rendered before, and closing and reopening feeds in turns costs as much as per-frame
decoding plus churn. So `render` caps the memory its feeds may use.

**The measurements.** Peak resident memory of a feed-shaped `ffmpeg` (`frames_from`'s command
line, `/usr/bin/time -l`), ffmpeg 9.0.2, the dev's M1 Pro, macOS 27.0, one run each unless
noted. Sources are the benchmark project's lane A, H.264 High 8-bit at 1080x1920 and
2160x3840.

Read as fast as `ffmpeg` writes (two runs each), 1080x1920 source:

| painted at | default threads (~10) | `-threads 2` | `-threads 1` |
|---|---|---|---|
| 576x1024 | 164 MB | 86 MB | 73 MB |
| 1080x1920 | 271–296 MB | 212 MB | 132–157 MB |

and the 2160x3840 source: 431 / 190–203 / 142 MB at 576x1024, and 530–540 / 270–274 /
172–197 MB at 1080x1920.

Read the way a painter reads — one frame, then a 20 ms pause — the maximum of three runs:

| source | painted at | `-threads 1` | `-threads 2` |
|---|---|---|---|
| 1080x1920 | 288x512 | 71 MB | 82 MB |
| 1080x1920 | 576x1024 | 94 MB | 102 MB |
| 1080x1920 | 1080x1920 | 192 MB | 209 MB |
| 1080x1920 | 2160x3840 | 655 MB | 664 MB |
| 2160x3840 | 288x512 | 157 MB | 197 MB |
| 2160x3840 | 576x1024 | 180 MB | 222 MB |
| 2160x3840 | 1080x1920 | 276 MB | 333 MB |
| 2160x3840 | 2160x3840 | 702 MB | 779 MB |

and three 1080x1920 samples for the margin, at `-threads 1`: a 16-reference H.264 with 8
B-frames, 148 MB at 576x1024 and 246 MB at 1080x1920; H.264 High 10, 115 and 216 MB; ProRes
422 10-bit, 114 and 215 MB.

**What they say.** A feed's memory is a straight line in two sizes, not one. The decoder costs
about 14 bytes per **source** pixel on the 8-bit sources and up to about 40 on the
16-reference one. And `ffmpeg`'s pipeline queues about sixteen RGBA frames at the **declared**
size while a slow reader is busy: 62–74 bytes per declared pixel. #636 §3's estimate had only
the two-frame prefetch at the declared size, which would have priced a 4K-painted feed at a
tenth of what it costs. So the estimate gains a declared-pixel term; that is the size tier
#636 §3 asks for when 4K is far off the line. The intercept is about 33 MB.

**The thread count is 1** (`decode::FEED_THREADS`). At one thread a feed uses 45–55% of the
default's memory and still decodes several hundred frames a second against the 30 the
timeline asks for (#636's 800 fps; #624's 150–500). `frame_at` keeps the default threads,
since there one process runs alone. TODO(measure): the rate half of *"the lowest that leaves
clear headroom over the rate the painter needs"* is a wall-clock reading, taken with §8's
before/after; 2 replaces 1 only if that reading says so. The standing byte-equality test runs
at this setting, because it is the only setting a feed has.

**The estimate** (`supply::feed_bytes`) is a pure function of the probe and the document:

```
48 MiB  +  40 B × source pixels  +  80 B × declared pixels  +  2 frames × 4 B × declared pixels
```

The first three terms sit 22–151% above every reading above, the 16-reference sample
included (closest at 4K painted at 4K and on that sample); the fourth is the two-frame prefetch #627's reader thread would hold. A source whose decoded size
the probe did not establish is priced as 4K. The benchmark project's feed (a 1080x1920 source
painted at 576x1024) is 185 MB; a 1080p source painted at its own size is 316 MB; the 2160p30
variant's feed (2160x3840 at 1152x2048) is 590 MB. Sources not measured here — interlaced,
high-bit-depth 4K, very long reference chains at 4K — are outside what the margin was sized
against.

**The budget is 2 GiB per painter** (`supply::FEED_BUDGET`). It covers feed processes and their
prefetch, never the painter's canvas or the encoder. It is a constant, never a fraction of
physical memory, so the split between feeds and per-frame decodes, the answer and the spawn
count are the same on every machine. A `#[doc(hidden)]` override lets tests shrink it; there is
no public setting, and raising it takes a superseding ADR. **Why it is safe on an 8 GB machine
for one painter:** it is a quarter of the machine, the estimate it is counted in sits 22–151%
above every measured peak, and what remains is the OS, `montagent`'s own canvas (8 MB at 1080p,
33 MB at 4K) and the encoder. If time chunks are ever built (#627 step 3), each worker's private
memory includes its feeds at this estimate, their prefetch and one transient `frame_at`, and its
K-sizing carries this guarantee.

**Admission.** An element without a feed asks for one when it is painted, in paint order. It
gets one if its estimate fits the bytes not yet held — **first-fit**, so a smaller source later
in paint order can take room a larger one could not. Otherwise it is decoded per frame through
`frame_at` for that frame and asks again on the next.

- **Holders keep their feed** until the element stops being painted. A feed is never taken
  away.
- **A reopen keeps its slot.** A loop wrap or a skip is the same holder reopening; only leaving
  frees its bytes.
- **Same-frame release.** `begin_frame(n)` closes the feeds not asked for at `n − 1` before
  anything asks at `n`, so bytes freed at frame `n` are there for askers at `n`. Because a feed
  is closed the frame *after* its element's last, an element entering at a cut waits one frame
  where the budget only fits the outgoing and incoming feeds apart. The 2160p30 variant is
  that case: four of its 590 MB feeds are 2.36 GB, so at each cut the incoming element is
  decoded per frame for one frame and gets a feed on the next. The 1080p30 benchmark project
  (four feeds, 0.74 GB) is not.
- **Promotion.** An element decoded per frame gets a feed at the first frame one fits, opened
  at that frame's offset under the open rule. An element that leaves and re-enters goes through
  admission again.
- The check is two additions and a comparison, with no allocation.

**Past the budget is a planned decision, not a fallback: it is taken when a feed would open,
never in reaction to a failure, and a feed that fails is still refused and never retried
through `frame_at`.** The pixels are the same (§7).

**The answer.** `render`'s answer gains `decoded_per_frame`: the ids of elements decoded per
frame because the budget was full, in the order first so decoded. It is **always present**,
`[]` when empty, like `not_painted` and `painted_partially`. Such an element was still painted
and is in `painted` too. The budget is the only reason an element appears there; a failed feed
is refused, never listed. It is not a finding, because findings describe the picture and the
picture is unchanged. `preview` inherits it through `encode_span`. No `CONTEXT.md` term:
"feed budget" is implementation, not the format's language.

One more element is decoded per frame, unlisted: a `speed` with more than six decimal places,
which ADR-0127 §2 refuses to sample in a run. It is painted per frame as `frame_at` always
painted it, and it is not a budget decision, so `decoded_per_frame` does not name it.

### 5. `preview`

`preview` paints through the feed supplier, inherited through `render::encode_span`. `Span`
has no supplier field, so `preview` and `render` still differ only in the surface and the
clock. Per-frame decoding is spawn-bound, not pixel-bound, so a 540p rung could never have
rescued it. Feeds decode at the element's declared size and the proxy scale is applied on the
canvas afterwards, so the rasters are `frame_at`'s. **Each attempt opens fresh feeds**: each
`encode_span` call builds its own painter, so the feeds of a missed 720p attempt die with it
and reopen lazily at the span's first frame on the 540p retry. **Abandon and cancel close feeds
by `Drop` only**: abandoning returns `Stop` out of `encode_span`, which drops the painter, the
supplier and every feed. There is no `close_all()`, which would be a second path that could
drift from `Drop`. The rungs and floors (ADR-0021, ADR-0067) are unchanged; they were measured
on fixtures without video.

### 6. Hardware decode

**Feeds decode in software.** `-hwaccel videotoolbox`, and its equivalents elsewhere, was
considered and rejected (#637). A feed is capped by the RGBA pipe at about 477 fps, and
VideoToolbox decodes at 200–260 fps against software's 150–500, so it saves only about 0.6
core per feed. Without a `format=yuv420p` prefix its pixels differed on 540 of 540 frames, and
the prefix was proven only on 8-bit H.264 and HEVC (#624). The feed budget's estimates and the
pinned `-threads` assume software decode. **The reopen trigger:** every built paint rung (#627)
and the encoder ladder (#628) have been tried, the benchmark project still misses, and the
per-stage breakdown (§9) blames CPU contention from feed decode. Then a new issue is opened,
with its own ADR, never a rung in this map. Before anything is built it measures byte parity
against `frame_at` (10-bit sources and the ffmpeg 7.1 floor included), how to detect a silent
fall back to software, hardware memory per feed, how many sessions can run at once, and the
other targets.

### 7. Evidence that keeps `frame` and `render` the same picture

- **`tests/seek_clamp.rs` stays** (ADR-0127): both paths pick the same source frames.
- **`tests/feeds.rs`, the standing byte-equality test.** It paints a short span through the
  feed supplier and through `frame`'s per-frame supplier (`render::paint_span`,
  `#[doc(hidden)]`), and compares the **rasters before encoding**, byte for byte. Its cases are
  #626 §5's: a hold; a loop wrap; leaving and re-entering; a partial `from`/`to` starting
  partway through an element; `speed` 2, 2.5, 0.5, 0.645 and 0.02; `fps` 24, 30 and 60 over a
  source whose frames start at 42 ms; two elements on one source at different offsets; holding
  past the source's end; an offset exactly at the probed end; and a VP9-with-alpha source
  through `libvpx-vp9`. It names the codecs it covers: H.264 (long GOP, B-frames), FFV1 in
  Matroska, VP9 with alpha.
- **Spawn counts**, through a `#[doc(hidden)]` per-thread counter (`supply::counts`), in every
  case: one element over 60 frames opens 1 feed, a loop opens 1 and reopens once per wrap, a
  hold takes one terminal `frame_at`. The counter is not on `render`'s answer, since that public
  API is the speed ticket's to shape.
- **#636 §7's seven budget cases**, through the budget override: a mixed span's rasters equal
  an all-per-frame span's; exactly 1 feed plus one `frame_at` per frame for the second element;
  promotion once the holder leaves, at the frame `begin_frame` frees its bytes; the render
  answer's `decoded_per_frame` names the second element and is `[]` otherwise; mixed sizes,
  where a budget that fits the small source but not the large one gives the large one no feed
  even when it asks first; same-frame release; and a holder that loops keeping its slot while
  another element waits, with spawns equal to feeds plus reopens. Plus a pure unit test of the
  estimate against every measured peak and of first-fit admission.
- **`tests/preview.rs`**: a `preview` over one video element opens 1 feed, and the open-feed
  count is 0 after an abandoned span and after a 720p→540p degrade (one feed per attempt).
- **`crates/montagent/tests/render_spawns.rs`**, #625 §4's CI test: the shipped binary with
  `ffmpeg` and `ffprobe` shimmed on `PATH`. The decode spawns are exactly the feeds plus their
  reopens, with no `frame_at`, and doubling the frames adds no spawn of any kind.

Mutation-checked: dropping the origin from the filter fails five of `feeds.rs`'s cases; measuring
feeds from their open frame fails the speed case (17 reopens); checking the hold first fails it
too (20 reopens).

### 8. The measurements

TODO(measure): fill from `render_target.rs`'s JSON line (ADR-0142's protocol), then set
`status: accepted`. The benchmark project's measurement records `decoded_per_frame: []`.

| | commit | median | min–max | peak memory | spawns | `decoded_per_frame` |
|---|---|---|---|---|---|---|
| before (base commit) | TODO(measure) | | | | | — |
| after (this ADR's head) | TODO(measure) | | | | | |

The chosen feed `-threads`: TODO(measure) — 1 on memory (§4), confirmed or raised by the rate
reading.

**`preview`** — one observed wall time over a video span, before and after, recorded here and
not in `budget.rs`, so it never reads as a ceiling: TODO(measure).

### 9. Producing a frame, apart from pushing it

`encode_span` now produces frame `n` (paint, then read back off the canvas) in one step and
pushes it to the encoder in the next (#627 §7), so a writer thread can take the second without
reshaping the loop. It adds no behaviour.

The protocol's per-stage breakdown comes from the same split. With `MONTAGENT_STAGES` set
(`render::STAGES_VAR`, `#[doc(hidden)]`), a span prints one line on stderr after its seal:
cumulative decode, paint (decode taken out), readback, encode-wait and seal milliseconds, and
the feed counts. `render_target.rs` records it with every run. It is measurement plumbing and
deliberately not an answer field. Unset, it costs four clock reads a frame.

## What this cites without amending

- **[ADR-0096](0096-the-frame-at-an-instant-is-the-last-one-starting-at-or-before-it.md).** The
  frame at an instant is still the last one starting at or before it, and the frame past the
  source's end is still `frame_at`'s (§3).
- **[ADR-0113](0113-a-seek-whose-ffmpeg-failed-is-refused-never-read-as-no-frame.md) and
  [ADR-0115](0115-ffmpeg-7-1-with-libx264-is-the-floor-and-a-tool-qualification-finds-out.md).**
  A failed feed is a refusal, never an empty answer, and is never retried.

**This amends ADR-0127.** Its *"run of frames"* is now a **feed** where `render` uses it, and the
run supplies `render` as well as `measure`. A run can open partway through an element, anchored
at the element's own origin (§2). `tests/feeds.rs` joins `seek_clamp.rs` as its evidence.

## Consequences

- `render` and `preview` spawn one `ffmpeg` per visible `video` element plus one per reopen,
  where they spawned one per frame per element.
- `frame`'s pixels, findings and spawns are unchanged.
- `render`'s answer has a new, always-present field, `decoded_per_frame`.
- Feed memory is bounded by an estimate, the same on every machine; a video wall past it renders
  as before, per frame, and says which elements.
- A feed that ends early or fails is refused loudly, never painted from a frozen frame.
