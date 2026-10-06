//! The document half of ADR-0157's checks on a `video` carrying `source_time`.
//!
//! - **`E-REMAP-FIELD`** (`error`, advise): `source_start`, `source_end`, `speed` or
//!   `overrun` beside the curve, one finding per field. The curve is the only author of the
//!   source (§2), so each is a second statement of it that one key edit could leave stale.
//! - **`E-REMAP-AUDIBLE`** (`error`, advise): a `volume` that is not the literal `0` — any
//!   other number, any keyframe list (all zeros included), or none, since the default is `1`
//!   (§4). One later edit to a list of zeros would bring the desync back unchecked.
//! - **`R-REMAP-HELD-END`** (`review`): a keyed curve whose element holds a painted frame
//!   instant before its first key, or after its last (§5), each end on its own. A literal
//!   never fires it: a literal is a freeze frame on purpose.
//!
//! The disk half, the remap arm of `E-SOURCE-OVERRUN`, needs the probe and is in
//! [`crate::checks::source`]. ADR-0020's two invariant checks skip a remapped element
//! ([`crate::checks::speed`]): the curve replaces the rule they evaluate.

use serde_json::{Value, json};

use crate::finding::Finding;
use crate::permissive::Loose;
use crate::remap;
use crate::report::Report;

pub fn check(document: &Loose, report: &mut Report) {
    let fps = document
        .value()
        .get("fps")
        .and_then(Value::as_i64)
        .filter(|fps| *fps > 0);
    for (track, element) in document.elements_in_tracks() {
        if !remap::is_remapped(element) {
            continue;
        }
        let subject = crate::checks::subject_of(element.get("id").and_then(Value::as_str));
        let mut findings = refused_fields(element, &subject);
        findings.extend(audible(element, &subject));
        if let Some(fps) = fps {
            findings.extend(held_ends(element, &subject, fps));
        }
        for finding in findings {
            let finding = finding.at_file(document.path()).at_element(&subject);
            report.push(match track {
                Some(track) => finding.at_track(track),
                None => finding,
            });
        }
    }
}

/// `E-REMAP-FIELD`, once per refused field the element writes, in [`remap::REFUSED`]'s order.
fn refused_fields(element: &Value, subject: &str) -> Vec<Finding> {
    remap::REFUSED
        .iter()
        .filter(|field| element.get(**field).is_some())
        .map(|field| {
            Finding::new("E-REMAP-FIELD")
                .field("element", json!(subject))
                .field("field", json!(field))
                .repair_value(json!({"value": format!("remove `{field}`")}))
        })
        .collect()
}

/// `E-REMAP-AUDIBLE`, where `volume` is anything but the literal `0`.
fn audible(element: &Value, subject: &str) -> Option<Finding> {
    let volume = match element.get("volume") {
        Some(Value::Number(level)) if level.as_f64() == Some(0.0) => return None,
        Some(Value::Number(level)) => json!(level),
        Some(Value::Array(_)) => json!("a keyframe list"),
        Some(other) => json!(other),
        None => json!("absent, so `1`"),
    };
    Some(
        Finding::new("E-REMAP-AUDIBLE")
            .field("element", json!(subject))
            .field("volume", volume)
            .repair_value(json!({
                "value": "set `volume` to 0, and put the sound on a separate `audio` element"
            })),
    )
}

/// `R-REMAP-HELD-END`: the painted frame instants of `[start, end)` lying before the first
/// key or after the last, each end its own finding, with the length of the held stretch.
fn held_ends(element: &Value, subject: &str, fps: i64) -> Vec<Finding> {
    let Some(records) = crate::animatable::records(element, remap::FIELD) else {
        return Vec::new();
    };
    let key_t = |record: Option<&Value>| record.and_then(|r| r.get("t")).and_then(Value::as_i64);
    let (Some(first), Some(last)) = (key_t(records.first()), key_t(records.last())) else {
        return Vec::new();
    };
    let (Some(start), Some(end)) = (
        element.get("start").and_then(Value::as_i64),
        element.get("end").and_then(Value::as_i64),
    ) else {
        return Vec::new();
    };
    let painted: Vec<i64> = remap::painted_instants(start, end, fps).collect();
    let mut findings = Vec::new();
    for (which, from, to, holds) in [
        (
            "before the first key",
            start,
            first.min(end),
            painted.iter().any(|instant| *instant < first),
        ),
        (
            "after the last key",
            last.max(start),
            end,
            painted.iter().any(|instant| *instant > last),
        ),
    ] {
        if holds {
            findings.push(
                Finding::new("R-REMAP-HELD-END")
                    .field("element", json!(subject))
                    .field("end", json!(which))
                    .field("held_ms", json!(to - from))
                    .field("from", json!(from))
                    .field("to", json!(to)),
            );
        }
    }
    findings
}
