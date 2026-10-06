//! **Time remap**: `source_time` on a `video` (ADR-0157, #745).
//!
//! A `video` carrying `source_time` names which moment of its file is on screen at every
//! instant: a literal is a freeze frame, a keyframe list a curve whose slope is the rate. A
//! steep segment plays fast, a flat one freezes, a falling one plays in reverse. The curve is
//! the only author of the source, so `source_start`, `source_end`, `speed` and `overrun` are
//! refused beside it, and the element is silent.
//!
//! Each case is asked through a verb: `validate`, `query --at`, the painter (through
//! `render::paint_span`), and `shift`.

mod common;

use std::path::{Path, PathBuf};
use std::process::Command;

use montagent_core::finding::{Class, Finding};
use montagent_core::media::tools;
use montagent_core::report::Report;
use montagent_core::validate;
use montagent_core::verbs::frame::supply::{self, Counts};
use montagent_core::verbs::render::{Supplying, paint_span};
use serde_json::{Value, json};

use common::{canonical, has_ffprobe, tempdir, write_project};

/// Frames in the generated clip: 82 at 25 fps, so it is 3280 ms long.
const CLIP_FRAMES: i64 = 82;

/// H.264 frames of a moving test pattern at 25 fps, every frame unlike its neighbours, so a
/// frame off by one cannot pass.
fn clip(dir: &Path) -> PathBuf {
    let path = dir.join("clip.mp4");
    let ok = Command::new(tools::resolve().expect("an ffmpeg").ffmpeg)
        .args(["-hide_banner", "-loglevel", "error", "-y", "-f", "lavfi"])
        .args(["-i", "testsrc2=s=64x48:r=25"])
        .args(["-frames:v", &CLIP_FRAMES.to_string()])
        .args([
            "-c:v", "libx264", "-pix_fmt", "yuv420p", "-g", "50", "-bf", "2",
        ])
        .arg(&path)
        .status()
        .expect("ffmpeg runs")
        .success();
    assert!(ok, "libx264 is ADR-0115's floor");
    path
}

/// A `video` element on `clip.mp4` over `[start, end)`, with `fields` written over it.
fn video(id: &str, (start, end): (i64, i64), fields: Value) -> Value {
    let mut element = json!({"id": id, "type": "video", "start": start, "end": end,
        "source": "clip.mp4", "x": 0, "y": 0, "origin": "top-left", "width": 64,
        "height": 48, "fit": "literal"});
    for (key, value) in fields.as_object().expect("an object of fields") {
        element[key] = value.clone();
    }
    element
}

/// A 64x48 project at 25 fps holding `elements` on one track, written beside a fresh clip.
fn project(line: u32, elements: Vec<Value>) -> PathBuf {
    let dir = tempdir(line);
    clip(&dir);
    let project = json!({
        "frame": {"width": 64, "height": 48}, "fps": 25, "background": "#000000",
        "output": "out/remap.mp4",
        "tracks": [{"name": "a", "layer": 0, "elements": elements}],
    });
    write_project(
        &dir,
        "remap.montagent.json",
        &canonical(&project.to_string()),
    )
}

fn codes(report: &Report) -> Vec<&str> {
    report.findings.iter().map(|f| f.code.as_str()).collect()
}

fn with_code<'r>(report: &'r Report, code: &str) -> Vec<&'r Finding> {
    report.findings.iter().filter(|f| f.code == code).collect()
}

// ---------------------------------------------------------------------------
// The field.
// ---------------------------------------------------------------------------

#[test]
fn a_video_takes_source_time_as_a_literal_or_a_keyframe_list_in_place_of_a_source_range() {
    if !has_ffprobe() {
        return;
    }
    let literal = video("still", (0, 400), json!({"source_time": 1200, "volume": 0}));
    let keyed = video(
        "ramp",
        (400, 1200),
        // The last key on the last painted frame, 1160 ms at 25 fps: a key at `end` is never
        // reached, and the frames between would hold its approach.
        json!({"source_time": [{"t": 400, "v": 0}, {"t": 1160, "v": 1900, "ease": "linear"}],
               "volume": 0}),
    );
    let report = validate(&project(line!(), vec![literal, keyed]));
    assert_eq!(codes(&report), Vec::<&str>::new(), "{:?}", report.findings);
}

#[test]
fn a_negative_or_fractional_source_time_is_a_schema_error() {
    if !has_ffprobe() {
        return;
    }
    for written in [
        json!(-1),
        json!(12.5),
        json!([{"t": 0, "v": 0}, {"t": 400, "v": -40, "ease": "linear"}]),
        json!([{"t": 0, "v": 0}, {"t": 400, "v": 40.5, "ease": "linear"}]),
    ] {
        let element = video("v", (0, 400), json!({"source_time": written, "volume": 0}));
        let report = validate(&project(line!(), vec![element]));
        assert_eq!(
            with_code(&report, "E-SCHEMA").len(),
            1,
            "{written}: {:?}",
            report.findings
        );
    }
}

#[test]
fn source_time_on_audio_is_a_schema_error() {
    if !has_ffprobe() {
        return;
    }
    let audio = json!({"id": "a", "type": "audio", "start": 0, "end": 400,
        "source": "clip.mp4", "source_start": 0, "source_end": 400, "source_time": 0});
    let report = validate(&project(line!(), vec![audio]));
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.code.starts_with("E-SCHEMA") && f.class == Class::Error),
        "{:?}",
        report.findings
    );
}

#[test]
fn a_video_with_neither_a_source_range_nor_source_time_is_a_schema_error() {
    if !has_ffprobe() {
        return;
    }
    let element = video("v", (0, 400), json!({"volume": 0}));
    let report = validate(&project(line!(), vec![element]));
    let schema = with_code(&report, "E-SCHEMA");
    assert_eq!(schema.len(), 1, "{:?}", report.findings);
    let reason = schema[0].fields["reason"].as_str().unwrap();
    assert!(
        reason.contains("source_start") && reason.contains("source_time"),
        "{reason}"
    );
}

// ---------------------------------------------------------------------------
// `validate`.
// ---------------------------------------------------------------------------

#[test]
fn each_source_range_speed_and_overrun_field_beside_source_time_is_its_own_error() {
    if !has_ffprobe() {
        return;
    }
    // Every field a finding of its own, and none of ADR-0020's two invariant checks: a
    // 100 ms source range at `speed` 1 on a 1000 ms element would be both, were it played.
    let element = video(
        "v",
        (0, 1000),
        json!({"source_start": 0, "source_end": 100, "source_time": 0, "speed": 1,
               "overrun": "hold", "volume": 0}),
    );
    let report = validate(&project(line!(), vec![element]));
    let refused = with_code(&report, "E-REMAP-FIELD");
    let fields: Vec<&str> = refused
        .iter()
        .map(|f| f.fields["field"].as_str().unwrap())
        .collect();
    assert_eq!(
        fields,
        ["source_start", "source_end", "speed", "overrun"],
        "{:?}",
        report.findings
    );
    for finding in &refused {
        assert_eq!(finding.class, Class::Error);
        assert_eq!(finding.location.element.as_deref(), Some("v"));
        let repair = serde_json::to_value(&finding.repair).unwrap();
        let field = finding.fields["field"].as_str().unwrap();
        assert!(
            repair.to_string().contains(&format!("remove `{field}`")),
            "{repair}"
        );
    }
    for code in ["E-SPEED-MISMATCH", "E-OVERRUN-UNNEEDED"] {
        assert!(
            with_code(&report, code).is_empty(),
            "{code} ran: {:?}",
            report.findings
        );
    }
    assert!(
        prose(&report).contains("`speed` beside it"),
        "{}",
        prose(&report)
    );
}

#[test]
fn a_remapped_video_that_is_not_written_silent_is_an_error() {
    if !has_ffprobe() {
        return;
    }
    let audible = [
        ("a number", json!({"source_time": 0, "volume": 1})),
        (
            "a keyframe list of zeros",
            json!({"source_time": 0, "volume": [{"t": 0, "v": 0}, {"t": 500, "v": 0, "ease": "linear"}]}),
        ),
        ("no volume", json!({"source_time": 0})),
    ];
    for (what, fields) in audible {
        let report = validate(&project(line!(), vec![video("v", (0, 1000), fields)]));
        let found = with_code(&report, "E-REMAP-AUDIBLE");
        assert_eq!(found.len(), 1, "{what}: {:?}", report.findings);
        assert_eq!(found[0].class, Class::Error);
        assert_eq!(found[0].location.element.as_deref(), Some("v"));
        let repair = serde_json::to_value(&found[0].repair).unwrap().to_string();
        assert!(
            repair.contains("set `volume` to 0") && repair.contains("separate `audio` element"),
            "{repair}"
        );
        assert!(prose(&report).contains("a remapped video is silent"));
    }
    let silent = video("v", (0, 1000), json!({"source_time": 0, "volume": 0}));
    let report = validate(&project(line!(), vec![silent]));
    assert!(with_code(&report, "E-REMAP-AUDIBLE").is_empty());
}

#[test]
fn a_curve_past_the_files_end_is_a_source_overrun_named_at_its_first_painted_instant() {
    if !has_ffprobe() {
        return;
    }
    // 3000 ms rising at 1×: the first painted instant whose source time reaches the file's
    // end is the first multiple of 40 at or past `duration − 3000`.
    let element = video(
        "v",
        (0, 1000),
        json!({"source_time": [{"t": 0, "v": 3000}, {"t": 1000, "v": 4000, "ease": "linear"}],
               "volume": 0}),
    );
    let report = validate(&project(line!(), vec![element]));
    let found = with_code(&report, "E-SOURCE-OVERRUN");
    assert_eq!(found.len(), 1, "{:?}", report.findings);
    let finding = found[0];
    let duration = finding.fields["probed_duration"].as_i64().unwrap();
    let first = (duration - 3000 + 39) / 40 * 40;
    assert_eq!(finding.fields["instant"], first);
    assert_eq!(finding.fields["source_time"], 3000 + first);
    assert_eq!(finding.fields["side"], "at or past the file's end");
    assert_eq!(finding.class, Class::Error);
    assert!(!finding.fields.contains_key("source_start"));
    let said = prose(&report);
    assert!(
        said.contains(&format!(
            "holds {duration} ms (video stream), and its `source_time` resolves to {} ms at the \
             painted instant {first} ms, at or past the file's end.",
            3000 + first
        )),
        "{said}"
    );
}

#[test]
fn the_overrun_check_and_the_painter_resolve_every_frame_through_one_function() {
    if !has_ffprobe() {
        return;
    }
    // An eased overshoot that ends past the file: every painted frame's offset — the one the
    // painter decodes at, read off the caption it paints from — is the one function's value,
    // and the check's first offending instant is the first frame where that value leaves
    // `[0, duration)`.
    let element = video(
        "v",
        (0, 2000),
        json!({"source_time": [{"t": 0, "v": 400},
                               {"t": 1960, "v": 3400, "ease": [0.3, -0.6, 0.6, 1.4]}],
               "volume": 0}),
    );
    let path = project(line!(), vec![element.clone()]);
    let report = validate(&path);
    let found = with_code(&report, "E-SOURCE-OVERRUN");
    assert_eq!(found.len(), 1, "{:?}", report.findings);
    let duration = found[0].fields["probed_duration"].as_i64().unwrap();
    let mut first_outside = None;
    for frame in 0..50 {
        let instant = frame * 40;
        let resolved = montagent_core::remap::source_ms(&element, instant).unwrap();
        let caption = queried(&path, instant, "v");
        assert_eq!(caption["source_offset"], resolved, "at {instant}");
        assert_eq!(caption["derived"]["source_time"], resolved, "at {instant}");
        if first_outside.is_none() && !(0..duration).contains(&resolved) {
            first_outside = Some((instant, resolved));
        }
    }
    let (instant, resolved) = first_outside.expect("the curve leaves the file");
    assert_eq!(found[0].fields["instant"], instant);
    assert_eq!(found[0].fields["source_time"], resolved);
}

/// The report as the text form prints it, every finding through its registered template.
fn prose(report: &Report) -> String {
    montagent_core::text::render(&report.to_json(), montagent_core::text::Options::verbose())
        .expect("the report renders")
}

#[test]
fn an_ease_overshooting_below_the_files_start_is_a_source_overrun() {
    if !has_ffprobe() {
        return;
    }
    // A bezier whose first handle dips below 0: the curve leaves 0 downwards first.
    let element = video(
        "v",
        (0, 1000),
        json!({"source_time": [{"t": 0, "v": 0},
                               {"t": 960, "v": 960, "ease": [0.5, -1.0, 0.5, 1.0]}],
               "volume": 0}),
    );
    let path = project(line!(), vec![element]);
    let report = validate(&path);
    let found = with_code(&report, "E-SOURCE-OVERRUN");
    assert_eq!(found.len(), 1, "{:?}", report.findings);
    let finding = found[0];
    assert_eq!(finding.fields["side"], "below 0");
    let instant = finding.fields["instant"].as_i64().unwrap();
    assert!(finding.fields["source_time"].as_i64().unwrap() < 0);
    // Named at the first: the frame before it shows source 0 or later.
    assert_eq!(instant % 40, 0);
    assert_eq!(
        instant, 40,
        "frame 0 shows source 0, and frame 1 already dips"
    );
}

#[test]
fn a_curve_holding_past_its_keys_on_painted_frames_is_a_review_at_each_end() {
    if !has_ffprobe() {
        return;
    }
    let held = video(
        "held",
        (0, 1000),
        json!({"source_time": [{"t": 200, "v": 0}, {"t": 600, "v": 400, "ease": "linear"}],
               "volume": 0}),
    );
    let report = validate(&project(line!(), vec![held]));
    let found = with_code(&report, "R-REMAP-HELD-END");
    let ends: Vec<(&str, i64)> = found
        .iter()
        .map(|f| {
            (
                f.fields["end"].as_str().unwrap(),
                f.fields["held_ms"].as_i64().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        ends,
        [("before the first key", 200), ("after the last key", 400)]
    );
    assert!(found.iter().all(|f| f.class == Class::Review));
    assert!(
        prose(&report).contains("written as a flat pair of keys"),
        "{}",
        prose(&report)
    );

    // A literal never fires it, nor a held stretch no painted frame instant falls in: at
    // 25 fps nothing is painted in [10, 30), and the last key sits on the last frame.
    let literal = video(
        "literal",
        (0, 1000),
        json!({"source_time": 400, "volume": 0}),
    );
    let tight = video(
        "tight",
        (10, 1000),
        json!({"source_time": [{"t": 30, "v": 0}, {"t": 960, "v": 930, "ease": "linear"}],
               "volume": 0}),
    );
    let report = validate(&project(line!(), vec![literal, tight]));
    assert!(
        with_code(&report, "R-REMAP-HELD-END").is_empty(),
        "{:?}",
        report.findings
    );
}

// ---------------------------------------------------------------------------
// The painter.
// ---------------------------------------------------------------------------

/// The frames of `[0, to)` the project at `path` paints, through each supplier, asserted byte
/// for byte equal between the two (ADR-0141): `frame`'s per-frame decode and `render`'s feeds.
#[track_caller]
fn painted(path: &Path, to: i64) -> Vec<Vec<u8>> {
    let mut by_supplier = [Supplying::PerFrame, Supplying::Feeds].map(|supplying| {
        let rasters = paint_span(path, 0, to, supplying).expect("the span paints");
        assert!(
            rasters.declined.is_empty(),
            "{supplying:?} declined: {:?}",
            rasters.declined
        );
        rasters.frames
    });
    let [per_frame, feeds] = &mut by_supplier;
    assert_eq!(per_frame.len(), feeds.len());
    for (n, (want, got)) in per_frame.iter().zip(feeds.iter()).enumerate() {
        assert!(want == got, "frame {n}: the feeds paint other pixels");
    }
    std::mem::take(per_frame)
}

/// One `video` element alone in a project, painted over `[0, 1000)`: 25 frames.
#[track_caller]
fn painted_alone(line: u32, fields: Value) -> Vec<Vec<u8>> {
    let path = project(line, vec![video("v", (0, 1000), fields)]);
    painted(&path, 1000)
}

#[test]
fn a_linear_curve_at_one_times_paints_the_frames_of_the_source_range_it_spells() {
    if !has_ffprobe() {
        return;
    }
    let s = 600;
    let ranged = painted_alone(
        line!(),
        json!({"source_start": s, "source_end": s + 1000, "volume": 0}),
    );
    let remapped = painted_alone(
        line!(),
        json!({"source_time": [{"t": 0, "v": s}, {"t": 1000, "v": s + 1000, "ease": "linear"}],
               "volume": 0}),
    );
    assert_eq!(ranged.len(), 25);
    assert!(ranged == remapped, "the curve paints other frames");
    assert_ne!(ranged[0], ranged[1], "the clip moves");
}

#[test]
fn a_literal_source_time_paints_one_source_frame_on_every_frame() {
    if !has_ffprobe() {
        return;
    }
    let frozen = painted_alone(line!(), json!({"source_time": 1200, "volume": 0}));
    let at_1200 = painted_alone(
        line!(),
        json!({"source_start": 1200, "source_end": 2200, "volume": 0}),
    );
    assert!(frozen.iter().all(|frame| *frame == at_1200[0]));
    assert_ne!(at_1200[0], at_1200[1], "the clip moves");
}

#[test]
fn a_falling_linear_segment_paints_the_source_frames_in_reverse_order() {
    if !has_ffprobe() {
        return;
    }
    // Frame k of the forward range shows source 40k ms. The falling curve shows 960 − 40k at
    // frame k: source frame 24 − k, so the order is the forward order reversed, frame by frame.
    let forward = painted_alone(
        line!(),
        json!({"source_start": 0, "source_end": 1000, "volume": 0}),
    );
    let reversed = painted_alone(
        line!(),
        json!({"source_time": [{"t": 0, "v": 960}, {"t": 960, "v": 0, "ease": "linear"}],
               "volume": 0}),
    );
    assert_eq!(forward.len(), 25);
    for (k, frame) in reversed.iter().enumerate() {
        assert!(
            *frame == forward[24 - k],
            "frame {k} shows something other than source frame {}",
            24 - k
        );
    }
}

/// What painting `[0, to)` through `render`'s feeds spawned.
fn spawned(path: &Path, to: i64) -> Counts {
    supply::reset_counts();
    paint_span(path, 0, to, Supplying::Feeds).expect("the span paints");
    let counts = supply::counts();
    assert_eq!(counts.open, 0, "every feed is closed with its painter");
    counts
}

#[test]
fn a_feed_serves_only_a_rising_one_times_stretch_and_every_other_frame_is_decoded_alone() {
    if !has_ffprobe() {
        return;
    }
    // 1× to 400 ms (frames 0–10), 0.3× to 1200 (frames 11–29), 1× to 1960 (frames 30–48).
    let ramp = video(
        "ramp",
        (0, 1960),
        json!({"source_time": [{"t": 0, "v": 0}, {"t": 400, "v": 400, "ease": "linear"},
                               {"t": 1200, "v": 640, "ease": "linear"},
                               {"t": 1960, "v": 1400, "ease": "linear"}],
               "volume": 0}),
    );
    let path = project(line!(), vec![ramp]);
    painted(&path, 1960);
    assert_eq!(
        spawned(&path, 1960),
        Counts {
            opened: 1,
            reopened: 1,
            frame_at: 19,
            open: 0
        },
        "a feed for each 1× stretch, and one decode for each slow frame"
    );

    // A reverse never rides a feed; a freeze decodes its one frame once.
    let reverse = video(
        "reverse",
        (0, 1000),
        json!({"source_time": [{"t": 0, "v": 960}, {"t": 960, "v": 0, "ease": "linear"}],
               "volume": 0}),
    );
    let path = project(line!(), vec![reverse]);
    assert_eq!(
        spawned(&path, 1000),
        Counts {
            frame_at: 25,
            ..Counts::default()
        }
    );
    let freeze = video(
        "freeze",
        (0, 1000),
        json!({"source_time": 1200, "volume": 0}),
    );
    let path = project(line!(), vec![freeze]);
    assert_eq!(
        spawned(&path, 1000),
        Counts {
            frame_at: 1,
            ..Counts::default()
        }
    );
}

// ---------------------------------------------------------------------------
// `measure`'s keyed-alpha series.
// ---------------------------------------------------------------------------

/// The keyed-alpha series `measure` reports for `element` over `wipe.mp4`: a green screen a
/// red edge crosses a pixel column a frame, so the key takes out less on every frame.
#[track_caller]
fn keyed_series(line: u32, fields: Value) -> Vec<Value> {
    let mut element = video("keyed", (0, 1000), fields);
    element["source"] = json!("wipe.mp4");
    element["effects"] = json!([{"name": "chroma", "color": "#00FF00", "tolerance": 0.4,
                                 "softness": 0.0, "spill": 0.0}]);
    let path = project(line, vec![element.clone()]);
    let ok = Command::new(tools::resolve().expect("an ffmpeg").ffmpeg)
        .args(["-hide_banner", "-loglevel", "error", "-y", "-f", "lavfi"])
        .args([
            "-i",
            "color=c=black:s=64x48:r=25,format=rgb24,\
             geq=r='if(lt(X,N),255,0)':g='if(lt(X,N),0,255)':b=0",
        ])
        .args(["-frames:v", &CLIP_FRAMES.to_string()])
        .args(["-c:v", "libx264", "-pix_fmt", "yuv420p"])
        .arg(path.with_file_name("wipe.mp4"))
        .status()
        .expect("ffmpeg runs")
        .success();
    assert!(ok);
    let answer = montagent_core::verbs::measure::measure(
        &path,
        &montagent_core::verbs::measure::Ask {
            element: Some(element),
            ..Default::default()
        },
    )
    .to_json();
    assert_eq!(answer["measure"]["mode"], "coverage", "{answer}");
    answer["measure"]["frames"]
        .as_array()
        .expect("a series")
        .clone()
}

#[test]
fn measure_reads_a_remapped_elements_frames_through_its_curve() {
    if !has_ffprobe() {
        return;
    }
    let forward = keyed_series(
        line!(),
        json!({"source_start": 0, "source_end": 1000, "volume": 0}),
    );
    let reversed = keyed_series(
        line!(),
        json!({"source_time": [{"t": 0, "v": 960}, {"t": 960, "v": 0, "ease": "linear"}],
               "volume": 0}),
    );
    assert_eq!(forward.len(), 25);
    assert_eq!(reversed.len(), 25);
    let fractions = |sample: &Value| {
        ["opaque", "partial", "transparent"].map(|key| sample[key].as_f64().unwrap())
    };
    assert_ne!(
        fractions(&forward[0]),
        fractions(&forward[12]),
        "the key takes out a different share as the pattern moves"
    );
    for (k, sample) in reversed.iter().enumerate() {
        assert_eq!(sample["frame"], k);
        assert_eq!(sample["at"], k as i64 * 40);
        assert_eq!(fractions(sample), fractions(&forward[24 - k]), "frame {k}");
    }
}

// ---------------------------------------------------------------------------
// Motion blur (ADR-0155): the frame instant decides a video's source frame.
// ---------------------------------------------------------------------------

#[test]
fn a_curve_is_not_motion_so_a_remapped_video_that_does_not_move_is_still_under_motion_blur() {
    if !has_ffprobe() {
        return;
    }
    let ramp = video(
        "ramp",
        (0, 1000),
        json!({"source_time": [{"t": 0, "v": 0}, {"t": 960, "v": 2400, "ease": "ease-in"}],
               "volume": 0, "motion_blur": {"shutter": 180, "samples": 8}}),
    );
    let path = project(line!(), vec![ramp]);
    let report = validate(&path);
    assert_eq!(
        with_code(&report, "R-MOTION-BLUR-STILL").len(),
        1,
        "{:?}",
        report.findings
    );
    assert_eq!(queried(&path, 400, "ramp")["motion"], "still");
}

#[test]
fn a_moving_remapped_video_under_motion_blur_shows_the_source_frame_of_its_frame_instant() {
    if !has_ffprobe() {
        return;
    }
    // Moving across the frame and ramping fast, blurred over a wide shutter: every sample of
    // a frame shows the one source frame its frame instant resolves, so the blurred element
    // painted on a still source frame (a literal at that millisecond) gives the same bytes.
    let curve = json!([{"t": 0, "v": 0}, {"t": 960, "v": 2880, "ease": "ease-in"}]);
    let x = json!([{"t": 0, "v": -20}, {"t": 960, "v": 20, "ease": "linear"}]);
    let blurred = |source_time: Value| {
        video(
            "v",
            (0, 1000),
            json!({"source_time": source_time, "x": x, "volume": 0,
                   "motion_blur": {"shutter": 360, "samples": 6}}),
        )
    };
    let ramped = painted(&project(line!(), vec![blurred(curve.clone())]), 1000);
    let element = video("probe", (0, 1000), json!({"source_time": curve}));
    for frame in [3usize, 11, 19] {
        let instant = frame as i64 * 40;
        let ms = montagent_core::remap::source_ms(&element, instant).unwrap();
        let frozen = painted(&project(line!(), vec![blurred(json!(ms))]), 1000);
        assert!(
            ramped[frame] == frozen[frame],
            "frame {frame}: the samples show other source frames than {ms} ms"
        );
    }
}

// ---------------------------------------------------------------------------
// `shift`.
// ---------------------------------------------------------------------------

#[test]
fn shift_refuses_a_cut_inside_a_remapped_element_and_moves_one_whole_with_its_values() {
    if !has_ffprobe() {
        return;
    }
    use montagent_core::verbs::shift::{Ask, shift};
    let ramp = video(
        "ramp",
        (1000, 2000),
        json!({"source_time": [{"t": 1000, "v": 500}, {"t": 1960, "v": 1460, "ease": "linear"}],
               "volume": 0}),
    );
    let path = project(line!(), vec![ramp]);
    let before = std::fs::read_to_string(&path).unwrap();
    let ask = |at| Ask {
        at,
        delta: 100,
        scope: None,
        release: Vec::new(),
    };

    let inside = shift(&path, &ask(1500));
    let refused: Vec<&str> = inside
        .report()
        .findings
        .iter()
        .map(|f| f.code.as_str())
        .collect();
    assert!(refused.contains(&"E-SHIFT-STRADDLE"), "{refused:?}");
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        before,
        "nothing written"
    );

    let before_it = shift(&path, &ask(500));
    assert_eq!(
        before_it.report().exit_code(),
        montagent_core::report::ExitCode::Ok,
        "{:?}",
        before_it.report().findings
    );
    let moved = common::document(&path)["tracks"][0]["elements"][0].clone();
    assert_eq!(
        (moved["start"].clone(), moved["end"].clone()),
        (json!(1100), json!(2100))
    );
    assert_eq!(
        moved["source_time"],
        json!([{"t": 1100, "v": 500}, {"t": 2060, "v": 1460, "ease": "linear"}]),
        "the keys move with the element and their source times stay"
    );
}

// ---------------------------------------------------------------------------
// Audio.
// ---------------------------------------------------------------------------

#[test]
fn a_remapped_video_contributes_nothing_to_the_mix() {
    if !has_ffprobe() {
        return;
    }
    let path = project(
        line!(),
        vec![
            video(
                "plain",
                (0, 1000),
                json!({"source": "sound.mp4", "source_start": 0, "source_end": 1000}),
            ),
            video(
                "ramp",
                (1000, 2000),
                json!({"source": "sound.mp4",
                       "source_time": [{"t": 1000, "v": 2000}, {"t": 1960, "v": 1040, "ease": "linear"}],
                       "volume": 0}),
            ),
        ],
    );
    // The clip again, with a tone under it, so a remapped element would have sound to mix.
    let ok = Command::new(tools::resolve().expect("an ffmpeg").ffmpeg)
        .args(["-hide_banner", "-loglevel", "error", "-y"])
        .args(["-f", "lavfi", "-i", "testsrc2=s=64x48:r=25"])
        .args(["-f", "lavfi", "-i", "sine=frequency=440:sample_rate=48000"])
        .args(["-frames:v", &CLIP_FRAMES.to_string(), "-shortest"])
        .args(["-c:v", "libx264", "-pix_fmt", "yuv420p", "-c:a", "aac"])
        .arg(path.with_file_name("sound.mp4"))
        .status()
        .expect("ffmpeg runs")
        .success();
    assert!(ok);

    let answer = montagent_core::verbs::render::render(
        &path,
        &montagent_core::verbs::render::Ask::default(),
        &mut |_| {},
    )
    .to_json();
    assert_eq!(answer["render"]["mixed"], json!(["plain"]), "{answer}");
    assert_eq!(answer["render"]["not_mixed"], json!([]), "{answer}");
}

// ---------------------------------------------------------------------------
// `query --at`: the derived `source_time` and `rate`.
// ---------------------------------------------------------------------------

/// What `query --at instant` says about `id`.
#[track_caller]
fn queried(path: &Path, instant: i64, id: &str) -> Value {
    use montagent_core::verbs::query::{Ask, query};
    let answer = query(
        path,
        &Ask {
            at: Some(instant),
            ..Ask::default()
        },
    );
    let view = answer.to_json()["query"].clone();
    view["stack"]
        .as_array()
        .unwrap_or_else(|| panic!("no stack: {}", answer.to_json()))
        .iter()
        .find(|present| present["id"] == id)
        .cloned()
        .unwrap_or_else(|| panic!("`{id}` is not present at {instant}"))
}

#[test]
fn query_at_derives_the_source_time_and_the_rate_on_a_ramp_a_reverse_and_a_freeze() {
    if !has_ffprobe() {
        return;
    }
    // ADR-0157's own example, shifted to 0: 1× for a second, 0.3× for two, then 1× again.
    let ramp = video(
        "ramp",
        (0, 4000),
        json!({"source_time": [{"t": 0, "v": 0}, {"t": 1000, "v": 1000, "ease": "linear"},
                               {"t": 3000, "v": 1600, "ease": "linear"},
                               {"t": 3960, "v": 2560, "ease": "linear"}],
               "volume": 0}),
    );
    let reverse = video(
        "reverse",
        (0, 2000),
        json!({"source_time": [{"t": 0, "v": 2000}, {"t": 1960, "v": 1040, "ease": "linear"}],
               "volume": 0}),
    );
    let freeze = video(
        "freeze",
        (0, 2000),
        json!({"source_time": 1200, "volume": 0}),
    );
    let path = project(line!(), vec![ramp, reverse, freeze]);

    // At 2000 the ramp resolves 1000 + 600 × 1000/2000 = 1300; at the next frame instant,
    // 2040, it resolves 1312: 12 ms of source over 40 ms of timeline.
    let ramp = queried(&path, 2000, "ramp");
    assert_eq!(
        ramp["derived"],
        json!({"source_time": 1300, "rate": "0.300×"}),
        "{ramp}"
    );
    let ramp = queried(&path, 400, "ramp");
    assert_eq!(
        ramp["derived"],
        json!({"source_time": 400, "rate": "1.000×"}),
        "{ramp}"
    );

    // Falling 960 ms over 1960 ms: at 400, 2000 − 960 × 400/1960 = 1804.08, so 1804; at 440,
    // 1784.49, so 1784. Twenty milliseconds back over forty.
    let reverse = queried(&path, 400, "reverse");
    assert_eq!(
        reverse["derived"],
        json!({"source_time": 1804, "rate": "-0.500×"}),
        "{reverse}"
    );

    let freeze = queried(&path, 400, "freeze");
    assert_eq!(
        freeze["derived"],
        json!({"source_time": 1200, "rate": "0.000×"}),
        "{freeze}"
    );
}

#[test]
fn query_at_derives_nothing_on_a_video_that_plays_a_source_range() {
    if !has_ffprobe() {
        return;
    }
    let plain = video(
        "plain",
        (0, 1000),
        json!({"source_start": 0, "source_end": 1000, "volume": 0}),
    );
    let path = project(line!(), vec![plain]);
    let plain = queried(&path, 400, "plain");
    assert_eq!(plain.get("derived"), None, "{plain}");
    assert_eq!(plain["source_offset"], 400);
}
