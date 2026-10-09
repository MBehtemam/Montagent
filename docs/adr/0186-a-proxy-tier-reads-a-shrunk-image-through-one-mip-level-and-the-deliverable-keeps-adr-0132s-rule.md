---
status: accepted
amends: 0132 (its sampling rule is the deliverable's — `render`, `frame` and `preview --full` — and a proxy tier reads a shrunk image through one mip level instead of two), 0065 (its 2.68 s / 3.78 s were measured with bilinear sampling and no mipmaps, which no shipped path uses, and they have not been re-measured under the proxy's sampling)
---

# A proxy tier reads a shrunk image through one mip level, and the deliverable keeps ADR-0132's rule

[ADR-0132](0132-an-image-is-sampled-by-what-its-draw-does-to-it.md) (#500, landed in #552)
chose one sampling rule for every raster draw. Shrunk images get bilinear sampling over
linear mipmaps, which is trilinear. Enlarged images get Catmull-Rom. The rule reads the matrix
that takes source pixels to device pixels, and a proxy canvas's base scale is part of that
matrix. So `preview` read through trilinear too.

`preview_budget`'s 10 s scrub of the committed fixture then failed on every CI leg, at
5.0–5.6 s against the enforced `<5 s`
([ADR-0021](0021-preview-budget-and-graceful-degradation.md),
[ADR-0065](0065-preview-proxy-target-720p-540p-floor-disclosed-not-certified.md)). A bisect
found the cause in commit `398cfdd9`. The fixture's 1536×2720 stills land at about 0.47× on
the 720p proxy of its 1080×1920 frame, so every frame takes the minification branch. On a
development sandbox the whole scrub went from about 5.6 s to about 9.0 s. Building the mip
chain once at decode time did not help. The cost is Skia's CPU trilinear read, not building
the mipmaps.

ADR-0132 measured `frame` (a Ken Burns frame went from 15 ms to 54 ms) and noted the
wall-clock cost to `render`. It never measured `preview`, which is the one path with an
enforced wall clock.

## The decision

**ADR-0132's sampling rule belongs to the deliverable.** `render`, `frame` (both its default
half-scale answer and `--full`) and `preview --full` paint true pixels. Their caller asked for
that quality, and their sampling does not change: nearest at identity, trilinear when
shrinking, Catmull-Rom when enlarging.

**A proxy tier may trade the minifier for the scrub budget.** On `preview`'s 720p and 540p
rungs, where the cap engaged, a shrunk image is read **bilinear within the nearest mip level**
(`MipmapMode::Nearest`), not blended across two levels. The identity and magnification
branches are the same as on true pixels. A proxy is already disclosed as not true pixels
(ADR-0021, ADR-0065), and the softness this adds is the same kind of softness the disclosure
already covers.

**The proxy is stated by the caller and never inferred from a scale.** `preview` marks its
surface as a proxy exactly where the tier's cap engaged. A `--full` run, or a project that is
already inside the 720p cap, is true pixels and samples like `render`. The canvas carries
this as its fidelity: `Canvas::scaled` (the proxy surface, which only `preview` uses) is a
proxy, `Canvas::new` is not, and a motion-blur layer takes the fidelity of the canvas it is
composited onto. A smaller surface is not a proxy because it is smaller. A true-pixel canvas
whose elements happen to shrink still reads them through trilinear.

## What was measured

All numbers below are release builds with ffmpeg 7.1.5 on a 4-core Linux sandbox. That machine
is noisy and roughly 1.5–2× slower than the CI runners, which is how CI recorded 5.0–5.6 s
where the sandbox recorded about 10 s. Each comparison was interleaved within a round, so drift
on the machine hits every variant alike.

The wall-clock figures were taken with a temporary environment switch over the proxy's
minifier that is not committed. They are a record of what was measured, not a check anyone can
re-run, and `preview_budget` on CI is the check. The quality figures are re-derived by the
committed unit test named below.

**The scrub preview.** This is `preview` over 0–10 s of the fixture with a fresh probe sidecar,
the same work `preview_budget` measures. The 720p column is the wall time of the 720p attempt
as `preview` reports it. "Missed" means that attempt reached the 5 s deadline and the verb went
on to 540p.

| Proxy minifier | 720p attempt, 4 rounds | 540p attempt where reached |
|---|---|---|
| trilinear (ADR-0132 as shipped) | 5.00–5.04 s, missed every time | 4.63–4.90 s |
| **one mip level (adopted)** | 4.33, 4.68, 4.86 s; one missed at 5.16 s | 3.08 s |
| bilinear, no mipmaps | 4.84 s; three missed at 5.22–5.43 s | 3.07–3.49 s |

The 540p attempt is the cleaner gauge, because it runs to completion. Trilinear costs about
1.5 s more there than either cheaper reader, and the two cheaper readers cannot be told apart
on this machine. Since no mipmaps buys no measurable time, the reader that keeps the mipmaps
is adopted. Run through `preview_budget` itself on this sandbox, the adopted reader held the
720p tier and measured 5.10, 5.25 and 5.29 s end to end. Before #552 the same sandbox
measured about 5.6 s. The CI runners decide whether the budget holds. If they are as much
faster as they were on the trilinear numbers, this lands near half the budget.

**Quality.** On ADR-0132's own golden, `sampling-shrunk` (rings and hatching at ×0.3), one mip
level measured a mean channel delta of **2.54** during the bisect. No mipmaps measured 8.30, and the golden's
ceiling, which holds the true-pixel path, is 0.25. That golden still renders through `frame`
and still passes unchanged. A unit test,
`a_raster_shrunk_onto_a_proxy_reads_one_mip_level_and_stays_near_true_pixels` in
`crates/montagent-render/src/canvas.rs`, paints the same kind of source at ×0.3 two ways: once
through a proxy's base scale, and once through an element `scale` on a true-pixel canvas. The
device matrix is the same, so only the fidelity differs. The proxy read measures **3.58** from
the true-pixel read over all four channels, and no mipmaps measures 11.63. The test holds the
proxy under 4.5, and requires it to stay under half of no mipmaps' figure.

**`frame`, for comparison.** True pixels do not change here. To answer whether `frame_budget`'s
misses (586 ms and 612 ms against 500 ms, on the x86_64 macOS leg only) share this cause, the
true-pixel minifier was varied experimentally over three interleaved rounds of the budget's
own `frame --at 11000`. Trilinear took 344–388 ms, one mip level 304–357 ms, and no mipmaps
313–359 ms. Trilinear's share of a cold `frame` is about 30–40 ms, roughly a tenth. That is
too small to be the whole of an 86–112 ms miss. It may contribute, and it is not this ADR's to
trade: `frame` is the deliverable's picture.

## Considered and rejected

- **One mip level everywhere.** It fails ADR-0132's `sampling-shrunk` golden (2.54 against
  0.25) on the paths whose callers asked for true pixels. That is the quality ADR-0132 bought,
  and #552's goldens exist to hold it.
- **Bilinear with no mipmaps on the proxy.** It was no faster than one mip level in these
  measurements, and it aliases about three times as badly. It remains the fallback if CI shows
  one mip level cannot hold the budget.
- **Inferring the proxy from the matrix's scale.** The base scale is in the matrix, but so is
  every element's own `scale`. A rule that guessed from the number would also degrade a
  true-pixel frame whose element is shrunk.
- **Building the mip chain once at decode.** Measured during the bisect. It did not move the
  wall clock, because the cost is in the per-draw trilinear read.
- **Raising `SCRUB_PREVIEW_LIMIT`.** The `<5 s` is ADR-0021's budget, and nothing measured
  here argues that it should move.

## ADR-0065's numbers

ADR-0065's **2.68 s (4K) and 3.78 s (8K)** at the 720p target came from #87's prototype
harness, `rast-bench`. That harness samples every still with **bilinear and no mipmaps**
(`docs/research/prototypes/rust-rasterizer/src/skia_arm.rs`, `sampling()`). No shipped path
has used that reader since ADR-0132. They were not re-measured for this ADR. Doing so needs
the prototype's separate Skia build and its generated 4K/8K media, and this decision does not
depend on them. Two things are known. The proxy now uses one mip level, which on the
committed fixture costs no more than bilinear with no mipmaps. And at 540p it costs about
1.5 s less than trilinear, which is the reader those two numbers would otherwise have been
quietly running under. Until someone re-measures, the two numbers describe the prototype's
reader and not the shipped one.

## Consequences

- `preview` on a proxy tier is softer on strongly shrunk detail than `frame` is: about 2.5
  mean-delta units on ADR-0132's hardest frame. The tier disclosure already tells the caller
  this frame is not true pixels.
- `render`, `frame` and `preview --full` are unchanged, byte for byte. All twelve goldens
  pass, including both of ADR-0132's sampling goldens.
- If a later proxy still misses the budget, the next lever is bilinear with no mipmaps on the
  proxy alone. Measure it on CI first, since this sandbox could not separate it from one mip
  level.
