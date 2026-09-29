//! `E-EMPTY-RANGE` — an element whose `start`..`end`, or whose `source_start`..`source_end`,
//! does not advance.
//!
//! ADR-0005's clock is half-open, so `"start": 500, "end": 500` holds no instant at all: the
//! element is neither painted nor mixed, at any `fps`. It is the one cross-field fact about
//! an element's range the schema cannot express, and until ADR-0107 no check stated it —
//! every clock check that met one stepped over it (`quantization`, `speed`, `coverage`,
//! `unreached`, `canvas`), each on the reasonable ground that an element with no instants
//! has nothing to say about instants. So the project validated at zero errors while
//! `render` refused it, which is the two-verbs-one-document shape ADR-0093 ruling 3 exists
//! to close.
//!
//! **The code is `render`'s own, reused rather than paralleled** (ADR-0107 §1). It is one
//! condition about one document, the repair form ADR-0043 fixes per code is `Refuse` on both
//! sides, and a second code would make the two verbs disagree about the *name* of the fact
//! instead of about its existence. With the check here, `render` refuses on the check
//! engine's report (ADR-0006) and never reaches its own arms, which ADR-0093's rule for an
//! unreachable arm makes `E-INTERNAL`.
//!
//! **`error`, and the `review` reading is dismissed rather than omitted** (ADR-0107 §2). An
//! element with no instants draws and mixes nothing, so nothing in the output is *missing*
//! in the way a dropped element is — but `review` means *"legal, renders, look at a
//! frame"*, and there is no frame at which this element could be looked at. The document
//! declares something the render cannot honour at any instant, which is what `error`'s
//! *"guaranteed wrong"* means, and it is `E-TRANSITION-NO-OVERLAP`'s class for the same
//! shape one field over.
//!
//! **One finding per empty range, and both ranges on one element are two findings.** They
//! are two facts the author repairs separately — which of `start`, `end` or the source
//! span was meant is not in the document (ADR-0020), which is why the repair is `Refuse`.
//!
//! **What this does not look at.** A `transition`'s range is derived from the two elements
//! it bridges (ADR-0059), and an empty one is already `E-TRANSITION-RANGE` or
//! `E-TRANSITION-NO-OVERLAP` — reporting it here too would be one fact under two codes. And
//! a range that is not a pair of integers is the schema's fact, not this one's.

use serde_json::{Value, json};

use crate::checks::subject_of;
use crate::finding::Finding;
use crate::permissive::Loose;
use crate::report::Report;

/// Both ranges, over every element.
pub fn check(document: &Loose, report: &mut Report) {
    for (track, element) in document.elements_in_tracks() {
        if element.get("type").and_then(Value::as_str) == Some("transition") {
            continue;
        }
        let subject = subject_of(element.get("id").and_then(Value::as_str));
        for (from, to, field) in [
            ("start", "end", "`start`..`end`"),
            ("source_start", "source_end", "`source_start`..`source_end`"),
        ] {
            let Some(finding) = empty(element, from, to, field) else {
                continue;
            };
            let finding = finding
                .at_file(document.path())
                .at_element(&subject)
                .field("element", json!(subject));
            report.push(match track {
                Some(name) => finding.at_track(name),
                None => finding,
            });
        }
    }
}

/// The finding for one pair of fields, if both are integers and the second does not exceed
/// the first. Field names and values are `render`'s own, so the two verbs print one sentence.
fn empty(element: &Value, from: &str, to: &str, field: &str) -> Option<Finding> {
    let start = element.get(from)?.as_i64()?;
    let end = element.get(to)?.as_i64()?;
    (end <= start).then(|| {
        Finding::new("E-EMPTY-RANGE")
            .field("field", json!(field))
            .field("from", json!(start))
            .field("to", json!(end))
    })
}
