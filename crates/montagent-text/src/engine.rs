//! What a piece of text actually occupies, in the fonts the project declares.
//!
//! One function, [`measure`], and everything `measure` the verb answers with comes out of
//! it. The arithmetic it is responsible for, and the ADR each piece answers to:
//!
//! - **The line partition** is [`crate::lines`]' — UAX #14's mandatory breaks, never
//!   `split('\n')` (ADR-0008) — and each line is laid out as its own paragraph.
//! - **A line's slot** is *the largest `size` among the runs on that line* × `line_height`
//!   (ADR-0007), and the block is the sum of the slots, placed by `origin`.
//! - **The baseline is half-leading**: `slot_centre_y + (ascent − descent) / 2`, with
//!   `ascent` and `descent` each **the maximum across every run on the line** and not the
//!   metrics of the one run that sets the slot height (ADR-0029).
//! - **The extent is the stroked one**, not the typographic one: on text the stroke falls
//!   *outside* the glyph contour, so the extent gains `2 × stroke_width` on both axes
//!   (ADR-0014). Returning stroke-naive numbers is the failure ADR-0014 names by name —
//!   *"every author adds `2 × stroke_width` by hand and they diverge"*.
//! - **Break opportunities** come back per line, from [`crate::breaks`], with the segmenter
//!   named (ADR-0008).
//! - **Each line's real inked extent**, and the seam between adjacent lines' ink, from
//!   [`crate::ink`] (ADR-0087). The slot above is a function of two declared numbers and
//!   never of the font's ink; this is the ink, stated beside it so the two can be compared
//!   at all. It changes no slot and moves no baseline. **The seam is compared per glyph,
//!   where two glyphs share horizontal space** — a whole-line comparison is a
//!   false-positive generator, and [`crate::ink`] records what it cost to learn that.
//! - **A run's `dir` is an isolate** (ADR-0007, ADR-0133): the run is laid out as though
//!   wrapped in LRI/RLI…PDI, and the line's base direction stays the one the author's own
//!   characters give. The marks exist only in the string handed to `parley` — see
//!   [`Laid`] — so no offset, line text or count published here ever includes one.
//!
//! # No verdict, ever
//!
//! ADR-0024 is explicit that `measure` writes the repair and never the verdict, so nothing
//! here compares what it computed against anything the document declares. It never sees
//! the declared `width`/`height`, which is the structural form of that rule: there is no
//! value here to compare against. Judging the document is `validate`'s alone (ADR-0006).
//!
//! # Where the exactness is, and where it deliberately is not
//!
//! Every vertical coordinate is derived in **exact integer twentieths of a pixel** and
//! converted to `f64` once, at the end. That is not decoration: `line_height` is restricted
//! to tenths precisely so the block arithmetic is exact (ADR-0028), a slot's centre halves
//! that tenth, and twentieths is the coarsest unit in which every intermediate is still an
//! integer. The block **height** — the integer that fills a text element's required
//! `height` field — is not computed here at all: [`Measurement::block_height_tenths`] is
//! handed back exactly, and the `ceil` that turns it into that integer is
//! `montagent_core::exact`'s one implementation of ADR-0028's formula, so `measure` and
//! `validate`'s `R-BOX-SLACK` cannot carry two.
//!
//! `baseline_y` itself is `f64`, and ADR-0029 says so in as many words: it *"feeds no
//! boundary-sensitive operation … it is a continuous coordinate consumed by antialiased
//! rasterization"*, the same exemption ADR-0028 carves out for `scale`/`rotation`.
//! Ascent and descent are font facts read at layout time and are `f64` from the font.

use std::borrow::Cow;
use std::ops::Range;

use parley::{
    Alignment, AlignmentOptions, FontFamily, FontFeatures, Layout, LayoutContext, StyleProperty,
};
use serde::Serialize;

use crate::breaks::{SEGMENTER, Segmenter, opportunities};
use crate::fonts::{FontError, Fonts};
use crate::ink::{InkSeam, LineInk, line_ink, placed, seam_between};
use crate::lines::partition;
use crate::place::{Align, offset};
use crate::spacing::{gaps, non_joining, shifts};

/// One stretch of a text element's content, with its style deltas over the base
/// (ADR-0007).
///
/// Only the deltas measurement can see. A colour or a highlight window changes no number
/// here, so carrying them would be carrying fields this module must then be trusted to
/// ignore.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Run<'a> {
    pub text: &'a str,
    /// A key into the project's `fonts` table, overriding the element's.
    pub font: Option<&'a str>,
    pub size: Option<i64>,
    /// ADR-0014 makes `stroke` a run-addressable paint, so the stroked extent is too.
    pub stroke_width: Option<i64>,
    /// A direction override with **isolate** semantics only (ADR-0007): the run is laid out
    /// as though wrapped in LRI/RLI…PDI, so it reorders nothing outside itself and never
    /// changes the line's base direction (ADR-0133).
    pub dir: Option<Dir>,
}

/// A run's direction override (ADR-0007). Never LRO/RLO: there is no way to spell one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir {
    Ltr,
    Rtl,
}

/// Which part of the block its `y` places — `origin`'s vertical component (ADR-0013).
///
/// The vertical half alone, because the vertical half is all the baseline needs. The block
/// is placed by `origin` (ADR-0007), which fixes the same point on the element's box and on
/// the block, so the two readings of *"placed according to `origin`"* agree on every one of
/// the nine keywords.
///
/// Spelled `VerticalOrigin` rather than the shorter word `CONTEXT.md` puts on **Origin**'s
/// avoid-list: in this project an **Anchor** is a layer stated relative to another
/// element's `id`, and a second meaning would be the `title`-the-track-or-the-element
/// collision the glossary already records once.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerticalOrigin {
    Top,
    Center,
    Bottom,
}

/// One text element, as much of it as measurement can see.
#[derive(Debug, Clone)]
pub struct Spec<'a> {
    pub runs: &'a [Run<'a>],
    /// The element's base font: a key into the project's `fonts` table (ADR-0007).
    pub font: &'a str,
    /// The element's base `size`, a literal integer — the renderer never chooses one.
    pub size: i64,
    /// `line_height × 10`, which ADR-0028 restricts to an integer by restricting the field
    /// to tenths. Never recovered from an `f64` product.
    pub line_height_tenths: i64,
    /// The element's base `stroke_width`, in element space.
    pub stroke_width: i64,
    /// The `y` the block is placed at, through [`Spec::vertical_origin`].
    pub y: i64,
    pub vertical_origin: VerticalOrigin,
    /// How the lines sit against each other (ADR-0007) — never how the block is placed.
    ///
    /// Here rather than only on [`crate::place`] because **the ink seam needs it**
    /// (ADR-0087, second court): two lines' glyphs must be in one horizontal frame before
    /// their ink can be compared, and `align` is what puts them there. `x`, `origin` and the
    /// declared `width` are still absent and still cancel — the seam is a difference between
    /// two lines of one block, so where that block sits moves both of them equally.
    pub align: Align,
    /// The element's `letter_spacing` at the instant being laid out, in thousandths of an em,
    /// already resolved (ADR-0151). Continuous and never rounded: a keyed spacing is resolved
    /// by the caller, per frame, and arrives here as the value at that instant.
    ///
    /// After every grapheme of a line but the last, `size × letter_spacing / 1000` pixels are
    /// added, where `size` is the size of the run the grapheme sits in — except between two
    /// letters of the same joining script (ADR-0153 §4). See [`crate::spacing`].
    pub letter_spacing: f64,
    /// Whether the element is shaped with its optional ligatures (`liga`, `clig`, `dlig`) off
    /// in every script run that is not a joining script (ADR-0151 §1, ADR-0153 §3).
    ///
    /// Decided by the caller **from the file, never from the instant**: a spacing animation
    /// passing through 0 must not swap glyphs mid-shot, so this is not derived from
    /// [`Spec::letter_spacing`].
    pub optional_ligatures_off: bool,
}

/// One line's measurement.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MeasuredLine {
    /// Its position in the block, from zero.
    pub index: usize,
    /// The line itself, with its mandatory break not included.
    pub text: String,
    /// Where the line starts, as a byte offset into the element's whole text.
    pub start: usize,
    /// The typographic advance width, before the stroke.
    pub advance_width: f64,
    /// The **maximum** ascent across every run on the line, not the largest run's
    /// (ADR-0029).
    pub ascent: f64,
    /// The maximum descent across every run on the line, on the same rule.
    pub descent: f64,
    /// The largest `size` among the runs on this line — the one that sets the slot's
    /// height (ADR-0007).
    pub size: i64,
    /// The slot this line reserves: `size × line_height`.
    pub slot_top: f64,
    pub slot_height: f64,
    /// Where the glyphs sit: `slot_centre_y + (ascent − descent) / 2` (ADR-0029).
    pub baseline_y: f64,
    /// The largest `stroke_width` among the runs on this line.
    pub stroke_width: i64,
    /// `advance_width + 2 × stroke_width` — the stroked width, which is the one an
    /// overflow question is asked about (ADR-0014).
    pub extent_width: f64,
    /// Where this line's glyphs **actually** start and end vertically, in the block's own
    /// coordinates — the real ink, not the slot (ADR-0087).
    ///
    /// `null` for a line that draws nothing: an empty line, or a line of spaces. A blank
    /// line reserves its slot exactly as before and simply has no ink to report, which is
    /// a different fact from having ink of zero height at the baseline.
    pub ink_top: Option<f64>,
    pub ink_bottom: Option<f64>,
    /// Every byte offset, into the element's whole text, at which this line may legally
    /// break (ADR-0008). Never a place Montagent would break it: nothing wraps.
    pub break_opportunities: Vec<usize>,
    /// This line's base direction — what `align`'s `start`/`end` resolve against
    /// (ADR-0133). UAX #9's P2/P3 over the author's own characters on the line, which a run's
    /// `dir` never changes.
    ///
    /// Not published: `measure`'s wire shape is unchanged by #457. It is here for the
    /// callers that align a line themselves — `query`'s ink box — so they read the
    /// direction the painter used rather than guessing one from the characters.
    #[serde(skip)]
    pub rtl: bool,
}

/// What one text element occupies.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Measurement {
    /// `mandatory breaks + 1`, always (ADR-0008).
    pub line_count: usize,
    /// The widest line's typographic advance, before the stroke.
    pub advance_width: f64,
    /// The largest ascent on any line, and the largest descent — the block's own, for a
    /// caller asking about the block rather than about a line.
    pub ascent: f64,
    pub descent: f64,
    /// The block's height in exact tenths of a pixel: `Σ (largest size on line ×
    /// line_height × 10)`. The integer a `height` field takes is `ceil` of this, and that
    /// `ceil` is ADR-0028's, evaluated once in `montagent_core::exact`.
    ///
    /// `i128`, like every intermediate below it. `size` is read off a document and carries
    /// no bound of its own, and an `i64` product of three such numbers is exactly the
    /// overflow `crate::checks::box_slack` already widened away from — a panic in a debug
    /// build and a wrapped, plausible number in a release one.
    pub block_height_tenths: i128,
    /// Where the block sits, from [`Spec::y`] through [`Spec::vertical_origin`].
    pub block_top: f64,
    pub block_bottom: f64,
    /// The **stroked** extent (ADR-0014), which is what an author compares a box against.
    pub extent: Extent,
    /// The topmost and bottommost ink anywhere in the block, for a caller asking about the
    /// block rather than about a line. `null` where the block draws nothing at all.
    ///
    /// **Not clamped to the block**, and that is the point: ADR-0087 records that on the
    /// Thai faces it measured, the first line's ink escapes `block_top` by 8.45 px at
    /// `line_height` 1.1 — a number that would be invisible if this were reported as the
    /// intersection of the ink with the slots rather than as the ink.
    pub ink_top: Option<f64>,
    pub ink_bottom: Option<f64>,
    /// The seam between each adjacent pair of **inked** lines (ADR-0087). Empty for a block
    /// of one line, and for one whose lines draw nothing.
    pub ink_seams: Vec<InkSeam>,
    pub lines: Vec<MeasuredLine>,
    /// Which segmenter produced every `break_opportunities` above (ADR-0008).
    pub segmenter: Segmenter,
}

/// The stroked extent: the typographic block grown by the stroke that falls outside the
/// glyph contours (ADR-0014).
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Extent {
    pub width: f64,
    pub height: f64,
    /// The largest `stroke_width` anywhere in the element — what `width` and `height` were
    /// each grown by twice. Stated rather than left to be inferred, so the typographic
    /// numbers above stay recoverable from the same object.
    pub stroke_width: i64,
}

/// A pixel, in the exact integer unit every vertical coordinate is derived in.
///
/// Twentieths: `line_height` is tenths (ADR-0028), and a slot's centre halves one.
const UNIT: i128 = 20;

/// Measure one text element.
///
/// The only failure is a font the project does not declare or cannot open — measurement
/// itself has no failure mode, because it reaches no conclusion that could be wrong about
/// the document.
pub fn measure(fonts: &mut Fonts, spec: &Spec<'_>) -> Result<Measurement, FontError> {
    Ok(measured(fonts, spec)?.0)
}

/// One line's shaped layout, and what letter spacing moves each of its glyphs by — in the
/// order the layout yields them ([`crate::spacing::shifts`]). Every reader of a glyph's `x`
/// adds the shift, so the ink, the seam and the drawn glyph all sit on the spaced line.
///
/// It also keeps every cluster of the line, with its text mapped back to the element's whole
/// text, and which cluster each glyph came from: a stagger draws each glyph with the unit
/// its text belongs to (ADR-0151 §3).
pub(crate) struct Shaped {
    pub(crate) layout: Layout<u32>,
    pub(crate) shifts: Vec<f64>,
    /// Every cluster, in visual order, with its text in the element's whole text — `None`
    /// for one an isolate mark makes, which is no character of the author's.
    pub(crate) clusters: Vec<(crate::spacing::LaidCluster, Option<Range<usize>>)>,
    /// Which of `clusters` each glyph came from, in the order `shifts` is in.
    pub(crate) glyph_clusters: Vec<usize>,
}

/// The same measurement, with the shaped layouts it was derived from kept.
///
/// [`crate::place`] needs both — the block arithmetic *and* the glyphs that arithmetic
/// placed — and it must get them from the same pass. Re-shaping the text a second time to
/// draw it would be a second line partition and a second set of baselines, which is
/// exactly the thing #213 forbids: *"no second line-breaking implementation exists"*. So
/// the two public entry points are wrappers over this, and `measure` is the one that
/// throws the layouts away.
pub(crate) fn measured(
    fonts: &mut Fonts,
    spec: &Spec<'_>,
) -> Result<(Measurement, Vec<Shaped>), FontError> {
    // Every chain the element names, resolved before anything is laid out: a run naming an
    // undeclared key is the same error whether it is the first run or the last, and
    // discovering it half-way through would leave a partial answer to throw away.
    let base_chain = fonts.chain(spec.font)?;
    let mut run_chains = Vec::with_capacity(spec.runs.len());
    for run in spec.runs {
        run_chains.push(match run.font {
            Some(key) => Some(fonts.chain(key)?),
            None => None,
        });
    }

    // The element's whole text, with each run's place in it. Runs concatenate: a run
    // boundary is style only, and a line break is a `\n` inside a run's text (ADR-0007).
    let mut full = String::new();
    let mut run_ranges = Vec::with_capacity(spec.runs.len());
    for run in spec.runs {
        let start = full.len();
        full.push_str(run.text);
        run_ranges.push(start..full.len());
    }

    let lines = partition(&full);
    let mut layout_context: LayoutContext<u32> = LayoutContext::new();

    // Pass one: each line's own metrics, in document order. The block's geometry needs
    // every slot height before any baseline can be placed, so the vertical coordinates are
    // a second pass rather than something accumulated here.
    let mut shaped = Vec::with_capacity(lines.len());
    let mut layouts = Vec::with_capacity(lines.len());
    for line in &lines {
        // Overlap, strictly: a run contributing no character to this line is not on it, so
        // an empty run sets no slot height and a run that ends exactly where the line
        // begins belongs to the line before. The empty line a trailing mandatory break
        // leaves therefore has no run at all, and falls back to the element's base style —
        // which is what ADR-0007's base-plus-deltas model says an absent delta means.
        let on_line: Vec<usize> = (0..spec.runs.len())
            .filter(|&i| run_ranges[i].start < line.end() && run_ranges[i].end > line.start)
            .collect();

        // Each on-line run's stretch of this line, in the line's own offsets.
        let pieces: Vec<Range<usize>> = on_line
            .iter()
            .map(|&i| {
                run_ranges[i].start.max(line.start) - line.start
                    ..run_ranges[i].end.min(line.end()) - line.start
            })
            .collect();
        let style = Styles {
            base: &base_chain,
            size: spec.size,
            runs: spec.runs,
            chains: &run_chains,
            on_line: &on_line,
            optional_ligatures_off: spec.optional_ligatures_off,
        };
        let plain = Laid::plain(line.text, &pieces);
        let mut layout = style.lay_out(&mut layout_context, fonts, &plain);
        let mut marks: Vec<Range<usize>> = Vec::new();
        let mut copies = plain.copies.clone();
        // A `dir` override is laid out a second time, as an isolate, in the base direction
        // the plain layout just read off the author's characters (ADR-0133). Only then: a
        // line that sets no `dir` is laid out from exactly the string it always was, so no
        // frame that does not use the field can move.
        if on_line.iter().any(|&i| spec.runs[i].dir.is_some()) {
            let dirs: Vec<Option<Dir>> = on_line.iter().map(|&i| spec.runs[i].dir).collect();
            let isolated = Laid::isolated(line.text, &pieces, &dirs, layout.is_rtl());
            layout = style.lay_out(&mut layout_context, fonts, &isolated);
            marks = isolated.marks;
            copies = isolated.copies;
        }

        // ADR-0151's letter spacing, added after shaping so the shaping is untouched: each
        // grapheme's gap is measured against the size of the run it sits in.
        let line_gaps = gaps(line.text, spec.letter_spacing, |at| {
            pieces
                .iter()
                .position(|piece| piece.contains(&at))
                .and_then(|p| spec.runs[on_line[p]].size)
                .unwrap_or(spec.size)
        });
        let walked = shifts(&layout, &line_gaps, |at| to_laid(&copies, at));
        let (line_shifts, spaced) = (walked.shifts, walked.moved);
        let line_clusters: Vec<_> = walked
            .clusters
            .into_iter()
            .map(|cluster| {
                let text = from_laid(&copies, cluster.laid.start)
                    .map(|at| line.start + at..line.start + at + cluster.laid.len());
                (cluster, text)
            })
            .collect();

        // ADR-0029's max-across-every-run, read off the runs rather than off parley's own
        // line metrics. Two reasons to spell it out: the rule is a decision this project
        // made against a 2–1 jury and should be visible where it is applied, and a line
        // carrying no glyph at all (the empty line a trailing break leaves) has no run to
        // read, which is the case the fallback below answers.
        let mut ascent: f64 = 0.0;
        let mut descent: f64 = 0.0;
        let mut advance = 0.0;
        for placed in layout.lines() {
            advance += f64::from(placed.metrics().advance);
            for run in placed.runs() {
                // A run of nothing but isolate marks is not a run the author wrote. Its
                // metrics are its style's, which an author's run already carries; skipped
                // all the same, so a face the fallback chose for an invisible mark can never
                // set a line's ascent (ADR-0029 is about the runs *on the line*).
                if only_marks(run.text_range(), &marks) {
                    continue;
                }
                ascent = ascent.max(f64::from(run.metrics().ascent));
                descent = descent.max(f64::from(run.metrics().descent));
            }
        }
        // An empty line advances by nothing. parley lays an empty paragraph out as one
        // synthetic whitespace cluster — a caret needs somewhere to sit — so its reported
        // advance is a space's, and taking it would report a width for a line carrying no
        // character. The *metrics* of that cluster are the base style's and are exactly
        // what an empty line's slot should be read from, so the layout is still run; only
        // the advance is dropped.
        //
        // Emptiness, not whitespace: the whitespace an author wrote is content, and
        // ADR-0007 is explicit that the fixture's `cobweb  -  cobweb` "carries deliberate
        // double spaces that no tidying pass may touch". A line of one space measures one
        // space wide.
        if line.text.is_empty() {
            advance = 0.0;
        }
        // The spaced advance is the advance (ADR-0151): the block, `align`, `origin` and the
        // ink box all measure the spaced line.
        advance += spaced;

        // ADR-0007: "A line's height is *the largest `size` among the runs on that line* x
        // `line_height`." The runs on the line, resolved — so the element's base `size`
        // counts where a run does not override it, and **not** where every run does. A base
        // of 88 with every run on the line at 40 reserves 40's slot, not 88's: the base is a
        // default under ADR-0007's base-plus-deltas model, and a default a delta has
        // replaced is not still in force.
        let size = greatest_of(&on_line, spec.size, |run| run.size, spec.runs);
        // ADR-0014 makes stroke a run-addressable paint, so the stroked extent resolves on
        // exactly the same rule.
        let stroke_width = greatest_of(
            &on_line,
            spec.stroke_width,
            |run| run.stroke_width,
            spec.runs,
        );

        shaped.push(LineMetrics {
            advance,
            ascent,
            descent,
            size,
            stroke_width,
            // Read off the layout rather than guessed from the characters. Under a `dir`
            // override the isolated layout carries the plain one's direction by its leading
            // mark, so this is the direction the line has without the override (ADR-0133).
            rtl: layout.is_rtl(),
            // Read off the layout this pass just built, for [`crate::place`]'s reason: the
            // glyphs whose ink is being measured must be the glyphs that will be drawn, and
            // a second shaping pass to find them would be a second answer to where they are.
            ink: line_ink(&layout, &line_shifts),
        });
        layouts.push(Shaped {
            layout,
            shifts: line_shifts,
            clusters: line_clusters,
            glyph_clusters: walked.glyph_clusters,
        });
    }

    // Pass two: the block, and every coordinate inside it.
    let slots: Vec<i128> = shaped
        .iter()
        .map(|line| i128::from(line.size) * i128::from(spec.line_height_tenths))
        .collect();
    // In twentieths, so that the block's half — which `center` needs — stays an integer.
    let block = slots.iter().sum::<i128>() * 2;
    let block_top = i128::from(spec.y) * UNIT
        - match spec.vertical_origin {
            VerticalOrigin::Top => 0,
            VerticalOrigin::Center => block / 2,
            VerticalOrigin::Bottom => block,
        };

    let mut measured = Vec::with_capacity(lines.len());
    let mut slot_top = block_top;
    for (index, (line, shaped)) in lines.iter().zip(&shaped).enumerate() {
        let slot = slots[index] * 2;
        // The slot's own centre, exactly: `slot / 2` is `size × line_height × 10`, an
        // integer by construction.
        let centre = slot_top + slots[index];
        let baseline_y = pixels(centre) + (shaped.ascent - shaped.descent) / 2.0;
        measured.push(MeasuredLine {
            index,
            text: line.text.to_string(),
            start: line.start,
            advance_width: shaped.advance,
            ascent: shaped.ascent,
            descent: shaped.descent,
            size: shaped.size,
            slot_top: pixels(slot_top),
            slot_height: pixels(slot),
            baseline_y,
            stroke_width: shaped.stroke_width,
            // ADR-0029's baseline plus the line's own ink offsets — the one place the two
            // halves meet, so there is no second derivation of either.
            ink_top: shaped.ink.as_ref().map(|ink| baseline_y + ink.top),
            ink_bottom: shaped.ink.as_ref().map(|ink| baseline_y + ink.bottom),
            extent_width: shaped.advance + 2.0 * shaped.stroke_width as f64,
            break_opportunities: opportunities(line.text)
                .into_iter()
                .map(|at| line.start + at)
                .collect(),
            rtl: shaped.rtl,
        });
        slot_top += slot;
    }

    let stroke_width = measured
        .iter()
        .map(|line| line.stroke_width)
        .max()
        .unwrap_or(spec.stroke_width);

    let ink_top = measured
        .iter()
        .filter_map(|line| line.ink_top)
        .reduce(f64::min);
    let ink_bottom = measured
        .iter()
        .filter_map(|line| line.ink_bottom)
        .reduce(f64::max);

    // The seam, per glyph and in the block's own horizontal frame (ADR-0087, second court).
    //
    // The alignment offset needs the block's width, which is the widest line's advance and
    // is not known until every line is shaped — so this sits here, after pass two, rather
    // than beside the vertical coordinates it is reported next to.
    let block_width = greatest(&measured, |line| line.advance_width);
    let inked: Vec<(usize, Vec<crate::ink::GlyphInk>)> = measured
        .iter()
        .zip(&shaped)
        .filter_map(|(line, shaped)| {
            let ink = shaped.ink.as_ref()?;
            // `crate::place::offset`'s, never a second copy: the glyphs a seam is measured
            // between must be the glyphs the renderer draws, at the same offsets.
            let dx = offset(spec.align, shaped.rtl, block_width, shaped.advance);
            Some((
                line.index,
                placed(ink, dx, line.baseline_y, line.stroke_width),
            ))
        })
        .collect();
    let ink_seams: Vec<InkSeam> = inked
        .windows(2)
        .map(|pair| InkSeam {
            above: pair[0].0,
            below: pair[1].0,
            overlap: seam_between(&pair[0].1, &pair[1].1),
        })
        .collect();

    Ok((
        Measurement {
            line_count: measured.len(),
            advance_width: greatest(&measured, |line| line.advance_width),
            ascent: greatest(&measured, |line| line.ascent),
            descent: greatest(&measured, |line| line.descent),
            block_height_tenths: slots.iter().sum(),
            block_top: pixels(block_top),
            block_bottom: pixels(block_top + block),
            extent: Extent {
                width: greatest(&measured, |line| line.extent_width),
                height: pixels(block) + 2.0 * stroke_width as f64,
                stroke_width,
            },
            ink_top,
            ink_bottom,
            ink_seams,
            lines: measured,
            segmenter: SEGMENTER,
        },
        layouts,
    ))
}

/// The string one line is laid out from, and where each on-line run sits in it.
///
/// For a line that sets no `dir` this is the line itself. For one that does, it is the line
/// with **marks** spliced in: LRI/RLI…PDI around each overridden run (ADR-0007's isolate),
/// and one LRM or RLM in front, which pins the base direction to the one the line has
/// without any override (ADR-0133) — UAX #9 P2 skips an isolate's contents, so a line that
/// is all one overridden run would otherwise lose its direction to the marks around it.
///
/// The marks are zero-width default-ignorables, so shaping hides them and they advance by
/// nothing. They live only here: every offset `measure` publishes is into the author's
/// text, and is computed from [`crate::lines::Line`] rather than from this string.
struct Laid<'a> {
    text: Cow<'a, str>,
    /// One range per on-line run, in [`Laid::text`]'s offsets, marks included — so a
    /// mark is styled and brushed as the run it belongs to, and never falls back to the
    /// element's base `size`, which the run may have replaced.
    ranges: Vec<Range<usize>>,
    /// Every inserted mark, in [`Laid::text`]'s offsets.
    marks: Vec<Range<usize>>,
    /// Where each stretch of the author's line was copied to: `(range in the line, start in
    /// [`Laid::text`])`, so a line offset can be found in the laid-out string.
    copies: Vec<(Range<usize>, usize)>,
}

/// Where an offset in the laid-out string sits in the author's line: `None` inside an
/// inserted isolate mark.
fn from_laid(copies: &[(Range<usize>, usize)], at: usize) -> Option<usize> {
    copies
        .iter()
        .find(|(range, start)| at >= *start && at < start + range.len())
        .map(|(range, start)| range.start + (at - start))
}

/// Where a line offset sits in the laid-out string.
fn to_laid(copies: &[(Range<usize>, usize)], at: usize) -> usize {
    copies
        .iter()
        .find(|(range, _)| range.contains(&at))
        .map_or(at, |(range, start)| start + (at - range.start))
}

/// U+2066 LEFT-TO-RIGHT ISOLATE, U+2067 RIGHT-TO-LEFT ISOLATE, U+2069 POP DIRECTIONAL
/// ISOLATE — never LRE/RLE/LRO/RLO, which ADR-0007 rules out.
const LRI: char = '\u{2066}';
const RLI: char = '\u{2067}';
const PDI: char = '\u{2069}';
/// U+200E LEFT-TO-RIGHT MARK and U+200F RIGHT-TO-LEFT MARK.
const LRM: char = '\u{200E}';
const RLM: char = '\u{200F}';

impl<'a> Laid<'a> {
    fn plain(text: &'a str, pieces: &[Range<usize>]) -> Laid<'a> {
        Laid {
            text: Cow::Borrowed(text),
            ranges: pieces.to_vec(),
            marks: Vec::new(),
            copies: vec![(0..text.len(), 0)],
        }
    }

    /// `pieces` and `dirs` are parallel, one entry per on-line run; `rtl` is the base
    /// direction the line has without any override.
    fn isolated(
        text: &str,
        pieces: &[Range<usize>],
        dirs: &[Option<Dir>],
        rtl: bool,
    ) -> Laid<'static> {
        let mut out = String::with_capacity(text.len() + 3 * (2 * pieces.len() + 1));
        let mut ranges = Vec::with_capacity(pieces.len());
        let mut marks = Vec::new();
        let mut copies = Vec::with_capacity(pieces.len());
        let mut mark = |out: &mut String, c: char| {
            let at = out.len();
            out.push(c);
            marks.push(at..out.len());
        };
        // The base-direction mark is styled as the first run, since it sits at its start.
        mark(&mut out, if rtl { RLM } else { LRM });
        for (index, (piece, dir)) in pieces.iter().zip(dirs).enumerate() {
            let start = if index == 0 { 0 } else { out.len() };
            if let Some(dir) = dir {
                mark(
                    &mut out,
                    match dir {
                        Dir::Ltr => LRI,
                        Dir::Rtl => RLI,
                    },
                );
            }
            copies.push((piece.clone(), out.len()));
            out.push_str(&text[piece.clone()]);
            if dir.is_some() {
                mark(&mut out, PDI);
            }
            ranges.push(start..out.len());
        }
        Laid {
            text: Cow::Owned(out),
            ranges,
            marks,
            copies,
        }
    }
}

/// Whether a stretch of the laid-out string holds nothing but marks.
///
/// Always `false` for a line with no marks, so a plain line skips no run, however empty.
fn only_marks(range: Range<usize>, marks: &[Range<usize>]) -> bool {
    let mut at = range.start;
    while at < range.end {
        match marks.iter().find(|mark| mark.start == at) {
            Some(mark) => at = mark.end,
            None => return false,
        }
    }
    !marks.is_empty()
}

/// What one line's layout is styled from — the same for its plain and its isolated string.
struct Styles<'s, 'a> {
    base: &'s FontFamily<'static>,
    size: i64,
    runs: &'s [Run<'a>],
    chains: &'s [Option<FontFamily<'static>>],
    on_line: &'s [usize],
    /// ADR-0151's rule, read from the file by the caller: see [`Spec::optional_ligatures_off`].
    optional_ligatures_off: bool,
}

/// The optional ligatures ADR-0151 switches off: `liga`, `clig` and `dlig`. Never `rlig`,
/// which a script needs to be spelled correctly.
///
/// In `font-feature-settings` syntax, which parley parses; its `Tag` type is not exported.
const OPTIONAL_LIGATURES_OFF: &str = r#""liga" 0, "clig" 0, "dlig" 0"#;

impl Styles<'_, '_> {
    fn lay_out(
        &self,
        layout_context: &mut LayoutContext<u32>,
        fonts: &mut Fonts,
        laid: &Laid<'_>,
    ) -> Layout<u32> {
        let text: &str = &laid.text;
        let mut builder =
            layout_context.ranged_builder(fonts.context(), text, 1.0, /* quantize */ false);
        builder.push_default(StyleProperty::FontFamily(self.base.clone()));
        builder.push_default(StyleProperty::FontSize(self.size as f32));
        for (&i, range) in self.on_line.iter().zip(&laid.ranges) {
            if let Some(chain) = &self.chains[i] {
                builder.push(StyleProperty::FontFamily(chain.clone()), range.clone());
            }
            if let Some(size) = self.runs[i].size {
                builder.push(StyleProperty::FontSize(size as f32), range.clone());
            }
            // The run's index, carried through the layout as parley's *brush*. Measurement
            // has no use for it — a colour changes no number here — but drawing does, and
            // the brush is the only channel that survives shaping: a glyph knows which
            // cluster it came from, and the brush is how that cluster says which run's
            // `color` and `stroke` it wears (ADR-0014's run-addressable paint). Pushed in
            // the shared pass rather than in a second one, because a second pass is a
            // second answer to "which run is this glyph". (A line with a `dir` is laid out
            // twice, but only the isolated layout is kept; the plain one is read for its
            // direction and nothing else.)
            builder.push(StyleProperty::Brush(i as u32), range.clone());
        }
        // ADR-0153 §3: off only in the script runs that are not a joining script, so an
        // Arabic lam-alef a font files under `liga` survives. A script boundary is already a
        // shaping boundary, so this splits no shaping item that was not split anyway.
        if self.optional_ligatures_off {
            for range in non_joining(text) {
                builder.push(
                    StyleProperty::FontFeatures(FontFeatures::Source(Cow::Borrowed(
                        OPTIONAL_LIGATURES_OFF,
                    ))),
                    range,
                );
            }
        }
        let mut layout: Layout<u32> = builder.build(text);
        // `None` is "no wrap width": the renderer never chooses a line break (ADR-0007),
        // and the partition above has already placed every break there is.
        layout.break_all_lines(None);
        layout.align(Alignment::Start, AlignmentOptions::default());
        layout
    }
}

/// One line's own metrics, before it knows where in the block it sits.
struct LineMetrics {
    advance: f64,
    ascent: f64,
    descent: f64,
    size: i64,
    stroke_width: i64,
    /// This line's own base direction, for the alignment offset (ADR-0007's `start`/`end`).
    rtl: bool,
    /// Relative to this line's own origin and baseline; the block's coordinates are added in
    /// pass two, which is where both the baseline and the block's width are known.
    ink: Option<LineInk>,
}

/// A coordinate in twentieths, as pixels.
fn pixels(units: i128) -> f64 {
    units as f64 / UNIT as f64
}

/// The largest of one measurement across every line, or zero where there is no line —
/// which [`partition`] never produces, since it emits at least one.
fn greatest(lines: &[MeasuredLine], of: impl Fn(&MeasuredLine) -> f64) -> f64 {
    lines.iter().map(of).fold(0.0, f64::max)
}

/// The largest value one style delta takes across the runs on a line, falling back to the
/// element's base **only where a run leaves the delta unstated**.
///
/// The fallback's placement is the whole point, and getting it wrong over-reports: a base
/// that every run on the line has overridden is a default that has been replaced, not a
/// floor the line still has to clear. ADR-0007's model is base-plus-deltas, and *"a delta
/// that is absent is a delta that was not made"* — so absence is what admits the base, and
/// nothing else does.
fn greatest_of(
    on_line: &[usize],
    base: i64,
    delta: impl Fn(&Run<'_>) -> Option<i64>,
    runs: &[Run<'_>],
) -> i64 {
    // A line carrying no run at all — the empty line a trailing mandatory break leaves —
    // is the base's, for the same reason: it states no delta.
    let any_unstated = on_line.is_empty() || on_line.iter().any(|&i| delta(&runs[i]).is_none());
    on_line
        .iter()
        .filter_map(|&i| delta(&runs[i]))
        .chain(any_unstated.then_some(base))
        .max()
        .unwrap_or(base)
}
