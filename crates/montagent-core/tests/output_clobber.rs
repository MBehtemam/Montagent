//! ADR-0104: what is already at the output path.
//!
//! The incident, in one sentence: a generator hardcoded `"output": "../../out/da-episode.mp4"`,
//! a sibling-language project inherited it unchanged, and rendering the second language would
//! have renamed an eleven-minute finished cut out of existence — four attempts and ninety
//! minutes of encode — at `0 errors`, because the file was legal, the path resolved and
//! nothing looked at what was already there.
//!
//! **The shape of this suite is the decision.** The predicate had to distinguish two cases
//! that every cheap heuristic confuses:
//!
//! - *attempt four of the same cut over attempt three* — the normal working loop, and never
//!   a finding;
//! - *the German cut over the Danish one* — two documents that may agree to the frame on
//!   duration, frame count and frame size, and that a duration comparison would therefore
//!   wave through.
//!
//! So the tests below render one project twice (must be silent) and then render its sibling
//! at the same path (must refuse). A suite that only asserted the refusal would be passed by
//! a `render` that simply refuses to overwrite anything.

use std::path::Path;

use montagent_core::finding::Class;
use montagent_core::report::ExitCode;
use montagent_core::verbs::render::{Answer, Ask, Progress, render};

mod common;
use common::{has_ffprobe, tempdir, write_project};

fn rendered(path: &Path, no_clobber: bool) -> Answer {
    render(
        path,
        &Ask {
            no_clobber,
            ..Ask::default()
        },
        &mut |_: Progress| {},
    )
}

/// A one-second black project writing to `out/episode.mp4`, named so two of them can sit in
/// one directory and contend for the same deliverable — which is the incident.
fn episode(dir: &Path, name: &str, background: &str) -> std::path::PathBuf {
    write_project(
        dir,
        name,
        &format!(
            r##"{{"frame":{{"width":160,"height":120}},"fps":25,"background":"{background}",
            "duration":1000,"output":"out/episode.mp4","tracks":[]}}"##
        ),
    )
}

fn codes(answer: &Answer) -> Vec<String> {
    answer
        .report()
        .findings
        .iter()
        .map(|finding| finding.code.to_string())
        .collect()
}

#[test]
fn a_first_render_onto_an_empty_path_says_nothing_about_it() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let answer = rendered(&episode(&dir, "da.montagent.json", "#000000"), false);

    assert_eq!(answer.report().exit_code(), ExitCode::Ok);
    assert!(
        dir.join("out/episode.mp4").exists(),
        "the deliverable is published"
    );
    assert!(
        !codes(&answer)
            .iter()
            .any(|code| code.contains("OUTPUT-FOREIGN") || code.contains("OUTPUT-UNATTESTED")),
        "nothing was there, so there is nothing to report: {:?}",
        codes(&answer)
    );
}

/// The case every cheap predicate gets wrong, and the reason mere existence was rejected.
///
/// The source incident's own cut took **four attempts**. A check that fired on the second
/// render of one project would fire three times in that story and be suppressed before it
/// ever met the German project.
#[test]
fn rerendering_the_same_project_over_its_own_last_answer_is_silent() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let project = episode(&dir, "da.montagent.json", "#000000");

    let first = rendered(&project, false);
    assert_eq!(first.report().exit_code(), ExitCode::Ok);

    let second = rendered(&project, false);
    assert_eq!(
        second.report().exit_code(),
        ExitCode::Ok,
        "attempt four over attempt three is the normal loop, not a defect"
    );
    assert!(
        !codes(&second)
            .iter()
            .any(|code| code.contains("OUTPUT-FOREIGN") || code.contains("OUTPUT-UNATTESTED")),
        "its own attestation is its own: {:?}",
        codes(&second)
    );
    assert!(dir.join("out/episode.mp4").exists());
}

/// **The incident.** Two sibling projects, one hardcoded `output`.
///
/// Note what is *not* asserted: nothing here compares durations or frame counts, because the
/// two documents are byte-identical apart from their filenames and agree on every one of
/// them. That agreement is precisely why the heuristic the source doc proposed would have
/// stayed silent, and why the predicate is an attestation instead.
#[test]
fn a_sibling_project_is_refused_and_the_finished_cut_survives() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    // The two differ in background and in nothing else: same duration, same frame count,
    // same frame size. Both halves are load-bearing. The differing pixels are what makes a
    // clobber *detectable* — Montagent is deterministic, so two identical documents produce
    // byte-identical files and the destruction would be invisible to the assertion below.
    // The identical duration and frame count are why this is the right fixture for the
    // decision: they are exactly what the heuristic the source doc proposed compares.
    let danish = episode(&dir, "da.montagent.json", "#000000");
    let german = episode(&dir, "de.montagent.json", "#112233");

    assert_eq!(rendered(&danish, false).report().exit_code(), ExitCode::Ok);
    let deliverable = dir.join("out/episode.mp4");
    let finished = std::fs::read(&deliverable).expect("the Danish cut");

    let answer = rendered(&german, false);

    assert_eq!(
        answer.report().exit_code(),
        ExitCode::Errors,
        "rendering German over the Danish deliverable is refused"
    );
    assert!(
        codes(&answer).iter().any(|code| code == "E-OUTPUT-FOREIGN"),
        "at its own code: {:?}",
        codes(&answer)
    );
    assert_eq!(
        answer
            .report()
            .findings
            .iter()
            .find(|finding| finding.code == "E-OUTPUT-FOREIGN")
            .map(|finding| finding.class),
        Some(Class::Error),
        "ADR-0093: a world-effect of the promotion is an `error`"
    );

    // The whole point. A counted finding is necessary and not sufficient — what the incident
    // would have lost is the bytes.
    assert_eq!(
        std::fs::read(&deliverable).expect("still there"),
        finished,
        "the Danish cut is untouched, byte for byte"
    );
}

/// ADR-0093 condition 1: everything pre-flightable is pre-flighted.
///
/// The injury is measured in ninety-minute encodes, so a refusal that arrives at promotion
/// time would save the file and still spend the clock. `Deliverable` writes to a dotted
/// sibling, so *"the encoder never ran"* is a claim about the whole directory: if a temp
/// sibling exists, frames were painted.
#[test]
fn the_refusal_costs_no_wall_clock() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let danish = episode(&dir, "da.montagent.json", "#000000");
    let german = episode(&dir, "de.montagent.json", "#112233");
    assert_eq!(rendered(&danish, false).report().exit_code(), ExitCode::Ok);

    rendered(&german, false);

    let beside: Vec<String> = std::fs::read_dir(dir.join("out"))
        .expect("out/")
        .filter_map(Result::ok)
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        beside,
        vec!["episode.mp4".to_string()],
        "no temp sibling: the encoder was never spawned"
    );
}

/// The migration case, and the one the jury split on.
///
/// A deliverable that predates attestation — or one a human re-encoded, moved or produced
/// with another tool — carries no stamp. Montagent has no evidence either way, and `error`
/// is defined as *"refused or **guaranteed** wrong"*, which is a guarantee it cannot make.
/// So: `review`, the file is replaced, and the render still succeeds.
#[test]
fn an_unattested_file_is_a_review_and_is_replaced() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let project = episode(&dir, "da.montagent.json", "#000000");
    std::fs::create_dir_all(dir.join("out")).expect("out/");
    std::fs::write(dir.join("out/episode.mp4"), b"not a montagent render").expect("squatter");

    let answer = rendered(&project, false);

    assert_eq!(
        answer.report().exit_code(),
        ExitCode::Ok,
        "no evidence is not a refusal"
    );
    assert!(
        codes(&answer)
            .iter()
            .any(|code| code == "R-OUTPUT-UNATTESTED"),
        "but it is said out loud: {:?}",
        codes(&answer)
    );
    assert!(
        std::fs::read(dir.join("out/episode.mp4")).expect("replaced") != b"not a montagent render",
        "and the render proceeded"
    );
}

/// `--no-clobber` only ever tightens. It turns the `review` above into a refusal; there is
/// deliberately no flag in the other direction, because ADR-0093 condition 2 forbids one
/// that would let an unwaived `error` reach the output path.
#[test]
fn no_clobber_escalates_the_unattested_case_to_a_refusal() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let project = episode(&dir, "da.montagent.json", "#000000");
    std::fs::create_dir_all(dir.join("out")).expect("out/");
    std::fs::write(dir.join("out/episode.mp4"), b"not a montagent render").expect("squatter");

    let answer = rendered(&project, true);

    assert_eq!(answer.report().exit_code(), ExitCode::Errors);
    assert_eq!(
        std::fs::read(dir.join("out/episode.mp4")).expect("still there"),
        b"not a montagent render",
        "refused means the bytes are untouched"
    );
}
