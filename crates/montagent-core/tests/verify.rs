//! ADR-0117: `verify` — *is X in the file?*
//!
//! Every test here renders a real deliverable and then does to it, or to what it was rendered
//! from, the one thing its name says. The four the ticket names are the acceptance: a
//! truncated audio track, a document edited after the render, a source swapped by a
//! renumbering shuffle, and a silent source that must be censused as the author's material.
//! The rest pin the identity gate's other arms and the census's other half, because a census
//! that only ever produced one group would be passed by a check that never measured a source.

use std::path::{Path, PathBuf};
use std::process::Command;

use montagent_core::finding::{Class, Finding};
use montagent_core::report::ExitCode;
use montagent_core::verbs::render::{Ask, Progress, render};
use montagent_core::verbs::verify::{Answer, verify};

mod common;
use common::{canonical, has_ffprobe, tempdir, write_project};

fn ffmpeg(args: &[&str]) {
    let status = Command::new("ffmpeg")
        .args(["-hide_banner", "-loglevel", "error", "-y"])
        .args(args)
        .status()
        .expect("run ffmpeg");
    assert!(status.success(), "ffmpeg {args:?}");
}

/// Three seconds of a 440 Hz tone, and three of digital silence, both 48 kHz mono.
fn media(dir: &Path) {
    ffmpeg(&[
        "-f",
        "lavfi",
        "-i",
        "sine=f=440:d=3",
        "-ar",
        "48000",
        dir.join("tone.wav").to_str().unwrap(),
    ]);
    ffmpeg(&[
        "-f",
        "lavfi",
        "-i",
        "anullsrc=r=48000:cl=mono",
        "-t",
        "3",
        dir.join("silent.wav").to_str().unwrap(),
    ]);
}

/// A 3 s project: `first` sounds over `[0, 1500)` and `second` over `[1500, 3000)`.
fn project(dir: &Path, first: &str, second: &str, duration: i64) -> PathBuf {
    write_project(
        dir,
        "p.montagent.json",
        &canonical(&format!(
            r##"{{"frame":{{"width":160,"height":120}},"fps":25,"duration":{duration},"output":"out/p.mp4",
            "tracks":[{{"name":"a","layer":1,"elements":[
              {{"id":"line-01","type":"audio","start":0,"end":1500,"source":"{first}","source_start":0,"source_end":1500}},
              {{"id":"line-02","type":"audio","start":1500,"end":3000,"source":"{second}","source_start":0,"source_end":1500}}]}}]}}"##
        )),
    )
}

fn rendered(project: &Path) {
    let answer = render(project, &Ask::default(), &mut |_: Progress| {});
    assert_eq!(
        answer.report().exit_code(),
        ExitCode::Ok,
        "{:?}",
        answer.report().findings
    );
}

fn codes(answer: &Answer) -> Vec<String> {
    answer
        .report()
        .findings
        .iter()
        .map(|finding| finding.code.clone())
        .collect()
}

fn only<'a>(answer: &'a Answer, code: &str) -> &'a Finding {
    let found: Vec<&Finding> = answer
        .report()
        .findings
        .iter()
        .filter(|finding| finding.code == code)
        .collect();
    assert_eq!(found.len(), 1, "one {code} in {:?}", codes(answer));
    found[0]
}

/// Re-mux the deliverable through `ffmpeg`, keeping its stamp (`-map_metadata 0`), with the
/// audio filtered by `audio_filter`. The file is still this project's render in every respect
/// the identity gate reads — which is what makes each mismatch below the engine's to answer for.
fn remux_audio(output: &Path, audio_filter: &str) {
    let temp = output.with_extension("tmp.mp4");
    ffmpeg(&[
        "-i",
        output.to_str().unwrap(),
        "-map",
        "0:v",
        "-map",
        "0:a",
        "-c:v",
        "copy",
        "-af",
        audio_filter,
        "-c:a",
        "aac",
        "-map_metadata",
        "0",
        temp.to_str().unwrap(),
    ]);
    std::fs::rename(&temp, output).unwrap();
}

#[test]
fn a_fresh_render_of_this_document_verifies_clean() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    media(&dir);
    let project = project(&dir, "tone.wav", "tone.wav", 3000);
    rendered(&project);

    let answer = verify(&project);
    assert_eq!(answer.report().exit_code(), ExitCode::Ok);
    assert!(
        codes(&answer).is_empty(),
        "a render of this document, measured by the decoder, agrees with it: {:?}",
        codes(&answer)
    );
    let measured = answer
        .measured()
        .expect("the gate passed, so the file was measured");
    assert_eq!(measured.attestation, "mine");
    assert_eq!(measured.frames, 75);
    assert_eq!((measured.width, measured.height), (160, 120));
}

#[test]
fn nothing_at_the_output_path_is_one_error_and_nothing_is_measured() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    media(&dir);
    let project = project(&dir, "tone.wav", "tone.wav", 3000);

    let answer = verify(&project);
    assert_eq!(codes(&answer), vec!["E-VERIFY-NO-OUTPUT"]);
    assert!(answer.measured().is_none());
}

/// **Acceptance 1.** The shape *"silent at 0:30"* most likely takes: the mix is cut short.
/// The audio-extent check catches it with no borrowed threshold.
#[test]
fn a_truncated_audio_track_is_an_error() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    media(&dir);
    let project = project(&dir, "tone.wav", "tone.wav", 3000);
    rendered(&project);
    remux_audio(&dir.join("out/p.mp4"), "atrim=end=1");

    let answer = verify(&project);
    let finding = only(&answer, "E-VERIFY-AUDIO-EXTENT");
    assert_eq!(finding.class, Class::Error);
    assert_eq!(answer.report().exit_code(), ExitCode::Errors);
    let audio = finding.fields["audio_ms"].as_f64().unwrap();
    assert!(
        (audio - 1000.0).abs() < 50.0,
        "the raw measurement is the substance: {audio}"
    );
}

/// **Acceptance 2.** After an edit every mismatch descends from one fact, so that fact is the
/// one finding. The edit here halves `duration`, which would otherwise be an extent error and
/// an audio-extent error at least.
#[test]
fn a_document_edited_after_the_render_is_one_stale_error_and_no_measurement() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    media(&dir);
    let rendered_project = project(&dir, "tone.wav", "tone.wav", 3000);
    rendered(&rendered_project);
    let edited = project(&dir, "tone.wav", "tone.wav", 1500);

    let answer = verify(&edited);
    assert_eq!(codes(&answer), vec!["E-VERIFY-STALE"]);
    assert!(answer.measured().is_none(), "nothing was measured");
}

/// A whitespace or key-order edit renders identically, so it is not staleness.
#[test]
fn reformatting_the_document_does_not_make_the_deliverable_stale() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    media(&dir);
    let project = project(&dir, "tone.wav", "tone.wav", 3000);
    rendered(&project);
    let body = std::fs::read_to_string(&project).unwrap();
    let value: serde_json::Value = serde_json::from_str(&body).unwrap();
    std::fs::write(&project, serde_json::to_string(&value).unwrap()).unwrap();

    assert!(!codes(&verify(&project)).contains(&"E-VERIFY-STALE".to_string()));
}

/// **Acceptance 3.** MONTAGENT-1's renumbering shuffle: two takes of one length trade names by
/// `mv`, which keeps each file's mtime to the nanosecond. The document is byte-identical; only
/// content identity can see the change.
#[test]
fn a_source_swapped_by_a_renumbering_shuffle_is_stale() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    media(&dir);
    std::fs::rename(dir.join("tone.wav"), dir.join("line-01.wav")).unwrap();
    std::fs::rename(dir.join("silent.wav"), dir.join("line-02.wav")).unwrap();
    let project = project(&dir, "line-01.wav", "line-02.wav", 3000);
    rendered(&project);

    let before = std::fs::metadata(dir.join("line-01.wav")).unwrap();
    std::fs::rename(dir.join("line-01.wav"), dir.join("swap.wav")).unwrap();
    std::fs::rename(dir.join("line-02.wav"), dir.join("line-01.wav")).unwrap();
    std::fs::rename(dir.join("swap.wav"), dir.join("line-02.wav")).unwrap();
    let after = std::fs::metadata(dir.join("line-02.wav")).unwrap();
    assert_eq!(
        (before.len(), before.modified().unwrap()),
        (after.len(), after.modified().unwrap()),
        "the fixture is the shuffle: size and mtime travel with the bytes"
    );

    assert_eq!(codes(&verify(&project)), vec!["E-VERIFY-STALE"]);
}

/// **Acceptance 4.** A recording that is genuinely silent is the author's material, and the
/// census says so; without the per-source split it would read as an engine bug.
#[test]
fn a_silent_source_is_censused_as_the_authors_material() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    media(&dir);
    let project = project(&dir, "tone.wav", "silent.wav", 3000);
    rendered(&project);

    let answer = verify(&project);
    let finding = only(&answer, "R-VERIFY-SILENT-SPAN");
    assert_eq!(
        finding.class,
        Class::Review,
        "borrowed thresholds decide `review` only"
    );
    assert!(
        finding.citation.is_some(),
        "and are cited inline (ADR-0061)"
    );
    let census = finding.census.as_ref().expect("a census");
    assert_eq!(census.groups.len(), 1, "{census:?}");
    assert_eq!(census.groups[0].value, "silent in its own source");
    assert_eq!(census.groups[0].members, vec!["line-02"]);
    assert!(finding.fields["from"].as_i64().unwrap() >= 1500);
}

/// The other half of the split: the source is audible and the mix is silent, which points at
/// the engine. Produced by replacing a real render's mix with silence under its own stamp.
#[test]
fn an_audible_source_under_a_silent_mix_is_censused_as_the_engines() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    media(&dir);
    let project = project(&dir, "tone.wav", "tone.wav", 3000);
    rendered(&project);
    remux_audio(&dir.join("out/p.mp4"), "volume=0");

    let answer = verify(&project);
    let finding = only(&answer, "R-VERIFY-SILENT-SPAN");
    let census = finding.census.as_ref().expect("a census");
    assert_eq!(census.groups.len(), 1, "{census:?}");
    assert_eq!(census.groups[0].value, "audible in its own source");
    assert_eq!(census.groups[0].members, vec!["line-01", "line-02"]);
}

/// No stamp is no evidence: `review`, and the measurements still run at their own classes.
#[test]
fn an_unstamped_file_is_reviewed_and_measured_anyway() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    media(&dir);
    let project = project(&dir, "tone.wav", "tone.wav", 3000);
    ffmpeg(&[
        "-f",
        "lavfi",
        "-i",
        "color=c=black:s=320x240:r=25:d=3",
        "-f",
        "lavfi",
        "-i",
        "sine=f=440:d=3",
        "-c:v",
        "libx264",
        "-pix_fmt",
        "yuv420p",
        "-c:a",
        "aac",
        dir.join("out-unstamped.mp4").to_str().unwrap(),
    ]);
    std::fs::create_dir_all(dir.join("out")).unwrap();
    std::fs::rename(dir.join("out-unstamped.mp4"), dir.join("out/p.mp4")).unwrap();

    let answer = verify(&project);
    assert_eq!(only(&answer, "R-VERIFY-UNATTESTED").class, Class::Review);
    assert_eq!(only(&answer, "E-VERIFY-FRAME-SIZE").class, Class::Error);
    assert_eq!(answer.measured().unwrap().attestation, "unattested");
}

/// A deliverable rendered before the digest existed is this project's, staleness unknown —
/// worded apart from the unstamped case, and still measured. `render` reads it as its own.
#[test]
fn a_montagent_1_stamp_is_mine_with_its_staleness_unknown() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    media(&dir);
    let project = project(&dir, "tone.wav", "tone.wav", 3000);
    rendered(&project);
    let output = dir.join("out/p.mp4");
    let temp = dir.join("out/old.mp4");
    let v1 = format!(
        "comment=montagent/1 project={}",
        montagent_core::media::attest::identity(&project)
    );
    ffmpeg(&[
        "-i",
        output.to_str().unwrap(),
        "-map",
        "0",
        "-c",
        "copy",
        "-metadata",
        &v1,
        temp.to_str().unwrap(),
    ]);
    std::fs::rename(&temp, &output).unwrap();

    let answer = verify(&project);
    assert_eq!(codes(&answer), vec!["R-VERIFY-STALENESS-UNKNOWN"]);
    assert_eq!(answer.measured().unwrap().attestation, "staleness_unknown");

    let rerender = render(&project, &Ask::default(), &mut |_: Progress| {});
    assert!(
        !rerender
            .report()
            .findings
            .iter()
            .any(|finding| finding.code.starts_with("R-OUTPUT")
                || finding.code.starts_with("E-OUTPUT")),
        "render's pre-flight reads a `montagent/1` stamp as its own"
    );
}
