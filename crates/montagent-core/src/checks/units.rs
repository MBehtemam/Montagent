//! A stagger's own checks (ADR-0151 §4–§6, amended by ADR-0153 §2).
//!
//! - **`E-UNIT-RUN-NOT-ONE-UNIT`** — a run carrying `unit` holds anything other than exactly
//!   one unit's graphemes, or its element has no `units` block. The replace rule needs one
//!   unit to replace; part of a unit, two units or a space give it none.
//! - **`E-UNIT-RUN-UNDECLARED`** — a run's `unit` names a list the `units` block does not
//!   declare. Naming a new property would add motion, not replace it.
//! - **`E-UNIT-RUN-MERGED`** — a run's `unit` singles out a unit that moves with a
//!   neighbour, by a join or a shaping merge: its timing cannot be its own.
//! - **`R-UNIT-MERGED`** — the units that move as one body, so the waiting steps a joined
//!   word makes are said rather than discovered in a frame.
//! - **`N-CAPTION-SETTLES`** — when a staggered caption's text is fully readable.
//!
//! The unit count and the joins come from the text alone; the shaping merges from the
//! element's fonts, through the same placement the painter draws.

use serde_json::{Value, json};

use crate::checks::subject_of;
use crate::finding::Finding;
use crate::permissive::Loose;
use crate::report::Report;
use crate::units::{LISTS, Plan, by_name, grouping};

/// Every stagger check, over every `text` element in the document.
pub fn check(document: &Loose, report: &mut Report) {
    for (track, element) in document.elements_in_tracks() {
        if element.get("type").and_then(Value::as_str) != Some("text") {
            continue;
        }
        let subject = subject_of(element.get("id").and_then(Value::as_str));
        let mut push = |finding: Finding| {
            let finding = finding
                .at_file(document.path())
                .at_element(subject.clone())
                .field("element", json!(subject));
            report.push(match track {
                Some(track) => finding.at_track(track),
                None => finding,
            });
        };
        let runs = crate::verbs::measure::runs_array(element);
        let run_text = |r: usize| {
            runs[r]
                .get("text")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string()
        };
        let Some(plan) = Plan::of(element) else {
            // No `units` block: every override is on an element with no units at all.
            // (A block the schema refuses is the schema check's, and is not doubled here.)
            if element.get("units").is_none() {
                for (r, run) in runs.iter().enumerate() {
                    if run.get("unit").is_some_and(Value::is_object) {
                        push(
                            Finding::new("E-UNIT-RUN-NOT-ONE-UNIT")
                                .field("run", json!(r))
                                .field("text", json!(run_text(r)))
                                .field("found", json!(0))
                                .field("no_block", json!(true)),
                        );
                    }
                }
            }
            continue;
        };

        let start = element.get("start").and_then(Value::as_i64).unwrap_or(0);
        let groups = grouping(document, element, &plan, start);

        for r in plan.override_runs().collect::<Vec<_>>() {
            let over = plan.override_on(r).expect("an override run carries one");
            let unit = match plan.singled_out(r) {
                Ok(unit) => unit,
                Err(not_one) => {
                    push(
                        Finding::new("E-UNIT-RUN-NOT-ONE-UNIT")
                            .field("run", json!(r))
                            .field("text", json!(run_text(r)))
                            .field("found", json!(not_one.found))
                            .field("no_block", json!(false))
                            .field("by", json!(by_name(plan.by))),
                    );
                    continue;
                }
            };
            for property in LISTS {
                if over.get(property).is_some() && plan.block().get(property).is_none() {
                    push(
                        Finding::new("E-UNIT-RUN-UNDECLARED")
                            .field("run", json!(r))
                            .field("text", json!(run_text(r)))
                            .field("property", json!(property)),
                    );
                }
            }
            let merged_with = groups.merged_with(unit);
            if !merged_with.is_empty() {
                let body = &groups.bodies[groups.body_of_unit[unit]];
                push(
                    Finding::new("E-UNIT-RUN-MERGED")
                        .field("run", json!(r))
                        .field("text", json!(run_text(r)))
                        .field("unit", json!(unit))
                        .field("merged_with", json!(merged_with))
                        .field("how", json!(how(body.joined, body.merged))),
                );
            }
        }

        let moving: Vec<&montagent_text::units::Body> = groups
            .bodies
            .iter()
            .filter(|body| body.units.len() > 1)
            .collect();
        if !moving.is_empty() {
            let detail: Vec<String> = moving
                .iter()
                .map(|body| {
                    let first = body.units[0];
                    let last = *body.units.last().expect("more than one unit");
                    let text: String = body
                        .units
                        .iter()
                        .map(|&u| plan.units[u].text.as_str())
                        .collect();
                    format!("units {first}–{last} `{text}`")
                })
                .collect();
            let groups_json: Vec<&Vec<usize>> = moving.iter().map(|body| &body.units).collect();
            push(
                Finding::new("R-UNIT-MERGED")
                    .field("by", json!(by_name(plan.by)))
                    .field(
                        "how",
                        json!(how(
                            moving.iter().any(|b| b.joined),
                            moving.iter().any(|b| b.merged)
                        )),
                    )
                    .field("groups", json!(groups_json))
                    .field("detail", json!(detail.join("; "))),
            );
        }

        // ADR-0151 §6: a staggered caption's text is fully readable once every unit list
        // has landed. Only where the caption checks run on the element (ADR-0136).
        if element.get("caption").and_then(Value::as_bool) != Some(false)
            && let (Some(end), Some((_, settles))) =
                (element.get("end").and_then(Value::as_i64), plan.window())
        {
            push(
                Finding::new("N-CAPTION-SETTLES")
                    .field("settles", json!(settles))
                    .field("end", json!(end))
                    .field("never", json!(settles >= end)),
            );
        }
    }
}

/// `joined`, `merged`, or both.
fn how(joined: bool, merged: bool) -> &'static str {
    match (joined, merged) {
        (true, true) => "joined and merged",
        (false, true) => "merged",
        _ => "joined",
    }
}
