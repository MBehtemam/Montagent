//! What a line's glyphs **actually cover**, as opposed to what its slot reserves.
//!
//! ADR-0007 makes a line's height *"the largest `size` among the runs on that line ×
//! `line_height`"* — a function of two numbers the document declares and never of the
//! font's ink. That is a deliberate decision and [ADR-0087](../../../docs/adr/0087-thai-line-height-collision-is-a-font-selection-problem.md)
//! does not reopen it. What that ADR adds is the **measurement** the decision left
//! unavailable: for a script whose marks stack — Thai's base + upper vowel + tone mark +
//! lower vowel — a `line_height` tuned on Latin can put one line's ink through the next
//! line's, with every field in the document individually valid and nothing in the tool
//! able to say so.
//!
//! So this module answers one question and no other: **where is the ink**. It reaches no
//! conclusion about whether the answer is acceptable — that is `validate`'s alone
//! (ADR-0006), and the structural form of the rule here is that nothing in this file ever
//! sees a `line_height`, a declared `height`, or a threshold.
//!
//! # The vertical axis only
//!
//! ADR-0011 names a per-line **ink box** beside the advance width, and
//! `crate::verbs::measure`'s own module doc records why it was not built: an ink *box* is
//! an absolute rect, so it needs the block's horizontal placement — `x`, `origin`'s
//! horizontal component, and how `align`'s `start`/`end` resolve against a line's base
//! direction under bidi — and no ADR settles the last of those.
//!
//! **None of that blocks the vertical half**, which is why it is here and the horizontal
//! half still is not. A line's ink top and bottom need the baseline and the glyph
//! outlines, both of which [`crate::engine`] already has, and the block is already placed
//! vertically by `origin` over its own height. The horizontal question is untouched and
//! stays where it was.
//!
//! # Bounds, not outlines
//!
//! The extent comes from `skrifa`'s [`GlyphMetrics::bounds`], which for a `glyf` face is
//! the bounding box the font itself stores and for CFF/CFF2 and any variable face is
//! derived from the outline. Both are the glyph's real contour bounds, so the number is
//! the ink's and not the advance box's — which is the entire point, since an advance box
//! is exactly the thing that does not know about a stacked tone mark.
//!
//! [`crate::place`] decodes the same glyphs' full outlines to draw them. It is not shared
//! with this, and the reason is that the two want different things from one font: drawing
//! needs every contour at every glyph, and measuring needs four numbers per glyph that a
//! `glyf` face hands over without decoding a contour at all. Named here rather than left
//! to be discovered as duplication.

use parley::{Layout, PositionedLayoutItem};
use serde::Serialize;
use skrifa::instance::{LocationRef, Size};
use skrifa::{FontRef, GlyphId, MetadataProvider};

/// One line's inked extent, in pixels relative to that line's own baseline, **y-down**.
///
/// `top` is normally negative (ink above the baseline) and `bottom` positive. Relative
/// rather than absolute because a line is shaped before the block knows where it sits:
/// [`crate::engine`]'s pass one produces this, and pass two adds the baseline.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct LineInk {
    pub(crate) top: f64,
    pub(crate) bottom: f64,
}

/// The gap between two adjacent lines' **ink**, where both lines have some.
///
/// Positive `overlap` means the lines collide: the line above has ink below where the line
/// below's ink starts. Negative is the ordinary case and states the clearance.
///
/// Stated as an overlap rather than as a gap because the sign that matters is the one that
/// is wrong, and a reader scanning a column of numbers for a defect should be looking for
/// the positive one. It is the same sign convention the measurements in
/// `docs/research/thai-vertical-metrics.md` use, so the two can be compared without
/// negating one of them in your head.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct InkSeam {
    /// The index of the upper line of the pair.
    pub above: usize,
    /// The index of the lower line — never `above + 1` when an uninked line sits between
    /// them, because a line with no ink cannot collide with anything.
    pub below: usize,
    /// `above.ink_bottom − below.ink_top`. Positive is a collision.
    pub overlap: f64,
}

/// One line's ink, or `None` where the line draws nothing.
///
/// An empty line, a line of spaces, and a line whose every glyph is a mark the face has no
/// contour for all return `None` rather than a zero-height extent at the baseline. The
/// distinction is load-bearing: a zero-height extent *is* ink as far as a seam comparison
/// is concerned, and a blank line would then appear to collide with its neighbour.
pub(crate) fn line_ink(layout: &Layout<u32>) -> Option<LineInk> {
    let mut top = f64::INFINITY;
    let mut bottom = f64::NEG_INFINITY;

    for placed in layout.lines() {
        let baseline = f64::from(placed.metrics().baseline);
        for item in placed.items() {
            let PositionedLayoutItem::GlyphRun(run) = item else {
                continue;
            };
            let font = run.run().font().clone();
            let size = run.run().font_size();
            // One `FontRef` and one `GlyphMetrics` per glyph run rather than per glyph:
            // the run is the unit over which the face and the size are constant, so
            // building them inside the glyph loop would rebuild identical objects for
            // every character of a word.
            let Ok(face) = FontRef::from_index(font.data.as_ref(), font.index) else {
                continue;
            };
            let metrics = face.glyph_metrics(Size::new(size), LocationRef::default());

            for glyph in run.positioned_glyphs() {
                let Some(bounds) = metrics.bounds(GlyphId::new(glyph.id)) else {
                    continue;
                };
                // A glyph the face draws nothing for — a space, most of them — reports an
                // empty box. It contributes no ink, and taking it would pin the extent to
                // the baseline.
                if bounds.y_min == bounds.y_max && bounds.x_min == bounds.x_max {
                    continue;
                }
                // The glyph origin's own offset from this line's baseline, y-down. parley
                // places glyphs against the layout's baseline; the block's baseline is
                // ADR-0029's and is added by the caller, so what is derived here is the
                // part that is the line's alone.
                let dy = f64::from(glyph.y) - baseline;
                // A font's outlines are y-up and every coordinate in this project is
                // y-down, so the glyph's `y_max` is its *top*. The same flip
                // [`crate::place::Pen`] applies to a contour, applied to a box.
                top = top.min(dy - f64::from(bounds.y_max));
                bottom = bottom.max(dy - f64::from(bounds.y_min));
            }
        }
    }

    (top.is_finite() && bottom.is_finite()).then_some(LineInk { top, bottom })
}

/// Every adjacent inked pair's seam, in line order.
///
/// Pairs of *inked* lines, so a blank line between two inked ones makes the pair that
/// spans it: the blank reserves a slot and draws nothing, and the question a seam answers
/// is whether two things that are drawn touch.
pub(crate) fn seams(ink: &[(usize, f64, f64)]) -> Vec<InkSeam> {
    ink.windows(2)
        .map(|pair| {
            let (above, _, above_bottom) = pair[0];
            let (below, below_top, _) = pair[1];
            InkSeam {
                above,
                below,
                overlap: above_bottom - below_top,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_seam_is_positive_exactly_when_the_ink_overlaps() {
        // Line 0's ink ends at 61.63, line 1's begins at 52.05: they overlap by 9.58 —
        // the figure `docs/research/thai-vertical-metrics.md` §5.1 measured at
        // `line_height` 1.1, restated here as the arithmetic alone so the sign convention
        // is pinned without a font.
        let seams = seams(&[(0, -8.45, 61.63), (1, 52.05, 108.10)]);
        assert_eq!(seams.len(), 1);
        assert_eq!(seams[0].above, 0);
        assert_eq!(seams[0].below, 1);
        assert!((seams[0].overlap - 9.58).abs() < 0.005, "{seams:?}");
    }

    #[test]
    fn clearance_reads_as_a_negative_overlap() {
        let seams = seams(&[(0, 0.0, 40.0), (1, 60.0, 100.0)]);
        assert_eq!(seams[0].overlap, -20.0);
    }

    #[test]
    fn a_blank_line_between_two_inked_ones_makes_one_pair_that_spans_it() {
        // Indices 0 and 2: the uninked line 1 is not in the input at all, because a line
        // that draws nothing cannot collide with anything.
        let seams = seams(&[(0, 0.0, 40.0), (2, 120.0, 160.0)]);
        assert_eq!(seams.len(), 1);
        assert_eq!((seams[0].above, seams[0].below), (0, 2));
    }

    #[test]
    fn one_inked_line_has_no_seam() {
        assert!(seams(&[(0, 0.0, 40.0)]).is_empty());
        assert!(seams(&[]).is_empty());
    }
}
