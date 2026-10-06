//! `E-TRANSITION-RANGE` (#202, ADR-0059): document-only, so `check` is called directly and
//! no project here ever needs `ffprobe`.

use montagent_core::finding::Class;
use montagent_core::report::Report;
use montagent_core::{parse, validate};

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
    let path = write_project(&dir, "p.montagent.json", &project(tracks));
    let document = parse::read(&path).expect("the project parses");
    let mut report = Report::new("validate", Some(document.path().to_string()));
    montagent_core::checks::transition::check(&document, &mut report);
    report
}

#[track_caller]
fn findings(report: &Report) -> Vec<&montagent_core::finding::Finding> {
    report
        .findings
        .iter()
        .filter(|f| f.code == "E-TRANSITION-RANGE")
        .collect()
}

#[track_caller]
fn no_overlap_findings(report: &Report) -> Vec<&montagent_core::finding::Finding> {
    report
        .findings
        .iter()
        .filter(|f| f.code == "E-TRANSITION-NO-OVERLAP")
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
        Some(montagent_core::finding::Repair::Advise(_)) => {}
        other => panic!("expected an advise-class repair, got {other:?}"),
    }
}

#[test]
fn a_dangling_from_or_to_is_not_a_drifted_range() {
    // `b` does not exist in the project at all — `E-TRANSITION-REF-MISSING`'s question
    // (ADR-0150, `tests/transition_kinds.rs`), never a range drift.
    let tracks = checkerboard(&clip("a", 0, 600), "", &transition("x", 400, 600, "a", "b"));
    let report = report_on(&tracks);
    assert!(findings(&report).is_empty(), "{:?}", report.findings);
}

// ---------------------------------------------------------------------------
// `E-TRANSITION-NO-OVERLAP`: the bridged pair shares no instant.
// ---------------------------------------------------------------------------

#[test]
fn bridging_two_elements_that_never_overlap_fires_and_never_e_transition_range() {
    // `a` runs 0..100, `b` runs 200..300 — they never coexist, so no crossfade window
    // exists at all. The declared range here doesn't matter to the outcome.
    let tracks = checkerboard(
        &clip("a", 0, 100),
        &clip("b", 200, 300),
        &transition("x", 0, 100, "a", "b"),
    );
    let report = report_on(&tracks);

    assert!(findings(&report).is_empty(), "{:?}", report.findings);
    let no_overlap = no_overlap_findings(&report);
    assert_eq!(no_overlap.len(), 1, "{:?}", report.findings);
    let finding = no_overlap[0];
    assert_eq!(finding.class, Class::Error);
    assert_eq!(finding.fields["from"], "a");
    assert_eq!(finding.fields["to"], "b");
    assert_eq!(finding.fields["from_start"], 0);
    assert_eq!(finding.fields["from_end"], 100);
    assert_eq!(finding.fields["to_start"], 200);
    assert_eq!(finding.fields["to_end"], 300);
    assert_eq!(
        finding.repair,
        Some(montagent_core::finding::Repair::None),
        "refuse-class: the document does not say whether the transition, `a` or `b` \
         should move"
    );
}

#[test]
fn a_declared_range_matching_the_degenerate_intersection_still_fires() {
    // The bug this test guards: `max(start)`/`min(end)` on a non-overlapping pair is
    // itself an inverted range (200..100 here), and a transition that happens to declare
    // exactly that pair must not validate clean just because it matches the arithmetic —
    // there is still no window a crossfade could show.
    let tracks = checkerboard(
        &clip("a", 0, 100),
        &clip("b", 200, 300),
        &transition("x", 200, 100, "a", "b"),
    );
    let report = report_on(&tracks);

    assert!(findings(&report).is_empty(), "{:?}", report.findings);
    assert_eq!(
        no_overlap_findings(&report).len(),
        1,
        "{:?}",
        report.findings
    );
}

#[test]
fn touching_endpoints_do_not_overlap_either() {
    // ADR-0005's half-open interval: `a` ending at 100 and `b` starting at 100 share no
    // instant, the same boundary rule `E-TRACK-OVERLAP` uses.
    let tracks = checkerboard(
        &clip("a", 0, 100),
        &clip("b", 100, 200),
        &transition("x", 90, 110, "a", "b"),
    );
    let report = report_on(&tracks);
    assert_eq!(
        no_overlap_findings(&report).len(),
        1,
        "{:?}",
        report.findings
    );
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
    let path = write_project(&dir, "p.montagent.json", &project(&tracks));
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

#[test]
fn the_no_overlap_finding_renders_as_refuse_class_prose() {
    let tracks = checkerboard(
        &clip("a", 0, 100),
        &clip("b", 200, 300),
        &transition("x", 0, 100, "a", "b"),
    );
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(&dir, "p.montagent.json", &project(&tracks));
    let rendered = montagent_core::wire::render(
        &validate(&path),
        montagent_core::Wire::Text { verbose: false },
    );
    assert!(rendered.contains("never overlap"), "{rendered}");
    assert!(rendered.contains("refuse-class"), "{rendered}");
    assert!(rendered.contains("0..100"), "{rendered}");
    assert!(rendered.contains("200..300"), "{rendered}");
}
