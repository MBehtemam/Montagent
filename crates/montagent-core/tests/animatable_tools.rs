//! **Every animatable property is carried by every tool** (ADR-0146 §7, #675).
//!
//! The list is the schema's ([`montagent_core::animatable`]), and this file walks it: for
//! every animatable property any element type has, a project keys it on an element of a
//! type that has it, and each tool that reads keyframe lists is asked about it — `shift`,
//! the ease, derivation and unreached checks, the contact sheet, `compare`, `timeline` and
//! `query --at`. A property the schema types as animatable and one tool skips fails here,
//! by name. That is the prototype's silent loss: `shift` moved an element's `x` keyframes and
//! left its `width` and `fill` behind.
//!
//! The projects sit beside the committed fixture so a `text` element can name its vendored
//! font, and an `audio` element its narration.

use montagent_core::animatable::{self, Kind};
use montagent_core::parse;
use montagent_core::report::{ExitCode, Report};
use montagent_core::verbs::frame::{Ask as FrameAsk, frame};
use montagent_core::verbs::query::at;
use montagent_core::verbs::shift::{Ask as ShiftAsk, shift};
use montagent_core::verbs::{compare::compare, timeline};
use serde_json::{Value, json};

mod common;
use common::{Scratch, canonical, fixture_project};

/// Two values of `kind`, written as the document writes them.
fn values(kind: Kind) -> (Value, Value) {
    match kind {
        Kind::Integer => (json!(100), json!(140)),
        Kind::Number => (json!(0.25), json!(0.75)),
        Kind::Pair => (json!([1.0, 1.0]), json!([1.2, 1.2])),
        Kind::Colour => (json!("#FF0000"), json!("#0000FF")),
    }
}

/// An element of the first type that has `property`, with `records` as its value.
fn subject(property: &str, records: Value) -> Value {
    let has = |kind| animatable::of(kind).iter().any(|p| p.name == property);
    let mut element = if has("rect") {
        json!({"id": "subject", "type": "rect", "start": 1000, "end": 3000, "x": 540, "y": 960,
               "width": 400, "height": 400, "fill": "#FFFFFF", "stroke": "#000000",
               "stroke_width": 2})
    } else if has("text") {
        json!({"id": "subject", "type": "text", "start": 1000, "end": 3000, "x": 540, "y": 960,
               "width": 900, "height": 200, "font": "brand", "size": 40,
               "runs": [{"text": "Hello"}], "caption": false})
    } else if has("audio") {
        json!({"id": "subject", "type": "audio", "start": 1000, "end": 3000,
               "source": "audio/05-cobweb.mp3", "source_start": 0, "source_end": 1500,
               "overrun": "loop"})
    } else {
        panic!("`{property}` is animatable on no type this test knows how to write");
    };
    element[property] = records;
    element
}

/// A project holding `subject`, and a `marker` rect whose `start` is `marker_start`.
fn project(subject: Value, marker_start: i64) -> String {
    let fixture = common::document(&fixture_project());
    canonical(
        &json!({
            "frame": {"width": 1080, "height": 1920},
            "fps": 25,
            "background": "#000000",
            "fonts": fixture["fonts"],
            "fontVendor": fixture["fontVendor"],
            "tracks": [
                {"name": "a", "layer": 1, "elements": [subject]},
                {"name": "b", "layer": 2, "elements": [
                    {"id": "marker", "type": "rect", "start": marker_start, "end": 3000,
                     "x": 100, "y": 100, "width": 10, "height": 10, "fill": "#FF0000"}
                ]}
            ]
        })
        .to_string(),
    )
}

fn findings_on(report: &Report, code: &str, property: &str) -> usize {
    report
        .findings
        .iter()
        .filter(|finding| {
            finding.code == code && finding.fields.get("property") == Some(&json!(property))
        })
        .count()
}

#[test]
fn every_animatable_property_is_carried_by_every_tool() {
    if !common::has_ffprobe() {
        return;
    }
    let mut missed: Vec<String> = Vec::new();
    for property in animatable::names() {
        let kind = animatable::property(property).unwrap().kind;
        let (a, b) = values(kind);
        let keyed = subject(
            property,
            json!([{"t": 1000, "v": a}, {"t": 1500, "v": b, "ease": "linear"},
                   {"t": 2000, "v": a, "ease": "linear"}]),
        );
        let mut miss = |tool: &str| missed.push(format!("{tool} skips `{property}`"));

        let base =
            Scratch::beside_the_fixture("animatable-tools-base", &project(keyed.clone(), 2000));
        let validated = montagent_core::validate(base.path());
        assert_eq!(
            validated.exit_code(),
            ExitCode::Ok,
            "`{property}`: {:?}",
            validated.findings
        );

        // `timeline` marks it as moving.
        let view = timeline::timeline(base.path()).to_json();
        let detail = view["timeline"]["groups"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|group| group["elements"].as_array().unwrap())
            .find(|element| element["id"] == "subject")
            .map(|element| element["detail"].as_str().unwrap().to_string())
            .unwrap();
        if !detail.ends_with("~anim") {
            miss("timeline");
        }

        // `query --at` prints its resolved value.
        let document = parse::read(base.path()).unwrap();
        let stack = at::at(&document, 1250, None);
        let resolved = stack
            .stack
            .iter()
            .find(|present| present.named.id.as_deref() == Some("subject"))
            .and_then(|present| {
                present
                    .values
                    .iter()
                    .find(|value| value.property == *property)
            });
        if !resolved.is_some_and(|value| value.animated && value.value.is_some()) {
            miss("query --at");
        }

        // The contact sheet counts its change point.
        let sheet = frame(
            base.path(),
            &FrameAsk {
                from: Some(0),
                to: Some(3000),
                keyframes: true,
                ..FrameAsk::default()
            },
        )
        .to_json();
        let point = format!("subject.{property}@1500");
        let tiled = sheet["sheet"]["provenance"]
            .as_array()
            .into_iter()
            .flatten()
            .any(|tile| {
                tile["keyframes"]
                    .as_array()
                    .unwrap()
                    .contains(&json!(point))
            });
        // The sheet counts change points on what it can see, so an audible property has none.
        let visual = keyed["type"] != "audio";
        if visual && !tiled {
            miss("the contact sheet");
        }

        // `compare` reads its instants: the keyframe at 2000 sat on `marker`'s start.
        let moved =
            Scratch::beside_the_fixture("animatable-tools-moved", &project(keyed.clone(), 2100));
        let drift = compare(base.path(), moved.path());
        let label = format!("subject's {property} keyframe");
        if !drift.findings.iter().any(|finding| {
            finding.code == "D-KEYFRAME-INSTANT-DRIFT" && finding.fields["left"] == json!(label)
        }) {
            miss("compare");
        }
        drop(moved);

        // `shift` carries it with its element.
        let answer = shift(
            base.path(),
            &ShiftAsk {
                at: 500,
                delta: 100,
                scope: None,
                release: Vec::new(),
            },
        );
        assert_eq!(answer.report().exit_code(), ExitCode::Ok, "`{property}`");
        let written = common::document(base.path());
        let times: Vec<i64> = written["tracks"][0]["elements"][0][property.as_str()]
            .as_array()
            .unwrap()
            .iter()
            .map(|record| record["t"].as_i64().unwrap())
            .collect();
        if times != [1100, 1600, 2100] {
            miss("shift");
        }
        drop(base);

        // The ease and derivation checks walk it.
        let held = subject(
            property,
            json!([{"t": 1000, "v": a},
                   {"t": 1500, "t_from": {"rule": "after-previous", "ms": 400}, "v": a,
                    "ease": "linear"},
                   {"t": 2000, "v": b, "ease": "linear"}]),
        );
        let checked = Scratch::beside_the_fixture("animatable-tools-held", &project(held, 2000));
        let report = montagent_core::validate(checked.path());
        if findings_on(&report, "R-EASE-INERT", property) == 0 {
            miss("the ease check");
        }
        if findings_on(&report, "R-DERIVED-T", property) == 0 {
            miss("the derivation check");
        }
        drop(checked);

        // The unreached check resolves it: the last record's plateau holds no frame.
        let late = subject(
            property,
            json!([{"t": 1000, "v": a}, {"t": 2990, "v": b, "ease": "linear"}]),
        );
        let checked = Scratch::beside_the_fixture("animatable-tools-late", &project(late, 2000));
        let report = montagent_core::validate(checked.path());
        if findings_on(&report, "R-KEYFRAME-UNREACHED", property) == 0 {
            miss("the unreached check");
        }
    }
    assert!(missed.is_empty(), "{missed:#?}");
}
