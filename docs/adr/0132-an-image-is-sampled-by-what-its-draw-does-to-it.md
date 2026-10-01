---
status: accepted
amends: 0010 (the resampler is a decision of its own, and the golden-frame guard gains two frames that can see it)
---

# An image is sampled by what its draw does to it

[#500](https://github.com/MBehtemam/Montagent/issues/500). The cut-out character prototype
(#487) enlarged its parts through `scale` — the background to 2.37×, the character to
1.38× — and Montagent drew their outlines stair-stepped where a PIL reference (a Lanczos
resize, then a bicubic affine) drew them smooth: 27–28 dB PSNR apart in the close-up
frames, 33 dB median in the wide ones.

Geometry was not the cause. Every image went through one sampling choice, bilinear with no
mipmaps, whose only stated reason was a code comment: *"the prototype's sampling, kept
because the golden frames the oracle guards were measured with it (ADR-0010)."* ADR-0010
asks for a golden-frame guard because *"a `skia-safe` bump can quietly shift resampling"*.
It does not choose a filter, and a guard that resists change is not a reason to keep the
thing it guards.

The report's own hypothesis, that the source is resampled twice, was checked and is false.
The painter composes the declared rect, `x`/`y`, `rotation`, `scale` and `origin` into one
matrix and reads the source once. The reference was the one that sampled twice.

## What was measured

The test was a 200 px RGBA cut-out (a circle, a quad and a 3 px diagonal) on skia-safe
0.153.2. PSNR was taken over edge pixels:

| Sampling | ×2.3, against PIL Lanczos | ×0.3, against the analytic shape |
|---|---|---|
| bilinear, no mipmaps (until now) | 32.3 dB | 31.2 dB |
| bilinear, linear mipmaps | 32.3 | **38.2** |
| cubic, Mitchell | 34.2 | 30.4 |
| cubic, Catmull-Rom | **40.2** | 29.1 |
| PIL bicubic, #500's reference | 40.0 | 38.3 |

**No one filter wins in both directions.** Skia's cubics ignore mipmaps, so they alias more
than bilinear when they shrink. Mipmaps do nothing when the image is enlarged. #500 reported
only enlargement, but the prototype's wide shots shrink the toast to 0.30× and the
character to 0.58×, and that is the same defect at the same call.

On a loaded machine, a 1080p full-screen layer costs about 33 ms under either cubic
whatever its scale, against about 10 ms bilinear at 1.4–2.4×. A Ken Burns test frame goes
from 15 ms to 54 ms, which is well inside ADR-0021's 500 ms. At exactly 1:1 with a
whole-pixel offset, bilinear and Catmull-Rom both take Skia's pass-through (1.5 ms).
Mitchell does not, because it blurs at 1:1.

## The decision

### 1. One rule, read off the draw's matrix

The renderer chooses the sampling for every raster draw from **the matrix that takes source
pixels to device pixels**. That matrix already contains the declared rect, the element's
transform and a proxy canvas's base scale. The rule has three branches, applied in order:

1. **Identity: nearest.** The matrix has no scale, no skew and no perspective, and its
   translation is a whole number of pixels, each within float tolerance (1e-6 on the linear
   part, 1e-3 px on the translation). The source lands pixel for pixel.
2. **Minification: bilinear over linear mipmaps.** The matrix's smaller singular value is
   under 1. Either axis counts, so a source stretched 2× wide and squeezed to half height
   minifies. A perspective matrix, which nothing in Montagent builds, lands here too, as
   the branch that cannot alias.
3. **Magnification, everything else: cubic Catmull-Rom**, `B = 0, C = ½`.

**Identity is a fact about the matrix, never about `scale`.** `scale: 1` over a declared
rect that is not the source's native size is a real resample, and a fractional offset at
scale 1 is too. The identity branch is stated rather than left to Skia's pass-through for
two reasons. The pass-through is an implementation detail a `skia-safe` bump could move,
and losing it would make every unscaled layer 22× slower without a golden noticing. And
the branch keeps 1:1 exact under any future choice of magnifier.

The encode-time downscale that `frame --scale` and `preview` answer through runs the same
rule. Since it only ever shrinks, it reads through mipmaps. `Canvas::composite` (the
contact sheet's tile, ADR-0095 §5) keeps the mipmapped sampling it already had, which is
the same answer this rule reaches.

### 2. The choice is the renderer's, not the format's

There is no schema field, no project setting and no per-element option. The format states
*what* is drawn (the declared rect is authoritative, ADR-0013/0015), not how pixels are
interpolated. A filter name in the format would bind the document to one rasterizer's
spelling of it. Pixel art that must stay crisp is a real need with no request behind it
yet. If one arrives it is an additive field, decided on its own ticket.

### 3. The golden guard is blind to this, so two frames are built to see it

ADR-0010's guard cannot tell the filters apart. Under this change, five of the eight goldens
moved, by a mean channel delta of 0.014 to 0.20, and all eight passed. A regression that
quietly restored bilinear would have passed too. Three things guard the rule:

- **A unit test of the rule itself.** It covers identity, float drift, a fractional offset,
  a declared rect off native size, rotation, one axis under 1, uniform scales of 0.3, 0.999,
  1, 1.001 and 2.3, and perspective.
- **An identity draw read back byte for byte** through the real element path.
- **Two purpose-built goldens held to a mean delta of 0.25.** `sampling-enlarged` is a
  dark-outlined cut-out at ×2.3, the contrast where Catmull-Rom's overshoot would show as a
  halo; it was looked at and shows none. `sampling-shrunk` is rings and hatching at ×0.3.
  Measured against them when they were committed:

  | Painted with | `sampling-enlarged` | `sampling-shrunk` |
  |---|---|---|
  | the rule (committed) | 0 | 0 |
  | bilinear, no mipmaps | 0.79 (SSIM 0.9965) | 8.30 |
  | Mitchell at and above 1:1 | 0.73 | 3.26 |
  | Catmull-Rom everywhere | 0 | 10.04 |

The five goldens that moved were regenerated and looked at. Their difference is confined to
the edges of the scaled logo, and the photograph and text are unchanged.

## Considered and rejected

- **Magnification only, minification filed separately.** It leaves half the measured gap
  and would regenerate the same goldens twice.
- **One filter for both directions.** The table above rules it out.
- **Mitchell.** It rings less than Catmull-Rom, but it gains 2 dB where Catmull-Rom gains 8,
  and it blurs at 1:1.
- **Lanczos through a runtime shader.** It is new surface to maintain, its cost was not
  measured, and it would beat a result that already matches the reference #500 holds up.
- **Tightening every golden's threshold.** It would fail every `skia-safe` bump for reasons
  unrelated to sampling, and still could not separate filters at a delta of 0.14.

Each part was put to a three-model court during #500's triage, and the panel agreed on
every one. Its one departure from the triage recommendation, on the identity branch, is
the reading adopted here.

## Consequences

- Enlarged images match a bicubic reference, and shrunk ones stop aliasing.
- A frame with full-screen enlarged images costs about 33 ms more per such layer at 1080p,
  and an estimated 4× that at 4K (not measured). Over a long render that is real wall clock:
  about 11 s more on a 10 s 30 fps Ken Burns. `frame` stays far inside its budget.
- Catmull-Rom can ring on a very high-contrast edge. The enlarged golden is the place that
  would show it. If a real asset shows it, the remedy is raised on #500 and not as a
  per-layer fallback, which section 2 rules out.
- If a `skia-safe` bump changes what Catmull-Rom or the mipmap chain computes, the two
  sampling goldens are where it fails.
