//! Project-level marker validation (ADR-0230).
//!
//! Negative `at`, negative `duration`, unknown `kind`, and unknown fields on project and
//! element markers are caught by the closed schema check (`E-SCHEMA` / `E-SCHEMA-UNKNOWN-KEY`).
//! This check enforces the cross-field invariant on root markers: when `project.duration` is
//! declared, every project-level marker must have `at <= project.duration`.

use serde_json::{Value, json};

use crate::finding::Finding;
use crate::permissive::Loose;
use crate::report::Report;

pub fn check(document: &Loose, report: &mut Report) {
    let root = document.value();
    let Some(duration) = root.get("duration").and_then(Value::as_i64) else {
        return;
    };
    if duration < 0 {
        return;
    }
    let Some(markers) = root.get("markers").and_then(Value::as_array) else {
        return;
    };
    for (index, marker) in markers.iter().enumerate() {
        let Some(at) = marker.get("at").and_then(Value::as_i64) else {
            continue;
        };
        if at > duration {
            report.push(
                Finding::new("E-MARKER-PAST-DURATION")
                    .at_file(document.path())
                    .field("index", json!(index))
                    .field("at", json!(at))
                    .field("duration", json!(duration)),
            );
        }
    }
}
