// Text: parley lays out and positions the glyphs, skrifa scales the outlines,
// the rasterizer fills the paths. Settled by ADR-0009 — not what #34 tests, so
// this module is shared verbatim by both backends.
//
// No automatic wrapping (ADR-0007): each `\N`-separated line is one parley
// layout with no wrap width. Block placement (the ASS \an anchor and the
// size*1.2 leading) is #6's ops.js math, kept so the frames are comparable.
use parley::{
    Alignment, AlignmentOptions, FontContext, FontFamily, Layout, LayoutContext,
    PositionedLayoutItem, StyleProperty,
};
use skrifa::instance::{LocationRef, Size};
use skrifa::outline::{DrawSettings, OutlinePen};
use skrifa::{FontRef, GlyphId, MetadataProvider};
use std::collections::HashMap;

/// Backend-neutral path, in device pixels, already translated to its glyph origin.
#[derive(Clone, Copy, Debug)]
pub enum PathEl {
    Move(f32, f32),
    Line(f32, f32),
    Quad(f32, f32, f32, f32),
    Cubic(f32, f32, f32, f32, f32, f32),
    Close,
}

#[derive(Default)]
struct Pen {
    els: Vec<PathEl>,
}

impl OutlinePen for Pen {
    fn move_to(&mut self, x: f32, y: f32) {
        self.els.push(PathEl::Move(x, -y));
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.els.push(PathEl::Line(x, -y));
    }
    fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        self.els.push(PathEl::Quad(cx, -cy, x, -y));
    }
    fn curve_to(&mut self, c1x: f32, c1y: f32, c2x: f32, c2y: f32, x: f32, y: f32) {
        self.els.push(PathEl::Cubic(c1x, -c1y, c2x, -c2y, x, -y));
    }
    fn close(&mut self) {
        self.els.push(PathEl::Close);
    }
}

/// One glyph, positioned in device space, with its outline at its own origin.
pub struct PlacedGlyph {
    pub x: f32,
    pub y: f32,
    pub key: (usize, u32, u32), // (font, glyph id, size bits) — the backends' cache key
}

pub struct TextShaper {
    font_cx: FontContext,
    layout_cx: LayoutContext<()>,
    fonts: Vec<(Vec<u8>, u32)>,
    font_ix: HashMap<(usize, u32), usize>,
    outlines: HashMap<(usize, u32, u32), Vec<PathEl>>,
}

impl TextShaper {
    pub fn new() -> Self {
        Self {
            font_cx: FontContext::new(),
            layout_cx: LayoutContext::new(),
            fonts: Vec::new(),
            font_ix: HashMap::new(),
            outlines: HashMap::new(),
        }
    }

    /// The outline for a cache key. Populated by `place`; both backends read it.
    pub fn outline(&self, key: &(usize, u32, u32)) -> &[PathEl] {
        self.outlines.get(key).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Lay out one text op and return every glyph, positioned in device space.
    pub fn place(
        &mut self,
        lines: &[String],
        family: &str,
        size: f64,
        x: f64,
        y: f64,
        align: u8,
    ) -> Vec<PlacedGlyph> {
        let (horiz, vert) = crate::ops::anchors(align);
        let lh = size * 1.2; // ops.js: leading is size * 1.2
        let block = (lines.len() as f64 - 1.0) * lh;

        let mut out = Vec::new();
        for (i, line_text) in lines.iter().enumerate() {
            let mut builder =
                self.layout_cx
                    .ranged_builder(&mut self.font_cx, line_text, 1.0, true);
            builder.push_default(StyleProperty::FontFamily(FontFamily::named(family)));
            builder.push_default(StyleProperty::FontSize(size as f32));
            let mut layout: Layout<()> = builder.build(line_text);
            layout.break_all_lines(None); // no wrap width — ADR-0007
            layout.align(Alignment::Start, AlignmentOptions::default());

            let width = layout.width() as f64;
            let (ascent, descent) = layout
                .lines()
                .next()
                .map(|l| (l.metrics().ascent as f64, l.metrics().descent as f64))
                .unwrap_or((size * 0.8, size * 0.2));

            // canvas textAlign
            let ox = match horiz {
                0 => x,
                1 => x - width / 2.0,
                _ => x - width,
            };
            // canvas textBaseline=middle, then the block anchored by \an
            let centre = match vert {
                0 => y + lh / 2.0 + i as f64 * lh,
                2 => y - block - lh / 2.0 + i as f64 * lh,
                _ => y - block / 2.0 + i as f64 * lh,
            };
            let baseline = centre + (ascent - descent) / 2.0;

            for pl in layout.lines() {
                for item in pl.items() {
                    let PositionedLayoutItem::GlyphRun(run) = item else {
                        continue;
                    };
                    let fd = run.run().font().clone();
                    let fkey = (fd.data.as_ref().as_ptr() as usize, fd.index);
                    let fi = *self.font_ix.entry(fkey).or_insert_with(|| {
                        self.fonts.push((fd.data.as_ref().to_vec(), fd.index));
                        self.fonts.len() - 1
                    });
                    let run_size = run.run().font_size();
                    for g in run.positioned_glyphs() {
                        let key = (fi, g.id, run_size.to_bits());
                        if !self.outlines.contains_key(&key) {
                            let (data, index) = &self.fonts[fi];
                            let mut pen = Pen::default();
                            if let Ok(font) = FontRef::from_index(data, *index) {
                                if let Some(og) = font.outline_glyphs().get(GlyphId::new(g.id)) {
                                    let _ = og.draw(
                                        DrawSettings::unhinted(
                                            Size::new(run_size),
                                            LocationRef::default(),
                                        ),
                                        &mut pen,
                                    );
                                }
                            }
                            self.outlines.insert(key, pen.els);
                        }
                        out.push(PlacedGlyph {
                            x: ox as f32 + g.x,
                            // parley's y is the baseline within its own layout; we
                            // supply the baseline ourselves, so drop parley's.
                            y: baseline as f32 + (g.y - pl.metrics().baseline),
                            key,
                        });
                    }
                }
            }
        }
        out
    }
}
