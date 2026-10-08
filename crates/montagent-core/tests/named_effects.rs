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
