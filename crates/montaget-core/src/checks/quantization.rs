//! *"What does quantization change?"* — never *"what is unaligned?"*
//!
//! ADR-0006 rewrote this check before it was ever implemented, and the fixture is the
//! argument:
//!
//! > ADR-0005 asks `validate` to *"note boundaries that are not frame-aligned."* **That
//! > instruction is wrong as literally written, and the fixture is the proof:** 109 of its
//! > 120 time values — 47 of 48 distinct instants — are off the 40 ms grid at the
//! > project's own 25 fps, the only aligned instant being `0`. A check with a 98% hit rate
//! > on a correct, published project is not a check; it is the alarm fatigue this ADR
//! > already identified as a safety problem.
//!
//! So alignment is not the question. The question is whether the frames the renderer
//! actually samples still show what the document declares, and ADR-0006 names the two ways
//! they might not:
//!
//! 1. **An element shorter than a frame**, which *"rounds out of existence"* — no sampled
//!    instant falls inside its half-open range, so it never appears. *"No element is
//!    shorter than a frame (the shortest is 1000 ms)"* on the fixture.
//! 2. **A gap shorter than a frame**, the other half of *"rounding can never manufacture
//!    an overlap or a gap"* — the black frames a gap declares are not there if no sampled
//!    instant falls in it. The fixture's shortest gap is 450 ms.
//!
//! An **overlap** is the case that cannot arise, and it is worth saying why rather than
//! leaving it untested: *"every cut is an exact adjacency, so both sides quantize to the
//! same frame."* One element ends where the next begins, at one instant, and one instant
//! resolves to one frame — there is no rounding by which the two sides could disagree.
//!
//! The grid is ADR-0035's and is evaluated in exact rational arithmetic
//! ([`crate::exact::holds_a_sampled_frame`]): the step is `1000/fps` ms and *"not
//! necessarily integral"* — at 30 fps it is `100/3` and only multiples of 100 ms are
//! frame-exact, *"the non-obvious fact one agent found only by building a private
//! checker"*.
//!
//! On a project where quantization changes nothing, this check emits nothing at all.

use serde_json::{Value, json};

use crate::exact;
use crate::finding::{Class, Finding};
use crate::permissive::Loose;
use crate::report::Report;
use crate::track;

/// The whole project, against its own frame grid.
///
/// One finding, not one per boundary. ADR-0006's answer on a clean project is *"one line
/// — nothing changes"*, and its complaint about the literal check was volume; a
/// per-boundary finding would reproduce that shape with a smaller constant.
pub fn check(document: &Loose, report: &mut Report) {
    let Some(fps) = document.value().get("fps").and_then(Value::as_i64) else {
        // No grid, so no question. `fps` is required, and the check that says so owns the
        // schema.
        return;
    };

    let mut changed: Vec<i64> = Vec::new();
    let mut detail: Vec<String> = Vec::new();

    for sequence in track::sequences(document) {
        for span in &sequence.spans {
            if exact::holds_a_sampled_frame(span.start, span.end, fps) == Some(false) {
                changed.extend([span.start, span.end]);
                detail.push(format!(
                    "`{}` ({}..{} ms) holds no sampled frame",
                    span.element, span.start, span.end
                ));
            }
        }
        for gap in sequence.gaps() {
            if exact::holds_a_sampled_frame(gap.from, gap.to, fps) == Some(false) {
                changed.extend([gap.from, gap.to]);
                detail.push(format!(
                    "the {} ms gap in track `{}` between `{}` and `{}` ({}..{} ms) holds none",
                    gap.size(),
                    gap.track,
                    gap.after,
                    gap.before,
                    gap.from,
                    gap.to
                ));
            }
        }
    }

    if detail.is_empty() {
        return;
    }

    // Distinct instants: one boundary shared by an element that vanishes and the gap after
    // it is one thing the grid changed, not two.
    changed.sort_unstable();
    changed.dedup();

    report.push(
        // ADR-0006: "Escalate to `review` only for the cases in (1) and (2)" — which are
        // the only two cases this check fires on, so every finding it emits is one. The
        // frames the renderer samples do not show what the document declares, and no
        // arithmetic can tell you whether that was meant.
        Finding::at_class("N-QUANTIZATION", Class::Review)
            .at_file(document.path())
            .field("fps", json!(fps))
            .field("changed", json!(changed.len()))
            .field("detail", json!(detail.join("; "))),
    );
}
