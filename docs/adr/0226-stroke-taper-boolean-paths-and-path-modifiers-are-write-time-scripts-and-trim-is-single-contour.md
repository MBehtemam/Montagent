---
status: accepted
amends: 0154 (confirms single-contour PathElement invariant; refuses procedural variable-width stroke envelopes, in-format subpaths, and in-format boolean CSG in favour of write-time literal vertex compilation), 0158 (confirms uniform stroke_width per element; refuses runtime stroke taper and wave modulation), 0160 (confirms trim is element-local on single outlines; refuses compound cross-element "Individually" trim), 0163 (confirms mask list remains strictly intersecting with invert subtraction; refuses multi-mode raster mask algebra), 0215 (disposes of remaining vector path, shape operation, and mask mode census rows from the After Effects censuses on map #896), 0217 (expands the write-time skill-script contract to vector path operations in paths.py)
---

# Stroke taper, boolean paths, and path modifiers are write-time skill scripts, and trim remains single-contour

[How do stroke taper/wave, boolean path/mask operations, path modifiers and individual trim enter?](https://github.com/MBehtemam/Montagent/issues/1068)
on the After Effects map ([#896](https://github.com/MBehtemam/Montagent/issues/896)).

[ADR-0154](0154-a-point-list-enters-as-one-path-element-in-integer-pixels-from-the-declared-box.md) established the single `path` element
with integer-pixel `points` and handles in a declared bounding box.
[ADR-0158](0158-a-path-chooses-its-stroke-join-and-cap-and-every-shape-takes-a-dash-pattern.md) established uniform `stroke_width`, joins, caps,
and dash patterns.
[ADR-0160](0160-a-stroke-draws-a-window-of-its-outline-measured-in-fractions-of-its-length.md) established `trim_start`, `trim_end`, and `trim_offset`
on an element's outline.
[ADR-0162](0162-a-morph-between-unlike-shapes-is-written-as-matching-vertex-lists-and-no-rule-resamples-them.md) and
[ADR-0163](0163-a-mask-takes-a-closed-path-inline-measured-from-its-own-rect.md) established path-based masks and hole-punching via intersecting inverted masks.
[ADR-0217](0217-motion-bakes-and-keyframe-assistants-enter-in-one-keyframes-script-auto-colour-is-refused-and-a-skill-script-with-its-spec-is-a-template.md)
established the skill-script contract (`skills/montagent-motion/scripts/keyframes.py`) for literal-emitting write-time operations under Python stdlib.
[ADR-0224](0224-units-gain-center-out-edges-in-and-blur-and-range-selectors-glyph-to-path-and-vertical-writing-mode-are-refused.md) §4 confirmed
that `PathElement` has a single `points` contour with no interior subpaths or compound contour array.

Census: AE shape layers and masks ([#901](https://github.com/MBehtemam/Montagent/issues/901) rows 21, 27–38, 46, 51, 52, 57, 71) surfaced four open
areas of vector path and mask gaps.

This ADR decides:
1. **Stroke width profile (Taper and Wave)**: Refused from `path` in the format schema and runtime renderer. Compiled at write time into closed
   filled `path` elements with literal integer `points` via a skill script (`skills/montagent-motion/scripts/paths.py`).
2. **Boolean path operations (Merge Paths) and mask modes**: Merge Paths on shapes is refused in the format and evaluated at write time via a
   skill script, emitting literal `points`. Mask composition in `effects[]` remains strictly intersecting (`DstIn`) plus `invert` (hole punching /
   subtraction); multi-mode raster mask algebra is refused.
3. **Procedural path operation modifiers (Offset Paths, Pucker & Bloat, Round Corners, Twist, Wiggle Paths, Zig Zag)**: Refused from the format
   schema and runtime renderer. Deterministic geometric transformers run at write time in `skills/montagent-motion/scripts/paths.py`, emitting
   literal integer vertices and handles into `project.json`.
4. **Trim paths across contours and sibling paths ("Individually" vs "Simultaneously")**: Compound cross-element / cross-contour "Individually"
   trim is refused from the format. Simultaneous trimming across sibling `path` elements is authored via identical keyframes. Sequential
   trimming across multiple paths is authored via staggered keyframes in time, with arc-lengths measured via `query --at` or baked by a skill script.

---

## Decisions

### 1. Stroke width profiles (Taper and Wave) are write-time compiled closed paths

After Effects provides stroke taper (start/end length, start/end width, and start/end ease) and wave (width modulation by wavelength, amount, and phase).

1. **Refused in the runtime format schema:**
   - Skia's native path stroker (`SkPaint::setStrokeWidth`) supports only constant scalar stroke widths. Evaluating variable-width envelopes at
     runtime requires tessellating dynamic triangle meshes or generating dynamic offset contours on every frame.
   - Procedural runtime envelopes violate [ADR-0145](0145-premiere-and-capcut-features-enter-when-they-preserve-a-flat-literal-json-schema-editable-by-exact-string-replace.md) §1:
     the rendered ink boundary is not in the file, and bounding box validation (`E-PATH-OUTSIDE-BOX`, [ADR-0158](0158-a-path-chooses-its-stroke-join-and-cap-and-every-shape-takes-a-dash-pattern.md) §4)
     cannot be computed from literal vertices alone without executing the non-linear envelope algorithm.
   - `path.stroke_width` remains a uniform scalar (static or animatable).

2. **Compiled to closed filled `path` elements at write time:**
   - A variable-width stroke is fundamentally an area boundary. A write-time tool evaluates the normal vectors $\vec{n}(s)$ along the spine curve,
     offsets the left and right boundaries by $w(s) / 2$, closes the contour with the selected start/end caps, and emits a literal closed
     `path` with `points` and `fill`.
   - The emitted element is a standard `type: "path", "closed": true`, fully verifiable by `validate`, sight-readable in `query --at`, and
     byte-identical across parallel renderers.
   - Animated taper/wave profiles are baked into matching-count `points` keyframes per [ADR-0162](0162-a-morph-between-unlike-shapes-is-written-as-matching-vertex-lists-and-no-rule-resamples-them.md).

---

### 2. Boolean path operations are write-time bakes, and masks remain intersecting

After Effects provides *Merge Paths* (Add, Subtract, Intersect, Exclude) on shape layer groups, and mask modes (*Add*, *Subtract*, *Intersect*, *Difference*)
on layer masks.

1. **Shape-level boolean operations (Merge Paths) are refused from the runtime format:**
   - [ADR-0154](0154-a-point-list-enters-as-one-path-element-in-integer-pixels-from-the-declared-box.md) §1 and
     [ADR-0224](0224-units-gain-center-out-edges-in-and-blur-and-range-selectors-glyph-to-path-and-vertical-writing-mode-are-refused.md) §4
     pin `PathElement` to a single literal `points` contour, refusing subpath arrays and in-project CSG trees.
   - Runtime boolean evaluation (`SkOp`) across animated shapes suffers from topological discontinuities (cracking, vertex-count jumping mid-animation),
     non-manifold degeneracies on collinear segments, and opaque non-literal JSON representations.
   - Merge Paths operations (Union, Difference, Intersection, XOR) are evaluated at write time by `skills/montagent-motion/scripts/paths.py`, which
     resolves the boolean outline and writes literal integer `points` into `project.json`.
   - For shapes with interior cutouts (holes), the outer filled shape is paired with an inner inverted `mask` per
     [ADR-0163](0163-a-mask-takes-a-closed-path-inline-measured-from-its-own-rect.md) and [ADR-0165](0165-a-hard-edged-inverted-mask-is-the-complement-off-the-edge-and-a-feathered-pair-sums-within-one-level.md).

2. **Mask modes remain strictly intersecting + `invert`:**
   - Masks in an element's `effects[]` array composite sequentially as alpha intersections (`DstIn` / multiply).
   - An inverted mask (`"invert": true`) erases its interior from the cumulative mask buffer, providing exact boolean subtraction (punching holes).
   - Multi-mode mask algebra (`mode: "add" | "subtract" | "intersect" | "difference"`) on a single element is refused:
     - Additive mask unions on a single element violate the flat sequential filtering pipeline of `effects[]`.
     - Where two distinct additive masks are required, author them on duplicate elements or nest them inside a `type: "nest"` group ([ADR-0201](0201-nest-is-a-recursive-track-holding-element-whose-transforms-compose-until-an-effect-or-blend-mode-flattens-it.md)).

---

### 3. Procedural path modifiers are deterministic write-time transformations

After Effects provides procedural operators inserted into shape groups: *Offset Paths*, *Pucker & Bloat*, *Round Corners*, *Twist*, *Wiggle Paths*, and *Zig Zag*.

1. **Refusal from format schema and runtime renderer:**
   - **`Wiggle Paths`**: relies on procedural pseudo-random noise generators, which are explicitly refused by
     [ADR-0148](0148-hold-and-cubic-bezier-enter-the-easing-model-steps-repeat-and-spring-do-not.md),
     [ADR-0151](0151-letter-spacing-is-an-animatable-element-field-and-a-stagger-is-a-units-block-on-one-text-element.md) §8, and
     [ADR-0217](0217-motion-bakes-and-keyframe-assistants-enter-in-one-keyframes-script-auto-colour-is-refused-and-a-skill-script-with-its-spec-is-a-template.md).
     Seeded procedural generation in the renderer breaks exact-string editing and deterministic state verification.
   - **`Offset Paths`**, **`Pucker & Bloat`**, **`Twist`**, and **`Zig Zag`**: procedurally rewrite vertex positions on every frame if evaluated in the engine,
     destroying literal coordinates and complicating bounding containment checks.
   - **`Round Corners`**: already natively reachable without extra operators by placing cubic Bezier handles (`in`/`out`) at vertices ([ADR-0154](0154-a-point-list-enters-as-one-path-element-in-integer-pixels-from-the-declared-box.md)).

2. **Write-time geometric execution in `skills/montagent-motion/scripts/paths.py`:**
   - Following the precedent of `keyframes.py` ([ADR-0217](0217-motion-bakes-and-keyframe-assistants-enter-in-one-keyframes-script-auto-colour-is-refused-and-a-skill-script-with-its-spec-is-a-template.md)),
     path operations are executed ahead of time via a Python 3 stdlib script:
     ```bash
     python3 skills/montagent-motion/scripts/paths.py <project.json> <spec.json>
     ```
   - Supported operations in `<spec.json>`:
     - `taper`: converts a spine `path` with a taper/wave profile into a closed filled outline `path`.
     - `offset`: expands or shrinks path outlines with selectable join styles (`round`, `miter`, `bevel`).
     - `pucker_bloat`: scales segment control handles inward or outward relative to vertex centers.
     - `twist`: rotates vertices around an element center with angle decaying quadratically with distance.
     - `zig_zag`: subdivides linear or curved segments into alternating ridges and valleys.
     - `wiggle`: generates seeded, keyframed procedural vertex noise using deterministic pseudo-random functions.
     - `boolean`: performs 2D polygon/bezier union, difference, and intersection.
   - The script reads `<spec.json>`, computes integer coordinates and handles, and updates the target element's `points` (or keyframed `points`)
     directly in `project.json` using exact-string replacement.

---

### 4. Trim Paths is element-local; sequential trim is authored across timeline keys

In After Effects, Trim Paths applied to multiple paths can be set to *Simultaneously* or *Individually*.

1. **Simultaneous trim:**
   - Trimming multiple sibling `path` elements concurrently is already fully supported with zero schema additions:
     author identical `trim_start`, `trim_end`, and `trim_offset` keyframes across each sibling element ([ADR-0160](0160-a-stroke-draws-a-window-of-its-outline-measured-in-fractions-of-its-length.md)).

2. **Sequential trim ("Individually") across multiple elements:**
   - In After Effects, "Individually" stitches multiple subpaths into a single virtual continuum, drawing the first path completely before
     starting the second.
   - A runtime "Individually" compound trim mode in Montagent is **refused**:
     - It couples independent elements together, violating element self-containment ([ADR-0145](0145-premiere-and-capcut-features-enter-when-they-preserve-a-flat-literal-json-schema-editable-by-exact-string-replace.md) §1, [ADR-0150](0150-wipe-slide-and-push-enter-as-transition-kinds-and-a-matte-from-another-element-is-refused.md)).
     - It introduces hidden cross-element timing calculations into the renderer.
   - **How sequential trimming is authored:**
     - Sequential trimming is expressed explicitly on the timeline by partitioning the total duration across the elements.
     - For $N$ paths with lengths $L_1, L_2, \dots, L_N$ (queried via `montagent query --at <element_id>`), each path $k$ has duration
       $\Delta t_k = T \cdot \frac{L_k}{\sum L_i}$ and starts when path $k-1$ finishes.
     - Each element receives standard, self-contained `trim_end: [{"t": t_{start}, "v": 0}, {"t": t_{end}, "v": 1}]` keyframes.
     - A helper pass in `skills/montagent-motion/scripts/paths.py` (`"op": "trim_sequential"`) automates this calculation, reading element lengths
       and writing exact timeline keyframes into each element.

---

## Invariants (ADR-0145)

| Invariant | How this decision preserves it |
| :--- | :--- |
| **Literal values** | Every vertex, handle, and trim window remains an explicit literal integer or fraction in `project.json`. No procedural geometry engines run in the renderer. |
| **Closed vocabulary** | No new element types, modifier nodes, or compound path structures enter the schema. `path`, `mask`, and `trim` retain their exact existing properties. |
| **Checkable by `validate`** | Bounding box containment (`E-PATH-OUTSIDE-BOX`), handle validity (`E-PATH-DANGLING-HANDLE`), and trim window ranges (`E-TRIM-EMPTY`) continue to be verified statically from JSON literals without running runtime geometric algorithms. |
| **Exact-string replace** | All path manipulations occur ahead of time via write-time scripts that modify explicit `points` arrays via standard exact-string replace. |

---

## Consequences

- **Schema and binary:** No changes required to `montagent-core` schema or `montagent-render` pipeline.
- **`CONTEXT.md`:** Updated to record the write-time boundaries for variable-width strokes, boolean path operations, procedural path modifiers, and sequential trim.
- **`skills/montagent-motion`:**
  - Gains `scripts/paths.py` supporting `taper`, `offset`, `pucker_bloat`, `twist`, `zig_zag`, `wiggle`, `boolean`, and `trim_sequential`.
  - Gains recipes in `SKILL.md` demonstrating variable-width stroke compilation, boolean shape cutouts, and sequential path draw-ons.
- **Census #901:** Rows 21, 27–38, 46, 51, 52, 57, and 71 are fully resolved and closed under this ADR.
