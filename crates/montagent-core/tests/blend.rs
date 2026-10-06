//! `blend`: a flat, static field of five values, and the finished element blends last
//! (ADR-0147, #680).
//!
//! Each seam the field reaches is asked here through its verb: the schema through
//! `validate` and the model, the paint through `frame`, `query --at`'s stack, the
//! `R-BLEND-BACKGROUND-ONLY` review, and `shift`, `compare` and `timeline` carrying it.
//! The gating test, frames byte-identical across painter counts, is in `tests/painters.rs`
//! beside the other painter-count guards.

use montagent_core::model::Project;
use montagent_core::report::{ExitCode, Report};
use montagent_core::validate;
use serde_json::{Value, json};

mod common;
use common::{canonical, tempdir, write_project};

/// The five values, in ADR-0147's table order.
const MODES: [&str; 5] = ["normal", "multiply", "screen", "overlay", "add"];

/// A project of one track per element, the first element lowest.
fn project(elements: &[Value]) -> Value {
    let tracks: Vec<Value> = elements
        .iter()
        .enumerate()
        .map(|(i, element)| json!({"name": format!("t{i}"), "layer": i, "elements": [element]}))
        .collect();
    json!({"frame": {"width": 64, "height": 48}, "fps": 25, "background": "#000000",
           "duration": 200, "tracks": tracks})
}

fn rect(id: &str, fill: &str, extra: Value) -> Value {
    let mut element = json!({"id": id, "type": "rect", "start": 0, "end": 200,
        "x": 0, "y": 0, "origin": "top-left", "width": 64, "height": 48, "fill": fill});
    for (key, value) in extra.as_object().expect("an object") {
        element[key] = value.clone();
    }
    element
}

#[track_caller]
fn validated(project: &Value) -> Report {
    let dir = tempdir(std::panic::Location::caller().line());
    let path = write_project(&dir, "p.montagent.json", &canonical(&project.to_string()));
    validate(&path)
}

fn codes(report: &Report) -> Vec<&str> {
    report.findings.iter().map(|f| f.code.as_str()).collect()
}

// ---------------------------------------------------------------------------------------
// The schema.
// ---------------------------------------------------------------------------------------

#[test]
fn every_mode_validates_clean_on_every_visual_type() {
    for mode in MODES {
        let report = validated(&project(&[
            rect("under", "#808080", json!({})),
            rect("over", "#C04020", json!({"blend": mode})),
        ]));
        assert_eq!(
            report.exit_code(),
            ExitCode::Ok,
            "{mode}: {:?}",
            codes(&report)
        );
        assert!(report.findings.is_empty(), "{mode}: {:?}", codes(&report));
    }
    // The other visual types take the field too, in the model.
    for element in [
        r##"{"id":"a","type":"ellipse","start":0,"end":1,"width":10,"height":10,"fill":"#000000","blend":"screen"}"##,
        r##"{"id":"a","type":"text","start":0,"end":1,"width":10,"height":10,"font":"f","size":20,"runs":[{"text":"hi"}],"blend":"add"}"##,
        r##"{"id":"a","type":"image","start":0,"end":1,"source":"a.png","width":10,"height":10,"fit":"literal","blend":"multiply"}"##,
        r##"{"id":"a","type":"video","start":0,"end":1,"source":"a.mp4","source_start":0,"source_end":1,"width":10,"height":10,"fit":"literal","blend":"overlay"}"##,
    ] {
        parsed(element).unwrap_or_else(|e| panic!("{element}: {e}"));
    }
}

fn parsed(element: &str) -> Result<Project, String> {
    let source = format!(
        r##"{{"frame":{{"width":1080,"height":1920}},"fps":25,"tracks":[{{"name":"t","layer":1,"elements":[{element}]}}]}}"##
    );
    serde_json::from_str::<Project>(&source).map_err(|e| e.to_string())
}

#[test]
fn audio_and_transition_refuse_the_field() {
    for element in [
        r##"{"id":"a","type":"audio","start":0,"end":1,"source":"a.wav","source_start":0,"source_end":1,"blend":"screen"}"##,
        r##"{"id":"a","type":"transition","start":0,"end":1,"kind":"crossfade","from":"b","to":"c","blend":"screen"}"##,
    ] {
        let message = parsed(element).expect_err(element);
        assert!(message.contains("blend"), "{message}");
    }
}

#[test]
fn an_unknown_mode_and_a_keyframe_list_are_schema_errors() {
    for blend in [
        json!("plus"),
        json!("linear_dodge"),
        json!("soft_light"),
        json!("Screen"),
        json!([{"t": 0, "v": "screen"}]),
    ] {
        let report = validated(&project(&[
            rect("under", "#808080", json!({})),
            rect("over", "#C04020", json!({"blend": blend})),
        ]));
        assert_eq!(report.exit_code(), ExitCode::Errors, "{blend}");
        assert!(
            codes(&report)
                .iter()
                .any(|code| code.starts_with("E-SCHEMA")),
            "{blend}: {:?}",
            codes(&report)
        );
    }
}

// ---------------------------------------------------------------------------------------
// The paint.
// ---------------------------------------------------------------------------------------

/// One instant of `project`, at true scale, as RGBA.
#[track_caller]
fn painted(project: &Value, at: i64) -> image::RgbaImage {
    let dir = tempdir(std::panic::Location::caller().line());
    let path = write_project(&dir, "p.montagent.json", &canonical(&project.to_string()));
    common::compare::rendered(&path, at, true)
}

fn pixel(picture: &image::RgbaImage, x: u32, y: u32) -> [u8; 3] {
    let [r, g, b, _] = picture.get_pixel(x, y).0;
    [r, g, b]
}

#[test]
fn each_mode_paints_adr_0147s_table_for_an_opaque_source_over_an_opaque_backdrop() {
    // Source #C04020 over backdrop #40C060. Worked by hand from the table in ADR-0147 §1,
    // per channel on 0..255, rounded to the nearest byte. The backdrop's channels sit either
    // side of one half, so `overlay` takes both of its branches: red and blue multiply, green
    // screens.
    let expected: [(&str, [u8; 3]); 5] = [
        ("normal", [192, 64, 32]),
        ("multiply", [48, 48, 12]),
        ("screen", [208, 208, 116]),
        ("overlay", [96, 161, 24]),
        ("add", [255, 255, 128]),
    ];
    for (mode, rgb) in expected {
        let picture = painted(
            &project(&[
                rect("backdrop", "#40C060", json!({})),
                rect("source", "#C04020", json!({"blend": mode})),
            ]),
            0,
        );
        assert_eq!(pixel(&picture, 32, 24), rgb, "{mode}");
    }
}

#[test]
fn a_written_normal_paints_the_bytes_the_field_omitted_paints() {
    let half = json!({"opacity": 0.5, "effects": [{"name": "shadow", "dx": 4, "dy": 4,
        "radius": 3, "color": "#000000", "opacity": 0.8}], "width": 30, "height": 20,
        "x": 10, "y": 10});
    let mut written = half.clone();
    written["blend"] = json!("normal");
    let omitted = painted(
        &project(&[
            rect("backdrop", "#40C060", json!({})),
            rect("source", "#C04020", half),
        ]),
        0,
    );
    let normal = painted(
        &project(&[
            rect("backdrop", "#40C060", json!({})),
            rect("source", "#C04020", written),
        ]),
        0,
    );
    assert!(
        omitted.as_raw() == normal.as_raw(),
        "a written `normal` changed bytes"
    );
}

#[test]
fn a_shadow_blends_in_the_elements_mode() {
    // A black shadow, offset clear of its element, over a white backdrop. Under `screen`
    // and `add` black is the identity, so the shadow vanishes; under `multiply` it darkens
    // as a `normal` one does.
    let shadowed = |mode: &str| {
        painted(
            &project(&[
                rect("backdrop", "#FFFFFF", json!({})),
                rect(
                    "card",
                    "#3060C0",
                    json!({"blend": mode, "x": 4, "y": 4, "width": 20,
                    "height": 16, "effects": [{"name": "shadow", "dx": 24, "dy": 0,
                    "radius": 0, "color": "#000000", "opacity": 1}]}),
                ),
            ]),
            0,
        )
    };
    // Inside the shadow's own footprint, 24 px right of the card and clear of it.
    let (x, y) = (38, 12);
    assert_eq!(
        pixel(&shadowed("normal"), x, y),
        [0, 0, 0],
        "a normal shadow"
    );
    assert_eq!(
        pixel(&shadowed("multiply"), x, y),
        [0, 0, 0],
        "multiply darkens"
    );
    assert_eq!(
        pixel(&shadowed("screen"), x, y),
        [255, 255, 255],
        "screen drops it"
    );
    assert_eq!(
        pixel(&shadowed("add"), x, y),
        [255, 255, 255],
        "add drops it"
    );
}

#[test]
fn a_crossfade_between_two_blended_elements_needs_no_special_case() {
    // Two `screen` elements bridged by a crossfade over a grey backdrop. Outside the window
    // each is alone and fully itself; in the middle the picture lies between the two.
    let from = json!({"id": "a", "type": "rect", "start": 0, "end": 160, "x": 0, "y": 0,
        "origin": "top-left", "width": 64, "height": 48, "fill": "#C00000", "blend": "screen"});
    let to = json!({"id": "b", "type": "rect", "start": 40, "end": 200, "x": 0, "y": 0,
        "origin": "top-left", "width": 64, "height": 48, "fill": "#0000C0", "blend": "screen"});
    let fade = json!({"id": "fade", "type": "transition", "start": 40, "end": 160,
        "kind": "crossfade", "from": "a", "to": "b"});
    let mut doc = project(&[rect("backdrop", "#404040", json!({}))]);
    doc["tracks"].as_array_mut().expect("tracks").extend([
        json!({"name": "a", "layer": 1, "elements": [from]}),
        json!({"name": "b", "layer": 2, "elements": [to]}),
        json!({"name": "fx", "layer": 3, "elements": [fade]}),
    ]);
    let report = validated(&doc);
    assert_eq!(report.exit_code(), ExitCode::Ok, "{:?}", codes(&report));

    // Screen of #C0 over #40 is 208; of #00 over #40 is 64.
    assert_eq!(pixel(&painted(&doc, 0), 32, 24), [208, 64, 64], "only `a`");
    assert_eq!(
        pixel(&painted(&doc, 199), 32, 24),
        [64, 64, 208],
        "only `b`"
    );
    let [r, _, b] = pixel(&painted(&doc, 100), 32, 24);
    assert!(64 < r && r < 208 && 64 < b && b < 208, "mid-fade: {r}, {b}");
}

// ---------------------------------------------------------------------------------------
// `R-BLEND-BACKGROUND-ONLY`.
// ---------------------------------------------------------------------------------------

const FINDING: &str = "R-BLEND-BACKGROUND-ONLY";

/// A 20×20 `screen` square whose top-left is at `(x, y)`, with `extra` laid over it.
fn glow(x: i64, y: i64, extra: Value) -> Value {
    rect(
        "glow",
        "#FFC040",
        json!({"blend": "screen", "x": x, "y": y, "width": 20,
        "height": 20}),
    )
    .as_object()
    .map(|base| {
        let mut element = Value::Object(base.clone());
        for (key, value) in extra.as_object().expect("an object") {
            element[key] = value.clone();
        }
        element
    })
    .expect("an object")
}

/// A 20×20 `normal` square at `(x, y)`, lower in the stack than [`glow`].
fn subject(x: i64, y: i64, extra: Value) -> Value {
    let mut element = rect(
        "subject",
        "#2040A0",
        json!({"x": x, "y": y, "width": 20,
        "height": 20}),
    );
    for (key, value) in extra.as_object().expect("an object") {
        element[key] = value.clone();
    }
    element
}

/// The lines `validate`'s report adds beneath its NOT CHECKED sentence.
fn not_checked_also(report: &Report) -> Vec<String> {
    report.to_json()["not_checked_also"]
        .as_array()
        .map(|lines| {
            lines
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn a_blended_element_with_nothing_beneath_it_is_a_review_naming_the_interval_and_instant() {
    // The `subject` is far from the glow: nothing lower in the stack meets its box.
    let report = validated(&project(&[
        subject(40, 26, json!({})),
        glow(2, 2, json!({})),
    ]));
    assert_eq!(codes(&report), [FINDING], "once per element");
    let finding = &report.findings[0];
    assert_eq!(finding.class, montagent_core::finding::Class::Review);
    assert_eq!(finding.fields["element"], "glow");
    assert_eq!(finding.fields["blend"], "screen");
    assert_eq!(finding.fields["start"], 0);
    assert_eq!(finding.fields["end"], 200);
    assert_eq!(finding.fields["instant"], 0);
    assert_eq!(report.exit_code(), ExitCode::Ok, "a review gates nothing");
    assert!(not_checked_also(&report).is_empty());
}

#[test]
fn the_instant_is_the_first_painted_frame_with_nothing_beneath() {
    // The glow slides right off its subject: x from 2 to 42 over the element's 200 ms, so
    // its box clears the subject's right edge (x = 22) once x > 22, past half way. At 25 fps
    // the frames are every 40 ms, and the first frame past half way is 120 ms (x = 26).
    let slide = json!([{"t": 0, "v": 2}, {"t": 200, "v": 42, "ease": "linear"}]);
    let report = validated(&project(&[
        subject(2, 2, json!({})),
        glow(2, 2, json!({"x": slide})),
    ]));
    let found: Vec<_> = report
        .findings
        .iter()
        .filter(|f| f.code == FINDING)
        .collect();
    assert_eq!(found.len(), 1, "{:?}", codes(&report));
    assert_eq!(found[0].fields["instant"], 120);
}

#[test]
fn a_rotated_element_with_nothing_beneath_it_still_fires() {
    // Its bounding box meets no lower bounding box, which proves no box meets it.
    let report = validated(&project(&[
        subject(40, 26, json!({})),
        glow(2, 2, json!({"rotation": 30})),
    ]));
    assert_eq!(codes(&report), [FINDING]);
}

#[test]
fn a_blended_element_over_its_subject_is_clean_and_a_partial_overhang_is_silent() {
    for (x, y) in [(2, 2), (12, 12), (-10, -10)] {
        let report = validated(&project(&[subject(2, 2, json!({})), glow(x, y, json!({}))]));
        assert!(
            report.findings.is_empty(),
            "({x}, {y}): {:?}",
            codes(&report)
        );
        assert!(not_checked_also(&report).is_empty(), "({x}, {y})");
    }
}

#[test]
fn whatever_the_lower_elements_opacity_or_blend_it_is_beneath() {
    let report = validated(&project(&[
        subject(2, 2, json!({"opacity": 0, "blend": "multiply"})),
        glow(4, 4, json!({})),
    ]));
    // The subject itself has nothing beneath it, and is reported; the glow is not.
    assert_eq!(codes(&report), [FINDING]);
    assert_eq!(report.findings[0].fields["element"], "subject");
}

#[test]
fn a_normal_element_with_nothing_beneath_it_draws_no_finding() {
    let report = validated(&project(&[rect(
        "alone",
        "#FFC040",
        json!({"blend": "normal",
        "width": 20, "height": 20}),
    )]));
    assert!(report.findings.is_empty(), "{:?}", codes(&report));
}

#[test]
fn when_the_only_box_beneath_is_rotated_the_element_is_named_under_not_checked() {
    // The bounding boxes meet, but the subject turns about its top-left corner, so whether
    // the boxes themselves meet is a question about a turned square.
    let report = validated(&project(&[
        subject(10, 10, json!({"rotation": 45})),
        glow(20, 20, json!({})),
    ]));
    assert!(report.findings.is_empty(), "{:?}", codes(&report));
    let lines = not_checked_also(&report);
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert!(
        lines[0].contains("glow") && lines[0].contains(FINDING),
        "{lines:?}"
    );

    let text =
        montagent_core::text::render(&report.to_json(), montagent_core::text::Options::default())
            .expect("the report renders");
    let boundary = text
        .split("NOT CHECKED")
        .nth(1)
        .expect("the boundary prints");
    assert!(boundary.contains("glow"), "{text}");

    // One unrotated intersection at every frame keeps it clean, rotated neighbours or not.
    let doc = project(&[
        rect("ground", "#101010", json!({})),
        subject(10, 10, json!({"rotation": 45})),
        glow(20, 20, json!({})),
    ]);
    let report = validated(&doc);
    assert!(report.findings.is_empty(), "{:?}", codes(&report));
    assert!(not_checked_also(&report).is_empty());
}

// ---------------------------------------------------------------------------------------
// `shift`, `compare` and `timeline` carry the field.
// ---------------------------------------------------------------------------------------

#[test]
fn shift_compare_and_timeline_carry_the_field_unchanged() {
    use montagent_core::verbs::{compare::compare, shift, timeline::timeline};

    let mut doc = project(&[
        rect("ground", "#40C060", json!({})),
        rect(
            "glow",
            "#C04020",
            json!({"blend": "overlay", "start": 40, "end": 160}),
        ),
    ]);
    // A declared `duration` is slack a shift would have to be told it may change.
    doc.as_object_mut().expect("an object").remove("duration");
    let dir = tempdir(line!());
    let reference = write_project(&dir, "ref.montagent.json", &canonical(&doc.to_string()));
    let path = write_project(&dir, "p.montagent.json", &canonical(&doc.to_string()));

    let shifted = shift::shift(
        &path,
        &shift::Ask {
            at: 0,
            delta: 40,
            scope: None,
            release: Vec::new(),
        },
    );
    assert_eq!(
        shifted.report().exit_code(),
        ExitCode::Ok,
        "{:?}",
        codes(shifted.report())
    );
    let written: Value =
        serde_json::from_str(&std::fs::read_to_string(&path).expect("the shifted file"))
            .expect("JSON");
    let glow = &written["tracks"][1]["elements"][0];
    assert_eq!(glow["start"], 80, "shift moved it");
    assert_eq!(glow["blend"], "overlay", "and kept its blend");
    assert_eq!(
        written["tracks"][0]["elements"][0].get("blend"),
        None,
        "nor added one"
    );

    let compared = compare(&reference, &path);
    assert_eq!(compared.exit_code(), ExitCode::Ok, "{:?}", codes(&compared));

    let viewed = timeline(&path);
    assert_eq!(
        viewed.report().exit_code(),
        ExitCode::Ok,
        "{:?}",
        codes(viewed.report())
    );
}

// ---------------------------------------------------------------------------------------
// `query --at`.
// ---------------------------------------------------------------------------------------

#[test]
fn query_at_reports_blend_for_every_visual_member_normal_included() {
    use montagent_core::verbs::query::{Ask, query};

    let mut doc = project(&[
        rect("ground", "#40C060", json!({})),
        rect("plain", "#C04020", json!({"blend": "normal", "width": 10})),
        rect("glow", "#C04020", json!({"blend": "screen", "width": 20})),
    ]);
    doc["tracks"]
        .as_array_mut()
        .expect("tracks")
        .push(json!({"name": "narration", "elements": [
        {"id": "narration", "type": "audio", "start": 0, "end": 200, "source": "line.wav",
         "source_start": 0, "source_end": 200}]}));
    let dir = tempdir(line!());
    let path = write_project(&dir, "p.montagent.json", &canonical(&doc.to_string()));
    let answer = query(
        &path,
        &Ask {
            at: Some(100),
            ..Ask::default()
        },
    );
    assert_eq!(answer.report().exit_code(), ExitCode::Ok);
    let json = answer.to_json();
    let blend_of = |id: &str| {
        json["query"]["stack"]
            .as_array()
            .expect("a stack")
            .iter()
            .find(|member| member["id"] == id)
            .unwrap_or_else(|| panic!("{id} is present"))["blend"]
            .clone()
    };
    assert_eq!(
        blend_of("ground"),
        json!("normal"),
        "omitted reads `normal`"
    );
    assert_eq!(blend_of("plain"), json!("normal"));
    assert_eq!(blend_of("glow"), json!("screen"));
    assert_eq!(blend_of("narration"), Value::Null, "audio has no blend");

    let text = montagent_core::text::render(&json, montagent_core::text::Options::default())
        .expect("the answer renders");
    let row = |id: &str| {
        text.lines()
            .find(|line| line.split_whitespace().nth(1) == Some(id))
            .unwrap_or_else(|| panic!("no row for {id}:\n{text}"))
            .to_string()
    };
    assert!(row("ground").contains("blend normal"), "{text}");
    assert!(row("glow").contains("blend screen"), "{text}");
    assert!(!row("narration").contains("blend"), "{text}");
}
