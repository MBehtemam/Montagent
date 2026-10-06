//! PROTOTYPE #722 — throwaway, never to merge.
//!
//! ADR-0156's four named effects: `grain`, `glow`, `posterize` and `directional_blur`.
//!
//! Every `RuntimeEffect` is compiled once **per painter thread** (a `thread_local` cache keyed
//! by the SkSL's name), so nothing Montagent builds is shared between painters. Each paint
//! then binds fresh uniforms to it.

use std::cell::RefCell;
use std::collections::HashMap;

use skia_safe::{
    AlphaType, Blender, BlendMode, ColorFilter, ColorType, Data, FilterMode, ISize, ImageFilter,
    ImageInfo, Matrix, MipmapMode, Paint as SkPaint, Rect, RuntimeEffect, SamplingOptions,
    TileMode, image_filters, images, runtime_effect::RuntimeShaderBuilder,
};

/// `posterize`, ADR-0156 §4: per channel on non-premultiplied colour,
/// `q = round(v × (levels − 1)) / (levels − 1)`, ties away from zero (`floor(x + 0.5)` for
/// `x ≥ 0`). Alpha untouched; transparent black stays transparent black.
const POSTERIZE: &str = r"
uniform float levels;
half4 main(half4 color) {
    float a = float(color.a);
    if (a <= 0.0) { return color; }
    float n = levels - 1.0;
    float3 v = float3(color.rgb) / a;
    float3 q = floor(v * n + 0.5) / n;
    return half4(half3(q * a), half(a));
}
";

/// `glow`'s bright-pass, ADR-0156 §4: the pixel scaled by
/// `max(0, luma − threshold) / (1 − threshold)`, luma Rec.709 on non-premultiplied sRGB.
/// The whole premultiplied pixel is scaled (alpha too), so the bright part is the element's
/// own colour at a fraction of its opacity. At `threshold: 1` the pass is empty by
/// definition, so nothing divides by zero.
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

/// `glow`'s `intensity`: the blurred bright part's premultiplied pixel times a gain.
const GAIN: &str = r"
uniform float gain;
half4 main(half4 color) {
    return half4(float4(color) * gain);
}
";

/// `directional_blur`, ADR-0156 §4: `count = ceil(length) + 1` samples evenly spaced along
/// the line through the pixel, centred, equal weights. `step` is the offset between two
/// samples in element (parameter) space.
const DIRECTIONAL: &str = r"
uniform shader src;
uniform float2 delta;
uniform float count;
half4 main(float2 p) {
    float4 sum = float4(0);
    float centre = (count - 1.0) * 0.5;
    for (int i = 0; i < 4097; i++) {
        if (float(i) >= count) { break; }
        sum += float4(src.eval(p + delta * (float(i) - centre)));
    }
    return half4(sum / count);
}
";

/// `grain`'s offset, ADR-0156 §4, as a blender over the element's own pixels: `src` is the
/// draw (an 8-bit value `d` per channel from the integer hash, read as `d / 255`), `dst` the
/// element. The offset `amount × (2d − 255) / 255` is symmetric, added to non-premultiplied
/// colour and clamped to 0–1; alpha is never changed.
const GRAIN: &str = r"
uniform float amount;
half4 main(half4 src, half4 dst) {
    float a = float(dst.a);
    if (a <= 0.0) { return dst; }
    float3 v = float3(dst.rgb) / a;
    float3 offset = amount * (float3(src.rgb) * 2.0 - 1.0);
    float3 c = clamp(v + offset, 0.0, 1.0);
    return half4(half3(c * a), half(a));
}
";

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Kind {
    ColourFilter,
    Shader,
    Blender,
}

thread_local! {
    /// One compiled `RuntimeEffect` per SkSL per painter thread.
    static COMPILED: RefCell<HashMap<&'static str, RuntimeEffect>> = RefCell::new(HashMap::new());
}

fn compiled(sksl: &'static str, kind: Kind) -> Option<RuntimeEffect> {
    COMPILED.with(|compiled| {
        if let Some(effect) = compiled.borrow().get(sksl) {
            return Some(effect.clone());
        }
        let started = std::time::Instant::now();
        let effect = match kind {
            Kind::ColourFilter => RuntimeEffect::make_for_color_filter(sksl, None),
            Kind::Shader => RuntimeEffect::make_for_shader(sksl, None),
            Kind::Blender => RuntimeEffect::make_for_blender(sksl, None),
        };
        let effect = match effect {
            Ok(effect) => effect,
            Err(error) => {
                eprintln!("proto-sksl-error {error}");
                return None;
            }
        };
        if std::env::var_os("MONTAGENT_PROTO_COMPILE").is_some() {
            eprintln!(
                "proto-compile {:?} {} ns",
                std::thread::current().id(),
                started.elapsed().as_nanos()
            );
        }
        compiled.borrow_mut().insert(sksl, effect.clone());
        Some(effect)
    })
}

fn uniforms(values: &[f32]) -> Data {
    let bytes: Vec<u8> = values.iter().flat_map(|v| v.to_ne_bytes()).collect();
    Data::new_copy(&bytes)
}

fn colour_filter(sksl: &'static str, values: &[f32]) -> Option<ColorFilter> {
    compiled(sksl, Kind::ColourFilter)?.make_color_filter(uniforms(values), None)
}

/// `posterize` as a colour filter; `levels` is already an integer from 2 to 256.
pub(super) fn posterize(levels: f64) -> Option<ColorFilter> {
    colour_filter(POSTERIZE, &[levels as f32])
}

/// `glow`: source Plus gain(blur(bright-pass(source))), inside the element's layer.
pub(super) fn glow(threshold: f64, sigma: f32, intensity: f64) -> Option<ImageFilter> {
    let bright = image_filters::color_filter(
        colour_filter(BRIGHT_PASS, &[threshold as f32])?,
        None,
        None,
    )?;
    let blurred = image_filters::blur((sigma, sigma), None, bright, None)?;
    let boosted =
        image_filters::color_filter(colour_filter(GAIN, &[intensity as f32])?, blurred, None)?;
    image_filters::blend(BlendMode::Plus, None, boosted, None)
}

/// `directional_blur` as a runtime-shader image filter, its sample radius declared to Skia
/// as the larger of the two axes' half-smear.
pub(super) fn directional_blur(angle: f64, length: f64) -> Option<ImageFilter> {
    let length = length.max(0.0);
    let count = length.ceil() + 1.0;
    let theta = angle.to_radians();
    let spacing = if count > 1.0 {
        length / (count - 1.0)
    } else {
        0.0
    };
    let step = [(theta.cos() * spacing) as f32, (theta.sin() * spacing) as f32];
    let mut builder = RuntimeShaderBuilder::new(compiled(DIRECTIONAL, Kind::Shader)?);
    builder.set_uniform_float("delta", &step).ok()?;
    builder.set_uniform_float("count", &[count as f32]).ok()?;
    let reach = (theta.cos().abs().max(theta.sin().abs()) * length / 2.0) as f32;
    image_filters::runtime_shader_with_options(&builder, reach.ceil() + 1.0, "src", None, false)
}

/// prototype(#722): `MONTAGENT_PROTO_PROBE_INT` asks this Skia whether a runtime effect may
/// use integer bitwise operators and `%`, which an in-shader integer hash would need.
pub fn probe_integer_hash() {
    for (what, sksl) in [
        ("xor", "half4 main(float2 p) { int x = int(p.x); x = x ^ 7; return half4(half(x)); }"),
        ("shift", "half4 main(float2 p) { int x = int(p.x); x = x >> 3; return half4(half(x)); }"),
        ("mod", "half4 main(float2 p) { int x = int(p.x); x = x % 7; return half4(half(x)); }"),
        ("uint", "half4 main(float2 p) { uint x = uint(p.x); return half4(half(x)); }"),
        ("int-mul-div", "half4 main(float2 p) { int x = int(p.x); x = x * 1103515245 + 12345; x = x / 65536; return half4(half(x)); }"),
    ] {
        match RuntimeEffect::make_for_shader(sksl, None) {
            Ok(_) => eprintln!("proto-probe-int {what}: compiles"),
            Err(error) => eprintln!("proto-probe-int {what}: refused: {}", error.trim()),
        }
    }
}

/// A fixed 64-bit integer mix (SplitMix64's finaliser). No floats anywhere in the draw.
fn mix(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// One 8-bit draw for (seed, local instant, cell, channel).
fn draw(seed: i64, tick: i64, cx: i64, cy: i64, channel: u64) -> u8 {
    let mut h = mix(seed as u64);
    h = mix(h ^ tick as u64);
    h = mix(h ^ cx as u64);
    h = mix(h ^ cy as u64);
    h = mix(h ^ channel);
    (h >> 56) as u8
}

/// Paint `grain` over what this layer holds: one draw per `size`×`size` cell in element
/// space, anchored at the box origin, so the cells ride the element's transform.
///
/// The draws are made on the CPU into a cell-resolution image (SkSL runtime effects are
/// ES2 here: no bitwise integer operators, so an integer hash cannot live in the shader).
/// The image is sampled nearest, under the element's matrix, and a runtime blender applies
/// the offset to the element's own pixels.
pub(super) fn grain(
    canvas: &skia_safe::Canvas,
    seed: i64,
    amount: f64,
    size: i64,
    mono: bool,
    tick: i64,
) {
    if std::env::var_os("MONTAGENT_PROTO_PROBE_INT").is_some() {
        probe_integer_hash();
    }
    if amount <= 0.0 {
        return;
    }
    let size = size.clamp(1, 8);
    let Some(clip) = canvas.device_clip_bounds() else {
        return;
    };
    let Some(to_element) = canvas.local_to_device_as_3x3().invert() else {
        return;
    };
    let (area, _) = to_element.map_rect(Rect::from(clip));
    let s = size as f32;
    let (x0, y0) = ((area.left / s).floor() as i64 - 1, (area.top / s).floor() as i64 - 1);
    let (x1, y1) = ((area.right / s).ceil() as i64 + 1, (area.bottom / s).ceil() as i64 + 1);
    let (w, h) = ((x1 - x0).max(1), (y1 - y0).max(1));
    if w * h > 64_000_000 {
        return;
    }
    let mut pixels = vec![0u8; (w * h * 4) as usize];
    for cy in 0..h {
        for cx in 0..w {
            let i = ((cy * w + cx) * 4) as usize;
            let (gx, gy) = (x0 + cx, y0 + cy);
            let r = draw(seed, tick, gx, gy, 0);
            let (g, b) = if mono {
                (r, r)
            } else {
                (draw(seed, tick, gx, gy, 1), draw(seed, tick, gx, gy, 2))
            };
            pixels[i..i + 4].copy_from_slice(&[r, g, b, 0xFF]);
        }
    }
    let info = ImageInfo::new(
        ISize::new(w as i32, h as i32),
        ColorType::RGBA8888,
        AlphaType::Opaque,
        None,
    );
    let Some(image) = images::raster_from_data(&info, Data::new_copy(&pixels), (w * 4) as usize)
    else {
        return;
    };
    let mut local = Matrix::translate(((x0 * size) as f32, (y0 * size) as f32));
    local.pre_scale((s, s), None);
    let Some(shader) = image.to_shader(
        (TileMode::Clamp, TileMode::Clamp),
        SamplingOptions::new(FilterMode::Nearest, MipmapMode::None),
        &local,
    ) else {
        return;
    };
    let Some(blender): Option<Blender> = compiled(GRAIN, Kind::Blender)
        .and_then(|effect| effect.make_blender(uniforms(&[amount as f32]), None))
    else {
        return;
    };
    let mut paint = SkPaint::default();
    paint.set_shader(shader);
    paint.set_blender(blender);
    canvas.draw_paint(&paint);
}
