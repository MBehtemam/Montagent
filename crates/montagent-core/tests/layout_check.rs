//! `LAYOUT` inside `validate` (ADR-0041, #198): unconditional, shared with `fmt`, and never
//! a gate.
//!
//! `tests/fmt.rs` asserts what the verb does with these findings. This file asserts the
//! three things ADR-0041 says about them that are *not* `fmt`'s business: that `validate`
//! produces them on a file nobody ran a formatter over, that they are the same findings
//! from the same function rather than a second opinion, and that a file carrying them still
//! renders.

use montagent_core::finding::Class;
use montagent_core::report::ExitCode;
use montagent_core::verbs::fmt::{self, Mode};
use montagent_core::{text, validate};

mod common;
use common::write_project;

/// A project whose element writes `type` before `id` and whose keys are otherwise
/// canonical — the `jq`-scripted edit that reorders keys without touching a line break.
///
/// Written out by hand rather than through `common::canonical`, because the bytes are the
/// subject here.
const REORDERED: &str = r##"{
  "frame": {"width": 1080, "height": 1920},
  "fps": 25,
  "tracks": [
    {
      "name": "panels",
      "layer": 10,
      "elements": [
        {"type":"rect","id":"panel-01","start":0,"end":1000,"width":100,"height":100}
      ]
    }
  ]
}
"##;

/// The same document, pretty-printed — the 154 → 1595 incident in miniature. Not one key
/// moves; every line does.
const PRETTY_PRINTED: &str = r##"{
  "frame": {
    "width": 1080,
    "height": 1920
  },
  "fps": 25,
  "tracks": [
    {
      "name": "panels",
      "layer": 10,
      "elements": [
        {
          "id": "panel-01",
          "type": "rect",
          "start": 0,
          "end": 1000,
          "width": 100,
          "height": 100
        }
      ]
    }
  ]
}
"##;

#[track_caller]
fn validate_source(body: &str) -> montagent_core::report::Report {
    let dir = common::tempdir(std::panic::Location::caller().line());
    validate(&write_project(&dir, "p.montagent.json", body))
}

fn codes(report: &montagent_core::report::Report) -> Vec<&str> {
    report.findings.iter().map(|f| f.code.as_str()).collect()
}

#[test]
fn validate_reports_a_key_order_violation_without_anyone_running_a_formatter() {
    // ADR-0041 rejected `fmt --check`-only outright: the incident agent "was not running a
    // formatter, it believed it was making a routine edit and had no reason to invoke one."
    let report = validate_source(REORDERED);

    assert_eq!(codes(&report), ["L-KEY-ORDER"]);
    let finding = &report.findings[0];
    assert_eq!(finding.class, Class::Layout);
    assert_eq!(finding.location.element.as_deref(), Some("panel-01"));
    assert_eq!(
        finding.location.line,
        Some(9),
        "the line to open the editor at"
    );
    assert_eq!(finding.fields["type"], "rect");
    assert_eq!(
        finding.fields["expected"], "id,type,start,end,width,height",
        "only the keys the element carries — naming the omitted ones would read as fields \
         to add (ADR-0030)"
    );
}

#[test]
fn validate_reports_a_pretty_printed_file_that_moved_no_key_at_all() {
    // The incident's own shape: 154 lines became 1595 "without disturbing a single key's
    // position", so a check carrying only `L-KEY-ORDER` would have reported nothing on the
    // very file ADR-0041 was written about.
    let report = validate_source(PRETTY_PRINTED);

    assert_eq!(codes(&report), ["L-LAYOUT"]);
    assert_eq!(report.findings[0].class, Class::Layout);
    assert_eq!(report.findings[0].fields["written_lines"], 23);
    assert_eq!(report.findings[0].fields["canonical_lines"], 13);
}

#[test]
fn a_layout_finding_never_gates_the_render() {
    // ADR-0041: `LAYOUT` is "not one of the categories `render` refuses on" — refusal stays
    // keyed to `error` alone, because the video is byte-identical either way. `render` is a
    // later ticket (#215), so the claim is asserted where it is decided: the class is not a
    // severity, it does not reach the error count, and the run exits 0.
    for body in [REORDERED, PRETTY_PRINTED] {
        let report = validate_source(body);

        assert!(!report.findings.is_empty(), "the file is non-canonical");
        for finding in &report.findings {
            assert_eq!(finding.class, Class::Layout);
            assert!(
                !finding.class.is_severity(),
                "`LAYOUT` is a report category, not one of ADR-0006's three severities"
            );
        }
        assert_eq!(report.summary().error, 0);
        assert_eq!(report.summary().layout, report.findings.len());
        assert_eq!(
            report.exit_code(),
            ExitCode::Ok,
            "a key-order violation is unsafe to edit, not unsafe to render"
        );
    }
}

#[test]
fn validate_and_fmt_check_report_the_identical_findings() {
    // ADR-0041: the two "share one implementation of 'what does canonical form look like' …
    // so there is exactly one place the rule lives, not two that can disagree." Two reports
    // that agree field for field is what one implementation looks like from outside; the
    // inside is that `fmt` calls `checks::layout` and owns no predicate of its own.
    for body in [REORDERED, PRETTY_PRINTED] {
        let dir = common::tempdir(std::panic::Location::caller().line());
        let path = write_project(&dir, "p.montagent.json", body);

        let validated = validate(&path).findings;
        let checked = fmt::fmt(&path, Mode::Check).findings;

        assert_eq!(validated, checked, "one rule, asked from two places");
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            body,
            "`--check` writes nothing"
        );
    }
}

#[test]
fn the_key_order_a_finding_names_is_the_published_schemas_own() {
    // The predicate is `layout::is_canonical`, which is defined as "`reorder` would change
    // nothing" and reads its order out of the generated schema. A second traversal that
    // answered the same question is the two-implementations-one-rule shape ADR-0041 was
    // written against, so the finding's `expected` is compared against the schema rather
    // than against a list written down here.
    let expected =
        montagent_core::layout::canonical_order(montagent_core::layout::Published::Element("rect"))
            .expect("rect is a published type");
    let report = validate_source(REORDERED);

    let named: Vec<&str> = report.findings[0].fields["expected"]
        .as_str()
        .unwrap()
        .split(',')
        .collect();
    assert!(
        named
            .iter()
            .eq(expected.iter().filter(|key| named.contains(&key.as_str()))),
        "the finding names the schema's order, in the schema's order: {named:?} against \
         {expected:?}"
    );
}

#[test]
fn a_canonical_file_produces_no_layout_finding_at_all() {
    // ADR-0041: layout compliance "fires on exactly the non-compliant files and never on a
    // correct one", which is what keeps it out of `validate`'s noise budget. The committed
    // fixture is the strongest available case — 154 lines, 60 elements, published.
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/en-halloween-decorating/en-halloween-decorating.montagent.json");
    let document = montagent_core::parse::read(&fixture).expect("the committed fixture");

    let mut report = montagent_core::report::Report::new("validate", None);
    montagent_core::checks::layout::check(&document, &mut report);

    assert!(report.findings.is_empty(), "{:?}", report.findings);
}

#[test]
fn the_summary_line_counts_layout_separately_from_every_severity() {
    // ADR-0041 gave `LAYOUT` a fourth report category "the same shape as `UNCHECKED`":
    // counted in the summary line, never a verdict.
    let rendered = text::render(
        &validate_source(REORDERED).to_json(),
        text::Options::default(),
    )
    .unwrap();
    let summary = rendered.lines().next().unwrap();

    assert!(summary.starts_with("0 errors"), "got: {summary}");
    assert!(summary.contains("1 layout"), "got: {summary}");
}
