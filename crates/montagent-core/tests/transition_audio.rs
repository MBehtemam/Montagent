//! A transition carries the audio across its window (ADR-0176): the `audio` field and the
//! `audio_crossfade` kind, in the schema (S1).

use montagent_core::report::Report;
use montagent_core::validate;
use montagent_core::verbs::fmt::{self, Mode};
use serde_json::{Value, json};

mod common;
use common::{canonical, write_project};

/// `a` over 0..2000 and `b` over 1000..3000 on two rects, bridged by `transition` over
/// the 1000..2000 window they share.
fn bridged(transition: Value) -> String {
    canonical(
        &json!({
            "frame": {"width": 400, "height": 200}, "fps": 25, "duration": 3000,
            "output": "out/t.mp4",
            "tracks": [
                {"name": "first", "layer": 0, "elements": [
                    {"id": "a", "type": "rect", "start": 0, "end": 2000, "x": 0, "y": 0,
                     "origin": "top-left", "width": 400, "height": 200, "fill": "#FF0000"}]},
                {"name": "second", "layer": 1, "elements": [
                    {"id": "b", "type": "rect", "start": 1000, "end": 3000, "x": 0, "y": 0,
                     "origin": "top-left", "width": 400, "height": 200, "fill": "#0000FF"}]},
                {"name": "bridge", "layer": 2, "elements": [transition]},
            ],
        })
        .to_string(),
    )
}

fn transition(kind: &str, extra: Value) -> Value {
    let mut element = json!({"id": "t", "type": "transition", "start": 1000, "end": 2000,
                             "kind": kind, "from": "a", "to": "b"});
    for (key, value) in extra.as_object().expect("an object").clone() {
        element[key] = value;
    }
    element
}

#[track_caller]
fn validated(body: &str) -> Report {
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(&dir, "p.montagent.json", body);
    validate(&path)
}

fn schema_faults(report: &Report) -> Vec<String> {
    report
        .findings
        .iter()
        .filter(|f| f.code.starts_with("E-SCHEMA"))
        .map(|f| serde_json::to_string(&f.fields).unwrap_or_default())
        .collect()
}

#[test]
fn every_new_spelling_parses_and_fmt_leaves_it_alone() {
    let mut elements = vec![transition(
        "audio_crossfade",
        json!({"audio": "constant_power"}),
    )];
    elements.push(transition(
        "audio_crossfade",
        json!({"audio": "constant_gain"}),
    ));
    for kind in ["crossfade", "wipe", "slide", "push"] {
        for audio in ["cut", "constant_power", "constant_gain"] {
            let mut extra = json!({"audio": audio});
            if kind != "crossfade" {
                extra["direction"] = json!("left");
            }
            elements.push(transition(kind, extra));
        }
    }
    for element in elements {
        let body = bridged(element.clone());
        let report = validated(&body);
        assert!(
            schema_faults(&report).is_empty(),
            "{element}: {:?}",
            report.findings
        );
        let dir = common::tempdir(line!());
        let path = write_project(&dir, "p.montagent.json", &body);
        let report = fmt::fmt(&path, Mode::Check);
        assert!(
            report.findings.is_empty(),
            "{element}: {:?}",
            report.findings
        );
    }
}

#[test]
fn the_refused_spellings_each_name_the_kinds_rule() {
    for (element, names) in [
        (transition("audio_crossfade", json!({})), "audio"),
        (
            transition("audio_crossfade", json!({"audio": "cut"})),
            "cut",
        ),
        (
            transition(
                "audio_crossfade",
                json!({"audio": "constant_power", "direction": "left"}),
            ),
            "direction",
        ),
        (
            transition(
                "audio_crossfade",
                json!({"audio": "constant_power", "ease": "ease-in"}),
            ),
            "ease",
        ),
        (
            transition("crossfade", json!({"audio": "exponential"})),
            "exponential",
        ),
    ] {
        let faults = schema_faults(&validated(&bridged(element.clone())));
        assert_eq!(faults.len(), 1, "{element}: {faults:?}");
        assert!(faults[0].contains(names), "{element}: {faults:?}");
    }
}

#[test]
fn the_published_schema_says_the_audio_rules() {
    let schema = montagent_core::schema::generate();
    let published = schema["$defs"]["Element"]["oneOf"]
        .as_array()
        .expect("the element union")
        .iter()
        .find(|branch| branch["properties"]["type"]["const"] == "transition")
        .expect("a transition branch")
        .to_string();
    for needle in [
        "audio_crossfade",
        "constant_power",
        "constant_gain",
        "ADR-0176",
    ] {
        assert!(
            published.contains(needle),
            "{needle} missing from {published}"
        );
    }
}

// ---------------------------------------------------------------------------
// The render: the transition gain stage (S2)
// ---------------------------------------------------------------------------

use montagent_core::report::ExitCode;
use montagent_core::verbs::render::{Ask, render};
use std::path::{Path, PathBuf};

/// A 3 s mono sine of constant amplitude, as a WAV the project can name.
fn tone(dir: &Path, name: &str, frequency: u32) -> String {
    let path = dir.join(name);
    let made = std::process::Command::new(
        montagent_core::media::tools::resolve()
            .expect("an ffmpeg")
            .ffmpeg,
    )
    .args([
        "-hide_banner",
        "-loglevel",
        "error",
        "-y",
        "-f",
        "lavfi",
        "-i",
    ])
    .arg(format!(
        "sine=frequency={frequency}:sample_rate=48000:duration=3"
    ))
    .arg(&path)
    .status()
    .expect("ffmpeg runs")
    .success();
    assert!(made);
    path.display().to_string().replace('\\', "/")
}

/// A 3 s picture-and-sine clip, for the picture kinds' two sides.
fn clip(dir: &Path, name: &str, frequency: u32) -> String {
    let path = dir.join(name);
    let made = std::process::Command::new(
        montagent_core::media::tools::resolve()
            .expect("an ffmpeg")
            .ffmpeg,
    )
    .args([
        "-hide_banner",
        "-loglevel",
        "error",
        "-y",
        "-f",
        "lavfi",
        "-i",
    ])
    .arg("color=c=red:s=64x64:r=25:d=3")
    .args(["-f", "lavfi", "-i"])
    .arg(format!(
        "sine=frequency={frequency}:sample_rate=48000:duration=3"
    ))
    .args([
        "-c:v", "libx264", "-pix_fmt", "yuv420p", "-c:a", "aac", "-b:a", "192k",
    ])
    .arg(&path)
    .status()
    .expect("ffmpeg runs")
    .success();
    assert!(made);
    path.display().to_string().replace('\\', "/")
}

/// Two audio elements, `a` over 0..2000 and `b` over 1000..3000, bridged by `transition`.
fn heard(dir: &Path, name: &str, transition: Option<Value>, extra_a: &str) -> PathBuf {
    sounding(dir, name, transition, extra_a, false)
}

/// The same two sides as `video` elements, so that a picture kind can bridge them.
fn seen(dir: &Path, name: &str, transition: Option<Value>) -> PathBuf {
    sounding(dir, name, transition, "", true)
}

/// A 3 s picture with no audio stream at all.
fn mute(dir: &Path, name: &str) -> String {
    let path = dir.join(name);
    let made = std::process::Command::new(
        montagent_core::media::tools::resolve()
            .expect("an ffmpeg")
            .ffmpeg,
    )
    .args([
        "-hide_banner",
        "-loglevel",
        "error",
        "-y",
        "-f",
        "lavfi",
        "-i",
    ])
    .arg("color=c=blue:s=64x64:r=25:d=3")
    .args(["-c:v", "libx264", "-pix_fmt", "yuv420p"])
    .arg(&path)
    .status()
    .expect("ffmpeg runs")
    .success();
    assert!(made);
    path.display().to_string().replace('\\', "/")
}

fn sounding(
    dir: &Path,
    name: &str,
    transition: Option<Value>,
    extra_a: &str,
    pictures: bool,
) -> PathBuf {
    let (a, b, kind, frame) = match pictures {
        true => (
            clip(dir, "a.mp4", 500),
            clip(dir, "b.mp4", 750),
            "video",
            r#","x":0,"y":0,"origin":"top-left","width":200,"height":200,"fit":"literal""#,
        ),
        false => (
            tone(dir, "a.wav", 500),
            tone(dir, "b.wav", 750),
            "audio",
            "",
        ),
    };
    let mut bridge = String::new();
    if let Some(transition) = transition {
        bridge = format!(r##",{{"name":"bridge","layer":2,"elements":[{transition}]}}"##);
    }
    let body = canonical(&format!(
        r##"{{"frame":{{"width":200,"height":200}},"fps":25,"background":"#000000",
            "duration":3000,"output":"out/{name}.mp4",
            "tracks":[
              {{"name":"first","layer":0,"elements":[
                {{"id":"a","type":"{kind}","start":0,"end":2000,"source":"{a}",
                  "source_start":0,"source_end":2000{frame}{extra_a}}}]}},
              {{"name":"second","layer":1,"elements":[
                {{"id":"b","type":"{kind}","start":1000,"end":3000,"source":"{b}",
                  "source_start":0,"source_end":2000{frame}}}]}}{bridge}]}}"##
    ));
    write_project(dir, &format!("{name}.montagent.json"), &body)
}

#[track_caller]
fn rendered_pcm(path: &Path, ask: &Ask, out: &str) -> Vec<f32> {
    let answer = render(path, ask, &mut |_| {});
    assert_eq!(
        answer.report().exit_code(),
        ExitCode::Ok,
        "{}",
        serde_json::to_string_pretty(&answer.to_json()).unwrap_or_default()
    );
    common::media::samples(&path.parent().unwrap().join("out").join(out))
}

fn rms(pcm: &[f32], from_ms: usize, to_ms: usize) -> f64 {
    let slice = &pcm[from_ms * 48..to_ms * 48];
    (slice.iter().map(|s| f64::from(*s).powi(2)).sum::<f64>() / slice.len() as f64).sqrt()
}

#[test]
fn a_cut_renders_the_same_samples_as_no_audio_key_at_all() {
    if !common::has_ffprobe() {
        return;
    }
    let dir = common::tempdir(line!());
    let absent = seen(&dir, "absent", Some(transition("crossfade", json!({}))));
    let cut = seen(
        &dir,
        "cut",
        Some(transition("crossfade", json!({"audio": "cut"}))),
    );
    let none = seen(&dir, "none", None);
    let absent = rendered_pcm(&absent, &Ask::default(), "absent.mp4");
    assert_eq!(absent, rendered_pcm(&cut, &Ask::default(), "cut.mp4"));
    assert_eq!(absent, rendered_pcm(&none, &Ask::default(), "none.mp4"));
}

#[test]
fn a_constant_power_crossfade_holds_the_level_through_its_window() {
    if !common::has_ffprobe() {
        return;
    }
    let dir = common::tempdir(line!());
    let faded = heard(
        &dir,
        "faded",
        Some(transition(
            "audio_crossfade",
            json!({"audio": "constant_power"}),
        )),
        "",
    );
    let plain = heard(&dir, "plain", None, "");
    let faded = rendered_pcm(&faded, &Ask::default(), "faded.mp4");
    let plain = rendered_pcm(&plain, &Ask::default(), "plain.mp4");
    // Before the window `a` is alone and untouched; after it `b` is.
    assert!((rms(&faded, 200, 900) - rms(&plain, 200, 900)).abs() < 0.005);
    assert!((rms(&faded, 2100, 2900) - rms(&plain, 2100, 2900)).abs() < 0.005);
    // Unrelated sines at the midpoint keep the level of one side alone (0 dB, ADR-0176 §1)
    // where the hard cut sums them 3 dB high.
    let alone = rms(&plain, 200, 900);
    let mid = rms(&faded, 1480, 1520);
    assert!(
        (20.0 * (mid / alone).log10()).abs() < 0.5,
        "{mid} vs {alone}"
    );
    assert!(rms(&plain, 1480, 1520) / alone > 1.3);
}

#[test]
fn a_range_starting_inside_the_window_renders_the_samples_the_full_render_has() {
    if !common::has_ffprobe() {
        return;
    }
    let dir = common::tempdir(line!());
    let path = heard(
        &dir,
        "span",
        Some(transition(
            "audio_crossfade",
            json!({"audio": "constant_power"}),
        )),
        "",
    );
    let full = rendered_pcm(&path, &Ask::default(), "span.mp4");
    let ranged = rendered_pcm(
        &path,
        &Ask {
            from: Some(1400),
            to: Some(2400),
            output: None,
            no_clobber: false,
        },
        "span.1400-2400.mp4",
    );
    // The encoder's priming differs between the two files, so compare the level, not the
    // sample: 100 ms slices across the window agree to within 0.5 dB.
    for at in [0usize, 200, 400] {
        let whole = rms(&full, 1400 + at, 1500 + at);
        let part = rms(&ranged, at, at + 100);
        assert!(
            (20.0 * (part / whole).log10()).abs() < 0.5,
            "{at}: {part} vs {whole}"
        );
    }
}

#[test]
fn a_picture_kind_with_audio_fades_its_video_elements_embedded_sound() {
    if !common::has_ffprobe() {
        return;
    }
    let dir = common::tempdir(line!());
    let faded = seen(
        &dir,
        "faded",
        Some(transition("crossfade", json!({"audio": "constant_power"}))),
    );
    let cut = seen(&dir, "cut", Some(transition("crossfade", json!({}))));
    let faded = rendered_pcm(&faded, &Ask::default(), "faded.mp4");
    let cut = rendered_pcm(&cut, &Ask::default(), "cut.mp4");
    let alone = rms(&cut, 200, 900);
    let mid = rms(&faded, 1480, 1520);
    assert!(
        (20.0 * (mid / alone).log10()).abs() < 0.7,
        "{mid} vs {alone}"
    );
    assert!(rms(&cut, 1480, 1520) / alone > 1.3);
}

#[test]
fn an_audio_crossfade_paints_nothing_and_frame_raises_nothing_for_it() {
    if !common::has_ffprobe() {
        return;
    }
    use montagent_core::verbs::frame::{Ask, frame};
    let dir = common::tempdir(line!());
    let path = heard(
        &dir,
        "quiet",
        Some(transition(
            "audio_crossfade",
            json!({"audio": "constant_power"}),
        )),
        "",
    );
    let answer = frame(
        &path,
        &Ask {
            at: Some(1500),
            full: true,
            ..Ask::default()
        },
    )
    .to_json();
    let text = answer.to_string();
    assert!(!text.contains("E-NOT-PAINTED"), "{text}");
    assert!(!text.contains("audio_crossfade"), "{text}");
}

// ---------------------------------------------------------------------------
// `validate`: one error, three reviews (S3)
// ---------------------------------------------------------------------------

/// The ADR-0176 codes in a report, in the order found.
fn audio_codes(report: &Report) -> Vec<&str> {
    report
        .findings
        .iter()
        .map(|f| f.code.as_str())
        .filter(|code| code.contains("TRANSITION") && !code.ends_with("-RANGE"))
        .collect()
}

/// The report as the text form prints it: where a finding's prose is read.
fn prose(report: &Report) -> String {
    montagent_core::text::render(&report.to_json(), montagent_core::text::Options::verbose())
        .unwrap()
}

#[track_caller]
fn checked(path: &Path) -> Report {
    validate(path)
}

/// `a` and `b` as `video` elements, `a` carrying `extra_a`, bridged by `transition`.
fn pictures(dir: &Path, name: &str, transition: Value, extra_a: &str) -> PathBuf {
    sounding(dir, name, Some(transition), extra_a, true)
}

#[test]
fn a_dissolve_between_two_sounding_clips_gains_exactly_one_unset_review() {
    if !common::has_ffprobe() {
        return;
    }
    let dir = common::tempdir(line!());
    let path = pictures(&dir, "unset", transition("crossfade", json!({})), "");
    let report = checked(&path);
    assert_eq!(audio_codes(&report), ["R-TRANSITION-AUDIO-UNSET"]);
    let text = prose(&report);
    assert!(
        text.contains("constant_power") && text.contains("cut"),
        "the text gives both literals: {text}"
    );
    // Clearing it is one string replace, either way.
    for value in ["constant_power", "cut"] {
        let body = std::fs::read_to_string(&path).unwrap().replace(
            r#""kind": "crossfade""#,
            &format!(r#""kind": "crossfade", "audio": "{value}""#),
        );
        let body = body.replace(
            r#""kind":"crossfade""#,
            &format!(r#""kind":"crossfade","audio":"{value}""#),
        );
        std::fs::write(&path, body).unwrap();
        assert!(
            audio_codes(&checked(&path)).is_empty(),
            "{value}: {:?}",
            audio_codes(&checked(&path))
        );
        std::fs::write(
            &path,
            std::fs::read_to_string(&path)
                .unwrap()
                .replace(&format!(r#","audio":"{value}""#), "")
                .replace(&format!(r#", "audio": "{value}""#), ""),
        )
        .unwrap();
    }
}

#[test]
fn the_unset_review_stays_quiet_when_a_side_is_silent_or_the_kind_is_not_a_picture() {
    if !common::has_ffprobe() {
        return;
    }
    let dir = common::tempdir(line!());
    // A constant `volume: 0` on a side: no sound to hard-cut, so no UNSET — but the silence
    // is itself reviewed.
    let path = pictures(
        &dir,
        "muted",
        transition("crossfade", json!({})),
        r#","volume":0"#,
    );
    assert_eq!(audio_codes(&checked(&path)), ["R-TRANSITION-AUDIO-SILENT"]);
    // A side with no audio stream is not an error on a picture kind, and UNSET needs both.
    let dir = common::tempdir(line!());
    let path = sounding(
        &dir,
        "mute-b",
        Some(transition("crossfade", json!({}))),
        "",
        true,
    );
    let body = std::fs::read_to_string(&path)
        .unwrap()
        .replace("b.mp4", "m.mp4");
    mute(&dir, "m.mp4");
    std::fs::write(&path, body).unwrap();
    let report = checked(&path);
    assert!(
        audio_codes(&report).is_empty(),
        "{:?}",
        audio_codes(&report)
    );
}

#[test]
fn an_audio_crossfade_side_without_an_audio_stream_is_a_refuse_class_error() {
    if !common::has_ffprobe() {
        return;
    }
    let dir = common::tempdir(line!());
    let path = pictures(
        &dir,
        "nostream",
        transition("audio_crossfade", json!({"audio": "constant_power"})),
        "",
    );
    let body = std::fs::read_to_string(&path)
        .unwrap()
        .replace("b.mp4", "m.mp4");
    mute(&dir, "m.mp4");
    std::fs::write(&path, body).unwrap();
    let report = checked(&path);
    assert_eq!(audio_codes(&report), ["E-TRANSITION-AUDIO-NO-STREAM"]);
    let finding = &report.findings[0];
    assert_eq!(finding.fields["side"], json!("to"));
    assert_eq!(finding.fields["target"], json!("b"));
    assert!(
        finding.fields["source"]
            .as_str()
            .unwrap()
            .ends_with("m.mp4"),
        "{:?}",
        finding.fields
    );
    // With sound on both sides it is clean.
    let clean = pictures(
        &dir,
        "clean",
        transition("audio_crossfade", json!({"audio": "constant_gain"})),
        "",
    );
    assert!(audio_codes(&checked(&clean)).is_empty());
}

#[test]
fn an_audio_crossfade_names_audio_or_video_and_a_picture_kind_still_refuses_an_audio() {
    if !common::has_ffprobe() {
        return;
    }
    let dir = common::tempdir(line!());
    // An audio_crossfade over two `audio` elements validates clean.
    let path = heard(
        &dir,
        "sound",
        Some(transition(
            "audio_crossfade",
            json!({"audio": "constant_power"}),
        )),
        "",
    );
    assert!(audio_codes(&checked(&path)).is_empty());
    // Naming something that makes no sound: refused, and the text names what the kind accepts.
    let body = std::fs::read_to_string(&path)
        .unwrap()
        .replace(r#""from":"a""#, r#""from":"nobody""#);
    let body = body.replace(r#""from": "a""#, r#""from": "nobody""#);
    std::fs::write(&path, body).unwrap();
    let report = checked(&path);
    let missing: Vec<_> = report
        .findings
        .iter()
        .filter(|f| f.code == "E-TRANSITION-REF-MISSING")
        .collect();
    assert_eq!(missing.len(), 1, "{:?}", audio_codes(&report));
    assert!(
        prose(&report).contains("audio or video"),
        "{}",
        prose(&report)
    );
    // A picture kind over `audio` elements keeps refusing them, naming its own kinds.
    let path = heard(
        &dir,
        "picture",
        Some(transition("crossfade", json!({}))),
        "",
    );
    let report = checked(&path);
    let missing: Vec<_> = report
        .findings
        .iter()
        .filter(|f| f.code == "E-TRANSITION-REF-MISSING")
        .collect();
    assert_eq!(missing.len(), 2);
    assert!(
        prose(&report).contains("visual element"),
        "{}",
        prose(&report)
    );
    // One fact, one code: a reference error silences the new reviews.
    assert!(
        !audio_codes(&report).iter().any(|c| c.starts_with("R-")),
        "{:?}",
        audio_codes(&report)
    );
}

#[test]
fn a_volume_that_changes_inside_the_window_is_reviewed_and_a_flat_one_is_not() {
    if !common::has_ffprobe() {
        return;
    }
    let crossfade = || transition("audio_crossfade", json!({"audio": "constant_power"}));
    for (extra, expected) in [
        // A keyframe strictly inside (1000, 2000).
        (r#","volume":[{"t":0,"v":1.0},{"t":1500,"v":0.5}]"#, true),
        // A segment straddling the whole window with differing levels.
        (r#","volume":[{"t":500,"v":1.0},{"t":2500,"v":0.5}]"#, true),
        // Changes before the window only.
        (r#","volume":[{"t":0,"v":1.0},{"t":900,"v":0.5}]"#, false),
        // A flat level across the window, written as keyframes or as a scalar.
        (r#","volume":[{"t":500,"v":0.5},{"t":2500,"v":0.5}]"#, false),
        (r#","volume":0.5"#, false),
    ] {
        let dir = common::tempdir(line!());
        let path = heard(&dir, "stack", Some(crossfade()), extra);
        let report = checked(&path);
        let codes = audio_codes(&report);
        match expected {
            true => assert_eq!(codes, ["R-TRANSITION-VOLUME-STACK"], "{extra}"),
            false => assert!(codes.is_empty(), "{extra}: {codes:?}"),
        }
    }
    // The text names the transition, the element and the keyframe times.
    let dir = common::tempdir(line!());
    let path = heard(
        &dir,
        "named",
        Some(crossfade()),
        r#","volume":[{"t":0,"v":1.0},{"t":1500,"v":0.5}]"#,
    );
    let report = checked(&path);
    let text = prose(&report);
    assert!(
        text.contains('t') && text.contains("`a`") && text.contains("1500"),
        "{text}"
    );
}

#[test]
fn a_silent_bridged_side_is_reviewed_on_any_kind() {
    if !common::has_ffprobe() {
        return;
    }
    let dir = common::tempdir(line!());
    let audio = heard(
        &dir,
        "silent-audio",
        Some(transition(
            "audio_crossfade",
            json!({"audio": "constant_power"}),
        )),
        r#","volume":0"#,
    );
    assert_eq!(audio_codes(&checked(&audio)), ["R-TRANSITION-AUDIO-SILENT"]);
    let picture = pictures(
        &dir,
        "silent-picture",
        transition("crossfade", json!({"audio": "constant_power"})),
        r#","volume":0"#,
    );
    assert_eq!(
        audio_codes(&checked(&picture)),
        ["R-TRANSITION-AUDIO-SILENT"]
    );
    // A side at any other constant level is not silent.
    let audible = heard(
        &dir,
        "audible",
        Some(transition(
            "audio_crossfade",
            json!({"audio": "constant_power"}),
        )),
        r#","volume":0.25"#,
    );
    assert!(audio_codes(&checked(&audible)).is_empty());
}

// ---------------------------------------------------------------------------
// The reading tools (S4)
// ---------------------------------------------------------------------------

use montagent_core::Wire;
use montagent_core::verbs::query::{self, Ask as QueryAsk};
use montagent_core::wire;

/// Two audio elements with no media on disk: `query --at` reads the document alone.
fn unplayed(transition: Option<Value>) -> String {
    let bridge = match transition {
        Some(transition) => {
            format!(r##",{{"name":"bridge","layer":2,"elements":[{transition}]}}"##)
        }
        None => String::new(),
    };
    canonical(&format!(
        r##"{{"frame":{{"width":200,"height":200}},"fps":25,"duration":3000,
            "tracks":[
              {{"name":"first","layer":0,"elements":[
                {{"id":"a","type":"audio","start":0,"end":2000,"source":"a.wav",
                  "source_start":0,"source_end":2000,"volume":0.5}}]}},
              {{"name":"second","layer":1,"elements":[
                {{"id":"b","type":"audio","start":1000,"end":3000,"source":"b.wav",
                  "source_start":0,"source_end":2000}}]}}{bridge}]}}"##
    ))
}

fn queried_at(body: &str, at: i64) -> (Value, String) {
    let dir = common::tempdir(line!());
    let path = write_project(&dir, "p.montagent.json", body);
    let ask = QueryAsk {
        at: Some(at),
        ..QueryAsk::default()
    };
    let answer = query::query(&path, &ask);
    (
        answer.to_json(),
        wire::render_query(&answer, Wire::Text { verbose: false }),
    )
}

fn sound<'v>(view: &'v Value, id: &str) -> &'v Value {
    view["query"]["stack"]
        .as_array()
        .expect("a stack")
        .iter()
        .find(|row| row["id"] == id)
        .map(|row| &row["sound"])
        .unwrap_or_else(|| panic!("no `{id}` in {view}"))
}

#[test]
fn query_at_prints_each_sides_gain_with_its_factors_and_the_transitions_id() {
    // Mid-window, the two curves' gains on the `afade` definitions: `qsin` is sin(π/2·p) and
    // `tri` is p, so at p = 0.5 they are 0.71 and 0.50 on both sides.
    for (curve, gain, text) in [
        (
            "constant_power",
            std::f64::consts::FRAC_PI_4.sin() * 0.5,
            "0.35",
        ),
        ("constant_gain", 0.5 * 0.5, "0.25"),
    ] {
        let (view, prose) = queried_at(
            &unplayed(Some(transition("audio_crossfade", json!({"audio": curve})))),
            1500,
        );
        let a = sound(&view, "a");
        assert!(
            (a["gain"].as_f64().unwrap() - gain).abs() < 1e-9,
            "{curve}: {a}"
        );
        assert_eq!(a["transitions"][0]["transition"], "t");
        assert!(
            a["text"]
                .as_str()
                .unwrap()
                .starts_with(&format!("volume 0.5 × transition t {curve} "))
                && a["text"].as_str().unwrap().ends_with(&format!("→ {text}")),
            "{curve}: {a}"
        );
        // `b` has no `volume`, so the gain is the curve alone.
        let b = sound(&view, "b");
        let alone = if curve == "constant_power" {
            std::f64::consts::FRAC_1_SQRT_2
        } else {
            0.5
        };
        assert!(
            (b["gain"].as_f64().unwrap() - alone).abs() < 1e-3,
            "{curve}: {b}"
        );
        assert!(prose.contains(&format!("transition t {curve}")), "{prose}");
    }
}

#[test]
fn query_at_with_audio_absent_says_the_transition_cuts_by_absence() {
    let (view, prose) = queried_at(&unplayed(Some(transition("crossfade", json!({})))), 1500);
    assert_eq!(
        sound(&view, "a")["text"],
        "transition t audio: cut (absent)"
    );
    assert!(
        prose.contains("transition t audio: cut (absent)"),
        "{prose}"
    );
    let (view, _) = queried_at(
        &unplayed(Some(transition("crossfade", json!({"audio": "cut"})))),
        1500,
    );
    assert_eq!(sound(&view, "b")["text"], "transition t audio: cut");
    // Outside the window, or with no transition at all, there is nothing to print.
    let (view, _) = queried_at(
        &unplayed(Some(transition(
            "audio_crossfade",
            json!({"audio": "constant_power"}),
        ))),
        500,
    );
    assert!(sound(&view, "a").is_null());
    let (view, _) = queried_at(&unplayed(None), 1500);
    assert!(sound(&view, "a").is_null());
}

#[test]
fn the_printed_gain_is_the_afade_curve_at_the_instant() {
    use montagent_core::model::TransitionAudio;
    // The same numbers the render's `afade` stage is tested against: out at p is the curve
    // at 1 - p, in at p is the curve at p.
    for (p, power_in, power_out) in [
        (0.25, 0.3827, 0.9239),
        (
            0.5,
            std::f64::consts::FRAC_1_SQRT_2,
            std::f64::consts::FRAC_1_SQRT_2,
        ),
    ] {
        let instant = 1000 + (p * 1000.0) as i64;
        let (view, _) = queried_at(
            &unplayed(Some(transition(
                "audio_crossfade",
                json!({"audio": "constant_power"}),
            ))),
            instant,
        );
        let b = sound(&view, "b")["gain"].as_f64().unwrap();
        let a = sound(&view, "a")["gain"].as_f64().unwrap() / 0.5;
        assert!(
            (b - power_in).abs() < 1e-3 && (a - power_out).abs() < 1e-3,
            "{p}: {a} {b}"
        );
    }
    assert_eq!(TransitionAudio::ConstantGain.as_str(), "constant_gain");
}

#[test]
fn timeline_prints_the_audio_value_as_one_token_on_the_transition_row() {
    let dir = common::tempdir(line!());
    let path = write_project(
        &dir,
        "p.montagent.json",
        &unplayed(Some(transition(
            "audio_crossfade",
            json!({"audio": "constant_gain"}),
        ))),
    );
    let answer = montagent_core::verbs::timeline::timeline(&path);
    let text = wire::render_timeline(&answer, Wire::Text { verbose: false });
    assert!(text.contains("a → b audio=constant_gain"), "{text}");
}

#[test]
fn shift_and_compare_carry_an_audio_crossfade_unchanged() {
    if !common::has_ffprobe() {
        return;
    }
    let dir = common::tempdir(line!());
    let made = heard(
        &dir,
        "src",
        Some(transition(
            "audio_crossfade",
            json!({"audio": "constant_power"}),
        )),
        "",
    );
    let mut value: Value = serde_json::from_str(&std::fs::read_to_string(&made).unwrap()).unwrap();
    value.as_object_mut().unwrap().remove("duration");
    let body = canonical(&value.to_string());
    let reference = write_project(&dir, "ref.montagent.json", &body);
    let path = write_project(&dir, "p.montagent.json", &body);
    let shifted = montagent_core::verbs::shift::shift(
        &path,
        &montagent_core::verbs::shift::Ask {
            at: 0,
            delta: 500,
            scope: None,
            release: Vec::new(),
        },
    );
    assert_eq!(shifted.report().exit_code(), ExitCode::Ok);
    let written: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let t = &written["tracks"][2]["elements"][0];
    assert_eq!(
        (t["start"].as_i64(), t["end"].as_i64()),
        (Some(1500), Some(2500))
    );
    assert_eq!(t["kind"], "audio_crossfade");
    assert_eq!(t["audio"], "constant_power");
    let compared = montagent_core::verbs::compare::compare(&reference, &path);
    assert_eq!(compared.exit_code(), ExitCode::Ok);
}
