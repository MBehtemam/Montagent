//! `E-TRANSITION-RANGE` and `E-TRANSITION-NO-OVERLAP` — ADR-0059: *"a transition's
//! `start`/`end` must exactly equal the intersection of the two elements it bridges."*
//! Outside that window only one of the two bridged elements exists, so a wider or
//! narrower declared range is a field the renderer cannot honour.
//!
//! The check ADR-0059 itself names: *"`validate` can therefore check transition bounds as
//! a closed-form function of the two referenced elements' own ranges — drift on either
//! side surfaces immediately as an error."* Because the correct value is exactly that
//! closed-form function of fields already in the document, drift is advise-class: the
//! repair is the recomputed intersection, not a request for the author's intent.
//!
//! **That closed-form function has no domain when `from` and `to` never coexist.**
//! `max(start)..min(end)` on two ranges that do not overlap collapses to an empty or
//! inverted pair, and a transition that happens to declare exactly that pair would
//! otherwise validate clean despite bridging nothing a crossfade could ever show. Reported
//! separately as `E-TRANSITION-NO-OVERLAP`, refuse-class — the document does not say
//! whether the transition should move, one of the bridged elements should, or the
//! transition should not exist at all, so there is no single recomputed value to advise,
//! on [`crate::checks::track`]'s `E-TRACK-OVERLAP` reasoning. Its own code rather than a
//! branch inside `E-TRANSITION-RANGE`: ADR-0043's uniformity rule fixes one repair class
//! per code, and the two outcomes here are determined differently.
//!
//! **What neither checks.** A `from`/`to` naming an element that is not in the project at
//! all is a dangling reference, not a drifted one — a different question this check
//! declines to answer, on [`crate::checks::anchor`]'s own precedent of reporting only what
//! it was asked and leaving an unresolved reference to whichever check owns that question.
//!
//! **Document-only**: every field this needs — `start`, `end`, `from`, `to`, and the two
//! bridged elements' own ranges — is already in the document, so this needs no session and
//! can sit anywhere in `validate`'s check list. The two bridged elements' ranges are read
//! through [`crate::stack::Stack`] — the same index [`crate::checks::anchor`] already
//! builds for exactly this id-to-range lookup — rather than a second, independent one kept
//! in step with it by hand.

use serde_json::{Value, json};

use crate::checks::subject_of;
use crate::finding::Finding;
use crate::permissive::Loose;
use crate::report::Report;
use crate::stack::{Stack, TimelineRange};

/// `E-TRANSITION-RANGE`/`E-TRANSITION-NO-OVERLAP`, over every `transition` element in the
/// document.
pub fn check(document: &Loose, report: &mut Report) {
    let file = document.path();
    let stack = Stack::of(document);

    for (track, element) in document.elements_in_tracks() {
        if element.get("type").and_then(Value::as_str) != Some("transition") {
            continue;
        }
        let Some(finding) = candidate(element, &stack) else {
            continue;
        };

        let mut finding = finding.at_file(file);
        if let Some(track) = track {
            finding = finding.at_track(track);
        }
        report.push(finding);
    }
}

/// This transition's finding, if its declared range has drifted from the intersection of
/// its two bridged elements, or if those two elements never overlap at all — `None` if
/// any field this check needs is absent or unresolvable (another check's business to
/// name) or if a real window exists and the declared range matches it.
fn candidate(element: &Value, stack: &Stack<'_>) -> Option<Finding> {
    let own_start = element.get("start").and_then(Value::as_i64)?;
    let own_end = element.get("end").and_then(Value::as_i64)?;
    let from = element.get("from").and_then(Value::as_str)?;
    let to = element.get("to").and_then(Value::as_str)?;

    let from_range = stack.placement(from)?.range?;
    let to_range = stack.placement(to)?.range?;

    if !from_range.overlaps(to_range) {
        return Some(no_overlap(
            element.get("id").and_then(Value::as_str),
            own_start,
            own_end,
            from,
            to,
            from_range,
            to_range,
        ));
    }

    let derived_start = from_range.start.max(to_range.start);
    let derived_end = from_range.end.min(to_range.end);

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

/// `E-TRANSITION-NO-OVERLAP`: the bridged pair share no instant, so no declared range —
/// including whatever this element happens to state — is ever correct. Fires
/// unconditionally, unlike `E-TRANSITION-RANGE`, because there is no matching value it
/// could accept.
fn no_overlap(
    id: Option<&str>,
    own_start: i64,
    own_end: i64,
    from: &str,
    to: &str,
    from_range: TimelineRange,
    to_range: TimelineRange,
) -> Finding {
    let subject = subject_of(id);
    Finding::new("E-TRANSITION-NO-OVERLAP")
        .at_element(subject.clone())
        .field("element", json!(subject))
        .field("start", json!(own_start))
        .field("end", json!(own_end))
        .field("from", json!(from))
        .field("to", json!(to))
        .field("from_start", json!(from_range.start))
        .field("from_end", json!(from_range.end))
        .field("to_start", json!(to_range.start))
        .field("to_end", json!(to_range.end))
}
