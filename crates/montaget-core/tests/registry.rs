//! The check registry: the thing every check ticket needs and none should own.

use montaget_core::finding::Class;
use montaget_core::registry::{self, ThresholdProvenance};

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

    let review = montaget_core::finding::Finding::at_class("R-VISUAL-GAP", Class::Review);
    let note = montaget_core::finding::Finding::at_class("R-VISUAL-GAP", Class::Note);
    assert_eq!(review.code, note.code);
    assert_ne!(review.class, note.class);
}

#[test]
#[should_panic(expected = "not declared as able to emit")]
fn a_check_may_not_emit_a_class_it_never_declared() {
    montaget_core::finding::Finding::at_class("R-VISUAL-GAP", Class::Error);
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
