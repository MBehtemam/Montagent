//! The compressor and the limiter, C1 and C3 of ADR-0180 (#846): the two `audio_effects`
//! members' schema and the six `validate` findings.
//!
//! `E-LIMITER-ABOVE-MASTER` reads the top-level `master.ceiling_dbtp`, which the master stage
//! (ADR-0172) adds to the schema; until it does, a document carrying a `master` is refused by
//! the schema, so that finding is exercised on the permissive tree in `checks::dynamics`.

use montagent_core::report::Report;
use montagent_core::validate;
use serde_json::{Value, json};

mod common;
use common::{canonical, write_project};

fn audio(list: Value) -> Value {
    json!({"id": "bed", "type": "audio", "start": 0, "end": 1000, "source": "a.wav",
           "source_start": 0, "source_end": 1000, "audio_effects": list})
}

fn project(element: Value) -> String {
    canonical(
        &json!({
            "frame": {"width": 100, "height": 100}, "fps": 25, "duration": 1000,
            "output": "out/t.mp4",
            "tracks": [{"name": "t", "layer": 0, "elements": [element]}],
        })
        .to_string(),
    )
}

#[track_caller]
fn validated(list: Value) -> Report {
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(&dir, "p.montagent.json", &project(audio(list)));
    validate(&path)
}

fn codes(report: &Report, prefix: &str) -> Vec<String> {
    report
        .findings
        .iter()
        .filter(|f| f.code.starts_with(prefix))
        .map(|f| f.code.clone())
        .collect()
}

/// The errors these members can cause. A sandbox without the audio file or ffmpeg 7.1 adds
/// `E-SOURCE-MISSING` and `E-TOOL-UNSUPPORTED`, which are not what is under test.
fn own(report: &Report) -> Vec<String> {
    let mut out = codes(report, "E-SCHEMA");
    out.extend(codes(report, "E-DYNAMICS"));
    out.extend(codes(report, "E-LIMITER"));
    out.extend(codes(report, "E-AUDIO-EFFECT"));
    out
}

fn compressor() -> Value {
    json!({"name": "compressor", "threshold_db": -24, "ratio": 3, "attack_ms": 20,
           "release_ms": 250, "makeup_db": 4})
}

fn limiter() -> Value {
    json!({"name": "limiter", "ceiling_db": -3, "release_ms": 50})
}

#[test]
fn both_members_are_accepted_with_every_key_written() {
    let report = validated(json!([compressor(), limiter()]));
    assert!(own(&report).is_empty(), "{:?}", report.findings);
}

#[test]
fn every_key_is_required_and_nothing_defaults() {
    for (member, keys) in [
        (
            compressor(),
            [
                "threshold_db",
                "ratio",
                "attack_ms",
                "release_ms",
                "makeup_db",
            ]
            .as_slice(),
        ),
        (limiter(), ["ceiling_db", "release_ms"].as_slice()),
    ] {
        for key in keys {
            let mut cut = member.clone();
            cut.as_object_mut().unwrap().remove(*key);
            let report = validated(json!([cut]));
            assert_eq!(
                codes(&report, "E-SCHEMA").len(),
                1,
                "{key}: {:?}",
                report.findings
            );
        }
    }
}

#[test]
fn an_unknown_key_is_a_schema_error() {
    let mut member = limiter();
    member["knee_db"] = json!(2);
    assert_eq!(codes(&validated(json!([member])), "E-SCHEMA").len(), 1);
}

#[test]
fn a_keyframe_list_on_any_parameter_is_a_schema_error() {
    let mut member = compressor();
    member["ratio"] = json!([{"t": 0, "v": 2}, {"t": 500, "v": 4}]);
    assert_eq!(codes(&validated(json!([member])), "E-SCHEMA").len(), 1);
    let mut member = limiter();
    member["ceiling_db"] = json!([{"t": 0, "v": -3}]);
    assert_eq!(codes(&validated(json!([member])), "E-SCHEMA").len(), 1);
}

#[test]
fn neither_member_is_singular() {
    let report = validated(json!([compressor(), compressor(), limiter(), limiter()]));
    assert!(codes(&report, "E-AUDIO-EFFECT-SINGULAR").is_empty());
}

#[test]
fn a_disabled_member_bypasses_and_is_reviewed() {
    let mut member = limiter();
    member["enabled"] = json!(false);
    let report = validated(json!([member]));
    assert!(own(&report).is_empty(), "{:?}", report.findings);
    assert_eq!(codes(&report, "R-AUDIO-EFFECT-DISABLED").len(), 1);
}

#[test]
fn a_value_outside_its_range_is_one_error_per_key_naming_the_nearest_bound() {
    let mut member = compressor();
    member["threshold_db"] = json!(-70);
    member["ratio"] = json!(25);
    member["attack_ms"] = json!(0.05);
    member["release_ms"] = json!(10000);
    member["makeup_db"] = json!(-1);
    let report = validated(json!([member]));
    let range: Vec<_> = report
        .findings
        .iter()
        .filter(|f| f.code == "E-DYNAMICS-RANGE")
        .collect();
    assert_eq!(range.len(), 5, "{:?}", report.findings);
    let ratio = range.iter().find(|f| f.fields["key"] == "ratio").unwrap();
    assert_eq!(ratio.fields["nearest"], 20.0);
    assert_eq!(ratio.fields["member"], "compressor");
    assert_eq!(ratio.fields["index"], 0);
}

#[test]
fn the_limiter_floor_is_alimiters_own_minus_24() {
    for (ceiling, fires) in [(-24.0, false), (0.0, false), (-30.0, true), (0.5, true)] {
        let report =
            validated(json!([{"name": "limiter", "ceiling_db": ceiling, "release_ms": 50}]));
        assert_eq!(
            codes(&report, "E-DYNAMICS-RANGE").len(),
            usize::from(fires),
            "{ceiling}: {:?}",
            report.findings
        );
    }
    let report = validated(json!([{"name": "limiter", "ceiling_db": -3, "release_ms": 1001}]));
    assert_eq!(codes(&report, "E-DYNAMICS-RANGE").len(), 1);
}

#[test]
fn the_bounds_themselves_are_in_range() {
    let report = validated(json!([
        {"name": "compressor", "threshold_db": -60, "ratio": 20, "attack_ms": 0.1,
         "release_ms": 9000, "makeup_db": 24},
        {"name": "compressor", "threshold_db": 0, "ratio": 1, "attack_ms": 2000,
         "release_ms": 1, "makeup_db": 0},
        {"name": "limiter", "ceiling_db": -24, "release_ms": 1},
        {"name": "limiter", "ceiling_db": 0, "release_ms": 1000},
    ]));
    assert!(
        codes(&report, "E-DYNAMICS-RANGE").is_empty(),
        "{:?}",
        report.findings
    );
}

#[test]
fn a_compressor_after_a_limiter_is_a_review_and_the_other_order_is_not() {
    let report = validated(json!([limiter(), compressor()]));
    assert_eq!(codes(&report, "R-DYNAMICS-ORDER"), ["R-DYNAMICS-ORDER"]);
    assert!(
        codes(
            &validated(json!([compressor(), limiter()])),
            "R-DYNAMICS-ORDER"
        )
        .is_empty()
    );
    // A bypassed limiter does not count.
    let mut off = limiter();
    off["enabled"] = json!(false);
    assert!(codes(&validated(json!([off, compressor()])), "R-DYNAMICS-ORDER").is_empty());
}

#[test]
fn makeup_that_can_clip_is_a_review_unless_a_limiter_follows() {
    // threshold -24, ratio 3: headroom = (-3 + 24) * (1 - 1/3) = 14 dB.
    let mut loud = compressor();
    loud["makeup_db"] = json!(15);
    assert_eq!(
        codes(
            &validated(json!([loud.clone()])),
            "R-COMPRESSOR-MAKEUP-CLIP"
        ),
        ["R-COMPRESSOR-MAKEUP-CLIP"]
    );
    assert!(
        codes(
            &validated(json!([loud.clone(), limiter()])),
            "R-COMPRESSOR-MAKEUP-CLIP"
        )
        .is_empty()
    );
    // A limiter before it does not hold it.
    assert_eq!(
        codes(
            &validated(json!([limiter(), loud])),
            "R-COMPRESSOR-MAKEUP-CLIP"
        )
        .len(),
        1
    );
    let mut within = compressor();
    within["makeup_db"] = json!(14);
    assert!(codes(&validated(json!([within])), "R-COMPRESSOR-MAKEUP-CLIP").is_empty());
}

#[test]
fn a_ratio_of_one_is_a_note() {
    let mut member = compressor();
    member["ratio"] = json!(1);
    member["makeup_db"] = json!(0);
    assert_eq!(
        codes(&validated(json!([member])), "N-COMPRESSOR-RATIO-1"),
        ["N-COMPRESSOR-RATIO-1"]
    );
}

#[test]
fn more_than_one_enabled_limiter_is_one_note_per_element() {
    let report = validated(json!([limiter(), limiter(), limiter()]));
    assert_eq!(codes(&report, "N-LIMITER-STACKED"), ["N-LIMITER-STACKED"]);
    let mut off = limiter();
    off["enabled"] = json!(false);
    assert!(codes(&validated(json!([limiter(), off])), "N-LIMITER-STACKED").is_empty());
}

#[test]
fn the_schema_publishes_both_branches() {
    let schema = montagent_core::schema::generate();
    let names: Vec<&str> = schema["$defs"]["AudioEffect"]["oneOf"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|b| b["properties"]["name"]["const"].as_str())
        .collect();
    assert!(
        names.contains(&"compressor") && names.contains(&"limiter"),
        "{names:?}"
    );
}
