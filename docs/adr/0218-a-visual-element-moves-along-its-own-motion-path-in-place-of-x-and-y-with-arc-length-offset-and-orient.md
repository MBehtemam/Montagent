---
status: accepted
date: 2026-04-14
amends: 0012 (adds optional motion_path in place of x and y on visual elements, with E-MOTION-PATH-XY when combined with x or y, and refuses spatial Bezier tangents on x and y keyframes), 0146 (adds motion_path.offset as an animatable scalar fraction along the path arc length with E-MOTION-PATH-OFFSET-RANGE on open paths), 0154 (reuses the closed and points vertex list and arc-length curve sampler for motion_path in frame coordinates without E-PATH-OUTSIDE-BOX, and adds E-MOTION-PATH-ZERO-LENGTH), 0215 (adopts After Effects motion paths, roving constant-speed progression, and Orient Along Path as one self-contained motion_path object on visual elements)
---

# A visual element moves along its own `motion_path` in place of `x` and `y`, with arc-length `offset` and `orient`

[#1060](https://github.com/MBehtemam/Montagent/issues/1060), on the map
[#896](https://github.com/MBehtemam/Montagent/issues/896).

## Context and Problem Statement

[ADR-0215](0215-after-effects-is-precedent-for-rendered-results-and-capcut-and-premiere-stay-first-for-shape.md) named After Effects our precedent for rendered motion-graphics results while keeping CapCut and Premiere first for format shape. Under [ADR-0012](0012-flat-transform-keyframes-carried-by-their-element.md) and [ADR-0146](0146-an-animatable-property-is-one-the-schema-types-so-colour-blends-premultiplied-and-spring-easing-is-refused.md), every visual element positions its `origin` with two independent scalar properties, `x` and `y`, each either a literal integer or a keyframe list interpolated independently in time. Meanwhile, [ADR-0154](0154-a-point-list-enters-as-one-path-element-in-integer-pixels-from-the-declared-box.md) introduced cubic Bezier vertex lists (`closed` + `points` with `at`, `in`, and `out` in integer pixels) and arc-length curve sampling, and [ADR-0161](0161-a-text-element-bends-its-one-line-along-its-own-inline-path.md) reused that curve vocabulary so a `text` element can bend its one line along its own inline `path` inside its declared box.

Moving a visual element as a whole along a curved trajectory exposes three gaps when an author only has independent `x` and `y` keyframe lists:

1. **Curved trajectory between stops.** Because `x` and `y` evaluate 1D temporal eases independently, moving an element along a true cubic Bezier arc, S-curve, or multi-stop spline requires either matching mismatched `cubic-bezier(...)` curves on `x` and `y` (which still cannot turn past an axis-aligned extremum within one segment) or baking dozens of dense `x` and `y` keyframes.
2. **Constant speed across uneven segments (roving).** When an element visits three or more waypoints of unequal spacing — or traces a curve whose Bezier handles compress one end and stretch the other — an author who wants constant speed along the trajectory must manually compute the Euclidean or cubic-Bezier arc length of every span and proportion each keyframe's `t` to cumulative distance.
3. **Tangent orientation (`Orient Along Path`).** Steering an arrow, cursor, vehicle, or callout badge so its heading stays tangent to its trajectory requires differentiating $(x(t), y(t))$ by hand and baking a matching `rotation` keyframe list.

We need to decide:

1. Whether curved element trajectories enter the format as spatial tangents on `x` and `y` keyframes, as a skill-script bake into `x`/`y`/`rotation`, or as an optional `motion_path` field on visual elements in place of `x` and `y`.
2. What shape `motion_path` takes, how its vertex coordinates are expressed and validated, and how constant-speed progression (roving) works across multi-segment curves and polylines.
3. How tangent orientation (`orient`) composes with the element's own `rotation`, how `query --at`, `validate`, and `shift` handle `motion_path`, and how degenerate paths and transform pipeline order behave.

## Decision Drivers

- **Keep `x` and `y` flat 1D scalar lists ([ADR-0012](0012-flat-transform-keyframes-carried-by-their-element.md), [ADR-0146](0146-an-animatable-property-is-one-the-schema-types-so-colour-blends-premultiplied-and-spring-easing-is-refused.md)).** Spatial `in`/`out` tangents on 1D `x` and `y` keyframes would break independent 1D evaluation by coupling `x` and `y` at identical timestamps, and still could not represent constant-speed arc-length progression without a second coupling flag.
- **Reuse the existing cubic Bezier geometry and arc-length sampler ([ADR-0154](0154-a-point-list-enters-as-one-path-element-in-integer-pixels-from-the-declared-box.md), [ADR-0161](0161-a-text-element-bends-its-one-line-along-its-own-inline-path.md)).** Montagent already parses `closed` and `points` (`at`, `in`, `out`), flattens cubic Bezier segments at `CURVE_RESOLUTION = 64.0` (`Curve::of`), and samples position and unit tangent `(tx, ty)` at any arc distance $d \in [0, L]$ (`Curve::sample_at`).
- **Solve constant-speed roving by construction.** Parameterizing progress along the trajectory by a single animatable arc-length fraction `offset` means a two-keyframe `linear` animation from `0` to `1` moves at constant pixel speed across any number of uneven straight or curved segments, with no timestamp-proportioning script needed.
- **One unambiguous source of truth for placement ([ADR-0157](0157-a-speed-ramp-is-a-time-remap-curve-of-source-times-on-a-video-element.md) precedent).** Just as `source_time` replaces `source_start`, `source_end`, `speed`, and `overrun` (`E-REMAP-FIELD`), `motion_path` replaces `x` and `y` (`E-MOTION-PATH-XY`) so an element never carries two competing position declarations.
- **Honest inspection and spatial validation ([ADR-0006](0006-validate-reports-facts-and-render-enforces.md), [ADR-0011](0011-tool-surface-reads-checks-renders.md)).** `query --at` and `validate` must evaluate the exact derived `(x, y)` and rotation that `render` draws so `R-OFF-CANVAS`, `NOT COVERED`, layer-tie overlap ([ADR-0060](0060-layer-tie-is-an-error-array-order-stays-meaningless.md)), and `R-TEXT-CONTRAST` remain exact.

## Considered Options

### Question 1 — Where curved trajectories live in the format

- **Option 1A: Optional `motion_path` field on every visual element in place of `x` and `y` (chosen).** Every visual element (`image`, `video`, `rect`, `ellipse`, `path`, `text`) accepts an optional `motion_path` object in place of `x` and `y`. Setting `x` or `y` alongside `motion_path` is refused with `E-MOTION-PATH-XY`. `x` and `y` stay strictly 1D scalar properties.
- **Option 1B: Spatial Bezier tangents and `roving: true` on `x` and `y` keyframes.** Couples two separate scalar properties at matching timestamps and breaks independent 1D keyframe evaluation.
- **Option 1C: Skill-script bake only (`keyframes.py`).** Floods the project with dense `x`, `y`, and `rotation` keyframes for a simple 2- or 3-vertex curve, obscuring author intent and colliding with [ADR-0217](0217-motion-bakes-and-keyframe-assistants-enter-in-one-keyframes-script-auto-colour-is-refused-and-a-skill-script-with-its-spec-is-a-template.md)'s boundary (skill scripts are for procedural motion and keyframe assistants, whereas cubic Bezier curves and arc-length sampling are already native to the engine).

### Question 2 — Shape of `motion_path` and constant-speed roving

- **Option 2A: Self-contained `motion_path` object (`closed`, `points`, `offset`, `orient`) with static integer vertices in frame space (chosen).** Reuses [ADR-0154](0154-a-point-list-enters-as-one-path-element-in-integer-pixels-from-the-declared-box.md)'s `closed` and `points` (`at`, `in`, `out` as `[i32; 2]`) in frame coordinates (the same coordinate space where `x` and `y` live), without `E-PATH-OUTSIDE-BOX` so trajectories can enter or exit off-canvas, plus an animatable arc-length fraction `offset` and static `orient: bool`.
- **Option 2B: Cross-element reference (`motion_path: {"element": "<id>", ...}`).** Introduces live element-to-element references, which [ADR-0012](0012-flat-transform-keyframes-carried-by-their-element.md), [ADR-0086](0086-recorded-intent-is-one-pattern-and-the-time-axis-instantiates-it.md), [ADR-0150](0150-wipe-slide-and-push-enter-as-transition-kinds-and-a-matte-from-another-element-is-refused.md), and [ADR-0161](0161-a-text-element-bends-its-one-line-along-its-own-inline-path.md) §2 refuse.

### Question 3 — `orient`, composition with `rotation`, and CLI inspection

- **Option 3A: `orient: bool` adds the forward curve tangent angle to `rotation`; `query --at` exposes derived `x`, `y`, `length`, `tangent_angle`, and `effective_rotation` (chosen).** When `orient: true` (default `false`), the curve's forward tangent angle $\theta_{\text{path}} = \operatorname{atan2}(t_y, t_x) \times \frac{180}{\pi}$ in degrees is added to the element's resolved `rotation` (`effective_rotation = rotation + tangent_angle`), so static `rotation` acts as a constant heading offset and keyframed `rotation` banks or spins relative to the heading.

## Decision Outcome

Chosen options: **1A**, **2A**, and **3A**.

### 1. `motion_path` on visual elements and mutual exclusion with `x` and `y` (`E-MOTION-PATH-XY`)

Every visual element (`image`, `video`, `rect`, `ellipse`, `path`, `text`) accepts an optional `motion_path` field:

```json
{
  "id": "paper-plane",
  "type": "image",
  "source": "assets/plane.png",
  "start": 0,
  "end": 3000,
  "width": 120,
  "height": 80,
  "origin": "center",
  "rotation": 0,
  "motion_path": {
    "closed": false,
    "points": [
      { "at": [-60, 800], "out": [300, -500] },
      { "at": [960, 440], "in": [-240, 120], "out": [240, -120] },
      { "at": [1980, 200], "in": [-300, 400] }
    ],
    "offset": [
      { "t": 0, "v": 0.0 },
      { "t": 3000, "v": 1.0, "ease": "cubic-bezier(0.4, 0, 0.2, 1)" }
    ],
    "orient": true
  }
}
```

- **`E-MOTION-PATH-XY`:** `motion_path` replaces `x` and `y` as the element's position declaration. When `motion_path` is omitted, `x` and `y` default to `0` as before ([ADR-0012](0012-flat-transform-keyframes-carried-by-their-element.md)). When `motion_path` is present, writing `x` or `y` (whether as a literal `i32` or a keyframe list, even `0`) is refused by `montagent validate` with `E-MOTION-PATH-XY`.
- **Distinct from `text.path` ([ADR-0161](0161-a-text-element-bends-its-one-line-along-its-own-inline-path.md)):** On a `text` element, `path` bends the text's one line inside the text element's declared `width × height` box, whereas `motion_path` translates the element's `origin` across the frame. A `text` element may carry either or both.
- **No spatial tangents on `x` and `y`:** `x` and `y` remain strictly 1D scalar properties (`i32` or a keyframe list of `i32`).

### 2. Geometry, coordinate space, and validation (`E-MOTION-PATH-ZERO-LENGTH`, `E-MOTION-PATH-OFFSET-RANGE`)

`motion_path` is a closed JSON object (`additionalProperties: false`):

| Field | Type | Default | Animatable? | Meaning |
| --- | --- | --- | --- | --- |
| `closed` | `bool` | required | No (static) | Whether the last vertex connects back to the first vertex as a closing segment. |
| `points` | `Vec<Vertex>` | required | No (static) | Cubic Bezier vertices `{"at": [x, y], "in"?: [dx, dy], "out"?: [dx, dy]}` ([ADR-0154](0154-a-point-list-enters-as-one-path-element-in-integer-pixels-from-the-declared-box.md)). |
| `offset` | `f64 \| Keyframe<f64>` | `0` | Yes | Arc-length fraction along `points` (`[0, 1]` when `closed: false`, `[-1, 2]` when `closed: true`). |
| `orient` | `bool` | `false` | No (static) | When `true`, adds the curve's forward tangent angle to the element's `rotation`. |

- **Coordinate space:** Each vertex's `at: [x, y]` is a pair of signed integer (`i32`) pixel coordinates in the frame (the same coordinate space as `x` and `y`). `in: [dx, dy]` and `out: [dx, dy]` are optional signed integer pixel handle offsets relative to their own vertex's `at`.
- **Vertex validation:** `motion_path.points` is checked by [ADR-0154](0154-a-point-list-enters-as-one-path-element-in-integer-pixels-from-the-declared-box.md)'s structural vertex rules:
  - `E-PATH-TOO-FEW-POINTS` when an open path has fewer than 2 vertices or a closed path has fewer than 3 vertices.
  - `E-PATH-DANGLING-HANDLE` when `closed: false` and the first vertex carries `in` or the last vertex carries `out`.
  - **No `E-PATH-OUTSIDE-BOX`:** Unlike `path.points` and `text.path.points`, `motion_path.points` and its Bezier hull are not constrained to an element box or the frame canvas, so elements can enter from or exit to off-canvas coordinates. (If an element's bounding rect never intersects the frame at any instant of its active span, `R-OFF-CANVAS` ([ADR-0044](0044-off-canvas-is-a-standing-review-check-not-a-frame-change-census.md)) still fires as usual.)
- **Non-zero arc length (`E-MOTION-PATH-ZERO-LENGTH`):** `montagent validate` refuses a `motion_path` with `E-MOTION-PATH-ZERO-LENGTH` when every segment of `motion_path.points` has zero length — that is, for every segment between consecutive vertices $v_0$ and $v_1$, all four cubic Bezier points coincide ($v_0.\text{at} = v_0.\text{at} + v_0.\text{out} = v_1.\text{at} + v_1.\text{in} = v_1.\text{at}$, giving total arc length $L = 0$). A stationary element should declare static `x` and `y` instead. Because vertex and handle coordinates are integers (`i32`), a segment has non-zero length under `Curve::of` at `CURVE_RESOLUTION = 64.0` if and only if its four Bezier points are not all identical.
- **Arc-length `offset` and constant-speed roving (`E-MOTION-PATH-OFFSET-RANGE`):**
  - `motion_path.offset` joins the one schema-derived list of animatable properties ([ADR-0146](0146-an-animatable-property-is-one-the-schema-types-so-colour-blends-premultiplied-and-spring-easing-is-refused.md)). The JSON Schema bounds literal and keyframe `v` values to `[-1, 2]`.
  - When `closed: false`, `montagent validate` checks that every literal or keyframe `v` of `motion_path.offset` lies in `[0, 1]`, emitting `E-MOTION-PATH-OFFSET-RANGE` otherwise.
  - At evaluation instant $t$, `offset` is resolved and clamped to `[0, 1]` when `closed: false` or `[-1, 2]` when `closed: true` (absorbing any `cubic-bezier` overshoot, matching [ADR-0160](0160-a-stroke-draws-a-window-of-its-outline-measured-in-fractions-of-its-length.md) and [ADR-0164](0164-path-offset-reaches-one-curve-length-past-each-end-and-a-closed-path-draws-a-body-only-if-it-fits-the-loop.md)).
  - The arc distance along the path of total length $L$ is:
    - Open path (`closed: false`): $d = \operatorname{clamp}(\text{offset}, 0, 1) \times L$.
    - Closed path (`closed: true`): $d = \operatorname{clamp}(\text{offset}, -1, 2).\text{rem\_euclid}(1.0) \times L$, with $\text{offset} = 1.0$ wrapping cleanly to the start/end vertex of the closed loop.
  - Because `d` is measured in cumulative arc length along the flattened polyline built by `Curve::of(points, closed)` (`CURVE_RESOLUTION = 64.0`, [ADR-0154](0154-a-point-list-enters-as-one-path-element-in-integer-pixels-from-the-declared-box.md)), a two-keyframe `linear` animation on `offset` from `0` to `1` moves the element at constant pixel speed across any number of uneven straight or curved segments — delivering After Effects' roving keyframe behavior without a separate roving flag or script pass.

### 3. `orient`, transform pipeline order, and CLI behavior (`query --at`, `validate`, `shift`)

- **Sampling position and tangent:** `Curve::sample_at(d)` returns the floating-point position $(p_x, p_y)$ and unit forward tangent $(t_x, t_y)$ in the direction of increasing `points` index at arc distance $d$.
  - On a curve with non-zero total length $L > 0$, if an individual segment is zero-length (a duplicate vertex) or has a zero-length handle at an endpoint, `Curve::of` only records non-zero chord steps (`seg_len > 1e-9`) into its cumulative arc-length table, so `Curve::sample_at(d)` naturally inherits the unit tangent of the adjacent non-zero step without `NaN` or division by zero.
- **Composing `orient` with `rotation`:**
  - When `motion_path.orient` is `false` (the default), the element's effective rotation is its resolved `rotation` (in degrees, default `0`).
  - When `motion_path.orient` is `true`, the curve's forward tangent angle is computed as:
    $$\theta_{\text{path}} = \operatorname{atan2}(t_y, t_x) \times \frac{180}{\pi}$$
    where `0°` points along `+x` (right) and `90°` points along `+y` (down), matching the clockwise screen-space convention of `rotation` ([ADR-0012](0012-flat-transform-keyframes-carried-by-their-element.md)). The element's effective rotation is:
    $$\text{effective\_rotation} = \text{rotation} + \theta_{\text{path}}$$
    An artwork drawn pointing right (`+x`) needs `"rotation": 0`; an artwork drawn pointing up (`-y`) sets `"rotation": 90` so its nose aligns with the tangent; keyframing `rotation` banks or spins the element on top of the path heading.
- **Transform pipeline order ([ADR-0012](0012-flat-transform-keyframes-carried-by-their-element.md), [ADR-0025](0025-clip-stays-static.md), [ADR-0084](0084-the-mask-rect-is-one-shape-independent-parameter-set.md), [ADR-0167](0167-an-element-may-be-projected-never-placed-swivel-tilt-and-perspective-join-the-transform.md)):**
  - At render time, `render` uses the floating-point $(p_x, p_y)$ from `Curve::sample_at(d)` as the element's `origin` position on the frame and `effective_rotation` as its 2D in-plane rotation, slotting into the exact same position in the transform pipeline as `x`, `y`, and `rotation`:
    1. The element is drawn flat with its `effects` and `mask` ([ADR-0084](0084-the-mask-rect-is-one-shape-independent-parameter-set.md)).
    2. Projection (`swivel`, `tilt`, `perspective`, [ADR-0167](0167-an-element-may-be-projected-never-placed-swivel-tilt-and-perspective-join-the-transform.md)), `scale`, and `effective_rotation` turn and scale about `origin` ([ADR-0013](0013-fitted-extents-floor-and-the-nine-origin-keywords.md)), and `motion_path` places `origin` at $(p_x, p_y)$ in the frame.
    3. `clip` ([ADR-0025](0025-clip-stays-static.md)) remains a static frame-space aperture applied after the element transform.
- **`montagent query --at <ms>` ([ADR-0011](0011-tool-surface-reads-checks-renders.md), [ADR-0070](0070-the-where-predicate-is-a-conjunction-of-whole-value-terms.md)):**
  - When an element carries `motion_path`, `resolved` omits `x` and `y` and includes `"motion_path": {"closed": <bool>, "points": [...], "offset": <f64>, "orient": <bool>}` with `offset` evaluated at `<ms>` and rounded to 4 decimal places.
  - `derived` includes `"x": p_x.round() as i32` and `"y": p_y.round() as i32`, `"length": <f64>` (the total arc length $L$ of `motion_path.points` in pixels, rounded to 4 decimal places), and, when `orient: true`, `"tangent_angle": <f64>` ($\theta_{\text{path}}$ in degrees, rounded to 4 decimal places) and `"effective_rotation": <f64>` ($\text{rotation} + \theta_{\text{path}}$, rounded to 4 decimal places).
  - The element's bounding `box` (and projected `corners` under [ADR-0167](0167-an-element-may-be-projected-never-placed-swivel-tilt-and-perspective-join-the-transform.md)) in `derived` is computed from the derived position and `effective_rotation`.
- **`montagent validate` ([ADR-0006](0006-validate-reports-facts-and-render-enforces.md), [ADR-0044](0044-off-canvas-is-a-standing-review-check-not-a-frame-change-census.md), [ADR-0060](0060-layer-tie-is-an-error-array-order-stays-meaningless.md)):**
  - All spatial and visibility checks (`R-OFF-CANVAS`, `NOT COVERED`, layer-tie overlap detection under [ADR-0060](0060-layer-tie-is-an-error-array-order-stays-meaningless.md), and `R-TEXT-CONTRAST`) evaluate the element at its derived position and `effective_rotation` across sample instants.
- **`montagent shift` ([ADR-0012](0012-flat-transform-keyframes-carried-by-their-element.md), [ADR-0146](0146-an-animatable-property-is-one-the-schema-types-so-colour-blends-premultiplied-and-spring-easing-is-refused.md)):**
  - `montagent shift` carries and splits `motion_path.offset` keyframes across a split point just like any other `f64` keyframe list, keeping `closed`, `points`, and `orient` intact.

## Consequences

- **Positive:**
  - Any visual element (`image`, `video`, `rect`, `ellipse`, `path`, `text`) can follow a multi-segment cubic Bezier trajectory or polyline with constant-speed roving (`offset: [{"t": start, "v": 0}, {"t": end, "v": 1, "ease": "linear"}]`) and tangent auto-orientation (`orient: true`) using the same vertex (`at`, `in`, `out`) and `Curve::of` machinery already proven in `path` ([ADR-0154](0154-a-point-list-enters-as-one-path-element-in-integer-pixels-from-the-declared-box.md)) and `text.path` ([ADR-0161](0161-a-text-element-bends-its-one-line-along-its-own-inline-path.md)).
  - `x` and `y` remain simple, decoupled 1D scalar properties, and `E-MOTION-PATH-XY` prevents ambiguous dual positioning.
  - `query --at` and `validate` see the exact derived position, arc length, tangent angle, and bounding box that `render` draws.
- **Negative:**
  - `motion_path.points` is static (like `text.path.points`); morphing a trajectory's shape over time while simultaneously sliding an element along it is not supported directly and would require baking to `x`/`y` keyframes.
  - Authors switching an existing element from static `x`/`y` to `motion_path` must remove `x` and `y` from the element JSON to satisfy `E-MOTION-PATH-XY`.
