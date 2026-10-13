//! ASC CDL and 3D LUT tetrahedral interpolation shaders (ADR-0206).
//!
//! - **ASC CDL**: Slope, Offset, Power, and Saturation color grading.
//! - **3D LUT**: Tetrahedral interpolation over 3D `.cube` color lookup tables.

use std::cell::OnceCell;

use skia_safe::images;
use skia_safe::runtime_effect::ChildPtr;
use skia_safe::{
    AlphaType, ColorFilter, ColorType, Data, FilterMode, ISize, ImageInfo, MipmapMode,
    RuntimeEffect, SamplingOptions,
};

use super::named::compiled;

/// ASC CDL shader (ADR-0206):
/// c_graded = [max(0, c * slope + offset)] ^ power
/// Rec.709 luma = 0.2126 * R + 0.7152 * G + 0.0722 * B
/// c_final = clamp(Y + saturation * (c_graded - Y), 0.0, 1.0)
const CDL_SKSL: &str = r"
uniform float slope_r;
uniform float slope_g;
uniform float slope_b;
uniform float offset_r;
uniform float offset_g;
uniform float offset_b;
uniform float power_r;
uniform float power_g;
uniform float power_b;
uniform float saturation;

half4 main(half4 color) {
    float a = float(color.a);
    if (a <= 0.0) { return color; }

    float3 c = clamp(float3(color.rgb) / a, 0.0, 1.0);
    float3 slope = float3(slope_r, slope_g, slope_b);
    float3 offset = float3(offset_r, offset_g, offset_b);
    float3 power = float3(power_r, power_g, power_b);

    float3 base = max(float3(0.0), c * slope + offset);
    float3 graded = float3(
        base.r > 0.0 ? pow(base.r, power.r) : 0.0,
        base.g > 0.0 ? pow(base.g, power.g) : 0.0,
        base.b > 0.0 ? pow(base.b, power.b) : 0.0
    );

    float y = dot(graded, float3(0.2126, 0.7152, 0.0722));
    float3 final_c = clamp(float3(y) + saturation * (graded - float3(y)), 0.0, 1.0);

    return half4(half3(final_c * a), color.a);
}
";

/// 3D LUT tetrahedral interpolation shader (ADR-0206):
/// Evaluates a 3D RGB lattice stored in a 2D atlas texture of width N*N and height N.
/// Samples exactly 4 simplex vertices based on fractional ordering to eliminate diagonal banding.
const LUT3D_SKSL: &str = r"
uniform shader lut;
uniform float lut_size;
uniform float intensity;

float3 sample_lut(float r, float g, float b, float size) {
    float x = b * size + r + 0.5;
    float y = g + 0.5;
    return float3(lut.eval(float2(x, y)).rgb);
}

half4 main(half4 color) {
    float a = float(color.a);
    if (a <= 0.0) { return color; }

    float3 rgb = clamp(float3(color.rgb) / a, 0.0, 1.0);
    float size = lut_size;
    float max_idx = size - 1.0;
    float3 scaled = rgb * max_idx;

    float3 f0 = floor(scaled);
    float3 f1 = min(f0 + float3(1.0), float3(max_idx));
    float3 delta = scaled - f0;

    float r0 = f0.r; float r1 = f1.r;
    float g0 = f0.g; float g1 = f1.g;
    float b0 = f0.b; float b1 = f1.b;

    float dr = delta.r;
    float dg = delta.g;
    float db = delta.b;

    float3 p000 = sample_lut(r0, g0, b0, size);
    float3 p111 = sample_lut(r1, g1, b1, size);
    float3 graded;

    if (dr >= dg && dg >= db) {
        float3 p100 = sample_lut(r1, g0, b0, size);
        float3 p110 = sample_lut(r1, g1, b0, size);
        graded = p000 + dr * (p100 - p000) + dg * (p110 - p100) + db * (p111 - p110);
    } else if (dr >= db && db > dg) {
        float3 p100 = sample_lut(r1, g0, b0, size);
        float3 p101 = sample_lut(r1, g0, b1, size);
        graded = p000 + dr * (p100 - p000) + db * (p101 - p100) + dg * (p111 - p101);
    } else if (dg >= dr && dr >= db) {
        float3 p010 = sample_lut(r0, g1, b0, size);
        float3 p110 = sample_lut(r1, g1, b0, size);
        graded = p000 + dg * (p010 - p000) + dr * (p110 - p010) + db * (p111 - p110);
    } else if (dg >= db && db > dr) {
        float3 p010 = sample_lut(r0, g1, b0, size);
        float3 p011 = sample_lut(r0, g1, b1, size);
        graded = p000 + dg * (p010 - p000) + db * (p011 - p010) + dr * (p111 - p011);
    } else if (db >= dr && dr >= dg) {
        float3 p001 = sample_lut(r0, g0, b1, size);
        float3 p101 = sample_lut(r1, g0, b1, size);
        graded = p000 + db * (p001 - p000) + dr * (p101 - p001) + dg * (p111 - p101);
    } else {
        float3 p001 = sample_lut(r0, g0, b1, size);
        float3 p011 = sample_lut(r0, g1, b1, size);
        graded = p000 + db * (p001 - p000) + dg * (p011 - p001) + dr * (p111 - p011);
    }

    float3 result = mix(rgb, graded, intensity);
    result = clamp(result, 0.0, 1.0);

    return half4(half3(result * a), color.a);
}
";

thread_local! {
    static CDL_EFFECT: OnceCell<Option<RuntimeEffect>> = const { OnceCell::new() };
    static LUT3D_EFFECT: OnceCell<Option<RuntimeEffect>> = const { OnceCell::new() };
}

/// Parsed `.cube` 3D lookup table.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedCube {
    pub title: Option<String>,
    pub size: u32,
    pub table: Vec<u8>,
}

/// Parses an Adobe / Iridas `.cube` file according to ADR-0206.
pub fn parse_cube(cube_text: &str) -> Result<ParsedCube, String> {
    let mut title = None;
    let mut size: Option<u32> = None;
    let mut entries = Vec::new();

    for (line_num, line) in cube_text.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("TITLE") {
            title = Some(rest.trim().trim_matches('"').to_string());
            continue;
        }

        if trimmed.starts_with("LUT_1D_SIZE") {
            return Err(
                "1D LUTs (LUT_1D_SIZE) are not supported; convert to a 3D LUT or use a color curve (ADR-0206)"
                    .to_string(),
            );
        }

        if let Some(rest) = trimmed.strip_prefix("LUT_3D_SIZE") {
            let n: u32 = rest.trim().parse().map_err(|_| {
                format!(
                    "invalid LUT_3D_SIZE value '{rest}' at line {}",
                    line_num + 1
                )
            })?;
            if !(2..=65).contains(&n) {
                return Err(format!(
                    "LUT_3D_SIZE must be between 2 and 65, got {n} (ADR-0206)"
                ));
            }
            size = Some(n);
            continue;
        }

        if trimmed.starts_with("DOMAIN_MIN") || trimmed.starts_with("DOMAIN_MAX") {
            // Optional domain bounds; standard [0, 1] assumed
            continue;
        }

        // Must be RGB data triplet
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() != 3 {
            return Err(format!(
                "malformed RGB table entry at line {}: expected 3 numbers, got {}",
                line_num + 1,
                parts.len()
            ));
        }

        let r: f64 = parts[0].parse().map_err(|_| {
            format!(
                "malformed float '{}' in RGB entry at line {}",
                parts[0],
                line_num + 1
            )
        })?;
        let g: f64 = parts[1].parse().map_err(|_| {
            format!(
                "malformed float '{}' in RGB entry at line {}",
                parts[1],
                line_num + 1
            )
        })?;
        let b: f64 = parts[2].parse().map_err(|_| {
            format!(
                "malformed float '{}' in RGB entry at line {}",
                parts[2],
                line_num + 1
            )
        })?;

        if !r.is_finite() || !g.is_finite() || !b.is_finite() {
            return Err(format!(
                "non-finite float in RGB entry at line {}",
                line_num + 1
            ));
        }

        entries.push([r, g, b]);
    }

    let Some(n) = size else {
        return Err("missing LUT_3D_SIZE declaration in header (ADR-0206)".to_string());
    };

    let expected = (n * n * n) as usize;
    if entries.len() != expected {
        return Err(format!(
            "expected {expected} RGB table data entries ({n}x{n}x{n}), found {} (ADR-0206)",
            entries.len()
        ));
    }

    // Convert entries to 2D atlas RGBA8888 bytes:
    // Atlas width = N * N, height = N.
    // For lattice coordinate (r, g, b), atlas pixel is (b * N + r, g).
    // In .cube files, data ordering is: b in 0..N, g in 0..N, r in 0..N.
    let mut table = vec![255u8; expected * 4];
    for b in 0..n {
        for g in 0..n {
            for r in 0..n {
                let cube_idx = (b * n * n + g * n + r) as usize;
                let [cr, cg, cb] = entries[cube_idx];
                let atlas_x = b * n + r;
                let atlas_y = g;
                let atlas_idx = (atlas_y * (n * n) + atlas_x) as usize * 4;

                table[atlas_idx] = (cr.clamp(0.0, 1.0) * 255.0).round() as u8;
                table[atlas_idx + 1] = (cg.clamp(0.0, 1.0) * 255.0).round() as u8;
                table[atlas_idx + 2] = (cb.clamp(0.0, 1.0) * 255.0).round() as u8;
                table[atlas_idx + 3] = 255;
            }
        }
    }

    Ok(ParsedCube {
        title,
        size: n,
        table,
    })
}

/// Builds an SkSL color filter for ASC CDL color grading.
pub fn cdl_filter(
    slope: [f64; 3],
    offset: [f64; 3],
    power: [f64; 3],
    saturation: f64,
) -> Option<ColorFilter> {
    // Identity fast-path
    if slope == [1.0, 1.0, 1.0]
        && offset == [0.0, 0.0, 0.0]
        && power == [1.0, 1.0, 1.0]
        && (saturation - 1.0).abs() < 1e-6
    {
        return None;
    }

    let effect = compiled(&CDL_EFFECT, || {
        RuntimeEffect::make_for_color_filter(CDL_SKSL, None).ok()
    })?;

    let uniforms = [
        slope[0] as f32,
        slope[1] as f32,
        slope[2] as f32,
        offset[0] as f32,
        offset[1] as f32,
        offset[2] as f32,
        power[0] as f32,
        power[1] as f32,
        power[2] as f32,
        saturation as f32,
    ];
    let bytes: Vec<u8> = uniforms.iter().flat_map(|v| v.to_ne_bytes()).collect();
    effect.make_color_filter(Data::new_copy(&bytes), None)
}

/// Builds an SkSL color filter for 3D LUT tetrahedral interpolation.
pub fn lut3d_filter(size: u32, table: &[u8], intensity: f64) -> Option<ColorFilter> {
    if intensity <= 0.0 {
        return None;
    }

    let effect = compiled(&LUT3D_EFFECT, || {
        RuntimeEffect::make_for_color_filter(LUT3D_SKSL, None).ok()
    })?;

    let info = ImageInfo::new(
        ISize::new((size * size) as i32, size as i32),
        ColorType::RGBA8888,
        AlphaType::Opaque,
        None,
    );
    let row_bytes = (size * size * 4) as usize;
    let data = Data::new_copy(table);
    let image = images::raster_from_data(&info, data, row_bytes)?;
    let shader = image.to_shader(
        None,
        SamplingOptions::new(FilterMode::Nearest, MipmapMode::None),
        None,
    )?;

    let uniforms = [size as f32, intensity.clamp(0.0, 1.0) as f32];
    let bytes: Vec<u8> = uniforms.iter().flat_map(|v| v.to_ne_bytes()).collect();
    let children = [ChildPtr::from(shader)];

    effect.make_color_filter(Data::new_copy(&bytes), Some(&children[..]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::{Canvas, Effect, Rgba};

    #[test]
    fn parse_cube_valid() {
        let cube_text = r#"
# Test Cube
TITLE "Test Identity"
LUT_3D_SIZE 2
0.0 0.0 0.0
1.0 0.0 0.0
0.0 1.0 0.0
1.0 1.0 0.0
0.0 0.0 1.0
1.0 0.0 1.0
0.0 1.0 1.0
1.0 1.0 1.0
"#;
        let parsed = parse_cube(cube_text).expect("valid cube");
        assert_eq!(parsed.size, 2);
        assert_eq!(parsed.title.as_deref(), Some("Test Identity"));
        assert_eq!(parsed.table.len(), 2 * 2 * 2 * 4);
    }

    #[test]
    fn parse_cube_rejects_1d() {
        let cube_text = "LUT_1D_SIZE 16\n0.0 0.0 0.0\n";
        let err = parse_cube(cube_text).unwrap_err();
        assert!(err.contains("1D LUTs (LUT_1D_SIZE) are not supported"));
    }

    #[test]
    fn parse_cube_rejects_out_of_bounds_size() {
        let err = parse_cube("LUT_3D_SIZE 1\n").unwrap_err();
        assert!(err.contains("LUT_3D_SIZE must be between 2 and 65"));

        let err2 = parse_cube("LUT_3D_SIZE 66\n").unwrap_err();
        assert!(err2.contains("LUT_3D_SIZE must be between 2 and 65"));
    }

    #[test]
    fn parse_cube_rejects_incomplete_data() {
        let cube_text = "LUT_3D_SIZE 2\n0.0 0.0 0.0\n1.0 1.0 1.0\n";
        let err = parse_cube(cube_text).unwrap_err();
        assert!(err.contains("expected 8 RGB table data entries (2x2x2), found 2"));
    }

    #[test]
    fn cdl_filter_identity_returns_none() {
        let filter = cdl_filter([1.0, 1.0, 1.0], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0], 1.0);
        assert!(filter.is_none());
    }

    #[test]
    fn cdl_filter_grades_pixels_correctly() {
        let mut canvas = Canvas::new(4, 4).unwrap();
        // Base color: mid-gray [128, 128, 128, 255] ~ 0.50196
        canvas.background(Rgba([128, 128, 128, 255]));

        // Graded: slope 2.0 -> 0.50196 * 2 = ~1.0039 -> clamped to 1.0 -> 255
        let effect = Effect::Cdl {
            slope: [2.0, 1.0, 0.5],
            offset: [0.0, 0.0, 0.0],
            power: [1.0, 1.0, 1.0],
            saturation: 1.0,
        };
        canvas.apply_master_effects(&[effect]);

        let rgba = canvas.rgba().expect("rgba pixels");
        // R should be 255 (scaled by 2.0)
        // G should be ~128 (scaled by 1.0)
        // B should be ~64 (scaled by 0.5)
        assert_eq!(rgba[0], 255);
        assert!((rgba[1] as i32 - 128).abs() <= 2);
        assert!((rgba[2] as i32 - 64).abs() <= 2);
        assert_eq!(rgba[3], 255);
    }

    #[test]
    fn lut3d_filter_inverts_colors() {
        let mut canvas = Canvas::new(4, 4).unwrap();
        canvas.background(Rgba([200, 50, 100, 255]));

        // 2x2x2 Inversion LUT: maps c to 1.0 - c
        let cube_text = r#"
LUT_3D_SIZE 2
1.0 1.0 1.0
0.0 1.0 1.0
1.0 0.0 1.0
0.0 0.0 1.0
1.0 1.0 0.0
0.0 1.0 0.0
1.0 0.0 0.0
0.0 0.0 0.0
"#;
        let parsed = parse_cube(cube_text).unwrap();
        let effect = Effect::Lut3d {
            size: parsed.size,
            table: parsed.table,
            intensity: 1.0,
        };
        canvas.apply_master_effects(&[effect]);

        let rgba = canvas.rgba().expect("rgba pixels");
        // Inverted:
        // R: 255 - 200 = 55
        // G: 255 - 50 = 205
        // B: 255 - 100 = 155
        assert!((rgba[0] as i32 - 55).abs() <= 2, "R was {}", rgba[0]);
        assert!((rgba[1] as i32 - 205).abs() <= 2, "G was {}", rgba[1]);
        assert!((rgba[2] as i32 - 155).abs() <= 2, "B was {}", rgba[2]);
        assert_eq!(rgba[3], 255);
    }
}
