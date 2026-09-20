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
//! `montaget_core::exact`'s one implementation of ADR-0028's formula, so `measure` and
//! `validate`'s `R-BOX-SLACK` cannot carry two.
//!
//! `baseline_y` itself is `f64`, and ADR-0029 says so in as many words: it *"feeds no
//! boundary-sensitive operation … it is a continuous coordinate consumed by antialiased
//! rasterization"*, the same exemption ADR-0028 carves out for `scale`/`rotation`.
//! Ascent and descent are font facts read at layout time and are `f64` from the font.

use parley::{Alignment, AlignmentOptions, Layout, LayoutContext, StyleProperty};
use serde::Serialize;

use crate::breaks::{SEGMENTER, Segmenter, opportunities};
use crate::fonts::{FontError, Fonts};
use crate::lines::partition;

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
}

/// Which part of the block its `y` places — `origin`'s vertical component (ADR-0013).
///
/// The vertical half alone, because the vertical half is all the baseline needs. The block
/// is placed by `origin` (ADR-0007), and `origin` anchors the element's box and the block
/// to the same point, so the two readings of *"placed according to `origin`"* agree on
/// every one of the nine keywords.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anchor {
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
    /// The `y` the block is placed at, through [`Spec::anchor`].
    pub y: i64,
    pub anchor: Anchor,
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
    /// Every byte offset, into the element's whole text, at which this line may legally
    /// break (ADR-0008). Never a place Montaget would break it: nothing wraps.
    pub break_opportunities: Vec<usize>,
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
    /// `ceil` is ADR-0028's, evaluated once in `montaget_core::exact`.
    ///
    /// `i128`, like every intermediate below it. `size` is read off a document and carries
    /// no bound of its own, and an `i64` product of three such numbers is exactly the
    /// overflow `crate::checks::box_slack` already widened away from — a panic in a debug
    /// build and a wrapped, plausible number in a release one.
    pub block_height_tenths: i128,
    /// Where the block sits, from [`Spec::y`] through [`Spec::anchor`].
    pub block_top: f64,
    pub block_bottom: f64,
    /// The **stroked** extent (ADR-0014), which is what an author compares a box against.
    pub extent: Extent,
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
    let mut spans = Vec::with_capacity(spec.runs.len());
    for run in spec.runs {
        let start = full.len();
        full.push_str(run.text);
        spans.push(start..full.len());
    }

    let lines = partition(&full);
    let mut layout_context: LayoutContext<()> = LayoutContext::new();

    // Pass one: each line's own metrics, in document order. The block's geometry needs
    // every slot height before any baseline can be placed, so the vertical coordinates are
    // a second pass rather than something accumulated here.
    let mut shaped = Vec::with_capacity(lines.len());
    for line in &lines {
        // Overlap, strictly: a run contributing no character to this line is not on it, so
        // an empty run sets no slot height and a run that ends exactly where the line
        // begins belongs to the line before. The empty line a trailing mandatory break
        // leaves therefore has no run at all, and falls back to the element's base style —
        // which is what ADR-0007's base-plus-deltas model says an absent delta means.
        let on_line: Vec<usize> = (0..spec.runs.len())
            .filter(|&i| spans[i].start < line.end() && spans[i].end > line.start)
            .collect();

        let mut builder = layout_context.ranged_builder(
            fonts.context(),
            line.text,
            1.0,
            /* quantize */ false,
        );
        builder.push_default(StyleProperty::FontFamily(base_chain.clone()));
        builder.push_default(StyleProperty::FontSize(spec.size as f32));
        for &i in &on_line {
            let start = spans[i].start.max(line.start) - line.start;
            let end = spans[i].end.min(line.end()) - line.start;
            if let Some(chain) = &run_chains[i] {
                builder.push(StyleProperty::FontFamily(chain.clone()), start..end);
            }
            if let Some(size) = spec.runs[i].size {
                builder.push(StyleProperty::FontSize(size as f32), start..end);
            }
        }
        let mut layout: Layout<()> = builder.build(line.text);
        // `None` is "no wrap width": the renderer never chooses a line break (ADR-0007),
        // and the partition above has already placed every break there is.
        layout.break_all_lines(None);
        layout.align(Alignment::Start, AlignmentOptions::default());

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

        let size = on_line
            .iter()
            .filter_map(|&i| spec.runs[i].size)
            .chain(std::iter::once(spec.size))
            .max()
            .unwrap_or(spec.size);
        let stroke_width = on_line
            .iter()
            .filter_map(|&i| spec.runs[i].stroke_width)
            .chain(std::iter::once(spec.stroke_width))
            .max()
            .unwrap_or(spec.stroke_width);

        shaped.push(Shaped {
            advance,
            ascent,
            descent,
            size,
            stroke_width,
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
        - match spec.anchor {
            Anchor::Top => 0,
            Anchor::Center => block / 2,
            Anchor::Bottom => block,
        };

    let mut measured = Vec::with_capacity(lines.len());
    let mut slot_top = block_top;
    for (index, (line, shaped)) in lines.iter().zip(&shaped).enumerate() {
        let slot = slots[index] * 2;
        // The slot's own centre, exactly: `slot / 2` is `size × line_height × 10`, an
        // integer by construction.
        let centre = slot_top + slots[index];
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
            baseline_y: pixels(centre) + (shaped.ascent - shaped.descent) / 2.0,
            stroke_width: shaped.stroke_width,
            extent_width: shaped.advance + 2.0 * shaped.stroke_width as f64,
            break_opportunities: opportunities(line.text)
                .into_iter()
                .map(|at| line.start + at)
                .collect(),
        });
        slot_top += slot;
    }

    let stroke_width = measured
        .iter()
        .map(|line| line.stroke_width)
        .max()
        .unwrap_or(spec.stroke_width);
    Ok(Measurement {
        line_count: measured.len(),
        advance_width: fold(&measured, |line| line.advance_width),
        ascent: fold(&measured, |line| line.ascent),
        descent: fold(&measured, |line| line.descent),
        block_height_tenths: slots.iter().sum(),
        block_top: pixels(block_top),
        block_bottom: pixels(block_top + block),
        extent: Extent {
            width: fold(&measured, |line| line.extent_width),
            height: pixels(block) + 2.0 * stroke_width as f64,
            stroke_width,
        },
        lines: measured,
        segmenter: SEGMENTER,
    })
}

/// One line's metrics, before it knows where it sits.
struct Shaped {
    advance: f64,
    ascent: f64,
    descent: f64,
    size: i64,
    stroke_width: i64,
}

/// A coordinate in twentieths, as pixels.
fn pixels(units: i128) -> f64 {
    units as f64 / UNIT as f64
}

/// The largest of one measurement across every line, or zero where there is no line —
/// which [`partition`] never produces, since it emits at least one.
fn fold(lines: &[MeasuredLine], of: impl Fn(&MeasuredLine) -> f64) -> f64 {
    lines.iter().map(of).fold(0.0, f64::max)
}
