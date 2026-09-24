//! The anchor checks — *"does this anchor resolve, and does resolving it change anything?"*
//!
//! ADR-0019 gave `validate` two checks over the anchor, and both exist because all three
//! agents in [#8](https://github.com/MBehtemam/Montagent/issues/8)'s editing exercise wrote
//! a defective anchor and nothing complained:
//!
//! - **It does not resolve** → `error`. A target that is absent, that is the element
//!   itself, or whose own layer is another anchor. ADR-0019 splits the last one out by
//!   construction rather than by search: *"an anchor's target's own `layer` must not itself
//!   be an object. No walk, no possibility of a cycle by construction."*
//! - **It resolves and can never matter** → `review`. Stacking has consequences only where
//!   two elements are on screen at once, so an anchor whose target never overlaps it in
//!   time is a permanent no-op — *"the hardest class of error to catch by reading"*, and a
//!   silent no-op that looks like a completed edit is exactly what `review` is for.
//!
//! Neither decides anything about layers itself. Both ask [`crate::stack`] and report what
//! it could not answer, which is what keeps one rule from becoming two: the resolution the
//! rasterizer will paint by is the resolution these findings are about.
//!
//! **What is not here.** Two elements *resolving to the same layer* is ADR-0060's business,
//! not this module's: it is legal while their boxes never meet and an `error` once they do,
//! which needs geometry sampled across keyframes rather than a lookup. That is
//! [`super::tie`], which asks [`crate::stack`] the same question this module does.

use serde_json::json;

use crate::finding::Finding;
use crate::permissive::Loose;
use crate::report::Report;
use crate::stack::{Stack, Unresolved};

/// Every anchor in the document, resolved, and what resolving it found.
pub fn check(document: &Loose, report: &mut Report) {
    let stack = Stack::of(document);

    for placement in stack.placements() {
        let Some((side, target)) = placement.anchor() else {
            continue;
        };

        let at = |finding: Finding| {
            let finding = finding
                .at_file(document.path())
                .at_element(placement.id)
                .field("side", json!(side.as_str()))
                .field("target", json!(target));
            match placement.track {
                Some(track) => finding.at_track(track),
                None => finding,
            }
        };

        match stack.layer_of(placement.id) {
            Err(Unresolved::MissingTarget(_)) => {
                report.push(at(Finding::new("E-ANCHOR-MISSING")).repair_value(json!({
                    "value": "name an element that is in the project, or state this \
                element's own `layer` as an integer"
                })))
            }
            Err(Unresolved::SelfReference(_)) => {
                report.push(at(Finding::new("E-ANCHOR-SELF")).repair_value(json!({
                    "value": "name the element this one should sit against, or state this \
                element's own `layer` as an integer"
                })))
            }
            // Refuse-class, so the `repair` comes from the declaration and nothing here
            // asks for it (ADR-0043).
            Err(Unresolved::ChainedTarget(_)) => report.push(at(Finding::new("E-ANCHOR-CHAIN"))),
            // The target resolves to nothing an integer can be read from, or carries a
            // `layer` that is not one of the format's two forms. Both are schema facts
            // about a document mid-edit, and the check that owns the schema says so — this
            // one would be adding a second voice to a defect it did not find. That check is
            // `crate::checks::schema`, live as of #244, and `tests/schema_check.rs` asserts
            // both conditions against it rather than leaving this silence unwitnessed.
            Err(Unresolved::Unstated | Unresolved::Malformed) => {}
            // `placements()` yields what the index was built from, so there is always an
            // element here. Kept as a branch rather than an `unwrap`, because an
            // unreachable panic in a check would turn a fact about a project into exit 70.
            Err(Unresolved::NoSuchElement) => {}
            Ok(layer) => {
                if let Some(finding) = inert(&stack, placement.id, target, layer) {
                    report.push(at(finding));
                }
            }
        }
    }
}

/// The `review`: it resolved, and the two elements are never on screen together.
///
/// Both timeline ranges have to be known. An element mid-edit carries no integer `start` —
/// and a window that cannot be computed is never reported as a window that is not there,
/// which is ADR-0006's standing rule that a stated number is a measured one.
fn inert(stack: &Stack<'_>, id: &str, target: &str, layer: i64) -> Option<Finding> {
    let range = stack.placement(id)?.range?;
    let target_range = stack.placement(target)?.range?;
    if range.overlaps(target_range) {
        return None;
    }

    Some(
        Finding::new("R-ANCHOR-NO-OVERLAP")
            .field("start", json!(range.start))
            .field("end", json!(range.end))
            .field("target_start", json!(target_range.start))
            .field("target_end", json!(target_range.end))
            // The integer it does resolve to. Stated rather than left implicit,
            // because the finding's whole claim is that this number is correct and
            // inconsequential — a reader who cannot see it has to take both halves on
            // trust.
            .field("layer", json!(layer)),
    )
}
