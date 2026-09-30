//! How big the contact sheet is, and whether it is drawn at all (ADR-0095).
//!
//! The one crate-private test point the range mode has (#486): a pixel diff says *that* a
//! sheet is wrong, not *which* rung it picked. It is pure — values in, values out — and knows
//! nothing about documents, painting or reports. The later tickets for labels, keyframes,
//! infill and sub-ranges (#490, #491, #492, #489) extend this one function rather than
//! adding a second.
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

use crate::exact;
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
pub(crate) fn size(
    frame: (i64, i64),
    fps: i64,
    states: &[(i64, i64)],
    labels: LabelWidths,
) -> Result<Fit, Overflow> {
    let tiled: Vec<bool> = states
        .iter()
        .map(|&(start, end)| exact::holds_a_sampled_frame(start, end, fps) == Some(true))
        .collect();
    let tiles = tiled.iter().filter(|&&tiled| tiled).count();
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
        sub_ranges: shares(tiles, admitted, &fits)
            .map(|shares| cut(states, &tiled, &shares))
            .unwrap_or_default(),
    })
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

/// How many tiles each sub-range takes (ADR-0126 §1): the fewest sub-ranges whose shares,
/// as even as they go, each fit, the larger shares first. `None` when not even one tile fits.
///
/// Starting at `⌈tiles / admitted⌉` finds the answer at once whenever a smaller sheet never
/// serves narrower, which the research model bears out. Checking each share rather than
/// assuming it keeps "every sub-range fits" true of the construction, not of that model.
fn shares(tiles: usize, admitted: usize, fits: &dyn Fn(usize) -> bool) -> Option<Vec<usize>> {
    if admitted == 0 {
        return None;
    }
    let count = (tiles.div_ceil(admitted)..=tiles)
        .find(|&count| fits(tiles.div_ceil(count)) && fits(tiles / count))?;
    let (share, larger) = (tiles / count, tiles % count);
    Some(
        (0..count)
            .map(|i| share + usize::from(i < larger))
            .collect(),
    )
}

/// The ranges `shares` cut `states` into (ADR-0126 §2): each after the first opens at the
/// start of its first tiled state, so an untiled state goes with the range before it, and
/// the first and last run to the range's own ends.
fn cut(states: &[(i64, i64)], tiled: &[bool], shares: &[usize]) -> Vec<(i64, i64)> {
    let mut starts = vec![states[0].0];
    let mut seen = 0;
    let mut next = shares.iter().scan(0, |total, share| {
        *total += share;
        Some(*total)
    });
    let mut boundary = next.next();
    for (&(start, _), &tiled) in states.iter().zip(tiled) {
        if !tiled {
            continue;
        }
        if Some(seen) == boundary {
            starts.push(start);
            boundary = next.next();
        }
        seen += 1;
    }
    let end = states[states.len() - 1].1;
    starts
        .iter()
        .zip(starts.iter().skip(1).chain([&end]))
        .map(|(&from, &to)| (from, to))
        .collect()
}

/// The most tiles below `tiles` whose near-square grid `fits`: serves at `refusal` px, with
/// a label core the type floor admits.
///
/// Scanned rather than solved, because the research model does not assume served width
/// falls monotonically with count. The scan is bounded: a grid serving tiles at `refusal`
/// px has at most `1568 / refusal` columns, and at most as many rows as cells of that width
/// stack inside 1568 px.
fn admitted(frame: (i64, i64), tiles: usize, refusal: i64, fits: &dyn Fn(usize) -> bool) -> usize {
    let columns = SERVED_EDGE / refusal.max(1);
    let cell = (refusal * cell_height(frame.1) / frame.0.max(1)).max(1);
    let bound = (columns * (SERVED_EDGE / cell)).max(0) as usize;
    (1..tiles.min(bound + 1))
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
        size(frame, 25, &states(tiles), fixture_labels())
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
            let refused = size(PORTRAIT, fps, &states, fixture_labels()).unwrap_err();
            assert_eq!(refused.sub_ranges, ranges, "{length} s at {fps} fps");
            assert_eq!(
                split(&refused, fps, &states),
                shares,
                "{length} s at {fps} fps"
            );
        }
        // At 24 fps, 20 seconds is 20 tiles, which fit.
        assert!(size(PORTRAIT, 24, &seconds(20), fixture_labels()).is_ok());
    }

    #[test]
    fn an_unpainted_first_state_stays_in_the_first_sub_range() {
        // `[1, 40)` holds no frame at 25 fps: it needs no tile, and the range still starts at 1.
        let mut states = vec![(1, 40)];
        states.extend((0..31).map(|i| (40 + i * 200, 40 + (i + 1) * 200)));
        let refused = size(PORTRAIT, 25, &states, fixture_labels()).unwrap_err();
        assert_eq!(refused.tiles, 31);
        assert_eq!(refused.sub_ranges, vec![(1, 3240), (3240, 6240)]);
    }

    #[test]
    fn a_frame_too_tall_for_one_tile_names_no_sub_range() {
        // One 100 × 20000 tile serves about 7 px wide, under even its own 100 px width.
        let refused = size((100, 20000), 25, &states(2), fixture_labels()).unwrap_err();
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
        let short = size((100, 60), 25, &states(1), fixture_labels()).unwrap_err();
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
            let sheet = size(frame, 25, &states(tiles), labels).unwrap();
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
        let refused = size(LETTERBOX, 25, &states(30), fixture_labels()).unwrap_err();
        assert_eq!((refused.limit, refused.admitted), (Limit::TypeFloor, 22));
        assert_eq!(
            (refused.limit.as_str(), refused.limit.px()),
            ("type-floor", 8)
        );
        let shares = split(&refused, 25, &states(30));
        assert_eq!(shares, [15, 15]);
        for share in shares {
            let sheet = size(LETTERBOX, 25, &states(share), fixture_labels())
                .unwrap_or_else(|e| panic!("a sub-range of {share} fits: {e:?}"));
            assert!(sheet.type_px >= TYPE_FLOOR_PX);
        }
    }

    #[test]
    fn a_label_with_no_room_at_all_is_refused_on_the_type_floor_not_divided_by() {
        // A 10 px project's tile leaves 6 px of room and a 1 px strip.
        let refused = size((10, 10), 25, &states(1), fixture_labels()).unwrap_err();
        assert_eq!((refused.limit, refused.admitted), (Limit::TypeFloor, 0));
        assert!(refused.sub_ranges.is_empty());
    }

    #[test]
    fn rounding_is_half_to_even_like_the_research_model() {
        assert_eq!(rounded(5, 2), 2);
        assert_eq!(rounded(7, 2), 4);
        assert_eq!(rounded(8, 3), 3);
        assert_eq!(rounded(7, 3), 2);
    }
}
