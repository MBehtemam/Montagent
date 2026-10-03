# Can Skia reuse a filtered raster across frames, and paint the same bytes from K painters?

Research for [#644](https://github.com/MBehtemam/Montagent/issues/644), part of the paint-speed map [#641](https://github.com/MBehtemam/Montagent/issues/641).

## Answer

| Candidate | Same transform | Integer translation | Subpixel translation | Opacity only | Memory per cached element (1080p) |
|---|---|---|---|---|---|
| **A. Reuse the `ImageFilter` object** | PASS | PASS | PASS | PASS | ~0 (a small ref-counted parameter object). **Saves no time**: the filter still runs every frame. |
| **B. Record an `SkPicture`** | PASS | PASS | PASS | PASS | 692 bytes measured (`approximate_bytes_used`, 5 ops). **Saves no time**: playback re-runs the `save_layer` and the filter. |
| **C. Raster snapshot, redrawn later** | PASS | **FAIL** (4 of 9 cases pass, by luck) | **FAIL** (0 of 9) | PASS (same transform, any opacity) | 1.17–1.40 MB measured for a 960×240 element (tight filtered bounds, RGBA8888). A full-frame snapshot is 8.29 MB. |
| **D. Skia's own caches** (image-filter cache, resource cache) | Never hit | Never hit | Never hit | Never hit | Global, process-wide: 128 MB image-filter cache by default. Every frame fills it with entries that are never hit. |

**Determinism across painters: PASS.** K = 2, 4 and 8 threads, each with its own surface, painted 96 animated frames. Each frame was hash-identical to one surface painting them sequentially. That held with frames interleaved or in contiguous chunks, and with every Skia cache purged before the parallel run. A fresh surface per frame also matched a reused one. At the Skia level this meets the precondition of the parallelism ladder's "same hashes" promise.

In short, the only reuse that is byte-identical **and** saves time is a raster snapshot (C), and only for frames where the element's transform is **exactly** the same. An opacity change is allowed. Any translation is not, and neither are integer steps. Candidates A and B are byte-identical but save nothing, and Skia's own caches (D) can never hit on the `save_layer` path.

All "PASS"/"FAIL" verdicts compare every byte of the composited 1920×1080 RGBA8888 premul frame. The evidence is the spike in [`spike/`](spike/), with its full output in [`results.txt`](results.txt).

## Setting

- **Skia build:** `skia-safe =0.153.2`, the workspace pin with its exact feature list (`binary-cache`, `embed-icudtl`, `pdf`, `jpeg`; CPU raster, no `gl`). `skia-bindings 0.153.2` declares `skia = "m153-0.101.1"` in its `Cargo.toml` metadata. That is the rust-skia fork's tag of Skia `chrome/m153`. Every Skia file cited below is byte-identical between that tag and upstream `chrome/m153`, so the links point at upstream.
- **Machine:** Apple M1 Pro, macOS 27.0, rustc 1.95.0, release build, private `CARGO_TARGET_DIR`.
- **Timings are indicative only.** Load average was 7–18 during the runs (other sessions), which breaks #641's timing rules. Bytes do not depend on load.

## How today's draw works (the reference)

`Canvas::in_element_space` and `Canvas::through` (`crates/montagent-render/src/canvas.rs`) do the following:

1. Open `save_layer_alpha_f(None, opacity)` when opacity < 1. This is *outside* the transform.
2. Translate, rotate, scale, and apply the origin.
3. For each effect, open `save_layer` with a paint whose `ImageFilter` is built fresh by `Effect::filter`.

`blur` is `image_filters::blur(σ = radius/2)`. `shadow` is `image_filters::drop_shadow`, with `opacity` multiplied into the colour's alpha. The spike's `element` and `through` functions copy this code path line for line. It uses one effect per element.

The scene is an opaque striped backdrop. The element is 960×240 and holds an AA rounded rect, a stroked oval and 110 px text. It was tested with three effects:

- blur r16;
- shadow at offset (12, 14), r24, black at 0.6;
- a zero-offset glow, r30.

Each effect was tested at three base transforms: integer position, fractional position (960.37, 540.81), and fractional plus 7° rotation. Five changes were applied to each base:

- the same transform;
- an integer translation of +37, −23;
- a subpixel translation of +0.5, +0.25;
- opacity 0.6;
- the integer translation plus opacity 0.6.

A ×1.1 scale ran as a control. That gives 9 cases per change. The live draw repeated twice was identical in all 54 cases, so the reference is stable.

## Candidate A: reusing the `ImageFilter` object

- **Bytes:** PASS, 54/54. An `SkImageFilter` is an immutable description. Building it once or every frame gives the same graph.
- **Time:** none saved. Measured at 50.4 ms/frame reused vs 50.1 ms/frame fresh, for one blurred element. The filter graph is evaluated when the layer is restored, every frame.
- **Why it can't feed the cache either:** each filter object gets its own id (`fUniqueID(next_image_filter_unique_id())`, [SkImageFilter.cpp:152](https://github.com/google/skia/blob/chrome/m153/src/core/SkImageFilter.cpp#L152)). That id is part of the cache key (see D), so reuse is *necessary* for a cache hit. It is not *sufficient*.
- **Memory:** negligible. A ref-counted object holding a few floats. skia-safe exposes no size for it.

## Candidate B: recording an `SkPicture`

- **Bytes:** PASS, 54/54. That covers every transform change, including subpixel and the scale control. The recording holds element-space commands only, and the transform is applied at playback.
- **Time:** none saved, 50.0 ms/frame. `SkCanvas::drawPicture` → `picture->playback(this)` ([SkCanvas.cpp:2891–2914](https://github.com/google/skia/blob/chrome/m153/src/core/SkCanvas.cpp#L2891-L2914)) re-issues the `saveLayer`/draw/`restore`. So the filter runs again, exactly as live. A picture saves only the recording work (Montagent's own per-element setup), and the profile ticket would have to show that is worth anything.
- **Memory:** 692 bytes (`approximate_bytes_used`, 5 ops). [SkPicture.h:221–228](https://github.com/google/skia/blob/chrome/m153/include/core/SkPicture.h#L221-L228) says this excludes "large objects referenced by SkPicture" (here the typeface, which Montagent holds anyway).

## Candidate C: raster snapshot of the filtered layer

The element was rendered through the live path onto a transparent full-frame surface. The snapshot was `image_snapshot_with_bounds`, taken at the filter's fast bounds mapped to device space, rounded out, plus 1 px. It was then drawn later with `draw_image` under the change.

- **Same transform:** PASS 9/9. The live restore and the snapshot redraw both SrcOver the same premul pixels at the same integer device offset.
- **Opacity only:** PASS 9/9, at an unchanged transform. It passes both through a `save_layer_alpha_f` around the redraw (as live does) and with `Paint::set_alpha_f` on the `draw_image`.
- **Integer translation:** **FAIL.** 4 of 9 cases pass, and the passes are luck rather than a rule:
  - all three effects pass at the fractional base;
  - blur and shadow fail at the integer base;
  - the glow passes there;
  - all three fail at the rotated base.

  Failures are small (21–318 bytes differ, max |Δ| 1–5), but they are not zero.

  The probe in `results.txt` places the cause **in the element's own antialiasing, not in the filter.** With a no-op `offset(0,0)` filter, a snapshot moved by (4, 4), (37, −23) or (−200, 100) already differs from a live draw at the moved position by 3–6 pixels on the element's edges. The ±1 coverage differences come from float rounding when local geometry is mapped through a different translation. The blur then spreads them. So "integer translation is exact" does not hold for Skia's AA rasterizer.
- **Subpixel translation:** FAIL 0/9 (up to 347k bytes, max |Δ| 222 on a shadow edge). This was expected: resampling, with nearest or linear sampling, is not re-rasterizing.
- **Scale** (control): FAIL 0/9.
- **Memory:** `width × height × 4` bytes of the filtered device bounds. For a W×H element on screen with blur σ, that is about (W + 6σ + 2)(H + 6σ + 2) × 4. Measured at 960×240:

  | Effect | Snapshot size | Bytes |
  |---|---|---|
  | blur r16 | 1010×290 | 1,171,600 |
  | shadow r24 | 1034×314 | 1,298,704 |
  | glow r30 | 1052×332 | 1,397,056 |

  A full-frame snapshot is 8,294,400 bytes. Each K painter holding its own cache multiplies this by K.
- **Time:** redraw 0.64 ms/frame vs 50 ms live. The floor (backdrop + pixel read) is 0.53 ms.

So a snapshot cache is byte-safe only when it is keyed on the element's **full transform**: x, y, rotation, scale and origin, bit-exact, plus everything that feeds the content and the filter. Opacity may stay outside the key, applied at redraw. For an element that sits still for a run of frames (a title holding position while it fades) that is a real cache. For anything moving, every frame is a miss.

## Candidate D: Skia's own caches

- **The image-filter cache is on for raster.** The raster backend is built with `SkImageFilterCache::Get()` ([SkImageFilterTypes.cpp:183–187](https://github.com/google/skia/blob/chrome/m153/src/core/SkImageFilterTypes.cpp#L183-L187)). That cache is a process-wide LRU of `kDefaultCacheSize = 128 MB` on non-iOS ([SkImageFilterCache.cpp:22–26](https://github.com/google/skia/blob/chrome/m153/src/core/SkImageFilterCache.cpp#L22-L26)), guarded by a mutex.
- **What it keys on:** the filter's unique id, the layer matrix, the desired output rect, the **source image's unique id** and the source subset ([SkImageFilterCache.h:23](https://github.com/google/skia/blob/chrome/m153/src/core/SkImageFilterCache.h#L23), [SkImageFilter.cpp:231–262](https://github.com/google/skia/blob/chrome/m153/src/core/SkImageFilter.cpp#L231-L262)).
- **Why `save_layer` never hits:** the source is the layer device, snapped at `restore` ([SkCanvas.cpp:805–807](https://github.com/google/skia/blob/chrome/m153/src/core/SkCanvas.cpp#L805-L807), `SkBitmapDevice::snapSpecial` → `MakeFromRaster` on the layer's bitmap). Each `save_layer` allocates a new layer device, so the source id is new every frame. Today's per-frame `ImageFilter` also gives a new filter id. **Measured:**
  - reused filter through `save_layer`: 50.4 ms, no faster than fresh;
  - the same filter on a *stable* source image through `images::make_with_filter`: 0.2 ms after the first call (48.0 ms with a fresh filter).

  So the cache works when its key repeats, and the `save_layer` path never repeats it.
- **The resource cache** (`graphics::resource_cache_*`) holds decoded/discardable bitmaps and similar. The filter path above does not consult it for filter results.
- **Bytes:** a frame painted after `graphics::purge_all_caches()` is identical to one painted warm (PASS). Cache state does not change pixels.
- **Memory:** nothing per element is under Montagent's control. The 128 MB process-wide budget fills with never-hit entries. Shrinking it is not exposed by skia-safe 0.153.2 (`graphics.rs` exposes font and resource cache limits only).
- **Wrapper bug:** in skia-safe 0.153.2, `images::make_with_filter` always returns `None`. The wrapper builds the result, then discards it (`image.rs:217–238`: `.map(...);` followed by `None`). Anyone pursuing "cache the unfiltered content, filter it via `make_with_filter`" must call through `skia_bindings` or patch the wrapper.

## Determinism across K painters

The scene had 8 elements per frame, mixing blur (radius varying per frame), drop shadow and glow, with text. Positions moved by fractional amounts, rotation and scale were animated, opacity varied, and every filter was rebuilt per frame as today. Results over 96 frames:

| Run | Result |
|---|---|
| Sequential, one reused surface | the reference hashes |
| Fresh surface per frame, same thread | IDENTICAL |
| K = 2, 4, 8 threads, frames interleaved (i mod K) | IDENTICAL, 0/96 differ for each K |
| K = 2, 4, 8 threads, contiguous chunks | IDENTICAL, 0/96 differ for each K |

All caches were purged before each parallel run, so the threads repopulated the glyph and filter caches concurrently.

This is consistent with the source:

- CPU raster drawing is single-threaded per canvas.
- The shared global caches (strike cache, image-filter cache, resource cache) are mutex-guarded. Their entries are pure functions of their keys, so which thread fills an entry first cannot change its bytes.
- No paint in `montagent-render` turns dithering on.

The wall times showed the parallel shape: 65 s sequential, then 34, 17 and 11 s for K = 2, 4, 8. They are only indicative under the load noted above.

**Scope of this answer:** Skia plus fresh surfaces, on one machine, in one process. It does not cover Montagent's `Painter` state (the stills cache, text shaping through `montagent-text`, report building). That is code above Skia, and the ladder's own implementation must keep it per-painter or pure.

## Side observation for #641: what the time actually is

One blurred 960×240 element costs about 50 ms/frame at 1080p, against a 0.5 ms floor. An unhinted filter `save_layer` sizes its layer from the device clip, i.e. the whole frame ([SkCanvas.cpp:939–947](https://github.com/google/skia/blob/chrome/m153/src/core/SkCanvas.cpp#L939-L947), plus 1 px of padding at :995), and the blur runs over that layer. A `bounds` hint of the element box cuts it to 7.5 ms. **The hint changes bytes**, though: 23/54 PASS, the rest within max |Δ| 5. Outsetting the hint by 8 or 64 px, or stretching it back to the frame origin, still fails (26/54 and 32/54). So the hint is pixel-trading and outside this map's rules unless a superseding ADR argues for it.

Raster blur stays in the Gaussian path up to σ = 135, with no downscaling ([SkBlurEngine.cpp:1207–1216](https://github.com/google/skia/blob/chrome/m153/src/core/SkBlurEngine.cpp#L1207-L1216)), so the radius alone does not explain the cost. Layer area does.

## Not settled

- **Whether a bounds hint can be made byte-identical.** Three hint shapes were tried and all fail. Why (layer extent, padding or restore sampling) was not chased, because it is outside #644's four candidates.
- **Per-element cost of an `ImageFilter` object.** skia-safe exposes no byte size. It is a few floats plus a ref count.
- **Determinism across machines or CPU feature sets.** Only one M1 Pro was tested. Skia picks SIMD paths per CPU, so the "same hashes" promise is per machine, which matches how golden frames are already treated.

## Reproduce

```sh
CARGO_TARGET_DIR=<scratch>/target cargo run --release \
  --manifest-path docs/research/skia-filtered-raster-reuse/spike/Cargo.toml -- all
# modes: bits | caches | threads | probe | all (probe is not part of `all`)
# side check env: HINT_PAD=<px>, HINT_ORIGIN=1 (bounds-hint variants)
```
