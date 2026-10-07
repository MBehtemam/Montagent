---
status: accepted
amends: 0158 (a cap also draws at a trim's ends, so `E-STROKE-CAP-UNDRAWN` and the square-cap reach factor follow; `E-STROKE-NO-STROKE` covers the trim fields), 0146 (`trim_start`, `trim_end` and `trim_offset` join the one derived list of animatable properties), 0011 (`query --at` reports the raw trim values and the window drawn)
---

# A stroke draws a window of its outline, measured in fractions of its length

[#712](https://github.com/MBehtemam/Montagent/issues/712), on the map
[#663](https://github.com/MBehtemam/Montagent/issues/663).
[ADR-0154](0154-a-point-list-enters-as-one-path-element-in-integer-pixels-from-the-declared-box.md)
admitted the `path`, so a stroke can now **draw on**: a line that grows along its own length,
or an arc that orbits a ring. This ADR decides how a stroke draws only part of its outline.

**Precedent.** The precedent research
([#664](https://github.com/MBehtemam/Montagent/issues/664), row 6) found no trim-path in
CapCut or Premiere, so
[ADR-0145](0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md)'s
entry test applies. Nothing here is built before a rendered prototype is accepted (§8). After
Effects' Trim Paths and Lottie are the nearest outside precedent; the pinned Skia build has
`SkTrimPathEffect` and `ContourMeasure` ([#665](https://github.com/MBehtemam/Montagent/issues/665)).

Settled by three `/court` rounds of three jurors each (Sonnet, Opus, Fable), with the owner
ruling with the Judge's read each time. Every question was unanimous; one split, on whether
`trim_offset` exists, was carried to the next round and settled there. The jurors judged each
question by how agents write, read, edit and validate the file.

**Why dashes are not enough.** A draw-on is already reachable in SVG with a dash as long as
the outline: `stroke_dash: [L, L]`, with `stroke_dash_offset` keyed from `L` to `0`. That
needs `L`, and
[ADR-0158](0158-a-path-chooses-its-stroke-join-and-cap-and-every-shape-takes-a-dash-pattern.md)
records that an outline's length is not an exact number in the file. An agent would paste in
`query`'s informative length, `validate` could not tell whether the line lands flush or stops
a pixel short, and the next vertex edit would quietly break it.

## The decision

### 1. Who takes it

`path`, `rect` and `ellipse` take trim. Text does not, for ADR-0158's reasons: its stroke is
per run and falls outside the glyph contour, so "the outline's length" is not one thing.

Trim is measured along **the same outline the dashes use**, from ADR-0158's start points and
in its directions: `points[0]` in point order on a path, the inset outline's top-left on a
`rect` (where the top-left arc meets the top edge with a `radius`), and 3 o'clock on an
`ellipse`, all clockwise. A ring loader is an `ellipse`; it is not rebuilt from cubics.

### 2. The fields and their unit

Three flat, optional fields, each an **animatable property** under
[ADR-0146](0146-an-animatable-property-is-one-the-schema-types-so-colour-blends-premultiplied-and-spring-easing-is-refused.md):

| Field | Value | Default | Where |
| --- | --- | --- | --- |
| `trim_start` | number in [0, 1] | `0` | `path`, `rect`, `ellipse` |
| `trim_end` | number in [0, 1] | `1` | `path`, `rect`, `ellipse` |
| `trim_offset` | any number, in **turns** | `0` | closed outlines only (§4) |

`trim_start` and `trim_end` are **fractions of the outline's length**, as the painter's path
measure finds it. The stroke draws the part of the outline between them.

**Why a fraction, against [ADR-0012](0012-flat-transform-keyframes-carried-by-their-element.md).**
ADR-0012 refused fractions for placement, because retargeting the frame silently stretches a
box. A trim fraction places nothing in the frame. It is a ratio of the element's own outline,
the same kind of value as `opacity` and `scale`, which ADR-0012 writes as floats. It is also
the only unit `validate` can bound from the file alone, and the only one whose meaning
survives an edit: move a vertex and "half drawn" is still half drawn.

Flat fields follow ADR-0012 and sit beside `stroke_dash_offset`. Each keys on its own, so a
draw-on names only `trim_end`. A block would be the format's first block with animatable
members; `motion_blur` is a block because it is static.

### 3. Trim shapes the stroke only

The fill is never trimmed. After Effects and Lottie trim the path and fill the open piece as a
chord, which is the implicit closing ADR-0154 §5 refuses: the format does not draw what the
file does not say. A trim field on an element with no stroke is `E-STROKE-NO-STROKE` (§7).

### 4. `trim_offset` rotates the window around a closed outline

Without an offset no window can cross a closed outline's start point. The spinning-arc loader
could then only be faked by keying start and end together with a jump at the seam.

- `trim_offset` is in **turns**: `1` is one full circuit. It is a fraction of length, like the
  other two, not an angle; a quarter turn on a rect is not a corner.
- It moves the window forward along the outline's direction. Any value is legal, negative
  included. The painter wraps it, and the file keeps the literal, as with `stroke_dash_offset`.
  A loader is an offset keyed linearly from `0` to `N`.
- A window that crosses the start point is drawn as **one continuous stroke** through it, with
  the outline's own join there and no caps.
- **Closed outlines only**: `rect`, `ellipse`, and `path` with `"closed": true`. On an open
  path it is an error. An open line has no round to wrap onto, and After Effects' wrap into two
  pieces, each with its own caps, is a picture no field states.
- It needs a window to rotate. With neither `trim_start` nor `trim_end` present, the window is
  the whole outline and an offset rotates it onto itself, drawing nothing different, so that is
  an error too.

### 5. Empty, reversed and past the ends

- The window is **empty** when the resolved start ≥ the resolved end. The offset never makes a
  window empty or non-empty.
- **Static.** When neither `trim_start` nor `trim_end` is keyed, and start ≥ end with an absent
  field read at its default, the element draws no stroke on any frame. That is
  `E-TRIM-EMPTY`. `"trim_start": 1` alone is caught.
- **Keyed** values may cross mid-animation, by different eases or key times. At an instant
  where the window is empty, nothing is drawn. There is no swap, because a swap draws a window
  the file never wrote. There is no error, because resolved instants cannot be checked
  exactly, and a deliberate draw-off is a crossing too.
- **An empty window draws nothing, under any cap.** Skia draws a dot for a zero-length segment
  under a round cap; the painter skips the stroke instead, so a draw-on does not pop a dot on
  its first frame.
- **Overshoot.** A bezier ease that carries `trim_start` or `trim_end` past 0 or 1 clamps to
  [0, 1] in the one resolving function, as ADR-0146 §5 requires. The schema bounds the written
  literals, keyframe values included.
- **Full window.** Start `0` and end `1`, under any offset, is the closed outline drawn whole,
  with no ends and no seam, byte-identical to the untrimmed element.

### 6. Caps and dashes

**A cap draws at a trim's ends.** ADR-0158 has a cap draw in two places, the ends of an open
path and both ends of every dash. A third joins them: **an element that carries `trim_start`
or `trim_end`**. Presence, not value, decides it, so `validate` reads it from the file without
resolving anything. Two rules follow that one clause:

- `E-STROKE-CAP-UNDRAWN` no longer fires on a closed path that carries `trim_start` or
  `trim_end`.
- The √2 square-cap factor in the reach factor `k` applies to such a path, so
  `E-PATH-OUTSIDE-BOX` keeps a square trim end inside the box.

`trim_offset` alone does not count: with no window it is an error anyway. On `rect` and
`ellipse` the cap stays butt, so their trim ends are butt. One accepted oddity: a closed path
with an explicit full window may carry a cap that draws only while the window is partial.
The containment it buys is conservative, never short.

**The dash pattern is anchored to the outline.** Dashes sit where they would sit untrimmed,
and the trim window reveals them. A trim end inside a dash takes the cap, and one inside a gap
draws nothing there. `stroke_dash_offset` alone moves the pattern; the trim fields alone move
the window. This differs from After Effects, which dashes the trimmed path so its dashes travel
with the trim start; `format.md` says so in one line. Dashes that travel with the window cannot
be written exactly, since that needs the outline's length in pixels.

### 7. `validate`, `query`, `shift`

**Schema.** `trim_start` and `trim_end` are numbers in [0, 1], keyframe values included.
`trim_offset` is any number. All three are unknown keys on `text`.

**New errors**, each naming the location of the field it is about:

- **`E-TRIM-EMPTY`**: a static window with start ≥ end (§5). The message quotes both values,
  saying which one is a default, so the replace target is in front of the agent.
- **`E-TRIM-OFFSET`**: `trim_offset` on an open path, or with neither `trim_start` nor
  `trim_end` present. The message says which.

**Widened errors** (ADR-0158):

- **`E-STROKE-NO-STROKE`** covers `trim_start`, `trim_end` and `trim_offset`.
- **`E-STROKE-CAP-UNDRAWN`** and the square-cap factor of **`E-PATH-OUTSIDE-BOX`** follow
  §6's clause.

No new `review`. A keyed window that draws nothing for a stretch shows in a render and in
`query`; a check that guessed at intent from keyframes would fire on every deliberate wipe-out.

**`query --at`**, only on an element carrying a trim field, so untrimmed output is unchanged
byte for byte:

- the resolved `trim_start`, `trim_end` and `trim_offset`, **raw**: the fractions before the
  overshoot clamp and the offset unwrapped, so each matches what the agent wrote;
- one derived line in one of three forms:
  - `drawn: [a, b]`: the window after the clamp and the wrap, as fractions of the outline
    from its start point. `a > b` means it crosses the start point;
  - `drawn: empty`;
  - `drawn: full`: the whole closed outline, with no ends.

**`shift`** needs nothing new. Under ADR-0146 a number splits exactly, and a split whose value
an overshoot carries past [0, 1] is refused. `trim_offset` is unbounded and always splits.
Motion blur samples keyed trim values with every other keyed value
([ADR-0155](0155-motion-blur-is-a-per-element-field-that-accumulates-the-element-over-a-centred-shutter.md)).

### 8. Gate, and the build

No part of this ADR is built before a prototype is accepted. It is built on the local branch of
[the stroke prototype](https://github.com/MBehtemam/Montagent/issues/750), since trim needs
ADR-0158's caps and dashes. It shows:

- an open-path draw-on under a round cap, with no dot on its first frame;
- a ring loader on an `ellipse` and on a `rect`, `trim_offset` keyed linearly across several
  turns, crossing the start point continuously (on the rect, a corner), with no cap or seam
  there;
- a closed `path` trimmed under a square cap, its ink inside the box on every frame;
- keyed start and end crossing under a round cap, drawing nothing in between;
- an overshooting bezier ease, clamped;
- a dashed line drawing on, its dashes held still;
- a full window under an offset, byte-identical to the untrimmed element;
- untrimmed `path`, `rect` and `ellipse` files, byte-identical to before;
- the `validate` text of `E-TRIM-EMPTY`, `E-TRIM-OFFSET` and each widened error;
- `query --at` captures of `drawn: empty` at a keyed crossing, `drawn: full` under an offset,
  and a crossing window with `trim_offset` past one turn.

Output must be byte-identical across 1 to 10 painters, with the bound hint on and off.

The build is **one slice**, after both of ADR-0158's slices (joins and caps, then dashes) and
the accepted prototype. Every trim rule then lives in one spec, which cites finished caps and
dashes rather than amending them mid-build. If the stroke prototype's branch has drifted from
what those slices built, the trim prototype is re-run against `main` before hand-off.

## Invariants (ADR-0145)

| Invariant | How it is kept |
| --- | --- |
| Literal values | Start, end and offset are written literals. The clamp and the wrap happen at paint time; the file and `query` keep the literal. No length is written or needed. |
| Closed vocabulary | Three flat fields on three element types. No trim mode, no individual/simultaneous switch, no fill trimming. |
| Checkable by `validate` | Bounds are schema; an empty static window, an offset with nothing to rotate or nowhere to wrap, and every cap and containment rule are decided from the file alone. Keyed crossings are defined, not checked. |
| Exact-string replace | Each field is one flat literal. A draw-on is one keyed `trim_end`; a loader is one keyed `trim_offset`. |

## Considered options

- **Refuse trim and teach the dash trick.** Refused: it needs a length the file does not carry
  (see above).
- **`path` only.** Refused: the commonest trims are a ring and a box border, and rebuilding an
  ellipse from cubics is busywork an agent gets wrong.
- **Trim on text.** Refused, as dashes are (§1).
- **Pixels of arc length.** Refused: `validate` cannot tell a value past the end, and the
  meaning moves with every vertex edit.
- **Integer thousandths.** Refused: a unit found nowhere else in the format, inviting `500`
  meaning a fraction or `0.5` meaning thousandths. `opacity` already lives with float literals.
- **A `trim` block.** Refused (§2).
- **Trimming the fill, as After Effects does.** Refused (§3).
- **No `trim_offset`.** Refused: the orbiting-arc loader would be written as a seam-jump hack
  in files that never migrate. Leaving a field out is cheaper to reverse, but its meaning was
  fully settled.
- **`trim_offset` on open paths, as After Effects wraps it.** Refused (§4).
- **Degrees for the offset.** Refused: it is a fraction of length, not an angle.
- **Swapping a reversed window.** Refused: it draws what the file does not say.
- **A `review` for keyed crossings, or a warning where both lists share a key time.** Refused:
  the first cries wolf on every draw-off, and the second covers only a coincidence.
- **Butt trim ends whatever `stroke_cap` says.** Refused: it ignores a field the file states,
  on the one element where a round draw-on is the look people want.
- **A dash pattern anchored to the trim start.** Refused: it couples two fields, and the
  uncoupled picture would become unwritable.
- **Trim blocked only by joins and caps, with the dash rule amended into the dashes slice.**
  Refused: it splits trim's rules across two specs.
