//! prototype(#750, ADR-0158 §6–§7): what the schema cannot state about a stroke's join, cap
//! and dash pattern, as errors.
//!
//! - `E-STROKE-NO-STROKE`: a stroke-shaping field on an element with no stroke to shape —
//!   no `stroke`, or a `stroke_width` that is a static 0 (or absent), or keyed with every key 0.
//! - `E-STROKE-MITER-LIMIT`: `"miter"` with no `stroke_miter_limit`, or a limit with any other
//!   join (an omitted join is `"round"`).
//! - `E-STROKE-CAP-UNDRAWN`: `stroke_cap` on a closed path with no `stroke_dash`.
//! - `E-DASH-SHAPE`: an odd number of entries, or a zero total.
//! - `E-DASH-ZERO-BUTT`: a zero-length dash under a butt cap.
//! - `E-DASH-OFFSET-ALONE` (a guess: ADR-0158 §5 makes this an error but §7 names no code):
//!   `stroke_dash_offset` with no `stroke_dash`.

use serde_json::{Value, json};

use crate::finding::Finding;
use crate::permissive::Loose;
use crate::report::Report;

const SHAPING: [&str; 8] = [
    "stroke_join",
    "stroke_miter_limit",
    "stroke_cap",
    "stroke_dash",
    "stroke_dash_offset",
    // prototype(#760, ADR-0160 §7): `E-STROKE-NO-STROKE` covers the trim fields.
    "trim_start",
    "trim_end",
    "trim_offset",
];

/// prototype(#760, ADR-0160 §5, §7): `E-TRIM-EMPTY` and `E-TRIM-OFFSET`.
fn trim(element: &Value, kind: Option<&str>, push: &mut dyn FnMut(Finding)) {
    let start = element.get("trim_start");
    let end = element.get("trim_end");
    let keyed = |v: Option<&Value>| v.is_some_and(Value::is_array);
    if (start.is_some() || end.is_some()) && !keyed(start) && !keyed(end) {
        let s = start.and_then(Value::as_f64);
        let e = end.and_then(Value::as_f64);
        let (sv, ev) = (s.unwrap_or(0.0), e.unwrap_or(1.0));
        // The literal as written, so the replace target is in front of the agent.
        let shown = |v: Option<&Value>, default: &str| {
            v.map_or(format!("{default} (the default)"), Value::to_string)
        };
        if (start.is_none() || s.is_some()) && (end.is_none() || e.is_some()) && sv >= ev {
            let field = if start.is_some() { "trim_start" } else { "trim_end" };
            push(
                Finding::new("E-TRIM-EMPTY")
                    .field("field", json!(field))
                    .field("start", json!(shown(start, "0")))
                    .field("end", json!(shown(end, "1"))),
            );
        }
    }
    if element.get("trim_offset").is_some() {
        let open = kind == Some("path")
            && element.get("closed").and_then(Value::as_bool) != Some(true);
        if open {
            push(
                Finding::new("E-TRIM-OFFSET")
                    .field(
                        "why",
                        json!("the path is open, and an offset rotates the window only around a closed outline — an open line has no round to wrap onto"),
                    )
                    .field("fix", json!(", or set `closed` to true"))
                    .repair_value(json!({"value": "drop `trim_offset`"})),
            );
        }
        if start.is_none() && end.is_none() {
            push(
                Finding::new("E-TRIM-OFFSET")
                    .field(
                        "why",
                        json!("there is no `trim_start` or `trim_end`, so the window is the whole outline and rotating it draws nothing different"),
                    )
                    .field("fix", json!(", or give the window an end with `trim_start` or `trim_end`"))
                    .repair_value(json!({"value": "drop `trim_offset`"})),
            );
        }
    }
}

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

        // §6: no stroke to shape.
        let why = if element.get("stroke").is_none() {
            Some("it has no `stroke`")
        } else if never_drawn(element) {
            Some("its `stroke_width` is 0 on every key, so the stroke never draws")
        } else {
            None
        };
        if let Some(why) = why {
            for field in SHAPING {
                if element.get(field).is_some() {
                    push(
                        Finding::new("E-STROKE-NO-STROKE")
                            .field("field", json!(field))
                            .field("why", json!(why))
                            .repair_value(json!({"value": format!("drop `{field}`")})),
                    );
                }
            }
        }

        let join = element.get("stroke_join").and_then(Value::as_str);
        let limit = element.get("stroke_miter_limit");
        if join == Some("miter") && limit.is_none() {
            push(
                Finding::new("E-STROKE-MITER-LIMIT")
                    .field("field", json!("stroke_join"))
                    .field(
                        "detail",
                        json!("`\"miter\"` needs a `stroke_miter_limit`, an integer from 1 to 10; the box's inset is computed from it"),
                    ),
            );
        }
        if let (Some(_), false) = (limit, join == Some("miter")) {
            let named = join.map_or("no `stroke_join` (so `\"round\"`)".to_string(), |join| {
                format!("`stroke_join` `\"{join}\"`")
            });
            push(
                Finding::new("E-STROKE-MITER-LIMIT")
                    .field("field", json!("stroke_miter_limit"))
                    .field(
                        "detail",
                        json!(format!(
                            "a limit applies only to `\"miter\"`, and this element has {named}; drop the limit or set `\"miter\"`"
                        )),
                    ),
            );
        }

        let closed = element.get("closed").and_then(Value::as_bool);
        let dash = element.get("stroke_dash").and_then(Value::as_array);
        trim(element, kind, &mut push);

        // prototype(#760, ADR-0160 §6): a closed path carrying `trim_start` or `trim_end`
        // draws its cap at the trim's ends.
        let trimmed = element.get("trim_start").is_some() || element.get("trim_end").is_some();
        if kind == Some("path")
            && closed == Some(true)
            && element.get("stroke_cap").is_some()
            && dash.is_none()
            && !trimmed
        {
            push(Finding::new("E-STROKE-CAP-UNDRAWN").field(
                "cap",
                json!(element.get("stroke_cap").cloned().unwrap_or(Value::Null)),
            ));
        }

        if let Some(dash) = dash {
            let lengths: Vec<i64> = dash.iter().filter_map(Value::as_i64).collect();
            if lengths.len() == dash.len() {
                if lengths.len() % 2 == 1 {
                    push(Finding::new("E-DASH-SHAPE").field(
                        "detail",
                        json!(format!(
                            "it has {} entries, an odd number; the format does not repeat an odd list as SVG does, so write the doubled list out",
                            lengths.len()
                        )),
                    ));
                } else if lengths.iter().sum::<i64>() == 0 {
                    push(Finding::new("E-DASH-SHAPE").field(
                        "detail",
                        json!("its entries add up to 0, so there is no pattern to repeat"),
                    ));
                }
                let cap = if kind == Some("path") {
                    element
                        .get("stroke_cap")
                        .and_then(Value::as_str)
                        .unwrap_or("butt")
                } else {
                    "butt"
                };
                if cap == "butt" {
                    for (index, _) in lengths
                        .iter()
                        .enumerate()
                        .filter(|(index, length)| index % 2 == 0 && **length == 0)
                    {
                        let repair = if kind == Some("path") {
                            "Set `stroke_cap` to `\"round\"` for a dot, or `\"square\"` for a square"
                        } else {
                            "A `rect`'s and an `ellipse`'s dash ends are always butt; draw dots with a `path` and `\"stroke_cap\": \"round\"`"
                        };
                        push(
                            Finding::new("E-DASH-ZERO-BUTT")
                                .field("index", json!(index))
                                .field("repair_text", json!(repair)),
                        );
                    }
                }
            }
        } else if element.get("stroke_dash_offset").is_some() {
            push(Finding::new("E-DASH-OFFSET-ALONE").repair_value(json!({"value": "drop `stroke_dash_offset`"})));
        }
    }
}

/// Whether `stroke_width` never draws: absent, a static 0, or keyed with every key 0.
fn never_drawn(element: &Value) -> bool {
    match element.get("stroke_width") {
        None => true,
        Some(Value::Number(n)) => n.as_f64() == Some(0.0),
        Some(Value::Array(records)) => records
            .iter()
            .all(|record| record.get("v").and_then(Value::as_f64) == Some(0.0)),
        Some(_) => false,
    }
}
