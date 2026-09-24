//! `R-KEYFRAME-UNREACHED` (#211, ADR-0035): a keyframe's declared target value that no
//! frame the renderer samples inside the element's own range ever produces — *"the fade
//! that never reaches zero"*.
//!
//! ADR-0035's measured case: `opacity` 1 → 0 over `[end-300, end]` leaves 0.010–0.171
//! residual opacity on the last sampled frame, so the element pops off rather than fading.
//! The renderer is not asked to round anything; the check names the gap instead.

use montagent_core::finding::{Class, Finding};
use montagent_core::report::Report;
use montagent_core::{checks, parse};

mod common;
use common::{canonical, write_project};

fn project(fps: i64, elements: &str) -> String {
    canonical(&format!(
        r##"{{"frame":{{"width":1080,"height":1920}},"fps":{fps},"tracks":[{{"name":"t","layer":10,"elements":[{elements}]}}]}}"##
    ))
}

/// A rectangle over `start..end` carrying `property` as the given raw records.
fn element(id: &str, start: i64, end: i64, property: &str, records: &str) -> String {
    format!(
        r##"{{"id":"{id}","type":"rect","start":{start},"end":{end},"x":0,"y":0,"origin":"top-left","width":100,"height":100,"fill":"#1E344C","{property}":[{records}]}}"##
    )
}

#[track_caller]
fn report_on(fps: i64, elements: &str) -> Report {
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(&dir, "p.montagent.json", &project(fps, elements));
    let document = parse::read(&path).expect("the project parses");
    let mut report = Report::new("validate", Some(document.path().to_string()));
    checks::unreached::check(&document, &mut report);
    report
}

#[track_caller]
fn unreached(report: &Report) -> Vec<&Finding> {
    report
        .findings
        .iter()
        .filter(|f| f.code == "R-KEYFRAME-UNREACHED")
        .collect()
}

#[test]
fn a_fade_targeting_the_elements_own_end_never_reaches_zero() {
    // ADR-0035's founding defect, literally spelled: `opacity` 1 → 0 over the last 300 ms,
    // with the final record at the element's own `end`. The range is half-open, so `end`
    // is never a sampled instant and the declared 0 is never shown.
    let records = r##"{"t":9700,"v":1.0},{"t":10000,"v":0.0,"ease":"linear"}"##;
    let report = report_on(25, &element("card", 0, 10000, "opacity", records));
    let findings = unreached(&report);
    assert_eq!(findings.len(), 1, "{:?}", report.findings);

    let finding = findings[0];
    assert_eq!(finding.class, Class::Review);
    assert_eq!(finding.fields["element"], serde_json::json!("card"));
    assert_eq!(finding.fields["property"], serde_json::json!("opacity"));
    assert_eq!(finding.fields["declared_t"], serde_json::json!(10000));
    assert_eq!(finding.fields["target"], serde_json::json!(0.0));
    // The last sampled frame is 249 at 9960 ms, where the fade has 40 of its 300 ms left.
    assert_eq!(finding.fields["frame"], serde_json::json!(249));
    assert_eq!(finding.fields["sampled_t"], serde_json::json!(9960.0));
    let sampled = finding.fields["sampled"].as_f64().unwrap();
    assert!(
        (sampled - 40.0 / 300.0).abs() < 1e-9,
        "residual opacity, got {sampled}"
    );
}

#[test]
fn a_fade_retargeted_onto_the_grid_reaches_its_zero() {
    // ADR-0035's own repair: "an author who wants `opacity` to read exactly 0 at the frame
    // nearest an element's `end` targets that returned instant instead of the literal
    // `end`." 9960 is that instant at 25 fps, and frame 249 samples it exactly.
    let records = r##"{"t":9660,"v":1.0},{"t":9960,"v":0.0,"ease":"linear"}"##;
    assert!(
        unreached(&report_on(
            25,
            &element("card", 0, 10000, "opacity", records)
        ))
        .is_empty()
    );
}

#[test]
fn a_trimmed_move_whose_keyframe_sits_past_the_element_is_not_a_finding() {
    // The fixture's own shape, seven times over: a Ken Burns move written to 15 s and an
    // element cut short of it. ADR-0012's clamping is what makes that spelling work and
    // `CONTEXT.md` calls it ordinary authoring; the element declares no plateau inside its
    // own range, so there is no claim for this check to test.
    let records = r##"{"t":3018,"v":[1.0,1.0]},{"t":18018,"v":[1.08,1.08],"ease":"linear"}"##;
    assert!(
        unreached(&report_on(
            25,
            &element("photo-05", 3018, 17472, "scale", records)
        ))
        .is_empty()
    );
}

#[test]
fn an_off_grid_start_is_not_by_itself_a_finding() {
    // ADR-0006 refused the literal off-grid check: 109 of the fixture's 120 time values
    // are off the 40 ms grid, and "a check with a 98% hit rate on a correct, published
    // project is not a check; it is the alarm fatigue this ADR already identified as a
    // safety problem." A first record at the element's own off-grid `start` declares no
    // plateau — only the instant the element enters — so it is not asked about.
    let records = r##"{"t":3018,"v":[1.0,1.0]},{"t":8000,"v":[1.08,1.08],"ease":"linear"}"##;
    assert!(
        unreached(&report_on(
            25,
            &element("photo-05", 3018, 9000, "scale", records)
        ))
        .is_empty()
    );
}

#[test]
fn a_first_keyframe_plateau_shorter_than_a_frame_is_never_shown() {
    // The other half of ADR-0035's "(or first)": the element declares a plateau from its
    // start to 3030, and at 25 fps the only frame at-or-after 3020 lands at 3040 — past
    // it. The declared entry value is shown on no frame.
    let records = r##"{"t":3030,"v":1.0},{"t":8000,"v":0.0,"ease":"linear"}"##;
    let report = report_on(25, &element("card", 3020, 9000, "opacity", records));
    let findings = unreached(&report);
    assert_eq!(findings.len(), 1, "{:?}", report.findings);
    assert_eq!(findings[0].fields["declared_t"], serde_json::json!(3030));
    assert_eq!(findings[0].fields["target"], serde_json::json!(1.0));
    assert_eq!(findings[0].fields["frame"], serde_json::json!(76));
}

#[test]
fn a_first_keyframe_plateau_holding_a_frame_reaches_its_value() {
    // The same shape, with the plateau long enough to hold frame 76 at 3040. Both tests
    // end their travel well inside the element so that only the first record is in
    // question.
    let records = r##"{"t":3100,"v":1.0},{"t":8000,"v":0.0,"ease":"linear"}"##;
    assert!(
        unreached(&report_on(
            25,
            &element("card", 3020, 9000, "opacity", records)
        ))
        .is_empty()
    );
}

#[test]
fn the_grid_is_not_necessarily_integral() {
    // ADR-0035: "at `fps=30` the grid step is `100/3` ms and ... multiples of 100 ms are
    // the only frame-exact instants — the non-obvious fact one agent found only by
    // building a private checker." The instant is carried as an exact ratio and only
    // divided where it prints.
    let records = r##"{"t":900,"v":1.0},{"t":1000,"v":0.0,"ease":"linear"}"##;
    let report = report_on(30, &element("card", 0, 1000, "opacity", records));
    let findings = unreached(&report);
    assert_eq!(findings.len(), 1, "{:?}", report.findings);
    // Frame 29 samples at 29 × 100/3 = 966.66… ms, which is not a whole millisecond.
    assert_eq!(findings[0].fields["frame"], serde_json::json!(29));
    let sampled_t = findings[0].fields["sampled_t"].as_f64().unwrap();
    assert!((sampled_t - 2900.0 / 3.0).abs() < 1e-9, "{sampled_t}");
}

#[test]
fn a_static_value_has_no_target_to_miss() {
    let card = r##"{"id":"card","type":"rect","start":0,"end":10000,"x":0,"y":0,"origin":"top-left","width":100,"height":100,"fill":"#1E344C","opacity":0.5}"##;
    assert!(unreached(&report_on(25, card)).is_empty());
}

#[test]
fn the_registry_declares_the_check_live_and_review_class() {
    let spec = montagent_core::registry::spec("R-KEYFRAME-UNREACHED").expect("registered");
    assert_eq!(spec.default_class(), Class::Review);
    assert_eq!(spec.adr, "ADR-0035");
    assert_eq!(spec.repair, None);
}

#[test]
fn the_finding_renders_as_prose_and_validate_runs_the_check() {
    let dir = common::tempdir(std::panic::Location::caller().line());
    let records = r##"{"t":9700,"v":1.0},{"t":10000,"v":0.0,"ease":"linear"}"##;
    let path = write_project(
        &dir,
        "p.montagent.json",
        &project(25, &element("card", 0, 10000, "opacity", records)),
    );
    let report = montagent_core::validate(&path);
    let rendered =
        montagent_core::text::render(&report.to_json(), montagent_core::text::Options::default())
            .expect("every template's fields are carried");
    assert!(rendered.contains("R-KEYFRAME-UNREACHED"), "{rendered}");
    assert!(rendered.contains("t=10000"), "{rendered}");
    assert!(rendered.contains("frame 249"), "{rendered}");
}
