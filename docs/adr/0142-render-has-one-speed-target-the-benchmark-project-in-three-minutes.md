---
status: accepted
amends: 0072 (render gets a target: one absolute number for one project on one machine, judged by an `#[ignore]`d test run on purpose; `Budget::Render.limit()` stays `None`, which now means not enforced in CI rather than no target)
---

# `render` has one speed target: the benchmark project in three minutes

> **Amended by [ADR-0144](0144-render-paints-on-k-painters-over-chunks-and-the-spy-trailer-renders-in-a-minute.md)**,
> which turns "one speed target" into one target per bottleneck class, a closed list of two:
> **decode**, this ADR's benchmark project in ≤ 3 minutes, and **paint**, the spy trailer in
> ≤ 60 s (`PAINT_TARGET`). Each moves only through its own superseding ADR, a miss in one
> never reopens the other, and adding a class takes an ADR with a profile in which one stage
> dominates. Everything else here stands.

[#625](https://github.com/MBehtemam/Montagent/issues/625), from
[#532](https://github.com/MBehtemam/Montagent/issues/532). Settled over two grilling rounds,
each also put to a court of three independent jurors. The decisions below are the Judge's
read, which the dev adopted.

## The gap

ADR-0072 retired the only render figure there was and left `render` with no target. It said a
replacement needs three things: numbers at more than one frame size, a spread to derive from,
and a statement of whose target it is.

Meanwhile a real 6-minute project with three videos on screen took about an hour to render.
Every `video` element is decoded through one `ffmpeg` seek per frame (ADR-0077 recorded that
as a cost and left it unmeasured). The committed fixture has no `video` element, so nothing
this project had measured could see it.

## Decision

### 1. The target

**The benchmark project, 6 minutes of 1080p30 with three videos visible at once, renders in
≤ 3 minutes median wall time on the dev's M1 Pro (8P+2E, 16 GB).** That is half of real
time, and 20× better than the hour.

- **Shape:** an absolute number for one scenario. A rate per visible video element is
  recorded as an observation and never enforced. Cost isn't linear in the number of
  elements: one element is bound by the encoder, many by the cores.
- **The number is fixed.** The first clean measurement confirms it; it does not adjust it.
- It was chosen for the speed wanted, not to force or skip the gated parallel-paint steps
  ([#627](https://github.com/MBehtemam/Montagent/issues/627)). If streaming alone meets it,
  those steps are never built, and that is the gates working.
- **Why 3 minutes:** every juror's arithmetic puts it within reach without the encoder
  ticket, with margin over the encoder floor. The floor is 37–68 s for libx264 `medium` at
  about 160–290 fps, from *Research: encoder and decoder throughput for render on Apple
  silicon* ([#624](https://github.com/MBehtemam/Montagent/issues/624)).
- **Frame sizes:** only 1080p30 is committed. 2160p30 is measured and recorded in
  `BENCHMARK_REFERENCES`, beside `RENDER_REFERENCES`, with no ceiling, which meets ADR-0072's "numbers at more than one frame
  size". A 4K number waits for a later ADR, because its floor is the encoder's, and that is
  the encoder ticket's question.

### 2. The benchmark project

`fixtures/benchmark/make_benchmark.py` builds it from the committed fixtures into a directory
the caller names. Generated media is not committed.

- 6 minutes of 1920x1080 at 30 fps.
- **Three `video` elements visible at once for 330 of the 360 s**: three portrait lanes,
  20 `video` elements cut back to back at staggered instants.
- Text (37 elements), rects (3, one moving on every frame) and an audio mix (18 narration
  lines, a looped bed, and the videos' own sound).
- Sources re-encoded to long GOPs. **The script pins the codec, profile and GOP**: H.264 High,
  `yuv420p`, preset `medium`, CRF 20, 3 B-frames, a closed fixed 10 s GOP. So the benchmark
  doesn't drift with the ffmpeg version.
- The same script builds the observed variants: **2160p30**, and a **one-video** variant
  (the source of the per-element rate).

**Naming.** It is the **benchmark project**, never the "reference project":
`RENDER_REFERENCES` already means reference *measurements*, and "reference" is on
`CONTEXT.md`'s Avoid lists. It gets no `CONTEXT.md` entry, because the glossary is the
format's language and this is repository infrastructure. It is defined here, in
`RENDER_TARGET`'s doc and in the generator's header.

**The dev's real 6-minute project** gets one confirmation run under the same protocol. This
ADR describes it by shape only (duration, element count, codecs, resolutions), with no paths
or names, since the repository is public.

### 3. What may be traded

- **Pixels: nothing.** The encoder's input stays identical, frame for frame. The evidence is
  frame-hash equality with a sequential render, not an MP4 checksum.
- **MP4 bytes** may change only through *Which encoder settings does render use once decode
  is fast?* ([#628](https://github.com/MBehtemam/Montagent/issues/628)), with its own quality
  evidence (VMAF and bitrate). They are not frozen to today's file; byte identity isn't in
  Montagent's control across ffmpeg and x264 versions anyway.
- **Cores and memory:** every core may be used.
  - Memory stays bounded only by what #627 set: 2 frames in flight per feed and per writer,
    plus the byte budget for chunks.
  - The encoder's share of cores is **measured**, not assumed (about 6 of 10 at 1080p; it
    differs at 4K).
  - **Peak memory is recorded** with every measurement, so "bounded" has a number behind it.
  - No "−1 core" and no RAM cap. A user-facing `--jobs` is a separate, later ask.

### 4. CI

- **No wall-clock gate.** The `macos-15` runner is a shared 3-vCPU M1, and ADR-0021 keeps the
  expensive arms observational.
- **A deterministic spawn-count test** instead: the `ffmpeg` spawns per render equal the
  feeds opened plus their reopens, never one per frame. It catches the structural regression
  that caused the hour, and reopen storms. It lands with the streaming feeds.
- It doesn't catch slowdowns that keep the same structure. Those show as drift against
  the readings recorded here and in `BENCHMARK_REFERENCES`. **Re-measure whenever render's hot path changes.**

### 5. The measurement protocol

`crates/montagent/tests/render_target.rs` is **the** measurement: for the target, for every
gated parallel-paint step (#627) and for the encoder ticket (#628).

```sh
cargo test --release -p montagent --test render_target -- --ignored --nocapture
```

- **Timed:** the whole `montagent render` command, process start to finished MP4 and report:
  probing, feeds, decode, paint, encode, audio mix, mux and faststart. Cold, with an empty
  probe sidecar on every run. Building the benchmark media is not timed.
- **Build:** `--release`, with a private `CARGO_TARGET_DIR`, from the measured commit.
- **Machine:** the M1 Pro on AC power, load average < 1.5 at the start, no other sessions,
  renders or builds running, output to local SSD.
- **Runs:** 1 discarded warm-up, then **5 timed runs**. The **median** is judged; min and max
  are reported.
- **Recorded with each measurement**, as one JSON line the test prints: the commit, the
  ffmpeg version string, macOS version and `available_parallelism`, load average before and
  after, user+sys CPU time and peak memory (`/usr/bin/time -l`), the spawn count (counted on
  the warm-up, through a shim that would slow a timed run), the render's own answer, and a
  per-stage breakdown (decode, paint, encode wait) once render exposes one.
- **Validity:** a run counts only if its frame hashes and report equal a sequential
  render's. Until a parallel path exists every render is sequential, so the hashes are
  compared across this binary's own runs. After it exists, the reference is the `framemd5` a
  sequential build wrote (`MONTAGENT_RENDER_TARGET_SEQUENTIAL_FRAMEMD5`). The hashes are of
  the output MP4's decoded frames: equal encoder input gives equal hashes, but a change small
  enough to quantize away could pass unseen. That is the test's stated limit.
- **Scenarios:** the 1080p30 benchmark project (committed); 2160p30 (observed); the one-video
  variant (observed); the real project (one-time confirmation).
- The 64 s profile taken at load 5–9 is not evidence for the number.

### 6. In `budget.rs`

- **`RENDER_TARGET`** = 3 min, for the benchmark project. The one place the number lives.
- `Budget::Render`'s doc no longer says "render has no performance target". It points here
  and says `limit() == None` means **not enforced in CI**, not "no target".
- The `#[ignore]`d test **refuses to judge and prints why** on a debug build, a load average
  ≥ 1.5 at the start, battery power (`pmset -g batt`), a machine other than the M1 Pro, fewer
  than 5 timed runs, or frame hashes or a report that differ from a sequential render's. The
  last also fails the test: different pixels are wrong before they are slow.
- New observations of the benchmark project go into **`BENCHMARK_REFERENCES`**, a list of
  their own beside `RENDER_REFERENCES`. That list holds the committed fixture's readings, and
  `nearest_reference` scores a run by output length alone, so a six-minute benchmark reading
  there would become the baseline for any long render and move the fixture's recorded
  spread. A benchmark reading is a record and never a baseline.

### 7. On a miss

- If the benchmark project still misses after every gated step that got built, **the target
  stands**.
- The remaining gap goes to the encoder ticket (#628), which may trade MP4 bytes with quality
  evidence.
- If it still misses after that, an ADR records the miss, a new issue is opened, and **the
  number changes only through a superseding ADR**.
- The streaming-only "after" may miss 3 minutes. That miss doesn't move the number; it
  triggers parallel-paint step 1.

## The measurements

**This first acceptance did not use §5's protocol.** By the dev's decision, it rests on one
observed run under load and a derived "before", in place of a warm-up and five timed runs at
load < 1.5. The reason is the machine: with a desktop running, this Mac's 1-minute load
average went under 1.5 once in about five hours of watching. The protocol test stays **the**
measurement for every later gated step (#627) and for the encoder ticket (#628), which judge
against the same number. An observed number under load is an upper bound on the quiet one,
so it can confirm a target it is inside of; it could not have recorded a miss.

Conditions: ffmpeg 9.0.2, macOS 27.0 (26A428), the dev's M1 Pro (8P+2E, 16 GB),
`available_parallelism` 10, AC power, release build with a private `CARGO_TARGET_DIR`.

**Before** — the render path without feeds (`97751b08`). **Derived, not timed.** #623's
profile put decode at 95% of the wall, at 93–113 ms per frame per visible video element, or
about 18–21 minutes per element for six minutes. The benchmark project has 30,600 video
element-frames, each one `frame_at` spawn.

| scenario | commit | how | wall | spawns |
|---|---|---|---|---|
| benchmark project, 1080p30 | `97751b08` | derived from #623 | about 50–60 min | 30,600 `frame_at` |

**After** — the streaming PR (ADR-0141).

| scenario | commit | how | wall | user+sys | peak memory | decode spawns | verdict |
|---|---|---|---|---|---|---|---|
| benchmark project, 1080p30 | `42bceacb` | one observed run, load 6.0 → 9.7, no warm-up | **135.4 s** | 749.6 + 51.7 s | 743 MiB | 20 feeds, 0 reopens, 0 `frame_at` | **within 3 min** (observed, upper bound) |
| 2160p30 (observed) | — | **not measured** | | | | | — |
| one video, 1080p30 (observed) | — | **not measured** | | | | | — |
| the dev's real project, a 6:33 cut (observed) | `6a5f3aab` | one observed run by the dev, installed binary | **171 s** | — | — | — | about 0.43 render minutes per minute of video |

- The after run's per-stage breakdown (ADR-0141 §9), summed over 10,800 frames: decode
  45.0 s, paint 54.6 s, readback 13.1 s, encode wait 20.4 s, seal 0.2 s. `decoded_per_frame`
  is `[]`.
- Peak memory is the largest single process (`/usr/bin/time -l`). The `ffprobe`, encoder and
  audio spawns were not counted; `render_spawns.rs` holds their structure in CI.
- Its frame hashes were not compared with a sequential render's. Pixel identity rests on
  ADR-0141 §7's standing byte-equality test.
- `BENCHMARK_REFERENCES` records this reading, labelled as observed under load. The 2160p30
  and one-video variants were not measured, so ADR-0072's "numbers at more than one frame
  size" is **still open** for the benchmark project. The first protocol run of a gated step
  should take them.
- **The dev's real project** was confirmed by one observed run of `montagent` 0.1.2 built at
  `6a5f3aab`. That build also carries ADR-0143's encoder pin. A 6:33 cut rendered in
  2 min 51 s. Before that, 0.1.0 rendered an earlier 4:53 cut in 24 min 25 s, which is about
  5.0 render minutes per minute of video, against about 0.43 now: roughly 11.5× faster. A
  6:43 cut on 0.1.0 took 44 min 11 s while another render shared the machine, so it is not
  used. The cuts differ, so this is indicative rather than an exact A/B. The project's
  shape (frame size, videos on screen at once, source codecs) was not recorded with the run,
  and neither were CPU time, peak memory or the spawn counts. Its smaller speed-up than the
  benchmark project's is consistent with fewer videos on screen at once, though that is
  unconfirmed. Scaled to six minutes, it would take about 2.6 min, inside `RENDER_TARGET`.

**Verdict:** the benchmark project renders within `RENDER_TARGET` on this evidence. The
gated parallel-paint steps (#627) are not triggered by a miss. Whether one is still worth
building is the dev's call, made with the protocol.

## Consequences

- `render` has a target, and ADR-0072's "no target" is amended: `Budget::Render` stays
  observational in CI, and that is all `limit() == None` now says.
- The streaming PR, the gated parallel-paint steps and the encoder ticket are each judged by
  the same test against the same number.
- The benchmark project is the only scenario the number speaks for. It is an animated short
  re-encoded to about 1.75 Mb/s, well under a phone's bitrate, so a real project's decode
  may cost more per frame; the real project's confirmation run is how that gets seen.
- Nothing about `frame`'s `< 500 ms` or the scrub preview's `< 5 s` changes.
