//! The master stage (ADR-0172, ADR-0174; #801, #824): a top-level `master` holding a loudness
//! target and a true-peak ceiling, reached by one measured gain and a 4×-rate limiter.
//!
//! Asked at the seams an agent meets: the model's parse and the published schema (what is a
//! schema error), `validate`'s document-level reviews, the mix graph `render` writes, and the
//! numbers `render` and `verify` report.

use montagent_core::model::Project;
use montagent_core::report::{ExitCode, Report};
use serde_json::{Value, json};

mod common;
use common::{canonical, tempdir, write_project};

fn header(master: Value) -> Value {
    json!({
        "frame": {"width": 64, "height": 64},
        "fps": 25,
        "duration": 1000,
        "output": "out.mp4",
        "master": master,
        "tracks": [],
    })
}

fn parse(master: Value) -> Result<Project, String> {
    serde_json::from_value::<Project>(header(master)).map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------------------
// M1: the schema.
// ---------------------------------------------------------------------------------------

#[test]
fn master_parses_with_either_key_both_or_neither() {
    for master in [
        json!({"target_lufs": -16.0, "ceiling_dbtp": -1.0}),
        json!({"target_lufs": -16.5}),
        json!({"ceiling_dbtp": -1.0}),
        json!({}),
    ] {
        let project = parse(master.clone()).unwrap_or_else(|e| panic!("{master}: {e}"));
        let written = serde_json::to_value(&project).expect("writes back");
        assert_eq!(written["master"], master, "round trip");
    }
    assert!(parse(json!({})).unwrap().master.unwrap().is_inert());
}

#[test]
fn the_ranges_ends_parse_and_one_step_past_them_is_a_schema_error() {
    for (key, value) in [
        ("target_lufs", -40.0),
        ("target_lufs", -5.0),
        ("ceiling_dbtp", -12.0),
        ("ceiling_dbtp", 0.0),
    ] {
        parse(json!({ key: value })).unwrap_or_else(|e| panic!("{key}: {value}: {e}"));
    }
    for (key, value) in [
        ("target_lufs", json!(-40.1)),
        ("target_lufs", json!(-4.9)),
        ("target_lufs", json!(14)),
        ("ceiling_dbtp", json!(-12.1)),
        ("ceiling_dbtp", json!(0.1)),
        ("ceiling_dbtp", json!(0.891)),
    ] {
        let error = parse(json!({ key: value.clone() })).expect_err(&format!("{key}: {value}"));
        assert!(error.contains(key), "{key}: {value}: {error}");
    }
}

#[test]
fn master_is_closed_and_not_animatable() {
    parse(json!({"gain_db": 3})).expect_err("a closed set: no agent-written gain");
    parse(json!({"audio_effects": []})).expect_err("no effect chain on the master");
    parse(json!({"target_lufs": [{"t": 0, "v": -16}]})).expect_err("not animatable");
    parse(json!({"ceiling_dbtp": "-1"})).expect_err("a number");
}

#[test]
fn master_sits_after_output_in_canonical_key_order() {
    let order = montagent_core::layout::canonical_order(montagent_core::layout::Published::Project)
        .expect("the project's order");
    let at = |key: &str| order.iter().position(|k| k == key).expect(key);
    assert_eq!(at("master"), at("output") + 1, "{order:?}");
}

#[test]
fn the_published_schema_states_the_ranges() {
    let schema = montagent_core::schema::generate();
    let master = &schema["$defs"]["Master"];
    assert_eq!(master["additionalProperties"], json!(false), "{master}");
    let def = |name: &str| &schema["$defs"][name];
    assert_eq!(def("TargetLufs")["minimum"], json!(-40.0));
    assert_eq!(def("TargetLufs")["maximum"], json!(-5.0));
    assert_eq!(def("CeilingDbtp")["minimum"], json!(-12.0));
    assert_eq!(def("CeilingDbtp")["maximum"], json!(0.0));
}

// ---------------------------------------------------------------------------------------
// M3: `validate`.
// ---------------------------------------------------------------------------------------

#[track_caller]
fn validated(master: Value) -> Report {
    let dir = tempdir(std::panic::Location::caller().line());
    let path = write_project(
        &dir,
        "p.montagent.json",
        &canonical(&header(master).to_string()),
    );
    montagent_core::validate(&path)
}

fn codes(report: &Report) -> Vec<&str> {
    report
        .findings
        .iter()
        .map(|f| f.code.as_str())
        .filter(|code| code.contains("MASTER") || code.starts_with("E-SCHEMA"))
        .collect()
}

fn field<'a>(report: &'a Report, code: &str, name: &str) -> &'a Value {
    let finding = report
        .findings
        .iter()
        .find(|f| f.code == code)
        .unwrap_or_else(|| panic!("no {code}"));
    finding
        .fields
        .get(name)
        .unwrap_or_else(|| panic!("{code} has no `{name}`"))
}

#[test]
fn a_usual_master_validates_clean() {
    for master in [
        json!({"target_lufs": -16, "ceiling_dbtp": -1}),
        json!({"target_lufs": -23, "ceiling_dbtp": -2}),
        json!({"ceiling_dbtp": -1}),
        json!({}),
    ] {
        let report = validated(master.clone());
        assert_eq!(codes(&report), Vec::<&str>::new(), "{master}");
        assert_eq!(report.exit_code(), ExitCode::Ok, "{master}");
    }
}

#[test]
fn an_out_of_range_value_is_a_validate_error() {
    for master in [json!({"target_lufs": -41}), json!({"ceiling_dbtp": 1})] {
        let report = validated(master.clone());
        assert_ne!(report.exit_code(), ExitCode::Ok, "{master}");
        assert!(
            codes(&report).iter().any(|c| c.starts_with("E-SCHEMA")),
            "{master}"
        );
    }
}

#[test]
fn a_target_without_a_ceiling_is_a_review_naming_the_fix() {
    let report = validated(json!({"target_lufs": -16}));
    assert_eq!(codes(&report), ["R-MASTER-NO-CEILING"]);
    assert_eq!(report.exit_code(), ExitCode::Ok, "a review, never an error");
    let text =
        montagent_core::text::render(&report.to_json(), montagent_core::text::Options::verbose())
            .expect("the report renders");
    assert!(text.contains("ceiling_dbtp: -1"), "{text}");
}

#[test]
fn a_target_louder_than_minus_9_or_quieter_than_minus_31_is_a_review() {
    for target in [-8.5, -6.0, -32.0, -40.0] {
        let report = validated(json!({"target_lufs": target, "ceiling_dbtp": 0}));
        assert!(
            codes(&report).contains(&"R-MASTER-TARGET-UNUSUAL"),
            "{target}: {:?}",
            codes(&report)
        );
        assert_eq!(
            field(&report, "R-MASTER-TARGET-UNUSUAL", "target_lufs"),
            &json!(target)
        );
    }
    for target in [-9.0, -14.0, -31.0] {
        let report = validated(json!({"target_lufs": target, "ceiling_dbtp": -1}));
        assert!(
            !codes(&report).contains(&"R-MASTER-TARGET-UNUSUAL"),
            "{target}"
        );
    }
}

#[test]
fn a_ceiling_above_minus_1_is_a_review() {
    let report = validated(json!({"ceiling_dbtp": -0.5}));
    assert_eq!(codes(&report), ["R-MASTER-CEILING-HIGH"]);
    let report = validated(json!({"ceiling_dbtp": -1}));
    assert_eq!(codes(&report), Vec::<&str>::new());
}

#[test]
fn headroom_under_6_db_is_a_review_carrying_the_number() {
    let report = validated(json!({"target_lufs": -14, "ceiling_dbtp": -9}));
    assert_eq!(codes(&report), ["R-MASTER-HEADROOM"]);
    assert_eq!(
        field(&report, "R-MASTER-HEADROOM", "headroom_db"),
        &json!(5.0)
    );
    // Exactly 6 dB is not under 6.
    let report = validated(json!({"target_lufs": -14, "ceiling_dbtp": -8}));
    assert_eq!(codes(&report), Vec::<&str>::new());
}

#[test]
fn the_borrowed_thresholds_cite_their_source_inline() {
    for (master, code) in [
        (
            json!({"target_lufs": -6, "ceiling_dbtp": 0}),
            "R-MASTER-TARGET-UNUSUAL",
        ),
        (json!({"ceiling_dbtp": 0}), "R-MASTER-CEILING-HIGH"),
        (
            json!({"target_lufs": -14, "ceiling_dbtp": -9}),
            "R-MASTER-HEADROOM",
        ),
    ] {
        let report = validated(master);
        let finding = report.findings.iter().find(|f| f.code == code).expect(code);
        assert!(finding.citation.is_some(), "{code} carries its citation");
    }
}
