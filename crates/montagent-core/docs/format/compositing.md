# The Montagent project format: compositing

A page of `montagent://format.md`, which holds the rules every element shares and lists the
other pages; read it first. This page holds the rules for how elements combine: an element's
`effects`, how the finished element composites into what is below it, and the transition
elements that bridge two others. Like the rest of the format docs, every rule here is an
accepted decision in the ADR series, cited inline by number, and the ADR is right where the
two disagree.

## Effects and compositing

- **Effects are an ordered list, and the order is semantically real** (ADR-0040). Blur-then-shadow is a
  different frame from shadow-then-blur. They attach to whole elements, never to a run, and
  no effect parameter is keyframable in v1.
- **`blend` is how the finished element composites into what is below it** (ADR-0147). One
  of five words: `normal`, `multiply`, `screen`, `overlay`, `add`. It sits beside `opacity`
  on `rect`, `ellipse`, `text`, `image` and `video`; `audio` and `transition` refuse it.
  Omitted means `normal`, and a written `"blend": "normal"` is legal and draws the same
  bytes. It is static and never keyframed: fade a blend in through `opacity`. The order is
  fixed: the element is drawn through its `effects` (masks and shadow included), `opacity`
  scales the result, and that result is blended **once** into everything painted below it,
  down to the `background`, inside the element's `clip`. The arithmetic runs on the stored
  sRGB values.
- **A shadow blends in its element's mode** (ADR-0147). Under `screen` or `add` a dark
  shadow all but vanishes, and under `multiply` it darkens. For an ordinary shadow under a
  blended element, put the shadow on a separate `normal` element beneath it.
- **Over the default black `background`, `screen` and `add` draw what `normal` draws, and
  `multiply` draws black** (ADR-0147). A blend needs something beneath it. `validate`
  reports `R-BLEND-BACKGROUND-ONLY` at `review` when, at some frame `render` paints, nothing
  lower in the stack meets a blended element's box; a partial overhang is silent.
- **A `mask`'s `x`, `y`, `width`, `height` are element-local, and their identity value is the
  element's own rect** (ADR-0084). `(0, 0)` is the element rect's top-left **whatever the
  `origin` keyword is** — `origin` places the box, it does not re-parameterise the box's
  interior — and they are unscaled element units, not frame pixels. Omit all four and the
  rect is `(0, 0, width, height)`, which is what makes `{"name": "mask", "shape": "circle"}`
  the largest circle inscribed in the element's rect. The two arities are two declarations,
  not two spellings: the bare form re-derives when the element is resized, the explicit one
  keeps saying the rect it names.
- **A mask rides the element's transform** (ADR-0084). It is declared inside the box in
  unscaled units, so `scale` grows it and `rotation` turns it — a rotated element's `rect`
  mask paints a *rotated* rectangle. That is the rule a `blur` radius and a `stroke_width`
  already follow, and it is not keyframing an effect parameter: the fields stay literal
  integers on every frame. `clip` is the other thing — frame-space, static, never rotating —
  and a *shaped* static porthole is not expressible in v1.
- **`chroma` keys a screen colour out, and `color` is a literal `#RRGGBB`** (ADR-0088). Not
  a hue angle: a bare hue *inverts* the key on real footage, and supplying the saturation
  and value it is missing is the colour restated in three fields. `tolerance` is a
  normalised distance in the chroma plane, `softness` the width of the partial-alpha band
  above it, and `spill` suppresses screen colour reflected onto what the matte keeps. All
  three run `0.0`–`1.0` and every one of them has its identity at `0` — `tolerance: 0` keys
  nothing, which makes the whole member a no-op.
- **A key serves a screen that is uniform in time** (ADR-0088). No effect parameter is
  keyframable, so one `tolerance` covers the whole element: footage whose lighting drifts
  mid-take has to be cut into elements at the drift boundaries, or keyed upstream and
  brought in already carrying alpha. `measure` on a keyed element reports the resulting
  alpha coverage per frame, which is how you find where a screen drifts — and how you find
  out that `tolerance: 0.01` keyed nothing, without looking at a picture.
- **Put `chroma` before the colour scalars, not after** (ADR-0088). `effects` is ordered,
  so a `tint` or a `saturation` ahead of the key changes the pixels the key is measured
  against and your `color` no longer names what is in the frame. `validate` reports it at
  `review` rather than refusing it, because a deliberate pre-grade is a real technique.
- **`circle` is the one mask shape that discards part of its rect** (ADR-0084). Its diameter
  is the short side, so on a non-square rect — derived or explicit — the difference is thrown
  away without the author ever typing that number, and `validate` says so at `review`.
  `ellipse` fills the rect and `rect` is the rect, so neither is ever reported. Writing a
  square rect out is itself the acknowledgement; there is no suppression mechanism.
- **`"invert": true` on a `mask` keeps the outside of the shape and erases the inside** (ADR-0152).
  A static boolean: omitted or `false` is the plain mask, and a mask that flips mid-clip is two
  elements. Masks in one list intersect, so `[mask ellipse, mask smaller ellipse with invert]`
  keeps a ring.
- **`R-MASK-ERASES-ALL` (`review`)** (ADR-0152): an inverted `rect` mask with no `radius` whose
  rect contains the element's keeps no pixel, and the bare form always does. Write the rect you
  meant, or drop `invert`; writing the covering rect out does not silence it. Keyed rects,
  keyed `radius` and keyed element sizes are never reported.

## Transitions

- **A transition is its own element bridging `from` and `to` by id** (ADR-0059): two
  distinct visual elements on separate tracks, its `start`/`end` exactly the time they share.
- **`kind`** (ADR-0150): `crossfade` ramps their opacity linearly; in a `slide` `to` moves in
  over a still `from`; in a `push` `to` moves in and pushes `from` out, joined; in a `wipe` a
  straight edge crosses the frame, `from` keeping the side not yet reached and `to` the side
  passed.
- **`direction`** is the way the motion travels: `left` means the content moves leftward;
  the incoming element enters from the right. Required on `wipe`, `slide` and `push`.
- **`ease`** takes the keyframe vocabulary; omitted means `linear`. Neither `direction` nor
  `ease` is allowed on a `crossfade`.
- **Offsets travel a full frame width or height, outside the element's transform,** in whole
  pixels: a lower-third slides as far as full-screen video, its own keyed motion, effects
  and `clip` moving with it. A wipe's hard edge is cut against each element's own `clip`.
- **In a slide, `to` must paint above `from`**; a transition never changes a layer.
