//! ADR-0007's three text-byte checks (#206): the grapheme-cluster check, the
//! invisible-character census and the mixed-normalization finding.
//!
//! None of them opens a font or touches the disk, so — like `tests/box_slack.rs` — the
//! check is called directly and the signature is the claim. One test at the bottom goes
//! the whole way through `validate`, because "the check is registered and actually runs"
//! is a different fact from "the check is correct".
//!
//! Every character these tests turn on is written as a Rust escape rather than pasted, so
//! that a reader can tell a ZWJ from a space without trusting their editor.

use montagent_core::finding::{Class, Finding, Repair};
use montagent_core::report::Report;
use montagent_core::{parse, validate_with_cache};

mod common;
use common::{canonical, write_project};

/// One text element whose `runs` array the caller composes.
fn text(id: &str, runs: &str) -> String {
    format!(
        r##"{{"id":"{id}","type":"text","start":0,"end":1000,"x":0,"y":0,"width":900,"height":100,"font":"brand","size":40,"runs":[{runs}]}}"##
    )
}

fn run(text: &str) -> String {
    serde_json::json!({"text": text}).to_string()
}

fn project(elements: &[String]) -> String {
    canonical(&format!(
        r##"{{"frame":{{"width":1080,"height":1920}},"fps":25,"tracks":[{{"name":"captions","layer":0,"elements":[{}]}}]}}"##,
        elements.join(",")
    ))
}

#[track_caller]
fn report_on(elements: &[String]) -> Report {
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(&dir, "p.montagent.json", &project(elements));
    let document = parse::read(&path).expect("the project parses");
    let mut report = Report::new("validate", Some(document.path().to_string()));
    montagent_core::checks::runs::check(&document, &mut report);
    report
}

#[track_caller]
fn of_code<'a>(report: &'a Report, code: &str) -> Vec<&'a Finding> {
    report.findings.iter().filter(|f| f.code == code).collect()
}

fn members(finding: &Finding, group: usize) -> &[String] {
    &finding.census.as_ref().expect("a census").groups[group].members
}

// ---- `E-RUN-SPLIT-CLUSTER` -------------------------------------------------------------

#[test]
fn a_run_boundary_between_a_base_and_its_combining_mark_is_an_error() {
    // ADR-0007: "a **grapheme-cluster** check that no run boundary splits a base from its
    // combining mark". `e` ends run 1 and U+0301 opens run 2, so the mark is shaped in a
    // run that holds no base for it.
    let report = report_on(&[text(
        "split",
        &format!("{},{}", run("caf\u{0065}"), run("\u{0301} noir")),
    )]);

    let findings = of_code(&report, "E-RUN-SPLIT-CLUSTER");
    assert_eq!(findings.len(), 1, "{:?}", report.findings);
    let finding = findings[0];
    assert_eq!(finding.class, Class::Error);
    assert_eq!(finding.location.element.as_deref(), Some("split"));
    assert_eq!(finding.location.track.as_deref(), Some("captions"));
    assert_eq!(finding.fields["run"], 2, "one-based, as the array reads");
    assert_eq!(finding.fields["codepoints"], "U+0065 U+0301");
    assert_eq!(
        finding.repair,
        Some(Repair::None),
        "which of the two styles the cluster should wear is not in the document"
    );
}

#[test]
fn a_run_boundary_on_a_cluster_boundary_never_fires() {
    // The must-not-fire half: the identical text, split one byte later, so every cluster
    // is whole inside one run. A check that fired here would make the ordinary
    // reference-class edit — emphasise one word — unexpressible.
    let report = report_on(&[text(
        "whole",
        &format!("{},{}", run("caf\u{0065}\u{0301}"), run(" noir")),
    )]);
    assert_eq!(of_code(&report, "E-RUN-SPLIT-CLUSTER").len(), 0);
}

#[test]
fn an_emoji_sequence_split_at_its_joiner_is_the_same_error() {
    // The cluster the ADR does not name and the one an agent is likeliest to cut: a family
    // emoji is one cluster of seven code points, and a run boundary anywhere inside it
    // leaves two people standing where a family was.
    let report = report_on(&[text(
        "family",
        &format!(
            "{},{}",
            run("\u{1F468}\u{200D}"),
            run("\u{1F469}\u{200D}\u{1F467}")
        ),
    )]);
    let findings = of_code(&report, "E-RUN-SPLIT-CLUSTER");
    assert_eq!(findings.len(), 1, "{:?}", report.findings);
    assert!(
        findings[0].fields["codepoints"]
            .as_str()
            .unwrap()
            .starts_with("U+1F468 U+200D U+1F469"),
        "{:?}",
        findings[0].fields
    );
}

#[test]
fn a_single_run_element_and_an_element_with_no_runs_are_both_silent() {
    // There is no boundary to be inside of. Stated because the walk below adds each run's
    // length and the last boundary is the end of the string — an off-by-one there would
    // fire on every one-run element in the fixture.
    let report = report_on(&[text("one", &run("caf\u{0065}\u{0301}")), text("none", "")]);
    assert_eq!(of_code(&report, "E-RUN-SPLIT-CLUSTER").len(), 0);
}

// ---- `N-TEXT-INVISIBLE` -----------------------------------------------------------------

#[test]
fn the_invisible_character_census_counts_and_locates_every_one() {
    // ADR-0007's set: ZWJ/ZWNJ, RLM/LRM and the variation selectors. One finding for the
    // project, because the distribution is the answer.
    let report = report_on(&[
        text("emoji", &run("a \u{1F468}\u{200D}\u{1F469} b")),
        text("bidi", &run("\u{200F}\u{0631} \u{200D}")),
        text("plain", &run("nothing to see")),
    ]);

    let findings = of_code(&report, "N-TEXT-INVISIBLE");
    assert_eq!(findings.len(), 1, "one census, not one per element");
    let finding = findings[0];
    assert_eq!(finding.class, Class::Note);
    assert_eq!(
        finding.fields["occurrences"],
        "3 characters in the project's text occupy no space"
    );
    assert_eq!(
        finding.fields["summary"],
        "U+200D ZERO WIDTH JOINER ×2, U+200F RIGHT-TO-LEFT MARK ×1"
    );
    assert_eq!(finding.census.as_ref().unwrap().field, "character");
    assert_eq!(members(finding, 0), ["emoji", "bidi"]);
    assert_eq!(members(finding, 1), ["bidi"]);
    assert_eq!(
        finding.repair, None,
        "`repair` is an axis of `error` alone (ADR-0043)"
    );
}

#[test]
fn a_project_whose_text_is_all_visible_produces_no_census() {
    let report = report_on(&[text("plain", &run("cobweb  -  cobweb"))]);
    assert_eq!(of_code(&report, "N-TEXT-INVISIBLE").len(), 0);
}

#[test]
fn a_variation_selector_is_named_as_one_without_a_table_of_every_unicode_name() {
    let report = report_on(&[text("emoji", &run("\u{2764}\u{FE0F}"))]);
    let finding = of_code(&report, "N-TEXT-INVISIBLE")[0];
    assert_eq!(finding.fields["summary"], "U+FE0F VARIATION SELECTOR ×1");
}

// ---- `N-TEXT-MIXED-NORMALIZATION` -------------------------------------------------------

#[test]
fn two_canonically_equivalent_spellings_of_one_caption_are_named() {
    // Spec #168's story 105: "two strings that look identical and compare unequal". The
    // two elements below render the same word and no exact-string replace finds both.
    let report = report_on(&[
        text("composed", &run("caf\u{00E9}")),
        text("decomposed", &run("cafe\u{0301}")),
    ]);

    let findings = of_code(&report, "N-TEXT-MIXED-NORMALIZATION");
    assert_eq!(findings.len(), 1, "{:?}", report.findings);
    let finding = findings[0];
    assert_eq!(finding.class, Class::Note);
    assert_eq!(finding.fields["spellings"], "2 spellings");
    assert_eq!(finding.fields["text"], "`caf\u{00E9}`");

    let census = finding.census.as_ref().expect("a census");
    assert_eq!(census.field, "normalization");
    assert_eq!(census.groups[0].value, "NFC");
    assert_eq!(members(finding, 0), ["composed"]);
    assert_eq!(census.groups[1].value, "NFD");
    assert_eq!(members(finding, 1), ["decomposed"]);
}

#[test]
fn mixed_normalization_names_an_element_in_each_form_under_verbose() {
    // ADR-0111: this census is Named, because NFC and NFD bytes look identical and no
    // search the reader can type finds one form and not the other. So the expanded note
    // names the members of each group, and "1 at NFC" is no longer the whole of it.
    let report = report_on(&[
        text("composed", &run("caf\u{00E9}")),
        text("decomposed", &run("cafe\u{0301}")),
    ]);
    let rendered =
        montagent_core::text::render(&report.to_json(), montagent_core::text::Options::verbose())
            .expect("the report renders");
    assert!(
        rendered.contains("census normalization: 1 at NFC (composed), 1 at NFD (decomposed)\n"),
        "{rendered}"
    );
}

#[test]
fn one_spelling_used_twice_is_not_mixed_normalization() {
    // The must-not-fire half, in the two shapes that matter: the same bytes twice, and a
    // decomposed string with no composed twin. Neither is two spellings of anything —
    // nothing compares unequal — and naming the second would name a defect whose only
    // repair is the tidying pass ADR-0007 forbids.
    let report = report_on(&[
        text("a", &run("caf\u{00E9}")),
        text("b", &run("caf\u{00E9}")),
        text("lone", &run("na\u{0069}\u{0308}ve")),
    ]);
    assert_eq!(of_code(&report, "N-TEXT-MIXED-NORMALIZATION").len(), 0);
}

// ---- registered and running ------------------------------------------------------------

#[test]
fn the_three_checks_reach_a_real_validate_run() {
    // The check being correct and the check being wired into `validate` are two facts, and
    // every other test here establishes only the first. No `source` anywhere in the
    // document, so this needs no `ffprobe`.
    let dir = common::tempdir(line!());
    let path = write_project(
        &dir,
        "p.montagent.json",
        &project(&[
            text(
                "split",
                &format!("{},{}", run("caf\u{0065}"), run("\u{0301}\u{200D}")),
            ),
            text("composed", &run("caf\u{00E9}")),
            text("decomposed", &run("cafe\u{0301}")),
        ]),
    );

    // Through a sidecar this test owns, not the machine's default: a test that recorded a
    // scratch project into the developer's own cache would leave it there.
    let report = validate_with_cache(&path, &dir.join("probe-cache.json"));
    let mut codes: Vec<&str> = report.findings.iter().map(|f| f.code.as_str()).collect();
    codes.sort_unstable();
    codes.dedup();
    assert!(codes.contains(&"E-RUN-SPLIT-CLUSTER"), "{codes:?}");
    assert!(codes.contains(&"N-TEXT-INVISIBLE"), "{codes:?}");
    assert!(codes.contains(&"N-TEXT-MIXED-NORMALIZATION"), "{codes:?}");
}

#[test]
fn each_of_the_three_renders_as_prose_from_its_own_field_set() {
    // "One code, one field set, one template" (ADR-0006): the prose renderer has never seen
    // a `Report`, so a template naming a field its check does not supply is a `RenderError`
    // at the moment a user asks for text — which no assertion about the JSON would catch.
    let report = report_on(&[
        text(
            "split",
            &format!("{},{}", run("caf\u{0065}"), run("\u{0301}\u{200D}")),
        ),
        text("composed", &run("caf\u{00E9}")),
        text("decomposed", &run("cafe\u{0301}")),
    ]);

    let prose =
        montagent_core::text::render(&report.to_json(), montagent_core::text::Options::verbose())
            .expect("every finding renders");
    assert!(
        prose.contains("the boundary before run 2 falls inside"),
        "{prose}"
    );
    assert!(
        prose.contains("1 character in the project's text occupies no space"),
        "{prose}"
    );
    assert!(
        prose.contains("2 spellings of `caf\u{00E9}` are the same string under Unicode"),
        "{prose}"
    );
}
