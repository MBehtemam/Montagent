---
status: accepted
amends: 0040 (expands closed effects vocabulary with matte and luma_key), 0088 (preserves chroma as a pure colour keyer while delegating alpha channel post-processing to matte), 0150 (preserves element isolation: matte and luma_key operate strictly on element-local pixels), 0156 (confirms refusal of procedural fractal edge noise Roughen Edges from the format), 0215 (disposes of After Effects tracking, roto, and keying census rows 27-38 from map #896)
---

# Standalone matte and luma_key enter effects[], and iterative chokers and roughen edges are refused

[Do alpha chokers, matte cleanup, luma key and roughened edges enter effects[]?](https://github.com/MBehtemam/Montagent/issues/1069)
on the After Effects map ([#896](https://github.com/MBehtemam/Montagent/issues/896)).

[ADR-0040](0040-effect-model-attachment-and-v1-vocabulary.md) established the ordered `effects[]` pipeline.
[ADR-0088](0088-chroma-key-enters-the-effects-list-as-a-bt601-chroma-distance-with-tolerance-softness-and-spill.md)
admitted the `chroma` keyer with normalized BT.601 parameters (`color`, `tolerance`, `softness`, `spill`),
focusing strictly on screen-colour suppression in the chroma plane without dedicated alpha morphological filtering.
[ADR-0150](0150-wipe-slide-and-push-enter-as-transition-kinds-and-a-matte-from-another-element-is-refused.md) §4
established that no element's pixels decide another's, keeping all effects strictly element-local.
[ADR-0156](0156-four-named-effects-join-the-effects-list-and-a-code-drawn-piece-enters-as-pre-rendered-footage.md)
established the three-part growth test for new effects and refused complex procedural generators in favour of pre-rendered assets.
[ADR-0215](0215-after-effects-is-precedent-for-rendered-results-and-capcut-and-premiere-stay-first-for-shape.md)
admitted After Effects precedent for rendered results while keeping Premiere and CapCut first for mechanism and shape.

The After Effects censuses ([#903](https://github.com/MBehtemam/Montagent/issues/903) rows 27–38, [#905](https://github.com/MBehtemam/Montagent/issues/905) rows 67–70, 95)
identified gaps in post-key alpha edge refinement (*Simple Choker*, *Matte Choker*, *Alpha Levels*, *Key Cleaner*),
luminance extraction (*Extract*, *Luma Key*, *Unmult*), and procedural alpha boundary erosion (*Roughen Edges*).

This ADR decides:
1. **Standalone `matte` effect enters `effects[]`**: Alpha erosion/dilation (choke/spread), edge feathering, and alpha contrast/thresholding are provided via a dedicated, composable `matte` effect rather than overloading `chroma`.
2. **`luma_key` enters `effects[]`**: Luminance keying with low/high threshold clamps and softness ramps enters the closed effects vocabulary.
3. **Iterative hole-filling (*Matte Choker*) and multi-stage detail reconstruction (*Key Cleaner*) are refused**: Multi-pass iterative morphological loops and spatial edge reconstruction kernels are refused from the format and remain on `montagent-prerender`.
4. **Procedural fractal edge erosion (*Roughen Edges*) is refused**: Complex fractal noise displacement of alpha boundaries is refused under ADR-0156 and ADR-0225, routed to `montagent-prerender` or the simulation map ([#1083](https://github.com/MBehtemam/Montagent/issues/1083)).

---

## Decisions

### 1. `matte` enters `effects[]` as a standalone alpha modifier

Post-processing an alpha channel is decoupled from key generation. A standalone `matte` effect operates on any element that produces or carries an alpha channel:
- Downstream of `chroma` or `luma_key` to choke green fringes or soften hard key boundaries.
- On pre-keyed RGBA footage (`video` / `image` with straight or premultiplied alpha) to trim compression halos.
- On vector shapes, text elements, or `nest` groups.

```json
{
  "name": "matte",
  "choke": 2.0,
  "feather": 1.5,
  "contrast": 1.2
}
```

#### Parameters and validation

- **`choke: Animatable<f64>`** (optional, defaults to `0.0`):
  - Signed distance in pixels by which the alpha boundary is shifted.
  - `choke > 0.0` erodes (shrinks/chokes) the matte boundary inward by distance $r = \text{choke}$.
  - `choke < 0.0` dilates (spreads/expands) the matte boundary outward by distance $r = |\text{choke}|$.
  - Range: `[-50.0, 50.0]`. Identity is `0.0`.
  - Evaluated via a separable min/max morphological filter or signed distance field sampling on the alpha channel.
- **`feather: Animatable<f64>`** (optional, defaults to `0.0`):
  - Gaussian blur radius $\sigma$ applied to the alpha channel only, softening hard cutouts.
  - Range: `[0.0, 100.0]`. Identity is `0.0`.
  - Evaluated via Skia's separable alpha blur filter (`TileMode::Clamp` within element box).
- **`contrast: Animatable<f64>`** (optional, defaults to `1.0`):
  - Linear contrast scaling around alpha midpoint $0.5$: $\alpha' = \text{clamp}((\alpha - 0.5) \times \text{contrast} + 0.5, 0.0, 1.0)$.
  - Range: `[0.0, 10.0]`. Identity is `1.0`. High values produce hard thresholded binary mattes (*Alpha Levels* / hard matte).

#### Reach and Bounds

- `matte` modifies transparency within the element's declared bounding box.
- While negative `choke` (dilation) expands ink outward, it is clamped to the element's declared box (`TileMode::Clamp`). It does not expand the outer bounding reach beyond the element rect (avoiding unbounded spatial invalidation cascades).

---

### 2. `luma_key` enters `effects[]` for luminance extraction

Luminance keying extracts alpha based on pixel brightness. It unifies Premiere's *Luma Key* and After Effects' *Extract* and *Unmult* (removing solid black or white backings).

```json
{
  "name": "luma_key",
  "threshold": 0.05,
  "tolerance": 0.10,
  "softness": 0.05,
  "invert": false
}
```

#### Parameters and validation

- Evaluated in standard Rec.709 luminance: $Y = 0.2126 R + 0.7152 G + 0.0722 B$ (operating on non-premultiplied RGB).
- **`threshold: Animatable<Fraction>`** (defaults to `0.0`, range `[0.0, 1.0]`):
  - The base luminance level targeted for removal.
- **`tolerance: Animatable<Fraction>`** (defaults to `0.1`, range `[0.0, 1.0]`):
  - Luminance distance $|Y - \text{threshold}| \le \text{tolerance}$ where pixels are fully keyed out ($\alpha = 0$).
- **`softness: Animatable<Fraction>`** (defaults to `0.0`, range `[0.0, 1.0]`):
  - Linear transition ramp width beyond tolerance where alpha ramps smoothly from $0.0$ to $1.0$:
    $$\alpha = \text{clamp}\left(\frac{|Y - \text{threshold}| - \text{tolerance}}{\text{softness}}, 0.0, 1.0\right)$$
- **`invert: bool`** (optional, defaults to `false`):
  - When `true`, keys out bright luminance above threshold rather than dark luminance below (or inverts the resulting matte).
- Implemented as a byte-identical Skia runtime shader (`SkRuntimeEffect`) running parallel-safe across CPU and GPU pipelines.

---

### 3. Iterative multi-pass hole filling and spatial detail reconstruction are refused

- **Refusal of *Matte Choker* multi-stage iterations:**
  - AE's *Matte Choker* performs multiple repeated cycles of dilation followed by erosion to seal pinholes within semi-transparent keys.
  - Performing multiple iterative 2D morphological convolution passes per frame introduces unbounded texture-sampling overhead, tile boundary synchronization seams in parallel Skia chunking, and latency spikes on CPU rendering.
  - Pinholes and noise are mitigated via `chroma.softness`, `matte.feather`, or upstream footage preparation.
- **Refusal of *Key Cleaner*:**
  - *Key Cleaner* attempts edge detail reconstruction across compression artifacts (chroma sub-sampling 4:2:0 boundaries).
  - Spatial edge-directed interpolation kernels are non-separable and non-linear. They are refused from the format schema.
- **Route:** Complex multi-pass hole plugging and compression artifact reconstruction remain on `montagent-prerender`.

---

### 4. Procedural fractal edge erosion (*Roughen Edges*) is refused

- AE's *Roughen Edges* deforms the alpha boundary using continuous 2D/3D Perlin or Simplex fractal noise functions (*Fractal Type*, *Border*, *Roughness*, *Scale*, *Stretch*, *Evolution*).
- Consistent with [ADR-0225](0225-edge-sampling-is-clamp-within-bounds-mosaic-radial-blur-sharpen-find-edges-emboss-and-four-distortions-enter-effects.md) §4 (which refused procedural coordinate distortions such as `turbulent_displace` and `wave_warp`) and [ADR-0156](0156-four-named-effects-join-the-effects-list-and-a-code-drawn-piece-enters-as-pre-rendered-footage.md):
  - Procedural fractal noise shaders with multi-octave loops are computationally heavy and introduce cross-platform floating-point divergence.
  - Procedural edge roughening is refused from the core format.
- **Route:** Users requiring organic ragged edges or dissolving matte borders render animated matte loops or alpha masks via `montagent-prerender` or route to [Map: do particles and simulation effects enter the format, or stay on montagent-prerender?](https://github.com/MBehtemam/Montagent/issues/1083).

---

## Invariants (ADR-0145)

| Invariant | How this decision preserves it |
| :--- | :--- |
| **1. Flat, literal JSON schema** | `matte` and `luma_key` declare explicit, bounded scalar fields (`choke`, `feather`, `contrast`, `threshold`, `tolerance`, `softness`). No complex procedural noise trees or script graphs. |
| **2. Closed vocabulary** | `effects[]` vocabulary expands by exactly two discrete members: `"matte"` and `"luma_key"`. |
| **3. Machine checkable by `validate`** | Parameter bounds ($[0, 1]$ fractions, $[-50, 50]$ choke range, non-negative feather and contrast) are validated statically at document parse time (`E-MATTE-BOUNDS`, `E-LUMA-KEY-BOUNDS`). |
| **4. Editable by exact-string replace** | All numeric parameters are explicit JSON numbers or keyframe arrays editable in place without recalculating adjacent fields. |

---

## Migration and Compatibility

- Existing projects carrying `chroma` are 100% backward compatible: `chroma` schema and behaviour are unchanged.
- `matte` and `luma_key` join `effects[]` without requiring schema bumps or migrations.
- `validate` verifies that `matte` and `luma_key` parameters obey declared ranges across static values and all keyframe records.
