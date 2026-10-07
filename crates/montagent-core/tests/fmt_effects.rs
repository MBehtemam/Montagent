//! `fmt` orders the keys inside every `effects` member (#774): each member's own keys in
//! the order the model's writer emits them, and nothing else about the document moved.

use std::collections::BTreeSet;

use montagent_core::verbs::fmt::{self, Mode};
use serde_json::Value;

mod common;

/// A one-element project around `effects`, already canonical everywhere but inside the
/// members it is handed.
fn project(effects: &str) -> String {
    format!(
        r##"{{
  "frame": {{"width": 1080, "height": 1920}},
  "fps": 25,
  "tracks": [
    {{
      "name": "photo",
      "layer": 10,
      "elements": [
        {{"id":"photo-01","type":"image","start":0,"end":1000,"source":"a.png","width":100,"height":100,"fit":"literal","effects":[{effects}]}}
      ]
    }}
  ]
}}
"##
    )
}

#[test]
fn a_path_mask_written_feather_before_points_comes_out_in_canonical_order() {
    // ADR-0163 §3: `name, shape, x, y, width, height, radius, points, invert, feather`.
    let dir = common::tempdir(line!());
    let path = common::write_project(
        &dir,
        "p.montagent.json",
        &project(
            r#"{"feather":6,"invert":true,"name":"mask","points":[{"at":[0,0]},{"at":[30,0]},{"at":[30,40]}],"shape":"path","x":1,"y":2,"width":30,"height":40}"#,
        ),
    );

    fmt::fmt(&path, Mode::Write);
    let after = std::fs::read_to_string(&path).unwrap();

    assert_eq!(
        after,
        project(
            r#"{"name":"mask","shape":"path","x":1,"y":2,"width":30,"height":40,"points":[{"at":[0,0]},{"at":[30,0]},{"at":[30,40]}],"invert":true,"feather":6}"#
        )
    );
}

/// One member of every effect kind, and a mask of every shape, each carrying every field its
/// branch admits. Values are spelled the way the writer spells them, so the only thing that
/// can differ between a member as written here and the writer's output is key order.
const EVERY_MEMBER: [&str; 16] = [
    r#"{"name":"blur","radius":2.5}"#,
    r##"{"name":"shadow","dx":1.5,"dy":2.5,"radius":3.5,"color":"#000000","opacity":0.5}"##,
    r#"{"name":"mask","shape":"circle","x":1,"y":2,"width":30,"height":40,"invert":true,"feather":6}"#,
    r#"{"name":"mask","shape":"rect","x":1,"y":2,"width":30,"height":40,"radius":4,"invert":false,"feather":6}"#,
    r#"{"name":"mask","shape":"ellipse","x":1,"y":2,"width":30,"height":40,"invert":true,"feather":6}"#,
    r#"{"name":"mask","shape":"path","x":1,"y":2,"width":30,"height":40,"points":[{"at":[0,0]},{"at":[30,0],"out":[0,5]},{"at":[30,40],"in":[-2,0]}],"invert":true,"feather":6}"#,
    r##"{"name":"tint","color":"#FF0000","amount":0.5}"##,
    r#"{"name":"saturation","amount":0.5}"#,
    r#"{"name":"brightness","amount":0.5}"#,
    r#"{"name":"contrast","amount":0.5}"#,
    r##"{"name":"chroma","color":"#00FF00","tolerance":0.25,"softness":0.5,"spill":0.5}"##,
    r#"{"name":"grain","seed":7,"amount":0.25,"size":2,"mono":true}"#,
    r#"{"name":"posterize","levels":8}"#,
    r#"{"name":"glow","threshold":0.5,"radius":10.5,"intensity":1.5}"#,
    r#"{"name":"directional_blur","angle":30.5,"length":12.5}"#,
    // A keyed parameter: the keyframe records inside a member are not a structure this
    // ticket orders, so they are written in their usual order and must come through intact.
    r#"{"name":"blur","radius":[{"t":0,"v":0.5},{"t":500,"v":4.5,"ease":"linear"}]}"#,
];

/// `member` with its own keys in reverse order — as far from canonical as one member gets,
/// with every value untouched.
fn reversed(member: &str) -> String {
    let value: serde_json::Value = serde_json::from_str(member).unwrap();
    let reversed: serde_json::Map<String, serde_json::Value> = value
        .as_object()
        .unwrap()
        .iter()
        .rev()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    serde_json::to_string(&reversed).unwrap()
}

/// The bytes the model's own writer produces for `source`: the strict model's parse, its
/// serde serialisation, and the one canonical writer.
fn writer_output(source: &str) -> String {
    let project: montagent_core::model::Project =
        serde_json::from_str(source).expect("a document the strict model accepts");
    montagent_core::write::canonical(&serde_json::to_value(&project).unwrap())
}

#[test]
fn every_effect_kind_and_every_mask_shape_comes_out_as_the_writer_writes_it() {
    for member in EVERY_MEMBER {
        let scrambled = project(&reversed(member));
        let dir = common::tempdir(line!());
        let path = common::write_project(&dir, "p.montagent.json", &scrambled);

        fmt::fmt(&path, Mode::Write);
        let after = std::fs::read_to_string(&path).unwrap();

        assert_eq!(
            after,
            writer_output(&scrambled),
            "`fmt` and the writer disagree on {member}"
        );
        // And the writer's output is the member as `EVERY_MEMBER` spells it, so the test
        // above cannot pass by the two agreeing on some third order.
        assert_eq!(after, project(member), "{member}");
    }
}

#[test]
fn many_members_in_one_list_each_get_their_order_and_the_list_keeps_its_own() {
    // ADR-0040: the list's order is significant — blur-then-shadow is a different frame
    // from shadow-then-blur — so only the keys inside each member move.
    let canonical: Vec<&str> = EVERY_MEMBER.iter().rev().copied().collect();
    let scrambled: Vec<String> = canonical.iter().map(|m| reversed(m)).collect();
    let source = project(&scrambled.join(","));
    let dir = common::tempdir(line!());
    let path = common::write_project(&dir, "p.montagent.json", &source);

    fmt::fmt(&path, Mode::Write);
    let after = std::fs::read_to_string(&path).unwrap();

    assert_eq!(after, project(&canonical.join(",")));
    assert_eq!(after, writer_output(&source));
}

#[test]
fn the_members_above_cover_every_published_effect_and_every_mask_shape() {
    // A new effect kind or mask shape fails here until it has a member above, so the two
    // tests before this one keep covering the whole vocabulary.
    let schema = montagent_core::schema::generate();
    let published: BTreeSet<String> = schema
        .pointer("/$defs/Effect/oneOf")
        .and_then(Value::as_array)
        .unwrap()
        .iter()
        .map(|branch| {
            branch["properties"]["name"]["const"]
                .as_str()
                .unwrap()
                .to_string()
        })
        .collect();
    let shapes: BTreeSet<String> = schema
        .pointer("/$defs/MaskShape/enum")
        .and_then(Value::as_array)
        .expect("the mask shapes are a published enum")
        .iter()
        .map(|shape| shape.as_str().unwrap().to_string())
        .collect();

    let members: Vec<Value> = EVERY_MEMBER
        .iter()
        .map(|m| serde_json::from_str(m).unwrap())
        .collect();
    let covered: BTreeSet<String> = members
        .iter()
        .map(|m| m["name"].as_str().unwrap().to_string())
        .collect();
    let covered_shapes: BTreeSet<String> = members
        .iter()
        .filter_map(|m| m.get("shape").and_then(Value::as_str))
        .map(str::to_string)
        .collect();

    assert_eq!(covered, published);
    assert_eq!(covered_shapes, shapes);
}

fn codes(report: &montagent_core::report::Report) -> Vec<&str> {
    report.findings.iter().map(|f| f.code.as_str()).collect()
}

#[test]
fn a_document_already_canonical_comes_out_byte_identical() {
    let canonical = project(&EVERY_MEMBER.join(","));
    let dir = common::tempdir(line!());
    let path = common::write_project(&dir, "p.montagent.json", &canonical);

    let checked = fmt::fmt(&path, Mode::Check);
    let written = fmt::fmt(&path, Mode::Write);

    assert!(checked.findings.is_empty(), "{:?}", checked.findings);
    assert!(written.findings.is_empty(), "{:?}", written.findings);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), canonical);
}

#[test]
fn check_reports_an_out_of_order_member_as_layout_and_writes_nothing() {
    // The element's own keys are canonical, so this is not `L-KEY-ORDER` — that finding
    // names the element's expected order, which is already met. It is the rest of the
    // rewrite, `L-LAYOUT`, and `--check` must say so: a `--check` silent about a rewrite
    // `fmt` then makes would be a second rule.
    let source = project(r#"{"radius":2.5,"name":"blur"}"#);
    let dir = common::tempdir(line!());
    let path = common::write_project(&dir, "p.montagent.json", &source);

    let checked = fmt::fmt(&path, Mode::Check);

    assert_eq!(codes(&checked), ["L-LAYOUT"], "{:?}", checked.findings);
    assert_eq!(
        checked.findings[0].location.line,
        Some(9),
        "the element's line"
    );
    assert_eq!(std::fs::read_to_string(&path).unwrap(), source);

    // `validate` asks the same predicate, unconditionally (ADR-0041).
    let validated = montagent_core::validate(&path);
    assert!(
        codes(&validated).contains(&"L-LAYOUT"),
        "{:?}",
        validated.findings
    );
}

#[test]
fn an_out_of_order_element_and_member_are_each_reported_once() {
    let source = project(r#"{"radius":2.5,"name":"blur"}"#).replace(
        r#"{"id":"photo-01","type":"image""#,
        r#"{"type":"image","id":"photo-01""#,
    );
    let dir = common::tempdir(line!());
    let path = common::write_project(&dir, "p.montagent.json", &source);

    let report = fmt::fmt(&path, Mode::Write);

    assert_eq!(
        codes(&report),
        ["L-KEY-ORDER", "L-LAYOUT"],
        "{:?}",
        report.findings
    );
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        project(r#"{"name":"blur","radius":2.5}"#)
    );
}

// ---- Documents the strict model cannot parse ---------------------------------------
//
// ADR-0042: `fmt` refuses only a document that is not a project at all. Everything below is
// one the strict model refuses, and `fmt` formats around it.

#[test]
fn an_undeclared_key_in_a_member_follows_the_declared_ones_in_the_order_written() {
    let dir = common::tempdir(line!());
    let path = common::write_project(
        &dir,
        "p.montagent.json",
        &project(r#"{"sigma":3,"radius":2.5,"why":"x","name":"blur"}"#),
    );

    fmt::fmt(&path, Mode::Write);

    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        project(r#"{"name":"blur","radius":2.5,"sigma":3,"why":"x"}"#)
    );
}

#[test]
fn a_member_with_no_published_name_is_left_exactly_as_written() {
    // An unknown name, a missing name, a name that is not a string, and a member that is not
    // an object: none has a published order, so none is given one. The known member beside
    // them is still ordered.
    let dir = common::tempdir(line!());
    let path = common::write_project(
        &dir,
        "p.montagent.json",
        &project(
            r#"{"radius":2,"name":"vignette"},{"radius":2,"shape":"x"},{"radius":2,"name":7},"blur",{"radius":2.5,"name":"blur"}"#,
        ),
    );

    let report = fmt::fmt(&path, Mode::Write);

    assert!(
        !codes(&report).contains(&"E-PARSE"),
        "{:?}",
        report.findings
    );
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        project(
            r#"{"radius":2,"name":"vignette"},{"radius":2,"shape":"x"},{"radius":2,"name":7},"blur",{"name":"blur","radius":2.5}"#
        )
    );
}

#[test]
fn effects_that_are_not_a_list_are_left_exactly_as_written() {
    let source = project("").replace(
        r#""effects":[]"#,
        r#""effects":{"radius":2.5,"name":"blur"}"#,
    );
    let dir = common::tempdir(line!());
    let path = common::write_project(&dir, "p.montagent.json", &source);

    fmt::fmt(&path, Mode::Write);

    assert_eq!(std::fs::read_to_string(&path).unwrap(), source);
}

#[test]
fn a_member_on_an_element_of_no_published_type_is_still_ordered_by_its_own_name() {
    // The element has no published order, so its own keys stay as written (the existing
    // rule). Its member names itself, and its order is its name's, wherever it sits.
    let source = project(r#"{"radius":2.5,"name":"blur"}"#)
        .replace(r#""type":"image""#, r#""type":"scene""#);
    let dir = common::tempdir(line!());
    let path = common::write_project(&dir, "p.montagent.json", &source);

    fmt::fmt(&path, Mode::Write);

    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        project(r#"{"name":"blur","radius":2.5}"#)
            .replace(r#""type":"image""#, r#""type":"scene""#)
    );
}
