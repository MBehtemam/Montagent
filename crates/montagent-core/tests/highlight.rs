//! `E-HIGHLIGHT-RANGE` and `E-HIGHLIGHT-OVERLAP` (#202, ADR-0051): both are document-only,
//! so — the same way `tests/box_slack.rs` and `tests/caption.rs` do — `check` is called
//! directly and no project here ever needs `ffprobe`.

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

/// A `text` element on `[start, end)` carrying the given `runs`, verbatim JSON.
fn text(id: &str, start: i64, end: i64, runs: &str) -> String {
    format!(
        r##"{{"id":"{id}","type":"text","start":{start},"end":{end},"x":0,"y":0,"width":900,"height":100,"font":"brand","size":40,"runs":[{runs}]}}"##
    )
}

/// One run carrying a `highlight` window.
fn run(text: &str, highlight_start: i64, highlight_end: i64) -> String {
    format!(
        r##"{{"text":"{text}","highlight":{{"start":{highlight_start},"end":{highlight_end}}}}}"##
    )
}

#[track_caller]
fn report_on(tracks: &str) -> Report {
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(&dir, "p.montagent.json", &project(tracks));
    let document = parse::read(&path).expect("the project parses");
    let mut report = Report::new("validate", Some(document.path().to_string()));
    montagent_core::checks::highlight::check(&document, &mut report);
    report
}

#[track_caller]
fn findings<'a>(report: &'a Report, code: &str) -> Vec<&'a montagent_core::finding::Finding> {
    report.findings.iter().filter(|f| f.code == code).collect()
}

#[test]
fn a_highlight_window_inside_the_elements_range_never_fires() {
    let report = report_on(&track(&text("t1", 0, 1000, &run("cobwebs", 100, 300))));
    assert!(
        findings(&report, "E-HIGHLIGHT-RANGE").is_empty(),
        "{:?}",
        report.findings
    );
}

#[test]
fn a_highlight_window_outside_the_elements_range_fires() {
    // Element runs 0..1000; the window's `end` at 1200 is past it.
    let report = report_on(&track(&text("t1", 0, 1000, &run("cobwebs", 800, 1200))));
    let findings = findings(&report, "E-HIGHLIGHT-RANGE");
    assert_eq!(findings.len(), 1, "{:?}", report.findings);
    let finding = findings[0];
    assert_eq!(finding.class, Class::Error);
    assert_eq!(finding.fields["run"], "cobwebs");
    assert_eq!(finding.fields["start"], 800);
    assert_eq!(finding.fields["end"], 1200);
    assert_eq!(finding.fields["element_start"], 0);
    assert_eq!(finding.fields["element_end"], 1000);
    assert_eq!(finding.repair, Some(montagent_core::finding::Repair::None));
}

#[test]
fn a_window_starting_before_the_elements_own_start_fires() {
    let report = report_on(&track(&text("t1", 500, 1000, &run("cobwebs", 400, 600))));
    assert_eq!(
        findings(&report, "E-HIGHLIGHT-RANGE").len(),
        1,
        "{:?}",
        report.findings
    );
}

#[test]
fn a_window_ending_exactly_at_the_elements_end_is_contained() {
    // Half-open: the element's own `end` is the exclusive boundary of its range, and the
    // window's `end` may equal it without being "outside".
    let report = report_on(&track(&text("t1", 0, 1000, &run("cobwebs", 800, 1000))));
    assert!(
        findings(&report, "E-HIGHLIGHT-RANGE").is_empty(),
        "{:?}",
        report.findings
    );
}

#[test]
fn non_overlapping_sibling_windows_never_fire() {
    let runs = format!("{},{}", run("cob", 100, 200), run("webs", 200, 300));
    let report = report_on(&track(&text("t1", 0, 1000, &runs)));
    assert!(
        findings(&report, "E-HIGHLIGHT-OVERLAP").is_empty(),
        "adjacent, half-open windows do not overlap: {:?}",
        report.findings
    );
}

#[test]
fn overlapping_sibling_windows_fire() {
    let runs = format!("{},{}", run("cob", 100, 250), run("webs", 200, 300));
    let report = report_on(&track(&text("t1", 0, 1000, &runs)));
    let findings = findings(&report, "E-HIGHLIGHT-OVERLAP");
    assert_eq!(findings.len(), 1, "{:?}", report.findings);
    let finding = findings[0];
    assert_eq!(finding.class, Class::Error);
    assert_eq!(finding.fields["run"], "cob");
    assert_eq!(finding.fields["other"], "webs");
    assert_eq!(finding.fields["overlap"], 50);
    assert_eq!(finding.repair, Some(montagent_core::finding::Repair::None));
}

#[test]
fn one_window_wholly_containing_two_later_ones_is_caught_against_both() {
    let runs = format!(
        "{},{},{}",
        run("whole", 0, 900),
        run("first", 100, 200),
        run("second", 300, 400)
    );
    let report = report_on(&track(&text("t1", 0, 1000, &runs)));
    let findings = findings(&report, "E-HIGHLIGHT-OVERLAP");
    assert_eq!(findings.len(), 2, "{:?}", report.findings);
}

#[test]
fn a_run_with_no_highlight_never_fires() {
    let report = report_on(&track(&text("t1", 0, 1000, r##"{"text":"plain"}"##)));
    assert!(findings(&report, "E-HIGHLIGHT-RANGE").is_empty());
    assert!(findings(&report, "E-HIGHLIGHT-OVERLAP").is_empty());
}

#[test]
fn a_non_text_element_never_fires() {
    let report = report_on(&track(
        r##"{"id":"card","type":"rect","start":0,"end":1000,"x":0,"y":0,"width":900,"height":100}"##,
    ));
    assert!(findings(&report, "E-HIGHLIGHT-RANGE").is_empty());
    assert!(findings(&report, "E-HIGHLIGHT-OVERLAP").is_empty());
}

#[test]
fn the_check_collapses_to_one_printed_line_per_finding() {
    let report = report_on(&track(&text("t1", 0, 1000, &run("cobwebs", 800, 1200))));
    let rendered =
        montagent_core::text::render(&report.to_json(), montagent_core::text::Options::default())
            .unwrap();
    assert_eq!(
        rendered.matches("E-HIGHLIGHT-RANGE").count(),
        1,
        "{rendered}"
    );
}

#[test]
fn validate_reports_it_with_no_ffprobe_needed() {
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(
        &dir,
        "p.montagent.json",
        &project(&track(&text("t1", 0, 1000, &run("cobwebs", 800, 1200)))),
    );
    let report = validate(&path);
    assert_eq!(
        report
            .findings
            .iter()
            .filter(|f| f.code == "E-HIGHLIGHT-RANGE")
            .count(),
        1,
        "{:?}",
        report.findings
    );
}

// ---- `R-HIGHLIGHT-UNPAINTED` (#554, ADR-0134). -----------------------------------------
// At 30 fps the painted instants are ⌊n × 1000 / 30⌋: …, 2400, 2433, 2466, … — the grid
// brief B's "Now" (2420–2440 ms, #542) sat on.

#[track_caller]
fn report_at(fps: i64, tracks: &str) -> Report {
    let dir = common::tempdir(std::panic::Location::caller().line());
    let project = canonical(&format!(
        r##"{{"frame":{{"width":1080,"height":1920}},"fps":{fps},"tracks":[{tracks}]}}"##
    ));
    let path = write_project(&dir, "p.montagent.json", &project);
    let document = parse::read(&path).expect("the project parses");
    let mut report = Report::new("validate", Some(document.path().to_string()));
    montagent_core::checks::highlight::check(&document, &mut report);
    report
}

#[test]
fn a_window_between_two_painted_instants_fires() {
    let report = report_at(30, &track(&text("t1", 2000, 3000, &run("Now", 2434, 2454))));
    let findings = findings(&report, "R-HIGHLIGHT-UNPAINTED");
    assert_eq!(findings.len(), 1, "{:?}", report.findings);
    let finding = findings[0];
    assert_eq!(finding.class, Class::Review);
    assert_eq!(finding.repair, None);
    assert_eq!(finding.fields["fps"], 30);
    assert_eq!(finding.fields["count"], 1);
    let detail = finding.fields["detail"].as_str().unwrap();
    assert!(detail.contains("`t1`"), "{detail}");
    assert!(detail.contains("\"Now\""), "{detail}");
    assert!(detail.contains("2434..2454 ms"), "{detail}");
    assert!(detail.contains("2433 and 2466 ms"), "{detail}");
}

#[test]
fn brief_bs_one_frame_window_does_not_fire() {
    // Frame 73 is painted at 2433, inside 2420..2440: one frame, which is not zero.
    let report = report_at(30, &track(&text("t1", 2000, 3000, &run("Now", 2420, 2440))));
    assert!(
        findings(&report, "R-HIGHLIGHT-UNPAINTED").is_empty(),
        "{:?}",
        report.findings
    );
}

#[test]
fn a_window_ending_on_a_painted_instant_does_not_light_it() {
    // Half-open: 2433 is outside 2401..2433, and 2400 is before it.
    let report = report_at(30, &track(&text("t1", 2000, 3000, &run("Now", 2401, 2433))));
    assert_eq!(
        findings(&report, "R-HIGHLIGHT-UNPAINTED").len(),
        1,
        "{:?}",
        report.findings
    );
}

#[test]
fn unpainted_windows_across_two_elements_are_one_finding() {
    let first = text(
        "t1",
        2000,
        3000,
        &format!("{},{}", run("Now", 2434, 2454), run("then", 2700, 2900)),
    );
    let second = text("t2", 3000, 4000, &run("a", 3401, 3433));
    let report = report_at(30, &track(&format!("{first},{second}")));
    let findings = findings(&report, "R-HIGHLIGHT-UNPAINTED");
    assert_eq!(findings.len(), 1, "{:?}", report.findings);
    assert_eq!(findings[0].fields["count"], 2);
    let detail = findings[0].fields["detail"].as_str().unwrap();
    assert!(
        detail.contains("\"Now\"") && detail.contains("\"a\""),
        "{detail}"
    );
    assert!(!detail.contains("\"then\""), "{detail}");
}

#[test]
fn a_project_whose_every_window_holds_a_frame_emits_nothing() {
    let report = report_at(
        30,
        &track(&text(
            "t1",
            2000,
            3000,
            &format!("{},{}", run("Now", 2400, 2500), run("then", 2500, 2900)),
        )),
    );
    assert!(
        findings(&report, "R-HIGHLIGHT-UNPAINTED").is_empty(),
        "{:?}",
        report.findings
    );
}

#[test]
fn validate_reports_an_unpainted_window_as_a_review() {
    let dir = common::tempdir(std::panic::Location::caller().line());
    let project = canonical(&format!(
        r##"{{"frame":{{"width":1080,"height":1920}},"fps":30,"tracks":[{}]}}"##,
        track(&text("t1", 2000, 3000, &run("Now", 2434, 2454)))
    ));
    let path = write_project(&dir, "p.montagent.json", &project);
    let report = validate(&path);
    let unpainted: Vec<_> = report
        .findings
        .iter()
        .filter(|f| f.code == "R-HIGHLIGHT-UNPAINTED")
        .collect();
    assert_eq!(unpainted.len(), 1, "{:?}", report.findings);
    assert_eq!(unpainted[0].class, Class::Review);
    let rendered =
        montagent_core::text::render(&report.to_json(), montagent_core::text::Options::default())
            .unwrap();
    assert!(rendered.contains("R-HIGHLIGHT-UNPAINTED"), "{rendered}");
}
