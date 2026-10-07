---
status: accepted
amends: 0154 (§3's "later morphing question" is settled: no second mechanism; `E-PATH-KEYFRAME-SHAPE` names the fix, and a new review `R-PATH-SEAM-CAP` fires on an open path whose coincident ends meet at a corner under a cap that is not `round`)
---

# A morph between unlike shapes is written as matching vertex lists, and no rule resamples them

[#714](https://github.com/MBehtemam/Montagent/issues/714), on the map
[#663](https://github.com/MBehtemam/Montagent/issues/663).
[ADR-0154](0154-a-point-list-enters-as-one-path-element-in-integer-pixels-from-the-declared-box.md)
§3 lets `points` animate as one whole list only when every keyframe value has the same vertex
count and each vertex the same handles, and left a morph between paths with different counts
or handles to "the later morphing question, not a second mechanism". This ADR is that
question, and its answer is **no second mechanism**.

**Precedent.** The precedent research
([#664](https://github.com/MBehtemam/Montagent/issues/664), row 6) found no shape-to-shape
morph in CapCut or Premiere. After Effects adds vertices to a mask or shape path
automatically, and GSAP MorphSVG and flubber resample outlines; all three are outside the
reference class, so
[ADR-0145](0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md)'s
entry test would apply to a resampling rule. Skia's `Path::interpolate` needs matching verbs
([#665](https://github.com/MBehtemam/Montagent/issues/665)), and the painter does not use it:
it resolves `points` number by number and builds the path after.

**The fact that decides it.** Any two paths can already be written as matching lists without
changing either drawing:

- a **coincident vertex**, the same `at` twice with no handles between them, adds a
  zero-length segment, which draws nothing;
- a written `[0, 0]` handle draws the same as a missing one (ADR-0154 §1).

So every end state a resampling rule could reach is already expressible in exact integers. A
rule would add convenience and smoother in-between shapes, not a new thing the file can say.

Settled by two `/court` rounds of three jurors each (Sonnet, Opus, Fable), unanimous on every
question, with the owner ruling with the Judge's read each time. The jurors judged each
question by how agents write, read, edit and validate the file. Their amendments put the seam
rule in the format docs rather than only in a recipe, made the capability line say the
matching is by hand, and fixed one wording across the message, the docs and the recipe. The
Judge corrected one premise between the rounds: at the seam of an open path the two ends meet
as two **caps**, and `stroke_join` does not apply there.

## The decision

### 1. No resampling rule; the lists are matched by hand

A morph between unlike shapes is one `points` keyframe list whose values match in count and
handles, written by the agent. `E-PATH-KEYFRAME-SHAPE` stays an error, unweakened. The
agent:

1. pads the shorter list with **coincident vertices** (the same `at` twice), and writes
   `[0, 0]` for a handle one side lacks;
2. matches the **start vertex**, so each vertex travels to its neighbour's counterpart and
   not across the shape;
3. matches the **winding direction**, so the outline does not turn inside out;
4. where a smoother in-between is wanted, **splits a segment** at a point on it instead of
   padding at a vertex. A straight segment splits exactly wherever an integer point lies on
   it; a curve splits to the nearest integers, which moves the drawing by under half a pixel.

These steps are craft, taught by a recipe (§5), not rules the format checks: only step 1 is
required for the file to be valid, and the others decide whether the in-between looks right.

**Reopen condition.** A resampling rule may be reconsidered as a fresh capability, under the
entry test, if evidence shows hand-padding is a real source of agent errors: morphs refused
by `E-PATH-KEYFRAME-SHAPE` that agents fail to repair, or twisted morphs that ship.

### 2. `closed` stays static

`closed` never changes within an element (ADR-0154 §1), so one keyframe list cannot cross it.

- **A stroke-only morph between a closed and an open shape** is written as one **open** path
  throughout. The closed shape is written with its last `at` on its first, so the outline
  looks closed, and the morph is an ordinary matched list. Its seam is two caps meeting (§4).
- **A filled shape that opens is two elements**, joined by a cut or a `crossfade`. An open
  path takes no `fill` (ADR-0154 §5), so the fill must end somewhere, and the file says where.

### 3. A shape that will morph is a `path` from its first frame

An element never changes its `type`, and `rect` and `ellipse` take no `points`: one element
has one geometry source.

- A rect is four corner vertices.
- A circle of radius `r` is four cubic segments with handles of `0.5523 × r`, rounded to
  integers. It approximates the circle to under half a pixel at these sizes and is **not
  pixel-identical to an `ellipse`**.
- Where an existing `ellipse` must stay exact up to the morph, keep the `ellipse` and cut to
  the `path` at the morph's start. The step at the cut is under half a pixel.

### 4. The seam, and `R-PATH-SEAM-CAP`

At the seam of an open path whose first and last `at` coincide, the two ends meet as two
**caps**; `stroke_join` applies only at interior vertices.

- A **`round`** cap hides the seam always.
- A **`butt`** cap (the default) is invisible where the seam is smooth, since the two ends meet
  flush, and leaves a **notch** where the seam is a corner.
- A **`square`** cap leaves a **spur** at a corner.

A new review, **`R-PATH-SEAM-CAP`**, fires for a `path` with `"closed": false`, a `stroke`, no
`stroke_dash`, and a `stroke_cap` other than `"round"`, in each literal value of `points`
(static, or a keyframe's) whose first and last `at` are equal and whose seam is a **corner**.

- **The leaving direction** at the first vertex is its `out` if non-zero; otherwise the
  first non-zero of the next vertex's `at + in − at₀` and `at − at₀`, moving on past any
  segment whose four control points coincide.
- **The arriving direction** at the last vertex is the negation of its `in` if non-zero;
  otherwise the first non-zero of `atₙ − (at + out)` and `atₙ − at` for the previous vertex,
  moving back past any zero-length segment in the same way.
- **The seam is smooth** when the two directions are the same: their cross product is 0 and
  their dot product is above 0. Every value is an integer, so the test is exact. Anything else
  is a corner.
- **If either direction does not exist** (every segment at that end is zero-length), the
  review says nothing.
- The finding names the element, the keyframe record and `t` where keyed, and the cap. Its
  message names the fix: write `"stroke_cap": "round"`, or make the seam smooth.

The review is silent under `stroke_dash`: the pattern's phase decides whether the seam is
inked, and ADR-0158 §7 left the dash seam documented, not checked. Trim is not exempted: a
window that reaches both ends draws the seam.

This seam is not ADR-0158's dash seam. That one turns on an outline length the file does not
hold; this one is decided from the file's integers alone.

### 5. `E-PATH-KEYFRAME-SHAPE` names the fix, and where each piece is written

**The error message** gains one sentence: "To morph between unlike shapes, give the shorter
list coincident vertices (the same `at` twice) and write `[0, 0]` for a handle one side
lacks." It names no skill: `validate` runs from the CLI, MCP and CI, often with no skills
installed, and a message states the format.

**The format page for paths** carries the format facts themselves: the matching condition,
that `[0, 0]` equals a missing handle, that a coincident vertex draws nothing, that `closed`
is static, the seam rule of §4 and `R-PATH-SEAM-CAP` with its exact trigger.

**The `montagent-motion` skill** gains a recipe, "Morph one shape into another", carrying the
words agents search for (*morph*, *shape to shape*, *tween*): the four steps of §1, the rect
and circle of §3 with the half-pixel caveat and the two `ellipse` routes, the open-path seam
under a `round` cap, and two elements for a fill that opens.

**The capability map** deletes the can't-do line "Morphs between paths with different
vertices", adds the capability "A shape morphing into an unlike one, with matching vertex
lists written by hand: `path`, `points`", and adds under **Not by design**: "Automatic vertex
matching between unlike paths. Pad the lists by hand (`montagent-motion`)." citing this ADR.

The same phrases, *matching vertex lists* and *coincident vertices (the same `at` twice)*,
are used word for word in the message, the format page, the recipe and the map, so an agent
connects them by search.

**Hand-off.** One slice carries the message, the review, the format page, the recipe and the
map, so none of them describes the decision before the others do. No prototype gate: nothing
new is painted.

## Invariants (ADR-0145)

| Invariant | How it is kept |
| --- | --- |
| Literal values | Every in-between is the number-by-number interpolation of vertex lists the file states. No geometry is computed that the file does not hold. |
| Closed vocabulary | No new field or element. One review code joins the registry. |
| Checkable by `validate` | `E-PATH-KEYFRAME-SHAPE` is unchanged in what it refuses. `R-PATH-SEAM-CAP` is exact integer arithmetic over the literals. |
| Exact-string replace | Padding is inserting vertex objects; the correspondence is the list order, which an edit can see and change. |

## Considered options

- **A render-time resampling rule** (After Effects, flubber): keyframe values may differ, and
  the painter resamples both outlines to one count. Refused: the in-between shapes are
  geometry the file never states, the correspondence is implicit and cannot be edited,
  `query --at` would print computed vertices, `E-PATH-KEYFRAME-SHAPE` would weaken into
  silently accepting a mismatch, and the entry test would need a prototype, all for no new
  expressible end state.
- **A write-time helper verb** that writes the matched lists into the file. Refused: it keeps
  the invariants but is an expander by another name, which
  [ADR-0148](0148-a-repeat-does-not-enter-and-a-stagger-enters-only-across-the-units-of-one-text-element.md)
  refused, and the format has no CRUD API. Agents would reach for it instead of learning the
  rule.
- **An animatable `closed`**, switching at a keyframe. Refused: the fill would pop in or out
  at an instant, a step pretending to be an animation, and it reopens ADR-0154's "a path that
  opens mid-clip is two elements".
- **Keyed `points` on `rect` and `ellipse`.** Refused: one element with two geometry sources
  needs a precedence rule (does `radius` or `points` win?) and makes every tool carry it.
- **A review whenever the cap is not `round`**, corner or not. Refused: it fires on correct
  files, a smooth seam under the default `butt` cap among them, and a review that must say
  "ignore me sometimes" teaches agents to ignore reviews.
- **No review; the recipe and the render.** Refused: a notch or spur of a few pixels is what
  agents miss in a frame, and the fact is decidable from the file.
- **Naming the skill in the error message.** Refused: it ties the binary's output to the
  skill layout, and the capability map already links the skill.
