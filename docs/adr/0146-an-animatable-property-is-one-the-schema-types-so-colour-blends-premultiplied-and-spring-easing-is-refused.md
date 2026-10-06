---
status: accepted
amends: 0012 ("transform properties only" is replaced by a closed list the schema types, admitted by three criteria), 0040 ("v1 effects are static" is retired: every numeric and colour effect parameter is animatable), 0038 (its `ease` rule covers every animatable property's list, unchanged), 0025 (reaffirmed: `clip` and a fitted box stay static, and the reveal it promised arrives as a keyed `mask` rect)
---

# An animatable property is one the schema types so; colour blends premultiplied in sRGB; spring easing is refused

> **Amended by three later ADRs.** Read them before relying on anything below.
>
> - [ADR-0149](0149-a-gradient-is-a-paint-linear-or-radial-measured-against-the-declared-box.md) — the
>   stop-list binding is discharged; gradient parameters join the one derived list as nested
>   paths; the run-override review widens to a gradient
> - [ADR-0151](0151-letter-spacing-is-an-animatable-element-field-and-a-stagger-is-a-units-block-on-one-text-element.md) — `letter_spacing`
>   and the five unit lists join the one derived list; per-letter and per-run animation is
>   settled as a `units` block with run overrides
> - [ADR-0157](0157-a-speed-ramp-is-a-time-remap-curve-of-source-times-on-a-video-element.md) — `source_time`
>   on `video` joins the one derived list; `speed` stays static

[#669](https://github.com/MBehtemam/Montagent/issues/669), on the map
[#663](https://github.com/MBehtemam/Montagent/issues/663).
[ADR-0012](0012-flat-transform-keyframes-carried-by-their-element.md) restricted keyframes to
transform properties, and
[ADR-0040](0040-effect-model-attachment-and-v1-vocabulary.md) shipped effects static for
that reason. [ADR-0145](0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md)
left the rule open to decision. This ADR decides it. It fixes a mechanism, not an
exhaustive list: later capabilities (gradient stops, trim-path, letter spacing, shader
parameters) reuse what it rules.

**Precedent.** Row 1 of the precedent research
([#664](https://github.com/MBehtemam/Montagent/issues/664)): Premiere keyframes any effect
property, with a closed list of interpolation kinds plus bezier handles. Animating effect
parameters and paint is therefore inside the reference class. Spring as a keyframe easing is
in neither tool. A keyed shape size could not be sourced for either tool; it rests on the
accepted prototype
([#668](https://github.com/MBehtemam/Montagent/issues/668), `prototype/animatable-shape`
`e66dc809`, the owner's words: *"wow great, approved"*).

## The decision

### 1. The rule

A property is an **animatable property** only where the schema types it as one: a literal,
or a keyframe list of `{t, t_from, v, ease}` records. A keyframe list on any other field is
a schema error. A property is admitted to the list by three criteria:

1. Its value type has a published interpolation (section 2).
2. The painter can resolve it from the file and the instant alone, with nothing carried
   from one frame to the next.
3. No `validate` check uses it as a single fixed value.

The third criterion keeps `clip` and a fitted box out, so
[ADR-0025](0025-clip-stays-static.md) is reaffirmed. Enum fields have no interpolation and
stay static.

### 2. Value types

| Type | Interpolation |
| --- | --- |
| Number | As today. |
| Fixed-size number array | Each component separately, as `scale` already does. The length is fixed per property by the schema, so a wrong-length record is a schema error. A point is this type. |
| Colour | sRGB-encoded components with premultiplied alpha (the CSS rule for hex colours). Each component is clamped to its range, then rounded to the nearest byte. A six-digit colour reads as fully opaque. |

**A list of gradient stops** gets no schema type here. This ADR binds the gradients decision
to two things: every record in one keyframe list has the same number of stops, checked by
`validate`, and each stop interpolates as a number plus a colour. That decision must also
say what happens when overshoot pushes one stop's offset past its neighbour.

**Integer-typed properties** (`width`, `height`, `radius`, `stroke_width`, the `mask` rect)
keep integer keyframe values. The resolved value is continuous and is never rounded, as the
resolved stack already says of `x`.

### 3. The admitted set

Built in two slices. **The schema types a field as animatable only in the slice that makes
the painter and every tool handle it**, so `validate` never passes a keyframe list that
`render` or `shift` ignores.

| Slice | Properties |
| --- | --- |
| 1. Shapes and text paint | `rect` and `ellipse`: `width`, `height`, `fill`, `stroke`, `stroke_width`; `rect`: `radius`. `text`: element-level `color`, `stroke`, `stroke_width`. |
| 2. Effect parameters | Every numeric and colour parameter of every effect: `blur.radius`; `shadow.dx`, `dy`, `radius`, `color`, `opacity`; `mask.x`, `y`, `width`, `height`, `radius`; `tint.color`, `amount`; `saturation.amount`; `brightness.amount`; `contrast.amount`; `chroma.color`, `tolerance`, `softness`, `spill`. |

Not admitted:

- **`width` and `height` on `image`, `video` and `text`.** On `image` and `video` the box is
  the `fit` claim checked against `clip`, and `scale` (two components) already animates the
  drawn size. On `text` the box drives line wrapping, so a keyed box would re-wrap the text
  on every frame.
- **Run-level and highlight paint.** A run that overrides `color`, `stroke` or
  `stroke_width` keeps its static override. Precedence is as today: a run override beats
  the element, and a highlight beats the run. Paint over time on a run already has a
  mechanism, the highlight window
  ([ADR-0048](0048-per-word-highlighting-is-a-timed-window-on-the-run.md)). Per-run and
  per-letter animation belongs to the letter-spacing capability.
- **`mask.shape`** and every other enum.

`chroma.color` animates and stays six-digit only, so alpha never enters.

### 4. Keyframes inside an effect

An animated effect parameter is written in place, in the same form as any animatable
property, and `t` means what it means on the element:

```json
{"name": "blur", "radius": [{"t": 0, "v": 0}, {"t": 400, "v": 24, "ease": "ease-out"}]}
```

Findings and `query --at` name it by its zero-based position in the list, with the effect's
name in the text: `effects[1].radius (blur)`. Position is the only unambiguous name, because
two effects of the same name are legal. A path goes stale when an effect is inserted or
reordered, so a finding is read against the file it was produced from.

### 5. Sizes, ranges and overshoot

- **Zero:** a keyframe value of `0` is legal for a size. Collapsing to nothing is a real
  move.
- **Negative:** a negative keyframe value for `width`, `height`, `radius`, `stroke_width` or
  a `mask` size is a schema error.
- **Bounded parameters:** a static range (such as `tolerance` 0 to 1) applies to every
  keyframe value by schema.
- **Overshoot:** a bezier may carry a resolved value outside its range between keyframes.
  The resolved value clamps to the range. A resolved `radius` clamps to half the shorter
  side. A resolved `width` or `height` at or below zero draws nothing for that frame, as
  `opacity` 0 does. A `mask` whose resolved `width` or `height` is at or below zero hides
  the whole element for that frame, so a reveal starts from a keyframe value of `0`.
- **Fractional boxes:** an in-between box is drawn unrounded.
- **One resolving function.** The clamps live in the one function that resolves a value at
  an instant. `query --at`, `validate`, the contact sheet and the painter all read it, so
  they report the same value.

### 6. `validate`

- **Never painted.** `E-NOT-PAINTED-NO-EXTENT` narrows to an element with no positive size
  at any frame in its range. `validate` and `render` decide it through the same resolving
  function at the same frame instants, so they agree. A frame hidden by a zero-size `mask`
  counts as not painted, and the finding names the cause: the box or the mask. A `mask`
  that is non-zero but lies wholly outside the element is not counted.
- **Existing checks widen.** The ease, derivation and ascending-`t` checks walk every
  keyframe list, including those inside `effects`. The off-canvas check
  ([ADR-0044](0044-off-canvas-is-a-standing-review-check-not-a-frame-change-census.md))
  resolves a keyed box the way it already resolves keyed `x` and `y`, over the whole range.
- **One new review finding.** A keyed element-level text paint field that every run
  overrides is reported, because those keyframes change nothing.

### 7. The tools

One list of animatable properties is derived from the schema and read by every tool. No
tool keeps its own. A test fails when a schema-animatable property is not carried by every
tool below.

| Tool | What it does with every keyframe list |
| --- | --- |
| `shift` | Carries and splits it, including lists inside `effects`. |
| The checks | Walk it (section 6). |
| The contact sheet | Counts its keyframe change points. |
| `compare` and `timeline` | Read it as they read the transform lists today. |
| `query --at` | Prints the resolved value of every animated property the element declares. A resolved colour is printed as a hex literal that can be pasted back: six digits when opaque, `#00000000` at alpha 0. |

**Splits.** A `shift` split writes a new keyframe, so its value must be a legal literal:

- A number-typed value splits exactly.
- An integer-typed value rounds to the nearest integer, ties away from zero. This is the
  existing `x` and `y` rule, with the same loss of at most half a unit at the split instant
  only.
- A colour rounds to bytes, so a split colour segment is exact only to byte rounding.
- **`shift` refuses a split whose resolved value cannot be written as a legal literal for
  that field**, naming the property path and the instant. This covers a size carried below
  zero by overshoot, a bounded parameter carried past its range, and a colour carried out
  of range. Clamping the written value instead would bend both halves of the curve without
  saying so.

### 8. Spring easing: no

The closed easing set is unchanged: six names, or a raw cubic bezier.

- `shift` splits a segment at an arbitrary instant. A split bezier is two beziers, which is
  why the raw form exists (ADR-0012). A split spring is not a spring, so `shift` would need
  a second mechanism or would change the motion.
- One overshoot is already a bezier with `y` outside 0 to 1. Several bounces are several
  keyframes, each a literal an agent can read and edit.
- It has no precedent in the reference class and no accepted prototype, so it could not
  enter under ADR-0145 today.

**Reopening** needs an owner-accepted prototype showing a look that beziers plus extra
keyframes cannot reach, and an answer for the split.

## The four tests

| Invariant | How this passes |
| --- | --- |
| Literal values | Every keyframe value is a number, array or colour written in the file. Interpolation and the clamps are fixed, schema-documented arithmetic. |
| Closed vocabulary | The animatable properties are the fields the schema types so. No key or enum is open. |
| Checkable by `validate` | Range, sign, array length and ascending `t` are schema or `validate` findings from the file alone. "Never painted" resolves values without painting. |
| Exact-string replace | Every keyframe record is its own string. An in-between value is singled out by adding a keyframe. |

## Why

**A closed list, not "every number".** An agent learns what it can animate by reading the
schema. With a default-everything rule, `clip`, a fitted box and a text box would look
animatable and fail only later, each by its own exception.

**Premultiplied sRGB.** The two sRGB rules differ only when colour and alpha both change.
That is the fade an agent writes: a colour to `#00000000`. Non-premultiplied passes through
grey, and `validate` cannot see it. Premultiplied gives the fade that was meant, and it is
the rule agents know from CSS
([#666](https://github.com/MBehtemam/Montagent/issues/666)).

**Deciding effects and paint in one ADR.** ADR-0025 promised the reveal as an animated
effect parameter, and [ADR-0088](0088-chroma-is-a-matte-operation-and-color-stays-literal.md)
named `tolerance` as the parameter that must drift. One rule for all of them avoids a second
ADR that differs slightly.

**One list for every tool.** The prototype's worst finding was silent: `shift` moved an
element's `x` keyframes and left its `width` and `fill` keyframes behind, because a six-name
list was copied in three places.

## Considered and refused

- **Every numeric or colour field is animatable by default.** Refused, as above.
- **Straight sRGB with alpha not premultiplied** (the prototype's guess, and what lottie-web
  and Skottie do). Refused: a fade to transparent black goes grey.
- **Oklab.** Refused: values can leave the sRGB gamut, Skia has no single-colour helper, and
  no agent expects it from a hex colour.
- **A keyed box on `image`, `video` or `text`.** Refused: section 3.
- **Animating run-level paint.** Refused: a run would have two timed sources for one field,
  its keyframes and its highlight, and they would need an ordering rule.
- **An exception for `chroma.color`.** Refused: an exception to "every numeric and colour
  effect parameter" is learned only by failing.
- **A warning when overshoot carries a size below zero.** Refused: a bounce into zero is a
  normal move, and the warning would fire on intended motion.
- **Clamping a split value that falls out of range.** Refused: section 7.
- **Spring as a kind with literal parameters, or a set of named springs.** Refused:
  section 8.
- **A per-tool choice of which properties to read.** Refused: that is how the silent `shift`
  loss happened.

## Consequences

- `CONTEXT.md` gains **Animatable property**; **Keyframe** and **Resolved stack** are
  reworded to it. "Keyframable" is on the avoid list.
- The prototype's colour code is not carried over: it blended without premultiplying.
- The prototype tested byte-identity on one painter only. Each slice shows byte-identical
  output across parallel painters
  ([ADR-0144](0144-render-paints-on-k-painters-over-chunks-and-the-spy-trailer-renders-in-a-minute.md)).
  Slice 2 resolves the blur and shadow bound per instant
  ([#652](https://github.com/MBehtemam/Montagent/issues/652)).
- Mattes, transitions and the animated `clip` no longer wait on this question: effect
  parameters animate, and `clip` stays static.
- Each slice updates `format.md` and the feature map.

## Not settled here

- The gradient paint vocabulary and the stop-list type.
- Per-letter and per-run animation.
- Whether a `mask` lying wholly outside its element counts as "never painted".
