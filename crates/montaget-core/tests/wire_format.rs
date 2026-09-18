//! ADR-0006's wire decision: JSON is canonical, the text form is *generated from it*,
//! and `--json` replaces the text output rather than accompanying it.

use montaget_core::finding::Finding;
use montaget_core::report::Report;
use montaget_core::text;
use serde_json::json;

fn a_report() -> Report {
    let mut report = Report::new(
        "validate",
        Some("en-halloween-decorating.montaget.json".into()),
    );
    report.push(
        Finding::new("E-SOURCE-OVERRUN")
            .at_file("en-halloween-decorating.montaget.json")
            .at_element("vo-sentence-06-a")
            // #203 gave the check its implementation, and with it the fields ADR-0006's
            // "every relevant number inline" asks for: which duration was measured
            // against (ADR-0011 returns four and forces a pick), the declared range, and
            // how far past the file it reaches. `speed` is not among them — whether a
            // range names bytes the file holds is a question about the source alone.
            .field("source", json!("audio/sentence-06-spider.mp3"))
            .field("axis", json!("audio stream"))
            .field("source_start", json!(0))
            .field("source_end", json!(3368))
            .field("declared_source_span", json!(3368))
            .field("probed_duration", json!(2568))
            .field("over_by", json!(800)),
    );
    report.push(
        Finding::new("N-QUANTIZATION")
            .at_file("en-halloween-decorating.montaget.json")
            .field("changed", json!(0)),
    );
    report
}

#[test]
fn the_text_report_is_generated_from_the_json_and_from_nothing_else() {
    // The acceptance criterion, asserted structurally: the canonical JSON is serialised
    // to a string, parsed back with no access to the `Report` at all, and rendered. If
    // the text form were assembled from anything the JSON does not carry, this would
    // differ from the render of the live value.
    let report = a_report();

    let from_live = text::render(&report.to_json(), text::Options::verbose()).unwrap();

    let wire = serde_json::to_string(&report.to_json()).unwrap();
    let reparsed: serde_json::Value = serde_json::from_str(&wire).unwrap();
    let from_wire = text::render(&reparsed, text::Options::verbose()).unwrap();

    assert_eq!(from_live, from_wire);
    assert!(from_wire.contains("2568"), "{from_wire}");
}

#[test]
fn the_canonical_json_carries_the_summary_the_exit_code_and_the_boundary() {
    let json = a_report().to_json();

    assert_eq!(json["tool"], "validate");
    assert_eq!(json["project"], "en-halloween-decorating.montaget.json");
    assert_eq!(json["summary"]["error"], 1);
    assert_eq!(json["summary"]["note"], 1);
    assert_eq!(json["exit_code"], 1);
    assert!(
        json["not_checked"]
            .as_str()
            .unwrap()
            .contains("internally legal"),
        "the report's own boundary travels in the JSON too (ADR-0006)"
    );
    assert_eq!(json["findings"].as_array().unwrap().len(), 2);
}

#[test]
fn informational_classes_collapse_to_one_counted_line_and_expand_on_request() {
    // ADR-0006's noise budget: "errors and near-errors print in full; informational
    // classes collapse to one counted line carrying their code, expandable on request."
    let mut report = Report::new("validate", Some("p.json".into()));
    for i in 0..47 {
        report.push(
            Finding::new("N-QUANTIZATION")
                .at_file("p.json")
                .at_element(format!("el-{i:02}"))
                .field("changed", json!(i)),
        );
    }
    let json = report.to_json();

    let collapsed = text::render(&json, text::Options::default()).unwrap();
    assert_eq!(
        collapsed.matches("N-QUANTIZATION").count(),
        1,
        "one counted line:\n{collapsed}"
    );
    assert!(collapsed.contains("47"), "carrying its count:\n{collapsed}");

    let expanded = text::render(&json, text::Options::verbose()).unwrap();
    assert_eq!(expanded.matches("N-QUANTIZATION").count(), 47);
}

#[test]
fn errors_and_reviews_always_print_in_full() {
    let mut report = Report::new("validate", Some("p.json".into()));
    for i in 0..3 {
        report.push(
            Finding::new("R-VISUAL-GAP")
                .at_file("p.json")
                .field("from", json!(i * 1000))
                .field("to", json!(i * 1000 + 800)),
        );
    }

    let rendered = text::render(&report.to_json(), text::Options::default()).unwrap();
    assert_eq!(rendered.matches("R-VISUAL-GAP").count(), 3, "{rendered}");
}

#[test]
fn an_unknown_code_in_the_json_is_a_render_error_not_invented_prose() {
    // One code, one field set, one template (ADR-0006). A finding whose code the
    // registry does not know has no template, and the renderer must say so rather than
    // improvise a sentence.
    let json = json!({
        "tool": "validate",
        "project": "p.json",
        "summary": {"error": 1, "review": 0, "note": 0, "unchecked": 0, "layout": 0},
        "exit_code": 1,
        "findings": [{"code": "E-MADE-UP", "class": "error", "location": {"file": "p.json"}, "fields": {}}],
        "not_checked": "…"
    });

    let err = text::render(&json, text::Options::default()).unwrap_err();
    assert!(format!("{err}").contains("E-MADE-UP"));
}
