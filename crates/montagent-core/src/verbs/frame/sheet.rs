//! `frame --from --to` — one contact sheet of a span (#488).
//!
//! **A tile is one visual state, sampled at the first frame the grid paints inside it**
//! (ADR-0094). The states are [`cuts::visual_states`], the one selection `validate` uses too
//! (ADR-0118), so the two verbs cut the clock at the same places. A state no frame paints
//! gets no tile: it is `skipped` as `no-grid-frame` and raises the same `N-QUANTIZATION`
//! `validate` raises, through [`quantization::unpainted_states`], so the fact has one
//! identity whichever verb saw it (ADR-0105 §5).
//!
//! **Every tile is `frame --at <its instant>`**, painted at true project pixels by the same
//! [`Painter`], reused across instants the way `render`'s loop reuses it, and composited
//! down onto the sheet (ADR-0095 §5). How big the sheet is, and whether it is drawn at all,
//! is [`sizing`]'s alone.
//!
//! **The answer discloses, on every call** (ADR-0094 §6, ADR-0097 §6, ADR-0105): the rule,
//! the served tile width and its rung, one provenance line per tile, what was skipped and
//! why, the audio-only boundaries the selection dropped, the coverage, and six fixed blind
//! spots. The plain-text form carries all of it; `--json` changes its form, never its
//! presence. The key names no ADR spells are ADR-0125's.
//!
//! Not yet here: tile labels and the READER CHECK (#490), keyframe tiles (#491), infill
//! (#492).

use std::collections::BTreeMap;
use std::path::Path as FilePath;

use serde::Serialize;
use serde_json::{Value, json};

use montagent_render::canvas::{Canvas, Encoding, Region, Rgba, Scale};

use crate::checks::quantization;
use crate::exact::{self, instant_of};
use crate::finding::Finding;
use crate::parse;
use crate::permissive::Loose;
use crate::report::Report;
use crate::verbs::query::Named;
use crate::verbs::query::at;
use crate::verbs::query::cuts::{self, Interval};

use super::sizing::{self, Fit, Overflow};
use super::{Answer, Ask, NotPainted, Painter, Rasterized, TOOL, not_a_frame};

/// The selection rule's name and version. The version moves when the rule does, so an
/// answer can say which rule chose its tiles.
const RULE: &str = "first-painted-frame-of-each-visual-state";
const RULE_VERSION: u32 = 1;
const RULE_SENTENCE: &str = "one tile per visual state (query's cut list over the range, \
audio members dropped and equal neighbours merged), sampled at the first frame the grid \
paints inside it, the least n with \u{230A}n\u{B7}1000/fps\u{230B} in [start, end)";

/// The colour of the sheet around and beneath its tiles: the label strips, until #490 draws
/// labels in them, and any cell the grid has no tile for.
const GUTTER: Rgba = Rgba([0x1A, 0x1A, 0x1A, 0xFF]);

/// ADR-0105 §5's six blind spots, with `between-keyframes` as ADR-0106 rewords it. Fixed
/// text, printed on every answer including a perfect one, so a blind spot's absence from
/// the answer never reads as an all-clear. A token never changes meaning; a new blind spot is a new token.
const BLIND_TO: [(&str, &str); 6] = [
    (
        "inside-run",
        "Each tile shows the first painted frame of its visual state; change inside a state \
         \u{2014} a source clip's own cut, motion within a still-looking element \u{2014} is \
         not on this sheet. `frame --at <instant>` looks at any other instant.",
    ),
    (
        "between-keyframes",
        "Keyframed values are shown only where a tile falls; keyframe change points are \
         untiled unless asked for, and a keyframe tile shows the first painted frame at or \
         after the change, so a wrong easing curve shows only if its endpoints are wrong.",
    ),
    (
        "below-tile-width",
        "Tiles are served at the width stated above; detail finer than that is not visible \
         here. `frame --crop --at <instant>` looks closely at one region at true scale.",
    ),
    (
        "across-sheets",
        "Only tiles on this one sheet can be compared with each other; a relation with a \
         state outside this range is not visible.",
    ),
    ("audio", "Nothing audible is on this sheet."),
    (
        "motion",
        "A sheet is stills; whether motion looks right is `preview`'s question.",
    ),
];

/// A range answer's record: everything the sheet shows, and everything it does not.
#[derive(Debug, Clone, Serialize)]
pub struct Sheet {
    pub from: i64,
    pub to: i64,
    pub rule: Rule,
    /// The picture, or `null` where no state in the range holds a painted frame.
    pub picture: Option<SheetPicture>,
    /// The size every tile was painted at before it was composited down: the project's
    /// true pixels (ADR-0095 §5).
    pub rasterized: Rasterized,
    /// One line per tile, in clock order: the complete record, of which the sheet is a
    /// picture (ADR-0097 §8).
    pub provenance: Vec<Tile>,
    /// How many tiles of each class, zeros asserted (ADR-0098 §6).
    pub classes: Classes,
    /// Every state in the range that drew no tile, and why (ADR-0105 §4).
    pub skipped: Vec<Skipped>,
    /// The boundaries inside the range that only audio crosses, which the selection drops
    /// (ADR-0094 §6, on ADR-0074's measurement).
    pub audio_boundaries_dropped: Dropped,
    pub coverage: Coverage,
    pub blind_to: Vec<BlindSpot>,
    /// Every media file and font file opened, over all tiles.
    pub sources: Vec<String>,
    pub fonts: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Rule {
    pub name: &'static str,
    pub version: u32,
    pub sentence: &'static str,
}

/// The composited sheet, and what it costs to look at.
#[derive(Debug, Clone, Serialize)]
pub struct SheetPicture {
    /// `jpeg` or `png`.
    pub encoding: String,
    /// The image's own pixel size: the size the standard tier serves it at, so nothing
    /// downscales it again.
    pub width: i64,
    pub height: i64,
    pub columns: i64,
    pub rows: i64,
    /// The width every tile is served at (ADR-0095 §6).
    pub served_tile_width: i64,
    /// Its height, at the project's aspect. A tile's top-left is `(column ×
    /// served_tile_width, row × height / rows)`; the label strip is the rest of its cell.
    pub served_tile_height: i64,
    /// `target` or `degraded`: which of ADR-0095's two widths the tiles cleared.
    pub rung: &'static str,
    /// The width that rung guarantees: 180 or 140, or the frame's own width where that is
    /// narrower (ADR-0125).
    pub rung_px: i64,
    pub path: Option<String>,
    pub bytes: usize,
}

/// One tile's provenance line.
#[derive(Debug, Clone, Serialize)]
pub struct Tile {
    /// Counted from 1, in reading order: left to right, then top to bottom.
    pub index: usize,
    /// The painted millisecond: `frame --at` this reproduces the tile.
    pub instant_ms: i64,
    /// The visual state the tile stands for.
    pub run: Span,
    /// Why the tile exists: `boundary`, a visual state's own opening.
    pub why: &'static str,
    /// The unabbreviated class token: `run`.
    pub class: &'static str,
    /// The visual presence set as element ids, in full on every line (ADR-0098 §7).
    pub present: Vec<String>,
    pub not_painted: Vec<NotPainted>,
    pub painted_partially: Vec<NotPainted>,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Span {
    pub start: i64,
    pub end: i64,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Classes {
    pub run: usize,
    pub keyframe: usize,
    pub infill: usize,
}

/// A state that drew no tile.
#[derive(Debug, Clone, Serialize)]
pub struct Skipped {
    pub run: Span,
    /// `no-grid-frame`: no frame the grid paints falls inside the state.
    pub reason: &'static str,
    pub present: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Dropped {
    pub count: usize,
    pub boundaries: Vec<AudioBoundary>,
}

/// One boundary only audio crosses: named, not only counted.
#[derive(Debug, Clone, Serialize)]
pub struct AudioBoundary {
    pub at: i64,
    pub entering: Vec<String>,
    pub leaving: Vec<String>,
}

/// How much of the range the sheet stands for.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Coverage {
    pub range_ms: i64,
    pub states: usize,
    pub tiled: usize,
    pub skipped: usize,
    /// The milliseconds of the range inside a state that has a tile.
    pub depicted_ms: i64,
    /// The milliseconds inside a state that drew none: the skipped states' total.
    pub not_depicted_ms: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct BlindSpot {
    pub token: &'static str,
    pub sentence: &'static str,
}

/// Draw the sheet of `[from, to)`. The flags are already known to compose.
pub(super) fn sheet(path: &FilePath, ask: &Ask, from: i64, to: i64) -> Answer {
    let project = Some(path.display().to_string());
    let document = match parse::read(path) {
        Ok(document) => document,
        Err(finding) => return Answer::refused(Report::unparseable(TOOL, project, *finding)),
    };
    let mut report = Report::new(TOOL, project.clone());
    if let Err(not_a_project) = document.shape() {
        report.push(Finding::not_a_project(
            document.path(),
            &not_a_project,
            "point frame at the project file",
        ));
        return Answer::refused(report);
    }
    let Some(frame) = at::frame_dimensions(&document) else {
        report.push(not_a_frame(&document));
        return Answer::refused(report);
    };
    let Some(fps) = document
        .value()
        .get("fps")
        .and_then(Value::as_i64)
        .filter(|fps| *fps > 0)
    else {
        // A grid is what the sheet samples, so a document without one has no sheet to draw.
        // `validate`'s schema check names which way `fps` is wrong.
        report.push(
            Finding::new("E-NOT-A-PROJECT")
                .at_file(document.path())
                .field("missing", json!("a positive integer `fps`"))
                .repair_value(json!({
                    "value": "give the project a positive integer `fps`, then run validate"
                })),
        );
        return Answer::refused(report);
    };

    let cut_list = cuts::cuts(&document, from, to);
    let states = cuts::visual_states(&cut_list);
    let (runs, skipped): (Vec<(&Interval, i64)>, Vec<&Interval>) = {
        let mut runs = Vec::new();
        let mut skipped = Vec::new();
        for state in &states {
            match first_painted(state, fps) {
                Some(instant) => runs.push((state, instant)),
                None => skipped.push(state),
            }
        }
        (runs, skipped)
    };

    // Sized before anything is painted: a range that does not fit is refused whole, and
    // never thinned, split or reshaped (ADR-0095 §4).
    let fit = match runs.len() {
        0 => None,
        _ => match sizing::size(frame, fps, &states.iter().map(bounds).collect::<Vec<_>>()) {
            Ok(fit) => Some(fit),
            Err(overflow) => {
                return Answer::refused(Report::refused_invocation(
                    TOOL,
                    project,
                    overflowed(from, to, overflow),
                ));
            }
        },
    };

    // One `N-QUANTIZATION` per `no-grid-frame` state, from `validate`'s own function.
    for finding in quantization::unpainted_states(&document, fps, from, to) {
        report.push(finding);
    }

    let mut record = Sheet {
        from,
        to,
        rule: Rule {
            name: RULE,
            version: RULE_VERSION,
            sentence: RULE_SENTENCE,
        },
        picture: None,
        rasterized: Rasterized {
            width: frame.0,
            height: frame.1,
        },
        provenance: Vec::new(),
        classes: Classes {
            run: runs.len(),
            keyframe: 0,
            infill: 0,
        },
        skipped: skipped
            .iter()
            .map(|state| Skipped {
                run: span(state),
                reason: "no-grid-frame",
                present: ids(state),
            })
            .collect(),
        audio_boundaries_dropped: dropped(&cut_list.intervals, &states),
        coverage: Coverage {
            range_ms: to - from,
            states: states.len(),
            tiled: runs.len(),
            skipped: skipped.len(),
            depicted_ms: runs.iter().map(|(state, _)| state.duration_ms).sum(),
            not_depicted_ms: skipped.iter().map(|state| state.duration_ms).sum(),
        },
        blind_to: BLIND_TO
            .iter()
            .map(|&(token, sentence)| BlindSpot { token, sentence })
            .collect(),
        sources: Vec::new(),
        fonts: Vec::new(),
    };

    let Some(fit) = fit else {
        return Answer {
            picture: None,
            view: None,
            image: None,
            report,
            sheet: Some(Box::new(record)),
        };
    };

    let painted = match paint(&document, frame, fit, &runs, &mut report) {
        Some(painted) => painted,
        None => return Answer::refused(report),
    };
    let Painted {
        mut canvas,
        tiles,
        sources,
        fonts,
    } = painted;

    let encoding = if ask.png {
        Encoding::Png
    } else {
        Encoding::Jpeg
    };
    let Some(encoded) = canvas.encode(None, Scale::Full, encoding) else {
        report.fail_internally(format!(
            "the contact sheet could not be encoded as {}",
            encoding.name()
        ));
        return Answer::refused(report);
    };
    let written = match &ask.out {
        Some(out) => match std::fs::write(out, &encoded.bytes) {
            Ok(()) => Some(out.display().to_string()),
            Err(e) => {
                report.could_not_write(out.display(), &e);
                None
            }
        },
        None => None,
    };

    record.picture = Some(SheetPicture {
        encoding: encoded.encoding.name().to_string(),
        width: encoded.width,
        height: encoded.height,
        columns: fit.columns,
        rows: fit.rows,
        served_tile_width: fit.tile_width,
        served_tile_height: fit.tile_height,
        rung: fit.rung.as_str(),
        rung_px: fit.rung.px(frame.0),
        path: written,
        bytes: encoded.bytes.len(),
    });
    record.provenance = tiles;
    record.sources = sources;
    record.fonts = fonts;

    Answer {
        picture: None,
        view: None,
        image: Some(encoded),
        report,
        sheet: Some(Box::new(record)),
    }
}

/// The first painted millisecond inside `state`, or `None` where the grid paints none.
///
/// The same predicate [`quantization::unpainted_states`] skips on, so a state is either
/// tiled here or a finding there, never both and never neither.
fn first_painted(state: &Interval, fps: i64) -> Option<i64> {
    if exact::holds_a_sampled_frame(state.start, state.end, fps) != Some(true) {
        return None;
    }
    exact::frame_at_or_after(state.start, fps).map(|frame| instant_of(frame.frame, fps))
}

/// The sheet with every tile painted on it, and the record of what was painted.
struct Painted {
    canvas: Canvas,
    tiles: Vec<Tile>,
    sources: Vec<String>,
    fonts: Vec<String>,
}

/// Paint each tile at true pixels and composite it into its cell. `None` where the run
/// stopped, with the report saying why.
fn paint(
    document: &Loose,
    frame: (i64, i64),
    fit: Fit,
    runs: &[(&Interval, i64)],
    report: &mut Report,
) -> Option<Painted> {
    let (Some(mut canvas), Some(mut tile), Some(mut sheet)) = (
        Canvas::new(frame.0, frame.1),
        Canvas::new(fit.tile_width, fit.tile_height),
        Canvas::new(fit.sheet_width, fit.sheet_height),
    ) else {
        report.fail_internally(format!(
            "no raster surface could be made at {}x{} for a {}x{} sheet",
            frame.0, frame.1, fit.sheet_width, fit.sheet_height
        ));
        return None;
    };
    sheet.background(GUTTER);

    let mut painter = Painter::new(document, runs[0].1, frame);
    // One finding per `(element, code)` over the whole sheet, as `render` keeps one over a
    // whole span: the same element declining for the same reason on every tile is one fact.
    let mut declined: BTreeMap<(String, String), Finding> = BTreeMap::new();
    let mut tiles = Vec::with_capacity(runs.len());
    for (i, (state, instant)) in runs.iter().enumerate() {
        // The caption `frame --at` paints from, so a tile and that frame cannot disagree
        // about which frame of a clip is on screen.
        let view = match at::answer(document, *instant) {
            Ok(view) => view,
            Err(reason) => {
                report.fail_internally(reason);
                return None;
            }
        };
        painter.begin(*instant);
        painter.paint(&mut canvas, &view);
        if let Some(missing) = painter.tool_missing.take() {
            missing.fail(report);
            return None;
        }
        if let Some(reason) = painter.internal.take() {
            report.fail_internally(reason);
            return None;
        }
        for finding in painter.declined.drain(..) {
            let key = (
                finding.location.element.clone().unwrap_or_default(),
                finding.code.clone(),
            );
            declined.entry(key).or_insert(finding);
        }

        // Resampled at the origin of a canvas of its own, then placed 1:1. Resampling
        // straight into its cell would make a tile's pixels depend on where on the sheet it
        // lands — Skia's sampling differs by a few pixels between offsets — and a tile is to
        // be a function of `frame --at <its instant>` alone.
        let (column, row) = (i as i64 % fit.columns, i as i64 / fit.columns);
        let whole = Region {
            x: 0,
            y: 0,
            width: fit.tile_width,
            height: fit.tile_height,
        };
        // Over opaque black, as `frame --at` encodes, so a translucent `background` shows
        // the same picture here and not the previous tile through it.
        tile.background(Rgba::BLACK);
        tile.composite(&canvas.snapshot(), whole);
        sheet.composite(
            &tile.snapshot(),
            Region {
                x: column * fit.tile_width,
                y: row * fit.cell_height,
                ..whole
            },
        );
        tiles.push(Tile {
            index: i + 1,
            instant_ms: *instant,
            run: span(state),
            why: "boundary",
            class: "run",
            present: ids(state),
            not_painted: painter.not_painted.clone(),
            painted_partially: painter.painted_partially.clone(),
        });
    }
    for finding in declined.into_values() {
        report.push(finding);
    }

    Some(Painted {
        canvas: sheet,
        tiles,
        sources: std::mem::take(&mut painter.sources),
        fonts: std::mem::take(&mut painter.fonts),
    })
}

/// ADR-0105 §1's refusal, with the fields its template reads and the sub-ranges an agent
/// loops over (ADR-0126).
fn overflowed(from: i64, to: i64, overflow: Overflow) -> Finding {
    let sub_ranges: Vec<Value> = overflow
        .sub_ranges
        .iter()
        .map(|&(from, to)| json!({"from": from, "to": to}))
        .collect();
    Finding::new("E-SHEET-OVERFLOW")
        .field("from", json!(from))
        .field("to", json!(to))
        .field("states", json!(overflow.tiles))
        .field("fits", json!(overflow.admitted))
        .field("limit", json!(overflow.limit.as_str()))
        .field("limit_px", json!(overflow.limit.px()))
        .field("sub_ranges", json!(sub_ranges))
}

/// Every boundary inside the range where the cut list changes and the visual states do
/// not: the ones only audio crosses, with what enters and leaves there.
fn dropped(intervals: &[Interval], states: &[Interval]) -> Dropped {
    let visual: Vec<i64> = states.iter().skip(1).map(|state| state.start).collect();
    let boundaries: Vec<AudioBoundary> = intervals
        .windows(2)
        .filter(|pair| !visual.contains(&pair[1].start))
        .map(|pair| {
            let (before, after) = (&pair[0], &pair[1]);
            let missing = |from: &Interval, of: &Interval| -> Vec<String> {
                of.present
                    .iter()
                    .filter(|named| !from.present.contains(named))
                    .map(id)
                    .collect()
            };
            AudioBoundary {
                at: after.start,
                entering: missing(before, after),
                leaving: missing(after, before),
            }
        })
        .collect();
    Dropped {
        count: boundaries.len(),
        boundaries,
    }
}

/// A state's `[start, end)`, as the sizing module reads it.
fn bounds(state: &Interval) -> (i64, i64) {
    (state.start, state.end)
}

fn span(state: &Interval) -> Span {
    Span {
        start: state.start,
        end: state.end,
    }
}

fn ids(state: &Interval) -> Vec<String> {
    state.present.iter().map(id).collect()
}

/// An element's id, or `(no id)` — the name `N-QUANTIZATION`'s `present` gives the same
/// element, so a skipped state and its finding name one set.
fn id(named: &Named) -> String {
    named.id.clone().unwrap_or_else(|| "(no id)".to_string())
}
