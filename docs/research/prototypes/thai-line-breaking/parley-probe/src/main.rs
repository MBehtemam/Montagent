//! Lay every shared sample out through parley at a fixed box width and print
//! where the lines actually broke.
//!
//! Build twice -- once plain, once with `--features complex-scripts` -- and diff
//! the two outputs. That diff is the whole question this prototype exists to
//! answer, because the feature is the only thing that changes between the runs.
//!
//! The two overflow-wrap modes mirror cosmic-probe's two wrap modes:
//!   Normal    ~ cosmic Wrap::Word        (no break inside a "word")
//!   BreakWord ~ cosmic Wrap::WordOrGlyph (fall back to breaking anywhere)

#[path = "../../samples.rs"]
mod samples;

#[path = "../../render.rs"]
mod render;

use std::sync::Arc;

use parley::{
    FontContext, FontFamily, LayoutContext, OverflowWrap, PositionedLayoutItem, StyleProperty,
};

use render::PlacedGlyph;

const PAD: f32 = 20.0;

fn main() {
    let mode = std::env::args().nth(1).unwrap_or_else(|| "BreakWord".to_string());
    let overflow_wrap = match mode.as_str() {
        "Normal" => OverflowWrap::Normal,
        "BreakWord" => OverflowWrap::BreakWord,
        "Anywhere" => OverflowWrap::Anywhere,
        other => panic!("unknown overflow-wrap mode {other}"),
    };

    let width_override: Option<f32> = std::env::args().nth(2).map(|w| w.parse().expect("width must be a number"));
    let png_dir = std::env::args().nth(3);
    let complex = cfg!(feature = "complex-scripts");
    println!("# stack: parley 0.11.1");
    println!("# complex-scripts: {complex}");
    println!("# overflow-wrap: {mode}");

    let mut fcx = FontContext::new();
    let mut lcx: LayoutContext<[u8; 4]> = LayoutContext::new();

    for s in samples::SAMPLES {
        let width = width_override.unwrap_or(s.width);
        let mut builder = lcx.ranged_builder(&mut fcx, s.text, 1.0, false);
        builder.push_default(StyleProperty::FontSize(s.size));
        builder.push_default(StyleProperty::FontFamily(FontFamily::named(s.family)));
        builder.push_default(StyleProperty::OverflowWrap(overflow_wrap));
        let mut layout = builder.build(s.text);
        layout.break_all_lines(Some(width));

        println!("\n## {} (family {}, size {}, width {})", s.id, s.family, s.size, width);
        println!("expect: {}", s.expect);

        let mut starts: Vec<usize> = Vec::new();
        let mut notdef = 0usize;
        let mut overflow = 0usize;
        let mut placed: Vec<PlacedGlyph> = Vec::new();

        for line in layout.lines() {
            let range = line.text_range();
            let advance = line.metrics().advance;
            starts.push(range.start);
            if advance > width + 0.5 {
                overflow += 1;
            }
            for run in line.runs() {
                notdef += run.clusters().flat_map(|c| c.glyphs()).filter(|g| g.id == 0).count();
            }
            for item in line.items() {
                let PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                    continue;
                };
                let font = glyph_run.run().font().clone();
                let data = Arc::new(font.data.as_ref().to_vec());
                let size_px = glyph_run.run().font_size();
                for g in glyph_run.positioned_glyphs() {
                    placed.push(PlacedGlyph {
                        font_data: data.clone(),
                        font_index: font.index,
                        glyph_id: g.id as u16,
                        x: g.x + PAD,
                        y: g.y + PAD,
                        size_px,
                    });
                }
            }
            println!(
                "line w={:7.2} [{:>3}..{:>3}] {:?}  because {:?}",
                advance,
                range.start,
                range.end,
                s.text[range.start..range.end].trim_end_matches('\n'),
                line.break_reason(),
            );
        }

        println!("breaks: {starts:?}");
        println!("lines: {}  overflowing: {}  notdef-glyphs: {}", starts.len(), overflow, notdef);

        if let Some(dir) = &png_dir {
            let tag = if complex { "on" } else { "off" };
            let path = format!("{dir}/parley-{tag}-{mode}-{}.png", s.id);
            let img_w = (width.max(1050.0) + PAD * 2.0).ceil() as u32;
            let img_h = (layout.height() + PAD * 2.0).ceil() as u32;
            render::render_png(&placed, img_w, img_h, width + PAD, &path)
                .unwrap_or_else(|e| panic!("render {path}: {e}"));
        }
    }
}
