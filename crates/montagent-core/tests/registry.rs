//! The check registry: the thing every check ticket needs and none should own.

use montagent_core::finding::Class;
use montagent_core::registry::{self, CensusMode, ThresholdProvenance};

#[test]
fn every_registered_error_class_code_has_a_declared_refuse_class() {
    // ADR-0043: the class is decided once, by whoever authors the check, at the moment
    // the check is written. A check that may emit an `error` with no declared repair
    // class is one whose author skipped that decision.
    let missing: Vec<_> = registry::all()
        .iter()
        .filter(|spec| spec.may_error() && spec.repair.is_none())
        .map(|spec| spec.code)
        .collect();

    assert!(
        missing.is_empty(),
        "error-class checks with no declared refuse class: {missing:?}"
    );
}

#[test]
fn only_error_class_checks_declare_a_refuse_class() {
    // `repair` is an axis of `error` alone — ADR-0043 attaches it to "every `error`-class
    // finding", and a `note` carrying one would read as a fourth severity.
    let stray: Vec<_> = registry::all()
        .iter()
        .filter(|spec| !spec.may_error() && spec.repair.is_some())
        .map(|spec| spec.code)
        .collect();

    assert!(
        stray.is_empty(),
        "non-error checks declaring a repair class: {stray:?}"
    );
}

#[test]
fn an_externally_sourced_threshold_is_never_error_class_and_always_cites() {
    // ADR-0061's fenced exception, as a registry invariant rather than a convention.
    for spec in registry::all() {
        if let ThresholdProvenance::External { source, adr } = spec.threshold {
            assert!(
                !spec.may_error(),
                "{}: an external threshold may never be `error`",
                spec.code
            );
            assert!(
                !source.is_empty(),
                "{}: the source must be cited",
                spec.code
            );
            assert!(
                adr.starts_with("ADR-"),
                "{}: the citation must name the introducing ADR, got {adr:?}",
                spec.code
            );
        }
    }
}

#[test]
fn every_registered_code_is_unique() {
    let mut seen = std::collections::BTreeSet::new();
    for spec in registry::all() {
        assert!(
            seen.insert(spec.code),
            "duplicate registration: {}",
            spec.code
        );
    }
}

#[test]
fn a_codes_prefix_is_not_its_class() {
    // Recorded as a test because the inverse rule is the obvious guess and it is false.
    // ADR-0006's exemplars (`E-SOURCE-OVERRUN`, `R-VISUAL-GAP`, `N-QUANTIZATION`) all
    // happen to agree with their class, but ADR-0058 names `R-BOX-SLACK` for its
    // `R-<subject>-<symptom>` shape and then resolves its severity to `note`. A check
    // reading the class off the code would get that one backwards.
    let box_slack = registry::spec("R-BOX-SLACK").unwrap();
    assert_eq!(
        box_slack.default_class(),
        Class::Note,
        "ADR-0058 resolved it as `note`"
    );
}

#[test]
fn every_check_declares_at_least_one_class_it_may_emit() {
    for spec in registry::all() {
        assert!(!spec.classes.is_empty(), "{}: no class declared", spec.code);
    }
}

#[test]
fn a_check_may_take_its_severity_from_the_consequence_rather_than_from_itself() {
    // ADR-0006: "Severity is computed from the consequence at an instant. Not from the
    // check, and not from the track." Its own worked example is the visual gap: one is
    // `review` because nothing on any visual track covered it, and every other is a
    // note. A registry that pinned one class per code would make that unrepresentable.
    let gap = registry::spec("R-VISUAL-GAP").unwrap();
    assert!(gap.may_emit(Class::Review));
    assert!(gap.may_emit(Class::Note));
    assert!(!gap.may_emit(Class::Error));

    let review = montagent_core::finding::Finding::at_class("R-VISUAL-GAP", Class::Review);
    let note = montagent_core::finding::Finding::at_class("R-VISUAL-GAP", Class::Note);
    assert_eq!(review.code, note.code);
    assert_ne!(review.class, note.class);
}

#[test]
#[should_panic(expected = "not declared as able to emit")]
fn a_check_may_not_emit_a_class_it_never_declared() {
    montagent_core::finding::Finding::at_class("R-VISUAL-GAP", Class::Error);
}

#[test]
fn every_registered_check_names_the_adr_it_comes_from() {
    for spec in registry::all() {
        assert!(
            spec.adr.starts_with("ADR-"),
            "{}: provenance is {:?}",
            spec.code,
            spec.adr
        );
    }
}

#[test]
fn every_registered_check_has_a_template_the_renderer_can_use() {
    for spec in registry::all() {
        assert!(!spec.template.is_empty(), "{}: empty template", spec.code);
    }
}

#[test]
fn a_finding_may_not_carry_a_code_the_registry_does_not_know() {
    assert!(registry::spec("E-PARSE").is_some());
    assert!(registry::spec("E-NOT-A-REAL-CODE").is_none());
}

#[test]
fn a_findings_repair_class_comes_from_the_registry_not_from_the_call_site() {
    // ADR-0043: "Granularity is per check, not per instance. If any instance a check can
    // match is capable of being load-bearing, the check emits `repair: "none"` for
    // **every** instance it matches, including ones that look safe." Nothing a check
    // author writes at a call site can vary it.
    use montagent_core::finding::{Finding, Repair};

    let refused = Finding::new("E-RETIRED-KEY").at_file("p.json");
    assert_eq!(
        refused.repair,
        Some(Repair::None),
        "a refuse-class check carries `none` without asking for it"
    );

    let advised = Finding::new("E-KEYFRAME-EASE");
    assert_eq!(
        advised.repair, None,
        "an advise-class check supplies its value per instance"
    );

    let not_about_document = Finding::new("E-INVOCATION");
    assert_eq!(
        not_about_document.repair, None,
        "ADR-0073: `NotAboutDocument` never carries a repair, supplied or not"
    );

    let non_error = Finding::new("N-QUANTIZATION").at_file("p.json");
    assert_eq!(
        non_error.repair, None,
        "`repair` is an axis of `error` alone"
    );
}

#[test]
#[should_panic(expected = "may not state a repair value")]
fn a_refuse_class_check_may_not_talk_itself_into_a_repair() {
    // The per-instance triage ADR-0043 forbids: the gravity experiment's 6 safe
    // deletions and 2 load-bearing ones were separated by a fact not in the document.
    montagent_core::finding::Finding::new("E-RETIRED-KEY")
        .at_file("p.json")
        .repair_value(serde_json::json!({"value": "delete the key"}));
}

#[test]
#[should_panic(expected = "carries no repair")]
fn an_error_finding_with_no_repair_cannot_reach_a_report() {
    // ADR-0043: "Every `error`-class finding carries a `repair` field." An advise-class
    // check that forgets its value is the only way the field goes missing. `E-INVOCATION`
    // no longer exercises this: ADR-0073 declares it `NotAboutDocument`, exempt from the
    // requirement, so `E-KEYFRAME-EASE` (still plain advise-class) stands in.
    let mut report = montagent_core::report::Report::new("validate", Some("p.json".into()));
    report.push(montagent_core::finding::Finding::new("E-KEYFRAME-EASE").at_file("p.json"));
}

#[test]
#[should_panic(expected = "not declared `NotAboutDocument`")]
fn an_advise_class_code_cannot_reach_exit_3_carrying_its_repair() {
    // ADR-0080's "a second inhabitant cannot be added without making the same decision"
    // rests on this. `Report::push` alone does not carry it: its assert is satisfied by
    // *either* an exempt declaration or a repair value, so an advise-class code that
    // states its repair passes `push` and would arrive at exit 3 with the `repair` field
    // ADR-0073 forbids there. `Report::refused_invocation` checks the declaration itself.
    montagent_core::report::Report::refused_invocation(
        "create_project",
        Some("p.json".into()),
        montagent_core::finding::Finding::new("E-KEYFRAME-EASE")
            .at_file("p.json")
            .repair_value(serde_json::json!({"value": "linear"})),
    );
}

#[test]
fn a_not_about_document_finding_reaches_the_report_with_no_repair_at_all() {
    // ADR-0073 (#224): the process-level codes #188 introduced are not about a document,
    // so ADR-0043's binary does not apply to them, and `Report::push` must not demand a
    // `repair` field on their behalf.
    let mut report = montagent_core::report::Report::new("montagent", None);
    report.push(montagent_core::finding::Finding::new("E-INVOCATION").field(
        "reason",
        serde_json::Value::String("--nope is not a flag".into()),
    ));
    assert_eq!(report.findings[0].repair, None);

    let json = serde_json::to_value(&report.findings[0]).unwrap();
    assert!(
        json.get("repair").is_none(),
        "the field is absent, not `null` and not `\"none\"`: {json}"
    );
}

/// Every code whose `Finding::new(..)` is followed by a `.census(..)` before the next
/// `Finding::new`, read off the crate's own source. A source scan and not a list, so that a
/// check which starts carrying a census cannot do it without this test seeing it.
fn codes_that_attach_a_census() -> Vec<String> {
    fn walk(dir: &std::path::Path, out: &mut Vec<String>) {
        for entry in std::fs::read_dir(dir).expect("src/ reads") {
            let path = entry.expect("an entry").path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                let source = std::fs::read_to_string(&path).expect("a source file reads");
                for segment in source.split("Finding::new(\"").skip(1) {
                    let code = &segment[..segment.find('"').expect("a code literal")];
                    if segment.contains(".census(") && !out.iter().any(|c| c == code) {
                        out.push(code.to_string());
                    }
                }
            }
        }
    }
    let mut codes = Vec::new();
    walk(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut codes,
    );
    codes.sort();
    codes
}

#[test]
fn every_code_that_carries_a_census_declares_how_it_renders() {
    // ADR-0111: whether a census names its members in the text form is decided per code,
    // in the registry, by whoever authors the check. A census-bearing code with no mode is
    // one whose author skipped that decision.
    //
    // `E-FONT-NO-GLYPH` is the one exception, and declares **no** mode: all its groups hold
    // the same codepoints, so its grouping partitions nothing and is not a census (#427
    // ruling 6, and the glossary's **Census**).
    let emitting = codes_that_attach_a_census();
    assert!(
        emitting.iter().any(|c| c == "E-FONT-NO-GLYPH"),
        "the scan finds the exception, so it is reading the source: {emitting:?}"
    );
    for code in &emitting {
        let spec = registry::spec(code).expect("a registered code");
        match code.as_str() {
            "E-FONT-NO-GLYPH" => assert_eq!(spec.census, None, "{code} is not a census"),
            _ => assert!(spec.census.is_some(), "{code} carries a census and no mode"),
        }
    }

    // And the other way: a mode on a code that never carries a census is a declaration
    // about nothing.
    for spec in registry::all().iter().filter(|s| s.census.is_some()) {
        assert!(
            emitting.iter().any(|c| c == spec.code),
            "{} declares a census mode and never attaches a census",
            spec.code
        );
    }
}

#[test]
fn the_codes_whose_value_cannot_be_searched_for_are_named() {
    // #427 ruling 3's classification, pinned: a move between the two lists is an ADR-0111
    // amendment, not an edit.
    let mode = |code: &str| registry::spec(code).expect("registered").census;
    for code in [
        "N-TEXT-INVISIBLE",
        "N-TEXT-MIXED-NORMALIZATION",
        "E-TRACK-OVERLAP",
        "E-RETIRED-KEY",
    ] {
        assert_eq!(mode(code), Some(CensusMode::Named), "{code}");
    }
    for code in [
        "R-BOX-SLACK",
        "N-FONT-CENSUS",
        "R-FONT-SWAP",
        "D-BOUNDARY-CLUSTER-DRIFT",
    ] {
        assert_eq!(mode(code), Some(CensusMode::Counted), "{code}");
    }
}
