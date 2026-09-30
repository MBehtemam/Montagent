//! How big the contact sheet is, and whether it is drawn at all (ADR-0095).
//!
//! The one crate-private test point the range mode has (#486): a pixel diff says *that* a
//! sheet is wrong, not *which* rung it picked. It is pure — values in, values out — and knows
//! nothing about documents, painting or reports. [`size`] fixes the sheet the
//! document-derived tiles need, and [`infill`] fills what it leaves (#492); the tickets for
//! labels, keyframes and sub-ranges (#490, #491, #489) extended `size` rather than adding a
//! third.
//!
//! ## The model
//!
//! **The currency is served tile width**: the width a tile is looked at after the API has
//! downscaled the whole sheet to its tier. The standard tier serves an image whose long edge
//! is at most 1568 px, in 28 px patches, and spends at most 1568 visual tokens on it. A
//! sheet is `columns × rows` cells, each the project's frame plus an 11% label strip beneath
//! it, and the served size of that composite is what [`served`] computes. This is
//! `docs/research/contact-sheet-budget/check_tile_budget.py`'s model, ported line for line,
//! and the table tests below are that script's numbers.
//!
//! **The grid is the near-square one**: of every column count, the one whose served tile
//! has the most area, the first on a tie. Count is derived, never asked for (ADR-0095 §2).
//!
//! **Two rungs, then a refusal.** The target is 180 px. A sheet that cannot serve every
//! tile at 180 px degrades once, to the best grid that still serves at 140 px or more, and
//! one that cannot do that either is refused. The degraded rung is not drawn at exactly
//! 140 px: it is whatever the near-square grid serves, which lies in `[140, 180)`. A sheet
//! drawn narrower than its budget allows would spend legibility for nothing (ADR-0125).
//!
//! **A tile is never asked to be wider than the project.** A 100 px project serves its
//! tiles at 100 px, true pixels, and loses nothing to a downscale, so each threshold is
//! capped at the frame's own width (ADR-0125). Without the cap, a small project could never
//! be shown on a sheet at all.
//!
//! **The labels are fitted here too** (ADR-0098 §4–5), because whether a label fits is a
//! fact about the grid: one type size for the whole sheet, the largest at which the longest
//! label spans its tile less an inset and whose em fits the strip. If that is under the 8 px
//! type floor with the identifying field, the field is elided on every tile; if it is under
//! the floor without it, the sheet is refused on `type-floor`. The widths come in as font
//! units of the chrome face, which is monospaced and ligature-free, so this is arithmetic and
//! nothing is shaped to find it out (ADR-0122 §3).
//!
//! **An overflow names the fewest sub-ranges that each fit**, their tiles shared as evenly as
//! they go and each cut at a visual state's start, so that together they cover the range
//! exactly and each, requested alone, draws the tiles counted for it here (ADR-0126).
//!
//! **Infill takes only what the rung leaves** (ADR-0106 D5): a sheet of one tile at the
//! target rung has seventeen slots, because eighteen tiles still serve at 180 px. It keeps
//! the rung and the identifying field, and its ceiling is honoured in every span or at the
//! smallest one that fits (ADR-0130).

use crate::exact::{self, instant_of};
use crate::fonts::chrome;

/// ADR-0095's target: every range gets tiles at least this wide, served, when they fit.
const TARGET_PX: i64 = 180;

/// ADR-0095's tile-width refusal: the measured cliff past which fine detail stops showing.
/// Reached by exactly one degrade step; a sheet that cannot serve this is refused.
const REFUSAL_PX: i64 = 140;

/// The standard tier's long-edge cap on a served image, in pixels.
const SERVED_EDGE: i64 = 1568;

/// The standard tier's cap on visual tokens per image.
const SERVED_TOKENS: i64 = 1568;

/// One visual token is a 28 × 28 px patch.
const PATCH: i64 = 28;

/// The label strip beneath each tile, as a percentage of the tile's height (ADR-0098 §1).
const STRIP_PERCENT: i64 = 11;

/// ADR-0098 §4's served type floor: no label is drawn smaller; content gives way first.
pub(crate) const TYPE_FLOOR_PX: i64 = 8;

/// The room kept clear at each end of a label strip, in pixels.
pub(crate) const LABEL_INSET: i64 = 2;

/// The two widths the longest label on the sheet has, in font units of the chrome face: with
/// its identifying field, and without it (the numeric core). Only the tiles in the
/// sheet-wide elision fit count (ADR-0106 §7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LabelWidths {
    pub(crate) core: u32,
    pub(crate) whole: u32,
}

/// Which of ADR-0095's two widths the sheet cleared.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Rung {
    /// Every tile serves at [`TARGET_PX`] or more.
    Target,
    /// Not at the target, so degraded once: every tile serves at [`REFUSAL_PX`] or more.
    Degraded,
}

impl Rung {
    /// The width this rung guarantees every tile, for a `frame`-wide project: 180 or 140,
    /// or the project's own width where that is narrower.
    pub(crate) fn px(self, frame_width: i64) -> i64 {
        match self {
            Rung::Target => TARGET_PX.min(frame_width),
            Rung::Degraded => REFUSAL_PX.min(frame_width),
        }
    }

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Rung::Target => "target",
            Rung::Degraded => "degraded",
        }
    }
}

/// A sheet that fits: its grid, and the pixel geometry the verb composites onto.
///
/// Every width and height here is **served** pixels: the sheet is drawn at exactly the size
/// the tier would serve it at, so nothing downscales it again.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Fit {
    pub(crate) rung: Rung,
    pub(crate) columns: i64,
    pub(crate) rows: i64,
    /// The width of one tile as served: the served sheet's width over its columns, floored.
    pub(crate) tile_width: i64,
    /// The tile's height, at the project's aspect.
    pub(crate) tile_height: i64,
    /// One cell: the tile and the label strip beneath it.
    pub(crate) cell_height: i64,
    /// The sheet the verb draws: `columns × tile_width` by `rows × cell_height`.
    pub(crate) sheet_width: i64,
    pub(crate) sheet_height: i64,
    /// The one type size every label is drawn at, in served pixels: never under
    /// [`TYPE_FLOOR_PX`].
    pub(crate) type_px: i64,
    /// Whether the labels carry their identifying field.
    pub(crate) ids: Ids,
}

impl Fit {
    /// The label strip's height: the cell beneath the tile.
    pub(crate) fn strip(&self) -> i64 {
        self.cell_height - self.tile_height
    }
}

/// Whether a sheet's labels carry their identifying field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Ids {
    Carried,
    /// ADR-0098 §5's sheet-wide elision: the longest label did not fit at the floor with it.
    Elided,
}

impl Ids {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Ids::Carried => "carried",
            Ids::Elided => "elided",
        }
    }
}

/// Which limit refused the sheet, as ADR-0126 spells it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Limit {
    /// ADR-0095's 140 px served tile width.
    TileWidth,
    /// ADR-0098's 8 px served type: the tiles are wide enough, and a label's numeric core
    /// still cannot be drawn legibly beneath them.
    TypeFloor,
}

impl Limit {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Limit::TileWidth => "tile-width",
            Limit::TypeFloor => "type-floor",
        }
    }

    /// The limit's value in served pixels, as the refusal states it.
    pub(crate) fn px(self) -> i64 {
        match self {
            Limit::TileWidth => REFUSAL_PX,
            Limit::TypeFloor => TYPE_FLOOR_PX,
        }
    }
}

/// A range the sheet cannot show legibly (ADR-0095 §3): never thinned, split or reshaped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Overflow {
    /// The tiles asked for: the visual states the grid paints.
    pub(crate) tiles: usize,
    /// The most tiles a sheet of this frame holds without passing [`Overflow::limit`].
    pub(crate) admitted: usize,
    pub(crate) limit: Limit,
    /// The fewest consecutive, half-open ranges that each draw a sheet, covering the range
    /// exactly (ADR-0126). Empty only when not even one tile of this frame fits.
    pub(crate) sub_ranges: Vec<(i64, i64)>,
}

/// Size a sheet of the tiles `states` need at `fps`, for a `frame`-sized picture whose
/// longest label is `labels` wide, or refuse it naming the sub-ranges that would fit.
///
/// `states` are the range's visual states, consecutive and in order, and a state needs a
/// tile when the grid paints a frame inside it. At least one does; a range with nothing to
/// tile draws no sheet, and the verb does not ask.
///
/// `keyframes` counts the keyframe tiles each state adds beside its run tile, one count per
/// state, or is empty where none were asked for (ADR-0106). They are counted like any tile
/// and never dropped to make the sheet fit; a sub-range keeps every one of its states'.
pub(crate) fn size(
    frame: (i64, i64),
    fps: i64,
    states: &[(i64, i64)],
    keyframes: &[usize],
    labels: LabelWidths,
) -> Result<Fit, Overflow> {
    // A state no frame paints holds no keyframe tile either: a keyframe tile is a painted
    // frame inside its state.
    let weights: Vec<usize> = states
        .iter()
        .enumerate()
        .map(|(i, &(start, end))| {
            match exact::holds_a_sampled_frame(start, end, fps) == Some(true) {
                true => 1 + keyframes.get(i).copied().unwrap_or(0),
                false => 0,
            }
        })
        .collect();
    let tiles: usize = weights.iter().sum();
    let target = Rung::Target.px(frame.0);
    let refusal = Rung::Degraded.px(frame.0);
    let fit = grid(frame, tiles);
    let rung = if fit.tile_width >= target {
        Some(Rung::Target)
    } else if fit.tile_width >= refusal {
        Some(Rung::Degraded)
    } else {
        None
    };
    // Content gives way before type does (ADR-0098 §4): the identifying field first, on
    // every tile at once, and the numeric core never.
    let whole = type_px(&fit, labels.whole);
    let core = type_px(&fit, labels.core);
    let limit = match rung {
        Some(rung) if whole >= TYPE_FLOOR_PX => {
            return Ok(Fit {
                rung,
                type_px: whole,
                ids: Ids::Carried,
                ..fit
            });
        }
        Some(rung) if core >= TYPE_FLOOR_PX => {
            return Ok(Fit {
                rung,
                type_px: core,
                ids: Ids::Elided,
                ..fit
            });
        }
        Some(_) => Limit::TypeFloor,
        None => Limit::TileWidth,
    };
    // A sub-range's labels are never longer than these: the same instants and offsets,
    // and an index counted from 1 again. So a share that fits `labels.core` fits.
    let fits = |n: usize| {
        let fit = grid(frame, n);
        fit.tile_width >= refusal && type_px(&fit, labels.core) >= TYPE_FLOOR_PX
    };
    let admitted = admitted(frame, tiles, refusal, &fits);
    Err(Overflow {
        tiles,
        admitted,
        limit,
        sub_ranges: split(states, &weights, admitted, &fits).unwrap_or_default(),
    })
}

/// What `--infill-ceiling` adds to a sheet [`size`] fitted (ADR-0106 D4–6, ADR-0130).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Infill {
    /// The sheet with its infill tiles: the document-derived tiles' own where none is added,
    /// and otherwise the grid for all of them, on the same rung and with the same ids.
    pub(crate) fit: Fit,
    /// The infill tiles' painted instants, in clock order.
    pub(crate) tiles: Vec<i64>,
    /// The ceiling every span on the sheet honours: the one asked for, or the smallest that
    /// fits. `None` where the request needed tiles and the sheet draws none.
    pub(crate) achieved: Option<i64>,
    /// The instants the requested ceiling would have added and the sheet does not draw, in
    /// clock order.
    pub(crate) evicted: Vec<i64>,
}

/// Fill the slots `fit` leaves at its rung with infill tiles, so that no span between two
/// consecutive tiles of any class — the document-derived ones at `tiles`, in clock order,
/// and the infill itself — is longer, in painted time, than `ceiling` (ADR-0106 D4).
///
/// **The document-derived tiles fix the rung, and infill only fills what it leaves** (D5): a
/// count of tiles is admitted while every sheet up to it keeps `fit`'s rung, and its labels,
/// as `labels` measures them for a given set of infill instants, keep `fit`'s identifying
/// field at the type floor. So infill never degrades, elides or refuses a sheet.
///
/// **The ceiling is honoured uniformly or not at all** (D6): at `ceiling` if its tiles fit,
/// and otherwise at the smallest ceiling whose tiles do, in every span at once. The last
/// tile's span runs to the first frame painted at or after `to`, where the range's frames
/// end. Inside a span, each infill tile is the latest painted frame no more than the ceiling
/// after the tile before it (ADR-0130).
pub(crate) fn infill(
    frame: (i64, i64),
    fps: i64,
    fit: Fit,
    tiles: &[i64],
    to: i64,
    ceiling: i64,
    labels: &dyn Fn(&[i64]) -> LabelWidths,
) -> Infill {
    let end = exact::frame_at_or_after(to, fps).map_or(to, |frame| instant_of(frame.frame, fps));
    let requested = placed(fps, tiles, end, ceiling, usize::MAX).unwrap_or_default();
    let unchanged = |achieved| Infill {
        fit,
        tiles: Vec::new(),
        achieved,
        evicted: requested.clone(),
    };
    if requested.is_empty() {
        return unchanged(Some(ceiling));
    }
    // The document's own longest span: a ceiling that long needs no tile, and always fits.
    let longest = tiles
        .iter()
        .zip(tiles.iter().skip(1).chain([&end]))
        .map(|(from, to)| to - from)
        .max()
        .unwrap_or(0);
    let rung = fit.rung.px(frame.0);
    let geometry = |n: usize| grid(frame, n).tile_width >= rung;
    let bound = bound(frame, rung);
    let slots = (1..)
        .take_while(|&extra| tiles.len() + extra <= bound && geometry(tiles.len() + extra))
        .last()
        .unwrap_or(0);
    // The most tiles first, each at the smallest ceiling that needs no more; a count whose
    // labels do not fit gives way to the next smaller.
    for slots in (1..=slots).rev() {
        let achieved = smallest(ceiling, longest, |c| {
            placed(fps, tiles, end, c, slots).is_some()
        });
        let infill = placed(fps, tiles, end, achieved, slots).unwrap_or_default();
        if infill.is_empty() {
            // Only the document's own spans fit this many: nothing uniform is left to add.
            break;
        }
        let mut sheet = Fit {
            rung: fit.rung,
            ids: fit.ids,
            ..grid(frame, tiles.len() + infill.len())
        };
        let widths = labels(&infill);
        sheet.type_px = type_px(
            &sheet,
            match fit.ids {
                Ids::Carried => widths.whole,
                Ids::Elided => widths.core,
            },
        );
        if sheet.type_px < TYPE_FLOOR_PX {
            continue;
        }
        return Infill {
            fit: sheet,
            evicted: requested
                .iter()
                .copied()
                .filter(|instant| infill.binary_search(instant).is_err())
                .collect(),
            tiles: infill,
            achieved: Some(achieved),
        };
    }
    unchanged(None)
}

/// The infill tiles `ceiling` places among `tiles`, whose last span ends at `end`, or `None`
/// where that is more than `most`, or where no frame is counted past a tile in 64 bits.
fn placed(fps: i64, tiles: &[i64], end: i64, ceiling: i64, most: usize) -> Option<Vec<i64>> {
    let mut infill = Vec::new();
    let closes = tiles.iter().skip(1).chain([&end]);
    for (&from, &to) in tiles.iter().zip(closes) {
        let mut last = from;
        while to - last > ceiling {
            // The latest painted frame at or before `last + ceiling`. The verb refuses a
            // ceiling under one frame period, so there is always one after `last`; the next
            // painted frame stands in for it here all the same, so the loop always moves on.
            let within = exact::frame_before(last.saturating_add(ceiling).saturating_add(1), fps);
            let after = exact::frame_at_or_after(last.saturating_add(1), fps);
            let next = [within, after]
                .into_iter()
                .flatten()
                .map(|frame| instant_of(frame.frame, fps))
                .max()
                .filter(|&next| next > last)?;
            if infill.len() == most {
                return None;
            }
            infill.push(next);
            last = next;
        }
    }
    Some(infill)
}

/// The least ceiling in `[from, to]` that `fits`, which holds of `to`. A larger ceiling
/// never needs more tiles, so the search halves.
fn smallest(from: i64, to: i64, fits: impl Fn(i64) -> bool) -> i64 {
    if fits(from) {
        return from;
    }
    let (mut lo, mut hi) = (from, to.max(from));
    while hi - lo > 1 {
        let mid = lo + (hi - lo) / 2;
        if fits(mid) { hi = mid } else { lo = mid }
    }
    hi
}

/// A bound on how many tiles a sheet can serve at `width` px or more: at most `1568 /
/// width` columns, and at most as many rows as cells of that width stack inside 1568 px.
fn bound(frame: (i64, i64), width: i64) -> usize {
    let columns = SERVED_EDGE / width.max(1);
    let cell = (width * cell_height(frame.1) / frame.0.max(1)).max(1);
    (columns * (SERVED_EDGE / cell)).max(0) as usize
}

/// The largest whole type size at which a label `units` wide fits `fit`'s cells: its
/// advance and one more character's inside the tile's width less [`LABEL_INSET`] at each
/// end, and its em inside the strip beneath the tile. The extra character keeps a space
/// before the next tile's label: flush, `… +0` and `11 2000ms …` read as `+011`.
fn type_px(fit: &Fit, units: u32) -> i64 {
    let strip = fit.strip();
    let room = (fit.tile_width - 2 * LABEL_INSET).max(0);
    let spaced = i64::from(units) + i64::from(chrome::ADVANCE);
    strip.min(room * i64::from(chrome::UNITS_PER_EM) / spaced)
}

/// The sub-ranges of an overflow (ADR-0126): the fewest whose tiles, shared as evenly as
/// the states' own counts allow, each fit. `None` when no split fits: not even one tile, or
/// one state's run and keyframe tiles are more than a sheet holds.
///
/// Starting at `⌈tiles / admitted⌉` finds the answer at once whenever a smaller sheet never
/// serves narrower, which the research model bears out. Checking each sub-range's own count
/// rather than assuming it keeps "every sub-range fits" true of the construction, not of
/// that model, and true where a state weighs more than one tile.
fn split(
    states: &[(i64, i64)],
    weights: &[usize],
    admitted: usize,
    fits: &dyn Fn(usize) -> bool,
) -> Option<Vec<(i64, i64)>> {
    let tiles: usize = weights.iter().sum();
    if admitted == 0 {
        return None;
    }
    (tiles.div_ceil(admitted)..=tiles).find_map(|count| {
        let (ranges, counts) = cut(states, weights, &shares(tiles, count));
        counts.iter().all(|&n| fits(n)).then_some(ranges)
    })
}

/// `tiles` shared as evenly as they go between `count` sub-ranges, the larger shares first
/// (ADR-0126 §1).
fn shares(tiles: usize, count: usize) -> Vec<usize> {
    let (share, larger) = (tiles / count, tiles % count);
    (0..count)
        .map(|i| share + usize::from(i < larger))
        .collect()
}

/// The ranges `shares` cut `states` into, and the tiles each one holds (ADR-0126 §2, as
/// ADR-0129 weighs it): a sub-range after the first opens at the start of the first tiled
/// state reached once the sub-ranges before it hold their shares, so an untiled state goes
/// with the range before it, and the first and last run to the range's own ends. With one
/// tile a state the counts are the shares exactly; a state carrying keyframe tiles is never
/// cut, so the counts are as near the shares as its weight lets them be.
fn cut(
    states: &[(i64, i64)],
    weights: &[usize],
    shares: &[usize],
) -> (Vec<(i64, i64)>, Vec<usize>) {
    let mut starts = vec![states[0].0];
    let mut counts = vec![0];
    let mut seen = 0;
    let mut next = shares.iter().scan(0, |total, share| {
        *total += share;
        Some(*total)
    });
    let mut boundary = next.next();
    for (&(start, _), &weight) in states.iter().zip(weights) {
        if weight == 0 {
            continue;
        }
        if boundary.is_some_and(|boundary| seen >= boundary) {
            starts.push(start);
            counts.push(0);
            // A heavy state may carry the count past more than one share.
            while boundary.is_some_and(|boundary| seen >= boundary) {
                boundary = next.next();
            }
        }
        seen += weight;
        *counts.last_mut().expect("one range at least") += weight;
    }
    let end = states[states.len() - 1].1;
    let ranges = starts
        .iter()
        .zip(starts.iter().skip(1).chain([&end]))
        .map(|(&from, &to)| (from, to))
        .collect();
    (ranges, counts)
}

/// The most tiles below `tiles` whose near-square grid `fits`: serves at `refusal` px, with
/// a label core the type floor admits.
///
/// Scanned rather than solved, because the research model does not assume served width
/// falls monotonically with count. The scan is [`bound`]ed.
fn admitted(frame: (i64, i64), tiles: usize, refusal: i64, fits: &dyn Fn(usize) -> bool) -> usize {
    (1..tiles.min(bound(frame, refusal) + 1))
        .rev()
        .find(|&n| fits(n))
        .unwrap_or(0)
}

/// The near-square grid for `tiles` tiles: the column count whose served tile has the most
/// area, the first on a tie. Its rung is not decided here.
fn grid(frame: (i64, i64), tiles: usize) -> Fit {
    let (width, height) = (frame.0.max(1), frame.1.max(1));
    let cell = cell_height(height);
    let tiles = tiles.max(1) as i64;
    let mut best: Option<(f64, i64, i64, (i64, i64))> = None;
    for columns in 1..=tiles {
        let rows = (tiles + columns - 1) / columns;
        let sheet = served(columns * width, rows * cell);
        // The research model's own arithmetic, in its own order, so a tie falls the same
        // way: served tile width by served tile height, the frame's share of the cell.
        let tile_w = sheet.0 as f64 / columns as f64;
        let tile_h = (sheet.1 as f64 / rows as f64) * (height as f64 / cell as f64);
        let area = tile_w * tile_h;
        if best.is_none_or(|(most, ..)| area > most) {
            best = Some((area, columns, rows, sheet));
        }
    }
    let (_, columns, rows, sheet) = best.expect("at least one column is tried");
    let tile_width = sheet.0 / columns;
    let cell_height = sheet.1 / rows;
    let tile_height = rounded(tile_width * height, width).min(cell_height);
    Fit {
        rung: Rung::Target,
        columns,
        rows,
        tile_width,
        tile_height,
        cell_height,
        sheet_width: columns * tile_width,
        sheet_height: rows * cell_height,
        type_px: 0,
        ids: Ids::Elided,
    }
}

/// A cell's height at true pixels: the frame and its label strip.
fn cell_height(height: i64) -> i64 {
    height + height * STRIP_PERCENT / 100
}

/// The size an image of `width × height` is served at on the standard tier: unchanged if it
/// already fits, and otherwise the widest aspect-preserving size that does.
fn served(width: i64, height: i64) -> (i64, i64) {
    if fits(width, height) {
        return (width, height);
    }
    if height > width {
        let (h, w) = served(height, width);
        return (w, h);
    }
    let (mut lo, mut hi) = (1, width);
    while lo + 1 < hi {
        let mid = (lo + hi) / 2;
        if fits(mid, rounded(mid * height, width).max(1)) {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    (lo, rounded(lo * height, width).max(1))
}

fn fits(width: i64, height: i64) -> bool {
    let patches = |px: i64| (px + PATCH - 1) / PATCH;
    patches(width) * PATCH <= SERVED_EDGE
        && patches(height) * PATCH <= SERVED_EDGE
        && patches(width) * patches(height) <= SERVED_TOKENS
}

/// `numerator / denominator` rounded half to even, which is what the research model's
/// Python `round` does. Both are positive here.
fn rounded(numerator: i64, denominator: i64) -> i64 {
    let (quotient, remainder) = (numerator / denominator, numerator % denominator);
    match (2 * remainder).cmp(&denominator) {
        std::cmp::Ordering::Less => quotient,
        std::cmp::Ordering::Greater => quotient + 1,
        std::cmp::Ordering::Equal => quotient + quotient % 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PORTRAIT: (i64, i64) = (1080, 1920);
    const LANDSCAPE: (i64, i64) = (1920, 1080);
    const SQUARE: (i64, i64) = (1080, 1080);

    /// A label's width in the chrome face: every character advances [`chrome::ADVANCE`].
    fn units(label: &str) -> u32 {
        label.chars().count() as u32 * chrome::ADVANCE
    }

    /// The main fixture's longest labels (`docs/research/tile-label/OUTPUT.txt`): tile 9's
    /// with its id, and tile 18's numeric core.
    fn fixture_labels() -> LabelWidths {
        LabelWidths {
            core: units("18 64040ms +24"),
            whole: units("9 42800ms +37 +word-08-bridge"),
        }
    }

    /// `tiles` consecutive 200 ms states at 25 fps, every one painted.
    fn states(tiles: usize) -> Vec<(i64, i64)> {
        (0..tiles as i64)
            .map(|i| (i * 200, (i + 1) * 200))
            .collect()
    }

    fn sized(frame: (i64, i64), tiles: usize) -> Result<Fit, Overflow> {
        size(frame, 25, &states(tiles), &[], fixture_labels())
    }

    fn fit(frame: (i64, i64), tiles: usize) -> Fit {
        sized(frame, tiles).unwrap_or_else(|overflow| panic!("{tiles} tiles: {overflow:?}"))
    }

    #[test]
    fn eighteen_portrait_tiles_serve_at_184_px_on_the_target_rung() {
        // ADR-0095 §1's working point, and #486's main-fixture fact.
        let sheet = fit(PORTRAIT, 18);
        assert_eq!(sheet.rung, Rung::Target);
        assert_eq!((sheet.columns, sheet.rows), (6, 3));
        assert_eq!(sheet.tile_width, 184);
        // Served 1107 × 1092; the drawn sheet is whole tiles inside it.
        assert_eq!((sheet.sheet_width, sheet.sheet_height), (1104, 1092));
        assert_eq!((sheet.tile_height, sheet.cell_height), (327, 364));
    }

    #[test]
    fn the_rung_turns_at_the_180_and_140_px_edges() {
        // (tiles, rung or refusal, served tile width) — check_tile_budget.py's numbers.
        let table = [
            (1, Some(Rung::Target), 784),
            (12, Some(Rung::Target), 227),
            (18, Some(Rung::Target), 184),
            (19, Some(Rung::Degraded), 173),
            (24, Some(Rung::Degraded), 160),
            (30, Some(Rung::Degraded), 141),
            (31, None, 138),
        ];
        for (tiles, rung, width) in table {
            assert_eq!(grid(PORTRAIT, tiles).tile_width, width, "{tiles} tiles");
            assert_eq!(
                sized(PORTRAIT, tiles).ok().map(|fit| fit.rung),
                rung,
                "{tiles}"
            );
        }
    }

    #[test]
    fn the_grid_is_the_near_square_one_for_the_frames_aspect() {
        // (frame, tiles, columns, rows, served tile width)
        let table = [
            (PORTRAIT, 2, 2, 1, 553),
            (PORTRAIT, 4, 2, 2, 392),
            (PORTRAIT, 30, 10, 3, 141),
            (LANDSCAPE, 2, 1, 2, 980),
            (LANDSCAPE, 18, 3, 6, 326),
            (LANDSCAPE, 19, 5, 4, 313),
            (SQUARE, 12, 4, 3, 301),
            (SQUARE, 18, 6, 3, 244),
        ];
        for (frame, tiles, columns, rows, width) in table {
            let sheet = fit(frame, tiles);
            assert_eq!(
                (sheet.columns, sheet.rows, sheet.tile_width),
                (columns, rows, width),
                "{frame:?} × {tiles}"
            );
        }
    }

    #[test]
    fn every_drawn_sheet_is_within_what_the_tier_serves_unchanged() {
        for frame in [PORTRAIT, LANDSCAPE, SQUARE, (200, 200), (640, 360)] {
            for tiles in 1..=40 {
                let Ok(sheet) = sized(frame, tiles) else {
                    continue;
                };
                assert!(
                    fits(sheet.sheet_width, sheet.sheet_height),
                    "{frame:?} × {tiles}: {sheet:?}"
                );
                assert!(sheet.columns * sheet.rows >= tiles as i64);
                assert!(sheet.tile_height <= sheet.cell_height);
            }
        }
    }

    #[test]
    fn keyframe_tiles_count_like_any_tile_and_a_sub_range_keeps_its_states_keyframes() {
        // #407's measurement: 18 states hold 12 keyframe tiles, and the 13th refuses.
        let mut keyframes = vec![1; 12];
        keyframes.resize(18, 0);
        let sheet = size(PORTRAIT, 25, &states(18), &keyframes, fixture_labels()).unwrap();
        assert_eq!((sheet.rung, sheet.tile_width), (Rung::Degraded, 141));

        keyframes[12] = 1;
        let refused = size(PORTRAIT, 25, &states(18), &keyframes, fixture_labels()).unwrap_err();
        assert_eq!((refused.tiles, refused.admitted), (31, 30));
        // Sixteen and fifteen, as without keyframes: the first eight states weigh two each.
        assert_eq!(refused.sub_ranges, [(0, 1600), (1600, 3600)]);

        // A state is never cut, so a heavy one moves the cut past the even share.
        let mut heavy = vec![0; 18];
        heavy[7] = 5;
        let refused = size(PORTRAIT, 25, &states(18), &heavy, fixture_labels());
        assert!(refused.is_ok(), "23 tiles fit: {refused:?}");
        let mut heavy = vec![0; 31];
        heavy[14] = 4;
        let refused = size(PORTRAIT, 25, &states(31), &heavy, fixture_labels()).unwrap_err();
        assert_eq!(refused.tiles, 35);
        assert_eq!(refused.sub_ranges, [(0, 3000), (3000, 6200)]);

        // One state with more tiles than a sheet holds names no sub-range.
        let refused = size(PORTRAIT, 25, &states(1), &[40], fixture_labels()).unwrap_err();
        assert_eq!(refused.sub_ranges, []);
    }

    #[test]
    fn an_overflow_names_the_tile_width_limit_and_the_most_tiles_admitted() {
        let refused = sized(PORTRAIT, 31).expect_err("31 portrait tiles pass 140 px");
        assert_eq!(
            (refused.tiles, refused.admitted, refused.limit),
            (31, 30, Limit::TileWidth)
        );
        assert_eq!(sized(PORTRAIT, 200).unwrap_err().admitted, 30);
        assert_eq!(sized(LANDSCAPE, 99).unwrap_err().admitted, 98);
        assert_eq!(sized(SQUARE, 57).unwrap_err().admitted, 56);
    }

    /// The painted states in each of `overflow`'s sub-ranges, asserting on the way that the
    /// sub-ranges cover `states` exactly and each one cuts at a state's start.
    #[track_caller]
    fn split(overflow: &Overflow, fps: i64, states: &[(i64, i64)]) -> Vec<usize> {
        let ranges = &overflow.sub_ranges;
        assert_eq!(ranges.first().map(|r| r.0), states.first().map(|s| s.0));
        assert_eq!(ranges.last().map(|r| r.1), states.last().map(|s| s.1));
        for pair in ranges.windows(2) {
            assert_eq!(pair[0].1, pair[1].0, "consecutive: {ranges:?}");
        }
        ranges
            .iter()
            .map(|&(from, to)| {
                assert!(
                    states.iter().any(|s| s.0 == from),
                    "{from} is not a state's start"
                );
                states
                    .iter()
                    .filter(|&&(start, end)| start >= from && end <= to)
                    .filter(|&&(start, end)| {
                        exact::holds_a_sampled_frame(start, end, fps) == Some(true)
                    })
                    .count()
            })
            .collect()
    }

    #[test]
    fn an_overflow_splits_into_the_fewest_sub_ranges_with_tiles_shared_evenly() {
        // (frame, tiles, tiles per sub-range): the fewest sheets, the larger shares first.
        let table: [((i64, i64), usize, &[usize]); 6] = [
            (PORTRAIT, 31, &[16, 15]),
            (PORTRAIT, 60, &[30, 30]),
            (PORTRAIT, 61, &[21, 20, 20]),
            (PORTRAIT, 200, &[29, 29, 29, 29, 28, 28, 28]),
            (LANDSCAPE, 99, &[50, 49]),
            (SQUARE, 57, &[29, 28]),
        ];
        for (frame, tiles, shares) in table {
            let refused = sized(frame, tiles).unwrap_err();
            assert_eq!(
                split(&refused, 25, &states(tiles)),
                shares,
                "{frame:?} × {tiles}"
            );
            for &share in shares {
                assert!(sized(frame, share).is_ok(), "a sub-range of {share} fits");
            }
        }
        // Even, not greedy: 30 and 1 would draw a degraded sheet and a one-tile sheet.
        assert_eq!(
            sized(PORTRAIT, 31).unwrap_err().sub_ranges,
            vec![(0, 3200), (3200, 6200)]
        );
    }

    /// Per second of `seconds`: a long state, a 30 ms state and a 10 ms one. The 10 ms state
    /// holds no frame at 24, 25 or 30 fps; the 30 ms one, `[960, 990)`, holds one at 25 and
    /// 30 fps (960, 966.7) and none at 24 (958.3, then 1000).
    fn seconds(seconds: i64) -> Vec<(i64, i64)> {
        (0..seconds)
            .flat_map(|i| {
                let t = i * 1000;
                [(t, t + 960), (t + 960, t + 990), (t + 990, t + 1000)]
            })
            .collect()
    }

    #[test]
    fn the_sub_ranges_follow_the_states_the_grid_paints_at_each_fps() {
        // (seconds, fps, sub-ranges, tiles in each)
        type Row = (i64, i64, &'static [(i64, i64)], &'static [usize]);
        let table: [Row; 6] = [
            (
                45,
                25,
                &[(0, 15000), (15000, 30000), (30000, 45000)],
                &[30, 30, 30],
            ),
            (
                45,
                30,
                &[(0, 15000), (15000, 30000), (30000, 45000)],
                &[30, 30, 30],
            ),
            (45, 24, &[(0, 23000), (23000, 45000)], &[23, 22]),
            // The second sub-range opens on a short state: it is the 22nd tile's.
            (
                31,
                25,
                &[(0, 10960), (10960, 21000), (21000, 31000)],
                &[21, 21, 20],
            ),
            (
                31,
                30,
                &[(0, 10960), (10960, 21000), (21000, 31000)],
                &[21, 21, 20],
            ),
            (31, 24, &[(0, 16000), (16000, 31000)], &[16, 15]),
        ];
        for (length, fps, ranges, shares) in table {
            let states = seconds(length);
            let refused = size(PORTRAIT, fps, &states, &[], fixture_labels()).unwrap_err();
            assert_eq!(refused.sub_ranges, ranges, "{length} s at {fps} fps");
            assert_eq!(
                split(&refused, fps, &states),
                shares,
                "{length} s at {fps} fps"
            );
        }
        // At 24 fps, 20 seconds is 20 tiles, which fit.
        assert!(size(PORTRAIT, 24, &seconds(20), &[], fixture_labels()).is_ok());
    }

    #[test]
    fn an_unpainted_first_state_stays_in_the_first_sub_range() {
        // `[1, 40)` holds no frame at 25 fps: it needs no tile, and the range still starts at 1.
        let mut states = vec![(1, 40)];
        states.extend((0..31).map(|i| (40 + i * 200, 40 + (i + 1) * 200)));
        let refused = size(PORTRAIT, 25, &states, &[], fixture_labels()).unwrap_err();
        assert_eq!(refused.tiles, 31);
        assert_eq!(refused.sub_ranges, vec![(1, 3240), (3240, 6240)]);
    }

    #[test]
    fn a_frame_too_tall_for_one_tile_names_no_sub_range() {
        // One 100 × 20000 tile serves about 7 px wide, under even its own 100 px width.
        let refused = size((100, 20000), 25, &states(2), &[], fixture_labels()).unwrap_err();
        assert_eq!((refused.admitted, refused.sub_ranges.len()), (0, 0));
    }

    #[test]
    fn a_project_narrower_than_a_threshold_is_held_to_its_own_width() {
        // 200 px frames serve at true pixels up to 24 tiles: nothing is lost to a downscale.
        let sheet = fit((200, 200), 24);
        assert_eq!((sheet.tile_width, sheet.rung), (200, Rung::Target));
        // A 100 px project is not refused for being small.
        let small = fit((100, 100), 10);
        assert_eq!((small.tile_width, small.rung), (100, Rung::Target));
        // But a tile too short for an 8 px strip is refused on the type floor, whatever the
        // count: a 60 px tile's strip is 6 px (ADR-0128 §4).
        let short = size((100, 60), 25, &states(1), &[], fixture_labels()).unwrap_err();
        assert_eq!((short.limit, short.admitted), (Limit::TypeFloor, 0));
        assert!(short.sub_ranges.is_empty());
    }

    #[test]
    fn the_fixtures_ids_print_at_the_target_rung_and_elide_at_the_refusal_width() {
        // ADR-0098 §5 on the main fixture: 29 characters (and a space) fit at 184 px, and
        // not at 141 px, where every tile keeps its 14-character core instead.
        let target = fit(PORTRAIT, 18);
        assert_eq!((target.ids, target.type_px), (Ids::Carried, 10));
        let degraded = fit(PORTRAIT, 19);
        assert_eq!(
            (degraded.rung, degraded.ids, degraded.type_px),
            (Rung::Degraded, Ids::Carried, 9)
        );
        let floor = fit(PORTRAIT, 30);
        assert_eq!((floor.tile_width, floor.ids), (141, Ids::Elided));
        assert_eq!(
            floor.type_px, 15,
            "the core is fitted afresh once the id is gone"
        );
    }

    #[test]
    fn elision_is_decided_by_the_floor_on_the_tiles_width_less_its_insets() {
        // 29 characters and a space at 600/1000 em each need 144 px at 8 px type: a tile of
        // 148 px (144 px of room) carries them, and anything narrower elides — sheet-wide.
        for tiles in 1..=30 {
            let sheet = fit(PORTRAIT, tiles);
            assert_eq!(
                sheet.ids == Ids::Carried,
                sheet.tile_width >= 148,
                "{tiles} tiles: {sheet:?}"
            );
            assert!(sheet.type_px >= TYPE_FLOOR_PX, "{tiles} tiles: {sheet:?}");
            assert!(
                sheet.type_px <= sheet.strip(),
                "the em fits the strip: {sheet:?}"
            );
        }
    }

    #[test]
    fn one_type_size_serves_the_sheet_and_the_strip_caps_it() {
        // (labels, frame, tiles, type px, ids): width-bound, then strip-bound.
        let short = LabelWidths {
            core: units("1 0ms +0"),
            whole: units("1 0ms +0 +a"),
        };
        let table = [
            (fixture_labels(), PORTRAIT, 18, 10, Ids::Carried),
            // 11 characters and a space in 180 px of room is 25 px; the 37 px strip does
            // not bind.
            (short, PORTRAIT, 18, 25, Ids::Carried),
            // One landscape tile served 1568 px wide has an 86 px strip; 12 characters
            // alone would fit at 217.
            (short, LANDSCAPE, 1, 86, Ids::Carried),
        ];
        for (labels, frame, tiles, type_px, ids) in table {
            let sheet = size(frame, 25, &states(tiles), &[], labels).unwrap();
            assert_eq!(
                (sheet.type_px, sheet.ids),
                (type_px, ids),
                "{frame:?} × {tiles}"
            );
        }
    }

    #[test]
    fn a_core_that_cannot_be_drawn_at_the_floor_is_refused_on_the_type_floor() {
        // A 1920 × 200 frame keeps its tiles wide and its strips thin. At 30 tiles each is
        // 588 px wide, far past the tile-width refusal, over a 7 px strip: no label fits
        // there with or without its id, so eliding cannot help and the type floor refuses.
        const LETTERBOX: (i64, i64) = (1920, 200);
        assert_eq!(
            fit(LETTERBOX, 20).type_px,
            8,
            "20 tiles still draw at the floor"
        );
        let refused = size(LETTERBOX, 25, &states(30), &[], fixture_labels()).unwrap_err();
        assert_eq!((refused.limit, refused.admitted), (Limit::TypeFloor, 22));
        assert_eq!(
            (refused.limit.as_str(), refused.limit.px()),
            ("type-floor", 8)
        );
        let shares = split(&refused, 25, &states(30));
        assert_eq!(shares, [15, 15]);
        for share in shares {
            let sheet = size(LETTERBOX, 25, &states(share), &[], fixture_labels())
                .unwrap_or_else(|e| panic!("a sub-range of {share} fits: {e:?}"));
            assert!(sheet.type_px >= TYPE_FLOOR_PX);
        }
    }

    #[test]
    fn a_label_with_no_room_at_all_is_refused_on_the_type_floor_not_divided_by() {
        // A 10 px project's tile leaves 6 px of room and a 1 px strip.
        let refused = size((10, 10), 25, &states(1), &[], fixture_labels()).unwrap_err();
        assert_eq!((refused.limit, refused.admitted), (Limit::TypeFloor, 0));
        assert!(refused.sub_ranges.is_empty());
    }

    /// Labels short enough never to bind: the tests below are about counts and instants.
    fn short(_: &[i64]) -> LabelWidths {
        LabelWidths {
            core: units("1 0ms +0"),
            whole: units("1 0ms +0 +a"),
        }
    }

    /// The infill of a portrait sheet whose document-derived tiles sit at `tiles`, at 25
    /// fps, over a range ending at `to`.
    fn infilled(tiles: &[i64], to: i64, ceiling: i64) -> Infill {
        infilled_at(25, tiles, to, ceiling)
    }

    fn infilled_at(fps: i64, tiles: &[i64], to: i64, ceiling: i64) -> Infill {
        let fit = fit(PORTRAIT, tiles.len());
        infill(PORTRAIT, fps, fit, tiles, to, ceiling, &short)
    }

    /// Every span the sheet leaves, in painted time: between consecutive tiles of any class,
    /// and from the last to the first frame painted at or after `to`.
    fn spans(fps: i64, tiles: &[i64], infill: &[i64], to: i64) -> Vec<i64> {
        let mut all: Vec<i64> = tiles.iter().chain(infill).copied().collect();
        all.sort();
        all.push(instant_of(
            exact::frame_at_or_after(to, fps).unwrap().frame,
            fps,
        ));
        all.windows(2).map(|pair| pair[1] - pair[0]).collect()
    }

    #[test]
    fn a_ceiling_that_fits_is_honoured_at_the_latest_frame_within_it_of_the_tile_before() {
        // One tile at 0 over five seconds, a second apart: four slots of the seventeen the
        // target rung leaves, and a sheet of five on it.
        let sheet = infilled(&[0], 5000, 1000);
        assert_eq!(sheet.tiles, [1000, 2000, 3000, 4000]);
        assert_eq!(sheet.achieved, Some(1000));
        assert!(sheet.evicted.is_empty());
        assert_eq!(
            (sheet.fit.rung, sheet.fit.columns, sheet.fit.rows),
            (Rung::Target, 3, 2)
        );

        // Off the grid, the latest painted frame no further than the ceiling: 1030 → 1000.
        let sheet = infilled(&[0], 3000, 1030);
        assert_eq!(sheet.tiles, [1000, 2000]);
        assert_eq!(sheet.achieved, Some(1030));
    }

    #[test]
    fn a_ceiling_that_only_partly_fits_is_honoured_uniformly_at_the_smallest_that_does() {
        // Twenty seconds at 1000 ms needs 19 tiles; the target rung has 17 slots. The
        // smallest ceiling 17 tiles honour on a 40 ms grid is 1120, everywhere at once.
        let sheet = infilled(&[0], 20000, 1000);
        assert_eq!(sheet.achieved, Some(1120));
        assert_eq!(sheet.tiles, (1..=17).map(|i| i * 1120).collect::<Vec<_>>());
        let honoured = spans(25, &[0], &sheet.tiles, 20000);
        assert_eq!(
            honoured.iter().max(),
            Some(&1120),
            "achieved is the longest span"
        );
        // What 1000 ms would have added, and the sheet does not draw.
        assert_eq!(
            sheet.evicted,
            (1..=19).map(|i| i * 1000).collect::<Vec<_>>()
        );
        assert_eq!((sheet.fit.rung, sheet.fit.tile_width), (Rung::Target, 184));

        // 1119 ms steps 1080 on this grid, and leaves 1640 at the end: not honoured.
        let near = spans(25, &[0], &infilled(&[0], 20000, 1119).tiles, 20000);
        assert!(near.iter().all(|&span| span <= 1120), "{near:?}");
    }

    #[test]
    fn with_no_slot_left_nothing_is_added_and_the_achieved_ceiling_is_none() {
        // Eighteen tiles fill the target rung: a nineteenth serves at 173 px.
        let states: Vec<i64> = (0..18).map(|i| i * 200).collect();
        let sheet = infilled(&states, 3600, 100);
        assert_eq!(sheet.achieved, None);
        assert!(sheet.tiles.is_empty());
        assert_eq!(
            sheet.fit,
            fit(PORTRAIT, 18),
            "the sheet is the document's own"
        );
        // Each 200 ms span would have had two: 80 ms in, and 80 ms after that.
        assert_eq!(
            sheet.evicted,
            states
                .iter()
                .flat_map(|start| [start + 80, start + 160])
                .collect::<Vec<_>>()
        );

        // Two slots, and sixteen equal spans: no ceiling short of the spans' own fits
        // uniformly, so none is achieved, though the slots were there.
        let states: Vec<i64> = (0..16).map(|i| i * 200).collect();
        let sheet = infilled(&states, 3200, 100);
        assert_eq!((sheet.achieved, sheet.tiles.len()), (None, 0));
        assert_eq!(sheet.evicted.len(), 32);
    }

    #[test]
    fn a_ceiling_longer_than_the_range_adds_nothing_and_is_achieved() {
        let sheet = infilled(&[0, 1000], 5000, 60_000);
        assert_eq!((sheet.tiles.len(), sheet.achieved), (0, Some(60_000)));
        assert!(sheet.evicted.is_empty());
        assert_eq!(sheet.fit, fit(PORTRAIT, 2));
    }

    #[test]
    fn the_span_is_across_tiles_of_any_class() {
        // A keyframe tile at 2600 is a tile like any other: the spans are 2600 and 2400.
        let sheet = infilled(&[0, 2600], 5000, 2000);
        assert_eq!(sheet.tiles, [2000, 4600]);
    }

    #[test]
    fn a_ceiling_of_one_frame_period_tiles_every_frame_whole_or_not() {
        // 40 ms at 25 fps is every frame.
        let sheet = infilled(&[0], 400, 40);
        assert_eq!(sheet.tiles, (1..10).map(|i| i * 40).collect::<Vec<_>>());
        assert_eq!(sheet.achieved, Some(40));
        // 30 fps paints 33.3 ms apart, at 0, 33, 66, 100, …: a whole-millisecond ceiling of
        // 34 is the smallest the grid can honour, and it is every frame too.
        let sheet = infilled_at(30, &[0], 500, 34);
        assert_eq!(
            sheet.tiles,
            (1..15).map(|n| instant_of(n, 30)).collect::<Vec<_>>()
        );
        assert!(
            spans(30, &[0], &sheet.tiles, 500)
                .iter()
                .all(|&span| span <= 34)
        );
    }

    #[test]
    fn infill_keeps_the_rung_and_the_identifying_field_the_document_tiles_fixed() {
        // A degraded sheet stays degraded: 19 tiles leave the 11 slots up to 30, and a
        // six-second last state takes all of them.
        let states: Vec<i64> = (0..19).map(|i| i * 200).collect();
        let sheet = infilled(&states, 9600, 520);
        assert_eq!(
            sheet.tiles,
            (1..=11).map(|i| 3600 + i * 520).collect::<Vec<_>>()
        );
        assert_eq!(sheet.achieved, Some(520));
        // 500 ms is 480 on this grid, twelve tiles: honoured at 520 instead.
        assert_eq!(infilled(&states, 9600, 500).achieved, Some(520));
        assert_eq!(
            (sheet.fit.rung, sheet.fit.tile_width),
            (Rung::Degraded, 141)
        );

        // A label that would not fit takes the slot away rather than the id: here any
        // infill tile's label is too long for the tile.
        let long = |infill: &[i64]| match infill.len() {
            0 => short(infill),
            _ => LabelWidths {
                core: units(&"9".repeat(200)),
                whole: units(&"9".repeat(200)),
            },
        };
        let docs = fit(PORTRAIT, 1);
        let sheet = infill(PORTRAIT, 25, docs, &[0], 5000, 1000, &long);
        assert_eq!((sheet.tiles.len(), sheet.achieved), (0, None));
        assert_eq!(sheet.fit, docs);
    }

    #[test]
    fn rounding_is_half_to_even_like_the_research_model() {
        assert_eq!(rounded(5, 2), 2);
        assert_eq!(rounded(7, 2), 4);
        assert_eq!(rounded(8, 3), 3);
        assert_eq!(rounded(7, 3), 2);
    }
}
