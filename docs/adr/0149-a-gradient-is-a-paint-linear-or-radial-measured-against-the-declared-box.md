---
status: accepted
amends: 0014 ("gradients are out of v1, as unevidenced" is closed: a gradient enters as a named entry in a closed vocabulary, as that ADR said it would), 0146 (its stop-list binding is discharged; the one derived list of animatable properties gains nested paint paths; its run-override review widens to a gradient)
---

# A gradient is a paint, linear or radial, measured against the declared box

[#677](https://github.com/MBehtemam/Montagent/issues/677), on the map
[#663](https://github.com/MBehtemam/Montagent/issues/663).
[ADR-0014](0014-stroke-is-paint-the-text-box-is-required.md) left gradients out of v1 "as
unevidenced rather than rejected", and said that if one arrived "it will be a named entry in a
closed vocabulary, not an open syntax".
[ADR-0146](0146-an-animatable-property-is-one-the-schema-types-so-colour-blends-premultiplied-and-spring-easing-is-refused.md)
bound this decision: every record in one keyframe list has the same number of stops, each stop
interpolates as a number plus a colour, and the decision must say what happens when overshoot
pushes one stop's offset past its neighbour.

**Precedent.** Row 3 of the precedent research
([#664](https://github.com/MBehtemam/Montagent/issues/664)): in Premiere a gradient is a kind
of paint on a text or shape layer's Fill, Stroke or Shadow. The kinds are Solid, Linear and
Radial, with colour stops, separate opacity stops, an Angle for linear and a Location per stop.
Whether the stops keyframe is not documented. CapCut documents gradient text only. Linear and
radial are inside the reference class, so
[ADR-0145](0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md)
asks for no accepted prototype to admit them. The map's evidence bar and byte-identical
painting still ask for one before the build (section 8).

**Skia.** Row 3 of the Skia research
([#665](https://github.com/MBehtemam/Montagent/issues/665)): `linear`, `radial`,
`two_point_conical` and `sweep` gradient shaders are in the pinned build with no feature
change, with an `Interpolation` that can be premultiplied. Dithering is off unless asked for,
and the painter never asks.

## The decision

### 1. Paint, and where it may be written

A **paint** is a flat colour or a gradient. Four fields take a paint:

- `rect` and `ellipse`: `fill` and `stroke`;
- `text`: element-level `color` and `stroke`.

Every other colour field takes a colour only: run and highlight paint, the project
`background`, and the effect colours `shadow.color`, `tint.color` and `chroma.color`.

A paint field holds either a colour string or a gradient object. The kind is the enum
`gradient`:

```json
"fill": {"gradient": "linear", "angle": 90,
         "stops": [{"offset": 0, "color": "#FF3366"}, {"offset": 1, "color": "#3366FF00"}]}
```

```json
"fill": {"gradient": "radial", "center": [0.5, 0.5], "radius": 1,
         "stops": [{"offset": 0, "color": "#FFFFFF"}, {"offset": 1, "color": "#000000"}]}
```

- **The kinds are `linear` and `radial`.** Sweep and two-point conical have no precedent in
  the reference class. Each could enter later through its own prototype under ADR-0145, as
  one more enum value.
- **A stop** is `{"offset", "color"}`. `offset` runs from 0 to 1. `color` is a colour as the
  glossary defines it, and its alpha is the stop's opacity. There are no separate opacity
  stops. A list holds at least two stops and has no upper limit.
- **Every parameter is required.** `linear` requires `angle` and refuses `center` and
  `radius`. `radial` requires `center` and `radius` and refuses `angle`. A wrong direction,
  centre or size gives a frame that looks plausible, so by
  [ADR-0012](0012-flat-transform-keyframes-carried-by-their-element.md)'s test none of them
  may default.

### 2. Geometry

A gradient is measured against the element's **declared box**: the declared rect of a shape,
and the declared text box of a text element, never the glyph bounds. A shape's `stroke` uses
the same box as its `fill`. Because every quantity is relative to the box, a keyed `width` or
`height` carries the gradient with it, and moving or resizing an element never edits its
paint.

**Linear.** `angle` is in degrees, using CSS's convention: `0` runs from bottom to top, `90`
from left to right, and angles turn clockwise. Any number is legal, so `370` is `10`. For a
box of width `W` and height `H`, the gradient line passes through the box's centre in
direction `(sin a, −cos a)` (y down) and has length

```
L = |W·sin a| + |H·cos a|
```

so the box's corners land exactly on offsets 0 and 1, whatever the aspect ratio.

**Radial.** `center` is `[fx, fy]`, fractions of the box from its top-left corner. Any number
is legal, so a glow may be centred outside the box. `radius` is ≥ 0 and is a fraction of the
distance from the centre to the farthest corner, measured in box fractions: `1` just reaches
that corner. The gradient is a circle in box fractions, so on a box that is not square it
draws as an ellipse that stretches with the box. With `center: [0.5, 0.5]` and `radius: 1`
this is CSS's default radial gradient.

**Past the ends** the first and last colours extend (Skia's clamp tile mode). There is no
repeat or mirror. Those exist only in Premiere's newer Gradient *effect*, and either can come
later as one more key without changing any existing file.

**Colour across the gradient** blends in sRGB with premultiplied alpha. This is the same rule
ADR-0146 uses across time, and it is the CSS rule. A stop fading to `#00000000` keeps its hue
and does not pass through grey. Equal offsets are legal and give a hard edge.

### 3. Animation

A gradient animates **in place**, the way ADR-0146 §4 animates an effect parameter:

- `angle`, `center` and `radius` may each be a keyframe list of their own type: a number, a
  two-number array, a number.
- `stops` may be a keyframe list whose `v` is a whole stop list. Every `v` in one list has the
  same number of stops. Stop *i* interpolates against stop *i*: its `offset` as a number and
  its `color` as a colour.
- **The kind never animates.** A keyframe list on `gradient` is a schema error.
- **A paint field is a colour, a keyframe list of colours, or a gradient object.** A keyframe
  list whose `v` is a gradient, or one that mixes colours and gradients, is a schema error. An
  agent that wants to go from a flat colour to a gradient writes the flat colour as a
  gradient whose stops share that colour, then keys the stops.

```json
"fill": {"gradient": "linear", "angle": [{"t": 0, "v": 0}, {"t": 2000, "v": 360, "ease": "linear"}],
         "stops": [{"offset": 0, "color": "#FF3366"}, {"offset": 1, "color": "#3366FF"}]}
```

### 4. Overshoot and the resolving function

A bezier may carry a resolved value past its range. As ADR-0146 §5 requires, the fix lives in
the one function that resolves a value at an instant, which the painter, `query --at`,
`validate` and the contact sheet all read.

- **Stop offsets:** clamp each resolved offset to 0..1, then, in list order, raise each stop's
  offset to the largest offset before it (the CSS rule). A raised stop keeps its own colour. A
  crossing therefore becomes a hard edge, not a failure.
- **`radius`:** a resolved `radius` at or below 0 paints the last stop's colour over the whole
  box, as CSS does.
- **`angle` and `center`** have no range, so there is nothing to fix.
- **Colours** clamp and round as ADR-0146 §2 says.

### 5. `validate`

What the schema states:

- `gradient` is `linear` or `radial`, with the required and refused keys of §1.
- At least two stops; `offset` is 0..1; `color` is a colour; `radius` is ≥ 0.
- The paint-field shapes of §3, including a keyed kind and a keyed gradient as errors.

What the schema cannot state, so `validate` checks it as an error:

- **`E-GRADIENT-STOP-ORDER`:** in every literal stop list, static or one keyframe's `v`, the
  offsets never decrease. The finding names the property path and the first pair out of
  order.
- **`E-GRADIENT-STOP-COUNT`:** every `v` in one `stops` keyframe list has the same number of
  stops. The finding names the path and the first record whose count differs.

Two reviews:

- **ADR-0146's run-override review widens.** It already reports a keyed element-level text
  paint field that every run overrides. It now also reports a text element-level `color` or
  `stroke` that holds a gradient, keyed or static, when every run overrides it, because that
  gradient is never drawn.
- **`R-GRADIENT-ONE-COLOUR`:** a gradient whose **resolved** paint is a single colour at every
  frame instant `render` paints in the element's range. It is decided by the resolving
  function, so it covers stops that share one colour, a `stops` keyframe list whose every
  record does, and a `radius` that stays at or below 0. A flat-to-gradient animation never
  fires it, because its stops differ at some instant. What fires it is a gradient that is
  dead weight from start to finish: a forgotten second colour, or a radial that never grows.

Nothing else is new. No existing check judges what a paint looks like: coverage ignores paint,
and `E-NOT-PAINTED-NO-EXTENT` reads size alone, so a gradient changes neither.

### 6. The tools

The nested paths `fill.angle`, `fill.center`, `fill.radius` and `fill.stops`, and the same
under `stroke` and text `color`, join ADR-0146 §7's **one** schema-derived list of animatable
properties. Its guard test covers them. No tool keeps its own list.

| Tool | What it does |
| --- | --- |
| `query --at` | Prints a resolved paint as a literal that can be pasted back: the kind, the resolved `angle` or `center` and `radius`, and the stops after §4's fix, colours as hex, numbers in their shortest exact form. The fixed stops are always a legal literal. |
| Findings | Name a nested property by its path: `fill.angle`, `fill.stops`, `stroke.radius`. |
| `shift` | Carries and splits every nested list. A split writes the resolved value, so `shift` tests the offsets **before** §4's fix and refuses a split whose offsets are out of range or crossed, naming the path and the instant (ADR-0146 §7). Testing after the fix would always pass and would silently flatten the curve past the cut. |
| The contact sheet | Counts the change points of every nested list. |
| `compare` and `timeline` | Read them as they read every other keyframe list. |

### 7. The painter

The painter builds one Skia gradient shader per paint per frame from the resolved values:
`linear` or `radial`, clamp tile mode, premultiplied interpolation, with dithering left off.
A radial is a circle in box fractions, drawn through a local matrix that scales by `W` and
`H`.

### 8. The prototype gates the build

Before the first slice is built, a prototype is rendered and accepted by the owner. It must
show:

- a linear and a radial fill on a `rect`, and a gradient on a `text` element's `color`;
- one keyed stop list, so the look of an animated gradient is judged;
- a fade to `#00000000`, to confirm the premultiplied rule;
- frames byte-identical across painter counts
  ([ADR-0144](0144-render-paints-on-k-painters-over-chunks-and-the-spy-trailer-renders-in-a-minute.md)),
  including a gradient element under `blur` and `shadow` with sub-pixel motion. The Skia
  research names a gradient's coordinates inside a layer whose origin moves as **not
  measured**, so this case is measured, not inferred.

A gradient kind that fails the byte-identity run is withdrawn, not excused.

## The four tests

| Invariant | How a gradient passes |
| --- | --- |
| Literal values | Every parameter and stop is a number, array or colour written in the file. The geometry formulas, the clamp and the overshoot fix are fixed, documented arithmetic. |
| Closed vocabulary | Two kinds in an enum, with fixed keys per kind. A third kind needs its own decision. |
| Checkable by `validate` | Shape errors are schema errors. Stop order and stop count are `validate` errors from the file alone. `R-GRADIENT-ONE-COLOUR` resolves values without painting. |
| Exact-string replace | Each parameter, each stop and each keyframe record is its own string. An in-between value is singled out by adding a keyframe. |

## Why

**The same field, not a sibling field.** A sibling `fill_gradient` would need a rule that it
and `fill` are never both set, and swapping a flat fill for a gradient would take two edits
that must agree. With one field, an agent looks in one place and replaces one value.

**Not an effect.** A gradient is how the element is painted, not a pass over its pixels after
painting. In `effects` it could not tell `fill` from `stroke`, and its position in the list
would change what colour a shape is.

**Measured against the declared box.** The declared box can be read from the file, so
`validate`, `query` and the agent can all say where a colour falls. Glyph bounds depend on
shaping, and would move whenever a word changed.

**In-place animation.** It is ADR-0146's rule for effect parameters, so there is one pattern
to learn. An agent turns an angle without retyping the stops, and each property keeps its own
easing.

**Premultiplied sRGB.** Space and time follow one rule, and the commonest gradient an agent
writes, a colour fading to transparent, comes out as meant.

**Alpha in the stop's colour.** One list, so the stop-count rule stays a single check, and
the colour literal is the one the format already has.

## Considered and refused

- **Sweep or two-point conical now.** Refused: no precedent, so each needs its own prototype.
- **A gradient on run or highlight paint.** Refused: ADR-0146 keeps that paint static, and a
  run's gradient would need a box the file does not name (the run, the line or the element).
- **A gradient on `background`.** Refused: a full-frame `rect` with a gradient `fill` does it.
- **A gradient on `shadow.color`.** Deferred, not refused: Premiere has it, and it can be
  added later without changing any existing file.
- **A gradient on `tint` or `chroma`.** Refused: a tint gradient is a different effect, and a
  key colour cannot be a gradient.
- **A sibling `fill_gradient` field, or a gradient effect.** Refused, as above.
- **Two points (`from`, `to`) for a linear gradient.** Refused: four numbers that must move
  together, and harder to predict on a box that is not square.
- **A radius in pixels, or a true circle.** Refused: it changes the look when the box is
  resized or its size is keyed.
- **Defaults for `angle`, `center` or `radius`.** Refused by ADR-0012's test.
- **Repeat or mirror.** Refused for now; see §2.
- **Unpremultiplied or Oklab interpolation across the gradient.** Refused for the reasons
  ADR-0146 gives across time.
- **Separate opacity stops.** Refused: two lists with independent positions and two counts.
- **The whole gradient as a keyframe value.** Refused: every keyframe repeats the kind and
  every stop, and kind or count could differ between records.
- **A cap on the number of stops.** Refused: removing a cap later is free, adding one later
  breaks files.
- **`at` or `location` for a stop's position.** Refused: `at` reads as a time beside `t` and
  `query --at`; `location` is Premiere's undocumented word. `offset` is SVG's and Skia's.
- **`shift` testing offsets after the fix.** Refused: section 6.
- **A separate note on a static `radius: 0`.** Refused: `R-GRADIENT-ONE-COLOUR` reports it.

## Consequences

- `CONTEXT.md` gains **Paint** and **Gradient**. **Fill** becomes "the paint inside a shape's
  outline", and *paint* leaves its avoid list. The Paint entry says in its first line that a
  paint is a value and the painter is a process.
- The hand-off spec is two slices:
  1. **Static paint:** the schema union, the painter, `E-GRADIENT-STOP-ORDER`, `query --at`,
     the widened run-override review and `R-GRADIENT-ONE-COLOUR` over static paint,
     `format.md` and the feature map. `format.md` says plainly that gradient parameters
     cannot be keyed yet. It is gated by the prototype of §8.
  2. **Animated gradients:** the nested paths through every tool, `E-GRADIENT-STOP-COUNT`,
     the `shift` refusal, and the keyed cases of both reviews. It is blocked by ADR-0146's
     first slice, which makes paint fields animatable, and by slice 1 here.

## How it was decided

Three grilling rounds, each put to a three-juror court (Opus, Sonnet, Fable), and the owner
ruled "go with the judge" on all three. The jurors added four things to the proposals: dithering
named explicitly, `shift` testing offsets before the fix, a raised stop keeping its own colour,
and the one-colour review defined on the resolved paint.
