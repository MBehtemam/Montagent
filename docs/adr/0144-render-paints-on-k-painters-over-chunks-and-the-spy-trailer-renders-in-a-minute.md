---
status: accepted
amends: 0142 (its "render has *one* speed target" becomes one target per bottleneck class, a closed list of two: decode, the benchmark project in ≤ 3 min, ADR-0142; paint, the spy trailer in ≤ 60 s, this ADR. Nothing else in ADR-0142 changes)
---

# `render` paints on K painters over chunks, and the spy trailer renders in a minute

[#653](https://github.com/MBehtemam/Montagent/issues/653), the paint-speed map's
([#641](https://github.com/MBehtemam/Montagent/issues/641)) paint-target ADR. It builds the
parallelism ladder's step 3 for the paint class. The decisions come from four specs, each
settled by grilling and put to a court of three jurors, whose Judge's read the dev adopted:

- [How does render paint on more than one core?](https://github.com/MBehtemam/Montagent/issues/627#issuecomment-5959503553)
  (#627): the ladder, and step 3's invariants (§5).
- [What speed does render commit to on paint-heavy projects?](https://github.com/MBehtemam/Montagent/issues/645#issuecomment-5977266740)
  (#645): the target, its gate and the miss branch.
- [Which byte-safe fix takes the full-frame blur layers' time?](https://github.com/MBehtemam/Montagent/issues/647#issuecomment-5977892956)
  (#647): step 3 opened directly, and the filtered-layer raster cache rejected.
- [How far may painting lead the encoder on a lumpy timeline?](https://github.com/MBehtemam/Montagent/issues/650#issuecomment-5980840391)
  (#650): dispatch in timeline order, and the window as a byte budget.

## Context

[Profile: where a paint-heavy render's time goes](https://github.com/MBehtemam/Montagent/issues/643#issuecomment-5969113075)
(#643) measured `fixtures/benchmark/spy-trailer/` on main: **338.7 s** for 36 s of 1080p30
(0.106× realtime, 313 ms a frame). Full-frame blur and glow layers took ~68% of the painting
thread and re-decoding stills 10.7%; the encoder alone needs 30.8 s on ~3.6 cores. One
painter thread fed one encoder, so nine of ten cores sat mostly idle.

ADR-0142 gave `render` *one* target, for a project whose time went to decoding video. The
trailer has no `video` element, and nothing that made the benchmark project fast reaches it.

## Decision

### 1. The target

**`fixtures/benchmark/spy-trailer/` renders in ≤ 60 s median wall time on the dev's M1 Pro
(8P+2E, 16 GB), with every one of its 1,080 frame hashes equal to `frames.framemd5`.**
`PAINT_TARGET` in `budget.rs`. That is 0.6× realtime, 5.6× faster than the profile.

- **Fixed** against the profile recorded on main before any fix (338.7 s). The first clean
  measurement confirms it; it does not adjust it.
- **≥ 1× realtime was dropped:** the encoder alone takes 30.8 of the 36 s.
- **Why 60 s:** stills decoded once plus about six painters gives ~50 s only if scaling is
  near-linear, which blur layers (full-frame, memory-bound) beside an encoder on ~3.6 cores
  make doubtful. 60 s survives 4–5 painters at ~80% efficiency and is about 2× the encoder
  floor.

### 2. One target per bottleneck class (amends ADR-0142)

- ADR-0142's "render has *one* speed target" becomes **one target per bottleneck class**.
- **The classes are a closed list of two:**
  - **decode**: the benchmark project, ≤ 3 min, `RENDER_TARGET`, ADR-0142;
  - **paint**: the spy trailer, ≤ 60 s, `PAINT_TARGET`, this ADR.
- **Each target moves only through its own superseding ADR.** A miss in one class never
  reopens the other.
- **Adding a class takes an ADR** backed by a profile in which one distinct stage dominates
  the render, as decode's 95% and the blur layers' ~68% do. This keeps ADR-0142's guard
  against target sprawl.

### 3. Encoder settings

ADR-0143's settings stand: libx264 `medium`, CRF 20, five threads. They are reopened only on
§9's encoder-bound branch, with VMAF and bitrate evidence.

### 4. What is gated, observed and traded

- **Gated:** the median wall time against 60 s, and frame-hash identity against
  `frames.framemd5`. Nothing else.
- **Observed**, recorded and never enforced: peak memory, CPU, the encoder-alone floor, the
  `--paint` bench sweep, `preview`'s timing, and the slowest frame (only through the total; a
  per-frame maximum would flake under scheduler noise).
- **Pixels:** nothing changes, as ADR-0142 requires.
- **Cores:** every core may be used. **Memory:** no cap, as in ADR-0142 §3. The K chosen and
  the peak memory observed are stated in §7 and the measurements.

### 5. What came before step 3, by reference

Both landed ahead of this ADR, and the trailer timing below measures them with step 3:

- **Stills decode once into raster pixels** ([#648](https://github.com/MBehtemam/Montagent/issues/648),
  PR #654). `Raster::decode` takes the pixels with `CachingHint::Disallow`, so Skia's 32 MiB
  resource cache can no longer evict a still and make it decode again. **CI guard:**
  `a_decoded_still_holds_its_pixels_rather_than_a_lazy_generator` (`montagent-render`
  `canvas.rs`) asserts a decoded still is not lazily generated. This is the paint target's
  first deterministic guard, the counterpart of ADR-0142's spawn count.
- **Blur and shadow layers take a byte-identical bound** ([#649](https://github.com/MBehtemam/Montagent/issues/649),
  [#652](https://github.com/MBehtemam/Montagent/issues/652), PR #655). Each blur/shadow
  filter layer gets a `SaveLayerRec` bounds hint from the recorded content through each
  filter's `computeFastBounds`, so it no longer covers the whole frame, under preconditions
  that keep every byte. Guarded by `crates/montagent-core/tests/filter_bound.rs`, an edge
  project painted with and without the hint and compared byte for byte.

### 6. Step 3 was opened directly; the filtered-layer raster cache was not built

The ladder (#627) gates each step on a measured miss of the previous one. For the paint class
the arithmetic was the measurement (#647): even with every filtered layer reused, one painter
takes ~67 s, over 60 s. Step 2 (per-feed readers) does nothing without a `video` element, and
step 1's writer thread is subsumed by step 3's encoder side.

**The filtered-layer raster cache was measured and rejected** (#647). Counted on a render of
`14380e0b` with a probe at every filtered-layer paint, frame hashes unchanged (branch
`research/647-blur-hitrate`, `f1b4f92e`, `spike/647/analyze.py`):

- **The key:** element id, the full effects list, the nine device-matrix values as f32 bits,
  and a content hash (glyphs, paints, extent, clip). Opacity excluded.
- **3,485 filtered-layer paints** (287 blur, 3,198 shadow) over 399 frames. **Hits: 235
  (6.7%)**; unbounded, previous-frame-only and last-30-frames caches all hit the same 235,
  each an element holding still on consecutive frames. Dense title frames 6.1%, credit
  letters 0%.
- **Misses:** 2,159 change scale and translation, 1,032 translation only, almost all to a
  fractional position. None change content or effect parameters.
- **Saving:** ~12–16 CPU-s of ~232 s of blur, ~2–3 s of wall at K≈6. Keys that would hit
  more (36% without translation, 13% with sub-pixel translation only) change bytes.

A fixture with long, static blurred titles may reopen it on evidence counted the same way.
It is never called a "snapshot", which `CONTEXT.md` avoids twice.

### 7. K painters over paint chunks, feeding one encoder in timeline order

`crates/montagent-core/src/verbs/render/painters.rs`. The span is cut into **paint chunks**
of C frames, handed out to K painters **in timeline order**. Each painter is a thread with a
whole `Painter` of its own (supplier, font registry, decoded stills) and its own canvas;
the document is shared read-only. The verb's own thread is the encoder's side: it takes
frames in timeline order out of a **reorder window** of at most W painted frames and pushes
them through the same loop one painter feeds (`encode_frames`).

- **K** is `available_parallelism` less **four cores for the encoder** (libx264 at five
  threads keeps ~3.6 busy on the trailer, plus its `ffmpeg`'s and the mix's share): **6** on
  the M1 Pro. A K below 2 paints on one painter.
- **C** is **CHUNK_FRAMES = 2** (measured, below).
- **W is a byte budget**, `WINDOW_BYTES` = 150 frames of 1080p RGB (≈ 0.93 GB), turned into
  frames at the render's own resolution, **with a floor of K·C**. When the floor exceeds the
  budget, the floor wins and the answer says so: `painting.window_floor` is `true`, and the
  text names the floor. **The budget covers the window only.** #627 §5 had it cover each
  painter's canvas and decoded stills as well; #650 narrowed it to W, and the painters'
  private memory is observed in the peak RSS below instead. The window counts frames painted and not yet taken by the encoder; a
  painter about to paint a frame W or more ahead of the encoder waits.
- **A document with a `video` element keeps one painter**, on the verb's own thread, as
  before. Step 3 was opened for the paint class. A paint chunk start reopens every visible
  feed, and the ladder's reopen floor (a chunk's paint ≥ 10× its reopens) puts C in the
  hundreds at the decode class's ~3 ms of paint a frame, which makes the window's floor K·C
  gigabytes at 1080p; and the decode class already meets its own target on one painter
  (ADR-0142). Forced (below), chunked painting of video is byte-identical; it is just not
  worth it. **The cost:** the test is "has a `video` element", not "is decode-bound", so a
  paint-heavy project with one small clip loses the painters. #627 §5's "K shrinks, not C"
  would give it fewer painters over long paint chunks instead; that needs a measured reopen
  cost, and waits for a project that shows the need.
- **The answer discloses it**, like `threads`: `render.painting` is `{painters, chunk,
  window, window_floor}`, `1`/the span/`1`/`false` for one painter. Not choosable: no project
  field, flag or MCP parameter reaches it, and it changes no byte of the file. It is the one
  departure from #627 §6's "the same report": like `wall_ms`, it describes how the file was
  made, so it differs with K while the findings and every list of what was painted do not.
  The tests compare the answer with `painting`, `wall_ms` and `realtime` removed.
- **Overrides**, `#[doc(hidden)]`: `render::force_painting` forces one painter or a K, C and
  window budget for the tests; `MONTAGENT_PAINTING=K,C[,W]` does the same for the shipped
  binary's measurement sweeps, as `MONTAGENT_STAGES` does for the stage breakdown. It goes past #627 §5's "a `#[doc(hidden)]` override lets tests force K and C",
  because the sweeps time the binary as a process; it is measurement plumbing, never a
  setting.

**Every invariant of #627 §5 holds:**

- **A chunk start is indistinguishable from `--from`.** `painters.rs`'s module doc lists every
  piece of painter state carried between frames — the per-frame record (reset by `begin`), the
  font registry (shaping reads only the element's declared chain), decoded stills (since #651
  they own their pixels), memoised probes and `ffmpeg`, the supplier (each frame provably
  `frame_at`'s), the append-only `sources`/`fonts` records, and the canvas (cleared every
  frame, its scale set the same way when made) — and why none changes a pixel or the report.
  Thread-local switches (the feed budget, the filter-layer bound) are handed to each painter's
  thread, and the feed counts come back.
- **Failures:** each carries its timeline frame (a painter that cannot make its canvas takes
  its chunk's first). A failure at f stops all work after f at once, lets work before f
  finish, wakes any painter waiting on the window, and is acted on only when the encoder's
  side reaches f — so **the timeline-earliest failure wins** whatever order they arrive in. A
  painter that panics is a failure at the frame it was on, never a frame waited for forever.
- **The report** is merged in chunk order from each chunk's raw record: `painted`, `sources`,
  `fonts` and `decoded_per_frame` by first appearance, `declined` first per `(element,
  code)`, `not_painted` and `painted_partially` as a union — what one painter makes of the
  same frames.
- **Progress** counts frames the encoder's side took, so the tenths stay monotonic.
  **Cancel** is checked by every painter and by the encoder's side (which polls it while
  waiting) and publishes nothing (ADR-0109). `preview`'s deadline is checked on the encoder's
  side, per frame.
- **`preview`** takes step 3 through the one `encode_span`.

### 8. Dispatch stays in timeline order; cost-ordered dispatch was simulated and not taken

#650 simulated step 3 (K painters, C-frame chunks, one in-order encoder at 30.8 s / 1,080
frames, ignoring core contention, so every figure is a **lower bound**) over per-frame paint
tables from two trailer renders (branch `research/650-dispatch-order`, `331410ec`,
`docs/research/dispatch-order/`). With the bound, K=6, C=2, ~1.2 GB:

| | in-order, W=150 | cost-ordered |
|---|---|---|
| as measured | 33.4 s | 30.9 s |
| paint ×1.3 to the profile's ~298 CPU-s | 36.5 s | 32.3 s |

**Cost-ordered dispatch saves 2.5–4 s against a 60 s target**, and would cost a pre-paint
cost estimator coupled to the resolved stack, a second scheduling rule for the failure and
abort tests, and painted-then-discarded late work when an earlier frame fails. **Not taken.**
It returns only through §9's miss branch, when a re-profile shows **the encoder starved while
painters work on a late section** — the observable is the gap between in-order dispatch and
an unbounded window at the chosen W, recorded below.

**The #650 per-frame profile was taken at load 23–35.** Its absolute numbers are inflated
and are not timings under any protocol; only the per-frame *shape* was used. Nobody should
read its 145 or 228 CPU-s as absolutes.

### 9. On a miss: re-profile, then branch

The order is not a fixed ladder. If the trailer takes more than 60 s with everything above
built, **re-profile** and take the branch the profile names:

- **Paint still the critical path:** exhaust the remaining byte-safe options first (the
  filtered-layer raster cache is measured and is *not* one of them, §6); then **cost-ordered
  dispatch** (§8) if the encoder is starved on a late section; then the **bounded filter
  layer that changes bytes**, only through a superseding ADR that states a pixel-diff bound,
  shows visual-diff evidence and re-pins the frame hashes.
- **The encoder the critical path:** reopen ADR-0143 with VMAF and bitrate evidence.
- **Last resort:** supersede this ADR with a slower number. The target never moves silently.

### 10. The gate's form

- **The measurement:** a second `#[ignore]`d case in `crates/montagent/tests/render_target.rs`,
  `the_spy_trailer_renders_within_the_paint_target`, run with ADR-0142's command:
  ```sh
  cargo test --release -p montagent --test render_target -- --ignored --nocapture the_spy_trailer
  ```
  ADR-0142's protocol, with the load checked before **every** run, since a trailer run is
  short enough for the machine to change between two of them. Each run's MP4 is hashed
  against `frames.framemd5` after its clock stops. `score.wav` is generated before anything
  is timed. One JSON line per measurement. **Runs:** the verdict is 1 warm-up and 5 timed
  runs (#645 §6); each sweep point below is the median of 3 (#653), with the spread.
- **The second CI guard** (#647 decision 4):
  `three_painters_over_two_frame_chunks_give_the_blur_and_glow_framemd5_of_one` in
  `crates/montagent-core/tests/painters.rs`. K=3 and C=2 forced on a 320x180 project with a
  `blur` and a zero-offset `shadow` glow over text, shapes and a still, 33 frames (a ragged
  last chunk), every element moving across the span (each chunk boundary mid-animation). The
  frames at the encoder's input, the MP4's bytes, the decoded framemd5 and the report equal
  K=1's.
- **The ladder's §8 tests**, in the same file, each run on its own thread under a timeout so
  a hung painter fails rather than hangs: K=1 with chunking on; K=2, 3 and 4; C=1 and C that
  does not divide the frame count, all against one painter on the verb's thread; paint chunk
  boundaries inside a feed's seek and its loop wrap; a scheduling hook (`fail_frames`) that
  makes a later chunk's failure arrive first while an earlier one lands later, and the
  converse — the earlier is reported both times; a budget below K·C that renders on the floor
  and says so; a cancel mid-encode that publishes nothing; and `preview`'s frames.

## The measurements

MEASUREMENTS

## Consequences

- `render` has two targets, one per bottleneck class, and adding a third takes an ADR with a
  profile behind it.
- A paint-heavy project renders on every core but the encoder's; a project with a `video`
  element renders exactly as before.
- `render` holds up to W painted frames in memory beside K painters' stills and canvases.
  W is a budget in bytes, so a 4K render holds a quarter as many frames, never four times the
  bytes — except where the floor K·C wins, which the answer says.
- `preview` inherits the painters through `encode_span`.
- The answer gains `painting`. It is disclosed, not choosable, and like `threads` it says how
  the file was made, not what is in it.
