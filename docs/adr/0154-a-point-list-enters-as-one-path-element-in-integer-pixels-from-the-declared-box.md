---
status: accepted
amends: 0014 (`line`, `polygon` and `path` are no longer rejected; one `path` element enters, and the `d`-string refusal stands)
---

# A point list enters as one `path` element, in integer pixels from the declared box

> **Amended by [ADR-0158](0158-a-path-chooses-its-stroke-join-and-cap-and-every-shape-takes-a-dash-pattern.md).** The round-join, butt-cap pin is lifted: a path
> chooses `stroke_join` and `stroke_cap`, the inset of §4 widens to `ceil(k × w / 2)` by the
> stroke's reach factor, and `E-PATH-OUTSIDE-BOX` names that factor.

> **Amended by [ADR-0161](0161-a-text-element-bends-its-one-line-along-its-own-inline-path.md).** The point vocabulary is also written
> inline on `text`, as the curve a line of text follows. `E-PATH-TOO-FEW-POINTS`,
> `E-PATH-DANGLING-HANDLE`, `E-PATH-KEYFRAME-SHAPE` and `E-PATH-OUTSIDE-BOX` fire on a text's
> `path` too, the last with the text's own inset.

> **Amended by [ADR-0162](0162-a-morph-between-unlike-shapes-is-written-as-matching-vertex-lists-and-no-rule-resamples-them.md).** §3's morphing question is settled with no
> second mechanism: a morph between unlike shapes is written as matching vertex lists, padded
> by hand with coincident vertices. `E-PATH-KEYFRAME-SHAPE` names that fix, and a new review,
> `R-PATH-SEAM-CAP`, fires on an open path whose coincident ends meet at a corner under a cap
> that is not `round`.

[#701](https://github.com/MBehtemam/Montagent/issues/701), on the map
[#663](https://github.com/MBehtemam/Montagent/issues/663).
[ADR-0014](0014-stroke-is-paint-the-text-box-is-required.md) rejected `line`, `polygon` and
`path`: *"A point list has no declared extent. Admitting one is a new ADR about placement,
not a schema addition."* It said a local coordinate space needed either a **second unit
system** or an **extent derived from the content**, and that both were closed. This is that
ADR about placement.

The premise had a gap. The space ADR-0014 weighed was a viewBox, which scales its contents
into the box. Points written in **integer pixels measured from the declared box's top-left
corner** need neither closed thing. They are the same pixels as `x` and `width`, so there is
no second unit. The box is still declared by `width` and `height`, and the points are checked
against it, never used to derive it. The points are content inside a declared box, as glyphs
are inside a text box.

**Precedent.** Row 6 of the precedent research
([#664](https://github.com/MBehtemam/Montagent/issues/664)): CapCut and Premiere both draw
Bezier shapes with a pen tool, with fill and stroke. The capability is inside the reference
class, so
[ADR-0145](0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md)
asks for no accepted prototype. The pinned Skia build has everything the painter needs
([#665](https://github.com/MBehtemam/Montagent/issues/665)).

Settled by one `/court` of three jurors (Sonnet, Opus, Fable), unanimous on all six
questions, with the owner ruling with the Judge's read. The jurors judged each question by
how agents write, read, edit and validate the file.

## The decision

### 1. One element type, `path`

A `path` is a visual element with the usual transform (`x`, `y`, `origin`, `width`,
`height`, `scale`, `rotation`, `opacity`) and every other field a `rect` takes except
`radius`. It adds two required fields:

- **`closed`**, a boolean. It is static. A path that opens mid-clip is two elements.
- **`points`**, a list of **vertices**, each an object:

  ```json
  {"at": [x, y], "in": [dx, dy], "out": [dx, dy]}
  ```

  - `at` is the vertex, in integer pixels from the declared box's top-left corner.
  - `in` and `out` are optional **handles**: integer pixel offsets from their own vertex.
    `out` shapes the segment leaving the vertex and `in` the segment arriving at it. Each
    segment is a cubic Bezier from `at` through `at + out`, then the next vertex's
    `at + in`, to that `at`.
  - A missing handle is a zero offset, so a vertex with neither is a corner and two such
    vertices make a straight segment. A written `[0, 0]` is legal and draws the same thing.
    It lets a keyframe that holds a corner match one that holds a curve (§3).
  - On a closed path a last segment runs from the last vertex back to the first, using the
    last `out` and the first `in`.

There is no `line` or `polygon` type. A line is a two-vertex open path with no handles, and
a polygon is a closed path of corners. A second spelling of one drawing breaks exact-string
replace, and a closed vocabulary does not carry names that add no capability.

An open line, for example:

```json
{"id": "rule", "type": "path", "start": 0, "end": 2000,
 "x": 960, "y": 540, "origin": "center", "width": 600, "height": 8,
 "closed": false, "stroke": "#FFFFFF", "stroke_width": 8,
 "points": [{"at": [4, 4]}, {"at": [596, 4]}]}
```

**Translating SVG.** An SVG `C c1 c2 p` segment from the vertex `prev` becomes
`prev.out = c1 − prev.at`, then a vertex with `at = p` and `in = c2 − p`. An `L p` segment
is a vertex at `p` with no handles. A `d` string is still refused as a value, for
ADR-0014's reasons.

**What the path does not take.** `radius` is a `rect` field and is refused. A gradient paint
on `fill` or `stroke` is measured against the declared box, as
[ADR-0149](0149-a-gradient-is-a-paint-linear-or-radial-measured-against-the-declared-box.md)
measures it for every shape.

### 2. Placement: integer pixels in a box that does not scale them

- The transform places the declared box exactly as it places a `rect`'s. `origin`, `scale`
  and `rotation` act on the box, so they act on the drawing with it. No placement rule is
  new.
- **Resizing the box does not move the points.** `width` and `height` bound the drawing.
  They do not stretch it.
- So **`width` and `height` are static on a `path`.** A keyframe list on either is a schema
  error, and its message says to resize a path by editing its points or to animate `scale`.
  This narrows [ADR-0146](0146-an-animatable-property-is-one-the-schema-types-so-colour-blends-premultiplied-and-spring-easing-is-refused.md),
  which admits a keyed shape size, to `rect` and `ellipse`.
- Fractions of the box were weighed and refused (see Considered options). A vertex is
  placement, and the format writes placement in pixels
  ([ADR-0012](0012-flat-transform-keyframes-carried-by-their-element.md)).

### 3. `points` is animatable as one whole list

`points` takes a keyframe list under ADR-0146. Each keyframe's value is a whole vertex list,
as `stops` is in ADR-0149.

- Every value in one keyframe list has the **same vertex count**, and each vertex carries
  the **same handles** (`in`, `out`, both or neither) in every value.
- Values interpolate number by number: each `at`, `in` and `out` coordinate is a number.
  Written values are integers, and resolved values in between may be fractional.
- `closed` is static, so every keyframe shares one topology.
- A morph between paths with different counts or handles is not written this way. It is the
  later morphing question, not a second mechanism.
- **Overshoot.** As ADR-0146 §5 requires, the fix lives in the one resolving function. A
  bezier ease that carries a resolved coordinate past the box clamps the absolute vertex and
  handle positions into the inset box of §4 at that instant, so the drawing never leaves its
  box.
- `shift` follows ADR-0146: it refuses a cut inside a keyed `points` window where the
  resolved value is not integer everywhere.

### 4. The stroke is centred, and the box contains it

On a `rect` or `ellipse` the stroke falls inside the declared box (ADR-0014). "Inside" means
nothing on an open line, so **a path's stroke is centred on its outline**, for open and
closed paths alike. A path takes `fill`, `stroke` and `stroke_width` with a `rect`'s rules
otherwise.

ADR-0014's promise that a stroke never enlarges the declared box is kept by a check, not by
where the stroke falls:

- Let the inset be `m = ceil(stroke_width / 2)`, or 0 with no stroke.
- Every vertex `at`, and every **absolute handle** `at + in` and `at + out`, lies in
  `[m, width − m] × [m, height − m]`.
- A cubic Bezier lies inside the convex hull of its control points, and every ink point of a
  stroke lies within `stroke_width / 2` of the curve, provided the join and cap reach no
  further. So the check is sound without painting a frame.
- **Until the stroke ticket decides joins and caps, the join is round and the cap is butt.**
  A miter join or a square cap reaches past `stroke_width / 2`, which would make the
  containment false. The stroke ticket that admits them must keep it, by a wider inset or
  otherwise.
- **Keyed values.** The check runs against every literal value of `points`, static or a
  keyframe's. When `stroke_width` is keyed, the inset uses its largest keyed value.

The path therefore has a different stroke rule from `rect` and `ellipse`. That is deliberate:
a single rule of "inside for closed, centred for open" would make one element type stroke two
ways depending on a boolean.

### 5. Fill needs a closed path

`fill` on a path with `"closed": false` is a schema error. SVG closes an open path silently
to fill it, and the format does not draw what the file does not say. An open path takes only
`stroke`, and a path with neither `fill` nor `stroke` is a schema error naming both, as on
the other shapes.

Fill uses the **nonzero** winding rule, Skia's and SVG's default, so a self-intersecting
outline fills its overlap.

### 6. `validate`

What the schema states:

- `closed` and `points` are required; each vertex has an integer `at` pair and optional
  integer `in` and `out` pairs, and no other keys.
- `fill` with `"closed": false` is an error.
- `width`, `height` and `closed` are static on a path; `radius` is refused.

What the schema cannot state, so `validate` checks it as an error:

- **`E-PATH-TOO-FEW-POINTS`:** an open path has at least two vertices and a closed path at
  least three, in every literal value.
- **`E-PATH-DANGLING-HANDLE`:** on an open path, an `in` on the first vertex or an `out` on
  the last shapes no segment. Nothing in the format is silently ignored.
- **`E-PATH-KEYFRAME-SHAPE`:** the values of one `points` keyframe list differ in vertex
  count or in which handles a vertex carries. The finding names the first record and the
  first vertex that differ.
- **`E-PATH-OUTSIDE-BOX`:** a vertex or absolute handle outside the inset box of §4. The
  finding names the vertex index, the keyframe record if any, the offending absolute
  position in box pixels and the inset it was measured against.

**`query --at`** reports a path's resolved vertices with their **absolute** control points
(`at`, `at + in`, `at + out`) in box pixels, beside the box's frame placement it already
reports. An agent reading a containment error then does not have to add offsets up.

`NOT COVERED` counts a path's declared box, as it counts an `ellipse`'s. No check judges what
a path looks like inside its box.

## Invariants (ADR-0145)

| Invariant | How a path keeps it |
| --- | --- |
| Literal values | Every vertex and handle is an integer pair written in the file. Segment geometry, the clamp and the inset are fixed, documented arithmetic. |
| Closed vocabulary | One element type with fixed keys. `line`, `polygon`, segment commands and a `d` string are not in it. |
| Checkable by `validate` | Shape errors are schema errors. Count, dangling handles, keyframe shape and containment are `validate` errors decided from the file alone. |
| Exact-string replace | A vertex is one object. Moving it is one replace of its `at`, and its relative handles move with it. |

## Considered options

- **Keep the rejection.** Refused: its reason does not reach box-local pixels, and it blocks
  stroke dashes, trim, text on a path, morphing and a shape-sourced mask.
- **Points as fractions of the box**, as ADR-0149 places gradient geometry. A resize would
  stretch the drawing for free. Refused: an endpoint is no longer readable as a pixel, a
  one-pixel nudge needs arithmetic against the box, and ADR-0012's reason against fractions
  for placement returns. A gradient is paint over the box; a vertex is placement.
- **SVG-style segment commands as JSON objects** (`move`, `line`, `cubic`). Refused: a
  vertex's shape is split across two neighbouring commands, so moving one vertex takes two
  coordinated edits. Commands also bring a `move` pseudo-point and subpaths, and give
  animation no vertex to match.
- **Absolute handle positions.** Refused: moving a vertex would mean editing three pairs.
- **Separate `line` and `polygon` types.** Refused: two spellings of one drawing.
- **An inside stroke on a closed path.** Refused: one element type would stroke two ways.
- **Clipping the stroke to the box.** Refused: it keeps the box honest by hiding ink the
  agent cannot see or check.
- **Fill on an open path, closing it implicitly.** Refused: the file would not say what is
  drawn.
- **A keyed `width` or `height` on a path.** Refused: it would change nothing drawn, or
  bring back a scaling box.
