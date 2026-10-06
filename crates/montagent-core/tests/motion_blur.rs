//! `motion_blur`: a per-element field that accumulates the element over a centred shutter
//! (ADR-0155, #719).
//!
//! Each seam the field reaches is asked here through its verb: the schema through
//! `validate`, the sample instants through the one function that states them, the paint
//! through `frame`, the `R-MOTION-BLUR-STILL` review, `query --at`'s moving or still line,
//! and `shift` carrying the field. The gating test, frames byte-identical across painter
//! counts, is in `tests/painters.rs` beside the other painter-count guards.

use montagent_core::motion_blur::sample_instants;

// ---------------------------------------------------------------------------------------
// The sample instants.
// ---------------------------------------------------------------------------------------

#[test]
fn the_sample_instants_at_30_fps_shutter_360_samples_4_are_the_midpoints_of_adr_0155() {
    // Frame 1 at 30 fps is painted at ⌊1000/30⌋ = 33 ms. A 360° shutter spans 1000/30 ms;
    // its four midpoints sit at −3/8, −1/8, 1/8 and 3/8 of it: ±12.5 ms and ±25/6 ms.
    assert_eq!(
        sample_instants(1, 30, 360, 4),
        vec![(41, 2), (173, 6), (223, 6), (91, 2)]
    );
    // Frame 2 at 66 ms.
    assert_eq!(
        sample_instants(2, 30, 360, 4),
        vec![(107, 2), (371, 6), (421, 6), (157, 2)]
    );
}

#[test]
fn at_360_degrees_no_instant_is_shared_with_the_next_frame() {
    for n in 0..90 {
        let this = sample_instants(n, 30, 360, 4);
        let next = sample_instants(n + 1, 30, 360, 4);
        let ms = |(numerator, denominator): (i128, i128)| numerator as f64 / denominator as f64;
        let last = this.last().copied().map(ms).expect("four samples");
        let first = next.first().copied().map(ms).expect("four samples");
        assert!(last < first, "frame {n}: {last} >= {first}");
        assert!(this.iter().all(|t| !next.contains(t)), "frame {n}");
    }
}

#[test]
fn every_instant_is_in_lowest_terms_with_a_positive_denominator() {
    for (shutter, samples) in [(180, 8), (1, 2), (360, 32), (90, 3)] {
        for t in sample_instants(7, 24, shutter, samples) {
            let (numerator, denominator) = t;
            assert!(denominator > 0, "{t:?}");
            let gcd = (1..=denominator)
                .rev()
                .find(|d| numerator % d == 0 && denominator % d == 0)
                .expect("1 divides both");
            assert_eq!(gcd, 1, "{t:?}");
        }
    }
}

// ---------------------------------------------------------------------------------------
// The schema.
// ---------------------------------------------------------------------------------------

use montagent_core::model::Project;
use montagent_core::report::{ExitCode, Report};
use montagent_core::validate;
use serde_json::{Value, json};

mod common;
use common::{canonical, tempdir, write_project};

/// A project of one track per element, the first element lowest.
fn project(elements: &[Value]) -> Value {
    let tracks: Vec<Value> = elements
        .iter()
        .enumerate()
        .map(|(i, element)| json!({"name": format!("t{i}"), "layer": i, "elements": [element]}))
        .collect();
    json!({"frame": {"width": 64, "height": 48}, "fps": 25, "background": "#000000",
           "duration": 400, "tracks": tracks})
}

/// A 16×12 rect at `(4, 4)`, top-left, with `extra` laid over it.
fn rect(id: &str, extra: Value) -> Value {
    let mut element = json!({"id": id, "type": "rect", "start": 0, "end": 400,
        "x": 4, "y": 4, "origin": "top-left", "width": 16, "height": 12, "fill": "#E0A030"});
    for (key, value) in extra.as_object().expect("an object") {
        element[key] = value.clone();
    }
    element
}

/// An `x` sliding 40 px over the element's first 200 ms.
fn slide() -> Value {
    json!([{"t": 0, "v": 4}, {"t": 200, "v": 44, "ease": "ease-out"}])
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

fn parsed(element: &str) -> Result<Project, String> {
    let source = format!(
        r##"{{"frame":{{"width":1080,"height":1920}},"fps":25,"tracks":[{{"name":"t","layer":1,"elements":[{element}]}}]}}"##
    );
    serde_json::from_str::<Project>(&source).map_err(|e| e.to_string())
}

#[test]
fn a_moving_element_with_the_field_validates_clean_and_every_visual_type_takes_it() {
    let report = validated(&project(&[rect(
        "card",
        json!({"x": slide(), "motion_blur": {"shutter": 180, "samples": 8}}),
    )]));
    assert_eq!(report.exit_code(), ExitCode::Ok, "{:?}", codes(&report));
    assert!(report.findings.is_empty(), "{:?}", codes(&report));
    let blur = r#""motion_blur":{"shutter":360,"samples":32}"#;
    for element in [
        format!(
            r##"{{"id":"a","type":"ellipse","start":0,"end":1,"width":10,"height":10,"fill":"#000000",{blur}}}"##
        ),
        format!(
            r##"{{"id":"a","type":"text","start":0,"end":1,"width":10,"height":10,"font":"f","size":20,"runs":[{{"text":"hi"}}],{blur}}}"##
        ),
        format!(
            r##"{{"id":"a","type":"image","start":0,"end":1,"source":"a.png","width":10,"height":10,"fit":"literal",{blur}}}"##
        ),
        format!(
            r##"{{"id":"a","type":"video","start":0,"end":1,"source":"a.mp4","source_start":0,"source_end":1,"width":10,"height":10,"fit":"literal",{blur}}}"##
        ),
        format!(
            r##"{{"id":"a","type":"path","start":0,"end":1,"width":10,"height":10,"points":[{{"at":[0,0]}},{{"at":[10,10]}}],"closed":false,"stroke":"#000000","stroke_width":2,{blur}}}"##
        ),
    ] {
        parsed(&element).unwrap_or_else(|e| panic!("{element}: {e}"));
    }
}

#[test]
fn audio_and_transition_refuse_the_field() {
    for element in [
        r##"{"id":"a","type":"audio","start":0,"end":1,"source":"a.wav","source_start":0,"source_end":1,"motion_blur":{"shutter":180,"samples":8}}"##,
        r##"{"id":"a","type":"transition","start":0,"end":1,"kind":"crossfade","from":"b","to":"c","motion_blur":{"shutter":180,"samples":8}}"##,
    ] {
        let message = parsed(element).expect_err(element);
        assert!(message.contains("motion_blur"), "{message}");
    }
}

#[test]
fn a_missing_key_an_out_of_range_value_a_non_integer_or_a_keyframe_list_is_a_schema_error() {
    for blur in [
        json!({"shutter": 180}),
        json!({"samples": 8}),
        json!({}),
        json!({"shutter": 0, "samples": 8}),
        json!({"shutter": 361, "samples": 8}),
        json!({"shutter": 180, "samples": 1}),
        json!({"shutter": 180, "samples": 33}),
        json!({"shutter": 180.5, "samples": 8}),
        json!({"shutter": "180", "samples": 8}),
        json!({"shutter": [{"t": 0, "v": 90}, {"t": 200, "v": 180, "ease": "linear"}], "samples": 8}),
        json!({"shutter": 180, "samples": [{"t": 0, "v": 8}]}),
        json!({"shutter": 180, "samples": 8, "phase": 0}),
        json!(true),
    ] {
        let report = validated(&project(&[rect(
            "card",
            json!({"x": slide(), "motion_blur": blur}),
        )]));
        assert_eq!(report.exit_code(), ExitCode::Errors, "{blur}");
        assert!(
            codes(&report)
                .iter()
                .any(|code| code.starts_with("E-SCHEMA")),
            "{blur}: {:?}",
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

/// `element` with `motion_blur` removed.
fn without_blur(element: &Value) -> Value {
    let mut element = element.clone();
    element
        .as_object_mut()
        .expect("an object")
        .remove("motion_blur");
    element
}

#[test]
fn a_still_element_with_the_field_paints_the_bytes_it_paints_without_it() {
    // Unkeyed, turned, shadowed, masked and screened; and one whose keys all lie before its
    // life, so it is clamped still on every frame it is present.
    let stills = [
        rect(
            "turned",
            json!({"rotation": 17.5, "opacity": 0.8, "blend": "screen",
                   "effects": [{"name": "shadow", "dx": 2, "dy": 2, "radius": 3,
                                "color": "#2050FF", "opacity": 0.7},
                               {"name": "mask", "shape": "ellipse"}],
                   "motion_blur": {"shutter": 360, "samples": 32}}),
        ),
        rect(
            "settled",
            json!({"start": 300, "x": slide(), "width": [{"t": 0, "v": 4}, {"t": 120, "v": 20, "ease": "linear"}],
                   "motion_blur": {"shutter": 180, "samples": 8}}),
        ),
    ];
    let ground = rect(
        "ground",
        json!({"x": 0, "y": 0, "width": 64, "height": 48, "fill": "#406080"}),
    );
    let blurred = project(&[ground.clone(), stills[0].clone(), stills[1].clone()]);
    let sharp = project(&[ground, without_blur(&stills[0]), without_blur(&stills[1])]);
    for at in [0, 40, 200, 360] {
        assert!(
            painted(&blurred, at).as_raw() == painted(&sharp, at).as_raw(),
            "at {at}: a still element's bytes changed"
        );
    }
}

/// The grey level of the red channel at `(x, y)`.
fn red(picture: &image::RgbaImage, x: u32, y: u32) -> u8 {
    picture.get_pixel(x, y).0[0]
}

#[test]
fn a_moving_element_smears_along_its_motion_and_its_body_stays_solid() {
    // `x` moves 0.25 px per ms on a 200 px frame: 10 px a frame at 25 fps. A 360° shutter of
    // 4 samples paints at ±15 ms and ±5 ms around the frame instant, so at 200 ms the rect
    // (sharp at x 50..66) is painted from x 46.25 to x 53.75 at its left edge.
    let mut doc = project(&[rect(
        "card",
        json!({"x": [{"t": 0, "v": 0}, {"t": 400, "v": 100, "ease": "linear"}],
               "fill": "#FFFFFF", "motion_blur": {"shutter": 360, "samples": 4}}),
    )]);
    doc["frame"] = json!({"width": 200, "height": 48});
    let blurred = painted(&doc, 200);
    let mut sharp_doc = doc.clone();
    sharp_doc["tracks"][0]["elements"][0] = without_blur(&doc["tracks"][0]["elements"][0]);
    let sharp = painted(&sharp_doc, 200);

    assert_eq!(red(&sharp, 47, 10), 0, "the sharp rect starts at 50");
    let trailing = red(&blurred, 47, 10);
    assert!(
        0 < trailing && trailing < 255,
        "a pixel the rect reaches in some samples is partly covered: {trailing}"
    );
    assert_eq!(red(&blurred, 58, 10), 255, "every sample covers the body");
    assert_eq!(red(&blurred, 44, 10), 0, "no sample reaches x 44");
    // One sample covers x 47 (46.25..), so it holds a quarter of white.
    assert_eq!(trailing, 64);
}

#[test]
fn a_moving_blend_composites_the_average_once() {
    // A white rect screened over a mid-grey ground: where every sample covers it the screen
    // gives white; where none does, the ground. A blend applied per sample and then averaged
    // would give the same here, so the edge pixel tells them apart: one sample of four
    // screens white over grey, which averaged first is a quarter-white layer screened once.
    let mut doc = project(&[
        rect(
            "ground",
            json!({"x": 0, "y": 0, "width": 200, "height": 48, "fill": "#808080"}),
        ),
        rect(
            "card",
            json!({"x": [{"t": 0, "v": 0}, {"t": 400, "v": 100, "ease": "linear"}],
                   "fill": "#FFFFFF", "blend": "screen",
                   "motion_blur": {"shutter": 360, "samples": 4}}),
        ),
    ]);
    doc["frame"] = json!({"width": 200, "height": 48});
    let picture = painted(&doc, 200);
    assert_eq!(red(&picture, 58, 10), 255);
    assert_eq!(red(&picture, 44, 10), 128);
    // screen(64 over 128) = 64 + 128 − 64·128/255 ≈ 160.
    let edge = red(&picture, 47, 10);
    assert!((159..=161).contains(&edge), "{edge}");
}

// ---------------------------------------------------------------------------------------
// `R-MOTION-BLUR-STILL`.
// ---------------------------------------------------------------------------------------

const STILL: &str = "R-MOTION-BLUR-STILL";

fn blur() -> Value {
    json!({"shutter": 180, "samples": 8})
}

#[test]
fn an_unkeyed_element_carrying_the_field_is_a_review_naming_it() {
    let report = validated(&project(&[rect(
        "card",
        json!({"rotation": 30, "motion_blur": blur()}),
    )]));
    assert_eq!(codes(&report), [STILL]);
    let finding = &report.findings[0];
    assert_eq!(finding.class, montagent_core::finding::Class::Review);
    assert_eq!(finding.fields["element"], "card");
    assert_eq!(report.exit_code(), ExitCode::Ok, "a review gates nothing");
    let text =
        montagent_core::text::render(&report.to_json(), montagent_core::text::Options::default())
            .expect("the report renders");
    assert!(text.contains("motion_blur"), "{text}");
}

#[test]
fn motion_wholly_outside_the_elements_life_is_still() {
    // Every key lies before the element starts, so every value is clamped to the last.
    let report = validated(&project(&[rect(
        "card",
        json!({"start": 250, "x": slide(), "fill": [{"t": 0, "v": "#000000"},
               {"t": 100, "v": "#FFFFFF", "ease": "linear"}], "motion_blur": blur()}),
    )]));
    assert_eq!(codes(&report), [STILL]);
}

#[test]
fn motion_inside_the_life_keeps_it_silent_whichever_value_moves() {
    let moving = [
        json!({"x": slide()}),
        json!({"start": 150, "x": slide()}),
        json!({"fill": [{"t": 0, "v": "#000000"}, {"t": 300, "v": "#FFFFFF", "ease": "linear"}]}),
        json!({"effects": [{"name": "blur", "radius": [{"t": 100, "v": 0}, {"t": 300, "v": 4, "ease": "linear"}]}]}),
        json!({"fill": {"type": "linear", "angle": [{"t": 0, "v": 0}, {"t": 300, "v": 90, "ease": "linear"}],
               "stops": [{"offset": 0, "color": "#000000"}, {"offset": 1, "color": "#FFFFFF"}]}}),
        json!({"opacity": [{"t": 390, "v": 1}, {"t": 395, "v": 0, "ease": "linear"}]}),
    ];
    for extra in moving {
        let mut extra = extra;
        extra["motion_blur"] = blur();
        let report = validated(&project(&[rect("card", extra.clone())]));
        assert!(
            !codes(&report).contains(&STILL),
            "{extra}: {:?}",
            codes(&report)
        );
    }
}

#[test]
fn a_units_stagger_counts_as_motion() {
    let title = json!({"id": "title", "type": "text", "font": "titles", "size": 12,
        "runs": [{"text": "GO"}], "width": 60, "height": 20, "start": 0, "end": 400,
        "x": 32, "y": 24, "caption": false, "motion_blur": blur(),
        "units": {"by": "letter", "every": 50,
                  "y": [{"t": 0, "v": 10}, {"t": 150, "v": 0, "ease": "ease-out"}]}});
    let report = validated(&project(std::slice::from_ref(&title)));
    assert!(!codes(&report).contains(&STILL), "{:?}", codes(&report));

    // The same title with no stagger is still.
    let mut unstaggered = title;
    unstaggered
        .as_object_mut()
        .expect("an object")
        .remove("units");
    let report = validated(&project(&[unstaggered]));
    assert!(codes(&report).contains(&STILL), "{:?}", codes(&report));
}

#[test]
fn an_element_without_the_field_never_draws_it() {
    let report = validated(&project(&[rect("card", json!({}))]));
    assert!(!codes(&report).contains(&STILL), "{:?}", codes(&report));
}

// ---------------------------------------------------------------------------------------
// `query --at`.
// ---------------------------------------------------------------------------------------

#[track_caller]
fn queried(doc: &Value, at: i64) -> Value {
    use montagent_core::verbs::query::{Ask, query};
    let dir = tempdir(std::panic::Location::caller().line());
    let path = write_project(&dir, "p.montagent.json", &canonical(&doc.to_string()));
    let answer = query(
        &path,
        &Ask {
            at: Some(at),
            ..Ask::default()
        },
    );
    assert_eq!(answer.report().exit_code(), ExitCode::Ok);
    answer.to_json()
}

fn member<'a>(json: &'a Value, id: &str) -> &'a Value {
    json["query"]["stack"]
        .as_array()
        .expect("a stack")
        .iter()
        .find(|member| member["id"] == id)
        .unwrap_or_else(|| panic!("{id} is present"))
}

#[test]
fn query_at_reports_the_field_as_written_and_moving_mid_move_and_still_after_the_last_key() {
    let doc = project(&[
        rect("card", json!({"x": slide(), "motion_blur": blur()})),
        rect("plain", json!({"x": slide()})),
    ]);
    let mid = queried(&doc, 100);
    assert_eq!(member(&mid, "card")["motion_blur"], blur(), "as written");
    assert_eq!(member(&mid, "card")["motion"], "moving");
    assert_eq!(member(&mid, "plain").get("motion_blur"), None);
    assert_eq!(member(&mid, "plain").get("motion"), None);

    let after = queried(&doc, 300);
    assert_eq!(member(&after, "card")["motion"], "still");
    // The values stay those at the instant asked, not at a sample.
    let x = member(&after, "card")["values"]
        .as_array()
        .expect("values")
        .iter()
        .find(|value| value["property"] == "x")
        .expect("x is keyed")
        .clone();
    assert_eq!(x["value"], 44.0);

    let text = montagent_core::text::render(&mid, montagent_core::text::Options::default())
        .expect("the answer renders");
    let row = text
        .lines()
        .find(|line| line.split_whitespace().nth(1) == Some("card"))
        .unwrap_or_else(|| panic!("no row for card:\n{text}"));
    assert!(row.contains("motion_blur 180/8 moving"), "{text}");
}

#[test]
fn moving_or_still_is_decided_at_the_frame_containing_the_instant() {
    // At 25 fps frame 5 is painted at 200 ms and frame 6 at 240. The slide's last key is at
    // 200, so frame 5's earlier samples still see it moving, and frame 6 is still. 239 ms is
    // inside frame 5.
    let doc = project(&[rect("card", json!({"x": slide(), "motion_blur": blur()}))]);
    assert_eq!(member(&queried(&doc, 239), "card")["motion"], "moving");
    assert_eq!(member(&queried(&doc, 240), "card")["motion"], "still");
}

// ---------------------------------------------------------------------------------------
// `shift`.
// ---------------------------------------------------------------------------------------

#[test]
fn shift_splits_a_moving_element_and_both_halves_carry_the_field_unchanged() {
    use montagent_core::verbs::shift;

    // `card` slides through the cut at 100 ms: its keyframes split there, and the hold the
    // shift writes leaves a still stretch between the two halves of the move.
    let mut doc = project(&[rect(
        "card",
        json!({"x": slide(), "motion_blur": {"shutter": 360, "samples": 16}}),
    )]);
    doc.as_object_mut().expect("an object").remove("duration");
    let dir = tempdir(line!());
    let path = write_project(&dir, "p.montagent.json", &canonical(&doc.to_string()));
    let shifted = shift::shift(
        &path,
        &shift::Ask {
            at: 100,
            delta: 120,
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
    let card = &written["tracks"][0]["elements"][0];
    assert_eq!(card["end"], 520, "shift stretched it");
    let times: Vec<i64> = card["x"]
        .as_array()
        .expect("still keyed")
        .iter()
        .filter_map(|record| record["t"].as_i64())
        .collect();
    assert_eq!(times, [0, 100, 220, 320], "split at the cut");
    assert_eq!(
        card["motion_blur"],
        json!({"shutter": 360, "samples": 16}),
        "the field is carried unchanged"
    );

    // Both halves still move, so the field is still of use, and each half is painted moving.
    let report = validate(&path);
    assert!(!codes(&report).contains(&STILL), "{:?}", codes(&report));
    let doc: Value = written;
    assert_eq!(member(&queried(&doc, 40), "card")["motion"], "moving");
    assert_eq!(member(&queried(&doc, 160), "card")["motion"], "still");
    assert_eq!(member(&queried(&doc, 280), "card")["motion"], "moving");
}
