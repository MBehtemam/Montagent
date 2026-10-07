---
status: accepted
amends: 0154 (its point vocabulary is also written inline on `text`, and its four `E-PATH-*` errors fire there too), 0133 (on a text on a path, `align` names an end of the curve, not of the reading direction), 0014 (the derived-height checks do not apply to a text on a path; its box frames the curve), 0146 (`path_offset` and a text's `path.points` join the one derived list of animatable properties), 0011 (`query --at` and `measure` report the units a curve hides)
---

# A text element bends its one line along its own inline path

[#713](https://github.com/MBehtemam/Montagent/issues/713), on the map
[#663](https://github.com/MBehtemam/Montagent/issues/663).
[ADR-0154](0154-a-point-list-enters-as-one-path-element-in-integer-pixels-from-the-declared-box.md)
admitted a curve that can be written. This ADR decides whether a line of text may follow one,
and how the text meets the curve.

**Precedent.** The precedent research
([#664](https://github.com/MBehtemam/Montagent/issues/664), row 6) found that CapCut has one
"Curve" control bending text along an arc, and that Premiere documents nothing. Text on an
*arbitrary* path is After Effects only (Path Options: a mask as the path, First and Last Margin,
Reverse Path, all animatable). So
[ADR-0145](0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md)'s
entry test applies, and nothing here is built before a rendered prototype is accepted (§9).
Skia has no draw-text-on-path call; the construction is `ContourMeasure::pos_tan` per glyph
([#665](https://github.com/MBehtemam/Montagent/issues/665) §6), the same path measure
[ADR-0158](0158-a-path-chooses-its-stroke-join-and-cap-and-every-shape-takes-a-dash-pattern.md)
and [ADR-0160](0160-a-stroke-draws-a-window-of-its-outline-measured-in-fractions-of-its-length.md)
use, and the painter already draws one outline per glyph.

Settled by three `/court` rounds of three jurors each (Sonnet, Opus, Fable), with the owner
ruling with the Judge's read each time. Every question was unanimous; the Judge's reads folded
in the jurors' amendments, two of which (§4's direction-free `align` and §7's wording of the
inset) closed traps the questions had missed. The jurors judged each question by how agents
write, read, edit and validate the file.

## The decision

### 1. Text on an arbitrary path enters; no arc shortcut

A text element may follow any curve ADR-0154's vocabulary can write: a line, an arc, a wave, a
circle. There is no CapCut-style `curve` number. An arc is a short `points` list, and a second
spelling of the same picture is what a closed vocabulary refuses; a `curve` number would also
leave every wave and circle to a later capability. The arc is the prototype's first case, so
the one precedent there is gets tested on the way.

### 2. The curve is written on the text

The text takes its own field, `path: {closed, points}`, in ADR-0154's vocabulary: `points` is
a list of `{at, in?, out?}` in integer pixels from **the text's** declared box's top-left, with
handles as offsets from their own vertex. `closed` is required and static. The text's `path`
takes no other key: it is a guide, and it is never painted.

**No reference to another element.** An `"on_path": "<id>"` naming a `path` element was refused
on
[ADR-0150](0150-wipe-slide-and-push-enter-as-transition-kinds-and-a-matte-from-another-element-is-refused.md)'s
two reasons. A guide that should not show would be a `path` element with no stroke and no
fill, an element in a track that does not paint, which every tool would special-case. And the
text would inherit another element's transform, box, keyed `points` and time window: an edit
to one element would change another's look, and a text would go blank when its guide's clip
ended. A reference `validate` can resolve still cannot tell the agent what the text does at a
given frame.

A guide line that should show is a separate `path` element with the same points copied in. The
two copies can drift; that is the author's to keep. A `review` that warns when they differ may
come later if it is ever needed.

### 3. One line, bent rigidly

- **One line only.** A `\n` in any run of a text on a path is `E-TEXT-PATH-BREAK`. ADR-0007 has
  no automatic wrapping, so `\n` is the only way a line breaks, and the error covers all of
  them. Several lines stacked on offset curves (After Effects) fold back inside any tight
  bend, so the format would promise a picture it cannot define. A second line on the same curve
  is a second text element.
- **Rigid placement.** A glyph is moved and turned, never bent. A **joined piece**
  ([ADR-0153](0153-a-joined-piece-moves-as-one-and-joining-scripts-keep-their-ligatures-and-take-no-letter-spacing.md))
  and a ligature cluster are each one rigid body. Every body is placed at its **advance
  midpoint**, SVG `textPath`'s rule. On a tight curve the ends of a wide body lift off the
  curve. That is accepted behaviour, not a defect, and `format.md` says so. Warping outlines
  along the curve was refused: it has no precedent, costs more, and the per-glyph-outline
  painter cannot do it.

### 4. The curve bends the finished flat line

Layout, `letter_spacing`
([ADR-0151](0151-letter-spacing-is-an-animatable-element-field-and-a-stagger-is-a-units-block-on-one-text-element.md))
and the `units` stagger all run in flat block space **exactly as they do today**. The curve
then maps each body's flat pose:

- its midpoint's flat `x` becomes a **distance** `d` along the curve (§5);
- its flat baseline `y` becomes an offset along the curve's **normal** at `d`, the tangent
  turned 90° clockwise on screen, so negative `y` is to the left of travel and a straight,
  left-to-right curve gives back the flat line;
- its rotation adds to the **tangent angle** at `d`.

So a stagger's `y: -200` lifts a letter perpendicular to the curve, and its `x` slides the
letter along it. Pivots, `order`, `origin`, scale and opacity keep their ADR-0151 meanings,
because they are still worked out in flat space. After Effects' per-character animators and
SVG's `dx`/`dy` on a `textPath` act the same way. A stagger in screen axes would give the same
`y` two meanings, one flat and one on a path.

**Direction.** Increasing flat `x`, the line's visual left-to-right after bidi, always runs in
`points` order, for every script. There is no reverse flag: to read along the bottom of a
circle, or from inside it, write the points the other way. A clockwise circle in y-down screen
space puts letters on the outside. A `path_reverse` flag may join later if the prototype shows
that reversing a list by hand, swapping every `in` and `out`, is error-prone enough to earn a
field.

### 5. Where the line sits: `path_offset` and `align`

A text on a path takes one flat, optional field, an **animatable property** under
[ADR-0146](0146-an-animatable-property-is-one-the-schema-types-so-colour-blends-premultiplied-and-spring-easing-is-refused.md):

| Field | Value | Default | Where |
| --- | --- | --- | --- |
| `path_offset` | number in [0, 1] | `0` | `text` carrying `path` |

It is a **fraction of the curve's length**, as the painter's path measure finds it, for
ADR-0160's reason: it is the only unit `validate` can bound from the file alone, and its
meaning survives a vertex edit. Pixels of arc length were refused there and here. Animating
`path_offset` slides the text along the curve, After Effects' main use of Path Options.

**`align` names which point of the line sits at `path_offset`**, and on a path it does not
depend on the text's direction:

- `start`: the line's **lowest-distance** end;
- `center`: its middle;
- `end`: its **highest-distance** end.

This narrows
[ADR-0133](0133-a-runs-dir-is-an-isolate-and-start-and-end-follow-the-lines-own-direction.md)
for a text on a path only. Under ADR-0133's reading, `start` on an Arabic or Hebrew line is its
visual right end, so with the default `path_offset: 0` every glyph would fall before the curve
starts and §6 would hide the whole title without a word. Read against the curve, the defaults
draw for every script. `format.md` states this prominently, since an agent used to flat text
will expect the other meaning.

The line's extent is its advance extent in flat space, letter spacing included, so for a body
whose flat midpoint is `x`, `d = path_offset × L + (x − x_anchor)`, where `L` is the curve's
length and `x_anchor` the flat `x` of the point `align` names.

### 6. Past the ends

- **Open path.** A body whose `d` falls before `0` or past `L` is not drawn, SVG's rule. Text
  sliding off an end disappears body by body. Continuing along the end tangent was refused: it
  draws text that no longer follows the curve.
- **Closed path.** `d` wraps around the start point continuously, as `trim_offset` does. A line
  longer than one loop draws its bodies up to one full loop from its lowest-distance end, and
  the rest are not drawn, so no two glyphs are laid on top of each other.
- A stagger's `x` moves a body's `d` too, so a staggered letter can slide off an end.
- Whether a line fits cannot be checked from the file: the line's length needs the font, and
  the curve's needs the path measure. There is no `validate` finding for it. Instead `query
  --at` and `measure` name what is hidden (§8). That report is part of this decision, not an
  implementation detail, since the agent cannot see the frame.

### 7. The box frames the curve

The text's box is the frame its `points` are written in, as on a `path` element. Every vertex
and absolute handle, on every `points` keyframe, must lie inside the box inset by

> `m = the largest size among the runs + the largest stroke_width among the runs and the element`

or `E-PATH-OUTSIDE-BOX` fires, naming `m` and how it was derived.

`m` is **a frame convention, not a containment guarantee**. It keeps a plain glyph sitting on
the curve inside the box, and `validate` computes it from the file alone. It does not bound a
wide rigid body, whose half-width runs straight along the tangent and can be several ems past
the curve, or a unit a stagger lifts off the curve. `measure` reports the bent line's ink
extent, and that is where overflow shows.

The derived-height checks, `R-BOX-SLACK` and height overflow
([ADR-0014](0014-stroke-is-paint-the-text-box-is-required.md),
[ADR-0058](0058-text-box-slack-is-a-note-with-sibling-census.md)), do not apply to a text on
a path: line count × `size` × `line_height` measures nothing on a curve.

### 8. Animation, `validate`, `query`, `measure`, `shift`

**Animation.** The `path` element's rules hold unchanged: `path.points` keys as one whole list
of unchanging shape (`E-PATH-KEYFRAME-SHAPE`), `path.closed` is static, and `path_offset` keys
on its own. The painter measures the curve on every frame. Motion blur samples both with every
other keyed value
([ADR-0155](0155-motion-blur-is-a-per-element-field-that-accumulates-the-element-over-a-centred-shutter.md)).

**`validate`.** The four `path` errors fire on a text's `path` too, one code per mistake on
whichever element it appears; each message names the element type:

- `E-PATH-TOO-FEW-POINTS`, `E-PATH-DANGLING-HANDLE`, `E-PATH-KEYFRAME-SHAPE`;
- `E-PATH-OUTSIDE-BOX`, with §7's `m`.

Two new errors:

- **`E-TEXT-PATH-BREAK`**: a `\n` in any run of a text carrying `path`. The message names the
  run.
- **`E-TEXT-PATH-OFFSET-ORPHAN`**: `path_offset` on a text with no `path`.

A `path_offset` outside [0, 1], keyframe values included, is the schema's range error and needs
no code of its own. A bezier ease that overshoots clamps to [0, 1] in the one resolving
function, as ADR-0146 §5 requires. No new `review`: hidden units need the font.

**`query --at`**, only on a text carrying `path`, so other output is unchanged byte for byte:

- the resolved `path_offset`, raw, and the curve's length, informative as ADR-0158's is;
- `hidden:` the letters not drawn at that instant, by their ADR-0151 letter index, or `none`. A
  hidden joined piece or ligature lists all its letters.

**`measure`** reports the bent line's ink extent and the same `hidden:` line, at the element's
first frame.

**`shift`** needs nothing new. ADR-0146 refuses a split of keyed `points` it cannot write as
literals, ADR-0151 refuses a cut inside a stagger window, and `path_offset` splits as any
number does.

### 9. Gate, and the build

No part of this ADR is built before a prototype is accepted. It renders, byte-identical across
1 to 10 painters with the bound hint on and off:

1. a CapCut-style arc, centred with `align: center, path_offset: 0.5`;
2. a wave, an open path, whose `path_offset` slides the title on and off both ends;
3. a closed circle badge that wraps;
4. an Arabic line on the arc: its joined pieces stay rigid, and the default `align: start,
   path_offset: 0` draws;
5. a `by: letter` stagger dropping in perpendicular to the curve;
6. keyed `points`: a waving banner;
7. text `stroke` on a curve;
8. a tight curve, its radius below the line height, with an "fi" ligature and an Arabic joined
   piece, to measure lift-off at its worst;
9. text longer than the curve, on an open and on a closed path;
10. keyed `points` that change the curve's length, so the hidden set changes from frame to
    frame;
11. a mixed-direction line, Arabic with Latin digits;
12. runs of different sizes, so `m` is exercised as the largest size;
13. `align: end` with `path_offset` near 1 on an open path, and a stagger under an animated
    `path_offset`.

It reports the per-frame cost, how far rigid bodies lift off, and whether ink stayed within `m`
in each case. It also shows that `query --at` names the same hidden letters the frame shows, on
every painter count. It shows `shift` refusing a cut inside a keyed `points` window and writing
literals outside one. And it shows each new and reused `validate` code firing on a minimal bad
file.

The build is **one slice**, after the accepted prototype. The `path` element's slice already
built the point parser, the keying and the path measure, so what remains is placement and its
reports. Splitting it into static and animated would ship a `path` that `shift`, the map and
`format.md` each describe twice. If review finds it too big, split it by layer, the painter
with `query` and `measure` against the schema, checks, `shift` and docs, never by static
against animated.

**The capability map** gains one "does" line:

> One line of text bent along its own curve (straight, arc, wave or circle) and sliding along
> it: `path` (the `path` element's points) and `path_offset` on `text`, with `align`.

The rules on direction, `align`, the ends and the inset go to `format.md`, not the map.

## Invariants (ADR-0145)

| Invariant | How it is kept |
| --- | --- |
| Literal values | The curve is written points; the offset is a written fraction. No length is written or needed; the clamp and the wrap happen at paint time. |
| Closed vocabulary | One `path` field in ADR-0154's existing vocabulary and one flat `path_offset`. `align` is reused. No `curve`, no reverse flag, no reference, no multi-line mode, no warp. |
| Checkable by `validate` | Point shape, keying, the inset, a line break and an orphan offset are decided from the file alone. Hidden units are reported by `query` and `measure`, which hold the font. |
| Exact-string replace | The curve lives on the element it bends, so an edit to it changes that element and no other. A slide is one keyed `path_offset`. |

## Considered options

- **Only CapCut's arc, as one `curve` number.** Refused (§1).
- **Refusing text on a path.** Refused: the workaround, one element or one stagger slot per
  letter with hand-computed positions and angles, is exactly what agents do worst, goes stale
  on every text or size edit, and nothing checks it.
- **A reference to a `path` element by id.** Refused (§2).
- **Several lines on offset curves.** Refused (§3).
- **Warping glyph outlines along the curve.** Refused (§3).
- **Placing on the curve first, then staggering in screen axes.** Refused (§4).
- **Refusing a stagger on a text on a path.** Refused: the staggered reveal on an arc is the
  headline use.
- **Pixels of arc length for the offset.** Refused (§5).
- **No offset field, with sliding done by a stagger's `x`.** Refused: a stagger is per unit,
  and nothing could centre the line on an arbitrary point.
- **`align` read against the text's direction, with a warning for the RTL trap.** Refused: it
  warns about a trap the direction-free reading removes (§5).
- **Continuing past an open end along the end tangent.** Refused (§6).
- **A `validate` error for a line longer than its curve.** Refused: not checkable from the
  file (§6).
- **A `path_reverse` flag, or mapping an RTL line's reading direction to distance.** Refused
  for now (§4); the second scatters a mixed-direction line.
- **Points inside the box with no inset, or the box ignored.** Refused (§7).
- **Static `points` on text.** Refused: a curve that moves could not carry text, and the
  waving banner is a common request.
- **Twin `E-TEXT-PATH-*` codes for the four `path` errors.** Refused: two codes for one rule,
  each fix landing in two places.
- **Two slices, static then animated.** Refused (§9).
