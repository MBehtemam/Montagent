//! `wipe`, `slide` and `push` beside `crossfade`, with `direction` and `ease` (#690,
//! ADR-0150).
//!
//! The seams are the verbs: `validate`'s report, `frame`'s picture and answer, `render`'s
//! frames at the encoder's input, `query --at`'s answer, and the files `shift` writes.

use montagent_core::report::Report;
use montagent_core::validate;
use serde_json::{Value, json};

mod common;
use common::{canonical, write_project};

/// A 400×200 project at 25 fps: `a` (red) on layer 0 over 0..2000, `b` (blue) on layer 1
/// over 1000..3000, and whatever transition a test bridges them with on layer 2.
fn bridged(transition: Value) -> String {
    bridged_with(
        json!({"id": "a", "type": "rect", "start": 0, "end": 2000, "x": 0, "y": 0,
               "origin": "top-left", "width": 400, "height": 200, "fill": "#FF0000"}),
        json!({"id": "b", "type": "rect", "start": 1000, "end": 3000, "x": 0, "y": 0,
               "origin": "top-left", "width": 400, "height": 200, "fill": "#0000FF"}),
        transition,
    )
}

fn bridged_with(a: Value, b: Value, transition: Value) -> String {
    canonical(
        &json!({
            "frame": {"width": 400, "height": 200}, "fps": 25, "background": "#00FF00",
            "duration": 3000, "output": "out/t.mp4",
            "tracks": [
                {"name": "first", "layer": 0, "elements": [a]},
                {"name": "second", "layer": 1, "elements": [b]},
                {"name": "bridge", "layer": 2, "elements": [transition]},
            ],
        })
        .to_string(),
    )
}

/// A transition element over the window `a` and `b` share, plus `extra` fields.
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

fn codes(report: &Report) -> Vec<&str> {
    report.findings.iter().map(|f| f.code.as_str()).collect()
}

fn schema_faults(report: &Report) -> Vec<String> {
    report
        .findings
        .iter()
        .filter(|f| f.code.starts_with("E-SCHEMA"))
        .map(|f| serde_json::to_string(&f.fields).unwrap_or_default())
        .collect()
}

const KINDS: [&str; 3] = ["wipe", "slide", "push"];
const DIRECTIONS: [&str; 4] = ["left", "right", "up", "down"];

// ---------------------------------------------------------------------------
// The schema
// ---------------------------------------------------------------------------

#[test]
fn each_geometric_kind_and_direction_validates_clean() {
    for kind in KINDS {
        for direction in DIRECTIONS {
            let report = validated(&bridged(transition(kind, json!({"direction": direction}))));
            assert!(
                report.findings.iter().all(|f| f.class != montagent_core::finding::Class::Error),
                "{kind} {direction}: {:?}",
                codes(&report)
            );
        }
    }
}

#[test]
fn an_ease_is_a_keyframe_ease_name_or_bezier() {
    for ease in [json!("ease-in-out"), json!("step"), json!([0.2, 0.0, 0.2, 1.4])] {
        let report = validated(&bridged(transition(
            "push",
            json!({"direction": "left", "ease": ease}),
        )));
        assert!(schema_faults(&report).is_empty(), "{ease}: {:?}", codes(&report));
    }
}

#[test]
fn direction_or_ease_on_a_crossfade_is_a_schema_error() {
    for extra in [json!({"direction": "left"}), json!({"ease": "ease-in"})] {
        let report = validated(&bridged(transition("crossfade", extra.clone())));
        assert_eq!(schema_faults(&report).len(), 1, "{extra}: {:?}", codes(&report));
    }
    // And the plain crossfade is still clean.
    let report = validated(&bridged(transition("crossfade", json!({}))));
    assert!(schema_faults(&report).is_empty(), "{:?}", codes(&report));
}

#[test]
fn a_missing_direction_on_wipe_slide_or_push_is_a_schema_error() {
    for kind in KINDS {
        let report = validated(&bridged(transition(kind, json!({}))));
        let faults = schema_faults(&report);
        assert_eq!(faults.len(), 1, "{kind}: {:?}", codes(&report));
        assert!(faults[0].contains("direction"), "{kind}: {faults:?}");
    }
}

#[test]
fn an_unknown_kind_direction_or_ease_is_a_schema_error() {
    for element in [
        transition("iris", json!({"direction": "left"})),
        transition("push", json!({"direction": "west"})),
        transition("push", json!({"direction": "left", "ease": "spring"})),
        transition("push", json!({"direction": "left", "ease": [1.5, 0.0, 0.2, 1.0]})),
    ] {
        let report = validated(&bridged(element.clone()));
        assert_eq!(schema_faults(&report).len(), 1, "{element}: {:?}", codes(&report));
    }
}

#[test]
fn the_published_schema_says_the_same_rules() {
    let schema = montagent_core::schema::generate();
    let transition = schema["$defs"]["Element"]["oneOf"]
        .as_array()
        .expect("the element union")
        .iter()
        .find(|branch| branch["properties"]["type"]["const"] == "transition")
        .expect("a transition branch")
        .clone();
    let kinds = schema["$defs"]["TransitionKind"].to_string();
    for kind in ["crossfade", "wipe", "slide", "push"] {
        assert!(kinds.contains(kind), "{kind} missing from {kinds}");
    }
    let published = transition.to_string();
    assert!(published.contains("direction"), "{published}");
    assert!(published.contains("ease"), "{published}");
    // The conditional: required off `crossfade`, refused on it.
    assert!(
        published.contains("\"if\"") && published.contains("\"else\""),
        "{published}"
    );
}

// ---------------------------------------------------------------------------
// `validate`'s three new checks
// ---------------------------------------------------------------------------

fn findings_of<'r>(report: &'r Report, code: &str) -> Vec<&'r montagent_core::finding::Finding> {
    report.findings.iter().filter(|f| f.code == code).collect()
}

#[test]
fn a_from_or_to_naming_no_element_is_e_transition_ref_missing() {
    for (from, to, missing) in [("nobody", "b", "nobody"), ("a", "ghost", "ghost")] {
        let mut element = transition("crossfade", json!({}));
        element["from"] = json!(from);
        element["to"] = json!(to);
        let report = validated(&bridged(element));
        let found = findings_of(&report, "E-TRANSITION-REF-MISSING");
        assert_eq!(found.len(), 1, "{from}->{to}: {:?}", codes(&report));
        assert_eq!(found[0].class, montagent_core::finding::Class::Error);
        assert_eq!(found[0].fields["target"], missing);
        assert!(found[0].repair.is_some(), "a repair: {:?}", found[0]);
    }
    let clean = validated(&bridged(transition("push", json!({"direction": "up"}))));
    assert!(findings_of(&clean, "E-TRANSITION-REF-MISSING").is_empty());
}

#[test]
fn a_transition_naming_an_audio_element_names_no_visual_element() {
    let audio = json!({"id": "a", "type": "audio", "start": 0, "end": 2000,
                       "source": "music.wav", "source_start": 0, "source_end": 2000});
    let b = json!({"id": "b", "type": "rect", "start": 1000, "end": 3000, "x": 0, "y": 0,
                   "origin": "top-left", "width": 400, "height": 200, "fill": "#0000FF"});
    let report = validated(&bridged_with(audio, b, transition("crossfade", json!({}))));
    assert_eq!(
        findings_of(&report, "E-TRANSITION-REF-MISSING").len(),
        1,
        "{:?}",
        codes(&report)
    );
}

#[test]
fn a_from_equal_to_to_is_e_transition_ref_self() {
    let mut element = transition("wipe", json!({"direction": "left"}));
    element["to"] = json!("a");
    let report = validated(&bridged(element));
    let found = findings_of(&report, "E-TRANSITION-REF-SELF");
    assert_eq!(found.len(), 1, "{:?}", codes(&report));
    assert!(findings_of(&report, "E-TRANSITION-REF-MISSING").is_empty());
    let clean = validated(&bridged(transition("wipe", json!({"direction": "left"}))));
    assert!(findings_of(&clean, "E-TRANSITION-REF-SELF").is_empty());
}

/// `b` on layer 0 and `a` on layer 1, so the incoming `b` paints beneath the outgoing `a`.
fn swapped_layers(transition: Value) -> String {
    let body = bridged(transition);
    let mut value: Value = serde_json::from_str(&body).expect("json");
    value["tracks"][0]["layer"] = json!(1);
    value["tracks"][1]["layer"] = json!(0);
    canonical(&value.to_string())
}

#[test]
fn a_slide_whose_to_paints_beneath_its_from_is_e_transition_slide_under() {
    let report = validated(&swapped_layers(transition("slide", json!({"direction": "left"}))));
    let found = findings_of(&report, "E-TRANSITION-SLIDE-UNDER");
    assert_eq!(found.len(), 1, "{:?}", codes(&report));
    let finding = found[0];
    assert_eq!(finding.fields["from"], "a");
    assert_eq!(finding.fields["to"], "b");
    assert_eq!(finding.fields["from_layer"], 1);
    assert_eq!(finding.fields["to_layer"], 0);
    assert!(
        matches!(finding.repair, Some(montagent_core::finding::Repair::None)),
        "refuse-class: {:?}",
        finding.repair
    );
    let rendered =
        montagent_core::wire::render(&report, montagent_core::Wire::Text { verbose: false });
    assert!(rendered.contains("must paint above"), "{rendered}");
}

#[test]
fn a_slide_in_order_and_a_push_or_wipe_in_either_order_stay_clean() {
    let clean = validated(&bridged(transition("slide", json!({"direction": "left"}))));
    assert!(findings_of(&clean, "E-TRANSITION-SLIDE-UNDER").is_empty());
    for kind in ["push", "wipe", "crossfade"] {
        let extra = match kind {
            "crossfade" => json!({}),
            _ => json!({"direction": "down"}),
        };
        let report = validated(&swapped_layers(transition(kind, extra)));
        assert!(
            findings_of(&report, "E-TRANSITION-SLIDE-UNDER").is_empty(),
            "{kind}: {:?}",
            codes(&report)
        );
    }
}
