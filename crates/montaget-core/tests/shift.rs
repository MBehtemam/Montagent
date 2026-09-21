//! `shift` (#220): the one edit that is arithmetic rather than authorship.

use montaget_core::report::ExitCode;
use montaget_core::verbs::shift::{Ask, shift};

mod common;
use common::{canonical, write_project};

fn ask(at: i64, delta: i64) -> Ask {
    Ask {
        at,
        delta,
        scope: None,
        release: Vec::new(),
    }
}

fn codes(report: &montaget_core::report::Report) -> Vec<&str> {
    report.findings.iter().map(|f| f.code.as_str()).collect()
}

/// Error-class codes alone — a project can legitimately carry informational findings
/// (e.g. `N-TRACK-GAP` on a deliberate gap) that these tests are not about.
fn error_codes(report: &montaget_core::report::Report) -> Vec<&str> {
    report
        .findings
        .iter()
        .filter(|f| f.class == montaget_core::finding::Class::Error)
        .map(|f| f.code.as_str())
        .collect()
}

// ---------------------------------------------------------------------------
// A plain shift moves everything at or after `at`.
// ---------------------------------------------------------------------------

#[test]
fn a_plain_shift_moves_everything_at_or_after_at_and_writes_clean() {
    let dir = common::tempdir(line!());
    let path = write_project(
        &dir,
        "p.json",
        &canonical(
            r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[{"name":"a","layer":1,"elements":[{"id":"r1","type":"rect","start":500,"end":1000,"width":100,"height":100,"fill":"#FF0000"}]}]}"##,
        ),
    );

    let answer = shift(&path, &ask(200, 300));

    assert_eq!(
        answer.report().exit_code(),
        ExitCode::Ok,
        "{:?}",
        codes(answer.report())
    );
    assert_eq!(answer.report().tool, "shift");

    let written: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let element = &written["tracks"][0]["elements"][0];
    assert_eq!(element["start"], 800);
    assert_eq!(element["end"], 1300);
}

// ---------------------------------------------------------------------------
// The write-tool invariant: the new state's findings, never `ok`.
// ---------------------------------------------------------------------------

#[test]
fn it_returns_the_new_states_findings_not_ok() {
    let dir = common::tempdir(line!());
    let path = write_project(
        &dir,
        "p.json",
        &canonical(
            r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[{"name":"a","layer":1,"elements":[{"id":"r1","type":"rect","start":0,"end":500,"width":100,"height":100,"fill":"#FF0000"}]}]}"##,
        ),
    );

    let answer = shift(&path, &ask(600, 100));

    // The report is `validate`'s own shape — a summary, an exit code, `not_checked` — not
    // a bespoke "ok" sentinel: this is what makes "then I don't run `validate`; `validate`
    // runs me" true rather than aspirational.
    let json = answer.report().to_json();
    assert!(json.get("not_checked").is_some());
    assert_eq!(json["summary"]["error"], 0);
    assert_eq!(answer.report().tool, "shift");
}

// ---------------------------------------------------------------------------
// A straddling time-based element is refused, naming the nearest legal boundaries.
// ---------------------------------------------------------------------------

#[test]
fn a_straddling_time_based_element_is_refused_naming_the_nearest_legal_boundaries() {
    if !common::has_ffprobe() {
        return;
    }
    let dir = common::tempdir(line!());
    std::fs::copy(
        common::fixture_dir().join("audio/05-cobweb.mp3"),
        dir.join("cobweb.mp3"),
    )
    .expect("copy real media beside the project");
    let path = write_project(
        &dir,
        "p.json",
        &canonical(
            r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[{"name":"vo","layer":1,"elements":[{"id":"vo-01","type":"audio","start":0,"end":1776,"source":"cobweb.mp3","source_start":0,"source_end":1776}]}]}"##,
        ),
    );
    let before = std::fs::read_to_string(&path).unwrap();

    let answer = shift(&path, &ask(800, 100));

    assert_eq!(answer.report().exit_code(), ExitCode::Errors);
    assert_eq!(codes(answer.report()), vec!["E-SHIFT-STRADDLE"]);
    let finding = &answer.report().findings[0];
    assert_eq!(finding.location.element.as_deref(), Some("vo-01"));
    assert_eq!(finding.fields["start"], 0);
    assert_eq!(finding.fields["end"], 1776);
    // A refused edit leaves the file exactly as it was.
    assert_eq!(std::fs::read_to_string(&path).unwrap(), before);
}

// ---------------------------------------------------------------------------
// Keyframes are carried with their element, not dragged by the at-or-after rule: a shot
// that finished before the insert point does not move.
// ---------------------------------------------------------------------------

#[test]
fn keyframes_move_with_their_element_a_shot_ending_before_the_insert_point_does_not_move() {
    let dir = common::tempdir(line!());
    // `r1` finishes at 1000, well before `at` — but its `scale` keyframe sits at 1500,
    // past its own end (a trimmed move). ADR-0012's table: `end <= at` is untouched,
    // "including any past `end`".
    let path = write_project(
        &dir,
        "p.json",
        &canonical(
            r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[{"name":"a","layer":1,"elements":[{"id":"r1","type":"rect","start":0,"end":1000,"width":100,"height":100,"fill":"#FF0000","scale":[{"t":1500,"v":[1.08,1.08]}]}]}]}"##,
        ),
    );

    let answer = shift(&path, &ask(1200, 500));

    assert_eq!(
        answer.report().exit_code(),
        ExitCode::Ok,
        "{:?}",
        codes(answer.report())
    );
    let written: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let element = &written["tracks"][0]["elements"][0];
    assert_eq!(element["start"], 0);
    assert_eq!(element["end"], 1000);
    // Not dragged to 2000: a timestamp-global rule would change a shot that finished
    // before the edit point, which is exactly what ADR-0012 forbids.
    assert_eq!(element["scale"][0]["t"], 1500);
}

/// A straddler's keyframes are SPLIT rather than uniformly dragged, and the value at
/// `at` is preserved exactly outside `[at, at+delta)`.
#[test]
fn a_time_invariant_straddlers_keyframes_are_split_not_dragged() {
    let dir = common::tempdir(line!());
    // `r1` runs the whole window; `at` cuts it mid-flight, inside its two keyframes.
    let path = write_project(
        &dir,
        "p.json",
        &canonical(
            r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[{"name":"a","layer":1,"elements":[{"id":"r1","type":"rect","start":0,"end":1000,"width":100,"height":100,"fill":"#FF0000","opacity":[{"t":0,"v":0.0},{"t":1000,"v":1.0,"ease":"linear"}]}]}]}"##,
        ),
    );

    let answer = shift(&path, &ask(400, 100));

    assert_eq!(
        answer.report().exit_code(),
        ExitCode::Ok,
        "{:?}",
        codes(answer.report())
    );
    let written: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let element = &written["tracks"][0]["elements"][0];
    assert_eq!(element["end"], 1100);
    let opacity = element["opacity"].as_array().unwrap();
    // 0, then the cut (v=0.4 at t=400, linear), the delta-long hold, then the shifted tail.
    assert_eq!(opacity.len(), 4);
    assert_eq!(opacity[0]["t"], 0);
    assert_eq!(opacity[1]["t"], 400);
    assert!((opacity[1]["v"].as_f64().unwrap() - 0.4).abs() < 1e-9);
    assert_eq!(opacity[2]["t"], 500);
    assert!((opacity[2]["v"].as_f64().unwrap() - 0.4).abs() < 1e-9);
    assert_eq!(opacity[2]["ease"], "step");
    assert_eq!(opacity[3]["t"], 1100);
    assert_eq!(opacity[3]["v"], 1.0);
}

// ---------------------------------------------------------------------------
// The coincident-instant preamble prints unconditionally.
// ---------------------------------------------------------------------------

#[test]
fn the_coincident_instant_preamble_prints_with_no_flag() {
    let dir = common::tempdir(line!());
    let path = write_project(
        &dir,
        "p.json",
        &canonical(
            r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[{"name":"a","layer":1,"elements":[{"id":"r1","type":"rect","start":1000,"end":2000,"width":100,"height":100,"fill":"#FF0000"}]}]}"##,
        ),
    );

    let answer = shift(&path, &ask(1000, 100));

    let json = answer.report().to_json();
    let _ = json;
    let full = answer.to_json();
    let coincident = full["shift"]["coincident"].as_array().unwrap();
    assert_eq!(coincident.len(), 1);
    assert_eq!(coincident[0]["element"], "r1");
    assert_eq!(coincident[0]["kind"], "boundary");
    assert_eq!(coincident[0]["role"], "start");
}

#[test]
fn a_shift_whose_at_lands_nowhere_reports_an_empty_preamble() {
    let dir = common::tempdir(line!());
    let path = write_project(
        &dir,
        "p.json",
        &canonical(
            r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[{"name":"a","layer":1,"elements":[{"id":"r1","type":"rect","start":1000,"end":2000,"width":100,"height":100,"fill":"#FF0000"}]}]}"##,
        ),
    );

    let answer = shift(&path, &ask(1500, 100));

    let full = answer.to_json();
    assert_eq!(full["shift"]["coincident"].as_array().unwrap().len(), 0);
}

// ---------------------------------------------------------------------------
// Slack is invariant by default; `shift` refuses to silently change it, and `release`
// consumes exactly the pairs a refusal reported.
// ---------------------------------------------------------------------------

fn gap_project() -> String {
    canonical(
        r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[{"name":"a","layer":1,"elements":[{"id":"a1","type":"rect","start":0,"end":500,"width":100,"height":100,"fill":"#FF0000"},{"id":"a2","type":"rect","start":600,"end":1000,"width":100,"height":100,"fill":"#00FF00"}]}]}"##,
    )
}

#[test]
fn an_edit_that_would_change_a_slacks_size_is_refused() {
    let dir = common::tempdir(line!());
    let path = write_project(&dir, "p.json", &gap_project());
    let before = std::fs::read_to_string(&path).unwrap();

    // 550 sits inside the 100 ms gap between `a1` and `a2`; nothing straddles it, but
    // widening the gap by 200 ms changes the slack between them.
    let answer = shift(&path, &ask(550, 200));

    assert_eq!(answer.report().exit_code(), ExitCode::Errors);
    assert_eq!(error_codes(answer.report()), vec!["E-SHIFT-SLACK"]);
    let finding = answer
        .report()
        .findings
        .iter()
        .find(|f| f.code == "E-SHIFT-SLACK")
        .unwrap();
    assert_eq!(finding.fields["from"], 500);
    assert_eq!(finding.fields["to"], 600);
    assert_eq!(finding.fields["size"], 100);
    assert_eq!(finding.fields["new_size"], 300);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), before);
}

#[test]
fn releasing_exactly_the_reported_pair_lets_the_edit_through() {
    let dir = common::tempdir(line!());
    let path = write_project(&dir, "p.json", &gap_project());

    let answer = shift(
        &path,
        &Ask {
            at: 550,
            delta: 200,
            scope: None,
            release: vec![(500, 600)],
        },
    );

    assert_eq!(
        answer.report().exit_code(),
        ExitCode::Ok,
        "{:?}",
        codes(answer.report())
    );
    let written: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let elements = written["tracks"][0]["elements"].as_array().unwrap();
    assert_eq!(elements[0]["end"], 500);
    assert_eq!(elements[1]["start"], 800);
    assert_eq!(elements[1]["end"], 1200);
}

/// Two back-to-back elements sharing one boundary instant — `a1` ending exactly where
/// `a2` starts — is "the ordinary shape of a tightly packed timeline" (ADR-0047), and a
/// shift that moves the later one and leaves the earlier one alone must not spuriously
/// flag either element's own span as a changed slack. Declared `a2` before `a1`, so a
/// reader who picked *any* edge at the shared instant rather than the one that actually
/// bounds each slack would get a different, wrong answer depending on this order alone.
#[test]
fn back_to_back_elements_at_the_shift_point_are_not_a_false_positive() {
    let dir = common::tempdir(line!());
    let path = write_project(
        &dir,
        "p.json",
        &canonical(
            r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[{"name":"a","layer":1,"elements":[{"id":"a2","type":"rect","start":1000,"end":2000,"width":100,"height":100,"fill":"#00FF00"},{"id":"a1","type":"rect","start":0,"end":1000,"width":100,"height":100,"fill":"#FF0000"}]}]}"##,
        ),
    );

    let answer = shift(&path, &ask(1000, 100));

    assert_eq!(
        answer.report().exit_code(),
        ExitCode::Ok,
        "{:?}",
        codes(answer.report())
    );
    let written: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let elements = written["tracks"][0]["elements"].as_array().unwrap();
    let by_id = |id: &str| elements.iter().find(|e| e["id"] == id).unwrap();
    assert_eq!(by_id("a1")["start"], 0);
    assert_eq!(by_id("a1")["end"], 1000);
    assert_eq!(by_id("a2")["start"], 1100);
    assert_eq!(by_id("a2")["end"], 2100);
}

#[test]
fn a_release_pair_that_does_not_bound_a_threatened_slack_is_itself_refused() {
    let dir = common::tempdir(line!());
    let path = write_project(&dir, "p.json", &gap_project());
    let before = std::fs::read_to_string(&path).unwrap();

    let answer = shift(
        &path,
        &Ask {
            at: 550,
            delta: 200,
            scope: None,
            // The real threat, plus a pair this edit never touches.
            release: vec![(500, 600), (9000, 9500)],
        },
    );

    assert_eq!(answer.report().exit_code(), ExitCode::Errors);
    assert_eq!(
        error_codes(answer.report()),
        vec!["E-SHIFT-RELEASE-INVALID"]
    );
    let finding = answer
        .report()
        .findings
        .iter()
        .find(|f| f.code == "E-SHIFT-RELEASE-INVALID")
        .unwrap();
    assert_eq!(finding.fields["from"], 9000);
    assert_eq!(finding.fields["to"], 9500);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), before);
}

// ---------------------------------------------------------------------------
// `scope` narrows which elements move.
// ---------------------------------------------------------------------------

#[test]
fn a_shift_scoped_to_a_track_with_no_elements_touches_nothing() {
    let dir = common::tempdir(line!());
    let path = write_project(
        &dir,
        "p.json",
        &canonical(
            r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[{"name":"a","layer":1,"elements":[{"id":"a1","type":"rect","start":0,"end":500,"width":100,"height":100,"fill":"#FF0000"},{"id":"a2","type":"rect","start":700,"end":1200,"width":100,"height":100,"fill":"#00FF00"}]},{"name":"b","layer":2,"elements":[]}]}"##,
        ),
    );

    let answer = shift(
        &path,
        &Ask {
            at: 600,
            delta: 100,
            scope: Some("b".to_string()),
            release: Vec::new(),
        },
    );

    assert_eq!(
        answer.report().exit_code(),
        ExitCode::Ok,
        "{:?}",
        codes(answer.report())
    );
    let written: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let elements = written["tracks"][0]["elements"].as_array().unwrap();
    assert_eq!(elements[1]["start"], 700, "out of scope: untouched");
    assert_eq!(elements[1]["end"], 1200, "out of scope: untouched");
}

#[test]
fn a_scope_naming_no_track_is_rejected() {
    let dir = common::tempdir(line!());
    let path = write_project(&dir, "p.json", &gap_project());

    let answer = shift(
        &path,
        &Ask {
            at: 550,
            delta: 200,
            scope: Some("nope".to_string()),
            release: Vec::new(),
        },
    );

    assert_eq!(answer.report().exit_code(), ExitCode::BadInvocation);
}

// ---------------------------------------------------------------------------
// Invocation-shaped refusals.
// ---------------------------------------------------------------------------

#[test]
fn a_non_positive_delta_is_rejected() {
    let dir = common::tempdir(line!());
    let path = write_project(&dir, "p.json", &gap_project());

    let answer = shift(&path, &ask(550, 0));
    assert_eq!(answer.report().exit_code(), ExitCode::BadInvocation);

    let answer = shift(&path, &ask(550, -100));
    assert_eq!(answer.report().exit_code(), ExitCode::BadInvocation);
}
