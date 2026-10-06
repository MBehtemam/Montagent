//! **Every numeric and colour effect parameter is animatable** (ADR-0146 §3, slice 2, #676):
//! written in place inside its member, resolved at the instant through the one resolving
//! function, and named by the member's position with its name in the text,
//! `effects[1].radius (blur)`.
//!
//! Asserted at the seams an agent meets: the model's parse (what is a schema error), the
//! `frame` verb's pixels, `query --at`, `validate`'s findings and `shift`'s edit. That every
//! tool carries every parameter is `tests/animatable_tools.rs`; byte identity across painters
//! and with the blur bounds hint on and off is `tests/painters.rs` and `tests/filter_bound.rs`.

use montagent_core::model::Effect;
use montagent_core::report::ExitCode;
use montagent_core::verbs::frame::{Ask, frame};
use serde_json::{Value, json};

mod common;
use common::{canonical, tempdir, write_project};

fn parse(effect: Value) -> Result<Effect, serde_json::Error> {
    serde_json::from_value::<Effect>(effect)
}

/// A project of `elements` on a 400×400 black frame at 25 fps; its path.
#[track_caller]
fn project(line: u32, elements: Vec<Value>) -> std::path::PathBuf {
    write_project(
        &tempdir(line),
        "p.montagent.json",
        &canonical(
            &json!({"frame": {"width": 400, "height": 400}, "fps": 25,
                    "background": "#000000",
                    "tracks": [{"name": "only", "layer": 0, "elements": elements}]})
            .to_string(),
        ),
    )
}

/// A 300×300 opaque white rect centred in the frame (it spans 50..350 on both axes), alive
/// 0..1000, carrying `effects`. White on black, so the red channel is its kept coverage.
fn plate(effects: Value) -> Value {
    json!({"id": "plate", "type": "rect", "start": 0, "end": 1000, "x": 200, "y": 200,
           "width": 300, "height": 300, "fill": "#FFFFFF", "effects": effects})
}

/// The exact pixels at `instant`, lossless.
#[track_caller]
fn painted_at(path: &std::path::Path, instant: i64) -> image::RgbaImage {
    let answer = frame(
        path,
        &Ask {
            at: Some(instant),
            full: true,
            png: true,
            ..Ask::default()
        },
    );
    image::load_from_memory(&answer.image().expect("a picture").bytes)
        .expect("the bytes decode")
        .to_rgba8()
}

fn red(picture: &image::RgbaImage, x: u32, y: u32) -> u8 {
    picture.get_pixel(x, y).0[0]
}

/// `query --at`'s resolved value of `property` on `plate`.
#[track_caller]
fn resolved(path: &std::path::Path, instant: i64, property: &str) -> Value {
    use montagent_core::verbs::query::{Ask, query};
    let answer = query(
        path,
        &Ask {
            at: Some(instant),
            ..Ask::default()
        },
    );
    let json = answer.to_json();
    json["query"]["stack"][0]["values"]
        .as_array()
        .expect("values")
        .iter()
        .find(|row| row["property"] == property)
        .unwrap_or_else(|| panic!("no `{property}` row: {json}"))["value"]
        .clone()
}

fn codes(path: &std::path::Path) -> Vec<(String, Value)> {
    montagent_core::validate(path)
        .findings
        .iter()
        .map(|finding| {
            (
                finding.code.to_string(),
                Value::Object(finding.fields.clone().into_iter().collect()),
            )
        })
        .collect()
}

// ---- schema ----------------------------------------------------------------------------

#[test]
fn every_numeric_and_colour_parameter_takes_a_keyframe_list_in_place() {
    let list =
        |a: Value, b: Value| json!([{"t": 0, "v": a}, {"t": 400, "v": b, "ease": "ease-out"}]);
    let n = || list(json!(0), json!(24));
    let f = || list(json!(0.0), json!(0.5));
    let c = || list(json!("#000000"), json!("#FF0000"));
    for effect in [
        json!({"name": "blur", "radius": n()}),
        json!({"name": "shadow", "dx": n(), "dy": n(), "radius": n(), "color": c(), "opacity": f()}),
        json!({"name": "mask", "shape": "rect", "x": n(), "y": n(), "width": n(), "height": n(),
               "radius": n()}),
        json!({"name": "tint", "color": c(), "amount": f()}),
        json!({"name": "saturation", "amount": f()}),
        json!({"name": "brightness", "amount": f()}),
        json!({"name": "contrast", "amount": f()}),
        json!({"name": "chroma", "color": c(), "tolerance": f(), "softness": f(), "spill": f()}),
    ] {
        parse(effect.clone()).unwrap_or_else(|e| panic!("{effect}: {e}"));
    }
}

#[test]
fn each_parameters_static_range_applies_to_every_keyframe_value() {
    let list = |b: Value| json!([{"t": 0, "v": 0}, {"t": 400, "v": b, "ease": "linear"}]);
    for (effect, says) in [
        // A negative mask size or radius is a schema error; `0` is legal.
        (
            json!({"name": "mask", "shape": "rect", "x": 0, "y": 0, "width": list(json!(-1)),
                   "height": 10}),
            "non-negative",
        ),
        (
            json!({"name": "mask", "shape": "rect", "x": 0, "y": 0, "width": 10,
                   "height": 10, "radius": list(json!(-2))}),
            "non-negative",
        ),
        (
            json!({"name": "mask", "shape": "rect", "x": 0, "y": 0, "width": -10,
                   "height": 10}),
            "non-negative",
        ),
        // `chroma`'s three scalars stay within `[0, 1]` in every record.
        (
            json!({"name": "chroma", "color": "#00FF00", "tolerance": list(json!(1.5)),
                   "softness": 0, "spill": 0}),
            "tolerance",
        ),
        // `chroma.color` stays six-digit in every record.
        (
            json!({"name": "chroma",
                   "color": [{"t": 0, "v": "#00FF00"}, {"t": 400, "v": "#00FF0080", "ease": "linear"}],
                   "tolerance": 0.2, "softness": 0, "spill": 0}),
            "#00FF0080",
        ),
    ] {
        let message = parse(effect.clone())
            .err()
            .unwrap_or_else(|| panic!("{effect} was accepted"))
            .to_string();
        assert!(message.contains(says), "{effect}: {message}");
    }
    // `0` is legal for a size, static and keyed.
    parse(
        json!({"name": "mask", "shape": "rect", "x": 0, "y": 0, "width": list(json!(0)),
                 "height": 0, "radius": 0}),
    )
    .expect("0 is a legal size");
}

#[test]
fn mask_shape_and_invert_stay_static() {
    for effect in [
        json!({"name": "mask", "shape": [{"t": 0, "v": "rect"}, {"t": 400, "v": "circle", "ease": "linear"}]}),
        json!({"name": "mask", "shape": "rect", "invert": [{"t": 0, "v": true}]}),
    ] {
        assert!(parse(effect.clone()).is_err(), "{effect}");
    }
}

#[test]
fn the_published_schema_states_the_bounds_on_every_record() {
    // The bound sits on the value type, so the static value and every record's `v` carry it.
    let schema = montagent_core::schema::generate();
    let defs = &schema["$defs"];
    assert_eq!(defs["Fraction"]["minimum"], json!(0.0));
    assert_eq!(defs["Fraction"]["maximum"], json!(1.0));
    assert_eq!(defs["ScreenColour"]["pattern"], "^#[0-9A-F]{6}$");
    let chroma = defs["Effect"]["oneOf"]
        .as_array()
        .unwrap()
        .iter()
        .find(|branch| branch.pointer("/properties/name/const") == Some(&json!("chroma")))
        .unwrap();
    assert_eq!(
        chroma["properties"]["tolerance"]["$ref"],
        "#/$defs/AnimatableFraction"
    );
    assert_eq!(
        chroma["properties"]["color"]["$ref"],
        "#/$defs/AnimatableScreenColour"
    );
    assert_eq!(
        defs["KeyframeFraction"]["properties"]["v"]["$ref"],
        "#/$defs/Fraction"
    );
}

// ---- the reveal ------------------------------------------------------------------------

#[test]
fn a_mask_keyed_from_width_zero_to_the_full_width_reveals_the_element() {
    // The reveal ADR-0025 promised: a `rect` mask whose `width` runs 0 → 300 over the
    // element's life. At the start it hides the whole element; halfway it keeps the left
    // half; at the end everything.
    let path = project(
        line!(),
        vec![plate(
            json!([{"name": "mask", "shape": "rect", "x": 0, "y": 0,
                            "width": [{"t": 0, "v": 0}, {"t": 800, "v": 300, "ease": "linear"}],
                            "height": 300}]),
        )],
    );
    assert!(
        codes(&path).iter().all(|(code, _)| !code.starts_with("E-")),
        "{:?}",
        codes(&path)
    );

    let start = painted_at(&path, 0);
    assert!(
        start.pixels().all(|pixel| pixel.0[0] == 0),
        "a mask of width 0 hides the whole element"
    );
    let half = painted_at(&path, 400);
    assert_eq!(
        red(&half, 60, 200),
        255,
        "the left of the element is revealed"
    );
    assert_eq!(red(&half, 190, 200), 255);
    assert_eq!(red(&half, 210, 200), 0, "the right is not yet");
    let end = painted_at(&path, 900);
    assert_eq!(
        red(&end, 340, 200),
        255,
        "at the end the whole element shows"
    );

    // `query --at` prints the resolved width, unrounded.
    assert_eq!(resolved(&path, 100, "effects[0].width (mask)"), json!(37.5));
}

#[test]
fn a_mask_that_hides_every_frame_is_never_painted_and_the_finding_names_the_mask() {
    let path = project(
        line!(),
        vec![plate(json!([{"name": "blur", "radius": 2},
                           {"name": "mask", "shape": "ellipse", "x": 0, "y": 0,
                            "width": [{"t": 0, "v": 0}, {"t": 2000, "v": 300, "ease": "step"}],
                            "height": 300}]))],
    );
    let found = codes(&path);
    let (_, fields) = found
        .iter()
        .find(|(code, _)| code == "E-NOT-PAINTED-NO-EXTENT")
        .unwrap_or_else(|| panic!("{found:?}"));
    let detail = fields["detail"].as_str().unwrap();
    assert!(detail.contains("`effects[1]`"), "{detail}");
    assert!(detail.contains("mask"), "{detail}");
    assert!(!detail.contains("the box"), "{detail}");
}

#[test]
fn a_mask_that_hides_some_frames_or_none_is_no_finding() {
    for effects in [
        // Hidden only at the first frames: a reveal.
        json!([{"name": "mask", "shape": "rect", "x": 0, "y": 0,
                "width": [{"t": 0, "v": 0}, {"t": 400, "v": 300, "ease": "linear"}],
                "height": 300}]),
        // Non-zero, but wholly outside the element: not counted (ADR-0146 §6).
        json!([{"name": "mask", "shape": "rect", "x": 900, "y": 900, "width": 10,
                "height": 10}]),
        // An inverted mask of no size keeps everything outside it, which is everything.
        json!([{"name": "mask", "shape": "rect", "x": 0, "y": 0, "width": 0, "height": 0,
                "invert": true}]),
    ] {
        let path = project(line!(), vec![plate(effects.clone())]);
        let found = codes(&path);
        assert!(
            found
                .iter()
                .all(|(code, _)| code != "E-NOT-PAINTED-NO-EXTENT"),
            "{effects}: {found:?}"
        );
    }
}

#[test]
fn a_static_zero_size_mask_hides_every_frame() {
    let path = project(
        line!(),
        vec![
            json!({"id": "plate", "type": "rect", "start": 0, "end": 1000, "x": 200,
                    "y": 200, "width": 300, "height": 300, "fill": "#FFFFFF",
                    "effects": [{"name": "mask", "shape": "rect", "x": 0, "y": 0,
                                 "width": 300, "height": 0}]}),
        ],
    );
    assert!(
        codes(&path)
            .iter()
            .any(|(code, _)| code == "E-NOT-PAINTED-NO-EXTENT")
    );
}

// ---- keyed parameters render, and `query --at` prints them resolved --------------------

#[test]
fn a_keyed_blur_radius_paints_its_resolved_value_and_query_prints_it() {
    let keyed = project(
        line!(),
        vec![plate(json!([{"name": "blur",
                            "radius": [{"t": 0, "v": 0}, {"t": 800, "v": 40, "ease": "linear"}]}]))],
    );
    let fixed = project(
        line!(),
        vec![plate(json!([{"name": "blur", "radius": 20}]))],
    );
    assert!(painted_at(&keyed, 400) == painted_at(&fixed, 400));
    assert!(painted_at(&keyed, 0) != painted_at(&keyed, 400));
    assert_eq!(
        resolved(&keyed, 400, "effects[0].radius (blur)"),
        json!(20.0)
    );
}

#[test]
fn a_keyed_shadow_colour_paints_its_resolved_value_and_query_prints_it() {
    let shadow = |color: Value| {
        plate(
            json!([{"name": "shadow", "dx": 20, "dy": 20, "radius": 0, "color": color,
                      "opacity": 1}]),
        )
    };
    let keyed = project(
        line!(),
        vec![shadow(
            json!([{"t": 0, "v": "#FF0000"}, {"t": 800, "v": "#0000FF", "ease": "linear"}]),
        )],
    );
    // Premultiplied sRGB, half way: #800080 (ADR-0146 §2).
    let fixed = project(line!(), vec![shadow(json!("#800080"))]);
    assert!(painted_at(&keyed, 400) == painted_at(&fixed, 400));
    let shadow_pixel = |picture: &image::RgbaImage| picture.get_pixel(360, 360).0;
    assert_eq!(shadow_pixel(&painted_at(&keyed, 0))[0], 255);
    assert_eq!(shadow_pixel(&painted_at(&keyed, 800))[2], 255);
    assert_eq!(
        resolved(&keyed, 400, "effects[0].color (shadow)"),
        json!("#800080")
    );
}

#[test]
fn a_keyed_chroma_tolerance_paints_its_resolved_value_and_query_prints_it() {
    // A green plate keyed against green: tolerance 0 keys nothing, and above the plate's own
    // distance it keys the plate out.
    let green = |tolerance: Value| {
        json!({"id": "plate", "type": "rect", "start": 0, "end": 1000, "x": 200, "y": 200,
               "width": 300, "height": 300, "fill": "#20C020",
               "effects": [{"name": "chroma", "color": "#00FF00", "tolerance": tolerance,
                            "softness": 0, "spill": 0}]})
    };
    let keyed = project(
        line!(),
        vec![green(
            json!([{"t": 0, "v": 0}, {"t": 800, "v": 0.8, "ease": "linear"}]),
        )],
    );
    let fixed = project(line!(), vec![green(json!(0.4))]);
    assert!(painted_at(&keyed, 400) == painted_at(&fixed, 400));
    assert!(
        painted_at(&keyed, 0).get_pixel(200, 200).0[1] > 100,
        "nothing keyed at 0"
    );
    assert_eq!(
        painted_at(&keyed, 800).get_pixel(200, 200).0[1],
        0,
        "keyed out at 0.8"
    );
    assert_eq!(
        resolved(&keyed, 400, "effects[0].tolerance (chroma)"),
        json!(0.4)
    );
}

#[test]
fn an_overshooting_parameter_clamps_to_its_range() {
    // An ease whose `y` leaves `[0, 1]` carries `tolerance` past 1 between the records; the
    // resolved value clamps to the range (ADR-0146 §5).
    let path = project(
        line!(),
        vec![
            json!({"id": "plate", "type": "rect", "start": 0, "end": 1000, "x": 200,
                    "y": 200, "width": 300, "height": 300, "fill": "#20C020",
                    "effects": [{"name": "chroma", "color": "#00FF00",
                                 "tolerance": [{"t": 0, "v": 0},
                                               {"t": 800, "v": 1, "ease": [0.3, 1.8, 0.6, 1.4]}],
                                 "softness": 0, "spill": 0}]}),
        ],
    );
    assert_eq!(
        resolved(&path, 500, "effects[0].tolerance (chroma)"),
        json!(1.0)
    );
}

// ---- two members of one name are told apart by position --------------------------------

#[test]
fn two_effects_of_one_name_are_told_apart_in_findings_by_position() {
    // The second blur's last record sits at 990: its plateau holds no frame of 25 fps
    // (R-KEYFRAME-UNREACHED), and the first blur is static, so nothing names it.
    let path = project(
        line!(),
        vec![plate(json!([{"name": "blur", "radius": 4},
                           {"name": "blur",
                            "radius": [{"t": 0, "v": 0}, {"t": 990, "v": 30, "ease": "linear"}]}]))],
    );
    let found = codes(&path);
    let properties: Vec<&Value> = found
        .iter()
        .filter(|(code, _)| code == "R-KEYFRAME-UNREACHED")
        .map(|(_, fields)| &fields["property"])
        .collect();
    assert_eq!(
        properties,
        [&json!("effects[1].radius (blur)")],
        "{found:?}"
    );
}

// ---- `shift` -----------------------------------------------------------------------------

#[track_caller]
fn shifted(path: &std::path::Path, at: i64, delta: i64) -> (ExitCode, Value) {
    use montagent_core::verbs::shift::{Ask, shift};
    let answer = shift(
        path,
        &Ask {
            at,
            delta,
            scope: None,
            release: Vec::new(),
        },
    );
    let written: Value =
        serde_json::from_str(&std::fs::read_to_string(path).expect("written")).expect("JSON");
    (
        answer.report().exit_code(),
        written["tracks"][0]["elements"][0]["effects"].clone(),
    )
}

fn pairs(list: &Value) -> Vec<(i64, Value)> {
    list.as_array()
        .expect("keyed")
        .iter()
        .map(|record| (record["t"].as_i64().unwrap(), record["v"].clone()))
        .collect()
}

#[test]
fn shift_moves_and_splits_a_keyed_effect_parameter_with_its_element() {
    let effects = json!([{"name": "mask", "shape": "rect", "x": 0, "y": 0,
                          "width": [{"t": 0, "v": 0}, {"t": 1000, "v": 25, "ease": "linear"}],
                          "height": 300},
                         {"name": "shadow", "dx": 4, "dy": 4,
                          "radius": [{"t": 0, "v": 0}, {"t": 1000, "v": 25, "ease": "linear"}],
                          "color": [{"t": 0, "v": "#FF0000"}, {"t": 1000, "v": "#0000FF", "ease": "linear"}],
                          "opacity": 1}]);
    let path = project(line!(), vec![plate(effects)]);
    let (code, effects) = shifted(&path, 500, 100);
    assert_ne!(code, ExitCode::Errors);
    // An integer mask value splits to the nearest integer, ties away from zero: 12.5 → 13.
    assert_eq!(
        pairs(&effects[0]["width"]),
        [
            (0, json!(0)),
            (500, json!(13)),
            (600, json!(13)),
            (1100, json!(25))
        ]
    );
    // A number-typed parameter splits exactly.
    assert_eq!(
        pairs(&effects[1]["radius"]),
        [
            (0, json!(0.0)),
            (500, json!(12.5)),
            (600, json!(12.5)),
            (1100, json!(25.0))
        ]
    );
    // A colour splits to bytes.
    assert_eq!(pairs(&effects[1]["color"])[1], (500, json!("#800080")));

    // A plain shift carries every record with its element.
    let path = project(
        line!(),
        vec![plate(json!([{"name": "tint", "color": "#FF0000",
                            "amount": [{"t": 100, "v": 0}, {"t": 900, "v": 1, "ease": "linear"}]}]))],
    );
    let (_, effects) = shifted(&path, 0, 300);
    assert_eq!(
        pairs(&effects[0]["amount"]),
        [(400, json!(0.0)), (1200, json!(1.0))]
    );
}

#[test]
fn shift_refuses_a_split_whose_value_leaves_the_parameters_range() {
    // The overshoot carries `tolerance` past 1 at the cut: writing 1 would bend both halves,
    // so the split is refused and names the parameter (ADR-0146 §7).
    let path = project(
        line!(),
        vec![
            json!({"id": "plate", "type": "rect", "start": 0, "end": 1000, "x": 200,
                    "y": 200, "width": 300, "height": 300, "fill": "#20C020",
                    "effects": [{"name": "chroma", "color": "#00FF00",
                                 "tolerance": [{"t": 0, "v": 0},
                                               {"t": 800, "v": 1, "ease": [0.3, 1.8, 0.6, 1.4]}],
                                 "softness": 0, "spill": 0}]}),
        ],
    );
    use montagent_core::verbs::shift::{Ask, shift};
    let answer = shift(
        &path,
        &Ask {
            at: 500,
            delta: 100,
            scope: None,
            release: Vec::new(),
        },
    );
    let refusal = answer
        .report()
        .findings
        .iter()
        .find(|finding| finding.code == "E-SHIFT-SPLIT-UNWRITABLE")
        .unwrap_or_else(|| panic!("{:?}", answer.report().findings));
    assert_eq!(
        refusal.fields["property"],
        json!("effects[0].tolerance (chroma)")
    );
}
