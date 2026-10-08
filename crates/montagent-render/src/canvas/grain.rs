//! `grain` (ADR-0156 §4): a cell image hashed on the CPU, laid over the element's own layer
//! through a runtime blender.
//!
//! **Why not one runtime shader.** ADR-0156 asks for a fixed integer hash with no floats in
//! it, and the SkSL that skia-safe compiles here is ES2: `^`, `>>`, `%` and `uint` are
//! refused (#722's probe). So the draws are made here, one byte per channel per cell, into
//! an image with one pixel per cell, and the image is sampled nearest under the element's
//! matrix, scaled by the cell size and anchored at the box origin. That puts the cells in
//! element space, so they move, turn and scale with the element. A runtime blender then
//! applies the offset to the non-premultiplied colour of what the layer holds, clamps it,
//! and keeps alpha.
//!
//! **The bound.** `grain` keeps it: transparent black stays transparent black. Its layer is
//! a plain one, as a `mask`'s is, and the bound passes through it to the next blur or
//! shadow ([`super::layer_bound`]).
//!
//! **Where the cells are drawn** is decided before any layer is opened, from what the
//! element draws and the filters ahead of the grain, intersected with the device clip: the
//! same answer on every painter and with the bounds hint on or off.

use skia_safe::{
    AlphaType, Canvas as SkCanvas, ColorType, Data, FilterMode, ISize, Image, ImageFilter,
    ImageInfo, Matrix, MipmapMode, Paint, Rect, RuntimeEffect, SamplingOptions, TileMode, images,
};

use super::Effect;

/// The most cells one grain draws. Past it the member paints nothing: a 4K frame of 1×1
/// cells is about 8.3 million, so only a box scaled far below a pixel per unit reaches it.
const MAX_CELLS: i64 = 1 << 26;

/// One draw, `0`–`255`, for `channel` of the cell at `(x, y)` on local frame `frame`
/// (ADR-0156 §3): SplitMix64's finaliser applied in turn to the seed, the frame, the cell's
/// row, its column and the channel, each folded in by exclusive or, and the top byte kept.
/// Integers only, so every painter draws the same value. With `mono` only channel `0` is
/// drawn.
pub fn grain_draw(seed: u32, frame: i64, x: i64, y: i64, channel: u8) -> u8 {
    let row = row_of(seed, frame, y);
    let cell = mix(row ^ x as u64);
    (mix(cell ^ u64::from(channel)) >> 56) as u8
}

/// The part of [`grain_draw`] one row of cells shares.
fn row_of(seed: u32, frame: i64, y: i64) -> u64 {
    mix(mix(mix(u64::from(seed)) ^ frame as u64) ^ y as u64)
}

/// SplitMix64: add the golden-ratio increment, then the finaliser.
fn mix(x: u64) -> u64 {
    let mut z = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// The blender: `dst` is the layer, `src` the cell image (opaque, or transparent past it).
/// The offset is `amount × (2d − 255) / 255` per channel, on unpremultiplied colour.
const BLENDER: &str = r"
uniform float amount;

half4 main(half4 src, half4 dst) {
    float a = float(dst.a);
    if (src.a <= 0.0 || a <= 0.0) { return dst; }
    float3 rgb = float3(dst.rgb) / a;
    rgb = clamp(rgb + amount * (float3(src.rgb) * 2.0 - 1.0), 0.0, 1.0);
    return half4(half3(rgb * a), half(a));
}
";

thread_local! {
    static BLENDER_EFFECT: std::cell::OnceCell<Option<RuntimeEffect>> =
        const { std::cell::OnceCell::new() };
}

/// One grain, ready to lay over its layer.
pub(super) struct Grained {
    image: Image,
    /// The image's top-left cell.
    origin: (i64, i64),
    size: f32,
    amount: f32,
    /// The cells' extent in element space, which the draw is clipped to.
    area: Rect,
}

/// One entry per effect, `Some` for a grain that has cells to draw.
pub(super) fn plan(
    canvas: &SkCanvas,
    effects: &[Effect],
    filters: &[Option<ImageFilter>],
    draw: &dyn Fn(&SkCanvas),
) -> Vec<Option<Grained>> {
    let mut planned: Vec<Option<Grained>> = effects.iter().map(|_| None).collect();
    if !effects.iter().any(|e| matches!(e, Effect::Grain { .. })) {
        return planned;
    }
    let ctm = canvas.local_to_device_as_3x3();
    let Some(to_element) = ctm.invert() else {
        return planned;
    };
    let Some(clip) = canvas.device_clip_bounds() else {
        return planned;
    };
    let clip = to_element.map_rect(Rect::from(clip)).0;
    let Some(mut content) = super::layer_bound::drawn(canvas, draw) else {
        return planned;
    };
    for ((effect, filter), slot) in effects.iter().zip(filters).zip(&mut planned) {
        if let Effect::Grain {
            seed,
            amount,
            size,
            mono,
            frame,
        } = *effect
        {
            let mut area = content;
            if area.intersect(clip) {
                *slot = cells(area, seed, amount, size, mono, frame);
            }
        } else if let Some(filter) = filter {
            content = filter.compute_fast_bounds(content);
        }
    }
    planned
}

/// The cell image covering `area`, with a cell of margin on every side.
fn cells(area: Rect, seed: u32, amount: f64, size: u32, mono: bool, frame: i64) -> Option<Grained> {
    let side = f64::from(size.max(1));
    let first = |edge: f32| (f64::from(edge) / side).floor() as i64 - 1;
    let last = |edge: f32| (f64::from(edge) / side).floor() as i64 + 1;
    let (x0, y0) = (first(area.left), first(area.top));
    let (x1, y1) = (last(area.right), last(area.bottom));
    let (width, height) = (x1 - x0 + 1, y1 - y0 + 1);
    if width <= 0 || height <= 0 || width.checked_mul(height)? > MAX_CELLS {
        return None;
    }
    let mut pixels = Vec::with_capacity((width * height * 4) as usize);
    for y in y0..=y1 {
        let row = row_of(seed, frame, y);
        for x in x0..=x1 {
            let cell = mix(row ^ x as u64);
            let draw = |channel: u8| (mix(cell ^ u64::from(channel)) >> 56) as u8;
            let r = draw(0);
            let (g, b) = if mono { (r, r) } else { (draw(1), draw(2)) };
            pixels.extend_from_slice(&[r, g, b, 0xFF]);
        }
    }
    let info = ImageInfo::new(
        ISize::new(i32::try_from(width).ok()?, i32::try_from(height).ok()?),
        ColorType::RGBA8888,
        AlphaType::Premul,
        None,
    );
    let image = images::raster_from_data(&info, Data::new_copy(&pixels), width as usize * 4)?;
    let side = side as f32;
    Some(Grained {
        image,
        origin: (x0, y0),
        size: side,
        amount: amount as f32,
        area: Rect::new(
            x0 as f32 * side,
            y0 as f32 * side,
            (x1 + 1) as f32 * side,
            (y1 + 1) as f32 * side,
        ),
    })
}

impl Grained {
    /// Lay the grain over the layer `canvas` is drawing into, in element space.
    pub(super) fn apply(&self, canvas: &SkCanvas) {
        // Compiled once per painter thread, as the other named effects are.
        let Some(blender) = super::named::compiled(&BLENDER_EFFECT, || {
            RuntimeEffect::make_for_blender(BLENDER, None).ok()
        })
        .and_then(|effect| effect.make_blender(Data::new_copy(&self.amount.to_ne_bytes()), None)) else {
            return;
        };
        let mut local = Matrix::translate((
            self.origin.0 as f32 * self.size,
            self.origin.1 as f32 * self.size,
        ));
        local.pre_scale((self.size, self.size), None);
        let Some(shader) = self.image.to_shader(
            (TileMode::Decal, TileMode::Decal),
            SamplingOptions::new(FilterMode::Nearest, MipmapMode::None),
            &local,
        ) else {
            return;
        };
        let mut paint = Paint::default();
        paint.set_shader(shader);
        paint.set_blender(blender);
        canvas.save();
        canvas.clip_rect(self.area, None, Some(false));
        canvas.draw_paint(&paint);
        canvas.restore();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_mix_is_splitmix64() {
        // SplitMix64's first output from state 0, the published reference value.
        assert_eq!(mix(0), 0xE220_A839_7B1D_CDAF);
    }

    #[test]
    fn a_draw_is_a_fixed_function_of_its_five_integers() {
        // Values from an independent reading of the formula `compositing.md` publishes, so a
        // change to the hash, which changes every grain ever rendered, fails here first.
        assert_eq!(grain_draw(7, 0, 0, 0, 0), grain_draw(7, 0, 0, 0, 0));
        let draws: Vec<u8> = (0..4).map(|x| grain_draw(7, 0, x, 0, 0)).collect();
        assert_eq!(draws, [11, 19, 73, 20]);
        // Negative frames and cells fold in as their two's-complement bits.
        assert_eq!(grain_draw(7, -1, -3, -2, 2), 37);
        assert_ne!(grain_draw(7, 0, 0, 0, 0), grain_draw(8, 0, 0, 0, 0));
        assert_ne!(grain_draw(7, 0, 0, 0, 0), grain_draw(7, 1, 0, 0, 0));
    }
}
