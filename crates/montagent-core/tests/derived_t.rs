//! `t_from` (#328, ADR-0086): the closed rule set as the schema publishes it, the schema
//! errors the deserializer names, and `R-DERIVED-T`.
//!
//! ADR-0086 is the decision and this is the implementation, so the two artifacts #168
//! requires to agree are both exercised here: a file the binary refuses must be a file the
//! published schema refuses too, and the rule set must be named in one place.

use montagent_core::finding::{Class, Finding, Repair};
use montagent_core::model::Element;
use montagent_core::report::Report;
use montagent_core::{checks, parse, schema};
use serde_json::{Value, json};

mod common;
use common::{canonical, write_project};

/// A `rect` carrying `scale` as the given raw keyframe-record JSON, starting at `start`.
fn element(id: &str, start: i64, records: &str) -> String {
    format!(
        r##"{{"id":"{id}","type":"rect","start":{start},"end":20000,"x":0,"y":0,"origin":"top-left","width":100,"height":100,"fill":"#1E344C","scale":[{records}]}}"##
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
    let path = write_project(&dir, "p.montagent.json", &project(elements));
    let document = parse::read(&path).expect("the project parses");
    let mut report = Report::new("validate", Some(document.path().to_string()));
    checks::derived::check(&document, &mut report);
    report
}

#[track_caller]
fn stale(report: &Report) -> Vec<&Finding> {
    report
        .findings
        .iter()
        .filter(|f| f.code == "R-DERIVED-T")
        .collect()
}

/// The message the types refuse this record list with, or the panic that they did not.
///
/// Read as one [`Element`], which is the scope `crate::checks::schema` parses at and so the
/// scope whose message a reader of a report actually sees.
#[track_caller]
fn refusal(records: &str) -> String {
    serde_json::from_str::<Element>(&element("a", 0, records))
        .expect_err("the types refuse it")
        .to_string()
}

/// The prose one report renders to.
#[track_caller]
fn prose(report: &Report) -> String {
    montagent_core::text::render(&report.to_json(), montagent_core::text::Options::default())
        .expect("every template's fields are carried")
}

// ---- The happy path: the fixture's own shape. -------------------------------------------

#[test]
fn a_ramp_declaring_both_rules_correctly_is_silent() {
    // The shape all seven of the fixture's Ken Burns ramps carry: `element-start` on the
    // first record, `after-previous` with `ms: 15000` on the second.
    let records = r##"{"t":3018,"t_from":{"rule":"element-start"},"v":[1.0,1.0]},{"t":18018,"t_from":{"rule":"after-previous","ms":15000},"v":[1.08,1.08],"ease":"linear"}"##;
    let report = report_on(&element("photo-05", 3018, records));
    assert!(stale(&report).is_empty(), "{:?}", report.findings);
}

#[test]
fn an_absent_declaration_is_no_claim_and_never_a_claim_of_independence() {
    // ADR-0086's sixth condition of admission. Both records here sit exactly where
    // `element-start` and `after-previous ms=15000` would put them, and nothing fires —
    // the mechanism is structurally blind to undeclared relationships, deliberately, and
    // the hole must not be patched by inference (ADR-0063's population explosion).
    let records = r##"{"t":3018,"v":[1.0,1.0]},{"t":18018,"v":[1.08,1.08],"ease":"linear"}"##;
    let report = report_on(&element("photo-05", 3018, records));
    assert!(stale(&report).is_empty(), "{:?}", report.findings);
}

#[test]
fn a_zero_ms_after_previous_is_legal_and_only_fires_on_the_arithmetic() {
    // Non-negative, not positive: `0` is a declaration that two records were meant to
    // coincide, and ADR-0082 already refuses a list that writes them so. The pair below
    // therefore derives 3018 for a record written at 18018 — a finding about the
    // arithmetic, not about the `0`.
    let records = r##"{"t":3018,"t_from":{"rule":"element-start"},"v":[1.0,1.0]},{"t":18018,"t_from":{"rule":"after-previous","ms":0},"v":[1.08,1.08],"ease":"linear"}"##;
    let report = report_on(&element("photo-05", 3018, records));
    let findings = stale(&report);
    assert_eq!(findings.len(), 1, "{:?}", report.findings);
    assert_eq!(findings[0].fields["derived"], json!(3018));
}

// ---- `R-DERIVED-T`. ---------------------------------------------------------------------

#[test]
fn a_drifted_after_previous_is_an_error_whose_repair_states_the_re_derived_integer() {
    // ADR-0086's own worked example: the ramp start moved to 17000 and the declaration was
    // left behind, so `after-previous ms=15000` derives 32000 where 32472 is written.
    let records = r##"{"t":17000,"t_from":{"rule":"element-start"},"v":[1.0,1.0]},{"t":32472,"t_from":{"rule":"after-previous","ms":15000},"v":[1.08,1.08],"ease":"linear"}"##;
    let report = report_on(&element("photo-06", 17000, records));
    let findings = stale(&report);
    assert_eq!(findings.len(), 1, "{:?}", report.findings);

    let finding = findings[0];
    // `error`, not `review` — the overturn ADR-0086's second jury round adopted. A violated
    // declaration has already said it was not meant, and no frame can say which of two
    // numbers is stale.
    assert_eq!(finding.class, Class::Error);
    assert_eq!(finding.fields["element"], json!("photo-06"));
    assert_eq!(finding.fields["property"], json!("scale"));
    assert_eq!(finding.fields["declared_t"], json!(32472));
    assert_eq!(finding.fields["rule"], json!("after-previous"));
    assert_eq!(finding.fields["derived"], json!(32000));
    // Advise-class: the author supplied the missing determinant, so exactly one integer is
    // legal and the repair states it.
    assert_eq!(finding.repair, Some(Repair::Advise(json!({"t": 32000}))));

    let rendered = prose(&report);
    assert!(rendered.contains("R-DERIVED-T"), "{rendered}");
    assert!(rendered.contains("photo-06`.scale"), "{rendered}");
    assert!(rendered.contains("t=32472"), "{rendered}");
    assert!(
        rendered.contains("the previous record's `t` is 17000 and `ms` is 15000"),
        "{rendered}"
    );
    assert!(rendered.contains("derives 32000"), "{rendered}");
    assert!(rendered.contains("drop the `t_from`"), "{rendered}");
}

#[test]
fn a_drifted_element_start_re_derives_from_the_elements_own_start() {
    let records = r##"{"t":3018,"t_from":{"rule":"element-start"},"v":[1.0,1.0]}"##;
    let report = report_on(&element("photo-05", 4000, records));
    let findings = stale(&report);
    assert_eq!(findings.len(), 1, "{:?}", report.findings);
    assert_eq!(findings[0].fields["rule"], json!("element-start"));
    assert_eq!(findings[0].fields["derived"], json!(4000));
    assert_eq!(findings[0].fields["declared_t"], json!(3018));
    assert_eq!(findings[0].repair, Some(Repair::Advise(json!({"t": 4000}))));
}

#[test]
fn a_shift_that_moves_source_and_derived_together_stays_silent() {
    // ADR-0086, stated as a limit rather than buried: the declaration asserts the relation
    // holds, not that it should have changed. Both records and the element's `start` are the
    // fixture's photo-06 plus 1000 ms.
    let records = r##"{"t":18472,"t_from":{"rule":"element-start"},"v":[1.0,1.0]},{"t":33472,"t_from":{"rule":"after-previous","ms":15000},"v":[1.08,1.08],"ease":"linear"}"##;
    let report = report_on(&element("photo-06", 18472, records));
    assert!(stale(&report).is_empty(), "{:?}", report.findings);
}

#[test]
fn every_animatable_property_is_read_not_only_the_ones_that_move_a_box() {
    // The fixture's fourteen declarations are all on `scale`, which is a fact about the
    // fixture and not about the field: a `t` is a `t` whatever it animates.
    let element = r##"{"id":"fade","type":"rect","start":500,"end":20000,"x":0,"y":0,"origin":"top-left","width":100,"height":100,"fill":"#1E344C","opacity":[{"t":0,"t_from":{"rule":"element-start"},"v":0.0},{"t":900,"v":1.0,"ease":"linear"}]}"##;
    let report = report_on(element);
    let findings = stale(&report);
    assert_eq!(findings.len(), 1, "{:?}", report.findings);
    assert_eq!(findings[0].fields["property"], json!("opacity"));
    assert_eq!(findings[0].fields["derived"], json!(500));
}

#[test]
fn the_committed_fixture_carries_fourteen_declarations_and_none_of_them_is_stale() {
    // ADR-0015 measured that agents author by copying the nearest example, so a field absent
    // from the fixture is a field that does not really exist — which is why #328 lands the
    // schema change and the fixture's declarations in one change.
    let path = "../../fixtures/en-halloween-decorating/en-halloween-decorating.montagent.json";
    let document = parse::read(std::path::Path::new(path)).expect("the fixture parses");

    // Every `t_from` anywhere in the file, found by walking the tree rather than the
    // animatable list: the count is the claim, and a walk that knew which properties to look
    // under could not notice a fifteenth written somewhere else.
    let mut declared: Vec<&Value> = Vec::new();
    collect(document.value(), &mut declared);
    assert_eq!(declared.len(), 14, "the 7 ramps take two each");
    assert_eq!(
        declared
            .iter()
            .filter(|t_from| t_from["rule"] == json!("element-start"))
            .count(),
        7
    );
    assert_eq!(
        declared
            .iter()
            .filter(|t_from| ***t_from == json!({"rule": "after-previous", "ms": 15000}))
            .count(),
        7
    );

    let mut report = Report::new("validate", Some(document.path().to_string()));
    checks::derived::check(&document, &mut report);
    assert!(
        stale(&report).is_empty(),
        "the fixture is the regression guard: {:?}",
        report.findings
    );
}

// ---- The schema errors. ----------------------------------------------------------------

/// Every `t_from` in `value`, at any depth.
fn collect<'a>(value: &'a Value, found: &mut Vec<&'a Value>) {
    match value {
        Value::Object(body) => {
            for (key, child) in body {
                if key == "t_from" {
                    found.push(child);
                } else {
                    collect(child, found);
                }
            }
        }
        Value::Array(items) => items.iter().for_each(|item| collect(item, found)),
        _ => {}
    }
}

#[test]
fn an_unknown_rule_names_the_closed_set() {
    let records = r##"{"t":0,"t_from":{"rule":"element-end"},"v":[1.0,1.0]}"##;
    let message = refusal(records);
    assert!(message.contains("`element-end`"), "{message}");
    assert!(message.contains("`element-start`"), "{message}");
    assert!(message.contains("`after-previous`"), "{message}");
    assert!(message.contains("closed and published"), "{message}");
}

#[test]
fn ms_on_a_rule_that_takes_no_argument_is_refused_and_names_the_rule_that_takes_one() {
    let records = r##"{"t":0,"t_from":{"rule":"element-start","ms":15000},"v":[1.0,1.0]}"##;
    let message = refusal(records);
    assert!(message.contains("takes no argument"), "{message}");
    assert!(message.contains("`after-previous`"), "{message}");
}

#[test]
fn ms_absent_on_after_previous_is_refused_and_no_default_is_published() {
    let records = r##"{"t":0,"v":[1.0,1.0]},{"t":15000,"t_from":{"rule":"after-previous"},"v":[1.08,1.08],"ease":"linear"}"##;
    let message = refusal(records);
    assert!(message.contains("has no default"), "{message}");
    assert!(
        !schema::generated_bytes().contains("\"default\": 0"),
        "no default is published for `ms` either"
    );
}

#[test]
fn a_negative_ms_is_refused_in_the_rules_own_words() {
    let records = r##"{"t":0,"v":[1.0,1.0]},{"t":15000,"t_from":{"rule":"after-previous","ms":-15000},"v":[1.08,1.08],"ease":"linear"}"##;
    let message = refusal(records);
    assert!(message.contains("non-negative integer"), "{message}");
    // Direction is carried in the rule name, never in the sign of the argument, so the
    // message says what a negative offset would have had to mean and that nothing means it.
    assert!(message.contains("`after-previous`"), "{message}");
}

#[test]
fn after_previous_on_the_first_record_of_a_list_is_refused() {
    let records = r##"{"t":15000,"t_from":{"rule":"after-previous","ms":15000},"v":[1.0,1.0]}"##;
    let message = refusal(records);
    assert!(
        message.contains("nothing comes before the first"),
        "{message}"
    );
    // Names the rule a first record *can* carry, so the next move is in the message.
    assert!(message.contains(r#""rule": "element-start""#), "{message}");
}

#[test]
fn element_start_is_legal_on_the_first_record_and_on_any_other() {
    // The first record is where the fixture's seven carry it; a later record declaring it is
    // ordinary too — a ramp whose second keyframe was derived from the element's own start is
    // a thing an author may mean, and nothing in ADR-0086 makes position the rule's business.
    let records = r##"{"t":0,"t_from":{"rule":"element-start"},"v":[1.0,1.0]},{"t":15000,"t_from":{"rule":"element-start"},"v":[1.08,1.08],"ease":"linear"}"##;
    serde_json::from_str::<Element>(&element("a", 0, records)).expect("both positions are legal");
}

#[test]
fn a_key_the_format_does_not_publish_inside_a_t_from_is_an_unknown_key() {
    // Phrased as `serde`'s own unknown field, deliberately: that is what
    // `crate::checks::schema` classifies as `E-SCHEMA-UNKNOWN-KEY`, which carries ADR-0016's
    // "it may belong to a newer format revision — do not delete the key to make the file
    // validate". An `ms` on `element-start` is *not* phrased that way, because there
    // deleting the key is the fix.
    let records = r##"{"t":0,"t_from":{"rule":"element-start","from":"photo-05"},"v":[1.0,1.0]}"##;
    let message = refusal(records);
    assert!(message.contains("unknown field `from`"), "{message}");
    assert!(
        message.contains("expected one of `rule`, `ms`"),
        "{message}"
    );
}

// ---- The published schema says the same things. ------------------------------------------

#[test]
fn the_record_shapes_publish_t_from_immediately_after_t() {
    // ADR-0041 derives canonical order from schema order, so placement is the whole of
    // ADR-0086's key-order specification — and the schema is where it is legible.
    let schema = schema::generate();
    let defs = schema["$defs"].as_object().expect("the definitions");

    let mut seen = 0;
    for (name, def) in defs {
        let keys: Vec<&str> = def["properties"]
            .as_object()
            .map(|body| body.keys().map(String::as_str).collect())
            .unwrap_or_default();
        if name.ends_with("Keyframe") || name.contains("Keyframe") {
            seen += 1;
            let expected: Vec<&str> = if name.starts_with("First") {
                vec!["t", "t_from", "v"]
            } else {
                vec!["t", "t_from", "v", "ease"]
            };
            assert_eq!(keys, expected, "{name}");
        }
    }
    assert_eq!(
        seen, 42,
        "twenty-one keyframe instantiations, first and non-first"
    );
}

#[test]
fn the_first_records_t_from_publishes_the_rule_set_minus_after_previous() {
    let schema = schema::generate();
    let defs = &schema["$defs"];

    let rules = |name: &str| -> Vec<String> {
        defs[name]["oneOf"]
            .as_array()
            .unwrap_or_else(|| panic!("`{name}` is a union"))
            .iter()
            .map(|branch| branch["properties"]["rule"]["const"].to_string())
            .collect()
    };
    assert_eq!(
        rules("Derivation"),
        ["\"element-start\"", "\"after-previous\""]
    );
    assert_eq!(rules("FirstDerivation"), ["\"element-start\""]);

    // Every instantiation, whatever value type it is named for (ADR-0137 §6).
    let records: Vec<&String> = defs
        .as_object()
        .unwrap()
        .keys()
        .filter(|name| name.starts_with("Keyframe") || name.starts_with("FirstKeyframe"))
        .collect();
    assert_eq!(records.len(), 42, "{records:?}");
    for name in records {
        let derivation = if name.starts_with("First") {
            "#/$defs/FirstDerivation"
        } else {
            "#/$defs/Derivation"
        };
        assert_eq!(
            defs[name]["properties"]["t_from"]["$ref"],
            json!(derivation),
            "{name}"
        );
    }
    assert!(
        defs["FirstDerivation"]["description"]
            .as_str()
            .is_some_and(|said| said.contains("nothing comes before the first")),
        "the narrowing names its reason, as the deserializer's own message does"
    );
}

#[test]
fn the_schema_states_the_ms_bound_the_deserializer_states_in_words() {
    // #168's two-artifact rule: the published schema must not admit files the binary
    // refuses. A negative `ms` is refused by `crate::model::keyframe`, so the schema says so
    // too — as an unsigned integer with its own `minimum`.
    let ms = &schema::generate()["$defs"]["Derivation"]["oneOf"][1]["properties"]["ms"];
    assert_eq!(ms["type"], json!("integer"));
    assert_eq!(ms["minimum"], json!(0));
}

#[test]
fn t_from_is_never_required_anywhere_the_schema_publishes_it() {
    // Optional, and the reason is measured: required would put an escape value on 121 of the
    // fixture's 135 instants to check 14, and on a format whose authors copy the nearest
    // example that produces 121 copies of the escape value rather than 121 decisions.
    let schema = schema::generate();
    for (name, def) in schema["$defs"].as_object().expect("the definitions") {
        let required = def["required"].as_array().cloned().unwrap_or_default();
        assert!(
            !required.contains(&json!("t_from")),
            "`{name}` requires `t_from`"
        );
    }
}

// ---- What `shift` does to a declaration it moves past. ----------------------------------

#[test]
fn a_segment_cut_leaves_a_stale_declaration_and_shift_hands_back_the_finding() {
    // ADR-0086 states the case where a shift moves source and derived together and the
    // relation survives. This is the other case, which it does not cover: `shift` splices two
    // records into a segment, so a record declaring `after-previous` has a new previous and
    // its arithmetic no longer holds.
    //
    // The declaration is left exactly as written. `shift` does not rewrite the `ms` — that
    // would make the tool the second author of a claim only the author can make — and does
    // not drop the key, which would discard the claim silently while meaning *no claim*. The
    // outcome is the one the two ADRs together specify: `validate` re-derives the instant,
    // and ADR-0011's *"every write tool returns the new state's findings"* puts that finding
    // in the agent's hands.
    let dir = common::tempdir(line!());
    let records = r##"{"t":0,"t_from":{"rule":"element-start"},"v":[1.0,1.0]},{"t":10000,"t_from":{"rule":"after-previous","ms":10000},"v":[1.08,1.08],"ease":"linear"}"##;
    let path = write_project(
        &dir,
        "p.montagent.json",
        &project(&element("photo", 0, records)),
    );

    let answer = montagent_core::verbs::shift::shift(
        &path,
        &montagent_core::verbs::shift::Ask {
            // Inside the ramp's one segment, so the cut branch runs rather than a plain move.
            at: 4000,
            delta: 1000,
            scope: None,
            release: Vec::new(),
        },
    );

    let written: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let ramp = written["tracks"][0]["elements"][0]["scale"]
        .as_array()
        .expect("the ramp survives the shift");
    // Four records now: the author's two, plus the cut's pair. Only the author's carry a
    // claim — an instant this verb invented asserts nothing.
    assert_eq!(ramp.len(), 4, "{ramp:#?}");
    assert_eq!(ramp[0]["t_from"], json!({"rule": "element-start"}));
    assert_eq!(ramp[1].get("t_from"), None);
    assert_eq!(ramp[2].get("t_from"), None);
    assert_eq!(
        ramp[3]["t_from"],
        json!({"rule": "after-previous", "ms": 10000}),
        "the author's claim is neither rewritten nor dropped"
    );

    // And the write tool says so, rather than leaving the file to be found broken later.
    let findings = stale(answer.report());
    assert_eq!(findings.len(), 1, "{:?}", answer.report().findings);
    assert_eq!(findings[0].class, Class::Error);
    assert_eq!(
        findings[0].fields["declared_t"], ramp[3]["t"],
        "the finding is about the record that still declares one"
    );
}
