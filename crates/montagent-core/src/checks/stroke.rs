//! What the schema cannot state about a stroke's shaping fields (ADR-0158 §5–§7, ADR-0160
//! §5–§7), as eight errors, each named at the field it is about.
//!
//! - `E-STROKE-NO-STROKE`: a `stroke_join`, `stroke_miter_limit`, `stroke_cap`,
//!   `stroke_dash`, `stroke_dash_offset`, `trim_start`, `trim_end` or `trim_offset` with no
//!   stroke to shape: no `stroke`, or a `stroke_width` that is absent, a static 0, or keyed
//!   with every key 0. A keyed width that only passes through 0 shapes the frames where it is
//!   above 0, and is fine.
//! - `E-STROKE-MITER-LIMIT`: `"miter"` with no `stroke_miter_limit`, or a limit with any
//!   other join, where it would sit with no effect.
//! - `E-STROKE-CAP-UNDRAWN`: a `stroke_cap`, even `"butt"`, where no cap draws: on a closed
//!   path with no `stroke_dash`, `trim_start` or `trim_end`, which has no ends
//!   ([`crate::stroke::caps_draw`]).
//! - `E-DASH-SHAPE`: a `stroke_dash` with an odd number of entries, which the format does
//!   not double as SVG does, or whose entries add up to 0.
//! - `E-DASH-ZERO-BUTT`: a zero-length dash — an entry at an even index — under a butt cap,
//!   where it draws nothing. A `rect`'s and an `ellipse`'s dash ends are always butt.
//! - `E-DASH-OFFSET-ALONE`: a `stroke_dash_offset` with no `stroke_dash` to move.
//! - `E-TRIM-EMPTY`: a static trim window that never draws: neither `trim_start` nor
//!   `trim_end` keyed, and start ≥ end with an absent field read at its default. A keyed
//!   window that crosses mid-animation is defined, never checked (ADR-0160 §5).
//! - `E-TRIM-OFFSET`: a `trim_offset` on an open path, which has no round to wrap onto, or
//!   with neither `trim_start` nor `trim_end`, so no window to rotate (ADR-0160 §4).
//!
//! The join, limit and cap are `path`'s alone; the dash and trim fields are `path`'s,
//! `rect`'s and `ellipse`'s (ADR-0158 §1, ADR-0160 §1). The schema refuses each on any
//! other type.

use serde_json::{Value, json};

use crate::finding::Finding;
use crate::model::is_keyframe_list;
use crate::permissive::Loose;
use crate::report::Report;

/// The stroke-shaping fields, in the canonical key order.
const SHAPING: [&str; 8] = [
    "stroke_join",
    "stroke_miter_limit",
    "stroke_cap",
    "stroke_dash",
    "stroke_dash_offset",
    "trim_start",
    "trim_end",
    "trim_offset",
];

pub fn check(document: &Loose, report: &mut Report) {
    for (track, element) in document.elements_in_tracks() {
        let kind = element.get("type").and_then(Value::as_str);
        if !matches!(kind, Some("path" | "rect" | "ellipse")) {
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
            for field in SHAPING
                .iter()
                .filter(|field| element.get(**field).is_some())
            {
                push(
                    Finding::new("E-STROKE-NO-STROKE")
                        .field("field", json!(field))
                        .field("zero", json!(zero))
                        .repair_value(json!({"value": format!("drop `{field}`")})),
                );
            }
        }
        // A join, limit or cap on a `rect` or `ellipse` is the schema's to refuse.
        if kind == Some("path") {
            miter_limit(element).into_iter().for_each(&mut push);
        }
        if kind == Some("path")
            && let Some(cap) = element.get("stroke_cap").and_then(Value::as_str)
            && !crate::stroke::caps_draw(element)
            && element.get("closed").and_then(Value::as_bool) == Some(true)
        {
            push(Finding::new("E-STROKE-CAP-UNDRAWN").field("cap", json!(cap)));
        }
        dash(element, kind.unwrap_or_default())
            .into_iter()
            .for_each(&mut push);
        trim(element).into_iter().for_each(&mut push);
    }
}

/// `E-TRIM-EMPTY` and `E-TRIM-OFFSET` on one element (ADR-0160 §5, §7). A value the schema
/// refuses reads as unknown here, and decides nothing.
fn trim(element: &Value) -> Vec<Finding> {
    let mut out = Vec::new();
    let start = element.get("trim_start");
    let end = element.get("trim_end");
    // §5: neither keyed, and an absent field read at its default. The message quotes each
    // value as written, so the replace target is in front of the agent.
    let literal = |written: Option<&Value>, default: f64| match written {
        None => Some((default, format!("{default} (the default)"))),
        Some(value) if is_keyframe_list(value) => None,
        Some(value) => value.as_f64().map(|number| (number, value.to_string())),
    };
    if (start.is_some() || end.is_some())
        && let (Some((from, from_text)), Some((to, to_text))) =
            (literal(start, 0.0), literal(end, 1.0))
        && from >= to
    {
        let field = match start {
            Some(_) => "trim_start",
            None => "trim_end",
        };
        out.push(
            Finding::new("E-TRIM-EMPTY")
                .field("field", json!(field))
                .field("start", json!(from_text))
                .field("end", json!(to_text)),
        );
    }
    if element.get("trim_offset").is_some() {
        let open = element.get("type").and_then(Value::as_str) == Some("path")
            && element.get("closed").and_then(Value::as_bool) == Some(false);
        let bare = !crate::stroke::trimmed(element);
        if open || bare {
            out.push(
                Finding::new("E-TRIM-OFFSET")
                    .field("open", json!(open))
                    .field("bare", json!(bare))
                    .field("both", json!(open && bare))
                    .repair_value(json!({"value": "drop `trim_offset`"})),
            );
        }
    }
    out
}

/// `E-DASH-SHAPE`, `E-DASH-ZERO-BUTT` and `E-DASH-OFFSET-ALONE` on one element. A list the
/// schema refuses — not 2 to 16 non-negative integers — is the schema's to report, and reads
/// as absent here.
fn dash(element: &Value, kind: &str) -> Vec<Finding> {
    let mut out = Vec::new();
    let Some(pattern) = crate::stroke::dash(element) else {
        if element.get("stroke_dash").is_none() && element.get("stroke_dash_offset").is_some() {
            out.push(
                Finding::new("E-DASH-OFFSET-ALONE")
                    .repair_value(json!({"value": "drop `stroke_dash_offset`"})),
            );
        }
        return out;
    };
    let odd = pattern.len() % 2 == 1;
    if odd || pattern.iter().sum::<i64>() == 0 {
        out.push(
            Finding::new("E-DASH-SHAPE")
                .field("odd", json!(odd))
                .field("count", json!(pattern.len())),
        );
    }
    let shape = kind != "path";
    if shape || crate::stroke::cap(element) == crate::stroke::Cap::Butt {
        for (index, _) in pattern
            .iter()
            .enumerate()
            .filter(|(index, length)| index % 2 == 0 && **length == 0)
        {
            out.push(
                Finding::new("E-DASH-ZERO-BUTT")
                    .field("index", json!(index))
                    .field("shape", json!(shape))
                    .field("kind", json!(kind)),
            );
        }
    }
    out
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
