//! ADR-0156's named effects: `posterize`, `glow` and `directional_blur` (#724).
//!
//! Each is a small runtime effect of Montagent's own, with every formula the ADR writes down
//! (§4), so the sample count, the quantiser and the bright-pass cannot change between builds.
//!
//! **Each painter compiles its own.** A compiled [`RuntimeEffect`] is kept per thread, so a
//! render on K painters compiles each SkSL K times and shares nothing Montagent builds
//! between threads (ADR-0156 §6, measured by the prototype
//! [#722](https://github.com/MBehtemam/Montagent/issues/722): about a millisecond per
//! compile). Each paint binds fresh uniforms to it.
//!
//! **An identity value builds no filter.** `levels: 256`, `threshold: 1`, `intensity: 0` and
//! `length: 0` paint a plain layer, which composites in device space: the same bytes as no
//! member, even on a rotated element, where any filter layer is drawn upright and resampled
//! into the rotation.

use std::cell::OnceCell;
use std::thread::LocalKey;

use skia_safe::{
    BlendMode, ColorFilter, Data, ImageFilter, Rect, RuntimeEffect, image_filters,
    runtime_effect::RuntimeShaderBuilder,
};

/// `posterize` (ADR-0156 §4): per channel on non-premultiplied colour,
/// `q = round(v × (levels − 1)) / (levels − 1)`. `v` is never negative, so `floor(x + 0.5)`
/// is rounding to nearest with ties away from zero. Alpha is untouched, so transparent black
/// stays transparent black and the member keeps the bound.
const POSTERIZE: &str = r"
uniform float steps;

half4 main(half4 color) {
    float a = float(color.a);
    if (a <= 0.0) { return color; }
    float3 v = clamp(float3(color.rgb) / a, 0.0, 1.0);
    float3 q = floor(v * steps + 0.5) / steps;
    return half4(half3(q * a), color.a);
}
";

/// `glow`'s bright-pass (ADR-0156 §4): the pixel scaled by
/// `max(0, luma − threshold) / (1 − threshold)`, luma Rec.709 on the non-premultiplied sRGB
/// values. The whole premultiplied pixel is scaled, alpha with it, so the bright part is the
/// element's own colour at a fraction of its opacity; scaling the colour alone would add a
/// dark halo (#722). At `threshold: 1` the pass is empty, so nothing divides by zero.
const BRIGHT_PASS: &str = r"
uniform float threshold;

half4 main(half4 color) {
    float a = float(color.a);
    if (threshold >= 1.0 || a <= 0.0) { return half4(0); }
    float3 v = float3(color.rgb) / a;
    float luma = dot(v, float3(0.2126, 0.7152, 0.0722));
    float f = max(0.0, luma - threshold) / (1.0 - threshold);
    return half4(float4(color) * f);
}
";

/// `glow`'s `intensity`: the gain on the blurred bright part's premultiplied pixel.
const GAIN: &str = r"
uniform float gain;

half4 main(half4 color) {
    return half4(float4(color) * gain);
}
";

/// The most samples [`DIRECTIONAL`] takes: `ceil(length) + 1` with `length` at its schema
/// maximum of 4096.
const MAX_SAMPLES: f64 = 4097.0;

/// `directional_blur` (ADR-0156 §4): `count = ceil(length) + 1` samples, evenly spaced along
/// the line through the pixel and centred on it, read bilinearly and weighted equally.
/// `delta` is the step between two samples, in the element's own space: Skia evaluates a
/// runtime image filter in the space its parameters are given in, so the smear turns with
/// `rotation`, mirrors under a flip and scales with `scale`, as a `blur` radius does.
const DIRECTIONAL: &str = r"
uniform shader source;
uniform float2 delta;
uniform float count;

half4 main(float2 p) {
    float4 sum = float4(0);
    float centre = (count - 1.0) * 0.5;
    for (int i = 0; i < 4097; i++) {
        if (float(i) >= count) { break; }
        sum += float4(source.eval(p + delta * (float(i) - centre)));
    }
    return half4(sum / count);
}
";

thread_local! {
    static POSTERIZE_EFFECT: OnceCell<Option<RuntimeEffect>> = const { OnceCell::new() };
    static BRIGHT_PASS_EFFECT: OnceCell<Option<RuntimeEffect>> = const { OnceCell::new() };
    static GAIN_EFFECT: OnceCell<Option<RuntimeEffect>> = const { OnceCell::new() };
    static DIRECTIONAL_EFFECT: OnceCell<Option<RuntimeEffect>> = const { OnceCell::new() };
    static MATTE_CONTRAST_EFFECT: OnceCell<Option<RuntimeEffect>> = const { OnceCell::new() };
    static LUMA_KEY_EFFECT: OnceCell<Option<RuntimeEffect>> = const { OnceCell::new() };
}

/// `matte`'s `contrast` (ADR-0227 §1): linear contrast scaling around alpha midpoint 0.5,
/// `alpha' = clamp((alpha - 0.5) * contrast + 0.5, 0.0, 1.0)`. Unpremultiplies RGB before
/// scaling alpha so colour saturation is preserved on semi-transparent edges.
const MATTE_CONTRAST: &str = r"
uniform float contrast;

half4 main(half4 color) {
    float a = float(color.a);
    if (a <= 0.0) { return half4(0); }
    float m = clamp((a - 0.5) * contrast + 0.5, 0.0, 1.0);
    float3 rgb = float3(color.rgb) / a;
    return half4(half3(rgb * m), half(m));
}
";

/// `luma_key` (ADR-0227 §2): keys pixels by Rec.709 luminance `Y = 0.2126 R + 0.7152 G + 0.0722 B`
/// on non-premultiplied RGB. Within `|Y - threshold| <= tolerance` the matte is `0`; across
/// `(tolerance, tolerance + softness]` it ramps linearly to `1`; `invert` flips `m` to `1 - m`.
const LUMA_KEY: &str = r"
uniform float threshold;
uniform float tolerance;
uniform float softness;
uniform float invert;

half4 main(half4 color) {
    float a = float(color.a);
    if (a <= 0.0) { return half4(0); }
    float3 rgb = float3(color.rgb) / a;
    float luma = dot(rgb, float3(0.2126, 0.7152, 0.0722));
    float d = abs(luma - threshold);
    float m = softness > 0.0
        ? clamp((d - tolerance) / softness, 0.0, 1.0)
        : (d <= tolerance ? 0.0 : 1.0);
    if (invert > 0.5) {
        m = 1.0 - m;
    }
    float out_a = a * m;
    return half4(half3(rgb * out_a), half(out_a));
}
";

/// One SkSL's compile on this painter thread, made by `make` the first time this thread
/// asks and kept in `cache` after; `None` if this Skia declined it. Every runtime effect of
/// the named effects (`grain`'s blender too) is compiled through this, once per thread.
pub(super) fn compiled(
    cache: &'static LocalKey<OnceCell<Option<RuntimeEffect>>>,
    make: impl FnOnce() -> Option<RuntimeEffect>,
) -> Option<RuntimeEffect> {
    cache.with(|cell| cell.get_or_init(make).clone())
}

/// This thread's compile of a colour-filter SkSL, bound to `uniforms`.
fn colour_filter(
    cache: &'static LocalKey<OnceCell<Option<RuntimeEffect>>>,
    sksl: &str,
    uniforms: &[f32],
) -> Option<ColorFilter> {
    let effect = compiled(cache, || {
        RuntimeEffect::make_for_color_filter(sksl, None).ok()
    })?;
    let bytes: Vec<u8> = uniforms.iter().flat_map(|v| v.to_ne_bytes()).collect();
    effect.make_color_filter(Data::new_copy(&bytes), None)
}

/// `posterize` at `levels` (already a whole number from 2 to 256), or `None` at 256, the
/// identity.
pub(super) fn posterize(levels: f64) -> Option<ImageFilter> {
    if levels >= 256.0 {
        return None;
    }
    let steps = (levels.max(2.0) - 1.0) as f32;
    image_filters::color_filter(
        colour_filter(&POSTERIZE_EFFECT, POSTERIZE, &[steps])?,
        None,
        None,
    )
}

/// `glow`: the source, plus (`Plus`) the gain on the blurred bright-pass of the source, in
/// one filter on the element's own layer. `None` where it adds nothing: an empty bright-pass
/// (`threshold` at 1) or no gain (`intensity` at 0).
pub(super) fn glow(threshold: f64, sigma: f32, intensity: f64) -> Option<ImageFilter> {
    if threshold >= 1.0 || intensity <= 0.0 {
        return None;
    }
    let bright = image_filters::color_filter(
        colour_filter(&BRIGHT_PASS_EFFECT, BRIGHT_PASS, &[threshold as f32])?,
        None,
        None,
    )?;
    let blurred = image_filters::blur((sigma, sigma), None, bright, None)?;
    let boosted = image_filters::color_filter(
        colour_filter(&GAIN_EFFECT, GAIN, &[intensity as f32])?,
        blurred,
        None,
    )?;
    image_filters::blend(BlendMode::Plus, None, boosted, None)
}

/// `matte` (ADR-0227 §1): morphological erode (`choke > 0`) or dilate (`choke < 0`), followed
/// by Gaussian feather (`sigma = feather / 2`) and linear alpha contrast around 0.5, applied
/// to the matte while preserving unblurred interior RGB via `SrcATop`. `None` at the identity
/// (`choke: 0`, `feather: 0`, `contrast: 1`).
pub(super) fn matte(choke: f64, feather: f64, contrast: f64) -> Option<ImageFilter> {
    let has_choke = choke != 0.0;
    let has_feather = feather > 0.0;
    let has_contrast = (contrast - 1.0).abs() > f64::EPSILON;
    if !has_choke && !has_feather && !has_contrast {
        return None;
    }
    let mut matte: Option<ImageFilter> = None;
    if choke > 0.0 {
        let r = choke as f32;
        matte = Some(image_filters::erode((r, r), matte, None)?);
    } else if choke < 0.0 {
        let r = (-choke) as f32;
        matte = Some(image_filters::dilate((r, r), matte, None)?);
    }
    if has_feather {
        let s = super::sigma(feather);
        matte = Some(image_filters::blur((s, s), None, matte, None)?);
    }
    if has_contrast {
        let cf = colour_filter(&MATTE_CONTRAST_EFFECT, MATTE_CONTRAST, &[contrast as f32])?;
        matte = Some(image_filters::color_filter(cf, matte, None)?);
    }
    if has_choke || has_feather {
        image_filters::blend(BlendMode::SrcATop, matte, None, None)
    } else {
        matte
    }
}

/// `luma_key` (ADR-0227 §2): luminance keyer on Rec.709 luma.
pub(super) fn luma_key(
    threshold: f64,
    tolerance: f64,
    softness: f64,
    invert: bool,
) -> Option<ImageFilter> {
    let cf = colour_filter(
        &LUMA_KEY_EFFECT,
        LUMA_KEY,
        &[
            threshold as f32,
            tolerance as f32,
            softness as f32,
            if invert { 1.0 } else { 0.0 },
        ],
    )?;
    image_filters::color_filter(cf, None, None)
}

/// How far a `directional_blur` reaches past what it is given, per axis, in element pixels:
/// ADR-0156's `(|cos θ| × length / 2, |sin θ| × length / 2)`, **rounded up to whole pixels
/// and one more**, because a bilinear read at a fractional offset touches the next pixel
/// (#722 measured 15 touched against a reach of 14.14 at 45°).
pub(super) fn directional_reach(angle: f64, length: f64) -> (f32, f32) {
    let theta = angle.to_radians();
    let half = length.max(0.0) / 2.0;
    (
        (theta.cos().abs() * half).ceil() as f32 + 1.0,
        (theta.sin().abs() * half).ceil() as f32 + 1.0,
    )
}

/// `directional_blur`, uncropped, or `None` at `length: 0`, the identity.
///
/// Skia cannot bound a runtime-shader filter: its output is the whole layer, so
/// [`crop_directional`] must always follow, hint or no hint (#722: about a second per
/// 1080p element without it).
pub(super) fn directional_blur(angle: f64, length: f64) -> Option<ImageFilter> {
    if length <= 0.0 {
        return None;
    }
    let count = (length.ceil() + 1.0).min(MAX_SAMPLES);
    let spacing = length / (count - 1.0);
    let theta = angle.to_radians();
    let delta = [
        (theta.cos() * spacing) as f32,
        (theta.sin() * spacing) as f32,
    ];
    let effect = compiled(&DIRECTIONAL_EFFECT, || {
        RuntimeEffect::make_for_shader(DIRECTIONAL, None).ok()
    })?;
    let mut builder = RuntimeShaderBuilder::new(effect);
    builder.set_uniform_float("delta", &delta).ok()?;
    builder.set_uniform_float("count", &[count as f32]).ok()?;
    let (x, y) = directional_reach(angle, length);
    image_filters::runtime_shader_with_options(&builder, x.max(y), "source", None, false)
}

/// `filter` cropped to `content`, the element-space bounds of what it is given, grown by
/// [`directional_reach`]; and those cropped bounds, which are what it passes on.
pub(super) fn crop_directional(
    filter: ImageFilter,
    content: Rect,
    angle: f64,
    length: f64,
) -> (Option<ImageFilter>, Rect) {
    let cropped = if content.is_empty() {
        Rect::new_empty()
    } else {
        content.with_outset(directional_reach(angle, length))
    };
    (image_filters::crop(cropped, None, filter), cropped)
}
