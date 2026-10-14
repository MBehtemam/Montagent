---
status: accepted
amends: 0150 (admits iris, radial_wipe, and blinds to TransitionKind; adds optional angle to wipe mutually exclusive with direction; generalises Bridge.cut to an aperture enum painted with aliased clipping; confirms feather is refused on transitions and soft reveals stay on element-local masks), 0215 (disposes of the Iris Wipe, Radial Wipe, Venetian Blinds, and Angled Wipe rows from the After Effects transition census on map #896)
---

# Iris, radial wipe, and blinds enter as transition kinds, and wipe gains angle

[Which new transition kinds (iris, clock/radial wipe, blinds, block dissolve, angled/feathered wipe) enter under ADR-0150?](https://github.com/MBehtemam/Montagent/issues/1062)
on the After Effects map ([#896](https://github.com/MBehtemam/Montagent/issues/896)).

[ADR-0150](0150-wipe-slide-and-push-enter-as-transition-kinds-and-a-matte-from-another-element-is-refused.md)
established transitions as top-level elements (`type: "transition"`) bridging two visual elements
across their shared temporal intersection, admitting `crossfade`, `wipe`, `slide`, and `push`
(with `audio_crossfade` added by [ADR-0176](0176-a-transition-carries-the-audio-across-its-cut-in-one-field-and-an-audio-only-crossfade-is-a-transition-kind.md)).
ADR-0150 explicitly left the `kind` enum open to grow as one-token literals when forcing precedent
shaped their parameters.

The After Effects effects census ([#905](https://github.com/MBehtemam/Montagent/issues/905), rows 124–131),
supported by Premiere and CapCut precedent ([ADR-0215](0215-after-effects-is-precedent-for-rendered-results-and-capcut-and-premiere-stay-first-for-shape.md)),
identified five widely used transition patterns absent in Montagent today:
- *Iris Wipe* (circle or box expanding/contracting from a point);
- *Radial Wipe* (clock sweep around a centre);
- *Venetian Blinds* (parallel slats opening across the frame);
- *Block Dissolve* (grid of blocks revealing the incoming element);
- Angled and soft-edged (*feathered*) linear wipes.

This ADR decides:
1. Which transition kinds enter `TransitionKind`, and which are refused.
2. The hard-edge invariant and the refusal of `feather` on transitions.
3. The closed parameter vocabulary and defaults for the new kinds.
4. The addition of `angle` on `wipe` as a mutually exclusive alternative to `direction`.
5. The aperture abstraction in `Bridge` and byte-identical Skia painter clipping.

## Decisions

### 1. `iris`, `radial_wipe`, and `blinds` enter `TransitionKind`; `block_dissolve` and star/polygon irises are refused

- **`iris`**, **`radial_wipe`**, and **`blinds`** join `TransitionKind`:
  ```json
  {"id": "t1", "type": "transition", "start": 3000, "end": 4000,
   "kind": "iris", "from": "clip_a", "to": "clip_b", "shape": "circle", "direction": "out"}
  ```
- **`block_dissolve` is refused from the format schema.**
  A block dissolve requires pseudo-random grid block ordering and hash generation. Unlike deterministic geometric
  clipping, random block transitions require seed negotiation and grid discretization that behave like procedural
  motion rather than a clean geometric cut. Authors desiring block reveals achieve them via a motion skill script
  or pre-rendered matte.
- **Complex star and arbitrary polygon irises are refused.**
  `iris.shape` is restricted strictly to `"circle"` and `"rect"`. Star and multi-point polygonal wipes are legacy
  decorative tropes that require variable point counts and inner/outer radii; custom vector shapes desiring transition-like
  reveals are authored directly as keyed path masks ([ADR-0163](0163-a-mask-takes-a-closed-path-inline-measured-from-its-own-rect.md)).

### 2. Transition boundaries remain strictly hard and aliased; `feather` is refused on `transition`

All geometric transitions (`wipe`, `iris`, `radial_wipe`, `blinds`) strictly maintain ADR-0150 §2's
**hard, aliased boundary invariant** (`aa: false` in Skia):

1. **Backdrop seam bleed.**
   If two overlapping elements are clipped with complementary feathered (soft) edges and composited independently
   over a background or backdrop, the fractional opacities $\alpha$ and $1 - \alpha$ do not sum to unity over the backdrop:
   $$C_{\text{out}} = C_{\text{incoming}} \cdot \alpha + (C_{\text{outgoing}} \cdot (1 - \alpha) + C_{\text{backdrop}} \cdot \alpha) \cdot (1 - \alpha) \neq C_{\text{incoming}} \cdot \alpha + C_{\text{outgoing}} \cdot (1 - \alpha)$$
   The underlying backdrop bleeds through along the transition region unless an expensive isolated offscreen group
   layer is allocated.
2. **Deterministic byte-identity across painters.**
   Anti-aliased path edge sampling differs slightly across hardware drivers, GPU rasterizers, and CPU Skia pipelines.
   Aliased integer and path clipping guarantees exact binary pixel identity across multiple render threads and chunks.
3. **Soft reveals belong to element-local masks.**
   When authorial intent calls for an element to fade softly along an edge or shape, it uses an element-local `mask`
   with `feather: u64` ([ADR-0152](0152-a-mask-gains-invert-and-feather-and-takes-no-text-shape-or-image-source.md)),
   where the soft boundary is local to that single element.
4. `feather` is therefore refused by the schema on `transition`.

### 3. Parameters and frame-centered defaults for the new kinds

Each new kind accepts a minimal, closed set of static parameters:

| `kind` | Field | Type | Required? | Default | Valid Values / Constraints |
|---|---|---|---|---|---|
| `iris` | `shape` | string | No | `"circle"` | `"circle"`, `"rect"` |
| `iris` | `direction` | string | No | `"out"` | `"out"` (revealing outwards), `"in"` (closing inwards) |
| `iris` | `center` | `[i64, i64]` | No | `[W/2, H/2]` | Frame-space integer coordinates |
| `radial_wipe` | `direction` | string | No | `"clockwise"` | `"clockwise"`, `"counter_clockwise"` |
| `radial_wipe` | `start_angle` | `f64` | No | `0.0` | Degrees from 12 o'clock (straight up, $-y$), increasing clockwise |
| `radial_wipe` | `center` | `[i64, i64]` | No | `[W/2, H/2]` | Frame-space integer coordinates |
| `blinds` | `direction` | string | **Yes** | — | `"left"`, `"right"`, `"up"`, `"down"` (reusing `Direction`) |
| `blinds` | `slats` | integer | No | `10` | Positive non-zero integer ($S \ge 1$, `E-TRANSITION-SLATS-ZERO`) |

- **`iris` geometry**:
  - The maximum radius $R_{\max}$ is the distance from `center` to the furthest corner of the frame:
    $$R_{\max} = \max_{(x_c, y_c) \in \text{corners}} \sqrt{(x_c - x_0)^2 + (y_c - y_0)^2}$$
  - At progress $p \in [0, 1]$, the active radius is $r = p \cdot R_{\max}$ for `direction: "out"`, and $r = (1 - p) \cdot R_{\max}$ for `direction: "in"`.
  - For `shape: "circle"`, the aperture is an exact circle centered at `center`.
  - For `shape: "rect"`, the aperture is a rectangle centered at `center` with half-extents $w(p) = p \cdot \max(x_0, W - x_0)$ and $h(p) = p \cdot \max(y_0, H - y_0)$.
- **`radial_wipe` geometry**:
  - The sweep angle is $\theta(p) = p \times 360.0^\circ$.
  - The aperture is a circular sector wedge centered at `center` spanning from `start_angle` through `start_angle` $\pm \theta(p)$ (depending on `direction`), bounded by a radius $R_{\max}$ enclosing the canvas.
- **`blinds` geometry**:
  - The frame dimension along `direction` ($W$ for horizontal, $H$ for vertical) is partitioned into $S = \text{slats}$ bands of thickness $T = D / S$.
  - Within each slat $k \in [0, S-1]$, a moving edge travels $n_s = \lfloor p \times T \rfloor$ pixels in `direction`, forming $S$ complementary rectangular strips.

### 4. `wipe` gains optional `angle` as a mutually exclusive alternative to `direction`

`wipe` now accepts either a cardinal `direction` or an explicit sweep `angle`:
```json
{"id": "t2", "type": "transition", "start": 4000, "end": 4500,
 "kind": "wipe", "from": "clip_a", "to": "clip_b", "angle": 45.0}
```

- **`angle`** is an optional static `f64` in degrees:
  - `0.0`: edge travels rightward ($+x$);
  - `90.0`: edge travels downward ($+y$);
  - `180.0`: edge travels leftward ($-x$);
  - `270.0`: edge travels upward ($-y$).
- **Mutual exclusivity**:
  - Specifying both `direction` and `angle`, or specifying neither on `wipe`, is a validation error: **`E-TRANSITION-DIRECTION-OR-ANGLE`**.
- **Geometry**:
  - The total travel distance across the frame rectangle $[0, 0, W, H]$ at angle $\theta$ is:
    $$D(\theta) = W \cdot |\cos \theta| + H \cdot |\sin \theta|$$
  - A boundary line perpendicular to angle vector $(\cos \theta, \sin \theta)$ travels $n = \lfloor p \times D(\theta) \rfloor$ pixels along the travel vector.
  - The incoming and outgoing partitions form complementary half-plane polygons clipped against the canvas bounding box.

### 5. `Bridge` aperture abstraction and tool behavior

1. **`Bridge.cut` generalisation**:
   In `crates/montagent-core/src/transition.rs`, `Bridge.cut` generalises from `Option<Rect>` to an `Aperture` enum:
   ```rust
   pub(crate) enum Aperture {
       Rect(Rect),
       MultiRect(Vec<Rect>),
       Path(skia_safe::Path),
   }
   ```
2. **Painter clipping**:
   In `crates/montagent-render/src/canvas.rs`, the painter applies `Aperture`:
   - `Rect`: `canvas.clip_rect(rect, ClipOp::Intersect, false)`
   - `MultiRect`: clips the union of rectangular slat regions (`ClipOp::Intersect, false`)
   - `Path`: `canvas.clip_path(path, ClipOp::Intersect, false)`
   All clipping operations use `aa: false` (hard aliasing).
3. **Stacking**:
   Because `iris`, `radial_wipe`, `blinds`, and `wipe` (cardinal or angled) partition the canvas into strict complementary,
   non-overlapping regions ($A \cap B = \emptyset$ and $A \cup B = \text{Frame}$), both `from` and `to` work in either
   stacking order. Only `slide` requires `to` above `from` (`E-TRANSITION-SLIDE-UNDER`, ADR-0150).
4. **`query --at` and `validate` bounds**:
   `query --at` and `validate` spatial checks (such as `NOT COVERED` and `R-OFF-CANVAS`) evaluate against the
   **conservative axis-aligned bounding box** of the element's aperture.
5. **New validation codes**:
   - **`E-TRANSITION-DIRECTION-OR-ANGLE`**: `wipe` specifies both `direction` and `angle`, or specifies neither.
   - **`E-TRANSITION-UNSUPPORTED-KEY`**: a transition specifies keys invalid for its `kind` (e.g. `angle` on `iris`, or `slats` on `radial_wipe`).
   - **`E-TRANSITION-SLATS-ZERO`**: `blinds` specifies `slats: 0`.

## The four tests

| Invariant | How it passes |
|---|---|
| **Literal values** | `kind`, `shape`, `direction` are strings from closed lists. `angle`, `start_angle` are literal numbers. `center` is a pair of integers. |
| **Closed vocabulary** | Three new kinds (`iris`, `radial_wipe`, `blinds`), two shapes (`circle`, `rect`), and two radial directions (`clockwise`, `counter_clockwise`). |
| **Checkable by `validate`** | Key presence, mutual exclusivity, slat count $> 0$, and valid enum tokens are verified statically from the JSON without painting. |
| **Exact-string replace** | All new keys are flat, self-contained fields modifiable by single string replacement. |

## Consequences

- `crates/montagent-core/src/model/mod.rs` adds `Iris`, `RadialWipe`, and `Blinds` to `TransitionKind`, and defines the new transition parameter types.
- `crates/montagent-core/src/transition.rs` generalises `Bridge.cut` to `Aperture` and implements the geometric generators for iris, radial sweep, blinds slats, and angled wipes.
- `crates/montagent-render/src/canvas.rs` clips elements using the resolved `Aperture` with hard aliasing (`aa: false`).
- `crates/montagent-core/src/verbs/validate/` checks `E-TRANSITION-DIRECTION-OR-ANGLE`, `E-TRANSITION-UNSUPPORTED-KEY`, and `E-TRANSITION-SLATS-ZERO`.
- `CONTEXT.md` updates the **Transition** and **Direction** entries with the new kinds and parameters.
- Implementation graduates to a build ticket on map #896.
