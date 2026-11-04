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

/// `E-RUN-LETTER-SPACING-CONFLICT` and `R-SPACING-SUPPRESSED`, over every `text` element in
/// the document.
pub fn check(document: &Loose, report: &mut Report) {
    for (track, element) in document.all_elements_in_tracks() {
        if element.get("type").and_then(Value::as_str) != Some("text") {
            continue;
        }
        let subject = subject_of(element.get("id").and_then(Value::as_str));
        let runs = crate::verbs::measure::runs_array(element);

        if element.get("letter_spacing").is_some_and(Value::is_array) {
            for (index, run) in runs.iter().enumerate() {
                if run.get("letter_spacing").and_then(Value::as_i64).is_some() {
                    let text = run.get("text").and_then(Value::as_str).unwrap_or_default();
                    let finding = Finding::new("E-RUN-LETTER-SPACING-CONFLICT")
                        .at_file(document.path())
                        .at_element(subject.clone())
                        .field("element", json!(subject))
                        .field("run", json!(index + 1))
                        .field("text", json!(text));
                    report.push(match track {
                        Some(track) => finding.at_track(track),
                        None => finding,
                    });
                }
            }
        }

        let elem_nonzero = written_letter_spacings(element).iter().any(|&v| v != 0);
        let mut text = String::new();
        let mut run_spacings: Vec<(std::ops::Range<usize>, Option<i64>)> =
            Vec::with_capacity(runs.len());
        for run in runs {
            let Some(s) = run.get("text").and_then(Value::as_str) else {
                continue;
            };
            let start = text.len();
            text.push_str(s);
            let end = text.len();
            let ls = run.get("letter_spacing").and_then(Value::as_i64);
            run_spacings.push((start..end, ls));
        }

        let any_nonzero = elem_nonzero
            || run_spacings
                .iter()
                .any(|(_, ls)| ls.is_some_and(|v| v != 0));
        if !any_nonzero {
            continue;
        }

        let Some(found) = montagent_text::first_suppressed_where(&text, |offset| {
            let run_ls = run_spacings
                .iter()
                .find(|(range, _)| range.contains(&offset))
                .and_then(|(_, ls)| *ls);
            match run_ls {
                Some(v) => v != 0,
                None => elem_nonzero,
            }
        }) else {
            continue;
        };

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
