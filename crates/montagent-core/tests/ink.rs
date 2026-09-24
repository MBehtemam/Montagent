//! `R-LINE-INK-COLLISION` and `measure`'s ink extents (#325, ADR-0087).
//!
//! # Why the assertions here are not Thai
//!
//! ADR-0087's measurements were taken on Noto Sans Thai and Sarabun, and reproducing them
//! is the job of its committed probe (`docs/research/prototypes/thai-vertical-metrics/`),
//! which fetches both faces by hash and re-asserts every number it published. Those files
//! are fetched, not vendored — the repo commits exactly one font, `OpenRunde-Bold.otf`
//! (#143) — so a test binary that asserted a Thai figure would fail on any checkout that
//! had not run the probe first.
//!
//! What *is* testable here without a second font is everything that is not face-specific:
//! that the ink is read from the glyphs rather than from the slot, that the seam's sign
//! convention is the one ADR-0087 published, that a collision fires a `review` and a
//! comfortable `line_height` fires nothing, and that a line which draws nothing has no ink
//! and
//! cannot collide. Those hold in any face. The collision is provoked by squeezing
//! `line_height` in a Latin face rather than by stacking marks in a Thai one — a different
//! route to the same geometry, which is the point: ADR-0087's finding is that the floor is
//! a property of the face, so nothing about this check is Thai-specific either.

use std::path::{Path, PathBuf};

use montagent_core::finding::{Class, Finding};
use montagent_core::parse;
use montagent_core::report::Report;
use montagent_core::verbs::measure::{Answer, Ask};
use serde_json::{Value, json};

mod common;
use common::{canonical, write_project};

// ---------------------------------------------------------------------------
// The harness.
// ---------------------------------------------------------------------------

/// The one font this repo commits (#143), used here as `tests/measure.rs` uses it: for its
/// real metrics, never for what the fixture declares about it.
fn font_file() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/en-halloween-decorating/fonts/OpenRunde-Bold.otf")
        .canonicalize()
        .expect("the vendored font (#143)")
}

/// A text element in the shape both `measure` and `validate` read it in.
///
/// Composed with `json!` rather than by interpolating into a JSON string literal, so a
/// caller writes a real `\n` and `serde_json` escapes it on the way to the file. ADR-0008
/// makes that character the format's only line break.
fn text(id: &str, size: i64, line_height: &str, text: &str) -> Value {
    json!({
        "id": id,
        "type": "text",
        "start": 0,
        "end": 1000,
        "x": 0,
        "y": 0,
        "width": 900,
        "height": 400,
        "font": "brand",
        "size": size,
        "line_height": serde_json::from_str::<Value>(line_height).expect("a number"),
        "runs": [{"text": text}],
    })
}

/// A project declaring the one font chain, with `elements` in a single track.
fn project(elements: &[Value], line: u32) -> PathBuf {
    let dir = common::tempdir(line);
    let document = json!({
        "frame": {"width": 1080, "height": 1920},
        "fps": 25,
        "fonts": {"brand": [{"file": font_file().display().to_string()}]},
        "tracks": [{"elements": elements}],
    });
    write_project(&dir, "p.montagent.json", &canonical(&document.to_string()))
}

/// `R-LINE-INK-COLLISION` alone, over a project written to a scratch directory.
///
/// The check is called directly rather than through the whole verb, `tests/box_slack.rs`'s
/// pattern: no element here carries a `source`, so the disk half of `validate` has nothing
/// to probe and this test binary never needs an `ffprobe`.
#[track_caller]
fn findings_for(elements: &[Value]) -> Vec<Finding> {
    let path = project(elements, std::panic::Location::caller().line());
    let document = parse::read(&path).expect("the project parses");
    let mut report = Report::new("validate", Some(document.path().to_string()));
    montagent_core::checks::ink::check(&document, &mut report);
    report
        .findings
        .into_iter()
        .filter(|f| f.code == "R-LINE-INK-COLLISION")
        .collect()
}

#[track_caller]
fn measure_one(element: Value) -> Value {
    let path = project(&[], std::panic::Location::caller().line());
    let answer: Answer = montagent_core::verbs::measure::measure(
        &path,
        &Ask {
            element: Some(element),
            at: None,
            elements: None,
            all: false,
        },
    );
    answer.to_json()["measure"].clone()
}

// ---------------------------------------------------------------------------
// `measure`'s ink extents.
// ---------------------------------------------------------------------------

#[test]
fn a_line_reports_ink_inside_its_own_slot_for_an_ordinary_latin_line_height() {
    let measured = measure_one(text("t", 55, "1.2", "Hamburgefonstiv"));
    let line = &measured["lines"][0];

    let ink_top = line["ink_top"].as_f64().expect("the line draws something");
    let ink_bottom = line["ink_bottom"].as_f64().expect("ink");
    let baseline = line["baseline_y"].as_f64().expect("a baseline");

    // The ink straddles the baseline: caps and ascenders above it, the descender of `g`
    // below. Asserted as a relationship rather than as two figures, because the figures are
    // this face's and this test is not about this face.
    assert!(ink_top < baseline, "ink_top {ink_top} baseline {baseline}");
    assert!(
        ink_bottom > baseline,
        "ink_bottom {ink_bottom} baseline {baseline}"
    );
}

#[test]
fn the_ink_is_the_glyphs_and_not_the_slot() {
    // The whole point of ADR-0087: these two numbers are derived from different things, so
    // they must be free to differ. `size × line_height` knows nothing about a contour.
    let measured = measure_one(text("t", 55, "1.2", "Hamburgefonstiv"));
    let line = &measured["lines"][0];
    let slot_top = line["slot_top"].as_f64().expect("a slot");
    let slot_bottom = slot_top + line["slot_height"].as_f64().expect("a slot height");
    let ink_top = line["ink_top"].as_f64().expect("ink");
    let ink_bottom = line["ink_bottom"].as_f64().expect("ink");

    assert!(
        (ink_top - slot_top).abs() > f64::EPSILON
            || (ink_bottom - slot_bottom).abs() > f64::EPSILON,
        "ink [{ink_top}, {ink_bottom}] is exactly the slot [{slot_top}, {slot_bottom}], \
         which would mean it was read from the declared numbers"
    );
}

#[test]
fn a_blank_line_has_no_ink_and_so_no_seam_of_its_own() {
    // Three lines, the middle one empty. The blank reserves its slot exactly as before —
    // that is ADR-0007's rule and ADR-0087 does not touch it — and simply has nothing to
    // report, which is a different fact from ink of zero height at the baseline.
    let measured = measure_one(text("t", 55, "1.2", "one\n\nthree"));
    assert_eq!(measured["line_count"], json!(3));
    assert!(measured["lines"][1]["ink_top"].is_null());
    assert!(measured["lines"][1]["ink_bottom"].is_null());
    assert!(measured["lines"][1]["slot_height"].as_f64().unwrap() > 0.0);

    // One seam, and it spans the blank: lines 0 and 2 are the two things that are drawn.
    let seams = measured["ink_seams"].as_array().expect("seams");
    assert_eq!(seams.len(), 1);
    assert_eq!(seams[0]["above"], json!(0));
    assert_eq!(seams[0]["below"], json!(2));
}

#[test]
fn a_single_line_block_has_no_seam() {
    let measured = measure_one(text("t", 55, "1.2", "one line"));
    assert_eq!(measured["ink_seams"].as_array().expect("an array").len(), 0);
}

#[test]
fn a_seam_is_negative_when_the_lines_clear_and_positive_when_they_collide() {
    // ADR-0087's sign convention, which the research doc's tables also use: positive is
    // the defect. Same text, same face, same size — only `line_height` moves.
    let roomy = measure_one(text("t", 55, "2.0", "Hamburgefonstiv\nHamburgefonstiv"));
    let squeezed = measure_one(text("t", 55, "0.4", "Hamburgefonstiv\nHamburgefonstiv"));

    let roomy_overlap = roomy["ink_seams"][0]["overlap"].as_f64().expect("a seam");
    let squeezed_overlap = squeezed["ink_seams"][0]["overlap"]
        .as_f64()
        .expect("a seam");

    assert!(
        roomy_overlap < 0.0,
        "at 2.0 the lines clear: {roomy_overlap}"
    );
    assert!(
        squeezed_overlap > 0.0,
        "at 0.4 the lines collide: {squeezed_overlap}"
    );
}

#[test]
fn tightening_line_height_by_one_tenth_closes_the_seam_by_exactly_that_slot() {
    // The seam moves with the slot and with nothing else, which is what makes the number
    // an author can act on: ADR-0028's tenths of `size` are exactly what a line's slot
    // gains, so a tenth of 55 px is 5.5 px of seam. This is the arithmetic underneath
    // ADR-0087's per-tenth tables, asserted without their font.
    let at = |line_height: &str| {
        measure_one(text(
            "t",
            55,
            line_height,
            "Hamburgefonstiv\nHamburgefonstiv",
        ))["ink_seams"][0]["overlap"]
            .as_f64()
            .expect("a seam")
    };
    assert!((at("1.1") - at("1.2") - 5.5).abs() < 1e-9);
    assert!((at("1.2") - at("1.3") - 5.5).abs() < 1e-9);
}

// ---------------------------------------------------------------------------
// The check.
// ---------------------------------------------------------------------------

#[test]
fn an_ordinary_line_height_in_the_vendored_face_fires_nothing() {
    let findings = findings_for(&[text("body", 55, "1.2", "Hamburgefonstiv\nHamburgefonstiv")]);
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn a_line_height_that_puts_one_lines_ink_through_the_next_fires_a_review() {
    let findings = findings_for(&[text("body", 55, "0.4", "Hamburgefonstiv\nHamburgefonstiv")]);
    assert_eq!(findings.len(), 1, "{findings:?}");

    let finding = &findings[0];
    // ADR-0087 is explicit that this may never be an `error`: a deliberately tight
    // `line_height` is a real typographic choice, and `error` is reserved for guaranteed
    // wrong.
    assert_eq!(finding.class, Class::Review);
    assert_eq!(finding.location.element.as_deref(), Some("body"));
    assert_eq!(finding.fields["element"], json!("body"));
    assert_eq!(finding.fields["font"], json!("brand"));
    assert_eq!(finding.fields["size"], json!(55));
    assert_eq!(finding.fields["line_height"], json!("0.4"));
    assert_eq!(finding.fields["above"], json!(0));
    assert_eq!(finding.fields["below"], json!(1));
    assert!(finding.fields["overlap"].as_f64().expect("an overlap") > 0.0);
    // `review` carries no repair, ADR-0043: the field is an axis of `error` alone, and
    // ADR-0087 leaves two repairs open in any case.
    assert!(finding.repair.is_none());
}

#[test]
fn a_single_line_element_can_never_fire() {
    // However tight the `line_height`: there is no second line for the first one's ink to
    // reach.
    let findings = findings_for(&[text("body", 55, "0.1", "Hamburgefonstiv")]);
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn one_finding_per_element_naming_the_worst_seam_and_counting_the_rest() {
    // ADR-0006's noise budget. Four lines collide at three seams by nearly the same amount,
    // and three findings saying so tell a reader nothing the first did not.
    let findings = findings_for(&[text(
        "body",
        55,
        "0.4",
        "Hamburgefonstiv\nHamburgefonstiv\nHamburgefonstiv\nHamburgefonstiv",
    )]);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].fields["collisions"], json!("3 seams"));
}

#[test]
fn each_colliding_element_gets_its_own_finding() {
    let findings = findings_for(&[
        text("first", 55, "0.4", "Hamburgefonstiv\nHamburgefonstiv"),
        text("second", 40, "1.4", "Hamburgefonstiv\nHamburgefonstiv"),
        text("third", 55, "0.5", "Hamburgefonstiv\nHamburgefonstiv"),
    ]);
    let named: Vec<_> = findings
        .iter()
        .map(|f| f.fields["element"].as_str().expect("an element"))
        .collect();
    assert_eq!(named, ["first", "third"]);
}

#[test]
fn an_element_whose_font_key_the_project_does_not_declare_is_silent_here() {
    // The dangling-`font`-key gap `crate::checks::fonts` names by hand. It is not this
    // check's to report, and a second differently-worded complaint about it would be the
    // drift ADR-0006 forbids — so the element is skipped, not guessed at.
    let mut element = text("body", 55, "0.4", "Hamburgefonstiv\nHamburgefonstiv");
    element["font"] = json!("no-such-chain");
    assert!(findings_for(&[element]).is_empty());
}

#[test]
fn a_non_text_element_is_not_measured_at_all() {
    let mut element = text("card", 55, "0.4", "Hamburgefonstiv\nHamburgefonstiv");
    element["type"] = json!("rect");
    assert!(findings_for(&[element]).is_empty());
}

#[test]
fn the_finding_renders_through_its_registered_template() {
    // ADR-0006: the prose is generated from the canonical JSON, so every field the template
    // names must be one the check actually supplies. A missing one surfaces here rather
    // than in a user's terminal.
    let findings = findings_for(&[text("body", 55, "0.4", "Hamburgefonstiv\nHamburgefonstiv")]);
    let mut report = Report::new("validate", Some("p.montagent.json".into()));
    report.push(findings.into_iter().next().expect("a finding"));

    let prose =
        montagent_core::text::render(&report.to_json(), montagent_core::text::Options::verbose())
            .expect("the template renders");
    assert!(prose.contains("R-LINE-INK-COLLISION"), "{prose}");
    assert!(prose.contains("body"), "{prose}");
    assert!(prose.contains("line_height 0.4"), "{prose}");
    assert!(!prose.contains('{'), "an unfilled placeholder: {prose}");
}

#[test]
fn the_committed_fixture_has_no_ink_collision() {
    // The fixture is 22 text elements of Latin at the `line_height`s its author chose. It is
    // not
    // the oracle for an extent (`tests/measure.rs` says why), but it is a real project, and
    // a check that fired on it would be one no author could ignore into usefulness.
    let document = parse::read(&common::fixture_project()).expect("the fixture parses");
    let mut report = Report::new("validate", Some(document.path().to_string()));
    montagent_core::checks::ink::check(&document, &mut report);
    let fired: Vec<_> = report
        .findings
        .iter()
        .filter(|f| f.code == "R-LINE-INK-COLLISION")
        .collect();
    assert!(fired.is_empty(), "{fired:?}");
}

#[test]
fn validate_runs_the_check_as_part_of_the_whole_engine() {
    // ADR-0006: `validate` always runs every check on the whole project. Called through the
    // verb rather than directly, so a check wired into `checks/mod.rs` but never called from
    // `run_checks` is caught.
    let path = project(
        &[text("body", 55, "0.4", "Hamburgefonstiv\nHamburgefonstiv")],
        std::panic::Location::caller().line(),
    );
    let report = montagent_core::verbs::validate::validate(&path);
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.code == "R-LINE-INK-COLLISION"),
        "{:?}",
        report.findings.iter().map(|f| &f.code).collect::<Vec<_>>()
    );
}
