//! What the schema cannot state about a stroke's shaping fields (ADR-0158 §6–§7), as three
//! errors, each named at the field it is about.
//!
//! - `E-STROKE-NO-STROKE`: a `stroke_join`, `stroke_miter_limit` or `stroke_cap` with no
//!   stroke to shape: no `stroke`, or a `stroke_width` that is absent, a static 0, or keyed
//!   with every key 0. A keyed width that only passes through 0 shapes the frames where it
//!   is above 0, and is fine.
//! - `E-STROKE-MITER-LIMIT`: `"miter"` with no `stroke_miter_limit`, or a limit with any
//!   other join, where it would sit with no effect.
//! - `E-STROKE-CAP-UNDRAWN`: a `stroke_cap`, even `"butt"`, where no cap draws: on a closed
//!   path, which has no ends ([`crate::stroke::caps_draw`]).
//!
//! The fields are `path`'s alone in this slice (ADR-0158 §1). The dash fields of the next
//! slice join [`SHAPING`] and the `rect` and `ellipse` walk, and a dash makes a cap draw on
//! a closed path, which is `caps_draw`'s to say.

use serde_json::{Value, json};

use crate::finding::Finding;
use crate::model::is_keyframe_list;
use crate::permissive::Loose;
use crate::report::Report;

/// The stroke-shaping fields, in the canonical key order.
const SHAPING: [&str; 3] = ["stroke_join", "stroke_miter_limit", "stroke_cap"];

pub fn check(document: &Loose, report: &mut Report) {
    for (track, element) in document.elements_in_tracks() {
        if element.get("type").and_then(Value::as_str) != Some("path") {
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
        if let Some(zero) = unstroked(element) {
            for field in SHAPING.iter().filter(|field| element.get(**field).is_some()) {
                push(
                    Finding::new("E-STROKE-NO-STROKE")
                        .field("field", json!(field))
                        .field("zero", json!(zero))
                        .repair_value(json!({"value": format!("drop `{field}`")})),
                );
            }
        }
        miter_limit(element).into_iter().for_each(&mut push);
        if let Some(cap) = element.get("stroke_cap").and_then(Value::as_str)
            && !crate::stroke::caps_draw(element)
            && element.get("closed").and_then(Value::as_bool) == Some(true)
        {
            push(Finding::new("E-STROKE-CAP-UNDRAWN").field("cap", json!(cap)));
        }
    }
}

/// Why the element's stroke never draws, where it never does: `Some(false)` with no
/// `stroke`, `Some(true)` where `stroke_width` is absent, a static 0 or keyed with every key
/// 0, and `None` where it draws on some frame.
fn unstroked(element: &Value) -> Option<bool> {
    if element.get("stroke").is_none() {
        return Some(false);
    }
    let zero = match element.get("stroke_width") {
        None => true,
        Some(width) if is_keyframe_list(width) => width
            .as_array()
            .into_iter()
            .flatten()
            .all(|record| record.get("v").and_then(Value::as_i64) == Some(0)),
        Some(width) => width.as_i64() == Some(0),
    };
    zero.then_some(true)
}

fn miter_limit(element: &Value) -> Option<Finding> {
    let join = element.get("stroke_join").and_then(Value::as_str);
    let limit = element.get("stroke_miter_limit");
    match (join, limit) {
        (Some("miter"), None) => Some(
            Finding::new("E-STROKE-MITER-LIMIT")
                .field("field", json!("stroke_join"))
                .field("missing", json!(true)),
        ),
        (Some("miter"), Some(_)) => None,
        (join, Some(_)) => Some(
            Finding::new("E-STROKE-MITER-LIMIT")
                .field("field", json!("stroke_miter_limit"))
                .field("missing", json!(false))
                .field("join", json!(join.unwrap_or("round"))),
        ),
        (_, None) => None,
    }
}
