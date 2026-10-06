//! `wipe`, `slide` and `push` beside `crossfade`, with `direction` and `ease` (#690,
//! ADR-0150).
//!
//! The seams are the verbs: `validate`'s report, `frame`'s picture and answer, `render`'s
//! frames at the encoder's input, `query --at`'s answer, and the files `shift` writes.

use montagent_core::report::Report;
use montagent_core::validate;
use serde_json::{Value, json};

mod common;
use common::{canonical, write_project};

/// A 400×200 project at 25 fps: `a` (red) on layer 0 over 0..2000, `b` (blue) on layer 1
/// over 1000..3000, and whatever transition a test bridges them with on layer 2.
fn bridged(transition: Value) -> String {
    bridged_with(
        json!({"id": "a", "type": "rect", "start": 0, "end": 2000, "x": 0, "y": 0,
               "origin": "top-left", "width": 400, "height": 200, "fill": "#FF0000"}),
        json!({"id": "b", "type": "rect", "start": 1000, "end": 3000, "x": 0, "y": 0,
               "origin": "top-left", "width": 400, "height": 200, "fill": "#0000FF"}),
        transition,
    )
}

fn bridged_with(a: Value, b: Value, transition: Value) -> String {
    canonical(
        &json!({
            "frame": {"width": 400, "height": 200}, "fps": 25, "background": "#00FF00",
            "duration": 3000, "output": "out/t.mp4",
            "tracks": [
                {"name": "first", "layer": 0, "elements": [a]},
                {"name": "second", "layer": 1, "elements": [b]},
                {"name": "bridge", "layer": 2, "elements": [transition]},
            ],
        })
        .to_string(),
    )
}

/// A transition element over the window `a` and `b` share, plus `extra` fields.
fn transition(kind: &str, extra: Value) -> Value {
    let mut element = json!({"id": "t", "type": "transition", "start": 1000, "end": 2000,
                             "kind": kind, "from": "a", "to": "b"});
    for (key, value) in extra.as_object().expect("an object").clone() {
        element[key] = value;
    }
    element
}

#[track_caller]
fn validated(body: &str) -> Report {
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(&dir, "p.montagent.json", body);
    validate(&path)
}

fn codes(report: &Report) -> Vec<&str> {
    report.findings.iter().map(|f| f.code.as_str()).collect()
}

fn schema_faults(report: &Report) -> Vec<String> {
    report
        .findings
        .iter()
        .filter(|f| f.code.starts_with("E-SCHEMA"))
        .map(|f| serde_json::to_string(&f.fields).unwrap_or_default())
        .collect()
}

const KINDS: [&str; 3] = ["wipe", "slide", "push"];
const DIRECTIONS: [&str; 4] = ["left", "right", "up", "down"];

// ---------------------------------------------------------------------------
// The schema
// ---------------------------------------------------------------------------

#[test]
fn each_geometric_kind_and_direction_validates_clean() {
    for kind in KINDS {
        for direction in DIRECTIONS {
            let report = validated(&bridged(transition(kind, json!({"direction": direction}))));
            assert!(
                report
                    .findings
                    .iter()
                    .all(|f| f.class != montagent_core::finding::Class::Error),
                "{kind} {direction}: {:?}",
                codes(&report)
            );
        }
    }
}

#[test]
fn an_ease_is_a_keyframe_ease_name_or_bezier() {
    for ease in [
        json!("ease-in-out"),
        json!("step"),
        json!([0.2, 0.0, 0.2, 1.4]),
    ] {
        let report = validated(&bridged(transition(
            "push",
            json!({"direction": "left", "ease": ease}),
        )));
        assert!(
            schema_faults(&report).is_empty(),
            "{ease}: {:?}",
            codes(&report)
        );
    }
}

#[test]
fn direction_or_ease_on_a_crossfade_is_a_schema_error() {
    for extra in [json!({"direction": "left"}), json!({"ease": "ease-in"})] {
        let report = validated(&bridged(transition("crossfade", extra.clone())));
        assert_eq!(
            schema_faults(&report).len(),
            1,
            "{extra}: {:?}",
            codes(&report)
        );
    }
    // And the plain crossfade is still clean.
    let report = validated(&bridged(transition("crossfade", json!({}))));
    assert!(schema_faults(&report).is_empty(), "{:?}", codes(&report));
}

#[test]
fn a_missing_direction_on_wipe_slide_or_push_is_a_schema_error() {
    for kind in KINDS {
        let report = validated(&bridged(transition(kind, json!({}))));
        let faults = schema_faults(&report);
        assert_eq!(faults.len(), 1, "{kind}: {:?}", codes(&report));
        assert!(faults[0].contains("direction"), "{kind}: {faults:?}");
    }
}

#[test]
fn an_unknown_kind_direction_or_ease_is_a_schema_error() {
    for element in [
        transition("iris", json!({"direction": "left"})),
        transition("push", json!({"direction": "west"})),
        transition("push", json!({"direction": "left", "ease": "spring"})),
        transition(
            "push",
            json!({"direction": "left", "ease": [1.5, 0.0, 0.2, 1.0]}),
        ),
    ] {
        let report = validated(&bridged(element.clone()));
        assert_eq!(
            schema_faults(&report).len(),
            1,
            "{element}: {:?}",
            codes(&report)
        );
    }
}

#[test]
fn the_published_schema_says_the_same_rules() {
    let schema = montagent_core::schema::generate();
    let transition = schema["$defs"]["Element"]["oneOf"]
        .as_array()
        .expect("the element union")
        .iter()
        .find(|branch| branch["properties"]["type"]["const"] == "transition")
        .expect("a transition branch")
        .clone();
    let kinds = schema["$defs"]["TransitionKind"].to_string();
    for kind in ["crossfade", "wipe", "slide", "push"] {
        assert!(kinds.contains(kind), "{kind} missing from {kinds}");
    }
    let published = transition.to_string();
    assert!(published.contains("direction"), "{published}");
    assert!(published.contains("ease"), "{published}");
    // The conditional: required off `crossfade`, refused on it.
    assert!(
        published.contains("\"if\"") && published.contains("\"else\""),
        "{published}"
    );
}

// ---------------------------------------------------------------------------
// `validate`'s three new checks
// ---------------------------------------------------------------------------

fn findings_of<'r>(report: &'r Report, code: &str) -> Vec<&'r montagent_core::finding::Finding> {
    report.findings.iter().filter(|f| f.code == code).collect()
}

#[test]
fn a_from_or_to_naming_no_element_is_e_transition_ref_missing() {
    for (from, to, missing) in [("nobody", "b", "nobody"), ("a", "ghost", "ghost")] {
        let mut element = transition("crossfade", json!({}));
        element["from"] = json!(from);
        element["to"] = json!(to);
        let report = validated(&bridged(element));
        let found = findings_of(&report, "E-TRANSITION-REF-MISSING");
        assert_eq!(found.len(), 1, "{from}->{to}: {:?}", codes(&report));
        assert_eq!(found[0].class, montagent_core::finding::Class::Error);
        assert_eq!(found[0].fields["target"], missing);
        assert!(found[0].repair.is_some(), "a repair: {:?}", found[0]);
    }
    let clean = validated(&bridged(transition("push", json!({"direction": "up"}))));
    assert!(findings_of(&clean, "E-TRANSITION-REF-MISSING").is_empty());
}

#[test]
fn a_transition_naming_an_audio_element_names_no_visual_element() {
    let audio = json!({"id": "a", "type": "audio", "start": 0, "end": 2000,
                       "source": "music.wav", "source_start": 0, "source_end": 2000});
    let b = json!({"id": "b", "type": "rect", "start": 1000, "end": 3000, "x": 0, "y": 0,
                   "origin": "top-left", "width": 400, "height": 200, "fill": "#0000FF"});
    let report = validated(&bridged_with(audio, b, transition("crossfade", json!({}))));
    assert_eq!(
        findings_of(&report, "E-TRANSITION-REF-MISSING").len(),
        1,
        "{:?}",
        codes(&report)
    );
}

#[test]
fn a_from_equal_to_to_is_e_transition_ref_self() {
    let mut element = transition("wipe", json!({"direction": "left"}));
    element["to"] = json!("a");
    let report = validated(&bridged(element));
    let found = findings_of(&report, "E-TRANSITION-REF-SELF");
    assert_eq!(found.len(), 1, "{:?}", codes(&report));
    assert!(findings_of(&report, "E-TRANSITION-REF-MISSING").is_empty());
    let clean = validated(&bridged(transition("wipe", json!({"direction": "left"}))));
    assert!(findings_of(&clean, "E-TRANSITION-REF-SELF").is_empty());
}

/// `b` on layer 0 and `a` on layer 1, so the incoming `b` paints beneath the outgoing `a`.
fn swapped_layers(transition: Value) -> String {
    let body = bridged(transition);
    let mut value: Value = serde_json::from_str(&body).expect("json");
    value["tracks"][0]["layer"] = json!(1);
    value["tracks"][1]["layer"] = json!(0);
    canonical(&value.to_string())
}

#[test]
fn a_slide_whose_to_paints_beneath_its_from_is_e_transition_slide_under() {
    let report = validated(&swapped_layers(transition(
        "slide",
        json!({"direction": "left"}),
    )));
    let found = findings_of(&report, "E-TRANSITION-SLIDE-UNDER");
    assert_eq!(found.len(), 1, "{:?}", codes(&report));
    let finding = found[0];
    assert_eq!(finding.fields["from"], "a");
    assert_eq!(finding.fields["to"], "b");
    assert_eq!(finding.fields["from_layer"], 1);
    assert_eq!(finding.fields["to_layer"], 0);
    assert!(
        matches!(finding.repair, Some(montagent_core::finding::Repair::None)),
        "refuse-class: {:?}",
        finding.repair
    );
    let rendered =
        montagent_core::wire::render(&report, montagent_core::Wire::Text { verbose: false });
    assert!(rendered.contains("must paint above"), "{rendered}");
}

// ---------------------------------------------------------------------------
// What `frame` paints
// ---------------------------------------------------------------------------

/// The picture `frame` draws at `instant`, at full scale, as PNG.
#[track_caller]
fn picture(body: &str, instant: i64) -> image::RgbaImage {
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(&dir, "p.montagent.json", body);
    common::compare::rendered(&path, instant, true)
}

/// The `frame` answer's JSON at `instant`.
#[track_caller]
fn answered(body: &str, instant: i64) -> Value {
    use montagent_core::verbs::frame::{Ask, frame};
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(&dir, "p.montagent.json", body);
    frame(
        &path,
        &Ask {
            at: Some(instant),
            full: true,
            png: true,
            ..Ask::default()
        },
    )
    .to_json()
}

const RED: [u8; 3] = [0xFF, 0, 0];
const BLUE: [u8; 3] = [0, 0, 0xFF];
const GREEN: [u8; 3] = [0, 0xFF, 0];

fn rgb(picture: &image::RgbaImage, x: u32, y: u32) -> [u8; 3] {
    let p = picture.get_pixel(x, y).0;
    [p[0], p[1], p[2]]
}

/// How many pixels of the picture are exactly `colour`.
fn count(picture: &image::RgbaImage, colour: [u8; 3]) -> usize {
    picture
        .pixels()
        .filter(|p| [p.0[0], p.0[1], p.0[2]] == colour)
        .count()
}

/// The window's frame instants at 25 fps: 1000, 1040, …, 1960.
fn window() -> impl Iterator<Item = i64> {
    (1000..2000).step_by(40)
}

#[test]
fn at_the_start_a_slide_or_push_is_the_outgoing_element_alone() {
    for kind in ["slide", "push"] {
        for direction in DIRECTIONS {
            let at_start = picture(
                &bridged(transition(kind, json!({"direction": direction}))),
                1000,
            );
            assert_eq!(
                count(&at_start, RED),
                400 * 200,
                "{kind} {direction} at p = 0 is not `a` alone"
            );
        }
    }
}

#[test]
fn at_the_last_frame_a_slide_or_push_is_within_one_step_of_the_incoming_element() {
    // 25 frames over 400 px: one frame's step is 16 px across, 8 px down.
    for kind in ["slide", "push"] {
        for (direction, step) in [("left", 16), ("right", 16), ("up", 8), ("down", 8)] {
            let last = picture(
                &bridged(transition(kind, json!({"direction": direction}))),
                1960,
            );
            let side = match direction {
                "left" | "right" => 200,
                _ => 400,
            };
            assert!(
                count(&last, BLUE) >= (400 * 200) - step * side,
                "{kind} {direction}: {} blue pixels at the last frame",
                count(&last, BLUE)
            );
            assert!(
                count(&last, BLUE) < 400 * 200,
                "{kind} {direction} arrived early"
            );
        }
    }
}

#[test]
fn a_left_slide_enters_from_the_right() {
    let mid = picture(
        &bridged(transition("slide", json!({"direction": "left"}))),
        1500,
    );
    // p = 0.5, n = 200: `b` has travelled half the frame in from the right edge.
    assert_eq!(rgb(&mid, 10, 100), RED);
    assert_eq!(rgb(&mid, 199, 100), RED);
    assert_eq!(rgb(&mid, 200, 100), BLUE);
    assert_eq!(rgb(&mid, 399, 100), BLUE);
    // And a `down` slide enters from the top.
    let down = picture(
        &bridged(transition("slide", json!({"direction": "down"}))),
        1500,
    );
    assert_eq!(rgb(&down, 200, 10), BLUE);
    assert_eq!(rgb(&down, 200, 190), RED);
}

#[test]
fn a_wipe_over_a_contrasting_background_shows_no_background_at_any_frame() {
    for direction in DIRECTIONS {
        for body in [
            bridged(transition(
                "wipe",
                json!({"direction": direction, "ease": "ease-in-out"}),
            )),
            swapped_layers(transition("wipe", json!({"direction": direction}))),
        ] {
            for instant in window() {
                let painted = picture(&body, instant);
                assert_eq!(
                    count(&painted, GREEN),
                    0,
                    "{direction} wipe shows the background at {instant}"
                );
                assert_eq!(
                    count(&painted, RED) + count(&painted, BLUE),
                    400 * 200,
                    "{direction} wipe paints something that is neither side at {instant}"
                );
            }
        }
    }
}

#[test]
fn a_left_wipe_edge_travels_left() {
    let mid = picture(
        &bridged(transition("wipe", json!({"direction": "left"}))),
        1500,
    );
    assert_eq!(rgb(&mid, 199, 100), RED);
    assert_eq!(rgb(&mid, 200, 100), BLUE);
}

/// Two opaque full-frame elements over a green background: a gap at the join shows green,
/// and an overlap makes the counts of the two colours disagree with `n`.
#[test]
fn a_push_shows_no_gap_and_no_overlap_at_the_join() {
    for direction in DIRECTIONS {
        for instant in window() {
            let body = bridged(transition(
                "push",
                json!({"direction": direction, "ease": [0.3, 0.0, 0.2, 1.0]}),
            ));
            let painted = picture(&body, instant);
            assert_eq!(
                count(&painted, GREEN),
                0,
                "{direction} push gap at {instant}"
            );
            let answer = answered(&body, instant);
            let n = answer["frame"]["transitions"][0]["pixels"]
                .as_i64()
                .expect("a push states its pixels");
            let side = match direction {
                "left" | "right" => 200,
                _ => 400,
            };
            // Exactly `n` rows or columns of `b` and the rest of `a`: no overlap, no gap.
            assert_eq!(
                count(&painted, BLUE),
                (n * side) as usize,
                "{direction} at {instant}"
            );
            assert_eq!(
                count(&painted, RED),
                (400 * 200 - n * side) as usize,
                "{direction} at {instant}"
            );
        }
    }
}

#[test]
fn an_eased_transitions_n_matches_a_hand_computation() {
    // `n = floor(p × 400)` for a `left` push over the 1000 ms window, `p` the ease at the
    // fraction of the window. The expected values were computed by hand, solving each
    // bezier's x(s) = fraction for s and taking y(s).
    let expected: [(Value, [(i64, i64); 3]); 2] = [
        // ease-in-out [0.42, 0, 0.58, 1]: fraction 0.2 → p 0.081660 (32.66 px),
        // 0.3 → 0.187396 (74.96 px), 0.8 → 0.918340 (367.34 px).
        (json!("ease-in-out"), [(1200, 32), (1300, 74), (1800, 367)]),
        // [0.2, 0, 0.2, 1.4], which overshoots: 0.2 → 0.442628 (177.05 px),
        // 0.3 → 0.706979 (282.79 px), 0.8 → 1.067001 (426.80 px, past the frame).
        (
            json!([0.2, 0.0, 0.2, 1.4]),
            [(1200, 177), (1300, 282), (1800, 426)],
        ),
    ];
    for (ease, frames) in expected {
        let body = bridged(transition(
            "push",
            json!({"direction": "left", "ease": ease}),
        ));
        for (instant, n) in frames {
            let answer = answered(&body, instant);
            assert_eq!(
                answer["frame"]["transitions"][0]["pixels"], n,
                "{ease} at {instant}: {}",
                answer["frame"]["transitions"]
            );
        }
    }
}

#[test]
fn a_small_element_slides_a_full_frame_width_and_keeps_its_own_keyed_position() {
    let small = json!({"id": "b", "type": "rect", "start": 1000, "end": 3000,
                       "x": [{"t": 1000, "v": 100}, {"t": 2000, "v": 140, "ease": "linear"}],
                       "y": 80, "origin": "top-left", "width": 40, "height": 40,
                       "fill": "#0000FF"});
    let a = json!({"id": "a", "type": "rect", "start": 0, "end": 2000, "x": 0, "y": 0,
                   "origin": "top-left", "width": 400, "height": 200, "fill": "#FF0000"});
    let body = bridged_with(a, small, transition("slide", json!({"direction": "left"})));
    // At 1000 it is a full frame width off to the right: `a` alone.
    assert_eq!(count(&picture(&body, 1000), BLUE), 0);
    // At 1750, p = 0.75, n = 300: its own x is 130, plus 400 − 300 = 230.
    let at = picture(&body, 1750);
    assert_eq!(rgb(&at, 229, 100), RED);
    assert_eq!(rgb(&at, 230, 100), BLUE);
    assert_eq!(rgb(&at, 269, 100), BLUE);
    assert_eq!(rgb(&at, 270, 100), RED);
}

#[test]
fn a_crossfade_still_fades_linearly() {
    let mid = picture(&bridged(transition("crossfade", json!({}))), 1500);
    // Both at half strength, `b` over `a` over green: `a` gives (128, 128, 0), and `b` at
    // one half over that gives (64, 64, 128).
    let [r, g, b] = rgb(&mid, 200, 100);
    assert!(
        (62..=66).contains(&r) && (62..=66).contains(&g) && (126..=130).contains(&b),
        "{r} {g} {b}"
    );
}

// ---------------------------------------------------------------------------
// `query --at` reads the same resolved stack
// ---------------------------------------------------------------------------

#[track_caller]
fn queried(body: &str, instant: i64) -> Value {
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(&dir, "p.montagent.json", body);
    let document = montagent_core::parse::read(&path).expect("the project parses");
    serde_json::to_value(montagent_core::verbs::query::at::at(
        &document, instant, None,
    ))
    .expect("the answer serialises")
}

fn row<'v>(view: &'v Value, id: &str) -> &'v Value {
    view["stack"]
        .as_array()
        .expect("a stack")
        .iter()
        .find(|row| row["id"] == id)
        .unwrap_or_else(|| panic!("no `{id}` in {view}"))
}

fn uncovered(view: &Value, x: i64, y: i64) -> bool {
    view["not_covered"]
        .as_array()
        .expect("not_covered")
        .iter()
        .any(|rect| {
            let (rx, ry) = (rect["x"].as_i64().unwrap(), rect["y"].as_i64().unwrap());
            let (w, h) = (
                rect["width"].as_i64().unwrap(),
                rect["height"].as_i64().unwrap(),
            );
            rx <= x && x < rx + w && ry <= y && y < ry + h
        })
}

#[test]
fn query_at_mid_slide_reports_the_incoming_elements_offset_box() {
    let a = json!({"id": "a", "type": "rect", "start": 0, "end": 2000, "x": 0, "y": 0,
                   "origin": "top-left", "width": 40, "height": 40, "fill": "#FF0000"});
    let b = json!({"id": "b", "type": "rect", "start": 1000, "end": 3000, "x": 100, "y": 80,
                   "origin": "top-left", "width": 40, "height": 40, "fill": "#0000FF"});
    let body = bridged_with(a, b, transition("slide", json!({"direction": "left"})));
    // p = 0.75, n = 300: `b` is 100 px right of where its `x` puts it.
    let view = queried(&body, 1750);
    let incoming = row(&view, "b");
    assert_eq!(
        incoming["transition"]["offset"],
        json!([100, 0]),
        "{incoming}"
    );
    assert_eq!(
        incoming["transition"]["box"],
        json!({"x": 200, "y": 80, "width": 40, "height": 40}),
        "{incoming}"
    );
    assert_eq!(
        row(&view, "a")["transition"],
        Value::Null,
        "a slide leaves `from` still"
    );
    // `NOT COVERED` follows the pixels: the offset box is covered and the declared one not.
    assert!(!uncovered(&view, 210, 90), "{}", view["not_covered"]);
    assert!(uncovered(&view, 110, 90), "{}", view["not_covered"]);
}

#[test]
fn query_at_mid_wipe_reports_each_sides_cut_box() {
    let view = queried(
        &bridged(transition("wipe", json!({"direction": "left"}))),
        1500,
    );
    assert_eq!(
        row(&view, "a")["transition"]["box"],
        json!({"x": 0, "y": 0, "width": 200, "height": 200})
    );
    assert_eq!(
        row(&view, "b")["transition"]["box"],
        json!({"x": 200, "y": 0, "width": 200, "height": 200})
    );
    assert!(view["not_covered"].as_array().expect("a list").is_empty());
}

#[test]
fn an_elements_own_clip_moves_with_a_push_and_is_intersected_with_a_wipe() {
    let clipped = json!({"id": "a", "type": "image", "source": "img/none.png", "start": 0,
                         "end": 2000, "x": 0, "y": 0, "origin": "top-left", "width": 400,
                         "height": 200, "fit": "literal", "clip": [0, 0, 300, 200]});
    let b = json!({"id": "b", "type": "rect", "start": 1000, "end": 3000, "x": 0, "y": 0,
                   "origin": "top-left", "width": 400, "height": 200, "fill": "#0000FF"});
    // A right wipe at p = 0.5: `a` keeps the side not yet reached, x ≥ 200, cut by its
    // own clip to x < 300.
    let wipe = queried(
        &bridged_with(
            clipped.clone(),
            b.clone(),
            transition("wipe", json!({"direction": "right"})),
        ),
        1500,
    );
    assert_eq!(
        row(&wipe, "a")["transition"]["box"],
        json!({"x": 200, "y": 0, "width": 100, "height": 200})
    );
    // A right push at p = 0.5: `a` and its clip move 200 px right together.
    let push = queried(
        &bridged_with(
            clipped,
            b,
            transition("push", json!({"direction": "right"})),
        ),
        1500,
    );
    assert_eq!(
        row(&push, "a")["transition"]["box"],
        json!({"x": 200, "y": 0, "width": 300, "height": 200})
    );
}

#[test]
fn the_printed_frame_answer_names_the_transition_and_where_it_moved_each_element() {
    let answer = answered(
        &bridged(transition("push", json!({"direction": "left"}))),
        1500,
    );
    let printed = montagent_core::text::render(&answer, montagent_core::text::Options::default())
        .expect("the answer renders");
    assert!(printed.contains("push"), "{printed}");
    assert!(printed.contains("left, 200 px"), "{printed}");
    assert!(printed.contains("transition: moved [-200, 0]"), "{printed}");
    assert!(printed.contains("transition: moved [200, 0]"), "{printed}");
}

// ---------------------------------------------------------------------------
// Gating: byte identity across painter counts and chunk sizes (ADR-0144)
// ---------------------------------------------------------------------------

/// The spy trailer's directory, for its title face and a photo.
fn trailer() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/benchmark/spy-trailer")
}

/// Every kind and direction in one 320×180, 30 fps project: thirteen clips handing over in
/// turn, each bridged to the next by its own transition over 200 ms (six frames), so every
/// chunk size below puts a boundary inside some transition. The clips are an image with a
/// `clip` and a `blur`, text with a `shadow` and a `blur`, a rect with a `mask`, and an
/// ellipse, each with sub-pixel keyed motion, a keyed `rotation` and a keyed `scale`. Each
/// clip sits one layer above the last, so every slide's `to` paints above its `from`.
fn every_kind_project(line: u32) -> std::path::PathBuf {
    let dir = common::tempdir(line);
    for asset in ["fonts/Cinzel-Bold.ttf", "img/boat.jpg"] {
        let to = dir.join(asset);
        std::fs::create_dir_all(to.parent().expect("a parent")).expect("asset dir");
        std::fs::copy(trailer().join(asset), &to).expect("copy the trailer's asset");
    }
    let mut pairs: Vec<(&str, Option<&str>)> = vec![("crossfade", None)];
    for kind in KINDS {
        for direction in DIRECTIONS {
            pairs.push((kind, Some(direction)));
        }
    }
    let keyed = |start: i64, end: i64, from: f64, to: f64| json!([{"t": start, "v": from}, {"t": end, "v": to, "ease": "ease-in-out"}]);
    let pixels = |start: i64, end: i64, from: i64, to: i64| json!([{"t": start, "v": from}, {"t": end, "v": to, "ease": "linear"}]);
    let blur = json!({"name": "blur", "radius": 3});
    let shadow = json!({"name": "shadow", "dx": 2, "dy": 3, "radius": 6, "color": "#000000",
                        "opacity": 0.7});
    let mut elements = Vec::new();
    for i in 0..=pairs.len() as i64 {
        let (start, end) = (i * 300, i * 300 + 500);
        let id = format!("c{i}");
        // `x` keyed by fewer pixels than the span has frames: sub-pixel motion.
        let mut element = json!({
            "id": id, "start": start, "end": end,
            "x": pixels(start, end, 150, 167), "y": pixels(start, end, 88, 95),
            "origin": "center",
            "rotation": keyed(start, end, -4.0, 7.5),
            "scale": [{"t": start, "v": [0.9, 0.95]},
                      {"t": end, "v": [1.1, 1.05], "ease": "linear"}],
        });
        let body = match i % 4 {
            0 => json!({"type": "image", "source": "img/boat.jpg", "width": 282, "height": 160,
                        "fit": "cover", "clip": [20, 10, 280, 160], "effects": [blur]}),
            1 => json!({"type": "text", "font": "cinzel-bold", "size": 40, "color": "#E3C067",
                        "align": "center", "runs": [{"text": "SPY"}], "width": 200,
                        "height": 60, "effects": [shadow, blur]}),
            2 => json!({"type": "rect", "width": 300, "height": 170, "fill": "#2E86FF",
                        "effects": [{"name": "mask", "shape": "circle"}]}),
            _ => json!({"type": "ellipse", "width": 260, "height": 150, "fill": "#F2F2F2",
                        "stroke": "#FF3B30", "stroke_width": 4}),
        };
        for (key, value) in body.as_object().expect("an object") {
            element[key] = value.clone();
        }
        elements.push(element);
    }
    let mut tracks: Vec<Value> = elements
        .into_iter()
        .enumerate()
        .map(|(i, element)| json!({"name": format!("clip{i}"), "layer": i, "elements": [element]}))
        .collect();
    let transitions: Vec<Value> = pairs
        .iter()
        .enumerate()
        .map(|(i, (kind, direction))| {
            let i = i as i64;
            let mut element = json!({"id": format!("t{i}"), "type": "transition",
                                     "start": (i + 1) * 300, "end": i * 300 + 500,
                                     "kind": kind, "from": format!("c{i}"),
                                     "to": format!("c{}", i + 1)});
            if let Some(direction) = direction {
                element["direction"] = json!(direction);
                if i % 2 == 0 {
                    element["ease"] = json!("ease-in-out");
                }
            }
            json!({"name": format!("bridge{i}"), "layer": 100 + i, "elements": [element]})
        })
        .collect();
    tracks.extend(transitions);
    let duration = pairs.len() as i64 * 300 + 500;
    let project = json!({
        "frame": {"width": 320, "height": 180}, "fps": 30, "background": "#101418",
        "duration": duration, "output": "out/every-kind.mp4",
        "fonts": {"cinzel-bold": [{"file": "fonts/Cinzel-Bold.ttf"}]},
        "fontVendor": {"fonts/Cinzel-Bold.ttf": {
            "licence": "OFL-1.1",
            "source": "google/fonts ofl/cinzel, instanced wght=700",
            "sha256": "c9ac320a5f48ecb57db76bc0b942ccc39eb89994e37fb182a454ec273ee43e02"}},
        "tracks": tracks,
    });
    write_project(
        &dir,
        "every-kind.montagent.json",
        &canonical(&project.to_string()),
    )
}

/// One render's per-frame hashes at the encoder's input, with K painters over chunks of C
/// frames, or on one painter.
fn frame_hashes(path: &std::path::Path, forced: montagent_core::verbs::render::Forced) -> Vec<u64> {
    use montagent_core::verbs::render::{Ask, Progress, force_painting, render, tap_frames};
    let path = path.to_path_buf();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _forced = force_painting(forced);
        let tap = tap_frames();
        let answer = render(&path, &Ask::default(), &mut |_: Progress| {});
        let _ = tx.send((answer.report().exit_code(), answer.to_json(), tap.hashes()));
    });
    let (exit, answer, hashes) = rx
        .recv_timeout(std::time::Duration::from_secs(300))
        .expect("the render finished rather than hanging");
    assert_eq!(exit, montagent_core::report::ExitCode::Ok, "{answer}");
    hashes
}

#[test]
fn every_kind_and_direction_paints_the_same_bytes_on_any_painter_count_and_chunk_size() {
    use montagent_core::verbs::render::Forced;
    if !common::has_ffprobe() {
        return;
    }
    let path = every_kind_project(line!());
    let report = validate(&path);
    assert!(
        report
            .findings
            .iter()
            .all(|f| f.class != montagent_core::finding::Class::Error),
        "the gating fixture is itself clean: {:?}",
        report
            .findings
            .iter()
            .filter(|f| f.class == montagent_core::finding::Class::Error)
            .map(|f| format!(
                "{} {}",
                f.code,
                serde_json::to_string(&f.fields).unwrap_or_default()
            ))
            .collect::<Vec<_>>()
    );
    // Every transition is running mid-window, and every clip it bridges is painted: the
    // fixture exercises what it claims to.
    for i in 0..13 {
        let answer = montagent_core::verbs::frame::frame(
            &path,
            &montagent_core::verbs::frame::Ask {
                at: Some((i + 1) * 300 + 100),
                ..montagent_core::verbs::frame::Ask::default()
            },
        )
        .to_json();
        let running = &answer["frame"]["transitions"];
        assert_eq!(running.as_array().map(Vec::len), Some(1), "t{i}: {running}");
        assert_eq!(running[0]["element"], format!("t{i}"));
        assert_eq!(answer["frame"]["not_painted"], json!([]), "t{i}: {answer}");
        assert_eq!(
            answer["frame"]["painted_partially"],
            json!([]),
            "t{i}: {answer}"
        );
    }
    let baseline = frame_hashes(&path, Forced::OnePainter);
    assert_eq!(
        baseline.len(),
        132,
        "13 transitions' worth of frames at 30 fps"
    );
    for (painters, chunk) in [(3, 2), (4, 7), (2, 1), (5, 5)] {
        let chunked = frame_hashes(
            &path,
            Forced::Chunks {
                painters,
                chunk,
                window_bytes: None,
            },
        );
        let differ: Vec<usize> = (0..baseline.len())
            .filter(|&i| baseline.get(i) != chunked.get(i))
            .collect();
        assert!(
            differ.is_empty() && chunked.len() == baseline.len(),
            "K={painters}, C={chunk}: frames {differ:?} differ"
        );
    }
}

// ---------------------------------------------------------------------------
// `shift`, `compare` and `timeline` carry the new fields unchanged
// ---------------------------------------------------------------------------

#[test]
fn shift_compare_and_timeline_carry_direction_and_ease_unchanged() {
    use montagent_core::report::ExitCode;
    let dir = common::tempdir(line!());
    let fields = json!({"direction": "up", "ease": [0.2, 0.0, 0.2, 1.4]});
    // No `duration`, so moving everything later leaves no slack to the end to release.
    let mut value: Value =
        serde_json::from_str(&bridged(transition("push", fields.clone()))).expect("json");
    value.as_object_mut().expect("an object").remove("duration");
    let body = canonical(&value.to_string());
    let reference = write_project(&dir, "ref.montagent.json", &body);
    let path = write_project(&dir, "p.montagent.json", &body);

    let shifted = montagent_core::verbs::shift::shift(
        &path,
        &montagent_core::verbs::shift::Ask {
            at: 0,
            delta: 500,
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
    let written: Value =
        serde_json::from_str(&std::fs::read_to_string(&path).expect("the file")).expect("json");
    let t = &written["tracks"][2]["elements"][0];
    assert_eq!(t["start"], 1500, "{t}");
    assert_eq!(t["kind"], "push");
    assert_eq!(t["direction"], fields["direction"]);
    assert_eq!(t["ease"], fields["ease"]);

    let compared = montagent_core::verbs::compare::compare(&reference, &path);
    assert_eq!(compared.exit_code(), ExitCode::Ok, "{:?}", codes(&compared));

    let timeline = montagent_core::verbs::timeline::timeline(&path);
    assert_eq!(timeline.report().exit_code(), ExitCode::Ok);
}

#[test]
fn a_slide_in_order_and_a_push_or_wipe_in_either_order_stay_clean() {
    let clean = validated(&bridged(transition("slide", json!({"direction": "left"}))));
    assert!(findings_of(&clean, "E-TRANSITION-SLIDE-UNDER").is_empty());
    for kind in ["push", "wipe", "crossfade"] {
        let extra = match kind {
            "crossfade" => json!({}),
            _ => json!({"direction": "down"}),
        };
        let report = validated(&swapped_layers(transition(kind, extra)));
        assert!(
            findings_of(&report, "E-TRANSITION-SLIDE-UNDER").is_empty(),
            "{kind}: {:?}",
            codes(&report)
        );
    }
}
