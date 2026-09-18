//! The check registry: the thing every check ticket needs and none should own.

use montaget_core::finding::Class;
use montaget_core::registry::{self, ThresholdProvenance};

#[test]
fn every_registered_error_class_code_has_a_declared_refuse_class() {
    // ADR-0043: the class is decided once, by whoever authors the check, at the moment
    // the check is written. A registered `error` with no declared class is a check whose
    // author skipped that decision.
    let missing: Vec<_> = registry::all()
        .iter()
        .filter(|spec| spec.class == Class::Error && spec.repair.is_none())
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
        .filter(|spec| spec.class != Class::Error && spec.repair.is_some())
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
            assert_ne!(
                spec.class,
                Class::Error,
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
        box_slack.class,
        Class::Note,
        "ADR-0058 resolved it as `note`"
    );
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
