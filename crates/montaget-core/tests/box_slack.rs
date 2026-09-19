//! `R-BOX-SLACK` (#201, ADR-0058): height-only, `note`-level, no font and no I/O.
//!
//! Every project here carries text elements with a declared `width`/`height` and no
//! `source` anywhere in the document, so this check — and this test file — never needs
//! the `ffprobe` the disk half of `validate` wants; `check` is called directly, the same
//! way `tests/caption.rs` does, so "no font, no I/O" is a property of the signature
//! rather than a claim a test has to make.

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

/// A text element carrying one run, an explicit `line_height`, and a declared `height`.
/// `\n` must reach the document as an escape, so the caller writes it as `\\n`.
fn text(id: &str, size: i64, line_height: &str, height: i64, runs: &str) -> String {
    format!(
        r##"{{"id":"{id}","type":"text","start":0,"end":1000,"x":0,"y":0,"width":900,"height":{height},"font":"brand","size":{size},"line_height":{line_height},"runs":[{{"text":"{runs}"}}]}}"##
    )
}

/// The same, with `line_height` omitted — ADR-0028's `1.2` default.
fn text_default_line_height(id: &str, size: i64, height: i64, runs: &str) -> String {
    format!(
        r##"{{"id":"{id}","type":"text","start":0,"end":1000,"x":0,"y":0,"width":900,"height":{height},"font":"brand","size":{size},"runs":[{{"text":"{runs}"}}]}}"##
    )
}

/// A `rect` declaring `height` — the sibling census's pool is not scoped to `text`.
fn rect(id: &str, height: i64) -> String {
    format!(
        r##"{{"id":"{id}","type":"rect","start":0,"end":1000,"x":0,"y":0,"width":900,"height":{height}}}"##
    )
}

/// `R-BOX-SLACK` alone, over a project written to a scratch directory.
#[track_caller]
fn report_on(tracks: &str) -> Report {
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(&dir, "p.montaget.json", &project(tracks));
    let document = parse::read(&path).expect("the project parses");
    let mut report = Report::new("validate", Some(document.path().to_string()));
    montaget_core::checks::box_slack::check(&document, &mut report);
    report
}

#[track_caller]
fn findings(report: &Report) -> Vec<&montaget_core::finding::Finding> {
    report
        .findings
        .iter()
        .filter(|f| f.code == "R-BOX-SLACK")
        .collect()
}

#[test]
fn a_declared_height_matching_the_computed_block_height_exactly_never_fires() {
    // `size 55 × line_height 1.1 × 1 line` computes to exactly `61`.
    let report = report_on(&track(&text("t1", 55, "1.1", 61, "hello")));
    assert!(findings(&report).is_empty(), "{:?}", report.findings);
}

#[test]
fn slack_at_the_two_pixel_floor_does_not_clear_it() {
    // Computed height 61; declared 63 is exactly 2px of slack, and ADR-0058's threshold
    // is `slack > max(2px, 10%)` — strictly greater, so exactly 2 does not fire.
    let report = report_on(&track(&text("t1", 55, "1.1", 63, "hello")));
    assert!(
        findings(&report).is_empty(),
        "exactly 2px of slack does not clear the floor: {:?}",
        report.findings
    );
}

#[test]
fn slack_one_pixel_past_the_floor_fires_when_it_also_clears_ten_percent() {
    // Computed height 61; declared 64 is 3px slack, which is both > 2px and > 10% of 61
    // (6.1) is false — 3 < 6.1, so this must NOT fire: `max(2, 10%)` requires clearing
    // both.
    let report = report_on(&track(&text("t1", 55, "1.1", 64, "hello")));
    assert!(
        findings(&report).is_empty(),
        "3px slack clears the 2px floor but not 10% of 61: {:?}",
        report.findings
    );
}

#[test]
fn slack_that_clears_both_the_floor_and_ten_percent_fires() {
    // ADR-0058's own worked fixture case: computed 61, declared 169, slack 108 (177%).
    let report = report_on(&track(&text("t1", 55, "1.1", 169, "hello")));
    let findings = findings(&report);
    assert_eq!(findings.len(), 1, "{:?}", report.findings);
    let finding = findings[0];
    assert_eq!(finding.class, Class::Note);
    assert_eq!(finding.fields["declared_height"], 169);
    assert_eq!(finding.fields["computed_height"], 61);
    assert_eq!(finding.fields["slack"], 108);
    assert_eq!(finding.fields["slack_percent"], 177);
    assert_eq!(
        finding.fields["derivation"],
        "size 55 × line_height 1.1 × 1 line"
    );
    assert!(finding.census.is_none(), "no sibling shares height 169");
}

#[test]
fn an_omitted_line_height_defaults_to_one_point_two() {
    // `size 50 × line_height 1.2 × 1 line`: `(50 × 12 × 1 + 9) // 10 = 60`.
    let report = report_on(&track(&text_default_line_height("t1", 50, 200, "hi")));
    let findings = findings(&report);
    assert_eq!(findings.len(), 1, "{:?}", report.findings);
    assert_eq!(findings[0].fields["computed_height"], 60);
    assert_eq!(
        findings[0].fields["derivation"],
        "size 50 × line_height 1.2 × 1 line"
    );
}

#[test]
fn line_count_is_the_authors_own_newline_count_never_a_wrap_estimate() {
    // ADR-0008: no auto-wrap. Two `\n` is three lines: `size 40 × line_height 1.1 × 3
    // lines`, `(40 × 11 × 3 + 9) // 10 = 132`.
    let report = report_on(&track(&text("t1", 40, "1.1", 400, "one\\ntwo\\nthree")));
    let findings = findings(&report);
    assert_eq!(findings.len(), 1, "{:?}", report.findings);
    assert_eq!(findings[0].fields["computed_height"], 132);
    assert_eq!(
        findings[0].fields["derivation"],
        "size 40 × line_height 1.1 × 3 lines"
    );
}

#[test]
fn the_sibling_census_matches_any_element_type_on_exact_declared_height() {
    let elements = format!(
        "{},{},{}",
        text("caption", 55, "1.1", 169, "hello"),
        rect("card", 169),
        rect("panel", 200),
    );
    let report = report_on(&track(&elements));
    let findings = findings(&report);
    assert_eq!(findings.len(), 1, "{:?}", report.findings);
    let census = findings[0].census.as_ref().expect("card shares height 169");
    assert_eq!(census.field, "height");
    assert_eq!(census.groups.len(), 1);
    assert_eq!(census.groups[0].value, 169);
    assert_eq!(census.groups[0].members, vec!["card".to_string()]);
}

#[test]
fn a_census_is_omitted_rather_than_stated_as_empty_when_nothing_matches() {
    let report = report_on(&track(&text("t1", 55, "1.1", 169, "hello")));
    let findings = findings(&report);
    assert_eq!(findings.len(), 1, "{:?}", report.findings);
    assert!(findings[0].census.is_none());
}

#[test]
fn a_non_text_element_never_fires() {
    let report = report_on(&track(&rect("card", 500)));
    assert!(findings(&report).is_empty(), "{:?}", report.findings);
}

#[test]
fn a_maliciously_large_declared_height_is_a_finding_not_a_panic() {
    // `declared_height - computed_height` and `slack * 10` must never overflow `i64`,
    // however large a `height` the document states — this is a fact about the project,
    // not a crash in the tool that read it.
    let report = report_on(&track(&text("t1", 55, "1.1", i64::MAX, "hello")));
    let findings = findings(&report);
    assert_eq!(findings.len(), 1, "{:?}", report.findings);
    assert_eq!(findings[0].fields["declared_height"], i64::MAX);
    assert_eq!(findings[0].fields["computed_height"], 61);
}

#[test]
fn the_check_never_gates_the_render_and_collapses_to_one_line() {
    let report = report_on(&track(&text("t1", 55, "1.1", 169, "hello")));
    assert_eq!(report.summary().note, 1);
    assert_eq!(report.summary().error, 0);
    let rendered =
        montaget_core::text::render(&report.to_json(), montaget_core::text::Options::default())
            .unwrap();
    assert_eq!(
        rendered.matches("R-BOX-SLACK").count(),
        1,
        "one counted line, not one printed in full: {rendered}"
    );
}

#[test]
fn validate_reports_it_with_no_ffprobe_needed() {
    // The document carries no `source` anywhere, so the disk half of `validate` has
    // nothing to probe and this runs on any machine.
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(
        &dir,
        "p.montaget.json",
        &project(&track(&text("t1", 55, "1.1", 169, "hello"))),
    );
    let report = validate(&path);
    assert_eq!(
        report
            .findings
            .iter()
            .filter(|f| f.code == "R-BOX-SLACK")
            .count(),
        1,
        "{:?}",
        report.findings
    );
}
