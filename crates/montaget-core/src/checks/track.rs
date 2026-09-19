//! The track's own two facts: *"do two of these share an instant, and is there a stretch
//! with nothing in it?"*
//!
//! One traversal, two findings, and ADR-0004 is explicit that it has to be two:
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
//! question — `R-VISUAL-GAP`, ADR-0018, #200 — and it is why every finding here is a
//! `note`: ADR-0006 computes a gap's severity *"from the consequence at an instant"*, and
//! the consequence at an instant is a fact about the whole frame, which one track cannot
//! see. Reporting `review` from here would be this check stating a number it did not
//! measure.
//!
//! A gap is also bounded by two elements **of this track** and never by the project's
//! ends. A track holding one five-second element inside a sixty-five-second project has
//! not had black frames punched into it; the distance from its end to the project's is
//! [`crate::slack`], which is a different quantity with a different owner.

use serde_json::{Value, json};

use crate::finding::Finding;
use crate::permissive::Loose;
use crate::report::Report;

use super::subject_of;

/// One element, as the sequence sees it: a name and a half-open range.
#[derive(Debug, Clone)]
pub(crate) struct Span {
    pub(crate) element: String,
    pub(crate) start: i64,
    pub(crate) end: i64,
}

/// One track's elements, in time order.
#[derive(Debug, Clone)]
pub(crate) struct Sequence {
    /// The name the document writes, where it writes one.
    pub(crate) track: Option<String>,
    pub(crate) spans: Vec<Span>,
}

impl Sequence {
    /// What a finding calls this track. Named here rather than at two call sites, so the
    /// unnamed case cannot come to mean two things (`crate::checks::subject_of`'s reason).
    pub(crate) fn name(&self) -> String {
        self.track
            .clone()
            .unwrap_or_else(|| "a track carrying no `name`".to_string())
    }
}

/// A stretch of one track with no element in it.
#[derive(Debug, Clone)]
pub(crate) struct Gap {
    pub(crate) track: String,
    pub(crate) from: i64,
    pub(crate) to: i64,
    /// The element the gap opens after — the one reaching furthest into the track so far,
    /// which is not always the one written last.
    pub(crate) after: String,
    pub(crate) before: String,
}

impl Gap {
    pub(crate) fn size(&self) -> i64 {
        self.to - self.from
    }
}

/// Both questions, over every track.
pub fn check(document: &Loose, report: &mut Report) {
    for sequence in sequences(document) {
        for finding in overlaps(&sequence) {
            report.push(located(finding, document, &sequence));
        }
        for gap in gaps_in(&sequence) {
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

/// Every pair of elements in one track that share an instant.
///
/// A sweep rather than a scan of adjacent pairs: `[0,100]`, `[10,20]`, `[30,40]` sorted by
/// `start` has one overlapping adjacent pair and two overlaps, and the one an adjacent
/// scan misses is the first element against the third. Each pair is reported once, which
/// is what makes the count the number of things to fix.
fn overlaps(sequence: &Sequence) -> Vec<Finding> {
    let mut findings = Vec::new();
    let mut active: Vec<&Span> = Vec::new();

    for span in &sequence.spans {
        // Half-open: an element ending at 7500 and its neighbour starting at 7500 do not
        // overlap — the boundary instant belongs to exactly one of them (ADR-0005).
        active.retain(|open| open.end > span.start);
        for open in &active {
            findings.push(
                Finding::new("E-TRACK-OVERLAP")
                    .at_element(open.element.clone())
                    .field("element", json!(open.element))
                    .field("start", json!(open.start))
                    .field("end", json!(open.end))
                    .field("other", json!(span.element))
                    .field("other_start", json!(span.start))
                    .field("other_end", json!(span.end))
                    .field("overlap", json!(open.end.min(span.end) - span.start)),
            );
        }
        active.push(span);
    }
    findings
}

/// Every stretch of one track with no element in it.
pub(crate) fn gaps_in(sequence: &Sequence) -> Vec<Gap> {
    let mut gaps = Vec::new();
    let Some(first) = sequence.spans.first() else {
        return gaps;
    };
    // The furthest instant the track has reached, and what reached it. Tracked rather than
    // read off the previous element, so that an overlapping element nested inside a longer
    // one cannot manufacture a gap that is not there.
    let mut covered_to = first.end;
    let mut covered_by = first.element.clone();

    for span in &sequence.spans[1..] {
        if span.start > covered_to {
            gaps.push(Gap {
                track: sequence.name(),
                from: covered_to,
                to: span.start,
                after: covered_by.clone(),
                before: span.element.clone(),
            });
        }
        if span.end > covered_to {
            covered_to = span.end;
            covered_by = span.element.clone();
        }
    }
    gaps
}

/// Every track's elements, in time order.
///
/// Array order carries no timing meaning (ADR-0004, ADR-0060), so the document's order is
/// sorted away here and never read as a sequence. An element stating no integer `start`
/// and `end` contributes nothing: a document mid-edit is exactly what a check runs on, and
/// an overlap that cannot be computed is never reported as an overlap that is not there.
pub(crate) fn sequences(document: &Loose) -> Vec<Sequence> {
    document
        .value()
        .get("tracks")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .map(|track| {
            let mut spans: Vec<Span> = track
                .get("elements")
                .and_then(Value::as_array)
                .map(Vec::as_slice)
                .unwrap_or_default()
                .iter()
                .filter_map(|element| {
                    let start = element.get("start")?.as_i64()?;
                    let end = element.get("end")?.as_i64()?;
                    // A range that is not a positive half-open interval is a schema fact
                    // and belongs to the check that owns the schema. There is no overlap
                    // and no gap to compute against it: `TimelineRange::overlaps` already
                    // states that an empty or inverted range overlaps nothing, "there is
                    // no instant at which it is on screen".
                    (end > start).then_some(Span {
                        element: subject_of(element.get("id").and_then(Value::as_str)),
                        start,
                        end,
                    })
                })
                .collect();
            // Ties broken by `end` and then by the name, so a report is byte-identical
            // across runs — nothing downstream may depend on which check spoke first, and
            // that has to include which instance of one check did.
            spans.sort_by(|a, b| (a.start, a.end, &a.element).cmp(&(b.start, b.end, &b.element)));
            Sequence {
                track: track
                    .get("name")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                spans,
            }
        })
        .collect()
}
