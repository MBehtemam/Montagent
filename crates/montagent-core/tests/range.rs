//! `E-EMPTY-RANGE` from `validate` (#410, ADR-0107): document-only, so `check` is called
//! directly and no project here needs `ffprobe`. The cross-verb half — that `render` now
//! refuses the same document on the check engine's report — is in `cross_verb.rs`.

use montagent_core::finding::{Class, Finding};
use montagent_core::report::Report;
use montagent_core::{parse, validate};

mod common;
use common::{canonical, write_project};

fn project(elements: &str) -> String {
    canonical(&format!(
        r##"{{"frame":{{"width":1080,"height":1920}},"fps":25,
            "tracks":[{{"name":"only","elements":[{elements}]}}]}}"##
    ))
}

fn rect(id: &str, start: i64, end: i64) -> String {
    format!(
        r##"{{"id":"{id}","type":"rect","start":{start},"end":{end},"x":0,"y":0,"width":900,"height":100}}"##
    )
}

fn clip(id: &str, start: i64, end: i64, source_start: i64, source_end: i64) -> String {
    format!(
        r##"{{"id":"{id}","type":"audio","start":{start},"end":{end},"source":"vo.mp3",
            "source_start":{source_start},"source_end":{source_end}}}"##
    )
}

#[track_caller]
fn empty_ranges(elements: &str) -> Vec<Finding> {
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(&dir, "p.montagent.json", &project(elements));
    let document = parse::read(&path).expect("the project parses");
    let mut report = Report::new("validate", Some(document.path().to_string()));
    montagent_core::checks::range::check(&document, &mut report);
    report
        .findings
        .into_iter()
        .filter(|f| f.code == "E-EMPTY-RANGE")
        .collect()
}

#[test]
fn an_element_whose_end_equals_its_start_is_an_error() {
    let found = empty_ranges(&rect("r", 500, 500));
    assert_eq!(found.len(), 1, "{found:?}");
    let finding = &found[0];
    assert_eq!(finding.class, Class::Error);
    assert_eq!(finding.location.element.as_deref(), Some("r"));
    assert_eq!(finding.location.track.as_deref(), Some("only"));
    assert_eq!(finding.fields["field"], "`start`..`end`");
    assert_eq!(finding.fields["from"], 500);
    assert_eq!(finding.fields["to"], 500);
}

#[test]
fn an_inverted_range_is_the_same_finding() {
    let found = empty_ranges(&rect("r", 800, 300));
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].fields["from"], 800);
    assert_eq!(found[0].fields["to"], 300);
}

#[test]
fn an_empty_source_range_is_its_own_finding() {
    let found = empty_ranges(&clip("vo", 0, 1000, 400, 400));
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].fields["field"], "`source_start`..`source_end`");
}

#[test]
fn both_ranges_empty_on_one_element_are_two_findings() {
    // Two facts the author repairs separately: which of the four fields was meant is not in
    // the document, so neither finding may stand in for the other.
    let found = empty_ranges(&clip("vo", 0, 0, 400, 400));
    assert_eq!(found.len(), 2, "{found:?}");
}

#[test]
fn a_range_that_advances_by_one_millisecond_never_fires() {
    assert!(empty_ranges(&rect("r", 500, 501)).is_empty());
    assert!(empty_ranges(&clip("vo", 0, 1, 400, 401)).is_empty());
}

#[test]
fn a_transitions_empty_range_is_left_to_the_transition_checks() {
    // ADR-0059 derives a transition's range from the two elements it bridges, so an empty
    // one is already `E-TRANSITION-RANGE` or `E-TRANSITION-NO-OVERLAP`. One fact, one code.
    let found = empty_ranges(
        r##"{"id":"x","type":"transition","start":400,"end":400,"kind":"crossfade","from":"a","to":"b"}"##,
    );
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn validate_states_it_and_does_not_pass_the_project_clean() {
    // The shape #410 was filed on: before ADR-0107 this validated at zero errors.
    let dir = common::tempdir(line!());
    let path = write_project(&dir, "p.montagent.json", &project(&rect("r", 0, 0)));
    let report = validate(&path);
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.code == "E-EMPTY-RANGE" && f.class == Class::Error),
        "{:?}",
        report.findings
    );
}
