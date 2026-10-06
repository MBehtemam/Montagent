//! The `units` block on a `text` element, and a run's `unit` override (ADR-0151 §2–§6,
//! amended by ADR-0153 §2), through the verbs: the schema, `validate`, `query --at`, `shift`,
//! the contact sheet and the painted frame.

use std::path::{Path, PathBuf};

use montagent_core::finding::Finding;
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
        "end": 3000,
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

/// A letter stagger: each letter rises 40 px and fades in over 300 ms, 40 ms apart.
fn rise() -> Value {
    json!({
        "by": "letter",
        "every": 40,
        "y": [{"t": 0, "v": 40}, {"t": 300, "v": 0, "ease": "ease-out"}],
        "opacity": [{"t": 0, "v": 0}, {"t": 300, "v": 1, "ease": "linear"}],
    })
}

fn staggered(id: &str, text: &str) -> Value {
    with(title(id, text), "units", rise())
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
        "fontVendor": {
            workspace(OSWALD).display().to_string(): {
                "licence": "OFL-1.1",
                "source": "google/fonts ofl/oswald, instanced wght=600",
                "sha256": "442420449b66e3f8a49025fbb229a8b4b1efa5f7be9458a7c3244498d34f8de9",
            },
            workspace(NASKH).display().to_string(): {
                "licence": "OFL-1.1",
                "source": "notofonts/arabic NotoNaskhArabic-v2.019",
                "sha256": "eb5cde7fecba8c6a481039257fe02d5fd69b7b0e36f8afc56ee22f4b1d7e8c21",
            },
        },
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

fn found<'a>(report: &'a Report, code: &str) -> Vec<&'a Finding> {
    report.findings.iter().filter(|f| f.code == code).collect()
}

fn with(mut element: Value, key: &str, value: Value) -> Value {
    element[key] = value;
    element
}

/// The element's runs replaced by `runs`.
fn runs(element: Value, runs: Value) -> Value {
    with(element, "runs", runs)
}

// ---------------------------------------------------------------------------
// The schema.
// ---------------------------------------------------------------------------

#[test]
fn a_units_block_and_a_run_override_are_part_of_the_format() {
    let element = runs(
        staggered("t", ""),
        json!([
            {"text": "TIT"},
            {"text": "L", "unit": {"delay": 400, "y": [{"t": 400, "v": 80}, {"t": 900, "v": 0, "ease": "ease-out"}]}},
            {"text": "E"},
        ]),
    );
    let report = report_on(&[element]);
    assert!(
        !codes(&report).iter().any(|c| c.starts_with("E-")),
        "{:?}",
        codes(&report)
    );
}

#[test]
fn every_value_of_by_order_and_origin_is_accepted() {
    let mut elements = Vec::new();
    for (i, (by, order, origin)) in [
        ("letter", "forward", "center"),
        ("word", "reverse", "bottom-left"),
        ("line", "forward", "top-right"),
    ]
    .into_iter()
    .enumerate()
    {
        let mut units = rise();
        units["by"] = json!(by);
        units["order"] = json!(order);
        units["origin"] = json!(origin);
        let mut element = with(title(&format!("t{i}"), "ONE TWO"), "units", units);
        element["start"] = json!(i as i64 * 4000);
        element["end"] = json!(i as i64 * 4000 + 3000);
        elements.push(element);
    }
    let report = report_on(&elements);
    assert!(
        !codes(&report).iter().any(|c| c.starts_with("E-SCHEMA")),
        "{:?}",
        codes(&report)
    );
}

#[test]
fn a_units_block_needs_by_a_positive_every_and_at_least_one_list() {
    let mut no_by = rise();
    no_by.as_object_mut().unwrap().remove("by");
    let mut zero_every = rise();
    zero_every["every"] = json!(0);
    let mut fractional_every = rise();
    fractional_every["every"] = json!(12.5);
    let no_list = json!({"by": "letter", "every": 40});
    let mut unknown = rise();
    unknown["color"] = json!("#FF0000");
    for units in [no_by, zero_every, fractional_every, no_list, unknown] {
        let report = report_on(&[with(title("t", "TITLE"), "units", units.clone())]);
        assert!(
            codes(&report).iter().any(|c| c.starts_with("E-SCHEMA")),
            "{units}: {:?}",
            codes(&report)
        );
    }
}

#[test]
fn a_run_override_needs_one_key_and_a_delay_that_is_not_negative() {
    for unit in [
        json!({}),
        json!({"delay": -10}),
        json!({"color": "#FFFFFF"}),
    ] {
        let element = runs(
            staggered("t", ""),
            json!([{"text": "A"}, {"text": "B", "unit": unit.clone()}]),
        );
        let report = report_on(&[element]);
        assert!(
            codes(&report).iter().any(|c| c.starts_with("E-SCHEMA")),
            "{unit}: {:?}",
            codes(&report)
        );
    }
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

#[test]
fn an_idle_units_block_paints_the_same_bytes_as_no_block() {
    // Oswald's `TITLE` has no optional ligature, so `by: letter` switching them off changes
    // no glyph; once every unit has landed its lists rest at 0 and 1, and no layer is made.
    let plain = project(&[title("t", "TITLE")], line!());
    let idle = project(&[staggered("t", "TITLE")], line!());
    assert!(painted(&plain, 2500) == painted(&idle, 2500));
}

#[test]
fn a_mid_cascade_frame_shows_the_letters_in_motion() {
    let plain = project(&[title("t", "TITLE")], line!());
    let moving = project(&[staggered("t", "TITLE")], line!());
    let settled = painted(&moving, 2500);
    let mid = painted(&moving, 150);
    assert!(mid != settled, "mid-cascade differs from the landed title");
    assert!(mid != painted(&plain, 150));
}

#[test]
fn a_letter_stagger_switches_the_optional_ligatures_off() {
    // Oswald's `fi` is one glyph; under `by: letter` it is two, landed or not.
    let plain = project(&[title("t", "fi")], line!());
    let letters = project(&[staggered("t", "fi")], line!());
    let mut words = staggered("t", "fi");
    words["units"]["by"] = json!("word");
    let words = project(&[words], line!());
    assert!(painted(&plain, 2500) != painted(&letters, 2500));
    assert!(painted(&plain, 2500) == painted(&words, 2500));
}

#[test]
fn a_unit_offset_draws_where_the_same_offset_on_the_element_draws() {
    // `x` and `y` are element pixels: a static unit offset of (12, 30) on every letter puts
    // the title where moving the element by (12, 30) does, to the byte.
    let mut offset = with(
        title("t", "TITLE"),
        "units",
        json!({"by": "letter", "every": 40, "x": 12, "y": 30}),
    );
    offset["id"] = json!("t");
    let mut moved = title("t", "TITLE");
    moved["x"] = json!(552);
    moved["y"] = json!(430);
    let a = project(&[offset], line!());
    let b = project(&[moved], line!());
    assert!(painted(&a, 500) == painted(&b, 500));
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

/// `TITLE` rising letter by letter, with `L` singled out to drop from higher and later.
fn singled_out_l() -> Value {
    runs(
        staggered("t", ""),
        json!([
            {"text": "TIT"},
            {"text": "L", "unit": {"y": [{"t": 0, "v": 80}, {"t": 1000, "v": 0, "ease": "linear"}]}},
            {"text": "E"},
        ]),
    )
}

#[test]
fn query_at_summarises_the_stagger_and_lists_every_unit() {
    let path = project(&[singled_out_l()], line!());
    let view = stack_at(&path, 100);
    let element = in_stack(&view, "t");
    assert_eq!(element["stagger"]["by"], "letter");
    assert_eq!(element["stagger"]["count"], 5);
    // The block's lists end at 160 + 300 for the last letter; `L`'s own `y` ends at 1000.
    assert_eq!(element["stagger"]["window"], json!([0, 1000]));
    let units = element["units"].as_array().expect("a units array");
    assert_eq!(units.len(), 5, "every unit, whatever its state");
    let texts: Vec<&str> = units.iter().map(|u| u["text"].as_str().unwrap()).collect();
    assert_eq!(texts, ["T", "I", "T", "L", "E"]);
    let delays: Vec<i64> = units.iter().map(|u| u["delay"].as_i64().unwrap()).collect();
    assert_eq!(delays, [0, 40, 80, 120, 160]);
    let runs: Vec<i64> = units.iter().map(|u| u["run"].as_i64().unwrap()).collect();
    assert_eq!(runs, [0, 0, 0, 1, 2]);
}

#[test]
fn query_at_flags_each_overridden_property_and_resolves_every_unit() {
    let path = project(&[singled_out_l()], line!());
    let view = stack_at(&path, 100);
    let units = in_stack(&view, "t")["units"].clone();
    // `L` overrides `y` only: its `opacity` and its delay are still the block's.
    assert_eq!(
        units[3]["overridden"],
        json!({"y": true, "opacity": false, "delay": false})
    );
    assert_eq!(
        units[0]["overridden"],
        json!({"y": false, "opacity": false, "delay": false})
    );
    // `T` (delay 0) is a third of the way in: 100 / 300 of a linear fade, and `ease-out`
    // on `y`. `L`'s own `y` is never delayed: 80 → 0 linear over a second, 72 at 100.
    let close = |v: &Value, want: f64| (v.as_f64().unwrap() - want).abs() < 1e-6;
    assert!(close(&units[0]["opacity"], 1.0 / 3.0), "{}", units[0]);
    assert!(close(&units[3]["y"], 72.0), "{}", units[3]);
    // `L`'s opacity waits for its delay of 120, so at 100 it has not started.
    assert!(close(&units[3]["opacity"], 0.0), "{}", units[3]);
    // `E`, delay 160, has not started either: the first record holds.
    assert!(close(&units[4]["y"], 40.0), "{}", units[4]);
    for unit in units.as_array().unwrap() {
        assert_eq!(unit["x"], json!(0.0));
        assert_eq!(unit["rotation"], json!(0.0));
        assert_eq!(unit["scale"], json!([1.0, 1.0]));
        assert_eq!(unit["merged_with"], json!([]));
    }
}

#[test]
fn query_at_names_the_units_a_joined_arabic_piece_moves_with() {
    let path = project(&[staggered("t", "السلام")], line!());
    let units = in_stack(&stack_at(&path, 100), "t")["units"].clone();
    assert_eq!(units.as_array().unwrap().len(), 6);
    assert_eq!(units[0]["merged_with"], json!([]));
    assert_eq!(units[2]["merged_with"], json!([1, 3, 4]));
    // A piece moves on its first letter's timing: `س` (delay 80) is drawn with `ل` (40).
    assert_eq!(units[2]["opacity"], units[1]["opacity"]);
}

#[test]
fn query_at_gives_a_plain_title_no_stagger() {
    let path = project(&[title("t", "TITLE")], line!());
    let element = in_stack(&stack_at(&path, 100), "t").clone();
    assert!(element.get("stagger").is_none() && element.get("units").is_none());
}
