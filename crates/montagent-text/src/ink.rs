//! What a line's glyphs **actually cover**, as opposed to what its slot reserves.
//!
//! ADR-0007 makes a line's slot *"the largest `size` among the runs on that line ×
//! `line_height`"* — a function of two numbers the document declares and never of the
//! font's ink. That is a deliberate decision and
//! [ADR-0087](../../../docs/adr/0087-thai-line-height-collision-is-a-font-selection-problem.md)
//! does not reopen it. What that ADR adds is the **measurement** the decision left
//! unavailable: for a script whose marks stack — Thai's base + upper vowel + tone mark +
//! lower vowel — a `line_height` tuned on Latin can put one line's ink through the next
//! line's, with every field in the document individually valid.
//!
//! This module answers where the ink is and whether two lines' ink meets. It reaches no
//! conclusion about whether the answer is acceptable — that is `validate`'s alone
//! (ADR-0006), and the structural form of the rule is that nothing here ever sees a
//! `line_height`, a declared `height`, or a threshold.
//!
//! # A seam is two-dimensional, and the first version of this was not
//!
//! **ADR-0087 first shipped a whole-line seam and it was wrong.** It compared the lowest
//! ink anywhere on one line against the highest ink anywhere on the next, so a descender at
//! one end of a line "collided" with a tone mark at the other end, which never touch. Its
//! second court measured the cost: on #130's own Thai prose at the format's **default**
//! `line_height` of 1.2 it claimed +4.07 px (Noto Sans Thai) and +18.26 px (Sarabun) of
//! overlap on renders that share **zero** pixels, and it fired 1–4 tenths above the real
//! floor on every string tested. That is exactly the failure ADR-0011 names — *"a
//! text-overflow check run on nominal metrics is a false-positive generator"* — one level
//! down.
//!
//! Worse, it was nearly **document-blind**: a whole-line maximum is reached by *any* cluster
//! with a deep below-mark and *any* cluster with a tall above-mark, so it returned the same
//! number for ordinary prose as for a string made entirely of worst-case stacks. A number
//! that cannot tell two documents apart is not a fact about the file — it is the face's own
//! floor recomputed at runtime, which is the thing ADR-0087 rejected as candidate 2.
//!
//! So a seam is compared **per glyph, where two glyphs actually share horizontal space**.
//!
//! # Which horizontal facts this needs, and which it still does not
//!
//! It needs [`crate::Spec::align`], because two lines' glyphs have to be in one horizontal
//! frame before their ink can be compared, and `align` is what puts them there. The offset
//! is [`crate::place::offset`]'s — the same function the renderer places glyphs with, not a
//! second copy.
//!
//! It still needs **nothing about where the block sits**: the element's `x`, its `origin`
//! and its declared `width` are all absent here and all cancel, because a seam is a
//! *difference* between two lines of one block and moving the block moves both equally.
//! ADR-0087's separability claim survives in that narrowed form and no other.
//!
//! # Bounds, not pixels
//!
//! The extent comes from `skrifa`'s [`GlyphMetrics::bounds`] — for a `glyf` face the box the
//! font stores, for CFF/CFF2 and variable faces one derived from the outline. Both are the
//! glyph's real contour bounds.
//!
//! **Rasterised coverage was considered and rejected**, and the stopping point is principled
//! rather than a cost compromise: counting pixels imports an anti-aliasing coverage
//! threshold, a resolution and a rasterizer dependency into `validate`, and that threshold
//! is an external constant of exactly the kind ADR-0061 fences — the same objection that
//! sank a script-aware `line_height` floor. Contour bounds are the tightest instrument that
//! is still a strict upper bound on painted overlap and needs no such choice. Measured, they
//! agree with rasterised coverage on the clearing tenth in every case tested; the whole-line
//! box did not.
//!
//! [`crate::place`] decodes the same glyphs' full outlines to draw them. Not shared with
//! this, because the two want different things from one font: drawing needs every contour,
//! and measuring needs four numbers per glyph that a `glyf` face hands over without decoding
//! one.

use parley::{Layout, PositionedLayoutItem};
use serde::Serialize;
use skrifa::instance::{LocationRef, Size};
use skrifa::{FontRef, GlyphId, MetadataProvider};

/// One glyph's inked box, in pixels relative to its line's origin and baseline, **y-down**.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct GlyphInk {
    pub(crate) x0: f64,
    pub(crate) x1: f64,
    pub(crate) top: f64,
    pub(crate) bottom: f64,
}

/// One line's ink: its overall vertical extent, and every glyph that made it.
///
/// The scalar `top`/`bottom` are what `measure` reports per line and are genuinely
/// vertical-only — they need no `align`, and the second court confirmed it. The glyph list
/// is what a *seam* needs, because a seam is two-dimensional.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LineInk {
    pub(crate) top: f64,
    pub(crate) bottom: f64,
    pub(crate) glyphs: Vec<GlyphInk>,
}

/// The gap between two adjacent lines' **ink**, where both lines have some.
///
/// Positive `overlap` means the lines collide: some glyph of the upper line has ink below
/// where a glyph of the lower line — one that shares horizontal space with it — begins.
/// Negative is the ordinary case and states the clearance at the tightest such pair.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct InkSeam {
    /// The index of the upper line of the pair.
    pub above: usize,
    /// The index of the lower line — never `above + 1` when an uninked line sits between
    /// them, because a line with no ink cannot collide with anything.
    pub below: usize,
    /// The tightest vertical relationship between any pair of glyphs that **share
    /// horizontal space**. Positive is a collision.
    ///
    /// `null` where no glyph of the upper line shares any horizontal span with a glyph of
    /// the lower one — two short lines at opposite ends of a wide centred block, say. They
    /// cannot meet at any `line_height`, and there is no clearance to state: a large
    /// negative number would read as a measurement of something that was never measured.
    pub overlap: Option<f64>,
}

/// One line's ink, or `None` where the line draws nothing.
///
/// An empty line, a line of spaces, and a line whose every glyph is one the face has no
/// contour for all return `None` rather than a zero-height extent at the baseline. The
/// distinction is load-bearing: a zero-height extent *is* ink as far as a seam comparison is
/// concerned, and a blank line would then appear to collide with its neighbour.
pub(crate) fn line_ink(layout: &Layout<u32>) -> Option<LineInk> {
    let mut glyphs: Vec<GlyphInk> = Vec::new();

    for placed in layout.lines() {
        let baseline = f64::from(placed.metrics().baseline);
        for item in placed.items() {
            let PositionedLayoutItem::GlyphRun(run) = item else {
                continue;
            };
            let font = run.run().font().clone();
            let size = run.run().font_size();
            // One `FontRef` and one `GlyphMetrics` per glyph run rather than per glyph: the
            // run is the unit over which the face and the size are constant.
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
                // ADR-0029's and is added by the caller.
                let dy = f64::from(glyph.y) - baseline;
                let x = f64::from(glyph.x);
                // A font's outlines are y-up and every coordinate in this project is y-down,
                // so the glyph's `y_max` is its *top* — the same flip
                // [`crate::place::Pen`] applies to a contour, applied to a box.
                glyphs.push(GlyphInk {
                    x0: x + f64::from(bounds.x_min),
                    x1: x + f64::from(bounds.x_max),
                    top: dy - f64::from(bounds.y_max),
                    bottom: dy - f64::from(bounds.y_min),
                });
            }
        }
    }

    if glyphs.is_empty() {
        return None;
    }
    let top = glyphs.iter().map(|g| g.top).fold(f64::INFINITY, f64::min);
    let bottom = glyphs
        .iter()
        .map(|g| g.bottom)
        .fold(f64::NEG_INFINITY, f64::max);
    Some(LineInk {
        top,
        bottom,
        glyphs,
    })
}

/// One line's glyphs, in the **block's** frame: shifted by the line's alignment offset and
/// its baseline, and grown by the stroke.
///
/// **The stroke is in the seam, and is in no other ink number.** ADR-0014 puts a text
/// stroke *outside* the glyph contour, so two outlined lines have `2 × stroke_width` less
/// clearance between them than their contours suggest. Dilating each box by the line's own
/// `stroke_width` is what keeps this instrument a strict upper bound on painted overlap for
/// stroked text as well as plain — without it, "over-reports but never misses" would be true
/// only of text nobody outlined.
pub(crate) fn placed(ink: &LineInk, dx: f64, baseline: f64, stroke_width: i64) -> Vec<GlyphInk> {
    let grow = stroke_width as f64;
    ink.glyphs
        .iter()
        .map(|g| GlyphInk {
            x0: dx + g.x0 - grow,
            x1: dx + g.x1 + grow,
            top: baseline + g.top - grow,
            bottom: baseline + g.bottom + grow,
        })
        .collect()
}

/// The seam between two lines' placed glyphs: the tightest vertical relationship over every
/// pair that shares horizontal space.
///
/// Pairwise rather than bucketed into columns. The two give identical numbers — the second
/// court checked — and a pair loop needs no bucket width, which would be one more arbitrary
/// constant in a module that has just finished arguing against arbitrary constants. Lines
/// are tens of glyphs, so the quadratic is not worth avoiding with a parameter.
pub(crate) fn seam_between(above: &[GlyphInk], below: &[GlyphInk]) -> Option<f64> {
    let mut worst: Option<f64> = None;
    for a in above {
        for b in below {
            // Touching at a single x is not sharing space: the boxes are half-open, and two
            // glyphs whose boxes abut exactly have no column in common.
            if a.x0 < b.x1 && b.x0 < a.x1 {
                let overlap = a.bottom - b.top;
                worst = Some(match worst {
                    Some(w) => w.max(overlap),
                    None => overlap,
                });
            }
        }
    }
    worst
}

#[cfg(test)]
mod tests {
    use super::*;

    fn g(x0: f64, x1: f64, top: f64, bottom: f64) -> GlyphInk {
        GlyphInk {
            x0,
            x1,
            top,
            bottom,
        }
    }

    #[test]
    fn a_seam_is_positive_exactly_when_ink_that_shares_x_overlaps() {
        // Directly stacked: the lower glyph starts above where the upper one ends.
        assert_eq!(
            seam_between(&[g(0.0, 10.0, 0.0, 60.0)], &[g(0.0, 10.0, 50.0, 100.0)]),
            Some(10.0)
        );
    }

    #[test]
    fn clearance_reads_as_a_negative_overlap() {
        assert_eq!(
            seam_between(&[g(0.0, 10.0, 0.0, 40.0)], &[g(0.0, 10.0, 60.0, 100.0)]),
            Some(-20.0)
        );
    }

    #[test]
    fn glyphs_that_share_no_horizontal_space_have_no_seam_at_all() {
        // The defect that made the first version of this a false-positive generator: a
        // descender at one end of a line and a tone mark at the other overlap vertically and
        // never touch. `None` rather than a negative clearance — nothing was measured.
        assert_eq!(
            seam_between(&[g(0.0, 10.0, 0.0, 60.0)], &[g(200.0, 210.0, 50.0, 100.0)]),
            None
        );
    }

    #[test]
    fn boxes_that_abut_exactly_do_not_share_space() {
        assert_eq!(
            seam_between(&[g(0.0, 10.0, 0.0, 60.0)], &[g(10.0, 20.0, 50.0, 100.0)]),
            None
        );
    }

    #[test]
    fn the_worst_sharing_pair_is_the_one_reported() {
        // Three glyphs below; only the two that share x count, and the tighter of those wins.
        let above = [g(0.0, 100.0, 0.0, 60.0)];
        let below = [
            g(0.0, 10.0, 58.0, 100.0),   // +2
            g(20.0, 30.0, 50.0, 100.0),  // +10, the worst
            g(500.0, 510.0, 0.0, 100.0), // shares no x, ignored however bad
        ];
        assert_eq!(seam_between(&above, &below), Some(10.0));
    }

    #[test]
    fn the_stroke_is_dilated_into_the_boxes() {
        // ADR-0014: the stroke falls outside the contour, so an outlined line has less
        // clearance than its contours claim. Two lines clearing by 20 px of contour collide
        // once each is stroked at 12.
        let ink = LineInk {
            top: 0.0,
            bottom: 40.0,
            glyphs: vec![g(0.0, 10.0, 0.0, 40.0)],
        };
        let bare_above = placed(&ink, 0.0, 0.0, 0);
        let bare_below = placed(&ink, 0.0, 60.0, 0);
        assert_eq!(seam_between(&bare_above, &bare_below), Some(-20.0));

        let stroked_above = placed(&ink, 0.0, 0.0, 12);
        let stroked_below = placed(&ink, 0.0, 60.0, 12);
        assert_eq!(seam_between(&stroked_above, &stroked_below), Some(4.0));
    }

    #[test]
    fn the_alignment_offset_moves_a_line_into_the_shared_frame() {
        // Two short lines pushed to opposite ends cannot meet, however tight the leading —
        // the fact a whole-line seam could not represent.
        let ink = LineInk {
            top: 0.0,
            bottom: 40.0,
            glyphs: vec![g(0.0, 30.0, 0.0, 40.0)],
        };
        let left = placed(&ink, 0.0, 0.0, 0);
        let right = placed(&ink, 400.0, 10.0, 0);
        assert_eq!(seam_between(&left, &right), None);
        // Aligned the same way, they do meet.
        let under = placed(&ink, 0.0, 10.0, 0);
        assert_eq!(seam_between(&left, &under), Some(30.0));
    }
}
