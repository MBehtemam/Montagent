# Research: how other tools interpolate a keyframed colour, and in which colour space

Research for [#666](https://github.com/MBehtemam/Montagent/issues/666), part of the map [#663](https://github.com/MBehtemam/Montagent/issues/663).

**Date of research:** 2026-10-05. Montagent at `a57c56e1`; `skia-safe` **0.153.2** (Skia milestone **m153**, per `skia-bindings`' `Cargo.toml`); CSS Color 4 Candidate Recommendation Draft of 2026-09-30; Skia, lottie-web and Skottie sources read at their `main`/`master` branches on the research date.

**Sourcing rule applied:** primary sources only: vendor documentation, W3C specifications, the Lottie specification, and the lottie-web, Skottie and Skia sources. Each finding is tagged **[verified]** (read in the primary source) or **[inference]** (reasoned from verified facts). Where a vendor does not document the behaviour, this note says **undocumented** and does not guess.

---

## 1. Answer

| Tool | Space the interpolation runs in | Alpha | Status |
|---|---|---|---|
| Premiere Pro | **Undocumented** | **Undocumented** | Docs say only that keyframes can animate "color changes" |
| After Effects | **Undocumented** for keyframes. Colour values are "interpreted to be in the working color space" | **Undocumented** | Linear-light settings are documented for layer blending, not for keyframed colour values |
| CapCut | **Undocumented** | **Undocumented** | No statement found on capcut.com |
| CSS transitions and animations | **Gamma-encoded sRGB** for legacy colours (hex, named, `rgb()`, `hsl()`, `hwb()`); **Oklab** otherwise | **Premultiplied**: premultiply, interpolate, un-premultiply | Normative, CSS Color 4 §13 |
| Lottie (specification) | **Undocumented**. A colour is a vector of RGB components in 0..1; no colour space is named | Colour has no reliable alpha ("most players ignore the last component"); transparency is a separate scalar opacity property | Spec defines the easing, not the colour maths |
| lottie-web | **Non-premultiplied, component-wise on the raw stored values**, with no colour-space conversion | RGB and opacity interpolate as independent properties | Source |
| Skottie (Skia's Lottie player) | **Non-premultiplied, component-wise float lerp on the raw stored values**, with no colour-space conversion | Fourth component lerped independently if present; no premultiplication | Source |
| Skia itself | No keyframe API. Gradients offer destination space (default), sRGB, linear sRGB, Lab, Oklab, LCH, OkLCH, HSL, HWB and four wide-gamut RGB spaces | `InPremul::kNo` by default, `kYes` on request | Source |

Three facts carry the answer:

1. **Every implementation whose behaviour can be read interpolates the stored sRGB-encoded components directly.** CSS does it for the legacy colour forms, which include hex, and lottie-web and Skottie do it for everything. None of them converts to linear light or a perceptual space by default. Oklab is CSS's default only for colours written in the non-legacy forms.
2. **They split on alpha.** CSS premultiplies before interpolating. The two Lottie players do not, but Lottie keeps transparency in a separate opacity property, so the case where the two methods differ (colour and alpha both changing) is rare in Lottie content.
3. **The three editors in the reference class document nothing.** Premiere, After Effects and CapCut give no statement of the space or the alpha handling for a keyframed colour. Precedent from the reference class cannot settle the ruling; the only written rule is CSS's.

---

## 2. Per-tool detail

### 2.1 Premiere Pro: undocumented

- Adobe's keyframe interpolation page says interpolation "can be used to animate movement, effects, audio levels, image adjustments, transparency, color changes, and many other visual and auditory elements", and defines linear interpolation as "an evenly-paced change from one keyframe to another, with each in-between frame given an equal share of the changed value." It names no colour space and says nothing about alpha. **[verified]** ([Control effect changes using keyframe interpolation](https://helpx.adobe.com/premiere/desktop/add-video-effects/control-effects-and-transitions-using-keyframes/control-effect-changes-using-keyframe-interpolation.html), last updated 2026-01-07)
- The one linear-light control Premiere documents is a compositing setting, not a keyframe setting: "Composite in linear color (linear light) can provide a more photo-realistic look for blended frames, for example, when blending natural images with alpha or feathered masks. This option, in some cases, reduces halos around text or graphics. Linear fades look smoother with this option turned off." **[verified]** ([Sequence settings reference](https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-sequence/sequence-settings-reference.html), last updated 2026-01-07)
- Whether that setting changes how a keyframed colour parameter's value is computed: **undocumented**.

### 2.2 After Effects: undocumented for keyframes

- The keyframe interpolation page says the same as Premiere's: interpolation "can animate movement, effects, audio levels, image adjustments, transparency, color changes", and "After Effects interpolates the values between two adjacent keyframes as directly as possible without accounting for the values of other keyframes." It names no colour space. **[verified]** ([Keyframe interpolation](https://helpx.adobe.com/after-effects/using/keyframe-interpolation.html), last updated 2026-05-05)
- Colour values live in the project's working space: "colors chosen in the color picker will change when you switch to a linear working color space because colors inside After Effects are interpreted to be in the working color space." **[verified]** ([Color management](https://helpx.adobe.com/after-effects/using/color-management.html), last updated 2026-03-27)
- The linear-blending switch is scoped to layers: "To blend colors in a linear color space, choose Blend colors using 1.0 gamma. This option affects only blending between layers." **[verified]** (same page)
- The expression language treats a colour as "an Array of normalized red, green, blue, and alpha channel values, all in the range of 0.0 to 1.0", with `rgbToHsl` and `hslToRgb` as explicit conversions. **[verified]** ([Expression language reference](https://helpx.adobe.com/after-effects/using/expression-language-reference.html))
- **[inference]** A keyframed colour is most likely interpolated component-wise on its working-space RGBA values, so in a default (non-linearised) project that is gamma-encoded RGB. This fits the three statements above and the fact that Lottie, which is an export of After Effects animation data, stores colours as plain RGB vectors that both players lerp directly (§2.5). Adobe does not state it, so the table records **undocumented**.

### 2.3 CapCut: undocumented

- CapCut's own keyframe guide lists position, scale, rotation, opacity, effects and audio as keyframable, and says effect keyframes "animate specific visual effects applied to your clips, such as color, brightness, or filters". It contains nothing on how values are computed between keyframes. **[verified, through a fetched-page summary rather than a raw read]** ([How to Add Keyframes in CapCut PC](https://www.capcut.com/resource/how-to-add-keyframes-in-capcut))
- No capcut.com page found states a colour space or an alpha rule for keyframed colour. **Undocumented.**

### 2.4 CSS transitions and animations: sRGB for legacy colours, premultiplied

All section numbers are from the [CSS Color 4 CRD of 2026-09-30](https://www.w3.org/TR/2026/CRD-css-color-4-20260930/).

- **The chain from "animation" to the colour maths.** The `color` property's "Animation type" is "by computed value type" ([§3.2](https://www.w3.org/TR/css-color-4/#the-color-property)). Web Animations defines that as: "Corresponding individual components of the computed values are combined (interpolated, added, or accumulated) using the indicated procedure for that value type" ([Web Animations, "by computed value"](https://www.w3.org/TR/web-animations-1/#by-computed-value)). For colours the procedure is CSS Color 4 §13, which lists "transitions" and "animations" among its uses. **[verified]**
- **The steps.** "(if required) converting them both to a given color space which will be referred to as the interpolation color space … changing the color components to premultiplied form; linearly interpolating each component of the computed value of the color separately; undoing premultiplication". **[verified]** ([§13](https://www.w3.org/TR/css-color-4/#interpolation))
- **The space.** "If the host syntax does not define what color space interpolation should take place in, it defaults to Oklab." Then: "However, user agents must handle interpolation between legacy sRGB color formats (hex colors, named colors, `rgb()`, `hsl()` or `hwb()` and the equivalent alpha-including forms) in gamma-encoded sRGB space. This provides Web compatibility; legacy sRGB content interpolates in the sRGB space by default." **[verified]** ([§13.2](https://www.w3.org/TR/css-color-4/#interpolation-space))
- **The spec's own verdict on sRGB.** "The sRGB color space, which is neither linear-light nor perceptually uniform, is the choice here, even though it produces poorer results (overly dark or greyish mixes)." It names the alternatives: linear-light spaces for "physically mixing two colored lights", Oklab for colours "evenly spaced perceptually", and OkLCh for "avoiding graying out". **[verified]** (§13.2)
- **Alpha.** "When the colors to be interpolated are not fully opaque, they are first premultiplied". For rectangular spaces "all component values are multiplied by the alpha value"; afterwards "each component which had been premultiplied is divided by the interpolated alpha value." **[verified]** ([§13.4](https://www.w3.org/TR/css-color-4/#interpolation-alpha))
- **When premultiplication matters.** "transitions where either the transparency or the color are held constant … have identical results whether the color interpolation is done in premultiplied or non-premultiplied color-space. Differences only arise when both the color and transparency differ between the two endpoints." **[verified]** (§13.4)
- **Worked example from the spec.** `rgb(24% 12% 98% / 0.4)` to `rgb(62% 26% 64% / 0.6)` in sRGB: the midpoint is `rgb(46.8% 20.4% 77.6% / 0.5)`. **[verified]** (§13.4)
- A Montagent colour is a hex colour, so under CSS's rule it falls in the legacy group: gamma-encoded sRGB, premultiplied. **[inference]**
- The CSS Transitions 1 Working Draft of 2026-01-08 and CSS Animations 1 carry no colour maths of their own; they defer to the chain above. **[verified]** ([CSS Transitions 1](https://www.w3.org/TR/css-transitions-1/), [CSS Animations 1](https://www.w3.org/TR/css-animations-1/))

### 2.5 Lottie: raw component lerp, non-premultiplied

**The specification.**

- "Colors are Vectors with values between 0 and 1 for the RGB components." and "sometimes you might find color values with 4 components (the 4th being alpha) but most players ignore the last component." **[verified]** ([Lottie spec, Values: Color](https://lottie.github.io/lottie-spec/latest/specs/values/#color))
- Keyframe easing is defined per dimension: "The y axis represents the value interpolation factor, a value of 0 represents the value at the current keyframe, a value of 1 represents the value at the next keyframe." For vector properties the handles are arrays "so you can have different easing curves per dimension", and y values may overshoot. **[verified]** ([Lottie spec, Properties: Keyframe Easing](https://lottie.github.io/lottie-spec/latest/specs/properties/#easing-handle))
- The spec names no colour space for the components and no colour-specific interpolation rule. **Undocumented** at the specification level; the behaviour below comes from the two players' sources.

**lottie-web.**

- A colour keyframe is interpolated by the generic multidimensional path, one component at a time: `keyValue = keyData.h === 1 ? keyData.s[i] : keyData.s[i] + (endValue[i] - keyData.s[i]) * perc;`. There is no colour-space conversion and no premultiplication in the file. **[verified]** ([`player/js/utils/PropertyFactory.js`](https://github.com/airbnb/lottie-web/blob/master/player/js/utils/PropertyFactory.js))
- The result is written out as `rgb(r,g,b)` with the opacity as a separate `fill-opacity` or `stroke-opacity` attribute taken from the separate `o` property. **[verified]** ([`SVGElementsRenderer.js`](https://github.com/airbnb/lottie-web/blob/master/player/js/elements/helpers/shapes/SVGElementsRenderer.js))

**Skottie.**

- A colour is a float vector (`ColorValue` derives from `VectorValue`, a `std::vector<float>`). **[verified]** ([`SkottieValue.h`](https://github.com/google/skia/blob/main/modules/skottie/src/SkottieValue.h))
- Keyframes are interpolated with a plain per-component `Lerp(v0, v1, lerp_info.weight)` over the stored floats. **[verified]** ([`VectorKeyframeAnimator.cpp`](https://github.com/google/skia/blob/main/modules/skottie/src/animator/VectorKeyframeAnimator.cpp))
- The vector becomes a colour only afterwards: each of r, g, b is pinned to 0..1, and alpha is the fourth component if present, else 1. The result is an unpremultiplied `SkColor4f`. **[verified]** (same file, `ColorValue::operator SkColor4f`)
- So Skottie interpolates non-premultiplied, in whatever encoding the file's numbers are in, and Skia's own Lottie player does not use any of Skia's colour-space machinery to animate a colour. **[verified]**

---

## 3. What Skia offers

### 3.1 There is no keyframe API

Skia core has no notion of time. A flat colour that changes over time is a different `SkPaint` colour on each frame, computed by the caller. Skottie is Skia's precedent for doing that, and it uses a float lerp (§2.5). **[verified]**

The building blocks Skia gives the caller:

- `SkColor4f` is four unpremultiplied floats: "Skia's public API always uses unpremultiplied colors". `premul()` and `unpremul()` convert between the two forms. **[verified]** ([`SkColor.h`](https://github.com/google/skia/blob/main/include/core/SkColor.h))
- `SkPaint::setColor(const SkColor4f&, SkColorSpace*)` accepts a colour tagged with a colour space, and `SkColorSpace::MakeSRGB()` and `MakeSRGBLinear()` provide the two sRGB encodings. **[verified]** ([`SkPaint.h`](https://github.com/google/skia/blob/main/include/core/SkPaint.h), [`SkColorSpace.h`](https://github.com/google/skia/blob/main/include/core/SkColorSpace.h))
- Skia exposes no public function that converts a single colour to or from Oklab. The conversion exists only inside the gradient shader. **[verified for the headers read above; inference for the rest of the public API]**

### 3.2 Gradients do have an interpolation setting

`SkGradient::Interpolation` controls how colours mix across space inside one gradient. The task brief names it `SkGradientShader::Interpolation`; on Skia `main` and in m153 the struct lives in `SkGradient`, and `include/effects/SkGradientShader.h` no longer exists on `main`. **[verified]** ([`SkGradient.h`](https://github.com/google/skia/blob/main/include/effects/SkGradient.h))

| Field | Values | Default |
|---|---|---|
| `fInPremul` | `kNo`, `kYes` | `kNo` |
| `fColorSpace` | `kDestination`, `kSRGBLinear`, `kLab`, `kOKLab`, `kOKLabGamutMap`, `kLCH`, `kOKLCH`, `kOKLCHGamutMap`, `kSRGB`, `kHSL`, `kHWB`, `kDisplayP3`, `kRec2020`, `kProphotoRGB`, `kA98RGB` | `kDestination` |
| `fHueMethod` | `kShorter`, `kLonger`, `kIncreasing`, `kDecreasing` | `kShorter` |

- `kDestination` is documented as "Default Skia behavior: interpolate in the color space of the destination surface". The other values cite CSS Color 4's interpolation section. **[verified]**
- `kOKLabGamutMap` and `kOKLCHGamutMap` carry the warning "This space is experimental and should not be used in production." **[verified]**
- Gradient stop colours are "treated as sRGB" when no colour space is passed. **[verified]**
- With premultiplied interpolation in a polar space, hue is not premultiplied, matching CSS. **[verified]** ([`SkGradientBaseShader.cpp`](https://github.com/google/skia/blob/main/src/shaders/gradients/SkGradientBaseShader.cpp))
- `skia-safe` 0.153.2 exposes all of this as `gradient::Interpolation { in_premul, color_space, hue_method }` with the same defaults. **[verified]** (`src/effects/gradient.rs` in the crate)
- Montagent's raster surface is `RGBA8888`, premultiplied, with no colour space (`crates/montagent-render/src/canvas.rs`). **[verified]** With no destination colour space, `kDestination` performs no conversion, so a default Skia gradient on that surface mixes the gamma-encoded sRGB values, non-premultiplied. **[inference]**

### 3.3 What that means per option

| Option | For a gradient | For a keyframed flat colour |
|---|---|---|
| Non-premultiplied sRGB | Skia's default (`kDestination` or `kSRGB`, `InPremul::kNo`) | Lerp the four components of the two `SkColor4f` values. This is what Skottie does |
| Premultiplied sRGB | `InPremul::kYes` | `premul()`, lerp, `unpremul()`. This is CSS's rule for hex colours |
| Linear light | `kSRGBLinear` | Caller applies the sRGB transfer function to each channel, lerps, and encodes back. Skia has the colour spaces but no one-call helper for a single colour |
| Perceptual (Oklab, OkLCH) | `kOKLab`, `kOKLCH` | Caller implements the Oklab conversion. Skia has no public single-colour Oklab function |

**[inference]** A gradient whose stops are keyframed involves both columns at once: the stops move over time by the keyframe rule, and the colours between stops mix by the gradient's `Interpolation`. The two can be set independently, so the gradients ticket can either tie them together or leave them separate.

**[inference]** Each per-frame colour is a pure function of the two keyframe values and the frame's progress, so any of the four options keeps output byte-identical across parallel painters. Linear light needs a power function and Oklab needs a cube root; neither is guaranteed bit-identical across platforms' maths libraries, which matters only if frames must also match across machines.

---

## 4. Visible consequences

The midpoints below were computed for this note from the sRGB transfer function and the published Oklab matrices, then rounded to 8 bits. **[inference: computed, not quoted]** The qualitative claims are CSS Color 4's (§2.4).

| From | To | sRGB midpoint | Linear-light midpoint | Oklab midpoint |
|---|---|---|---|---|
| `#FF0000` red | `#00FFFF` cyan | `#808080` mid grey | `#BCBCBC` light grey | `#D2A993` light tan |
| `#FF0000` red | `#00FF00` green | `#808000` dark olive | `#BCBC00` bright yellow-green | `#D0A800` gold |
| `#0000FF` blue | `#FFFF00` yellow | `#808080` mid grey | `#BCBCBC` light grey | `#6CABC7` light blue |
| `#000000` black | `#FFFFFF` white | `#808080` | `#BCBCBC` | `#636363` |

- **sRGB (gamma-encoded).** The halfway colour between two saturated colours is darker than either end, and between complementary colours it is a neutral grey. CSS Color 4 calls these "overly dark or greyish mixes". It is also what every readable implementation does for hex colours, so it is what an author who knows CSS or Lottie expects.
- **Linear light.** The dip in brightness goes away, since this is how light mixes physically. Complementary colours still pass through grey, only a lighter one. A black-to-white ramp spends most of its time looking bright (`#BCBCBC` at the midpoint), so fades to and from black look uneven.
- **Oklab.** Brightness changes evenly and the midpoint keeps some colour, so there is less greying. The path can pass through a hue that is in neither endpoint (red to cyan passes through tan). In-between values can fall outside sRGB and must be clipped or gamut-mapped; Skia marks its gamut-mapping variants experimental.
- **OkLCH and other polar spaces.** The midpoint stays saturated because the hue rotates. The colour travels around the hue wheel, so red to cyan passes through yellow and green or through magenta and blue, depending on the hue method. A fourth setting (the hue method) becomes part of the rule.

**Alpha.** The two methods differ only when colour and alpha both change (CSS Color 4 §13.4).

| From | To | Non-premultiplied midpoint | Premultiplied midpoint |
|---|---|---|---|
| `#FF0000FF` opaque red | `#0000FF00` transparent blue | `#80008080` half-transparent purple | `#FF000080` half-transparent red |
| `#FFFFFFFF` opaque white | `#00000000` transparent black | `#80808080` half-transparent grey | `#FFFFFF80` half-transparent white |

- **Non-premultiplied.** The colour of a fully transparent keyframe leaks into the in-between frames. A fade to `#00000000` passes through grey, which shows as a dark fringe or a dirty fade. CSS Color 4 describes the same effect in a gradient: "the center of the gradient would be a noticeably grayish color, because 'transparent' is actually a shorthand for rgba(0,0,0,0)".
- **Premultiplied.** A transparent keyframe contributes no colour, so fading out keeps the visible colour and only lowers the alpha. The RGB written on a fully transparent keyframe has no effect on any frame.
- When alpha is the same at both keyframes, or the colour is, both methods give the same result.

---

## 5. Not sourced

- **Premiere Pro, After Effects, CapCut:** no primary source states the colour space or the alpha handling of a keyframed colour. Recorded as undocumented in §1 and §2. Measuring the applications directly (keyframe red to cyan, read the midpoint pixel) would settle it; that was outside a documentation-only ticket.
- **Adobe pages were read from Internet Archive captures.** helpx.adobe.com returned HTTP 403 to direct fetches. The text and "Last updated" dates quoted are from the archived copies of the same URLs.
- **CapCut's page was read through a fetched-page summary**, not a raw read. A second fetch service returned HTTP 429 and was not retried.
- **CSS Values 4 §3** ("Combining Values") was not read; the chain in §2.4 rests on CSS Color 4 and Web Animations.
- **Browser behaviour was not tested.** §2.4 states what the specification requires, not what each browser ships.
