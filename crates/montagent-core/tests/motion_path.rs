//! Motion paths, arc-length `offset`, and `orient` (#1091, ADR-0218).

use std::path::{Path, PathBuf};

use montagent_core::animatable::{self, Kind};
use montagent_core::finding::Class;
use montagent_core::report::{ExitCode, Report};
use montagent_core::verbs::query::at;
use montagent_core::verbs::render::{Supplying, paint_span};
use serde_json::{Value, json};

mod common;
use common::{canonical, tempdir, write_project};

const WIDTH: usize = 640;
const HEIGHT: usize = 360;

fn scratch() -> PathBuf {
    use std::sync::atomic::{AtomicU32, Ordering};
    static NEXT: AtomicU32 = AtomicU32::new(0);
    tempdir(2_180_000 + NEXT.fetch_add(1, Ordering::Relaxed))
}

fn project(dir: &Path, elements: &[Value]) -> PathBuf {
    let tracks: Vec<Value> = elements
        .iter()
        .enumerate()
        .map(|(i, element)| json!({"name": format!("t{i}"), "layer": i, "elements": [element]}))
        .collect();
    write_project(
        dir,
        "p.montagent.json",
        &canonical(
            &json!({
                "frame": {"width": WIDTH, "height": HEIGHT},
                "fps": 30,
                "background": "#000000",
                "duration": 1000,
                "output": "out/p.mp4",
                "tracks": tracks,
            })
            .to_string(),
        ),
    )
}

fn codes(report: &Report) -> Vec<&str> {
    report.findings.iter().map(|f| f.code.as_str()).collect()
}

fn resolved_value(present: &Value, prop: &str) -> Option<f64> {
    present["values"]
        .as_array()?
        .iter()
        .find(|v| v["property"] == prop)?
        .get("value")?
        .as_f64()
}

#[test]
fn schema_exposes_motion_path_offset_on_all_six_visual_elements_only() {
    let prop = animatable::property("motion_path.offset")
        .expect("motion_path.offset must be in animatable table");
    assert_eq!(prop.kind, Kind::Number);
    assert_eq!(prop.minimum, Some(-1.0));
    assert_eq!(prop.maximum, Some(2.0));

    for kind in ["image", "video", "text", "rect", "ellipse", "path"] {
        let props = animatable::of_element(&json!({"type": kind}));
        assert!(
            props.iter().any(|p| p.name == "motion_path.offset"),
            "{kind} must carry animatable motion_path.offset"
        );
    }
    for kind in ["nest", "audio", "transition"] {
        let props = animatable::of_element(&json!({"type": kind}));
        assert!(
            !props.iter().any(|p| p.name == "motion_path.offset"),
            "{kind} must not carry motion_path.offset"
        );
    }
}

#[test]
fn validate_rejects_motion_path_alongside_x_or_y_with_e_motion_path_xy() {
    let dir = scratch();
    let with_x = json!({
        "id": "box_x",
        "type": "rect",
        "start": 0,
        "end": 1000,
        "x": 10,
        "motion_path": {
            "closed": false,
            "points": [{"at": [0, 0]}, {"at": [100, 0]}]
        },
        "width": 50,
        "height": 50,
        "fill": "#FF0000"
    });
    let with_xy = json!({
        "id": "box_xy",
        "type": "rect",
        "start": 0,
        "end": 1000,
        "x": 10,
        "y": 20,
        "motion_path": {
            "closed": false,
            "points": [{"at": [0, 0]}, {"at": [100, 0]}]
        },
        "width": 50,
        "height": 50,
        "fill": "#00FF00"
    });
    let path = project(&dir, &[with_x, with_xy]);
    let report = montagent_core::validate(&path);
    assert_eq!(report.exit_code(), ExitCode::Errors);
    let xy_findings: Vec<_> = report
        .findings
        .iter()
        .filter(|f| f.code == "E-MOTION-PATH-XY")
        .collect();
    assert_eq!(xy_findings.len(), 2, "{:?}", report.findings);
    assert_eq!(xy_findings[0].fields["fields"], json!("`x`"));
    assert_eq!(xy_findings[1].fields["fields"], json!("`x` and `y`"));
}

#[test]
fn validate_checks_curve_geometry_and_offset_range_on_motion_path() {
    let dir = scratch();
    // 1. Too few points (1 vertex)
    let too_few = json!({
        "id": "too_few",
        "type": "rect",
        "start": 0,
        "end": 1000,
        "motion_path": {
            "closed": false,
            "points": [{"at": [10, 20]}]
        },
        "width": 40,
        "height": 40,
        "fill": "#FFFFFF"
    });
    // 2. Dangling handle on open path
    let dangling = json!({
        "id": "dangling",
        "type": "rect",
        "start": 0,
        "end": 1000,
        "motion_path": {
            "closed": false,
            "points": [
                {"at": [10, 20], "in": [-5, 0]},
                {"at": [110, 20]}
            ]
        },
        "width": 40,
        "height": 40,
        "fill": "#FFFFFF"
    });
    // 3. Zero-length motion path
    let zero_len = json!({
        "id": "zero_len",
        "type": "rect",
        "start": 0,
        "end": 1000,
        "motion_path": {
            "closed": false,
            "points": [
                {"at": [50, 50]},
                {"at": [50, 50]}
            ]
        },
        "width": 40,
        "height": 40,
        "fill": "#FFFFFF"
    });
    // 4. Open path offset outside [0.0, 1.0] (static and keyframed)
    let open_oob_static = json!({
        "id": "open_oob_static",
        "type": "rect",
        "start": 0,
        "end": 1000,
        "motion_path": {
            "closed": false,
            "points": [{"at": [0, 0]}, {"at": [100, 0]}],
            "offset": 1.25
        },
        "width": 40,
        "height": 40,
        "fill": "#FFFFFF"
    });
    let open_oob_keyed = json!({
        "id": "open_oob_keyed",
        "type": "rect",
        "start": 0,
        "end": 1000,
        "motion_path": {
            "closed": false,
            "points": [{"at": [0, 0]}, {"at": [100, 0]}],
            "offset": [
                {"t": 0, "v": -0.2},
                {"t": 1000, "v": 1.0, "ease": "linear"}
            ]
        },
        "width": 40,
        "height": 40,
        "fill": "#FFFFFF"
    });
    // 5. Closed path with offset in [-1.0, 2.0] and endpoint handles is valid!
    let closed_valid = json!({
        "id": "closed_valid",
        "type": "rect",
        "start": 0,
        "end": 1000,
        "motion_path": {
            "closed": true,
            "points": [
                {"at": [100, 100], "in": [0, 20], "out": [20, 0]},
                {"at": [200, 200], "in": [0, -20], "out": [-20, 0]}
            ],
            "offset": [
                {"t": 0, "v": -0.5},
                {"t": 900, "v": 1.5, "ease": "linear"}
            ]
        },
        "width": 40,
        "height": 40,
        "fill": "#FFFFFF"
    });

    let path = project(
        &dir,
        &[
            too_few,
            dangling,
            zero_len,
            open_oob_static,
            open_oob_keyed,
            closed_valid,
        ],
    );
    let report = montagent_core::validate(&path);
    let found = codes(&report);
    assert!(found.contains(&"E-PATH-TOO-FEW-POINTS"), "{found:?}");
    assert!(found.contains(&"E-PATH-DANGLING-HANDLE"), "{found:?}");
    assert!(found.contains(&"E-MOTION-PATH-ZERO-LENGTH"), "{found:?}");
    let offset_errs: Vec<_> = report
        .findings
        .iter()
        .filter(|f| f.code == "E-MOTION-PATH-OFFSET-RANGE")
        .collect();
    assert_eq!(offset_errs.len(), 2, "{offset_errs:?}");
    assert!(
        !report
            .findings
            .iter()
            .any(|f| f.location.element.as_deref() == Some("closed_valid")),
        "closed_valid must produce no findings: {:?}",
        report.findings
    );
}

#[test]
fn non_visual_elements_and_animated_points_rejected_with_e_schema() {
    let dir = scratch();
    let bad_nest = json!({
        "id": "bad_nest",
        "type": "nest",
        "start": 0,
        "end": 1000,
        "motion_path": {
            "closed": false,
            "points": [{"at": [0, 0]}, {"at": [100, 0]}]
        },
        "tracks": []
    });
    let animated_points = json!({
        "id": "anim_pts",
        "type": "rect",
        "start": 0,
        "end": 1000,
        "motion_path": {
            "closed": false,
            "points": [
                {"t": 0, "v": [{"at": [0, 0]}, {"at": [100, 0]}]},
                {"t": 1000, "v": [{"at": [0, 0]}, {"at": [200, 0]}], "ease": "linear"}
            ]
        },
        "width": 40,
        "height": 40,
        "fill": "#FFFFFF"
    });
    let path = project(&dir, &[bad_nest, animated_points]);
    let report = montagent_core::validate(&path);
    let schema_errs: Vec<_> = report
        .findings
        .iter()
        .filter(|f| f.code == "E-SCHEMA")
        .collect();
    assert_eq!(schema_errs.len(), 2, "{:?}", report.findings);
}

#[test]
fn arc_length_parameterisation_closed_wrapping_and_orient_report_in_query_at() {
    let dir = scratch();
    // Asymmetric 1D cubic Bezier from (100, 100) to (400, 100) with handles at (100, 100) and
    // (110, 100): raw Bezier parameter t = 0.5 is NOT at x = 250, but arc-length offset = 0.5
    // MUST land at distance 150 px -> x = 250.0, y = 100.0!
    let asymmetric_cubic = json!({
        "id": "cubic",
        "type": "rect",
        "start": 0,
        "end": 1000,
        "motion_path": {
            "closed": false,
            "points": [
                {"at": [100, 100], "out": [0, 0]},
                {"at": [400, 100], "in": [-290, 0]}
            ],
            "offset": [
                {"t": 0, "v": 0.0},
                {"t": 1000, "v": 1.0, "ease": "linear"}
            ]
        },
        "origin": "center",
        "width": 40,
        "height": 20,
        "fill": "#FF0000"
    });

    // Closed square of perimeter 400 px:
    // (100, 100) -> (200, 100) -> (200, 200) -> (100, 200) -> (100, 100)
    // At t = 0: offset = -0.25 -> u = 0.75 -> d = 300 -> (100, 200), heading up (-90 deg).
    // At t = 500: offset = 0.5 -> u = 0.5 -> d = 200 -> (200, 200), heading left (180 or -180).
    // At t = 750: offset = 0.875 -> u = 0.875 -> d = 350 -> (100, 150), heading up (-90 deg) + base rotation 15 deg = -75 deg.
    let closed_orient = json!({
        "id": "loop_box",
        "type": "rect",
        "start": 0,
        "end": 1000,
        "motion_path": {
            "closed": true,
            "points": [
                {"at": [100, 100]},
                {"at": [200, 100]},
                {"at": [200, 200]},
                {"at": [100, 200]}
            ],
            "offset": [
                {"t": 0, "v": -0.25},
                {"t": 1000, "v": 1.25, "ease": "linear"}
            ],
            "orient": true
        },
        "origin": "center",
        "rotation": 15.0,
        "width": 30,
        "height": 10,
        "fill": "#00FF00"
    });

    let path = project(&dir, &[asymmetric_cubic, closed_orient]);
    let report = montagent_core::validate(&path);
    assert!(
        !report.findings.iter().any(|f| f.class == Class::Error),
        "{:?}",
        report.findings
    );

    let document = montagent_core::parse::read(&path).unwrap();
    let answer_500 = serde_json::to_value(at::at(&document, 500, None)).unwrap();
    let stack_500 = answer_500["stack"].as_array().unwrap();
    let cubic_500 = &stack_500[0];
    assert!((resolved_value(cubic_500, "x").unwrap() - 250.0).abs() < 0.5);
    assert!((resolved_value(cubic_500, "y").unwrap() - 100.0).abs() < 1e-4);
    assert!((resolved_value(cubic_500, "motion_path.offset").unwrap() - 0.5).abs() < 1e-6);
    assert!((cubic_500["derived"]["length"].as_f64().unwrap() - 300.0).abs() < 0.1);
    assert!((cubic_500["derived"]["distance"].as_f64().unwrap() - 150.0).abs() < 0.1);
    assert!((cubic_500["derived"]["tangent_angle"].as_f64().unwrap() - 0.0).abs() < 1e-4);

    // At t = 750: offset = -0.25 + 0.75 * 1.5 = 0.875 -> distance = 350 px -> (100, 150), tangent = -90 deg, effective rotation = 15 + (-90) = -75 deg.
    let answer_750 = serde_json::to_value(at::at(&document, 750, None)).unwrap();
    let loop_750 = &answer_750["stack"].as_array().unwrap()[1];
    assert!((resolved_value(loop_750, "x").unwrap() - 100.0).abs() < 1e-4);
    assert!((resolved_value(loop_750, "y").unwrap() - 150.0).abs() < 1e-4);
    assert!((resolved_value(loop_750, "rotation").unwrap() - (-75.0)).abs() < 1e-4);
    assert!((loop_750["derived"]["length"].as_f64().unwrap() - 400.0).abs() < 1e-4);
    assert!((loop_750["derived"]["distance"].as_f64().unwrap() - 350.0).abs() < 1e-4);
    assert!((loop_750["derived"]["tangent_angle"].as_f64().unwrap() - (-90.0)).abs() < 1e-4);
}

#[test]
fn static_checks_r_off_canvas_and_r_motion_blur_still_use_motion_path() {
    let dir = scratch();
    // Moving element that crosses the canvas and has motion_blur: neither R-OFF-CANVAS nor R-MOTION-BLUR-STILL should fire.
    let moving_on_canvas = json!({
        "id": "moving_ok",
        "type": "rect",
        "start": 0,
        "end": 1000,
        "motion_path": {
            "closed": false,
            "points": [{"at": [-100, 180]}, {"at": [740, 180]}],
            "offset": [
                {"t": 0, "v": 0.0},
                {"t": 1000, "v": 1.0, "ease": "linear"}
            ]
        },
        "width": 50,
        "height": 50,
        "fill": "#FF0000",
        "motion_blur": {"shutter": 180, "samples": 5}
    });
    // Still element on motion_path far off canvas with motion_blur: BOTH R-OFF-CANVAS and R-MOTION-BLUR-STILL must fire.
    let still_off_canvas = json!({
        "id": "still_off",
        "type": "rect",
        "start": 0,
        "end": 1000,
        "motion_path": {
            "closed": false,
            "points": [{"at": [2000, 2000]}, {"at": [2100, 2000]}],
            "offset": 0.5
        },
        "width": 50,
        "height": 50,
        "fill": "#00FF00",
        "motion_blur": {"shutter": 180, "samples": 5}
    });
    let path = project(&dir, &[moving_on_canvas, still_off_canvas]);
    let report = montagent_core::validate(&path);
    let off_canvas: Vec<_> = report
        .findings
        .iter()
        .filter(|f| f.code == "R-OFF-CANVAS")
        .collect();
    assert_eq!(off_canvas.len(), 1, "{off_canvas:?}");
    assert_eq!(off_canvas[0].location.element.as_deref(), Some("still_off"));

    let blur_still: Vec<_> = report
        .findings
        .iter()
        .filter(|f| f.code == "R-MOTION-BLUR-STILL")
        .collect();
    assert_eq!(blur_still.len(), 1, "{blur_still:?}");
    assert_eq!(blur_still[0].location.element.as_deref(), Some("still_off"));
}

#[test]
fn painter_draws_element_at_motion_path_position_and_orient_angle() {
    let dir = scratch();
    let el = json!({
        "id": "runner",
        "type": "rect",
        "start": 0,
        "end": 1000,
        "motion_path": {
            "closed": false,
            "points": [{"at": [100, 180]}, {"at": [500, 180]}],
            "offset": [
                {"t": 0, "v": 0.0},
                {"t": 1000, "v": 1.0, "ease": "linear"}
            ]
        },
        "origin": "center",
        "width": 40,
        "height": 40,
        "fill": "#FF0000"
    });
    let path = project(&dir, &[el]);
    let rasters = paint_span(&path, 500, 501, Supplying::PerFrame).expect("the span paints");
    assert!(rasters.declined.is_empty(), "{:?}", rasters.declined);
    let rgb = rasters.frames.into_iter().next().unwrap();
    let px = |x: usize, y: usize| -> [u8; 3] {
        let idx = (y * WIDTH + x) * 3;
        [rgb[idx], rgb[idx + 1], rgb[idx + 2]]
    };
    assert_eq!(px(300, 180), [255, 0, 0]);
    assert_eq!(px(100, 180), [0, 0, 0]);
}
