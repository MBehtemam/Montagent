//! `R-EASE-INERT` (#211, ADR-0052): an `ease` that describes no motion, because the two
//! keyframe records it sits between carry the identical author-written `v`.
//!
//! ADR-0038 made `ease` required on every record but the first and named this as the
//! acknowledged cost; this check is the review-level lint that discharges it.

use montaget_core::finding::{Class, Finding};
use montaget_core::report::Report;
use montaget_core::{checks, parse};

mod common;
use common::{canonical, write_project};

/// A text element carrying `scale` as the given raw keyframe-record JSON.
fn element(id: &str, property: &str, records: &str) -> String {
    format!(
        r##"{{"id":"{id}","type":"rect","start":0,"end":20000,"x":0,"y":0,"origin":"top-left","width":100,"height":100,"fill":"#1E344C","{property}":[{records}]}}"##
    )
}

fn project(elements: &str) -> String {
    canonical(&format!(
        r##"{{"frame":{{"width":1080,"height":1920}},"fps":25,"tracks":[{{"name":"t","layer":10,"elements":[{elements}]}}]}}"##
    ))
}

#[track_caller]
fn report_on(elements: &str) -> Report {
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(&dir, "p.montaget.json", &project(elements));
    let document = parse::read(&path).expect("the project parses");
    let mut report = Report::new("validate", Some(document.path().to_string()));
    checks::ease::check(&document, &mut report);
    report
}

#[track_caller]
fn inert(report: &Report) -> Vec<&Finding> {
    report
        .findings
        .iter()
        .filter(|f| f.code == "R-EASE-INERT")
        .collect()
}

#[test]
fn two_consecutive_keyframes_holding_one_value_carry_an_inert_ease() {
    // ADR-0052's own worked example: "2 consecutive keyframes hold `v=[1.0,1.0]` from
    // `t=12000` to `t=18000`; `ease=\"linear\"` describes no motion."
    let records = r##"{"t":12000,"v":[1.0,1.0]},{"t":18000,"v":[1.0,1.0],"ease":"linear"}"##;
    let report = report_on(&element("photo-06", "scale", records));
    let findings = inert(&report);
    assert_eq!(findings.len(), 1, "{:?}", report.findings);

    let finding = findings[0];
    assert_eq!(finding.class, Class::Review);
    assert_eq!(finding.fields["element"], serde_json::json!("photo-06"));
    assert_eq!(finding.fields["property"], serde_json::json!("scale"));
    assert_eq!(finding.fields["records"], serde_json::json!(2));
    assert_eq!(finding.fields["from"], serde_json::json!(12000));
    assert_eq!(finding.fields["to"], serde_json::json!(18000));
    assert_eq!(finding.fields["value"], serde_json::json!([1.0, 1.0]));
    // No repair is proposed — delete the `ease`? change `v`? — because that is an
    // authorial-intent judgment ADR-0006 forbids a finding from making.
    assert_eq!(finding.repair, None);
}

#[test]
fn an_ease_over_a_value_that_actually_changes_is_not_inert() {
    let records = r##"{"t":0,"v":[1.0,1.0]},{"t":18000,"v":[1.08,1.08],"ease":"linear"}"##;
    assert!(inert(&report_on(&element("photo-06", "scale", records))).is_empty());
}

#[test]
fn a_run_of_holds_is_one_finding_and_not_one_per_pair() {
    // ADR-0052: "three keyframe records all sharing one `v` produce two inert-ease
    // segments back to back, and reporting them as two lines duplicates the same fact."
    let records =
        r##"{"t":0,"v":1},{"t":5000,"v":1,"ease":"linear"},{"t":9000,"v":1,"ease":"linear"}"##;
    let report = report_on(&element("card", "opacity", records));
    let findings = inert(&report);
    assert_eq!(findings.len(), 1, "{:?}", report.findings);
    assert_eq!(findings[0].fields["records"], serde_json::json!(3));
    assert_eq!(findings[0].fields["from"], serde_json::json!(0));
    assert_eq!(findings[0].fields["to"], serde_json::json!(9000));
}

#[test]
fn a_hold_on_one_axis_only_is_not_a_finding() {
    // ADR-0052 decided whole-value equality, 2–1: "`scale: [1.0, 1.0] → [1.5, 1.0]`
    // genuinely shapes `sx` and describes nothing on `sy`", and firing on a partial hold
    // would nag an author for a legitimate horizontal-only stretch.
    let records = r##"{"t":0,"v":[1.0,1.0]},{"t":9000,"v":[1.5,1.0],"ease":"linear"}"##;
    assert!(inert(&report_on(&element("card", "scale", records))).is_empty());
}

#[test]
fn a_minuscule_authored_difference_is_a_real_difference() {
    // ADR-0052: "if an agent writes `1.0` on one record and `0.9999999999` on the next,
    // that is a real — if minuscule — authored difference", and a finding asserting they
    // are the same would state an intent the file does not carry. Exact equality, no
    // epsilon.
    let records = r##"{"t":0,"v":1.0},{"t":9000,"v":0.9999999999,"ease":"linear"}"##;
    assert!(inert(&report_on(&element("card", "opacity", records))).is_empty());
}

#[test]
fn a_hold_and_a_move_in_one_list_report_only_the_hold() {
    let records = r##"{"t":0,"v":0},{"t":3000,"v":1,"ease":"linear"},{"t":6000,"v":1,"ease":"linear"},{"t":9000,"v":0,"ease":"linear"}"##;
    let report = report_on(&element("card", "opacity", records));
    let findings = inert(&report);
    assert_eq!(findings.len(), 1, "{:?}", report.findings);
    assert_eq!(findings[0].fields["from"], serde_json::json!(3000));
    assert_eq!(findings[0].fields["to"], serde_json::json!(6000));
}

#[test]
fn a_static_value_carries_no_ease_and_is_never_inert() {
    // `[1.0, 1.0]` is `scale`'s own static spelling, not a keyframe list (ADR-0012's
    // shape test: a record is an object).
    let element = r##"{"id":"card","type":"rect","start":0,"end":20000,"x":0,"y":0,"origin":"top-left","width":100,"height":100,"fill":"#1E344C","scale":[1.0,1.0]}"##;
    assert!(inert(&report_on(element)).is_empty());
}

#[test]
fn the_registry_declares_the_check_live_and_review_class() {
    let spec = montaget_core::registry::spec("R-EASE-INERT").expect("registered");
    assert_eq!(spec.default_class(), Class::Review);
    assert_eq!(spec.adr, "ADR-0052");
    assert_eq!(spec.repair, None);
}

#[test]
fn the_finding_renders_as_prose_and_validate_runs_the_check() {
    let dir = common::tempdir(std::panic::Location::caller().line());
    let records = r##"{"t":12000,"v":[1.0,1.0]},{"t":18000,"v":[1.0,1.0],"ease":"linear"}"##;
    let path = write_project(
        &dir,
        "p.montaget.json",
        &project(&element("photo-06", "scale", records)),
    );
    let report = montaget_core::validate(&path);
    let rendered =
        montaget_core::text::render(&report.to_json(), montaget_core::text::Options::default())
            .expect("every template's fields are carried");
    // ADR-0052's own worked line, in the product's words.
    assert!(rendered.contains("R-EASE-INERT"), "{rendered}");
    assert!(rendered.contains("photo-06`.scale"), "{rendered}");
    assert!(rendered.contains("t=12000"), "{rendered}");
    assert!(rendered.contains("\"linear\""), "{rendered}");
}
