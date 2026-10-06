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
    // And `query --at` reports the pose that frame was drawn with.
    for unit in in_stack(&stack_at(&a, 500), "t")["units"]
        .as_array()
        .unwrap()
    {
        assert_eq!((&unit["x"], &unit["y"]), (&json!(12.0), &json!(30.0)));
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

// ---------------------------------------------------------------------------
// `validate`: the three override errors, and the merged-unit review.
// ---------------------------------------------------------------------------

/// The prose a finding renders to, so every template placeholder is proved filled.
fn prose(report: &Report) -> String {
    montagent_core::text::render(
        &report.to_json(),
        montagent_core::text::Options {
            verbose: true,
            ..montagent_core::text::Options::default()
        },
    )
    .expect("the report renders")
}

#[test]
fn a_valid_split_is_silent() {
    let report = report_on(&[singled_out_l()]);
    for code in [
        "E-UNIT-RUN-NOT-ONE-UNIT",
        "E-UNIT-RUN-UNDECLARED",
        "E-UNIT-RUN-MERGED",
        "R-UNIT-MERGED",
    ] {
        assert!(
            found(&report, code).is_empty(),
            "{code}: {:?}",
            codes(&report)
        );
    }
}

#[test]
fn an_override_on_a_run_holding_other_than_one_unit_is_an_error_naming_the_count() {
    let override_on = |text: &str| {
        runs(
            staggered("t", ""),
            json!([{"text": "AB"}, {"text": text, "unit": {"delay": 300}}, {"text": "Z"}]),
        )
    };
    for (text, count) in [("CD", 2), (" ", 0)] {
        let report = report_on(&[override_on(text)]);
        let errors = found(&report, "E-UNIT-RUN-NOT-ONE-UNIT");
        assert_eq!(errors.len(), 1, "{text:?}: {:?}", codes(&report));
        assert_eq!(errors[0].class, montagent_core::finding::Class::Error);
        assert_eq!(errors[0].fields["found"], count, "{text:?}");
        assert_eq!(errors[0].fields["run"], 1);
    }
    // Under `by: word`, part of a word is part of a unit: one unit touched, not held.
    let mut words = runs(
        staggered("t", ""),
        json!([{"text": "Hel"}, {"text": "lo,", "unit": {"delay": 300}}, {"text": " you"}]),
    );
    words["units"]["by"] = json!("word");
    let report = report_on(&[words.clone()]);
    let errors = found(&report, "E-UNIT-RUN-NOT-ONE-UNIT");
    assert_eq!(errors.len(), 1, "{:?}", codes(&report));
    assert_eq!(errors[0].fields["found"], 1);
    // The whole word with its comma is one unit.
    words["runs"] = json!([{"text": "Hello,", "unit": {"delay": 300}}, {"text": " you"}]);
    assert!(found(&report_on(&[words]), "E-UNIT-RUN-NOT-ONE-UNIT").is_empty());
    assert!(prose(&report).contains("E-UNIT-RUN-NOT-ONE-UNIT"));
}

#[test]
fn an_override_on_an_element_with_no_units_block_is_an_error() {
    let element = runs(
        title("t", ""),
        json!([{"text": "A"}, {"text": "B", "unit": {"delay": 300}}]),
    );
    let report = report_on(&[element]);
    assert_eq!(
        found(&report, "E-UNIT-RUN-NOT-ONE-UNIT").len(),
        1,
        "{:?}",
        codes(&report)
    );
}

#[test]
fn an_override_naming_a_list_the_block_does_not_declare_is_an_error() {
    let element = runs(
        staggered("t", ""),
        json!([
            {"text": "A"},
            {"text": "B", "unit": {"x": [{"t": 0, "v": 30}, {"t": 300, "v": 0, "ease": "linear"}]}},
        ]),
    );
    let report = report_on(&[element]);
    let errors = found(&report, "E-UNIT-RUN-UNDECLARED");
    assert_eq!(errors.len(), 1, "{:?}", codes(&report));
    assert_eq!(errors[0].fields["property"], "x");
    assert!(prose(&report).contains("E-UNIT-RUN-UNDECLARED"));
}

#[test]
fn an_arabic_letter_stagger_names_its_joined_letters_for_review() {
    let report = report_on(&[staggered("t", "السلام عليكم")]);
    let reviews = found(&report, "R-UNIT-MERGED");
    assert_eq!(reviews.len(), 1, "{:?}", codes(&report));
    assert_eq!(reviews[0].class, montagent_core::finding::Class::Review);
    assert_eq!(reviews[0].fields["how"], "joined");
    assert_eq!(
        reviews[0].fields["groups"],
        json!([[1, 2, 3, 4], [6, 7, 8, 9, 10]])
    );
    assert!(prose(&report).contains("by"), "{}", prose(&report));
    // Under `by: word` nothing moves with anything else.
    let mut words = staggered("t", "السلام عليكم");
    words["units"]["by"] = json!("word");
    assert!(found(&report_on(&[words]), "R-UNIT-MERGED").is_empty());
}

#[test]
fn singling_out_a_joined_letter_is_an_error_and_a_lone_letter_is_not() {
    // `ا` joins nothing, so it is a piece of its own; `س` is inside `لسلا`.
    let split = |single: &str| -> Value {
        let parts: Vec<Value> = ["ا", "ل", "س", "لام"]
            .iter()
            .map(|part| {
                if *part == single {
                    json!({"text": part, "unit": {"delay": 400}})
                } else {
                    json!({"text": part})
                }
            })
            .collect();
        runs(staggered("t", ""), Value::Array(parts))
    };
    let joined = report_on(&[split("س")]);
    let errors = found(&joined, "E-UNIT-RUN-MERGED");
    assert_eq!(errors.len(), 1, "{:?}", codes(&joined));
    assert_eq!(errors[0].fields["unit"], 2);
    let lone = report_on(&[split("ا")]);
    assert!(
        found(&lone, "E-UNIT-RUN-MERGED").is_empty(),
        "{:?}",
        codes(&lone)
    );
}

// ---------------------------------------------------------------------------
// `N-CAPTION-SETTLES` (ADR-0151 §6).
// ---------------------------------------------------------------------------

/// A staggered caption: the caption checks run on it.
fn caption(element: Value) -> Value {
    let mut element = element;
    element.as_object_mut().unwrap().remove("caption");
    element
}

#[test]
fn a_staggered_caption_says_when_it_settles_counting_an_override_that_ends_last() {
    // `L`'s own `y` ends at 1000, later than the block's last unit at 160 + 300.
    let report = report_on(&[caption(singled_out_l())]);
    let notes = found(&report, "N-CAPTION-SETTLES");
    assert_eq!(notes.len(), 1, "{:?}", codes(&report));
    assert_eq!(notes[0].class, montagent_core::finding::Class::Note);
    assert_eq!(notes[0].fields["settles"], 1000);
    assert_eq!(notes[0].fields["never"], false);
    assert!(prose(&report).contains("1000 ms"));
}

#[test]
fn a_caption_whose_stagger_lands_at_or_after_its_end_never_settles() {
    let mut element = caption(staggered("t", "TITLE"));
    element["end"] = json!(400);
    let report = report_on(&[element]);
    let notes = found(&report, "N-CAPTION-SETTLES");
    assert_eq!(notes.len(), 1, "{:?}", codes(&report));
    assert_eq!(notes[0].fields["settles"], 460);
    assert_eq!(notes[0].fields["never"], true);
    assert!(
        prose(&report).contains("never settles"),
        "{}",
        prose(&report)
    );
}

#[test]
fn a_title_that_is_not_a_caption_gets_no_settle_note() {
    let report = report_on(&[staggered("t", "TITLE")]);
    assert!(found(&report, "N-CAPTION-SETTLES").is_empty());
}

// ---------------------------------------------------------------------------
// `R-OFF-CANVAS` widens by the unit offsets (ADR-0151 §5).
// ---------------------------------------------------------------------------

/// A title whose declared box sits wholly past the frame's right edge.
fn past_the_right_edge() -> Value {
    let mut element = title("t", "TITLE");
    element["x"] = json!(1080 + 600);
    element
}

#[test]
fn a_unit_offset_that_reaches_the_frame_keeps_the_element_on_canvas() {
    assert_eq!(
        found(&report_on(&[past_the_right_edge()]), "R-OFF-CANVAS").len(),
        1
    );
    let reaching = with(
        past_the_right_edge(),
        "units",
        json!({"by": "letter", "every": 40,
               "x": [{"t": 0, "v": -200}, {"t": 300, "v": 0, "ease": "linear"}]}),
    );
    assert!(found(&report_on(&[reaching]), "R-OFF-CANVAS").is_empty());
}

#[test]
fn an_off_canvas_stagger_names_the_list_that_widened_each_edge_and_what_was_not_checked() {
    // Every unit list, the block's and each override's, widens the rect.
    let mut element = with(
        past_the_right_edge(),
        "units",
        json!({"by": "letter", "every": 40,
               "x": [{"t": 0, "v": 30}, {"t": 300, "v": 0, "ease": "linear"}],
               "scale": [{"t": 0, "v": [0.5, 0.5]}, {"t": 300, "v": [1.0, 1.0], "ease": "linear"}]}),
    );
    element["runs"] = json!([
        {"text": "TIT"},
        {"text": "L", "unit": {"x": [{"t": 0, "v": -50}, {"t": 300, "v": 0, "ease": "linear"}]}},
        {"text": "E"},
    ]);
    let report = report_on(&[element]);
    let off = found(&report, "R-OFF-CANVAS");
    assert_eq!(off.len(), 1, "{:?}", codes(&report));
    assert_eq!(off[0].fields["widened"]["left"], "runs[1].unit.x");
    assert_eq!(off[0].fields["widened"]["right"], "units.x");
    assert!(off[0].fields["widened"].get("top").is_none());
    let said = prose(&report);
    assert!(said.contains("runs[1].unit.x"), "{said}");
    assert!(said.contains("not checked"), "{said}");
}

// ---------------------------------------------------------------------------
// The keyframe checks walk the unit lists.
// ---------------------------------------------------------------------------

#[test]
fn an_ease_on_a_unit_list_hold_is_inert() {
    let mut element = staggered("t", "TITLE");
    element["units"]["x"] = json!([{"t": 0, "v": 10}, {"t": 300, "v": 10, "ease": "ease-in"}]);
    let report = report_on(&[element]);
    let inert = found(&report, "R-EASE-INERT");
    assert_eq!(inert.len(), 1, "{:?}", codes(&report));
    assert_eq!(inert[0].fields["property"], "units.x");
}

#[test]
fn an_ease_on_a_run_override_hold_is_inert() {
    let mut element = singled_out_l();
    element["runs"][1]["unit"]["y"] =
        json!([{"t": 0, "v": 10}, {"t": 300, "v": 10, "ease": "ease-in"}]);
    let report = report_on(&[element]);
    let inert = found(&report, "R-EASE-INERT");
    assert_eq!(inert.len(), 1, "{:?}", codes(&report));
    assert_eq!(inert[0].fields["property"], "runs[1].unit.y");
}

#[test]
fn a_stale_derived_t_on_a_unit_list_is_reported() {
    let mut element = staggered("t", "TITLE");
    element["units"]["y"] = json!([
        {"t": 0, "v": 40},
        {"t": 250, "t_from": {"rule": "after-previous", "ms": 300}, "v": 0, "ease": "ease-out"}
    ]);
    let report = report_on(&[element]);
    let stale = found(&report, "R-DERIVED-T");
    assert_eq!(stale.len(), 1, "{:?}", codes(&report));
    assert_eq!(stale[0].fields["property"], "units.y");
}

#[test]
fn unit_keyframes_out_of_order_or_missing_an_ease_are_schema_errors() {
    for y in [
        json!([{"t": 300, "v": 40}, {"t": 0, "v": 0, "ease": "linear"}]),
        json!([{"t": 0, "v": 40}, {"t": 300, "v": 0}]),
    ] {
        let mut element = staggered("t", "TITLE");
        element["units"]["y"] = y.clone();
        let report = report_on(&[element]);
        assert!(
            codes(&report).iter().any(|c| c.starts_with("E-")),
            "{y}: {:?}",
            codes(&report)
        );
    }
}

// ---------------------------------------------------------------------------
// `shift` (ADR-0151 §5).
// ---------------------------------------------------------------------------

/// `TITLE` with `L` singled out, starting at 1000 so a cut can fall before its stagger.
fn late_title() -> Value {
    let mut element = singled_out_l();
    element["start"] = json!(0);
    element["end"] = json!(4000);
    element["units"]["y"] = json!([{"t": 1000, "v": 40}, {"t": 1300, "v": 0, "ease": "ease-out"}]);
    element["units"]["opacity"] =
        json!([{"t": 1000, "v": 0}, {"t": 1300, "v": 1, "ease": "linear"}]);
    element["runs"][1]["unit"] = json!({
        "delay": 300,
        "y": [{"t": 1200, "v": 80}, {"t": 2000, "v": 0, "ease": "linear"}],
    });
    element
}

#[track_caller]
fn shifted(element: Value, at: i64) -> (montagent_core::verbs::shift::Answer, Value) {
    use montagent_core::verbs::shift::{Ask, shift};
    let path = project(&[element], std::panic::Location::caller().line());
    let answer = shift(
        &path,
        &Ask {
            at,
            delta: 500,
            scope: None,
            release: Vec::new(),
        },
    );
    let written: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    (answer, written["tracks"][0]["elements"][0].clone())
}

fn ts(list: &Value) -> Vec<i64> {
    list.as_array()
        .unwrap()
        .iter()
        .map(|r| r["t"].as_i64().unwrap())
        .collect()
}

#[test]
fn the_timeline_marks_a_title_whose_letters_move_as_moving() {
    use montagent_core::verbs::timeline;
    let still = timeline::timeline(&project(&[title("t", "TITLE")], line!())).to_json();
    let staggered = timeline::timeline(&project(&[staggered("t", "TITLE")], line!())).to_json();
    assert_ne!(still["timeline"], staggered["timeline"]);
}

#[test]
fn a_shift_cutting_inside_the_stagger_window_is_refused_naming_the_window() {
    // The window runs from 1000 to `L`'s own 2000.
    let (answer, written) = shifted(late_title(), 1500);
    let refused: Vec<_> = answer
        .report()
        .findings
        .iter()
        .filter(|f| f.code == "E-SHIFT-UNITS-WINDOW")
        .collect();
    assert_eq!(refused.len(), 1, "{:?}", answer.report().findings);
    assert_eq!(refused[0].fields["from"], 1000);
    assert_eq!(refused[0].fields["to"], 2000);
    assert_eq!(written, late_title(), "a refused shift writes nothing");
}

#[test]
fn a_shift_cutting_before_the_window_carries_every_unit_list_and_keeps_the_delays() {
    use montagent_core::report::ExitCode;
    let (answer, written) = shifted(late_title(), 500);
    assert_ne!(
        answer.report().exit_code(),
        ExitCode::Errors,
        "{:?}",
        answer.report().findings
    );
    assert_eq!(written["end"], 4500);
    assert_eq!(ts(&written["units"]["y"]), [1500, 1800]);
    assert_eq!(ts(&written["units"]["opacity"]), [1500, 1800]);
    assert_eq!(ts(&written["runs"][1]["unit"]["y"]), [1700, 2500]);
    assert_eq!(written["runs"][1]["unit"]["delay"], 300);
    assert_eq!(written["units"]["every"], 40);
}

#[test]
fn a_shift_moving_a_staggered_element_whole_carries_its_unit_lists() {
    let mut element = late_title();
    element["start"] = json!(800);
    let (_, written) = shifted(element, 0);
    assert_eq!(written["start"], 1300);
    assert_eq!(ts(&written["units"]["y"]), [1500, 1800]);
    assert_eq!(ts(&written["runs"][1]["unit"]["y"]), [1700, 2500]);
}

#[test]
fn a_shift_cutting_after_the_window_leaves_the_unit_lists_where_they_are() {
    let (_, written) = shifted(late_title(), 3000);
    assert_eq!(written["end"], 4500);
    assert_eq!(ts(&written["units"]["y"]), [1000, 1300]);
    assert_eq!(ts(&written["runs"][1]["unit"]["y"]), [1200, 2000]);
}

// ---------------------------------------------------------------------------
// The contact sheet (ADR-0151 §5, ADR-0106).
// ---------------------------------------------------------------------------

/// Every change point the sheet over `from..to` names, tiled or not.
#[track_caller]
fn sheet_points(project: &Path, from: i64, to: i64) -> Vec<String> {
    use montagent_core::report::ExitCode;
    use montagent_core::verbs::frame::{Ask, frame};
    let answer = frame(
        project,
        &Ask {
            from: Some(from),
            to: Some(to),
            ..Ask::default()
        },
    );
    assert_eq!(answer.report().exit_code(), ExitCode::Ok);
    let json = answer.to_json();
    let sheet = &json["sheet"];
    let mut points: Vec<String> = sheet["keyframes"]["untiled_points"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["point"].as_str().unwrap().to_string())
        .collect();
    for tile in sheet["provenance"].as_array().unwrap() {
        for point in tile["keyframes"].as_array().into_iter().flatten() {
            points.push(point.as_str().unwrap().to_string());
        }
    }
    points.sort();
    points
}

#[test]
fn the_sheet_names_the_first_and_last_scheduled_units_and_every_override_list() {
    // Five letters 40 ms apart from 1000; `L` (unit 3) is overridden and ends last, at 2000,
    // so it is the last scheduled unit, not `E` (unit 4).
    let path = project(&[late_title()], line!());
    let points = sheet_points(&path, 0, 4000);
    let mut expected = vec![
        "t.units[0].opacity@1000",
        "t.units[0].opacity@1300",
        "t.units[0].y@1000",
        "t.units[0].y@1300",
        // `L`: its own `y` on the absolute clock, its block `opacity` at its delay of 300.
        "t.units[3].opacity@1300",
        "t.units[3].opacity@1600",
        "t.units[3].y@1200",
        "t.units[3].y@2000",
    ];
    expected.sort();
    assert_eq!(points, expected);
}
