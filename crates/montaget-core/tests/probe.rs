//! `probe` — the one authority on what a media file's numbers are (#190).
//!
//! Every test here is one of the ticket's acceptance criteria, and most of them run
//! against a **recorded `ffprobe`** rather than a live one. That is not a convenience: the
//! behaviours that matter most — a 404 that must be an `error`, a DNS failure that must
//! not be, a local probe that must be incapable of reaching the network — are each a
//! property of what Montaget does with what `ffprobe` said, and reproducing them against
//! real servers would make this suite depend on the network it is asserting Montaget does
//! not touch.
//!
//! The tests that do run the real binary are the ones whose whole point is that the
//! numbers are real: ADR-0011's quad, read off the fixture the ADR itself measured.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use montaget_core::finding::{Class, UncheckedReason};
use montaget_core::media::probe::{Execution, Outcome, Runner};
use montaget_core::media::session::{MissKind, Session};
use montaget_core::media::tools::{self, Tools};
use montaget_core::media::{Rational, Source};
use montaget_core::verbs::probe as verb;

// ---------------------------------------------------------------------------
// A recorded `ffprobe`.
// ---------------------------------------------------------------------------

/// An `ffprobe` that answers from a script and records every call.
struct Recorded {
    /// One reply per call, consumed in order; the last one repeats.
    replies: Mutex<Vec<Execution>>,
    calls: Mutex<Vec<Vec<String>>>,
}

impl Recorded {
    fn new(replies: Vec<Execution>) -> Recorded {
        Recorded {
            replies: Mutex::new(replies),
            calls: Mutex::new(Vec::new()),
        }
    }

    fn ok(stdout: &str) -> Execution {
        Execution {
            success: true,
            code: Some(0),
            stdout: stdout.to_string(),
            stderr: String::new(),
        }
    }

    fn failed(stderr: &str) -> Execution {
        Execution {
            success: false,
            // What `ffprobe` itself answers for a file it cannot make sense of. A code
            // that means "the tool did not run" is a different test, below.
            code: Some(1),
            stdout: String::new(),
            stderr: stderr.to_string(),
        }
    }

    fn calls(&self) -> Vec<Vec<String>> {
        self.calls.lock().unwrap().clone()
    }
}

/// The recording as a `Runner`. A newtype because `Arc` is not a fundamental type, so the
/// impl cannot hang off `Arc<Recorded>` from outside the crate that defines `Runner`.
struct Shared(Arc<Recorded>);

impl Runner for Shared {
    fn run(&self, _program: &Path, args: &[String]) -> std::io::Result<Execution> {
        self.0.calls.lock().unwrap().push(args.to_vec());
        let mut replies = self.0.replies.lock().unwrap();
        Ok(if replies.len() > 1 {
            replies.remove(0)
        } else {
            replies.first().cloned().unwrap_or(Recorded::ok("{}"))
        })
    }
}

/// A session over a recorded `ffprobe`, and a handle on the recording.
///
/// Shared rather than borrowed: a `Session` owns its runner for its whole life, and the
/// test goes on reading what was recorded after handing it over.
fn recorded(replies: Vec<Execution>) -> (Arc<Recorded>, Session) {
    let recorder = Arc::new(Recorded::new(replies));
    let tools = Tools {
        ffmpeg: PathBuf::from("/nowhere/ffmpeg"),
        ffprobe: PathBuf::from("/nowhere/ffprobe"),
    };
    let session = Session::with(tools, Box::new(Shared(Arc::clone(&recorder))));
    (recorder, session)
}

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/en-halloween-decorating")
        .canonicalize()
        .expect("the committed fixture")
}

/// A live session, or `None` where this machine has no `ffprobe` — which is a legitimate
/// state, since ADR-0009 ships Montaget as *"a binary, plus an `ffmpeg` the user
/// supplies"*.
fn live() -> Option<Session> {
    match Session::open() {
        Ok(session) => Some(session),
        Err(missing) => {
            eprintln!("skipping: {}", missing.reason());
            None
        }
    }
}

// ---------------------------------------------------------------------------
// The quad.
// ---------------------------------------------------------------------------

#[test]
fn the_quad_comes_back_in_integer_milliseconds() {
    let Some(mut session) = live() else { return };
    let dir = fixture_dir();

    let outcome = session
        .probe(&Source::resolve(
            "reference/en-halloween-decorating.mp4",
            &dir,
        ))
        .expect("ffprobe runs");
    let quad = outcome.probe().expect("the reference MP4 probes").quad;

    // ADR-0011's own table, which is where this ticket's reason for existing comes from.
    assert_eq!(quad.video_stream_ms, Some(65216));
    assert_eq!(quad.container_ms, Some(65258));
    assert_eq!(quad.start_time_ms, Some(42));
    assert_eq!(quad.r_frame_rate, Some(Rational::new(50, 1)));
    assert_eq!(
        quad.avg_frame_rate,
        Some(Rational::new(1_390_080, 55_651)),
        "kept as the exact ratio it is, not as the 24.9785 it prints as"
    );
}

#[test]
fn the_quad_refuses_to_collapse_to_a_scalar_named_duration() {
    let Some(mut session) = live() else { return };
    let dir = fixture_dir();

    let outcome = session
        .probe(&Source::resolve(
            "reference/en-halloween-decorating.mp4",
            &dir,
        ))
        .expect("ffprobe runs");
    let probe = outcome.probe().expect("the reference MP4 probes");

    assert!(
        probe.quad.durations_disagree(),
        "the whole argument rests on these two numbers differing on this very file"
    );

    // ADR-0011: "A single scalar named `duration` is the failure mode, because it invites
    // arithmetic against the wrong axis." There is no such field, no such method, and —
    // asserted here rather than trusted — no such key anywhere in the wire form. Every
    // duration is named for the axis it measures.
    let json = serde_json::to_value(probe).expect("the probe serialises");
    let mut offenders = Vec::new();
    collect_keys(&json, &mut offenders);
    assert!(
        !offenders.iter().any(|key| key == "duration"),
        "a key named `duration` reached the wire: {offenders:?}"
    );
    assert!(
        offenders.iter().any(|key| key == "video_stream_ms"),
        "the axes are named: {offenders:?}"
    );
}

fn collect_keys(value: &serde_json::Value, into: &mut Vec<String>) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, child) in map {
                into.push(key.clone());
                collect_keys(child, into);
            }
        }
        serde_json::Value::Array(items) => items.iter().for_each(|item| collect_keys(item, into)),
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// ADR-0023: one pipeline for images and video.
// ---------------------------------------------------------------------------

#[test]
fn an_image_and_a_video_resolve_dimensions_through_the_same_code_path() {
    let Some(mut session) = live() else { return };
    let dir = fixture_dir();

    let image = session
        .probe(&Source::resolve("images/06.png", &dir))
        .expect("ffprobe runs");
    let video = session
        .probe(&Source::resolve(
            "reference/en-halloween-decorating.mp4",
            &dir,
        ))
        .expect("ffprobe runs");

    let image = image
        .probe()
        .unwrap()
        .dimensions
        .expect("an image has them");
    let video = video.probe().unwrap().dimensions.expect("a video has them");

    // ADR-0011 cites `images/06.png` at 1536×2720, and the fixture's render is portrait.
    assert_eq!((image.width, image.height), (1536, 2720));
    assert_eq!((video.width, video.height), (1080, 1920));

    // The degenerate case is the same case: PAR 1:1, no rotation, and the same fields
    // carrying the same account of how the number was reached.
    assert_eq!(image.par, Rational::ONE);
    assert_eq!(image.rotation.degrees, 0);
    assert_eq!(image.decoded.width, 1536);
    assert_eq!(video.par, Rational::ONE);
}

#[test]
fn rotation_and_par_are_applied_in_that_order_and_round_once() {
    // A recorded portrait clip with non-square pixels: the case the fixture does not have
    // and ADR-0023 says a synthetic one is sufficient to exercise.
    let stream = r#"{
        "streams": [{
            "codec_type": "video",
            "width": 720,
            "height": 576,
            "sample_aspect_ratio": "16:15",
            "pix_fmt": "yuv420p",
            "r_frame_rate": "25/1",
            "avg_frame_rate": "25/1",
            "duration": "10.000000",
            "side_data_list": [{"side_data_type": "Display Matrix", "rotation": -90}]
        }],
        "format": {"duration": "10.000000"}
    }"#;
    let (_recorder, mut session) = recorded(vec![Recorded::ok(stream)]);
    // A real file, because a probe answers a missing one from `stat` and never reaches
    // the decoder. Its bytes are irrelevant: the recorded `ffprobe` supplies the stream.
    let path = std::env::temp_dir().join("montaget-probe-portrait.mov");
    std::fs::write(&path, b"stand-in").unwrap();

    let outcome = session
        .probe(&Source::Local(path.clone()))
        .expect("the recorded ffprobe runs");
    std::fs::remove_file(&path).ok();
    let dimensions = outcome.probe().unwrap().dimensions.unwrap();

    assert_eq!(
        dimensions.rotation.degrees, 270,
        "the container's transform"
    );
    assert_eq!(
        dimensions.rotation.source,
        montaget_core::media::dimensions::RotationSource::DisplayMatrix,
        "ADR-0023 requires the report to say which rotation source was applied, so the \
         resolved dimensions carry it"
    );
    assert_eq!(
        (dimensions.width, dimensions.height),
        (614, 720),
        "rotate first (576×720), then PAR along the displayed width (576 × 16/15 = 614.4), \
         floored exactly once"
    );
    assert_eq!(
        dimensions.exact_width,
        Rational::new(9216, 15),
        "the exact rational survives to `fit`, which does its own single rounding"
    );
}

// ---------------------------------------------------------------------------
// The cache, and the miss that is the point of it.
// ---------------------------------------------------------------------------

#[test]
fn a_second_probe_of_an_unchanged_file_does_no_work_and_reports_no_miss() {
    let (recorder, mut session) = recorded(vec![Recorded::ok(
        r#"{"streams":[{"codec_type":"audio","sample_rate":"24000","channels":1,"duration":"1.776000"}],"format":{"duration":"1.776000"}}"#,
    )]);
    let source = Source::resolve("audio/05-cobweb.mp3", &fixture_dir());

    let first = session.probe(&source).unwrap();
    let second = session.probe(&source).unwrap();

    assert_eq!(first, second);
    assert_eq!(recorder.calls().len(), 1, "the second probe did no I/O");
    assert_eq!(
        session.misses().len(),
        1,
        "one miss, for the probe that actually ran"
    );
    assert_eq!(session.misses()[0].kind, MissKind::First);
}

#[test]
fn a_file_that_changed_on_disk_is_announced_unprompted() {
    // ADR-0011 upgrades this line from an optimisation's side effect to "the sole
    // mechanism" catching a source that grew: the document records no source duration, so
    // a file that gained 800 ms while the project used its first two seconds is invisible
    // to every check constructible from the document plus the disk.
    let (_recorder, mut session) = recorded(vec![Recorded::ok(
        r#"{"streams":[{"codec_type":"audio","sample_rate":"24000","channels":1,"duration":"1.776000"}],"format":{"duration":"1.776000"}}"#,
    )]);

    let path = std::env::temp_dir().join("montaget-probe-changed.mp3");
    std::fs::write(&path, b"one").unwrap();
    let source = Source::Local(path.clone());
    session.probe(&source).unwrap();

    // Longer bytes, and a fresh mtime: the two halves of ADR-0006's key that the tool
    // observes for itself.
    std::thread::sleep(std::time::Duration::from_millis(10));
    std::fs::write(&path, b"one and a half").unwrap();
    session.probe(&source).unwrap();

    let miss = session.misses().last().expect("the change is a miss");
    match miss.kind {
        MissKind::Changed {
            previous_size,
            size,
            ..
        } => {
            assert_eq!(previous_size, 3);
            assert_eq!(size, 14);
        }
        ref other => panic!("a changed file must announce itself, got {other:?}"),
    }

    std::fs::remove_file(&path).ok();
}

#[test]
fn the_cache_miss_prints_at_the_top_without_being_asked_for() {
    let (_recorder, mut session) = recorded(vec![Recorded::ok(
        r#"{"streams":[{"codec_type":"video","width":1536,"height":2720,"pix_fmt":"rgb24","r_frame_rate":"25/1"}],"format":{}}"#,
    )]);

    let answer = verb::probe_with(&mut session, &fixture_dir(), &["images/06.png".to_string()])
        .expect("the recorded ffprobe works");
    // Not `--verbose`, which is the whole point: ADR-0006 asks for it unprompted.
    let rendered =
        montaget_core::wire::render_answer(&answer, montaget_core::Wire::Text { verbose: false });

    assert!(rendered.starts_with("CACHE\n"), "{rendered}");
    assert!(rendered.contains("images/06.png"), "{rendered}");
}

// ---------------------------------------------------------------------------
// Remote sources (ADR-0056).
// ---------------------------------------------------------------------------

const REMOTE_OK: &str = r#"{
    "streams": [{"codec_type":"video","width":1920,"height":1080,"pix_fmt":"yuv420p","r_frame_rate":"25/1","avg_frame_rate":"25/1","duration":"12.000000"}],
    "format": {"duration": "12.041000"}
}"#;

#[test]
fn one_remote_url_is_fetched_once_per_run_however_many_elements_name_it() {
    // ADR-0056: "a project referencing one remote clip from five elements probes that URL
    // once per run, not five times."
    let (recorder, mut session) = recorded(vec![Recorded::ok(REMOTE_OK)]);
    let url = "https://cdn.example/take3.mov";

    let answer = verb::probe_with(&mut session, Path::new("/p"), &vec![url.to_string(); 5])
        .expect("the recorded ffprobe works");

    assert_eq!(recorder.calls().len(), 1, "deduplicated by URL");
    assert_eq!(session.network_attempts(), 1);
    assert_eq!(
        answer.sources.len(),
        5,
        "every element still gets an answer"
    );
}

#[test]
fn nothing_a_remote_probe_learned_survives_the_run() {
    // The 1-2 split ADR-0056's author broke for no persistent cache: "no cache entry for a
    // remote source survives past the `validate` process that created it."
    let (recorder, mut session) = recorded(vec![Recorded::ok(REMOTE_OK)]);
    let url = Source::Remote("https://cdn.example/take3.mov".to_string());

    session.begin_run();
    session.probe(&url).unwrap();
    session.probe(&url).unwrap();
    assert_eq!(recorder.calls().len(), 1, "within one run, deduplicated");

    session.begin_run();
    session.probe(&url).unwrap();
    assert_eq!(
        recorder.calls().len(),
        2,
        "a fresh run pays a fresh round trip rather than trusting a server's word"
    );
}

#[test]
fn a_network_failure_is_unchecked_with_a_reason_and_never_a_confirmed_absence() {
    let cases: Vec<(&str, UncheckedReason)> = vec![
        (
            "[tcp @ 0x1] Failed to resolve hostname cdn.example: Name or service not known",
            UncheckedReason::Dns,
        ),
        (
            "[tcp @ 0x1] Connection to tcp://cdn.example:443 failed: Operation timed out",
            UncheckedReason::Timeout,
        ),
        (
            "[tcp @ 0x1] Connection to tcp://cdn.example:443 failed: Connection refused",
            UncheckedReason::Unreachable,
        ),
        (
            "[http @ 0x1] Server returned 503 Service Unavailable",
            UncheckedReason::Http { status: 503 },
        ),
    ];

    for (stderr, expected) in cases {
        let (_recorder, mut session) = recorded(vec![Recorded::failed(stderr)]);
        let outcome = session
            .probe(&Source::Remote("https://cdn.example/take3.mov".into()))
            .unwrap();

        match &outcome {
            Outcome::Unchecked { reason, .. } => {
                assert_eq!(reason.as_ref(), Some(&expected), "{stderr}")
            }
            other => panic!("{stderr} must not be read as {other:?}"),
        }

        let finding = verb::finding_for(&outcome).expect("an unknown is reported, not dropped");
        assert_eq!(finding.code, "U-SOURCE-UNPROBEABLE");
        assert_eq!(finding.class, Class::Unchecked);
        assert_eq!(
            finding.reason.as_ref(),
            Some(&expected),
            "ADR-0056's reason is mandatory and structured"
        );
    }
}

#[test]
fn a_server_that_refuses_the_object_is_the_plain_error_it_always_was() {
    // The other side of the same line: ADR-0053's confirmed absence is unchanged by
    // ADR-0056, and the two must never be conflated in either direction.
    let (_recorder, mut session) = recorded(vec![Recorded::failed(
        "[http @ 0x1] HTTP error 404 Not Found\nServer returned 404 Not Found",
    )]);

    let outcome = session
        .probe(&Source::Remote("https://cdn.example/gone.mov".into()))
        .unwrap();
    assert!(matches!(outcome, Outcome::Missing { .. }));

    let finding = verb::finding_for(&outcome).unwrap();
    assert_eq!(finding.code, "E-SOURCE-MISSING");
    assert_eq!(finding.class, Class::Error);
}

#[test]
fn a_server_that_will_not_serve_a_partial_read_degrades_to_existence_only() {
    // ADR-0056: the probe attempts real content verification "before falling back to an
    // existence-only check … for a URL or protocol that won't support partial reads". Each
    // of these statuses is a server answering *about the object* — so the object is there,
    // and its duration is not known.
    for stderr in [
        "[http @ 0x1] Server returned 416 Range Not Satisfiable",
        "[http @ 0x1] Server returned 405 Method Not Allowed",
        "[http @ 0x1] Server returned 501 Not Implemented",
    ] {
        let (_recorder, mut session) = recorded(vec![Recorded::failed(stderr)]);
        let outcome = session
            .probe(&Source::Remote("https://cdn.example/take3.mov".into()))
            .unwrap();

        match &outcome {
            Outcome::ExistenceOnly { detail, .. } => {
                assert!(detail.contains("NOT CHECKED"), "{detail}")
            }
            other => panic!("{stderr} establishes existence, not {other:?}"),
        }
        assert_eq!(
            verb::finding_for(&outcome).unwrap().code,
            "U-SOURCE-EXISTENCE-ONLY",
            "never the slot a confirmed duration occupies, and never a missing-source error"
        );
    }
}

#[test]
fn an_existence_only_answer_never_occupies_a_confirmed_durations_slot() {
    // ADR-0056: the degradation "is never silent" — it says exactly what it established,
    // under its own code.
    let (_recorder, mut session) = recorded(vec![Recorded::ok(r#"{"streams":[],"format":{}}"#)]);

    let outcome = session
        .probe(&Source::Remote("https://cdn.example/opaque.bin".into()))
        .unwrap();
    assert!(matches!(outcome, Outcome::ExistenceOnly { .. }));
    assert!(
        outcome.probe().is_none(),
        "it carries no quad, because none was established"
    );

    let finding = verb::finding_for(&outcome).unwrap();
    assert_eq!(finding.code, "U-SOURCE-EXISTENCE-ONLY");
    assert_eq!(finding.class, Class::Unchecked);
    assert_eq!(
        finding.reason, None,
        "nothing about the network failed, so there is no network reason to give"
    );
}

// ---------------------------------------------------------------------------
// The invariant this ticket exists to fence: no unsolicited network call.
// ---------------------------------------------------------------------------

#[test]
fn a_project_of_local_sources_never_reaches_the_network() {
    let (recorder, mut session) = recorded(vec![Recorded::ok(
        r#"{"streams":[{"codec_type":"video","width":1536,"height":2720,"pix_fmt":"rgb24","r_frame_rate":"25/1"}],"format":{}}"#,
    )]);
    let dir = fixture_dir();

    let answer = verb::probe_with(
        &mut session,
        &dir,
        &[
            "images/06.png".to_string(),
            "audio/05-cobweb.mp3".to_string(),
            "reference/en-halloween-decorating.mp4".to_string(),
            // ADR-0053 permits an absolute local path, and it is still local.
            dir.join("images/07.png").display().to_string(),
        ],
    )
    .expect("the recorded ffprobe works");

    assert_eq!(answer.network_attempts, 0);
    assert_eq!(session.network_attempts(), 0);

    for args in recorder.calls() {
        let whitelist = protocol_whitelist(&args);
        assert_eq!(
            whitelist.as_deref(),
            Some("file,crypto,data"),
            "a local probe is told which protocols exist, so a file that names a remote \
             resource inside itself cannot turn it into a fetch: {args:?}"
        );
        assert!(
            !args.iter().any(|arg| arg.contains("http")),
            "no local probe argument mentions a network protocol: {args:?}"
        );
    }
}

#[test]
fn only_a_source_the_document_spells_as_a_url_may_reach_the_network() {
    let (recorder, mut session) = recorded(vec![Recorded::ok(REMOTE_OK)]);

    session
        .probe(&Source::Remote("https://cdn.example/take3.mov".into()))
        .unwrap();

    assert_eq!(session.network_attempts(), 1);
    let args = recorder.calls().pop().expect("one call");
    assert_eq!(
        protocol_whitelist(&args).as_deref(),
        Some("file,crypto,data,http,https,tcp,tls")
    );
    assert!(
        args.iter().any(|arg| arg == "-rw_timeout"),
        "an unbounded round trip would hang the run rather than reporting a timeout: {args:?}"
    );
}

fn protocol_whitelist(args: &[String]) -> Option<String> {
    let index = args.iter().position(|arg| arg == "-protocol_whitelist")?;
    args.get(index + 1).cloned()
}

// ---------------------------------------------------------------------------
// FFmpeg resolution.
// ---------------------------------------------------------------------------

#[test]
fn a_missing_ffmpeg_is_exit_70_naming_what_was_looked_for() {
    let empty = std::env::temp_dir().join("montaget-empty-path");
    std::fs::create_dir_all(&empty).unwrap();

    let missing = tools::resolve_in(Some(empty.as_os_str())).expect_err("nothing to find");
    let report = missing.clone().into_report();

    assert_eq!(
        report.exit_code(),
        montaget_core::report::ExitCode::Internal,
        "ADR-0011: exit 70 is for Montaget breaking, and a short PATH is not a defect in \
         the project"
    );
    let rendered =
        montaget_core::wire::render(&report, montaget_core::Wire::Text { verbose: false });
    assert!(rendered.contains("ffmpeg"), "{rendered}");
    assert!(
        rendered.contains(&empty.display().to_string()),
        "{rendered}"
    );
}

#[test]
fn a_local_file_that_is_not_there_is_answered_without_spawning_anything() {
    // ADR-0053's confirmed absence is a fact about the filesystem, not something to infer
    // from a decoder's prose — so no subprocess runs to establish it.
    let (recorder, mut session) = recorded(vec![Recorded::ok("{}")]);

    let outcome = session
        .probe(&Source::Local(PathBuf::from("/p/not-here.mov")))
        .unwrap();

    assert!(matches!(outcome, Outcome::Missing { .. }));
    assert!(recorder.calls().is_empty());
    assert_eq!(session.network_attempts(), 0);
}

// ---------------------------------------------------------------------------
// A tool that will not do the job is exit 70, not a fact about the media.
// ---------------------------------------------------------------------------

#[test]
fn an_ffprobe_that_rejects_our_invocation_is_exit_70_and_never_a_finding() {
    // ADR-0009 ships Montaget as "a binary, plus an `ffmpeg` the user supplies", so an
    // `ffprobe` too old to know `-protocol_whitelist` is the likeliest field failure there
    // is. Reporting it as `U-SOURCE-UNPROBEABLE` would blame the media for a broken tool,
    // exit 0 on it, and — because the rejected flag is the network fence — quietly report
    // "nothing learned" for every source in the project.
    let (_recorder, mut session) = recorded(vec![Execution {
        success: false,
        code: Some(1),
        stdout: String::new(),
        stderr: "Unrecognized option 'protocol_whitelist'.\n\
                 Error splitting the argument list: Option not found"
            .to_string(),
    }]);

    let missing = session
        .probe(&Source::resolve("images/06.png", &fixture_dir()))
        .expect_err("a tool that rejects our invocation is not a media fact");

    assert_eq!(missing.program, "ffprobe");
    assert_eq!(
        missing.resolved.as_deref(),
        Some(Path::new("/nowhere/ffprobe")),
        "exit 70 names the resolved path, which is the thing to go and look at"
    );
    assert!(
        missing.reason().contains("protocol_whitelist"),
        "{}",
        missing.reason()
    );
    assert_eq!(
        missing.into_report().exit_code(),
        montaget_core::report::ExitCode::Internal
    );
}

#[test]
fn an_ffprobe_that_answers_with_something_other_than_json_is_exit_70() {
    // A zero-byte executable on `PATH` exits 0 and says nothing. `-print_format json` was
    // not honoured, so whatever ran is not an `ffprobe`; that is a broken tool, and
    // "ffprobe reported no streams" would be a claim about media nobody looked at.
    let (_recorder, mut session) = recorded(vec![Execution {
        success: true,
        code: Some(0),
        stdout: String::new(),
        stderr: String::new(),
    }]);

    let missing = session
        .probe(&Source::resolve("images/06.png", &fixture_dir()))
        .expect_err("silence is not an answer about the media");
    assert!(
        missing.reason().contains("not the JSON"),
        "{}",
        missing.reason()
    );
}

#[test]
fn an_ffprobe_the_os_could_not_run_is_exit_70() {
    // `garbage +x` on PATH: the shell answers 127, not `ffprobe`. ADR-0011's exit 70 is
    // literally "ffmpeg died".
    for code in [Some(126), Some(127), None] {
        let (_recorder, mut session) = recorded(vec![Execution {
            success: false,
            code,
            stdout: String::new(),
            stderr: "ffprobe: line 1: not: command not found".to_string(),
        }]);

        let missing = session
            .probe(&Source::resolve("images/06.png", &fixture_dir()))
            .unwrap_err();
        assert_eq!(missing.program, "ffprobe", "{code:?}");
    }
}

#[test]
fn a_file_that_will_not_decode_is_still_the_medias_fault_and_not_the_tools() {
    // The non-regression that matters: this is real `ffprobe`'s own answer for a file whose
    // bytes are not media (exit 1, `moov atom not found`, an empty JSON object on stdout).
    // Nothing is wrong with the tool, so this must stay `UNCHECKED` at exit 0 — the
    // discrimination is the point, and a rule that swept both into exit 70 would be the
    // same collapse in the other direction.
    let (_recorder, mut session) = recorded(vec![Execution {
        success: false,
        code: Some(1),
        stdout: "{\n\n}".to_string(),
        stderr: "[mov,mp4,m4a,3gp,3g2,mj2 @ 0x1] moov atom not found\n\
                 garbage.mp4: Invalid data found when processing input"
            .to_string(),
    }]);

    let outcome = session
        .probe(&Source::resolve("images/06.png", &fixture_dir()))
        .expect("a media failure is an answer, not a broken tool");

    match outcome {
        Outcome::Unchecked { reason, detail, .. } => {
            assert_eq!(reason, None, "nothing about the network failed");
            assert!(detail.contains("Invalid data"), "{detail}");
        }
        other => panic!("{other:?}"),
    }
}
