//! **Every animatable property is carried by every tool** (ADR-0146 §7, #675).
//!
//! The list is the schema's ([`montagent_core::animatable`]), and this file walks it: for
//! every animatable property any element type has, and every parameter of every `effects`
//! member (#676, named `effects[1].radius (blur)`), a project keys it on an element that has
//! it, and each tool that reads keyframe lists is asked about it — `shift`,
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
        Kind::Colour | Kind::Paint => (json!("#FF0000"), json!("#0000FF")),
        Kind::Points => (
            json!([{"at": [10, 390]}, {"at": [200, 10]}, {"at": [390, 390]}]),
            json!([{"at": [10, 390]}, {"at": [200, 200]}, {"at": [390, 390]}]),
        ),
        Kind::Stops => (
            json!([{"offset": 0, "color": "#FF0000"}, {"offset": 1, "color": "#0000FF"}]),
            json!([{"offset": 0.2, "color": "#00FF00"}, {"offset": 0.8, "color": "#FFFFFF"}]),
        ),
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
    } else if has("path") {
        json!({"id": "subject", "type": "path", "start": 1000, "end": 3000, "x": 540, "y": 960,
               "width": 400, "height": 400, "closed": true, "fill": "#FFFFFF",
               "stroke": "#000000", "stroke_width": 2,
               "points": [{"at": [10, 390]}, {"at": [200, 10]}, {"at": [390, 390]}]})
    } else if has("audio") {
        json!({"id": "subject", "type": "audio", "start": 1000, "end": 3000,
               "source": "audio/05-cobweb.mp3", "source_start": 0, "source_end": 1500,
               "overrun": "loop"})
    } else {
        panic!("`{property}` is animatable on no type this test knows how to write");
    };
    // A gradient's parameter is a nested path, `fill.angle` (ADR-0149 §6): the paint holds a
    // gradient whose other parameters are literals.
    match property.split_once('.') {
        None => element[property] = records,
        Some((paint, parameter)) => {
            let mut gradient = match parameter {
                "center" | "radius" => json!({"gradient": "radial", "center": [0.5, 0.5],
                                              "radius": 1}),
                _ => json!({"gradient": "linear", "angle": 90}),
            };
            gradient["stops"] = values(Kind::Stops).0;
            gradient[parameter] = records;
            element[paint] = gradient;
        }
    }
    element
}

/// What the document writes at `property`, which may be a nested path.
fn written<'a>(element: &'a Value, property: &str) -> &'a Value {
    property.split('.').fold(element, |value, key| &value[key])
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

/// A member of `effects` with every required parameter written static, so one of them can be
/// keyed in its place. A member the format gains fails here until it is given one, which is
/// the point: it has to be carried by every tool too.
fn effect(name: &str) -> Value {
    match name {
        "blur" => json!({"name": "blur", "radius": 4}),
        "shadow" => {
            json!({"name": "shadow", "dx": 4, "dy": 4, "radius": 6, "color": "#000000",
                   "opacity": 0.5})
        }
        "mask" => {
            json!({"name": "mask", "shape": "rect", "x": 0, "y": 0, "width": 400,
                   "height": 400})
        }
        "tint" => json!({"name": "tint", "color": "#FF8800", "amount": 0.5}),
        "saturation" | "brightness" | "contrast" => json!({"name": name, "amount": 0.5}),
        "chroma" => {
            json!({"name": "chroma", "color": "#00FF00", "tolerance": 0.2, "softness": 0.1,
                   "spill": 0.1})
        }
        "grain" => json!({"name": "grain", "seed": 7, "amount": 0.2, "size": 2, "mono": true}),
        "posterize" => json!({"name": "posterize", "levels": 8}),
        "glow" => json!({"name": "glow", "threshold": 0.5, "radius": 10, "intensity": 1}),
        "directional_blur" => json!({"name": "directional_blur", "angle": 30, "length": 12}),
        other => panic!("no static `{other}` for this test to key a parameter of"),
    }
}

/// Every animatable property, as `(the name a tool calls it, kind, its key, the effect it
/// sits in)`: the element's own, then every parameter of every `effects` member, named by
/// its position with the effect's name in the text (ADR-0146 §4) — at position `1`, behind a
/// static member of the same name, so a tool that read the first member of a name, or the
/// bare key, is caught.
fn every_property() -> Vec<(String, Kind, String, Option<&'static str>)> {
    let mut out: Vec<_> = animatable::names()
        .iter()
        .map(|name| {
            let kind = animatable::property(name).unwrap().kind;
            (name.clone(), kind, name.clone(), None)
        })
        .collect();
    for (member, parameters) in animatable::effect_members() {
        for parameter in parameters {
            out.push((
                format!("effects[1].{} ({member})", parameter.name),
                parameter.kind,
                parameter.name.clone(),
                Some(member),
            ));
        }
    }
    out
}

/// The subject keyed at `key` with `records`: the element's own key, or the parameter of the
/// second of two `member` effects.
fn keyed_subject(key: &str, member: Option<&str>, records: Value) -> Value {
    let Some(member) = member else {
        return subject(key, records);
    };
    let mut element = subject("x", json!(540));
    let mut keyed = effect(member);
    keyed[key] = records;
    element["effects"] = json!([effect(member), keyed]);
    element
}

/// The keyframe list a tool wrote back for `key` on the subject.
fn written_list<'a>(subject: &'a Value, key: &str, member: Option<&str>) -> &'a Value {
    match member {
        None => written(subject, key),
        Some(_) => &subject["effects"][1][key],
    }
}

#[test]
fn every_animatable_property_is_carried_by_every_tool() {
    if !common::has_ffprobe() {
        return;
    }
    let mut missed: Vec<String> = Vec::new();
    let every = every_property();
    // The effect parameters are in the list (ADR-0146 §3, slice 2).
    assert!(
        every
            .iter()
            .any(|(name, ..)| name == "effects[1].tolerance (chroma)"),
        "{every:#?}"
    );
    for (property, kind, key, member) in &every {
        let (kind, key, member) = (*kind, key.as_str(), *member);
        let (a, b) = values(kind);
        let keyed = keyed_subject(
            key,
            member,
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
        // A gradient's parameter is printed inside its paint, resolved: the midpoint is
        // neither end.
        let (printed_as, parameter) = match member {
            None => property.split_once('.').unwrap_or((property, "")),
            Some(_) => (property.as_str(), ""),
        };
        let resolved = stack
            .stack
            .iter()
            .find(|present| present.named.id.as_deref() == Some("subject"))
            .and_then(|present| {
                present
                    .values
                    .iter()
                    .find(|value| value.property == printed_as)
            });
        let midway = |value: &Value| {
            parameter.is_empty() || (value[parameter] != a && value[parameter] != b)
        };
        if !resolved.is_some_and(|value| value.animated && value.value.as_ref().is_some_and(midway))
        {
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
        let times: Vec<i64> = written_list(&written["tracks"][0]["elements"][0], key, member)
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
        let held = keyed_subject(
            key,
            member,
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
        let late = keyed_subject(
            key,
            member,
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
