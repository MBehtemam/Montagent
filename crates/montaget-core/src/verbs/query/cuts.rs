//! `--from --to` — the cut list.
//!
//! ADR-0011: *"the intervals over which the set of on-screen elements is constant. Not
//! sampled instants. It **must always name the boundary immediately outside the range on
//! each side**, which folds in the `boundaries` want without a fourth verb and without the
//! caller guessing a window."*
//!
//! ## Where this departs from the ADR's wording, and why
//!
//! The ADR says *on-screen*; this builds the presence set from **every element**, audio
//! included, and states each member's `type` so a caller wanting only the visual cut list
//! filters one field. Surfaced rather than done quietly, because it is a departure:
//!
//! - ADR-0001 is explicit that *"audio is an element like any other; nothing owns it"*, and
//!   a verb that silently dropped a third of the fixture's elements from *"the presence
//!   set"* would be the one place in Montaget where the word means something narrower.
//! - The information only travels one way. A caller given every element can compute the
//!   visual cut list; a caller given the visual one cannot recover where the narration
//!   started, and the drift ADR-0011's own consumer task hunts — *"an 800 ms drift on
//!   disk"* — is a relationship between a narration boundary and a photo boundary.
//!
//! Nothing here judges the intervals. Whether a stretch with nothing in it is a defect is
//! `validate`'s question (ADR-0006), and this verb reports that the stretch exists.

use serde::Serialize;
use serde_json::Value;

use crate::permissive::Loose;

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
    pub present: Vec<Present>,
}

/// One element, as a member of a presence set.
#[derive(Debug, Clone, Serialize)]
pub struct Present {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: Option<String>,
    pub track: Option<String>,
    pub group: Option<String>,
}

/// An element the document places on the clock.
struct Placed {
    id: String,
    start: i64,
    end: i64,
    present: Present,
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
        let id = element
            .get("id")
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or_else(|| format!("(element {index}, no id)"));
        match (
            element.get("start").and_then(Value::as_i64),
            element.get("end").and_then(Value::as_i64),
        ) {
            (Some(start), Some(end)) => placed.push(Placed {
                id: id.clone(),
                start,
                end,
                present: Present {
                    id,
                    kind: element
                        .get("type")
                        .and_then(Value::as_str)
                        .map(str::to_string),
                    track: track.map(str::to_string),
                    group: element
                        .get("group")
                        .and_then(Value::as_str)
                        .map(str::to_string),
                },
            }),
            _ => unplaced.push(id),
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
    let intervals = cut_points
        .windows(2)
        .map(|pair| interval(&placed, pair[0], pair[1]))
        .collect();

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

fn interval(placed: &[Placed], start: i64, end: i64) -> Interval {
    Interval {
        start,
        end,
        // `saturating_sub`, not bare arithmetic: a hand-written `start` of `i64::MIN` makes
        // the difference overflow, and a verb that aborted on a number it was only
        // reporting would be the least useful moment for Montaget to stop.
        duration_ms: end.saturating_sub(start),
        // Constant across the whole interval by construction — no boundary falls strictly
        // inside it — so it is read once, at the instant the interval opens.
        present: placed
            .iter()
            .filter(|element| element.start <= start && element.end > start)
            .map(|element| element.present.clone())
            .collect(),
    }
}

fn boundary(placed: &[Placed], at: i64) -> Boundary {
    Boundary {
        at,
        entering: placed
            .iter()
            .filter(|element| element.start == at)
            .map(|element| element.id.clone())
            .collect(),
        leaving: placed
            .iter()
            .filter(|element| element.end == at)
            .map(|element| element.id.clone())
            .collect(),
    }
}
