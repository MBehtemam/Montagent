//! `E-TRANSITION-RANGE` (#202, ADR-0059): document-only, so `check` is called directly and
//! no project here ever needs `ffprobe`.

use montaget_core::finding::Class;
use montaget_core::report::Report;
use montaget_core::{parse, validate};

mod common;
use common::{canonical, write_project};

fn project(tracks: &str) -> String {
    canonical(&format!(
        r##"{{"frame":{{"width":1080,"height":1920}},"fps":25,"tracks":[{tracks}]}}"##
    ))
}

fn track(elements: &str) -> String {
    format!(r##"{{"elements":[{elements}]}}"##)
}

fn clip(id: &str, start: i64, end: i64) -> String {
    format!(
        r##"{{"id":"{id}","type":"rect","start":{start},"end":{end},"x":0,"y":0,"width":900,"height":100}}"##
    )
}

fn transition(id: &str, start: i64, end: i64, from: &str, to: &str) -> String {
    format!(
        r##"{{"id":"{id}","type":"transition","start":{start},"end":{end},"kind":"crossfade","from":"{from}","to":"{to}"}}"##
    )
}

#[track_caller]
fn report_on(tracks: &str) -> Report {
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(&dir, "p.montaget.json", &project(tracks));
    let document = parse::read(&path).expect("the project parses");
    let mut report = Report::new("validate", Some(document.path().to_string()));
    montaget_core::checks::transition::check(&document, &mut report);
    report
}

#[track_caller]
fn findings(report: &Report) -> Vec<&montaget_core::finding::Finding> {
    report
        .findings
        .iter()
        .filter(|f| f.code == "E-TRANSITION-RANGE")
        .collect()
}

/// The checkerboard ADR-0059 names — `a` and `b` on their own tracks so they may overlap
/// each other in time, `x` bridging them on a third track of its own — since this check's
/// own business is the range, not the per-track non-overlap rule `E-TRACK-OVERLAP` already
/// enforces separately.
fn checkerboard(a: &str, b: &str, x: &str) -> String {
    format!("{},{},{}", track(a), track(b), track(x))
}

#[test]
fn a_transition_whose_range_is_exactly_the_bridged_intersection_never_fires() {
    // `a` runs 0..600, `b` runs 400..1000; the intersection is 400..600.
    let tracks = checkerboard(
        &clip("a", 0, 600),
        &clip("b", 400, 1000),
        &transition("x", 400, 600, "a", "b"),
    );
    let report = report_on(&tracks);
    assert!(findings(&report).is_empty(), "{:?}", report.findings);
}

#[test]
fn a_transition_whose_declared_range_has_drifted_fires() {
    // Same bridged pair, but one of them was trimmed and the transition never followed.
    // Declared 400..700, but the true intersection is still 400..600.
    let tracks = checkerboard(
        &clip("a", 0, 600),
        &clip("b", 400, 1000),
        &transition("x", 400, 700, "a", "b"),
    );
    let report = report_on(&tracks);
    let findings = findings(&report);
    assert_eq!(findings.len(), 1, "{:?}", report.findings);
    let finding = findings[0];
    assert_eq!(finding.class, Class::Error);
    assert_eq!(finding.fields["start"], 400);
    assert_eq!(finding.fields["end"], 700);
    assert_eq!(finding.fields["derived_start"], 400);
    assert_eq!(finding.fields["derived_end"], 600);
    assert_eq!(finding.fields["from"], "a");
    assert_eq!(finding.fields["to"], "b");
    match &finding.repair {
        Some(montaget_core::finding::Repair::Advise(_)) => {}
        other => panic!("expected an advise-class repair, got {other:?}"),
    }
}

#[test]
fn a_dangling_from_or_to_is_silently_out_of_scope() {
    // `b` does not exist in the project at all — a different question than this check
    // answers, and left to whichever check owns dangling references.
    let tracks = checkerboard(&clip("a", 0, 600), "", &transition("x", 400, 600, "a", "b"));
    let report = report_on(&tracks);
    assert!(findings(&report).is_empty(), "{:?}", report.findings);
}

#[test]
fn a_non_transition_element_never_fires() {
    let report = report_on(&track(&clip("a", 0, 600)));
    assert!(findings(&report).is_empty(), "{:?}", report.findings);
}

#[test]
fn validate_reports_it_with_no_ffprobe_needed() {
    let tracks = checkerboard(
        &clip("a", 0, 600),
        &clip("b", 400, 1000),
        &transition("x", 400, 700, "a", "b"),
    );
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(&dir, "p.montaget.json", &project(&tracks));
    let report = validate(&path);
    assert_eq!(
        report
            .findings
            .iter()
            .filter(|f| f.code == "E-TRANSITION-RANGE")
            .count(),
        1,
        "{:?}",
        report.findings
    );
}
