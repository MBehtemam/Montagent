---
status: accepted
amends: 0059 (discharges its deferral: `wipe`, `slide` and `push` join `crossfade`, with `direction` and an optional `ease`; `validate` now checks the two id references, which only `frame` caught before)
---

# Wipe, slide and push enter as transition kinds, and a matte from another element is refused

> **Amended by [ADR-0176](./0176-a-transition-carries-the-audio-across-its-cut-in-one-field-and-an-audio-only-crossfade-is-a-transition-kind.md)**: adds a fifth kind, `audio_crossfade`, and makes the `from`/`to` reference rule
> depend on it: it accepts an `audio` or `video` element, and the visual kinds still refuse an audio-only one.
> `direction` and `ease` are unknown keys for the new kind. A visual `ease` does not bend the audio.

[#678](https://github.com/MBehtemam/Montagent/issues/678), on the map
[#663](https://github.com/MBehtemam/Montagent/issues/663).
[ADR-0059](0059-transitions-element-type-crossfade-only-exact-window.md) made a transition
its own element type and deferred wipe, slide and push "until a forcing case shapes their
parameters". [ADR-0146](0146-an-animatable-property-is-one-the-schema-types-so-colour-blends-premultiplied-and-spring-easing-is-refused.md)
made the `mask` rect animatable, so a reveal on one element is a keyed mask. What was left
is what a keyed mask cannot say: a transition between two elements that is not a crossfade,
and one element's shape or alpha used as another's matte.

**Precedent.** Row 7 of the precedent research
([#664](https://github.com/MBehtemam/Montagent/issues/664)):
- Premiere has a closed, named list of transitions that includes Push, Slide and Wipe. Their
  settings are a direction edge, Start/End %, border width and colour, and Reverse.
- Premiere also has a Track Matte Key, where a hidden layer's alpha or luma is the matte,
  and "Mask with Text/Shape".
- CapCut has named slide and wipe transitions and per-clip shape masks. It has no matte
  taken from another track.

Everything here is inside the reference class, so
[ADR-0145](0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md)
asks for no accepted prototype. Premiere's settings list is the forcing case ADR-0059 waited
for: it fixes the parameter shape.

Settled in two grilling rounds, each put to a three-juror `/court` and decided by the
owner with the Judge's read. The jury was unanimous on everything recorded here. The one
split was where the mask question goes on the map: two jurors said fog and one said a
ticket, and the owner took the Judge's side, a ticket.

## The decision

### 1. The fields

```json
{"id": "t1", "type": "transition", "start": 4000, "end": 4600,
 "kind": "push", "from": "a", "to": "b", "direction": "left", "ease": "ease-in-out"}
```

- **`kind`** is one of `crossfade`, `wipe`, `slide`, `push`.
- **`direction`** is one of `left`, `right`, `up`, `down`. It is required on `wipe`, `slide`
  and `push`, and refused on `crossfade` by the schema. It names **the way the motion
  travels**. `"left"` means the content moves leftward, so the incoming element enters from
  the right. For a wipe, the edge travels left. The field is not called `from`, which is
  already the outgoing element's id.
- **`ease`** is optional on `wipe`, `slide` and `push`, and refused on `crossfade` by the
  schema. It takes the keyframe `Ease` vocabulary unchanged: a name (`linear`, `ease`,
  `ease-in`, `ease-out`, `ease-in-out`, `step`) or a cubic-bezier array. Omitted means
  `linear`. Under `step` the transition is a cut at the end of its window, as on a keyframe.
- Nothing else enters: no border, no Start/End %, no Reverse, no soft edge. Each waits for a
  case that asks for it.
- The window rule, the two-track rule and `crossfade`'s linear opacity ramp are unchanged.

### 2. What each kind resolves to

Every kind is resolved before painting, as `crossfade` already is. The rasterizer never sees
a transition.

**The terms:**
- `p` is progress: the eased fraction `(t − start) / (end − start)`.
- `W` and `H` are the frame's width and height.
- `D` is the travel distance: `W` for `left` and `right`, `H` for `up` and `down`.
- `n = floor(p × D)` is a whole number of pixels, computed once per frame and shared by
  every use below.

| `kind` | `from` | `to` |
| --- | --- | --- |
| `slide` | stays still | offset by `D − n` against the direction of travel, so it starts one frame off-screen and arrives at `p = 1` |
| `push` | offset by `n` in the direction of travel | offset as in `slide`, so the two stay joined |
| `wipe` | clipped to the side the edge has not reached | clipped to the side the edge has passed |

- **Offsets act in frame space by the full frame dimension.** They are applied outside the
  element's own transform, the way `clip` is. A lower-third slides a full frame width like
  a full-screen video, and the element's keyed `position`, `scale` and `rotation` keep
  working underneath, untouched.
- **A wipe's edge is a frame-space line** at `D − n` from the edge it travels towards. Both
  elements are clipped to complementary sides of the same line, so every pixel comes from
  exactly one of the two whatever their stacking order. The wipe clip is intersected with
  an element's own `clip`.
- **Edges and offsets are whole pixels and hard.** Two anti-aliased edges that abut don't
  add up to full coverage, so a seam of backdrop would show along the moving edge. Snapping
  removes the seam by construction, and integer geometry is the same on every painter. At
  ordinary lengths the edge moves tens of pixels a frame, so the steps cannot be seen. A
  very slow wipe (many seconds across the frame) does step visibly. That is accepted.
- `opacity`, `blend`, effects and masks apply to each element as without the transition.
  The offset moves the finished element, and the wipe clip cuts it.

**Stacking.** `wipe` and `push` work in either stacking order. In a `slide` only the
incoming element moves, over a still outgoing one, so `to` must paint above `from`. When it
doesn't, `validate` refuses (§3). The transition never changes a layer: layer stays declared,
never implied.

### 3. What the tools say

Each code is new in `validate`.
- **`E-TRANSITION-REF-MISSING`**: `from` or `to` names no visual element. Until now only
  `frame` and `render` caught this, as `E-NOT-PAINTED-UNRESOLVED-REF`. It mirrors
  `E-ANCHOR-MISSING`.
- **`E-TRANSITION-REF-SELF`**: `from` and `to` name the same element. It mirrors
  `E-ANCHOR-SELF`.
- **`E-TRANSITION-SLIDE-UNDER`**: in a `slide`, `to` paints beneath `from` in the resolved
  stack. The repair names both elements' layers and says `to` must paint above `from`.
- A missing or refused `direction` or `ease`, and an unknown `kind` or `direction`, are
  schema errors.

`query --at` and `frame` read the same resolved stack. During a wipe, slide or push, the
bridged elements' boxes are the offset and clipped ones, so `NOT COVERED` stays true to the
pixels.

### 4. A matte from another element is refused

One element's alpha or luminance deciding where another shows does not enter. A
`{"matte": "<id>"}` field would pass the four invariants as spellings: an id is a literal,
the mode an enum, a dangling reference checkable. It is refused for what the reference does:

- **It is the first edit `validate` cannot follow.** Editing element B's source, transform,
  keyframes or timing would silently change which pixels of element A show. The dependency
  lives in rendered pixels, not in the JSON. The anchor reference changes order, and a
  transition's `from`/`to` change geometry and opacity over a window `validate` can bound.
  Neither reaches into another element's samples.
- **The matte element would sit in a track without painting.** The format has never had
  that kind of element, and every tool (`query`, `frame`, `preview`, `NOT COVERED`) would
  have to special-case it.
- **The reference class is split.** CapCut has no matte taken from another track.

The reveals a matte serves (text revealing footage, a shape sweeping in as a window) go
through the smaller door: an element-local `mask` that grows, so every dependency stays
inside one element. Whether `mask` gains invert, a feather, or a source taken from text or a
shape is
[its own ticket](https://github.com/MBehtemam/Montagent/issues/688). A matte from another
element may return only with an answer to both reasons above.

## The four tests

| Invariant | How it passes |
| --- | --- |
| Literal values | `kind`, `direction` and a named `ease` are strings from closed lists, and a bezier `ease` is four numbers. The geometry in §2 is fixed arithmetic on the frame size and `p`. |
| Closed vocabulary | Four kinds, four directions, and the existing `Ease` vocabulary. A fifth kind or a new parameter needs its own case and decision. |
| Checkable by `validate` | Bad values are schema errors. The references and the slide's stacking are checked from the file, without painting a frame. |
| Exact-string replace | `"kind"`, `"direction"` and `"ease"` are one string each, changed in place. |

## Why

**All three, not `wipe` alone.** A keyed `position` can fake a slide or a push only with
keyframes on two elements on two tracks, whose windows must agree to the frame. `validate`
cannot tell that those keyframes are one transition. Recovering an authorial unit from
scattered state is exactly what ADR-0059 refused for crossfade.

**Frame space, because it is predictable.** "Starts one frame off-screen" holds without
knowing the element's bounds, which an agent often cannot compute from the JSON (text,
auto-sized images). Travel by the element's own size would leave a small element starting
inside the frame, turn a rotated one along a rotated axis, and change distance mid-transition
under a keyed `scale`.

**Travel direction, because one field serves three kinds.** "The edge it enters from" fits a
slide, strains for a push (both move), and means nothing for a wipe (an edge travels and
does not enter). It is also the word agents bring from "slide left" in CSS and CapCut.
Premiere's "From West to East" is a two-ended name, not a precedent for one field.

**An `ease`, because linear motion looks mechanical** and agents already write that
vocabulary on keyframes. Refusing it on `crossfade` keeps that kind's two halves summing to
a constant. The geometric kinds keep full coverage under any curve, because their two parts
are complementary.

## Considered and refused

- **`wipe` only, or none yet.** Refused for the reason above.
- **Offsets by the element's own size, through its transform.** Refused for the reason above.
- **Clipping only `to` in a wipe.** Refused: it works only when `to` paints above `from`, and
  needs a second stacking check.
- **Fractional, anti-aliased edges.** Refused: the seam along the moving edge.
- **Linear only, or a fixed baked-in curve.** Refused. Linear-only would be reopened the first
  time anyone watches a push. A baked-in curve is one the agent cannot name or change.
- **Entry edge as `direction`'s meaning.** Refused for the reason above.
- **The transition raising `to` for its window.** Refused: it changes stacking where the file
  does not show it.
- **A matte from another element.** Refused, §4.
- **A prototype gating the build.** Refused: ADR-0145 exempts a capability with precedent,
  and nothing a prototype would show changes the JSON. Byte identity is a test in the build.

## Consequences

- **`CONTEXT.md`.** The **Transition** entry gains the three kinds and **Direction**. "wipe
  (as a v1 value)" leaves its _Avoid_ list.
- **Feature map and `format.md`.** They list the kinds, `direction` with its one-line
  meaning, `ease`, the slide's stacking rule, and the frame-space offset.
- **The hand-off spec is one slice**, with three permanent gating tests:
  - every kind and direction, under sub-pixel element motion, compared byte for byte across
    painter counts and chunk sizes against a one-painter baseline;
  - a complementary wipe over a solid backdrop, showing no backdrop along the edge;
  - a push showing no gap at the join.
  A kind that fails is withdrawn, not excused.
