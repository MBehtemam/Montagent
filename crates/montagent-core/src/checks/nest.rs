//! ADR-0201: Bounded nested sequences validation.
//!
//! Validates:
//! 1. Temporal bounds: all child elements are contained within parent nest start..end (`E-NEST-TEMPORAL-BOUNDS`).
//! 2. Maximum nesting depth <= 4 (`E-NEST-DEPTH`).
//! 3. DAG cycle prevention (`E-NEST-CIRCULAR`).
//! 4. Document-wide unique element IDs (`E-ID-DUPLICATE`).

use serde_json::{Value, json};
use std::collections::BTreeMap;

use crate::finding::Finding;
use crate::permissive::Loose;
use crate::report::Report;

pub fn check(document: &Loose, report: &mut Report) {
    let mut seen_ids = BTreeMap::<String, Option<String>>::new();

    for (track, element) in document.elements_in_tracks() {
        let mut ancestors = Vec::new();
        check_element(
            document,
            track,
            element,
            1,
            &mut ancestors,
            &mut seen_ids,
            report,
        );
    }
}

fn check_element<'a>(
    document: &Loose,
    track: Option<&str>,
    element: &'a Value,
    depth: usize,
    ancestors: &mut Vec<&'a str>,
    seen_ids: &mut BTreeMap<String, Option<String>>,
    report: &mut Report,
) {
    let id = element.get("id").and_then(Value::as_str);
    let kind = element.get("type").and_then(Value::as_str);

    // ID Uniqueness across the whole document (ADR-0019, ADR-0201 §4).
    if let Some(id_str) = id {
        if seen_ids.contains_key(id_str) {
            let mut finding = Finding::new("E-ID-DUPLICATE")
                .at_file(document.path())
                .field("id", json!(id_str));
            if let Some(t) = track {
                finding = finding.at_track(t);
            }
            finding = finding.at_element(id_str);
            report.push(finding);
        } else {
            seen_ids.insert(id_str.to_string(), track.map(String::from));
        }
    }

    if kind != Some("nest") {
        return;
    }

    let nest_id = id.unwrap_or("nest");

    // Check circular references (DAG cycle prevention)
    if let Some(id_str) = id
        && let Some(idx) = ancestors.iter().position(|&anc| anc == id_str)
    {
        let cycle = format!("{} -> {}", ancestors[idx..].join(" -> "), id_str);
        let mut finding = Finding::new("E-NEST-CIRCULAR")
            .at_file(document.path())
            .field("cycle", json!(cycle));
        if let Some(t) = track {
            finding = finding.at_track(t);
        }
        finding = finding.at_element(nest_id);
        report.push(finding);
        return; // Stop traversal on cycle
    }

    // Check depth (ADR-0201 §4)
    if depth > 4 {
        let mut finding = Finding::new("E-NEST-DEPTH")
            .at_file(document.path())
            .field("depth", json!(depth));
        if let Some(t) = track {
            finding = finding.at_track(t);
        }
        finding = finding.at_element(nest_id);
        report.push(finding);
    }

    let nest_start = element.get("start").and_then(Value::as_i64);
    let nest_end = element.get("end").and_then(Value::as_i64);

    let Some(elements) = element.get("elements").and_then(Value::as_array) else {
        return;
    };

    if let Some(id_str) = id {
        ancestors.push(id_str);
    }

    for child in elements {
        let child_id = child.get("id").and_then(Value::as_str).unwrap_or("element");
        let child_start = child.get("start").and_then(Value::as_i64);
        let child_end = child.get("end").and_then(Value::as_i64);

        // Temporal bounds check (ADR-0201 §2)
        if let (Some(ns), Some(ne), Some(cs), Some(ce)) =
            (nest_start, nest_end, child_start, child_end)
            && (cs < ns || ce > ne)
        {
            let mut finding = Finding::new("E-NEST-TEMPORAL-BOUNDS")
                .at_file(document.path())
                .field("child", json!(child_id))
                .field("child_start", json!(cs))
                .field("child_end", json!(ce))
                .field("nest_start", json!(ns))
                .field("nest_end", json!(ne));
            if let Some(t) = track {
                finding = finding.at_track(t);
            }
            finding = finding.at_element(nest_id);
            report.push(finding);
        }

        // Recurse into child
        check_element(
            document,
            track,
            child,
            depth + 1,
            ancestors,
            seen_ids,
            report,
        );
    }

    if id.is_some() {
        ancestors.pop();
    }
}
