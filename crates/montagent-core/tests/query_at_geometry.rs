//! `query --at`'s four components that reach outside the document (#210): the offset
//! into the source, the crop rectangle, the ink box, and `NOT COVERED`.
//!
//! Real media and a real `ffprobe` for the crop rectangle, on `tests/fit.rs`'s own
//! reasoning: a recorded probe would let it pass against a source that never existed.

use std::path::{Path, PathBuf};

use montagent_core::report::ExitCode;
use montagent_core::verbs::query::{Ask, query};
use serde_json::Value;

mod common;
use common::{canonical, has_ffprobe, write_project};

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/en-halloween-decorating")
        .canonicalize()
        .expect("the committed fixture")
}

fn fixture_project() -> PathBuf {
    fixture_dir().join("en-halloween-decorating.montagent.json")
}

fn font_file() -> PathBuf {
    fixture_dir()
        .join("fonts/OpenRunde-Bold.otf")
        .canonicalize()
        .expect("the vendored font (#143)")
}

fn ask_at(instant: i64) -> Ask {
    Ask {
        at: Some(instant),
        ..Ask::default()
    }
}

#[track_caller]
fn at(project: &Path, instant: i64) -> Value {
    let answer = query(project, &ask_at(instant));
    assert_eq!(
        answer.report().exit_code(),
        ExitCode::Ok,
        "query --at {instant} did not answer: {}",
        serde_json::to_string_pretty(&answer.to_json()).unwrap_or_default()
    );
    answer.to_json()["query"].clone()
}

fn element<'a>(view: &'a Value, id: &str) -> &'a Value {
    view["stack"]
        .as_array()
        .expect("a stack array")
        .iter()
        .find(|e| e["id"] == id)
        .unwrap_or_else(|| panic!("`{id}` is not in the presence set"))
}

// ---------------------------------------------------------------------------
// All four, on the committed fixture.
// ---------------------------------------------------------------------------

#[test]
fn all_four_components_answer_on_the_committed_fixtures_own_elements() {
    if !has_ffprobe() {
        return;
    }
    // `item-06`: an audio element, a `cover` image with a `clip`, and its caption text —
    // one instant that exercises every one of #210's four components at once.
    let view = at(&fixture_project(), 17472);

    let audio = element(&view, "vo-word-06-a");
    assert_eq!(audio["source_offset"], 0);
    assert_eq!(audio["source_offset_unresolved"], Value::Null);

    let photo = element(&view, "photo-06");
    assert_eq!(photo["crop_unresolved"], Value::Null);
    assert!(
        photo["crop"].is_object(),
        "photo-06 should carry a crop rectangle"
    );

    let caption = element(&view, "word-06");
    assert_eq!(caption["ink_box_unresolved"], Value::Null);
    assert!(
        caption["ink_box"].is_object(),
        "word-06 should carry an ink box"
    );

    assert_eq!(view["not_covered_unresolved"], Value::Null);
    assert!(
        view["not_covered"]
            .as_array()
            .is_some_and(|v| !v.is_empty()),
        "the caption card does not fill the whole 1080x1920 frame"
    );
}

// ---------------------------------------------------------------------------
// The crop rectangle — ADR-0013/ADR-0011's own worked example.
// ---------------------------------------------------------------------------

#[test]
fn the_crop_rectangle_matches_adr_0013s_worked_example() {
    if !has_ffprobe() {
        return;
    }
    // `images/06.png` is 1536x2720; `clip` is 1080x1300; cover floors the declared rect to
    // 1080x1912 (ADR-0013's own arithmetic, reproduced by `photo-06` in the fixture). At
    // `x:0,y:0,origin:"top-left"`, `clip` and the drawn rect share their top-left corner,
    // so the crop is `[0, 0, source_width, round(1300 * source_height / 1912)]`.
    let view = at(&fixture_project(), 17472);
    let photo = element(&view, "photo-06");
    assert_eq!(
        photo["crop"],
        serde_json::json!({"x": 0, "y": 0, "width": 1536, "height": 1850})
    );
}

#[test]
fn a_fit_literal_element_still_derives_a_crop_from_its_declared_rect() {
    if !has_ffprobe() {
        return;
    }
    // ADR-0015's load-bearing sentence: the declared rect is authoritative *regardless*
    // of which rule `fit` claims — so `literal` answers identically to `cover` here.
    let source = fixture_dir().join("images/06.png");
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(
        &dir,
        "p.montagent.json",
        &canonical(&format!(
            r##"{{"frame":{{"width":1080,"height":1920}},"fps":25,"tracks":[{{"name":"photo","layer":1,"elements":[{{"id":"photo-01","type":"image","start":0,"end":1000,"source":{:?},"x":0,"y":0,"origin":"top-left","width":1080,"height":1912,"fit":"literal","clip":[0,0,1080,1300]}}]}}]}}"##,
            source.display().to_string()
        )),
    );

    let view = at(&path, 0);
    let photo = element(&view, "photo-01");
    assert_eq!(
        photo["crop"],
        serde_json::json!({"x": 0, "y": 0, "width": 1536, "height": 1850})
    );
}

#[test]
fn an_element_with_no_clip_names_that_as_the_reason() {
    let source = fixture_dir().join("images/06.png");
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(
        &dir,
        "p.montagent.json",
        &canonical(&format!(
            r##"{{"frame":{{"width":1080,"height":1920}},"fps":25,"tracks":[{{"name":"photo","layer":1,"elements":[{{"id":"photo-01","type":"image","start":0,"end":1000,"source":{:?},"x":0,"y":0,"origin":"top-left","width":1080,"height":1912,"fit":"cover"}}]}}]}}"##,
            source.display().to_string()
        )),
    );

    // No `clip` at all: no probe is needed to answer this refusal, so it holds even
    // without `ffprobe` on `PATH`.
    let view = at(&path, 0);
    let photo = element(&view, "photo-01");
    assert_eq!(photo["crop"], Value::Null);
    assert!(
        photo["crop_unresolved"]
            .as_str()
            .is_some_and(|s| s.contains("clip")),
        "{:?}",
        photo["crop_unresolved"]
    );
}

// ---------------------------------------------------------------------------
// Offset into source — ADR-0020, under `hold` and `loop`.
// ---------------------------------------------------------------------------

fn video_project(overrun: &str, speed: Option<f64>, timeline_span: i64) -> String {
    let speed_field = match speed {
        Some(s) => format!(r#","speed":{s}"#),
        None => String::new(),
    };
    canonical(&format!(
        r##"{{"frame":{{"width":1080,"height":1920}},"fps":25,"tracks":[{{"name":"v","layer":1,"elements":[{{"id":"v","type":"video","start":0,"end":{timeline_span},"source":"video/does-not-exist.mp4","source_start":0,"source_end":2000,"x":0,"y":0,"width":1080,"height":1920,"fit":"literal","overrun":{overrun:?}{speed_field}}}]}}]}}"##
    ))
}

#[test]
fn offset_into_source_holds_the_last_frame_past_the_as_played_duration() {
    if !has_ffprobe() {
        return;
    }
    let dir = common::tempdir(std::panic::Location::caller().line());
    // `source_end - source_start` is 2000ms and `speed` is 1, so `played` is 2000ms; the
    // 3000ms timeline span overruns it by 1000ms.
    let path = write_project(&dir, "p.montagent.json", &video_project("hold", None, 3000));

    // Inside the as-played span: a straight 1:1 offset.
    let view = at(&path, 500);
    assert_eq!(element(&view, "v")["source_offset"], 500);

    // Past it: held at `source_end`, not extrapolated further.
    let view = at(&path, 2500);
    assert_eq!(element(&view, "v")["source_offset"], 2000);
    let view = at(&path, 2999);
    assert_eq!(element(&view, "v")["source_offset"], 2000);
}

#[test]
fn offset_into_source_loops_with_a_hard_cut_every_as_played_duration() {
    if !has_ffprobe() {
        return;
    }
    let dir = common::tempdir(std::panic::Location::caller().line());
    // `played` is 2000ms (speed 1); a 5000ms span loops it twice and truncates the third.
    let path = write_project(&dir, "p.montagent.json", &video_project("loop", None, 5000));

    // First iteration: unchanged from the plain case.
    assert_eq!(element(&at(&path, 500), "v")["source_offset"], 500);
    // Second iteration starts the instant the first ends, with a hard cut back to zero.
    assert_eq!(element(&at(&path, 2000), "v")["source_offset"], 0);
    assert_eq!(element(&at(&path, 2500), "v")["source_offset"], 500);
    // Third iteration, truncated by the timeline span rather than by the loop period.
    assert_eq!(element(&at(&path, 4000), "v")["source_offset"], 0);
    assert_eq!(element(&at(&path, 4999), "v")["source_offset"], 999);
}

#[test]
fn offset_into_source_applies_speed_before_overrun_using_the_fixtures_own_element() {
    if !has_ffprobe() {
        return;
    }
    // `vo-sentence-05-b` (ADR-0020's own worked example): `source_start:0`,
    // `source_end:2184`, `speed:0.645`, `start:13172`. 1000ms of timeline elapsed is
    // `round(1000 * 0.645)` = 645ms into the source — no overrun involved, since
    // `end-start` (3386) is exactly the as-played duration.
    let view = at(&fixture_project(), 14172);
    let vo = element(&view, "vo-sentence-05-b");
    assert_eq!(vo["source_offset"], 645);
}

// ---------------------------------------------------------------------------
// `NOT COVERED`.
// ---------------------------------------------------------------------------

fn shapes_project(shapes: &str) -> String {
    canonical(&format!(
        r##"{{"frame":{{"width":100,"height":100}},"fps":25,"tracks":[{{"name":"t","layer":1,"elements":[{shapes}]}}]}}"##
    ))
}

#[test]
fn not_covered_is_the_frame_when_nothing_is_present() {
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(&dir, "p.montagent.json", &shapes_project(""));
    let view = at(&path, 0);
    assert_eq!(
        view["not_covered"],
        serde_json::json!([{"x": 0, "y": 0, "width": 100, "height": 100}])
    );
}

#[test]
fn not_covered_is_empty_when_one_rect_fills_the_frame() {
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(
        &dir,
        "p.montagent.json",
        &shapes_project(
            r##"{"id":"bg","type":"rect","start":0,"end":1000,"x":0,"y":0,"origin":"top-left","width":100,"height":100,"fill":"#000000"}"##,
        ),
    );
    let view = at(&path, 0);
    assert_eq!(view["not_covered"], serde_json::json!([]));
}

#[test]
fn not_covered_reports_the_strip_a_smaller_rect_leaves_uncovered() {
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(
        &dir,
        "p.montagent.json",
        &shapes_project(
            r##"{"id":"panel","type":"rect","start":0,"end":1000,"x":0,"y":0,"origin":"top-left","width":100,"height":40,"fill":"#000000"}"##,
        ),
    );
    let view = at(&path, 0);
    assert_eq!(
        view["not_covered"],
        serde_json::json!([{"x": 0, "y": 40, "width": 100, "height": 60}])
    );
}

#[test]
fn an_opaque_zero_element_never_covers_anything() {
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(
        &dir,
        "p.montagent.json",
        &shapes_project(
            r##"{"id":"invisible","type":"rect","start":0,"end":1000,"x":0,"y":0,"origin":"top-left","width":100,"height":100,"fill":"#000000","opacity":0}"##,
        ),
    );
    let view = at(&path, 0);
    assert_eq!(
        view["not_covered"],
        serde_json::json!([{"x": 0, "y": 0, "width": 100, "height": 100}])
    );
}

#[test]
fn a_rotated_element_refuses_not_covered_and_crop_rather_than_approximate() {
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(
        &dir,
        "p.montagent.json",
        &shapes_project(
            r##"{"id":"spinner","type":"rect","start":0,"end":1000,"x":50,"y":50,"width":20,"height":20,"fill":"#000000","rotation":45}"##,
        ),
    );
    let view = at(&path, 0);
    assert_eq!(view["not_covered"], serde_json::json!([]));
    assert!(
        view["not_covered_unresolved"]
            .as_str()
            .is_some_and(|s| s.contains("rotation")),
        "{:?}",
        view["not_covered_unresolved"]
    );
}

#[test]
fn an_invisible_rotated_element_does_not_force_not_covered_to_refuse() {
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(
        &dir,
        "p.montagent.json",
        &shapes_project(
            r##"{"id":"spinner","type":"rect","start":0,"end":1000,"x":50,"y":50,"width":20,"height":20,"fill":"#000000","rotation":45,"opacity":0}"##,
        ),
    );
    let view = at(&path, 0);
    assert_eq!(view["not_covered_unresolved"], Value::Null);
    assert_eq!(
        view["not_covered"],
        serde_json::json!([{"x": 0, "y": 0, "width": 100, "height": 100}])
    );
}

// ---------------------------------------------------------------------------
// The ink box — built on the real text engine, not a nominal-size approximation.
// ---------------------------------------------------------------------------

fn text_project(align: &str, line: u32) -> PathBuf {
    let dir = common::tempdir(line);
    write_project(
        &dir,
        "p.montagent.json",
        &canonical(&format!(
            r##"{{"frame":{{"width":1080,"height":1920}},"fps":25,"fonts":{{"brand":[{{"file":{:?}}}]}},"tracks":[{{"name":"t","layer":1,"elements":[{{"id":"cap","type":"text","start":0,"end":1000,"x":540,"y":100,"origin":"top-center","width":600,"height":100,"font":"brand","size":60,"align":{align:?},"runs":[{{"text":"cobweb"}}]}}]}}]}}"##,
            font_file().display().to_string()
        )),
    )
}

#[test]
fn the_ink_box_is_tighter_than_the_nominal_size_times_line_height_box() {
    let view = at(&text_project("center", line!()), 0);
    let cap = element(&view, "cap");
    let ink_box = &cap["ink_box"];
    assert_eq!(cap["ink_box_unresolved"], Value::Null);
    // The declared box is 100px tall for one line at size 60 — real ink is tighter than
    // the box an author leaves room in, on the same reading ADR-0011 measured (1.25×–1.48×
    // over nominal `size × line_height`, for the fonts it sampled).
    assert!(
        ink_box["height"].as_f64().unwrap() < 100.0,
        "ink box height {:?} should be tighter than the declared 100px box",
        ink_box["height"]
    );
}

#[test]
fn align_moves_the_ink_box_horizontally_inside_the_declared_width() {
    let start = at(&text_project("start", line!()), 0);
    let center = at(&text_project("center", line!()), 0);
    let end = at(&text_project("end", line!()), 0);

    let x = |view: &Value| element(view, "cap")["ink_box"]["x"].as_f64().unwrap();
    // `origin:"top-center"` at `x:540` puts the declared box's own left edge at
    // `540 - 600/2 = 240`. `start` sits flush there; `center` and `end` sit strictly to
    // its right, in that order, because the single short run is narrower than the box.
    assert_eq!(x(&start), 240.0);
    assert!(x(&center) > x(&start));
    assert!(x(&end) > x(&center));
}

#[test]
fn a_scaled_text_elements_ink_box_refuses_rather_than_approximate() {
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(
        &dir,
        "p.montagent.json",
        &canonical(&format!(
            r##"{{"frame":{{"width":1080,"height":1920}},"fps":25,"fonts":{{"brand":[{{"file":{:?}}}]}},"tracks":[{{"name":"t","layer":1,"elements":[{{"id":"cap","type":"text","start":0,"end":1000,"x":540,"y":100,"width":600,"height":100,"font":"brand","size":60,"scale":[1.5,1.5],"runs":[{{"text":"cobweb"}}]}}]}}]}}"##,
            font_file().display().to_string()
        )),
    );
    let view = at(&path, 0);
    let cap = element(&view, "cap");
    assert_eq!(cap["ink_box"], Value::Null);
    assert!(
        cap["ink_box_unresolved"]
            .as_str()
            .is_some_and(|s| s.contains("scale")),
        "{:?}",
        cap["ink_box_unresolved"]
    );
}
