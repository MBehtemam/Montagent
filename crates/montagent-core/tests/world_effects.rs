//! ADR-0093 ruling 6, and the invariant MONTAGENT-1 broke.
//!
//! > **A file at the output path is a render with zero errors.**
//!
//! What shipped an eleven-minute silent cut was not an unread report — it was **a plausible
//! file existing**, which is what a human uploads and what an MCP agent `stat`s. A counted
//! finding is necessary and not sufficient, so the assertions here are about the filesystem
//! and not only about the report.
//!
//! The other half of the ruling is condition 1, *"everything pre-flightable is pre-flighted"*.
//! The mix is a pure function of the document, the established facts and the range, so it is
//! decided before the encoder is spawned — and that is checkable from outside: a refused
//! render leaves **no temp file** beside the output, because there was never one to withhold.

use std::path::Path;

use montagent_core::finding::Class;
use montagent_core::report::ExitCode;
use montagent_core::verbs::render::{Answer, Ask, Progress, render};

mod common;
use common::{fixture_dir, has_ffprobe, tempdir, write_project};

fn rendered(path: &Path) -> Answer {
    render(path, &Ask::default(), &mut |_: Progress| {})
}

/// Everything sitting beside where the deliverable would be — the temp sibling included.
///
/// `Deliverable` writes to a dotted sibling of the target and publishes with one rename, so
/// *"nothing was written"* is a claim about the whole directory rather than about one path.
fn beside_the_output(dir: &Path) -> Vec<String> {
    let out = dir.join("out");
    std::fs::read_dir(&out)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .map(|entry| entry.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default()
}

fn project(dir: &Path, elements: &str) -> std::path::PathBuf {
    write_project(
        dir,
        "p.montagent.json",
        &format!(
            r##"{{"frame":{{"width":200,"height":200}},"fps":25,"background":"#000000",
                "duration":1000,"output":"out/p.mp4",
                "tracks":[{{"name":"only","layer":0,"elements":[{elements}]}}]}}"##
        ),
    )
}

/// A readable copy of the fixture's shortest narration, beside the project.
fn narration(dir: &Path, name: &str) -> String {
    std::fs::copy(fixture_dir().join("audio/05-cobweb.mp3"), dir.join(name)).expect("copied");
    name.to_string()
}

// ---------------------------------------------------------------------------
// Ruling 6: an `error` withholds the deliverable
// ---------------------------------------------------------------------------

#[test]
fn an_audible_element_that_cannot_be_mixed_leaves_no_file_at_the_output_path() {
    if !has_ffprobe() {
        eprintln!("skipped: no ffprobe on PATH (ADR-0009)");
        return;
    }
    let dir = tempdir(line!());
    let source = narration(&dir, "vo.mp3");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(dir.join(&source), std::fs::Permissions::from_mode(0o000))
            .expect("chmod");
    }
    #[cfg(not(unix))]
    {
        eprintln!("skipped: needs POSIX permissions to make a present file unreadable");
        return;
    }

    let path = project(
        &dir,
        &format!(
            r##"{{"id":"vo","type":"audio","start":0,"end":1000,
                "source":"{source}","source_start":0,"source_end":1000}}"##
        ),
    );

    let answer = rendered(&path);

    // This is the MONTAGENT-1 case, and every clause of it used to be the other way round.
    assert_eq!(
        answer.report().exit_code(),
        ExitCode::Errors,
        "{:?}",
        answer.report().findings
    );
    assert!(
        answer.report().findings.iter().any(|finding| {
            finding.code.starts_with("E-NOT-MIXED-") && finding.class == Class::Error
        }),
        "the drop was not a counted `error`: {:?}",
        answer.report().findings
    );
    assert!(
        answer.video().is_none(),
        "an answer with an `error` still described a video"
    );
    assert!(
        !dir.join("out/p.mp4").exists(),
        "a render with an `error` finding left a file at the output path — the invariant \
         ADR-0093 ruling 6 buys is exactly that this cannot happen"
    );

    // Condition 1: the mix is pre-flighted, so there was never a temp file. An empty `out`
    // directory (or none at all) is the observable form of *"no wall clock was spent"*.
    assert!(
        beside_the_output(&dir).is_empty(),
        "the refusal happened after the encoder started: {:?}",
        beside_the_output(&dir)
    );

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(dir.join(&source), std::fs::Permissions::from_mode(0o644));
    }
}

#[test]
fn a_render_with_no_world_effect_still_publishes_its_deliverable() {
    if !has_ffprobe() {
        eprintln!("skipped: no ffprobe on PATH (ADR-0009)");
        return;
    }
    let dir = tempdir(line!());
    let source = narration(&dir, "vo.mp3");
    let path = project(
        &dir,
        &format!(
            r##"{{"id":"vo","type":"audio","start":0,"end":1000,
                "source":"{source}","source_start":0,"source_end":1000}}"##
        ),
    );

    let answer = rendered(&path);

    // The other direction, without which every assertion above could be satisfied by a
    // `render` that has simply stopped working.
    assert_eq!(
        answer.report().exit_code(),
        ExitCode::Ok,
        "{:?}",
        answer.report().findings
    );
    assert!(
        answer.video().is_some(),
        "a clean render described no video"
    );
    assert!(
        dir.join("out/p.mp4").exists(),
        "a clean render published nothing"
    );
    // Published means *renamed*: the temp sibling is gone, so the only thing in `out` is the
    // deliverable.
    assert_eq!(beside_the_output(&dir), vec!["p.mp4".to_string()]);
}

// ---------------------------------------------------------------------------
// Ruling 2: the arms nobody had claimed
// ---------------------------------------------------------------------------

#[test]
fn an_element_whose_range_is_empty_is_an_error_rather_than_nothing() {
    if !has_ffprobe() {
        eprintln!("skipped: no ffprobe on PATH (ADR-0009)");
        return;
    }
    let dir = tempdir(line!());
    let source = narration(&dir, "vo.mp3");

    // `end` not exceeding `start` is the one cross-field fact the schema cannot express.
    // Before ADR-0093 this rendered at zero errors with the element silently absent; since
    // ADR-0107 the check engine refuses it before the mix, under the same code.
    let path = project(
        &dir,
        &format!(
            r##"{{"id":"vo","type":"audio","start":500,"end":500,
                "source":"{source}","source_start":0,"source_end":1000}}"##
        ),
    );

    let answer = rendered(&path);
    assert!(
        answer
            .report()
            .findings
            .iter()
            .any(|finding| finding.code == "E-EMPTY-RANGE"),
        "an element occupying no instant was passed over: {:?}",
        answer.report().findings
    );
    assert!(!dir.join("out/p.mp4").exists());
}

#[test]
fn a_video_with_no_audio_stream_is_a_note_and_does_not_withhold_the_file() {
    if !has_ffprobe() {
        eprintln!("skipped: no ffprobe on PATH (ADR-0009)");
        return;
    }
    let dir = tempdir(line!());
    // A committed fixture that carries a video stream and no audio one — an ordinary
    // silent clip, which is the whole point of the case.
    let source = fixture_dir()
        .join("reference/kenburns/05.mp4")
        .display()
        .to_string();
    let path = project(
        &dir,
        &format!(
            r##"{{"id":"clip","type":"video","start":0,"end":200,"x":100,"y":100,
                "width":100,"height":100,"fit":"cover",
                "source":"{}","source_start":0,"source_end":200}}"##,
            source.replace('\\', "/")
        ),
    );

    let answer = rendered(&path);

    // ADR-0093 ruling 2 reclassified this one on the way through, and it is the reason the
    // ruling insists on a code per reason: an ordinary video with no audio track is not a
    // defect, so it must not withhold the deliverable — but it is still not in the mix, and a
    // reader who cannot tell "silent by nature" from "dropped" chases the wrong thing.
    let note = answer
        .report()
        .findings
        .iter()
        .find(|finding| finding.code == "N-NO-AUDIO-STREAM");
    if let Some(note) = note {
        assert_eq!(note.class, Class::Note, "{note:?}");
        assert_eq!(
            answer.report().exit_code(),
            ExitCode::Ok,
            "a `note` withheld the deliverable: {:?}",
            answer.report().findings
        );
        assert!(
            dir.join("out/p.mp4").exists(),
            "a `note` withheld the deliverable"
        );
    } else {
        panic!(
            "a silent video was mixed in silence — the `note` never fired: {:?}",
            answer.report().findings
        );
    }
}

// ---------------------------------------------------------------------------
// ADR-0121: a partial render's world effects stop at its range, and it says so
// ---------------------------------------------------------------------------

#[test]
fn a_partial_render_publishes_past_a_world_effect_outside_its_range_and_says_so() {
    if !has_ffprobe() {
        eprintln!("skipped: no ffprobe on PATH (ADR-0009)");
        return;
    }
    #[cfg(not(unix))]
    {
        eprintln!("skipped: needs POSIX permissions to make a present file unreadable");
        return;
    }
    let dir = tempdir(line!());
    let source = narration(&dir, "vo.mp3");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(dir.join(&source), std::fs::Permissions::from_mode(0o000))
            .expect("chmod");
    }
    // The unmixable element sits at 1000..2000, wholly outside the partial range 0..1000.
    let path = write_project(
        &dir,
        "p.montagent.json",
        &format!(
            r##"{{"frame":{{"width":200,"height":200}},"fps":25,"background":"#000000",
                "duration":2000,"output":"out/p.mp4",
                "tracks":[{{"name":"only","layer":0,"elements":[
                    {{"id":"vo","type":"audio","start":1000,"end":2000,
                      "source":"{source}","source_start":0,"source_end":1000}}]}}]}}"##
        ),
    );
    let boundary = |json: &serde_json::Value| -> Vec<String> {
        json["not_checked_also"]
            .as_array()
            .map(|lines| {
                lines
                    .iter()
                    .filter_map(|line| line.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default()
    };

    // The full render refuses on the element: this is ADR-0093, unchanged.
    let full = rendered(&path);
    assert_eq!(
        full.report().exit_code(),
        ExitCode::Errors,
        "{:?}",
        full.report().findings
    );
    assert!(
        !boundary(&full.to_json())
            .iter()
            .any(|line| line.contains("partial render")),
        "a full render states no partial scope: {:?}",
        boundary(&full.to_json())
    );

    // The partial render publishes: it is never the deliverable, and its file is right for
    // its range.
    let partial = render(
        &path,
        &Ask {
            from: Some(0),
            to: Some(1000),
            ..Ask::default()
        },
        &mut |_: Progress| {},
    );
    assert_eq!(
        partial.report().exit_code(),
        ExitCode::Ok,
        "{:?}",
        partial.report().findings
    );
    assert!(
        partial.video().is_some(),
        "the partial render wrote nothing"
    );

    // And it now says where its world effects stopped, in the JSON and in the text.
    let json = partial.to_json();
    let scope: Vec<String> = boundary(&json)
        .into_iter()
        .filter(|line| line.contains("partial render"))
        .collect();
    assert_eq!(scope.len(), 1, "{:?}", boundary(&json));
    assert!(scope[0].contains("0..1000 ms"), "{}", scope[0]);
    assert!(
        scope[0].contains("says nothing about whether the full render will pass"),
        "{}",
        scope[0]
    );
    let prose = montagent_core::text::render(&json, montagent_core::text::Options::default())
        .expect("the report renders");
    assert!(prose.contains("0..1000 ms"), "{prose}");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(dir.join(&source), std::fs::Permissions::from_mode(0o644));
    }
}
