# Prototype #786: projection (ADR-0167), byte-identical across painters

A throwaway prototype for [#786](https://github.com/MBehtemam/Montagent/issues/786), testing
[ADR-0167](../../docs/adr/0167-an-element-may-be-projected-never-placed-swivel-tilt-and-perspective-join-the-transform.md).
It is evidence, not a starting point: the build carries none of this code.

**The run is byte-identical.** There were 291 renders of the 9 scenes, at 1 to 10 painters, with
chunks of 1, 2, 7 and 30 frames, and with the bounds hint on and off. Each run's raw RGB frames
were compared byte for byte against a one-painter, hint-on baseline, and **0 frames differ**.
That includes the projected card with blur, shadow and mask at 720p (81 runs) and at 1080p
(21 runs).

## The clips, for the owner's yes or no

All are in `clips/`.

1. **`01-card-flip`:** the front swivels 0 → 180 and the back −180 → 0. Each draws nothing past 90°.
2. **`02-door-swing`:** `origin: "center-left"`, with `swivel` going 0 → −75 → 0, in front of an
   unprojected frame.
3. **`03-tilted-screen`:** a `video` with `tilt` and `swivel` both keyed, plus a rounded mask. The
   only small video fixture is a green-screen test clip, so the screen is mostly green.
4. **`04-blur-shadow-mask`:** the effects `[mask, shadow, blur]` are drawn flat and then projected.
   The shadow and the rounded corners tilt with the card, which is ADR-0167 §3's named loss.
5. **`05-grain-directional`:** the same card with `[mask, shadow, grain, directional_blur]`.
6. **`06-text-and-path`:** the word SPY in Cinzel swivels, and a stroked star `path` tilts. Both are
   rasterised flat and then projected.
7. **`07-project-then-rotate`:** `swivel: 50` stays fixed while `rotation` goes 0 → 90 and `scale`
   goes 1 → [1.4, 0.8]. The trapezoid turns and stretches as a picture, so the projection applies
   first. A faint unprojected twin is shown for comparison.
8. **`08-motion-blur-flip`:** the card flip at 27° per frame under `motion_blur` (shutter 360, 16
   samples). Around 90° both cards blur toward transparent.
9. **`09-eye-bound`:** a 320×180 card about `center`, with `perspective: 184` against r = 183.58 and
   swivel going to ±70.

## How it was built (prototype only)

- **Fields.** `swivel`, `tilt` and `perspective` are `Option<Animatable<f64>>` on the six visual
  elements, and the schema was regenerated. The ADR-0146 animatable list is derived from the
  schema, so motion blur's "still" test sees the new fields with no other change.
- **Not built.** There are no `validate` codes (§8) and no `query --at` output (§7).
- **Where it branches.** `Painter::transform` builds `Transform.projection` when `perspective` and an
  angle are written. The painter branches only in `in_element_space`, after translate, rotate and
  scale. That branch is `projected()` in `canvas.rs`, and it runs in three steps:
  1. **Draws nothing when facing away, edge-on or too close.** Facing is decided in degree space,
     so exactly 90° counts as edge-on. A `perspective` at or below the eye bound also draws
     nothing, as a stand-in for `validate`'s refusal.
  2. **Draws the element flat.** It goes onto its own raster surface, which covers the
     reach-widened box plus 2 transparent pixels, rounded out to whole layer pixels. The surface
     holds `|scale|` layer pixels per element unit, under a matrix that is only a translation and
     a positive scale. The normal `Canvas::through` runs there, with effects and mask in list
     order.
  3. **Draws the snapshot through the projection.** The snapshot is drawn once through
     PROJECT · origin offset · 1/|scale|, sampled with `sampling_for` (ADR-0132) and antialiased.
     Opacity and blend wrap it, and `clip` stays outside everything.
- **The PROJECT matrix.** It is CSS `perspective(d) rotateX(tilt) rotateY(swivel)` with the z row
  dropped: `[[cos s, 0, 0], [sin s·sin t, cos t, 0], [sin s·cos t/d, −sin t/d, 1]]`. It is built in
  f64 and passed to Skia as an f32 `Matrix`.
- **Evidence hooks.**
  - A raw-bytes frame tap, `tap_frame_bytes`.
  - Process-wide counters: `proto786::{PROJECTED, AWAY, EYE_REFUSED, HINTED, FORCE_FLAT}`.
  - An `assert!` at the top of `Canvas::through` that the matrix has no perspective. It stays on in
    release builds.

## Byte-identity

The oracle is the full RGB bytes of every frame at the encoder's input, not a hash. Every run is
listed in `byte_identity.log`.

| Project | Frames | Runs | Settings | Result |
| --- | --- | --- | --- | --- |
| 04 blur+shadow+mask, 720p | 90 × 2,764,800 B | 81 | one painter with the hint off, plus K 1..10 × C {1,2,7,30} × hint on/off | 81/81 identical |
| 04 at 1080p (×1.5, perspective 1650) | 90 × 6,220,800 B | 21 | one painter with the hint off, plus K {1,2,3,6,10} × C {1,7} × hint on/off | 21/21 identical |
| Scenes 01–09, 720p | 90 each | 9 × 21 | one painter with the hint off, plus K {1,2,3,5,10} × C {1,7} × hint on/off | 189/189 identical |

The counters show that each path was actually exercised:

- **Bounds hint.** It bounded 180 layers per hint-on run of scene 04 (blur and shadow) and 90 in
  scene 05. Hint-off runs bounded 0.
- **Projected paints.** Scenes 03, 04, 05, 07 and 09 had 90 projected paints each, and scene 06 had
  180. Scenes 01 and 02 had 70 and 71, because frames at exactly 0° take the plain path. Scene 08
  had 96, counting motion-blur samples.
- **Facing away or edge-on.** Scene 01 had 91 such paints and scene 08 had 195.
- **Eye-bound refusals.** None.
- **Video.** The video scene was chunked only under `Forced::Chunks`, because the planner keeps a
  video project on one painter.

## No-op projection

- **Matches, by construction.** With `swivel: 0` and `tilt: 0`, `perspective` 400, 1100, 5000 and
  1e9 all match no projection byte for byte. This is not a measurement: the painter takes the plain
  path whenever both angles are 0. The test card also had rotation 12 and scale [1.1, 0.9], over 15
  frames.
- **Forced through the flat layer at 0°, it differs.** All 15 frames differ, in 3.6% of bytes, by at
  most 5 levels. That difference is the resample.
- **Consequence.** A keyed angle that passes through exactly 0 switches path for that one frame. It
  is still deterministic, and those frames are inside the sweeps for scenes 01 and 02.

## Matrix readers

- **Where they are called.** The bounds hint, `grain::plan` and `crop_directional` are called only
  from `Canvas::through`.
- **The guard.** `Canvas::through` now hard-asserts that the matrix has no perspective. The assert
  never tripped in any render or in the full test suite.
- **What runs under perspective.** On the projected path, `through` runs on the flat surface under
  translate · positive scale. The only draw under a perspective matrix is the single `draw_image`
  of the finished layer.

## The flat layer

- **Bounds.** The surface is the declared box widened by each effect's reach, summed in list order:
  - blur and glow reach ⌈3σ⌉, with σ = radius/2;
  - a shadow reaches its offset plus ⌈3σ⌉;
  - a directional blur reaches `directional_reach`;
  - mask, grain, colour members and chroma reach 0.

  The prototype adds a 2 px pad, rounds out to whole pixels and clips to the reach box. All of it
  depends only on the element at that instant.
- **Sampling.** `sampling_for` gives linear filtering with linear mipmaps under any perspective,
  because `min_scale()` is −1. That suits the receding side. The near side is magnified bilinearly
  with no cubic, which looks soft at strong foreshortening (clip 09).
- **Byte-identical:** yes, in every run above.

## Cost

These come from a MacBook Pro (MacBookPro18,3), 10 cores, at a load average of about 5.5 to 5.8.
The machine was not quiet; the load was 19 earlier. They are a short observed run, labelled as
such.

- **Method.** One element painted on one thread at 1920×1080: the median of 60 paints after 5
  warm-up paints, with the background clear subtracted. The element is a 960×600 `image` with swivel
  30, tilt 15 and perspective 1800.
- **Results, three runs each:**

| Element | Flat (ms per frame) | Projected (ms per frame) |
| --- | --- | --- |
| No effects | 6.45 / 6.69 / 6.47 | 10.82 / 11.18 / 10.83 |
| mask + shadow + blur | 75.67 / 77.47 / 76.20 | 48.90 / 48.63 / 49.41 |

- **Without effects,** projection adds about 4.4 ms per element per frame, about 1.7× the flat cost.
- **With effects,** the projected element is about 27 ms cheaper. The likely reason is that the flat
  surface bounds every effect layer, including the mask's layer, which is frame-sized on the plain
  path. This was not profiled.

## Corners

`projected_corners` gives the four corners of the reach-widened box, in the order TL, TR, BR, BL.
It projects them about `origin`, then applies scale, rotation and x/y in f64. The test painted a
white 400×250 rect on black and sampled 3 px inside and 3 px outside each corner. **All 20 samples
matched:** 255 inside and 0 outside.

| Case | TL | TR | BR | BL |
| --- | --- | --- | --- | --- |
| centre, swivel 35, tilt 20, d 900 | (463.41, 191.10) | (780.35, 292.98) | (792.79, 506.14) | (443.26, 453.94) |
| door at center-left, swivel −60, d 700 | (400.00, 235.00) | (795.94, 112.54) | (795.94, 607.46) | (400.00, 485.00) |
| top-left, tilt 50, d 800 | (300.00, 150.00) | (700.00, 150.00) | (825.89, 361.27) | (300.00, 361.27) |
| eye bound + 1 (r 235.8, d 236), swivel 40 | (303.47, 85.43) | (739.18, 279.08) | (739.18, 440.92) | (303.47, 634.57) |
| swivel 40, tilt −25, d 1000, then rotation 30 and scale [1.3, 0.8] | (460.74, 190.94) | (865.19, 344.45) | (767.34, 480.10) | (384.17, 377.67) |

The signs match the ADR. Positive swivel shortens the right edge. Positive tilt about the top-left
widens the bottom. The door at −60 grows its right edge.

## Guesses: what ADR-0167 does not settle

1. **The order of the two angles.** The prototype applies swivel first, then tilt (CSS
   `rotateX(t) rotateY(s)`). Facing, cos s · cos t, does not depend on the order, but the corners
   do when both angles are set.
2. **What "exactly edge-on" means.** It is decided in degrees: an angle ≡ 90 or 270 (mod 360) is
   edge-on. The element faces front when both angles are on the same side.
3. **Each effect's "declared reach".** The values used are the ones listed under the flat layer.
4. **Ink outside the reach box is cut.** This covers text that overflows its box. It makes §6's
   "the quadrilateral bounds everything painted" true.
5. **The flat layer's resolution** is `|scale|` layer pixels per element unit. The ADR does not say
   whether the layer follows `scale`.
6. **Angles at exactly 0 take the plain path.** So a keyed angle crossing 0 changes path for that
   one frame, by at most 5 levels.
7. **An angle without `perspective` paints flat.** `validate` refuses that in the build.
8. **The painter stands in for `E-PROJECTION-EYE`.** When `perspective` is at or below r, it paints
   nothing.
9. **The eye bound allows a lot of magnification.** At r + 1, clip 09's near edge is magnified about
   5× and leaves the frame. It looks soft because sampling is bilinear with mipmaps. The owner may
   want to decide whether to set a floor.
10. **Motion blur needed no change.** A sample that faces away is a transparent layer that is still
    added to the sum, because ADR-0155's accumulation counts every sample.

## Baseline test suite

`cargo test --workspace --no-fail-fast` on the branch: **1950 passed and 3 failed.** All 3 failures
are pinned lists of animatable properties, which now also contain `swivel`, `tilt` and
`perspective`:

- `tests/animatable.rs` `the_list_is_what_the_schema_types_as_animatable`
- `tests/path.rs` `points_is_on_the_one_list_as_a_whole_list_value_and_the_box_is_not`
- `tests/resolve.rs` `every_animatable_property_the_schema_publishes_is_one_the_view_resolves`. The
  schema publishes the new fields, but the `query` view does not resolve them, because §7 was not
  built.

These are expected results of adding the fields, not regressions in untouched behaviour, and they
are left unfixed. Every other test passes, including `filter_bound.rs`, the painter tests, the
`schema.rs` committed-schema check and the motion-blur tests.

## Reproduce

```
python3 prototype/projection-786/make_scenes.py
cargo test --release -p montagent-core --test projection_786 -- --ignored --nocapture --test-threads 1
```

All builds used a private `CARGO_TARGET_DIR`, not the shared target. The committed scene projects
are the formatted versions, except `09-eye-bound`, which was regenerated after the last `fmt` and
may differ in key layout only.
