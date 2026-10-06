//! `R-SPACING-SUPPRESSED` — *"letter spacing does nothing between these letters"*
//! (ADR-0153 §4, [#694](https://github.com/MBehtemam/Montagent/issues/694)).
//!
//! A joining script's letters take no letter spacing between them: spacing would tear every
//! cursive join, and CSS Text 3 §7.2.1 tells a renderer that does not elongate to add none.
//! So an Arabic title keyed from 0 to 200 spreads its words apart and leaves each word as it
//! was. The file cannot show that, and the frame shows it only to a reader who knew to look,
//! so `validate` says it.
//!
//! **`review`, never `error`.** The spacing still does something — it spreads spaces,
//! punctuation, digits and every other script on the line — and a tracked mixed line is a
//! legitimate thing to write.
//!
//! It fires only where a pair was actually suppressed, and only on an element with a
//! non-zero `letter_spacing` value or keyframe: an Arabic element whose spacing is zero
//! throughout loses nothing. Both halves come from the file alone — the pairs from the
//! text's Unicode properties, never from a font.

use serde_json::{Value, json};

use crate::checks::subject_of;
use crate::finding::Finding;
use crate::permissive::Loose;
use crate::report::Report;
use crate::verbs::measure::written_letter_spacings;

/// `R-SPACING-SUPPRESSED`, over every `text` element in the document.
pub fn check(document: &Loose, report: &mut Report) {
    for (track, element) in document.elements_in_tracks() {
        if element.get("type").and_then(Value::as_str) != Some("text") {
            continue;
        }
        if written_letter_spacings(element).iter().all(|&v| v == 0) {
            continue;
        }
        let text: String = crate::verbs::measure::runs_array(element)
            .iter()
            .filter_map(|run| run.get("text").and_then(Value::as_str))
            .collect();
        let Some(found) = montagent_text::first_suppressed(&text) else {
            continue;
        };
        let subject = subject_of(element.get("id").and_then(Value::as_str));
        let finding = Finding::new("R-SPACING-SUPPRESSED")
            .at_file(document.path())
            .at_element(subject.clone())
            .field("element", json!(subject))
            .field("script", json!(found.script))
            .field("word", json!(found.word));
        report.push(match track {
            Some(track) => finding.at_track(track),
            None => finding,
        });
    }
}
