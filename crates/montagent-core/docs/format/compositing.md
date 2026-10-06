# The Montagent project format: compositing

A page of `montagent://format.md`, which holds the rules every element shares and lists the
other pages; read it first. This page holds the rules for how elements combine: the paint
drawn through an element's box, an element's `effects`, how the finished element composites
into what is below it, and the transition elements that bridge two others. Like the rest of the format docs, every rule here is an
accepted decision in the ADR series, cited inline by number, and the ADR is right where the
two disagree.

## Paint

- **A paint is a colour, a keyframe list of colours, or a gradient** (ADR-0149). These fields
  take one: a `rect`'s, an `ellipse`'s and a `path`'s `fill` and `stroke`, and a `text`
  element's element-level `color` and `stroke`. Every other colour field takes a colour only: run and
  highlight paint, the project `background`, and `shadow`, `tint` and `chroma` colours. For a
  gradient background, put a full-frame `rect` with a gradient `fill` at the bottom.
- **A gradient is `linear` or `radial`, and every parameter is required** (ADR-0149):
  `{"gradient": "linear", "angle": 90, "stops": [...]}` or
  `{"gradient": "radial", "center": [0.5, 0.5], "radius": 1, "stops": [...]}`. `linear`
  refuses `center` and `radius`; `radial` refuses `angle`. Nothing defaults.
- **A stop is `{"offset", "color"}`**, `offset` from `0` to `1`, `color` a colour whose alpha
  is the stop's opacity. At least two stops, no upper limit. Offsets never decrease
  (`E-GRADIENT-STOP-ORDER`, naming the first pair out of order); equal offsets are legal and
  make a hard edge. A `stops` array is a keyframe list when its first item has a `t`.
- **It is measured against the declared box, never the glyphs** (ADR-0149): the declared
  rect of a shape, the same box for its `stroke` as for its `fill`, and the declared text box
  of a text element. That text box shares the typographic block's pivot point: with `origin`
  at `(fx, fy)` of each, a `center` origin centres the box on the block. Moving, scaling or
  resizing the element carries the gradient with it.
- **`linear`**: `angle` in degrees, CSS's convention — `0` runs bottom to top, `90` left to
  right, turning clockwise; any number, so `370` is `10`. The line passes through the box's
  centre in direction `(sin a, −cos a)` (y down) and is `|W·sin a| + |H·cos a|` long, so the
  box's corners land on offsets `0` and `1` whatever the aspect ratio.
- **`radial`**: `center` is `[fx, fy]`, fractions of the box from its top-left (any numbers,
  so a glow may sit outside it). `radius` (≥ 0) is a fraction of the distance from the centre
  to the farthest corner, in box fractions: `1` just reaches it. The circle is in box
  fractions, so on a box that is not square it draws as an ellipse. `center: [0.5, 0.5]`,
  `radius: 1` is CSS's default radial gradient. `radius: 0` paints the last stop's colour
  over the whole box.
- **Past the ends the first and last colours extend; colour blends in sRGB with premultiplied
  alpha** (ADR-0149), the rule a keyed colour uses across time. A fade to `#00000000` keeps
  its hue rather than passing through grey. There is no repeat or mirror.
- **Gradient parameters cannot be keyed yet.** `angle`, `center`, `radius` and `stops` are
  literals, and a paint's keyframe list holds colours, never a gradient: both are schema
  errors. A paint field holds a keyed colour or a static gradient.
- **Two reviews** (ADR-0149): `R-GRADIENT-ONE-COLOUR`, a gradient that paints a single colour
  (its stops share one, or its `radius` is 0); and `R-TEXT-PAINT-OVERRIDDEN`, a text
  `color` or `stroke` gradient that every run overrides, so it is never drawn. A run's own
  colour still beats the element's gradient, run by run.
- **`query --at` prints a gradient as a literal you can paste back**: kind, parameters and
  stops as written, colours as hex. Where offsets cross, it prints what is drawn: each stop
  raised to the largest offset before it, keeping its own colour.

## Effects and compositing

- **Effects are an ordered list, and the order is semantically real** (ADR-0040). Blur-then-shadow is a
  different frame from shadow-then-blur. They attach to whole elements, never to a run, and
  the one animatable effect parameter so far is a `mask`'s `feather`.
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
- **A key serves a screen that is uniform in time** (ADR-0088). `chroma`'s parameters are
  not animatable yet, so one `tolerance` covers the whole element: footage whose lighting drifts
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
- **`feather` on a `mask` softens its edge, and only its edge** (ADR-0152). A non-negative
  integer in unscaled element units that rides the transform; omitted and `0` are the same hard
  edge. It reads as a `blur` radius: the mask's coverage is blurred by a Gaussian of
  σ = `feather` / 2, centred on the edge, so the ramp reaches past the rect as far as it falls
  short of it, and an inverted feathered mask keeps exactly what the plain one erases.
  `[mask, blur]` also blurs the picture; `feather` does not. It may be keyed, and resolves
  unrounded between keys.
- **`R-MASK-ERASES-ALL` (`review`)** (ADR-0152): an inverted `rect` mask with no `radius` or
  `feather` whose rect contains the element's keeps no pixel, and the bare form always does.
  Write the rect you meant, or drop `invert`; writing the covering rect out does not silence it.
  Keyed rects, `radius`, `feather` and element sizes are never reported.

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
