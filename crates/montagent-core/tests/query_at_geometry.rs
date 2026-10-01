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
    runs_project(align, r#"[{"text":"cobweb"}]"#, line)
}

/// `text_project`'s element, with `runs` spelled by the caller.
fn runs_project(align: &str, runs: &str, line: u32) -> PathBuf {
    placed_project(align, "top-center", 540, runs, line)
}

/// `runs_project`'s element, with its `origin` and `x` spelled by the caller too. The
/// declared box stays 600 wide whatever the text, so a reading that consulted it would
/// show up as a gap between the ink box and the ink.
fn placed_project(align: &str, origin: &str, x: i64, runs: &str, line: u32) -> PathBuf {
    let dir = common::tempdir(line);
    write_project(
        &dir,
        "p.montagent.json",
        &canonical(&format!(
            r##"{{"frame":{{"width":1080,"height":1920}},"fps":25,"fonts":{{"brand":[{{"file":{:?}}}]}},"tracks":[{{"name":"t","layer":1,"elements":[{{"id":"cap","type":"text","start":0,"end":1000,"x":{x},"y":400,"origin":{origin:?},"width":600,"height":100,"font":"brand","size":60,"color":"#FF0000","align":{align:?},"runs":{runs}}}]}}]}}"##,
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
fn a_single_line_is_placed_by_origin_over_its_own_width_whatever_the_align() {
    // ADR-0134: lines align inside the block they make, and `origin` places that block —
    // the declared `width` is a container claim and is never consulted. One line *is* its
    // block, so `align` has nowhere to move it.
    let start = at(&text_project("start", line!()), 0);
    let center = at(&text_project("center", line!()), 0);
    let end = at(&text_project("end", line!()), 0);

    let ink_box = |view: &Value| element(view, "cap")["ink_box"].clone();
    assert_eq!(ink_box(&center), ink_box(&start));
    assert_eq!(ink_box(&end), ink_box(&start));
    // `origin:"top-center"` at `x:540` centres the block on 540 — not the declared box's
    // left edge at `540 - 600/2 = 240`, which is where #551 found it.
    let got = ink_box(&start);
    let middle = got["x"].as_f64().unwrap() + got["width"].as_f64().unwrap() / 2.0;
    assert!((middle - 540.0).abs() < 1e-6, "{got}");
}

#[test]
fn align_moves_a_shorter_line_inside_the_block_and_not_the_block() {
    // The block is the widest line's advance, so `align` moves the short line within it and
    // the block's extent — the ink box's horizontal extent — stays where `origin` put it.
    let runs = r#"[{"text":"cobweb cobweb\nab"}]"#;
    let ink_box = |align: &str| {
        element(&at(&runs_project(align, runs, line!()), 0), "cap")["ink_box"].clone()
    };
    let start = ink_box("start");
    for align in ["center", "end"] {
        let got = ink_box(align);
        assert_eq!(got["x"], start["x"], "{align}");
        assert_eq!(got["width"], start["width"], "{align}");
    }
}

#[test]
fn a_run_set_to_rtl_answers_an_ink_box_and_the_override_moves_nothing_outside_it() {
    // #457: this refused while no ADR said how `start`/`end` resolve under bidi. ADR-0133
    // does — against the line's base direction, which a run's isolate never changes — so
    // a Latin line with an RTL run is still a left-to-right line, flush with the left edge.
    let plain = at(&text_project("start", line!()), 0);
    let isolated = at(
        &runs_project(
            "start",
            r#"[{"text":"cob"},{"text":"web","dir":"rtl"}]"#,
            line!(),
        ),
        0,
    );
    let cap = element(&isolated, "cap");
    assert_eq!(cap["ink_box_unresolved"], Value::Null);
    let (got, want) = (&cap["ink_box"], &element(&plain, "cap")["ink_box"]);
    // On the same baseline, and centred on the same `x`. The width is *not* asserted equal:
    // shaping does not cross an isolate's edge, so the `b`–`w` kern is lost, and that is
    // the isolate working.
    let middle = |b: &Value| b["x"].as_f64().unwrap() + b["width"].as_f64().unwrap() / 2.0;
    assert!((middle(got) - 540.0).abs() < 1e-6, "{got}");
    assert!((middle(want) - 540.0).abs() < 1e-6, "{want}");
    assert_eq!(got["y"], want["y"]);
    assert_eq!(got["height"], want["height"]);
}

#[test]
fn start_on_a_right_to_left_line_is_the_blocks_right_edge() {
    // An all-Hebrew line is right-to-left, so `start` is the right edge (ADR-0007's
    // reason for the vocabulary) — with or without a `dir` on the run, because the
    // override is an isolate and never the line's direction (ADR-0133). The face has no
    // Hebrew; the replacement glyphs still advance, and bidi reads the characters.
    //
    // The edge is the block's (ADR-0134): here a wider, unstroked Latin line makes the
    // block, centred on 540. The ink box is the union of the lines, so the Hebrew line is
    // seen through its stroke, which reaches 10 px past whichever block edge it sits on.
    let ink_box = |align: &str, dir: &str| {
        let runs = format!(
            r##"[{{"text":"cobweb cobweb\n"}},{{"text":"שלום"{dir},"stroke":"#00FF00","stroke_width":10}}]"##
        );
        element(&at(&runs_project(align, &runs, line!()), 0), "cap")["ink_box"].clone()
    };
    let unstroked = element(
        &at(
            &runs_project("start", r#"[{"text":"cobweb cobweb"}]"#, line!()),
            0,
        ),
        "cap",
    )["ink_box"]
        .clone();
    let block_left = unstroked["x"].as_f64().unwrap();
    let block_width = unstroked["width"].as_f64().unwrap();
    assert!(
        (block_left + block_width / 2.0 - 540.0).abs() < 1e-6,
        "{unstroked}"
    );

    let near = |got: &Value, key: &str, want: f64| (got[key].as_f64().unwrap() - want).abs() < 1e-6;
    for dir in ["", r#","dir":"ltr""#] {
        let start = ink_box("start", dir);
        assert!(near(&start, "x", block_left), "start{dir}: {start}");
        assert!(
            near(&start, "width", block_width + 10.0),
            "start{dir}: {start}"
        );
        let end = ink_box("end", dir);
        assert!(near(&end, "x", block_left - 10.0), "end{dir}: {end}");
        assert!(near(&end, "width", block_width + 10.0), "end{dir}: {end}");
    }
}

/// The leftmost and rightmost columns `frame` painted anything in, at true scale. "Painted"
/// is "differs from the top-left pixel", which no test element reaches, so the reading does
/// not depend on what colour an empty frame is.
fn painted_columns(project: &Path) -> (u32, u32) {
    use montagent_core::verbs::frame::{Ask, frame};
    let answer = frame(
        project,
        &Ask {
            at: Some(0),
            full: true,
            png: true,
            ..Ask::default()
        },
    );
    let picture = image::load_from_memory(&answer.image().expect("a picture").bytes)
        .expect("a PNG")
        .to_rgba8();
    let empty = *picture.get_pixel(0, 0);
    let columns: Vec<u32> = picture
        .enumerate_pixels()
        .filter(|(_, _, pixel)| **pixel != empty)
        .map(|(x, _, _)| x)
        .collect();
    (
        *columns.iter().min().expect("some ink"),
        *columns.iter().max().expect("some ink"),
    )
}

#[test]
fn the_ink_boxs_horizontal_extent_is_the_ink_frame_paints() {
    // #551: the ink box was 185 px from the ink, because it aligned inside the declared
    // `width` and the painter aligns inside the block. Held against the picture itself so
    // the two cannot drift apart again — a single line and a block of mixed line widths,
    // under every `align`, at origins that pivot on the left, the centre and the right.
    //
    // The two stroked blocks put a narrow line's 10 px stroke past whichever block edge
    // `align` sends it to, so the picture itself tells `start` from `end` — on a
    // left-to-right line and on a right-to-left one.
    //
    // The box is advance-based and the picture is ink, so the ink sits inside the box by
    // the edge glyphs' side bearings: 2 px at most for the Latin glyphs, and up to 6 px for
    // the replacement glyph the face draws for Hebrew. Antialiasing can put a faint pixel
    // one column past the box.
    let blocks = [
        (r#"[{"text":"cobweb"}]"#, 3.0),
        (r#"[{"text":"cobweb cobweb\nab\ncobweb"}]"#, 3.0),
        (
            r##"[{"text":"cobweb cobweb\n"},{"text":"ab","stroke":"#00FF00","stroke_width":10}]"##,
            3.0,
        ),
        (
            r##"[{"text":"cobweb cobweb\n"},{"text":"שלום","stroke":"#00FF00","stroke_width":10}]"##,
            7.0,
        ),
    ];
    for (runs, slack) in blocks {
        for (origin, x) in [
            ("top-center", 540),
            ("top-left", 100),
            ("bottom-right", 1000),
        ] {
            for align in ["start", "center", "end"] {
                let project = placed_project(align, origin, x, runs, line!());
                let ink_box = element(&at(&project, 0), "cap")["ink_box"].clone();
                let left = ink_box["x"].as_f64().unwrap();
                let right = left + ink_box["width"].as_f64().unwrap();
                let (ink_left, ink_right) = painted_columns(&project);
                // The ink's last column is `ink_right`; its right edge is one past it.
                let (ink_left, ink_right) = (f64::from(ink_left), f64::from(ink_right) + 1.0);
                let case = format!(
                    "{runs} {origin} x={x} {align}: box {left}→{right}, ink {ink_left}→{ink_right}"
                );
                assert!(
                    ink_left >= left - 1.0 && ink_right <= right + 1.0,
                    "ink outside the box: {case}"
                );
                assert!(
                    ink_left - left <= slack && right - ink_right <= slack,
                    "box wider than the ink: {case}"
                );
            }
        }
    }
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
