//! A transition carries the audio across its window (ADR-0176): the `audio` field and the
//! `audio_crossfade` kind, in the schema (S1).

use montagent_core::report::Report;
use montagent_core::validate;
use montagent_core::verbs::fmt::{self, Mode};
use serde_json::{Value, json};

mod common;
use common::{canonical, write_project};

/// `a` over 0..2000 and `b` over 1000..3000 on two rects, bridged by `transition` over
/// the 1000..2000 window they share.
fn bridged(transition: Value) -> String {
    canonical(
        &json!({
            "frame": {"width": 400, "height": 200}, "fps": 25, "duration": 3000,
            "output": "out/t.mp4",
            "tracks": [
                {"name": "first", "layer": 0, "elements": [
                    {"id": "a", "type": "rect", "start": 0, "end": 2000, "x": 0, "y": 0,
                     "origin": "top-left", "width": 400, "height": 200, "fill": "#FF0000"}]},
                {"name": "second", "layer": 1, "elements": [
                    {"id": "b", "type": "rect", "start": 1000, "end": 3000, "x": 0, "y": 0,
                     "origin": "top-left", "width": 400, "height": 200, "fill": "#0000FF"}]},
                {"name": "bridge", "layer": 2, "elements": [transition]},
            ],
        })
        .to_string(),
    )
}

fn transition(kind: &str, extra: Value) -> Value {
    let mut element = json!({"id": "t", "type": "transition", "start": 1000, "end": 2000,
                             "kind": kind, "from": "a", "to": "b"});
    for (key, value) in extra.as_object().expect("an object").clone() {
        element[key] = value;
    }
    element
}

#[track_caller]
fn validated(body: &str) -> Report {
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(&dir, "p.montagent.json", body);
    validate(&path)
}

fn schema_faults(report: &Report) -> Vec<String> {
    report
        .findings
        .iter()
        .filter(|f| f.code.starts_with("E-SCHEMA"))
        .map(|f| serde_json::to_string(&f.fields).unwrap_or_default())
        .collect()
}

#[test]
fn every_new_spelling_parses_and_fmt_leaves_it_alone() {
    let mut elements = vec![transition("audio_crossfade", json!({"audio": "constant_power"}))];
    elements.push(transition("audio_crossfade", json!({"audio": "constant_gain"})));
    for kind in ["crossfade", "wipe", "slide", "push"] {
        for audio in ["cut", "constant_power", "constant_gain"] {
            let mut extra = json!({"audio": audio});
            if kind != "crossfade" {
                extra["direction"] = json!("left");
            }
            elements.push(transition(kind, extra));
        }
    }
    for element in elements {
        let body = bridged(element.clone());
        let report = validated(&body);
        assert!(schema_faults(&report).is_empty(), "{element}: {:?}", report.findings);
        let dir = common::tempdir(line!());
        let path = write_project(&dir, "p.montagent.json", &body);
        let report = fmt::fmt(&path, Mode::Check);
        assert!(report.findings.is_empty(), "{element}: {:?}", report.findings);
    }
}

#[test]
fn the_refused_spellings_each_name_the_kinds_rule() {
    for (element, names) in [
        (transition("audio_crossfade", json!({})), "audio"),
        (
            transition("audio_crossfade", json!({"audio": "cut"})),
            "cut",
        ),
        (
            transition(
                "audio_crossfade",
                json!({"audio": "constant_power", "direction": "left"}),
            ),
            "direction",
        ),
        (
            transition(
                "audio_crossfade",
                json!({"audio": "constant_power", "ease": "ease-in"}),
            ),
            "ease",
        ),
        (
            transition("crossfade", json!({"audio": "exponential"})),
            "exponential",
        ),
    ] {
        let faults = schema_faults(&validated(&bridged(element.clone())));
        assert_eq!(faults.len(), 1, "{element}: {faults:?}");
        assert!(faults[0].contains(names), "{element}: {faults:?}");
    }
}

#[test]
fn the_published_schema_says_the_audio_rules() {
    let schema = montagent_core::schema::generate();
    let published = schema["$defs"]["Element"]["oneOf"]
        .as_array()
        .expect("the element union")
        .iter()
        .find(|branch| branch["properties"]["type"]["const"] == "transition")
        .expect("a transition branch")
        .to_string();
    for needle in ["audio_crossfade", "constant_power", "constant_gain", "ADR-0176"] {
        assert!(published.contains(needle), "{needle} missing from {published}");
    }
}
