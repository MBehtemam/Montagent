//! `letter_spacing` on a `text` element (ADR-0151 §1, ADR-0153 §3–§4), through the verbs:
//! the schema, `validate`, `query --at`, `shift` and the painted frame.

use std::path::{Path, PathBuf};

use montagent_core::report::Report;
use montagent_core::validate;
use serde_json::{Value, json};

mod common;
use common::{canonical, write_project};

fn workspace(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative)
        .canonicalize()
        .expect("a committed fixture")
}

const OSWALD: &str = "fixtures/benchmark/spy-trailer/fonts/Oswald-SemiBold.ttf";
const NASKH: &str = "fixtures/letter-spacing/fonts/NotoNaskhArabic-Regular.ttf";

/// A title in Oswald with Noto Naskh Arabic behind it, centred on `x` 540.
fn title(id: &str, text: &str) -> Value {
    json!({
        "id": id,
        "type": "text",
        "start": 0,
        "end": 2000,
        "x": 540,
        "y": 400,
        "width": 1000,
        "height": 120,
        "font": "brand",
        "size": 100,
        "align": "center",
        "color": "#FFFFFF",
        "runs": [{"text": text}],
        "caption": false,
    })
}

fn project(elements: &[Value], line: u32) -> PathBuf {
    let dir = common::tempdir(line);
    let document = json!({
        "frame": {"width": 1080, "height": 1920},
        "fps": 25,
        "background": "#000000",
        "fonts": {"brand": [
            {"file": workspace(OSWALD).display().to_string()},
            {"file": workspace(NASKH).display().to_string()},
        ]},
        "tracks": [{"name": "titles", "layer": 1, "elements": elements}],
    });
    write_project(&dir, "p.montagent.json", &canonical(&document.to_string()))
}

#[track_caller]
fn report_on(elements: &[Value]) -> Report {
    validate(&project(elements, std::panic::Location::caller().line()))
}

fn codes(report: &Report) -> Vec<&str> {
    report.findings.iter().map(|f| f.code.as_str()).collect()
}

fn with(mut element: Value, key: &str, value: Value) -> Value {
    element[key] = value;
    element
}

// ---------------------------------------------------------------------------
// The schema.
// ---------------------------------------------------------------------------

#[test]
fn letter_spacing_is_a_text_field_static_or_keyed_in_integers() {
    let still = with(title("a", "TITLE"), "letter_spacing", json!(200));
    let keyed = with(
        title("b", "TITLE"),
        "letter_spacing",
        json!([{"t": 0, "v": 0}, {"t": 1000, "v": -40, "ease": "ease-out"}]),
    );
    let report = report_on(&[still, keyed]);
    assert!(
        !codes(&report).iter().any(|c| c.starts_with("E-SCHEMA")),
        "{:?}",
        codes(&report)
    );
}

#[test]
fn letter_spacing_on_a_run_is_a_schema_error() {
    // Element-level only (ADR-0151): a run may not override it.
    let mut element = title("a", "TITLE");
    element["runs"][0]["letter_spacing"] = json!(200);
    let report = report_on(&[element]);
    let unknown: Vec<_> = report
        .findings
        .iter()
        .filter(|f| f.code == "E-SCHEMA-UNKNOWN-KEY")
        .collect();
    assert_eq!(unknown.len(), 1, "{:?}", codes(&report));
    assert_eq!(unknown[0].fields["key"], "letter_spacing");
}

#[test]
fn a_fractional_letter_spacing_is_a_schema_error_static_or_keyed() {
    for value in [
        json!(12.5),
        json!([{"t": 0, "v": 0}, {"t": 1000, "v": 12.5, "ease": "linear"}]),
    ] {
        let report = report_on(&[with(title("a", "TITLE"), "letter_spacing", value.clone())]);
        assert!(
            codes(&report).contains(&"E-SCHEMA"),
            "{value}: {:?}",
            codes(&report)
        );
    }
}

// ---------------------------------------------------------------------------
// `query --at`.
// ---------------------------------------------------------------------------

#[track_caller]
fn stack_at(project: &Path, instant: i64) -> Value {
    use montagent_core::report::ExitCode;
    use montagent_core::verbs::query::{Ask, query};
    let answer = query(
        project,
        &Ask {
            at: Some(instant),
            ..Ask::default()
        },
    );
    assert_eq!(answer.report().exit_code(), ExitCode::Ok);
    answer.to_json()["query"].clone()
}

fn in_stack<'a>(view: &'a Value, id: &str) -> &'a Value {
    view["stack"]
        .as_array()
        .expect("a stack array")
        .iter()
        .find(|e| e["id"] == id)
        .unwrap_or_else(|| panic!("`{id}` is not in the presence set"))
}

fn resolved<'a>(element: &'a Value, property: &str) -> Option<&'a Value> {
    element["values"]
        .as_array()?
        .iter()
        .find(|row| row["property"] == property)
}

#[test]
fn a_centred_spaced_title_has_an_ink_box_symmetric_about_its_x() {
    // Nothing is added after the line's last grapheme, so the spaced line stays centred.
    let path = project(
        &[with(title("t", "TRACKED"), "letter_spacing", json!(200))],
        line!(),
    );
    let view = stack_at(&path, 500);
    let ink = &in_stack(&view, "t")["ink_box"];
    let left = ink["x"].as_f64().expect("an ink box");
    let right = left + ink["width"].as_f64().unwrap();
    assert!(
        ((540.0 - left) - (right - 540.0)).abs() <= 1.0,
        "{left}..{right}"
    );
}

#[test]
fn the_ink_box_measures_the_spaced_line() {
    let plain = project(&[title("t", "TRACKED")], line!());
    let spaced = project(
        &[with(title("t", "TRACKED"), "letter_spacing", json!(200))],
        line!(),
    );
    let width = |path: &Path| {
        in_stack(&stack_at(path, 500), "t")["ink_box"]["width"]
            .as_f64()
            .expect("an ink box")
    };
    // Seven graphemes, six gaps of 20 px.
    assert!((width(&spaced) - width(&plain) - 120.0).abs() < 0.01);
}

#[test]
fn query_at_prints_the_resolved_letter_spacing_never_rounded() {
    let path = project(
        &[with(
            title("t", "TRACKED"),
            "letter_spacing",
            json!([{"t": 0, "v": 0}, {"t": 1000, "v": 25, "ease": "linear"}]),
        )],
        line!(),
    );
    let view = stack_at(&path, 500);
    let row = resolved(in_stack(&view, "t"), "letter_spacing").expect("a letter_spacing row");
    assert_eq!(row["animated"], true);
    assert_eq!(row["value"], json!(12.5));
}

// ---------------------------------------------------------------------------
// The painted frame.
// ---------------------------------------------------------------------------

/// The frame at `instant`, as PNG bytes at true scale.
#[track_caller]
fn painted(project: &Path, instant: i64) -> Vec<u8> {
    use montagent_core::report::ExitCode;
    use montagent_core::verbs::frame::{Ask, frame};
    let answer = frame(
        project,
        &Ask {
            at: Some(instant),
            full: true,
            png: true,
            ..Ask::default()
        },
    );
    assert_eq!(
        answer.report().exit_code(),
        ExitCode::Ok,
        "{}",
        serde_json::to_string_pretty(&answer.to_json()).unwrap_or_default()
    );
    answer.image().expect("a picture").bytes.clone()
}

/// A tracking-out title: spacing keyed from 0 to 200 over the first second.
fn tracking_out(id: &str, text: &str) -> Value {
    with(
        title(id, text),
        "letter_spacing",
        json!([{"t": 0, "v": 0}, {"t": 1000, "v": 200, "ease": "ease-out"}]),
    )
}

#[test]
fn spacing_reaches_the_painted_frame() {
    let plain = project(&[title("t", "TRACKED")], line!());
    let spaced = project(
        &[with(title("t", "TRACKED"), "letter_spacing", json!(200))],
        line!(),
    );
    assert_ne!(painted(&plain, 500), painted(&spaced, 500));
}

#[test]
fn a_spacing_keyed_through_zero_breaks_the_ligature_even_at_zero() {
    // At `t` 0 the keyed spacing resolves to 0, yet the file has a non-zero keyframe, so
    // Oswald's `fi` is drawn as two glyphs; with no field, or a static 0, it is one.
    let none = painted(&project(&[title("t", "fi")], line!()), 0);
    let zero = painted(
        &project(
            &[with(title("t", "fi"), "letter_spacing", json!(0))],
            line!(),
        ),
        0,
    );
    let keyed = painted(&project(&[tracking_out("t", "fi")], line!()), 0);
    assert_eq!(none, zero, "a static 0 keeps the ligature");
    assert_ne!(none, keyed, "a keyed spacing breaks it at 0 too");
}

#[test]
fn keyed_spacing_paints_the_same_bytes_whatever_was_painted_before() {
    // The gating fixture (ADR-0144): keyed spacing under rotation, non-uniform scale and
    // `blur`, on a mixed Arabic and Latin line with an `fi`. Every frame is a function of
    // the file and its instant alone, so the order frames are painted in — which is what
    // parallel painters change — changes no byte.
    let mut element = tracking_out("t", "fi سلام عليكم fi");
    element["rotation"] = json!(12.0);
    element["scale"] = json!([1.3, 0.8]);
    element["effects"] = json!([{"name": "blur", "radius": 4}]);
    let path = project(&[element], line!());

    let instants = [0, 280, 520, 760, 1000, 1400];
    let forward: Vec<Vec<u8>> = instants.iter().map(|&t| painted(&path, t)).collect();
    let backward: Vec<Vec<u8>> = instants.iter().rev().map(|&t| painted(&path, t)).collect();
    for (i, t) in instants.iter().enumerate() {
        assert!(
            forward[i] == backward[instants.len() - 1 - i],
            "frame {t} differs by painting order"
        );
    }
    // And the spacing does move between them.
    assert_ne!(forward[0], forward[4]);
}

#[test]
fn query_at_prints_no_letter_spacing_where_the_element_declares_none() {
    let path = project(&[title("t", "TRACKED")], line!());
    let view = stack_at(&path, 500);
    assert!(resolved(in_stack(&view, "t"), "letter_spacing").is_none());
}
