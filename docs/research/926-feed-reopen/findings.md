# #926: what a feed reopen costs, and what chunking makes painters pay off on a video document

Research for wayfinder map #913, ticket #926. Findings only; no product code lands. Question: ADR-0144 §7 keeps
one painter for any document with a `video` element, and made a measured reopen cost the precondition for lifting
that. This measures it on HEAD (`51aa4b52`, 0.2.0, release build, private `CARGO_TARGET_DIR`).

## Headline

1. **A feed reopen is not a flat ~100 ms. It is ~50-80 ms plus the decode from the source's previous keyframe
   up to the chunk's first frame.** Measured with `decode::frames_at`'s own command line: 48 ms at a keyframe and
   475 ms just before the next one on a 720p GOP-250 source (mean about 250-285 ms); 82 ms and 890 ms on the
   benchmark project's 1080x1920 GOP-300 source (mean about 480 ms uncontended, about 870 ms per open measured
   inside a real render at load 11-21). The ADR's "~100 ms" is the keyframe-aligned best case.
2. **On a paint-heavy document with videos, painters pay off from C = 15 frames** (kit: 0.121 s/frame on one painter
   to 0.030-0.046 s/frame at K=6, C=15, for one to four visible videos: 2.7x to 4x), and the gain flattens or
   shrinks beyond that. **C = 2 (the paint-chunk default) is a loss or a tie from two videos up**
   (N=4: 0.114-0.129 against 0.124-0.127 on one painter, with 144 CPU-s spent reopening).
3. **On the ADR-0142 decode-bound project painters never win.** One painter: 136-149 s. Forced K=6: C=30 195.8 and
   201.9 s (slower), C=60 153.2 and 163.1 s, C=120 136.9 s (a tie, with a 720-frame / 4.5 GB window floor). The
   project is paint+decode+readback serial work of about 121 s on one thread against an encoder floor of
   about 88 s; painters can at most win about a quarter, and the reopen tax eats it. **§7's one-painter rule is
   right for that class.**
4. **Feeds held per painter across chunks** are possible under ADR-0141 only by amending its serve order (a new
   "ahead on the same pass: read forward and discard" step, today the skip goes to step 3 and reopens). They win
   only while the skip is shorter than the reopen, which is (K-1)·C below roughly 150 frames, and they decode every
   source K times over. **Not a substitute for larger chunks on the decode-bound class, a modest win for
   small C on the paint class.** Details in Q3.
5. **Byte identity held everywhere it was tried**, including sources ADR-0141 did not cover (Q4).

Method caveat, once: the machine was never quiet. Every number below carries the 1-min load at the start; the
protocol's < 1.5 was reached on none of the runs (lowest 3.5, at the first benchmark run of the second session).
Absolute seconds are upper bounds. The conclusions rest on interleaved relative comparisons, the stable
per-open cost, and the shapes of the curves.

## Setup

- Machine: Apple M1 Pro, 10 cores, ffmpeg 9.0.2, macOS. Build: `cargo build --release -p montagent`.
- Kit: `docs/research/long-form-feedback/montagent-repro/` from `origin/claude/confident-clarke-r5iqm5` (not
  copied here). The missing 12 MB mp4 stood in with two synthetic sources: `testsrc2` 1280x720 30 fps 12 s,
  x264 default (keyint 250), and the same with `-g 300 -bf 3 -sc_threshold 0` (the benchmark's GOP shape). The
  kit has one video element (declared 688x387); **N visible videos = N clones of that element, one feed each**, at
  640x360. The slice is `--from 1500000`, 240 frames (8 s), no loop wrap inside it. The kit is paint-heavy:
  0.121 s/frame on one painter, 0.022-0.025 with the video removed and 6 painters (the floor of the rest).
  Synthetic sources decode faster than camera footage, so reopens on real footage cost more, not less.
- Decode-bound project: `fixtures/benchmark/make_benchmark.py --only-1080`, the 1080p30 six-minute project
  (three lanes, 20 video elements, 10,800 frames, 30,600 element-frames, audio mix included).
- Forced painters: `MONTAGENT_PAINTING=K,C[,W]`; per-stage times from `MONTAGENT_STAGES=1`
  (`decode_ms`, `feeds_opened`, `feeds_reopened`); ffmpeg count and RSS sampled with `ps` every 0.2-0.5 s.
- Scripts and raw data in this directory: `reopen.py` (Q1), `sweep.py` (Q2), `bench.py` (Q5), `feeds_sources.rs.txt`
  (Q4, a throwaway integration test that sat beside `tests/feeds.rs` and was never committed as a test),
  `data/*.jsonl`.

## Q1. What one reopen costs (spawn + seek + first frame)

`reopen.py` runs `frames_at`'s exact command (`-threads 1 -ss <from-200ms> -copyts`, the `settb/setpts/fps/scale`
chain, rgba pipe) and times it to the first complete frame, at five positions in the GOP, 5 runs each, and
again with 6 opened at once. Load 9-12 during these.

| source | at keyframe | 25% in | 50% | 75% | just before next key | mean over GOP | steady state after |
|---|---|---|---|---|---|---|---|
| 720p GOP 250, painted 688x387 | 49 ms | 147 | 258 | 368 | 475 | about 250 | 590 fps |
| 720p GOP 300 (B=3) | 47 | 175 | 304 | 437 | 538 | about 280 | 604 fps |
| benchmark lane, 1080x1920 GOP 300, painted 576x1024 | 82 | 286 | 480 | 675 | 890 | about 480 | 356 fps |
| same lane, 6 reopened at once | 133 | 363 | 564 | 755 | 979 | about 560 | 156 fps each (6 in parallel) |

So the cost is about **50-80 ms of spawn and container open plus 1.7 ms (720p) to 3.4 ms (1080x1920) per frame
of GOP distance**: ffmpeg's accurate input seek lands on the keyframe before the window and decodes forward at one
thread (`FEED_THREADS` = 1) to the frame. The 200 ms seek window adds a few frames. Six reopens at once
(six painters starting chunks together) cost about 20% more per open.

Inside real renders, `decode_ms` over `feeds_opened + feeds_reopened` gives the all-in cost per open:
**263-324 ms** on the kit's GOP-250 source (240-frame slices at load 5-29; least squares over all configs
285 ms/open plus 0.2 ms/frame), 278 ms/open plus 1.4 ms/frame for the GOP-300 source, and **871 ms and 913 ms** per
open on the benchmark project at load 11-25 (1019 opens, 888 s and 930 s; K=6 C=60 gives about 950 ms, C=120 about 1020 ms).
A reopen is the cost of decoding roughly 100-170 source frames, and a chunk of C frames pays it once per visible video.

Chunk starts do not land on keyframes, so the mean is the figure to plan with. **Aligning a chunk's first frame to the
keyframe of the most expensive visible source would cut that source's reopen to the 50-80 ms floor** (5-6x), but
sources of different phases cannot all be aligned, and a loop wrap or a cut puts the open somewhere else.

## Q2. s/frame against chunk size, and where K and memory become limits

`sweep.py`: kit, 240 frames, K=6, each cell run twice (loads 5-41, shown as min-max of the two runs, s/frame).
`one` is the shipped behaviour (one painter) and is flat at 0.121-0.127 for every N, which says that on this kit the
video is not the cost: paint is.

| N visible videos | one painter | C=2 | C=5 | C=15 | C=30 | C=60 |
|---|---|---|---|---|---|---|
| 0 (control, no video; 6 painters) | 0.022-0.024 | 0.022-0.028 | 0.023-0.025 | 0.025-0.026 | 0.032-0.035 | 0.035-0.044 |
| 1 | 0.121-0.122 | 0.051-0.052 | 0.035-0.036 | **0.030-0.033** | 0.035-0.037 | 0.037-0.038 |
| 2 | 0.122-0.124 | 0.071-0.079 | 0.044-0.047 | **0.034-0.035** | 0.038 | 0.038 |
| 4 | 0.124-0.127 | 0.114-0.129 | 0.062-0.071 | **0.041-0.046** | 0.043-0.044 | 0.041-0.043 |
| opens + reopens at N=4 | 4 | 480 | 192 | 64 | 32 | 16 |

(GOP-300 source, one run: N=2 one 0.123, C=5 0.044, 15 0.035, 30 0.037; N=4 one 0.126, 5 0.065, 15 0.044, 30 0.046.)

Reading it:

- **The curve has an elbow at C = 15 and a floor at 0.03-0.045.** Below it the reopen dominates (C=2, N=4 is a tie
  with one painter and burns 6x the CPU: 192 s user against 40 s on `/usr/bin/time`); above it the window of
  240 frames cannot keep 6 painters busy (C=30 is 8 chunks, C=60 is 4), so the right-hand rise on the N=0 control
  is the slice's tail, not the reopen. A longer span moves the right-hand side but not the elbow.
- **The model that fits:** speedup is about K / (1 + N·R / (C·p)) with R the reopen cost (0.285 s here) and p the
  paint time per frame (0.12 s here), capped by the encoder and the rest at the control's 0.022 s/frame.
  A chunk's reopen overhead is N·R/(C·p): 0.63 at N=4, C=15 on this kit. **The chunk rule that follows is C ≥ about
  N·R/(0.5·p) for a 33% overhead cap**, which is why decode-bound projects (p about 6.6 ms) land in the hundreds
  (Q5) and this kit (p = 120 ms) in the tens.
- **Process count is K·N feeds plus one encoder, and a little more** (sampled peaks: 7, 13, 25 `ffmpeg` at K=6 and
  N=1, 2, 4; 27 on the benchmark at K=6 with 3 lanes, where cuts briefly overlap an outgoing and an incoming feed).
  A chunk start kills the old feed before the new one opens, so reopens do not add to the peak.
- **Memory.** Peak ffmpeg RSS summed over processes: 0.53-0.86 GB at N=1, 0.7-1.2 GB at N=2, 1.1-1.8 GB at N=4
  (640x360 painted, 720p source), about 60-75 MB per feed. On the benchmark lanes (1080x1920 painted at
  576x1024): 0.86-1.1 GB with one painter (9 processes), **2.0-2.5 GB at K=6** (27 processes, 1.9-2.1 GB at K=4-6, C=60-120).
  `montagent`'s own RSS on the kit at N=4: 0.49 GB on one painter, 0.80 GB at K=6, C=15, 1.16 GB at C=60.
  The reorder window's floor is K·C frames of 1080p RGB, 6.2 MB each: **1.1 GB at K=6, C=30; 2.2 GB at C=60;
  4.5 GB at C=120**. ADR-0141's per-painter 2 GiB feed budget times K=6 would allow 12 GiB of feeds; measured
  feed RSS is a fifth of that, so the process count, not the feed budget, is the binding cap.
- **Where K stops helping** is the CPU, not memory: 18 parallel feeds each reading a full-rate stream fall to
  52 fps (6: 156 fps; 1: 338 fps) at load 10-13 on this 10-core machine, which still clears the 30 fps the timeline
  needs per feed, but the encoder is competing for the same cores. ADR-0144's "four cores for the encoder" rule
  already leaves K=6; the video cap should be on K·N, a figure around 16-24 feeds on this machine.

## Q3. Feeds held per painter across chunks, versus larger chunks alone (read, not prototyped)

What the code does today (`crates/montagent-core/src/verbs/render/painters.rs` `work`, `.../frame/supply.rs`
`Feeds::supply`): a painter is created once per thread and keeps its `Feeds` across all its chunks. At a new chunk the
feed is still held, but the request is for frame n while the feed's `next` is the frame after its last one, so
serve step 1 fails, step 2 (hold) fails, and **step 3 ("anything else: a loop wrap, a skip") drops the feed and
reopens it.** So the holder is already there; the missing piece is the serve rule.

**Feasible under ADR-0141's feed rules, with one amendment.** A feed's k-th frame is defined by its origin and
`decode::offset_at` (ADR-0141 §2), not by where it opened, which is what `a_feed_opened_partway_is_the_run_the_render_paints_from_there`
asserts. So reading forward and discarding G frames lands on exactly the frame a reopen at that offset would deliver:
byte identity is the same argument as the existing partial-range case, and the feed budget, `decoded_per_frame` and the
spawn-count tests (opens + reopens = feeds + wraps) need only the new case. The amendment: **serve step 2b, "the
request is for a frame ahead of the feed's next on the same pass (the offset formula agrees), by at most a cap: read
and discard up to it".** A loop wrap still reopens; so does a skip beyond the cap.

**Its cost against a reopen.** With dispatch in timeline order and C-frame chunks, painter p's next chunk starts
(K-1)·C frames after its last one. Reading and discarding costs the pipe's rate: 3 ms a frame (lane, one feed at
340 fps), 1.7 ms (720p). Against a reopen's mean of 250-480 ms that is cheaper while (K-1)·C is below about
85-160 frames: **C up to about 17-32 at K=6**. Beyond that a reopen is cheaper, so the held feed is a rule for small C only.
It also makes **every painter decode every source in full**, K times the decode CPU of one painter: on the benchmark
that is 6 x 30,600 frames at 3 ms = 550 CPU-s, against 888 s measured for reopens at C=30 and 90 s for the one painter.

**Larger chunks alone** need nothing but a chunk rule and pay in window memory (K·C·6.2 MB: C=60 is 2.2 GB, C=120 is
4.5 GB at K=6) and in tail/utilisation (a span shorter than K·C leaves painters idle, as the 240-frame slice shows at C=30 and 60).

| | small C (<= about 30) | large C (>= 60) |
|---|---|---|
| larger chunks alone | reopen tax N·R/(C·p), measured 0.5-1.5 at paint-heavy, far more at decode-bound | amortised, window 2-4.5 GB, tail |
| held feeds + discard-forward | one decode of every source per painter, no reopen, window small | skip longer than a reopen, no benefit |

**The pair is complementary at the paint-heavy end, not at the decode-bound end:** for a paint-heavy document with
N=2-4 videos the kit reaches 0.034-0.045 s/frame at C=15 by chunk size alone; held feeds would let C=5 reach about the
same without a bigger window. At the decode-bound end neither wins (Q5). **Held feeds are not needed to meet the
precondition; they are an optimisation to price after the rule lands.** A third option, **keyframe-aligned chunk starts**,
cuts R by 5-6x for the aligned source (Q1) and costs nothing in memory; it can be tried with the chunk rule.

## Q4. Byte identity for seek-based feeds

Three layers, all equal:

1. **Whole renders, kit slice (90 frames from 1500000, one video), forced `MONTAGENT_PAINTING` = 3,2 / 6,7 / 6,15 / 2,45 against
   one painter:** the MP4 bytes are identical (sha256 `efd020381af8731a...`) and so is the decoded `framemd5` of
   every frame.
2. **Whole renders, benchmark project (10,800 frames, 3 lanes, audio mix), one painter against K=6 C=30, K=6 C=60,
   K=6 C=120, K=4 C=60:** all ten runs of Q5 wrote the same MP4 (sha256 `9113603a80381a13...`), hence the same frame hashes.
3. **Raster-level, at every chunk start, `render::paint_span` with `Supplying::Feeds` against `PerFrame`** (the
   `tests/feeds.rs` method, `feeds_sources.rs.txt`): a 82-frame 25 fps source looped inside a 4 s element, painted
   once per frame as truth and then in chunks of C = 1, 2, 7 and 25 frames (169 fresh painters per source, each chunk start
   a seek-based open), every raster byte-equal. **169 of 169 chunks passed for each of nine cases (seven sources, two with a second `speed`):**

| source | what it adds beyond ADR-0141's gate list |
|---|---|
| H.264 long GOP with B-frames, loop | control, speed 1 and 0.645 |
| **H.264 interlaced, top field first** | ffprobe: `field_order tt`, 81 of 82 frames flagged interlaced |
| **H.264 interlaced, bottom field first** | same, bff |
| **H.264 full range** (`yuvj420p`, `-color_range pc`) | ffprobe: `color_range pc` |
| H.264 full range, bt709-tagged | colourspace tags on top |
| **VP9 with alpha** (`libvpx-vp9`, `yuva420p`) | loop, and speed 2.5 as a second case |
| **HEVC 10-bit** (`yuv420p10le`) | |

Limits: sources are 64x48 synthetic `testsrc2` (cheap, deterministic; real footage's GOP structure is the benchmark's, which was
covered at whole-render level); the raster test uses fresh painters per chunk, so a held feed discarding forward (Q3) is not covered
and would need its own case; frames are compared before encoding, so the encoder cannot hide a difference.

## Q5. The ADR-0142 decode-bound project, painters on or off

Whole project (10,800 frames, three visible lanes, audio mix), cold probe cache, `MONTAGENT_STAGES=1`. Two sessions, interleaved. **Load is the
1-min average at each run's start**, never under the protocol's 1.5.

| run | cfg | load at start | wall | opens + reopens | decode_ms (summed, all painters) | window floor | peak ffmpeg (procs, RSS) |
|---|---|---|---|---|---|---|---|
| A1 | one painter | 9.6 | 144.2 s | 20 + 0 | 50,474 | 1 | 9, 1,115 MB |
| A2 | K=6 C=30 | 10.9 | **195.8 s** | 120 + 899 | 887,962 | 180 fr = 1.1 GB | 27, 2,420 MB |
| A3 | one painter | 21.2 | 149.4 s | 20 + 0 | 55,765 | 1 | 9, 1,088 MB |
| A4 | K=6 C=30 | 14.6 | **201.9 s** | 120 + 899 | 930,117 | 180 fr | 27, 2,519 MB |
| A5 | K=6 C=60 | 24.6 | 163.1 s | 120 + 398 | 490,433 | 360 fr = 2.2 GB | 26, 2,348 MB |
| A6 | K=4 C=60 | 24.8 | 148.9 s | 80 + 438 | 418,358 | 240 fr = 1.5 GB | 19, 1,980 MB |
| B1 | one painter | 3.5 | **136.3 s** | 20 + 0 | 45,140 | 1 | 5, 857 MB |
| B2 | K=6 C=60 | 9.8 | 153.2 s | 120 + 398 | 458,696 | 360 fr | 24, 2,167 MB |
| B3 | K=6 C=120 | 16.5 | **136.9 s** | 119 + 148 | 272,750 | 720 fr = 4.5 GB | 23, 2,097 MB |
| B4 | one painter | 17.3 | 141.8 s | 20 + 0 | 49,441 | 1 | 8, 1,132 MB |

All ten MP4s are byte-identical (`9113603a80381a13...`).

- **One painter: 136.3-149.4 s** (the ADR's own observation was 135.4 s). Its stages sum to the wall: decode wait
  45-56 s, paint 57 s, readback 13 s, encode wait 20 s.
- **Painters on: never faster.** C=30 is 36-47% slower; C=60 is 8-12% slower (K=6) or a tie (K=4); C=120 ties, and
  needs a 4.5 GB window floor. The cost is the reopen: 399-1019 opens at 0.5-1 s each is 270-930 CPU-s, against 90 CPU-s for
  decode on one painter.
- **Why there is no headroom:** the encoder alone takes about **88 s** for this output (`ffmpeg -i out.mp4 -c:v libx264 -preset
  medium -crf 20 -threads 5 -f null -`, 421% CPU, load 8; re-encoding an already-compressed picture, so a floor, not the
  render's own encoder time). The most any number of painters could save from 136 s is about 40 s, and the reopen tax is larger than that
  at any C a memory budget allows. The paint class's per-frame paint is 6.6 ms here (kit: 120 ms), which is the number that decides it.
- **The ADR's gate stays satisfied:** with the rule "decode-bound documents keep one painter" no regression is possible; the
  timed no-regression gate with painters on would measure the opposite (a regression) at every C tried.

## What the design ticket should decide

1. **The trigger is paint cost per frame, not "has a `video` element".** The kit (p about 120 ms/frame) gains 2.7-4x; the
   benchmark (p about 6.6 ms/frame) loses or ties. The rule needs p. Options: (a) probe, painting the first M frames on one
   painter and reading `paint_ms/frames` (the stage clock exists; the decision changes no byte, so a late switch is safe),
   or (b) a static estimate from the resolved stack (blend, blur, mask, text counts) that ADR-0144 §8 rejected for dispatch order.
   Painters on iff `p·C >= 2·N·R` at the C the window allows, with R about 0.3 s (720p) to 0.5 s (1080 lane) planned at the keyframe-distance mean.
2. **Chunk rule for video documents:** C = ceil(N·R / (0.5·p)) clamped to [15, floor from the window], and **at most
   K·C·frame_bytes inside `WINDOW_BYTES`**; C=2 stays for no-video documents. On the kit that gives C = 15-17 (N=1-4 at R=0.3 s, p=0.12 s;
   for N=4 the measured optimum C=15-60 is flat). Do not let C exceed span/K, or the tail idles painters.
3. **Process cap:** K·N (+1 encoder) `ffmpeg` processes. Measured limit around 16-24 feeds on this machine; propose `K = min(K,
   floor(24 / N))` and `K >= 2` or one painter; note transient cut overlap of up to one extra feed per painter. Memory is not the
   binding cap (feed RSS 60-190 MB each against a 2 GiB per-painter budget): the window floor K·C·6.2 MB is the
   bigger memory term, and should stay inside `WINDOW_BYTES` (K·C <= 150 at 1080p).
4. **Held feeds (serve step 2b) are an optional later optimisation,** to be priced after (1)-(3): worth it only for small C (K-1)·C < about 100
   frames, costs K-fold decode CPU, needs its own byte-equality case (read-forward-and-discard against `PerFrame`) and an ADR-0141 amendment.
   Keyframe-aligned chunk starts are the cheaper experiment first (R down 5-6x for the aligned source).
5. **ADR-0144 §7's condition (3), the gate:** the decode-bound project must stay on one painter by the trigger; state it as a no-regression test
   (136-149 s, within the same run session) rather than a speed-up, and run it under the quiet-machine rule (not obtained here).
6. **Still unmeasured:** a quiet-machine run (load < 1.5) of any of this; the kit with the reporter's real 12 MB clip (the stand-in is
   synthetic, so real footage's reopen is a little more); a paint-heavy project with videos at 4K; discard-forward feeds; the `preview` path (rungs and the 5 s budget).
