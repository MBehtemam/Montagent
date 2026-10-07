//! Text on a path (#765, ADR-0161): a text element's one line bent along its own inline
//! `path`, placed by `path_offset` and `align`, with the four point errors and two of its
//! own, and `query --at` and `measure` naming the letters the curve hides.

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

/// A scratch directory of its own for every call, so tests running in parallel never share
/// one.
fn scratch() -> PathBuf {
    use std::sync::atomic::{AtomicU32, Ordering};
    static NEXT: AtomicU32 = AtomicU32::new(0);
    tempdir(2_000_000 + NEXT.fetch_add(1, Ordering::Relaxed))
}

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// A 640×360 project at 30 fps over black, one track holding `elements`, with Oswald
/// SemiBold and Noto Naskh Arabic behind it under the key `title`.
fn project(dir: &Path, elements: &[Value]) -> PathBuf {
    for (from, to) in [
        (
            "fixtures/benchmark/spy-trailer/fonts/Oswald-SemiBold.ttf",
            "fonts/Oswald-SemiBold.ttf",
        ),
        (
            "fixtures/letter-spacing/fonts/NotoNaskhArabic-Regular.ttf",
            "fonts/Naskh.ttf",
        ),
    ] {
        let to = dir.join(to);
        std::fs::create_dir_all(to.parent().unwrap()).unwrap();
        std::fs::copy(repo().join(from), to).unwrap();
    }
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
                "frame": {"width": WIDTH, "height": HEIGHT}, "fps": 30, "background": "#000000",
                "duration": 1000, "output": "out/p.mp4",
                "fonts": {"title": [{"file": "fonts/Oswald-SemiBold.ttf"}, {"file": "fonts/Naskh.ttf"}]},
                "fontVendor": {
                    "fonts/Oswald-SemiBold.ttf": {"licence": "OFL-1.1",
                        "source": "google/fonts ofl/oswald, instanced wght=600",
                        "sha256": "442420449b66e3f8a49025fbb229a8b4b1efa5f7be9458a7c3244498d34f8de9"},
                    "fonts/Naskh.ttf": {"licence": "OFL-1.1",
                        "source": "notofonts/arabic NotoNaskhArabic-v2.019",
                        "sha256": "eb5cde7fecba8c6a481039257fe02d5fd69b7b0e36f8afc56ee22f4b1d7e8c21"}
                },
                "tracks": tracks,
            })
            .to_string(),
        ),
    )
}

/// An arc over a 600×300 box: a line `HELLO CURVE` of size 40, centred on the arc's
/// middle.
fn arc() -> Value {
    json!({"id": "arc", "type": "text", "start": 0, "end": 1000, "x": 20, "y": 30, "origin": "top-left",
           "width": 600, "height": 300, "font": "title", "size": 40, "color": "#FFFFFF",
           "align": "center", "runs": [{"text": "HELLO CURVE"}],
           "path": {"closed": false, "points": [
               {"at": [60, 240], "out": [145, -150]},
               {"at": [540, 240], "in": [-145, -150]}]},
           "path_offset": 0.5, "caption": false})
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

fn validate(elements: &[Value]) -> Report {
    montagent_core::validate(&project(&scratch(), elements))
}

fn validated(element: Value) -> Vec<(String, String)> {
    errors(&validate(&[element]))
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
// The schema and the one list.
// ---------------------------------------------------------------------------

#[test]
fn a_text_on_an_arc_validates_clean() {
    assert_eq!(validated(arc()), []);
}

#[test]
fn path_offset_outside_minus_one_to_two_is_the_schema_range_error_in_a_static_value_and_every_record()
 {
    // ADR-0164 §1: one curve length past each end, and no further.
    for value in [2.01, -1.01] {
        schema_error(with(arc(), json!({"path_offset": value})), "path_offset");
        schema_error(
            with(
                arc(),
                json!({"path_offset": [{"t": 0, "v": 0.0}, {"t": 500, "v": value, "ease": "linear"}]}),
            ),
            "path_offset",
        );
    }
    for value in [-1.0, 2.0, 1.5, -0.25] {
        assert_eq!(validated(with(arc(), json!({"path_offset": value}))), []);
    }
}

#[test]
fn closed_is_required_and_static_and_the_path_takes_no_other_key() {
    let points = arc()["path"]["points"].clone();
    schema_error(with(arc(), json!({"path": {"points": points}})), "closed");
    schema_error(
        with(
            arc(),
            json!({"path": {"closed": [{"t": 0, "v": false}, {"t": 500, "v": true, "ease": "step"}],
                            "points": points}}),
        ),
        "two elements",
    );
    schema_error(
        with(
            arc(),
            json!({"path": {"closed": false, "points": points, "stroke": "#FFFFFF"}}),
        ),
        "stroke",
    );
}

#[test]
fn path_points_and_path_offset_join_the_one_list_of_animatable_properties() {
    let names: Vec<&str> = animatable::of("text")
        .iter()
        .map(|property| property.name.as_str())
        .collect();
    let at = |name: &str| names.iter().position(|n| *n == name);
    assert!(at("path.points").is_some(), "{names:?}");
    assert!(at("path_offset").is_some(), "{names:?}");
    assert_eq!(at("path.points").unwrap() + 1, at("path_offset").unwrap());
    let offset = animatable::of("text")
        .iter()
        .find(|property| property.name == "path_offset")
        .unwrap();
    assert_eq!(
        (offset.kind, offset.minimum, offset.maximum),
        (Kind::Number, Some(-1.0), Some(2.0))
    );
    assert_eq!(
        animatable::of("text")
            .iter()
            .find(|property| property.name == "path.points")
            .map(|property| property.kind),
        Some(Kind::Points)
    );
}

#[test]
fn an_overshooting_ease_clamps_path_offset_to_minus_one_to_two_in_the_one_resolving_function() {
    let element = with(
        arc(),
        json!({"path_offset": [{"t": 0, "v": -1.0}, {"t": 1000, "v": 2.0, "ease": [0.5, 2.5, 0.5, 1.0]}]}),
    );
    let mut overshot = false;
    for t in (0..=1000).step_by(50) {
        let value = animatable::number_at(&element, "path_offset", t, f64::NAN);
        assert!((-1.0..=2.0).contains(&value), "{t}: {value}");
        overshot |= value == 2.0 && t < 1000;
    }
    assert!(overshot, "the ease overshoots and is clamped");
}

#[test]
fn a_text_paths_points_clamp_into_the_text_inset_box() {
    // `m` = 40 (the size) on a 600×300 box: the apex sinking past the top is held at y 40.
    let element = with(
        arc(),
        json!({"path": {"closed": false, "points": [
            {"t": 0, "v": [{"at": [60, 240]}, {"at": [300, 100]}, {"at": [540, 240]}]},
            {"t": 1000, "v": [{"at": [60, 240]}, {"at": [300, 60]}, {"at": [540, 240]}],
             "ease": [0.5, 2.5, 0.5, 2.5]}]}}),
    );
    let mut low = f64::INFINITY;
    for t in (0..=1000).step_by(25) {
        let resolved =
            serde_json::to_value(animatable::at(&element, "path.points", t).unwrap().unwrap())
                .unwrap();
        low = low.min(resolved[1]["at"][1].as_f64().unwrap());
    }
    assert_eq!(low, 40.0);
}

// ---------------------------------------------------------------------------
// `validate`.
// ---------------------------------------------------------------------------

/// `validate`'s text report for `elements`.
fn validate_text(elements: &[Value]) -> String {
    let report = validate(elements);
    montagent_core::text::render(&report.to_json(), montagent_core::text::Options::default())
        .unwrap()
}

/// The `code` findings `validate` gives `element`, each as its fields.
fn findings(element: Value, code: &str) -> Vec<Value> {
    validate(&[element])
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

#[test]
fn a_line_break_in_any_run_of_a_text_on_a_path_is_an_error_naming_the_run() {
    let element = with(
        arc(),
        json!({"runs": [{"text": "ONE"}, {"text": " TWO\nTHREE"}]}),
    );
    let found = findings(element.clone(), "E-TEXT-PATH-BREAK");
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0]["run"], json!(1));
    let text = validate_text(&[element]);
    assert!(
        text.contains("`arc`.runs[1]: its text holds a line break"),
        "{text}"
    );
    // The same break on a flat text is an ordinary second line.
    let flat = with(
        arc(),
        json!({"runs": [{"text": "ONE\nTWO"}], "path": null, "path_offset": null}),
    );
    assert_eq!(findings(flat, "E-TEXT-PATH-BREAK"), Vec::<Value>::new());
}

#[test]
fn path_offset_on_a_text_with_no_path_is_an_orphan_error_whose_fix_is_to_drop_it() {
    let element = with(arc(), json!({"path": null}));
    let report = validate(std::slice::from_ref(&element));
    let orphan: Vec<_> = report
        .findings
        .iter()
        .filter(|finding| finding.code == "E-TEXT-PATH-OFFSET-ORPHAN")
        .collect();
    assert_eq!(orphan.len(), 1, "{:?}", report.findings);
    assert_eq!(orphan[0].class, Class::Error);
    let text = validate_text(&[element]);
    assert!(
        text.contains("`arc`.path_offset: this text carries no `path`"),
        "{text}"
    );
    assert_eq!(validated(arc()), [], "with its `path` it is no orphan");
}

#[test]
fn the_four_point_errors_fire_on_a_texts_path_and_name_the_element_type() {
    let points = |points: Value| with(arc(), json!({"path": {"closed": false, "points": points}}));
    let too_few = points(json!([{"at": [60, 240]}]));
    assert_eq!(findings(too_few.clone(), "E-PATH-TOO-FEW-POINTS").len(), 1);
    let dangling = points(json!([{"at": [60, 240], "in": [-10, 0]}, {"at": [540, 240]}]));
    assert_eq!(
        findings(dangling.clone(), "E-PATH-DANGLING-HANDLE").len(),
        1
    );
    let shape = points(json!([
        {"t": 0, "v": [{"at": [60, 240]}, {"at": [540, 240]}]},
        {"t": 500, "v": [{"at": [60, 240]}, {"at": [300, 100]}, {"at": [540, 240]}], "ease": "linear"}]));
    assert_eq!(findings(shape.clone(), "E-PATH-KEYFRAME-SHAPE").len(), 1);
    let outside = points(json!([{"at": [20, 240]}, {"at": [540, 240]}]));
    assert_eq!(findings(outside.clone(), "E-PATH-OUTSIDE-BOX").len(), 1);
    for (element, says) in [
        (
            too_few,
            "`arc`.path.points: on a `text`, an open path needs at least 2 vertices",
        ),
        (
            dangling,
            "`arc`.path.points: on a `text`, vertex 0 carries an `in`",
        ),
        (shape, "`arc`.path.points: on a `text`, keyframe record 2"),
        (
            outside,
            "`arc`.path.points: on a `text`, vertex 0's `at` sits at [20,240]",
        ),
    ] {
        let text = validate_text(&[element]);
        assert!(text.contains(says), "{says:?} in\n{text}");
    }
}

#[test]
fn outside_the_box_on_a_text_uses_the_largest_run_size_plus_the_largest_stroke_and_says_so() {
    // Runs of 30 and 52 on an element of 40, a run stroke of 3 and an element stroke keyed
    // up to 5: m = 52 + 5 = 57.
    let element = with(
        arc(),
        json!({"runs": [{"text": "BIG", "size": 52, "stroke_width": 3}, {"text": " small", "size": 30}],
               "stroke": "#000000",
               "stroke_width": [{"t": 0, "v": 2}, {"t": 500, "v": 5, "ease": "linear"}],
               "path": {"closed": false, "points": [{"at": [56, 240]}, {"at": [540, 240]}]}}),
    );
    let found = findings(element.clone(), "E-PATH-OUTSIDE-BOX");
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0]["inset"], json!(57));
    assert_eq!(found[0]["right"], json!(600 - 57));
    let text = validate_text(&[element]);
    assert!(
        text.contains(
            "inset 57 = 52 (the largest `size` among the runs) + 5 (the largest `stroke_width` \
             among the runs and the element)"
        ),
        "{text}"
    );
    assert!(text.contains("not a containment guarantee"), "{text}");
}

#[test]
fn outside_the_box_checks_every_keyframe_of_a_texts_points() {
    let element = with(
        arc(),
        json!({"path": {"closed": false, "points": [
            {"t": 0, "v": [{"at": [60, 240]}, {"at": [540, 240]}]},
            {"t": 500, "v": [{"at": [60, 270]}, {"at": [540, 240]}], "ease": "linear"}]}}),
    );
    let found = findings(element, "E-PATH-OUTSIDE-BOX");
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0]["record"], json!(2));
}

#[test]
fn box_slack_is_silent_on_a_text_on_a_path_and_still_speaks_on_the_same_box_flat() {
    let slack = |element: Value| {
        validate(&[element])
            .findings
            .iter()
            .filter(|finding| finding.code == "R-BOX-SLACK")
            .count()
    };
    assert_eq!(slack(arc()), 0);
    assert_eq!(
        slack(with(arc(), json!({"path": null, "path_offset": null}))),
        1,
        "a 300-px box over one 48-px line is slack when flat"
    );
}

// ---------------------------------------------------------------------------
// The painter, and `query --at` naming what it hides.
// ---------------------------------------------------------------------------

/// The frame painted at `t`, as RGB.
fn painted_at(path: &Path, t: i64) -> Vec<u8> {
    let rasters = paint_span(path, t, t + 1, Supplying::PerFrame).expect("the span paints");
    assert!(rasters.declined.is_empty(), "{:?}", rasters.declined);
    rasters.frames.into_iter().next().unwrap()
}

fn painted(elements: &[Value], t: i64) -> Vec<u8> {
    painted_at(&project(&scratch(), elements), t)
}

/// The bounding box of every lit pixel, `[left, top, right, bottom]`.
fn lit(rgb: &[u8]) -> Option<[usize; 4]> {
    let mut out: Option<[usize; 4]> = None;
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let at = (y * WIDTH + x) * 3;
            if rgb[at..at + 3].iter().any(|c| *c > 64) {
                out = Some(match out {
                    None => [x, y, x, y],
                    Some([l, t, r, b]) => [l.min(x), t.min(y), r.max(x), b.max(y)],
                });
            }
        }
    }
    out
}

/// `query --at`'s answer for `elements`' project, as JSON.
fn query_at(elements: &[Value], t: i64) -> Value {
    let path = project(&scratch(), elements);
    let document = montagent_core::parse::read(&path).unwrap();
    serde_json::to_value(at::at(&document, t, None)).unwrap()
}

/// `query --at`'s row for the one element of `elements`' project.
fn queried(elements: &[Value], t: i64) -> Value {
    query_at(elements, t)["stack"][0].clone()
}

/// A text running down a vertical line at box x 300, top to bottom: 240 px of curve.
fn falling(text: &str, fields: Value) -> Value {
    with(
        json!({"id": "fall", "type": "text", "start": 0, "end": 1000, "x": 20, "y": 20, "origin": "top-left",
               "width": 600, "height": 320, "font": "title", "size": 30, "color": "#FFFFFF",
               "runs": [{"text": text}],
               "path": {"closed": false, "points": [{"at": [300, 40]}, {"at": [300, 280]}]},
               "caption": false}),
        fields,
    )
}

const LONG: &str = "ABCDEFGHIJKLMNOPQRSTUVWX";

#[test]
fn a_text_on_a_vertical_path_paints_a_column_down_the_curve() {
    let [left, top, right, bottom] = lit(&painted(&[falling("ABCDEF", json!({}))], 0)).unwrap();
    // The column straddles frame x 320 and runs down from frame y 60; flat, the same line
    // would be one row about 30 pixels tall.
    assert!(left > 280 && right < 360, "{left}..{right}");
    assert!((55..=66).contains(&top), "{top}");
    assert!(bottom - top > 80, "{top}..{bottom}");
}

#[test]
fn the_guide_is_never_painted() {
    // One letter far along the curve: nothing is drawn near the curve's start.
    let rgb = painted(&[falling("A", json!({"path_offset": 0.9}))], 0);
    let [_, top, _, _] = lit(&rgb).unwrap();
    assert!(top > 250, "{top}");
}

#[test]
fn query_names_the_letters_an_open_end_hides_and_the_painter_hides_exactly_them() {
    // `align: start` at offset 0 keeps the head on the 240-px curve and runs the tail past
    // its far end.
    let row = queried(&[falling(LONG, json!({}))], 0);
    let reading = &row["text_path"];
    assert_eq!(reading["path_offset"], json!(0.0), "{row}");
    assert!(
        (reading["length"].as_f64().unwrap() - 240.0).abs() < 0.05,
        "{row}"
    );
    let hidden: Vec<usize> = serde_json::from_value(reading["hidden"].clone()).unwrap();
    assert!(!hidden.is_empty() && hidden.len() < 24, "{row}");
    let first = hidden[0];
    assert_eq!(
        hidden,
        (first..24).collect::<Vec<_>>(),
        "the tail is hidden"
    );
    // The painter draws exactly the letters `query` does not name: the line cut back to
    // them paints the same bytes.
    let shown: String = LONG.chars().take(first).collect();
    assert!(
        painted(&[falling(LONG, json!({}))], 0) == painted(&[falling(&shown, json!({}))], 0),
        "the frame hides {hidden:?}"
    );
}

#[test]
fn one_element_slides_fully_on_and_fully_off_an_open_curve() {
    // ADR-0164 §1: `align: start` keyed −1 → 1 brings a line no longer than its curve in
    // from the start and takes it out past the end.
    let slide = falling(
        "ABCDEF",
        json!({"align": "start",
               "path_offset": [{"t": 0, "v": -1.0}, {"t": 900, "v": 1.0, "ease": "linear"}]}),
    );
    let at = |t: i64| queried(std::slice::from_ref(&slide), t)["text_path"].clone();
    let all = json!([0, 1, 2, 3, 4, 5]);
    assert_eq!(at(0)["hidden"], all);
    assert_eq!(at(0)["path_offset"], json!(-1.0));
    assert_eq!(at(500)["hidden"], json!("none"), "fully drawn in between");
    assert_eq!(at(900)["hidden"], all);
    assert_eq!(lit(&painted(std::slice::from_ref(&slide), 0)), None);
    assert!(lit(&painted(std::slice::from_ref(&slide), 500)).is_some());
    assert_eq!(lit(&painted(&[slide], 900)), None);
}

#[test]
fn a_slide_on_from_the_start_end_hides_the_head_until_it_arrives() {
    // `align: end` with the offset keyed 0 → 1: at 0 the whole line is before the start.
    let sliding = falling(
        "ABCDEF",
        json!({"align": "end",
               "path_offset": [{"t": 0, "v": 0.0}, {"t": 1000, "v": 1.0, "ease": "linear"}]}),
    );
    let at = |t: i64| queried(std::slice::from_ref(&sliding), t)["text_path"]["hidden"].clone();
    assert_eq!(at(0), json!([0, 1, 2, 3, 4, 5]));
    assert_eq!(at(999), json!("none"));
    let middle: Vec<usize> = serde_json::from_value(at(150)).unwrap_or_default();
    assert!(
        !middle.is_empty() && middle[0] == 0 && middle.len() < 6,
        "the head arrives last: {middle:?}"
    );
    // Nothing is drawn while every letter is hidden.
    assert_eq!(lit(&painted(&[sliding], 0)), None);
}

/// A closed circle of radius 60 centred in a 200×200 box, clockwise on screen from 12
/// o'clock.
fn circle() -> Value {
    json!({"closed": true, "points": [
        {"at": [100, 40], "in": [-33, 0], "out": [33, 0]},
        {"at": [160, 100], "in": [0, -33], "out": [0, 33]},
        {"at": [100, 160], "in": [33, 0], "out": [-33, 0]},
        {"at": [40, 100], "in": [0, 33], "out": [0, -33]}]})
}

fn badge(text: &str) -> Value {
    json!({"id": "badge", "type": "text", "start": 0, "end": 1000, "x": 200, "y": 80, "origin": "top-left",
           "width": 200, "height": 200, "font": "title", "size": 24, "color": "#FFFFFF",
           "runs": [{"text": text}], "path": circle(), "caption": false})
}

#[test]
fn a_closed_loop_draws_only_the_bodies_within_one_loop_and_query_names_the_rest() {
    let long = "AROUND THE WORLD AND BACK AGAIN TWICE OVER";
    let row = queried(&[badge(long)], 0);
    let reading = &row["text_path"];
    let length = reading["length"].as_f64().unwrap();
    assert!((370.0..385.0).contains(&length), "{row}");
    let hidden: Vec<usize> = serde_json::from_value(reading["hidden"].clone()).unwrap();
    let letters = long.chars().filter(|c| !c.is_whitespace()).count();
    let first = hidden[0];
    assert_eq!(hidden, (first..letters).collect::<Vec<_>>(), "{row}");
    // The painter draws exactly the letters that are not named.
    let mut kept = 0;
    let shown: String = long
        .chars()
        .take_while(|c| {
            kept += usize::from(!c.is_whitespace());
            kept <= first
        })
        .collect();
    assert!(painted(&[badge(long)], 0) == painted(&[badge(shown.trim_end())], 0));
    // A line that fits the loop hides nothing.
    assert_eq!(
        queried(&[badge("BADGE")], 0)["text_path"]["hidden"],
        json!("none")
    );
}

#[test]
fn on_a_closed_loop_no_drawn_letter_reaches_past_one_loop_from_the_lines_start() {
    // §6: the drawn bodies lie whole within one loop, so the last drawn letter's trailing
    // edge is at most one loop from the line's start, and none lies on the first.
    // Unbroken lines, so the letter at the seam straddles it: drawn by its midpoint, it would
    // reach past the loop and onto the first letter.
    use montagent_core::verbs::measure::{Ask, measure};
    let path = project(&scratch(), &[]);
    let alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHIJKLMNOPQRSTUVWXYZ";
    for skip in 0..6 {
        let long = &alphabet[skip..skip + 40];
        let row = queried(&[badge(long)], 0);
        let length = row["text_path"]["length"].as_f64().unwrap();
        let hidden: Vec<usize> =
            serde_json::from_value(row["text_path"]["hidden"].clone()).unwrap();
        let flat = |count: usize| {
            let element = with(badge(&long[..count]), json!({"path": null}));
            measure(
                &path,
                &Ask {
                    element: Some(element),
                    ..Ask::default()
                },
            )
            .to_json()["measure"]["advance_width"]
                .as_f64()
                .unwrap()
        };
        assert!(
            flat(hidden[0]) <= length,
            "{long}: the drawn letters fit one loop"
        );
        assert!(
            flat(hidden[0] + 1) > length,
            "{long}: the next letter would pass it"
        );
    }
}

#[test]
fn keyed_points_that_change_the_curves_length_change_the_hidden_set_from_frame_to_frame() {
    let growing = falling(
        "ABCDEFGHIJ",
        json!({"path": {"closed": false, "points": [
            {"t": 0, "v": [{"at": [300, 40]}, {"at": [300, 100]}]},
            {"t": 1000, "v": [{"at": [300, 40]}, {"at": [300, 280]}], "ease": "linear"}]}}),
    );
    let at = |t: i64| queried(std::slice::from_ref(&growing), t)["text_path"].clone();
    let (early, late) = (at(0), at(900));
    assert!(early["length"].as_f64().unwrap() < late["length"].as_f64().unwrap());
    let count = |reading: &Value| reading["hidden"].as_array().map_or(0, Vec::len);
    assert!(count(&early) > count(&late), "{early} {late}");
}

#[test]
fn query_ink_box_is_the_bent_lines_ink_where_the_frame_shows_it() {
    // The declared box is the frame the curve is written in, so `origin` places that box:
    // with `center` it is centred on (320, 180), whatever the line's own block is.
    for fields in [json!({}), json!({"origin": "center", "x": 320, "y": 180})] {
        let falling = falling("ABCDEF", fields.clone());
        let ink = queried(std::slice::from_ref(&falling), 0)["ink_box"].clone();
        let [left, top, right, bottom] = lit(&painted(&[falling], 0)).unwrap();
        let near = |a: f64, b: usize| (a - b as f64).abs() <= 2.0;
        let (x, y) = (ink["x"].as_f64().unwrap(), ink["y"].as_f64().unwrap());
        let (w, h) = (
            ink["width"].as_f64().unwrap(),
            ink["height"].as_f64().unwrap(),
        );
        assert!(
            near(x, left) && near(y, top),
            "{fields}: {ink} vs {left},{top}"
        );
        assert!(
            near(x + w, right + 1) && near(y + h, bottom + 1),
            "{fields}: {ink} vs {right},{bottom}"
        );
        // The curve runs down box x 300 from box y 40: centred, the box's top-left is at
        // (20, 20), as with `top-left` at (20, 20).
        assert!(
            (300..340).contains(&left) && (55..66).contains(&top),
            "{left},{top}"
        );
    }
}

#[test]
fn query_prints_the_offset_length_and_hidden_letters_on_a_text_on_a_path_only() {
    let path = project(
        &scratch(),
        &[
            falling(LONG, json!({"path_offset": 0.25})),
            with(
                arc(),
                json!({"id": "flat", "path": null, "path_offset": null}),
            ),
        ],
    );
    let answer = montagent_core::verbs::query::query(
        &path,
        &montagent_core::verbs::query::Ask {
            at: Some(0),
            ..Default::default()
        },
    );
    let json = answer.to_json();
    let flat = json["query"]["stack"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == "flat")
        .unwrap();
    assert!(flat.get("text_path").is_none(), "{flat}");
    let text =
        montagent_core::text::render(&json, montagent_core::text::Options::default()).unwrap();
    let line = text
        .lines()
        .find(|line| line.contains("fall"))
        .expect("a row for the bent text");
    assert!(
        line.contains("path_offset 0.25, blend normal, curve 240 px (informative), hidden: "),
        "{line}"
    );
    assert_eq!(line.matches("path_offset").count(), 1, "{line}");
    let flat_line = text.lines().find(|line| line.contains("flat")).unwrap();
    assert!(!flat_line.contains("path_offset"), "{flat_line}");
}

// ---------------------------------------------------------------------------
// `measure`.
// ---------------------------------------------------------------------------

#[test]
fn measure_reports_the_bent_lines_ink_and_hidden_letters_at_the_first_frame() {
    use montagent_core::verbs::measure::{Ask, measure};
    let path = project(&scratch(), &[]);
    let element = with(falling(LONG, json!({})), json!({"start": 200}));
    let answer = measure(
        &path,
        &Ask {
            element: Some(element),
            ..Ask::default()
        },
    )
    .to_json();
    let bent = &answer["measure"]["path"];
    let hidden = bent["hidden"].as_array().expect("letters are hidden");
    assert!(!hidden.is_empty(), "{answer}");
    let [left, top, right, bottom] =
        serde_json::from_value::<[f64; 4]>(bent["ink"].clone()).expect("ink");
    // A column at box x 300, from box y 40 down to the curve's end.
    assert!(left > 270.0 && right < 330.0, "{left}..{right}");
    assert!(
        (35.0..45.0).contains(&top) && bottom > 250.0,
        "{top}..{bottom}"
    );
    // The flat numbers stay the flat line's.
    assert_eq!(answer["measure"]["line_count"], json!(1));
    let text =
        montagent_core::text::render(&answer, montagent_core::text::Options::default()).unwrap();
    assert!(text.contains("on its path"), "{text}");
    // A flat text answers as before.
    let flat = measure(
        &path,
        &Ask {
            element: Some(with(arc(), json!({"path": null, "path_offset": null}))),
            ..Ask::default()
        },
    )
    .to_json();
    assert!(flat["measure"].get("path").is_none(), "{flat}");
}

/// `measure`'s bent ink for `element`, `[left, top, right, bottom]` in box pixels.
fn bent_ink(element: Value) -> [f64; 4] {
    use montagent_core::verbs::measure::{Ask, measure};
    let answer = measure(
        &project(&scratch(), &[]),
        &Ask {
            element: Some(element),
            ..Ask::default()
        },
    )
    .to_json();
    serde_json::from_value(answer["measure"]["path"]["ink"].clone()).expect("ink")
}

// ---------------------------------------------------------------------------
// Where the line sits: `align`, direction, rigid bodies and the stagger.
// ---------------------------------------------------------------------------

#[test]
fn align_center_at_offset_one_half_centres_the_line_on_a_symmetric_arc() {
    let [left, _, right, _] = bent_ink(arc());
    assert!(
        ((left + right) / 2.0 - 300.0).abs() < 3.0,
        "{left}..{right}"
    );
}

#[test]
fn align_start_and_end_name_ends_of_the_curve_not_of_the_reading_direction() {
    // An Arabic line, right-to-left, with the defaults: `align: start` and offset 0 put its
    // lowest-distance end — its visual left — at the curve's start, so it draws.
    let arabic = falling(
        "بسم الله",
        json!({"path": {"closed": false, "points": [
        {"at": [60, 160]}, {"at": [560, 160]}]}}),
    );
    assert_eq!(
        queried(std::slice::from_ref(&arabic), 0)["text_path"]["hidden"],
        json!("none")
    );
    let [left, _, right, _] = bent_ink(arabic.clone());
    assert!(
        (55.0..75.0).contains(&left) && right < 300.0,
        "{left}..{right}"
    );
    // `end` at offset 1 puts its highest-distance end at the curve's end, whatever the
    // script.
    for text in ["بسم الله", "HELLO"] {
        let ended = with(
            falling(text, json!({"path": arabic["path"].clone()})),
            json!({"align": "end", "path_offset": 1.0}),
        );
        let [left, _, right, _] = bent_ink(ended);
        assert!(
            right > 540.0 && right < 566.0 && left > 300.0,
            "{text}: {left}..{right}"
        );
    }
}

#[test]
fn a_hidden_joined_piece_or_ligature_lists_all_its_letters() {
    let straight = json!({"closed": false, "points": [{"at": [60, 160]}, {"at": [560, 160]}]});
    // `بسم` is one joined piece of three letters, and `fi` one ligature of two: each is one
    // rigid body, so past the end it is hidden whole.
    for (text, letters) in [("بسم", json!([0, 1, 2])), ("fi", json!([0, 1]))] {
        let past = falling(text, json!({"path": straight.clone(), "path_offset": 1.0}));
        assert_eq!(
            queried(&[past], 0)["text_path"]["hidden"],
            letters,
            "{text}"
        );
        let on = falling(text, json!({"path": straight.clone()}));
        assert_eq!(
            queried(&[on], 0)["text_path"]["hidden"],
            json!("none"),
            "{text}"
        );
    }
}

#[test]
fn a_stagger_y_lifts_a_letter_along_the_normal_and_its_x_slides_it_along_the_curve() {
    // Down a vertical curve the normal points to screen left of travel for negative `y`:
    // travelling down, that is screen right.
    let held = |property: &str, v: i64| {
        json!({"by": "letter", "every": 10,
               property: [{"t": 0, "v": v}, {"t": 1000, "v": v, "ease": "linear"}]})
    };
    let [rest_left, rest_top, ..] = lit(&painted(&[falling("ABC", json!({}))], 0)).unwrap();
    let [lifted_left, lifted_top, ..] = lit(&painted(
        &[falling("ABC", json!({"units": held("y", -100)}))],
        0,
    ))
    .unwrap();
    assert!(
        (lifted_left as i64 - rest_left as i64 - 100).abs() <= 2,
        "{rest_left} → {lifted_left}"
    );
    assert!((lifted_top as i64 - rest_top as i64).abs() <= 2);
    let [slid_left, slid_top, ..] = lit(&painted(
        &[falling("ABC", json!({"units": held("x", 50)}))],
        0,
    ))
    .unwrap();
    assert!(
        (slid_top as i64 - rest_top as i64 - 50).abs() <= 2,
        "{rest_top} → {slid_top}"
    );
    assert!((slid_left as i64 - rest_left as i64).abs() <= 2);
}

#[test]
fn a_stagger_x_can_slide_a_letter_off_an_open_end() {
    let pushed = falling(
        "ABC",
        json!({"units": {"by": "letter", "every": 10, "x": [{"t": 0, "v": -400}, {"t": 1000, "v": 0, "ease": "linear"}]}}),
    );
    assert_eq!(
        queried(std::slice::from_ref(&pushed), 0)["text_path"]["hidden"],
        json!([0, 1, 2])
    );
    assert_eq!(
        queried(&[pushed], 999)["text_path"]["hidden"],
        json!("none")
    );
}

#[test]
fn motion_blur_samples_path_offset_and_a_slide_alone_is_motion() {
    let sliding = falling(
        "ABCDEF",
        json!({"path_offset": [{"t": 0, "v": 0.0}, {"t": 1000, "v": 0.6, "ease": "linear"}]}),
    );
    let blurred = with(
        sliding.clone(),
        json!({"motion_blur": {"shutter": 180, "samples": 8}}),
    );
    let still = validate(std::slice::from_ref(&blurred))
        .findings
        .iter()
        .filter(|finding| finding.code == "R-MOTION-BLUR-STILL")
        .count();
    assert_eq!(still, 0, "a slide along the curve is motion");
    assert!(painted(&[blurred], 500) != painted(&[sliding], 500));
}

// ---------------------------------------------------------------------------
// `shift` needs nothing new.
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

/// The arc with its apex keyed from box y 100 to y 61 over 0–1000 ms, and its offset keyed
/// from 0.2 to 0.6.
fn keyed_arc() -> Value {
    with(
        arc(),
        json!({"path": {"closed": false, "points": [
                   {"t": 0, "v": [{"at": [60, 240]}, {"at": [300, 100]}, {"at": [540, 240]}]},
                   {"t": 1000, "v": [{"at": [60, 240]}, {"at": [300, 61]}, {"at": [540, 240]}],
                    "ease": "linear"}]},
               "path_offset": [{"t": 0, "v": 0.2}, {"t": 1000, "v": 0.6, "ease": "linear"}]}),
    )
}

#[test]
fn shift_refuses_a_cut_inside_a_keyed_path_points_window_it_cannot_write_as_literals() {
    // At 500 ms the apex is at box y 80.5: no integer pixel.
    let path = project(&scratch(), &[keyed_arc()]);
    let before = std::fs::read_to_string(&path).unwrap();
    let answer = shifted(&path, 500);
    let refusals: Vec<_> = answer
        .report()
        .findings
        .iter()
        .filter(|finding| finding.code == "E-SHIFT-SPLIT-UNWRITABLE")
        .collect();
    assert_eq!(refusals.len(), 1, "{:?}", answer.report().findings);
    assert_eq!(refusals[0].fields["property"], "path.points");
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        before,
        "nothing is written"
    );
}

#[test]
fn shift_splits_path_offset_like_any_number_and_keyed_points_where_they_are_whole() {
    // A cut at the window's start moves both lists whole.
    let path = project(&scratch(), &[keyed_arc()]);
    let answer = shifted(&path, 0);
    assert_eq!(
        answer.report().exit_code(),
        ExitCode::Ok,
        "{:?}",
        answer.report().findings
    );
    let written = common::document(&path);
    let element = &written["tracks"][0]["elements"][0];
    let offsets: Vec<(i64, f64)> = element["path_offset"]
        .as_array()
        .unwrap()
        .iter()
        .map(|record| (record["t"].as_i64().unwrap(), record["v"].as_f64().unwrap()))
        .collect();
    assert_eq!(offsets, [(100, 0.2), (1100, 0.6)]);
    let times: Vec<i64> = element["path"]["points"]
        .as_array()
        .unwrap()
        .iter()
        .map(|record| record["t"].as_i64().unwrap())
        .collect();
    assert_eq!(times, [100, 1100], "the keyed curve moves with the cut");

    // Inside the window, with whole points: the offset splits at its own resolved value.
    let whole = with(
        keyed_arc(),
        json!({"path": {"closed": false, "points": [
            {"t": 0, "v": [{"at": [60, 240]}, {"at": [300, 100]}, {"at": [540, 240]}]},
            {"t": 1000, "v": [{"at": [60, 240]}, {"at": [300, 60]}, {"at": [540, 240]}],
             "ease": "linear"}]}}),
    );
    let path = project(&scratch(), &[whole]);
    let answer = shifted(&path, 250);
    assert_eq!(
        answer.report().exit_code(),
        ExitCode::Ok,
        "{:?}",
        answer.report().findings
    );
    let written = common::document(&path);
    let element = &written["tracks"][0]["elements"][0];
    let offsets: Vec<(i64, f64)> = element["path_offset"]
        .as_array()
        .unwrap()
        .iter()
        .map(|record| (record["t"].as_i64().unwrap(), record["v"].as_f64().unwrap()))
        .collect();
    assert_eq!(offsets, [(0, 0.2), (250, 0.3), (350, 0.3), (1100, 0.6)]);
    let apex: Vec<(i64, Value)> = element["path"]["points"]
        .as_array()
        .unwrap()
        .iter()
        .map(|record| (record["t"].as_i64().unwrap(), record["v"][1]["at"].clone()))
        .collect();
    assert_eq!(
        apex,
        [
            (0, json!([300, 100])),
            (250, json!([300, 90])),
            (350, json!([300, 90])),
            (1100, json!([300, 60]))
        ]
    );
}

// ---------------------------------------------------------------------------
// The committed fixture: ADR-0161 §9's thirteen cases.
// ---------------------------------------------------------------------------

fn fixture() -> PathBuf {
    repo().join("fixtures/text-path/text-path.montagent.json")
}

/// The `text_path` reading of every bent text present at `t` in the fixture, by id.
fn fixture_readings(t: i64) -> std::collections::BTreeMap<String, Value> {
    let document = montagent_core::parse::read(&fixture()).unwrap();
    let answer = serde_json::to_value(at::at(&document, t, None)).unwrap();
    answer["stack"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            (
                row["id"].as_str().unwrap().to_string(),
                row["text_path"].clone(),
            )
        })
        .collect()
}

#[test]
fn the_fixture_validates_clean_and_paints_every_scene() {
    let report = montagent_core::validate(&fixture());
    assert_eq!(report.exit_code(), ExitCode::Ok, "{:?}", report.findings);
    assert!(report.findings.is_empty(), "{:?}", report.findings);
    for t in [700, 2200, 3600, 5000] {
        let rasters = paint_span(&fixture(), t, t + 1, Supplying::PerFrame).unwrap();
        assert!(rasters.declined.is_empty(), "{t}: {:?}", rasters.declined);
    }
}

#[test]
fn the_fixtures_cases_hide_and_show_what_they_were_accepted_for() {
    let none = json!("none");
    let at = |t: i64, id: &str| fixture_readings(t)[id]["hidden"].clone();
    // Cases 1, 3, 4, 7, 11, 12, 13: every letter on its curve.
    for (t, id) in [
        (700, "c1-arc"),
        (700, "c3-badge"),
        (1400, "c3-badge"),
        (700, "c4-arabic"),
        (2200, "c7-stroke"),
        (2200, "c11-mixed"),
        (3600, "c12-sizes"),
        (3600, "c13-end"),
        (3600, "c6-banner"),
        (5000, "c8-tight-fi"),
        (5000, "c8-tight-arabic"),
    ] {
        assert_eq!(at(t, id), none, "{id} at {t}");
    }
    // Case 2, one element under ADR-0164: on from the start, then off past the far end.
    assert_eq!(at(0, "c2-wave").as_array().map(Vec::len), Some(8));
    assert_eq!(at(1500, "c2-wave"), none);
    assert_eq!(at(2900, "c2-wave").as_array().map(Vec::len), Some(8));
    // Case 9: a line longer than its curve, open and closed, hides its tail.
    for id in ["c9-long-open", "c9-long-closed"] {
        assert!(
            at(5000, id).as_array().is_some_and(|tail| !tail.is_empty()),
            "{id}"
        );
    }
    // Case 10: keyed points change the curve's length, and the hidden set with it.
    let sets: std::collections::BTreeSet<String> = (3000..4500)
        .step_by(100)
        .map(|t| at(t, "c10-grow").to_string())
        .collect();
    assert!(sets.len() > 3, "{sets:?}");
}
