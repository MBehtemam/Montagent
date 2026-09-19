//! The closed schema, as findings (#244) — the check that turns `Loose::strict` into a
//! report.
//!
//! `tests/closed_schema.rs` asserts what the *types* refuse. This file asserts that
//! `validate` says so: ADR-0016 makes the unknown-key error the whole migration mechanism
//! and *"an optional signal is indistinguishable from no signal"*, so a schema fault that
//! parses into `serde` and never reaches a report is the same as no schema at all.
//!
//! Every project here is built from `rect` elements, which carry no `source` — so the disk
//! half of `validate` has nothing to ask and these tests run without an `ffprobe`.

use montaget_core::finding::{Class, Finding, Repair};
use montaget_core::permissive::Loose;
use montaget_core::report::{ExitCode, Report};
use montaget_core::{text, validate};

mod common;
use common::{canonical, write_project};

/// A project of whatever tracks a test needs, in the canonical convention — so that
/// `L-LAYOUT` is not a finding every assertion here has to filter around.
fn project(tracks: &str) -> String {
    canonical(&format!(
        r##"{{"frame":{{"width":1080,"height":1920}},"fps":25,"tracks":[{tracks}]}}"##
    ))
}

fn track(name: &str, layer: i64, elements: &str) -> String {
    format!(r##"{{"name":"{name}","layer":{layer},"elements":[{elements}]}}"##)
}

/// A legal rect, plus whatever a test splices in after `end`.
fn rect(id: &str, extra: &str) -> String {
    format!(
        r##"{{"id":"{id}","type":"rect","start":0,"end":1000{extra},"x":0,"y":0,"width":100,"height":100}}"##
    )
}

#[track_caller]
fn report_on(body: &str) -> Report {
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(&dir, "p.montaget.json", body);
    validate(&path)
}

/// The report for a project holding one track of `elements`.
#[track_caller]
fn report_on_elements(elements: &str) -> Report {
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(
        &dir,
        "p.montaget.json",
        &project(&track("titles", 10, elements)),
    );
    validate(&path)
}

#[track_caller]
fn codes(report: &Report) -> Vec<&str> {
    report.findings.iter().map(|f| f.code.as_str()).collect()
}

#[track_caller]
fn only(report: &Report, code: &str) -> Finding {
    let mut of_code = report.findings.iter().filter(|f| f.code == code);
    let finding = of_code
        .next()
        .unwrap_or_else(|| panic!("no {code} in {:?}", codes(report)))
        .clone();
    assert!(
        of_code.next().is_none(),
        "expected exactly one {code}, got {:?}",
        codes(report)
    );
    finding
}

fn render(report: &Report) -> String {
    text::render(&report.to_json(), text::Options::verbose()).expect("the report renders")
}

// ---------------------------------------------------------------------------
// The gap the ticket opens with: a fault that used to validate clean.
// ---------------------------------------------------------------------------

#[test]
fn an_unknown_key_on_an_element_is_an_error_naming_the_key() {
    // #48 measured agents systematically mistyping fields copied from examples — `gravty`
    // for `gravity`. Before this check the typo validated clean, exit 0.
    let report = report_on_elements(&rect("card-05", r##","gravty":"top""##));

    let finding = only(&report, "E-SCHEMA-UNKNOWN-KEY");
    assert_eq!(finding.class, Class::Error);
    assert_eq!(finding.fields["key"], "gravty");
    assert_eq!(finding.location.element.as_deref(), Some("card-05"));
    assert_eq!(finding.location.track.as_deref(), Some("titles"));
    assert_eq!(report.exit_code(), ExitCode::Errors);
}

#[test]
fn the_five_conditions_the_ticket_names_all_report() {
    // #244's own list: an unknown key, an `fps` of `"abc"`, a `layer` of `1.5`, a `group`
    // written as `null`, and a missing required `width`. Every one of them validated clean
    // before this check existed.
    let unknown = report_on_elements(&rect("a", r##","nope":1"##));
    assert!(codes(&unknown).contains(&"E-SCHEMA-UNKNOWN-KEY"));

    let fps = report_on(&canonical(
        r##"{"frame":{"width":1080,"height":1920},"fps":"abc","tracks":[]}"##,
    ));
    assert_eq!(only(&fps, "E-SCHEMA").fields["subject"], "the project");

    let layer = report_on_elements(&rect("a", r##","layer":1.5"##));
    assert!(
        only(&layer, "E-SCHEMA").fields["reason"]
            .as_str()
            .unwrap()
            .contains("not an integer or an anchor"),
        "{:?}",
        only(&layer, "E-SCHEMA").fields
    );

    let group = report_on_elements(&rect("a", r##","group":null"##));
    assert!(
        only(&group, "E-SCHEMA").fields["reason"]
            .as_str()
            .unwrap()
            .contains("omit the key instead")
    );

    let width = report_on_elements(
        r##"{"id":"a","type":"rect","start":0,"end":1000,"x":0,"y":0,"height":100}"##,
    );
    assert_eq!(
        only(&width, "E-SCHEMA").fields["reason"],
        "missing field `width`"
    );
    assert_eq!(width.exit_code(), ExitCode::Errors);
}

#[test]
fn a_clean_project_produces_no_schema_finding() {
    let report = report_on_elements(&rect("a", ""));

    assert!(
        report.findings.is_empty(),
        "a legal project has nothing to say: {:?}",
        codes(&report)
    );
    assert_eq!(report.exit_code(), ExitCode::Ok);
}

// ---------------------------------------------------------------------------
// Where the finding is.
// ---------------------------------------------------------------------------

#[test]
fn a_fault_is_located_at_the_scope_it_is_in() {
    let at_project = report_on(&canonical(
        r##"{"frame":{"width":1080,"height":1920},"fps":25,"version":1,"tracks":[]}"##,
    ));
    let finding = only(&at_project, "E-SCHEMA-UNKNOWN-KEY");
    assert_eq!(finding.fields["subject"], "the project");
    assert_eq!(finding.fields["key"], "version");
    assert_eq!(finding.location.element, None);
    assert_eq!(finding.location.track, None);

    // A track's kind is the kind of its elements — derived, and therefore impossible to
    // forget, mistype or leave stale (ADR-0006).
    let at_track = report_on(&project(
        r##"{"name":"titles","layer":10,"kind":"visual","elements":[]}"##,
    ));
    let finding = only(&at_track, "E-SCHEMA-UNKNOWN-KEY");
    assert_eq!(finding.fields["subject"], "the track `titles`");
    assert_eq!(finding.fields["key"], "kind");
    assert_eq!(finding.location.track.as_deref(), Some("titles"));
    assert_eq!(finding.location.element, None);
}

#[test]
fn an_element_fault_carries_the_line_it_is_written_on() {
    // The canonical convention is one element per line (ADR-0041), which is what makes an
    // id's own spelling a line number at all: a pretty-printed file would put the id and
    // the offending key on different lines. In this project the one element is line 9.
    let report = report_on_elements(&rect("card-05", r##","gravty":"top""##));

    assert_eq!(only(&report, "E-SCHEMA-UNKNOWN-KEY").location.line, Some(9));
}

#[test]
fn a_fault_inside_a_run_is_reported_at_its_element() {
    // ADR-0017's scope decision, 2–1 for uniform closure: a `run` misspelling a style key
    // is exactly the silent-drift failure the policy exists to catch. `runs[2]` is not a
    // locus the report knows how to name, so the finding lands on the element.
    let report = report_on_elements(
        r##"{"id":"line-01","type":"text","start":0,"end":1000,"width":900,"height":100,"font":"brand","size":40,"runs":[{"text":"hi","colour":"#FFFFFF"}]}"##,
    );

    let finding = only(&report, "E-SCHEMA-UNKNOWN-KEY");
    assert_eq!(finding.fields["key"], "colour");
    assert_eq!(finding.location.element.as_deref(), Some("line-01"));
}

#[test]
fn an_element_carrying_no_id_is_still_named() {
    let report =
        report_on_elements(r##"{"type":"rect","start":0,"end":1000,"width":1,"height":1}"##);

    let finding = only(&report, "E-SCHEMA");
    assert_eq!(finding.fields["subject"], "an element carrying no `id`");
    assert_eq!(finding.location.element, None);
    assert_eq!(finding.location.track.as_deref(), Some("titles"));
}

// ---------------------------------------------------------------------------
// One fault does not hide the next.
// ---------------------------------------------------------------------------

#[test]
fn every_faulty_element_is_reported_not_only_the_first() {
    // `serde` stops at the first thing it cannot read, so a single strict parse of the
    // whole document would answer with one fact about whichever element came first. The
    // check parses by scope precisely so it does not.
    let report = report_on_elements(&format!(
        "{},{},{}",
        rect("a", r##","nope":1"##),
        rect("b", ""),
        rect("c", r##","also-nope":2"##),
    ));

    let keys: Vec<&str> = report
        .findings
        .iter()
        .filter(|f| f.code == "E-SCHEMA-UNKNOWN-KEY")
        .map(|f| f.fields["key"].as_str().unwrap())
        .collect();
    assert_eq!(keys, ["nope", "also-nope"]);
}

#[test]
fn a_faulty_track_does_not_silence_its_own_elements() {
    let report = report_on(&project(
        r##"{"name":"titles","kind":"visual","layer":10,"elements":[{"id":"a","type":"rect","start":0,"end":1000,"nope":1,"width":1,"height":1}]}"##,
    ));

    let subjects: Vec<&str> = report
        .findings
        .iter()
        .filter(|f| f.code == "E-SCHEMA-UNKNOWN-KEY")
        .map(|f| f.fields["subject"].as_str().unwrap())
        .collect();
    assert_eq!(subjects, ["the track `titles`", "a"]);
}

// ---------------------------------------------------------------------------
// ADR-0016's two cases: the retired key is not reported twice.
// ---------------------------------------------------------------------------

#[test]
fn a_retired_key_is_named_by_the_retired_check_and_not_by_this_one() {
    // ADR-0016 requires the two unknown-key cases distinguished in the message text, and
    // its own draft says so out loud: "not a key this Montaget knows, *and not one it has
    // retired*". `gravity` is retired, so it is the retirement's sentence, not this one's.
    let report = report_on_elements(&rect("photo-06", r##","gravity":"bottom""##));

    assert_eq!(codes(&report), ["E-RETIRED-KEY"]);
}

#[test]
fn every_retired_key_stays_the_retired_checks_to_report() {
    // The five retirements that are *keys* rather than values. Each is an unknown field to
    // the types, so each would otherwise be reported twice.
    for (spelling, element) in [
        ("gravity", rect("a", r##","gravity":"top""##)),
        ("box", rect("a", r##","box":[0,0,10,10]"##)),
        ("mask", rect("a", r##","mask":"circle""##)),
        ("weight", rect("a", r##","weight":"bold""##)),
        (
            "align",
            r##"{"id":"a","type":"image","start":0,"end":1000,"source":"s.png","align":"top","width":10,"height":10,"fit":"literal"}"##.to_string(),
        ),
    ] {
        let report = report_on_elements(&element);
        assert!(
            !codes(&report).contains(&"E-SCHEMA-UNKNOWN-KEY"),
            "`{spelling}` is retired, so the retirement speaks: {:?}",
            codes(&report)
        );
        assert!(
            codes(&report).contains(&"E-RETIRED-KEY") || codes(&report).contains(&"E-RETIRED-SPELLING"),
            "`{spelling}` should still be reported by someone: {:?}",
            codes(&report)
        );
    }
}

#[test]
fn a_retired_value_is_left_to_the_retirement_too() {
    // ADR-0016 speaks about keys, and four of the nine retirements are *values*. They are
    // left to the retirement on a different ground: the retirement is advise-class and
    // states the replacement, this check is refuse-class and states that no repair is
    // determined, and a report carrying both about the same byte contradicts itself.
    let background = report_on(&canonical(
        r##"{"frame":{"width":1080,"height":1920},"fps":25,"background":"#FBF3E3FF","tracks":[]}"##,
    ));
    assert_eq!(codes(&background), ["E-RETIRED-SPELLING"]);

    let origin = report_on_elements(&rect("a", r##","origin":"center-center""##));
    assert_eq!(codes(&origin), ["E-RETIRED-SPELLING"]);

    // An image, so this one also reaches the disk — its `source` is not there, and that is
    // the source check's finding and not a schema one.
    let fit = report_on_elements(
        r##"{"id":"a","type":"image","start":0,"end":1000,"source":"s.png","width":10,"height":10,"fit":"none"}"##,
    );
    assert!(
        codes(&fit).contains(&"E-RETIRED-SPELLING"),
        "{:?}",
        codes(&fit)
    );
    assert!(!codes(&fit).contains(&"E-SCHEMA"), "{:?}", codes(&fit));
}

#[test]
fn a_retirement_silences_this_check_only_at_the_element_it_is_on() {
    // The predicate is `(element, key)`, not `key`. A second element misspelling the same
    // word is not covered by the first element's retirement.
    let report = report_on_elements(&format!(
        "{},{}",
        rect("a", r##","gravity":"top""##),
        rect("b", r##","nope":1"##),
    ));

    let finding = only(&report, "E-SCHEMA-UNKNOWN-KEY");
    assert_eq!(finding.location.element.as_deref(), Some("b"));
}

// ---------------------------------------------------------------------------
// The three silences #198 left for this check.
// ---------------------------------------------------------------------------

#[test]
fn the_silences_the_anchor_check_left_to_this_one_now_speak() {
    // `stack.rs`'s `Unresolved::Malformed` and `Unresolved::Unstated`, and the `anchor`
    // check that matches both to `{}`, stay silent "because the check that owns the schema
    // says so". Each of the three conditions is asserted here, at that check.

    // A `layer` that is neither an integer nor an object (`Own::Malformed`).
    let malformed = report_on_elements(&rect("a", r##","layer":"titles""##));
    assert!(
        codes(&malformed).contains(&"E-SCHEMA"),
        "{:?}",
        codes(&malformed)
    );

    // A target whose own `layer` is unreadable, reached through an anchor.
    let via_anchor = report_on_elements(&format!(
        "{},{}",
        rect("title", r##","layer":"nonsense""##),
        rect("panel", r##","layer":{"below":"title"}"##),
    ));
    assert!(
        codes(&via_anchor).contains(&"E-SCHEMA"),
        "{:?}",
        codes(&via_anchor)
    );

    // A track carrying no `layer` (`Unresolved::Unstated`).
    let unstated = report_on(&project(
        r##"{"name":"titles","elements":[{"id":"a","type":"rect","start":0,"end":1000,"width":1,"height":1}]}"##,
    ));
    let finding = only(&unstated, "E-SCHEMA");
    assert_eq!(finding.fields["subject"], "the track `titles`");
    assert_eq!(finding.fields["reason"], "missing field `layer`");
}

// ---------------------------------------------------------------------------
// The wiring: the registry, the classes, the prose.
// ---------------------------------------------------------------------------

#[test]
fn the_registry_declares_every_code_this_check_can_fire() {
    // A check emitting an unregistered code panics in `Finding::new`, which is a bug in the
    // check rather than a condition of the project.
    for code in ["E-SCHEMA", "E-SCHEMA-UNKNOWN-KEY"] {
        let spec = montaget_core::registry::spec(code)
            .unwrap_or_else(|| panic!("{code} is not registered"));
        assert_eq!(spec.classes, &[Class::Error]);
        assert_eq!(spec.status, montaget_core::registry::Status::Live);
    }
}

#[test]
fn both_codes_are_refuse_class() {
    // ADR-0043's uniformity rule, per check and never per instance: this check matches a
    // typo and a key from a newer format revision alike, and ADR-0016 forbids deleting the
    // second — "Do not delete the key to make the file validate."
    let unknown = report_on_elements(&rect("a", r##","nope":1"##));
    assert_eq!(
        only(&unknown, "E-SCHEMA-UNKNOWN-KEY").repair,
        Some(Repair::None)
    );

    let fault = report_on_elements(&rect("a", r##","layer":1.5"##));
    assert_eq!(only(&fault, "E-SCHEMA").repair, Some(Repair::None));
}

#[test]
fn the_unknown_key_finding_reads_as_adr_0016_wrote_it() {
    // ADR-0016's message is measured rather than drafted: of nine agents met by a stale
    // binary, the arm naming the binary preserved and compensated where the arm printing a
    // bare key list squashed the photos. The sentences that did that work are asserted.
    let rendered = render(&report_on_elements(&rect(
        "card-05",
        r##","gravty":"top""##,
    )));

    for sentence in [
        "card-05: unknown key `gravty`",
        "not a key this Montaget knows, and not one it has retired",
        "It may belong to a newer format revision than this binary implements",
        "Check your Montaget version before removing it",
        "Do not delete the key to make the file validate",
        // The near-miss list, which is what turns a typo back into the key it was copied
        // from: `gravty` against a published `gravity` that is no longer there.
        "`x`, `y`, `origin`, `width`, `height`, `fill`",
    ] {
        assert!(
            rendered.contains(sentence),
            "missing {sentence:?}:\n{rendered}"
        );
    }
}

#[test]
fn a_schema_finding_speaks_the_formats_own_words() {
    // The reason is whatever the types said, which for the values the format has an opinion
    // about is a sentence the format wrote: two spellings of one colour break the
    // write-read round trip (ADR-0014), and the message says which one to write.
    let rendered = render(&report_on_elements(&rect("a", r##","fill":"#FFF""##)));

    assert!(
        rendered.contains("a does not fit the published schema: #FFF is the three-digit shorthand; write all six digits."),
        "{rendered}"
    );
}

// ---------------------------------------------------------------------------
// The invariant the ticket exists for.
// ---------------------------------------------------------------------------

#[test]
fn a_document_the_strict_parse_refuses_never_validates_clean() {
    // The property #244 is about, stated directly: `Loose::strict` erring and the report
    // saying nothing is the state the ticket found, and it must not be reachable.
    let faults = [
        r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[],"version":1}"##,
        r##"{"frame":{"width":1080},"fps":25,"tracks":[]}"##,
        r##"{"frame":{"width":1080,"height":1920},"fps":25}"##,
        r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":{}}"##,
        r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[{"name":"t"}]}"##,
        r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[{"name":"t","layer":1,"elements":[{"id":"a"}]}]}"##,
        r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[{"name":"t","layer":1,"elements":[{"id":"a","type":"rect","start":0.5,"end":1,"width":1,"height":1}]}]}"##,
        r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[{"name":"t","layer":1,"elements":[{"id":"a","type":"recct","start":0,"end":1,"width":1,"height":1}]}]}"##,
    ];

    for fault in faults {
        let document = Loose::new("p.montaget.json", serde_json::from_str(fault).unwrap());
        assert!(
            document.strict().is_err(),
            "this test's own premise: {fault} should not fit the types"
        );
        let findings = montaget_core::checks::schema::findings(&document);
        assert!(
            !findings.is_empty(),
            "`strict` refuses this and the check said nothing: {fault}"
        );
    }
}

#[test]
fn a_document_the_strict_parse_accepts_produces_nothing() {
    // The other direction, so the invariant above cannot be satisfied by a check that
    // fires on everything.
    let document = Loose::new(
        "p.montaget.json",
        serde_json::from_str(&project(&track("titles", 10, &rect("a", "")))).unwrap(),
    );

    assert!(document.strict().is_ok());
    assert!(montaget_core::checks::schema::findings(&document).is_empty());
}
