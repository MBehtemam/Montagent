//! ADR-0156's first three named effects (#724): `posterize`, `glow` and `directional_blur`.
//!
//! Asserted at the `frame` verb: a project file goes in, the pixels come back and are read
//! through a decoder that is not the one that wrote them. The expected values come from the
//! ADR's own worked numbers (an identity value paints the same bytes as no member; `levels: 2`
//! sends 0.49 to 0 and 0.5 to 1; a smear at `angle: 0` reaches `length / 2` each side), never
//! from re-running the painter's arithmetic.

use std::path::Path;

use montagent_core::model::Effect;
use montagent_core::report::ExitCode;
use montagent_core::verbs::frame::{Ask, frame};
use serde_json::{Value, json};

mod common;
use common::{canonical, tempdir, write_project};

/// One 400×400 frame at 500 ms, at true scale and lossless.
#[track_caller]
fn painted(line: u32, elements: &str) -> image::RgbaImage {
    let dir = tempdir(line);
    let project = write_project(
        &dir,
        "p.montagent.json",
        &canonical(&format!(
            r##"{{"frame":{{"width":400,"height":400}},"fps":25,"background":"#000000",
                "tracks":[{{"name":"only","layer":0,"elements":[{elements}]}}]}}"##
        )),
    );
    pixels_of(&project)
}

#[track_caller]
fn pixels_of(project: &Path) -> image::RgbaImage {
    let answer = frame(
        project,
        &Ask {
            at: Some(500),
            full: true,
            png: true,
            ..Ask::default()
        },
    );
    assert_eq!(
        answer.report().exit_code(),
        ExitCode::Ok,
        "frame did not answer: {}",
        serde_json::to_string_pretty(&answer.to_json()).unwrap_or_default()
    );
    image::load_from_memory(&answer.image().expect("a picture").bytes)
        .expect("the bytes decode")
        .to_rgba8()
}

/// A `width`×`height` rect of `fill`, centred on `(x, y)`, carrying `effects` verbatim, with
/// `extra` fields laid in (`"rotation": 20,` and the like).
fn rect(
    fill: &str,
    (x, y): (i64, i64),
    (width, height): (i64, i64),
    extra: &str,
    effects: &str,
) -> String {
    format!(
        r##"{{"id":"subject","type":"rect","start":0,"end":1000,"x":{x},"y":{y},
            "origin":"center","width":{width},"height":{height},"fill":"{fill}",{extra}
            "effects":[{effects}]}}"##
    )
}

/// A 100×100 square of `fill` in the middle of the frame (150..250 on both axes), carrying
/// `effects`.
fn square(fill: &str, effects: &str) -> String {
    rect(fill, (200, 200), (100, 100), "", effects)
}

fn differing(a: &image::RgbaImage, b: &image::RgbaImage) -> usize {
    a.pixels().zip(b.pixels()).filter(|(a, b)| a != b).count()
}

fn rgb(picture: &image::RgbaImage, x: u32, y: u32) -> [u8; 3] {
    let p = picture.get_pixel(x, y).0;
    [p[0], p[1], p[2]]
}

/// A horizontal ramp through all 256 byte values, so an identity has every value to keep.
fn ramp(effects: &str) -> String {
    format!(
        r##"{{"id":"ramp","type":"rect","start":0,"end":1000,"x":72,"y":100,
            "origin":"top-left","width":256,"height":200,
            "fill":{{"gradient":"linear","angle":90,"stops":[
                {{"offset":0,"color":"#000000"}},{{"offset":1,"color":"#FFFFFF"}}]}},
            "effects":[{effects}]}}"##
    )
}

// ---------------------------------------------------------------------------
// The schema (ADR-0156 §5)
// ---------------------------------------------------------------------------

fn parse(effect: Value) -> Result<Effect, String> {
    serde_json::from_value::<Effect>(effect).map_err(|e| e.to_string())
}

fn keyed(a: Value, b: Value) -> Value {
    json!([{"t": 0, "v": a}, {"t": 1000, "v": b, "ease": "linear"}])
}

#[test]
fn every_parameter_is_required_and_the_three_parse_as_written() {
    for effect in [
        json!({"name": "posterize", "levels": 4}),
        json!({"name": "glow", "threshold": 0.6, "radius": 12.5, "intensity": 1.5}),
        json!({"name": "directional_blur", "angle": -30, "length": 4096}),
    ] {
        parse(effect.clone()).unwrap_or_else(|e| panic!("{effect}: {e}"));
        for key in effect.as_object().unwrap().keys().filter(|k| *k != "name") {
            let mut missing = effect.clone();
            missing.as_object_mut().unwrap().remove(key);
            let error = parse(missing).expect_err(key);
            assert!(error.contains(&format!("`{key}`")), "{key}: {error}");
        }
    }
    assert!(
        parse(
            json!({"name": "glow", "threshold": 0.5, "radius": 4, "intensity": 1,
                     "color": "#FF0000"})
        )
        .expect_err("a glow takes no colour")
        .contains("unknown field `color`")
    );
}

#[test]
fn a_value_out_of_range_is_a_schema_error_naming_the_member_and_its_range() {
    let refused = [
        (
            json!({"name": "posterize", "levels": 1}),
            "`posterize`'s `levels`",
            "from `2` to `256`",
        ),
        (
            json!({"name": "posterize", "levels": 257}),
            "`posterize`'s `levels`",
            "from `2` to `256`",
        ),
        (
            json!({"name": "glow", "threshold": 1.2, "radius": 4, "intensity": 1}),
            "`glow`'s `threshold`",
            "from `0` to `1`",
        ),
        (
            json!({"name": "glow", "threshold": 0.5, "radius": -1, "intensity": 1}),
            "`glow`'s `radius`",
            "at least `0`",
        ),
        (
            json!({"name": "glow", "threshold": 0.5, "radius": 4, "intensity": 4.5}),
            "`glow`'s `intensity`",
            "from `0` to `4`",
        ),
        (
            json!({"name": "directional_blur", "angle": 0, "length": -2}),
            "`directional_blur`'s `length`",
            "from `0` to `4096`",
        ),
        (
            json!({"name": "directional_blur", "angle": 0, "length": 4097}),
            "`directional_blur`'s `length`",
            "from `0` to `4096`",
        ),
        // In a keyframe record as in a static value.
        (
            json!({"name": "posterize", "levels": keyed(json!(4), json!(300))}),
            "`posterize`'s `levels`",
            "every keyframe record",
        ),
    ];
    for (effect, names, range) in refused {
        let error = parse(effect.clone()).expect_err(&effect.to_string());
        assert!(
            error.contains(names) && error.contains(range),
            "{effect}: {error}"
        );
        assert!(error.contains("ADR-0156"), "{error}");
    }
    // `levels` is an integer.
    assert!(parse(json!({"name": "posterize", "levels": 4.5})).is_err());
}

#[test]
fn every_parameter_is_animatable_and_query_at_reports_what_each_resolves_to() {
    use montagent_core::verbs::query::{Ask, query};
    let dir = tempdir(line!());
    let element = json!({"id": "subject", "type": "rect", "start": 0, "end": 1000, "x": 200,
        "y": 200, "width": 100, "height": 100, "fill": "#F0E0A0", "effects": [
            {"name": "posterize", "levels": keyed(json!(2), json!(10))},
            {"name": "glow", "threshold": keyed(json!(0.2), json!(0.6)), "radius": 8,
             "intensity": keyed(json!(0), json!(2))},
            {"name": "directional_blur", "angle": keyed(json!(350), json!(10)),
             "length": keyed(json!(0), json!(30))}]});
    let project = write_project(
        &dir,
        "p.montagent.json",
        &canonical(
            &json!({"frame": {"width": 400, "height": 400}, "fps": 25,
                    "background": "#000000",
                    "tracks": [{"name": "only", "layer": 0, "elements": [element]}]})
            .to_string(),
        ),
    );
    let answer = query(
        &project,
        &Ask {
            at: Some(500),
            ..Ask::default()
        },
    )
    .to_json();
    let value = |property: &str| {
        answer["query"]["stack"][0]["values"]
            .as_array()
            .expect("values")
            .iter()
            .find(|row| row["property"] == property)
            .unwrap_or_else(|| panic!("no `{property}` row: {answer}"))["value"]
            .clone()
    };
    assert_eq!(value("effects[0].levels (posterize)"), json!(6.0));
    assert_eq!(value("effects[1].threshold (glow)"), json!(0.4));
    assert_eq!(value("effects[1].radius (glow)"), json!(8.0));
    assert_eq!(value("effects[1].intensity (glow)"), json!(1.0));
    // A keyed angle interpolates literally: 350 → 10 sweeps the long way round.
    assert_eq!(value("effects[2].angle (directional_blur)"), json!(180.0));
    assert_eq!(value("effects[2].length (directional_blur)"), json!(15.0));
    assert!(
        montagent_core::validate(&project)
            .findings
            .iter()
            .all(|finding| !finding.code.to_string().starts_with("E-")),
        "the keyed members validate"
    );
}

// ---------------------------------------------------------------------------
// posterize
// ---------------------------------------------------------------------------

#[test]
fn posterize_at_256_levels_paints_the_same_bytes_as_no_effect() {
    let bare = painted(line!(), &ramp(""));
    let posterized = painted(line!(), &ramp(r#"{"name":"posterize","levels":256}"#));
    assert_eq!(differing(&bare, &posterized), 0);
}

#[test]
fn posterize_at_2_levels_sends_just_under_half_to_0_and_half_to_1() {
    // 0x7D is 0.490 and 0x80 is 0.502: either side of the tie at 0.5.
    let two = r#"{"name":"posterize","levels":2}"#;
    let under = painted(line!(), &square("#7D7D7D", two));
    let over = painted(line!(), &square("#808080", two));
    assert_eq!(rgb(&under, 200, 200), [0x00; 3]);
    assert_eq!(rgb(&over, 200, 200), [0xFF; 3]);
}

#[test]
fn posterize_quantises_a_ramp_to_its_levels() {
    // Four levels: every channel value is one of 0, 1/3, 2/3 and 1, as bytes 0, 85, 170, 255.
    let four = painted(line!(), &ramp(r#"{"name":"posterize","levels":4}"#));
    let mut seen: Vec<u8> = (72..328).map(|x| rgb(&four, x, 200)[0]).collect();
    seen.dedup();
    assert_eq!(seen, vec![0, 85, 170, 255]);
}

// ---------------------------------------------------------------------------
// glow
// ---------------------------------------------------------------------------

/// A bright 120×60 bar on a dark ground, rotated, so a glow has edges to bloom past.
fn bar(effects: &str) -> String {
    rect(
        "#F0E0A0",
        (200, 200),
        (120, 60),
        r#""rotation": 20,"#,
        effects,
    )
}

#[test]
fn glow_with_an_empty_bright_pass_or_no_gain_paints_the_same_bytes_as_no_effect() {
    let bare = painted(line!(), &bar(""));
    for glow in [
        r#"{"name":"glow","threshold":1,"radius":20,"intensity":2}"#,
        r#"{"name":"glow","threshold":0.2,"radius":20,"intensity":0}"#,
    ] {
        assert_eq!(differing(&bare, &painted(line!(), &bar(glow))), 0, "{glow}");
    }
}

#[test]
fn glow_blooms_past_the_element_and_only_from_what_is_brighter_than_the_threshold() {
    let glow = r#"{"name":"glow","threshold":0.5,"radius":20,"intensity":1}"#;
    let bare = painted(line!(), &square("#F0E0A0", ""));
    let glowing = painted(line!(), &square("#F0E0A0", glow));
    // The square spans 150..250; 10 px outside it the frame was black and now is not.
    assert_eq!(rgb(&bare, 200, 140), [0; 3]);
    assert_ne!(
        rgb(&glowing, 200, 140),
        [0; 3],
        "the glow reaches past the edge"
    );
    // A dark square has nothing above the threshold, so nothing blooms.
    let dark = painted(line!(), &square("#402010", ""));
    let dark_glow = painted(line!(), &square("#402010", glow));
    assert_eq!(differing(&dark, &dark_glow), 0);
}

// ---------------------------------------------------------------------------
// directional_blur
// ---------------------------------------------------------------------------

/// A white line from `(x, y)`, `width`×`height`, top-left placed, carrying `effects`.
fn line(x: i64, y: i64, (width, height): (i64, i64), effects: &str) -> String {
    format!(
        r##"{{"id":"line","type":"rect","start":0,"end":1000,"x":{x},"y":{y},
            "origin":"top-left","width":{width},"height":{height},"fill":"#FFFFFF",
            "effects":[{effects}]}}"##
    )
}

#[test]
fn directional_blur_of_length_0_paints_the_same_bytes_as_no_effect() {
    let bare = painted(line!(), &bar(""));
    let smeared = painted(
        line!(),
        &bar(r#"{"name":"directional_blur","angle":30,"length":0}"#),
    );
    assert_eq!(differing(&bare, &smeared), 0);
}

#[test]
fn at_angle_0_a_one_pixel_vertical_line_spreads_only_sideways_by_half_the_length() {
    // A 1×100 white line on column 200, rows 150..250, smeared 20 px at 0°.
    let bare = painted(line!(), &line(200, 150, (1, 100), ""));
    assert_ne!(rgb(&bare, 200, 200), [0; 3]);
    assert_eq!(rgb(&bare, 199, 200), [0; 3]);
    let smeared = painted(
        line!(),
        &line(
            200,
            150,
            (1, 100),
            r#"{"name":"directional_blur","angle":0,"length":20}"#,
        ),
    );
    let lit = |x: u32, y: u32| rgb(&smeared, x, y) != [0; 3];
    for y in [150, 200, 249] {
        assert!(
            lit(190, y) && lit(210, y),
            "reaches 10 px each side on row {y}"
        );
        assert!(!lit(189, y) && !lit(211, y), "and no further, on row {y}");
    }
    for x in 185..216 {
        assert!(
            !lit(x, 149) && !lit(x, 250),
            "nothing above or below, column {x}"
        );
    }
}

#[test]
fn the_angle_is_measured_in_the_elements_own_space_so_it_turns_with_rotation() {
    // The vertical 1×100 line turned 90° about its centre lies across the frame; `angle: 0`
    // smears along the element's own x, which is now the frame's y.
    let turned = rect(
        "#FFFFFF",
        (200, 200),
        (1, 100),
        r#""rotation": 90,"#,
        r#"{"name":"directional_blur","angle":0,"length":20}"#,
    );
    let smeared = painted(line!(), &turned);
    let lit = |x: u32, y: u32| rgb(&smeared, x, y) != [0; 3];
    assert!(lit(200, 192) && lit(200, 208), "smeared up and down");
    assert!(!lit(200, 185) && !lit(200, 215), "by half the length");
    assert!(
        !lit(140, 200) && !lit(260, 200),
        "and not along the line's ends"
    );
}

#[test]
fn the_length_is_in_element_pixels_so_it_grows_with_scale() {
    // At `scale: [2, 1]` the 20-px smear reaches 20 frame pixels each side, not 10.
    let scaled = rect(
        "#FFFFFF",
        (200, 200),
        (1, 100),
        r#""scale": [2, 1],"#,
        r#"{"name":"directional_blur","angle":0,"length":20}"#,
    );
    let smeared = painted(line!(), &scaled);
    let lit = |x: u32, y: u32| rgb(&smeared, x, y) != [0; 3];
    assert!(lit(182, 200) && lit(217, 200), "past 10 px each side");
    assert!(!lit(176, 200) && !lit(224, 200), "and within 20 or so");
}

#[test]
fn a_directional_blur_at_90_degrees_smears_up_and_down() {
    let smeared = painted(
        line!(),
        &line(
            150,
            200,
            (100, 1),
            r#"{"name":"directional_blur","angle":90,"length":20}"#,
        ),
    );
    let lit = |x: u32, y: u32| rgb(&smeared, x, y) != [0; 3];
    assert!(lit(200, 190) && lit(200, 210));
    assert!(!lit(200, 189) && !lit(200, 211));
    assert!(!lit(149, 200) && !lit(250, 200));
}

// ---------------------------------------------------------------------------
// ADR-0225 (#1100): mosaic, radial_blur, find_edges, emboss, sharpen,
// twirl, bulge, optics_compensation, and offset
// ---------------------------------------------------------------------------

#[test]
fn adr_0225_effects_require_every_parameter_and_reject_unknown_fields() {
    for (effect, required_keys) in [
        (json!({"name": "mosaic", "blocks": [8, 8]}), &["blocks"][..]),
        (
            json!({"name": "radial_blur", "kind": "spin", "center": [0.5, 0.5], "amount": 20}),
            &["kind", "amount"][..],
        ),
        (
            json!({"name": "radial_blur", "kind": "zoom", "amount": 50}),
            &["kind", "amount"][..],
        ),
        (json!({"name": "find_edges", "invert": false}), &[][..]),
        (
            json!({"name": "emboss", "angle": 135, "height": 2, "contrast": 1.5}),
            &["angle", "height"][..],
        ),
        (json!({"name": "sharpen", "amount": 1.5}), &["amount"][..]),
        (
            json!({"name": "twirl", "center": [0.5, 0.5], "radius": 0.5, "angle": 90}),
            &["radius", "angle"][..],
        ),
        (
            json!({"name": "bulge", "center": [0.5, 0.5], "radius": 0.5, "height": 0.5}),
            &["radius", "height"][..],
        ),
        (
            json!({"name": "optics_compensation", "fov": 60, "reverse": false}),
            &["fov"][..],
        ),
        (
            json!({"name": "offset", "shift_x": 10, "shift_y": -20, "wrap": true}),
            &["shift_x", "shift_y"][..],
        ),
    ] {
        parse(effect.clone()).unwrap_or_else(|e| panic!("{effect}: {e}"));
        for key in required_keys {
            let mut missing = effect.clone();
            missing.as_object_mut().unwrap().remove(*key);
            let error = parse(missing).expect_err(key);
            assert!(error.contains(&format!("`{key}`")), "{key}: {error}");
        }
        let mut extra = effect.clone();
        extra
            .as_object_mut()
            .unwrap()
            .insert("unexpected".into(), json!(1));
        assert!(
            parse(extra)
                .expect_err("unknown field")
                .contains("unknown field `unexpected`")
        );
    }
}

#[test]
fn adr_0225_out_of_range_values_cite_the_member_range_and_adr_0225() {
    let refused = [
        (
            json!({"name": "mosaic", "blocks": [0, 4]}),
            "`mosaic`'s `blocks`",
            "at least `1`",
        ),
        (
            json!({"name": "mosaic", "blocks": [4, -1]}),
            "`mosaic`'s `blocks`",
            "at least `1`",
        ),
        (
            json!({"name": "radial_blur", "kind": "spin", "center": [-0.1, 0.5], "amount": 10}),
            "`radial_blur`'s `center`",
            "from `0` to `1`",
        ),
        (
            json!({"name": "radial_blur", "kind": "zoom", "center": [0.5, 1.1], "amount": 10}),
            "`radial_blur`'s `center`",
            "from `0` to `1`",
        ),
        (
            json!({"name": "radial_blur", "kind": "spin", "center": [0.5, 0.5], "amount": -1}),
            "`radial_blur`'s `amount`",
            "from `0` to `100`",
        ),
        (
            json!({"name": "radial_blur", "kind": "spin", "center": [0.5, 0.5], "amount": 101}),
            "`radial_blur`'s `amount`",
            "from `0` to `100`",
        ),
        (
            json!({"name": "emboss", "angle": 0, "height": -0.5, "contrast": 1}),
            "`emboss`'s `height`",
            "from `0` to `10`",
        ),
        (
            json!({"name": "emboss", "angle": 0, "height": 10.5, "contrast": 1}),
            "`emboss`'s `height`",
            "from `0` to `10`",
        ),
        (
            json!({"name": "emboss", "angle": 0, "height": 2, "contrast": -0.5}),
            "`emboss`'s `contrast`",
            "from `0` to `5`",
        ),
        (
            json!({"name": "emboss", "angle": 0, "height": 2, "contrast": 5.5}),
            "`emboss`'s `contrast`",
            "from `0` to `5`",
        ),
        (
            json!({"name": "sharpen", "amount": -0.5}),
            "`sharpen`'s `amount`",
            "from `0` to `5`",
        ),
        (
            json!({"name": "sharpen", "amount": 5.5}),
            "`sharpen`'s `amount`",
            "from `0` to `5`",
        ),
        (
            json!({"name": "twirl", "center": [1.5, 0.5], "radius": 0.5, "angle": 45}),
            "`twirl`'s `center`",
            "from `0` to `1`",
        ),
        (
            json!({"name": "twirl", "center": [0.5, 0.5], "radius": 0, "angle": 45}),
            "`twirl`'s `radius`",
            "strictly greater than `0`",
        ),
        (
            json!({"name": "twirl", "center": [0.5, 0.5], "radius": -0.5, "angle": 45}),
            "`twirl`'s `radius`",
            "strictly greater than `0`",
        ),
        (
            json!({"name": "bulge", "center": [0.5, -0.2], "radius": 0.5, "height": 0.5}),
            "`bulge`'s `center`",
            "from `0` to `1`",
        ),
        (
            json!({"name": "bulge", "center": [0.5, 0.5], "radius": 0, "height": 0.5}),
            "`bulge`'s `radius`",
            "strictly greater than `0`",
        ),
        (
            json!({"name": "bulge", "center": [0.5, 0.5], "radius": 0.5, "height": -1.5}),
            "`bulge`'s `height`",
            "from `-1` to `1`",
        ),
        (
            json!({"name": "bulge", "center": [0.5, 0.5], "radius": 0.5, "height": 1.5}),
            "`bulge`'s `height`",
            "from `-1` to `1`",
        ),
        (
            json!({"name": "optics_compensation", "fov": -1, "reverse": false}),
            "`optics_compensation`'s `fov`",
            "from `0` to `180`",
        ),
        (
            json!({"name": "optics_compensation", "fov": 181, "reverse": false}),
            "`optics_compensation`'s `fov`",
            "from `0` to `180`",
        ),
        // Keyframe records are checked against the same bounds.
        (
            json!({"name": "mosaic", "blocks": keyed(json!([2, 2]), json!([0, 4]))}),
            "`mosaic`'s `blocks`",
            "every keyframe record",
        ),
        (
            json!({"name": "optics_compensation", "fov": keyed(json!(30), json!(185)), "reverse": false}),
            "`optics_compensation`'s `fov`",
            "every keyframe record",
        ),
    ];
    for (effect, names, range) in refused {
        let error = parse(effect.clone()).expect_err(&effect.to_string());
        assert!(
            error.contains(names) && error.contains(range),
            "{effect}: {error}"
        );
        assert!(error.contains("ADR-0225"), "{error}");
        assert!(error.contains("E-OUT-OF-RANGE"), "{error}");
    }
}

#[test]
fn adr_0225_animatable_parameters_resolve_in_query_at_and_validate() {
    use montagent_core::verbs::query::{Ask, query};
    let dir = tempdir(line!());
    let element = json!({
        "id": "subject", "type": "rect", "start": 0, "end": 1000,
        "x": 200, "y": 200, "width": 100, "height": 100, "fill": "#F0E0A0",
        "effects": [
            {"name": "mosaic", "blocks": keyed(json!([2, 4]), json!([10, 20]))},
            {"name": "radial_blur", "kind": "spin",
             "center": keyed(json!([0.2, 0.4]), json!([0.8, 0.6])),
             "amount": keyed(json!(10), json!(50))},
            {"name": "emboss", "angle": keyed(json!(0), json!(90)),
             "height": keyed(json!(1), json!(5)),
             "contrast": keyed(json!(0.5), json!(2.5))},
            {"name": "sharpen", "amount": keyed(json!(0), json!(4))},
            {"name": "twirl", "center": keyed(json!([0.3, 0.3]), json!([0.7, 0.7])),
             "radius": keyed(json!(0.4), json!(1.2)),
             "angle": keyed(json!(-90), json!(90))},
            {"name": "bulge", "center": keyed(json!([0.4, 0.6]), json!([0.6, 0.4])),
             "radius": keyed(json!(0.5), json!(1.5)),
             "height": keyed(json!(-1), json!(1))},
            {"name": "optics_compensation",
             "fov": keyed(json!(20), json!(100)), "reverse": false},
            {"name": "offset",
             "shift_x": keyed(json!(-10), json!(30)),
             "shift_y": keyed(json!(20), json!(-40)),
             "wrap": true}
        ]
    });
    let project = write_project(
        &dir,
        "p.montagent.json",
        &canonical(
            &json!({"frame": {"width": 400, "height": 400}, "fps": 25,
                    "background": "#000000",
                    "tracks": [{"name": "only", "layer": 0, "elements": [element]}]})
            .to_string(),
        ),
    );
    let answer = query(
        &project,
        &Ask {
            at: Some(500),
            ..Ask::default()
        },
    )
    .to_json();
    let value = |property: &str| {
        answer["query"]["stack"][0]["values"]
            .as_array()
            .expect("values")
            .iter()
            .find(|row| row["property"] == property)
            .unwrap_or_else(|| panic!("no `{property}` row: {answer}"))["value"]
            .clone()
    };
    assert_eq!(value("effects[0].blocks (mosaic)"), json!([6.0, 12.0]));
    assert_eq!(value("effects[1].center (radial_blur)"), json!([0.5, 0.5]));
    assert_eq!(value("effects[1].amount (radial_blur)"), json!(30.0));
    assert_eq!(value("effects[2].angle (emboss)"), json!(45.0));
    assert_eq!(value("effects[2].height (emboss)"), json!(3.0));
    assert_eq!(value("effects[2].contrast (emboss)"), json!(1.5));
    assert_eq!(value("effects[3].amount (sharpen)"), json!(2.0));
    assert_eq!(value("effects[4].center (twirl)"), json!([0.5, 0.5]));
    assert_eq!(value("effects[4].radius (twirl)"), json!(0.8));
    assert_eq!(value("effects[4].angle (twirl)"), json!(0.0));
    assert_eq!(value("effects[5].center (bulge)"), json!([0.5, 0.5]));
    assert_eq!(value("effects[5].radius (bulge)"), json!(1.0));
    assert_eq!(value("effects[5].height (bulge)"), json!(0.0));
    assert_eq!(value("effects[6].fov (optics_compensation)"), json!(60.0));
    assert_eq!(value("effects[7].shift_x (offset)"), json!(10.0));
    assert_eq!(value("effects[7].shift_y (offset)"), json!(-10.0));
    assert!(
        montagent_core::validate(&project)
            .findings
            .iter()
            .all(|finding| !finding.code.to_string().starts_with("E-")),
        "the keyed ADR-0225 members validate"
    );
}

#[test]
fn adr_0225_identity_parameters_paint_the_same_bytes_as_no_effect() {
    let bare = painted(line!(), &ramp(""));
    for identity in [
        r#"{"name":"mosaic","blocks":[256,200]}"#,
        r#"{"name":"radial_blur","kind":"spin","center":[0.5,0.5],"amount":0}"#,
        r#"{"name":"radial_blur","kind":"zoom","center":[0.5,0.5],"amount":0}"#,
        r#"{"name":"emboss","angle":135,"height":0,"contrast":1}"#,
        r#"{"name":"emboss","angle":135,"height":2,"contrast":0}"#,
        r#"{"name":"sharpen","amount":0}"#,
        r#"{"name":"twirl","center":[0.5,0.5],"radius":0.5,"angle":0}"#,
        r#"{"name":"bulge","center":[0.5,0.5],"radius":0.5,"height":0}"#,
        r#"{"name":"optics_compensation","fov":0,"reverse":false}"#,
        r#"{"name":"optics_compensation","fov":0,"reverse":true}"#,
        r#"{"name":"offset","shift_x":0,"shift_y":0,"wrap":false}"#,
        r#"{"name":"offset","shift_x":0,"shift_y":0,"wrap":true}"#,
    ] {
        assert_eq!(
            differing(&bare, &painted(line!(), &ramp(identity))),
            0,
            "{identity}"
        );
    }
}

#[test]
fn mosaic_quantises_into_flat_blocks_and_steps_at_block_boundaries() {
    // `ramp` is at x: 72..328 (width 256), y: 100..300 (height 200).
    // With 4 horizontal blocks, each block is 64 px wide: 72..136, 136..200, 200..264, 264..328.
    let pixelated = painted(
        line!(),
        &ramp(r#"{"name":"mosaic","blocks":[4,2]}"#),
    );
    // Inside the first block (x = 80..130), every pixel is identical.
    let first_block = rgb(&pixelated, 80, 150)[0];
    for x in 80..130 {
        assert_eq!(rgb(&pixelated, x, 150)[0], first_block, "flat block at x={x}");
    }
    let second_block = rgb(&pixelated, 150, 150)[0];
    assert!(second_block > first_block + 40, "next block steps up");
}

#[test]
fn radial_blur_spin_and_zoom_smear_off_centre_features() {
    // A small bright dot above the centre of a 100×100 black-filled element:
    // we use a radial gradient with a narrow white disc near [0.5, 0.2].
    let dot_box = |effects: &str| {
        format!(
            r##"{{"id":"box","type":"rect","start":0,"end":1000,"x":150,"y":150,
                "origin":"top-left","width":100,"height":100,
                "fill":{{"gradient":"radial","center":[0.5,0.2],"radius":0.08,"stops":[
                    {{"offset":0,"color":"#FFFFFF"}},
                    {{"offset":1,"color":"#FFFFFF"}},
                    {{"offset":1,"color":"#000000"}}]}},
                "effects":[{effects}]}}"##
        )
    };
    let bare = painted(line!(), &dot_box(""));
    // At (200, 170) the dot is bright; sideways at (220, 175) and inward at (200, 190) it is black.
    assert_ne!(rgb(&bare, 200, 170), [0; 3]);
    assert_eq!(rgb(&bare, 220, 175), [0; 3]);
    assert_eq!(rgb(&bare, 200, 190), [0; 3]);

    // Spin about [0.5, 0.5] smears the dot along the arc through (220, 175) but not inward to (200, 200).
    let spun = painted(
        line!(),
        &dot_box(r#"{"name":"radial_blur","kind":"spin","center":[0.5,0.5],"amount":60}"#),
    );
    assert_ne!(
        rgb(&spun, 215, 174),
        [0; 3],
        "spin smears along the circle"
    );
    assert_eq!(
        rgb(&spun, 200, 200),
        [0; 3],
        "spin does not smear through the centre"
    );

    // Zoom about [0.5, 0.5] smears the dot radially along x = 200 toward the centre (200, 185).
    let zoomed = painted(
        line!(),
        &dot_box(r#"{"name":"radial_blur","kind":"zoom","center":[0.5,0.5],"amount":80}"#),
    );
    assert_ne!(
        rgb(&zoomed, 200, 185),
        [0; 3],
        "zoom smears radially toward the centre"
    );
}

#[test]
fn find_edges_emboss_and_sharpen_clamp_at_element_boundaries_and_respond_to_edges() {
    // A 100×100 square split vertically at x = 200: left half dark (#202020), right half bright (#E0E0E0).
    let split_square = |effects: &str| {
        format!(
            r##"{{"id":"split","type":"rect","start":0,"end":1000,"x":150,"y":150,
                "origin":"top-left","width":100,"height":100,
                "fill":{{"gradient":"linear","angle":90,"stops":[
                    {{"offset":0,"color":"#202020"}},
                    {{"offset":0.5,"color":"#202020"}},
                    {{"offset":0.5,"color":"#E0E0E0"}},
                    {{"offset":1,"color":"#E0E0E0"}}]}},
                "effects":[{effects}]}}"##
        )
    };

    // 1. find_edges: flat regions are black (invert: false) or white (invert: true); the step at x = 200 is opposite.
    let edges = painted(line!(), &split_square(r#"{"name":"find_edges","invert":false}"#));
    assert_eq!(rgb(&edges, 170, 200), [0; 3], "flat left half has zero Sobel gradient");
    assert_eq!(rgb(&edges, 230, 200), [0; 3], "flat right half has zero Sobel gradient");
    assert!(
        rgb(&edges, 200, 200)[0] > 200,
        "vertical step edge has strong Sobel gradient"
    );
    // Edge-clamped sampling (ADR-0225 §1): the outer pixel (150, 200) samples clamped [0, W]x[0, H],
    // so there is NO artificial edge against transparent black at the element boundary!
    assert_eq!(
        rgb(&edges, 150, 200),
        [0; 3],
        "element boundary does not trigger a false Sobel edge"
    );

    let edges_inv = painted(line!(), &split_square(r#"{"name":"find_edges","invert":true}"#));
    assert_eq!(rgb(&edges_inv, 170, 200), [255; 3], "inverted flat region is white");
    assert!(rgb(&edges_inv, 200, 200)[0] < 55, "inverted step edge is dark");

    // 2. emboss: flat region blends with neutral grey; step edge lights up or darkens with angle.
    let embossed_right = painted(
        line!(),
        &split_square(r#"{"name":"emboss","angle":0,"height":2,"contrast":2}"#),
    );
    let embossed_left = painted(
        line!(),
        &split_square(r#"{"name":"emboss","angle":180,"height":2,"contrast":2}"#),
    );
    assert!(
        u16::from(rgb(&embossed_right, 200, 200)[0])
            > u16::from(rgb(&embossed_left, 200, 200)[0]) + 80,
        "reversing emboss angle flips the relief highlight at the step edge"
    );

    // 3. sharpen: a soft ramp across x = 200 becomes steeper at the transition and stays flat away from it.
    let soft_split = |effects: &str| {
        format!(
            r##"{{"id":"soft","type":"rect","start":0,"end":1000,"x":150,"y":150,
                "origin":"top-left","width":100,"height":100,
                "fill":{{"gradient":"linear","angle":90,"stops":[
                    {{"offset":0,"color":"#404040"}},
                    {{"offset":0.48,"color":"#404040"}},
                    {{"offset":0.52,"color":"#C0C0C0"}},
                    {{"offset":1,"color":"#C0C0C0"}}]}},
                "effects":[{effects}]}}"##
        )
    };
    let unsharpened = painted(line!(), &soft_split(""));
    let sharpened = painted(line!(), &soft_split(r#"{"name":"sharpen","amount":4}"#));
    // Just to the dark side of the knee (x = 198) sharpen pulls darker; just to the bright side (x = 202) brighter.
    assert!(
        rgb(&sharpened, 198, 200)[0] < rgb(&unsharpened, 198, 200)[0],
        "sharpen darkens the dark knee"
    );
    assert!(
        rgb(&sharpened, 202, 200)[0] > rgb(&unsharpened, 202, 200)[0],
        "sharpen brightens the bright knee"
    );
    // And the element's outer edge (150, 200) does not get a halo from outside the box.
    assert_eq!(rgb(&sharpened, 150, 200), rgb(&unsharpened, 150, 200));
}

#[test]
fn twirl_bulge_and_optics_compensation_warp_interior_coordinates_and_clamp_edges() {
    // A 100×100 box at (150, 150) split horizontally at y = 200: top half red (#FF0000), bottom half blue (#0000FF).
    let two_tone = |effects: &str| {
        format!(
            r##"{{"id":"box","type":"rect","start":0,"end":1000,"x":150,"y":150,
                "origin":"top-left","width":100,"height":100,
                "fill":{{"gradient":"linear","angle":180,"stops":[
                    {{"offset":0,"color":"#FF0000"}},
                    {{"offset":0.5,"color":"#FF0000"}},
                    {{"offset":0.5,"color":"#0000FF"}},
                    {{"offset":1,"color":"#0000FF"}}]}},
                "effects":[{effects}]}}"##
        )
    };
    let bare = painted(line!(), &two_tone(""));
    // At (210, 195) (right of centre, just above midline) the bare box is pure red; at (190, 205) pure blue.
    assert_eq!(rgb(&bare, 210, 195), [255, 0, 0]);
    assert_eq!(rgb(&bare, 190, 205), [0, 0, 255]);

    // 1. twirl: a 180° twist within radius 0.4 swaps the top and bottom halves near the centre,
    // while corners outside radius 0.4 (40 px) are untouched.
    let twirled = painted(
        line!(),
        &two_tone(r#"{"name":"twirl","center":[0.5,0.5],"radius":0.4,"angle":180}"#),
    );
    assert_eq!(
        rgb(&twirled, 200, 195),
        [0, 0, 255],
        "5 px above centre has twisted 180° to sample the blue half"
    );
    assert_eq!(
        rgb(&twirled, 155, 155),
        [255, 0, 0],
        "corner outside radius 0.4 is untouched"
    );

    // 2. bulge: a disc near the centre gets pushed outward when height > 0 and pulled inward when height < 0.
    let stripe_box = |effects: &str| {
        format!(
            r##"{{"id":"stripe","type":"rect","start":0,"end":1000,"x":150,"y":150,
                "origin":"top-left","width":100,"height":100,
                "fill":{{"gradient":"radial","center":[0.5,0.5],"radius":0.6,"stops":[
                    {{"offset":0,"color":"#FFFFFF"}},
                    {{"offset":0.5,"color":"#FFFFFF"}},
                    {{"offset":0.5,"color":"#202020"}},
                    {{"offset":1,"color":"#202020"}}]}},
                "effects":[{effects}]}}"##
        )
    };
    let bare_disc = painted(line!(), &stripe_box(""));
    // The disc has radius 0.5 * 0.6 * √0.5 * 100 ≈ 21.2 px, so at 25 px from centre (225, 200) it is dark (#202020),
    // and at 15 px from centre (215, 200) it is white (#FFFFFF).
    assert_eq!(rgb(&bare_disc, 225, 200), [0x20; 3]);
    assert_eq!(rgb(&bare_disc, 215, 200), [0xFF; 3]);

    let bulged_out = painted(
        line!(),
        &stripe_box(r#"{"name":"bulge","center":[0.5,0.5],"radius":0.5,"height":1}"#),
    );
    assert_eq!(
        rgb(&bulged_out, 225, 200),
        [0xFF; 3],
        "positive bulge magnifies the white disc outward to r=25"
    );

    let pinched_in = painted(
        line!(),
        &stripe_box(r#"{"name":"bulge","center":[0.5,0.5],"radius":0.5,"height":-1}"#),
    );
    assert_eq!(
        rgb(&pinched_in, 215, 200),
        [0x20; 3],
        "negative bulge pinches the dark exterior inward to r=15"
    );

    // 3. optics_compensation: reverse: false maps r' = r*(1 + k*u^2) (pulls exterior inward),
    // reverse: true maps r' = r/(1 + k*u^2) (pushes interior outward), and edge clamping keeps
    // the corners opaque rather than pulling in transparent black from outside the box.
    let wide_disc = |effects: &str| {
        format!(
            r##"{{"id":"wide","type":"rect","start":0,"end":1000,"x":150,"y":150,
                "origin":"top-left","width":100,"height":100,
                "fill":{{"gradient":"radial","center":[0.5,0.5],"radius":1.2,"stops":[
                    {{"offset":0,"color":"#FFFFFF"}},
                    {{"offset":0.5,"color":"#FFFFFF"}},
                    {{"offset":0.5,"color":"#202020"}},
                    {{"offset":1,"color":"#202020"}}]}},
                "effects":[{effects}]}}"##
        )
    };
    let bare_wide = painted(line!(), &wide_disc(""));
    assert_eq!(rgb(&bare_wide, 240, 200), [0xFF; 3]);
    assert_eq!(rgb(&bare_wide, 245, 200), [0x20; 3]);

    let optics_forward = painted(
        line!(),
        &wide_disc(r#"{"name":"optics_compensation","fov":180,"reverse":false}"#),
    );
    let optics_reverse = painted(
        line!(),
        &wide_disc(r#"{"name":"optics_compensation","fov":180,"reverse":true}"#),
    );
    assert_eq!(
        rgb(&optics_forward, 245, 200),
        [0xFF; 3],
        "reverse: false (barrel) pushes the white disc outward to r=45"
    );
    assert_eq!(
        rgb(&optics_reverse, 240, 200),
        [0x20; 3],
        "reverse: true (pincushion) pulls the dark exterior inward to r=40"
    );
    assert_eq!(
        rgb(&optics_reverse, 152, 152),
        [0x20; 3],
        "corners clamp to the box edge colour instead of transparent black"
    );
}

#[test]
fn offset_wraps_periodically_when_wrap_is_true_and_clamps_to_edge_when_false() {
    // A 100×100 box at (150, 150) split vertically at x = 200: left half red (#FF0000), right half green (#00FF00).
    let split = |effects: &str| {
        format!(
            r##"{{"id":"split","type":"rect","start":0,"end":1000,"x":150,"y":150,
                "origin":"top-left","width":100,"height":100,
                "fill":{{"gradient":"linear","angle":90,"stops":[
                    {{"offset":0,"color":"#FF0000"}},
                    {{"offset":0.5,"color":"#FF0000"}},
                    {{"offset":0.5,"color":"#00FF00"}},
                    {{"offset":1,"color":"#00FF00"}}]}},
                "effects":[{effects}]}}"##
        )
    };
    // Shift right by +30 px:
    // At x = 160 (10 px inside left edge), source coordinate is 10 - 30 = -20.
    // With `wrap: true`, -20 mod 100 = 80 (in the green right half).
    // With `wrap: false`, -20 clamps to 0 (in the red left half).
    let wrapped = painted(
        line!(),
        &split(r#"{"name":"offset","shift_x":30,"shift_y":0,"wrap":true}"#),
    );
    let clamped = painted(
        line!(),
        &split(r#"{"name":"offset","shift_x":30,"shift_y":0,"wrap":false}"#),
    );
    assert_eq!(
        rgb(&wrapped, 160, 200),
        [0, 255, 0],
        "wrap: true wraps the green right half around to the left side"
    );
    assert_eq!(
        rgb(&clamped, 160, 200),
        [255, 0, 0],
        "wrap: false clamps to the red left edge"
    );
    // In both modes, x = 210 (60 px from left) samples 60 - 30 = 30 (from the red left half).
    assert_eq!(rgb(&wrapped, 210, 200), [255, 0, 0]);
    assert_eq!(rgb(&clamped, 210, 200), [255, 0, 0]);
}

