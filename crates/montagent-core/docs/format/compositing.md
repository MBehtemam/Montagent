# The Montagent project format: compositing

A page of `montagent://format.md`, which holds the rules every element shares and lists the
other pages; read it first. This page holds the rules for how elements combine: the paint
drawn through an element's box, an element's `effects` and the named effects, its motion blur, and how the finished element composites
into what is below it. The transition elements that bridge two others are on `montagent://format/transitions.md`. Like the rest of the format docs, every rule here is an
accepted decision in the ADR series, cited inline by number, and the ADR is right where the
two disagree.

## Paint

- **A paint is a colour, a keyframe list of colours, or a gradient** (ADR-0149). These fields
  take one: a `rect`'s, an `ellipse`'s and a `path`'s `fill` and `stroke`, and a `text`
  element's element-level `color` and `stroke`. Every other colour field takes a colour only: run and
  highlight paint, the project `background`, and `shadow`, `tint` and `chroma` colours. For a
  gradient background, put a full-frame `rect` with a gradient `fill` at the bottom.
- **A gradient is `linear`, `radial`, `sweep` or `two_point_conical`, and every parameter is
  required** (ADR-0149, ADR-0222):
  `{"gradient": "linear", "angle": 90, "stops": [...]}`,
  `{"gradient": "radial", "center": [0.5, 0.5], "radius": 1, "stops": [...]}`,
  `{"gradient": "sweep", "center": [0.5, 0.5], "angle": 0, "stops": [...]}`, or
  `{"gradient": "two_point_conical", "center": [0.5, 0.5], "radius": 1, "focal": [0.35, 0.35], "stops": [...]}`.
  Each kind refuses the others' keys (`linear` refuses `center`, `radius` and `focal`; `radial`
  refuses `angle` and `focal`; `sweep` refuses `radius` and `focal`; `two_point_conical`
  refuses `angle`). Nothing defaults.
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
- **`sweep`** (ADR-0222): `center` is `[fx, fy]` in box fractions; `angle` is in degrees,
  CSS `conic-gradient` convention — `0` starts at 12 o'clock (top, `[cx, cy − ε]`) and sweeps
  `360°` clockwise, so `90` starts at 3 o'clock, `180` at 6 o'clock, and `270` at 9 o'clock.
  The rotation happens in unit-box coordinates before scaling to `(W, H)`, so on a non-square
  box `45°` points toward the top-right corner `[1, 0]`. When the first and last stops differ,
  a hard seam sits along `angle`; match the first and last stop colours for a continuous ring.
- **`two_point_conical`** (ADR-0222): `center` and `focal` are `[fx, fy]` in box fractions,
  and `radius` (≥ 0) is a fraction of the distance from `center` to the box's farthest corner
  in box fractions (`R = radius × farthest_corner(center)`). Stop offset `0` sits at `focal`
  (inner radius `0`), and stop offset `1` sits on the circle of radius `R` around `center`,
  giving an off-axis specular highlight or 3D sphere shading. When `focal == center`, it draws
  identically to `radial`. When `focal` lies on or outside the outer circle (`|focal − center| ≥ R`),
  the renderer clamps `focal` along the ray from `center` to `0.999 × R` so the entire box stays
  painted (`query --at` still prints the unclamped `focal` from the document). `radius: 0`
  paints the last stop's colour over the whole box.
- **Past the ends the first and last colours extend; colour blends in sRGB with premultiplied
  alpha** (ADR-0149), the rule a keyed colour uses across time. A fade to `#00000000` keeps
  its hue rather than passing through grey. There is no repeat or mirror.
- **A gradient animates in place** (ADR-0149, ADR-0222): `angle`, `center`, `radius`, `focal`
  and `stops` each take a literal or a keyframe list of their own type, with their own `ease`.
  `stops` as a keyframe list has a whole stop list as each `v`: every `v` holds the same number
  of stops (`E-GRADIENT-STOP-COUNT`, naming the first record that differs), and stop *i* blends
  with stop *i*, its `offset` as a number and its `color` premultiplied. The kind never
  animates. A paint's keyframe list holds colours, never a gradient, and never a mix: both are
  schema errors. For flat to gradient, write the flat colour as a gradient whose stops share it,
  then key the stops:
  `{"gradient": "linear", "angle": 90, "stops": [{"t": 0, "v": [{"offset": 0, "color": "#FF3366"}, {"offset": 1, "color": "#FF3366"}]}, {"t": 800, "v": [{"offset": 0, "color": "#FF3366"}, {"offset": 1, "color": "#3366FF"}], "ease": "ease-out"}]}`.
- **A bezier may carry an offset past its neighbour**: each resolved offset is clamped to
  `0..1`, then raised to the largest offset before it, and a raised stop keeps its own colour,
  so a crossing is a hard edge. A resolved `radius` at or below `0` on `radial` or
  `two_point_conical` paints the last stop's colour over the box. `query --at` prints what is
  drawn, always a legal literal.
- **Two reviews** (ADR-0149): `R-GRADIENT-ONE-COLOUR`, a gradient whose resolved paint is a
  single colour at every frame `render` paints in the element's range (its stops share one, or
  its `radius` on `radial` or `two_point_conical` stays at or below `0`); a flat-to-gradient
  animation never fires it. And `R-TEXT-PAINT-OVERRIDDEN`, a text `color` or `stroke` that is
  keyed or a gradient and that every run overrides, so it is never drawn. A run's own colour
  still beats the element's gradient, run by run.
- **`query --at` prints a gradient as a literal you can paste back**: kind, resolved
  parameters and stops, colours as hex, under the paint's own name (`fill`). `validate` and
  the contact sheet name a nested list by its path: `fill.angle`, `fill.focal`, `stroke.stops`.
- **`shift` carries and splits every nested list.** A split writes the resolved value, so it
  is refused (`E-SHIFT-SPLIT-UNWRITABLE`, naming `fill.stops` and the instant) where the
  resolved offsets are out of `0..1` or crossed, tested before the fix above; splitting
  elsewhere is exact to colour byte rounding.

## Effects and compositing

- **Effects are an ordered list, and the order is semantically real** (ADR-0040). Blur-then-shadow is a
  different frame from shadow-then-blur. They attach to whole elements, never to a run.
- **Every numeric and colour effect parameter is animatable** (ADR-0146), written in place
  inside its member with `t` on the element's clock:
  `{"name": "blur", "radius": [{"t": 0, "v": 0}, {"t": 400, "v": 24, "ease": "ease-out"}]}`.
  `mask.shape`, `invert` and every other enum or boolean stay static, and `chroma.color` stays
  six-digit in every record. A parameter's range holds in every record (a `mask` size or
  `radius` is non-negative, `chroma`'s scalars `0`–`1`, `posterize`'s `levels` `2`–`256`); an overshoot between records
  clamps to it. Findings and `query --at` name a parameter by its member's position:
  `effects[1].radius (blur)`, since two members of one name are legal. `shift` carries and
  splits these lists like any other.
- **A reveal is a keyed `mask` rect from `0`** (ADR-0146, ADR-0025). A plain `mask` whose
  resolved `width` or `height` is at or below zero hides the whole element for that frame,
  so `"width": [{"t": 0, "v": 0}, {"t": 600, "v": 400, "ease": "ease-out"}]` on a 400-wide
  element wipes it on from the left edge. One that hides every frame is
  `E-NOT-PAINTED-NO-EXTENT`, naming the mask; an inverted one of no size hides nothing.
- **`blend` is how the finished element composites into what is below it** (ADR-0147, ADR-0221). One
  of seventeen words: `normal`, `multiply`, `screen`, `overlay`, `add`, `darken`, `lighten`,
  `color_dodge`, `color_burn`, `hard_light`, `soft_light`, `difference`, `exclusion`, `hue`,
  `saturation`, `color`, `luminosity`. It sits beside `opacity`
  on `rect`, `ellipse`, `path`, `text`, `image`, `video` and `nest`; `audio` and `transition` refuse it.
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
  already follow. `clip` is the other thing — frame-space, static, never rotating — and a
  *shaped* static porthole is not expressible in v1.
- **`chroma` keys a screen colour out, and `color` is a literal `#RRGGBB`** (ADR-0088). Not
  a hue angle: a bare hue *inverts* the key on real footage, and supplying the saturation
  and value it is missing is the colour restated in three fields. `tolerance` is a
  normalised distance in the chroma plane, `softness` the width of the partial-alpha band
  above it, and `spill` suppresses screen colour reflected onto what the matte keeps. All
  three run `0.0`–`1.0` and every one of them has its identity at `0` — `tolerance: 0` keys
  nothing, which makes the whole member a no-op.
- **A screen whose lighting drifts takes a keyed `tolerance`** (ADR-0088, ADR-0146), so the
  key follows the drift instead of the element being cut at each one. `measure` on a keyed
  element reports the resulting alpha coverage per frame, which is how you find where a
  screen drifts — and how you find out that `tolerance: 0.01` keyed nothing, without
  looking at a picture.
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
  keeps a ring. Plain and inverted are complements where either keeps or erases a pixel whole;
  a hard edge may differ, so a pair rebuilding one source takes `feather` ≥ 1.
  Stacked complements still show the backdrop by c·(1 − c) (ADR-0165).
- **`feather` on a `mask` softens its edge, and only its edge** (ADR-0152). A non-negative
  integer in unscaled element units that rides the transform; omitted and `0` are the same hard
  edge. It reads as a `blur` radius: the mask's coverage is blurred by a Gaussian of
  σ = `feather` / 2, centred on the edge, so the ramp reaches past the rect as far as it falls
  short of it, and the inverted one keeps what the plain one erases, within one level of 255.
  `[mask, blur]` also blurs the picture; `feather` does not. It may be keyed, and resolves
  unrounded between keys.
- **`R-MASK-ERASES-ALL` (`review`)** (ADR-0152): an inverted `rect` mask with no `radius` or
  `feather` whose rect contains the element's keeps no pixel, and the bare form always does.
  Write the rect you meant, or drop `invert`; writing the covering rect out does not silence it.
  Keyed rects, `radius`, `feather` and element sizes are never reported.
- **`"shape": "path"` masks through any closed outline** (ADR-0163), written inline in a
  `path` element's vertex vocabulary (see the paths page):

  ```json
  {"name": "mask", "shape": "path",
   "points": [{"at": [40, 300]}, {"at": [220, 40], "out": [60, 0]},
              {"at": [400, 280], "in": [-40, -60]}]}
  ```

  `points` is required under `path` and an unknown key under the other shapes; `radius` is
  an unknown key under `path`.
- **A path mask's `points` are measured from its rect, which bounds them and never scales
  them.** Omit the rect, the common case, and the points are element-local pixels from the
  element rect's top-left. Written, the rect is still all-or-none: a keyed `x` or `y` moves
  the whole mask without keying a vertex, and `width` and `height` are static, so a keyframe
  list on either is a schema error; reshape the mask by editing its `points`. It rides the
  transform as every mask does.
- **A path mask always closes**: the last segment runs from the last vertex back to the
  first, using the last `out` and the first `in`. There is no `closed` field, and a stray
  one is a schema error saying *a mask path always closes; drop `closed`.*
- **It keeps the outline's interior by the nonzero rule**, the rule a `path` fills with, so a
  self-crossing outline keeps its overlap. A hole is a second, inverted mask in the same list.
  `invert` and `feather` work as on every shape. `points` animates as a `path`'s does: a
  whole list per keyframe, number by number, and an overshoot clamps each absolute vertex
  and handle into the mask's box at that instant.
- **`validate` runs the point checks on a path mask's `points`**, in every literal value:
  `E-PATH-TOO-FEW-POINTS` (three vertices at least), `E-PATH-KEYFRAME-SHAPE`, and
  `E-PATH-OUTSIDE-BOX` with an inset of `0`, against the written rect, or the element's own
  rect where it is omitted. Where that element's `width` or `height` is keyed, the box is the
  smallest literal value of each. A feather reaching past the box is fine. Each finding
  names the mask's index in `effects`. `R-MASK-ERASES-ALL` and `R-MASK-CIRCLE-NON-SQUARE`
  never fire on a path mask. `query --at` prints each path mask's resolved outline with
  absolute control points, in box pixels, under `masks`.
- **A mask never names another element's outline** (ADR-0150): two masks that want one
  outline each carry a copy of its `points`.

## Motion blur

- **`motion_blur` smears an element along its own motion** (ADR-0155):
  `"motion_blur": {"shutter": 180, "samples": 8}` on a `rect`, `ellipse`, `path`, `text`,
  `image` or `video`. `shutter` is an integer angle from `1` to `360`: the interval is
  `shutter / 360` of one frame, centred on the frame instant. `samples`, from `2` to `32`, is
  how many paints a moving frame averages, so the cost is read from the file. Both keys are
  required and static; there is no `phase` and no project-wide setting. It is a field, not an
  `effects` member.
- **The samples sit at the midpoints** (ADR-0155). Frame n is painted at
  `instant(n) = ⌊n × 1000 / fps⌋` ms, and its samples at
  `instant(n) + shutter/360 × (1000/fps) × ((k + ½)/N − ½)` for k = 0 … N−1, exact rationals,
  not whole milliseconds. At `360` no two frames share an instant. `frame --at` an instant
  off the frame grid centres the samples on that instant.
- **The whole element follows the sample** (ADR-0155). Every value its own keyframes
  resolve is read at each sample instant: `x`, `y`, `scale`, `rotation`, `opacity`, every
  animatable property, effect parameters, gradient parameters, a path's `points` and a
  `units` stagger's poses. A `mask` of no size hides that sample alone. Transitions are not
  sampled. Presence, a run's `highlight` window and a `video`'s source frame are decided once,
  at the frame instant.
- **The order is: samples, average, blend** (ADR-0155). Each sample is painted with its
  `effects`, its `mask` and its `opacity` on a transparent layer; the N layers are averaged
  byte by byte on premultiplied RGBA, `(Σ + ⌊N/2⌋) / N`, so a tie rounds up and N identical
  samples give back exactly their bytes; then `blend` composites the average **once**.
- **A still element is painted once** (ADR-0155). Where every value is equal at all N
  instants, the frame paints the element as it would without the field, byte for byte. On
  the frame where a move stops, an antialiased edge may step by one level as it goes from
  averaged to painted once.
- **A `video`'s footage is not blurred, only its keyed values.** It holds the one source
  frame shown at the frame instant, resampled into each sample's box.
- **The smear reaches only as far as the keyframes do** (ADR-0155). Keyed values clamp past
  their first and last record, so on a frame where motion starts or stops at a keyframe the
  smear is one-sided. That includes the frame just after a `shift` cut, where the keys the
  cut writes clamp the later half's earlier samples. `shift` copies the field unchanged.
- **`R-MOTION-BLUR-STILL` (`review`)** (ADR-0155): no value of an element carrying the field
  differs between two instants inside its `[start, end)`, a `units` stagger counting as
  motion; motion wholly outside its life counts as still. Remove the field. `query --at`
  prints the field as written and `moving` or `still` for the frame holding the instant.

## The static prefix

- **The bottom run of unchanging layers is drawn once and reused** by `render` and `preview`
  (ADR-0195), until one of them enters or leaves. The frame's bytes are the same either way.
- **A time-varying layer ends the run**: a keyframe (effect and gradient parameters too), a
  `video`, a `group` member, a non-`normal` `blend`, `motion_blur`, `units`, a `highlight`, an
  effect outside `blur`, `shadow`, `mask`, `tint`, `saturation`, `brightness`, `contrast`,
  `chroma`, `posterize`, `glow`, or a running transition bridging it. Every layer above is
  drawn every frame, a static one between moving ones included.
- **`audio` and `transition` elements draw nothing** (ADR-0198): on any layer, they neither
  join nor end the run, and their start or end does not restart it.
- `render --profile` counts `frames_painted` (drawn) and `frames_reused` (copied from the run).
