---
status: accepted
amends: 0014 (confirms stroke stays a geometry paint property on vector elements; outline enters effects[] for alpha-contour borders across all visual elements), 0040 (expands closed effects vocabulary with outline, color_overlay, and inner_shadow), 0049 (distinguishes flat silhouette color_overlay from perceptual tint; confirms refusal of multi-stop gradient_overlay from effects[]), 0156 (reaffirms refusal of procedural/3D surface generators bevel and satin from runtime effects in favour of montagent-prerender), 0215 (disposes of After Effects Layer Styles census rows from map #896)
---

# Outline, color_overlay, and inner_shadow enter effects[], and bevel and gradient_overlay are refused

[Which layer styles (alpha outline/stroke, colour/gradient overlay, inner shadow, bevel) enter on raster and vector elements?](https://github.com/MBehtemam/Montagent/issues/1073)
on the After Effects map ([#896](https://github.com/MBehtemam/Montagent/issues/896)).

[ADR-0014](0014-stroke-is-paint-the-text-box-is-required.md) established that `stroke_color` and `stroke_width` are paint properties on geometric vector paths and text boxes, not general raster modifiers.
[ADR-0040](0040-effect-model-attachment-and-v1-vocabulary.md) established the ordered `effects[]` pipeline and admitted drop `shadow` and `blur`.
[ADR-0049](0049-v1-colour-filter-vocabulary-four-scalar-members.md) established the colour-filter vocabulary with `tint`, `saturation`, `brightness`, and `contrast` under a mechanical stopping rule forbidding arbitrary open-ended families.
[ADR-0146](0146-an-animatable-property-is-one-the-schema-types-so-colour-blends-premultiplied-and-spring-easing-is-refused.md) established that all numeric and colour parameters in `effects[]` are animatable in place as keyframe lists.
[ADR-0156](0156-four-named-effects-join-the-effects-list-and-a-code-drawn-piece-enters-as-pre-rendered-footage.md) admitted `glow` and established the principle that complex multi-pass procedural generators enter via pre-rendered footage on `montagent-prerender`.
[ADR-0215](0215-after-effects-is-precedent-for-rendered-results-and-capcut-and-premiere-stay-first-for-shape.md) admitted After Effects precedent for rendered results while keeping Premiere and CapCut first for mechanism and shape.
[ADR-0225](0225-edge-sampling-is-clamp-within-bounds-mosaic-radial-blur-sharpen-find-edges-emboss-and-four-distortions-enter-effects.md) admitted directional `emboss` as a 3×3 convolution kernel in `effects[]`.
[ADR-0227](0227-standalone-matte-and-luma-key-enter-effects-and-iterative-chokers-and-roughen-edges-are-refused.md) admitted standalone `matte` for alpha morphological choke and feathering.

The After Effects censuses ([#898](https://github.com/MBehtemam/Montagent/issues/898) rows 113–122, [#905](https://github.com/MBehtemam/Montagent/issues/905) row 20) compared After Effects' nine Layer Styles against Montagent's element fields and `effects[]`:
- *Drop Shadow* and *Outer Glow* are already covered by `shadow` (ADR-0040) and `glow` (ADR-0156).
- *Stroke*, *Color Overlay*, and *Gradient Overlay* exist for vector elements (`rect`, `path`, `text`) via `stroke_color`, `fill`, and `gradient`, but have no equivalent for transparent raster cutouts (`image`, `video`) or `nest` elements (e.g. sticker borders or silhouette badges).
- *Inner Shadow*, *Inner Glow*, *Bevel and Emboss*, and *Satin* had no equivalent on any element type.

This ADR decides:
1. **`outline` enters `effects[]` as an alpha-silhouette contour**: Enables external sticker outlines across all visual elements (`image`, `video`, `nest`, `text`, `rect`, `ellipse`, `path`) via morphological dilation without mutating element geometry schemas.
2. **`color_overlay` enters `effects[]` as a flat silhouette tint, while `gradient_overlay` is refused**: Flat silhouette replacement enters `effects[]`, while multi-stop gradient overlays are refused from `effects[]` to preserve a clean, flat parameter model.
3. **`inner_shadow` enters `effects[]`, while `inner_glow`, `bevel`, and `satin` are refused**: Inward alpha shadow enters `effects[]` matching `shadow`'s parameter structure; `inner_glow` is refused as redundant, while heavy 3D surface modeling (*Bevel and Emboss*) and procedural sheens (*Satin*) are refused from runtime rendering and remain on `montagent-prerender`.

---

## Decisions

### 1. `outline` enters `effects[]` for alpha-contour borders

Sticker borders, white cutout outlines, and badge highlights around non-geometric elements (`image`, `video`, `nest`, and styled text) enter `effects[]` as `outline`:

```json
{
  "name": "outline",
  "width": 6.0,
  "color": "#FFFFFF",
  "opacity": 1.0
}
```

#### Parameters and validation
- **`width: Animatable<f64>`** (required):
  - Thickness of the dilated contour in pixels.
  - Must satisfy $\text{width} \ge 0.0$. Identity value is `0.0`.
  - Evaluated via Skia morphological dilation (`SkImageFilter::Dilate`) of the element's alpha channel, subtracting the source alpha and drawing behind the original element with `color`.
- **`color: Animatable<Colour>`** (required):
  - Border stroke colour (`#RRGGBB` or `#RRGGBBAA`).
- **`opacity: Animatable<f64>`** (optional, defaults to `1.0`):
  - Alpha multiplier in $[0.0, 1.0]$. Identity value is `1.0`.

#### Relation to `stroke` (ADR-0014)
Naming this effect `outline` avoids overloading `stroke_color` and `stroke_width`. Per ADR-0014, `stroke` remains an intrinsic geometric paint attribute on vector shapes and text. `outline` operates post-rasterisation on the element's alpha silhouette, ensuring consistent sticker borders even on feathered video cutouts, transparent PNGs, and nested compositions.

---

### 2. `color_overlay` enters `effects[]`; `gradient_overlay` is refused

#### `color_overlay` enters `effects[]`
After Effects' *Color Overlay* allows tinting or completely replacing an element's interior pixels with a flat colour while preserving its alpha silhouette:

```json
{
  "name": "color_overlay",
  "color": "#FF2040",
  "opacity": 1.0
}
```

- **`color: Animatable<Colour>`** (required): Target fill colour.
- **`opacity: Animatable<f64>`** (optional, defaults to `1.0`): Blend factor in $[0.0, 1.0]$. At `opacity: 1.0`, all non-zero alpha pixels have their RGB replaced with `color` (evaluated via `SkBlendMode::SrcIn` or lerping RGB toward `color`).
- **Distinction from `tint` (ADR-0049)**: `tint` blends existing pixel luminance and colour toward a hue. `color_overlay` replaces RGB uniformly, enabling solid silhouette graphics, pop-art fills, and duotone cutouts from transparent rasters.

#### `gradient_overlay` is refused from `effects[]`
`gradient_overlay` would require embedding multi-stop `Gradient` structures (linear, radial, sweep, two-point conical) into the `effects[]` array. Admitting full gradient structures inside `effects[]` breaks the flat parameter property of the visual effects pipeline and duplicates existing vector gradient capabilities. Gradient looks on rasters are achieved cleanly via `type: "nest"` compositing or `montagent-prerender`.

---

### 3. `inner_shadow` enters `effects[]`; `inner_glow`, `bevel`, and `satin` are refused

#### `inner_shadow` enters `effects[]`
*Inner Shadow* casts an inset directional shadow within the opaque boundaries of an element, widely used for cutout depth, inset card UI, and debossed text:

```json
{
  "name": "inner_shadow",
  "dx": 2.0,
  "dy": 4.0,
  "radius": 6.0,
  "color": "#000000",
  "opacity": 0.6
}
```

- Exactly mirrors `shadow`'s parameter signature ([ADR-0040](0040-effect-model-attachment-and-v1-vocabulary.md)):
  - `dx`, `dy`: directional pixel offsets (`Animatable<f64>`).
  - `radius`: Gaussian blur radius (`Animatable<f64>`, $\ge 0.0$).
  - `color`: shadow colour (`Animatable<Colour>`).
  - `opacity`: shadow opacity (`Animatable<f64>`, $[0.0, 1.0]$).
- **Evaluation**: The element's alpha channel is inverted, offset by `(dx, dy)`, blurred by `radius`, clipped strictly to the element's interior alpha mask, and painted over the element's pixels using standard alpha compositing.

#### Refusal of `inner_glow`, `bevel`, and `satin`
- **`inner_glow` is refused**: Functionally identical to `inner_shadow` with `dx: 0.0, dy: 0.0`. Admitting a separate effect violates ADR-0049's non-composability clause.
- **`bevel` (*Bevel and Emboss*) is refused**: Full 3D beveling requires multi-pass surface normal estimation, light angle computation, highlight/shadow split blend modes, and specular gloss contours. Runtime directional embossing is already provided by `emboss` ([ADR-0225](0225-edge-sampling-is-clamp-within-bounds-mosaic-radial-blur-sharpen-find-edges-emboss-and-four-distortions-enter-effects.md)); full 3D chiseled/pillow bevels belong to asset creation in `montagent-prerender`.
- **`satin` is refused**: Procedural wave-based interior sheen has negligible adoption in modern video design and fails the entry threshold under ADR-0145.

---

## Consequences

- Resolves census rows 113–122 of [#898](https://github.com/MBehtemam/Montagent/issues/898) and row 20 of [#905](https://github.com/MBehtemam/Montagent/issues/905).
- `outline`, `color_overlay`, and `inner_shadow` join the closed `VisualEffect` enum and JSON schema in `schema/montagent.schema.json`.
- Raster cutout assets (`image` with alpha, pre-keyed `video`, and `nest` groups) gain first-class sticker border and silhouette fill capabilities without requiring pre-composition workarounds.
- Bounding boxes: Like `shadow` and `glow`, `outline` may paint pixels outside the element's declared layout box, but does not alter the semantic `width` and `height` coordinates for layout measurement or query inspection.
