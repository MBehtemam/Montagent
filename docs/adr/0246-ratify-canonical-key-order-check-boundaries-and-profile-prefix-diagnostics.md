---
status: accepted
amends: 0007 (a run's highlight window is paint-only and moves no glyph), 0011 (ratifies query --at JSON shape, frame --out/JPEG_QUALITY/opaque-black default background, frame.transitions, and render.profile.prefix_ended_by), 0012 (ratifies translate -> rotate -> scale transform order and element-level opacity layer), 0014 (ratifies stroke, stroke_width, radius canonical order on rect and radius on rect alone), 0015 (ratifies SAR as an input to fit arithmetic rather than raw source blit), 0019 (ratifies E-ANCHOR-MISSING, E-ANCHOR-SELF, E-ANCHOR-CHAIN, and R-ANCHOR-DISJOINT), 0033 (ratifies single-finding-per-cut detail list on R-SOURCE-CUT-POP), 0035 (ratifies still-image exclusion and >=1 whole-frame threshold on R-SOURCE-UNREACHED), 0040 (ratifies 2*sigma blur/shadow radius, opacity outside effects, and multiplicative shadow.opacity), 0041 (ratifies layer at the tail of the universal element prefix id, type, group, start, end, layer), 0044 (ratifies frame-grid sampling and rotated-element exclusion on R-OFF-CANVAS), 0059 (ratifies linear crossfade ramp), 0060 (ratifies U-LAYER-TIE-ROTATED at unchecked, same-track deferral to E-TRACK-OVERLAP, and file-level pair location), 0062 (ratifies single-element loop wrap on R-SOURCE-CUT-POP), 0194 (render.profile gains prefix_ended_by), 0195 (render.profile.prefix_ended_by names which element ended the static prefix, why, at what layer, and on how many frames)
---

# Ratify canonical key order, `query --at` and `frame` readings, check boundary rules, and `render.profile.prefix_ended_by`

Ratifies the implementation readings recorded in [#243](https://github.com/MBehtemam/Montagent/issues/243),
[#269](https://github.com/MBehtemam/Montagent/issues/269),
[#274](https://github.com/MBehtemam/Montagent/issues/274),
[#280](https://github.com/MBehtemam/Montagent/issues/280),
[#282](https://github.com/MBehtemam/Montagent/issues/282), and
[#285](https://github.com/MBehtemam/Montagent/issues/285) (Following [ADR-0077](0077-the-nine-render-readings-are-ratified.md)'s
precedent), together with `render.profile.prefix_ended_by` ([#975](https://github.com/MBehtemam/Montagent/issues/975)).

---

## Decisions

### 1. Anchor check codes and `layer`'s canonical prefix position ([#243](https://github.com/MBehtemam/Montagent/issues/243), amends [ADR-0019](0019-layer-anchor-gets-an-id-a-validate-check-and-one-hop.md) and [ADR-0041](0041-canonical-key-order-is-schema-order-validate-checks-it-fmt-splits.md))

1. **Three `error` codes and one `review` code:** Because [ADR-0043](0043-refuse-class-findings-a-repair-field-uniform-per-check-non-bypassable.md)
   fixes repair form per code, anchor validation emits three distinct `error` codes (`E-ANCHOR-MISSING` with `Advise`,
   `E-ANCHOR-SELF` with `Refuse`, and `E-ANCHOR-CHAIN` with `Refuse`) and `R-ANCHOR-DISJOINT` at `review`.
2. **`layer` at the tail of the universal element prefix:** Every element's canonical key prefix is
   `id, type, group, start, end, layer`, followed by its per-type fields.

### 2. `query --at` answer shape and resolution rules ([#269](https://github.com/MBehtemam/Montagent/issues/269), amends [ADR-0011](0011-tool-surface-reads-checks-renders.md))

1. **Declared properties only, no defaults:** `present.values` lists only animated properties the element explicitly writes, in
   schema declaration order, without fabricating schema defaults.
2. **One-hop layer resolution:** `layer` carries the resolved integer layer and `layer_unresolved` carries `null` (or `layer` is
   `null` and `layer_unresolved` names why the anchor did not resolve). Unplaced elements sit in `unplaced`.
3. **Source offset and coverage:** `source_offset` and `source_origin` report media offset at the queried instant; `not_covered`
   and `not_covered_unresolved` report cross-track group coverage.

### 3. `frame` Phase 1 and Phase 2 rendering readings ([#274](https://github.com/MBehtemam/Montagent/issues/274), [#280](https://github.com/MBehtemam/Montagent/issues/280), amends [ADR-0007](0007-text-runs-literal-size-declared-fonts.md), [ADR-0011](0011-tool-surface-reads-checks-renders.md), [ADR-0012](0012-flat-transform-keyframes-carried-by-their-element.md), [ADR-0014](0014-stroke-is-paint-the-text-box-is-required.md), [ADR-0015](0015-fit-is-a-derivation-claim-and-gravity-retires.md), [ADR-0040](0040-effect-model-attachment-and-v1-vocabulary.md), [ADR-0059](0059-transitions-element-type-crossfade-only-exact-window.md))

1. **CLI `--out`, JPEG quality, and default background (#274):** `montagent frame` requires `--out <PATH>` on the CLI; JPEG
   encoding uses `JPEG_QUALITY` (`85`); a project with no `background` (and `alpha: false`) clears to opaque black (`#000000FF`).
2. **Transform order and element-level `opacity` (#274, #280):** An element's 2D transform applies `translate → rotate → scale`
   about `origin`. `opacity` composites the finished element — including its `effects` (such as `shadow`) — once via a single
   layer outside the effects stack.
3. **`rect` and `ellipse` field order (#274):** `rect`'s canonical tail after `fill` is `stroke, stroke_width, radius`; `radius`
   belongs to `rect` alone and is not inherited by `ellipse`.
4. **SAR in `fit` (#274):** Sample aspect ratio (SAR/PAR) is an input to `fit` derivation arithmetic in `validate`/`measure`, not
   a separate resampling stage during raw source decode.
5. **Gaussian `radius`, `shadow.opacity`, linear `crossfade`, paint-only `highlight`, and `frame.transitions` (#280):**
   - `blur.radius` and `shadow.radius` follow the CSS convention `radius = 2σ` (`σ = radius / 2`).
   - `shadow.opacity` multiplies the alpha channel of `shadow.color`.
   - A `crossfade` transition's opacity ramp is linear so the two bridged elements sum to 1 at every instant.
   - A run's `highlight` window is paint-only and never alters glyph shaping or placement (`a_highlight_moves_no_glyph`).
   - `frame`'s JSON answer includes `transitions`, listing every transition active at `at` with its window and progress.

### 4. `E-LAYER-TIE` and `U-LAYER-TIE-ROTATED` boundaries ([#282](https://github.com/MBehtemam/Montagent/issues/282), amends [ADR-0060](0060-layer-tie-is-an-error-array-order-stays-meaningless.md))

1. **Rotated pairs emit `U-LAYER-TIE-ROTATED` (`unchecked`):** When two elements share a resolved layer and overlap in time and at
   least one has a non-zero `rotation` (`swivel`/`tilt` included), `validate` emits `U-LAYER-TIE-ROTATED` (`unchecked`) per pair
   without a bounding-box pre-filter.
2. **Same-track overlap and pair location:** Same-track overlapping elements are left to `E-TRACK-OVERLAP` rather than double-reported
   as a layer tie; cross-track `E-LAYER-TIE` and `U-LAYER-TIE-ROTATED` findings carry file-level location naming both element IDs.

### 5. `R-OFF-CANVAS`, `R-SOURCE-UNREACHED`, and `R-SOURCE-CUT-POP` boundaries ([#285](https://github.com/MBehtemam/Montagent/issues/285), amends [ADR-0033](0033-same-source-cut-continuity-is-a-review-check.md), [ADR-0035](0035-keyframe-grid-alignment-is-a-review-check-not-a-schema-rule.md), [ADR-0044](0044-off-canvas-is-a-standing-review-check-not-a-frame-change-census.md), [ADR-0062](0062-loop-declares-a-boolean-wrap-r-source-cut-pop-extends-mechanically.md))

1. **`R-OFF-CANVAS`:** Evaluates unrotated elements across the frame grid of their active range and skips rotated elements rather
   than approximating rotated bounding boxes.
2. **`R-SOURCE-UNREACHED`:** Applies to time-bearing media (`video` and `audio`, never still `image` elements) and fires only when
   at least one whole frame (`1000 / fps` ms) of the source after `trim.end` is unreached.
3. **`R-SOURCE-CUT-POP`:** Emits one finding per cut carrying all discontinuous properties in `detail`, and under `loop: true`
   checks a single full-duration element on a track wrapping onto itself.

### 6. `render.profile.prefix_ended_by` ([#975](https://github.com/MBehtemam/Montagent/issues/975), amends [ADR-0194](0194-render-profile-says-where-the-painters-cpu-went-by-element-type-and-effect.md) and [ADR-0195](0195-time-invariant-bottom-layers-share-a-byte-identical-prefix-composite.md))

`render.profile` gains `prefix_ended_by`, an array of `{id, type, layer, reason, frames}` objects ordered by `frames` descending
then `id`, recording for each frame which drawn element first stopped the bottom-up static prefix and why (`video`,
`running transition`, `keyframe on <path>`, `blend: <mode>`, `motion_blur`, `group`, `units`, `effect: <name>`, `highlight`, or
`non-invariant nest child`).
