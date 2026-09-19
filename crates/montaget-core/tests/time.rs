//! The structural time checks, and `slack` (#197).
//!
//! Every project here is built from `rect` elements or from `audio` elements with no
//! `source`: the questions are entirely about the document, so none of these tests needs
//! the `ffprobe` the disk half of `validate` wants.

use montaget_core::finding::{Class, Repair};
use montaget_core::report::{ExitCode, Report};
use montaget_core::{parse, slack, text, validate};
use std::path::PathBuf;

mod common;
use common::{canonical, write_project};

fn project(tracks: &str) -> String {
    canonical(&format!(
        r##"{{"frame":{{"width":1080,"height":1920}},"fps":25,"tracks":[{tracks}]}}"##
    ))
}

fn track(name: &str, layer: i64, elements: &str) -> String {
    format!(r##"{{"name":"{name}","layer":{layer},"elements":[{elements}]}}"##)
}

fn rect(id: &str, start: i64, end: i64) -> String {
    format!(
        r##"{{"id":"{id}","type":"rect","start":{start},"end":{end},"x":0,"y":0,"width":100,"height":100}}"##
    )
}

/// An audio element carrying a source range, which is what ADR-0020's invariant is about.
///
/// It carries no `source`, which the schema requires — so these projects are read by
/// [`speed_report`], which asks ADR-0020's check directly. The alternative is a real mp3
/// beside every one of them, which would put the disk half's own findings (and an
/// `ffprobe` requirement) in front of a question that is purely about the document. One
/// end-to-end test carries the wiring, against real media.
fn clip(id: &str, start: i64, end: i64, source_span: i64, extra: &str) -> String {
    format!(
        r##"{{"id":"{id}","type":"audio","start":{start},"end":{end},"source_start":0,"source_end":{source_span}{extra}}}"##
    )
}

#[track_caller]
fn report_on(tracks: &str) -> Report {
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(&dir, "p.montaget.json", &project(tracks));
    validate(&path)
}

#[track_caller]
fn codes(report: &Report) -> Vec<&str> {
    report.findings.iter().map(|f| f.code.as_str()).collect()
}

/// ADR-0020's invariant alone, over a project the schema would also have something to
/// say about.
#[track_caller]
fn speed_report(tracks: &str) -> Report {
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(&dir, "p.montaget.json", &project(tracks));
    let document = parse::read(&path).expect("parses");
    let mut report = Report::new("validate", Some(document.path().to_string()));
    montaget_core::checks::speed::check(&document, &mut report);
    report
}

fn render(report: &Report) -> String {
    text::render(&report.to_json(), text::Options::verbose()).expect("the report renders")
}

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/en-halloween-decorating/en-halloween-decorating.montaget.json")
}

// ---------------------------------------------------------------------------
// Overlap within a track is an error (ADR-0004).
// ---------------------------------------------------------------------------

#[test]
fn two_elements_of_one_track_sharing_an_instant_is_an_error() {
    let report = report_on(&track(
        "photo",
        10,
        &format!("{},{}", rect("a", 0, 3000), rect("b", 2000, 5000)),
    ));

    assert_eq!(codes(&report), ["E-TRACK-OVERLAP"]);
    assert_eq!(report.exit_code(), ExitCode::Errors);

    let finding = &report.findings[0];
    assert_eq!(finding.class, Class::Error);
    assert_eq!(finding.location.track.as_deref(), Some("photo"));
    assert_eq!(finding.fields["element"], "a");
    assert_eq!(finding.fields["other"], "b");
    assert_eq!(finding.fields["overlap"], 1000);

    // ADR-0043: which of the two is in the wrong place is not readable off the document,
    // so the check states no repair and no flag may lift that.
    assert_eq!(finding.repair, Some(Repair::None));

    let rendered = render(&report);
    assert!(rendered.contains("overlap by 1000 ms"), "{rendered}");
    assert!(rendered.contains("refuse-class"), "{rendered}");
}

#[test]
fn a_cut_is_a_single_instant_and_never_an_overlap() {
    // The must-not-fire. `CONTEXT.md`: the range is half-open, "so a cut where one range
    // ends and the next begins names a single instant, not an overlap and not a gap". The
    // committed fixture is made of these — every one of its 14 tracks cuts this way.
    let report = report_on(&track(
        "photo",
        10,
        &format!(
            "{},{},{}",
            rect("a", 0, 3018),
            rect("b", 3018, 17472),
            rect("c", 17472, 30603)
        ),
    ));

    assert!(report.findings.is_empty(), "{:?}", report.findings);
    assert_eq!(report.exit_code(), ExitCode::Ok);
}

#[test]
fn elements_that_overlap_in_two_tracks_are_exactly_what_tracks_are_for() {
    // The other must-not-fire, and ADR-0004's stated cost: "two elements that genuinely
    // should overlap now need two tracks. This is the constraint being bought deliberately."
    let report = report_on(&format!(
        "{},{}",
        track("photo", 10, &rect("a", 0, 3000)),
        track("caption", 20, &rect("b", 2000, 5000)),
    ));

    assert!(report.findings.is_empty(), "{:?}", report.findings);
}

#[test]
fn an_overlap_between_two_elements_that_are_not_adjacent_in_time_order_is_found() {
    // A scan of adjacent pairs would miss this: sorted by `start`, the long element is
    // first, the short one inside it is second, and the third does not overlap its
    // predecessor at all — but it does overlap the first.
    let report = report_on(&track(
        "photo",
        10,
        &format!(
            "{},{},{}",
            rect("long", 0, 10000),
            rect("inside", 1000, 2000),
            rect("later", 3000, 4000)
        ),
    ));

    let pairs: Vec<(&str, &str)> = report
        .findings
        .iter()
        .map(|f| {
            (
                f.fields["element"].as_str().unwrap(),
                f.fields["other"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(pairs, [("long", "inside"), ("long", "later")]);
}

#[test]
fn array_order_is_not_time_order() {
    // ADR-0060: "array order carries no meaning, for timing or for stacking". The overlap
    // is the same overlap whichever way round the two are written.
    let forwards = report_on(&track(
        "photo",
        10,
        &format!("{},{}", rect("a", 0, 3000), rect("b", 2000, 5000)),
    ));
    let backwards = report_on(&track(
        "photo",
        10,
        &format!("{},{}", rect("b", 2000, 5000), rect("a", 0, 3000)),
    ));

    assert_eq!(codes(&forwards), codes(&backwards));
    assert_eq!(
        forwards.findings[0].fields, backwards.findings[0].fields,
        "and it reads the same way round either way"
    );
}

// ---------------------------------------------------------------------------
// Gaps are reported separately, and never as errors (ADR-0004, ADR-0006).
// ---------------------------------------------------------------------------

#[test]
fn a_gap_is_reported_and_is_never_an_error() {
    let report = report_on(&track(
        "narration",
        0,
        &format!(
            "{},{}",
            rect("vo-intro", 0, 2568),
            rect("vo-hook", 3018, 4866)
        ),
    ));

    assert_eq!(codes(&report), ["N-TRACK-GAP"]);
    let finding = &report.findings[0];
    assert_eq!(finding.class, Class::Note);
    assert_eq!(finding.repair, None, "`repair` is an axis of `error` alone");
    assert_eq!(
        report.exit_code(),
        ExitCode::Ok,
        "a gap never refuses a render"
    );

    assert_eq!(finding.fields["from"], 2568);
    assert_eq!(finding.fields["to"], 3018);
    assert_eq!(finding.fields["size"], 450);
    assert_eq!(finding.fields["after"], "vo-intro");
    assert_eq!(finding.fields["before"], "vo-hook");

    let rendered = render(&report);
    assert!(rendered.contains("2568 ms to 3018 ms"), "{rendered}");
}

#[test]
fn an_overlap_and_a_gap_in_one_track_are_two_findings_and_not_one() {
    // ADR-0004's stated consequence, verbatim: the validator "must distinguish *overlap*
    // from *gap*: a forgotten shift leaving a silent gap passes an overlap-only check."
    let report = report_on(&track(
        "photo",
        10,
        &format!(
            "{},{},{}",
            rect("a", 0, 3000),
            rect("b", 2000, 5000),
            rect("c", 8000, 9000)
        ),
    ));

    assert_eq!(codes(&report), ["E-TRACK-OVERLAP", "N-TRACK-GAP"]);
    assert_eq!(report.findings[1].fields["from"], 5000);
    assert_eq!(report.findings[1].fields["after"], "b");
}

#[test]
fn an_element_nested_inside_a_longer_one_does_not_manufacture_a_gap() {
    // The overlap is real; the "gap" after the short element is not, because the long one
    // is still covering it. A check reading the previous element rather than the furthest
    // instant reached would report one.
    let report = report_on(&track(
        "photo",
        10,
        &format!("{},{}", rect("long", 0, 10000), rect("inside", 1000, 2000)),
    ));

    assert_eq!(codes(&report), ["E-TRACK-OVERLAP"]);
}

#[test]
fn a_track_that_starts_late_or_ends_early_has_no_gap_in_it() {
    // A gap is bounded by two elements *of this track*. A single element inside a longer
    // project has not had black frames punched into its track; the distance from its end
    // to the project's is slack, and it has a different owner.
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(
        &dir,
        "p.montaget.json",
        &canonical(&format!(
            r##"{{"frame":{{"width":1080,"height":1920}},"fps":25,"duration":60000,"tracks":[{}]}}"##,
            track("chip", 30, &rect("chip-panel", 5000, 10000))
        )),
    );

    let report = validate(&path);
    assert!(report.findings.is_empty(), "{:?}", report.findings);
}

// ---------------------------------------------------------------------------
// The `speed` invariant, in exact arithmetic (ADR-0020, ADR-0045).
// ---------------------------------------------------------------------------

#[test]
fn the_invariant_is_evaluated_where_ieee_double_diverges() {
    // ADR-0045's minimal case, and the whole reason the ADR exists: "`7 / 0.560` is
    // exactly `12.5` in decimal, so round-half-up gives `13`, but `f64` computes
    // `12.499999999999998`, giving `12`."
    //
    // A timeline span of 13 is therefore legal and 12 is not — and an implementation that
    // divided in `f64` would have these two tests exactly the wrong way round.
    let exact = speed_report(&track(
        "vo",
        0,
        &clip("vo-01", 0, 13, 7, r##","speed":0.560"##),
    ));
    assert!(
        exact.findings.is_empty(),
        "13 is the exact answer: {:?}",
        exact.findings
    );

    let ieee = speed_report(&track(
        "vo",
        0,
        &clip("vo-01", 0, 12, 7, r##","speed":0.560"##),
    ));
    assert_eq!(
        codes(&ieee),
        ["E-SPEED-MISMATCH"],
        "12 is the `f64` answer, and it is wrong"
    );
    assert_eq!(ieee.findings[0].fields["played"], 13);
    assert_eq!(ieee.findings[0].fields["timeline_span"], 12);
}

#[test]
fn the_fixtures_own_speed_elements_are_legal() {
    // ADR-0045: "All four `speed` elements in the committed fixture ... share `speed:
    // 0.645` ... None sits on a round-half-up tie", so this ADR "changes zero bytes of the
    // committed project file". `2184 / 0.645 = 3386.0465...`, which rounds to 3386 —
    // exactly `vo-sentence-05-b`'s committed `16558 - 13172` (ADR-0020).
    let report = speed_report(&track(
        "vo",
        0,
        &format!(
            "{},{},{}",
            clip("vo-05-b", 13172, 16558, 2184, r##","speed":0.645"##),
            clip("vo-06-b", 16558, 20539, 2568, r##","speed":0.645"##),
            clip("vo-08-b", 20539, 23627, 1992, r##","speed":0.645"##),
        ),
    ));

    assert!(report.findings.is_empty(), "{:?}", report.findings);
}

#[test]
fn an_absent_speed_is_the_equality_adr_0005_stated() {
    let legal = speed_report(&track("vo", 0, &clip("vo-01", 0, 2568, 2568, "")));
    assert!(legal.findings.is_empty(), "{:?}", legal.findings);

    let stale = speed_report(&track("vo", 0, &clip("vo-01", 0, 2568, 3368, "")));
    assert_eq!(codes(&stale), ["E-SPEED-MISMATCH"]);
}

#[test]
fn the_mismatch_states_the_corrective_speed_and_it_actually_corrects() {
    // ADR-0020: "`speed` is the free variable: when the invariant fails, `validate`'s
    // error reports the `speed` value that would satisfy it, and the agent edits `speed`,
    // never the spans." So the advise-class repair is a `speed`, and feeding it back
    // through the invariant has to make the project legal.
    let report = speed_report(&track(
        "vo",
        0,
        &clip("vo-01", 0, 3981, 2568, r##","speed":0.9"##),
    ));

    assert_eq!(codes(&report), ["E-SPEED-MISMATCH"]);
    let Some(Repair::Advise(repair)) = &report.findings[0].repair else {
        panic!("advise-class: {:?}", report.findings[0]);
    };
    let stated = repair["value"].as_str().unwrap();
    let corrective = stated
        .rsplit_once(' ')
        .map(|(_, value)| value.to_string())
        .expect("the repair names a value");

    let repaired = speed_report(&track(
        "vo",
        0,
        &clip(
            "vo-01",
            0,
            3981,
            2568,
            &format!(r##","speed":{corrective}"##),
        ),
    ));
    assert!(
        repaired.findings.is_empty(),
        "the stated repair must repair: {stated} left {:?}",
        repaired.findings
    );
}

#[test]
fn a_speed_the_document_cannot_be_read_as_a_rate_is_left_to_the_schema() {
    // ADR-0020 makes `0`, a negative and a non-number schema errors, and the check that
    // owns the schema says so. This one must not quietly read any of them as `1.0`.
    for spelling in ["0", "-0.5", "\"0.645\"", "null"] {
        let report = speed_report(&track(
            "vo",
            0,
            &clip("vo-01", 0, 12, 7, &format!(r##","speed":{spelling}"##)),
        ));
        assert!(
            !codes(&report).contains(&"E-SPEED-MISMATCH"),
            "speed {spelling}: {:?}",
            report.findings
        );
    }
}

#[test]
fn overrun_turns_the_equality_into_an_inequality() {
    // ADR-0020: "without `overrun`, `end - start` must equal `round(source_span / speed)`
    // exactly; with `overrun` declared, `end - start` must be strictly *greater than* that
    // value."
    let holding = speed_report(&track(
        "b-roll",
        5,
        &clip("shot", 0, 5000, 2568, r##","overrun":"hold""##),
    ));
    assert!(holding.findings.is_empty(), "{:?}", holding.findings);
}

#[test]
fn an_overrun_with_nothing_to_cover_is_its_own_error_and_states_no_repair() {
    // "An `overrun` declared on an element that doesn't need it — where `end - start`
    // doesn't exceed the played duration — is itself a `validate` error" (ADR-0020). Its
    // own code, because the repair goes the other way: deleting the key and extending the
    // element are different videos, and the document does not say which was meant.
    let report = speed_report(&track(
        "b-roll",
        5,
        &clip("shot", 0, 2568, 2568, r##","overrun":"loop""##),
    ));

    assert_eq!(codes(&report), ["E-OVERRUN-UNNEEDED"]);
    assert_eq!(report.findings[0].repair, Some(Repair::None));
    assert_eq!(report.findings[0].fields["overrun"], "loop");
    assert!(render(&report).contains("nothing past the source to cover"));
}

#[test]
fn the_invariant_reaches_the_report_through_validate() {
    // The wiring, once, against real media: every other test in this section asks
    // ADR-0020's check directly, and a check nothing calls would pass all of them.
    //
    // `05-cobweb.mp3` is 1776 ms and the committed fixture declares exactly that on four
    // elements, so the source range here is one the published project also depends on —
    // and it does not overrun, which keeps the disk half quiet and this finding alone.
    if !common::has_ffprobe() {
        return;
    }
    let dir = common::tempdir(std::panic::Location::caller().line());
    std::fs::copy(
        fixture().parent().unwrap().join("audio/05-cobweb.mp3"),
        dir.join("cobweb.mp3"),
    )
    .expect("copy real media beside the project");
    let path = write_project(
        &dir,
        "p.montaget.json",
        &project(&track(
            "narration",
            0,
            r##"{"id":"vo-01","type":"audio","start":0,"end":1000,"source":"cobweb.mp3","source_start":0,"source_end":1776}"##,
        )),
    );

    let report = validate(&path);

    assert_eq!(codes(&report), ["E-SPEED-MISMATCH"]);
    assert_eq!(report.findings[0].fields["played"], 1776);
    assert_eq!(report.exit_code(), ExitCode::Errors);
}

// ---------------------------------------------------------------------------
// Quantization reports what it changes (ADR-0006, ADR-0035).
// ---------------------------------------------------------------------------

#[test]
fn nothing_is_said_about_boundaries_that_are_merely_off_the_grid() {
    // ADR-0006: "109 of its 120 time values — 47 of 48 distinct instants — are off the
    // 40 ms grid at the project's own 25 fps ... A check with a 98% hit rate on a correct,
    // published project is not a check; it is the alarm fatigue this ADR already identified
    // as a safety problem." Every boundary here is off the grid and nothing is reported.
    let report = report_on(&track(
        "photo",
        10,
        &format!(
            "{},{},{}",
            rect("a", 1, 3018),
            rect("b", 3018, 17472),
            rect("c", 20001, 30603)
        ),
    ));

    assert_eq!(
        codes(&report),
        ["N-TRACK-GAP"],
        "the gap is reported; the six off-grid boundaries are not"
    );
}

#[test]
fn an_element_no_sampled_frame_falls_inside_is_what_quantization_changes() {
    // ADR-0006's first derived fact: "No element is shorter than a frame (the shortest is
    // 1000 ms), so nothing can round out of existence." Here one is.
    let report = report_on(&track("photo", 10, &rect("blink", 1001, 1020)));

    assert_eq!(codes(&report), ["N-QUANTIZATION"]);
    let finding = &report.findings[0];
    // ADR-0006: "Escalate to `review` only for the cases in (1) and (2)" — and this is
    // case (1) verbatim. The `N-` prefix is not the class: "the prefix is a convention and
    // not a rule" (`CONTEXT.md`).
    assert_eq!(finding.class, Class::Review);
    assert_eq!(finding.fields["fps"], 25);
    assert_eq!(finding.fields["changed"], 2, "its two boundaries");
    assert!(
        finding.fields["detail"]
            .as_str()
            .unwrap()
            .contains("`blink` (1001..1020 ms) holds no sampled frame"),
        "{finding:?}"
    );
}

#[test]
fn a_gap_no_sampled_frame_falls_inside_is_the_other_half() {
    // The black frames the gap declares are not there. ADR-0006's second derived fact is
    // that the fixture has none of these either — its shortest gap is 450 ms.
    let report = report_on(&track(
        "photo",
        10,
        &format!("{},{}", rect("a", 0, 1001), rect("b", 1020, 3000)),
    ));

    assert_eq!(codes(&report), ["N-TRACK-GAP", "N-QUANTIZATION"]);
    assert!(
        report.findings[1].fields["detail"]
            .as_str()
            .unwrap()
            .contains("19 ms gap"),
        "{:?}",
        report.findings[1]
    );
}

#[test]
fn the_grid_is_fps_dependent_and_not_the_intuitive_step() {
    // ADR-0035: "at 30fps only multiples of 100ms are frame-exact, not the more intuitive
    // 33.33ms, a fact one agent found only because it had written its own checker." A
    // 20 ms element at 1 ms survives at 30 fps (the frame at 100/3 ms is not inside it —
    // but the one at 0 is not either, and ceil puts the first candidate at frame 1)…
    let dir = common::tempdir(std::panic::Location::caller().line());
    let at_30 = write_project(
        &dir,
        "p30.montaget.json",
        &canonical(&format!(
            r##"{{"frame":{{"width":1080,"height":1920}},"fps":30,"tracks":[{}]}}"##,
            track("photo", 10, &rect("blink", 1, 35))
        )),
    );
    assert!(
        validate(&at_30).findings.is_empty(),
        "the frame at 100/3 ms falls inside 1..35"
    );

    let dir = common::tempdir(std::panic::Location::caller().line() + 1);
    let narrower = write_project(
        &dir,
        "p30.montaget.json",
        &canonical(&format!(
            r##"{{"frame":{{"width":1080,"height":1920}},"fps":30,"tracks":[{}]}}"##,
            track("photo", 10, &rect("blink", 34, 35))
        )),
    );
    assert_eq!(
        codes(&validate(&narrower)),
        ["N-QUANTIZATION"],
        "and 34..35 sits between two of them"
    );
}

#[test]
fn quantization_says_nothing_about_the_committed_fixture() {
    // The acceptance criterion, and ADR-0006's own answer: "on the fixture the honest
    // answer is one line — *nothing changes*". Asked of the check directly, so the
    // assertion does not depend on having an `ffprobe`.
    let document = parse::read(&fixture()).expect("the fixture parses");
    let mut report = Report::new("validate", Some(document.path().to_string()));

    montaget_core::checks::quantization::check(&document, &mut report);

    assert!(report.findings.is_empty(), "{:?}", report.findings);
}

// ---------------------------------------------------------------------------
// Slack: one derivation, two consumers (ADR-0032, ADR-0047).
// ---------------------------------------------------------------------------

#[test]
fn slack_is_the_distance_to_the_nearest_boundary_in_any_track() {
    // `CONTEXT.md`: "Every gap is slack; slack additionally names the cross-track case a
    // gap can't reach, such as the distance from the last narration's end to a still
    // photo's end." The 4 ms below is exactly that case: two boundaries in two different
    // tracks, which no gap in either track reaches.
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(
        &dir,
        "p.montaget.json",
        &canonical(&format!(
            r##"{{"frame":{{"width":1080,"height":1920}},"fps":25,"duration":62000,"tracks":[{},{}]}}"##,
            track("narration", 0, &rect("vo-quiz", 50000, 56112)),
            track("photo", 10, &rect("photo-quiz", 50000, 56116)),
        )),
    );
    let document = parse::read(&path).expect("parses");

    let slacks = slack::of(&document);
    let sizes: Vec<(i64, i64, i64)> = slacks.iter().map(|s| (s.from, s.to, s.size())).collect();
    assert_eq!(
        sizes,
        [
            (50000, 56112, 6112),
            (56112, 56116, 4),
            (56116, 62000, 5884)
        ]
    );

    // The 4 ms one is named by both its ends, which is ADR-0047's identity.
    let cross_track = slack::at(&slacks, 56112, 56116).expect("the pair names a real slack");
    assert_eq!(
        cross_track.from_edges[0].element, "vo-quiz",
        "the narration's end opens it"
    );
    assert_eq!(
        cross_track.to_edges[0].element, "photo-quiz",
        "and the photo's end closes it"
    );
    assert_eq!(
        cross_track.to_edges[0].track,
        Some("photo"),
        "each end names its track, because `shift`'s refusal does (ADR-0032)"
    );
    assert_eq!(cross_track.to_edges[0].side, slack::Side::End);
    assert!(!cross_track.to_duration);

    // And the project's own last boundary is the derived `duration`, with nothing on its
    // far end to point at.
    let lead_out = slack::at(&slacks, 56116, 62000).expect("the tail is a slack too");
    assert!(lead_out.to_duration);
    assert!(lead_out.to_edges.is_empty());

    // A pair the document does not currently bound is not a slack — ADR-0047 requires
    // `shift --release` to refuse exactly this.
    assert!(slack::at(&slacks, 56112, 62000).is_none());
}

#[test]
fn an_element_stating_only_half_a_range_contributes_no_boundary() {
    // Both or neither. An element mid-edit carrying a `start` and no `end` states no
    // range, and one boundary of a range that is not there would manufacture a slack out
    // of a half-written element. `crate::track` holds the same rule, and the two agreeing
    // is the point — spec #168 names slack as the quantity two verbs would otherwise
    // define twice.
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(
        &dir,
        "p.montaget.json",
        &canonical(&format!(
            r##"{{"frame":{{"width":1080,"height":1920}},"fps":25,"duration":9000,"tracks":[{}]}}"##,
            track(
                "photo",
                10,
                &format!(
                    "{},{}",
                    rect("whole", 0, 3000),
                    r##"{"id":"half","type":"rect","start":5000,"x":0,"y":0,"width":10,"height":10}"##
                )
            )
        )),
    );
    let document = parse::read(&path).expect("parses");

    assert_eq!(
        slack::of(&document)
            .iter()
            .map(|s| (s.from, s.to))
            .collect::<Vec<_>>(),
        [(0, 3000), (3000, 9000)],
        "5000 is not a boundary, so no slack is measured to it"
    );
}

#[test]
fn a_cut_is_not_a_zero_width_slack() {
    // One instant, not two boundaries: "a cut where one range ends and the next begins
    // names a single instant" (`CONTEXT.md`). A zero-width slack is not a distance any
    // edit could release, and a derivation that produced one would hand `shift` a refusal
    // to print at every cut in the project.
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(
        &dir,
        "p.montaget.json",
        &project(&track(
            "photo",
            10,
            &format!("{},{}", rect("a", 0, 3018), rect("b", 3018, 17472)),
        )),
    );
    let document = parse::read(&path).expect("parses");

    let slacks = slack::of(&document);
    assert!(slacks.iter().all(|s| s.size() > 0));
    assert_eq!(
        slacks.iter().map(|s| (s.from, s.to)).collect::<Vec<_>>(),
        [(0, 3018), (3018, 17472)]
    );
}

#[test]
fn every_slack_in_the_committed_fixture_is_derived_from_one_function() {
    // The acceptance criterion behind the derivation's existence: `shift` (#220) and
    // `compare` (#221) both consume this, and spec #168 names the hazard — slack is "a
    // derived quantity that stories 68 and 72 both consume and neither defines".
    let document = parse::read(&fixture()).expect("the fixture parses");

    let slacks = slack::of(&document);

    // 47 distinct boundary instants, `duration` among them, so 46 adjacent pairs.
    assert_eq!(slacks.len(), 46);
    assert!(
        slacks.windows(2).all(|w| w[0].to == w[1].from),
        "contiguous"
    );
    assert_eq!(
        slacks.iter().map(|s| s.size()).min(),
        Some(4),
        "the smallest is cross-track and no gap reaches it"
    );
    assert!(
        slacks.iter().filter(|s| s.to_duration).count() == 1,
        "exactly one slack ends at the project's own last boundary"
    );

    // Every gap the gap check reports is also a slack — "every gap is slack" — but not
    // the other way round.
    let gap_count = validate_document_gaps(&document);
    assert!(
        gap_count < slacks.len(),
        "{gap_count} gaps, {} slacks",
        slacks.len()
    );
}

fn validate_document_gaps(document: &montaget_core::permissive::Loose) -> usize {
    let mut report = Report::new("validate", Some(document.path().to_string()));
    montaget_core::checks::track::check(document, &mut report);
    report
        .findings
        .iter()
        .filter(|f| f.code == "N-TRACK-GAP")
        .count()
}

// ---------------------------------------------------------------------------
// The declarations these checks fire through.
// ---------------------------------------------------------------------------

#[test]
fn the_registry_declares_every_code_these_checks_can_fire() {
    // A check emitting an unregistered code panics in `Finding::new`, which is a bug in
    // the check rather than a condition of the project. And a code the table still calls
    // `Declared` is one a reader would take for unimplemented.
    use montaget_core::registry::{self, RepairClass, Status};

    for (code, classes, repair) in [
        (
            "E-TRACK-OVERLAP",
            &[Class::Error][..],
            Some(RepairClass::Refuse),
        ),
        ("N-TRACK-GAP", &[Class::Note][..], None),
        (
            "E-SPEED-MISMATCH",
            &[Class::Error][..],
            Some(RepairClass::Advise),
        ),
        (
            "E-OVERRUN-UNNEEDED",
            &[Class::Error][..],
            Some(RepairClass::Refuse),
        ),
        ("N-QUANTIZATION", &[Class::Review, Class::Note][..], None),
    ] {
        let spec = registry::spec(code).unwrap_or_else(|| panic!("{code} is not registered"));
        assert_eq!(spec.classes, classes, "{code}");
        assert_eq!(spec.repair, repair, "{code}");
        assert_eq!(spec.status, Status::Live, "{code}");
    }
}

#[test]
fn a_gap_is_never_declared_as_able_to_be_an_error() {
    // `CONTEXT.md` and ADR-0006 both say it outright, and the registry is where that
    // becomes unrepresentable rather than remembered: "Gaps are never errors ... restated
    // because the severity rule above could be misread as overturning it."
    let gap = montaget_core::registry::spec("N-TRACK-GAP").unwrap();
    assert!(!gap.may_error());
}
