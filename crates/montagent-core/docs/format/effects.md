# The Montagent project format: grain and named effects

A page of `montagent://format.md`, which holds the rules every element shares and lists the
other pages; read it first. This page holds `grain` and the named `effects` members
(`posterize`, `glow`, `directional_blur`, `mosaic`, `radial_blur`, `find_edges`, `emboss`,
`sharpen`, `twirl`, `bulge`, `optics_compensation` and `offset`). Like the rest of the format
docs, every rule here is an accepted decision in the ADR series, cited inline by number, and
the ADR is right where the two disagree.

## Grain

- **`grain` is film grain** (ADR-0156):
  `{"name": "grain", "seed": 7, "amount": 0.2, "size": 2, "mono": true}`. Every key is
  required. `seed` is an integer from `0` to `2147483647`; `amount` runs `0`–`1`; `size` is
  an integer from `1` to `8`; `mono` is a boolean. Only `amount` may be keyed: a keyframe
  list on `seed`, `size` or `mono` is a schema error. `amount: 0` paints the same bytes as no
  member, and no finding flags it, so a grain can fade in from `0`.
- **Cells sit in element space** (ADR-0156). Each `size`×`size` cell of unscaled element
  units, counted from the box's top-left, shares one draw, so the cells move, turn and scale
  with the element. With `mono: true` one draw offsets R, G and B alike (luma grain); with
  `false` each channel draws on its own (colour grain). The draw `d`, `0`–`255`, offsets the
  non-premultiplied colour by `amount × (2d − 255) / 255`, clamped to `0`–`1`. Alpha never
  changes: a transparent pixel stays transparent, so a `rect` keeps its edges, and `grain`
  has no reach.
- **The seed re-rolls on every output frame** (ADR-0156 §3). The draw is a fixed integer
  hash of the seed, the cell and the element's **local frame**: the frame being painted,
  less the first frame the element's `start` lets it paint (`⌈start × fps / 1000⌉`). So the
  re-roll rate follows `fps`; an element moved by N whole frames paints the same pixels N
  frames later; and under `motion_blur` every sample of one frame shares that frame's draw.
  The hash is SplitMix64's step (add `0x9E3779B97F4A7C15`, then its finaliser) applied in
  turn to the seed, then folding in by exclusive or the local frame, the cell row, the cell
  column and the channel (`0`, `1`, `2`; `mono` draws `0`), as 64-bit two's complement; `d`
  is the top byte. `query --at` prints each grain's resolved values and its local frame.
- **A texture is a `rect` with `grain` and a `blend`** (ADR-0156 §1). There is no element
  that paints from nothing: a mid-grey `rect` (`#808080`) with `grain`, blended `overlay`
  over footage, adds grain to the footage and leaves it otherwise as it was.
- **`R-GRAIN-SEED-SHARED` (`review`)** (ADR-0156): two `grain` members with the same `seed`,
  `size` and `mono`, on elements visible together whose `start`s fall on the same frame
  (one element's list included), draw the same pattern on every frame, a locked texture.
  Change one `seed`. `amount` is not compared.

## Named effects

- **Every named effect is an `effects` member** (ADR-0156, ADR-0225): applied in list order,
  each filtering the element's own pixels plus any reach it declares. Every numeric or pair
  parameter is animatable in place and every enum or boolean (`kind`, `invert`, `reverse`,
  `wrap`) is static. A value out of range is `E-OUT-OF-RANGE`, in a static value and in every
  keyframe record.
- **Edge-clamped coordinate sampling** (ADR-0225): spatial kernels, radial blurs and geometric
  distortions (`mosaic`, `radial_blur`, `find_edges`, `emboss`, `sharpen`, `twirl`, `bulge`,
  `optics_compensation`, and `offset` with `"wrap": false`) sample the input at coordinates
  clamped to the element's local box `[0, width] × [0, height]` (`TileMode::Clamp`), so border
  pixels extend cleanly instead of bleeding transparent black inward or expanding the layer
  bound. Only `blur`, `shadow`, `glow` and `directional_blur` expand the layer bound.
- **`posterize`: `{"levels": 2–256}`, an integer** (ADR-0156). Per channel, on
  non-premultiplied sRGB values from 0 to 1, `q = round(v × (levels − 1)) / (levels − 1)`,
  ties away from zero, so `levels: 2` sends 0.49 to 0 and 0.5 to 1. Alpha is untouched, and
  `256` paints the same bytes as no member. A keyed `levels` resolves to a continuous value
  and is rounded half away from zero, as `shift` rounds.
- **`glow`: `{"threshold": 0–1, "radius": ≥ 0, "intensity": 0–4}`, a threshold bloom**
  (ADR-0156). The bright-pass scales each pixel by `max(0, luma − threshold) / (1 − threshold)`,
  luma Rec.709 on the non-premultiplied sRGB values, alpha with the colour. That bright part
  is blurred with σ = `radius` / 2, exactly as `blur` reads a radius, multiplied by
  `intensity`, and added (`Plus`) over the element inside its own layer, before `blend`. Its
  reach is `blur`'s 3σ. `threshold: 1` (an empty bright-pass) and `intensity: 0` paint the
  same bytes as no member. There is no `color`: a coloured glow from the alpha is a
  zero-offset `shadow`.
- **`directional_blur`: `{"angle": degrees, "length": 0–4096}`, a centred smear** (ADR-0156).
  `angle` 0 smears horizontally and positive turns clockwise, as `rotation` does, so 0 and 180
  give the same smear. It is measured in the element's own space: it turns with the element's
  `rotation`, mirrors under a flip, and `length` is in element pixels, so it grows with
  `scale`. It takes `ceil(length) + 1` samples, evenly spaced along the line through each
  pixel and centred on it, read bilinearly and weighted equally; `length: 0` paints the same
  bytes as no member. Its reach is `(|cos θ| × length / 2, |sin θ| × length / 2)`, rounded up
  to whole pixels. This is not motion blur: it smears whether or not the element moves.
- **A keyed `length` steps the sample count**, since `ceil(length) + 1` is a whole number, and
  **a keyed `angle` interpolates literally**: `350 → 10` sweeps the long way round, through
  180. Write `350 → 370` for the short way.
- **`mosaic`: `{"blocks": [bx, by]}`, each an integer `≥ 1`** (ADR-0225). Divides the element
  box `(W, H)` into `bx × by` blocks of size `(W / bx, H / by)` and samples the input at each
  block's centre `((⌊x · bx / W⌋ + 0.5) · W / bx, (⌊y · by / H⌋ + 0.5) · H / by)`. `blocks: [W, H]`
  is the identity. A keyed `blocks` interpolates each axis and rounds half away from zero.
- **`radial_blur`: `{"kind": "zoom" | "spin", "center"?: [cx, cy], "amount": 0–100}`**
  (ADR-0225). `center` is in normalised element-box fractions (`0..=1` per axis; default
  `[0.5, 0.5]`). `amount: 0` is the identity. `spin` averages `N = 16` samples along the arc
  through `(x, y)` around `(cx·W, cy·H)` over total angle `Δθ = (amount / 100) × (π / 2)`
  radians (`±Δθ/2`); `zoom` averages `N = 16` samples radially along the ray from `(cx·W, cy·H)`
  to `(x, y)` with scale factor `s ∈ [1 − 0.5·(amount/100), 1]`.
- **`find_edges`: `{"invert"?: bool}`** (ADR-0225). Computes the 3×3 Sobel gradient magnitude
  `G = clamp(√(Gx² + Gy²), 0, 1)` per RGB channel on non-premultiplied sRGB; `"invert": false`
  (the default) draws light edges on dark (`E = G`), `"invert": true` draws dark edges on white
  (`E = 1 − G`). Alpha is preserved from the centre pixel.
- **`emboss`: `{"angle": degrees, "height": 0–10, "contrast"?: 0–5}`** (ADR-0225). Computes
  directional relief `R = clamp(0.5 + contrast × (I(p + d) − I(p − d)), 0, 1)` along
  `d = (cos θ, sin θ) × height` in element pixels (`contrast` defaults to `1.0`; `height: 0` is
  the identity). Alpha is preserved from the centre pixel.
- **`sharpen`: `{"amount": 0–5}`** (ADR-0225). Applies a 3×3 Laplacian unsharp mask on
  non-premultiplied sRGB: `out = clamp(c + amount × (4·c − n − s − e − w), 0, 1)`, preserving
  the centre pixel's alpha. `amount: 0` is the identity.
- **`twirl`: `{"center"?: [cx, cy], "radius": > 0, "angle": degrees}`** (ADR-0225). `center`
  defaults to `[0.5, 0.5]` (`0..=1` per axis). Within `R = radius × max(W, H)`, rotates sample
  coordinates around `(cx·W, cy·H)` by `θ(r) = radians(angle) × (1 − r / R)²` for `r < R`
  (`angle: 0` is the identity).
- **`bulge`: `{"center"?: [cx, cy], "radius": > 0, "height": -1–1}`** (ADR-0225). `center`
  defaults to `[0.5, 0.5]` (`0..=1` per axis). Within `R = radius × max(W, H)`, scales radial
  distance around `(cx·W, cy·H)` by the smooth parabolic profile
  `r' = r × (1 − height × (1 − (r / R)²))` (`height > 0` magnifies, `height < 0` pinches,
  `height: 0` is the identity).
- **`optics_compensation`: `{"fov": 0–180, "reverse"?: bool}`** (ADR-0225). Applies quadratic
  radial lens distortion around the element box centre `(0.5·W, 0.5·H)` normalised by the
  half-diagonal `D = 0.5 × √(W² + H²)` with `k = 0.75 × (fov / 180)²`: `"reverse": false` (the
  default, barrel distortion) maps `r' = r × max(0.1, 1 − k·u²)`, `"reverse": true` (pincushion
  distortion) maps `r' = r × (1 + k·u²)` (`fov: 0` is the identity).
- **`offset`: `{"shift_x": sx, "shift_y": sy, "wrap"?: bool}`** (ADR-0225). Translates content
  in unscaled element-local pixels by sampling at `(x − shift_x, y − shift_y)`. `"wrap": true`
  (the default) wraps periodically modulo `(W, H)` for seamless scrolling or rolling loops;
  `"wrap": false` clamps at the element's border. `shift_x: 0, shift_y: 0` is the identity.
- **The bound criterion** (ADR-0156, ADR-0225): a member keeps the element's bounds when it
  stays inside the box or a reach it declares as a formula of its parameters. A colour filter
  that maps transparent black to transparent black keeps it (`posterize`), and every
  edge-clamped member keeps the element's own `[0, width] × [0, height]` box. None of this is
  visible: the frame is the same bytes either way.
- **Cost**: `directional_blur` reads `ceil(length) + 1` pixels and `radial_blur` reads 16
  pixels for every pixel it paints, so either over a full-frame element is slower than a plain
  `blur`; `glow` costs more than a `blur` of the same radius.
