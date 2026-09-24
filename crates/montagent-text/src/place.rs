//! Where every glyph goes — the same partition and the same baselines `measure` reports,
//! turned into outlines a rasterizer can fill (ADR-0010: *"parley shapes and breaks,
//! skrifa scales the outlines, the rasterizer fills the paths"*).
//!
//! # One shaping pass, and why that is the whole point
//!
//! [`place`] does not lay the text out again. It calls the same
//! [`crate::engine::measured`] `measure` calls and reads the glyphs off the layouts that
//! pass already built, so the line partition, the slot heights and every `baseline_y` are
//! **the same numbers**, not numbers derived the same way. #213 states the requirement in
//! as many words — *"no second line-breaking implementation exists"* — and a second
//! implementation is what a separate layout here would be, however faithfully it copied
//! [`crate::engine`]'s arithmetic. The failure it prevents is specific: `measure` tells an
//! author their two-line block is 121 px tall, the renderer draws three lines, and both
//! answers are internally consistent.
//!
//! # Block-local coordinates, because placement is the canvas's
//!
//! Every coordinate here is relative to the **block's own top-left**, in unscaled element
//! space. Nothing in this module knows the element's `x`, its `origin`, its `scale` or its
//! `rotation`: those are one transform, ADR-0012's, and the rasterizer already applies it
//! to every other element type. Handing back absolute frame coordinates would mean text
//! was placed by different code from everything else, and the two would drift the first
//! time a pivot rule changed.
//!
//! The block's box is the **typographic** one — the widest line's advance by the sum of
//! the slots — and not the element's declared `width`/`height`. ADR-0014 is explicit that
//! a text element's box is *"a container claim, not painted geometry"*, and ADR-0007 makes
//! `align` *"how lines align to each other"* rather than how they sit in a container. So
//! the box the lines align inside is the block they themselves make. This also keeps the
//! vertical and horizontal readings the same shape: `measure` already places the block by
//! `origin` over the block's own height, with the declared `height` never consulted.
//!
//! # The stroke is not in the box
//!
//! ADR-0014: on text the stroke falls **outside** the glyph contour and *"does not enlarge
//! the element"*. So the extent below is stroke-naive on purpose — it is the box `origin`
//! pivots about, and growing it by the stroke would move every glyph by `stroke_width`
//! the moment an author outlined one word. The stroked extent is
//! [`crate::Measurement::extent`]'s, and it answers a different question: what an author
//! compares a declared box against.

use std::collections::HashMap;

use skrifa::instance::{LocationRef, Size};
use skrifa::outline::{DrawSettings, OutlinePen};
use skrifa::{FontRef, GlyphId, MetadataProvider};

use parley::PositionedLayoutItem;

use crate::engine::measured;
use crate::fonts::{FontError, Fonts};
use crate::{Measurement, Spec};

/// How lines align to each other (ADR-0007) — never how a box is placed, which is
/// `origin`.
///
/// `start`/`end` rather than `left`/`right` because RTL is in v1, and they are resolved
/// here against the line's own base direction rather than assumed to mean left and right.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Align {
    #[default]
    Start,
    Center,
    End,
}

/// One segment of a glyph's outline, at the glyph's own origin, **y already flipped** so
/// that down is positive — the direction every coordinate in this project runs.
///
/// A font's outlines are y-up; a frame's pixels are y-down. The flip happens once, in
/// [`Pen`], rather than in whichever rasterizer consumes this.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PathEl {
    Move(f32, f32),
    Line(f32, f32),
    Quad(f32, f32, f32, f32),
    Cubic(f32, f32, f32, f32, f32, f32),
    Close,
}

/// One glyph, placed in the block's own coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Glyph {
    /// The glyph origin, relative to the block's top-left.
    pub x: f64,
    pub y: f64,
    /// Which of the element's runs this glyph's characters came from — the index into
    /// [`Spec::runs`] — so the caller can resolve ADR-0014's run-addressable `color` and
    /// `stroke`.
    pub run: usize,
    /// Which of [`Placement::outlines`] to draw.
    ///
    /// An index rather than the path itself: one glyph's outline is scaled once and drawn
    /// wherever it repeats, and `cobweb  -  cobweb` is nine distinct outlines over
    /// seventeen glyphs. The rasterizer can key its own path cache on the same integer.
    pub outline: usize,
}

/// Every glyph of one text element, and the block they make.
#[derive(Debug, Clone, PartialEq)]
pub struct Placement {
    /// What `measure` would answer for this element — the same numbers, from the same
    /// pass.
    pub measurement: Measurement,
    /// The typographic block, before the stroke: the widest line's advance by the sum of
    /// the slots. The box `origin` pivots about.
    pub width: f64,
    pub height: f64,
    pub glyphs: Vec<Glyph>,
    /// Each distinct outline, already scaled to its glyph's `size` in element space.
    pub outlines: Vec<Vec<PathEl>>,
}

/// Place one text element's glyphs, aligned as [`Spec::align`] says.
///
/// The failure is the same one [`crate::measure`] has and the only one there is: a font
/// the project does not declare, or one that cannot be opened.
pub fn place(fonts: &mut Fonts, spec: &Spec<'_>) -> Result<Placement, FontError> {
    let align = spec.align;
    let (measurement, layouts) = measured(fonts, spec)?;

    // The block the lines align inside is the block the lines make (ADR-0007), so its
    // width is the widest line's advance — which `measure` has already computed, and is
    // read from there rather than recomputed.
    let block_width = measurement.advance_width;
    let block_top = measurement.block_top;
    let height = measurement.block_bottom - block_top;

    let mut outlines: Vec<Vec<PathEl>> = Vec::new();
    let mut seen: HashMap<(usize, u32, u32, u32), usize> = HashMap::new();
    let mut glyphs = Vec::new();

    for (line, layout) in measurement.lines.iter().zip(&layouts) {
        // ADR-0029's baseline, taken from `measure`'s answer and shifted into the block's
        // own frame. Not recomputed: the half-leading rule is the engine's, and a second
        // copy of `slot_centre + (ascent − descent) / 2` here is a second rule to keep in
        // step.
        let baseline = line.baseline_y - block_top;
        let dx = offset(align, layout.is_rtl(), block_width, line.advance_width);

        for placed in layout.lines() {
            for item in placed.items() {
                let PositionedLayoutItem::GlyphRun(run) = item else {
                    continue;
                };
                let which = run.style().brush as usize;
                let font = run.run().font().clone();
                let size = run.run().font_size();
                let data = font.data.as_ref();
                let key_font = data.as_ptr() as usize;
                for glyph in run.positioned_glyphs() {
                    let key = (key_font, font.index, glyph.id, size.to_bits());
                    let outline = *seen.entry(key).or_insert_with(|| {
                        outlines.push(outline_of(data, font.index, glyph.id, size));
                        outlines.len() - 1
                    });
                    glyphs.push(Glyph {
                        x: dx + f64::from(glyph.x),
                        // parley's `y` is the baseline of its own one-line layout; this
                        // block's baseline is ADR-0029's, so parley's is subtracted back
                        // out and ours put in its place.
                        y: baseline + f64::from(glyph.y - placed.metrics().baseline),
                        run: which,
                        outline,
                    });
                }
            }
        }
    }

    Ok(Placement {
        measurement,
        width: block_width,
        height,
        glyphs,
        outlines,
    })
}

/// How far into the block one line starts.
///
/// **`start` and `end` are resolved against the line's own base direction**, which is what
/// ADR-0007 bought them for — *"uses start/end rather than left/right because RTL is in
/// v1"*. The direction is parley's, read off the layout rather than guessed from the
/// characters, so a `dir` override on a run reaches this the same way it reaches shaping.
///
/// **Recorded residual: trailing whitespace is not hung.** A line's advance here is the
/// one `measure` reports, which includes whatever whitespace the author wrote at the end
/// of it — and ADR-0007 is explicit that *"whitespace inside a run is content"*. A
/// typesetter would normally let a trailing space hang outside the alignment box, which
/// would centre `"a "` the way it centres `"a"`. Montagent does not, because the two
/// readings disagree by half a space and only one of them is the number `measure` already
/// published. No fixture text ends in a space; named so the next reader does not have to
/// discover it from a half-space offset.
pub(crate) fn offset(align: Align, rtl: bool, block_width: f64, advance: f64) -> f64 {
    let free = (block_width - advance).max(0.0);
    match (align, rtl) {
        (Align::Start, false) | (Align::End, true) => 0.0,
        (Align::Start, true) | (Align::End, false) => free,
        (Align::Center, _) => free / 2.0,
    }
}

/// One glyph's outline, scaled to `size` in element space.
///
/// An unreadable font or a glyph with no outline gives an empty path rather than an error:
/// the font has already been opened and registered by this point, and a glyph the face
/// carries no contour for — a space, most of them — is not a failure to report. What it
/// must never be is a *substituted* glyph, which is why nothing here falls back.
fn outline_of(data: &[u8], index: u32, glyph: u32, size: f32) -> Vec<PathEl> {
    let mut pen = Pen::default();
    if let Ok(font) = FontRef::from_index(data, index)
        && let Some(outline) = font.outline_glyphs().get(GlyphId::new(glyph))
    {
        let _ = outline.draw(
            DrawSettings::unhinted(Size::new(size), LocationRef::default()),
            &mut pen,
        );
    }
    pen.path
}

/// The pen skrifa draws into, flipping y on the way past.
#[derive(Default)]
struct Pen {
    path: Vec<PathEl>,
}

impl OutlinePen for Pen {
    fn move_to(&mut self, x: f32, y: f32) {
        self.path.push(PathEl::Move(x, -y));
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.path.push(PathEl::Line(x, -y));
    }
    fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        self.path.push(PathEl::Quad(cx, -cy, x, -y));
    }
    fn curve_to(&mut self, c1x: f32, c1y: f32, c2x: f32, c2y: f32, x: f32, y: f32) {
        self.path.push(PathEl::Cubic(c1x, -c1y, c2x, -c2y, x, -y));
    }
    fn close(&mut self) {
        self.path.push(PathEl::Close);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_and_end_swap_under_a_right_to_left_base_direction() {
        // The whole reason ADR-0007 spells the values `start`/`end`: one document renders
        // correctly in both directions, and neither value means "left".
        assert_eq!(offset(Align::Start, false, 100.0, 60.0), 0.0);
        assert_eq!(offset(Align::End, false, 100.0, 60.0), 40.0);
        assert_eq!(offset(Align::Start, true, 100.0, 60.0), 40.0);
        assert_eq!(offset(Align::End, true, 100.0, 60.0), 0.0);
        assert_eq!(offset(Align::Center, false, 100.0, 60.0), 20.0);
        assert_eq!(offset(Align::Center, true, 100.0, 60.0), 20.0);
    }

    #[test]
    fn the_widest_line_starts_at_the_block_edge_whatever_the_alignment() {
        for align in [Align::Start, Align::Center, Align::End] {
            assert_eq!(offset(align, false, 100.0, 100.0), 0.0);
        }
    }
}
