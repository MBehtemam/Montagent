# Where a paint-heavy render's time goes

Profile for [#643](https://github.com/MBehtemam/Montagent/issues/643), part of the map [#641](https://github.com/MBehtemam/Montagent/issues/641) (render is fast on paint-heavy projects). Measured 2026-10-03. Like [#623](https://github.com/MBehtemam/Montagent/issues/623), it says where the time goes and decides nothing about fixes.

## Pinned

- **Commit:** main at `0ccd0778`, release build with `CARGO_PROFILE_RELEASE_DEBUG=line-tables-only` (symbols only; same optimisation level) into a private `CARGO_TARGET_DIR`. No code patch: the profile is from macOS `sample` at 1 ms.
- **Machine:** Apple M1 Pro (8P+2E), 16 GB, macOS 27.0, on mains power. Homebrew ffmpeg 9.0.2, x264 core 165. The render reported `libx264, preset medium, CRF 20, 5 threads, ffmpeg 9.0.2`.
- **Load: observed, not measured.** The load average was 9–47 throughout, never below the 1.5 gate. Another session was rendering its own project at the same time. Following the dev's standing preference (2026-10-03), these are labelled observed runs rather than waiting for a quiet machine. Spreads are reported so the noise is visible.
- **Projects:**
  - `fixtures/benchmark/spy-trailer/trailer.montagent.json` with `score.wav` regenerated (sha256 `8093372d…`).
  - `make_benchmark.py --paint` variants, each rendered over `--from 0 --to 6000` (180 frames). Every bench element is on screen for the whole timeline, so a 6 s range is steady state.

## The trailer

| run | load at start | wall | CPU | peak RSS | frame hashes vs `frames.framemd5` |
|---|---|---|---|---|---|
| 1 | 29.3 | 338.7 s | 114% | 511 MB | identical |
| 2 | 11.7 | 332.7 s | 114% | 519 MB | identical |
| 3 | 46.9 | 356.4 s | 109% | 488 MB | identical |
| **median** | | **338.7 s [332.7–356.4]** | **114%** | **511 MB** | |

That is **0.106× realtime, 313 ms per frame**. It was 273 s at ~137% CPU on the older binary in #509, also under unknown load.

The CPU figure includes the ffmpeg encoder child. The encoder spent 117 s of CPU (sampled run), so the painting thread alone is about one core for the whole render. Paint is single-threaded and is the critical path.

## Where the painting thread's time goes

From one sampled render of the trailer: 244,700 samples on the main thread. The ms/frame column scales each share to the 313 ms median frame.

| element class → operation | share | ms/frame |
|---|---|---|
| **text** | **66.5%** | **208** |
| ↳ blur filter (`blur` and the blur inside `shadow`) | 61.7% | 193 |
| ↳ compositing the filtered layer back (`drawBitmap`) | 4.1% | 13 |
| ↳ shadow merge, layer clear | 0.5% | 2 |
| ↳ shaping and layout (`montagent_text`, HarfBuzz) | 0.1% | <1 |
| **image** | **28.3%** | **89** |
| ↳ drawing (photos, alpha plates; scaled, masked) | 17.1% | 54 |
| ↳ **re-decoding stills** (PNG 95%, JPEG 5%) | 10.7% | 33 |
| **rect / ellipse** | **2.6%** | **8** |
| ↳ blur filter | 2.3% | 7 |
| **render loop** | **2.0%** | **6** |
| ↳ pipe write to the encoder (backpressure) | 0.8% | 2.5 |
| per-frame bookkeeping (element walk, resolve, captions, reports) | 0.5% | 1.6 |

### Full-frame filter layers: 68% of the time

Every effect opens `save_layer` with no bounds (`canvas.rs`, `Canvas::through`). That includes `mask` and the colour effects, and `opacity < 1` opens one more. Layers whose filter is a blur (`blur`, `shadow`) take **about 68% of the painting thread: 61.7% + 2.3% blur, 4.1% composite and 0.5% merge and clear, about 215 ms per frame**. Colour-matrix layers cost almost nothing (<0.1%).

How many layers a frame opens (`layers.py`, from the project's own timings and opacity keyframes):

| per frame | median | mean | p10 | p90 | max |
|---|---|---|---|---|---|
| painted elements | 6 | 9.3 | 5 | 22 | 35 |
| elements carrying any effect | 1 | 4.1 | 1 | 14 | 28 |
| **blur/shadow layers** | **0** | **3.2** | 0 | 14 | 28 |
| full-frame layers of any kind | 3 | 6.9 | 2 | 17 | 59 |

The cost is lumpy. Most frames carry no blur at all. The dense title and credit sections carry 14–28, and at ~50–60 ms each those frames take 1–1.5 s apiece. 3.2 layers × 1080 frames × ~60 ms is about 210 s, which matches the profile.

**What a bounded layer would save** (an estimate, not measured on the trailer): the #644 spike measured 50 ms unbounded against 7.5 ms bounded per blurred element. Applied to the 193 ms of blur per frame, that leaves ~29 ms, a saving of ~165 ms per frame. The trailer would drop from ~339 s to roughly 160 s (~0.22×). It still would not reach realtime on its own, and it changes bytes (#644).

### Stills are decoded again and again

`Painter::stills` keeps an `Image::from_encoded`, which is a **lazy** Skia image. Skia decodes it into its global resource cache at draw time (`SkImage_Lazy::getROPixels` → `SkCodecImageGenerator::getPixels`). The trailer's decoded stills total about 100 MB: `embers.png` 17.6 MB, `grain.png` 12.8 MB, `hud`/`vignette` 7.9 MB each, and six photos at 8.4 MB. That is more than Skia's default resource-cache budget, so decoded bitmaps are evicted and decoded again. This costs **10.7% of the render, about 33 ms per frame or 36 s in all**, and PNG inflate is 95% of it. The map's Notes said stills are cached for the whole render. That is true of the encoded bytes, not of the pixels.

## The `--paint` sweep

Median of 3 over 180 frames. "Over floor" is the per-frame cost the knob adds.

| project | wall, median [range] | ms/frame | over floor | CPU | peak RSS |
|---|---|---|---|---|---|
| floor (`b0-g0-off-t0`) | 2.10 s [1.93–2.24] | 11.7 | — | 331% | 546 MB |
| `--plate still` | 2.16 s [2.15–2.20] | 12.0 | +0.3 | 328% | 546 MB |
| `--plate moving` | 6.66 s [6.38–7.03] | 37.0 | **+25.3** | 272% | 547 MB |
| `--blur-texts 1` | 10.47 s [10.33–10.66] | 58.2 | **+46.5** | 143% | 473 MB |
| `--blur-texts 4` | 37.46 s [37.22–37.75] | 208.1 | **+196.4** (49.1 each) | 111% | 467 MB |
| `--glow-texts 1` | 11.06 s [10.89–11.25] | 61.4 | **+49.8** | 141% | 475 MB |
| `--glow-texts 4` | 41.72 s [40.34–41.74] | 231.8 | **+220.1** (55.0 each) | 108% | 456 MB |
| `--extra-tracks 108` | 2.00 s [1.96–2.36] | 11.1 | −0.6 | 343% | 546 MB |

- **Blur and glow are linear per element:** ~47–49 ms per `blur` text and ~50–55 ms per `shadow` glow, each on its own full-frame layer. This repeats the #644 spike's ~50 ms.
- **The track count costs nothing.** 108 extra tracks are within noise.
- **The moving plate is encoder cost, not paint.** Sampling the full 36 s renders, the painting thread sits in the pipe write 83% of the time with the moving plate and 50% with the still one. The plate draws at ~1–2 ms either way. What moves is x264. Grain that changes every other frame has to be encoded as new detail, while a still plate is nearly free to encode.

## The encoder ceiling

The trailer's own frames, decoded and piped as RGBA into the render's encoder settings (`libx264 -preset medium -crf 20 -threads 5`, no painting at all):

| run | wall | CPU |
|---|---|---|
| 1 | 31.9 s | 358% |
| 2 | 30.8 s | 365% |
| 3 | 29.6 s | 378% |
| **median** | **30.8 s** (28.5 ms/frame, 1.17× realtime) | ~3.6 cores |

Decoding alone took 3.8 s, so the figure is the encoder's. **Even with free painting, this trailer cannot render faster than ~31 s, or ~1.17× realtime, with ADR-0143's settings**, and the encoder needs ~3.6 of the 10 cores while it does it. A ≥ 1× target leaves ~5 s for everything else. Painting in parallel would compete with the encoder for the same cores.

## Compared with the render-speed profile (#623)

| | #623, video-heavy | #643, paint-heavy |
|---|---|---|
| dominant cost | `frame_at` decode, 95% | full-frame blur layers, ~68% |
| paint per frame | 2.3 ms | ~300 ms |
| second cost | encoder backpressure 1.5% | still re-decode 10.7%, image draw 17% |
| cores used | 1.8–2.6 | ~1.1 |
| encoder alone | ~280 fps (screen recording) | ~35 fps (grain and blur detail) |

## Reproduce

Set `PAINT_PROFILE_OUT` to a scratch directory holding `target/` (the release build), `bench/` (the `--paint` projects plus `fonts/ fx/ img/` copied from `spy-trailer/`) and `p/`.

- `time.sh <project> <tag> check`: one timed render, plus the frame-hash check.
- `prof.sh <project> <tag>`: a whole-render `sample` profile.
- `sweep.sh`: the 3 × 8 bench grid.
- `tree2.py <sample.txt>`: the class × operation table.
- `layers.py <project>`: per-frame layer counts.
