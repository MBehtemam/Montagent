//! ADR-0109 (#440): a cancelled `render` or `preview` stops where it stands and publishes
//! nothing. The MCP half — that `notifications/cancelled` reaches the flag — is in the
//! `montagent` crate's `mcp_concurrency.rs`; this file is the core's promise about the disk.

use std::path::{Path, PathBuf};

use montagent_core::report::ExitCode;
use montagent_core::verbs::preview::{self, preview_cancellable};
use montagent_core::verbs::render::{Answer, Ask, Cancel, Progress, render, render_cancellable};

mod common;
use common::{canonical, has_ffprobe, tempdir, write_project};

/// A few seconds of frames, so a cancel at frame 0 has a long way left to run.
fn project(dir: &Path, background: &str) -> PathBuf {
    write_project(
        dir,
        "p.montagent.json",
        &canonical(&format!(
            r##"{{"frame":{{"width":160,"height":120}},"fps":25,"background":"{background}",
            "duration":4000,"output":"out/p.mp4","tracks":[]}}"##
        )),
    )
}

fn codes(answer: &Answer) -> Vec<String> {
    codes_of(answer.report())
}

fn codes_of(report: &montagent_core::report::Report) -> Vec<String> {
    report
        .findings
        .iter()
        .map(|finding| finding.code.to_string())
        .collect()
}

/// Everything in `out/`, so a leftover temp file is caught as well as a published one.
fn beside_the_output(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir.join("out"))
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .map(|entry| entry.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    names
}

#[test]
fn a_render_cancelled_mid_encode_publishes_nothing_and_leaves_the_last_deliverable_alone() {
    if !has_ffprobe() {
        eprintln!("skipped: no ffprobe on PATH (ADR-0009)");
        return;
    }
    let dir = tempdir(line!());
    let path = project(&dir, "#000000");

    // A previous, stamped deliverable at the path — the file the cancelled call must not
    // replace.
    let first = render(&path, &Ask::default(), &mut |_: Progress| {});
    assert!(first.video().is_some(), "{:?}", first.report().findings);
    let before = std::fs::read(dir.join("out/p.mp4")).expect("the first deliverable");

    // Edit the project so a completed second render would produce different bytes: a test
    // that passed because the replacement happened to be identical would prove nothing.
    project(&dir, "#FF0000");

    // Cancelled from inside the first progress report — frame 0 — so the stop is observed
    // mid-loop, which is the path `notifications/cancelled` takes.
    let cancel = Cancel::new();
    let mut frames_seen = Vec::new();
    let answer = render_cancellable(
        &path,
        &Ask::default(),
        &mut |p: Progress| {
            frames_seen.push(p.done);
            cancel.cancel();
        },
        Some(&cancel),
    );

    assert!(answer.video().is_none(), "a cancelled render published");
    assert_eq!(codes(&answer), vec!["E-CANCELLED".to_string()]);
    assert!(answer.report().is_not_about_document());
    assert_eq!(answer.report().exit_code(), ExitCode::Internal);
    // It stopped where it stood: only the frame-0 report, never the tenths after it.
    assert_eq!(frames_seen, vec![0]);
    assert_eq!(
        std::fs::read(dir.join("out/p.mp4")).expect("the first deliverable"),
        before,
        "the previous deliverable was touched"
    );
    // No temp file left behind to read as a half-finished render.
    assert_eq!(beside_the_output(&dir), vec!["p.mp4".to_string()]);
}

#[test]
fn a_render_cancelled_before_it_starts_spawns_no_encoder() {
    if !has_ffprobe() {
        eprintln!("skipped: no ffprobe on PATH (ADR-0009)");
        return;
    }
    let dir = tempdir(line!());
    let path = project(&dir, "#000000");
    let cancel = Cancel::new();
    cancel.cancel();

    let mut frames_seen = 0;
    let answer = render_cancellable(
        &path,
        &Ask::default(),
        &mut |_: Progress| frames_seen += 1,
        Some(&cancel),
    );
    assert_eq!(codes(&answer), vec!["E-CANCELLED".to_string()]);
    assert_eq!(frames_seen, 0, "the frame loop was entered");
    assert!(
        beside_the_output(&dir).is_empty(),
        "{:?}",
        beside_the_output(&dir)
    );
}

#[test]
fn a_cancel_nobody_sets_changes_nothing() {
    if !has_ffprobe() {
        eprintln!("skipped: no ffprobe on PATH (ADR-0009)");
        return;
    }
    let dir = tempdir(line!());
    let path = project(&dir, "#000000");
    let answer = render_cancellable(
        &path,
        &Ask::default(),
        &mut |_: Progress| {},
        Some(&Cancel::new()),
    );
    assert!(answer.video().is_some(), "{:?}", answer.report().findings);
}

#[test]
fn a_preview_cancelled_mid_encode_publishes_nothing_and_tries_no_other_rung() {
    if !has_ffprobe() {
        eprintln!("skipped: no ffprobe on PATH (ADR-0009)");
        return;
    }
    let dir = tempdir(line!());
    let path = project(&dir, "#000000");
    let cancel = Cancel::new();
    let mut reports = 0;
    let answer = preview_cancellable(
        &path,
        &preview::Ask::default(),
        &mut |_: Progress| {
            reports += 1;
            cancel.cancel();
        },
        Some(&cancel),
    );
    assert!(answer.preview().is_none(), "a cancelled preview published");
    assert_eq!(codes_of(answer.report()), vec!["E-CANCELLED".to_string()]);
    // One rung's frame-0 report and nothing after: a cancel is not a missed budget, so the
    // ladder does not descend.
    assert_eq!(reports, 1);
    assert!(
        beside_the_output(&dir).is_empty(),
        "{:?}",
        beside_the_output(&dir)
    );
}
