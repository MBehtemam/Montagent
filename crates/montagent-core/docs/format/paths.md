# The Montagent project format: paths and stroke shapes

A page of `montagent://format.md`, which holds the rules every element shares and lists the
other pages; read it first. This page holds the rules for a `path` element, its stroke's join
and cap, and the dash pattern and trim a `path`, a `rect` and an `ellipse` take. Like the rest of the
format docs, every rule here is an accepted decision in the ADR series, cited inline by
number, and the ADR is right where the two disagree.

## Paths (ADR-0154, ADR-0158)

- **A `path` draws a list of vertices in integer pixels from its declared box's top-left
  corner.** It takes a `rect`'s fields except `radius`, plus a required `closed` (a boolean)
  and a required `points` list. The transform places the box exactly as a `rect`'s; the box
  **bounds** the drawing and never stretches it, so `width`, `height` and `closed` are
  static. Resize a path by editing its points, or animate `scale`.
- **A vertex is `{"at": [x, y], "in": [dx, dy], "out": [dx, dy]}`**, every value an
  integer, and no other key. `in` and `out` are optional **handles**: offsets from their
  own vertex. A missing handle is a zero offset, so a vertex with neither is a corner, and a
  written `[0, 0]` draws the same.
- **The same `points` list is also a mask's outline**: a `mask` with `"shape": "path"`
  writes it inline, measured from the mask's rect and always closed (ADR-0163). See the
  compositing page.
- **Each segment is a cubic Bezier** from one vertex's `at` through `at + out`, then the next
  vertex's `at + in`, to that `at`. A closed path adds the segment from the last vertex back
  to the first, using the last `out` and the first `in`. There is no `line` or `polygon`
  type: **a line is a two-vertex open path** with no handles, and a polygon is a closed path
  of corners. An open path needs two vertices and a closed one three
  (`E-PATH-TOO-FEW-POINTS`); an `in` on an open path's first vertex or an `out` on its last
  shapes nothing (`E-PATH-DANGLING-HANDLE`).
- **From SVG:** a `C c1 c2 p` segment from the vertex `prev` sets `prev.out = c1 − prev.at`,
  then adds a vertex `at = p` with `in = c2 − p`; an `L p` segment adds a vertex at `p` with
  no handles. A `d` string is not a value.
- **`fill` needs `"closed": true`**; on an open path it is a schema error. Fill uses the
  nonzero winding rule, so a self-intersecting outline fills its overlap. A gradient `fill`
  or `stroke` is measured against the declared box, as on every shape.
- **The stroke is centred on the outline**, open or closed (ADR-0158). `stroke_join` is
  `"round"` (default), `"bevel"` or `"miter"`; `"miter"` needs `stroke_miter_limit`, an
  integer 1–10, and a corner whose tip would reach past that many half-widths is beveled.
  Under keyed `points` a corner that sharpens past the limit snaps to a bevel on one frame.
  `stroke_cap` is `"butt"` (default), `"round"` or `"square"`, and draws only where the
  stroke ends: an open path's two ends, both ends of every dash, and a trim's two ends. All
  three are static and `path`'s alone. A miter without its limit, or a limit on another
  join, is `E-STROKE-MITER-LIMIT`; a cap on a closed path with no `stroke_dash`,
  `trim_start` or `trim_end` is `E-STROKE-CAP-UNDRAWN`; any of them with no `stroke`, or a
  `stroke_width` absent or 0 on every key, is `E-STROKE-NO-STROKE`.
- **The box contains the stroke:** with the inset `m = ceil(k × w / 2)`, every `at`,
  `at + in` and `at + out` lies in `[m, width − m] × [m, height − m]`, in every literal
  value of `points`. `w` is `stroke_width` (its largest key where keyed, 0 with no
  `stroke`). The **reach factor** `k` is the larger of the miter limit (for `"miter"`, else
  1) and √2 (for a `"square"` cap where a cap draws: an open path, or any dashed path or
  path carrying `trim_start` or `trim_end`; else 1). Width 3 under limit 10 gives `m = 15`; width 8 under a square cap, `ceil(4√2) = 6`.
  The bound is worst-case: it holds the margin even where every corner is gentle. Outside is
  `E-PATH-OUTSIDE-BOX`, naming the vertex, the record, the absolute position, `m` and `k`.
  Nothing is clipped to the box. Dashes change nothing else here: every dash end lies on
  the curve.
- **Keyed `points` is a whole list per keyframe**, interpolated number by number. Every
  value has the same vertex count and each vertex the same handles
  (`E-PATH-KEYFRAME-SHAPE`). An overshooting ease clamps each absolute vertex and handle
  into the inset box at that instant. `shift` cuts a keyed `points` only where every
  resolved number is an integer, and refuses elsewhere.
- **`query --at` prints a path's `path`**: its resolved vertices with absolute control
  points in box pixels, and its `stroke`: the inset `m` and `k` with its source. `NOT COVERED` counts the declared box, as for an `ellipse`.
- **A `text` takes the same vertex list as its own `path`**, a guide its one line bends
  along (ADR-0161), with the same four errors and its own inset; the text page has its
  rules.

## Morphing between unlike shapes (ADR-0162)

- **A morph is one keyed `points` list of matching vertex lists**: every value has the same
  vertex count, and each vertex the same handles. The lists are matched by hand; **nothing
  resamples an outline**, and a mismatch stays `E-PATH-KEYFRAME-SHAPE`.
- **Padding never changes a drawing.** A written `[0, 0]` draws the same as a missing handle,
  and a coincident vertex (the same `at` twice, no handles between them) adds a segment of
  zero length, which draws nothing. So the shorter list takes coincident vertices (the same
  `at` twice), and a handle one side lacks is written `[0, 0]`.
- **`closed` is static**, so one keyframe list never crosses it. A stroke-only morph between
  a closed and an open shape is one open path throughout, the closed shape written with its
  last `at` on its first. **A fill that opens is two elements**, joined by a cut or a
  `crossfade`: an open path takes no `fill`.
- **The seam.** Where an open path's first and last `at` coincide, the two ends meet as two
  caps, and `stroke_join` does not apply there. A `"round"` cap hides the seam. A butt cap
  (the default) is invisible where the seam is smooth and notches a corner seam; a
  `"square"` cap spurs one.
- **`R-PATH-SEAM-CAP` (`review`)** fires for a `path` with `"closed": false`, a `stroke`, no
  `stroke_dash`, and a `stroke_cap` absent, `"butt"` or `"square"`, once for each literal
  value of `points` (the static one, or each keyframe's, named by record and `t`) whose first
  and last `at` are equal and whose seam is a corner:
  - the **leaving direction** at vertex 0 is its `out` if non-zero; else the first non-zero
    of `v1.at + v1.in − v0.at` and `v1.at − v0.at`. A segment of zero length (its four
    control points equal) is passed over, and the next one, from `v1`, decides;
  - the **arriving direction** at the last vertex `vn` is `−vn.in` if non-zero; else the first
    non-zero of `vn.at − (vp.at + vp.out)` and `vn.at − vp.at` for the vertex `vp` before it,
    moving back past zero-length segments the same way;
  - the seam is **smooth** when the two have the same direction: cross product 0 and dot
    product above 0, in exact integers. Anything else is a corner;
  - where either end has no direction, every segment there being zero-length, it is silent.

  The fix is `"stroke_cap": "round"`, or a smooth seam. Under `stroke_dash` the pattern's
  phase decides whether the seam is inked, so the review is silent there (the dash seam
  below).

## Dashes (ADR-0158 §5)

- **`stroke_dash` is a list of lengths, dash, gap, dash, gap, starting with a dash**, on a
  `path`, a `rect` or an `ellipse`, never on text. It is static: 2 to 16 integer pixels,
  each at least 0, an even number of them with a total above 0. The list is drawn as
  written: an odd list is not doubled as SVG doubles it (write the doubled list out), and
  dashes are never stretched to fit the outline. An odd list or a zero total is
  `E-DASH-SHAPE`, and the message says which.
- **Its units are element pixels along the outline the stroke is drawn on**, as
  `stroke_width` is, so they scale with `scale`. On a `rect` or `ellipse` that is the inset
  outline, half the stroke width inside the box.
- **Where the pattern starts, and which way it runs** (SVG's own conventions):

  | Shape | Start | Direction |
  | --- | --- | --- |
  | `path` | `points[0].at` | in `points` order, through the closing segment when closed |
  | `rect` | the inset outline's top-left corner | clockwise on screen, top edge first |
  | `rect` with `radius` | where the top-left arc meets the top edge | clockwise on screen |
  | `ellipse` | 3 o'clock on the inset outline | clockwise, toward 6 o'clock first |

- **Each dash's ends take the cap.** On a `path` that is `stroke_cap`; on a `rect` or an
  `ellipse` it is always butt, and a rect's corners inside a dash stay square. A dash
  boundary that lands exactly on a vertex draws two ends and no join between them.
- **A zero-length dash draws a dot under `"round"`**, the dotted line:
  `"stroke_cap": "round", "stroke_dash": [0, 20]` on a `path`. Under `"square"` it draws a
  square, axis-aligned whatever way the path runs. Under butt it draws nothing, so it is
  `E-DASH-ZERO-BUTT`, named at the entry, including on every `rect` and `ellipse`: draw
  dots with a `path`. A zero gap is legal, but it is not a seamless line: two abutting dashes
  can leave a faint antialiased seam on a curve or a diagonal, and drop the join at a vertex.
- **`stroke_dash_offset` is how far into the pattern the outline's start falls**, in the
  same pixels, any integer. A larger offset moves the dashes **back**, toward the start; the
  painter wraps it into `[0, total)`, so `o` and `o + total` draw the same frame, and the
  file keeps what you wrote. Without `stroke_dash` it is `E-DASH-OFFSET-ALONE`. It is an
  animatable property, so `query --at`, `shift` and the contact sheet see it.
- **Marching ants** are an offset keyed linearly from `0` to a whole multiple of the total.
  With `"stroke_dash": [6, 4]` (total 10), `[{"t": 0, "v": 0}, {"t": 1000, "v": -40,
  "ease": "linear"}]` marches the dashes forward along the outline four times a second,
  and `"v": 40` marches them backward; either loops with no jump, because the last frame
  wraps onto the first. `shift` refuses a cut where the resolved offset is not a whole
  number, rather than round it.
- **The seam.** A pattern that does not divide a closed outline's length meets itself at the
  start point. Where the pattern is "on" there, the dash left over at the end and the first
  dash draw as **one longer dash**, turning a corner with the join if there is one; where it
  is "off", the leftover is a short gap. Move the seam by choosing `points[0]` on a path, or
  with the offset on any shape. Nothing checks it.
- **No stroke, no dash:** either field with no `stroke`, or a `stroke_width` absent or 0 on
  every key, is `E-STROKE-NO-STROKE`.
- **`query --at` prints a dashed shape's `stroke`** with `dash_offset`, the resolved offset
  as written (not wrapped), and `outline_length`, the painter's own measure of the outline at
  1:1. That length is informative, not a contract: it is a measure of a curve, not a number
  in the file, and on a curve it falls a little short of the exact one.

## Trim (ADR-0160)

- **A stroke can draw a window of its outline**, on a `path`, a `rect` or an `ellipse`,
  never on text. Three optional fields, each an animatable property: `trim_start` (default
  `0`) and `trim_end` (default `1`), numbers from 0 to 1 in every keyframe value, and
  `trim_offset` (default `0`), any number.
- **The unit is a fraction of the outline's length**, measured along the outline the dashes
  use, from the same start point in the same direction (the table under Dashes). `0.5` is
  half way round whatever the outline's size, and stays half way when a vertex moves.
- **The stroke is trimmed, the fill is not.** A trim field on an element with no stroke, or
  a `stroke_width` absent or 0 on every key, is `E-STROKE-NO-STROKE`.
- **Empty:** where `trim_start` ≥ `trim_end` no stroke is drawn at all, under any cap, so a
  draw-on's first frame shows no dot. A static window that never draws (neither field keyed,
  an absent one read at its default, so `"trim_start": 1` alone) is `E-TRIM-EMPTY`, which
  quotes both values. Keyed values may cross mid-animation: nothing draws while they do, and
  nothing checks it.
- **Full:** `0` to `1`, under any offset, draws the outline whole, the same bytes as no trim.
- **The clamp:** an overshooting ease that carries `trim_start` or `trim_end` past 0 or 1
  draws as 0 or 1.
- **`trim_offset` rotates the window forward around a closed outline**, in turns: `1` is one
  full circuit, and a quarter turn on a rect is a quarter of its length, not a corner. Any
  value is legal; the painter wraps it and the file keeps it. It needs a window and a closed
  outline: on a path with `"closed": false`, or with neither `trim_start` nor `trim_end`, it
  is `E-TRIM-OFFSET`, and the message says which.
- **Crossing the start point:** a window rotated across it is drawn as one stroke through it,
  turning there with the outline's own join, with no cap and no seam.
- **Caps draw at the trim's ends:** `stroke_cap` on a path, butt on a `rect` and an
  `ellipse`. A closed path carrying `trim_start` or `trim_end` takes a cap
  (`E-STROKE-CAP-UNDRAWN` does not fire), and a `"square"` one widens its inset by √2.
  `trim_offset` alone does not count.
- **Dashes stay put.** The pattern is laid along the whole outline first, as it is untrimmed,
  and the window reveals it: a trim end inside a dash takes the cap, one inside a gap draws
  nothing there, and a dash "on" across a closed outline's start point stays one dash through
  it. Only `stroke_dash_offset` moves the pattern. This is unlike After Effects, where the
  dashes travel with the trim start.
- **A draw-on** is `trim_end` keyed from 0 to 1, here on an open path with a round cap:
  `"stroke_cap": "round", "trim_end": [{"t": 0, "v": 0}, {"t": 1200, "v": 1, "ease":
  [0.65, 0, 0.35, 1]}]`. A draw-off keys `trim_start` from 0 to 1 the same way.
- **A ring loader** is a quarter of an `ellipse` orbiting: `"trim_start": 0, "trim_end":
  0.25, "trim_offset": [{"t": 0, "v": 0}, {"t": 3000, "v": 3, "ease": "linear"}]`. A whole
  number of turns loops with no jump.
- **`query --at` prints a trimmed shape's `trim`**: `trim_start`, `trim_end` and
  `trim_offset` at the instant, raw (not clamped, not wrapped), and `drawn`, the window the
  painter draws: `[a, b]` in fractions from the start point after the clamp and the wrap
  (`a > b` crosses the start point), `empty`, or `full`. An element with no trim field
  prints none of it. `shift` refuses a cut where an overshoot carries `trim_start` or
  `trim_end` past 0 or 1, and cuts `trim_offset` anywhere.
