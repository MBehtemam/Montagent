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

/// `vibrance` (ADR-0221): chroma-weighted saturation boost/cut on non-premultiplied sRGB
/// colour, with Rec. 709 luma `[0.213, 0.715, 0.072]` and pixel chroma `max(r,g,b) - min(r,g,b)`.
const VIBRANCE: &str = r"
uniform float amount;

half4 main(half4 color) {
    float a = float(color.a);
    if (a <= 0.0) { return color; }
    float3 v = clamp(float3(color.rgb) / a, 0.0, 1.0);
    float mx = max(v.r, max(v.g, v.b));
    float mn = min(v.r, min(v.g, v.b));
    float sat = mx - mn;
    float luma = dot(v, float3(0.213, 0.715, 0.072));
    float scale = max(0.0, 1.0 + amount * (1.0 - sat));
    float3 q = clamp(luma + (v - luma) * scale, 0.0, 1.0);
    return half4(half3(q * a), color.a);
}
";

thread_local! {
    static POSTERIZE_EFFECT: OnceCell<Option<RuntimeEffect>> = const { OnceCell::new() };
    static BRIGHT_PASS_EFFECT: OnceCell<Option<RuntimeEffect>> = const { OnceCell::new() };
    static GAIN_EFFECT: OnceCell<Option<RuntimeEffect>> = const { OnceCell::new() };
    static DIRECTIONAL_EFFECT: OnceCell<Option<RuntimeEffect>> = const { OnceCell::new() };
    static VIBRANCE_EFFECT: OnceCell<Option<RuntimeEffect>> = const { OnceCell::new() };
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

/// `vibrance` at `amount` in `[-1, 1]`, or `None` at `0`, the identity (ADR-0221).
pub(super) fn vibrance(amount: f64) -> Option<ImageFilter> {
    if amount == 0.0 {
        return None;
    }
    let amount = amount.clamp(-1.0, 1.0) as f32;
    image_filters::color_filter(
        colour_filter(&VIBRANCE_EFFECT, VIBRANCE, &[amount])?,
        None,
        None,
    )
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

// ---------------------------------------------------------------------------
// ADR-0225: edge-clamped stylize, blur, sharpen, edge, relief, and distortion shaders
// ---------------------------------------------------------------------------

/// `mosaic` (ADR-0225 §2): quantise element `[0, width] × [0, height]` into `blocks = [cols, rows]`
/// uniform cells, sampling each cell's centre with edge-clamped coordinates.
const MOSAIC: &str = r"
uniform shader source;
uniform float2 size;
uniform float2 blocks;

float2 clamp_edge(float2 p, float2 s) {
    return clamp(p, float2(0.5), max(float2(0.5), s - float2(0.5)));
}

half4 main(float2 p) {
    float2 cell = size / max(blocks, float2(1.0));
    float2 sample_p = (floor(p / cell) + float2(0.5)) * cell;
    return source.eval(clamp_edge(sample_p, size));
}
";

/// `radial_blur` (ADR-0225 §2): `zoom` (`mode < 0.5`) or `spin` (`mode >= 0.5`) around
/// `center_px = center * size`, with 16 evenly spaced samples along the radial ray or circular arc
/// and edge-clamped (`TileMode::Clamp`) coordinates within `[0, width] × [0, height]`.
const RADIAL_BLUR: &str = r"
uniform shader source;
uniform float2 size;
uniform float2 center_px;
uniform float amount;
uniform float mode;

float2 clamp_edge(float2 p, float2 s) {
    return clamp(p, float2(0.5), max(float2(0.5), s - float2(0.5)));
}

half4 main(float2 p) {
    const int N = 16;
    float2 rel = p - center_px;
    float4 sum = float4(0.0);
    if (mode < 0.5) {
        // Zoom: smear radially along the ray through center_px.
        float spread = clamp(amount / 100.0, 0.0, 1.0);
        for (int i = 0; i < N; i++) {
            float t = 2.0 * (float(i) / float(N - 1)) - 1.0;
            float2 q = center_px + rel * (1.0 + t * spread);
            sum += float4(source.eval(clamp_edge(q, size)));
        }
    } else {
        // Spin: smear angularly around center_px.
        float max_rad = (amount / 100.0) * 1.57079632679;
        for (int i = 0; i < N; i++) {
            float t = (float(i) / float(N - 1)) - 0.5;
            float a = t * max_rad;
            float c = cos(a);
            float s = sin(a);
            float2 rot = float2(rel.x * c - rel.y * s, rel.x * s + rel.y * c);
            sum += float4(source.eval(clamp_edge(center_px + rot, size)));
        }
    }
    return half4(sum / float(N));
}
";

/// `find_edges` (ADR-0225 §3): 3×3 Sobel gradient magnitude `sqrt(Gx² + Gy²)` per RGB channel
/// with edge-clamped neighbourhood coordinates (`TileMode::Clamp`), optionally inverted (`1 - g`),
/// preserving source alpha.
const FIND_EDGES: &str = r"
uniform shader source;
uniform float2 size;
uniform float invert_flag;

float2 clamp_edge(float2 p, float2 s) {
    return clamp(p, float2(0.5), max(float2(0.5), s - float2(0.5)));
}

float3 rgb_at(float2 p) {
    half4 c = source.eval(clamp_edge(p, size));
    float a = float(c.a);
    return a > 0.0 ? clamp(float3(c.rgb) / a, 0.0, 1.0) : float3(0.0);
}

half4 main(float2 p) {
    half4 centre = source.eval(clamp_edge(p, size));
    float a = float(centre.a);
    if (a <= 0.0) { return centre; }

    float3 tl = rgb_at(p + float2(-1.0, -1.0));
    float3 tc = rgb_at(p + float2( 0.0, -1.0));
    float3 tr = rgb_at(p + float2( 1.0, -1.0));
    float3 ml = rgb_at(p + float2(-1.0,  0.0));
    float3 mr = rgb_at(p + float2( 1.0,  0.0));
    float3 bl = rgb_at(p + float2(-1.0,  1.0));
    float3 bc = rgb_at(p + float2( 0.0,  1.0));
    float3 br = rgb_at(p + float2( 1.0,  1.0));

    float3 gx = -tl - 2.0 * ml - bl + tr + 2.0 * mr + br;
    float3 gy = -tl - 2.0 * tc - tr + bl + 2.0 * bc + br;
    float3 mag = clamp(sqrt(gx * gx + gy * gy), 0.0, 1.0);
    float3 out_rgb = invert_flag > 0.5 ? (float3(1.0) - mag) : mag;
    return half4(half3(out_rgb * a), centre.a);
}
";

/// `emboss` (ADR-0225 §3): directional relief along `angle` at sample offset `height` pixels,
/// centred on neutral mid-grey `(0.5, 0.5, 0.5)` at `contrast` (identity at `height == 0` or
/// `contrast == 0`), with edge-clamped coordinates (`TileMode::Clamp`).
const EMBOSS: &str = r"
uniform shader source;
uniform float2 size;
uniform float2 offset_px;
uniform float contrast;

float2 clamp_edge(float2 p, float2 s) {
    return clamp(p, float2(0.5), max(float2(0.5), s - float2(0.5)));
}

float3 rgb_at(float2 p) {
    half4 c = source.eval(clamp_edge(p, size));
    float a = float(c.a);
    return a > 0.0 ? clamp(float3(c.rgb) / a, 0.0, 1.0) : float3(0.0);
}

half4 main(float2 p) {
    half4 centre = source.eval(clamp_edge(p, size));
    float a = float(centre.a);
    if (a <= 0.0) { return centre; }

    float3 pos = rgb_at(p + offset_px);
    float3 neg = rgb_at(p - offset_px);
    float3 out_rgb = clamp(float3(0.5) + (pos - neg) * contrast, 0.0, 1.0);
    return half4(half3(out_rgb * a), centre.a);
}
";

/// `sharpen` (ADR-0225 §4): unsharp mask `out = clamp(src + amount * (src - blurred), 0, 1)`
/// against a uniform 3×3 box-blurred neighbourhood `/ 9` with edge-clamped coordinates
/// (`TileMode::Clamp`), preserving alpha.
const SHARPEN: &str = r"
uniform shader source;
uniform float2 size;
uniform float amount;

float2 clamp_edge(float2 p, float2 s) {
    return clamp(p, float2(0.5), max(float2(0.5), s - float2(0.5)));
}

float3 rgb_at(float2 p) {
    half4 c = source.eval(clamp_edge(p, size));
    float a = float(c.a);
    return a > 0.0 ? clamp(float3(c.rgb) / a, 0.0, 1.0) : float3(0.0);
}

half4 main(float2 p) {
    half4 centre = source.eval(clamp_edge(p, size));
    float a = float(centre.a);
    if (a <= 0.0) { return centre; }

    float3 cc = clamp(float3(centre.rgb) / a, 0.0, 1.0);
    float3 tl = rgb_at(p + float2(-1.0, -1.0));
    float3 tc = rgb_at(p + float2( 0.0, -1.0));
    float3 tr = rgb_at(p + float2( 1.0, -1.0));
    float3 ml = rgb_at(p + float2(-1.0,  0.0));
    float3 mr = rgb_at(p + float2( 1.0,  0.0));
    float3 bl = rgb_at(p + float2(-1.0,  1.0));
    float3 bc = rgb_at(p + float2( 0.0,  1.0));
    float3 br = rgb_at(p + float2( 1.0,  1.0));

    float3 blurred = (tl + tc + tr + ml + cc + mr + bl + bc + br) / 9.0;
    float3 out_rgb = clamp(cc + amount * (cc - blurred), 0.0, 1.0);
    return half4(half3(out_rgb * a), centre.a);
}
";

/// `twirl` (ADR-0225 §5): quadratic falloff vortex twist inside `radius` around `center_px`:
/// for `r < radius`, `u = 1 - r / radius`, `theta = radians(angle) * u * u`, rotating `p - center_px`
/// by `-theta` for inverse mapping and clamping to element edges (`TileMode::Clamp`).
const TWIRL: &str = r"
uniform shader source;
uniform float2 size;
uniform float2 center_px;
uniform float radius;
uniform float angle_rad;

float2 clamp_edge(float2 p, float2 s) {
    return clamp(p, float2(0.5), max(float2(0.5), s - float2(0.5)));
}

half4 main(float2 p) {
    float2 rel = p - center_px;
    float r = length(rel);
    if (r < radius && radius > 0.0) {
        float u = 1.0 - r / radius;
        float theta = -angle_rad * u * u;
        float c = cos(theta);
        float s = sin(theta);
        rel = float2(rel.x * c - rel.y * s, rel.x * s + rel.y * c);
    }
    return source.eval(clamp_edge(center_px + rel, size));
}
";

/// `bulge` (ADR-0225 §5): smooth radial magnify (`height > 0`) or pinch (`height < 0`) inside
/// `radius` around `center_px`, using smooth quadratic falloff `(1 - u²)²` and clamping to
/// element edges (`TileMode::Clamp`).
const BULGE: &str = r"
uniform shader source;
uniform float2 size;
uniform float2 center_px;
uniform float radius;
uniform float height;

float2 clamp_edge(float2 p, float2 s) {
    return clamp(p, float2(0.5), max(float2(0.5), s - float2(0.5)));
}

half4 main(float2 p) {
    float2 rel = p - center_px;
    float r = length(rel);
    if (r < radius && radius > 0.0) {
        float u = r / radius;
        float falloff = (1.0 - u * u) * (1.0 - u * u);
        float scale = height >= 0.0
            ? 1.0 / (1.0 + 2.0 * height * falloff)
            : 1.0 - 2.0 * height * falloff;
        rel *= scale;
    }
    return source.eval(clamp_edge(center_px + rel, size));
}
";

/// `optics_compensation` (ADR-0225 §5): symmetric Brown-Conrady radial lens distortion /
/// compensation around the element centre `(width / 2, height / 2)` with `k` derived from `fov`
/// (`reverse == false` barrel `k < 0`, `reverse == true` pincushion `k > 0`),
/// clamped to element edges (`TileMode::Clamp`).
const OPTICS_COMPENSATION: &str = r"
uniform shader source;
uniform float2 size;
uniform float k;

float2 clamp_edge(float2 p, float2 s) {
    return clamp(p, float2(0.5), max(float2(0.5), s - float2(0.5)));
}

half4 main(float2 p) {
    float2 center_px = size * 0.5;
    float half_diag = max(length(center_px), 1.0);
    float2 rel = (p - center_px) / half_diag;
    float r2 = dot(rel, rel);
    float factor = max(0.1, 1.0 + k * r2);
    float2 src_p = center_px + rel * factor * half_diag;
    return source.eval(clamp_edge(src_p, size));
}
";

/// `offset` (ADR-0225 §5): translate pixels by `(shift_x, shift_y)` in element space, sampling
/// `p - shift` with periodic modulo wrap (`wrap_flag > 0.5`) or edge clamp (`wrap_flag <= 0.5`).
const OFFSET: &str = r"
uniform shader source;
uniform float2 size;
uniform float2 shift;
uniform float wrap_flag;

float2 clamp_edge(float2 p, float2 s) {
    return clamp(p, float2(0.5), max(float2(0.5), s - float2(0.5)));
}

half4 main(float2 p) {
    float2 q = p - shift;
    if (wrap_flag > 0.5) {
        q = q - floor(q / size) * size;
    }
    return source.eval(clamp_edge(q, size));
}
";

thread_local! {
    static MOSAIC_EFFECT: OnceCell<Option<RuntimeEffect>> = const { OnceCell::new() };
    static RADIAL_BLUR_EFFECT: OnceCell<Option<RuntimeEffect>> = const { OnceCell::new() };
    static FIND_EDGES_EFFECT: OnceCell<Option<RuntimeEffect>> = const { OnceCell::new() };
    static EMBOSS_EFFECT: OnceCell<Option<RuntimeEffect>> = const { OnceCell::new() };
    static SHARPEN_EFFECT: OnceCell<Option<RuntimeEffect>> = const { OnceCell::new() };
    static TWIRL_EFFECT: OnceCell<Option<RuntimeEffect>> = const { OnceCell::new() };
    static BULGE_EFFECT: OnceCell<Option<RuntimeEffect>> = const { OnceCell::new() };
    static OPTICS_EFFECT: OnceCell<Option<RuntimeEffect>> = const { OnceCell::new() };
    static OFFSET_EFFECT: OnceCell<Option<RuntimeEffect>> = const { OnceCell::new() };
}

fn cropped_shader(
    builder: &RuntimeShaderBuilder,
    max_sample_radius: f32,
    width: f64,
    height: f64,
) -> Option<ImageFilter> {
    let rect = Rect::from_wh(width.max(0.0) as f32, height.max(0.0) as f32);
    let filter = image_filters::runtime_shader_with_options(
        builder,
        max_sample_radius,
        "source",
        None,
        false,
    )?;
    image_filters::crop(rect, None, filter)
}

pub(super) fn mosaic(blocks: [f64; 2], width: f64, height: f64) -> Option<ImageFilter> {
    let bx = blocks[0].round().max(1.0);
    let by = blocks[1].round().max(1.0);
    if width <= 0.0 || height <= 0.0 || (bx >= width.round() && by >= height.round()) {
        return None;
    }
    let effect = compiled(&MOSAIC_EFFECT, || {
        RuntimeEffect::make_for_shader(MOSAIC, None).ok()
    })?;
    let mut builder = RuntimeShaderBuilder::new(effect);
    builder
        .set_uniform_float("size", &[width as f32, height as f32])
        .ok()?;
    builder
        .set_uniform_float("blocks", &[bx as f32, by as f32])
        .ok()?;
    let max_reach = (width / bx).max(height / by).ceil() as f32 + 1.0;
    cropped_shader(&builder, max_reach, width, height)
}

pub(super) fn radial_blur(
    spin: bool,
    center: [f64; 2],
    amount: f64,
    width: f64,
    height: f64,
) -> Option<ImageFilter> {
    if amount <= 0.0 || width <= 0.0 || height <= 0.0 {
        return None;
    }
    let effect = compiled(&RADIAL_BLUR_EFFECT, || {
        RuntimeEffect::make_for_shader(RADIAL_BLUR, None).ok()
    })?;
    let mut builder = RuntimeShaderBuilder::new(effect);
    let cx = (center[0].clamp(0.0, 1.0) * width) as f32;
    let cy = (center[1].clamp(0.0, 1.0) * height) as f32;
    builder
        .set_uniform_float("size", &[width as f32, height as f32])
        .ok()?;
    builder
        .set_uniform_float("center_px", &[cx, cy])
        .ok()?;
    builder
        .set_uniform_float("amount", &[amount as f32])
        .ok()?;
    builder
        .set_uniform_float("mode", &[if spin { 1.0 } else { 0.0 }])
        .ok()?;
    let diag = width.hypot(height) as f32;
    cropped_shader(&builder, diag, width, height)
}

pub(super) fn find_edges(invert: bool, width: f64, height: f64) -> Option<ImageFilter> {
    if width <= 0.0 || height <= 0.0 {
        return None;
    }
    let effect = compiled(&FIND_EDGES_EFFECT, || {
        RuntimeEffect::make_for_shader(FIND_EDGES, None).ok()
    })?;
    let mut builder = RuntimeShaderBuilder::new(effect);
    builder
        .set_uniform_float("size", &[width as f32, height as f32])
        .ok()?;
    builder
        .set_uniform_float("invert_flag", &[if invert { 1.0 } else { 0.0 }])
        .ok()?;
    cropped_shader(&builder, 2.0, width, height)
}

pub(super) fn emboss(
    angle: f64,
    height_px: f64,
    contrast: f64,
    width: f64,
    height: f64,
) -> Option<ImageFilter> {
    if height_px <= 0.0 || contrast <= 0.0 || width <= 0.0 || height <= 0.0 {
        return None;
    }
    let effect = compiled(&EMBOSS_EFFECT, || {
        RuntimeEffect::make_for_shader(EMBOSS, None).ok()
    })?;
    let theta = angle.to_radians();
    let offset_px = [
        (theta.cos() * height_px) as f32,
        (theta.sin() * height_px) as f32,
    ];
    let mut builder = RuntimeShaderBuilder::new(effect);
    builder
        .set_uniform_float("size", &[width as f32, height as f32])
        .ok()?;
    builder.set_uniform_float("offset_px", &offset_px).ok()?;
    builder
        .set_uniform_float("contrast", &[contrast as f32])
        .ok()?;
    cropped_shader(&builder, height_px.ceil() as f32 + 2.0, width, height)
}

pub(super) fn sharpen(amount: f64, width: f64, height: f64) -> Option<ImageFilter> {
    if amount <= 0.0 || width <= 0.0 || height <= 0.0 {
        return None;
    }
    let effect = compiled(&SHARPEN_EFFECT, || {
        RuntimeEffect::make_for_shader(SHARPEN, None).ok()
    })?;
    let mut builder = RuntimeShaderBuilder::new(effect);
    builder
        .set_uniform_float("size", &[width as f32, height as f32])
        .ok()?;
    builder
        .set_uniform_float("amount", &[amount as f32])
        .ok()?;
    cropped_shader(&builder, 2.0, width, height)
}

pub(super) fn twirl(
    center: [f64; 2],
    radius: f64,
    angle: f64,
    width: f64,
    height: f64,
) -> Option<ImageFilter> {
    if angle.abs() <= 1e-9 || radius <= 0.0 || width <= 0.0 || height <= 0.0 {
        return None;
    }
    let effect = compiled(&TWIRL_EFFECT, || {
        RuntimeEffect::make_for_shader(TWIRL, None).ok()
    })?;
    let cx = (center[0].clamp(0.0, 1.0) * width) as f32;
    let cy = (center[1].clamp(0.0, 1.0) * height) as f32;
    let radius_px = (radius * width.max(height)) as f32;
    let mut builder = RuntimeShaderBuilder::new(effect);
    builder
        .set_uniform_float("size", &[width as f32, height as f32])
        .ok()?;
    builder
        .set_uniform_float("center_px", &[cx, cy])
        .ok()?;
    builder.set_uniform_float("radius", &[radius_px]).ok()?;
    builder
        .set_uniform_float("angle_rad", &[angle.to_radians() as f32])
        .ok()?;
    cropped_shader(&builder, radius_px.ceil() + 2.0, width, height)
}

pub(super) fn bulge(
    center: [f64; 2],
    radius: f64,
    bulge_height: f64,
    width: f64,
    height: f64,
) -> Option<ImageFilter> {
    if bulge_height.abs() <= 1e-9 || radius <= 0.0 || width <= 0.0 || height <= 0.0 {
        return None;
    }
    let effect = compiled(&BULGE_EFFECT, || {
        RuntimeEffect::make_for_shader(BULGE, None).ok()
    })?;
    let cx = (center[0].clamp(0.0, 1.0) * width) as f32;
    let cy = (center[1].clamp(0.0, 1.0) * height) as f32;
    let radius_px = (radius * width.max(height)) as f32;
    let mut builder = RuntimeShaderBuilder::new(effect);
    builder
        .set_uniform_float("size", &[width as f32, height as f32])
        .ok()?;
    builder
        .set_uniform_float("center_px", &[cx, cy])
        .ok()?;
    builder.set_uniform_float("radius", &[radius_px]).ok()?;
    builder
        .set_uniform_float("height", &[bulge_height as f32])
        .ok()?;
    cropped_shader(&builder, radius_px.ceil() + 2.0, width, height)
}

pub(super) fn optics_compensation(
    fov: f64,
    reverse: bool,
    width: f64,
    height: f64,
) -> Option<ImageFilter> {
    if fov <= 0.0 || width <= 0.0 || height <= 0.0 {
        return None;
    }
    let effect = compiled(&OPTICS_EFFECT, || {
        RuntimeEffect::make_for_shader(OPTICS_COMPENSATION, None).ok()
    })?;
    let half_rad = (fov.clamp(0.0, 180.0) * 0.5).to_radians() / std::f64::consts::FRAC_PI_2;
    let sign = if reverse { 1.0 } else { -1.0 };
    let k = (sign * 0.75 * half_rad * half_rad) as f32;
    let mut builder = RuntimeShaderBuilder::new(effect);
    builder
        .set_uniform_float("size", &[width as f32, height as f32])
        .ok()?;
    builder.set_uniform_float("k", &[k]).ok()?;
    let diag = width.hypot(height) as f32;
    cropped_shader(&builder, diag, width, height)
}

pub(super) fn offset(
    shift_x: f64,
    shift_y: f64,
    wrap: bool,
    width: f64,
    height: f64,
) -> Option<ImageFilter> {
    if (shift_x.abs() <= 1e-9 && shift_y.abs() <= 1e-9) || width <= 0.0 || height <= 0.0 {
        return None;
    }
    let effect = compiled(&OFFSET_EFFECT, || {
        RuntimeEffect::make_for_shader(OFFSET, None).ok()
    })?;
    let mut builder = RuntimeShaderBuilder::new(effect);
    builder
        .set_uniform_float("size", &[width as f32, height as f32])
        .ok()?;
    builder
        .set_uniform_float("shift", &[shift_x as f32, shift_y as f32])
        .ok()?;
    builder
        .set_uniform_float("wrap_flag", &[if wrap { 1.0 } else { 0.0 }])
        .ok()?;
    let diag = width.hypot(height) as f32;
    cropped_shader(&builder, diag, width, height)
}
