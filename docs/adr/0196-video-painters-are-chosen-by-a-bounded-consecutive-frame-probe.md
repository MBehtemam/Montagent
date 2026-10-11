---
status: accepted
amends: 0144 (§7's one-painter-with-video rule only)
---

# Video painters are chosen by a bounded consecutive-frame probe

[#933](https://github.com/MBehtemam/Montagent/issues/933), implementing the owner-adopted
[#927](https://github.com/MBehtemam/Montagent/issues/927) design after the
[#926](https://github.com/MBehtemam/Montagent/issues/926) reopen research.

## Context and measurement exception

ADR-0144 §7 kept one painter whenever the document contained a video. A paint-heavy
video document can profit from parallel painting, but reopening its feeds at every
paint chunk can erase the gain. The two-frame default is particularly expensive.

**The owner explicitly authorised measurements under load on 2026-10-11**, replacing
#933's quiet-machine prerequisite for this implementation. These are interleaved
observations on an M1 Pro, not quiet-machine capacity guarantees. The committed
[measurements and harnesses](../research/video-painters/README.md) record load and bytes.
A quiet run remains reproducible through the ignored gate; this ADR does not claim one.

The prefix cache in ADR-0195 changes the original reproduction: its one-painter
240-frame slice now takes a median 4.52 seconds, versus 3.63 seconds at forced K=6,
C=15 and 8.50 seconds at C=2. Automatic scheduling must measure current paint cost,
not reuse the old uncached kit's approximately 120 ms/frame.

## Decision

1. **Count the peak video presence in the requested span.** A sweep of half-open
   `start`/`end` intervals yields N, independently of probe samples. Transparent,
   occluded or off-screen videos are conservatively counted. Videos outside the span
   cannot force it to one painter. This is peak simultaneous presence, not the total
   number of video elements: the decode benchmark has 20 elements but at most three
   simultaneously present. Meeting endpoints do not overlap.
2. **Apply a core-derived process cap first.** K starts at cores minus ADR-0144's four
   encoder cores. Set the feed-process cap to twice the available cores and reduce K
   to `min(K, floor(cap/N))`. K below two keeps one painter without probing. On the
   ten-core M1 Pro, this caps simultaneous painter feeds at 20, beside the encoder
   and any audio processes. It is a conservative policy, not a measured universal
   optimum. The four-video sweep observed 21 ffmpeg processes with K=5 (about
   1,250–1,329 MiB RSS), and 25 with K=6 (about 1,436–1,466 MiB). Feed admission's existing
   per-painter memory budget still applies. This cap admits parallel painting; a
   one-painter exit retains ADR-0141's feed admission even when N itself exceeds the cap.
3. **Probe before output painting or encoding.** On the verb thread, paint three
   four-frame consecutive runs, at the beginning, middle and end of the requested
   span. Each has a fresh canvas and feed supplier. Exclude each run's first frame
   from timing and take the median of the nine later frame durations (decode, paint,
   presence resolution and readback included). Discard all probe pixels, records,
   suppliers and prefix composites. Single evenly spaced frames are rejected: their
   reopen cost is not steady paint cost. Probe painting is excluded from the output
   profile; it encodes no frame and advances no output progress count.
4. **Bound probe work.** A four-second wall budget is checked between and after frames;
   one in-flight frame or feed teardown can overrun it. An over-budget or zero-duration
   probe keeps one painter. Cancellation and preview's deadline are checked between
   samples. A span with fewer than K times the minimum two-frame paint chunk skips
   the probe. The four-second policy lets a few ordinary opens fit while bounding the
   work spent on documents with expensive seeks. A measured two-second pilot exhausted
   its budget on the original uncached paint-heavy kit and added 2.08 seconds without
   enabling painters, so this policy allows four seconds. It is not an asserted latency bound
   for a single ffmpeg request.
5. **Amortise reopening before granting parallelism.** The fixed contended allowance
   R is **1.1 seconds per feed**, rounding above the newly measured worst GOP-phase
   median (1.004 seconds; 1.018 seconds worst sample). With measured p seconds/frame:
   `C = max(2, ceil(N*R/(0.5*p)))`. Keep one painter if the span is shorter than K*C,
   K*C exceeds twice the existing byte-window capacity, or predicted speedup
   `K/(1+N*R/(C*p))` is below 1.5. These are conservative scheduling policies, not
   new render speed targets. The normal window floor K*C and its disclosure remain.
6. **Disclose the choice.** Automatic video schedules add `feed_cap` and
   `peak_video_feeds` to `render.painting`, including one-painter exits; the existing
   `painters`, `chunk`, `window` and `window_floor` disclose effective K and C.
   Video-free spans and forced measurement schedules retain the old answer shape.
   Internal measurement plumbing accepts `MONTAGENT_PAINTING=one` for a sequential
   baseline without the probe, beside existing `K,C[,W]`. No project field, CLI flag
   or MCP argument chooses scheduling.
7. **Same pixels, one existing paint class.** ADR-0144's byte gate and ADR-0190's
   per-target MP4 guarantees stand. Feed reopen/seek behaviour is unchanged; held
   feeds, discard-forward serving and keyframe-aligned paint chunk starts are deferred.
   Prefix sharing still applies among output painters, never from probe to output.

## Verification and timing

`crates/montagent-core/tests/painters.rs` exercises probe-selected scheduling, process
cap reduction, short-span/zero-time/budget/core exits, half-open peak presence, ragged
paint chunks and whole-render identity against one painter and forced C=2. Its clock
and core overrides mock machine boundaries; the actual probe still paints its samples.

`crates/montagent-core/tests/feeds.rs` compares every frame at every tested paint chunk
start against `frame_at` for H.264, fractional speed, interlaced TFF/BFF, full-range and
BT.709-tagged full-range, VP9 alpha (including fast playback), and HEVC 10-bit. It also
compares encoder-input hashes and whole-render MP4 bytes at probe-selected and forced
schedules. Optional VP9/HEVC encoders are skipped only if unavailable; H.264 is required.

`video_painters_do_not_regress_the_decode_bound_benchmark` in
`crates/montagent/tests/render_target.rs` is an ignored one-sided 10% no-regression gate.
It uses one discarded pair then five timed pairs over all 10,800 benchmark frames,
alternating sequential/automatic order, checking load before every run, with fresh
sidecars. Each output's frame hashes must equal the sequential reference. Speedup is
reported, never required. By default it requires a quiet M1 Pro on AC; the explicit
`MONTAGENT_VIDEO_PAINTERS_ALLOW_LOAD=1` measurement exception is recorded in every row.

The measurement record and final comparison are in
[the evidence directory](../research/video-painters/README.md). The complete checker passes:
the full decode comparison's medians are 139.556 seconds sequential and 143.344 seconds
automatic (2.7% overhead, below 10%), with every frame hash equal. The uncached paint-heavy
kit's medians are 33.388 versus 14.871 seconds (2.25× observed speedup), with exact frame
and MP4 digests. The full suite passes in a checkout of tracked files plus this change;
local untracked generic skills are excluded from that verification. A quiet-machine run
is still owed; acceptance uses the owner's explicit load exception.
