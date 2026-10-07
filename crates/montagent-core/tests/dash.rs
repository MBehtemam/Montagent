//! Dash patterns on `path`, `rect` and `ellipse`, with an animatable offset (#752,
//! ADR-0158 §5–§7).

use std::path::PathBuf;

use montagent_core::finding::Class;
use serde_json::{Value, json};

mod common;
use common::{canonical, tempdir, write_project};

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

/// An open, stroked path in a 200×200 box at the frame's origin.
fn path(fields: Value) -> Value {
    with(
        json!({"id": "line", "type": "path", "start": 0, "end": 1000, "x": 0, "y": 0,
            "origin": "top-left", "width": 200, "height": 200, "closed": false,
            "stroke": "#FFFFFF", "stroke_width": 4,
            "points": [{"at": [20, 100]}, {"at": [180, 100]}]}),
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

/// A scratch directory of its own for every call, so tests running in parallel never share
/// one.
fn scratch() -> PathBuf {
    use std::sync::atomic::{AtomicU32, Ordering};
    static NEXT: AtomicU32 = AtomicU32::new(0);
    tempdir(2_100_000 + NEXT.fetch_add(1, Ordering::Relaxed))
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

// ---------------------------------------------------------------------------
// The schema.
// ---------------------------------------------------------------------------

#[test]
fn a_path_a_rect_and_an_ellipse_take_a_dash_and_an_offset() {
    let dashed = json!({"stroke_dash": [12, 6], "stroke_dash_offset": -3});
    assert_eq!(errors(path(dashed.clone())), []);
    assert_eq!(errors(shape("rect", dashed.clone())), []);
    assert_eq!(
        errors(shape("rect", with(dashed.clone(), json!({"radius": 30})))),
        []
    );
    assert_eq!(errors(shape("ellipse", dashed)), []);
}

#[test]
fn text_refuses_the_dash_fields_as_belonging_to_the_shapes() {
    for (key, value) in [
        ("stroke_dash", json!([4, 4])),
        ("stroke_dash_offset", json!(2)),
    ] {
        let errors = errors(text(json!({key: value})));
        let says = format!("`{key}` belongs to `path`, `rect` and `ellipse`");
        assert!(
            errors
                .iter()
                .any(|(code, fields)| code == "E-SCHEMA" && fields.to_string().contains(&says)),
            "{key}: {errors:#?}"
        );
    }
}

#[test]
fn the_dash_list_is_static_and_holds_two_to_sixteen_whole_non_negative_lengths() {
    let keyed = json!([{"t": 0, "v": [4, 4]}, {"t": 500, "v": [8, 8], "ease": "linear"}]);
    schema_error(path(json!({"stroke_dash": keyed})), "static");
    for list in [json!([4]), json!([]), json!(vec![1; 18])] {
        schema_error(path(json!({"stroke_dash": list})), "2 to 16");
    }
    for list in [json!([4, -1]), json!([4, 2.5]), json!([4, "2"]), json!(4)] {
        schema_error(path(json!({"stroke_dash": list})), "stroke_dash");
    }
    for list in [json!([0, 4]), json!(vec![1; 16])] {
        let element = path(json!({"stroke_dash": list, "stroke_cap": "round"}));
        assert_eq!(errors(element), [], "{list}");
    }
}

#[test]
fn the_offset_is_any_integer_and_may_be_keyed() {
    for offset in [json!(-40), json!(0), json!(17)] {
        let element = path(json!({"stroke_dash": [4, 4], "stroke_dash_offset": offset}));
        assert_eq!(errors(element), [], "{offset}");
    }
    let ants = json!([{"t": 0, "v": 0}, {"t": 1000, "v": -80, "ease": "linear"}]);
    let element = shape(
        "rect",
        json!({"stroke_dash": [6, 4], "stroke_dash_offset": ants}),
    );
    assert_eq!(errors(element), []);
    schema_error(
        path(json!({"stroke_dash": [4, 4], "stroke_dash_offset": 2.5})),
        "expected i64",
    );
}

#[test]
fn the_offset_is_an_animatable_property_on_every_dashed_shape_and_not_on_text() {
    for kind in ["path", "rect", "ellipse"] {
        let property = montagent_core::animatable::of(kind)
            .iter()
            .find(|property| property.name == "stroke_dash_offset")
            .map(|property| property.kind);
        assert_eq!(
            property,
            Some(montagent_core::animatable::Kind::Integer),
            "{kind}"
        );
        assert!(
            montagent_core::animatable::of(kind)
                .iter()
                .all(|property| property.name != "stroke_dash"),
            "the list is static on {kind}"
        );
    }
    assert!(
        montagent_core::animatable::of("text")
            .iter()
            .all(|property| property.name != "stroke_dash_offset")
    );
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
fn an_odd_list_is_a_dash_shape_error_that_says_it_is_odd() {
    for element in [
        path(json!({"stroke_dash": [6, 4, 2]})),
        shape("rect", json!({"stroke_dash": [6, 4, 2]})),
        shape("ellipse", json!({"stroke_dash": [6, 4, 2]})),
    ] {
        let found = fired(element.clone(), "E-DASH-SHAPE");
        assert_eq!(found.len(), 1, "{element}: {found:?}");
        let text = prose(element.clone());
        let id = element["id"].as_str().unwrap();
        assert!(
            text.contains(&format!(
                "`{id}`.stroke_dash: it has 3 entries, an odd number"
            )),
            "{text}"
        );
        assert!(text.contains("does not repeat an odd list"), "{text}");
    }
}

#[test]
fn a_zero_total_is_a_dash_shape_error_that_says_it_adds_up_to_zero() {
    let element = path(json!({"stroke_dash": [0, 0], "stroke_cap": "round"}));
    let found = fired(element.clone(), "E-DASH-SHAPE");
    assert_eq!(found.len(), 1, "{found:?}");
    let text = prose(element);
    assert!(
        text.contains("`line`.stroke_dash: its entries add up to 0"),
        "{text}"
    );
}

#[test]
fn a_zero_dash_under_a_butt_cap_is_an_error_at_that_entry() {
    for cap in [Value::Null, json!("butt")] {
        let element = path(json!({"stroke_dash": [4, 6, 0, 6], "stroke_cap": cap}));
        let found = fired(element.clone(), "E-DASH-ZERO-BUTT");
        assert_eq!(found.len(), 1, "{cap}: {found:?}");
        assert_eq!(found[0]["index"], json!(2));
        let text = prose(element);
        assert!(
            text.contains("`line`.stroke_dash[2]: a zero-length dash"),
            "{text}"
        );
        assert!(text.contains("`\"round\"`"), "{text}");
    }
    for cap in ["round", "square"] {
        let element = path(json!({"stroke_dash": [0, 6], "stroke_cap": cap}));
        assert_eq!(errors(element), [], "{cap} draws a dot");
    }
    // A zero gap is legal.
    assert_eq!(errors(path(json!({"stroke_dash": [6, 0]}))), []);
}

#[test]
fn a_zero_dash_on_a_rect_or_an_ellipse_says_to_use_a_round_capped_path() {
    for kind in ["rect", "ellipse"] {
        let element = shape(kind, json!({"stroke_dash": [0, 8]}));
        let found = fired(element.clone(), "E-DASH-ZERO-BUTT");
        assert_eq!(found.len(), 1, "{kind}: {found:?}");
        let text = prose(element);
        assert!(text.contains("`box`.stroke_dash[0]:"), "{text}");
        assert!(
            text.contains("a `path` with `\"stroke_cap\": \"round\"`"),
            "{text}"
        );
    }
}

#[test]
fn zero_zero_is_both_a_shape_and_a_zero_butt_error() {
    let element = path(json!({"stroke_dash": [0, 0]}));
    assert_eq!(fired(element.clone(), "E-DASH-SHAPE").len(), 1);
    assert_eq!(fired(element, "E-DASH-ZERO-BUTT").len(), 1);
}

#[test]
fn a_dash_field_with_no_stroke_to_shape_is_an_error_at_each_field() {
    let dashed = |element: Value| {
        with(
            element,
            json!({"stroke_dash": [6, 4], "stroke_dash_offset": 3}),
        )
    };
    for element in [
        dashed(path(
            json!({"stroke": null, "closed": true, "fill": "#FF0000"}),
        )),
        dashed(shape("rect", json!({"stroke_width": 0}))),
        dashed(shape("ellipse", json!({"stroke_width": null}))),
    ] {
        let found = fired(element.clone(), "E-STROKE-NO-STROKE");
        let named: Vec<&Value> = found.iter().map(|fields| &fields["field"]).collect();
        assert_eq!(
            named,
            [&json!("stroke_dash"), &json!("stroke_dash_offset")],
            "{element}"
        );
        let id = element["id"].as_str().unwrap();
        assert!(prose(element.clone()).contains(&format!("`{id}`.stroke_dash_offset:")));
    }
}

#[test]
fn an_offset_with_no_dash_is_an_error_at_the_offset() {
    for element in [
        path(json!({"stroke_dash_offset": 4})),
        shape("rect", json!({"stroke_dash_offset": 4})),
        shape("ellipse", json!({"stroke_dash_offset": 4})),
    ] {
        let found = fired(element.clone(), "E-DASH-OFFSET-ALONE");
        assert_eq!(found.len(), 1, "{element}: {found:?}");
        let id = element["id"].as_str().unwrap();
        let text = prose(element.clone());
        assert!(
            text.contains(&format!(
                "`{id}`.stroke_dash_offset: there is no `stroke_dash`"
            )),
            "{text}"
        );
    }
}

#[test]
fn a_cap_on_a_closed_path_draws_once_it_is_dashed() {
    let closed = |fields: Value| {
        path(with(
            json!({"closed": true, "points": [{"at": [40, 40]}, {"at": [160, 40]}, {"at": [100, 160]}]}),
            fields,
        ))
    };
    let undrawn = closed(json!({"stroke_cap": "round"}));
    assert_eq!(fired(undrawn.clone(), "E-STROKE-CAP-UNDRAWN").len(), 1);
    let text = prose(undrawn);
    assert!(
        text.contains("`closed`") && text.contains("`stroke_dash`"),
        "{text}"
    );
    for cap in ["butt", "round", "square"] {
        let dashed = closed(json!({"stroke_cap": cap, "stroke_dash": [6, 4]}));
        assert_eq!(errors(dashed), [], "{cap}");
    }
}

#[test]
fn a_square_cap_widens_the_inset_on_a_dashed_closed_path() {
    let element = path(
        json!({"closed": true, "stroke_cap": "square", "stroke_width": 8,
        "stroke_dash": [6, 4]}),
    );
    let reach = montagent_core::stroke::reach(&element);
    assert_eq!(reach.inset, 6);
    assert_eq!(reach.source, montagent_core::stroke::Source::SquareCap);
}

// ---------------------------------------------------------------------------
// The painter.
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

/// One 40-px dash, then a gap longer than any outline here: only the first dash draws, so
/// where it lies is where the pattern starts, and which way it runs.
const ONE_DASH: [i64; 2] = [40, 10_000];

#[test]
fn a_rects_pattern_starts_at_the_inset_top_left_corner_and_runs_along_the_top() {
    // Width 4: the inset outline is (2, 2)–(198, 198), so the dash covers x 2..42 at y 0..4.
    let rgb = painted(shape("rect", json!({"stroke_dash": ONE_DASH})));
    assert_eq!(pixel(&rgb, 20, 1), WHITE, "along the top edge");
    assert_eq!(pixel(&rgb, 60, 1), BLACK, "past the dash's end");
    assert_eq!(pixel(&rgb, 1, 20), BLACK, "not back down the left edge");
}

#[test]
fn a_rounded_rects_pattern_starts_where_the_top_left_arc_meets_the_top_edge() {
    // Radius 30 less the inset 2 is 28: the arc meets the top edge at x 30, so the dash runs
    // x 30..70; the arc itself, through (25, 2.45), is not drawn.
    let rgb = painted(shape(
        "rect",
        json!({"radius": 30, "stroke_dash": ONE_DASH}),
    ));
    assert_eq!(pixel(&rgb, 34, 1), WHITE, "just past the start");
    assert_eq!(pixel(&rgb, 66, 1), WHITE, "near the dash's end");
    assert_eq!(pixel(&rgb, 75, 1), BLACK, "past it");
    assert_eq!(pixel(&rgb, 25, 2), BLACK, "not on the arc before it");
}

#[test]
fn an_ellipses_pattern_starts_at_three_oclock_and_runs_toward_six() {
    // The inset ellipse has radius 98 about (100, 100): the dash leaves (198, 100) downward.
    let rgb = painted(shape("ellipse", json!({"stroke_dash": ONE_DASH})));
    assert_eq!(pixel(&rgb, 197, 115), WHITE, "below 3 o'clock");
    assert_eq!(pixel(&rgb, 197, 85), BLACK, "not above it");
}

#[test]
fn a_closed_paths_pattern_starts_at_its_first_point_in_points_order() {
    let element = path(json!({"closed": true,
        "points": [{"at": [40, 40]}, {"at": [160, 40]}, {"at": [160, 160]}, {"at": [40, 160]}],
        "stroke_dash": ONE_DASH}));
    let rgb = painted(element);
    assert_eq!(pixel(&rgb, 60, 40), WHITE, "along the first segment");
    assert_eq!(pixel(&rgb, 100, 40), BLACK, "past the dash's end");
    assert_eq!(
        pixel(&rgb, 40, 60),
        BLACK,
        "not back along the closing segment"
    );
}

#[test]
fn the_offset_moves_the_dashes_back_toward_the_start() {
    // An offset of 30 starts the outline 30 px into the first dash: 10 px of it remain.
    let rgb = painted(shape(
        "rect",
        json!({"stroke_dash": ONE_DASH, "stroke_dash_offset": 30}),
    ));
    assert_eq!(pixel(&rgb, 8, 1), WHITE);
    assert_eq!(pixel(&rgb, 20, 1), BLACK);
}

#[test]
fn an_offset_and_that_offset_plus_the_total_paint_the_same_bytes() {
    let pattern = json!([12, 8, 4, 8]); // total 32
    for element in [
        path(json!({"stroke_dash": pattern})),
        shape("rect", json!({"stroke_dash": pattern, "radius": 24})),
        shape("ellipse", json!({"stroke_dash": pattern})),
    ] {
        let at =
            |offset: i64| painted(with(element.clone(), json!({"stroke_dash_offset": offset})));
        for offset in [-37, 0, 5] {
            assert!(at(offset) == at(offset + 32), "{element} at {offset}");
            assert!(at(offset) == at(offset - 64), "{element} at {offset}");
        }
        assert!(at(0) != at(5), "the offset moves the pattern: {element}");
    }
}

#[test]
fn a_zero_dash_under_a_round_cap_draws_dots() {
    // Dots of diameter 6 every 20 px along y 100, from x 20.
    let rgb = painted(path(json!({"stroke_width": 6, "stroke_cap": "round",
        "stroke_dash": [0, 20]})));
    for x in [20, 40, 60, 160] {
        assert_eq!(pixel(&rgb, x, 100), WHITE, "a dot at x {x}");
    }
    for x in [30, 50, 150] {
        assert_eq!(pixel(&rgb, x, 100), BLACK, "a gap at x {x}");
    }
}

#[test]
fn a_dash_with_no_gap_on_a_rect_paints_the_bytes_of_no_dash() {
    let plain = painted(shape("rect", json!({})));
    assert!(plain == painted(shape("rect", json!({"stroke_dash": [10, 0]}))));
}

#[test]
fn a_dashed_shape_never_inks_outside_its_box() {
    // The box is 120×120 at (40, 40), on every frame of keyed marching ants.
    let ants = json!([{"t": 0, "v": 0}, {"t": 1000, "v": -96, "ease": "linear"}]);
    let place = json!({"x": 40, "y": 40, "width": 120, "height": 120, "stroke_width": 9,
        "stroke_dash": [20, 12], "stroke_dash_offset": ants});
    for element in [
        shape("rect", place.clone()),
        shape("rect", with(place.clone(), json!({"radius": 30}))),
        shape("ellipse", place.clone()),
        path(with(
            place.clone(),
            json!({"closed": true, "stroke_join": "miter", "stroke_miter_limit": 4,
                "stroke_cap": "square",
                "points": [{"at": [18, 18]}, {"at": [102, 18]}, {"at": [60, 102]}]}),
        )),
    ] {
        assert_eq!(errors(element.clone()), [], "{element}");
        for t in (0..1000).step_by(100) {
            let rgb = painted_at(std::slice::from_ref(&element), t);
            let mut lit = 0;
            for y in 0..200 {
                for x in 0..200 {
                    let on = pixel(&rgb, x, y) != BLACK;
                    lit += usize::from(on);
                    let inside = (40..160).contains(&x) && (40..160).contains(&y);
                    assert!(
                        !on || inside,
                        "{} at {t} ms inks ({x}, {y})",
                        element["type"]
                    );
                }
            }
            assert!(lit > 0);
        }
    }
}

// ---------------------------------------------------------------------------
// `query --at`.
// ---------------------------------------------------------------------------

/// `query --at`'s answer at `t` for the one element of `element`'s project, as JSON.
fn queried_at(element: Value, t: i64) -> Value {
    let document = montagent_core::parse::read(&project(&[element])).unwrap();
    let answer = serde_json::to_value(montagent_core::verbs::query::at::at(&document, t, None));
    answer.unwrap()["stack"][0].clone()
}

fn queried(element: Value) -> Value {
    queried_at(element, 0)
}

#[test]
fn query_at_reports_the_offset_raw_as_written() {
    for offset in [5, 5 + 32, -59] {
        let present = queried(shape(
            "rect",
            json!({"stroke_dash": [12, 8, 4, 8], "stroke_dash_offset": offset}),
        ));
        assert_eq!(
            present["stroke"]["dash_offset"],
            json!(offset as f64),
            "{present}"
        );
    }
    // Keyed, it is the resolved value at the instant, unwrapped and unrounded.
    let ants = json!([{"t": 0, "v": 0}, {"t": 1000, "v": -300, "ease": "linear"}]);
    let present = queried_at(
        path(json!({"stroke_dash": [6, 4], "stroke_dash_offset": ants})),
        500,
    );
    assert_eq!(present["stroke"]["dash_offset"], json!(-150.0), "{present}");
    // No offset written is 0.
    let present = queried(path(json!({"stroke_dash": [6, 4]})));
    assert_eq!(present["stroke"]["dash_offset"], json!(0.0), "{present}");
}

#[test]
fn query_at_reports_the_painters_own_outline_length_labelled_informative() {
    let length = |element: Value| {
        let present = queried(element);
        assert!(
            present["stroke"]["outline_length_note"]
                .as_str()
                .is_some_and(|note| note.contains("not a contract")),
            "{present}"
        );
        present["stroke"]["outline_length"].as_f64().unwrap()
    };
    let dashed = json!({"stroke_dash": [6, 4]});
    // The 200×200 box inset by 2: a 196-px square.
    assert_eq!(length(shape("rect", dashed.clone())), 784.0);
    // Four 140-px sides and four quarter arcs of radius 28 (735.93). Skia measures a curve by
    // chords within a tolerance, so the figure is a little short of the exact one: that is
    // the measure the dashes are laid along, and why the figure is informative.
    let short_of = |measured: f64, exact: f64| measured <= exact && measured > exact * 0.997;
    let rounded = length(shape("rect", with(dashed.clone(), json!({"radius": 30}))));
    assert!(
        short_of(rounded, 560.0 + 56.0 * std::f64::consts::PI),
        "{rounded}"
    );
    let ellipse = length(shape("ellipse", dashed.clone()));
    assert!(short_of(ellipse, 196.0 * std::f64::consts::PI), "{ellipse}");
    assert_eq!(length(path(dashed.clone())), 160.0);
}

#[test]
fn query_at_reports_no_dash_figures_for_an_undashed_shape() {
    assert!(queried(shape("rect", json!({})))["stroke"].is_null());
    let present = queried(path(json!({})));
    assert!(present["stroke"]["dash_offset"].is_null(), "{present}");
    assert!(present["stroke"]["outline_length"].is_null(), "{present}");
    assert_eq!(
        present["stroke"]["inset"],
        json!(2),
        "the path's reach stays"
    );
    // A dashed rect has no inset or reach of its own to report.
    let present = queried(shape("rect", json!({"stroke_dash": [6, 4]})));
    assert!(present["stroke"]["inset"].is_null(), "{present}");
}

#[test]
fn query_at_prose_names_the_offset_and_the_length() {
    use montagent_core::verbs::query::{Ask, query};
    use montagent_core::wire::{Wire, render_query};
    let path = project(&[shape(
        "rect",
        json!({"stroke_dash": [6, 4], "stroke_dash_offset": -12}),
    )]);
    let ask = Ask {
        at: Some(0),
        ..Ask::default()
    };
    let text = render_query(&query(&path, &ask), Wire::Text { verbose: false });
    assert!(
        text.contains("dash_offset -12 (outline 784 px, informative)"),
        "{text}"
    );
}

// ---------------------------------------------------------------------------
// `shift`.
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

/// Marching ants on a rect: the offset runs 0 to −100 over the element's second.
fn ants() -> Value {
    shape(
        "rect",
        json!({"stroke_dash": [6, 4],
            "stroke_dash_offset": [{"t": 0, "v": 0}, {"t": 1000, "v": -100, "ease": "linear"}]}),
    )
}

#[test]
fn shift_refuses_a_cut_mid_marching_ants_at_a_fractional_offset() {
    // At 333 ms the offset is −33.3, and rounding it would jump the ants.
    let path = project(&[ants()]);
    let before = std::fs::read_to_string(&path).unwrap();
    let answer = shifted(&path, 333);
    let refusals: Vec<_> = answer
        .report()
        .findings
        .iter()
        .filter(|finding| finding.code == "E-SHIFT-SPLIT-UNWRITABLE")
        .collect();
    assert_eq!(refusals.len(), 1, "{:?}", answer.report().findings);
    assert_eq!(refusals[0].fields["property"], "stroke_dash_offset");
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        before,
        "nothing is written"
    );
}

#[test]
fn shift_cuts_marching_ants_at_a_whole_offset_and_keeps_the_pattern() {
    // At 250 ms the offset is −25.
    let path = project(&[ants()]);
    let answer = shifted(&path, 250);
    assert_eq!(
        answer.report().exit_code(),
        montagent_core::report::ExitCode::Ok,
        "{:?}",
        answer.report().findings
    );
    let written = common::document(&path);
    let element = &written["tracks"][0]["elements"][0];
    assert_eq!(element["stroke_dash"], json!([6, 4]));
    let records: Vec<(i64, i64)> = element["stroke_dash_offset"]
        .as_array()
        .unwrap()
        .iter()
        .map(|record| (record["t"].as_i64().unwrap(), record["v"].as_i64().unwrap()))
        .collect();
    assert_eq!(records, [(0, 0), (250, -25), (350, -25), (1100, -100)]);
}
