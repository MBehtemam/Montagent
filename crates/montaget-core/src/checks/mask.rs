//! `R-MASK-CIRCLE-NON-SQUARE` — *"does a `circle` mask discard part of a rect nobody asked
//! it to?"* (ADR-0084).
//!
//! `circle` is the one mask shape whose meaning **discards** part of its rect: its diameter
//! is `min(width, height)`, so on a 1080×1912 rect it erases 832 px of the long axis that
//! the author never typed a number for. `rect` is the rect and `ellipse` fills it; both are
//! total, and neither gets a finding — an ellipse inscribed in a non-square rect is an
//! ordinary oval and nobody is surprised by it.
//!
//! **`review`, not an error.** The behaviour is determinate, ADR-0068 ratified it, and the
//! committed fixture legitimately relies on it — `handle-logo`'s badge is 68×68, square, and
//! triggers nothing.
//!
//! **Derived and explicit rects alike.** ADR-0084 makes the bare form the identity value of
//! the same parameter set rather than a different declaration, so a check that looked only
//! at written numbers would be silent on exactly the case the finding exists for: the rect
//! nobody wrote.
//!
//! **The repair is what makes the finding honest**, and it exists only because ADR-0084
//! admits explicit geometry: use `ellipse` to fill the rect, or give the mask a square rect.
//! Under a param-less-only vocabulary the sole repair would have been "resize the element" —
//! moving the picture to satisfy a checker, which is noise. And writing the square rect *is*
//! the acknowledgement (ADR-0030: presence is content), so there is no suppression mechanism
//! and none is needed.
//!
//! Read permissively throughout: an `effects` that is not an array, a member that is not an
//! object, a `shape` that is not one of the three words, a rect field that is not an integer
//! — all are the schema check's to name, and simply are not read here.

use serde_json::{Value, json};

use crate::checks::subject_of;
use crate::finding::Finding;
use crate::model::effects::MASK_RECT;
use crate::permissive::Loose;
use crate::report::Report;

/// `R-MASK-CIRCLE-NON-SQUARE`, over every `circle` mask in the document.
pub fn check(document: &Loose, report: &mut Report) {
    let file = document.path();

    for (track, element) in document.elements_in_tracks() {
        let subject = subject_of(element.get("id").and_then(Value::as_str));
        let effects = element
            .get("effects")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or_default();

        for (index, effect) in effects.iter().enumerate() {
            let Some(finding) = candidate(element, effect, index, &subject) else {
                continue;
            };
            let mut finding = finding.at_file(file).at_element(subject.clone());
            if let Some(track) = track {
                finding = finding.at_track(track);
            }
            report.push(finding);
        }
    }
}

/// This effect's finding, if it is a `circle` mask whose rect is not square.
fn candidate(element: &Value, effect: &Value, index: usize, subject: &str) -> Option<Finding> {
    if effect.get("name").and_then(Value::as_str) != Some("mask")
        || effect.get("shape").and_then(Value::as_str) != Some("circle")
    {
        return None;
    }

    // The mask rect, by ADR-0084's own arithmetic: the four fields where all four are
    // written, and the element's own rect where none is. A partial tuple is a schema error
    // the model refuses and this check has no reading of — it asks nothing of a rect that
    // does not exist.
    let written: Vec<i64> = MASK_RECT
        .iter()
        .filter_map(|field| effect.get(*field).and_then(Value::as_i64))
        .collect();
    let (source, width, height) = match written.as_slice() {
        [_, _, width, height] => ("the mask's own rect", *width, *height),
        [] => (
            "the element's own rect, which the bare form inherits",
            element.get("width").and_then(Value::as_i64)?,
            element.get("height").and_then(Value::as_i64)?,
        ),
        _ => return None,
    };

    if width == height {
        return None;
    }

    // The ADR's own arithmetic, in its own words: "a circle on a 1080×1912 rect erases
    // 832 px". The diameter is the short axis, and what falls outside is the difference.
    let diameter = width.min(height);
    let discarded = width.abs_diff(height);

    Some(
        Finding::new("R-MASK-CIRCLE-NON-SQUARE")
            .field("element", json!(subject))
            .field("index", json!(index))
            .field("source", json!(source))
            .field("width", json!(width))
            .field("height", json!(height))
            .field("diameter", json!(diameter))
            .field("discarded", json!(discarded)),
    )
}
