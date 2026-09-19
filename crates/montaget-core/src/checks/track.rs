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

use crate::finding::Finding;
use crate::permissive::Loose;
use crate::report::Report;
use crate::track::{self, Sequence, Span};

/// Both questions, over every track.
pub fn check(document: &Loose, report: &mut Report) {
    for sequence in track::sequences(document) {
        for finding in overlaps(&sequence) {
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
