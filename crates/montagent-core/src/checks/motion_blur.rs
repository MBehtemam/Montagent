//! **`R-MOTION-BLUR-STILL`** (`review`, ADR-0155 §5): an element carrying `motion_blur`
//! whose resolved values never differ between two instants inside `[start, end)`, a `units`
//! stagger counting as motion. Such a field paints nothing a sharp element would not, so the
//! repair is to remove it. Motion lying wholly outside the element's life counts as still,
//! so the finding fires only when the field is certainly useless.
//!
//! Decided from the file without painting a frame, through
//! [`crate::motion_blur::still_throughout`]. No check is made for `video` (a video with keyed
//! motion is a proper use, and its footage is not blurred) and none for cost.

use serde_json::{Value, json};

use crate::finding::Finding;
use crate::permissive::Loose;
use crate::report::Report;

pub fn check(document: &Loose, report: &mut Report) {
    for (track, element) in document.elements_in_tracks() {
        if crate::motion_blur::of(element).is_none() {
            continue;
        }
        if crate::motion_blur::still_throughout(element) != Some(true) {
            continue;
        }
        let subject = crate::checks::subject_of(element.get("id").and_then(Value::as_str));
        let finding = Finding::new("R-MOTION-BLUR-STILL")
            .field("element", json!(subject))
            .field("start", json!(element.get("start")))
            .field("end", json!(element.get("end")))
            .at_file(document.path())
            .at_element(&subject);
        report.push(match track {
            Some(track) => finding.at_track(track),
            None => finding,
        });
    }
}
