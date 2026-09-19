//! `E-TRANSITION-RANGE` — ADR-0059: *"a transition's `start`/`end` must exactly equal the
//! intersection of the two elements it bridges."* Outside that window only one of the two
//! bridged elements exists, so a wider or narrower declared range is a field the renderer
//! cannot honour.
//!
//! The check ADR-0059 itself names: *"`validate` can therefore check transition bounds as
//! a closed-form function of the two referenced elements' own ranges — drift on either
//! side surfaces immediately as an error."* Because the correct value is exactly that
//! closed-form function of fields already in the document, this is advise-class: the
//! repair is the recomputed intersection, not a request for the author's intent.
//!
//! **What this does not check.** A `from`/`to` naming an element that is not in the
//! project at all is a dangling reference, not a drifted one — a different question this
//! check declines to answer, on [`crate::checks::anchor`]'s own precedent of reporting
//! only what it was asked and leaving an unresolved reference to whichever check owns
//! that question.
//!
//! **Document-only**: every field this needs — `start`, `end`, `from`, `to`, and the two
//! bridged elements' own `start`/`end` — is already in the document, so this needs no
//! session and can sit anywhere in `validate`'s check list.

use std::collections::HashMap;

use serde_json::{Value, json};

use crate::checks::subject_of;
use crate::finding::Finding;
use crate::permissive::Loose;
use crate::report::Report;

/// `E-TRANSITION-RANGE`, over every `transition` element in the document.
pub fn check(document: &Loose, report: &mut Report) {
    let file = document.path();
    let ranges = element_ranges(document);

    for (track, element) in document.elements_in_tracks() {
        if element.get("type").and_then(Value::as_str) != Some("transition") {
            continue;
        }
        let Some(finding) = candidate(element, &ranges) else {
            continue;
        };

        let mut finding = finding.at_file(file);
        if let Some(track) = track {
            finding = finding.at_track(track);
        }
        report.push(finding);
    }
}

/// Every element's own `start`/`end`, by `id` — the pool a transition's `from`/`to`
/// resolve against. Read straight off the permissive tree, the same reason
/// [`crate::stack::Stack`] does: a document mid-edit is exactly the one this check most
/// wants to still answer over.
fn element_ranges(document: &Loose) -> HashMap<&str, (i64, i64)> {
    document
        .elements_in_tracks()
        .filter_map(|(_, element)| {
            let id = element.get("id").and_then(Value::as_str)?;
            let start = element.get("start").and_then(Value::as_i64)?;
            let end = element.get("end").and_then(Value::as_i64)?;
            Some((id, (start, end)))
        })
        .collect()
}

/// This transition's finding, if its declared range has drifted from the intersection of
/// its two bridged elements — `None` if any field this check needs is absent or
/// unresolvable (another check's business to name) or if the range still matches.
fn candidate(element: &Value, ranges: &HashMap<&str, (i64, i64)>) -> Option<Finding> {
    let own_start = element.get("start").and_then(Value::as_i64)?;
    let own_end = element.get("end").and_then(Value::as_i64)?;
    let from = element.get("from").and_then(Value::as_str)?;
    let to = element.get("to").and_then(Value::as_str)?;

    let &(from_start, from_end) = ranges.get(from)?;
    let &(to_start, to_end) = ranges.get(to)?;

    let derived_start = from_start.max(to_start);
    let derived_end = from_end.min(to_end);

    if own_start == derived_start && own_end == derived_end {
        return None;
    }

    let subject = subject_of(element.get("id").and_then(Value::as_str));
    Some(
        Finding::new("E-TRANSITION-RANGE")
            .at_element(subject.clone())
            .field("element", json!(subject))
            .field("start", json!(own_start))
            .field("end", json!(own_end))
            .field("from", json!(from))
            .field("to", json!(to))
            .field("derived_start", json!(derived_start))
            .field("derived_end", json!(derived_end))
            .repair_value(json!({
                "value": format!(
                    "set `start` to {derived_start} and `end` to {derived_end} — the \
                     intersection of `{from}` and `{to}`"
                )
            })),
    )
}
