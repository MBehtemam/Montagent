//! `R-DERIVED-T` — ADR-0086: a keyframe's `t_from` names a rule that re-derives an instant
//! which is not the `t` written beside it.
//!
//! The one instance of that ADR's **recorded intent** pattern: a renderer-ignored
//! declaration whose only consumer is `validate`. Structurally ADR-0015's `fit` transplanted
//! off the raster axis — the author supplies a determinant the format cannot infer, and the
//! check re-runs the author's own arithmetic against the literal they wrote.
//!
//! # Why `validate` and not `compare`
//!
//! Both sides of a declared derivation sit in one document, so the check is stateless, needs
//! no I/O, and belongs to the tool defined as stateless-over-one-document. `compare` is
//! defined by having an input `validate` lacks — the earlier state — and a declared
//! relationship needs none. ADR-0063 stays unreversed by this: it refused to *infer* fixed
//! offsets, and a **declared** offset is neither inferred nor unbounded. Nothing here looks
//! at a pair the author did not annotate.
//!
//! # What it does not detect, deliberately
//!
//! A shift that moves source and derived together preserves the declared relation, so this
//! check stays silent even if the coupling was meant to break. The declaration asserts the
//! relation holds, not that it should have changed. And an absent `t_from` is
//! indistinguishable on disk from a deliberately-unrelated literal — `fit`'s *"an omitted
//! field is indistinguishable from a decision not to check"*, transplanted verbatim and
//! accepted as a permanent hole. **The hole is not patched by inference**: a census reporting
//! *"this value happens to equal the previous one plus N"* is exactly the population
//! explosion ADR-0063 refused.
//!
//! # Array order, not clock order
//!
//! *"Previous"* is positional — the record before this one in the array — which ADR-0082
//! makes the same thing as the record before it on the clock, because a keyframe list must
//! be written in ascending `t`. Reading the array is what keeps this check saying the same
//! thing the deserializer does, and spends none of ADR-0086's per-axis reference permission.
//!
//! # Read permissively, through the format's own reading
//!
//! A declaration is read back through [`Derivation`]'s own deserializer rather than by
//! matching `rule` here: ADR-0086's rule set is *"finite and published"*, and a second parse
//! of it in this module would be the second answer that undoes. It also gives the permissive
//! behaviour for free — anything that reading refuses is a schema fact and
//! [`crate::checks::schema`]'s finding, never a second report of the same byte here. Same for
//! a record with no integer `t` and an element with no integer `start`; and
//! `after-previous` on the first record of a list is refused outright by
//! `crate::model::keyframe`, so a list that reaches here never carries one there.

use serde_json::{Value, json};

use crate::finding::Finding;
use crate::model::Derivation;
use crate::permissive::Loose;
use crate::report::Report;

/// `R-DERIVED-T`, over every `t_from` in the document.
pub fn check(document: &Loose, report: &mut Report) {
    for (track, element) in document.elements_in_tracks() {
        let subject = crate::checks::subject_of(element.get("id").and_then(Value::as_str));
        // Every animated property, not `crate::checks::MOVES_THE_BOX`: a `t` is a `t`
        // whatever it animates, and an `opacity` ramp can carry a declaration as readily as
        // a `scale` one. The fixture's fourteen are all on `scale`, which is a fact about
        // the fixture and not about the field.
        // A stagger's unit lists too, the block's and each run override's (ADR-0151 §5):
        // their `element-start` is still the element's own `start`.
        for (name, container, property) in crate::checks::keyframe_lists(element) {
            for finding in stale(element, container, property) {
                let finding = finding
                    .field("element", json!(subject))
                    .field("property", json!(name))
                    .at_file(document.path())
                    .at_element(&subject);
                report.push(match track {
                    Some(track) => finding.at_track(track),
                    None => finding,
                });
            }
        }
    }
}

/// Every declaration on this property's keyframe list whose rule re-derives some other
/// instant than the one written.
fn stale(element: &Value, container: &Value, property: &str) -> Vec<Finding> {
    let Some(records) = crate::checks::keyframe_records(container, property) else {
        return Vec::new();
    };
    records
        .iter()
        .enumerate()
        .filter_map(|(index, record)| {
            let declared_t = record.get("t")?.as_i64()?;
            let declaration = declaration(record)?;
            let (derived, derivation) = match declaration {
                Derivation::ElementStart => {
                    let start = element.get("start")?.as_i64()?;
                    (start, format!("the element's own `start` is {start}"))
                }
                Derivation::AfterPrevious { ms } => {
                    let ms = i64::try_from(ms).ok()?;
                    let previous = records.get(index.checked_sub(1)?)?.get("t")?.as_i64()?;
                    (
                        previous + ms,
                        format!("the previous record's `t` is {previous} and `ms` is {ms}"),
                    )
                }
            };
            (derived != declared_t).then(|| {
                Finding::new("R-DERIVED-T")
                    .field("declared_t", json!(declared_t))
                    .field("rule", json!(declaration.rule()))
                    .field("derivation", json!(derivation))
                    .field("derived", json!(derived))
                    // Advise-class, whose content is the re-derived integer ADR-0086 asks
                    // the repair to state. The template's second move — dropping the
                    // `t_from` — is ADR-0086's own draft message ("write 32000, or drop
                    // t_from") and stays prose: a `repair` field is the one value the
                    // document determines, and there is exactly one of those here.
                    .repair_value(json!({"t": derived}))
            })
        })
        .collect()
}

/// The declaration this record carries, as the format itself reads one.
///
/// `None` covers all of: no `t_from` at all (*no claim*, ADR-0086), and a `t_from` whose
/// shape, `rule` or `ms` the format does not publish — which is
/// [`crate::checks::schema`]'s finding about the same byte.
fn declaration(record: &Value) -> Option<Derivation> {
    serde_json::from_value(record.get("t_from")?.clone()).ok()
}
