//! `validate` against the disk (#203): *"does this document agree with the media on
//! disk?"*
//!
//! These run against **real media and a real `ffprobe`**, not a recorded one. That is a
//! deliberate split from `tests/probe.rs`, which records `ffprobe` because the things it
//! asserts are network behaviours that cannot be reproduced honestly any other way. What
//! this file asserts is the opposite kind of claim — that the numbers in a document and
//! the numbers in a file agree — and a recorded probe would let the check pass against
//! media that never existed.
//!
//! The boundary case is free, and it is the fixture's own: **all 20 of its audio elements
//! consume their source to the exact last millisecond** (`source_end == probed duration`,
//! margin 0 on every one). ADR-0011 noticed this as an accident — *"a source that grew is
//! caught only by the accident that this fixture consumes every source to its last
//! millisecond"* — and it makes an off-by-one here impossible to hide: `<=` and `<` give
//! different answers on 20 committed elements.

use std::path::{Path, PathBuf};

use montaget_core::finding::Class;
use montaget_core::media::probe::ProcessRunner;
use montaget_core::media::session::Session;
use montaget_core::media::tools;
use montaget_core::report::{ExitCode, Report};

mod common;
use common::{has_ffprobe, write_project};

/// `validate`, over a session with **no sidecar**.
///
/// The sidecar is real behaviour and ADR-0069 asks for it — `tests/sidecar.rs` is where it
/// is tested — but a cache that persists across runs would make the miss counts below
/// depend on what an earlier `cargo test` left in the user's cache directory. These tests
/// are about what *one* run reports, so they get a cache that begins and ends with the call.
fn validate(path: &Path) -> Report {
    match tools::resolve() {
        Ok(tools) => {
            let mut session = Session::with(tools, Box::new(ProcessRunner));
            montaget_core::verbs::validate::validate_with(path, &mut session)
        }
        // No `ffprobe`: there is no session to hold a cache, and the exit-70 path is the
        // behaviour under test.
        Err(_) => montaget_core::validate(path),
    }
}

/// A resolved path with `/` separators, so a tail can be compared to one.
///
/// The project format writes `audio/05-cobweb.mp3`; a resolved path on Windows
/// comes back with backslashes, and `ends_with("audio/05-cobweb.mp3")` is then
/// false for a path that is entirely correct. The separator is the platform's,
/// never the document's.
fn with_forward_slashes(path: &str) -> String {
    path.replace('\\', "/")
}

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/en-halloween-decorating")
        .canonicalize()
        .expect("the committed fixture")
}

/// A one-element audio project around a source, written the way the fixture writes them.
fn audio_project(source: &str, source_start: i64, source_end: i64) -> String {
    format!(
        r##"{{"frame":{{"width":1080,"height":1920}},"fps":25,"tracks":[{{"name":"vo","layer":1,"elements":[{{"id":"vo-01","type":"audio","start":0,"end":{span},"source":{source},"source_start":{source_start},"source_end":{source_end}}}]}}]}}"##,
        span = source_end - source_start,
        source = json_string(source)
    )
}

/// A path is not a JSON string until something escapes it.
///
/// These builders write a project by interpolation, and an absolute Windows path
/// carries backslashes: `C:\Users\...` lands in the file as the invalid escape
/// `\U` and the whole project fails to parse. The product was right to refuse
/// it; the test was wrong to write it. Found by #189's CI the first time the
/// suite ran on `aarch64-pc-windows-msvc` -- which is the class of thing ADR-0064
/// recorded a dissent about and asked implementation to report back on.
fn json_string(value: &str) -> String {
    serde_json::to_string(value).expect("a string is always serialisable")
}

/// The builders above escape what they are given, checked on a path this machine
/// will never produce.
///
/// The bug these tests had was invisible on Unix: a source path with no
/// backslashes interpolates into a JSON string literal unharmed, so four tests
/// passed here and failed on `aarch64-pc-windows-msvc` the first time #189's CI
/// ran them -- with `E-PARSE ... invalid escape` on a file the test itself had
/// written. This asserts the fix from a platform that cannot reproduce the
/// failure, so the guard does not depend on a Windows runner to be useful.
#[test]
fn a_windows_path_survives_being_written_into_a_project() {
    let windows_path = r"C:\Users\RUNNER~1\AppData\Local\Temp\montaget\cobweb.mp3";

    for project in [
        audio_project(windows_path, 0, 1776),
        video_project(windows_path, 65216),
    ] {
        let parsed: serde_json::Value = serde_json::from_str(&project).unwrap_or_else(|e| {
            panic!("a path with backslashes did not survive into valid JSON: {e}\n{project}")
        });
        let source = parsed["tracks"][0]["elements"][0]["source"]
            .as_str()
            .expect("the source is a string");
        assert_eq!(
            source, windows_path,
            "the path came back out changed, so escaping it altered what it names"
        );
    }
}

/// One real mp3 from the fixture, copied beside a scratch project, and its true length.
///
/// 1776 ms, and the fixture declares exactly that on four elements — so the number this
/// returns is one the committed project also depends on.
fn scratch_with_audio(line: u32) -> (PathBuf, i64) {
    let dir = common::tempdir(line);
    std::fs::copy(
        fixture_dir().join("audio/05-cobweb.mp3"),
        dir.join("cobweb.mp3"),
    )
    .expect("copy real media beside the project");
    (dir, 1776)
}

fn codes(report: &Report) -> Vec<&str> {
    report.findings.iter().map(|f| f.code.as_str()).collect()
}

// ---------------------------------------------------------------------------
// The fixture agrees with its own media.
// ---------------------------------------------------------------------------

#[test]
fn the_committed_fixture_agrees_with_the_media_beside_it() {
    if !has_ffprobe() {
        return;
    }
    let report = validate(&fixture_dir().join("en-halloween-decorating.montaget.json"));

    assert!(
        report.findings.is_empty(),
        "the fixture is a published video; a finding on it is a defect in the check: {:?}",
        report.findings
    );
    assert_eq!(report.exit_code(), ExitCode::Ok);
    assert_eq!(
        report.misses.len(),
        16,
        "every distinct source is probed once — 16 of them, deduplicated from 60 elements"
    );
}

#[test]
fn a_range_ending_exactly_at_the_last_millisecond_is_legal() {
    if !has_ffprobe() {
        return;
    }
    // ADR-0005's half-open interval: `source_end` is exclusive, so a range ending exactly
    // at the source's length consumes all of it and reaches past nothing. This is not a
    // hypothetical — it is the shape of all 20 of the fixture's audio elements, and a
    // check that got it wrong would condemn the committed project.
    let (dir, duration) = scratch_with_audio(std::panic::Location::caller().line());
    let path = write_project(
        &dir,
        "p.montaget.json",
        &audio_project("cobweb.mp3", 0, duration),
    );

    let report = validate(&path);
    assert!(report.findings.is_empty(), "{:?}", report.findings);
}

// ---------------------------------------------------------------------------
// `E-SOURCE-OVERRUN`.
// ---------------------------------------------------------------------------

#[test]
fn a_source_range_reaching_past_the_file_is_an_error_with_both_numbers_inline() {
    if !has_ffprobe() {
        return;
    }
    // One millisecond past the end — the smallest possible overrun, so the test cannot
    // pass by accident on a check with a sloppy comparison.
    let (dir, duration) = scratch_with_audio(std::panic::Location::caller().line());
    let path = write_project(
        &dir,
        "p.montaget.json",
        &audio_project("cobweb.mp3", 0, duration + 1),
    );

    let report = validate(&path);

    assert_eq!(codes(&report), ["E-SOURCE-OVERRUN"]);
    let finding = &report.findings[0];
    assert_eq!(finding.class, Class::Error);
    assert_eq!(finding.location.element.as_deref(), Some("vo-01"));

    // ADR-0006: every relevant number inline, because "that re-read, not the validator's
    // output, is the real context cost".
    assert_eq!(finding.fields["probed_duration"], duration);
    assert_eq!(finding.fields["source_end"], duration + 1);
    assert_eq!(finding.fields["over_by"], 1);
    assert_eq!(
        finding.fields["axis"], "audio stream",
        "ADR-0011 returns four durations and forces a pick; the finding says which it made"
    );

    // ADR-0043: which of the document and the disk is the mistake is not readable off
    // either, so there is no repair to state.
    assert_eq!(
        finding.repair,
        Some(montaget_core::finding::Repair::None),
        "refuse-class"
    );
    assert_eq!(report.exit_code(), ExitCode::Errors);
}

#[test]
fn the_overrun_finding_renders_every_number_it_carries() {
    if !has_ffprobe() {
        return;
    }
    let (dir, duration) = scratch_with_audio(std::panic::Location::caller().line());
    let path = write_project(
        &dir,
        "p.montaget.json",
        &audio_project("cobweb.mp3", 0, duration + 800),
    );

    let rendered = montaget_core::wire::render(
        &validate(&path),
        montaget_core::Wire::Text { verbose: false },
    );

    assert!(rendered.contains("1776 ms (audio stream)"), "{rendered}");
    assert!(rendered.contains("800 ms past it"), "{rendered}");
    assert!(rendered.contains("cobweb.mp3"), "{rendered}");
}

// ---------------------------------------------------------------------------
// A missing local source, against a network failure (ADR-0053, ADR-0056).
// ---------------------------------------------------------------------------

#[test]
fn a_missing_local_source_is_a_plain_error_and_never_a_network_unknown() {
    if !has_ffprobe() {
        return;
    }
    // ADR-0053: "a missing `source` file is a plain `error`", and #203 exists partly so a
    // typo in a path is not confused with a network problem. The finding carries both the
    // spelling the document used and the path that was actually looked for.
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(
        &dir,
        "p.montaget.json",
        &audio_project("audio/typo.mp3", 0, 1000),
    );

    let report = validate(&path);

    assert_eq!(codes(&report), ["E-SOURCE-MISSING"]);
    let finding = &report.findings[0];
    assert_eq!(finding.class, Class::Error);
    assert_eq!(finding.reason, None, "nothing about a network was involved");
    assert_eq!(finding.fields["source"], "audio/typo.mp3");
    assert!(
        with_forward_slashes(finding.fields["resolved"].as_str().unwrap())
            .ends_with("audio/typo.mp3"),
        "{finding:?}"
    );
}

// ---------------------------------------------------------------------------
// Every source, every run (ADR-0006).
// ---------------------------------------------------------------------------

#[test]
fn every_referenced_source_is_probed_however_many_elements_share_one() {
    if !has_ffprobe() {
        return;
    }
    // ADR-0006 rejected scoping the probe to "the sources this write touched" partly on
    // this shape: in the fixture "every media source is referenced by 2–4 elements", so
    // "I already checked that file" is wrong the moment you look elsewhere. Two elements,
    // one file: both are checked, and the file is probed once.
    let (dir, duration) = scratch_with_audio(std::panic::Location::caller().line());
    let body = format!(
        r##"{{"frame":{{"width":1080,"height":1920}},"fps":25,"tracks":[{{"name":"vo","layer":1,"elements":[
            {{"id":"vo-ok","type":"audio","start":0,"end":{duration},"source":"cobweb.mp3","source_start":0,"source_end":{duration}}},
            {{"id":"vo-over","type":"audio","start":2000,"end":4000,"source":"cobweb.mp3","source_start":0,"source_end":{over}}}
        ]}}]}}"##,
        over = duration + 500
    );
    let path = write_project(&dir, "p.montaget.json", &body);

    let report = validate(&path);

    assert_eq!(
        codes(&report),
        ["E-SOURCE-OVERRUN"],
        "the second element is checked against the same file the first one passed on"
    );
    assert_eq!(
        report.findings[0].location.element.as_deref(),
        Some("vo-over")
    );
    assert_eq!(
        report.misses.len(),
        1,
        "and the file is probed once, not once per element"
    );
}

#[test]
fn the_cache_miss_reaches_the_report_unprompted() {
    if !has_ffprobe() {
        return;
    }
    // ADR-0006: "then report the cache miss, unprompted, at the top". ADR-0011 upgrades it
    // to "the sole mechanism" catching a source that grew on disk, so it is not behind
    // `--verbose` and does not collapse into a count.
    let (dir, duration) = scratch_with_audio(std::panic::Location::caller().line());
    let path = write_project(
        &dir,
        "p.montaget.json",
        &audio_project("cobweb.mp3", 0, duration),
    );

    let report = validate(&path);
    assert_eq!(report.misses.len(), 1);

    let rendered = montaget_core::wire::render(
        &report,
        // Not verbose: the whole point is that it prints without being asked.
        montaget_core::Wire::Text { verbose: false },
    );
    assert!(rendered.contains("CACHE"), "{rendered}");
    assert!(rendered.contains("cobweb.mp3"), "{rendered}");
}

// ---------------------------------------------------------------------------
// ADR-0053's path resolution.
// ---------------------------------------------------------------------------

#[test]
fn a_relative_source_resolves_against_the_project_file_and_not_the_working_directory() {
    if !has_ffprobe() {
        return;
    }
    // ADR-0053 rejected `assetRoot` and left the project file's own directory as the one
    // thing a relative path resolves against — "a project is a movable unit: the
    // `.montaget.json` file plus its relative media siblings". The test is that the same
    // project validates identically from anywhere, so it is run against a path that shares
    // no prefix with the working directory.
    let (dir, duration) = scratch_with_audio(std::panic::Location::caller().line());
    std::fs::create_dir_all(dir.join("nested")).unwrap();
    std::fs::copy(dir.join("cobweb.mp3"), dir.join("nested/cobweb.mp3")).unwrap();
    let path = write_project(
        &dir.join("nested"),
        "p.montaget.json",
        &audio_project("cobweb.mp3", 0, duration),
    );

    let report = validate(&path);

    assert!(
        report.findings.is_empty(),
        "the sibling beside the project file is the one that resolves: {:?}",
        report.findings
    );
    assert!(
        report.misses[0].source.contains("nested"),
        "and it is the nested copy that was probed: {:?}",
        report.misses
    );
}

#[test]
fn an_absolute_source_is_permitted() {
    if !has_ffprobe() {
        return;
    }
    // ADR-0053, 2-1: "an agent pointing at `/Volumes/footage/take3.mov` on its own
    // machine, or a shared NAS mount, is a legitimate authoring state", and `validate`
    // does not police portability as a property of the format.
    let (dir, duration) = scratch_with_audio(std::panic::Location::caller().line());
    let absolute = dir.join("cobweb.mp3");
    let path = write_project(
        &dir,
        "p.montaget.json",
        &audio_project(&absolute.display().to_string(), 0, duration),
    );

    let report = validate(&path);
    assert!(report.findings.is_empty(), "{:?}", report.findings);
}

// ---------------------------------------------------------------------------
// A project with no media at all.
// ---------------------------------------------------------------------------

#[test]
fn a_project_referencing_no_media_needs_no_ffmpeg() {
    // The one thing that must not become a fast mode: this skips a *tool Montaget has
    // nothing to ask*, never a source it has something to ask about. A header-only project
    // has no source to probe, so opening a session for it would turn "you have no FFmpeg"
    // into a failure of a run that was never going to spawn anything.
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(
        &dir,
        "p.montaget.json",
        r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[]}"##,
    );

    let report = validate(&path);
    assert!(report.findings.is_empty(), "{:?}", report.findings);
    assert!(report.misses.is_empty());
}

// ---------------------------------------------------------------------------
// Which of ADR-0011's four durations the check measures against.
// ---------------------------------------------------------------------------

/// A one-element video project pointing at an absolute source.
fn video_project(source: &str, source_end: i64) -> String {
    format!(
        r##"{{"frame":{{"width":1080,"height":1920}},"fps":25,"tracks":[{{"name":"clip","layer":1,"elements":[{{"id":"clip-01","type":"video","start":0,"end":{source_end},"source":{source},"source_start":0,"source_end":{source_end},"x":0,"y":0,"origin":"top-left","width":1080,"height":1920,"fit":"cover"}}]}}]}}"##,
        source = json_string(source)
    )
}

#[test]
fn the_check_measures_against_the_video_stream_and_not_the_container() {
    if !has_ffprobe() {
        return;
    }
    // ADR-0011's whole reason for the quad, as a check. On the fixture's reference MP4 the
    // container says 65258 ms and the video stream says 65216 ms, and the project's own
    // `duration: 65216` agrees with the stream. A range ending at 65230 therefore reaches
    // past the decodable content while still sitting inside the container — so the two
    // axes give *different verdicts on the same document*, and this pins which one the
    // check asks. Measuring against the container here would silently pass a range whose
    // last 14 ms the renderer cannot decode.
    let reference = fixture_dir().join("reference/en-halloween-decorating.mp4");
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(
        &dir,
        "p.montaget.json",
        &video_project(&reference.display().to_string(), 65230),
    );

    let report = validate(&path);

    assert_eq!(codes(&report), ["E-SOURCE-OVERRUN"]);
    let finding = &report.findings[0];
    assert_eq!(finding.fields["probed_duration"], 65216, "the video stream");
    assert_eq!(finding.fields["axis"], "video stream");
    assert_eq!(finding.fields["over_by"], 14);
}

#[test]
fn a_range_inside_the_video_stream_is_legal_even_though_the_container_is_longer() {
    if !has_ffprobe() {
        return;
    }
    // The other side of the same pick: 65216 is the stream's exact length, so the range
    // consumes all of it and reaches past nothing.
    let reference = fixture_dir().join("reference/en-halloween-decorating.mp4");
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(
        &dir,
        "p.montaget.json",
        &video_project(&reference.display().to_string(), 65216),
    );

    assert!(validate(&path).findings.is_empty());
}

// ---------------------------------------------------------------------------
// The facts themselves (#203's first acceptance criterion).
// ---------------------------------------------------------------------------

#[test]
fn a_clean_run_reports_the_duration_and_dimensions_it_established() {
    if !has_ffprobe() {
        return;
    }
    // "Every referenced source is probed and its real duration and dimensions reported."
    // A run that measured the numbers and printed none of them would leave the reader to
    // re-derive them — and the fit checks downstream consume exactly these rather than
    // probing a second time.
    let report = validate(&fixture_dir().join("en-halloween-decorating.montaget.json"));

    assert_eq!(report.media.len(), 16, "one entry per distinct source");

    let image = report
        .media
        .iter()
        .find(|probe| with_forward_slashes(&probe.source).ends_with("images/06.png"))
        .expect("the image is among them");
    let dimensions = image.dimensions.expect("an image has dimensions");
    assert_eq!((dimensions.width, dimensions.height), (1536, 2720));

    let audio = report
        .media
        .iter()
        .find(|probe| with_forward_slashes(&probe.source).ends_with("audio/05-cobweb.mp3"))
        .expect("the audio is among them");
    assert_eq!(
        audio.audio.and_then(|a| a.audio_stream_ms),
        Some(1776),
        "named for the axis it measures, never collapsed to `duration`"
    );
}

#[test]
fn the_facts_print_under_verbose_and_stay_in_the_json_either_way() {
    if !has_ffprobe() {
        return;
    }
    // ADR-0006 scopes the *output*, never the analysis: "a checked-but-unprinted finding
    // still exists; an unchecked one silently does not". Sixteen media blocks on every
    // routine run is the noise budget that ADR spends a page on, so they print on request
    // — and the canonical JSON carries them regardless, which is what keeps this a
    // filter on printing rather than on looking.
    let (dir, duration) = scratch_with_audio(std::panic::Location::caller().line());
    let path = write_project(
        &dir,
        "p.montaget.json",
        &audio_project("cobweb.mp3", 0, duration),
    );
    let report = validate(&path);

    let quiet = montaget_core::wire::render(&report, montaget_core::Wire::Text { verbose: false });
    assert!(!quiet.contains("MEDIA"), "{quiet}");

    let loud = montaget_core::wire::render(&report, montaget_core::Wire::Text { verbose: true });
    assert!(loud.contains("MEDIA"), "{loud}");
    assert!(loud.contains("1776 ms"), "{loud}");

    let json = montaget_core::wire::render(&report, montaget_core::Wire::Json);
    assert!(json.contains("\"media\""), "{json}");
    assert!(json.contains("1776"), "{json}");
}

// ---------------------------------------------------------------------------
// Numbers the finding must not invent (ADR-0045).
// ---------------------------------------------------------------------------

/// An audio project carrying whatever `speed` spelling is under test.
fn audio_project_at_speed(source: &str, source_start: i64, source_end: i64, speed: &str) -> String {
    format!(
        r##"{{"frame":{{"width":1080,"height":1920}},"fps":25,"tracks":[{{"name":"vo","layer":1,"elements":[{{"id":"vo-01","type":"audio","start":0,"end":100,"source":"{source}","source_start":{source_start},"source_end":{source_end},"speed":{speed}}}]}}]}}"##
    )
}

#[test]
fn the_overrun_finding_states_no_number_that_speed_would_have_decided() {
    if !has_ffprobe() {
        return;
    }
    // ADR-0045's own minimal counterexample: a 7 ms span at `speed: 0.560` is exactly 12.5
    // in decimal, so round-half-up gives 13, while `f64` computes 12.499999999999998 and
    // gives 12. An earlier version of this check divided in `f64` and printed 12 — a number
    // #197's exact check would contradict on the same element.
    //
    // The fix is not a more careful division here: it is that this finding has no business
    // doing one. Whether a source range names bytes the file holds is a question about the
    // source alone, and the same range overruns by the same amount at any rate.
    let (dir, duration) = scratch_with_audio(std::panic::Location::caller().line());
    let path = write_project(
        &dir,
        "p.montaget.json",
        &audio_project_at_speed("cobweb.mp3", duration, duration + 7, "0.560"),
    );

    let report = validate(&path);
    let finding = &report.findings[0];

    assert_eq!(finding.code, "E-SOURCE-OVERRUN");
    assert_eq!(finding.fields["over_by"], 7);
    for invented in ["speed", "timeline_span"] {
        assert!(
            !finding.fields.contains_key(invented),
            "`{invented}` is ADR-0020's invariant, evaluated exactly by its own check \
             (ADR-0045) and not restated approximately here: {finding:?}"
        );
    }

    let rendered =
        montaget_core::wire::render(&report, montaget_core::Wire::Text { verbose: false });
    assert!(!rendered.contains(" 12 "), "the wrong answer: {rendered}");
}

#[test]
fn a_speed_the_document_does_not_carry_is_never_supplied_for_it() {
    if !has_ffprobe() {
        return;
    }
    // `speed: 0` is a schema error (ADR-0020 — it names a hold, not a rate) and a `speed`
    // that is a string is a schema error too; both belong to the checks that own the
    // schema. What must not happen is this check quietly reading either as `1.0`, or
    // dividing by zero and reporting a timeline span of i64::MAX. Both did happen before
    // the field went away.
    let (dir, duration) = scratch_with_audio(std::panic::Location::caller().line());
    for spelling in ["0", "\"0.645\"", "null", "-1"] {
        let path = write_project(
            &dir,
            "p.montaget.json",
            &audio_project_at_speed("cobweb.mp3", 0, duration + 100, spelling),
        );

        let report = validate(&path);
        let finding = &report.findings[0];
        assert_eq!(finding.code, "E-SOURCE-OVERRUN", "speed: {spelling}");
        assert_eq!(finding.fields["over_by"], 100, "speed: {spelling}");

        let rendered =
            montaget_core::wire::render(&report, montaget_core::Wire::Text { verbose: false });
        assert!(
            !rendered.contains("9223372036854775807"),
            "speed: {spelling} — {rendered}"
        );
        assert!(
            !rendered.contains("speed"),
            "speed: {spelling} — {rendered}"
        );
    }
}
