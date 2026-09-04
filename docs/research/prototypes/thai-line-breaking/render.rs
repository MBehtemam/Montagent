//! The one shared rasterizer.
//!
//! Both probes hand this module the same thing -- a list of positioned glyphs
//! plus the font blob each came from -- and it draws them with skrifa outlines
//! into a tiny-skia pixmap. Neither stack rasterizes its own output, so a
//! difference between two PNGs is the line breaking and not the renderer.

use std::collections::HashMap;

use skrifa::{
    instance::{LocationRef, Size},
    outline::{DrawSettings, OutlinePen},
    prelude::*,
    FontRef, MetadataProvider,
};
use tiny_skia::{FillRule, Paint, PathBuilder, Pixmap, Transform};

pub struct PlacedGlyph {
    /// Whole font file, plus the face index if it is a collection.
    pub font_data: std::sync::Arc<Vec<u8>>,
    pub font_index: u32,
    pub glyph_id: u16,
    /// Pen position, in pixels, origin at the top left of the image.
    pub x: f32,
    pub y: f32,
    pub size_px: f32,
}

struct Outliner {
    builder: PathBuilder,
    ox: f32,
    oy: f32,
}

// skrifa hands out outlines in font space with y pointing up; tiny-skia wants
// y pointing down from the top-left, so every y is negated and offset by the
// glyph's baseline position.
impl OutlinePen for Outliner {
    fn move_to(&mut self, x: f32, y: f32) {
        self.builder.move_to(self.ox + x, self.oy - y);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.builder.line_to(self.ox + x, self.oy - y);
    }
    fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        self.builder
            .quad_to(self.ox + cx, self.oy - cy, self.ox + x, self.oy - y);
    }
    fn curve_to(&mut self, c1x: f32, c1y: f32, c2x: f32, c2y: f32, x: f32, y: f32) {
        self.builder.cubic_to(
            self.ox + c1x,
            self.oy - c1y,
            self.ox + c2x,
            self.oy - c2y,
            self.ox + x,
            self.oy - y,
        );
    }
    fn close(&mut self) {
        self.builder.close();
    }
}

/// Draw `glyphs` into a `width` x `height` PNG at `path`, with a guide line
/// down the right edge of the text box so an overflowing line is unmissable.
pub fn render_png(
    glyphs: &[PlacedGlyph],
    width: u32,
    height: u32,
    box_width: f32,
    path: &str,
) -> Result<(), String> {
    let mut pixmap = Pixmap::new(width, height).ok_or("bad pixmap size")?;
    pixmap.fill(tiny_skia::Color::WHITE);

    // The box edge. Anything drawn to the right of this line did not fit.
    let mut guide = Paint::default();
    guide.set_color_rgba8(220, 80, 80, 255);
    if let Some(rect) = tiny_skia::Rect::from_xywh(box_width, 0.0, 1.0, height as f32) {
        pixmap.fill_rect(rect, &guide, Transform::identity(), None);
    }

    let mut ink = Paint::default();
    ink.set_color_rgba8(17, 17, 17, 255);
    ink.anti_alias = true;

    // One FontRef per distinct (blob, index); building it per glyph is wasteful
    // and the blobs are large.
    let mut cache: HashMap<(usize, u32), std::sync::Arc<Vec<u8>>> = HashMap::new();

    for g in glyphs {
        let key = (std::sync::Arc::as_ptr(&g.font_data) as usize, g.font_index);
        let data = cache.entry(key).or_insert_with(|| g.font_data.clone()).clone();
        let Ok(font) = FontRef::from_index(data.as_slice(), g.font_index) else {
            continue;
        };
        let outlines = font.outline_glyphs();
        let Some(outline) = outlines.get(GlyphId::from(g.glyph_id)) else {
            continue;
        };
        let mut pen = Outliner {
            builder: PathBuilder::new(),
            ox: g.x,
            oy: g.y,
        };
        let settings = DrawSettings::unhinted(Size::new(g.size_px), LocationRef::default());
        if outline.draw(settings, &mut pen).is_err() {
            continue;
        }
        if let Some(p) = pen.builder.finish() {
            pixmap.fill_path(&p, &ink, FillRule::Winding, Transform::identity(), None);
        }
    }

    pixmap.save_png(path).map_err(|e| e.to_string())
}
