//! ADR-0088's four `chroma` findings (#342), and the model rules the derive cannot state.
//!
//! The findings are the half of ADR-0088 that answers *"is this key legal, and does it
//! agree with the media on disk"* — never *"did it key well"*, which is `measure`'s
//! question and needs a frame. So every assertion here is made against a document, and the
//! one that needs the disk needs exactly one `ffprobe` field.
//!
//! Both directions throughout, as `chroma_key_scan.sh` does: the document that must fire
//! and the one that must not. A check that fired on every `chroma` member would pass a
//! one-sided test and make the whole vocabulary unusable.

use montagent_core::finding::{Class, Finding};
use montagent_core::report::Report;
use montagent_core::{checks, parse};
use serde_json::json;

mod common;
use common::{canonical, write_project};

fn project(elements: &str) -> String {
    canonical(&format!(
        r##"{{"frame":{{"width":1920,"height":1080}},"fps":24,"tracks":[{{"name":"t","layer":10,"elements":[{elements}]}}]}}"##
    ))
}

/// A `video` element carrying `effects` verbatim — the case the keyer exists for.
fn clip(id: &str, effects: &str) -> String {
    format!(
        r##"{{"id":"{id}","type":"video","start":0,"end":5833,"source":"green-screen-trex.mp4","source_start":0,"source_end":5833,"x":0,"y":0,"origin":"top-left","width":1920,"height":1080,"fit":"literal","effects":[{effects}]}}"##
    )
}

/// The key ADR-0088's forcing case uses, at the middle of its measured plateau.
const KEY: &str =
    r##"{"name":"chroma","color":"#00CD00","tolerance":0.1,"softness":0.08,"spill":0.5}"##;

#[track_caller]
fn report_on(elements: &str) -> Report {
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(&dir, "p.montagent.json", &project(elements));
    let document = parse::read(&path).expect("the project parses");
    let mut report = Report::new("validate", Some(document.path().to_string()));
    checks::chroma::check(&document, &mut report);
    report
}

#[track_caller]
fn coded<'a>(report: &'a Report, code: &str) -> Vec<&'a Finding> {
    report
        .findings
        .iter()
        .filter(|finding| finding.code == code)
        .collect()
}

// ---------------------------------------------------------------------------
// The key that is legal, which is most of them
// ---------------------------------------------------------------------------

#[test]
fn the_forcing_cases_own_key_on_its_own_clip_reports_nothing() {
    // ADR-0088's worked example, on the element type it was measured on. A check suite that
    // could not stay silent here would have made the member unusable the day it shipped.
    let report = report_on(&clip("trex", KEY));
    assert!(report.findings.is_empty(), "{:?}", report.findings);
}

// ---------------------------------------------------------------------------
// R-CHROMA-AFTER-COLOUR
// ---------------------------------------------------------------------------

#[test]
fn a_colour_scalar_ahead_of_the_key_is_reported_with_both_positions() {
    // "`effects` is ordered and order is semantically real (ADR-0040), so a colour scalar
    // ahead of the key changes the pixels the key is measured against, and the author's
    // `color` no longer names what is in the frame."
    let report = report_on(&clip(
        "trex",
        &format!(r##"{{"name":"saturation","amount":1.4}},{KEY}"##),
    ));
    let found = coded(&report, "R-CHROMA-AFTER-COLOUR");
    assert_eq!(found.len(), 1, "{:?}", report.findings);
    assert_eq!(found[0].class, Class::Review, "a pre-grade is a technique");
    assert_eq!(found[0].fields["index"], json!(1));
    assert_eq!(found[0].fields["colour_index"], json!(0));
    assert_eq!(found[0].fields["colour_name"], json!("saturation"));
    assert_eq!(found[0].fields["color"], json!("#00CD00"));
}

#[test]
fn a_colour_scalar_after_the_key_is_ordinary() {
    // The other direction, and the one that matters most: grading what the key kept is the
    // normal order, and a check that fired on it would be noise on every correct document.
    let report = report_on(&clip(
        "trex",
        &format!(r##"{KEY},{{"name":"saturation","amount":1.4}}"##),
    ));
    assert!(
        coded(&report, "R-CHROMA-AFTER-COLOUR").is_empty(),
        "{:?}",
        report.findings
    );
}

#[test]
fn blur_shadow_and_mask_ahead_of_the_key_are_not_colour_operations() {
    // The finding is scoped to ADR-0049's four colour-filter members. `blur` and `shadow`
    // do not restate a pixel's colour, and `mask` is the other matte operation — ADR-0088's
    // whole jurisdictional argument turns on that distinction, so a check that lumped them
    // together would contradict the ADR it comes from.
    for ahead in [
        r##"{"name":"blur","radius":4}"##,
        r##"{"name":"shadow","dx":2,"dy":2,"radius":4,"color":"#000000","opacity":0.5}"##,
        r##"{"name":"mask","shape":"rect"}"##,
    ] {
        let report = report_on(&clip("trex", &format!("{ahead},{KEY}")));
        assert!(
            coded(&report, "R-CHROMA-AFTER-COLOUR").is_empty(),
            "{ahead} fired: {:?}",
            report.findings
        );
    }
}

#[test]
fn two_scalars_ahead_of_one_key_are_one_finding_naming_the_first() {
    // One key, one fact: "the key is not measured against the source's own colour". A
    // finding per pair would report the same defect three times on a graded element, and
    // the reader would have three sentences and one problem.
    let report = report_on(&clip(
        "trex",
        &format!(
            r##"{{"name":"brightness","amount":0.1}},{{"name":"contrast","amount":0.2}},{KEY}"##
        ),
    ));
    let found = coded(&report, "R-CHROMA-AFTER-COLOUR");
    assert_eq!(found.len(), 1, "{:?}", report.findings);
    assert_eq!(found[0].fields["colour_index"], json!(0));
    assert_eq!(found[0].fields["colour_name"], json!("brightness"));
}

// ---------------------------------------------------------------------------
// R-CHROMA-ON-AUTHORED-ELEMENT
// ---------------------------------------------------------------------------

#[test]
fn a_key_on_pixels_the_format_authored_is_reported_on_all_three_types() {
    // "The format authored those pixels; keying a colour out of them is the author asking
    // for a shape they could have declared."
    for (declared, tail) in [
        ("rect", r##""width":100,"height":100,"fill":"#00CD00""##),
        ("ellipse", r##""width":100,"height":100,"fill":"#00CD00""##),
        (
            "text",
            r##""width":100,"height":100,"size":40,"runs":[{"text":"hi"}]"##,
        ),
    ] {
        let element = format!(
            r##"{{"id":"a","type":"{declared}","start":0,"end":1000,"x":0,"y":0,"origin":"top-left",{tail},"effects":[{KEY}]}}"##
        );
        let report = report_on(&element);
        let found = coded(&report, "R-CHROMA-ON-AUTHORED-ELEMENT");
        assert_eq!(found.len(), 1, "on a `{declared}`: {:?}", report.findings);
        assert_eq!(found[0].fields["type"], json!(declared));
        assert_eq!(found[0].class, Class::Review);
    }
}

#[test]
fn a_key_on_a_source_element_is_the_ordinary_case() {
    // `image` and `video` are the other side of the line: their pixels come off a disk this
    // document did not write, which is the case the keyer exists for.
    let report = report_on(&clip("trex", KEY));
    assert!(
        coded(&report, "R-CHROMA-ON-AUTHORED-ELEMENT").is_empty(),
        "{:?}",
        report.findings
    );
}

// ---------------------------------------------------------------------------
// N-CHROMA-INERT
// ---------------------------------------------------------------------------

#[test]
fn a_key_at_its_identity_tolerance_is_a_note_and_any_other_tolerance_is_not() {
    // ADR-0088 gives `tolerance` its identity at `0`, "which keys nothing" — the shape
    // ADR-0052 already made a finding for with inert ease. Note-class: the document is
    // legal and the render is a picture, it is simply not the picture the author drew.
    let inert = r##"{"name":"chroma","color":"#00CD00","tolerance":0,"softness":0,"spill":0}"##;
    let report = report_on(&clip("trex", inert));
    let found = coded(&report, "N-CHROMA-INERT");
    assert_eq!(found.len(), 1, "{:?}", report.findings);
    assert_eq!(found[0].class, Class::Note);
    assert_eq!(found[0].fields["index"], json!(0));

    // The identity-adjacent value, which is the one `chroma_key_scan.sh` measured keying
    // nothing on the forcing case. It is still a real key, and the finding is about the
    // declared identity rather than about the outcome — which `validate` cannot see.
    let barely = r##"{"name":"chroma","color":"#00CD00","tolerance":0.01,"softness":0,"spill":0}"##;
    let report = report_on(&clip("trex", barely));
    assert!(
        coded(&report, "N-CHROMA-INERT").is_empty(),
        "{:?}",
        report.findings
    );
}

// ---------------------------------------------------------------------------
// Two keys in one list, and the prose
// ---------------------------------------------------------------------------

#[test]
fn two_keys_on_one_element_are_two_findings_naming_their_own_positions() {
    // "Two `chroma` members in one list are legal and apply in order" (ADR-0088), so the
    // index is part of a finding's identity — without it a reader with two keys cannot tell
    // which one the sentence is about.
    let inert = r##"{"name":"chroma","color":"#00CD00","tolerance":0,"softness":0,"spill":0}"##;
    let report = report_on(&clip(
        "trex",
        &format!("{inert},{{\"name\":\"blur\",\"radius\":2}},{inert}"),
    ));
    let found = coded(&report, "N-CHROMA-INERT");
    assert_eq!(found.len(), 2, "{:?}", report.findings);
    assert_eq!(found[0].fields["index"], json!(0));
    assert_eq!(found[1].fields["index"], json!(2));
}

#[test]
fn every_finding_renders_the_prose_the_registry_declares() {
    // ADR-0006's one-code-one-template contract. Each sentence states a fact and none
    // proposes a repair (ADR-0043, ADR-0088) — the author is the only one who knows which
    // of these was meant.
    let graded = format!(r##"{{"name":"tint","color":"#FF0000","amount":0.2}},{KEY}"##);
    let report = report_on(&clip("trex", &graded));
    // `--verbose`, because `note` collapses to a counted line otherwise and the sentence
    // under test is the thing that would not print.
    let prose =
        montagent_core::text::render(&report.to_json(), montagent_core::text::Options::verbose())
            .expect("the template renders");
    assert!(prose.contains("#00CD00"), "{prose}");
    assert!(prose.contains("`effects[1]`"), "{prose}");
    assert!(prose.contains("`tint`"), "{prose}");

    let inert = r##"{"name":"chroma","color":"#00CD00","tolerance":0,"softness":0,"spill":0}"##;
    let report = report_on(&clip("trex", inert));
    let prose =
        montagent_core::text::render(&report.to_json(), montagent_core::text::Options::verbose())
            .expect("the template renders");
    assert!(prose.contains("keys nothing"), "{prose}");
}

// ---------------------------------------------------------------------------
// The model rules the derive cannot state (ADR-0088)
// ---------------------------------------------------------------------------

/// The same project, but held to the types rather than read permissively — where the
/// bounds and the `#RRGGBB` narrowing live.
#[track_caller]
fn parsed(effects: &str) -> Result<montagent_core::model::Project, String> {
    serde_json::from_str::<montagent_core::model::Project>(&project(&clip("trex", effects)))
        .map_err(|e| e.to_string())
}

#[test]
fn the_forcing_cases_own_key_parses() {
    // The one worked example ADR-0088 writes out, verbatim. If this does not parse, nothing
    // below about what is *refused* means anything.
    parsed(KEY).expect("ADR-0088's own example is a legal key");
}

#[test]
fn each_of_the_three_scalars_is_bounded_at_both_ends_and_names_itself() {
    // "`tolerance` — `0.0`–`1.0`", and the same for `softness` and `spill`. An `f64` field
    // cannot say so, which is why the rule sits in `Effect::checked` beside ADR-0084's two.
    for scalar in ["tolerance", "softness", "spill"] {
        for out_of_range in ["1.5", "-0.1"] {
            let effects = r##"{"name":"chroma","color":"#00CD00","tolerance":0.1,"softness":0.0,"spill":0.0}"##
            .replace(
                &format!(r#""{scalar}":0.1"#),
                &format!(r#""{scalar}":{out_of_range}"#),
            )
            .replace(
                &format!(r#""{scalar}":0.0"#),
                &format!(r#""{scalar}":{out_of_range}"#),
            );
            let message = parsed(&effects)
                .err()
                .unwrap_or_else(|| panic!("`{scalar}` at {out_of_range} was accepted"));
            assert!(
                message.contains(scalar),
                "the message names the parameter that is wrong: {message}"
            );
            assert!(message.contains("[0, 1]"), "{message}");
        }
    }
}

#[test]
fn both_ends_of_the_range_are_legal_and_the_identity_is_one_of_them() {
    // A bound that refused its own endpoints would make the identity value unwritable,
    // which is the one value ADR-0049's clause (c) requires every parameter to have.
    for value in ["0", "1"] {
        let effects = format!(
            r##"{{"name":"chroma","color":"#00CD00","tolerance":{value},"softness":{value},"spill":{value}}}"##
        );
        parsed(&effects).unwrap_or_else(|e| panic!("{value} at every end is legal: {e}"));
    }
}

#[test]
fn an_alpha_on_the_key_colour_is_refused_and_says_why() {
    // "Not `#RRGGBBAA`: an alpha on the key colour is meaningless and would be a second way
    // to say nothing." `Colour` admits both spellings — `shadow.color` and every `fill` may
    // carry an alpha — so the narrowing is this member's own.
    // A *translucent* key colour, because `#00CD00FF` never reaches this rule: `Colour`
    // itself refuses the fully-opaque eight-digit form as a second spelling of the
    // six-digit one (ADR-0014). This is the spelling that is a legal colour everywhere
    // else in the format and is not one here.
    let effects =
        r##"{"name":"chroma","color":"#00CD0080","tolerance":0.1,"softness":0.0,"spill":0.0}"##;
    let message = parsed(effects).expect_err("an alpha is refused");
    assert!(message.contains("#00CD0080"), "{message}");
    assert!(
        message.contains("never one to composite"),
        "the refusal names its reason: {message}"
    );

    // And the other direction: a `shadow` in the same list keeps its alpha, because the
    // rule is this member's rather than the format's.
    parsed(r##"{"name":"shadow","dx":1,"dy":1,"radius":2,"color":"#00000080","opacity":1.0}"##)
        .expect("`shadow.color` still takes an alpha");
}

#[test]
fn the_member_is_closed_like_every_other_one() {
    // ADR-0017 at the member level: `chroma` is an eighth *named* member with a fixed
    // parameter set, not an escape hatch. A fifth parameter is an unknown key.
    let effects = r##"{"name":"chroma","color":"#00CD00","tolerance":0.1,"softness":0.0,"spill":0.0,"despill_colour":"#FFFFFF"}"##;
    let message = parsed(effects).expect_err("a fifth parameter is refused");
    assert!(message.contains("despill_colour"), "{message}");
}

// ---------------------------------------------------------------------------
// `measure`'s dispatch: which answer an element has is a property of the element
// ---------------------------------------------------------------------------

#[test]
fn a_keyed_text_element_still_measures_as_text() {
    // ADR-0088 makes a key on an authored element a `review`, not an error — so such a
    // document is legal, and the text in it still occupies a block. `measure`'s coverage
    // answer belongs to elements whose pixels came off a file; taking ADR-0024's extent
    // away from a legal `text` element would be a regression this ADR never asked for, and
    // `R-CHROMA-ON-AUTHORED-ELEMENT` is already the thing that says the key is odd.
    let dir = common::tempdir(line!());
    let element = format!(
        r##"{{"id":"caption","type":"text","start":0,"end":1000,"x":0,"y":0,"origin":"top-left","width":400,"height":80,"font":"brand","size":40,"runs":[{{"text":"hi"}}],"effects":[{KEY}]}}"##
    );
    let fonts = common::fixture_dir();
    let body = canonical(&format!(
        r##"{{"frame":{{"width":400,"height":400}},"fps":24,"fonts":{{"brand":[{{"file":"{path}"}}]}},"tracks":[{{"name":"t","layer":10,"elements":[{element}]}}]}}"##,
        path = common::with_forward_slashes(
            &fonts.join("fonts/OpenRunde-Bold.otf").display().to_string()
        ),
    ));
    let path = write_project(&dir, "p.montagent.json", &body);

    let answer = montagent_core::verbs::measure::measure(
        &path,
        &montagent_core::verbs::measure::Ask {
            element: Some(serde_json::from_str(&element).expect("the element is JSON")),
            ..Default::default()
        },
    );
    let json = answer.to_json();
    assert_eq!(
        json["measure"]["mode"],
        "element",
        "a keyed `text` element keeps its text answer: {}",
        serde_json::to_string_pretty(&json).unwrap_or_default()
    );
    assert!(
        json["measure"]["advance_width"].as_f64().unwrap_or(0.0) > 0.0,
        "{}",
        serde_json::to_string_pretty(&json).unwrap_or_default()
    );
}
