//! ADR-0020's invariant: *"does the timeline range agree with the source range, at the
//! rate the element declares?"*
//!
//! ```text
//! end - start == round((source_end - source_start) / speed)
//! ```
//!
//! round-half-up, and **evaluated in exact arithmetic** — [`crate::exact`] holds the
//! division, because ADR-0045 measured the alternative: naive IEEE-double evaluation
//! diverges from the exact answer *"at a small but real and reproducible rate — 0.0041% of
//! cases at `speed`'s current three-decimal precision (the fixture's own `0.645`)"*. Its
//! minimal case is `7 / 0.560`, exactly `12.5`, which round-half-up sends to `13` and `f64`
//! sends to `12`.
//!
//! # Two codes, because the two arms repair opposite ways round
//!
//! Without `overrun`, the invariant is an equality and the repair is determined: ADR-0020
//! names `speed` *"the free variable"* and requires the finding to print the value that
//! would satisfy it, so the agent edits `speed` and never the spans. That is
//! `E-SPEED-MISMATCH`, advise-class.
//!
//! With `overrun` declared, the invariant becomes an inequality — *"`end - start` must be
//! strictly greater"* — and an `overrun` on an element that does not need it *"is itself a
//! `validate` error"*. That one is refuse-class: the document does not say whether the
//! author meant a longer element or no `overrun`, and those are different videos.
//!
//! # What is deliberately not here
//!
//! Whether the source file actually holds the range declared against it. That is the disk
//! half — `E-SOURCE-OVERRUN`, [`crate::checks::source`] — and it is *"a question about the
//! source alone: the same range overruns by the same amount at any rate."* The two checks
//! share no number, which is what keeps them from printing two of them.
//!
//! A `speed` that is zero, negative or not a number at all is a schema error (ADR-0020),
//! and the check that owns the schema says so. This one reports that it cannot evaluate
//! the invariant, by saying nothing about an element whose rate it cannot read.

use serde_json::{Value, json};

use crate::exact::{self, Decimal};
use crate::finding::Finding;
use crate::permissive::Loose;
use crate::report::Report;

/// The invariant, over every element that declares a source range.
pub fn check(document: &Loose, report: &mut Report) {
    for (track, element) in document.elements_in_tracks() {
        let Some(finding) = evaluate(element) else {
            continue;
        };
        let finding = finding.at_file(document.path()).at_element(
            element
                .get("id")
                .and_then(Value::as_str)
                .map(str::to_string)
                .unwrap_or_else(|| super::subject_of(None)),
        );
        report.push(match track {
            Some(name) => finding.at_track(name),
            None => finding,
        });
    }
}

/// One element's arm of the invariant, or `None` where it holds or cannot be evaluated.
fn evaluate(element: &Value) -> Option<Finding> {
    let source_start = element.get("source_start")?.as_i64()?;
    let source_end = element.get("source_end")?.as_i64()?;
    let start = element.get("start")?.as_i64()?;
    let end = element.get("end")?.as_i64()?;

    let source_span = source_end - source_start;
    let timeline_span = end - start;
    if source_span <= 0 || timeline_span <= 0 {
        return None;
    }

    // Absent `speed` is ADR-0005's `speed == 1` special case, which ADR-0020 restates the
    // general rule from — not a defaulted field this check invents.
    let speed = match element.get("speed") {
        None => Decimal::parse("1")?,
        Some(Value::Number(number)) => Decimal::of(number)?,
        Some(_) => return None,
    };
    let played = exact::played_ms(source_span, speed)?;

    let subject = |finding: Finding| {
        finding
            .field("source_start", json!(source_start))
            .field("source_end", json!(source_end))
            .field("source_span", json!(source_span))
            .field("speed", json!(speed.to_string()))
            .field("played", json!(played))
            .field("start", json!(start))
            .field("end", json!(end))
            .field("timeline_span", json!(timeline_span))
    };

    match element.get("overrun").and_then(Value::as_str) {
        // "with `overrun` declared, `end - start` must be strictly *greater than* that
        // value" — and where it is not, the `overrun` covers nothing.
        Some(overrun) => (timeline_span <= played)
            .then(|| subject(Finding::new("E-OVERRUN-UNNEEDED")).field("overrun", json!(overrun))),
        None => (timeline_span != played).then(|| {
            let finding = subject(Finding::new("E-SPEED-MISMATCH"));
            // ADR-0020: the error "must compute and print the corrective `speed`, not just
            // flag the mismatch". Every value stated here has been fed back through the
            // invariant it repairs (`crate::exact::corrective_speed`), so an advise-class
            // repair is never a value that would fail the check it answers.
            //
            // The fallback states the exact ratio in words. It is unreachable on any span
            // a project can address — eighteen decimal places is far past the width of the
            // satisfying interval there — and it is written anyway because the alternative
            // is an `error` finding with no `repair`, which ADR-0043's gate turns into a
            // panic and ADR-0011 into exit 70: a defect in the check reported as Montaget
            // failing to run.
            let value = match exact::corrective_speed(source_span, timeline_span) {
                Some(corrective) => format!("set `speed` to {corrective}"),
                None => format!(
                    "set `speed` to the exact ratio {source_span}/{timeline_span}; no decimal of \
                     eighteen or fewer places satisfies the invariant at this span"
                ),
            };
            finding.repair_value(json!({"value": value}))
        }),
    }
}
