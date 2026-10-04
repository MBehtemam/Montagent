# Is there a filter-layer bound that keeps bytes identical?

Research for [#649](https://github.com/MBehtemam/Montagent/issues/649), part of the paint-speed map [#641](https://github.com/MBehtemam/Montagent/issues/641). It follows [#644](https://github.com/MBehtemam/Montagent/issues/644)'s side observation: a `bounds` hint on the filter `save_layer` cut a blurred element from ~50 ms to 7.5 ms, but changed bytes.

## Answer

**Yes, there is one. But no bound that shrinks the layer on all four sides is byte-identical, however wide it is.** That includes Skia's own computed bounds, 3σ, 4σ, and each of those plus 1 or 2 px. All of them fail framemd5 on 609–617 of the trailer's 1,080 frames.

The kernel's reach is not the cause. Moving the layer's **top-left** corner moves where the element is rasterized inside the layer. The integer shift is applied in float, so antialiased coverage changes by ±1 on a few edge pixels, and the blur then spreads that change.

**The byte-identical bound keeps the unbounded layer's top-left corner and pulls in only the right and bottom edges.** It pulls them in to Skia's own bounds of what the element draws: the cull rect of an `SkPicture` recorded with a bounding-box hierarchy, passed through the filter's `computeFastBounds`. In the spike this is mode `keep:pic`.

| Check | Result for `keep:pic` |
|---|---|
| framemd5, all 1,080 trailer frames, against `fixtures/benchmark/spy-trailer/frames.framemd5` | **PASS**, 0 frames differ (run twice) |
| Raw frames: a hash of the RGB bytes handed to the encoder, against the unbounded render | **PASS**, 0 of 1,080 differ |
| Synthetic edge cases, full-size PNG, against the unbounded render | **PASS**, 104 of 104 instants (26 cases) |
| Edge cases with layer-space σ > 135 | **FAIL**, 1 of 16 identical. This is a precondition, explained below. |

**Precondition:** the layer-space sigma must be ≤ 135 on both axes, which is `sigma × scale`, i.e. a Montagent `radius × scale ≤ 270`. Above it, Skia downsamples the whole layer before blurring, and that resampling depends on the layer's size. The trailer's largest is σ = 14 × 1.4 = 19.6.

**Observed time** for back-to-back renders on the same binary, with load average 4–9:
- The filtered elements' time drops from **176.7 s to 59.5 s** of CPU across the trailer's 3,485 filtered-element paints. The median per element goes from **50.8 ms to 16.7 ms**.
- The whole render's wall time drops from **258 s to 143 s**.

That is about two-thirds of the filter time, not the ~97% a pixel-trading four-sided bound gets (2.2 ms/element). The layer still starts at the frame's top-left, so an element near the bottom-right corner saves little.

## Contents

- [The sweep on the trailer](#the-sweep-on-the-trailer)
- [Edge cases](#edge-cases)
- [Why four-sided bounds fail](#why-four-sided-bounds-fail)
- [Derivation: why `keep:pic` is exact](#derivation-why-keeppic-is-exact)
- [Why the content bound must be Skia's recorded bounds, not the element box](#why-the-content-bound-must-be-skias-recorded-bounds-not-the-element-box)
- [Timing](#timing)
- [What a shippable version must hold](#what-a-shippable-version-must-hold)
- [Not settled](#not-settled)
- [Reproduce](#reproduce)

## The sweep on the trailer

The trailer has 59 elements with a `blur` or `shadow`. Each has exactly one such effect. 45 are zero-offset shadows (glows) of radius 10–30 and 14 are blurs of radius 14. Many are text, some animate `scale`, and none rotate. Painted over 1,080 frames, they make 3,485 filtered-element paints.

The modes are implemented in `crates/montagent-render/src/canvas.rs` on this branch (`spike649`, selected with `MONTAGENT_FILTER_BOUND`). The hint is the `SaveLayerRec::bounds` of each filter layer, in element space. "Chained" means the hint for effect *i* is the output bound of effects 0..*i* applied to the element's content.

Swept in the order the issue adopted:

| Mode | Hint | framemd5 frames differ (of 1,080) | Raw frames differ |
|---|---|---|---|
| `none` | no hint (today) | **0** | – |
| `fast` | Skia `computeFastBounds` of the element box (box + 3σ) | 609 | 230 |
| `out` | Skia `filterBounds(kForward)` of the element's device bounds, mapped back | 609 | 230 |
| `k3` | box + 3σ | 609 | 230 |
| `k3+1` | box + 3σ + 1 px | 613 | 231 |
| `k3+2` | box + 3σ + 2 px | 612 | 231 |
| `k4` | box + 4σ | 617 | 228 |
| `k4+1` | box + 4σ + 1 px | 611 | 225 |
| `k4+2` | box + 4σ + 2 px | 617 | 228 |
| `keep:fast` | `fast`, left/top pinned to −10⁷ | **0** | **0** |
| `keep:out` | `out`, left/top pinned | **0** | **0** |
| `keep:pic` | Skia's recorded content bounds → `computeFastBounds`, left/top pinned | **0** (twice) | **0** (twice) |

The raw hash is the stricter check: framemd5 hashes the encoder's output, and x264's prediction spreads one changed frame into its neighbours (230 raw → 609 encoded). `keep:fast` and `keep:out` also pass the trailer, but they fail the synthetic text cases below, so they are not the answer. They pass here only because no trailer element's glyphs overflow its box by more than 3σ.

## Edge cases

The project is generated by `harness/gen_edge.py`: 26 cases, each sampled at 4 instants while its `x` drifts by a fractional amount. Each instant was rendered as a full-size PNG with `montagent frame --full --png` and compared pixel by pixel against `none`. Each cell gives instants identical (of 4), then max |Δ|.

| Case | `fast` | `k3` | `k4+2` | `keep:k0` (box only) | `keep:fast` | **`keep:pic`** |
|---|---|---|---|---|---|---|
| rect blur r40, centre | 4, 0 | 4, 0 | 4, 0 | 4, 0 | 4, 0 | **4, 0** |
| ellipse shadow offset (24, 18) r24 | 3, 4 | 3, 4 | 3, 8 | 4, 0 | 4, 0 | **4, 0** |
| rounded rect + stroke, shadow offset (7.5, −3.25) | 4, 0 | 4, 0 | 3, 4 | 4, 0 | 4, 0 | **4, 0** |
| ellipse, zero-offset glow r60 | 3, 8 | 3, 8 | 3, 8 | 4, 0 | 4, 0 | **4, 0** |
| text, zero-offset glow r28 | 4, 0 | 4, 0 | 4, 0 | 0, 192 | 4, 0 | **4, 0** |
| text blur, over the left frame edge | 4, 0 | 4, 0 | 4, 0 | 0, 6 | 4, 0 | **4, 0** |
| rect blur, over the right/bottom edges | 4, 0 | 4, 0 | 4, 0 | 4, 0 | 4, 0 | **4, 0** |
| rect off-frame, blur reaching in | 4, 0 | 4, 0 | 4, 0 | 4, 0 | 4, 0 | **4, 0** |
| off-frame rect, shadow offset into the frame | 4, 0 | 4, 0 | 4, 0 | 4, 0 | 4, 0 | **4, 0** |
| text, scale 2.5, blur r16 (σ under the CTM) | 4, 0 | 4, 0 | 4, 0 | 0, 5 | 4, 0 | **4, 0** |
| text, scale 0.4, glow r28 | 3, 2 | 3, 2 | 3, 2 | 0, 159 | 4, 0 | **4, 0** |
| text, animated scale 1.4 → 0.93, blur r14 | 2, 1 | 2, 1 | 2, 1 | 0, 7 | 4, 0 | **4, 0** |
| rect, anisotropic scale (2, 0.5), shadow offset | 4, 0 | 4, 0 | 4, 0 | 4, 0 | 4, 0 | **4, 0** |
| rect, rotated 30°, blur r24 | 0, 1 | 0, 1 | 0, 1 | 4, 0 | 4, 0 | **4, 0** |
| text overflowing a 60×40 box, glow r20 | 4, 0 | 4, 0 | 4, 0 | 0, 191 | 4, 0 | **4, 0** |
| text + stroke 8, glow r24 | 2, 2 | 2, 2 | 2, 2 | 0, 220 | 4, 0 | **4, 0** |
| ellipse, opacity 0.5, shadow offset | 4, 0 | 4, 0 | 4, 0 | 4, 0 | 4, 0 | **4, 0** |
| `[blur, shadow]` | 4, 0 | 4, 0 | 4, 0 | 4, 0 | 4, 0 | **4, 0** |
| `[shadow, blur]` | 4, 0 | 4, 0 | 4, 0 | 0, 57 | 4, 0 | **4, 0** |
| `[mask, blur]` | 3, 1 | 3, 1 | 1, 1 | 4, 0 | 4, 0 | **4, 0** |
| image (clip region), blur r20 | 4, 0 | 4, 0 | 4, 0 | 4, 0 | 4, 0 | **4, 0** |
| rect, blur r3 (σ 1.5: Gaussian pass, not box) | 4, 0 | 4, 0 | 4, 0 | 4, 0 | 4, 0 | **4, 0** |
| text overflowing its box, blur r2 | 0, 57 | 0, 57 | 4, 0 | 0, 110 | **0, 57** | **4, 0** |
| text + stroke 24, glow r3 | 0, 239 | 0, 239 | 0, 239 | 0, 239 | **0, 239** | **4, 0** |
| ellipse + stroke 20, blur r2 | 1, 2 | 1, 2 | 1, 2 | 4, 0 | 4, 0 | **4, 0** |
| text scale 3, blur r1, over the bottom-right corner | 4, 0 | 4, 0 | 4, 0 | 4, 0 | 4, 0 | **4, 0** |
| **Total identical** | 81/104 | 81/104 | 82/104 | 64/104 | 96/104 | **104/104** |

The full table, with `out`, `k3+1`, `k3+2`, `k4`, `k4+1`, `keep:out`, `keep:k3` and `pic` (four-sided, recorded bounds: 87/104), is in `results/edge.txt`.

**Large sigmas** come from `harness/gen_big.py` and are in `results/edge-large-sigma.txt`:

| Case | layer σ | `keep:pic` |
|---|---|---|
| rect blur r268 | 134 | 4/4, max 0 |
| rect blur r300 | 150 | 1/4, max 2 |
| text, scale 3, blur r100 | 150 | 0/4, max 2 |
| ellipse shadow r280, offset (40, 30) | 140 | 0/4, max 1 |
| rect blur r700 (the layer would also exceed `maxLayerDim`) | 350 | 0/4, max 2 |

## Why four-sided bounds fail

The prior spike tried the element box, then outsets of 8 and 64 px, then a hint stretched to the frame origin. All failed, which already said the extent was not the problem. 64 px is more than 3σ for every effect it tried. This sweep confirms it with a controlled pair:
- `fast` and `keep:fast` hint the same right and bottom edges and differ only in the top-left corner.
- `fast` fails 23 of 104 edge instants and 230 raw trailer frames; `keep:fast` fails no trailer frame. Its 8 edge failures are the separate content-bound problem in the next section.

The mechanism, in Skia's source (`rust-skia/skia` at `c9c3c5a9`, the commit `skia-bindings 0.153.2` builds):

1. **A hint is a hard clip on the layer, in layer space.** It is used whenever the restore is trivial, which it is here: no backdrop, no `kInitWithPrevious`, no blender ([SkCanvas.cpp:943–945](https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/src/core/SkCanvas.cpp#L943-L945)). The layer bounds are the filter's required input for the device clip, intersected with `roundOut(paramToLayer(hint))` ([SkCanvas.cpp:614–630](https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/src/core/SkCanvas.cpp#L614-L630)). One pixel of transparent padding is then added ([:986–1003](https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/src/core/SkCanvas.cpp#L986-L1003)).

   With no hint, the blur's required input is the device clip outset by `ceil(3σ)` ([SkBlurImageFilter.cpp:64–69, 217–224](https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/src/effects/imagefilters/SkBlurImageFilter.cpp#L64-L69)). So the unbounded layer's origin is about `(−ceil(3σ) − 1, −ceil(3σ) − 1)`, not the frame origin. That is why "stretch the hint back to the frame origin" still failed.

2. **The layer's top-left becomes a translation in the element's matrix.** The new device is created with `setDeviceCoordinateSystem(..., layerBounds.left(), layerBounds.top())` ([SkCanvas.cpp:1058–1064](https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/src/core/SkCanvas.cpp#L1058-L1064)). That call does `fLocalToDevice.postTranslate(-bufferOriginX, -bufferOriginY)` ([SkDevice.cpp:55–74](https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/src/core/SkDevice.cpp#L55-L74)).

   Every path point the element draws is then mapped as `fl(s·x + (t − left))` instead of `fl(s·x + (t − left₀))`. Those differ in the last bits whenever the magnitudes differ. The scan converter rounds those coordinates to sub-pixel precision, so occasionally one edge pixel's coverage flips by 1. #644's probe saw the same thing without any blur, with a no-op filter and integer translations (3–6 pixels on element edges).

3. The blur spreads each ±1 into a small patch, which is the "max |Δ| 1–8, a few dozen pixels" signature seen in every four-sided failure. A wider four-sided bound cannot help, because any bound tighter than the unbounded layer on the left or top moves the origin.

So the negative half of the answer is structural: **a byte-identical bound must not move the layer's top-left corner.**

## Derivation: why `keep:pic` is exact

Let *H* be the hint. Its left and top lie beyond the unbounded layer's left and top in layer space. Its right and bottom are Skia's recorded device bounds of the element's drawing, passed through the effect chain's `computeFastBounds` and mapped back to element space.

**Preconditions:**
- (P1) no perspective;
- (P2) layer-space σ ≤ 135 on both axes;
- (P3) the effects are Montagent's `blur` and `shadow`: `SkImageFilters::Blur`/`DropShadow` with the default `kDecal` tile mode and no crop. skia-safe defaults the tile mode to `Decal` (`image_filters.rs:136–150`);
- (P4) an RGBA8888 surface.

1. **Same layer matrix.** The layer/remainder split comes from `Mapping::decomposeCTM` ([SkImageFilterTypes.cpp:259–298](https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/src/core/SkImageFilterTypes.cpp#L259-L298)). The hint's centre, the "representative point", is used only for perspective ([:93–116](https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/src/core/SkImageFilterTypes.cpp#L93-L116)). Under P1 the mapping is therefore identical with or without the hint.

2. **Same origin, so the same matrix inside the layer.** The layer is the unbounded layer intersected with `roundOut(H)` (step 1 of the previous section). *H*'s left and top do not bind, so `layerBounds.left/top`, and therefore `fLocalToDevice`, are bit-identical to the unbounded case. Every coordinate the element's drawing computes is the same float.

   Pinning "left/top" means layer space. For a scale+translate matrix, layer space is device space. For a rotation, layer space is a positive scale of element space: `decomposeScale` takes vector lengths ([SkMatrix.cpp:1479–1499](https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/src/core/SkMatrix.cpp#L1479-L1499)). Pinning the element-space left/top is correct in both cases unless the scale is negative; see [What a shippable version must hold](#what-a-shippable-version-must-hold).

3. **Same content pixels.** *H* contains Skia's own conservative bounds of everything drawn. `SkRecordFillBounds` maps each op's geometry through the recorded matrix after `SkPaint::computeFastBounds`, which accounts for stroke width and joins ([SkRecordDraw.cpp:249–275, 539–546](https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/src/core/SkRecordDraw.cpp#L249-L275)). For a non-empty `fast`/`pic` chain the hint is that bound outset by the filter's 3σ or more.

   So the layer's right and bottom clip edges lie beyond everything the element draws. Every pixel kept is drawn with the same coordinates and the same clip on every side the geometry reaches. Every pixel dropped was transparent black in the unbounded layer. This is the one step resting on observed behaviour rather than a line-by-line reading of Skia's scan converter: a clip edge the geometry does not reach does not change coverage inside it. The trailer and 104 instants support it, and a CI guard should hold it.

4. **The blur is a pure, local function of a zero-extended source.** `Builder::blur` outsets the input's layer bounds by `ceil(3σ)` and intersects them with the desired output ([SkImageFilterTypes.cpp:2121–2135](https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/src/core/SkImageFilterTypes.cpp#L2121-L2135)). The desired output's left/top come from the device clip, so they are the same in both cases, and so is every intermediate image's origin.

   Under P2 the rescale is the identity (`sx = sy = 1`, [:2137–2149](https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/src/core/SkImageFilterTypes.cpp#L2137-L2149)). The raster engine picks `Raster8888BlurAlgorithm`, which is decal-only with `maxSigma` 135 ([SkBlurEngine.cpp:1204–1259](https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/src/core/SkBlurEngine.cpp#L1204-L1259)).

   `Pass::blur` writes zeros where the source has not started, runs the window over the source, then drains it with zeros ([:62–131](https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/src/core/SkBlurEngine.cpp#L62-L131)). It therefore computes the convolution of the zero-extended source. Both passes are pure functions of the source pixels in their window:
   - `ThreeBoxApproxPass` keeps exact `uint32` running sums and divides once ([:380–702](https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/src/core/SkBlurEngine.cpp#L380-L702)).
   - `GaussianPass` (σ < 2) re-convolves the window for every output ([:274–378](https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/src/core/SkBlurEngine.cpp#L274-L378)).

   The window's reach is `border = 3⌊(w−1)/2⌋` (odd *w*) or `3(w/2)−1` (even), with `w = ⌊1.88σ + 0.5⌋` ([:479](https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/src/core/SkBlurEngine.cpp#L479), [SkBlurEngine.h:89–92](https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/src/core/SkBlurEngine.h#L89-L92)). That is ≤ 2.82σ < `ceil(3σ)`. The Gaussian pass's radius is exactly `ceil(3σ)` ([SkBlurEngine.h:69–72](https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/src/core/SkBlurEngine.h#L69-L72)).

   So every output pixel the bounded layer produces equals the unbounded one. Every output pixel only the unbounded layer produces is more than a window away from any non-zero source pixel, so it is **exactly 0**. This is the "outside the bound is exactly zero" invariant: it comes from the kernel's support, not a tuned constant.

5. **Drop shadow.** A drop shadow is `Merge(MatrixTransform(translate, kLinear, ColorFilter(Blend SrcIn, Blur(src))), src)` ([SkDropShadowImageFilter.cpp:35–64](https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/src/effects/imagefilters/SkDropShadowImageFilter.cpp#L35-L64)). `SrcIn` with a colour maps transparent black to transparent black. The bilinear translate samples at positions fixed by the offset and the unchanged image origins. Merge composites the children. Each stage preserves "same values where both exist, zero elsewhere".

6. **Same composite.** The restore draws the result through the same layer-to-device mapping. Pixels present only in the unbounded result are transparent, and SrcOver with transparent black leaves the destination unchanged.

**The precondition is real, not decorative.** Above σ 135, `rescale()` resamples the visible layer bounds ([SkImageFilterTypes.cpp:1617–1700](https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/src/core/SkImageFilterTypes.cpp#L1617-L1700)), so the right and bottom edges change the resampling grid. The large-σ table shows it: σ 134 passes and σ 140–350 fail. P2 also keeps the unbounded layer inside `maxLayerDim` (`max(2·max(W,H), 2048)`, [SkCanvas.cpp:594–612](https://github.com/rust-skia/skia/blob/c9c3c5a91e9d74181a2ca34de81d78fef9b4d2b6/src/core/SkCanvas.cpp#L594-L612)) for an unrotated layer: `W + 2·ceil(3·135) + 2 ≤` that limit for any frame. Past that limit Skia would also rescale.

## Why the content bound must be Skia's recorded bounds, not the element box

`keep:fast`, `keep:k3` and `keep:out` start from the element's box (`extent`), and they pass the whole trailer. They fail two synthetic cases:
- text overflowing its box, with blur r2: max |Δ| 57;
- text with a 24 px stroke, with glow r3: max |Δ| 239.

A text element's glyphs, and a text stroke, can extend past the box. The 3σ outset hides that whenever σ is large, as in every trailer element, and stops hiding it when σ is small. `keep:k0` (the bare box, 64/104) shows the overflow directly.

A bound that passes the fixture by margin would be the tuned constant the issue ruled out. The recorded bounds remove the dependency on σ. For shapes, `keep:k0` passes all strokes tested, which suggests a shape's stroke stays inside its box. Text does not.

The recording costs one extra pass of the element's draw closure into an `SkPictureRecorder` with an RTree. That records commands only, with no rasterizing. The spike records under the real matrix (`set_matrix(local_to_device)`) so the closure's matrix-dependent choices, such as image sampling, match. It turned `in_element_space`/`through`'s `draw` from `FnOnce` into `Fn`, and every caller compiled unchanged.

## Timing

All timing is observed only, on the dev's M1 Pro, under other sessions' load. Two other sessions shared the machine throughout.

- **Per element:** a `P649` line is wall time from opening the element's first filter layer to its last restore. That covers the content draw, the filter and the composite, but not `keep:pic`'s recording.
- **Paint total:** `MONTAGENT_STAGES` `paint_ms` includes everything, recording too.

**Clean pair**, run back to back on the same binary as the last two runs of the session:

| | Load avg (start → end) | Elements | Σ filter-element time | Median / element | Paint total | Render wall |
|---|---|---|---|---|---|---|
| unbounded (`none_b`) | 7.3 → 4.0 | 3,485 | **176.7 s** | **50.8 ms** | 250.2 s | **258 s** |
| `keep:pic` (`keep_pic_2`) | 4.2 → 9.0 | 3,485 | **59.5 s** | **16.7 ms** | 135.6 s | **143 s** |

The first `keep:pic` run, under load 16 → 6, gave 57.5 s, 16.2 ms, 131.5 s and 140 s. That is the same picture.

Paint fell by 114.6 s while the timed filter time fell by 117.2 s. So the extra recording and the rest of paint add at most a few seconds over the whole trailer (≲ 1 ms per element), and that is within the noise.

By effect in the clean pair, as median ms per element, unbounded → `keep:pic`:

| Effect | Paints | Unbounded | `keep:pic` |
|---|---|---|---|
| glow r28 | 2,007 | 51.1 | 18.7 |
| glow r14 | 615 | 48.5 | 15.5 |
| glow r16 | 352 | 49.6 | 16.7 |
| blur r14 | 287 | 46.0 | 13.8 |
| glow r10 | 100 | 47.8 | 14.1 |
| glow r12 | 88 | 49.0 | 15.0 |
| glow r26 | 32 | 52.9 | 24.4 |
| glow r30 | 4 | 51.9 | 23.0 |

**Estimate for the trailer:** about **⅔ of the filter-layer time**, 117 of 177 CPU-s here. Scaled to the issue's ~232 CPU-s, that is roughly 150 CPU-s saved and ~80 s left.

The four-sided pixel-trading bounds (`fast`, `k3`) cost **2.1–2.5 ms** per element, 7.2–7.6 s in total. So the byte-identical bound keeps about 70% of the four-sided bound's saving. The loss is the price of pinning the top-left: a `keep` layer spans from the frame's top-left to the element's bottom-right plus ~3σ. A bottom-right element saves almost nothing, and a top-left element saves almost all.

Timings of the sweep's other runs, under load 11–29, are in `results/trailer-sweep.txt`. They are noisier and not comparable to each other.

## What a shippable version must hold

These are notes for whoever turns this into a change. They are not a design decision.

- Hint **only blur/shadow layers**, and only when (P2) layer σ ≤ 135 holds on both axes. Fall back to no hint otherwise. A checkable condition is `radius / 2 × max(|scale|) ≤ 135`.
- Pin the hint's **layer-space** left/top. For a scale+translate matrix with a negative scale (a flip), the element-space edge that maps to the layer's left is the right edge. The spike pins element-space left/top and does not test negative scales.
- The right/bottom must come from **Skia's recorded content bounds**, not `extent`. Text overflows its box.
- Colour-filter layers (`tint`, `brightness`, …) are out of scope:
  - `keep:pic` leaves them unhinted unless the element also has a blur/shadow, and then chains through them with `computeFastBounds`. That combination was not tested.
  - In the `keep:fast`/`keep:out` trailer runs the spike also hinted the colour-only elements, and those runs still passed. That is not a claim about them.
- A CI guard can hold the invariant as: render the trailer and the edge project with and without the hint, and require identical raw frames.

## Not settled

- **Negative `scale`, perspective, and skew** were not exercised.
- **Clip edges the geometry does not reach do not change coverage.** That is supported by every run here, but not read line by line in Skia's scan converter (derivation step 3).
- **Other machines.** All runs were on one M1 Pro. The bytes compared are within one machine and one Skia build, as golden frames already are.
- **A tighter right/bottom.** The hint is the filter's *output* bound. The *input*, Skia's recorded content bounds alone, would also be exact by step 4, since Skia outsets the blur's output past the layer by `ceil(3σ)` itself. It would save ~3σ more on two sides. Not measured.

## Reproduce

The spike code is on this branch: `crates/montagent-render/src/canvas.rs` (`mod spike649`, the hinted `through`) and `crates/montagent-core/src/verbs/render.rs` (`P649RAW`). It is not for merge.

    CARGO_TARGET_DIR=<scratch>/target cargo build --release -p montagent
    export MONTAGENT=<scratch>/target/release/montagent

    # trailer verdict (from the repo root; make score.wav first, see the fixture README)
    OUT=<scratch>/runs docs/research/filter-bound/harness/run_trailer.sh none
    OUT=<scratch>/runs docs/research/filter-bound/harness/run_trailer.sh keep:pic
    python3 docs/research/filter-bound/harness/summarize.py <scratch>/runs keep_pic

    # edge cases (in a scratch dir holding copies of the fixture's fonts/ and img/)
    python3 <repo>/docs/research/filter-bound/harness/gen_edge.py
    <repo>/docs/research/filter-bound/harness/run_edge.sh edge.montagent.json instants.tsv png none keep:pic fast
    uv run --with numpy --with pillow python <repo>/docs/research/filter-bound/harness/compare.py instants.tsv png keep_pic fast

`MONTAGENT_FILTER_BOUND` takes `none`, `fast`, `out`, `pic`, `k0`, `k3`, `k4`, each with an optional `+N` device-px slop and an optional `keep:` prefix.
