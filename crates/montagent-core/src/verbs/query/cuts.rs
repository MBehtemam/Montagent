//! `--from --to` — the cut list.
//!
//! ADR-0011: *"the intervals over which the [presence set] is constant. Not sampled
//! instants. It **must always name the boundary immediately outside the range on each
//! side**, which folds in the `boundaries` want without a fourth verb and without the
//! caller guessing a window."*
//!
//! ## The presence set is every element, and `unplaced` is part of the answer
//!
//! ADR-0011 originally wrote *on-screen* there. ADR-0074 amends it to the defined term,
//! and settles #250: the presence set is built from **every element**, audio included,
//! each member stating its `type` so a caller wanting only the visual cut list filters one
//! field.
//!
//! - ADR-0001 is explicit that *"audio is an element like any other; nothing owns it"*, and
//!   a verb that silently dropped a third of the fixture's elements from *"the presence
//!   set"* would be the one place in Montagent where the word means something narrower.
//! - The information only travels one way. A caller given every element can compute the
//!   visual cut list; a caller given the visual one cannot recover where the narration
//!   started — 28 of the fixture's 47 boundaries are reachable only through audio
//!   (`docs/adr/cut_presence_scan.py`) — and the drift ADR-0011's own consumer task hunts,
//!   *"an 800 ms drift on disk"*, is a relationship between a narration boundary and a
//!   photo boundary.
//!
//! `unplaced`, below, is specified by the same ADR.
//!
//! Nothing here judges the intervals. Whether a stretch with nothing in it is a defect is
//! `validate`'s question (ADR-0006), and this verb reports that the stretch exists.

use serde::Serialize;
use serde_json::Value;

use crate::permissive::Loose;

use super::Named;

/// The cut list over one range, with the boundary immediately outside it on each side.
#[derive(Debug, Clone, Serialize)]
pub struct Cuts {
    /// The range asked for, half-open `[from, to)` like every other range in the format
    /// (ADR-0005). An element ending exactly at `from` is not in the first interval, and an
    /// element starting exactly at `to` is not in the last.
    pub from: i64,
    pub to: i64,
    /// The last boundary strictly before `from`, or `null` where the document has none.
    pub previous: Option<Boundary>,
    /// The intervals, in clock order. Consecutive and gapless: they partition `[from, to)`.
    pub intervals: Vec<Interval>,
    /// The first boundary at or after `to`, or `null` where the document has none.
    ///
    /// *At* or after, because `to` is outside the half-open range: a boundary landing
    /// exactly on it is the first one the range does not contain, which makes the two sides
    /// symmetric under the same convention.
    pub next: Option<Boundary>,
    /// Elements the document does not place on the clock — no `start`, no `end`, or one of
    /// them written as something other than whole milliseconds.
    ///
    /// Named rather than dropped. A cut list silently computed over 58 of 60 elements is a
    /// wrong answer that looks like a right one; *which* way the range is malformed is
    /// `validate`'s question and not a view's.
    pub unplaced: Vec<String>,
}

/// One instant at which the presence set changes, and what changes there.
#[derive(Debug, Clone, Serialize)]
pub struct Boundary {
    pub at: i64,
    /// Elements whose `start` is this instant.
    pub entering: Vec<String>,
    /// Elements whose `end` is this instant. Half-open, so an element leaving at `t` is
    /// already gone at `t` (ADR-0005).
    pub leaving: Vec<String>,
}

/// One stretch over which the presence set does not change.
#[derive(Debug, Clone, Serialize)]
pub struct Interval {
    pub start: i64,
    pub end: i64,
    /// `end - start`. Derived, never stored — the format carries no `duration` on anything
    /// that has a range (ADR-0005).
    pub duration_ms: i64,
    pub present: Vec<Named>,
}

/// An element the document places on the clock.
struct Placed {
    /// What the lists of names above call it — its `id`, or where it was found.
    name: String,
    start: i64,
    end: i64,
    named: Named,
}

/// Build the cut list over `[from, to)`.
pub fn cuts(document: &Loose, from: i64, to: i64) -> Cuts {
    let mut placed: Vec<Placed> = Vec::new();
    let mut unplaced: Vec<String> = Vec::new();

    for (index, (track, element)) in document.elements_in_tracks().enumerate() {
        // An element with no `id` still occupies the clock, and a cut list that omitted it
        // would report a presence set that is not the document's. ADR-0019 requires the id;
        // that it is missing is `validate`'s finding, and this names the element by where it
        // was found so the two reports can be read side by side.
        let named = Named::of(element, track);
        let name = named.called(index);
        match (
            element.get("start").and_then(Value::as_i64),
            element.get("end").and_then(Value::as_i64),
        ) {
            (Some(start), Some(end)) => placed.push(Placed {
                name,
                start,
                end,
                named,
            }),
            _ => unplaced.push(name),
        }
    }

    // Every instant at which some element begins or ends. Sorted and deduplicated, because
    // two elements changing at one instant is one boundary and not two — the fixture's cuts
    // are exactly that, a photo and its caption turning over together.
    let mut boundaries: Vec<i64> = placed
        .iter()
        .flat_map(|element| [element.start, element.end])
        .collect();
    boundaries.sort_unstable();
    boundaries.dedup();

    let cut_points = cut_points(&boundaries, from, to);
    let intervals = merged(
        cut_points
            .windows(2)
            .map(|pair| interval(&placed, pair[0], pair[1]))
            .collect(),
    );

    Cuts {
        from,
        to,
        previous: boundaries
            .iter()
            .rev()
            .find(|at| **at < from)
            .map(|at| boundary(&placed, *at)),
        intervals,
        next: boundaries
            .iter()
            .find(|at| **at >= to)
            .map(|at| boundary(&placed, *at)),
        unplaced,
    }
}

/// The **visual states** of a cut list: its intervals with audio members dropped and equal
/// neighbours re-merged (ADR-0094 §1).
///
/// ADR-0074 kept this out of `query` itself — *"a caller that wants the visual cut list
/// filters one field of an answer it already has"* — and `validate`'s quantization check
/// is that caller, as `frame`'s range mode will be (#488). The two are to share this one
/// function rather than each filtering for itself, because an unpainted visual state is `N-QUANTIZATION` from
/// both (ADR-0105 §5), and one fact has one identity only if both verbs cut the clock at
/// the same places.
///
/// The filter is one line; the re-merge is the work, and it is [`merged`] — the same join
/// the cut list's own intervals went through. Dropping narration makes neighbours that
/// differed only in audio identical, and that join is what turns the fixture's 46
/// intervals into 18 states. Equality stays on the set: two intervals that differ only in
/// *which* text card is up are two states.
///
/// Only `type: "audio"` is dropped (ADR-0118). An element with no `type`, or one this build does not
/// know, is kept — a state wrongly split is visible on a sheet, and one wrongly merged away
/// is not.
pub fn visual_states(cuts: &Cuts) -> Vec<Interval> {
    merged(
        cuts.intervals
            .iter()
            .map(|interval| Interval {
                present: interval
                    .present
                    .iter()
                    .filter(|named| named.kind.as_deref() != Some("audio"))
                    .cloned()
                    .collect(),
                ..interval.clone()
            })
            .collect(),
    )
}

/// The instants the range is cut at: its own two edges, plus every boundary strictly
/// inside it.
///
/// `from` and `to` are always present, so the intervals partition the range the caller
/// asked about rather than the sub-range the document happens to have boundaries in. A
/// caller that asked about a stretch with nothing in it gets one interval with an empty
/// presence set, which is an answer; an empty list would read as "the question failed".
fn cut_points(boundaries: &[i64], from: i64, to: i64) -> Vec<i64> {
    let mut points = vec![from];
    points.extend(
        boundaries
            .iter()
            .copied()
            .filter(|at| *at > from && *at < to),
    );
    points.push(to);
    points
}

/// Join adjacent intervals whose presence set is the same one.
///
/// The claim the mode makes is *"the intervals over which the presence set is constant"*,
/// and an interval that could have been longer is not that: it is a cut the reader has to
/// notice nothing happened at. The case is real rather than theoretical — an element whose
/// `start` equals its `end` occupies no instant of the half-open clock (ADR-0005) but still
/// contributes two boundaries, so cutting at every boundary produces neighbours that are
/// indistinguishable.
fn merged(intervals: Vec<Interval>) -> Vec<Interval> {
    let mut out: Vec<Interval> = Vec::with_capacity(intervals.len());
    for interval in intervals {
        match out.last_mut() {
            Some(last) if same_members(last, &interval) => {
                last.end = interval.end;
                last.duration_ms = last.end.saturating_sub(last.start);
            }
            _ => out.push(interval),
        }
    }
    out
}

/// Two presence sets are the same set when they hold the same elements in the same order —
/// and the order is one traversal's, so equal membership implies equal order.
fn same_members(a: &Interval, b: &Interval) -> bool {
    a.present.len() == b.present.len() && a.present.iter().zip(&b.present).all(|(a, b)| a == b)
}

fn interval(placed: &[Placed], start: i64, end: i64) -> Interval {
    Interval {
        start,
        end,
        // `saturating_sub`, not bare arithmetic: a hand-written `start` of `i64::MIN` makes
        // the difference overflow, and a verb that aborted on a number it was only
        // reporting would be the least useful moment for Montagent to stop.
        duration_ms: end.saturating_sub(start),
        // Constant across the whole interval by construction — no boundary falls strictly
        // inside it — so it is read once, at the instant the interval opens.
        present: placed
            .iter()
            .filter(|element| element.start <= start && element.end > start)
            .map(|element| element.named.clone())
            .collect(),
    }
}

fn boundary(placed: &[Placed], at: i64) -> Boundary {
    Boundary {
        at,
        entering: placed
            .iter()
            .filter(|element| element.start == at)
            .map(|element| element.name.clone())
            .collect(),
        leaving: placed
            .iter()
            .filter(|element| element.end == at)
            .map(|element| element.name.clone())
            .collect(),
    }
}
