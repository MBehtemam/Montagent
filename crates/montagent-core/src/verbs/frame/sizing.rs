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
}

/// Which limit refused the sheet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Limit {
    /// ADR-0095's 140 px served tile width.
    TileWidth,
}

impl Limit {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Limit::TileWidth => "tile-width",
        }
    }

    /// The limit's value in served pixels, as the refusal states it.
    pub(crate) fn px(self) -> i64 {
        match self {
            Limit::TileWidth => REFUSAL_PX,
        }
    }
}

/// A range the sheet cannot show legibly (ADR-0095 §3): never thinned, split or reshaped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Overflow {
    /// The tiles asked for.
    pub(crate) tiles: usize,
    /// The most tiles a sheet of this frame holds without passing [`Overflow::limit`].
    pub(crate) admitted: usize,
    pub(crate) limit: Limit,
}

/// Size a sheet of `tiles` tiles of a `frame`-sized picture, or refuse it.
///
/// `tiles` is at least one; a range with nothing to tile draws no sheet, and the verb does
/// not ask.
pub(crate) fn size(frame: (i64, i64), tiles: usize) -> Result<Fit, Overflow> {
    let target = Rung::Target.px(frame.0);
    let refusal = Rung::Degraded.px(frame.0);
    let fit = grid(frame, tiles);
    if fit.tile_width >= target {
        return Ok(Fit {
            rung: Rung::Target,
            ..fit
        });
    }
    if fit.tile_width >= refusal {
        return Ok(Fit {
            rung: Rung::Degraded,
            ..fit
        });
    }
    Err(Overflow {
        tiles,
        admitted: admitted(frame, tiles, refusal),
        limit: Limit::TileWidth,
    })
}

/// The most tiles below `tiles` whose near-square grid still serves at `refusal` px.
///
/// Scanned rather than solved, because the research model does not assume served width
/// falls monotonically with count. The scan is bounded: a grid serving tiles at `refusal`
/// px has at most `1568 / refusal` columns, and at most as many rows as cells of that width
/// stack inside 1568 px.
fn admitted(frame: (i64, i64), tiles: usize, refusal: i64) -> usize {
    let columns = SERVED_EDGE / refusal.max(1);
    let cell = (refusal * cell_height(frame.1) / frame.0.max(1)).max(1);
    let bound = (columns * (SERVED_EDGE / cell)).max(0) as usize;
    (1..tiles.min(bound + 1))
        .rev()
        .find(|&n| grid(frame, n).tile_width >= refusal)
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

    fn fit(frame: (i64, i64), tiles: usize) -> Fit {
        size(frame, tiles).unwrap_or_else(|overflow| panic!("{tiles} tiles: {overflow:?}"))
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
                size(PORTRAIT, tiles).ok().map(|fit| fit.rung),
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
                let Ok(sheet) = size(frame, tiles) else {
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
        let refused = size(PORTRAIT, 31).expect_err("31 portrait tiles pass 140 px");
        assert_eq!(
            refused,
            Overflow {
                tiles: 31,
                admitted: 30,
                limit: Limit::TileWidth,
            }
        );
        assert_eq!(size(PORTRAIT, 200).unwrap_err().admitted, 30);
        assert_eq!(size(LANDSCAPE, 99).unwrap_err().admitted, 98);
        assert_eq!(size(SQUARE, 57).unwrap_err().admitted, 56);
    }

    #[test]
    fn a_project_narrower_than_a_threshold_is_held_to_its_own_width() {
        // 200 px frames serve at true pixels up to 24 tiles: nothing is lost to a downscale.
        let sheet = fit((200, 200), 24);
        assert_eq!((sheet.tile_width, sheet.rung), (200, Rung::Target));
        // A 100 px project is not refused for being small.
        let small = fit((100, 60), 10);
        assert_eq!((small.tile_width, small.rung), (100, Rung::Target));
    }

    #[test]
    fn rounding_is_half_to_even_like_the_research_model() {
        assert_eq!(rounded(5, 2), 2);
        assert_eq!(rounded(7, 2), 4);
        assert_eq!(rounded(8, 3), 3);
        assert_eq!(rounded(7, 3), 2);
    }
}
