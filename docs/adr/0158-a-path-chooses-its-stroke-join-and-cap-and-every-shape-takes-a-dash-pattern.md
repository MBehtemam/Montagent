---
status: accepted
amends: 0154 (the round-join, butt-cap pin is lifted; the inset widens to the stroke's reach and `E-PATH-OUTSIDE-BOX` names it), 0014 (`rect` and `ellipse` take a dash pattern; their join and cap stay fixed), 0146 (`stroke_dash_offset` joins the one derived list of animatable properties), 0011 (`query --at` reports a path's inset and reach, the resolved dash offset, and an informative outline length)
---

# A path chooses its stroke join and cap, and every shape takes a dash pattern

> **Amended by [ADR-0160](0160-a-stroke-draws-a-window-of-its-outline-measured-in-fractions-of-its-length.md).** A cap also draws at a trim's
> ends: an element carrying `trim_start` or `trim_end` counts wherever §3 and §4 say "where a
> cap draws", so `E-STROKE-CAP-UNDRAWN` and the square-cap reach factor follow it.
> `E-STROKE-NO-STROKE` covers the three trim fields, and the dash pattern stays anchored to the
> outline under a trim.

[#711](https://github.com/MBehtemam/Montagent/issues/711), on the map
[#663](https://github.com/MBehtemam/Montagent/issues/663).
[ADR-0154](0154-a-point-list-enters-as-one-path-element-in-integer-pixels-from-the-declared-box.md)
centred a path's stroke and pinned it to a round join and a butt cap. Its containment check
holds only while the stroke reaches no further than half its width from the outline. That ADR
left this one to admit other joins and caps "by a wider inset or otherwise".

**Precedent.** The precedent research
([#664](https://github.com/MBehtemam/Montagent/issues/664), row 6) found none in CapCut or
Premiere for stroke dashes or caps, so
[ADR-0145](0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md)'s
entry test applies. Every part of this ADR is gated by a rendered prototype the owner accepts
(§8). The pinned Skia build has `PathEffect::dash` and the join and cap setters
([#665](https://github.com/MBehtemam/Montagent/issues/665)).

Settled by two `/court` rounds of three jurors each (Sonnet, Opus, Fable), with the owner
ruling with the Judge's read each time. In the first round one juror would have admitted only
`round` and `bevel` joins and `butt` and `round` caps, and only on `path`. The others voted
for the shape below. The second round adopted every proposal, with amendments. The jurors
judged each question by how agents write, read, edit and validate the file.

**Where the painter is now.** A path's stroke is drawn with `PaintJoin::Round` and
`PaintCap::Butt`. A `rect` or `ellipse` stroke is drawn on the box inset by half the width,
with Skia's default miter join (limit 4), so a rect's right-angled corners are square and sit
inside the box. Text strokes use round joins. This ADR changes only the first, and adds
dashes to the first two.

## The decision

### 1. Who takes what

| Field | `path` | `rect`, `ellipse` | `text` |
| --- | --- | --- | --- |
| `stroke_join`, `stroke_miter_limit` | yes | no | no |
| `stroke_cap` | yes | no | no |
| `stroke_dash`, `stroke_dash_offset` | yes | yes | no |

- On a `rect` or `ellipse`, the join and cap stay as the painter draws them now: square corners
  at a rect's corners, and butt ends on every dash. A join field there would add nothing a
  `radius` does not already say. A `stroke_join`, `stroke_cap` or `stroke_miter_limit` on
  either is a schema error, and its message says the field belongs to `path`.
- Text takes none of the five. Its stroke falls outside the glyph contour with round joins, a
  different mechanism, and a dashed glyph outline is not a look either editor offers. Text
  stroke is also per run, so it would need a run-level answer of its own.
- A dashed rounded box is a `rect` with a `radius`, not a hand-built path. Rebuilding one
  from cubics gives a different picture, and it is exactly the geometry an agent gets wrong.

### 2. Joins

`stroke_join` is `"round"`, `"bevel"` or `"miter"`. It is static and defaults to `"round"`,
ADR-0154's pin, so every existing path draws as it does now.

- `"miter"` requires `stroke_miter_limit`, a static **integer** from 1 to 10. It is the
  greatest miter tip distance from its vertex, in multiples of half the stroke width. A corner
  whose tip would reach further is drawn beveled. This is the SVG and Skia meaning.
- The limit is required, not defaulted to Skia's 4, because the inset in §4 depends on it.
  The number that widens the box's margin is written in the file.
- A limit with any other join is an error. It would otherwise sit there with no effect after
  an edit from `"miter"` to `"round"`.
- An integer rather than a number avoids `4` versus `4.0` under exact-string replace, and a
  fractional limit is not something anyone asks for. Limit 1 always bevels. It is legal, and
  it draws the same as `"bevel"`.

### 3. Caps

`stroke_cap` is `"butt"`, `"round"` or `"square"`, on `path` only. It is static and defaults to
`"butt"`, ADR-0154's pin.

- A cap draws in exactly two places: the two ends of an open path, and both ends of every
  dash. A `stroke_cap` on a closed path with no `stroke_dash` draws nowhere, so it is an error,
  even when it is `"butt"`. The finding names `closed` and `stroke_dash`, because an edit to
  either can make an existing cap draw or stop drawing.
- `"round"` and `"square"` extend past the end by half the stroke width. `"square"` reaches
  furthest at its corners, up to √2 times half the width along one box axis on a diagonal line.

### 4. Containment: a uniform inset from the stroke's worst-case reach

ADR-0154's check keeps its shape: every vertex `at`, and every absolute handle `at + in` and
`at + out`, lies in `[m, width − m] × [m, height − m]`. Only `m` changes:

```
m = ceil(k × w / 2)
```

- `w` is `stroke_width`, or its **largest** keyed value when keyed.
- `k` is the **reach factor**, the larger of:
  - the **join factor**: `stroke_miter_limit` for `"miter"`, otherwise 1;
  - the **cap factor**: √2 for `"square"` **where a cap draws** (an open path, or any dashed
    path), otherwise 1.
- With no stroke, `m` is 0, as before.

**Why it is sound.** Every ink point lies within `k × w / 2` of the curve. Round and bevel
joins and butt and round caps reach `w / 2`, a miter reaches at most its limit times `w / 2`,
and a square cap's corner at most √2 times `w / 2`. Every point of the curve lies in the
control points' hull, and the hull lies in the inset box. Dashes change nothing here: every
dash end and every join inside a dash lies on the curve.

**Why not a check per corner.** A miter's reach depends on the corner's angle. Under keyed
`points` the frames in between can turn a sharper corner than any keyframe does. A check run
on the literal values would therefore be unsound, and its tangent arithmetic would be one an
agent cannot redo by reading the file. The uniform inset depends on no angle, so no frame in
between can break it.

**The cost** is a looser box. A miter limit of 10 with `w` 20 needs a 100-pixel margin even
where every corner is gentle, and a square cap widens the margin by about 41% even on a level
line. The agent chooses that cost when it writes the limit or the cap. `E-PATH-OUTSIDE-BOX`
(§7) says the bound is worst-case, so the agent does not hunt for an overlap that is not there.

### 5. The dash pattern

`stroke_dash` is a list of lengths that alternate dash, gap, dash, gap, starting with a dash.

- **Shape.** An even number of entries, from 2 to 16. Each entry is an integer pixel count of
  at least 0, and the total is above 0. It is static. A keyed list has no whole-list rule
  worth having, and the offset carries the motion.
- **No doubling.** An odd list is an error. SVG repeats an odd list twice, which would make
  the pattern drawn differ from the literal in the file. An agent translating SVG writes the
  doubled list out.
- **Units.** The pattern is in element space, as `stroke_width` is, so it scales with `scale`.
  On a `rect` or `ellipse` it is measured along the inset outline the stroke is drawn on.
- **Zero entries.** A zero-length **dash** draws a dot under a `"round"` cap and a square
  under `"square"`. That is the dotted-line idiom. Under `"butt"` it draws nothing, so it is
  an error. That includes every `rect` and `ellipse`, whose cap is fixed at butt, and there the
  message says to use a `path` with a `"round"` cap. A zero **gap** is legal: it joins two
  dashes, which is what it says.

`stroke_dash_offset` is how far into the pattern the outline's start falls, in the same
pixels.

- It follows the SVG and Skia dash phase. A larger offset moves the dashes **back**, toward
  the outline's start. It is any integer, negative allowed, and the painter wraps it into
  `[0, total)`. The file keeps the literal the agent wrote.
- It is an **animatable property** under
  [ADR-0146](0146-an-animatable-property-is-one-the-schema-types-so-colour-blends-premultiplied-and-spring-easing-is-refused.md),
  so it joins the one derived list. Marching ants are an offset keyed from `0` to `−N × total`
  with a linear ease. That loops with no jump, and the sign picks the direction. `format.md`
  gives the example in both signs.
- `shift` refuses a cut where the resolved offset is not a whole number, as ADR-0146 has it for
  every keyed number.
- `stroke_dash_offset` without `stroke_dash` is an error.

**Where the pattern starts and which way it runs.** These are SVG's own conventions, so a
translated SVG dashes the same:

| Shape | Start | Direction |
| --- | --- | --- |
| `path` | `points[0].at` | in `points` order, through the closing segment on a closed path |
| `rect`, no `radius` | the inset outline's top-left corner | clockwise on screen, along the top edge first |
| `rect` with `radius` | where the inset outline's top-left arc meets its top edge | clockwise on screen |
| `ellipse` | 3 o'clock on the inset outline | clockwise on screen, toward 6 o'clock first |

The painter builds each outline with that start and direction, not with Skia's default start
index for a rect, rounded rect or oval. That keeps the result tied to this table, not to a
library default that a Skia update could move.

**The seam.** A pattern that does not divide a closed outline's length leaves a short dash or
gap at the start point. It is left as drawn and documented in `format.md`, which says where it
lands. On a path the agent can move it by choosing `points[0]`, and on any shape it can move it
with the offset. No check judges it. The length of a cubic or an ellipse is not an exact
number in the file, and a check that fired only for square-cornered rects would teach a rule
that does not hold for most outlines. Rescaling the dashes to fit the length, as Illustrator's
"align dashes to corners" does, is refused, because the lengths drawn would no longer be the
ones written.

### 6. Stroke fields with no stroke

`stroke_join`, `stroke_miter_limit`, `stroke_cap`, `stroke_dash` and `stroke_dash_offset` on an
element with no `stroke` are errors naming the field. So are they when `stroke_width` is a
static 0, or keyed with every key 0. Such a width never draws, so it is the same case. A keyed
`stroke_width` that only passes through 0 is fine, because the fields matter on the frames
where it is above 0.

### 7. `validate` and `query`

The schema states what it can. That covers each field's type, enum and range, the list's 2 to
16 integer entries, and each element type's field set, so a join on a `rect` is an unknown-key
error whose message points to `path`. `validate` checks the rest as errors, each naming the
location of the field it is about:

- **`E-STROKE-NO-STROKE`**: a stroke-shaping field with no stroke to shape (§6).
- **`E-STROKE-MITER-LIMIT`**: `"miter"` with no `stroke_miter_limit`, or a limit with any
  other join.
- **`E-STROKE-CAP-UNDRAWN`**: `stroke_cap` on a closed path with no `stroke_dash`. The finding
  names `closed` and `stroke_dash`.
- **`E-DASH-SHAPE`**: an odd number of entries, or a zero total. The message says which.
- **`E-DASH-ZERO-BUTT`**: a zero-length dash under a butt cap, on a path or a `rect` or
  `ellipse`.
- **`E-PATH-OUTSIDE-BOX`** (ADR-0154) keeps its fields and adds `k` and where it came from:
  the join and its limit, the square cap, or neither. For example: *inset 15 = ceil(10 × 3 / 2),
  from `stroke_miter_limit` 10; this bound is worst-case, not a measured overlap.*

**`query --at`** reports:

- a path's inset `m` and reach factor `k`, with its source, beside the absolute control points
  ADR-0154 has it report;
- the resolved `stroke_dash_offset` on any dashed shape, **raw**, not wrapped, so it matches
  what the agent wrote;
- an informative **outline length** for a dashed shape, labelled as not a contract. It must
  come from the same path measure the painter dashes with. If `query` cannot reach that
  measure, the spec drops the figure rather than compute a second length that could disagree
  with what is drawn.

No new `review`. The seam is a design choice no tool can resolve, and the outline length helps
the agent more than a warning would.

### 8. Gate: a rendered prototype the owner accepts

No part of this ADR is built before a prototype is accepted. It shows:

- miter joins at limits 1, 4 and 10 against the box edge;
- square caps on a 45° line;
- a keyed-`points` miter path whose frames in between turn a sharper corner than any keyframe,
  rendered across its whole time range, with the ink inside the box on every frame;
- a dashed rect, a dashed rounded rect and a dashed ellipse, from the start points in §5;
- a closed path's dash seam;
- a dotted line from zero dashes under round caps;
- marching ants;
- the `validate` text of every new error, which agents work through as much as through pixels;
- a regression check that existing `rect`, `ellipse` and `path` files render byte-identically
  to before.

Output must be byte-identical across 1 to 10 painters, with the bound hint on and off.

The build lands in two slices so that a rejection of dashes is a clean deletion: joins, caps
and the widened inset first (§2 to §4, and the `validate` and `query` items for them), then
dashes (§5, the rest of §6 and §7).

## Invariants (ADR-0145)

| Invariant | How it is kept |
| --- | --- |
| Literal values | Join, cap, limit and dash lengths are written literals. Nothing is defaulted that a check depends on: the miter limit is required, and dashes are never doubled or stretched. The offset's wrap happens at paint time, and the file keeps the literal. |
| Closed vocabulary | Three joins, three caps, one dash list, one offset, with per-type field sets. No `line_style` names, no fit-to-corners mode. |
| Checkable by `validate` | Containment is one formula over the file's literals, sound under animation. Every field that would draw nothing is an error. The seam is documented, not checked, because no exact length is in the file. |
| Exact-string replace | Each is one flat field. A dash list is one array literal. Switching to `"miter"` is two edits, and `validate` names the second. |

## Considered options

- **Only `round` and `bevel` joins and `butt` and `round` caps, with the current inset** (one
  juror). It keeps today's check exact, but it refuses the sharp miter corners that arrows,
  stars and polylines are drawn with, and the square ends a translated SVG carries. The uniform
  inset admits them soundly. That juror agreed it is the right mechanism if they enter.
- **An exact check per vertex from its tangents.** Refused: unsound under keyed `points`, and
  not something an agent can redo by reading the file (§4).
- **Clipping the stroke to the box.** Refused for ADR-0154's reason: it hides ink the agent
  cannot see or check.
- **Dashes on `path` only.** Refused: a dashed box or circle is the commonest dashed outline,
  and rebuilding a rounded rect or an ellipse from cubics is a worse picture and an agent's
  likeliest mistake.
- **Joins and caps on `rect` and `ellipse` too.** Refused: a rect's corner is already square or
  `radius`-round, an ellipse has none, and a cap on their dashes would need the square-cap
  inset on a shape whose stroke falls inside a box with no inset check.
- **Dashes on text.** Refused (§1).
- **A defaulted miter limit (Skia's 4).** Refused: the margin would depend on a number the
  file does not carry.
- **Doubling an odd dash list, as SVG does.** Refused: the pattern drawn would not be the
  literal.
- **A keyed `stroke_dash` list.** Not admitted: the offset gives marching ants, and a keyed
  list would need a rule for lists that change shape.
- **Fitting the dashes to the outline's length.** Refused: the lengths drawn would not be the
  ones written.
- **A `review` for the seam.** Refused (§7).
- **A field choosing where a shape's pattern starts.** Not admitted: the offset, and
  `points[0]` on a path, already move the seam.
- **Splitting dashes into their own ticket.** Not needed: the two slices make dropping dashes a
  clean deletion.
