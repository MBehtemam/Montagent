---
status: accepted
amends: 0049 (admits vibrance, hue_rotate, and invert to effects[] as bounded scalar effects with identity 0.0; confirms exposure and 3-way color balance are covered by cdl under ADR-0206; reaffirms refusal of duotone/palette remaps and posterize/threshold), 0147 (admits 12 standard SkBlendMode/W3C modes to blend: darken, lighten, color_dodge, color_burn, hard_light, soft_light, difference, exclusion, hue, saturation, color, luminosity; refuses legacy, stochastic, and non-standard modes), 0215 (disposes of the Color Correction and Blending Modes census rows from the After Effects census on map #896)
---

# Vibrance, hue_rotate, and invert enter effects[], and twelve SkBlendMode variants enter blend

[Which scalar colour-filter additions (ADR-0049) and held-back blend modes (ADR-0147) enter?](https://github.com/MBehtemam/Montagent/issues/1063)
on the After Effects map ([#896](https://github.com/MBehtemam/Montagent/issues/896)).

[ADR-0049](0049-v1-colour-filter-vocabulary-four-scalar-members.md) established the colour-filter vocabulary
with four scalar effects (`tint`, `saturation`, `brightness`, `contrast`) under a mechanical stopping rule:
(a) parameter count fixed by name, (b) bounded scalar parameters, (c) documented identity value, and
(d) non-reproducibility by composing admitted members. [ADR-0206](0206-asc-cdl-and-3d-lut-tetrahedral-shaders-grade-colour-under-closed-arithmetic.md)
subsequently admitted ASC CDL (`cdl`) and 3D LUT (`lut3d`) under closed arithmetic.

[ADR-0147](0147-a-blend-mode-is-a-flat-static-field-of-five-values-and-the-finished-element-blends-last.md)
admitted five blend modes (`normal`, `multiply`, `screen`, `overlay`, `add`) into the static `blend` field,
holding back the remaining W3C / Skia blend modes "for now" until forcing cases emerged.

The After Effects colour census ([#904](https://github.com/MBehtemam/Montagent/issues/904), rows 16–47, 79) and
compositing census ([#898](https://github.com/MBehtemam/Montagent/issues/898), rows 67–102), backed by
Premiere Pro reference parity ([ADR-0215](0215-after-effects-is-precedent-for-rendered-results-and-capcut-and-premiere-stay-first-for-shape.md)),
identified two closely related sets of compositing and grading gaps that were deferred rather than refused on invariants.

This ADR decides:
1. Which scalar colour effects enter `effects[]`, and which are refused as redundant or out-of-scope.
2. Which held-back `SkBlendMode` variants enter `blend`, and which specialty/legacy modes are refused.
3. The parameter bounds, schemas, identity values, and byte-identical evaluation rules.

---

## Decisions

### 1. `vibrance`, `hue_rotate`, and `invert` enter `effects[]`

Three scalar colour effects join the closed `VisualEffect` discriminated union in `effects[]`:

```json
{"name": "vibrance", "amount": 0.25}
{"name": "hue_rotate", "angle": 30.0}
{"name": "invert", "amount": 1.0}
```

- **`vibrance{amount}`**:
  - `amount`: float in `[-1.0, 1.0]`. Identity value: `0.0`.
  - Non-linearly scales saturation, boosting lower-saturated pixels more aggressively while protecting skin tones and already-saturated pixels. Cannot be composed from `saturation`.
- **`hue_rotate{angle}`**:
  - `angle`: float in degrees in `[-180.0, 180.0]`. Identity value: `0.0`.
  - Rotates hue angle around the color circle in RGB/HSV space. Cannot be composed from existing filters.
- **`invert{amount}`**:
  - `amount`: float in `[0.0, 1.0]`. Identity value: `0.0`.
  - Linearly interpolates between the original pixel and its RGB inversion ($1.0 - c$). Cannot be composed because `contrast` clamps at 0.0 and `difference` was held back from element-local filter pipelines.

All three satisfy ADR-0049's mechanical stopping rule: fixed arity, bounded scalar parameters, documented identity values, and non-composability.

### 2. Separate `exposure` and `color_balance` are refused as redundant with `cdl` (ADR-0206)

- **`exposure` is refused from `effects[]`**:
  Exposure adjustment in photographic f-stops ($2^{\text{exposure}}$) is a pure scalar gain. Under [ADR-0206](0206-asc-cdl-and-3d-lut-tetrahedral-shaders-grade-colour-under-closed-arithmetic.md), `cdl` with `slope: [s, s, s]` (where $s = 2^{\text{exposure}}$) achieves exact linear-gain exposure. Introducing a separate `exposure` effect violates ADR-0049 clause (d) by duplicating an admitted grading mechanism.
- **`color_balance` is refused from `effects[]`**:
  Photoshop/AE 9-slider `color_balance` (shadows, midtones, highlights for R, G, B) is directly covered by ASC CDL's standardized three-way grading:
  - `offset` controls shadows/lift;
  - `power` controls midtones/gamma;
  - `slope` controls highlights/gain.
  Admitting a separate ad-hoc 9-parameter color balance effect duplicates the standardized `cdl` format without adding capabilities.

### 3. Two-colour `tint` remaps, `posterize`, and `threshold` are refused

- **AE two-colour `tint` remap (Map Black To / Map White To) is refused**:
  ADR-0049 explicitly states: *"duotone-style two-colour maps are categorically excluded... `tint.color` is the sole grandfathered exception to 'parameters must be bounded scalars.'... This exception is closed: a future colour-op proposal does not get the same allowance by citing `tint` as precedent."* Duotone and tritone palette remapping remain refused under ADR-0049.
- **`posterize` and `threshold` are refused**:
  Posterization (quantizing channel values into discrete steps) and thresholding (binarizing) are stylization/posterization transforms without a smooth identity continuum. They do not belong in the continuous grading vocabulary and remain reachable via pre-render (`montagent-prerender`).

### 4. Twelve standard W3C / Skia blend modes enter `blend`

The `blend` field on visual elements (`rect`, `ellipse`, `path`, `text`, `image`, `video`, `nest`) expands from 5 to 17 modes:

| Category | Modes | Skia Equivalent (`SkBlendMode`) |
| --- | --- | --- |
| **Existing** | `normal`, `multiply`, `screen`, `overlay`, `add` | `SrcOver`, `Multiply`, `Screen`, `Overlay`, `Plus` |
| **Subtractive / Additive** | `darken`, `lighten`, `color_dodge`, `color_burn` | `Darken`, `Lighten`, `ColorDodge`, `ColorBurn` |
| **Contrast** | `hard_light`, `soft_light` | `HardLight`, `SoftLight` |
| **Inversion / Cancellation** | `difference`, `exclusion` | `Difference`, `Exclusion` |
| **Non-separable HSL** | `hue`, `saturation`, `color`, `luminosity` | `Hue`, `Saturation`, `Color`, `Luminosity` |

- All 12 admitted modes are standard W3C Compositing and Blending Level 1 / PDF modes implemented natively by `SkBlendMode`.
- **Stored sRGB arithmetic**: In accordance with ADR-0147 §2, all blend arithmetic runs strictly on stored sRGB values without linearization.
- **Order & isolation**: The blend mode remains the final step of an element's paint, compositing the finished element into its backdrop. Group isolation is not introduced; backdrop is everything lower in the stack.
- **Review finding**: `R-BLEND-BACKGROUND-ONLY` continues to fire when any non-`normal` element has no lower element beneath its bounding box.

### 5. Legacy, stochastic, and non-standard blend modes are refused

The following After Effects / Photoshop blend modes are refused:
- **`dissolve` and `dancing_dissolve`**: Stochastic dither patterns violate deterministic byte-identical rendering across parallel tile and chunk painters.
- **`classic_color_burn`, `classic_color_dodge`, `classic_difference`**: Obsolete AE 5.0 legacy algorithms kept solely for backwards compatibility with 2001 projects.
- **`linear_light`, `vivid_light`, `pin_light`, `hard_mix`**: Extreme contrast hybrid modes lacking forcing cases in editing workflows.
- **`alpha_add` and `luminescent_premul`**: Depend on group compositing semantics or unmultiplied HDR buffers that do not exist in Montagent's flat 8-bit sRGB frame model.
- **`subtract` and `divide`**: Unclamped arithmetic modes outside the standard W3C non-separable/separable specification.

---

## Consequences

- `crates/montagent-core/src/model/mod.rs` and `schema/montagent.schema.json`:
  - `Effect` union gains:
    - `{"name": "vibrance", "amount": f64}`
    - `{"name": "hue_rotate", "angle": f64}`
    - `{"name": "invert", "amount": f64}`
  - `blend` enum expands to include `darken`, `lighten`, `color_dodge`, `color_burn`, `hard_light`, `soft_light`, `difference`, `exclusion`, `hue`, `saturation`, `color`, `luminosity`.
- `crates/montagent-render/src/canvas.rs`:
  - `vibrance`, `hue_rotate`, and `invert` are evaluated in the element effect pipeline via Skia runtime shaders / color filters.
  - `apply_blend_mode` maps all 17 string values directly to their respective `SkBlendMode` variants.
- Gating tests verify byte-identical output across chunk and painter configurations for all 17 blend modes and 3 new scalar filters.
