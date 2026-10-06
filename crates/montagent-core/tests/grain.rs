//! `grain`: a named effect with a literal seed that re-rolls on every output frame
//! (ADR-0156 §3, §4; #725).
//!
//! Asked at the seams an agent meets: the model's parse (what is a schema error), the `frame`
//! verb's pixels, `validate`'s `R-GRAIN-SEED-SHARED` and `query --at`. The gating test, frames
//! byte-identical across painter counts with the bounds hint on and off, is in
//! `tests/painters.rs` and `tests/filter_bound.rs` beside the other painter-count guards.

use montagent_core::model::Effect;
use serde_json::{Value, json};

mod common;

fn parse(effect: Value) -> Result<Effect, String> {
    serde_json::from_value::<Effect>(effect).map_err(|e| e.to_string())
}

fn grain() -> Value {
    json!({"name": "grain", "seed": 7, "amount": 0.2, "size": 2, "mono": true})
}

/// `grain()` with `key` set to `value`.
fn with(key: &str, value: Value) -> Value {
    let mut member = grain();
    member[key] = value;
    member
}

/// `grain()` without `key`.
fn without(key: &str) -> Value {
    let mut member = grain();
    member.as_object_mut().expect("an object").remove(key);
    member
}

// ---------------------------------------------------------------------------------------
// The schema.
// ---------------------------------------------------------------------------------------

#[test]
fn the_member_parses_with_its_four_keys_at_their_ranges_ends() {
    parse(grain()).expect("the spec's own example");
    for (key, value) in [
        ("seed", json!(0)),
        ("seed", json!(2_147_483_647_i64)),
        ("amount", json!(0)),
        ("amount", json!(1)),
        ("size", json!(1)),
        ("size", json!(8)),
        ("mono", json!(false)),
    ] {
        parse(with(key, value.clone())).unwrap_or_else(|e| panic!("{key}: {value}: {e}"));
    }
}

#[test]
fn every_key_is_required() {
    for key in ["seed", "amount", "size", "mono"] {
        let error = parse(without(key)).expect_err(key);
        assert!(error.contains(key), "{key}: {error}");
    }
}

#[test]
fn a_value_out_of_range_or_of_the_wrong_type_is_a_schema_error() {
    for (key, value) in [
        ("seed", json!(-1)),
        ("seed", json!(2_147_483_648_i64)),
        ("seed", json!(7.5)),
        ("amount", json!(-0.1)),
        ("amount", json!(1.5)),
        ("size", json!(0)),
        ("size", json!(9)),
        ("size", json!(2.5)),
    ] {
        let error = parse(with(key, value.clone())).expect_err(&format!("{key}: {value}"));
        assert!(error.contains(key), "{key}: {value}: {error}");
    }
    parse(with("mono", json!(1))).expect_err("mono is a boolean");
}

#[test]
fn a_keyframe_list_on_seed_size_or_mono_is_a_schema_error_and_on_amount_is_legal() {
    let keyed = |from: Value, to: Value| json!([{"t": 0, "v": from}, {"t": 400, "v": to, "ease": "linear"}]);
    for (key, from, to) in [
        ("seed", json!(1), json!(2)),
        ("size", json!(1), json!(4)),
        ("mono", json!(true), json!(false)),
    ] {
        parse(with(key, keyed(from, to))).expect_err(key);
    }
    parse(with("amount", keyed(json!(0), json!(0.3)))).expect("amount animates (ADR-0146)");
    // A keyed amount holds its range in every record.
    parse(with("amount", keyed(json!(0), json!(1.2)))).expect_err("a record past 1");
}

#[test]
fn only_amount_is_on_the_one_derived_list_of_animatable_properties() {
    let element = json!({"type": "rect", "effects": [grain()]});
    let names: Vec<String> = montagent_core::animatable::declared(&element)
        .into_iter()
        .map(|declared| declared.path.clone())
        .collect();
    assert_eq!(names, ["effects[0].amount (grain)"]);
}

// ---------------------------------------------------------------------------------------
// The paint, through `frame`.
// ---------------------------------------------------------------------------------------

use common::{canonical, tempdir, write_project};

/// A project of one track per element, the first lowest, on a 96×64 grey frame.
fn project(fps: i64, elements: &[Value]) -> Value {
    let tracks: Vec<Value> = elements
        .iter()
        .enumerate()
        .map(|(i, element)| json!({"name": format!("t{i}"), "layer": i, "elements": [element]}))
        .collect();
    json!({"frame": {"width": 96, "height": 64}, "fps": fps, "background": "#404040",
           "duration": 2000, "tracks": tracks})
}

/// A 40×24 rect at `(8, 8)`, top-left, alive 0..2000, a horizontal gradient so every channel
/// has room to move both ways, carrying `effects`, with `extra` laid over it.
fn plate(effects: Value, extra: Value) -> Value {
    let mut element = json!({"id": "plate", "type": "rect", "start": 0, "end": 2000,
        "x": 8, "y": 8, "origin": "top-left", "width": 40, "height": 24,
        "fill": {"gradient": "linear", "angle": 0, "stops": [
            {"offset": 0, "color": "#203060"}, {"offset": 1, "color": "#E0C0A0"}]},
        "effects": effects});
    for (key, value) in extra.as_object().expect("an object") {
        element[key] = value.clone();
    }
    element
}

/// One instant of `project`, at true scale, as RGBA.
#[track_caller]
fn painted(project: &Value, at: i64) -> image::RgbaImage {
    let dir = tempdir(std::panic::Location::caller().line());
    let path = write_project(&dir, "p.montagent.json", &canonical(&project.to_string()));
    common::compare::rendered(&path, at, true)
}

fn same(a: &image::RgbaImage, b: &image::RgbaImage) -> bool {
    a.as_raw() == b.as_raw()
}

#[test]
fn amount_zero_paints_the_bytes_of_no_effect() {
    for extra in [json!({}), json!({"rotation": 17.5, "scale": [1.3, 0.8]})] {
        let bare = project(25, &[plate(json!([]), extra.clone())]);
        let zero = project(
            25,
            &[plate(json!([with("amount", json!(0))]), extra.clone())],
        );
        let background = painted(&project(25, &[]), 0);
        assert!(
            !same(&painted(&bare, 0), &background),
            "the plate is painted"
        );
        for at in [0, 40, 520] {
            assert!(
                same(&painted(&bare, at), &painted(&zero, at)),
                "{extra} at {at}"
            );
        }
    }
}

#[test]
fn the_same_project_paints_the_same_bytes_and_another_seed_another_pattern() {
    let doc = project(25, &[plate(json!([grain()]), json!({}))]);
    let first = painted(&doc, 200);
    assert!(same(&first, &painted(&doc, 200)), "painted twice");
    let bare = painted(&project(25, &[plate(json!([]), json!({}))]), 200);
    assert!(!same(&first, &bare), "the grain is visible");
    let reseeded = project(25, &[plate(json!([with("seed", json!(8))]), json!({}))]);
    assert!(
        !same(&first, &painted(&reseeded, 200)),
        "seed 8 is another pattern"
    );
}

#[test]
fn two_consecutive_frames_differ() {
    let doc = project(25, &[plate(json!([grain()]), json!({}))]);
    for (a, b) in [(0, 40), (40, 80), (960, 1000)] {
        assert!(!same(&painted(&doc, a), &painted(&doc, b)), "{a} and {b}");
    }
    // Two instants inside one frame paint that frame's draw.
    assert!(same(&painted(&doc, 40), &painted(&doc, 79)));
}

/// The whole millisecond frame `n` is painted at, `⌊n × 1000 / fps⌋` (ADR-0077).
fn instant(n: i64, fps: i64) -> i64 {
    n * 1000 / fps
}

#[test]
fn an_element_starting_n_whole_frames_later_paints_the_same_pixels_n_frames_later() {
    // At 30 fps a frame is 33⅓ ms, so a `start` of whole milliseconds never sits exactly on
    // most frames: the shift test holds for every N, not only multiples of 3.
    for fps in [30, 24, 25] {
        let at = |start: i64| project(fps, &[plate(json!([grain()]), json!({"start": start}))]);
        let original = at(0);
        for n in [1, 2, 3, 7, 30] {
            let shifted = at(instant(n, fps));
            for k in [0, 1, 5] {
                assert!(
                    same(
                        &painted(&original, instant(k, fps)),
                        &painted(&shifted, instant(k + n, fps))
                    ),
                    "{fps} fps, shifted {n} frames, local frame {k}"
                );
            }
        }
    }
}

#[test]
fn two_starts_on_the_same_frame_draw_the_same_pattern() {
    // At 30 fps frame 1 is painted at 33 ms, the first frame for any start from 1 to 33.
    let at = |start: i64| project(30, &[plate(json!([grain()]), json!({"start": start}))]);
    assert!(same(&painted(&at(1), 66), &painted(&at(33), 66)));
}

#[test]
fn a_transparent_pixel_stays_transparent_and_a_rect_keeps_its_edges() {
    // A transparent fill under the strongest grain paints nothing.
    let clear = plate(
        json!([with("amount", json!(1))]),
        json!({"fill": "#FFFFFF00"}),
    );
    let bare = painted(&project(25, &[]), 200);
    assert!(same(&painted(&project(25, &[clear]), 200), &bare));

    // A flat rect: every pixel outside it is the background's, and inside it the grain
    // moves pixels both ways.
    let flat = plate(
        json!([with("amount", json!(0.3))]),
        json!({"fill": "#808080"}),
    );
    let picture = painted(&project(25, &[flat]), 200);
    let (mut lighter, mut darker) = (0, 0);
    for (x, y, pixel) in picture.enumerate_pixels() {
        let inside = (8..48).contains(&x) && (8..32).contains(&y);
        if !inside {
            assert_eq!(
                pixel.0,
                [0x40, 0x40, 0x40, 0xFF],
                "({x}, {y}) is background"
            );
            continue;
        }
        assert_eq!(pixel.0[3], 0xFF);
        lighter += usize::from(pixel.0[0] > 0x80);
        darker += usize::from(pixel.0[0] < 0x80);
    }
    assert!(
        lighter > 100 && darker > 100,
        "{lighter} lighter, {darker} darker"
    );
}

#[test]
fn cells_are_size_square_in_element_space_anchored_at_the_box_origin() {
    // `size: 4` on an element at `scale: 2`: 8×8 frame-pixel blocks from (8, 8), each one
    // colour; `mono`, so each block is grey.
    let flat = plate(
        json!([with("size", json!(4))]),
        json!({"fill": "#808080", "scale": [2, 2], "width": 20, "height": 12}),
    );
    let picture = painted(&project(25, &[flat]), 200);
    let mut blocks = std::collections::BTreeSet::new();
    for by in 0..3 {
        for bx in 0..5 {
            let (x0, y0) = (8 + bx * 8, 8 + by * 8);
            let corner = picture.get_pixel(x0, y0).0;
            assert_eq!(corner[0], corner[1], "mono");
            assert_eq!(corner[1], corner[2], "mono");
            for y in y0..y0 + 8 {
                for x in x0..x0 + 8 {
                    assert_eq!(
                        picture.get_pixel(x, y).0,
                        corner,
                        "({x}, {y}) in block ({bx}, {by})"
                    );
                }
            }
            blocks.insert(corner[0]);
        }
    }
    assert!(blocks.len() > 5, "the blocks draw apart: {blocks:?}");
}

#[test]
fn mono_false_draws_each_channel_on_its_own() {
    let flat = plate(
        json!([with("mono", json!(false))]),
        json!({"fill": "#808080"}),
    );
    let picture = painted(&project(25, &[flat]), 200);
    let unequal = picture
        .enumerate_pixels()
        .filter(|(x, y, _)| (8..48).contains(x) && (8..32).contains(y))
        .filter(|(_, _, p)| p.0[0] != p.0[1] || p.0[1] != p.0[2])
        .count();
    assert!(unequal > 500, "{unequal}");
}

// ---------------------------------------------------------------------------------------
// `R-GRAIN-SEED-SHARED`.
// ---------------------------------------------------------------------------------------

use montagent_core::report::{ExitCode, Report};

const SHARED: &str = "R-GRAIN-SEED-SHARED";

#[track_caller]
fn validated(project: &Value) -> Report {
    let dir = tempdir(std::panic::Location::caller().line());
    let path = write_project(&dir, "p.montagent.json", &canonical(&project.to_string()));
    montagent_core::validate(&path)
}

fn codes(report: &Report) -> Vec<&str> {
    report.findings.iter().map(|f| f.code.as_str()).collect()
}

/// A flat 20×20 rect named `id` at `x`, alive `start..end`, carrying `effects`.
fn card(id: &str, x: i64, start: i64, end: i64, effects: Value) -> Value {
    json!({"id": id, "type": "rect", "start": start, "end": end, "x": x, "y": 30,
           "width": 20, "height": 20, "fill": "#808080", "effects": effects})
}

#[test]
fn two_copied_grains_starting_together_are_a_review_naming_both() {
    let report = validated(&project(
        30,
        &[
            card("left", 20, 0, 1000, json!([grain()])),
            card(
                "right",
                70,
                0,
                1000,
                json!([{"name": "blur", "radius": 2}, grain()]),
            ),
        ],
    ));
    assert_eq!(codes(&report), [SHARED]);
    let finding = &report.findings[0];
    assert_eq!(finding.class, montagent_core::finding::Class::Review);
    assert_eq!(report.exit_code(), ExitCode::Ok, "a review gates nothing");
    let text =
        montagent_core::text::render(&report.to_json(), montagent_core::text::Options::default())
            .expect("the report renders");
    for word in ["left", "right", "effects[1]", "seed"] {
        assert!(text.contains(word), "{word}: {text}");
    }
}

#[test]
fn starts_on_the_same_frame_count_as_together_and_on_another_frame_do_not() {
    // At 30 fps both 1 ms and 33 ms first paint on frame 1; 34 ms first paints on frame 2.
    let pair = |a: i64, b: i64| {
        validated(&project(
            30,
            &[
                card("left", 20, a, 1000, json!([grain()])),
                card("right", 70, b, 1000, json!([grain()])),
            ],
        ))
    };
    // Starts off the frame grid are also `N-QUANTIZATION`'s to note; only this code is asked.
    let shared = |report: Report| codes(&report).contains(&SHARED);
    assert!(shared(pair(1, 33)));
    assert!(!shared(pair(0, 34)));
    assert!(!shared(pair(33, 34)));
}

#[test]
fn it_is_silent_when_mono_size_or_seed_differs_or_the_two_are_never_visible_together() {
    let pair = |a: Value, b: Value, second: (i64, i64)| {
        validated(&project(
            30,
            &[
                card("left", 20, 0, 500, json!([a])),
                card("right", 70, second.0, second.1, json!([b])),
            ],
        ))
    };
    for (other, why) in [
        (with("mono", json!(false)), "mono"),
        (with("size", json!(3)), "size"),
        (with("seed", json!(8)), "seed"),
    ] {
        let report = pair(grain(), other, (0, 500));
        assert!(codes(&report).is_empty(), "{why}: {:?}", codes(&report));
    }
    // Not visible together: one ends before the other begins.
    assert!(codes(&pair(grain(), grain(), (500, 900))).is_empty());
    // A different `amount` draws the same pattern, so it still counts.
    assert_eq!(
        codes(&pair(grain(), with("amount", json!(0.6)), (0, 500))),
        [SHARED]
    );
}

#[test]
fn two_identical_members_in_one_list_draw_the_same_pattern_too() {
    let report = validated(&project(
        30,
        &[card("only", 20, 0, 1000, json!([grain(), grain()]))],
    ));
    assert_eq!(codes(&report), [SHARED]);
}

// ---------------------------------------------------------------------------------------
// `query --at`.
// ---------------------------------------------------------------------------------------

#[track_caller]
fn queried(doc: &Value, at: i64) -> Value {
    use montagent_core::verbs::query::{Ask, query};
    let dir = tempdir(std::panic::Location::caller().line());
    let path = write_project(&dir, "p.montagent.json", &canonical(&doc.to_string()));
    let answer = query(
        &path,
        &Ask {
            at: Some(at),
            ..Ask::default()
        },
    );
    assert_eq!(answer.report().exit_code(), ExitCode::Ok);
    answer.to_json()
}

#[test]
fn query_at_reports_each_grain_s_resolved_values_and_the_frame_it_draws_from() {
    // `amount` keyed 0 → 0.4 over 400 ms; at 30 fps a start of 100 ms first paints on frame
    // 3, and 250 ms lies in frame 7, so the draw is local frame 4.
    let keyed = json!([{"t": 0, "v": 0}, {"t": 400, "v": 0.4, "ease": "linear"}]);
    let doc = project(
        30,
        &[card(
            "card",
            20,
            100,
            1000,
            json!([{"name": "blur", "radius": 2}, with("amount", keyed)]),
        )],
    );
    let answer = queried(&doc, 250);
    let card = answer["query"]["stack"]
        .as_array()
        .expect("a stack")
        .iter()
        .find(|member| member["id"] == "card")
        .expect("card is present");
    assert_eq!(
        card["grain"],
        json!([{"effect": 1, "seed": 7, "amount": 0.25, "size": 2, "mono": true, "frame": 4}])
    );
    let amount = card["values"]
        .as_array()
        .expect("values")
        .iter()
        .find(|value| value["property"] == "effects[1].amount (grain)")
        .expect("the keyed amount is on the list");
    assert_eq!(amount["value"], 0.25);

    let text = montagent_core::text::render(&answer, montagent_core::text::Options::default())
        .expect("the answer renders");
    let row = text
        .lines()
        .find(|line| line.split_whitespace().nth(1) == Some("card"))
        .unwrap_or_else(|| panic!("no row for card:\n{text}"));
    assert!(
        row.contains("grain effects[1] seed 7 size 2 mono amount 0.25 frame 4"),
        "{text}"
    );
}
