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
}

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
