//! `feather` on a `mask` (#698, ADR-0152 §2): the hard mask's coverage blurred by a
//! Gaussian of σ = `feather` / 2, centred on the edge, riding the transform.
//!
//! Asserted at three seams: the model's parse (what is a schema error), the `frame` verb's
//! pixels (what is painted), and `validate`'s report (`R-MASK-ERASES-ALL`). Byte identity
//! across painters and with the blur bounds hint on and off lives beside the other cases
//! of each rule, in `tests/painters.rs` and `tests/filter_bound.rs`.

use montagent_core::model::Effect;
use montagent_core::report::ExitCode;
use montagent_core::verbs::frame::{Ask, frame};

mod common;
use common::{canonical, tempdir, write_project};

fn parse(text: &str) -> Result<Effect, serde_json::Error> {
    serde_json::from_str::<Effect>(text)
}

/// The exact pixels at `instant` of a 400×400 black frame holding `elements`, at true scale
/// and lossless, decoded by a decoder that is not the one that wrote them.
#[track_caller]
fn painted_at(line: u32, elements: &str, instant: i64) -> image::RgbaImage {
    let dir = tempdir(line);
    let project = write_project(
        &dir,
        "p.montagent.json",
        &canonical(&format!(
            r##"{{"frame":{{"width":400,"height":400}},"fps":25,"background":"#000000",
                "tracks":[{{"name":"only","layer":0,"elements":[{elements}]}}]}}"##
        )),
    );
    let answer = frame(
        &project,
        &Ask {
            at: Some(instant),
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

#[track_caller]
fn painted(line: u32, elements: &str) -> image::RgbaImage {
    painted_at(line, elements, 500)
}

/// A 300×300 opaque white rect centred in the frame (it spans 50..350 on both axes),
/// carrying `effects` verbatim. White on black, so the red channel is its kept coverage.
fn white_with(effects: &str) -> String {
    format!(
        r##"{{"id":"plate","type":"rect","start":0,"end":1000,"x":200,"y":200,
            "origin":"center","width":300,"height":300,"fill":"#FFFFFF",
            "effects":[{effects}]}}"##
    )
}

fn red(picture: &image::RgbaImage, x: u32, y: u32) -> i32 {
    i32::from(picture.get_pixel(x, y).0[0])
}

/// The standard normal CDF, by Abramowitz and Stegun 7.1.26 (error below 1.5e-7): an
/// independent statement of the Gaussian, not the painter's own kernel.
fn phi(z: f64) -> f64 {
    let x = z.abs() / std::f64::consts::SQRT_2;
    let t = 1.0 / (1.0 + 0.327_591_1 * x);
    let poly = t
        * (0.254_829_592
            + t * (-0.284_496_736
                + t * (1.421_413_741 + t * (-1.453_152_027 + t * 1.061_405_429))));
    let erf = 1.0 - poly * (-x * x).exp();
    if z >= 0.0 {
        0.5 * (1.0 + erf)
    } else {
        0.5 * (1.0 - erf)
    }
}

// ---- `shift` carries a keyed feather -------------------------------------------------

/// A project of one plate whose first effect is a mask with a keyed `feather`, over
/// `start`..2000; its path.
fn keyed_plate(line: u32, start: i64, feather: serde_json::Value) -> std::path::PathBuf {
    let plate = format!(
        r##"{{"id":"plate","type":"rect","start":{start},"end":2000,"x":200,"y":200,
            "origin":"center","width":300,"height":300,"fill":"#FFFFFF",
            "effects":[{{"name":"mask","shape":"ellipse","feather":{feather}}}]}}"##
    );
    write_project(
        &tempdir(line),
        "p.montagent.json",
        &canonical(&format!(
            r##"{{"frame":{{"width":400,"height":400}},"fps":25,"background":"#000000",
                "tracks":[{{"name":"only","layer":0,"elements":[{plate}]}}]}}"##
        )),
    )
}

#[track_caller]
fn shifted_feather(path: &std::path::Path, at: i64, delta: i64) -> Vec<(i64, serde_json::Value)> {
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
    assert_ne!(
        answer.report().exit_code(),
        ExitCode::Errors,
        "{:?}",
        answer.report().findings
    );
    let written: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).expect("written")).expect("JSON");
    written["tracks"][0]["elements"][0]["effects"][0]["feather"]
        .as_array()
        .expect("still keyed")
        .iter()
        .map(|record| (record["t"].as_i64().expect("a t"), record["v"].clone()))
        .collect()
}

#[test]
fn a_plain_shift_carries_a_keyed_feather_with_its_element() {
    let path = keyed_plate(
        line!(),
        100,
        serde_json::json!([{"t": 100, "v": 0}, {"t": 900, "v": 40, "ease": "linear"}]),
    );
    assert_eq!(
        shifted_feather(&path, 0, 300),
        vec![(400, serde_json::json!(0)), (1200, serde_json::json!(40))]
    );
}

#[test]
fn a_shift_split_inside_a_keyed_feather_segment_writes_an_integer() {
    // 0 → 25 over a second, cut at 500: the resolved 12.5 is written as 13, ties away from
    // zero, as `x` and every integer-typed property splits (ADR-0012, ADR-0146 §7).
    let path = keyed_plate(
        line!(),
        0,
        serde_json::json!([{"t": 0, "v": 0}, {"t": 1000, "v": 25, "ease": "linear"}]),
    );
    assert_eq!(
        shifted_feather(&path, 500, 100),
        vec![
            (0, serde_json::json!(0)),
            (500, serde_json::json!(13)),
            (600, serde_json::json!(13)),
            (1100, serde_json::json!(25)),
        ]
    );
}

// ---- `query --at` reads a keyed feather ----------------------------------------------

#[test]
fn query_at_resolves_a_feather_by_its_position_in_effects_unrounded() {
    use montagent_core::verbs::query::{Ask, query};
    let path = keyed_plate(
        line!(),
        0,
        serde_json::json!([{"t": 0, "v": 0}, {"t": 1000, "v": 25, "ease": "linear"}]),
    );
    let answer = query(
        &path,
        &Ask {
            at: Some(500),
            ..Ask::default()
        },
    );
    assert_eq!(answer.report().exit_code(), ExitCode::Ok);
    let json = answer.to_json();
    let row = json["query"]["stack"][0]["values"]
        .as_array()
        .expect("values")
        .iter()
        .find(|row| row["property"] == "effects[0].feather (mask)")
        .unwrap_or_else(|| panic!("no feather row: {json}"))
        .clone();
    assert_eq!(row["value"], serde_json::json!(12.5));
    assert_eq!(row["animated"], serde_json::json!(true));
}

// ---- the picture ---------------------------------------------------------------------

#[test]
fn a_keyed_feather_paints_its_resolved_value_unrounded() {
    let keyed = |list: &str| {
        white_with(&format!(
            r##"{{"name":"mask","shape":"ellipse","x":40,"y":40,"width":220,"height":220,"feather":{list}}}"##
        ))
    };
    let fixed = |feather: i64| {
        white_with(&format!(
            r##"{{"name":"mask","shape":"ellipse","x":40,"y":40,"width":220,"height":220,"feather":{feather}}}"##
        ))
    };
    // 0 → 40 over the element's life: 20 at its midpoint, the static 20's own bytes.
    let ramp = keyed(r##"[{"t":0,"v":0},{"t":1000,"v":40,"ease":"linear"}]"##);
    assert!(painted_at(line!(), &ramp, 500) == painted(line!(), &fixed(20)));
    // 0 → 21: 10.5 at the midpoint, which is neither integer either side of it.
    let odd = painted_at(
        line!(),
        &keyed(r##"[{"t":0,"v":0},{"t":1000,"v":21,"ease":"linear"}]"##),
        500,
    );
    assert!(odd != painted(line!(), &fixed(10)), "10.5 was rounded down");
    assert!(odd != painted(line!(), &fixed(11)), "10.5 was rounded up");
}

#[test]
fn a_written_feather_zero_paints_the_same_bytes_as_none() {
    for (bare, zero) in [
        (
            r##"{"name":"mask","shape":"circle"}"##,
            r##"{"name":"mask","shape":"circle","feather":0}"##,
        ),
        (
            r##"{"name":"mask","shape":"rect","x":40,"y":60,"width":200,"height":120,"radius":30,"invert":true}"##,
            r##"{"name":"mask","shape":"rect","x":40,"y":60,"width":200,"height":120,"radius":30,"invert":true,"feather":0}"##,
        ),
    ] {
        let (a, b) = (
            painted(line!(), &white_with(bare)),
            painted(line!(), &white_with(zero)),
        );
        assert!(a == b, "{zero} paints differently from {bare}");
    }
}

#[test]
fn an_edge_pixel_is_the_gaussian_of_sigma_feather_over_two_on_the_hard_coverage() {
    // A rect mask whose right edge sits on the frame's x = 300, its other edges 100 px
    // (over eight σ) away from row 200. σ = 24 / 2 = 12. The softness is centred on the
    // edge, so the pixel centred at x + 0.5 keeps Φ((300 − (x + 0.5)) / σ).
    let picture = painted(
        line!(),
        &white_with(
            r##"{"name":"mask","shape":"rect","x":50,"y":50,"width":200,"height":200,"feather":24}"##,
        ),
    );
    let sigma = 12.0;
    for x in [299_u32, 300] {
        let expected = (255.0 * phi((300.0 - (f64::from(x) + 0.5)) / sigma)).round() as i32;
        assert_eq!(red(&picture, x, 200), expected, "x = {x}");
    }
    // The ramp reaches past the stated rect and falls short of it, and is monotone across.
    assert!(
        red(&picture, 310, 200) > 0,
        "the feather reaches past the rect"
    );
    assert!(
        red(&picture, 290, 200) < 255,
        "and falls short of it inside"
    );
    assert!(red(&picture, 280, 200) > red(&picture, 290, 200));
    assert_eq!(red(&picture, 200, 200), 255, "the middle is kept whole");
    assert_eq!(
        red(&picture, 345, 200),
        0,
        "beyond the reach nothing is kept"
    );
}

#[test]
fn a_feathered_mask_and_its_inverted_twin_sum_to_the_unmasked_alpha() {
    for mask in [
        r##"{"name":"mask","shape":"circle","x":50,"y":50,"width":200,"height":200,"feather":30}"##,
        r##"{"name":"mask","shape":"ellipse","x":20,"y":80,"width":260,"height":140,"feather":16}"##,
        r##"{"name":"mask","shape":"rect","x":60,"y":60,"width":180,"height":180,"radius":40,"feather":50}"##,
    ] {
        let inverted = mask.replacen('}', r#","invert":true}"#, 1);
        let plain = painted(line!(), &white_with(mask));
        let twin = painted(line!(), &white_with(&inverted));
        let mut soft = 0;
        for y in (50..350).step_by(5) {
            for x in (50..350).step_by(5) {
                let (p, i) = (red(&plain, x, y), red(&twin, x, y));
                assert!(
                    (p + i - 255).abs() <= 1,
                    "{mask} at ({x},{y}): plain {p}, inverted {i}"
                );
                soft += i32::from(p > 8 && p < 247);
            }
        }
        assert!(
            soft > 40,
            "{mask}: too few soft pixels ({soft}) for a feather"
        );
        // Outside the element there is nothing to keep either way.
        assert_eq!((red(&plain, 20, 20), red(&twin, 20, 20)), (0, 0));
    }
}

// ---- the schema ----------------------------------------------------------------------

#[test]
fn feather_is_a_non_negative_integer_or_a_keyframe_list_of_them_on_a_mask() {
    for accepted in [
        r##"{"name":"mask","shape":"circle","feather":0}"##,
        r##"{"name":"mask","shape":"rect","feather":24}"##,
        r##"{"name":"mask","shape":"ellipse","invert":true,"feather":8}"##,
        r##"{"name":"mask","shape":"circle","feather":[{"t":0,"v":0},{"t":500,"v":40,"ease":"ease-out"}]}"##,
    ] {
        parse(accepted).unwrap_or_else(|e| panic!("{accepted}: {e}"));
    }
}

#[test]
fn a_negative_or_fractional_feather_or_one_on_another_effect_is_a_schema_error() {
    for refused in [
        r##"{"name":"mask","shape":"circle","feather":-1}"##,
        r##"{"name":"mask","shape":"circle","feather":2.5}"##,
        r##"{"name":"mask","shape":"circle","feather":"8"}"##,
        r##"{"name":"mask","shape":"circle","feather":[{"t":0,"v":4},{"t":500,"v":-4,"ease":"linear"}]}"##,
        r##"{"name":"mask","shape":"circle","feather":[{"t":0,"v":4},{"t":500,"v":4.5,"ease":"linear"}]}"##,
        r##"{"name":"blur","radius":4,"feather":8}"##,
        r##"{"name":"shadow","dx":0,"dy":0,"radius":4,"color":"#000000","opacity":1,"feather":8}"##,
    ] {
        assert!(parse(refused).is_err(), "{refused} parsed");
    }
}

#[test]
fn feather_sits_after_invert_in_the_canonical_key_order_and_a_written_zero_is_kept() {
    for written in [
        r##"{"name":"mask","shape":"rect","x":1,"y":2,"width":3,"height":4,"radius":5,"invert":true,"feather":6}"##,
        r##"{"name":"mask","shape":"circle","feather":0}"##,
    ] {
        let parsed = parse(written).expect("it parses");
        assert_eq!(
            serde_json::to_string(&parsed).expect("it serialises"),
            written
        );
    }
}
