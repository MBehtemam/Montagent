//! Lay every shared sample out through cosmic-text at a fixed box width and
//! print where the lines actually broke.
//!
//! Run once per wrap mode, because the two modes fail differently when a stack
//! has no segmentation for the script:
//!
//!   Word        -- a scriptless Thai paragraph is one 67-character "word", so
//!                  it cannot wrap at all and overflows the box.
//!   WordOrGlyph -- the same paragraph falls back to breaking between glyphs,
//!                  anywhere, including inside a word.

#[path = "../../samples.rs"]
mod samples;

#[path = "../../render.rs"]
mod render;

use std::sync::Arc;

use cosmic_text::{Attrs, Buffer, Family, FontSystem, Metrics, Shaping, Wrap};

use render::PlacedGlyph;

const PAD: f32 = 20.0;

fn main() {
    let mode = std::env::args().nth(1).unwrap_or_else(|| "WordOrGlyph".to_string());
    let wrap = match mode.as_str() {
        "Word" => Wrap::Word,
        "Glyph" => Wrap::Glyph,
        "WordOrGlyph" => Wrap::WordOrGlyph,
        other => panic!("unknown wrap mode {other}"),
    };

    let width_override: Option<f32> = std::env::args().nth(2).map(|w| w.parse().expect("width must be a number"));
    let png_dir = std::env::args().nth(3);
    let mut fs = FontSystem::new();
    println!("# stack: cosmic-text 0.19.0");
    println!("# wrap: {mode}");

    for s in samples::SAMPLES {
        let width = width_override.unwrap_or(s.width);
        let mut buf = Buffer::new(&mut fs, Metrics::new(s.size, s.size * 1.35));
        buf.set_wrap(wrap);
        buf.set_size(Some(width), None);
        buf.set_text(
            s.text,
            &Attrs::new().family(Family::Name(s.family)),
            Shaping::Advanced,
            None,
        );
        buf.shape_until_scroll(&mut fs, false);

        println!("\n## {} (family {}, size {}, width {})", s.id, s.family, s.size, width);
        println!("expect: {}", s.expect);

        let mut starts: Vec<usize> = Vec::new();
        let mut notdef = 0usize;
        let mut fonts: Vec<String> = Vec::new();
        let mut overflow = 0usize;
        let mut placed: Vec<PlacedGlyph> = Vec::new();
        let mut bottom = 0.0f32;

        for run in buf.layout_runs() {
            // LayoutRun::text is the whole source paragraph, not the visual
            // line, so the visual line's byte range comes from its glyphs.
            let Some(first) = run.glyphs.first() else { continue };
            let lo = run.glyphs.iter().map(|g| g.start).min().unwrap_or(first.start);
            let hi = run.glyphs.iter().map(|g| g.end).max().unwrap_or(first.end);
            starts.push(lo);
            notdef += run.glyphs.iter().filter(|g| g.glyph_id == 0).count();
            if run.line_w > width + 0.5 {
                overflow += 1;
            }
            bottom = bottom.max(run.line_top + run.line_height);
            for g in run.glyphs {
                if let Some(data) = fs.db().with_face_data(g.font_id, |data, index| {
                    (Arc::new(data.to_vec()), index)
                }) {
                    placed.push(PlacedGlyph {
                        font_data: data.0,
                        font_index: data.1,
                        glyph_id: g.glyph_id,
                        x: g.x + PAD,
                        y: run.line_y + g.y + PAD,
                        size_px: g.font_size,
                    });
                }
                if let Some(face) = fs.db().face(g.font_id) {
                    let name = face.families.first().map(|f| f.0.clone()).unwrap_or_default();
                    if !fonts.contains(&name) {
                        fonts.push(name);
                    }
                }
            }
            println!(
                "line w={:7.2} [{:>3}..{:>3}] {:?}",
                run.line_w,
                lo,
                hi,
                &s.text[lo..hi]
            );
        }

        println!("breaks: {starts:?}");
        println!("lines: {}  overflowing: {}  notdef-glyphs: {}", starts.len(), overflow, notdef);
        println!("resolved-fonts: {}", fonts.join(", "));

        if let Some(dir) = &png_dir {
            let path = format!("{dir}/cosmic-{mode}-{}.png", s.id);
            let img_w = (width.max(1050.0) + PAD * 2.0).ceil() as u32;
            let img_h = (bottom + PAD * 2.0).ceil() as u32;
            render::render_png(&placed, img_w, img_h, width + PAD, &path)
                .unwrap_or_else(|e| panic!("render {path}: {e}"));
        }
    }
}
