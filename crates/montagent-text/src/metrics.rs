//! One face's own metrics, as the integers the file stores — for arithmetic that has to be
//! done *before* anything is shaped.
//!
//! #421 exists for `frame`'s contact sheet, whose sizing decides a label's type size and
//! whether a tile's id fits at ADR-0098's 8 px floor **across the whole sheet** before a
//! single label is placed. That is a width question asked many times over, in exact
//! arithmetic, and a layout per question would be the wrong tool: what it needs is the
//! face's em, its vertical metrics and each character's advance, which is what is here.
//!
//! **Unshaped, and that is a property of the face it is asked of.** [`FaceMetrics::advance`]
//! sums `hmtx` advances through `cmap`; it applies no kerning, ligature or contextual
//! alternate. For a face with none of those — which Montagent's chrome face is chosen to
//! be, and is tested to be against the shaper — the sum *is* the shaped width. For any
//! other face it is an approximation, and the caller who asks it of one owns that.
//!
//! Read, never judged, on [`names`](crate::names)'s rule: which face is the chrome face and
//! what a label may spend are the core's.

use skrifa::instance::{LocationRef, Size};
use skrifa::metrics::GlyphMetrics;
use skrifa::{FontRef, GlyphId, MetadataProvider};

/// One face's em, vertical metrics and advances, in the file's own font units.
pub struct FaceMetrics<'a> {
    font: FontRef<'a>,
    glyphs: GlyphMetrics<'a>,
    units_per_em: u16,
    ascender: i32,
    descender: i32,
    line_gap: i32,
}

impl<'a> FaceMetrics<'a> {
    /// The face at `index` of `bytes` (face 0 when omitted, as ADR-0007 defaults it).
    ///
    /// The error is one sentence about the bytes, like [`crate::Charmap::of`]'s.
    pub fn read(bytes: &'a [u8], index: Option<u32>) -> Result<FaceMetrics<'a>, String> {
        let index = index.unwrap_or(0);
        let font = FontRef::from_index(bytes, index)
            .map_err(|e| format!("face {index} could not be read ({e})"))?;
        // Unscaled, at the default location: the numbers the file stores, which are
        // integers. `skrifa` hands them over as `f32`, which holds every `i16` exactly, so
        // the conversion below is a change of type and never a rounding.
        let metrics = font.metrics(Size::unscaled(), LocationRef::default());
        let glyphs = font.glyph_metrics(Size::unscaled(), LocationRef::default());
        Ok(FaceMetrics {
            units_per_em: metrics.units_per_em,
            ascender: metrics.ascent as i32,
            descender: metrics.descent as i32,
            line_gap: metrics.leading as i32,
            glyphs,
            font,
        })
    }

    /// Font units per em — the divisor that turns every other number here into a
    /// proportion of the type size.
    pub fn units_per_em(&self) -> u16 {
        self.units_per_em
    }

    /// The ascender, y-up: positive, above the baseline.
    pub fn ascender(&self) -> i32 {
        self.ascender
    }

    /// The descender, y-up: negative, below the baseline.
    pub fn descender(&self) -> i32 {
        self.descender
    }

    /// The line gap the face asks for between one line's descender and the next's ascender.
    pub fn line_gap(&self) -> i32 {
        self.line_gap
    }

    /// The unshaped advance of `text`: each character's `hmtx` advance, summed.
    ///
    /// A character the face does not map is drawn as `.notdef`, so it advances by glyph 0's
    /// width rather than by nothing — a width that counted a replacement glyph as empty
    /// would under-fit exactly the label that most needs the room.
    pub fn advance(&self, text: &str) -> u32 {
        let charmap = self.font.charmap();
        text.chars()
            .map(|c| {
                let glyph = charmap.map(c).unwrap_or(GlyphId::NOTDEF);
                // A glyph with no `hmtx` entry takes the last one's, which `skrifa` applies.
                // `None` is a glyph id past the font's count — a malformed `cmap` — and what
                // is drawn for that is `.notdef`, so it advances as `.notdef` does, never by
                // nothing.
                self.glyphs
                    .advance_width(glyph)
                    .or_else(|| self.glyphs.advance_width(GlyphId::NOTDEF))
                    .unwrap_or(0.0) as u32
            })
            .sum()
    }
}
