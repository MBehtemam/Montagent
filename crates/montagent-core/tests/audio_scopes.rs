//! Audio dynamics (gate), parametric EQ, and perceptual diagnostic scopes (ADR-0207).

use std::path::PathBuf;

use montagent_core::verbs::measure::Ask;
use montagent_core::verbs::validate::validate;
use montagent_core::wire::Wire;
use serde_json::json;

mod common;
use common::{canonical, tempdir, write_project};

fn basic_project(dir: &std::path::Path, audio_effects: &str) -> PathBuf {
    let body = canonical(&format!(
        r##"{{
            "frame": {{"width": 100, "height": 100}},
            "fps": 30,
            "background": "#000000",
            "duration": 1000,
            "output": "out.mp4",
            "tracks": [
                {{
                    "name": "audio",
                    "layer": 0,
                    "elements": [
                        {{
                            "id": "a1",
                            "type": "audio",
                            "start": 0,
                            "end": 1000,
                            "source": "dummy.wav",
                            "source_start": 0,
                            "source_end": 1000,
                            "audio_effects": {audio_effects}
                        }}
                    ]
                }}
            ]
        }}"##
    ));
    write_project(dir, "p.montagent.json", &body)
}

#[test]
fn gate_range_and_schema_validation() {
    let dir = tempdir(line!());

    // Valid gate
    let valid_path = basic_project(
        &dir,
        r#"[{"name": "gate", "threshold_db": -24.0, "attack_ms": 10.0, "release_ms": 100.0}]"#,
    );
    let report = validate(&valid_path);
    assert!(
        !report.findings.iter().any(|f| f.code == "E-DYNAMICS-RANGE"),
        "valid gate should have no E-DYNAMICS-RANGE findings: {:?}",
        report.findings
    );

    // Invalid threshold_db (out of -70..=0)
    let bad_thresh = basic_project(
        &dir,
        r#"[{"name": "gate", "threshold_db": 10.0, "attack_ms": 10.0, "release_ms": 100.0}]"#,
    );
    let report = validate(&bad_thresh);
    assert!(
        report.findings.iter().any(|f| f.code == "E-DYNAMICS-RANGE"),
        "expected E-DYNAMICS-RANGE for gate threshold > 0: {:?}",
        report.findings
    );

    // Invalid attack_ms (out of 0.01..=1000)
    let bad_attack = basic_project(
        &dir,
        r#"[{"name": "gate", "threshold_db": -20.0, "attack_ms": 0.0, "release_ms": 100.0}]"#,
    );
    let report = validate(&bad_attack);
    assert!(
        report.findings.iter().any(|f| f.code == "E-DYNAMICS-RANGE"),
        "expected E-DYNAMICS-RANGE for gate attack <= 0: {:?}",
        report.findings
    );

    // Invalid release_ms (out of 1..=9000)
    let bad_release = basic_project(
        &dir,
        r#"[{"name": "gate", "threshold_db": -20.0, "attack_ms": 10.0, "release_ms": 10000.0}]"#,
    );
    let report = validate(&bad_release);
    assert!(
        report.findings.iter().any(|f| f.code == "E-DYNAMICS-RANGE"),
        "expected E-DYNAMICS-RANGE for gate release > 9000: {:?}",
        report.findings
    );
}

#[test]
fn parametric_equalizer_range_and_schema_validation() {
    let dir = tempdir(line!());

    // Valid equalizer bands (peaking, lowshelf, highshelf, bandpass)
    let valid_path = basic_project(
        &dir,
        r#"[
            {"name": "equalizer", "filter": "peaking", "frequency": 1000.0, "gain_db": 3.0, "q": 1.4},
            {"name": "equalizer", "filter": "lowshelf", "frequency_hz": 120.0, "gain_db": -2.0, "q": 0.7},
            {"name": "equalizer", "filter": "highshelf", "frequency": 8000.0, "gain_db": 4.0},
            {"name": "equalizer", "filter": "bandpass", "frequency": 500.0, "q": 2.0}
        ]"#,
    );
    let report = validate(&valid_path);
    assert!(
        !report.findings.iter().any(|f| f.code == "E-EQ-RANGE"),
        "valid parametric EQ bands should pass without E-EQ-RANGE: {:?}",
        report.findings
    );

    // Invalid frequency (out of 20..=20000)
    let bad_freq = basic_project(
        &dir,
        r#"[{"name": "equalizer", "filter": "peaking", "frequency": 10.0, "gain_db": 0.0, "q": 1.0}]"#,
    );
    let report = validate(&bad_freq);
    assert!(
        report.findings.iter().any(|f| f.code == "E-EQ-RANGE"),
        "expected E-EQ-RANGE for frequency < 20 Hz: {:?}",
        report.findings
    );

    // Invalid gain_db (out of -48..=24)
    let bad_gain = basic_project(
        &dir,
        r#"[{"name": "equalizer", "filter": "peaking", "frequency": 1000.0, "gain_db": 30.0, "q": 1.0}]"#,
    );
    let report = validate(&bad_gain);
    assert!(
        report.findings.iter().any(|f| f.code == "E-EQ-RANGE"),
        "expected E-EQ-RANGE for gain_db > 24 dB: {:?}",
        report.findings
    );

    // Invalid Q (out of 0.1..=10)
    let bad_q = basic_project(
        &dir,
        r#"[{"name": "equalizer", "filter": "peaking", "frequency": 1000.0, "gain_db": 0.0, "q": 12.0}]"#,
    );
    let report = validate(&bad_q);
    assert!(
        report.findings.iter().any(|f| f.code == "E-EQ-RANGE"),
        "expected E-EQ-RANGE for Q > 10: {:?}",
        report.findings
    );
}

#[test]
fn measure_scopes_computes_visual_histogram_and_vectorscope() {
    let dir = tempdir(line!());
    // Create a simple project with background color #336699 (R=51, G=102, B=153)
    let body = canonical(
        r##"{
            "frame": {"width": 100, "height": 100},
            "fps": 30,
            "background": "#336699",
            "duration": 1000,
            "output": "out.mp4",
            "tracks": []
        }"##,
    );
    let project = write_project(&dir, "p.montagent.json", &body);

    let answer = montagent_core::verbs::measure::measure(
        &project,
        &Ask {
            scopes: true,
            at: Some(0),
            ..Default::default()
        },
    );

    let json = answer.to_json();
    assert_eq!(json["measure"]["mode"], "scopes", "{json}");
    assert_eq!(json["measure"]["at_ms"], 0);

    let visual = &json["measure"]["visual_scopes"];
    let hist = &visual["histogram"];
    let luma = hist["luma"].as_array().expect("luma array");
    let red = hist["red"].as_array().expect("red array");
    let green = hist["green"].as_array().expect("green array");
    let blue = hist["blue"].as_array().expect("blue array");

    assert_eq!(luma.len(), 256);
    assert_eq!(red.len(), 256);
    assert_eq!(green.len(), 256);
    assert_eq!(blue.len(), 256);

    // Total pixel count is 100 x 100 = 10,000 pixels
    let total_pixels: u64 = 100 * 100;
    let sum_r: u64 = red.iter().filter_map(|v| v.as_u64()).sum();
    let sum_g: u64 = green.iter().filter_map(|v| v.as_u64()).sum();
    let sum_b: u64 = blue.iter().filter_map(|v| v.as_u64()).sum();
    let sum_y: u64 = luma.iter().filter_map(|v| v.as_u64()).sum();

    assert_eq!(sum_r, total_pixels);
    assert_eq!(sum_g, total_pixels);
    assert_eq!(sum_b, total_pixels);
    assert_eq!(sum_y, total_pixels);

    // All pixels are R=51, G=102, B=153
    assert_eq!(red[51].as_u64().unwrap(), total_pixels);
    assert_eq!(green[102].as_u64().unwrap(), total_pixels);
    assert_eq!(blue[153].as_u64().unwrap(), total_pixels);

    // Vectorscope is a 64x64 grid
    let vectorscope = visual["vectorscope"].as_array().expect("vectorscope array");
    assert_eq!(vectorscope.len(), 64);
    let mut vec_sum: u64 = 0;
    for row in vectorscope {
        let cols = row.as_array().expect("row is array");
        assert_eq!(cols.len(), 64);
        for cell in cols {
            vec_sum += cell.as_u64().unwrap();
        }
    }
    assert_eq!(vec_sum, total_pixels);

    // Verify wire text output
    let text = montagent_core::wire::render_measure(&answer, Wire::Text { verbose: false });
    assert!(text.contains("MEASURE  scopes at 0 ms"), "text: {text}");
    assert!(
        text.contains(
            "visual scopes: histogram 256 bins (luma, red, green, blue), vectorscope 64x64 grid"
        ),
        "text: {text}"
    );
}

#[test]
fn measure_scopes_mutual_exclusivity() {
    let dir = tempdir(line!());
    let body = canonical(
        r##"{
            "frame": {"width": 100, "height": 100},
            "fps": 30,
            "background": "#000000",
            "duration": 1000,
            "output": "out.mp4",
            "tracks": []
        }"##,
    );
    let project = write_project(&dir, "p.montagent.json", &body);

    // scopes + all is rejected
    let answer = montagent_core::verbs::measure::measure(
        &project,
        &Ask {
            scopes: true,
            all: true,
            ..Default::default()
        },
    );
    assert!(answer.to_json()["measure"].is_null());
    assert_eq!(
        answer.report().exit_code(),
        montagent_core::report::ExitCode::BadInvocation
    );

    // scopes + element is rejected
    let answer = montagent_core::verbs::measure::measure(
        &project,
        &Ask {
            scopes: true,
            element: Some(json!({"type": "text", "font": "brand", "size": 20})),
            ..Default::default()
        },
    );
    assert!(answer.to_json()["measure"].is_null());
    assert_eq!(
        answer.report().exit_code(),
        montagent_core::report::ExitCode::BadInvocation
    );
}
