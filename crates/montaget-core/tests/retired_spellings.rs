//! Retired spellings: the migration mechanism, as a pair of checks.
//!
//! ADR-0016 settled that there is no version number and no `montaget migrate` — *"the
//! unknown-key error is the migration mechanism"* — and that a retired spelling's error
//! **names its replacement**. ADR-0043 settled what the finding may then say about the
//! fix: nothing at all where the repair depends on intent the document does not carry.
//!
//! Every spelling below gets a **fire** case and a **must-not-fire** case, because a
//! check that fires is only half the evidence: the other half is that the modern spelling
//! it names is silent. The committed fixture is the third half — `tests/fixture.rs`
//! asserts it reports clean, and it carries `handle-logo`'s migrated
//! `effects: [{"name": "mask", "shape": "circle"}]`.

use montaget_core::finding::{Class, Finding, Repair};
use montaget_core::report::ExitCode;
use montaget_core::{text, validate};

mod common;
use common::write_project;

/// A one-track project around `elements`, written the way the fixture writes them.
fn project_with(elements: &str) -> String {
    format!(
        r##"{{"frame":{{"width":1080,"height":1920}},"fps":25,"tracks":[{{"name":"photos","layer":1,"elements":[{elements}]}}]}}"##
    )
}

#[track_caller]
fn findings_on(elements: &str) -> Vec<Finding> {
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(&dir, "p.montaget.json", &project_with(elements));
    let report = validate(&path);
    report.findings
}

/// An image element carrying `extra`, positioned and clipped the way the fixture's photos
/// are, so that every fire case differs from its must-not-fire twin in one key alone.
fn image(id: &str, extra: &str) -> String {
    let comma = if extra.is_empty() { "" } else { "," };
    format!(
        r##"{{"id":"{id}","type":"image","start":0,"end":1000,"source":"images/05.png","x":0,"y":0,"origin":"top-left","width":1080,"height":1912,"fit":"cover","clip":[0,0,1080,1300]{comma}{extra}}}"##
    )
}

/// A text element carrying `extra`.
fn text_element(id: &str, extra: &str) -> String {
    let comma = if extra.is_empty() { "" } else { "," };
    format!(
        r##"{{"id":"{id}","type":"text","start":0,"end":1000,"x":48,"y":1324,"origin":"top-left","width":984,"height":169,"font":"brand","size":55,"runs":[{{"text":"hi"}}]{comma}{extra}}}"##
    )
}

/// Every retired spelling in the set ADR-0016's mechanism has to cover, as the one key or
/// value that separates a file written against an older vocabulary from a current one.
fn every_retired_spelling() -> Vec<(&'static str, String, &'static str)> {
    vec![
        ("gravity", image("photo-06", r##""gravity":"bottom""##), "clip"),
        (
            "box",
            image("photo-07", r##""box":[0,0,1080,1912]"##),
            "width",
        ),
        ("align", image("photo-08", r##""align":"top""##), "origin"),
        (
            "mask",
            image("handle-logo", r##""mask":"circle""##),
            "effects",
        ),
        (
            "anchor",
            text_element("sentence-05", r##""anchor":"center-left""##),
            "origin",
        ),
        (
            "origin",
            text_element("sentence-06", r##""origin":"center-center""##),
            "center",
        ),
        (
            "color",
            text_element("sentence-07", r##""color":"#FBF3E3FF""##),
            "#FBF3E3",
        ),
        (
            "bold",
            text_element("sentence-08", r##""runs":[{"text":"hi","bold":true}]"##),
            "fonts",
        ),
        (
            "weight",
            text_element("sentence-09", r##""runs":[{"text":"hi","weight":700}]"##),
            "fonts",
        ),
        ("fit", image("photo-09", r##""fit":"none""##), "literal"),
        ("fit", image("photo-10", r##""fit":"fill""##), "literal"),
    ]
}

/// The modern spelling of each, one per fire case above.
fn every_current_spelling() -> Vec<(&'static str, String)> {
    vec![
        ("gravity", image("photo-06", "")),
        ("box", image("photo-07", "")),
        // `align` is not retired — it is retired *on a non-text element*. Its own type is
        // the must-not-fire case, and the sharpest one in the set.
        ("align", text_element("sentence-05", r##""align":"center""##)),
        (
            "mask",
            image(
                "handle-logo",
                r##""effects":[{"name":"mask","shape":"circle"}]"##,
            ),
        ),
        // ADR-0019's ordinary feature: the schema catches the rediscovery on *shape*, not
        // presence, so an anchor carrying a below/above object is not a retired spelling.
        (
            "anchor",
            text_element("sentence-06", r##""anchor":{"below":"title"}"##),
        ),
        ("origin", text_element("sentence-07", r##""origin":"center""##)),
        // `center-left` contains `center` and is a legal keyword; only the doubled middle
        // is retired.
        (
            "origin",
            text_element("sentence-08", r##""origin":"center-left""##),
        ),
        ("color", text_element("sentence-09", r##""color":"#FBF3E3""##)),
        // Eight digits are legal; it is the *opaque* eight-digit form that is a second
        // spelling of a six-digit value.
        (
            "color",
            text_element("sentence-10", r##""color":"#FBF3E380""##),
        ),
        (
            "bold",
            text_element("sentence-11", r##""runs":[{"text":"hi","font":"brand-bold"}]"##),
        ),
        ("fit", image("photo-09", "")),
    ]
}

#[test]
fn every_retired_spelling_is_an_error_naming_its_replacement() {
    for (key, element, replacement) in every_retired_spelling() {
        let findings = findings_on(&element);

        assert_eq!(
            findings.len(),
            1,
            "`{key}` should produce exactly one finding, got {findings:#?}"
        );
        let finding = &findings[0];
        assert_eq!(finding.class, Class::Error, "`{key}`: {finding:#?}");
        assert_eq!(
            finding.fields["key"], key,
            "the finding names the retired spelling: {finding:#?}"
        );

        let named = finding.fields["replacement"].as_str().unwrap_or_default();
        assert!(
            named.contains(replacement),
            "`{key}` must name `{replacement}` as its replacement, got {named:?}"
        );
    }
}

#[test]
fn the_current_spelling_of_each_is_silent() {
    for (key, element) in every_current_spelling() {
        let findings = findings_on(&element);
        assert!(
            findings.is_empty(),
            "the current spelling of `{key}` must not fire: {findings:#?}"
        );
    }
}

#[test]
fn a_retired_spelling_refuses_the_render() {
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(
        &dir,
        "p.montaget.json",
        &project_with(&image("photo-06", r##""gravity":"bottom""##)),
    );

    assert_eq!(validate(&path).exit_code(), ExitCode::Errors);
}

#[test]
fn gravity_is_refuse_class_and_carries_a_geometry_grouped_census() {
    // ADR-0043's founding instance: 6 of 8 `gravity` deletions were geometric no-ops and
    // 2 silently changed the picture, and the fact separating them is not in the
    // document. So every instance refuses, including the ones that look safe, and the
    // census groups the affected elements by an observable geometry fact instead.
    let elements = format!(
        "{},{},{}",
        image("photo-05", r##""gravity":"top""##),
        image("photo-06", r##""gravity":"bottom""##),
        // A third, sharing `photo-05`'s aperture, so the census has a group of two.
        image("photo-07", r##""gravity":"top""##),
    );
    let findings = findings_on(&elements);

    assert_eq!(findings.len(), 3, "one per instance: {findings:#?}");
    for finding in &findings {
        assert_eq!(finding.code, "E-RETIRED-KEY");
        assert_eq!(
            finding.repair,
            Some(Repair::None),
            "refuse-class, uniformly: {finding:#?}"
        );
        let census = finding.census.as_ref().expect("a sibling census");
        assert_eq!(census.field, "clip");
        assert_eq!(
            census.groups.len(),
            1,
            "all three share one aperture, so the census is one group: {census:#?}"
        );
        assert_eq!(census.groups[0].members.len(), 3);
    }
}

#[test]
fn a_census_groups_the_affected_elements_and_ranks_nothing() {
    // The 6-vs-2 shape of the gravity experiment, with the minority declared first: the
    // census must not be worded, or ordered, so that the larger group reads as correct.
    // `photo-06`'s own geometry from the gravity experiment: the rect translated to the
    // aperture's opposite edge, which is what a correct `gravity:"bottom"` repair does and
    // what deleting the key unmoved does not.
    let minority = r##"{"id":"photo-06","type":"image","start":0,"end":1000,"source":"images/06.png","x":0,"y":-612,"origin":"top-left","width":1080,"height":1912,"fit":"cover","clip":[0,620,1080,1300],"gravity":"bottom"}"##;
    let elements = format!(
        "{},{},{}",
        minority,
        image("photo-05", r##""gravity":"top""##),
        image("photo-07", r##""gravity":"top""##),
    );

    let findings = findings_on(&elements);
    let census = findings[0].census.as_ref().expect("a sibling census");

    assert_eq!(census.groups.len(), 2);
    assert_eq!(
        census.groups[0].members,
        vec!["photo-06"],
        "declaration order survives; nothing sorts by size"
    );
    assert_eq!(census.groups[1].members, vec!["photo-05", "photo-07"]);
}

#[test]
fn the_bare_mask_key_is_advise_class_and_repairs_to_an_effects_member() {
    // ADR-0068: it never had accepted semantics in any document, so no prior meaning
    // exists for a repair to misread, and with the param-less form defined the repair is
    // a pure spelling transposition of a value already in the target enum.
    let findings = findings_on(&image("handle-logo", r##""mask":"circle""##));
    let finding = &findings[0];

    assert_eq!(
        finding.repair,
        Some(Repair::Advise(serde_json::json!({
            "value": {"effects": [{"name": "mask", "shape": "circle"}]}
        }))),
    );

    let rendered = render(&findings);
    assert!(rendered.contains("advise-class"), "{rendered}");
    assert!(rendered.contains("effects"), "{rendered}");
}

#[test]
fn box_and_align_on_a_non_text_element_refuse_until_an_adr_rules() {
    // **Recorded, not decided** (#192). Neither has been classified by any ADR:
    //
    // - `box: [x,y,w,h]` retires into `x`, `y`, `origin`, `width`, `height` — and the
    //   pivot the 4-array left implicit is exactly the fact the repair needs and the
    //   document does not carry. `box: "card-05"` retires into literal `width`/`height`,
    //   whose values live on an element that 15 of the fixture's 22 text elements do not
    //   have.
    // - `align` on an image meant *which part of the source survives the crop*
    //   (ADR-0012) — `gravity`'s question in `align`'s spelling, with `gravity`'s fork.
    //
    // ADR-0068 remarks in passing that *"`box` was migrated by arithmetic script"*, which
    // points the other way, and ADR-0043 is explicit that it *"classifies `gravity` and
    // nothing else"*. Until an ADR rules, these sit under ADR-0043's own standing rule —
    // if any instance a check can match could be load-bearing, the check refuses — which
    // is the conservative half of the fork and the one whose cost the ADR has already
    // accepted. The ruling belongs to an ADR, not to this check.
    for element in [
        image("photo-07", r##""box":[0,0,1080,1912]"##),
        image("photo-08", r##""align":"top""##),
    ] {
        let findings = findings_on(&element);
        assert_eq!(findings[0].repair, Some(Repair::None), "{findings:#?}");
        assert!(findings[0].census.is_some(), "{findings:#?}");
    }
}

#[test]
fn no_check_emits_both_repair_shapes_across_its_instances() {
    // ADR-0043: "Granularity is per check, not per instance." The two codes exist because
    // the uniformity rule and the two-class split cannot both hold inside one code —
    // `gravity` refuses and the bare `mask` key advises, so they are different checks.
    let elements = every_retired_spelling()
        .iter()
        .map(|(_, element, _)| element.clone())
        .collect::<Vec<_>>()
        .join(",");

    let findings = findings_on(&elements);
    assert!(findings.len() >= every_retired_spelling().len());

    let mut shape_of: std::collections::BTreeMap<String, bool> = Default::default();
    for finding in &findings {
        let refuses = finding.repair == Some(Repair::None);
        match shape_of.get(&finding.code) {
            Some(seen) => assert_eq!(
                *seen, refuses,
                "{} emits both repair shapes across its instances",
                finding.code
            ),
            None => {
                shape_of.insert(finding.code.clone(), refuses);
            }
        }
    }
    assert_eq!(shape_of.len(), 2, "one refuse-class code, one advise-class");
}

#[test]
fn every_error_class_finding_carries_a_repair() {
    // ADR-0043, over every instance this check can produce rather than over one of them.
    let elements = every_retired_spelling()
        .iter()
        .map(|(_, element, _)| element.clone())
        .collect::<Vec<_>>()
        .join(",");

    for finding in findings_on(&elements) {
        assert!(
            finding.repair.is_some(),
            "{} carries no repair: {finding:#?}",
            finding.code
        );
    }
}

#[test]
fn a_refuse_class_retirement_tells_the_agent_to_stop_rather_than_repair() {
    let findings = findings_on(&image("photo-06", r##""gravity":"bottom""##));
    let rendered = render(&findings);
    let flowed = rendered.split_whitespace().collect::<Vec<_>>().join(" ");

    assert!(flowed.contains("refuse-class"), "{rendered}");
    assert!(
        flowed.contains("Surface this finding verbatim to whoever is operating Montaget"),
        "{rendered}"
    );
    assert!(
        flowed.contains("do not repair it by ordinary file edit"),
        "{rendered}"
    );
    // ADR-0016: the retired case names the replacement, refuse-class or not. Refusing to
    // state a *repair* is not refusing to state what the format says now.
    assert!(flowed.contains("clip"), "{rendered}");
}

#[test]
fn a_retired_value_is_named_with_the_value_the_file_carries() {
    // "One code, one field set, one template" — and the template's substance is the
    // element, the key and the value, so a reader never re-reads the project to learn
    // what the finding is about (ADR-0006).
    let findings = findings_on(&text_element("sentence-06", r##""origin":"center-center""##));
    let rendered = render(&findings);

    assert!(rendered.contains("sentence-06"), "{rendered}");
    assert!(rendered.contains("center-center"), "{rendered}");
    assert!(rendered.contains("center"), "{rendered}");
}

#[test]
fn the_project_background_is_checked_too() {
    // A retired colour spelling is a retired spelling wherever a colour is written, and
    // the project's own background is the one colour that is not on an element.
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(
        &dir,
        "p.montaget.json",
        r##"{"frame":{"width":1080,"height":1920},"fps":25,"background":"#FBF3E3FF","tracks":[]}"##,
    );

    let findings = validate(&path).findings;
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].fields["key"], "background");
    assert_eq!(findings[0].location.element, None);
    assert!(
        render(&findings).contains("#FBF3E3"),
        "{}",
        render(&findings)
    );
}

fn render(findings: &[Finding]) -> String {
    let mut report = montaget_core::report::Report::new("validate", Some("p.json".into()));
    for finding in findings {
        report.push(finding.clone());
    }
    text::render(&report.to_json(), text::Options::verbose()).unwrap()
}
