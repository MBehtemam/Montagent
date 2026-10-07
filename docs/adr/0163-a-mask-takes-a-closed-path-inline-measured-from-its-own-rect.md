---
status: accepted
amends: 0084 (`shape` gains `path`, and gates a second field, `points`, beside `radius`; the rect is still the one set every shape shares), 0152 (the shape source it left waiting on paths enters, inline and closed)
---

# A mask takes a closed path inline, measured from its own rect

> **Amended by [ADR-0165](0165-a-hard-edged-inverted-mask-is-the-complement-off-the-edge-and-a-feathered-pair-sums-within-one-level.md)**: §5's "`invert` keeps exactly the complement" holds only where
> either mask keeps a pixel whole or erases it whole. On a hard antialiased edge the two may differ;
> a feathered pair sums within one level of 255.

[#715](https://github.com/MBehtemam/Montagent/issues/715), on the map
[#663](https://github.com/MBehtemam/Montagent/issues/663).
[ADR-0152](0152-a-mask-gains-invert-and-feather-and-takes-no-text-shape-or-image-source.md)
left a shape source waiting on paths: *"Any other figure is a path, and paths are still in the
map's fog."*
[ADR-0154](0154-a-point-list-enters-as-one-path-element-in-integer-pixels-from-the-declared-box.md)
has since placed a vertex list in integer pixels from a declared box, and
[ADR-0161](0161-a-text-element-bends-its-one-line-along-its-own-inline-path.md) wrote that
vocabulary inline on a second host. A mask today is a `circle`, `rect` or `ellipse` cut in one
element-local rect
([ADR-0084](0084-the-mask-rect-is-one-shape-independent-parameter-set.md)), with `invert` and
`feather`. Any other figure cannot be written.

**Precedent.** Row 7 of the precedent research
([#664](https://github.com/MBehtemam/Montagent/issues/664)): CapCut and Premiere both draw pen
(Bezier) masks on a clip, with feather and invert, and keyframe them to follow a subject. The
capability is inside the reference class, so
[ADR-0145](0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md)
asks for no accepted prototype.

Settled by one `/court` of three jurors (Opus, Sonnet, Fable), unanimous on all seven
questions, judged by how agents write, read, edit and validate the file, with the owner ruling
with the Judge's read and its four amendments.

## The decision

### 1. `shape: "path"`, written inline

`MaskShape` gains `path`. A path mask carries its vertices inline, in a field named
`points`:

```json
{"name": "mask", "shape": "path",
 "points": [{"at": [40, 300]}, {"at": [220, 40], "out": [60, 0]},
            {"at": [400, 280], "in": [-40, -60]}]}
```

- `points` is exactly ADR-0154's vertex list: `{"at": [x, y], "in": [dx, dy], "out": [dx, dy]}`,
  integer pixels, handles offset from their own vertex, a missing handle a zero offset, cubic
  segments.
- The mask never names a `path` element. A mask drawn from another element's outline is the
  cross-element dependency
  [ADR-0150](0150-wipe-slide-and-push-enter-as-transition-kinds-and-a-matte-from-another-element-is-refused.md)
  refused: its source would be an element that does not paint, and `validate` would follow an
  id into another element's keyframes. Two masks that want one outline each carry a copy.

### 2. The mask rect is the path's declared box

ADR-0084's rect stays the one parameter set every shape shares. Under `path` it plays the part
ADR-0154 §2 gives a path's declared box:

- `points` are measured from the rect's top-left corner.
- **The rect bounds the drawing and never scales it.** No viewBox enters.
- **Omitted, the rect is the element's own rect** (ADR-0084), so the points are element-local
  pixels. This is the common case.
- **A keyed `x` or `y` translates the whole mask** without keying a vertex.
- **`width` and `height` are static under `path`.** A keyframe list on either is a schema error
  whose message says to reshape the mask by editing its points. ADR-0154's reason carries over
  unchanged: resizing a box that does not scale its drawing changes nothing drawn.
- The rect is still all-or-none.

### 3. Always closed; `points` gated by `shape`

A mask keeps an inside, so **a mask path is always closed**: the last segment runs from the
last vertex back to the first, using the last `out` and the first `in`. There is no `closed`
field. A field with one legal value is noise an agent must write and `validate` must refuse.

- **`points` is required when `shape` is `path`, and an unknown key otherwise.** It is the second
  field `shape` gates, beside `radius` under `rect`. `radius` is an unknown key under `path`.
- This is not ADR-0084's refused two-level lookup. That refusal was of a field set that
  *varies* with `shape` (`circle{cx, cy, r}` against `rect{x, y, width, height}`). Here the rect
  is still shared by all four shapes, and `shape` adds one field that only one figure can
  read, as `radius` already does.
- **A stray `closed`** (the likely slip, from copying `{closed, points}` off a `path` element or
  a text's `path`) is a schema error as any unknown key is. Its message says: *a mask path
  always closes; drop `closed`.*
- Key order: `name, shape, x, y, width, height, radius, points, invert, feather`.

### 4. `points` animates as ADR-0154 and ADR-0162 say

`points` is an animatable property under
[ADR-0146](0146-an-animatable-property-is-one-the-schema-types-so-colour-blends-premultiplied-and-spring-easing-is-refused.md),
with no rule of its own:

- each keyframe value is a whole vertex list, with the same vertex count and the same handles
  per vertex in every value;
- values interpolate number by number;
- a morph between unlike outlines is padded by hand with coincident vertices
  ([ADR-0162](0162-a-morph-between-unlike-shapes-is-written-as-matching-vertex-lists-and-no-rule-resamples-them.md));
- an ease that overshoots clamps each absolute vertex and handle into the mask's box at that
  instant: the written rect, or the element's resolved rect at that instant when the rect is
  omitted, inset 0;
- `shift` refuses a cut inside a keyed `points` window where the resolved value is not integer.

A pen mask that follows a moving subject is the main reason both reference tools have one.

### 5. Coverage, `invert`, `feather`

- The kept area is the interior of the closed outline under the **nonzero** winding rule, the
  rule the `path` element fills with. One list of `points` therefore keeps, as a mask, exactly
  what it paints as a fill. A self-crossing outline keeps its overlap. A hole is a second,
  inverted mask, by the ring rule masks already have.
- `invert` keeps exactly the complement, and `feather` blurs the coverage by a Gaussian of
  σ = `feather` / 2, centred on the edge, as for every other shape (ADR-0152 §2).
- Masks in one list still intersect.

### 6. What `validate` says

The point checks of ADR-0154 §6 run on a mask's `points`, in every literal value, as ADR-0161
made them run on a text's `path`:

- **`E-PATH-TOO-FEW-POINTS`**: at least three vertices, since a mask path is closed.
- **`E-PATH-KEYFRAME-SHAPE`**: unchanged, naming the record and vertex that differ.
- **`E-PATH-OUTSIDE-BOX`**: every vertex and absolute handle lies in the mask's box with an
  inset of **0**, since a mask has no stroke. A feather that reaches past the box is fine, as it
  is for the other shapes. The box is:
  - the written rect, when one is written (its `width` and `height` are static, §2);
  - **the element's own rect when the rect is omitted.** Where the element's `width` or
    `height` is keyed (a `rect` or `ellipse` element), the check uses the **smallest** value of
    each, over its literal values, so the outline lies inside the element at every instant. The
    painter is unaffected: the box never scales or moves the points.
  - The finding names the mask's index in `effects`.
- **`E-PATH-DANGLING-HANDLE`** cannot arise: every vertex's `in` and `out` shape a segment on a
  closed path.
- **`R-MASK-ERASES-ALL`** stays `rect`-only. It catches the inverted bare rect whose omitted rect
  is the whole element. A path is written out vertex by vertex, so an outline that covers the
  element is deliberate, not that slip.
- **`R-MASK-CIRCLE-NON-SQUARE`** is `circle`-only and is untouched.
- **`query --at`** prints a path mask's resolved absolute control points (`at`, `at + in`,
  `at + out`) in box pixels, as it does for a `path` element.

No new finding code.

### 7. Byte-identical painting

No prototype gate. Both halves are measured: the path painter
([#750](https://github.com/MBehtemam/Montagent/issues/750)) and the feathered eraser composited
`DstIn` or `DstOut` ([#696](https://github.com/MBehtemam/Montagent/issues/696)), each
byte-identical across 1 to 10 painters with the bound hint on and off. A path mask draws the
same eraser from a path. The slice's tests carry the run: 1 to 10 painters, the bound hint on
and off, keyed `points`, `feather`, `invert` and `rotation`. If the run fails, the slice fails.

## The four tests

| Invariant | How a path mask passes |
| --- | --- |
| Literal values | Every vertex and handle is an integer pair in the file. Closing, winding, the clamp and the smallest-size box are fixed, documented rules. |
| Closed vocabulary | One enum value and one field, gated by `shape`. A referenced outline and a `closed` field are refused by name. |
| Checkable by `validate` | Shape errors are schema errors. Count, keyframe shape and containment are existing `validate` errors decided from the file alone. |
| Exact-string replace | A vertex is one object; moving it is one replace of its `at`. Moving the whole mask is one replace of `x` or `y`. |

## Considered and refused

- **A mask that references a `path` element by id.** Refused in §1, by ADR-0150.
- **Refusing the rect under `path`**, so points are always element-local. Refused: it breaks the
  one rect every shape shares, and an agent loses the one-field translation.
- **A rect that scales the points** (a viewBox). Refused for the `path` element by ADR-0154, and
  for the same reasons here: equal `points` would draw differently in two places.
- **`"closed": true` required, with `false` an error.** Refused in §3. Copying from a `path`
  element costs one deleted key, which the error message names.
- **Static `points`.** Refused: a pen mask that cannot follow its subject misses the reason the
  reference tools have one.
- **Even-odd winding for masks.** Refused: one outline would keep different areas as a fill and
  as a mask.
- **Extending `R-MASK-ERASES-ALL` to paths.** Refused in §6.
- **A measuring prototype.** Refused in §7.

## Consequences

- `CONTEXT.md`'s **Mask** entry gains the path shape, inline and always closed, measured from
  the rect.
- The schema: `MaskShape` gains `path`; `points` is required under `path` and an unknown key
  otherwise; `radius` and `closed` are unknown keys under `path`; `width` and `height` are
  static under `path`.
- `format.md`'s compositing page and the capability map describe the path mask.
- The hand-off spec is one slice: [A path mask](https://github.com/MBehtemam/Montagent/issues/768).
  Nothing gates it. The `path` element, mask `invert` and `feather` have shipped.
