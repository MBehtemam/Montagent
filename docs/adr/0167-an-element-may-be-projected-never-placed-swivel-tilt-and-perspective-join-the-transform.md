---
status: accepted
amends: 0040 (its "3D / perspective distortion" refusal on model grounds: a per-element projection enters as transform fields; a camera, depth sorting, skew and corner pin stay refused), 0012 (the flat transform gains `swivel`, `tilt` and `perspective`; skew stays out)
---

# An element may be projected, never placed: `swivel`, `tilt` and `perspective` join the transform

> **Amended by [ADR-0168](./0168-a-written-angle-always-projects-swivel-applies-before-tilt-and-strong-foreshortening-is-a-review.md)**: an element carrying `swivel` or `tilt` is always
> projected, even at 0°, and only an element with neither field is unchanged (§1). The angles
> compose as CSS's `perspective(d) rotateX(tilt) rotateY(swivel)` (§2). A review,
> `R-PROJECTION-SOFT`, fires when the near edge's magnification d / (d − r) exceeds 2 (§8).

[#705](https://github.com/MBehtemam/Montagent/issues/705), on the map
[#663](https://github.com/MBehtemam/Montagent/issues/663).
[ADR-0040](0040-effect-model-attachment-and-v1-vocabulary.md) refused "3D / perspective
distortion" on model grounds, because the format's spatial model is 2D.
[ADR-0145](0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md)
then changed how a capability enters: a precedent by mechanism in either tool, plus the four
invariants. Premiere's Basic 3D is that precedent (row 10 of
[#664](https://github.com/MBehtemam/Montagent/issues/664)). It has Swivel, Tilt and Distance to
Image, with no camera and no depth sort.

This ADR's thesis is that **an element may be projected, never placed**. Nothing gains a
position in depth, nothing is sorted and nothing occludes another by z. One flat element is drawn
as a plane turned in front of an eye.

Settled by four `/court` rounds of three jurors (Fable, Opus, Sonnet), judged by how agents would
work with each choice, with the owner ruling with the Judge's read. The round records are the
comments on #705.

## The decision

### 1. Three transform fields, on every visual element

`image`, `video`, `text`, `rect`, `ellipse` and `path` may carry:

| Field | Meaning | Default |
| --- | --- | --- |
| `swivel` | Degrees about the element's vertical axis. Positive sends the right edge away (CSS `rotateY`; Premiere's Swivel). | 0 |
| `tilt` | Degrees about the element's horizontal axis. Positive sends the top edge away (CSS `rotateX`; Premiere's Tilt). | 0 |
| `perspective` | The eye's distance from the element's plane, in px (CSS's `perspective`). | none: required whenever `swivel` or `tilt` is present |

All three are animatable numbers by
[ADR-0146](0146-an-animatable-property-is-one-the-schema-types-so-colour-blends-premultiplied-and-spring-easing-is-refused.md)'s
criteria. They join its single schema-derived list, so the renderer, `validate`, `query`, `shift`
and the contact sheet read them from one place. The angles are never normalised, as with
`rotation`.

`perspective` has CSS's meaning, not Premiere's. With both angles at 0 the element is unchanged
whatever `perspective` holds, and a smaller value only makes the foreshortening stronger.
`scale` stays the only control of size. Premiere's Distance to Image, which makes the image
recede, would be a second size control.

The capability is called **Projection** in the glossary and in check codes. `tilt` names one of
its two angles, never the capability.

### 2. About `origin`, applied first

The element turns about `origin`, the format's one pivot, and the eye sits in front of the origin
point. The projection is innermost. It acts on the element first, and `scale`, `rotation` and
`x`/`y` then act on the projected picture, as Premiere's Motion acts after Basic 3D. Every
existing transform field keeps its meaning: `rotation` spins the projected shape in the frame,
and `scale` never changes what the projection does. With `origin: "center-left"`, a swivel opens like a
door.

### 3. Drawn flat, then projected

The element is drawn flat into a layer, with its effects and its mask in list order, and the layer
is then projected. No effect ever runs under a perspective matrix, so a blur, shadow, mask, grain
or directional blur is projected with the element. Text and paths are rasterised flat and
projected through the same single path; they are never projected as vectors. Opacity, then the
blend, wrap the projected layer as they wrap every element
([ADR-0147](0147-a-blend-mode-is-a-flat-static-field-of-five-values-and-the-finished-element-blends-last.md)).

Named loss: a shadow that falls straight down from a tilted card cannot be written on that
element, because its shadow tilts with it. It needs a second element.

### 4. Facing away draws nothing

While the element faces away, or is exactly edge-on, it draws nothing. There is no field to change
this. A card flip is two elements: the front swivels 0 → 180 and the back −180 → 0. A mirrored
back, which Premiere and CSS show by default, is built as a second element with `scale: [-1, 1]`.

An easing that overshoots past 90° blinks the element out for those frames. That is correct, and
`query --at` shows it (§7).

### 5. No point reaches the eye

Let r be the distance from the origin point to the farthest corner of the element's box, widened
by every reach its effects declare. The layer that is projected contains the shadow offset and
the blur reach, so they count. `perspective` must exceed r at every key and every eased extreme
of `perspective`, of the keyed box and of each effect reach. The bound does not depend on the
angles, so it holds whatever they are. The painter therefore never needs to clip a point behind
the eye (CSS's w < 0 path), and that is the near-plane case most likely to break byte-identity.

For scale: a 1920 × 1080 element with no reach needs more than about 1102 about `center` and more
than about 1994 about `center-left`. Named cost: on an `image` or `video`, a strong perspective on
a large picture needs a smaller box scaled back up, which loses resolution.

### 6. What the checks measure

Every check that reads an element's frame-space box uses the axis-aligned bounds of its projected
quadrilateral. The quadrilateral is the four corners of the reach-widened box, projected about
`origin`, then carried through `scale`, `rotation` and `x`/`y`, computed exactly at each instant.
The checks concerned are `R-OFF-CANVAS`, the ink and collision checks, the layer tie and the
motion-blur still test. §5 keeps every corner in front of the eye, so the quadrilateral is convex
and its bounds contain everything the element paints. An element that faces away or is edge-on
paints nothing at that instant, as under `opacity: 0`.

### 7. What `query --at` reports

For a projected element, `query --at` prints:

- the resolved `swivel`, `tilt` and `perspective`;
- `facing`: `front`, `away` or `edge`;
- `corners`: the four frame-space corners in the box's order (top-left, top-right, bottom-right,
  bottom-left).

The `ink_box` slot holds §6's bounds. It is empty while `facing` is `away` or `edge`, and the
corners are still printed.

### 8. `validate`

| Code | Fires when |
| --- | --- |
| `E-PROJECTION-PERSPECTIVE-MISSING` | `swivel` or `tilt` is present without `perspective`. The schema enforces this too; the code gives the targeted message. |
| `E-PROJECTION-PERSPECTIVE-ALONE` | `perspective` is present with no `swivel` or `tilt` key at all. It is a dead value, by the precedent of `E-DASH-OFFSET-ALONE`. An angle written as `0`, static or keyed, counts as present. |
| `E-PROJECTION-EYE` | §5's bound fails. The message names the failing instant (the key or the eased extreme), r, and the smallest `perspective` that passes. |
| `R-PROJECTION-AWAY` (review) | No instant of the element's presence draws, evaluated at keys and eased extremes. A static pose that faces away or is edge-on is the plainest case. This is the counterpart of `R-MASK-ERASES-ALL`. |

There is no finding for an overshoot that blinks the element out near 90°. It would fire on
intended card flips, and `facing` already shows it.

### 9. What follows from earlier rules

None of these is a new rule; each is stated here so that it does not have to be derived.

- **Motion blur**
  ([ADR-0155](0155-motion-blur-is-a-per-element-field-that-accumulates-the-element-over-a-centred-shutter.md)):
  the samples evaluate the angles and `perspective` like every keyed value, and "still" includes
  them. A sample that faces away is a **transparent sample, and the divisor stays the full sample
  count**, so a card that passes 90° within one frame blurs toward transparency.
- **`shift`**: it splits the three fields as plain animatable numbers.
- **Transitions**
  ([ADR-0150](0150-wipe-slide-and-push-enter-as-transition-kinds-and-a-matte-from-another-element-is-refused.md)):
  slide and push offsets and wipe clips are frame space and apply outside the projection, as they
  apply outside `rotation`.
- **`clip`** stays a static frame-space rect, applied before everything.
- **A mask** rides the projection, because it is drawn flat with the element (§3).

### 10. A nest takes no projection

The projection sits on elements only. A projected element inside a
[nest](0166-capcut-and-premiere-nest-with-a-transform-and-adr-0001-refuses-the-clock-not-the-shared-space.md)
is projected first, and the nest's 2D matrix applies outside that, which is still a single
projective map. A projected nest, one group turned as one card, means flattening the group into
one layer first. That is group compositing, which nests refuse. It is left in the map's fog,
gated on the nest's score and on group compositing. This ruling holds whether or not nests ship.

### 11. A prototype gates the build, not this ADR

The precedent admits the capability. It does not show that the painter can draw it
deterministically. Before the build, a rendered prototype must:

- show that §3's flat-layer path is byte-identical across 1–10 painters with the bounds hint on
  and off, for a projected card with a blur, a shadow and a mask;
- include a card flip and a door swing;
- report the cost per frame at 1080p.

The owner judges it by eye.

## The four tests

| Invariant | How this keeps it |
| --- | --- |
| Literal values | Three numbers, written as literals or keyframe lists of literals. |
| Closed vocabulary | Three named fields with no enum. No camera, no depth and no fourth axis. |
| Checkable by `validate` | §5's bound is computed from literals at keys and eased extremes; §6's bounds and `facing` are exact at each instant. |
| Exact-string replace | Each field is one key; `E-PROJECTION-EYE` names the value that passes, so the fix is one edit. |

## Considered and refused

- **An `effects` member**, as in Premiere's Basic 3D, with list position deciding which effects
  are drawn flat and which act on the projected image. Refused, unanimously. A geometry switch
  hidden in list position cannot be seen by an agent editing by exact-string replace. It puts
  perspective inside the effect pipeline, where the bounds hint, the grain planner and the
  directional-blur crop all read the matrix. It also makes every later member's reach depend on a
  projected quadrilateral, and could never sit on a nest. Its one gain, a shadow that falls
  straight down, is the named loss in §3.
- **Premiere's Distance to Image** (the image recedes as the value grows). Refused: it is a second
  size control beside `scale`.
- **`rotation_x` / `rotation_y`**. One juror's choice: agents know CSS's axis names. Refused for
  `swivel`/`tilt` with CSS's signs and names in each field's description, because `rotate_x`
  beside `rotation` invites reading `rotation` as `rotate_z`. The collision of "tilt" with the
  capability's name is avoided by calling the capability Projection.
- **The box centre as pivot**, as in Premiere. Refused: it is a second, hidden pivot, and Premiere
  users complain that Basic 3D is "stuck in the middle".
- **The projection between rotation and scale.** Refused: foreshortening would then depend on
  `scale`, and §5's bound would need every eased scale extreme.
- **A mirrored back, or a `backface` field.** Refused: a mirrored back is rarely wanted (it shows
  reversed text) and can be built with a second element. A field widens the vocabulary for a
  case that is already buildable.
- **A plain positive minimum on `perspective`, with the painter clipping behind the eye as CSS
  does.** Refused: it ships the near-plane clipping path on day one, which is the riskiest code
  for byte-identity. A warning-level review of §5 was refused for the same reason, since the cases
  it let through would still need the path.
- **Checks on the flat box, or on the quadrilateral itself.** Refused. The flat box makes
  `R-OFF-CANVAS` and collision wrong for a swung card, and the quadrilateral would make every check
  handle a polygon. `corners` in `query --at` gives the exact shape when the bounds are too
  coarse.
- **Skew, corner pin, a camera, depth sorting.** Still refused. Skew and corner pin are free
  deformations with no reference-class precedent
  ([ADR-0012](0012-flat-transform-keyframes-carried-by-their-element.md)); a camera and depth
  sorting are After Effects-only and outside the map's destination.

## Consequences

- ADR-0040's "3D / perspective distortion" refusal and ADR-0012's field list are read through
  this ADR, and both carry amendment banners. Skew stays refused.
- The glossary gains **Projection**, and its **Transform** entry lists the three fields.
- The capability map gains one line, and the 2.5D can't-do item is retired. The rules go to
  `format.md`.
- The painter's bounds hint, grain planner and directional-blur crop never see a perspective
  matrix (§3). The prototype confirms this.
- A prototype ticket and one hand-off spec go on the map. The spec waits on the prototype.
