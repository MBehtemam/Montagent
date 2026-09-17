//! Ticket #130: does size x line_height (ADR-0007/0028) and the half-leading
//! baseline rule (ADR-0029) actually contain a stacking-diacritic script's
//! real ink, the way ADR-0011 measured it does for Latin?
//!
//! Method, mirroring ADR-0011's own methodology (pixel-scan the real ink,
//! don't trust nominal metrics): lay real paragraphs out through parley
//! (complex-scripts on, matching #27/#28's accepted config), read each line's
//! font ascent/descent from parley's own LineMetrics, place every line's
//! baseline with ADR-0029's formula, then extract every glyph's *outline*
//! (skrifa) rather than its advance box, and take the min/max y across all of
//! it. That is the same "real ink" measurement ADR-0011 did by pixel-scanning
//! rendered frames -- done here from the vector outlines directly, which is
//! exact rather than sampled.
//!
//! No auto-wrap: `\n` is placed by hand in every sample, per ADR-0008.

use std::collections::HashMap;
use std::sync::Arc;

use parley::{FontContext, FontFamily, LayoutContext, PositionedLayoutItem, StyleProperty};
use skrifa::{
    instance::{LocationRef, Size},
    outline::{DrawSettings, OutlinePen},
    prelude::*,
    FontRef, MetadataProvider,
};
use tiny_skia::{FillRule, Paint, PathBuilder, Pixmap, Transform};

struct Sample {
    id: &'static str,
    family: &'static str,
    size: f32,
    /// Tenths, per ADR-0028's restriction.
    line_height: f32,
    text: &'static str,
}

// Real captioning-register Thai, chosen for above-base tone marks (่ ้ ๊ ๋)
// and above/below vowel signs (ิ ี ึ ื ุ ู) stacking on ordinary base
// consonants -- not a cherry-picked pathological string, the ordinary shape
// of a Thai sentence. `\n` placed by hand, as ADR-0008 requires of an author,
// at a plausible clause break.
const SAMPLES: &[Sample] = &[
    Sample {
        id: "thai-2line-lh11",
        family: "Ayuthaya",
        size: 55.0,
        line_height: 1.1,
        text: "ที่พักผ่อนสุดหรูสำหรับนักท่องเที่ยว\nที่ต้องการความสงบและใกล้ชิดธรรมชาติ",
    },
    Sample {
        id: "thai-2line-lh12",
        family: "Ayuthaya",
        size: 55.0,
        line_height: 1.2,
        text: "ที่พักผ่อนสุดหรูสำหรับนักท่องเที่ยว\nที่ต้องการความสงบและใกล้ชิดธรรมชาติ",
    },
    Sample {
        id: "thai-3line-lh11",
        family: "Ayuthaya",
        size: 55.0,
        line_height: 1.1,
        text: "รู้สึกดีใจที่ได้กลับมาเที่ยว\nที่ประเทศไทยอีกครั้งหนึ่ง\nหลังจากที่ห่างหายไปนานมาก",
    },
    // The exact control ADR-0011 measured: same size, same line_height, same
    // methodology, Latin script. Confirms this probe reproduces the
    // established "nominal overstates ink" direction before trusting it on
    // Thai.
    Sample {
        id: "latin-control-2line-lh11",
        family: "Helvetica",
        size: 55.0,
        line_height: 1.1,
        text: "The quiet cabin by the lake\nis perfect for a long weekend",
    },
];

struct PlacedGlyph {
    font_data: Arc<Vec<u8>>,
    font_index: u32,
    glyph_id: u16,
    x: f32,
    y: f32, // absolute baseline y within the block, top-anchored at 0
    size_px: f32,
}

/// Tracks the ink bounding box (screen space, y down) instead of drawing.
struct BBoxPen {
    ox: f32,
    oy: f32,
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
    any: bool,
}

impl BBoxPen {
    fn new(ox: f32, oy: f32) -> Self {
        Self { ox, oy, min_x: f32::MAX, max_x: f32::MIN, min_y: f32::MAX, max_y: f32::MIN, any: false }
    }
    fn visit(&mut self, x: f32, y: f32) {
        let (sx, sy) = (self.ox + x, self.oy - y);
        self.min_x = self.min_x.min(sx);
        self.max_x = self.max_x.max(sx);
        self.min_y = self.min_y.min(sy);
        self.max_y = self.max_y.max(sy);
        self.any = true;
    }
}

impl OutlinePen for BBoxPen {
    fn move_to(&mut self, x: f32, y: f32) {
        self.visit(x, y);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.visit(x, y);
    }
    fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        self.visit(cx, cy);
        self.visit(x, y);
    }
    fn curve_to(&mut self, c1x: f32, c1y: f32, c2x: f32, c2y: f32, x: f32, y: f32) {
        self.visit(c1x, c1y);
        self.visit(c2x, c2y);
        self.visit(x, y);
    }
    fn close(&mut self) {}
}

fn ink_bbox(glyphs: &[PlacedGlyph]) -> Option<(f32, f32, f32, f32)> {
    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    let mut min_y = f32::MAX;
    let mut max_y = f32::MIN;
    let mut any = false;
    let mut cache: HashMap<(usize, u32), Arc<Vec<u8>>> = HashMap::new();
    for g in glyphs {
        let key = (Arc::as_ptr(&g.font_data) as usize, g.font_index);
        let data = cache.entry(key).or_insert_with(|| g.font_data.clone()).clone();
        let Ok(font) = FontRef::from_index(data.as_slice(), g.font_index) else { continue };
        let outlines = font.outline_glyphs();
        let Some(outline) = outlines.get(GlyphId::from(g.glyph_id)) else { continue };
        let mut pen = BBoxPen::new(g.x, g.y);
        let settings = DrawSettings::unhinted(Size::new(g.size_px), LocationRef::default());
        if outline.draw(settings, &mut pen).is_err() || !pen.any {
            continue;
        }
        min_x = min_x.min(pen.min_x);
        max_x = max_x.max(pen.max_x);
        min_y = min_y.min(pen.min_y);
        max_y = max_y.max(pen.max_y);
        any = true;
    }
    any.then_some((min_x, max_x, min_y, max_y))
}

fn render_debug_png(glyphs: &[PlacedGlyph], width: u32, height: u32, slot_ys: &[f32], path: &str) {
    let Some(mut pixmap) = Pixmap::new(width, height) else { return };
    pixmap.fill(tiny_skia::Color::WHITE);

    let mut slot_line = Paint::default();
    slot_line.set_color_rgba8(120, 170, 230, 255);
    for &y in slot_ys {
        if let Some(rect) = tiny_skia::Rect::from_xywh(0.0, y.max(0.0), width as f32, 1.0) {
            pixmap.fill_rect(rect, &slot_line, Transform::identity(), None);
        }
    }

    let mut ink = Paint::default();
    ink.set_color_rgba8(17, 17, 17, 255);
    ink.anti_alias = true;
    let mut cache: HashMap<(usize, u32), Arc<Vec<u8>>> = HashMap::new();
    for g in glyphs {
        let key = (Arc::as_ptr(&g.font_data) as usize, g.font_index);
        let data = cache.entry(key).or_insert_with(|| g.font_data.clone()).clone();
        let Ok(font) = FontRef::from_index(data.as_slice(), g.font_index) else { continue };
        let outlines = font.outline_glyphs();
        let Some(outline) = outlines.get(GlyphId::from(g.glyph_id)) else { continue };
        let mut builder = PathBuilder::new();
        struct DrawPen<'a> { b: &'a mut PathBuilder, ox: f32, oy: f32 }
        impl OutlinePen for DrawPen<'_> {
            fn move_to(&mut self, x: f32, y: f32) { self.b.move_to(self.ox + x, self.oy - y); }
            fn line_to(&mut self, x: f32, y: f32) { self.b.line_to(self.ox + x, self.oy - y); }
            fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) { self.b.quad_to(self.ox + cx, self.oy - cy, self.ox + x, self.oy - y); }
            fn curve_to(&mut self, c1x: f32, c1y: f32, c2x: f32, c2y: f32, x: f32, y: f32) { self.b.cubic_to(self.ox + c1x, self.oy - c1y, self.ox + c2x, self.oy - c2y, self.ox + x, self.oy - y); }
            fn close(&mut self) { self.b.close(); }
        }
        let mut pen = DrawPen { b: &mut builder, ox: g.x, oy: g.y };
        let settings = DrawSettings::unhinted(Size::new(g.size_px), LocationRef::default());
        if outline.draw(settings, &mut pen).is_err() { continue; }
        if let Some(p) = builder.finish() {
            pixmap.fill_path(&p, &ink, FillRule::Winding, Transform::identity(), None);
        }
    }
    let _ = pixmap.save_png(path);
}

fn main() {
    let png_dir = std::env::args().nth(1);
    if let Some(dir) = &png_dir {
        let _ = std::fs::create_dir_all(dir);
    }

    let mut fcx = FontContext::new();
    let mut lcx: LayoutContext<[u8; 4]> = LayoutContext::new();

    for s in SAMPLES {
        println!("\n## {} (family {}, size {}, line_height {})", s.id, s.family, s.size, s.line_height);

        // ADR-0008: Montaget owns the line partition and splits on mandatory
        // breaks itself, before handing a *line* to the shaper -- it does not
        // hand a multi-line paragraph to the shaper and rely on the shaper's
        // own hard-break handling. So each `\n`-delimited line here gets its
        // own single-line parley layout, exactly as the renderer must.
        let raw_lines: Vec<&str> = s.text.split('\n').collect();

        let slot_height = s.size * s.line_height;
        let line_count = raw_lines.len();
        let block_height = (slot_height * line_count as f32).ceil();
        println!("slot_height (size*line_height) = {slot_height:.2}  declared block height (ceil, ADR-0014/0028) = {block_height}");

        let mut all_glyphs: Vec<PlacedGlyph> = Vec::new();
        let mut per_line: Vec<(f32, f32, f32, f32, f32, f32, f32)> = Vec::new(); // slot_top, slot_bottom, baseline_y, ascent, descent, ink_top, ink_bottom
        let mut max_x = 0f32;

        for (i, line_text) in raw_lines.iter().enumerate() {
            let mut builder = lcx.ranged_builder(&mut fcx, line_text, 1.0, false);
            builder.push_default(StyleProperty::FontSize(s.size));
            builder.push_default(StyleProperty::FontFamily(FontFamily::named(s.family)));
            let mut layout = builder.build(line_text);
            layout.break_all_lines(None); // single line, no wrap
            let line = layout.lines().next().expect("one line per split");

            let lm = line.metrics();
            let ascent = lm.ascent;
            let descent = lm.descent;
            let slot_top = i as f32 * slot_height;
            let slot_bottom = (i as f32 + 1.0) * slot_height;
            let slot_centre = (slot_top + slot_bottom) / 2.0;
            // ADR-0029: baseline_y = slot_centre + (ascent - descent) / 2
            let baseline_y = slot_centre + (ascent - descent) / 2.0;

            let mut line_glyphs: Vec<PlacedGlyph> = Vec::new();
            for item in line.items() {
                let PositionedLayoutItem::GlyphRun(glyph_run) = item else { continue };
                let font = glyph_run.run().font().clone();
                let data = Arc::new(font.data.as_ref().to_vec());
                let size_px = glyph_run.run().font_size();
                for g in glyph_run.positioned_glyphs() {
                    max_x = max_x.max(g.x);
                    line_glyphs.push(PlacedGlyph {
                        font_data: data.clone(),
                        font_index: font.index,
                        glyph_id: g.id as u16,
                        x: g.x,
                        y: baseline_y,
                        size_px,
                    });
                }
            }

            let (ink_top, ink_bottom) = match ink_bbox(&line_glyphs) {
                Some((_, _, min_y, max_y)) => (min_y, max_y),
                None => (baseline_y, baseline_y),
            };

            println!(
                "line {i}: slot=[{slot_top:.2},{slot_bottom:.2}] baseline_y={baseline_y:.2} ascent={ascent:.2} descent={descent:.2}  real ink=[{ink_top:.2},{ink_bottom:.2}]  \
                 overshoot_above_own_slot={:.2}  overshoot_below_own_slot={:.2}",
                slot_top - ink_top,
                ink_bottom - slot_bottom,
            );

            per_line.push((slot_top, slot_bottom, baseline_y, ascent, descent, ink_top, ink_bottom));
            all_glyphs.extend(line_glyphs);
        }

        // Ground truth: does line i's real ink physically overlap line i+1's
        // real ink in the rendered image, independent of slot bookkeeping?
        for i in 0..per_line.len().saturating_sub(1) {
            let ink_bottom_i = per_line[i].6;
            let ink_top_next = per_line[i + 1].5;
            let overlap = ink_bottom_i - ink_top_next;
            println!(
                "seam {i}/{}: line {i} ink bottom={ink_bottom_i:.2}  line {} ink top={ink_top_next:.2}  ink-to-ink overlap={overlap:.2}{}",
                i + 1, i + 1,
                if overlap > 0.0 { "  <-- COLLIDES" } else { "" },
            );
        }
        if let (Some(first), Some(last)) = (per_line.first(), per_line.last()) {
            let top_escape = -first.5; // ink_top negative means above block top (y=0)
            let bottom_escape = last.6 - block_height;
            println!(
                "block edges: top ink={:.2} (escapes box top by {:.2}{})  bottom ink={:.2} (escapes box bottom by {:.2}{})",
                first.5, top_escape, if top_escape > 0.0 { " <-- ESCAPES" } else { "" },
                last.6, bottom_escape, if bottom_escape > 0.0 { " <-- ESCAPES" } else { "" },
            );
        }

        if let Some(dir) = &png_dir {
            let slot_ys: Vec<f32> = (0..=line_count).map(|i| i as f32 * slot_height).collect();
            let path = format!("{dir}/{}.png", s.id);
            render_debug_png(&all_glyphs, (max_x + 40.0).ceil() as u32, block_height.ceil() as u32 + 20, &slot_ys, &path);
        }
    }
}
