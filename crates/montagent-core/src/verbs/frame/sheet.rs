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
//! **Every tile is labelled in the strip beneath it**, never on its pixels (ADR-0098 §1):
//! [`label`] says what the label reads and draws it, and [`sizing`] decides how big, and
//! whether the identifying field survives, before anything is painted. The exact string
//! drawn is each provenance line's `label`, and the READER CHECK quotes tile 1's
//! (ADR-0114, as ADR-0116 narrows it).
//!
//! **The answer discloses, on every call** (ADR-0094 §6, ADR-0097 §6, ADR-0105): the rule,
//! the served tile width and its rung, one provenance line per tile, what was skipped and
//! why, the audio-only boundaries the selection dropped, the coverage, and six fixed blind
//! spots. The plain-text form carries all of it; `--json` changes its form, never its
//! presence. The key names no ADR spells are ADR-0125's.
//!
//! **Keyframe tiles are added only with `--keyframes`** (ADR-0106), at the first painted
//! frame of each keyframe change point inside a state, and [`keyframes`] places every
//! change point; the census of them is on every answer, flag or not. A keyframe tile is
//! marked, carries no identifying field, and is counted like any tile: never dropped to
//! make the sheet fit (ADR-0129).
//!
//! **Infill tiles are added only with `--infill-ceiling <MS>`** (ADR-0106 D4–6), into the
//! slots the document-derived tiles leave at their rung, so that no span between two
//! consecutive tiles of any class is longer than the ceiling. [`sizing::infill`] places them
//! and says which ceiling the sheet achieved; what the requested one would have added and
//! the sheet does not draw is `skipped` as `infill-evicted`, by run, and is never a finding
//! (ADR-0105 §4). An infill tile is marked and carries no identifying field (ADR-0130).

use std::collections::BTreeMap;
use std::path::Path as FilePath;

use serde::Serialize;
use serde_json::{Value, json};

use montagent_render::canvas::{Canvas, Encoding, Region, Rgba, Scale};

use crate::checks::quantization;
use crate::exact::{self, instant_of};
use crate::finding::Finding;
use crate::fonts::chrome;
use crate::parse;
use crate::permissive::Loose;
use crate::report::Report;
use crate::verbs::query::Named;
use crate::verbs::query::at;
use crate::verbs::query::cuts::{self, Interval};

use super::keyframes::{self, KeyframeCensus, Placed, Point};
use super::label::{self, Changes, Label};
use super::sizing::{self, Fit, Ids, LabelWidths, Overflow};
use super::{Answer, Ask, NotPainted, Painter, Rasterized, TOOL, not_a_frame};

/// The selection rule's name and version. The version moves when the rule does, so an
/// answer can say which rule chose its tiles.
const RULE: &str = "first-painted-frame-of-each-visual-state";
const RULE_VERSION: u32 = 1;
const RULE_SENTENCE: &str = "one tile per visual state (query's cut list over the range, \
audio members dropped and equal neighbours merged), sampled at the first frame the grid \
paints inside it, the least n with \u{230A}n\u{B7}1000/fps\u{230B} in [start, end)";

/// The colour of the sheet around and beneath its tiles: the label strips, and any cell the
/// grid has no tile for.
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

/// ADR-0114 §3's READER CHECK, around tile 1's exact label. Its four parts — where the
/// label is, the exact string, which channel is the record, and the next call — and its
/// closing sentence may be reworded, never dropped. It names no reader (ADR-0114 §4), and a
/// reader's pass is not evidence that the sheet was read (ADR-0116), so it claims nothing
/// of the kind.
fn reader_sentence(label: &str) -> String {
    format!(
        "Each tile's label is the line in the strip beneath it, outside the video frame; \
         text inside a tile is the video's own. Tile 1's label reads exactly `{label}`. The \
         provenance list below is the complete record of this range, and this sheet is a \
         picture of it. If the strip beneath tile 1 does not read exactly that, this sheet \
         is below what you can see, and `frame --at <instant>` shows any listed instant at \
         full scale. Reading the labels is necessary for seeing the pictures, not sufficient."
    )
}

/// A range answer's record: everything the sheet shows, and everything it does not.
#[derive(Debug, Clone, Serialize)]
pub struct Sheet {
    pub from: i64,
    pub to: i64,
    pub rule: Rule,
    /// The picture, or `null` where no state in the range holds a painted frame.
    pub picture: Option<SheetPicture>,
    /// The READER CHECK, quoting tile 1's label: `null` only with no picture, where there
    /// is no tile 1 to quote.
    pub reader_check: Option<ReaderCheck>,
    /// The size every tile was painted at before it was composited down: the project's
    /// true pixels (ADR-0095 §5).
    pub rasterized: Rasterized,
    /// One line per tile, in clock order: the complete record, of which the sheet is a
    /// picture (ADR-0097 §8).
    pub provenance: Vec<Tile>,
    /// How many tiles of each class, zeros asserted (ADR-0098 §6).
    pub classes: Classes,
    /// The keyframe change points interior to a state on a visible element, split by
    /// whether a tile of this sheet shows them: on every answer, flag or not (ADR-0106 D8).
    pub keyframes: KeyframeCensus,
    /// The ceiling asked for and the one the sheet honours: present only when
    /// `--infill-ceiling` was passed (ADR-0106 D6).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub infill: Option<InfillCeiling>,
    /// Every state in the range that drew no tile, and every one whose infill tiles the
    /// sheet had no room for, and why (ADR-0105 §4).
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

/// ADR-0114 §3's handshake, as the JSON carries it.
#[derive(Debug, Clone, Serialize)]
pub struct ReaderCheck {
    /// Always 1: the tile whose label is quoted.
    pub tile: usize,
    /// Byte for byte the string drawn beneath tile 1, and `provenance[0].label`.
    pub label: String,
    pub sentence: String,
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
    /// The one type size every label is drawn at, in served pixels.
    pub label_px: i64,
    /// ADR-0098 §4's floor, which `label_px` is never under.
    pub label_floor_px: i64,
    /// `carried`, or `elided` on every tile: the longest label did not fit at the floor
    /// with its identifying field (ADR-0098 §5).
    pub ids: &'static str,
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
    /// Why the tile exists: `boundary`, a visual state's own opening; `keyframe`, a change
    /// point inside it; or `infill`, a span longer than the infill ceiling.
    pub why: &'static str,
    /// The unabbreviated class token: `run`, `keyframe` or `infill`.
    pub class: &'static str,
    /// The change points this tile shows, as `element.property@t`, in clock order. A run or
    /// infill tile's are the points sampled at its frame; it keeps its class (ADR-0106 D11).
    pub keyframes: Vec<String>,
    /// The exact string drawn in the strip beneath the tile, after the sheet-wide elision.
    pub label: String,
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

/// A state that drew no tile, or none of the infill tiles asked for inside it.
#[derive(Debug, Clone, Serialize)]
pub struct Skipped {
    pub run: Span,
    /// `no-grid-frame`: no frame the grid paints falls inside the state. `infill-evicted`:
    /// the requested ceiling would have added tiles inside it, and the sheet had no room.
    pub reason: &'static str,
    pub present: Vec<String>,
    /// How many tiles the requested ceiling would have added inside the state: an
    /// `infill-evicted` entry's alone.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evicted: Option<usize>,
}

/// ADR-0106 D6's disclosure of the infill ceiling.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct InfillCeiling {
    pub requested_ms: i64,
    /// The ceiling every span on the sheet honours: `requested_ms` where it fits, the
    /// smallest that does where it only partly fits, and `null` where the request needed
    /// tiles and the sheet draws none.
    pub achieved_ms: Option<i64>,
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

/// Draw the sheet of `[from, to)`, with infill tiles under `ceiling` where one is asked for.
/// The flags are already known to compose.
pub(super) fn sheet(
    path: &FilePath,
    ask: &Ask,
    from: i64,
    to: i64,
    ceiling: Option<i64>,
) -> Answer {
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
    // A ceiling under one frame period asks for a spacing the grid cannot paint: a malformed
    // request, not a tight one, so it is refused rather than clamped (ADR-0106 D14). Only
    // here, once the document says what the period is.
    if let Some(ceiling) = ceiling.filter(|&ceiling| i128::from(ceiling) * i128::from(fps) < 1000) {
        return Answer::refused(Report::rejected(
            TOOL,
            project,
            below_one_frame(
                ask.infill_ceiling.as_deref().unwrap_or_default(),
                ceiling,
                fps,
            ),
        ));
    }

    let cut_list = cuts::cuts(&document, from, to);
    let states = cuts::visual_states(&cut_list);
    // Each state's run tile, `None` where the grid paints no frame inside it.
    let painted: Vec<Option<i64>> = states
        .iter()
        .map(|state| first_painted(state, fps))
        .collect();
    let (runs, skipped): (Vec<(&Interval, i64)>, Vec<&Interval>) = {
        let mut runs = Vec::new();
        let mut skipped = Vec::new();
        for (state, painted) in states.iter().zip(&painted) {
            match painted {
                Some(instant) => runs.push((state, *instant)),
                None => skipped.push(state),
            }
        }
        (runs, skipped)
    };

    // Every change point placed, whether or not keyframe tiles were asked for: the census
    // is on every answer (ADR-0106 D8).
    let mut placed = keyframes::place(&document, &states, &painted, fps, ask.keyframes);
    let changes = Changes::of(&document, to);
    let bounds: Vec<(i64, i64)> = states.iter().map(bounds).collect();

    // Sized before anything is painted: a range that does not fit is refused whole, and
    // never thinned, split or reshaped (ADR-0095 §4), and a keyframe tile is never dropped
    // to make it fit (ADR-0106 D9).
    let planned = plan(&states, &painted, &placed, &changes, &[]);
    let fit = match planned.len() {
        0 => None,
        _ => match sizing::size(
            frame,
            fps,
            &bounds,
            &placed.per_state(states.len()),
            widths(&planned),
        ) {
            Ok(fit) => Some(fit),
            Err(overflow) => {
                let mut refusal = overflowed(from, to, runs.len(), &overflow);
                if ask.keyframes {
                    // Would dropping the flag be the cheaper remedy? The run tiles alone,
                    // sized as a sheet without the flag would size them.
                    let without = sizing::size(
                        frame,
                        fps,
                        &bounds,
                        &[],
                        widths(&plan(&states, &painted, &Placed::default(), &changes, &[])),
                    );
                    refusal = refusal
                        .field("keyframe_tiles", json!(placed.tiles.len()))
                        .field("fits_without_keyframes", json!(without.is_ok()))
                        .field(
                            "keyframe_tiles_admitted",
                            json!(overflow.admitted.saturating_sub(runs.len())),
                        );
                }
                return Answer::refused(Report::refused_invocation(TOOL, project, refusal));
            }
        },
    };

    // Infill fills what the document-derived tiles leave at their rung, and changes nothing
    // else about them but their index (ADR-0106 D5).
    let (fit, planned, infill, evicted) = match (fit, ceiling) {
        (Some(fit), Some(ceiling)) => {
            let instants: Vec<i64> = planned.iter().map(|tile| tile.instant).collect();
            let labels = |infill: &[i64]| {
                widths(&plan(
                    &states,
                    &painted,
                    &placed,
                    &changes,
                    &infill_tiles(&states, infill),
                ))
            };
            let infill = sizing::infill(frame, fps, fit, &instants, to, ceiling, &labels);
            placed.onto_infill(&infill.tiles);
            let planned = plan(
                &states,
                &painted,
                &placed,
                &changes,
                &infill_tiles(&states, &infill.tiles),
            );
            let disclosed = InfillCeiling {
                requested_ms: ceiling,
                achieved_ms: infill.achieved,
            };
            (Some(infill.fit), planned, Some(disclosed), infill.evicted)
        }
        // No tile, so no span to bound: the ceiling holds of the sheet as it is.
        (None, Some(ceiling)) => (
            None,
            planned,
            Some(InfillCeiling {
                requested_ms: ceiling,
                achieved_ms: Some(ceiling),
            }),
            Vec::new(),
        ),
        (fit, None) => (fit, planned, None, Vec::new()),
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
        reader_check: None,
        rasterized: Rasterized {
            width: frame.0,
            height: frame.1,
        },
        provenance: Vec::new(),
        classes: Classes {
            run: runs.len(),
            keyframe: placed.tiles.len(),
            infill: planned
                .iter()
                .filter(|tile| tile.class == Class::Infill)
                .count(),
        },
        keyframes: placed.census_over(&states),
        infill,
        skipped: skipped_states(&states, &skipped, &evicted),
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

    let drawn: Vec<String> = planned
        .iter()
        .map(|tile| match fit.ids {
            Ids::Carried => tile.label.whole.clone(),
            Ids::Elided => tile.label.core.clone(),
        })
        .collect();
    let painted = match paint(&document, frame, fit, &planned, &drawn, &mut report) {
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
        label_px: fit.type_px,
        label_floor_px: sizing::TYPE_FLOOR_PX,
        ids: fit.ids.as_str(),
        path: written,
        bytes: encoded.bytes.len(),
    });
    record.reader_check = tiles.first().map(|tile| ReaderCheck {
        tile: tile.index,
        label: tile.label.clone(),
        sentence: reader_sentence(&tile.label),
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

/// Why a tile is on the sheet: ADR-0098 §6's class, and everything that follows from it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Class {
    /// A visual state's first painted frame.
    Run,
    /// A painted frame inside a state, where a keyframe change point first paints.
    Keyframe,
    /// A painted frame inside a span longer than the infill ceiling.
    Infill,
}

impl Class {
    /// What put the instant on the sheet (ADR-0094 §6).
    fn why(self) -> &'static str {
        match self {
            Class::Run => "boundary",
            Class::Keyframe => "keyframe",
            Class::Infill => "infill",
        }
    }

    /// The unabbreviated class token (ADR-0098 §6).
    fn token(self) -> &'static str {
        match self {
            Class::Run => "run",
            Class::Keyframe => "keyframe",
            Class::Infill => "infill",
        }
    }

    /// Whether its strip carries the mark: a run tile is the unmarked default.
    fn marked(self) -> bool {
        self != Class::Run
    }
}

/// One tile the sheet will draw, before it is painted.
struct Planned<'a> {
    state: &'a Interval,
    instant: i64,
    class: Class,
    /// The change points it shows.
    points: Vec<Point>,
    label: Label,
}

/// An infill tile before it is planned: the state it samples inside, and its instant.
type InfillTile = (usize, i64);

/// The infill tiles at `instants`, each with the state holding it.
fn infill_tiles(states: &[Interval], instants: &[i64]) -> Vec<InfillTile> {
    instants
        .iter()
        .filter_map(|&instant| {
            let state = states
                .iter()
                .position(|state| state.start <= instant && instant < state.end)?;
            Some((state, instant))
        })
        .collect()
}

/// Every tile, run, keyframe and infill, in clock order and so in reading order, each with
/// both forms of its label. `painted` is each state's run tile, `None` where it has none.
fn plan<'a>(
    states: &'a [Interval],
    painted: &[Option<i64>],
    placed: &Placed,
    changes: &Changes,
    infill: &[InfillTile],
) -> Vec<Planned<'a>> {
    let runs = states
        .iter()
        .zip(painted)
        .enumerate()
        .filter_map(|(i, (state, painted))| {
            let points = placed.on_run_tiles.get(&i).cloned().unwrap_or_default();
            Some((state, (*painted)?, Class::Run, points))
        });
    let keyframe_tiles = placed.tiles.iter().map(|tile| {
        (
            &states[tile.state],
            tile.instant,
            Class::Keyframe,
            tile.points.clone(),
        )
    });
    let infill_tiles = infill.iter().map(|&(state, instant)| {
        let points = placed
            .on_infill_tiles
            .get(&instant)
            .cloned()
            .unwrap_or_default();
        (&states[state], instant, Class::Infill, points)
    });
    let mut tiles: Vec<_> = runs.chain(keyframe_tiles).chain(infill_tiles).collect();
    // A keyframe tile is never at its state's run tile's frame, and an infill tile is
    // strictly between two others, so no two share an instant.
    tiles.sort_by_key(|&(_, instant, ..)| instant);
    tiles
        .into_iter()
        .enumerate()
        .map(|(i, (state, instant, class, points))| Planned {
            state,
            instant,
            class,
            label: match class {
                Class::Keyframe => Label::keyframe(i + 1, instant, points[0].at),
                Class::Infill => Label::infill(i + 1, instant, state.start),
                Class::Run => Label::run(
                    i + 1,
                    instant,
                    state.start,
                    changes.at(state.start).as_ref(),
                ),
            },
            points,
        })
        .collect()
}

/// The longest label on the sheet, in both forms, in the chrome face's units: whether they
/// fit is the sizing's to decide. A keyframe or infill tile has no identifying field, so it
/// counts in both forms at its one length and cannot force the sheet-wide elision (ADR-0106
/// D7).
fn widths(planned: &[Planned]) -> LabelWidths {
    let metrics = chrome::metrics();
    let longest = |form: fn(&Label) -> &String| {
        planned
            .iter()
            .map(|tile| metrics.advance(form(&tile.label)))
            .max()
            .unwrap_or(0)
    };
    LabelWidths {
        core: longest(|label| &label.core),
        whole: longest(|label| &label.whole),
    }
}

/// The sheet with every tile painted on it, and the record of what was painted.
struct Painted {
    canvas: Canvas,
    tiles: Vec<Tile>,
    sources: Vec<String>,
    fonts: Vec<String>,
}

/// Paint each tile at true pixels and composite it into its cell, and draw its label in the
/// strip beneath it. `None` where the run stopped, with the report saying why.
fn paint(
    document: &Loose,
    frame: (i64, i64),
    fit: Fit,
    planned: &[Planned],
    labels: &[String],
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
    let mut chrome = chrome::fonts();

    let mut painter = Painter::new(
        document,
        planned[0].instant,
        frame,
        Box::new(super::PerFrame::default()),
    );
    // One finding per `(element, code)` over the whole sheet, as `render` keeps one over a
    // whole span: the same element declining for the same reason on every tile is one fact.
    let mut declined: BTreeMap<(String, String), Finding> = BTreeMap::new();
    let mut tiles = Vec::with_capacity(planned.len());
    for (i, planned) in planned.iter().enumerate() {
        let (state, instant) = (planned.state, &planned.instant);
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
        // The label is drawn from the very string the provenance line records, so the two
        // cannot differ.
        let strip = Region {
            x: column * fit.tile_width,
            y: row * fit.cell_height + fit.tile_height,
            width: fit.tile_width,
            height: fit.strip(),
        };
        if let Err(reason) = label::draw(
            &mut sheet,
            &mut chrome,
            &labels[i],
            fit.type_px,
            strip,
            planned.class.marked(),
        ) {
            report.fail_internally(reason);
            return None;
        }
        tiles.push(Tile {
            index: i + 1,
            instant_ms: *instant,
            run: span(state),
            why: planned.class.why(),
            class: planned.class.token(),
            keyframes: planned.points.iter().map(Point::name).collect(),
            label: labels[i].clone(),
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
/// loops over (ADR-0126). `states` counts the visual states needing a run tile.
fn overflowed(from: i64, to: i64, states: usize, overflow: &Overflow) -> Finding {
    let sub_ranges: Vec<Value> = overflow
        .sub_ranges
        .iter()
        .map(|&(from, to)| json!({"from": from, "to": to}))
        .collect();
    Finding::new("E-SHEET-OVERFLOW")
        .field("from", json!(from))
        .field("to", json!(to))
        .field("states", json!(states))
        .field("fits", json!(overflow.admitted))
        .field("limit", json!(overflow.limit.as_str()))
        .field("limit_px", json!(overflow.limit.px()))
        .field("sub_ranges", json!(sub_ranges))
}

/// ADR-0106 D14's refusal of a ceiling the grid cannot paint, naming the least it can.
fn below_one_frame(spelling: &str, ceiling: i64, fps: i64) -> String {
    let period = match 1000 % fps {
        0 => format!("{}", 1000 / fps),
        _ => format!("{:.1}", 1000.0 / fps as f64),
    };
    format!(
        "`--infill-ceiling {spelling}` is under one frame period: at {fps} fps a frame is \
         painted every {period} ms, so no two tiles can be closer than that, and {ceiling} ms \
         is not a spacing the grid paints. The least ceiling at {fps} fps is {} ms",
        (1000 + fps - 1) / fps
    )
}

/// `skipped[]`: every state no frame paints, as `no-grid-frame`, and every state holding
/// instants the requested ceiling would have added and the sheet does not draw, as
/// `infill-evicted` with how many, in clock order. A state is never both: infill is a
/// painted frame.
fn skipped_states(states: &[Interval], unpainted: &[&Interval], evicted: &[i64]) -> Vec<Skipped> {
    let mut by_state: BTreeMap<usize, usize> = BTreeMap::new();
    for (state, _) in infill_tiles(states, evicted) {
        *by_state.entry(state).or_default() += 1;
    }
    let mut skipped: Vec<Skipped> = unpainted
        .iter()
        .map(|state| Skipped {
            run: span(state),
            reason: "no-grid-frame",
            present: ids(state),
            evicted: None,
        })
        .chain(by_state.into_iter().map(|(state, count)| Skipped {
            run: span(&states[state]),
            reason: "infill-evicted",
            present: ids(&states[state]),
            evicted: Some(count),
        }))
        .collect();
    skipped.sort_by_key(|entry| entry.run.start);
    skipped
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
