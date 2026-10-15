---
status: accepted
amends: 0040 (expands closed effects vocabulary with mosaic, radial_blur, find_edges, emboss, sharpen, twirl, bulge, optics_compensation, and offset), 0156 (settles deferred edge-sampling rule as clamp within element bounds; admits deferred members mosaic, radial_blur, find_edges, and emboss; admits sharpen; admits closed coordinate distortions twirl, bulge, optics_compensation, and offset; confirms refusal of bilateral/smart_blur, Color Emboss, procedural noise distortions, and freehand mesh warps), 0215 (disposes of Blur & Sharpen, Stylize, and Distort census rows from the After Effects censuses on map #896)
---

# Edge sampling is clamp within bounds, mosaic, radial_blur, sharpen, find_edges, emboss, and four distortions enter effects[]

[Which sharpen, edge-aware blur, distortion and deferred ADR-0156 filter members enter, and what is the edge-sampling rule?](https://github.com/MBehtemam/Montagent/issues/1064)
on the After Effects map ([#896](https://github.com/MBehtemam/Montagent/issues/896)).

[ADR-0040](0040-effect-model-attachment-and-v1-vocabulary.md) established the ordered `effects[]` pipeline.
[ADR-0156](0156-four-named-effects-join-the-effects-list-and-a-code-drawn-piece-enters-as-pre-rendered-footage.md)
admitted four named effects (`grain`, `glow`, `posterize`, `directional_blur`) and established the three-part
growth test for future members:
1. The look cannot already be composed from existing members, paints, and blends.
2. It is one built-in Skia filter or one small runtime shader written by Montagent with typed parameters.
3. It stays inside the element's box, or inside a reach it declares as a formula of its parameters.

In §9, ADR-0156 deferred the distortions and radial blurs because their edge-sampling rule was undecided, and
deferred sharpen, emboss, find edges, and mosaic to keep the first batch small.

The After Effects censuses ([#904](https://github.com/MBehtemam/Montagent/issues/904) rows 48–62 and
[#905](https://github.com/MBehtemam/Montagent/issues/905) rows 29–60, 74–102) documented the full scope of these
unresolved filter groups.

This ADR decides:
1. The canonical **edge-sampling rule** across all spatial filters and coordinate-remapping shaders.
2. The schema, parameters, and evaluation rules for deferred ADR-0156 members: **`mosaic`**, **`radial_blur`**,
   **`find_edges`**, and **`emboss`**.
3. The addition of **`sharpen`**, and the refusal / routing of heavy edge-preserving blurs (`bilateral`, `smart_blur`).
4. The addition of four closed-form, single-element coordinate distortions: **`twirl`**, **`bulge`**,
   **`optics_compensation`**, and **`offset`**, and the refusal of procedural or freehand warps.

---

## Decisions

### 1. The edge-sampling rule: clamp within element bounds, decal across declared reach

When spatial convolution filters and coordinate remappings sample pixels outside the element's declared box,
sampling transparent black produces an unwanted dark border vignette and transparent fringe along the edges.

1. **Within-box filters and distortions use `TileMode::Clamp`:**
   All filters and distortions that operate strictly within the element's declared bounding box (`mosaic`,
   `radial_blur`, `sharpen`, `find_edges`, `emboss`, `twirl`, `bulge`, `optics_compensation`) clamp their sampling
   coordinates to the element boundary ($x \in [0, W-1]$, $y \in [0, H-1]$). This matches Skia's `TileMode::Clamp`
   and After Effects' *Repeat Edge Pixels* standard.
2. **Outward-reach filters retain decal transparency (`TileMode::Decal`):**
   Effects that declare an outward reach beyond the element rect (`blur`, `shadow`, `glow`) continue to use decal
   transparency, fading to transparent black across their declared margin ($3\sigma$).
3. **`offset` supports selectable wrapping:**
   `offset` pans the image within its box. It defaults to `wrap: true` (`TileMode::Repeat`), wrapping coordinates
   modulo width and height for seamless looping patterns. When `wrap: false`, it clamps coordinates (`TileMode::Clamp`).

---

### 2. `mosaic` and `radial_blur` enter `effects[]`

#### `mosaic`
Pixelates the element into discrete rectangular blocks:

```json
{"name": "mosaic", "blocks": [32, 18]}
```

- `blocks`: `[u32; 2]`, specifying $[N_x, N_y]$ block divisions across width and height. Both $\ge 1$.
- Identity value: when $[N_x, N_y]$ equals $[W, H]$ (element pixel dimensions).
- Animatable: both block counts animate in place, resolving to rounded integers $\ge 1$.
- Evaluated in element-local coordinates by quantizing sample coordinates:
  $$u' = \frac{\lfloor u \cdot N_x \rfloor + 0.5}{N_x}, \qquad v' = \frac{\lfloor v \cdot N_y \rfloor + 0.5}{N_y}$$

#### `radial_blur`
Simulates zoom or rotational camera smear around a focal origin:

```json
{"name": "radial_blur", "kind": "zoom", "center": [0.5, 0.5], "amount": 15.0}
{"name": "radial_blur", "kind": "spin", "center": [0.5, 0.5], "amount": 8.0}
```

- `kind`: `"zoom"` (radial smear along vectors to/from `center`) or `"spin"` (tangential smear along circular arcs
  around `center`).
- `center`: `[cx, cy]` in declared-box fractions `[0..1, 0..1]` from top-left. Optional; default `[0.5, 0.5]`.
- `amount`: float in $[0.0, 100.0]$. Identity value: `0.0`.
- Animatable: both `center` and `amount` animate in place.
- Evaluation: Single-pass Skia runtime shader taking a fixed 16 samples evenly distributed along the radial or
  tangential vector, weighted equally with `TileMode::Clamp`. The fixed sample count guarantees byte-identical
  parallel rendering across 1–10 painter threads without CPU/GPU branch drift.

---

### 3. `find_edges` and `emboss` enter `effects[]`; Color Emboss is refused

#### `find_edges`
Detects and highlights contrast boundaries:

```json
{"name": "find_edges", "invert": false}
```

- `invert`: optional boolean, default `false`. When `false`, black background with highlighted colored edges; when
  `true`, inverts output to a white background with dark edges (matching After Effects' default).
- Evaluation: Standard $3 \times 3$ Sobel gradient convolution on RGB channels with `TileMode::Clamp`.

#### `emboss`
Directional bas-relief lighting from a virtual light source:

```json
{"name": "emboss", "angle": 45.0, "height": 2.0, "contrast": 1.0}
```

- `angle`: float in degrees specifying the illumination direction (0° from $+x$, clockwise).
- `height`: float in $[0.0, 10.0]$ specifying the relief depth. Identity value: `0.0`.
- `contrast`: optional float in $[0.0, 5.0]$, default `1.0`.
- Evaluation: Directional difference convolution centered on neutral mid-grey `(0.5, 0.5, 0.5)` with `TileMode::Clamp`.
- **Color Emboss is refused:** Composing `emboss` on an element with `blend: "overlay"` or `"hard_light"` achieves the
  identical chroma-preserving look using existing primitives.

---

### 4. `sharpen` enters `effects[]`; edge-preserving blurs stay on `montagent-prerender`

#### `sharpen`
Accentuates high-frequency edge contrast:

```json
{"name": "sharpen", "amount": 1.5}
```

- `amount`: float in $[0.0, 5.0]$. Identity value: `0.0`.
- Animatable in place.
- Evaluation: $3 \times 3$ unsharp mask / Laplacian convolution with `TileMode::Clamp`:
  $$I' = I + \text{amount} \cdot (I - \text{box\_blur}_3(I))$$
- **`bilateral` and `smart_blur` are refused from the format:**
  Non-separable range/spatial filters require $O(r^2)$ kernel passes or iterative bilateral grid calculations.
  They present substantial GPU execution variance and thread-timeout risks in parallel rasterization, and are
  cleanly served by `montagent-prerender` ([ADR-0156](0156-four-named-effects-join-the-effects-list-and-a-code-drawn-piece-enters-as-pre-rendered-footage.md) §7).

---

### 5. Four closed coordinate distortions enter `effects[]`

Four closed-form, single-element parametric coordinate remappings join `effects[]`. Each is a pure mathematical
mapping $f(x, y) \to (x', y')$ executed in a single-pass Skia runtime shader:

#### `twirl`
Rotates pixels around a center point with quadratic falloff:

```json
{"name": "twirl", "center": [0.5, 0.5], "radius": 0.5, "angle": 90.0}
```

- `center`: `[cx, cy]` in declared-box fractions (default `[0.5, 0.5]`).
- `radius`: float $> 0.0$ in declared-box fraction (relative to $d_{\max}$).
- `angle`: float in degrees (signed; positive = clockwise). Identity value: `0.0`.
- Evaluated via angle rotation $\theta(r) = \text{angle} \cdot (1 - r/R)^2$ for $r < R$, with `TileMode::Clamp`.

#### `bulge`
Magnifies or pinches a spherical region around a center point (unifying AE Bulge and Spherize):

```json
{"name": "bulge", "center": [0.5, 0.5], "radius": 0.5, "height": 0.4}
```

- `center`: `[cx, cy]` in declared-box fractions (default `[0.5, 0.5]`).
- `radius`: float $> 0.0$ in declared-box fraction.
- `height`: float in $[-1.0, 1.0]$. Identity value: `0.0`. Positive values bulge outwards (magnify center);
  negative values pinch inwards.

#### `optics_compensation`
Radial lens distortion correcting or adding wide-angle curvature:

```json
{"name": "optics_compensation", "fov": 45.0, "reverse": false}
```

- `fov`: field-of-view angle in degrees in $[0.0, 180.0]$. Identity value: `0.0`.
- `reverse`: optional boolean, default `false`. `false` generates barrel distortion (wide-angle bulge); `true`
  generates pincushion distortion.

#### `offset`
Pans image coordinates within the element frame:

```json
{"name": "offset", "shift_x": 100.0, "shift_y": -50.0, "wrap": true}
```

- `shift_x`, `shift_y`: signed pixel translation. Identity value: `[0.0, 0.0]`.
- `wrap`: optional boolean, default `true`. When `true`, coordinates wrap modulo element box (`TileMode::Repeat`);
  when `false`, coordinates clamp (`TileMode::Clamp`).

#### Refused distortions
- **`turbulent_displace` and `wave_warp`**: Procedural noise and sinusoidal generators require multi-octave Perlin/simplex
  noise state; they belong on `montagent-prerender` or Ticket #1070.
- **`liquify` and `mesh_warp`**: Freehand interactive brush deformation nets lack flat, deterministic scalar schemas.
- **`mirror`**: Refused as an effect because cloning the element with `scale: [-1, 1]` and a `mask` already achieves
  exact reflections in the format.

---

## Consequences

- `crates/montagent-core/src/model/effects.rs` and `schema/montagent.schema.json`:
  - `Effect` discriminated union adds:
    - `Mosaic { blocks: Animatable<[u32; 2]> }`
    - `RadialBlur { kind: RadialBlurKind, center: Option<Animatable<[f64; 2]>>, amount: Animatable<f64> }`
    - `FindEdges { invert: Option<bool> }`
    - `Emboss { angle: Animatable<f64>, height: Animatable<f64>, contrast: Option<Animatable<f64>> }`
    - `Sharpen { amount: Animatable<f64> }`
    - `Twirl { center: Option<Animatable<[f64; 2]>>, radius: Animatable<f64>, angle: Animatable<f64> }`
    - `Bulge { center: Option<Animatable<[f64; 2]>>, radius: Animatable<f64>, height: Animatable<f64> }`
    - `OpticsCompensation { fov: Animatable<f64>, reverse: Option<bool> }`
    - `Offset { shift_x: Animatable<f64>, shift_y: Animatable<f64>, wrap: Option<bool> }`
- `crates/montagent-render/src/canvas.rs`:
  - Implements Skia runtime shaders / convolution filters for all 9 new members with `TileMode::Clamp` (and `TileMode::Repeat` for `offset.wrap: true`).
  - Gated by byte-identical multi-painter tests (1–10 threads).
- `CONTEXT.md` is updated to record the nine new effect members and the within-box edge-sampling rule.
