//! **Nest** (prototype #780): a container for space only, composed before each child's own
//! transform.
//!
//! What the prototype owes the ruling (#632), asked through each verb: the matrix itself, a
//! nest at rest as a no-op, the one conformance test that every reader of a child's place
//! sees its *composed* position, the new findings through `validate`, `shift` carrying a
//! nest's keyframes, and `query --at`'s nest rows.

use std::path::Path;

use montagent_core::nest::{Affine, matrix_of};
use montagent_core::report::{ExitCode, Report};
use montagent_core::validate;
use montagent_core::verbs::frame::{Ask as FrameAsk, frame};
use montagent_core::verbs::query::{self, Ask as QueryAsk};
use montagent_core::verbs::shift;
use serde_json::{Value, json};

mod common;
use common::{canonical, tempdir, write_project};

const W: u32 = 400;
const H: u32 = 300;

/// The codes a report carries, less the layout notes a test's hand-built file may draw.
fn codes(report: &Report) -> Vec<&str> {
    report
        .findings
        .iter()
        .map(|f| f.code.as_str())
        .filter(|code| !code.starts_with("L-"))
        .collect()
}

/// The prose of finding `index`, as `validate` prints it, its wrapped lines joined with
/// single spaces.
fn message_of(report: &Report, index: usize) -> String {
    let prose =
        montagent_core::text::render(&report.to_json(), montagent_core::text::Options::verbose())
            .expect("every template's fields are carried");
    let code = &report.findings[index].code;
    let lines: Vec<&str> = prose.lines().collect();
    let at = lines
        .iter()
        .position(|line| line.contains(code.as_str()))
        .expect("the finding is printed");
    lines[at + 1..]
        .iter()
        .take_while(|line| line.starts_with("  ") && !line.trim().is_empty())
        .map(|line| line.trim())
        .collect::<Vec<_>>()
        .join(" ")
}

fn project(tracks: Value) -> Value {
    json!({"frame": {"width": W, "height": H}, "fps": 25, "background": "#000000",
           "duration": 1000, "tracks": tracks})
}

fn keyed(from: f64, to: f64) -> Value {
    json!([{"t": 0, "v": from}, {"t": 1000, "v": to, "ease": "linear"}])
}

fn keyed_int(from: i64, to: i64) -> Value {
    json!([{"t": 0, "v": from}, {"t": 1000, "v": to, "ease": "linear"}])
}

fn rect(id: &str, extra: Value) -> Value {
    let mut element = json!({"id": id, "type": "rect", "start": 0, "end": 1000,
                             "x": 150, "y": 120, "origin": "center",
                             "width": 60, "height": 30, "fill": "#FF8800"});
    for (key, value) in extra.as_object().expect("an object") {
        element[key] = value.clone();
    }
    element
}

fn nest(id: &str, extra: Value, tracks: Value) -> Value {
    let mut element = json!({"id": id, "type": "nest", "start": 0, "end": 1000,
                             "pivot": [200, 150], "tracks": tracks});
    for (key, value) in extra.as_object().expect("an object") {
        element[key] = value.clone();
    }
    element
}

fn track(name: &str, layer: i64, elements: Vec<Value>) -> Value {
    json!({"name": name, "layer": layer, "elements": elements})
}

#[track_caller]
fn written(line: u32, doc: &Value) -> std::path::PathBuf {
    let dir = tempdir(line);
    write_project(&dir, "p.montagent.json", &canonical(&doc.to_string()))
}

/// The exact pixels at `instant`, true scale and lossless.
#[track_caller]
fn painted(path: &Path, instant: i64) -> (Vec<u8>, image::RgbaImage) {
    let answer = frame(
        path,
        &FrameAsk {
            at: Some(instant),
            full: true,
            png: true,
            ..FrameAsk::default()
        },
    );
    assert_eq!(
        answer.report().exit_code(),
        ExitCode::Ok,
        "{}",
        serde_json::to_string_pretty(&answer.to_json()).unwrap_or_default()
    );
    let bytes = answer.image().expect("a picture").bytes.clone();
    let picture = image::load_from_memory(&bytes)
        .expect("decodes")
        .to_rgba8();
    (bytes, picture)
}

/// The bounding box of every pixel that is not the background: `(left, top, right, bottom)`,
/// right and bottom exclusive.
fn ink(picture: &image::RgbaImage) -> Option<(u32, u32, u32, u32)> {
    let mut found: Option<(u32, u32, u32, u32)> = None;
    for (x, y, pixel) in picture.enumerate_pixels() {
        if pixel.0[..3] != [0, 0, 0] {
            found = Some(match found {
                None => (x, y, x + 1, y + 1),
                Some((l, t, r, b)) => (l.min(x), t.min(y), r.max(x + 1), b.max(y + 1)),
            });
        }
    }
    found
}

fn query_at(path: &Path, instant: i64) -> Value {
    query::query(
        path,
        &QueryAsk {
            at: Some(instant),
            ..QueryAsk::default()
        },
    )
    .to_json()["query"]
        .clone()
}

// ---------------------------------------------------------------------------------------
// The matrix.
// ---------------------------------------------------------------------------------------

fn near(a: (f64, f64), b: (f64, f64)) {
    assert!(
        (a.0 - b.0).abs() < 1e-9 && (a.1 - b.1).abs() < 1e-9,
        "{a:?} != {b:?}"
    );
}

#[test]
fn the_matrix_is_translate_then_rotate_and_scale_about_the_pivot() {
    // 90° clockwise (y down) about (100, 100): the point 50 to the right lands 50 below.
    let turn = matrix_of(&json!({"pivot": [100, 100], "rotation": 90}), (0, 1));
    near(turn.apply((150.0, 100.0)), (100.0, 150.0));
    // The pivot is a fixed point of rotation and scale.
    let both = matrix_of(
        &json!({"pivot": [100, 100], "rotation": 33.0, "scale": [2.0, 3.0]}),
        (0, 1),
    );
    near(both.apply((100.0, 100.0)), (100.0, 100.0));
    // `x`, `y` is an offset in the parent's space, applied last.
    let moved = matrix_of(
        &json!({"pivot": [100, 100], "x": 7, "y": -3, "scale": [2.0, 2.0]}),
        (0, 1),
    );
    near(moved.apply((110.0, 100.0)), (127.0, 97.0));
    // A nest at rest is the identity, whatever its pivot.
    assert_eq!(
        matrix_of(&json!({"pivot": [321, 77]}), (0, 1)),
        Affine::IDENTITY
    );
}

// ---------------------------------------------------------------------------------------
// A nest at rest is a no-op.
// ---------------------------------------------------------------------------------------

#[test]
fn a_nest_at_rest_paints_the_bytes_the_same_project_paints_without_it() {
    let parts = || {
        vec![
            rect("bar", json!({"rotation": keyed(0.0, 40.0)})),
            json!({"id": "dot", "type": "ellipse", "start": 0, "end": 1000, "x": keyed_int(90, 300),
                   "y": 220, "origin": "center", "width": 40, "height": 40, "fill": "#33CCFF",
                   "scale": [{"t": 0, "v": [1.0, 1.0]}, {"t": 1000, "v": [1.5, 0.7], "ease": "linear"}]}),
        ]
    };
    let flat = written(
        line!(),
        &project(json!([track("parts", 1, parts())])),
    );
    // One identity nest, and a nest in a nest whose keys hold the identity.
    let nested = written(
        line!(),
        &project(json!([track(
            "rig",
            0,
            vec![nest(
                "outer",
                json!({}),
                json!([track(
                    "mid",
                    1,
                    vec![nest(
                        "inner",
                        json!({"pivot": [10, 10], "rotation": [{"t": 0, "v": 0.0},
                            {"t": 1000, "v": 0.0, "ease": "linear"}]}),
                        json!([track("parts", 1, parts())]),
                    )]
                )]),
            )]
        )])),
    );
    for instant in [0, 250, 500, 999] {
        let (a, _) = painted(&flat, instant);
        let (b, _) = painted(&nested, instant);
        assert!(a == b, "the nested project differs from the flat one at {instant} ms");
    }
}

// ---------------------------------------------------------------------------------------
// The conformance test the ruling names: every reader sees the composed position.
// ---------------------------------------------------------------------------------------

#[test]
fn every_reader_sees_a_childs_composed_position() {
    // A nest that moves and scales (no turn, so every reader can answer in rectangles).
    let doc = project(json!([track(
        "rig",
        0,
        vec![nest(
            "push",
            json!({"x": keyed_int(0, 60), "y": keyed_int(0, -20),
                   "scale": [{"t": 0, "v": [1.0, 1.0]}, {"t": 1000, "v": [1.5, 1.5], "ease": "linear"}]}),
            json!([track("parts", 1, vec![rect("bar", json!({}))])]),
        )]
    )]));
    let path = written(line!(), &doc);
    let instant = 500;
    // Half way: offset (30, -10), scale 1.25 about (200, 150). The bar's own box is
    // 120..180 x 105..135, centred (150, 120).
    let (_, picture) = painted(&path, instant);
    let (left, top, right, bottom) = ink(&picture).expect("something is drawn");

    // 1. The painter, in pixels: the composed box.
    let expect = |p: (f64, f64)| {
        let m = matrix_of(
            &json!({"pivot": [200, 150], "x": 30, "y": -10, "scale": [1.25, 1.25]}),
            (0, 1),
        );
        m.apply(p)
    };
    let (l, t) = expect((120.0, 105.0));
    let (r, b) = expect((180.0, 135.0));
    assert_eq!(
        (left, top, right, bottom),
        (l.floor() as u32, t.floor() as u32, r.ceil() as u32, b.ceil() as u32),
        "the painter's box"
    );

    // 2. `query --at`: the row names the chain and reports the composed placement point.
    let view = query_at(&path, instant);
    let bar = &view["stack"][0];
    assert_eq!(bar["nest"], json!(["push"]));
    let (cx, cy) = expect((150.0, 120.0));
    assert!((bar["composed"]["x"].as_f64().unwrap() - cx).abs() < 1e-6);
    assert!((bar["composed"]["y"].as_f64().unwrap() - cy).abs() < 1e-6);
    assert!((bar["composed"]["scale"][0].as_f64().unwrap() - 1.25).abs() < 1e-9);
    // The nest has a section of its own, with its resolved values.
    assert_eq!(view["nests"][0]["id"], "push");
    assert_eq!(view["nests"][0]["pivot"], json!([200, 150]));

    // 3. The geometry the checks read (`NOT COVERED` is the frame minus every drawn
    // rectangle): its complement is exactly the painter's box.
    assert_eq!(view["not_covered_unresolved"], Value::Null);
    let area: i64 = view["not_covered"]
        .as_array()
        .expect("rectangles")
        .iter()
        .map(|r| r["width"].as_i64().unwrap() * r["height"].as_i64().unwrap())
        .sum();
    assert_eq!(
        area,
        i64::from(W) * i64::from(H) - ((r - l).round() as i64) * ((b - t).round() as i64),
        "the drawn rectangle every check asks (whole pixels, as `drawn_rect` rounds)"
    );
}

#[test]
fn a_turning_nest_leaves_rectangles_unanswered_rather_than_wrong() {
    let doc = project(json!([track(
        "rig",
        0,
        vec![nest(
            "spin",
            json!({"rotation": keyed(0.0, 90.0)}),
            json!([track("parts", 1, vec![rect("bar", json!({}))])]),
        )]
    )]));
    let path = written(line!(), &doc);
    let view = query_at(&path, 500);
    assert!(
        view["not_covered_unresolved"]
            .as_str()
            .is_some_and(|why| why.contains("is 45°")),
        "{view}"
    );
    assert!((view["stack"][0]["composed"]["rotation"].as_f64().unwrap() - 45.0).abs() < 1e-9);
}

#[test]
fn a_projected_child_reports_corners_that_hold_what_the_painter_drew() {
    // ADR-0167's quad is one of the geometry functions every check reads; under a nest it
    // must be composed too, or `ink` and the layer bound would answer for the wrong place.
    let card = json!({"id": "card", "type": "rect", "start": 0, "end": 1000, "x": 150, "y": 120,
        "origin": "center", "width": 60, "height": 40, "fill": "#CC4466",
        "swivel": 30, "perspective": 200});
    let doc = project(json!([track(
        "rig",
        0,
        vec![nest(
            "push",
            json!({"x": keyed_int(0, 80), "rotation": keyed(0.0, 40.0),
                   "scale": [{"t": 0, "v": [1.0, 1.0]}, {"t": 1000, "v": [1.5, 1.5], "ease": "linear"}]}),
            json!([track("parts", 1, vec![card])]),
        )]
    )]));
    let path = written(line!(), &doc);
    let (_, picture) = painted(&path, 500);
    let (left, top, right, bottom) = ink(&picture).expect("drawn");
    let view = query_at(&path, 500);
    let corners = view["stack"][0]["projection"]["corners"]
        .as_array()
        .expect("corners");
    let xs: Vec<f64> = corners.iter().map(|c| c[0].as_f64().unwrap()).collect();
    let ys: Vec<f64> = corners.iter().map(|c| c[1].as_f64().unwrap()).collect();
    let (min_x, max_x) = (xs.iter().cloned().fold(f64::MAX, f64::min), xs.iter().cloned().fold(f64::MIN, f64::max));
    let (min_y, max_y) = (ys.iter().cloned().fold(f64::MAX, f64::min), ys.iter().cloned().fold(f64::MIN, f64::max));
    for (found, wanted, what) in [
        (f64::from(left), min_x, "left"),
        (f64::from(top), min_y, "top"),
        (f64::from(right), max_x, "right"),
        (f64::from(bottom), max_y, "bottom"),
    ] {
        assert!(
            (found - wanted).abs() <= 1.0,
            "{what}: painted {found}, corners say {wanted}: {corners:?}"
        );
    }
}

#[test]
fn nests_compose_outermost_first() {
    let doc = project(json!([track(
        "rig",
        0,
        vec![nest(
            "outer",
            json!({"pivot": [0, 0], "x": 100, "y": 0}),
            json!([track(
                "mid",
                1,
                vec![nest(
                    "inner",
                    json!({"pivot": [0, 0], "scale": [2.0, 2.0]}),
                    json!([track("parts", 2, vec![rect("bar", json!({"x": 10, "y": 10}))])]),
                )]
            )]),
        )]
    )]));
    let path = written(line!(), &doc);
    let view = query_at(&path, 0);
    // The point (10, 10) scales to (20, 20) in the outer's space, then moves to (120, 20).
    assert_eq!(view["stack"][0]["nest"], json!(["outer", "inner"]));
    assert!((view["stack"][0]["composed"]["x"].as_f64().unwrap() - 120.0).abs() < 1e-9);
    assert!((view["stack"][0]["composed"]["y"].as_f64().unwrap() - 20.0).abs() < 1e-9);
    assert_eq!(view["nests"].as_array().map(Vec::len), Some(2));
    assert_eq!(view["nests"][1]["nest"], json!(["outer"]));
}

// ---------------------------------------------------------------------------------------
// `validate`.
// ---------------------------------------------------------------------------------------

#[track_caller]
fn validated(doc: &Value) -> Report {
    let dir = tempdir(std::panic::Location::caller().line());
    let path = write_project(&dir, "p.montagent.json", &canonical(&doc.to_string()));
    validate(&path)
}

fn plain(extra: Value) -> Value {
    project(json!([track(
        "rig",
        0,
        vec![nest(
            "spin",
            extra,
            json!([track("parts", 1, vec![rect("bar", json!({}))])]),
        )]
    )]))
}

#[test]
fn a_well_formed_nest_validates_clean() {
    let report = validated(&plain(json!({"rotation": keyed(0.0, 30.0)})));
    assert!(codes(&report).is_empty(), "{:?}", codes(&report));
}

#[test]
fn a_child_outside_the_window_is_an_error_that_says_the_window_bounds_presence_only() {
    let mut doc = plain(json!({}));
    doc["tracks"][0]["elements"][0]["end"] = json!(800);
    let report = validated(&doc);
    assert_eq!(codes(&report), vec!["E-NEST-OUTSIDE-WINDOW"]);
    assert!(
        message_of(&report, 0).contains("bounds its presence only"),
        "{:?}",
        message_of(&report, 0)
    );
}

#[test]
fn an_empty_nest_is_named_once_at_its_outermost() {
    let doc = project(json!([track(
        "rig",
        0,
        vec![nest(
            "outer",
            json!({}),
            json!([track(
                "mid",
                1,
                vec![nest("inner", json!({}), json!([track("none", 2, vec![])]))]
            )]),
        )]
    )]));
    let report = validated(&doc);
    assert_eq!(codes(&report), vec!["E-NEST-EMPTY"]);
    assert_eq!(report.findings[0].location.element.as_deref(), Some("outer"));
}

#[test]
fn depth_over_four_is_an_error() {
    let mut inner = vec![rect("bar", json!({}))];
    for level in (1..=5).rev() {
        inner = vec![nest(
            &format!("n{level}"),
            json!({}),
            json!([track("t", level, inner)]),
        )];
    }
    let report = validated(&project(json!([track("rig", 0, inner)])));
    assert_eq!(codes(&report), vec!["E-NEST-TOO-DEEP"]);
    assert_eq!(report.findings[0].location.element.as_deref(), Some("n5"));
}

#[test]
fn the_schema_refuses_what_a_pure_matrix_does_not_carry() {
    for (key, value) in [
        ("layer", json!(3)),
        ("opacity", json!(0.5)),
        ("blend", json!("multiply")),
        ("effects", json!([])),
        ("clip", json!([0, 0, 10, 10])),
        ("motion_blur", json!({"shutter": 180, "samples": 4})),
    ] {
        let report = validated(&plain(json!({ key: value })));
        assert_eq!(codes(&report), vec!["E-SCHEMA"], "{key}");
        let message = message_of(&report, 0);
        assert!(
            message.contains("a nest") && (message.contains(key)),
            "{key}: {message}"
        );
    }
}

#[test]
fn a_pivot_is_required_and_a_keyword_is_refused() {
    let mut missing = plain(json!({}));
    missing["tracks"][0]["elements"][0]
        .as_object_mut()
        .expect("an object")
        .remove("pivot");
    assert_eq!(codes(&validated(&missing)), vec!["E-SCHEMA"]);
    assert_eq!(
        codes(&validated(&plain(json!({"pivot": "center"})))),
        vec!["E-SCHEMA"]
    );
}

#[test]
fn a_nest_is_not_an_anchor_target_or_a_transition_end() {
    let mut doc = plain(json!({}));
    doc["tracks"][0]["elements"][0]["tracks"][0]["elements"][0]["layer"] = json!({"below": "spin"});
    assert_eq!(codes(&validated(&doc)), vec!["E-NEST-TARGET"]);
}

#[test]
fn a_clip_does_not_travel_with_a_moving_nest() {
    let image = json!({"id": "pic", "type": "image", "source": "pic.png", "start": 0, "end": 1000,
                       "x": 150, "y": 120, "origin": "center", "width": 60, "height": 30,
                       "fit": "literal", "clip": [100, 90, 200, 100]});
    let dir = tempdir(line!());
    image::RgbaImage::from_pixel(60, 30, image::Rgba([255, 136, 0, 255]))
        .save(dir.join("pic.png"))
        .expect("the still");
    let nested = |extra: Value| {
        project(json!([track(
            "rig",
            0,
            vec![nest("spin", extra, json!([track("parts", 1, vec![image.clone()])]))]
        )]))
    };
    // Moving: the cut stays in frame space and the child does not.
    let path = write_project(&dir, "moving.montagent.json", &canonical(&nested(json!({"rotation": keyed(0.0, 30.0)})).to_string()));
    let report = validate(&path);
    assert!(codes(&report).contains(&"R-CLIP-IN-MOVING-NEST"), "{:?}", codes(&report));
    // At rest: a nest that never moves its child says nothing.
    let path = write_project(&dir, "still.montagent.json", &canonical(&nested(json!({})).to_string()));
    let report = validate(&path);
    assert!(!codes(&report).contains(&"R-CLIP-IN-MOVING-NEST"), "{:?}", codes(&report));
}

#[test]
fn a_still_nest_does_not_make_a_child_look_blurred_and_a_moving_one_does() {
    let blur = json!({"motion_blur": {"shutter": 180, "samples": 4}});
    let still = plain(json!({}));
    let mut still = still;
    still["tracks"][0]["elements"][0]["tracks"][0]["elements"][0]["motion_blur"] =
        blur["motion_blur"].clone();
    assert!(codes(&validated(&still)).contains(&"R-MOTION-BLUR-STILL"));

    let mut moving = plain(json!({"rotation": keyed(0.0, 90.0)}));
    moving["tracks"][0]["elements"][0]["tracks"][0]["elements"][0]["motion_blur"] =
        blur["motion_blur"].clone();
    assert!(
        !codes(&validated(&moving)).contains(&"R-MOTION-BLUR-STILL"),
        "a child of a moving nest is moving"
    );
}

#[test]
fn nested_tracks_obey_the_no_overlap_rule_and_so_does_the_nest_in_its_own_track() {
    // Two nests in one track overlapping by window.
    let doc = project(json!([track(
        "rig",
        0,
        vec![
            nest(
                "a",
                json!({}),
                json!([track("pa", 1, vec![rect("bar-a", json!({}))])])
            ),
            nest(
                "b",
                json!({"start": 500}),
                json!([track("pb", 2, vec![rect("bar-b", json!({"start": 500}))])])
            ),
        ]
    )]));
    let report = validated(&doc);
    assert!(
        codes(&report).iter().any(|c| c.contains("OVERLAP")),
        "{:?}",
        codes(&report)
    );
}

// ---------------------------------------------------------------------------------------
// `shift`.
// ---------------------------------------------------------------------------------------

#[test]
fn shift_treats_a_nests_keys_as_keyframes_and_its_children_as_elements() {
    let doc = plain(json!({"rotation": keyed(0.0, 90.0)}));
    let path = written(line!(), &doc);
    let shifted = shift::shift(
        &path,
        &shift::Ask {
            at: 500,
            delta: 200,
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
    let after: Value =
        serde_json::from_str(&std::fs::read_to_string(&path).expect("the file")).expect("JSON");
    let spin = &after["tracks"][0]["elements"][0];
    assert_eq!(spin["end"], 1200, "the nest straddled the cut and stretched");
    let bar = &spin["tracks"][0]["elements"][0];
    assert_eq!(bar["end"], 1200, "so did its child");
    // SPLIT wrote a hold at the cut: 45° at 500 and still 45° at 700.
    let times: Vec<i64> = spin["rotation"]
        .as_array()
        .expect("a list")
        .iter()
        .map(|record| record["t"].as_i64().expect("t"))
        .collect();
    assert_eq!(times, vec![0, 500, 700, 1200]);
}

// ---------------------------------------------------------------------------------------
// The phone mock-up's push-in (#597's hypothetical group), re-spelled as a nest.
// ---------------------------------------------------------------------------------------

#[test]
fn the_phone_push_in_as_a_nest_matches_the_scale_copied_onto_every_child() {
    // `x: 0, y: 0, pivot: [960, 540]`, against the baker's way: the group's `scale` keyframes
    // written onto each child. Not byte-identical in general — the nest composes in `f32`
    // and the flat spelling scales the child directly — but never more than one level on a
    // handful of pixels, as the exact-scale pair in #612 was held to.
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/nest/pip");
    let nested = dir.join("pip-nest.montagent.json");
    let flat = dir.join("pip-flat.montagent.json");
    let mut largest = 0u8;
    for instant in [0, 1250, 3000, 5900] {
        let (_, a) = painted(&nested, instant);
        let (_, b) = painted(&flat, instant);
        assert_eq!(a.dimensions(), b.dimensions());
        for (p, q) in a.pixels().zip(b.pixels()) {
            for channel in 0..4 {
                largest = largest.max(p.0[channel].abs_diff(q.0[channel]));
            }
        }
    }
    assert!(largest <= 1, "the push-in differs by {largest}/255");
}

// ---------------------------------------------------------------------------------------
// Byte-identical across painters — the gating test every composing feature is held to.
// ---------------------------------------------------------------------------------------

mod painters {
    use std::sync::mpsc;
    use std::time::Duration;

    use super::*;
    use montagent_core::verbs::render::{
        Ask, Forced, Progress, force_painting, render, tap_frames,
    };
    use montagent_render::canvas::bound_filter_layers;

    use crate::common::has_ffprobe;

    fn within<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(f());
        });
        rx.recv_timeout(Duration::from_secs(120))
            .expect("the render finished inside the timeout")
    }

    /// One hash per frame the encoder was handed, and the MP4's bytes.
    fn rendered(path: &Path, forced: Forced, bounded: bool) -> (Vec<u64>, Vec<u8>) {
        let path = path.to_path_buf();
        within(move || {
            bound_filter_layers(bounded);
            let _forced = force_painting(forced);
            let tap = tap_frames();
            let answer = render(&path, &Ask::default(), &mut |_: Progress| {});
            assert_eq!(
                answer.report().exit_code(),
                ExitCode::Ok,
                "{}",
                answer.to_json()
            );
            let mp4 = std::fs::read(&answer.video().expect("a file").path).expect("the file");
            (tap.hashes(), mp4)
        })
    }

    /// A camera: one nest around the whole scene, panning, zooming and turning a little,
    /// with `motion_blur` on its children, `effects` that reach outside their boxes, and a
    /// rig inside it (two levels) whose arm is blurred as well.
    pub fn camera_project(line: u32) -> std::path::PathBuf {
        let arm = json!({"id": "arm", "type": "rect", "start": 0, "end": 600, "x": 60, "y": 30,
            "origin": "center-left", "width": 36, "height": 8, "fill": "#E0A030",
            "rotation": [{"t": 0, "v": -30.0}, {"t": 600, "v": 60.0, "ease": "ease-in-out"}],
            "motion_blur": {"shutter": 180, "samples": 4}});
        let body = json!({"id": "body", "type": "rect", "start": 0, "end": 600, "x": 60, "y": 45,
            "origin": "center", "width": 30, "height": 40, "fill": "#33CCFF",
            "effects": [{"name": "blur", "radius": 2},
                        {"name": "shadow", "dx": 4, "dy": 4, "radius": 5, "color": "#000000", "opacity": 0.6}]});
        let ground = json!({"id": "ground", "type": "rect", "start": 0, "end": 600, "x": 80, "y": 80,
            "origin": "center", "width": 400, "height": 10, "fill": "#446644",
            "motion_blur": {"shutter": 360, "samples": 6}});
        // ADR-0167's projection, on a child: it composes with the nest's matrix.
        let card = json!({"id": "card", "type": "rect", "start": 0, "end": 600, "x": 110, "y": 25,
            "origin": "center", "width": 30, "height": 20, "fill": "#CC4466",
            "swivel": keyed(-40.0, 40.0), "perspective": 90,
            "effects": [{"name": "blur", "radius": 1.5}]});
        let rig = nest(
            "rig",
            json!({"end": 600, "pivot": [60, 45], "x": keyed_int(0, 14), "rotation": keyed(0.0, 8.0),
                   "scale": [{"t": 0, "v": [1.0, 1.0]}, {"t": 600, "v": [1.2, 0.9], "ease": "ease-in"}]}),
            json!([track("rig-body", 1, vec![body]), track("rig-arm", 3, vec![arm]), track("rig-card", 4, vec![card])]),
        );
        let camera = nest(
            "camera",
            json!({"end": 600, "pivot": [80, 45], "x": keyed_int(0, -40), "y": keyed_int(0, 6),
                   "rotation": keyed(0.0, -5.0),
                   "scale": [{"t": 0, "v": [1.0, 1.0]}, {"t": 600, "v": [1.3, 1.3], "ease": "ease-out"}]}),
            json!([track("ground", 0, vec![ground]), track("scene", 2, vec![rig])]),
        );
        let doc = json!({"frame": {"width": 160, "height": 90}, "fps": 30, "background": "#101418",
                         "duration": 600, "output": "out/camera.mp4",
                         "tracks": [track("cam", 0, vec![camera])]});
        let dir = tempdir(line);
        write_project(&dir, "camera.montagent.json", &canonical(&doc.to_string()))
    }

    #[test]
    fn a_nested_scene_paints_the_same_frames_on_any_number_of_painters_with_the_hint_on_or_off() {
        if !has_ffprobe() {
            return;
        }
        let path = camera_project(line!());
        let (one, mp4) = rendered(&path, Forced::OnePainter, true);
        assert_eq!(one.len(), 18);
        assert_ne!(one[2], one[9], "the scene moves");
        for (painters, chunk) in [(3, 2), (2, 5), (4, 1), (10, 1)] {
            let (frames, other) = rendered(
                &path,
                Forced::Chunks {
                    painters,
                    chunk,
                    window_bytes: None,
                },
                true,
            );
            assert_eq!(frames, one, "K={painters}, C={chunk}");
            assert!(other == mp4, "K={painters}, C={chunk}: the MP4s differ");
        }
        // The bounded filter layers must equal the unbounded ones under a nest's matrix.
        let (unbounded, _) = rendered(&path, Forced::OnePainter, false);
        assert_eq!(unbounded, one, "the layer bound is a hint a nest must not break");
    }
}
