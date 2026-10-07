//! A path's stroke join, miter limit and cap, and the inset widened to the stroke's reach
//! (#751, ADR-0158 §1–§4 and §6–§7).

use std::path::PathBuf;

use montagent_core::finding::Class;
use montagent_core::stroke::{self, Source};
use serde_json::{Value, json};

mod common;
use common::{canonical, tempdir, write_project};

/// An open, stroked two-segment path in a 200×200 box, its corner well inside every inset
/// these tests compute.
fn open(fields: Value) -> Value {
    let mut element = json!({"id": "zig", "type": "path", "start": 0, "end": 1000, "x": 0, "y": 0,
        "origin": "top-left", "width": 200, "height": 200, "closed": false, "stroke": "#FFFFFF", "stroke_width": 3,
        "points": [{"at": [100, 100]}, {"at": [110, 110]}, {"at": [120, 100]}]});
    for (key, value) in fields.as_object().expect("fields are an object") {
        if value.is_null() {
            element.as_object_mut().unwrap().remove(key);
        } else {
            element[key] = value.clone();
        }
    }
    element
}

// ---------------------------------------------------------------------------
// The inset, `m = ceil(k × w / 2)`.
// ---------------------------------------------------------------------------

#[test]
fn round_and_bevel_joins_inset_by_half_the_width_rounded_up() {
    for join in [Value::Null, json!("round"), json!("bevel")] {
        for (width, inset) in [(3, 2), (4, 2), (5, 3)] {
            let reach = stroke::reach(&open(json!({"stroke_join": join, "stroke_width": width})));
            assert_eq!(reach.inset, inset, "{join} at width {width}");
            assert_eq!(reach.source, Source::Neither, "{join}");
        }
    }
}

#[test]
fn a_miter_limit_of_ten_on_a_width_of_three_insets_by_fifteen() {
    let reach = stroke::reach(&open(
        json!({"stroke_join": "miter", "stroke_miter_limit": 10, "stroke_width": 3}),
    ));
    assert_eq!(reach.inset, 15);
    assert_eq!(reach.source, Source::MiterLimit(10));
}

#[test]
fn a_square_cap_on_an_open_path_of_width_eight_insets_by_ceil_four_root_two() {
    // 4√2 = 5.657, so 6.
    let reach = stroke::reach(&open(json!({"stroke_cap": "square", "stroke_width": 8})));
    assert_eq!(reach.inset, 6);
    assert_eq!(reach.source, Source::SquareCap);
    // The prototype's 45° line: width 40, the cap corner at 28.28, so 29.
    let reach = stroke::reach(&open(json!({"stroke_cap": "square", "stroke_width": 40})));
    assert_eq!(reach.inset, 29);
}

#[test]
fn round_and_butt_caps_add_no_factor() {
    for cap in ["butt", "round"] {
        let reach = stroke::reach(&open(json!({"stroke_cap": cap, "stroke_width": 8})));
        assert_eq!((reach.inset, reach.source), (4, Source::Neither), "{cap}");
    }
}

#[test]
fn a_square_cap_on_a_closed_path_adds_no_factor() {
    let reach = stroke::reach(&open(
        json!({"closed": true, "stroke_cap": "square", "stroke_width": 8}),
    ));
    assert_eq!((reach.inset, reach.source), (4, Source::Neither));
}

#[test]
fn the_larger_factor_wins_and_a_miter_names_its_limit_over_a_square_cap() {
    // Limit 2 against √2: the miter is the larger factor.
    let reach = stroke::reach(&open(
        json!({"stroke_join": "miter", "stroke_miter_limit": 2,
        "stroke_cap": "square", "stroke_width": 8}),
    ));
    assert_eq!((reach.inset, reach.source), (8, Source::MiterLimit(2)));
    // Limit 1 always bevels, and √2 is the larger factor.
    let reach = stroke::reach(&open(
        json!({"stroke_join": "miter", "stroke_miter_limit": 1,
        "stroke_cap": "square", "stroke_width": 8}),
    ));
    assert_eq!((reach.inset, reach.source), (6, Source::SquareCap));
}

#[test]
fn a_keyed_stroke_width_widens_by_its_largest_key() {
    let reach = stroke::reach(&open(
        json!({"stroke_join": "miter", "stroke_miter_limit": 4,
        "stroke_width": [{"t": 0, "v": 2}, {"t": 500, "v": 5, "ease": "linear"}]}),
    ));
    assert_eq!(reach.inset, 10);
}

#[test]
fn no_stroke_insets_by_nothing() {
    let reach = stroke::reach(&open(json!({"stroke": null, "stroke_join": "miter",
        "stroke_miter_limit": 10})));
    assert_eq!(reach.inset, 0);
}

// ---------------------------------------------------------------------------
// The schema.
// ---------------------------------------------------------------------------

/// A scratch directory of its own for every call, so tests running in parallel never share
/// one.
fn scratch() -> PathBuf {
    use std::sync::atomic::{AtomicU32, Ordering};
    static NEXT: AtomicU32 = AtomicU32::new(0);
    tempdir(2_000_000 + NEXT.fetch_add(1, Ordering::Relaxed))
}

/// A 200×200 project at 10 fps over black, one track holding `elements`.
fn project(elements: &[Value]) -> PathBuf {
    write_project(
        &scratch(),
        "p.json",
        &canonical(
            &json!({"frame": {"width": 200, "height": 200}, "fps": 10, "background": "#000000",
                    "tracks": [{"name": "only", "layer": 0, "elements": elements}]})
            .to_string(),
        ),
    )
}

/// Every error `validate` gives `element`, as its code and its fields.
fn errors(element: Value) -> Vec<(String, Value)> {
    montagent_core::validate(&project(&[element]))
        .findings
        .iter()
        .filter(|finding| finding.class == Class::Error)
        .map(|finding| {
            let fields: serde_json::Map<String, Value> = finding
                .fields
                .iter()
                .map(|(name, value)| (name.to_string(), value.clone()))
                .collect();
            (finding.code.clone(), fields.into())
        })
        .collect()
}

/// The one schema error `element` gets, whose text contains `says`.
#[track_caller]
fn schema_error(element: Value, says: &str) {
    let errors = errors(element);
    assert!(
        errors
            .iter()
            .any(|(code, fields)| code.starts_with("E-SCHEMA") && fields.to_string().contains(says)),
        "no schema error saying {says:?}: {errors:#?}"
    );
}

#[test]
fn a_path_takes_a_join_a_limit_and_a_cap() {
    assert_eq!(
        errors(open(
            json!({"stroke_join": "miter", "stroke_miter_limit": 4,
                           "stroke_cap": "square"})
        )),
        []
    );
    for join in ["round", "bevel"] {
        for cap in ["butt", "round", "square"] {
            let element = open(json!({"stroke_join": join, "stroke_cap": cap}));
            assert_eq!(errors(element), [], "{join} {cap}");
        }
    }
}

#[test]
fn each_field_is_static() {
    let keyed = |v: Value| json!([{"t": 0, "v": v}, {"t": 500, "v": v, "ease": "linear"}]);
    schema_error(
        open(json!({"stroke_join": keyed(json!("bevel"))})),
        "static",
    );
    schema_error(open(json!({"stroke_cap": keyed(json!("round"))})), "static");
    schema_error(
        open(json!({"stroke_join": "miter", "stroke_miter_limit": keyed(json!(4))})),
        "static",
    );
}

#[test]
fn the_limit_is_an_integer_from_one_to_ten() {
    for limit in [json!(0), json!(11), json!(2.5), json!("4")] {
        schema_error(
            open(json!({"stroke_join": "miter", "stroke_miter_limit": limit})),
            "a static integer from 1 to 10",
        );
    }
    for limit in [1, 10] {
        let element = open(json!({"stroke_join": "miter", "stroke_miter_limit": limit}));
        assert_eq!(errors(element), [], "{limit}");
    }
}

#[test]
fn a_join_or_cap_outside_its_vocabulary_is_a_schema_error() {
    schema_error(open(json!({"stroke_join": "arcs"})), "arcs");
    schema_error(open(json!({"stroke_cap": "triangle"})), "triangle");
}

#[test]
fn on_a_rect_an_ellipse_or_text_each_field_is_refused_as_belonging_to_path() {
    let rect = json!({"id": "box", "type": "rect", "start": 0, "end": 1000, "width": 100,
                      "height": 100, "stroke": "#FFFFFF", "stroke_width": 4});
    let ellipse = json!({"id": "box", "type": "ellipse", "start": 0, "end": 1000, "width": 100,
                         "height": 100, "stroke": "#FFFFFF", "stroke_width": 4});
    let text = json!({"id": "box", "type": "text", "start": 0, "end": 1000, "width": 100,
                      "height": 100, "font": "Inter", "size": 20,
                      "runs": [{"text": "Hi"}], "stroke": "#FFFFFF", "stroke_width": 2});
    for base in [rect, ellipse, text] {
        for (key, value) in [
            ("stroke_join", json!("bevel")),
            ("stroke_miter_limit", json!(4)),
            ("stroke_cap", json!("round")),
        ] {
            let mut element = base.clone();
            element[key] = value;
            let errors = errors(element);
            let says = format!("`{key}` belongs to `path`");
            assert!(
                errors
                    .iter()
                    .any(|(code, fields)| code == "E-SCHEMA" && fields.to_string().contains(&says)),
                "{} with {key}: {errors:#?}",
                base["type"]
            );
            assert!(
                errors
                    .iter()
                    .all(|(code, _)| code != "E-SCHEMA-UNKNOWN-KEY"),
                "not the newer-format unknown key: {errors:#?}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// `validate`.
// ---------------------------------------------------------------------------

/// The fields of every `code` error `validate` gives `element`.
fn fired(element: Value, code: &str) -> Vec<Value> {
    errors(element)
        .into_iter()
        .filter(|(fired, _)| fired == code)
        .map(|(_, fields)| fields)
        .collect()
}

/// `validate`'s prose for `element`'s project.
fn prose(element: Value) -> String {
    let report = montagent_core::validate(&project(&[element]));
    montagent_core::text::render(&report.to_json(), montagent_core::text::Options::default())
        .expect("the report renders")
}

#[test]
fn a_miter_with_no_limit_is_an_error_at_the_join() {
    let element = open(json!({"stroke_join": "miter"}));
    let found = fired(element.clone(), "E-STROKE-MITER-LIMIT");
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0]["field"], json!("stroke_join"));
    assert!(
        prose(element).contains("`zig`.stroke_join: `\"miter\"` needs a `stroke_miter_limit`"),
        "names the field's location"
    );
}

#[test]
fn a_limit_with_any_other_join_is_an_error_at_the_limit() {
    for join in [Value::Null, json!("round"), json!("bevel")] {
        let element = open(json!({"stroke_join": join, "stroke_miter_limit": 4}));
        let found = fired(element.clone(), "E-STROKE-MITER-LIMIT");
        assert_eq!(found.len(), 1, "{join}: {found:?}");
        assert_eq!(found[0]["field"], json!("stroke_miter_limit"));
        assert!(
            prose(element).contains("`zig`.stroke_miter_limit: a limit applies only to"),
            "{join}"
        );
    }
    assert_eq!(
        fired(
            open(json!({"stroke_join": "miter", "stroke_miter_limit": 4})),
            "E-STROKE-MITER-LIMIT"
        ),
        Vec::<Value>::new()
    );
}

#[test]
fn any_cap_on_a_closed_path_is_undrawn_and_names_closed() {
    for cap in ["butt", "round", "square"] {
        let element = open(json!({"closed": true, "stroke_cap": cap}));
        let found = fired(element.clone(), "E-STROKE-CAP-UNDRAWN");
        assert_eq!(found.len(), 1, "{cap}: {found:?}");
        assert_eq!(found[0]["cap"], json!(cap));
        let text = prose(element);
        assert!(text.contains("`zig`.stroke_cap:"), "{text}");
        assert!(text.contains("`closed`"), "{text}");
    }
    for cap in ["butt", "round", "square"] {
        let element = open(json!({"stroke_cap": cap}));
        assert_eq!(fired(element, "E-STROKE-CAP-UNDRAWN"), Vec::<Value>::new());
    }
}

#[test]
fn a_stroke_field_with_no_stroke_to_shape_is_an_error_at_each_field() {
    let fields = json!({"stroke_join": "miter", "stroke_miter_limit": 4, "stroke_cap": "round"});
    let keyed_zero = json!([{"t": 0, "v": 0}, {"t": 500, "v": 0, "ease": "linear"}]);
    for (case, unstroked) in [
        ("no stroke", json!({"stroke": null, "stroke_width": null})),
        ("no stroke_width", json!({"stroke_width": null})),
        ("a static zero", json!({"stroke_width": 0})),
        ("every key zero", json!({"stroke_width": keyed_zero})),
    ] {
        let mut element = open(fields.clone());
        for (key, value) in unstroked.as_object().unwrap() {
            if value.is_null() {
                element.as_object_mut().unwrap().remove(key);
            } else {
                element[key] = value.clone();
            }
        }
        let found = fired(element.clone(), "E-STROKE-NO-STROKE");
        let named: Vec<&Value> = found.iter().map(|fields| &fields["field"]).collect();
        assert_eq!(
            named,
            [
                &json!("stroke_join"),
                &json!("stroke_miter_limit"),
                &json!("stroke_cap")
            ],
            "{case}"
        );
        assert!(prose(element).contains("`zig`.stroke_cap:"), "{case}");
    }
    // A keyed width that only passes through 0 shapes the frames where it is above 0.
    let passing = json!([{"t": 0, "v": 0}, {"t": 500, "v": 4, "ease": "linear"}]);
    let element = open(json!({"stroke_join": "bevel", "stroke_width": passing}));
    assert_eq!(fired(element, "E-STROKE-NO-STROKE"), Vec::<Value>::new());
}

#[test]
fn outside_the_box_names_k_and_its_source_and_says_the_bound_is_worst_case() {
    // Miter limit 10 on width 3: inset 15, so a vertex at x 10 is outside.
    let element = open(json!({"stroke_join": "miter", "stroke_miter_limit": 10,
        "points": [{"at": [10, 20]}, {"at": [100, 100]}]}));
    let found = fired(element.clone(), "E-PATH-OUTSIDE-BOX");
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0]["inset"], json!(15));
    assert_eq!(found[0]["k"], json!("10"));
    let text = prose(element);
    assert!(
        text.contains(
            "inset 15 = ceil(10 × 3 / 2), from `stroke_miter_limit` 10. This bound is \
             worst-case, not a measured overlap."
        ),
        "{text}"
    );
    // A square cap on an open path of width 20: inset ceil(√2 × 10) = 15.
    let element = open(json!({"stroke_cap": "square", "stroke_width": 20,
        "points": [{"at": [10, 100]}, {"at": [100, 100]}]}));
    let found = fired(element.clone(), "E-PATH-OUTSIDE-BOX");
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0]["k"], json!("√2"));
    assert!(prose(element).contains("inset 15 = ceil(√2 × 20 / 2), from `stroke_cap` \"square\""),);
}

#[test]
fn the_overshoot_clamp_uses_the_widened_inset() {
    // The apex sinks from box y 20 to 60 under an ease whose `y` far below 0 carries it up
    // past the top of the 200×200 box. A miter limit of 5 on width 4 makes the inset 10, so
    // the clamped apex sits at y 10, not at the 2 a round join would give.
    let element = open(
        json!({"stroke_join": "miter", "stroke_miter_limit": 5, "stroke_width": 4,
        "points": [
            {"t": 0, "v": [{"at": [20, 90]}, {"at": [50, 20]}, {"at": [90, 90]}]},
            {"t": 1000, "v": [{"at": [20, 90]}, {"at": [50, 60]}, {"at": [90, 90]}],
             "ease": [0.5, -3.0, 0.5, 1.0]}]}),
    );
    let resolved = montagent_core::animatable::at(&element, "points", 300)
        .expect("points are declared")
        .expect("points resolve");
    let resolved = serde_json::to_value(resolved).unwrap();
    assert_eq!(
        resolved[1]["at"][1],
        json!(10.0),
        "clamped to the widened inset: {resolved}"
    );
}

// ---------------------------------------------------------------------------
// The painter.
// ---------------------------------------------------------------------------

const BLACK: [u8; 3] = [0, 0, 0];
const WHITE: [u8; 3] = [255, 255, 255];

/// The frame at 0 ms of a project holding `element` alone, as RGB, 200 pixels wide.
fn painted(element: Value) -> Vec<u8> {
    use montagent_core::verbs::render::{Supplying, paint_span};
    let rasters = paint_span(&project(&[element]), 0, 1, Supplying::PerFrame).expect("it paints");
    assert!(rasters.declined.is_empty(), "{:?}", rasters.declined);
    rasters.frames.into_iter().next().unwrap()
}

fn pixel(rgb: &[u8], x: usize, y: usize) -> [u8; 3] {
    let at = (y * 200 + x) * 3;
    [rgb[at], rgb[at + 1], rgb[at + 2]]
}

/// A V of width 10 whose corner is at box (100, 160), with the box at the frame's origin.
/// Its arms meet at 53.13°, so a miter's tip reaches 1/sin(26.57°) = 2.236 half-widths
/// below the vertex, to y 171.18; a round join reaches y 165, and a bevel's edge y 162.24.
fn vee(fields: Value) -> Value {
    let mut element = open(json!({"stroke_width": 10,
        "points": [{"at": [40, 40]}, {"at": [100, 160]}, {"at": [160, 40]}]}));
    for (key, value) in fields.as_object().unwrap() {
        element[key] = value.clone();
    }
    element
}

#[test]
fn a_join_and_a_cap_written_as_their_defaults_paint_the_bytes_of_none() {
    let bare = painted(vee(json!({})));
    let written = painted(vee(json!({"stroke_join": "round", "stroke_cap": "butt"})));
    assert!(bare == written);
    assert!(bare.iter().any(|byte| *byte != 0), "the vee paints");
}

#[test]
fn each_join_draws_its_own_corner_and_the_limit_bevels_past_it() {
    let below_round = |fields: Value| pixel(&painted(vee(fields)), 100, 167);
    let below_bevel = |fields: Value| pixel(&painted(vee(fields)), 100, 163);
    assert_eq!(
        below_round(json!({"stroke_join": "miter", "stroke_miter_limit": 3})),
        WHITE,
        "a miter reaches past the round join"
    );
    assert_eq!(
        below_round(json!({})),
        BLACK,
        "the round join stops at y 165"
    );
    assert_eq!(
        below_bevel(json!({})),
        WHITE,
        "the round join reaches y 163"
    );
    assert_eq!(
        below_bevel(json!({"stroke_join": "bevel"})),
        BLACK,
        "a bevel stops at y 162.24"
    );
    assert_eq!(
        below_bevel(json!({"stroke_join": "miter", "stroke_miter_limit": 2})),
        BLACK,
        "2.236 is past a limit of 2, so the corner bevels"
    );
}

#[test]
fn each_cap_draws_its_own_end() {
    // A level line from box (40, 100) to (160, 100), width 10: a butt end stops at x 160, a
    // round cap reaches 5 px past it in a half-disc, and a square cap fills the square.
    let line = |cap: Option<&str>| {
        let mut element = open(json!({"stroke_width": 10,
            "points": [{"at": [40, 100]}, {"at": [160, 100]}]}));
        if let Some(cap) = cap {
            element["stroke_cap"] = json!(cap);
        }
        painted(element)
    };
    let (butt, round, square) = (line(None), line(Some("round")), line(Some("square")));
    assert_eq!(pixel(&butt, 162, 100), BLACK, "butt");
    assert_eq!(pixel(&round, 162, 100), WHITE, "round, on the axis");
    assert_eq!(
        pixel(&round, 164, 104),
        BLACK,
        "round, outside its half-disc"
    );
    assert_eq!(pixel(&square, 164, 104), WHITE, "square, in its corner");
    assert_eq!(pixel(&square, 165, 100), BLACK, "square, past its end");
}

// ---------------------------------------------------------------------------
// Containment: the widened inset keeps every frame's ink inside the box.
// ---------------------------------------------------------------------------

/// Every frame of `element`, alone in a 200×200 frame at 30 fps with its 160×160 box at
/// (20, 20), checked for any ink outside the box. `validate` must pass it first: the bound
/// is only promised for a file `validate` accepts.
#[track_caller]
fn contained(name: &str, element: Value, frames: usize) {
    use montagent_core::verbs::render::{Supplying, paint_span};
    let mut element = element;
    for (key, value) in [
        ("x", json!(20)),
        ("y", json!(20)),
        ("width", json!(160)),
        ("height", json!(160)),
    ] {
        element[key] = value;
    }
    let path = write_project(
        &scratch(),
        "p.json",
        &canonical(
            &json!({"frame": {"width": 200, "height": 200}, "fps": 30, "background": "#000000",
                    "tracks": [{"name": "only", "layer": 0, "elements": [element]}]})
            .to_string(),
        ),
    );
    let report = montagent_core::validate(&path);
    assert!(
        report.findings.iter().all(|f| f.class != Class::Error),
        "{name} validates: {:#?}",
        report.findings
    );
    let rasters = paint_span(&path, 0, 1000, Supplying::PerFrame).expect("it paints");
    assert_eq!(rasters.frames.len(), frames, "{name}");
    let mut inked = 0;
    for (index, frame) in rasters.frames.iter().enumerate() {
        let mut outside = 0;
        for y in 0..200 {
            for x in 0..200 {
                let inside = (20..180).contains(&x) && (20..180).contains(&y);
                let lit = pixel(frame, x, y) != BLACK;
                outside += usize::from(lit && !inside);
                inked += usize::from(lit);
            }
        }
        assert_eq!(outside, 0, "{name}, frame {index}: ink outside the box");
    }
    assert!(inked > 0, "{name} paints");
}

#[test]
fn a_keyed_miter_whose_in_between_corners_sharpen_past_the_limit_stays_in_the_box() {
    // ADR-0158 §8: the two arms swap sides, so the corner at box (140, 80) is 45° at both
    // keys and closes to 0° and back in between. Limit 10, width 4: inset 20, and the tip
    // reaches at most 20 px right of x 140, to the box's right edge. Past the limit the
    // corner snaps to a bevel.
    let element = open(
        json!({"stroke_join": "miter", "stroke_miter_limit": 10, "stroke_width": 4,
        "points": [
            {"t": 0, "v": [{"at": [20, 30]}, {"at": [140, 80]}, {"at": [20, 130]}]},
            {"t": 1000, "v": [{"at": [20, 130]}, {"at": [140, 80]}, {"at": [20, 30]}],
             "ease": "linear"}]}),
    );
    contained("keyed miter", element, 30);
}

#[test]
fn a_miter_corner_just_inside_limit_ten_stays_in_the_box() {
    // Half-angle atan(10 / 80) = 7.1°, a miter ratio of 8.06 at width 8: the tip sits 32 px
    // above the vertex at box y 40, inside the inset 40 = ceil(10 × 8 / 2).
    let element = open(
        json!({"stroke_join": "miter", "stroke_miter_limit": 10, "stroke_width": 8,
        "points": [{"at": [70, 120]}, {"at": [80, 40]}, {"at": [90, 120]}]}),
    );
    contained("miter 10", element, 30);
}

#[test]
fn a_square_cap_on_a_forty_five_degree_line_stays_in_the_box() {
    // Width 40: the cap's corner reaches 20√2 = 28.28 along each axis, inside the inset 29.
    let element = open(json!({"stroke_cap": "square", "stroke_width": 40,
        "points": [{"at": [29, 131]}, {"at": [131, 29]}]}));
    contained("square cap at 45°", element, 30);
}

// ---------------------------------------------------------------------------
// `query --at`.
// ---------------------------------------------------------------------------

/// `query --at`'s answer for the one element of `element`'s project, as JSON.
fn queried(element: Value) -> Value {
    let document = montagent_core::parse::read(&project(&[element])).unwrap();
    let answer = serde_json::to_value(montagent_core::verbs::query::at::at(&document, 0, None));
    answer.unwrap()["stack"][0].clone()
}

#[test]
fn query_at_reports_a_miter_paths_inset_and_reach_beside_its_control_points() {
    let present = queried(open(
        json!({"stroke_join": "miter", "stroke_miter_limit": 10}),
    ));
    assert_eq!(
        present["stroke"],
        json!({"inset": 15, "reach_factor": "10", "reach_source": "`stroke_miter_limit` 10"}),
        "{present}"
    );
    assert!(present["path"].is_array(), "beside the control points");
    let present = queried(open(json!({"stroke_cap": "square", "stroke_width": 8})));
    assert_eq!(
        present["stroke"],
        json!({"inset": 6, "reach_factor": "√2", "reach_source": "`stroke_cap` \"square\""})
    );
    let present = queried(open(json!({})));
    assert_eq!(present["stroke"]["inset"], json!(2));
    assert_eq!(present["stroke"]["reach_factor"], json!("1"));
}

#[test]
fn query_at_prose_names_the_inset_and_its_reach() {
    use montagent_core::verbs::query::{Ask, query};
    use montagent_core::wire::{Wire, render_query};
    let path = project(&[open(
        json!({"stroke_join": "miter", "stroke_miter_limit": 10}),
    )]);
    let ask = Ask {
        at: Some(0),
        ..Ask::default()
    };
    let text = render_query(&query(&path, &ask), Wire::Text { verbose: false });
    assert!(
        text.contains("inset 15 (k 10: `stroke_miter_limit` 10)"),
        "{text}"
    );
}
