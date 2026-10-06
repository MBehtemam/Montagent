//! Gradients, slice 1 (#686, ADR-0149): a paint is a colour, a keyframe list of colours, or
//! a static `linear` or `radial` gradient, measured against the element's declared box.

use std::path::{Path, PathBuf};

use montagent_core::finding::Class;
use montagent_core::report::Report;
use montagent_core::verbs::render::{Supplying, paint_span};
use serde_json::{Value, json};

mod common;
use common::{canonical, tempdir, write_project};

fn codes(report: &Report, class: Class) -> Vec<String> {
    report
        .findings
        .iter()
        .filter(|finding| finding.class == class)
        .map(|finding| format!("{}: {:?}", finding.code, finding.fields))
        .collect()
}

fn errors(report: &Report) -> Vec<String> {
    codes(report, Class::Error)
}

/// One track per element, stacked in the order given.
fn tracks(elements: Value) -> Value {
    let elements = elements.as_array().expect("a list of elements");
    elements
        .iter()
        .enumerate()
        .map(|(i, element)| json!({"name": format!("t{i}"), "layer": i, "elements": [element]}))
        .collect()
}

/// A `frame`-sized project at 10 fps over black, with the fixture's fonts so a text element
/// measures. Written beside the fixture, where its fonts resolve.
fn project(frame: (i64, i64), elements: Value) -> Value {
    let fixture = common::document(&common::fixture_project());
    json!({
        "frame": {"width": frame.0, "height": frame.1}, "fps": 10, "background": "#000000",
        "fonts": fixture["fonts"], "fontVendor": fixture["fontVendor"],
        "tracks": tracks(elements)
    })
}

/// A shapes-only project in a scratch directory of its own.
fn shapes(dir: &Path, frame: (i64, i64), elements: Value) -> PathBuf {
    let body = json!({
        "frame": {"width": frame.0, "height": frame.1}, "fps": 10, "background": "#000000",
        "tracks": tracks(elements)
    });
    write_project(dir, "p.json", &canonical(&body.to_string()))
}

fn linear(angle: f64, stops: Value) -> Value {
    json!({"gradient": "linear", "angle": angle, "stops": stops})
}

fn radial(center: [f64; 2], radius: f64, stops: Value) -> Value {
    json!({"gradient": "radial", "center": center, "radius": radius, "stops": stops})
}

fn two(from: &str, to: &str) -> Value {
    json!([{"offset": 0, "color": from}, {"offset": 1, "color": to}])
}

/// The acceptance project: a linear and a radial fill on a `rect` and an `ellipse`, a
/// gradient `stroke`, and a gradient text `color`.
fn acceptance() -> Value {
    project(
        (400, 300),
        json!([
            {"id": "card", "type": "rect", "start": 0, "end": 1000, "x": 100, "y": 80,
             "width": 160, "height": 100, "fill": linear(30.0, two("#FF3366", "#3366FF")),
             "stroke": radial([0.5, 0.5], 1.0, two("#FFFFFF", "#000000")), "stroke_width": 6},
            {"id": "glow", "type": "ellipse", "start": 0, "end": 1000, "x": 300, "y": 80,
             "width": 140, "height": 90, "fill": radial([0.25, 0.3], 0.8, two("#FFCC00", "#FF330000"))},
            {"id": "title", "type": "text", "start": 0, "end": 1000, "x": 200, "y": 220,
             "width": 360, "height": 100, "font": "brand", "size": 60,
             "color": linear(90.0, two("#FFFFFF", "#00CCFF")),
             "runs": [{"text": "Hello"}], "caption": false}
        ]),
    )
}

#[test]
fn a_project_with_every_paint_field_holding_a_gradient_validates_and_renders() {
    let scratch = common::Scratch::beside_the_fixture(
        "gradient-acceptance",
        &canonical(&acceptance().to_string()),
    );
    let report = montagent_core::validate(scratch.path());
    assert!(errors(&report).is_empty(), "{:#?}", errors(&report));

    let rasters = paint_span(scratch.path(), 500, 600, Supplying::PerFrame).expect("it paints");
    assert!(rasters.declined.is_empty(), "{:?}", rasters.declined);
}

// ---------------------------------------------------------------------------
// What the schema refuses.
// ---------------------------------------------------------------------------

/// The `E-SCHEMA` reasons `validate` gives for a project, written to a scratch directory.
#[track_caller]
fn schema_reasons(body: Value) -> Vec<String> {
    let dir = tempdir(std::panic::Location::caller().line());
    let path = write_project(&dir, "p.json", &canonical(&body.to_string()));
    montagent_core::validate(&path)
        .findings
        .iter()
        .filter(|finding| finding.code.starts_with("E-SCHEMA"))
        .map(|finding| format!("{}: {:?}", finding.code, finding.fields))
        .collect()
}

/// One rect whose `fill` is `fill`, and the reasons the schema refuses it for.
#[track_caller]
fn refused_fill(fill: Value) -> Vec<String> {
    schema_reasons(json!({
        "frame": {"width": 200, "height": 200}, "fps": 10,
        "tracks": tracks(json!([{"id": "box", "type": "rect", "start": 0, "end": 1000,
                                 "width": 100, "height": 50, "fill": fill}]))
    }))
}

#[track_caller]
fn refused_saying(reasons: Vec<String>, words: &str) {
    assert!(
        reasons.iter().any(|reason| reason.contains(words)),
        "expected a schema error saying {words:?}, got {reasons:?}"
    );
}

#[test]
fn the_schema_refuses_a_malformed_gradient() {
    let stops = two("#FFFFFF", "#000000");
    refused_saying(
        refused_fill(json!({"gradient": "conic", "angle": 0, "stops": stops})),
        "unknown variant `conic`",
    );
    refused_saying(
        refused_fill(
            json!({"gradient": "radial", "center": [0.5, 0.5], "radius": 1, "angle": 0, "stops": stops}),
        ),
        r#"UNKNOWN-KEY: {"key": String("angle")"#,
    );
    refused_saying(
        refused_fill(
            json!({"gradient": "linear", "angle": 0, "center": [0.5, 0.5], "stops": stops}),
        ),
        r#"UNKNOWN-KEY: {"key": String("center")"#,
    );
    refused_saying(
        refused_fill(json!({"gradient": "linear", "stops": stops})),
        "missing field `angle`",
    );
    refused_saying(
        refused_fill(json!({"gradient": "radial", "center": [0.5, 0.5], "stops": stops})),
        "missing field `radius`",
    );
    refused_saying(
        refused_fill(json!({"gradient": "linear", "angle": 0,
                            "stops": [{"offset": 0, "color": "#FFFFFF"}]})),
        "at least two",
    );
    refused_saying(
        refused_fill(json!({"gradient": "linear", "angle": 0,
                            "stops": [{"offset": 0, "color": "#FFFFFF"}, {"offset": 1.5, "color": "#000000"}]})),
        "`offset` is 1.5",
    );
    refused_saying(
        refused_fill(
            json!({"gradient": "radial", "center": [0.5, 0.5], "radius": -0.5, "stops": stops}),
        ),
        "`radius` is -0.5",
    );
}

#[test]
fn the_schema_refuses_a_keyed_gradient_parameter_in_this_slice() {
    let stops = two("#FFFFFF", "#000000");
    refused_saying(
        refused_fill(json!({"gradient": "linear",
                            "angle": [{"t": 0, "v": 0}, {"t": 1000, "v": 90, "ease": "linear"}],
                            "stops": stops})),
        "cannot be keyed yet",
    );
    refused_saying(
        refused_fill(json!({"gradient": "linear", "angle": 0,
                            "stops": [{"t": 0, "v": stops}, {"t": 1000, "v": stops, "ease": "linear"}]})),
        "cannot be keyed yet",
    );
    // A paint's keyframe list holds colours, never a gradient.
    refused_saying(
        refused_fill(json!([{"t": 0, "v": "#FFFFFF"},
                            {"t": 1000, "v": linear(0.0, stops.clone()), "ease": "linear"}])),
        "never a gradient",
    );
}

#[test]
fn the_schema_refuses_a_gradient_on_a_colour_only_field() {
    let gradient = linear(90.0, two("#FFFFFF", "#000000"));
    let text = |runs: Value| {
        json!({"id": "word", "type": "text", "start": 0, "end": 1000, "width": 100,
               "height": 50, "font": "brand", "size": 20, "runs": runs})
    };
    let one = |element: Value| json!({"frame": {"width": 200, "height": 200}, "fps": 10, "tracks": tracks(json!([element]))});
    // Run and highlight paint.
    assert!(!schema_reasons(one(text(json!([{"text": "Hi", "color": gradient}])))).is_empty());
    assert!(!schema_reasons(one(text(json!([{"text": "Hi", "stroke": gradient}])))).is_empty());
    assert!(
        !schema_reasons(one(text(json!([{"text": "Hi", "highlight":
            {"start": 0, "end": 500, "color": gradient}}]))))
        .is_empty()
    );
    // The project `background`.
    assert!(
        !schema_reasons(json!({"frame": {"width": 200, "height": 200}, "fps": 10,
                               "background": gradient, "tracks": []}))
        .is_empty()
    );
    // An effect colour.
    assert!(
        !schema_reasons(one(
            json!({"id": "box", "type": "rect", "start": 0, "end": 1000,
            "width": 100, "height": 50, "fill": "#FFFFFF",
            "effects": [{"name": "shadow", "dx": 0, "dy": 0, "radius": 4, "color": gradient,
                         "opacity": 1}]})
        ))
        .is_empty()
    );
}

// ---------------------------------------------------------------------------
// What `validate` checks that the schema cannot.
// ---------------------------------------------------------------------------

/// The findings coded `code` for one rect with `fill` and `stroke`, as (class, fields).
#[track_caller]
fn findings_for(code: &str, fill: Value, stroke: Option<Value>) -> Vec<(Class, Value)> {
    let dir = tempdir(std::panic::Location::caller().line());
    let mut rect = json!({"id": "box", "type": "rect", "start": 0, "end": 1000, "x": 100,
                          "y": 100, "width": 100, "height": 50, "fill": fill});
    if let Some(stroke) = stroke {
        rect["stroke"] = stroke;
        rect["stroke_width"] = json!(4);
    }
    let path = shapes(&dir, (200, 200), json!([rect]));
    montagent_core::validate(&path)
        .findings
        .iter()
        .filter(|finding| finding.code == code)
        .map(|finding| {
            let fields: serde_json::Map<String, Value> = finding
                .fields
                .iter()
                .map(|(name, value)| (name.to_string(), value.clone()))
                .collect();
            (finding.class, Value::Object(fields))
        })
        .collect()
}

#[test]
fn a_stop_list_whose_offsets_decrease_is_an_error_naming_the_first_pair() {
    let stops = json!([{"offset": 0, "color": "#FFFFFF"}, {"offset": 0.6, "color": "#FF0000"},
                       {"offset": 0.4, "color": "#00FF00"}, {"offset": 0.2, "color": "#0000FF"}]);
    let found = findings_for(
        "E-GRADIENT-STOP-ORDER",
        json!("#000000"),
        Some(linear(0.0, stops)),
    );
    assert_eq!(found.len(), 1, "{found:?}");
    let (class, fields) = &found[0];
    assert_eq!(*class, Class::Error);
    assert_eq!(fields["property"], json!("stroke.stops"));
    // Stops are counted from 1: the third stop is the first that comes before the one
    // ahead of it.
    assert_eq!(fields["stop"], json!(3));
    assert_eq!(fields["offset"], json!(0.4));
    assert_eq!(fields["previous_offset"], json!(0.6));
}

#[test]
fn equal_offsets_are_a_hard_edge_and_no_finding() {
    let stops = json!([{"offset": 0, "color": "#FFFFFF"}, {"offset": 0.5, "color": "#FFFFFF"},
                       {"offset": 0.5, "color": "#000000"}, {"offset": 1, "color": "#000000"}]);
    assert!(findings_for("E-GRADIENT-STOP-ORDER", linear(0.0, stops), None).is_empty());
}

#[test]
fn a_gradient_that_paints_one_colour_is_reviewed() {
    let same = linear(45.0, two("#FF3366", "#FF3366"));
    let found = findings_for("R-GRADIENT-ONE-COLOUR", same, None);
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].0, Class::Review);
    assert_eq!(found[0].1["property"], json!("fill"));

    let collapsed = radial([0.5, 0.5], 0.0, two("#FFFFFF", "#000000"));
    let found = findings_for("R-GRADIENT-ONE-COLOUR", json!("#000000"), Some(collapsed));
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].1["property"], json!("stroke"));
    assert_eq!(found[0].1["colour"], json!("#000000"));
}

#[test]
fn a_gradient_with_two_colours_is_not_reviewed() {
    // The same colour at different alphas is two colours.
    let fade = linear(45.0, two("#FF3366", "#FF336600"));
    assert!(findings_for("R-GRADIENT-ONE-COLOUR", fade, None).is_empty());
    let glow = radial([0.5, 0.5], 0.1, two("#FFFFFF", "#000000"));
    assert!(findings_for("R-GRADIENT-ONE-COLOUR", glow, None).is_empty());
}

// ---------------------------------------------------------------------------
// The run-override review widens to a gradient (ADR-0149 §5, ADR-0146 §6).
// ---------------------------------------------------------------------------

#[track_caller]
fn overridden(color: Value, runs: Value) -> Vec<String> {
    let dir = tempdir(std::panic::Location::caller().line());
    let path = shapes(
        &dir,
        (200, 200),
        json!([{"id": "title", "type": "text", "start": 0, "end": 1000, "x": 100, "y": 100,
                "width": 180, "height": 60, "font": "brand", "size": 20, "color": color,
                "runs": runs}]),
    );
    let document = montagent_core::parse::read(&path).unwrap();
    let mut report = Report::new("validate", Some(document.path().to_string()));
    montagent_core::checks::overridden::check(&document, &mut report);
    report
        .findings
        .iter()
        .map(|finding| {
            assert_eq!(finding.code, "R-TEXT-PAINT-OVERRIDDEN");
            finding.fields["property"].as_str().unwrap().to_string()
        })
        .collect()
}

#[test]
fn a_text_gradient_every_run_overrides_is_reported() {
    let gradient = linear(90.0, two("#FFFFFF", "#00CCFF"));
    let runs =
        json!([{"text": "Hello ", "color": "#00FF00"}, {"text": "world", "color": "#0000FF"}]);
    assert_eq!(overridden(gradient, runs), ["color"]);
}

#[test]
fn a_text_gradient_one_run_leaves_alone_and_a_static_colour_are_not_reported() {
    let gradient = linear(90.0, two("#FFFFFF", "#00CCFF"));
    let some = json!([{"text": "Hello ", "color": "#00FF00"}, {"text": "world"}]);
    assert!(overridden(gradient, some).is_empty());
    let all =
        json!([{"text": "Hello ", "color": "#00FF00"}, {"text": "world", "color": "#0000FF"}]);
    assert!(overridden(json!("#FFFFFF"), all).is_empty());
}

// ---------------------------------------------------------------------------
// `query --at` prints a literal that can be pasted back.
// ---------------------------------------------------------------------------

/// What `query --at` prints for `property` on the element `id`.
#[track_caller]
fn printed(path: &Path, id: &str, property: &str, at: i64) -> Value {
    use montagent_core::verbs::query::{Ask, query};
    let answer = query(
        path,
        &Ask {
            at: Some(at),
            ..Ask::default()
        },
    )
    .to_json();
    let element = answer["query"]["stack"]
        .as_array()
        .expect("a stack")
        .iter()
        .find(|element| element["id"] == id)
        .unwrap_or_else(|| panic!("`{id}` is not in the stack: {answer}"))
        .clone();
    element["values"]
        .as_array()
        .expect("values")
        .iter()
        .find(|value| value["property"] == property)
        .unwrap_or_else(|| panic!("no `{property}` in {element}"))["value"]
        .clone()
}

#[test]
fn query_at_prints_a_gradient_that_pastes_back_to_the_same_bytes() {
    let fill = json!({"gradient": "linear", "angle": 30, "stops": [
        {"offset": 0, "color": "#FF3366"}, {"offset": 0.25, "color": "#FFCC00"},
        {"offset": 1, "color": "#3366FF00"}]});
    let stroke = json!({"gradient": "radial", "center": [0.25, 1.5], "radius": 0.75, "stops": [
        {"offset": 0, "color": "#FFFFFF"}, {"offset": 1, "color": "#000000"}]});
    let element = |fill: &Value, stroke: &Value| {
        json!([{"id": "card", "type": "rect", "start": 0, "end": 1000, "x": 100, "y": 100,
                "width": 160, "height": 90, "fill": fill, "stroke": stroke, "stroke_width": 8}])
    };
    let dir = tempdir(line!());
    let written = shapes(&dir, (200, 200), element(&fill, &stroke));

    // The literal, with its numbers in their shortest exact form and its colours as hex.
    let printed_fill = printed(&written, "card", "fill", 500);
    let printed_stroke = printed(&written, "card", "stroke", 500);
    assert_eq!(printed_fill, fill);
    assert_eq!(printed_stroke, stroke);
    assert_eq!(printed_fill.to_string(), fill.to_string());

    let dir = tempdir(line!());
    let pasted = shapes(&dir, (200, 200), element(&printed_fill, &printed_stroke));
    let report = montagent_core::validate(&pasted);
    assert!(errors(&report).is_empty(), "{:#?}", errors(&report));
    assert!(painted_at(&written, 500) == painted_at(&pasted, 500));
}

#[test]
fn query_at_prints_a_crossed_stop_list_fixed() {
    // Out of order is an error, and the printed paint is what is drawn: the crossing stop
    // raised to the one before it, keeping its own colour (ADR-0149 §4).
    let dir = tempdir(line!());
    let path = box_with(
        &dir,
        linear(
            90.0,
            json!([{"offset": 0.6, "color": "#FF0000"}, {"offset": 0.2, "color": "#0000FF"}]),
        ),
    );
    assert_eq!(
        printed(&path, "box", "fill", 0)["stops"],
        json!([{"offset": 0.6, "color": "#FF0000"}, {"offset": 0.6, "color": "#0000FF"}])
    );
}

// ---------------------------------------------------------------------------
// The tools carry a static gradient unchanged.
// ---------------------------------------------------------------------------

#[test]
fn shift_compare_timeline_and_the_contact_sheet_carry_a_static_gradient() {
    use montagent_core::report::ExitCode;
    use montagent_core::verbs::frame::{Ask as FrameAsk, frame};
    use montagent_core::verbs::shift::{Ask as ShiftAsk, shift};
    use montagent_core::verbs::{compare::compare, timeline};

    let body = canonical(&acceptance().to_string());
    let base = common::Scratch::beside_the_fixture("gradient-tools-base", &body);
    let before = common::document(base.path());

    // `timeline` names the paint.
    let view = timeline::timeline(base.path()).to_json();
    let details: Vec<String> = view["timeline"]["groups"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|group| group["elements"].as_array().unwrap())
        .filter_map(|element| element["detail"].as_str().map(str::to_string))
        .collect();
    assert!(
        details
            .iter()
            .any(|detail| detail == "160×100 linear #FF3366→#3366FF"),
        "{details:?}"
    );

    // The contact sheet paints it.
    let sheet = frame(
        base.path(),
        &FrameAsk {
            from: Some(0),
            to: Some(1000),
            ..FrameAsk::default()
        },
    );
    assert_eq!(
        sheet.report().exit_code(),
        ExitCode::Ok,
        "{}",
        sheet.to_json()
    );

    // `compare` reads the two as one project.
    let same = common::Scratch::beside_the_fixture("gradient-tools-same", &body);
    let drift = compare(base.path(), same.path());
    assert!(drift.findings.is_empty(), "{:?}", drift.findings);

    // `shift` moves every element and leaves each paint exactly as written.
    let answer = shift(
        base.path(),
        &ShiftAsk {
            at: 0,
            delta: 100,
            scope: None,
            release: Vec::new(),
        },
    );
    assert_eq!(
        answer.report().exit_code(),
        ExitCode::Ok,
        "{}",
        answer.to_json()
    );
    let after = common::document(base.path());
    for (was, is) in common::elements(&before).zip(common::elements(&after)) {
        assert_eq!(is["start"].as_i64(), was["start"].as_i64().map(|t| t + 100));
        for paint in ["fill", "stroke", "color"] {
            assert_eq!(is[paint], was[paint], "{} {paint}", was["id"]);
        }
    }
}

// ---------------------------------------------------------------------------
// The geometry of ADR-0149 §2, on a box that is not square.
// ---------------------------------------------------------------------------

/// The frame painted at `t`, as RGB.
fn painted_at(path: &Path, t: i64) -> Vec<u8> {
    let rasters = paint_span(path, t, t + 100, Supplying::PerFrame).expect("the span paints");
    assert!(rasters.declined.is_empty(), "{:?}", rasters.declined);
    rasters.frames.into_iter().next().unwrap()
}

fn pixel(rgb: &[u8], width: usize, x: usize, y: usize) -> [u8; 3] {
    let at = (y * width + x) * 3;
    [rgb[at], rgb[at + 1], rgb[at + 2]]
}

/// Within one byte per channel: the expected values are worked by hand to a tenth.
#[track_caller]
fn near(painted: [u8; 3], expected: [u8; 3]) {
    let off = painted
        .iter()
        .zip(expected)
        .any(|(p, e)| (i16::from(*p) - i16::from(e)).abs() > 1);
    assert!(!off, "painted {painted:?}, expected {expected:?}");
}

/// A 200×100 rect whose top-left is the frame's (0, 50), on a 200×200 frame.
fn box_with(dir: &Path, fill: Value) -> PathBuf {
    shapes(
        dir,
        (200, 200),
        json!([{"id": "box", "type": "rect", "start": 0, "end": 1000, "x": 0, "y": 50,
                "origin": "top-left", "width": 200, "height": 100, "fill": fill}]),
    )
}

#[test]
fn a_linear_pixel_lands_where_the_line_through_the_centre_puts_it() {
    let dir = tempdir(line!());
    let path = box_with(&dir, linear(30.0, two("#000000", "#FFFFFF")));
    // Pixel (150, 80) is (150.5, 30.5) in the box. From the centre (100, 50) that is
    // (50.5, −19.5); along (sin 30°, −cos 30°) = (0.5, −0.866) it is 42.14. The line is
    // |200·sin 30°| + |100·cos 30°| = 186.60 long, so the offset is 0.5 + 42.14/186.60 =
    // 0.7258, and the grey is 0.7258 × 255 = 185.1.
    near(pixel(&painted_at(&path, 0), 200, 150, 80), [185, 185, 185]);
}

#[test]
fn a_radial_pixel_lands_where_the_circle_in_box_fractions_puts_it() {
    let dir = tempdir(line!());
    let path = box_with(&dir, radial([0.25, 0.5], 1.0, two("#000000", "#FFFFFF")));
    // The farthest corner from (0.25, 0.5) is (1, 0), at √(0.75² + 0.5²) = 0.9014 in box
    // fractions. Pixel (100, 80) is (100.5, 30.5) in the box, (0.5025, 0.305) as fractions:
    // (0.2525, −0.195) from the centre, 0.3190 away. The offset is 0.3190/0.9014 = 0.3539,
    // and the grey is 0.3539 × 255 = 90.3.
    near(pixel(&painted_at(&path, 0), 200, 100, 80), [90, 90, 90]);
}

#[test]
fn a_fade_to_transparent_black_keeps_its_hue_at_the_midpoint() {
    let dir = tempdir(line!());
    let to_black = box_with(&dir, linear(90.0, two("#FF0000", "#00000000")));
    // Premultiplied: half way the stop is full red at half alpha, which over black is half
    // red. Straight interpolation would mix towards black and paint a quarter.
    let [r, g, b] = pixel(&painted_at(&to_black, 0), 200, 100, 100);
    assert!((126..=130).contains(&r), "{r}");
    assert_eq!([g, b], [0, 0]);

    // And a fade to transparent red paints the same bytes: the hue of a transparent stop
    // carries no weight.
    let dir = tempdir(line!());
    let to_red = box_with(&dir, linear(90.0, two("#FF0000", "#FF000000")));
    assert!(painted_at(&to_black, 0) == painted_at(&to_red, 0));
}

/// The first and last lit pixels along row `y` of a 400-wide frame.
fn ends_of_row(frame: &[u8], y: usize) -> ([u8; 3], [u8; 3]) {
    let lit: Vec<[u8; 3]> = (0..400)
        .map(|x| pixel(frame, 400, x, y))
        .filter(|p| p.iter().any(|c| *c > 100))
        .collect();
    (
        *lit.first().expect("a lit pixel"),
        *lit.last().expect("a lit pixel"),
    )
}

fn word(runs: Value) -> Value {
    project(
        (400, 300),
        json!([{"id": "word", "type": "text", "start": 0, "end": 1000, "x": 200, "y": 150,
                "width": 380, "height": 120, "font": "brand", "size": 90,
                "color": linear(90.0, two("#FF0000", "#0000FF")),
                "runs": runs, "caption": false}]),
    )
}

#[test]
fn a_text_gradient_runs_across_the_whole_block_and_a_run_colour_beats_it() {
    let across = common::Scratch::beside_the_fixture(
        "gradient-text-across",
        &canonical(&word(json!([{"text": "HHHH"}])).to_string()),
    );
    let (left, right) = ends_of_row(&painted_at(across.path(), 0), 150);
    // One gradient across the box, not one per letter: red at the left, blue at the right.
    assert!(left[0] > left[2], "{left:?}");
    assert!(right[2] > right[0], "{right:?}");

    let overridden = common::Scratch::beside_the_fixture(
        "gradient-text-overridden",
        &canonical(&word(json!([{"text": "HH"}, {"text": "HH", "color": "#00FF00"}])).to_string()),
    );
    let (left, right) = ends_of_row(&painted_at(overridden.path(), 0), 150);
    assert!(left[0] > left[2], "{left:?}");
    // The edge pixel is antialiased against black, so only its hue is the run's.
    assert_eq!([right[0], right[2]], [0, 0], "{right:?}");
}

#[test]
fn a_radial_of_radius_zero_paints_the_last_colour_over_the_box() {
    let dir = tempdir(line!());
    let path = box_with(&dir, radial([0.5, 0.5], 0.0, two("#FF0000", "#00FF00")));
    let frame = painted_at(&path, 0);
    near(pixel(&frame, 200, 100, 100), [0, 255, 0]);
    near(pixel(&frame, 200, 5, 55), [0, 255, 0]);
}
