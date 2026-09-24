//! `R-MASK-CIRCLE-NON-SQUARE` (#322, ADR-0084): a `circle` mask whose rect is not square
//! discards part of that rect, and the discarded amount is a number the author never typed.
//!
//! ADR-0084's own worked case: *"a circle on a 1080×1912 rect erases 832 px that the author
//! never typed a number for, which is the signature of a footgun rather than a decision."*
//! `review` rather than `error`, because the behaviour is determinate and ADR-0068 ratified
//! it — and `circle`-specific, because `rect` and `ellipse` are total on their rect.

use montagent_core::finding::{Class, Finding};
use montagent_core::report::Report;
use montagent_core::{checks, parse};
use serde_json::json;

mod common;
use common::{canonical, write_project};

fn project(elements: &str) -> String {
    canonical(&format!(
        r##"{{"frame":{{"width":1080,"height":1920}},"fps":25,"tracks":[{{"name":"t","layer":10,"elements":[{elements}]}}]}}"##
    ))
}

/// A `width`×`height` rectangle carrying one `mask` member verbatim.
fn element(id: &str, width: i64, height: i64, mask: &str) -> String {
    format!(
        r##"{{"id":"{id}","type":"rect","start":0,"end":1000,"x":0,"y":0,"origin":"top-left","width":{width},"height":{height},"fill":"#1E344C","effects":[{mask}]}}"##
    )
}

#[track_caller]
fn report_on(elements: &str) -> Report {
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(&dir, "p.montagent.json", &project(elements));
    let document = parse::read(&path).expect("the project parses");
    let mut report = Report::new("validate", Some(document.path().to_string()));
    checks::mask::check(&document, &mut report);
    report
}

#[track_caller]
fn findings(report: &Report) -> Vec<&Finding> {
    report
        .findings
        .iter()
        .filter(|f| f.code == "R-MASK-CIRCLE-NON-SQUARE")
        .collect()
}

#[test]
fn a_circle_on_a_derived_non_square_rect_is_reported_with_the_adrs_own_arithmetic() {
    // ADR-0084's worked case, spelled as the document spells it: a bare `circle` mask on a
    // full-frame element. The rect is *derived* — nobody wrote 1080 or 1912 into the mask —
    // which is precisely why a check that read only written numbers would be silent here.
    let report = report_on(&element(
        "hero",
        1080,
        1912,
        r##"{"name":"mask","shape":"circle"}"##,
    ));
    let found = findings(&report);
    assert_eq!(found.len(), 1, "{:?}", report.findings);

    let finding = found[0];
    assert_eq!(
        finding.class,
        Class::Review,
        "ADR-0084 resolves it `review`"
    );
    assert_eq!(finding.location.element.as_deref(), Some("hero"));
    assert_eq!(finding.fields["width"], json!(1080));
    assert_eq!(finding.fields["height"], json!(1912));
    assert_eq!(finding.fields["diameter"], json!(1080));
    assert_eq!(
        finding.fields["discarded"],
        json!(832),
        "the ADR's own number for its own case"
    );
    assert_eq!(finding.fields["index"], json!(0));
}

#[test]
fn an_explicit_non_square_rect_is_reported_too() {
    // "derived *or* explicit". The two arities are two declarations (ADR-0030) but they are
    // one parameter set, and the footgun is the same one.
    let report = report_on(&element(
        "hero",
        400,
        400,
        r##"{"name":"mask","shape":"circle","x":0,"y":0,"width":300,"height":100}"##,
    ));
    let found = findings(&report);
    assert_eq!(found.len(), 1, "{:?}", report.findings);
    assert_eq!(found[0].fields["width"], json!(300));
    assert_eq!(found[0].fields["discarded"], json!(200));
    assert!(
        found[0].fields["rect_source"]
            .as_str()
            .is_some_and(|s| s.contains("mask's own")),
        "the finding must say which rect it measured: {:?}",
        found[0].fields["rect_source"]
    );
}

#[test]
fn writing_the_square_rect_is_itself_the_acknowledgement() {
    // ADR-0084: "an author who genuinely wants a 1080×1080 circle on a tall element spells
    // the square rect and the finding falls silent — no suppression mechanism, no
    // annotation, and the file ends up recording the intent rather than leaning on
    // arithmetic." The element is the same tall one the first test reports.
    let report = report_on(&element(
        "hero",
        1080,
        1912,
        r##"{"name":"mask","shape":"circle","x":0,"y":416,"width":1080,"height":1080}"##,
    ));
    assert!(findings(&report).is_empty(), "{:?}", report.findings);
}

#[test]
fn rect_and_ellipse_never_fire_however_oblong_their_rect_is() {
    // "circle is the one shape whose meaning *discards* part of its rect. `rect` is the
    // rect and `ellipse` fills it; both are total, and neither gets a finding — an ellipse
    // inscribed in a non-square rect is an ordinary oval and nobody is surprised by it."
    for shape in ["rect", "ellipse"] {
        let report = report_on(&element(
            "hero",
            1080,
            1912,
            &format!(r##"{{"name":"mask","shape":"{shape}"}}"##),
        ));
        assert!(
            findings(&report).is_empty(),
            "a `{shape}` mask fired: {:?}",
            report.findings
        );
    }
}

#[test]
fn a_square_element_with_a_bare_circle_triggers_nothing() {
    // The committed fixture's own case, in miniature: `handle-logo` is a 68×68 badge
    // carrying `{"name":"mask","shape":"circle"}`, and spec #168 makes a check that fires
    // on the fixture wrong unless an ADR says otherwise. ADR-0084 says the opposite here —
    // "the fixture's square badge triggers nothing".
    let report = report_on(&element(
        "handle-logo",
        68,
        68,
        r##"{"name":"mask","shape":"circle"}"##,
    ));
    assert!(findings(&report).is_empty(), "{:?}", report.findings);
}

#[test]
fn two_circle_masks_on_one_element_are_two_findings_naming_their_own_positions() {
    // Two effects of the same name are ordinary (ADR-0040), so the list index is part of
    // the finding's identity — without it a reader with two masks cannot tell which one
    // the sentence is about.
    let report = report_on(&element(
        "hero",
        300,
        100,
        r##"{"name":"mask","shape":"circle"},{"name":"blur","radius":4},{"name":"mask","shape":"circle","x":0,"y":0,"width":200,"height":100}"##,
    ));
    let found = findings(&report);
    assert_eq!(found.len(), 2, "{:?}", report.findings);
    assert_eq!(found[0].fields["index"], json!(0));
    assert_eq!(found[1].fields["index"], json!(2));
}

#[test]
fn the_finding_renders_the_prose_the_registry_declares() {
    // ADR-0006's one-code-one-template contract, and the half of the finding that carries
    // ADR-0084's two repairs — the ones that exist only because explicit geometry is
    // admitted. A finding that could name neither would be the noise the ADR refuses.
    let report = report_on(&element(
        "hero",
        1080,
        1912,
        r##"{"name":"mask","shape":"circle"}"##,
    ));
    let prose =
        montagent_core::text::render(&report.to_json(), montagent_core::text::Options::default())
            .expect("the template renders");

    assert!(prose.contains("1080×1912"), "{prose}");
    assert!(prose.contains("832 px"), "{prose}");
    assert!(
        prose.contains("Use `ellipse` to fill the rect, or give the mask a square rect."),
        "both repairs must be in the sentence: {prose}"
    );
}
