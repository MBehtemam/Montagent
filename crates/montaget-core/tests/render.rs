//! `render` — files in, video out, and the enforcement point for every check (#215).
//!
//! Asserted at spec #168's **seam 1**: a real project file goes to the core verb and the
//! answer comes back as values. Where a claim is about the *file* — its frame count, its
//! duration, whether it carries an audio stream, how loud that stream is — the test reads
//! the file back through `ffprobe`/`ffmpeg`, which is a decoder that is not the encoder
//! that wrote it. Every test that needs one skips, saying so, on a machine without them:
//! ADR-0009 ships Montaget as "a binary, plus an `ffmpeg` the user supplies".
//!
//! **Small projects on purpose.** A 200×200 frame for one second is 25 frames and renders
//! in a debug build in well under a second; the committed fixture is 1631 frames of
//! 1080×1920 and is rendered in full only by the release-gated test at the bottom, which is
//! also where the budget lives.

use std::path::{Path, PathBuf};

use montaget_core::report::ExitCode;
use montaget_core::verbs::render::{Answer, Ask, Progress, render};
use serde_json::Value;

mod common;
use common::media::{audio_stream, peak_db, video_stream};
use common::{
    canonical, document, elements, fixture_dir, fixture_project, has_ffprobe, tempdir,
    write_project,
};

// ---------------------------------------------------------------------------
// Harness
// ---------------------------------------------------------------------------

/// A project on a 200×200 frame at 25 fps, with the header fields given and one track
/// holding the elements.
fn project(header: &str, elements: &str) -> String {
    canonical(&format!(
        r##"{{"frame":{{"width":200,"height":200}},"fps":25,"background":"#000000",{header}
            "tracks":[{{"name":"only","layer":0,"elements":[{elements}]}}]}}"##
    ))
}

/// One rect for the whole of `[start, end)`.
fn rect(id: &str, start: i64, end: i64) -> String {
    format!(
        r##"{{"id":"{id}","type":"rect","start":{start},"end":{end},"x":100,"y":100,
            "width":100,"height":100,"fill":"#FF0000"}}"##
    )
}

/// The fixture's shortest narration, as an absolute path a scratch project can name
/// (ADR-0053: an absolute path is permitted and resolved as-is).
fn narration() -> String {
    fixture_dir()
        .join("audio/05-cobweb.mp3")
        .display()
        .to_string()
        .replace('\\', "/")
}

fn full() -> Ask {
    Ask::default()
}

fn range(from: i64, to: i64) -> Ask {
    Ask {
        from: Some(from),
        to: Some(to),
        output: None,
    }
}

/// Run the verb with progress collected rather than printed.
fn run(path: &Path, ask: &Ask) -> (Answer, Vec<Progress>) {
    let mut steps = Vec::new();
    let answer = render(path, ask, &mut |p| steps.push(p));
    (answer, steps)
}

/// One render that must succeed, refusing to continue if it did not.
#[track_caller]
fn rendered(path: &Path, ask: &Ask) -> Value {
    let (answer, _) = run(path, ask);
    let json = answer.to_json();
    assert_eq!(
        answer.report().exit_code(),
        ExitCode::Ok,
        "render did not answer: {}",
        serde_json::to_string_pretty(&json).unwrap_or_default()
    );
    assert!(
        !json["render"].is_null(),
        "an exit-0 render carries a video"
    );
    json
}

/// The finding codes a report carries, sorted, so two reports can be compared as sets.
fn codes(report: &Value) -> Vec<String> {
    let mut codes: Vec<String> = report["findings"]
        .as_array()
        .expect("findings")
        .iter()
        .map(|f| f["code"].as_str().unwrap_or("?").to_string())
        .collect();
    codes.sort();
    codes
}

fn entries(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .map(|e| e.expect("entry").file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    names
}

// ---------------------------------------------------------------------------
// The enforcement: the identical engine, and the refusal
// ---------------------------------------------------------------------------

#[test]
fn render_refuses_on_any_error_and_its_findings_are_validates() {
    // A key the format does not publish is ADR-0017's closed-schema error — the one
    // mechanism ADR-0016 rests on — and it must stop a video from ever being written.
    let dir = tempdir(line!());
    let body = project(
        r##""duration":1000,"output":"out/refused.mp4","invented":true,"##,
        &rect("card", 0, 1000),
    );
    let path = write_project(&dir, "p.montaget.json", &body);

    let (answer, steps) = run(&path, &full());
    let json = answer.to_json();

    assert_eq!(answer.report().exit_code(), ExitCode::Errors, "{json}");
    assert!(
        json["render"].is_null(),
        "a refused render carries no video: {json}"
    );
    assert!(
        !dir.join("out").exists(),
        "nothing was written, not even the directory"
    );
    assert!(steps.is_empty(), "no frame was started");

    // The identical check engine: the same codes, the same count, on the same file.
    let validated = montaget_core::validate(&path).to_json();
    assert_eq!(codes(&json), codes(&validated));
    assert!(
        codes(&json).contains(&"E-SCHEMA".to_string())
            || json["summary"]["error"].as_u64() > Some(0),
        "{json}"
    );
    assert_eq!(json["tool"], "render");
}

#[test]
fn the_engine_is_one_function_and_render_calls_no_check_of_its_own() {
    // The structural form of "identical": `render` reaches the checks through
    // `validate::checked` and nowhere else. A `crate::checks::<name>::check(` call in
    // `render.rs` would be a second check list, however faithfully copied.
    let source =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/verbs/render.rs"))
            .expect("render.rs");
    assert!(
        source.contains("crate::verbs::validate::checked("),
        "render must run the checks through validate's own entry point"
    );
    assert!(
        !source.contains("::check(document"),
        "render must not invoke any individual check itself"
    );
    let validate = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/verbs/validate.rs"),
    )
    .expect("validate.rs");
    assert!(
        validate.contains("match checked(TOOL, path, session, cache)"),
        "and validate must be the same function with the document thrown away"
    );
}

#[test]
fn a_layout_finding_never_gates_a_render_and_the_review_findings_print_after_it() {
    if !has_ffprobe() {
        return;
    }
    // Pretty-printed on purpose: `L-LAYOUT` fires (ADR-0041) and must not block. The gap
    // between the two rects is `N-TRACK-GAP`, a note that must print beneath the block.
    let dir = tempdir(line!());
    let body = format!(
        "{{\n  \"frame\": {{\"width\": 200, \"height\": 200}},\n  \"fps\": 25,\n  \
         \"duration\": 1000,\n  \"output\": \"out/layout.mp4\",\n  \"tracks\": [{{\"name\": \
         \"only\", \"layer\": 0, \"elements\": [{}, {}]}}]\n}}\n",
        rect("a", 0, 400),
        rect("b", 600, 1000)
    );
    let path = write_project(&dir, "p.montaget.json", &body);

    let json = rendered(&path, &full());
    assert_eq!(json["summary"]["layout"], 1, "{json}");
    assert_eq!(json["summary"]["error"], 0);
    assert!(dir.join("out/layout.mp4").is_file());

    // The prose: the block first, then the findings it did not refuse on, then the
    // footer — so exit 0 never reads as "the video is right" (ADR-0011).
    let (answer, _) = run(&path, &full());
    let prose =
        montaget_core::wire::render_video(&answer, montaget_core::Wire::Text { verbose: true });
    let block = prose.find("RENDER  ").expect("the block");
    let gap = prose.find("N-TRACK-GAP").expect("the note");
    let layout = prose.find("L-LAYOUT").expect("the layout finding");
    let footer = prose.find("NOT CHECKED").expect("the footer");
    assert!(block < gap && gap < footer, "{prose}");
    assert!(block < layout && layout < footer, "{prose}");
    assert!(
        prose.contains("half-open"),
        "the range is stated half-open: {prose}"
    );

    // And the findings are `validate`'s, exactly.
    assert_eq!(
        codes(&json),
        codes(&montaget_core::validate(&path).to_json())
    );
}

// ---------------------------------------------------------------------------
// The deliverable
// ---------------------------------------------------------------------------

#[test]
fn a_full_render_writes_the_declared_output_via_a_temp_path_and_leaves_nothing_else() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let body = project(
        r##""duration":1000,"output":"out/full.mp4","##,
        &rect("card", 0, 1000),
    );
    let path = write_project(&dir, "p.montaget.json", &body);

    let json = rendered(&path, &full());
    let video = &json["render"];
    assert_eq!(video["partial"], false);
    assert_eq!(video["from"], 0);
    assert_eq!(video["to"], 1000);
    assert_eq!(video["duration_ms"], 1000);
    assert_eq!(
        video["frames"], 25,
        "frames 0..=24 sample inside [0, 1000) at 25 fps"
    );
    assert_eq!(video["fps"], 25);
    assert_eq!(video["width"], 200);
    assert_eq!(video["height"], 200);
    assert!(video["encoded"].is_null(), "an even frame is not padded");
    assert!(video["wall_ms"].as_u64().is_some());
    assert!(video["realtime"].as_f64().unwrap() > 0.0);
    assert_eq!(video["painted"], serde_json::json!(["card"]));
    assert_eq!(video["mixed"], serde_json::json!([]));

    let written = dir.join("out/full.mp4");
    assert_eq!(
        video["path"].as_str().map(PathBuf::from),
        Some(written.clone())
    );
    assert_eq!(
        entries(&dir.join("out")),
        vec!["full.mp4".to_string()],
        "the temp sibling is gone and only the deliverable remains"
    );
    let stream = video_stream(&written);
    assert_eq!((stream.width, stream.height), (Some(200), Some(200)));
    assert_eq!(stream.frames, Some(25));
    assert_eq!(stream.duration_ms, Some(1000));
    assert!(
        audio_stream(&written).is_none(),
        "no audible element, no audio stream"
    );
}

#[test]
fn an_interrupted_render_leaves_no_partial_file_at_the_declared_path() {
    if !has_ffprobe() {
        return;
    }
    // The encoder cannot even start: `out` is a *file*, so no temp sibling can be made
    // under it. Exit 70 — Montaget could not run, and nothing about the project is wrong.
    let dir = tempdir(line!());
    std::fs::write(dir.join("out"), b"not a directory").unwrap();
    let body = project(
        r##""duration":1000,"output":"out/blocked.mp4","##,
        &rect("card", 0, 1000),
    );
    let path = write_project(&dir, "p.montaget.json", &body);

    let (answer, _) = run(&path, &full());
    assert_eq!(answer.report().exit_code(), ExitCode::Internal);
    assert!(answer.video().is_none());
    assert_eq!(std::fs::read(dir.join("out")).unwrap(), b"not a directory");
    assert_eq!(
        entries(&dir),
        vec!["out".to_string(), "p.montaget.json".to_string()],
        "nothing else was left behind"
    );
}

#[test]
fn a_partial_render_derives_its_name_and_can_never_land_on_the_deliverable() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let body = project(
        r##""duration":2000,"output":"out/clip.mp4","##,
        &rect("card", 0, 2000),
    );
    let path = write_project(&dir, "p.montaget.json", &body);
    // A deliverable already there, which no partial render may touch.
    std::fs::create_dir_all(dir.join("out")).unwrap();
    std::fs::write(dir.join("out/clip.mp4"), b"the real deliverable").unwrap();

    // Derived: `out/<name>.<from>-<to>.mp4` (ADR-0011).
    let json = rendered(&path, &range(500, 1000));
    let video = &json["render"];
    assert_eq!(video["partial"], true);
    assert_eq!(video["from"], 500);
    assert_eq!(video["to"], 1000);
    assert_eq!(
        video["frames"], 12,
        "frames 13..=24 at 25 fps: 520 ms up to 960 ms; frame 25 is 1000 ms, excluded"
    );
    let derived = dir.join("out/clip.500-1000.mp4");
    assert_eq!(
        video["path"].as_str().map(PathBuf::from),
        Some(derived.clone())
    );
    assert_eq!(video_stream(&derived).frames, Some(12));
    assert_eq!(video_stream(&derived).duration_ms, Some(480));

    // Refused: an explicit `--output` naming the project's own `output` while a range is
    // set, in either spelling.
    for explicit in [dir.join("out/clip.mp4"), dir.join("out/../out/./clip.mp4")] {
        let (answer, steps) = run(
            &path,
            &Ask {
                from: Some(0),
                to: Some(500),
                output: Some(explicit.clone()),
            },
        );
        assert_eq!(
            answer.report().exit_code(),
            ExitCode::BadInvocation,
            "{}",
            explicit.display()
        );
        assert_eq!(answer.report().findings[0].code, "E-INVOCATION");
        assert!(steps.is_empty());
    }
    assert_eq!(
        std::fs::read(dir.join("out/clip.mp4")).unwrap(),
        b"the real deliverable"
    );

    // An explicit `--output` elsewhere is honoured.
    let elsewhere = dir.join("elsewhere/part.mp4");
    let json = rendered(
        &path,
        &Ask {
            from: Some(0),
            to: Some(200),
            output: Some(elsewhere.clone()),
        },
    );
    assert_eq!(
        json["render"]["path"].as_str().map(PathBuf::from),
        Some(elsewhere.clone())
    );
    assert_eq!(json["render"]["frames"], 5);
    assert!(elsewhere.is_file());
}

#[test]
fn a_range_is_both_flags_or_neither_and_is_half_open() {
    let dir = tempdir(line!());
    let body = project(
        r##""duration":1000,"output":"out/x.mp4","##,
        &rect("card", 0, 1000),
    );
    let path = write_project(&dir, "p.montaget.json", &body);

    for ask in [
        Ask {
            from: Some(0),
            to: None,
            output: None,
        },
        Ask {
            from: None,
            to: Some(500),
            output: None,
        },
        range(500, 500),
        range(600, 500),
        range(-1, 500),
    ] {
        let (answer, _) = run(&path, &ask);
        assert_eq!(
            answer.report().exit_code(),
            ExitCode::BadInvocation,
            "{ask:?}"
        );
        assert_eq!(answer.report().findings[0].code, "E-INVOCATION");
    }
    assert!(!dir.join("out").exists());
}

#[test]
fn a_project_with_no_output_and_no_flag_is_exit_3_naming_the_field() {
    let dir = tempdir(line!());
    let body = project(r##""duration":1000,"##, &rect("card", 0, 1000));
    let path = write_project(&dir, "p.montaget.json", &body);

    let (answer, _) = run(&path, &full());
    assert_eq!(answer.report().exit_code(), ExitCode::BadInvocation);
    // By code and by the finding's own field — never by the rendered prose (spec #168).
    let finding = &answer.to_json()["findings"][0];
    assert_eq!(finding["code"], "E-INVOCATION");
    let reason = finding["fields"]["reason"]
        .as_str()
        .expect("the reason field");
    assert!(
        reason.contains("`output`") && reason.contains("--output"),
        "{reason}"
    );
}

#[test]
fn a_project_with_no_duration_renders_to_its_last_boundary() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let body = project(
        r##""output":"out/derived.mp4","##,
        &format!("{},{}", rect("a", 0, 400), rect("b", 400, 800)),
    );
    let path = write_project(&dir, "p.montaget.json", &body);

    let json = rendered(&path, &full());
    assert_eq!(json["render"]["to"], 800);
    assert_eq!(json["render"]["frames"], 20);
    assert_eq!(
        video_stream(&dir.join("out/derived.mp4")).duration_ms,
        Some(800)
    );
}

#[test]
fn the_frame_count_follows_the_sampling_grid() {
    if !has_ffprobe() {
        return;
    }
    // 30 fps: frame n samples at n × 100/3 ms. [500, 1000) holds frames 15..=29.
    let dir = tempdir(line!());
    let body = canonical(&format!(
        r##"{{"frame":{{"width":200,"height":200}},"fps":30,"duration":1000,"output":"out/grid.mp4",
            "tracks":[{{"name":"only","layer":0,"elements":[{}]}}]}}"##,
        rect("card", 0, 1000)
    ));
    let path = write_project(&dir, "p.montaget.json", &body);

    let json = rendered(&path, &range(500, 1000));
    assert_eq!(json["render"]["frames"], 15);
    assert_eq!(
        video_stream(&dir.join("out/grid.500-1000.mp4")).frames,
        Some(15)
    );

    let json = rendered(&path, &full());
    assert_eq!(json["render"]["frames"], 30);
}

// ---------------------------------------------------------------------------
// Never proxy-scaled
// ---------------------------------------------------------------------------

#[test]
fn the_output_is_the_declared_frame_never_the_proxy_target() {
    if !has_ffprobe() {
        return;
    }
    // Long edge past ADR-0046's 1280 px cap, which `preview` would scale to and `render`
    // must not (ADR-0021). Odd on both axes, so the even-padding disclosure is exercised
    // in the same file.
    let dir = tempdir(line!());
    let body = canonical(
        r##"{"frame":{"width":1441,"height":2561},"fps":25,"duration":80,"output":"out/big.mp4",
            "tracks":[{"name":"only","layer":0,"elements":[
              {"id":"card","type":"rect","start":0,"end":80,"x":100,"y":100,"width":100,"height":100,"fill":"#FF0000"}]}]}"##,
    );
    let path = write_project(&dir, "p.montaget.json", &body);

    let json = rendered(&path, &full());
    let video = &json["render"];
    assert_eq!(
        (video["width"].as_i64(), video["height"].as_i64()),
        (Some(1441), Some(2561))
    );
    assert_eq!(
        video["encoded"],
        serde_json::json!({"width": 1442, "height": 2562})
    );
    let stream = video_stream(&dir.join("out/big.mp4"));
    assert_eq!((stream.width, stream.height), (Some(1442), Some(2562)));
    assert_eq!(video["frames"], 2);
}

// ---------------------------------------------------------------------------
// Audio
// ---------------------------------------------------------------------------

#[test]
fn audio_elements_are_mixed_into_the_output_at_their_place_on_the_clock() {
    if !has_ffprobe() {
        return;
    }
    // One narration from 1000 to 2500 ms, in a 3 s project. The first second is digital
    // silence; the narration's own peak is well above it.
    let dir = tempdir(line!());
    let body = project(
        r##""duration":3000,"output":"out/mixed.mp4","##,
        &format!(
            r##"{{"id":"vo","type":"audio","start":1000,"end":2500,"source":"{}","source_start":0,"source_end":1500}}"##,
            narration()
        ),
    );
    let path = write_project(&dir, "p.montaget.json", &body);

    let json = rendered(&path, &full());
    assert_eq!(json["render"]["mixed"], serde_json::json!(["vo"]));
    assert_eq!(json["render"]["not_mixed"], serde_json::json!([]));
    let written = dir.join("out/mixed.mp4");
    let audio = audio_stream(&written).expect("an audio stream");
    assert_eq!(audio.sample_rate, Some(48000));
    assert!((audio.duration_ms.unwrap() - 3000).abs() <= 50, "{audio:?}");

    let before = peak_db(&written, 0.0, 0.9);
    let during = peak_db(&written, 1.2, 2.4);
    assert!(
        before < -60.0,
        "silence before the element starts: {before} dB"
    );
    assert!(
        during > -30.0,
        "the narration plays where it is placed: {during} dB"
    );
}

#[test]
fn the_projects_loop_flag_changes_nothing_about_the_encode() {
    if !has_ffprobe() {
        return;
    }
    // ADR-0062 is explicit that `loop` is *"purely `validate`-facing … `render` and every
    // other tool are unaffected"*: it is the author's assertion that `duration` connects
    // back to `0`, which one check reads and nothing else does. Montaget writes no
    // container-level loop metadata, does not repeat the timeline, and does not wrap the
    // mix — so the two files must be identical **byte for byte**, not merely similar. A
    // weaker assertion would pass a render that quietly wrapped the last 40 ms of audio.
    let dir = tempdir(line!());
    let elements = format!(
        r##"{{"id":"vo","type":"audio","start":0,"end":1500,"source":"{}","source_start":0,"source_end":1500}}"##,
        narration()
    );
    let plain = write_project(
        &dir,
        "plain.montaget.json",
        &project(r##""duration":2000,"output":"out/plain.mp4","##, &elements),
    );
    let looping = write_project(
        &dir,
        "looping.montaget.json",
        &project(
            r##""duration":2000,"loop":true,"output":"out/looping.mp4","##,
            &elements,
        ),
    );

    let plain_answer = rendered(&plain, &full());
    let looping_answer = rendered(&looping, &full());
    assert_eq!(
        looping_answer["render"]["frames"], plain_answer["render"]["frames"],
        "the flag is not a duration"
    );
    assert_eq!(
        looping_answer["render"]["mixed"], plain_answer["render"]["mixed"],
        "nor a claim about what is audible"
    );
    assert_eq!(
        std::fs::read(dir.join("out/looping.mp4")).expect("the looping render"),
        std::fs::read(dir.join("out/plain.mp4")).expect("the plain render"),
        "`loop` reached the encode"
    );
}

#[test]
fn volume_speed_and_loop_go_through_the_mix() {
    if !has_ffprobe() {
        return;
    }
    // The word in `05-cobweb.mp3` sounds from 0.187 s to 0.712 s of the file (measured
    // with `silencedetect` at -40 dB); the rest is silence. Every window below is placed
    // against that. `vo-fade` fades 1 → 0 across its own range (ADR-0055 keyframes),
    // `vo-slow` plays at 0.645 (ADR-0020's `atempo`), `vo-loop` loops a 500 ms cut over
    // 2 s, `vo-mute` is `volume: 0`, and `vo-ref` is the word untouched, as the level the
    // fade is measured against. Every one is mixed; the file says so in its levels.
    let dir = tempdir(line!());
    let src = narration();
    let elements = format!(
        r##"{{"id":"vo-fade","type":"audio","start":0,"end":1500,"source":"{src}","source_start":0,"source_end":1500,
             "volume":[{{"t":0,"v":1.0}},{{"t":1500,"v":0.0,"ease":"linear"}}]}},
            {{"id":"vo-slow","type":"audio","start":2000,"end":4326,"source":"{src}","source_start":0,"source_end":1500,"speed":0.645}},
            {{"id":"vo-loop","type":"audio","start":5000,"end":7000,"source":"{src}","source_start":300,"source_end":800,"overrun":"loop"}},
            {{"id":"vo-mute","type":"audio","start":8000,"end":9500,"source":"{src}","source_start":0,"source_end":1500,"volume":0}},
            {{"id":"vo-ref","type":"audio","start":10000,"end":11500,"source":"{src}","source_start":0,"source_end":1500}}"##
    );
    let body = canonical(&format!(
        r##"{{"frame":{{"width":200,"height":200}},"fps":25,"duration":12000,"output":"out/levels.mp4",
            "tracks":[{{"name":"a","layer":0,"elements":[{}]}},{{"name":"b","layer":1,"elements":[{elements}]}}]}}"##,
        rect("card", 0, 12000)
    ));
    let path = write_project(&dir, "p.montaget.json", &body);

    let json = rendered(&path, &full());
    assert_eq!(
        json["render"]["mixed"],
        serde_json::json!(["vo-fade", "vo-slow", "vo-loop", "vo-mute", "vo-ref"]),
        "{}",
        json["render"]
    );
    let written = dir.join("out/levels.mp4");

    // The fade: at 0.2 s the volume is still ~0.87 and at 0.55 s it is ~0.63 — so the
    // word's onset is within 2 dB of the untouched reference and its tail is at least
    // 3 dB below it (0.63 is -4 dB).
    let onset = peak_db(&written, 0.19, 0.30) - peak_db(&written, 10.19, 10.30);
    let tail = peak_db(&written, 0.50, 0.61) - peak_db(&written, 10.50, 10.61);
    assert!(
        onset.abs() < 2.0,
        "the fade starts at 1: {onset:+.1} dB against the reference"
    );
    assert!(
        tail < -3.0,
        "and has come down by its tail: {tail:+.1} dB against the reference"
    );
    // Slowed: at 1x the word would end at 2.712 s; at 0.645x it runs to ~3.1 s.
    assert!(
        peak_db(&written, 2.30, 2.60) > -30.0,
        "the slowed word is audible"
    );
    assert!(
        peak_db(&written, 2.75, 2.95) > -30.0,
        "0.645x: still sounding past where 1x ended"
    );
    assert!(
        peak_db(&written, 2.00, 2.25) < -60.0,
        "and its lead-in silence is stretched too"
    );
    // Looped: the cut sounds 0..0.41 s of each 500 ms repetition, four times, then stops.
    assert!(
        peak_db(&written, 6.55, 6.80) > -30.0,
        "the fourth repetition"
    );
    // -40 dB, the threshold the source's own silences were measured at: the word's
    // decaying tail sits above digital silence for a while after `silencedetect` calls it
    // quiet, and a bar set at -60 would be measuring the mp3 rather than the loop.
    assert!(
        peak_db(&written, 6.90, 6.99) < -40.0,
        "and the gap inside it"
    );
    assert!(
        peak_db(&written, 7.05, 7.90) < -60.0,
        "then nothing past the element's end"
    );
    // Muted: silent for its whole range.
    assert!(peak_db(&written, 8.1, 9.4) < -60.0, "volume 0 is silence");
}

#[test]
fn a_video_elements_embedded_audio_is_the_same_volume_field() {
    if !has_ffprobe() {
        return;
    }
    // ADR-0055 declined `mute` on the ground that there is nothing else to mute: *"a
    // `video` element is one element with intrinsic audio"*, and `volume: 0` already says
    // silent. That is a claim about the encode, not about the schema — asserted here on
    // the committed reference render, one second of it where the narration is sounding
    // (`vo-sentence-05-a` runs 10468..12652 ms), played once at its own level and once at
    // `volume: 0`.
    let source = fixture_dir()
        .join("reference/en-halloween-decorating.mp4")
        .display()
        .to_string()
        .replace('\\', "/");
    let clip = |volume: &str| {
        format!(
            r##"{{"id":"clip","type":"video","start":0,"end":1000,"source":"{source}",
                "source_start":11000,"source_end":12000,"x":0,"y":0,"origin":"top-left",
                "width":200,"height":200,"fit":"literal"{volume}}}"##
        )
    };

    let dir = tempdir(line!());
    let audible = write_project(
        &dir,
        "audible.montaget.json",
        &project(
            r##""duration":1000,"output":"out/audible.mp4","##,
            &clip(""),
        ),
    );
    let silent = write_project(
        &dir,
        "silent.montaget.json",
        &project(
            r##""duration":1000,"output":"out/silent.mp4","##,
            &clip(r##","volume":0"##),
        ),
    );

    for (path, name) in [(&audible, "audible"), (&silent, "silent")] {
        let json = rendered(path, &full());
        assert_eq!(
            json["render"]["mixed"],
            serde_json::json!(["clip"]),
            "the video's own audio is what is mixed ({name})"
        );
    }
    let heard = peak_db(&dir.join("out/audible.mp4"), 0.1, 0.9);
    let muted = peak_db(&dir.join("out/silent.mp4"), 0.1, 0.9);
    assert!(
        heard > -30.0,
        "the clip carries its own narration: {heard} dB"
    );
    assert!(
        muted < -60.0,
        "`volume: 0` silences a video's embedded audio, with no `mute` field to reach for: \
         {muted} dB"
    );
}

#[test]
fn what_render_paints_is_what_frame_paints() {
    if !has_ffprobe() {
        return;
    }
    // One painter (ADR-0021's "the picture frame shows is the picture render produces"):
    // the elements the render reports painting are exactly what `frame` paints at an
    // instant inside the range, and what neither could paint is named by both.
    let dir = tempdir(line!());
    let body = canonical(&format!(
        r##"{{"frame":{{"width":200,"height":200}},"fps":25,"duration":200,"output":"out/same.mp4",
            "tracks":[{{"name":"a","layer":0,"elements":[{}]}},
                      {{"name":"b","layer":1,"elements":[{{"id":"badge","type":"ellipse","start":0,"end":200,"x":30,"y":30,"width":40,"height":40,"fill":"#00FF00"}}]}}]}}"##,
        rect("card", 0, 200)
    ));
    let path = write_project(&dir, "p.montaget.json", &body);

    let json = rendered(&path, &full());
    let picture = montaget_core::verbs::frame::frame(
        &path,
        &montaget_core::verbs::frame::Ask {
            at: Some(100),
            ..Default::default()
        },
    )
    .to_json();
    assert_eq!(json["render"]["painted"], picture["frame"]["painted"]);
    assert_eq!(
        json["render"]["not_painted"],
        picture["frame"]["not_painted"]
    );
}

// ---------------------------------------------------------------------------
// The committed fixture, in full — release builds only
// ---------------------------------------------------------------------------

#[test]
fn the_whole_fixture_renders_to_its_declared_duration_with_every_narration_mixed() {
    if !has_ffprobe() {
        return;
    }
    // 1631 frames of 1080x1920 through a debug build is minutes, and ADR-0021's budget is
    // stated for optimised builds. A debug run renders the first two seconds instead, so
    // the path is still exercised on the real file, and says so.
    let dir = tempdir(line!());
    let output = dir.join("fixture.mp4");
    let optimised = cfg!(not(debug_assertions));
    let ask = if optimised {
        Ask {
            output: Some(output.clone()),
            ..Default::default()
        }
    } else {
        eprintln!("debug build: rendering 0..2000 ms of the fixture rather than all of it");
        Ask {
            from: Some(0),
            to: Some(2000),
            output: Some(output.clone()),
        }
    };

    let (answer, steps) = run(&fixture_project(), &ask);
    let json = answer.to_json();
    assert_eq!(answer.report().exit_code(), ExitCode::Ok, "{json}");
    let video = &json["render"];
    assert_eq!(video["not_painted"], serde_json::json!([]), "{video}");
    assert_eq!(video["not_mixed"], serde_json::json!([]), "{video}");
    assert!(
        steps.len() >= 2,
        "progress at the start and the end at least: {steps:?}"
    );
    assert_eq!(steps.last().map(|p| p.done), steps.last().map(|p| p.of));

    if !optimised {
        assert_eq!(video["frames"], 50);
        return;
    }

    // Spec #168: the whole-video comparison is what falsifies duration, frame count and
    // audio placement — frames cannot.
    assert_eq!(video["duration_ms"], 65216);
    assert_eq!(video["frames"], 1631, "ceil(65216 x 25 / 1000)");
    assert_eq!(
        video["mixed"].as_array().map(Vec::len),
        Some(20),
        "20 audio elements"
    );
    let stream = video_stream(&output);
    assert_eq!(stream.frames, Some(1631));
    assert_eq!((stream.width, stream.height), (Some(1080), Some(1920)));
    let audio = audio_stream(&output).expect("the narration");
    assert!(
        (audio.duration_ms.unwrap() - 65216).abs() <= 100,
        "{audio:?}"
    );
    narration_lands_at_the_instants_the_document_states(&output);
    the_four_stretched_sentences_play_for_as_long_as_their_speed_says(&output);
}

/// #216's first acceptance criterion, read literally: *"the rendered fixture carries its
/// narration at the right instants"*.
///
/// The windows are read off the document rather than written down here — a table of 20
/// hand-copied instants is a second statement of the fixture, and the first one to drift
/// would be this one. Each element's range must carry sound and every gap between two of
/// them must not, which is the pair of claims that pins placement: a mix that summed
/// everything at `0` would pass the first half alone, and one that mixed nothing would
/// pass the second.
fn narration_lands_at_the_instants_the_document_states(output: &Path) {
    let mut windows = audible_windows();
    windows.sort_by_key(|(_, start, _)| *start);

    let mut previous_end: Option<i64> = None;
    for (id, start, end) in &windows {
        // Inset by 50 ms at each edge: `volumedetect` reads whole packets, so a window
        // flush against a boundary would sample the neighbouring silence too.
        assert!(
            peak_db(output, ms(*start) + 0.05, ms(*end) - 0.05) > -30.0,
            "`{id}` is silent over its own {start}..{end} ms"
        );
        if let Some(previous_end) = previous_end
            && *start - previous_end > 200
        {
            assert!(
                peak_db(output, ms(previous_end) + 0.05, ms(*start) - 0.05) < -60.0,
                "the gap {previous_end}..{start} ms before `{id}` carries sound"
            );
        }
        previous_end = Some(*end);
    }
    let tail = previous_end.expect("the fixture has narration");
    assert!(
        peak_db(output, ms(tail) + 0.1, ms(tail) + 1.9) < -60.0,
        "the video runs on past the last narration line, in silence"
    );
}

/// #216: *"`speed: 0.645` reproduces through `atempo` on all four fixture narration
/// elements"* — ADR-0020's convention, and the only place in the committed project where
/// a rate other than 1 is exercised at all.
///
/// Falsified by the sound itself rather than by the graph: at `speed` 1 each of these four
/// sources would run out `source span` after its `start` and the rest of the element would
/// be silence, so the assertion is that there is still speech playing *past* the instant an
/// unstretched source would have ended. The silence in the gap after it — asserted above —
/// is the other side: stretched, but not past its declared `end`.
fn the_four_stretched_sentences_play_for_as_long_as_their_speed_says(output: &Path) {
    let document = document(&fixture_project());
    let stretched: Vec<&Value> = elements(&document)
        .filter(|e| e["speed"].as_f64().is_some_and(|speed| speed != 1.0))
        .collect();
    assert_eq!(stretched.len(), 4, "the fixture's four 0.645 sentences");

    for element in stretched {
        let id = element["id"].as_str().expect("an id");
        let (start, end) = (
            element["start"].as_i64().unwrap(),
            element["end"].as_i64().unwrap(),
        );
        let span =
            element["source_end"].as_i64().unwrap() - element["source_start"].as_i64().unwrap();
        assert_eq!(element["speed"].as_f64(), Some(0.645), "`{id}`");
        // Where the source would have run dry unstretched, and where `atempo` carries it to.
        let unstretched_end = start + span;
        assert!(
            unstretched_end + 400 < end,
            "`{id}` at 1x would end at {unstretched_end}, inside {start}..{end} — the \
             window this test reads is only a window if it is"
        );
        assert!(
            peak_db(output, ms(unstretched_end) + 0.1, ms(unstretched_end) + 0.4) > -30.0,
            "`{id}` is silent at {unstretched_end} ms, where an unstretched source ends — \
             `speed` 0.645 did not reach `atempo`"
        );
    }
}

/// Every `audio` element's `id` and timeline range, read off the committed fixture.
fn audible_windows() -> Vec<(String, i64, i64)> {
    let document = document(&fixture_project());
    elements(&document)
        .filter(|e| e["type"] == "audio")
        .map(|e| {
            (
                e["id"].as_str().expect("an id").to_string(),
                e["start"].as_i64().expect("a start"),
                e["end"].as_i64().expect("an end"),
            )
        })
        .collect()
}

/// Milliseconds as the seconds [`peak_db`] takes.
fn ms(t: i64) -> f64 {
    t as f64 / 1000.0
}
