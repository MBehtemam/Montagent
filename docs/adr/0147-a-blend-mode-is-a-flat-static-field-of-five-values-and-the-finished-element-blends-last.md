---
status: accepted
amends: 0011 (each member of the resolved stack gains `blend`; `NOT COVERED` is unchanged and a blended element still counts as covering), 0040 (a blend mode is refused as an `effects` member; the order is written down: effects, then `opacity`, then the blend)
---

# A blend mode is a flat, static field of five values, and the finished element blends last

[#670](https://github.com/MBehtemam/Montagent/issues/670), on the map
[#663](https://github.com/MBehtemam/Montagent/issues/663). Elements composite with normal
alpha only. The spy trailer needed screen, add, multiply and overlay for light leaks, glows
and a HUD over footage, and faked them with stacked semi-transparent copies
([#521](https://github.com/MBehtemam/Montagent/issues/521)).

**Precedent.** Row 2 of the precedent research
([#664](https://github.com/MBehtemam/Montagent/issues/664)): Premiere gives one enumerated
mode per clip beside opacity, and lists 27. CapCut's list could not be sourced. Blend modes
are inside the reference class, so
[ADR-0145](0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md)
asks for no accepted prototype. The four invariants are shown below.

## The decision

### 1. The field

A visual element (`rect`, `ellipse`, `text`, `image`, `video`) may carry a flat field
`blend` beside `opacity`. Its value is one of five strings:

| Value | Skia mode | Per channel, opaque source `s` over opaque backdrop `d`, both in 0..1 |
| --- | --- | --- |
| `normal` | `SrcOver` | `s` |
| `multiply` | `Multiply` | `s·d` |
| `screen` | `Screen` | `s + d − s·d` |
| `overlay` | `Overlay` | `2·s·d` where `d ≤ 0.5`, else `1 − 2·(1−s)·(1−d)` |
| `add` | `Plus` | `min(1, s + d)` |

- Omitted means `normal`. Writing `"blend": "normal"` is legal and draws no finding, so a
  mode is turned off by the same exact-string replace that turned it on.
- The mode is spelled `add`. `plus` and `linear_dodge` are not values.
- `audio` and `transition` refuse the field in the schema.
- The field is static. It is not an animatable property: ADR-0146 admits numbers,
  fixed-size number arrays and colours, and a mode is none of them. A blend is faded in
  through `opacity`.
- There is one `blend` per element. A text element blends as one finished picture, fill and
  stroke together. There is no per-run blend.

### 2. The order

The blend is the last step of an element's paint:

1. the element is drawn through its ordered `effects` (masks and shadow included);
2. `opacity` scales the finished result;
3. that result is composited **once** into its backdrop, using the mode, inside the
   element's `clip`.

The **backdrop** is everything painted below the element in the stack, down to the
`background`. There is no isolation and no group backdrop.

What follows from the order:

- **A shadow blends in the element's mode.** Under `screen` or `add` a dark shadow all but
  disappears, and under `multiply` it darkens. An agent that wants an ordinary shadow under
  a blended element puts the shadow on a separate `normal` element. `format.md` says so.
- **A crossfade needs no rule.** It reaches the painter as opacity on the two bridged
  elements, and opacity is step 2.
- **The arithmetic runs on the stored sRGB values**, with no linearisation. This is what the
  reference editors do by default.
- **Over the default black `background`**, `screen` and `add` draw what `normal` draws, and
  `multiply` draws black.

The painter opens one layer for the element whenever `blend` is not `normal` or `opacity`
is below 1. The layer's paint carries the alpha and the mode, and the layer has no bounds
hint.

### 3. What the tools say

- **`query --at`:** each visual member of the resolved stack reports `blend`, with the same
  word the file uses, `normal` included.
- **`NOT COVERED`** stays a union of drawn rectangles. A blended element counts as covering,
  as a half-transparent one already does.
- **A new `review` finding, `R-BLEND-BACKGROUND-ONLY`:** a non-`normal` element with
  nothing beneath it. It is decided at the frame instants `render` paints, from boxes alone:
  - *Beneath* means a visual element lower in the stack whose box intersects this element's
    box, whatever that element's `opacity` or `blend`.
  - Boxes are compared as axis-aligned bounding boxes. A bounding box contains the real box,
    so "no bounding boxes intersect" proves "no boxes intersect".
  - **Fires** when no lower bounding box intersects the element's bounding box at some
    frame. The finding names the interval and the first such frame's instant, so
    `query --at` can be run on it.
  - **Clean** when, at every frame, at least one intersection is between two unrotated
    boxes.
  - **NOT CHECKED**, naming the element, when it does not fire and at some frame every
    intersection that keeps it quiet involves a rotated box.
- **`validate`** raises nothing else that is new. An unknown mode, or `blend` on `audio` or
  `transition`, is a schema error.

### 4. Byte-identical painting

Every admitted mode was measured, not inferred, on a throwaway spike (branch
`prototype/blend-modes`, commit `15921652`, built on the K-painters work of
[#653](https://github.com/MBehtemam/Montagent/issues/653); the branch is local only because
that work is not yet on `main`).

- One 90-frame project per mode held blended elements with sub-pixel motion, rotation,
  non-uniform scale, static and keyed `opacity`, `blur` with `shadow`, a `mask`, a `clip`,
  stroked text, images and a video.
- Raw frames at the encoder's input were compared byte for byte against a one-painter
  baseline, at seven settings of painters and chunk size from 1 to 10 painters, with one
  setting run twice. All four modes and the `normal` control were identical in every run,
  and the encoded files' hashes matched.
- With the blur and shadow bounds hint of
  [#652](https://github.com/MBehtemam/Montagent/issues/652) forced off, the frames were
  still identical. The hint fired on 270 element paints per project.
- One pixel per mode matched the table in §1 to the nearest byte, which also confirms the
  sRGB arithmetic.

Not measured: another machine or architecture, a written `"blend": "normal"` (inferred to
take the omitted path), and the formula under partial alpha. The hand-off spec carries the
painter comparison as a permanent gating test. A mode that fails it is withdrawn, not
excused.

## The four tests

| Invariant | How `blend` passes |
| --- | --- |
| Literal values | One string from the schema's list. The arithmetic in §1 is fixed and documented. |
| Closed vocabulary | Five enum values. A sixth needs its own forcing case and decision. |
| Checkable by `validate` | A bad value is a schema error, and `R-BLEND-BACKGROUND-ONLY` is decided from boxes without painting a frame. |
| Exact-string replace | One `"blend": "<mode>"` string per element, changed or reset in place. |

## Why

**A blend mode is not an effect.** An effect is element-local, and a blend mode reads what
is under the element. This is the reason
[ADR-0059](0059-transitions-element-type-crossfade-only-exact-window.md) gave for keeping
transitions out of `effects`. In an ordered list a blend would also raise questions with no
good answer: what its position means, and whether two are allowed.

**Five values, because four have a forcing case.** The trailer used exactly these four.
An agent can hold a five-word set whole and predict each from a prompt. A refused
`soft_light` costs one `validate` round, and adding a mode later costs one enum value and
no migration.

**Last, because one rule is predictable.** "Finish the element, then composite it once" can
be followed without seeing the picture, and it leaves today's paint order unchanged.

**The finding is narrow on purpose.** An element with nothing beneath it is the plain
mistake: a light leak with no footage under it. A glow that overhangs its subject is
ordinary design, so a partial overhang is silent. A rotated box that might be the only
thing keeping the finding quiet is said out loud, because a silent pass is the failure
that hurts an agent trusting `validate`.

## Considered and refused

- **A member of `effects`.** Refused for the reason above.
- **A wider vocabulary** (`darken`, `lighten`, `soft_light`, `hard_light`, `color_dodge`,
  `color_burn`, `difference`, or Premiere's list). Refused for now: none has a forcing case,
  each adds a byte-identity case to hold, and near-synonyms make a blind choice harder.
- **Per-run blend, or a stroke that blends apart from its fill.** Refused: it would be the
  format's only addressing level below the element for compositing. Two text elements do
  it.
- **Isolation or a group backdrop.** Not decided here. There is no group concept, and the
  transform map ([#591](https://github.com/MBehtemam/Montagent/issues/591)) owns it.
- **Linear-light arithmetic.** Refused: it differs from the reference editors' default and
  from what an agent expects "screen" to do.
- **A blended element that does not count as covering.** Refused: `NOT COVERED` is a fact
  about boxes, and it would disagree with how a half-transparent element is counted.
- **The finding on any part of the box over the `background`.** Refused as noise.
- **Sending every rotated element to NOT CHECKED.** Refused: where no bounding boxes
  intersect, the answer is certain.
- **An animatable mode.** Refused: a mode has no in-between.

## Consequences

- `CONTEXT.md` gains **Blend mode**, and the image swap now "steps rather than crossfades".
  ADR-0146's phrase "colour blends" is the one older use that stays.
- The feature map and `format.md` list the field, the order and the shadow note. The motion
  skill's stacked-copies **Workaround** is replaced by the field, and
  [#521](https://github.com/MBehtemam/Montagent/issues/521) closes with the build.
- The hand-off spec is one slice.
