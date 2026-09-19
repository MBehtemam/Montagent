//! `E-HIGHLIGHT-RANGE` and `E-HIGHLIGHT-OVERLAP` — ADR-0051's two document-only checks on
//! a run's per-word `highlight` window (ADR-0048): *"a whole-document unit or offset
//! mixup that would otherwise validate clean and render every word wrong or invisible"*.
//!
//! **Containment.** A run's `highlight.start`/`highlight.end` must fall within its parent
//! element's own `[start, end)` — half-open, matching every other interval in the format.
//! Catches the highest-frequency merge-script failure ADR-0051 names: a systematic offset
//! or unit mixup in a hand-scripted alignment merge.
//!
//! **Non-overlap.** Sibling runs of one element must not carry overlapping `highlight`
//! windows — two different words can't both be "the highlighted one" at the same instant.
//! Catches a distinct failure containment can't see: alignment data sliced against the
//! wrong sentence, or a script that copies one whole-sentence window into every run.
//!
//! Both are `error`/refuse-class, on `E-TRACK-OVERLAP`'s own reasoning
//! ([`crate::checks::track`]): the document does not say which side of the mismatch is
//! wrong — the run's window, or the element's own range — so there is no fix a check
//! could compute rather than ask for.
//!
//! **Document-only, ADR-0051's own scope**: no font, no I/O, so this needs no session and
//! can sit anywhere in `validate`'s check list.

use serde_json::{Value, json};

use crate::checks::subject_of;
use crate::finding::Finding;
use crate::permissive::Loose;
use crate::report::Report;

/// One run's highlight window, within its parent element.
struct Window {
    run: String,
    start: i64,
    end: i64,
}

/// Both checks, over every `text` element that carries at least one `highlight` window.
pub fn check(document: &Loose, report: &mut Report) {
    let file = document.path();

    for (track, element) in document.elements_in_tracks() {
        if element.get("type").and_then(Value::as_str) != Some("text") {
            continue;
        }
        let windows = highlight_windows(element);
        if windows.is_empty() {
            continue;
        }
        let subject = subject_of(element.get("id").and_then(Value::as_str));

        let at = |finding: Finding| {
            let finding = finding.at_file(file).at_element(subject.clone());
            match track {
                Some(track) => finding.at_track(track),
                None => finding,
            }
        };

        if let (Some(own_start), Some(own_end)) = (
            element.get("start").and_then(Value::as_i64),
            element.get("end").and_then(Value::as_i64),
        ) {
            for window in &windows {
                if window.start < own_start || window.end > own_end {
                    report.push(at(Finding::new("E-HIGHLIGHT-RANGE")
                        .field("element", json!(subject))
                        .field("run", json!(window.run))
                        .field("start", json!(window.start))
                        .field("end", json!(window.end))
                        .field("element_start", json!(own_start))
                        .field("element_end", json!(own_end))));
                }
            }
        }

        for finding in overlaps(&subject, &windows) {
            report.push(at(finding));
        }
    }
}

/// Every run of this element that carries a `highlight` window, in document order.
fn highlight_windows(element: &Value) -> Vec<Window> {
    element
        .get("runs")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .filter_map(|run| {
            let highlight = run.get("highlight")?;
            let start = highlight.get("start").and_then(Value::as_i64)?;
            let end = highlight.get("end").and_then(Value::as_i64)?;
            let run = run.get("text").and_then(Value::as_str)?.to_string();
            Some(Window { run, start, end })
        })
        .collect()
}

/// Every pair of sibling windows that share an instant, on `crate::checks::track`'s own
/// sweep: sorted by `start`, each newly-opened window is checked against every window
/// still active rather than only its immediate predecessor, so a window fully containing
/// two later ones is still caught against both.
fn overlaps(subject: &str, windows: &[Window]) -> Vec<Finding> {
    let mut ordered: Vec<&Window> = windows.iter().collect();
    ordered.sort_by_key(|window| window.start);

    let mut findings = Vec::new();
    let mut active: Vec<&Window> = Vec::new();
    for window in ordered {
        // Half-open: a highlight ending at 5680 and its neighbour starting at 5680 do not
        // overlap (ADR-0005's rule, restated for this interval).
        active.retain(|open| open.end > window.start);
        for open in &active {
            report_overlap(subject, open, window, &mut findings);
        }
        active.push(window);
    }
    findings
}

fn report_overlap(subject: &str, open: &Window, window: &Window, findings: &mut Vec<Finding>) {
    let overlap = open.end.min(window.end) - window.start;
    findings.push(
        Finding::new("E-HIGHLIGHT-OVERLAP")
            .field("element", json!(subject))
            .field("run", json!(open.run))
            .field("start", json!(open.start))
            .field("end", json!(open.end))
            .field("other", json!(window.run))
            .field("other_start", json!(window.start))
            .field("other_end", json!(window.end))
            .field("overlap", json!(overlap)),
    );
}
