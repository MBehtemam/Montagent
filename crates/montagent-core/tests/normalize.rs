//! Per-element loudness normalisation, N1 and N2 of ADR-0178 (#844): the `normalize_loudness`
//! member's schema and its `validate` findings.
//!
//! `E-NORMALIZE-NO-AUDIO` needs the probe, so it is exercised on real files made from lavfi;
//! everything else is decided from the document.

use std::path::Path;

use montagent_core::report::Report;
use montagent_core::validate;
use serde_json::{Value, json};

mod common;
use common::{canonical, write_project};

fn element(kind: &str, source: &str, list: Value) -> Value {
    json!({"id": "bed", "type": kind, "start": 0, "end": 1000, "source": source,
           "source_start": 0, "source_end": 1000, "audio_effects": list})
}

fn audio(list: Value) -> Value {
    element("audio", "a.wav", list)
}

fn project(element: Value, master: Option<Value>) -> String {
    let mut body = json!({
        "frame": {"width": 100, "height": 100}, "fps": 25, "duration": 1000,
        "output": "out/t.mp4",
        "tracks": [{"name": "t", "layer": 0, "elements": [element]}],
    });
    if let Some(master) = master {
        body["master"] = master;
    }
    canonical(&body.to_string())
}

#[track_caller]
fn validated_with(element: Value, master: Option<Value>) -> Report {
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(&dir, "p.montagent.json", &project(element, master));
    validate(&path)
}

#[track_caller]
fn validated(list: Value) -> Report {
    validated_with(audio(list), None)
}

fn codes(report: &Report, prefix: &str) -> Vec<String> {
    report
        .findings
        .iter()
        .filter(|f| f.code.starts_with(prefix))
        .map(|f| f.code.clone())
        .collect()
}

/// The findings this member can cause. A sandbox without the audio file or ffmpeg 7.1 adds
/// `E-SOURCE-MISSING` and `E-TOOL-UNSUPPORTED`, which are not what is under test.
fn own(report: &Report) -> Vec<String> {
    let mut out = codes(report, "E-SCHEMA");
    out.extend(codes(report, "E-NORMALIZE"));
    out.extend(codes(report, "R-NORMALIZE"));
    out.extend(codes(report, "E-AUDIO-EFFECT"));
    out
}

fn normalize(target: f64) -> Value {
    json!({"name": "normalize_loudness", "target_lufs": target})
}

// ---- N1: the schema. --------------------------------------------------------------------

#[test]
fn the_member_is_accepted_with_its_target_written() {
    let report = validated(json!([normalize(-23.0)]));
    assert!(own(&report).is_empty(), "{:?}", report.findings);
}

#[test]
fn the_target_is_required_and_nothing_defaults() {
    let report = validated(json!([{"name": "normalize_loudness"}]));
    assert_eq!(codes(&report, "E-SCHEMA").len(), 1, "{:?}", report.findings);
    let reason = format!("{:?}", report.findings);
    assert!(reason.contains("target_lufs"), "{reason}");
}

#[test]
fn an_unknown_key_is_a_schema_error() {
    for key in ["window", "gate", "mode"] {
        let mut member = normalize(-23.0);
        member[key] = json!(1);
        let report = validated(json!([member]));
        assert_eq!(
            codes(&report, "E-SCHEMA").len(),
            1,
            "{key}: {:?}",
            report.findings
        );
        assert!(format!("{:?}", report.findings).contains(key), "{key}");
    }
}

#[test]
fn a_keyframe_list_on_the_target_is_a_schema_error() {
    let mut member = normalize(-23.0);
    member["target_lufs"] = json!([{"t": 0, "v": -23}, {"t": 500, "v": -30}]);
    assert_eq!(codes(&validated(json!([member])), "E-SCHEMA").len(), 1);
}

#[test]
fn a_second_enabled_copy_is_adr_0169s_singular_error() {
    let report = validated(json!([normalize(-23.0), normalize(-30.0)]));
    assert_eq!(
        codes(&report, "E-AUDIO-EFFECT-SINGULAR"),
        ["E-AUDIO-EFFECT-SINGULAR"],
        "{:?}",
        report.findings
    );
    // A disabled copy does not count.
    let mut off = normalize(-30.0);
    off["enabled"] = json!(false);
    let report = validated(json!([off, normalize(-23.0)]));
    assert!(codes(&report, "E-AUDIO-EFFECT-SINGULAR").is_empty());
    assert_eq!(codes(&report, "R-AUDIO-EFFECT-DISABLED").len(), 1);
}

#[test]
fn fmt_writes_name_then_the_target_then_enabled() {
    let dir = common::tempdir(line!());
    let body = project(
        audio(json!([{"enabled": true, "target_lufs": -23, "name": "normalize_loudness"}])),
        None,
    );
    let path = write_project(&dir, "p.montagent.json", &body);
    montagent_core::verbs::fmt::fmt(&path, montagent_core::verbs::fmt::Mode::Write);
    let written = std::fs::read_to_string(&path).unwrap();
    let name = written.find("\"normalize_loudness\"").unwrap();
    let target = written.find("\"target_lufs\"").unwrap();
    let enabled = written.find("\"enabled\"").unwrap();
    assert!(name < target && target < enabled, "{written}");
}

// ---- The range (E-NORMALIZE-TARGET-RANGE). -----------------------------------------------

#[test]
fn a_target_outside_minus_40_to_minus_6_is_an_error_naming_the_nearest_bound() {
    for (target, nearest) in [(-41.0, -40.0), (-5.0, -6.0), (-70.0, -40.0), (0.0, -6.0)] {
        let report = validated(json!([normalize(target)]));
        let range: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.code == "E-NORMALIZE-TARGET-RANGE")
            .collect();
        assert_eq!(range.len(), 1, "{target}: {:?}", report.findings);
        assert_eq!(range[0].fields["nearest"], json!(nearest));
        assert_eq!(range[0].fields["index"], json!(0));
        assert!(range[0].repair.is_some(), "an error carries its repair");
    }
}

#[test]
fn the_bounds_themselves_are_in_range() {
    for target in [-40.0, -6.0, -23.0, -38.0, -14.0] {
        let report = validated(json!([normalize(target)]));
        assert!(
            codes(&report, "E-NORMALIZE").is_empty(),
            "{target}: {:?}",
            report.findings
        );
    }
}

#[test]
fn a_disabled_member_is_still_range_checked() {
    let mut member = normalize(-3.0);
    member["enabled"] = json!(false);
    let report = validated(json!([member]));
    assert_eq!(codes(&report, "E-NORMALIZE"), ["E-NORMALIZE-TARGET-RANGE"]);
}

// ---- R-NORMALIZE-ABOVE-MASTER. -----------------------------------------------------------

#[test]
fn a_target_above_the_masters_is_a_review_and_at_or_below_it_is_not() {
    let master = Some(json!({"target_lufs": -16, "ceiling_dbtp": -1}));
    let report = validated_with(audio(json!([normalize(-14.0)])), master.clone());
    assert_eq!(
        codes(&report, "R-NORMALIZE"),
        ["R-NORMALIZE-ABOVE-MASTER"],
        "{:?}",
        report.findings
    );
    let finding = report
        .findings
        .iter()
        .find(|f| f.code == "R-NORMALIZE-ABOVE-MASTER")
        .unwrap();
    assert_eq!(finding.fields["target_lufs"], json!(-14.0));
    assert_eq!(finding.fields["master_lufs"], json!(-16.0));
    assert!(finding.repair.is_none(), "a review carries no repair");
    for target in [-16.0, -23.0] {
        let report = validated_with(audio(json!([normalize(target)])), master.clone());
        assert!(codes(&report, "R-NORMALIZE").is_empty(), "{target}");
    }
}

#[test]
fn no_master_target_means_no_review() {
    let report = validated_with(audio(json!([normalize(-6.0)])), None);
    assert!(codes(&report, "R-NORMALIZE").is_empty());
    let report = validated_with(
        audio(json!([normalize(-6.0)])),
        Some(json!({"ceiling_dbtp": -1})),
    );
    assert!(codes(&report, "R-NORMALIZE").is_empty());
}

// ---- E-NORMALIZE-NO-AUDIO. ---------------------------------------------------------------

fn ffmpeg_makes(dir: &Path, name: &str, args: &[&str]) -> String {
    let path = dir.join(name);
    let made = std::process::Command::new(
        montagent_core::media::tools::resolve()
            .expect("an ffmpeg")
            .ffmpeg,
    )
    .args(["-hide_banner", "-loglevel", "error", "-y"])
    .args(args)
    .arg(&path)
    .status()
    .expect("ffmpeg runs")
    .success();
    assert!(made, "{name}");
    name.to_string()
}

#[test]
fn a_source_with_no_audio_stream_is_a_refuse_class_error_and_one_with_sound_is_not() {
    if !common::has_ffprobe() {
        return;
    }
    let dir = common::tempdir(line!());
    let mute = ffmpeg_makes(
        &dir,
        "mute.mp4",
        &[
            "-f",
            "lavfi",
            "-i",
            "color=c=blue:s=64x64:r=25:d=2",
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
        ],
    );
    let sounding = ffmpeg_makes(
        &dir,
        "sound.mp4",
        &[
            "-f",
            "lavfi",
            "-i",
            "color=c=red:s=64x64:r=25:d=2",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:sample_rate=48000:duration=2",
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
            "-c:a",
            "aac",
        ],
    );
    let path = write_project(
        &dir,
        "mute.montagent.json",
        &project(element("video", &mute, json!([normalize(-23.0)])), None),
    );
    let report = validate(&path);
    assert_eq!(
        codes(&report, "E-NORMALIZE"),
        ["E-NORMALIZE-NO-AUDIO"],
        "{:?}",
        report.findings
    );
    let finding = report
        .findings
        .iter()
        .find(|f| f.code == "E-NORMALIZE-NO-AUDIO")
        .unwrap();
    assert_eq!(finding.fields["index"], json!(0));
    assert!(
        finding.fields["source"]
            .as_str()
            .unwrap()
            .ends_with("mute.mp4"),
        "{:?}",
        finding.fields
    );
    let repair = serde_json::to_value(&finding.repair).unwrap();
    assert_eq!(repair, json!("none"), "refuse-class: the fix forks");
    // The census: which normalised elements carry sound and which do not.
    let census = finding.census.as_ref().expect("a census");
    assert_eq!(census.field, "audio_stream");
    assert_eq!(census.groups.len(), 1, "{census:?}");
    assert_eq!(census.groups[0].value, json!(false));
    assert_eq!(census.groups[0].members, ["bed"]);

    let path = write_project(
        &dir,
        "sound.montagent.json",
        &project(element("video", &sounding, json!([normalize(-23.0)])), None),
    );
    let report = validate(&path);
    assert!(
        codes(&report, "E-NORMALIZE").is_empty(),
        "{:?}",
        report.findings
    );

    // Without the member, a mute video is an ordinary video.
    let path = write_project(
        &dir,
        "plain.montagent.json",
        &project(element("video", &mute, json!([])), None),
    );
    assert!(codes(&validate(&path), "E-NORMALIZE").is_empty());
}
