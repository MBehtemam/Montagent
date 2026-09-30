//! ADR-0112: a report names the check sets that ran, and prints no zero it did not earn.
//!
//! Every assertion goes through a verb's own call and reads the canonical JSON or the text
//! generated from it. `verify`'s `deliverable` set is asserted in `verify.rs`, beside the
//! renders it measures. The missing-`ffprobe` run is in `check_sets_environment.rs`, because
//! it empties `PATH` and needs a test binary of its own.

use montagent_core::finding::Class;
use montagent_core::registry::CheckSet;
use montagent_core::report::{ExitCode, Report};
use montagent_core::text;
use montagent_core::validate;
use montagent_core::verbs::compare::compare;
use montagent_core::verbs::create_project::{Scaffold, create_project};
use montagent_core::verbs::fmt::{Mode, fmt};
use montagent_core::verbs::{frame, preview, query, render, shift, timeline};
use serde_json::Value;

mod common;
use common::{canonical, fixture_project, tempdir, write_project};

/// No `source` anywhere, so the disk half has nothing to probe and needs no `ffprobe`.
const NO_MEDIA: &str = r##"{"frame":{"width":64,"height":64},"fps":25,"tracks":[{"name":"a","layer":1,"elements":[{"id":"r1","type":"rect","start":0,"end":1000,"width":10,"height":10,"fill":"#FF0000"}]}]}"##;

/// Two elements of one track sharing an instant: an `error`, so `render` and `preview`
/// refuse after the checks and before anything is encoded.
const OVERLAP: &str = r##"{"frame":{"width":64,"height":64},"fps":25,"tracks":[{"name":"a","layer":1,"elements":[{"id":"r1","type":"rect","start":0,"end":1000,"width":10,"height":10,"fill":"#FF0000"},{"id":"r2","type":"rect","start":500,"end":1500,"width":10,"height":10,"fill":"#00FF00"}]}]}"##;

fn sets(json: &Value) -> Vec<&str> {
    json["check_sets"]
        .as_array()
        .expect("every report carries `check_sets`")
        .iter()
        .map(|set| set.as_str().expect("a set is named by a string"))
        .collect()
}

fn prose(json: &Value) -> String {
    text::render(json, text::Options::default()).expect("the report renders")
}

fn header(json: &Value) -> String {
    prose(json).lines().next().unwrap_or_default().to_string()
}

fn not_checked(json: &Value) -> String {
    let prose = prose(json);
    let block = &prose[prose
        .find("NOT CHECKED")
        .expect("the block is unconditional")..];
    block.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn project(line: u32, body: &str) -> std::path::PathBuf {
    write_project(&tempdir(line), "p.montagent.json", &canonical(body))
}

fn not_a_project(line: u32) -> std::path::PathBuf {
    write_project(&tempdir(line), "transcript.json", r#"{"words": []}"#)
}

// ---- The field, as data ----------------------------------------------------------------

#[test]
fn a_new_report_starts_at_no_check_sets_and_keeps_all_six_summary_keys() {
    let json = Report::new("timeline", None).to_json();
    assert_eq!(sets(&json), Vec::<&str>::new());
    for key in ["error", "review", "note", "unchecked", "layout", "drift"] {
        assert_eq!(json["summary"][key], 0, "`summary.{key}` stays on the wire");
    }
}

#[test]
fn each_set_s_classes_are_the_registry_s_own() {
    // Derived, never copied (ADR-0112 §2). This pins what the registry says today; the
    // ADR's table also lists `note` under `disk`, which no disk-half code emits.
    let classes = |set: CheckSet| set.classes().collect::<Vec<_>>();
    use Class::*;
    assert_eq!(
        classes(CheckSet::Document),
        [Error, Review, Note, Unchecked, Layout]
    );
    assert_eq!(classes(CheckSet::Disk), [Error, Review, Unchecked]);
    assert_eq!(classes(CheckSet::Layout), [Layout]);
    assert_eq!(classes(CheckSet::Drift), [Drift]);
    assert_eq!(classes(CheckSet::Deliverable), [Error, Review, Note]);
}

// ---- validate's two halves -------------------------------------------------------------

#[test]
fn a_complete_validate_run_records_both_halves_and_prints_no_drift_zero() {
    let json = validate(&fixture_project()).to_json();
    assert_eq!(sets(&json), ["document", "disk"]);
    let header = header(&json);
    assert!(header.starts_with("0 errors, "), "{header}");
    assert!(header.contains(" 0 unchecked, 0 layout — "), "{header}");
    assert!(
        !header.contains("drift"),
        "validate runs no drift check:\n{header}"
    );
    assert_eq!(json["summary"]["drift"], 0, "the JSON keeps `drift: 0`");
    assert!(
        !not_checked(&json).contains("were not run"),
        "a complete run adds nothing to NOT CHECKED:\n{}",
        not_checked(&json)
    );
}

#[test]
fn a_project_with_no_media_completes_the_disk_half() {
    // ADR-0112 §4: the disk runner is entered, finds nothing to probe, and completes.
    let json = validate(&project(line!(), NO_MEDIA)).to_json();
    assert_eq!(sets(&json), ["document", "disk"]);
    assert_eq!(json["exit_code"], 0);
    assert_eq!(
        header(&json),
        format!(
            "0 errors, 0 reviews, 0 notes, 0 unchecked, 0 layout — {}",
            json["project"].as_str().unwrap()
        )
    );
}

#[test]
fn a_file_validate_cannot_read_as_a_project_records_no_check_set() {
    // ADR-0112 §3's own case: declared sets would put the incident into validate's output.
    for json in [
        validate(&not_a_project(line!())).to_json(),
        validate(&write_project(&tempdir(line!()), "p.json", "{ not json")).to_json(),
    ] {
        assert_eq!(sets(&json), Vec::<&str>::new());
        assert!(
            header(&json).starts_with("no checks run (validate runs them); 1 error — "),
            "{}",
            header(&json)
        );
        assert!(
            not_checked(&json).contains("validate's checks were not run; run validate for them."),
            "{}",
            not_checked(&json)
        );
    }
}

// ---- The verbs that run validate's engine ----------------------------------------------

#[test]
fn render_preview_shift_and_create_project_record_what_validate_records() {
    let refused_render =
        render::render(&project(line!(), OVERLAP), &Default::default(), &mut |_| {});
    let refused_preview =
        preview::preview(&project(line!(), OVERLAP), &Default::default(), &mut |_| {});
    let shifted = shift::shift(
        &project(line!(), NO_MEDIA),
        &shift::Ask {
            at: 0,
            delta: 100,
            scope: None,
            release: Vec::new(),
        },
    );
    let dir = tempdir(line!());
    let scaffolded = create_project(
        &dir.join("new.montagent.json"),
        &Scaffold {
            width: 64,
            height: 64,
            fps: 25,
            background: None,
            duration: Some(1000),
            output: None,
        },
    );

    for (verb, json) in [
        ("render", refused_render.to_json()),
        ("preview", refused_preview.to_json()),
        ("shift", shifted.to_json()),
        ("create-project", scaffolded.to_json()),
    ] {
        assert_eq!(sets(&json), ["document", "disk"], "{verb}");
        let header = header(&json);
        assert!(!header.contains("drift"), "{verb}: {header}");
        assert!(header.contains("0 layout"), "{verb}: {header}");
        assert_eq!(json["summary"]["drift"], 0, "{verb}");
    }
}

#[test]
fn a_refused_shift_or_create_project_records_no_check_set() {
    let refused_shift = shift::shift(
        &project(line!(), NO_MEDIA),
        &shift::Ask {
            at: 0,
            delta: -1,
            scope: None,
            release: Vec::new(),
        },
    );
    let dir = tempdir(line!());
    let existing = write_project(&dir, "p.montagent.json", &canonical(NO_MEDIA));
    let refused_scaffold = create_project(
        &existing,
        &Scaffold {
            width: 64,
            height: 64,
            fps: 25,
            background: None,
            duration: None,
            output: None,
        },
    );
    for (verb, report) in [
        ("shift", refused_shift.report()),
        ("create-project", &refused_scaffold),
    ] {
        assert_eq!(report.exit_code(), ExitCode::BadInvocation, "{verb}");
        assert_eq!(report.check_sets(), [], "{verb}");
    }
}

// ---- The subsets -----------------------------------------------------------------------

#[test]
fn fmt_records_the_layout_set_alone() {
    let clean = fmt(&project(line!(), NO_MEDIA), Mode::Check).to_json();
    let messy = fmt(
        &write_project(&tempdir(line!()), "p.montagent.json", NO_MEDIA),
        Mode::Check,
    )
    .to_json();
    for json in [&clean, &messy] {
        assert_eq!(sets(json), ["layout"]);
        assert!(
            header(json).starts_with("layout check only (validate's checks not run); "),
            "{}",
            header(json)
        );
    }
    assert!(
        header(&clean).contains("; 0 layout — "),
        "{}",
        header(&clean)
    );
    assert!(!header(&messy).contains("error"), "{}", header(&messy));

    let refused = fmt(&not_a_project(line!()), Mode::Check);
    assert_eq!(refused.check_sets(), []);
}

#[test]
fn compare_records_the_drift_set_alone() {
    let path = project(line!(), NO_MEDIA);
    let json = compare(&path, &path).to_json();
    assert_eq!(sets(&json), ["drift"]);
    assert!(
        header(&json).starts_with("drift checks only (validate's checks not run); 0 drift — "),
        "{}",
        header(&json)
    );
    assert!(not_checked(&json).contains("validate's checks were not run"));

    let refused = compare(&not_a_project(line!()), &path);
    assert_eq!(refused.check_sets(), []);
}

// ---- The verbs that run no checks ------------------------------------------------------

#[test]
fn a_verb_that_runs_no_checks_says_so_instead_of_printing_zeros() {
    let fixture = fixture_project();
    for (verb, json) in [
        ("timeline", timeline::timeline(&fixture).to_json()),
        (
            "query",
            query::query(
                &fixture,
                &query::Ask {
                    at: Some(1000),
                    ..Default::default()
                },
            )
            .to_json(),
        ),
    ] {
        assert_eq!(sets(&json), Vec::<&str>::new(), "{verb}");
        assert_eq!(
            header(&json),
            format!(
                "no checks run (validate runs them) — {}",
                json["project"].as_str().unwrap()
            ),
            "{verb}"
        );
        assert!(
            not_checked(&json).contains("validate's checks were not run; run validate for them."),
            "{verb}: {}",
            not_checked(&json)
        );
    }
}

#[test]
fn a_refusal_is_a_finding_without_being_a_check() {
    // ADR-0112 §2: `frame` runs no checks and can still raise an `error`.
    let json = frame::frame(
        &not_a_project(line!()),
        &frame::Ask {
            at: Some(0),
            ..Default::default()
        },
    )
    .to_json();
    assert_eq!(sets(&json), Vec::<&str>::new());
    assert_eq!(json["summary"]["error"], 1);
    assert!(
        header(&json).starts_with("no checks run (validate runs them); 1 error — "),
        "{}",
        header(&json)
    );
}
