//! Ticket #325: which of three candidates repairs Thai's `line_height`
//! collision — OS/2 typo metrics, a script-aware floor, or font selection?
//!
//! Extends #130's probe (`prototype/thai-vertical-metrics`, Ayuthaya) in four
//! ways, all of which #130 explicitly left unsettled:
//!
//!   1. The faces are **vendorable** — OFL-1.1 Noto Sans Thai and Sarabun,
//!      loaded from `../fonts/` by path exactly as ADR-0007 requires, never
//!      from the system font set. #130 measured a macOS system font (Ayuthaya)
//!      that `fonts vendor` would never copy.
//!   2. It prints, per face, the **raw table numbers** — `hhea`
//!      ascender/descender/lineGap, `OS/2` sTypoAscender/sTypoDescender/
//!      sTypoLineGap, `OS/2` usWinAscent/usWinDescent, and the fsSelection
//!      bit 7 USE_TYPO_METRICS flag — read straight off the binary through
//!      read-fonts, alongside what `skrifa::metrics::Metrics::new` resolves
//!      and what parley then reports. That triple is candidate 1's whole
//!      question, answered by inspection rather than argument.
//!   3. It **sweeps `line_height`** over ADR-0028's tenths and reports the
//!      first tenth at which no seam collides, instead of testing two values.
//!   4. It reports, for each face, the `line_height` each candidate metric
//!      source would imply if the slot were derived from the font.
//!
//! Method is otherwise #130's, unchanged, so the numbers are comparable:
//! lay each `\n`-delimited line out as its own single-line parley layout
//! (ADR-0008 — Montagent owns the line partition), place the baseline with
//! ADR-0029's half-leading formula, then take real ink from every glyph's
//! skrifa *outline* rather than its advance box.

use std::sync::Arc;

use parley::{FontContext, FontFamily, LayoutContext, PositionedLayoutItem, StyleProperty};
use skrifa::{
    instance::{LocationRef, Size},
    metrics::Metrics,
    outline::{DrawSettings, OutlinePen},
    prelude::*,
    raw::TableProvider,
    FontRef, MetadataProvider,
};
use tiny_skia::{FillRule, Paint, PathBuilder, Pixmap, Transform};

/// A vendorable face, by path, per ADR-0007.
struct Face {
    family: &'static str,
    file: &'static str,
}

const FACES: &[Face] = &[
    Face { family: "Noto Sans Thai", file: "../fonts/NotoSansThai-Regular.ttf" },
    Face { family: "Sarabun", file: "../fonts/Sarabun-Regular.ttf" },
];

struct Sample {
    id: &'static str,
    family: &'static str,
    /// The fixture's own authored size (ADR-0007's worked example is 55).
    size: f32,
    text: &'static str,
}

// The identical Thai strings #130 measured, so the only thing that changed
// between that run and this one is the face. Captioning-register Thai with
// above-base tone marks (่ ้ ๊ ๋) and above/below vowel signs (ิ ี ึ ื ุ ู)
// on ordinary base consonants. `\n` placed by hand, per ADR-0008.
const THAI_2LINE: &str =
    "ที่พักผ่อนสุดหรูสำหรับนักท่องเที่ยว\nที่ต้องการความสงบและใกล้ชิดธรรมชาติ";
const THAI_3LINE: &str =
    "รู้สึกดีใจที่ได้กลับมาเที่ยว\nที่ประเทศไทยอีกครั้งหนึ่ง\nหลังจากที่ห่างหายไปนานมาก";
// A worst-case full stack: base + upper vowel + tone mark + lower vowel on
// every cluster (ปู + สระอุ below, สระอิ/อี above, ไม้โท above that).
const THAI_STACK: &str = "ปู่ปี้ปุ๋ยปิ๊งปู๊ปื้น\nปู่ปี้ปุ๋ยปิ๊งปู๊ปื้น";
// Latin control in the *same face*, so the variable is the script, not the
// font. Reproduces ADR-0011's "nominal overstates ink" direction first.
const LATIN: &str = "The quiet cabin by the lake\nis perfect for a long weekend";

const SAMPLES: &[Sample] = &[
    Sample { id: "noto-thai-2line", family: "Noto Sans Thai", size: 55.0, text: THAI_2LINE },
    Sample { id: "noto-thai-3line", family: "Noto Sans Thai", size: 55.0, text: THAI_3LINE },
    Sample { id: "noto-thai-fullstack", family: "Noto Sans Thai", size: 55.0, text: THAI_STACK },
    Sample { id: "noto-latin-control", family: "Noto Sans Thai", size: 55.0, text: LATIN },
    Sample { id: "sarabun-thai-2line", family: "Sarabun", size: 55.0, text: THAI_2LINE },
    Sample { id: "sarabun-thai-fullstack", family: "Sarabun", size: 55.0, text: THAI_STACK },
    Sample { id: "sarabun-latin-control", family: "Sarabun", size: 55.0, text: LATIN },
];

/// ADR-0028 restricts `line_height` to tenths. Sweep them.
const SWEEP_TENTHS: std::ops::RangeInclusive<i32> = 10..=25;

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
    for g in glyphs {
        let Ok(font) = FontRef::from_index(g.font_data.as_slice(), g.font_index) else { continue };
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
    for g in glyphs {
        let Ok(font) = FontRef::from_index(g.font_data.as_slice(), g.font_index) else { continue };
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

/// What the font binary itself says, read straight off the tables. This is
/// candidate 1's entire evidentiary basis.
fn dump_tables(face: &Face, data: &[u8], size: f32) {
    let font = FontRef::new(data).expect("parse font");
    let upem = font.head().map(|h| h.units_per_em()).unwrap_or(0);
    println!("### face {} ({})", face.family, face.file);
    println!("unitsPerEm = {upem}");

    if let Ok(hhea) = font.hhea() {
        let (a, d, g) = (
            hhea.ascender().to_i16(),
            hhea.descender().to_i16(),
            hhea.line_gap().to_i16(),
        );
        println!(
            "hhea:  ascender={a} descender={d} lineGap={g}  -> sum={} em={:.4}  => at size {size}: {:.2}px",
            a - d + g,
            (a - d + g) as f32 / upem as f32,
            (a - d + g) as f32 / upem as f32 * size
        );
    }
    if let Ok(os2) = font.os2() {
        let (ta, td, tg) = (os2.s_typo_ascender(), os2.s_typo_descender(), os2.s_typo_line_gap());
        let (wa, wd) = (os2.us_win_ascent(), os2.us_win_descent());
        println!("OS/2 version = {}", os2.version());
        println!(
            "OS/2 sTypo: ascender={ta} descender={td} lineGap={tg}  -> sum={} em={:.4}  => at size {size}: {:.2}px",
            ta - td + tg,
            (ta - td + tg) as f32 / upem as f32,
            (ta - td + tg) as f32 / upem as f32 * size
        );
        println!(
            "OS/2 usWin: ascent={wa} descent={wd}  -> sum={} em={:.4}  => at size {size}: {:.2}px",
            wa as i32 + wd as i32,
            (wa as i32 + wd as i32) as f32 / upem as f32,
            (wa as i32 + wd as i32) as f32 / upem as f32 * size
        );
        let fs = os2.fs_selection();
        println!(
            "OS/2 fsSelection = {:#010b}   USE_TYPO_METRICS (bit 7) = {}",
            fs.bits(),
            fs.contains(skrifa::raw::tables::os2::SelectionFlags::USE_TYPO_METRICS)
        );
        println!(
            "hhea == sTypo ?  ascender {}  descender {}  lineGap {}",
            ta == hhea_i(&font, 0),
            td == hhea_i(&font, 1),
            tg == hhea_i(&font, 2)
        );
    }
    // What the stack actually resolves. skrifa::metrics::Metrics::new is the
    // single arbitration point: parley 0.11.1 calls exactly this
    // (parley/src/layout/data.rs) and adds no logic of its own.
    let m = Metrics::new(&font, Size::new(size), LocationRef::default());
    println!(
        "skrifa Metrics::new(size={size}) -> ascent={:.2} descent={:.2} leading={:.2}  (ascent-descent+leading = {:.2}px, = line_height {:.4} at size {size})",
        m.ascent,
        m.descent,
        m.leading,
        m.ascent - m.descent + m.leading,
        (m.ascent - m.descent + m.leading) / size
    );
    println!();
}

fn hhea_i(font: &FontRef<'_>, which: u8) -> i16 {
    let Ok(h) = font.hhea() else { return 0 };
    match which {
        0 => h.ascender().to_i16(),
        1 => h.descender().to_i16(),
        _ => h.line_gap().to_i16(),
    }
}

struct LineResult {
    slot_top: f32,
    slot_bottom: f32,
    baseline_y: f32,
    ascent: f32,
    descent: f32,
    ink_top: f32,
    ink_bottom: f32,
    glyphs: Vec<PlacedGlyph>,
}

fn lay_out(
    fcx: &mut FontContext,
    lcx: &mut LayoutContext<[u8; 4]>,
    s: &Sample,
    line_height: f32,
) -> (Vec<LineResult>, f32, f32) {
    let slot_height = s.size * line_height;
    let mut out = Vec::new();
    let mut max_x = 0f32;
    for (i, line_text) in s.text.split('\n').enumerate() {
        let mut builder = lcx.ranged_builder(fcx, line_text, 1.0, false);
        builder.push_default(StyleProperty::FontSize(s.size));
        builder.push_default(StyleProperty::FontFamily(FontFamily::named(s.family)));
        let mut layout = builder.build(line_text);
        layout.break_all_lines(None); // single line, no wrap (ADR-0008)
        let line = layout.lines().next().expect("one line per split");

        let lm = line.metrics();
        let (ascent, descent) = (lm.ascent, lm.descent);
        let slot_top = i as f32 * slot_height;
        let slot_bottom = (i as f32 + 1.0) * slot_height;
        let slot_centre = (slot_top + slot_bottom) / 2.0;
        // ADR-0029: baseline_y = slot_centre + (ascent - descent) / 2
        let baseline_y = slot_centre + (ascent - descent) / 2.0;

        let mut glyphs: Vec<PlacedGlyph> = Vec::new();
        for item in line.items() {
            let PositionedLayoutItem::GlyphRun(glyph_run) = item else { continue };
            let font = glyph_run.run().font().clone();
            let data = Arc::new(font.data.as_ref().to_vec());
            let size_px = glyph_run.run().font_size();
            for g in glyph_run.positioned_glyphs() {
                max_x = max_x.max(g.x);
                glyphs.push(PlacedGlyph {
                    font_data: data.clone(),
                    font_index: font.index,
                    glyph_id: g.id as u16,
                    x: g.x,
                    y: baseline_y,
                    size_px,
                });
            }
        }
        let (ink_top, ink_bottom) = match ink_bbox(&glyphs) {
            Some((_, _, min_y, max_y)) => (min_y, max_y),
            None => (baseline_y, baseline_y),
        };
        out.push(LineResult {
            slot_top, slot_bottom, baseline_y, ascent, descent, ink_top, ink_bottom, glyphs,
        });
    }
    (out, max_x, slot_height)
}

fn worst_seam(lines: &[LineResult]) -> f32 {
    let mut worst = f32::MIN;
    for i in 0..lines.len().saturating_sub(1) {
        worst = worst.max(lines[i].ink_bottom - lines[i + 1].ink_top);
    }
    worst
}

fn main() {
    let png_dir = std::env::args().nth(1);
    if let Some(dir) = &png_dir {
        let _ = std::fs::create_dir_all(dir);
    }

    let mut fcx = FontContext::new();
    let mut lcx: LayoutContext<[u8; 4]> = LayoutContext::new();

    println!("# Part 1 -- what the font binaries say");
    println!();
    println!("Candidate 1 asks whether reading OS/2 sTypo* instead of hhea would");
    println!("change anything. These are the numbers that decide it.");
    println!();
    for face in FACES {
        let data = std::fs::read(face.file)
            .unwrap_or_else(|e| panic!("{}: {e} -- run ./run.sh, which fetches the pinned fonts", face.file));
        dump_tables(face, &data, 55.0);
        // ADR-0007: fonts are files the project declares by path. Register the
        // exact bytes; never consult the system font set.
        fcx.collection.register_fonts(data.into(), None);
    }

    println!("# Part 2 -- real ink vs the declared slot, swept over ADR-0028's tenths");
    println!();
    for s in SAMPLES {
        println!("## {} (family {}, size {})", s.id, s.family, s.size);
        let mut first_clear: Option<f32> = None;
        for t in SWEEP_TENTHS {
            let lh = t as f32 / 10.0;
            let (lines, _, slot_height) = lay_out(&mut fcx, &mut lcx, s, lh);
            let seam = worst_seam(&lines);
            let block_height = (slot_height * lines.len() as f32).ceil();
            let top_escape = -lines[0].ink_top;
            let bottom_escape = lines.last().unwrap().ink_bottom - block_height;
            if seam <= 0.0 && first_clear.is_none() {
                first_clear = Some(lh);
            }
            println!(
                "line_height={lh:.1}  slot={slot_height:.2}  worst ink-to-ink seam={seam:+.2}{}  block top escape={top_escape:+.2}{}  block bottom escape={bottom_escape:+.2}{}",
                if seam > 0.0 { " COLLIDES" } else { "" },
                if top_escape > 0.0 { " ESCAPES" } else { "" },
                if bottom_escape > 0.0 { " ESCAPES" } else { "" },
            );
        }
        match first_clear {
            Some(lh) => println!(
                "-> first tenth with no ink-to-ink collision: line_height={lh:.1}  (1.1 is {} it)",
                if lh > 1.1 { "BELOW" } else { "at or above" }
            ),
            None => println!("-> no tenth in {SWEEP_TENTHS:?} clears the seam"),
        }

        // Detail at the two values actually in use, plus the resolved floor.
        for lh in [1.1f32, 1.2, first_clear.unwrap_or(2.5)] {
            let (lines, max_x, slot_height) = lay_out(&mut fcx, &mut lcx, s, lh);
            println!("   -- detail at line_height={lh:.1} --");
            for (i, l) in lines.iter().enumerate() {
                println!(
                    "   line {i}: slot=[{:.2},{:.2}] baseline_y={:.2} ascent={:.2} descent={:.2}  real ink=[{:.2},{:.2}]",
                    l.slot_top, l.slot_bottom, l.baseline_y, l.ascent, l.descent, l.ink_top, l.ink_bottom
                );
            }
            for i in 0..lines.len().saturating_sub(1) {
                let overlap = lines[i].ink_bottom - lines[i + 1].ink_top;
                println!(
                    "   seam {i}/{}: ink-to-ink overlap={overlap:+.2}{}",
                    i + 1,
                    if overlap > 0.0 { "  <-- COLLIDES" } else { "" }
                );
            }
            if let Some(dir) = &png_dir {
                let all: Vec<PlacedGlyph> = lines
                    .iter()
                    .flat_map(|l| l.glyphs.iter().map(|g| PlacedGlyph {
                        font_data: g.font_data.clone(),
                        font_index: g.font_index,
                        glyph_id: g.glyph_id,
                        x: g.x,
                        y: g.y,
                        size_px: g.size_px,
                    }))
                    .collect();
                let block_height = (slot_height * lines.len() as f32).ceil();
                let slot_ys: Vec<f32> =
                    (0..=lines.len()).map(|i| i as f32 * slot_height).collect();
                let path = format!("{dir}/{}-lh{}.png", s.id, (lh * 10.0).round() as i32);
                render_debug_png(
                    &all,
                    (max_x + 80.0).ceil() as u32,
                    block_height.ceil() as u32 + 20,
                    &slot_ys,
                    &path,
                );
            }
        }
        println!();
    }
}
