---
status: accepted
amends: 0167 (§1: "with both angles at 0 the element is unchanged" becomes "an element with no `swivel` or `tilt` field is unchanged"; §2: the two angles compose in CSS's order; §8: a review joins for strong foreshortening)
---

# A written angle always projects, swivel applies before tilt, and strong foreshortening is a review

[The prototype](https://github.com/MBehtemam/Montagent/issues/786) for
[ADR-0167](0167-an-element-may-be-projected-never-placed-swivel-tilt-and-perspective-join-the-transform.md)
was accepted by the owner. It is byte-identical across 1–10 painters, and its report is
`prototype/projection-786/REPORT.md` on branch `prototype/projection-786`. It left three points
that ADR-0167 does not settle:

- it composed the two angles in one order without a rule;
- when both angles are exactly 0 it took the plain, unprojected path, so a keyed angle that passes
  through 0 switches path for one frame, shifting up to 5 levels in about 3.6% of bytes;
- just above the eye bound the near edge is magnified about 5× and looks soft.

Settled by one `/court` round of three jurors (Fable, Opus, Sonnet), judged by how agents would work
with each choice, with the owner ruling with the Judge's read. The round was unanimous on §1 and
§2, and split two to one on §3. The round record is on
[the hand-off spec](https://github.com/MBehtemam/Montagent/issues/787).

## The decision

### 1. A written angle always projects

An element that carries a `swivel` or `tilt` field, static or keyed, at any value, is always drawn
through the projected path (ADR-0167 §3), including at instants where both angles are 0. An element
with neither field is untouched, so every existing project keeps its bytes.

The path therefore depends on which fields are written, never on a value at an instant. A card
flip that passes through 0 does not pop for one frame. This matches the presence test ADR-0167 §8
already uses for `perspective`.

ADR-0167 §1's "with both angles at 0 the element is unchanged whatever `perspective` holds" now
reads: **an element with no `swivel` or `tilt` field is unchanged; one that carries either is
always projected, and at 0° it differs from the unprojected element only by the resample.** The
prototype measured that resample at up to 5 levels, and about 4.4 ms per element per frame at
1080p.

### 2. Swivel applies before tilt

The projection is CSS's `perspective(d) rotateX(tilt) rotateY(swivel)`: the swivel turns the
element first, then the tilt turns it in its already-swivelled frame. This is the order of the
accepted clips. Facing (cos swivel · cos tilt) and the eye bound do not depend on the order; the
projected corners do when both angles are non-zero. `format.md` states the order once, as that
CSS string.

### 3. Strong foreshortening is a review

The eye bound (ADR-0167 §5) stays the only error. A new review, **`R-PROJECTION-SOFT`**, fires when
the near edge's magnification, about d / (d − r), exceeds 2 at any key or eased extreme. Here d is
`perspective` and r is ADR-0167 §5's farthest-corner distance, so the trigger is the same as
d < 2r, stated as the magnification. The message names the magnification and the `perspective`
that brings it back to 2×.

It is a review, not an error, because strong foreshortening may be intended. An agent can leave it
standing on a shot meant to be dramatic, as it can `R-MOTION-BLUR-STILL` and `R-PROJECTION-AWAY`.

`R-PROJECTION-AWAY` (ADR-0167 §8) is decided at the same instants plus a quarter, a half and three quarters of every `swivel` and `tilt` segment and at least one sample per 30 degrees of travel between its keys, so a turn of more than half a revolution between two away-facing keys is not reported as never facing.

## The four tests

| Invariant | How this keeps it |
| --- | --- |
| Literal values | No value changes. |
| Closed vocabulary | No field is added; one review code joins. |
| Checkable by `validate` | §1's path is read from the file's keys. §3 is computed from literals at keys and eased extremes. |
| Exact-string replace | Editing a keyframe from 0 to 0.01 no longer changes the drawing path. `R-PROJECTION-SOFT` names the value that clears it. |

## Considered and refused

- **Keep the prototype's switch at 0 and document it.** Refused unanimously. The one-frame shift
  depends on a coincidence of values, and an agent inspecting frames cannot explain it from the
  JSON.
- **Tilt first, then swivel.** Refused unanimously. It matches no external reference, and it would
  undo the accepted clips.
- **No floor, documented only** (Fable). Refused two to one. Softness shows only after rendering,
  and agents learn about problems mainly through `validate`. Fable's objection that 2r has no
  basis is met by stating the trigger as the magnification.
- **An error at 2r.** Refused: it forbids a look the format can draw, and a hard limit is hard to
  loosen later.
- **Better magnification sampling**, such as drawing the flat layer at the expected peak
  magnification. Not refused but deferred: it changes no project file and no rule, but it changes
  bytes and cost, so it waits for measured evidence and is recorded in the map's fog. If it lands,
  `R-PROJECTION-SOFT` may be narrowed or dropped.

## Consequences

- ADR-0167 carries an amendment banner, and its §1, §2 and §8 are read through this ADR.
- [The hand-off spec](https://github.com/MBehtemam/Montagent/issues/787) builds the presence rule,
  the CSS order and `R-PROJECTION-SOFT`. Its acceptance test for an element with no angle field
  replaces "0° matches no projection".
