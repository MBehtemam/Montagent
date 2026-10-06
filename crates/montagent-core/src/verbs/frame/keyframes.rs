//! The sheet's keyframe change points: which the census counts, where each is sampled, and
//! whether a tile shows it (ADR-0106, as ADR-0129 spells the record).
//!
//! **The population is read off the document alone.** A change point is one keyframe
//! record's `t`, on an element that is on screen in the visual state holding it, strictly
//! inside that state: `s < t < e`. One on a boundary is that state's own run tile, and one
//! outside its element's lifetime (a trimmed move) is never on screen. `volume` is not
//! among the properties: a change there is audible, and nothing audible is on the sheet.
//!
//! **Where each is sampled, and whether it is tiled, is a fact about this sheet.** A point
//! at `t` samples at the first frame the grid paints at or after `t` (ADR-0106 D10):
//!
//! - inside its own state, at the state's run tile: that tile shows it, and keeps its class
//!   (D11);
//! - inside its own state, anywhere else: a keyframe tile shows it when the flag asks for
//!   one, shared with every other point on that frame, and otherwise it is untiled as
//!   `not-requested`;
//! - past its state's end: that frame is the next tiled state's run tile (D12), which shows
//!   it where the element is on screen there. Otherwise, or with that state outside the
//!   range, it is untiled as `no-grid-frame`: disclosed, never a finding.
//!
//! An infill tile that lands on a `not-requested` point's sample frame shows it, and keeps its
//! class as a run tile does (ADR-0130).
//!
//! No instant here is derived from a curve (D13): every sample is a change point's own.

use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::Value;

use crate::exact::{self, instant_of};
use crate::permissive::Loose;
use crate::verbs::query::cuts::Interval;

use super::sheet::Span;

/// One keyframe change point: `element.property@t`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Point {
    /// Clock first, so every list of points reads in the order the video plays them.
    pub(super) at: i64,
    pub(super) element: String,
    pub(super) property: String,
}

impl Point {
    /// How provenance names it (ADR-0106 D10): `photo-05.scale@1013`.
    pub(super) fn name(&self) -> String {
        format!("{}.{}@{}", self.element, self.property, self.at)
    }
}

/// A keyframe tile the sheet adds: one painted frame inside one state, and every change
/// point sampled there.
#[derive(Debug, Clone)]
pub(super) struct KeyframeTile {
    /// The index of the state it samples inside.
    pub(super) state: usize,
    pub(super) instant: i64,
    /// In clock order, never empty.
    pub(super) points: Vec<Point>,
}

/// Why a change point in the population has no tile on this sheet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Reason {
    /// Its sample frame is a keyframe tile's, and `--keyframes` was not passed.
    NotRequested,
    /// No tile of this sheet sits at its sample frame showing its element: no painted frame
    /// is left in its state, and the next state's tile does not show the element or is
    /// outside the range.
    NoGridFrame,
}

impl Reason {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Reason::NotRequested => "not-requested",
            Reason::NoGridFrame => "no-grid-frame",
        }
    }
}

/// One change point the sheet does not show, as the census names it.
#[derive(Debug, Clone, Serialize)]
pub struct UntiledPoint {
    /// `element.property@t`.
    pub point: String,
    pub element: String,
    pub property: String,
    pub at: i64,
    /// The painted millisecond it samples at: `frame --at` this shows it. `null` only where
    /// `t` is too far out for its frame to be counted in 64 bits.
    pub sample_ms: Option<i64>,
    /// The visual state it is interior to.
    pub run: Span,
    /// `not-requested` or `no-grid-frame`.
    pub reason: &'static str,
}

/// ADR-0106 D8's census. `untiled` is the count and `untiled_points` names every member,
/// apart from `skipped[]`: an untiled change point is disclosure, never a finding
/// (ADR-0129).
#[derive(Debug, Clone, Serialize)]
pub struct KeyframeCensus {
    pub tiled: usize,
    pub untiled: usize,
    pub untiled_points: Vec<UntiledPoint>,
}

/// A change point no tile of this sheet shows, before the census names it.
#[derive(Debug, Clone)]
pub(super) struct Untiled {
    point: Point,
    sample: Option<i64>,
    /// The index of the state it is interior to.
    state: usize,
    reason: Reason,
}

/// Every change point in the range, placed.
#[derive(Debug, Default)]
pub(super) struct Placed {
    /// The keyframe tiles to add, in clock order: empty unless they were asked for.
    pub(super) tiles: Vec<KeyframeTile>,
    /// The points a run tile shows, by the index of its state.
    pub(super) on_run_tiles: BTreeMap<usize, Vec<Point>>,
    /// The points an infill tile shows, by its instant.
    pub(super) on_infill_tiles: BTreeMap<i64, Vec<Point>>,
    /// The rest, in clock order.
    pub(super) untiled: Vec<Untiled>,
}

impl Placed {
    /// The census of these points over `states`, as the answer carries it.
    pub(super) fn census_over(&self, states: &[Interval]) -> KeyframeCensus {
        KeyframeCensus {
            tiled: self
                .tiles
                .iter()
                .map(|tile| tile.points.len())
                .sum::<usize>()
                + self.on_run_tiles.values().map(Vec::len).sum::<usize>()
                + self.on_infill_tiles.values().map(Vec::len).sum::<usize>(),
            untiled: self.untiled.len(),
            untiled_points: self
                .untiled
                .iter()
                .map(|untiled| UntiledPoint {
                    point: untiled.point.name(),
                    element: untiled.point.element.clone(),
                    property: untiled.point.property.clone(),
                    at: untiled.point.at,
                    sample_ms: untiled.sample,
                    run: Span {
                        start: states[untiled.state].start,
                        end: states[untiled.state].end,
                    },
                    reason: untiled.reason.as_str(),
                })
                .collect(),
        }
    }

    /// Give the infill tiles at `instants` the untiled points sampled there. Only a
    /// `not-requested` point can be: its sample frame is inside its own state, where its
    /// element is on screen. A `no-grid-frame` point samples at a run tile's frame or past
    /// the range, and an infill tile is at neither.
    pub(super) fn onto_infill(&mut self, instants: &[i64]) {
        let (shown, untiled) = std::mem::take(&mut self.untiled)
            .into_iter()
            .partition::<Vec<_>, _>(|untiled| {
                untiled.reason == Reason::NotRequested
                    && untiled
                        .sample
                        .is_some_and(|sample| instants.binary_search(&sample).is_ok())
            });
        self.untiled = untiled;
        for untiled in shown {
            let sample = untiled.sample.expect("partitioned on a sample");
            self.on_infill_tiles
                .entry(sample)
                .or_default()
                .push(untiled.point);
        }
    }

    /// How many keyframe tiles each of `states` adds beside its run tile.
    pub(super) fn per_state(&self, states: usize) -> Vec<usize> {
        let mut counts = vec![0; states];
        for tile in &self.tiles {
            counts[tile.state] += 1;
        }
        counts
    }
}

/// Place every change point interior to one of `states`, whose run tiles sit at `painted`
/// (`None` for a state no frame paints), on a grid of `fps`. Keyframe tiles are added only
/// where `asked`.
pub(super) fn place(
    document: &Loose,
    states: &[Interval],
    painted: &[Option<i64>],
    fps: i64,
    asked: bool,
) -> Placed {
    let mut placed = Placed::default();
    let mut on_frames: BTreeMap<(usize, i64), Vec<Point>> = BTreeMap::new();
    for point in population(document, states) {
        let (i, state) = point_state(states, point.at).expect("the population is interior");
        let untiled = |point, sample, reason| Untiled {
            point,
            sample,
            state: i,
            reason,
        };
        // A frame past what 64 bits count is on no grid this sheet paints; it stays in the
        // census rather than dropping out of it.
        let Some(sample) =
            exact::frame_at_or_after(point.at, fps).map(|frame| instant_of(frame.frame, fps))
        else {
            placed
                .untiled
                .push(untiled(point, None, Reason::NoGridFrame));
            continue;
        };
        if sample < state.end {
            if painted[i] == Some(sample) {
                placed.on_run_tiles.entry(i).or_default().push(point);
            } else if asked {
                on_frames.entry((i, sample)).or_default().push(point);
            } else {
                placed
                    .untiled
                    .push(untiled(point, Some(sample), Reason::NotRequested));
            }
            continue;
        }
        // No painted frame left in its own state: the least one at or after `t` is the next
        // tiled state's run tile, if the range holds one.
        match (i + 1..states.len()).find(|&j| painted[j].is_some()) {
            Some(j) if painted[j] == Some(sample) && on_screen(&states[j], &point.element) => {
                placed.on_run_tiles.entry(j).or_default().push(point)
            }
            _ => placed
                .untiled
                .push(untiled(point, Some(sample), Reason::NoGridFrame)),
        }
    }
    placed.tiles = on_frames
        .into_iter()
        .map(|((state, instant), points)| KeyframeTile {
            state,
            instant,
            points,
        })
        .collect();
    placed
}

/// The census's population, in clock order: every keyframe record's `t` on a visual
/// property of an element with an id, strictly inside a state that element is on screen in.
fn population(document: &Loose, states: &[Interval]) -> Vec<Point> {
    let mut points: Vec<Point> = document
        .elements()
        .filter_map(|element| Some((element, element.get("id")?.as_str()?)))
        .flat_map(|(element, id)| {
            // The one list (ADR-0146), less what cannot be seen.
            crate::animatable::names()
                .iter()
                .map(String::as_str)
                .filter(|&property| property != "volume")
                .filter_map(move |property| {
                    let records = element.get(property)?.as_array()?;
                    // A keyframe list, as `Animatable` reads one: an array of objects.
                    // `scale`'s static `[sx, sy]` is not.
                    records.first().filter(|first| first.is_object())?;
                    Some(records.iter().filter_map(move |record| {
                        Some(Point {
                            at: record.get("t").and_then(Value::as_i64)?,
                            element: id.to_string(),
                            property: property.to_string(),
                        })
                    }))
                })
                .flatten()
                .chain(stagger_points(element, id))
        })
        .filter(|point| {
            point_state(states, point.at).is_some_and(|(_, state)| on_screen(state, &point.element))
        })
        .collect();
    points.sort();
    points.dedup();
    points
}

/// A stagger's change points (ADR-0151 §5): every unit list of the **first scheduled** unit
/// (the earliest start after delays) and of the **last scheduled** unit (the latest end after
/// delays and overrides, not always the highest index), plus every run override's own lists.
/// Each is named by the unit's reading-order index, `title.units[39].y@2860`, at the instant
/// it runs at. Every unit in between is left to `query --at`: a tile per letter would bury
/// every other change point.
fn stagger_points(element: &Value, id: &str) -> Vec<Point> {
    let Some(plan) = crate::units::Plan::of(element) else {
        return Vec::new();
    };
    let spans = plan.spans();
    let first = spans
        .iter()
        .min_by_key(|(unit, _, start, _)| (*start, *unit))
        .map(|span| span.0);
    let last = spans
        .iter()
        .max_by_key(|(unit, _, _, end)| (*end, *unit))
        .map(|span| span.0);
    let mut out = Vec::new();
    for unit in 0..plan.units.len() {
        let scheduled = Some(unit) == first || Some(unit) == last;
        for property in crate::units::LISTS {
            if !scheduled && !plan.overridden(unit, property) {
                continue;
            }
            let (from, late) = plan.source(unit, property);
            let Some(records) = crate::checks::keyframe_records(from, property) else {
                continue;
            };
            out.extend(records.iter().filter_map(|record| {
                Some(Point {
                    at: record.get("t").and_then(Value::as_i64)? + late,
                    element: id.to_string(),
                    property: format!("units[{unit}].{property}"),
                })
            }));
        }
    }
    out
}

/// Whether the element `id` is in `state`'s visual presence set.
fn on_screen(state: &Interval, id: &str) -> bool {
    state
        .present
        .iter()
        .any(|named| named.id.as_deref() == Some(id))
}

/// The state `at` is strictly interior to, with its index.
fn point_state(states: &[Interval], at: i64) -> Option<(usize, &Interval)> {
    states
        .iter()
        .enumerate()
        .find(|(_, state)| state.start < at && at < state.end)
}
