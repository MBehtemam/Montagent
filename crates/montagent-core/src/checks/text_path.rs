//! prototype(#764): ADR-0161's two new errors.
//!
//! - `E-TEXT-PATH-BREAK`: a `\n` in any run of a text carrying `path` (§3).
//! - `E-TEXT-PATH-OFFSET-ORPHAN`: `path_offset` on a text with no `path` (§8).

use serde_json::{Value, json};

use crate::finding::Finding;
use crate::permissive::Loose;
use crate::report::Report;

pub fn check(document: &Loose, report: &mut Report) {
    for (track, element) in document.elements_in_tracks() {
        if element.get("type").and_then(Value::as_str) != Some("text") {
            continue;
        }
        let subject = crate::checks::subject_of(element.get("id").and_then(Value::as_str));
        let mut push = |finding: Finding| {
            let finding = finding.at_file(document.path()).at_element(&subject);
            report.push(match track {
                Some(track) => finding.at_track(track),
                None => finding,
            });
        };
        if crate::text_path::has_path(element) {
            let runs = element
                .get("runs")
                .and_then(Value::as_array)
                .map(Vec::as_slice)
                .unwrap_or_default();
            for (index, run) in runs.iter().enumerate() {
                if run
                    .get("text")
                    .and_then(Value::as_str)
                    .is_some_and(|text| text.contains('\n'))
                {
                    push(Finding::new("E-TEXT-PATH-BREAK").field("run", json!(index)));
                }
            }
        } else if element.get("path_offset").is_some() {
            push(
                Finding::new("E-TEXT-PATH-OFFSET-ORPHAN")
                    .repair_value(json!({"value": "drop `path_offset`"})),
            );
        }
    }
}
