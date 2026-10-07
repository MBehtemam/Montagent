//! A `path` element (#710, ADR-0154): vertices with handles in integer pixels from the
//! declared box, a centred stroke the box contains, and four `validate` errors.

use std::path::{Path, PathBuf};

use montagent_core::animatable::{self, Kind};
use montagent_core::finding::Class;
use montagent_core::report::{ExitCode, Report};
use montagent_core::verbs::query::at;
use montagent_core::verbs::render::{Supplying, paint_span};
use serde_json::{Value, json};

mod common;
use common::{canonical, tempdir, write_project};

/// A 200×200 project at 10 fps over black, one track holding `elements`.
fn project(dir: &Path, elements: &[Value]) -> PathBuf {
    write_project(
        dir,
        "p.json",
        &canonical(
            &json!({"frame": {"width": 200, "height": 200}, "fps": 10, "background": "#000000",
                    "tracks": [{"name": "only", "layer": 0, "elements": elements}]})
            .to_string(),
        ),
    )
}

/// A closed triangle in a 100×100 box centred on the frame, filled white.
fn triangle() -> Value {
    json!({"id": "tri", "type": "path", "start": 0, "end": 1000, "x": 100, "y": 100,
           "origin": "center", "width": 100, "height": 100, "closed": true, "fill": "#FFFFFF",
           "points": [{"at": [10, 90]}, {"at": [50, 10]}, {"at": [90, 90]}]})
}

/// `base` with `fields` laid over it; a `null` field is removed.
fn with(mut base: Value, fields: Value) -> Value {
    for (key, value) in fields.as_object().expect("fields are an object") {
        if value.is_null() {
            base.as_object_mut().unwrap().remove(key);
        } else {
            base[key] = value.clone();
        }
    }
    base
}

fn errors(report: &Report) -> Vec<(String, String)> {
    report
        .findings
        .iter()
        .filter(|finding| finding.class == Class::Error)
        .map(|finding| {
            (
                finding.code.clone(),
                serde_json::to_string(&finding.fields).unwrap(),
            )
        })
        .collect()
}

/// A scratch directory of its own for every call, so tests running in parallel never share
/// one.
fn scratch() -> PathBuf {
    use std::sync::atomic::{AtomicU32, Ordering};
    static NEXT: AtomicU32 = AtomicU32::new(0);
    tempdir(1_000_000 + NEXT.fetch_add(1, Ordering::Relaxed))
}

fn validated(element: Value) -> Vec<(String, String)> {
    errors(&montagent_core::validate(&project(&scratch(), &[element])))
}

/// The one schema error `element` gets, whose text contains `says`.
#[track_caller]
fn schema_error(element: Value, says: &str) {
    let errors = validated(element);
    assert!(
        errors
            .iter()
            .any(|(code, fields)| code.starts_with("E-SCHEMA") && fields.contains(says)),
        "no schema error saying {says:?}: {errors:#?}"
    );
}

// ---------------------------------------------------------------------------
// The schema.
// ---------------------------------------------------------------------------

#[test]
fn a_closed_triangle_validates() {
    assert_eq!(validated(triangle()), []);
}

#[test]
fn fill_on_an_open_path_is_a_schema_error() {
    schema_error(with(triangle(), json!({"closed": false})), "drop `fill`");
}

#[test]
fn a_keyed_width_or_height_is_a_schema_error_that_says_to_edit_the_points_or_animate_scale() {
    for side in ["width", "height"] {
        schema_error(
            with(
                triangle(),
                json!({side: [{"t": 0, "v": 100}, {"t": 500, "v": 120, "ease": "linear"}]}),
            ),
            "animate `scale`",
        );
    }
}

#[test]
fn a_keyed_closed_is_a_schema_error() {
    schema_error(
        with(
            triangle(),
            json!({"closed": [{"t": 0, "v": true}, {"t": 500, "v": false, "ease": "step"}]}),
        ),
        "two elements",
    );
}

#[test]
fn radius_is_a_schema_error() {
    schema_error(with(triangle(), json!({"radius": 4})), "radius");
}

#[test]
fn an_extra_vertex_key_is_a_schema_error() {
    schema_error(
        with(
            triangle(),
            json!({"points": [{"at": [10, 90], "smooth": true}, {"at": [50, 10]}, {"at": [90, 90]}]}),
        ),
        "smooth",
    );
}

#[test]
fn a_non_integer_coordinate_is_a_schema_error() {
    schema_error(
        with(
            triangle(),
            json!({"points": [{"at": [10.5, 90]}, {"at": [50, 10]}, {"at": [90, 90]}]}),
        ),
        "10.5",
    );
}

// ---------------------------------------------------------------------------
// `points` on the one list, and the one resolving function.
// ---------------------------------------------------------------------------

#[test]
fn points_is_on_the_one_list_as_a_whole_list_value_and_the_box_is_not() {
    let names: Vec<&str> = animatable::of("path")
        .iter()
        .map(|property| property.name.as_str())
        .collect();
    assert_eq!(
        names,
        [
            "x",
            "y",
            "fill",
            "fill.angle",
            "fill.stops",
            "fill.center",
            "fill.radius",
            "stroke",
            "stroke.angle",
            "stroke.stops",
            "stroke.center",
            "stroke.radius",
            "stroke_width",
            "stroke_dash_offset",
            "trim_start",
            "trim_end",
            "trim_offset",
            "points",
            "scale",
            "rotation",
            "swivel",
            "tilt",
            "perspective",
            "opacity"
        ]
    );
    assert_eq!(
        animatable::property("points").map(|property| property.kind),
        Some(Kind::Points)
    );
}

/// A triangle in a 100×100 box whose apex sinks from `[50, 10]` to `[50, 50]`.
fn sinking(ease: Value) -> Value {
    with(
        triangle(),
        json!({"points": [
            {"t": 0, "v": [{"at": [10, 90]}, {"at": [50, 10], "out": [10, 0]}, {"at": [90, 90]}]},
            {"t": 1000, "v": [{"at": [10, 90]}, {"at": [50, 50], "out": [30, 0]}, {"at": [90, 90]}],
             "ease": ease}
        ]}),
    )
}

#[test]
fn keyed_points_interpolate_number_by_number() {
    let resolved = animatable::at(&sinking(json!("linear")), "points", 500)
        .unwrap()
        .unwrap();
    assert_eq!(
        serde_json::to_value(resolved).unwrap(),
        json!([{"at": [10.0, 90.0]}, {"at": [50.0, 30.0], "out": [20.0, 0.0]}, {"at": [90.0, 90.0]}])
    );
}

#[test]
fn an_overshoot_clamps_each_absolute_vertex_and_handle_into_the_inset_box() {
    // `y` far below 0 carries the apex up past the top of the box: unclamped it would sit
    // near y = -34. A stroke of 5 makes the inset `ceil(5 / 2) = 3`.
    let element = with(
        sinking(json!([0.5, -3.0, 0.5, 1.0])),
        json!({"stroke": "#FFFFFF", "stroke_width": 5}),
    );
    let resolved =
        serde_json::to_value(animatable::at(&element, "points", 300).unwrap().unwrap()).unwrap();
    let apex = &resolved[1];
    assert_eq!(apex["at"][1], json!(3.0), "{resolved}");
    // The handle is clamped as an absolute position, then written back as an offset from
    // its clamped vertex: level with it here.
    assert_eq!(apex["out"][1], json!(0.0), "{resolved}");
    for vertex in resolved.as_array().unwrap() {
        let at = [
            vertex["at"][0].as_f64().unwrap(),
            vertex["at"][1].as_f64().unwrap(),
        ];
        for handle in ["in", "out"] {
            if let Some(offset) = vertex.get(handle) {
                for axis in 0..2 {
                    let absolute = at[axis] + offset[axis].as_f64().unwrap();
                    assert!((3.0..=97.0).contains(&absolute), "{resolved}");
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// The painter.
// ---------------------------------------------------------------------------

/// The committed fixture: a line, a filled and stroked triangle, an S-curve, a
/// self-intersecting star and a triangle filled with a linear gradient, each in a box of its
/// own, every box placed by its top-left corner.
fn fixture(dir: &Path) -> PathBuf {
    let body = std::fs::read_to_string(
        common::fixture_dir()
            .parent()
            .unwrap()
            .join("path/paths.montagent.json"),
    )
    .unwrap();
    write_project(dir, "paths.montagent.json", &canonical(&body))
}

/// The frame painted at `t`, as RGB.
fn painted_at(path: &Path, t: i64) -> Vec<u8> {
    let rasters = paint_span(path, t, t + 1, Supplying::PerFrame).expect("the span paints");
    assert!(rasters.declined.is_empty(), "{:?}", rasters.declined);
    rasters.frames.into_iter().next().unwrap()
}

fn pixel(rgb: &[u8], width: usize, x: usize, y: usize) -> [u8; 3] {
    let at = (y * width + x) * 3;
    [rgb[at], rgb[at + 1], rgb[at + 2]]
}

const BLACK: [u8; 3] = [0, 0, 0];
const WHITE: [u8; 3] = [255, 255, 255];

#[test]
fn the_fixture_validates() {
    let report = montagent_core::validate(&fixture(&scratch()));
    assert_eq!(errors(&report), []);
}

#[test]
fn the_fixture_paints_each_case_where_its_points_say() {
    let frame = painted_at(&fixture(&scratch()), 0);
    let at = |x, y| pixel(&frame, 400, x, y);
    // Each case: the pixel and what it must be, worked by hand from the points, the box's
    // top-left corner and a centred stroke.
    let cases: [(&str, (usize, usize), [u8; 3]); 13] = [
        // The line: box (20, 20); y 5 in the box is 25 in the frame, and a 4 px stroke
        // centred on it covers rows 23 to 26.
        ("line, on its stroke", (100, 25), WHITE),
        ("line, below its stroke", (100, 29), BLACK),
        // The triangle: box (220, 20). Its base runs along y 118, frame row 138.
        ("triangle base, on", (300, 138), WHITE),
        ("triangle base, below", (300, 142), BLACK),
        // Its left side, from (2, 118) to (80, 2), crosses box y 60.5 at x 40.7.
        ("triangle left side, on", (261, 80), WHITE),
        ("triangle left side, outside", (255, 80), BLACK),
        // Its right side, the mirror image about x 80.
        ("triangle right side, on", (338, 80), WHITE),
        ("triangle right side, outside", (345, 80), BLACK),
        ("triangle, filled inside", (300, 100), [255, 0, 0]),
        // The S-curve: box (20, 160), control points (10, 60), (50, 10), (110, 110),
        // (150, 60). At t = 0.5 it is at (80, 60), frame (100, 220), leaving at slope 0.5.
        ("s-curve, on", (100, 220), [0, 255, 0]),
        ("s-curve, ten pixels below", (100, 230), BLACK),
        // The star: box (220, 160). Its centre, (80, 60), is wound twice: nonzero fills it,
        // where even-odd would leave it empty.
        ("star, centre", (300, 220), [255, 255, 0]),
        ("star, outside its points", (225, 165), BLACK),
    ];
    let wrong: Vec<String> = cases
        .iter()
        .filter(|(_, (x, y), want)| at(*x, *y) != *want)
        .map(|(name, (x, y), want)| format!("{name} ({x}, {y}): {:?}, not {want:?}", at(*x, *y)))
        .collect();
    assert!(wrong.is_empty(), "{wrong:#?}");

    // The gradient triangle: box (20, 40), 160 wide, a linear gradient from red at the box's
    // left edge to blue at its right. It is measured against the box, not the triangle: box
    // x 20.5 is an eighth of the way across, mostly red, and box x 140.5 seven eighths.
    let [r, _, b] = at(40, 140);
    assert!(r > 200 && b < 50, "near the left: {:?}", at(40, 140));
    let [r, _, b] = at(160, 140);
    assert!(r < 50 && b > 200, "near the right: {:?}", at(160, 140));
    assert_eq!(at(25, 45), BLACK, "outside the triangle, inside its box");
}

#[test]
fn a_written_zero_handle_paints_the_same_bytes_as_none() {
    let stroked = || with(triangle(), json!({"stroke": "#00FF00", "stroke_width": 4}));
    let bare = painted_at(&project(&scratch(), &[stroked()]), 0);
    let zeroed = with(
        stroked(),
        json!({"points": [{"at": [10, 90], "in": [0, 0], "out": [0, 0]},
                          {"at": [50, 10], "out": [0, 0]}, {"at": [90, 90], "in": [0, 0]}]}),
    );
    assert!(bare == painted_at(&project(&scratch(), &[zeroed]), 0));
    assert!(bare.iter().any(|byte| *byte != 0), "the triangle paints");
}

#[test]
fn a_two_keyframe_triangle_paints_the_in_between_value_at_its_midpoint() {
    // The box's top-left corner is frame (50, 50). The apex sinks from box y 10 to 50 over a
    // second, so at 500 ms it is at box y 30, frame y 80.
    let frame = painted_at(&project(&scratch(), &[sinking(json!("linear"))]), 500);
    let at = |x, y| pixel(&frame, 200, x, y);
    assert_eq!(at(100, 84), WHITE, "just under the apex at 500 ms");
    assert_eq!(at(100, 76), BLACK, "just over it");
}

#[test]
fn an_overshooting_ease_keeps_the_drawing_inside_the_inset_box() {
    // The ease carries the apex up towards box y -34. A 4 px stroke makes the inset 2, so it
    // is clamped to box y 2, frame y 52, and its round join reaches the box's top edge at
    // frame y 50 and no further. Unclamped, the fill alone would cover row 49.
    let element = with(
        sinking(json!([0.5, -3.0, 0.5, 1.0])),
        json!({"stroke": "#FFFFFF", "stroke_width": 4}),
    );
    let frame = painted_at(&project(&scratch(), &[element]), 300);
    let at = |x, y| pixel(&frame, 200, x, y);
    assert_eq!(at(100, 49), BLACK, "just outside the box");
    assert_eq!(at(100, 52), WHITE, "the clamped apex");
}

// ---------------------------------------------------------------------------
// The four `validate` errors.
// ---------------------------------------------------------------------------

/// The fields of every `code` finding `validate` gives `element`.
fn findings(element: Value, code: &str) -> Vec<Value> {
    let report = montagent_core::validate(&project(&scratch(), &[element]));
    report
        .findings
        .iter()
        .filter(|finding| finding.code == code)
        .map(|finding| {
            assert_eq!(finding.class, Class::Error, "{code} is an error");
            finding
                .fields
                .iter()
                .map(|(name, value)| (name.to_string(), value.clone()))
                .collect::<serde_json::Map<_, _>>()
                .into()
        })
        .collect()
}

/// An open, stroked path in the 100×100 box.
fn open(points: Value) -> Value {
    with(
        triangle(),
        json!({"closed": false, "fill": null, "stroke": "#FFFFFF", "stroke_width": 2,
               "points": points}),
    )
}

#[test]
fn too_few_points_fires_below_two_open_and_three_closed() {
    let fires = |element| findings(element, "E-PATH-TOO-FEW-POINTS");
    assert_eq!(fires(open(json!([{"at": [10, 10]}]))).len(), 1);
    assert_eq!(
        fires(open(json!([{"at": [10, 10]}, {"at": [90, 90]}]))),
        Vec::<Value>::new()
    );
    let closed_two = with(
        triangle(),
        json!({"points": [{"at": [10, 10]}, {"at": [90, 90]}]}),
    );
    assert_eq!(fires(closed_two).len(), 1);
    assert_eq!(fires(triangle()), Vec::<Value>::new());
}

#[test]
fn too_few_points_fires_on_a_keyframe_value_and_names_its_record() {
    let element = with(
        triangle(),
        json!({"points": [
            {"t": 0, "v": [{"at": [10, 90]}, {"at": [50, 10]}, {"at": [90, 90]}]},
            {"t": 500, "v": [{"at": [10, 90]}, {"at": [90, 90]}], "ease": "linear"}
        ]}),
    );
    let fired = findings(element, "E-PATH-TOO-FEW-POINTS");
    assert_eq!(fired.len(), 1, "{fired:?}");
    assert_eq!(fired[0]["record"], json!(2));
}

#[test]
fn a_dangling_handle_fires_on_an_open_path_only() {
    let fires = |element| findings(element, "E-PATH-DANGLING-HANDLE");
    let arriving = fires(open(
        json!([{"at": [10, 10], "in": [5, 0]}, {"at": [90, 90]}]),
    ));
    assert_eq!(arriving.len(), 1);
    assert_eq!(
        (arriving[0]["vertex"].clone(), arriving[0]["handle"].clone()),
        (json!(0), json!("in"))
    );
    let leaving = fires(open(
        json!([{"at": [10, 10]}, {"at": [90, 90], "out": [-5, 0]}]),
    ));
    assert_eq!(leaving.len(), 1);
    assert_eq!(
        (leaving[0]["vertex"].clone(), leaving[0]["handle"].clone()),
        (json!(1), json!("out"))
    );
    // On a closed path both shape the closing segment.
    let closed = with(
        triangle(),
        json!({"points": [{"at": [10, 90], "in": [5, 0]}, {"at": [50, 10]},
                          {"at": [90, 90], "out": [-5, 0]}]}),
    );
    assert_eq!(fires(closed), Vec::<Value>::new());
    assert_eq!(
        fires(open(
            json!([{"at": [10, 10], "out": [5, 0]}, {"at": [90, 90], "in": [-5, 0]}])
        )),
        Vec::<Value>::new()
    );
}

#[test]
fn a_keyframe_list_whose_values_differ_in_count_or_handles_names_the_first_difference() {
    let fires = |element| findings(element, "E-PATH-KEYFRAME-SHAPE");
    let keyed = |second: Value| {
        with(
            triangle(),
            json!({"points": [
                {"t": 0, "v": [{"at": [10, 90]}, {"at": [50, 10]}, {"at": [90, 90]}]},
                {"t": 500, "v": [{"at": [10, 90]}, {"at": [50, 10]}, {"at": [90, 90]}], "ease": "linear"},
                {"t": 1000, "v": second, "ease": "linear"}
            ]}),
        )
    };
    let counted = fires(keyed(
        json!([{"at": [10, 90]}, {"at": [50, 10]}, {"at": [90, 90]}, {"at": [10, 50]}]),
    ));
    assert_eq!(counted.len(), 1, "{counted:?}");
    assert_eq!(
        (counted[0]["record"].clone(), counted[0]["vertex"].clone()),
        (json!(3), json!(3))
    );
    let handled = fires(keyed(
        json!([{"at": [10, 90]}, {"at": [50, 10], "out": [0, 0]}, {"at": [90, 90]}]),
    ));
    assert_eq!(handled.len(), 1, "{handled:?}");
    assert_eq!(
        (handled[0]["record"].clone(), handled[0]["vertex"].clone()),
        (json!(3), json!(1))
    );
    assert_eq!(fires(sinking(json!("linear"))), Vec::<Value>::new());
}

#[test]
fn outside_the_box_fires_on_a_handle_whose_vertex_is_inside() {
    let element = with(
        triangle(),
        json!({"points": [{"at": [10, 90]}, {"at": [50, 10], "out": [60, 0]}, {"at": [90, 90]}]}),
    );
    let fired = findings(element, "E-PATH-OUTSIDE-BOX");
    assert_eq!(fired.len(), 1, "{fired:?}");
    let finding = &fired[0];
    assert_eq!(finding["vertex"], json!(1));
    assert_eq!(finding["handle"], json!("out"));
    assert_eq!(finding["position"], json!([110, 10]));
    assert_eq!(finding["inset"], json!(0));
    assert_eq!(
        findings(triangle(), "E-PATH-OUTSIDE-BOX"),
        Vec::<Value>::new()
    );
}

#[test]
fn an_odd_stroke_width_insets_by_its_half_rounded_up() {
    // `stroke_width` 5: the inset is 3, so a vertex at 2 is outside and one at 3 is not.
    let at = |y: i64| {
        with(
            triangle(),
            json!({"stroke": "#FFFFFF", "stroke_width": 5,
                   "points": [{"at": [10, 90]}, {"at": [50, y]}, {"at": [90, 90]}]}),
        )
    };
    let fired = findings(at(2), "E-PATH-OUTSIDE-BOX");
    assert_eq!(fired.len(), 1, "{fired:?}");
    assert_eq!(fired[0]["inset"], json!(3));
    assert_eq!(fired[0]["position"], json!([50, 2]));
    assert_eq!(findings(at(3), "E-PATH-OUTSIDE-BOX"), Vec::<Value>::new());
}

#[test]
fn a_keyed_stroke_width_insets_by_its_largest_value() {
    let element = with(
        triangle(),
        json!({"stroke": "#FFFFFF",
               "stroke_width": [{"t": 0, "v": 2}, {"t": 500, "v": 6, "ease": "linear"}],
               "points": [{"at": [10, 90]}, {"at": [50, 2]}, {"at": [90, 90]}]}),
    );
    let fired = findings(element, "E-PATH-OUTSIDE-BOX");
    assert_eq!(fired.len(), 1, "{fired:?}");
    assert_eq!(fired[0]["inset"], json!(3));
}

#[test]
fn outside_the_box_checks_every_keyframe_value_and_names_its_record() {
    let element = with(
        triangle(),
        json!({"points": [
            {"t": 0, "v": [{"at": [10, 90]}, {"at": [50, 10]}, {"at": [90, 90]}]},
            {"t": 500, "v": [{"at": [10, 90]}, {"at": [50, 101]}, {"at": [90, 90]}], "ease": "linear"}
        ]}),
    );
    let fired = findings(element, "E-PATH-OUTSIDE-BOX");
    assert_eq!(fired.len(), 1, "{fired:?}");
    assert_eq!(fired[0]["record"], json!(2));
    assert_eq!(fired[0]["position"], json!([50, 101]));
}

// ---------------------------------------------------------------------------
// `query --at`.
// ---------------------------------------------------------------------------

/// `query --at`'s answer for the one element of `element`'s project, as JSON.
fn queried(element: Value, t: i64) -> Value {
    let path = project(&scratch(), &[element]);
    let document = montagent_core::parse::read(&path).unwrap();
    let answer = serde_json::to_value(at::at(&document, t, None)).unwrap();
    answer["stack"][0].clone()
}

#[test]
fn query_at_prints_a_paths_resolved_vertices_with_absolute_control_points() {
    let present = queried(sinking(json!("linear")), 500);
    assert_eq!(
        present["path"],
        json!([
            {"at": [10.0, 90.0]},
            {"at": [50.0, 30.0], "out": [70.0, 30.0]},
            {"at": [90.0, 90.0]}
        ]),
        "{present}"
    );
    // The relative form is still the one resolved value the one list prints.
    let points = present["values"]
        .as_array()
        .unwrap()
        .iter()
        .find(|value| value["property"] == "points")
        .unwrap();
    assert_eq!(points["animated"], json!(true));
}

#[test]
fn not_covered_counts_a_paths_declared_box() {
    let answer = {
        let path = project(&scratch(), &[with(triangle(), json!({"x": 100, "y": 100}))]);
        let document = montagent_core::parse::read(&path).unwrap();
        serde_json::to_value(at::at(&document, 0, None)).unwrap()
    };
    // The 200×200 frame less the box (50, 50)–(150, 150), whatever the triangle draws.
    let uncovered: i64 = answer["not_covered"]
        .as_array()
        .unwrap()
        .iter()
        .map(|rect| rect["width"].as_i64().unwrap() * rect["height"].as_i64().unwrap())
        .sum();
    assert_eq!(uncovered, 200 * 200 - 100 * 100, "{answer}");
}

// ---------------------------------------------------------------------------
// `shift`.
// ---------------------------------------------------------------------------

fn shifted(path: &Path, at: i64) -> montagent_core::verbs::shift::Answer {
    montagent_core::verbs::shift::shift(
        path,
        &montagent_core::verbs::shift::Ask {
            at,
            delta: 100,
            scope: None,
            release: Vec::new(),
        },
    )
}

#[test]
fn shift_cuts_keyed_points_where_every_resolved_number_is_an_integer() {
    // At 250 ms the apex is at box y 20 and its handle at 15: whole numbers.
    let path = project(&scratch(), &[sinking(json!("linear"))]);
    let answer = shifted(&path, 250);
    assert_eq!(
        answer.report().exit_code(),
        ExitCode::Ok,
        "{:?}",
        answer.report().findings
    );
    let written = common::document(&path);
    let apex: Vec<(i64, Value)> = written["tracks"][0]["elements"][0]["points"]
        .as_array()
        .unwrap()
        .iter()
        .map(|record| (record["t"].as_i64().unwrap(), record["v"][1].clone()))
        .collect();
    assert_eq!(
        apex,
        [
            (0, json!({"at": [50, 10], "out": [10, 0]})),
            (250, json!({"at": [50, 20], "out": [15, 0]})),
            (350, json!({"at": [50, 20], "out": [15, 0]})),
            (1100, json!({"at": [50, 50], "out": [30, 0]})),
        ]
    );
}

#[test]
fn shift_refuses_a_cut_where_a_resolved_coordinate_is_fractional() {
    // At 333 ms the apex is at box y 23.32.
    let path = project(&scratch(), &[sinking(json!("linear"))]);
    let before = std::fs::read_to_string(&path).unwrap();
    let answer = shifted(&path, 333);
    let refusals: Vec<_> = answer
        .report()
        .findings
        .iter()
        .filter(|finding| finding.code == "E-SHIFT-SPLIT-UNWRITABLE")
        .collect();
    assert_eq!(refusals.len(), 1, "{:?}", answer.report().findings);
    assert_eq!(refusals[0].fields["property"], "points");
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        before,
        "nothing is written"
    );
}

// ---------------------------------------------------------------------------
// The published schema says what the deserializer says.
// ---------------------------------------------------------------------------

#[test]
fn the_published_schema_refuses_fill_on_an_open_path() {
    let schema = montagent_core::schema::generate();
    let branch = schema["$defs"]["Element"]["oneOf"]
        .as_array()
        .unwrap()
        .iter()
        .find(|branch| branch["properties"]["type"]["const"] == "path")
        .expect("a path branch");
    assert_eq!(
        branch["if"],
        json!({"properties": {"closed": {"const": false}}, "required": ["closed"]})
    );
    assert_eq!(branch["then"]["not"], json!({"required": ["fill"]}));
    assert_eq!(
        branch["required"],
        json!([
            "id", "type", "start", "end", "width", "height", "closed", "points"
        ])
    );
}

// ---------------------------------------------------------------------------
// `fmt` and `timeline` carry the element.
// ---------------------------------------------------------------------------

#[test]
fn fmt_rewrites_a_path_without_changing_it_and_timeline_lists_it() {
    use montagent_core::verbs::fmt::{Mode, fmt};

    let dir = scratch();
    let raw = std::fs::read_to_string(
        common::fixture_dir()
            .parent()
            .unwrap()
            .join("path/paths.montagent.json"),
    )
    .unwrap();
    let path = write_project(&dir, "paths.montagent.json", &raw);
    let before = common::document(&path);
    assert_eq!(fmt(&path, Mode::Write).exit_code(), ExitCode::Ok);
    assert_eq!(common::document(&path), before, "the same document");
    assert!(
        fmt(&path, Mode::Check).findings.is_empty(),
        "canonical after one rewrite"
    );

    let view = montagent_core::verbs::timeline::timeline(&path).to_json();
    let details: Vec<(String, String)> = view["timeline"]["groups"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|group| group["elements"].as_array().unwrap())
        .map(|element| {
            (
                element["id"].as_str().unwrap().to_string(),
                element["detail"].as_str().unwrap().to_string(),
            )
        })
        .collect();
    assert!(
        details.contains(&("triangle".to_string(), "160×120 #FF0000".to_string())),
        "{details:?}"
    );
}
