//! `preview` — the proxy ladder, its two floors, and the disclosure that is never optional
//! (#218).
//!
//! Asserted at spec #168's **seam 1**: a real project file goes to the core verb and the
//! answer comes back as values. Where the claim is about the *file* — what frame size the
//! video actually carries — the test reads it back through `ffprobe`, a decoder that is not
//! the encoder that wrote it, and skips saying so on a machine with neither (ADR-0009).
//!
//! **Small projects on purpose**, as `tests/render.rs` does it: what the ladder decides is
//! arithmetic on the declared frame, and a 2000×1000 project exercises the 1280 px cap as
//! well as an 8K one would while rendering in a debug build in well under a second.
//!
//! **The clock is stated, not raced.** A test that produced a budget miss by finding a
//! machine slow enough would assert the hardware, not the ladder; `Clock::Stated` names the
//! wall clock each rung is judged against, and `Clock::Scrub` — ADR-0021's `<5 s`, the only
//! thing either adapter can ask for — is what every other test here runs under.

use std::path::Path;
use std::process::Command;
use std::time::Duration;

use montagent_core::report::ExitCode;
use montagent_core::verbs::preview::{Answer, Ask, Clock, preview};
use serde_json::Value;

mod common;
use common::{canonical, has_ffprobe, tempdir, write_project};

// ---------------------------------------------------------------------------
// Harness
// ---------------------------------------------------------------------------

/// A project on `width`×`height` at 25 fps, one rect for the whole of its 200 ms — five
/// frames, which is enough to have a ladder walk a real encode.
fn project(width: i64, height: i64) -> String {
    canonical(&format!(
        r##"{{"frame":{{"width":{width},"height":{height}}},"fps":25,"background":"#000000",
            "duration":200,"output":"out/video.mp4",
            "tracks":[{{"name":"only","layer":0,"elements":[
              {{"id":"card","type":"rect","start":0,"end":200,"x":{},"y":{},
                "width":40,"height":40,"fill":"#FF0000"}}]}}]}}"##,
        width / 2,
        height / 2,
    ))
}

fn run(path: &Path, ask: &Ask) -> Answer {
    preview(path, ask, &mut |_| {})
}

/// One preview that must succeed, refusing to continue if it did not.
#[track_caller]
fn previewed(path: &Path, ask: &Ask) -> Value {
    let answer = run(path, ask);
    let json = answer.to_json();
    assert_eq!(
        answer.report().exit_code(),
        ExitCode::Ok,
        "preview did not answer: {}",
        serde_json::to_string_pretty(&json).unwrap_or_default()
    );
    assert!(
        !json["preview"].is_null(),
        "an exit-0 preview carries a video"
    );
    json
}

/// The one sentence a refusal is made of, from the report's findings.
fn refusal(answer: &Answer) -> String {
    let json = answer.to_json();
    json["findings"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|f| f["fields"]["reason"].as_str().map(str::to_string))
        .collect::<Vec<_>>()
        .join(" ")
}

/// The video stream's frame size, as `ffprobe` reports it — the encoder's own claim is not
/// evidence about the file.
fn encoded_frame(path: &Path) -> (i64, i64) {
    let tools = montagent_core::media::tools::resolve().expect("ffprobe");
    let out = Command::new(tools.ffprobe)
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=width,height",
            "-of",
            "json",
        ])
        .arg(path)
        .output()
        .expect("ffprobe runs");
    let value: Value = serde_json::from_slice(&out.stdout).expect("ffprobe json");
    let stream = &value["streams"][0];
    (
        stream["width"].as_i64().expect("width"),
        stream["height"].as_i64().expect("height"),
    )
}

// ---------------------------------------------------------------------------
// The target (story 61, ADR-0046)
// ---------------------------------------------------------------------------

#[test]
fn the_default_target_is_720p_with_the_long_edge_capped_at_1280_and_rounded_to_even() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    // 2001×1000: an odd long edge and an aspect whose scaled short edge does not land on
    // an integer, so both halves of ADR-0046's rule have something to do.
    let path = write_project(&dir, "project.json", &project(2001, 1000));
    let json = previewed(&path, &Ask::default());
    let preview = &json["preview"];

    assert_eq!(preview["width"], 1280);
    assert_eq!(preview["height"], 640);
    assert_eq!(preview["tier"]["name"], "720p");
    assert_eq!(preview["tier"]["long_edge_cap"], 1280);
    assert_eq!(preview["tier"]["proxied"], true);
    assert_eq!(preview["tier"]["declared_width"], 2001);
    assert_eq!(preview["tier"]["declared_height"], 1000);

    // And the file itself carries it — read back by a decoder that is not the encoder.
    let written = Path::new(preview["path"].as_str().expect("a path"));
    assert_eq!(encoded_frame(written), (1280, 640));
    // The derived name, never the project's own `output`.
    assert!(
        written.ends_with("out/video.preview.0-200.mp4"),
        "{}",
        written.display()
    );
    assert!(!dir.join("out/video.mp4").exists(), "the deliverable");
}

#[test]
fn a_project_already_inside_the_cap_is_previewed_at_its_own_pixels() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let path = write_project(&dir, "project.json", &project(200, 200));
    let json = previewed(&path, &Ask::default());
    let preview = &json["preview"];

    assert_eq!(
        (preview["width"].as_i64(), preview["height"].as_i64()),
        (Some(200), Some(200))
    );
    // `native`, not `720p`: ADR-0046 makes the field report the resolution actually
    // rendered at, and naming a tier whose cap never engaged would be the one thing the
    // field exists to prevent. The prose says *which* native it is — inside the cap, not
    // the escape hatch.
    assert_eq!(preview["tier"]["name"], "native");
    assert_eq!(preview["tier"]["proxied"], false);
    assert!(preview["tier"]["long_edge_cap"].is_null());
    assert!(
        preview["tier"]["disclosure"]
            .as_str()
            .unwrap_or_default()
            .contains("true pixels"),
        "{}",
        preview["tier"]["disclosure"]
    );
}

#[test]
fn the_full_resolution_escape_hatch_is_true_pixels_and_carries_no_budget() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let path = write_project(&dir, "project.json", &project(1600, 900));
    let json = previewed(
        &path,
        &Ask {
            full: true,
            // Stated at zero, and it must not matter: the observational arm has no
            // ceiling, so there is nothing for a clock to stop (ADR-0021).
            clock: Clock::Stated(vec![Duration::ZERO]),
            ..Ask::default()
        },
    );
    let preview = &json["preview"];
    assert_eq!(preview["tier"]["name"], "native");
    assert_eq!(preview["width"], 1600);
    assert_eq!(preview["height"], 900);
    assert_eq!(preview["tier"]["proxied"], false);
    assert_eq!(preview["tier"]["degraded"], false);
    assert!(preview["tier"]["budget_ms"].is_null());
    assert!(preview["tier"]["long_edge_cap"].is_null());
}

// ---------------------------------------------------------------------------
// The ladder (story 62, ADR-0065)
// ---------------------------------------------------------------------------

#[test]
fn a_miss_degrades_exactly_once_to_540p_and_the_tier_is_disclosed() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let path = write_project(&dir, "project.json", &project(2000, 1000));
    // No clock at all for the first rung, a generous one for the second: the ladder takes
    // exactly one step down and lands.
    let json = previewed(
        &path,
        &Ask {
            clock: Clock::Stated(vec![Duration::ZERO, Duration::from_secs(600)]),
            ..Ask::default()
        },
    );
    let preview = &json["preview"];

    assert_eq!(preview["tier"]["name"], "540p");
    assert_eq!(preview["tier"]["degraded"], true);
    assert_eq!(preview["tier"]["long_edge_cap"], 960);
    assert_eq!(preview["width"], 960);
    assert_eq!(preview["height"], 480);
    assert_eq!(
        encoded_frame(Path::new(preview["path"].as_str().unwrap())),
        (960, 480)
    );

    // The disclosure names the degradation rather than implying it from a number.
    let said = preview["tier"]["disclosure"].as_str().unwrap_or_default();
    assert!(said.contains("degraded"), "{said}");
    assert!(said.contains("540p"), "{said}");
    assert!(said.contains("720p"), "{said}");

    // Exactly one step: two rungs tried, the first missed, the second did not.
    let attempts = preview["attempts"].as_array().expect("attempts");
    assert_eq!(attempts.len(), 2);
    assert_eq!(attempts[0]["tier"], "720p");
    assert_eq!(attempts[0]["missed"], true);
    assert_eq!(attempts[1]["tier"], "540p");
    assert_eq!(attempts[1]["missed"], false);
}

#[test]
fn a_540p_miss_hard_fails_and_there_is_no_third_tier() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let path = write_project(&dir, "project.json", &project(2000, 1000));
    let answer = run(
        &path,
        &Ask {
            clock: Clock::Stated(vec![Duration::ZERO]),
            ..Ask::default()
        },
    );

    assert!(answer.preview().is_none(), "a hard fail carries no video");
    assert_eq!(answer.report().exit_code(), ExitCode::BadInvocation);
    let said = refusal(&answer);
    assert!(said.contains("720p"), "{said}");
    assert!(said.contains("540p"), "{said}");
    assert!(said.contains("gives up"), "{said}");
    // The refusal is the wall-clock one, and says so: the other floor is a different
    // refusal and must not be reported as this one (ADR-0067).
    assert!(said.contains("wall-clock refusal"), "{said}");
    assert!(!said.contains("legible"), "{said}");
    // Nothing is left behind: not the deliverable, and not a half-written preview.
    assert!(!dir.join("out/video.mp4").exists());
    assert!(
        !dir.join("out/video.preview.0-200.mp4").exists(),
        "an abandoned rung leaves no file"
    );
}

#[test]
fn a_project_with_nothing_left_to_degrade_to_fails_rather_than_re_render_the_same_frame() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    // Inside the 540p cap already, so the second rung would rasterize the identical
    // surface. The ladder stops rather than pay for a second identical attempt.
    let path = write_project(&dir, "project.json", &project(400, 300));
    let answer = run(
        &path,
        &Ask {
            clock: Clock::Stated(vec![Duration::ZERO]),
            ..Ask::default()
        },
    );
    assert!(answer.preview().is_none());
    assert_eq!(answer.report().exit_code(), ExitCode::BadInvocation);
    let json = answer.to_json();
    assert!(json["preview"].is_null());
}

#[test]
fn a_project_between_the_two_caps_degrades_from_true_pixels_to_a_real_proxy() {
    if !has_ffprobe() {
        return;
    }
    // 1200 px long edge: inside the 720p cap of 1280 and outside the 540p cap of 960. The
    // ladder is defined on *caps*, not on sizes, so the first rung is true pixels — ADR-0046:
    // "for a 1280x720-native or smaller project, no proxy applies at all" — and the second is
    // a real proxy. Ratified by ADR-0078 (#295, reading 10).
    let dir = tempdir(line!());
    let path = write_project(&dir, "project.json", &project(1200, 600));

    // The first rung on its own: true pixels, and the disclosure says so rather than naming
    // a tier whose cap never engaged (reading 11).
    let native = previewed(&path, &Ask::default());
    assert_eq!(native["preview"]["tier"]["name"], "native");
    assert_eq!(native["preview"]["tier"]["proxied"], false);
    assert!(native["preview"]["tier"]["long_edge_cap"].is_null());
    assert_eq!(native["preview"]["width"], 1200);

    // And on a miss it degrades anyway, even though nothing proxied it to begin with.
    let json = previewed(
        &path,
        &Ask {
            clock: Clock::Stated(vec![Duration::ZERO, Duration::from_secs(600)]),
            ..Ask::default()
        },
    );
    let preview = &json["preview"];
    assert_eq!(preview["tier"]["name"], "540p");
    assert_eq!(preview["tier"]["degraded"], true);
    assert_eq!(preview["tier"]["proxied"], true);
    assert_eq!(preview["tier"]["long_edge_cap"], 960);
    assert_eq!(
        (preview["width"].as_i64(), preview["height"].as_i64()),
        (Some(960), Some(480))
    );
    assert_eq!(
        encoded_frame(Path::new(preview["path"].as_str().unwrap())),
        (960, 480)
    );

    // The rung it came down from is the one that rendered true pixels — and it is named for
    // what it rendered, not for the cap that never engaged on it. One invocation may not
    // name the same frame two ways: `native` in the disclosure and `720p` in the trace.
    let attempts = preview["attempts"].as_array().expect("attempts");
    assert_eq!(attempts.len(), 2);
    assert_eq!(attempts[0]["width"], 1200);
    assert_eq!(attempts[0]["missed"], true);
    assert_eq!(attempts[0]["tier"], "native");

    // And the degraded sentence says what actually ran, rather than asserting that a 720p
    // proxy ran when true pixels did.
    let said = preview["tier"]["disclosure"].as_str().unwrap_or_default();
    assert!(said.contains("native (1200x600) ran to"), "{said}");
}

#[test]
fn the_budget_is_judged_per_attempt_and_wall_ms_is_the_whole_invocation() {
    if !has_ffprobe() {
        return;
    }
    // Each rung gets its own clock, and the one disclosed is the rung that landed — not the
    // whole invocation's, and not what the first rung left over. A whole-invocation clock
    // would leave the second rung whatever the first miss did not spend, which is usually
    // nothing, so ADR-0021's required degrade step could never land. Ratified by ADR-0078
    // (#295, readings 4 and 5).
    let dir = tempdir(line!());
    let path = write_project(&dir, "project.json", &project(2000, 1000));
    let json = previewed(
        &path,
        &Ask {
            clock: Clock::Stated(vec![Duration::ZERO, Duration::from_secs(600)]),
            ..Ask::default()
        },
    );
    let preview = &json["preview"];

    // The second rung's own number, not the first's and not their difference.
    assert_eq!(preview["tier"]["budget_ms"], 600_000);

    // The answer's `wall_ms` is the whole invocation, attempts included, because that is
    // what the caller actually waited.
    let attempts = preview["attempts"].as_array().expect("attempts");
    let spent: u64 = attempts
        .iter()
        .map(|a| a["wall_ms"].as_u64().expect("wall_ms"))
        .sum();
    let whole = preview["wall_ms"].as_u64().expect("wall_ms");
    assert!(whole >= spent, "{whole} ms covers every rung's {spent} ms");

    // A rung past its deadline is abandoned where it stands rather than finished and then
    // judged: it encodes fewer frames than the span has.
    let span = preview["frames"].as_u64().expect("frames");
    assert!(
        attempts[0]["frames"].as_u64().expect("frames") < span,
        "the missed rung ran to completion: {attempts:?}"
    );
    assert_eq!(attempts[1]["frames"].as_u64(), Some(span));
}

// ---------------------------------------------------------------------------
// The floors (story 62a, ADR-0050 / ADR-0067)
// ---------------------------------------------------------------------------

#[test]
fn the_legibility_floor_does_not_govern_a_project_that_declares_a_small_frame() {
    if !has_ffprobe() {
        return;
    }
    // 320×180 is below the 360p floor as a number, and is previewed anyway: nothing
    // downscaled it, so there is no proxy for the floor to judge, and `preview` is not the
    // verb that tells an author their project is too small (ADR-0067's guard is on a
    // *proxy* resolution).
    let dir = tempdir(line!());
    let path = write_project(&dir, "project.json", &project(320, 180));
    let json = previewed(&path, &Ask::default());
    assert_eq!(json["preview"]["width"], 320);
    assert_eq!(json["preview"]["tier"]["proxied"], false);
}

#[test]
fn the_two_floors_are_two_constants_and_the_ladder_never_reaches_the_lower_one() {
    // The claim ADR-0067 makes in prose, as an assertion: the ladder's rungs are both
    // clear of the legibility floor, so it cannot fire today — and it is a different
    // number from the one that stops the ladder.
    use montagent_render::proxy::{
        LEGIBILITY_FLOOR_LONG_EDGE, TARGET_LONG_EDGE, Tier, WALL_CLOCK_GIVE_UP_LONG_EDGE, admit,
    };
    assert_ne!(WALL_CLOCK_GIVE_UP_LONG_EDGE, LEGIBILITY_FLOOR_LONG_EDGE);
    assert!(admit(TARGET_LONG_EDGE).is_ok());
    assert!(admit(WALL_CLOCK_GIVE_UP_LONG_EDGE).is_ok());
    assert_eq!(Tier::Degraded.next(), None, "there is no third rung");

    // And below it, the refusal names the floor and the legibility reason — a different
    // sentence from the wall-clock one, because it is a different refusal (ADR-0050).
    let said = admit(LEGIBILITY_FLOOR_LONG_EDGE - 2).expect_err("below the floor");
    assert!(said.contains("360p floor"), "{said}");
    assert!(said.contains("legible"), "{said}");
    assert!(!said.contains("budget"), "{said}");
}

// ---------------------------------------------------------------------------
// Disclosure, and what `preview` is not (stories 62, 63)
// ---------------------------------------------------------------------------

#[test]
fn every_preview_discloses_its_tier_whether_or_not_it_degraded() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let path = write_project(&dir, "project.json", &project(1920, 1080));
    for ask in [
        Ask::default(),
        Ask {
            full: true,
            ..Ask::default()
        },
        Ask {
            from: Some(40),
            to: Some(120),
            ..Ask::default()
        },
    ] {
        let json = previewed(&path, &ask);
        let tier = &json["preview"]["tier"];
        assert!(!tier.is_null(), "the disclosure is never absent");
        for key in [
            "name",
            "width",
            "height",
            "declared_width",
            "declared_height",
            "proxied",
            "degraded",
            "disclosure",
        ] {
            assert!(!tier[key].is_null(), "{key} is always present");
        }
        assert!(
            !tier["disclosure"].as_str().unwrap_or_default().is_empty(),
            "the sentence is never empty"
        );
    }
}

#[test]
fn the_prose_states_the_tier_above_the_files_own_numbers() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let path = write_project(&dir, "project.json", &project(1920, 1080));
    let answer = run(&path, &Ask::default());
    let text = montagent_core::wire::render_preview(
        &answer,
        montagent_core::wire::Wire::from_flags(false, false),
    );
    assert!(text.contains("PREVIEW"), "{text}");
    let tier_at = text.find("tier").expect("a tier line");
    let range_at = text.find("range").expect("a range line");
    assert!(tier_at < range_at, "the disclosure comes first:\n{text}");
    assert!(text.contains("720p"), "{text}");
    assert!(text.contains("budget"), "{text}");
}

#[test]
fn the_degraded_prose_names_every_rung_the_ladder_tried() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let path = write_project(&dir, "project.json", &project(2000, 1000));
    let answer = run(
        &path,
        &Ask {
            clock: Clock::Stated(vec![Duration::ZERO, Duration::from_secs(600)]),
            ..Ask::default()
        },
    );
    let text = montagent_core::wire::render_preview(
        &answer,
        montagent_core::wire::Wire::from_flags(false, false),
    );
    // An agent told only the tier it ended on cannot tell a project that missed by a
    // tenth of a second from one that missed by four, and those want different next moves.
    assert!(text.contains("tier        540p"), "{text}");
    assert!(text.contains("tried       720p"), "{text}");
    assert!(text.contains("tried       540p"), "{text}");
    assert!(text.contains("missed the budget"), "{text}");
}

#[test]
fn a_preview_may_never_land_on_the_deliverable() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let path = write_project(&dir, "project.json", &project(400, 400));
    let answer = run(
        &path,
        &Ask {
            output: Some(dir.join("out/video.mp4")),
            ..Ask::default()
        },
    );
    assert!(answer.preview().is_none());
    assert_eq!(answer.report().exit_code(), ExitCode::BadInvocation);
    let said = refusal(&answer);
    assert!(said.contains("never land on the deliverable"), "{said}");
    assert!(!dir.join("out/video.mp4").exists());

    // A different explicit path is taken as given.
    let elsewhere = dir.join("scratch/look.mp4");
    let json = previewed(
        &path,
        &Ask {
            output: Some(elsewhere.clone()),
            ..Ask::default()
        },
    );
    assert_eq!(
        json["preview"]["path"]
            .as_str()
            .map(std::path::PathBuf::from),
        Some(elsewhere)
    );
}

#[test]
fn render_is_never_degraded_however_the_preview_of_the_same_project_lands() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let path = write_project(&dir, "project.json", &project(2000, 1000));

    // The preview degrades all the way to its last rung...
    let preview_json = previewed(
        &path,
        &Ask {
            clock: Clock::Stated(vec![Duration::ZERO, Duration::from_secs(600)]),
            ..Ask::default()
        },
    );
    assert_eq!(preview_json["preview"]["tier"]["name"], "540p");

    // ...and the deliverable is still the declared frame, at true pixels, from a verb that
    // has no tier to report (ADR-0067: "nothing about `render` changes").
    let answer = montagent_core::verbs::render::render(
        &path,
        &montagent_core::verbs::render::Ask::default(),
        &mut |_| {},
    );
    let json = answer.to_json();
    assert_eq!(answer.report().exit_code(), ExitCode::Ok, "{json}");
    assert_eq!(json["render"]["width"], 2000);
    assert_eq!(json["render"]["height"], 1000);
    assert!(json["render"]["tier"].is_null(), "`render` has no tier");
    assert_eq!(
        encoded_frame(&dir.join("out/video.mp4")),
        (2000, 1000),
        "the deliverable is never proxy-scaled"
    );
}

#[test]
fn preview_runs_the_same_checks_render_runs_and_refuses_on_an_error() {
    let dir = tempdir(line!());
    // Two elements at one layer, overlapping in time and space: the layer-tie error
    // (ADR-0060), which is an `error` and gates a render.
    let body = canonical(
        r##"{"frame":{"width":400,"height":400},"fps":25,"background":"#000000",
            "duration":200,"output":"out/video.mp4",
            "tracks":[{"name":"only","layer":0,"elements":[
              {"id":"a","type":"rect","start":0,"end":200,"x":200,"y":200,
               "width":100,"height":100,"fill":"#FF0000"},
              {"id":"b","type":"rect","start":0,"end":200,"x":200,"y":200,
               "width":100,"height":100,"fill":"#00FF00","track":"only"}]}]}"##,
    );
    let path = write_project(&dir, "project.json", &body);
    let answer = run(&path, &Ask::default());
    let render = montagent_core::verbs::render::render(
        &path,
        &montagent_core::verbs::render::Ask::default(),
        &mut |_| {},
    );
    // Whatever the checks say about this document, both verbs say the same thing about it.
    assert_eq!(
        answer.report().exit_code(),
        render.report().exit_code(),
        "preview and render answer the same document differently"
    );
    if render.report().exit_code() != ExitCode::Ok {
        assert!(answer.preview().is_none(), "a refused preview has no video");
    }
}

// ---------------------------------------------------------------------------
// Feeds (ADR-0141): `preview` inherits `render`'s feed supplier through `encode_span`
// ---------------------------------------------------------------------------

/// [`project`] with one `video` element over its whole 200 ms, on a generated H.264 source.
fn project_with_video(dir: &Path) -> std::path::PathBuf {
    let tools = montagent_core::media::tools::resolve().expect("ffmpeg");
    let clip = dir.join("clip.mp4");
    let made = Command::new(&tools.ffmpeg)
        .args(["-hide_banner", "-loglevel", "error", "-y", "-f", "lavfi"])
        .args(["-i", "testsrc2=s=64x48:r=25", "-frames:v", "25"])
        .args(["-c:v", "libx264", "-pix_fmt", "yuv420p"])
        .arg(&clip)
        .status()
        .expect("ffmpeg runs");
    assert!(made.success(), "the source could not be generated");
    let source = clip.display().to_string().replace('\\', "/");
    write_project(
        dir,
        "project.json",
        &canonical(&format!(
            r##"{{"frame":{{"width":2000,"height":1000}},"fps":25,"background":"#000000",
                "duration":200,"output":"out/video.mp4",
                "tracks":[{{"name":"only","layer":0,"elements":[
                  {{"id":"clip","type":"video","start":0,"end":200,"source":"{source}",
                    "source_start":0,"source_end":200,"x":1000,"y":500,"width":640,
                    "height":480,"fit":"literal","volume":0}}]}}]}}"##
        )),
    )
}

#[test]
fn a_preview_over_a_video_element_opens_one_feed_and_leaves_none_open() {
    use montagent_core::verbs::frame::supply;
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let path = project_with_video(&dir);

    // One span, one feed: not one `ffmpeg` per frame.
    supply::reset_counts();
    let json = previewed(&path, &Ask::default());
    let counts = supply::counts();
    assert_eq!((counts.opened, counts.reopened, counts.frame_at), (1, 0, 0));
    assert_eq!(counts.open, 0);
    assert_eq!(json["preview"]["decoded_per_frame"], serde_json::json!([]));

    // Abandoned at an artificially short deadline, on both rungs: the miss is reported as it
    // always was, and the feeds went with the abandoned spans — `Drop`, and nothing else.
    supply::reset_counts();
    let answer = run(
        &path,
        &Ask {
            clock: Clock::Stated(vec![Duration::ZERO]),
            ..Ask::default()
        },
    );
    assert!(answer.preview().is_none());
    assert_eq!(answer.report().exit_code(), ExitCode::BadInvocation);
    assert!(
        refusal(&answer).contains("gives up"),
        "{}",
        refusal(&answer)
    );
    assert_eq!(
        supply::counts().open,
        0,
        "an abandoned span leaves no feed open"
    );

    // 720p misses and 540p lands: each attempt builds its own painter, so the retry opens a
    // fresh feed at the span's first frame, and neither is left open.
    supply::reset_counts();
    let json = previewed(
        &path,
        &Ask {
            clock: Clock::Stated(vec![Duration::ZERO, Duration::from_secs(600)]),
            ..Ask::default()
        },
    );
    assert_eq!(json["preview"]["tier"]["name"], "540p");
    let counts = supply::counts();
    assert_eq!(
        (counts.opened, counts.reopened),
        (2, 0),
        "one feed per attempt"
    );
    assert_eq!(counts.open, 0, "a successful degrade leaves no feed open");
}
