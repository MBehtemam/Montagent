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

use montagent_core::report::ExitCode;
use montagent_core::{parse, validate};
use std::path::PathBuf;

mod common;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/en-halloween-decorating/en-halloween-decorating.montagent.json")
}

#[test]
fn the_fixture_parses_and_carries_only_the_findings_two_adrs_asked_for() {
    // Since #203 this exercises the disk half too, so it needs the `ffmpeg` ADR-0009 has
    // the user supply. Asked directly rather than inferred from the exit code afterwards,
    // which would also swallow every other internal failure.
    if !common::has_ffprobe() {
        return;
    }
    let report = validate(&fixture());

    let summary = report.summary();
    assert_eq!(
        (summary.error, summary.unchecked, summary.layout),
        (0, 0, 0),
        "the fixture is a published video: {:?}",
        report.findings
    );

    // **Twelve `review` findings, and the fixture is still correct.** Spec #168 makes the
    // fixture the regression guard — "a check that fires on it is wrong *unless an ADR
    // says otherwise*" — and here three ADRs say otherwise about this exact file. ADR-0034
    // was written *from* `hook-loop`: 1200 ms for a line its own first showing gives 2298,
    // "at exactly the seam a looping short is supposed to make invisible". ADR-0054 was
    // written from the same element's silence. And ADR-0033's whole measured table is this
    // file's `photo` track: of its six cuts, the two same-source ones pop, at 3018 and at
    // 64016. Ten caption findings and two cut pops; none is an `error`, and none gates the
    // render.
    assert_eq!(summary.review, 12, "{:?}", report.findings);
    assert_eq!(report.exit_code(), ExitCode::Ok);
}

#[test]
fn the_fixture_s_ten_review_findings_are_the_two_defects_its_own_adrs_name() {
    // Named individually rather than counted, because a count that moved would not say
    // which check moved. ADR-0034: `hook-loop` outruns the pace floor, and three lines are
    // repeated at durations that disagree. ADR-0054: the loop-out and the five countdown
    // cards have nothing declared to be heard under them.
    if !common::has_ffprobe() {
        return;
    }
    let report = validate(&fixture());

    let mut captions: Vec<(String, String)> = report
        .findings
        .iter()
        .filter(|f| f.code.starts_with("R-CAPTION-"))
        .map(|f| {
            let subject = f
                .location
                .element
                .clone()
                .unwrap_or_else(|| f.fields["text"].as_str().unwrap_or("?").to_string());
            (f.code.clone(), subject)
        })
        .collect();
    captions.sort();

    assert_eq!(
        captions,
        [
            ("R-CAPTION-NO-AUDIO", "count-1"),
            ("R-CAPTION-NO-AUDIO", "count-2"),
            ("R-CAPTION-NO-AUDIO", "count-3"),
            ("R-CAPTION-NO-AUDIO", "count-4"),
            ("R-CAPTION-NO-AUDIO", "count-5"),
            ("R-CAPTION-NO-AUDIO", "hook-loop"),
            ("R-CAPTION-PACE", "hook-loop"),
            ("R-CAPTION-REPEAT-DURATION", "I hang cobwebs over the door."),
            (
                "R-CAPTION-REPEAT-DURATION",
                "What is this called\nin English?"
            ),
            ("R-CAPTION-REPEAT-DURATION", "cobweb  -  cobweb"),
        ]
        .map(|(code, subject)| (code.to_string(), subject.to_string()))
    );
}

#[test]
fn the_fixture_s_notes_are_its_own_gaps_and_they_collapse_to_one_line() {
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

    let gaps: Vec<&montagent_core::finding::Finding> = report
        .findings
        .iter()
        .filter(|f| f.code == "N-TRACK-GAP")
        .collect();
    assert_eq!(
        gaps.len(),
        report
            .findings
            .iter()
            .filter(|f| {
                !f.code.starts_with("R-CAPTION-")
                    && f.code != "R-VISUAL-GAP"
                    && f.code != "R-BOX-SLACK"
                    && f.code != "R-SOURCE-CUT-POP"
            })
            .count(),
        "nothing but the gaps, the group-paired silences and the seven `R-BOX-SLACK` \
         findings (asserted separately, below), the ten caption findings two ADRs asked \
         for, and the two same-source cut pops ADR-0033 measured here: {:?}",
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
            .all(|f| f.class == montagent_core::finding::Class::Note),
        "a gap is never an error, and the review belongs to the check that can see the \
         whole frame (ADR-0018, #200)"
    );

    let rendered =
        montagent_core::text::render(&report.to_json(), montagent_core::text::Options::default())
            .unwrap();
    assert_eq!(
        rendered.matches("N-TRACK-GAP").count(),
        1,
        "one counted line, not 27:\n{rendered}"
    );
}

#[test]
fn the_fixture_s_group_pairs_hold_their_photos_through_every_narration_breath() {
    // ADR-0018, #200: group-scoped coverage, symmetric between audio and visual. The
    // fixture's `header` group — eight elements, no audio member — is exactly the case
    // the union rule it replaces could never tell apart from real content, and pairing
    // asserts nothing about it rather than a false "covered".
    //
    // The 21 it does fire on are every group's ordinary narration breath: a photo held
    // through the silence between two lines, the same fact `N-TRACK-GAP` already reports
    // for the audio track's own gap. **Zero** fire the other way — audio outrunning its
    // group's visual, ADR-0018's own two founding defects (#23) — which is why this
    // fixture's `review` count (asserted elsewhere as 10) does not move: a published
    // video has none of that defect to report.
    if !common::has_ffprobe() {
        return;
    }
    let report = validate(&fixture());

    let gaps: Vec<&montagent_core::finding::Finding> = report
        .findings
        .iter()
        .filter(|f| f.code == "R-VISUAL-GAP")
        .collect();
    assert_eq!(gaps.len(), 21, "{gaps:#?}");
    assert!(
        gaps.iter()
            .all(|f| f.class == montagent_core::finding::Class::Note
                && f.fields["active"] == "visual"
                && f.fields["missing"] == "audio"),
        "every one is the ordinary direction — a photo outlasting a narration breath: {gaps:#?}"
    );
    assert!(
        gaps.iter().all(|f| f.fields["group"] != "header"),
        "the header group carries no audio member and is not evaluated: {gaps:#?}"
    );
}

#[test]
fn the_fixture_s_box_slack_is_the_seven_elements_adr_0058_enumerates() {
    // ADR-0058's own re-derivation, exactly: 7 of 22 text elements declare a `height`
    // that copies a card/chip/panel's rather than their own computed block height, and
    // the remaining 15 read exactly zero slack and never fire.
    if !common::has_ffprobe() {
        return;
    }
    let report = validate(&fixture());

    let mut findings: Vec<(&str, i64, i64, i64, i64)> = report
        .findings
        .iter()
        .filter(|f| f.code == "R-BOX-SLACK")
        .map(|f| {
            (
                f.location.element.as_deref().unwrap(),
                f.fields["declared_height"].as_i64().unwrap(),
                f.fields["computed_height"].as_i64().unwrap(),
                f.fields["slack"].as_i64().unwrap(),
                f.fields["slack_percent"].as_i64().unwrap(),
            )
        })
        .collect();
    findings.sort();

    assert_eq!(
        findings,
        [
            ("chip-text", 84, 58, 26, 45),
            ("handle-text", 84, 38, 46, 121),
            ("sentence-05", 169, 61, 108, 177),
            ("sentence-06", 169, 126, 43, 34),
            ("sentence-07", 169, 126, 43, 34),
            ("sentence-08", 169, 61, 108, 177),
            ("sentence-quiz", 169, 61, 108, 177),
        ],
        "{:#?}",
        report.findings
    );
    assert!(
        report
            .findings
            .iter()
            .filter(|f| f.code == "R-BOX-SLACK")
            .all(|f| f.class == montagent_core::finding::Class::Note && f.census.is_some()),
        "note-class, every one carrying a sibling census on `height`: {:?}",
        report.findings
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

#[test]
fn the_fixtures_two_same_source_cuts_are_the_ones_adr_0033_measured() {
    // ADR-0033's own table, asserted against the file it was measured on: six cuts on the
    // `photo` track, two of them same-source, `scale` resetting across both. The four
    // different-source resets are correct — a new image starting its own Ken Burns move —
    // and the discrimination is exactly one field wide.
    //
    // **64016, not 64816.** The ADR corrects ADR-0006's transposed digit, and this is the
    // assertion that keeps the correction from drifting back.
    if !common::has_ffprobe() {
        return;
    }
    let report = validate(&fixture());

    let mut pops: Vec<(i64, String, String)> = report
        .findings
        .iter()
        .filter(|f| f.code == "R-SOURCE-CUT-POP")
        .map(|f| {
            (
                f.fields["instant"].as_i64().unwrap(),
                f.fields["element"].as_str().unwrap().to_string(),
                f.fields["detail"].as_str().unwrap().to_string(),
            )
        })
        .collect();
    pops.sort();

    assert_eq!(pops.len(), 2, "{:?}", report.findings);
    assert_eq!(pops[0].0, 3018);
    assert_eq!(pops[0].1, "photo-05-intro");
    assert!(pops[0].2.contains("1.01610"), "{}", pops[0].2);
    assert_eq!(pops[1].0, 64016);
    assert_eq!(pops[1].1, "photo-05-quiz");
    assert!(pops[1].2.contains("1.05419"), "{}", pops[1].2);

    // Nothing else the motion-and-geometry ticket added fires here: every keyframe list is
    // a trimmed move whose target sits past its element (ADR-0012), every element is on
    // canvas, and every `ease` in the file describes real travel.
    assert!(
        report
            .findings
            .iter()
            .all(|f| f.code != "R-KEYFRAME-UNREACHED"
                && f.code != "R-OFF-CANVAS"
                && f.code != "R-EASE-INERT"),
        "{:?}",
        report.findings
    );
}
