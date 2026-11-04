//! ADR-0232's frame blending check (`#1106`).
//!
//! `frame_blend` on a `video` element admits `"none"` (default nearest-prior-frame hold,
//! ADR-0096) or `"mix"` (linear cross-dissolve between consecutive source frames, ADR-0232).
//! Any other value on a `video` element is refused as `E-INVALID-FRAME-BLEND`; on any other
//! element type `frame_blend` is rejected as an unknown key by the closed schema check
//! (`E-SCHEMA-UNKNOWN-KEY`).

use serde_json::{Value, json};

use crate::finding::Finding;
use crate::model::FrameBlend;
use crate::permissive::Loose;
use crate::report::Report;

/// Whether `element`'s schema fault was caused by an invalid `frame_blend` on a `video`
/// element that [`check`] reports as `E-INVALID-FRAME-BLEND`.
pub(crate) fn speaks_for(element: &Value, reason: &str) -> bool {
    match element.get("type").and_then(Value::as_str) {
        Some("video") => {
            let Some(fb) = element.get("frame_blend") else {
                return false;
            };
            let Err(err) = serde_json::from_value::<FrameBlend>(fb.clone()) else {
                return false;
            };
            reason.contains(&err.to_string())
        }
        Some("nest") => element
            .get("elements")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .any(|child| speaks_for(child, reason)),
        _ => false,
    }
}

/// Run the `frame_blend` check across every `video` element in `document`.
pub fn check(document: &Loose, report: &mut Report) {
    for (track, element) in document.all_elements_in_tracks() {
        if element.get("type").and_then(Value::as_str) != Some("video") {
            continue;
        }
        let Some(fb) = element.get("frame_blend") else {
            continue;
        };
        if serde_json::from_value::<FrameBlend>(fb.clone()).is_ok() {
            continue;
        }
        let id = element.get("id").and_then(Value::as_str);
        let subject = crate::checks::subject_of(id);
        let finding = Finding::new("E-INVALID-FRAME-BLEND")
            .at_file(document.path())
            .at_element(subject.clone())
            .field("element", json!(subject))
            .field("value", fb.clone());
        report.push(match track {
            Some(name) => finding.at_track(name),
            None => finding,
        });
    }
}
