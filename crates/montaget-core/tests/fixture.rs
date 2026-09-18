//! The committed fixture as a regression guard.
//!
//! `fixtures/en-halloween-decorating/` is a real published video. Spec #168: *"it is the
//! regression guard: it is a real published video, and a check that fires on it is wrong
//! unless an ADR says otherwise."*
//!
//! Its authority stops there. The fixture is evidence that a capability is **needed**,
//! never evidence that one is **unneeded** — it carries zero layer anchors, zero
//! non-`linear` eases, zero `opacity` occurrences and zero `video` elements, and none of
//! that is a reason to drop a check.

use montaget_core::report::ExitCode;
use montaget_core::{parse, validate};
use std::path::PathBuf;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/en-halloween-decorating/en-halloween-decorating.montaget.json")
}

#[test]
fn the_fixture_parses_and_reports_clean() {
    let report = validate(&fixture());

    assert!(
        report.findings.is_empty(),
        "the fixture is a published video; a finding on it is a defect in the check: {:?}",
        report.findings
    );
    assert_eq!(report.exit_code(), ExitCode::Ok);
}

#[test]
fn the_spine_reads_the_fixtures_shape() {
    let document = parse::read(&fixture()).expect("the fixture parses");

    assert_eq!(document.fps, Some(25));
    assert_eq!(document.duration, Some(65216));
    let frame = document.frame.expect("a frame");
    assert_eq!((frame.width, frame.height), (1080, 1920));
    assert_eq!(document.tracks.len(), 14, "14 tracks");
    assert_eq!(document.elements().count(), 60, "60 elements");

    // Every element carries the same shape whatever its type (`CONTEXT.md`).
    for element in document.elements() {
        assert!(
            element.id.is_some(),
            "every element has a required unique id"
        );
        assert!(element.kind.is_some(), "every element has a type");
        assert!(
            element.start.is_some() && element.end.is_some(),
            "a time range"
        );
    }
}

#[test]
fn the_raw_value_stays_beside_the_typed_view() {
    // Later checks read fields this ticket's spine does not type — the schema layer is
    // ADR-0016/ADR-0017's, and a later ticket's. They must not have to re-read the file.
    let document = parse::read(&fixture()).expect("the fixture parses");
    let first = document.elements().next().expect("an element");

    assert_eq!(first.value["source"], "images/05.png");
    assert_eq!(first.value["clip"], serde_json::json!([0, 0, 1080, 1300]));
    assert_eq!(
        document.value["fonts"]["brand"][0]["file"],
        "fonts/OpenRunde-Bold.otf"
    );
}
