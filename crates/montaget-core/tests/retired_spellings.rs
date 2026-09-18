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
use montaget_core::report::{ExitCode, Report};
use montaget_core::{text, validate};

mod common;
use common::write_project;

/// A one-track project around `elements`, written the way the fixture writes them.
fn project_with(elements: &str) -> String {
    format!(
        r##"{{"frame":{{"width":1080,"height":1920}},"fps":25,"tracks":[{{"name":"photos","layer":1,"elements":[{elements}]}}]}}"##
    )
}

/// The retired-spelling findings on a one-element project.
///
/// This drives the check rather than the whole verb, which it did until #203 gave
/// `validate` its disk half. These elements carry a `source` that deliberately has no file
/// beside the scratch project, so running the verb would now add an `E-SOURCE-MISSING` to
/// every case and make "exactly one finding" a statement about two checks at once. Whether
/// a retired spelling fires is a question about the document alone, so it is asked of the
/// check alone — and `the_verb_still_runs_this_check` below keeps the wiring covered.
#[track_caller]
fn findings_on(elements: &str) -> Vec<Finding> {
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(&dir, "p.montaget.json", &project_with(elements));
    let document = montaget_core::parse::read(&path).expect("the scratch project parses");

    let mut report = Report::new("validate", Some(path.display().to_string()));
    montaget_core::checks::retired::check(&document, &mut report);
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
        // On the element, not in a run: ADR-0007 records that the one real project file
        // carries `"weight": "bold"` on all 22 of its text elements, so this is the
        // position the corpus actually used and the run-level one is the variant.
        (
            "weight",
            text_element("sentence-09", r##""weight":"bold""##),
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
        (
            "weight",
            text_element("sentence-12", r##""runs":[{"text":"hi","size":44}]"##),
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
fn weight_refuses_and_censuses_the_font_chain_each_element_points_at() {
    // Reversed from advise by a four-model jury, 4–0
    // (`docs/research/juries/retired-spelling-classes/`): the repair names a font file
    // that may not be vendored, so it is an instruction rather than a value, and an
    // advise-class repair that cannot be applied verbatim spends the machine-checkable
    // guarantee the field exists to give. Recorded in #228, because what is missing here
    // is an asset rather than an intent and fits neither arm of ADR-0043 cleanly.
    let elements = format!(
        "{},{},{}",
        text_element("sentence-05", r##""weight":"bold""##),
        text_element("sentence-06", r##""weight":"bold""##),
        // Same spelling, a different chain — which is the whole point of the census: the
        // repair for this one is not the repair for the other two.
        r##"{"id":"quiz-05","type":"text","start":0,"end":1000,"width":984,"height":169,"font":"brand-regular","size":55,"weight":"bold","runs":[{"text":"hi"}]}"##,
    );
    let findings = findings_on(&elements);

    assert_eq!(findings.len(), 3);
    for finding in &findings {
        assert_eq!(finding.code, "E-RETIRED-KEY");
        assert_eq!(finding.repair, Some(Repair::None));
    }
    let census = findings[0].census.as_ref().expect("a sibling census");
    assert_eq!(census.field, "font");
    assert_eq!(census.groups.len(), 2);
    assert_eq!(census.groups[0].members, vec!["sentence-05", "sentence-06"]);
    assert_eq!(census.groups[1].members, vec!["quiz-05"]);
}

#[test]
fn the_weight_spelling_is_caught_on_the_element_and_inside_a_run() {
    for element in [
        text_element("sentence-05", r##""weight":"bold""##),
        text_element("sentence-05", r##""runs":[{"text":"hi","weight":"bold"}]"##),
        text_element("sentence-05", r##""runs":[{"text":"hi","bold":true}]"##),
    ] {
        let findings = findings_on(&element);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert_eq!(findings[0].location.element, Some("sentence-05".into()));
    }
}

#[test]
fn box_and_align_on_a_non_text_element_refuse_until_an_adr_rules() {
    // **Recorded, not decided** (#192). Neither has been classified by any ADR, and both
    // carry `gravity`'s fork. The argument, and what would change it, is stated once —
    // in `checks::retired`'s module doc and in `CONTEXT.md` under **Box** — rather than
    // re-argued here. This test pins the behaviour that follows from it, so the day an
    // ADR rules, the thing that fails is a named expectation and not a surprise.
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

#[test]
fn the_bare_mask_key_is_retired_whatever_it_carries() {
    // ADR-0068: *"a bare `mask` key on any element"* — the value's shape does not qualify
    // the retirement, and a repair that transposes the key carries whatever was written
    // into the `shape` slot. What that value then is, is the schema's question.
    for value in [r##""circle""##, r##"{"shape":"circle"}"##, "true"] {
        let findings = findings_on(&image("handle-logo", &format!(r##""mask":{value}"##)));
        assert_eq!(findings.len(), 1, "`mask: {value}` should fire: {findings:#?}");
        assert_eq!(findings[0].fields["key"], "mask");
    }
}

#[test]
fn an_element_whose_type_cannot_be_read_is_not_a_non_text_element() {
    // `align` is retired *on a non-text element*. An element whose `type` is absent or
    // misspelled is neither — it is an element whose type is the finding, which is a
    // schema question and another ticket's. Answering it here would answer it with a
    // non-bypassable refusal.
    let untyped = r##"{"id":"mystery","start":0,"end":1000,"align":"center","width":10,"height":10}"##;
    assert!(findings_on(untyped).is_empty());

    let misspelled = r##"{"id":"mystery","type":"imag","start":0,"end":1000,"align":"top","width":10,"height":10}"##;
    assert!(findings_on(misspelled).is_empty());

    // `gravity`, by contrast, is retired on *every* element type (ADR-0015), so it still
    // fires — and names both halves of the replacement rather than a confident half.
    let findings = findings_on(
        r##"{"id":"mystery","type":"imag","start":0,"end":1000,"gravity":"top","width":10,"height":10}"##,
    );
    assert_eq!(findings.len(), 1);
    let named = findings[0].fields["replacement"].as_str().unwrap();
    assert!(named.contains("clip") && named.contains("origin"), "{named}");
}

#[test]
fn the_css_habit_spelling_of_an_opaque_colour_is_caught_in_either_case() {
    // `#fbf3e3ff` is two retired spellings at once — hex digits are uppercase (ADR-0014),
    // and the opaque alpha is a second spelling of six digits — and it is the one a CSS
    // habit actually types. `validate` reads the permissive spine, never `Colour`'s
    // deserializer, so nothing else in the run would have caught it.
    for written in ["#FBF3E3FF", "#fbf3e3ff", "#FBF3E3ff"] {
        let findings = findings_on(&text_element(
            "sentence-05",
            &format!(r##""color":"{written}""##),
        ));
        assert_eq!(findings.len(), 1, "{written} should fire: {findings:#?}");
        assert_eq!(
            findings[0].fields["replacement"], "`#FBF3E3`",
            "the six-digit form the schema will accept, in the case it will accept"
        );
    }
}

#[test]
fn box_censuses_the_pivot_its_repair_would_need() {
    // A file still carrying `box` predates `clip` — ADR-0012 retired one and introduced
    // the other in the same breath — so an aperture census would group every affected
    // element under "absent". The pivot is the fact the repair needs and the 4-array left
    // implicit, so the census reports which of them state one.
    let elements = r##"{"id":"card-05","type":"rect","start":0,"end":1,"box":[0,0,10,10],"width":10,"height":10,"fill":"#000000"},{"id":"card-06","type":"rect","start":0,"end":1,"box":[0,20,10,10],"origin":"top-left","width":10,"height":10,"fill":"#000000"}"##;
    let findings = findings_on(elements);
    let census = findings[0].census.as_ref().expect("a sibling census");

    assert_eq!(census.field, "origin");
    assert_eq!(census.groups.len(), 2);
    assert_eq!(census.groups[0].members, vec!["card-05"]);
    assert_eq!(census.groups[1].members, vec!["card-06"]);
}

fn render(findings: &[Finding]) -> String {
    let mut report = montaget_core::report::Report::new("validate", Some("p.json".into()));
    for finding in findings {
        report.push(finding.clone());
    }
    text::render(&report.to_json(), text::Options::verbose()).unwrap()
}

#[test]
fn the_verb_still_runs_this_check() {
    // `findings_on` drives the check directly, which is the right unit for "does this
    // spelling fire". The wiring is a separate claim, and it is the one that would break
    // silently: a check nobody calls passes every test it has. So one case goes the whole
    // way through `validate`, on a project whose media is real — the fixture's own — so
    // that the disk half has nothing to say and the retired finding stands alone.
    if !common::has_ffprobe() {
        return;
    }
    let dir = common::tempdir(std::panic::Location::caller().line());
    std::fs::create_dir_all(dir.join("images")).unwrap();
    std::fs::copy(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/en-halloween-decorating/images/05.png"),
        dir.join("images/05.png"),
    )
    .unwrap();
    let path = write_project(
        &dir,
        "p.montaget.json",
        &project_with(&image("photo-06", r##""gravity":"bottom""##)),
    );

    let report = validate(&path);

    assert_eq!(
        report
            .findings
            .iter()
            .map(|f| f.code.as_str())
            .collect::<Vec<_>>(),
        ["E-RETIRED-KEY"],
        "the verb reaches the check, and the real image beside it reports nothing"
    );
    assert_eq!(report.exit_code(), ExitCode::Errors);
}
