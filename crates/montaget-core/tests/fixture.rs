//! The committed fixture as a regression guard.
//!
//! `fixtures/en-halloween-decorating/` is a real published video. Spec #168: *"it is the
//! regression guard: it is a real published video, and a check that fires on it is wrong
//! unless an ADR says otherwise."*
//!
//! Its authority stops there. The fixture is evidence that a capability is **needed**,
//! never evidence that one is **unneeded** — it carries zero layer anchors, zero
//! non-`linear` eases, zero `opacity` occurrences and zero `video` elements, and none of
//! that is a reason to drop a check.

use montaget_core::report::ExitCode;
use montaget_core::{parse, validate};
use std::path::PathBuf;

mod common;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/en-halloween-decorating/en-halloween-decorating.montaget.json")
}

#[test]
fn the_fixture_parses_and_reports_nothing_to_act_on() {
    // Since #203 this exercises the disk half too, so it needs the `ffmpeg` ADR-0009 has
    // the user supply. Asked directly rather than inferred from the exit code afterwards,
    // which would also swallow every other internal failure.
    if !common::has_ffprobe() {
        return;
    }
    let report = validate(&fixture());

    let summary = report.summary();
    assert_eq!(
        (
            summary.error,
            summary.review,
            summary.unchecked,
            summary.layout
        ),
        (0, 0, 0, 0),
        "the fixture is a published video; anything to act on is a defect in the check: {:?}",
        report.findings
    );
    assert_eq!(report.exit_code(), ExitCode::Ok);
}

#[test]
fn the_only_thing_the_fixture_has_to_say_is_its_own_gaps() {
    // The one check that fires on the fixture, and an ADR says it should: ADR-0004
    // requires the validator to distinguish an overlap from a gap rather than pass a gap
    // in silence, and `CONTEXT.md` calls a gap legal and ordinary — "the silence between
    // two narration lines is a gap". Nineteen of these are in `narration` and are exactly
    // that.
    //
    // **27, against ADR-0006's and #200's "eleven".** Not a disagreement: that eleven is
    // the count of *visual* gaps, which is the question `R-VISUAL-GAP` asks (ADR-0018,
    // #200) — this check counts every track's, `narration` included, and the fixture's
    // audio track is where most of them are.
    //
    // They are `note`, so ADR-0006's noise budget collapses them to one counted line: the
    // clean case is still one line, and an edit that punched black frames into a track
    // moves the count.
    if !common::has_ffprobe() {
        return;
    }
    let report = validate(&fixture());

    let gaps: Vec<&montaget_core::finding::Finding> = report
        .findings
        .iter()
        .filter(|f| f.code == "N-TRACK-GAP")
        .collect();
    assert_eq!(
        gaps.len(),
        report.findings.len(),
        "nothing else fires: {:?}",
        report.findings
    );
    assert_eq!(gaps.len(), 27, "27 gaps across three tracks");

    // The shortest is the 450 ms between the intro and the hook, and the longest the
    // 7260 ms `sentence-card` holds between item-08 and the quiz. Both are silence a
    // reader can hear in the published video.
    let sizes: Vec<i64> = gaps
        .iter()
        .map(|f| f.fields["size"].as_i64().unwrap())
        .collect();
    assert_eq!(sizes.iter().min(), Some(&450));
    assert_eq!(sizes.iter().max(), Some(&7260));
    assert!(
        gaps.iter()
            .all(|f| f.class == montaget_core::finding::Class::Note),
        "a gap is never an error, and the review belongs to the check that can see the \
         whole frame (ADR-0018, #200)"
    );

    let rendered =
        montaget_core::text::render(&report.to_json(), montaget_core::text::Options::default())
            .unwrap();
    assert_eq!(
        rendered.matches("N-TRACK-GAP").count(),
        1,
        "one counted line, not 27:\n{rendered}"
    );
}

#[test]
fn the_fixture_is_a_project_by_the_shared_structural_predicate() {
    let document = parse::read(&fixture()).expect("the fixture parses");

    assert!(document.shape().is_ok());
    assert_eq!(document.elements().count(), 60, "60 elements");
    assert_eq!(
        document.value()["tracks"].as_array().unwrap().len(),
        14,
        "14 tracks"
    );
}

#[test]
fn the_fixture_fits_the_format_s_own_types() {
    // The strict view is the whole format, and the fixture is the only evidence that it
    // describes a coherent document rather than seven ADRs that each read well alone.
    let project = parse::read(&fixture())
        .expect("the fixture parses")
        .strict()
        .expect("the fixture is a legal project");

    assert_eq!(project.fps, 25);
    assert_eq!(project.duration, Some(65216));
    assert_eq!((project.frame.width, project.frame.height), (1080, 1920));
    assert_eq!(project.tracks.len(), 14);

    // Every element carries the same shape whatever its type (`CONTEXT.md`) — which the
    // types now enforce rather than assert: `id`, `start` and `end` are not `Option`.
    let ids: Vec<&str> = project
        .tracks
        .iter()
        .flat_map(|track| track.elements.iter())
        .map(|element| element.id.as_str())
        .collect();
    assert_eq!(ids.len(), 60);
    assert_eq!(
        ids.iter().collect::<std::collections::BTreeSet<_>>().len(),
        60,
        "every element id is unique (ADR-0019)"
    );
}

#[test]
fn the_raw_value_stays_beside_the_typed_view() {
    // A check that reports on a document the types cannot hold — which is every check that
    // reports on a broken one — reads the value. It must not have to re-read the file.
    let document = parse::read(&fixture()).expect("the fixture parses");
    let first = document.elements().next().expect("an element");

    assert_eq!(first["source"], "images/05.png");
    assert_eq!(first["clip"], serde_json::json!([0, 0, 1080, 1300]));
    assert_eq!(
        document.value()["fonts"]["brand"][0]["file"],
        "fonts/OpenRunde-Bold.otf"
    );
}
