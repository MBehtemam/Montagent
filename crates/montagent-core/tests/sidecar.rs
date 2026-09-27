//! The probe sidecar — the persistence that lets a cache miss cross a process boundary
//! ([#231](https://github.com/MBehtemam/Montagent/issues/231), ADR-0069).
//!
//! Every test here is one of the ticket's acceptance criteria. The seams are three:
//!
//! - **[`Session`]**, with a sidecar path the test owns. Two sessions over one sidecar
//!   file is a process boundary as far as the cache is concerned, and it is deterministic
//!   because the `ffprobe` behind it is recorded.
//! - **The sidecar file itself**, read back as JSON — what it holds, and what it must
//!   never hold.
//! - **The CLI binary**, in `crates/montagent/tests/adapters.rs`, which is the only seam
//!   that proves the criterion as it is written: *two CLI runs*.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use montagent_core::media::Source;
use montagent_core::media::probe::{Execution, Runner};
use montagent_core::media::session::{MissKind, Session};
use montagent_core::media::sidecar::Sidecar;
use montagent_core::media::tools::Tools;
use montagent_core::verbs::probe as verb;

// ---------------------------------------------------------------------------
// A recorded `ffprobe`, and a scratch sidecar.
// ---------------------------------------------------------------------------

struct Recorded {
    replies: Mutex<Vec<Execution>>,
    calls: Mutex<usize>,
}

struct Shared(Arc<Recorded>);

impl Runner for Shared {
    fn run(&self, _program: &Path, _args: &[String]) -> std::io::Result<Execution> {
        *self.0.calls.lock().unwrap() += 1;
        let mut replies = self.0.replies.lock().unwrap();
        Ok(if replies.len() > 1 {
            replies.remove(0)
        } else {
            replies.first().cloned().expect("a scripted reply")
        })
    }
}

const AUDIO: &str = r#"{"streams":[{"codec_type":"audio","sample_rate":"24000","channels":1,"duration":"1.776000"}],"format":{"duration":"1.776000"}}"#;

fn ok(stdout: &str) -> Execution {
    Execution {
        success: true,
        code: Some(0),
        stdout: stdout.to_string(),
        stderr: String::new(),
    }
}

/// A session over a recorded `ffprobe` and the given sidecar file.
fn session(sidecar: &Path, replies: Vec<Execution>) -> (Arc<Recorded>, Session) {
    let recorder = Arc::new(Recorded {
        replies: Mutex::new(replies),
        calls: Mutex::new(0),
    });
    let tools = Tools {
        ffmpeg: PathBuf::from("/nowhere/ffmpeg"),
        ffprobe: PathBuf::from("/nowhere/ffprobe"),
    };
    let session = Session::with_sidecar(
        tools,
        Box::new(Shared(Arc::clone(&recorder))),
        sidecar.to_path_buf(),
    );
    (recorder, session)
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("montagent-sidecar-tests/{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

/// Rewrite a file with different bytes and a later mtime — the two halves of ADR-0006's
/// key that Montagent observes for itself.
fn grow(path: &Path, bytes: &[u8]) {
    std::thread::sleep(std::time::Duration::from_millis(10));
    std::fs::write(path, bytes).expect("grow the source");
}

// ---------------------------------------------------------------------------
// The criterion: a source that grew between two runs.
// ---------------------------------------------------------------------------

#[test]
fn a_source_that_grew_between_two_runs_is_announced_as_changed() {
    let dir = scratch("grew-between-runs");
    let sidecar = dir.join("probe-cache.json");
    let media = dir.join("05-cobweb.mp3");
    std::fs::write(&media, b"one").unwrap();
    let source = Source::Local(media.clone());

    {
        let (_recorder, mut first) = session(&sidecar, vec![ok(AUDIO)]);
        first.begin_run();
        first.probe(&source).unwrap();
        assert_eq!(first.misses()[0].kind, MissKind::First);
    }

    grow(&media, b"one and a half");

    let (_recorder, mut second) = session(&sidecar, vec![ok(AUDIO)]);
    second.begin_run();
    second.probe(&source).unwrap();

    let miss = second.misses().last().expect("the growth is a miss");
    match miss.kind {
        MissKind::Changed {
            previous_size,
            size,
            previous_mtime_ns,
            mtime_ns,
        } => {
            assert_eq!(previous_size, 3);
            assert_eq!(size, 14);
            assert!(
                previous_mtime_ns.is_some() && mtime_ns > previous_mtime_ns,
                "the previous mtime crossed the boundary too: {previous_mtime_ns:?} → {mtime_ns:?}"
            );
        }
        ref other => panic!("a source that grew must announce itself, got {other:?}"),
    }
}

#[test]
fn an_unchanged_source_costs_nothing_across_the_boundary_and_reports_no_miss() {
    let dir = scratch("unchanged-across-runs");
    let sidecar = dir.join("probe-cache.json");
    let media = dir.join("05-cobweb.mp3");
    std::fs::write(&media, b"one").unwrap();
    let source = Source::Local(media);

    let first_calls = {
        let (recorder, mut first) = session(&sidecar, vec![ok(AUDIO)]);
        first.begin_run();
        let probed = first.probe(&source).unwrap();
        // Dropped here, which is what writes the sidecar.
        (*recorder.calls.lock().unwrap(), probed)
    };

    let (recorder, mut second) = session(&sidecar, vec![ok(AUDIO)]);
    second.begin_run();
    let again = second.probe(&source).unwrap();

    assert_eq!(first_calls.0, 1);
    assert_eq!(
        *recorder.calls.lock().unwrap(),
        0,
        "a file that has not changed has not changed: no second `ffprobe`"
    );
    assert_eq!(again, first_calls.1, "and the facts came back identical");
    assert!(
        second.misses().is_empty(),
        "a hit is not a miss, however it was warmed: {:?}",
        second.misses()
    );
}

// ---------------------------------------------------------------------------
// What the file holds, and what it must never hold.
// ---------------------------------------------------------------------------

const VIDEO: &str = r#"{
    "streams": [{"codec_type":"video","width":1920,"height":1080,"pix_fmt":"yuv420p","r_frame_rate":"25/1","avg_frame_rate":"25/1","duration":"12.000000","sample_aspect_ratio":"40:33"}],
    "format": {"duration": "12.041000"}
}"#;

fn written(sidecar: &Path) -> serde_json::Value {
    let text = std::fs::read_to_string(sidecar).expect("the sidecar was written");
    serde_json::from_str(&text).expect("the sidecar is JSON a person can read")
}

#[test]
fn the_sidecar_carries_the_resolved_dimensions_and_the_par_that_produced_them() {
    // ADR-0023 parked exactly this: whether `par` and the resolved rotation-applied
    // dimensions belong in the sidecar "at zero extra I/O". ADR-0069 answers yes — they are
    // already computed by the probe that fills the entry, so the alternative is to throw
    // them away and re-derive them on the next run.
    let dir = scratch("dimensions-round-trip");
    let sidecar = dir.join("probe-cache.json");
    let media = dir.join("take3.mov");
    std::fs::write(&media, b"bytes").unwrap();
    let source = Source::Local(media.clone());

    let probed = {
        let (_recorder, mut first) = session(&sidecar, vec![ok(VIDEO)]);
        first.begin_run();
        first.probe(&source).unwrap()
    };

    let document = written(&sidecar);
    let canonical = media.canonicalize().unwrap().display().to_string();
    let entry = &document["entries"][&canonical];
    assert_eq!(
        entry["probe"]["dimensions"]["par"],
        serde_json::json!({"num": 40, "den": 33})
    );
    assert_eq!(entry["probe"]["dimensions"]["width"], 2327);

    // And a fresh session answers from it without probing — the facts, not a re-derivation.
    let (recorder, mut second) = session(&sidecar, vec![ok(VIDEO)]);
    second.begin_run();
    assert_eq!(second.probe(&source).unwrap(), probed);
    assert_eq!(*recorder.calls.lock().unwrap(), 0);
}

#[test]
fn nothing_about_a_remote_source_is_ever_persisted() {
    // ADR-0056's no-persistent-cache decision is untouched by ADR-0069, and the two halves
    // stay separate: the sidecar is the local cache's, and the remote cache never reaches
    // it. This test is the pin that keeps them apart.
    let dir = scratch("remote-stays-out");
    let sidecar = dir.join("probe-cache.json");
    let url = "https://cdn.example/take3.mov";

    {
        let (_recorder, mut first) = session(&sidecar, vec![ok(VIDEO)]);
        first.begin_run();
        first.probe(&Source::Remote(url.to_string())).unwrap();
    }

    let document = written(&sidecar);
    assert_eq!(
        document["entries"],
        serde_json::json!({}),
        "a remote probe left an entry behind"
    );
    assert!(
        !std::fs::read_to_string(&sidecar)
            .unwrap()
            .contains("cdn.example"),
        "the URL itself must not appear anywhere in the file"
    );

    // And a second session pays the round trip again, exactly as ADR-0056 requires.
    let (recorder, mut second) = session(&sidecar, vec![ok(VIDEO)]);
    second.begin_run();
    second.probe(&Source::Remote(url.to_string())).unwrap();
    assert_eq!(*recorder.calls.lock().unwrap(), 1);
    assert_eq!(second.misses()[0].kind, MissKind::Remote);
}

/// Probe one source through the `probe` verb, so there is a whole report to look at and
/// not just a session — which is where "never a finding" can actually be asserted.
fn report_for(sidecar: &Path, media: &Path) -> montagent_core::report::Report {
    let (_recorder, mut session) = session(sidecar, vec![ok(AUDIO)]);
    session.begin_run();
    let answer = verb::probe_with(
        &mut session,
        Path::new("/"),
        &[media.to_str().unwrap().to_string()],
    )
    .expect("the recorded ffprobe works");
    answer.report().clone()
}

#[test]
fn a_sidecar_that_cannot_be_read_is_a_cache_miss_and_never_a_finding_about_the_project() {
    // A cache directory is not the project. `validate` and `probe` answer questions about
    // the document and the media; a cache that could not be read is neither, so it must
    // reach the report as a miss and in no other way.
    let dir = scratch("unreadable");
    let media = dir.join("05-cobweb.mp3");
    std::fs::write(&media, b"one").unwrap();

    let mut cases: Vec<(&str, PathBuf)> = Vec::new();

    let corrupt = dir.join("corrupt.json");
    std::fs::write(&corrupt, b"{\"version\": 1, \"entries\": [ truncated").unwrap();
    cases.push(("corrupt", corrupt));

    let future = dir.join("future.json");
    std::fs::write(&future, br#"{"version": 4000, "entries": {}}"#).unwrap();
    cases.push(("a version this binary does not know", future));

    cases.push(("absent", dir.join("never-written.json")));

    // Present, and the process may not read it. Unix-only: there is no portable way to
    // make a file unreadable to its own owner.
    #[cfg(unix)]
    {
        let forbidden = dir.join("forbidden.json");
        std::fs::write(&forbidden, br#"{"version": 2, "entries": {}}"#).unwrap();
        let mut mode = std::fs::metadata(&forbidden).unwrap().permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut mode, 0o000);
        std::fs::set_permissions(&forbidden, mode).unwrap();
        cases.push(("unreadable", forbidden));
    }

    for (what, sidecar) in cases {
        let report = report_for(&sidecar, &media);
        assert!(
            report.findings.is_empty(),
            "a {what} sidecar produced a finding about the project: {:?}",
            report.findings
        );
        assert_eq!(
            report.misses.len(),
            1,
            "a {what} sidecar is a cache that has not seen this file"
        );
        assert_eq!(report.misses[0].kind, MissKind::First, "{what}");
        assert_eq!(
            report.media.len(),
            1,
            "and the probe still answered: {what}"
        );
    }
}

#[test]
fn a_sidecar_that_cannot_be_written_is_silence_rather_than_a_failure() {
    let dir = scratch("unwritable");
    // A directory where the file should be: every write to it fails, for the whole life of
    // the session, and none of it is the caller's problem.
    let sidecar = dir.join("probe-cache.json");
    std::fs::create_dir(&sidecar).unwrap();
    let media = dir.join("05-cobweb.mp3");
    std::fs::write(&media, b"one").unwrap();

    let report = report_for(&sidecar, &media);
    assert!(
        report.findings.is_empty(),
        "an unwritable cache is not a defect in the project: {:?}",
        report.findings
    );
    assert_eq!(report.media.len(), 1, "and the probe answered anyway");

    // The failed write left nothing behind — no temporary beside the thing it could not
    // replace. A run killed mid-write is the case the ignore rule covers; a run that
    // simply failed should not need it.
    let debris: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|name| name.ends_with(".tmp"))
        .collect();
    assert!(debris.is_empty(), "left behind: {debris:?}");
}

#[test]
fn a_source_whose_filesystem_will_not_state_an_mtime_is_never_persisted() {
    // `LocalKey`'s `mtime_ns` is an `Option`, and `None == None` — so such a key matches on
    // `(path, size)` alone and cannot notice a file rewritten to the same length. Within
    // one process that is #190's accepted blind spot. Persisted, it would be permanent, and
    // it is exactly `MissKind::Changed` — ADR-0011's sole mechanism — that would go silent.
    // The sidecar therefore refuses a key Montagent only partly observed.
    let dir = scratch("no-mtime");
    let sidecar = dir.join("probe-cache.json");
    let media = dir.join("05-cobweb.mp3");
    std::fs::write(&media, b"one").unwrap();

    // Written by hand, because no filesystem to hand withholds an mtime: this is the entry
    // such a filesystem would produce, and the question is whether a later run trusts it.
    let entry = serde_json::json!({
        "version": 2,
        "entries": {
            media.canonicalize().unwrap().display().to_string(): {
                "size": 3,
                "mtime_ns": null,
                "last_used_ns": 0,
                "probe": {
                    "source": media.display().to_string(),
                    "quad": {"video_stream_ms": null, "container_ms": 9999,
                             "start_time_ms": null, "r_frame_rate": null, "avg_frame_rate": null},
                    "dimensions": null, "alpha": null, "codec_name": null, "audio": null
                }
            }
        }
    });
    std::fs::write(&sidecar, serde_json::to_string(&entry).unwrap()).unwrap();

    let (recorder, mut session) = session(&sidecar, vec![ok(AUDIO)]);
    session.begin_run();
    // The file has not changed since that entry was written — same path, same size. A
    // seeded entry would answer from cache and the run would do no work.
    let outcome = session.probe(&Source::Local(media)).unwrap();
    assert_eq!(
        *recorder.calls.lock().unwrap(),
        1,
        "the half-observed entry must not have answered for this file"
    );
    assert_eq!(
        outcome.probe().unwrap().quad.container_ms,
        Some(1776),
        "and what came back is what `ffprobe` said, not the 9999 the entry claimed"
    );
    assert_eq!(session.misses()[0].kind, MissKind::First);
    drop(session);

    // What it wrote back is fully observed: the entry it refused is gone, replaced by one
    // keyed on both halves.
    let entries = written(&sidecar);
    let entries = entries["entries"].as_object().unwrap();
    assert_eq!(entries.len(), 1);
    assert!(
        entries.values().all(|e| !e["mtime_ns"].is_null()),
        "a key Montagent only partly observed must not reach the file: {entries:?}"
    );
}

#[test]
fn an_entry_whose_source_no_longer_exists_is_dropped() {
    let dir = scratch("vanished");
    let sidecar = dir.join("probe-cache.json");
    let media = dir.join("05-cobweb.mp3");
    std::fs::write(&media, b"one").unwrap();

    {
        let (_recorder, mut first) = session(&sidecar, vec![ok(AUDIO)]);
        first.begin_run();
        first.probe(&Source::Local(media.clone())).unwrap();
    }
    assert_eq!(written(&sidecar)["entries"].as_object().unwrap().len(), 1);

    std::fs::remove_file(&media).unwrap();
    {
        let (_recorder, mut second) = session(&sidecar, vec![ok(AUDIO)]);
        second.begin_run();
    }

    assert_eq!(
        written(&sidecar)["entries"],
        serde_json::json!({}),
        "an entry with nothing to stat can never match again"
    );
}

#[test]
fn only_a_source_with_content_facts_is_remembered() {
    // A missing file establishes no content facts, and an absence that outlived the process
    // that saw it would be a claim about the disk made from memory.
    let dir = scratch("no-facts");
    let sidecar = dir.join("probe-cache.json");

    {
        let (_recorder, mut first) = session(&sidecar, vec![ok(AUDIO)]);
        first.begin_run();
        let outcome = first.probe(&Source::Local(dir.join("absent.mp3"))).unwrap();
        assert!(outcome.probe().is_none());
    }

    assert_eq!(written(&sidecar)["entries"], serde_json::json!({}));
}

#[test]
fn the_default_sidecar_is_never_inside_a_project() {
    // ADR-0069's answer to "where it lives": under the per-user cache directory, which is
    // not anybody's repository — so there is no file beside a project to commit by
    // accident, and ADR-0053's "movable unit" gains nothing that travels with it.
    let overridden = std::env::var_os(montagent_core::media::sidecar::CACHE_DIR_VAR);

    let Some(path) = Sidecar::default_path() else {
        // Two legitimate ways to have no default: the off switch is set, or this machine
        // will not say where a cache belongs. Neither is a failure — "no sidecar" is the
        // one place a sidecar certainly cannot be committed from.
        assert_eq!(
            overridden.as_deref(),
            Some(std::ffi::OsStr::new("")),
            "a machine with a cache directory and no override must name a default path"
        );
        return;
    };

    assert!(path.is_absolute(), "{path:?}");
    assert!(
        path.components().any(|c| c.as_os_str() == "montagent") || overridden.is_some(),
        "the default path is namespaced to Montagent: {path:?}"
    );
}

// ---------------------------------------------------------------------------
// ADR-0092: the identity a probe observed, and the content guard.
// ---------------------------------------------------------------------------

/// Restore a file's modification time, so a rewrite is invisible to ADR-0006's key.
///
/// This is what `mv` does for free, and it is the whole reason the renumbering shuffle
/// below cannot be caught by `(path, size, mtime)`.
fn rewrite_preserving_mtime(path: &Path, bytes: &[u8]) {
    let was = std::fs::metadata(path)
        .and_then(|m| m.modified())
        .expect("the filesystem states an mtime");
    std::fs::write(path, bytes).expect("rewrite the source");
    let file = std::fs::OpenOptions::new()
        .write(true)
        .open(path)
        .expect("reopen to set times");
    file.set_modified(was).expect("restore the mtime");
}

#[test]
fn a_renumbering_shuffle_that_preserves_size_and_mtime_is_a_rewritten_miss() {
    // #385's mechanism, reduced to its mechanics: different bytes arrive at a path whose
    // `(size, mtime)` did not move, which is what renaming `line-01.wav`…`line-19.wav` onto
    // each other's names does. Every component of ADR-0006's key still matches.
    let dir = scratch("renumbering-shuffle");
    let sidecar = dir.join("probe-cache.json");
    let media = dir.join("line-07.wav");
    std::fs::write(&media, b"take one").unwrap();
    let source = Source::Local(media.clone());

    {
        let (_recorder, mut first) = session(&sidecar, vec![ok(AUDIO)]);
        first.begin_run();
        first.probe(&source).unwrap();
        assert_eq!(first.misses()[0].kind, MissKind::First);
    }

    rewrite_preserving_mtime(&media, b"take two");

    let (recorder, mut second) = session(&sidecar, vec![ok(AUDIO)]);
    second.begin_run();
    second.probe(&source).unwrap();

    match second.misses().last().map(|miss| &miss.kind) {
        Some(MissKind::Rewritten { size, .. }) => assert_eq!(*size, 8),
        other => panic!(
            "a file rewritten under an unchanged key must announce itself as rewritten, \
             got {other:?}"
        ),
    }
    // And it actually re-probed, rather than merely saying so.
    assert_eq!(*recorder.calls.lock().unwrap(), 1);
}

#[test]
fn an_entry_written_before_the_content_guard_is_still_trusted_on_its_key() {
    // The guard is `serde(default)`, so an entry from an older binary has none. It must read
    // as "unguarded" and keep answering — ADR-0092 bumps no `VERSION` precisely so that no
    // machine throws its probes away, and that is worth nothing if an unguarded entry misses.
    let dir = scratch("unguarded-entry");
    let sidecar = dir.join("probe-cache.json");
    let media = dir.join("vo.wav");
    std::fs::write(&media, b"one").unwrap();
    let source = Source::Local(media.clone());

    {
        let (_recorder, mut first) = session(&sidecar, vec![ok(AUDIO)]);
        first.begin_run();
        first.probe(&source).unwrap();
    }

    // Strip the guard, leaving exactly the file an older binary would have written.
    let text = std::fs::read_to_string(&sidecar).unwrap();
    assert!(
        text.contains("\"content\""),
        "the guard was recorded at all"
    );
    let mut document: serde_json::Value = serde_json::from_str(&text).unwrap();
    for entry in document["entries"].as_object_mut().unwrap().values_mut() {
        entry.as_object_mut().unwrap().remove("content");
    }
    std::fs::write(&sidecar, serde_json::to_string_pretty(&document).unwrap()).unwrap();

    let (recorder, mut second) = session(&sidecar, vec![ok(AUDIO)]);
    second.begin_run();
    second.probe(&source).unwrap();

    assert!(
        second.misses().is_empty(),
        "an unguarded entry still answers: {:?}",
        second.misses()
    );
    assert_eq!(*recorder.calls.lock().unwrap(), 0);
}

#[test]
fn a_probe_read_back_from_the_sidecar_carries_the_identity_the_key_states() {
    // The silent-audio-drop criterion, at the seam that can state it cheaply.
    //
    // `Probe::source` holds whatever spelling first cached the probe. `Mix::of` used to
    // canonicalise *that*, against the calling process's working directory — so a relative
    // spelling cached by one run was unresolvable in a run started elsewhere, the entry was
    // dropped, and `render` emitted a silent video at exit 0. What a consumer matches on now
    // is `Probe::identity`, which comes off the canonical key and so cannot depend on where
    // anybody stood.
    let dir = scratch("identity-off-the-key");
    let sidecar = dir.join("probe-cache.json");
    let media = dir.join("vo.wav");
    std::fs::write(&media, b"one").unwrap();
    let source = Source::Local(media.clone());

    {
        let (_recorder, mut first) = session(&sidecar, vec![ok(AUDIO)]);
        first.begin_run();
        first.probe(&source).unwrap();
    }

    // Rewrite the stored spelling to a relative one that resolves nowhere from here, which
    // is the state a run started in the project directory leaves behind.
    let text = std::fs::read_to_string(&sidecar).unwrap();
    let mut document: serde_json::Value = serde_json::from_str(&text).unwrap();
    for entry in document["entries"].as_object_mut().unwrap().values_mut() {
        entry["probe"]["source"] = serde_json::json!("assets/vo.wav");
    }
    std::fs::write(&sidecar, serde_json::to_string_pretty(&document).unwrap()).unwrap();
    assert!(
        std::fs::canonicalize("assets/vo.wav").is_err(),
        "the spelling really is unresolvable from this working directory"
    );

    let (_recorder, mut second) = session(&sidecar, vec![ok(AUDIO)]);
    second.begin_run();
    let probe = second
        .probe(&source)
        .unwrap()
        .probe()
        .cloned()
        .expect("the cache answered");

    assert_eq!(
        probe.source, "assets/vo.wav",
        "the spelling is untouched — it is what the report names"
    );
    assert_eq!(
        probe.identity,
        Some(std::fs::canonicalize(&media).unwrap()),
        "and the identity is the canonical file, off the key rather than off the spelling"
    );
}
