//! Trim: a stroke draws a window of its outline, measured in fractions of its length
//! (#761, ADR-0160).

use std::path::PathBuf;

use montagent_core::finding::Class;
use montagent_core::stroke::{Window, window};
use serde_json::{Value, json};

mod common;
use common::{canonical, tempdir, write_project};

// ---------------------------------------------------------------------------
// The resolved window (ADR-0160 §4, §5).
// ---------------------------------------------------------------------------

#[track_caller]
fn part(window: Window, from: f64, to: f64) {
    match window {
        Window::Part { from: a, to: b } => assert!(
            (a - from).abs() < 1e-9 && (b - to).abs() < 1e-9,
            "[{a}, {b}], not [{from}, {to}]"
        ),
        other => panic!("{other:?}, not [{from}, {to}]"),
    }
}

#[test]
fn a_window_inside_the_outline_is_its_start_and_end() {
    part(window(0.2, 0.6, 0.0), 0.2, 0.6);
}

#[test]
fn an_offset_rotates_the_window_across_the_start_point() {
    part(window(0.0, 0.25, 0.9), 0.9, 0.15);
}

#[test]
fn a_start_at_or_past_the_end_is_empty_under_any_offset() {
    assert_eq!(window(0.5, 0.5, 0.0), Window::Empty);
    assert_eq!(window(0.6, 0.4, 0.0), Window::Empty);
    assert_eq!(window(0.6, 0.4, 0.25), Window::Empty);
    assert_eq!(window(1.0, 1.0, 0.0), Window::Empty);
}

#[test]
fn the_whole_outline_is_full_under_any_offset() {
    assert_eq!(window(0.0, 1.0, 0.3), Window::Full);
    assert_eq!(window(0.0, 1.0, -2.75), Window::Full);
}

#[test]
fn an_overshoot_clamps_to_the_ends_of_the_outline() {
    part(window(-0.1, 0.4, 0.0), 0.0, 0.4);
    part(window(0.3, 1.087, 0.0), 0.3, 1.0);
    assert_eq!(window(-0.05, 1.2, 0.0), Window::Full);
}

#[test]
fn an_offset_wraps_by_whole_turns_in_either_direction() {
    part(window(0.3, 0.9, 1.5), 0.8, 0.4);
    part(window(0.3, 0.9, -0.5), 0.8, 0.4);
    part(window(0.1, 0.6, 3.0), 0.1, 0.6);
}

#[test]
fn a_window_that_lands_on_the_start_point_starts_or_ends_there_without_crossing() {
    // 0.1 + (2.9 mod 1) is 0.9999999999999999 in f64.
    part(window(0.1, 0.5, 2.9), 0.0, 0.4);
    // A window ending exactly on the start point ends there.
    part(window(0.5, 0.75, 0.25), 0.75, 1.0);
}

// ---------------------------------------------------------------------------
// Fixtures.
// ---------------------------------------------------------------------------

/// `base` with `fields` merged in; a `null` removes the key.
fn with(mut element: Value, fields: Value) -> Value {
    for (key, value) in fields.as_object().expect("fields are an object") {
        if value.is_null() {
            element.as_object_mut().unwrap().remove(key);
        } else {
            element[key] = value.clone();
        }
    }
    element
}

/// An open, stroked path in a 200×200 box at the frame's origin: a line along y 100.
fn path(fields: Value) -> Value {
    with(
        json!({"id": "line", "type": "path", "start": 0, "end": 1000, "x": 0, "y": 0,
            "origin": "top-left", "width": 200, "height": 200, "closed": false,
            "stroke": "#FFFFFF", "stroke_width": 4,
            "points": [{"at": [20, 100]}, {"at": [180, 100]}]}),
        fields,
    )
}

/// A closed, stroked square path: (40, 40) to (160, 160), clockwise from the top-left.
fn square(fields: Value) -> Value {
    with(
        path(json!({"id": "square", "closed": true,
            "points": [{"at": [40, 40]}, {"at": [160, 40]}, {"at": [160, 160]}, {"at": [40, 160]}]})),
        fields,
    )
}

/// A stroked `rect` or `ellipse` filling the 200×200 frame.
fn shape(kind: &str, fields: Value) -> Value {
    with(
        json!({"id": "box", "type": kind, "start": 0, "end": 1000, "x": 0, "y": 0,
            "origin": "top-left", "width": 200, "height": 200,
            "stroke": "#FFFFFF", "stroke_width": 4}),
        fields,
    )
}

fn text(fields: Value) -> Value {
    with(
        json!({"id": "words", "type": "text", "start": 0, "end": 1000, "width": 100,
            "height": 100, "font": "Inter", "size": 20, "runs": [{"text": "Hi"}],
            "stroke": "#FFFFFF", "stroke_width": 2}),
        fields,
    )
}

/// A keyframe list from `from` at 0 ms to `to` at 1000 ms, linear.
fn keyed(from: f64, to: f64) -> Value {
    json!([{"t": 0, "v": from}, {"t": 1000, "v": to, "ease": "linear"}])
}

/// A scratch directory of its own for every call, so tests running in parallel never share
/// one.
fn scratch() -> PathBuf {
    use std::sync::atomic::{AtomicU32, Ordering};
    static NEXT: AtomicU32 = AtomicU32::new(0);
    tempdir(2_300_000 + NEXT.fetch_add(1, Ordering::Relaxed))
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

// ---------------------------------------------------------------------------
// The schema (ADR-0160 §2, §7).
// ---------------------------------------------------------------------------

#[test]
fn a_path_a_rect_and_an_ellipse_take_the_three_trim_fields() {
    assert_eq!(errors(path(json!({"trim_end": 0.5}))), []);
    assert_eq!(
        errors(path(
            json!({"trim_start": 0.1, "trim_end": keyed(0.0, 1.0)})
        )),
        []
    );
    let loader = json!({"trim_start": 0, "trim_end": 0.25, "trim_offset": keyed(0.0, 3.0)});
    assert_eq!(errors(shape("rect", loader.clone())), []);
    assert_eq!(
        errors(shape("rect", with(loader.clone(), json!({"radius": 30})))),
        []
    );
    assert_eq!(errors(shape("ellipse", loader.clone())), []);
    assert_eq!(errors(square(loader)), []);
}

#[test]
fn trim_start_and_trim_end_run_from_zero_to_one_in_every_keyframe_value() {
    for field in ["trim_start", "trim_end"] {
        for value in [json!(1.2), json!(-0.1)] {
            schema_error(path(json!({field: value})), "from 0 to 1");
        }
        schema_error(path(json!({field: keyed(0.0, 1.5)})), "from 0 to 1");
        schema_error(path(json!({field: keyed(-0.25, 1.0)})), "from 0 to 1");
        schema_error(path(json!({field: "half"})), "expected f64");
    }
    assert_eq!(errors(path(json!({"trim_start": 0, "trim_end": 1}))), []);
}

#[test]
fn trim_offset_is_any_number_in_turns() {
    for offset in [json!(-2.75), json!(0), json!(0.5), json!(17)] {
        let element = square(json!({"trim_end": 0.5, "trim_offset": offset}));
        assert_eq!(errors(element), [], "{offset}");
    }
}

#[test]
fn text_takes_no_trim_field() {
    for field in ["trim_start", "trim_end", "trim_offset"] {
        let errors = errors(text(json!({field: 0.5})));
        assert!(
            errors
                .iter()
                .any(|(code, fields)| code.starts_with("E-SCHEMA")
                    && fields.to_string().contains(field)),
            "{field}: {errors:#?}"
        );
    }
}

#[test]
fn the_three_fields_are_animatable_properties_of_every_trimmed_shape_and_not_of_text() {
    use montagent_core::animatable::{Kind, of};
    for kind in ["path", "rect", "ellipse"] {
        let find = |name: &str| {
            of(kind)
                .iter()
                .find(|property| property.name == name)
                .map(|property| (property.kind, property.minimum, property.maximum))
        };
        let bounded = Some((Kind::Number, Some(0.0), Some(1.0)));
        assert_eq!(find("trim_start"), bounded, "{kind}");
        assert_eq!(find("trim_end"), bounded, "{kind}");
        assert_eq!(
            find("trim_offset"),
            Some((Kind::Number, None, None)),
            "{kind}"
        );
    }
    assert!(
        of("text")
            .iter()
            .all(|property| !property.name.starts_with("trim_"))
    );
}

// ---------------------------------------------------------------------------
// `validate` (ADR-0160 §5–§7).
// ---------------------------------------------------------------------------

#[test]
fn a_static_window_that_never_draws_is_trim_empty_at_the_field_quoting_both_values() {
    for (fields, at, says) in [
        (
            json!({"trim_start": 1}),
            "trim_start",
            "`trim_start` is 1 and `trim_end` is 1 (the default)",
        ),
        (
            json!({"trim_end": 0}),
            "trim_end",
            "`trim_start` is 0 (the default) and `trim_end` is 0",
        ),
        (
            json!({"trim_start": 0.6, "trim_end": 0.4}),
            "trim_start",
            "`trim_start` is 0.6 and `trim_end` is 0.4",
        ),
        (
            json!({"trim_start": 0.5, "trim_end": 0.5}),
            "trim_start",
            "`trim_start` is 0.5 and `trim_end` is 0.5",
        ),
    ] {
        for element in [path(fields.clone()), shape("ellipse", fields.clone())] {
            let found = fired(element.clone(), "E-TRIM-EMPTY");
            assert_eq!(found.len(), 1, "{element}: {found:?}");
            let id = element["id"].as_str().unwrap();
            let text = prose(element.clone());
            assert!(text.contains(&format!("`{id}`.{at}:")), "{text}");
            assert!(text.contains(says), "{text}");
        }
    }
}

#[test]
fn a_window_that_draws_or_is_keyed_is_not_trim_empty() {
    for fields in [
        json!({"trim_start": 0.2, "trim_end": 0.3}),
        json!({"trim_start": 0}),
        json!({"trim_end": 1}),
        // A keyed crossing is defined, never checked: a draw-off is one too.
        json!({"trim_start": keyed(0.0, 0.7), "trim_end": keyed(1.0, 0.3)}),
        json!({"trim_start": 1, "trim_end": keyed(0.0, 1.0)}),
    ] {
        let element = path(fields.clone());
        assert_eq!(
            fired(element, "E-TRIM-EMPTY"),
            Vec::<Value>::new(),
            "{fields}"
        );
    }
}

#[test]
fn an_offset_on_an_open_path_is_trim_offset_saying_it_is_open() {
    let element = path(json!({"trim_end": 0.5, "trim_offset": 0.25}));
    let found = fired(element.clone(), "E-TRIM-OFFSET");
    assert_eq!(found.len(), 1, "{found:?}");
    let text = prose(element);
    assert!(text.contains("`line`.trim_offset:"), "{text}");
    assert!(text.contains("the path is open"), "{text}");
    assert!(!text.contains("no `trim_start` or `trim_end`"), "{text}");
}

#[test]
fn an_offset_with_no_window_is_trim_offset_saying_there_is_nothing_to_rotate() {
    for element in [
        shape("rect", json!({"trim_offset": keyed(0.0, 2.0)})),
        shape("ellipse", json!({"trim_offset": 0.5})),
        square(json!({"trim_offset": 0.5})),
    ] {
        let found = fired(element.clone(), "E-TRIM-OFFSET");
        assert_eq!(found.len(), 1, "{element}: {found:?}");
        let id = element["id"].as_str().unwrap();
        let text = prose(element.clone());
        assert!(text.contains(&format!("`{id}`.trim_offset:")), "{text}");
        assert!(text.contains("no `trim_start` or `trim_end`"), "{text}");
        assert!(!text.contains("the path is open"), "{text}");
    }
    // Both at once: one finding, saying both.
    let element = path(json!({"trim_offset": 0.5}));
    assert_eq!(fired(element.clone(), "E-TRIM-OFFSET").len(), 1);
    let text = prose(element);
    assert!(text.contains("the path is open"), "{text}");
    assert!(text.contains("no `trim_start` or `trim_end`"), "{text}");
}

#[test]
fn an_offset_rotating_a_window_on_a_closed_outline_is_fine() {
    for element in [
        shape("rect", json!({"trim_end": 0.25, "trim_offset": 0.5})),
        shape("ellipse", json!({"trim_start": 0.5, "trim_offset": -3})),
        square(json!({"trim_end": 0.25, "trim_offset": keyed(0.0, 3.0)})),
    ] {
        assert_eq!(errors(element.clone()), [], "{element}");
    }
}

#[test]
fn a_trim_field_with_no_stroke_to_shape_is_an_error_at_each_field() {
    let trimmed = |element: Value| {
        with(
            element,
            json!({"trim_start": 0.1, "trim_end": 0.6, "trim_offset": 0.5}),
        )
    };
    for element in [
        trimmed(square(json!({"stroke": null, "fill": "#FF0000"}))),
        trimmed(shape("rect", json!({"stroke_width": 0}))),
        trimmed(shape(
            "ellipse",
            json!({"stroke_width": null, "fill": "#FF0000"}),
        )),
    ] {
        let found = fired(element.clone(), "E-STROKE-NO-STROKE");
        let named: Vec<&Value> = found.iter().map(|fields| &fields["field"]).collect();
        assert_eq!(
            named,
            [
                &json!("trim_start"),
                &json!("trim_end"),
                &json!("trim_offset")
            ],
            "{element}"
        );
        let id = element["id"].as_str().unwrap();
        assert!(prose(element.clone()).contains(&format!("`{id}`.trim_end:")));
    }
}

#[test]
fn a_cap_on_a_closed_path_draws_once_it_is_trimmed() {
    for cap in ["butt", "round", "square"] {
        for window in [json!({"trim_end": 0.5}), json!({"trim_start": 0.25})] {
            let element = square(with(window, json!({"stroke_cap": cap})));
            assert_eq!(errors(element.clone()), [], "{element}");
        }
    }
    // An offset alone makes no window, so the cap still draws nowhere.
    let element = square(json!({"stroke_cap": "round", "trim_offset": 0.5}));
    assert_eq!(fired(element.clone(), "E-STROKE-CAP-UNDRAWN").len(), 1);
    let text = prose(square(json!({"stroke_cap": "round"})));
    assert!(text.contains("`trim_start`"), "{text}");
}

#[test]
fn a_square_cap_widens_the_inset_on_a_trimmed_closed_path() {
    let reach = |fields: Value| {
        montagent_core::stroke::reach(&square(with(
            json!({"stroke_cap": "square", "stroke_width": 8}),
            fields,
        )))
    };
    for window in [
        json!({"trim_end": 0.5}),
        json!({"trim_start": 0}),
        json!({"trim_end": 1}),
    ] {
        let reach = reach(window.clone());
        assert_eq!(reach.inset, 6, "{window}");
        assert_eq!(
            reach.source,
            montagent_core::stroke::Source::SquareCap,
            "{window}"
        );
    }
    // A vertex 5 px in from the box's edge is inside ceil(8 / 2) = 4 and outside
    // ceil(√2 × 8 / 2) = 6.
    let element = square(
        json!({"stroke_cap": "square", "stroke_width": 8, "trim_end": 0.5,
        "points": [{"at": [5, 40]}, {"at": [160, 40]}, {"at": [160, 160]}, {"at": [40, 160]}]}),
    );
    let found = fired(element.clone(), "E-PATH-OUTSIDE-BOX");
    assert_eq!(found.len(), 1, "{found:?}");
    let text = prose(element);
    assert!(text.contains("`square`.points"), "{text}");
    assert!(text.contains("from `stroke_cap` \"square\""), "{text}");
}

// ---------------------------------------------------------------------------
// The painter (ADR-0160 §1, §3–§6).
// ---------------------------------------------------------------------------

const BLACK: [u8; 3] = [0, 0, 0];
const WHITE: [u8; 3] = [255, 255, 255];

/// The frame at `t` ms of a project holding `elements`, as RGB, 200 pixels wide.
fn painted_at(elements: &[Value], t: i64) -> Vec<u8> {
    use montagent_core::verbs::render::{Supplying, paint_span};
    let rasters = paint_span(&project(elements), t, t + 1, Supplying::PerFrame).expect("it paints");
    assert!(rasters.declined.is_empty(), "{:?}", rasters.declined);
    rasters.frames.into_iter().next().unwrap()
}

fn painted(element: Value) -> Vec<u8> {
    painted_at(&[element], 0)
}

fn pixel(rgb: &[u8], x: usize, y: usize) -> [u8; 3] {
    let at = (y * 200 + x) * 3;
    [rgb[at], rgb[at + 1], rgb[at + 2]]
}

/// Whether two frames agree on every pixel of the square `[x0, x1) × [y0, y1)`.
fn same_in(one: &[u8], other: &[u8], (x0, x1): (usize, usize), (y0, y1): (usize, usize)) -> bool {
    (y0..y1).all(|y| (x0..x1).all(|x| pixel(one, x, y) == pixel(other, x, y)))
}

/// Every pixel of the frame that is not black.
fn lit(rgb: &[u8]) -> usize {
    rgb.chunks(3).filter(|pixel| *pixel != BLACK).count()
}

#[test]
fn a_draw_on_draws_the_line_from_its_start_to_trim_end() {
    // The line runs x 20..180 along y 100: half of it ends at x 100.
    let half = painted(path(json!({"trim_end": 0.5})));
    assert_eq!(pixel(&half, 60, 100), WHITE);
    assert_eq!(pixel(&half, 98, 100), WHITE);
    assert_eq!(pixel(&half, 103, 100), BLACK, "a butt end");
    assert_eq!(pixel(&half, 140, 100), BLACK);
    let rest = painted(path(json!({"trim_start": 0.5})));
    assert_eq!(pixel(&rest, 60, 100), BLACK);
    assert_eq!(pixel(&rest, 140, 100), WHITE);
}

#[test]
fn an_empty_window_under_a_round_cap_paints_no_pixel() {
    // A draw-on's first frame: `trim_end` 0, so the window is [0, 0].
    let draw_on = path(json!({"stroke_cap": "round", "stroke_width": 12,
        "trim_end": keyed(0.0, 1.0)}));
    assert_eq!(lit(&painted(draw_on.clone())), 0);
    assert!(lit(&painted_at(&[draw_on], 500)) > 0);
    // Keyed start and end that cross: at 500 ms start is 0.5 and end 0.45.
    let crossed = path(json!({"stroke_cap": "round", "stroke_width": 12,
        "trim_start": keyed(0.0, 1.0), "trim_end": keyed(0.9, 0.0)}));
    assert_eq!(lit(&painted_at(&[crossed], 500)), 0);
}

#[test]
fn a_trim_end_takes_the_cap_on_a_path_and_is_butt_on_a_shape() {
    // A round end reaches 6 px past x 100 under width 12.
    let round = painted(path(json!({"stroke_cap": "round", "stroke_width": 12,
        "trim_end": 0.5})));
    assert_eq!(pixel(&round, 104, 100), WHITE, "the round cap");
    assert_eq!(pixel(&round, 108, 100), BLACK);
    let butt = painted(path(json!({"stroke_width": 12, "trim_end": 0.5})));
    assert_eq!(pixel(&butt, 103, 100), BLACK);
    // A rect's window from its inset top-left corner, 40 px along the top edge (196 × 4).
    let rect = painted(shape("rect", json!({"trim_end": 40.0 / 784.0})));
    assert_eq!(pixel(&rect, 20, 1), WHITE, "along the top edge");
    assert_eq!(pixel(&rect, 40, 1), WHITE);
    assert_eq!(pixel(&rect, 44, 1), BLACK, "a butt end");
    assert_eq!(pixel(&rect, 1, 20), BLACK, "not back down the left edge");
}

#[test]
fn the_window_runs_from_each_outlines_dash_start_in_its_direction() {
    // An ellipse leaves 3 o'clock downward.
    let ellipse = painted(shape("ellipse", json!({"trim_end": 0.05})));
    assert_eq!(pixel(&ellipse, 197, 115), WHITE, "below 3 o'clock");
    assert_eq!(pixel(&ellipse, 197, 85), BLACK, "not above it");
    // A rounded rect starts where the top-left arc meets the top edge: x 30.
    let rounded = painted(shape("rect", json!({"radius": 30, "trim_end": 0.05})));
    assert_eq!(pixel(&rounded, 34, 1), WHITE, "just past the start");
    assert_eq!(pixel(&rounded, 25, 2), BLACK, "not on the arc before it");
    // A closed path starts at `points[0]` and runs in points order; 480 px around.
    let square = painted(square(json!({"trim_end": 0.1})));
    assert_eq!(pixel(&square, 60, 40), WHITE, "along the first segment");
    assert_eq!(
        pixel(&square, 100, 40),
        BLACK,
        "past the window's end, 48 px on"
    );
    assert_eq!(
        pixel(&square, 40, 60),
        BLACK,
        "not back along the closing segment"
    );
}

#[test]
fn an_offset_rotates_the_window_forward_and_whole_turns_paint_the_same_bytes() {
    // A quarter turn on the square moves a 0.1 window from the top edge to the right edge.
    let turned = painted(square(json!({"trim_end": 0.1, "trim_offset": 0.25})));
    assert_eq!(pixel(&turned, 160, 60), WHITE, "down the right edge");
    assert_eq!(pixel(&turned, 60, 40), BLACK, "off the top edge");
    for element in [
        square(json!({"trim_start": 0.1, "trim_end": 0.3})),
        shape("ellipse", json!({"trim_end": 0.3})),
        shape("rect", json!({"trim_end": 0.3, "radius": 20})),
    ] {
        let at = |offset: f64| painted(with(element.clone(), json!({"trim_offset": offset})));
        assert!(at(0.4) == at(2.4), "{element}");
        assert!(at(0.4) == at(-1.6), "{element}");
        assert!(at(0.0) != at(0.4), "{element}");
    }
}

#[test]
fn a_full_window_paints_the_bytes_of_no_trim() {
    let full = json!({"trim_start": 0, "trim_end": 1, "trim_offset": 0.37});
    for element in [
        shape("rect", json!({})),
        shape("rect", json!({"radius": 30})),
        shape("ellipse", json!({})),
        shape(
            "rect",
            json!({"stroke_dash": [12, 6], "stroke_dash_offset": 3}),
        ),
        square(json!({"stroke_join": "miter", "stroke_miter_limit": 4})),
        square(json!({"stroke_dash": [20, 8], "stroke_cap": "round"})),
        path(json!({"stroke_cap": "round"})),
    ] {
        let mut trimmed = with(element.clone(), full.clone());
        if element["closed"] == json!(false) {
            trimmed.as_object_mut().unwrap().remove("trim_offset");
        }
        assert!(painted(element.clone()) == painted(trimmed), "{element}");
    }
    // Overshoot clamps to the full window too.
    let element = shape("ellipse", json!({}));
    // `[0.34, 1.56, 0.64, 1]` carries `trim_end` to about 1.097 at 600 ms.
    let overshoot = json!([{"t": 0, "v": 0}, {"t": 1000, "v": 1, "ease": [0.34, 1.56, 0.64, 1]}]);
    let over = with(
        element.clone(),
        json!({"trim_end": overshoot, "trim_offset": 0.5}),
    );
    assert!(painted(element) == painted_at(&[over], 600));
}

#[test]
fn a_window_crossing_the_start_point_is_one_stroke_with_the_outlines_join_there() {
    // The window [0.9, 0.1] crosses the square's start point, its top-left corner, under a
    // miter join: the corner draws exactly as the untrimmed square draws it, with no ends.
    let style = json!({"stroke_join": "miter", "stroke_miter_limit": 4, "stroke_width": 8});
    let whole = painted(square(style.clone()));
    let crossing = painted(square(with(
        style.clone(),
        json!({"trim_end": 0.2, "trim_offset": 0.9}),
    )));
    assert_eq!(pixel(&crossing, 37, 37), WHITE, "the miter's outer corner");
    assert!(same_in(&whole, &crossing, (30, 70), (30, 70)));
    assert_eq!(pixel(&crossing, 100, 40), BLACK, "past the window's end");
    // A rect's start point is its inset top-left corner, which keeps its miter.
    let rect = shape("rect", json!({"stroke_width": 8}));
    let crossing = painted(with(
        rect.clone(),
        json!({"trim_end": 0.2, "trim_offset": 0.9}),
    ));
    assert!(same_in(&painted(rect), &crossing, (0, 40), (0, 40)));
}

#[test]
fn the_fill_is_never_trimmed() {
    let element = square(json!({"fill": "#FF0000", "stroke_width": 8, "trim_end": 0.25}));
    let rgb = painted(element);
    assert_eq!(pixel(&rgb, 100, 100), [255, 0, 0], "the fill, whole");
    assert_eq!(
        pixel(&rgb, 100, 162),
        BLACK,
        "the bottom edge is not stroked"
    );
    assert_eq!(pixel(&rgb, 100, 157), [255, 0, 0], "the fill reaches it");
    assert_eq!(pixel(&rgb, 100, 40), WHITE, "the top edge is");
}

#[test]
fn dashes_stay_where_they_sit_untrimmed_and_the_window_reveals_them() {
    // Dashes of 20 every 30 px from x 20; the window ends at 0.6 of 160, x 116, inside the
    // dash x 110..130.
    let dashed = json!({"stroke_dash": [20, 10]});
    let whole = painted(path(dashed.clone()));
    let trimmed = painted(path(with(dashed.clone(), json!({"trim_end": 0.6}))));
    assert!(same_in(&whole, &trimmed, (0, 114), (90, 110)));
    assert_eq!(
        pixel(&trimmed, 112, 100),
        WHITE,
        "the dash the window ends inside"
    );
    assert_eq!(pixel(&trimmed, 120, 100), BLACK, "cut at the window's end");
    assert_eq!(pixel(&trimmed, 145, 100), BLACK);
    // A window from 0.3 (x 68, inside the dash x 50..70) keeps the later dashes where they
    // were: they are not re-laid from the window's start.
    let late = painted(path(with(dashed, json!({"trim_start": 0.3}))));
    assert_eq!(pixel(&late, 60, 100), BLACK, "before the window");
    assert_ne!(pixel(&late, 69, 100), BLACK, "the cut dash's tail");
    assert!(same_in(&whole, &late, (72, 200), (90, 110)));
}

#[test]
fn dashes_on_a_curve_inside_the_window_are_the_untrimmed_bytes() {
    // The window [0.5, 1] of an ellipse is its top half, from 9 o'clock round to 3; the
    // dashes there are cut at the very distances the untrimmed pattern is, so they do not
    // drift by a fraction of a pixel.
    let dashed = json!({"stroke_dash": [20, 10], "stroke_dash_offset": 7});
    let whole = painted(shape("ellipse", dashed.clone()));
    let trimmed = painted(shape("ellipse", with(dashed, json!({"trim_start": 0.5}))));
    assert!(same_in(&whole, &trimmed, (0, 200), (0, 90)));
    assert!(lit(&trimmed) < lit(&whole));
}

#[test]
fn dots_inside_the_window_draw_as_they_do_untrimmed() {
    // Round dots every 20 px from x 20; the window [0.25, 0.75] runs x 60..140.
    let dots = json!({"stroke_width": 6, "stroke_cap": "round", "stroke_dash": [0, 20]});
    let whole = painted(path(dots.clone()));
    let trimmed = painted(path(with(
        dots,
        json!({"trim_start": 0.25, "trim_end": 0.75}),
    )));
    assert!(same_in(&whole, &trimmed, (55, 145), (90, 110)));
    for x in [80, 100, 120] {
        assert_eq!(pixel(&trimmed, x, 100), WHITE, "a dot at x {x}");
    }
    for x in [20, 40, 160] {
        assert_eq!(pixel(&trimmed, x, 100), BLACK, "no dot at x {x}");
    }
}

/// **ADR-0160 §4 against §6, decided by the spec's "dash first, then cut the window".**
/// A pattern "on" at a closed outline's start point draws, untrimmed, as one dash through
/// it with the outline's join. Dashing first and cutting the window after keeps that dash
/// whole where a window crossing the start point covers it, so the crossing is one
/// continuous stroke (§4) and every dash sits where it sits untrimmed (§6).
#[test]
fn a_dash_on_at_the_start_point_stays_one_dash_through_a_window_crossing_it() {
    // 480 px around; `[50, 10]` with offset 25 leaves the dash 455..480 on at the start point
    // and joins it to the first, 0..25, round the miter corner at (40, 40).
    let style = json!({"stroke_join": "miter", "stroke_miter_limit": 4, "stroke_width": 8,
        "stroke_cap": "butt", "stroke_dash": [50, 10], "stroke_dash_offset": 25});
    let whole = painted(square(style.clone()));
    let crossing = painted(square(with(
        style,
        json!({"trim_end": 0.2, "trim_offset": 0.9}),
    )));
    assert_eq!(
        pixel(&whole, 37, 37),
        WHITE,
        "untrimmed, the dash turns the corner"
    );
    assert!(same_in(&whole, &crossing, (30, 70), (30, 75)));
}

#[test]
fn a_trimmed_shape_never_inks_outside_its_box() {
    // A 120×120 box at (40, 40), a window orbiting twice over the element's second.
    let window = json!({"trim_start": keyed(0.1, 0.3), "trim_end": keyed(0.35, 0.45),
        "trim_offset": keyed(0.0, 2.0), "stroke_width": 9});
    let place = with(
        window,
        json!({"x": 40, "y": 40, "width": 120, "height": 120}),
    );
    for element in [
        shape("rect", place.clone()),
        shape("rect", with(place.clone(), json!({"radius": 30}))),
        shape("ellipse", place.clone()),
        square(with(
            place.clone(),
            json!({"stroke_cap": "square", "stroke_join": "bevel",
                "points": [{"at": [7, 60]}, {"at": [60, 7]}, {"at": [113, 60]}, {"at": [60, 113]}]}),
        )),
    ] {
        assert_eq!(errors(element.clone()), [], "{element}");
        for t in (0..1000).step_by(100) {
            let rgb = painted_at(std::slice::from_ref(&element), t);
            for y in 0..200 {
                for x in 0..200 {
                    let inside = (40..160).contains(&x) && (40..160).contains(&y);
                    assert!(
                        pixel(&rgb, x, y) == BLACK || inside,
                        "{} at {t} ms inks ({x}, {y})",
                        element["id"]
                    );
                }
            }
            assert!(lit(&rgb) > 0, "{} at {t} ms", element["id"]);
        }
    }
}

// ---------------------------------------------------------------------------
// `query --at` (ADR-0160 §7).
// ---------------------------------------------------------------------------

/// `query --at`'s answer at `t` for the one element of `element`'s project, as JSON.
fn queried_at(element: Value, t: i64) -> Value {
    let document = montagent_core::parse::read(&project(&[element])).unwrap();
    let answer = serde_json::to_value(montagent_core::verbs::query::at::at(&document, t, None));
    answer.unwrap()["stack"][0].clone()
}

/// `query --at`'s prose at `t` for `element`'s project.
fn queried_text(element: Value, t: i64) -> String {
    use montagent_core::verbs::query::{Ask, query};
    use montagent_core::wire::{Wire, render_query};
    let ask = Ask {
        at: Some(t),
        ..Ask::default()
    };
    render_query(
        &query(&project(&[element]), &ask),
        Wire::Text { verbose: false },
    )
}

/// The resolved value `query --at` lists for `property`.
fn value_of(present: &Value, property: &str) -> Value {
    present["values"]
        .as_array()
        .unwrap()
        .iter()
        .find(|value| value["property"] == property)
        .map(|value| value["value"].clone())
        .unwrap_or(Value::Null)
}

#[test]
fn query_at_prints_drawn_empty_at_a_keyed_crossing() {
    let element = path(json!({"trim_start": keyed(0.0, 1.0), "trim_end": keyed(0.9, 0.0)}));
    let present = queried_at(element.clone(), 500);
    assert_eq!(present["trim"]["drawn"], json!("empty"), "{present}");
    assert_eq!(present["trim"]["trim_start"], json!(0.5));
    assert_eq!(present["trim"]["trim_end"], json!(0.45));
    assert_eq!(present["trim"]["trim_offset"], json!(0.0), "the default");
    assert!(queried_text(element, 500).contains("drawn: empty"));
}

#[test]
fn query_at_prints_drawn_full_under_an_offset() {
    let element = shape(
        "rect",
        json!({"trim_start": 0, "trim_end": 1, "trim_offset": 0.37}),
    );
    let present = queried_at(element.clone(), 0);
    assert_eq!(present["trim"]["drawn"], json!("full"), "{present}");
    assert!(queried_text(element, 0).contains("drawn: full"));
}

#[test]
fn query_at_prints_a_crossing_window_and_the_offset_raw_past_one_turn() {
    let element = square(json!({"trim_start": 0.3, "trim_end": 0.9, "trim_offset": 1.5}));
    let present = queried_at(element.clone(), 0);
    assert_eq!(present["trim"]["drawn"], json!("[0.8, 0.4]"), "{present}");
    assert_eq!(present["trim"]["trim_offset"], json!(1.5));
    assert_eq!(value_of(&present, "trim_offset"), json!(1.5));
    let text = queried_text(element, 0);
    assert!(text.contains("drawn: [0.8, 0.4]"), "{text}");
    assert!(text.contains("trim_offset 1.5"), "{text}");
    // Keyed past several turns, it is the resolved value, unwrapped.
    let loader = shape(
        "ellipse",
        json!({"trim_end": 0.25, "trim_offset": keyed(0.0, 3.0)}),
    );
    let present = queried_at(loader, 500);
    assert_eq!(present["trim"]["trim_offset"], json!(1.5));
    assert_eq!(present["trim"]["drawn"], json!("[0.5, 0.75]"), "{present}");
}

#[test]
fn query_at_prints_an_overshoot_raw_and_the_window_clamped() {
    let overshoot = json!([{"t": 0, "v": 0}, {"t": 1000, "v": 1, "ease": [0.34, 1.56, 0.64, 1]}]);
    let present = queried_at(shape("rect", json!({"trim_end": overshoot})), 600);
    let raw = present["trim"]["trim_end"].as_f64().unwrap();
    assert!(raw > 1.05, "{present}");
    assert_eq!(
        value_of(&present, "trim_end"),
        json!(raw),
        "the row is raw too"
    );
    assert_eq!(present["trim"]["drawn"], json!("full"), "{present}");
    // On an open path the whole line is `[0, 1]`: `full` is a closed outline with no ends.
    let line = path(json!({"trim_start": 0, "trim_end": 1}));
    assert_eq!(queried_at(line, 0)["trim"]["drawn"], json!("[0, 1]"));
}

// ---------------------------------------------------------------------------
// `shift` (ADR-0160 §7, under ADR-0146).
// ---------------------------------------------------------------------------

fn shifted(path: &std::path::Path, at: i64) -> montagent_core::verbs::shift::Answer {
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
fn shift_refuses_a_cut_where_an_overshoot_carries_trim_end_past_one() {
    let overshoot = json!([{"t": 0, "v": 0}, {"t": 1000, "v": 1, "ease": [0.34, 1.56, 0.64, 1]}]);
    let path = project(&[shape("rect", json!({"trim_end": overshoot}))]);
    let before = std::fs::read_to_string(&path).unwrap();
    let answer = shifted(&path, 600);
    let refusals: Vec<_> = answer
        .report()
        .findings
        .iter()
        .filter(|finding| finding.code == "E-SHIFT-SPLIT-UNWRITABLE")
        .collect();
    assert_eq!(refusals.len(), 1, "{:?}", answer.report().findings);
    assert_eq!(refusals[0].fields["property"], "trim_end");
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        before,
        "nothing is written"
    );
}

#[test]
fn shift_cuts_a_loader_whose_offset_is_past_one_turn() {
    let path = project(&[shape(
        "ellipse",
        json!({"trim_end": 0.25, "trim_offset": keyed(0.0, 3.0)}),
    )]);
    let answer = shifted(&path, 500);
    assert_eq!(
        answer.report().exit_code(),
        montagent_core::report::ExitCode::Ok,
        "{:?}",
        answer.report().findings
    );
    let written = common::document(&path);
    let records: Vec<(i64, f64)> = written["tracks"][0]["elements"][0]["trim_offset"]
        .as_array()
        .unwrap()
        .iter()
        .map(|record| (record["t"].as_i64().unwrap(), record["v"].as_f64().unwrap()))
        .collect();
    assert_eq!(records, [(0, 0.0), (500, 1.5), (600, 1.5), (1100, 3.0)]);
}

#[test]
fn query_at_says_nothing_of_trim_on_an_untrimmed_element() {
    for element in [
        path(json!({})),
        shape("rect", json!({"stroke_dash": [6, 4]})),
        shape("ellipse", json!({})),
    ] {
        let present = queried_at(element.clone(), 0);
        assert!(present.get("trim").is_none(), "{present}");
        assert!(!queried_text(element, 0).contains("drawn:"));
    }
}

// ---------------------------------------------------------------------------
// The motion recipes.
// ---------------------------------------------------------------------------

/// The one JSON project in `montagent-motion`'s recipe headed `heading`.
fn recipe(heading: &str) -> String {
    let skill = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../skills/montagent-motion/SKILL.md"),
    )
    .unwrap();
    let start = skill.find(heading).expect("the recipe is in the skill");
    let recipe = &skill[start..];
    let recipe = &recipe[..recipe[4..]
        .find("\n### ")
        .map_or(recipe.len(), |end| end + 4)];
    let body = &recipe[recipe
        .find("```json\n")
        .expect("the recipe has a JSON project")
        + 8..];
    body[..body.find("```").unwrap()].to_string()
}

/// The recipe's project, validated with exactly the findings `expected`, and its frame at
/// each of `times` as RGB, 1920 pixels wide.
fn recipe_frames(heading: &str, expected: &[&str], times: &[i64]) -> Vec<Vec<u8>> {
    let path = write_project(
        &scratch(),
        "recipe.montagent.json",
        &canonical(&recipe(heading)),
    );
    let report = montagent_core::validate(&path);
    let codes: Vec<&str> = report.findings.iter().map(|f| f.code.as_str()).collect();
    assert_eq!(codes, expected, "{heading}");
    times
        .iter()
        .map(|&t| {
            use montagent_core::verbs::render::{Supplying, paint_span};
            let rasters = paint_span(&path, t, t + 1, Supplying::PerFrame).expect("it paints");
            assert!(rasters.declined.is_empty(), "{:?}", rasters.declined);
            rasters.frames.into_iter().next().unwrap()
        })
        .collect()
}

fn wide(rgb: &[u8], x: usize, y: usize) -> [u8; 3] {
    let at = (y * 1920 + x) * 3;
    [rgb[at], rgb[at + 1], rgb[at + 2]]
}

#[test]
fn the_draw_on_recipe_validates_and_draws_nothing_then_the_whole_line() {
    let frames = recipe_frames("### Draw a line on", &[], &[0, 1500]);
    let ground = wide(&frames[0], 10, 10);
    let inked = |rgb: &[u8]| {
        (0..1080)
            .flat_map(|y| (0..1920).map(move |x| (x, y)))
            .filter(|&(x, y)| wide(rgb, x, y) != ground)
            .count()
    };
    assert_eq!(inked(&frames[0]), 0, "nothing drawn on the first frame");
    // The line's last vertex, (760, 60) in a box centred at (960, 600): (1320, 560).
    assert_eq!(wide(&frames[1], 1318, 560), [0xFF, 0x5A, 0x36]);
}

#[test]
fn the_ring_loader_recipe_validates_and_its_arc_orbits() {
    // At 0 the arc runs from 3 o'clock to 6 o'clock; at frame 8, 266.7 ms on, it has turned
    // 0.267 of a turn, so it runs from just past 6 o'clock to just past 9.
    // The one finding is the loop's join, which the recipe explains.
    let frames = recipe_frames(
        "### Spin a ring loader",
        &["R-KEYFRAME-UNREACHED"],
        &[0, 266],
    );
    let arc = [0xFF, 0x5A, 0x36];
    let (cx, cy, r) = (960.0, 540.0, 73.0_f64);
    let at = |rgb: &[u8], degrees: f64| {
        let radians = degrees.to_radians();
        let (x, y) = (cx + r * radians.cos(), cy + r * radians.sin());
        wide(rgb, x.round() as usize, y.round() as usize)
    };
    assert_eq!(at(&frames[0], 45.0), arc, "between 3 and 6 o'clock");
    assert_ne!(at(&frames[0], 135.0), arc);
    assert_eq!(at(&frames[1], 135.0), arc, "between 6 and 9 o'clock");
    assert_ne!(at(&frames[1], 45.0), arc);
}
