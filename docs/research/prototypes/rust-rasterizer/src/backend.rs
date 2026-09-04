// The only thing that differs between the two arms.
use crate::text::{PathEl, PlacedGlyph, TextShaper};

pub trait Backend {
    fn bg(&mut self, c: [u8; 4]);
    fn rect(&mut self, x: f64, y: f64, w: f64, h: f64, c: [u8; 4]);
    /// A still from disk, sampled from a source rect (in source pixels).
    /// `crop` is the centred window; `s` is the zoomed rect inside it.
    /// `crop == (0,0)` means "the whole image".
    fn still(
        &mut self,
        src: &str,
        crop: (f64, f64),
        s: (f64, f64, f64, f64),
        d: (f64, f64, f64, f64),
    );
    /// One decoded video frame, handed over as RGBA every time.
    fn video(&mut self, rgba: &[u8], w: u32, h: u32, d: (f64, f64, f64, f64));
    fn glyphs(&mut self, glyphs: &[PlacedGlyph], shaper: &TextShaper, c: [u8; 4]);
    /// Raw RGBA of the finished frame, for the encoder.
    fn pixels(&mut self) -> &[u8];
    fn name(&self) -> &'static str;
}

/// Source rect for a still: the ops give it inside a centred cropW x cropH
/// window, exactly as #6's skia-canvas arm resolved it.
pub fn src_rect(iw: u32, ih: u32, crop_w: f64, crop_h: f64, sx: f64, sy: f64) -> (f64, f64) {
    ((iw as f64 - crop_w) / 2.0 + sx, (ih as f64 - crop_h) / 2.0 + sy)
}

pub fn path_els<'a>(shaper: &'a TextShaper, g: &PlacedGlyph) -> &'a [PathEl] {
    shaper.outline(&g.key)
}
