//! `R-OFF-CANVAS` (#211, ADR-0044): a visual element whose declared rect, resolved across
//! its entire active range, never intersects the frame at any instant.
//!
//! A **standing** check, not a frame-change census: nothing here edits a `frame` first,
//! because the finding is a fact about the current file however it came to be one.

use montagent_core::finding::{Class, Finding};
use montagent_core::report::Report;
use montagent_core::{checks, parse};

mod common;
use common::{canonical, write_project};

fn project(elements: &str) -> String {
    canonical(&format!(
        r##"{{"frame":{{"width":1080,"height":1920}},"fps":25,"tracks":[{{"name":"t","layer":10,"elements":[{elements}]}}]}}"##
    ))
}

/// A static rectangle whose box is exactly `x`,`y`,`width`,`height`.
fn rect(id: &str, x: i64, y: i64, width: i64, height: i64) -> String {
    format!(
        r##"{{"id":"{id}","type":"rect","start":0,"end":10000,"x":{x},"y":{y},"origin":"top-left","width":{width},"height":{height},"fill":"#1E344C"}}"##
    )
}

/// The same rectangle with `x` animated through the given `(t, v)` records.
fn sliding(id: &str, xs: &[(i64, i64)], y: i64, width: i64, height: i64) -> String {
    let records: Vec<String> = xs
        .iter()
        .enumerate()
        .map(|(i, (t, v))| match i {
            0 => format!(r##"{{"t":{t},"v":{v}}}"##),
            _ => format!(r##"{{"t":{t},"v":{v},"ease":"linear"}}"##),
        })
        .collect();
    format!(
        r##"{{"id":"{id}","type":"rect","start":0,"end":10000,"x":[{}],"y":{y},"origin":"top-left","width":{width},"height":{height},"fill":"#1E344C"}}"##,
        records.join(",")
    )
}

#[track_caller]
fn report_on(elements: &str) -> Report {
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(&dir, "p.montagent.json", &project(elements));
    let document = parse::read(&path).expect("the project parses");
    let mut report = Report::new("validate", Some(document.path().to_string()));
    checks::canvas::check(&document, &mut report);
    report
}

#[track_caller]
fn off_canvas(report: &Report) -> Vec<&Finding> {
    report
        .findings
        .iter()
        .filter(|f| f.code == "R-OFF-CANVAS")
        .collect()
}

#[test]
fn an_element_parked_entirely_outside_the_frame_is_reported() {
    // ADR-0012's measured defect, reduced to one element: a rect never repositioned after
    // a retarget, sitting nowhere near the canvas for its entire life.
    let report = report_on(&rect("stale-card", 2400, 48, 984, 169));
    let findings = off_canvas(&report);
    assert_eq!(findings.len(), 1, "{:?}", report.findings);

    let finding = findings[0];
    assert_eq!(finding.class, Class::Review);
    assert_eq!(finding.fields["element"], serde_json::json!("stale-card"));
    // Every number inline: the rect it occupies, and the frame it never reaches.
    assert_eq!(finding.fields["x"], serde_json::json!(2400));
    assert_eq!(finding.fields["width"], serde_json::json!(984));
    assert_eq!(finding.fields["frame_width"], serde_json::json!(1080));
    assert_eq!(finding.fields["frame_height"], serde_json::json!(1920));
    assert_eq!(finding.fields["start"], serde_json::json!(0));
    assert_eq!(finding.fields["end"], serde_json::json!(10000));
}

#[test]
fn an_element_inside_the_frame_is_not_reported() {
    assert!(off_canvas(&report_on(&rect("card", 48, 1453, 984, 169))).is_empty());
}

#[test]
fn an_element_overlapping_the_frame_by_one_pixel_is_not_reported() {
    // The trigger is intersection, not containment: a rect ending at x=1 claims the
    // frame's own first column.
    assert!(off_canvas(&report_on(&rect("bleed", -983, 100, 984, 169))).is_empty());
}

#[test]
fn a_slide_in_that_starts_off_canvas_fires_nothing() {
    // ADR-0044: "an element that is off-canvas at *some* instants of its active range — a
    // slide-in starting at `x:-500` and animating to `x:0` — is ordinary, legal animation
    // vocabulary and fires nothing." Whole-range, never per-instant.
    let sliding = sliding("slide-in", &[(0, -1200), (2000, 48)], 100, 984, 169);
    assert!(off_canvas(&report_on(&sliding)).is_empty());
}

#[test]
fn an_element_that_travels_and_is_off_canvas_throughout_is_reported() {
    // Animated, and never on screen at any sampled instant — the union of where it went
    // is what the finding reports.
    let sliding = sliding("far-away", &[(0, 2000), (10000, 3000)], 100, 100, 100);
    let report = report_on(&sliding);
    let findings = off_canvas(&report);
    assert_eq!(findings.len(), 1, "{:?}", report.findings);
    assert_eq!(findings[0].fields["x"], serde_json::json!(2000));
    // The union spans the travel rather than either end of it.
    assert!(
        findings[0].fields["width"].as_i64().unwrap() > 900,
        "{:?}",
        findings[0].fields
    );
}

#[test]
fn an_audio_element_has_no_spatial_extent_and_is_never_off_canvas() {
    let audio = r##"{"id":"vo","type":"audio","start":0,"end":2568,"source":"audio/intro.mp3","source_start":0,"source_end":2568}"##;
    assert!(off_canvas(&report_on(audio)).is_empty());
}

#[test]
fn a_rotated_element_is_passed_over_rather_than_claimed_about() {
    // A rotated footprint is a parallelogram and this check measures rectangles; a
    // bounding box is larger than what it bounds, so it could never prove the footprint
    // never meets the frame.
    let rotated = rect("spun", 2400, 48, 984, 169).replace(
        r##""fill":"#1E344C""##,
        r##""fill":"#1E344C","rotation":12.0"##,
    );
    assert!(off_canvas(&report_on(&rotated)).is_empty());
}

#[test]
fn the_registry_declares_the_check_live_and_review_class() {
    let spec = montagent_core::registry::spec("R-OFF-CANVAS").expect("registered");
    assert_eq!(spec.default_class(), Class::Review);
    assert_eq!(spec.adr, "ADR-0044");
    // `repair` is an axis of `error` alone (ADR-0043), and off-canvas geometry is legal.
    assert_eq!(spec.repair, None);
}

#[test]
fn the_finding_renders_as_prose_and_validate_runs_the_check() {
    // Through `validate` rather than the check directly: the template is only exercised
    // at the renderer, and a template naming a field the finding does not carry is a
    // defect nothing else catches. The project references no media, so no `ffprobe`.
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(
        &dir,
        "p.montagent.json",
        &project(&rect("stale-card", 2400, 48, 984, 169)),
    );
    let report = montagent_core::validate(&path);
    let rendered =
        montagent_core::text::render(&report.to_json(), montagent_core::text::Options::default())
            .expect("every template's fields are carried");
    assert!(rendered.contains("R-OFF-CANVAS"), "{rendered}");
    assert!(rendered.contains("984\u{d7}169"), "{rendered}");
    assert!(rendered.contains("1080\u{d7}1920"), "{rendered}");
}
