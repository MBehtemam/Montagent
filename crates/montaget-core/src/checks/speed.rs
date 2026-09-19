//! ADR-0020's invariant: *"does the timeline range agree with the source range, at the
//! `speed` the element declares?"*
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
//! # Two checks, because the two arms repair opposite ways round
//!
//! Without `overrun`, the invariant is an equality and the repair is determined: ADR-0020
//! names `speed` *"the free variable"* and requires the finding to print the value that
//! would satisfy it, so the agent edits `speed` and never the spans. That is [`mismatch`],
//! advise-class.
//!
//! With `overrun` declared, the invariant becomes an inequality — *"`end - start` must be
//! strictly greater"* — and an `overrun` on an element that does not need it *"is itself a
//! `validate` error"*. That is [`unneeded_overrun`], refuse-class: the document does not
//! say whether the author meant a longer element or no `overrun`, and those are different
//! videos.
//!
//! **Two functions rather than one with a branch, and the split is ADR-0043's.** The
//! repair class is decided *per check*, so one function picking `Advise` or `Refuse` off a
//! field of the element under test would read as a check triaging its own matches — the
//! thing the ADR forbids — even though what `overrun` selects is a different *question*
//! and not a guess at which instances look safe. Making them two checks costs less than
//! arguing that distinction: `overrun` decides which question is asked, and each question
//! then answers uniformly for every instance it matches, including the ones that look
//! safe.
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
//! the invariant, by saying nothing about an element whose `speed` it cannot read.

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

/// Everything both arms need, computed once.
///
/// The eight numbers travel together into whichever finding fires, because ADR-0006 wants
/// all eight inline — *"every number inline — especially the numbers that are not in the
/// file"* — so they are one type rather than eight parameters threaded through two
/// functions.
struct Invariant {
    source_start: i64,
    source_end: i64,
    source_span: i64,
    speed: Decimal,
    /// How long the source plays for: [`crate::exact::played_ms`], never an `f64` divide.
    played: i64,
    start: i64,
    end: i64,
    timeline_span: i64,
}

impl Invariant {
    /// `None` where the invariant cannot be evaluated at all — no source range, no
    /// readable timeline range, or a `speed` that is not a rate the format accepts.
    fn of(element: &Value) -> Option<Invariant> {
        let source_start = element.get("source_start")?.as_i64()?;
        let source_end = element.get("source_end")?.as_i64()?;
        let start = element.get("start")?.as_i64()?;
        let end = element.get("end")?.as_i64()?;

        let source_span = source_end - source_start;
        let timeline_span = end - start;
        if source_span <= 0 || timeline_span <= 0 {
            return None;
        }

        // Absent `speed` is ADR-0005's `speed == 1` special case, which ADR-0020 restates
        // the general rule from — not a defaulted field this check invents.
        let speed = match element.get("speed") {
            None => Decimal::parse("1")?,
            Some(Value::Number(number)) => Decimal::of(number)?,
            Some(_) => return None,
        };

        Some(Invariant {
            source_start,
            source_end,
            source_span,
            speed,
            played: exact::played_ms(source_span, speed)?,
            start,
            end,
            timeline_span,
        })
    }

    /// Put every number on a finding, the same way round for both arms.
    fn stated_on(&self, finding: Finding) -> Finding {
        finding
            .field("source_start", json!(self.source_start))
            .field("source_end", json!(self.source_end))
            .field("source_span", json!(self.source_span))
            .field("speed", json!(self.speed.to_string()))
            .field("played", json!(self.played))
            .field("start", json!(self.start))
            .field("end", json!(self.end))
            .field("timeline_span", json!(self.timeline_span))
    }
}

/// Which of the two questions this element's `overrun` puts.
enum Arm<'a> {
    /// No `overrun`: the invariant is an equality.
    Equality,
    /// `overrun` declared: the invariant is an inequality, and the key names the value.
    Inequality(&'a str),
}

impl<'a> Arm<'a> {
    /// `None` where `overrun` is present but is not a string. ADR-0020 fixes its legal
    /// values at `"hold"` and `"loop"`, so anything else is a schema error — and reading
    /// it as *absent* would quietly evaluate the other invariant, the equality, against an
    /// element whose author declared an overrun.
    fn of(element: &'a Value) -> Option<Arm<'a>> {
        match element.get("overrun") {
            None => Some(Arm::Equality),
            Some(Value::String(overrun)) => Some(Arm::Inequality(overrun)),
            Some(_) => None,
        }
    }
}

/// One element, through whichever arm its `overrun` selects.
fn evaluate(element: &Value) -> Option<Finding> {
    let invariant = Invariant::of(element)?;
    match Arm::of(element)? {
        Arm::Inequality(overrun) => unneeded_overrun(&invariant, overrun),
        Arm::Equality => mismatch(&invariant),
    }
}

/// **The inequality arm.** *"An `overrun` declared on an element that doesn't need it —
/// where `end - start` doesn't exceed the played duration — is itself a `validate` error"*
/// (ADR-0020).
///
/// Refuse-class for every instance it matches, the ones that look safe included: deleting
/// the key and extending `end` are different videos, and nothing in the document says
/// which was meant.
fn unneeded_overrun(invariant: &Invariant, overrun: &str) -> Option<Finding> {
    (invariant.timeline_span <= invariant.played).then(|| {
        invariant
            .stated_on(Finding::new("E-OVERRUN-UNNEEDED"))
            .field("overrun", json!(overrun))
    })
}

/// **The equality arm.** `end - start == round(source_span / speed)`, evaluated exactly.
///
/// Advise-class for every instance it matches, and ADR-0020 rather than this module is
/// what decides that: *"`speed` is the free variable: when the invariant fails,
/// `validate`'s error reports the `speed` value that would satisfy it, and the agent edits
/// `speed`, never the spans."*
fn mismatch(invariant: &Invariant) -> Option<Finding> {
    if invariant.timeline_span == invariant.played {
        return None;
    }
    let finding = invariant.stated_on(Finding::new("E-SPEED-MISMATCH"));

    // ADR-0020: the error "must compute and print the corrective `speed`, not just flag
    // the mismatch". Every value stated here has been fed back through the invariant it
    // repairs (`crate::exact::corrective_speed`), so an advise-class repair is never a
    // value that would fail the check it answers.
    //
    // The fallback states the exact ratio in words. It is unreachable on any span a
    // project can address — eighteen decimal places is far past the width of the
    // satisfying interval there — and it is written anyway because the alternative is an
    // `error` finding with no `repair`, which ADR-0043's gate turns into a panic and
    // ADR-0011 into exit 70: a defect in the check reported as Montaget failing to run.
    let (source_span, timeline_span) = (invariant.source_span, invariant.timeline_span);
    let value = match exact::corrective_speed(source_span, timeline_span) {
        Some(corrective) => format!("set `speed` to {corrective}"),
        None => format!(
            "set `speed` to the exact ratio {source_span}/{timeline_span}; no decimal of \
             eighteen or fewer places satisfies the invariant at this span"
        ),
    };
    Some(finding.repair_value(json!({"value": value})))
}
