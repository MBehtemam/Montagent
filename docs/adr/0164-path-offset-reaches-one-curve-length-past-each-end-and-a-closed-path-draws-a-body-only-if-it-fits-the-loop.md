---
status: accepted
amends: 0161 (§5 and §8: `path_offset` ranges over [−1, 2], and an overshoot clamps there; §6: on a closed path a body is drawn only if its whole advance fits within one loop)
---

# `path_offset` reaches one curve length past each end, and a closed path draws a body only if it fits the loop

[The text-on-a-path prototype](https://github.com/MBehtemam/Montagent/issues/764), on the map
[#663](https://github.com/MBehtemam/Montagent/issues/663), built
[ADR-0161](0161-a-text-element-bends-its-one-line-along-its-own-inline-path.md) as written and
was accepted by the owner. Its
[report](https://github.com/MBehtemam/Montagent/issues/764#issuecomment-6034182705) found two
places where the decision as written cannot do what it says:

1. **One element cannot slide on from one end of an open curve and off the other.** ADR-0161 §9
   asked for "a wave, an open path, whose `path_offset` slides the title on and off both ends".
   With `path_offset` in [0, 1] and `align` static, no single `align` does both. `align: end` at
   offset 0 hides the line before the start, but at 1 its end sits on the curve's end and it
   cannot leave. `align: start` at 1 hides the line past the end, but at 0 it already shows.
   The prototype needed two elements and a cut.
2. **On a closed path the last drawn body can overlap the first.** The prototype decided "drawn"
   by a body's midpoint, so at the seam two bodies could overlap by up to half of each advance
   ("NOT" touches "ONE" at 18 s in the clip). §6 says "no two glyphs are laid on top of each
   other", and nothing an agent can read reported the collision: both bodies are drawn, so
   neither is in `hidden:`.

Settled by one `/court` of three jurors (Opus, Sonnet, Fable), unanimous on both questions,
judged by how agents write, read, edit and validate the file, with the owner ruling with the
Judge's read. All three jurors held that the prototype's rule moves, not ADR-0161's promises.

## The decision

### 1. `path_offset` ranges over [−1, 2]

`path_offset` is a number in **[−1, 2]**, default `0`. It is still a fraction of the curve's
length `L`, and `d = path_offset × L + (x − x_anchor)` is unchanged. A value below 0 or above 1
puts the point `align` names off the curve, and §6's rule for an open end says what happens to
the bodies there.

- **Why these bounds.** One curve length past each end is exactly enough to hide, entirely, any
  line no longer than the curve: under `start` at −1 the line ends at or before the start, and
  under `end` at 2 it begins at or after the end. (Under `center`, −1 and 2 hide a line up to
  2L.) A line longer than its curve can never be shown whole on an open path, so a wider bound
  would buy nothing visible, and a bound is still what lets `validate` catch a stray `20`.
- **The range check holds in every keyframe value**, as before. A value outside [−1, 2] is the
  schema's range error, and an ease that overshoots clamps to [−1, 2] in the one resolving
  function (ADR-0146 §5).
- **One element slides on and off.** With `align: start`, keying `path_offset` from `-1` to `1`
  brings a line no longer than the curve in from the start and takes it out past the end.
- **A line longer than its curve** cannot fully enter or fully leave an open path in one
  element. The format page says so and names the way out: a stagger's `x`, or a cut.
- On a closed path an offset outside [0, 1] wraps as any `d` does, so −0.25 and 0.75 draw the
  same.

### 2. On a closed path, a body is drawn only if its whole advance fits within one loop

ADR-0161 §6's closed-path rule is made exact. Measured from the line's lowest-distance end, a
body is drawn only if its **whole advance** lies within one loop, `[0, L)`. A body whose
midpoint fits but whose advance does not is hidden. So no two bodies are ever laid on top of
each other, as §6 says.

- **"Advance" is the body's shaped advance in flat space**, letter spacing included, after a
  stagger's `x` has moved it: the same extent §5 already uses to place `x_anchor`. It is not the
  ink extent, which would depend on the font's outlines rather than on shaping. A joined piece
  or a ligature is one body with one advance.
- **The open path keeps the midpoint rule.** A body overhanging an open end collides with
  nothing. A body overhanging the seam of a loop collides with the first body. The two rules
  differ by what they prevent.
- **`query --at` and `measure` list the body the seam rule hides**, by its letters, in the
  `hidden:` line §8 already defines. The painter and both tools call the one function that
  decides what is drawn.
- A loop slightly too long for its line can show a gap of up to one advance at the seam, and one
  letter fewer than the midpoint rule would draw. That letter is named in `hidden:`.

## The four tests

| Invariant | How these keep it |
| --- | --- |
| Literal values | `path_offset` is still one written fraction. The seam rule is fixed arithmetic on shaped advances. |
| Closed vocabulary | No new field or value. A range widens and a rule is made exact. |
| Checkable by `validate` | The range is still a schema range error in every keyframe value. What a loop hides still needs the font, so it is reported by `query --at` and `measure`, as ADR-0161 §6 decided. |
| Exact-string replace | A slide on and off is one keyed `path_offset` on one element. |

## Considered and refused

- **Keep [0, 1], and slide on and off with two elements or a stagger's `x`.** Refused: every
  later edit to the title is made twice and the cut kept in step by hand, and a stagger's `x` is
  per unit, in pixels the agent cannot compute without the font. ADR-0161 had already refused
  sliding by a stagger.
- **An unbounded `path_offset`.** Refused: it gives up the one check `validate` can make from the
  file, for no visible gain past one length beyond each end.
- **Keep the midpoint rule and reword §6 to "no body is drawn twice".** Refused: it accepts an
  overlap at the seam that no tool reports, which is the kind of defect an agent that cannot see
  the frame has no way to find.
- **The extent rule on open paths too.** Refused: an overhang at an open end collides with
  nothing, and the midpoint rule is SVG's.
- **The ink extent instead of the advance.** Refused: it depends on glyph outlines, and the
  advance is what §5 already places by.

## Consequences

- ADR-0161 carries an amendment banner. Its §5 table row, §8's range sentence and §6's
  closed-path rule are read through this ADR.
- No new prototype. Both changes are deterministic arithmetic on values the accepted prototype
  already computes, and the build's byte-identity run covers them.
- The hand-off spec [Text on a path](https://github.com/MBehtemam/Montagent/issues/765) gains
  the widened range, the seam rule, and three tests: one element sliding fully on and fully off,
  no overlap at a closed path's seam, and `hidden:` naming the body the seam rule hides.
