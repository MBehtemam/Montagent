//! `R-TEXT-PAINT-OVERRIDDEN` — a keyed element-level text paint field that every run
//! overrides (ADR-0146 §6).
//!
//! A run's `color`, `stroke` and `stroke_width` stay static and beat the element's, as they
//! always have (ADR-0146 §3). So where every run states its own value for a field the element
//! keys, no keyframe of that field reaches a single glyph: the list changes nothing, and an
//! author who wrote it meant something the file does not do.
//!
//! The fields are the ones both sides can state: the text element's animatable properties
//! that are also a run's own keys, both read off the schema, so a run field that gains a
//! keyed element counterpart later joins without an edit here. `review`, because a run
//! override written on purpose and a keyed fade written on purpose cannot both be meant.

use std::sync::OnceLock;

use serde_json::{Value, json};

use crate::finding::Finding;
use crate::permissive::Loose;
use crate::report::Report;

/// The text element's animatable properties a run can also state.
fn run_paint() -> &'static [String] {
    static FIELDS: OnceLock<Vec<String>> = OnceLock::new();
    FIELDS.get_or_init(|| {
        let schema = crate::schema::generate();
        let run = &schema["$defs"]["Run"]["properties"];
        crate::animatable::of("text")
            .iter()
            .filter(|property| run.get(&property.name).is_some())
            .map(|property| property.name.clone())
            .collect()
    })
}

pub fn check(document: &Loose, report: &mut Report) {
    for (track, element) in document.elements_in_tracks() {
        if element.get("type").and_then(Value::as_str) != Some("text") {
            continue;
        }
        let runs = crate::verbs::measure::runs_array(element);
        if runs.is_empty() {
            continue;
        }
        let subject = crate::checks::subject_of(element.get("id").and_then(Value::as_str));
        for property in run_paint() {
            if crate::animatable::records(element, property).is_none()
                || !runs.iter().all(|run| run.get(property).is_some())
            {
                continue;
            }
            let finding = Finding::new("R-TEXT-PAINT-OVERRIDDEN")
                .field("property", json!(property))
                .field("runs", json!(runs.len()))
                .at_file(document.path())
                .at_element(&subject);
            report.push(match track {
                Some(track) => finding.at_track(track),
                None => finding,
            });
        }
    }
}
