//! The `Finding` type exercised by its hardest consumers rather than by `E-PARSE` alone
//! (ticket #188's last acceptance criterion): a census-carrying finding, a refuse-class
//! finding, and a citation-carrying finding under ADR-0061's fenced exception.

use montaget_core::finding::{Census, Citation, Class, Finding, Repair};
use montaget_core::report::Report;
use montaget_core::text;
use serde_json::json;

/// ADR-0006: "four of five are 1597, one is 1537" — inert data that carries the fix
/// without proposing it.
fn census_carrying() -> Finding {
    Finding::new("R-BOX-SLACK")
        .at_file("en-halloween-decorating.montaget.json")
        .at_element("sentence-05")
        .field("declared_height", json!(169))
        .field("computed_height", json!(66))
        .field("slack", json!(103))
        .field("derivation", json!("size 55 × line_height 1.2 × 1 line"))
        .field("slack_percent", json!(156))
        .census(
            Census::on("height")
                .group(
                    json!(169),
                    ["sentence-05", "sentence-06", "sentence-07", "sentence-08"],
                )
                .group(json!(137), ["sentence-quiz"]),
        )
}

/// ADR-0043: the fix depends on knowing what the author meant, so the check emits
/// `repair: "none"` for every instance it matches.
fn refuse_class() -> Finding {
    Finding::new("E-RETIRED-KEY")
        .at_file("en-halloween-decorating.montaget.json")
        .at_element("photo-06")
        .field("key", json!("gravity"))
        .field("value", json!("bottom"))
        .refuse_class()
        .census(
            Census::on("clip")
                .group(
                    json!([0, 0, 1080, 1300]),
                    ["photo-05", "photo-07", "photo-08"],
                )
                .group(json!([0, 620, 1080, 1300]), ["photo-06"]),
        )
}

/// ADR-0061: a document-derived fact compared against a cited external threshold, at
/// `review`, stating the raw measurement as its substance.
fn citation_carrying() -> Finding {
    Finding::new("R-CAPTION-PACE")
        .at_file("en-halloween-decorating.montaget.json")
        .at_element("hook-loop")
        .field("measured_cps", json!(25.8))
        .field("threshold_cps", json!(20))
        .citation(Citation {
            threshold: json!(20),
            source: "Netflix and BBC timed-text guidance".into(),
            adr: "ADR-0034".into(),
        })
}

#[test]
fn a_census_carrying_finding_round_trips_and_renders_every_group() {
    let f = census_carrying();
    let json = serde_json::to_value(&f).unwrap();

    assert_eq!(json["census"]["field"], "height");
    assert_eq!(json["census"]["groups"][0]["value"], 169);
    assert_eq!(
        json["census"]["groups"][0]["members"]
            .as_array()
            .unwrap()
            .len(),
        4
    );
    assert_eq!(json["fields"]["slack"], 103);

    let rendered = render_one(f);
    assert!(
        rendered.contains("103"),
        "every relevant number inline:\n{rendered}"
    );
    assert!(rendered.contains("4 at 169"), "{rendered}");
    assert!(rendered.contains("1 at 137"), "{rendered}");
}

#[test]
fn a_census_never_ranks_its_groups() {
    // ADR-0043: "it must not be worded in a way that implies the larger group is the
    // correct one." Declaration order survives; nothing sorts by size.
    let f = Finding::new("E-RETIRED-KEY")
        .at_file("p.json")
        .refuse_class()
        .census(
            Census::on("clip")
                .group(json!("minority"), ["a"])
                .group(json!("majority"), ["b", "c", "d"]),
        );
    let json = serde_json::to_value(&f).unwrap();
    assert_eq!(json["census"]["groups"][0]["value"], "minority");
}

#[test]
fn a_refuse_class_finding_says_none_in_json_and_says_so_in_words() {
    let f = refuse_class();
    let json = serde_json::to_value(&f).unwrap();

    assert_eq!(json["class"], "error");
    assert_eq!(
        json["repair"], "none",
        "ADR-0043: the literal string \"none\""
    );

    let rendered = render_one(f);
    assert!(
        rendered.contains("refuse-class"),
        "the prose renderer states the class in words (ADR-0043):\n{rendered}"
    );
    // ADR-0043's instruction to the agent — stop, do not repair by ordinary file edit,
    // surface it — rides on this check's own template rather than on the shared
    // refuse-class paragraph, which a malformed-JSON finding also prints. Wrapped for
    // the terminal, so compare on collapsed whitespace.
    let flowed = rendered.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(
        flowed.contains("Surface this finding verbatim to whoever is operating Montaget"),
        "stop, do not repair, surface it (ADR-0043):\n{rendered}"
    );
    assert!(
        flowed.contains("do not repair it by ordinary file edit"),
        "{rendered}"
    );
}

#[test]
fn an_advise_class_finding_carries_its_structured_value() {
    let f = Finding::new("E-KEYFRAME-EASE")
        .at_file("p.json")
        .at_element("photo-06")
        .field("property", json!("scale"))
        .field("t", json!(17472))
        .advise_class(json!({"value": {"ease": "linear"}}));
    let json = serde_json::to_value(&f).unwrap();

    assert_eq!(json["repair"]["value"]["ease"], "linear");
    assert!(render_one(f).contains("advise-class"));
}

#[test]
fn a_citation_carrying_finding_states_the_measurement_and_cites_its_source() {
    let f = citation_carrying();
    let json = serde_json::to_value(&f).unwrap();

    assert_eq!(json["class"], "review", "never `error` (ADR-0061)");
    assert_eq!(
        json["citation"]["source"],
        "Netflix and BBC timed-text guidance"
    );
    assert_eq!(json["citation"]["adr"], "ADR-0034");
    assert_eq!(json["fields"]["measured_cps"], 25.8);

    let rendered = render_one(f);
    assert!(
        rendered.contains("25.8"),
        "the raw measured fact:\n{rendered}"
    );
    assert!(
        rendered.contains("Netflix and BBC timed-text guidance"),
        "cited inline in the finding (ADR-0061):\n{rendered}"
    );
}

#[test]
fn the_three_fixtures_share_one_report_and_one_summary() {
    let mut report = Report::new(
        "validate",
        Some("en-halloween-decorating.montaget.json".into()),
    );
    report.push(census_carrying());
    report.push(refuse_class());
    report.push(citation_carrying());

    let summary = report.summary();
    assert_eq!(summary.error, 1);
    assert_eq!(summary.review, 1);
    assert_eq!(summary.note, 1);
    assert_eq!(report.exit_code(), montaget_core::report::ExitCode::Errors);
}

#[test]
fn every_class_has_a_home_in_the_summary() {
    let mut report = Report::new("validate", Some("p.json".into()));
    for (code, class) in [
        ("U-SOURCE-UNPROBEABLE", Class::Unchecked),
        ("L-KEY-ORDER", Class::Layout),
    ] {
        assert_eq!(
            montaget_core::registry::spec(code).unwrap().default_class(),
            class,
            "{code} is registered as {class:?}"
        );
    }

    report.push(
        Finding::new("U-SOURCE-UNPROBEABLE")
            .at_file("p.json")
            .unchecked_because(montaget_core::finding::UncheckedReason::Timeout),
    );
    report.push(
        Finding::new("L-KEY-ORDER")
            .at_file("p.json")
            .at_element("photo-06"),
    );

    let summary = report.summary();
    assert_eq!(summary.unchecked, 1);
    assert_eq!(summary.layout, 1);
    assert_eq!(
        report.exit_code(),
        montaget_core::report::ExitCode::Ok,
        "UNCHECKED and LAYOUT never gate (ADR-0013, ADR-0041)"
    );
}

#[test]
fn an_unchecked_finding_carries_a_structured_reason() {
    let f = Finding::new("U-SOURCE-UNPROBEABLE")
        .at_file("p.json")
        .field("source", json!("https://cdn.example/clip.mp4"))
        .unchecked_because(montaget_core::finding::UncheckedReason::Http { status: 503 });
    let json = serde_json::to_value(&f).unwrap();

    assert_eq!(json["reason"]["kind"], "http");
    assert_eq!(json["reason"]["status"], 503);
    assert!(render_one(f).contains("503"));
}

#[test]
fn a_repair_serialises_as_the_bare_string_none_not_as_null() {
    assert_eq!(serde_json::to_value(Repair::None).unwrap(), json!("none"));
}

fn render_one(f: Finding) -> String {
    let mut report = Report::new("validate", Some("p.json".into()));
    report.push(f);
    text::render(&report.to_json(), text::Options::verbose()).unwrap()
}
