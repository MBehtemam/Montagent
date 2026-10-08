//! The `audio_effects` list, F0 of the audio-effects map (ADR-0169; #795, #845): the field on
//! `audio` and `video`, the union that has no members yet, and the `validate` rule that keeps
//! the two vocabularies in their own lists.
//!
//! No member exists in this slice, so everything that needs one (the bypass review, the
//! singular error, the render stage) is tested against an injected vocabulary beside the code
//! (`checks::audio_effects`, `verbs::render`). What can be said of the real binary is here.

use montagent_core::report::Report;
use montagent_core::validate;
use montagent_core::verbs::fmt::{self, Mode};
use serde_json::{Value, json};

mod common;
use common::{canonical, write_project};

fn audio(extra: Value) -> Value {
    let mut element = json!({"id": "bed", "type": "audio", "start": 0, "end": 1000,
                             "source": "a.wav", "source_start": 0, "source_end": 1000});
    for (key, value) in extra.as_object().expect("an object").clone() {
        element[key] = value;
    }
    element
}

fn video(extra: Value) -> Value {
    let mut element = json!({"id": "clip", "type": "video", "start": 0, "end": 1000,
                             "source": "v.mp4", "source_start": 0, "source_end": 1000,
                             "x": 0, "y": 0, "width": 100, "height": 100, "fit": "contain"});
    for (key, value) in extra.as_object().expect("an object").clone() {
        element[key] = value;
    }
    element
}

fn rect(extra: Value) -> Value {
    let mut element = json!({"id": "box", "type": "rect", "start": 0, "end": 1000,
                             "x": 0, "y": 0, "width": 10, "height": 10, "fill": "#FF0000"});
    for (key, value) in extra.as_object().expect("an object").clone() {
        element[key] = value;
    }
    element
}

fn project(element: Value) -> String {
    canonical(
        &json!({
            "frame": {"width": 100, "height": 100}, "fps": 25, "duration": 1000,
            "output": "out/t.mp4",
            "tracks": [{"name": "t", "layer": 0, "elements": [element]}],
        })
        .to_string(),
    )
}

#[track_caller]
fn validated(body: &str) -> Report {
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(&dir, "p.montagent.json", body);
    validate(&path)
}

fn codes_starting(report: &Report, prefix: &str) -> Vec<String> {
    report
        .findings
        .iter()
        .filter(|f| f.code.starts_with(prefix))
        .map(|f| f.code.clone())
        .collect()
}

#[test]
fn an_empty_list_is_legal_on_audio_and_video_and_fmt_leaves_it_alone() {
    for element in [
        audio(json!({"audio_effects": []})),
        video(json!({"audio_effects": []})),
    ] {
        let body = project(element.clone());
        let report = validated(&body);
        assert!(
            codes_starting(&report, "E-SCHEMA").is_empty(),
            "{element}: {:?}",
            report.findings
        );
        let dir = common::tempdir(line!());
        let path = write_project(&dir, "p.montagent.json", &body);
        let report = fmt::fmt(&path, Mode::Check);
        assert!(
            report.findings.is_empty(),
            "{element}: {:?}",
            report.findings
        );
    }
}

#[test]
fn the_list_is_refused_on_a_type_that_has_no_sound() {
    let report = validated(&project(rect(json!({"audio_effects": []}))));
    assert_eq!(codes_starting(&report, "E-SCHEMA-UNKNOWN-KEY").len(), 1);
}

#[test]
fn the_union_has_no_members_so_any_name_is_a_schema_error() {
    for element in [
        audio(json!({"audio_effects": [{"name": "bell"}]})),
        video(json!({"audio_effects": [{"name": "bell"}]})),
    ] {
        let report = validated(&project(element.clone()));
        assert_eq!(
            codes_starting(&report, "E-SCHEMA").len(),
            1,
            "{element}: {:?}",
            report.findings
        );
    }
}

#[test]
fn the_list_is_the_last_key_of_an_audio_and_of_a_video() {
    // ADR-0068: a list appends after the type's existing fields. The graph order (before
    // `volume`) is the renderer's, not the file's.
    let schema = montagent_core::schema::generate();
    for (kind, last_two) in [
        ("audio", ["volume", "audio_effects"]),
        ("video", ["effects", "audio_effects"]),
    ] {
        let branch = schema["$defs"]["Element"]["oneOf"]
            .as_array()
            .expect("the element union")
            .iter()
            .find(|branch| branch["properties"]["type"]["const"] == kind)
            .expect("a branch");
        let keys: Vec<&String> = branch["properties"].as_object().unwrap().keys().collect();
        assert_eq!(
            &keys[keys.len() - 2..],
            &last_two.iter().collect::<Vec<_>>()
        );
    }
}

#[test]
fn a_visual_member_in_audio_effects_names_the_right_list_and_is_not_also_a_schema_error() {
    for element in [
        audio(json!({"audio_effects": [{"name": "blur", "radius": 3}]})),
        video(json!({"audio_effects": [{"name": "blur", "radius": 3}]})),
    ] {
        let report = validated(&project(element.clone()));
        assert_eq!(
            codes_starting(&report, "E-AUDIO-EFFECT-WRONG-LIST"),
            vec!["E-AUDIO-EFFECT-WRONG-LIST"],
            "{element}: {:?}",
            report.findings
        );
        assert!(
            codes_starting(&report, "E-SCHEMA").is_empty(),
            "{element}: {:?}",
            report.findings
        );
        let text = serde_json::to_string(&report.findings).unwrap();
        assert!(text.contains("blur") && text.contains("effects"), "{text}");
    }
}

#[test]
fn the_published_schema_lists_the_union_and_it_has_no_branch() {
    let schema = montagent_core::schema::generate();
    for kind in ["audio", "video"] {
        let branch = schema["$defs"]["Element"]["oneOf"]
            .as_array()
            .unwrap()
            .iter()
            .find(|branch| branch["properties"]["type"]["const"] == kind)
            .unwrap();
        assert_eq!(
            branch["properties"]["audio_effects"]["items"]["$ref"],
            "#/$defs/AudioEffect"
        );
    }
    assert!(schema["$defs"]["AudioEffect"].is_object());
    assert!(schema["$defs"]["AudioEffect"]["oneOf"].is_null());
}
