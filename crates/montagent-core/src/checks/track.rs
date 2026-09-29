//! The track's own two facts: *"do two of these share an instant, and is there a stretch
//! with nothing in it?"*
//!
//! One traversal ([`crate::track`]), two findings, and ADR-0004 is explicit that it has to
//! be two:
//!
//! > **The overlap rule needs a validator**, and it must distinguish *overlap* from
//! > *gap*: a forgotten shift leaving a silent gap passes an overlap-only check.
//!
//! **An overlap is an `error`** — it is the rule tracks exist to enforce. ADR-0004 bought
//! the constraint knowingly (*"two elements that genuinely should overlap now need two
//! tracks"*) after eight agents ranked the constrained shape first for fewest mistakes,
//! 8 of 8, on the argument that *"a constraint you cannot forget beats a check you must
//! remember"*.
//!
//! **A gap is never an error.** `CONTEXT.md`: *"Gaps are legal and ordinary — the silence
//! between two narration lines is a gap. A gap is never an error, which is why it is
//! reported apart from an overlap rather than alongside one."* ADR-0006 restates it as a
//! check that must not fire wrongly, because its own severity rule could be misread as
//! overturning it. The committed fixture holds **27** of them and is a published video.
//!
//! # What this check does not look at
//!
//! Whether anything *else* on the timeline covers a gap. That is the cross-track coverage
//! question — `R-VISUAL-GAP`, ADR-0018, #200 — and it is why every gap here is a `note`:
//! ADR-0006 computes a gap's severity *"from the consequence at an instant"*, and the
//! consequence at an instant is a fact about the whole frame, which one track cannot see.
//! Reporting `review` from here would be this check stating a number it did not measure.

use serde_json::json;

use crate::finding::{Census, Finding};
use crate::permissive::Loose;
use crate::report::Report;
use crate::track::{self, Sequence, Span};

/// Both questions, over every track.
pub fn check(document: &Loose, report: &mut Report) {
    for sequence in track::sequences(document) {
        if let Some(finding) = overlaps(&sequence) {
            report.push(located(finding, document, &sequence));
        }
        for gap in sequence.gaps() {
            report.push(located(
                Finding::new("N-TRACK-GAP")
                    .field("from", json!(gap.from))
                    .field("to", json!(gap.to))
                    .field("size", json!(gap.size()))
                    .field("after", json!(gap.after))
                    .field("before", json!(gap.before)),
                document,
                &sequence,
            ));
        }
    }
}

/// The file, and the track — the two halves of a location that is not an element's.
///
/// `track` is also a *field*, not only a location: the template names it, and a document
/// whose track carries no `name` must still render a sentence rather than fail the whole
/// report (`crate::text` treats a missing template field as an error, correctly).
fn located(finding: Finding, document: &Loose, sequence: &Sequence) -> Finding {
    let finding = finding
        .at_file(document.path())
        .field("track", json!(sequence.name()));
    match &sequence.track {
        Some(name) => finding.at_track(name.clone()),
        None => finding,
    }
}

/// One track's overlapping elements, as **one finding carrying a census of them** — never
/// one finding per overlapping pair.
///
/// Per-pair emission was quadratic in a single authorial mistake: fifteen elements written
/// onto one track produced **105** findings and a 38,739-character report for one sentence
/// of a fix ("these fifteen are on one track and must not be"). The count read as 105
/// things to do.
///
/// ADR-0100 replaces it with ADR-0006's **sibling census**, which is what a refuse-class
/// finding carries instead of a repair (ADR-0043) — and overlap is refuse-class precisely
/// because *which* of fifteen elements should move depends on intent the document does not
/// hold. The census groups the offending elements by the **stretch of track they contend
/// for**, which is an observable, document-derived fact about them; a group is therefore a
/// knot an author untangles as a unit, and the group *count* is the number of things to
/// fix that the pair count only claimed to be.
///
/// **Nothing here names a first offending pair.** ADR-0043 requires a census never to be
/// worded so that one group reads as the correct one, and a pointer at one pair out of a
/// knot is exactly that wording. The reader gets the set.
///
/// # Why this is [`Sequence::gaps`]'s traversal and not a sweep over pairs
///
/// The per-pair check had to enumerate pairs, because a pair *was* the finding, and it
/// needed a sweep rather than a scan of neighbours: `[0,10000]`, `[1000,2000]`,
/// `[3000,4000]` sorted by `start` has one overlapping adjacent pair and two overlaps, and
/// the one a neighbour scan misses is the first element against the third.
///
/// A knot needs no pairs at all. Overlap over half-open ranges is an interval-graph
/// relation, so a knot is a **run** of the sorted spans — each one starting before the
/// furthest instant the run has reached — and that is the identical `covered_to`
/// bookkeeping [`Sequence::gaps`] does one predicate away, for the identical reason: the
/// element reaching furthest is not always the one written last, and reading the previous
/// element's `end` would split `[3000,4000]` out of the knot `[0,10000]` holds it in. One
/// traversal, linear, and the two halves of ADR-0004's *"distinguish overlap from gap"*
/// stay visibly the same walk with `<` where the other has `>`.
fn overlaps(sequence: &Sequence) -> Option<Finding> {
    let spans = &sequence.spans;
    let mut knots: Vec<Vec<usize>> = Vec::new();
    // The furthest instant the knot under construction has reached. Half-open: an element
    // ending at 7500 and its neighbour starting at 7500 do not overlap — the boundary
    // instant belongs to exactly one of them (ADR-0005) — so the test is strict, where
    // `gaps` tests the same quantity the other way round and a gap needs `>`.
    let mut covered_to = i64::MIN;

    for (i, span) in spans.iter().enumerate() {
        match (span.start < covered_to, knots.last_mut()) {
            (true, Some(knot)) => knot.push(i),
            // Not overlapping what came before, so it opens a knot of its own — which
            // stays a knot of one, and is dropped below, unless something joins it.
            _ => knots.push(vec![i]),
        }
        covered_to = covered_to.max(span.end);
    }

    // A run of one is an element with the track to itself, which is the whole point of a
    // track. Groups keep the order their first member appears on the clock — `spans` is
    // sorted by `start` — and are never sorted by size: ADR-0043 requires that a census
    // "must not be worded in a way that implies the larger group is the correct one", and
    // sorting by size is that wording written into the ordering.
    knots.retain(|knot| knot.len() > 1);
    if knots.is_empty() {
        return None;
    }

    let mut census = Census::on("contended_stretch");
    for knot in &knots {
        // The knot's own stretch of track: its first member's `start` — the spans are
        // sorted, so it is the earliest — to the furthest `end` any member reaches. The
        // union and not the contended sub-intervals, because the union is the extent the
        // knot occupies and therefore what the author is looking at, and because a knot's
        // contended part can be several disjoint intervals, which would make the one
        // value this census groups on a list.
        let from = spans[knot[0]].start;
        let to = knot.iter().map(|&i| spans[i].end).max().unwrap_or(from);
        census = census.group(
            json!(format!("{from}..{to}")),
            knot.iter().map(|&i| spans[i].element.clone()),
        );
    }

    let count: usize = knots.iter().map(Vec::len).sum();
    Some(
        Finding::new("E-TRACK-OVERLAP")
            .field("count", json!(count))
            .field("sets", json!(plural(knots.len(), "overlapping set")))
            .field("overlap", json!(contended(spans)))
            .census(census),
    )
}

/// How much of this track is covered by more than one element, in ms.
///
/// The one number per track that survives the collapse: it is what the per-pair finding's
/// own `overlap` field measured, summed over the track instead of over a pair — so the
/// two-element case reports the same figure it always did, and the fifteen-element case
/// reports contended time rather than 105 pairwise slices of it that double-count nearly
/// all of it.
///
/// A depth sweep and not a sum of pairwise intersections, for exactly that reason: three
/// elements over one 1000 ms stretch contend for 1000 ms of track, not 3000.
fn contended(spans: &[Span]) -> i64 {
    let mut events: Vec<(i64, i64)> = Vec::with_capacity(spans.len() * 2);
    for span in spans {
        events.push((span.start, 1));
        events.push((span.end, -1));
    }
    events.sort_unstable();

    let mut total = 0;
    let mut depth = 0i64;
    let mut at = 0i64;
    for (instant, delta) in events {
        if depth >= 2 {
            total += instant - at;
        }
        depth += delta;
        at = instant;
    }
    total
}

/// Pluralised by hand, because a registered template interpolates a field and never
/// inflects one (`crate::text`) — the same move `N-TEXT-INVISIBLE` makes for its own
/// counted phrase.
fn plural(n: usize, noun: &str) -> String {
    match n {
        1 => format!("{n} {noun}"),
        _ => format!("{n} {noun}s"),
    }
}
