---
status: accepted
amends: 0149 (admits sweep and two_point_conical to the Gradient enum with required center/angle/stops and center/radius/focal/stops in declared-box fractions; extends animatable gradient paths with fill.focal, stroke.focal, and color.focal; extends R-GRADIENT-ONE-COLOUR to sweep and two_point_conical), 0215 (disposes of the Angle Gradient Overlay, Radial Highlight Length/Angle, and 4-Color Gradient census rows on map #896)
---

# Sweep and two_point_conical enter gradient, and 4-Color Gradient is refused

[Do sweep, two_point_conical and 4-Color gradients enter under ADR-0149 and ADR-0215?](https://github.com/MBehtemam/Montagent/issues/1065)
on the After Effects map ([#896](https://github.com/MBehtemam/Montagent/issues/896)).

[ADR-0149](0149-a-gradient-is-a-paint-linear-or-radial-measured-against-the-declared-box.md)
introduced `Gradient` as a `Paint` variant on shape `fill`/`stroke` (`rect`, `ellipse`, and `path` per
[ADR-0154](0154-a-point-list-enters-as-one-path-element-in-integer-pixels-from-the-declared-box.md))
and `text` element-level `color`/`stroke`, admitting `linear` and `radial` while holding out `sweep`
and `two_point_conical` solely because neither was documented in CapCut or Premiere.
[ADR-0215](0215-after-effects-is-precedent-for-rendered-results-and-capcut-and-premiere-stay-first-for-shape.md)
§3 lifted that reference-class bar after the After Effects censuses confirmed both rendered results:
- **Sweep (conical / angle) gradient**: Layer Styles' *Gradient Overlay* (`Style: Angle`,
  [#898](https://github.com/MBehtemam/Montagent/issues/898) row 116);
- **Focal-point radial gradient (`two_point_conical`)**: shape-layer *Gradient Fill* and *Gradient Stroke*
  (*Highlight Length* and *Highlight Angle*, [#901](https://github.com/MBehtemam/Montagent/issues/901) row 15);
- **Four-corner colour blend (*Generate → 4-Color Gradient*)**: four `[x, y]` points with four colours blended
  across the frame ([#905](https://github.com/MBehtemam/Montagent/issues/905) row 11).

This ADR decides:
1. Which of the three After Effects gradient capabilities enter `Gradient`, and why *4-Color Gradient* is refused.
2. The schema, geometry, and focal-clamping rules for `sweep` and `two_point_conical`.
3. Animation, overshoot, `validate` (`R-GRADIENT-ONE-COLOUR`), tool integration, and the Skia byte-identity gate.

---

## Decisions

### 1. `sweep` and `two_point_conical` enter `Gradient`; *4-Color Gradient* is refused

1. **`sweep` and `two_point_conical` join the `Gradient` enum** alongside `linear` and `radial` on every paint field
   (`rect`, `ellipse`, and `path` `fill` and `stroke`, and `text` element-level `color` and `stroke`):

   ```json
   "fill": {
     "gradient": "sweep",
     "center": [0.5, 0.5],
     "angle": 0,
     "stops": [
       {"offset": 0, "color": "#FF3366"},
       {"offset": 1, "color": "#3366FF"}
     ]
   }
   ```

   ```json
   "fill": {
     "gradient": "two_point_conical",
     "center": [0.5, 0.5],
     "radius": 1,
     "focal": [0.3, 0.25],
     "stops": [
       {"offset": 0, "color": "#FFFFFF"},
       {"offset": 1, "color": "#2040A0"}
     ]
   }
   ```

   Every parameter on each variant is required ([ADR-0149](0149-a-gradient-is-a-paint-linear-or-radial-measured-against-the-declared-box.md) §1):
   - `sweep` requires `center`, `angle`, and `stops`, and refuses `radius` and `focal`.
   - `two_point_conical` requires `center`, `radius`, `focal`, and `stops`, and refuses `angle`.

2. **Four-corner colour blend (*4-Color Gradient*) is refused from both `Gradient` and `effects[]`:**
   - **Not a 1D stop shader:** *4-Color Gradient* blends four 2D points via inverse-distance weighting (`Point 1..4`,
     `Color 1..4`, `Blend`, `Jitter`) rather than evaluating a monotonic `0..1` stop list along a line, radius, or angle.
   - **No native `SkGradientShader`:** `skia_safe::gradient::shaders` provides `linear_gradient`, `radial_gradient`,
     `two_point_conical_gradient`, and `sweep_gradient` directly in Skia's raster pipeline; it has no four-point IDW
     gradient shader.
   - **Violates ADR-0049 and ADR-0149 in `effects[]`:** [ADR-0149](0149-a-gradient-is-a-paint-linear-or-radial-measured-against-the-declared-box.md)
     refuses gradient effects because a paint belongs on `fill`/`stroke`/`color`, and [ADR-0049](0049-v1-colour-filter-vocabulary-four-scalar-members.md)
     limits colour effects to bounded scalar grading operations.
   - **Sanctioned routes:**
     1. **In-project composition recipe (`skills/montagent-motion/SKILL.md`):** Multi-corner ambient colour washes are
        authored in `project.json` by stacking `rect` elements with `radial` (or `two_point_conical`) gradients whose
        outer stop fades to transparent (`#RRGGBB00`) or composites with `blend: "screen"` / `"add"`.
     2. **`montagent-prerender`:** When exact inverse-distance or mesh interpolation is required, the plate is generated
        via `montagent-prerender` ([ADR-0156](0156-four-named-effects-join-the-effects-list-and-a-code-drawn-piece-enters-as-pre-rendered-footage.md) §7).

### 2. Geometry of `sweep` and `two_point_conical`

Both new kinds measure their coordinates against the element's **declared box** (`[x, y, W, H]`), with premultiplied sRGB
stop interpolation (`InPremul::Yes`, `ColorSpace::Destination`, `HueMethod::Shorter`), `TileMode::Clamp`, and dithering
off (`set_dither(false)`), matching [ADR-0149](0149-a-gradient-is-a-paint-linear-or-radial-measured-against-the-declared-box.md) §2 and §7.

#### `sweep`

- **`center`:** `[cx, cy]` in declared-box fractions from the top-left corner (any numbers legal; centre may sit outside
  the box).
- **`angle`:** Degrees, following CSS `conic-gradient`, [ADR-0149](0149-a-gradient-is-a-paint-linear-or-radial-measured-against-the-declared-box.md)
  §2 (`linear.angle`), and [ADR-0220](0220-iris-radial-wipe-and-blinds-enter-as-transition-kinds-and-wipe-gains-angle.md)
  §3 (`radial_wipe.start_angle`): `0` starts at **12 o'clock (top, $-y$)** and sweeps stops `0..1` **clockwise** through
  $360^\circ$ around `center` (`90` starts at 3 o'clock $+x$, `180` at 6 o'clock $+y$, `270` at 9 o'clock $-x$; any number
  is legal, so `370` is `10`). A counter-clockwise sweep is written by reversing the stop order.
- **Box-fraction local matrix:** Evaluated in unit box fractions `[0..1, 0..1]` through
  `Matrix::translate((x, y)) * Matrix::scale((W, H)) * Matrix::rotate_deg(angle - 90.0, Point::new(cx, cy))` with
  `shaders::sweep_gradient(Point::new(cx, cy), (0.0, 360.0), &spec, &local)`. Because Skia's `0°` ray points along $+x$
  (3 o'clock) and sweeps clockwise in y-down coordinates, rotating by `angle - 90°` around `(cx, cy)` in box fractions
  places `angle: 0` at 12 o'clock ($-y$) and ensures that `45°`, `135°`, `225°`, and `315°` from `[0.5, 0.5]` pass through
  the declared box's four corners regardless of aspect ratio.

#### `two_point_conical`

- **`center` and `radius`:** Same meaning and domain as `radial` ([ADR-0149](0149-a-gradient-is-a-paint-linear-or-radial-measured-against-the-declared-box.md)
  §2): `center` is `[cx, cy]` in box fractions; `radius` is $\ge 0$ as a fraction of the distance in box fractions from
  `center` to the box's farthest corner:
  $$d_{\max} = \max_{(u, v) \in \{0, 1\}^2} \hypot(u - c_x, v - c_y), \qquad R = \text{radius} \times d_{\max}$$
- **`focal`:** `[fx, fy]` in declared-box fractions from the top-left corner (any numbers legal), specifying where stop
  `0` originates (`start_radius = 0`), while stop `1` lands on the circumference of the outer circle `(center, R)`.
- **Focal-point clamping inside the outer circle:**
  In Skia's `two_point_conical_gradient`, placing the start point on or outside the outer circle `(center, R)` changes the
  shader from an enclosed focal-radial gradient into an open two-circle cone wedge that leaves pixels outside the cone
  unpainted (transparent). To match After Effects (*Highlight Length* / *Highlight Angle*) and SVG 2 (`<radialGradient fx fy>`),
  the painter keeps the effective focal origin strictly inside the outer circle:
  - If $R \le 0$ (`radius <= 0` at the resolved instant), `shader()` returns `None` and the painter fills the box with the
    last stop's colour, identical to `radial`.
  - Let $\Delta = (f_x - c_x, f_y - c_y)$ and $d = \hypot(\Delta_x, \Delta_y)$.
  - If $d = 0$, the focal origin coincides with `center`, and the painter delegates to `shaders::radial_gradient((Point::new(cx, cy), R), &spec, &local)`.
  - If $d \ge 0.999 \times R$, the painter clamps the effective focal point along $\Delta$ to radius $0.999 \times R$:
    $$(f'_x, f'_y) = \left(c_x + \Delta_x \cdot \frac{0.999 \times R}{d},\; c_y + \Delta_y \cdot \frac{0.999 \times R}{d}\right)$$
    and calls `shaders::two_point_conical_gradient((Point::new(f'_x, f'_y), 0.0), (Point::new(cx, cy), R), &spec, &local)`
    with `local = Matrix::translate((x, y)) * Matrix::scale((W, H))`.
  - This clamping is purely a rendering rule (like `radius <= 0` painting the last stop): `query --at` prints the resolved
    `focal` coordinates before clamping so the output pastes back without altering curves, and `shift` never refuses a split
    on `focal`.

### 3. Animation, `validate`, and tools

1. **In-place animation and schema-derived property table ([ADR-0149](0149-a-gradient-is-a-paint-linear-or-radial-measured-against-the-declared-box.md) §3, §6):**
   - `sweep` reuses `center` (`Animatable<Center>`), `angle` (`Animatable<Angle>`), and `stops` (`Animatable<Stops>`).
   - `two_point_conical` reuses `center`, `radius` (`Animatable<Radius>`), and `stops`, and adds `focal` (`Animatable<Center>`).
   - `fill.focal`, `stroke.focal`, and `color.focal` automatically join `animatable::of` via `gradient_parameters(defs)`
     with no per-tool property list.
2. **Overshoot and `shift` ([ADR-0149](0149-a-gradient-is-a-paint-linear-or-radial-measured-against-the-declared-box.md) §4, §6):**
   - Stop offsets clamp and settle identically across all four gradient kinds; `shift` checks stop offsets before §4's fix.
   - `center`, `angle`, and `focal` are unbounded numbers/pairs.
   - A resolved `radius <= 0` on `two_point_conical` paints the last stop's colour over the box, just as on `radial`.
3. **`validate` (`E-GRADIENT-STOP-ORDER`, `E-GRADIENT-STOP-COUNT`, `R-GRADIENT-ONE-COLOUR`, and run-override review):**
   - `E-GRADIENT-STOP-ORDER`, `E-GRADIENT-STOP-COUNT`, and the text run-override review apply unchanged to `sweep` and
     `two_point_conical`.
   - `R-GRADIENT-ONE-COLOUR` (`ResolvedGradient::one_colour`) fires when every drawn frame instant resolves to a single
     colour: all resolved stops share one colour (for any of the four kinds), or resolved `radius <= 0` (for `radial` and
     `two_point_conical`).
4. **Skia byte-identity gate ([ADR-0149](0149-a-gradient-is-a-paint-linear-or-radial-measured-against-the-declared-box.md) §8):**
   - Implementation is gated by extending `crates/montagent-core/tests/filter_bound.rs` and multi-painter (`1..=10`)
     byte-identity tests to cover `sweep` and `two_point_conical` (static and keyed, including clamped focal overshoot) on
     shapes and text under `blur`, `shadow`, anisotropic scale, rotation, and sub-pixel motion.

---

## Considered and refused

- **Admitting *4-Color Gradient* as a fifth `gradient` kind or an `effects[]` entry.** Refused: it takes four 2D points
  and colours rather than a 1D stop list, has no `SkGradientShader` in Skia, and is cleanly covered by stacked radial
  gradients in `project.json` or `montagent-prerender`.
- **Adding an optional `focal` field onto `"gradient": "radial"` instead of `"gradient": "two_point_conical"`.** Refused:
  breaks [ADR-0149](0149-a-gradient-is-a-paint-linear-or-radial-measured-against-the-declared-box.md) §1's invariant that
  every field of a gradient variant is required, and complicates `animatable::gradient_parameters` schema introspection.
- **Passing unclamped `focal` outside the outer circle to `two_point_conical_gradient`.** Refused: Skia renders an open
  two-circle cone that leaves the rest of the element transparent, which contradicts After Effects' *Highlight Length* and
  SVG 2's `<radialGradient>` and causes bezier overshoot on `focal` or `center` to tear holes in the fill.
- **Polar `focal_angle` and `focal_length` fields instead of `focal: [fx, fy]`.** Refused: `[fx, fy]` matches `center: [cx, cy]`
  in box fractions, uses the existing `Center` type, and lets an agent place or animate the highlight at a known box
  coordinate without trigonometry.
- **Counter-clockwise or isotropic-pixel `sweep` angles.** Refused: clockwise from 12 o'clock ($-y$) matches CSS
  `conic-gradient`, `linear.angle`, and `radial_wipe.start_angle`, and evaluating in box fractions via `Matrix::scale((W, H))`
  keeps diagonal rays aligned with the box's corners when `width` or `height` changes.

---

## Consequences

- `crates/montagent-core/src/model/paint.rs` and `schema/montagent.schema.json`:
  - `Gradient` and `ResolvedGradient` gain `Sweep { center, angle, stops }` and `TwoPointConical { center, radius, focal, stops }`.
  - `ResolvedGradient::one_colour` treats `TwoPointConical { radius, .. }` with `radius <= 0` the same as `Radial`.
- `crates/montagent-render/src/canvas/gradient.rs` and `crates/montagent-core/src/verbs/frame/ink.rs`:
  - `GradientKind` gains `Sweep { center: [f64; 2], angle: f64 }` and `TwoPointConical { center: [f64; 2], radius: f64, focal: [f64; 2] }`.
  - `Gradient::shader` builds `shaders::sweep_gradient` and `shaders::two_point_conical_gradient` (with `0.999 * R` focal clamping and `d == 0` radial fallback).
- `skills/montagent-motion/SKILL.md`:
  - Documents the multi-point radial stack recipe for *4-Color Gradient* washes.
- `CONTEXT.md`, [ADR-0149](0149-a-gradient-is-a-paint-linear-or-radial-measured-against-the-declared-box.md), and
  [ADR-0215](0215-after-effects-is-precedent-for-rendered-results-and-capcut-and-premiere-stay-first-for-shape.md) are updated.
