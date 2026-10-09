//! The four EQ members of `audio_effects` (ADR-0179; #845 slices E1 and E3): the schema
//! shape and the five findings, through the real `validate`.

use montagent_core::report::Report;
use montagent_core::validate;
use serde_json::{Value, json};

mod common;
use common::{canonical, write_project};

fn audio(list: Value) -> Value {
    json!({"id": "bed", "type": "audio", "start": 0, "end": 1000,
           "source": "a.wav", "source_start": 0, "source_end": 1000,
           "audio_effects": list})
}

#[track_caller]
fn validated(element: Value) -> Report {
    let body = canonical(
        &json!({
            "frame": {"width": 100, "height": 100}, "fps": 25, "duration": 1000,
            "output": "out/t.mp4",
            "tracks": [{"name": "t", "layer": 0, "elements": [element]}],
        })
        .to_string(),
    );
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(&dir, "p.montagent.json", &body);
    validate(&path)
}

#[track_caller]
fn codes(list: Value, prefix: &str) -> Vec<String> {
    validated(audio(list))
        .findings
        .iter()
        .filter(|f| f.code.starts_with(prefix))
        .map(|f| f.code.clone())
        .collect()
}

fn hp(f: f64, s: f64) -> Value {
    json!({"name": "highpass", "frequency_hz": f, "slope_db_per_oct": s})
}
fn lp(f: f64, s: f64) -> Value {
    json!({"name": "lowpass", "frequency_hz": f, "slope_db_per_oct": s})
}
fn shelf(side: &str, f: f64, g: f64) -> Value {
    json!({"name": "shelf", "side": side, "frequency_hz": f, "gain_db": g})
}
fn bell(f: f64, g: f64, q: f64) -> Value {
    json!({"name": "bell", "frequency_hz": f, "gain_db": g, "q": q})
}

// ---- E1: schema -------------------------------------------------------------------------

#[test]
fn the_adr_examples_validate_clean() {
    let report = validated(audio(json!([
        hp(100.0, 24.0),
        lp(8000.0, 12.0),
        shelf("high", 4000.0, 3.0),
        bell(1500.0, -6.0, 1.4)
    ])));
    // The sandbox has no `a.wav` and may have no qualified ffmpeg: only the list's own codes.
    let own: Vec<_> = report
        .findings
        .iter()
        .filter(|f| {
            ["E-SCHEMA", "E-EQ", "R-EQ", "N-EQ", "E-AUDIO", "R-AUDIO"]
                .iter()
                .any(|p| f.code.starts_with(p))
        })
        .collect();
    assert!(own.is_empty(), "{own:?}");
}

#[test]
fn every_key_is_required_and_there_is_no_default() {
    for member in [
        json!({"name": "bell"}),
        json!({"name": "bell", "frequency_hz": 100, "gain_db": 1}),
        json!({"name": "highpass", "frequency_hz": 100}),
        json!({"name": "lowpass", "slope_db_per_oct": 12}),
        json!({"name": "shelf", "frequency_hz": 100, "gain_db": 1}),
        json!({"name": "shelf", "side": "low", "frequency_hz": 100}),
    ] {
        assert_eq!(
            codes(json!([member.clone()]), "E-SCHEMA").len(),
            1,
            "{member}"
        );
    }
}

#[test]
fn an_unknown_key_a_bad_side_and_a_keyframe_list_are_schema_errors() {
    let mut extra = bell(1000.0, 1.0, 1.0);
    extra["width"] = json!(3);
    let mut keyed = bell(1000.0, 1.0, 1.0);
    keyed["gain_db"] = json!([{"t": 0, "value": 1}, {"t": 500, "value": 2}]);
    for member in [extra, keyed, shelf("middle", 100.0, 1.0)] {
        assert_eq!(
            codes(json!([member.clone()]), "E-SCHEMA").len(),
            1,
            "{member}"
        );
    }
}

#[test]
fn enabled_is_accepted_on_every_member() {
    for mut member in [
        hp(100.0, 12.0),
        lp(100.0, 12.0),
        shelf("low", 100.0, 1.0),
        bell(100.0, 1.0, 1.0),
    ] {
        member["enabled"] = json!(true);
        assert!(
            codes(json!([member.clone()]), "E-SCHEMA").is_empty(),
            "{member}"
        );
    }
}

// ---- E3: findings -----------------------------------------------------------------------

#[test]
fn range_edges_are_clean_and_one_step_past_each_is_e_eq_range() {
    let ok = [
        hp(20.0, 12.0),
        hp(20000.0, 48.0),
        lp(20.0, 24.0),
        shelf("low", 20.0, -24.0),
        shelf("high", 20000.0, 24.0),
        bell(20.0, -24.0, 0.1),
        bell(20000.0, 24.0, 10.0),
    ];
    for member in ok {
        assert!(
            codes(json!([member.clone()]), "E-EQ").is_empty(),
            "{member}"
        );
    }
    let bad = [
        hp(19.9, 12.0),
        hp(20000.1, 12.0),
        lp(100.0, 18.0),
        lp(100.0, 6.0),
        hp(100.0, 36.0),
        shelf("low", 19.0, 1.0),
        shelf("high", 100.0, 24.1),
        shelf("high", 100.0, -24.1),
        bell(100.0, 25.0, 1.0),
        bell(100.0, 1.0, 0.09),
        bell(100.0, 1.0, 10.1),
    ];
    for member in bad {
        assert_eq!(
            codes(json!([member.clone()]), "E-EQ"),
            vec!["E-EQ-RANGE"],
            "{member}"
        );
    }
}

#[test]
fn a_disabled_member_is_still_range_checked() {
    let mut member = bell(100.0, 30.0, 1.0);
    member["enabled"] = json!(false);
    assert!(codes(json!([member]), "E-EQ").contains(&"E-EQ-RANGE".to_string()));
}

#[test]
fn eight_enabled_members_are_fine_and_a_ninth_is_the_cap() {
    let eight: Vec<Value> = (0..8)
        .map(|i| bell(100.0 + i as f64 * 100.0, 1.0, 1.0))
        .collect();
    assert!(codes(json!(eight), "E-EQ").is_empty());
    let mut nine = eight.clone();
    nine.push(bell(1000.0, 1.0, 1.0));
    assert_eq!(codes(json!(nine.clone()), "E-EQ"), vec!["E-EQ-STACK-CAP"]);
    // The cap counts enabled members only.
    nine[8]["enabled"] = json!(false);
    assert!(codes(json!(nine), "E-EQ").is_empty());
}

#[test]
fn the_cap_counts_all_four_kinds_together() {
    let mut list = vec![
        hp(100.0, 12.0),
        lp(9000.0, 12.0),
        shelf("low", 100.0, 1.0),
        shelf("high", 5000.0, 1.0),
    ];
    list.extend((0..5).map(|i| bell(500.0 + i as f64 * 100.0, 1.0, 1.0)));
    assert_eq!(codes(json!(list), "E-EQ"), vec!["E-EQ-STACK-CAP"]);
}

#[test]
fn a_gain_above_twelve_is_a_review_and_twelve_is_not() {
    assert!(codes(json!([bell(100.0, 12.0, 1.0)]), "R-EQ").is_empty());
    assert!(codes(json!([shelf("low", 100.0, -12.0)]), "R-EQ").is_empty());
    for member in [
        bell(100.0, 12.5, 1.0),
        bell(100.0, -14.0, 1.0),
        shelf("high", 100.0, 13.0),
    ] {
        assert_eq!(
            codes(json!([member.clone()]), "R-EQ"),
            vec!["R-EQ-GAIN-EXTREME"],
            "{member}"
        );
    }
}

#[test]
fn a_crossed_band_pair_is_a_review_and_a_proper_band_is_not() {
    for (h, l) in [(8000.0, 100.0), (1000.0, 1000.0)] {
        assert_eq!(
            codes(json!([hp(h, 12.0), lp(l, 12.0)]), "R-EQ"),
            vec!["R-EQ-BAND-CROSSED"]
        );
    }
    assert!(codes(json!([hp(100.0, 12.0), lp(8000.0, 12.0)]), "R-EQ").is_empty());
    // A bypassed member crosses nothing.
    let mut off = lp(100.0, 12.0);
    off["enabled"] = json!(false);
    assert!(codes(json!([hp(8000.0, 12.0), off]), "R-EQ-BAND").is_empty());
    // Order does not matter.
    assert_eq!(
        codes(json!([lp(100.0, 12.0), hp(8000.0, 12.0)]), "R-EQ-BAND"),
        vec!["R-EQ-BAND-CROSSED"]
    );
}

#[test]
fn zero_gain_is_a_note_on_a_shelf_and_a_bell() {
    for member in [bell(100.0, 0.0, 1.0), shelf("low", 100.0, 0.0)] {
        assert_eq!(
            codes(json!([member.clone()]), "N-EQ"),
            vec!["N-EQ-NO-OP"],
            "{member}"
        );
    }
    assert!(codes(json!([bell(100.0, 0.5, 1.0)]), "N-EQ").is_empty());
    assert!(codes(json!([hp(100.0, 12.0)]), "N-EQ").is_empty());
}

#[test]
fn the_findings_apply_on_a_video_too() {
    let element = json!({"id": "v", "type": "video", "start": 0, "end": 1000,
        "source": "v.mp4", "source_start": 0, "source_end": 1000,
        "x": 0, "y": 0, "width": 100, "height": 100, "fit": "contain",
        "audio_effects": [bell(100.0, 30.0, 1.0)]});
    let report = validated(element);
    assert!(report.findings.iter().any(|f| f.code == "E-EQ-RANGE"));
}
