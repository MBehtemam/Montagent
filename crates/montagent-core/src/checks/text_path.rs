//! What the schema cannot state about a text on a path (ADR-0161 §8).
//!
//! - The four point errors, `E-PATH-TOO-FEW-POINTS`, `E-PATH-DANGLING-HANDLE`,
//!   `E-PATH-KEYFRAME-SHAPE` and `E-PATH-OUTSIDE-BOX`, on the text's `path`, through the one
//!   host-generic [`crate::checks::path::points`]. Each names the element type, and the box is
//!   the text's declared box inset by §7's `m`.
//! - `E-TEXT-PATH-BREAK`: a `\n` in a run of a text carrying `path`, which sets one line only
//!   (§3). One finding per run, naming it.
//! - `E-TEXT-PATH-OFFSET-ORPHAN`: `path_offset` on a text with no `path`, where it places
//!   nothing.
//!
//! Document-only, and read permissively: a `path` that is not an object, or runs that are
//! not an array, are the schema check's to name.

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
        if crate::text_path::carries_path(element) {
            crate::checks::path::text_points(element)
                .into_iter()
                .for_each(&mut push);
            for (index, run) in crate::verbs::measure::runs_array(element)
                .iter()
                .enumerate()
            {
                if run
                    .get("text")
                    .and_then(Value::as_str)
                    .is_some_and(|text| text.contains('\n'))
                {
                    push(Finding::new("E-TEXT-PATH-BREAK").field("run", json!(index)));
                }
            }
        } else if element.get("path_offset").is_some() && element.get("path").is_none() {
            push(
                Finding::new("E-TEXT-PATH-OFFSET-ORPHAN")
                    .repair_value(json!({"value": "drop `path_offset`"})),
            );
        }
    }
}
