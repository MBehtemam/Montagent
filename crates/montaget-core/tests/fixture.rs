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
fn the_fixture_is_a_project_by_the_shared_structural_predicate() {
    let document = parse::read(&fixture()).expect("the fixture parses");

    assert!(document.shape().is_ok());
    assert_eq!(document.elements().count(), 60, "60 elements");
    assert_eq!(
        document.value()["tracks"].as_array().unwrap().len(),
        14,
        "14 tracks"
    );
}

#[test]
fn the_fixture_fits_the_format_s_own_types() {
    // The strict view is the whole format, and the fixture is the only evidence that it
    // describes a coherent document rather than seven ADRs that each read well alone.
    let project = parse::read(&fixture())
        .expect("the fixture parses")
        .strict()
        .expect("the fixture is a legal project");

    assert_eq!(project.fps, 25);
    assert_eq!(project.duration, Some(65216));
    assert_eq!((project.frame.width, project.frame.height), (1080, 1920));
    assert_eq!(project.tracks.len(), 14);

    // Every element carries the same shape whatever its type (`CONTEXT.md`) — which the
    // types now enforce rather than assert: `id`, `start` and `end` are not `Option`.
    let ids: Vec<&str> = project
        .tracks
        .iter()
        .flat_map(|track| track.elements.iter())
        .map(|element| element.id.as_str())
        .collect();
    assert_eq!(ids.len(), 60);
    assert_eq!(
        ids.iter().collect::<std::collections::BTreeSet<_>>().len(),
        60,
        "every element id is unique (ADR-0019)"
    );
}

#[test]
fn the_raw_value_stays_beside_the_typed_view() {
    // A check that reports on a document the types cannot hold — which is every check that
    // reports on a broken one — reads the value. It must not have to re-read the file.
    let document = parse::read(&fixture()).expect("the fixture parses");
    let first = document.elements().next().expect("an element");

    assert_eq!(first["source"], "images/05.png");
    assert_eq!(first["clip"], serde_json::json!([0, 0, 1080, 1300]));
    assert_eq!(
        document.value()["fonts"]["brand"][0]["file"],
        "fonts/OpenRunde-Bold.otf"
    );
}
